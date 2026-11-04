//! Stage 3, validate: every invariant of DESIGN.md 3–6 on the parsed model,
//! the versification check and the pinned facts. Errors are collected and
//! reported together (at most 100), then the build stops.
//!
//! Check names: V1–V7 (verse text, 3.2), P2/P4 (marks, 4), R1/R3–R6 (red
//! pieces, 5.3; R2 is structural and fires in the parser), B1 (5.1), T3
//! (versification), T8 (red facts), T13 (Strong's), `facts` (other pinned
//! counts).

use std::collections::BTreeMap;

use crate::error::{Collector, Error, Result};
use crate::osis::{MarkKind, Model, Verse};
use crate::profile::Profile;
use crate::strongs::Lang;
use crate::vrs::Table;

/// Red totals for one book: (q by start-tag book, pieces, code points).
pub type RedTotals = BTreeMap<u8, (u32, u32, u64)>;

/// Per-book red totals computed from the model.
pub fn red_totals(m: &Model) -> RedTotals {
    let mut t: RedTotals = BTreeMap::new();
    for q in &m.qs {
        t.entry(q.book).or_default().0 += 1;
    }
    for v in &m.verses {
        for r in &v.red {
            let e = t.entry(v.book).or_default();
            e.1 += 1;
            e.2 += u64::from(r.end - r.start);
        }
    }
    t
}

fn verr(check: &'static str, v: &Verse, msg: impl Into<String>) -> Error {
    Error::new(check, msg).verse(v.osis.clone())
}

fn is_v6(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{FEFF}' | '\u{FFFD}' | '\u{2028}' | '\u{2029}')
        || ('\u{202A}'..='\u{202E}').contains(&c)
        || ('\u{2066}'..='\u{2069}').contains(&c)
}

fn text_rules(v: &Verse, p: &Profile, c: &mut Collector) {
    if v.len == 0 || v.len > 4096 {
        c.push(verr(
            "V1",
            v,
            format!("stored text has {} code points (1..=4096)", v.len),
        ));
    }
    for ch in v.text.chars() {
        if ch.is_whitespace() && ch != ' ' {
            c.push(verr(
                "V2",
                v,
                format!("whitespace U+{:04X} other than U+0020", ch as u32),
            ));
            return;
        }
        if is_v6(ch) {
            c.push(verr(
                "V6",
                v,
                format!("forbidden character U+{:04X}", ch as u32),
            ));
            return;
        }
        if !(' '..='~').contains(&ch) && !p.chars.iter().any(|a| a.c == ch) {
            c.push(verr(
                "V7",
                v,
                format!("U+{:04X} is not in the profile's allowlist", ch as u32),
            ));
            return;
        }
    }
    if v.text.starts_with(' ') {
        c.push(verr("V3", v, "leading space"));
    }
    if v.text.contains("  ") {
        c.push(verr("V4", v, "two consecutive spaces"));
    }
}

/// Book main titles (DESIGN.md 8, `book.main_title`): stored raw, so only
/// checked, never changed: 1..=256 code points, printable ASCII or the
/// profile's allowlist, U+0020 the only whitespace, no V6 character. A
/// profile with `main_titles` requires one per book.
fn title_rules(m: &Model, p: &Profile, c: &mut Collector) {
    for b in &m.books {
        let Some(t) = &b.main_title else {
            if p.main_titles {
                c.push(Error::new("title", format!("{}: no main title", b.osis)));
            }
            continue;
        };
        let n = t.chars().count();
        let bad = t.chars().find(|&ch| {
            (ch.is_whitespace() && ch != ' ')
                || is_v6(ch)
                || (!(' '..='~').contains(&ch) && !p.chars.iter().any(|a| a.c == ch))
        });
        if n == 0 || n > 256 {
            c.push(Error::new(
                "title",
                format!("{}: main title has {n} code points (1..=256)", b.osis),
            ));
        } else if let Some(ch) = bad {
            c.push(Error::new(
                "title",
                format!(
                    "{}: U+{:04X} is not allowed in a main title",
                    b.osis, ch as u32
                ),
            ));
        }
    }
}

/// B1: no boundary (other than 0 and the length) immediately before an `M*`
/// character, or immediately before or after U+200D.
fn b1(v: &Verse, p: &Profile, c: &mut Collector) {
    let special = |ch: char| ch == '\u{200D}' || p.chars.iter().any(|a| a.c == ch && a.is_mark);
    if !v.text.chars().any(special) {
        return;
    }
    let chars: Vec<char> = v.text.chars().collect();
    let bad = |b: u32| {
        let b = b as usize;
        if b == 0 || b >= chars.len() {
            return false;
        }
        let after = chars[b];
        let before = chars[b - 1];
        p.chars.iter().any(|a| a.c == after && a.is_mark)
            || after == '\u{200D}'
            || before == '\u{200D}'
    };
    let mut bounds: Vec<(&str, u32)> = Vec::new();
    for r in &v.red {
        bounds.push(("red piece", r.start));
        bounds.push(("red piece", r.end));
    }
    for m in &v.marks {
        bounds.push(("mark", m.start));
        bounds.push(("mark", m.end));
    }
    for w in &v.words {
        bounds.push(("word", w.start));
        bounds.push(("word", w.end));
    }
    for (what, b) in bounds {
        if bad(b) {
            c.push(verr(
                "B1",
                v,
                format!("{what} boundary at {b} splits a grapheme"),
            ));
            return;
        }
    }
}

fn red_rules(v: &Verse, p: &Profile, c: &mut Collector) {
    let chars: Option<Vec<char>> = if p.kjv_red_boundaries && !v.red.is_empty() {
        Some(v.text.chars().collect())
    } else {
        None
    };
    for (i, r) in v.red.iter().enumerate() {
        if !(r.start < r.end && r.end <= v.len) {
            c.push(verr(
                "R1",
                v,
                format!("piece [{}, {}) empty or outside the verse", r.start, r.end),
            ));
        }
        if i > 0 && r.start <= v.red[i - 1].end {
            c.push(verr(
                "R3",
                v,
                "pieces overlap or touch (no character between them)",
            ));
        }
        if let Some(chars) = &chars {
            let (s, e) = (r.start as usize, r.end as usize);
            if e > chars.len() || s >= e {
                continue;
            }
            let start_ok = s == 0 || chars[s - 1] == ' ';
            let end_ok = e == chars.len() || chars[e] == ' ';
            if !start_ok || !end_ok {
                c.push(verr(
                    "R4",
                    v,
                    format!("boundary of [{s}, {e}) is not at a verse edge or next to U+0020"),
                ));
            }
            if chars[s].is_whitespace() || chars[e - 1].is_whitespace() {
                c.push(verr(
                    "R5",
                    v,
                    format!("piece [{s}, {e}) starts or ends with whitespace"),
                ));
            }
        }
    }
    // P4: a mark is inside one piece or disjoint from all of them.
    for m in &v.marks {
        if m.start >= m.end {
            c.push(verr("P2", v, "empty mark"));
        }
        for r in &v.red {
            let inside = r.start <= m.start && m.end <= r.end;
            let disjoint = m.end <= r.start || r.end <= m.start;
            if !inside && !disjoint {
                c.push(verr(
                    "P4",
                    v,
                    format!(
                        "mark [{}, {}) crosses red piece [{}, {})",
                        m.start, m.end, r.start, r.end
                    ),
                ));
            }
        }
        if m.end > v.len {
            c.push(verr("P2", v, "mark ends after the verse"));
        }
    }
    for w in &v.words {
        if w.start > w.end || w.end > v.len {
            c.push(verr("T13", v, "word range outside the verse"));
        }
    }
}

/// R6 on the model: the pieces of one q_ord lie in consecutive verses, one
/// per verse; every piece but the first starts at 0, every piece but the
/// last ends at the verse length; q_ord runs 1..N.
fn r6(m: &Model, c: &mut Collector) {
    let mut by_q: BTreeMap<u32, Vec<(usize, u32, u32, u32)>> = BTreeMap::new();
    for (vi, v) in m.verses.iter().enumerate() {
        for r in &v.red {
            by_q.entry(r.q_ord)
                .or_default()
                .push((vi, r.start, r.end, v.len));
        }
    }
    let n = m.qs.len() as u32;
    if by_q.len() as u32 != n
        || by_q.keys().next().is_some_and(|&k| k != 1)
        || by_q.keys().last().is_some_and(|&k| k != n)
    {
        c.push(Error::new(
            "R6",
            format!("q_ord values are not exactly 1..{n} (some q has no piece)"),
        ));
    }
    for (q, pieces) in &by_q {
        for (k, &(vi, s, e, len)) in pieces.iter().enumerate() {
            let v = &m.verses[vi];
            if k > 0 && vi != pieces[k - 1].0 + 1 {
                c.push(verr(
                    "R6",
                    v,
                    format!("q {q}: pieces not in consecutive verses"),
                ));
            }
            if k > 0 && s != 0 {
                c.push(verr(
                    "R6",
                    v,
                    format!("q {q}: a continuing piece does not start at 0"),
                ));
            }
            if k + 1 < pieces.len() && e != len {
                c.push(verr(
                    "R6",
                    v,
                    format!("q {q}: a continued piece does not end at the verse end"),
                ));
            }
        }
    }
}

fn strongs_rules(m: &Model, p: &Profile, c: &mut Collector) {
    if !p.testament_rule {
        return;
    }
    for v in &m.verses {
        let want = if v.book <= 39 {
            Lang::Hebrew
        } else {
            Lang::Greek
        };
        if v.words
            .iter()
            .flat_map(|w| &w.keys)
            .any(|k| k.lang() != want)
        {
            c.push(verr("T13", v, "Strong's key from the wrong testament"));
        }
    }
}

/// The versification check (T3 on the model): books in canonical order, and
/// every chapter's verse count equal to the table.
pub fn versification(m: &Model, table: &Table, c: &mut Collector) {
    let books: Vec<&str> = m.books.iter().map(|b| b.usfm.as_str()).collect();
    let want: Vec<&str> = table.books.iter().map(|(b, _)| b.as_str()).collect();
    if books != want {
        c.push(Error::new(
            "T3",
            "books differ from the versification table",
        ));
        return;
    }
    let mut counts: BTreeMap<(u8, u16), u16> = BTreeMap::new();
    for v in &m.verses {
        *counts.entry((v.book, v.chapter)).or_default() += 1;
    }
    for (bi, (usfm, chapters)) in table.books.iter().enumerate() {
        let b = &m.books[bi];
        if usize::from(b.chapters) != chapters.len() {
            c.push(Error::new(
                "T3",
                format!(
                    "{usfm}: {} chapters, table has {}",
                    b.chapters,
                    chapters.len()
                ),
            ));
            continue;
        }
        for (ci, &n) in chapters.iter().enumerate() {
            let got = counts
                .get(&((bi + 1) as u8, (ci + 1) as u16))
                .copied()
                .unwrap_or(0);
            if got != n {
                c.push(Error::new(
                    "T3",
                    format!("{usfm} {}: {got} verses, table has {n}", ci + 1),
                ));
            }
        }
    }
}

/// The pinned facts (T6 V5, T8, and counts) — KJV only.
fn facts(m: &Model, p: &Profile, c: &mut Collector) {
    let Some(f) = &p.facts else { return };
    let mut fact = |check: &'static str, what: &str, got: usize, want: usize| {
        if got != want {
            c.push(Error::new(
                check,
                format!("{what}: source has {got}, pinned fact is {want}"),
            ));
        }
    };
    fact("facts", "verses", m.verses.len(), f.verses);
    fact(
        "title",
        "books with a main title",
        m.books.iter().filter(|b| b.main_title.is_some()).count(),
        f.main_titles,
    );
    fact(
        "facts",
        "chapters",
        m.books.iter().map(|b| usize::from(b.chapters)).sum(),
        f.chapters,
    );
    let trailing = m.verses.iter().filter(|v| v.text.ends_with(' ')).count();
    fact(
        "V5",
        "verses ending with a space",
        trailing,
        f.trailing_space_verses,
    );
    let marks = |k: MarkKind| {
        m.verses
            .iter()
            .flat_map(|v| &v.marks)
            .filter(|x| x.kind == k)
            .count()
    };
    fact(
        "facts",
        "added marks",
        marks(MarkKind::Added),
        f.marks_added,
    );
    fact(
        "facts",
        "divine marks",
        marks(MarkKind::Divine),
        f.marks_divine,
    );
    fact(
        "facts",
        "words",
        m.verses.iter().map(|v| v.words.len()).sum(),
        f.words,
    );
    let empty = m
        .verses
        .iter()
        .flat_map(|v| &v.words)
        .filter(|w| w.start == w.end)
        .count();
    fact("facts", "empty words", empty, f.empty_words);
    fact(
        "T8",
        "verses with red",
        m.verses.iter().filter(|v| !v.red.is_empty()).count(),
        f.red_verses,
    );
    fact(
        "T8",
        "verses with two pieces",
        m.verses.iter().filter(|v| v.red.len() == 2).count(),
        f.two_piece_verses,
    );
    let totals = red_totals(m);
    for b in &m.books {
        let got = totals.get(&b.ord).copied().unwrap_or_default();
        let want = f
            .red
            .iter()
            .find(|r| r.book == b.osis)
            .map(|r| (r.q, r.pieces, r.cp))
            .unwrap_or_default();
        if got != want {
            c.push(Error::new(
                "T8",
                format!(
                    "{}: q/pieces/red code points {:?}, pinned fact {:?}",
                    b.osis, got, want
                ),
            ));
        }
    }
    let total = totals
        .values()
        .fold((0u32, 0u32, 0u64), |a, t| (a.0 + t.0, a.1 + t.1, a.2 + t.2));
    let want = f.red.iter().fold((0u32, 0u32, 0u64), |a, r| {
        (a.0 + r.q, a.1 + r.pieces, a.2 + r.cp)
    });
    if total != want || total != (648, 2035, 210_502) {
        c.push(Error::new(
            "T8",
            format!("red totals {total:?}, pinned {want:?}"),
        ));
    }
}

/// All stage-3 checks. `table` is the versification table (with overrides);
/// fixtures pass `None`.
pub fn validate(m: &Model, p: &Profile, table: Option<&Table>) -> Result<()> {
    let mut c = Collector::default();
    for v in &m.verses {
        text_rules(v, p, &mut c);
        red_rules(v, p, &mut c);
        b1(v, p, &mut c);
    }
    r6(m, &mut c);
    title_rules(m, p, &mut c);
    strongs_rules(m, p, &mut c);
    if let Some(t) = table {
        versification(m, t, &mut c);
    }
    facts(m, p, &mut c);
    c.finish()
}
