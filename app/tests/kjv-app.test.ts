// The reader screen on the REAL KJV golden (IPC mocked from
// src-tauri/tests/golden/kjv-render.json, machine-generated through
// open_verified; no Scripture is typed here):
// - a content checksum failure shows the repair/reinstall message and no text;
// - copy puts the exact stored text plus the reference on the clipboard;
// - clicking a Strong's-tagged word opens the panel with an entry for each of
//   its keys (entries from the real lexicon golden, lexicon-entries.json), or
//   the "lexicon unavailable" state when content.sqlite is missing/damaged.
import { afterAll, describe, expect, it, vi } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { kjv } from "./kjv-golden";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { LexEntry } from "../src/lib/types";

const lexGolden: { entries: { entry: LexEntry }[] } = JSON.parse(
  readFileSync(resolve(process.cwd(), "src-tauri/tests/golden/lexicon-entries.json"), "utf8"),
);
const LEX_ERROR = "content.sqlite: failed its checksum (damaged or replaced)";

const calls: string[] = [];
const mode = { broken: true, lexicon: true };
const john = kjv.chapters.find((c) => c.osis === "John" && c.chapter === 3)!;
const johnBook = kjv.books.find((b) => b.osis === "John")!;
const CHECKSUM_ERROR = "content file failed its checksum (damaged or replaced): kjv.sqlite";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
    calls.push(cmd);
    switch (cmd) {
      case "app_info":
        return mode.broken
          ? { content_error: CHECKSUM_ERROR, user_error: null, translations: [], lexicon_available: false }
          : {
              content_error: null, user_error: null, translations: [kjv.translation],
              lexicon_available: mode.lexicon, lexicon_error: mode.lexicon ? null : LEX_ERROR,
            };
      case "list_books": return kjv.books;
      case "get_chapter": {
        const c = kjv.chapters.find((x) => x.book === args.book && x.chapter === args.chapter);
        if (!c) throw "not found";
        return { book: c.book, osis: c.osis, chapter: c.chapter, verses: c.verses.map(({ v, text, runs }) => ({ v, text, runs })) };
      }
      case "get_highlights": return [];
      case "get_strongs": {
        const e = lexGolden.entries.find((x) => x.entry.key === args.key);
        if (!mode.lexicon || !e) return { status: "not_available", reason: mode.lexicon ? "not in the test golden" : LEX_ERROR };
        return { status: "ok", entry: e.entry };
      }
      case "load_state":
        return {
          position: { translation: "KJV", book: "John", chapter: 3, anchor_verse: null, anchor_frac: null, current_verse: null },
          settings: { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, right_width_rem: 22, right_split: 0.5 },
        };
      case "bookmarks_list": return [];
      case "bookmark_set": case "bookmark_restore": return null;
      case "bookmark_clear": return null;
      case "save_state": return null;
    }
    throw new Error(`unexpected command ${cmd}`);
  },
}));

const settle = async (ms = 0) => {
  for (let i = 0; i < 5; i++) { await new Promise((r) => setTimeout(r, ms)); flushSync(); }
};
const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);

let app: ReturnType<typeof mount> | null = null;
async function start() {
  if (app) unmount(app);
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  await settle();
}
afterAll(() => { if (app) unmount(app); });

function textNodes(root: Node): Text[] {
  const out: Text[] = [];
  const walk = document.createTreeWalker(root, 4 /* SHOW_TEXT */);
  for (let n = walk.nextNode(); n; n = walk.nextNode()) out.push(n as Text);
  return out;
}
const vtext = (v: number) => $<HTMLElement>(`.verse[data-v="${v}"] > .vtext`)!;

/** DOM point at UTF-16 offset `off` of a verse's text (as the copy handler counts). */
function pointAt(v: number, off: number): [Text, number] {
  let pos = 0;
  for (const t of textNodes(vtext(v))) {
    if (off <= pos + t.data.length) return [t, off - pos];
    pos += t.data.length;
  }
  throw new Error("offset beyond verse");
}

function copySelection(from: [number, number], to: [number, number]): string | null {
  const r = document.createRange();
  r.setStart(...pointAt(...from));
  r.setEnd(...pointAt(...to));
  const sel = document.getSelection()!;
  sel.removeAllRanges();
  sel.addRange(r);
  let data: string | null = null;
  const ev = new Event("copy", { bubbles: true, cancelable: true });
  Object.defineProperty(ev, "clipboardData", { value: { setData: (_t: string, d: string) => { data = d; } } });
  vtext(from[0]).dispatchEvent(ev);
  expect(ev.defaultPrevented).toBe(true);
  sel.removeAllRanges();
  return data;
}

describe("real KJV in the reader", () => {
  it("shows the repair/reinstall message and no text when the checksum fails", async () => {
    mode.broken = true;
    await start();
    expect($("main.fatal")?.textContent).toContain("Please repair or reinstall Revelation.");
    expect($("main.fatal")?.textContent).toContain(CHECKSUM_ERROR);
    expect(document.querySelectorAll(".vtext, .verse").length).toBe(0);
    expect(calls).not.toContain("get_chapter");
    expect(calls).not.toContain("list_books");
  });

  it("opens John 3 from the KJV with every verse's stored text", async () => {
    mode.broken = false;
    await start();
    expect($("main.fatal")).toBeNull();
    expect($(".chapter-head")?.textContent).toBe(`${johnBook.name} 3`);
    // W2-B: the translation button shows the abbreviation, its name is the owner-approved label
    const tr = $('button[aria-haspopup="menu"][title="King James Version (1769)"]');
    expect(tr?.textContent).toBe("KJV");
    expect(Array.from(document.querySelectorAll(".verse > .vtext")).map((e) => e.textContent)).toEqual(john.verses.map((v) => v.text));
  });

  it("copy across verses gives the exact stored text plus the reference", () => {
    const t16 = john.verses.find((v) => v.v === 16)!.text;
    const t17 = john.verses.find((v) => v.v === 17)!.text;
    const a = Math.floor(t16.length / 3), b = Math.floor(t17.length / 2);
    expect(copySelection([16, a], [17, b])).toBe(`${t16.slice(a)}\n${t17.slice(0, b)}\n${johnBook.name} 3:16–17 (KJV)`);
  });

  it("copy of a whole verse and of part of one verse", () => {
    const v = john.verses.find((x) => x.red.length > 0 && x.red[0][0] > 0)!; // red starts mid-verse
    expect(copySelection([v.v, 0], [v.v, v.text.length])).toBe(`${v.text}\n${johnBook.name} 3:${v.v} (KJV)`);
    // a selection that starts exactly at a red boundary keeps every character
    const s = Array.from(v.text).slice(0, v.red[0][0]).join("").length;
    expect(copySelection([v.v, s], [v.v, v.text.length])).toBe(`${v.text.slice(s)}\n${johnBook.name} 3:${v.v} (KJV)`);
  });

  it("a Strong's word with two keys shows both entries, as stored", async () => {
    // John 3:2: the first word whose keys are both in the lexicon golden
    const v2 = john.verses.find((v) => v.v === 2)!;
    const have = new Set(lexGolden.entries.map((e) => e.entry.key));
    const word = v2.words.find((w) => w[2].split(" ").length === 2 && w[2].split(" ").every((k) => have.has(k)))!;
    const keys = word[2].split(" ");
    const w = [...vtext(2).querySelectorAll<HTMLElement>(".w")].find((x) => x.dataset.strong === word[2])!;
    w.click();
    await settle();
    const panel = $("#p-study")!;
    expect(panel).not.toBeNull();
    expect([...panel.querySelectorAll<HTMLElement>("section.lex-entry")].map((x) => x.dataset.key)).toEqual(keys);
    for (const k of keys) {
      const entry = lexGolden.entries.find((x) => x.entry.key === k)!.entry;
      const sec = panel.querySelector<HTMLElement>(`section[data-key="${k}"]`)!;
      for (const el of sec.querySelectorAll<HTMLElement>(".lt")) {
        expect(el.textContent).toBe(entry.nodes[Number(el.dataset.n)].text);
      }
      expect(sec.querySelectorAll(".lt").length).toBeGreaterThan(0);
    }
    expect(panel.textContent).toContain(`“${w.textContent}”`);
    expect(panel.querySelector("footer.lex-credit")).not.toBeNull();
    // a cross-reference opens its target (here: one that is in the golden)
    const ref = [...panel.querySelectorAll<HTMLButtonElement>("button.lex-ref")].find((b) => have.has(b.dataset.ref!));
    if (ref) {
      ref.click();
      await settle();
      expect([...panel.querySelectorAll<HTMLElement>("section.lex-entry")].map((x) => x.dataset.key)).toEqual([ref.dataset.ref]);
    }
  });

  it("with the lexicon missing or damaged the text still shows and the panel says why", async () => {
    mode.lexicon = false;
    await start();
    expect(vtext(16).textContent).toBe(john.verses.find((v) => v.v === 16)!.text);
    const w = vtext(16).querySelector<HTMLElement>(".w")!;
    const keys = w.dataset.strong!.split(" ");
    w.click();
    await settle();
    const panel = $("#p-study")!;
    expect(panel.querySelectorAll("section.lex-entry").length).toBe(keys.length);
    expect(panel.querySelectorAll(".lt").length).toBe(0);
    expect(panel.textContent).toContain("The lexicon is unavailable");
    expect(panel.textContent).toContain(LEX_ERROR);
    mode.lexicon = true;
  });
});
