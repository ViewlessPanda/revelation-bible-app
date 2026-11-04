//! MHCC commentary on the real pinned file (REVELATION_SOURCES), DESIGN.md
//! 17: parse with every pinned fact, mhcc.sqlite built and checked (T22:
//! every block rebuilt from its rows equals its source bytes), T5 against the
//! Python oracle files, every anchor and reference a KJV verse, T16
//! (determinism, committed expected hash), the label rows equal the
//! scholar's block, and database mutations asserting which check fires.

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use revelation_pipeline::check;
use revelation_pipeline::commentary::{self, MHCC_DEST};
use revelation_pipeline::labels;
use revelation_pipeline::open::{open_bytes, open_verified};
use revelation_pipeline::oracle;
use revelation_pipeline::run::{self, PreparedCmt};
use revelation_pipeline::sources::sha256_hex;
use rusqlite::Connection;

struct Real {
    cmt: PreparedCmt,
    out: PathBuf,
    hash: String,
    bytes: Vec<u8>,
}

fn real() -> Option<&'static Real> {
    static R: OnceLock<Option<Real>> = OnceLock::new();
    R.get_or_init(|| {
        let raw = common::sources_dir()?;
        let kjv = run::prepare_kjv(&raw).unwrap_or_else(|e| panic!("{e}"));
        let cmt = run::prepare_mhcc(&raw, &kjv).unwrap_or_else(|e| panic!("{e}"));
        let out = common::temp_dir("mhcc-real");
        let hash = run::publish_mhcc(&out, &cmt, &mut |_| Ok(())).unwrap_or_else(|e| panic!("{e}"));
        let bytes = fs::read(out.join(MHCC_DEST)).unwrap();
        Some(Real {
            cmt,
            out,
            hash,
            bytes,
        })
    })
    .as_ref()
}

fn db(r: &Real) -> Connection {
    open_verified(&r.out.join(MHCC_DEST), Some(&r.hash))
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

fn check(r: &Real, c: &Connection) -> revelation_pipeline::error::Result<()> {
    check::check_cmt(
        c,
        &r.cmt.model,
        &r.cmt.meta(),
        &r.cmt.src.bytes,
        Some(&r.cmt.table),
    )
}

// ------------------------------------------------------------ positive

#[test]
fn built_mhcc_db_passes_every_check_and_reads_back_the_counts() {
    let Some(r) = real() else { return };
    let c = db(r);
    check(r, &c).unwrap_or_else(|e| panic!("{e}"));
    // Survey facts, read back from the database with plain SQL.
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM cmt_block"), 16_255);
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM cmt_anchor"), 4_297);
    assert_eq!(
        one::<i64>(&c, "SELECT count(*) FROM cmt_anchor_verse"),
        28_968
    );
    assert_eq!(
        one::<i64>(&c, "SELECT count(DISTINCT verse_id) FROM cmt_anchor_verse"),
        28_718
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM cmt_block WHERE anchor IS NOT NULL"
        ),
        8_078
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM cmt_attr WHERE ref_start IS NOT NULL"
        ),
        1_204
    );
    assert_eq!(
        one::<i64>(&c, "SELECT count(*) FROM cmt_node WHERE name = 'hi'"),
        3_930
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM cmt_attr WHERE ref_start > ref_end"
        ),
        4
    );
    assert_eq!(
        one::<String>(&c, "SELECT value FROM meta WHERE key = 'commentary_source'"),
        "CMT-MHCC-CW20"
    );
    assert_eq!(
        one::<String>(&c, "SELECT sha256 FROM source WHERE id = 'CMT-MHCC-CW20'"),
        "b00780749c2d6130f41441db2b5cb16df490231e6a315a7d2d9eabcfb36a9fd7"
    );
    // The meta keys besides the scholar's `label.*` rows (DESIGN.md 17.5).
    let keys: Vec<String> = c
        .prepare("SELECT key FROM meta WHERE key NOT LIKE 'label.%' ORDER BY key")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        keys,
        [
            "commentary_source",
            "label_cited_from",
            "label_file",
            "label_file_sha256",
            "schema_version",
            "verse_ids",
            "versification_overrides_sha256",
        ]
    );
    // The app's query: the comment blocks for one verse.
    let for_verse = |v: i64| -> Vec<i64> {
        c.prepare(
            "SELECT b.id FROM cmt_anchor_verse v JOIN cmt_block b ON b.anchor = v.anchor              WHERE v.verse_id = ?1 ORDER BY b.id",
        )
        .unwrap()
        .query_map([v], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    // John 3:1 is in the source's anchor John.3.1-8 (sID line 43801).
    assert!(!for_verse(43_003_001).is_empty());
    // John 3:16 is in no anchor: the source's anchors in John 3 are 1-8 and
    // 22-36 only (its comment on 9-21 is inside the 1-8 anchor's text). The
    // anchor table is stored as the source lists it, not filled in.
    assert!(for_verse(43_003_016).is_empty());
}

#[test]
fn every_anchor_and_reference_is_a_kjv_verse() {
    let Some(r) = real() else { return };
    // resolve() ran in prepare_mhcc; run it again here so the claim has its
    // own test, and count what was resolved.
    commentary::resolve(&r.cmt.model, &r.cmt.table).unwrap_or_else(|e| panic!("{e}"));
    let refs: usize = r.cmt.model.anchors.iter().map(|a| a.verses.len()).sum();
    assert_eq!(refs, 28_968);
}

#[test]
fn t5_database_equals_the_python_oracle_files() {
    let Some(r) = real() else { return };
    let c = db(r);
    let src = (
        commentary::MHCC_ID,
        "mhcc.osis.xml",
        r.cmt.src.sha256.as_str(),
    );
    oracle::compare_cmt(
        &c,
        &expected("mhcc-blocks.tsv"),
        &expected("mhcc-anchors.tsv"),
        src,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // A changed oracle row is reported by name.
    let blocks =
        expected("mhcc-blocks.tsv").replacen("\n2\tGen\t1\t-\tp\t", "\n2\tGen\t2\t-\tp\t", 1);
    let f = oracle::compare_cmt(&c, &blocks, &expected("mhcc-anchors.tsv"), src).unwrap_err();
    assert!(
        f.has("T5") && f.to_string().contains("row 2 differs in [\"chapter\"]"),
        "{f}"
    );
}

#[test]
fn t16_two_builds_are_identical_and_equal_the_committed_hash() {
    let Some(r) = real() else { return };
    let out2 = common::temp_dir("mhcc-real-2");
    let h2 = run::publish_mhcc(&out2, &r.cmt, &mut |_| Ok(())).unwrap();
    assert_eq!(h2, r.hash);
    assert_eq!(fs::read(out2.join(MHCC_DEST)).unwrap(), r.bytes);
    assert_eq!(sha256_hex(&r.bytes), r.hash);
    let sums = expected("output-sha256.txt");
    let want = sums
        .lines()
        .find(|l| l.ends_with(&format!("  {MHCC_DEST}")))
        .and_then(|l| l.split(' ').next())
        .expect("mhcc.sqlite line in output-sha256.txt");
    assert_eq!(r.hash, want);
    let _ = fs::remove_dir_all(&out2);
}

#[test]
fn check_subcommand_accepts_the_published_file() {
    let Some(r) = real() else { return };
    assert_eq!(run::check_published_mhcc(&r.cmt, &r.out).unwrap(), r.hash);
}

// ------------------------------------------------------------ labels

/// T23: the committed label file is byte-for-byte the YAML block of the
/// scholar's document (§5), and its rows are in meta exactly as written.
#[test]
fn label_file_equals_the_scholars_block_and_is_in_meta() {
    let doc = fs::read(revelation_pipeline::repo_dir().join("docs/scholar/commentary-labels.md"))
        .unwrap();
    let open = b"```yaml\n";
    let s = doc.windows(open.len()).position(|w| w == open).unwrap() + open.len();
    let e = s + doc[s..].windows(4).position(|w| w == b"\n```").unwrap() + 1;
    let data = fs::read(labels::mhcc_labels_path()).unwrap();
    assert_eq!(
        &doc[s..e],
        &data[..],
        "pipeline/data/mhcc-labels.yaml differs from the doc block"
    );
    // Exactly one YAML block in the doc.
    assert_eq!(doc.windows(open.len()).filter(|w| *w == open).count(), 1);

    let rows = run::load_labels(commentary::MHCC_ID).unwrap();
    let get = |k: &str| rows.iter().find(|r| r.0 == k).map(|r| r.1.as_str());
    assert_eq!(
        get("label.display_name"),
        Some("Matthew Henry's Concise Commentary")
    );
    assert_eq!(
        get("label.author_label"),
        Some(
            "Matthew Henry \u{b7} Presbyterian (English Nonconformist), with Thomas Scott \u{b7} Anglican"
        )
    );
    assert_eq!(
        get("label.date_label"),
        Some(
            "Abridgement c. 1838 (Religious Tract Society), from Henry's Exposition (1708\u{2013}1710)"
        )
    );
    assert_eq!(get("label.date_label_status"), Some("to_confirm"));
    assert_eq!(
        get("label.date_label_status.comment"),
        Some("\"c. 1838\"; fallback \"1830s\u{2013}1850s\"")
    );
    assert_eq!(
        get("label.to_confirm.1"),
        Some("date c. 1838 (Google Books 1838 vs Internet Archive [185-?])")
    );
    // W2-LBL: the owner approved the full label on 2026-10-27; the
    // to_confirm items stay as written.
    assert_eq!(get("label.status"), Some(labels::LABEL_APPROVED));
    assert_eq!(get("label.status.comment"), None);
    assert_eq!(get("label.approved_by"), Some("owner"));
    assert_eq!(get("label.approved_on"), Some("2026-10-27"));
    assert_eq!(
        get("label.to_confirm.2"),
        Some("Moody 1982 attribution to Henry and Scott")
    );
    assert_eq!(
        get("label.authors.2.role.comment"),
        Some("displayed as \"with Thomas Scott\"")
    );
    assert_eq!(
        get("label.citations.8").map(|s| s.starts_with("Retailer listing")),
        Some(true)
    );
    assert_eq!(get("label.citations.9"), None);
    assert_eq!(get("label_file_sha256"), Some(sha256_hex(&data).as_str()));
    // Every quoted value of the block occurs in the doc verbatim.
    let text = String::from_utf8(doc).unwrap();
    for (k, v) in &rows {
        if k.starts_with("label.") {
            assert!(text.contains(v.as_str()), "{k}: {v}");
        }
    }

    let Some(r) = real() else { return };
    let c = db(r);
    for (k, v) in &rows {
        let got: String = c
            .query_row("SELECT value FROM meta WHERE key = ?1", [k], |r| r.get(0))
            .unwrap();
        assert_eq!(&got, v, "{k}");
    }
}

// ------------------------------------------------------------ mutations

#[test]
fn database_mutations_fail_the_named_check() {
    let Some(r) = real() else { return };
    for (name, sql, want) in [
        (
            "text",
            "UPDATE cmt_node SET text = text || ' ' WHERE block = 2 AND n = 1",
            "T22",
        ),
        (
            "amp-not-decoded",
            "UPDATE cmt_node SET text = replace(text, '&', '&amp;') WHERE text LIKE '%&%'",
            "T22",
        ),
        (
            "attr-order",
            "UPDATE cmt_attr SET i = 1 - i WHERE block = 1 AND n = 0 AND i IN (0, 1)",
            "T22",
        ),
        (
            "anchor-verse",
            "UPDATE cmt_anchor_verse SET verse_id = verse_id + 1 WHERE anchor = 1 AND k = 0",
            "T22",
        ),
        (
            "anchor-verse-not-kjv",
            "UPDATE cmt_anchor_verse SET verse_id = 1001099 WHERE anchor = 1 AND k = 0",
            "cmt-kjv",
        ),
        (
            "block-out-of-anchor",
            "UPDATE cmt_block SET anchor = NULL WHERE id = (SELECT min(id) FROM cmt_block WHERE anchor = 2)",
            "T22",
        ),
        (
            "block-chapter",
            "UPDATE cmt_block SET chapter = 2 WHERE id = (SELECT min(id) FROM cmt_block WHERE anchor = 1)",
            "T22",
        ),
        (
            "ref-dropped",
            "UPDATE cmt_attr SET ref_start = NULL, ref_end = NULL WHERE ref_start IS NOT NULL",
            "cmt-ref",
        ),
        (
            "ref-end",
            "UPDATE cmt_attr SET ref_end = 1001099 WHERE ref_start IS NOT NULL",
            "cmt-kjv",
        ),
        (
            "node-parent-text",
            "UPDATE cmt_node SET parent = 1 WHERE block = (SELECT block FROM cmt_node WHERE n = 2 AND name IS NOT NULL LIMIT 1) AND n = 2 AND (SELECT name FROM cmt_node x WHERE x.block = cmt_node.block AND x.n = 1) IS NULL",
            "T22",
        ),
        (
            "block-deleted",
            "DELETE FROM cmt_node WHERE block = 5; DELETE FROM cmt_attr WHERE block = 5; DELETE FROM cmt_block WHERE id = 5",
            "T22",
        ),
        (
            "meta",
            "UPDATE meta SET value = 'X' WHERE key = 'commentary_source'",
            "meta",
        ),
        (
            "meta-label",
            "UPDATE meta SET value = 'c. 1838' WHERE key = 'label.date_label_status'",
            "meta",
        ),
        (
            "source",
            "UPDATE source SET size = size + 1 WHERE id = 'CMT-MHCC-CW20'",
            "meta",
        ),
        ("index", "DROP INDEX cmt_anchor_verse_by_verse", "schema"),
        ("app-id", "PRAGMA application_id = 0", "schema"),
        ("user-version", "PRAGMA user_version = 99", "schema"),
    ] {
        let c = open_bytes(&mutate(&r.bytes, sql)).unwrap();
        let f = check(r, &c).expect_err(name);
        assert!(f.has(want), "{name}: expected {want}, got {:?}", f.checks());
    }
}
