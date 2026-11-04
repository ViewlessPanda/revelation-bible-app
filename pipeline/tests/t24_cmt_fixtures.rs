//! T24: the commentary profile fails closed (DESIGN.md 17.6). The synthetic
//! `fixtures/mhcc/good.xml` builds a checked mhcc.sqlite; every line of
//! `fixtures/mhcc/mutations.tsv` turns it into a bad file that must be refused
//! by the named check, with nothing published. No pinned source is read.

mod common;

use std::fs;
use std::path::PathBuf;

use revelation_pipeline::commentary::MHCC_DEST;
use revelation_pipeline::open::open_verified;
use revelation_pipeline::run;
use revelation_pipeline::vrs::Table;

fn fixture(name: &str) -> PathBuf {
    revelation_pipeline::repo_dir()
        .join("pipeline/tests/fixtures/mhcc")
        .join(name)
}

/// KJV verse counts of the chapters the fixture uses (Gen 1-2, John 1).
fn table() -> Table {
    Table {
        books: vec![("GEN".into(), vec![31, 25]), ("JHN".into(), vec![51])],
    }
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('u') => {
                assert_eq!(it.next(), Some('{'), "bad \\u escape in {s:?}");
                let hex: String = it.by_ref().take_while(|&c| c != '}').collect();
                out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap());
            }
            other => panic!("bad escape \\{other:?} in {s:?}"),
        }
    }
    out
}

struct Mutation {
    id: String,
    check: String,
    who: String,
    old: String,
    new: String,
}

fn mutations() -> Vec<Mutation> {
    let text = fs::read_to_string(fixture("mutations.tsv")).unwrap();
    let mut v = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        // A missing <new> column means "delete <old>".
        assert!(f.len() == 4 || f.len() == 5, "bad line {line:?}");
        v.push(Mutation {
            id: f[0].into(),
            check: f[1].into(),
            who: f[2].into(),
            old: unescape(f[3]),
            new: f.get(4).map(|s| unescape(s)).unwrap_or_default(),
        });
    }
    v
}

#[test]
fn good_fixture_builds_and_reads_back() {
    let good = fs::read(fixture("good.xml")).unwrap();
    let out = common::temp_dir("t24-good");
    let hash = run::build_cmt_bytes(&good, &table(), &out).unwrap_or_else(|e| panic!("{e}"));
    let (c, _) = open_verified(&out.join(MHCC_DEST), Some(&hash)).unwrap();
    let one = |q: &str| -> i64 { c.query_row(q, [], |r| r.get(0)).unwrap() };
    assert_eq!(one("SELECT count(*) FROM cmt_anchor"), 3);
    assert_eq!(one("SELECT count(*) FROM cmt_anchor_verse"), 4);
    assert_eq!(
        one("SELECT count(*) FROM cmt_attr WHERE ref_start IS NOT NULL"),
        2
    );
    // '&amp;' is stored decoded, and only there.
    assert_eq!(
        one("SELECT count(*) FROM cmt_node WHERE text LIKE '%alpha &c. beta%'"),
        1
    );
    drop(c);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn every_mutation_is_refused_by_the_named_check_and_nothing_is_published() {
    let good = String::from_utf8(fs::read(fixture("good.xml")).unwrap()).unwrap();
    let list = mutations();
    assert_eq!(
        list.len(),
        65,
        "mutation count (a removed line must be deliberate)"
    );
    let mut ids = std::collections::BTreeSet::new();
    let out = common::temp_dir("t24-bad");
    for m in &list {
        assert!(ids.insert(m.id.clone()), "duplicate id {}", m.id);
        assert!(
            m.who == "both" || m.who == "rust",
            "{}: who {}",
            m.id,
            m.who
        );
        assert_eq!(
            good.matches(m.old.as_str()).count(),
            1,
            "{}: <old> must occur exactly once in good.xml",
            m.id
        );
        let bad = good.replacen(m.old.as_str(), &m.new, 1);
        let f = run::build_cmt_bytes(bad.as_bytes(), &table(), &out)
            .expect_err(&format!("{} was accepted", m.id));
        assert!(
            f.has(&m.check),
            "{}: expected check {}, got {:?}: {f}",
            m.id,
            m.check,
            f.checks()
        );
        assert!(
            common::list(&out).is_empty(),
            "{}: something was published",
            m.id
        );
    }
    let _ = fs::remove_dir_all(&out);
}
