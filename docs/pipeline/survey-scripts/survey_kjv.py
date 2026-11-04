"""Structural survey of the CrossWire KJV OSIS file (read-only, regex tokenizer).

Usage: py -I -B survey_kjv.py <kjv.osis.xml> <BIB-KJV-CW31.sha256> <out_dir>
Refuses to run unless the source's SHA-256 matches the checksum file and the
whole file passes the strict walker (osis_tokens.strict_tokenize). Only then
writes survey.txt, verses.tsv (ref, text-candidate lengths) and
chapter_counts.tsv to out_dir.
"""
import json
import os
import sys
from collections import Counter, defaultdict

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from osis_tokens import load_pinned, strict_tokenize  # noqa: E402

sys.stdout.reconfigure(encoding="utf-8", newline="\n")

EXCLUDED_SUBTREES = {"note"}  # candidate rule: text inside these is not verse text


def main(path, sumfile, out_dir):
    data = load_pinned(path, sumfile)
    toks = strict_tokenize(data)
    lines = []  # the report is written only after the whole walk succeeds

    def p(*a):
        lines.append(" ".join(str(x) for x in a))

    p("tokens:", len(toks))
    stack = []  # list of (name, attrs, tok_index)
    book = None
    chapter = None
    verse = None  # current open verse osisID
    verse_start_byte = None
    verse_open_stack = None
    books = []
    chapters = []
    verses = []  # dicts
    verse_by_chapter = defaultdict(list)
    problems = []

    # q tracking
    q_open = []  # stack of dicts
    q_spans = []
    # inventory
    in_verse_elems = Counter()
    in_verse_paths = Counter()
    outside_verse_elems = Counter()
    outside_verse_text = Counter()
    open_at_verse_start = Counter()
    open_at_verse_end = Counter()
    verse_parent = Counter()
    attrs_by_elem = defaultdict(Counter)
    q_contents_elems = Counter()
    text_in_verse_ws_only = 0
    text_in_verse_total = 0
    note_text_bytes = 0
    excluded_depth = 0
    cur_text = []  # bytes of candidate verse text (current verse)
    cur_raw_start = None
    q_spans_per_verse = defaultdict(list)  # ref -> list of [start,end] in candidate text bytes
    active_q_start_in_verse = None

    def in_verse():
        return verse is not None

    for i, t in enumerate(toks):
        if t.kind == "text":
            raw = data[t.start:t.end]
            if in_verse():
                text_in_verse_total += 1
                if raw.strip() == b"":
                    text_in_verse_ws_only += 1
                if excluded_depth:
                    note_text_bytes += len(raw)
                else:
                    cur_text.append(raw)
            else:
                if raw.strip():
                    ctx = "/".join(s[0] for s in stack[-3:])
                    outside_verse_text[ctx] += 1
            continue

        name = t.name
        if t.kind in ("start", "empty"):
            for k, v in t.attrs.items():
                if k in ("osisID", "sID", "eID", "osisRef", "n", "src", "lemma", "morph", "chapterTitle"):
                    attrs_by_elem[name][k] += 1
                else:
                    attrs_by_elem[name]["%s=%s" % (k, v)] += 1

        if name == "verse":
            if t.kind != "empty":
                problems.append("non-milestone verse at %d" % t.start)
                continue
            if "sID" in t.attrs:
                if verse is not None:
                    problems.append("verse %s opened inside %s" % (t.attrs["sID"], verse))
                if t.attrs.get("osisID") != t.attrs["sID"]:
                    problems.append("osisID != sID at %s" % t.attrs["sID"])
                verse = t.attrs["sID"]
                verse_start_byte = t.end
                verse_open_stack = [s[0] for s in stack]
                verse_parent[stack[-1][0] if stack else None] += 1
                for s in stack:
                    open_at_verse_start[s[0]] += 1
                cur_text = []
                for q in q_open:
                    q.setdefault("verses", []).append(verse)
                if q_open:
                    active_q_start_in_verse = 0
                else:
                    active_q_start_in_verse = None
                b, c, v = verse.split(".")
                if chapter != "%s.%s" % (b, c):
                    problems.append("verse %s outside its chapter %s" % (verse, chapter))
            elif "eID" in t.attrs:
                if verse != t.attrs["eID"]:
                    problems.append("eID %s does not close %s" % (t.attrs["eID"], verse))
                for s in stack:
                    open_at_verse_end[s[0]] += 1
                close_stack = [s[0] for s in stack]
                text = b"".join(cur_text)
                if q_open and active_q_start_in_verse is not None:
                    q_spans_per_verse[verse].append([active_q_start_in_verse, len(text)])
                verses.append({
                    "ref": verse,
                    "raw": (verse_start_byte, t.start),
                    "text": text,
                    "open_stack": verse_open_stack,
                    "close_stack": close_stack,
                })
                verse_by_chapter[chapter].append(verse)
                verse = None
                active_q_start_in_verse = None
            else:
                problems.append("verse without sID/eID at %d" % t.start)
            continue

        if in_verse():
            if t.kind in ("start", "empty"):
                in_verse_elems[name] += 1
                path_ = "/".join([s[0] for s in stack if s[0] not in ("osis", "osisText", "div", "chapter")] + [name])
                in_verse_paths[path_] += 1
        else:
            if t.kind in ("start", "empty"):
                outside_verse_elems["%s (parent %s)" % (name, stack[-1][0] if stack else None)] += 1

        if t.kind == "start":
            if name == "div" and t.attrs.get("type") == "book":
                book = t.attrs["osisID"]
                books.append(book)
            if name == "chapter":
                chapter = t.attrs["osisID"]
                chapters.append(chapter)
                if verse is not None:
                    problems.append("chapter start inside verse %s" % verse)
            if name in EXCLUDED_SUBTREES:
                excluded_depth += 1
            if name == "q":
                if excluded_depth:
                    problems.append("q inside note at %d" % t.start)
                for q in q_open:
                    problems.append("NESTED q at byte %d (outer opened in %s)" % (t.start, q["start_verse"]))
                text_so_far = len(b"".join(cur_text)) if in_verse() else None
                q_open.append({
                    "start_byte": t.start,
                    "start_verse": verse,
                    "start_chapter": chapter,
                    "prev_verse": verses[-1]["ref"] if verses else None,
                    "verses": [verse] if verse else [],
                    "attrs": t.attrs,
                })
                if in_verse():
                    active_q_start_in_verse = text_so_far
            stack.append((name, t.attrs, i))
        elif t.kind == "end":
            if not stack or stack[-1][0] != name:
                raise ValueError("mismatched end tag %s at %d (open: %s)" % (name, t.start, stack[-1][0] if stack else None))
            stack.pop()
            if name in EXCLUDED_SUBTREES:
                excluded_depth -= 1
            if name == "chapter":
                if verse is not None:
                    problems.append("chapter end inside verse %s" % verse)
            if name == "q":
                q = q_open.pop()
                q["end_byte"] = t.end
                q["end_verse"] = verse
                q["end_chapter"] = chapter
                q["next_after_end_is_verse_end"] = None
                q_spans.append(q)
                if in_verse() and active_q_start_in_verse is not None:
                    q_spans_per_verse[verse].append([active_q_start_in_verse, len(b"".join(cur_text))])
                active_q_start_in_verse = None
        # inventory of elements inside q
        if q_open and t.kind in ("start", "empty") and name != "q":
            q_contents_elems[name] += 1

    if stack:
        problems.append("unclosed at EOF: %s" % [s[0] for s in stack])

    # ---------------- report ----------------
    p("books:", len(books))
    p("chapters:", len(chapters))
    p("verses:", len(verses))
    p("problems:", len(problems))
    for pr in problems[:100]:
        p("   ", pr)
    # verse numbering continuity
    gaps = []
    for ch, vs in verse_by_chapter.items():
        nums = [int(v.split(".")[2]) for v in vs]
        if nums != list(range(1, len(nums) + 1)):
            gaps.append((ch, nums[:5], len(nums)))
    p("chapters whose verses are not exactly 1..n in order:", len(gaps), gaps[:10])
    chnums = defaultdict(list)
    for ch in chapters:
        b, c = ch.split(".")
        chnums[b].append(int(c))
    badch = [b for b, cs in chnums.items() if cs != list(range(1, len(cs) + 1))]
    p("books whose chapters are not exactly 1..n:", badch)
    p("verse parent element counts:", dict(verse_parent))
    p("elements open at verse sID:", dict(open_at_verse_start))
    p("elements open at verse eID:", dict(open_at_verse_end))
    stacks = Counter(("/".join(v["open_stack"]), "/".join(v["close_stack"])) for v in verses)
    p("distinct (open stack at sID, at eID):")
    for k, n in stacks.most_common():
        p("   %6d  %s  ->  %s" % (n, k[0], k[1]))
    p("elements inside verses:", dict(in_verse_elems.most_common()))
    p("element paths inside verses (excluding osis/osisText/div/chapter):")
    for k, n in in_verse_paths.most_common():
        p("   %7d %s" % (n, k))
    p("elements outside verses:")
    for k, n in outside_verse_elems.most_common():
        p("   %7d %s" % (n, k))
    p("non-whitespace text outside verses, by context (last 3 open elements):")
    for k, n in outside_verse_text.most_common():
        p("   %7d %s" % (n, k))
    p("text nodes inside verses:", text_in_verse_total, "whitespace-only:", text_in_verse_ws_only)
    p("bytes of text inside <note> within verses:", note_text_bytes)
    p("attributes by element:")
    for e, c in sorted(attrs_by_elem.items()):
        p("   ", e, dict(c.most_common(30)))
    p("elements inside q:", dict(q_contents_elems.most_common()))

    # q statistics
    p("\n== q who=Jesus ==")
    p("q spans:", len(q_spans))
    attrsets = Counter(json.dumps(q["attrs"], sort_keys=True) for q in q_spans)
    p("attribute sets:", dict(attrsets))
    opened_outside_verse = [q for q in q_spans if q["start_verse"] is None]
    closed_outside_verse = [q for q in q_spans if q["end_verse"] is None]
    p("q opened between verses (not inside a verse):", len(opened_outside_verse))
    for q in opened_outside_verse[:20]:
        p("    opened after", q["prev_verse"], "context bytes:", data[q["start_byte"] - 40:q["start_byte"] + 60])
    p("q closed between verses:", len(closed_outside_verse))
    for q in closed_outside_verse[:20]:
        p("    closed; context:", data[q["end_byte"] - 60:q["end_byte"] + 40])
    multi = [q for q in q_spans if len(set(q["verses"])) > 1]
    p("q spans covering more than one verse:", len(multi))
    crosschap = [q for q in q_spans if len({v.rsplit(".", 1)[0] for v in q["verses"]}) > 1]
    p("q spans covering more than one chapter:", len(crosschap))
    longest = sorted(q_spans, key=lambda q: -len(set(q["verses"])))[:10]
    p("longest q spans (verses):")
    for q in longest:
        vs = q["verses"]
        p("    %s .. %s (%d verses)" % (vs[0], vs[-1], len(set(vs))))
    perbook_q = Counter()
    for q in q_spans:
        first = q["verses"][0] if q["verses"] else q["prev_verse"]
        perbook_q[first.split(".")[0]] += 1
    p("q elements per book (book of first verse):", dict(perbook_q))

    # per-verse spans
    per_book_spans = Counter()
    per_book_bytes = Counter()
    per_book_chars = Counter()
    per_book_u16 = Counter()
    per_book_red_verses = Counter()
    empty_spans = []
    ws_edge = Counter()
    for v in verses:
        sp = q_spans_per_verse.get(v["ref"], [])
        b = v["ref"].split(".")[0]
        txt = v["text"]
        if sp:
            per_book_red_verses[b] += 1
        for s, e in sp:
            per_book_spans[b] += 1
            seg = txt[s:e]
            if not seg.strip():
                empty_spans.append((v["ref"], s, e, seg))
            per_book_bytes[b] += len(seg)
            per_book_chars[b] += len(seg.decode("utf-8"))
            per_book_u16[b] += len(seg.decode("utf-8").encode("utf-16-le")) // 2
            if seg[:1].isspace():
                ws_edge["leading ws"] += 1
            if seg[-1:].isspace():
                ws_edge["trailing ws"] += 1
            if s == 0:
                ws_edge["starts at verse text start"] += 1
            if e == len(txt):
                ws_edge["ends at verse text end"] += 1
    p("per-verse red spans total:", sum(per_book_spans.values()))
    p("red spans that are empty or whitespace-only:", len(empty_spans), empty_spans[:20])
    p("span edge facts:", dict(ws_edge))
    multi_span_verses = [(r, s) for r, s in q_spans_per_verse.items() if len(s) > 1]
    p("verses with >1 red span:", len(multi_span_verses), [r for r, _ in multi_span_verses][:40])
    p("per book: red verses / spans / UTF-8 bytes / scalars / UTF-16 units")
    for b in books:
        if per_book_spans[b]:
            p("    %-6s %5d %5d %8d %8d %8d" % (b, per_book_red_verses[b], per_book_spans[b], per_book_bytes[b], per_book_chars[b], per_book_u16[b]))
    ot = set(books[:39])
    p("red spans in OT books:", sum(per_book_spans[b] for b in ot))

    # verse text facts
    p("\n== candidate verse text (all text nodes in verse, minus <note> subtrees) ==")
    lead = sum(1 for v in verses if v["text"][:1].isspace())
    trail = sum(1 for v in verses if v["text"][-1:].isspace())
    empty = [v["ref"] for v in verses if not v["text"].strip()]
    p("verses with leading whitespace:", lead, "trailing whitespace:", trail, "empty:", empty[:20])
    dbl = sum(1 for v in verses if b"  " in v["text"])
    nl = [v["ref"] for v in verses if b"\n" in v["text"] or b"\t" in v["text"] or b"\r" in v["text"]]
    p("verses containing double space:", dbl, "containing newline/tab/CR:", len(nl), nl[:20])
    na = Counter()
    na_verses = Counter()
    for v in verses:
        s = v["text"].decode("utf-8")
        for ch in set(ch for ch in s if ord(ch) > 127):
            na_verses[ch] += 1
        for ch in s:
            if ord(ch) > 127:
                na[ch] += 1
    p("non-ASCII chars in verse text:", {("U+%04X %s" % (ord(k), k)): n for k, n in na.most_common()})
    astral = sum(1 for v in verses if any(ord(c) > 0xFFFF for c in v["text"].decode("utf-8")))
    p("verses containing chars outside the BMP:", astral)
    ascii_only = sum(1 for v in verses if max(v["text"]) < 128) if verses else 0
    p("verses whose candidate text is pure ASCII:", ascii_only, "of", len(verses))
    total_bytes = sum(len(v["text"]) for v in verses)
    p("total candidate verse text bytes:", total_bytes)

    with open(os.path.join(out_dir, "survey.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write("".join(line + "\n" for line in lines))
    with open(os.path.join(out_dir, "verses.tsv"), "w", encoding="utf-8", newline="\n") as f:
        for v in verses:
            f.write("%s\t%d\t%d\t%s\n" % (v["ref"], v["raw"][0], v["raw"][1], json.dumps(q_spans_per_verse.get(v["ref"], []))))
    with open(os.path.join(out_dir, "chapter_counts.tsv"), "w", encoding="utf-8", newline="\n") as f:
        for ch in chapters:
            f.write("%s\t%d\n" % (ch, len(verse_by_chapter[ch])))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3])
