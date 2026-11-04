//! Golden samples (DESIGN.md 12; T18, T19): a strict, read-only scan of the
//! scholar's section 10 YAML block, the golden TSV files, and the values the
//! built database gives for each reference.
//!
//! Derived values, as the scholar's file defines them (section 10, "How to
//! read this block"): `spans` = red pieces in the verse; `start` is about
//! colour: `mid` if the first piece starts after 0, otherwise `carried` when
//! the previous verse in canonical order ends red and `verse_start` when it
//! doesn't; `end` = `verse_end` if the last piece ends at the verse end, else
//! `mid`; first/last words are the whitespace-separated tokens at the edges
//! of a piece's text, punctuation included; `black_between` is the black text
//! between two pieces; `black_after_first` the first word after a piece that
//! ends mid-verse; `q_open_at_start`/`q_open_at_end` describe the `<q>`
//! structure (the piece continues a q from the previous verse / continues
//! into the next verse).

use std::collections::BTreeMap;

use rusqlite::Connection;

use crate::error::Error;
use crate::profile::KJV_BOOKS;

/// One reference's `source_expected` map, as the scholar wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// USFM reference, e.g. `MAT.3.15`.
    pub usfm: String,
    pub fields: BTreeMap<String, String>,
}

fn gerr(msg: impl Into<String>) -> Error {
    Error::new("T18", msg).file("red-letter-golden-samples.md")
}

fn parse_map(s: &str, at: &str) -> Result<BTreeMap<String, String>, Error> {
    let body = s
        .strip_prefix('{')
        .and_then(|b| b.strip_suffix('}'))
        .ok_or_else(|| gerr(format!("{at}: source_expected is not a flow map")))?;
    let mut out = BTreeMap::new();
    let mut rest = body;
    while !rest.is_empty() {
        let (key, after) = rest
            .split_once(": ")
            .ok_or_else(|| gerr(format!("{at}: expected 'key: value'")))?;
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(gerr(format!("{at}: bad key")));
        }
        let (value, tail) = if let Some(q) = after.strip_prefix('"') {
            let end = q
                .find('"')
                .ok_or_else(|| gerr(format!("{at}: unterminated quote")))?;
            (&q[..end], &q[end + 1..])
        } else {
            match after.find(", ") {
                Some(i) => (&after[..i], &after[i..]),
                None => (after, ""),
            }
        };
        if value.contains('\\') || value.contains('\t') {
            return Err(gerr(format!("{at}: escape or tab in a value")));
        }
        if out.insert(key.to_string(), value.to_string()).is_some() {
            return Err(gerr(format!("{at}: duplicate key {key}")));
        }
        rest = match tail {
            "" => "",
            t => t
                .strip_prefix(", ")
                .ok_or_else(|| gerr(format!("{at}: expected ', '")))?,
        };
    }
    Ok(out)
}

/// The rows of the scholar's section 10 YAML block, in order.
pub fn scholar_rows(md: &str) -> Result<Vec<Row>, Error> {
    let sec = md
        .split_once("\n## 10. ")
        .ok_or_else(|| gerr("no section 10"))?
        .1;
    let block = sec
        .split_once("```yaml\n")
        .and_then(|(_, b)| b.split_once("\n```"))
        .ok_or_else(|| gerr("no yaml block in section 10"))?
        .0;
    let mut rows: Vec<Row> = Vec::new();
    let mut have_expect: Vec<bool> = Vec::new();
    for (i, line) in block.split('\n').enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(r) = line.strip_prefix("- ref: ") {
            let parts: Vec<&str> = r.split('.').collect();
            let ok = parts.len() == 3
                && KJV_BOOKS.iter().any(|(_, u)| *u == parts[0])
                && parts[1..].iter().all(|p| {
                    !p.is_empty() && !p.starts_with('0') && p.bytes().all(|b| b.is_ascii_digit())
                });
            if !ok {
                return Err(gerr(format!("line {}: bad ref", i + 1)));
            }
            if rows.iter().any(|x| x.usfm == r) {
                return Err(gerr(format!("duplicate ref {r}")));
            }
            rows.push(Row {
                usfm: r.to_string(),
                fields: BTreeMap::new(),
            });
            have_expect.push(false);
            continue;
        }
        let Some(kv) = line.strip_prefix("  ") else {
            return Err(gerr(format!("line {}: unknown shape", i + 1)));
        };
        let (k, v) = kv
            .split_once(": ")
            .ok_or_else(|| gerr(format!("line {}: unknown shape", i + 1)))?;
        if rows.is_empty() || !k.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(gerr(format!("line {}: unknown shape", i + 1)));
        }
        if k == "source_expected" {
            let n = rows.len() - 1;
            if have_expect[n] {
                return Err(gerr(format!("{}: two source_expected", rows[n].usfm)));
            }
            rows[n].fields = parse_map(v, &rows[n].usfm)?;
            have_expect[n] = true;
        }
    }
    if let Some(i) = have_expect.iter().position(|h| !h) {
        return Err(gerr(format!("{} has no source_expected", rows[i].usfm)));
    }
    Ok(rows)
}

/// The columns of red-letter-source.tsv after `ref` and `osis`.
pub const COLUMNS: [&str; 15] = [
    "spans",
    "start",
    "end",
    "first",
    "last",
    "span1_first",
    "span1_last",
    "black_between",
    "span2_first",
    "span2_last",
    "black_after_first",
    "q_open_at_start",
    "q_open_at_end",
    "status",
    "human",
];

/// Parses red-letter-source.tsv into rows (fields with value `-` omitted).
pub fn tsv_rows(tsv: &str) -> Result<Vec<Row>, Error> {
    let mut out = Vec::new();
    let mut header_seen = false;
    for line in tsv.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        if !header_seen {
            let mut want = vec!["ref", "osis"];
            want.extend(COLUMNS);
            if f != want {
                return Err(Error::new(
                    "T18",
                    "red-letter-source.tsv: unexpected column header",
                ));
            }
            header_seen = true;
            continue;
        }
        if f.len() != COLUMNS.len() + 2 {
            return Err(Error::new(
                "T18",
                "red-letter-source.tsv: wrong column count",
            ));
        }
        let mut fields = BTreeMap::new();
        for (k, v) in COLUMNS.iter().zip(&f[2..]) {
            if *v != "-" {
                fields.insert(k.to_string(), v.to_string());
            }
        }
        out.push(Row {
            usfm: f[0].to_string(),
            fields,
        });
    }
    Ok(out)
}

/// `MAT.3.15` → (verse id, `Matt.3.15`).
pub fn usfm_ref(r: &str) -> Option<(i64, String)> {
    let mut it = r.split('.');
    let (b, c, v) = (
        it.next()?,
        it.next()?.parse::<i64>().ok()?,
        it.next()?.parse::<i64>().ok()?,
    );
    let ord = KJV_BOOKS.iter().position(|(_, u)| *u == b)? as i64 + 1;
    let osis = KJV_BOOKS[(ord - 1) as usize].0;
    Some((ord * 1_000_000 + c * 1000 + v, format!("{osis}.{c}.{v}")))
}

/// A verse's red pieces from the database: (n, start, end, q_ord).
pub fn pieces(conn: &Connection, id: i64) -> rusqlite::Result<Vec<(i64, usize, usize, i64)>> {
    let mut st = conn.prepare(
        "SELECT n, start_cp, end_cp, q_ord FROM red_piece WHERE verse_id = ?1 ORDER BY n",
    )?;
    let rows = st.query_map([id], |r| {
        Ok((
            r.get(0)?,
            r.get::<_, i64>(1)? as usize,
            r.get::<_, i64>(2)? as usize,
            r.get(3)?,
        ))
    })?;
    rows.collect()
}

fn slice(chars: &[char], s: usize, e: usize) -> String {
    chars[s..e].iter().collect()
}

/// The derived golden values of verse `id` from the database.
pub fn derived(conn: &Connection, id: i64) -> rusqlite::Result<BTreeMap<String, String>> {
    let text: String =
        conn.query_row("SELECT text FROM verse WHERE id = ?1", [id], |r| r.get(0))?;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let ps = pieces(conn, id)?;
    let mut out = BTreeMap::new();
    let put = |out: &mut BTreeMap<String, String>, k: &str, v: String| {
        out.insert(k.to_string(), v);
    };
    put(&mut out, "spans", ps.len().to_string());
    // Previous and next verse in canonical order.
    let prev: Option<(i64, usize, usize)> = conn
        .query_row(
            "SELECT v.id, length(v.text), coalesce((SELECT max(end_cp) FROM red_piece r WHERE r.verse_id = v.id), -1)
               FROM verse v WHERE v.id < ?1 ORDER BY v.id DESC LIMIT 1",
            [id],
            |r| Ok((r.get(0)?, r.get::<_, i64>(1)? as usize, r.get::<_, i64>(2)?.max(0) as usize)),
        )
        .ok();
    let q_has = |q: i64, cmp: &str| -> rusqlite::Result<bool> {
        let n: i64 = conn.query_row(
            &format!("SELECT count(*) FROM red_piece WHERE q_ord = ?1 AND verse_id {cmp} ?2"),
            [q, id],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    let (mut open_start, mut open_end) = (false, false);
    if let (Some(first), Some(last)) = (ps.first(), ps.last()) {
        let start = if first.1 > 0 {
            "mid"
        } else if prev.is_some_and(|(_, plen, pend)| pend == plen && plen > 0) {
            "carried"
        } else {
            "verse_start"
        };
        put(&mut out, "start", start.into());
        put(
            &mut out,
            "end",
            if last.2 == len { "verse_end" } else { "mid" }.into(),
        );
        let words = |s: usize, e: usize| -> Vec<String> {
            slice(&chars, s, e)
                .split(' ')
                .filter(|w| !w.is_empty())
                .map(str::to_string)
                .collect()
        };
        if ps.len() == 1 {
            let w = words(first.1, first.2);
            put(&mut out, "first", w.first().cloned().unwrap_or_default());
            put(&mut out, "last", w.last().cloned().unwrap_or_default());
        } else {
            for (i, p) in ps.iter().enumerate().take(2) {
                let w = words(p.1, p.2);
                put(
                    &mut out,
                    &format!("span{}_first", i + 1),
                    w.first().cloned().unwrap_or_default(),
                );
                put(
                    &mut out,
                    &format!("span{}_last", i + 1),
                    w.last().cloned().unwrap_or_default(),
                );
            }
            put(&mut out, "black_between", words(ps[0].2, ps[1].1).join(" "));
        }
        if let Some(p) = ps.iter().find(|p| p.2 < len)
            && let Some(w) = words(p.2, len).first()
        {
            put(&mut out, "black_after_first", w.clone());
        }
        open_start = first.1 == 0 && q_has(first.3, "<")?;
        open_end = last.2 == len && q_has(last.3, ">")?;
    }
    put(&mut out, "q_open_at_start", open_start.to_string());
    put(&mut out, "q_open_at_end", open_end.to_string());
    Ok(out)
}

/// Compares one expected row with the derived values: every expected field
/// except `status`, `human` and `basis` must be present and equal. Returns
/// the differences as `field: expected → database`.
pub fn diff(expected: &Row, got: &BTreeMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in &expected.fields {
        if matches!(k.as_str(), "status" | "human" | "basis") {
            continue;
        }
        let g = got.get(k).map(String::as_str).unwrap_or("<absent>");
        if g != v {
            out.push(format!(
                "{k}: {} → {}",
                crate::error::show_ws(v),
                crate::error::show_ws(g)
            ));
        }
    }
    out
}
