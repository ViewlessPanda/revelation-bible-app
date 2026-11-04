//! T2: versification table (DESIGN.md 9.2, M1-F10; SOURCES.md VRS-ENG-PALASO).
//! Expected verse counts: Paratext 'English' versification (eng.vrs, SIL
//! libpalaso, MIT), with the KJV overrides listed in SOURCES.md VRS-ENG-PALASO.

mod common;

use revelation_pipeline::sources::{self, CAP_OVERRIDES, CAP_VRS};
use revelation_pipeline::vrs::{self, LineCounts, Override};

fn overrides() -> Vec<Override> {
    let bytes =
        sources::read_capped(&revelation_pipeline::overrides_path(), CAP_OVERRIDES).unwrap();
    vrs::parse_overrides(&bytes).unwrap()
}

fn ov(book: &str, chapter: u16, verses: u16) -> Override {
    Override {
        book: book.into(),
        chapter,
        verses,
    }
}

#[test]
fn overrides_file_has_exactly_the_reviewed_lines() {
    let o = overrides();
    assert_eq!(o, vec![ov("3JN", 1, 14), ov("REV", 12, 17)]);
    assert_eq!(o, revelation_pipeline::profile::kjv_overrides());
}

#[test]
fn eng_vrs_line_classes_and_table() {
    let Some(raw) = common::sources_dir() else {
        return;
    };
    let v = sources::verify(
        &revelation_pipeline::checksums_dir(),
        &raw,
        "VRS-ENG-PALASO",
        CAP_VRS,
    )
    .unwrap();
    let (table, counts) = vrs::parse_vrs(&v.bytes).unwrap();
    assert_eq!(
        counts,
        LineCounts {
            total: 452,
            comments: 89,
            directives: 1,
            blank: 1,
            canon_books: 66,
            ignored_books: 26,
            mappings: 269,
        }
    );
    assert_eq!(table.books.len(), 66);
    assert_eq!((table.chapters(), table.verses()), (1189, 31104));
    assert_eq!(table.get("3JN", 1), Some(15));
    assert_eq!(table.get("REV", 12), Some(18));
    let kjv = vrs::apply(&table, &overrides()).unwrap();
    // Asserted before any database is tested against it (condition 3).
    assert_eq!((kjv.chapters(), kjv.verses()), (1189, 31102));
    assert_eq!(kjv.get("3JN", 1), Some(14));
    assert_eq!(kjv.get("REV", 12), Some(17));
}

#[test]
fn overrides_must_name_a_chapter_and_change_it() {
    let table = vrs::Table {
        books: vec![("3JN".into(), vec![15])],
    };
    let ok = vrs::apply(&table, &[ov("3JN", 1, 14)]);
    assert_eq!(ok.unwrap().get("3JN", 1), Some(14));
    for o in [ov("3JN", 1, 15), ov("3JN", 2, 1), ov("REV", 1, 1)] {
        assert!(
            vrs::apply(&table, std::slice::from_ref(&o)).is_err(),
            "{o:?}"
        );
    }
}
