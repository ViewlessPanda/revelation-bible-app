"""Independent oracle file for the two Strong's lexicons (DESIGN.md 11.1, M1-F4).

Usage:
  py -I -B oracle_lexicons.py <HebrewStrong.xml> <LEX-HEB-OSHB.sha256> \
      <strongsgreek.xml> <LEX-GRC-MGNT19.sha256> <out_dir>

Writes <out_dir>/lexicon-entries.tsv, and nothing else. Run from outside
sources/raw, with paths as arguments. Shares no code with the Rust pipeline
(no XML parser at all: a strict regex tokenizer over the raw bytes).

Fail-closed, in this order, before anything is computed:
- each file is read once and checked against its checksum file
  (osis_tokens.load_pinned / check_sha);
- the declaration is the pinned one; for MGNT one LF, then the internal DTD
  subset at bytes 56..1090 must hash to the pinned SHA-256 (it is skipped,
  never interpreted); no other '<!', no '<?', no '&', no ']]>' anywhere in
  the parsed part;
- the strict walker: every element name, token kind (start/end/empty) and
  parent is allowlisted per lexicon, every attribute name is allowlisted per
  element, end-tag names must match the open element, one root, no
  non-whitespace text outside entries (except the MGNT prologue, which is
  not imported), entry keys in the pinned spelling, unique and ascending;
- the pinned entry counts (8,674 and 5,624).
The file is written only after both lexicons passed.

One line per entry, OSHB first, both in file order:
- key: canonical (letter + number, no leading zeros);
- text_sha256 / text_bytes: the raw character data between <entry ...> and
  </entry>, every tag stripped, CR bytes kept (DESIGN.md 11.1);
- attr_count / attr_sha256: 'element@name=value' for the entry's own
  attribute and every attribute inside it, document order, LF-joined;
- tree_sha256: the tokens between the entry tags re-serialised without
  attributes: start '<x>', end '</x>', empty '<x></x>', text raw. It pins
  names, nesting and where every text node sits.
"""
import hashlib
import os
import re
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from osis_tokens import Refused, load_pinned  # noqa: E402

OSHB_DECL = b'<?xml version="1.0" encoding="utf-8"?>'
MGNT_DECL = b"<?xml version='1.0' encoding='utf-8' standalone='yes'?>"
MGNT_SUBSET = (56, 1090, "9b89a2867fda6f8bfcda951f9bad1ccfc5b06416eb6d38f84f8177078e9441f6")

TAG = re.compile(rb'<(/?)([A-Za-z_][\w.:-]*)((?:\s+[A-Za-z_][\w.:-]*="[^"<&]*")*)\s*(/?)>')
ATTR = re.compile(rb'([A-Za-z_][\w.:-]*)="([^"<&]*)"')
WS = frozenset(b" \t\r\n")

S, E, M = "start", "empty", "both"
# name: (kinds, parents, attributes). Parents: element names; None = root.
OSHB = {
    "lexicon": (S, {None}, {"xmlns", "xmlns:xsi", "xsi:schemaLocation"}),
    "entry": (S, {"lexicon"}, {"id"}),
    "w": (S, {"entry", "source", "meaning", "usage"}, {"pos", "pron", "xlit", "xml:lang", "src"}),
    "source": (M, {"entry"}, set()),
    "meaning": (S, {"entry"}, set()),
    "usage": (S, {"entry"}, set()),
    "def": (S, {"source", "meaning"}, set()),
    "note": (S, {"entry", "source", "meaning"}, set()),
}
MGNT = {
    "strongsdictionary": (S, {None}, set()),
    "prologue": (S, {"strongsdictionary"}, set()),
    "entries": (S, {"strongsdictionary"}, set()),
    "entry": (S, {"entries"}, {"strongs"}),
    "strongs": (S, {"entry"}, set()),
    "greek": (E, {"entry", "strongs_derivation", "strongs_def"}, {"BETA", "unicode", "translit"}),
    "pronunciation": (E, {"entry", "strongs_derivation", "strongs_def"}, {"strongs"}),
    "strongs_derivation": (S, {"entry"}, set()),
    "strongs_def": (S, {"entry"}, set()),
    "kjv_def": (S, {"entry"}, set()),
    "latin": (S, {"strongs_derivation", "strongs_def"}, set()),
    "strongsref": (E, {"entry", "strongs_derivation", "strongs_def", "kjv_def"}, {"language", "strongs"}),
}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def file_sha(path):
    with open(path, "rb") as f:
        return sha(f.read())


def body_start(data, which):
    if which == "OSHB":
        if not data.startswith(OSHB_DECL):
            raise Refused("HebrewStrong.xml: not the pinned XML declaration")
        return len(OSHB_DECL)
    if not data.startswith(MGNT_DECL) or data[len(MGNT_DECL):MGNT_SUBSET[0]] != b"\n":
        raise Refused("strongsgreek.xml: not the pinned declaration + LF")
    a, b, want = MGNT_SUBSET
    sub = data[a:b]
    if not sub.startswith(b"<!DOCTYPE strongsdictionary [") or sha(sub) != want:
        raise Refused("strongsgreek.xml: DTD subset differs from the pinned one")
    return b


def tokens(data, pos):
    """(kind, name, [(attr, value)], start, end) from pos; raises on anything else."""
    body = data[pos:]
    for bad in (b"<!", b"<?", b"&", b"]]>"):
        if bad in body:
            raise Refused("%r in the parsed part of the file" % bad)
    n = len(data)
    out = []
    while pos < n:
        lt = data.find(b"<", pos)
        if lt == -1:
            lt = n
        if lt > pos:
            out.append(("text", None, None, pos, lt))
        if lt == n:
            break
        m = TAG.match(data, lt)
        if not m:
            raise Refused("unparseable tag at byte %d" % lt)
        slash, name, attrs, empty = m.groups()
        if slash and (attrs.strip() or empty):
            raise Refused("bad end tag at byte %d" % lt)
        al = []
        for am in ATTR.finditer(attrs):
            k = am.group(1).decode("ascii")
            if any(k == x for x, _ in al):
                raise Refused("duplicate attribute at byte %d" % lt)
            al.append((k, am.group(2).decode("utf-8")))
        kind = "end" if slash else ("empty" if empty else "start")
        out.append((kind, name.decode("ascii"), al, lt, m.end()))
        pos = m.end()
    return out


def walk(data, which):
    """Strict walk of one lexicon; returns a list of entry records."""
    prof = OSHB if which == "OSHB" else MGNT
    toks = tokens(data, body_start(data, which))
    stack = []
    root_done = False
    entries = []
    cur = None  # the open entry: dict
    prev = 0
    for kind, name, attrs, s, e in toks:
        if kind == "text":
            raw = data[s:e]
            if cur is not None:
                cur["text"].append(raw)
                cur["tree"].append(raw)
            elif not (stack and stack[-1] == "prologue") and not all(b in WS for b in raw):
                raise Refused("%s: text outside an entry at byte %d" % (which, s))
            continue
        if kind == "end":
            if not stack or stack[-1] != name:
                raise Refused("%s: mismatched end tag </%s> at byte %d" % (which, name, s))
            stack.pop()
            if name == "entry":
                entries.append(cur)
                cur = None
            elif cur is not None:
                cur["tree"].append(("</%s>" % name).encode("ascii"))
            if not stack:
                root_done = True
            continue
        rule = prof.get(name)
        if rule is None:
            raise Refused("%s: element <%s> at byte %d is not in the profile" % (which, name, s))
        kinds, parents, allowed = rule
        if kinds != M and kinds != kind:
            raise Refused("%s: <%s> at byte %d as %s is not allowed" % (which, name, s, kind))
        parent = stack[-1] if stack else None
        if parent not in parents or root_done:
            raise Refused("%s: <%s> at byte %d not allowed in <%s>" % (which, name, s, parent))
        for k, v in attrs:
            if k not in allowed:
                raise Refused("%s: attribute %s on <%s> at byte %d" % (which, k, name, s))
        if name == "entry":
            if len(attrs) != 1:
                raise Refused("%s: entry at byte %d needs exactly one attribute" % (which, s))
            v = attrs[0][1]
            if which == "OSHB":
                if not re.fullmatch(r"H[1-9][0-9]{0,4}", v):
                    raise Refused("OSHB: entry id %r not in the pinned spelling" % v)
                num, key = int(v[1:]), v
            else:
                if not re.fullmatch(r"[0-9]{5}", v) or int(v) == 0:
                    raise Refused("MGNT: entry strongs %r not in the pinned spelling" % v)
                num = int(v)
                key = "G%d" % num
            if num <= prev:
                raise Refused("%s: entry %s not ascending" % (which, key))
            prev = num
            cur = {"key": key, "text": [], "tree": [], "attrs": ["entry@%s=%s" % attrs[0]]}
        elif cur is not None:
            cur["attrs"].extend("%s@%s=%s" % (name, k, v) for k, v in attrs)
            cur["tree"].append(("<%s>" % name).encode("ascii"))
            if kind == "empty":
                cur["tree"].append(("</%s>" % name).encode("ascii"))
        if kind == "start":
            stack.append(name)
        elif not stack:
            root_done = True
    if stack or not root_done:
        raise Refused("%s: unclosed at EOF: %s" % (which, stack))
    return entries


def main(heb, heb_sum, grc, grc_sum, out_dir):
    if os.path.basename(heb) != "HebrewStrong.xml" or os.path.basename(grc) != "strongsgreek.xml":
        raise Refused("expected HebrewStrong.xml and strongsgreek.xml")
    hd = load_pinned(heb, heb_sum)
    gd = load_pinned(grc, grc_sum)
    he = walk(hd, "OSHB")
    ge = walk(gd, "MGNT")
    if len(he) != 8674 or len(ge) != 5624:
        raise Refused("unexpected entry counts %d / %d" % (len(he), len(ge)))
    lines = [
        "# lexicon-entries.tsv: one line per lexicon entry (8,674 OSHB, then 5,624 MGNT, file order).",
        "# generator: docs/pipeline/survey-scripts/oracle_lexicons.py sha256 %s" % file_sha(os.path.join(HERE, "oracle_lexicons.py")),
        "# helper: docs/pipeline/survey-scripts/osis_tokens.py sha256 %s" % file_sha(os.path.join(HERE, "osis_tokens.py")),
        "# source: LEX-HEB-OSHB HebrewStrong.xml sha256 %s" % sha(hd),
        "# source: LEX-GRC-MGNT19 strongsgreek.xml sha256 %s" % sha(gd),
        "# mgnt-dtd-subset: bytes 56..1090 sha256 %s (skipped, never interpreted)" % MGNT_SUBSET[2],
        "# python: %d.%d.%d" % sys.version_info[:3],
        "# Generated by an independent implementation (DESIGN.md 11.1). Never edit by hand;",
        "# regenerate only when a source pin or a generator changes, with the owner's review.",
        "# columns: key\tlexicon\tord\ttext_bytes\ttext_sha256\tattr_count\tattr_sha256\ttree_sha256",
        "# text: raw character data between the entry tags, tags stripped, CR kept; attrs: element@name=value,",
        "# document order (the entry's own first), LF-joined; tree: tokens re-serialised without attributes",
        "# ('<x>', '</x>', empty as '<x></x>', text raw).",
    ]
    for lexicon, entries in (("LEX-HEB-OSHB", he), ("LEX-GRC-MGNT19", ge)):
        for ord_, r in enumerate(entries, 1):
            text = b"".join(r["text"])
            attrs = "\n".join(r["attrs"]).encode("utf-8")
            lines.append("\t".join([r["key"], lexicon, str(ord_), str(len(text)), sha(text),
                                    str(len(r["attrs"])), sha(attrs), sha(b"".join(r["tree"]))]))
    with open(os.path.join(out_dir, "lexicon-entries.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    print("wrote lexicon-entries.tsv (%d + %d entries)" % (len(he), len(ge)))


if __name__ == "__main__":
    if len(sys.argv) != 6:
        sys.exit(__doc__)
    main(*sys.argv[1:6])
