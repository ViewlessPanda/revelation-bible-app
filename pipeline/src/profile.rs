//! Per-source facts and allowlists (DESIGN.md 14): the KJV profile.
//!
//! Everything here is a property of one pinned file (BIB-KJV-CW31), taken
//! from `docs/pipeline/kjv-osis-survey.md` and DESIGN.md (sections 3.2, 3.3,
//! 5.3, 5.4, 6.2, 6.3, 9.4). A different source gets its own profile; a fact
//! that no longer holds fails the build so a person decides.

use crate::vrs::Override;
use crate::xmlscan::{Form, XmlProfile};

/// The reviewed KJV overrides (T2): the overrides file must say exactly this.
pub fn kjv_overrides() -> Vec<Override> {
    vec![
        Override {
            book: "3JN".into(),
            chapter: 1,
            verses: 14,
        },
        Override {
            book: "REV".into(),
            chapter: 12,
            verses: 17,
        },
    ]
}

/// KJV element forms (DESIGN.md 9.4).
pub const KJV_FORMS: &[(&str, Form)] = &[
    ("verse", Form::EmptyOnly),
    ("milestone", Form::EmptyOnly),
    ("w", Form::Either),
    ("q", Form::ContainerOnly),
    ("transChange", Form::ContainerOnly),
    ("divineName", Form::ContainerOnly),
    ("note", Form::ContainerOnly),
    ("inscription", Form::ContainerOnly),
    ("catchWord", Form::ContainerOnly),
    ("rdg", Form::ContainerOnly),
    ("chapter", Form::ContainerOnly),
    ("div", Form::ContainerOnly),
    ("title", Form::ContainerOnly),
    ("foreign", Form::ContainerOnly),
    ("abbr", Form::ContainerOnly),
    ("osis", Form::ContainerOnly),
    ("osisText", Form::ContainerOnly),
    ("header", Form::ContainerOnly),
    ("work", Form::ContainerOnly),
    ("identifier", Form::ContainerOnly),
    ("refSystem", Form::ContainerOnly),
    ("scope", Form::ContainerOnly),
    ("date", Form::ContainerOnly),
    ("p", Form::ContainerOnly),
    ("revisionDesc", Form::ContainerOnly),
];

/// The KJV XML profile (DESIGN.md 9.2: 38-byte declaration, no CR).
pub const KJV_XML: XmlProfile = XmlProfile {
    file: "kjv.osis.xml",
    prefix: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
    allow_cr: false,
    double_quoted: true,
    forms: KJV_FORMS,
    subset: None,
    comments: &[],
    amp: false,
};

/// The 66 books: (OSIS name as in the source, USFM code as in eng.vrs).
pub const KJV_BOOKS: [(&str, &str); 66] = [
    ("Gen", "GEN"),
    ("Exod", "EXO"),
    ("Lev", "LEV"),
    ("Num", "NUM"),
    ("Deut", "DEU"),
    ("Josh", "JOS"),
    ("Judg", "JDG"),
    ("Ruth", "RUT"),
    ("1Sam", "1SA"),
    ("2Sam", "2SA"),
    ("1Kgs", "1KI"),
    ("2Kgs", "2KI"),
    ("1Chr", "1CH"),
    ("2Chr", "2CH"),
    ("Ezra", "EZR"),
    ("Neh", "NEH"),
    ("Esth", "EST"),
    ("Job", "JOB"),
    ("Ps", "PSA"),
    ("Prov", "PRO"),
    ("Eccl", "ECC"),
    ("Song", "SNG"),
    ("Isa", "ISA"),
    ("Jer", "JER"),
    ("Lam", "LAM"),
    ("Ezek", "EZK"),
    ("Dan", "DAN"),
    ("Hos", "HOS"),
    ("Joel", "JOL"),
    ("Amos", "AMO"),
    ("Obad", "OBA"),
    ("Jonah", "JON"),
    ("Mic", "MIC"),
    ("Nah", "NAM"),
    ("Hab", "HAB"),
    ("Zeph", "ZEP"),
    ("Hag", "HAG"),
    ("Zech", "ZEC"),
    ("Mal", "MAL"),
    ("Matt", "MAT"),
    ("Mark", "MRK"),
    ("Luke", "LUK"),
    ("John", "JHN"),
    ("Acts", "ACT"),
    ("Rom", "ROM"),
    ("1Cor", "1CO"),
    ("2Cor", "2CO"),
    ("Gal", "GAL"),
    ("Eph", "EPH"),
    ("Phil", "PHP"),
    ("Col", "COL"),
    ("1Thess", "1TH"),
    ("2Thess", "2TH"),
    ("1Tim", "1TI"),
    ("2Tim", "2TI"),
    ("Titus", "TIT"),
    ("Phlm", "PHM"),
    ("Heb", "HEB"),
    ("Jas", "JAS"),
    ("1Pet", "1PE"),
    ("2Pet", "2PE"),
    ("1John", "1JN"),
    ("2John", "2JN"),
    ("3John", "3JN"),
    ("Jude", "JUD"),
    ("Rev", "REV"),
];

/// A non-ASCII character the profile allows in stored text, with whether it
/// has General_Category `M*` (for B1, DESIGN.md 5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowedChar {
    pub c: char,
    pub is_mark: bool,
}

/// One reviewed source anomaly (DESIGN.md 6.3): keyed by verse **and** the
/// exact `lemma` value; the action is "skip this one token".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anomaly {
    pub verse: &'static str,
    pub lemma: &'static str,
    pub skip_token: &'static str,
    pub why: &'static str,
}

/// Red-letter facts for one book: `q` (by the book of the start tag),
/// pieces, red code points (DESIGN.md 5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedFacts {
    pub book: &'static str,
    pub q: u32,
    pub pieces: u32,
    pub cp: u64,
}

/// Pinned facts of the real file (T3, T6, T8, T13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub verses: usize,
    pub chapters: usize,
    /// Books with a `<title type="main">` (survey section 7: 66).
    pub main_titles: usize,
    /// Books with red; every other book must have none.
    pub red: Vec<RedFacts>,
    pub red_verses: usize,
    pub two_piece_verses: usize,
    pub trailing_space_verses: usize,
    pub marks_added: usize,
    pub marks_divine: usize,
    pub words: usize,
    pub empty_words: usize,
}

/// Everything the parser and validator need to know about one source.
#[derive(Debug, Clone)]
pub struct Profile {
    pub source_id: &'static str,
    pub xml: XmlProfile,
    /// (OSIS, USFM) in canonical order; the file must have exactly these.
    pub books: Vec<(&'static str, &'static str)>,
    /// Non-ASCII characters allowed in stored text (V7). Printable ASCII
    /// U+0020..U+007E is always allowed.
    pub chars: Vec<AllowedChar>,
    pub anomalies: Vec<Anomaly>,
    /// R4 and R5 (red boundaries at a verse edge or next to U+0020, no
    /// whitespace at a piece edge): facts of the KJV, asserted for it.
    pub kjv_red_boundaries: bool,
    /// KJV `strong:` spellings (6.2): Hebrew `H0` + unpadded, Greek unpadded.
    pub kjv_strongs_spelling: bool,
    /// Hebrew keys only in books 1–39, Greek only in 40–66.
    pub testament_rule: bool,
    /// Every book has a `<title type="main">` (stored as `book.main_title`).
    pub main_titles: bool,
    pub facts: Option<Facts>,
}

/// The KJV profile (BIB-KJV-CW31).
pub fn kjv() -> Profile {
    let r = |book, q, pieces, cp| RedFacts {
        book,
        q,
        pieces,
        cp,
    };
    Profile {
        source_id: "BIB-KJV-CW31",
        xml: KJV_XML,
        books: KJV_BOOKS.to_vec(),
        chars: ['\u{2019}', '\u{2013}', '\u{2014}', '\u{00C6}']
            .into_iter()
            .map(|c| AllowedChar { c, is_mark: false })
            .collect(),
        anomalies: vec![Anomaly {
            verse: "2Cor.8.10",
            lemma: "strong:G1722 G5129 lemma.TR:εν lemma.TR:τουτω",
            skip_token: "G5129",
            why: "token has no 'strong:' prefix (survey anomaly 1); not guessed",
        }],
        kjv_red_boundaries: true,
        kjv_strongs_spelling: true,
        testament_rule: true,
        main_titles: true,
        facts: Some(Facts {
            verses: 31_102,
            main_titles: 66,
            chapters: 1_189,
            red: vec![
                r("Matt", 146, 646, 69_530),
                r("Mark", 131, 285, 27_482),
                r("Luke", 185, 589, 62_696),
                r("John", 153, 422, 39_461),
                r("Acts", 21, 28, 2_479),
                r("1Cor", 2, 2, 181),
                r("2Cor", 1, 1, 77),
                r("Rev", 9, 62, 8_596),
            ],
            red_verses: 2_026,
            two_piece_verses: 9,
            trailing_space_verses: 91,
            marks_added: 21_571,
            marks_divine: 6_887,
            words: 355_295,
            // The survey counts 6,331 empty <w/> in the whole file; one of them
            // is outside verse text (a title or colophon), so 6,330 are in
            // verses (Python probe with osis_tokens.py, 2026-10-21).
            empty_words: 6_330,
        }),
    }
}

/// The KJV profile for synthetic fixtures: same rules, one book `Test`
/// (counted as Old Testament), no pinned facts.
pub fn fixture() -> Profile {
    Profile {
        books: vec![("Test", "TST")],
        facts: None,
        main_titles: false,
        anomalies: Vec::new(),
        ..kjv()
    }
}
