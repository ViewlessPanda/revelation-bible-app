//! Pre-scan (DESIGN.md 9.2) and the strict event loop over quick-xml 0.42.0
//! (9.3, 9.4): an own element stack, caps (9.1) and positioned errors.
//!
//! quick-xml is used only as a tokenizer: `Reader` over the in-memory buffer,
//! `read_event()`, full qualified names compared as bytes, raw text and raw
//! attribute values. None of its unescaping, decoding, trimming or
//! line-end-normalising helpers is called (9.3, "forbidden" column).

use std::borrow::Cow;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::error::{Error, excerpt};

/// Which event forms an element may take (9.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `<x …/>` only.
    EmptyOnly,
    /// `<x …/>` or `<x …>…</x>`.
    Either,
    /// `<x …>…</x>` only; an `Empty` event fails.
    ContainerOnly,
}

/// The XML half of a source profile.
#[derive(Debug, Clone)]
pub struct XmlProfile {
    /// File name used in error messages.
    pub file: &'static str,
    /// Exact bytes the file must start with (the XML declaration). The parser
    /// sees only the bytes after it.
    pub prefix: &'static [u8],
    /// CR bytes allowed (OSHB only).
    pub allow_cr: bool,
    /// Attribute values must be double-quoted (KJV, OSHB).
    pub double_quoted: bool,
    /// Every element the profile knows, with its allowed forms.
    pub forms: &'static [(&'static str, Form)],
    /// MGNT only: an internal DTD subset that is **skipped, never
    /// interpreted** (DESIGN.md 9.2): pinned position, first bytes and SHA-256.
    pub subset: Option<DtdSubset>,
    /// Comments the profile allows, as (exact comment bytes, the `type` of
    /// the `<title>` that must follow it directly). Any other comment, and an
    /// allowed one not directly followed by that title, fails (DESIGN.md
    /// 17.3). Empty for every profile except the commentary's.
    pub comments: &'static [(&'static str, &'static str)],
    /// `&amp;` allowed in text (and only there), passed on as a text token
    /// `"&"`; any other `&` still fails (DESIGN.md 9.2, 17.3).
    pub amp: bool,
}

/// A pinned internal DTD subset (DESIGN.md 9.2, M1-F15). The bytes between
/// the declaration and `start` must equal `between`; `data[start..end]` must
/// start with `head` and hash to `sha256`. The parser sees only bytes from
/// `end` on, and any other `<!` anywhere from `end` on fails.
#[derive(Debug, Clone, Copy)]
pub struct DtdSubset {
    pub between: &'static [u8],
    pub start: usize,
    pub end: usize,
    pub head: &'static [u8],
    pub sha256: &'static str,
}

impl XmlProfile {
    /// The file offset where the parser input starts: after the declaration,
    /// or after the pinned DTD subset.
    pub fn body_start(&self) -> usize {
        self.subset.map_or(self.prefix.len(), |s| s.end)
    }
}

/// Caps (DESIGN.md 9.1).
pub const MAX_DEPTH: usize = 16;
pub const MAX_ATTRS: usize = 16;
pub const MAX_ATTR_VALUE: usize = 4096;
pub const MAX_NAME: usize = 64;
pub const MAX_TEXT: usize = 64 * 1024;
pub const MAX_TOKENS: u64 = 4_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Start,
    End,
    Empty,
    Text,
}

/// One event, with file offsets `[start, end)`.
#[derive(Debug)]
pub struct Tok<'a> {
    pub kind: Kind,
    /// Full qualified element name; empty for text.
    pub name: &'a str,
    /// Raw attribute (name, value) pairs, document order. Empty for End/Text.
    pub attrs: Vec<(&'a str, &'a str)>,
    /// Raw text bytes (Text events only). `"&"` for an allowed `&amp;`.
    pub text: &'a str,
    /// On a `Start` event only: the allowed comment directly before it
    /// (`XmlProfile::comments`), else `None`.
    pub comment: Option<&'static str>,
    pub start: u64,
    pub end: u64,
}

impl Tok<'_> {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    }
}

/// Totals from one scan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub tokens: u64,
    pub max_depth: usize,
}

/// A reader over `body` with every configuration field this pipeline relies
/// on set explicitly (9.3). T15 reads them back.
pub fn new_reader(body: &str) -> Reader<&[u8]> {
    let mut r = Reader::from_str(body);
    let c = r.config_mut();
    c.trim_text_start = false;
    c.trim_text_end = false;
    c.expand_empty_elements = false;
    c.check_end_names = true;
    c.allow_unmatched_ends = false;
    c.allow_dangling_amp = false;
    c.check_comments = true;
    c.trim_markup_names_in_closing_tags = false;
    r
}

fn err(p: &XmlProfile, check: &'static str, at: u64, msg: impl Into<String>) -> Error {
    Error::new(check, msg).file(p.file).at(at)
}

/// The byte-level pre-scan. Returns the parser input (bytes after the prefix).
pub fn prescan<'a>(data: &'a [u8], p: &XmlProfile) -> Result<&'a str, Error> {
    let text = std::str::from_utf8(data)
        .map_err(|e| err(p, "utf8", e.valid_up_to() as u64, "invalid UTF-8"))?;
    if !data.starts_with(p.prefix) {
        return Err(err(
            p,
            "decl",
            0,
            "file does not start with the pinned XML declaration",
        ));
    }
    for (i, &b) in data.iter().enumerate() {
        let bad = match b {
            b'\t' | b'\n' => false,
            b'\r' => !p.allow_cr,
            0..=0x1f => true,
            b'&' if p.amp && data[i..].starts_with(b"&amp;") => false,
            b'&' => {
                return Err(err(
                    p,
                    "entity",
                    i as u64,
                    "'&' (entity or character reference) in an XML input",
                ));
            }
            _ => false,
        };
        if bad {
            return Err(err(p, "bytes", i as u64, format!("control byte 0x{b:02x}")));
        }
    }
    for pat in [&b"\xEF\xBF\xBE"[..], b"\xEF\xBF\xBF"] {
        if let Some(i) = find(data, pat) {
            return Err(err(p, "bytes", i as u64, "U+FFFE or U+FFFF"));
        }
    }
    let body = p.body_start();
    if let Some(sub) = &p.subset {
        let at = p.prefix.len();
        let fits = sub.start == at + sub.between.len() && data.len() >= sub.end;
        if !fits || &data[at..sub.start] != sub.between {
            return Err(err(
                p,
                "dtd",
                at as u64,
                "the DTD subset is not where the profile pins it",
            ));
        }
        let subset = &data[sub.start..sub.end];
        if !subset.starts_with(sub.head) || crate::sources::sha256_hex(subset) != sub.sha256 {
            return Err(err(
                p,
                "dtd",
                sub.start as u64,
                "the DTD subset differs from the pinned one (a new release needs a review)",
            ));
        }
    }
    let mut from = body;
    while let Some(i) = find(&data[from..], b"<!") {
        let at = from + i;
        match p
            .comments
            .iter()
            .find(|(c, _)| data[at..].starts_with(c.as_bytes()))
        {
            Some((c, _)) => from = at + c.len(),
            None => {
                return Err(err(
                    p,
                    "dtd",
                    at as u64,
                    "'<!' (DOCTYPE, ENTITY, comment or CDATA)",
                ));
            }
        }
    }
    if let Some(i) = find(&data[body..], b"<?") {
        return Err(err(
            p,
            "decl",
            (i + body) as u64,
            "a second XML declaration or a processing instruction",
        ));
    }
    Ok(&text[body..])
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Every attribute value in the raw attribute text is double-quoted.
fn all_double_quoted(raw: &str) -> bool {
    let mut quote: Option<char> = None;
    for c in raw.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' => quote = Some('"'),
            None if c == '\'' => return false,
            None => {}
        }
    }
    true
}

/// Pre-scan, then the strict event loop. `f` sees every event in document
/// order with the names of its open ancestors (for `Start`/`Empty`, the
/// element itself is not included; for `End`, it has already been popped).
pub fn scan<'a, F>(data: &'a [u8], p: &XmlProfile, mut f: F) -> Result<Stats, Error>
where
    F: FnMut(&Tok<'a>, &[&'a str]) -> Result<(), Error>,
{
    let body = prescan(data, p)?;
    let base = p.body_start() as u64;
    let mut reader = new_reader(body);
    let mut stack: Vec<&'a str> = Vec::new();
    let mut root_done = false;
    let mut root_seen = false;
    let mut stats = Stats::default();
    // An allowed comment just read: the next event must be its title.
    let mut pending: Option<(&'static str, &'static str, u64)> = None;
    loop {
        let before = reader.buffer_position();
        let at = base + before;
        let ev = reader.read_event().map_err(|e| {
            err(
                p,
                "xml",
                base + reader.error_position(),
                format!("not well-formed: {}", excerpt(e.to_string().as_bytes())),
            )
        })?;
        let after = base + reader.buffer_position();
        stats.tokens += 1;
        if stats.tokens > MAX_TOKENS {
            return Err(err(p, "cap", at, "more than 4,000,000 tokens"));
        }
        let comment = pending.take();
        let comment_err = |want: &str, cat: u64| {
            err(
                p,
                "comment",
                cat,
                format!("comment not directly followed by <title type=\"{want}\">"),
            )
        };
        // An Empty event after a comment is never its title; it is refused
        // below, after its form check, so `<title />` reports the form (M1-F3).
        if let Some((_, want, cat)) = comment
            && !matches!(ev, Event::Empty(_))
        {
            let ok = match &ev {
                Event::Start(e) => {
                    e.name().as_ref() == "title"
                        && e.attributes_raw() == format!(" type=\"{want}\"")
                }
                _ => false,
            };
            if !ok {
                return Err(comment_err(want, cat));
            }
        }
        match ev {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let empty = matches!(ev, Event::Empty(_));
                let qlen = e.name().as_ref().len();
                if qlen == 0 || qlen > MAX_NAME {
                    return Err(err(
                        p,
                        "cap",
                        at,
                        "element name empty or longer than 64 bytes",
                    ));
                }
                // The name, borrowed from the input itself (it follows '<').
                let start = before as usize + 1;
                let name: &'a str = body
                    .get(start..start + qlen)
                    .filter(|n| *n == e.name().as_ref())
                    .ok_or_else(|| err(p, "xml", at, "element name not where expected"))?;
                if root_done {
                    return Err(err(p, "xml", at, "a second root element"));
                }
                let form = p
                    .forms
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, f)| *f)
                    .ok_or_else(|| {
                        err(
                            p,
                            "form",
                            at,
                            format!(
                                "element <{}> is not in the profile",
                                excerpt(name.as_bytes())
                            ),
                        )
                    })?;
                let ok = match form {
                    Form::EmptyOnly => empty,
                    Form::Either => true,
                    Form::ContainerOnly => !empty,
                };
                if !ok {
                    let as_ = if empty {
                        "an empty element"
                    } else {
                        "a container"
                    };
                    return Err(err(
                        p,
                        "form",
                        at,
                        format!("<{name}> is not allowed as {as_}"),
                    ));
                }
                if let (true, Some((_, want, cat))) = (empty, comment) {
                    return Err(comment_err(want, cat));
                }
                // Attributes: counted by us, capped at 16 before any duplicate check.
                if p.double_quoted && !all_double_quoted(e.attributes_raw()) {
                    return Err(err(p, "attr", at, "attribute value not double-quoted"));
                }
                let mut attrs: Vec<(&'a str, &'a str)> = Vec::new();
                let mut it = e.attributes();
                it.with_checks(false);
                for a in it {
                    if attrs.len() == MAX_ATTRS {
                        return Err(err(p, "cap", at, "more than 16 attributes"));
                    }
                    let a = a.map_err(|_| err(p, "xml", at, "malformed attribute"))?;
                    let key = a.key.0;
                    let value = match a.value {
                        Cow::Borrowed(v) => v,
                        Cow::Owned(_) => return Err(err(p, "attr", at, "attribute value not raw")),
                    };
                    if key.is_empty() || key.len() > MAX_NAME {
                        return Err(err(
                            p,
                            "cap",
                            at,
                            "attribute name empty or longer than 64 bytes",
                        ));
                    }
                    if value.len() > MAX_ATTR_VALUE {
                        return Err(err(p, "cap", at, "attribute value longer than 4 KiB"));
                    }
                    if value.contains('<') {
                        return Err(err(p, "attr", at, "'<' in an attribute value"));
                    }
                    if value.contains('&') {
                        return Err(err(p, "entity", at, "'&' in an attribute value"));
                    }
                    // SAFETY of lifetimes: re-borrow from the input by offset so
                    // the pair lives as long as the buffer, not the event.
                    let key = reborrow(body, key)
                        .ok_or_else(|| err(p, "xml", at, "attribute name not in input"))?;
                    let value = reborrow(body, value)
                        .ok_or_else(|| err(p, "xml", at, "attribute value not in input"))?;
                    if attrs.iter().any(|(k, _)| *k == key) {
                        return Err(err(
                            p,
                            "attr",
                            at,
                            format!("duplicate attribute {}", excerpt(key.as_bytes())),
                        ));
                    }
                    attrs.push((key, value));
                }
                let tok = Tok {
                    kind: if empty { Kind::Empty } else { Kind::Start },
                    name,
                    attrs,
                    text: "",
                    comment: comment.map(|(c, _, _)| c),
                    start: at,
                    end: after,
                };
                f(&tok, &stack)?;
                root_seen = true;
                if empty {
                    if stack.is_empty() {
                        root_done = true;
                    }
                } else {
                    if stack.len() == MAX_DEPTH {
                        return Err(err(p, "cap", at, "element nesting deeper than 16"));
                    }
                    stack.push(name);
                    stats.max_depth = stats.max_depth.max(stack.len());
                }
            }
            Event::End(ref e) => {
                let name = e.name();
                let top = stack
                    .pop()
                    .ok_or_else(|| err(p, "xml", at, "end tag with no open element"))?;
                if name.as_ref() != top {
                    return Err(err(p, "xml", at, format!("end tag does not match <{top}>")));
                }
                let tok = Tok {
                    kind: Kind::End,
                    name: top,
                    attrs: Vec::new(),
                    text: "",
                    comment: None,
                    start: at,
                    end: after,
                };
                f(&tok, &stack)?;
                if stack.is_empty() {
                    root_done = true;
                }
            }
            Event::Text(t) => {
                let raw: &'a str = match t.into_inner() {
                    Cow::Borrowed(s) => {
                        reborrow(body, s).ok_or_else(|| err(p, "xml", at, "text not in input"))?
                    }
                    Cow::Owned(_) => return Err(err(p, "xml", at, "text not raw")),
                };
                if raw.len() > MAX_TEXT {
                    return Err(err(p, "cap", at, "text node larger than 64 KiB"));
                }
                if let Some(i) = raw.find("]]>") {
                    return Err(err(p, "xml", at + i as u64, "']]>' in text"));
                }
                let ws = |b: u8| matches!(b, b' ' | b'\n' | b'\t') || (p.allow_cr && b == b'\r');
                if stack.is_empty() && !raw.bytes().all(ws) {
                    let where_ = if root_seen { "after" } else { "before" };
                    return Err(err(p, "xml", at, format!("text {where_} the root element")));
                }
                let tok = Tok {
                    kind: Kind::Text,
                    name: "",
                    attrs: Vec::new(),
                    text: raw,
                    comment: None,
                    start: at,
                    end: after,
                };
                f(&tok, &stack)?;
            }
            Event::Decl(_) | Event::DocType(_) => {
                return Err(err(p, "decl", at, "a second declaration or a DOCTYPE"));
            }
            Event::CData(_) => return Err(err(p, "xml", at, "CDATA section")),
            Event::Comment(_) => {
                let raw = body
                    .get(before as usize..(after - base) as usize)
                    .unwrap_or("");
                match p.comments.iter().find(|(c, _)| *c == raw) {
                    Some(&(c, want)) if !stack.is_empty() => pending = Some((c, want, at)),
                    _ => return Err(err(p, "comment", at, "comment")),
                }
            }
            Event::PI(_) => return Err(err(p, "xml", at, "processing instruction")),
            Event::GeneralRef(ref r) if p.amp && r.as_ref() == "amp" && !stack.is_empty() => {
                let tok = Tok {
                    kind: Kind::Text,
                    name: "",
                    attrs: Vec::new(),
                    text: "&",
                    comment: None,
                    start: at,
                    end: after,
                };
                f(&tok, &stack)?;
            }
            Event::GeneralRef(_) => {
                return Err(err(p, "entity", at, "entity or character reference"));
            }
            Event::Eof => {
                if !root_done || !stack.is_empty() {
                    return Err(err(
                        p,
                        "xml",
                        at,
                        "end of file before the root element closed",
                    ));
                }
                return Ok(stats);
            }
        }
    }
}

/// The same bytes as `part`, but borrowed from `whole` (by address), so the
/// result lives as long as the input buffer.
fn reborrow<'a>(whole: &'a str, part: &str) -> Option<&'a str> {
    let w = whole.as_ptr() as usize;
    let p = part.as_ptr() as usize;
    if p < w || p + part.len() > w + whole.len() {
        return None;
    }
    let off = p - w;
    whole.get(off..off + part.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_scan() {
        assert!(all_double_quoted(r#" a="x" b="it's""#));
        assert!(!all_double_quoted(" a='x'"));
        assert!(all_double_quoted(""));
    }

    #[test]
    fn reborrow_bounds() {
        let s = String::from("hello world");
        assert_eq!(reborrow(&s, &s[6..]), Some("world"));
        assert_eq!(reborrow(&s, "elsewhere"), None);
    }
}

#[cfg(test)]
mod commentary_profile_tests {
    //! The two allowances of the commentary profile (DESIGN.md 17.3):
    //! `&amp;` in text and the pinned comments, each fail-closed otherwise.
    use super::*;

    const T: XmlProfile = XmlProfile {
        file: "t.xml",
        prefix: b"<?xml version=\"1.0\"?>",
        allow_cr: false,
        double_quoted: true,
        forms: &[
            ("r", Form::ContainerOnly),
            ("title", Form::ContainerOnly),
            ("m", Form::EmptyOnly),
        ],
        subset: None,
        comments: &[("<!-- s2 -->", "x-s2")],
        amp: true,
    };

    fn run(p: &XmlProfile, body: &str) -> Result<Vec<String>, Error> {
        let src = format!("<?xml version=\"1.0\"?>{body}");
        let mut out = Vec::new();
        scan(src.as_bytes(), p, |t, _| {
            out.push(match t.kind {
                Kind::Text => format!("text:{}", t.text),
                Kind::Start => format!("start:{}:{:?}", t.name, t.comment),
                Kind::End => format!("end:{}", t.name),
                Kind::Empty => format!("empty:{}", t.name),
            });
            Ok(())
        })?;
        Ok(out)
    }

    #[test]
    fn amp_is_its_own_text_token_with_the_decoded_character() {
        let got = run(&T, "<r>a &amp;c. b</r>").unwrap();
        assert_eq!(
            got,
            ["start:r:None", "text:a ", "text:&", "text:c. b", "end:r"]
        );
    }

    #[test]
    fn every_other_reference_fails_and_amp_fails_without_the_flag() {
        for body in [
            "<r>&lt;</r>",
            "<r>&#65;</r>",
            "<r>&#x202E;</r>",
            "<r>&amp</r>",
            "<r>& </r>",
            "<r a=\"&amp;\"></r>",
        ] {
            let e = run(&T, body).unwrap_err();
            assert!(["entity", "xml"].contains(&e.check), "{body}: {e}");
        }
        let off = XmlProfile { amp: false, ..T };
        assert_eq!(run(&off, "<r>&amp;</r>").unwrap_err().check, "entity");
    }

    #[test]
    fn a_pinned_comment_must_be_followed_directly_by_its_title() {
        let got = run(&T, "<r><!-- s2 --><title type=\"x-s2\">C</title></r>").unwrap();
        assert_eq!(got[1], "start:title:Some(\"<!-- s2 -->\")");
        for (body, check) in [
            (
                "<r><!-- s2 -->\n<title type=\"x-s2\">C</title></r>",
                "comment",
            ),
            (
                "<r><!-- s2 --><title type=\"x-IS\">C</title></r>",
                "comment",
            ),
            (
                "<r><!-- s2 --><title type=\"x-s2\" n=\"1\">C</title></r>",
                "comment",
            ),
            ("<r><!-- s3 --><title type=\"x-s2\">C</title></r>", "dtd"),
            ("<r><!--s2--><title type=\"x-s2\">C</title></r>", "dtd"),
            ("<!-- s2 --><r></r>", "comment"),
            // An empty title is refused by its form; any other empty
            // element after the comment by the comment check.
            ("<r><!-- s2 --><title type=\"x-s2\" /></r>", "form"),
            ("<r><!-- s2 --><m /></r>", "comment"),
            ("<r><![CDATA[x]]></r>", "dtd"),
        ] {
            assert_eq!(run(&T, body).unwrap_err().check, check, "{body}");
        }
        let none = XmlProfile { comments: &[], ..T };
        assert_eq!(
            run(&none, "<r><!-- s2 --><title type=\"x-s2\">C</title></r>")
                .unwrap_err()
                .check,
            "dtd"
        );
    }
}
