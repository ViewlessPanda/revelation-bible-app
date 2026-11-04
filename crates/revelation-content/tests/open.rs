//! open_verified (DESIGN.md 10.3, M1-F1): wrong hash, one-bit tamper,
//! planted `-wal` (with a control that proves the attack is real), oversize,
//! non-regular files, header and identity checks. Databases are made here
//! with SQL; no content text is involved.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use revelation_content::{
    APPLICATION_ID, MAX_DB, OpenError, SCHEMA_VERSION, check_plain, open_bytes, open_verified,
    read_capped, require_content_identity, sha256_hex,
};
use rusqlite::{Connection, OpenFlags};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A fresh, empty directory under `target/tmp` (reruns reuse the names).
fn temp_dir(label: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("rc-{label}-{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const ROW_TEXT: &str = "original row";

/// A small content-like database in rollback-journal (DELETE) mode.
fn make_db(path: &Path) -> String {
    let c = Connection::open(path).unwrap();
    c.execute_batch(&format!(
        "PRAGMA journal_mode = DELETE; PRAGMA application_id = {APPLICATION_ID};
         PRAGMA user_version = {SCHEMA_VERSION};
         CREATE TABLE t (id INTEGER PRIMARY KEY, text TEXT NOT NULL) STRICT;
         INSERT INTO t VALUES (1, '{ROW_TEXT}');"
    ))
    .unwrap();
    drop(c);
    sha256_hex(&std::fs::read(path).unwrap())
}

fn row(c: &Connection) -> String {
    c.query_row("SELECT text FROM t WHERE id = 1", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn good_file_opens_read_only_and_returns_its_hash() {
    let dir = temp_dir("good");
    let path = dir.join("a.sqlite");
    let hash = make_db(&path);
    let (c, h) = open_verified(&path, Some(&hash), MAX_DB).unwrap();
    assert_eq!(h, hash);
    assert_eq!(row(&c), ROW_TEXT);
    require_content_identity(&c).unwrap();
    // query_only: no write, even to the in-memory copy
    assert!(c.execute("DELETE FROM t", []).is_err());
    assert!(c.execute("CREATE TABLE x (a)", []).is_err());
    // None computes the hash without comparing (the pipeline's check stage)
    let (_, h2) = open_verified(&path, None, MAX_DB).unwrap();
    assert_eq!(h2, hash);
}

#[test]
fn wrong_expected_hash_is_refused() {
    let dir = temp_dir("wrong");
    let path = dir.join("a.sqlite");
    let hash = make_db(&path);
    let zero = "0".repeat(64);
    let mut one_hex_off = hash.clone();
    let last = if hash.ends_with('0') { "1" } else { "0" };
    one_hex_off.replace_range(63..64, last);
    for want in [
        zero.as_str(),
        one_hex_off.as_str(),
        "",
        &hash.to_uppercase(),
    ] {
        let e = open_verified(&path, Some(want), MAX_DB).unwrap_err();
        assert!(matches!(e, OpenError::HashMismatch { .. }), "{e}");
        assert_eq!(e.check(), "sha256");
        assert_eq!(e.file(), Some("a.sqlite"));
    }
}

#[test]
fn one_flipped_bit_is_refused() {
    let dir = temp_dir("tamper");
    let path = dir.join("a.sqlite");
    let hash = make_db(&path);
    let mut bytes = std::fs::read(&path).unwrap();
    for at in [0, 100, bytes.len() / 2, bytes.len() - 1] {
        bytes[at] ^= 0x01;
        std::fs::write(&path, &bytes).unwrap();
        let e = open_verified(&path, Some(&hash), MAX_DB).unwrap_err();
        assert!(
            matches!(e, OpenError::HashMismatch { .. }),
            "byte {at}: {e}"
        );
        bytes[at] ^= 0x01;
    }
    std::fs::write(&path, &bytes).unwrap();
    assert!(open_verified(&path, Some(&hash), MAX_DB).is_ok());
}

/// Review M1-F1: a `-wal` planted beside a DELETE-mode database changes
/// what a plain read-only open sees, while the file's hash still matches.
/// open_verified must ignore it.
#[test]
fn planted_wal_cannot_change_what_open_verified_reads() {
    let dir = temp_dir("wal");
    let victim = dir.join("victim.sqlite");
    let attacker = dir.join("attacker.sqlite");
    let hash = make_db(&victim);
    std::fs::copy(&victim, &attacker).unwrap();

    // The attacker writes a changed row into a WAL and keeps it un-checkpointed.
    let a = Connection::open(&attacker).unwrap();
    a.execute_batch("PRAGMA journal_mode = WAL; PRAGMA wal_autocheckpoint = 0;")
        .unwrap();
    a.execute("UPDATE t SET text = 'TAMPERED' WHERE id = 1", [])
        .unwrap();
    let wal = dir.join("attacker.sqlite-wal");
    assert!(wal.exists());
    std::fs::copy(&wal, dir.join("victim.sqlite-wal")).unwrap();
    // The victim's main file is unchanged, so its hash still matches.
    assert_eq!(sha256_hex(&std::fs::read(&victim).unwrap()), hash);

    // Control: with this SQLite, a plain read-only open of the path DOES apply
    // the planted WAL (reproduces M1-F1), so the assertion below is meaningful.
    let plain = Connection::open_with_flags(&victim, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(
        row(&plain),
        "TAMPERED",
        "control: the planted WAL had no effect on a plain open"
    );
    drop(plain);

    let (c, h) = open_verified(&victim, Some(&hash), MAX_DB).unwrap();
    assert_eq!(h, hash);
    assert_eq!(row(&c), ROW_TEXT);
    drop(a);
}

#[test]
fn oversize_is_refused_before_reading() {
    let dir = temp_dir("size");
    let path = dir.join("a.sqlite");
    let hash = make_db(&path);
    let len = std::fs::metadata(&path).unwrap().len();
    let e = open_verified(&path, Some(&hash), len - 1).unwrap_err();
    assert!(matches!(e, OpenError::TooLarge { .. }), "{e}");
    assert_eq!(e.check(), "size");
    assert!(open_verified(&path, Some(&hash), len).is_ok());
    assert_eq!(read_capped(&path, len).unwrap().len() as u64, len);
}

#[test]
fn missing_files_directories_and_links_are_refused() {
    let dir = temp_dir("plain");
    let path = dir.join("a.sqlite");
    let hash = make_db(&path);

    let e = open_verified(&dir.join("no-such.sqlite"), Some(&hash), MAX_DB).unwrap_err();
    assert!(matches!(e, OpenError::Missing { .. }), "{e}");
    assert_eq!(e.check(), "path");

    let sub = dir.join("sub.sqlite");
    std::fs::create_dir(&sub).unwrap();
    let e = open_verified(&sub, Some(&hash), MAX_DB).unwrap_err();
    assert!(
        matches!(e, OpenError::NotPlain { why, .. } if why.contains("not a regular file")),
        "{e}"
    );
    assert!(check_plain(&sub, true).is_ok());
    assert!(check_plain(&path, true).is_err());

    // A link to the good file must be refused even though its bytes match.
    let link = dir.join("link.sqlite");
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(&path, &link);
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(&path, &link);
    match made {
        Ok(()) => {
            let e = open_verified(&link, Some(&hash), MAX_DB).unwrap_err();
            assert!(matches!(e, OpenError::NotPlain { .. }), "{e}");
        }
        // Creating file symlinks needs Developer Mode or admin rights on Windows.
        Err(err) => eprintln!("NOTICE: cannot create a file symlink here ({err}); skipped"),
    }

    // A directory junction (Windows; no privilege needed) or symlink is a
    // reparse point: refused as a directory and as a file.
    let target = dir.join("target-dir");
    std::fs::create_dir(&target).unwrap();
    let junction = dir.join("junction");
    #[cfg(windows)]
    let made = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(&junction)
        .arg(&target)
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success());
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(&target, &junction).map(|()| true);
    match made {
        Ok(true) => {
            let e = check_plain(&junction, true).unwrap_err();
            assert!(matches!(e, OpenError::NotPlain { .. }), "{e}");
            let e = open_verified(&junction, Some(&hash), MAX_DB).unwrap_err();
            assert!(matches!(e, OpenError::NotPlain { .. }), "{e}");
            // remove the link itself, never the target's contents
            #[cfg(windows)]
            std::fs::remove_dir(&junction).unwrap();
            #[cfg(unix)]
            std::fs::remove_file(&junction).unwrap();
        }
        other => eprintln!("NOTICE: cannot create a directory link here ({other:?}); skipped"),
    }
}

#[test]
fn headers_other_than_rollback_journal_sqlite_are_refused() {
    let dir = temp_dir("header");
    let path = dir.join("a.sqlite");
    make_db(&path);
    let good = std::fs::read(&path).unwrap();
    assert!(open_bytes(&good).is_ok());

    let mut wal = good.clone();
    wal[18] = 2;
    wal[19] = 2;
    assert_eq!(open_bytes(&wal).unwrap_err().check(), "header");
    assert_eq!(open_bytes(b"not a database").unwrap_err().check(), "header");
    assert_eq!(open_bytes(&good[..99]).unwrap_err().check(), "header");

    // A file in WAL mode on disk is refused by open_verified too.
    let w = dir.join("w.sqlite");
    let c = Connection::open(&w).unwrap();
    c.execute_batch("PRAGMA journal_mode = WAL; CREATE TABLE t (a);")
        .unwrap();
    drop(c);
    let e = open_verified(&w, None, MAX_DB).unwrap_err();
    assert!(matches!(e, OpenError::Header(_)), "{e}");
}

#[test]
fn identity_requires_rvln_and_the_schema_version() {
    let dir = temp_dir("identity");
    let path = dir.join("plain.sqlite");
    let c = Connection::open(&path).unwrap();
    c.execute_batch("PRAGMA journal_mode = DELETE; CREATE TABLE t (a);")
        .unwrap();
    drop(c);
    let (c, _) = open_verified(&path, None, MAX_DB).unwrap();
    let e = require_content_identity(&c).unwrap_err();
    assert!(matches!(e, OpenError::NotContent(_)), "{e}");

    let path2 = dir.join("v2.sqlite");
    let c = Connection::open(&path2).unwrap();
    c.execute_batch(&format!(
        "PRAGMA journal_mode = DELETE; PRAGMA application_id = {APPLICATION_ID};
         PRAGMA user_version = 2; CREATE TABLE t (a);"
    ))
    .unwrap();
    drop(c);
    let (c, _) = open_verified(&path2, None, MAX_DB).unwrap();
    assert!(require_content_identity(&c).is_err());
}

#[test]
fn messages_name_the_file_only() {
    let e = OpenError::HashMismatch {
        file: "kjv.sqlite".into(),
        actual: "a".repeat(64),
        expected: "b".repeat(64),
    };
    let s = e.to_string();
    assert!(s.starts_with("kjv.sqlite: failed its checksum"), "{s}");
    let dir = temp_dir("msg");
    let e = read_capped(&dir.join("missing.sqlite"), 10).unwrap_err();
    let s = e.to_string();
    assert!(s.starts_with("missing.sqlite: "), "{s}");
    assert!(!s.contains(env!("CARGO_TARGET_TMPDIR")), "{s}");
}
