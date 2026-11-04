//! Stage 1, verify (DESIGN.md 2, 9.1, 9.5).
//!
//! A source is `<raw-dir>/<ID>/<file>`, where `<ID>` and `<file>` come from
//! the repo's own checksum file `sources/checksums/<ID>.sha256` (never from
//! inside a source). The file is read into memory **once**, hashed, and only
//! that buffer is handed on, so the bytes parsed are the bytes hashed.
//!
//! Order: name rules → checksum file → reparse-point / file-type checks →
//! size cap from metadata → read at most cap + 1 bytes → SHA-256 → compare.

use std::path::{Path, PathBuf};

use crate::error::{Error, excerpt};

/// A source whose bytes matched its pinned SHA-256.
#[derive(Debug, Clone)]
pub struct Verified {
    pub id: String,
    pub file: String,
    pub sha256: String,
    pub bytes: Vec<u8>,
}

/// Size caps (DESIGN.md 9.1).
pub const CAP_KJV: u64 = 64 * 1024 * 1024;
pub const CAP_LEXICON: u64 = 16 * 1024 * 1024;
pub const CAP_COMMENTARY: u64 = 16 * 1024 * 1024;
pub const CAP_VRS: u64 = 1024 * 1024;
pub const CAP_OVERRIDES: u64 = 64 * 1024;
/// A checksum file is one short line.
const CAP_CHECKSUM_FILE: u64 = 4096;

/// Lowercase hex SHA-256 of `bytes` (the shared implementation).
pub use revelation_content::sha256_hex;

fn is_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn common_name_rules(name: &str, what: &str) -> Result<(), Error> {
    if name.contains("..") {
        return Err(Error::new("path", format!("{what} contains '..'")));
    }
    if name.ends_with('.') {
        return Err(Error::new("path", format!("{what} ends with '.'")));
    }
    if is_device_name(name) {
        return Err(Error::new(
            "path",
            format!("{what} is a Windows device name"),
        ));
    }
    Ok(())
}

/// `^[A-Z0-9][A-Z0-9-]{0,40}$`, plus the M1-F11 rules.
pub fn check_id(id: &str) -> Result<(), Error> {
    let b = id.as_bytes();
    let ok = !b.is_empty()
        && b.len() <= 41
        && (b[0].is_ascii_uppercase() || b[0].is_ascii_digit())
        && b[1..]
            .iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == b'-');
    if !ok {
        return Err(Error::new(
            "path",
            format!("source ID '{}' is not allowed", excerpt(b)),
        ));
    }
    common_name_rules(id, "source ID")
}

/// `^[A-Za-z0-9][A-Za-z0-9._-]{0,80}$`, plus the M1-F11 rules.
pub fn check_file_name(name: &str) -> Result<(), Error> {
    let b = name.as_bytes();
    let ok = !b.is_empty()
        && b.len() <= 81
        && b[0].is_ascii_alphanumeric()
        && b[1..]
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'));
    if !ok {
        return Err(Error::new(
            "path",
            format!("file name '{}' is not allowed", excerpt(b)),
        ));
    }
    common_name_rules(name, "file name")
}

/// Parses a checksum file: exactly 64 lowercase hex digits, two spaces, a
/// file name, one LF, nothing else. Returns `(hex, file name)`.
pub fn parse_checksum_file(bytes: &[u8]) -> Result<(String, String), Error> {
    let bad = |why: &str| Error::new("checksum-file", why.to_string());
    let line = bytes
        .strip_suffix(b"\n")
        .ok_or_else(|| bad("must end with exactly one LF"))?;
    if line.contains(&b'\n') || line.contains(&b'\r') {
        return Err(bad("must be exactly one line, LF only"));
    }
    if line.len() < 67 || &line[64..66] != b"  " {
        return Err(bad("must be '<64 hex>  <file name>'"));
    }
    let hex = &line[..64];
    if !hex.iter().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(bad("digest must be 64 lowercase hex digits"));
    }
    let name = std::str::from_utf8(&line[66..]).map_err(|_| bad("file name is not ASCII"))?;
    check_file_name(name)?;
    Ok((
        String::from_utf8(hex.to_vec()).expect("hex is ASCII"),
        name.to_string(),
    ))
}

/// Refuses symlinks, junctions and other reparse points, and anything that is
/// not a plain directory (`want_dir`) or a regular file. One implementation,
/// shared with the app: `revelation_content::check_plain`.
pub fn check_plain(path: &Path, want_dir: bool) -> Result<(), Error> {
    revelation_content::check_plain(path, want_dir).map_err(Error::from)
}

/// Reads a regular file of at most `cap` bytes: checks the metadata size
/// first, then reads at most `cap + 1` bytes, so a file that grows after the
/// size check is still refused without a full read. One implementation,
/// shared with the app: `revelation_content::read_capped`.
pub fn read_capped(path: &Path, cap: u64) -> Result<Vec<u8>, Error> {
    revelation_content::read_capped(path, cap).map_err(Error::from)
}

/// Verifies `<raw_dir>/<id>/<file>` against `<checksums_dir>/<id>.sha256`.
pub fn verify(checksums_dir: &Path, raw_dir: &Path, id: &str, cap: u64) -> Result<Verified, Error> {
    check_id(id)?;
    let sum_path = checksums_dir.join(format!("{id}.sha256"));
    let sum =
        read_capped(&sum_path, CAP_CHECKSUM_FILE).map_err(|e| e.file(format!("{id}.sha256")))?;
    let (expected, file) = parse_checksum_file(&sum).map_err(|e| e.file(format!("{id}.sha256")))?;
    let dir = raw_dir.join(id);
    check_plain(&dir, true)?;
    let path: PathBuf = dir.join(&file);
    let bytes = read_capped(&path, cap)?;
    let got = sha256_hex(&bytes);
    if got != expected {
        return Err(Error::new(
            "sha256",
            format!("SHA-256 mismatch: got {got}, pinned {expected}"),
        )
        .file(file));
    }
    Ok(Verified {
        id: id.to_string(),
        file,
        sha256: got,
        bytes,
    })
}

/// Checks that `hex` appears in the `### <ID>:` section of SOURCES.md, so the
/// checksum file and the SOURCES.md row can't drift (T1).
pub fn check_sources_md(sources_md: &str, id: &str, hex: &str) -> Result<(), Error> {
    let head = format!("### {id}:");
    let start = sources_md
        .lines()
        .position(|l| l.starts_with(&head))
        .ok_or_else(|| Error::new("sources-md", format!("no '{head}' row in SOURCES.md")))?;
    let section: Vec<&str> = sources_md
        .lines()
        .skip(start + 1)
        .take_while(|l| !l.starts_with("### ") && !l.starts_with("## "))
        .collect();
    let found = section
        .iter()
        .any(|l| l.starts_with("- **SHA-256:**") && l.contains(&format!("`{hex}`")));
    if !found {
        return Err(Error::new(
            "sources-md",
            format!("SHA-256 {hex} is not in the SOURCES.md row {id}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_names() {
        for n in [
            "CON", "con", "NUL.xml", "COM1", "lpt9.txt", "AUX", "PRN.a.b",
        ] {
            assert!(is_device_name(n), "{n}");
        }
        for n in ["COM0", "COM10", "CONSOLE", "kjv.osis.xml", "NULL"] {
            assert!(!is_device_name(n), "{n}");
        }
    }
}
