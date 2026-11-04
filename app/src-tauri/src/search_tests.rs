//! Search against the real pipeline output (`pipeline/out/kjv-search.sqlite`
//! and `kjv.sqlite`), opened only through `open_verified` with the compiled-in
//! hashes; plus the golden `tests/golden/kjv-search.json` that the vitest
//! render test reads.
//!
//! Regenerate the golden (in app/src-tauri):
//! `cargo test -- --ignored regenerate_search_golden`

use crate::commands::{self, AppState, KJV_FILE, LEX_FILE};
use crate::content::{self, MAX_CONTENT_BYTES};
use crate::expected;
use crate::search::{SEARCH_FILE, SearchHit, SearchResult, SearchScope};
use revelation_content::search as rs;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

fn state(content: Result<commands::Content, String>) -> AppState {
    AppState {
        content: Mutex::new(content),
        user: Mutex::new(Err("not used".into())),
    }
}

fn real() -> AppState {
    state(commands::load_content_from(
        &commands::content_dir().unwrap(),
        expected::KJV_SQLITE,
        expected::CONTENT_SQLITE,
    ))
}

fn raw_kjv() -> Connection {
    content::open_content(
        &commands::content_dir().unwrap().join(KJV_FILE),
        expected::KJV_SQLITE,
        MAX_CONTENT_BYTES,
    )
    .unwrap()
}

fn ok(r: SearchResult) -> (usize, usize, bool, Vec<SearchHit>) {
    match r {
        SearchResult::Ok {
            total,
            exact_total,
            phrase,
            hits,
            ..
        } => (total, exact_total, phrase, hits),
        SearchResult::Unavailable { reason } => panic!("unavailable: {reason}"),
    }
}

fn find(
    s: &AppState,
    q: &str,
    scope: SearchScope,
    offset: u32,
) -> (usize, usize, bool, Vec<SearchHit>) {
    ok(commands::run_search(s, "KJV", q, scope, offset, 20).unwrap())
}

/// Every hit: the text is the stored `verse.text`, the runs concatenate to
/// it, red is exactly the reader's red, and the emphasis is `match_spans`.
fn check_hits(raw: &Connection, q: &str, hits: &[SearchHit]) {
    let query = rs::parse_query(q).unwrap().unwrap();
    let s = real();
    for h in hits {
        let stored: String = raw
            .query_row("SELECT text FROM verse WHERE id = ?1", [h.id], |r| r.get(0))
            .unwrap();
        assert_eq!(h.text, stored, "{}", h.reference);
        let joined: String = h.runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, stored, "{}", h.reference);
        assert!(h.emphasised, "{}", h.reference);
        // emphasis == match_spans, as code-point ranges of the runs
        let mut got: Vec<(u32, u32)> = Vec::new();
        let mut pos = 0u32;
        for r in &h.runs {
            let n = r.text.chars().count() as u32;
            if r.hit {
                match got.last_mut() {
                    Some(l) if l.1 == pos => l.1 = pos + n,
                    _ => got.push((pos, pos + n)),
                }
            }
            pos += n;
        }
        assert_eq!(got, rs::match_spans(&stored, &query), "{}", h.reference);
        // red and its pieces are exactly the reader's
        let ch = commands::with_bible(&s, "KJV", |b| b.chapter(h.book, h.chapter)).unwrap();
        let v = ch.verses.iter().find(|v| v.v == h.verse).unwrap();
        let red_of = |runs: Vec<(String, bool, Option<u32>, bool, bool)>| -> Vec<(char, bool, Option<u32>, bool, bool)> {
            runs.into_iter()
                .flat_map(|(t, red, rp, add, nd)| t.chars().map(move |c| (c, red, rp, add, nd)).collect::<Vec<_>>())
                .collect()
        };
        assert_eq!(
            red_of(
                h.runs
                    .iter()
                    .map(|r| (r.text.clone(), r.red, r.rp, r.add, r.nd))
                    .collect()
            ),
            red_of(
                v.runs
                    .iter()
                    .map(|r| (r.text.clone(), r.red, r.rp, r.add, r.nd))
                    .collect()
            ),
            "{}",
            h.reference
        );
        assert_eq!(
            h.reference.split(' ').next_back().unwrap(),
            format!("{}:{}", h.chapter, h.verse)
        );
    }
}

#[test]
fn jesus_wept_is_john_11_35_with_stored_text_and_emphasis() {
    let s = real();
    let (total, exact, phrase, hits) = find(&s, "jesus wept", SearchScope::Bible, 0);
    // John 11:35 is the one exact phrase; two more verses have both words
    assert_eq!((total, exact, phrase), (3, 1, false));
    assert_eq!(hits[0].reference, "John 11:35");
    assert_eq!(hits[0].id, 43_011_035);
    assert_eq!(hits[0].text, "Jesus wept.");
    check_hits(&raw_kjv(), "jesus wept", &hits);
    // quoted: a phrase
    let (total, _, phrase, _) = find(&s, "\u{201C}Jesus wept", SearchScope::Bible, 0);
    assert_eq!((total, phrase), (1, true));
}

#[test]
fn pages_come_from_the_cached_list_with_true_totals() {
    let s = real();
    let raw = raw_kjv();
    for q in [
        "love one another",
        "\"verily, verily",
        "lord god",
        "living water",
        "holy",
    ] {
        let (total, exact, _, first) = find(&s, q, SearchScope::Bible, 0);
        assert!(total > 0, "{q}");
        assert_eq!(first.len(), total.min(20), "{q}");
        check_hits(&raw, q, &first);
        // ranked order: exact matches first, then canonical order in each group
        let all = rs::ranked(
            &index_conn(),
            &rs::parse_query(q).unwrap().unwrap(),
            rs::Scope::Bible,
        )
        .unwrap();
        assert_eq!(all.ids.len(), total, "{q}");
        assert_eq!(all.exact_total, exact, "{q}");
        let ids: Vec<i64> = first.iter().map(|h| h.id).collect();
        assert_eq!(ids, all.ids[..first.len()], "{q}");
        assert!(
            first
                .iter()
                .enumerate()
                .all(|(i, h)| h.exact == (i < exact)),
            "{q}"
        );
        if total > 20 {
            let (t2, _, _, second) = find(&s, q, SearchScope::Bible, 20);
            assert_eq!(t2, total);
            let ids: Vec<i64> = second.iter().map(|h| h.id).collect();
            assert_eq!(ids, all.ids[20..(40.min(total))], "{q}");
            check_hits(&raw, q, &second);
        }
        // OT + NT = Bible
        let (ot, ..) = find(&s, q, SearchScope::OldTestament, 0);
        let (nt, ..) = find(&s, q, SearchScope::NewTestament, 0);
        assert_eq!(ot + nt, total, "{q}");
    }
    // a book scope; past the end: no hits, same total
    let (t, _, _, hits) = find(&s, "love", SearchScope::Book(62), 0);
    assert!(t > 0 && hits.iter().all(|h| h.book == 62));
    let (t2, _, _, none) = find(&s, "love", SearchScope::Book(62), 40_000);
    assert_eq!((t2, none.len()), (t, 0));
    // nothing found, nothing to search
    assert_eq!(find(&s, "jesus wpet", SearchScope::Bible, 0).0, 0);
    assert_eq!(find(&s, "\" ... \"", SearchScope::Bible, 0).0, 0);
}

fn index_conn() -> Connection {
    content::open_content(
        &commands::content_dir().unwrap().join(SEARCH_FILE),
        expected::KJV_SEARCH_SQLITE,
        MAX_CONTENT_BYTES,
    )
    .unwrap()
}

#[test]
fn hostile_and_out_of_range_input_is_refused_or_literal() {
    let s = real();
    let bad = |q: &str, scope: SearchScope, offset: u32, limit: u32| {
        commands::run_search(&s, "KJV", q, scope, offset, limit).is_err()
    };
    assert!(bad(&"a".repeat(201), SearchScope::Bible, 0, 20));
    assert!(bad("a", SearchScope::Bible, 40_001, 20));
    assert!(bad("a", SearchScope::Bible, 0, 0));
    assert!(bad("a", SearchScope::Bible, 0, 101));
    assert!(bad("a", SearchScope::Book(0), 0, 20));
    assert!(bad("a", SearchScope::Book(67), 0, 20));
    assert!(commands::run_search(&s, "K J V", "a", SearchScope::Bible, 0, 20).is_err());
    for q in [
        "a OR b",
        "NEAR(a b)",
        "norm:jesus",
        "* OR *",
        "a;DROP TABLE x",
        "\0",
        "\"",
        "rowid:1",
    ] {
        assert!(matches!(
            commands::run_search(&s, "KJV", q, SearchScope::Bible, 0, 20).unwrap(),
            SearchResult::Ok { .. }
        ));
    }
}

/// Missing, damaged, stale or built from another kjv.sqlite: the Bible opens,
/// search is unavailable with the reason, and no result is ever given.
#[test]
fn a_missing_or_bad_index_makes_search_unavailable_not_the_bible() {
    let dir = commands::content_dir().unwrap();
    let tmp = crate::fixture::temp_dir("search-bad");
    for f in [KJV_FILE, LEX_FILE] {
        std::fs::copy(dir.join(f), tmp.join(f)).unwrap();
    }
    let mut stale = expected::KJV_SEARCH_SQLITE.to_string();
    let last = if stale.ends_with('0') { "1" } else { "0" };
    stale.replace_range(63..64, last);

    let check = |d: &std::path::Path, search_sha: &str, why: &str| {
        let c = commands::load_content_with(
            d,
            expected::KJV_SQLITE,
            expected::CONTENT_SQLITE,
            search_sha,
        )
        .unwrap_or_else(|e| panic!("{why}: the Bible must still open: {e}"));
        let err = c.search.as_ref().err().cloned().expect(why);
        assert!(err.contains(SEARCH_FILE), "{why}: {err}");
        let s = state(Ok(c));
        let info = commands::info(&s);
        assert!(
            info.content_error.is_none() && !info.search_available,
            "{why}"
        );
        assert_eq!(info.search_error.as_deref(), Some(err.as_str()), "{why}");
        assert_eq!(
            commands::with_bible(&s, "KJV", |b| b.chapter(43, 11))
                .unwrap()
                .verses
                .len(),
            57,
            "{why}"
        );
        assert_eq!(
            commands::run_search(&s, "KJV", "jesus wept", SearchScope::Bible, 0, 20).unwrap(),
            SearchResult::Unavailable { reason: err },
            "{why}"
        );
    };
    check(&tmp, expected::KJV_SEARCH_SQLITE, "missing");
    let mut bytes = std::fs::read(dir.join(SEARCH_FILE)).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    std::fs::write(tmp.join(SEARCH_FILE), &bytes).unwrap();
    check(&tmp, expected::KJV_SEARCH_SQLITE, "tampered");
    check(&dir, &stale, "stale hash");

    // Verified bytes, but built from a different kjv.sqlite: refused by
    // require_search_index.
    let other = crate::search::SearchIndex::open(
        &dir.join(SEARCH_FILE),
        expected::KJV_SEARCH_SQLITE,
        &"0".repeat(64),
    );
    assert!(
        other
            .err()
            .unwrap()
            .contains("built from a different kjv.sqlite")
    );

    // The real file works.
    let s = real();
    assert!(commands::info(&s).search_available);
}

// ---------------------------------------------------------------------------
// Golden for the UI render test (app/tests/search-render.test.ts)
// ---------------------------------------------------------------------------

/// Red letters inside and across matches, supplied words, the divine name, a
/// phrase and paging.
const GOLDEN_QUERIES: &[&str] = &[
    "jesus wept",
    "\"verily, verily, I say unto thee",
    "love one another",
    "lord god",
    "living water",
    "\"i am the way",
];
const GOLDEN_FILE: &str = "tests/golden/kjv-search.json";

fn search_golden() -> String {
    let s = real();
    let mut out = Vec::new();
    for q in GOLDEN_QUERIES {
        let r = commands::run_search(&s, "KJV", q, SearchScope::Bible, 0, 8).unwrap();
        let query = rs::parse_query(q).unwrap().unwrap();
        let (total, exact_total, phrase, hits) = ok(r);
        let spans: Vec<Vec<(u32, u32)>> = hits
            .iter()
            .map(|h| rs::match_spans(&h.text, &query))
            .collect();
        out.push(serde_json::json!({
            "query": q, "total": total, "exact_total": exact_total, "phrase": phrase,
            "hits": hits, "spans": spans,
        }));
    }
    serde_json::to_string_pretty(&out).unwrap() + "\n"
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_FILE)
}

#[test]
#[ignore]
fn regenerate_search_golden() {
    std::fs::write(golden_path(), search_golden()).unwrap();
}

#[test]
fn search_golden_is_up_to_date() {
    let committed = std::fs::read_to_string(golden_path())
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        committed == search_golden(),
        "{GOLDEN_FILE} is stale: cargo test -- --ignored regenerate_search_golden"
    );
}
