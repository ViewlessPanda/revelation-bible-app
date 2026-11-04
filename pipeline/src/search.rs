//! `kjv-search.sqlite` (DESIGN.md 18): the search-only normalised copy of
//! every KJV verse in an FTS5 table, built from the **published, verified**
//! `kjv.sqlite` (its verse rows, read through `open_verified` with the hash
//! that build just produced), never from the parse model, so the index is a
//! function of exactly the text the app displays.
//!
//! The normalisation is `revelation_content::search::normalise_indexed`,
//! the same function the app runs on a query. The check stage
//! (`check::check_search`) recomputes it independently.

use std::path::Path;

use rusqlite::{Connection, params};

use crate::db::{self, Meta, SCHEMA_VERSION, SourceRow};
use crate::error::{Error, Result};
use crate::open::open_verified;

pub use revelation_content::search::NORMALISATION;

/// Output file name.
pub const SEARCH_DEST: &str = "kjv-search.sqlite";

/// The kjv-search.sqlite schema (DESIGN.md 18.3). `meta` and `source` as in
/// kjv.sqlite; `verse_fts` has `rowid` = `verse.id` and one column, `norm`.
/// The `ascii` tokenizer only splits the already-normalised `[a-z0-9 ]` text
/// at its spaces; no stemming (no `porter`), no prefix indexes, no
/// diacritic folding of its own.
pub const SEARCH_SCHEMA: &str = "\
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE source (
  id     TEXT PRIMARY KEY,                                  -- SOURCES.md ID: 'BIB-KJV-CW31'
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;

CREATE VIRTUAL TABLE verse_fts USING fts5(norm, tokenize = 'ascii', detail = full);
";

/// The verse rows of a verified kjv.sqlite, and where they came from.
#[derive(Debug, Clone)]
pub struct SearchInput {
    /// SHA-256 of the kjv.sqlite read.
    pub kjv_sha256: String,
    /// Its `source` row for BIB-KJV-CW31.
    pub source: SourceRow,
    /// `(verse.id, verse.text)` in id order.
    pub verses: Vec<(i64, String)>,
}

impl SearchInput {
    pub fn meta(&self) -> Meta {
        let mut e: Vec<(String, String)> = vec![
            ("schema_version".into(), SCHEMA_VERSION.to_string()),
            ("translation_abbrev".into(), "KJV".into()),
            ("search_built_from".into(), crate::run::KJV_DEST.into()),
            ("search_built_from_sha256".into(), self.kjv_sha256.clone()),
            ("search_normalisation".into(), NORMALISATION.into()),
        ];
        e.sort();
        Meta {
            entries: e,
            sources: vec![self.source.clone()],
        }
    }
}

fn sql(check: &'static str) -> impl Fn(rusqlite::Error) -> Error {
    move |e| Error::new(check, format!("SQLite: {e}"))
}

/// Reads the verse rows of `kjv_path` through `open_verified`, requiring
/// SHA-256 `kjv_sha256` (the value its build or check just produced) and
/// the content identity.
pub fn load(kjv_path: &Path, kjv_sha256: &str) -> Result<SearchInput> {
    let (conn, hash) = open_verified(kjv_path, Some(kjv_sha256))?;
    revelation_content::require_content_identity(&conn).map_err(Error::from)?;
    let s = sql("search-input");
    let source = conn
        .query_row(
            "SELECT id, file, size, sha256 FROM source WHERE id = 'BIB-KJV-CW31'",
            [],
            |r| {
                Ok(SourceRow {
                    id: r.get(0)?,
                    file: r.get(1)?,
                    size: r.get(2)?,
                    sha256: r.get(3)?,
                })
            },
        )
        .map_err(&s)?;
    let mut st = conn
        .prepare("SELECT id, text FROM verse ORDER BY id")
        .map_err(&s)?;
    let verses = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(&s)?
        .collect::<rusqlite::Result<Vec<(i64, String)>>>()
        .map_err(&s)?;
    Ok(SearchInput {
        kjv_sha256: hash,
        source,
        verses,
    })
}

/// Stage 4: conventions, schema, meta, one row per verse in id order (one
/// transaction), FTS5 `optimize` (one b-tree segment), then the usual
/// integrity check, foreign-key check and `VACUUM`. Fails, naming the verse,
/// if a verse has a character the normalisation does not cover.
pub fn write(conn: &mut Connection, input: &SearchInput, meta: &Meta) -> Result<()> {
    let w = sql("write");
    db::setup(conn)?;
    let tx = conn.transaction().map_err(&w)?;
    tx.execute_batch(SEARCH_SCHEMA).map_err(&w)?;
    db::insert_meta(&tx, meta)?;
    {
        let mut st = tx
            .prepare("INSERT INTO verse_fts (rowid, norm) VALUES (?1, ?2)")
            .map_err(&w)?;
        for (id, text) in &input.verses {
            let norm = revelation_content::search::normalise_indexed(text).map_err(|c| {
                Error::new(
                    "search-norm",
                    format!(
                        "verse {id}: character U+{:04X} is not covered by the search normalisation",
                        c as u32
                    ),
                )
            })?;
            if norm.is_empty() {
                return Err(
                    Error::new("search-norm", format!("verse {id}: no words to index")).into(),
                );
            }
            st.execute(params![id, norm]).map_err(&w)?;
        }
    }
    tx.commit().map_err(&w)?;
    conn.execute_batch("INSERT INTO verse_fts (verse_fts) VALUES ('optimize');")
        .map_err(&w)?;
    db::seal(conn)
}
