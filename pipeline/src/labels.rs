//! The scholar's label metadata for a commentary database (DESIGN.md 17.7).
//!
//! `pipeline/data/mhcc-labels.yaml` is a byte-for-byte copy of the YAML block
//! in `docs/scholar/commentary-labels.md` §5 (a test compares the two). This
//! module reads that file with a strict reader for the small YAML subset the
//! block uses and turns it into flat `meta` rows. Values are stored exactly as
//! written (the quotes of a quoted scalar removed, nothing else): no
//! re-wording, trimming or completion. A trailing `# comment` is stored too,
//! as `<key>.comment`, because the scholar uses comments to qualify values
//! (the "c. 1838" date is `to_confirm`). Anything outside the subset fails.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::error::{Error, Result, excerpt};

/// The committed copy of the scholar's block.
pub const MHCC_LABELS_FILE: &str = "mhcc-labels.yaml";
/// Where the block is cited from (stored in `meta` as `label_cited_from`).
pub const MHCC_LABELS_CITED_FROM: &str =
    "docs/scholar/commentary-labels.md section 5 (Machine-readable metadata), YAML block";
/// The label file is a few kilobytes.
pub const CAP_LABELS: u64 = 64 * 1024;

/// `pipeline/data/mhcc-labels.yaml` in the repo.
pub fn mhcc_labels_path() -> PathBuf {
    crate::repo_dir()
        .join("pipeline")
        .join("data")
        .join(MHCC_LABELS_FILE)
}

fn bad(line: usize, msg: impl Into<String>) -> Error {
    Error::new("labels", msg)
        .file(MHCC_LABELS_FILE)
        .verse(format!("line {line}"))
}

/// `[a-z_]+`.
fn is_key(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

/// A scalar and its optional trailing comment: `"…"` (no `"` or `\` inside)
/// or a plain `[A-Za-z0-9_-]+`, then nothing or spaces + `# ` + comment.
fn scalar(line: usize, s: &str) -> std::result::Result<(String, Option<String>), Error> {
    let (value, rest) = if let Some(q) = s.strip_prefix('"') {
        let end = q
            .find('"')
            .ok_or_else(|| bad(line, "unterminated quoted value"))?;
        let v = &q[..end];
        if v.contains('\\') {
            return Err(bad(line, "backslash in a quoted value"));
        }
        (v, &q[end + 1..])
    } else {
        let end = s.find(' ').unwrap_or(s.len());
        let v = &s[..end];
        let plain = v
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
        if v.is_empty() || !plain {
            return Err(bad(
                line,
                format!("value {} is not quoted or plain", excerpt(v.as_bytes())),
            ));
        }
        (v, &s[end..])
    };
    if rest.is_empty() {
        return Ok((value.to_string(), None));
    }
    let after = rest.trim_start_matches(' ');
    match after.strip_prefix("# ") {
        Some(c) if after.len() < rest.len() && !c.is_empty() => {
            Ok((value.to_string(), Some(c.to_string())))
        }
        _ => Err(bad(line, "text after a value that is not a '# ' comment")),
    }
}

/// `key: rest` with a `[a-z_]+` key.
fn key_value(line: usize, s: &str) -> std::result::Result<(&str, &str), Error> {
    let (k, rest) = s
        .split_once(':')
        .ok_or_else(|| bad(line, "expected 'key: value'"))?;
    if !is_key(k) {
        return Err(bad(
            line,
            format!("key {} is not [a-z_]+", excerpt(k.as_bytes())),
        ));
    }
    match rest.strip_prefix(' ') {
        Some(v) if !v.is_empty() => Ok((k, v)),
        _ if rest.is_empty() => Ok((k, "")),
        _ => Err(bad(line, "expected one space after ':'")),
    }
}

/// Every Unicode `General_Category` Cf (format), Zl (line separator) and Zp
/// (paragraph separator) character, as of Unicode 16: the bidi marks,
/// embeddings, overrides and isolates (U+200E/F, U+061C, U+202A–E,
/// U+2066–9), the zero-width characters (U+200B–D, U+2060, U+FEFF), the soft
/// hyphen, invisible operators, interlinear annotation, tags and the like.
/// Listed explicitly (std has no category table); the label file is refused
/// if it holds any of them (W2P-A1).
pub fn is_invisible_format(c: char) -> bool {
    matches!(c,
        '\u{00AD}'
        | '\u{0600}'..='\u{0605}'
        | '\u{061C}'
        | '\u{06DD}'
        | '\u{070F}'
        | '\u{0890}'..='\u{0891}'
        | '\u{08E2}'
        | '\u{180E}'
        | '\u{200B}'..='\u{200F}'
        | '\u{2028}'..='\u{202E}'
        | '\u{2060}'..='\u{2064}'
        | '\u{2066}'..='\u{206F}'
        | '\u{FEFF}'
        | '\u{FFF9}'..='\u{FFFB}'
        | '\u{110BD}'
        | '\u{110CD}'
        | '\u{13430}'..='\u{1343F}'
        | '\u{1BCA0}'..='\u{1BCA3}'
        | '\u{1D173}'..='\u{1D17A}'
        | '\u{E0001}'
        | '\u{E0020}'..='\u{E007F}')
}

/// The one value of `label.status` that means the owner approved the labels
/// (W2P-A2; DESIGN.md 17.7). The app shows the author, tradition and date
/// lines only when `label.status` is exactly this.
pub const LABEL_APPROVED: &str = "approved_by_owner";
/// The scholar's value before the owner's approval.
pub const LABEL_PENDING: &str = "recommended_pending_owner_approval";

enum List {
    None,
    Strings,
    Maps,
}

/// Parses the label file into `meta` rows `label.<key>` (lists as
/// `label.<key>.<i>`, list maps as `label.<key>.<i>.<sub>`, i from 1;
/// comments as `<row key>.comment`), sorted by key.
pub fn parse_labels(data: &[u8]) -> Result<Vec<(String, String)>> {
    let text = std::str::from_utf8(data).map_err(|_| bad(0, "not UTF-8"))?;
    if !text.ends_with('\n') {
        return Err(bad(0, "does not end with LF").into());
    }
    // Characters: printable, no CR/TAB/other controls, and no invisible or
    // direction-changing format character (W2P-A1): a reviewer can't see them,
    // and in a label they could reorder or hide the attribution.
    if let Some(c) = text
        .chars()
        .find(|&c| (c.is_control() && c != '\n') || is_invisible_format(c))
    {
        return Err(bad(0, format!("character U+{:04X} is not allowed", c as u32)).into());
    }
    let mut rows = Rows::default();
    // The open list: its key, its kind, the item count, the keys of the
    // current map item.
    let mut list: Option<(String, List, usize, BTreeSet<String>)> = None;
    for (i, raw) in text[..text.len() - 1].split('\n').enumerate() {
        let line = i + 1;
        if let Some(item) = raw.strip_prefix("  - ") {
            let Some((key, kind, n, subs)) = list.as_mut() else {
                return Err(bad(line, "list item outside a list").into());
            };
            *n += 1;
            let is_map = item
                .split_once(':')
                .is_some_and(|(k, _)| is_key(k) && !item.starts_with('"'));
            let want = if is_map { List::Maps } else { List::Strings };
            match (&*kind, &want) {
                (List::None, _) => *kind = want,
                (List::Strings, List::Strings) | (List::Maps, List::Maps) => {}
                _ => return Err(bad(line, "a list mixes strings and maps").into()),
            }
            let base = format!("label.{key}.{n}");
            if is_map {
                subs.clear();
                let (k, v) = key_value(line, item)?;
                subs.insert(k.to_string());
                let (v, c) = scalar(line, v)?;
                rows.push(line, format!("{base}.{k}"), v)?;
                if let Some(c) = c {
                    rows.push(line, format!("{base}.{k}.comment"), c)?;
                }
            } else {
                let (v, c) = scalar(line, item)?;
                rows.push(line, base.clone(), v)?;
                if let Some(c) = c {
                    rows.push(line, format!("{base}.comment"), c)?;
                }
            }
        } else if let Some(sub) = raw.strip_prefix("    ") {
            let Some((key, List::Maps, n, subs)) = list.as_mut() else {
                return Err(bad(line, "indented line outside a list map").into());
            };
            let (k, v) = key_value(line, sub)?;
            if !subs.insert(k.to_string()) {
                return Err(bad(line, format!("duplicate key {k} in a list map")).into());
            }
            let (v, c) = scalar(line, v)?;
            let base = format!("label.{key}.{n}.{k}");
            rows.push(line, base.clone(), v)?;
            if let Some(c) = c {
                rows.push(line, format!("{base}.comment"), c)?;
            }
        } else {
            if let Some((key, _, 0, _)) = &list {
                return Err(bad(line, format!("list {key} has no items")).into());
            }
            list = None;
            let (k, v) = key_value(line, raw)?;
            if v.is_empty() {
                rows.reserve(line, format!("label.{k}"))?;
                list = Some((k.to_string(), List::None, 0, BTreeSet::new()));
                continue;
            }
            let (v, c) = scalar(line, v)?;
            rows.push(line, format!("label.{k}"), v)?;
            if let Some(c) = c {
                rows.push(line, format!("label.{k}.comment"), c)?;
            }
        }
    }
    if let Some((key, _, 0, _)) = &list {
        return Err(bad(0, format!("list {key} has no items")).into());
    }
    // W2P-A2: `status` is one of the two known values and carries no comment
    // (a comment such as "# approved by owner" must not read like approval).
    if rows.seen.contains("label.status.comment") {
        return Err(bad(0, "a comment on status is not allowed").into());
    }
    if let Some((_, v)) = rows.out.iter().find(|(k, _)| k == "label.status")
        && v != LABEL_APPROVED
        && v != LABEL_PENDING
    {
        return Err(bad(
            0,
            format!("status {} is not a known value", excerpt(v.as_bytes())),
        )
        .into());
    }
    rows.out.sort();
    Ok(rows.out)
}

/// The rows read so far and every key used (a list key included).
#[derive(Default)]
struct Rows {
    out: Vec<(String, String)>,
    seen: BTreeSet<String>,
}

impl Rows {
    fn reserve(&mut self, line: usize, key: String) -> std::result::Result<(), Error> {
        if self.seen.insert(key.clone()) {
            Ok(())
        } else {
            Err(bad(line, format!("duplicate key {key}")))
        }
    }

    fn push(&mut self, line: usize, key: String, value: String) -> std::result::Result<(), Error> {
        self.reserve(line, key.clone())?;
        self.out.push((key, value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(rows: &'a [(String, String)], k: &str) -> Option<&'a str> {
        rows.iter().find(|r| r.0 == k).map(|r| r.1.as_str())
    }

    #[test]
    fn subset_reads_scalars_lists_maps_and_comments_exactly() {
        let src = "id: X-1\nname: \"A 'b' \u{b7} c\"   # note \"q\"\nlist:\n  - \"one\"\n  - \"two\"\nauthors:\n  - name: \"N\"\n    role: primary\n  - name: \"M\"\n    role: secondary   # with M\nstatus: approved_by_owner\n";
        let rows = parse_labels(src.as_bytes()).unwrap();
        assert_eq!(get(&rows, "label.id"), Some("X-1"));
        assert_eq!(get(&rows, "label.name"), Some("A 'b' \u{b7} c"));
        assert_eq!(get(&rows, "label.name.comment"), Some("note \"q\""));
        assert_eq!(get(&rows, "label.list.2"), Some("two"));
        assert_eq!(get(&rows, "label.authors.1.role"), Some("primary"));
        assert_eq!(get(&rows, "label.authors.2.name"), Some("M"));
        assert_eq!(get(&rows, "label.authors.2.role.comment"), Some("with M"));
        assert_eq!(get(&rows, "label.status"), Some("approved_by_owner"));
        assert_eq!(rows.len(), 11);
    }

    #[test]
    fn anything_outside_the_subset_fails() {
        for src in [
            "id: X\n\nb: c\n",                     // blank line
            "id: X",                               // no final LF
            "id: X\r\n",                           // CR
            "id:\tX\n",                            // tab
            "id: X\nid: Y\n",                      // duplicate
            "Id: X\n",                             // key case
            "id: 'X'\n",                           // single quotes
            "id: \"X\\\"\"\n",                     // backslash
            "id: \"X\" trailing\n",                // text after a value
            "id: \"X\"#c\n",                       // comment without space
            "id: two words\n",                     // plain with a space
            "id:  X\n",                            // two spaces
            "list:\nnext: X\n",                    // empty list
            "list:\n  - \"a\"\n  - k: \"b\"\n",    // mixed list
            "  - \"a\"\n",                         // item outside a list
            "list:\n  - \"a\"\n    k: \"b\"\n",    // map line under a string list
            "list:\n   - \"a\"\n",                 // three-space indent
            "id: \"a\u{202e}b\"\n",                // bidi control
            "id: \"X\"\nid:\n  - \"a\"\n",         // duplicate as a list
            "a: [1, 2]\n",                         // flow sequence
            "list:\n  - k: \"a\"\n    k: \"b\"\n", // duplicate in a map item
            "status: done\n",                      // unknown status value
            "status: recommended_pending_owner_approval   # approved by owner\n",
        ] {
            let e = parse_labels(src.as_bytes()).unwrap_err();
            assert_eq!(e.first().check, "labels", "{src:?}: {e}");
        }
    }

    /// W2P-A1: every invisible or direction-changing format character is
    /// refused, in a value, a comment, a key line or a list item: fail closed.
    #[test]
    fn invisible_and_bidi_format_characters_are_refused() {
        for c in [
            '\u{200E}',
            '\u{200F}',
            '\u{061C}',
            '\u{200B}',
            '\u{200C}',
            '\u{200D}',
            '\u{2060}',
            '\u{FEFF}',
            '\u{00AD}',
            '\u{2028}',
            '\u{2029}',
            '\u{202A}',
            '\u{202B}',
            '\u{202C}',
            '\u{202D}',
            '\u{202E}',
            '\u{2066}',
            '\u{2067}',
            '\u{2068}',
            '\u{2069}',
            '\u{E0041}',
        ] {
            assert!(is_invisible_format(c), "U+{:04X}", c as u32);
            for src in [
                format!("name: \"Matthew{c} Henry\"\n"),
                format!("name: \"X\"   # note{c}\n"),
                format!("{c}name: \"X\"\n"),
                format!("list:\n  - \"a{c}\"\n"),
            ] {
                let e = parse_labels(src.as_bytes()).unwrap_err();
                assert_eq!(e.first().check, "labels", "U+{:04X}: {e}", c as u32);
                assert!(e.to_string().contains("is not allowed"), "{e}");
            }
        }
        // The non-ASCII characters the scholar's block uses stay allowed.
        for c in ['\u{B7}', '\u{2013}', '\u{2019}', '\u{201C}', '\u{E9}'] {
            assert!(!is_invisible_format(c));
        }
        assert!(parse_labels("name: \"Henry \u{b7} 1662\u{2013}1714\"\n".as_bytes()).is_ok());
    }

    /// W2P-A2: the committed label file is accepted and has a known status.
    #[test]
    fn committed_label_status_is_a_known_value() {
        let rows = parse_labels(&std::fs::read(mhcc_labels_path()).unwrap()).unwrap();
        let s = get(&rows, "label.status").unwrap();
        assert!(s == LABEL_PENDING || s == LABEL_APPROVED, "{s}");
    }
}
