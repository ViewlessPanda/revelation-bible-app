"""Detail survey of the CrossWire KJV OSIS file, for docs/pipeline/kjv-osis-survey.md.

Usage:
  py -I -B survey_kjv_detail.py <kjv.osis.xml> <BIB-KJV-CW31.sha256> <eng.vrs.txt> <VRS-ENG-PALASO.sha256> <out.txt>

Read-only. No XML parser, no DTD, no entities: the strict regex tokenizer and
strict walker in osis_tokens.py. Refuses to run if either input's SHA-256 does
not match its checksum file, or if the KJV fails the strict walker. Writes
UTF-8 with LF to <out.txt> only after both checks pass; prints nothing else.

"Candidate verse text" below = every character-data byte between a verse's
sID and eID milestones, in document order, minus the content of <note>
subtrees. That is the definition proposed in pipeline/DESIGN.md section 3.
"""
import os
import re
import sys
from collections import Counter, defaultdict

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import Refused, load_pinned, strict_tokenize  # noqa: E402

# OSIS book IDs (as used by this file) -> USFM codes (as used by eng.vrs).
OSIS_TO_USFM = dict(zip(
    "Gen Exod Lev Num Deut Josh Judg Ruth 1Sam 2Sam 1Kgs 2Kgs 1Chr 2Chr Ezra Neh "
    "Esth Job Ps Prov Eccl Song Isa Jer Lam Ezek Dan Hos Joel Amos Obad Jonah Mic "
    "Nah Hab Zeph Hag Zech Mal Matt Mark Luke John Acts Rom 1Cor 2Cor Gal Eph Phil "
    "Col 1Thess 2Thess 1Tim 2Tim Titus Phlm Heb Jas 1Pet 2Pet 1John 2John 3John "
    "Jude Rev".split(),
    "GEN EXO LEV NUM DEU JOS JDG RUT 1SA 2SA 1KI 2KI 1CH 2CH EZR NEH EST JOB PSA "
    "PRO ECC SNG ISA JER LAM EZK DAN HOS JOL AMO OBA JON MIC NAM HAB ZEP HAG ZEC "
    "MAL MAT MRK LUK JHN ACT ROM 1CO 2CO GAL EPH PHP COL 1TH 2TH 1TI 2TI TIT PHM "
    "HEB JAS 1PE 2PE 1JN 2JN 3JN JUD REV".split(),
))
assert len(OSIS_TO_USFM) == 66
# KJV overrides to VRS-ENG-PALASO (SOURCES.md, condition 2): (book, chapter) -> verses.
KJV_OVERRIDES = {("3JN", 1): 14, ("REV", 12): 17}


def read_vrs(data):
    """Book/chapter/max-verse lines for the 66 books; mapping lines ignored."""
    text = data.decode("ascii")
    want = set(OSIS_TO_USFM.values())
    table = {}
    for line in text.split("\n"):
        if not line or line.startswith("#") or "=" in line:
            continue
        parts = line.split(" ")
        if parts[0] not in want:
            continue
        if parts[0] in table:
            sys.exit("duplicate vrs line " + parts[0])
        chs = []
        for i, p in enumerate(parts[1:], 1):
            c, v = p.split(":")
            if int(c) != i:
                sys.exit("vrs chapters out of order in " + parts[0])
            chs.append(int(v))
        table[parts[0]] = chs
    if set(table) != want:
        sys.exit("vrs missing books: %s" % sorted(want - set(table)))
    return table


def show(b):
    return b.decode("utf-8").replace("\n", "\\n").replace("\t", "\\t")


def main(src, src_sum, vrs, vrs_sum, out_path):
    if os.path.basename(src) != "kjv.osis.xml" or os.path.basename(vrs) != "eng.vrs.txt":
        sys.exit("expected kjv.osis.xml and eng.vrs.txt; refusing")
    data = load_pinned(src, src_sum)
    vrs_data = load_pinned(vrs, vrs_sum)
    toks = strict_tokenize(data)
    lines = []  # the report is written only after everything succeeds

    def p(*a):
        lines.append(" ".join(str(x) for x in a))

    stack = []
    verse = None
    chapter = None
    note_depth = 0
    max_depth = 0
    cur = []          # candidate text pieces of the current verse
    cur_len = 0
    verses = []       # (ref, text bytes)
    verse_index = {}
    last_eid_end = None
    between = Counter()          # text between eID and next sID (shape)
    between_examples = {}
    lemma_kinds = Counter()
    lemma_bad = []
    strong_shapes = Counter()
    empty_w = 0
    empty_w_in_q = 0
    w_multi_strong = 0
    text_parent = Counter()      # bytes of candidate text by innermost element
    divine_texts = Counter()
    inscription_texts = []
    amplified = []
    split_w = []
    q_open = None
    q_list = []                  # dicts
    q_prev_char = Counter()
    q_next_char = Counter()
    q_first_tok = Counter()
    q_last_tok = Counter()
    ms_pos = Counter()
    ms_in_q = 0
    trailing = []
    tc_open_at_verse = 0
    contexts = defaultdict(list)  # char -> refs
    ws_chars = Counter()
    pending_q_end = None

    for i, t in enumerate(toks):
        if t.kind == "text":
            raw = data[t.start:t.end]
            if verse is not None and not note_depth:
                cur.append(raw)
                cur_len += len(raw)
                inner = next((s[0] for s in reversed(stack) if s[0] not in ("q",)), None)
                text_parent[inner] += len(raw)
                if stack and stack[-1][0] == "divineName":
                    divine_texts[raw] += 1
                if any(s[0] == "inscription" for s in stack):
                    inscription_texts.append((verse, raw))
                for ch in raw.decode("utf-8"):
                    if ch.isspace():
                        ws_chars["U+%04X" % ord(ch)] += 1
                    if ord(ch) > 127 and len(contexts[ch]) < 400:
                        contexts[ch].append(verse)
                if pending_q_end is not None and raw:
                    q_next_char[show(raw[:1])] += 1
                    pending_q_end = None
            elif verse is None and last_eid_end is not None:
                shape = show(raw) if len(raw) < 12 else "<%d bytes>" % len(raw)
                between[shape] += 1
                between_examples.setdefault(shape, toks[i - 1].attrs.get("eID"))
            continue

        name = t.name
        if name == "verse":
            if "sID" in t.attrs:
                verse = t.attrs["sID"]
                cur, cur_len = [], 0
                if any(s[0] == "transChange" for s in stack):
                    tc_open_at_verse += 1
                if q_open is not None:
                    q_open["verses"].append(verse)
                last_eid_end = None
            else:
                text = b"".join(cur)
                verse_index[verse] = len(verses)
                verses.append((verse, text))
                if text[-1:].isspace():
                    nxt = toks[i - 1]
                    prev_el = None
                    for back in range(i - 1, max(i - 40, 0), -1):
                        if toks[back].kind != "text":
                            prev_el = toks[back].name + ("/" if toks[back].kind == "end" else "")
                            break
                    trailing.append((verse, text[-30:], prev_el, nxt.kind))
                if pending_q_end is not None:
                    q_next_char["<verse eID>"] += 1
                    pending_q_end = None
                verse = None
                last_eid_end = t.end
            continue

        if t.kind in ("start", "empty"):
            if name == "w":
                lemma = t.attrs.get("lemma", "")
                toks_ = lemma.split(" ")
                n_strong = 0
                for tk in toks_:
                    if tk.startswith("strong:"):
                        lemma_kinds["strong:"] += 1
                        n_strong += 1
                        m = re.fullmatch(r"strong:([HG])(\d+)", tk)
                        if not m:
                            lemma_bad.append((verse, lemma))
                        else:
                            d = m.group(2)
                            strong_shapes["%s, %d digits, leading zero=%s" % (
                                m.group(1), len(d), d.startswith("0"))] += 1
                    elif tk.startswith("lemma.TR:"):
                        lemma_kinds["lemma.TR:"] += 1
                    elif tk == "":
                        lemma_kinds["(empty token)"] += 1
                    else:
                        lemma_kinds["other"] += 1
                        lemma_bad.append((verse, lemma))
                if n_strong > 1:
                    w_multi_strong += 1
                if t.kind == "empty":
                    empty_w += 1
                    if q_open is not None:
                        empty_w_in_q += 1
                if t.attrs.get("type", "").startswith("x-split") and len(split_w) < 400:
                    split_w.append((verse, t.attrs["type"]))
            if name == "transChange" and t.attrs.get("type") == "amplified":
                amplified.append(verse)
            if name == "milestone" and verse is not None:
                ms_pos["at verse text start" if cur_len == 0 else "mid-verse"] += 1
                if q_open is not None:
                    ms_in_q += 1
            if name == "q":
                # strict_tokenize already refused these; kept as a local guard.
                if t.kind != "start" or q_open is not None:
                    raise Refused("empty or nested q at byte %d" % t.start)
                pc = b"".join(cur)[-1:]
                q_prev_char[show(pc) if pc else "<verse sID>"] += 1
                q_open = {"start": verse, "verses": [verse], "start_off": cur_len,
                          "tok": i}
                # first content token after <q>
                j = i + 1
                q_first_tok[toks[j].kind + ":" + (toks[j].name or "")] += 1
        if t.kind == "start":
            if name == "chapter":
                chapter = t.attrs["osisID"]
            if name == "note":
                note_depth += 1
            stack.append((name, t.attrs))
            max_depth = max(max_depth, len(stack))
        elif t.kind == "end":
            if not stack or stack.pop()[0] != name:
                raise Refused("mismatched end tag </%s> at byte %d" % (name, t.start))
            if name == "note":
                note_depth -= 1
            if name == "q":
                j = i - 1
                q_last_tok[toks[j].kind + ":" + (toks[j].name or "")] += 1
                q_open["end"] = verse
                q_open["end_off"] = cur_len
                q_list.append(q_open)
                q_open = None
                pending_q_end = True

    # ---------------- report ----------------
    p("source SHA-256 verified; eng.vrs.txt SHA-256 verified")
    p("tokens:", len(toks), " max element depth:", max_depth)

    # Versification
    vt = read_vrs(vrs_data)
    p("\n== versification (VRS-ENG-PALASO + 2 KJV overrides) ==")
    raw_total = sum(sum(v) for v in vt.values())
    raw_ch = sum(len(v) for v in vt.values())
    for (b, c), n in KJV_OVERRIDES.items():
        p("override %s %d: eng.vrs says %d, KJV override %d" % (b, c, vt[b][c - 1], n))
        vt[b][c - 1] = n
    tot = sum(sum(v) for v in vt.values())
    p("eng.vrs 66 books: chapters %d verses %d; with overrides: chapters %d verses %d"
      % (raw_ch, raw_total, sum(len(v) for v in vt.values()), tot))
    got = defaultdict(lambda: defaultdict(int))
    order = []
    for ref, _ in verses:
        b, c, v = ref.split(".")
        if b not in got:
            order.append(b)
        got[b][int(c)] += 1
    p("books in file order match canonical order:", order == list(OSIS_TO_USFM))
    mism = []
    p("per book: OSIS USFM chapters verses expected match")
    for b in order:
        u = OSIS_TO_USFM[b]
        chs = [got[b][c] for c in range(1, max(got[b]) + 1)]
        ok = chs == vt[u]
        if not ok:
            mism.append(b)
        p("   %-6s %s %4d %6d %6d %s" % (b, u, len(chs), sum(chs), sum(vt[u]), "yes" if ok else "NO"))
    p("books with any chapter mismatch:", mism)
    p("file total verses:", len(verses))

    # Between verses
    p("\n== character data between a verse eID and the next start tag ==")
    for k, n in between.most_common(20):
        p("   %7d %r (first after %s)" % (n, k, between_examples[k]))

    # Verse text facts
    p("\n== candidate verse text ==")
    p("whitespace characters inside candidate verse text:", dict(ws_chars))
    p("bytes of candidate text by innermost enclosing element (q ignored):")
    for k, n in text_parent.most_common():
        p("   %9d %s" % (n, k))
    lens = sorted(((len(t.decode("utf-8")), r) for r, t in verses))
    p("shortest verses (chars):", lens[:3], " longest:", lens[-3:])
    p("verses with trailing whitespace:", len(trailing))
    why = Counter((pe, ) for _, _, pe, _ in trailing)
    p("   element just before eID for those verses:", dict(why))
    for r, tail, pe, k in trailing[:6]:
        p("   %s ...%r (last element before eID: %s)" % (r, show(tail), pe))
    p("transChange open at a verse sID (spans a verse boundary):", tc_open_at_verse)
    for ch in sorted(contexts, key=ord):
        refs = contexts[ch]
        p("U+%04X %s in verses: %d; first refs %s" % (ord(ch), ch, len(refs),
                                                      sorted(set(refs), key=refs.index)[:8]))
    p("divineName text values:", {show(k): n for k, n in divine_texts.most_common()})
    p("inscription texts:", [(r, show(x)) for r, x in inscription_texts][:20])
    p("transChange type=amplified in verses:", amplified)
    p("pilcrow milestones in verses:", dict(ms_pos), " inside q:", ms_in_q)

    # Lemma
    p("\n== w / lemma ==")
    p("lemma token kinds:", dict(lemma_kinds))
    p("strong: token shapes:", dict(strong_shapes.most_common()))
    p("w with more than one strong: token:", w_multi_strong)
    p("empty <w/> elements (Strong's with no English text):", empty_w, " inside q:", empty_w_in_q)
    p("lemma values not matching strong:[HG]digits or lemma.TR: (all):", len(lemma_bad))
    for r, lm in lemma_bad[:20]:
        p("   %s %r" % (r, lm))
    p("x-split w (first 10):", split_w[:10], " total:", len(split_w))

    # q
    p("\n== q who=Jesus ==")
    p("q total:", len(q_list))
    cross_v = [q for q in q_list if q["start"] != q["end"]]
    cross_c = [q for q in q_list if q["start"].rsplit(".", 1)[0] != q["end"].rsplit(".", 1)[0]]
    p("q whose start and end tags are in different verses:", len(cross_v))
    p("q whose start and end tags are in different chapters:", len(cross_c))
    starts_at_0 = sum(1 for q in q_list if q["start_off"] == 0)
    p("q starting at offset 0 of its verse's text:", starts_at_0)
    ends_at_end = 0
    vt_len = {r: len(t) for r, t in verses}
    for q in q_list:
        if q["end_off"] == vt_len[q["end"]]:
            ends_at_end += 1
    p("q ending at the end of its verse's text:", ends_at_end)
    p("character before <q> (or <verse sID>):", dict(q_prev_char.most_common()))
    p("character after </q> (or <verse eID>):", dict(q_next_char.most_common()))
    p("token right after <q>:", dict(q_first_tok))
    p("token right before </q>:", dict(q_last_tok))
    perbook = Counter(q["start"].split(".")[0] for q in q_list)
    perbook_x = Counter(q["start"].split(".")[0] for q in cross_v)
    p("per book: q elements / of which cross a verse boundary")
    for b in order:
        if perbook[b]:
            p("   %-6s %4d %4d" % (b, perbook[b], perbook_x[b]))
    nverses = Counter(len(q["verses"]) for q in q_list)
    p("q by number of verses touched:", dict(sorted(nverses.items())))
    with open(out_path, "w", encoding="utf-8", newline="\n") as out:
        out.write("".join(line + "\n" for line in lines))


if __name__ == "__main__":
    main(*sys.argv[1:6])
