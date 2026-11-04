//! Phrase and word search for the command box (wave-2-layout 6;
//! pipeline/DESIGN.md 18.4 and 18.6).
//!
//! - `kjv-search.sqlite` is opened only through `open_verified` with its
//!   compiled-in hash, then `require_content_identity` and
//!   `require_search_index` (built from exactly the `kjv.sqlite` this app
//!   expects). Missing, damaged or stale: search answers "unavailable" and the
//!   Bible is not affected.
//! - The index gives **verse ids only**. Every result's text and runs are read
//!   from `kjv.sqlite`; emphasis is `match_spans` on that stored text, turned
//!   into runs here so the UI does no offset arithmetic.
//! - The ranked id list of the last query is kept, so "Show 20 more" and F3
//!   page through it without asking the index again.

use crate::books;
use crate::content::{self, Bible, MAX_CONTENT_BYTES};
use crate::runs::Run;
use revelation_content::search::{self as rs, Query, Ranked, Scope};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

/// The search index's file name inside the content folder.
pub const SEARCH_FILE: &str = "kjv-search.sqlite";
/// The translation the index was built from (its `meta.translation_abbrev`).
pub const SEARCH_TRANSLATION: &str = "KJV";
/// IPC bounds: no page starts past this (the KJV has 31,102 verses).
pub const MAX_OFFSET: u32 = 40_000;
/// IPC bounds: at most this many results per call (the UI asks for 20).
pub const MAX_LIMIT: u32 = 100;
/// IPC bounds: a query longer than this in bytes is refused before its
/// characters are counted (200 code points are at most 800 bytes).
pub const MAX_QUERY_BYTES: usize = rs::MAX_QUERY_CHARS * 4;

/// Which verses to search, as the UI sends it: `"bible"`, `"old_testament"`,
/// `"new_testament"` or `{"book": 43}`. Anything else is refused by serde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchScope {
    Bible,
    OldTestament,
    NewTestament,
    Book(u8),
}

impl From<SearchScope> for Scope {
    fn from(s: SearchScope) -> Scope {
        match s {
            SearchScope::Bible => Scope::Bible,
            SearchScope::OldTestament => Scope::OldTestament,
            SearchScope::NewTestament => Scope::NewTestament,
            SearchScope::Book(b) => Scope::Book(b),
        }
    }
}

struct Cached {
    input: String,
    scope: Scope,
    query: Option<Query>,
    ranked: Ranked,
}

pub struct SearchIndex {
    conn: Connection,
    /// The last query's full ranked list (one entry: paging and F3 reuse it).
    cache: Mutex<Option<Cached>>,
}

/// One page of the last query, read from the cache.
pub struct Page {
    pub query: Option<Query>,
    pub total: usize,
    pub exact_total: usize,
    /// `(verse id, exact phrase)` for results `offset ..`.
    pub hits: Vec<(i64, bool)>,
}

impl SearchIndex {
    /// Open `path` against `search_sha256` and require that it was built
    /// from the `kjv.sqlite` with SHA-256 `kjv_sha256`.
    pub fn open(path: &Path, search_sha256: &str, kjv_sha256: &str) -> Result<SearchIndex, String> {
        let conn = content::open_content(path, search_sha256, MAX_CONTENT_BYTES)
            .map_err(|e| e.to_string())?;
        rs::require_search_index(&conn, kjv_sha256).map_err(|e| format!("{SEARCH_FILE}: {e}"))?;
        Ok(SearchIndex {
            conn,
            cache: Mutex::new(None),
        })
    }

    /// Results `offset .. offset + limit` of `input` in `scope`. The ranked
    /// list is computed once per (input, scope) and kept for the next page.
    pub fn page(
        &self,
        input: &str,
        scope: Scope,
        offset: usize,
        limit: usize,
    ) -> Result<Page, rs::SearchError> {
        scope.id_range()?;
        let mut guard = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        let fresh = !matches!(&*guard, Some(c) if c.input == input && c.scope == scope);
        if fresh {
            let query = rs::parse_query(input)?;
            let ranked = match &query {
                Some(q) => rs::ranked(&self.conn, q, scope)?,
                None => Ranked::default(),
            };
            *guard = Some(Cached {
                input: input.to_string(),
                scope,
                query,
                ranked,
            });
        }
        let c = guard.as_ref().expect("cache was just filled");
        Ok(Page {
            query: c.query.clone(),
            total: c.ranked.ids.len(),
            exact_total: c.ranked.exact_total,
            hits: c
                .ranked
                .ids
                .iter()
                .enumerate()
                .skip(offset)
                .take(limit)
                .map(|(i, &id)| (id, i < c.ranked.exact_total))
                .collect(),
        })
    }
}

/// A presentation run of a result verse: the reader's run (red piece,
/// supplied words, divine name) without Strong's words (not interactive in
/// the dropdown), split where the emphasis starts and ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HitRun {
    /// Exact stored characters.
    pub text: String,
    pub red: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rp: Option<u32>,
    pub add: bool,
    pub nd: bool,
    /// Inside a match (`<mark class="hit">`, presentation only).
    pub hit: bool,
}

/// Splits `runs` (which concatenate to the stored text) at the code-point
/// boundaries of `spans` and flags the parts inside a span. Adjacent parts
/// with the same presentation are joined. `None` if a span doesn't fit the
/// text (the verse is then shown without emphasis).
pub fn hit_runs(runs: &[Run], spans: &[(u32, u32)]) -> Option<Vec<HitRun>> {
    let len: u32 = runs.iter().map(|r| r.text.chars().count() as u32).sum();
    if spans.iter().any(|&(a, b)| a >= b || b > len) {
        return None;
    }
    let mut out: Vec<HitRun> = Vec::new();
    let mut pos = 0u32;
    for r in runs {
        let chars: Vec<char> = r.text.chars().collect();
        let (start, end) = (pos, pos + chars.len() as u32);
        let mut cuts = vec![start, end];
        for &(a, b) in spans {
            cuts.extend([a, b].into_iter().filter(|&x| x > start && x < end));
        }
        cuts.sort_unstable();
        cuts.dedup();
        for w in cuts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let text: String = chars[(a - start) as usize..(b - start) as usize]
                .iter()
                .collect();
            let hit = spans.iter().any(|&(s, e)| s <= a && b <= e);
            match out.last_mut() {
                Some(l)
                    if l.red == r.red
                        && l.rp == r.rp
                        && l.add == r.add
                        && l.nd == r.nd
                        && l.hit == hit =>
                {
                    l.text.push_str(&text)
                }
                _ => out.push(HitRun {
                    text,
                    red: r.red,
                    rp: r.rp,
                    add: r.add,
                    nd: r.nd,
                    hit,
                }),
            }
        }
        pos = end;
    }
    Some(out)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchHit {
    /// `book*1000000 + chapter*1000 + verse`.
    pub id: i64,
    pub book: u32,
    pub osis: String,
    pub chapter: u32,
    pub verse: u32,
    /// "John 11:35" (book name from books.rs, as in the reader).
    pub reference: String,
    /// Has the exact phrase (sorted first).
    pub exact: bool,
    /// The stored verse text from `kjv.sqlite`, exactly.
    pub text: String,
    /// Concatenate to `text`.
    pub runs: Vec<HitRun>,
    /// False when the match couldn't be mapped onto the stored text: shown
    /// whole, without emphasis (wave-2-layout 6.5).
    pub emphasised: bool,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SearchResult {
    Ok {
        /// The true total in scope ("Verses · 248 in the KJV").
        total: usize,
        /// `hits` with `exact` come first; this many of `total` have it.
        exact_total: usize,
        offset: usize,
        /// The query was a quoted phrase.
        phrase: bool,
        hits: Vec<SearchHit>,
    },
    /// The index is missing, damaged or stale: no results, ever, from it.
    Unavailable { reason: String },
}

/// Validates the IPC arguments (before any lookup).
pub fn validate(query: &str, offset: u32, limit: u32) -> Result<(), String> {
    if query.len() > MAX_QUERY_BYTES || query.chars().count() > rs::MAX_QUERY_CHARS {
        return Err("query too long".into());
    }
    if offset > MAX_OFFSET || limit == 0 || limit > MAX_LIMIT {
        return Err("invalid offset or limit".into());
    }
    Ok(())
}

/// Builds the result rows of one page: text and runs from `bible`, emphasis
/// from `match_spans` on that text.
pub fn hits(bible: &Bible, page: &Page) -> Result<Vec<SearchHit>, String> {
    let mut out = Vec::with_capacity(page.hits.len());
    let mut names: Vec<(u32, String, String)> = Vec::new();
    for &(id, exact) in &page.hits {
        let (book, chapter, verse) = (
            (id / 1_000_000) as u32,
            (id / 1000 % 1000) as u32,
            (id % 1000) as u32,
        );
        let v = bible
            .verse_by_id(id)
            .map_err(|e| e.to_string())?
            .ok_or("the search index names a verse that is not in the Bible")?;
        if v.v != verse {
            return Err("the search index names a verse that is not in the Bible".into());
        }
        if !names.iter().any(|(b, _, _)| *b == book) {
            let osis = bible
                .book_osis(book)
                .map_err(|e| e.to_string())?
                .ok_or("the search index names a book that is not in the Bible")?;
            let name = books::names(&osis).name;
            names.push((book, osis, name));
        }
        let (_, osis, name) = names.iter().find(|(b, _, _)| *b == book).expect("added");
        let spans = page
            .query
            .as_ref()
            .map(|q| rs::match_spans(&v.text, q))
            .unwrap_or_default();
        let mapped = if spans.is_empty() {
            None
        } else {
            hit_runs(&v.runs, &spans)
        };
        let emphasised = mapped.is_some();
        if !emphasised {
            eprintln!("search: no emphasis could be mapped for verse {id}");
        }
        let runs = mapped.unwrap_or_else(|| hit_runs(&v.runs, &[]).expect("no spans always fit"));
        out.push(SearchHit {
            id,
            book,
            osis: osis.clone(),
            chapter,
            verse,
            reference: format!("{name} {chapter}:{verse}"),
            exact,
            text: v.text,
            runs,
            emphasised,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, red: bool, rp: Option<u32>, add: bool, wn: Option<u32>) -> Run {
        Run {
            text: text.into(),
            red,
            rp,
            add,
            nd: false,
            w: wn.map(|_| vec!["G1".to_string()]),
            wn,
        }
    }

    #[test]
    fn hit_runs_split_at_spans_and_keep_presentation() {
        // "Jesus wept." with "wept." red (piece 0) and "wept" one word.
        let runs = [
            run("Jesus", false, None, false, Some(0)),
            run(" ", false, None, false, None),
            run("wept", true, Some(0), false, Some(1)),
            run(".", true, Some(0), false, None),
        ];
        let out = hit_runs(&runs, &[(0, 5), (6, 10)]).unwrap();
        let got: Vec<(&str, bool, bool)> = out
            .iter()
            .map(|r| (r.text.as_str(), r.red, r.hit))
            .collect();
        assert_eq!(
            got,
            [
                ("Jesus", false, true),
                (" ", false, false),
                ("wept", true, true),
                (".", true, false)
            ]
        );
        let joined: String = out.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "Jesus wept.");
        // A span crossing the red boundary is split at it.
        let out = hit_runs(&runs, &[(0, 10)]).unwrap();
        let got: Vec<(&str, bool, bool)> = out
            .iter()
            .map(|r| (r.text.as_str(), r.red, r.hit))
            .collect();
        assert_eq!(
            got,
            [
                ("Jesus ", false, true),
                ("wept", true, true),
                (".", true, false)
            ]
        );
        // A span inside one run splits it in three.
        let runs = [run("in the beginning", false, None, true, None)];
        let out = hit_runs(&runs, &[(3, 6)]).unwrap();
        let got: Vec<(&str, bool, bool)> = out
            .iter()
            .map(|r| (r.text.as_str(), r.add, r.hit))
            .collect();
        assert_eq!(
            got,
            [
                ("in ", true, false),
                ("the", true, true),
                (" beginning", true, false)
            ]
        );
        // Astral characters count as one code point.
        let runs = [run("a\u{1D400}b c", false, None, false, None)];
        let out = hit_runs(&runs, &[(0, 3)]).unwrap();
        assert_eq!(out[0].text, "a\u{1D400}b");
        assert!(out[0].hit && !out[1].hit);
        // Spans that don't fit are refused, never clamped.
        assert_eq!(hit_runs(&runs, &[(0, 6)]), None);
        assert_eq!(hit_runs(&runs, &[(2, 2)]), None);
        assert_eq!(hit_runs(&[], &[]), Some(vec![]));
    }

    #[test]
    fn ipc_arguments_are_bounded() {
        assert!(validate("jesus wept", 0, 20).is_ok());
        assert!(validate(&"a".repeat(200), MAX_OFFSET, MAX_LIMIT).is_ok());
        assert!(validate(&"a".repeat(201), 0, 20).is_err());
        assert!(validate(&"\u{1D400}".repeat(201), 0, 20).is_err());
        assert!(validate(&"a".repeat(MAX_QUERY_BYTES + 1), 0, 20).is_err());
        assert!(validate("a", MAX_OFFSET + 1, 20).is_err());
        assert!(validate("a", 0, 0).is_err());
        assert!(validate("a", 0, MAX_LIMIT + 1).is_err());
    }

    #[test]
    fn scope_is_a_closed_enum() {
        let p = |s: &str| serde_json::from_str::<SearchScope>(s);
        assert_eq!(p("\"bible\"").unwrap(), SearchScope::Bible);
        assert_eq!(p("\"old_testament\"").unwrap(), SearchScope::OldTestament);
        assert_eq!(p("\"new_testament\"").unwrap(), SearchScope::NewTestament);
        assert_eq!(p("{\"book\":43}").unwrap(), SearchScope::Book(43));
        for bad in [
            "\"Bible\"",
            "\"all\"",
            "{\"book\":256}",
            "{\"book\":-1}",
            "{\"book\":\"43\"}",
            "{\"book\":43,\"x\":1}",
            "null",
            "1",
        ] {
            assert!(p(bad).is_err(), "{bad}");
        }
        // In range by type, out of range for the Bible: refused by the lookup.
        assert!(Scope::from(SearchScope::Book(0)).id_range().is_err());
        assert!(Scope::from(SearchScope::Book(67)).id_range().is_err());
    }
}
