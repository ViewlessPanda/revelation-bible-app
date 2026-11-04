//! Pen-mode range highlights (highlights-pen.md 5 and 6): the pure rules.
//!
//! No I/O here. `user.rs` loads a chapter's marks, calls these functions and
//! writes the result in one transaction. Offsets are Unicode scalar values
//! (code points, Rust `chars()`) of the verse text exactly as stored in the
//! translation's content, half-open `[start, end)`: the same unit as red-letter
//! pieces (W2P-B3: never UTF-16). Nothing here reads or changes content text;
//! it only measures it (length, fingerprint, grapheme boundaries).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const COLORS: [&str; 5] = ["gold", "green", "blue", "rose", "violet"];
/// At most this many pieces in one stroke (5.3).
pub const MAX_STROKE_PIECES: usize = 999;
/// At most this many pieces per chapter per translation (11, E12).
pub const MAX_CHAPTER_PIECES: usize = 2000;

/// One piece as the frontend sends it: verse number and code-point offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub verse: u32,
    pub start: u32,
    pub end: u32,
}

/// A mark as an undo snapshot carries it back (`marks_replace_chapter`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkIn {
    pub color: String,
    pub created_at: i64,
    pub pieces: Vec<Piece>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PieceOut {
    pub verse: u32,
    pub start: u32,
    pub end: u32,
    /// The verse text no longer matches the piece's fingerprint or length
    /// (5.4): never drawn, never moved.
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkOut {
    pub id: i64,
    pub color: String,
    pub created_at: i64,
    pub pieces: Vec<PieceOut>,
}

/// A mark in the working model: `id` is `None` for one not yet stored.
/// `pieces` are the fresh (drawable) pieces only; stale pieces never enter
/// the model, so no rule can move or cut them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub id: Option<i64>,
    pub color: String,
    pub created_at: i64,
    pub pieces: Vec<Piece>,
}

/// One verse of the chapter as stored: its code points.
pub struct VerseText {
    pub verse: u32,
    pub chars: Vec<char>,
    /// First 16 hex digits of SHA-256 of the UTF-8 verse text (5.2).
    pub fp: String,
}

/// The chapter's verses, in order, measured once per call.
pub struct ChapterText {
    verses: Vec<VerseText>,
    index: BTreeMap<u32, usize>,
}

pub fn fingerprint(text: &str) -> String {
    revelation_content::sha256_hex(text.as_bytes())[..16].to_string()
}

impl ChapterText {
    pub fn new(texts: &[(u32, String)]) -> ChapterText {
        let verses: Vec<VerseText> = texts
            .iter()
            .map(|(v, t)| VerseText {
                verse: *v,
                chars: t.chars().collect(),
                fp: fingerprint(t),
            })
            .collect();
        let index = verses
            .iter()
            .enumerate()
            .map(|(i, v)| (v.verse, i))
            .collect();
        ChapterText { verses, index }
    }

    pub fn get(&self, verse: u32) -> Option<&VerseText> {
        self.index.get(&verse).map(|&i| &self.verses[i])
    }

    /// The verse after `verse` in this chapter's versification.
    pub fn next_verse(&self, verse: u32) -> Option<u32> {
        let i = *self.index.get(&verse)?;
        self.verses.get(i + 1).map(|v| v.verse)
    }

    pub fn len(&self, verse: u32) -> Option<u32> {
        self.get(verse).map(|v| v.chars.len() as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.verses.is_empty()
    }
}

/// Unicode White_Space (Rust `char::is_whitespace`; the UI uses the same list).
fn ws(c: char) -> bool {
    c.is_whitespace()
}

/// General_Category M* (Mn, Mc, Me), for B1. A conservative table of the
/// combining-mark blocks of the world's scripts (it may include a few
/// unassigned code points inside those blocks, which validated content never
/// holds). The UI snaps to grapheme clusters with `Intl.Segmenter`, whose
/// boundaries never fall before any of these, so the table only ever refuses
/// a hand-made or tampered offset.
pub fn is_mark(c: char) -> bool {
    const R: &[(u32, u32)] = &[
        (0x0300, 0x036F),
        (0x0483, 0x0489),
        (0x0591, 0x05BD),
        (0x05BF, 0x05BF),
        (0x05C1, 0x05C2),
        (0x05C4, 0x05C5),
        (0x05C7, 0x05C7),
        (0x0610, 0x061A),
        (0x064B, 0x065F),
        (0x0670, 0x0670),
        (0x06D6, 0x06DC),
        (0x06DF, 0x06E4),
        (0x06E7, 0x06E8),
        (0x06EA, 0x06ED),
        (0x0711, 0x0711),
        (0x0730, 0x074A),
        (0x07A6, 0x07B0),
        (0x07EB, 0x07F3),
        (0x0816, 0x0819),
        (0x081B, 0x0823),
        (0x0825, 0x0827),
        (0x0829, 0x082D),
        (0x0859, 0x085B),
        (0x08D3, 0x08E1),
        (0x08E3, 0x0903),
        (0x093A, 0x093C),
        (0x093E, 0x094F),
        (0x0951, 0x0957),
        (0x0962, 0x0963),
        (0x0981, 0x0983),
        (0x09BC, 0x09BC),
        (0x09BE, 0x09CD),
        (0x09D7, 0x09D7),
        (0x09E2, 0x09E3),
        (0x0A01, 0x0A03),
        (0x0A3C, 0x0A51),
        (0x0A70, 0x0A71),
        (0x0A75, 0x0A75),
        (0x0A81, 0x0A83),
        (0x0ABC, 0x0ABC),
        (0x0ABE, 0x0ACD),
        (0x0AE2, 0x0AE3),
        (0x0B01, 0x0B03),
        (0x0B3C, 0x0B3C),
        (0x0B3E, 0x0B57),
        (0x0B62, 0x0B63),
        (0x0B82, 0x0B82),
        (0x0BBE, 0x0BCD),
        (0x0BD7, 0x0BD7),
        (0x0C00, 0x0C04),
        (0x0C3E, 0x0C56),
        (0x0C62, 0x0C63),
        (0x0C81, 0x0C83),
        (0x0CBC, 0x0CBC),
        (0x0CBE, 0x0CD6),
        (0x0CE2, 0x0CE3),
        (0x0D00, 0x0D03),
        (0x0D3B, 0x0D3C),
        (0x0D3E, 0x0D4D),
        (0x0D57, 0x0D57),
        (0x0D62, 0x0D63),
        (0x0D81, 0x0D83),
        (0x0DCA, 0x0DDF),
        (0x0DF2, 0x0DF3),
        (0x0E31, 0x0E31),
        (0x0E34, 0x0E3A),
        (0x0E47, 0x0E4E),
        (0x0EB1, 0x0EB1),
        (0x0EB4, 0x0EBC),
        (0x0EC8, 0x0ECD),
        (0x0F18, 0x0F19),
        (0x0F35, 0x0F35),
        (0x0F37, 0x0F37),
        (0x0F39, 0x0F39),
        (0x0F3E, 0x0F3F),
        (0x0F71, 0x0F84),
        (0x0F86, 0x0F87),
        (0x0F8D, 0x0FBC),
        (0x0FC6, 0x0FC6),
        (0x102B, 0x103E),
        (0x1056, 0x1059),
        (0x105E, 0x1060),
        (0x1062, 0x1064),
        (0x1067, 0x106D),
        (0x1071, 0x1074),
        (0x1082, 0x108D),
        (0x108F, 0x108F),
        (0x109A, 0x109D),
        (0x135D, 0x135F),
        (0x1712, 0x1714),
        (0x1732, 0x1734),
        (0x1752, 0x1753),
        (0x1772, 0x1773),
        (0x17B4, 0x17D3),
        (0x17DD, 0x17DD),
        (0x180B, 0x180D),
        (0x1885, 0x1886),
        (0x18A9, 0x18A9),
        (0x1920, 0x193B),
        (0x1A17, 0x1A1B),
        (0x1A55, 0x1A7F),
        (0x1AB0, 0x1AFF),
        (0x1B00, 0x1B04),
        (0x1B34, 0x1B44),
        (0x1B6B, 0x1B73),
        (0x1B80, 0x1B82),
        (0x1BA1, 0x1BAD),
        (0x1BE6, 0x1BF3),
        (0x1C24, 0x1C37),
        (0x1CD0, 0x1CD2),
        (0x1CD4, 0x1CE8),
        (0x1CED, 0x1CED),
        (0x1CF4, 0x1CF4),
        (0x1CF7, 0x1CF9),
        (0x1DC0, 0x1DFF),
        (0x20D0, 0x20F0),
        (0x2CEF, 0x2CF1),
        (0x2D7F, 0x2D7F),
        (0x2DE0, 0x2DFF),
        (0x302A, 0x302F),
        (0x3099, 0x309A),
        (0xA66F, 0xA672),
        (0xA674, 0xA67D),
        (0xA69E, 0xA69F),
        (0xA6F0, 0xA6F1),
        (0xA802, 0xA802),
        (0xA806, 0xA806),
        (0xA80B, 0xA80B),
        (0xA823, 0xA827),
        (0xA880, 0xA881),
        (0xA8B4, 0xA8C5),
        (0xA8E0, 0xA8F1),
        (0xA8FF, 0xA8FF),
        (0xA926, 0xA92D),
        (0xA947, 0xA953),
        (0xA980, 0xA983),
        (0xA9B3, 0xA9C0),
        (0xAA29, 0xAA36),
        (0xAA43, 0xAA43),
        (0xAA4C, 0xAA4D),
        (0xAA7B, 0xAA7D),
        (0xAAB0, 0xAAB0),
        (0xAAB2, 0xAAB4),
        (0xAAB7, 0xAAB8),
        (0xAABE, 0xAABF),
        (0xAAC1, 0xAAC1),
        (0xAAEB, 0xAAEF),
        (0xAAF5, 0xAAF6),
        (0xABE3, 0xABEA),
        (0xABEC, 0xABED),
        (0xFB1E, 0xFB1E),
        (0xFE00, 0xFE0F),
        (0xFE20, 0xFE2F),
        (0x101FD, 0x101FD),
        (0x102E0, 0x102E0),
        (0x10376, 0x1037A),
        (0x10A01, 0x10A0F),
        (0x10A38, 0x10A3F),
        (0x11000, 0x11002),
        (0x11038, 0x11046),
        (0x1107F, 0x11082),
        (0x110B0, 0x110BA),
        (0x11100, 0x11102),
        (0x11127, 0x11134),
        (0x1D165, 0x1D169),
        (0x1D16D, 0x1D172),
        (0x1D17B, 0x1D182),
        (0x1D185, 0x1D18B),
        (0x1D1AA, 0x1D1AD),
        (0x1D242, 0x1D244),
        (0x1E000, 0x1E02A),
        (0x1E8D0, 0x1E8D6),
        (0x1E944, 0x1E94A),
        (0xE0100, 0xE01EF),
    ];
    let c = c as u32;
    R.binary_search_by(|&(lo, hi)| {
        if hi < c {
            std::cmp::Ordering::Less
        } else if lo > c {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    })
    .is_ok()
}

/// B1 (pipeline DESIGN.md 5.1, the red-letter rule): a boundary other than 0
/// and the length never falls immediately before a combining mark, nor
/// immediately before or after U+200D (ZWJ).
pub fn b1_ok(chars: &[char], b: u32) -> bool {
    let b = b as usize;
    if b == 0 || b >= chars.len() {
        return true;
    }
    let (before, after) = (chars[b - 1], chars[b]);
    !(is_mark(after) || after == '\u{200D}' || before == '\u{200D}')
}

pub fn valid_color(c: &str) -> bool {
    COLORS.contains(&c)
}

/// Every check on one piece against the current text (E1, E2, E4).
pub fn check_piece(text: &ChapterText, p: &Piece) -> Result<(), String> {
    let v = text
        .get(p.verse)
        .ok_or_else(|| format!("no verse {} in this chapter", p.verse))?;
    let len = v.chars.len() as u32;
    if !(p.start < p.end && p.end <= len) {
        return Err(format!("offsets out of bounds in verse {}", p.verse));
    }
    if !b1_ok(&v.chars, p.start) || !b1_ok(&v.chars, p.end) {
        return Err(format!(
            "a highlight boundary splits a character in verse {}",
            p.verse
        ));
    }
    if only_ws(&v.chars, p.start, p.end) {
        return Err(format!("a highlight of only spaces in verse {}", p.verse));
    }
    Ok(())
}

/// A pen or eraser stroke (5.3): 1 to 999 pieces, each valid, in
/// consecutive verses, inner pieces whole, the first running to its verse's
/// end and the last starting at 0 when there are several. Rejects the whole
/// stroke on any failure.
pub fn check_stroke(text: &ChapterText, pieces: &[Piece]) -> Result<(), String> {
    if pieces.is_empty() || pieces.len() > MAX_STROKE_PIECES {
        return Err("a stroke has 1 to 999 pieces".into());
    }
    for p in pieces {
        check_piece(text, p)?;
    }
    for w in pieces.windows(2) {
        if text.next_verse(w[0].verse) != Some(w[1].verse) {
            return Err("a stroke covers consecutive verses only".into());
        }
        if w[0].end != text.len(w[0].verse).unwrap_or(0) {
            return Err("a stroke is contiguous: a piece must run to its verse's end".into());
        }
        if w[1].start != 0 {
            return Err("a stroke is contiguous: a piece must start at its verse's start".into());
        }
    }
    Ok(())
}

/// Cut `[s, e)` of `verse` out of every mark (rule 1 and the eraser, rule 4).
/// A piece left by a cut is trimmed of outer whitespace; empty pieces drop.
pub fn cut(marks: &mut [Mark], text: &ChapterText, verse: u32, s: u32, e: u32) {
    let Some(v) = text.get(verse) else { return };
    for m in marks.iter_mut() {
        let mut out = Vec::with_capacity(m.pieces.len() + 1);
        for p in &m.pieces {
            if p.verse != verse || p.end <= s || e <= p.start {
                out.push(*p);
                continue;
            }
            for (a, b) in [(p.start, s.max(p.start)), (e.min(p.end), p.end)] {
                if let Some((a, b)) = trim(&v.chars, a, b) {
                    out.push(Piece {
                        verse,
                        start: a,
                        end: b,
                    });
                }
            }
        }
        m.pieces = out;
    }
}

/// `[a, b)` without leading and trailing whitespace, or `None` if nothing is left.
fn trim(chars: &[char], mut a: u32, mut b: u32) -> Option<(u32, u32)> {
    while a < b && ws(chars[a as usize]) {
        a += 1;
    }
    while b > a && ws(chars[b as usize - 1]) {
        b -= 1;
    }
    (a < b).then_some((a, b))
}

fn only_ws(chars: &[char], a: u32, b: u32) -> bool {
    (a..b).all(|i| ws(chars[i as usize]))
}

/// (start, end, owner mark index) of one piece while merging.
type Span = (u32, u32, usize);

/// Rules 2 and 4 over the whole chapter: same-colour pieces that overlap,
/// touch or are separated only by whitespace join (the whitespace between is
/// included) and their marks become one; same-colour pieces touching across a
/// verse boundary (only whitespace between) join their marks, the pieces
/// staying per verse. The joined mark keeps `prefer`'s id if it is in the group
/// (the new or recoloured mark), else the smallest id, and the earliest
/// `created_at`. Marks left with no pieces are dropped.
pub fn normalize(marks: Vec<Mark>, text: &ChapterText, prefer: Option<usize>) -> Vec<Mark> {
    let n = marks.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    fn union(p: &mut [usize], a: usize, b: usize) {
        let (ra, rb) = (find(p, a), find(p, b));
        if ra != rb {
            p[rb] = ra;
        }
    }
    // (color, verse) -> [(start, end, owner)]
    let mut by_cv: BTreeMap<(&str, u32), Vec<Span>> = BTreeMap::new();
    for (i, m) in marks.iter().enumerate() {
        for p in &m.pieces {
            by_cv
                .entry((m.color.as_str(), p.verse))
                .or_default()
                .push((p.start, p.end, i));
        }
    }
    // merged intervals per (color, verse), each with one owner
    let mut merged: BTreeMap<(&str, u32), Vec<Span>> = BTreeMap::new();
    for (&(color, verse), list) in by_cv.iter_mut() {
        list.sort();
        let chars = text.get(verse).map(|v| v.chars.as_slice()).unwrap_or(&[]);
        let mut out: Vec<Span> = Vec::new();
        for &(s, e, o) in list.iter() {
            if let Some(last) = out.last_mut()
                && (s <= last.1 || only_ws(chars, last.1, s))
            {
                last.1 = last.1.max(e);
                union(&mut parent, last.2, o);
                continue;
            }
            out.push((s, e, o));
        }
        merged.insert((color, verse), out);
    }
    // across verse boundaries
    for (&(color, verse), list) in merged.iter() {
        let (Some(next), Some(v)) = (text.next_verse(verse), text.get(verse)) else {
            continue;
        };
        let (Some(last), Some(nv)) = (list.last(), text.get(next)) else {
            continue;
        };
        let Some(first) = merged.get(&(color, next)).and_then(|l| l.first()) else {
            continue;
        };
        if only_ws(&v.chars, last.1, v.chars.len() as u32) && only_ws(&nv.chars, 0, first.0) {
            union(&mut parent, last.2, first.2);
        }
    }
    // gather groups
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut owner_of = vec![0usize; n];
    let mut out: Vec<Mark> = Vec::new();
    for members in groups.values() {
        let pick = match prefer {
            Some(p) if members.contains(&p) => p,
            _ => *members
                .iter()
                .min_by_key(|&&i| (marks[i].id.is_none(), marks[i].id))
                .unwrap(),
        };
        let idx = out.len();
        for &i in members {
            owner_of[i] = idx;
        }
        out.push(Mark {
            id: marks[pick].id,
            color: marks[pick].color.clone(),
            created_at: members.iter().map(|&i| marks[i].created_at).min().unwrap(),
            pieces: Vec::new(),
        });
    }
    for (&(_, verse), list) in &merged {
        for &(s, e, o) in list {
            out[owner_of[o]].pieces.push(Piece {
                verse,
                start: s,
                end: e,
            });
        }
    }
    for m in out.iter_mut() {
        m.pieces.sort();
    }
    out.retain(|m| !m.pieces.is_empty());
    out
}

/// The stored-state invariant (5.2, 11 E5): within a verse no two pieces
/// overlap; same-colour pieces never touch or sit only whitespace apart, and
/// same-colour pieces touching across a verse boundary belong to one mark;
/// every mark has a piece and every piece is valid against the text.
pub fn check_invariant(marks: &[Mark], text: &ChapterText) -> Result<(), String> {
    let mut by_verse: BTreeMap<u32, Vec<(u32, u32, &str, usize)>> = BTreeMap::new();
    for (i, m) in marks.iter().enumerate() {
        if m.pieces.is_empty() {
            return Err("a mark without pieces".into());
        }
        if !valid_color(&m.color) {
            return Err("unknown colour".into());
        }
        for p in &m.pieces {
            check_piece(text, p)?;
            by_verse
                .entry(p.verse)
                .or_default()
                .push((p.start, p.end, m.color.as_str(), i));
        }
    }
    for (verse, list) in by_verse.iter_mut() {
        list.sort();
        let chars = &text.get(*verse).unwrap().chars;
        for w in list.windows(2) {
            if w[1].0 < w[0].1 {
                return Err(format!("overlapping pieces in verse {verse}"));
            }
            if w[0].2 == w[1].2 && only_ws(chars, w[0].1, w[1].0) {
                return Err(format!("same-colour pieces not merged in verse {verse}"));
            }
        }
    }
    for (verse, list) in &by_verse {
        let (Some(next), Some(v)) = (text.next_verse(*verse), text.get(*verse)) else {
            continue;
        };
        let (Some(nlist), Some(nv)) = (by_verse.get(&next), text.get(next)) else {
            continue;
        };
        let (last, first) = (list.last().unwrap(), nlist.first().unwrap());
        if last.2 == first.2
            && last.3 != first.3
            && only_ws(&v.chars, last.1, v.chars.len() as u32)
            && only_ws(&nv.chars, 0, first.0)
        {
            return Err(format!(
                "same-colour marks touching across verses {verse} and {next}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(verses: &[&str]) -> ChapterText {
        let t: Vec<(u32, String)> = verses
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u32 + 1, s.to_string()))
            .collect();
        ChapterText::new(&t)
    }
    fn p(verse: u32, start: u32, end: u32) -> Piece {
        Piece { verse, start, end }
    }
    fn mk(id: Option<i64>, color: &str, created_at: i64, pieces: Vec<Piece>) -> Mark {
        Mark {
            id,
            color: color.into(),
            created_at,
            pieces,
        }
    }

    #[test]
    fn fingerprint_is_16_hex_of_sha256() {
        // SHA-256("abc") = ba7816bf8f01cfea...
        assert_eq!(fingerprint("abc"), "ba7816bf8f01cfea");
        assert_eq!(fingerprint("abc").len(), 16);
    }

    #[test]
    fn offsets_are_code_points_not_utf16() {
        // W2P-B3: an astral character is one code point (two UTF-16 units)
        let t = ch(&["\u{1D400}leph end"]);
        assert_eq!(t.len(1), Some(9));
        assert!(check_piece(&t, &p(1, 0, 5)).is_ok()); // U+1D400 plus "leph"
        assert!(check_piece(&t, &p(1, 6, 9)).is_ok());
        assert!(
            check_piece(&t, &p(1, 6, 10)).is_err(),
            "UTF-16 length is out of bounds"
        );
    }

    #[test]
    fn offset_bounds_and_wrong_verse_are_rejected() {
        let t = ch(&["alpha beta", "gamma"]);
        for bad in [
            p(1, 0, 0),
            p(1, 3, 2),
            p(1, 0, 11),
            p(1, u32::MAX - 1, u32::MAX),
            p(3, 0, 1),
            p(0, 0, 1),
        ] {
            assert!(check_piece(&t, &bad).is_err(), "{bad:?}");
        }
        assert!(check_piece(&t, &p(1, 0, 10)).is_ok());
    }

    #[test]
    fn b1_refuses_a_boundary_before_a_combining_mark_or_at_a_zwj() {
        let t = ch(&[
            "cafe\u{301} x",
            "a\u{200D}b",
            "\u{05D1}\u{05BC}\u{05B0}\u{05E8}",
        ]);
        assert!(check_piece(&t, &p(1, 0, 4)).is_err(), "end before U+0301");
        assert!(check_piece(&t, &p(1, 0, 5)).is_ok());
        assert!(check_piece(&t, &p(1, 4, 7)).is_err(), "start before U+0301");
        assert!(check_piece(&t, &p(2, 0, 1)).is_err(), "end before ZWJ");
        assert!(check_piece(&t, &p(2, 2, 3)).is_err(), "start after ZWJ");
        assert!(check_piece(&t, &p(2, 0, 3)).is_ok());
        // Hebrew: bet + dagesh + sheva, then resh
        assert!(check_piece(&t, &p(3, 0, 1)).is_err());
        assert!(check_piece(&t, &p(3, 0, 2)).is_err());
        assert!(check_piece(&t, &p(3, 0, 3)).is_ok());
        assert!(
            is_mark('\u{0591}') && is_mark('\u{05C7}') && !is_mark('\u{05D0}') && !is_mark('a')
        );
    }

    #[test]
    fn strokes_are_contiguous_across_verses() {
        let t = ch(&["alpha beta", "gamma delta", "epsilon"]);
        assert!(check_stroke(&t, &[p(1, 6, 10), p(2, 0, 11), p(3, 0, 3)]).is_ok());
        assert!(check_stroke(&t, &[]).is_err());
        assert!(
            check_stroke(&t, &[p(1, 6, 10), p(3, 0, 3)]).is_err(),
            "gap verse"
        );
        assert!(
            check_stroke(&t, &[p(1, 6, 9), p(2, 0, 3)]).is_err(),
            "first not to the end"
        );
        assert!(
            check_stroke(&t, &[p(1, 6, 10), p(2, 1, 3)]).is_err(),
            "last not from 0"
        );
        assert!(
            check_stroke(&t, &[p(1, 6, 10), p(2, 0, 5), p(3, 0, 3)]).is_err(),
            "inner not whole"
        );
        assert!(
            check_stroke(&t, &[p(2, 0, 3), p(1, 0, 3)]).is_err(),
            "unsorted"
        );
        let many = vec![p(1, 0, 1); MAX_STROKE_PIECES + 1];
        assert!(check_stroke(&t, &many).is_err());
    }

    #[test]
    fn last_write_wins_and_cuts_trim() {
        let t = ch(&["alpha beta gamma"]);
        let mut marks = vec![mk(Some(1), "gold", 5, vec![p(1, 0, 16)])];
        cut(&mut marks, &t, 1, 6, 10); // "beta"
        assert_eq!(
            marks[0].pieces,
            vec![p(1, 0, 5), p(1, 11, 16)],
            "spaces trimmed off the remnants"
        );
        marks.push(mk(None, "blue", 9, vec![p(1, 6, 10)]));
        let out = normalize(marks, &t, Some(1));
        assert_eq!(out.len(), 2);
        let gold = out.iter().find(|m| m.color == "gold").unwrap();
        assert_eq!(
            gold.pieces,
            vec![p(1, 0, 5), p(1, 11, 16)],
            "one mark, non-contiguous pieces"
        );
        check_invariant(&out, &t).unwrap();
    }

    #[test]
    fn same_colour_merges_over_whitespace_keeping_the_new_id_and_first_time() {
        let t = ch(&["alpha beta gamma delta"]);
        let marks = vec![
            mk(Some(4), "gold", 100, vec![p(1, 0, 5)]),
            mk(Some(7), "gold", 50, vec![p(1, 17, 22)]),
            mk(None, "gold", 200, vec![p(1, 6, 10)]),
        ];
        let out = normalize(marks, &t, Some(2));
        // "alpha" and "beta" join (one space between) under the new mark;
        // "delta" is a word away, so it stays its own mark
        assert_eq!(out.len(), 2);
        assert_eq!(
            out.iter().find(|m| m.id == Some(7)).unwrap().pieces,
            vec![p(1, 17, 22)]
        );
        let joined = out.iter().find(|m| m.id.is_none()).unwrap();
        assert_eq!(joined.pieces, vec![p(1, 0, 10)]);
        assert_eq!(
            joined.created_at, 100,
            "earliest created_at of the joined marks"
        );
        // overlapping same colour joins too
        let out = normalize(
            vec![
                mk(Some(1), "rose", 1, vec![p(1, 0, 10)]),
                mk(None, "rose", 2, vec![p(1, 6, 16)]),
            ],
            &t,
            Some(1),
        );
        assert_eq!(out, vec![mk(None, "rose", 1, vec![p(1, 0, 16)])]);
    }

    #[test]
    fn different_colours_butt_and_do_not_merge() {
        let t = ch(&["alpha beta"]);
        let out = normalize(
            vec![
                mk(Some(1), "gold", 1, vec![p(1, 0, 5)]),
                mk(None, "green", 2, vec![p(1, 6, 10)]),
            ],
            &t,
            Some(1),
        );
        assert_eq!(out.len(), 2);
        check_invariant(&out, &t).unwrap();
    }

    #[test]
    fn same_colour_across_a_verse_boundary_joins_marks_but_pieces_stay_per_verse() {
        let t = ch(&["alpha beta ", "gamma delta"]);
        let out = normalize(
            vec![
                mk(Some(3), "blue", 1, vec![p(1, 6, 10)]),
                mk(None, "blue", 2, vec![p(2, 0, 5)]),
            ],
            &t,
            Some(1),
        );
        assert_eq!(
            out,
            vec![mk(None, "blue", 1, vec![p(1, 6, 10), p(2, 0, 5)])]
        );
        // not adjacent (a word between): two marks
        let out = normalize(
            vec![
                mk(Some(3), "blue", 1, vec![p(1, 0, 5)]),
                mk(None, "blue", 2, vec![p(2, 0, 5)]),
            ],
            &t,
            Some(1),
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn the_invariant_catches_overlap_and_unmerged_neighbours() {
        let t = ch(&["alpha beta gamma"]);
        let a = mk(Some(1), "gold", 1, vec![p(1, 0, 5)]);
        assert!(
            check_invariant(&[a.clone(), mk(Some(2), "blue", 1, vec![p(1, 3, 10)])], &t).is_err()
        );
        assert!(
            check_invariant(&[a.clone(), mk(Some(2), "gold", 1, vec![p(1, 6, 10)])], &t).is_err()
        );
        assert!(
            check_invariant(&[a.clone(), mk(Some(2), "blue", 1, vec![p(1, 6, 10)])], &t).is_ok()
        );
        assert!(check_invariant(&[mk(Some(2), "blue", 1, vec![])], &t).is_err());
    }

    #[test]
    fn random_strokes_always_keep_the_invariant() {
        // E5: many overlapping writes in every colour, plus erasing, in a
        // deterministic pseudo-random order; the invariant holds after each.
        let t = ch(&[
            "In the beginning was the Word, and the Word was with God.",
            "The same was in the beginning with God. ",
            "All things were made by him",
        ]);
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut rnd = |n: u32| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n as u64) as u32
        };
        let mut marks: Vec<Mark> = Vec::new();
        let mut next_id = 1;
        for round in 0..400 {
            let v = 1 + rnd(3);
            let len = t.len(v).unwrap();
            let a = rnd(len);
            let b = a + 1 + rnd(len - a);
            let Some((a, b)) = trim(&t.get(v).unwrap().chars, a, b) else {
                continue;
            };
            let erase = rnd(5) == 0;
            cut(&mut marks, &t, v, a, b);
            let prefer = if erase {
                None
            } else {
                marks.push(mk(None, COLORS[rnd(5) as usize], round, vec![p(v, a, b)]));
                Some(marks.len() - 1)
            };
            marks = normalize(marks, &t, prefer);
            for m in marks.iter_mut() {
                if m.id.is_none() {
                    m.id = Some(next_id);
                    next_id += 1;
                }
            }
            check_invariant(&marks, &t).unwrap_or_else(|e| panic!("round {round}: {e}"));
            // and every character has at most one colour
            let mut seen = std::collections::HashSet::new();
            for m in &marks {
                for p in &m.pieces {
                    for i in p.start..p.end {
                        assert!(seen.insert((p.verse, i)), "round {round}: double colour");
                    }
                }
            }
        }
    }
}
