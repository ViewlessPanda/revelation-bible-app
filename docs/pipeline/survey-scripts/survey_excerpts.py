"""Machine-extract the short excerpts quoted in docs/pipeline/kjv-osis-survey.md.

Usage:
  py -I -B survey_excerpts.py <sources_dir> <out.md>

<sources_dir> is the repo's sources/ directory. Every input is checked against
its sources/checksums/<ID>.sha256 first (osis_tokens.load_pinned). Each excerpt
is located by byte search, copied byte for byte, and written with its offset,
length and SHA-256. Nothing is typed by hand, decoded with replacement, or
normalised. No tokenizer is used here: excerpts are byte ranges, not parsed text.
"""
import hashlib
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned  # noqa: E402

FILES = {
    "kjv": ("BIB-KJV-CW31", "kjv.osis.xml"),
    "heb": ("LEX-HEB-OSHB", "HebrewStrong.xml"),
    "grc": ("LEX-GRC-MGNT19", "strongsgreek.xml"),
}


def load(src_dir, key):
    sid, name = FILES[key]
    return load_pinned(os.path.join(src_dir, "raw", sid, name),
                       os.path.join(src_dir, "checksums", sid + ".sha256"))


def find(data, needle, start=0):
    i = data.find(needle, start)
    if i < 0:
        sys.exit("not found: %r" % needle)
    return i


def verse(data, ref):
    a = find(data, b'<verse osisID="%s" sID="%s"/>' % (ref.encode(), ref.encode()))
    tag = b'<verse eID="%s"/>' % ref.encode()
    b = find(data, tag, a) + len(tag)
    return a, b


def main(src_dir, out_path):
    kjv, heb, grc = (load(src_dir, k) for k in ("kjv", "heb", "grc"))
    ex = []  # (label, file label, data, start, end)

    def add(label, fname, data, a, b):
        ex.append((label, fname, data, a, b))

    a, b = verse(kjv, "Gen.1.1")
    add("K1. Gen 1:1, a whole verse (milestones, `w`, `lemma`, punctuation outside `w`)", "kjv.osis.xml", kjv, a, b)

    a, b = verse(kjv, "Matt.3.17")
    add("K2. Matt 3:17, start: an empty `<w/>` carrying a Strong's number with no English text", "kjv.osis.xml", kjv, a, find(kjv, b"</w>", a) + 4)

    a, b = verse(kjv, "Matt.3.15")
    q = find(kjv, b'<q who="Jesus"', a)
    add("K3. Matt 3:15, a red span opening mid-verse (space before `<q>`, `marker=\"\"`)", "kjv.osis.xml", kjv,
        kjv.rfind(b"<w ", a, q), find(kjv, b"</w>", q) + 4)

    a10, b10 = verse(kjv, "John.3.10")
    a11, b11 = verse(kjv, "John.3.11")
    add("K4. John 3:10 to 3:11, a red span crossing a verse boundary (`<q>` stays open across the milestones)", "kjv.osis.xml", kjv,
        kjv.rfind(b"<w ", a10, b10), find(kjv, b"</w>", a11) + 4)

    a21, b21 = verse(kjv, "John.3.21")
    a22, b22 = verse(kjv, "John.3.22")
    end_q = kjv.rfind(b"</q>", a21, b21)
    add("K5. John 3:21 to 3:22, a red span closing at verse end, then a pilcrow milestone and an empty `<w/>`", "kjv.osis.xml", kjv,
        kjv.rfind(b"<w ", a21, end_q), find(kjv, b"/>", find(kjv, b"<w ", a22)) + 2)

    a, b = verse(kjv, "Matt.8.9")
    add("K6. Matt 8:9, end: a space before the verse's end milestone (one of 91 verses)", "kjv.osis.xml", kjv,
        kjv.rfind(b"<w ", a, kjv.rfind(b"<transChange", a, b)), b)

    a, b = verse(kjv, "2Cor.8.10")
    i = find(kjv, b"G5129", a)
    add("K7. 2 Cor 8:10, the one malformed `lemma` token (`G5129` without `strong:`)", "kjv.osis.xml", kjv,
        kjv.rfind(b"<w ", a, i), find(kjv, b"</w>", i) + 4)

    a, b = verse(kjv, "Ps.23.1")
    add("K8. Ps 23:1, start: `divineName` (stored as `Lord`, printed as small capitals)", "kjv.osis.xml", kjv, a, find(kjv, b"</w>", a) + 4)

    a, b = verse(kjv, "Dan.5.25")
    i = find(kjv, b"<inscription>", a)
    add("K9. Dan 5:25, `inscription` (text already in capitals)", "kjv.osis.xml", kjv, i, find(kjv, b"</inscription>", i) + 14)

    a, b = verse(kjv, "Exod.26.4")
    i = find(kjv, b"<note", a)
    add("K10. Exod 26:4, a study note inside a verse (excluded from verse text in M1)", "kjv.osis.xml", kjv, i, find(kjv, b"</note>", i) + 7)

    i = find(kjv, b'<chapter osisID="Ps.3"')
    j = find(kjv, b"</title>", find(kjv, b'<title type="psalm"', i)) + 8
    add("K11. Psalm 3, chapter start: a generated chapter title and the canonical psalm title, both outside any verse", "kjv.osis.xml", kjv, i, j)

    i = find(kjv, b'<title type="acrostic"')
    add("K12. Psalm 119, first acrostic title (outside verses)", "kjv.osis.xml", kjv, i, find(kjv, b"</title>", i) + 8)

    i = find(kjv, b'<div type="colophon" osisID="Rom.c">')
    add("K13. Romans colophon, start (outside verses)", "kjv.osis.xml", kjv, i, find(kjv, b"</w>", find(kjv, b"</w>", i) + 4) + 4)

    i = find(heb, b'<entry id="H1">')
    add("H1. HebrewStrong.xml, entry H1 (CRLF and tab indentation are in the file)", "HebrewStrong.xml", heb, i, find(heb, b"</entry>", i) + 8)
    i = find(heb, b'<entry id="H5223">')
    add("H2. HebrewStrong.xml, entry H5223 (the one empty `<source/>`)", "HebrewStrong.xml", heb, i, find(heb, b"</entry>", i) + 8)

    i = find(grc, b"<!DOCTYPE")
    add("G1. strongsgreek.xml, the internal DTD subset (skipped, never interpreted)", "strongsgreek.xml", grc, i, find(grc, b"]>", i) + 2)
    i = find(grc, b'<entry strongs="00001">')
    add("G2. strongsgreek.xml, entry 00001 (text directly inside `<entry>`, hard line feeds, a leading space)", "strongsgreek.xml", grc, i, find(grc, b"</entry>", i) + 8)
    i = find(grc, b'<entry strongs="02717">')
    add("G3. strongsgreek.xml, entry 02717 (one of 101 \"Not Used\" placeholders)", "strongsgreek.xml", grc, i, find(grc, b"</entry>", i) + 8)

    out = open(out_path, "wb")
    out.write(b"## Appendix: machine-extracted excerpts\n\n")
    out.write(b"Generated by `docs/pipeline/survey-scripts/survey_excerpts.py`; do not edit by hand. "
              b"Each block is the exact bytes at the given 0-based offset of the named file "
              b"(CR bytes, if any, are shown as they are in the file).\n")
    for label, fname, data, a, b in ex:
        blob = data[a:b]
        if b"```" in blob or not (0 <= a < b <= len(data)):
            sys.exit("bad excerpt " + label)
        blob.decode("utf-8")  # must be whole characters
        out.write(b"\n### %s\n\n" % label.encode())
        out.write(b"`%s` offset %d, length %d, SHA-256 `%s`\n\n" % (
            fname.encode(), a, b - a, hashlib.sha256(blob).hexdigest().encode()))
        out.write(b"```xml\n" + blob + (b"" if blob.endswith(b"\n") else b"\n") + b"```\n")
    out.close()


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
