// The T20 hover/focus pass (red-letters-and-highlights.md 3.4.6 assertion 5)
// as far as happy-dom allows; shared by hover-underline.test.ts and
// kjv-hover.test.ts (split so the slow chapter passes run in parallel).
//
// happy-dom never matches :hover / :focus-visible, computes no layout and
// ignores the pseudo-element argument of getComputedStyle. So:
// - the forced-state pass rewrites :hover and :focus-visible to attributes
//   (data-force-hover, data-force-fv) set on every .w, .vtext and .verse: the
//   analogue of DevTools' CSS.forcePseudoState;
// - "no layout shift" compares every layout-affecting computed property of
//   .verse, .vtext and everything inside between rest and each state (the CSS
//   lint additionally rejects any non-paint property in a hover/focus rule
//   that can reach verse text, tests/csslint.test.ts);
// - the ::after and ::before boxes (E1-E4) are asserted on the parsed CSS,
//   and their colour on the resolved token in every theme
//   (hover-underline.test.ts);
// - the selected pass (3.4.6 assertion 6) sets aria-expanded="true" and
//   aria-controls on every .w, as App.svelte does on the one selected word.
import { afterEach, beforeAll } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import ChapterView from "../src/components/ChapterView.svelte";
import type { Chapter, HighlightColor } from "../src/lib/types";
import { kjv, type KChapter } from "./kjv-golden";
import { checkColours, textNodes } from "./render-check";

const read = (p: string) => readFileSync(resolve(process.cwd(), p), "utf8");
export const TOKENS = read("src/styles/tokens.css");
export const APP = read("src/styles/app.css");
export const CSS = TOKENS + "\n" + APP;
export const FORCED = CSS
  .replace(/:hover(?![\w-])/g, "[data-force-hover]")
  .replace(/:focus-visible(?![\w-])/g, "[data-force-fv]");
const LAYOUT = [
  "display", "position", "top", "right", "bottom", "left", "width", "height", "box-sizing",
  "margin-top", "margin-right", "margin-bottom", "margin-left",
  "padding-top", "padding-right", "padding-bottom", "padding-left",
  "border-top-width", "border-right-width", "border-bottom-width", "border-left-width",
  "font-family", "font-size", "font-weight", "font-style", "font-variant-caps", "font-stretch",
  "letter-spacing", "word-spacing", "line-height", "white-space", "vertical-align", "text-indent",
  "text-transform", "transform", "translate", "scale", "float",
];

let styleEl: HTMLStyleElement;
let app: ReturnType<typeof mount> | null = null;

/** Register the stylesheet and per-test cleanup (call at the top of a test file). */
export function useHarness() {
  beforeAll(() => {
    styleEl = document.createElement("style");
    styleEl.textContent = CSS;
    document.head.appendChild(styleEl);
  });
  afterEach(() => {
    unrender();
    styleEl.textContent = CSS;
    const root = document.documentElement;
    root.dataset.redLetters = "on";
    delete root.dataset.theme;
  });
}

export const setCss = (css: string) => { styleEl.textContent = css; };

export function unrender() {
  if (app) unmount(app);
  app = null;
  document.body.replaceChildren();
}

/** `pen`: the verses with drawn range marks and with a paint layer (ChapterView's `ranges` / `ink`). */
export function renderChapter(ch: KChapter, highlights: Record<number, HighlightColor> = {}, pen?: { ranges: Set<number>; ink: Set<number> }): HTMLElement {
  const reading = document.createElement("div");
  reading.className = "reading";
  document.body.appendChild(reading);
  const book = kjv.books.find((b) => b.ord === ch.book)!;
  app = mount(ChapterView, {
    target: reading,
    props: {
      chapter: { book: ch.book, osis: ch.osis, chapter: ch.chapter, verses: ch.verses } as Chapter,
      book, highlights, currentVerse: null, ...(pen ?? {}),
      prevLabel: null, nextLabel: null, onPrev: () => {}, onNext: () => {},
    },
  });
  flushSync();
  return reading;
}

export const token = (name: string) =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim().toLowerCase();

interface Snap { el: HTMLElement; layout: string; color: string; decoLine: string; decoColor: string }

/** Every layout-affecting computed property of .verse, .vtext and inside, per element (no layout shift, T21 assertion 5). */
export const layoutOf = (reading: HTMLElement): string[] => snapshot(reading).map((s) => s.layout);

/** One getComputedStyle per element (happy-dom recomputes on every call). */
function snapshot(reading: HTMLElement): Snap[] {
  return Array.from(reading.querySelectorAll<HTMLElement>(".verse, .vtext, .vtext *")).map((el) => {
    const cs = getComputedStyle(el);
    return {
      el,
      // happy-dom reports a font-size as calc(1.25rem * 1) on a fresh tree and
      // as calc(20px * 1) after any attribute change; the root is 100% = 16px
      layout: LAYOUT.map((p) => `${p}=${cs.getPropertyValue(p)}`).join(";")
        .replace(/(\d*\.?\d+)rem\b/g, (_m, n: string) => `${Number(n) * 16}px`),
      color: cs.color.toLowerCase(),
      decoLine: cs.getPropertyValue("text-decoration-line"),
      decoColor: cs.getPropertyValue("text-decoration-color").trim().toLowerCase(),
    };
  });
}

/** `selected`: aria-expanded="true" + aria-controls on every .w (3.4.6 assertion 6, v0.3.1). */
export type State = { hover: boolean; fv: boolean; wrapped: boolean; selected?: boolean };
function setState(reading: HTMLElement, s: State) {
  for (const el of reading.querySelectorAll<HTMLElement>(".verse, .vtext, .vtext .w")) {
    el.toggleAttribute("data-force-hover", s.hover);
    el.toggleAttribute("data-force-fv", s.fv);
  }
  for (const w of reading.querySelectorAll<HTMLElement>(".vtext .w")) {
    w.toggleAttribute("data-wrapped", s.wrapped);
    if (s.selected) { w.setAttribute("aria-expanded", "true"); w.setAttribute("aria-controls", "p-study"); }
    else { w.removeAttribute("aria-expanded"); w.removeAttribute("aria-controls"); }
  }
}

export interface Rest { nodes: Text[]; data: string[]; snap: Snap[]; problems: string[] }

/** The rest pass: absolute colour check (red on) and the baseline for every state. */
export function restPass(reading: HTMLElement, ch: KChapter, redOn: boolean): Rest {
  const nodes = textNodes(reading);
  const verseEls = Array.from(reading.querySelectorAll<HTMLElement>(".verse"));
  // happy-dom does not compute `color: inherit` over an earlier declaration
  // (R2), so with red letters off the colour check is relative to rest only
  const problems = redOn
    ? verseEls.flatMap((el, i) => checkColours(el, ch.verses[i], token("--color-red-letter"), token("--color-text")))
    : [];
  return { nodes, data: nodes.map((t) => t.data), snap: snapshot(reading), problems };
}

/** One T20 state pass: colours, text, layout and decoration against rest. */
export function statePass(reading: HTMLElement, rest: Rest, s: State): string[] {
  const p: string[] = [...rest.problems];
  const underline = token("--color-word-underline");
  setState(reading, s);
  try {
    // display = storage: the very same text nodes, same characters
    const { nodes, data } = rest;
    const now = textNodes(reading);
    if (now.length !== nodes.length || now.some((t, i) => t !== nodes[i] || t.data !== data[i])) p.push("text nodes changed");
    const snap = snapshot(reading);
    if (snap.length !== rest.snap.length) p.push("elements changed");
    // the wrapped-word fallback: the token colour, only when hovered or
    // selected and without the focus ring
    const want = (s.hover || !!s.selected) && s.wrapped && !s.fv;
    snap.forEach((x, i) => {
      const r = rest.snap[i];
      const what = `${x.el.className} "${x.el.textContent?.slice(0, 20)}"`;
      // 2. every colour (the word's own, red runs, marks) unchanged from rest
      if (x.color !== r.color) p.push(`${what}: colour ${r.color} -> ${x.color}`);
      // no layout shift
      if (x.layout !== r.layout) p.push(`${what}: layout ${r.layout} -> ${x.layout}`);
      if (!x.el.classList.contains("w")) return;
      const underlined = /underline/.test(x.decoLine);
      if (underlined !== want) p.push(`${what}: text-decoration-line ${x.decoLine}`);
      if (want && x.decoColor !== underline) p.push(`${what}: decoration colour ${x.decoColor}`);
    });
  } finally {
    setState(reading, { hover: false, fv: false, wrapped: false, selected: false });
  }
  return p.slice(0, 20);
}
