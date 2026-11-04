//! kjv-search.sqlite on the real KJV (DESIGN.md 18): built from the
//! published kjv.sqlite and checked (T25), deterministic and equal to the
//! committed hash (T16), opened the app's way, and the query semantics
//! against an independent brute-force oracle over all 31,102 verses (counts,
//! id sets and order), including FTS5 syntax typed as a query. Database
//! mutations assert which check fires.

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use revelation_content::search::{
    PAGE, Query, Scope, match_spans, parse_query, ranked, require_search_index, search,
};
use revelation_pipeline::check::{check_search, search_norm_oracle};
use revelation_pipeline::open::{open_bytes, open_verified};
use revelation_pipeline::run::{self, KJV_DEST};
use revelation_pipeline::search::{SEARCH_DEST, SearchInput};
use rusqlite::Connection;

struct Real {
    input: SearchInput,
    out: PathBuf,
    hash: String,
    bytes: Vec<u8>,
}

fn real() -> Option<&'static Real> {
    static R: OnceLock<Option<Real>> = OnceLock::new();
    R.get_or_init(|| {
        let raw = common::sources_dir()?;
        let prep = run::prepare_kjv(&raw).unwrap_or_else(|e| panic!("{e}"));
        let out = common::temp_dir("search-real");
        let kjv_hash = run::publish_model(
            &out,
            KJV_DEST,
            &prep.model,
            &prep.profile,
            Some(&prep.table),
            &prep.meta(),
            &mut |_| Ok(()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let input = run::load_search_input(&out, &kjv_hash).unwrap_or_else(|e| panic!("{e}"));
        let hash =
            run::publish_search(&out, &input, &mut |_| Ok(())).unwrap_or_else(|e| panic!("{e}"));
        let bytes = fs::read(out.join(SEARCH_DEST)).unwrap();
        Some(Real {
            input,
            out,
            hash,
            bytes,
        })
    })
    .as_ref()
}

/// The index opened the app's way: compiled-in hash, content identity,
/// search identity against the kjv.sqlite hash.
fn db(r: &Real) -> Connection {
    let (c, _) = open_verified(&r.out.join(SEARCH_DEST), Some(&r.hash)).unwrap();
    revelation_content::require_content_identity(&c).unwrap();
    require_search_index(&c, &r.input.kjv_sha256).unwrap();
    c
}

// ------------------------------------------------------------------ oracle

/// The oracle's own words of a stored verse: ASCII letters and digits
/// lower-cased, Æ as "ae", every other character a break. Written without
/// `revelation_content::search` or the check stage's normaliser.
fn oracle_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut w = String::new();
    for b in text.replace('\u{00C6}', "AE").bytes() {
        if b.is_ascii_alphanumeric() {
            w.push(b.to_ascii_lowercase() as char);
        } else if !w.is_empty() {
            words.push(std::mem::take(&mut w));
        }
    }
    if !w.is_empty() {
        words.push(w);
    }
    words
}

/// (all-words set, exact-phrase set) of `want` in `[lo, hi]`, by scanning
/// every verse.
fn oracle(r: &Real, want: &[String], lo: i64, hi: i64) -> (Vec<i64>, Vec<i64>) {
    let mut all = Vec::new();
    let mut phrase = Vec::new();
    for (id, text) in &r.input.verses {
        if *id < lo || *id > hi {
            continue;
        }
        let words = oracle_words(text);
        if want.iter().all(|w| words.contains(w)) {
            all.push(*id);
        }
        if words.windows(want.len()).any(|win| win == want) {
            phrase.push(*id);
        }
    }
    (all, phrase)
}

const QUERIES: &[&str] = &[
    "jesus wept",
    // W2P-B1: repeats and phrases longer than FTS_PHRASE_WORDS (two-step match).
    "verily verily",
    "the the",
    "holy holy holy",
    "\"and it came to pass when",
    "\"in the beginning god created the heaven and the earth",
    "\"the lord is my shepherd i shall not want",
    "\"and the lord said unto moses",
    "\"of the of the",
    "and it came to pass when the lord",
    "\"holy holy holy is the lord of hosts",
    // W2P-B2: a pasted BOM or zero-width space, and other opening quotes.
    "\u{FEFF}\"jesus wept",
    "\u{200B} \"jesus wept",
    "\u{201E}jesus wept",
    "\u{00AB}jesus wept\u{00BB}",
    "\u{2039}jesus wept",
    "\"jesus wept\"",
    "wept",
    "swept",
    "weep",
    "the",
    "and",
    "lord",
    "LORD",
    "god",
    "in the beginning",
    "\"in the beginning",
    "In the beginning was the Word",
    "love one another",
    "'love one another'",
    "living water",
    "the lord is my shepherd",
    "\u{201C}The LORD is my shepherd\u{201D}",
    "holy holy holy",
    "\"holy, holy, holy",
    "aenon",
    "Ænon",
    "beer sheba",
    "\"beer-sheba",
    "beersheba",
    "lord's",
    "\"the lord's",
    "\u{2018}the LORD\u{2019}s house",
    "selah",
    "verily verily",
    "\"verily, verily, I say unto you",
    "faith hope charity",
    "a",
    "o",
    "jesus christ",
    "\"christ jesus",
    "son of man",
    "\"son of man",
    "lamb of god",
    "the the",
    "\"the the",
    "god so loved the world",
    "\"for god so loved the world",
    "alpha omega",
    "\"i am alpha and omega",
    "melchisedec",
    "mahershalalhashbaz",
    "jesus 666",
    "café",
];

/// Typed by a reader as a query: FTS5 syntax, SQL, odd characters.
const HOSTILE: &[&str] = &[
    "\"",
    "\"\"",
    "\"\"\"",
    "*",
    "jesus*",
    "je*",
    "NEAR(jesus wept)",
    "NEAR(jesus wept, 2)",
    "jesus NEAR wept",
    "jesus NEAR/2 wept",
    "jesus OR wept",
    "jesus AND wept",
    "jesus NOT wept",
    "AND",
    "OR",
    "NOT",
    "NEAR",
    "-jesus",
    "jesus -wept",
    "norm:jesus",
    "verse_fts:jesus",
    "{norm}:jesus",
    "- norm : jesus",
    "rowid:43011035",
    "^jesus",
    "^",
    "(jesus)",
    "((jesus",
    "jesus))",
    "jesus + wept",
    "\"jesus\" OR \"wept\"",
    "\"jesus\" \"wept\"",
    ":",
    "-",
    "+",
    "(",
    ")",
    "'",
    "''",
    "\\",
    "\0",
    "jesus\u{0}wept",
    "jesus\"wept",
    "'; DROP TABLE meta; --",
    "1=1",
    "%",
    "_",
    "[a-z]",
    "{ }",
    "~",
    "`",
    "@#$&|<>=?!",
    "\u{FEFF}jesus",
    "\u{200B}",
    "jesus\u{00AD}wept",
    "\u{1F600}",
    "jesus \u{1F600}",
    "יהוה",
    "θεος",
    "\u{0301}",
    "e\u{0301}",
];

fn check_query(r: &Real, c: &Connection, q: &str, scope: Scope) {
    let page = search(c, q, scope, 0, PAGE).unwrap_or_else(|e| panic!("{q:?}: {e}"));
    let Some(query) = &page.query else {
        assert_eq!(page.total, 0, "{q:?}");
        return;
    };
    let (lo, hi) = scope.id_range().unwrap();
    let rk = ranked(c, query, scope).unwrap();
    assert_eq!(rk.ids.len(), page.total, "{q:?}");
    assert_eq!(rk.exact_total, page.exact_total, "{q:?}");
    let (all, phrase) = if query.matchable() {
        oracle(r, query.words(), lo, hi)
    } else {
        (Vec::new(), Vec::new())
    };
    let want = match query {
        Query::Phrase(_) => phrase.clone(),
        Query::AllWords(_) => all.clone(),
    };
    let (exact, rest) = rk.ids.split_at(rk.exact_total);
    assert_eq!(exact, &phrase[..], "{q:?} {scope:?}: exact-phrase part");
    assert!(
        rest.windows(2).all(|w| w[0] < w[1]),
        "{q:?}: rest in canonical order"
    );
    let got: BTreeSet<i64> = rk.ids.iter().copied().collect();
    assert_eq!(got.len(), rk.ids.len(), "{q:?}: an id twice");
    assert_eq!(got, want.iter().copied().collect(), "{q:?} {scope:?}");
    let first: Vec<i64> = rk.ids.iter().take(PAGE).copied().collect();
    assert_eq!(
        page.hits.iter().map(|h| h.verse_id).collect::<Vec<_>>(),
        first,
        "{q:?}: first page"
    );
}

// ------------------------------------------------------------------ tests

#[test]
fn t25_index_built_checked_and_deterministic() {
    let Some(r) = real() else { return };
    assert_eq!(r.input.verses.len(), 31_102);
    // T16: a second build into another directory gives the same bytes.
    let out = common::temp_dir("search-real-2");
    fs::copy(r.out.join(KJV_DEST), out.join(KJV_DEST)).unwrap();
    let input = run::load_search_input(&out, &r.input.kjv_sha256).unwrap();
    let h2 = run::publish_search(&out, &input, &mut |_| Ok(())).unwrap();
    assert_eq!(h2, r.hash);
    assert_eq!(fs::read(out.join(SEARCH_DEST)).unwrap(), r.bytes);
    // The committed, owner-reviewed value (the app compiles it in).
    let expected = fs::read_to_string(
        revelation_pipeline::repo_dir().join("pipeline/tests/expected/output-sha256.txt"),
    )
    .unwrap();
    let entries = revelation_content::hashfile::parse(&expected).unwrap();
    assert_eq!(
        revelation_content::hashfile::lookup(&entries, SEARCH_DEST).unwrap(),
        r.hash,
        "built kjv-search.sqlite differs from output-sha256.txt"
    );
    assert_eq!(
        revelation_content::hashfile::lookup(&entries, KJV_DEST).unwrap(),
        r.input.kjv_sha256,
    );
    // The check subcommand's path.
    assert_eq!(
        run::check_published_search(&r.out, &r.input.kjv_sha256).unwrap(),
        r.hash
    );
    // The app's identity checks; a different kjv.sqlite hash is refused.
    let c = db(r);
    assert!(require_search_index(&c, &"0".repeat(64)).is_err());
    let keys = revelation_pipeline::db::query_strings(
        &c,
        "SELECT key || '=' || value FROM meta ORDER BY key",
    )
    .unwrap();
    assert_eq!(
        keys,
        [
            "schema_version=1".to_string(),
            "search_built_from=kjv.sqlite".to_string(),
            format!("search_built_from_sha256={}", r.input.kjv_sha256),
            "search_normalisation=1".to_string(),
            "translation_abbrev=KJV".to_string(),
        ]
    );
    let src = revelation_pipeline::db::query_strings(&c, "SELECT id FROM source").unwrap();
    assert_eq!(src, ["BIB-KJV-CW31"]);
}

#[test]
fn every_verse_once_and_normalised_as_documented() {
    let Some(r) = real() else { return };
    let c = db(r);
    let mut st = c
        .prepare("SELECT rowid, norm FROM verse_fts ORDER BY rowid")
        .unwrap();
    let rows: Vec<(i64, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(rows.len(), 31_102);
    for ((id, text), (rid, norm)) in r.input.verses.iter().zip(&rows) {
        assert_eq!(id, rid);
        assert_eq!(*norm, oracle_words(text).join(" "), "{id}");
        assert_eq!(Some(norm.clone()), search_norm_oracle(text), "{id}");
    }
    // The characters of the KJV that become separators: ASCII punctuation
    // and space, and exactly ’ – — (Æ folds to "ae").
    let non_ascii: BTreeSet<char> = r
        .input
        .verses
        .iter()
        .flat_map(|(_, t)| t.chars())
        .filter(|c| !c.is_ascii())
        .collect();
    assert_eq!(
        non_ascii,
        ['\u{00C6}', '\u{2013}', '\u{2014}', '\u{2019}']
            .into_iter()
            .collect()
    );
}

#[test]
fn jesus_wept_is_john_11_35_first() {
    let Some(r) = real() else { return };
    let c = db(r);
    for q in [
        "jesus wept",
        "Jesus wept.",
        "JESUS WEPT",
        "\"jesus wept\"",
        "“Jesus wept”",
    ] {
        let p = search(&c, q, Scope::Bible, 0, PAGE).unwrap();
        assert_eq!(p.hits[0].verse_id, 43_011_035, "{q}");
        assert!(p.hits[0].exact);
        assert_eq!(p.exact_total, 1, "{q}");
    }
    let (_, text) = r.input.verses.iter().find(|v| v.0 == 43_011_035).unwrap();
    assert_eq!(text, "Jesus wept.");
    let q = parse_query("jesus wept").unwrap().unwrap();
    assert_eq!(match_spans(text, &q), [(0, 5), (6, 10)]);
    let q = parse_query("\"jesus wept").unwrap().unwrap();
    assert_eq!(match_spans(text, &q), [(0, 10)]);
}

#[test]
fn oracle_agrees_for_every_query_and_scope() {
    let Some(r) = real() else { return };
    let c = db(r);
    let ids: BTreeSet<i64> = r.input.verses.iter().map(|v| v.0).collect();
    let scopes = [
        Scope::Bible,
        Scope::OldTestament,
        Scope::NewTestament,
        Scope::Book(19),
        Scope::Book(43),
        Scope::Book(66),
    ];
    let mut checked = 0;
    for q in QUERIES.iter().chain(HOSTILE) {
        for s in scopes {
            check_query(r, &c, q, s);
            checked += 1;
        }
        // Every id returned is a kjv.sqlite verse, and its emphasis maps
        // onto the stored text.
        if let Some(query) = parse_query(q).unwrap() {
            let rk = ranked(&c, &query, Scope::Bible).unwrap();
            for id in &rk.ids {
                assert!(ids.contains(id), "{q:?}: {id} is not a verse");
            }
            for id in rk.ids.iter().take(200) {
                let text = &r.input.verses.iter().find(|v| v.0 == *id).unwrap().1;
                assert!(
                    !match_spans(text, &query).is_empty(),
                    "{q:?}: no span in {id}"
                );
            }
        }
    }
    eprintln!("oracle: {checked} query/scope pairs agree");
}

#[test]
fn whole_words_and_true_totals() {
    let Some(r) = real() else { return };
    let c = db(r);
    let n = |q: &str| search(&c, q, Scope::Bible, 0, PAGE).unwrap().total;
    // "wept" never returns a verse for its "swept".
    let wept = ranked(&c, &parse_query("wept").unwrap().unwrap(), Scope::Bible).unwrap();
    for id in &wept.ids {
        let text = &r.input.verses.iter().find(|v| v.0 == *id).unwrap().1;
        assert!(oracle_words(text).iter().any(|w| w == "wept"), "{id}");
    }
    let swept = ranked(&c, &parse_query("swept").unwrap().unwrap(), Scope::Bible).unwrap();
    assert!(!swept.ids.is_empty());
    assert!(swept.ids.iter().all(|id| !wept.ids.contains(id) || {
        let text = &r.input.verses.iter().find(|v| v.0 == *id).unwrap().1;
        oracle_words(text).iter().any(|w| w == "wept")
    }));
    assert_eq!(n("weep"), oracle(r, &["weep".into()], 0, i64::MAX).0.len());
    // Totals are true totals, not the page size.
    assert!(n("the") > 20_000, "{}", n("the"));
    let all = search(&c, "the", Scope::Bible, 0, PAGE).unwrap();
    assert_eq!(all.hits.len(), PAGE);
    let last = search(&c, "the", Scope::Bible, all.total - 3, PAGE).unwrap();
    assert_eq!(last.hits.len(), 3);
    assert_eq!(last.offset, all.total - 3);
    // OT + NT = Bible.
    for q in ["lord", "and", "jesus", "love one another"] {
        let s = |sc| search(&c, q, sc, 0, PAGE).unwrap().total;
        assert_eq!(
            s(Scope::OldTestament) + s(Scope::NewTestament),
            s(Scope::Bible),
            "{q}"
        );
    }
    assert_eq!(
        search(&c, "jesus", Scope::OldTestament, 0, PAGE)
            .unwrap()
            .total,
        0
    );
}

#[test]
fn timings_on_the_real_index() {
    let Some(r) = real() else { return };
    let c = db(r);
    for q in [
        "the",
        "and the",
        "jesus wept",
        "\"in the beginning",
        "lord god of israel",
    ] {
        let t = Instant::now();
        let p = search(&c, q, Scope::Bible, 0, PAGE).unwrap();
        eprintln!("search {q:?}: {} results in {:?}", p.total, t.elapsed());
    }
    // W2P-B1: repeated common words. 50 x "the" must stay under 20 ms in a
    // release build (debug builds only report the time).
    let worst = ["the ".repeat(50), "a ".repeat(100), "of the ".repeat(28)];
    for q in &worst {
        let p0 = search(&c, q, Scope::Bible, 0, PAGE).unwrap();
        let t = Instant::now();
        let p = search(&c, q, Scope::Bible, 0, PAGE).unwrap();
        let took = t.elapsed();
        eprintln!("search {:?}…: {} results in {took:?}", &q[..12], p.total);
        assert_eq!(p, p0);
        if !cfg!(debug_assertions) && q.starts_with("the the") {
            assert!(took.as_millis() < 20, "50 x the took {took:?}");
        }
    }
    // De-duplicating keeps the results: "the the" has the same all-words
    // set as "the", and its exact-phrase matches are ranked first.
    let rk = |q: &str| ranked(&c, &parse_query(q).unwrap().unwrap(), Scope::Bible).unwrap();
    let (one, two) = (rk("the"), rk("the the"));
    let mut a = one.ids.clone();
    let mut b = two.ids.clone();
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b);
    assert!(two.exact_total < two.ids.len());
    // A pasted BOM before a quote still makes a phrase (W2P-B2).
    let p = search(&c, "\u{FEFF}\"jesus wept", Scope::Bible, 0, PAGE).unwrap();
    assert!(matches!(p.query, Some(Query::Phrase(_))));
    assert_eq!(p.total, 1);
    // W2P-B4: limit and offset are clamped inside `search`.
    let big = search(&c, "the", Scope::Bible, 0, usize::MAX).unwrap();
    assert_eq!(big.hits.len(), revelation_content::search::MAX_PAGE);
    let past = search(&c, "the", Scope::Bible, usize::MAX, PAGE).unwrap();
    assert_eq!((past.offset, past.hits.len()), (past.total, 0));
    let vv = search(&c, "verily verily", Scope::Bible, 0, PAGE).unwrap();
    assert!(vv.exact_total > 0 && vv.exact_total < vv.total);
    assert_eq!(vv.hits[0].verse_id, 43_001_051); // John 1:51 "Verily, verily"
}

// ------------------------------------------------------------------ check stage

/// A copy of the index with `sql` applied (a normal connection, so shadow
/// tables can be written), serialised back to bytes.
fn mutate(bytes: &[u8], sql: &str) -> Vec<u8> {
    let mut c = Connection::open_in_memory().unwrap();
    c.deserialize_read_exact("main", bytes, bytes.len(), false)
        .unwrap();
    c.execute_batch(sql).unwrap();
    c.serialize("main").unwrap().to_vec()
}

fn checks_fired(r: &Real, bytes: &[u8]) -> Vec<&'static str> {
    let c = open_bytes(bytes).unwrap();
    match check_search(&c, &r.input, &r.input.meta()) {
        Ok(()) => Vec::new(),
        Err(f) => f.checks(),
    }
}

#[test]
fn mutations_are_caught_by_the_check_stage() {
    let Some(r) = real() else { return };
    assert!(checks_fired(r, &r.bytes).is_empty());
    let cases: &[(&str, &str)] = &[
        // Content changed behind the index's back (shadow table).
        (
            "UPDATE verse_fts_content SET c0 = 'jesus wept' WHERE id = 1001001",
            "integrity",
        ),
        (
            "UPDATE verse_fts_content SET c0 = 'jesus wept' WHERE id = 1001001",
            "search-norm",
        ),
        // A consistent change through FTS5 itself.
        (
            "UPDATE verse_fts SET norm = 'in the beginning' WHERE rowid = 1001001",
            "search-norm",
        ),
        (
            "DELETE FROM verse_fts WHERE rowid = 43011035",
            "search-count",
        ),
        (
            "INSERT INTO verse_fts (rowid, norm) VALUES (43011099, 'extra')",
            "search-count",
        ),
        (
            "INSERT INTO verse_fts (rowid, norm) VALUES (67001001, 'extra')",
            "search-id",
        ),
        (
            "UPDATE verse_fts SET norm = 'Jesus wept' WHERE rowid = 43011035",
            "search-norm",
        ),
        (
            "UPDATE meta SET value = 'x' WHERE key = 'search_built_from_sha256'",
            "meta",
        ),
        ("CREATE TABLE extra (a)", "schema"),
        // The index itself damaged: every segment's leaf data cleared.
        (
            "UPDATE verse_fts_data SET block = zeroblob(length(block)) WHERE id > 10",
            "integrity",
        ),
    ];
    for (sql, check) in cases {
        let fired = checks_fired(r, &mutate(&r.bytes, sql));
        assert!(
            fired.contains(check),
            "{sql}: expected {check}, got {fired:?}"
        );
    }
}

#[test]
fn a_verse_the_normalisation_does_not_cover_stops_the_build() {
    let Some(r) = real() else { return };
    for (text, check) in [("caf\u{00E9}", "search-norm"), ("\u{2014}", "search-norm")] {
        let mut input = r.input.clone();
        input.verses[0].1 = text.to_string();
        let out = common::temp_dir("search-bad");
        let err = run::publish_search(&out, &input, &mut |_| Ok(())).unwrap_err();
        assert!(err.has(check), "{text:?}: {err}");
        assert!(
            common::list(&out).is_empty(),
            "{text:?}: {:?}",
            common::list(&out)
        );
    }
}
