//! Tests against the real pipeline output, `pipeline/out/kjv.sqlite`, opened
//! only through `open_verified` with the SHA-256 compiled in from
//! `pipeline/tests/expected/output-sha256.txt` (so they follow a rebuilt DB
//! automatically once that file changes).
//!
//! - every verse: runs concatenate to the stored text; red runs = `red_piece`
//!   rows; `add`/`nd` runs = `mark` rows; `.w` runs = `word`/`word_key` rows;
//! - the repair/reinstall path: a wrong or stale hash never yields text;
//! - the render golden `tests/golden/kjv-render.json` (machine-generated, read
//!   by the vitest render, copy and Strong's tests) is up to date.
//!
//! Regenerate the golden (one command, in app/src-tauri):
//! `cargo test -- --ignored regenerate_kjv_render_golden`

use crate::commands::{self, AppState, KJV_FILE};
use crate::content::{self, Bible, MAX_CONTENT_BYTES};
use crate::expected;
use crate::runs::Run;
use crate::runs::tests::red_from_runs;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;

fn kjv_path() -> PathBuf {
    commands::content_dir().unwrap().join(KJV_FILE)
}

fn open_kjv() -> Bible {
    Bible::open(&kjv_path(), expected::KJV_SQLITE).unwrap_or_else(|e| {
        panic!(
            "{e}. pipeline/out/kjv.sqlite must match pipeline/tests/expected/output-sha256.txt \
             (rebuild it with the pipeline, or rebuild the app after that file changed)"
        )
    })
}

/// A second, independent verified copy for raw table reads.
fn raw_kjv() -> Connection {
    content::open_content(&kjv_path(), expected::KJV_SQLITE, MAX_CONTENT_BYTES).unwrap()
}

type Ranges = BTreeMap<i64, Vec<(u32, u32)>>;
/// word n -> (start, end, keys)
type Words = BTreeMap<u32, (u32, u32, Vec<String>)>;

fn ranges(raw: &Connection, sql: &str) -> Ranges {
    let mut out: Ranges = BTreeMap::new();
    let mut st = raw.prepare(sql).unwrap();
    let rows = st
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?)))
        .unwrap();
    for row in rows {
        let (vid, s, e) = row.unwrap();
        out.entry(vid).or_default().push((s, e));
    }
    out
}

/// Code-point positions covered by runs where `flag` holds.
fn covered_by_runs(runs: &[Run], flag: impl Fn(&Run) -> bool) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    let mut pos = 0u32;
    for r in runs {
        let l = r.text.chars().count() as u32;
        if flag(r) {
            out.extend(pos..pos + l);
        }
        pos += l;
    }
    out
}

fn covered_by_rows(rows: Option<&Vec<(u32, u32)>>) -> BTreeSet<u32> {
    rows.into_iter()
        .flatten()
        .flat_map(|&(s, e)| s..e)
        .collect()
}

/// `.w` words as reassembled from runs: n -> (start, end, keys).
fn words_from_runs(runs: &[Run]) -> Words {
    let mut out: Words = BTreeMap::new();
    let mut pos = 0u32;
    for r in runs {
        let l = r.text.chars().count() as u32;
        if let Some(n) = r.wn {
            let keys = r.w.clone().expect("word run without keys");
            match out.get_mut(&n) {
                Some(w) => {
                    assert_eq!(w.1, pos, "word {n} is not contiguous");
                    assert_eq!(w.2, keys, "word {n} keys differ between its runs");
                    w.1 = pos + l;
                }
                None => {
                    out.insert(n, (pos, pos + l, keys));
                }
            }
        } else {
            assert!(r.w.is_none());
        }
        pos += l;
    }
    out
}

#[test]
fn real_kjv_meta_and_schema_assumptions() {
    let bible = open_kjv();
    assert_eq!(bible.abbrev, "KJV");
    assert!(bible.has_red_letters, "meta has_red_letters must be '1'");
    let raw = raw_kjv();
    let meta: BTreeMap<String, String> = raw
        .prepare("SELECT key, value FROM meta")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    for k in [
        "has_red_letters",
        "red_letter_source",
        "schema_version",
        "translation_abbrev",
        "versification",
    ] {
        assert!(meta.contains_key(k), "meta lacks {k}");
    }
    assert_eq!(meta["has_red_letters"], "1");
    assert_eq!(meta["schema_version"], "1");
    // No reader-facing name in the DB yet (DESIGN.md 8): the app takes the
    // owner-approved label from content::TRANSLATION_LABELS (kjv-label.md).
    // Once the pipeline writes translation_name, it must say the same.
    assert_eq!(bible.abbrev, "KJV");
    assert_eq!(bible.name, "King James Version (1769)");
    if let Some(n) = meta.get("translation_name") {
        assert_eq!(n, "King James Version (1769)");
    }
    println!("meta keys: {:?}", meta.keys().collect::<Vec<_>>());

    let books = bible.books().unwrap();
    assert_eq!(books.len(), 66);
    for (i, b) in books.iter().enumerate() {
        assert_eq!(b.ord as usize, i + 1);
        // every OSIS ID in the real file is in the provisional books.rs table
        assert!(!b.abbrevs.is_empty(), "{} not in books.rs", b.osis);
        assert!(b.verses.iter().all(|&v| v > 0), "{}: empty chapter", b.osis);
    }
    let chapters: u32 = books.iter().map(|b| b.chapters).sum();
    assert_eq!(chapters, 1189);
}

/// The full-Bible run check (task D4a.2).
#[test]
fn every_kjv_verse_runs_equal_stored_rows() {
    let bible = open_kjv();
    let raw = raw_kjv();
    let texts: BTreeMap<i64, String> = raw
        .prepare("SELECT id, text FROM verse")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    let mut red_rows: BTreeMap<i64, Vec<(u32, u32, u32)>> = BTreeMap::new();
    for row in raw
        .prepare("SELECT verse_id, n, start_cp, end_cp FROM red_piece ORDER BY verse_id, n")
        .unwrap()
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap()
    {
        let (vid, n, s, e) = row.unwrap();
        red_rows.entry(vid).or_default().push((n, s, e));
    }
    let added = ranges(
        &raw,
        "SELECT verse_id, start_cp, end_cp FROM mark WHERE kind = 'added' ORDER BY verse_id, n",
    );
    let divine = ranges(
        &raw,
        "SELECT verse_id, start_cp, end_cp FROM mark WHERE kind = 'divine' ORDER BY verse_id, n",
    );
    let mut word_rows: BTreeMap<i64, Words> = BTreeMap::new();
    for row in raw
        .prepare("SELECT verse_id, n, start_cp, end_cp FROM word ORDER BY verse_id, n")
        .unwrap()
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap()
    {
        let (vid, n, s, e) = row.unwrap();
        word_rows
            .entry(vid)
            .or_default()
            .insert(n, (s, e, Vec::new()));
    }
    let mut key_rows = 0u64;
    for row in raw
        .prepare("SELECT verse_id, n, key FROM word_key ORDER BY verse_id, n, k")
        .unwrap()
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .unwrap()
    {
        let (vid, n, key) = row.unwrap();
        word_rows
            .get_mut(&vid)
            .and_then(|w| w.get_mut(&n))
            .expect("word_key without word")
            .2
            .push(key);
        key_rows += 1;
    }
    let mut sql_substr = raw
        .prepare("SELECT substr(text, ?2 + 1, ?3 - ?2) FROM verse WHERE id = ?1")
        .unwrap();

    let (mut verses, mut pieces, mut red_cp, mut red_verses) = (0u64, 0u64, 0u64, 0u64);
    let (mut add_marks, mut nd_marks, mut words, mut empty_words, mut runs_total) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut seen: BTreeSet<i64> = BTreeSet::new();
    for b in bible.books().unwrap() {
        for c in 1..=b.chapters {
            let ch = bible.chapter(b.ord, c).unwrap();
            assert_eq!(ch.verses.len() as u32, b.verses[(c - 1) as usize]);
            for v in &ch.verses {
                let vid = b.ord as i64 * 1_000_000 + c as i64 * 1000 + v.v as i64;
                assert!(seen.insert(vid));
                let stored = &texts[&vid];
                let joined: String = v.runs.iter().map(|r| r.text.as_str()).collect();
                assert_eq!(&joined, stored, "verse {vid}: runs differ from stored text");
                assert_eq!(&v.text, stored, "verse {vid}");
                assert!(v.runs.iter().all(|r| !r.text.is_empty()), "verse {vid}");

                // red: runs carry exactly the stored pieces (n, start, end)
                let rows = red_rows.get(&vid).cloned().unwrap_or_default();
                assert_eq!(red_from_runs(&v.runs), rows, "verse {vid}: red differs");
                for &(n, s, e) in &rows {
                    let sql: String = sql_substr
                        .query_row(params![vid, s, e], |r| r.get(0))
                        .unwrap();
                    let ui: String = v
                        .runs
                        .iter()
                        .filter(|r| r.rp == Some(n))
                        .map(|r| r.text.as_str())
                        .collect();
                    assert_eq!(ui, sql, "verse {vid} piece {n}");
                    red_cp += (e - s) as u64;
                }
                pieces += rows.len() as u64;
                red_verses += u64::from(!rows.is_empty());

                // supplied words and divine name: same code points as the rows
                assert_eq!(
                    covered_by_runs(&v.runs, |r| r.add),
                    covered_by_rows(added.get(&vid)),
                    "verse {vid}: added"
                );
                assert_eq!(
                    covered_by_runs(&v.runs, |r| r.nd),
                    covered_by_rows(divine.get(&vid)),
                    "verse {vid}: divine"
                );
                add_marks += added.get(&vid).map_or(0, |x| x.len() as u64);
                nd_marks += divine.get(&vid).map_or(0, |x| x.len() as u64);

                // words: every non-empty word row is one `.w` with its keys
                let want: Words = word_rows
                    .get(&vid)
                    .map(|w| {
                        w.iter()
                            .filter(|(_, x)| x.0 < x.1)
                            .map(|(n, x)| (*n, x.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                assert_eq!(words_from_runs(&v.runs), want, "verse {vid}: words");
                words += want.len() as u64;
                empty_words += word_rows
                    .get(&vid)
                    .map_or(0, |w| w.values().filter(|x| x.0 == x.1).count() as u64);
                runs_total += v.runs.len() as u64;
                verses += 1;
            }
        }
    }
    assert_eq!(
        seen.len(),
        texts.len(),
        "verses outside the book/chapter walk"
    );
    println!(
        "KJV run check: {verses} verses, {pieces} red pieces in {red_verses} verses, \
         {red_cp} red code points, {add_marks} added marks, {nd_marks} divine-name marks, \
         {words} words rendered as .w ({empty_words} zero-length words not rendered), \
         {key_rows} word_key rows, {runs_total} runs"
    );
    assert_eq!(verses, 31_102);
    assert_eq!(pieces, 2_035);
    assert_eq!(red_cp, 210_502);
}

/// A hash that doesn't match (damaged file, or the DB was rebuilt and the app
/// wasn't): no Bible, the UI's repair/reinstall state, and no command returns
/// text.
#[test]
fn stale_or_wrong_hash_gives_the_repair_state_and_no_text() {
    let dir = commands::content_dir().unwrap();
    let mut stale = expected::KJV_SQLITE.to_string();
    let last = if stale.ends_with('0') { "1" } else { "0" };
    stale.replace_range(63..64, last);

    let tmp = crate::fixture::temp_dir("kjv-tamper");
    let mut bytes = std::fs::read(dir.join(KJV_FILE)).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x01;
    std::fs::write(tmp.join(KJV_FILE), &bytes).unwrap();
    drop(bytes);

    for (d, h) in [
        (dir.clone(), stale.as_str()),
        (tmp.clone(), expected::KJV_SQLITE),
        (tmp.join("missing"), expected::KJV_SQLITE),
    ] {
        let err = match commands::load_content_from(&d, h, expected::CONTENT_SQLITE) {
            Ok(_) => panic!("content opened with a wrong hash or file"),
            Err(e) => e,
        };
        let state = AppState {
            content: Mutex::new(Err(err.clone())),
            user: Mutex::new(Err("not used".into())),
        };
        let info = commands::info(&state);
        assert_eq!(info.content_error.as_deref(), Some(err.as_str()));
        assert!(info.translations.is_empty());
        assert!(commands::with_bible(&state, "KJV", |b| b.chapter(43, 3)).is_err());
    }
    // and the real file with the right hash does open
    assert!(
        commands::load_content_from(&dir, expected::KJV_SQLITE, expected::CONTENT_SQLITE).is_ok()
    );
}

// ---------------------------------------------------------------------------
// Render golden for the UI tests
// ---------------------------------------------------------------------------

/// (book ord, chapter): tricky red boundaries (mid-verse starts and ends,
/// cross-verse speeches, narration inside speech, John 3:16-21), plus Psalm 23
/// for the divine name (LORD small capitals), which these NT chapters lack.
const GOLDEN_CHAPTERS: &[(u32, u32)] = &[(43, 3), (40, 5), (41, 8), (66, 1), (66, 22), (19, 23)];
const GOLDEN_FILE: &str = "tests/golden/kjv-render.json";

#[derive(Serialize)]
struct GVerse<'a> {
    v: u32,
    id: i64,
    text: &'a str,
    red: Vec<(u32, u32)>,
    add: Vec<(u32, u32)>,
    nd: Vec<(u32, u32)>,
    /// non-empty words: [start, end, "keys"]
    words: Vec<(u32, u32, String)>,
    runs: &'a [Run],
}

fn render_golden() -> String {
    let bible = open_kjv();
    let raw = raw_kjv();
    let q = |sql: &str, vid: i64| -> Vec<(u32, u32)> {
        raw.prepare(sql)
            .unwrap()
            .query_map([vid], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    };
    let books = bible.books().unwrap();
    let mut s = String::from("{\n");
    s.push_str("\"note\": \"GENERATED from pipeline/out/kjv.sqlite through open_verified (expected SHA-256 compiled in from pipeline/tests/expected/output-sha256.txt). Do not edit. Regenerate in app/src-tauri: cargo test -- --ignored regenerate_kjv_render_golden\",\n");
    s.push_str(&format!(
        "\"translation\": {},\n",
        serde_json::json!({"abbrev": bible.abbrev, "name": bible.name, "has_red_letters": bible.has_red_letters})
    ));
    s.push_str(&format!(
        "\"books\": {},\n",
        serde_json::to_string(&books).unwrap()
    ));
    s.push_str("\"chapters\": [\n");
    for (i, &(book, chapter)) in GOLDEN_CHAPTERS.iter().enumerate() {
        let ch = bible.chapter(book, chapter).unwrap();
        let b = &books[(book - 1) as usize];
        s.push_str(&format!(
            "{{\"book\": {book}, \"osis\": {}, \"name\": {}, \"chapter\": {chapter}, \"verses\": [\n",
            serde_json::to_string(&b.osis).unwrap(),
            serde_json::to_string(&b.name).unwrap()
        ));
        for (j, v) in ch.verses.iter().enumerate() {
            let id = book as i64 * 1_000_000 + chapter as i64 * 1000 + v.v as i64;
            let text: String = raw
                .query_row("SELECT text FROM verse WHERE id = ?1", [id], |r| r.get(0))
                .unwrap();
            let mut words = Vec::new();
            let mut wst = raw
                .prepare("SELECT n, start_cp, end_cp FROM word WHERE verse_id = ?1 AND start_cp < end_cp ORDER BY n")
                .unwrap();
            let wrows: Vec<(u32, u32, u32)> = wst
                .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(|x| x.unwrap())
                .collect();
            for (n, ws, we) in wrows {
                let keys: Vec<String> = raw
                    .prepare("SELECT key FROM word_key WHERE verse_id = ?1 AND n = ?2 ORDER BY k")
                    .unwrap()
                    .query_map(params![id, n], |r| r.get(0))
                    .unwrap()
                    .map(|x| x.unwrap())
                    .collect();
                words.push((ws, we, keys.join(" ")));
            }
            let gv = GVerse {
                v: v.v,
                id,
                text: &text,
                red: q(
                    "SELECT start_cp, end_cp FROM red_piece WHERE verse_id = ?1 ORDER BY n",
                    id,
                ),
                add: q(
                    "SELECT start_cp, end_cp FROM mark WHERE verse_id = ?1 AND kind = 'added' ORDER BY n",
                    id,
                ),
                nd: q(
                    "SELECT start_cp, end_cp FROM mark WHERE verse_id = ?1 AND kind = 'divine' ORDER BY n",
                    id,
                ),
                words,
                runs: &v.runs,
            };
            s.push_str(&serde_json::to_string(&gv).unwrap());
            s.push_str(if j + 1 < ch.verses.len() { ",\n" } else { "\n" });
        }
        s.push_str(if i + 1 < GOLDEN_CHAPTERS.len() {
            "]},\n"
        } else {
            "]}\n"
        });
    }
    s.push_str("]\n}\n");
    s
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_FILE)
}

#[test]
#[ignore]
fn regenerate_kjv_render_golden() {
    let p = golden_path();
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, render_golden()).unwrap();
    println!("wrote {}", p.display());
}

#[test]
fn kjv_render_golden_is_up_to_date() {
    let committed = std::fs::read_to_string(golden_path())
        .expect("tests/golden/kjv-render.json missing: run the regenerate command")
        .replace("\r\n", "\n");
    assert!(
        committed == render_golden(),
        "{GOLDEN_FILE} differs from the DB: run `cargo test -- --ignored regenerate_kjv_render_golden` \
         and review the diff"
    );
    // it really contains mid-verse red boundaries on both sides and a cross-verse speech
    let v: serde_json::Value = serde_json::from_str(&committed).unwrap();
    let verses: Vec<&serde_json::Value> = v["chapters"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["verses"].as_array().unwrap())
        .collect();
    let len = |x: &serde_json::Value| x["text"].as_str().unwrap().chars().count() as u64;
    let red = |x: &serde_json::Value| x["red"].as_array().unwrap().clone();
    assert!(
        verses
            .iter()
            .any(|x| red(x).iter().any(|r| r[0].as_u64().unwrap() > 0))
    );
    assert!(
        verses
            .iter()
            .any(|x| red(x).iter().any(|r| r[1].as_u64().unwrap() < len(x)))
    );
    assert!(verses.iter().any(|x| red(x).len() >= 2));
    assert!(verses.iter().any(|x| red(x).is_empty()));
}

/// All 66 books get their name from the scholar's table (no OSIS fallback),
/// and `main_title` is exactly the stored `book.main_title`.
#[test]
fn every_book_has_a_cited_name_and_its_stored_main_title() {
    let bible = open_kjv();
    let raw = raw_kjv();
    let books = bible.books().unwrap();
    assert_eq!(books.len(), 66);
    for b in &books {
        assert!(!b.abbrevs.is_empty(), "{} has no table row", b.osis);
        let stored: Option<String> = raw
            .query_row("SELECT main_title FROM book WHERE ord = ?1", [b.ord], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(stored.is_some(), "{}", b.osis);
        assert_eq!(b.main_title, stored, "{}", b.osis);
    }
    assert_eq!(
        (books[18].name.as_str(), books[18].heading.as_str()),
        ("Psalms", "Psalm")
    );
    assert_eq!(books[8].name, "1 Samuel");
}

/// D5-F6: highlights and saved positions are stored only for verses and
/// chapters that exist in the loaded translation (OSIS IDs, case-sensitive).
#[test]
fn highlights_and_positions_must_name_real_verses() {
    use crate::user::{Position, SavedState, Settings, UserDb};
    let tmp = crate::fixture::temp_dir("kjv-refs");
    let state = AppState {
        content: Mutex::new(commands::load_content()),
        user: Mutex::new(Ok(UserDb::open(&tmp.join("user.sqlite")).unwrap())),
    };
    commands::highlight(&state, "John.3.16", Some("gold")).unwrap();
    commands::highlight(&state, "John.3.36", Some("gold")).unwrap(); // last verse
    commands::highlight(&state, "John.3.16", None).unwrap();
    for bad in [
        "Zzzz.999.999",
        "JOHN.3.16",
        "john.3.16",
        "John.3.37",
        "John.22.1",
        "John.3",
    ] {
        assert!(
            commands::highlight(&state, bad, Some("gold")).is_err(),
            "{bad}"
        );
    }
    let pos = |book: &str, chapter: u32, cv: Option<u32>| SavedState {
        position: Some(Position {
            translation: "KJV".into(),
            book: book.into(),
            chapter,
            anchor_verse: Some(1),
            anchor_frac: Some(0.0),
            current_verse: cv,
        }),
        settings: Settings::default(),
    };
    commands::save(&state, &pos("Rev", 22, Some(21))).unwrap();
    for bad in [
        pos("Nonbook", 999, None),
        pos("Rev", 23, None),
        pos("Rev", 22, Some(22)),
        pos("rev", 22, None),
    ] {
        assert!(commands::save(&state, &bad).is_err(), "{bad:?}");
    }
    let mut other = pos("Rev", 22, None);
    other.position.as_mut().unwrap().translation = "XYZ".into();
    assert!(commands::save(&state, &other).is_err());
    // with no position, settings still save
    commands::save(
        &state,
        &SavedState {
            position: None,
            settings: Settings::default(),
        },
    )
    .unwrap();
    // with the content refused, nothing can be referenced
    let no_content = AppState {
        content: Mutex::new(Err("damaged".into())),
        user: Mutex::new(Ok(UserDb::open(&tmp.join("user2.sqlite")).unwrap())),
    };
    assert!(commands::highlight(&no_content, "John.3.16", Some("gold")).is_err());
}

/// W2-BM: bookmarks are stored only for chapters of the loaded translation;
/// clear returns the row and restore puts it back exactly.
#[test]
fn bookmarks_must_name_real_chapters() {
    use crate::user::{Bookmark, UserDb};
    let tmp = crate::fixture::temp_dir("kjv-bm");
    let state = AppState {
        content: Mutex::new(commands::load_content()),
        user: Mutex::new(Ok(UserDb::open(&tmp.join("user.sqlite")).unwrap())),
    };
    commands::set_bookmark(&state, "Luke", 12).unwrap();
    commands::set_bookmark(&state, "Ps", 150).unwrap();
    for (book, ch) in [
        ("Luke", 25),
        ("Ps", 151),
        ("luke", 1),
        ("Zzzz", 1),
        ("Luke", 0),
        ("Lu ke", 1),
    ] {
        assert!(
            commands::set_bookmark(&state, book, ch).is_err(),
            "{book} {ch}"
        );
    }
    let old = commands::clear_bookmark(&state, "Luke").unwrap().unwrap();
    assert_eq!((old.book.as_str(), old.chapter), ("Luke", 12));
    assert_eq!(commands::clear_bookmark(&state, "Luke").unwrap(), None);
    commands::restore_bookmark(&state, &old).unwrap();
    let bad = Bookmark {
        book: "Luke".into(),
        chapter: 99,
        read_at: old.read_at,
    };
    assert!(commands::restore_bookmark(&state, &bad).is_err());
    let rows = {
        let g = state.user.lock().unwrap();
        g.as_ref().unwrap().bookmarks().unwrap()
    };
    assert_eq!(rows.len(), 2);
    assert!(rows.contains(&old));
}

/// D5-F5 through the command layer: a working file is never set aside; a
/// refused one is kept and a new one started.
#[test]
fn start_new_user_data_only_after_a_refusal() {
    use crate::user::UserDb;
    let tmp = crate::fixture::temp_dir("kjv-start-new");
    let ok = AppState {
        content: Mutex::new(Err("not used".into())),
        user: Mutex::new(Ok(UserDb::open(&tmp.join("user.sqlite")).unwrap())),
    };
    assert!(commands::start_new_user(&ok, &tmp).is_err());
    drop(ok);
    std::fs::write(
        tmp.join("user.sqlite"),
        b"not a database, placeholder bytes .............",
    )
    .unwrap();
    let refused = commands::open_user(&tmp);
    assert!(refused.is_err());
    let state = AppState {
        content: Mutex::new(Err("not used".into())),
        user: Mutex::new(refused.map(|_| unreachable!())),
    };
    let kept = commands::start_new_user(&state, &tmp).unwrap();
    assert!(kept.starts_with("user-kept-") && !kept.contains(['\\', '/']));
    assert!(tmp.join(&kept).exists() && tmp.join("user.sqlite").exists());
    assert!(commands::info(&state).user_error.is_none());
}

/// W2A-U1 through the command layer: a newer build's file (schema v5) is
/// reported as too new and the command refuses to set it aside.
#[test]
fn start_new_user_data_refuses_a_newer_builds_file() {
    use crate::user::{USER_SCHEMA_VERSION, UserDb};
    let tmp = crate::fixture::temp_dir("kjv-too-new");
    drop(UserDb::open(&tmp.join("user.sqlite")).unwrap());
    rusqlite::Connection::open(tmp.join("user.sqlite"))
        .unwrap()
        .execute_batch(&format!(
            "PRAGMA user_version = {}",
            USER_SCHEMA_VERSION + 1
        ))
        .unwrap();
    let bytes = std::fs::read(tmp.join("user.sqlite")).unwrap();
    let refused = commands::open_user(&tmp);
    let state = AppState {
        content: Mutex::new(Err("not used".into())),
        user: Mutex::new(refused.map(|_| unreachable!())),
    };
    let info = commands::info(&state);
    assert!(info.user_too_new);
    assert!(info.user_error.unwrap().contains("newer version"));
    let e = commands::start_new_user(&state, &tmp).unwrap_err();
    assert!(e.contains("newer version"), "{e}");
    assert_eq!(std::fs::read(tmp.join("user.sqlite")).unwrap(), bytes);
    assert!(std::fs::read_dir(&tmp).unwrap().all(|f| {
        !f.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("user-kept-")
    }));
    // a damaged file is not "too new": the button stays for it
    let tmp2 = crate::fixture::temp_dir("kjv-damaged");
    std::fs::write(
        tmp2.join("user.sqlite"),
        b"not a database, placeholder bytes .............",
    )
    .unwrap();
    let damaged = AppState {
        content: Mutex::new(Err("not used".into())),
        user: Mutex::new(commands::open_user(&tmp2).map(|_| unreachable!())),
    };
    assert!(!commands::info(&damaged).user_too_new);
}
