//! Helpers shared by the integration tests.
#![allow(dead_code)]

pub mod fixtures;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// `REVELATION_SOURCES` (the `sources/raw` directory). When it is unset the
/// real-source tests **fail** if `CI` is set and are skipped with a printed
/// notice otherwise (DESIGN.md 14), so CI can never look green by skipping.
pub fn sources_dir() -> Option<PathBuf> {
    match decide(
        std::env::var_os("REVELATION_SOURCES"),
        std::env::var_os("CI"),
    ) {
        Ok(Some(dir)) => Some(dir),
        Ok(None) => {
            eprintln!("NOTICE: REVELATION_SOURCES unset; skipping a real-source test");
            None
        }
        Err(msg) => panic!("{msg}"),
    }
}

/// The rule behind [`sources_dir`], as a pure function so T1 can test it
/// without touching the process environment.
pub fn decide(
    revelation_sources: Option<std::ffi::OsString>,
    ci: Option<std::ffi::OsString>,
) -> Result<Option<PathBuf>, String> {
    match revelation_sources {
        Some(dir) if !dir.is_empty() => Ok(Some(PathBuf::from(dir))),
        _ if ci.is_some() => {
            Err("REVELATION_SOURCES is unset and CI is set: real-source tests must run".into())
        }
        _ => Ok(None),
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A fresh, empty directory under Cargo's per-target temp dir.
pub fn temp_dir(label: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    // No process id in the name: reruns reuse (after emptying) the same
    // directories instead of piling up copies of the 24 MB database.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{label}-{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Names of the entries in `dir`, sorted.
pub fn list(dir: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}
