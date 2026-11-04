//! T1: source verification and paths (DESIGN.md 2 stage 1, 9.5; findings 20–22).

mod common;

use std::fs;
use std::path::Path;

use revelation_pipeline::sources::{
    self, CAP_KJV, CAP_VRS, check_file_name, check_id, check_sources_md, parse_checksum_file,
    sha256_hex,
};

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A tiny fake source tree: checksums/<ID>.sha256 and raw/<ID>/<file>.
fn fake(label: &str, id: &str, file: &str, data: &[u8], sum_line: &[u8]) -> std::path::PathBuf {
    let root = common::temp_dir(label);
    write(
        &root.join("checksums").join(format!("{id}.sha256")),
        sum_line,
    );
    write(&root.join("raw").join(id).join(file), data);
    root
}

fn line(hex: &str, name: &str) -> Vec<u8> {
    format!("{hex}  {name}\n").into_bytes()
}

#[test]
fn good_file_verifies_and_returns_the_hashed_bytes() {
    let data = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><osis/>\n";
    let root = fake(
        "t1-good",
        "TEST-SRC",
        "a.xml",
        data,
        &line(&sha256_hex(data), "a.xml"),
    );
    let v = sources::verify(&root.join("checksums"), &root.join("raw"), "TEST-SRC", 1024).unwrap();
    assert_eq!(v.bytes, data);
    assert_eq!(v.sha256, sha256_hex(data));
    assert_eq!(v.file, "a.xml");
}

#[test]
fn one_byte_changed_and_truncated_files_are_refused() {
    let data = b"0123456789abcdef".to_vec();
    let hex = sha256_hex(&data);
    let mut changed = data.clone();
    changed[3] ^= 1;
    let truncated = &data[..data.len() - 1];
    for (label, bytes) in [("changed", &changed[..]), ("truncated", truncated)] {
        let root = fake(
            &format!("t1-{label}"),
            "TEST-SRC",
            "a.xml",
            bytes,
            &line(&hex, "a.xml"),
        );
        let e = sources::verify(&root.join("checksums"), &root.join("raw"), "TEST-SRC", 1024)
            .unwrap_err();
        assert_eq!(e.check, "sha256", "{label}: {e}");
    }
}

#[test]
fn malformed_checksum_files_are_refused() {
    let hex = sha256_hex(b"x");
    let upper = hex.to_ascii_uppercase();
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("crlf", format!("{hex}  a.xml\r\n").into_bytes()),
        ("uppercase", format!("{upper}  a.xml\n").into_bytes()),
        ("one space", format!("{hex} a.xml\n").into_bytes()),
        ("no final LF", format!("{hex}  a.xml").into_bytes()),
        (
            "two lines",
            format!("{hex}  a.xml\n{hex}  a.xml\n").into_bytes(),
        ),
        ("empty", Vec::new()),
        (
            "short digest",
            format!("{}  a.xml\n", &hex[..63]).into_bytes(),
        ),
        ("star (binary mode)", format!("{hex} *a.xml\n").into_bytes()),
    ];
    for (label, bytes) in cases {
        assert!(parse_checksum_file(&bytes).is_err(), "{label} accepted");
    }
    // Another file name: parses, but verify() looks for that name only.
    let (h, name) = parse_checksum_file(&line(&hex, "b.xml")).unwrap();
    assert_eq!((h.as_str(), name.as_str()), (hex.as_str(), "b.xml"));
    let root = fake(
        "t1-othername",
        "TEST-SRC",
        "a.xml",
        b"x",
        &line(&hex, "b.xml"),
    );
    let e =
        sources::verify(&root.join("checksums"), &root.join("raw"), "TEST-SRC", 1024).unwrap_err();
    assert_eq!(e.check, "path", "{e}");
}

#[test]
fn bad_ids_and_names_are_refused() {
    for id in [
        "NUL", "CON", "con", "..", "A/B", "A\\B", "", "a-lower", "-X", "COM1", "LPT9",
    ] {
        assert!(check_id(id).is_err(), "id {id:?} accepted");
    }
    for name in [
        "kjv.osis.xml.",
        "kjv.osis.xml:ads",
        "CON.xml",
        "nul.txt",
        "..",
        "a..b",
        "a/b",
        "a\\b",
        ".hidden",
        "",
        "COM3.sha256",
    ] {
        assert!(check_file_name(name).is_err(), "name {name:?} accepted");
    }
    assert!(check_id("BIB-KJV-CW31").is_ok());
    assert!(check_file_name("kjv.osis.xml").is_ok());
    // A checksum file naming a device or an ADS is refused at parse time.
    let hex = sha256_hex(b"x");
    for name in ["CON.xml", "kjv.osis.xml:ads", "kjv.osis.xml."] {
        assert!(parse_checksum_file(&line(&hex, name)).is_err(), "{name}");
    }
}

#[test]
fn size_cap_is_checked_before_reading() {
    let data = vec![b'a'; 2048];
    let root = fake(
        "t1-cap",
        "TEST-SRC",
        "a.xml",
        &data,
        &line(&sha256_hex(&data), "a.xml"),
    );
    let e =
        sources::verify(&root.join("checksums"), &root.join("raw"), "TEST-SRC", 1024).unwrap_err();
    assert_eq!(e.check, "size", "{e}");
}

#[test]
fn symlinks_in_the_raw_tree_are_refused() {
    let data = b"payload";
    let root = fake(
        "t1-link",
        "TEST-SRC",
        "real.xml",
        data,
        &line(&sha256_hex(data), "a.xml"),
    );
    let target = root.join("raw").join("TEST-SRC").join("real.xml");
    let link = root.join("raw").join("TEST-SRC").join("a.xml");
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(&target, &link);
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(&target, &link);
    match made {
        Ok(()) => {
            let e = sources::verify(&root.join("checksums"), &root.join("raw"), "TEST-SRC", 1024)
                .unwrap_err();
            assert_eq!(e.check, "path", "{e}");
        }
        Err(err) => {
            // Creating symlinks needs Developer Mode or admin rights on Windows.
            // The reparse-point check itself is exercised by the directory case
            // below when that can be created; print why this half was skipped.
            eprintln!("NOTICE: cannot create a file symlink here ({err}); file half skipped");
        }
    }
    // Directory link in place of raw/<ID>.
    let root2 = fake(
        "t1-dirlink",
        "REAL-ID",
        "a.xml",
        data,
        &line(&sha256_hex(data), "a.xml"),
    );
    let sums = root2.join("checksums");
    fs::copy(sums.join("REAL-ID.sha256"), sums.join("LINK-ID.sha256")).unwrap();
    // Windows: a symlink needs a privilege; a junction doesn't, so fall back
    // to `mklink /J` (test-only; the pipeline itself never runs anything).
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_dir(
        root2.join("raw").join("REAL-ID"),
        root2.join("raw").join("LINK-ID"),
    )
    .or_else(|_| {
        let st = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(root2.join("raw").join("LINK-ID"))
            .arg(root2.join("raw").join("REAL-ID"))
            .stdout(std::process::Stdio::null())
            .status()?;
        if st.success() {
            Ok(())
        } else {
            Err(std::io::Error::other("mklink /J failed"))
        }
    });
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(
        root2.join("raw").join("REAL-ID"),
        root2.join("raw").join("LINK-ID"),
    );
    match made {
        Ok(()) => {
            let e = sources::verify(&sums, &root2.join("raw"), "LINK-ID", 1024).unwrap_err();
            assert_eq!(e.check, "path", "{e}");
        }
        Err(err) => eprintln!("NOTICE: cannot create a directory symlink here ({err}); skipped"),
    }
}

#[test]
fn checksum_must_appear_in_its_sources_md_row() {
    let md = "## Sources\n\n### AAA-X: one\n- **SHA-256:** `aaaa` (1 byte)\n\n### BBB-Y: two\n- **SHA-256:** `bbbb`\n";
    assert!(check_sources_md(md, "AAA-X", "aaaa").is_ok());
    assert!(
        check_sources_md(md, "AAA-X", "bbbb").is_err(),
        "value from another row"
    );
    assert!(
        check_sources_md(md, "CCC-Z", "aaaa").is_err(),
        "missing row"
    );
}

#[test]
fn real_checksums_appear_in_sources_md() {
    let md = fs::read_to_string(revelation_pipeline::sources_md_path()).unwrap();
    for id in [
        "BIB-KJV-CW31",
        "VRS-ENG-PALASO",
        "LEX-HEB-OSHB",
        "LEX-GRC-MGNT19",
    ] {
        let bytes =
            fs::read(revelation_pipeline::checksums_dir().join(format!("{id}.sha256"))).unwrap();
        let (hex, _) = parse_checksum_file(&bytes).unwrap();
        check_sources_md(&md, id, &hex).unwrap();
    }
}

#[test]
fn ci_without_sources_fails_instead_of_skipping() {
    assert!(common::decide(None, Some("true".into())).is_err());
    assert!(common::decide(Some("".into()), Some("1".into())).is_err());
    assert_eq!(common::decide(None, None), Ok(None));
    assert_eq!(
        common::decide(Some("x".into()), Some("1".into())),
        Ok(Some("x".into()))
    );
}

#[test]
fn real_sources_verify() {
    let Some(raw) = common::sources_dir() else {
        return;
    };
    let sums = revelation_pipeline::checksums_dir();
    let kjv = sources::verify(&sums, &raw, "BIB-KJV-CW31", CAP_KJV).unwrap();
    assert_eq!(kjv.bytes.len(), 28_043_075);
    assert_eq!(
        kjv.sha256,
        "4bcbfe45722fca396adbeac9c81025bbb173cb1e30296da90864f0400cb0e3f5"
    );
    let vrs = sources::verify(&sums, &raw, "VRS-ENG-PALASO", CAP_VRS).unwrap();
    assert_eq!(vrs.bytes.len(), 18_787);
}
