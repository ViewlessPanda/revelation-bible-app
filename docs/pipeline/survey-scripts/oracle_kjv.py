"""Independent oracle files for the KJV pipeline (DESIGN.md 11.1, M1-F4).

Usage:
  py -I -B oracle_kjv.py <kjv.osis.xml> <BIB-KJV-CW31.sha256> <out_dir>

Writes <out_dir>/kjv-chapters.tsv and <out_dir>/kjv-q.tsv, and nothing else.
Run from outside sources/raw, with paths as arguments. Shares no code with
the Rust pipeline. Fail-closed: the source is read once and checked against
its checksum file (check_sha via load_pinned), the whole file passes the
strict walker (StrictWalker via strict_tokenize: element/kind allowlist,
end-tag names, nested/empty/milestone q refused) before anything is
computed, and both files are written only after every step succeeded.

Definitions (DESIGN.md 3.1, 5.2, 6.1), implemented here independently:
- stored verse text: the raw bytes of every text token after a verse's sID
  milestone and before its eID milestone, outside <note> subtrees;
- offsets are code points (Python str indices);
- marks: transChange / divineName outside notes, range of their whole text;
- red pieces: one per (q, verse); start 0 if the q opened in an earlier
  verse, end = verse length if it continues;
- words: every <w> in verse text, its range and its strong: keys (canonical:
  letter + number without leading zeros), lemma.TR: tokens skipped, and the
  one reviewed anomaly (2Cor.8.10, token G5129) skipped.

kjv-q.tsv uses a structurally different method for the red text: the source
bytes between the q's start and end tags, minus <note>...</note>, minus every
tag, minus every LF (between verses there is exactly one LF; verse text
contains none). Its byte offsets can be checked with a hex viewer.
"""
import hashlib
import os
import re
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from osis_tokens import Refused, load_pinned, strict_tokenize  # noqa: E402

ANOMALY = ("2Cor.8.10", "strong:G1722 G5129 lemma.TR:εν lemma.TR:τουτω", "G5129")
STRONG = re.compile(r"strong:([HG])([0-9]{1,5})\Z")
NOTE = re.compile(rb"<note\b[^>]*>.*?</note>", re.S)
TAG = re.compile(rb"<[^>]*>")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def file_sha(path):
    with open(path, "rb") as f:
        return sha(f.read())


def keys_of(lemma, verse):
    out = []
    for tok in lemma.split(" "):
        if tok == "":
            raise Refused("empty lemma token in %s" % verse)
        if tok.startswith("lemma.TR:"):
            continue
        m = STRONG.match(tok)
        if m:
            n = int(m.group(2))
            if n == 0:
                raise Refused("Strong's number 0 in %s" % verse)
            out.append("%s%d" % (m.group(1), n))
            continue
        if (verse, lemma, tok) == ANOMALY:
            continue
        raise Refused("unexpected lemma token %r in %s" % (tok, verse))
    return out


def walk(data, toks):
    """Return (verses, qs). verses: list of dicts in document order."""
    verses = []
    qs = []  # dicts: q_ord, book, start, end
    cur = None
    note = 0
    q = None  # index into qs
    piece_start = None
    marks_open = []
    word_open = None
    book = None
    for t in toks:
        if t.kind == "text":
            if cur is not None and note == 0:
                s = data[t.start:t.end].decode("utf-8")
                cur["parts"].append(s)
                cur["len"] += len(s)
            continue
        name = t.name
        if name == "div" and t.kind == "start" and t.attrs.get("type") == "book":
            book = t.attrs["osisID"]
            continue
        if name == "verse":
            if "sID" in t.attrs:
                cur = {"ref": t.attrs["sID"], "parts": [], "len": 0,
                       "marks": [], "red": [], "words": []}
                if q is not None:
                    piece_start = 0
            else:
                if q is not None:
                    if piece_start >= cur["len"]:
                        raise Refused("empty red piece at end of %s" % cur["ref"])
                    cur["red"].append((piece_start, cur["len"], qs[q]["q_ord"]))
                    piece_start = None
                cur["text"] = "".join(cur["parts"])
                verses.append(cur)
                cur = None
            continue
        if cur is None:
            if name == "q":
                raise Refused("q outside a verse at byte %d" % t.start)
            continue
        if name == "note":
            note += 1 if t.kind == "start" else -1
            continue
        if note:
            continue
        if name in ("transChange", "divineName"):
            if t.kind == "start":
                kind = "added" if name == "transChange" else "divine"
                if name == "transChange" and t.attrs.get("type") != "added":
                    raise Refused("transChange type %r in verse text" % t.attrs.get("type"))
                cur["marks"].append([cur["len"], None, kind])
                marks_open.append(cur["marks"][-1])
            else:
                mk = marks_open.pop()
                mk[1] = cur["len"]
                if mk[0] == mk[1]:
                    raise Refused("empty mark in %s" % cur["ref"])
        elif name == "w":
            if t.kind == "end":
                word_open[1] = cur["len"]
                word_open = None
            else:
                if word_open is not None:
                    raise Refused("nested w in %s" % cur["ref"])
                wd = [cur["len"], cur["len"], keys_of(t.attrs["lemma"], cur["ref"])]
                cur["words"].append(wd)
                if t.kind == "start":
                    word_open = wd
        elif name == "q":
            if t.kind == "start":
                qs.append({"q_ord": len(qs) + 1, "book": book, "start": t.start, "end": None})
                q = len(qs) - 1
                piece_start = cur["len"]
            else:
                if piece_start >= cur["len"]:
                    raise Refused("empty red piece in %s" % cur["ref"])
                cur["red"].append((piece_start, cur["len"], qs[q]["q_ord"]))
                qs[q]["end"] = t.end
                q = None
                piece_start = None
    return verses, qs


def main(src, sumfile, out_dir):
    if os.path.basename(src) != "kjv.osis.xml":
        raise Refused("expected kjv.osis.xml")
    data = load_pinned(src, sumfile)
    src_sha = sha(data)
    toks = strict_tokenize(data)
    verses, qs = walk(data, toks)
    if len(verses) != 31102 or len(qs) != 648:
        raise Refused("unexpected totals: %d verses, %d q" % (len(verses), len(qs)))

    header = [
        "# generator: docs/pipeline/survey-scripts/oracle_kjv.py sha256 %s" % file_sha(os.path.join(HERE, "oracle_kjv.py")),
        "# helper: docs/pipeline/survey-scripts/osis_tokens.py sha256 %s" % file_sha(os.path.join(HERE, "osis_tokens.py")),
        "# source: BIB-KJV-CW31 kjv.osis.xml sha256 %s" % src_sha,
        "# python: %d.%d.%d" % sys.version_info[:3],
        "# Generated by an independent implementation (DESIGN.md 11.1). Never edit by hand;",
        "# regenerate only when the source pin or a generator changes, with the owner's review.",
    ]

    # kjv-chapters.tsv
    chapters = []  # (ref, [verses])
    for v in verses:
        b, c, n = v["ref"].split(".")
        key = "%s.%s" % (b, c)
        if not chapters or chapters[-1][0] != key:
            chapters.append((key, []))
        chapters[-1][1].append(v)
    lines = ["# kjv-chapters.tsv: one line per chapter (1,189)."] + header
    lines.append("# columns: chapter\tverses\ttext_sha256\tmarks\tred\twords_sha256")
    lines.append("# text_sha256: SHA-256 of the chapter's stored verse texts joined with LF (UTF-8).")
    lines.append("# marks: verse:start-end:kind; red: verse:start-end:q_ord (code points, space-separated, '-' if none).")
    lines.append("# words_sha256: SHA-256 of 'verse:n:start-end:key+key...' lines joined with LF.")
    for key, vs in chapters:
        text = "\n".join(v["text"] for v in vs).encode("utf-8")
        marks = []
        red = []
        words = []
        for v in vs:
            vn = v["ref"].split(".")[2]
            for s, e, k in v["marks"]:
                marks.append("%s:%d-%d:%s" % (vn, s, e, k))
            for s, e, qo in v["red"]:
                red.append("%s:%d-%d:%d" % (vn, s, e, qo))
            for i, (s, e, ks) in enumerate(v["words"]):
                words.append("%s:%d:%d-%d:%s" % (vn, i, s, e, "+".join(ks)))
        lines.append("\t".join([key, str(len(vs)), sha(text), " ".join(marks) or "-",
                                " ".join(red) or "-", sha("\n".join(words).encode("utf-8"))]))
    if len(chapters) != 1189:
        raise Refused("unexpected chapter count %d" % len(chapters))
    chapters_out = "\n".join(lines) + "\n"

    # kjv-q.tsv: tag stripping, structurally different from the walk above.
    by_q = {}
    for v in verses:
        for s, e, qo in v["red"]:
            by_q.setdefault(qo, []).append(v["ref"])
    qlines = ["# kjv-q.tsv: one line per <q> (648)."] + header
    qlines.append("# columns: q_ord\tbook\tstart_byte\tend_byte\tverses\tfirst_verse\tlast_verse\tred_cp\tred_sha256")
    qlines.append("# start_byte: 0-based offset of the start tag's '<'; end_byte: offset of the byte after the end tag's '>'.")
    qlines.append("# red text by tag stripping: bytes between the tags, minus <note>...</note>, minus every tag, minus every LF.")
    total_cp = 0
    for q in qs:
        s, e = q["start"], q["end"]
        open_tag = data[s:data.index(b">", s) + 1]
        if open_tag != b'<q who="Jesus" marker="">' or data[e - 4:e] != b"</q>":
            raise Refused("q %d tags not where expected" % q["q_ord"])
        seg = data[s + len(open_tag):e - 4]
        seg = NOTE.sub(b"", seg)
        if b"<note" in seg or b"</note>" in seg:
            raise Refused("unbalanced note inside q %d" % q["q_ord"])
        seg = TAG.sub(b"", seg).replace(b"\n", b"")
        if b"<" in seg or b">" in seg:
            raise Refused("stray angle bracket in q %d" % q["q_ord"])
        red = seg.decode("utf-8")
        total_cp += len(red)
        refs = by_q[q["q_ord"]]
        qlines.append("\t".join([str(q["q_ord"]), q["book"], str(s), str(e), str(len(refs)),
                                 refs[0], refs[-1], str(len(red)), sha(seg)]))
    if total_cp != 210502:
        raise Refused("tag-stripping red total %d, expected 210,502" % total_cp)
    q_out = "\n".join(qlines) + "\n"

    for name, text in (("kjv-chapters.tsv", chapters_out), ("kjv-q.tsv", q_out)):
        with open(os.path.join(out_dir, name), "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
    print("wrote kjv-chapters.tsv (%d chapters) and kjv-q.tsv (%d q, %d red code points); source sha256 %s"
          % (len(chapters), len(qs), total_cp, src_sha))


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:4])
