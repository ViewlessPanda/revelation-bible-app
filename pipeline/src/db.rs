//! Stage 4 (write) and the stage 4–6 rule (DESIGN.md 2, 8, 10.1; M1-F1, M1-F2):
//! the schema as one SQL constant, the writer, and `publish`, which builds a
//! `create_new` temp file, checks **that file** through `open_verified`, and
//! only then writes the checksum and renames both into place.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};

use crate::commentary::{CmtModel, CmtNodeKind};
use crate::error::{Error, Failure, Result};
use crate::lexicon::{LexModel, NodeKind};
use crate::open::open_verified;
use crate::osis::Model;

/// The kjv.sqlite schema, exactly as DESIGN.md section 8 gives it. The check
/// stage compares `sqlite_schema` with what this text creates.
pub const SCHEMA: &str = "\
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE source (
  id     TEXT PRIMARY KEY,                                  -- SOURCES.md ID, e.g. 'BIB-KJV-CW31'
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;

CREATE TABLE book (
  ord      INTEGER PRIMARY KEY CHECK (ord BETWEEN 1 AND 66),
  osis     TEXT NOT NULL UNIQUE,                            -- 'Gen' (as in the source)
  usfm     TEXT NOT NULL UNIQUE,                            -- 'GEN' (as in eng.vrs)
  chapters INTEGER NOT NULL CHECK (chapters >= 1),
  main_title TEXT CHECK (main_title IS NULL OR length(main_title) BETWEEN 1 AND 256)
                                                            -- the main title element, raw character data
) STRICT;

CREATE TABLE verse (
  id      INTEGER PRIMARY KEY,                              -- book*1000000 + chapter*1000 + verse
  book    INTEGER NOT NULL REFERENCES book(ord),
  chapter INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  verse   INTEGER NOT NULL CHECK (verse BETWEEN 1 AND 999),
  text    TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 4096),   -- stored text, section 3
  CHECK (id = book * 1000000 + chapter * 1000 + verse),
  UNIQUE (book, chapter, verse)
) STRICT;

CREATE TABLE mark (                                         -- section 4
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),                 -- document order within the verse
  kind     TEXT NOT NULL CHECK (kind IN ('added', 'divine')),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE red_piece (                                    -- section 5
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  q_ord    INTEGER NOT NULL CHECK (q_ord >= 1),             -- which <q> in the source, 1-based
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX red_piece_by_q ON red_piece (q_ord, verse_id);

CREATE TABLE word (                                         -- section 6; zero length allowed
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp <= end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE word_key (
  verse_id INTEGER NOT NULL,
  n        INTEGER NOT NULL,
  k        INTEGER NOT NULL CHECK (k >= 0),                 -- token order within the lemma
  key      TEXT NOT NULL CHECK (key GLOB '[HG][1-9]*' AND key NOT GLOB '?*[^0-9]*'
                                AND length(key) BETWEEN 2 AND 6),   -- canonical, section 6.2
  PRIMARY KEY (verse_id, n, k),
  FOREIGN KEY (verse_id, n) REFERENCES word (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX word_key_by_key ON word_key (key, verse_id);
";

/// The content.sqlite schema (lexicons, DESIGN.md 7 and 8): `meta` and
/// `source` as in kjv.sqlite, then one tree per lexicon entry.
pub const LEX_SCHEMA: &str = "\
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE source (
  id     TEXT PRIMARY KEY,                                  -- SOURCES.md ID, e.g. 'LEX-HEB-OSHB'
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;

CREATE TABLE lex_entry (
  key     TEXT PRIMARY KEY CHECK (key GLOB '[HG][1-9]*' AND key NOT GLOB '?*[^0-9]*'
                                  AND length(key) BETWEEN 2 AND 6),   -- canonical: 'H1', 'G1'
  lexicon TEXT NOT NULL CHECK (lexicon IN ('LEX-HEB-OSHB', 'LEX-GRC-MGNT19')),
  ord     INTEGER NOT NULL CHECK (ord >= 1),                -- document order in its file
  CHECK (substr(key, 1, 1) = CASE lexicon WHEN 'LEX-HEB-OSHB' THEN 'H' ELSE 'G' END)
) STRICT, WITHOUT ROWID;

CREATE TABLE lex_node (                                     -- section 7
  key    TEXT NOT NULL REFERENCES lex_entry(key),
  n      INTEGER NOT NULL CHECK (n >= 0),                   -- document order within the entry
  parent INTEGER,                                           -- NULL = direct child of <entry>
  name   TEXT,                                              -- element name, or NULL for text
  text   TEXT,                                              -- text bytes, or NULL for an element
  CHECK ((name IS NULL) <> (text IS NULL)),
  CHECK (parent IS NULL OR parent < n),
  PRIMARY KEY (key, n),
  FOREIGN KEY (key, parent) REFERENCES lex_node (key, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE lex_attr (
  key     TEXT NOT NULL,
  n       INTEGER NOT NULL,
  i       INTEGER NOT NULL CHECK (i >= 0),                  -- order within the element's start tag
  name    TEXT NOT NULL,
  value   TEXT NOT NULL,                                    -- exactly as in the source
  ref_key TEXT,                                             -- canonical target for cross-references
  PRIMARY KEY (key, n, name),
  FOREIGN KEY (key, n) REFERENCES lex_node (key, n)
) STRICT, WITHOUT ROWID;
";

/// A commentary database, one file per commentary source (DESIGN.md 17.5):
/// `meta` and `source` as above, then the anchors and the block trees.
pub const CMT_SCHEMA: &str = "\
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE source (
  id     TEXT PRIMARY KEY,                                  -- SOURCES.md ID, e.g. 'CMT-MHCC-CW20'
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;

CREATE TABLE cmt_anchor (                                   -- section 17.4: <verse sID/> ... <verse eID/>
  id  INTEGER PRIMARY KEY CHECK (id >= 1),                  -- document order, 1-based
  sid TEXT NOT NULL CHECK (length(sid) BETWEEN 5 AND 32)    -- the sID as written (not unique in MHCC)
) STRICT;

CREATE TABLE cmt_anchor_verse (
  anchor   INTEGER NOT NULL REFERENCES cmt_anchor(id),
  k        INTEGER NOT NULL CHECK (k >= 0),                 -- position in the osisID list
  verse_id INTEGER NOT NULL CHECK (verse_id BETWEEN 1001001 AND 66999999
                                   AND verse_id % 1000 >= 1 AND verse_id / 1000 % 1000 >= 1),
                                                            -- kjv.sqlite verse.id
  PRIMARY KEY (anchor, k)
) STRICT, WITHOUT ROWID;
CREATE INDEX cmt_anchor_verse_by_verse ON cmt_anchor_verse (verse_id, anchor);

CREATE TABLE cmt_block (                                    -- a <p>, <title> or <table> in a book
  id      INTEGER PRIMARY KEY CHECK (id >= 1),              -- document order, 1-based
  book    INTEGER NOT NULL CHECK (book BETWEEN 1 AND 66),   -- kjv.sqlite book.ord
  chapter INTEGER CHECK (chapter IS NULL OR chapter BETWEEN 1 AND 150),
                                                            -- open at the block's start; NULL before the first
  anchor  INTEGER REFERENCES cmt_anchor(id)                 -- NULL: outside every verse anchor
) STRICT;
CREATE INDEX cmt_block_by_anchor ON cmt_block (anchor);
CREATE INDEX cmt_block_by_chapter ON cmt_block (book, chapter);

CREATE TABLE cmt_node (
  block  INTEGER NOT NULL REFERENCES cmt_block(id),
  n      INTEGER NOT NULL CHECK (n >= 0),                   -- document order; 0 = the block element
  parent INTEGER,                                           -- NULL only for n = 0
  name   TEXT,                                              -- element name, or NULL for text
  text   TEXT,                                              -- text exactly ('&amp;' stored as '&'), or NULL
  CHECK ((name IS NULL) <> (text IS NULL)),
  CHECK ((n = 0) = (parent IS NULL)),
  CHECK (n > 0 OR name IN ('p', 'title', 'table')),
  CHECK (parent IS NULL OR parent < n),
  PRIMARY KEY (block, n),
  FOREIGN KEY (block, parent) REFERENCES cmt_node (block, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE cmt_attr (
  block     INTEGER NOT NULL,
  n         INTEGER NOT NULL,
  i         INTEGER NOT NULL CHECK (i >= 0),                -- order within the start tag
  name      TEXT NOT NULL,
  value     TEXT NOT NULL,                                  -- exactly as in the source
  ref_start INTEGER,                                        -- reference@osisRef only: KJV verse id of
  ref_end   INTEGER,                                        -- its first and last verse, as written
  CHECK ((ref_start IS NULL) = (ref_end IS NULL)),
  PRIMARY KEY (block, n, name),
  FOREIGN KEY (block, n) REFERENCES cmt_node (block, n)
) STRICT, WITHOUT ROWID;
";

/// Output file name of the lexicon database.
pub const LEX_DEST: &str = "content.sqlite";

/// `application_id` = "RVLN".
pub const APPLICATION_ID: i64 = 0x5256_4C4E;
pub const SCHEMA_VERSION: i64 = 1;

/// One `source` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRow {
    pub id: String,
    pub file: String,
    pub size: i64,
    pub sha256: String,
}

/// Everything written to `meta` and `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    /// Sorted by key (the table is WITHOUT ROWID, ordered by key anyway).
    pub entries: Vec<(String, String)>,
    pub sources: Vec<SourceRow>,
}

impl Meta {
    /// The KJV meta rows for `model`. `has_red_letters` is derived from the
    /// model (T10): it is `1` only when the source had `q who="Jesus"`.
    /// Nothing from the build environment (commit, time, host) is stored, so
    /// the file is a pure function of the pinned inputs and the pipeline code
    /// (DESIGN.md 10.1); provenance goes to the CLI output and the header of
    /// `pipeline/tests/expected/output-sha256.txt`.
    pub fn kjv(
        model: &Model,
        red_source: &str,
        sources: Vec<SourceRow>,
        overrides_sha256: Option<&str>,
    ) -> Meta {
        let has_red = model.verses.iter().any(|v| !v.red.is_empty());
        let mut e: Vec<(String, String)> = vec![
            ("schema_version".into(), SCHEMA_VERSION.to_string()),
            ("translation_abbrev".into(), "KJV".into()),
            (
                "has_red_letters".into(),
                if has_red { "1" } else { "0" }.into(),
            ),
            (
                "versification".into(),
                "VRS-ENG-PALASO + KJV overrides".into(),
            ),
        ];
        if has_red {
            e.push(("red_letter_source".into(), red_source.into()));
        }
        if let Some(h) = overrides_sha256 {
            e.push(("versification_overrides_sha256".into(), h.into()));
        }
        e.sort();
        Meta {
            entries: e,
            sources,
        }
    }
}

impl Meta {
    /// The content.sqlite meta rows: the schema version and the sources.
    pub fn lexicons(sources: Vec<SourceRow>) -> Meta {
        Meta {
            entries: vec![("schema_version".into(), SCHEMA_VERSION.to_string())],
            sources,
        }
    }

    /// A commentary database (DESIGN.md 17.5): its source ID, the verse-id
    /// scheme its anchors use, and the scholar's label rows (DESIGN.md 17.7,
    /// `labels::parse_labels` plus the label file's path and SHA-256), which
    /// are empty for synthetic fixtures and never filled in with placeholders.
    pub fn commentary(
        source_id: &str,
        sources: Vec<SourceRow>,
        overrides_sha256: Option<&str>,
        labels: Vec<(String, String)>,
    ) -> Meta {
        let mut e: Vec<(String, String)> = vec![
            ("schema_version".into(), SCHEMA_VERSION.to_string()),
            ("commentary_source".into(), source_id.into()),
            (
                "verse_ids".into(),
                "kjv.sqlite verse.id (VRS-ENG-PALASO + KJV overrides)".into(),
            ),
        ];
        if let Some(h) = overrides_sha256 {
            e.push(("versification_overrides_sha256".into(), h.into()));
        }
        e.extend(labels);
        e.sort();
        Meta {
            entries: e,
            sources,
        }
    }
}

/// `REVELATION_PIPELINE_COMMIT` (DESIGN.md 10.1): 40 lowercase hex digits,
/// optionally followed by `-dirty`. Unset or malformed gives `unknown-dirty`.
/// Build provenance only: printed by the CLI and recorded in the header of
/// `output-sha256.txt`, never written into a content database.
pub fn commit_from(value: Option<&str>) -> String {
    let ok = |s: &str| s.len() == 40 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    match value {
        Some(v) if ok(v.strip_suffix("-dirty").unwrap_or(v)) => v.to_string(),
        _ => "unknown-dirty".to_string(),
    }
}

/// The commit a build reports (DESIGN.md 10.1), from the value of
/// `REVELATION_PIPELINE_COMMIT`. A release build (`REVELATION_RELEASE=1`)
/// refuses anything ending in `-dirty`, including the `unknown-dirty` of an
/// unset or malformed value (M1-F12).
pub fn build_commit(value: Option<&str>, release: bool) -> std::result::Result<String, Error> {
    let c = commit_from(value);
    if release && c.ends_with("-dirty") {
        return Err(Error::new(
            "release",
            format!("release builds refuse pipeline_commit '{c}'"),
        ));
    }
    Ok(c)
}

fn sql(check: &'static str) -> impl Fn(rusqlite::Error) -> Error {
    move |e| Error::new(check, format!("SQLite: {e}"))
}

/// Sets the file conventions (section 8) on an empty database, creates the
/// schema and inserts everything in document order in one transaction.
pub fn write(conn: &mut Connection, m: &Model, meta: &Meta) -> Result<()> {
    let w = sql("write");
    setup(conn)?;
    let tx = conn.transaction().map_err(&w)?;
    tx.execute_batch(SCHEMA).map_err(&w)?;
    insert_meta(&tx, meta)?;
    {
        let mut st = tx
            .prepare(
                "INSERT INTO book (ord, osis, usfm, chapters, main_title) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(&w)?;
        for b in &m.books {
            st.execute(params![b.ord, b.osis, b.usfm, b.chapters, b.main_title])
                .map_err(&w)?;
        }
        let mut sv = tx
            .prepare(
                "INSERT INTO verse (id, book, chapter, verse, text) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(&w)?;
        let mut sm = tx
            .prepare("INSERT INTO mark (verse_id, n, kind, start_cp, end_cp) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(&w)?;
        let mut sr = tx
            .prepare("INSERT INTO red_piece (verse_id, n, start_cp, end_cp, q_ord) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(&w)?;
        let mut sw = tx
            .prepare("INSERT INTO word (verse_id, n, start_cp, end_cp) VALUES (?1, ?2, ?3, ?4)")
            .map_err(&w)?;
        let mut sk = tx
            .prepare("INSERT INTO word_key (verse_id, n, k, key) VALUES (?1, ?2, ?3, ?4)")
            .map_err(&w)?;
        for v in &m.verses {
            let id = v.id();
            sv.execute(params![id, v.book, v.chapter, v.verse, v.text])
                .map_err(&w)?;
            for (n, mk) in v.marks.iter().enumerate() {
                sm.execute(params![id, n as i64, mk.kind.as_str(), mk.start, mk.end])
                    .map_err(&w)?;
            }
            // The only place red_piece rows are written: from pieces the parser
            // made from <q who="Jesus"> (T10).
            for (n, r) in v.red.iter().enumerate() {
                sr.execute(params![id, n as i64, r.start, r.end, r.q_ord])
                    .map_err(&w)?;
            }
            for (n, wd) in v.words.iter().enumerate() {
                sw.execute(params![id, n as i64, wd.start, wd.end])
                    .map_err(&w)?;
                for (k, key) in wd.keys.iter().enumerate() {
                    sk.execute(params![id, n as i64, k as i64, key.to_string()])
                        .map_err(&w)?;
                }
            }
        }
    }
    tx.commit().map_err(&w)?;
    seal(conn)
}

/// Writes content.sqlite: every entry of every lexicon, in file order, as
/// `lex_entry` + `lex_node` + `lex_attr` rows (DESIGN.md 7).
pub fn write_lex(conn: &mut Connection, lexicons: &[&LexModel], meta: &Meta) -> Result<()> {
    let w = sql("write");
    setup(conn)?;
    let tx = conn.transaction().map_err(&w)?;
    tx.execute_batch(LEX_SCHEMA).map_err(&w)?;
    insert_meta(&tx, meta)?;
    {
        let mut se = tx
            .prepare("INSERT INTO lex_entry (key, lexicon, ord) VALUES (?1, ?2, ?3)")
            .map_err(&w)?;
        let mut sn = tx
            .prepare(
                "INSERT INTO lex_node (key, n, parent, name, text) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(&w)?;
        let mut sa = tx
            .prepare(
                "INSERT INTO lex_attr (key, n, i, name, value, ref_key) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(&w)?;
        for m in lexicons {
            for e in &m.entries {
                let key = e.key.to_string();
                se.execute(params![key, m.source_id, e.ord]).map_err(&w)?;
                for (n, node) in e.nodes.iter().enumerate() {
                    let (name, text, attrs) = match &node.kind {
                        NodeKind::Element { name, attrs } => {
                            (Some(name.as_str()), None, &attrs[..])
                        }
                        NodeKind::Text(t) => (None, Some(t.as_str()), &[][..]),
                    };
                    sn.execute(params![key, n as i64, node.parent, name, text])
                        .map_err(&w)?;
                    for (i, a) in attrs.iter().enumerate() {
                        sa.execute(params![
                            key,
                            n as i64,
                            i as i64,
                            a.name,
                            a.value,
                            a.ref_key.map(|k| k.to_string())
                        ])
                        .map_err(&w)?;
                    }
                }
            }
        }
    }
    tx.commit().map_err(&w)?;
    seal(conn)
}

/// Stage 4 for a commentary database: anchors (with their verses) first,
/// then each block, its nodes and attributes, all in document order.
pub fn write_cmt(conn: &mut Connection, m: &CmtModel, meta: &Meta) -> Result<()> {
    let w = sql("write");
    setup(conn)?;
    let tx = conn.transaction().map_err(&w)?;
    tx.execute_batch(CMT_SCHEMA).map_err(&w)?;
    insert_meta(&tx, meta)?;
    {
        let mut sa = tx
            .prepare("INSERT INTO cmt_anchor (id, sid) VALUES (?1, ?2)")
            .map_err(&w)?;
        let mut sv = tx
            .prepare("INSERT INTO cmt_anchor_verse (anchor, k, verse_id) VALUES (?1, ?2, ?3)")
            .map_err(&w)?;
        for (i, a) in m.anchors.iter().enumerate() {
            let id = i as i64 + 1;
            sa.execute(params![id, a.sid]).map_err(&w)?;
            for (k, v) in a.verses.iter().enumerate() {
                sv.execute(params![id, k as i64, v]).map_err(&w)?;
            }
        }
        let mut sb = tx
            .prepare("INSERT INTO cmt_block (id, book, chapter, anchor) VALUES (?1, ?2, ?3, ?4)")
            .map_err(&w)?;
        let mut sn = tx
            .prepare(
                "INSERT INTO cmt_node (block, n, parent, name, text) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(&w)?;
        let mut st = tx
            .prepare(
                "INSERT INTO cmt_attr (block, n, i, name, value, ref_start, ref_end) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(&w)?;
        for (i, b) in m.blocks.iter().enumerate() {
            let id = i as i64 + 1;
            sb.execute(params![id, b.book, b.chapter, b.anchor])
                .map_err(&w)?;
            for (n, node) in b.nodes.iter().enumerate() {
                let (name, text, attrs) = match &node.kind {
                    CmtNodeKind::Element { name, attrs } => (Some(name.as_str()), None, &attrs[..]),
                    CmtNodeKind::Text(t) => (None, Some(t.as_str()), &[][..]),
                };
                sn.execute(params![id, n as i64, node.parent, name, text])
                    .map_err(&w)?;
                for (k, a) in attrs.iter().enumerate() {
                    st.execute(params![
                        id,
                        n as i64,
                        k as i64,
                        a.name,
                        a.value,
                        a.refs.map(|r| r.0),
                        a.refs.map(|r| r.1)
                    ])
                    .map_err(&w)?;
                }
            }
        }
    }
    tx.commit().map_err(&w)?;
    seal(conn)
}

/// The file conventions of section 8, set on an empty database before any
/// table: rollback journal (never WAL), FULL sync, foreign keys on.
pub(crate) fn setup(conn: &mut Connection) -> Result<()> {
    let w = sql("write");
    let mode: String = conn
        .query_row("PRAGMA journal_mode = DELETE", [], |r| r.get(0))
        .map_err(&w)?;
    if mode != "delete" && mode != "memory" {
        return Err(Error::new("write", format!("journal_mode is '{mode}', not 'delete'")).into());
    }
    conn.execute_batch(&format!(
        "PRAGMA synchronous = FULL;
         PRAGMA foreign_keys = ON;
         PRAGMA page_size = 4096;
         PRAGMA auto_vacuum = NONE;
         PRAGMA encoding = 'UTF-8';
         PRAGMA application_id = {APPLICATION_ID};
         PRAGMA user_version = {SCHEMA_VERSION};"
    ))
    .map_err(&w)?;
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .map_err(&w)?;
    if fk != 1 {
        return Err(Error::new("write", "foreign_keys could not be turned on").into());
    }
    Ok(())
}

pub(crate) fn insert_meta(tx: &rusqlite::Transaction<'_>, meta: &Meta) -> Result<()> {
    let w = sql("write");
    let mut st = tx
        .prepare("INSERT INTO meta (key, value) VALUES (?1, ?2)")
        .map_err(&w)?;
    for (k, v) in &meta.entries {
        st.execute(params![k, v]).map_err(&w)?;
    }
    let mut st = tx
        .prepare("INSERT INTO source (id, file, size, sha256) VALUES (?1, ?2, ?3, ?4)")
        .map_err(&w)?;
    for s in &meta.sources {
        st.execute(params![s.id, s.file, s.size, s.sha256])
            .map_err(&w)?;
    }
    Ok(())
}

/// After the commit: integrity_check, foreign_key_check, VACUUM.
pub(crate) fn seal(conn: &mut Connection) -> Result<()> {
    let w = sql("write");
    let ic: Vec<String> = query_strings(conn, "PRAGMA integrity_check").map_err(&w)?;
    if ic != ["ok"] {
        return Err(Error::new("integrity", format!("integrity_check: {ic:?}")).into());
    }
    let fk: Vec<String> = query_strings(conn, "PRAGMA foreign_key_check").map_err(&w)?;
    if !fk.is_empty() {
        return Err(Error::new("fk", format!("foreign_key_check: {} rows", fk.len())).into());
    }
    conn.execute_batch("VACUUM").map_err(&w)?;
    Ok(())
}

/// Every row's first column as text (or the row count, for queries whose
/// first column isn't text).
pub fn query_strings(conn: &Connection, q: &str) -> rusqlite::Result<Vec<String>> {
    let mut st = conn.prepare(q)?;
    let rows = st.query_map([], |r| {
        Ok(match r.get_ref(0)? {
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
            other => format!("{other:?}"),
        })
    })?;
    rows.collect()
}

/// The steps of the stage 4–6 rule (DESIGN.md 2). The hook runs after each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Cleanup,
    CreateTemp,
    Write,
    Flush,
    Check,
    Checksum,
    /// Between the two renames: the new database is in place, the old
    /// checksum file still beside it.
    DbRenamed,
    Publish,
}

pub const STEPS: [Step; 8] = [
    Step::Cleanup,
    Step::CreateTemp,
    Step::Write,
    Step::Flush,
    Step::Check,
    Step::Checksum,
    Step::DbRenamed,
    Step::Publish,
];

const SIDECARS: [&str; 3] = ["-journal", "-wal", "-shm"];

fn io(check: &'static str, what: &str) -> impl Fn(std::io::Error) -> Error {
    let what = what.to_string();
    move |e| Error::new(check, format!("{what}: {e}"))
}

fn is_hex16(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether `name` is a leftover temp of `dest` (step 0): exactly
/// `<dest>.tmp-<16 hex>` or `<dest>.sha256.tmp-<16 hex>`, optionally followed
/// by `-journal`, `-wal` or `-shm`.
pub fn is_leftover(dest: &str, name: &str) -> bool {
    for prefix in [format!("{dest}.tmp-"), format!("{dest}.sha256.tmp-")] {
        if let Some(rest) = name.strip_prefix(&prefix) {
            let rest = SIDECARS
                .iter()
                .find_map(|s| rest.strip_suffix(s))
                .unwrap_or(rest);
            if is_hex16(rest) {
                return true;
            }
        }
    }
    false
}

fn unique_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let pid = u64::from(std::process::id());
    format!("{:016x}", (pid << 40) ^ nanos)
}

fn remove_temp(path: &Path) {
    let _ = fs::remove_file(path);
    for s in SIDECARS {
        let mut p = path.as_os_str().to_owned();
        p.push(s);
        let _ = fs::remove_file(PathBuf::from(p));
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut p = path.as_os_str().to_owned();
    p.push(suffix);
    PathBuf::from(p)
}

/// The sidecar files of `path` that exist.
pub fn existing_sidecars(path: &Path) -> Vec<PathBuf> {
    SIDECARS
        .iter()
        .map(|s| with_suffix(path, s))
        .filter(|p| fs::symlink_metadata(p).is_ok())
        .collect()
}

/// Stages 4–6 as one rule. `write` fills the new database; `check` gets the
/// connection `open_verified` made from the temp file's bytes, and their
/// SHA-256. Returns the published SHA-256. `hook` runs after every step and
/// can fail it (T21; the CLI passes one that does nothing).
pub fn publish<W, C>(
    out_dir: &Path,
    dest: &str,
    write: W,
    check: C,
    hook: &mut dyn FnMut(Step) -> Result<()>,
) -> Result<String>
where
    W: FnOnce(&mut Connection) -> Result<()>,
    C: FnOnce(&Connection, &str) -> Result<()>,
{
    crate::sources::check_plain(out_dir, true)?;
    // Step 0: clean up leftovers of earlier crashes; nothing else is deleted.
    for e in fs::read_dir(out_dir).map_err(io("io", "read out-dir"))? {
        let e = e.map_err(io("io", "read out-dir"))?;
        let name = e.file_name().to_string_lossy().into_owned();
        if is_leftover(dest, &name) {
            fs::remove_file(e.path()).map_err(io("io", "remove leftover temp"))?;
        }
    }
    hook(Step::Cleanup)?;

    let suffix = unique_suffix();
    let temp = out_dir.join(format!("{dest}.tmp-{suffix}"));
    let sum_temp = out_dir.join(format!("{dest}.sha256.tmp-{suffix}"));
    let dest_path = out_dir.join(dest);
    let sum_path = out_dir.join(format!("{dest}.sha256"));

    let fail = |e: Failure| -> Failure {
        remove_temp(&temp);
        let _ = fs::remove_file(&sum_temp);
        e
    };

    // Step 1: create_new, so an existing or planted file is never reused.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(io("io", "create temp database"))?;
    hook(Step::CreateTemp).map_err(fail)?;

    // Step 2: write, integrity and FK checks, VACUUM, close.
    let step2 = || -> Result<()> {
        let mut conn = Connection::open(&temp).map_err(sql("write"))?;
        write(&mut conn)?;
        conn.close()
            .map_err(|(_, e)| Error::new("write", format!("close: {e}")))?;
        Ok(())
    };
    step2().map_err(fail)?;
    hook(Step::Write).map_err(fail)?;

    // Step 3: no sidecars, then flush.
    let step3 = || -> Result<()> {
        let side = existing_sidecars(&temp);
        if !side.is_empty() {
            return Err(
                Error::new("sidecar", format!("sidecar beside the temp file: {side:?}")).into(),
            );
        }
        OpenOptions::new()
            .write(true)
            .open(&temp)
            .and_then(|f| f.sync_all())
            .map_err(io("io", "flush temp database"))?;
        Ok(())
    };
    step3().map_err(fail)?;
    hook(Step::Flush).map_err(fail)?;

    // Step 4: check the temp file, through open_verified.
    let step4 = || -> Result<String> {
        let (conn, hash) = open_verified(&temp, None)?;
        check(&conn, &hash)?;
        Ok(hash)
    };
    let hash = step4().map_err(fail)?;
    hook(Step::Check).map_err(fail)?;

    // Step 5: the checksum, from the hash of the bytes just checked.
    let step5 = || -> Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&sum_temp)
            .map_err(io("io", "create temp checksum"))?;
        f.write_all(format!("{hash}  {dest}\n").as_bytes())
            .and_then(|_| f.sync_all())
            .map_err(io("io", "write temp checksum"))?;
        Ok(())
    };
    step5().map_err(fail)?;
    hook(Step::Checksum).map_err(fail)?;

    // Step 6: publish. Refuse (never delete) while a sidecar of <dest> exists.
    let side = existing_sidecars(&dest_path);
    if !side.is_empty() {
        let names: Vec<String> = side
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        return Err(fail(
            Error::new(
                "sidecar",
                format!("refusing to publish: remove {names:?} first"),
            )
            .into(),
        ));
    }
    fs::rename(&temp, &dest_path)
        .map_err(io("publish", "rename database"))
        .map_err(|e| fail(e.into()))?;
    hook(Step::DbRenamed).map_err(fail)?;
    fs::rename(&sum_temp, &sum_path)
        .map_err(io("publish", "rename checksum"))
        .map_err(|e| fail(e.into()))?;
    #[cfg(unix)]
    {
        fs::File::open(out_dir)
            .and_then(|d| d.sync_all())
            .map_err(io("publish", "sync out-dir"))?;
    }
    hook(Step::Publish)?;
    Ok(hash)
}
