//! Versification: `eng.vrs.txt` line classes (DESIGN.md 9.2, M1-F10) and the
//! KJV overrides file → the chapter table that T2/T3 check against.
//!
//! Expected verse counts: Paratext 'English' versification (eng.vrs, SIL
//! libpalaso, MIT), with the KJV overrides listed in SOURCES.md VRS-ENG-PALASO.
//!
//! Every line of `eng.vrs.txt` is exactly one of: a `#!` directive (not a
//! comment), another `#` comment, a blank line, a book line
//! (`<BOOK> <c>:<n> …`) or a mapping line (`<BOOK> <c>:<v> = …`). Anything
//! else fails closed. Only the 66 canonical book lines are used; the other
//! book codes are ignored by a pinned list; directives and mappings are
//! recognised and ignored (M1 maps no versifications).

use crate::error::Error;

/// The 66 canonical books in canonical order (USFM codes, as in eng.vrs).
pub const CANON: [&str; 66] = [
    "GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI", "1CH",
    "2CH", "EZR", "NEH", "EST", "JOB", "PSA", "PRO", "ECC", "SNG", "ISA", "JER", "LAM", "EZK",
    "DAN", "HOS", "JOL", "AMO", "OBA", "JON", "MIC", "NAM", "HAB", "ZEP", "HAG", "ZEC", "MAL",
    "MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL", "EPH", "PHP", "COL", "1TH",
    "2TH", "1TI", "2TI", "TIT", "PHM", "HEB", "JAS", "1PE", "2PE", "1JN", "2JN", "3JN", "JUD",
    "REV",
];

/// Book lines in eng.vrs.txt that M1 ignores (deuterocanonical and others).
pub const IGNORED: [&str; 26] = [
    "TOB", "JDT", "ESG", "WIS", "SIR", "BAR", "LJE", "S3Y", "SUS", "BEL", "1MA", "2MA", "3MA",
    "4MA", "1ES", "2ES", "MAN", "PS2", "JSA", "JDB", "TBS", "SST", "DNT", "BLT", "DAG", "LAO",
];

/// How many lines of each class the file has (T2 pins these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineCounts {
    pub total: usize,
    pub comments: usize,
    pub directives: usize,
    pub blank: usize,
    pub canon_books: usize,
    pub ignored_books: usize,
    pub mappings: usize,
}

/// Maximum verse number per chapter, per canonical book, in canonical order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub books: Vec<(String, Vec<u16>)>,
}

impl Table {
    pub fn chapters(&self) -> usize {
        self.books.iter().map(|(_, c)| c.len()).sum()
    }

    pub fn verses(&self) -> u64 {
        self.books
            .iter()
            .flat_map(|(_, c)| c.iter())
            .map(|&v| u64::from(v))
            .sum()
    }

    /// Verse count of `usfm` chapter `chapter` (1-based).
    pub fn get(&self, usfm: &str, chapter: u16) -> Option<u16> {
        let (_, ch) = self.books.iter().find(|(b, _)| b == usfm)?;
        ch.get(usize::from(chapter).checked_sub(1)?).copied()
    }
}

/// One override line: `<USFM> <chapter> <verses>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Override {
    pub book: String,
    pub chapter: u16,
    pub verses: u16,
}

fn err(line: usize, msg: impl Into<String>) -> Error {
    Error::new("T2", format!("line {line}: {}", msg.into())).file("eng.vrs.txt")
}

fn is_code(s: &str) -> bool {
    s.len() == 3
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// A positive decimal number of 1 to 3 digits without a leading zero.
fn num(s: &str) -> Option<u16> {
    let b = s.as_bytes();
    if b.is_empty() || b.len() > 3 || b[0] == b'0' || !b.iter().all(u8::is_ascii_digit) {
        return None;
    }
    s.parse().ok()
}

/// `<c>:<v>` or `<c>:<v>-<v>`, each optionally followed by one lowercase
/// letter (eng.vrs uses `1:1a` for split verses in mappings).
fn is_ref(s: &str) -> bool {
    let Some((c, v)) = s.split_once(':') else {
        return false;
    };
    if num(c).is_none() {
        return false;
    }
    let one = |x: &str| {
        let x = x
            .strip_suffix(|c: char| c.is_ascii_lowercase())
            .unwrap_or(x);
        // Verse 0 occurs in mappings (Psalm titles), so allow "0" here.
        x == "0" || num(x).is_some()
    };
    match v.split_once('-') {
        Some((a, b)) => one(a) && one(b),
        None => one(v),
    }
}

/// Parses `eng.vrs.txt`. Returns the canonical table and the line counts.
pub fn parse_vrs(bytes: &[u8]) -> Result<(Table, LineCounts), Error> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| Error::new("T2", "not UTF-8").file("eng.vrs.txt"))?;
    if !text.is_ascii() {
        return Err(Error::new("T2", "not ASCII").file("eng.vrs.txt"));
    }
    if text.contains('\r') {
        return Err(Error::new("T2", "contains CR").file("eng.vrs.txt"));
    }
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| Error::new("T2", "last line is not LF-terminated").file("eng.vrs.txt"))?;
    let mut counts = LineCounts::default();
    let mut books: Vec<(String, Vec<u16>)> = Vec::new();
    let mut ignored_seen: Vec<&str> = Vec::new();
    for (i, line) in body.split('\n').enumerate() {
        let n = i + 1;
        counts.total += 1;
        if line.starts_with("#!") {
            counts.directives += 1;
        } else if line.starts_with('#') {
            counts.comments += 1;
        } else if line.is_empty() {
            counts.blank += 1;
        } else if line.contains('=') {
            // Mapping: "<BOOK> <ref> = <BOOK> <ref>", single spaces.
            let parts: Vec<&str> = line.split(' ').collect();
            let ok = parts.len() == 5
                && is_code(parts[0])
                && is_ref(parts[1])
                && parts[2] == "="
                && is_code(parts[3])
                && is_ref(parts[4]);
            if !ok {
                return Err(err(n, "unrecognised mapping line"));
            }
            counts.mappings += 1;
        } else {
            // Book line: "<BOOK> 1:<n> 2:<n> …", chapters numbered 1, 2, 3, …
            let mut parts = line.split(' ');
            let code = parts.next().unwrap_or("");
            if !is_code(code) {
                return Err(err(n, "line fits no class"));
            }
            let mut verses = Vec::new();
            for (k, p) in parts.enumerate() {
                let (c, v) = p
                    .split_once(':')
                    .ok_or_else(|| err(n, "book line entry is not <c>:<n>"))?;
                if num(c) != Some((k + 1) as u16) {
                    return Err(err(n, "book line chapters are not numbered 1, 2, 3, …"));
                }
                verses.push(num(v).ok_or_else(|| err(n, "bad verse count"))?);
            }
            if verses.is_empty() {
                return Err(err(n, "book line has no chapters"));
            }
            if CANON.contains(&code) {
                if books.iter().any(|(b, _)| b == code) {
                    return Err(err(n, "duplicate canonical book line"));
                }
                books.push((code.to_string(), verses));
                counts.canon_books += 1;
            } else if IGNORED.contains(&code) {
                if ignored_seen.contains(&code) {
                    return Err(err(n, "duplicate ignored book line"));
                }
                ignored_seen.push(code);
                counts.ignored_books += 1;
            } else {
                return Err(err(
                    n,
                    "book code is neither canonical nor on the ignore list",
                ));
            }
        }
    }
    let order: Vec<&str> = books.iter().map(|(b, _)| b.as_str()).collect();
    if order != CANON {
        return Err(Error::new(
            "T2",
            "the canonical book lines are not exactly the 66 books in canonical order",
        )
        .file("eng.vrs.txt"));
    }
    Ok((Table { books }, counts))
}

/// Parses the overrides file (`pipeline/data/kjv-versification-overrides.txt`).
pub fn parse_overrides(bytes: &[u8]) -> Result<Vec<Override>, Error> {
    let name = "kjv-versification-overrides.txt";
    let text = std::str::from_utf8(bytes).map_err(|_| Error::new("T2", "not UTF-8").file(name))?;
    if text.contains('\r') {
        return Err(Error::new("T2", "contains CR").file(name));
    }
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| Error::new("T2", "last line is not LF-terminated").file(name))?;
    let mut out = Vec::new();
    for (i, line) in body.split('\n').enumerate() {
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(' ').collect();
        let bad = || {
            Error::new(
                "T2",
                format!("line {}: not '<BOOK> <chapter> <verses>'", i + 1),
            )
            .file(name)
        };
        if parts.len() != 3 || !is_code(parts[0]) {
            return Err(bad());
        }
        out.push(Override {
            book: parts[0].to_string(),
            chapter: num(parts[1]).ok_or_else(bad)?,
            verses: num(parts[2]).ok_or_else(bad)?,
        });
    }
    Ok(out)
}

/// Applies the overrides. Each must name an existing chapter of a canonical
/// book and change its value.
pub fn apply(table: &Table, overrides: &[Override]) -> Result<Table, Error> {
    let mut t = table.clone();
    for o in overrides {
        let (_, ch) = t
            .books
            .iter_mut()
            .find(|(b, _)| *b == o.book)
            .ok_or_else(|| Error::new("T2", format!("override names unknown book {}", o.book)))?;
        let slot = ch.get_mut(usize::from(o.chapter) - 1).ok_or_else(|| {
            Error::new(
                "T2",
                format!("override names missing chapter {} {}", o.book, o.chapter),
            )
        })?;
        if *slot == o.verses {
            return Err(Error::new(
                "T2",
                format!(
                    "override {} {} {} changes nothing",
                    o.book, o.chapter, o.verses
                ),
            ));
        }
        *slot = o.verses;
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_classes() {
        let f = b"# c\n#! &X 1:1 = Y 1:1\n\nGEN 1:2 2:3\nGEN 1:1 = GEN 1:1a\nTOB 1:3\n";
        let mut full = Vec::new();
        full.extend_from_slice(f);
        // Not a full canon: must fail the canon check, after classifying.
        assert!(parse_vrs(&full).is_err());
        assert!(
            parse_vrs(b"GEN 1:2 3:4\n").is_err(),
            "chapters must be sequential"
        );
        assert!(parse_vrs(b"XYZ 1:2\n").is_err(), "unknown code");
        assert!(parse_vrs(b"GEN 1:2\r\n").is_err(), "CR");
        assert!(parse_vrs(b"GEN 1:2").is_err(), "no final LF");
        assert!(parse_vrs(b"just words\n").is_err());
    }

    #[test]
    fn refs() {
        for r in ["1:1", "5:2-15", "1:1a", "3:0", "31:55"] {
            assert!(is_ref(r), "{r}");
        }
        for r in ["1", "01:1", "1:", ":1", "1:1-", "1:1A"] {
            assert!(!is_ref(r), "{r}");
        }
    }

    #[test]
    fn overrides_grammar() {
        assert_eq!(
            parse_overrides(b"# x\n3JN 1 14\n").unwrap(),
            vec![Override {
                book: "3JN".into(),
                chapter: 1,
                verses: 14
            }]
        );
        for bad in [
            &b"3JN 1 14"[..],
            b"3JN  1 14\n",
            b"3JN 1 014\n",
            b"3JN 1 14 x\n",
            b"3JN 1 14\r\n",
        ] {
            assert!(parse_overrides(bad).is_err());
        }
    }
}
