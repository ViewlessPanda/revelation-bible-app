//! `user.sqlite`: the only database the app writes (PLAN.md 2.1, 7).
//!
//! One small module, every write in a transaction, a rollback journal
//! (`journal_mode = DELETE`) with `synchronous = FULL`, `PRAGMA
//! integrity_check` on open, versioned migrations (v1 to v4 here). It holds
//! highlights (keyed by verse reference, never by translation), the last
//! reading position, the reader's settings, bookmarks, pen marks and daily
//! notes (v4). It never touches content text.
//!
//! Why not WAL (D5-F1, a deliberate deviation from PLAN.md's "SQLite WAL"): in
//! WAL mode every committed write stays in `user.sqlite-wal` until a
//! checkpoint, and the app's writes are far too small to trigger one, so the
//! main file was an empty shell and copying only `user.sqlite` (or losing the
//! `-wal`) silently lost every highlight. With a rollback journal each commit
//! lands in `user.sqlite` itself before the call returns; the writes are tiny
//! and rare, so WAL's concurrency buys nothing here. A file left in WAL mode by
//! an earlier build is checkpointed and switched on open, and a WAL-mode file
//! whose data can't be found (no schema in the main file and a missing or
//! damaged `-wal`) is refused, never migrated over.

use crate::marks::{self, ChapterText, Mark, MarkIn, MarkOut, Piece, PieceOut};
use crate::notes;
use rusqlite::config::DbConfig;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// "RVLU": marks the file as a Revelation user database.
pub const USER_APPLICATION_ID: i64 = 0x5256_4C55;
/// The newest schema this build knows: the last entry of [`MIGRATIONS`].
pub const USER_SCHEMA_VERSION: i64 = 4;

pub use crate::marks::COLORS;

const MIGRATION_V1: &str = r#"
CREATE TABLE highlight (
  reference  TEXT PRIMARY KEY,                     -- 'John.3.16': OSIS book ID, chapter, verse
  color      TEXT NOT NULL CHECK (color IN ('gold', 'green', 'blue', 'rose', 'violet')),
  created_at INTEGER NOT NULL                      -- Unix seconds
) STRICT, WITHOUT ROWID;
CREATE TABLE position (
  id            INTEGER PRIMARY KEY CHECK (id = 1),
  translation   TEXT NOT NULL,
  book          TEXT NOT NULL,                     -- OSIS book ID
  chapter       INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  anchor_verse  INTEGER CHECK (anchor_verse BETWEEN 1 AND 999),
  anchor_frac   REAL CHECK (anchor_frac BETWEEN 0 AND 1),
  current_verse INTEGER CHECK (current_verse BETWEEN 1 AND 999),
  updated_at    INTEGER NOT NULL
) STRICT;
CREATE TABLE setting (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
"#;

/// v2 (W2-BM, bookmarks.md 8): one automatic bookmark per book, the last
/// chapter of it actually read. Translation-independent (OSIS book ID).
const MIGRATION_V2: &str = r#"
CREATE TABLE IF NOT EXISTS bookmark (
  book     TEXT PRIMARY KEY,                          -- OSIS book ID ('Luke'): one per book, any translation
  chapter  INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  read_at  INTEGER NOT NULL                           -- Unix seconds: when this chapter last counted as read
) STRICT, WITHOUT ROWID;
"#;

/// v3 (W2-5h, highlights-pen.md 5.2, additive): pen-mode range highlights.
/// One `mark` per stroke (one translation, one chapter), one `mark_piece` per
/// verse in code points of that translation's stored verse text, with the
/// verse's length and fingerprint at write time. The v1 `highlight` table is
/// untouched and stays the whole-verse table; nothing is converted.
const MIGRATION_V3: &str = r#"
CREATE TABLE mark (
  id          INTEGER PRIMARY KEY,
  translation TEXT    NOT NULL CHECK (length(translation) BETWEEN 1 AND 40),
  book        TEXT    NOT NULL,                      -- OSIS book ID
  chapter     INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  color       TEXT    NOT NULL CHECK (color IN ('gold', 'green', 'blue', 'rose', 'violet')),
  created_at  INTEGER NOT NULL,                      -- Unix seconds, first stroke
  updated_at  INTEGER NOT NULL
) STRICT;
CREATE INDEX mark_by_chapter ON mark (translation, book, chapter);
CREATE TABLE mark_piece (
  mark_id   INTEGER NOT NULL REFERENCES mark (id) ON DELETE CASCADE,
  verse     INTEGER NOT NULL CHECK (verse BETWEEN 1 AND 999),
  start_cp  INTEGER NOT NULL CHECK (start_cp >= 0),
  end_cp    INTEGER NOT NULL CHECK (end_cp > start_cp),
  verse_len INTEGER NOT NULL CHECK (end_cp <= verse_len),   -- code points at write time
  verse_fp  TEXT    NOT NULL CHECK (length(verse_fp) = 16), -- first 16 hex of SHA-256(UTF-8 verse text)
  PRIMARY KEY (mark_id, verse, start_cp)
) STRICT, WITHOUT ROWID;
"#;

/// v4 (W2-5, wave-2-layout 8, additive): daily notes. One row per local day
/// that holds text (an empty day has no row): the ProseMirror document as
/// JSON, checked against the wave-2 schema by `notes::check_doc` on every
/// save and every read. Passage tags are block attributes inside it, never
/// text. One transaction per save.
const MIGRATION_V4: &str = r#"
CREATE TABLE note_day (
  day        TEXT    PRIMARY KEY CHECK (length(day) = 10 AND day GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
  doc        TEXT    NOT NULL CHECK (length(doc) BETWEEN 2 AND 524288),  -- JSON, at most notes::MAX_NOTE_BYTES
  created_at INTEGER NOT NULL,                     -- Unix seconds, first save of the day
  updated_at INTEGER NOT NULL
) STRICT;
"#;

/// Every schema step, in order: `(user_version after it, SQL)`. A file at
/// version `n` runs each step above `n`, each in its own transaction with its
/// `user_version`, so an interrupted upgrade resumes where it stopped. New
/// steps are appended here; no earlier step ever changes.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, MIGRATION_V1),
    (2, MIGRATION_V2),
    (3, MIGRATION_V3),
    (4, MIGRATION_V4),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    Corrupt(String),
    /// The main file is in WAL mode with no schema: the data was in a
    /// `user.sqlite-wal` that is missing or damaged (D5-F1).
    MissingWal,
    NotUserFile,
    TooNew(i64),
    Invalid(String),
    Sqlite(String),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::Corrupt(e) => write!(
                f,
                "your highlights and settings file is damaged ({e}); it was left untouched"
            ),
            UserError::MissingWal => write!(
                f,
                "your highlights and settings file is incomplete (its companion file user.sqlite-wal is missing or damaged); it was left untouched"
            ),
            UserError::NotUserFile => write!(
                f,
                "the highlights and settings file is not a Revelation file; it was left untouched"
            ),
            UserError::TooNew(v) => write!(
                f,
                "the highlights and settings file was made by a newer version of Revelation (schema {v}); open it with that version. It was left untouched"
            ),
            UserError::Invalid(e) => write!(f, "invalid value: {e}"),
            UserError::Sqlite(e) => write!(f, "user database error: {e}"),
        }
    }
}

fn sq(e: rusqlite::Error) -> UserError {
    UserError::Sqlite(e.to_string())
}

/// rusqlite's open error can carry the full path (with the Windows user name):
/// say only the file name and SQLite's own reason (D5-F5).
fn open_err(e: rusqlite::Error) -> UserError {
    match e {
        rusqlite::Error::SqliteFailure(f, _) => {
            UserError::Sqlite(format!("user.sqlite could not be opened ({f})"))
        }
        _ => UserError::Sqlite("user.sqlite could not be opened".into()),
    }
}

/// Whether the file on disk says WAL mode (header bytes 18 and 19 are 2),
/// read before SQLite opens it.
fn wal_mode_on_disk(path: &Path) -> bool {
    use std::io::Read;
    let mut head = [0u8; 20];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut head))
        .is_ok()
        && (head[18] == 2 || head[19] == 2)
}

/// `user.sqlite` plus a suffix, e.g. `-wal`, next to it.
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// SQLite's companion files of a database.
const SIDECARS: [&str; 3] = ["-wal", "-shm", "-journal"];

/// W2A-U1: the schema version of a Revelation user file made by a newer
/// build, read from the SQLite header only (application_id at byte 68,
/// user_version at byte 60, both big-endian). Opens nothing and writes nothing.
fn too_new_on_disk(path: &Path) -> Option<i64> {
    use std::io::Read;
    let mut h = [0u8; 100];
    std::fs::File::open(path).ok()?.read_exact(&mut h).ok()?;
    if &h[..16] != b"SQLite format 3\0" {
        return None;
    }
    let be = |i: usize| i64::from(i32::from_be_bytes([h[i], h[i + 1], h[i + 2], h[i + 3]]));
    let (version, app_id) = (be(60), be(68));
    (app_id == USER_APPLICATION_ID && version > USER_SCHEMA_VERSION).then_some(version)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub translation: String,
    pub book: String,
    pub chapter: u32,
    pub anchor_verse: Option<u32>,
    pub anchor_frac: Option<f64>,
    pub current_verse: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub verse_numbers: bool,
    pub red_letters: bool,
    pub supplied_italics: bool,
    /// 1..=10, index into the reading-size steps (5 = 20px).
    pub text_step: u32,
    pub panel_open: bool,
    /// Commentary panel open (wave-2-layout 4; first run closed).
    #[serde(default)]
    pub commentary_open: bool,
    /// The right column's width in rem (wave-2-layout 17.4; the UI clamps it
    /// to 18..=36 on restore). On the first start with this version it is the
    /// larger of the two old panel widths (see `load_state`).
    #[serde(default = "default_right_width")]
    pub right_width_rem: f64,
    /// Commentary's share of the right column's height, 0..=1 (17.4).
    #[serde(default = "default_right_split")]
    pub right_split: f64,
    /// The highlighter pen's last colour (highlights-pen.md 3.1, key
    /// `pen.color`). Pen mode itself is a tool, never saved.
    #[serde(default = "default_pen_color")]
    pub pen_color: String,
    /// Notes panel open (wave-2-layout 4; first run closed).
    #[serde(default)]
    pub notes_open: bool,
    /// The notes panel's width in rem (the UI clamps it to 16..=32).
    #[serde(default = "default_notes_width")]
    pub notes_width_rem: f64,
}

fn default_pen_color() -> String {
    "gold".into()
}

fn default_right_width() -> f64 {
    22.0
}

fn default_right_split() -> f64 {
    0.5
}

fn default_notes_width() -> f64 {
    20.0
}

/// The right column's width limits in rem (wave-2-layout 17.1).
const RIGHT_WIDTH_REM: std::ops::RangeInclusive<f64> = 18.0..=36.0;

impl Default for Settings {
    fn default() -> Self {
        Settings {
            verse_numbers: true,
            red_letters: true,
            supplied_italics: true,
            text_step: 5,
            panel_open: false,
            commentary_open: false,
            right_width_rem: default_right_width(),
            right_split: default_right_split(),
            pen_color: default_pen_color(),
            notes_open: false,
            notes_width_rem: default_notes_width(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedState {
    pub position: Option<Position>,
    pub settings: Settings,
}

/// One automatic bookmark (bookmarks.md 8): a book's last chapter read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bookmark {
    pub book: String,
    pub chapter: u32,
    /// Unix seconds.
    pub read_at: i64,
}

/// A restored `read_at` is at most this (the year 2100): a bound, not a clock check.
pub const MAX_READ_AT: i64 = 4_102_444_800;

/// The chapter a pen-highlight call is about: translation id (as loaded),
/// OSIS book ID, chapter number.
#[derive(Debug, Clone, Copy)]
pub struct ChapterKey<'a> {
    pub translation: &'a str,
    pub book: &'a str,
    pub chapter: u32,
}

impl ChapterKey<'_> {
    fn check(&self) -> Result<(), UserError> {
        let t = self.translation;
        if t.is_empty() || t.len() > 16 || !t.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err(UserError::Invalid("translation".into()));
        }
        if !valid_book_id(self.book) || !(1..=999).contains(&self.chapter) {
            return Err(UserError::Invalid("book or chapter".into()));
        }
        Ok(())
    }
}

/// A piece whose verse text changed since it was written (5.4): kept exactly.
#[derive(Debug, Clone)]
struct StalePiece {
    verse: u32,
    start: u32,
    end: u32,
    len: u32,
    fp: String,
}

#[derive(Debug, Clone)]
struct StoredMark {
    id: i64,
    color: String,
    created_at: i64,
    fresh: Vec<Piece>,
    stale: Vec<StalePiece>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Highlight {
    pub verse: u32,
    pub color: String,
}

/// OSIS-style book ID as used in references: 1–16 ASCII letters or digits.
pub fn valid_book_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 16 && s.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn parse_num(s: &str) -> Option<u32> {
    if s.is_empty() || s.len() > 3 || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// `Book.chapter.verse`, e.g. `John.3.16`.
pub fn parse_reference(r: &str) -> Option<(&str, u32, u32)> {
    let mut it = r.split('.');
    let (b, c, v) = (it.next()?, it.next()?, it.next()?);
    if it.next().is_some() || !valid_book_id(b) {
        return None;
    }
    Some((b, parse_num(c)?, parse_num(v)?))
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct UserDb {
    conn: Connection,
}

impl UserDb {
    /// Open (or create) the user database at `path`, check it, migrate it.
    ///
    /// Nothing is written until every check has passed: a refused file (and
    /// its `-wal`) is left exactly as it was.
    pub fn open(path: &Path) -> Result<UserDb, UserError> {
        let wal_on_disk = wal_mode_on_disk(path);
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(open_err)?;
        // While checking, closing must not checkpoint (or delete) a `-wal` an
        // earlier build left: if the file is refused, both stay untouched.
        conn.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)
            .map_err(sq)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(sq)?;

        // Integrity first, before anything writes to the file.
        let integrity: Vec<String> = {
            let mut st = conn
                .prepare("PRAGMA integrity_check")
                .map_err(|e| UserError::Corrupt(e.to_string()))?;
            st.query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| UserError::Corrupt(e.to_string()))?
                .collect::<Result<_, _>>()
                .map_err(|e| UserError::Corrupt(e.to_string()))?
        };
        if integrity != ["ok"] {
            return Err(UserError::Corrupt(
                integrity.into_iter().take(3).collect::<Vec<_>>().join("; "),
            ));
        }
        let app_id: i64 = conn
            .query_row("PRAGMA application_id", [], |r| r.get(0))
            .map_err(sq)?;
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sq)?;
        let tables: i64 = conn
            .query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))
            .map_err(sq)?;
        let empty = app_id == 0 && version == 0 && tables == 0;
        // An empty file in WAL mode was made by an earlier build, which kept
        // everything in `user.sqlite-wal`: that file is missing or damaged.
        // Migrating would make the loss permanent (D5-F1), so refuse.
        if empty && wal_on_disk {
            return Err(UserError::MissingWal);
        }
        if app_id != USER_APPLICATION_ID && !empty {
            return Err(UserError::NotUserFile);
        }
        if version > USER_SCHEMA_VERSION {
            return Err(UserError::TooNew(version));
        }

        // Checks passed. A file from an earlier build is still in WAL mode:
        // copy every committed frame into the main file, then switch to a
        // rollback journal (which deletes the emptied `-wal`).
        conn.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, false)
            .map_err(sq)?;
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .map_err(sq)?;
        if mode.eq_ignore_ascii_case("wal") {
            let (busy, log, done): (i64, i64, i64) = conn
                .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .map_err(sq)?;
            if busy != 0 || log != done {
                return Err(UserError::Sqlite(
                    "the highlights file is in use by another program; close it and restart".into(),
                ));
            }
        }
        let mode: String = conn
            .query_row("PRAGMA journal_mode = DELETE", [], |r| r.get(0))
            .map_err(sq)?;
        if !mode.eq_ignore_ascii_case("delete") {
            return Err(UserError::Sqlite(format!(
                "could not leave journal mode {mode}; close other programs using the highlights file and restart"
            )));
        }
        conn.execute_batch("PRAGMA synchronous = FULL; PRAGMA foreign_keys = ON;")
            .map_err(sq)?;
        let mut db = UserDb { conn };
        db.migrate(version)?;
        Ok(db)
    }

    /// D5-F5: the reader asked to start again after the file was refused.
    /// The refused file and its companions are renamed to
    /// `user-kept-<unix seconds>.sqlite[-wal|-shm|-journal]` beside it (never
    /// deleted), then a new file is created. Returns the new database and the
    /// kept file's name (empty if there was no file to keep).
    pub fn set_aside_and_start_new(path: &Path) -> Result<(UserDb, String), UserError> {
        // W2A-U1: a newer build's file is never set aside (that would hide
        // its notes and highlights from that build); it opens there.
        if let Some(v) = too_new_on_disk(path) {
            return Err(UserError::TooNew(v));
        }
        let dir = path
            .parent()
            .ok_or_else(|| UserError::Sqlite("no folder for user.sqlite".into()))?;
        let mut kept = String::new();
        if path.exists() {
            let stamp = now();
            let name = (0..100)
                .map(|i| {
                    if i == 0 {
                        format!("user-kept-{stamp}.sqlite")
                    } else {
                        format!("user-kept-{stamp}-{i}.sqlite")
                    }
                })
                .find(|n| {
                    let t = dir.join(n);
                    !t.exists() && SIDECARS.iter().all(|s| !sidecar(&t, s).exists())
                })
                .ok_or_else(|| UserError::Sqlite("no free name for the kept file".into()))?;
            let target = dir.join(&name);
            std::fs::rename(path, &target).map_err(|e| {
                UserError::Sqlite(format!("could not rename user.sqlite: {}", e.kind()))
            })?;
            for s in SIDECARS {
                let from = sidecar(path, s);
                if from.exists() {
                    std::fs::rename(&from, sidecar(&target, s)).map_err(|e| {
                        UserError::Sqlite(format!("could not rename user.sqlite{s}: {}", e.kind()))
                    })?;
                }
            }
            kept = name;
        }
        let db = UserDb::open(path)?;
        Ok((db, kept))
    }

    fn migrate(&mut self, from: i64) -> Result<(), UserError> {
        for &(version, sql) in MIGRATIONS.iter().filter(|(v, _)| *v > from) {
            let tx = self.conn.transaction().map_err(sq)?;
            tx.execute_batch(sql).map_err(sq)?;
            tx.execute_batch(&format!(
                "PRAGMA application_id = {USER_APPLICATION_ID}; PRAGMA user_version = {version};"
            ))
            .map_err(sq)?;
            tx.commit().map_err(sq)?;
        }
        Ok(())
    }

    /// Every bookmark (at most one per book), by book ID.
    pub fn bookmarks(&self) -> Result<Vec<Bookmark>, UserError> {
        let mut st = self
            .conn
            .prepare("SELECT book, chapter, read_at FROM bookmark ORDER BY book")
            .map_err(sq)?;
        let rows = st
            .query_map([], |r| {
                Ok(Bookmark {
                    book: r.get(0)?,
                    chapter: r.get(1)?,
                    read_at: r.get(2)?,
                })
            })
            .map_err(sq)?;
        let mut out = Vec::new();
        for b in rows {
            let b = b.map_err(sq)?;
            // a row edited outside the app that breaks the rules is skipped, never shown
            if valid_book_id(&b.book) {
                out.push(b);
            }
        }
        Ok(out)
    }

    /// Upsert `book`'s bookmark at `chapter`, read now (bookmarks.md 2.5).
    pub fn set_bookmark(&mut self, book: &str, chapter: u32) -> Result<(), UserError> {
        self.put_bookmark(&Bookmark {
            book: book.to_string(),
            chapter,
            read_at: now(),
        })
    }

    /// Delete `book`'s bookmark; returns the old row, for Undo.
    pub fn clear_bookmark(&mut self, book: &str) -> Result<Option<Bookmark>, UserError> {
        if !valid_book_id(book) {
            return Err(UserError::Invalid("book".into()));
        }
        let tx = self.conn.transaction().map_err(sq)?;
        let old = tx
            .query_row(
                "SELECT book, chapter, read_at FROM bookmark WHERE book = ?1",
                [book],
                |r| {
                    Ok(Bookmark {
                        book: r.get(0)?,
                        chapter: r.get(1)?,
                        read_at: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(sq)?;
        tx.execute("DELETE FROM bookmark WHERE book = ?1", [book])
            .map_err(sq)?;
        tx.commit().map_err(sq)?;
        Ok(old)
    }

    /// Undo of a clear: that exact row back (same chapter and `read_at`).
    pub fn restore_bookmark(&mut self, b: &Bookmark) -> Result<(), UserError> {
        if !(0..=MAX_READ_AT).contains(&b.read_at) {
            return Err(UserError::Invalid("read_at".into()));
        }
        self.put_bookmark(b)
    }

    fn put_bookmark(&mut self, b: &Bookmark) -> Result<(), UserError> {
        if !valid_book_id(&b.book) || !(1..=999).contains(&b.chapter) {
            return Err(UserError::Invalid("book or chapter".into()));
        }
        let tx = self.conn.transaction().map_err(sq)?;
        tx.execute(
            "INSERT INTO bookmark (book, chapter, read_at) VALUES (?1, ?2, ?3)
             ON CONFLICT (book) DO UPDATE SET chapter = excluded.chapter, read_at = excluded.read_at",
            params![b.book, b.chapter, b.read_at],
        )
        .map_err(sq)?;
        tx.commit().map_err(sq)
    }

    pub fn schema_version(&self) -> Result<i64, UserError> {
        self.conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sq)
    }

    /// Highlights of one chapter, by verse number.
    pub fn highlights(&self, book: &str, chapter: u32) -> Result<Vec<Highlight>, UserError> {
        if !valid_book_id(book) || !(1..=999).contains(&chapter) {
            return Err(UserError::Invalid("book or chapter".into()));
        }
        // book IDs are alphanumeric, so the pattern has no LIKE wildcards in it
        let pattern = format!("{book}.{chapter}.%");
        let mut st = self
            .conn
            .prepare("SELECT reference, color FROM highlight WHERE reference LIKE ?1 ESCAPE '\\' ORDER BY reference")
            .map_err(sq)?;
        let rows = st
            .query_map([pattern], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(sq)?;
        let mut out = Vec::new();
        for row in rows {
            let (reference, color) = row.map_err(sq)?;
            // LIKE is case-insensitive for ASCII: compare exactly
            if let Some((b, c, v)) = parse_reference(&reference)
                && b == book
                && c == chapter
            {
                out.push(Highlight { verse: v, color });
            }
        }
        out.sort_by_key(|h| h.verse);
        Ok(out)
    }

    /// Set (Some) or remove (None) the highlight of one verse, in a transaction.
    pub fn set_highlight(&mut self, reference: &str, color: Option<&str>) -> Result<(), UserError> {
        if parse_reference(reference).is_none() {
            return Err(UserError::Invalid("reference".into()));
        }
        if let Some(c) = color
            && !COLORS.contains(&c)
        {
            return Err(UserError::Invalid("color".into()));
        }
        let tx = self.conn.transaction().map_err(sq)?;
        match color {
            Some(c) => {
                tx.execute(
                    "INSERT INTO highlight (reference, color, created_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT (reference) DO UPDATE SET color = excluded.color, created_at = excluded.created_at",
                    params![reference, c, now()],
                )
                .map_err(sq)?;
            }
            None => {
                tx.execute("DELETE FROM highlight WHERE reference = ?1", [reference])
                    .map_err(sq)?;
            }
        }
        tx.commit().map_err(sq)
    }

    // ---- pen-mode range highlights (highlights-pen.md 5, 6) ----
    // Every write: validate against the current verse texts (passed in by the
    // caller from the read-only content), apply the rules of `marks.rs` to the
    // chapter's fresh pieces, check the invariant, then rewrite the chapter's
    // pieces in one transaction. Stale pieces (text changed) are carried over
    // byte for byte: never cut, moved, merged or deleted by a stroke.

    /// The chapter's marks; each piece flagged `stale` if the verse text no
    /// longer matches its fingerprint or length (5.4).
    pub fn marks(
        &self,
        key: &ChapterKey,
        texts: &[(u32, String)],
    ) -> Result<Vec<MarkOut>, UserError> {
        key.check()?;
        let text = ChapterText::new(texts);
        Ok(self
            .load_chapter(key, &text)?
            .into_iter()
            .filter(|m| !(m.fresh.is_empty() && m.stale.is_empty()))
            .map(|m| {
                let mut pieces: Vec<PieceOut> = m
                    .fresh
                    .iter()
                    .map(|p| PieceOut {
                        verse: p.verse,
                        start: p.start,
                        end: p.end,
                        stale: false,
                    })
                    .chain(m.stale.iter().map(|s| PieceOut {
                        verse: s.verse,
                        start: s.start,
                        end: s.end,
                        stale: true,
                    }))
                    .collect();
                pieces.sort_by_key(|p| (p.verse, p.start));
                MarkOut {
                    id: m.id,
                    color: m.color,
                    created_at: m.created_at,
                    pieces,
                }
            })
            .collect())
    }

    /// One pen stroke in `color` (rules 1 and 2): the stroke's characters
    /// leave every other mark, then same-colour neighbours join it.
    pub fn pen_apply(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        color: &str,
        pieces: &[Piece],
    ) -> Result<Vec<MarkOut>, UserError> {
        if !marks::valid_color(color) {
            return Err(UserError::Invalid("color".into()));
        }
        let text = ChapterText::new(texts);
        marks::check_stroke(&text, pieces).map_err(UserError::Invalid)?;
        self.rewrite(key, texts, |model| {
            for p in pieces {
                marks::cut(model, &text, p.verse, p.start, p.end);
            }
            model.push(Mark {
                id: None,
                color: color.to_string(),
                created_at: now(),
                pieces: pieces.to_vec(),
            });
            Ok(Some(model.len() - 1))
        })
    }

    /// One eraser stroke: removes every range-highlighted character under it,
    /// any colour, this translation (4.2).
    pub fn pen_erase(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        pieces: &[Piece],
    ) -> Result<Vec<MarkOut>, UserError> {
        let text = ChapterText::new(texts);
        marks::check_stroke(&text, pieces).map_err(UserError::Invalid)?;
        self.rewrite(key, texts, |model| {
            for p in pieces {
                marks::cut(model, &text, p.verse, p.start, p.end);
            }
            Ok(None)
        })
    }

    /// Remove a whole mark (its drawn pieces; stale ones stay listed).
    pub fn mark_remove(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        id: i64,
    ) -> Result<Vec<MarkOut>, UserError> {
        self.rewrite(key, texts, |model| {
            let i = model
                .iter()
                .position(|m| m.id == Some(id))
                .ok_or_else(|| UserError::Invalid("no such highlight".into()))?;
            model.remove(i);
            Ok(None)
        })
    }

    /// Recolour a whole mark, then join same-colour neighbours (rule 5).
    pub fn mark_recolor(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        id: i64,
        color: &str,
    ) -> Result<Vec<MarkOut>, UserError> {
        if !marks::valid_color(color) {
            return Err(UserError::Invalid("color".into()));
        }
        self.rewrite(key, texts, |model| {
            let i = model
                .iter()
                .position(|m| m.id == Some(id))
                .ok_or_else(|| UserError::Invalid("no such highlight".into()))?;
            model[i].color = color.to_string();
            Ok(Some(i))
        })
    }

    /// Undo / redo: the chapter's drawn marks become exactly `snapshot`
    /// (every piece validated as in `pen_apply`; no two pieces may overlap).
    pub fn marks_replace_chapter(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        snapshot: &[MarkIn],
    ) -> Result<Vec<MarkOut>, UserError> {
        let text = ChapterText::new(texts);
        let mut all: Vec<(u32, u32, u32)> = Vec::new();
        for m in snapshot {
            if !marks::valid_color(&m.color)
                || !(0..=MAX_READ_AT).contains(&m.created_at)
                || m.pieces.is_empty()
                || m.pieces.len() > marks::MAX_STROKE_PIECES
            {
                return Err(UserError::Invalid("highlight".into()));
            }
            for p in &m.pieces {
                marks::check_piece(&text, p).map_err(UserError::Invalid)?;
                all.push((p.verse, p.start, p.end));
            }
        }
        all.sort();
        if all.windows(2).any(|w| w[0].0 == w[1].0 && w[1].1 < w[0].2) {
            return Err(UserError::Invalid("overlapping highlights".into()));
        }
        self.rewrite(key, texts, |model| {
            model.clear();
            model.extend(snapshot.iter().map(|m| Mark {
                id: None,
                color: m.color.clone(),
                created_at: m.created_at,
                pieces: m.pieces.clone(),
            }));
            Ok(None)
        })
    }

    /// Load the chapter, let `edit` change the model (returning the index of
    /// the mark that keeps its identity when marks join), apply the merge
    /// rules, check the invariant and the cap, and write it all in one
    /// transaction. Returns the chapter's marks as stored.
    fn rewrite(
        &mut self,
        key: &ChapterKey,
        texts: &[(u32, String)],
        edit: impl FnOnce(&mut Vec<Mark>) -> Result<Option<usize>, UserError>,
    ) -> Result<Vec<MarkOut>, UserError> {
        key.check()?;
        let text = ChapterText::new(texts);
        if text.is_empty() {
            return Err(UserError::Invalid("no such chapter".into()));
        }
        let before = self.load_chapter(key, &text)?;
        let mut model: Vec<Mark> = before
            .iter()
            .filter(|m| !m.fresh.is_empty())
            .map(|m| Mark {
                id: Some(m.id),
                color: m.color.clone(),
                created_at: m.created_at,
                pieces: m.fresh.clone(),
            })
            .collect();
        let prefer = edit(&mut model)?;
        let after = marks::normalize(model, &text, prefer);
        marks::check_invariant(&after, &text).map_err(UserError::Invalid)?;
        let stale: usize = before.iter().map(|m| m.stale.len()).sum();
        let total_before: usize = stale + before.iter().map(|m| m.fresh.len()).sum::<usize>();
        let total_after: usize = stale + after.iter().map(|m| m.pieces.len()).sum::<usize>();
        if total_after > marks::MAX_CHAPTER_PIECES && total_after > total_before {
            return Err(UserError::Invalid(
                "this chapter has too many highlights (2,000 pieces at most)".into(),
            ));
        }
        let t = now();
        let tx = self.conn.transaction().map_err(sq)?;
        tx.execute(
            "DELETE FROM mark_piece WHERE mark_id IN
               (SELECT id FROM mark WHERE translation = ?1 AND book = ?2 AND chapter = ?3)",
            params![key.translation, key.book, key.chapter],
        )
        .map_err(sq)?;
        {
            let mut ins_piece = tx
                .prepare(
                    "INSERT INTO mark_piece (mark_id, verse, start_cp, end_cp, verse_len, verse_fp)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .map_err(sq)?;
            for m in &after {
                let id = match m.id {
                    Some(id) => {
                        let old = before.iter().find(|b| b.id == id);
                        let changed = old.is_none_or(|o| {
                            o.color != m.color
                                || o.created_at != m.created_at
                                || o.fresh != m.pieces
                        });
                        if changed {
                            tx.execute(
                                "UPDATE mark SET color = ?2, created_at = ?3, updated_at = ?4 WHERE id = ?1",
                                params![id, m.color, m.created_at, t],
                            )
                            .map_err(sq)?;
                        }
                        id
                    }
                    None => {
                        tx.execute(
                            "INSERT INTO mark (translation, book, chapter, color, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            params![key.translation, key.book, key.chapter, m.color, m.created_at, t],
                        )
                        .map_err(sq)?;
                        tx.last_insert_rowid()
                    }
                };
                for p in &m.pieces {
                    let v = text.get(p.verse).expect("checked by the invariant");
                    ins_piece
                        .execute(params![
                            id,
                            p.verse,
                            p.start,
                            p.end,
                            v.chars.len() as u32,
                            v.fp
                        ])
                        .map_err(sq)?;
                }
            }
            for b in &before {
                for s in &b.stale {
                    ins_piece
                        .execute(params![b.id, s.verse, s.start, s.end, s.len, s.fp])
                        .map_err(sq)?;
                }
            }
        }
        // a mark left with no pieces (emptied, joined into another, or a
        // tampered row without pieces, E13) is deleted in the same transaction
        tx.execute(
            "DELETE FROM mark WHERE translation = ?1 AND book = ?2 AND chapter = ?3
               AND NOT EXISTS (SELECT 1 FROM mark_piece WHERE mark_id = mark.id)",
            params![key.translation, key.book, key.chapter],
        )
        .map_err(sq)?;
        tx.commit().map_err(sq)?;
        self.marks(key, texts)
    }

    /// Every mark of the chapter as stored, its pieces split into fresh
    /// (valid against the current text) and stale. Rows a tampered file could
    /// hold that break the rules are stale (pieces) or skipped (unknown
    /// colour), never drawn and never clamped.
    fn load_chapter(
        &self,
        key: &ChapterKey,
        text: &ChapterText,
    ) -> Result<Vec<StoredMark>, UserError> {
        let mut st = self
            .conn
            .prepare(
                "SELECT m.id, m.color, m.created_at, p.verse, p.start_cp, p.end_cp, p.verse_len, p.verse_fp
                 FROM mark m LEFT JOIN mark_piece p ON p.mark_id = m.id
                 WHERE m.translation = ?1 AND m.book = ?2 AND m.chapter = ?3
                 ORDER BY m.id, p.verse, p.start_cp",
            )
            .map_err(sq)?;
        type Row = (
            i64,
            String,
            i64,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
        );
        let rows = st
            .query_map(params![key.translation, key.book, key.chapter], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                ))
            })
            .map_err(sq)?
            .collect::<Result<Vec<Row>, _>>()
            .map_err(sq)?;
        let mut out: Vec<StoredMark> = Vec::new();
        let mut taken: std::collections::BTreeMap<u32, Vec<(u32, u32)>> = Default::default();
        for (id, color, created_at, verse, start, end, len, fp) in rows {
            if !marks::valid_color(&color) {
                continue;
            }
            if out.last().is_none_or(|m| m.id != id) {
                out.push(StoredMark {
                    id,
                    color,
                    created_at,
                    fresh: Vec::new(),
                    stale: Vec::new(),
                });
            }
            let m = out.last_mut().unwrap();
            let (Some(verse), Some(start), Some(end), Some(len), Some(fp)) =
                (verse, start, end, len, fp)
            else {
                continue; // a mark without pieces (E13)
            };
            let u = |x: i64| u32::try_from(x).ok();
            let (Some(verse), Some(start), Some(end), Some(len)) =
                (u(verse), u(start), u(end), u(len))
            else {
                continue; // cannot be stored under the CHECKs; never drawn
            };
            let p = Piece { verse, start, end };
            // a piece overlapping one already loaded (only a tampered file
            // can hold that) is set aside like a stale one, so the drawn
            // pieces always keep the invariant
            let overlaps = taken
                .get(&verse)
                .is_some_and(|l: &Vec<(u32, u32)>| l.iter().any(|&(a, b)| start < b && a < end));
            let fresh = !overlaps
                && text
                    .get(verse)
                    .is_some_and(|v| v.chars.len() as u32 == len && v.fp == fp)
                && marks::check_piece(text, &p).is_ok();
            if fresh {
                taken.entry(verse).or_default().push((start, end));
            }
            if fresh {
                m.fresh.push(p);
            } else {
                m.stale.push(StalePiece {
                    verse,
                    start,
                    end,
                    len,
                    fp,
                });
            }
        }
        Ok(out)
    }

    pub fn load_state(&self) -> Result<SavedState, UserError> {
        let position = self
            .conn
            .query_row(
                "SELECT translation, book, chapter, anchor_verse, anchor_frac, current_verse FROM position WHERE id = 1",
                [],
                |r| {
                    Ok(Position {
                        translation: r.get(0)?,
                        book: r.get(1)?,
                        chapter: r.get(2)?,
                        anchor_verse: r.get(3)?,
                        anchor_frac: r.get(4)?,
                        current_verse: r.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(sq)?;
        let mut s = Settings::default();
        let mut st = self
            .conn
            .prepare("SELECT key, value FROM setting")
            .map_err(sq)?;
        let rows = st
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(sq)?;
        // The two old per-panel widths, read only to migrate (wave-2-layout 17.4).
        let (mut saw_right, mut old_study, mut old_comm) = (false, None::<f64>, None::<f64>);
        let width_ok = |v: &str| {
            v.parse::<f64>()
                .ok()
                .filter(|w| w.is_finite() && (10.0..=80.0).contains(w))
        };
        for row in rows {
            let (k, v) = row.map_err(sq)?;
            let flag = match v.as_str() {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            };
            // unknown keys and out-of-range values fall back to the defaults
            match k.as_str() {
                "verse_numbers" => s.verse_numbers = flag.unwrap_or(s.verse_numbers),
                "red_letters" => s.red_letters = flag.unwrap_or(s.red_letters),
                "supplied_italics" => s.supplied_italics = flag.unwrap_or(s.supplied_italics),
                "panel_open" => s.panel_open = flag.unwrap_or(s.panel_open),
                "commentary_open" => s.commentary_open = flag.unwrap_or(s.commentary_open),
                "notes_open" => s.notes_open = flag.unwrap_or(s.notes_open),
                "notes_width_rem" => {
                    if let Some(w) = width_ok(&v) {
                        s.notes_width_rem = w;
                    }
                }
                "text_step" => {
                    if let Ok(n) = v.parse::<u32>()
                        && (1..=10).contains(&n)
                    {
                        s.text_step = n;
                    }
                }
                "panel_width_rem" => old_study = width_ok(&v),
                "commentary_width_rem" => old_comm = width_ok(&v),
                "right_width_rem" => {
                    if let Some(w) = width_ok(&v) {
                        s.right_width_rem = w;
                        saw_right = true;
                    }
                }
                "right_split" => {
                    if let Ok(f) = v.parse::<f64>()
                        && f.is_finite()
                        && (0.0..=1.0).contains(&f)
                    {
                        s.right_split = f;
                    }
                }
                "pen.color" if marks::valid_color(&v) => s.pen_color = v,
                _ => {}
            }
        }
        // First start with the one right column: its width is the larger of
        // the two widths the reader had saved, clamped (17.4). The split starts
        // at its default. Nothing saved before: the default width.
        if !saw_right && let Some(w) = old_study.into_iter().chain(old_comm).reduce(f64::max) {
            s.right_width_rem = w.clamp(*RIGHT_WIDTH_REM.start(), *RIGHT_WIDTH_REM.end());
        }
        Ok(SavedState {
            position,
            settings: s,
        })
    }

    pub fn save_state(&mut self, state: &SavedState) -> Result<(), UserError> {
        let s = &state.settings;
        if !(1..=10).contains(&s.text_step)
            || !s.right_width_rem.is_finite()
            || !(10.0..=80.0).contains(&s.right_width_rem)
            || !s.right_split.is_finite()
            || !(0.0..=1.0).contains(&s.right_split)
            || !marks::valid_color(&s.pen_color)
            || !s.notes_width_rem.is_finite()
            || !(10.0..=80.0).contains(&s.notes_width_rem)
        {
            return Err(UserError::Invalid("settings".into()));
        }
        if let Some(p) = &state.position {
            let verse_ok = |v: Option<u32>| v.is_none_or(|v| (1..=999).contains(&v));
            let frac_ok = p
                .anchor_frac
                .is_none_or(|f| f.is_finite() && (0.0..=1.0).contains(&f));
            if p.translation.is_empty()
                || p.translation.len() > 16
                || !p.translation.bytes().all(|b| b.is_ascii_alphanumeric())
                || !valid_book_id(&p.book)
                || !(1..=999).contains(&p.chapter)
                || !verse_ok(p.anchor_verse)
                || !verse_ok(p.current_verse)
                || !frac_ok
            {
                return Err(UserError::Invalid("position".into()));
            }
        }
        let b = |x: bool| if x { "1" } else { "0" };
        let tx = self.conn.transaction().map_err(sq)?;
        if let Some(p) = &state.position {
            tx.execute(
                "INSERT INTO position (id, translation, book, chapter, anchor_verse, anchor_frac, current_verse, updated_at)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT (id) DO UPDATE SET translation = excluded.translation, book = excluded.book,
                   chapter = excluded.chapter, anchor_verse = excluded.anchor_verse, anchor_frac = excluded.anchor_frac,
                   current_verse = excluded.current_verse, updated_at = excluded.updated_at",
                params![p.translation, p.book, p.chapter, p.anchor_verse, p.anchor_frac, p.current_verse, now()],
            )
            .map_err(sq)?;
        }
        let text_step = s.text_step.to_string();
        let width = format!("{}", s.right_width_rem);
        // 3 decimals, as the UI rounds it (17.4)
        let split = format!("{}", (s.right_split * 1000.0).round() / 1000.0);
        let notes_width = format!("{}", s.notes_width_rem);
        for (k, v) in [
            ("verse_numbers", b(s.verse_numbers)),
            ("red_letters", b(s.red_letters)),
            ("supplied_italics", b(s.supplied_italics)),
            ("panel_open", b(s.panel_open)),
            ("text_step", text_step.as_str()),
            ("commentary_open", b(s.commentary_open)),
            ("right_width_rem", width.as_str()),
            ("right_split", split.as_str()),
            ("pen.color", s.pen_color.as_str()),
            ("notes_open", b(s.notes_open)),
            ("notes_width_rem", notes_width.as_str()),
        ] {
            tx.execute(
                "INSERT INTO setting (key, value) VALUES (?1, ?2) ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![k, v],
            )
            .map_err(sq)?;
        }
        tx.commit().map_err(sq)
    }

    // ---- daily notes (W2-5, wave-2-layout 8) ----

    /// Save one day's note: the whole document, in one transaction. A
    /// document with no text deletes the day (empty days are not stored).
    /// The day is the UI's local date; nothing is written on a bad call.
    pub fn note_save(&mut self, day: &str, doc: &str) -> Result<(), UserError> {
        if !notes::valid_day(day) {
            return Err(UserError::Invalid("note day".into()));
        }
        let info = notes::check_doc(doc).map_err(UserError::Invalid)?;
        let tx = self.conn.transaction().map_err(sq)?;
        if info.chars == 0 {
            tx.execute("DELETE FROM note_day WHERE day = ?1", params![day])
                .map_err(sq)?;
        } else {
            let t = now();
            tx.execute(
                "INSERT INTO note_day (day, doc, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)
                 ON CONFLICT (day) DO UPDATE SET doc = excluded.doc, updated_at = excluded.updated_at",
                params![day, doc, t],
            )
            .map_err(sq)?;
        }
        tx.commit().map_err(sq)
    }

    /// Days with notes, newest first: those before `before` (all when None),
    /// at most `limit` (capped at `notes::MAX_PAGE`), and whether more exist.
    /// A row that fails the schema check is returned with `doc: None`.
    pub fn notes_list(&self, before: Option<&str>, limit: u32) -> Result<NotesPage, UserError> {
        if before.is_some_and(|b| !notes::valid_day(b)) {
            return Err(UserError::Invalid("note day".into()));
        }
        let limit = limit.clamp(1, notes::MAX_PAGE);
        let mut st = self
            .conn
            .prepare(
                "SELECT day, doc, updated_at FROM note_day WHERE ?1 IS NULL OR day < ?1
                 ORDER BY day DESC LIMIT ?2",
            )
            .map_err(sq)?;
        let mut days: Vec<NoteDay> = st
            .query_map(params![before, limit + 1], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(sq)?
            .map(|row| {
                let (day, doc, updated_at) = row.map_err(sq)?;
                let ok = notes::valid_day(&day) && notes::check_doc(&doc).is_ok();
                Ok(NoteDay {
                    day,
                    doc: ok.then_some(doc),
                    updated_at,
                })
            })
            .collect::<Result<_, UserError>>()?;
        let more = days.len() > limit as usize;
        days.truncate(limit as usize);
        Ok(NotesPage { days, more })
    }
}

/// One day's note as stored (wave-2-layout 8.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoteDay {
    /// The local date, `YYYY-MM-DD`.
    pub day: String,
    /// The ProseMirror document as JSON; None when the stored row fails the
    /// schema check (it is left untouched and shown as unreadable).
    pub doc: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotesPage {
    pub days: Vec<NoteDay>,
    /// More (earlier) days exist ("Show earlier days").
    pub more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::temp_dir;

    #[test]
    fn references() {
        assert_eq!(parse_reference("John.3.16"), Some(("John", 3, 16)));
        assert_eq!(parse_reference("Testbook.1.1"), Some(("Testbook", 1, 1)));
        for bad in [
            "",
            "John.3",
            "John.3.16.1",
            "John.03.16",
            "John.3.0",
            "Jo hn.3.16",
            "John.3.1000",
            "%.1.1",
            "_.1.1",
        ] {
            assert_eq!(parse_reference(bad), None, "{bad}");
        }
    }

    fn journal_mode(db: &UserDb) -> String {
        db.conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap()
    }

    /// Header bytes 18 and 19 of the file on disk (1 = rollback, 2 = WAL).
    fn header_mode(path: &Path) -> (u8, u8) {
        let b = std::fs::read(path).unwrap();
        (b[18], b[19])
    }

    fn sha(path: &Path) -> String {
        revelation_content::sha256_hex(&std::fs::read(path).unwrap())
    }

    /// A user database exactly as the previous build (WAL, never
    /// checkpointed) left it: `n` highlights, a position and a setting, all in
    /// `user.sqlite-wal`; the main file is the 4,096-byte empty shell.
    fn old_build_db(path: &Path, n: u32) {
        let mut c = Connection::open(path).unwrap();
        c.execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA foreign_keys = ON;",
        )
        .unwrap();
        let tx = c.transaction().unwrap();
        tx.execute_batch(MIGRATION_V1).unwrap();
        tx.execute_batch(&format!(
            "PRAGMA application_id = {USER_APPLICATION_ID}; PRAGMA user_version = 1;"
        ))
        .unwrap();
        tx.commit().unwrap();
        for i in 1..=n {
            let tx = c.transaction().unwrap();
            tx.execute(
                "INSERT INTO highlight VALUES (?1, 'gold', 0)",
                [format!("Testbook.{}.{}", 1 + i / 100, 1 + i % 100)],
            )
            .unwrap();
            tx.commit().unwrap();
        }
        c.execute_batch(
            "INSERT INTO position VALUES (1, 'PH1', 'Testbook', 3, 2, 0.5, 4, 0);
             INSERT INTO setting VALUES ('text_step', '7');",
        )
        .unwrap();
        // the old build never closed the connection: nothing reached the main file
        c.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)
            .unwrap();
        drop(c);
        assert_eq!(header_mode(path), (2, 2));
        assert_eq!(std::fs::metadata(path).unwrap().len(), 4096);
        assert!(sidecar(path, "-wal").metadata().unwrap().len() > 4096);
    }

    fn count_highlights(db: &UserDb) -> i64 {
        db.conn
            .query_row("SELECT count(*) FROM highlight", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn new_file_is_migrated_to_the_newest_version_with_a_rollback_journal() {
        let d = temp_dir("user-new");
        let path = d.join("user.sqlite");
        let db = UserDb::open(&path).unwrap();
        assert_eq!(db.schema_version().unwrap(), USER_SCHEMA_VERSION);
        assert_eq!(MIGRATIONS.last().unwrap().0, USER_SCHEMA_VERSION);
        assert_eq!(journal_mode(&db), "delete");
        let sync: i64 = db
            .conn
            .query_row("PRAGMA synchronous", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sync, 2, "synchronous = FULL");
        assert_eq!(header_mode(&path), (1, 1));
        let st = db.load_state().unwrap();
        assert_eq!(st.position, None);
        assert_eq!(st.settings, Settings::default());
    }

    #[test]
    fn highlight_persists_across_reopen_and_toggles_off() {
        let d = temp_dir("user-hl");
        let path = d.join("user.sqlite");
        {
            let mut db = UserDb::open(&path).unwrap();
            db.set_highlight("Testbook.1.4", Some("rose")).unwrap();
            db.set_highlight("Testbook.1.5", Some("gold")).unwrap();
            db.set_highlight("Testbook.1.5", Some("blue")).unwrap(); // change colour
            db.set_highlight("Testbook.2.1", Some("green")).unwrap();
            db.set_highlight("testbook.1.6", Some("violet")).unwrap(); // different book ID (case)
        } // dropped: connection closed
        {
            let mut db = UserDb::open(&path).unwrap();
            let hl = db.highlights("Testbook", 1).unwrap();
            assert_eq!(
                hl,
                vec![
                    Highlight {
                        verse: 4,
                        color: "rose".into()
                    },
                    Highlight {
                        verse: 5,
                        color: "blue".into()
                    }
                ]
            );
            db.set_highlight("Testbook.1.4", None).unwrap();
        }
        let db = UserDb::open(&path).unwrap();
        assert_eq!(
            db.highlights("Testbook", 1).unwrap(),
            vec![Highlight {
                verse: 5,
                color: "blue".into()
            }]
        );
        assert_eq!(db.highlights("Testbook", 2).unwrap().len(), 1);
    }

    #[test]
    fn highlight_survives_without_a_clean_close() {
        // A committed transaction is durable even if the process never closes
        // the connection (a hard kill): simulate by leaking it.
        let d = temp_dir("user-leak");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.set_highlight("Testbook.1.1", Some("gold")).unwrap();
        std::mem::forget(db);
        let db2 = UserDb::open(&path).unwrap();
        assert_eq!(
            db2.highlights("Testbook", 1).unwrap(),
            vec![Highlight {
                verse: 1,
                color: "gold".into()
            }]
        );
    }

    #[test]
    fn committed_data_is_in_user_sqlite_alone() {
        // D5-F1: write, never close (as on a hard kill or Tauri's exit), copy
        // only user.sqlite elsewhere: everything committed is in it.
        let d = temp_dir("user-alone");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.set_highlight("Testbook.1.2", Some("violet")).unwrap();
        let mut state = SavedState {
            position: None,
            settings: Settings::default(),
        };
        state.settings.text_step = 9;
        db.save_state(&state).unwrap();
        std::mem::forget(db);
        for s in SIDECARS {
            assert!(!sidecar(&path, s).exists(), "no {s} after a commit");
        }
        let copy = temp_dir("user-alone-copy").join("user.sqlite");
        std::fs::copy(&path, &copy).unwrap();
        let db = UserDb::open(&copy).unwrap();
        assert_eq!(
            db.highlights("Testbook", 1).unwrap(),
            vec![Highlight {
                verse: 2,
                color: "violet".into()
            }]
        );
        assert_eq!(db.load_state().unwrap(), state);
    }

    #[test]
    fn wal_file_from_the_previous_build_is_converted_with_every_row() {
        let d = temp_dir("user-oldwal");
        let path = d.join("user.sqlite");
        old_build_db(&path, 250);
        {
            let db = UserDb::open(&path).unwrap();
            assert_eq!(count_highlights(&db), 250);
            assert_eq!(journal_mode(&db), "delete");
            let st = db.load_state().unwrap();
            assert_eq!(st.position.unwrap().chapter, 3);
            assert_eq!(st.settings.text_step, 7);
        }
        assert_eq!(header_mode(&path), (1, 1));
        assert!(!sidecar(&path, "-wal").exists(), "the emptied -wal is gone");
        // and now the main file alone holds everything
        let copy = temp_dir("user-oldwal-copy").join("user.sqlite");
        std::fs::copy(&path, &copy).unwrap();
        let mut db = UserDb::open(&copy).unwrap();
        assert_eq!(count_highlights(&db), 250);
        db.set_highlight("Testbook.9.9", Some("blue")).unwrap();
        assert_eq!(count_highlights(&db), 251);
    }

    #[test]
    fn empty_wal_shell_is_refused_not_migrated() {
        // D5-F1: the main file without its -wal (copied alone, or the -wal
        // lost) must be refused, never migrated into an empty database.
        let d = temp_dir("user-shell");
        let path = d.join("user.sqlite");
        old_build_db(&path, 3);
        let alone = temp_dir("user-shell-alone").join("user.sqlite");
        std::fs::copy(&path, &alone).unwrap();
        let before = sha(&alone);
        assert_eq!(UserDb::open(&alone).err(), Some(UserError::MissingWal));
        assert_eq!(UserDb::open(&alone).err(), Some(UserError::MissingWal));
        assert_eq!(sha(&alone), before, "the shell is left untouched");

        // a damaged -wal beside it: every frame is dropped, so the same refusal,
        // and neither file is changed (the -wal is not checkpointed or deleted)
        let wal = sidecar(&path, "-wal");
        let mut w = std::fs::read(&wal).unwrap();
        w[32 + 24 + 100] ^= 0x55; // inside the first frame's page
        std::fs::write(&wal, &w).unwrap();
        let (main_before, wal_before) = (sha(&path), sha(&wal));
        assert_eq!(UserDb::open(&path).err(), Some(UserError::MissingWal));
        assert_eq!(sha(&path), main_before);
        assert_eq!(sha(&wal), wal_before);
        assert!(
            UserError::MissingWal
                .to_string()
                .contains("user.sqlite-wal")
        );
    }

    #[test]
    fn a_refused_file_is_kept_and_a_new_one_started() {
        let d = temp_dir("user-restart");
        let path = d.join("user.sqlite");
        old_build_db(&path, 2);
        let wal = sidecar(&path, "-wal");
        std::fs::remove_file(&wal).unwrap(); // the -wal is lost
        let shell = std::fs::read(&path).unwrap();
        assert_eq!(UserDb::open(&path).err(), Some(UserError::MissingWal));
        let (mut db, kept) = UserDb::set_aside_and_start_new(&path).unwrap();
        assert!(
            kept.starts_with("user-kept-") && kept.ends_with(".sqlite"),
            "{kept}"
        );
        assert_eq!(
            std::fs::read(d.join(&kept)).unwrap(),
            shell,
            "kept, byte for byte"
        );
        db.set_highlight("Testbook.1.1", Some("gold")).unwrap();
        assert_eq!(count_highlights(&db), 1);
        drop(db);
        // a second restart keeps both earlier files under distinct names
        let (_db, kept2) = UserDb::set_aside_and_start_new(&path).unwrap();
        assert_ne!(kept, kept2);
        assert!(d.join(&kept).exists() && d.join(&kept2).exists());
    }

    /// W2A-U1: a file from a newer build (here schema v5) is refused, and is
    /// never set aside: it stays byte-identical where that build finds it.
    #[test]
    fn a_newer_builds_file_is_refused_and_never_set_aside() {
        let d = temp_dir("user-too-new");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.set_highlight("Testbook.1.1", Some("gold")).unwrap();
        drop(db);
        let v5 = USER_SCHEMA_VERSION + 1;
        Connection::open(&path)
            .unwrap()
            .execute_batch(&format!("PRAGMA user_version = {v5}"))
            .unwrap();
        let files = |d: &Path| {
            let mut v: Vec<String> = std::fs::read_dir(d)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            v.sort();
            v
        };
        let (bytes, before) = (std::fs::read(&path).unwrap(), files(&d));
        let e = UserDb::open(&path).err().unwrap();
        assert_eq!(e, UserError::TooNew(v5));
        assert!(e.to_string().contains("newer version of Revelation"), "{e}");
        assert_eq!(
            UserDb::set_aside_and_start_new(&path).err(),
            Some(UserError::TooNew(v5))
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "left byte for byte");
        assert_eq!(files(&d), before, "nothing renamed or created");
        // a foreign file with a high user_version is still set aside (not ours)
        let foreign = d.join("foreign").join("user.sqlite");
        std::fs::create_dir_all(foreign.parent().unwrap()).unwrap();
        Connection::open(&foreign)
            .unwrap()
            .execute_batch(&format!("CREATE TABLE x (y); PRAGMA user_version = {v5}"))
            .unwrap();
        assert_eq!(UserDb::open(&foreign).err(), Some(UserError::NotUserFile));
        let (_db, kept) = UserDb::set_aside_and_start_new(&foreign).unwrap();
        assert!(kept.starts_with("user-kept-"), "{kept}");
    }

    #[test]
    fn open_errors_do_not_carry_the_folder_path() {
        let d = temp_dir("user-dir-in-place");
        let path = d.join("user.sqlite");
        std::fs::create_dir(&path).unwrap();
        let e = UserDb::open(&path).err().unwrap().to_string();
        let folder = d.file_name().unwrap().to_string_lossy().to_string();
        assert!(!e.contains(&folder), "{e}");
    }

    #[test]
    fn failed_write_rolls_back() {
        let d = temp_dir("user-rollback");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.set_highlight("Testbook.1.1", Some("gold")).unwrap();
        assert!(db.set_highlight("Testbook.1.1", Some("red")).is_err()); // not a highlight colour
        assert!(db.set_highlight("bad ref", Some("gold")).is_err());
        // a transaction that fails part-way leaves nothing behind
        {
            let tx = db.conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO highlight VALUES ('Testbook.1.2', 'gold', 0)",
                [],
            )
            .unwrap();
            assert!(
                tx.execute(
                    "INSERT INTO highlight VALUES ('Testbook.1.3', 'red', 0)",
                    []
                )
                .is_err()
            );
            // dropped without commit -> rolled back
        }
        assert_eq!(
            db.highlights("Testbook", 1).unwrap(),
            vec![Highlight {
                verse: 1,
                color: "gold".into()
            }]
        );
    }

    #[test]
    fn state_round_trips_and_is_validated() {
        let d = temp_dir("user-state");
        let path = d.join("user.sqlite");
        let state = SavedState {
            position: Some(Position {
                translation: "PH1".into(),
                book: "Testbook".into(),
                chapter: 2,
                anchor_verse: Some(7),
                anchor_frac: Some(0.25),
                current_verse: Some(9),
            }),
            settings: Settings {
                verse_numbers: false,
                red_letters: false,
                supplied_italics: true,
                text_step: 8,
                panel_open: true,
                commentary_open: true,
                right_width_rem: 30.5,
                right_split: 0.375,
                pen_color: "violet".into(),
                notes_open: true,
                notes_width_rem: 24.5,
            },
        };
        UserDb::open(&path).unwrap().save_state(&state).unwrap();
        assert_eq!(UserDb::open(&path).unwrap().load_state().unwrap(), state);

        let mut db = UserDb::open(&path).unwrap();
        let mut bad = state.clone();
        bad.settings.text_step = 11;
        assert!(db.save_state(&bad).is_err());
        for w in [f64::NAN, f64::INFINITY, 9.9, 80.1] {
            let mut bad = state.clone();
            bad.settings.right_width_rem = w;
            assert!(db.save_state(&bad).is_err(), "{w}");
        }
        for w in [f64::NAN, f64::NEG_INFINITY, 9.9, 80.1] {
            let mut bad = state.clone();
            bad.settings.notes_width_rem = w;
            assert!(db.save_state(&bad).is_err(), "{w}");
        }
        for f in [f64::NAN, -0.1, 1.1] {
            let mut bad = state.clone();
            bad.settings.right_split = f;
            assert!(db.save_state(&bad).is_err(), "{f}");
        }
        // A UI from before W2-4a / W2-RC sends no commentary or column fields
        // (and may still send the old widths, which are ignored): the defaults.
        let old: Settings = serde_json::from_str(
            r#"{"verse_numbers":true,"red_letters":true,"supplied_italics":true,"text_step":5,"panel_open":false,"panel_width_rem":22}"#,
        )
        .unwrap();
        assert_eq!(old, Settings::default());
        let mut bad = state.clone();
        bad.settings.pen_color = "red".into();
        assert!(db.save_state(&bad).is_err());
        let mut bad = state.clone();
        bad.position.as_mut().unwrap().anchor_frac = Some(f64::NAN);
        assert!(db.save_state(&bad).is_err());
        let mut bad = state.clone();
        bad.position.as_mut().unwrap().book = "../x".into();
        assert!(db.save_state(&bad).is_err());
        assert_eq!(db.load_state().unwrap(), state);
    }

    /// wave-2-layout 17.4: the first start with the one right column takes the
    /// larger of the two saved old widths (clamped to 18..=36 rem), the split
    /// starts at 0.5, and once `right_width_rem` is saved it is never migrated again.
    #[test]
    fn right_column_width_migrates_from_the_two_old_widths_once() {
        let d = temp_dir("user-right-migrate");
        let seed = |name: &str, rows: &[(&str, &str)]| {
            let path = d.join(name);
            let db = UserDb::open(&path).unwrap();
            for (k, v) in rows {
                db.conn
                    .execute(
                        "INSERT INTO setting (key, value) VALUES (?1, ?2)",
                        params![k, v],
                    )
                    .unwrap();
            }
            drop(db);
            path
        };
        let load = |p: &std::path::Path| UserDb::open(p).unwrap().load_state().unwrap().settings;
        // the larger of the two
        let p = seed(
            "a.sqlite",
            &[("panel_width_rem", "24"), ("commentary_width_rem", "30.5")],
        );
        let s = load(&p);
        assert_eq!((s.right_width_rem, s.right_split), (30.5, 0.5));
        let p = seed(
            "b.sqlite",
            &[("panel_width_rem", "26"), ("commentary_width_rem", "19")],
        );
        assert_eq!(load(&p).right_width_rem, 26.0);
        // clamped to the column's limits
        let p = seed(
            "c.sqlite",
            &[("panel_width_rem", "40"), ("commentary_width_rem", "20")],
        );
        assert_eq!(load(&p).right_width_rem, 36.0);
        let p = seed(
            "d.sqlite",
            &[("panel_width_rem", "12"), ("commentary_width_rem", "11")],
        );
        assert_eq!(load(&p).right_width_rem, 18.0);
        // only one old width saved: that one; none saved: the default
        let p = seed("e.sqlite", &[("commentary_width_rem", "20.5")]);
        assert_eq!(load(&p).right_width_rem, 20.5);
        let p = seed("f.sqlite", &[]);
        assert_eq!(load(&p).right_width_rem, 22.0);
        // a saved right width wins over the old ones, and the old ones are no longer written
        let p = seed(
            "g.sqlite",
            &[("panel_width_rem", "30"), ("right_width_rem", "21")],
        );
        assert_eq!(load(&p).right_width_rem, 21.0);
        let mut db = UserDb::open(&p).unwrap();
        let mut st = db.load_state().unwrap();
        st.settings.right_width_rem = 27.5;
        st.settings.right_split = 0.3333;
        db.save_state(&st).unwrap();
        let s = db.load_state().unwrap().settings;
        assert_eq!((s.right_width_rem, s.right_split), (27.5, 0.333));
        let keys: Vec<String> = db
            .conn
            .prepare("SELECT key FROM setting ORDER BY key")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(!keys.contains(&"commentary_width_rem".to_string()));
        assert!(keys.contains(&"right_split".to_string()));
        // out-of-range saved values fall back to the defaults
        let p = seed(
            "h.sqlite",
            &[("right_width_rem", "500"), ("right_split", "2")],
        );
        let s = load(&p);
        assert_eq!((s.right_width_rem, s.right_split), (22.0, 0.5));
    }

    #[test]
    fn refuses_foreign_newer_and_corrupt_files() {
        let d = temp_dir("user-bad");
        // a database that isn't ours
        let foreign = d.join("foreign.sqlite");
        Connection::open(&foreign)
            .unwrap()
            .execute_batch("CREATE TABLE x (y)")
            .unwrap();
        assert_eq!(UserDb::open(&foreign).err(), Some(UserError::NotUserFile));
        // a newer schema
        let newer = d.join("newer.sqlite");
        drop(UserDb::open(&newer).unwrap());
        Connection::open(&newer)
            .unwrap()
            .execute_batch(&format!(
                "PRAGMA user_version = {}",
                USER_SCHEMA_VERSION + 1
            ))
            .unwrap();
        assert_eq!(
            UserDb::open(&newer).err(),
            Some(UserError::TooNew(USER_SCHEMA_VERSION + 1))
        );
        // garbage bytes
        let junk = d.join("junk.sqlite");
        std::fs::write(
            &junk,
            b"this is not a database at all, just placeholder bytes ....",
        )
        .unwrap();
        let before = std::fs::read(&junk).unwrap();
        assert!(UserDb::open(&junk).is_err());
        assert_eq!(
            std::fs::read(&junk).unwrap(),
            before,
            "a bad file is left untouched"
        );
    }

    /// A file as each earlier build left it: the migrations up to `v`, with
    /// a highlight, a position and a setting from v1 on.
    fn db_at_version(path: &Path, v: i64) {
        let c = Connection::open(path).unwrap();
        for &(version, sql) in MIGRATIONS.iter().filter(|(n, _)| *n <= v) {
            c.execute_batch(sql).unwrap();
            c.execute_batch(&format!(
                "PRAGMA application_id = {USER_APPLICATION_ID}; PRAGMA user_version = {version};"
            ))
            .unwrap();
        }
        if v >= 1 {
            c.execute_batch(
                "INSERT INTO highlight VALUES ('Testbook.1.2', 'rose', 7);
                 INSERT INTO position VALUES (1, 'PH1', 'Testbook', 3, 2, 0.5, 4, 0);
                 INSERT INTO setting VALUES ('text_step', '7');",
            )
            .unwrap();
        }
        if v >= 2 {
            c.execute_batch("INSERT INTO bookmark VALUES ('Ps', 23, 1700000000);")
                .unwrap();
        }
        if v >= 3 {
            c.execute_batch(
                "INSERT INTO mark VALUES (1, 'PH9', 'Testbook', 1, 'blue', 5, 5);
                 INSERT INTO mark_piece VALUES (1, 1, 0, 3, 10, '0123456789abcdef');",
            )
            .unwrap();
        }
    }

    #[test]
    fn every_earlier_version_migrates_in_order_and_keeps_its_data() {
        assert_eq!(MIGRATIONS[0].0, 1);
        assert!(MIGRATIONS.windows(2).all(|w| w[1].0 == w[0].0 + 1));
        for v in 0..USER_SCHEMA_VERSION {
            let d = temp_dir(&format!("user-from-v{v}"));
            let path = d.join("user.sqlite");
            if v > 0 {
                db_at_version(&path, v);
            }
            let mut db = UserDb::open(&path).unwrap_or_else(|e| panic!("from v{v}: {e}"));
            assert_eq!(
                db.schema_version().unwrap(),
                USER_SCHEMA_VERSION,
                "from v{v}"
            );
            if v >= 1 {
                assert_eq!(count_highlights(&db), 1, "from v{v}");
                let st = db.load_state().unwrap();
                assert_eq!(st.position.unwrap().chapter, 3, "from v{v}");
                assert_eq!(st.settings.text_step, 7, "from v{v}");
            }
            // a v2 file keeps its bookmarks
            assert_eq!(
                db.bookmarks().unwrap().len(),
                usize::from(v >= 2),
                "from v{v}"
            );
            // a v3 file keeps its pen marks
            let kept: i64 = db
                .conn
                .query_row("SELECT count(*) FROM mark_piece", [], |r| r.get(0))
                .unwrap();
            assert_eq!(kept, i64::from(v >= 3), "from v{v}");
            // the newest tables work after the upgrade, and survive a reopen
            db.set_bookmark("Luke", 12).unwrap();
            assert!(
                db.notes_list(None, 30).unwrap().days.is_empty(),
                "from v{v}"
            );
            db.note_save("2026-10-27", &note_doc("upgraded")).unwrap();
            let t = texts(&["alpha beta gamma"]);
            assert!(
                db.marks(&KEY, &t).unwrap().is_empty(),
                "from v{v}: no marks yet"
            );
            db.pen_apply(&KEY, &t, "gold", &[pc(1, 6, 10)]).unwrap();
            drop(db);
            let db = UserDb::open(&path).unwrap();
            assert!(
                db.bookmarks()
                    .unwrap()
                    .iter()
                    .any(|b| b.book == "Luke" && b.chapter == 12),
                "from v{v}"
            );
            assert_eq!(db.marks(&KEY, &t).unwrap().len(), 1, "from v{v}");
            let page = db.notes_list(None, 30).unwrap();
            assert_eq!(page.days.len(), 1, "from v{v}");
            assert_eq!(
                page.days[0].doc.as_deref(),
                Some(note_doc("upgraded").as_str())
            );
            if v >= 1 {
                // nothing converted: whole-verse highlights stay whole-verse
                assert_eq!(count_highlights(&db), 1, "from v{v}");
            }
        }
    }

    // ---- pen-mode range highlights (W2-5h) ----

    const KEY: ChapterKey<'static> = ChapterKey {
        translation: "PH1",
        book: "Testbook",
        chapter: 1,
    };
    fn texts(vs: &[&str]) -> Vec<(u32, String)> {
        vs.iter()
            .enumerate()
            .map(|(i, s)| (i as u32 + 1, s.to_string()))
            .collect()
    }
    fn pc(verse: u32, start: u32, end: u32) -> Piece {
        Piece { verse, start, end }
    }
    /// (colour, [(verse, start, end)]) of each mark, fresh pieces only.
    type Shape = (String, Vec<(u32, u32, u32)>);
    fn shape(ms: &[MarkOut]) -> Vec<Shape> {
        let mut v: Vec<_> = ms
            .iter()
            .map(|m| {
                (
                    m.color.clone(),
                    m.pieces
                        .iter()
                        .filter(|p| !p.stale)
                        .map(|p| (p.verse, p.start, p.end))
                        .collect(),
                )
            })
            .collect();
        v.sort();
        v
    }
    fn s(c: &str, ps: &[(u32, u32, u32)]) -> Shape {
        (c.to_string(), ps.to_vec())
    }
    fn piece_rows(db: &UserDb) -> Vec<(i64, u32, u32, u32, u32, String)> {
        let mut st = db
            .conn
            .prepare("SELECT mark_id, verse, start_cp, end_cp, verse_len, verse_fp FROM mark_piece ORDER BY 1, 2, 3")
            .unwrap();
        st.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }
    fn model(db: &UserDb, t: &[(u32, String)]) -> Vec<Mark> {
        db.marks(&KEY, t)
            .unwrap()
            .into_iter()
            .filter(|m| m.pieces.iter().any(|p| !p.stale))
            .map(|m| Mark {
                id: Some(m.id),
                color: m.color,
                created_at: m.created_at,
                pieces: m
                    .pieces
                    .iter()
                    .filter(|p| !p.stale)
                    .map(|p| pc(p.verse, p.start, p.end))
                    .collect(),
            })
            .collect()
    }

    #[test]
    fn pen_strokes_overlap_merge_and_last_write_wins() {
        let d = temp_dir("user-pen");
        let path = d.join("user.sqlite");
        let t = texts(&["alpha beta gamma delta", "epsilon zeta eta ", "theta iota"]);
        let mut db = UserDb::open(&path).unwrap();
        // two gold words a space apart become one mark
        db.pen_apply(&KEY, &t, "gold", &[pc(1, 0, 5)]).unwrap();
        let out = db.pen_apply(&KEY, &t, "gold", &[pc(1, 6, 10)]).unwrap();
        assert_eq!(shape(&out), vec![s("gold", &[(1, 0, 10)])]);
        // blue over "beta gamma": cuts gold back to "alpha"
        let out = db.pen_apply(&KEY, &t, "blue", &[pc(1, 6, 16)]).unwrap();
        assert_eq!(
            shape(&out),
            vec![s("blue", &[(1, 6, 16)]), s("gold", &[(1, 0, 5)])]
        );
        // a cross-verse gold stroke from "delta" to "zeta"
        let out = db
            .pen_apply(&KEY, &t, "gold", &[pc(1, 17, 22), pc(2, 0, 12)])
            .unwrap();
        assert_eq!(
            shape(&out),
            vec![
                s("blue", &[(1, 6, 16)]),
                s("gold", &[(1, 0, 5)]),
                s("gold", &[(1, 17, 22), (2, 0, 12)])
            ]
        );
        // same colour in the next verse after only trailing whitespace joins across the boundary
        db.pen_apply(&KEY, &t, "gold", &[pc(2, 13, 16)]).unwrap();
        let out = db.pen_apply(&KEY, &t, "gold", &[pc(3, 0, 5)]).unwrap();
        assert_eq!(
            shape(&out),
            vec![
                s("blue", &[(1, 6, 16)]),
                s("gold", &[(1, 0, 5)]),
                s("gold", &[(1, 17, 22), (2, 0, 16), (3, 0, 5)])
            ]
        );
        marks::check_invariant(&model(&db, &t), &ChapterText::new(&t)).unwrap();
        // the eraser cuts any colour; a mark cut in the middle stays one mark
        let out = db.pen_erase(&KEY, &t, &[pc(2, 0, 7)]).unwrap();
        assert_eq!(
            shape(&out),
            vec![
                s("blue", &[(1, 6, 16)]),
                s("gold", &[(1, 0, 5)]),
                s("gold", &[(1, 17, 22), (2, 8, 16), (3, 0, 5)])
            ]
        );
        // recolour joins same-colour neighbours; remove takes a whole mark
        let blue = out.iter().find(|m| m.color == "blue").unwrap().id;
        let out = db.mark_recolor(&KEY, &t, blue, "gold").unwrap();
        assert_eq!(
            shape(&out)[0],
            s("gold", &[(1, 0, 22), (2, 8, 16), (3, 0, 5)])
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, blue, "the recoloured mark keeps its id");
        // undo snapshot: replace the chapter with an earlier set
        let snap = vec![MarkIn {
            color: "rose".into(),
            created_at: 5,
            pieces: vec![pc(3, 6, 10)],
        }];
        let out = db.marks_replace_chapter(&KEY, &t, &snap).unwrap();
        assert_eq!(shape(&out), vec![s("rose", &[(3, 6, 10)])]);
        assert_eq!(out[0].created_at, 5);
        let out = db.mark_remove(&KEY, &t, out[0].id).unwrap();
        assert!(out.is_empty());
        let marks_left: i64 = db
            .conn
            .query_row("SELECT count(*) FROM mark", [], |r| r.get(0))
            .unwrap();
        assert_eq!(marks_left, 0, "an emptied mark is deleted with its pieces");
        drop(db);
        // and every write is durable across reopen
        let mut db = UserDb::open(&path).unwrap();
        db.pen_apply(&KEY, &t, "violet", &[pc(1, 0, 5)]).unwrap();
        drop(db);
        assert_eq!(
            shape(&UserDb::open(&path).unwrap().marks(&KEY, &t).unwrap()),
            vec![s("violet", &[(1, 0, 5)])]
        );
    }

    #[test]
    fn pen_writes_are_validated_and_nothing_is_written_on_a_bad_call() {
        let d = temp_dir("user-pen-bad");
        let t = texts(&["cafe\u{301} \u{1D400}leph end", "two"]);
        let mut db = UserDb::open(&d.join("user.sqlite")).unwrap();
        db.pen_apply(&KEY, &t, "gold", &[pc(2, 0, 3)]).unwrap();
        let before = piece_rows(&db);
        let bad: Vec<Vec<Piece>> = vec![
            vec![],
            vec![pc(1, 0, 0)],
            vec![pc(1, 3, 2)],
            vec![pc(1, 0, 16)], // one past the end (15 code points)
            vec![pc(1, 0, 4)],  // before U+0301 (B1)
            vec![pc(1, 4, 6)],  // starts on U+0301
            vec![pc(3, 0, 1)],  // no verse 3
            vec![pc(1, 4294967294, 4294967295)],
            vec![pc(1, 6, 15), pc(2, 1, 3)], // not contiguous
            vec![pc(1, 5, 6)],               // only a space
        ];
        for p in &bad {
            assert!(db.pen_apply(&KEY, &t, "gold", p).is_err(), "{p:?}");
            assert!(db.pen_erase(&KEY, &t, p).is_err(), "{p:?}");
        }
        assert!(db.pen_apply(&KEY, &t, "red", &[pc(2, 0, 3)]).is_err());
        assert!(db.mark_recolor(&KEY, &t, 1, "Gold").is_err());
        assert!(db.mark_remove(&KEY, &t, 999).is_err());
        for k in [
            ChapterKey {
                translation: "",
                ..KEY
            },
            ChapterKey {
                translation: "../x",
                ..KEY
            },
            ChapterKey {
                translation: "PH1' OR 1=1 --",
                ..KEY
            },
            ChapterKey {
                book: "Test book",
                ..KEY
            },
            ChapterKey { chapter: 0, ..KEY },
            ChapterKey {
                chapter: 1000,
                ..KEY
            },
        ] {
            assert!(
                db.pen_apply(&k, &t, "gold", &[pc(2, 0, 3)]).is_err(),
                "{k:?}"
            );
            assert!(db.marks(&k, &t).is_err(), "{k:?}");
        }
        // overlapping pieces in an undo snapshot
        let snap = |a: (u32, u32), b: (u32, u32)| {
            vec![
                MarkIn {
                    color: "gold".into(),
                    created_at: 1,
                    pieces: vec![pc(1, a.0, a.1)],
                },
                MarkIn {
                    color: "blue".into(),
                    created_at: 1,
                    pieces: vec![pc(1, b.0, b.1)],
                },
            ]
        };
        assert!(
            db.marks_replace_chapter(&KEY, &t, &snap((0, 5), (3, 14)))
                .is_err()
        );
        let neg = vec![MarkIn {
            color: "gold".into(),
            created_at: -1,
            pieces: vec![pc(2, 0, 3)],
        }];
        assert!(db.marks_replace_chapter(&KEY, &t, &neg).is_err());
        assert_eq!(piece_rows(&db), before, "nothing written by a refused call");
        // astral: U+1D400 is one code point, so the word starting with it is [6, 11)
        let out = db.pen_apply(&KEY, &t, "rose", &[pc(1, 6, 11)]).unwrap();
        assert!(shape(&out).contains(&s("rose", &[(1, 6, 11)])));
        // the backend computed length and fingerprint: 15 code points, SHA-256 prefix
        let rows = piece_rows(&db);
        let r = rows.iter().find(|r| r.1 == 1).unwrap();
        assert_eq!(
            (r.4, r.5.as_str()),
            (15, marks::fingerprint(&t[0].1).as_str())
        );
        // IPC shapes refuse unknown fields
        assert!(
            serde_json::from_str::<Piece>(r#"{"verse":1,"start":0,"end":1,"verse_len":9}"#)
                .is_err()
        );
        assert!(serde_json::from_str::<Piece>(r#"{"verse":1,"start":-1,"end":1}"#).is_err());
    }

    #[test]
    fn a_changed_verse_text_makes_pieces_stale_and_they_are_never_moved() {
        let d = temp_dir("user-pen-stale");
        let t1 = texts(&["alpha beta gamma", "delta epsilon"]);
        let t2 = texts(&["alpha BETA gamma", "delta epsilon"]); // verse 1 re-issued
        let mut db = UserDb::open(&d.join("user.sqlite")).unwrap();
        db.pen_apply(&KEY, &t1, "gold", &[pc(1, 6, 10)]).unwrap();
        db.pen_apply(&KEY, &t1, "blue", &[pc(2, 0, 5)]).unwrap();
        let rows_before = piece_rows(&db);
        let out = db.marks(&KEY, &t2).unwrap();
        let gold = out.iter().find(|m| m.color == "gold").unwrap();
        assert!(gold.pieces.iter().all(|p| p.stale), "listed, not drawn");
        assert!(
            out.iter()
                .find(|m| m.color == "blue")
                .unwrap()
                .pieces
                .iter()
                .all(|p| !p.stale)
        );
        // a stroke over the changed verse neither cuts nor moves the stale piece
        db.pen_apply(&KEY, &t2, "green", &[pc(1, 0, 16)]).unwrap();
        db.pen_erase(&KEY, &t2, &[pc(1, 0, 16)]).unwrap();
        let rows = piece_rows(&db);
        let stale_row = rows_before.iter().find(|r| r.1 == 1).unwrap();
        assert!(rows.contains(stale_row), "kept byte for byte: {rows:?}");
        // the original text back: drawn again, at the same place
        let out = db.marks(&KEY, &t1).unwrap();
        assert_eq!(
            shape(&out),
            vec![s("blue", &[(2, 0, 5)]), s("gold", &[(1, 6, 10)])]
        );
        // a verse that no longer exists is stale too
        let fewer = texts(&["alpha beta gamma"]);
        let out = db.marks(&KEY, &fewer).unwrap();
        assert!(out.iter().find(|m| m.color == "blue").unwrap().pieces[0].stale);
    }

    #[test]
    fn marks_are_per_translation_and_chapter() {
        let d = temp_dir("user-pen-tr");
        let t = texts(&["alpha beta"]);
        let mut db = UserDb::open(&d.join("user.sqlite")).unwrap();
        db.pen_apply(&KEY, &t, "gold", &[pc(1, 0, 5)]).unwrap();
        for k in [
            ChapterKey {
                translation: "PH2",
                ..KEY
            },
            ChapterKey {
                translation: "ph1",
                ..KEY
            },
            ChapterKey { chapter: 2, ..KEY },
            ChapterKey {
                book: "testbook",
                ..KEY
            },
        ] {
            assert!(db.marks(&k, &t).unwrap().is_empty(), "{k:?}");
        }
        // a write in another translation does not touch this one's marks
        db.pen_apply(
            &ChapterKey {
                translation: "PH2",
                ..KEY
            },
            &t,
            "blue",
            &[pc(1, 0, 10)],
        )
        .unwrap();
        assert_eq!(
            shape(&db.marks(&KEY, &t).unwrap()),
            vec![s("gold", &[(1, 0, 5)])]
        );
    }

    #[test]
    fn tampered_rows_are_never_drawn_and_a_pieceless_mark_goes_on_the_next_write() {
        let d = temp_dir("user-pen-tamper");
        let t = texts(&["alpha beta gamma"]);
        let mut db = UserDb::open(&d.join("user.sqlite")).unwrap();
        let fp = marks::fingerprint(&t[0].1);
        // the CHECKs refuse bad rows
        for sql in [
            "INSERT INTO mark VALUES (50, 'PH1', 'Testbook', 1, 'red', 0, 0)",
            "INSERT INTO mark VALUES (51, '', 'Testbook', 1, 'gold', 0, 0)",
            "INSERT INTO mark_piece VALUES (999, 1, 0, 5, 16, '0123456789abcdef')", // no such mark
        ] {
            assert!(db.conn.execute(sql, []).is_err(), "{sql}");
        }
        db.conn
            .execute_batch(&format!(
                "INSERT INTO mark VALUES (60, 'PH1', 'Testbook', 1, 'gold', 0, 0);
             INSERT INTO mark VALUES (61, 'PH1', 'Testbook', 1, 'blue', 0, 0);
             INSERT INTO mark_piece VALUES (61, 1, 0, 10, 16, '{fp}');
             INSERT INTO mark VALUES (62, 'PH1', 'Testbook', 1, 'rose', 0, 0);
             INSERT INTO mark_piece VALUES (62, 1, 6, 16, 16, '{fp}');
             INSERT INTO mark VALUES (63, 'PH1', 'Testbook', 1, 'green', 0, 0);
             INSERT INTO mark_piece VALUES (63, 1, 1, 3, 16, '{fp}');"
            ))
            .unwrap();
        // 60 has no pieces: not shown; 62 overlaps 61: set aside; 63's
        // boundaries are fine but it overlaps 61 too
        let out = db.marks(&KEY, &t).unwrap();
        assert!(out.iter().all(|m| m.id != 60));
        assert!(out.iter().find(|m| m.id == 62).unwrap().pieces[0].stale);
        assert!(!out.iter().find(|m| m.id == 61).unwrap().pieces[0].stale);
        // the next write keeps the invariant among drawn pieces and drops mark 60
        db.pen_apply(&KEY, &t, "violet", &[pc(1, 11, 16)]).unwrap();
        let ids: Vec<i64> = db
            .conn
            .prepare("SELECT id FROM mark ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(!ids.contains(&60), "{ids:?}");
        marks::check_invariant(&model(&db, &t), &ChapterText::new(&t)).unwrap();
    }

    #[test]
    fn the_chapter_cap_refuses_more_pieces_but_never_an_erase() {
        let d = temp_dir("user-pen-cap");
        // 999 three-word verses; three pieces per verse in three colours (no merging)
        let vs: Vec<String> = (1..=999).map(|i| format!("w{i} x{i} y{i}")).collect();
        let t: Vec<(u32, String)> = vs
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u32 + 1, s.clone()))
            .collect();
        let mut db = UserDb::open(&d.join("user.sqlite")).unwrap();
        let words = |v: u32| {
            let s = &t[v as usize - 1].1;
            let (w, x) = (s.find(' ').unwrap() as u32, s.rfind(' ').unwrap() as u32);
            [
                pc(v, 0, w),
                pc(v, w + 1, x),
                pc(v, x + 1, s.chars().count() as u32),
            ]
        };
        // 666 verses x 3 = 1,998 pieces in one snapshot (one mark per colour)
        let snap: Vec<MarkIn> = (0..3)
            .map(|c| MarkIn {
                color: COLORS[c].into(),
                created_at: 1,
                pieces: (1..=666).map(|v| words(v)[c]).collect(),
            })
            .collect();
        db.marks_replace_chapter(&KEY, &t, &snap).unwrap();
        db.pen_apply(&KEY, &t, "rose", &[words(700)[0]]).unwrap();
        db.pen_apply(&KEY, &t, "rose", &[words(700)[2]]).unwrap();
        assert_eq!(piece_rows(&db).len(), marks::MAX_CHAPTER_PIECES);
        let e = db
            .pen_apply(&KEY, &t, "rose", &[words(701)[0]])
            .unwrap_err();
        assert!(e.to_string().contains("2,000"), "{e}");
        assert_eq!(piece_rows(&db).len(), marks::MAX_CHAPTER_PIECES);
        db.pen_erase(&KEY, &t, &[pc(1, 0, 8)]).unwrap();
        assert_eq!(piece_rows(&db).len(), marks::MAX_CHAPTER_PIECES - 3);
    }

    /// Kill mid-write (11 E-durability): a child process writes strokes in a
    /// loop and is killed at an arbitrary moment; the file then opens, passes
    /// `integrity_check` and holds a consistent chapter (the invariant), with
    /// every stroke either wholly there or wholly absent.
    #[test]
    fn a_kill_mid_write_leaves_a_consistent_file() {
        const CHILD: &str = "REVELATION_PEN_KILL_CHILD";
        let t = texts(&[
            "In the beginning was the Word, and the Word was with God.",
            "The same was in the beginning with God.",
            "All things were made by him",
        ]);
        if let Ok(p) = std::env::var(CHILD) {
            let mut db = UserDb::open(Path::new(&p)).unwrap();
            let mut i: u32 = 0;
            loop {
                let v = 1 + i % 3;
                let len = t[v as usize - 1].1.chars().count() as u32;
                let a = (i * 7) % (len - 3);
                let b = a + 1 + (i * 13) % (len - a - 1);
                let piece = [pc(v, a, b)];
                let r = if i % 4 == 3 {
                    db.pen_erase(&KEY, &t, &piece)
                } else {
                    db.pen_apply(&KEY, &t, COLORS[(i % 5) as usize], &piece)
                };
                let _ = r; // a whitespace-only piece is refused; keep going
                i = i.wrapping_add(1);
            }
        }
        let d = temp_dir("user-pen-kill");
        let path = d.join("user.sqlite");
        drop(UserDb::open(&path).unwrap());
        for round in 0..3 {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "user::tests::a_kill_mid_write_leaves_a_consistent_file",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, &path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            // let it write a few strokes (polling a read-only connection), then
            // kill it without warning at an arbitrary moment
            let start = std::time::Instant::now();
            loop {
                let n: i64 = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .and_then(|c| c.query_row("SELECT count(*) FROM mark_piece", [], |r| r.get(0)))
                    .unwrap_or(0);
                if n >= 3 || start.elapsed().as_secs() > 60 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            std::thread::sleep(std::time::Duration::from_millis(30 + 41 * round));
            child.kill().unwrap();
            child.wait().unwrap();
            let mut db = UserDb::open(&path).unwrap_or_else(|e| panic!("round {round}: {e}"));
            let ms = model(&db, &t);
            assert!(!ms.is_empty(), "round {round}: the child wrote nothing");
            marks::check_invariant(&ms, &ChapterText::new(&t))
                .unwrap_or_else(|e| panic!("round {round}: {e}"));
            let orphans: i64 = db
                .conn
                .query_row(
                    "SELECT count(*) FROM mark_piece WHERE mark_id NOT IN (SELECT id FROM mark)",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(orphans, 0);
            // and it still takes writes
            db.pen_apply(&KEY, &t, "gold", &[pc(3, 0, 3)]).unwrap();
        }
    }

    #[test]
    fn bookmarks_are_one_per_book_cleared_and_restored_exactly() {
        let d = temp_dir("user-bm");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.set_bookmark("Luke", 12).unwrap();
        db.set_bookmark("Ps", 24).unwrap();
        // a newer read replaces it, even an earlier chapter (Q37)
        db.set_bookmark("Luke", 3).unwrap();
        let all = db.bookmarks().unwrap();
        assert_eq!(
            all.iter()
                .map(|b| (b.book.as_str(), b.chapter))
                .collect::<Vec<_>>(),
            [("Luke", 3), ("Ps", 24)]
        );
        assert!(all.iter().all(|b| (b.read_at - now()).abs() < 60));
        // clear returns the old row; restore puts that exact row back
        let old = Bookmark {
            book: "Ps".into(),
            chapter: 24,
            read_at: 1_700_000_000,
        };
        db.restore_bookmark(&old).unwrap();
        assert_eq!(db.clear_bookmark("Ps").unwrap(), Some(old.clone()));
        assert_eq!(db.clear_bookmark("Ps").unwrap(), None);
        assert_eq!(db.bookmarks().unwrap().len(), 1);
        db.restore_bookmark(&old).unwrap();
        drop(db);
        let mut db = UserDb::open(&path).unwrap();
        assert!(db.bookmarks().unwrap().contains(&old));
        // validation: nothing is written for a bad value
        let before = db.bookmarks().unwrap();
        let long = "x".repeat(17);
        for (book, ch) in [
            ("", 1),
            ("Lu ke", 1),
            ("%", 1),
            ("Luke", 0),
            ("Luke", 1000),
            (long.as_str(), 1),
        ] {
            assert!(db.set_bookmark(book, ch).is_err(), "{book} {ch}");
        }
        for read_at in [-1, MAX_READ_AT + 1] {
            let b = Bookmark {
                book: "Luke".into(),
                chapter: 1,
                read_at,
            };
            assert!(db.restore_bookmark(&b).is_err());
        }
        assert!(db.clear_bookmark("a'b").is_err());
        assert_eq!(db.bookmarks().unwrap(), before);
        // the IPC shape: unknown fields are refused
        assert!(
            serde_json::from_str::<Bookmark>(r#"{"book":"Luke","chapter":1,"read_at":0,"x":1}"#)
                .is_err()
        );
    }

    // ---- daily notes (W2-5) ----

    /// A day's document: one paragraph tagged John 3:16 with `text`.
    fn note_doc(text: &str) -> String {
        serde_json::json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "attrs": { "tags": [{ "book": "John", "chapter": 3, "v1": 16, "v2": 16 }] },
                "content": [{ "type": "text", "text": text }],
            }],
        })
        .to_string()
    }
    const EMPTY_DOC: &str =
        r#"{"type":"doc","content":[{"type":"paragraph","attrs":{"tags":null}}]}"#;

    fn note_rows(db: &UserDb) -> i64 {
        db.conn
            .query_row("SELECT count(*) FROM note_day", [], |r| r.get(0))
            .unwrap()
    }

    fn created_at(db: &UserDb, day: &str) -> i64 {
        db.conn
            .query_row(
                "SELECT created_at FROM note_day WHERE day = ?1",
                params![day],
                |r| r.get(0),
            )
            .unwrap()
    }

    #[test]
    fn notes_save_and_restore_across_reopen_and_empty_days_are_not_kept() {
        let d = temp_dir("user-notes");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.note_save("2026-10-26", &note_doc("yesterday's words"))
            .unwrap();
        db.note_save("2026-10-27", &note_doc("first")).unwrap();
        let created = created_at(&db, "2026-10-27");
        db.note_save("2026-10-27", &note_doc("first, then more"))
            .unwrap();
        drop(db);
        // reopened (a fresh process would see the same file)
        let mut db = UserDb::open(&path).unwrap();
        let page = db.notes_list(None, 30).unwrap();
        let got: Vec<(&str, Option<&str>)> = page
            .days
            .iter()
            .map(|n| (n.day.as_str(), n.doc.as_deref()))
            .collect();
        let (a, b) = (note_doc("first, then more"), note_doc("yesterday's words"));
        assert_eq!(
            got,
            [
                ("2026-10-27", Some(a.as_str())),
                ("2026-10-26", Some(b.as_str()))
            ]
        );
        assert!(!page.more);
        assert_eq!(
            created_at(&db, "2026-10-27"),
            created,
            "an update keeps the day's first save time"
        );
        // all text deleted: the day goes (8.1: empty days aren't stored)
        db.note_save("2026-10-27", EMPTY_DOC).unwrap();
        db.note_save("2026-10-25", EMPTY_DOC).unwrap();
        let page = db.notes_list(None, 30).unwrap();
        assert_eq!(page.days.len(), 1);
        assert_eq!(page.days[0].day, "2026-10-26");
        // committed data is in user.sqlite alone (rollback journal, no -wal)
        drop(db);
        assert!(!sidecar(&path, "-wal").exists() && !sidecar(&path, "-journal").exists());
    }

    #[test]
    fn notes_page_newest_first_and_bad_calls_write_nothing() {
        let d = temp_dir("user-notes-page");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        for day in 1..=9 {
            db.note_save(&format!("2026-09-0{day}"), &note_doc(&format!("day {day}")))
                .unwrap();
        }
        let days = |p: &NotesPage| p.days.iter().map(|n| n.day.clone()).collect::<Vec<_>>();
        let p1 = db.notes_list(None, 4).unwrap();
        assert_eq!(
            days(&p1),
            ["2026-09-09", "2026-09-08", "2026-09-07", "2026-09-06"]
        );
        assert!(p1.more);
        let p2 = db.notes_list(Some("2026-09-06"), 4).unwrap();
        assert_eq!(
            days(&p2),
            ["2026-09-05", "2026-09-04", "2026-09-03", "2026-09-02"]
        );
        let p3 = db.notes_list(Some("2026-09-02"), 4).unwrap();
        assert_eq!(days(&p3), ["2026-09-01"]);
        assert!(!p3.more);
        assert_eq!(
            db.notes_list(None, 10_000).unwrap().days.len(),
            9,
            "capped, not refused"
        );
        assert!(db.notes_list(Some("x' OR 1=1 --"), 4).is_err());

        drop(db);
        let before = sha(&path);
        let mut db = UserDb::open(&path).unwrap();
        let link = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}"#;
        let too_big = note_doc(&"a".repeat(notes::MAX_NOTE_BYTES));
        let ok = note_doc("x");
        for (day, doc) in [
            ("2026-09-31", ok.as_str()),
            ("2026-10-27'; DROP TABLE note_day; --", ok.as_str()),
            ("../2026-10-27", ok.as_str()),
            ("2026-10-27", link),
            ("2026-10-27", "<p>hi</p>"),
            ("2026-10-27", too_big.as_str()),
        ] {
            assert!(db.note_save(day, doc).is_err(), "{day}");
        }
        assert_eq!(note_rows(&db), 9);
        drop(db);
        assert_eq!(
            sha(&path),
            before,
            "a refused save leaves the file byte-identical"
        );
    }

    #[test]
    fn a_damaged_note_row_is_reported_unreadable_and_left_untouched() {
        let d = temp_dir("user-notes-bad");
        let path = d.join("user.sqlite");
        let mut db = UserDb::open(&path).unwrap();
        db.note_save("2026-10-27", &note_doc("good")).unwrap();
        // edited by hand: an image with a script handler
        let evil = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"image","attrs":{"src":"x","onerror":"alert(1)"}}]}]}"#;
        db.conn
            .execute(
                "INSERT INTO note_day VALUES ('2026-10-26', ?1, 1, 1)",
                params![evil],
            )
            .unwrap();
        let page = db.notes_list(None, 30).unwrap();
        assert_eq!(page.days.len(), 2);
        assert_eq!(page.days[1].day, "2026-10-26");
        assert_eq!(page.days[1].doc, None, "never handed to the editor");
        let raw: String = db
            .conn
            .query_row(
                "SELECT doc FROM note_day WHERE day = '2026-10-26'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(raw, evil, "kept exactly");
    }

    /// Kill the writer at an arbitrary moment while it saves a growing note
    /// over and over (one transaction per save, rollback journal, synchronous
    /// FULL): the file opens, passes integrity_check, and holds one complete
    /// saved version, never older than the last one seen committed.
    #[test]
    fn a_kill_mid_note_save_leaves_the_last_committed_version() {
        const CHILD: &str = "REVELATION_NOTE_KILL_CHILD";
        let doc = |i: u64| {
            let words: String = (0..=i % 200).map(|k| format!("word{k} ")).collect();
            note_doc(&format!("save {i:08} {words}"))
        };
        let saved_i = |db: &UserDb| -> Option<u64> {
            let page = db.notes_list(None, 30).ok()?;
            let d = page.days.first()?.doc.clone().expect("a valid document");
            let at = d.find("save ")? + 5;
            d[at..at + 8].parse().ok()
        };
        if let Ok(p) = std::env::var(CHILD) {
            let mut db = UserDb::open(Path::new(&p)).unwrap();
            let start: u64 = std::env::var("REVELATION_NOTE_KILL_FROM")
                .unwrap()
                .parse()
                .unwrap();
            for i in start..=u64::MAX {
                db.note_save("2026-10-27", &doc(i)).unwrap();
            }
        }
        let d = temp_dir("user-note-kill");
        let path = d.join("user.sqlite");
        drop(UserDb::open(&path).unwrap());
        let mut from = 0;
        for round in 0..3u64 {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "user::tests::a_kill_mid_note_save_leaves_the_last_committed_version",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, &path)
                .env("REVELATION_NOTE_KILL_FROM", from.to_string())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            // watch through a read-only connection until a few saves landed
            let start = std::time::Instant::now();
            let mut seen = 0;
            loop {
                let n: Option<String> =
                    Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                        .and_then(|c| {
                            c.query_row("SELECT doc FROM note_day", [], |r| r.get(0))
                                .optional()
                        })
                        .unwrap_or(None);
                if let Some(d) = n
                    && let Some(at) = d.find("save ")
                    && let Ok(i) = d[at + 5..at + 13].parse::<u64>()
                {
                    seen = i;
                    if i >= from + 5 {
                        break;
                    }
                }
                if start.elapsed().as_secs() > 60 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
            std::thread::sleep(std::time::Duration::from_millis(20 + 37 * round));
            child.kill().unwrap();
            child.wait().unwrap();
            let mut db = UserDb::open(&path).unwrap_or_else(|e| panic!("round {round}: {e}"));
            let got = saved_i(&db).unwrap_or_else(|| panic!("round {round}: nothing saved"));
            assert!(
                got >= seen && got >= from + 5,
                "round {round}: {got} < {seen}"
            );
            assert_eq!(
                db.notes_list(None, 30).unwrap().days[0].doc.as_deref(),
                Some(doc(got).as_str()),
                "round {round}: one complete version"
            );
            assert_eq!(note_rows(&db), 1);
            // and it still takes writes
            db.note_save("2026-10-28", &note_doc("after")).unwrap();
            db.note_save("2026-10-28", EMPTY_DOC).unwrap();
            from = got + 1;
        }
    }
}
