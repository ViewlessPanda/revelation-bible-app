//! Lexicons on the real pinned files (REVELATION_SOURCES): parse with every
//! pinned fact, T14 (round trip of every entry, byte-exact against the
//! source), T5 (the independent lexicon oracle), T13 (every KJV key
//! resolves), T16 (determinism, committed expected hash), T17 (check refuses
//! sidecars), and mutations asserting which check fires.

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use revelation_pipeline::check;
use revelation_pipeline::db::LEX_DEST;
use revelation_pipeline::lexicon::{self, LexModel, NodeKind};
use revelation_pipeline::open::{open_bytes, open_verified};
use revelation_pipeline::run::{self, PreparedLex};
use revelation_pipeline::sources::sha256_hex;
use rusqlite::Connection;

struct Real {
    lex: PreparedLex,
    out: PathBuf,
    hash: String,
    bytes: Vec<u8>,
}

fn real() -> Option<&'static Real> {
    static R: OnceLock<Option<Real>> = OnceLock::new();
    R.get_or_init(|| {
        let raw = common::sources_dir()?;
        let lex = run::prepare_lexicons(&raw).unwrap_or_else(|e| panic!("{e}"));
        let out = common::temp_dir("lex-real");
        let hash =
            run::publish_lexicons(&out, &lex, &mut |_| Ok(())).unwrap_or_else(|e| panic!("{e}"));
        let bytes = fs::read(out.join(LEX_DEST)).unwrap();
        Some(Real {
            lex,
            out,
            hash,
            bytes,
        })
    })
    .as_ref()
}

fn db(r: &Real) -> Connection {
    open_verified(&r.out.join(LEX_DEST), Some(&r.hash))
        .unwrap()
        .0
}

fn one<T: rusqlite::types::FromSql>(c: &Connection, q: &str) -> T {
    c.query_row(q, [], |r| r.get(0)).unwrap()
}

fn expected(name: &str) -> String {
    fs::read_to_string(
        revelation_pipeline::repo_dir()
            .join("pipeline/tests/expected")
            .join(name),
    )
    .unwrap()
}

/// An in-memory copy of the database with `sql` applied (constraints and
/// foreign keys off, so the mutation lands), serialised back to bytes.
fn mutate(bytes: &[u8], sql: &str) -> Vec<u8> {
    let mut c = Connection::open_in_memory().unwrap();
    c.deserialize_read_exact("main", bytes, bytes.len(), false)
        .unwrap();
    c.execute_batch(&format!(
        "PRAGMA foreign_keys = OFF; PRAGMA ignore_check_constraints = ON; {sql}"
    ))
    .unwrap();
    c.serialize("main").unwrap().to_vec()
}

fn models(r: &Real) -> [&LexModel; 2] {
    r.lex.models()
}

// ------------------------------------------------------------ positive

#[test]
fn built_content_db_passes_every_check_and_reads_back_the_counts() {
    let Some(r) = real() else { return };
    let c = db(r);
    check::check_lex(&c, &models(r), &r.lex.meta()).unwrap_or_else(|e| panic!("{e}"));
    let n = |lex: &str| -> i64 {
        c.query_row(
            "SELECT count(*) FROM lex_entry WHERE lexicon = ?1",
            [lex],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(n("LEX-HEB-OSHB"), 8_674);
    assert_eq!(n("LEX-GRC-MGNT19"), 5_624);
    assert_eq!(
        one::<String>(
            &c,
            "SELECT min(key) || ' ' || max(ord) FROM lex_entry WHERE lexicon = 'LEX-HEB-OSHB'"
        ),
        "H1 8674"
    );
    // Independent SQL facts (survey 9.1, 9.2).
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM lex_node WHERE name = 'w' AND key GLOB 'H*'"
        ),
        17_784
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM lex_attr WHERE name = 'src' AND ref_key IS NOT NULL"
        ),
        7_934
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM lex_node WHERE name = 'strongsref'"
        ),
        7_063
    );
    // Every CR inside an entry is stored (DESIGN.md 7): the database's CR
    // count equals a naive count over the entries' source bytes.
    let crs: i64 = one(
        &c,
        "SELECT sum(length(text) - length(replace(text, char(13), ''))) FROM lex_node WHERE text IS NOT NULL",
    );
    let src = &r.lex.oshb.bytes;
    let mut naive = 0i64;
    for e in &r.lex.heb.entries {
        let at = e.offset as usize;
        let len = src[at..].windows(8).position(|w| w == b"</entry>").unwrap();
        naive += src[at..at + len].iter().filter(|&&b| b == b'\r').count() as i64;
    }
    assert_eq!(crs, naive);
    assert!(crs > 40_000, "{crs}");
    assert_eq!(
        r.lex.grc.entries.iter().filter(|e| e.placeholder).count(),
        101
    );
}

/// (n, parent, name, text) of one `lex_node` row.
type NodeRow = (i64, Option<i64>, Option<String>, Option<String>);

/// Rebuilds an entry's XML from the database rows: the entry tag from the
/// key (pinned spelling), attributes in stored order and double-quoted, an
/// element without child nodes in the empty form.
fn rebuild(c: &Connection, key: &str, lexicon: &str) -> Vec<u8> {
    let own = if lexicon == lexicon::OSHB_ID {
        format!("<entry id=\"{key}\">")
    } else {
        format!(
            "<entry strongs=\"{:05}\">",
            key[1..].parse::<u32>().unwrap()
        )
    };
    let mut st = c
        .prepare("SELECT n, parent, name, text FROM lex_node WHERE key = ?1 ORDER BY n")
        .unwrap();
    let nodes: Vec<NodeRow> = st
        .query_map([key], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    let mut sa = c
        .prepare("SELECT name, value FROM lex_attr WHERE key = ?1 AND n = ?2 ORDER BY i")
        .unwrap();
    let mut out = own.into_bytes();
    let mut open: Vec<(i64, String)> = Vec::new();
    for (i, (n, parent, name, text)) in nodes.iter().enumerate() {
        while open.last().map(|o| o.0) != *parent {
            let (_, nm) = open.pop().unwrap();
            out.extend_from_slice(format!("</{nm}>").as_bytes());
        }
        match (name, text) {
            (Some(nm), None) => {
                out.extend_from_slice(format!("<{nm}").as_bytes());
                let attrs: Vec<(String, String)> = sa
                    .query_map(rusqlite::params![key, n], |r| Ok((r.get(0)?, r.get(1)?)))
                    .unwrap()
                    .collect::<rusqlite::Result<_>>()
                    .unwrap();
                for (a, v) in attrs {
                    out.extend_from_slice(format!(" {a}=\"{v}\"").as_bytes());
                }
                let has_child = nodes.get(i + 1).is_some_and(|x| x.1 == Some(*n));
                if has_child {
                    out.push(b'>');
                    open.push((*n, nm.clone()));
                } else {
                    out.extend_from_slice(b"/>");
                }
            }
            (None, Some(t)) => out.extend_from_slice(t.as_bytes()),
            _ => panic!("malformed node"),
        }
    }
    while let Some((_, nm)) = open.pop() {
        out.extend_from_slice(format!("</{nm}>").as_bytes());
    }
    out.extend_from_slice(b"</entry>");
    out
}

/// T14: every one of the 14,298 entries, rebuilt from the database alone,
/// equals the source bytes at its offset exactly: names, attributes and
/// their order, every text byte (OSHB's CRLF + TAB nodes, MGNT's hard line
/// feeds and leading spaces), nesting and order.
#[test]
fn t14_every_entry_rebuilt_from_the_db_equals_the_source_bytes() {
    let Some(r) = real() else { return };
    let c = db(r);
    for (m, src) in [
        (&r.lex.heb, &r.lex.oshb.bytes),
        (&r.lex.grc, &r.lex.mgnt.bytes),
    ] {
        for e in &m.entries {
            let key = e.key.to_string();
            let got = rebuild(&c, &key, m.source_id);
            let at = e.offset as usize;
            assert!(
                src.get(at..at + got.len()) == Some(&got[..]),
                "entry {key} differs from the source at byte {at}"
            );
        }
    }
    // Survey excerpt H1 (offset 264, length 334, pinned SHA-256); its
    // whitespace nodes keep CR LF TAB.
    let h1 = &r.lex.oshb.bytes[264..264 + 334];
    assert_eq!(
        sha256_hex(h1),
        "73eb81b081cacba9bfff098215d4165778694daaba3f89832c3653c93b245f78"
    );
    assert_eq!(rebuild(&c, "H1", lexicon::OSHB_ID), h1);
    let ws: Vec<String> = c
        .prepare("SELECT text FROM lex_node WHERE key = 'H1' AND parent IS NULL AND name IS NULL ORDER BY n")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(ws[0], "\r\n\t\t");
    assert_eq!(ws.last().unwrap(), "\r\n\t");
    assert!(ws.iter().all(|s| s.contains("\r\n\t")));
    // Excerpt H2: the one empty <source/> (H5223).
    let h2 = &r.lex.oshb.bytes[1_657_902..1_657_902 + 244];
    assert_eq!(
        sha256_hex(h2),
        "775af1cd0cdc286527891c492318130f9210daab926de89dc63fd448e02e7c16"
    );
    assert_eq!(rebuild(&c, "H5223", lexicon::OSHB_ID), h2);
    // Excerpts G2 (text directly in <entry>, hard LFs) and G3 (placeholder).
    for (key, at, len, sha) in [
        (
            "G1",
            4484,
            708,
            "f2987e0cdf76b233f086892e2e081c29410706efa058dd251513ee8cf829143f",
        ),
        (
            "G2717",
            1_157_765,
            67,
            "d12d7772b47db0788090f70eb7783ba55b67d686749bab55284f5b508d3d6b88",
        ),
    ] {
        let x = &r.lex.mgnt.bytes[at..at + len];
        assert_eq!(sha256_hex(x), sha, "{key}");
        assert_eq!(rebuild(&c, key, lexicon::MGNT_ID), x, "{key}");
    }
    // Text nodes concatenate to the entry's character data: compared with a
    // naive tag strip of the source bytes (no XML tokenizer).
    for (m, src) in [
        (&r.lex.heb, &r.lex.oshb.bytes),
        (&r.lex.grc, &r.lex.mgnt.bytes),
    ] {
        for e in m.entries.iter().step_by(97) {
            let at = e.offset as usize;
            let open_end = at + src[at..].iter().position(|&b| b == b'>').unwrap() + 1;
            let close = open_end
                + src[open_end..]
                    .windows(8)
                    .position(|w| w == b"</entry>")
                    .unwrap();
            let mut naive = Vec::new();
            let mut in_tag = false;
            for &b in &src[open_end..close] {
                match b {
                    b'<' => in_tag = true,
                    b'>' => in_tag = false,
                    _ if !in_tag => naive.push(b),
                    _ => {}
                }
            }
            let concat: String = e
                .nodes
                .iter()
                .filter_map(|n| match &n.kind {
                    NodeKind::Text(t) => Some(t.as_str()),
                    NodeKind::Element { .. } => None,
                })
                .collect();
            assert_eq!(concat.as_bytes(), &naive[..], "{}", e.key);
        }
    }
}

#[test]
fn t5_content_db_matches_the_independent_lexicon_oracle() {
    let Some(r) = real() else { return };
    let tsv = expected("lexicon-entries.tsv");
    let sources = [
        (
            lexicon::OSHB_ID,
            "HebrewStrong.xml",
            r.lex.oshb.sha256.as_str(),
        ),
        (
            lexicon::MGNT_ID,
            "strongsgreek.xml",
            r.lex.mgnt.sha256.as_str(),
        ),
    ];
    revelation_pipeline::oracle::compare_lexicons(&db(r), &tsv, &sources)
        .unwrap_or_else(|f| panic!("{f}"));
    let gen_path =
        revelation_pipeline::repo_dir().join("docs/pipeline/survey-scripts/oracle_lexicons.py");
    let gen_sha = sha256_hex(&fs::read(gen_path).unwrap());
    assert_eq!(
        revelation_pipeline::oracle::header_value(&tsv, "generator").as_deref(),
        Some(gen_sha.as_str()),
        "lexicon-entries.tsv: stale generator hash"
    );
    // A one-byte text change in the database: T14 and T5 both see it.
    let bytes = mutate(
        &r.bytes,
        "UPDATE lex_node SET text = replace(text, 'father', 'fathe') WHERE key = 'H1' AND name IS NULL",
    );
    let c = open_bytes(&bytes).unwrap();
    let f = revelation_pipeline::oracle::compare_lexicons(&c, &tsv, &sources).unwrap_err();
    assert!(
        f.to_string()
            .contains("entry H1 differs in [\"text_bytes\", \"text_sha256\", \"tree_sha256\"]"),
        "{f}"
    );
    let f = check::check_lex(&c, &models(r), &r.lex.meta()).unwrap_err();
    assert!(f.has("T14"), "{f}");
}

// ------------------------------------------------------------ T13

#[test]
fn t13_every_kjv_strongs_key_resolves_to_a_used_lexicon_entry() {
    let Some(r) = real() else { return };
    let raw = common::sources_dir().unwrap();
    let kjv = run::prepare_kjv(&raw).unwrap_or_else(|e| panic!("{e}"));
    let keys: Vec<_> = kjv
        .model
        .verses
        .iter()
        .flat_map(|v| &v.words)
        .flat_map(|w| &w.keys)
        .collect();
    let distinct: std::collections::BTreeSet<_> = keys.iter().collect();
    eprintln!(
        "T13: {} key links, {} distinct keys",
        keys.len(),
        distinct.len()
    );
    // Survey section 6 counts 14,089 distinct keys in the whole file; 14 of
    // them occur only in psalm titles and colophons, which are not imported
    // (probe 2026-10-22 with osis_tokens.strict_tokenize), so verse text has
    // 14,075. All 14,089 resolve.
    assert_eq!(distinct.len(), 14_075);
    let bad = lexicon::unresolved_keys(keys.iter().copied(), &models(r));
    assert!(bad.is_empty(), "unresolved KJV keys: {bad:?}");
    let title_only: Vec<revelation_pipeline::strongs::StrongsKey> = [
        "G2858", "G3390", "G3505", "G3818", "H1665", "H3128", "H4210", "H4257", "H4387", "H4905",
        "H5155", "H516", "H760", "H7802",
    ]
    .iter()
    .map(|k| revelation_pipeline::strongs::StrongsKey::from_prefixed(k).unwrap())
    .collect();
    assert!(title_only.iter().all(|k| !distinct.contains(&k)));
    assert!(lexicon::unresolved_keys(&title_only, &models(r)).is_empty());
    run::resolve_kjv(&kjv.model, &models(r)).unwrap();
    // The same check bites: without entry H430 (Elohim) or with G1 marked a
    // placeholder, the exact keys and their link counts are listed.
    let mut heb = r.lex.heb.clone();
    heb.entries.retain(|e| e.key.to_string() != "H430");
    let mut grc = r.lex.grc.clone();
    grc.entries[0].placeholder = true;
    let f = run::resolve_kjv(&kjv.model, &[&heb, &grc]).unwrap_err();
    assert!(f.has("T13"), "{f}");
    let msg = f.to_string();
    assert!(
        msg.contains("2 KJV Strong's keys") && msg.contains("H430 (") && msg.contains("G1 ("),
        "{msg}"
    );
}

// ------------------------------------------------------------ T16, T17, mutations

#[test]
fn t16_two_builds_are_identical_and_match_the_committed_hash() {
    let Some(r) = real() else { return };
    let out = common::temp_dir("lex-real-2");
    let h2 = run::publish_lexicons(&out, &r.lex, &mut |_| Ok(())).unwrap();
    assert_eq!(h2, r.hash);
    assert_eq!(fs::read(out.join(LEX_DEST)).unwrap(), r.bytes);
    // The same parser the app's build.rs uses to compile the value in.
    let entries = revelation_content::hashfile::parse(&expected("output-sha256.txt")).unwrap();
    assert_eq!(
        revelation_content::hashfile::lookup(&entries, LEX_DEST).unwrap(),
        r.hash,
        "built content.sqlite differs from output-sha256.txt"
    );
    let keys = revelation_pipeline::db::query_strings(&db(r), "SELECT key FROM meta").unwrap();
    assert_eq!(keys, ["schema_version"]);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn t17_check_refuses_a_sidecar_and_a_stale_checksum() {
    let Some(r) = real() else { return };
    let out = common::temp_dir("lex-real-t17");
    for f in [LEX_DEST, "content.sqlite.sha256"] {
        fs::copy(r.out.join(f), out.join(f)).unwrap();
    }
    assert_eq!(run::check_published_lex(&r.lex, &out).unwrap(), r.hash);
    for side in [
        "content.sqlite-wal",
        "content.sqlite-journal",
        "content.sqlite-shm",
    ] {
        fs::write(out.join(side), b"x").unwrap();
        let f = run::check_published_lex(&r.lex, &out).unwrap_err();
        assert!(f.has("sidecar"), "{side}: {f}");
        fs::remove_file(out.join(side)).unwrap();
    }
    fs::write(
        out.join("content.sqlite.sha256"),
        format!("{}  {LEX_DEST}\n", "0".repeat(64)),
    )
    .unwrap();
    let f = run::check_published_lex(&r.lex, &out).unwrap_err();
    assert!(f.has("sha256"), "{f}");
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn t14_database_mutations_each_fail_the_named_check() {
    let Some(r) = real() else { return };
    let cases: [(&str, &str, &str); 9] = [
        (
            "drop a CR",
            "UPDATE lex_node SET text = replace(text, char(13), '') WHERE key = 'H1' AND n = 0",
            "T14",
        ),
        (
            "drop a node",
            "DELETE FROM lex_node WHERE key = 'G1' AND n = (SELECT max(n) FROM lex_node WHERE key = 'G1')",
            "T14",
        ),
        (
            "swap attribute order",
            "UPDATE lex_attr SET i = 1 - i WHERE key = 'H1' AND n = 1 AND i IN (0, 1)",
            "T14",
        ),
        (
            "change an attribute value",
            "UPDATE lex_attr SET value = 'awv' WHERE key = 'H1' AND name = 'pron'",
            "T14",
        ),
        (
            "dangling cross-reference",
            "UPDATE lex_attr SET ref_key = 'H9999' WHERE key = 'H2' AND name = 'src'",
            "lex-ref",
        ),
        (
            "reparent a node",
            "UPDATE lex_node SET parent = NULL WHERE key = 'H1' AND n = 2",
            "T14",
        ),
        (
            "move an entry",
            "UPDATE lex_entry SET ord = ord + 1 WHERE key = 'G5624'",
            "T14",
        ),
        (
            "delete an entry",
            "DELETE FROM lex_attr WHERE key = 'G5624'; DELETE FROM lex_node WHERE key = 'G5624'; DELETE FROM lex_entry WHERE key = 'G5624'",
            "T14",
        ),
        (
            "meta row",
            "INSERT INTO meta VALUES ('pipeline_commit', 'unknown-dirty')",
            "meta",
        ),
    ];
    for (name, sql, want) in cases {
        let c = open_bytes(&mutate(&r.bytes, sql)).unwrap();
        let f = check::check_lex(&c, &models(r), &r.lex.meta()).expect_err(name);
        assert!(f.has(want), "{name}: expected {want}, got {:?}", f.checks());
    }
}

#[test]
fn source_mutations_fail_closed_or_are_caught_by_the_oracle() {
    let Some(r) = real() else { return };
    let heb = &r.lex.oshb.bytes;
    let grc = &r.lex.mgnt.bytes;
    let find =
        |hay: &[u8], needle: &[u8]| hay.windows(needle.len()).position(|w| w == needle).unwrap();
    let edit = |src: &[u8], needle: &[u8], with: &[u8]| -> Vec<u8> {
        let at = find(src, needle);
        let mut v = src[..at].to_vec();
        v.extend_from_slice(with);
        v.extend_from_slice(&src[at + needle.len()..]);
        v
    };
    let oshb = lexicon::oshb();
    let mgnt = lexicon::mgnt();
    let mut cases: Vec<(&str, Vec<u8>, &lexicon::LexProfile, &str)> = Vec::new();
    cases.push((
        "a CR inside a def",
        edit(heb, b"<def>father</def>", b"<def>fa\r\nther</def>"),
        &oshb,
        "lex-char",
    ));
    cases.push((
        "a bare LF",
        edit(
            heb,
            b"\r\n\t\t<source>a primitive word;",
            b"\n\t\t<source>a primitive word;",
        ),
        &oshb,
        "lex-char",
    ));
    cases.push((
        "H01 next to H1",
        edit(heb, b"<entry id=\"H2\">", b"<entry id=\"H02\">"),
        &oshb,
        "lex-key",
    ));
    cases.push((
        "w@src padded",
        edit(heb, b"src=\"H24\"", b"src=\"H024\""),
        &oshb,
        "lex-ref",
    ));
    cases.push((
        "U+202E in text",
        edit(
            heb,
            b"<def>father</def>",
            "<def>fa\u{202E}ther</def>".as_bytes(),
        ),
        &oshb,
        "lex-char",
    ));
    cases.push((
        "unknown element",
        edit(heb, b"<def>father</def>", b"<b>father</b>"),
        &oshb,
        "form",
    ));
    cases.push((
        "element in the wrong parent",
        edit(heb, b"<def>father</def>", b"<usage>father</usage>"),
        &oshb,
        "lex-structure",
    ));
    cases.push((
        "extra attribute",
        edit(heb, b"<entry id=\"H2\">", b"<entry id=\"H2\" x=\"1\">"),
        &oshb,
        "lex-attr",
    ));
    cases.push((
        "one entry fewer",
        edit(heb, b"<entry id=\"H8674\">", b"<entryX id=\"H8674\">"),
        &oshb,
        "form",
    ));
    cases.push((
        "strongsref language=\"Greek\"",
        edit(
            grc,
            b"language=\"GREEK\" strongs=\"427\"",
            b"language=\"Greek\" strongs=\"427\"",
        ),
        &mgnt,
        "lex-ref",
    ));
    cases.push((
        "strongs=\"0\"",
        edit(
            grc,
            b"language=\"GREEK\" strongs=\"427\"",
            b"language=\"GREEK\" strongs=\"0\"",
        ),
        &mgnt,
        "lex-ref",
    ));
    cases.push((
        "Hebrew ref unpadded",
        edit(grc, b"strongs=\"0175\"", b"strongs=\"175\""),
        &mgnt,
        "lex-ref",
    ));
    cases.push((
        "stale exception",
        edit(grc, b"strongs=\"5689\"", b"strongs=\"05689\""),
        &mgnt,
        "lex-ref",
    ));
    cases.push((
        "<strongs> differs",
        edit(grc, b"<strongs>1</strongs>", b"<strongs>01</strongs>"),
        &mgnt,
        "lex-structure",
    ));
    cases.push((
        "TAB in an entry",
        edit(grc, b" Often used", b"\tOften used"),
        &mgnt,
        "lex-char",
    ));
    cases.push((
        "greek without translit",
        edit(grc, b" translit=\"A\"/>", b"/>"),
        &mgnt,
        "lex-attr",
    ));
    cases.push((
        "DTD subset changed",
        edit(
            grc,
            b"<!ELEMENT latin (#PCDATA) >",
            b"<!ELEMENT latin (#PCDATA)  >",
        ),
        &mgnt,
        "dtd",
    ));
    cases.push((
        "DTD subset moved",
        edit(grc, b"standalone='yes'?>\n", b"standalone='yes'?>\n\n"),
        &mgnt,
        "dtd",
    ));
    cases.push((
        "second DOCTYPE",
        edit(
            grc,
            b"<strongsdictionary>",
            b"<!DOCTYPE x><strongsdictionary>",
        ),
        &mgnt,
        "dtd",
    ));
    cases.push((
        "comment after 1090",
        edit(
            grc,
            b"<strongsdictionary>",
            b"<!-- x --><strongsdictionary>",
        ),
        &mgnt,
        "dtd",
    ));
    cases.push((
        "entity",
        edit(grc, b"--Alpha.", b"--Alpha &amp;."),
        &mgnt,
        "entity",
    ));
    cases.push((
        "placeholder gone",
        edit(
            grc,
            b"<strongs>2717</strongs>  Not Used",
            b"<strongs>2717</strongs>  Not used",
        ),
        &mgnt,
        "lex-facts",
    ));
    cases.push((
        "17 attributes",
        edit(
            grc,
            b"<greek BETA=\"*A\"",
            format!(
                "<greek BETA=\"*A\"{}",
                (0..16).map(|i| format!(" a{i}=\"x\"")).collect::<String>()
            )
            .as_bytes(),
        ),
        &mgnt,
        "cap",
    ));
    for (name, bytes, p, want) in &cases {
        let f = lexicon::parse(bytes, p).expect_err(name);
        assert!(f.has(want), "{name}: expected {want}, got {f}");
    }
    // A text change no fact covers parses, and the oracle sees it.
    let changed = edit(heb, b"<def>father</def>", b"<def>fathex</def>");
    let m = lexicon::parse(&changed, &oshb).unwrap();
    let mut c = Connection::open_in_memory().unwrap();
    revelation_pipeline::db::write_lex(&mut c, &[&m, &r.lex.grc], &r.lex.meta()).unwrap();
    let bytes = c.serialize("main").unwrap().to_vec();
    let sources = [
        (
            lexicon::OSHB_ID,
            "HebrewStrong.xml",
            r.lex.oshb.sha256.as_str(),
        ),
        (
            lexicon::MGNT_ID,
            "strongsgreek.xml",
            r.lex.mgnt.sha256.as_str(),
        ),
    ];
    let f = revelation_pipeline::oracle::compare_lexicons(
        &open_bytes(&bytes).unwrap(),
        &expected("lexicon-entries.tsv"),
        &sources,
    )
    .unwrap_err();
    assert!(
        f.to_string()
            .contains("entry H1 differs in [\"text_sha256\", \"tree_sha256\"]"),
        "{f}"
    );
}
