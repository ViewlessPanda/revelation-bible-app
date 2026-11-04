// The right column in the whole reader screen (wave-2-layout.md 17), IPC mocked
// from the fixture golden as in app.test.ts: Commentary above Study in one
// column, the open panel fills it, the divider (drag, keys, Enter, double-click,
// Esc), the remembered split and width, and the restore of saved values.
// happy-dom has no layout, so the column's height is stubbed (701px: 700 less
// the 1px divider row).
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

interface GVerse { v: number; text: string; runs: unknown[] }
interface GChapter { book: number; osis: string; chapter: number; verses: GVerse[] }
interface GTr { abbrev: string; has_red_letters: boolean; chapters: GChapter[] }
const golden: { translations: GTr[] } = JSON.parse(
  readFileSync(resolve(process.cwd(), "src-tauri/tests/fixtures/placeholder-render.json"), "utf8"),
);
const cmtGolden = JSON.parse(readFileSync(resolve(process.cwd(), "src-tauri/tests/golden/mhcc-render.json"), "utf8"));

type Saved = { position: null; settings: Record<string, unknown> };
const base = { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, commentary_open: false, pen_color: "gold" };
let saved: Saved = { position: null, settings: { ...base } };
const stored = () => (saved as Saved).settings;

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
    const tr = golden.translations.find((t) => t.abbrev === args.translation) ?? golden.translations[0];
    switch (cmd) {
      case "app_info":
        return {
          content_error: null, user_error: null, lexicon_available: true,
          translations: golden.translations.map((t) => ({ abbrev: t.abbrev, name: `${t.abbrev} placeholder`, has_red_letters: t.has_red_letters })),
        };
      case "list_books": return books(tr);
      case "get_chapter": {
        const c = tr.chapters.find((x) => x.book === args.book && x.chapter === args.chapter);
        if (!c) throw "not found";
        return { book: c.book, osis: c.osis, chapter: c.chapter, verses: c.verses.map(({ v, text, runs }) => ({ v, text, runs })) };
      }
      case "get_highlights": return [];
      case "get_strongs":
        return { status: "ok", entry: { key: args.key, lexicon: "LEX-HEB-OSHB", nodes: [
          { n: 0, parent: null, name: "def", text: null, attrs: [] },
          { n: 1, parent: 0, name: null, text: `Placeholder definition for ${args.key}.`, attrs: [] },
        ] } };
      case "load_state": return saved;
      case "bookmarks_list": return [];
      case "save_state": saved = args.saved as Saved; return null;
      case "get_commentary": {
        const r = cmtGolden["John.3.3"];
        return { ...r, verse_id: (args.book as number) * 1_000_000 + (args.chapter as number) * 1000 + (args.verse as number) };
      }
    }
    throw new Error(`unexpected command ${cmd}`);
  },
}));

const settle = async (ms = 0) => {
  for (let i = 0; i < 5; i++) { await new Promise((r) => setTimeout(r, ms)); flushSync(); }
};
const sleep = async (ms: number) => { await new Promise((r) => setTimeout(r, ms)); flushSync(); };
const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);
const key = (target: EventTarget, k: string, init: KeyboardEventInit = {}) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true, ...init }));
const ptr = (target: EventTarget, type: string, y: number, x = 0) =>
  target.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientY: y, clientX: x }));
/** Commentary's height as the style says it, rounded to whole px (the split is kept to 3 decimals) */
const commH = () => `${Math.round(parseFloat($("#rcol")!.style.getPropertyValue("--rc-comm-h")))}px`;
const commRaw = () => $("#rcol")!.style.getPropertyValue("--rc-comm-h");

let app: ReturnType<typeof mount> | null = null;
async function start(settings: Record<string, unknown> = {}) {
  saved = { position: null, settings: { ...base, ...settings } };
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  await settle();
}
afterEach(() => { if (app) unmount(app); app = null; });

beforeAll(() => {
  // no layout engine: the column is 701px tall (avail = 700)
  Object.defineProperty(HTMLElement.prototype, "clientHeight", { configurable: true, get() { return (this as HTMLElement).id === "rcol" ? 701 : 0; } });
});

describe("one column, Commentary on top, Study underneath", () => {
  it("both open: comment, divider, study in that order, in the one column", async () => {
    await start({ commentary_open: true, panel_open: true });
    const col = $("#rcol")!;
    expect([...col.children].map((c) => c.id || c.className.split(" ")[0])).toEqual(["splitter", "p-comm", "rc-div", "p-study"]);
    expect(col.classList.contains("two")).toBe(true);
    expect(document.querySelectorAll(".splitter").length).toBe(1); // the width splitter, once
    const div = $(".rc-div")!;
    expect(div.getAttribute("role")).toBe("separator");
    expect(div.getAttribute("aria-orientation")).toBe("horizontal");
    expect(div.getAttribute("aria-label")).toBe("Resize commentary and lexicon");
    expect(div.getAttribute("aria-controls")).toBe("p-comm");
    expect(div.tabIndex).toBe(0);
    expect(div.contains($("#p-comm"))).toBe(false); // neither panel contains it
    expect($("#p-comm")!.contains(div) || $("#p-study")!.contains(div)).toBe(false);
  });

  it("with one panel open it fills the column: no divider, a single row", async () => {
    await start({ commentary_open: true });
    expect($("#p-comm")).not.toBeNull();
    expect($("#p-study")).toBeNull();
    expect($(".rc-div")).toBeNull();
    expect($("#rcol")!.classList.contains("two")).toBe(false);
    unmount(app!);
    await start({ panel_open: true });
    expect($("#p-study")).not.toBeNull();
    expect($("#p-comm")).toBeNull();
    expect($(".rc-div")).toBeNull();
    expect($("#rcol")!.classList.contains("two")).toBe(false);
  });

  it("nothing open: no column at all", async () => {
    await start({});
    expect($("#rcol")).toBeNull();
  });

  it("closing one of two: the other fills the column; closing the last takes the column away", async () => {
    await start({ commentary_open: true, panel_open: true });
    $<HTMLButtonElement>('#p-study [aria-label="Close study panel"]')!.click();
    await settle(30);
    expect($("#p-study")).toBeNull();
    expect($(".rc-div")).toBeNull();
    expect($("#rcol")!.classList.contains("two")).toBe(false);
    expect($("#p-comm")).not.toBeNull();
    await sleep(550);
    expect(stored().panel_open).toBe(false);
    // and the other way round: Study alone after Commentary closes
    $<HTMLButtonElement>('[aria-label="Study panel"]')!.click(); // the toggle opens Study again
    await settle(30);
    expect($("#rcol")!.classList.contains("two")).toBe(true);
    $<HTMLButtonElement>('#p-comm [aria-label="Close commentary panel"]')!.click();
    await settle(30);
    expect($("#p-comm")).toBeNull();
    expect($("#p-study")).not.toBeNull();
    expect($("#rcol")!.classList.contains("two")).toBe(false);
    $<HTMLButtonElement>('#p-study [aria-label="Close study panel"]')!.click();
    await settle(40);
    expect($("#rcol")).toBeNull();
  });

  it("a second panel opens inside the open column without moving the width", async () => {
    await start({ commentary_open: true, right_width_rem: 25 });
    const w = $("#rcol")!.style.width;
    key(document.body, "j", { ctrlKey: true }); // Ctrl+J
    await settle(30);
    expect($("#rcol")!.classList.contains("two")).toBe(true);
    expect($("#rcol")!.style.width).toBe(w);
    expect(document.querySelectorAll("#rcol").length).toBe(1);
    expect(document.activeElement?.closest("#p-study")).not.toBeNull(); // opening the second panel by shortcut moves focus to it
  });

  it("Tab order: the column splitter, then Commentary, then the divider, then Study", async () => {
    await start({ commentary_open: true, panel_open: true });
    const order = [...$("#rcol")!.querySelectorAll<HTMLElement>('[tabindex="0"], button, .panel-title')]
      .filter((e) => e.tabIndex >= 0)
      .map((e) => e.closest("#p-comm") ? "comm" : e.closest("#p-study") ? "study" : e.classList.contains("rc-div") ? "divider" : "splitter");
    expect(order[0]).toBe("splitter");
    expect(order.indexOf("comm")).toBeLessThan(order.indexOf("divider"));
    expect(order.indexOf("divider")).toBeLessThan(order.indexOf("study"));
  });
});

describe("the split: restore, clamp, keys, reset, drag (17.2, 17.4)", () => {
  it("restores the saved split; it is clamped to the minimums when drawn but never rewritten", async () => {
    await start({ commentary_open: true, panel_open: true, right_split: 0.25 });
    expect(commH()).toBe("224px"); // 0.25 * 700 = 175 < 14rem
    const div = $(".rc-div")!;
    expect(div.getAttribute("aria-valuenow")).toBe("32");
    expect(div.getAttribute("aria-valuemin")).toBe("32");
    expect(div.getAttribute("aria-valuemax")).toBe("77");
    expect(div.getAttribute("aria-valuetext")).toBe("Commentary 32%");
    window.dispatchEvent(new Event("pagehide")); // saves now
    await settle();
    expect(stored().right_split).toBe(0.25);
    unmount(app!);
    await start({ commentary_open: true, panel_open: true, right_split: 0.6 });
    expect(commH()).toBe("420px");
    expect($(".rc-div")!.getAttribute("aria-valuetext")).toBe("Commentary 60%");
    unmount(app!);
    await start({ commentary_open: true, panel_open: true, right_split: 7 }); // junk: the default
    expect(commH()).toBe("350px");
  });

  it("arrow keys move 16px, Shift 64px, Home and End go to the minimums; each step is saved", async () => {
    await start({ commentary_open: true, panel_open: true });
    const div = $<HTMLElement>(".rc-div")!;
    div.focus();
    expect(commH()).toBe("350px");
    key(div, "ArrowDown");
    await settle(20);
    expect(commH()).toBe("366px");
    key(div, "ArrowUp", { shiftKey: true });
    await settle(20);
    expect(commH()).toBe("302px");
    key(div, "Home");
    await settle(20);
    expect(commH()).toBe("224px"); // Commentary at its minimum (224px)
    key(div, "ArrowUp");
    await settle(20);
    expect(commH()).toBe("224px"); // clamped
    key(div, "End");
    await settle(20);
    expect(commH()).toBe("540px"); // Study at its minimum (160px)
    expect(div.getAttribute("aria-valuenow")).toBe("77");
    await sleep(550); // 500ms after the last change
    expect(stored().right_split).toBe(0.771);
    expect(document.activeElement).toBe(div); // the keys never move focus
  });

  it("Enter resets to the default split; so does a double-click", async () => {
    await start({ commentary_open: true, panel_open: true, right_split: 0.7 });
    const div = $<HTMLElement>(".rc-div")!;
    div.focus();
    expect(commH()).toBe("490px");
    key(div, "Enter");
    await settle(20);
    expect(commH()).toBe("350px");
    await sleep(550);
    expect(stored().right_split).toBe(0.5);
    key(div, "End");
    await settle(20);
    expect(commH()).toBe("540px");
    div.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    await settle(20);
    expect(commH()).toBe("350px");
    await sleep(550);
    expect(stored().right_split).toBe(0.5);
  });

  it("a pointer drag writes the height once per frame, saves on release; Esc puts it back", async () => {
    await start({ commentary_open: true, panel_open: true });
    const col = $("#rcol")!;
    const div = $(".rc-div")!;
    const writes: string[] = [];
    const orig = col.style.setProperty.bind(col.style);
    col.style.setProperty = ((name: string, value: string | null, prio?: string) => {
      if (name === "--rc-comm-h") writes.push(String(value));
      return orig(name, value, prio);
    }) as typeof col.style.setProperty;

    ptr(div, "pointerdown", 400);
    flushSync();
    expect(document.documentElement.classList.contains("rc-drag")).toBe(true);
    expect(div.classList.contains("dragging")).toBe(true);
    for (const y of [410, 420, 430, 440]) ptr(div, "pointermove", y);
    expect(writes).toEqual([]); // nothing is written between frames
    await new Promise((r) => setTimeout(r, 60));
    expect(writes).toEqual(["390px"]); // one write for the frame, the latest y (350 + 40)
    ptr(div, "pointermove", 460);
    ptr(div, "pointerup", 470);
    flushSync();
    expect(writes.at(-1)).toBe("420px"); // the final y, applied on release
    expect(document.documentElement.classList.contains("rc-drag")).toBe(false);
    expect(div.classList.contains("dragging")).toBe(false);
    await sleep(550);
    expect(stored().right_split).toBe(0.6);
    expect(commH()).toBe("420px");

    // Esc during a drag: back to where it began, nothing saved
    writes.length = 0;
    ptr(div, "pointerdown", 100);
    ptr(div, "pointermove", 160);
    await new Promise((r) => setTimeout(r, 60));
    expect(commRaw()).toBe("480px");
    key(document.body, "Escape");
    expect(commRaw()).toBe("420px");
    expect(document.documentElement.classList.contains("rc-drag")).toBe(false);
    ptr(div, "pointerup", 160); // a release after the cancel does nothing
    await sleep(550);
    expect(commH()).toBe("420px");
    expect(stored().right_split).toBe(0.6);
  });

  it("a click on the divider without a move keeps the split (even one that is clamped)", async () => {
    await start({ commentary_open: true, panel_open: true, right_split: 0.1 });
    const div = $(".rc-div")!;
    ptr(div, "pointerdown", 100);
    ptr(div, "pointerup", 100);
    window.dispatchEvent(new Event("pagehide"));
    await settle();
    expect(stored().right_split).toBe(0.1);
  });

  it("Esc on the divider moves focus to the reading pane and keeps both panels", async () => {
    await start({ commentary_open: true, panel_open: true });
    const div = $<HTMLElement>(".rc-div")!;
    div.focus();
    key(div, "Escape");
    await settle();
    expect(document.activeElement).toBe($(".reader-scroll"));
    expect($("#p-study")).not.toBeNull();
    expect($("#p-comm")).not.toBeNull();
  });

  it("Ctrl+Shift+C with focus on the divider counts as focus elsewhere: it focuses Commentary", async () => {
    await start({ commentary_open: true, panel_open: true });
    const div = $<HTMLElement>(".rc-div")!;
    div.focus();
    key(div, "C", { ctrlKey: true, shiftKey: true });
    await settle();
    expect($("#p-comm")).not.toBeNull();
    expect(document.activeElement).toBe($("#p-comm .panel-title"));
  });
});

describe("the column's width splitter and the remembered width (17.1, 17.4)", () => {
  it("restores right_width_rem clamped to 18..36 rem", async () => {
    await start({ commentary_open: true, right_width_rem: 30 });
    expect($("#rcol")!.style.width).toBe("480px");
    unmount(app!);
    await start({ commentary_open: true, right_width_rem: 50 });
    expect($("#rcol")!.style.width).toBe(`${Math.min(36 * 16, window.innerWidth - 26 * 16 - 8)}px`);
    unmount(app!);
    await start({ commentary_open: true, right_width_rem: 3 });
    expect($("#rcol")!.style.width).toBe("288px");
  });

  it("first start: whatever width the host hands over (the larger old width, from user.sqlite) is used; no value is the default", async () => {
    // user.rs migrates the larger of the two old widths into right_width_rem; here the UI just takes it
    await start({ commentary_open: true, right_width_rem: 28.5 });
    expect($("#rcol")!.style.width).toBe("456px");
    unmount(app!);
    await start({ commentary_open: true });
    expect($("#rcol")!.style.width).toBe("352px");
  });

  it("the splitter's arrows resize by 16px and save right_width_rem; Enter is not bound; double-click resets to 22rem", async () => {
    await start({ commentary_open: true, panel_open: true, right_width_rem: 25 });
    const sp = $<HTMLElement>(".splitter")!;
    expect(sp.getAttribute("aria-label")).toBe("Resize right column");
    expect(sp.getAttribute("aria-valuenow")).toBe("400");
    sp.focus();
    key(sp, "ArrowLeft");
    await settle(20);
    expect($("#rcol")!.style.width).toBe("416px");
    key(sp, "ArrowRight", { shiftKey: true });
    await settle(20);
    expect($("#rcol")!.style.width).toBe("352px");
    key(sp, "Enter");
    await settle(20);
    expect($("#p-comm")).not.toBeNull(); // Enter closes nothing
    expect($("#p-study")).not.toBeNull();
    key(sp, "Home");
    await settle(20);
    expect($("#rcol")!.style.width).toBe("288px");
    sp.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    await settle(20);
    expect($("#rcol")!.style.width).toBe("352px");
    await sleep(550);
    expect(stored().right_width_rem).toBe(22);
    expect(stored().right_split).toBe(0.5);
    expect("panel_width_rem" in stored() || "commentary_width_rem" in stored()).toBe(false); // the old widths are no longer written
  });

  it("opening Commentary then Study keeps one column width, saved once", async () => {
    await start({});
    key(document.body, "C", { ctrlKey: true, shiftKey: true });
    await settle(40);
    key(document.body, "j", { ctrlKey: true });
    await settle(40);
    expect(document.querySelectorAll("#rcol").length).toBe(1);
    await sleep(550);
    expect(stored()).toMatchObject({ commentary_open: true, panel_open: true, right_width_rem: 22, right_split: 0.5 });
  });
});
