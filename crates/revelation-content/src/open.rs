//! `open_verified` and the capped, link-refusing file read it is built on.

use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

use rusqlite::Connection;
use rusqlite::config::DbConfig;
use sha2::{Digest, Sha256};

/// The largest content database read into memory (pipeline/DESIGN.md 9.1).
pub const MAX_DB: u64 = 256 * 1024 * 1024;

/// `PRAGMA application_id` of every content file: "RVLN" (DESIGN.md 8).
pub const APPLICATION_ID: i64 = 0x5256_4C4E;
/// `PRAGMA user_version` = content schema version (DESIGN.md 8).
pub const SCHEMA_VERSION: i64 = 1;

/// Why a file can't be used. Messages carry a file name (never a full path)
/// and never any file content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The path can't be stat'ed (usually: it does not exist).
    Missing {
        file: String,
        detail: String,
    },
    /// A symbolic link, junction or other reparse point, or not the wanted
    /// kind (regular file / directory). `why` says which.
    NotPlain {
        file: String,
        why: &'static str,
    },
    /// Over the size cap (by metadata, or grew past it while being read).
    TooLarge {
        file: String,
        detail: String,
    },
    Io {
        file: String,
        detail: String,
    },
    /// The bytes read do not have the expected SHA-256.
    HashMismatch {
        file: String,
        actual: String,
        expected: String,
    },
    /// Not an SQLite file, or not in rollback-journal mode (header bytes 18/19).
    Header(String),
    /// Opens as SQLite but is not a Revelation content database of this schema.
    NotContent(String),
    Sqlite(String),
}

impl OpenError {
    /// The pipeline's check name for this failure (pipeline/src/error.rs).
    pub fn check(&self) -> &'static str {
        match self {
            OpenError::Missing { .. } | OpenError::NotPlain { .. } => "path",
            OpenError::TooLarge { .. } => "size",
            OpenError::Io { .. } => "io",
            OpenError::HashMismatch { .. } => "sha256",
            OpenError::Header(_) => "header",
            OpenError::NotContent(_) => "content",
            OpenError::Sqlite(_) => "open",
        }
    }

    /// The file name the error is about, if any.
    pub fn file(&self) -> Option<&str> {
        match self {
            OpenError::Missing { file, .. }
            | OpenError::NotPlain { file, .. }
            | OpenError::TooLarge { file, .. }
            | OpenError::Io { file, .. }
            | OpenError::HashMismatch { file, .. } => Some(file),
            _ => None,
        }
    }

    /// What went wrong, without the file name.
    pub fn reason(&self) -> String {
        match self {
            OpenError::Missing { detail, .. } => format!("cannot stat: {detail}"),
            OpenError::NotPlain { why, .. } => why.to_string(),
            OpenError::TooLarge { detail, .. } => detail.clone(),
            OpenError::Io { detail, .. } => detail.clone(),
            OpenError::HashMismatch { .. } => {
                "failed its checksum (damaged or replaced)".to_string()
            }
            OpenError::Header(why) => format!("not a content database: {why}"),
            OpenError::NotContent(why) => format!("not a Revelation content database: {why}"),
            OpenError::Sqlite(e) => format!("SQLite: {e}"),
        }
    }
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.file() {
            Some(file) => write!(f, "{file}: {}", self.reason()),
            None => write!(f, "{}", self.reason()),
        }
    }
}

impl std::error::Error for OpenError {}

fn sql_err(e: rusqlite::Error) -> OpenError {
    OpenError::Sqlite(e.to_string())
}

fn shown(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Lower-case hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest.iter() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Refuses symbolic links, junctions and other reparse points, and anything
/// that is not a plain directory (`want_dir`) or a regular file.
pub fn check_plain(path: &Path, want_dir: bool) -> Result<(), OpenError> {
    let file = shown(path);
    let meta = fs::symlink_metadata(path).map_err(|e| OpenError::Missing {
        file: file.clone(),
        detail: e.to_string(),
    })?;
    if meta.file_type().is_symlink() {
        return Err(OpenError::NotPlain {
            file,
            why: "is a symbolic link",
        });
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(OpenError::NotPlain {
                file,
                why: "is a reparse point (junction or link)",
            });
        }
    }
    let ok = if want_dir {
        meta.file_type().is_dir()
    } else {
        meta.file_type().is_file()
    };
    if !ok {
        let why = if want_dir {
            "is not a directory"
        } else {
            "is not a regular file"
        };
        return Err(OpenError::NotPlain { file, why });
    }
    Ok(())
}

/// Reads a regular file of at most `cap` bytes: checks the metadata size
/// first, then reads at most `cap + 1` bytes, so a file that grows after the
/// size check is still refused without a full read.
pub fn read_capped(path: &Path, cap: u64) -> Result<Vec<u8>, OpenError> {
    let file = shown(path);
    check_plain(path, false)?;
    let io = |what: &str, e: std::io::Error| OpenError::Io {
        file: file.clone(),
        detail: format!("{what}: {e}"),
    };
    let mut f = fs::File::open(path).map_err(|e| io("cannot open", e))?;
    let len = f.metadata().map_err(|e| io("cannot stat", e))?.len();
    if len > cap {
        return Err(OpenError::TooLarge {
            file,
            detail: format!("{len} bytes is over the cap of {cap}"),
        });
    }
    let mut buf = Vec::with_capacity(len as usize);
    (&mut f)
        .take(cap + 1)
        .read_to_end(&mut buf)
        .map_err(|e| io("read failed", e))?;
    if buf.len() as u64 > cap {
        return Err(OpenError::TooLarge {
            file,
            detail: format!("file grew past the cap of {cap} bytes"),
        });
    }
    Ok(buf)
}

/// Reads `path` once (at most `cap` bytes, regular file, no link or reparse
/// point), computes its SHA-256 and, if `expected` is given, requires it to
/// match; then opens **that buffer** read-only ([`open_bytes`]). Returns the
/// connection and the hash.
///
/// The app always passes the value compiled into it (never one read from
/// beside the file); the pipeline's check stage passes `None`, because the
/// hash is the value it is producing (DESIGN.md 10.3).
pub fn open_verified(
    path: &Path,
    expected: Option<&str>,
    cap: u64,
) -> Result<(Connection, String), OpenError> {
    let bytes = read_capped(path, cap)?;
    let hash = sha256_hex(&bytes);
    if let Some(want) = expected
        && want != hash
    {
        return Err(OpenError::HashMismatch {
            file: shown(path),
            actual: hash,
            expected: want.to_string(),
        });
    }
    let conn = open_bytes(&bytes)?;
    Ok((conn, hash))
}

/// Opens an in-memory copy of `bytes` read-only and locked down (DESIGN.md
/// 10.3 rule 2, as far as the cleared rusqlite features allow). The caller
/// has hashed them.
pub fn open_bytes(bytes: &[u8]) -> Result<Connection, OpenError> {
    if bytes.len() < 100 || !bytes.starts_with(b"SQLite format 3\0") {
        return Err(OpenError::Header("not an SQLite database".into()));
    }
    // Bytes 18 and 19: file format write/read version, 1 = rollback journal,
    // 2 = WAL. Content files are never WAL.
    if bytes[18] != 1 || bytes[19] != 1 {
        return Err(OpenError::Header(
            "header bytes 18/19 are not 01 01 (WAL or unknown mode)".into(),
        ));
    }
    let mut conn = Connection::open_in_memory().map_err(sql_err)?;
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
        .map_err(sql_err)?;
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA, false)
        .map_err(sql_err)?;
    // Content files have no triggers or views (the schema check requires
    // that); refuse to run any a crafted file might carry.
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER, false)
        .map_err(sql_err)?;
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_VIEW, false)
        .map_err(sql_err)?;
    conn.deserialize_read_exact("main", bytes, bytes.len(), true)
        .map_err(sql_err)?;
    conn.execute_batch(
        "PRAGMA query_only = 1; PRAGMA cell_size_check = ON; PRAGMA trusted_schema = OFF;",
    )
    .map_err(sql_err)?;
    Ok(conn)
}

/// Requires `application_id` = "RVLN" and `user_version` = [`SCHEMA_VERSION`]
/// (DESIGN.md 8). The app calls it on every content file it opens.
pub fn require_content_identity(conn: &Connection) -> Result<(), OpenError> {
    let app_id: i64 = conn
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(sql_err)?;
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(sql_err)?;
    if app_id != APPLICATION_ID {
        return Err(OpenError::NotContent(format!("application_id {app_id:#x}")));
    }
    if version != SCHEMA_VERSION {
        return Err(OpenError::NotContent(format!("schema version {version}")));
    }
    Ok(())
}
