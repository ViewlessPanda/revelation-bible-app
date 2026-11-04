"""Strict reader for the CrossWire MHCC 2.0 OSIS file (CMT-MHCC-CW20), shared
by survey_mhcc.py and oracle_mhcc.py (DESIGN.md 17, docs/pipeline/mhcc-osis-survey.md).

No XML parser, no DTD, no entity mechanism: a regex tokenizer over the raw
bytes and an allowlist walker, written independently of the Rust pipeline.
Everything raises Refused on anything it does not recognise.

read(data) returns a Doc with, in document order:
- blocks: every <p>, <title> and <table> inside a book (they never nest), with
  book, chapter open at the block's start (None before the book's first
  chapter milestone), the 1-based anchor it belongs to (or None), its raw
  bytes [start, end) and its text: the character data with every tag
  stripped and the one entity '&amp;' decoded to '&';
- anchors: every <verse sID .../> ... <verse eID .../> pair: sID, the osisID
  list exactly as written, first and last block;
- refs: every <reference osisRef="...">.
"""
import re

from osis_tokens import Refused

DECL = b'<?xml version="1.0" encoding="utf-8"?>'

# The only comments in the file: the converter's USFM marker names. Each must
# be directly followed by the <title> of the matching type, and every such
# title must be directly preceded by its comment. Not imported.
COMMENTS = {
    b"<!-- s2 -->": "x-s2",
    b"<!-- is -->": "x-IS",
    b"<!-- ms -->": "x-ms",
    b"<!-- mt -->": "main",
}

ROOT_ATTRS = [
    ("xmlns", "http://www.bibletechnologies.net/2003/OSIS/namespace"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("xsi:schemaLocation", "http://www.bibletechnologies.net/2003/OSIS/namespace\n"
                           "        http://www.bibletechnologies.net/osisCore.2.1.1.xsd"),
]
OSISTEXT_ATTRS = [("osisIDWork", "MHCC"), ("osisRefWork", "Bible"), ("xml:lang", "en")]

BOOKS = (
    "Gen Exod Lev Num Deut Josh Judg Ruth 1Sam 2Sam 1Kgs 2Kgs 1Chr 2Chr Ezra Neh "
    "Esth Job Ps Prov Eccl Song Isa Jer Lam Ezek Dan Hos Joel Amos Obad Jonah Mic "
    "Nah Hab Zeph Hag Zech Mal Matt Mark Luke John Acts Rom 1Cor 2Cor Gal Eph Phil "
    "Col 1Thess 2Thess 1Tim 2Tim Titus Phlm Heb Jas 1Pet 2Pet 1John 2John 3John "
    "Jude Rev"
).split()
assert len(BOOKS) == 66

TAG = re.compile(rb'<(/?)([A-Za-z][A-Za-z0-9:]*)((?:\s+[A-Za-z][A-Za-z0-9:.-]*="[^"<&]*")*)\s*(/?)>')
ATTR = re.compile(rb'([A-Za-z][A-Za-z0-9:.-]*)="([^"<&]*)"')
WS = frozenset(b" \n")
NUM = r"[1-9][0-9]{0,2}"

S, E = "start", "empty"
# context -> (kinds, parent contexts, attribute rule). Contexts: element name,
# except div:book / div:introduction and the header's p:h / title:h.
# Attribute rule: a list of allowed exact attribute-name tuples (in order).
RULES = {
    "header": (S, {"osisText"}, [()]),
    "revisionDesc": (S, {"header"}, [("resp",)]),
    "date": (S, {"revisionDesc"}, [()]),
    "p:h": (S, {"revisionDesc"}, [()]),
    "work": (S, {"header"}, [("osisWork",)]),
    "title:h": (S, {"work"}, [()]),
    "description": (S, {"work"}, [("type", "subType")]),
    "type": (S, {"work"}, [("type",)]),
    "identifier": (S, {"work"}, [("type",)]),
    "refSystem": (S, {"work"}, [()]),
    "div:book": (S, {"osisText"}, [("type", "osisID", "canonical")]),
    "div:introduction": (S, {"div:book"}, [("type",)]),
    "milestone": (E, {"div:book"}, [("type", "n")]),
    "chapter": (E, {"div:book", "p"}, [("sID", "osisID", "n"), ("eID",)]),
    "title": (S, {"div:book", "div:introduction"}, [("type",)]),
    "p": (S, {"div:book", "div:introduction"}, [(), ("subType",)]),
    "table": (S, {"div:book", "div:introduction"}, [()]),
    "row": (S, {"table"}, [()]),
    "cell": (S, {"row"}, [()]),
    "hi": (S, {"p"}, [("type",)]),
    "reference": (S, {"p"}, [("osisRef",)]),
    "verse": (E, {"p"}, [("sID", "osisID", "n"), ("eID",)]),
}
VALUES = {
    ("milestone", "type"): {"x-usfm-toc1", "x-usfm-toc2", "x-usfm-toc3"},
    ("title", "type"): {"x-s2", "x-IS", "x-ms", "main"},
    ("p", "subType"): {"x-introduction"},
    ("hi", "type"): {"bold"},
    ("div:introduction", "type"): {"introduction"},
}
BLOCKS = {"p", "title", "table"}
# Contexts whose text must be whitespace (space or LF) only.
WS_ONLY = {"osis", "osisText", "header", "revisionDesc", "work", "div:book",
           "div:introduction", "table", "row"}


class Tok:
    __slots__ = ("kind", "name", "attrs", "start", "end")

    def __init__(self, kind, name, attrs, start, end):
        self.kind, self.name, self.attrs, self.start, self.end = kind, name, attrs, start, end


def serialise(name, attrs, kind):
    if kind == "end":
        return b"</" + name.encode() + b">"
    body = b"".join(b" " + k.encode() + b'="' + v.encode() + b'"' for k, v in attrs)
    return b"<" + name.encode() + body + (b" />" if kind == "empty" else b">")


def tokenize(data):
    if not data.startswith(DECL + b"\n"):
        raise Refused("file does not start with the pinned declaration and LF")
    for i, b in enumerate(data):
        if b != 0x0A and not 0x20 <= b <= 0x7E:
            raise Refused("byte 0x%02x at %d: only printable ASCII and LF are allowed" % (b, i))
    toks = []
    pos = len(DECL)
    n = len(data)
    while pos < n:
        lt = data.find(b"<", pos)
        if lt == -1:
            lt = n
        if lt > pos:
            text = data[pos:lt]
            if b"&" in text.replace(b"&amp;", b""):
                raise Refused("'&' other than '&amp;' at byte %d" % (pos + text.replace(b"&amp;", b"\0\0\0\0\0").find(b"&")))
            if b"]]>" in text:
                raise Refused("']]>' in text at %d" % pos)
            toks.append(Tok("text", None, None, pos, lt))
        if lt == n:
            break
        if data.startswith(b"<!", lt):
            for c in COMMENTS:
                if data.startswith(c, lt):
                    toks.append(Tok("comment", c, None, lt, lt + len(c)))
                    pos = lt + len(c)
                    break
            else:
                raise Refused("'<!' at byte %d is not one of the four pinned comments" % lt)
            continue
        if data.startswith(b"<?", lt):
            raise Refused("processing instruction or second declaration at %d" % lt)
        m = TAG.match(data, lt)
        if not m:
            raise Refused("unparseable tag at byte %d" % lt)
        slash, name, attrs, empty = m.groups()
        if slash and (attrs.strip() or empty):
            raise Refused("bad end tag at %d" % lt)
        al = []
        for am in ATTR.finditer(attrs):
            k = am.group(1).decode("ascii")
            if any(k == x for x, _ in al):
                raise Refused("duplicate attribute at %d" % lt)
            al.append((k, am.group(2).decode("ascii")))
        if len(al) > 16:
            raise Refused("more than 16 attributes at %d" % lt)
        kind = "end" if slash else ("empty" if empty else "start")
        toks.append(Tok(kind, name.decode("ascii"), al, lt, m.end()))
        pos = m.end()
    return toks


class Doc:
    def __init__(self):
        self.blocks = []
        self.anchors = []
        self.refs = []
        self.comments = {}
        self.chapters = 0
        self.books = []
        self.amp = 0
        self.top_chapter_milestones = 0
        self.tokens = 0
        self.max_depth = 0


def _context(name, attrs, parent):
    if name == "div":
        t = dict(attrs).get("type")
        if t not in ("book", "introduction"):
            raise Refused("div with type %r" % t)
        return "div:" + t
    if parent in ("revisionDesc",) and name == "p":
        return "p:h"
    if parent == "work" and name == "title":
        return "title:h"
    return name


def _verse_ids(sid, book, chapter, at):
    m = re.fullmatch(r"([1-3]?[A-Za-z]+)\.(%s)\.(%s)(?:-(%s))?" % (NUM, NUM, NUM), sid)
    if not m:
        raise Refused("verse sID %r at %d is not Book.C.V or Book.C.V-V" % (sid, at))
    b, c, v1 = m.group(1), int(m.group(2)), int(m.group(3))
    v2 = int(m.group(4)) if m.group(4) else v1
    if b != book or c != chapter:
        raise Refused("verse %s at %d is not in the open book/chapter %s.%s" % (sid, at, book, chapter))
    if m.group(4) and v2 <= v1:
        raise Refused("verse range %s at %d does not ascend" % (sid, at))
    n = str(v1) if not m.group(4) else "%d-%d" % (v1, v2)
    return ["%s.%d.%d" % (b, c, v) for v in range(v1, v2 + 1)], n


def read(data, fixture=False):
    """fixture=True (synthetic test files only): fewer than 66 books are
    allowed, still in canonical order, as the Rust fixture profile does."""
    toks = tokenize(data)
    doc = Doc()
    doc.tokens = len(toks)
    doc.amp = data.count(b"&amp;")
    stack = []  # contexts
    root_done = False
    book = None
    chapter = None  # open chapter number
    last_chapter = 0
    block = None  # dict of the open block
    block_depth = None
    anchor = None  # open anchor dict
    pending_comment = None
    header_seen = False
    block_tok = 0
    for i, t in enumerate(toks):
        parent = stack[-1] if stack else None
        if pending_comment is not None:
            if not (t.kind == "start" and t.name == "title"
                    and t.attrs == [("type", COMMENTS[pending_comment])]):
                raise Refused("comment at %d is not directly followed by its title" % toks[i - 1].start)
        if t.kind == "comment":
            if parent not in ("div:book", "div:introduction"):
                raise Refused("comment at %d outside a book" % t.start)
            doc.comments[t.name] = doc.comments.get(t.name, 0) + 1
            pending_comment = t.name
            continue
        if t.kind == "start" and t.name == "title" and parent in ("div:book", "div:introduction"):
            if pending_comment is None or COMMENTS[pending_comment] != dict(t.attrs).get("type"):
                raise Refused("title at %d is not directly preceded by its comment" % t.start)
        pending_comment = None

        if t.kind == "text":
            raw = data[t.start:t.end]
            if not stack:
                if raw.strip(b" \n"):
                    raise Refused("text outside the root at %d" % t.start)
                continue
            if parent in WS_ONLY and any(b not in WS for b in raw):
                raise Refused("non-whitespace text in <%s> at %d" % (parent, t.start))
            if parent == "p" and anchor is not None and anchor.get("closed_in") is block:
                if raw.strip(b" \n"):
                    raise Refused("text after verse eID %s at %d" % (anchor["sid"], t.start))
            if block is not None:
                block["text"].append(raw.replace(b"&amp;", b"&"))
            continue

        if t.kind == "end":
            if not stack:
                raise Refused("end tag with nothing open at %d" % t.start)
            top = stack.pop()
            if top.split(":")[0] != t.name:
                raise Refused("end tag </%s> at %d does not match <%s>" % (t.name, t.start, top))
            if data[t.start:t.end] != serialise(t.name, [], "end"):
                raise Refused("end tag at %d not in canonical form" % t.start)
            if block is not None and len(stack) == block_depth:
                block["end"] = t.end
                if anchor is not None and anchor.get("closed_in") is block:
                    anchor = None
                block = None
            if top == "div:book":
                if anchor is not None or chapter is not None:
                    raise Refused("book %s closes with an open verse or chapter" % book)
                book = None
            if not stack:
                root_done = True
            continue

        # start or empty
        if root_done:
            raise Refused("second root element at %d" % t.start)
        ctx = _context(t.name, t.attrs, parent) if stack or t.name != "osis" else "osis"
        if ctx == "osis":
            if stack or t.attrs != ROOT_ATTRS or t.kind != "start":
                raise Refused("root <osis> differs from the pinned one")
        elif ctx == "osisText":
            if parent != "osis" or t.attrs != OSISTEXT_ATTRS or t.kind != "start":
                raise Refused("<osisText> differs from the pinned one")
        else:
            rule = RULES.get(ctx)
            if rule is None:
                raise Refused("element <%s> at %d is not in the profile" % (t.name, t.start))
            kinds, parents, attr_rules = rule
            if t.kind != kinds:
                raise Refused("<%s> at %d as %s token is not allowed" % (t.name, t.start, t.kind))
            if parent not in parents:
                raise Refused("<%s> at %d is not allowed in <%s>" % (t.name, t.start, parent))
            if tuple(k for k, _ in t.attrs) not in attr_rules:
                raise Refused("<%s> at %d has attributes %r" % (t.name, t.start, [k for k, _ in t.attrs]))
            for k, v in t.attrs:
                allowed = VALUES.get((ctx, k))
                if allowed is not None and v not in allowed:
                    raise Refused("<%s %s=%r> at %d" % (t.name, k, v, t.start))
            if book is not None or ctx == "div:book":
                if data[t.start:t.end] != serialise(t.name, t.attrs, t.kind):
                    raise Refused("tag at %d not in canonical form" % t.start)
        a = dict(t.attrs)

        if ctx == "header":
            if doc.books or header_seen:
                raise Refused("header not once, before the books")
            header_seen = True
        if ctx == "div:book":
            want = BOOKS[len(doc.books)] if len(doc.books) < 66 else None
            if fixture and a["osisID"] in BOOKS:
                later = BOOKS[BOOKS.index(doc.books[-1]) + 1:] if doc.books else BOOKS
                want = a["osisID"] if a["osisID"] in later else want
            if a["osisID"] != want or a["canonical"] != "true" or a["type"] != "book":
                raise Refused("book %r at %d is not the next canonical book %r" % (a["osisID"], t.start, want))
            doc.books.append(want)
            book = want
            last_chapter = 0

        if ctx in BLOCKS:
            if block is not None:
                raise Refused("nested block <%s> at %d" % (t.name, t.start))
            block = {
                "ord": len(doc.blocks) + 1, "book": book, "chapter": chapter, "name": t.name,
                "attrs": t.attrs, "start": t.start, "end": None, "text": [], "anchor": None,
            }
            if anchor is not None:
                block["anchor"] = anchor["ord"]
                anchor["last"] = block["ord"]
            doc.blocks.append(block)
            block_depth = len(stack)

        if ctx == "chapter":
            if parent == "div:book":
                doc.top_chapter_milestones += 1
            if "sID" in a:
                c = last_chapter + 1
                if chapter is not None or a["sID"] != "%s.%d" % (book, c) or a["osisID"] != a["sID"] or a["n"] != str(c):
                    raise Refused("chapter sID %r at %d out of order" % (a["sID"], t.start))
                chapter = last_chapter = c
                doc.chapters += 1
            else:
                if chapter is None or a["eID"] != "%s.%d" % (book, chapter):
                    raise Refused("chapter eID %r at %d does not close the open chapter" % (a["eID"], t.start))
                if anchor is not None and anchor.get("closed_in") is None:
                    raise Refused("chapter %s closes inside verse %s" % (a["eID"], anchor["sid"]))
                chapter = None

        if ctx == "verse":
            if "sID" in a:
                if anchor is not None:
                    raise Refused("verse %s opened inside %s at %d" % (a["sID"], anchor["sid"], t.start))
                before = toks[block_tok:i]
                if any(x.kind != "text" or data[x.start:x.end].strip(b" \n") for x in before):
                    raise Refused("verse sID %s at %d is not first in its block" % (a["sID"], t.start))
                if chapter is None:
                    raise Refused("verse %s outside a chapter" % a["sID"])
                ids, n = _verse_ids(a["sID"], book, chapter, t.start)
                if a["osisID"] != " ".join(ids) or a["n"] != n:
                    raise Refused("verse %s at %d: osisID or n does not list exactly its range" % (a["sID"], t.start))
                anchor = {"ord": len(doc.anchors) + 1, "sid": a["sID"], "osisID": a["osisID"],
                          "first": block["ord"], "last": block["ord"], "closed_in": None,
                          "chapter": chapter}
                block["anchor"] = anchor["ord"]
                doc.anchors.append(anchor)
            else:
                if anchor is None or a["eID"] != anchor["sid"] or anchor.get("closed_in") is not None:
                    raise Refused("verse eID %r at %d does not close the open verse" % (a["eID"], t.start))
                if chapter != anchor["chapter"]:
                    raise Refused("verse %s closes outside its chapter" % a["eID"])
                anchor["closed_in"] = block
        elif block is not None and anchor is not None and anchor.get("closed_in") is block and ctx != "chapter":
            raise Refused("<%s> after verse eID %s at %d" % (t.name, anchor["sid"], t.start))

        if ctx == "reference":
            doc.refs.append((a["osisRef"], t.start, block["ord"]))

        if ctx in BLOCKS:
            block_tok = i + 1

        if t.kind == "start":
            stack.append(ctx)
            doc.max_depth = max(doc.max_depth, len(stack))
            if len(stack) > 16:
                raise Refused("nesting deeper than 16 at %d" % t.start)
    if stack or not root_done:
        raise Refused("unclosed at EOF: %s" % stack)
    if pending_comment is not None:
        raise Refused("comment at EOF")
    if len(doc.books) != 66 and not fixture:
        raise Refused("%d books, not 66" % len(doc.books))
    for bl in doc.blocks:
        bl["raw"] = data[bl["start"]:bl["end"]]
        bl["text"] = b"".join(bl["text"])
    for an in doc.anchors:
        del an["closed_in"]
    return doc


REF = re.compile(r"([1-3]?[A-Za-z]+)\.(%s)\.(%s)(?:-([1-3]?[A-Za-z]+)\.(%s)\.(%s))?\Z" % (NUM, NUM, NUM, NUM))


def parse_ref(value):
    """osisRef -> ((book, c, v), (book, c, v)); a single verse gives the same
    start and end. Refused unless both ends are well-formed and in one book.
    The order of the two ends is not checked here (the survey reports
    descending ranges; they are stored as written)."""
    m = REF.match(value)
    if not m:
        raise Refused("osisRef %r is not Book.C.V or Book.C.V-Book.C.V" % value)
    a = (m.group(1), int(m.group(2)), int(m.group(3)))
    b = (m.group(4), int(m.group(5)), int(m.group(6))) if m.group(4) else a
    if a[0] not in BOOKS or b[0] != a[0]:
        raise Refused("osisRef %r: unknown book or two books" % value)
    return a, b
