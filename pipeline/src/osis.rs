//! Stage 2, parse: KJV OSIS bytes → typed model (DESIGN.md 3–6).
//!
//! A pure function over the verified buffer. Stored verse text is the
//! concatenation of the raw character data between a verse's `sID` and `eID`
//! milestones, minus `<note>` subtrees, copied unchanged (3.1). Marks, red
//! pieces and words are code-point ranges into that text (5.1). Red pieces
//! come **only** from `<q who="Jesus" marker="">` containers (5.2); there is
//! no other code path that creates one.

use crate::error::{Error, excerpt};
use crate::profile::Profile;
use crate::strongs::StrongsKey;
use crate::xmlscan::{self, Kind, Tok};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub books: Vec<Book>,
    /// Document order (= canonical order, checked).
    pub verses: Vec<Verse>,
    /// One per `<q>`, index = `q_ord - 1`.
    pub qs: Vec<QSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    /// 1-based canonical position.
    pub ord: u8,
    pub osis: String,
    pub usfm: String,
    pub chapters: u16,
    /// The character data of the book's `<title type="main">`, raw (tags
    /// such as `<abbr>` removed, their text kept), or `None` when the book
    /// has none (fixtures; the KJV profile requires one per book).
    pub main_title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse {
    pub book: u8,
    pub chapter: u16,
    pub verse: u16,
    /// OSIS reference as in the source (`Gen.1.1`).
    pub osis: String,
    pub text: String,
    /// `text` length in code points.
    pub len: u32,
    pub marks: Vec<Mark>,
    pub red: Vec<RedPiece>,
    pub words: Vec<Word>,
    /// File offset of the `sID` milestone.
    pub offset: u64,
}

impl Verse {
    /// `book * 1000000 + chapter * 1000 + verse` (the schema's verse id).
    pub fn id(&self) -> i64 {
        i64::from(self.book) * 1_000_000 + i64::from(self.chapter) * 1000 + i64::from(self.verse)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// `transChange type="added"`: italics.
    Added,
    /// `divineName`: small capitals.
    Divine,
}

impl MarkKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MarkKind::Added => "added",
            MarkKind::Divine => "divine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    pub kind: MarkKind,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedPiece {
    pub start: u32,
    pub end: u32,
    pub q_ord: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub start: u32,
    pub end: u32,
    pub keys: Vec<StrongsKey>,
}

/// One `<q>` container: where its tags are in the file (kjv-q.tsv).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QSpan {
    pub q_ord: u32,
    /// Book of the start tag (1-based).
    pub book: u8,
    /// Offset of the start tag's `<`.
    pub start_byte: u64,
    /// Offset of the byte after the end tag's `>`.
    pub end_byte: u64,
}

pub const MAX_VERSES: usize = 40_000;
pub const MAX_Q: usize = 10_000;
pub const MAX_VERSE_CP: u32 = 4096;

struct OpenVerse {
    v: Verse,
    note_depth: u32,
    open_marks: Vec<usize>,
    open_word: Option<usize>,
}

struct State<'p> {
    p: &'p Profile,
    file: &'static str,
    m: Model,
    /// Index into p.books of the current book div, if open.
    book: Option<usize>,
    /// Number of book divs seen.
    books_seen: usize,
    chapter: u16,
    chapter_open: bool,
    last_verse: u16,
    verse: Option<OpenVerse>,
    /// Open q: its index in m.qs.
    q: Option<usize>,
    /// Where the open q's piece starts in the current verse.
    piece_start: Option<u32>,
    /// Stack depth at which a colophon div opened.
    colophon: Option<usize>,
    anomaly_hits: Vec<usize>,
    /// Number of open ancestors of the current event (for matching div ends).
    depth_hint: usize,
    /// Ancestor count at which the open book div started.
    book_div_depth: Option<usize>,
    /// The open `<title type="main">`: its ancestor count and the text so far.
    main_title: Option<(usize, String)>,
}

fn has(ancestors: &[&str], name: &str) -> bool {
    ancestors.contains(&name)
}

/// Exactly these attribute names, each with an optional required value.
fn attrs_exactly(
    t: &Tok<'_>,
    required: &[(&str, Option<&str>)],
    optional: &[(&str, Option<&str>)],
) -> bool {
    for (k, v) in &t.attrs {
        let spec = required.iter().chain(optional).find(|(n, _)| n == k);
        match spec {
            None => return false,
            Some((_, Some(want))) if want != v => return false,
            _ => {}
        }
    }
    required.iter().all(|(n, _)| t.attr(n).is_some())
}

impl<'p> State<'p> {
    fn err(&self, check: &'static str, t: &Tok<'_>, msg: impl Into<String>) -> Error {
        let e = Error::new(check, msg).file(self.file).at(t.start);
        match &self.verse {
            Some(ov) => e.verse(ov.v.osis.clone()),
            None => e,
        }
    }

    fn bad_attrs(&self, t: &Tok<'_>) -> Error {
        let list: Vec<String> = t
            .attrs
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", excerpt(k.as_bytes()), excerpt(v.as_bytes())))
            .collect();
        self.err(
            "attr",
            t,
            format!(
                "<{}> attributes not allowed here: {}",
                t.name,
                list.join(" ")
            ),
        )
    }

    fn book_osis(&self) -> &'static str {
        self.book.map(|b| self.p.books[b].0).unwrap_or("")
    }

    fn on(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        if self.main_title.is_some() {
            return self.in_main_title(t, anc);
        }
        if self.verse.is_some() {
            self.in_verse(t, anc)
        } else {
            self.outside_verse(t, anc)
        }
    }

    // ------------------------------------------------------------ outside verses

    fn outside_verse(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let not_imported = has(anc, "title") || has(anc, "header") || self.colophon.is_some();
        match t.kind {
            Kind::Text => {
                if not_imported || t.text.bytes().all(|b| matches!(b, b' ' | b'\n' | b'\t')) {
                    Ok(())
                } else {
                    Err(self.err("structure", t, "stray text between verses"))
                }
            }
            Kind::End => self.end_outside(t),
            Kind::Start | Kind::Empty => {
                let name = t.name;
                if name == "q" && !not_imported {
                    return Err(self.err("R2", t, "<q> start tag between verses"));
                }
                match name {
                    "title" if t.attr("type") == Some("main") && !has(anc, "header") => {
                        self.main_title_start(t, anc)
                    }
                    "osis" | "osisText" | "header" | "title" => Ok(()),
                    "div" => self.div_start(t, anc),
                    "chapter" => self.chapter_start(t),
                    "verse" => self.verse_start(t, anc),
                    _ if not_imported => Ok(()),
                    _ => Err(self.err("structure", t, format!("<{name}> between verses"))),
                }
            }
        }
    }

    fn end_outside(&mut self, t: &Tok<'_>) -> Result<(), Error> {
        match t.name {
            "q" if self.q.is_some() => Err(self.err("R2", t, "</q> end tag between verses")),
            "chapter" => {
                if self.last_verse == 0 {
                    return Err(self.err("structure", t, "chapter without verses"));
                }
                self.chapter_open = false;
                Ok(())
            }
            "div" => {
                if let Some(depth) = self.colophon {
                    // The colophon div is the element just closed when the
                    // stack is back at the depth it opened at.
                    if depth == self.depth_hint {
                        self.colophon = None;
                    }
                    return Ok(());
                }
                if self.book.is_some() && self.book_div_depth == Some(self.depth_hint) {
                    let b = self.book.take().unwrap();
                    self.book_div_depth = None;
                    let book = &mut self.m.books[b];
                    book.chapters = self.chapter;
                    if self.chapter == 0 {
                        return Err(self.err("structure", t, "book without chapters"));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// `<title type="main">` directly inside the open book div, before its
    /// first chapter, at most once per book.
    fn main_title_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let Some(b) = self.book else {
            return Err(self.err("title", t, "main title outside a book"));
        };
        if t.kind != Kind::Start || !attrs_exactly(t, &[("type", Some("main"))], &[]) {
            return Err(self.bad_attrs(t));
        }
        if self.book_div_depth != Some(anc.len().saturating_sub(1))
            || anc.last() != Some(&"div")
            || self.chapter != 0
            || self.m.books[b].main_title.is_some()
        {
            return Err(self.err(
                "title",
                t,
                "main title not directly in the book div before chapter 1, or a second one",
            ));
        }
        self.main_title = Some((anc.len(), String::new()));
        Ok(())
    }

    /// Inside the main title: raw text is kept; `<abbr expansion="…">` is
    /// markup (its text is kept, the attribute is not imported); anything
    /// else fails closed.
    fn in_main_title(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let depth = self.main_title.as_ref().unwrap().0;
        match t.kind {
            Kind::Text => {
                self.main_title.as_mut().unwrap().1.push_str(t.text);
                Ok(())
            }
            Kind::Start if t.name == "abbr" && anc.len() == depth + 1 => {
                if attrs_exactly(t, &[("expansion", None)], &[]) {
                    Ok(())
                } else {
                    Err(self.bad_attrs(t))
                }
            }
            Kind::End if t.name == "abbr" && anc.len() == depth + 1 => Ok(()),
            Kind::End if t.name == "title" && anc.len() == depth => {
                let (_, text) = self.main_title.take().unwrap();
                let b = self.book.expect("main title is inside a book");
                self.m.books[b].main_title = Some(text);
                Ok(())
            }
            _ => Err(self.err(
                "title",
                t,
                format!("<{}> is not allowed in a main title", t.name),
            )),
        }
    }

    fn div_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let ty = t.attr("type").unwrap_or("");
        match ty {
            "bookGroup" if self.book.is_none() => Ok(()),
            "colophon" if self.book.is_some() && !self.chapter_open => {
                if !attrs_exactly(t, &[("type", None)], &[("osisID", None)]) {
                    return Err(self.bad_attrs(t));
                }
                self.colophon = Some(anc.len());
                Ok(())
            }
            "book" if self.book.is_none() => {
                if !attrs_exactly(
                    t,
                    &[("type", None), ("osisID", None)],
                    &[("canonical", Some("true"))],
                ) {
                    return Err(self.bad_attrs(t));
                }
                let idx = self.books_seen;
                let want = self.p.books.get(idx).map(|b| b.0);
                if t.attr("osisID") != want {
                    return Err(self.err(
                        "structure",
                        t,
                        format!(
                            "book {} out of canonical order",
                            excerpt(t.attr("osisID").unwrap_or("").as_bytes())
                        ),
                    ));
                }
                let (osis, usfm) = self.p.books[idx];
                self.m.books.push(Book {
                    ord: (idx + 1) as u8,
                    osis: osis.into(),
                    usfm: usfm.into(),
                    chapters: 0,
                    main_title: None,
                });
                self.books_seen += 1;
                self.book = Some(idx);
                self.book_div_depth = Some(anc.len());
                self.chapter = 0;
                Ok(())
            }
            _ => Err(self.err("structure", t, "unexpected <div>")),
        }
    }

    fn chapter_start(&mut self, t: &Tok<'_>) -> Result<(), Error> {
        if self.book.is_none() || self.chapter_open {
            return Err(self.err(
                "structure",
                t,
                "<chapter> outside a book or inside a chapter",
            ));
        }
        if !attrs_exactly(t, &[("osisID", None)], &[("chapterTitle", None)]) {
            return Err(self.bad_attrs(t));
        }
        let want = format!("{}.{}", self.book_osis(), self.chapter + 1);
        if t.attr("osisID") != Some(want.as_str()) {
            return Err(self.err(
                "structure",
                t,
                format!("chapter out of order, expected {want}"),
            ));
        }
        self.chapter += 1;
        self.chapter_open = true;
        self.last_verse = 0;
        Ok(())
    }

    fn verse_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        if t.attr("eID").is_some() {
            return Err(self.err("structure", t, "verse eID with no verse open"));
        }
        if !attrs_exactly(t, &[("osisID", None), ("sID", None)], &[]) {
            return Err(self.err(
                "structure",
                t,
                "verse start must have exactly osisID and sID",
            ));
        }
        let parent = anc.last().copied().unwrap_or("");
        if !self.chapter_open || !(parent == "chapter" || parent == "q") {
            return Err(self.err(
                "structure",
                t,
                "verse milestone not directly in a chapter or q",
            ));
        }
        let sid = t.attr("sID").unwrap();
        if t.attr("osisID") != Some(sid) {
            return Err(self.err("structure", t, "verse osisID differs from sID"));
        }
        let want = format!(
            "{}.{}.{}",
            self.book_osis(),
            self.chapter,
            self.last_verse + 1
        );
        if sid != want {
            return Err(self.err(
                "structure",
                t,
                format!(
                    "verse {} out of order or in the wrong chapter, expected {want}",
                    excerpt(sid.as_bytes())
                ),
            ));
        }
        if self.m.verses.len() >= MAX_VERSES {
            return Err(self.err("cap", t, "more than 40,000 verses"));
        }
        self.last_verse += 1;
        self.verse = Some(OpenVerse {
            v: Verse {
                book: (self.book.unwrap() + 1) as u8,
                chapter: self.chapter,
                verse: self.last_verse,
                osis: want,
                text: String::new(),
                len: 0,
                marks: Vec::new(),
                red: Vec::new(),
                words: Vec::new(),
                offset: t.start,
            },
            note_depth: 0,
            open_marks: Vec::new(),
            open_word: None,
        });
        if self.q.is_some() {
            self.piece_start = Some(0);
        }
        Ok(())
    }

    // ------------------------------------------------------------ inside verses

    fn in_verse(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let in_note = self.verse.as_ref().unwrap().note_depth > 0;
        if in_note {
            return self.in_note(t);
        }
        match t.kind {
            Kind::Text => {
                let ov = self.verse.as_mut().unwrap();
                ov.v.text.push_str(t.text);
                ov.v.len += t.text.chars().count() as u32;
                if ov.v.len > MAX_VERSE_CP {
                    return Err(self.err("V1", t, "verse longer than 4,096 code points"));
                }
                Ok(())
            }
            Kind::Start | Kind::Empty => self.start_in_verse(t, anc),
            Kind::End => self.end_in_verse(t),
        }
    }

    fn in_note(&mut self, t: &Tok<'_>) -> Result<(), Error> {
        let ok = match t.kind {
            Kind::Text => true,
            Kind::End => {
                if t.name == "note" {
                    self.verse.as_mut().unwrap().note_depth -= 1;
                }
                true
            }
            Kind::Start | Kind::Empty => match t.name {
                "catchWord" | "divineName" => t.attrs.is_empty(),
                "rdg" => {
                    attrs_exactly(t, &[], &[("type", None)])
                        && matches!(
                            t.attr("type"),
                            None | Some(
                                "alternate"
                                    | "x-equivalent"
                                    | "x-identity"
                                    | "x-literal"
                                    | "x-meaning"
                            )
                        )
                }
                "transChange" => {
                    attrs_exactly(t, &[("type", None)], &[])
                        && matches!(t.attr("type"), Some("added" | "amplified"))
                }
                "q" => return Err(self.err("R2", t, "<q> inside a note")),
                _ => {
                    return Err(self.err(
                        "verse-content",
                        t,
                        format!("<{}> inside a note", t.name),
                    ));
                }
            },
        };
        if ok { Ok(()) } else { Err(self.bad_attrs(t)) }
    }

    fn start_in_verse(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let empty = t.kind == Kind::Empty;
        match t.name {
            "verse" => self.verse_end(t, anc),
            "w" => self.word_start(t, empty),
            "transChange" | "divineName" => {
                let kind = if t.name == "transChange" {
                    if !attrs_exactly(t, &[("type", Some("added"))], &[]) {
                        return Err(self.bad_attrs(t));
                    }
                    crate::osis::MarkKind::Added
                } else {
                    if !t.attrs.is_empty() {
                        return Err(self.bad_attrs(t));
                    }
                    crate::osis::MarkKind::Divine
                };
                let ov = self.verse.as_mut().unwrap();
                ov.v.marks.push(Mark {
                    kind,
                    start: ov.v.len,
                    end: ov.v.len,
                });
                ov.open_marks.push(ov.v.marks.len() - 1);
                Ok(())
            }
            "inscription" => {
                if t.attrs.is_empty() {
                    Ok(())
                } else {
                    Err(self.bad_attrs(t))
                }
            }
            "milestone" => {
                if attrs_exactly(
                    t,
                    &[("type", Some("x-p")), ("marker", Some("¶"))],
                    &[("subType", Some("x-added"))],
                ) {
                    Ok(())
                } else {
                    Err(self.bad_attrs(t))
                }
            }
            "note" => {
                if !attrs_exactly(t, &[("type", Some("study"))], &[]) {
                    return Err(self.bad_attrs(t));
                }
                self.verse.as_mut().unwrap().note_depth += 1;
                Ok(())
            }
            "q" => self.q_start(t, anc),
            other => Err(self.err("verse-content", t, format!("<{other}> inside a verse"))),
        }
    }

    fn end_in_verse(&mut self, t: &Tok<'_>) -> Result<(), Error> {
        match t.name {
            "w" => {
                let ov = self.verse.as_mut().unwrap();
                let i = ov.open_word.take().expect("xmlscan matched the end tag");
                ov.v.words[i].end = ov.v.len;
                Ok(())
            }
            "transChange" | "divineName" => {
                let ov = self.verse.as_mut().unwrap();
                let i = ov.open_marks.pop().expect("xmlscan matched the end tag");
                ov.v.marks[i].end = ov.v.len;
                if ov.v.marks[i].start == ov.v.len {
                    return Err(self.err("P2", t, format!("empty <{}>", t.name)));
                }
                Ok(())
            }
            "q" => {
                let ov = self.verse.as_mut().unwrap();
                let start = self.piece_start.take().expect("q open");
                let end = ov.v.len;
                let qi = self.q.take().expect("q open");
                let q_ord = self.m.qs[qi].q_ord;
                ov.v.red.push(RedPiece { start, end, q_ord });
                self.m.qs[qi].end_byte = t.end;
                if start >= end {
                    return Err(self.err(
                        "R1",
                        t,
                        "empty red piece (q ends where it starts in this verse)",
                    ));
                }
                Ok(())
            }
            "chapter" => Err(self.err("structure", t, "verse open across </chapter>")),
            _ => Ok(()),
        }
    }

    fn q_start(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        if t.attr("sID").is_some() || t.attr("eID").is_some() {
            return Err(self.err(
                "q-form",
                t,
                "milestone q (sID/eID) is not supported by this profile",
            ));
        }
        if t.attrs.len() != 2 || t.attr("who") != Some("Jesus") || t.attr("marker").is_none() {
            return Err(self.err(
                "q-attr",
                t,
                "q must have exactly who=\"Jesus\" and marker=\"\"",
            ));
        }
        if t.attr("marker") != Some("") {
            return Err(self.err(
                "q-attr",
                t,
                "non-empty q marker (would mean generating characters)",
            ));
        }
        if self.q.is_some() {
            return Err(self.err("R2", t, "nested q"));
        }
        let ov = self.verse.as_ref().unwrap();
        if !ov.open_marks.is_empty() {
            return Err(self.err("P4", t, "q starts inside a transChange or divineName"));
        }
        if anc.last() != Some(&"chapter") {
            return Err(self.err("R2", t, "q not directly inside the chapter"));
        }
        if self.m.qs.len() >= MAX_Q {
            return Err(self.err("cap", t, "more than 10,000 q"));
        }
        let q_ord = self.m.qs.len() as u32 + 1;
        self.m.qs.push(QSpan {
            q_ord,
            book: ov.v.book,
            start_byte: t.start,
            end_byte: 0,
        });
        self.q = Some(self.m.qs.len() - 1);
        self.piece_start = Some(ov.v.len);
        Ok(())
    }

    fn verse_end(&mut self, t: &Tok<'_>, anc: &[&str]) -> Result<(), Error> {
        let ov = self.verse.as_ref().unwrap();
        if t.attr("sID").is_some() {
            return Err(self.err("structure", t, "verse opened inside another verse"));
        }
        if !attrs_exactly(t, &[("eID", None)], &[]) || t.attr("eID") != Some(ov.v.osis.as_str()) {
            return Err(self.err("structure", t, "verse eID does not close the open verse"));
        }
        let parent = anc.last().copied().unwrap_or("");
        if !(parent == "chapter" || parent == "q") {
            return Err(self.err("structure", t, "verse end milestone inside another element"));
        }
        if let Some(qi) = self.q {
            let start = self.piece_start.take().expect("q open");
            let len = ov.v.len;
            let q_ord = self.m.qs[qi].q_ord;
            let ov = self.verse.as_mut().unwrap();
            ov.v.red.push(RedPiece {
                start,
                end: len,
                q_ord,
            });
            if start >= len {
                return Err(self.err("R1", t, "empty red piece (q starts at the verse end)"));
            }
        }
        let ov = self.verse.take().unwrap();
        self.m.verses.push(ov.v);
        Ok(())
    }

    fn word_start(&mut self, t: &Tok<'_>, empty: bool) -> Result<(), Error> {
        if self.verse.as_ref().unwrap().open_word.is_some() {
            return Err(self.err("verse-content", t, "<w> inside <w>"));
        }
        if !attrs_exactly(
            t,
            &[("lemma", None)],
            &[("morph", None), ("src", None), ("type", None)],
        ) {
            return Err(self.bad_attrs(t));
        }
        if let Some(ty) = t.attr("type") {
            let ok = ty
                .strip_prefix("x-split-")
                .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
            if !ok {
                return Err(self.bad_attrs(t));
            }
        }
        let lemma = t.attr("lemma").unwrap();
        let keys = self.lemma_keys(t, lemma)?;
        let ov = self.verse.as_mut().unwrap();
        ov.v.words.push(Word {
            start: ov.v.len,
            end: ov.v.len,
            keys,
        });
        if !empty {
            ov.open_word = Some(ov.v.words.len() - 1);
        }
        Ok(())
    }

    fn lemma_keys(&mut self, t: &Tok<'_>, lemma: &str) -> Result<Vec<StrongsKey>, Error> {
        let verse = self.verse.as_ref().unwrap().v.osis.clone();
        if lemma.chars().any(|c| c.is_whitespace() && c != ' ') {
            return Err(self.err("T13", t, "lemma contains whitespace other than U+0020"));
        }
        let mut keys = Vec::new();
        for token in lemma.split(' ') {
            if token.is_empty() {
                return Err(self.err(
                    "T13",
                    t,
                    "empty lemma token (double, leading or trailing space)",
                ));
            }
            if token.starts_with("lemma.TR:") {
                continue;
            }
            if let Some(rest) = token.strip_prefix("strong:") {
                if self.p.kjv_strongs_spelling && !kjv_spelling(rest) {
                    return Err(self.err(
                        "T13",
                        t,
                        format!(
                            "Strong's token {} is not in the KJV spelling",
                            excerpt(token.as_bytes())
                        ),
                    ));
                }
                let key = StrongsKey::from_osis_lemma(token)
                    .map_err(|e| self.err("T13", t, e.to_string()))?;
                keys.push(key);
                continue;
            }
            let hit = self
                .p
                .anomalies
                .iter()
                .position(|a| a.verse == verse && a.lemma == lemma && a.skip_token == token);
            match hit {
                Some(i) => self.anomaly_hits[i] += 1,
                None => {
                    return Err(self.err(
                        "T13",
                        t,
                        format!(
                            "lemma token {} is neither strong: nor lemma.TR:",
                            excerpt(token.as_bytes())
                        ),
                    ));
                }
            }
        }
        Ok(keys)
    }
}

/// KJV spelling (DESIGN.md 6.2): Hebrew `H0` + a number with no further
/// leading zero (at most 5 digits in all); Greek `G` + a number with no
/// leading zero (at most 4 digits).
fn kjv_spelling(key: &str) -> bool {
    let digits_ok = |d: &str, max: usize| {
        !d.is_empty()
            && d.len() <= max
            && d.bytes().all(|b| b.is_ascii_digit())
            && !d.starts_with('0')
    };
    if let Some(d) = key.strip_prefix("H0") {
        digits_ok(d, 4)
    } else if let Some(d) = key.strip_prefix('G') {
        digits_ok(d, 4)
    } else {
        false
    }
}

/// Parses a verified KJV OSIS buffer with `profile`.
pub fn parse(data: &[u8], profile: &Profile) -> Result<Model, Error> {
    let mut st = State {
        p: profile,
        file: profile.xml.file,
        m: Model {
            books: Vec::new(),
            verses: Vec::new(),
            qs: Vec::new(),
        },
        book: None,
        books_seen: 0,
        chapter: 0,
        chapter_open: false,
        last_verse: 0,
        verse: None,
        q: None,
        piece_start: None,
        colophon: None,
        anomaly_hits: vec![0; profile.anomalies.len()],
        depth_hint: 0,
        book_div_depth: None,
        main_title: None,
    };
    xmlscan::scan(data, &profile.xml, |t, anc| {
        st.depth_hint = anc.len();
        st.on(t, anc)
    })?;
    let end = |msg: &str| Error::new("structure", msg.to_string()).file(profile.xml.file);
    if st.verse.is_some() {
        return Err(end("verse open at end of file"));
    }
    if st.books_seen != profile.books.len() {
        return Err(end(&format!(
            "{} books in the file, the profile expects {}",
            st.books_seen,
            profile.books.len()
        )));
    }
    for (a, hits) in profile.anomalies.iter().zip(&st.anomaly_hits) {
        if *hits != 1 {
            return Err(Error::new(
                "anomaly",
                format!("anomaly list entry for {} matched {hits} times, expected exactly 1 (stale list)", a.verse),
            )
            .file(profile.xml.file));
        }
    }
    Ok(st.m)
}
