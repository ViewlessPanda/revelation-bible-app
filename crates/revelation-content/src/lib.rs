//! The security-critical opener for Revelation's read-only content databases,
//! in one place (pipeline/DESIGN.md 10.3 rule 1, review item M1-F1):
//! the pipeline's check stage and the app core both call [`open_verified`];
//! neither has its own copy.
//!
//! - [`hashfile`]: the parser for `pipeline/tests/expected/output-sha256.txt`
//!   (std only; the app's `build.rs` uses it with default features off).
//! - [`open_verified`] (feature `sqlite`, on by default): read the file once
//!   (regular file, no link or reparse point, size cap), SHA-256 it, compare
//!   with the expected value, and open **that same buffer** read-only with
//!   `sqlite3_deserialize`. An in-memory database has no `-wal`, `-shm` or
//!   `-journal`, so nothing beside the file can change what is read, and there
//!   is no gap between the hash and the use.
//! - [`search`]: the search-only normalisation (std only) and the phrase and
//!   word query over `kjv-search.sqlite` (feature `sqlite`), shared so the
//!   pipeline indexes and the app looks up through one function
//!   (pipeline/DESIGN.md 18).

pub mod hashfile;
pub mod search;

#[cfg(feature = "sqlite")]
mod open;
#[cfg(feature = "sqlite")]
pub use open::*;
