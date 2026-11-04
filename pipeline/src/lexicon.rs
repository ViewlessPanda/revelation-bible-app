//! Stage 2 and 3 for the two Strong's lexicons (DESIGN.md 7, 9.2, 9.4):
//! HebrewStrong.xml (LEX-HEB-OSHB) and strongsgreek.xml (LEX-GRC-MGNT19) →
//! one entry tree per `<entry>`.
//!
//! Each entry is stored as the source's own XML tree, made safe and typed:
//! element nodes keep their name and allowed attributes (raw values), text
//! nodes keep their bytes exactly, including OSHB's CRLF + TAB whitespace
//! nodes and MGNT's hard line feeds. Nothing is trimmed, normalised or
//! re-worded. Every element, parent, attribute, attribute value spelling and
//! character not in the profile fails closed with a positioned error.

use std::collections::BTreeMap;

use crate::error::{Collector, Error, Result, excerpt};
use crate::strongs::{Lang, StrongsKey};
use crate::xmlscan::{self, DtdSubset, Form, Kind, Tok, XmlProfile};

pub const OSHB_ID: &str = "LEX-HEB-OSHB";
pub const MGNT_ID: &str = "LEX-GRC-MGNT19";

/// Which lexicon format a profile reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexKind {
    Oshb,
    Mgnt,
}

/// One element allowed inside an `<entry>`: its allowed parents ("entry" =
/// directly in the entry), allowed attributes and required attributes.
#[derive(Debug, Clone, Copy)]
pub struct ElementRule {
    pub name: &'static str,
    pub parents: &'static [&'static str],
    pub attrs: &'static [&'static str],
    pub required: &'static [&'static str],
}

/// Pinned facts of one real lexicon file (survey section 9; survey probes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexFacts {
    pub entries: usize,
    /// Elements inside entries, by name.
    pub elements: Vec<(&'static str, usize)>,
    /// Attributes inside entries (`element@name`, or `element@name=value`
    /// for the one enumeration, `strongsref@language`).
    pub attrs: Vec<(&'static str, usize)>,
    /// CRLF pairs in the whole file (OSHB; MGNT has no CR).
    pub crlf: usize,
    /// Entries with non-whitespace text directly inside `<entry>`.
    pub entries_with_direct_text: usize,
    /// MGNT "Not Used" placeholders, by number.
    pub placeholders: Vec<u32>,
}

/// A reviewed spelling exception (like DESIGN.md 6.3): an MGNT
/// `strongsref language="HEBREW"` whose number is not zero-padded. Keyed by
/// entry **and** exact value; each must match exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellingException {
    pub entry: u32,
    pub value: &'static str,
}

#[derive(Debug, Clone)]
pub struct LexProfile {
    pub source_id: &'static str,
    pub kind: LexKind,
    pub xml: XmlProfile,
    pub rules: &'static [ElementRule],
    /// OSHB root `<lexicon>`: exactly these attributes and values (not
    /// imported; inert strings, never resolved).
    pub root_attrs: &'static [(&'static str, &'static str)],
    pub exceptions: Vec<SpellingException>,
    pub facts: Option<LexFacts>,
}

// ------------------------------------------------------------------ model

/// One allowed attribute, raw. `ref_key` is the canonical target of a
/// cross-reference (`w@src`, `strongsref@strongs`), else `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexAttr {
    pub name: String,
    pub value: String,
    pub ref_key: Option<StrongsKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Element { name: String, attrs: Vec<LexAttr> },
    Text(String),
}

/// One node inside an entry, in document order. `parent` is the index of
/// the parent element node, `None` for a direct child of `<entry>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexNode {
    pub parent: Option<u32>,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexEntry {
    pub key: StrongsKey,
    /// 1-based document order in its file.
    pub ord: u32,
    /// The entry's own attribute, raw (`id="H1"` / `strongs="00001"`). Not
    /// stored: under the pinned spelling it is a function of the key.
    pub id_attr: (String, String),
    pub nodes: Vec<LexNode>,
    /// File offset of `<entry`.
    pub offset: u64,
    /// MGNT "Not Used" placeholder (only `<strongs>` plus the text "Not Used").
    pub placeholder: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexModel {
    pub source_id: &'static str,
    pub kind: LexKind,
    pub entries: Vec<LexEntry>,
}

impl LexEntry {
    /// The entry attribute rebuilt from the key under the pinned spelling.
    pub fn spelled_id_attr(&self, kind: LexKind) -> (String, String) {
        match kind {
            LexKind::Oshb => ("id".into(), self.key.to_string()),
            LexKind::Mgnt => ("strongs".into(), format!("{:05}", self.key.number())),
        }
    }
}

// ------------------------------------------------------------------ profiles

const OSHB_FORMS: &[(&str, Form)] = &[
    ("lexicon", Form::ContainerOnly),
    ("entry", Form::ContainerOnly),
    ("w", Form::ContainerOnly),
    ("source", Form::Either),
    ("meaning", Form::ContainerOnly),
    ("usage", Form::ContainerOnly),
    ("def", Form::ContainerOnly),
    ("note", Form::ContainerOnly),
];

const OSHB_RULES: &[ElementRule] = &[
    ElementRule {
        name: "w",
        parents: &["entry", "source", "meaning", "usage"],
        attrs: &["pos", "pron", "xlit", "xml:lang", "src"],
        required: &[],
    },
    ElementRule {
        name: "source",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "meaning",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "usage",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "def",
        parents: &["source", "meaning"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "note",
        parents: &["entry", "source", "meaning"],
        attrs: &[],
        required: &[],
    },
];

const MGNT_FORMS: &[(&str, Form)] = &[
    ("strongsdictionary", Form::ContainerOnly),
    ("prologue", Form::ContainerOnly),
    ("entries", Form::ContainerOnly),
    ("entry", Form::ContainerOnly),
    ("strongs", Form::ContainerOnly),
    ("greek", Form::EmptyOnly),
    ("pronunciation", Form::EmptyOnly),
    ("strongs_derivation", Form::ContainerOnly),
    ("strongs_def", Form::ContainerOnly),
    ("kjv_def", Form::ContainerOnly),
    ("latin", Form::ContainerOnly),
    ("strongsref", Form::EmptyOnly),
];

const MGNT_RULES: &[ElementRule] = &[
    ElementRule {
        name: "strongs",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "greek",
        parents: &["entry", "strongs_derivation", "strongs_def"],
        attrs: &["BETA", "unicode", "translit"],
        required: &["BETA", "unicode", "translit"],
    },
    ElementRule {
        name: "pronunciation",
        parents: &["entry", "strongs_derivation", "strongs_def"],
        attrs: &["strongs"],
        required: &["strongs"],
    },
    ElementRule {
        name: "strongs_derivation",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "strongs_def",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "kjv_def",
        parents: &["entry"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "latin",
        parents: &["strongs_derivation", "strongs_def"],
        attrs: &[],
        required: &[],
    },
    ElementRule {
        name: "strongsref",
        parents: &["entry", "strongs_derivation", "strongs_def", "kjv_def"],
        attrs: &["language", "strongs"],
        required: &["language", "strongs"],
    },
];

/// OSHB XML profile (DESIGN.md 9.2: 38-byte declaration, CR allowed).
pub const OSHB_XML: XmlProfile = XmlProfile {
    file: "HebrewStrong.xml",
    prefix: b"<?xml version=\"1.0\" encoding=\"utf-8\"?>",
    allow_cr: true,
    double_quoted: true,
    forms: OSHB_FORMS,
    subset: None,
    comments: &[],
    amp: false,
};

/// MGNT XML profile: the 55-byte declaration, one LF, then the pinned
/// 1,034-byte DTD subset (bytes 56..1,090), skipped and never interpreted.
/// The parser input starts at byte 1,090 (DESIGN.md 9.2).
pub const MGNT_XML: XmlProfile = XmlProfile {
    file: "strongsgreek.xml",
    prefix: b"<?xml version='1.0' encoding='utf-8' standalone='yes'?>",
    allow_cr: false,
    // No single-quoted attribute value exists after the declaration (survey
    // probe 2026-10-22), so MGNT is held to double quotes as well.
    double_quoted: true,
    forms: MGNT_FORMS,
    subset: Some(DtdSubset {
        between: b"\n",
        start: 56,
        end: 1090,
        head: b"<!DOCTYPE strongsdictionary [",
        sha256: "9b89a2867fda6f8bfcda951f9bad1ccfc5b06416eb6d38f84f8177078e9441f6",
    }),
    comments: &[],
    amp: false,
};

/// The OSHB profile (LEX-HEB-OSHB).
pub fn oshb() -> LexProfile {
    LexProfile {
        source_id: OSHB_ID,
        kind: LexKind::Oshb,
        xml: OSHB_XML,
        rules: OSHB_RULES,
        root_attrs: &[
            (
                "xsi:schemaLocation",
                "http://openscriptures.github.com/morphhb/namespace StrongSchema.xsd",
            ),
            (
                "xmlns",
                "http://openscriptures.github.com/morphhb/namespace",
            ),
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
        ],
        exceptions: Vec::new(),
        facts: Some(LexFacts {
            entries: 8_674,
            elements: vec![
                ("w", 17_784),
                ("def", 15_469),
                ("usage", 8_673),
                ("source", 8_668),
                ("meaning", 8_435),
                ("note", 167),
            ],
            attrs: vec![
                ("w@pos", 8_674),
                ("w@pron", 9_850),
                ("w@xlit", 9_850),
                ("w@xml:lang", 8_674),
                ("w@src", 7_934),
            ],
            crlf: 51_928,
            entries_with_direct_text: 0,
            placeholders: Vec::new(),
        }),
    }
}

/// The MGNT profile (LEX-GRC-MGNT19).
pub fn mgnt() -> LexProfile {
    let mut placeholders = vec![2717];
    placeholders.extend(3203..=3302);
    LexProfile {
        source_id: MGNT_ID,
        kind: LexKind::Mgnt,
        xml: MGNT_XML,
        rules: MGNT_RULES,
        root_attrs: &[],
        // Four HEBREW strongsref values are not zero-padded like the other
        // 313 (probe 2026-10-22; the survey's shape table counts them under
        // "HEBREW 9999"). Kept as the source has them; listed so nothing
        // else can be unpadded.
        exceptions: vec![
            SpellingException {
                entry: 25,
                value: "5689",
            },
            SpellingException {
                entry: 137,
                value: "5869",
            },
            SpellingException {
                entry: 3160,
                value: "4992",
            },
            SpellingException {
                entry: 4460,
                value: "7343",
            },
        ],
        facts: Some(LexFacts {
            entries: 5_624,
            elements: vec![
                ("strongsref", 7_063),
                ("greek", 6_139),
                ("pronunciation", 5_652),
                ("strongs", 5_624),
                ("kjv_def", 5_522),
                ("strongs_derivation", 5_510),
                ("strongs_def", 5_507),
                ("latin", 12),
            ],
            attrs: vec![
                ("greek@BETA", 6_139),
                ("greek@unicode", 6_139),
                ("greek@translit", 6_139),
                ("pronunciation@strongs", 5_652),
                ("strongsref@language=GREEK", 6_746),
                ("strongsref@language=HEBREW", 317),
                ("strongsref@strongs", 7_063),
            ],
            crlf: 0,
            entries_with_direct_text: 324,
            placeholders,
        }),
    }
}

/// A profile for synthetic fixtures: same rules, no pinned facts and no
/// spelling exceptions (those name real entries).
pub fn fixture(p: LexProfile) -> LexProfile {
    LexProfile {
        facts: None,
        exceptions: Vec::new(),
        ..p
    }
}

// ------------------------------------------------------------------ characters

/// Non-ASCII code points allowed in lexicon text and attribute values: the
/// letter, mark and punctuation blocks the two files use (survey probe
/// 2026-10-22), and none of the C1 controls, NBSP, soft hyphen, bidi or
/// zero-width format characters, line or paragraph separators, BOM,
/// replacement character or private use.
const LEX_RANGES: &[(char, char)] = &[
    ('\u{00C0}', '\u{00FF}'), // Latin-1 letters (with × and ÷)
    ('\u{0100}', '\u{024F}'), // Latin Extended-A, -B
    ('\u{0250}', '\u{02FF}'), // IPA, spacing modifier letters (ʻ ʼ)
    ('\u{0300}', '\u{036F}'), // combining diacritics (incl. U+034F CGJ in pointed Hebrew)
    ('\u{0370}', '\u{03FF}'), // Greek and Coptic
    ('\u{0591}', '\u{05C7}'), // Hebrew accents, points, maqaf, sof pasuq
    ('\u{05D0}', '\u{05EA}'), // Hebrew letters
    ('\u{1D00}', '\u{1DBF}'), // phonetic extensions (ᵉ)
    ('\u{1E00}', '\u{1EFF}'), // Latin Extended Additional
    ('\u{1F00}', '\u{1FFF}'), // Greek Extended
    ('\u{2010}', '\u{2010}'), // hyphen
    ('\u{2014}', '\u{2014}'), // em dash
    ('\u{2019}', '\u{2019}'), // right single quotation mark
    ('\u{2026}', '\u{2026}'), // horizontal ellipsis
];

/// Printable ASCII or one of [`LEX_RANGES`].
pub fn char_allowed(c: char) -> bool {
    (' '..='~').contains(&c) || LEX_RANGES.iter().any(|&(a, b)| (a..=b).contains(&c))
}

fn is_ws_only(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
}

// ------------------------------------------------------------------ parser

struct OpenEntry {
    e: LexEntry,
    /// Open element nodes (indices into e.nodes).
    stack: Vec<u32>,
    /// Ancestor count of the `<entry>` start tag.
    depth: usize,
}

struct State<'p> {
    p: &'p LexProfile,
    m: LexModel,
    entry: Option<OpenEntry>,
    root_seen: bool,
    /// MGNT: `prologue` then `entries`, each once.
    mgnt_parts: Vec<&'static str>,
    exception_hits: Vec<usize>,
}

impl State<'_> {
    fn err(&self, check: &'static str, t: &Tok<'_>, msg: impl Into<String>) -> Error {
        let e = Error::new(check, msg).file(self.p.xml.file).at(t.start);
        match &self.entry {
            Some(oe) => e.verse(format!("entry {}", oe.e.key)),
            None => e,
        }
    }

    fn on(&mut self, t: &Tok<'_>, anc: &[&str]) -> std::result::Result<(), Error> {
        if self.entry.is_some() {
            return self.in_entry(t, anc);
        }
        match t.kind {
            Kind::Text => {
                // Outside entries only whitespace (the MGNT prologue's text is
                // not imported and only passes the byte rules).
                if anc.last() == Some(&"prologue") || is_ws_only(t.text) {
                    if t.text.contains('\r') && self.p.kind != LexKind::Oshb {
                        return Err(self.err("lex-char", t, "CR outside the OSHB profile"));
                    }
                    Ok(())
                } else {
                    Err(self.err("lex-structure", t, "text outside an entry"))
                }
            }
            Kind::End => Ok(()),
            Kind::Start | Kind::Empty => self.outside_start(t, anc),
        }
    }

    fn outside_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> std::result::Result<(), Error> {
        let parent = anc.last().copied();
        match (self.p.kind, t.name, parent) {
            (LexKind::Oshb, "lexicon", None) => {
                let ok = t.attrs.len() == self.p.root_attrs.len()
                    && self.p.root_attrs.iter().all(|(k, v)| t.attr(k) == Some(*v));
                if !ok {
                    return Err(self.err(
                        "lex-attr",
                        t,
                        "<lexicon> attributes differ from the pinned ones",
                    ));
                }
                self.root_seen = true;
                Ok(())
            }
            (LexKind::Mgnt, "strongsdictionary", None) => {
                self.no_attrs(t)?;
                self.root_seen = true;
                Ok(())
            }
            (LexKind::Mgnt, part @ ("prologue" | "entries"), Some("strongsdictionary")) => {
                self.no_attrs(t)?;
                let want = ["prologue", "entries"].get(self.mgnt_parts.len()).copied();
                if want != Some(part) {
                    return Err(self.err(
                        "lex-structure",
                        t,
                        "expected <prologue> then <entries>, once each",
                    ));
                }
                self.mgnt_parts.push(if part == "prologue" {
                    "prologue"
                } else {
                    "entries"
                });
                Ok(())
            }
            (LexKind::Oshb, "entry", Some("lexicon"))
            | (LexKind::Mgnt, "entry", Some("entries")) => self.entry_start(t, anc),
            _ => Err(self.err(
                "lex-structure",
                t,
                format!("<{}> is not allowed here", t.name),
            )),
        }
    }

    fn no_attrs(&self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        if t.attrs.is_empty() {
            Ok(())
        } else {
            Err(self.err("lex-attr", t, format!("<{}> has attributes", t.name)))
        }
    }

    fn entry_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> std::result::Result<(), Error> {
        let (attr, lang) = match self.p.kind {
            LexKind::Oshb => ("id", Lang::Hebrew),
            LexKind::Mgnt => ("strongs", Lang::Greek),
        };
        let value = match t.attrs.as_slice() {
            [(k, v)] if *k == attr => *v,
            _ => {
                return Err(self.err(
                    "lex-attr",
                    t,
                    format!("<entry> must have exactly the attribute {attr}"),
                ));
            }
        };
        let key = match self.p.kind {
            LexKind::Oshb => StrongsKey::from_prefixed(value),
            LexKind::Mgnt => StrongsKey::from_digits(lang, value),
        }
        .map_err(|e| self.err("lex-key", t, e.to_string()))?;
        let spelled = match self.p.kind {
            LexKind::Oshb => key.to_string(),
            LexKind::Mgnt => format!("{:05}", key.number()),
        };
        if key.lang() != lang || value != spelled {
            return Err(self.err(
                "lex-key",
                t,
                format!(
                    "entry key {} is not in the pinned spelling",
                    excerpt(value.as_bytes())
                ),
            ));
        }
        if let Some(prev) = self.m.entries.last()
            && prev.key.number() >= key.number()
        {
            return Err(self.err(
                "lex-key",
                t,
                format!("entry {key} is not after {} (unique, ascending)", prev.key),
            ));
        }
        self.entry = Some(OpenEntry {
            e: LexEntry {
                key,
                ord: (self.m.entries.len() + 1) as u32,
                id_attr: (attr.to_string(), value.to_string()),
                nodes: Vec::new(),
                offset: t.start,
                placeholder: false,
            },
            stack: Vec::new(),
            depth: anc.len(),
        });
        Ok(())
    }

    fn in_entry(&mut self, t: &Tok<'_>, anc: &[&str]) -> std::result::Result<(), Error> {
        let kind = self.p.kind;
        match t.kind {
            Kind::Text => {
                let oe = self.entry.as_ref().unwrap();
                let parent = oe.stack.last().copied();
                let ws = is_ws_only(t.text);
                let direct = parent.is_none();
                if kind == LexKind::Oshb && direct && !ws {
                    return Err(self.err(
                        "lex-structure",
                        t,
                        "non-whitespace text directly in an OSHB entry",
                    ));
                }
                // CR and TAB only in OSHB's whitespace-only nodes between an
                // entry's children (CRLF + TAB indentation, DESIGN.md 7).
                let layout = kind == LexKind::Oshb && direct && ws;
                if !layout && t.text.bytes().any(|b| b == b'\r' || b == b'\t') {
                    return Err(self.err(
                        "lex-char",
                        t,
                        "CR or TAB outside an OSHB whitespace node between entry children",
                    ));
                }
                if let Some(c) = t
                    .text
                    .chars()
                    .find(|&c| c != '\n' && c != '\r' && c != '\t' && !char_allowed(c))
                {
                    return Err(self.err(
                        "lex-char",
                        t,
                        format!("U+{:04X} is not allowed in lexicon text", c as u32),
                    ));
                }
                let oe = self.entry.as_mut().unwrap();
                oe.e.nodes.push(LexNode {
                    parent,
                    kind: NodeKind::Text(t.text.to_string()),
                });
                Ok(())
            }
            Kind::End => {
                let oe = self.entry.as_mut().unwrap();
                if oe.stack.pop().is_none() {
                    // </entry>
                    debug_assert_eq!(anc.len(), oe.depth);
                    let oe = self.entry.take().unwrap();
                    return self.finish_entry(oe.e);
                }
                Ok(())
            }
            Kind::Start | Kind::Empty => {
                let oe = self.entry.as_ref().unwrap();
                let parent_name = match oe.stack.last() {
                    Some(&i) => match &oe.e.nodes[i as usize].kind {
                        NodeKind::Element { name, .. } => name.clone(),
                        NodeKind::Text(_) => unreachable!("only elements are on the stack"),
                    },
                    None => "entry".to_string(),
                };
                let rule = self
                    .p
                    .rules
                    .iter()
                    .find(|r| r.name == t.name)
                    .filter(|r| r.parents.contains(&parent_name.as_str()))
                    .ok_or_else(|| {
                        self.err(
                            "lex-structure",
                            t,
                            format!("<{}> is not allowed in <{parent_name}>", t.name),
                        )
                    })?;
                let mut attrs = Vec::new();
                for (k, v) in &t.attrs {
                    if !rule.attrs.contains(k) {
                        return Err(self.err(
                            "lex-attr",
                            t,
                            format!(
                                "attribute {} is not allowed on <{}>",
                                excerpt(k.as_bytes()),
                                t.name
                            ),
                        ));
                    }
                    if let Some(c) = v.chars().find(|&c| !char_allowed(c)) {
                        return Err(self.err(
                            "lex-char",
                            t,
                            format!("U+{:04X} is not allowed in an attribute value", c as u32),
                        ));
                    }
                    attrs.push(LexAttr {
                        name: k.to_string(),
                        value: v.to_string(),
                        ref_key: None,
                    });
                }
                for r in rule.required {
                    if t.attr(r).is_none() {
                        return Err(self.err(
                            "lex-attr",
                            t,
                            format!("<{}> lacks the attribute {r}", t.name),
                        ));
                    }
                }
                self.cross_ref(t, &mut attrs)?;
                let oe = self.entry.as_mut().unwrap();
                let n = oe.e.nodes.len() as u32;
                oe.e.nodes.push(LexNode {
                    parent: oe.stack.last().copied(),
                    kind: NodeKind::Element {
                        name: t.name.to_string(),
                        attrs,
                    },
                });
                if t.kind == Kind::Start {
                    oe.stack.push(n);
                }
                Ok(())
            }
        }
    }

    /// `w@src` (OSHB, `H` + unpadded) and `strongsref` (MGNT: Greek unpadded,
    /// Hebrew `0` + unpadded or a listed exception): canonical target key.
    fn cross_ref(&mut self, t: &Tok<'_>, attrs: &mut [LexAttr]) -> std::result::Result<(), Error> {
        match (self.p.kind, t.name) {
            (LexKind::Oshb, "w") => {
                if let Some(a) = attrs.iter_mut().find(|a| a.name == "src") {
                    let key = StrongsKey::from_prefixed(&a.value)
                        .map_err(|e| self.err("lex-ref", t, e.to_string()))?;
                    if key.lang() != Lang::Hebrew || a.value != key.to_string() {
                        return Err(self.err(
                            "lex-ref",
                            t,
                            format!(
                                "w@src {} is not in the pinned spelling",
                                excerpt(a.value.as_bytes())
                            ),
                        ));
                    }
                    a.ref_key = Some(key);
                }
            }
            (LexKind::Mgnt, "strongsref") => {
                let lang = t.attr("language").unwrap_or("");
                let digits = t.attr("strongs").unwrap_or("");
                let key = StrongsKey::from_strongsref(lang, digits)
                    .map_err(|e| self.err("lex-ref", t, e.to_string()))?;
                let spelled = match key.lang() {
                    Lang::Greek => digits == key.number().to_string(),
                    Lang::Hebrew => {
                        if digits == format!("0{}", key.number()) {
                            true
                        } else {
                            let entry = self.entry.as_ref().unwrap().e.key.number();
                            match self
                                .p
                                .exceptions
                                .iter()
                                .position(|x| x.entry == entry && x.value == digits)
                            {
                                Some(i) => {
                                    self.exception_hits[i] += 1;
                                    true
                                }
                                None => false,
                            }
                        }
                    }
                };
                if !spelled {
                    return Err(self.err(
                        "lex-ref",
                        t,
                        format!(
                            "strongsref strongs={} is not in the pinned spelling",
                            excerpt(digits.as_bytes())
                        ),
                    ));
                }
                if let Some(a) = attrs.iter_mut().find(|a| a.name == "strongs") {
                    a.ref_key = Some(key);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn finish_entry(&mut self, mut e: LexEntry) -> std::result::Result<(), Error> {
        if self.p.kind == LexKind::Mgnt {
            // Exactly one <strongs>, directly in the entry, text = the number.
            let strongs: Vec<usize> = e
                .nodes
                .iter()
                .enumerate()
                .filter(
                    |(_, n)| matches!(&n.kind, NodeKind::Element { name, .. } if name == "strongs"),
                )
                .map(|(i, _)| i)
                .collect();
            let text = |i: usize| -> String {
                e.nodes
                    .iter()
                    .filter(|n| n.parent == Some(i as u32))
                    .filter_map(|n| match &n.kind {
                        NodeKind::Text(s) => Some(s.as_str()),
                        NodeKind::Element { .. } => None,
                    })
                    .collect()
            };
            let ok = strongs.len() == 1
                && e.nodes[strongs[0]].parent.is_none()
                && text(strongs[0]) == e.key.number().to_string();
            if !ok {
                return Err(Error::new(
                    "lex-structure",
                    format!(
                        "entry {}: <strongs> must occur once and equal the entry number",
                        e.key
                    ),
                )
                .file(self.p.xml.file)
                .at(e.offset));
            }
            let elements = e
                .nodes
                .iter()
                .filter(|n| matches!(n.kind, NodeKind::Element { .. }))
                .count();
            let direct: String = e
                .nodes
                .iter()
                .filter(|n| n.parent.is_none())
                .filter_map(|n| match &n.kind {
                    NodeKind::Text(s) => Some(s.as_str()),
                    NodeKind::Element { .. } => None,
                })
                .collect();
            e.placeholder =
                elements == 1 && direct.trim_matches(|c| c == ' ' || c == '\n') == "Not Used";
        }
        self.m.entries.push(e);
        Ok(())
    }
}

/// Parses a verified lexicon buffer with `p` (stage 2), then checks the
/// per-file rules and pinned facts (stage 3).
pub fn parse(data: &[u8], p: &LexProfile) -> Result<LexModel> {
    let mut st = State {
        p,
        m: LexModel {
            source_id: p.source_id,
            kind: p.kind,
            entries: Vec::new(),
        },
        entry: None,
        root_seen: false,
        mgnt_parts: Vec::new(),
        exception_hits: vec![0; p.exceptions.len()],
    };
    xmlscan::scan(data, &p.xml, |t, anc| st.on(t, anc))?;
    let file = p.xml.file;
    if !st.root_seen || (p.kind == LexKind::Mgnt && st.mgnt_parts.len() != 2) {
        return Err(
            Error::new("lex-structure", "root, prologue or entries missing")
                .file(file)
                .into(),
        );
    }
    for (x, hits) in p.exceptions.iter().zip(&st.exception_hits) {
        if *hits != 1 {
            return Err(Error::new(
                "lex-ref",
                format!(
                    "spelling exception G{} strongs={} matched {hits} times, expected exactly 1 (stale list)",
                    x.entry, x.value
                ),
            )
            .file(file)
            .into());
        }
    }
    let m = st.m;
    line_ends(data, p)?;
    facts(data, &m, p)?;
    Ok(m)
}

/// OSHB: CR only as part of CRLF, and every LF part of CRLF (survey 9.1:
/// 51,928 CRLF, no bare LF or CR). MGNT: CR is already refused by the
/// pre-scan.
fn line_ends(data: &[u8], p: &LexProfile) -> std::result::Result<(), Error> {
    if p.kind != LexKind::Oshb {
        return Ok(());
    }
    for (i, &b) in data.iter().enumerate() {
        let bad = match b {
            b'\r' => data.get(i + 1) != Some(&b'\n'),
            b'\n' => i == 0 || data[i - 1] != b'\r',
            _ => false,
        };
        if bad {
            return Err(
                Error::new("lex-char", "bare CR or LF (the file is CRLF only)")
                    .file(p.xml.file)
                    .at(i as u64),
            );
        }
    }
    Ok(())
}

/// Counts used by the pinned facts (also printed by the CLI).
pub fn counts(m: &LexModel) -> BTreeMap<String, usize> {
    let mut c: BTreeMap<String, usize> = BTreeMap::new();
    for e in &m.entries {
        for n in &e.nodes {
            if let NodeKind::Element { name, attrs } = &n.kind {
                *c.entry(name.clone()).or_default() += 1;
                for a in attrs {
                    *c.entry(format!("{name}@{}", a.name)).or_default() += 1;
                    if name == "strongsref" && a.name == "language" {
                        *c.entry(format!("{name}@language={}", a.value)).or_default() += 1;
                    }
                }
            }
        }
    }
    c
}

fn facts(data: &[u8], m: &LexModel, p: &LexProfile) -> Result<()> {
    let Some(f) = &p.facts else { return Ok(()) };
    let mut col = Collector::default();
    let mut fact = |what: String, got: usize, want: usize| {
        if got != want {
            col.push(
                Error::new(
                    "lex-facts",
                    format!("{what}: source has {got}, pinned fact is {want}"),
                )
                .file(p.xml.file),
            );
        }
    };
    fact("entries".into(), m.entries.len(), f.entries);
    let c = counts(m);
    for (name, want) in f.elements.iter().chain(&f.attrs) {
        fact(
            format!("<{name}>"),
            c.get(*name).copied().unwrap_or(0),
            *want,
        );
    }
    let crlf = data.windows(2).filter(|w| w == b"\r\n").count();
    fact("CRLF".into(), crlf, f.crlf);
    let direct = m
        .entries
        .iter()
        .filter(|e| {
            e.nodes.iter().any(|n| {
                n.parent.is_none() && matches!(&n.kind, NodeKind::Text(s) if !is_ws_only(s))
            })
        })
        .count();
    fact(
        "entries with text directly in <entry>".into(),
        direct,
        f.entries_with_direct_text,
    );
    let ph: Vec<u32> = m
        .entries
        .iter()
        .filter(|e| e.placeholder)
        .map(|e| e.key.number())
        .collect();
    if ph != f.placeholders {
        col.push(
            Error::new(
                "lex-facts",
                format!(
                    "{} \"Not Used\" placeholders, pinned {} (2717, 3203-3302)",
                    ph.len(),
                    f.placeholders.len()
                ),
            )
            .file(p.xml.file),
        );
    }
    col.finish()
}

// ------------------------------------------------------------------ resolution

/// All entries of both lexicons by key.
pub fn index<'a>(lexicons: &[&'a LexModel]) -> BTreeMap<StrongsKey, &'a LexEntry> {
    let mut idx = BTreeMap::new();
    for m in lexicons {
        for e in &m.entries {
            idx.insert(e.key, e);
        }
    }
    idx
}

/// Every lexicon cross-reference resolves to an entry of its language that
/// is not a placeholder (DESIGN.md 6.2, T14).
pub fn check_cross_refs(lexicons: &[&LexModel]) -> Result<()> {
    let idx = index(lexicons);
    let mut c = Collector::default();
    for m in lexicons {
        for e in &m.entries {
            for n in &e.nodes {
                let NodeKind::Element { name, attrs } = &n.kind else {
                    continue;
                };
                for a in attrs {
                    let Some(k) = a.ref_key else { continue };
                    match idx.get(&k) {
                        Some(t) if !t.placeholder => {}
                        Some(_) => c.push(Error::new(
                            "lex-ref",
                            format!("{}: <{name}> refers to placeholder {k}", e.key),
                        )),
                        None => c.push(Error::new(
                            "lex-ref",
                            format!("{}: <{name}> refers to {k}, which no lexicon has", e.key),
                        )),
                    }
                }
            }
        }
    }
    c.finish()
}

/// T13: the KJV keys that don't resolve to a used lexicon entry, with the
/// number of word links using each (empty when all resolve). Never fixed up:
/// a non-empty list stops the build and is reported as is.
pub fn unresolved_keys<'a>(
    keys: impl IntoIterator<Item = &'a StrongsKey>,
    lexicons: &[&LexModel],
) -> Vec<(String, usize)> {
    let idx = index(lexicons);
    let mut bad: BTreeMap<StrongsKey, usize> = BTreeMap::new();
    for k in keys {
        match idx.get(k) {
            Some(e) if !e.placeholder => {}
            _ => *bad.entry(*k).or_default() += 1,
        }
    }
    bad.into_iter().map(|(k, n)| (k.to_string(), n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_policy() {
        for ok in [
            'a', '~', ' ', 'א', '\u{05B0}', 'ἀ', 'Ω', '×', 'ʼ', '\u{034F}', '…',
        ] {
            assert!(char_allowed(ok), "{ok:?}");
        }
        for bad in [
            '\u{202E}', '\u{200B}', '\u{200F}', '\u{00A0}', '\u{00AD}', '\u{FEFF}', '\u{FFFD}',
            '\u{2028}', '\u{0085}', '\u{E000}', '\u{7F}', '\t', '\n',
        ] {
            assert!(!char_allowed(bad), "{bad:?}");
        }
    }
}
