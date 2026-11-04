//! Revelation content pipeline: library half (`pipeline/DESIGN.md`).
//!
//! Stages: verify (`sources`) → parse (`xmlscan`, `osis`, `vrs`) → validate
//! (`validate`, with the per-source facts in `profile`) → write, check and
//! publish (`db`, `check`, `open`). The search index (`search`) is built
//! from the published kjv.sqlite.

pub mod check;
pub mod commentary;
pub mod db;
pub mod error;
pub mod golden;
pub mod labels;
pub mod lexicon;
pub mod open;
pub mod oracle;
pub mod osis;
pub mod profile;
pub mod run;
pub mod search;
pub mod sources;
pub mod strongs;
pub mod validate;
pub mod vrs;
pub mod xmlscan;

use std::path::PathBuf;

/// The repository root (the parent of `pipeline/`), fixed at compile time.
/// Checksum files, SOURCES.md and the overrides file are read from here,
/// never from a path inside a source.
pub fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("pipeline/ has a parent")
        .to_path_buf()
}

/// `sources/checksums` in the repo.
pub fn checksums_dir() -> PathBuf {
    repo_dir().join("sources").join("checksums")
}

/// `pipeline/data/kjv-versification-overrides.txt` in the repo.
pub fn overrides_path() -> PathBuf {
    repo_dir()
        .join("pipeline")
        .join("data")
        .join("kjv-versification-overrides.txt")
}

/// `sources/SOURCES.md` in the repo.
pub fn sources_md_path() -> PathBuf {
    repo_dir().join("sources").join("SOURCES.md")
}
