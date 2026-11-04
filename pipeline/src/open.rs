//! `open_verified` (DESIGN.md 10.3 "Best", M1-F1): read the whole file once,
//! hash it, compare with the expected value (when one is given), and open
//! **that same buffer** as a read-only in-memory database with
//! `sqlite3_deserialize`.
//!
//! The implementation lives in `crates/revelation-content`, shared with the
//! app core, so there is exactly one; these wrappers only turn its error into
//! the pipeline's [`Error`] (same check names: `path`, `size`, `io`,
//! `sha256`, `header`, `open`).

use std::path::Path;

use rusqlite::Connection;

use crate::error::Error;

/// The largest content database read into memory (DESIGN.md 9.1).
pub use revelation_content::MAX_DB;

/// Reads `path` (at most 256 MiB, regular file, no reparse point), computes
/// its SHA-256 and, if `expected` is given, requires it to match. Returns the
/// read-only connection and the hash.
pub fn open_verified(path: &Path, expected: Option<&str>) -> Result<(Connection, String), Error> {
    revelation_content::open_verified(path, expected, MAX_DB).map_err(Error::from)
}

/// Opens an in-memory copy of `bytes` read-only. The caller has hashed them.
pub fn open_bytes(bytes: &[u8]) -> Result<Connection, Error> {
    revelation_content::open_bytes(bytes).map_err(Error::from)
}
