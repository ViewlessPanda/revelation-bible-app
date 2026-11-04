//! W2-5h: the pen-highlight IPC layer against the placeholder fixture (two
//! translations, PH1 with red letters, PH2 without). The backend is the gate:
//! translation loaded (exact id), verse exists, offsets in bounds in code
//! points, B1, contiguity; lengths and fingerprints computed here.

use crate::commands::{self, AppState, Content};
use crate::content::Bible;
use crate::fixture::{self, temp_dir};
use crate::marks::{self, MarkIn, Piece};
use crate::user::UserDb;
use std::sync::Mutex;

fn state(name: &str) -> AppState {
    let bibles = fixture::FILES
        .iter()
        .map(|f| Bible::open(&fixture::dir().join(f.file), f.sha256).unwrap())
        .collect();
    AppState {
        content: Mutex::new(Ok(Content {
            bibles,
            lexicon: Err("not used".into()),
            search: Err("not used".into()),
            commentary: Err("not used".into()),
        })),
        user: Mutex::new(Ok(
            UserDb::open(&temp_dir(name).join("user.sqlite")).unwrap()
        )),
    }
}

fn pc(verse: u32, start: u32, end: u32) -> Piece {
    Piece { verse, start, end }
}

fn apply(
    s: &AppState,
    tr: &str,
    book: &str,
    ch: u32,
    color: &str,
    p: &[Piece],
) -> Result<Vec<marks::MarkOut>, String> {
    commands::with_marks(s, tr, book, ch, |db, k, t| db.pen_apply(k, t, color, p))
}

fn all(s: &AppState, tr: &str) -> Vec<marks::MarkOut> {
    commands::with_marks(s, tr, "Testbook", 1, |db, k, t| db.marks(k, t)).unwrap()
}

#[test]
fn wrong_translation_book_or_verse_is_refused_and_nothing_is_written() {
    let s = state("pen-cmd-wrong");
    let ok = [pc(1, 0, 5)]; // "Alpha"
    for tr in [
        "", "ph1", "Ph1", "KJV", "PH3", "../PH1", "PH1'--", "PH1 ", "PH1\u{0}",
    ] {
        assert!(apply(&s, tr, "Testbook", 1, "gold", &ok).is_err(), "{tr:?}");
    }
    for book in ["testbook", "Testbook ", "Shortbook2", "", "Gen"] {
        assert!(apply(&s, "PH1", book, 1, "gold", &ok).is_err(), "{book:?}");
    }
    for ch in [0, 4, 999, 1000] {
        assert!(
            apply(&s, "PH1", "Testbook", ch, "gold", &ok).is_err(),
            "{ch}"
        );
    }
    // Testbook 1 has 10 verses: verse 11 doesn't exist
    assert!(apply(&s, "PH1", "Testbook", 1, "gold", &[pc(11, 0, 1)]).is_err());
    assert!(all(&s, "PH1").is_empty());
    assert!(all(&s, "PH2").is_empty());
    // the same call with the right id works, and only in that translation
    apply(&s, "PH1", "Testbook", 1, "gold", &ok).unwrap();
    assert_eq!(all(&s, "PH1").len(), 1);
    assert!(
        all(&s, "PH2").is_empty(),
        "a range mark shows only in its own translation"
    );
}

#[test]
fn offsets_are_checked_against_the_stored_text_in_code_points() {
    let s = state("pen-cmd-bounds");
    // Testbook 1:5 "Chi psi café 𝐀leph omega’s end." with U+0301 and an astral letter
    let texts = commands::chapter_texts(&s, "PH1", "Testbook", 1).unwrap();
    let v5: Vec<char> = texts[4].1.chars().collect();
    let acute = v5.iter().position(|&c| c == '\u{301}').unwrap() as u32;
    let astral = v5.iter().position(|&c| c == '\u{1D400}').unwrap() as u32;
    let len = v5.len() as u32;
    assert!(
        texts[4].1.encode_utf16().count() as u32 > len,
        "UTF-16 is longer than code points"
    );
    // a boundary before the combining acute is refused (B1)
    assert!(apply(&s, "PH1", "Testbook", 1, "gold", &[pc(5, acute - 4, acute)]).is_err());
    // the whole cluster is fine
    apply(
        &s,
        "PH1",
        "Testbook",
        1,
        "gold",
        &[pc(5, acute - 4, acute + 1)],
    )
    .unwrap();
    // "𝐀leph" in code points
    apply(
        &s,
        "PH1",
        "Testbook",
        1,
        "blue",
        &[pc(5, astral, astral + 5)],
    )
    .unwrap();
    // the UTF-16 length is out of bounds
    let utf16 = texts[4].1.encode_utf16().count() as u32;
    assert!(apply(&s, "PH1", "Testbook", 1, "rose", &[pc(5, 0, utf16)]).is_err());
    assert!(apply(&s, "PH1", "Testbook", 1, "rose", &[pc(5, 0, len + 1)]).is_err());
    apply(&s, "PH1", "Testbook", 1, "rose", &[pc(5, 0, 3)]).unwrap(); // "Chi"
    assert_eq!(all(&s, "PH1").len(), 3);
}

#[test]
fn a_cross_verse_stroke_is_one_mark_and_stays_in_one_chapter() {
    let s = state("pen-cmd-cross");
    let texts = commands::chapter_texts(&s, "PH1", "Testbook", 1).unwrap();
    let len = |v: usize| texts[v - 1].1.chars().count() as u32;
    // from "Zeta" in 1:2 through 1:3 into "Rho" in 1:4
    let stroke = [pc(2, 0, len(2)), pc(3, 0, len(3)), pc(4, 0, 3)];
    let out = apply(&s, "PH1", "Testbook", 1, "violet", &stroke).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].pieces.len(), 3);
    // a stroke with a verse missing in the middle, or one that names another
    // chapter's verse, is refused: a stroke is contiguous and in one chapter
    assert!(
        apply(
            &s,
            "PH1",
            "Testbook",
            1,
            "gold",
            &[pc(2, 0, len(2)), pc(4, 0, 3)]
        )
        .is_err()
    );
    assert!(
        apply(
            &s,
            "PH1",
            "Testbook",
            1,
            "gold",
            &[pc(10, 0, len(10)), pc(11, 0, 3)]
        )
        .is_err()
    );
    // undo snapshot round trip through the command layer
    let snap: Vec<MarkIn> = out
        .iter()
        .map(|m| MarkIn {
            color: m.color.clone(),
            created_at: m.created_at,
            pieces: m
                .pieces
                .iter()
                .map(|p| pc(p.verse, p.start, p.end))
                .collect(),
        })
        .collect();
    commands::with_marks(&s, "PH1", "Testbook", 1, |db, k, t| {
        db.pen_erase(k, t, &[pc(3, 0, len(3))])
    })
    .unwrap();
    let back = commands::with_marks(&s, "PH1", "Testbook", 1, |db, k, t| {
        db.marks_replace_chapter(k, t, &snap)
    })
    .unwrap();
    assert_eq!(back[0].pieces, out[0].pieces);
}

#[test]
fn the_content_file_is_never_written() {
    // highlights live in user.sqlite only: the fixture files are unchanged
    let before: Vec<Vec<u8>> = fixture::FILES
        .iter()
        .map(|f| std::fs::read(fixture::dir().join(f.file)).unwrap())
        .collect();
    let s = state("pen-cmd-ro");
    apply(&s, "PH1", "Testbook", 1, "gold", &[pc(1, 0, 5)]).unwrap();
    commands::with_marks(&s, "PH1", "Testbook", 1, |db, k, t| {
        db.pen_erase(k, t, &[pc(1, 0, 5)])
    })
    .unwrap();
    drop(s);
    let after: Vec<Vec<u8>> = fixture::FILES
        .iter()
        .map(|f| std::fs::read(fixture::dir().join(f.file)).unwrap())
        .collect();
    assert!(before == after);
}
