//! Model-level fixture tests: T6 (verse text), T9 (red invariants), T10 (no
//! inference), T12 (units), T13 (Strong's) and the verse-level half of T15.
//! Every failing fixture asserts which check fires.

mod common;

use common::fixtures::{Q, doc, one, v, w};
use revelation_pipeline::error::Failure;
use revelation_pipeline::osis::{self, Mark, MarkKind, Model, RedPiece};
use revelation_pipeline::profile::{self, AllowedChar, Anomaly, Profile};
use revelation_pipeline::validate;

fn run_with(bytes: &[u8], p: &Profile) -> Result<Model, Failure> {
    let m = osis::parse(bytes, p)?;
    validate::validate(&m, p, None)?;
    Ok(m)
}

fn run(bytes: &[u8]) -> Result<Model, Failure> {
    run_with(bytes, &profile::fixture())
}

#[track_caller]
fn fails(name: &str, bytes: &[u8], check: &str) {
    match run(bytes) {
        Ok(_) => panic!("{name}: accepted"),
        Err(f) => assert!(f.has(check), "{name}: expected {check}, got {f}"),
    }
}

fn red_text(m: &Model, i: usize) -> Vec<String> {
    let v = &m.verses[i];
    v.red
        .iter()
        .map(|p| {
            v.text
                .chars()
                .skip(p.start as usize)
                .take((p.end - p.start) as usize)
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------- T6

#[test]
fn t6_verse_text_invariants() {
    let cases: &[(&str, String, &str)] = &[
        ("tab", w("alpha\tbeta"), "V2"),
        ("nbsp", w("alpha\u{a0}beta"), "V2"),
        ("u2028", w("alpha\u{2028}beta"), "V2"),
        ("rlo", w("alpha\u{202e}beta"), "V6"),
        ("zwsp", w("alpha\u{200b}beta"), "V7"),
        ("bom-mid", w("alpha\u{feff}beta"), "V6"),
        ("fffd", w("alpha\u{fffd}beta"), "V6"),
        ("c1-control", w("alpha\u{85}beta"), "V2"),
        ("lrm-unlisted", w("alpha\u{200e}beta"), "V7"),
        ("leading-space", format!(" {}", w("alpha")), "V3"),
        ("two-spaces", format!("{} {}", w("alpha"), " beta"), "V4"),
        (
            "two-spaces-after-note",
            format!(
                "{} <note type=\"study\">gamma</note> {}",
                w("alpha"),
                w("beta")
            ),
            "V4",
        ),
        ("empty-verse", String::new(), "V1"),
        (
            "only-note",
            "<note type=\"study\">gamma</note>".into(),
            "V1",
        ),
        ("4097-code-points", w(&"x".repeat(4097)), "V1"),
    ];
    for (name, body, check) in cases {
        fails(name, &one(body), check);
    }
    // 4,096 code points is allowed; the trailing space is stored as is.
    let m = run(&one(&format!("{} ", w(&"x".repeat(4095))))).unwrap();
    assert_eq!(m.verses[0].len, 4096);
    assert!(m.verses[0].text.ends_with("x "));
}

#[test]
fn t6_text_is_byte_faithful() {
    let body = format!(
        "<milestone type=\"x-p\" marker=\"¶\"/>{}, {} <transChange type=\"added\">it</transChange>.<note type=\"study\">n</note>",
        w("Alpha\u{2019}s"),
        w("be\u{2013}ta")
    );
    let m = run(&one(&body)).unwrap();
    // No pilcrow inserted, no note text, nothing trimmed or converted.
    assert_eq!(m.verses[0].text, "Alpha\u{2019}s, be\u{2013}ta it.");
}

// ---------------------------------------------------------------- T9

#[test]
fn t9_red_invariant_fixtures() {
    let two_chapters = |a: &str, b: &str| {
        format!(
            "{}<osis><osisText><div type=\"book\" osisID=\"Test\">\n<chapter osisID=\"Test.1\">\n{a}\n</chapter>\n<chapter osisID=\"Test.2\">\n{b}\n</chapter>\n</div></osisText></osis>\n",
            common::fixtures::DECL
        )
        .into_bytes()
    };
    let v2 = |n: u32, body: &str| {
        format!(
            "<verse osisID=\"Test.2.{n}\" sID=\"Test.2.{n}\"/>{body}<verse eID=\"Test.2.{n}\"/>"
        )
    };
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "q-across-chapter",
            two_chapters(&v(1, &format!("{} {Q}{}", w("alpha"), w("beta"))), &v2(1, &format!("{}</q>", w("gamma")))),
            "xml",
        ),
        (
            "q-wrapping-chapters",
            format!(
                "{}<osis><osisText><div type=\"book\" osisID=\"Test\">{Q}<chapter osisID=\"Test.1\">{}</chapter></q></div></osisText></osis>\n",
                common::fixtures::DECL,
                v(1, &w("alpha"))
            )
            .into_bytes(),
            "R2",
        ),
        ("q-in-note", one(&format!("{}<note type=\"study\">{Q}beta</q></note>", w("alpha"))), "R2"),
        (
            "q-start-between-verses",
            doc(&format!("{}\n{Q}{}</q>", v(1, &w("alpha")), v(2, &w("beta")))),
            "R2",
        ),
        (
            "q-end-between-verses",
            doc(&format!("{Q}{}\n</q>{}", v(1, &w("alpha")), v(2, &w("beta")))),
            "R2",
        ),
        ("q-nested", one(&format!("{Q}{} {Q}{}</q></q>", w("alpha"), w("beta"))), "R2"),
        ("q-in-w", one(&format!("<w lemma=\"strong:H01\">{Q}alpha</q></w>")), "R2"),
        (
            "empty-piece-at-verse-end",
            doc(&format!("{}\n{}", v(1, &format!("{} {Q}", w("alpha"))), v(2, &format!("{}</q>", w("beta"))))),
            "R1",
        ),
        (
            "empty-piece-at-verse-start",
            doc(&format!("{}\n{}", v(1, &format!("{Q}{}", w("alpha"))), v(2, &format!("</q> {}", w("beta"))))),
            "R1",
        ),
        ("empty-q-pair", one(&format!("{} {Q}</q>", w("alpha"))), "R1"),
        ("adjacent-q-no-separator", one(&format!("{Q}{}</q>{Q}{}</q>", w("alpha"), w("beta"))), "R3"),
        (
            "transChange-containing-q",
            one(&format!("<transChange type=\"added\">alpha {Q}{}</q></transChange>", w("beta"))),
            "P4",
        ),
        ("q-who-lowercase", one(&format!("<q who=\"jesus\" marker=\"\">{}</q>", w("alpha"))), "q-attr"),
        ("q-who-trailing-space", one(&format!("<q who=\"Jesus \" marker=\"\">{}</q>", w("alpha"))), "q-attr"),
        ("q-extra-attribute", one(&format!("<q who=\"Jesus\" marker=\"\" level=\"1\">{}</q>", w("alpha"))), "q-attr"),
        ("q-no-marker", one(&format!("<q who=\"Jesus\">{}</q>", w("alpha"))), "q-attr"),
        ("q-quote-marker", one(&format!("<q who=\"Jesus\" marker=\"\u{201c}\">{}</q>", w("alpha"))), "q-attr"),
        ("q-milestone-start", one(&format!("<q sID=\"q1\" who=\"Jesus\" marker=\"\">{}</q>", w("alpha"))), "q-form"),
        // R4/R5 (KJV boundary facts): a boundary not next to a space.
        ("boundary-mid-word", one(&format!("{}{Q}{}</q>", w("alpha"), w("beta"))), "R4"),
    ];
    for (name, bytes, check) in &cases {
        fails(name, bytes, check);
    }
}

#[test]
fn t9_passing_cases_have_exact_ranges() {
    // Two q separated by one space: two pieces, the space black.
    let m = run(&one(&format!("{Q}{}</q> {Q}{}</q>", w("alpha"), w("beta")))).unwrap();
    assert_eq!(
        m.verses[0].red,
        vec![
            RedPiece {
                start: 0,
                end: 5,
                q_ord: 1
            },
            RedPiece {
                start: 6,
                end: 10,
                q_ord: 2
            }
        ]
    );
    // q containing transChange containing divineName: nested exact ranges.
    let m = run(&one(&format!(
        "{} {Q}<transChange type=\"added\">beta <divineName>Gamma</divineName></transChange> {}</q>",
        w("alpha"),
        w("delta")
    )))
    .unwrap();
    let vs = &m.verses[0];
    assert_eq!(vs.text, "alpha beta Gamma delta");
    assert_eq!(
        vs.red,
        vec![RedPiece {
            start: 6,
            end: 22,
            q_ord: 1
        }]
    );
    assert_eq!(
        vs.marks,
        vec![
            Mark {
                kind: MarkKind::Added,
                start: 6,
                end: 16
            },
            Mark {
                kind: MarkKind::Divine,
                start: 11,
                end: 16
            }
        ]
    );
}

#[test]
fn t9_q_over_three_verses_with_a_note_in_the_middle() {
    // Findings case 2: passes; the middle piece is [0, len) and the note
    // shifts nothing.
    let m = run(&doc(&format!(
        "{}\n{}\n{}",
        v(1, &format!("{} {Q}{}", w("alpha"), w("beta"))),
        v(
            2,
            &format!("{}<note type=\"study\">zeta</note>", w("gamma"))
        ),
        v(3, &format!("{}</q> {}", w("delta"), w("epsilon")))
    )))
    .unwrap();
    let r: Vec<_> = m.verses.iter().map(|v| v.red.clone()).collect();
    assert_eq!(
        r[0],
        vec![RedPiece {
            start: 6,
            end: 10,
            q_ord: 1
        }]
    );
    assert_eq!(
        r[1],
        vec![RedPiece {
            start: 0,
            end: 5,
            q_ord: 1
        }]
    );
    assert_eq!(
        r[2],
        vec![RedPiece {
            start: 0,
            end: 5,
            q_ord: 1
        }]
    );
    assert_eq!(m.verses[1].text, "gamma");
    assert_eq!(red_text(&m, 0), vec!["beta"]);
    assert_eq!(m.qs.len(), 1);
}

// ---------------------------------------------------------------- T10

#[test]
fn t10_no_inference() {
    // Narration about Jesus speaking, without q: no red anywhere.
    let m = run(&doc(&format!(
        "{}\n{}",
        v(
            1,
            &format!("{} {}, {}", w("Jesus"), w("said"), w("Follow me."))
        ),
        v(2, &format!("{} {}", w("And"), w("Jesus answered."))),
    )))
    .unwrap();
    assert!(m.verses.iter().all(|v| v.red.is_empty()));
    assert!(m.qs.is_empty());
}

// ---------------------------------------------------------------- T12

fn units_profile() -> Profile {
    let mut p = profile::fixture();
    p.chars.push(AllowedChar {
        c: '\u{1D400}',
        is_mark: false,
    });
    p.chars.push(AllowedChar {
        c: '\u{0301}',
        is_mark: true,
    });
    // R4 would also catch a boundary before a mark here; turn it off so the
    // test shows B1 on its own.
    p.kjv_red_boundaries = false;
    p
}

#[test]
fn t12_code_points_end_to_end() {
    // "alpha 𝐀́ " then red "𝐀́ beta": code points, not UTF-8 bytes or UTF-16 units.
    let body = format!(
        "{} {} {Q}{} {}</q>",
        w("alpha"),
        w("\u{1D400}\u{301}"),
        w("\u{1D400}\u{301}"),
        w("beta")
    );
    let m = run_with(&one(&body), &units_profile()).unwrap();
    let vs = &m.verses[0];
    assert_eq!(vs.text, "alpha \u{1D400}\u{301} \u{1D400}\u{301} beta");
    let p = vs.red[0];
    assert_eq!((p.start, p.end), (9, 16));
    let prefix = "alpha \u{1D400}\u{301} ";
    assert_eq!(prefix.len(), 13, "UTF-8 bytes differ");
    assert_eq!(prefix.encode_utf16().count(), 10, "UTF-16 units differ");
    assert_eq!(prefix.chars().count(), 9, "code points are what is stored");
    assert_eq!(red_text(&m, 0), vec!["\u{1D400}\u{301} beta"]);
    assert_eq!(vs.len, 16);
}

#[test]
fn t12_boundary_before_a_combining_mark_fails_b1() {
    let p = units_profile();
    // Red piece ends between 𝐀 and its combining mark.
    let red = format!(
        "{} {Q}{}</q>\u{301} {}",
        w("alpha"),
        w("\u{1D400}"),
        w("beta")
    );
    let f = run_with(&one(&red), &p).unwrap_err();
    assert!(f.has("B1"), "{f}");
    // A word ending before the mark.
    let word = format!("{} {}\u{301}", w("alpha"), w("\u{1D400}"));
    let f = run_with(&one(&word), &p).unwrap_err();
    assert!(f.has("B1"), "{f}");
    // ZWJ: a boundary right after U+200D.
    let mut pz = p.clone();
    pz.chars.push(AllowedChar {
        c: '\u{200D}',
        is_mark: false,
    });
    let zwj = format!("{} {}{}", w("alpha"), w("x\u{200D}"), w("y"));
    let f = run_with(&one(&zwj), &pz).unwrap_err();
    assert!(f.has("B1"), "{f}");
}

// ---------------------------------------------------------------- T13

#[test]
fn t13_kjv_strongs_spelling() {
    let lw = |lemma: &str| one(&format!("<w lemma=\"{lemma}\">alpha</w>"));
    for (name, lemma) in [
        ("hebrew-unpadded", "strong:H7225"),
        ("hebrew-double-zero", "strong:H00430"),
        ("greek-padded", "strong:G01"),
        ("double-space", "strong:H01  strong:H02"),
        ("tab", "strong:H01\tstrong:H02"),
        ("leading-space", " strong:H01"),
        ("trailing-space", "strong:H01 "),
        ("no-prefix", "H01"),
        ("suffix", "strong:H01a"),
        ("zero", "strong:H00"),
        ("greek-in-ot", "strong:G1"),
        ("unknown-token", "strong:H01 morph:X"),
    ] {
        fails(name, &lw(lemma), "T13");
    }
    // Good spellings and lemma.TR tokens.
    let m = run(&lw("strong:H0853 strong:H01254 lemma.TR:x")).unwrap();
    let keys: Vec<String> = m.verses[0].words[0]
        .keys
        .iter()
        .map(|k| k.to_string())
        .collect();
    assert_eq!(keys, vec!["H853", "H1254"]);
    // Empty <w/> is a zero-length word at its position.
    let m = run(&one(&format!(
        "{} <w lemma=\"strong:H01\"/>{}",
        w("alpha"),
        w("beta")
    )))
    .unwrap();
    let ws = &m.verses[0].words;
    assert_eq!((ws[1].start, ws[1].end), (6, 6));
    // A w without lemma, or with an unknown attribute, fails closed.
    fails("w-no-lemma", &one("<w>alpha</w>"), "attr");
    fails(
        "w-extra-attr",
        &one("<w lemma=\"strong:H01\" xml:lang=\"en\">alpha</w>"),
        "attr",
    );
    fails(
        "w-bad-type",
        &one("<w lemma=\"strong:H01\" type=\"x-split\">alpha</w>"),
        "attr",
    );
}

#[test]
fn t13_anomaly_list_entries_must_match_exactly_once() {
    let mut p = profile::fixture();
    p.anomalies.push(Anomaly {
        verse: "Test.1.1",
        lemma: "strong:H01 H02",
        skip_token: "H02",
        why: "test",
    });
    let m = run_with(&one("<w lemma=\"strong:H01 H02\">alpha</w>"), &p).unwrap();
    assert_eq!(m.verses[0].words[0].keys.len(), 1);
    // Stale: the entry matches nothing.
    let f = run_with(&one(&w("alpha")), &p).unwrap_err();
    assert!(f.has("anomaly"), "{f}");
    // Same token elsewhere (other verse or other lemma): not covered.
    let f = run_with(&one("<w lemma=\"strong:H03 H02\">alpha</w>"), &p).unwrap_err();
    assert!(f.has("T13"), "{f}");
}

// ---------------------------------------------------------------- T15, verse level

#[test]
fn t15_verse_structure_and_content() {
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "osisID-differs-from-sID",
            doc(&format!("<verse osisID=\"Test.1.2\" sID=\"Test.1.1\"/>{}<verse eID=\"Test.1.1\"/>", w("a"))),
            "structure",
        ),
        (
            "eID-mismatch",
            doc(&format!("<verse osisID=\"Test.1.1\" sID=\"Test.1.1\"/>{}<verse eID=\"Test.1.2\"/>", w("a"))),
            "structure",
        ),
        ("duplicate-sID", doc(&format!("{}\n{}", v(1, &w("a")), v(1, &w("b")))), "structure"),
        ("out-of-order", doc(&format!("{}\n{}", v(2, &w("a")), v(1, &w("b")))), "structure"),
        ("gap", doc(&format!("{}\n{}", v(1, &w("a")), v(3, &w("b")))), "structure"),
        (
            "wrong-chapter",
            doc(&format!("<verse osisID=\"Test.2.1\" sID=\"Test.2.1\"/>{}<verse eID=\"Test.2.1\"/>", w("a"))),
            "structure",
        ),
        (
            "multi-verse-osisID",
            doc(&format!("<verse osisID=\"Test.1.1 Test.1.2\" sID=\"Test.1.1 Test.1.2\"/>{}<verse eID=\"Test.1.1 Test.1.2\"/>", w("a"))),
            "structure",
        ),
        (
            "verse-open-across-chapter",
            doc(&format!("<verse osisID=\"Test.1.1\" sID=\"Test.1.1\"/>{}", w("a"))),
            "structure",
        ),
        ("unmatched-eID", doc(&format!("{}\n<verse eID=\"Test.1.2\"/>", v(1, &w("a")))), "structure"),
        ("verse-in-verse", one(&format!("{}<verse osisID=\"Test.1.2\" sID=\"Test.1.2\"/>", w("a"))), "structure"),
        ("stray-text-between-verses", doc(&format!("{}\nstray\n{}", v(1, &w("a")), v(2, &w("b")))), "structure"),
        ("stray-element-between-verses", doc(&format!("{}\n{}\n{}", v(1, &w("a")), w("x"), v(2, &w("b")))), "structure"),
        ("unknown-element-in-verse", one("<foreign>alpha</foreign>"), "verse-content"),
        ("title-in-verse", one("<title>alpha</title>"), "verse-content"),
        ("w-in-w", one("<w lemma=\"strong:H01\"><w lemma=\"strong:H02\">a</w></w>"), "verse-content"),
        ("transChange-other-type", one("<transChange type=\"amplified\">alpha</transChange>"), "attr"),
        ("divineName-attribute", one("<divineName type=\"x\">alpha</divineName>"), "attr"),
        ("milestone-other-marker", one(&format!("<milestone type=\"x-p\" marker=\"x\"/>{}", w("a"))), "attr"),
        ("note-other-type", one(&format!("{}<note type=\"x\">n</note>", w("a"))), "attr"),
        ("element-in-note", one(&format!("{}<note type=\"study\"><w lemma=\"strong:H01\">n</w></note>", w("a"))), "verse-content"),
        ("inscription-attribute", one("<inscription type=\"x\">ALPHA</inscription>"), "attr"),
        ("empty-transChange-pair", one(&format!("{} <transChange type=\"added\"></transChange>", w("a"))), "P2"),
        (
            "book-out-of-order",
            format!("{}<osis><osisText><div type=\"book\" osisID=\"Other\"><chapter osisID=\"Other.1\">{}</chapter></div></osisText></osis>\n", common::fixtures::DECL, w("a")).into_bytes(),
            "structure",
        ),
    ];
    for (name, bytes, check) in &cases {
        fails(name, bytes, check);
    }
}

/// The "both" set: every fail-* fixture of the Python walker's self-test
/// (docs/pipeline/survey-scripts/fixtures/) must also fail in Rust, and so
/// must the M1-F3 copies in pipeline/tests/fixtures/.
#[test]
fn t15_shared_fixtures_fail_in_rust_too() {
    let dirs = [
        revelation_pipeline::repo_dir().join("docs/pipeline/survey-scripts/fixtures"),
        revelation_pipeline::repo_dir().join("pipeline/tests/fixtures/m1-f3"),
    ];
    let mut n = 0;
    for dir in dirs {
        for e in std::fs::read_dir(&dir).unwrap() {
            let path = e.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if !name.starts_with("fail-") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            assert!(run(&bytes).is_err(), "{name} accepted by Rust");
            n += 1;
        }
    }
    assert!(n >= 23, "only {n} shared fixtures found");
}

// ---------------------------------------------------------------- book main titles

/// A one-book document whose book div starts with `head` (before chapter 1).
fn titled(head: &str, verse_body: &str) -> Vec<u8> {
    format!(
        "{}<osis><osisText><div type=\"book\" osisID=\"Test\">\n{head}\n<chapter osisID=\"Test.1\">\n{}\n</chapter>\n</div></osisText></osis>\n",
        common::fixtures::DECL,
        v(1, verse_body)
    )
    .into_bytes()
}

#[test]
fn main_title_is_stored_raw_with_abbr_markup_dropped() {
    let m = run(&titled(
        "<title type=\"main\">THE FIRST ALPHA OF <abbr expansion=\"Saint\">ST.</abbr> BETA</title>",
        "alpha",
    ))
    .unwrap();
    assert_eq!(
        m.books[0].main_title.as_deref(),
        Some("THE FIRST ALPHA OF ST. BETA")
    );
    // No title: None (fixtures); the KJV profile requires one per book.
    let m = run(&one("alpha")).unwrap();
    assert_eq!(m.books[0].main_title, None);
    let mut p = profile::fixture();
    p.main_titles = true;
    let f = run_with(&one("alpha"), &p).unwrap_err();
    assert!(f.has("title"), "{f}");
    // Other title types at book level are not imported (unchanged behaviour).
    let m = run(&titled("<title type=\"x-other\">GAMMA</title>", "alpha")).unwrap();
    assert_eq!(m.books[0].main_title, None);
}

#[test]
fn main_title_fixtures_fail_closed() {
    let t = |s: &str| titled(s, "alpha");
    fails(
        "element in title",
        &t("<title type=\"main\">ALPHA <w lemma=\"strong:H01\">BETA</w></title>"),
        "title",
    );
    fails(
        "extra attribute",
        &t("<title type=\"main\" short=\"A\">ALPHA</title>"),
        "attr",
    );
    fails(
        "abbr with another attribute",
        &t("<title type=\"main\">ALPHA <abbr type=\"x\">ST.</abbr></title>"),
        "attr",
    );
    fails(
        "two main titles",
        &t("<title type=\"main\">ALPHA</title><title type=\"main\">BETA</title>"),
        "title",
    );
    fails("empty title", &t("<title type=\"main\"></title>"), "title");
    fails(
        "tab in title",
        &t("<title type=\"main\">ALPHA\tBETA</title>"),
        "title",
    );
    fails(
        "line feed in title",
        &t("<title type=\"main\">ALPHA\nBETA</title>"),
        "title",
    );
    fails(
        "bidi control in title",
        &t("<title type=\"main\">ALPHA \u{202E}BETA</title>"),
        "title",
    );
    fails(
        "non-allowlisted character",
        &t("<title type=\"main\">ALPHA \u{00E9}</title>"),
        "title",
    );
    // After chapter 1 started: a title inside a chapter is not imported, but a
    // main title there is a structure the survey didn't see.
    let late = format!(
        "{}<osis><osisText><div type=\"book\" osisID=\"Test\">\n<chapter osisID=\"Test.1\">\n<title type=\"main\">ALPHA</title>\n{}\n</chapter>\n</div></osisText></osis>\n",
        common::fixtures::DECL,
        v(1, "alpha")
    );
    fails("main title inside a chapter", late.as_bytes(), "title");
}
