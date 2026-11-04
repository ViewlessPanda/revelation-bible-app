//! Stages 4–6 on synthetic fixtures: fail closed with no output file (T15,
//! M1-F3), T10 and T12 in the database, T17 (read-only, sidecars), T21
//! (atomic publish with fault injection).

mod common;

use std::fs;
use std::path::Path;

use common::fixtures::{self, Q, doc, one, v, w};
use revelation_pipeline::db::{self, Meta, STEPS, Step};
use revelation_pipeline::error::{Error, Failure};
use revelation_pipeline::open::{open_bytes, open_verified};
use revelation_pipeline::osis;
use revelation_pipeline::profile::{self, AllowedChar, Profile};
use revelation_pipeline::run;
use revelation_pipeline::sources::{parse_checksum_file, sha256_hex};
use revelation_pipeline::validate;

const DEST: &str = "kjv.sqlite";

fn build(bytes: &[u8], p: &Profile, out: &Path) -> Result<String, Failure> {
    run::build_bytes(bytes, p, None, out, DEST)
}

fn good_doc(word: &str) -> Vec<u8> {
    doc(&format!(
        "{}\n{}",
        v(
            1,
            &format!(
                "{} {Q}{} <transChange type=\"added\">it</transChange>.</q>",
                w("Alpha"),
                w(word)
            )
        ),
        v(
            2,
            &format!("{} <w lemma=\"strong:H02\"/>{}", w("Gamma"), w("delta."))
        )
    ))
}

// ------------------------------------------------- fail closed, no output file

#[test]
fn every_failing_fixture_leaves_no_file() {
    let out = common::temp_dir("noout");
    let p = profile::fixture();
    let mut n = 0;
    let mut all: Vec<(String, Vec<u8>)> = fixtures::xml_failures()
        .into_iter()
        .map(|(name, b, _)| (name.to_string(), b))
        .collect();
    for dir in [
        "pipeline/tests/fixtures/m1-f3",
        "docs/pipeline/survey-scripts/fixtures",
    ] {
        for e in fs::read_dir(revelation_pipeline::repo_dir().join(dir)).unwrap() {
            let path = e.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with("fail-") {
                all.push((format!("{dir}/{name}"), fs::read(&path).unwrap()));
            }
        }
    }
    for (name, bytes) in &all {
        assert!(build(bytes, &p, &out).is_err(), "{name} built");
        assert!(
            common::list(&out).is_empty(),
            "{name} left {:?}",
            common::list(&out)
        );
        n += 1;
    }
    assert!(n > 60, "{n}");
    // The five M1-F3 cases by name.
    for f in [
        "fail-q-empty.xml",
        "fail-q-milestone.xml",
        "fail-transchange-empty.xml",
        "fail-divinename-empty.xml",
        "fail-verse-container.xml",
    ] {
        assert!(
            all.iter().any(|(n, _)| n.ends_with(&format!("m1-f3/{f}"))),
            "{f}"
        );
    }
}

#[test]
fn a_good_fixture_publishes_exactly_two_files() {
    let out = common::temp_dir("good");
    let hash = build(&good_doc("beta"), &profile::fixture(), &out).unwrap();
    assert_eq!(common::list(&out), vec!["kjv.sqlite", "kjv.sqlite.sha256"]);
    let bytes = fs::read(out.join(DEST)).unwrap();
    assert_eq!(sha256_hex(&bytes), hash);
    let (h, name) = parse_checksum_file(&fs::read(out.join("kjv.sqlite.sha256")).unwrap()).unwrap();
    assert_eq!((h.as_str(), name.as_str()), (hash.as_str(), DEST));
}

// ------------------------------------------------- T10, T12 in the database

#[test]
fn t10_no_markup_means_no_red_rows_and_flag_zero() {
    let out = common::temp_dir("t10");
    let bytes = one(&format!(
        "{} {}, {}",
        w("Jesus"),
        w("said"),
        w("Follow me.")
    ));
    build(&bytes, &profile::fixture(), &out).unwrap();
    let (c, _) = open_verified(&out.join(DEST), None).unwrap();
    let n: i64 = c
        .query_row("SELECT count(*) FROM red_piece", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
    let flag: String = c
        .query_row(
            "SELECT value FROM meta WHERE key = 'has_red_letters'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(flag, "0");
    let src: i64 = c
        .query_row(
            "SELECT count(*) FROM meta WHERE key = 'red_letter_source'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(src, 0);
}

#[test]
fn t12_substr_audit_query_uses_code_points() {
    let mut p = profile::fixture();
    p.chars.push(AllowedChar {
        c: '\u{1D400}',
        is_mark: false,
    });
    p.chars.push(AllowedChar {
        c: '\u{0301}',
        is_mark: true,
    });
    p.kjv_red_boundaries = false;
    let body = format!(
        "{} {} {Q}{} {}</q>",
        w("alpha"),
        w("\u{1D400}\u{301}"),
        w("\u{1D400}\u{301}"),
        w("beta")
    );
    let out = common::temp_dir("t12");
    build(&one(&body), &p, &out).unwrap();
    let (c, _) = open_verified(&out.join(DEST), None).unwrap();
    let (red, len, s, e): (String, i64, i64, i64) = c
        .query_row(
            "SELECT substr(v.text, r.start_cp + 1, r.end_cp - r.start_cp), length(v.text), r.start_cp, r.end_cp
               FROM red_piece r JOIN verse v ON v.id = r.verse_id",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(red, "\u{1D400}\u{301} beta");
    assert_eq!((len, s, e), (16, 9, 16));
}

// ------------------------------------------------- T17

#[test]
fn t17_open_verified_is_read_only_and_header_is_rollback() {
    let out = common::temp_dir("t17ro");
    let hash = build(&good_doc("beta"), &profile::fixture(), &out).unwrap();
    let bytes = fs::read(out.join(DEST)).unwrap();
    assert_eq!((bytes[18], bytes[19]), (1, 1));
    let (c, h) = open_verified(&out.join(DEST), Some(&hash)).unwrap();
    assert_eq!(h, hash);
    assert!(c.execute("UPDATE verse SET text = 'x'", []).is_err());
    assert!(
        c.execute_batch("PRAGMA query_only = 0; UPDATE verse SET text = 'x'")
            .is_err()
    );
    // A wrong expected hash is refused.
    let e = open_verified(&out.join(DEST), Some(&"0".repeat(64))).unwrap_err();
    assert_eq!(e.check, "sha256");
    // A WAL-mode header is refused.
    let mut wal = bytes.clone();
    wal[18] = 2;
    wal[19] = 2;
    assert_eq!(open_bytes(&wal).unwrap_err().check, "header");
    // No sidecars beside the published file.
    assert!(db::existing_sidecars(&out.join(DEST)).is_empty());
}

fn verse_text(c: &rusqlite::Connection) -> String {
    c.query_row("SELECT text FROM verse WHERE id = 1001001", [], |r| {
        r.get(0)
    })
    .unwrap()
}

#[test]
fn t17_planted_wal_cannot_change_what_open_verified_reads() {
    let out = common::temp_dir("t17wal");
    let hash = build(&good_doc("beta"), &profile::fixture(), &out).unwrap();
    let dest = out.join(DEST);
    let original = verse_text(&open_verified(&dest, None).unwrap().0);

    // A twin in WAL mode whose verse text differs, with its -wal kept.
    let twin_dir = common::temp_dir("t17twin");
    let twin = twin_dir.join("twin.sqlite");
    fs::copy(&dest, &twin).unwrap();
    {
        let c = rusqlite::Connection::open(&twin).unwrap();
        c.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
            true,
        )
        .unwrap();
        let m: String = c
            .query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))
            .unwrap();
        assert_eq!(m, "wal");
        c.execute_batch("PRAGMA wal_autocheckpoint = 0").unwrap();
        c.execute("UPDATE verse SET text = 'TAMPERED' WHERE id = 1001001", [])
            .unwrap();
    }
    let twin_wal = twin_dir.join("twin.sqlite-wal");
    assert!(
        fs::metadata(&twin_wal).unwrap().len() > 0,
        "twin WAL not kept"
    );
    fs::copy(&twin_wal, out.join("kjv.sqlite-wal")).unwrap();

    // The attack is real: a plain read-only open of the path shows the WAL text
    // (or at least does not reliably show the original).
    let plain =
        rusqlite::Connection::open_with_flags(&dest, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .ok()
            .and_then(|c| {
                c.query_row("SELECT text FROM verse WHERE id = 1001001", [], |r| {
                    r.get::<_, String>(0)
                })
                .ok()
            });
    eprintln!("plain read-only open with planted -wal shows: {plain:?}");

    // open_verified reads only the hashed bytes: original text, same hash.
    let (c, h) = open_verified(&dest, Some(&hash)).unwrap();
    assert_eq!(h, hash);
    assert_eq!(verse_text(&c), original);
    assert_ne!(verse_text(&c), "TAMPERED");

    // Publishing over a <dest> with a sidecar is refused, and the sidecar is
    // not deleted.
    let e = build(&good_doc("other"), &profile::fixture(), &out).unwrap_err();
    assert!(e.has("sidecar"), "{e}");
    assert!(out.join("kjv.sqlite-wal").exists());
    assert_eq!(sha256_hex(&fs::read(&dest).unwrap()), hash, "dest changed");
    // Nothing else left behind.
    // (The plain open above may have created a -shm; no temp file is left.)
    assert!(common::list(&out).iter().all(|n| !n.contains(".tmp-")));
    let _ = plain;
}

#[test]
fn t17_hot_journal_cannot_change_what_open_verified_reads() {
    let out = common::temp_dir("t17jrnl");
    let hash = build(&good_doc("beta"), &profile::fixture(), &out).unwrap();
    let dest = out.join(DEST);
    let original = verse_text(&open_verified(&dest, None).unwrap().0);
    let tampered = "X".repeat(original.chars().count());

    // Twin T: same file with the verse changed (same length, same layout).
    let twin_dir = common::temp_dir("t17jtwin");
    let twin = twin_dir.join("twin.sqlite");
    fs::copy(&dest, &twin).unwrap();
    let c = rusqlite::Connection::open(&twin).unwrap();
    c.execute("UPDATE verse SET text = ?1 WHERE id = 1001001", [&tampered])
        .unwrap();
    // Start a second change and copy the journal while it is hot: it holds
    // T's pages, i.e. the tampered verse.
    c.execute_batch("PRAGMA synchronous = OFF; PRAGMA cache_size = 1; BEGIN; UPDATE verse SET text = 'zz' WHERE id = 1001001;")
        .unwrap();
    let jr = twin_dir.join("twin.sqlite-journal");
    assert!(jr.exists(), "no journal");
    let hot = twin_dir.join("hot.journal");
    fs::copy(&jr, &hot).unwrap();
    c.execute_batch("ROLLBACK").unwrap();
    drop(c);

    // The attack is real: a plain open of a copy rolls the journal back into
    // the file and shows the twin's text.
    let demo = common::temp_dir("t17jdemo");
    fs::copy(&dest, demo.join(DEST)).unwrap();
    fs::copy(&hot, demo.join("kjv.sqlite-journal")).unwrap();
    let plain = rusqlite::Connection::open(demo.join(DEST))
        .ok()
        .and_then(|c| {
            c.query_row("SELECT text FROM verse WHERE id = 1001001", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
        });
    eprintln!("plain open with planted hot -journal shows: {plain:?} (original {original:?})");

    // Planted next to the published file: open_verified shows the original.
    fs::copy(&hot, out.join("kjv.sqlite-journal")).unwrap();
    let (c, h) = open_verified(&dest, Some(&hash)).unwrap();
    assert_eq!(h, hash);
    assert_eq!(verse_text(&c), original);
    assert_ne!(verse_text(&c), tampered);
    assert!(
        out.join("kjv.sqlite-journal").exists(),
        "open_verified never touches sidecars"
    );
    // And publish refuses while the -journal exists.
    let e = build(&good_doc("other"), &profile::fixture(), &out).unwrap_err();
    assert!(e.has("sidecar"), "{e}");
}

// ------------------------------------------------- T21

fn snapshot(out: &Path) -> (Vec<u8>, Vec<u8>) {
    (
        fs::read(out.join(DEST)).unwrap(),
        fs::read(out.join("kjv.sqlite.sha256")).unwrap(),
    )
}

fn fault_at(step: Step) -> impl FnMut(Step) -> Result<(), Failure> {
    move |s| {
        if s == step {
            Err(Error::new("injected", format!("fault after {s:?}")).into())
        } else {
            Ok(())
        }
    }
}

fn publish_fixture(
    bytes: &[u8],
    out: &Path,
    hook: &mut dyn FnMut(Step) -> Result<(), Failure>,
) -> Result<String, Failure> {
    let p = profile::fixture();
    let m = osis::parse(bytes, &p)?;
    validate::validate(&m, &p, None)?;
    let meta = Meta::kjv(&m, p.source_id, Vec::new(), None);
    run::publish_model(out, DEST, &m, &p, None, &meta, hook)
}

#[test]
fn t21_fault_after_every_step_keeps_the_previous_good_build() {
    let old_doc = good_doc("beta");
    let new_doc = good_doc("betb");
    for step in STEPS {
        let out = common::temp_dir("t21");
        let old_hash = publish_fixture(&old_doc, &out, &mut |_| Ok(())).unwrap();
        let before = snapshot(&out);
        let res = publish_fixture(&new_doc, &out, &mut fault_at(step));
        let err = res.expect_err("fault injected");
        assert!(err.has("injected"), "{step:?}: {err}");
        let files = common::list(&out);
        assert_eq!(
            files,
            vec!["kjv.sqlite", "kjv.sqlite.sha256"],
            "{step:?}: temps left"
        );
        let after = snapshot(&out);
        match step {
            Step::DbRenamed => {
                // Between the renames: the new DB is in place and passes check;
                // the stale .sha256 is reported as a mismatch.
                assert_ne!(after.0, before.0);
                assert_eq!(after.1, before.1);
                let (_, h) = open_verified(&out.join(DEST), None).unwrap();
                let (sidecar, _) = parse_checksum_file(&after.1).unwrap();
                assert_ne!(h, sidecar, "stale checksum must be detectable");
                assert_eq!(sidecar, old_hash);
                // The new file passes the stage-5 check.
                let p = profile::fixture();
                let m = osis::parse(&new_doc, &p).unwrap();
                let meta = Meta::kjv(&m, p.source_id, Vec::new(), None);
                let (c, _) = open_verified(&out.join(DEST), None).unwrap();
                revelation_pipeline::check::check(&c, &m, &p, None, &meta).unwrap();
            }
            Step::Publish => {
                // Everything was already in place: new DB and its checksum.
                let (h, _) = parse_checksum_file(&after.1).unwrap();
                assert_eq!(h, sha256_hex(&after.0));
                assert_ne!(after.0, before.0);
            }
            _ => assert_eq!(after, before, "{step:?}: previous build changed"),
        }
    }
}

#[test]
fn t21_leftover_temps_are_cleaned_and_names_are_exact() {
    let out = common::temp_dir("t21clean");
    for name in [
        "kjv.sqlite.tmp-0123456789abcdef",
        "kjv.sqlite.tmp-0123456789abcdef-journal",
        "kjv.sqlite.sha256.tmp-fedcba9876543210",
        "kjv.sqlite.tmp-0123456789abcdef-wal",
    ] {
        fs::write(out.join(name), b"junk").unwrap();
    }
    // Not leftovers: must survive.
    for keep in [
        "kjv.sqlite.tmp-XYZ",
        "notes.txt",
        "kjv.sqlite.tmp-0123456789abcdef.bak",
    ] {
        fs::write(out.join(keep), b"keep").unwrap();
    }
    publish_fixture(&good_doc("beta"), &out, &mut |_| Ok(())).unwrap();
    assert_eq!(
        common::list(&out),
        vec![
            "kjv.sqlite",
            "kjv.sqlite.sha256",
            "kjv.sqlite.tmp-0123456789abcdef.bak",
            "kjv.sqlite.tmp-XYZ",
            "notes.txt"
        ]
    );
    assert!(db::is_leftover(
        "kjv.sqlite",
        "kjv.sqlite.tmp-0123456789abcdef-shm"
    ));
    assert!(!db::is_leftover(
        "kjv.sqlite",
        "content.sqlite.tmp-0123456789abcdef"
    ));
}

#[cfg(windows)]
#[test]
fn t21_rename_fails_while_dest_is_held_open() {
    use std::os::windows::fs::OpenOptionsExt;
    let out = common::temp_dir("t21held");
    publish_fixture(&good_doc("beta"), &out, &mut |_| Ok(())).unwrap();
    let before = snapshot(&out);
    // Share mode 0: no other open, delete or rename while held.
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(out.join(DEST))
        .unwrap();
    let err = publish_fixture(&good_doc("betb"), &out, &mut |_| Ok(())).unwrap_err();
    assert!(err.has("publish"), "{err}");
    drop(held);
    assert_eq!(common::list(&out), vec!["kjv.sqlite", "kjv.sqlite.sha256"]);
    assert_eq!(snapshot(&out), before);
}

#[test]
fn t16_fixture_builds_are_byte_identical() {
    let a = common::temp_dir("t16a");
    let b = common::temp_dir("t16b");
    let ha = build(&good_doc("beta"), &profile::fixture(), &a).unwrap();
    let hb = build(&good_doc("beta"), &profile::fixture(), &b).unwrap();
    assert_eq!(ha, hb);
    assert_eq!(
        fs::read(a.join(DEST)).unwrap(),
        fs::read(b.join(DEST)).unwrap()
    );
    assert_eq!(
        fs::read(a.join("kjv.sqlite.sha256")).unwrap(),
        fs::read(b.join("kjv.sqlite.sha256")).unwrap()
    );
}

#[test]
fn t16_pipeline_commit_rules() {
    let h = "0123456789abcdef0123456789abcdef01234567";
    assert_eq!(db::commit_from(Some(h)), h);
    assert_eq!(
        db::commit_from(Some(&format!("{h}-dirty"))),
        format!("{h}-dirty")
    );
    for bad in [
        None,
        Some(""),
        Some("abc"),
        Some(&h.to_uppercase()[..]),
        Some(&format!("{h}-clean")[..]),
    ] {
        assert_eq!(db::commit_from(bad), "unknown-dirty", "{bad:?}");
    }
    // Release builds refuse every -dirty value, including unknown-dirty.
    assert_eq!(db::build_commit(Some(h), true).unwrap(), h);
    for bad in [None, Some("abc"), Some(&format!("{h}-dirty")[..])] {
        let e = db::build_commit(bad, true).unwrap_err();
        assert_eq!(e.check, "release", "{bad:?}");
        assert_eq!(db::build_commit(bad, false).unwrap(), db::commit_from(bad));
    }
}

/// T16 (PM decision 2026-10-22): no build-environment value reaches a content
/// database, so its hash is a pure function of the inputs and the code.
#[test]
fn t16_meta_holds_no_build_environment_value() {
    let out = common::temp_dir("pub-meta");
    build(
        &fixtures::one(&format!("{Q}alpha</q>")),
        &profile::fixture(),
        &out,
    )
    .unwrap();
    let (c, _) = revelation_pipeline::open::open_verified(&out.join(DEST), None).unwrap();
    let keys =
        revelation_pipeline::db::query_strings(&c, "SELECT key FROM meta ORDER BY key").unwrap();
    assert_eq!(
        keys,
        [
            "has_red_letters",
            "red_letter_source",
            "schema_version",
            "translation_abbrev",
            "versification"
        ]
    );
    drop(c);
    let _ = fs::remove_dir_all(&out);
}
