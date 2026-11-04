// W2-5h: pen-mode range highlights, the pure part (highlights-pen.md 4.3,
// 6, 7.5): code points, grapheme and word snapping, cross-verse strokes,
// line boxes. The core re-checks everything (it is the gate).
import { describe, expect, it } from "vitest";
import {
  b1Ok, clickRange, cpToU16, cps, drawnPieces, edgesOf, graphemeBounds, lineBoxes, markAt, padEnds, snapshot,
  splitForErase, strokePieces, trimWs, u16ToCp, type MarkOut,
} from "../src/lib/pen";

describe("code points, never UTF-16 (W2P-B3)", () => {
  it("converts both ways around an astral character", () => {
    const s = "a\u{1D400}leph";
    expect(cps(s).length).toBe(6);
    expect(s.length).toBe(7);
    expect(cpToU16(s, 2)).toBe(3);
    expect(u16ToCp(s, 3)).toBe(2);
    expect(u16ToCp(s, 2)).toBe(1); // inside the pair: its start
    expect(cpToU16(s, 6)).toBe(7);
  });
});

describe("grapheme snapping (4.3 step 5, B1)", () => {
  it("never splits a combining mark, a Hebrew pointed letter or a ZWJ sequence", () => {
    const greek = "\u{3BB}\u{3BF}\u{301}\u{3B3}\u{3BF}\u{3C2}"; // omicron + U+0301
    expect(graphemeBounds(greek)).toEqual([0, 1, 3, 4, 5, 6]);
    const hebrew = "\u{5D1}\u{5BC}\u{5B0}\u{5E8}\u{5B5}"; // bet dagesh sheva, resh tsere
    expect(graphemeBounds(hebrew, "he")).toEqual([0, 3, 5]);
    const family = "a\u{1F468}\u{200D}\u{1F469}b";
    expect(graphemeBounds(family)).toEqual([0, 1, 4, 5]);
    // a letter-mode click on the bare omicron takes the whole cluster
    expect(clickRange(greek, "el", 1, false)).toEqual([1, 3]);
    expect(clickRange(hebrew, "he", 1, false)).toEqual([0, 3]);
    // the B1 check the core applies
    expect(b1Ok(cps(greek), 2)).toBe(false);
    expect(b1Ok(cps(greek), 3)).toBe(true);
    expect(b1Ok(cps(family), 2)).toBe(false);
  });

  it("letter-mode strokes round the start down and the end up to whole clusters", () => {
    const v = [{ v: 1, text: "cafe\u{301} au lait" }];
    // a drag from inside the e + acute cluster (before the acute) to inside "au"
    expect(strokePieces(v, { verse: 1, offset: 4 }, { verse: 1, offset: 7 }, "en", false)).toEqual([{ verse: 1, start: 3, end: 7 }]);
    expect(strokePieces(v, { verse: 1, offset: 1 }, { verse: 1, offset: 4 }, "en", false)).toEqual([{ verse: 1, start: 1, end: 5 }]);
  });
});

describe("word snapping (4.3 step 6) and trimming (step 7)", () => {
  const v = [{ v: 1, text: "In the beginning God created the heaven and the earth." }];
  it("snaps both ends out to whole words and trims spaces", () => {
    // from the middle of "beginning" to the middle of "created"
    expect(strokePieces(v, { verse: 1, offset: 9 }, { verse: 1, offset: 22 }, "en", true)).toEqual([{ verse: 1, start: 7, end: 28 }]);
    // starting on a space moves up to the next word; ending on a space moves down
    expect(strokePieces(v, { verse: 1, offset: 6 }, { verse: 1, offset: 17 }, "en", true)).toEqual([{ verse: 1, start: 7, end: 16 }]);
  });
  it("is the same mark whichever way the drag went", () => {
    const a = { verse: 1, offset: 9 }, b = { verse: 1, offset: 22 };
    expect(strokePieces(v, b, a, "en", true)).toEqual(strokePieces(v, a, b, "en", true));
  });
  it("includes punctuation only when the pointer passed over it", () => {
    const t = v[0].text;
    const end = t.length; // after the full stop
    expect(strokePieces(v, { verse: 1, offset: 48 }, { verse: 1, offset: end }, "en", true)).toEqual([{ verse: 1, start: 48, end }]);
    expect(strokePieces(v, { verse: 1, offset: 48 }, { verse: 1, offset: end - 1 }, "en", true)).toEqual([{ verse: 1, start: 48, end: end - 1 }]);
  });
  it("a click takes the word under it; whitespace is a no-op", () => {
    expect(clickRange(v[0].text, "en", 18, true)).toEqual([17, 20]); // "God"
    expect(clickRange(v[0].text, "en", 16, true)).toBeNull();
    expect(trimWs("  a b  ", 0, 7)).toEqual([2, 5]);
    expect(trimWs("   ", 0, 3)).toBeNull();
  });
});

describe("cross-verse strokes (4.3 Pieces)", () => {
  const verses = [
    { v: 1, text: "In the beginning was the Word, and the Word was with God, and the Word was God. " },
    { v: 2, text: "The same was in the beginning with God." },
    { v: 3, text: "All things were made by him." },
  ];
  it("one piece per verse: the first to its end, whole verses between, the last from 0", () => {
    const p = strokePieces(verses, { verse: 1, offset: 66 }, { verse: 3, offset: 5 }, "en", true);
    expect(p).toEqual([
      { verse: 1, start: 66, end: Array.from(verses[0].text).length },
      { verse: 2, start: 0, end: Array.from(verses[1].text).length },
      { verse: 3, start: 0, end: 10 },
    ]);
  });
  it("a start in the trailing space of a verse drops that verse's piece", () => {
    const len1 = Array.from(verses[0].text).length;
    expect(strokePieces(verses, { verse: 1, offset: len1 - 1 }, { verse: 2, offset: 3 }, "en", true)).toEqual([{ verse: 2, start: 0, end: 3 }]);
  });
  it("a spot in another chapter's verse is no stroke (clamped by the caller)", () => {
    expect(strokePieces(verses, { verse: 1, offset: 0 }, { verse: 9, offset: 0 }, "en", true)).toEqual([]);
  });
});

describe("marks, snapshots and the eraser preview", () => {
  const marks: MarkOut[] = [
    { id: 7, color: "gold", created_at: 1, pieces: [{ verse: 2, start: 4, end: 9, stale: false }, { verse: 1, start: 0, end: 3, stale: true }] },
    { id: 9, color: "blue", created_at: 2, pieces: [{ verse: 2, start: 9, end: 14, stale: false }] },
  ];
  it("stale pieces are neither drawn nor snapshotted", () => {
    expect(drawnPieces(marks).map((p) => [p.id, p.verse, p.start])).toEqual([[7, 2, 4], [9, 2, 9]]);
    expect(snapshot(marks)).toEqual([
      { color: "gold", created_at: 1, pieces: [{ verse: 2, start: 4, end: 9 }] },
      { color: "blue", created_at: 2, pieces: [{ verse: 2, start: 9, end: 14 }] },
    ]);
    expect(markAt(marks, 2, 9)?.id).toBe(9);
    expect(markAt(marks, 1, 1)).toBeNull();
  });
  it("splits drawn pieces at the eraser's range", () => {
    const parts = splitForErase(drawnPieces(marks), [{ verse: 2, start: 6, end: 11 }]);
    expect(parts.map((p) => [p.id, p.start, p.end, !!p.erasing])).toEqual([
      [7, 4, 6, false], [7, 6, 9, true], [9, 9, 11, true], [9, 11, 14, false],
    ]);
  });
  it("edges: pad beside a space or the verse edge, join at another colour, none mid-word", () => {
    const d = drawnPieces(marks);
    const text = "abc defghijklm"; // 14 code points
    expect(edgesOf(text, d[0], d)).toEqual(["pad", "join"]);
    expect(edgesOf(text, d[1], d)).toEqual(["join", "pad"]);
    expect(edgesOf("abcdefghijklmnopq", { ...d[1], color: "blue" }, [d[1]])).toEqual(["none", "none"]);
  });
});

describe("measuring (7.5)", () => {
  const r = (left: number, top: number, right: number, bottom: number) => ({ left, top, right, bottom });
  it("groups by line, joins touching rects, evens the height, pads the block", () => {
    const boxes = lineBoxes([
      r(10, 0, 40, 20), r(40.3, 2, 60, 18), // one line, touching (gap 0.3)
      r(70, 1, 90, 19),                    // same line, a real gap (bidi run)
      r(0, 30, 50, 50),                    // next line
      r(5, 5, 5, 15),                      // empty, ignored
    ], 2);
    expect(boxes).toEqual([r(10, -2, 60, 22), r(70, -2, 90, 22), r(0, 28, 50, 52)]);
  });
  it("pads the outer ends only, never at line wraps; pulls back at a join", () => {
    const boxes = [r(10, 0, 60, 20), r(0, 30, 50, 50)];
    expect(padEnds(boxes, "pad", "pad", 2, 2)).toEqual([r(8, 0, 60, 20), r(0, 30, 52, 50)]);
    expect(padEnds(boxes, "none", "join", 2, 2)).toEqual([r(10, 0, 60, 20), r(0, 30, 49, 50)]);
    expect(padEnds(boxes, "pad", "none", 2, 2, true)).toEqual([r(10, 0, 62, 20), r(0, 30, 50, 50)]);
  });
});
