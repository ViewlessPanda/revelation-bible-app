"""Write pipeline/tests/golden/red-letter-source.tsv and verse-text.tsv.

Usage:
  py -I -B golden_tsv.py <red-letter-golden-samples.md> <golden-source-excerpts.md> \
      <kjv.osis.xml> <BIB-KJV-CW31.sha256> <out_dir>

red-letter-source.tsv is a mechanical transcription of the `source_expected`
column of the scholar's section 10 YAML block (one row per `- ref:`), so the
Rust test can compare the built database with values that come from outside
the pipeline (DESIGN.md 12). Nothing is derived from pipeline output.

verse-text.tsv holds the exact stored text expected for each golden
reference plus Ps 23:1-6 (DESIGN.md 12.3, T19). It is machine-extracted from
the pinned source by tag stripping: the bytes between the verse's sID tag and
its eID tag, minus <note>...</note>, minus every tag. That is not the Rust
parser's method and not the oracle walker's. The design asks for hand-copied
rows; a person has not checked these yet (M1-F14), and the header says so.

Strict: an unknown line shape in the YAML block, a duplicate ref, a missing
source_expected, or a reference not found in the source refuses, and nothing
is written.
"""
import hashlib
import os
import re
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from osis_tokens import Refused, load_pinned, strict_tokenize  # noqa: E402

USFM_TO_OSIS = {"GEN": "Gen", "PSA": "Ps", "MAT": "Matt", "MRK": "Mark", "LUK": "Luke",
                "JHN": "John", "ACT": "Acts", "1CO": "1Cor", "2CO": "2Cor", "REV": "Rev"}
COLUMNS = ["spans", "start", "end", "first", "last", "span1_first", "span1_last",
           "black_between", "span2_first", "span2_last", "black_after_first",
           "q_open_at_start", "q_open_at_end", "status", "human"]
REF_LINE = re.compile(r"- ref: ([0-9A-Z]{3})\.([1-9][0-9]*)\.([1-9][0-9]*)\Z")
KEY_LINE = re.compile(r"  ([A-Za-z_0-9]+): (.*)\Z")
NOTE = re.compile(rb"<note\b[^>]*>.*?</note>", re.S)
TAG = re.compile(rb"<[^>]*>")


def parse_map(s, where):
    if not (s.startswith("{") and s.endswith("}")):
        raise Refused("%s: not a flow map" % where)
    body = s[1:-1]
    out = {}
    i = 0
    while i < len(body):
        m = re.compile(r"([a-z_0-9]+): ").match(body, i)
        if not m:
            raise Refused("%s: bad key at %d" % (where, i))
        key = m.group(1)
        i = m.end()
        if body.startswith('"', i):
            j = body.index('"', i + 1)
            val = body[i + 1:j]
            if "\\" in val or "\t" in val:
                raise Refused("%s: escape or tab in a value" % where)
            i = j + 1
        else:
            j = body.find(", ", i)
            j = len(body) if j < 0 else j
            val = body[i:j]
            i = j
        if key in out:
            raise Refused("%s: duplicate key %s" % (where, key))
        out[key] = val
        if i < len(body):
            if not body.startswith(", ", i):
                raise Refused("%s: expected ', ' at %d" % (where, i))
            i += 2
    return out


def scholar_rows(md_text):
    sec = md_text.split("\n## 10. ", 1)
    if len(sec) != 2:
        raise Refused("no section 10 in the golden-samples file")
    block = sec[1].split("```yaml\n", 1)[1].split("\n```", 1)[0]
    rows = []
    for n, line in enumerate(block.split("\n"), 1):
        if line.startswith("#") or line == "":
            continue
        m = REF_LINE.match(line)
        if m:
            b, c, v = m.groups()
            if b not in USFM_TO_OSIS:
                raise Refused("unknown book %s" % b)
            ref = "%s.%s.%s" % (b, c, v)
            if any(r[0] == ref for r in rows):
                raise Refused("duplicate ref %s" % ref)
            rows.append([ref, "%s.%s.%s" % (USFM_TO_OSIS[b], c, v), None])
            continue
        m = KEY_LINE.match(line)
        if m and rows:
            if m.group(1) == "source_expected":
                if rows[-1][2] is not None:
                    raise Refused("two source_expected for %s" % rows[-1][0])
                rows[-1][2] = parse_map(m.group(2), rows[-1][0])
            continue
        raise Refused("YAML block line %d has an unknown shape" % n)
    for r in rows:
        if r[2] is None:
            raise Refused("%s has no source_expected" % r[0])
        unknown = set(r[2]) - set(COLUMNS) - {"basis"}
        if unknown:
            raise Refused("%s: unknown source_expected keys %s" % (r[0], sorted(unknown)))
    return rows


def sha_file(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def main(golden_md, excerpts_md, src, sumfile, out_dir):
    with open(golden_md, "rb") as f:
        md = f.read().decode("utf-8")
    rows = scholar_rows(md)
    data = load_pinned(src, sumfile)
    strict_tokenize(data)  # the whole file passes the strict walker first

    hdr = [
        "# generator: docs/pipeline/survey-scripts/golden_tsv.py sha256 %s" % sha_file(os.path.abspath(__file__)),
        "# scholar file: docs/scholar/red-letter-golden-samples.md sha256 %s" % sha_file(golden_md),
        "# excerpts: docs/pipeline/golden-source-excerpts.md sha256 %s" % sha_file(excerpts_md),
        "# source: BIB-KJV-CW31 kjv.osis.xml sha256 %s" % hashlib.sha256(data).hexdigest(),
    ]
    red = ["# red-letter-source.tsv (DESIGN.md 12.3, T18). Machine transcription of the source_expected",
           "# column of docs/scholar/red-letter-golden-samples.md section 10, one row per ref, in that order.",
           "# WHO FILLED THE VALUES: the scholar agent (an AI), from golden-source-excerpts.md; the print",
           "# columns were read by an AI from page images. NOT checked by a person yet: every row is",
           "# human: pending (M1-F14, DESIGN.md open question 15). Do not edit by hand; the Rust test checks",
           "# that this file equals the scholar's YAML. '-' = not given."] + hdr
    red.append("\t".join(["ref", "osis"] + COLUMNS))
    for ref, osis, m in rows:
        red.append("\t".join([ref, osis] + [m.get(k, "-") or "-" for k in COLUMNS]))

    # verse-text.tsv
    refs = [(r[0], r[1]) for r in rows]
    for v in range(1, 7):
        if ("PSA.23.%d" % v) not in [r[0] for r in refs]:
            refs.append(("PSA.23.%d" % v, "Ps.23.%d" % v))
    vt = ["# verse-text.tsv (DESIGN.md 12.3, T19): exact stored text expected per reference.",
          "# MACHINE-EXTRACTED by tag stripping from the pinned source (bytes between the sID and eID",
          "# tags, minus <note>...</note>, minus every tag), not hand-copied as DESIGN.md 12.3 asks and",
          "# not checked by a person yet (M1-F14). Trailing spaces are kept; diffs show them as \\x20."] + hdr
    vt.append("ref\tosis\tcode_points\ttext")
    for ref, osis in refs:
        sid = ('<verse osisID="%s" sID="%s"/>' % (osis, osis)).encode()
        eid = ('<verse eID="%s"/>' % osis).encode()
        a = data.find(sid)
        b = data.find(eid, a)
        if a < 0 or b < 0 or data.find(sid, a + 1) >= 0:
            raise Refused("%s not found exactly once" % osis)
        seg = TAG.sub(b"", NOTE.sub(b"", data[a + len(sid):b]))
        text = seg.decode("utf-8")
        if "\t" in text or "\n" in text or "\r" in text:
            raise Refused("%s: control character in text" % osis)
        vt.append("\t".join([ref, osis, str(len(text)), text]))

    os.makedirs(out_dir, exist_ok=True)
    for name, lines in (("red-letter-source.tsv", red), ("verse-text.tsv", vt)):
        with open(os.path.join(out_dir, name), "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(lines) + "\n")
    print("wrote %d red-letter rows and %d verse-text rows" % (len(rows), len(refs)))


if __name__ == "__main__":
    if len(sys.argv) != 6:
        sys.exit(__doc__)
    main(*sys.argv[1:6])
