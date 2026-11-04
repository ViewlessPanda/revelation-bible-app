"""Survey of the pinned MHCC OSIS file (CMT-MHCC-CW20) for
docs/pipeline/mhcc-osis-survey.md.

Usage:
  py -I -B survey_mhcc.py <mhcc.osis.xml> <CMT-MHCC-CW20.sha256> \
      <kjv.osis.xml> <BIB-KJV-CW31.sha256>

Prints facts only (no source text beyond element names, attribute values of
enumerations and OSIS references). Run from outside sources/raw, with paths
as arguments. Both files are read once and checked against their checksum
files before anything else (osis_tokens.load_pinned); the MHCC file must pass
the strict reader (mhcc_osis.read) and the KJV file the strict KJV walker
(osis_tokens.strict_tokenize) before any fact is printed.
"""
import collections
import os
import re
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from osis_tokens import load_pinned, strict_tokenize  # noqa: E402
import mhcc_osis  # noqa: E402


def kjv_verses(data):
    out = set()
    for t in strict_tokenize(data):
        if t.kind == "empty" and t.name == "verse" and "sID" in t.attrs:
            out.add(t.attrs["sID"])
    return out


def main(argv):
    if len(argv) != 5:
        sys.exit(__doc__)
    data = load_pinned(argv[1], argv[2])
    kjv = kjv_verses(load_pinned(argv[3], argv[4]))
    doc = mhcc_osis.read(data)
    toks = mhcc_osis.tokenize(data)
    P = print

    P("## Bytes")
    P("size", len(data), "LF", data.count(b"\n"), "CR", data.count(b"\r"), "TAB", data.count(b"\t"))
    P("non-ASCII bytes", sum(1 for b in data if b > 0x7E), "control bytes other than LF",
      sum(1 for b in data if b < 0x20 and b != 0x0A))
    P("'&amp;'", doc.amp, "other '&'", data.count(b"&") - doc.amp)
    P("'<!'", data.count(b"<!"), "comments", dict(sorted((k.decode(), v) for k, v in doc.comments.items())))
    P("'<?'", data.count(b"<?"), "']]>'", data.count(b"]]>"))
    gt = sum(data[t.start:t.end].count(b">") for t in toks if t.kind == "text")
    P("'>' in text", gt)
    P("tokens", doc.tokens, "max depth", doc.max_depth)
    P("longest text token", max(t.end - t.start for t in toks if t.kind == "text"))
    P("longest attribute value", max(len(v) for t in toks if t.attrs for _, v in t.attrs))
    P("most attributes on one tag", max(len(t.attrs) for t in toks if t.attrs is not None))
    chars = collections.Counter(data)
    P("distinct byte values", len(chars))

    P("\n## Elements (name, form): count")
    forms = collections.Counter((t.name, t.kind) for t in toks if t.kind in ("start", "empty"))
    for k, v in sorted(forms.items()):
        P(" ", k, v)
    P("\n## Attributes (element@name): count")
    attrs = collections.Counter("%s@%s" % (t.name, k) for t in toks if t.kind in ("start", "empty") for k, _ in t.attrs)
    for k, v in sorted(attrs.items()):
        P(" ", k, v)
    P("\n## Enumerated values")
    for el, at in (("div", "type"), ("title", "type"), ("p", "subType"), ("hi", "type"), ("milestone", "type")):
        c = collections.Counter(dict(t.attrs)[at] for t in toks if t.kind in ("start", "empty") and t.name == el and at in dict(t.attrs))
        P(" ", "%s@%s" % (el, at), dict(sorted(c.items())))

    P("\n## Books and chapters")
    P("books", len(doc.books), "chapters", doc.chapters, "chapter milestones directly in a book div",
      doc.top_chapter_milestones)

    P("\n## Blocks (p, title, table inside books)")
    P("blocks", len(doc.blocks))
    kinds = collections.Counter((b["name"], tuple(b["attrs"])) for b in doc.blocks)
    for k, v in sorted(kinds.items()):
        P(" ", k, v)
    inside = sum(1 for b in doc.blocks if b["anchor"] is not None)
    P("inside an anchor", inside, "outside", len(doc.blocks) - inside)
    P("before the book's first chapter milestone (chapter None)", sum(1 for b in doc.blocks if b["chapter"] is None))
    P("whitespace-only blocks", sum(1 for b in doc.blocks if not b["text"].strip(b" \n")))
    P("largest block (bytes)", max(len(b["raw"]) for b in doc.blocks))
    P("total raw block bytes", sum(len(b["raw"]) for b in doc.blocks),
      "total text bytes", sum(len(b["text"]) for b in doc.blocks))
    outside = collections.Counter((b["name"], tuple(b["attrs"])) for b in doc.blocks
                                  if b["anchor"] is None and b["text"].strip(b" \n"))
    P("non-empty blocks outside anchors, by kind:", dict(sorted(outside.items())))

    P("\n## Anchors (verse sID/eID milestone pairs)")
    P("anchors", len(doc.anchors))
    ids = [v for a in doc.anchors for v in a["osisID"].split(" ")]
    cnt = collections.Counter(ids)
    P("verse references listed", len(ids), "distinct verses", len(cnt),
      "verses listed by more than one anchor", sum(1 for v in cnt.values() if v > 1),
      "max anchors on one verse", max(cnt.values()))
    P("single-verse anchors", sum(1 for a in doc.anchors if " " not in a["osisID"]),
      "largest anchor (verses)", max(len(a["osisID"].split(" ")) for a in doc.anchors))
    span = collections.Counter(a["last"] - a["first"] + 1 for a in doc.anchors)
    P("blocks per anchor", dict(sorted(span.items())))
    per_book = collections.Counter(a["sid"].split(".")[0] for a in doc.anchors)
    P("anchors in Ps", per_book["Ps"], "books without anchors", [b for b in mhcc_osis.BOOKS if per_book[b] == 0])
    # Chapters whose only anchor is a single-verse anchor on verse 1.
    by_ch = collections.defaultdict(list)
    for a in doc.anchors:
        by_ch[a["sid"].rsplit(".", 1)[0]].append(a["sid"])
    only_v1 = sorted(c for c, l in by_ch.items() if l == [c + ".1"])
    P("chapters whose only anchor is <verse sID=Book.C.1> alone", len(only_v1),
      "of which Ps", sum(1 for c in only_v1 if c.startswith("Ps.")))
    chs = ["%s.%d" % (b["book"], b["chapter"]) for b in doc.blocks if b["chapter"] is not None]
    P("chapters without any anchor", sorted(set(chs) - set(by_ch)))
    # Chapter outlines inside a verse-1 anchor: an anchor Book.C.1 directly
    # followed by another anchor that also lists Book.C.1.
    dup_v1 = 0
    for x, y in zip(doc.anchors, doc.anchors[1:]):
        c1 = x["sid"]
        if c1.endswith(".1") and c1 in y["osisID"].split(" "):
            dup_v1 += 1
    P("single verse-1 anchors directly followed by an anchor that also lists that verse", dup_v1)
    multi = sorted(v for v, n in cnt.items() if n > 1)
    P("first verses listed twice:", multi[:5])
    unresolved = sorted(set(ids) - kjv)
    P("anchor verses that are not KJV verses (kjv.osis.xml sIDs)", len(unresolved), unresolved[:20])
    P("KJV verses not listed by any anchor", len(kjv - set(ids)))

    P("\n## Cross-references")
    P("references", len(doc.refs))
    forms = collections.Counter()
    bad = []
    cross_ch = 0
    desc = []
    for value, at, _ in doc.refs:
        a, b = mhcc_osis.parse_ref(value)
        forms["range" if "-" in value else "single"] += 1
        if a[1] != b[1]:
            cross_ch += 1
        if (b[1], b[2]) < (a[1], a[2]):
            desc.append(value)
        for end in {a, b}:
            if "%s.%d.%d" % end not in kjv:
                bad.append(value)
    P("forms", dict(forms), "ranges across chapters", cross_ch)
    P("descending ranges (stored as written)", len(desc), desc)
    P("references with an end that is not a KJV verse", len(bad), sorted(set(bad))[:20])
    P("\nstrict reader: passed")


if __name__ == "__main__":
    main(sys.argv)
