// Render test (PLAN.md 2.6 / 2.8, red-letters-and-highlights.md 3.4.6 as far
// as happy-dom allows): the rendered DOM text of every fixture chapter equals
// the stored text, and the red spans contain exactly the stored red
// characters, nothing more and nothing less.
//
// The input is placeholder-render.json, written by the Rust core from the
// committed fixture databases through open_verified and the same query code
// the app uses (a Rust test fails if it is stale). Red ranges in it come raw
// from the red_piece table, independently of the runs.
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import ChapterView from "../src/components/ChapterView.svelte";
import ReadingOptions from "../src/components/ReadingOptions.svelte";
import { renderVerseText } from "../src/lib/verse-dom";
import type { Book, Chapter, Run } from "../src/lib/types";
import { checkColours, checkVerse, cp } from "./render-check";

interface GVerse { v: number; id: number; text: string; red: [number, number][]; runs: Run[] }
interface GChapter { book: number; osis: string; chapter: number; verses: GVerse[] }
interface GTranslation { abbrev: string; has_red_letters: boolean; chapters: GChapter[] }
const golden: { translations: GTranslation[] } = JSON.parse(
  readFileSync(resolve(process.cwd(), "src-tauri/tests/fixtures/placeholder-render.json"), "utf8"),
);
const CSS =
  readFileSync(resolve(process.cwd(), "src/styles/tokens.css"), "utf8") +
  readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");

function bookOf(ch: GChapter): Book {
  return { ord: ch.book, osis: ch.osis, name: ch.osis, heading: ch.osis, main_title: null, abbrevs: [], chapters: 3, verses: [] };
}

let styleEl: HTMLStyleElement;
beforeAll(() => {
  styleEl = document.createElement("style");
  styleEl.textContent = CSS;
  document.head.appendChild(styleEl);
});

let app: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.replaceChildren();
  document.documentElement.dataset.redLetters = "on";
});

function renderChapter(ch: GChapter): HTMLElement {
  const reading = document.createElement("div");
  reading.className = "reading";
  document.body.appendChild(reading);
  app = mount(ChapterView, {
    target: reading,
    props: {
      chapter: ch as unknown as Chapter, book: bookOf(ch), highlights: {}, currentVerse: null,
      prevLabel: null, nextLabel: null, onPrev: () => {}, onNext: () => {},
    },
  });
  flushSync();
  return reading;
}

const chapters = golden.translations.flatMap((t) => t.chapters.map((c) => ({ t, c })));

describe("render test on the placeholder fixture", () => {
  it("covers both fixture translations and the edge cases", () => {
    expect(golden.translations.map((t) => t.abbrev)).toEqual(["PH1", "PH2"]);
    const all = chapters.flatMap(({ c }) => c.verses);
    expect(all.length).toBeGreaterThan(90);
    expect(all.some((v) => v.red.length === 2)).toBe(true);
    expect(all.some((v) => v.text.includes("\u{1D400}") && v.red.length)).toBe(true);
    expect(golden.translations[1].chapters.every((c) => c.verses.every((v) => v.red.length === 0))).toBe(true);
  });

  it.each(chapters.map(({ t, c }) => [`${t.abbrev} ${c.osis} ${c.chapter}`, t, c] as const))(
    "%s: DOM text = stored text, red spans = stored ranges",
    (_label, _t, ch) => {
      const reading = renderChapter(ch);
      const verseEls = Array.from(reading.querySelectorAll<HTMLElement>(".verse"));
      expect(verseEls.map((e) => Number(e.dataset.v))).toEqual(ch.verses.map((v) => v.v));
      const problems = verseEls.flatMap((el, i) => checkVerse(el, ch.verses[i]));
      expect(problems).toEqual([]);

      // total red characters in the DOM = the database's count for the chapter
      const domRed = Array.from(reading.querySelectorAll(".vtext .wj")).reduce((n, e) => n + cp(e.textContent ?? "").length, 0);
      const dbRed = ch.verses.reduce((n, v) => n + v.red.reduce((m, [s, e]) => m + e - s, 0), 0);
      expect(domRed).toBe(dbRed);

      // computed colours with red letters on (rest state)
      const rootStyle = getComputedStyle(document.documentElement);
      const red = rootStyle.getPropertyValue("--color-red-letter").trim();
      const text = rootStyle.getPropertyValue("--color-text").trim();
      expect(red).not.toBe("");
      expect(red.toLowerCase()).not.toBe(text.toLowerCase());
      expect(verseEls.flatMap((el, i) => checkColours(el, ch.verses[i], red, text))).toEqual([]);

      // the toggle only flips an attribute on <html>: the DOM is identical
      const before = reading.innerHTML;
      document.documentElement.dataset.redLetters = "off";
      flushSync();
      expect(reading.innerHTML).toBe(before);
    },
  );
});

describe("the harness bites", () => {
  const ch = golden.translations[0].chapters[0];

  it("catches a colour rule inside a red span (M1-F7)", () => {
    const planted = document.createElement("style");
    planted.textContent = ".vtext .add { color: #123456; } .vtext .nd { color: #123457; }";
    document.head.appendChild(planted);
    try {
      const reading = renderChapter(ch);
      const els = Array.from(reading.querySelectorAll<HTMLElement>(".verse"));
      const rs = getComputedStyle(document.documentElement);
      const problems = els.flatMap((el, i) =>
        checkColours(el, ch.verses[i], rs.getPropertyValue("--color-red-letter").trim(), rs.getPropertyValue("--color-text").trim()));
      expect(problems.length).toBeGreaterThan(0);
    } finally {
      planted.remove();
    }
  });

  it("catches a shifted, missing or extra red span and foreign elements", () => {
    const v = ch.verses.find((x) => x.red.length === 1 && x.red[0][0] > 0)!;
    const el = document.createElement("div");
    el.className = "verse";
    el.innerHTML = '<span class="vnum">1</span><span class="vtext"></span>';
    const vt = el.querySelector<HTMLElement>(".vtext")!;
    document.body.appendChild(el);
    renderVerseText(vt, v.runs);
    expect(checkVerse(el, v)).toEqual([]);
    // shift the stored range by one: must fail
    expect(checkVerse(el, { ...v, red: [[v.red[0][0] + 1, v.red[0][1]]] }).length).toBeGreaterThan(0);
    // stored piece missing from the DOM
    renderVerseText(vt, v.runs.map((r) => ({ ...r, red: false, rp: undefined })));
    expect(checkVerse(el, v).length).toBeGreaterThan(0);
    // extra red: a verse without pieces rendered red
    const plain = ch.verses.find((x) => x.red.length === 0)!;
    renderVerseText(vt, plain.runs.map((r) => ({ ...r, red: true, rp: 0 })));
    expect(checkVerse(el, plain).length).toBeGreaterThan(0);
    // foreign element and changed text
    renderVerseText(vt, plain.runs);
    vt.appendChild(document.createElement("em")).textContent = "x";
    expect(checkVerse(el, plain).length).toBeGreaterThan(0);
  });
});

// D5-F2 / design Q6: a Windows contrast theme (forced colours) replaces the
// red-letter colour with the theme's text colour. The app respects that: the
// same words are marked, no rule overrides the theme on verse text, and the
// red-letter toggle says the colour is hidden.
describe("forced colours (Windows contrast theme)", () => {
  type Device = { forcedColors: string };
  const device = () => (window as unknown as { happyDOM: { settings: { device: Device } } }).happyDOM.settings.device;
  let opts: ReturnType<typeof mount> | null = null;
  afterEach(() => {
    device().forcedColors = "none";
    if (opts) unmount(opts);
    opts = null;
  });

  function optionsNote(hasRedLetters: boolean) {
    const host = document.createElement("div");
    document.body.appendChild(host);
    opts = mount(ReadingOptions, {
      target: host,
      props: {
        settings: { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, commentary_open: false, right_width_rem: 22, right_split: 0.5, pen_color: "gold", notes_open: false, notes_width_rem: 20 },
        hasRedLetters, sizePx: 20, onToggle: () => {}, onSize: () => {}, onClose: () => {},
      },
    });
    flushSync();
    const row = host.querySelectorAll<HTMLElement>("[role=switch]")[1];
    return { row, note: row.querySelector<HTMLElement>(".fc-note") };
  }

  it("marks exactly the same words as in normal colours", () => {
    const { c } = chapters.find(({ c }) => c.verses.some((v) => v.red.length))!;
    const normal = renderChapter(c).innerHTML;
    unmount(app!);
    app = null;
    document.body.replaceChildren();
    device().forcedColors = "active";
    expect(matchMedia("(forced-colors: active)").matches).toBe(true);
    const reading = renderChapter(c);
    expect(reading.innerHTML).toBe(normal);
    const verseEls = Array.from(reading.querySelectorAll<HTMLElement>(".verse"));
    expect(verseEls.flatMap((el, i) => checkVerse(el, c.verses[i]))).toEqual([]);
  });

  it("the red-letter toggle shows the note only under a contrast theme", () => {
    let { row, note } = optionsNote(true);
    expect(row.getAttribute("aria-checked")).toBe("true");
    expect(note?.textContent).toBe("Hidden by your Windows contrast theme");
    expect(getComputedStyle(note!).display).toBe("none");
    unmount(opts!);
    opts = null;
    device().forcedColors = "active";
    ({ row, note } = optionsNote(true));
    expect(row.getAttribute("aria-checked")).toBe("true");
    expect(getComputedStyle(note!).display).toBe("block");
    unmount(opts!);
    opts = null;
    // a translation without red letters keeps its own message only
    ({ row, note } = optionsNote(false));
    expect(note).toBeNull();
  });

  it("no forced-colours rule reaches verse text or overrides the theme on it", () => {
    const blocks = [...CSS.matchAll(/@media\s*\(forced-colors:\s*active\)\s*\{([\s\S]*?)\n\}/g)].map((m) => m[1]);
    expect(blocks.length).toBeGreaterThan(0);
    // the two exceptions (3.5): the hover underline's empty ::after box and
    // the selected line's empty ::before box, which hold no text, keep a
    // CanvasText / Highlight background so they stay visible
    const E = /\s*\.vtext \.w::after \{ forced-color-adjust: none; background-color: CanvasText; \}/;
    const S = /\s*\.vtext \.w::before \{ forced-color-adjust: none; background-color: Highlight; \}/;
    expect(blocks.filter((b) => E.test(b)).length).toBe(1);
    expect(blocks.filter((b) => S.test(b)).length).toBe(1);
    for (const b of blocks) expect(b.replace(/\/\*[\s\S]*?\*\//g, "").replace(E, "").replace(S, "")).not.toMatch(/\.(wj|vtext|add|nd|w)\b/);
    expect(CSS).toMatch(/\.fc-note\s*\{\s*display:\s*none;?\s*\}/);
  });
});
