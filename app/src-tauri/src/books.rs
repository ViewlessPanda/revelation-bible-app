//! Reader-facing short book names and the abbreviations the command box
//! accepts, keyed by the OSIS book ID stored in `book.osis` (pipeline/DESIGN.md 8).
//!
//! The table is app data: `data/kjv-book-names.tsv`, a byte-for-byte copy of
//! the TSV in the scholar's cited recommendation (docs/scholar/kjv-book-names.md
//! section 6; the citation is in the file's comment header and section 8 of
//! that document). A test checks the copy. The long main titles
//! ("THE FIRST BOOK OF MOSES …") are content and come from `book.main_title`.
//!
//! The two owner decisions (kjv-book-names.md 4.1 and 4.3, decided 2026-10-22:
//! arabic numerals, "Psalm" in Psalms headings) are each the one-line constant
//! below. A book whose OSIS ID is not in the table (for
//! example the placeholder fixture's "Testbook") shows its OSIS ID as its name
//! and has no abbreviations.

use std::sync::OnceLock;

/// The scholar's table, compiled in (no file is read at run time).
const TSV: &str = include_str!("../data/kjv-book-names.tsv");

/// Numeral style of numbered books (kjv-book-names.md 4.1; owner decision
/// 2026-10-22: arabic). `Arabic` = "1 Samuel" (recommended option A, as stored
/// in the table); `Roman` = "I. Samuel", as both cited editions print it.
const NUMERALS: Numerals = Numerals::Arabic;

/// The book name used in Psalms chapter headings (kjv-book-names.md 4.3;
/// owner decision 2026-10-22: "Psalm"). The book is "Psalms"; S1 (n475) heads
/// each chapter "PSALM I." etc., so the heading reads "Psalm 23".
const PSALMS_HEADING: &str = "Psalm";

#[allow(dead_code)] // `Roman` is kept for the one-line switch (owner chose arabic)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Numerals {
    Arabic,
    Roman,
}

struct Row {
    osis: String,
    short_name: String,
    abbrevs: Vec<String>,
}

/// Parses the TSV: `#` comment lines, then the header row
/// `osis\tshort_name\tcitation\tabbreviations`, then one row per book.
fn parse(text: &str) -> Result<Vec<Row>, String> {
    let mut lines = text.lines().filter(|l| !l.starts_with('#'));
    if lines.next() != Some("osis\tshort_name\tcitation\tabbreviations") {
        return Err("missing or changed header row".into());
    }
    let mut rows = Vec::new();
    for (i, line) in lines.enumerate() {
        let f: Vec<&str> = line.split('\t').collect();
        let [osis, short_name, citation, abbrevs] = f[..] else {
            return Err(format!("row {}: expected 4 tab-separated fields", i + 1));
        };
        if osis.is_empty() || short_name.is_empty() || citation.is_empty() {
            return Err(format!("row {}: empty field", i + 1));
        }
        rows.push(Row {
            osis: osis.to_string(),
            short_name: short_name.to_string(),
            abbrevs: abbrevs.split(',').map(str::to_string).collect(),
        });
    }
    Ok(rows)
}

fn table() -> &'static [Row] {
    static T: OnceLock<Vec<Row>> = OnceLock::new();
    T.get_or_init(|| parse(TSV).expect("data/kjv-book-names.tsv is malformed (a test checks it)"))
}

/// "1 Samuel" -> "I. Samuel" when the owner picks roman numerals.
fn with_numerals(short: &str, style: Numerals) -> String {
    if style == Numerals::Roman {
        for (arabic, roman) in [("1 ", "I. "), ("2 ", "II. "), ("3 ", "III. ")] {
            if let Some(rest) = short.strip_prefix(arabic) {
                return format!("{roman}{rest}");
            }
        }
    }
    short.to_string()
}

/// Reader-facing names for an OSIS book ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    /// Short name (picker, command box, references): "1 Samuel", "Psalms".
    pub name: String,
    /// Name used in chapter headings ("Psalm 23"; see PSALMS_HEADING).
    pub heading: String,
    /// Command-box abbreviations, lower case, no spaces.
    pub abbrevs: Vec<String>,
}

/// Names and abbreviations for an OSIS book ID.
pub fn names(osis: &str) -> Names {
    match table().iter().find(|r| r.osis == osis) {
        Some(r) => {
            let name = with_numerals(&r.short_name, NUMERALS);
            let heading = if osis == "Ps" {
                PSALMS_HEADING.to_string()
            } else {
                name.clone()
            };
            Names {
                name,
                heading,
                abbrevs: r.abbrevs.clone(),
            }
        }
        None => Names {
            name: osis.to_string(),
            heading: osis.to_string(),
            abbrevs: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The data file is the scholar's TSV block, byte for byte, below its
    /// comment header.
    #[test]
    fn data_file_is_the_scholars_table() {
        let doc = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/scholar/kjv-book-names.md"),
        )
        .unwrap();
        let start = doc.find("```tsv\n").expect("TSV block") + "```tsv\n".len();
        let end = start + doc[start..].find("```").unwrap();
        let block = &doc[start..end];
        let ours: String = TSV
            .split_inclusive('\n')
            .skip_while(|l| l.starts_with('#'))
            .collect();
        assert_eq!(ours, block);
        assert!(TSV.lines().take_while(|l| l.starts_with('#')).count() >= 10);
    }

    #[test]
    fn table_is_well_formed() {
        let t = table();
        assert_eq!(t.len(), 66);
        let mut seen = std::collections::HashSet::new();
        let mut abbrevs = std::collections::HashSet::new();
        let full = |s: &str| s.to_lowercase().replace(' ', "");
        for r in t {
            assert!(seen.insert(r.osis.clone()), "duplicate {}", r.osis);
            for a in &r.abbrevs {
                assert!(abbrevs.insert(a.clone()), "abbreviation {a} used twice");
                assert!(
                    !a.is_empty()
                        && a.chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                    "{a}"
                );
                // an abbreviation may never equal another book's full name
                for r2 in t {
                    if r2.osis != r.osis {
                        assert_ne!(*a, full(&r2.short_name), "{a} vs {}", r2.short_name);
                    }
                }
            }
        }
        // the scholar's two changes (kjv-book-names.md 5.4)
        assert!(!names("Jude").abbrevs.contains(&"jud".to_string()));
        assert!(names("Song").abbrevs.contains(&"cant".to_string()));
        assert_eq!(abbrevs.len(), 170);
    }

    #[test]
    fn names_headings_and_the_two_owner_decisions() {
        assert_eq!(names("John").name, "John");
        assert_eq!(names("1Sam").name, "1 Samuel");
        assert_eq!(names("Ps").name, "Psalms");
        assert_eq!(names("Ps").heading, "Psalm");
        assert_eq!(names("Song").heading, "Song of Solomon");
        let none = names("Testbook");
        assert_eq!((none.name.as_str(), none.abbrevs.len()), ("Testbook", 0));
        assert_eq!(names("1Sam").heading, "1 Samuel");
        // the roman option, ready for a one-line switch
        assert_eq!(with_numerals("1 Samuel", Numerals::Roman), "I. Samuel");
        assert_eq!(with_numerals("3 John", Numerals::Roman), "III. John");
        assert_eq!(with_numerals("Isaiah", Numerals::Roman), "Isaiah");
        assert_eq!(with_numerals("2 Kings", Numerals::Arabic), "2 Kings");
    }

    #[test]
    fn malformed_tables_are_refused() {
        assert!(parse("").is_err());
        assert!(parse("osis\tshort\n").is_err());
        let h = "osis\tshort_name\tcitation\tabbreviations\n";
        assert!(parse(&format!("{h}Gen\tGenesis\tcite\n")).is_err());
        assert!(parse(&format!("{h}Gen\t\tcite\tgen\n")).is_err());
        assert_eq!(
            parse(&format!("# c\n{h}Gen\tGenesis\tcite\tgen,ge\n"))
                .unwrap()
                .len(),
            1
        );
    }
}
