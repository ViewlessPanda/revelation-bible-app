// The notes panel in the whole reader screen (wave-2-layout.md 8), IPC mocked
// from the fixture golden as in app.test.ts. Typing is driven by paste and
// key events (ProseMirror's own handlers); the backend is a Map standing in
// for user.sqlite. Covers: open/close (Ctrl+Shift+N, the toggle, Esc), the
// autosave and its restore on reopen, flush on the window's close (and the
// question when a save failed), day groups across midnight, tags created from
// the reading context and clicked to jump, pasted HTML and links rendering
// inert, the backend's size refusal shown as "Not saved", and the verse DOM
// left exactly as it was.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushSync, mount, unmount } from "svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

interface GVerse { v: number; text: string; runs: unknown[] }
interface GChapter { book: number; osis: string; chapter: number; verses: GVerse[] }
interface GTr { abbrev: string; has_red_letters: boolean; chapters: GChapter[] }
const golden: { translations: GTr[] } = JSON.parse(
  readFileSync(resolve(process.cwd(), "src-tauri/tests/fixtures/placeholder-render.json"), "utf8"),
);

type Saved = { position: unknown; settings: Record<string, unknown> };
const base = { verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false, commentary_open: false, pen_color: "gold" };
let saved: Saved = { position: null, settings: { ...base } };
/** user.sqlite's note_day table */
const notesDb = new Map<string, string>();
const calls: { cmd: string; args: Record<string, unknown> }[] = [];
let saveFails: string | null = null;
/** save_state (the place) fails with this, for W2A-N3 */
let stateFails: string | null = null;
/** The backend's limit (notes.rs MAX_NOTE_BYTES), refused as the backend does. */
const MAX = 512 * 1024;

function books(tr: GTr) {
  const by = new Map<number, GChapter[]>();
  for (const c of tr.chapters) by.set(c.book, [...(by.get(c.book) ?? []), c]);
  return [...by.entries()].map(([ord, cs]) => ({
    ord, osis: cs[0].osis, name: cs[0].osis, heading: cs[0].osis, main_title: null, abbrevs: [], chapters: cs.length,
    verses: cs.map((c) => Math.max(...c.verses.map((v) => v.v))),
  }));
}

/**
 * The parts of notes.rs `check_doc` that a paste could reach (W2A-N1/N2):
 * serde_json refuses lone surrogate escapes; `order` is 0..99,999; `tags` is
 * null or at most 3 tags with only book/chapter/v1/v2 and an ASCII book id.
 */
function backendRefuses(doc: string): string | null {
  if (/\\ud[89ab][0-9a-f]{2}(?!\\ud[c-f][0-9a-f]{2})|(?<!\\ud[89ab][0-9a-f]{2})\\ud[c-f][0-9a-f]{2}/i.test(doc)) return "lone surrogate";
  let why: string | null = null;
  const walk = (n: { attrs?: Record<string, unknown>; content?: unknown[] }) => {
    const a = n.attrs ?? {};
    if ("order" in a && !(Number.isInteger(a.order) && (a.order as number) >= 0 && (a.order as number) <= 99_999)) why = "order";
    if ("tags" in a && a.tags !== null) {
      const ok = Array.isArray(a.tags) && a.tags.length <= 3 && a.tags.every((t) => t && typeof t === "object"
        && Object.keys(t).every((k) => ["book", "chapter", "v1", "v2"].includes(k))
        && typeof (t as { book: unknown }).book === "string" && /^[A-Za-z0-9]{1,16}$/.test((t as { book: string }).book));
      if (!ok) why = "tags";
    }
    for (const c of n.content ?? []) walk(c as typeof n);
  };
  walk(JSON.parse(doc));
  return why;
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
    calls.push({ cmd, args });
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
      case "marks_for_chapter": return [];
      case "load_state": return saved;
      case "bookmarks_list": return [];
      case "bookmark_set": return null;
      case "save_state": if (stateFails) throw stateFails; saved = args.saved as Saved; return null;
      case "notes_list": {
        const days = [...notesDb.keys()].sort().reverse().filter((d) => !args.before || d < (args.before as string));
        return { days: days.slice(0, args.limit as number).map((day) => ({ day, doc: notesDb.get(day)!, updated_at: 0 })), more: days.length > (args.limit as number) };
      }
      case "note_save": {
        if (saveFails) throw saveFails;
        const doc = args.doc as string;
        if (doc.length > MAX) throw "invalid value: the note is too long";
        const bad = backendRefuses(doc);
        if (bad) throw `not a note document: ${bad}`;
        const d = JSON.parse(doc);
        const text = JSON.stringify(d).match(/"text":"[^"]/);
        if (text) notesDb.set(args.day as string, doc); else notesDb.delete(args.day as string);
        return null;
      }
      case "app_close": return null;
    }
    throw new Error(`unexpected command ${cmd}`);
  },
}));

const settle = async (ms = 0) => {
  for (let i = 0; i < 5; i++) { await new Promise((r) => setTimeout(r, ms)); flushSync(); }
};
/** Wait `ms` once (real time), then let the promises settle. */
const sleep = async (ms: number) => { await new Promise((r) => setTimeout(r, ms)); await settle(); };
const $ = <T extends Element = HTMLElement>(s: string) => document.querySelector<T>(s);
const $$ = (s: string) => [...document.querySelectorAll<HTMLElement>(s)];
const key = (target: EventTarget, k: string, init: KeyboardEventInit = {}) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true, ...init }));
const editor = () => $(".note-live .ProseMirror")!;
/** Paste into the live editor through ProseMirror's own paste handler. */
function paste(data: { text?: string; html?: string }) {
  const dt = new DataTransfer();
  if (data.html !== undefined) dt.setData("text/html", data.html);
  dt.setData("text/plain", data.text ?? "");
  const e = new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true });
  editor().dispatchEvent(e);
  flushSync();
}
const enter = () => { key(editor(), "Enter"); flushSync(); };
const savesOf = (day: string) => calls.filter((c) => c.cmd === "note_save" && c.args.day === day);
const TODAY = "2026-10-27";
const note = (day: string) => JSON.parse(notesDb.get(day) ?? "null");

let app: ReturnType<typeof mount> | null = null;
async function start(settings: Record<string, unknown> = {}) {
  saved = { position: null, settings: { ...base, ...settings } };
  document.body.innerHTML = '<div id="app"></div>';
  const { default: App } = await import("../src/App.svelte");
  app = mount(App, { target: $("#app")! });
  await settle();
}
async function openNotes() {
  key(window, "N", { ctrlKey: true, shiftKey: true });
  await settle();
  await settle(); // the first page of notes loads
}
/** Click a verse: it becomes the current verse, the reading context (8.4). */
async function readVerse(v: number) {
  $(`.verse[data-v="${v}"]`)!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  await settle();
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(new Date(2026, 9, 27, 21, 0, 0));
  notesDb.clear();
  calls.length = 0;
  saveFails = null;
  stateFails = null;
});
afterEach(() => {
  if (app) unmount(app);
  app = null;
  vi.useRealTimers();
});

describe("the notes panel opens on the left", () => {
  it("closed on first run; Ctrl+Shift+N opens it with the caret in today's note; Esc goes to the reading pane; the toggle closes it", async () => {
    await start();
    expect($("#p-notes")).toBeNull();
    await openNotes();
    const panel = $("#p-notes")!;
    expect(panel).not.toBeNull();
    // left of the reader, in the same row
    expect(panel.nextElementSibling?.classList.contains("reader")).toBe(true);
    const full = new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric", month: "long", year: "numeric" }).format(new Date(2026, 9, 27));
    expect($$(".date-head").map((h) => [h.querySelector(".dh-name")!.textContent, h.querySelector(".dh-date")!.textContent, h.getAttribute("aria-label")]))
      .toEqual([["Today", full, `Today, ${full}`]]);
    expect($("section.day")!.getAttribute("aria-labelledby")).toBe($("h2.date-head")!.id);
    expect(editor().querySelector("p.is-empty")?.getAttribute("data-placeholder")).toBe("Write about what you're reading…");
    expect(document.activeElement).toBe(editor());
    expect($('button[aria-label="Notes panel"]')!.getAttribute("aria-pressed")).toBe("true");
    key(editor(), "Escape");
    await settle();
    expect(document.activeElement).toBe($(".reader-scroll"));
    expect($("#p-notes")).not.toBeNull(); // Esc only moves focus (5)
    $('button[aria-label="Notes panel"]')!.click();
    await sleep(400);
    expect($("#p-notes")).toBeNull();
    await sleep(600);
    expect(saved.settings.notes_open).toBe(false);
  });

  it("the open state is remembered", async () => {
    await start({ notes_open: true });
    await settle();
    expect($("#p-notes")).not.toBeNull();
    expect(saved.settings).toBeTruthy();
  });
});

describe("autosave: save and restore across reopen, debounce, flush on close", () => {
  it("a pause of 500ms saves the day; reopening shows it, tagged with the passage, newest first", async () => {
    notesDb.set("2026-10-25", JSON.stringify({ type: "doc", content: [{ type: "paragraph", attrs: { tags: [{ book: "Testbook", chapter: 2, v1: 4, v2: 4 }] }, content: [{ type: "text", text: "an older note" }] }] }));
    await start();
    await readVerse(3);
    await openNotes();
    expect($(".notes-toolbar .tagging")!.textContent).toBe("Tagging Testbook 1:3");
    paste({ text: "First thought" });
    expect(editor().textContent).toContain("First thought");
    await sleep(200);
    expect(savesOf(TODAY)).toHaveLength(0); // still typing
    paste({ text: " and more" });
    await sleep(300);
    expect(savesOf(TODAY)).toHaveLength(0);
    await sleep(250);
    expect(savesOf(TODAY)).toHaveLength(1); // one save, 500ms after the last change
    const d = note(TODAY);
    expect(d.content[0]).toEqual({
      type: "paragraph", attrs: { tags: [{ book: "Testbook", chapter: 1, v1: 3, v2: 3 }] },
      content: [{ type: "text", text: "First thought and more" }],
    });
    expect($(".save-state")!.textContent!.trim()).toBe("Saved");

    // reopen the app: the notes come back from user.sqlite
    unmount(app!);
    app = null;
    await start({ notes_open: true });
    await settle();
    await settle();
    expect($$(".date-head").map((h) => h.querySelector(".dh-name")!.textContent)).toEqual(["Today", "Sunday"]);
    expect(editor().textContent).toContain("First thought and more");
    expect($$(".note-live .tag-chip").map((c) => c.getAttribute("aria-label"))).toEqual(["Go to Testbook 1:3"]);
    // an earlier day is static until clicked, its chip a normal button
    const older = $(".note-static")!;
    expect(older.textContent).toContain("an older note");
    expect(older.querySelector<HTMLButtonElement>(".tag-chip")!.tabIndex).toBe(0);
    expect(older.querySelector("p")!.getAttribute("aria-description")).toBe("Passage: Testbook 2:4");
  });

  it("the window's close saves at once (no 500ms wait), then closes", async () => {
    await start();
    await readVerse(2);
    await openNotes();
    paste({ text: "Last words before closing" });
    expect(savesOf(TODAY)).toHaveLength(0);
    (window as unknown as { __revelationClose(): void }).__revelationClose();
    await settle();
    expect(savesOf(TODAY)).toHaveLength(1);
    expect(note(TODAY).content[0].content[0].text).toBe("Last words before closing");
    const close = calls.filter((c) => c.cmd === "app_close");
    expect(close.map((c) => c.args.hold)).toEqual([false]);
    // the place is saved too, after the note
    const order = calls.map((c) => c.cmd).filter((c) => c === "note_save" || c === "app_close");
    expect(order).toEqual(["note_save", "app_close"]);
  });

  it("closing the panel saves at once too", async () => {
    await start();
    await openNotes();
    paste({ text: "panel closing" });
    key(editor(), "N", { ctrlKey: true, shiftKey: true }); // focus in Notes: Ctrl+Shift+N closes
    await settle();
    expect(savesOf(TODAY)).toHaveLength(1);
  });

  it("a failed save shows Not saved and the close asks first; Close anyway closes", async () => {
    await start();
    await openNotes();
    saveFails = "disk full";
    paste({ text: "unsaved" });
    await sleep(600);
    const st = $(".save-state.failed")!;
    expect(st.textContent!.trim()).toBe("Not saved");
    st.click();
    flushSync();
    expect($(".save-pop")!.textContent).toContain("disk full");
    (window as unknown as { __revelationClose(): void }).__revelationClose();
    await settle();
    expect(calls.filter((c) => c.cmd === "app_close").map((c) => c.args.hold)).toEqual([true]);
    const dlg = $<HTMLDialogElement>("dialog.close-ask")!;
    expect(dlg.open).toBe(true);
    [...dlg.querySelectorAll("button")].find((b) => b.textContent === "Close anyway")!.click();
    await settle();
    expect(calls.filter((c) => c.cmd === "app_close").map((c) => c.args.hold)).toEqual([true, false]);
  });

  it("an unrelated failed write (the place) keeps the editor and the close question (W2A-N3)", async () => {
    await start();
    await openNotes();
    saveFails = "disk full";
    paste({ text: "unsaved" });
    await sleep(600);
    expect($(".save-state.failed")!.textContent!.trim()).toBe("Not saved");
    stateFails = "disk full";
    await readVerse(3); // the place is saved, and fails
    await sleep(600);
    expect($(".banner.warn")!.textContent).toContain("Highlights and your place can't be saved right now");
    // a later write error doesn't offer to replace the data file, and Notes stays live
    expect([...document.querySelectorAll(".banner.warn button")].map((b) => b.textContent!.trim())).not.toContain("Start a new data file (the old one is kept as a backup)");
    expect($(".note-live .ProseMirror")).not.toBeNull();
    expect($(".notes-msg")).toBeNull();
    (window as unknown as { __revelationClose(): void }).__revelationClose();
    await settle();
    expect(calls.filter((c) => c.cmd === "app_close").map((c) => c.args.hold)).toEqual([true]);
    expect($<HTMLDialogElement>("dialog.close-ask")!.open).toBe(true);
    // keep it open; the disk recovers; the editor kept working, and the next close saves and closes
    [...$("dialog.close-ask")!.querySelectorAll("button")].find((b) => b.textContent === "Keep Revelation open")!.click();
    flushSync();
    saveFails = null;
    paste({ text: " now" });
    (window as unknown as { __revelationClose(): void }).__revelationClose();
    await settle();
    expect(note(TODAY).content[0].content[0].text).toBe("unsaved now");
    expect($(".save-state.failed")).toBeNull();
    expect(calls.filter((c) => c.cmd === "app_close").map((c) => c.args.hold)).toEqual([true, false]);
  });

  it("the backend's size limit is enforced: a too-long note is refused and shown as Not saved", async () => {
    await start();
    await openNotes();
    paste({ text: "x".repeat(MAX) });
    await sleep(600);
    expect(notesDb.has(TODAY)).toBe(false);
    expect($(".save-state.failed")!.textContent).toContain("Not saved");
  });

  it("Not saved offers 'Copy this day's text', which puts the day's words on the clipboard", async () => {
    let clip = "";
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async (t: string) => { clip = t; } } });
    await start();
    await openNotes();
    saveFails = "disk full";
    paste({ text: "first line" });
    enter();
    paste({ text: "second line" });
    await sleep(600);
    $(".save-state.failed")!.click();
    flushSync();
    [...document.querySelectorAll<HTMLButtonElement>(".save-pop button")].find((b) => b.textContent === "Copy this day's text")!.click();
    await settle();
    expect(clip).toBe("first line\nsecond line");
    expect($(".save-pop")!.textContent).toContain("Copied.");
    Reflect.deleteProperty(navigator, "clipboard");
  });
});

describe("pasted slice contexts and broken text can't stop or wedge saving (W2A-N1, N2)", () => {
  const ctxPaste = (ctx: string, text: string) =>
    paste({ html: `<p data-pm-slice='0 0 ${ctx}'>${text}</p>`, text });
  const allText = () => JSON.stringify(note(TODAY)).match(/"text":"[^"]*"/g)!.map((s) => s.slice(8, -1)).join("");

  it("a non-list tags context (N1): the editor keeps drawing, every later edit is saved, the status is honest", async () => {
    await start();
    await readVerse(2);
    await openNotes();
    ctxPaste('["bullet_list",{"tags":"AAAA"},"list_item",null]', "pasted");
    paste({ text: "one" });
    paste({ text: "two" });
    await sleep(600);
    expect(editor().textContent).toContain("pastedonetwo");
    expect(savesOf(TODAY).length).toBeGreaterThan(0);
    expect(allText()).toBe("pastedonetwo");
    expect($(".save-state.failed")).toBeNull();
    // no tags from the clipboard: only the reading context's
    expect(JSON.stringify(note(TODAY))).not.toContain("AAAA");
    (window as unknown as { __revelationClose(): void }).__revelationClose();
    await settle();
    expect(calls.filter((c) => c.cmd === "app_close").map((c) => c.args.hold)).toEqual([false]);
  });

  it.each([
    ["order past 99,999", '["ordered_list",{"order":1000000000},"list_item",null]'],
    ["object tags", '["bullet_list",{"tags":{"x":1}},"list_item",null]'],
    ["a book that isn't a book id", '["bullet_list",{"tags":[{"book":"Gospel of Thomas","chapter":1,"v1":0,"v2":0}]},"list_item",null]'],
    ["a tag with an extra key", '["bullet_list",{"tags":[{"book":"John","chapter":1,"v1":0,"v2":0,"x":1}]},"list_item",null]'],
    ["four tags", `["bullet_list",{"tags":${JSON.stringify(Array(4).fill({ book: "John", chapter: 1, v1: 0, v2: 0 }))}},"list_item",null]`],
  ])("a pasted context with %s (N2) is saved, within the backend's rules", async (_name, ctx) => {
    await start();
    await openNotes();
    ctxPaste(ctx, "pasted");
    paste({ text: " more" });
    await sleep(600);
    expect($(".save-state.failed")).toBeNull();
    expect(backendRefuses(notesDb.get(TODAY)!)).toBeNull();
    expect(allText()).toBe("pasted more");
  });

  it("a lone surrogate in pasted text (N2) is saved as U+FFFD; whole emoji are kept", async () => {
    await start();
    await openNotes();
    paste({ text: "abc\uD83D def \uDE00 😀" });
    await sleep(600);
    expect($(".save-state.failed")).toBeNull();
    expect(note(TODAY).content[0].content[0].text).toBe("abc� def � 😀");
  });
});

describe("days across midnight (8.1)", () => {
  it("a note keeps the day it was started on; after midnight a new empty Today appears above it", async () => {
    vi.setSystemTime(new Date(2026, 9, 27, 23, 59, 30));
    await start();
    await openNotes();
    paste({ text: "late" });
    vi.setSystemTime(new Date(2026, 9, 28, 0, 0, 10));
    window.dispatchEvent(new Event("focus"));
    await settle();
    paste({ text: " still typing" });
    await sleep(600);
    expect($$(".date-head .dh-name").map((h) => h.textContent)).toEqual(["Today", "Yesterday"]);
    expect(note("2026-10-27").content[0].content[0].text).toBe("late still typing");
    expect(notesDb.has("2026-10-28")).toBe(false); // an empty day isn't stored
    expect($(".note-static.note-empty")!.textContent).toBe("Write about what you're reading…");
  });
});

describe("passage tags (8.4)", () => {
  it("new paragraphs typed on another verse get their own chip row; a chip click jumps there with focus kept", async () => {
    await start();
    await readVerse(3);
    await openNotes();
    paste({ text: "On verse three" });
    enter();
    paste({ text: "Same passage, same chip row" });
    expect($$(".note-live .tag-row")).toHaveLength(1);
    expect($$(".note-live .tag-chip").map((c) => c.textContent)).toEqual(["Testbook 1:3"]);
    // the reader moves on while still writing: the composing paragraph's range grows (3 -> 3-5); the first is frozen
    await readVerse(5);
    expect($$(".note-live .tag-chip").map((c) => c.textContent)).toEqual(["Testbook 1:3", "Testbook 1:3–5"]);
    // a new paragraph is tagged with the context when its first character is typed
    enter();
    await readVerse(7);
    paste({ text: "On verse seven" });
    expect($$(".note-live .tag-chip").map((c) => c.textContent)).toEqual(["Testbook 1:3", "Testbook 1:3–5", "Testbook 1:7"]);
    // never in the text: the chips sit between the paragraphs, not in them
    expect($$(".note-live .ProseMirror > p").map((p) => p.textContent)).toEqual(["On verse three", "Same passage, same chip row", "On verse seven"]);
    // jump: go to chapter 2 first, then click the first chip
    key(window, "PageDown", { ctrlKey: true });
    await settle();
    expect($(".verse.current, .verse[aria-current]")).toBeNull();
    editor().focus();
    const chip = $$(".note-live .tag-chip")[1];
    chip.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    chip.click();
    await settle();
    expect($(".reader-scroll")!.getAttribute("aria-label")).toBe("Testbook 1");
    expect($('.verse[data-v="3"]')!.getAttribute("aria-current")).toBeTruthy();
    expect($$(".verse.flashing").map((v) => v.dataset.v)).toEqual(["3", "4", "5"]);
    expect(document.activeElement).toBe(editor()); // the editor keeps focus (8.4 Click)
    // the jump ended composing: the paragraph's tag didn't grow to the jumped-to verse
    expect($$(".note-live .tag-chip").map((c) => c.textContent)).toEqual(["Testbook 1:3", "Testbook 1:3–5", "Testbook 1:7"]);
  });

  it("the × removes a tag (undoable)", async () => {
    await start();
    await readVerse(4);
    await openNotes();
    paste({ text: "tagged" });
    $(".note-live .tag-x")!.click();
    flushSync();
    expect($$(".note-live .tag-chip")).toHaveLength(0);
    key(editor(), "z", { ctrlKey: true });
    flushSync();
    expect($$(".note-live .tag-chip").map((c) => c.textContent)).toEqual(["Testbook 1:4"]);
  });
});

describe("XSS strings render inert; the verse DOM is untouched", () => {
  it("pasted HTML, javascript: links and markdown become plain text in the four formats", async () => {
    await start();
    await openNotes();
    paste({
      html: '<p>a<img src=x onerror="window.__pwned=1"><a href="javascript:window.__pwned=1">link</a><script>window.__pwned=1</script><b>bold</b></p>',
      text: "ignored",
    });
    enter();
    paste({ text: "[x](javascript:alert(1)) **md** <img src=x onerror=alert(1)>" });
    await sleep(600);
    const live = $(".note-live")!;
    expect(live.querySelector("img, a, script, iframe, [onerror], [href]")).toBeNull();
    expect(live.querySelector("strong")!.textContent).toBe("bold");
    expect(editor().textContent).toContain("<img src=x onerror=alert(1)>");
    expect((window as unknown as { __pwned?: number }).__pwned).toBeUndefined();
    const json = notesDb.get(TODAY)!;
    expect(json).not.toMatch(/"(link|image|href|src)"/);
    // reopened as a static day, still text
    unmount(app!);
    app = null;
    vi.setSystemTime(new Date(2026, 9, 28, 9, 0, 0));
    await start({ notes_open: true });
    await settle();
    await settle();
    const old = $(".note-static")!;
    expect(old.textContent).toContain("[x](javascript:alert(1)) **md** <img src=x onerror=alert(1)>");
    expect(old.querySelector("img, a, script, [onerror], [href]")).toBeNull();
  });

  it("writing notes and tagging never touches the verse DOM (colour lock)", async () => {
    await start();
    const before = $$(".vtext").map((v) => v.outerHTML);
    await readVerse(2);
    const afterClick = $$(".vtext").map((v) => v.outerHTML);
    expect(afterClick).toEqual(before);
    await openNotes();
    paste({ text: "words" });
    enter();
    paste({ html: "<b>more</b>", text: "more" });
    await sleep(600);
    expect($$(".vtext").map((v) => v.outerHTML)).toEqual(before);
  });
});
