//! Phrase and word search over `kjv-search.sqlite` (pipeline/DESIGN.md 18).
//!
//! Two halves, both shared by the pipeline (which builds the index) and the
//! app (which queries it), so the text that was indexed and the words that
//! are looked up go through **one** normalisation:
//!
//! - [`tokens`] / [`normalise`] (std only): the search-only normalised copy
//!   of a verse or a query (DESIGN.md 18.2). It is never shown to anyone.
//! - [`search`] and friends (feature `sqlite`): parse what the reader typed
//!   ([`parse_query`]), look it up with FTS5 `MATCH` built from quoted,
//!   normalised tokens only (never the reader's raw text), and return
//!   **verse ids**. The app reads `verse.text` from `kjv.sqlite` for display
//!   and computes emphasis on that stored text with [`match_spans`]; FTS5
//!   `snippet()` and `highlight()` are never used (DESIGN.md 10.3 rule 4,
//!   M1-F8).

/// Version of the normalisation below; stored in the index's `meta` as
/// `search_normalisation` and required by [`require_search_index`]. Any
/// change to [`fold`] is a new version and a new, owner-reviewed index hash.
pub const NORMALISATION: &str = "1";

/// The longest query accepted, in code points (the command box has
/// `maxlength="200"`, wave-2-layout 6.3).
pub const MAX_QUERY_CHARS: usize = 200;

/// Results per page (wave-2-layout 6.4: "Show 20 more").
pub const PAGE: usize = 20;

/// The most results one `search` call returns (W2P-B4): `limit` is clamped
/// to this, and `offset` to the number of results.
pub const MAX_PAGE: usize = 100;

/// Phrases of more words are matched as the FTS5 phrase of their first this
/// many words, then checked whole on the candidates (W2P-B1, `phrase_ids`).
pub const FTS_PHRASE_WORDS: usize = 4;

/// The quote marks that, typed first, make the whole query an exact phrase
/// (wave-2-layout 6.2), including the opening marks of other languages
/// (W2P-B2): „ « » ‚ ‹ ›. All of them separate words in [`fold`] too.
pub const PHRASE_QUOTES: [char; 10] = [
    '"', '\'', '\u{201C}', '\u{2018}', '\u{201E}', '\u{00AB}', '\u{00BB}', '\u{201A}', '\u{2039}',
    '\u{203A}',
];

/// Invisible characters a pasted query may start with (BOM, zero-width
/// space, non-joiner, joiner, word joiner). They are dropped from the start
/// of a query before the phrase test and before its words are read, so a
/// pasted `\u{FEFF}"jesus wept` is still a phrase (W2P-B2).
pub const LEADING_INVISIBLE: [char; 5] =
    ['\u{FEFF}', '\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}'];

/// Non-ASCII characters that separate words (besides `char::is_whitespace`).
/// Every ASCII character that is not a letter or digit separates words too.
pub const SEPARATORS: &[char] = &[
    '\u{00AB}', // «
    '\u{00AD}', // soft hyphen
    '\u{00B6}', // ¶
    '\u{00B7}', // ·
    '\u{00BB}', // »
    '\u{200B}', // zero width space
    '\u{2010}', // hyphen
    '\u{2011}', // non-breaking hyphen
    '\u{2012}', // figure dash
    '\u{2013}', // – en dash (KJV)
    '\u{2014}', // — em dash (KJV)
    '\u{2015}', // horizontal bar
    '\u{2018}', // ‘
    '\u{2019}', // ’ (KJV)
    '\u{201A}', // ‚
    '\u{201B}', // ‛
    '\u{201C}', // “
    '\u{201D}', // ”
    '\u{201E}', // „
    '\u{201F}', // ‟
    '\u{2026}', // …
    '\u{2032}', // ′
    '\u{2033}', // ″
    '\u{2039}', // ‹
    '\u{203A}', // ›
    '\u{FEFF}', // zero width no-break space
];

/// What one character contributes to the normalised text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// Part of a word, as this one lower-case ASCII letter or digit.
    Char(char),
    /// Part of a word, as the two letters `ae` (`Æ`, `æ`: "Ænon", John 3:23).
    Ae,
    /// Ends a word and is dropped.
    Sep,
    /// Any other character (a non-ASCII letter, mark or symbol). Kept in the
    /// word, which can then never match the index (the index holds only
    /// `[a-z0-9]`): no guessing what an unknown character "means".
    Foreign(char),
}

/// The normalisation, one character at a time (DESIGN.md 18.2):
/// ASCII letters fold to lower case; ASCII digits stay; `Æ`/`æ` become `ae`;
/// every other ASCII character, every whitespace character and the
/// [`SEPARATORS`] separate words; anything else is [`Fold::Foreign`].
pub fn fold(c: char) -> Fold {
    match c {
        'A'..='Z' => Fold::Char(c.to_ascii_lowercase()),
        'a'..='z' | '0'..='9' => Fold::Char(c),
        '\u{00C6}' | '\u{00E6}' => Fold::Ae,
        _ if c.is_ascii() || c.is_whitespace() || SEPARATORS.contains(&c) => Fold::Sep,
        _ => Fold::Foreign(c),
    }
}

/// One word of a text: its code-point range in the text as given and its
/// normalised form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub start_cp: u32,
    /// Exclusive.
    pub end_cp: u32,
    pub norm: String,
    /// False when the word has a [`Fold::Foreign`] character.
    pub matchable: bool,
}

/// The words of `text` in order, with code-point offsets into `text`.
pub fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut cur: Option<Token> = None;
    for (i, c) in text.chars().enumerate() {
        let i = i as u32;
        let f = fold(c);
        if f == Fold::Sep {
            out.extend(cur.take());
            continue;
        }
        let t = cur.get_or_insert_with(|| Token {
            start_cp: i,
            end_cp: i,
            norm: String::new(),
            matchable: true,
        });
        t.end_cp = i + 1;
        match f {
            Fold::Char(c) => t.norm.push(c),
            Fold::Ae => t.norm.push_str("ae"),
            Fold::Foreign(c) => {
                t.norm.push(c);
                t.matchable = false;
            }
            Fold::Sep => unreachable!(),
        }
    }
    out.extend(cur);
    out
}

/// The search-only normalised copy of `text`: its words ([`tokens`]) joined
/// by single spaces. Never displayed.
pub fn normalise(text: &str) -> String {
    let words: Vec<String> = tokens(text).into_iter().map(|t| t.norm).collect();
    words.join(" ")
}

/// [`normalise`] for indexed text: fails with the first character that
/// would make a word unmatchable, so the index never holds a word no query
/// can reach (the build stops instead; DESIGN.md 18.2).
pub fn normalise_indexed(text: &str) -> Result<String, char> {
    if let Some(c) = text.chars().find_map(|c| match fold(c) {
        Fold::Foreign(c) => Some(c),
        _ => None,
    }) {
        return Err(c);
    }
    Ok(normalise(text))
}

/// What a query asks for (wave-2-layout 6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// Typed with a leading quote mark: these words, consecutive, in order.
    Phrase(Vec<String>),
    /// Unquoted: every one of these words, whole words, any order. Verses
    /// with them as an exact phrase come first.
    AllWords(Vec<String>),
}

/// Why a query or an index can't be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Longer than [`MAX_QUERY_CHARS`] code points.
    TooLong {
        chars: usize,
    },
    /// `Scope::Book` outside 1..=66.
    BadScope(u8),
    /// The file is not the search index this code expects.
    NotSearchIndex(String),
    Sqlite(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::TooLong { chars } => write!(
                f,
                "query of {chars} characters is over the limit of {MAX_QUERY_CHARS}"
            ),
            SearchError::BadScope(b) => write!(f, "no book {b}"),
            SearchError::NotSearchIndex(why) => write!(f, "not the search index: {why}"),
            SearchError::Sqlite(e) => write!(f, "SQLite: {e}"),
        }
    }
}

impl std::error::Error for SearchError {}

/// Parses what the reader typed. A leading quote mark (after spaces) makes
/// the whole query a phrase; quote marks anywhere else are punctuation.
/// `Ok(None)` when it has no words (nothing to search).
pub fn parse_query(input: &str) -> Result<Option<Query>, SearchError> {
    // More than 4 bytes per allowed code point is too long whatever it holds,
    // so an oversized string is refused before it is scanned (W2P-B4);
    // `chars` is then a lower bound.
    if input.len() > 4 * MAX_QUERY_CHARS {
        return Err(SearchError::TooLong {
            chars: input.len() / 4,
        });
    }
    let chars = input.chars().count();
    if chars > MAX_QUERY_CHARS {
        return Err(SearchError::TooLong { chars });
    }
    let body =
        input.trim_start_matches(|c: char| c.is_whitespace() || LEADING_INVISIBLE.contains(&c));
    let words: Vec<String> = tokens(body).into_iter().map(|t| t.norm).collect();
    if words.is_empty() {
        return Ok(None);
    }
    let quoted = body
        .chars()
        .next()
        .is_some_and(|c| PHRASE_QUOTES.contains(&c));
    Ok(Some(if quoted {
        Query::Phrase(words)
    } else {
        Query::AllWords(words)
    }))
}

impl Query {
    pub fn words(&self) -> &[String] {
        match self {
            Query::Phrase(w) | Query::AllWords(w) => w,
        }
    }

    /// Whether every word can occur in the index (`[a-z0-9]+`). A query
    /// with a foreign character matches nothing, without running FTS5.
    pub fn matchable(&self) -> bool {
        self.words().iter().all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
    }

    /// The FTS5 query for the exact phrase: `"w1 w2 …"`. `None` if not
    /// [`matchable`](Self::matchable). Words are `[a-z0-9]+` by then, so
    /// nothing in them is FTS5 syntax; they are quoted anyway.
    pub fn fts_phrase(&self) -> Option<String> {
        self.matchable()
            .then(|| fts_string(&self.words().join(" ")))
    }

    /// The distinct words, in first-seen order (W2P-B1). "Every one of these
    /// words" is the same set with or without repeats, so the all-words pass
    /// asks FTS5 for each word once; the phrase keeps its repeats.
    pub fn distinct_words(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for w in self.words() {
            if !out.contains(&w.as_str()) {
                out.push(w);
            }
        }
        out
    }

    /// The FTS5 query for all the words: `"w1" "w2" …` (implicit AND), each
    /// distinct word once.
    pub fn fts_all_words(&self) -> Option<String> {
        self.matchable().then(|| {
            let parts: Vec<String> = self.distinct_words().into_iter().map(fts_string).collect();
            parts.join(" ")
        })
    }
}

/// An FTS5 string literal: double quotes, any `"` inside doubled.
fn fts_string(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// Code-point ranges of `text` (a stored verse text) to emphasise for `q`:
/// each matching word for [`Query::AllWords`]; each occurrence of the whole
/// phrase, from its first word's start to its last word's end, for
/// [`Query::Phrase`]. Sorted; overlapping ranges merged. Empty if nothing
/// matches (the caller then shows the verse without emphasis).
pub fn match_spans(text: &str, q: &Query) -> Vec<(u32, u32)> {
    let toks = tokens(text);
    let mut spans: Vec<(u32, u32)> = Vec::new();
    match q {
        Query::AllWords(words) => {
            for t in &toks {
                if words.contains(&t.norm) {
                    spans.push((t.start_cp, t.end_cp));
                }
            }
        }
        Query::Phrase(words) => {
            let n = words.len();
            if n > 0 && toks.len() >= n {
                for i in 0..=toks.len() - n {
                    if toks[i..i + n].iter().zip(words).all(|(t, w)| &t.norm == w) {
                        spans.push((toks[i].start_cp, toks[i + n - 1].end_cp));
                    }
                }
            }
        }
    }
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for s in spans {
        match merged.last_mut() {
            Some(last) if s.0 <= last.1 => last.1 = last.1.max(s.1),
            _ => merged.push(s),
        }
    }
    merged
}

/// Which verses to search (wave-2-layout 6: whole Bible, OT, NT, a book).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Bible,
    OldTestament,
    NewTestament,
    /// `book.ord`, 1..=66.
    Book(u8),
}

impl Scope {
    /// The inclusive verse-id range (`id = book*1000000 + chapter*1000 + verse`).
    pub fn id_range(self) -> Result<(i64, i64), SearchError> {
        let books = match self {
            Scope::Bible => (1, 66),
            Scope::OldTestament => (1, 39),
            Scope::NewTestament => (40, 66),
            Scope::Book(b) if (1..=66).contains(&b) => (i64::from(b), i64::from(b)),
            Scope::Book(b) => return Err(SearchError::BadScope(b)),
        };
        Ok((books.0 * 1_000_000, books.1 * 1_000_000 + 999_999))
    }
}

#[cfg(feature = "sqlite")]
pub use db::*;

#[cfg(feature = "sqlite")]
mod db {
    use rusqlite::Connection;

    use super::{
        FTS_PHRASE_WORDS, NORMALISATION, Query, Scope, SearchError, fts_string, parse_query,
    };

    /// The FTS5 table of `kjv-search.sqlite`: `rowid` = `verse.id`, one
    /// column `norm` (DESIGN.md 18.3).
    pub const FTS_TABLE: &str = "verse_fts";

    fn sql(e: rusqlite::Error) -> SearchError {
        SearchError::Sqlite(e.to_string())
    }

    /// Requires the index's `meta` to name this normalisation and to have
    /// been built from the `kjv.sqlite` with SHA-256 `kjv_sha256` (the value
    /// compiled into the app), so the ids it returns are that file's verses.
    /// Call after `open_verified` and `require_content_identity`.
    pub fn require_search_index(conn: &Connection, kjv_sha256: &str) -> Result<(), SearchError> {
        let get = |k: &str| -> Result<Option<String>, SearchError> {
            let mut st = conn
                .prepare("SELECT value FROM meta WHERE key = ?1")
                .map_err(sql)?;
            let mut rows = st.query([k]).map_err(sql)?;
            match rows.next().map_err(sql)? {
                Some(r) => Ok(Some(r.get(0).map_err(sql)?)),
                None => Ok(None),
            }
        };
        if get("search_normalisation")?.as_deref() != Some(NORMALISATION) {
            return Err(SearchError::NotSearchIndex(
                "search_normalisation differs".into(),
            ));
        }
        if get("search_built_from_sha256")?.as_deref() != Some(kjv_sha256) {
            return Err(SearchError::NotSearchIndex(
                "built from a different kjv.sqlite".into(),
            ));
        }
        Ok(())
    }

    /// Every result of `q` in `scope`, in display order: exact-phrase
    /// matches first, then the other all-words matches, canonical (verse id)
    /// order within each.
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Ranked {
        pub ids: Vec<i64>,
        /// `ids[..exact_total]` are the exact-phrase matches.
        pub exact_total: usize,
    }

    fn ids(conn: &Connection, fts: &str, lo: i64, hi: i64) -> Result<Vec<i64>, SearchError> {
        // The query text is a bound parameter, never part of the SQL.
        let mut st = conn
            .prepare(
                "SELECT rowid FROM verse_fts WHERE verse_fts MATCH ?1 \
                 AND rowid BETWEEN ?2 AND ?3 ORDER BY rowid",
            )
            .map_err(sql)?;
        let rows = st
            .query_map(rusqlite::params![fts, lo, hi], |r| r.get(0))
            .map_err(sql)?;
        rows.collect::<rusqlite::Result<Vec<i64>>>().map_err(sql)
    }

    /// Verses with `words` as consecutive words (W2P-B1). Up to
    /// [`FTS_PHRASE_WORDS`] words: one FTS5 phrase query. Longer: the FTS5
    /// phrase of the first [`FTS_PHRASE_WORDS`] words finds the candidates
    /// (every match of the whole phrase is one of them), and each candidate's
    /// indexed `norm` (lower-case `[a-z0-9]` words joined by single spaces,
    /// the very tokens FTS5's `ascii` tokenizer sees) is checked for the whole
    /// phrase. Same result as the long FTS5 phrase, without reading a common
    /// word's position list once per repeat. `norm` is never displayed.
    fn phrase_ids(
        conn: &Connection,
        words: &[String],
        lo: i64,
        hi: i64,
    ) -> Result<Vec<i64>, SearchError> {
        if words.len() <= FTS_PHRASE_WORDS {
            return ids(conn, &fts_string(&words.join(" ")), lo, hi);
        }
        let head = fts_string(&words[..FTS_PHRASE_WORDS].join(" "));
        let mut st = conn
            .prepare(
                "SELECT rowid, norm FROM verse_fts WHERE verse_fts MATCH ?1 \
                 AND rowid BETWEEN ?2 AND ?3 ORDER BY rowid",
            )
            .map_err(sql)?;
        let rows = st
            .query_map(rusqlite::params![head, lo, hi], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(sql)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, norm) = row.map_err(sql)?;
            let toks: Vec<&str> = norm.split(' ').collect();
            if toks
                .windows(words.len())
                .any(|w| w.iter().zip(words).all(|(t, q)| *t == q))
            {
                out.push(id);
            }
        }
        Ok(out)
    }

    /// All results of a parsed query (see [`Ranked`]). A query with a
    /// foreign character has none and runs no SQL.
    pub fn ranked(conn: &Connection, q: &Query, scope: Scope) -> Result<Ranked, SearchError> {
        let (lo, hi) = scope.id_range()?;
        let Some(all) = q.fts_all_words() else {
            return Ok(Ranked::default());
        };
        let exact = phrase_ids(conn, q.words(), lo, hi)?;
        if matches!(q, Query::Phrase(_)) || q.words().len() == 1 {
            let n = exact.len();
            return Ok(Ranked {
                ids: exact,
                exact_total: n,
            });
        }
        let every = ids(conn, &all, lo, hi)?;
        // Both lists are sorted; every exact match is an all-words match.
        let mut rest = Vec::with_capacity(every.len().saturating_sub(exact.len()));
        let mut e = exact.iter().peekable();
        for &id in &every {
            if e.peek() == Some(&&id) {
                e.next();
            } else {
                rest.push(id);
            }
        }
        if e.next().is_some() {
            return Err(SearchError::NotSearchIndex(
                "a phrase match is not an all-words match".into(),
            ));
        }
        let exact_total = exact.len();
        let mut ids = exact;
        ids.extend(rest);
        Ok(Ranked { ids, exact_total })
    }

    /// One result row: which verse, and whether it has the exact phrase.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Hit {
        pub verse_id: i64,
        pub exact: bool,
    }

    /// One page of results.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SearchPage {
        /// The parsed query (for [`super::match_spans`]); `None` when the
        /// input has no words.
        pub query: Option<Query>,
        /// The true total in scope ("Verses · 248 in the KJV").
        pub total: usize,
        /// How many of `total` have the exact phrase ("Exact phrase · 1");
        /// `total - exact_total` have all the words only.
        pub exact_total: usize,
        /// Position of `hits[0]` in the full list.
        pub offset: usize,
        pub hits: Vec<Hit>,
    }

    /// Parses `input` and returns results `offset .. offset + limit` (the
    /// app passes `limit` = [`super::PAGE`]). Bounded whatever it is given
    /// (W2P-B4): `limit` is clamped to [`super::MAX_PAGE`] and `offset` to
    /// the number of results (the page is then empty; `SearchPage::offset`
    /// is the clamped value).
    pub fn search(
        conn: &Connection,
        input: &str,
        scope: Scope,
        offset: usize,
        limit: usize,
    ) -> Result<SearchPage, SearchError> {
        scope.id_range()?;
        let query = parse_query(input)?;
        let r = match &query {
            Some(q) => ranked(conn, q, scope)?,
            None => Ranked::default(),
        };
        let offset = offset.min(r.ids.len());
        let limit = limit.min(super::MAX_PAGE);
        let hits = r
            .ids
            .iter()
            .enumerate()
            .skip(offset)
            .take(limit)
            .map(|(i, &verse_id)| Hit {
                verse_id,
                exact: i < r.exact_total,
            })
            .collect();
        Ok(SearchPage {
            query,
            total: r.ids.len(),
            exact_total: r.exact_total,
            offset,
            hits,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_punctuation_dashes_and_ae() {
        assert_eq!(normalise("Jesus wept."), "jesus wept");
        assert_eq!(
            normalise("In the beginning God created the heaven and the earth."),
            "in the beginning god created the heaven and the earth"
        );
        assert_eq!(normalise("the LORD’s house"), "the lord s house");
        assert_eq!(normalise("Beer\u{2013}sheba"), "beer sheba");
        assert_eq!(normalise("son-in-law"), "son in law");
        assert_eq!(normalise("Ænon near to Salim"), "aenon near to salim");
        assert_eq!(normalise("æ"), "ae");
        assert_eq!(normalise("  [a]  (b) — c; d: e? f! "), "a b c d e f");
        assert_eq!(normalise(""), "");
        assert_eq!(normalise("…—“”"), "");
        assert_eq!(normalise("Psalm 119"), "psalm 119");
    }

    #[test]
    fn offsets_are_code_points_of_the_given_text() {
        let t = tokens("Ænon, “Lord’s”");
        assert_eq!(
            t.iter()
                .map(|t| (t.start_cp, t.end_cp, t.norm.as_str()))
                .collect::<Vec<_>>(),
            [(0, 4, "aenon"), (7, 11, "lord"), (12, 13, "s")]
        );
    }

    #[test]
    fn foreign_characters_make_a_word_unmatchable() {
        let t = tokens("café ok");
        assert!(!t[0].matchable);
        assert_eq!(t[0].norm, "café");
        assert!(t[1].matchable);
        assert_eq!(normalise_indexed("café"), Err('é'));
        assert_eq!(normalise_indexed("Ænon—x’s"), Ok("aenon x s".to_string()));
        let q = parse_query("jesus café").unwrap().unwrap();
        assert!(!q.matchable());
        assert_eq!(q.fts_phrase(), None);
        assert_eq!(q.fts_all_words(), None);
    }

    #[test]
    fn quotes_first_make_a_phrase() {
        assert_eq!(
            parse_query("jesus wept").unwrap(),
            Some(Query::AllWords(vec!["jesus".into(), "wept".into()]))
        );
        for s in [
            "\"Jesus wept\"",
            "  'jesus wept",
            "“Jesus wept.”",
            "‘jesus wept’",
        ] {
            assert_eq!(
                parse_query(s).unwrap(),
                Some(Query::Phrase(vec!["jesus".into(), "wept".into()])),
                "{s}"
            );
        }
        // Quote marks elsewhere are punctuation.
        assert_eq!(
            parse_query("jesus \"wept\"").unwrap(),
            Some(Query::AllWords(vec!["jesus".into(), "wept".into()]))
        );
        assert_eq!(parse_query("").unwrap(), None);
        assert_eq!(parse_query("\" ... \"").unwrap(), None);
        assert!(parse_query(&"a".repeat(200)).unwrap().is_some());
        assert_eq!(
            parse_query(&"a".repeat(201)),
            Err(SearchError::TooLong { chars: 201 })
        );
        // An oversized string is refused before it is scanned (W2P-B4).
        assert!(matches!(
            parse_query(&"\u{1D400}".repeat(201)),
            Err(SearchError::TooLong { .. })
        ));
        assert!(matches!(
            parse_query(&"a".repeat(10_000)),
            Err(SearchError::TooLong { chars: 2500 })
        ));
    }

    /// W2P-B2: a pasted BOM or zero-width character before the quote, and
    /// the opening quote marks of other languages.
    #[test]
    fn leading_invisibles_are_dropped_and_other_quotes_open_a_phrase() {
        let phrase = Some(Query::Phrase(vec!["jesus".into(), "wept".into()]));
        let all = Some(Query::AllWords(vec!["jesus".into(), "wept".into()]));
        for s in [
            "\u{FEFF}\"jesus wept",
            "\u{200B}\"jesus wept",
            "\u{200C}\u{200D}\u{2060}\"jesus wept",
            " \u{FEFF} \u{200B} 'jesus wept",
            "\u{201E}Jesus wept\u{201C}",
            "\u{00AB}jesus wept\u{00BB}",
            "\u{00BB}jesus wept\u{00AB}",
            "\u{201A}jesus wept\u{2018}",
            "\u{2039}jesus wept\u{203A}",
            "\u{203A}jesus wept\u{2039}",
        ] {
            assert_eq!(parse_query(s).unwrap(), phrase, "{s:?}");
        }
        for s in [
            "\u{FEFF}jesus wept",
            "\u{200D}jesus wept",
            "jesus \u{201E}wept",
        ] {
            assert_eq!(parse_query(s).unwrap(), all, "{s:?}");
        }
        // Only leading ones: inside a query a ZWJ is still a foreign character.
        assert!(
            !parse_query("jesus\u{200D} wept")
                .unwrap()
                .unwrap()
                .matchable()
        );
        assert_eq!(parse_query("\u{FEFF}\u{200B}").unwrap(), None);
        // Every phrase quote separates words, so it never joins a word.
        for c in PHRASE_QUOTES {
            assert_eq!(fold(c), Fold::Sep, "{c}");
        }
    }

    /// W2P-B1: repeated words are asked for once in the all-words pass; the
    /// phrase keeps them.
    #[test]
    fn repeated_words_are_deduplicated_for_all_words_only() {
        let q = parse_query("the lord the the LORD").unwrap().unwrap();
        assert_eq!(q.words().len(), 5);
        assert_eq!(q.distinct_words(), ["the", "lord"]);
        assert_eq!(q.fts_all_words().unwrap(), "\"the\" \"lord\"");
        assert_eq!(q.fts_phrase().unwrap(), "\"the lord the the lord\"");
        let fifty = parse_query(&"the ".repeat(50)).unwrap().unwrap();
        assert_eq!(fifty.fts_all_words().unwrap(), "\"the\"");
    }

    #[test]
    fn fts_strings_hold_only_quoted_words() {
        let q = parse_query("Jesus wept").unwrap().unwrap();
        assert_eq!(q.fts_phrase().unwrap(), "\"jesus wept\"");
        assert_eq!(q.fts_all_words().unwrap(), "\"jesus\" \"wept\"");
        // FTS5 syntax typed by the reader becomes words or disappears.
        for s in [
            "\"",
            "*",
            "a*",
            "NEAR(a b)",
            "a OR b",
            "a AND b",
            "NOT a",
            "-a",
            "a:b",
            "norm:jesus",
            "{norm}: a",
            "^a",
            "(a)",
            "a + b",
            "\"a\" \"b\"",
            "''",
            "\0a",
            "a\"b",
            "a)b(",
            "NEAR/2",
            "rowid:1",
            "* OR *",
            "\\",
            "%_",
            "a;DROP TABLE x",
        ] {
            let Some(q) = parse_query(s).unwrap() else {
                continue;
            };
            for w in q.words() {
                assert!(
                    w.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()),
                    "{s:?} gave word {w:?}"
                );
            }
            let all = q.fts_all_words().unwrap();
            let inner: String = all.chars().filter(|&c| c != '"').collect();
            assert!(
                inner
                    .bytes()
                    .all(|b| b == b' ' || b.is_ascii_lowercase() || b.is_ascii_digit()),
                "{s:?} gave {all:?}"
            );
        }
        assert_eq!(
            parse_query("NEAR(a b)")
                .unwrap()
                .unwrap()
                .fts_all_words()
                .unwrap(),
            "\"near\" \"a\" \"b\""
        );
        assert_eq!(
            parse_query("a OR b")
                .unwrap()
                .unwrap()
                .fts_all_words()
                .unwrap(),
            "\"a\" \"or\" \"b\""
        );
        assert_eq!(fts_string("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn spans_cover_whole_tokens_on_the_stored_text() {
        let text = "Holy, holy, holy, is the LORD’s name.";
        let q = parse_query("holy").unwrap().unwrap();
        assert_eq!(match_spans(text, &q), [(0, 4), (6, 10), (12, 16)]);
        let q = parse_query("\"holy holy").unwrap().unwrap();
        assert_eq!(match_spans(text, &q), [(0, 16)]);
        let q = parse_query("lord name").unwrap().unwrap();
        assert_eq!(match_spans(text, &q), [(25, 29), (32, 36)]);
        let q = parse_query("\"lord's name").unwrap().unwrap();
        assert_eq!(match_spans(text, &q), [(25, 36)]);
        let q = parse_query("swept").unwrap().unwrap();
        assert!(match_spans("Jesus wept.", &q).is_empty());
    }

    #[test]
    fn scopes_are_verse_id_ranges() {
        assert_eq!(Scope::Bible.id_range(), Ok((1_000_000, 66_999_999)));
        assert_eq!(Scope::OldTestament.id_range(), Ok((1_000_000, 39_999_999)));
        assert_eq!(Scope::NewTestament.id_range(), Ok((40_000_000, 66_999_999)));
        assert_eq!(Scope::Book(43).id_range(), Ok((43_000_000, 43_999_999)));
        assert_eq!(Scope::Book(0).id_range(), Err(SearchError::BadScope(0)));
        assert_eq!(Scope::Book(67).id_range(), Err(SearchError::BadScope(67)));
    }
}
