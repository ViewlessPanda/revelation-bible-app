"""Machine-extract byte-exact raw OSIS excerpts for the golden-sample references.

Usage:
  py -I -B golden_excerpts.py <kjv.osis.xml> <BIB-KJV-CW31.sha256> <red-letter-golden-samples.md> <out.md>

Read-only on the source. No XML parser: the strict regex tokenizer and strict
walker in osis_tokens.py. Refuses to run if the source's SHA-256 does not match
the checksum file, if the source fails the strict walker, or if no `ref:` is
found in the golden-samples YAML block.
"""
import hashlib
import os
import re
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned, strict_tokenize  # noqa: E402

USFM_TO_OSIS = {
    "GEN": "Gen", "PSA": "Ps", "MAT": "Matt", "MRK": "Mark", "LUK": "Luke",
    "JHN": "John", "ACT": "Acts", "1CO": "1Cor", "2CO": "2Cor", "REV": "Rev",
}
W_TAG = re.compile(rb"<w(?:\s[^>]*)?/?>")
W_ATTR = re.compile(rb'\s(?:src|lemma|morph|type)="[^"]*"')
TAGS = re.compile(rb"<[^>]*>")
SOURCE_URL = ("https://gitlab.com/crosswire-bible-society/kjv/-/raw/"
              "d490be7e34762deb2c76cb2c1306d4808e27890d/kjv.osis.xml")


def main(src, sumfile, golden, out_path):
    if os.path.basename(src) != "kjv.osis.xml":
        sys.exit("expected kjv.osis.xml; refusing")
    data = load_pinned(src, sumfile)
    digest = hashlib.sha256(data).hexdigest()
    toks = strict_tokenize(data)

    gtext = open(golden, "rb").read().decode("utf-8")
    block = gtext.split("```yaml", 1)[1].split("```", 1)[0]
    refs = []
    # Accepts both row styles the scholar's file has used:
    #   {ref: MAT.3.15, ...}   and   - ref: MAT.3.15
    for rm in re.finditer(r"(?:\{ref: |^- ref: )([0-9A-Z]{3})\.(\d+)\.(\d+)(?:,|[ \t]*$)",
                          block, re.M):
        b, c, v = rm.groups()
        if b not in USFM_TO_OSIS:
            sys.exit("unknown book code %s" % b)
        r = ("%s.%s.%s" % (b, c, v), "%s.%s.%s" % (USFM_TO_OSIS[b], c, v))
        if r not in refs:
            refs.append(r)
    if not refs:
        sys.exit("no ref: values found in the golden-samples YAML block; refusing")

    # Index verses: sID tag start, eID tag end; q state at verse start and end.
    verses = {}
    verse = None
    q_open_at = None  # (verse where the open q started, byte offset of its start tag)
    cur = None
    for t in toks:
        if t.kind == "empty" and t.name == "verse":
            if "sID" in t.attrs:
                verse = t.attrs["sID"]
                cur = {"a": t.start, "q_at_start": q_open_at, "q_starts": 0, "q_ends": 0}
            else:
                if t.attrs["eID"] != verse:
                    sys.exit("unbalanced verse milestones at %d" % t.start)
                cur["b"] = t.end
                cur["q_at_end"] = q_open_at
                verses[verse] = cur
                verse = None
        elif t.name == "q":
            if t.kind == "start":
                if q_open_at is not None:
                    sys.exit("nested q at %d" % t.start)
                q_open_at = (verse, t.start)
                if verse:
                    cur["q_starts"] += 1
            elif t.kind == "end":
                q_open_at = None
                if verse:
                    cur["q_ends"] += 1

    o = []
    o.append("# Golden-sample source excerpts (KJV, BIB-KJV-CW31)\n")
    o.append("**Machine-extracted. Do not edit by hand.** Every excerpt below was copied by a script "
             "from the pinned source file, byte for byte, from the first byte of the verse's "
             "`<verse … sID=…/>` tag to the last byte of its `<verse eID=…/>` tag. Nothing here was "
             "typed by a person, and nothing was normalised, re-wrapped or re-encoded.\n")
    o.append("| Field | Value |\n|---|---|")
    o.append("| Source ID | BIB-KJV-CW31 (`sources/SOURCES.md`) |")
    o.append("| Source file | `sources/raw/BIB-KJV-CW31/kjv.osis.xml` |")
    o.append("| Pinned URL | %s |" % SOURCE_URL)
    o.append("| Size (bytes) | %d |" % len(data))
    o.append("| SHA-256 | `%s` |" % digest)
    o.append("| Git blob SHA-1 | `369705ecfc1f734202bdb8c195fd1c930f9f89d2` (checked with `git hash-object`) |")
    o.append("| References | the %d distinct `ref:` values in the YAML block of `docs/scholar/red-letter-golden-samples.md`, in that order |" % len(refs))
    o.append("| Generator | `docs/pipeline/survey-scripts/golden_excerpts.py` (regex tokenizer, no XML parser; checks the SHA-256 first) |")
    o.append("")
    o.append("## How to use this file\n")
    o.append("- This is the **source** column for the golden samples: what the pinned digital file "
             "actually marks. A person compares it with the printed edition (the scholar's file) and "
             "fills `source_expected` by hand from these excerpts.")
    o.append("- Red letters are the `<q who=\"Jesus\" marker=\"\">` … `</q>` containers. They are not "
             "milestones, so a span that started in an earlier verse has its start tag **outside** "
             "this excerpt. The line \"`q` open at verse start\" says so, and names the verse where "
             "that `<q>` tag is. Those two lines and the tag counts are machine-derived from the "
             "same tokenizer.")
    o.append("- `<milestone type=\"x-p\" marker=\"¶\"/>` is a paragraph mark, not text. `<note>` "
             "content (KJV margin notes) is not verse text and is not imported in M1.")
    o.append("- Each excerpt (`xml` block, authoritative) is followed by a **reading aid** (`text` "
             "block): the same bytes with the `src`, `lemma`, `morph` and `type` attributes deleted "
             "from `<w>` tags, so the `<q>` boundaries are easier to see. The generator checks that "
             "the aid's text content (everything outside tags) is identical to the excerpt's. When "
             "in doubt, read the `xml` block.")
    o.append("- To re-check an excerpt's bytes: take `length` bytes starting at byte `offset` "
             "(0-based) of the source file and compare the SHA-256 with the one given.")
    o.append("- Excerpts are kept to the golden references only (internal review document; the "
             "text is under BIB-KJV-CW31's terms).\n")
    for usfm, osis in refs:
        v = verses.get(osis)
        if v is None:
            sys.exit("reference %s not found" % osis)
        raw = data[v["a"]:v["b"]]
        if b"\n" in raw or b"```" in raw:
            sys.exit("excerpt for %s contains a newline or backtick fence" % osis)
        qs = v["q_at_start"]
        qe = v["q_at_end"]
        o.append("### %s (OSIS `%s`)\n" % (usfm, osis))
        o.append("- offset %d, length %d, excerpt SHA-256 `%s`" % (v["a"], len(raw), hashlib.sha256(raw).hexdigest()))
        o.append("- `q` open at verse start: %s" % ("no" if qs is None else "**yes**, its `<q>` tag is in `%s` at byte %d" % (qs[0], qs[1])))
        o.append("- `q` open at verse end: %s" % ("no" if qe is None else "**yes** (continues into the next verse)"))
        o.append("- `<q>` start tags in excerpt: %d; `</q>` end tags in excerpt: %d\n" % (v["q_starts"], v["q_ends"]))
        o.append("```xml")
        o.append(raw.decode("utf-8"))
        o.append("```\n")
        aid = W_TAG.sub(lambda wm: W_ATTR.sub(b"", wm.group(0)), raw)
        if TAGS.sub(b"", aid) != TAGS.sub(b"", raw):
            sys.exit("reading aid changed text content for %s" % osis)
        o.append("Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, "
                 "`morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):\n")
        o.append("```text")
        o.append(aid.decode("utf-8"))
        o.append("```\n")
    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(o))

    # Self-check: every excerpt in the written file must occur byte-for-byte in the source.
    written = open(out_path, "rb").read()
    blocks = re.findall(rb"```xml\n(.*?)\n```", written, re.S)
    if len(blocks) != len(refs):
        sys.exit("self-check failed: block count")
    for blk in blocks:
        if data.find(blk) < 0:
            sys.exit("self-check failed: excerpt not found in source")
    print("wrote %d excerpts; self-check passed; source sha256 %s" % (len(refs), digest))


if __name__ == "__main__":
    main(*sys.argv[1:5])
