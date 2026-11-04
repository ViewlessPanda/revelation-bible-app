//! Daily notes (W2-5, wave-2-layout 8): the checks on what the UI sends.
//!
//! A day's note is the ProseMirror document as JSON (`Node.toJSON`), stored
//! in `user.sqlite` table `note_day` (see `user.rs`, migration v4). The
//! backend never trusts it: before a save it checks the size, the date and
//! that the JSON is exactly the wave-2 schema (doc, paragraph, two list types,
//! list item, text, hard break; bold and italic marks; passage tags as a block
//! attribute). Anything else (an unknown node or mark, a link, an HTML string
//! smuggled into an attribute, an extra key, a too-deep tree) is refused, so a
//! stored note can only ever render as plain text with those four formats.
//! The same check runs again on read: a row that fails it (edited by hand,
//! damaged) is reported as unreadable and never handed to the editor.

use crate::user::valid_book_id;
use serde_json::{Map, Value};

/// The largest note one day may hold, in bytes of JSON (about 60,000 words of
/// plain text, far beyond a day's writing; a size bound).
pub const MAX_NOTE_BYTES: usize = 512 * 1024;
/// Nodes in one day's document (blocks, list items, text runs, breaks).
pub const MAX_NODES: usize = 20_000;
/// Nesting depth of nodes (doc > list > item > list > ... > text).
pub const MAX_DEPTH: usize = 24;
/// Passage tags on one block (wave-2-layout 8.4: at most 3).
pub const MAX_TAGS: usize = 3;
/// Days returned by one `notes_list` call (8.1 loads 30 at a time).
pub const MAX_PAGE: u32 = 100;

/// What a valid document holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocInfo {
    /// Characters of text (code points); 0 means the day is empty and is not kept.
    pub chars: usize,
}

/// `YYYY-MM-DD`, a real calendar date in 1970..=2100 (the UI's local date).
pub fn valid_day(s: &str) -> bool {
    let b = s.as_bytes();
    if !s.is_ascii() || b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| -> Option<u32> {
        let p = &s[r];
        p.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| p.parse().ok())?
    };
    let (Some(y), Some(m), Some(d)) = (digits(0..4), digits(5..7), digits(8..10)) else {
        return false;
    };
    if !(1970..=2100).contains(&y) || !(1..=12).contains(&m) || d == 0 {
        return false;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    d <= days
}

struct Walk {
    nodes: usize,
    chars: usize,
}

type R = Result<(), String>;

fn only_keys(o: &Map<String, Value>, allowed: &[&str], what: &str) -> R {
    match o.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(format!("{what}: unexpected key {k:?}")),
        None => Ok(()),
    }
}

fn int_in(v: &Value, lo: u64, hi: u64) -> Option<u64> {
    v.as_u64().filter(|n| (lo..=hi).contains(n))
}

/// `tags`: null, or 0..=3 of `{book, chapter, v1, v2}`. `v1 = v2 = 0` is the
/// whole chapter; otherwise `1 <= v1 <= v2 <= 999`.
fn check_tags(v: Option<&Value>) -> R {
    let Some(v) = v else { return Ok(()) };
    if v.is_null() {
        return Ok(());
    }
    let list = v.as_array().ok_or("tags: not a list")?;
    if list.len() > MAX_TAGS {
        return Err("tags: more than 3".into());
    }
    for t in list {
        let o = t.as_object().ok_or("tag: not an object")?;
        only_keys(o, &["book", "chapter", "v1", "v2"], "tag")?;
        let book = o.get("book").and_then(Value::as_str).ok_or("tag: book")?;
        if !valid_book_id(book) {
            return Err("tag: book".into());
        }
        o.get("chapter")
            .and_then(|c| int_in(c, 1, 999))
            .ok_or("tag: chapter")?;
        let v1 = o
            .get("v1")
            .and_then(|c| int_in(c, 0, 999))
            .ok_or("tag: v1")?;
        let v2 = o
            .get("v2")
            .and_then(|c| int_in(c, 0, 999))
            .ok_or("tag: v2")?;
        let whole = v1 == 0 && v2 == 0;
        if !whole && (v1 == 0 || v2 < v1) {
            return Err("tag: verse range".into());
        }
    }
    Ok(())
}

/// The `content` array of a node (absent = empty).
fn content<'a>(o: &'a Map<String, Value>, what: &str) -> Result<&'a [Value], String> {
    match o.get("content") {
        None => Ok(&[]),
        Some(Value::Array(a)) => Ok(a.as_slice()),
        Some(_) => Err(format!("{what}: content is not a list")),
    }
}

fn node_type(v: &Value) -> Result<(&Map<String, Value>, &str), String> {
    let o = v.as_object().ok_or("node: not an object")?;
    let t = o
        .get("type")
        .and_then(Value::as_str)
        .ok_or("node: no type")?;
    Ok((o, t))
}

impl Walk {
    fn count(&mut self, depth: usize) -> R {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("too many nodes".into());
        }
        if depth > MAX_DEPTH {
            return Err("nested too deeply".into());
        }
        Ok(())
    }

    /// A block: paragraph, bullet_list or ordered_list.
    fn block(&mut self, v: &Value, depth: usize) -> R {
        self.count(depth)?;
        let (o, t) = node_type(v)?;
        match t {
            "paragraph" => {
                only_keys(o, &["type", "attrs", "content"], t)?;
                self.attrs(o, &["tags"], t)?;
                for c in content(o, t)? {
                    self.inline(c, depth + 1)?;
                }
                Ok(())
            }
            "bullet_list" | "ordered_list" => {
                only_keys(o, &["type", "attrs", "content"], t)?;
                if t == "ordered_list" {
                    self.attrs(o, &["order", "tags"], t)?;
                    if let Some(n) = o.get("attrs").and_then(|a| a.get("order"))
                        && int_in(n, 0, 99_999).is_none()
                    {
                        return Err("ordered_list: order".into());
                    }
                } else {
                    self.attrs(o, &["tags"], t)?;
                }
                let items = content(o, t)?;
                if items.is_empty() {
                    return Err(format!("{t}: no items"));
                }
                for c in items {
                    self.item(c, depth + 1)?;
                }
                Ok(())
            }
            other => Err(format!("node type {other:?} is not allowed here")),
        }
    }

    /// A list item: a paragraph, then any blocks.
    fn item(&mut self, v: &Value, depth: usize) -> R {
        self.count(depth)?;
        let (o, t) = node_type(v)?;
        if t != "list_item" {
            return Err(format!("node type {t:?} is not allowed in a list"));
        }
        only_keys(o, &["type", "content"], t)?;
        let kids = content(o, t)?;
        match kids.first().map(node_type).transpose()? {
            Some((_, "paragraph")) => {}
            _ => return Err("list_item: must start with a paragraph".into()),
        }
        for c in kids {
            self.block(c, depth + 1)?;
        }
        Ok(())
    }

    fn inline(&mut self, v: &Value, depth: usize) -> R {
        self.count(depth)?;
        let (o, t) = node_type(v)?;
        match t {
            "text" => {
                only_keys(o, &["type", "text", "marks"], t)?;
                let s = o
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or("text: no text")?;
                if s.is_empty() {
                    return Err("text: empty".into());
                }
                self.chars += s.chars().count();
                if let Some(m) = o.get("marks") {
                    let ms = m.as_array().ok_or("marks: not a list")?;
                    if ms.len() > 2 {
                        return Err("marks: too many".into());
                    }
                    for mk in ms {
                        let mo = mk.as_object().ok_or("mark: not an object")?;
                        only_keys(mo, &["type"], "mark")?;
                        match mo.get("type").and_then(Value::as_str) {
                            Some("strong" | "em") => {}
                            other => return Err(format!("mark {other:?} is not allowed")),
                        }
                    }
                }
                Ok(())
            }
            "hard_break" => only_keys(o, &["type"], t),
            other => Err(format!("node type {other:?} is not allowed inline")),
        }
    }

    fn attrs(&mut self, o: &Map<String, Value>, allowed: &[&str], what: &str) -> R {
        let Some(a) = o.get("attrs") else {
            return Ok(());
        };
        let a = a.as_object().ok_or(format!("{what}: attrs"))?;
        only_keys(a, allowed, what)?;
        check_tags(a.get("tags"))
    }
}

/// Check a day's document (JSON text) against the size limit and the wave-2
/// schema. Returns how much text it holds.
pub fn check_doc(json: &str) -> Result<DocInfo, String> {
    if json.len() > MAX_NOTE_BYTES {
        return Err(format!(
            "the note is too long ({} KB; the limit is {} KB)",
            json.len().div_ceil(1024),
            MAX_NOTE_BYTES / 1024
        ));
    }
    // serde_json refuses nesting beyond 128 levels on its own (no stack overflow)
    let v: Value = serde_json::from_str(json).map_err(|e| format!("not a note document: {e}"))?;
    let (o, t) = node_type(&v)?;
    if t != "doc" {
        return Err("not a note document".into());
    }
    only_keys(o, &["type", "content"], "doc")?;
    let mut w = Walk { nodes: 1, chars: 0 };
    let blocks = content(o, "doc")?;
    if blocks.is_empty() {
        return Err("doc: no blocks".into());
    }
    for b in blocks {
        w.block(b, 1)?;
    }
    Ok(DocInfo { chars: w.chars })
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: &str = r#"{"type":"doc","content":[{"type":"paragraph","attrs":{"tags":[{"book":"John","chapter":3,"v1":16,"v2":18}]},"content":[{"type":"text","text":"Hello "},{"type":"text","marks":[{"type":"strong"},{"type":"em"}],"text":"world"},{"type":"hard_break"},{"type":"text","text":"x"}]},{"type":"bullet_list","attrs":{"tags":null},"content":[{"type":"list_item","content":[{"type":"paragraph","attrs":{"tags":null},"content":[{"type":"text","text":"item"}]},{"type":"ordered_list","attrs":{"order":1,"tags":null},"content":[{"type":"list_item","content":[{"type":"paragraph"}]}]}]}]},{"type":"paragraph","attrs":{"tags":[{"book":"Ps","chapter":23,"v1":0,"v2":0}]}}]}"#;

    #[test]
    fn days() {
        for ok in [
            "2026-10-27",
            "2024-02-29",
            "2000-02-29",
            "1970-01-01",
            "2100-12-31",
        ] {
            assert!(valid_day(ok), "{ok}");
        }
        for bad in [
            "",
            "2026-1-27",
            "2026-10-32",
            "2026-13-01",
            "2026-00-10",
            "2026-10-00",
            "2023-02-29",
            "1900-02-29",
            "1969-12-31",
            "2101-01-01",
            "2026/10/27",
            "2026-10-27 ",
            "+026-10-27",
            "ï¼’ï¼ï¼’ï¼–-10-27",
            "2026-1ï¼-27",
        ] {
            assert!(!valid_day(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_wave_2_schema_is_accepted_and_text_is_counted() {
        assert_eq!(
            check_doc(P).unwrap().chars,
            "Hello worldxitem".chars().count()
        );
        let empty = r#"{"type":"doc","content":[{"type":"paragraph","attrs":{"tags":null}}]}"#;
        assert_eq!(check_doc(empty).unwrap().chars, 0);
    }

    #[test]
    fn anything_outside_the_schema_is_refused() {
        let doc = |blocks: &str| format!(r#"{{"type":"doc","content":[{blocks}]}}"#);
        let para = |inline: &str| doc(&format!(r#"{{"type":"paragraph","content":[{inline}]}}"#));
        let bad = [
            "".to_string(),
            "null".into(),
            "[]".into(),
            r#"{"type":"doc"}"#.into(),
            r#"{"type":"doc","content":[]}"#.into(),
            r#"{"type":"paragraph","content":[]}"#.into(),
            r#"{"type":"doc","content":[],"html":"<b>"}"#.into(),
            // nodes and marks outside wave 2 (Q19): headings, quotes, links, images, code
            doc(r#"{"type":"heading","attrs":{"level":1}}"#),
            doc(r#"{"type":"blockquote","content":[{"type":"paragraph"}]}"#),
            doc(r#"{"type":"code_block"}"#),
            para(r#"{"type":"image","attrs":{"src":"x","onerror":"alert(1)"}}"#),
            para(
                r#"{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}"#,
            ),
            para(r#"{"type":"text","text":"x","marks":[{"type":"strong","attrs":{"style":"x"}}]}"#),
            para(
                r#"{"type":"text","text":"x","marks":[{"type":"strong"},{"type":"em"},{"type":"strong"}]}"#,
            ),
            para(r#"{"type":"text","text":""}"#),
            para(r#"{"type":"text","text":"x","html":"<img src=x onerror=alert(1)>"}"#),
            para(r#"{"type":"hard_break","content":[{"type":"text","text":"x"}]}"#),
            para(r#"{"type":"paragraph"}"#),
            doc(r#"{"type":"text","text":"bare text at the top"}"#),
            doc(r#"{"type":"paragraph","attrs":{"tags":null,"class":"x"}}"#),
            doc(r#"{"type":"paragraph","attrs":{"onclick":"x"}}"#),
            doc(r#"{"type":"bullet_list","content":[]}"#),
            doc(r#"{"type":"bullet_list","content":[{"type":"paragraph"}]}"#),
            doc(r#"{"type":"bullet_list","content":[{"type":"list_item","content":[]}]}"#),
            doc(
                r#"{"type":"bullet_list","content":[{"type":"list_item","content":[{"type":"bullet_list","content":[{"type":"list_item","content":[{"type":"paragraph"}]}]}]}]}"#,
            ),
            doc(
                r#"{"type":"ordered_list","attrs":{"order":-1},"content":[{"type":"list_item","content":[{"type":"paragraph"}]}]}"#,
            ),
            doc(
                r#"{"type":"ordered_list","attrs":{"order":"1"},"content":[{"type":"list_item","content":[{"type":"paragraph"}]}]}"#,
            ),
            doc(r#"{"type":"list_item","content":[{"type":"paragraph"}]}"#),
        ];
        for b in &bad {
            assert!(check_doc(b).is_err(), "accepted: {b}");
        }
    }

    #[test]
    fn tags_are_checked() {
        let with = |tags: &str| {
            format!(
                r#"{{"type":"doc","content":[{{"type":"paragraph","attrs":{{"tags":{tags}}}}}]}}"#
            )
        };
        for ok in [
            "null",
            "[]",
            r#"[{"book":"John","chapter":3,"v1":16,"v2":16}]"#,
            r#"[{"book":"1John","chapter":1,"v1":1,"v2":999},{"book":"Ps","chapter":23,"v1":0,"v2":0},{"book":"Gen","chapter":1,"v1":1,"v2":1}]"#,
        ] {
            assert!(check_doc(&with(ok)).is_ok(), "{ok}");
        }
        for bad in [
            r#""John 3:16""#,
            r#"{"book":"John","chapter":3,"v1":16,"v2":16}"#,
            r#"[{"book":"John","chapter":3,"v1":16,"v2":16},{"book":"John","chapter":4,"v1":1,"v2":1},{"book":"John","chapter":5,"v1":1,"v2":1},{"book":"John","chapter":6,"v1":1,"v2":1}]"#,
            r#"[{"book":"<b>","chapter":3,"v1":16,"v2":16}]"#,
            r#"[{"book":"","chapter":3,"v1":16,"v2":16}]"#,
            r#"[{"book":"John","chapter":0,"v1":16,"v2":16}]"#,
            r#"[{"book":"John","chapter":1000,"v1":16,"v2":16}]"#,
            r#"[{"book":"John","chapter":3,"v1":18,"v2":16}]"#,
            r#"[{"book":"John","chapter":3,"v1":0,"v2":16}]"#,
            r#"[{"book":"John","chapter":3,"v1":16,"v2":1000}]"#,
            r#"[{"book":"John","chapter":3.5,"v1":16,"v2":16}]"#,
            r#"[{"book":"John","chapter":3,"v1":16,"v2":16,"text":"For God so loved"}]"#,
            r#"[{"book":"John","chapter":3,"v1":16}]"#,
        ] {
            assert!(check_doc(&with(bad)).is_err(), "accepted: {bad}");
        }
    }

    #[test]
    fn size_and_shape_limits_are_enforced() {
        // bytes
        let long = "a".repeat(MAX_NOTE_BYTES);
        let big = format!(
            r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{long}"}}]}}]}}"#
        );
        let e = check_doc(&big).unwrap_err();
        assert!(e.contains("too long"), "{e}");
        let fits = "a".repeat(MAX_NOTE_BYTES - 200);
        let ok = format!(
            r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{fits}"}}]}}]}}"#
        );
        assert_eq!(check_doc(&ok).unwrap().chars, fits.len());
        // depth: lists nested past the limit
        let mut d = r#"{"type":"paragraph"}"#.to_string();
        for _ in 0..20 {
            d = format!(
                r#"{{"type":"bullet_list","content":[{{"type":"list_item","content":[{{"type":"paragraph"}},{d}]}}]}}"#
            );
        }
        let deep = format!(r#"{{"type":"doc","content":[{d}]}}"#);
        assert!(check_doc(&deep).unwrap_err().contains("deeply"));
        // JSON nesting beyond serde's own limit is an error, not a crash
        let brackets = format!("{}{}", "[".repeat(10_000), "]".repeat(10_000));
        assert!(check_doc(&brackets).is_err());
        // node count
        let br = vec![r#"{"type":"hard_break"}"#; MAX_NODES + 1].join(",");
        let many =
            format!(r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{br}]}}]}}"#);
        assert!(check_doc(&many).unwrap_err().contains("too many"));
    }
}
