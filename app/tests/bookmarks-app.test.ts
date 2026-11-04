// Automatic bookmarks in the reader (docs/design/bookmarks.md), on the real KJV
// golden with the IPC mocked and FAKE timers (the dwell clock, the debounced
// writer, the tooltip delay):
// - the launch restore writes nothing until the reader touches the app;
// - quick jumps write nothing; the command box pauses the dwell clock;
// - only user scrolls count; programmatic scrolls never do;
// - the ribbon, its words and tooltip in the picker and on Go to rows;
// - Shift+Delete / the context menu clear, Ctrl+Z and Undo restore exactly.
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { kjv } from "./kjv-golden";
import { trust, type Bookmark } from "../src/lib/bookmarks";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];
const store = new Map<string, Bookmark>();
const NOW_S = () => Math.floor(Date.now() / 1000);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
    calls.push({ cmd, args });
    switch (cmd) {
      case "app_info":
        return { content_error: null, user_error: null, translations: [kjv.translation], lexicon_available: true, lexicon_error: null, search_available: true, search_error: null };
      case "list_books": return kjv.books;
      case "get_chapter": {
        const c = kjv.chapters.find((x) => x.book === args.book && x.chapter === args.chapter);
        if (!c) throw "not found";
        return { book: c.book, osis: c.osis, chapter: c.chapter, verses: c.verses.map(({ v, text, runs }) => ({ v, text, runs })) };
      }
      case "get_highlights": return [];
      case "load_state":
        return {
          position: { translation: "KJV", book: "John", chapter: 3, anchor_verse: null, anchor_frac: null, current_verse: null },
          settings: { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, right_width_rem: 22, right_split: 0.5 },
        };
      case "save_state": return null;
      case "search": return { status: "ok", total: 0, exact_total: 0, offset: 0, phrase: false, hits: [] };
      case "bookmarks_list": return [...store.values()];
      case "bookmark_set": store.set(args.book as string, { book: args.book as string, chapter: args.chapter as number, read_at: NOW_S() }); return null;
      case "bookmark_clear": { const old = store.get(args.book as string) ?? null; store.delete(args.book as string); return old; }
      case "bookmark_restore": { const r = args.row as Bookmark; store.set(r.book, r); return null; }
    }
    throw new Error(`unexpected command ${cmd}`);
  },
}));

const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);
const $$ = <T extends Element = HTMLElement>(s: string) => Array.from(document.querySelectorAll<T>(s));
const advance = async (ms: number) => {
  await vi.advanceTimersByTimeAsync(ms);
  flushSync();
};
const settle = () => advance(20);
const key = (target: EventTarget, k: string, init: KeyboardEventInit = {}) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true, ...init }));
const sets = () => calls.filter((c) => c.cmd === "bookmark_set").map((c) => `${c.args.book} ${c.args.chapter}`);
const input = () => $<HTMLInputElement>("#cmd-input")!;
const pane = () => $<HTMLElement>(".reader-scroll")!;

async function go(text: string) {
  input().focus();
  await settle();
  input().value = text;
  input().dispatchEvent(new Event("input", { bubbles: true }));
  await advance(200);
  key(input(), "Enter");
  await settle();
}

let app: ReturnType<typeof mount> | null = null;
beforeAll(async () => {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "setInterval", "clearInterval", "performance", "Date"] });
  vi.setSystemTime(new Date(2026, 9, 25, 12, 0));
  trust.isTrusted = () => true; // synthetic events stand for the reader's here
  document.hasFocus = () => true;
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  for (let i = 0; i < 10; i++) await settle();
});
afterAll(() => {
  if (app) unmount(app);
  vi.useRealTimers();
});

describe("what counts as read (bookmarks.md 2)", () => {
  it("the launch restore writes nothing while untouched; after the first input, 10 s counts", async () => {
    expect(document.visibilityState).toBe("visible");
    expect($(".chapter-head")?.textContent).toBe("John 3");
    await advance(20_000);
    expect(sets()).toEqual([]);
    key(document.body, "ArrowDown"); // the first trusted input arms the restored visit
    await advance(9_500);
    expect(sets()).toEqual([]);
    await advance(1_300); // 10 s (ticks every 250ms), then the 500ms writer
    expect(sets()).toEqual(["John 3"]);
  });

  it("quick jumps write nothing (the field's own time is paused, each visit under 10 s)", async () => {
    calls.length = 0;
    for (const t of ["ps 23", "mark 8", "matt 5"]) { await go(t); await advance(3_000); }
    expect($(".chapter-head")?.textContent).toBe("Matthew 5");
    // leaving John 3 (qualified) refreshed it; nothing else was written
    expect(sets()).toEqual(["John 3"]);
  });

  it("the command box pauses the dwell clock without resetting it", async () => {
    await go("rev 1");
    calls.length = 0;
    await advance(6_000);
    input().focus();
    await settle();
    await advance(30_000);
    expect(sets()).toEqual([]);
    key(input(), "Escape");
    await settle();
    expect(document.activeElement).toBe(pane());
    await advance(3_500);
    expect(sets()).toEqual([]);
    await advance(1_300);
    expect(sets()).toEqual(["Rev 1"]);
  });

  it("only user scrolls count: a programmatic scroll never does; one pane height of user scrolling does", async () => {
    await go("rev 22");
    await advance(600); // the end-of-visit refresh of Rev 1
    calls.length = 0;
    const el = pane();
    let top = 0;
    Object.defineProperty(el, "clientHeight", { configurable: true, value: 500 });
    Object.defineProperty(el, "scrollTop", { configurable: true, get: () => top, set: (v: number) => { top = v; } });
    el.scrollTop = 2_000; // a programmatic scroll (jump, resume, text size)
    el.dispatchEvent(new Event("scroll"));
    await advance(600);
    expect(sets()).toEqual([]);
    for (let i = 0; i < 3; i++) {
      el.dispatchEvent(new WheelEvent("wheel", { deltaY: 200, bubbles: true }));
      el.scrollTop += 200;
      el.dispatchEvent(new Event("scroll"));
    }
    await advance(600);
    expect(sets()).toEqual(["Rev 22"]);
  });
});

describe("the ribbon, its words, and clearing (bookmarks.md 3 to 7)", () => {
  it("the picker shows the ribbon on bookmarked books, and opens a bookmarked book on its chapter", async () => {
    store.set("Ps", { book: "Ps", chapter: 23, read_at: NOW_S() - 86_400 });
    $<HTMLButtonElement>(".cmd-chev")!.click();
    await settle();
    await settle();
    const ps = $<HTMLButtonElement>('.pk-cell[data-ord="19"]')!;
    expect(ps.querySelector("svg.bm")?.getAttribute("aria-hidden")).toBe("true");
    expect(ps.textContent).toBe("Psalms, bookmarked at chapter 23, last read yesterday");
    expect(ps.getAttribute("aria-keyshortcuts")).toBe("Shift+F10 Shift+Delete");
    const job = $<HTMLButtonElement>('.pk-cell[data-ord="18"]')!;
    expect(job.querySelector(".bm")).toBeNull();
    expect(job.hasAttribute("aria-keyshortcuts")).toBe(false);
    // tooltip after 400ms of keyboard focus
    ps.focus();
    await advance(350);
    expect($('[role="tooltip"]')).toBeNull();
    await advance(100);
    expect($('[role="tooltip"]')?.textContent).toBe("Bookmark: chapter 23 · last read yesterday");
    expect(ps.getAttribute("aria-describedby")).toBe("pk-tip");
    key(ps, "Escape"); // Esc closes the tooltip first
    await settle();
    expect($('[role="tooltip"]')).toBeNull();
    expect($(".picker")).not.toBeNull();
    ps.click();
    await settle();
    const c23 = $<HTMLButtonElement>('.pk-cell[data-ch="23"]')!;
    expect(document.activeElement).toBe(c23);
    expect(c23.getAttribute("aria-label")).toBe("Psalms 23, bookmarked, last read yesterday");
    expect(c23.querySelector("svg.bm")).not.toBeNull();
    expect($$(".pk-grid.chapters .bm").length).toBe(1);
  });

  it("Shift+F10 opens the one-item menu on a bookmarked cell only; Shift+Delete clears; Ctrl+Z restores exactly", async () => {
    const c23 = $<HTMLButtonElement>('.pk-cell[data-ch="23"]')!;
    key(c23, "F10", { shiftKey: true });
    await settle();
    expect($$('.bm-menu [role="menuitem"]').map((e) => e.textContent?.trim())).toEqual(["Clear bookmark"]);
    expect(document.activeElement?.getAttribute("role")).toBe("menuitem");
    key(document.activeElement!, "Escape");
    await settle();
    expect($(".bm-menu")).toBeNull();
    expect(document.activeElement).toBe(c23);
    const c22 = $<HTMLButtonElement>('.pk-cell[data-ch="22"]')!;
    c22.focus();
    key(c22, "F10", { shiftKey: true });
    key(c22, "Delete", { shiftKey: true });
    await settle();
    expect($(".bm-menu")).toBeNull();
    expect(calls.some((c) => c.cmd === "bookmark_clear")).toBe(false);

    const old = { ...store.get("Ps")! };
    c23.focus();
    key(c23, "Delete", { shiftKey: true });
    await advance(200);
    expect(store.has("Ps")).toBe(false);
    expect($(".undo-toast")?.textContent).toContain("Bookmark cleared");
    expect($(".picker")).not.toBeNull(); // the picker stays open
    expect(document.activeElement).toBe(c23);
    key(c23, "z", { ctrlKey: true });
    await settle();
    expect(store.get("Ps")).toEqual(old); // same chapter and read_at
    expect($(".undo-toast")).toBeNull();
    expect($('.pk-cell[data-ch="23"] svg.bm')).not.toBeNull();
    key($('.pk-cell[data-ch="23"]')!, "Escape");
    await settle();
  });

  it("the Undo toast goes after 6 s", async () => {
    $<HTMLButtonElement>(".cmd-chev")!.click();
    await settle();
    await settle();
    const ps = $<HTMLButtonElement>('.pk-cell[data-ord="19"]')!;
    ps.focus();
    key(ps, "Delete", { shiftKey: true });
    await advance(200);
    expect($(".undo-toast")).not.toBeNull();
    await advance(6_000);
    expect($(".undo-toast")).toBeNull();
    key(document.activeElement!, "z", { ctrlKey: true });
    await settle();
    expect(store.has("Ps")).toBe(false);
    key(document.activeElement!, "Escape");
    await settle();
  });

  it("a bookmarked Go to row names its chapter, and Enter goes there; Shift+Delete clears it", async () => {
    store.set("Ps", { book: "Ps", chapter: 23, read_at: NOW_S() });
    pane().focus(); // the dropdown reads the rows when the field gains focus
    await settle();
    input().focus();
    await settle();
    input().value = "psalms";
    input().dispatchEvent(new Event("input", { bubbles: true }));
    await advance(200);
    const row = $<HTMLElement>("#cmd-opt-go-0")!;
    expect(row.querySelector("svg.bm")).not.toBeNull();
    expect(row.querySelector(".sub")?.textContent).toBe("Chapter 23 · last read today");
    expect(row.querySelector(".sub")?.getAttribute("aria-hidden")).toBe("true");
    expect(row.querySelector(".sr-only")?.textContent).toBe(", bookmarked at chapter 23, last read today");
    key(input(), "Delete", { shiftKey: true });
    await advance(200);
    expect(store.has("Ps")).toBe(false);
    expect($("#cmd-opt-go-0 .sub")?.textContent).toBe("150 chapters");
    expect(document.activeElement).toBe(input()); // focus and the dropdown stay
    key(input(), "z", { ctrlKey: true });
    await settle();
    expect(store.get("Ps")?.chapter).toBe(23);
    expect($("#cmd-opt-go-0 .sub")?.textContent).toBe("Chapter 23 · last read today");
    key(input(), "Enter");
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Psalm 23");
  });
});
