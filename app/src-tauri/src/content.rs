//! Read-only content (pipeline/DESIGN.md 8 and 10.3).
//!
//! Every content file is opened ONLY through [`open_content`], which calls the
//! one shared `revelation_content::open_verified` (crates/revelation-content,
//! also used by the pipeline): the whole file is read once into memory (with
//! a size cap), its SHA-256 is compared with a value compiled into the app
//! (never one read from beside the file), and that same buffer is handed to
//! `sqlite3_deserialize`, read-only, with `PRAGMA query_only = 1`. No `-wal`,
//! `-shm` or `-journal` file next to the database is ever consulted, and there
//! is no gap between check and use.
//!
//! The app has no code path that writes to content.

use crate::books;
use crate::runs::{self, Mark, MarkKind, RedPiece, Run, Word};
pub use revelation_content::{
    APPLICATION_ID, MAX_DB as MAX_CONTENT_BYTES, OpenError, SCHEMA_VERSION, sha256_hex,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use std::fmt;
use std::path::Path;

/// Open a content file: `open_verified` against `expected_sha256` (at most
/// `cap` bytes), then require the content identity (application_id "RVLN",
/// schema version 1).
pub fn open_content(path: &Path, expected_sha256: &str, cap: u64) -> Result<Connection, OpenError> {
    let (conn, _) = revelation_content::open_verified(path, Some(expected_sha256), cap)?;
    revelation_content::require_content_identity(&conn)?;
    Ok(conn)
}

fn sql_err(e: rusqlite::Error) -> OpenError {
    OpenError::Sqlite(e.to_string())
}

// ---------------------------------------------------------------------------
// Bible (kjv.sqlite schema, DESIGN.md 8)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BookInfo {
    pub ord: u32,
    pub osis: String,
    /// Reader-facing short name (books.rs: the scholar's cited table).
    pub name: String,
    /// Name used in chapter headings (books.rs; "Psalm" for the Psalms).
    pub heading: String,
    /// `book.main_title`, the source's main title, exactly as stored (NULL in
    /// fixtures).
    pub main_title: Option<String>,
    pub abbrevs: Vec<String>,
    pub chapters: u32,
    /// Highest verse number of each chapter, index 0 = chapter 1.
    pub verses: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VerseOut {
    pub v: u32,
    /// Exact stored text (used for copy; the runs concatenate to it).
    pub text: String,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChapterOut {
    pub book: u32,
    pub osis: String,
    pub chapter: u32,
    pub verses: Vec<VerseOut>,
}

/// Reader-facing translation names, keyed by `meta.translation_abbrev`.
///
/// "King James Version (1769)": owner-approved 2026-10-22. Wording and
/// citations: docs/scholar/kjv-label.md section 1 (the pinned source's own
/// work title, `kjv.osis.xml` `<work osisWork="KJV"><title>`, without its
/// tagging description; facts 4 and 6 in section 2).
///
/// This is content metadata. It moves into the content DB's
/// `meta.translation_name` in the pipeline later (pipeline/DESIGN.md 8); a
/// `translation_name` in the DB already takes precedence over this table.
pub const TRANSLATION_LABELS: &[(&str, &str)] = &[("KJV", "King James Version (1769)")];

/// The label for `abbrev` from [`TRANSLATION_LABELS`], if there is one.
pub fn translation_label(abbrev: &str) -> Option<&'static str> {
    TRANSLATION_LABELS
        .iter()
        .find(|(a, _)| *a == abbrev)
        .map(|(_, name)| *name)
}

pub struct Bible {
    conn: Connection,
    pub abbrev: String,
    pub name: String,
    pub has_red_letters: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    NotFound,
    Content(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::NotFound => write!(f, "not found"),
            QueryError::Content(e) => write!(f, "{e}"),
        }
    }
}

fn q_err(e: rusqlite::Error) -> QueryError {
    QueryError::Content(e.to_string())
}

impl Bible {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Bible, OpenError> {
        let conn = open_content(path, expected_sha256, MAX_CONTENT_BYTES)?;
        Bible::from_conn(conn)
    }

    fn from_conn(conn: Connection) -> Result<Bible, OpenError> {
        let meta = |k: &str| -> Result<Option<String>, OpenError> {
            conn.query_row("SELECT value FROM meta WHERE key = ?1", [k], |r| r.get(0))
                .optional()
                .map_err(sql_err)
        };
        let abbrev = meta("translation_abbrev")?
            .ok_or_else(|| OpenError::NotContent("no translation_abbrev".into()))?;
        let name = match meta("translation_name")? {
            Some(n) => n,
            None => translation_label(&abbrev).unwrap_or(&abbrev).to_string(),
        };
        let has_red_letters = match meta("has_red_letters")?.as_deref() {
            Some("1") => true,
            Some("0") | None => false,
            Some(_) => {
                return Err(OpenError::NotContent(
                    "has_red_letters is not 0 or 1".into(),
                ));
            }
        };
        Ok(Bible {
            conn,
            abbrev,
            name,
            has_red_letters,
        })
    }

    /// Whether this translation has `book` (OSIS ID, case-sensitive) chapter
    /// `chapter`, and, if `verse` is given, that verse (D5-F6).
    pub fn has(&self, book: &str, chapter: u32, verse: Option<u32>) -> Result<bool, QueryError> {
        self.conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM verse v JOIN book b ON b.ord = v.book
                 WHERE b.osis = ?1 AND v.chapter = ?2 AND (?3 IS NULL OR v.verse = ?3))",
                params![book, chapter, verse],
                |r| r.get(0),
            )
            .map_err(q_err)
    }

    pub fn books(&self) -> Result<Vec<BookInfo>, QueryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT ord, osis, chapters, main_title FROM book ORDER BY ord")
            .map_err(q_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u32>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(q_err)?;
        let mut vstmt = self
            .conn
            .prepare("SELECT chapter, max(verse) FROM verse WHERE book = ?1 GROUP BY chapter ORDER BY chapter")
            .map_err(q_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (ord, osis, chapters, main_title) = row.map_err(q_err)?;
            let mut verses = vec![0u32; chapters as usize];
            let counts = vstmt
                .query_map([ord], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, u32>(1)?)))
                .map_err(q_err)?;
            for c in counts {
                let (ch, max) = c.map_err(q_err)?;
                if ch == 0 || ch > chapters {
                    return Err(QueryError::Content(format!(
                        "verse in chapter {ch} beyond book {ord}'s {chapters}"
                    )));
                }
                verses[(ch - 1) as usize] = max;
            }
            let books::Names {
                name,
                heading,
                abbrevs,
            } = books::names(&osis);
            out.push(BookInfo {
                ord,
                osis,
                name,
                heading,
                main_title,
                abbrevs,
                chapters,
                verses,
            });
        }
        Ok(out)
    }

    pub fn chapter(&self, book: u32, chapter: u32) -> Result<ChapterOut, QueryError> {
        let osis: String = self
            .conn
            .query_row("SELECT osis FROM book WHERE ord = ?1", [book], |r| r.get(0))
            .optional()
            .map_err(q_err)?
            .ok_or(QueryError::NotFound)?;
        let lo = book as i64 * 1_000_000 + chapter as i64 * 1000;
        let verses = self.verses_between(lo, lo + 999)?;
        if verses.is_empty() {
            return Err(QueryError::NotFound);
        }
        Ok(ChapterOut {
            book,
            osis,
            chapter,
            verses: verses.into_iter().map(|(_, v)| v).collect(),
        })
    }

    /// `(verse, exact stored text)` of every verse of `book` (OSIS ID,
    /// case-sensitive) chapter `chapter`, in order; empty if there is no such
    /// chapter. Pen highlights measure against this (highlights-pen.md 5.2).
    pub fn chapter_texts(
        &self,
        book: &str,
        chapter: u32,
    ) -> Result<Vec<(u32, String)>, QueryError> {
        let mut st = self
            .conn
            .prepare(
                "SELECT v.verse, v.text FROM verse v JOIN book b ON b.ord = v.book
                 WHERE b.osis = ?1 AND v.chapter = ?2 ORDER BY v.id",
            )
            .map_err(q_err)?;
        st.query_map(params![book, chapter], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(q_err)?
            .collect::<Result<_, _>>()
            .map_err(q_err)
    }

    /// The OSIS ID of book `ord`, if this translation has it.
    pub fn book_osis(&self, ord: u32) -> Result<Option<String>, QueryError> {
        self.conn
            .query_row("SELECT osis FROM book WHERE ord = ?1", [ord], |r| r.get(0))
            .optional()
            .map_err(q_err)
    }

    /// One verse by id (`book*1000000 + chapter*1000 + verse`), with its runs,
    /// read from this file (search results: never from the index).
    pub fn verse_by_id(&self, id: i64) -> Result<Option<VerseOut>, QueryError> {
        Ok(self.verses_between(id, id)?.pop().map(|(_, v)| v))
    }

    /// The verses with ids in `lo..=hi`, in id order, each with its runs.
    fn verses_between(&self, lo: i64, hi: i64) -> Result<Vec<(i64, VerseOut)>, QueryError> {
        let mut vstmt = self
            .conn
            .prepare("SELECT id, verse, text FROM verse WHERE id BETWEEN ?1 AND ?2 ORDER BY id")
            .map_err(q_err)?;
        let verses: Vec<(i64, u32, String)> = vstmt
            .query_map(params![lo, hi], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(q_err)?
            .collect::<Result<_, _>>()
            .map_err(q_err)?;
        if verses.is_empty() {
            return Ok(Vec::new());
        }
        let mut red_stmt = self
            .conn
            .prepare("SELECT verse_id, n, start_cp, end_cp FROM red_piece WHERE verse_id BETWEEN ?1 AND ?2 ORDER BY verse_id, n")
            .map_err(q_err)?;
        let mut mark_stmt = self
            .conn
            .prepare("SELECT verse_id, kind, start_cp, end_cp FROM mark WHERE verse_id BETWEEN ?1 AND ?2 ORDER BY verse_id, n")
            .map_err(q_err)?;
        let mut word_stmt = self
            .conn
            .prepare("SELECT verse_id, n, start_cp, end_cp FROM word WHERE verse_id BETWEEN ?1 AND ?2 ORDER BY verse_id, n")
            .map_err(q_err)?;
        let mut key_stmt = self
            .conn
            .prepare("SELECT verse_id, n, key FROM word_key WHERE verse_id BETWEEN ?1 AND ?2 ORDER BY verse_id, n, k")
            .map_err(q_err)?;

        let red: Vec<(i64, RedPiece)> = red_stmt
            .query_map(params![lo, hi], |r| {
                Ok((
                    r.get(0)?,
                    RedPiece {
                        n: r.get(1)?,
                        start: r.get(2)?,
                        end: r.get(3)?,
                    },
                ))
            })
            .map_err(q_err)?
            .collect::<Result<_, _>>()
            .map_err(q_err)?;
        if !red.is_empty() && !self.has_red_letters {
            return Err(QueryError::Content(
                "red pieces in a translation without words-of-Jesus markup".into(),
            ));
        }
        let mut marks: Vec<(i64, Mark)> = Vec::new();
        for row in mark_stmt
            .query_map(params![lo, hi], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u32>(2)?,
                    r.get::<_, u32>(3)?,
                ))
            })
            .map_err(q_err)?
        {
            let (vid, kind, start, end) = row.map_err(q_err)?;
            let kind = match kind.as_str() {
                "added" => MarkKind::Added,
                "divine" => MarkKind::Divine,
                _ => return Err(QueryError::Content("unknown mark kind".into())),
            };
            marks.push((vid, Mark { kind, start, end }));
        }
        let mut words: Vec<(i64, Word)> = word_stmt
            .query_map(params![lo, hi], |r| {
                Ok((
                    r.get(0)?,
                    Word {
                        n: r.get(1)?,
                        start: r.get(2)?,
                        end: r.get(3)?,
                        keys: Vec::new(),
                    },
                ))
            })
            .map_err(q_err)?
            .collect::<Result<_, _>>()
            .map_err(q_err)?;
        for row in key_stmt
            .query_map(params![lo, hi], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, u32>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(q_err)?
        {
            let (vid, n, key) = row.map_err(q_err)?;
            if !is_canonical_key(&key) {
                return Err(QueryError::Content("malformed Strong's key".into()));
            }
            match words.iter_mut().find(|(v, w)| *v == vid && w.n == n) {
                Some((_, w)) => w.keys.push(key),
                None => return Err(QueryError::Content("word_key without word".into())),
            }
        }

        let mut out = Vec::with_capacity(verses.len());
        for (vid, v, text) in verses {
            let r: Vec<RedPiece> = red
                .iter()
                .filter(|(id, _)| *id == vid)
                .map(|(_, p)| p.clone())
                .collect();
            let m: Vec<Mark> = marks
                .iter()
                .filter(|(id, _)| *id == vid)
                .map(|(_, x)| x.clone())
                .collect();
            let w: Vec<Word> = words
                .iter()
                .filter(|(id, _)| *id == vid)
                .map(|(_, x)| x.clone())
                .collect();
            let runs = runs::build_runs(&text, &r, &m, &w)
                .map_err(|e| QueryError::Content(format!("verse {vid}: {e}")))?;
            out.push((vid, VerseOut { v, text, runs }));
        }
        Ok(out)
    }
}

/// Canonical Strong's key (DESIGN.md 6.2): H or G, then 1 to 5 digits, no
/// leading zero.
pub fn is_canonical_key(key: &str) -> bool {
    let b = key.as_bytes();
    b.len() >= 2
        && b.len() <= 6
        && (b[0] == b'H' || b[0] == b'G')
        && b[1] != b'0'
        && b[1..].iter().all(|c| c.is_ascii_digit())
}

// ---------------------------------------------------------------------------
// Lexicon (content.sqlite schema, DESIGN.md 7 and 8)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LexAttr {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_key: Option<String>,
}

/// The tree the UI walks: nodes numbered 0..k-1 in document order, each
/// parent an earlier element (the pipeline's check stage requires the same;
/// checked again here so a malformed entry is refused, not drawn). `n == i` is
/// checked first, so `p < n` keeps `nodes[p]` in bounds whatever the rows hold
/// (D5-F8: no reliance on the file hash).
fn tree_ok(nodes: &[LexNode]) -> bool {
    nodes.iter().enumerate().all(|(i, node)| {
        node.n as usize == i
            && match node.parent {
                None => true,
                Some(p) => p < node.n && nodes[p as usize].name.is_some(),
            }
    })
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LexNode {
    pub n: u32,
    pub parent: Option<u32>,
    /// Element name, or None for a text node.
    pub name: Option<String>,
    /// Text bytes exactly as stored, or None for an element.
    pub text: Option<String>,
    pub attrs: Vec<LexAttr>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LexEntry {
    pub key: String,
    pub lexicon: String,
    pub nodes: Vec<LexNode>,
}

pub struct Lexicon {
    conn: Connection,
}

impl Lexicon {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Lexicon, OpenError> {
        Ok(Lexicon {
            conn: open_content(path, expected_sha256, MAX_CONTENT_BYTES)?,
        })
    }

    pub fn entry(&self, key: &str) -> Result<Option<LexEntry>, QueryError> {
        if !is_canonical_key(key) {
            return Ok(None);
        }
        let lexicon: Option<String> = self
            .conn
            .query_row("SELECT lexicon FROM lex_entry WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(q_err)?;
        let Some(lexicon) = lexicon else {
            return Ok(None);
        };
        let mut stmt = self
            .conn
            .prepare("SELECT n, parent, name, text FROM lex_node WHERE key = ?1 ORDER BY n")
            .map_err(q_err)?;
        let mut nodes: Vec<LexNode> = stmt
            .query_map([key], |r| {
                Ok(LexNode {
                    n: r.get(0)?,
                    parent: r.get(1)?,
                    name: r.get(2)?,
                    text: r.get(3)?,
                    attrs: Vec::new(),
                })
            })
            .map_err(q_err)?
            .collect::<Result<_, _>>()
            .map_err(q_err)?;
        // The tree the UI walks: nodes numbered 0..k-1 in document order, each
        // parent an earlier element (the pipeline's check stage requires the
        // same; checked again here so a malformed entry is refused, not drawn).
        if !tree_ok(&nodes) {
            return Err(QueryError::Content("malformed lexicon entry tree".into()));
        }
        if nodes.is_empty() {
            return Err(QueryError::Content("lexicon entry without nodes".into()));
        }
        let mut astmt = self
            .conn
            .prepare("SELECT n, name, value, ref_key FROM lex_attr WHERE key = ?1 ORDER BY n, i")
            .map_err(q_err)?;
        for row in astmt
            .query_map([key], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    LexAttr {
                        name: r.get(1)?,
                        value: r.get(2)?,
                        ref_key: r.get(3)?,
                    },
                ))
            })
            .map_err(q_err)?
        {
            let (n, attr) = row.map_err(q_err)?;
            if attr
                .ref_key
                .as_deref()
                .is_some_and(|k| !is_canonical_key(k))
            {
                return Err(QueryError::Content("malformed cross-reference key".into()));
            }
            match nodes.get_mut(n as usize) {
                Some(node) if node.name.is_some() => node.attrs.push(attr),
                _ => return Err(QueryError::Content("lex_attr without element".into())),
            }
        }
        Ok(Some(LexEntry {
            key: key.to_string(),
            lexicon,
            nodes,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::runs::tests::red_from_runs;

    #[test]
    fn malformed_lexicon_trees_are_refused_without_panicking() {
        let node = |n: u32, parent: Option<u32>, el: bool| LexNode {
            n,
            parent,
            name: el.then(|| "e".to_string()),
            text: (!el).then(|| "t".to_string()),
            attrs: Vec::new(),
        };
        assert!(tree_ok(&[node(0, None, true), node(1, Some(0), false)]));
        // D5-F8: a large `n` with a parent past the end used to index out of bounds
        assert!(!tree_ok(&[
            node(0, None, true),
            node(1_000_000, Some(999_999), false)
        ]));
        assert!(!tree_ok(&[node(0, None, true), node(1, Some(1), true)])); // own parent
        assert!(!tree_ok(&[node(0, None, false), node(1, Some(0), false)])); // text parent
        assert!(!tree_ok(&[node(1, None, true)]));
    }

    /// W2-B: the KJV is labelled with the owner-approved name; a DB
    /// `translation_name` wins; an unknown abbreviation falls back to itself.
    #[test]
    fn translation_name_comes_from_db_then_label_table_then_abbrev() {
        assert_eq!(translation_label("KJV"), Some("King James Version (1769)"));
        assert_eq!(translation_label("kjv"), None);
        let bible = |rows: &[(&str, &str)]| {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT)", [])
                .unwrap();
            for (k, v) in rows {
                conn.execute("INSERT INTO meta VALUES (?1, ?2)", [k, v])
                    .unwrap();
            }
            let b = Bible::from_conn(conn).unwrap();
            (b.abbrev, b.name)
        };
        assert_eq!(
            bible(&[("translation_abbrev", "KJV")]),
            ("KJV".into(), "King James Version (1769)".into())
        );
        assert_eq!(
            bible(&[
                ("translation_abbrev", "KJV"),
                ("translation_name", "From DB")
            ]),
            ("KJV".into(), "From DB".into())
        );
        assert_eq!(
            bible(&[("translation_abbrev", "XYZ")]),
            ("XYZ".into(), "XYZ".into())
        );
    }

    #[test]
    fn canonical_keys() {
        for ok in ["H1", "G1", "H7225", "G99999"] {
            assert!(is_canonical_key(ok), "{ok}");
        }
        for bad in ["", "H", "H0", "H01", "h1", "X1", "H1a", "H123456", "G 1"] {
            assert!(!is_canonical_key(bad), "{bad}");
        }
    }

    #[test]
    fn committed_fixtures_open_verified() {
        for f in fixture::FILES {
            let path = fixture::dir().join(f.file);
            let conn = open_content(&path, f.sha256, MAX_CONTENT_BYTES).expect(f.file);
            // query_only: any write is refused even on the in-memory copy
            assert!(conn.execute("DELETE FROM meta", []).is_err());
        }
    }

    #[test]
    fn open_verified_rejects_wrong_hash_missing_and_oversized() {
        let f = &fixture::FILES[0];
        let path = fixture::dir().join(f.file);
        let wrong = "0".repeat(64);
        assert!(matches!(
            open_content(&path, &wrong, MAX_CONTENT_BYTES),
            Err(OpenError::HashMismatch { .. })
        ));
        let mut flipped = f.sha256.to_string();
        let last = if flipped.ends_with('0') { "1" } else { "0" };
        flipped.replace_range(63..64, last);
        assert!(matches!(
            open_content(&path, &flipped, MAX_CONTENT_BYTES),
            Err(OpenError::HashMismatch { .. })
        ));
        assert!(matches!(
            open_content(&path, f.sha256, 1024),
            Err(OpenError::TooLarge { .. })
        ));
        assert!(matches!(
            open_content(
                &fixture::dir().join("no-such-file.sqlite"),
                f.sha256,
                MAX_CONTENT_BYTES
            ),
            Err(OpenError::Missing { .. })
        ));
        assert!(matches!(
            open_content(&fixture::dir(), f.sha256, MAX_CONTENT_BYTES),
            Err(OpenError::NotPlain { .. }) | Err(OpenError::Missing { .. })
        ));
    }

    #[test]
    fn open_verified_rejects_a_tampered_copy() {
        let f = &fixture::FILES[0];
        let mut bytes = std::fs::read(fixture::dir().join(f.file)).unwrap();
        let tmp = fixture::temp_dir("tamper");
        let path = tmp.join("copy.sqlite");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            open_content(&path, f.sha256, MAX_CONTENT_BYTES),
            Err(OpenError::HashMismatch { .. })
        ));
    }

    /// Review M1-F1: a `-wal` planted beside a DELETE-mode database changes
    /// what a plain read-only open sees, while the file's hash still matches.
    /// open_verified must ignore it.
    #[test]
    fn open_verified_ignores_a_planted_wal() {
        let f = &fixture::FILES[0];
        let original = std::fs::read(fixture::dir().join(f.file)).unwrap();
        let tmp = fixture::temp_dir("wal");
        let victim = tmp.join("victim.sqlite");
        let attacker = tmp.join("attacker.sqlite");
        std::fs::write(&victim, &original).unwrap();
        std::fs::write(&attacker, &original).unwrap();

        let vid = fixture::FIRST_VERSE_ID;
        // The attacker writes a changed verse into a WAL and keeps it un-checkpointed.
        let a = Connection::open(&attacker).unwrap();
        a.execute_batch("PRAGMA journal_mode = WAL; PRAGMA wal_autocheckpoint = 0;")
            .unwrap();
        a.execute("UPDATE verse SET text = 'TAMPERED' WHERE id = ?1", [vid])
            .unwrap();
        let wal = tmp.join("attacker.sqlite-wal");
        assert!(wal.exists());
        std::fs::copy(&wal, tmp.join("victim.sqlite-wal")).unwrap();
        // The victim's main file is unchanged, so its hash still matches.
        assert_eq!(sha256_hex(&std::fs::read(&victim).unwrap()), f.sha256);

        // Control: a plain read-only open of the path. If SQLite applies the
        // planted WAL here, the attack is real and the test below is meaningful.
        let plain_saw_tamper = {
            use rusqlite::OpenFlags;
            match Connection::open_with_flags(&victim, OpenFlags::SQLITE_OPEN_READ_ONLY) {
                Ok(c) => c
                    .query_row("SELECT text FROM verse WHERE id = ?1", [vid], |r| {
                        r.get::<_, String>(0)
                    })
                    .map(|t| t == "TAMPERED")
                    .unwrap_or(false),
                Err(_) => false,
            }
        };
        // Control: with this SQLite, a plain read-only open DOES apply the planted
        // WAL (reproduces M1-F1), so the assertion below is meaningful.
        assert!(
            plain_saw_tamper,
            "control: the planted WAL had no effect on a plain open"
        );

        let conn = open_content(&victim, f.sha256, MAX_CONTENT_BYTES).unwrap();
        let text: String = conn
            .query_row("SELECT text FROM verse WHERE id = ?1", [vid], |r| r.get(0))
            .unwrap();
        assert_ne!(text, "TAMPERED");
        assert_eq!(text, fixture::first_verse_text());
        drop(a);
    }

    #[test]
    fn every_fixture_verse_runs_concatenate_and_red_equals_stored_ranges() {
        let mut checked_verses = 0;
        let mut checked_pieces = 0;
        for f in fixture::FILES
            .iter()
            .filter(|f| f.kind == fixture::Kind::Bible)
        {
            let bible = Bible::open(&fixture::dir().join(f.file), f.sha256).unwrap();
            let raw = Connection::open_with_flags(
                fixture::dir().join(f.file),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            for b in bible.books().unwrap() {
                for c in 1..=b.chapters {
                    let ch = bible.chapter(b.ord, c).unwrap();
                    assert_eq!(ch.verses.len() as u32, b.verses[(c - 1) as usize]);
                    for v in &ch.verses {
                        let vid = b.ord as i64 * 1_000_000 + c as i64 * 1000 + v.v as i64;
                        let stored: String = raw
                            .query_row("SELECT text FROM verse WHERE id = ?1", [vid], |r| r.get(0))
                            .unwrap();
                        let joined: String = v.runs.iter().map(|r| r.text.as_str()).collect();
                        assert_eq!(joined, stored, "verse {vid}");
                        assert_eq!(v.text, stored);
                        assert!(v.runs.iter().all(|r| !r.text.is_empty()));
                        // red: (n, start, end) from runs == rows in red_piece
                        let mut st = raw
                            .prepare("SELECT n, start_cp, end_cp FROM red_piece WHERE verse_id = ?1 ORDER BY n")
                            .unwrap();
                        let rows: Vec<(u32, u32, u32)> = st
                            .query_map([vid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                            .unwrap()
                            .map(|x| x.unwrap())
                            .collect();
                        assert_eq!(red_from_runs(&v.runs), rows, "verse {vid}");
                        // and the red characters are exactly SQLite's substr() of the stored text
                        for (n, s, e) in &rows {
                            let sql_red: String = raw
                                .query_row(
                                    "SELECT substr(text, ?2 + 1, ?3 - ?2) FROM verse WHERE id = ?1",
                                    params![vid, s, e],
                                    |r| r.get(0),
                                )
                                .unwrap();
                            let ui_red: String = v
                                .runs
                                .iter()
                                .filter(|r| r.rp == Some(*n))
                                .map(|r| r.text.as_str())
                                .collect();
                            assert_eq!(ui_red, sql_red);
                            checked_pieces += 1;
                        }
                        checked_verses += 1;
                    }
                }
            }
        }
        assert!(checked_verses >= 40, "{checked_verses}");
        assert!(checked_pieces >= 8, "{checked_pieces}");
    }

    #[test]
    fn fixture_edge_cases_are_present() {
        let f = fixture::FILES
            .iter()
            .find(|f| f.file == fixture::BIBLE_RED)
            .unwrap();
        let bible = Bible::open(&fixture::dir().join(f.file), f.sha256).unwrap();
        assert!(bible.has_red_letters);
        let ch = bible.chapter(1, 1).unwrap();
        let v = |n: u32| ch.verses.iter().find(|x| x.v == n).unwrap();
        // cross-verse span: v2 ends red at its end, v3 starts red at 0
        let v2 = v(2);
        assert!(v2.runs.last().unwrap().red && !v2.runs.first().unwrap().red);
        let v3 = v(3);
        assert!(v3.runs.first().unwrap().red && !v3.runs.last().unwrap().red);
        // divine name and supplied words inside red
        assert!(v(4).runs.iter().any(|r| r.red && r.nd));
        assert!(v(4).runs.iter().any(|r| r.red && r.add));
        // astral + combining mark inside a red piece and a word
        let v5 = v(5);
        assert!(v5.text.contains('\u{1D400}') && v5.text.contains("e\u{301}"));
        assert!(
            v5.runs
                .iter()
                .any(|r| r.red && r.text.contains('\u{1D400}'))
        );
        // whole-verse red, two pieces in one verse, trailing space kept
        assert!(v(7).runs.iter().all(|r| r.red));
        assert_eq!(v(8).runs.iter().filter_map(|r| r.rp).max(), Some(1));
        assert!(v(6).text.ends_with(' '));
        // multi-key word
        assert!(
            ch.verses
                .iter()
                .flat_map(|x| &x.runs)
                .any(|r| r.w.as_ref().is_some_and(|k| k.len() == 2))
        );

        let f2 = fixture::FILES
            .iter()
            .find(|f| f.file == fixture::BIBLE_PLAIN)
            .unwrap();
        let plain = Bible::open(&fixture::dir().join(f2.file), f2.sha256).unwrap();
        assert!(!plain.has_red_letters);
        assert!(
            plain
                .chapter(1, 1)
                .unwrap()
                .verses
                .iter()
                .flat_map(|v| &v.runs)
                .all(|r| !r.red)
        );
    }

    #[test]
    fn missing_chapter_and_book_are_not_found() {
        let f = &fixture::FILES[0];
        let bible = Bible::open(&fixture::dir().join(f.file), f.sha256).unwrap();
        assert_eq!(bible.chapter(1, 99).unwrap_err(), QueryError::NotFound);
        assert_eq!(bible.chapter(66, 1).unwrap_err(), QueryError::NotFound);
    }
}
