"""Survey of the two Strong's lexicon files, for docs/pipeline/kjv-osis-survey.md.

Usage:
  py -I -B survey_lexicons.py <HebrewStrong.xml> <LEX-HEB-OSHB.sha256> <strongsgreek.xml> <LEX-GRC-MGNT19.sha256> <out.txt>

Read-only. No XML parser, no DTD processing, no entity expansion. A strict regex
tokenizer. The Greek file's internal DTD subset is located by bytes, checked
to declare no entities and no default or fixed attribute values, and skipped;
nothing in it is interpreted. Refuses to run on a SHA-256 mismatch
(osis_tokens.load_pinned). Writes <out.txt> only after both files pass.

The KJV strict walker (osis_tokens.StrictWalker) does not apply to the
lexicons: they have their own element sets. This script checks end-tag names
and unclosed elements itself; it does not yet allowlist empty elements per
lexicon (no lexicon profile exists before the Phase B lexicon importer).
"""
import os
import re
import sys
from collections import Counter

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned  # noqa: E402

TAG = re.compile(
    rb"<(/?)([A-Za-z_][\w.:-]*)((?:\s+[A-Za-z_][\w.:-]*=(?:\"[^\"<&]*\"|'[^'<&]*'))*)\s*(/?)>"
)
ATTR = re.compile(rb"([A-Za-z_][\w.:-]*)=(?:\"([^\"<&]*)\"|'([^'<&]*)')")


def tokens(data, pos):
    """Yield (kind, name, attrs, start, end) from pos. Raises on anything unknown."""
    n = len(data)
    if b"&" in data:
        raise ValueError("entity or character reference present")
    while pos < n:
        lt = data.find(b"<", pos)
        if lt == -1:
            lt = n
        if lt > pos:
            yield ("text", None, None, pos, lt)
        if lt == n:
            return
        if data.startswith(b"<!--", lt):
            raise ValueError("comment outside the DTD at %d" % lt)
        m = TAG.match(data, lt)
        if not m:
            raise ValueError("unparseable markup at byte %d: %r" % (lt, data[lt:lt + 60]))
        slash, name, attrs, empty = m.groups()
        ad = {}
        for am in ATTR.finditer(attrs):
            k = am.group(1).decode()
            if k in ad:
                raise ValueError("duplicate attribute at %d" % lt)
            v = am.group(2) if am.group(2) is not None else am.group(3)
            ad[k] = v.decode("utf-8")
        kind = "end" if slash else ("empty" if empty else "start")
        yield (kind, name.decode(), ad, lt, m.end())
        pos = m.end()


def survey(p, label, data, skip_dtd):
    p("\n== %s ==" % label)
    decl_end = data.find(b"?>") + 2
    p("XML declaration:", data[:decl_end].decode())
    pos = decl_end
    if skip_dtd:
        a = data.find(b"<!DOCTYPE", pos)
        b = data.find(b"]>", a)
        subset = data[a:b + 2]
        p("internal DTD subset: bytes %d..%d (%d bytes), %d lines" % (a, b + 2, len(subset), subset.count(b"\n") + 1))
        for kw in (b"<!ENTITY", b"#FIXED", b"SYSTEM", b"PUBLIC", b"%"):
            p("   %s in subset: %d" % (kw.decode(), subset.count(kw)))
        defaults = re.findall(rb"(?:CDATA|\))\s+(\"[^\"]*\"|'[^']*')", subset)
        p("   ATTLIST default values:", defaults)
        p("   ATTLIST declarations:", [x.decode() for x in re.findall(rb"<!ATTLIST\s+(\w+)", subset)])
        p("   comments in subset:", subset.count(b"<!--"))
        if data.count(b"<!") != subset.count(b"<!"):
            p("   MARKUP DECLARATIONS OUTSIDE THE SUBSET:", data.count(b"<!") - subset.count(b"<!"))
        pos = b + 2
    elif b"<!" in data:
        p("UNEXPECTED <! markup")
    names = Counter()
    attrs = Counter()
    stack = []
    paths = Counter()
    depth = 0
    entries = []
    ent_children = Counter()
    text_in = Counter()
    ws_between = Counter()
    for kind, name, ad, s, e in tokens(data, pos):
        if kind == "text":
            raw = data[s:e]
            if raw.strip():
                text_in[stack[-1] if stack else None] += 1
            else:
                ws_between[repr(raw[:6].decode())] += 1
            continue
        if kind in ("start", "empty"):
            names[name] += 1
            for k, v in ad.items():
                if k in ("id", "strongs", "src", "unicode", "BETA", "translit", "xlit", "pron"):
                    attrs["%s@%s" % (name, k)] += 1
                else:
                    attrs["%s@%s=%s" % (name, k, v)] += 1
            paths["/".join(stack[1:] + [name])] += 1
            if name == "entry":
                entries.append(ad.get("id") or ad.get("strongs"))
            if len(stack) >= 2 and stack[-1] == "entry":
                ent_children[name] += 1
        if kind == "start":
            stack.append(name)
            depth = max(depth, len(stack))
        elif kind == "end":
            if not stack or stack[-1] != name:
                raise ValueError("mismatched end tag %s at %d" % (name, s))
            stack.pop()
    if stack:
        raise ValueError("unclosed at EOF: %s" % stack)
    p("well-nested: yes; max depth:", depth)
    p("elements:", dict(names.most_common()))
    p("attributes (value shown for enumerations):")
    for k, n in sorted(attrs.items()):
        if n > 0:
            p("   %7d %s" % (n, k))
    p("element paths below the root:")
    for k, n in paths.most_common(40):
        p("   %7d %s" % (n, k))
    p("non-whitespace text by parent element:", dict(text_in.most_common()))
    p("entries:", len(entries), " first:", entries[:3], " last:", entries[-3:])
    p("duplicate entry keys:", [k for k, n in Counter(entries).items() if n > 1][:10])
    nums = [int(re.sub(r"^[A-Z]", "", k)) for k in entries]
    p("entry keys ascending:", nums == sorted(nums), " min/max:", min(nums), max(nums))
    missing = sorted(set(range(1, max(nums) + 1)) - set(nums))
    p("numbers missing in 1..max: %d" % len(missing), missing[:30], "..." if len(missing) > 30 else "")
    shapes = Counter(re.sub(r"\d", "9", k) for k in entries)
    p("entry key shapes:", dict(shapes))
    return names


def main(heb, heb_sum, grc, grc_sum, out_path):
    if os.path.basename(heb) != "HebrewStrong.xml" or os.path.basename(grc) != "strongsgreek.xml":
        sys.exit("expected HebrewStrong.xml and strongsgreek.xml; refusing")
    hd = load_pinned(heb, heb_sum)
    gd = load_pinned(grc, grc_sum)
    lines = []  # the report is written only after everything succeeds

    def p(*a):
        lines.append(" ".join(str(x) for x in a))

    p("both SHA-256 verified")
    survey(p, "HebrewStrong.xml (LEX-HEB-OSHB)", hd, skip_dtd=False)
    # Hebrew cross-references and samples
    srcs = Counter(re.sub(rb"\d", b"9", m).decode() for m in re.findall(rb'<w src="([^"]*)"', hd))
    p("Hebrew w@src shapes:", dict(srcs))
    notes = re.findall(rb"<note>(.*?)</note>", hd, re.S)
    p("Hebrew note samples:", [x.decode()[:120] for x in notes[:3]])
    m = re.search(rb'<entry id="H[0-9]+">\s*(?:(?!</entry>).)*<source/>(?:(?!</entry>).)*</entry>', hd, re.S)
    p("entry with empty <source/>:", m.group(0).decode()[:400] if m else None)
    no_usage = [e.decode() for e in re.findall(rb'<entry id="(H\d+)">(?:(?!</entry>).)*</entry>', hd, re.S)]
    blocks = re.findall(rb'<entry id="(H\d+)">((?:(?!</entry>).)*)</entry>', hd, re.S)
    p("Hebrew entries without <usage>:", [k.decode() for k, b in blocks if b"<usage>" not in b])
    p("Hebrew entries without <meaning>:", sum(1 for k, b in blocks if b"<meaning>" not in b))
    p("Hebrew entry head-w xml:lang values:", dict(Counter(re.findall(rb'<w [^>]*xml:lang="([^"]*)"', hd)).most_common()))
    del no_usage

    survey(p, "strongsgreek.xml (LEX-GRC-MGNT19)", gd, skip_dtd=True)
    refs = Counter()
    for lang, num in re.findall(rb'<strongsref language="([A-Z]+)" strongs="([^"]*)"', gd):
        refs["%s %s" % (lang.decode(), re.sub(r"\d", "9", num.decode()))] += 1
    p("Greek strongsref (language, digit shape):", dict(refs.most_common()))
    gblocks = re.findall(rb'<entry strongs="(\d+)">((?:(?!</entry>).)*)</entry>', gd, re.S)
    mism = [k.decode() for k, b in gblocks
            if not re.search(rb"<strongs>0*" + k.lstrip(b"0") + rb"</strongs>", b)]
    p("Greek entries whose <strongs> text differs from @strongs:", mism[:20], len(mism))
    p("Greek entries without <greek>:", [k.decode() for k, b in gblocks if b"<greek " not in b][:20])
    p("Greek entries without kjv_def:", len([1 for k, b in gblocks if b"<kjv_def>" not in b]))
    loose = sum(1 for k, b in gblocks
                if re.sub(rb"<(strongs_def|strongs_derivation|kjv_def|strongs|latin)>.*?</\1>|<[^>]*/>|\s", b"", b, flags=re.S))
    p("Greek entries with non-whitespace text directly inside <entry> (outside child elements):", loose)
    lf_in = sum(1 for k, b in gblocks if re.search(rb"<strongs_def>[^<]*\n", b))
    p("Greek entries whose strongs_def text contains a line feed:", lf_in)
    with open(out_path, "w", encoding="utf-8", newline="\n") as out:
        out.write("".join(line + "\n" for line in lines))


if __name__ == "__main__":
    main(*sys.argv[1:6])
