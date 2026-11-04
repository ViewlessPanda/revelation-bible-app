// T21 (highlights-pen.md 10): range highlights never change the text. Every
// real-KJV golden chapter, plus a Hebrew/Greek chapter with combining marks,
// is rendered without marks and with a stress set of marks painted, and:
// 1. every .vtext's outerHTML is byte-identical, and so are .vnum and the
//    screen-reader label apart from the ", pen highlight" suffix;
// 2. the verse text (what the copy handler reads) equals the stored text;
// 3. every .hl-ink / .hl-box is empty, aria-hidden, pointer-events and
//    user-select none, and outside every .vtext;
// 4. T20 passes unchanged with marks on (red on and off, every state);
// 5. no layout shift: every layout property of .verse, .vtext and inside is
//    the same with and without marks, and in pen mode;
// 7. the CSS lint holds the paint layer to its property list.
// Harness: hover-pass.ts (happy-dom). happy-dom computes no layout, so
// assertion 6 (box geometry over glyph rects) is covered by the pure 7.5
// functions in pen.test.ts; a real-browser run is the W2-1 gap. Range rects
// are stubbed here so that boxes are actually created and written.
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { kjv, type KChapter, type KVerse } from "./kjv-golden";
import { APP, FORCED, layoutOf, renderChapter, restPass, setCss, statePass, unrender, useHarness, type State } from "./hover-pass";
import { paintInk } from "../src/lib/ink";
import { cpLen, drawnPieces, graphemeBounds, type MarkOut } from "../src/lib/pen";
import { COLORS } from "../src/lib/types";
import { lintCss } from "../scripts/csslint.mjs";

useHarness();

// happy-dom has no layout: give every Range one rect from its offsets, so the
// paint layer measures, groups and writes real boxes
const proto = Range.prototype as Range & { getClientRects: () => DOMRectList };
const original = proto.getClientRects;
beforeAll(() => {
  proto.getClientRects = function (this: Range) {
    const a = this.startOffset * 8, b = this.endOffset * 8;
    return [{ left: a, right: b, top: 0, bottom: 16, width: b - a, height: 16, x: a, y: 0 }] as unknown as DOMRectList;
  };
});
afterAll(() => { proto.getClientRects = original; });

/** A deterministic PRNG (a fixed seed per chapter). */
function rng(seed: number) {
  let s = seed >>> 0 || 1;
  return () => { s = (Math.imul(s, 1103515245) + 12345) >>> 0; return s / 2 ** 32; };
}

/** Code-point offsets where runs (red, add, nd, w) begin or end. */
function runEdges(v: KVerse): number[] {
  const out: number[] = [];
  let cp = 0;
  for (const r of v.runs) { out.push(cp); cp += cpLen(r.text); }
  out.push(cp);
  return out;
}

/**
 * The stress set (10): (a) marks at random cluster boundaries in all five
 * colours, (b) ends at and across .wj/.add/.nd/.w edges, (c) three
 * cross-verse marks, one over a whole red passage, (d) butting different
 * colours. Non-overlapping, as the core stores them.
 */
function stressMarks(ch: KChapter): MarkOut[] {
  const r = rng(ch.book * 1000 + ch.chapter);
  const marks: MarkOut[] = [];
  let id = 1;
  const add = (color: (typeof COLORS)[number], pieces: { verse: number; start: number; end: number }[]) => {
    if (pieces.length) marks.push({ id: id++, color, created_at: id, pieces: pieces.map((p) => ({ ...p, stale: false })) });
  };
  const vs = ch.verses;
  const used = new Set<number>();
  // (c) cross-verse: the first three verses whole as one mark
  if (vs.length >= 4) {
    add("blue", vs.slice(0, 3).map((v) => ({ verse: v.v, start: 0, end: cpLen(v.text) })));
    vs.slice(0, 3).forEach((v) => used.add(v.v));
  }
  // (c) over a whole red passage: from the first red run's start to the last's end (and on into the next verse)
  const red = vs.findIndex((v) => !used.has(v.v) && v.runs.some((x) => x.red));
  if (red >= 0 && red + 1 < vs.length) {
    const v = vs[red];
    let cp = 0, s = -1, e = -1;
    for (const x of v.runs) { if (x.red && s < 0) s = cp; cp += cpLen(x.text); if (x.red) e = cp; }
    add("rose", [{ verse: v.v, start: s, end: cpLen(v.text) }, { verse: vs[red + 1].v, start: 0, end: cpLen(vs[red + 1].text) }]);
    used.add(v.v); used.add(vs[red + 1].v);
    if (s > 0) add("gold", [{ verse: v.v, start: 0, end: s }]); // (d) butting the red passage
  }
  for (const v of vs) {
    if (used.has(v.v)) continue;
    const g = graphemeBounds(v.text);
    const len = cpLen(v.text);
    if (len < 2) continue;
    // (a) + (b): cut points from random cluster boundaries and run edges
    const edges = runEdges(v).filter((x) => g.includes(x));
    const pool = new Set<number>([0, len]);
    for (let i = 0; i < 4; i++) pool.add(g[Math.floor(r() * g.length)]);
    for (const x of edges) if (r() < 0.5) pool.add(x);
    const cuts = [...pool].sort((a, b) => a - b);
    for (let i = 0; i + 1 < cuts.length; i++) {
      // most segments marked, so neighbours butt in different colours (d); some left plain
      if (r() < 0.2) continue;
      add(COLORS[(v.v + i) % COLORS.length], [{ verse: v.v, start: cuts[i], end: cuts[i + 1] }]);
    }
  }
  return marks;
}

/** (f) Hebrew and Greek with combining marks (the T12 B1 kind): points, accents, polytonic. */
const HEB = "בְּרֵאשִׁית בָּרָא אֱלֹהִים";
const GRK = "Ἐν ἀρχῇ ἦν ὁ λόγος é \u{1D49C}";
const plain = (text: string) => [{ text, red: false, add: false, nd: false }];
const combining: KChapter = {
  book: kjv.chapters[0].book, osis: kjv.chapters[0].osis, name: kjv.chapters[0].name, chapter: 99,
  verses: [HEB, GRK, HEB + " " + GRK, GRK, HEB].map((text, i) => ({
    v: i + 1, id: i + 1, text, red: [], add: [], nd: [], words: [], runs: plain(text),
  })),
};

const CHAPTERS: [string, KChapter][] = [
  ...kjv.chapters.map((c) => [`${c.osis} ${c.chapter}`, c] as [string, KChapter]),
  ["Hebrew/Greek combining marks", combining],
];

interface Seen { vtext: string[]; vnum: string[]; label: string[]; text: string[]; layout: string[] }
function seen(reading: HTMLElement): Seen {
  const all = (s: string) => Array.from(reading.querySelectorAll<HTMLElement>(s));
  return {
    vtext: all(".vtext").map((e) => e.outerHTML),
    vnum: all(".vnum").map((e) => e.outerHTML),
    label: all(".verse > .sr-only").map((e) => (e.textContent ?? "").replace(", pen highlight", "")),
    text: all(".vtext").map((e) => e.textContent ?? ""),
    layout: layoutOf(reading),
  };
}

function renderWithMarks(ch: KChapter, marks: MarkOut[]) {
  const drawn = drawnPieces(marks);
  const verses = new Set(drawn.map((p) => p.verse));
  const reading = renderChapter(ch, { [ch.verses[0].v]: "gold" }, { ranges: verses, ink: verses }); // (e) whole-verse under range marks
  const chapterEl = reading;
  paintInk(chapterEl, drawn, new Map(ch.verses.map((v) => [v.v, v.text])));
  return { reading, drawn, verses };
}

describe("T21: range highlights never change the text", () => {
  it.each(CHAPTERS)("%s: verse DOM identical, layer empty and inert, T20 unchanged", (_label, ch) => {
    const marks = stressMarks(ch);
    expect(marks.length).toBeGreaterThan(0);
    setCss(FORCED);
    for (const red of ["on", "off"] as const) {
      document.documentElement.dataset.redLetters = red;
      const off = seen(renderChapter(ch, { [ch.verses[0].v]: "gold" }));
      unrender();
      const { reading, verses } = renderWithMarks(ch, marks);
      const on = seen(reading);
      // 1 and 2
      expect(on.vtext).toEqual(off.vtext);
      expect(on.vnum).toEqual(off.vnum);
      expect(on.label).toEqual(off.label);
      expect(on.text).toEqual(ch.verses.map((v) => v.text));
      expect(off.text).toEqual(on.text);
      for (const el of reading.querySelectorAll<HTMLElement>(".verse")) {
        const has = verses.has(Number(el.dataset.v));
        expect(el.hasAttribute("data-ranges")).toBe(has);
        expect(el.querySelector(".sr-only")!.textContent!.includes(", pen highlight")).toBe(has);
      }
      // 3
      const inks = Array.from(reading.querySelectorAll<HTMLElement>(".hl-ink"));
      expect(inks.length).toBe(verses.size);
      const boxes = reading.querySelectorAll<HTMLElement>(".hl-box");
      expect(boxes.length).toBeGreaterThan(0);
      for (const el of [...inks, ...boxes]) {
        expect(el.textContent).toBe("");
        expect(el.closest(".vtext")).toBeNull();
        expect(el.childNodes.length === 0 || el.classList.contains("hl-ink")).toBe(true);
      }
      for (const ink of inks) {
        expect(ink.getAttribute("aria-hidden")).toBe("true");
        expect(ink.previousElementSibling?.classList.contains("vtext")).toBe(true);
        const cs = getComputedStyle(ink);
        expect(cs.pointerEvents).toBe("none");
        expect(cs.getPropertyValue("user-select")).toBe("none");
        for (const b of ink.children) expect(b.classList.contains("hl-box") && b.childNodes.length === 0).toBe(true);
      }
      // 5: no layout shift, with marks and in pen mode
      expect(on.layout).toEqual(off.layout);
      for (const mode of ["pen", "eraser"]) {
        document.documentElement.dataset.pen = mode;
        expect(layoutOf(reading)).toEqual(off.layout);
      }
      delete document.documentElement.dataset.pen;
      // 4: T20 unchanged with marks on (the "range marks: stress set" axis)
      const rest = restPass(reading, ch, red === "on");
      for (const [name, s] of STATES) expect(statePass(reading, rest, s), `${red} ${name}`).toEqual([]);
      // and in pen mode (E5 only hides the hover line; colours and text unchanged)
      document.documentElement.dataset.pen = "pen";
      expect(restPass(reading, ch, red === "on").problems).toEqual([]);
      delete document.documentElement.dataset.pen;
      unrender();
    }
  }, 120_000);

  it("repainting replaces the boxes and an emptied verse keeps an empty layer", () => {
    const ch = kjv.chapters[0];
    const marks = stressMarks(ch);
    const { reading, drawn } = renderWithMarks(ch, marks);
    const before = reading.querySelectorAll(".hl-box").length;
    paintInk(reading, drawn, new Map(ch.verses.map((v) => [v.v, v.text])));
    expect(reading.querySelectorAll(".hl-box").length).toBe(before);
    paintInk(reading, [], new Map());
    expect(reading.querySelectorAll(".hl-box").length).toBe(0);
    expect(reading.querySelectorAll(".hl-ink").length).toBeGreaterThan(0);
    unrender();
  });
});

const STATES: [string, State][] = [
  ["hover", { hover: true, fv: false, wrapped: false }],
  ["hover + focus-visible", { hover: true, fv: true, wrapped: false }],
  ["hover, wrapped words", { hover: true, fv: false, wrapped: true }],
  ["selected", { hover: false, fv: false, wrapped: false, selected: true }],
  ["selected + focus-visible, wrapped words", { hover: false, fv: true, wrapped: true, selected: true }],
];

describe("T21 assertion 7: the CSS lint holds the paint layer", () => {
  it("the shipped stylesheet passes", () => {
    expect(lintCss(APP)).toEqual([]);
  });
  it("rejects colour, content or layout on .hl-ink / .hl-box, and extras on the fixed rules", () => {
    const bad = [
      ".hl-box { color: red; }",
      ".hl-ink { content: 'x'; }",
      ".hl-box[data-c=\"gold\"] { font-weight: 700; }",
      ".verse .hl-ink { margin-left: 2px; }",
      ".verse[data-ranges] { padding: 1px; }",
      ":root[data-pen] .vtext .w::after { display: block; }",
      ":root[data-pen] .vtext .w { color: red; }",
    ];
    for (const css of bad) expect(lintCss(APP + "\n" + css).length, css).toBeGreaterThan(0);
    const good = [
      ".hl-box { transform: translate(1px, 2px); width: 3px; height: 4px; --hl-c: red; opacity: .35; }",
      ".hl-ink { position: absolute; inset-inline-start: 0; z-index: -1; pointer-events: none; user-select: none; }",
      ".verse[data-ranges] { isolation: isolate; }",
    ];
    for (const css of good) expect(lintCss(APP + "\n" + css), css).toEqual([]);
  });
});
