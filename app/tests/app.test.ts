// Smoke test of the whole reader screen with the IPC mocked from the fixture
// golden: start-up, command box navigation, highlight keys,
// chapter keys, Strong's lookup, the "Not available in this translation"
// state and resume. (The real IPC is exercised by the Rust tests and the
// release-exe launch.)
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
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

const calls: { cmd: string; args: Record<string, unknown> }[] = [];
const hl = new Map<string, string>();
let userError: string | null = null;
let userTooNew = false;
let saved: unknown = { position: null, settings: { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, right_width_rem: 22, right_split: 0.5 } };

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
          content_error: null, user_error: userError, user_too_new: userTooNew, lexicon_available: true,
          translations: golden.translations.map((t) => ({ abbrev: t.abbrev, name: `${t.abbrev} placeholder`, has_red_letters: t.has_red_letters })),
        };
      case "list_books": return books(tr);
      case "get_chapter": {
        const c = tr.chapters.find((x) => x.book === args.book && x.chapter === args.chapter);
        if (!c) throw "not found";
        return { book: c.book, osis: c.osis, chapter: c.chapter, verses: c.verses.map(({ v, text, runs }) => ({ v, text, runs })) };
      }
      case "get_highlights":
        return [...hl.entries()]
          .filter(([r]) => r.startsWith(`${args.book}.${args.chapter}.`))
          .map(([r, color]) => ({ verse: Number(r.split(".")[2]), color }));
      case "set_highlight":
        if (args.color) hl.set(args.reference as string, args.color as string); else hl.delete(args.reference as string);
        return null;
      case "get_strongs":
        return { status: "ok", entry: { key: args.key, lexicon: "LEX-HEB-OSHB", nodes: [
          { n: 0, parent: null, name: "def", text: null, attrs: [] },
          { n: 1, parent: 0, name: null, text: `Placeholder definition for ${args.key}.`, attrs: [] },
        ] } };
      case "load_state": return saved;
      case "bookmarks_list": return [];
      case "bookmark_set": case "bookmark_restore": return null;
      case "bookmark_clear": return null;
      case "save_state": saved = args.saved; return null;
      case "start_new_user_data": userError = null; return "user-kept-1761100000.sqlite";
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
const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);
const key = (target: EventTarget, k: string, init: KeyboardEventInit = {}) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true, ...init }));

let app: ReturnType<typeof mount>;
async function start() {
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  await settle();
}

beforeAll(start);
afterAll(() => unmount(app));

describe("reader screen", () => {
  it("starts at the first chapter, with no warning banner", () => {
    expect($(".banner")).toBeNull();
    expect($(".chapter-head")?.textContent).toBe("Testbook 1");
    expect(document.querySelectorAll(".verse").length).toBe(10);
    expect(document.documentElement.dataset.redLetters).toBe("on");
  });

  it("goes to a reference typed in the command box", async () => {
    const input = $<HTMLInputElement>("#cmd-input")!;
    input.focus();
    await settle();
    input.value = "te 2:5";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    expect($("#cmd-pop")?.textContent).toContain("Testbook 2:5");
    key(input, "Enter");
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Testbook 2");
    expect($('.verse[aria-current="true"]')?.dataset.v).toBe("5");
    expect(document.activeElement?.classList.contains("reader-scroll")).toBe(true);
  });

  it("shows an error for a chapter that does not exist, without navigating", async () => {
    const input = $<HTMLInputElement>("#cmd-input")!;
    input.focus();
    await settle();
    input.value = "testbook 9";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    expect($("#cmd-err")?.textContent).toContain("Testbook has 3 chapters.");
    key(input, "Escape");
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Testbook 2");
  });

  it("keys 1-5 toggle a whole-verse highlight on the current verse", async () => {
    const pane = $(".reader-scroll")!;
    pane.focus();
    key(pane, "3");
    await settle();
    expect(hl.get("Testbook.2.5")).toBe("blue");
    expect($('.verse[data-v="5"]')?.dataset.hl).toBe("blue");
    key(pane, "3");
    await settle();
    expect(hl.has("Testbook.2.5")).toBe(false);
    expect($('.verse[data-v="5"]')?.hasAttribute("data-hl")).toBe(false);
    key(pane, "1");
    await settle();
    expect(hl.get("Testbook.2.5")).toBe("gold");
  });

  it("changes chapter with Ctrl+PgDn and ]", async () => {
    key($(".reader-scroll")!, "PageDown", { ctrlKey: true });
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Testbook 3");
    expect($('.verse[aria-current="true"]')).toBeNull();
    $(".reader-scroll")!.focus();
    key($(".reader-scroll")!, "]");
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Shortbook"); // single-chapter book: name only
    key($(".reader-scroll")!, "[");
    await settle();
    expect($(".chapter-head")?.textContent).toBe("Testbook 3");
  });

  it("opens the Strong's entry in the study panel when a word is clicked", async () => {
    const input = $<HTMLInputElement>("#cmd-input")!;
    input.focus();
    await settle();
    input.value = "testbook 1";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await settle();
    key(input, "Enter");
    await settle();
    expect($("#p-study")).toBeNull(); // closed by default
    const w = $<HTMLElement>('.vtext .w[data-strong="H1"]')!;
    w.click();
    await settle();
    expect($("#p-study")).not.toBeNull();
    expect($("#p-study")?.textContent).toContain("Placeholder definition for H1.");
    key(document.body, "j", { ctrlKey: true }); // open, focus elsewhere: Ctrl+J focuses the panel
    await settle();
    expect($("#p-study")?.contains(document.activeElement)).toBe(true);
    key(document.activeElement!, "j", { ctrlKey: true }); // focus in the panel: Ctrl+J closes
    await settle();
    expect($("#p-study")).toBeNull();
    expect(document.querySelectorAll("[aria-expanded].w").length).toBe(0);
  });

  describe("the selected word (wave-2-layout.md 15)", () => {
    const words = () => [...document.querySelectorAll<HTMLElement>(".vtext .w")];
    const selected = () => words().filter((x) => x.hasAttribute("aria-expanded") || x.hasAttribute("aria-controls"));
    const texts = () => [...document.querySelectorAll<HTMLElement>(".vtext")].map((v) => v.innerHTML.replace(/ aria-(expanded|controls)="[^"]*"/g, ""));

    it("click selects (that word only), another click moves it, a click on it releases and closes", async () => {
      const before = texts();
      const [a, b] = words();
      a.click();
      await settle();
      expect(selected()).toEqual([a]);
      expect(a.getAttribute("aria-expanded")).toBe("true");
      expect(a.getAttribute("aria-controls")).toBe("p-study");
      expect($("#p-study")).not.toBeNull();
      b.click();
      await settle();
      expect(selected()).toEqual([b]);
      expect($("#p-study")).not.toBeNull();
      b.click();
      await settle();
      expect(selected()).toEqual([]);
      expect($("#p-study")).toBeNull();
      expect(texts()).toEqual(before); // DOM text and spans unchanged; attributes only
    });

    it("Enter and Space on a focused word select and release it; focus stays on the word", async () => {
      const w = words()[0];
      w.focus();
      key(w, "Enter");
      await settle();
      expect(selected()).toEqual([w]);
      expect(document.activeElement).toBe(w);
      expect($("#p-study")).not.toBeNull();
      expect(key(w, " ")).toBe(false); // Space doesn't scroll
      await settle();
      expect(selected()).toEqual([]);
      expect($("#p-study")).toBeNull();
    });

    it("Esc from the reading pane closes the panel and clears the selection", async () => {
      words()[1].click();
      await settle();
      const pane = $(".reader-scroll")!;
      pane.focus();
      key(pane, "Escape");
      await settle();
      expect($("#p-study")).toBeNull();
      expect(selected()).toEqual([]);
      expect(document.activeElement).toBe(pane);
    });

    it("Esc from inside the panel closes it and returns focus to the selected word", async () => {
      const w = words()[1];
      w.click();
      await settle();
      const close = $<HTMLButtonElement>('#p-study [aria-label="Close study panel"]')!;
      close.focus();
      key(close, "Escape");
      await settle();
      expect($("#p-study")).toBeNull();
      expect(selected()).toEqual([]);
      expect(document.activeElement).toBe(w);
    });

    it("the close button and the toggle clear the selection", async () => {
      words()[0].click();
      await settle();
      $<HTMLButtonElement>('#p-study [aria-label="Close study panel"]')!.click();
      await settle();
      expect(selected()).toEqual([]);
      words()[0].click();
      await settle();
      $<HTMLButtonElement>('[aria-label="Study panel"]')!.click();
      await settle();
      expect($("#p-study")).toBeNull();
      expect(selected()).toEqual([]);
    });

    it("a chapter change ends the selection; the panel keeps its entry", async () => {
      words()[0].click();
      await settle();
      key($(".reader-scroll")!, "PageDown", { ctrlKey: true });
      await settle();
      expect(selected()).toEqual([]);
      expect($("#p-study")?.textContent).toContain("Placeholder definition for");
      key($(".reader-scroll")!, "PageUp", { ctrlKey: true });
      await settle();
      $(".reader-scroll")!.focus();
      key($(".reader-scroll")!, "Escape");
      await settle();
      expect($("#p-study")).toBeNull();
    });
  });

  it("About opens from the menu, owns the keyboard while open, and gives focus back to the menu button", async () => {
    const menu = $<HTMLButtonElement>('[aria-label="Menu"]')!;
    menu.click();
    await settle();
    Array.from(document.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')).find((b) => b.textContent === "About Revelation")!.click();
    await settle();
    const d = $<HTMLDialogElement>("dialog.about")!;
    expect(d.open).toBe(true);
    expect(d.textContent).toContain("Made by C. Ethan Key");
    key(document.body, "j", { ctrlKey: true }); // app shortcut ignored behind the modal
    await settle();
    expect($("#p-study")).toBeNull();
    [...d.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === "Close")!.click();
    await settle();
    expect(d.open).toBe(false);
    expect(document.activeElement).toBe(menu);
  });

  it("shows 'Not available in this translation' for a translation without markup", async () => {
    const trBtn = Array.from(document.querySelectorAll<HTMLButtonElement>(".tb-end .btn")).find((b) => b.textContent === "PH1")!;
    trBtn.click();
    await settle();
    Array.from(document.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')).find((b) => b.textContent?.includes("PH2"))!.click();
    await settle();
    expect(document.querySelectorAll(".vtext .wj").length).toBe(0);
    ($('[aria-label="Reading options"]') as HTMLButtonElement).click();
    await settle();
    const sw = Array.from(document.querySelectorAll<HTMLElement>('[role="switch"]')).find((s) => s.textContent?.includes("Words of Jesus"))!;
    expect(sw.getAttribute("aria-disabled")).toBe("true");
    expect(sw.getAttribute("aria-checked")).toBe("false");
    expect(sw.textContent).toContain("Not available in this translation");
    sw.click(); // does nothing; the stored preference is untouched
    await settle();
    expect(document.documentElement.dataset.redLetters).toBe("on");
  });

  it("Ctrl+Shift+C opens the Commentary panel next to the text; it follows the current verse; Esc leaves it open", async () => {
    const pane = $(".reader-scroll")!;
    pane.focus();
    const before = calls.length;
    key(pane, "C", { ctrlKey: true, shiftKey: true });
    await settle();
    const comm = $("#p-comm")!;
    expect(comm).not.toBeNull();
    // order on the right: reader | the right column (commentary above study, 17)
    const kids = [...$(".app")!.children];
    expect(kids.indexOf($("#rcol")!)).toBe(kids.indexOf($(".reader")!) + 1);
    expect(comm.parentElement).toBe($("#rcol"));
    if ($("#p-study")) expect([...$("#rcol")!.children].indexOf($("#p-study")!)).toBeGreaterThan([...$("#rcol")!.children].indexOf(comm));
    expect(document.activeElement).toBe($("#p-comm .panel-title"));
    const q = calls.slice(before).filter((c) => c.cmd === "get_commentary");
    expect(q.length).toBe(1);
    expect($("#p-comm .cmt-block")).not.toBeNull();
    // W2-LBL: the owner approved the label, so the author and date lines show
    expect($("#p-comm .cmt-author")?.textContent).toBe(cmtGolden["John.3.3"].source.author_line);
    expect($("#p-comm .cmt-date")?.textContent).toBe(cmtGolden["John.3.3"].source.date_line);
    expect($("#p-comm .cmt-pending")).toBeNull();
    // a click on another verse makes it current: the panel follows it
    const v = [...document.querySelectorAll<HTMLElement>(".verse")].find((x) => x.dataset.v === "3")!;
    v.click();
    await settle();
    const last = calls.filter((c) => c.cmd === "get_commentary").at(-1)!;
    expect(last.args.verse).toBe(3);
    // Esc in the panel: focus to the reading pane, the panel stays (5)
    $<HTMLElement>("#p-comm .panel-title")!.focus();
    key(document.activeElement!, "Escape");
    await settle();
    expect(document.activeElement).toBe(pane);
    expect($("#p-comm")).not.toBeNull();
    // remembered in user.sqlite settings (4)
    await settle(120);
    expect((saved as { settings: { commentary_open: boolean } }).settings.commentary_open).toBe(true);
    // focus elsewhere -> focus the panel; in the panel -> close
    key(pane, "C", { ctrlKey: true, shiftKey: true });
    await settle();
    expect(document.activeElement).toBe($("#p-comm .panel-title"));
    key(document.activeElement!, "C", { ctrlKey: true, shiftKey: true });
    await settle(100); // the slide-out runs its course
    expect($("#p-comm")).toBeNull();
    expect(document.activeElement).toBe(pane);
    await settle(120);
    expect((saved as { settings: { commentary_open: boolean } }).settings.commentary_open).toBe(false);
  });

  it("saves the position and resumes there after a restart", async () => {
    await settle(600); // debounced save (500ms)
    const s = saved as { position: { translation: string; book: string; chapter: number } };
    expect(s.position).toMatchObject({ translation: "PH2", book: "Testbook", chapter: 1 });
    unmount(app);
    await start();
    expect($(".chapter-head")?.textContent).toBe("Testbook 1");
    expect(Array.from(document.querySelectorAll(".tb-end .btn")).some((b) => b.textContent === "PH2")).toBe(true);
    expect(calls.filter((c) => c.cmd === "save_state").length).toBeGreaterThan(0);
  });
});

describe("browser behaviour of the WebView (D5-F4)", () => {
  it("swallows reload, print, find and history keys, but not copy", () => {
    const pane = $(".reader-scroll")!;
    for (const [k, init] of [
      ["F5", {}], ["r", { ctrlKey: true }], ["R", { ctrlKey: true, shiftKey: true }], ["p", { ctrlKey: true }],
      ["f", { ctrlKey: true }], ["s", { ctrlKey: true }], ["ArrowLeft", { altKey: true }], ["F3", {}], ["F7", {}],
    ] as const) {
      expect(key(pane, k, init), `${k} ${JSON.stringify(init)}`).toBe(false); // dispatchEvent returns false when prevented
    }
    expect(key(pane, "c", { ctrlKey: true })).toBe(true);
  });

  it("shows no browser context menu over the text; text fields keep theirs", () => {
    const ev = () => new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
    expect($(".vtext")!.dispatchEvent(ev())).toBe(false);
    expect($("#cmd-input")!.dispatchEvent(ev())).toBe(true);
  });
});

describe("refused user data (D5-F5)", () => {
  it("offers a new data file, keeps the old one, and saving resumes", async () => {
    unmount(app);
    userError = "your highlights and settings file is incomplete (its companion file user.sqlite-wal is missing or damaged); it was left untouched";
    await start();
    expect($(".banner.warn")?.textContent).toContain("user.sqlite-wal is missing or damaged");
    const btn = $<HTMLButtonElement>(".banner.warn button")!;
    expect(btn.textContent?.trim()).toBe("Start a new data file (the old one is kept as a backup)");
    const before = calls.filter((c) => c.cmd === "save_state").length;
    btn.click();
    await settle();
    expect(calls.some((c) => c.cmd === "start_new_user_data")).toBe(true);
    expect($(".banner.warn")).toBeNull();
    expect($('.banner[role="status"]')?.textContent).toContain("The old one was kept as user-kept-1761100000.sqlite");
    expect(calls.filter((c) => c.cmd === "save_state").length).toBeGreaterThan(before);
    ($('.banner[role="status"] button') as HTMLButtonElement).click();
    await settle();
    expect($(".banner")).toBeNull();
  });

  it("a file from a newer build (W2A-U1): says so, offers no new data file, and writes nothing", async () => {
    unmount(app);
    userError = "the highlights and settings file was made by a newer version of Revelation (schema 5); open it with that version. It was left untouched";
    userTooNew = true;
    calls.length = 0;
    try {
      await start();
      expect($(".banner.warn")?.textContent).toContain("open it with that version");
      expect($(".banner.warn button")).toBeNull();
      expect(calls.some((c) => c.cmd === "start_new_user_data")).toBe(false);
      expect(calls.some((c) => c.cmd === "save_state" || c.cmd === "load_state")).toBe(false);
    } finally {
      userTooNew = false;
      userError = null;
    }
  });
});
