//! Stage 2 and 3 for commentaries (DESIGN.md 17): the CrossWire MHCC 2.0 OSIS
//! file (CMT-MHCC-CW20) → blocks, each stored as the source's own XML tree,
//! and verse anchors, each the KJV verse list exactly as the source writes it.
//!
//! A **block** is a `<p>`, `<title>` or `<table>` inside a book (they never
//! nest). Its element tree is kept byte for byte: element names, attributes
//! in source order, text exactly (the one entity `&amp;` decoded to `&`), so
//! [`serialise_block`] rebuilds the source bytes of every block. An
//! **anchor** is a `<verse sID … />` … `<verse eID … />` pair; its verses are
//! the `osisID` list as written. Nothing is inferred, merged, re-anchored,
//! trimmed or corrected. Everything outside the import scope
//! (condition 5) fails closed with a positioned error.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{Error, Result, excerpt};
use crate::profile::KJV_BOOKS;
use crate::vrs::Table;
use crate::xmlscan::{self, Form, Kind, Tok, XmlProfile};

pub const MHCC_ID: &str = "CMT-MHCC-CW20";
/// Output file: one database per commentary source (DESIGN.md 17.5).
pub const MHCC_DEST: &str = "mhcc.sqlite";

// ------------------------------------------------------------------ profile

/// Event forms (DESIGN.md 9.4): milestones are `Empty` only, everything
/// else is a container only.
const MHCC_FORMS: &[(&str, Form)] = &[
    ("verse", Form::EmptyOnly),
    ("chapter", Form::EmptyOnly),
    ("milestone", Form::EmptyOnly),
    ("osis", Form::ContainerOnly),
    ("osisText", Form::ContainerOnly),
    ("header", Form::ContainerOnly),
    ("revisionDesc", Form::ContainerOnly),
    ("date", Form::ContainerOnly),
    ("work", Form::ContainerOnly),
    ("description", Form::ContainerOnly),
    ("type", Form::ContainerOnly),
    ("identifier", Form::ContainerOnly),
    ("refSystem", Form::ContainerOnly),
    ("div", Form::ContainerOnly),
    ("title", Form::ContainerOnly),
    ("p", Form::ContainerOnly),
    ("table", Form::ContainerOnly),
    ("row", Form::ContainerOnly),
    ("cell", Form::ContainerOnly),
    ("hi", Form::ContainerOnly),
    ("reference", Form::ContainerOnly),
];

/// The converter's four USFM marker comments, each directly before the
/// title of the paired type (survey 1). Validated, not imported.
pub const MHCC_COMMENTS: &[(&str, &str)] = &[
    ("<!-- s2 -->", "x-s2"),
    ("<!-- is -->", "x-IS"),
    ("<!-- ms -->", "x-ms"),
    ("<!-- mt -->", "main"),
];

pub const MHCC_XML: XmlProfile = XmlProfile {
    file: "mhcc.osis.xml",
    prefix: b"<?xml version=\"1.0\" encoding=\"utf-8\"?>",
    allow_cr: false,
    double_quoted: true,
    forms: MHCC_FORMS,
    subset: None,
    comments: MHCC_COMMENTS,
    amp: true,
};

/// The root and `osisText` start tags: exactly these attributes and values
/// (inert strings, never resolved).
const ROOT_ATTRS: &[(&str, &str)] = &[
    (
        "xmlns",
        "http://www.bibletechnologies.net/2003/OSIS/namespace",
    ),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    (
        "xsi:schemaLocation",
        "http://www.bibletechnologies.net/2003/OSIS/namespace\n        http://www.bibletechnologies.net/osisCore.2.1.1.xsd",
    ),
];
const OSISTEXT_ATTRS: &[(&str, &str)] = &[
    ("osisIDWork", "MHCC"),
    ("osisRefWork", "Bible"),
    ("xml:lang", "en"),
];

/// Pinned facts of the real file (docs/pipeline/mhcc-osis-survey.md). A fact
/// that no longer holds fails the build (condition 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmtFacts {
    pub chapters: usize,
    /// (element, its attributes as `name=value;…` or `-`, count).
    pub blocks: Vec<(String, String, usize)>,
    pub blocks_in_anchors: usize,
    pub blocks_before_first_chapter: usize,
    pub anchors: usize,
    pub single_verse_anchors: usize,
    pub verse_refs: usize,
    pub distinct_verses: usize,
    pub references: usize,
    pub hi: usize,
    pub amp: usize,
    /// Per [`MHCC_COMMENTS`] entry.
    pub comments: [usize; 4],
    pub toc_milestones: usize,
    pub chapter_milestones_in_book_div: usize,
    // Anomalies (survey 6), stored as the source has them.
    pub verses_in_two_anchors: usize,
    pub verse1_anchor_then_relisted: usize,
    pub chapters_only_verse1_anchor: usize,
    pub multi_block_anchors: usize,
    pub reused_sids: Vec<String>,
    pub chapters_without_anchor: Vec<String>,
    pub descending_refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CmtProfile {
    pub source_id: &'static str,
    pub xml: XmlProfile,
    /// `None` for synthetic fixtures: then fewer than 66 books are allowed
    /// (still in canonical order) and no count is pinned.
    pub facts: Option<CmtFacts>,
}

pub fn mhcc() -> CmtProfile {
    CmtProfile {
        source_id: MHCC_ID,
        xml: MHCC_XML,
        facts: Some(CmtFacts {
            chapters: 1189,
            blocks: [
                ("p", "-", 7014),
                ("p", "subType=x-introduction", 4657),
                ("table", "-", 2531),
                ("title", "type=main", 5),
                ("title", "type=x-IS", 800),
                ("title", "type=x-ms", 61),
                ("title", "type=x-s2", 1187),
            ]
            .map(|(n, a, c)| (n.to_string(), a.to_string(), c))
            .to_vec(),
            blocks_in_anchors: 8078,
            blocks_before_first_chapter: 10,
            anchors: 4297,
            single_verse_anchors: 850,
            verse_refs: 28968,
            distinct_verses: 28718,
            references: 1204,
            hi: 3930,
            amp: 8,
            comments: [1187, 800, 61, 5],
            toc_milestones: 198,
            chapter_milestones_in_book_div: 97,
            verses_in_two_anchors: 250,
            verse1_anchor_then_relisted: 248,
            chapters_only_verse1_anchor: 118,
            multi_block_anchors: 3513,
            reused_sids: strings(&[
                "Isa.4.1",
                "Isa.56.1",
                "Isa.57.1",
                "Isa.58.1",
                "Jer.4.1",
                "Prov.10.1",
                "Song.1.1",
                "Song.5.1",
                "Song.6.1",
            ]),
            chapters_without_anchor: strings(&["Lev.19", "Ps.108"]),
            descending_refs: strings(&[
                "Exod.13.2-Exod.10.13",
                "Ezek.3.12-Ezek.3.2",
                "Jer.50.1-Jer.3.8",
                "Jer.50.4-Jer.7.17",
            ]),
        }),
    }
}

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// The same rules without pinned facts, for synthetic fixtures.
pub fn fixture() -> CmtProfile {
    CmtProfile {
        facts: None,
        ..mhcc()
    }
}

// ------------------------------------------------------------------ model

/// Allowed attribute-name sets for one element (each set is one allowed shape).
type AttrSets<'a> = &'a [&'a [&'a str]];
/// Closed value lists, per attribute name.
type AttrValues<'a> = &'a [(&'a str, &'a [&'a str])];

/// One attribute, raw. `refs` is set on `reference@osisRef` only: the KJV
/// verse ids of the reference's first and last verse, as written (a single
/// verse gives the same id twice; a descending range stays descending).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmtAttr {
    pub name: String,
    pub value: String,
    pub refs: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmtNodeKind {
    Element {
        name: String,
        attrs: Vec<CmtAttr>,
    },
    /// Text exactly as in the source, `&amp;` decoded to `&`.
    Text(String),
}

/// A node of a block, in document order. Node 0 is the block element itself
/// (`parent` `None`); every other node has a parent element node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmtNode {
    pub parent: Option<u32>,
    pub kind: CmtNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// KJV book order, 1..66.
    pub book: u8,
    /// The chapter open at the block's start tag, `None` before the book's
    /// first chapter milestone.
    pub chapter: Option<u16>,
    /// 1-based anchor this block belongs to, `None` outside every anchor.
    pub anchor: Option<u32>,
    pub nodes: Vec<CmtNode>,
    /// File offsets of the block's first and one past its last byte.
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    /// The milestone's sID as written (not unique in MHCC, survey 6.5).
    pub sid: String,
    /// KJV verse ids, in `osisID` order.
    pub verses: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CmtStats {
    pub chapters: usize,
    pub amp: usize,
    pub comments: [usize; 4],
    pub toc_milestones: usize,
    pub chapter_milestones_in_book_div: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmtModel {
    pub source_id: &'static str,
    pub blocks: Vec<Block>,
    pub anchors: Vec<Anchor>,
    pub stats: CmtStats,
}

/// KJV verse id (DESIGN.md 8): book * 1,000,000 + chapter * 1,000 + verse.
pub fn verse_id(book: u8, chapter: u16, verse: u16) -> u32 {
    u32::from(book) * 1_000_000 + u32::from(chapter) * 1000 + u32::from(verse)
}

/// `Gen.1.1` for a verse id.
pub fn osis_ref(id: u32) -> String {
    let book = (id / 1_000_000) as usize;
    let name = KJV_BOOKS.get(book.wrapping_sub(1)).map_or("?", |b| b.0);
    format!("{name}.{}.{}", id / 1000 % 1000, id % 1000)
}

fn book_ord(osis: &str) -> Option<u8> {
    KJV_BOOKS
        .iter()
        .position(|(o, _)| *o == osis)
        .map(|i| (i + 1) as u8)
}

/// `[1-9][0-9]{0,2}`: the number spelling the profile accepts.
fn num(s: &str) -> Option<u16> {
    let ok = !s.is_empty() && s.len() <= 3 && !s.starts_with('0');
    if ok && s.bytes().all(|b| b.is_ascii_digit()) {
        s.parse().ok()
    } else {
        None
    }
}

/// `Book.C.V` → (book ord, chapter, verse); the book must be one of the 66.
fn parse_verse(s: &str) -> Option<(u8, u16, u16)> {
    let mut it = s.split('.');
    let (b, c, v) = (it.next()?, it.next()?, it.next()?);
    if it.next().is_some() {
        return None;
    }
    Some((book_ord(b)?, num(c)?, num(v)?))
}

/// `reference@osisRef`: `Book.C.V` or `Book.C.V-Book.C.V` in one book.
pub fn parse_osis_ref(s: &str) -> Option<(u32, u32)> {
    let (a, b) = match s.split_once('-') {
        Some((a, b)) => (parse_verse(a)?, parse_verse(b)?),
        None => {
            let a = parse_verse(s)?;
            (a, a)
        }
    };
    (a.0 == b.0).then(|| (verse_id(a.0, a.1, a.2), verse_id(b.0, b.1, b.2)))
}

// ------------------------------------------------------------------ serialise

/// The canonical spelling of a tag inside a book (survey 1): attributes in
/// order, double-quoted, one space before `/>`.
fn tag(name: &str, attrs: &[(&str, &str)], empty: bool) -> String {
    let mut s = format!("<{name}");
    for (k, v) in attrs {
        s.push_str(&format!(" {k}=\"{v}\""));
    }
    s.push_str(if empty { " />" } else { ">" });
    s
}

/// Elements written in the empty form (DESIGN.md 9.4, `Empty` only).
fn empty_only(name: &str) -> bool {
    matches!(name, "verse" | "chapter" | "milestone")
}

/// One node row as stored: (parent, element name, text).
pub type NodeRow = (Option<i64>, Option<String>, Option<String>);
/// One attribute row as stored: (node n, name, value).
pub type AttrRow = (i64, String, String);

/// Rebuilds a block's source bytes from its stored rows alone (nodes in `n`
/// order; attributes in `(n, i)` order): the canonical tag spelling, text
/// with `&` written back as `&amp;`. T22 compares the result with the source
/// bytes of every block.
pub fn serialise_block(nodes: &[NodeRow], attrs: &[AttrRow]) -> String {
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (n, (parent, _, _)) in nodes.iter().enumerate() {
        if let Some(p) = parent.and_then(|p| usize::try_from(p).ok())
            && p < n
        {
            children[p].push(n);
        }
    }
    let mut out = String::new();
    if !nodes.is_empty() {
        write_node(0, nodes, attrs, &children, &mut out);
    }
    out
}

fn write_node(
    n: usize,
    nodes: &[NodeRow],
    attrs: &[AttrRow],
    children: &[Vec<usize>],
    out: &mut String,
) {
    match &nodes[n] {
        (_, Some(name), _) => {
            let list: Vec<(&str, &str)> = attrs
                .iter()
                .filter(|a| a.0 == n as i64)
                .map(|a| (a.1.as_str(), a.2.as_str()))
                .collect();
            let empty = empty_only(name) && children[n].is_empty();
            out.push_str(&tag(name, &list, empty));
            if !empty {
                for &c in &children[n] {
                    write_node(c, nodes, attrs, children, out);
                }
                out.push_str(&format!("</{name}>"));
            }
        }
        (_, None, Some(text)) => out.push_str(&text.replace('&', "&amp;")),
        (_, None, None) => {}
    }
}

impl Block {
    /// The block's rows as [`serialise_block`] takes them.
    pub fn rows(&self) -> (Vec<NodeRow>, Vec<AttrRow>) {
        let mut nodes = Vec::new();
        let mut attrs = Vec::new();
        for (n, node) in self.nodes.iter().enumerate() {
            let parent = node.parent.map(i64::from);
            match &node.kind {
                CmtNodeKind::Element { name, attrs: a } => {
                    nodes.push((parent, Some(name.clone()), None));
                    for x in a {
                        attrs.push((n as i64, x.name.clone(), x.value.clone()));
                    }
                }
                CmtNodeKind::Text(t) => nodes.push((parent, None, Some(t.clone()))),
            }
        }
        (nodes, attrs)
    }

    /// The block's text: every text node in document order.
    pub fn text(&self) -> String {
        self.nodes
            .iter()
            .filter_map(|n| match &n.kind {
                CmtNodeKind::Text(t) => Some(t.as_str()),
                CmtNodeKind::Element { .. } => None,
            })
            .collect()
    }

    fn element(&self) -> (&str, &[CmtAttr]) {
        match &self.nodes[0].kind {
            CmtNodeKind::Element { name, attrs } => (name, attrs),
            CmtNodeKind::Text(_) => ("", &[]),
        }
    }

    /// `name=value;…` of the block element's attributes, `-` for none.
    pub fn attr_signature(&self) -> String {
        let a = self.element().1;
        if a.is_empty() {
            return "-".into();
        }
        a.iter()
            .map(|x| format!("{}={}", x.name, x.value))
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn name(&self) -> &str {
        self.element().0
    }
}

// ------------------------------------------------------------------ parser

/// Where an element outside blocks sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Osis,
    OsisText,
    Header,
    /// `revisionDesc`, `work`: whitespace text only.
    RevisionDesc,
    Work,
    /// `date`, the header's `p` and `title`, `description`, `type`,
    /// `identifier`, `refSystem`: text (not imported).
    HeaderLeaf,
    Book,
    Intro,
}

struct OpenBlock {
    b: Block,
    /// Open element nodes (indices into `b.nodes`).
    stack: Vec<u32>,
    /// The previous event was text (an `&amp;` splits one text run into
    /// three events; they make one node).
    last_text: bool,
    /// The open anchor's eID was read in this block: only whitespace and
    /// chapter milestones may follow.
    after_eid: bool,
}

struct OpenAnchor {
    ord: u32,
    chapter: u16,
    closed: bool,
}

struct State<'a> {
    p: &'a CmtProfile,
    data: &'a [u8],
    ctx: Vec<Ctx>,
    header_seen: bool,
    books: usize,
    last_book: u8,
    book: Option<u8>,
    last_chapter: u16,
    chapter: Option<u16>,
    block: Option<OpenBlock>,
    anchor: Option<OpenAnchor>,
    m: CmtModel,
}

fn is_ws(s: &str) -> bool {
    s.bytes().all(|b| b == b' ' || b == b'\n')
}

impl State<'_> {
    fn err(&self, check: &'static str, t: &Tok<'_>, msg: impl Into<String>) -> Error {
        let e = Error::new(check, msg).file(self.p.xml.file).at(t.start);
        match (self.book, self.chapter) {
            (Some(b), Some(c)) => e.verse(format!("{}.{c}", KJV_BOOKS[usize::from(b) - 1].0)),
            _ => e,
        }
    }

    /// The tag's bytes must be the canonical spelling (inside books).
    fn canonical(&self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        let raw = &self.data[t.start as usize..t.end as usize];
        let want = match t.kind {
            Kind::End => format!("</{}>", t.name),
            _ => tag(t.name, &t.attrs, t.kind == Kind::Empty),
        };
        if raw == want.as_bytes() {
            Ok(())
        } else {
            Err(self.err(
                "cmt-tag",
                t,
                format!("tag not in the canonical spelling: {}", excerpt(raw)),
            ))
        }
    }

    fn attrs_are(
        &self,
        t: &Tok<'_>,
        allowed: AttrSets<'_>,
        values: AttrValues<'_>,
    ) -> std::result::Result<(), Error> {
        let names: Vec<&str> = t.attrs.iter().map(|(k, _)| *k).collect();
        if !allowed.contains(&names.as_slice()) {
            return Err(self.err(
                "cmt-attr",
                t,
                format!(
                    "<{}> with attributes {} (allowed: {allowed:?})",
                    t.name,
                    excerpt(names.join(",").as_bytes())
                ),
            ));
        }
        for (k, v) in &t.attrs {
            if let Some((_, list)) = values.iter().find(|(n, _)| n == k)
                && !list.contains(v)
            {
                return Err(self.err(
                    "cmt-attr",
                    t,
                    format!(
                        "<{} {k}=\"{}\"> is not allowed",
                        t.name,
                        excerpt(v.as_bytes())
                    ),
                ));
            }
        }
        Ok(())
    }

    fn on(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        if self.block.is_some() {
            return self.in_block(t);
        }
        match t.kind {
            Kind::Text => {
                let leaf = self.ctx.last() == Some(&Ctx::HeaderLeaf);
                if leaf || is_ws(t.text) {
                    Ok(())
                } else {
                    Err(self.err("cmt-structure", t, "text outside a block"))
                }
            }
            Kind::End => self.end_outside(t),
            Kind::Start | Kind::Empty => self.start_outside(t),
        }
    }

    fn end_outside(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        let ctx = self.ctx.pop();
        if matches!(ctx, Some(Ctx::Book | Ctx::Intro)) {
            self.canonical(t)?;
        }
        if ctx == Some(Ctx::Book) {
            if self.anchor.is_some() || self.chapter.is_some() {
                return Err(self.err(
                    "cmt-structure",
                    t,
                    "book ends with an open verse anchor or chapter",
                ));
            }
            self.book = None;
        }
        Ok(())
    }

    fn start_outside(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        let parent = self.ctx.last().copied();
        let in_book = matches!(parent, Some(Ctx::Book | Ctx::Intro));
        if in_book {
            self.canonical(t)?;
        }
        if t.name == "title" && in_book && t.comment.is_none() {
            return Err(self.err(
                "cmt-comment",
                t,
                "title without its converter comment directly before it",
            ));
        }
        let none: &[&[&str]] = &[&[]];
        let ctx = match (t.name, parent) {
            ("osis", None) => {
                if t.attrs.as_slice() != ROOT_ATTRS {
                    return Err(self.err("cmt-attr", t, "<osis> differs from the pinned root"));
                }
                Ctx::Osis
            }
            ("osisText", Some(Ctx::Osis)) => {
                if t.attrs.as_slice() != OSISTEXT_ATTRS {
                    return Err(self.err("cmt-attr", t, "<osisText> differs from the pinned one"));
                }
                Ctx::OsisText
            }
            ("header", Some(Ctx::OsisText)) => {
                if self.header_seen || self.books > 0 {
                    return Err(self.err("cmt-structure", t, "header not once, before the books"));
                }
                self.attrs_are(t, none, &[])?;
                self.header_seen = true;
                Ctx::Header
            }
            ("revisionDesc", Some(Ctx::Header)) => {
                self.attrs_are(t, &[&["resp"]], &[])?;
                Ctx::RevisionDesc
            }
            ("work", Some(Ctx::Header)) => {
                self.attrs_are(t, &[&["osisWork"]], &[])?;
                Ctx::Work
            }
            ("date" | "p", Some(Ctx::RevisionDesc)) => {
                self.attrs_are(t, none, &[])?;
                Ctx::HeaderLeaf
            }
            ("title" | "description" | "type" | "identifier" | "refSystem", Some(Ctx::Work)) => {
                let allowed: &[&[&str]] = match t.name {
                    "description" => &[&["type", "subType"]],
                    "type" | "identifier" => &[&["type"]],
                    _ => none,
                };
                self.attrs_are(t, allowed, &[])?;
                Ctx::HeaderLeaf
            }
            ("div", Some(Ctx::OsisText)) => {
                self.book_start(t)?;
                Ctx::Book
            }
            ("div", Some(Ctx::Book)) => {
                self.attrs_are(t, &[&["type"]], &[("type", &["introduction"])])?;
                Ctx::Intro
            }
            ("milestone", Some(Ctx::Book)) => {
                self.attrs_are(
                    t,
                    &[&["type", "n"]],
                    &[("type", &["x-usfm-toc1", "x-usfm-toc2", "x-usfm-toc3"])],
                )?;
                self.m.stats.toc_milestones += 1;
                return Ok(());
            }
            ("chapter", Some(Ctx::Book)) => {
                self.m.stats.chapter_milestones_in_book_div += 1;
                return self.chapter_milestone(t);
            }
            ("p" | "title" | "table", Some(Ctx::Book | Ctx::Intro)) => {
                return self.block_start(t);
            }
            _ => {
                return Err(self.err(
                    "cmt-structure",
                    t,
                    format!(
                        "<{}> is not allowed here (outside the import scope)",
                        t.name
                    ),
                ));
            }
        };
        self.ctx.push(ctx);
        Ok(())
    }

    fn book_start(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        self.attrs_are(
            t,
            &[&["type", "osisID", "canonical"]],
            &[("type", &["book"]), ("canonical", &["true"])],
        )?;
        // Canonical order: with pinned facts every book in turn, in a fixture
        // a subset in canonical order.
        let osis = t.attr("osisID").unwrap_or("");
        let ord = book_ord(osis).filter(|&o| o > self.last_book);
        let ord = match (ord, &self.p.facts) {
            (Some(o), Some(_)) if usize::from(o) == self.books + 1 => o,
            (Some(o), None) => o,
            _ => {
                return Err(self.err(
                    "cmt-structure",
                    t,
                    format!(
                        "book {} is not the next canonical book",
                        excerpt(osis.as_bytes())
                    ),
                ));
            }
        };
        self.books += 1;
        self.last_book = ord;
        self.book = Some(ord);
        self.last_chapter = 0;
        Ok(())
    }

    fn chapter_milestone(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        self.attrs_are(t, &[&["sID", "osisID", "n"], &["eID"]], &[])?;
        let book = KJV_BOOKS[usize::from(self.book.unwrap_or(1)) - 1].0;
        if let Some(sid) = t.attr("sID") {
            let c = self.last_chapter + 1;
            let want = format!("{book}.{c}");
            let ok = self.chapter.is_none()
                && sid == want
                && t.attr("osisID") == Some(sid)
                && t.attr("n") == Some(c.to_string().as_str());
            if !ok {
                return Err(self.err(
                    "cmt-chapter",
                    t,
                    format!("chapter sID {} is not {want}", excerpt(sid.as_bytes())),
                ));
            }
            self.chapter = Some(c);
            self.last_chapter = c;
            self.m.stats.chapters += 1;
        } else {
            let eid = t.attr("eID").unwrap_or("");
            let open = self.chapter.map(|c| format!("{book}.{c}"));
            if open.as_deref() != Some(eid) {
                return Err(self.err(
                    "cmt-chapter",
                    t,
                    format!(
                        "chapter eID {} does not close the open chapter",
                        excerpt(eid.as_bytes())
                    ),
                ));
            }
            if self.anchor.as_ref().is_some_and(|a| !a.closed) {
                return Err(self.err("cmt-anchor", t, "chapter ends inside a verse anchor"));
            }
            self.chapter = None;
        }
        Ok(())
    }

    fn block_start(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        let (allowed, values): (AttrSets<'_>, AttrValues<'_>) = match t.name {
            "p" => (&[&[], &["subType"]], &[("subType", &["x-introduction"])]),
            "title" => (&[&["type"]], &[("type", &["x-s2", "x-IS", "x-ms", "main"])]),
            _ => (&[&[]], &[]),
        };
        self.attrs_are(t, allowed, values)?;
        let book = self.book.expect("blocks only inside books");
        let anchor = self.anchor.as_ref().map(|a| a.ord);
        let b = Block {
            book,
            chapter: self.chapter,
            anchor,
            nodes: vec![CmtNode {
                parent: None,
                kind: CmtNodeKind::Element {
                    name: t.name.to_string(),
                    attrs: own_attrs(&t.attrs),
                },
            }],
            start: t.start,
            end: t.end,
        };
        self.block = Some(OpenBlock {
            b,
            stack: vec![0],
            last_text: false,
            after_eid: false,
        });
        Ok(())
    }

    fn in_block(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        let ob = self.block.as_ref().unwrap();
        let top = *ob.stack.last().unwrap();
        let parent_name = match &ob.b.nodes[top as usize].kind {
            CmtNodeKind::Element { name, .. } => name.clone(),
            CmtNodeKind::Text(_) => unreachable!("only elements are on the stack"),
        };
        match t.kind {
            Kind::Text => {
                let ws = is_ws(t.text);
                if !ws && matches!(parent_name.as_str(), "table" | "row") {
                    return Err(self.err("cmt-structure", t, "text directly in <table> or <row>"));
                }
                if !ws && ob.after_eid {
                    return Err(self.err("cmt-anchor", t, "text after a verse eID in its block"));
                }
                let ob = self.block.as_mut().unwrap();
                match ob.b.nodes.last_mut() {
                    Some(CmtNode {
                        parent: Some(p),
                        kind: CmtNodeKind::Text(s),
                    }) if ob.last_text && *p == top => s.push_str(t.text),
                    _ => ob.b.nodes.push(CmtNode {
                        parent: Some(top),
                        kind: CmtNodeKind::Text(t.text.to_string()),
                    }),
                }
                ob.last_text = true;
                Ok(())
            }
            Kind::End => {
                self.canonical(t)?;
                let ob = self.block.as_mut().unwrap();
                ob.last_text = false;
                ob.stack.pop();
                if ob.stack.is_empty() {
                    let mut ob = self.block.take().unwrap();
                    ob.b.end = t.end;
                    if ob.after_eid {
                        self.anchor = None;
                    }
                    self.m.blocks.push(ob.b);
                }
                Ok(())
            }
            Kind::Start | Kind::Empty => {
                self.canonical(t)?;
                let allowed_parent: &[&str] = match t.name {
                    "verse" | "chapter" | "hi" | "reference" => &["p"],
                    "row" => &["table"],
                    "cell" => &["row"],
                    _ => &[],
                };
                if !allowed_parent.contains(&parent_name.as_str()) {
                    return Err(self.err(
                        "cmt-structure",
                        t,
                        format!(
                            "<{}> is not allowed in <{parent_name}> (outside the import scope)",
                            t.name
                        ),
                    ));
                }
                if ob.after_eid && t.name != "chapter" {
                    return Err(self.err(
                        "cmt-anchor",
                        t,
                        format!("<{}> after a verse eID in its block", t.name),
                    ));
                }
                let mut attrs = own_attrs(&t.attrs);
                match t.name {
                    "verse" => self.verse_milestone(t)?,
                    "chapter" => self.chapter_milestone(t)?,
                    "hi" => self.attrs_are(t, &[&["type"]], &[("type", &["bold"])])?,
                    "reference" => {
                        self.attrs_are(t, &[&["osisRef"]], &[])?;
                        let v = t.attr("osisRef").unwrap_or("");
                        let refs = parse_osis_ref(v).ok_or_else(|| {
                            self.err(
                                "cmt-ref",
                                t,
                                format!(
                                    "osisRef {} is not Book.C.V or Book.C.V-Book.C.V in one book",
                                    excerpt(v.as_bytes())
                                ),
                            )
                        })?;
                        attrs[0].refs = Some(refs);
                    }
                    _ => self.attrs_are(t, &[&[]], &[])?,
                }
                let ob = self.block.as_mut().unwrap();
                ob.last_text = false;
                let n = ob.b.nodes.len() as u32;
                ob.b.nodes.push(CmtNode {
                    parent: Some(top),
                    kind: CmtNodeKind::Element {
                        name: t.name.to_string(),
                        attrs,
                    },
                });
                if t.kind == Kind::Start {
                    ob.stack.push(n);
                }
                Ok(())
            }
        }
    }

    fn verse_milestone(&mut self, t: &Tok<'_>) -> std::result::Result<(), Error> {
        self.attrs_are(t, &[&["sID", "osisID", "n"], &["eID"]], &[])?;
        let book = self.book.unwrap_or(1);
        let bname = KJV_BOOKS[usize::from(book) - 1].0;
        if let Some(sid) = t.attr("sID") {
            if self.anchor.is_some() {
                return Err(self.err("cmt-anchor", t, "verse sID inside an open verse anchor"));
            }
            let ob = self.block.as_ref().unwrap();
            let first = ob.b.nodes[1..]
                .iter()
                .all(|n| matches!(&n.kind, CmtNodeKind::Text(s) if is_ws(s)));
            if !first {
                return Err(self.err("cmt-anchor", t, "verse sID is not first in its block"));
            }
            let chapter = self
                .chapter
                .ok_or_else(|| self.err("cmt-anchor", t, "verse sID outside a chapter"))?;
            // sID: Book.C.V or Book.C.V1-V2 (V1 < V2) in the open book and chapter.
            let prefix = format!("{bname}.{chapter}.");
            let range = sid
                .strip_prefix(&prefix)
                .and_then(|r| match r.split_once('-') {
                    Some((a, b)) => Some((num(a)?, num(b)?, true)),
                    None => num(r).map(|v| (v, v, false)),
                });
            let Some((v1, v2, is_range)) = range.filter(|(a, b, r)| a < b || (!r && a == b)) else {
                return Err(self.err(
                    "cmt-anchor",
                    t,
                    format!(
                        "verse sID {} is not {prefix}V or {prefix}V-V",
                        excerpt(sid.as_bytes())
                    ),
                ));
            };
            // osisID must list exactly every verse of the range, in order;
            // n is "V" or "V1-V2". The stored verses are the osisID items.
            let listed: Vec<&str> = t.attr("osisID").unwrap_or("").split(' ').collect();
            let want: Vec<String> = (v1..=v2).map(|v| format!("{prefix}{v}")).collect();
            let want_n = if is_range {
                format!("{v1}-{v2}")
            } else {
                v1.to_string()
            };
            if listed != want || t.attr("n") != Some(want_n.as_str()) {
                return Err(self.err(
                    "cmt-anchor",
                    t,
                    format!(
                        "verse {}: osisID or n does not list exactly its verses",
                        excerpt(sid.as_bytes())
                    ),
                ));
            }
            let mut verses = Vec::with_capacity(listed.len());
            for item in &listed {
                let (b, c, v) = parse_verse(item)
                    .ok_or_else(|| self.err("cmt-anchor", t, "osisID item not Book.C.V"))?;
                verses.push(verse_id(b, c, v));
            }
            self.m.anchors.push(Anchor {
                sid: sid.to_string(),
                verses,
            });
            let ord = self.m.anchors.len() as u32;
            self.anchor = Some(OpenAnchor {
                ord,
                chapter,
                closed: false,
            });
            self.block.as_mut().unwrap().b.anchor = Some(ord);
        } else {
            let eid = t.attr("eID").unwrap_or("");
            let ok = match &self.anchor {
                Some(a) => {
                    !a.closed
                        && self.m.anchors[a.ord as usize - 1].sid == eid
                        && self.chapter == Some(a.chapter)
                }
                None => false,
            };
            if !ok {
                return Err(self.err(
                    "cmt-anchor",
                    t,
                    format!(
                        "verse eID {} does not close the open verse anchor",
                        excerpt(eid.as_bytes())
                    ),
                ));
            }
            self.anchor.as_mut().unwrap().closed = true;
            self.block.as_mut().unwrap().after_eid = true;
        }
        Ok(())
    }
}

fn own_attrs(attrs: &[(&str, &str)]) -> Vec<CmtAttr> {
    attrs
        .iter()
        .map(|(k, v)| CmtAttr {
            name: k.to_string(),
            value: v.to_string(),
            refs: None,
        })
        .collect()
}

/// Stages 2 and 3: parse `data` under `p` and check every pinned fact.
/// Anchors and references are resolved against the KJV by [`resolve`].
pub fn parse(data: &[u8], p: &CmtProfile) -> Result<CmtModel> {
    // Characters (DESIGN.md 17.3): LF and printable ASCII only, checked on
    // the raw bytes before the XML scan.
    if let Some(i) = data
        .iter()
        .position(|&b| b != b'\n' && !(0x20..=0x7e).contains(&b))
    {
        return Err(Error::new(
            "cmt-char",
            format!(
                "byte 0x{:02x}: only printable ASCII and LF are allowed",
                data[i]
            ),
        )
        .file(p.xml.file)
        .at(i as u64)
        .into());
    }
    let mut st = State {
        p,
        data,
        ctx: Vec::new(),
        header_seen: false,
        books: 0,
        last_book: 0,
        book: None,
        last_chapter: 0,
        chapter: None,
        block: None,
        anchor: None,
        m: CmtModel {
            source_id: p.source_id,
            blocks: Vec::new(),
            anchors: Vec::new(),
            stats: CmtStats::default(),
        },
    };
    let mut comments = [0usize; 4];
    xmlscan::scan(data, &p.xml, |t, _anc| {
        if let Some(c) = t.comment
            && let Some(i) = MHCC_COMMENTS.iter().position(|(x, _)| *x == c)
        {
            comments[i] += 1;
        }
        st.on(t)
    })?;
    let books = st.books;
    let mut m = st.m;
    m.stats.comments = comments;
    m.stats.amp = count(data, b"&amp;");
    if books == 0 || (p.facts.is_some() && books != 66) {
        return Err(
            Error::new("cmt-structure", format!("{books} books, not 66"))
                .file(p.xml.file)
                .into(),
        );
    }
    if let Some(f) = &p.facts {
        let got = facts_of(&m);
        if got != *f {
            return Err(Error::new(
                "cmt-facts",
                format!("pinned facts differ (survey): got {got:?}"),
            )
            .file(p.xml.file)
            .into());
        }
    }
    Ok(m)
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

/// The facts the model shows, in the shape of [`CmtFacts`] (the survey's
/// numbers, computed from the parse).
pub fn facts_of(m: &CmtModel) -> CmtFacts {
    let mut kinds: BTreeMap<(String, String), usize> = BTreeMap::new();
    for b in &m.blocks {
        *kinds
            .entry((b.name().to_string(), b.attr_signature()))
            .or_default() += 1;
    }
    let mut per_verse: BTreeMap<u32, usize> = BTreeMap::new();
    for a in &m.anchors {
        for &v in &a.verses {
            *per_verse.entry(v).or_default() += 1;
        }
    }
    let single_v1 = |a: &Anchor| a.verses.len() == 1 && a.verses[0] % 1000 == 1;
    let relisted = m
        .anchors
        .windows(2)
        .filter(|w| single_v1(&w[0]) && w[1].verses.contains(&w[0].verses[0]))
        .count();
    let mut by_chapter: BTreeMap<u32, Vec<&Anchor>> = BTreeMap::new();
    for a in &m.anchors {
        by_chapter.entry(a.verses[0] / 1000).or_default().push(a);
    }
    let only_v1 = by_chapter
        .values()
        .filter(|l| l.len() == 1 && single_v1(l[0]))
        .count();
    let mut span: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
    for (i, b) in m.blocks.iter().enumerate() {
        if let Some(a) = b.anchor {
            span.entry(a).or_insert((i, i)).1 = i;
        }
    }
    let mut sids: BTreeMap<&str, usize> = BTreeMap::new();
    for a in &m.anchors {
        *sids.entry(a.sid.as_str()).or_default() += 1;
    }
    let chapters: BTreeSet<u32> = m
        .blocks
        .iter()
        .filter_map(|b| b.chapter.map(|c| u32::from(b.book) * 1000 + u32::from(c)))
        .collect();
    let mut refs = 0;
    let mut descending = Vec::new();
    let mut hi = 0;
    for b in &m.blocks {
        for n in &b.nodes {
            if let CmtNodeKind::Element { name, attrs } = &n.kind {
                hi += usize::from(name == "hi");
                for a in attrs {
                    if let Some((s, e)) = a.refs {
                        refs += 1;
                        if e < s {
                            descending.push(a.value.clone());
                        }
                    }
                }
            }
        }
    }
    descending.sort();
    CmtFacts {
        chapters: m.stats.chapters,
        blocks: kinds.into_iter().map(|((n, a), c)| (n, a, c)).collect(),
        blocks_in_anchors: m.blocks.iter().filter(|b| b.anchor.is_some()).count(),
        blocks_before_first_chapter: m.blocks.iter().filter(|b| b.chapter.is_none()).count(),
        anchors: m.anchors.len(),
        single_verse_anchors: m.anchors.iter().filter(|a| a.verses.len() == 1).count(),
        verse_refs: m.anchors.iter().map(|a| a.verses.len()).sum(),
        distinct_verses: per_verse.len(),
        references: refs,
        hi,
        amp: m.stats.amp,
        comments: m.stats.comments,
        toc_milestones: m.stats.toc_milestones,
        chapter_milestones_in_book_div: m.stats.chapter_milestones_in_book_div,
        verses_in_two_anchors: per_verse.values().filter(|&&n| n > 1).count(),
        verse1_anchor_then_relisted: relisted,
        chapters_only_verse1_anchor: only_v1,
        multi_block_anchors: span.values().filter(|(a, b)| b > a).count(),
        reused_sids: sids
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(s, _)| s.to_string())
            .collect(),
        chapters_without_anchor: chapters
            .iter()
            .filter(|c| !by_chapter.contains_key(c))
            .map(|&c| {
                let r = osis_ref(c * 1000 + 1);
                r[..r.rfind('.').unwrap_or(r.len())].to_string()
            })
            .collect(),
        descending_refs: descending,
    }
}

/// Every anchored verse and both ends of every reference are KJV verses
/// (`table`: VRS-ENG-PALASO + the KJV overrides, the table kjv.sqlite's
/// verses are checked against). On failure the error lists every one that
/// is not; nothing is dropped, moved or fixed.
pub fn resolve(m: &CmtModel, table: &Table) -> Result<()> {
    let mut bad: BTreeSet<String> = BTreeSet::new();
    for a in &m.anchors {
        for &v in &a.verses {
            if !is_kjv_verse(table, v) {
                bad.insert(format!("anchor {} lists {}", a.sid, osis_ref(v)));
            }
        }
    }
    for b in &m.blocks {
        for n in &b.nodes {
            if let CmtNodeKind::Element { attrs, .. } = &n.kind {
                for a in attrs {
                    if let Some((s, e)) = a.refs
                        && (!is_kjv_verse(table, s) || !is_kjv_verse(table, e))
                    {
                        bad.insert(format!("reference {}", excerpt(a.value.as_bytes())));
                    }
                }
            }
        }
    }
    if bad.is_empty() {
        return Ok(());
    }
    let list: Vec<String> = bad.into_iter().collect();
    Err(Error::new(
        "cmt-kjv",
        format!(
            "{} anchors or references are not KJV verses: {}",
            list.len(),
            list.join(", ")
        ),
    )
    .into())
}

/// `id` is a verse of the KJV versification table.
pub fn is_kjv_verse(table: &Table, id: u32) -> bool {
    let book = (id / 1_000_000) as usize;
    let (c, v) = ((id / 1000 % 1000) as u16, (id % 1000) as u16);
    match KJV_BOOKS.get(book.wrapping_sub(1)) {
        Some(b) => v >= 1 && table.get(b.1, c).is_some_and(|max| v <= max),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osis_ref_grammar() {
        assert_eq!(parse_osis_ref("John.3.16"), Some((43003016, 43003016)));
        assert_eq!(
            parse_osis_ref("Isa.40.1-Isa.40.5"),
            Some((23040001, 23040005))
        );
        // Descending ranges are kept as written (survey 6.7).
        assert_eq!(
            parse_osis_ref("Jer.50.1-Jer.3.8"),
            Some((24050001, 24003008))
        );
        for bad in [
            "John 3:16",
            "John.3",
            "John.03.16",
            "John.3.16-Rom.1.1",
            "Joh.3.16",
            "John.3.16-",
            "John.3.16.1",
            "John.3.0",
            "John.3.1000",
            "",
        ] {
            assert_eq!(parse_osis_ref(bad), None, "{bad}");
        }
    }

    #[test]
    fn serialise_block_spells_tags_canonically_and_escapes_amp() {
        let nodes: Vec<NodeRow> = vec![
            (None, Some("p".into()), None),
            (Some(0), None, Some("\n".into())),
            (Some(0), Some("verse".into()), None),
            (Some(0), Some("hi".into()), None),
            (Some(3), None, Some("A &c.".into())),
            (Some(0), Some("cell".into()), None),
        ];
        let attrs: Vec<AttrRow> = vec![
            (2, "sID".into(), "Gen.1.1".into()),
            (2, "n".into(), "1".into()),
            (3, "type".into(), "bold".into()),
        ];
        assert_eq!(
            serialise_block(&nodes, &attrs),
            "<p>\n<verse sID=\"Gen.1.1\" n=\"1\" /><hi type=\"bold\">A &amp;c.</hi><cell></cell></p>"
        );
    }
}
