//! The search query over a small FTS5 index made here (pipeline/DESIGN.md
//! 18): FTS5 is compiled into the bundled SQLite (proved, not assumed), the
//! SQLite version has the 2026 FTS5 fixes, ordering, scope, paging, and FTS5
//! syntax typed by a reader never reaches `MATCH` as syntax. The real index
//! is tested in pipeline/tests/search_real.rs.

use revelation_content::open_bytes;
use revelation_content::search::{
    Hit, PAGE, Query, Scope, SearchError, normalise, parse_query, ranked, require_search_index,
    search,
};
use rusqlite::Connection;

const VERSES: &[(i64, &str)] = &[
    (
        1_001_001,
        "In the beginning God created the heaven and the earth.",
    ),
    (19_023_001, "The LORD is my shepherd; I shall not want."),
    (
        23_006_003,
        "And one cried unto another, and said, Holy, holy, holy, is the LORD of hosts:",
    ),
    (
        40_005_004,
        "Blessed are they that mourn: for they shall be comforted.",
    ),
    (
        42_019_041,
        "And when he was come near, he beheld the city, and wept over it,",
    ),
    (
        43_003_023,
        "And John also was baptizing in Ænon near to Salim,",
    ),
    (43_011_035, "Jesus wept."),
    (
        43_020_011,
        "But Mary stood without at the sepulchre weeping:",
    ),
    (40_012_044, "he findeth it empty, swept, and garnished."),
    (
        43_011_033,
        "When Jesus therefore saw her weeping, and the Jews also weeping which came with her, he groaned in the spirit,",
    ),
    (
        66_022_021,
        "The grace of our Lord Jesus Christ be with you all. Amen. wept jesus",
    ),
];

/// An in-memory index with the pipeline's FTS5 table definition, serialised
/// and reopened through `open_bytes` (the app's locked-down settings).
fn index() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT, WITHOUT ROWID;
         CREATE VIRTUAL TABLE verse_fts USING fts5(norm, tokenize = 'ascii', detail = full);",
    )
    .unwrap();
    let mut v = VERSES.to_vec();
    v.sort();
    for (id, text) in v {
        c.execute(
            "INSERT INTO verse_fts (rowid, norm) VALUES (?1, ?2)",
            rusqlite::params![id, normalise(text)],
        )
        .unwrap();
    }
    c.execute_batch(
        "INSERT INTO meta VALUES ('search_normalisation', '1'), ('search_built_from_sha256', 'abc');
         INSERT INTO verse_fts (verse_fts) VALUES ('optimize');",
    )
    .unwrap();
    let bytes = c.serialize("main").unwrap().to_vec();
    open_bytes(&bytes).unwrap()
}

fn ids(c: &Connection, q: &str) -> Vec<i64> {
    search(c, q, Scope::Bible, 0, 1000)
        .unwrap()
        .hits
        .iter()
        .map(|h| h.verse_id)
        .collect()
}

#[test]
fn fts5_is_compiled_into_the_bundled_sqlite() {
    let c = Connection::open_in_memory().unwrap();
    let opts: Vec<String> = c
        .prepare("PRAGMA compile_options")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(opts.iter().any(|o| o == "ENABLE_FTS5"), "{opts:?}");
    let on: i64 = c
        .query_row("SELECT sqlite_compileoption_used('ENABLE_FTS5')", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(on, 1);
    c.execute_batch("CREATE VIRTUAL TABLE t USING fts5(x, tokenize = 'ascii')")
        .unwrap();
    // The two 2026 FTS5 memory-corruption CVEs are fixed in 3.53.2 (M1-F8).
    let v: String = c
        .query_row("SELECT sqlite_version()", [], |r| r.get(0))
        .unwrap();
    let n: Vec<u32> = v.split('.').map(|p| p.parse().unwrap()).collect();
    assert!(n >= vec![3, 53, 2], "SQLite {v}");
}

#[test]
fn jesus_wept_finds_john_11_35_first() {
    let c = index();
    let p = search(&c, "jesus wept", Scope::Bible, 0, PAGE).unwrap();
    assert_eq!(p.total, 2);
    assert_eq!(p.exact_total, 1);
    assert_eq!(
        p.hits,
        [
            Hit {
                verse_id: 43_011_035,
                exact: true
            },
            Hit {
                verse_id: 66_022_021,
                exact: false
            },
        ]
    );
    assert_eq!(ids(&c, "\"jesus wept\""), [43_011_035]);
    assert_eq!(ids(&c, "“Jesus wept.”"), [43_011_035]);
    assert_eq!(ids(&c, "\"wept jesus"), [66_022_021]);
}

#[test]
fn case_and_punctuation_do_not_matter() {
    let c = index();
    for q in [
        "JESUS WEPT",
        "Jesus, wept!",
        "jesus—wept",
        "  jesus   wept  ",
        "jesus;wept?",
    ] {
        assert_eq!(ids(&c, q), [43_011_035, 66_022_021], "{q}");
    }
    assert_eq!(ids(&c, "Ænon"), [43_003_023]);
    assert_eq!(ids(&c, "aenon"), [43_003_023]);
    assert_eq!(ids(&c, "æNON"), [43_003_023]);
}

#[test]
fn whole_words_only_no_prefix_no_stemming() {
    let c = index();
    assert_eq!(ids(&c, "wept"), [42_019_041, 43_011_035, 66_022_021]); // not "swept"
    assert_eq!(ids(&c, "swept"), [40_012_044]);
    assert_eq!(ids(&c, "weep"), Vec::<i64>::new()); // not "weeping"
    assert_eq!(ids(&c, "weeping"), [43_011_033, 43_020_011]);
    assert_eq!(ids(&c, "jesu"), Vec::<i64>::new());
    assert_eq!(ids(&c, "jesu*"), Vec::<i64>::new());
}

#[test]
fn phrase_first_then_all_words_with_true_totals_and_paging() {
    let c = index();
    let all = search(&c, "the lord", Scope::Bible, 0, 1).unwrap();
    // "the LORD" as a phrase: Ps 23:1, Isa 6:3; all the words also Rev 22:21.
    assert_eq!(all.total, 3);
    assert_eq!(all.exact_total, 2);
    assert_eq!(
        all.hits,
        [Hit {
            verse_id: 19_023_001,
            exact: true
        }]
    );
    let p2 = search(&c, "the lord", Scope::Bible, 1, 1).unwrap();
    assert_eq!(
        p2.hits,
        [Hit {
            verse_id: 23_006_003,
            exact: true
        }]
    );
    let p3 = search(&c, "the lord", Scope::Bible, 2, 1).unwrap();
    assert_eq!(
        p3.hits,
        [Hit {
            verse_id: 66_022_021,
            exact: false
        }]
    );
    let p4 = search(&c, "the lord", Scope::Bible, 3, 1).unwrap();
    assert!(p4.hits.is_empty());
    assert_eq!(p4.total, 3);
    // Quoted: the phrase only.
    let q = search(&c, "'the lord", Scope::Bible, 0, PAGE).unwrap();
    assert_eq!((q.total, q.exact_total), (2, 2));
}

#[test]
fn scope_filters_by_verse_id() {
    let c = index();
    let n = |q: &str, s: Scope| search(&c, q, s, 0, PAGE).unwrap().total;
    assert_eq!(n("and", Scope::Bible), 6);
    assert_eq!(n("and", Scope::OldTestament), 2);
    assert_eq!(n("and", Scope::NewTestament), 4);
    assert_eq!(n("and", Scope::Book(43)), 2);
    assert_eq!(n("and", Scope::Book(1)), 1);
    assert_eq!(n("and", Scope::Book(2)), 0);
    assert_eq!(
        search(&c, "and", Scope::Book(67), 0, PAGE),
        Err(SearchError::BadScope(67))
    );
}

#[test]
fn fts_syntax_is_literal_text_and_never_an_error() {
    let c = index();
    let hostile = [
        "\"",
        "\"\"",
        "\"\"\"",
        "*",
        "**",
        "jesus*",
        "NEAR(jesus wept)",
        "NEAR(jesus wept, 1)",
        "jesus OR wept",
        "jesus AND wept",
        "jesus NOT wept",
        "NOT",
        "OR",
        "AND",
        "-jesus",
        "jesus -wept",
        "norm:jesus",
        "verse_fts:jesus",
        "{norm}:jesus",
        "rowid:1",
        "^jesus",
        "(jesus)",
        "((",
        "))",
        "jesus + wept",
        "\"jesus\" OR \"wept\"",
        "'",
        "''",
        "“",
        "”",
        ":",
        "^",
        "-",
        "+",
        "\\",
        "\0",
        "jesus\"wept",
        "jesus\u{0}wept",
        ";",
        "'; DROP TABLE meta; --",
        "%",
        "_",
        "[",
        "]",
        "{",
        "}",
        "~",
        "`",
        "@",
        "#",
        "$",
        "&",
        "|",
        "<",
        ">",
        "=",
        "?",
        "!",
        "near",
        "Near/3",
        "jesus NEAR/2 wept",
        "\u{FEFF}jesus",
        "\u{200B}",
        "😀",
        "jesus 😀",
        "é",
        "יהוה",
        "θεος",
    ];
    for q in hostile {
        let r = search(&c, q, Scope::Bible, 0, PAGE);
        let page = r.unwrap_or_else(|e| panic!("{q:?}: {e}"));
        // Same result as the words alone, by a plain scan.
        let want: Vec<i64> = match &page.query {
            None => Vec::new(),
            Some(query) => brute(query),
        };
        let mut got: Vec<i64> = page.hits.iter().map(|h| h.verse_id).collect();
        got.sort();
        assert_eq!(got, want, "{q:?}");
    }
    // Operators are words: "or" as a word, not a union.
    assert_eq!(ids(&c, "jesus OR wept"), Vec::<i64>::new());
    assert_eq!(ids(&c, "NOT wept"), Vec::<i64>::new());
    assert_eq!(ids(&c, "-jesus wept"), [43_011_035, 66_022_021]);
    // Still usable afterwards.
    assert_eq!(ids(&c, "jesus wept")[0], 43_011_035);
}

/// All words / phrase by scanning the verses (sorted ids).
fn brute(q: &Query) -> Vec<i64> {
    let mut out: Vec<i64> = VERSES
        .iter()
        .filter(|(_, t)| {
            let words: Vec<String> = normalise(t).split(' ').map(String::from).collect();
            match q {
                Query::AllWords(w) => w.iter().all(|w| words.contains(w)),
                Query::Phrase(w) => words.windows(w.len()).any(|win| win == &w[..]),
            }
        })
        .map(|v| v.0)
        .collect();
    out.sort();
    out
}

#[test]
fn ranked_agrees_with_a_scan() {
    let c = index();
    for q in [
        "and",
        "the lord",
        "jesus",
        "wept over",
        "holy holy",
        "'holy holy holy",
        "he was come",
    ] {
        let query = parse_query(q).unwrap().unwrap();
        let r = ranked(&c, &query, Scope::Bible).unwrap();
        let mut got = r.ids.clone();
        got.sort();
        assert_eq!(got, brute(&query), "{q}");
        let exact = brute(&Query::Phrase(query.words().to_vec()));
        assert_eq!(r.ids[..r.exact_total], exact[..], "{q}");
    }
}

#[test]
fn index_identity_is_checked() {
    let c = index();
    require_search_index(&c, "abc").unwrap();
    assert!(matches!(
        require_search_index(&c, "other"),
        Err(SearchError::NotSearchIndex(_))
    ));
    let plain = Connection::open_in_memory().unwrap();
    plain
        .execute_batch("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        .unwrap();
    assert!(require_search_index(&plain, "abc").is_err());
}

/// W2P-B1: a phrase longer than `FTS_PHRASE_WORDS` is matched as its first
/// words, then checked whole; the result equals the plain phrase meaning.
#[test]
fn long_phrases_are_checked_whole() {
    let c = index();
    assert_eq!(
        ids(&c, "\"holy holy holy is the lord of hosts"),
        [23_006_003]
    );
    assert_eq!(ids(&c, "\"holy, holy, holy, is the LORD"), [23_006_003]);
    // The first four words match; the fifth doesn't.
    assert!(ids(&c, "\"holy holy holy is the king").is_empty());
    assert!(ids(&c, "\"and one cried unto another and wept").is_empty());
    assert_eq!(
        ids(&c, "\"and one cried unto another and said holy"),
        [23_006_003]
    );
    // Unquoted: the exact phrase first, then all-words matches.
    let p = search(
        &c,
        "in the beginning god created the heaven",
        Scope::Bible,
        0,
        PAGE,
    )
    .unwrap();
    assert_eq!((p.total, p.exact_total), (1, 1));
    // Repeats in the all-words pass don't change the set.
    assert_eq!(ids(&c, "holy holy holy holy holy"), [23_006_003]);
    assert_eq!(ids(&c, "jesus jesus wept"), ids(&c, "jesus wept"));
}
