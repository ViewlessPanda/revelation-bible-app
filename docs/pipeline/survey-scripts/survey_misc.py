"""Cross-checks and small facts quoted in docs/pipeline/kjv-osis-survey.md.

Usage:
  py -I -B survey_misc.py <sources_dir> <out.txt>

<sources_dir> is the repo's sources/ directory; each file is checked against
sources/checksums/<ID>.sha256 first, and the KJV must pass the strict walker,
before anything is written. Read-only, no XML parser (strict regex tokenizer
from osis_tokens.py for the KJV; byte-level regexes elsewhere).
"""
import os
import re
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned, strict_tokenize  # noqa: E402

FILES = {"kjv": ("BIB-KJV-CW31", "kjv.osis.xml"),
         "heb": ("LEX-HEB-OSHB", "HebrewStrong.xml"),
         "grc": ("LEX-GRC-MGNT19", "strongsgreek.xml")}


def load(src_dir, key):
    sid, name = FILES[key]
    return load_pinned(os.path.join(src_dir, "raw", sid, name),
                       os.path.join(src_dir, "checksums", sid + ".sha256"))


def main(src_dir, out_path):
    kjv, heb, grc = (load(src_dir, k) for k in ("kjv", "heb", "grc"))
    kjv_toks = strict_tokenize(kjv)
    lines = []  # the report is written only after everything succeeds

    def p(*a):
        lines.append(" ".join(str(x) for x in a))

    # Candidate verse texts (same definition as survey_kjv_detail.py).
    verse, cur, note, text, insc = None, [], 0, {}, []
    for t in kjv_toks:
        if t.kind == "text":
            if verse and not note:
                cur.append(kjv[t.start:t.end])
            continue
        if t.name == "verse":
            if "sID" in t.attrs:
                verse, cur = t.attrs["sID"], []
            else:
                text[verse] = b"".join(cur).decode("utf-8")
                verse = None
            continue
        if t.name == "note" and t.kind != "empty":
            note += 1 if t.kind == "start" else -1
        if t.name == "inscription" and t.kind == "start":
            insc.append(verse)
    p("verses:", len(text))
    p("verses with an inscription:", insc)
    trailing = [r for r, s in text.items() if s.endswith(" ")]
    p("verses ending with a space:", len(trailing), trailing)
    for needle in ("[", "-", "–", "’", "'", "Beth–el", "Tubal–cain"):
        refs = [r for r, s in text.items() if needle in s]
        p("%r: occurrences %d in %d verses; first %s" % (
            needle, sum(s.count(needle) for s in text.values()), len(refs), refs[:12]))
    for r in ("John.3.23", "Exod.32.32", "1John.2.23"):
        p(r, "=", text[r])

    # Strong's keys: KJV vs lexicons.
    keys = {}
    for lem in re.findall(rb'lemma="([^"]*)"', kjv):
        for tk in lem.split(b" "):
            m = re.fullmatch(rb"strong:([HG])(\d+)", tk)
            if m:
                k = (m.group(1).decode(), int(m.group(2)))
                keys[k] = keys.get(k, 0) + 1
    hids = {int(x) for x in re.findall(rb'<entry id="H(\d+)">', heb)}
    gall = {int(x) for x in re.findall(rb'<entry strongs="(\d+)">', grc)}
    gnot = {int(k) for k in re.findall(
        rb'<entry strongs="(\d+)">\s*<strongs>\d+</strongs>\s*Not Used\s*</entry>', grc)}
    p("distinct KJV keys:", len(keys), " H:", sum(1 for k in keys if k[0] == "H"),
      " G:", sum(1 for k in keys if k[0] == "G"))
    p("KJV H keys missing from HebrewStrong.xml:", sorted(k for k in keys if k[0] == "H" and k[1] not in hids))
    p("KJV G keys missing from strongsgreek.xml:", sorted(k for k in keys if k[0] == "G" and k[1] not in gall))
    p("Greek 'Not Used' entries:", len(gnot), " KJV keys hitting one:",
      sorted(k for k in keys if k[0] == "G" and k[1] in gnot))
    bad = [(lang, num) for lang, num in re.findall(rb'<strongsref language="([A-Z]+)" strongs="(\d+)"', grc)
           if int(num) == 0 or (lang == b"GREEK" and (int(num) not in gall or int(num) in gnot))
           or (lang == b"HEBREW" and int(num) not in hids)]
    p("Greek strongsref targets missing, Not Used or zero:", len(bad))
    p("Hebrew w@src targets missing:", [s for s in re.findall(rb'<w src="H(\d+)"', heb) if int(s) not in hids])
    nt = kjv.find(b'<div type="book" osisID="Matt"')
    p("strong:G tokens before Matthew:", len(re.findall(rb"strong:G\d", kjv[:nt])),
      " strong:H tokens from Matthew on:", len(re.findall(rb"strong:H\d", kjv[nt:])))
    p("distinct Hebrew w@pos values:", len(set(re.findall(rb'pos="([^"]*)"', heb))))

    # Lexicon line breaks; size facts used for the parser caps.
    for name, d in (("HebrewStrong.xml", heb), ("strongsgreek.xml", grc)):
        mixed = sum(1 for m in re.finditer(rb">([^<]*)<", d)
                    if (b"\r" in m.group(1) or b"\n" in m.group(1)) and m.group(1).strip())
        p(name, "text nodes containing a line break and non-whitespace:", mixed)
    for name, d in (("kjv.osis.xml", kjv), ("HebrewStrong.xml", heb), ("strongsgreek.xml", grc)):
        longest = max(len(m.group(1)) for m in re.finditer(rb">([^<]*)<", d))
        attr = max(len(x) for x in re.findall(rb'="([^"]*)"', d))
        per_tag = max(len(re.findall(rb"\s[\w:.-]+=", t)) for t in re.findall(rb"<[A-Za-z][^>]*>", d))
        p(name, "longest text node:", longest, " longest attribute value:", attr, " most attributes on a tag:", per_tag)
    with open(out_path, "w", encoding="utf-8", newline="\n") as out:
        out.write("".join(line + "\n" for line in lines))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
