//! Tests on the real pinned KJV (REVELATION_SOURCES): T3, T4, T6 (V5), T7,
//! T8, T11 (mutations, asserting which check fires), T13 counts, T16
//! (determinism and the committed expected hash), T17 (check refuses
//! sidecars) on the built database.

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use revelation_pipeline::check;
use revelation_pipeline::db::Meta;
use revelation_pipeline::error::Failure;
use revelation_pipeline::open::{open_bytes, open_verified};
use revelation_pipeline::osis;
use revelation_pipeline::run::{self, KJV_DEST, Prepared};
use revelation_pipeline::validate;
use rusqlite::Connection;

struct Real {
    prep: Prepared,
    meta: Meta,
    out: PathBuf,
    hash: String,
    bytes: Vec<u8>,
}

fn real() -> Option<&'static Real> {
    static R: OnceLock<Option<Real>> = OnceLock::new();
    R.get_or_init(|| {
        let raw = common::sources_dir()?;
        let t0 = std::time::Instant::now();
        let prep = run::prepare_kjv(&raw).unwrap_or_else(|e| panic!("{e}"));
        let meta = prep.meta();
        let out = common::temp_dir("kjv-real");
        let hash = run::publish_model(
            &out,
            KJV_DEST,
            &prep.model,
            &prep.profile,
            Some(&prep.table),
            &meta,
            &mut |_| Ok(()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let bytes = fs::read(out.join(KJV_DEST)).unwrap();
        eprintln!("real build: {:?}", t0.elapsed());
        Some(Real {
            prep,
            meta,
            out,
            hash,
            bytes,
        })
    })
    .as_ref()
}

fn db(r: &Real) -> Connection {
    open_verified(&r.out.join(KJV_DEST), Some(&r.hash))
        .unwrap()
        .0
}

fn one<T: rusqlite::types::FromSql>(c: &Connection, q: &str) -> T {
    c.query_row(q, [], |r| r.get(0)).unwrap()
}

// ------------------------------------------------------------ positive

#[test]
fn built_db_passes_every_check_and_the_pinned_numbers() {
    let Some(r) = real() else { return };
    let c = db(r);
    check::check(
        &c,
        &r.prep.model,
        &r.prep.profile,
        Some(&r.prep.table),
        &r.meta,
    )
    .unwrap();
    // Independent SQL read-back of the pinned facts (T3, T6 V5, T8).
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM verse"), 31_102);
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM book"), 66);
    assert_eq!(one::<i64>(&c, "SELECT sum(chapters) FROM book"), 1_189);
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM (SELECT DISTINCT book, chapter FROM verse)"
        ),
        1_189
    );
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM red_piece"), 2_035);
    assert_eq!(
        one::<i64>(&c, "SELECT count(DISTINCT q_ord) FROM red_piece"),
        648
    );
    assert_eq!(one::<i64>(&c, "SELECT max(q_ord) FROM red_piece"), 648);
    assert_eq!(
        one::<i64>(&c, "SELECT sum(end_cp - start_cp) FROM red_piece"),
        210_502
    );
    assert_eq!(
        one::<i64>(&c, "SELECT count(DISTINCT verse_id) FROM red_piece"),
        2_026
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM verse WHERE substr(text, -1) = ' '"
        ),
        91
    );
    assert_eq!(
        one::<i64>(&c, "SELECT count(*) FROM mark WHERE kind = 'added'"),
        21_571
    );
    assert_eq!(
        one::<i64>(&c, "SELECT count(*) FROM mark WHERE kind = 'divine'"),
        6_887
    );
    assert_eq!(one::<i64>(&c, "SELECT count(*) FROM word"), 355_295);
    assert_eq!(
        one::<i64>(&c, "SELECT count(*) FROM word WHERE start_cp = end_cp"),
        6_330
    );
    assert_eq!(
        one::<i64>(
            &c,
            "SELECT count(*) FROM verse WHERE book <= 39 AND id IN (SELECT verse_id FROM red_piece)"
        ),
        0
    );
    assert_eq!(
        one::<String>(&c, "SELECT value FROM meta WHERE key = 'has_red_letters'"),
        "1"
    );
    // Per book, against DESIGN.md 5.4.
    let mut st = c
        .prepare(
            "SELECT b.osis, count(*), sum(r.end_cp - r.start_cp),
                    (SELECT count(*) FROM red_piece x JOIN verse y ON y.id = x.verse_id
                      WHERE y.book = b.ord AND x.verse_id = (SELECT min(verse_id) FROM red_piece z WHERE z.q_ord = x.q_ord))
               FROM red_piece r JOIN verse v ON v.id = r.verse_id JOIN book b ON b.ord = v.book
              GROUP BY b.ord ORDER BY b.ord",
        )
        .unwrap();
    let rows: Vec<(String, i64, i64, i64)> = st
        .query_map([], |x| Ok((x.get(0)?, x.get(3)?, x.get(1)?, x.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let want: Vec<(String, i64, i64, i64)> = [
        ("Matt", 146, 646, 69_530),
        ("Mark", 131, 285, 27_482),
        ("Luke", 185, 589, 62_696),
        ("John", 153, 422, 39_461),
        ("Acts", 21, 28, 2_479),
        ("1Cor", 2, 2, 181),
        ("2Cor", 1, 1, 77),
        ("Rev", 9, 62, 8_596),
    ]
    .iter()
    .map(|&(b, q, p, cp)| (b.to_string(), q, p, cp))
    .collect();
    assert_eq!(rows, want);
    // John 3:16, the audit query from DESIGN.md 8.
    let red: String = one(
        &c,
        "SELECT substr(v.text, r.start_cp + 1, r.end_cp - r.start_cp) FROM red_piece r JOIN verse v ON v.id = r.verse_id WHERE v.id = 43003016",
    );
    assert!(red.starts_with("For God so loved the world"), "{red}");
}

#[test]
fn t3_every_chapter_matches_the_versification_table() {
    let Some(r) = real() else { return };
    let c = db(r);
    let mut st = c
        .prepare("SELECT b.usfm, v.chapter, count(*), max(v.verse) FROM verse v JOIN book b ON b.ord = v.book GROUP BY v.book, v.chapter ORDER BY v.book, v.chapter")
        .unwrap();
    let rows: Vec<(String, u16, i64, i64)> = st
        .query_map([], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(rows.len(), 1_189);
    for (usfm, ch, n, max) in rows {
        let want = r.prep.table.get(&usfm, ch).unwrap();
        assert_eq!((n, max), (i64::from(want), i64::from(want)), "{usfm} {ch}");
    }
    assert_eq!(r.prep.table.get("3JN", 1), Some(14));
    assert_eq!(r.prep.table.get("REV", 12), Some(17));
}

#[test]
fn t4_t7_every_verse_round_trips_through_open_verified() {
    let Some(r) = real() else { return };
    let c = db(r);
    let mut st = c.prepare("SELECT id, text FROM verse ORDER BY id").unwrap();
    let texts: Vec<(i64, String)> = st
        .query_map([], |x| Ok((x.get(0)?, x.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(texts.len(), r.prep.model.verses.len());
    let mut st = c
        .prepare("SELECT verse_id, start_cp, end_cp, q_ord FROM red_piece ORDER BY verse_id, n")
        .unwrap();
    let red: Vec<(i64, u32, u32, u32)> = st
        .query_map([], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut want_red = Vec::new();
    for (v, (id, text)) in r.prep.model.verses.iter().zip(&texts) {
        assert_eq!((v.id(), &v.text), (*id, text), "{}", v.osis);
        for p in &v.red {
            want_red.push((v.id(), p.start, p.end, p.q_ord));
        }
    }
    assert_eq!(red, want_red);
    // The 91 trailing spaces are stored as is.
    let trailing = texts.iter().filter(|(_, t)| t.ends_with(' ')).count();
    assert_eq!(trailing, 91);
    assert!(
        texts
            .iter()
            .any(|(id, t)| *id == 40008009 && t.ends_with("it. ")),
        "Matt 8:9 keeps its space"
    );
}

// ------------------------------------------------------------ T11 mutations

fn mutate(bytes: &[u8], sql: &str) -> Vec<u8> {
    let mut c = Connection::open_in_memory().unwrap();
    c.deserialize_read_exact("main", bytes, bytes.len(), false)
        .unwrap();
    c.execute_batch(sql).unwrap();
    c.serialize("main").unwrap().to_vec()
}

fn check_bytes(r: &Real, bytes: &[u8]) -> Result<(), Failure> {
    let c = open_bytes(bytes)?;
    check::check(
        &c,
        &r.prep.model,
        &r.prep.profile,
        Some(&r.prep.table),
        &r.meta,
    )
}

#[test]
fn t11_database_mutations_each_fail_the_named_check() {
    let Some(r) = real() else { return };
    let m = &r.prep.model;
    // A q whose last piece ends at a verse end, followed directly by a q
    // starting at 0 in the next verse (merge at a verse edge).
    let mut edge = None;
    let mut leak = None;
    for (i, v) in m.verses.iter().enumerate().take(m.verses.len() - 1) {
        let n = &m.verses[i + 1];
        if let (Some(a), Some(b)) = (v.red.last(), n.red.first())
            && a.end == v.len
            && b.start == 0
            && b.q_ord == a.q_ord + 1
            && edge.is_none()
        {
            edge = Some(b.q_ord);
        }
        if let Some(a) = v.red.last()
            && a.end == v.len
            && n.red.is_empty()
            && n.book == v.book
            && leak.is_none()
        {
            leak = Some((n.id(), a.q_ord));
        }
    }
    let edge = edge.expect("a verse-edge q pair exists");
    let (leak_v, leak_q) = leak.expect("a q ending at a verse end before a black verse");
    let cases: Vec<(&str, String, &[&str], &[&str])> = vec![
        ("drop", "DELETE FROM red_piece WHERE verse_id = 40003015".into(), &["T7", "T8", "R6"], &[]),
        ("add", "INSERT INTO red_piece VALUES (40003017, 0, 0, 5, 649)".into(), &["T7", "T8"], &[]),
        ("shift+1", "UPDATE red_piece SET start_cp = start_cp + 1, end_cp = end_cp + 1 WHERE verse_id = 40003015".into(), &["T7"], &["T8"]),
        ("shift-1", "UPDATE red_piece SET start_cp = start_cp - 1, end_cp = end_cp - 1 WHERE verse_id = 40003015".into(), &["T7"], &["T8"]),
        ("extend", "UPDATE red_piece SET end_cp = end_cp + 1 WHERE verse_id = 40003015".into(), &["T7", "T8"], &[]),
        ("trim", "UPDATE red_piece SET start_cp = start_cp + 1 WHERE verse_id = 40003015".into(), &["T7", "T8"], &[]),
        (
            "split",
            "UPDATE red_piece SET end_cp = 60 WHERE verse_id = 40003015; INSERT INTO red_piece VALUES (40003015, 1, 60, 111, 1)".into(),
            &["R3", "T7", "T8"],
            &[],
        ),
        (
            "merge-mid-verse",
            "UPDATE red_piece SET end_cp = (SELECT end_cp FROM red_piece WHERE verse_id = 40009006 AND n = 1) WHERE verse_id = 40009006 AND n = 0;
             DELETE FROM red_piece WHERE verse_id = 40009006 AND n = 1;"
                .into(),
            &["T7", "T8"],
            &[],
        ),
        (
            "merge-at-verse-edge",
            format!("UPDATE red_piece SET q_ord = q_ord - 1 WHERE q_ord >= {edge}"),
            &["T7", "T8"],
            &["R6", "R1", "R3"],
        ),
        ("move-to-next-verse", "UPDATE red_piece SET verse_id = 40003016 WHERE verse_id = 40003015".into(), &["T7"], &["T8"]),
        ("leak-into-next-verse", format!("INSERT INTO red_piece VALUES ({leak_v}, 0, 0, 1, {leak_q})"), &["T7", "T8"], &[]),
        ("empty-piece", "PRAGMA ignore_check_constraints = ON; UPDATE red_piece SET end_cp = start_cp WHERE verse_id = 40003015".into(), &["R1"], &[]),
        ("text-changed-char", "UPDATE verse SET text = replace(text, 'God', 'Gad') WHERE id = 1001001".into(), &["T4"], &[]),
        ("text-dropped-char", "UPDATE verse SET text = substr(text, 2) WHERE id = 1001001".into(), &["T4"], &[]),
        ("text-added-char", "UPDATE verse SET text = text || '.' WHERE id = 1001001".into(), &["T4"], &[]),
        ("mark-shift", "UPDATE mark SET start_cp = start_cp + 1 WHERE verse_id = 1001002 AND n = 0".into(), &["T4"], &[]),
        ("key-changed", "UPDATE word_key SET key = 'H1' WHERE verse_id = 1001001 AND n = 0".into(), &["T4"], &[]),
        ("word-dropped", "DELETE FROM word_key WHERE verse_id = 1001001 AND n = 5; DELETE FROM word WHERE verse_id = 1001001 AND n = 5".into(), &["T4"], &[]),
        ("schema-trigger", "CREATE TRIGGER t AFTER INSERT ON meta BEGIN SELECT 1; END".into(), &["schema"], &[]),
    ];
    for (name, sql, must, must_not) in cases {
        let bytes = mutate(&r.bytes, &sql);
        let f = check_bytes(r, &bytes).expect_err(name);
        for c in must {
            assert!(f.has(c), "{name}: expected {c}; fired {:?}", f.checks());
        }
        for c in must_not {
            assert!(
                !f.has(c),
                "{name}: {c} should not fire; fired {:?}",
                f.checks()
            );
        }
        eprintln!("{name}: {:?}", f.checks());
    }
}

#[test]
fn t11_source_mutations_fail_the_pinned_facts() {
    let Some(r) = real() else { return };
    let src = &r.prep.kjv.bytes;
    let q1 = r.prep.model.qs[0];
    // Drop a span: remove q 1's start and end tags (Matt 3:15).
    let (s, e) = (q1.start_byte as usize, q1.end_byte as usize);
    let start_tag = b"<q who=\"Jesus\" marker=\"\">";
    assert_eq!(&src[s..s + start_tag.len()], start_tag);
    assert_eq!(&src[e - 4..e], b"</q>");
    let mut dropped = Vec::new();
    dropped.extend_from_slice(&src[..s]);
    dropped.extend_from_slice(&src[s + start_tag.len()..e - 4]);
    dropped.extend_from_slice(&src[e..]);
    // Add a span: wrap Matt 3:17's whole text in a q.
    let sid = b"<verse osisID=\"Matt.3.17\" sID=\"Matt.3.17\"/>";
    let eid = b"<verse eID=\"Matt.3.17\"/>";
    let a = find(src, sid) + sid.len();
    let b = find(src, eid);
    let mut added = Vec::new();
    added.extend_from_slice(&src[..a]);
    added.extend_from_slice(start_tag);
    added.extend_from_slice(&src[a..b]);
    added.extend_from_slice(b"</q>");
    added.extend_from_slice(&src[b..]);
    for (name, bytes) in [("drop-q", dropped), ("add-q", added)] {
        let p = &r.prep.profile;
        let m = osis::parse(&bytes, p).unwrap_or_else(|e| panic!("{name}: {e}"));
        let f = validate::validate(&m, p, Some(&r.prep.table)).expect_err(name);
        assert!(f.has("T8"), "{name}: {:?}", f.checks());
        eprintln!("{name}: {:?}", f.checks());
    }
}

fn find(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).position(|w| w == needle).unwrap()
}

// ------------------------------------------------------------ T16, T17

#[test]
fn t16_two_builds_are_identical_and_match_the_committed_hash() {
    let Some(r) = real() else { return };
    let out = common::temp_dir("kjv-real-2");
    let h2 = run::publish_model(
        &out,
        KJV_DEST,
        &r.prep.model,
        &r.prep.profile,
        Some(&r.prep.table),
        &r.meta,
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(h2, r.hash);
    assert_eq!(fs::read(out.join(KJV_DEST)).unwrap(), r.bytes);
    assert_eq!(
        fs::read(out.join("kjv.sqlite.sha256")).unwrap(),
        fs::read(r.out.join("kjv.sqlite.sha256")).unwrap()
    );
    let expected = fs::read_to_string(
        revelation_pipeline::repo_dir().join("pipeline/tests/expected/output-sha256.txt"),
    )
    .unwrap();
    // The same parser the app's build.rs uses to compile the value in.
    let entries = revelation_content::hashfile::parse(&expected).unwrap();
    assert_eq!(
        revelation_content::hashfile::lookup(&entries, KJV_DEST).unwrap(),
        r.hash,
        "built output differs from output-sha256.txt"
    );
    // No build-environment value is stored (DESIGN.md 10.1).
    let keys = revelation_pipeline::db::query_strings(&db(r), "SELECT key FROM meta ORDER BY key")
        .unwrap();
    assert_eq!(
        keys,
        [
            "has_red_letters",
            "red_letter_source",
            "schema_version",
            "translation_abbrev",
            "versification",
            "versification_overrides_sha256"
        ]
    );
}

#[test]
fn t17_check_refuses_a_sidecar_and_a_stale_checksum() {
    let Some(r) = real() else { return };
    let out = common::temp_dir("kjv-real-t17");
    fs::copy(r.out.join(KJV_DEST), out.join(KJV_DEST)).unwrap();
    fs::copy(
        r.out.join("kjv.sqlite.sha256"),
        out.join("kjv.sqlite.sha256"),
    )
    .unwrap();
    assert_eq!(run::check_published(&r.prep, &out).unwrap(), r.hash);
    for side in ["kjv.sqlite-wal", "kjv.sqlite-journal", "kjv.sqlite-shm"] {
        fs::write(out.join(side), b"x").unwrap();
        let f = run::check_published(&r.prep, &out).unwrap_err();
        assert!(f.has("sidecar"), "{side}: {f}");
        fs::remove_file(out.join(side)).unwrap();
    }
    fs::write(
        out.join("kjv.sqlite.sha256"),
        format!("{}  {KJV_DEST}\n", "0".repeat(64)),
    )
    .unwrap();
    let f = run::check_published(&r.prep, &out).unwrap_err();
    assert!(f.has("sha256"), "{f}");
}

// ------------------------------------------------------------ T5 oracle

fn expected(name: &str) -> String {
    fs::read_to_string(
        revelation_pipeline::repo_dir()
            .join("pipeline/tests/expected")
            .join(name),
    )
    .unwrap()
}

fn oracle_check(r: &Real, c: &Connection) -> Result<(), Failure> {
    revelation_pipeline::oracle::compare(
        c,
        &expected("kjv-chapters.tsv"),
        &expected("kjv-q.tsv"),
        &r.prep.kjv.sha256,
    )
}

#[test]
fn t5_database_matches_the_independent_oracle() {
    let Some(r) = real() else { return };
    oracle_check(r, &db(r)).unwrap_or_else(|f| panic!("{f}"));
    // The oracle files were generated by the committed generator version.
    let gen_path =
        revelation_pipeline::repo_dir().join("docs/pipeline/survey-scripts/oracle_kjv.py");
    let gen_sha = revelation_pipeline::sources::sha256_hex(&fs::read(gen_path).unwrap());
    for f in ["kjv-chapters.tsv", "kjv-q.tsv"] {
        assert_eq!(
            revelation_pipeline::oracle::header_value(&expected(f), "generator").as_deref(),
            Some(gen_sha.as_str()),
            "{f}: stale generator hash"
        );
    }
    // Byte offsets in kjv-q.tsv equal the Rust parser's.
    let q = expected("kjv-q.tsv");
    let rows: Vec<&str> = q.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(rows.len(), r.prep.model.qs.len());
    for (line, qs) in rows.iter().zip(&r.prep.model.qs) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            (f[0], f[2], f[3]),
            (
                qs.q_ord.to_string().as_str(),
                qs.start_byte.to_string().as_str(),
                qs.end_byte.to_string().as_str()
            )
        );
    }
}

#[test]
fn t5_catches_what_the_totals_cannot() {
    let Some(r) = real() else { return };
    // Merge plus split in one book: pieces and red code points per book are
    // unchanged, but the q grouping is not. Matt 3:15 (q 1) and Matt 4:4 (q 2)
    // swap q_ord: T8 totals stay equal, T5 (and T7) catch it.
    let swap = "PRAGMA ignore_check_constraints = ON; UPDATE red_piece SET q_ord = 0 WHERE q_ord = 1;
                UPDATE red_piece SET q_ord = 1 WHERE q_ord = 2; UPDATE red_piece SET q_ord = 2 WHERE q_ord = 0;";
    let bytes = mutate(&r.bytes, swap);
    let c = open_bytes(&bytes).unwrap();
    let f = oracle_check(r, &c).unwrap_err();
    assert!(f.has("T5"), "{f}");
    assert!(
        f.first().msg.contains("q_ord 1") || f.to_string().contains("Matt.3"),
        "{f}"
    );
    let f = check::check(
        &c,
        &r.prep.model,
        &r.prep.profile,
        Some(&r.prep.table),
        &r.meta,
    )
    .unwrap_err();
    assert!(f.has("T7") && !f.has("T8"), "{:?}", f.checks());

    // Source-side text change (no pinned fact covers it): only the oracle sees it.
    let src = &r.prep.kjv.bytes;
    let at = find(src, b">In the beginning</w>") + 1;
    let mut changed = src.clone();
    changed[at] = b'I';
    changed[at + 1] = b'N';
    let p = &r.prep.profile;
    let m = osis::parse(&changed, p).unwrap();
    validate::validate(&m, p, Some(&r.prep.table)).unwrap();
    let mut mem = Connection::open_in_memory().unwrap();
    revelation_pipeline::db::write(&mut mem, &m, &r.prep.meta()).unwrap();
    let bytes = mem.serialize("main").unwrap().to_vec();
    let c = open_bytes(&bytes).unwrap();
    let f = oracle_check(r, &c).unwrap_err();
    assert!(
        f.to_string()
            .contains("chapter Gen.1 differs in [\"text_sha256\"]"),
        "{f}"
    );
}

// ------------------------------------------------------------ T18, T19 golden

fn read_repo(rel: &str) -> String {
    fs::read_to_string(revelation_pipeline::repo_dir().join(rel)).unwrap()
}

#[test]
fn t18_golden_tsv_equals_the_scholars_yaml() {
    // No real sources needed: the two files must not drift.
    use revelation_pipeline::golden;
    let yaml =
        golden::scholar_rows(&read_repo("docs/scholar/red-letter-golden-samples.md")).unwrap();
    let tsv = golden::tsv_rows(&read_repo("pipeline/tests/golden/red-letter-source.tsv")).unwrap();
    assert_eq!(yaml.len(), 100);
    let strip = |rows: &[golden::Row]| -> Vec<golden::Row> {
        rows.iter()
            .map(|r| {
                let mut r = r.clone();
                r.fields.remove("basis");
                r
            })
            .collect()
    };
    assert_eq!(strip(&tsv), strip(&yaml));
    assert!(
        yaml.iter()
            .all(|r| r.fields.get("human").map(String::as_str) == Some("pending"))
    );
}

#[test]
fn t18_built_database_matches_source_expected() {
    let Some(r) = real() else { return };
    use revelation_pipeline::golden;
    let c = db(r);
    let rows = golden::tsv_rows(&read_repo("pipeline/tests/golden/red-letter-source.tsv")).unwrap();
    let mut failures = Vec::new();
    for row in &rows {
        let (id, osis) = golden::usfm_ref(&row.usfm).unwrap();
        let got = golden::derived(&c, id).unwrap();
        let d = golden::diff(row, &got);
        if !d.is_empty() {
            failures.push(format!("{} ({osis}): {}", row.usfm, d.join("; ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} golden rows differ:\n{}",
        failures.len(),
        rows.len(),
        failures.join("\n")
    );
}

#[test]
fn t19_golden_verse_text() {
    let Some(r) = real() else { return };
    let c = db(r);
    let tsv = read_repo("pipeline/tests/golden/verse-text.tsv");
    let mut n = 0;
    let mut failures = Vec::new();
    for line in tsv.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let f: Vec<&str> = line.splitn(4, '\t').collect();
        let (id, osis) = revelation_pipeline::golden::usfm_ref(f[0]).unwrap();
        assert_eq!(osis, f[1]);
        let text: String = c
            .query_row("SELECT text FROM verse WHERE id = ?1", [id], |x| x.get(0))
            .unwrap();
        if text != f[3] || text.chars().count().to_string() != f[2] {
            use revelation_pipeline::error::show_ws;
            failures.push(format!(
                "{}: expected {:?}\n   database {:?}",
                f[0],
                show_ws(f[3]),
                show_ws(&text)
            ));
        }
        n += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(n, 105);
    for must in ["GEN.1.1", "PSA.23.1", "PSA.23.6", "JHN.3.16"] {
        assert!(
            tsv.lines().any(|l| l.starts_with(&format!("{must}\t"))),
            "{must}"
        );
    }
}

#[test]
fn t18_golden_comparison_bites() {
    let Some(r) = real() else { return };
    use revelation_pipeline::golden;
    let rows = golden::tsv_rows(&read_repo("pipeline/tests/golden/red-letter-source.tsv")).unwrap();
    let row = rows.iter().find(|x| x.usfm == "MAT.3.15").unwrap();
    // Move the start of Matt 3:15's piece one word later ("it" instead of "Suffer").
    let bytes = mutate(
        &r.bytes,
        "UPDATE red_piece SET start_cp = start_cp + 7 WHERE verse_id = 40003015",
    );
    let c = open_bytes(&bytes).unwrap();
    let d = golden::diff(row, &golden::derived(&c, 40003015).unwrap());
    assert_eq!(d, vec!["first: Suffer → it".to_string()]);
    // Drop the second piece of Matt 9:6: spans, end and span2 differ.
    let bytes = mutate(
        &r.bytes,
        "DELETE FROM red_piece WHERE verse_id = 40009006 AND n = 1",
    );
    let c = open_bytes(&bytes).unwrap();
    let row = rows.iter().find(|x| x.usfm == "MAT.9.6").unwrap();
    let d = golden::diff(row, &golden::derived(&c, 40009006).unwrap());
    assert!(d.iter().any(|x| x.starts_with("spans: 2 → 1")), "{d:?}");
}

// ------------------------------------------------------------ book main titles

/// `book.main_title` round trip against an independent byte-level reading of
/// the raw file (no quick-xml): each `<title type="main">…</title>` in
/// document order, minus every tag, must equal the stored title of the
/// book at that position, and the stored text must be byte-identical.
#[test]
fn main_titles_equal_a_naive_byte_reading_of_the_source() {
    let Some(r) = real() else { return };
    let src = &r.prep.kjv.bytes;
    let open = b"<title type=\"main\">";
    let mut naive: Vec<String> = Vec::new();
    let mut at = 0;
    while let Some(i) = src[at..].windows(open.len()).position(|w| w == open) {
        let start = at + i + open.len();
        let len = src[start..]
            .windows(8)
            .position(|w| w == b"</title>")
            .unwrap();
        let inner = &src[start..start + len];
        let mut text = Vec::new();
        let mut in_tag = false;
        for &b in inner {
            match b {
                b'<' => in_tag = true,
                b'>' => in_tag = false,
                _ if !in_tag => text.push(b),
                _ => {}
            }
        }
        naive.push(String::from_utf8(text).unwrap());
        at = start + len;
    }
    assert_eq!(naive.len(), 66);
    let c = db(r);
    let mut st = c
        .prepare("SELECT osis, main_title FROM book ORDER BY ord")
        .unwrap();
    let got: Vec<(String, String)> = st
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(got.len(), 66);
    for ((osis, title), want) in got.iter().zip(&naive) {
        assert_eq!(title, want, "{osis}");
    }
    assert_eq!(got[0].1, "THE FIRST BOOK OF MOSES CALLED GENESIS");
    assert_eq!(got[39].1, "THE GOSPEL ACCORDING TO ST. MATTHEW");
    assert_eq!(got[65].1, "THE REVELATION OF ST. JOHN THE DIVINE");
    // Five titles have <abbr expansion="Saint">ST.</abbr>; the attribute is
    // markup and is not stored.
    let abbr = src
        .windows(b"<abbr expansion=\"Saint\">ST.</abbr>".len())
        .filter(|w| *w == b"<abbr expansion=\"Saint\">ST.</abbr>")
        .count();
    assert_eq!(abbr, 5);
    assert_eq!(got.iter().filter(|g| g.1.contains("ST.")).count(), 5);
}
