// Pen mode in the whole reader screen (highlights-pen.md 3, 4): the keys,
// the button and tray, clicks and drags through the pointer handling, the
// one-chapter clamp, Esc cancelling a stroke, undo, the whole-verse click on a
// verse number and Q30 selection highlighting. The IPC is mocked from the
// fixture golden; the pen store here only records strokes (the merge rules,
// bounds, B1, wrong translation, durability and migration are the Rust
// tests: marks.rs, user.rs, pen_tests.rs). happy-dom has no layout, so the
// caret under the pointer is stubbed per gesture.
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

interface GVerse { v: number; text: string; runs: unknown[] }
interface GChapter { book: number; osis: string; chapter: number; verses: GVerse[] }
interface GTr { abbrev: string; has_red_letters: boolean; chapters: GChapter[] }
const golden: { translations: GTr[] } = JSON.parse(
  readFileSync(resolve(process.cwd(), "src-tauri/tests/fixtures/placeholder-render.json"), "utf8"),
);

interface P { verse: number; start: number; end: number }
interface M { id: number; color: string; created_at: number; pieces: (P & { stale: boolean })[]; key: string }
const calls: { cmd: string; args: Record<string, unknown> }[] = [];
let store: M[] = [];
let nextId = 1;
let saved: unknown = { position: null, settings: { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, right_width_rem: 22, right_split: 0.5 } };
const keyOf = (a: Record<string, unknown>) => `${a.translation}/${a.book}/${a.chapter}`;
const out = (a: Record<string, unknown>) => store.filter((m) => m.key === keyOf(a)).map(({ key: _k, ...m }) => m);

function books(tr: GTr) {
  const by = new Map<number, GChapter[]>();
  for (const c of tr.chapters) by.set(c.book, [...(by.get(c.book) ?? []), c]);
  return [...by.entries()].map(([ord, cs]) => ({
    ord, osis: cs[0].osis, name: cs[0].osis, heading: cs[0].osis, main_title: null, abbrevs: [], chapters: cs.length,
    verses: cs.map((c) => Math.max(...c.verses.map((v) => v.v))),
  }));
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
    calls.push({ cmd, args });
    const tr = golden.translations.find((t) => t.abbrev === args.translation) ?? golden.translations[0];
    switch (cmd) {
      case "app_info":
        return {
          content_error: null, user_error: null, lexicon_available: true,
          translations: golden.translations.map((t) => ({ abbrev: t.abbrev, name: t.abbrev, has_red_letters: t.has_red_letters })),
        };
      case "list_books": return books(tr);
      case "get_chapter": {
        const c = tr.chapters.find((x) => x.book === args.book && x.chapter === args.chapter)!;
        return { book: c.book, osis: c.osis, chapter: c.chapter, verses: c.verses.map(({ v, text, runs }) => ({ v, text, runs })) };
      }
      case "get_highlights": return [];
      case "set_highlight": return null;
      case "load_state": return saved;
      case "save_state": saved = args.saved; return null;
      case "bookmarks_list": return [];
      case "bookmark_set": case "bookmark_restore": case "bookmark_clear": return null;
      case "marks_for_chapter": return out(args);
      case "pen_apply":
        store.push({ id: nextId++, color: args.color as string, created_at: nextId, key: keyOf(args), pieces: (args.pieces as P[]).map((p) => ({ ...p, stale: false })) });
        return out(args);
      case "pen_erase": store = store.filter((m) => m.key !== keyOf(args)); return out(args);
      case "mark_remove": store = store.filter((m) => m.id !== args.id); return out(args);
      case "mark_recolor": store.forEach((m) => { if (m.id === args.id) m.color = args.color as string; }); return out(args);
      case "marks_replace_chapter":
        store = store.filter((m) => m.key !== keyOf(args));
        for (const m of args.marks as { color: string; created_at: number; pieces: P[] }[]) {
          store.push({ id: nextId++, color: m.color, created_at: m.created_at, key: keyOf(args), pieces: m.pieces.map((p) => ({ ...p, stale: false })) });
        }
        return out(args);
    }
    throw new Error(`unexpected command ${cmd}`);
  },
}));

const settle = async (ms = 0) => {
  for (let i = 0; i < 5; i++) { await new Promise((r) => setTimeout(r, ms)); flushSync(); }
};
const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);
const root = document.documentElement;
const key = (target: EventTarget, k: string, init: KeyboardEventInit = {}) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true, ...init }));
const pane = () => $(".reader-scroll")!;
const verse = (v: number) => $(`.verse[data-v="${v}"]`)!;
const firstText = (v: number) => {
  const w = document.createTreeWalker(verse(v).querySelector(".vtext")!, 4);
  return w.nextNode() as Text;
};
const lastText = (v: number) => {
  const w = document.createTreeWalker(verse(v).querySelector(".vtext")!, 4);
  let n: Node | null, last: Text | null = null;
  while ((n = w.nextNode())) last = n as Text;
  return last!;
};

/** Where the "pointer" is: the element under it and the caret it maps to. */
let hit: { el: Element | null; caret: { node: Node; offset: number } | null } = { el: null, caret: null };
const d = document as unknown as { caretPositionFromPoint: (x: number, y: number) => { offsetNode: Node; offset: number } | null };
function at(el: Element | null, node: Node | null = null, offset = 0) { hit = { el, caret: node ? { node, offset } : null }; }
function ptr(type: string, target: Element, x: number, y: number, init: PointerEventInit = {}) {
  target.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, button: 0, pointerId: 1, clientX: x, clientY: y, ...init }));
}
const penCalls = () => calls.filter((c) => ["pen_apply", "pen_erase", "mark_remove", "mark_recolor", "marks_replace_chapter"].includes(c.cmd));

let app: ReturnType<typeof mount>;
beforeAll(async () => {
  d.caretPositionFromPoint = () => (hit.caret ? { offsetNode: hit.caret.node, offset: hit.caret.offset } : null);
  document.elementFromPoint = () => hit.el;
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  await settle();
});
afterAll(() => unmount(app));
beforeEach(async () => {
  if (root.dataset.pen) { pane().focus(); key(pane(), "Escape"); }
  store = [];
  await settle(130);
  // reload the chapter so the reader's marks match the emptied store
  pane().focus(); key(pane(), "]"); await settle(); key(pane(), "["); await settle();
  calls.length = 0;
});

describe("pen mode: keys, button and tray (3.1, 3.2)", () => {
  it("H in the reading pane turns the pen on with the tray; H again turns it off", async () => {
    pane().focus();
    key(pane(), "h");
    await settle();
    expect(root.dataset.pen).toBe("pen");
    expect($('button[aria-label="Highlighter"]')?.getAttribute("aria-pressed")).toBe("true");
    const tray = $('.pen-tray[role="toolbar"]')!;
    expect(tray.querySelectorAll('[role="radio"]').length).toBe(5);
    expect(tray.querySelector('[aria-checked="true"]')?.getAttribute("data-c")).toBe("gold");
    expect(document.activeElement).toBe(pane()); // the tray never takes focus
    expect(root.style.getPropertyValue("--cursor-pen")).toMatch(/^url\("data:image\/svg\+xml,/);
    key(pane(), "H");
    await settle(130);
    expect(root.dataset.pen).toBeUndefined();
    expect($(".pen-tray")).toBeNull();
  });

  it("1-5 choose the colour (saved as pen.color), E the eraser, Esc turns the pen off", async () => {
    pane().focus();
    key(pane(), "h");
    await settle();
    key(pane(), "3");
    await settle(600);
    expect($('.pen-tray [aria-checked="true"]')?.getAttribute("data-c")).toBe("blue");
    expect((saved as { settings: { pen_color: string } }).settings.pen_color).toBe("blue");
    expect(calls.some((c) => c.cmd === "set_highlight")).toBe(false); // not the whole-verse key now
    key(pane(), "e");
    await settle();
    expect(root.dataset.pen).toBe("eraser");
    expect($('.pen-tray [aria-label="Eraser"]')?.getAttribute("aria-pressed")).toBe("true");
    key(pane(), "E");
    await settle();
    expect(root.dataset.pen).toBe("pen");
    key(pane(), "Escape");
    await settle(130);
    expect(root.dataset.pen).toBeUndefined();
  });

  it("E14: H typed in the command box does not toggle the pen", async () => {
    const input = $<HTMLInputElement>("#cmd-input")!;
    input.focus();
    key(input, "h");
    await settle();
    expect(root.dataset.pen).toBeUndefined();
    pane().focus();
  });

  it("the toolbar button toggles the pen", async () => {
    $<HTMLButtonElement>('button[aria-label="Highlighter"]')!.click();
    await settle();
    expect(root.dataset.pen).toBe("pen");
    $<HTMLButtonElement>(".pen-tray .pen-done")!.click();
    await settle(130);
    expect(root.dataset.pen).toBeUndefined();
  });
});

describe("pen gestures (4.1, 4.3)", () => {
  beforeEach(async () => { pane().focus(); key(pane(), "h"); await settle(); key(pane(), "1"); await settle(); });

  it("a click on plain text highlights that word; the verse DOM is unchanged", async () => {
    const before = verse(2).querySelector(".vtext")!.outerHTML;
    const t = firstText(2); // "Zeta eta theta, ..."
    at(t.parentElement, t, 2);
    ptr("pointerdown", t.parentElement!, 10, 10);
    ptr("pointerup", t.parentElement!, 10, 10);
    await settle();
    const c = penCalls();
    expect(c.map((x) => x.cmd)).toEqual(["pen_apply"]);
    expect(c[0].args).toMatchObject({ translation: "PH1", chapter: 1, color: "gold", pieces: [{ verse: 2, start: 0, end: 4 }] });
    expect(verse(2).hasAttribute("data-ranges")).toBe(true);
    expect(verse(2).querySelector(":scope > .hl-ink")).not.toBeNull();
    expect(verse(2).querySelector(".sr-only")?.textContent).toContain(", pen highlight");
    expect(verse(2).querySelector(".vtext")!.outerHTML).toBe(before);
  });

  it("a drag across verses writes one mark with a piece per verse; Alt snaps to letters", async () => {
    const a = firstText(2), b = firstText(4);
    at(a.parentElement, a, 6); // inside "eta"
    ptr("pointerdown", a.parentElement!, 10, 10, { altKey: true });
    at(b.parentElement, b, 2); // inside "Rho"
    ptr("pointermove", pane(), 60, 80, { altKey: true });
    await settle();
    expect(verse(3).hasAttribute("data-ranges")).toBe(true); // the live preview
    ptr("pointerup", pane(), 60, 80, { altKey: true });
    await settle();
    const c = penCalls();
    expect(c.length).toBe(1);
    expect(c[0].args.pieces).toEqual([
      { verse: 2, start: 6, end: 34 },
      { verse: 3, start: 0, end: 20 },
      { verse: 4, start: 0, end: 2 },
    ]);
  });

  it("Q31: a drag onto the end-of-chapter buttons is clamped to the chapter's last character", async () => {
    const a = firstText(9);
    at(a.parentElement, a, 0);
    ptr("pointerdown", a.parentElement!, 10, 10);
    at($(".chapter-end button") ?? $(".chapter-end"), null);
    ptr("pointermove", pane(), 10, 900);
    await settle();
    expect($(".pen-hint")?.textContent).toBe("Highlights stay within one chapter");
    ptr("pointerup", pane(), 10, 900);
    await settle();
    const pieces = penCalls()[0].args.pieces as P[];
    const last = Array.from(document.querySelectorAll<HTMLElement>(".verse")).pop()!;
    const lv = Number(last.dataset.v);
    expect(pieces[pieces.length - 1].verse).toBe(lv);
    const text = last.querySelector(".vtext")!.textContent!;
    expect(pieces[pieces.length - 1].end).toBe(Array.from(text.replace(/\s+$/u, "")).length);
    expect(pieces.every((p) => p.verse >= 9 && p.verse <= lv)).toBe(true);
    expect(penCalls()[0].args.chapter).toBe(1);
  });

  it("Esc during a drag cancels it: nothing is written; a second Esc turns the pen off", async () => {
    const a = firstText(2), b = lastText(3);
    at(a.parentElement, a, 0);
    ptr("pointerdown", a.parentElement!, 10, 10);
    at(b.parentElement, b, 3);
    ptr("pointermove", pane(), 50, 50);
    await settle();
    key(pane(), "Escape");
    ptr("pointerup", pane(), 50, 50);
    await settle();
    expect(penCalls()).toEqual([]);
    expect(root.dataset.pen).toBe("pen");
    expect(document.querySelectorAll(".hl-ink").length).toBe(0);
    key(pane(), "Escape");
    await settle(130);
    expect(root.dataset.pen).toBeUndefined();
  });

  it("E9: a lost pointer capture without pointerup cancels the stroke", async () => {
    const a = firstText(2), b = lastText(3);
    at(a.parentElement, a, 0);
    ptr("pointerdown", a.parentElement!, 10, 10);
    at(b.parentElement, b, 3);
    ptr("pointermove", pane(), 50, 50);
    ptr("lostpointercapture", pane(), 50, 50);
    ptr("pointerup", pane(), 50, 50);
    await settle();
    expect(penCalls()).toEqual([]);
  });

  it("a click on a mark in the pen's colour removes it with an Undo toast; Ctrl+Z restores it", async () => {
    const t = firstText(2);
    at(t.parentElement, t, 1);
    ptr("pointerdown", t.parentElement!, 10, 10);
    ptr("pointerup", t.parentElement!, 10, 10);
    await settle();
    ptr("pointerdown", t.parentElement!, 10, 10);
    ptr("pointerup", t.parentElement!, 10, 10);
    await settle();
    expect(penCalls().map((c) => c.cmd)).toEqual(["pen_apply", "mark_remove"]);
    expect($(".undo-toast")?.textContent).toContain("Highlight removed");
    expect(verse(2).hasAttribute("data-ranges")).toBe(false);
    key(pane(), "z", { ctrlKey: true });
    await settle();
    const r = penCalls().pop()!;
    expect(r.cmd).toBe("marks_replace_chapter");
    expect(r.args.marks).toEqual([{ color: "gold", created_at: expect.any(Number), pieces: [{ verse: 2, start: 0, end: 4 }] }]);
    expect(verse(2).hasAttribute("data-ranges")).toBe(true);
    key(pane(), "y", { ctrlKey: true }); // redo: removed again
    await settle();
    expect(verse(2).hasAttribute("data-ranges")).toBe(false);
  });

  it("a click on a mark in another colour recolours it", async () => {
    const t = firstText(2);
    at(t.parentElement, t, 1);
    ptr("pointerdown", t.parentElement!, 10, 10);
    ptr("pointerup", t.parentElement!, 10, 10);
    await settle();
    key(pane(), "4");
    await settle();
    ptr("pointerdown", t.parentElement!, 10, 10);
    ptr("pointerup", t.parentElement!, 10, 10);
    await settle();
    expect(penCalls().pop()).toMatchObject({ cmd: "mark_recolor", args: { color: "rose" } });
  });

  it("a click on a verse number toggles that verse's whole-verse highlight in the pen colour", async () => {
    const n = verse(5).querySelector(".vnum")!;
    at(n, null);
    ptr("pointerdown", n, 1, 1);
    ptr("pointerup", n, 1, 1);
    await settle();
    expect(calls.filter((c) => c.cmd === "set_highlight").pop()?.args).toMatchObject({ color: "gold" });
    expect(verse(5).dataset.hl).toBe("gold");
    ptr("pointerdown", n, 1, 1);
    ptr("pointerup", n, 1, 1);
    await settle();
    expect(verse(5).dataset.hl).toBeUndefined();
    expect(penCalls()).toEqual([]);
  });

  it("a press on the chapter heading starts no stroke", async () => {
    const h = $(".chapter-head")!;
    at(h, null);
    ptr("pointerdown", h, 1, 1);
    ptr("pointerup", h, 1, 1);
    await settle();
    expect(penCalls()).toEqual([]);
  });
});

describe("Q30: 1-5 outside pen mode with a selection highlight the selection", () => {
  it("snaps to grapheme clusters only, trims, and clears the selection", async () => {
    pane().focus();
    const t = firstText(1); // "Alpha beta ..."
    const sel = document.getSelection()!;
    const r = document.createRange();
    r.setStart(t, 2); r.setEnd(t, 5);
    sel.removeAllRanges(); sel.addRange(r);
    key(pane(), "2");
    await settle();
    const c = penCalls();
    expect(c.length).toBe(1);
    expect(c[0].args).toMatchObject({ color: "green", pieces: [{ verse: 1, start: 2, end: 5 }] });
    expect(document.getSelection()?.isCollapsed ?? true).toBe(true);
  });
});
