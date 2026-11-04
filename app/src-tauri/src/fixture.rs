//! The synthetic placeholder fixture (task D3): tiny content databases in the
//! DESIGN.md 8 schema, filled with OBVIOUS placeholder text ("alpha beta
//! gamma"), never Scripture. Used by the Rust tests, the UI render test and to
//! run the app before the real KJV arrives (D4). The UI shows a "placeholder
//! fixture" banner whenever these files are loaded.
//!
//! The files are built by the test helper below (`cargo test -- --ignored
//! regenerate_fixture` rewrites them) and committed under
//! `tests/fixtures/`. Their SHA-256 values are compiled in here (DESIGN.md
//! 10.2: never read from beside the file); `fixture_files_are_up_to_date`
//! fails if a rebuild differs from them.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bible,
}

#[derive(Debug, Clone, Copy)]
pub struct ContentFile {
    pub file: &'static str,
    pub kind: Kind,
    pub sha256: &'static str,
}

pub const BIBLE_RED: &str = "placeholder-bible-red.sqlite";
pub const BIBLE_PLAIN: &str = "placeholder-bible-plain.sqlite";

/// Expected SHA-256 of each committed fixture file (compiled in).
pub const FILES: &[ContentFile] = &[
    ContentFile {
        file: BIBLE_RED,
        kind: Kind::Bible,
        sha256: "14b0519d5c09525526baff32a4f74d34aac5d79581d46a6b55913eb9de51bd80",
    },
    ContentFile {
        file: BIBLE_PLAIN,
        kind: Kind::Bible,
        sha256: "9197368474c75ffaeca2c5a0c1d0ce3e16cd86570ac3ff69650eb93a4563fc49",
    },
];

/// Where the fixture files live. D3 only: an absolute path fixed at compile
/// time (the crate's own directory). D4 replaces this with the real content
/// location.
pub fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Testbook 1:1, used by the tamper tests.
#[cfg(test)]
pub const FIRST_VERSE_ID: i64 = 1_001_001;

#[cfg(test)]
pub fn first_verse_text() -> String {
    build::parse_markup(build::TESTBOOK_1[0], true).0
}

/// A fresh, empty directory under the workspace's `target/tmp` (stays on the
/// build drive). No process id in the name: reruns reuse (after emptying) the
/// same directories instead of piling up copies of the 24 MB database.
#[cfg(test)]
pub fn temp_dir(name: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tmp")
        .join(format!("app-{name}-{}", N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[cfg(test)]
pub(crate) mod build {
    use super::*;
    use crate::content::{APPLICATION_ID, Bible, SCHEMA_VERSION, sha256_hex};
    use rusqlite::{Connection, params};
    use serde::Serialize;

    /// pipeline/DESIGN.md 8, copied verbatim (Bible part).
    pub const BIBLE_SCHEMA: &str = r#"
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
CREATE TABLE source (
  id     TEXT PRIMARY KEY,
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;
CREATE TABLE book (
  ord      INTEGER PRIMARY KEY CHECK (ord BETWEEN 1 AND 66),
  osis     TEXT NOT NULL UNIQUE,
  usfm     TEXT NOT NULL UNIQUE,
  chapters INTEGER NOT NULL CHECK (chapters >= 1),
  main_title TEXT CHECK (main_title IS NULL OR length(main_title) BETWEEN 1 AND 256)
) STRICT;
CREATE TABLE verse (
  id      INTEGER PRIMARY KEY,
  book    INTEGER NOT NULL REFERENCES book(ord),
  chapter INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  verse   INTEGER NOT NULL CHECK (verse BETWEEN 1 AND 999),
  text    TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 4096),
  CHECK (id = book * 1000000 + chapter * 1000 + verse),
  UNIQUE (book, chapter, verse)
) STRICT;
CREATE TABLE mark (
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  kind     TEXT NOT NULL CHECK (kind IN ('added', 'divine')),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE TABLE red_piece (
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  q_ord    INTEGER NOT NULL CHECK (q_ord >= 1),
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX red_piece_by_q ON red_piece (q_ord, verse_id);
CREATE TABLE word (
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp <= end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE TABLE word_key (
  verse_id INTEGER NOT NULL,
  n        INTEGER NOT NULL,
  k        INTEGER NOT NULL CHECK (k >= 0),
  key      TEXT NOT NULL CHECK (key GLOB '[HG][1-9]*' AND key NOT GLOB '?*[^0-9]*'
                                AND length(key) BETWEEN 2 AND 6),
  PRIMARY KEY (verse_id, n, k),
  FOREIGN KEY (verse_id, n) REFERENCES word (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX word_key_by_key ON word_key (key, verse_id);
"#;

    // ---- Placeholder text. NOT SCRIPTURE. --------------------------------
    // Mini markup, only for building the fixture:
    //   [r]…[/r] words-of-Jesus span (may run across verses)
    //   [a]…[/a] supplied words      [d]…[/d] divine name
    //   [w H1 H2]…[/w] Strong's word link (empty = zero-length word)

    pub const TESTBOOK_1: &[&str] = &[
        "[w H1]Alpha[/w] [w H2]beta[/w] [a]gamma[/a] delta epsilon.",
        "Zeta eta theta, [r][w H3]iota[/w] kappa lambda.",
        "Mu nu xi[/r] omicron pi.",
        "Rho [r]sigma [w H4][d]Lord[/d][/w] [a]tau[/a] upsilon[/r] phi.",
        "Chi psi [r][w G1]cafe\u{301}[/w] \u{1D400}leph[/r] omega\u{2019}s end.",
        "Placeholder verse with a trailing space ",
        "[r]Whole verse red alpha beta.[/r]",
        "[r]Sigma one,[/r] said the placeholder narrator, [r]sigma two.[/r]",
        "[a][w H5]supplied[/w] [w H6]words[/w][/a] around [w H7 H8]two keys[/w] and an empty word[w H9][/w] here.",
        "Ten alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega, and once more alpha beta gamma delta.",
    ];

    fn testbook_2() -> Vec<String> {
        (1..=30)
            .map(|n| {
                let red_open = if n == 10 { "[r]" } else { "" };
                let red_close = if n == 12 { "[/r]" } else { "" };
                match n {
                    10 => format!("Placeholder verse {n} of Testbook chapter two, alpha beta {red_open}gamma delta epsilon."),
                    11 => format!("Placeholder verse {n} of Testbook chapter two, all of it inside one span."),
                    12 => format!("Placeholder verse {n} of Testbook chapter two{red_close}, zeta eta theta."),
                    _ => format!("Placeholder verse {n} of Testbook chapter two, alpha beta gamma delta epsilon zeta eta theta iota kappa."),
                }
            })
            .collect()
    }

    fn testbook_3() -> Vec<String> {
        (1..=12)
            .map(|n| {
                if n % 4 == 0 {
                    format!("Placeholder verse {n} of chapter three, the [d]Lord[/d] placeholder, alpha beta.")
                } else {
                    format!("Placeholder verse {n} of chapter three, alpha beta gamma.")
                }
            })
            .collect()
    }

    fn shortbook_1() -> Vec<String> {
        (1..=5)
            .map(|n| format!("Shortbook placeholder verse {n}: alpha beta [a]gamma[/a]."))
            .collect()
    }

    #[derive(Debug, Default, Clone)]
    pub struct Parsed {
        pub text: String,
        pub marks: Vec<(&'static str, u32, u32)>,
        pub words: Vec<(u32, u32, Vec<String>)>,
        /// (start, end) pieces; `None` end = still open at the verse end.
        pub red: Vec<(u32, Option<u32>)>,
        pub red_open_at_start: bool,
    }

    /// Parse one verse of markup. Returns (text, parsed). With `with_red =
    /// false`, `[r]` tags are dropped (the "no markup" translation).
    pub fn parse_markup(src: &str, with_red: bool) -> (String, Parsed) {
        parse_markup_state(src, with_red, false)
    }

    fn parse_markup_state(src: &str, with_red: bool, red_open: bool) -> (String, Parsed) {
        let mut p = Parsed {
            red_open_at_start: red_open && with_red,
            ..Default::default()
        };
        let mut cp = 0u32;
        let mut stack: Vec<(String, u32)> = Vec::new();
        if red_open && with_red {
            p.red.push((0, None));
        }
        let mut rest = src;
        while !rest.is_empty() {
            if let Some(stripped) = rest.strip_prefix('[') {
                let close = stripped.find(']').expect("unclosed tag");
                let tag = &stripped[..close];
                rest = &stripped[close + 1..];
                if let Some(name) = tag.strip_prefix('/') {
                    if name == "r" {
                        if with_red {
                            let last = p.red.last_mut().expect("[/r] without [r]");
                            assert!(last.1.is_none());
                            last.1 = Some(cp);
                        }
                        continue;
                    }
                    let (open, start) = stack.pop().expect("close without open");
                    assert_eq!(open.split(' ').next().unwrap(), name);
                    match name {
                        "a" => p.marks.push(("added", start, cp)),
                        "d" => p.marks.push(("divine", start, cp)),
                        "w" => {
                            let keys = open.split(' ').skip(1).map(String::from).collect();
                            p.words.push((start, cp, keys));
                        }
                        _ => panic!("unknown tag {name}"),
                    }
                } else if tag == "r" {
                    if with_red {
                        p.red.push((cp, None));
                    }
                } else {
                    stack.push((tag.to_string(), cp));
                }
            } else {
                let ch = rest.chars().next().unwrap();
                p.text.push(ch);
                cp += 1;
                rest = &rest[ch.len_utf8()..];
            }
        }
        assert!(stack.is_empty(), "unclosed tags in {src}");
        // marks are stored in document order of their start tags (P3)
        p.marks.sort_by_key(|m| m.1);
        p.words.sort_by_key(|w| w.0);
        (p.text.clone(), p)
    }

    struct BookSrc {
        ord: u32,
        osis: &'static str,
        usfm: &'static str,
        /// `book.main_title` (placeholder; NULL allowed for fixtures).
        main_title: Option<&'static str>,
        chapters: Vec<Vec<String>>,
    }

    fn books() -> Vec<BookSrc> {
        vec![
            BookSrc {
                ord: 1,
                osis: "Testbook",
                usfm: "TST",
                main_title: Some("PLACEHOLDER MAIN TITLE OF THE TESTBOOK"),
                chapters: vec![
                    TESTBOOK_1.iter().map(|s| s.to_string()).collect(),
                    testbook_2(),
                    testbook_3(),
                ],
            },
            BookSrc {
                ord: 40,
                osis: "Shortbook",
                usfm: "SHB",
                main_title: None,
                chapters: vec![shortbook_1()],
            },
        ]
    }

    fn header(conn: &Connection) {
        conn.execute_batch(&format!(
            "PRAGMA page_size = 4096; PRAGMA auto_vacuum = NONE; PRAGMA encoding = 'UTF-8';
             PRAGMA application_id = {APPLICATION_ID}; PRAGMA user_version = {SCHEMA_VERSION};
             PRAGMA foreign_keys = ON;"
        ))
        .unwrap();
    }

    fn finish(conn: Connection) -> Vec<u8> {
        let fk: Vec<String> = {
            let mut st = conn.prepare("PRAGMA foreign_key_check").unwrap();
            st.query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .map(|x| x.unwrap())
                .collect()
        };
        assert!(fk.is_empty(), "foreign key violations: {fk:?}");
        conn.execute_batch("VACUUM").unwrap();
        conn.serialize("main").unwrap().to_vec()
    }

    /// Build one placeholder Bible file in memory and return its bytes.
    pub fn build_bible(with_red: bool) -> Vec<u8> {
        let conn = Connection::open_in_memory().unwrap();
        header(&conn);
        conn.execute_batch(BIBLE_SCHEMA).unwrap();
        let (abbrev, name) = if with_red {
            (
                "PH1",
                "Placeholder fixture with red letters (not Scripture)",
            )
        } else {
            (
                "PH2",
                "Placeholder fixture without words-of-Jesus markup (not Scripture)",
            )
        };
        let tx = conn.unchecked_transaction().unwrap();
        for (k, v) in [
            ("schema_version", "1"),
            ("pipeline_commit", "unknown-dirty"),
            ("translation_abbrev", abbrev),
            ("translation_name", name),
            ("has_red_letters", if with_red { "1" } else { "0" }),
            ("placeholder_fixture", "1"),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![k, v],
            )
            .unwrap();
        }
        tx.execute(
            "INSERT INTO source (id, file, size, sha256) VALUES ('FIXTURE-PLACEHOLDER', 'generated by app/src-tauri/src/fixture.rs', 0, ?1)",
            [&"0".repeat(64)],
        )
        .unwrap();
        let mut q_ord = 0u32;
        for b in books() {
            tx.execute(
                "INSERT INTO book (ord, osis, usfm, chapters, main_title) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![b.ord, b.osis, b.usfm, b.chapters.len() as u32, b.main_title],
            )
            .unwrap();
            for (ci, verses) in b.chapters.iter().enumerate() {
                let c = ci as u32 + 1;
                let mut red_open = false;
                for (vi, src) in verses.iter().enumerate() {
                    let v = vi as u32 + 1;
                    let id = b.ord as i64 * 1_000_000 + c as i64 * 1000 + v as i64;
                    let (text, p) = parse_markup_state(src, with_red, red_open);
                    let len = text.chars().count() as u32;
                    tx.execute(
                        "INSERT INTO verse (id, book, chapter, verse, text) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![id, b.ord, c, v, text],
                    )
                    .unwrap();
                    for (n, (kind, s, e)) in p.marks.iter().enumerate() {
                        tx.execute(
                            "INSERT INTO mark (verse_id, n, kind, start_cp, end_cp) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![id, n as u32, kind, s, e],
                        )
                        .unwrap();
                    }
                    for (n, (s, e)) in p.red.iter().enumerate() {
                        // a piece that continues from the previous verse keeps its q_ord
                        if !(n == 0 && p.red_open_at_start) {
                            q_ord += 1;
                        }
                        let end = e.unwrap_or(len);
                        tx.execute(
                            "INSERT INTO red_piece (verse_id, n, start_cp, end_cp, q_ord) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![id, n as u32, s, end, q_ord],
                        )
                        .unwrap();
                    }
                    red_open = p.red.last().is_some_and(|r| r.1.is_none());
                    for (n, (s, e, keys)) in p.words.iter().enumerate() {
                        tx.execute(
                            "INSERT INTO word (verse_id, n, start_cp, end_cp) VALUES (?1, ?2, ?3, ?4)",
                            params![id, n as u32, s, e],
                        )
                        .unwrap();
                        for (k, key) in keys.iter().enumerate() {
                            tx.execute(
                                "INSERT INTO word_key (verse_id, n, k, key) VALUES (?1, ?2, ?3, ?4)",
                                params![id, n as u32, k as u32, key],
                            )
                            .unwrap();
                        }
                    }
                }
                assert!(!red_open, "red span left open at the end of a chapter");
            }
        }
        tx.commit().unwrap();
        finish(conn)
    }

    pub fn build_file(file: &str) -> Vec<u8> {
        match file {
            BIBLE_RED => build_bible(true),
            BIBLE_PLAIN => build_bible(false),
            _ => panic!("unknown fixture {file}"),
        }
    }

    // ---- Render golden for the UI test (vitest reads this JSON) ----------

    #[derive(Serialize)]
    struct GoldenVerse {
        v: u32,
        id: i64,
        text: String,
        /// Stored red pieces [start, end) in code points, straight from red_piece.
        red: Vec<[u32; 2]>,
        runs: Vec<crate::runs::Run>,
    }
    #[derive(Serialize)]
    struct GoldenChapter {
        book: u32,
        osis: String,
        chapter: u32,
        verses: Vec<GoldenVerse>,
    }
    #[derive(Serialize)]
    struct GoldenTranslation {
        abbrev: String,
        file: String,
        has_red_letters: bool,
        chapters: Vec<GoldenChapter>,
    }
    #[derive(Serialize)]
    struct Golden {
        note: &'static str,
        translations: Vec<GoldenTranslation>,
    }

    /// Render golden from the committed files, through open_verified and the
    /// same query code the app uses; red ranges read raw from red_piece.
    pub fn render_golden() -> String {
        let mut translations = Vec::new();
        for f in FILES.iter().filter(|f| f.kind == Kind::Bible) {
            let path = dir().join(f.file);
            let bible = Bible::open(&path, f.sha256).expect("fixture opens");
            let raw =
                Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .unwrap();
            let mut chapters = Vec::new();
            for b in bible.books().unwrap() {
                for c in 1..=b.chapters {
                    let ch = bible.chapter(b.ord, c).unwrap();
                    let verses = ch
                        .verses
                        .into_iter()
                        .map(|v| {
                            let id = b.ord as i64 * 1_000_000 + c as i64 * 1000 + v.v as i64;
                            let mut st = raw
                                .prepare("SELECT start_cp, end_cp FROM red_piece WHERE verse_id = ?1 ORDER BY n")
                                .unwrap();
                            let red = st
                                .query_map([id], |r| Ok([r.get(0)?, r.get(1)?]))
                                .unwrap()
                                .map(|x| x.unwrap())
                                .collect();
                            GoldenVerse { v: v.v, id, text: v.text, red, runs: v.runs }
                        })
                        .collect();
                    chapters.push(GoldenChapter {
                        book: b.ord,
                        osis: b.osis.clone(),
                        chapter: c,
                        verses,
                    });
                }
            }
            translations.push(GoldenTranslation {
                abbrev: bible.abbrev.clone(),
                file: f.file.to_string(),
                has_red_letters: bible.has_red_letters,
                chapters,
            });
        }
        let mut s = serde_json::to_string_pretty(&Golden {
            note: "GENERATED by `cargo test -- --ignored regenerate_fixture` (app/src-tauri). Placeholder text, NOT Scripture.",
            translations,
        })
        .unwrap();
        s.push('\n');
        s
    }

    pub const GOLDEN_FILE: &str = "placeholder-render.json";

    /// Rewrites the committed fixture files and the render golden. Run by hand
    /// after changing the fixture, then paste the printed hashes into FILES.
    #[test]
    #[ignore]
    fn regenerate_fixture() {
        std::fs::create_dir_all(dir()).unwrap();
        for f in FILES {
            let bytes = build_file(f.file);
            std::fs::write(dir().join(f.file), &bytes).unwrap();
            println!("{} {}", sha256_hex(&bytes), f.file);
        }
        // the golden needs the new hashes compiled in; written only when they match
        if FILES
            .iter()
            .all(|f| sha256_hex(&std::fs::read(dir().join(f.file)).unwrap()) == f.sha256)
        {
            std::fs::write(dir().join(GOLDEN_FILE), render_golden()).unwrap();
            println!("wrote {GOLDEN_FILE}");
        } else {
            println!(
                "update FILES with the hashes above, rebuild, and run this again to write {GOLDEN_FILE}"
            );
        }
    }

    #[test]
    fn fixture_files_are_up_to_date() {
        for f in FILES {
            let fresh = build_file(f.file);
            // deterministic: two builds give the same bytes
            assert_eq!(fresh, build_file(f.file), "{} is not deterministic", f.file);
            assert_eq!(
                sha256_hex(&fresh),
                f.sha256,
                "rebuild of {} differs from the compiled-in hash",
                f.file
            );
            let committed = std::fs::read(dir().join(f.file)).unwrap();
            assert_eq!(
                sha256_hex(&committed),
                f.sha256,
                "committed {} differs",
                f.file
            );
            // DELETE journal mode in the header (bytes 18/19 = 01 01), never WAL
            assert_eq!(&committed[18..20], &[1, 1]);
        }
        let golden = std::fs::read_to_string(dir().join(GOLDEN_FILE)).unwrap();
        assert_eq!(
            golden.replace("\r\n", "\n"),
            render_golden(),
            "{GOLDEN_FILE} is stale"
        );
    }

    #[test]
    fn fixture_passes_the_design_check_queries() {
        // DESIGN.md 8: range-end check, R1, R3, R6 and q_ord gaps must return no rows.
        let checks = [
            "SELECT 'mark' AS t, verse_id, n FROM mark JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
             UNION ALL SELECT 'red_piece', verse_id, n FROM red_piece JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
             UNION ALL SELECT 'word', verse_id, n FROM word JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)",
            "SELECT 'R1', r.verse_id, r.n FROM red_piece r JOIN verse v ON v.id = r.verse_id
             WHERE NOT (r.start_cp < r.end_cp AND r.end_cp <= length(v.text))",
            "SELECT 'R3', a.verse_id, a.n FROM red_piece a
               JOIN red_piece b ON b.verse_id = a.verse_id AND b.n = a.n + 1
              WHERE b.start_cp <= a.end_cp
             UNION ALL
             SELECT 'R3', verse_id, NULL FROM red_piece GROUP BY verse_id
             HAVING min(n) <> 0 OR max(n) <> count(*) - 1",
            "WITH p AS (
               SELECT r.q_ord, r.verse_id, r.start_cp, r.end_cp, length(v.text) AS len,
                      lag(r.verse_id) OVER w AS prev_v, row_number() OVER w AS i,
                      count(*) OVER (PARTITION BY r.q_ord) AS k
                 FROM red_piece r JOIN verse v ON v.id = r.verse_id
               WINDOW w AS (PARTITION BY r.q_ord ORDER BY r.verse_id))
             SELECT 'R6', verse_id, q_ord FROM p
              WHERE (i > 1 AND start_cp <> 0) OR (i < k AND end_cp <> len)
                 OR (i > 1 AND (prev_v = verse_id
                                OR EXISTS (SELECT 1 FROM verse WHERE id > p.prev_v AND id < p.verse_id)))",
            // DESIGN.md 8 writes this one without aggregates in the result columns, which
            // SQLite rejects ("HAVING clause on a non-aggregate query"); same check:
            "SELECT 'q_ord', count(DISTINCT q_ord), max(q_ord) FROM red_piece HAVING count(DISTINCT q_ord) <> max(q_ord)",
        ];
        for file in [BIBLE_RED, BIBLE_PLAIN] {
            let conn = Connection::open_in_memory().unwrap();
            let bytes = build_file(file);
            let mut conn = conn;
            conn.deserialize_read_exact("main", &bytes[..], bytes.len(), true)
                .unwrap();
            for sql in checks {
                let n: i64 = conn
                    .query_row(&format!("SELECT count(*) FROM ({sql})"), [], |r| r.get(0))
                    .unwrap();
                assert_eq!(n, 0, "{file}: {sql}");
            }
            let pieces: i64 = conn
                .query_row("SELECT count(*) FROM red_piece", [], |r| r.get(0))
                .unwrap();
            if file == BIBLE_RED {
                assert!(pieces >= 10);
                // the cross-verse spans share one q_ord
                let shared: i64 = conn
                    .query_row("SELECT count(*) FROM (SELECT q_ord FROM red_piece GROUP BY q_ord HAVING count(*) > 1)", [], |r| {
                        r.get(0)
                    })
                    .unwrap();
                assert_eq!(shared, 2);
            } else {
                assert_eq!(pieces, 0);
            }
        }
    }

    #[test]
    fn placeholder_text_only() {
        // Guard: fixture text stays obvious placeholder filler.
        let mut open = false;
        for src in TESTBOOK_1 {
            let (t, p) = parse_markup_state(src, true, open);
            assert!(!t.is_empty() && !t.contains('['));
            open = p.red.last().is_some_and(|r| r.1.is_none());
        }
        assert!(!open);
        let (t, p) = parse_markup("[w H1]a[/w] [r]b[/r]", true);
        assert_eq!(t, "a b");
        assert_eq!(p.words, vec![(0, 1, vec!["H1".to_string()])]);
        assert_eq!(p.red, vec![(2, Some(3))]);
    }
}
