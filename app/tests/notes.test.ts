// Daily notes, the pure parts (wave-2-layout.md 8): the schema and its
// parsing of pasted HTML (XSS strings stay inert text), passage tags
// (creation, extension, freeze, split, join, removal; never text), the
// autosave (debounce, flush, failure and retry), days across midnight, the
// reading context, and the static rendering of earlier days.
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState, TextSelection, type Transaction } from "prosemirror-state";
import { DOMParser as PMParser, type Node as PMNode } from "prosemirror-model";
import { asTags, emptyDoc, extend, merge, schema, tagLabel, union, validTag, validTags, type PassageTag } from "../src/lib/notes-schema";
import { COMPOSE_IDLE_MS, chipRuns, keepTags, tagKey, tagWithContext } from "../src/lib/notes-tags";
import { joinBackward } from "prosemirror-commands";
import { cleanSlice, docJSON, docText, notesState, parseDoc, renderStatic, toggleList, wellFormed } from "../src/lib/notes-editor";
import { Fragment, Slice } from "prosemirror-model";
import { createSaver, type SaveStatus, type Timers } from "../src/lib/notes-save";
import { dayHeading, dayOrder, halfVisible, localDay, msToMidnight, readingContext } from "../src/lib/notes-days";
import { caretGeometry, snaps } from "../src/lib/notes-caret";
import type { Book } from "../src/lib/types";

const J316: PassageTag = { book: "John", chapter: 3, v1: 16, v2: 16 };
const J318: PassageTag = { book: "John", chapter: 3, v1: 18, v2: 18 };
const J41: PassageTag = { book: "John", chapter: 4, v1: 1, v2: 1 };
const BOOKS = [
  { ord: 43, osis: "John", name: "John", heading: "John", main_title: null, abbrevs: [], chapters: 21, verses: [] },
  { ord: 19, osis: "Ps", name: "Psalm", heading: "Psalms", main_title: null, abbrevs: [], chapters: 150, verses: [] },
  { ord: 65, osis: "Jude", name: "Jude", heading: "Jude", main_title: null, abbrevs: [], chapters: 1, verses: [] },
] as unknown as Book[];
const label = (t: PassageTag) => tagLabel(t, BOOKS) ?? t.book;

let clock = 1_000_000;
const opts = (jump = (_t: PassageTag) => {}) => ({
  tags: { label, jump, now: () => clock },
  caret: { layer: () => null },
  jump,
});

/** A state-only editor: transactions run through every plugin (appendTransaction included). */
function ed(doc: PMNode = emptyDoc()) {
  let state = notesState(doc, opts());
  const run = (f: (s: EditorState) => Transaction) => { state = state.apply(f(state)); };
  return {
    get state() { return state; },
    run,
    ctx(t: PassageTag | null) { run((s) => s.tr.setMeta(tagKey, { ctx: t }).setMeta("addToHistory", false)); },
    type(text: string) { run((s) => s.tr.insertText(text)); },
    at(pos: number) { run((s) => s.tr.setSelection(TextSelection.create(s.doc, pos))); },
    tags(i: number) { return state.doc.child(i).attrs.tags; },
    cmd(c: (s: EditorState, d?: (tr: Transaction) => void) => boolean) { return c(state, (tr) => { state = state.apply(tr); }); },
    end() { run((s) => s.tr.setMeta(tagKey, { end: true })); },
  };
}

describe("schema: the wave-2 set only; pasted HTML is parsed into it", () => {
  const parse = (html: string) => {
    // an inert document (nothing in it runs or loads), as ProseMirror's own clipboard parsing uses
    const doc = new DOMParser().parseFromString(`<body>${html}</body>`, "text/html");
    return PMParser.fromSchema(schema).parse(doc.body);
  };
  it("keeps bold, italic, lists and breaks; drops everything else to text", () => {
    const d = parse('<h1>Title</h1><p><b>bold</b> <i>it</i><br>x</p><ul><li>one</li></ul><ol start="3"><li>three</li></ol><blockquote>q</blockquote>');
    expect(d.toJSON().content.map((n: { type: string }) => n.type)).toEqual(["paragraph", "paragraph", "bullet_list", "ordered_list", "paragraph"]);
    expect(d.textContent).toBe("Titlebold itxonethreeq");
    expect(d.child(2).firstChild!.type.name).toBe("list_item");
    expect(d.child(3).attrs.order).toBe(3);
  });
  it("XSS strings render inert: no links, no handlers, no scripts, no images", () => {
    const evil = [
      '<a href="javascript:alert(1)">click</a>',
      '<img src=x onerror="alert(1)">',
      '<script>alert(1)</script>after',
      '<p onclick="alert(1)" style="color:red" class="x">p</p>',
      '<svg><a xlink:href="javascript:alert(1)">s</a></svg>',
      '<ol start="-5"><li>neg</li></ol><ol start="9999999"><li>big</li></ol>',
    ].join("");
    const d = parse(evil);
    const json = docJSON(d);
    expect(json).not.toMatch(/javascript|onerror|onclick|script|iframe|href|src|style|class/);
    expect(d.textContent).toBe("clickafterpsnegbig");
    const lists: number[] = [];
    d.descendants((n) => { if (n.type.name === "ordered_list") lists.push(n.attrs.order); return true; });
    expect(lists).toEqual([0, 99999]);
    // and its static rendering is only the schema's elements
    const host = document.createElement("div");
    renderStatic(host, d, { label, jump: () => {} });
    for (const el of host.querySelectorAll("*")) {
      expect(["P", "UL", "OL", "LI", "STRONG", "EM", "BR"]).toContain(el.tagName);
      for (const a of el.attributes) expect(a.name).toBe("start");
    }
  });
  it("markdown and HTML typed or pasted as text stay text", () => {
    const e = ed();
    e.ctx(J316);
    const s = "**bold** [x](javascript:alert(1)) <b onmouseover=alert(1)>hi</b> # h";
    e.type(s);
    expect(e.state.doc.textContent).toBe(s);
    expect(e.state.doc.firstChild!.firstChild!.marks).toEqual([]);
    const host = document.createElement("div");
    renderStatic(host, e.state.doc, { label, jump: () => {} });
    expect(host.querySelector("p")!.textContent).toBe(s);
    expect(host.querySelector("b, a, [onmouseover]")).toBeNull();
  });
  it("stored JSON is the second gate: anything outside the schema is unreadable", () => {
    const ok = docJSON(schema.node("doc", null, [schema.node("paragraph", { tags: [J316] }, [schema.text("hi")])]));
    expect(parseDoc(ok)!.textContent).toBe("hi");
    for (const bad of [
      null, "", "{", "[]", '{"type":"doc","content":[{"type":"heading","content":[]}]}',
      '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:x"}}]}]}]}',
      '{"type":"doc","content":[{"type":"paragraph","attrs":{"tags":"<img src=x onerror=alert(1)>"}}]}',
      '{"type":"doc","content":[{"type":"paragraph","attrs":{"tags":[{"book":"<b>","chapter":1,"v1":1,"v2":1}]}}]}',
      '{"type":"doc","content":[{"type":"text","text":"bare"}]}',
    ]) expect(parseDoc(bad), String(bad)).toBeNull();
  });
});

describe("tag rules (8.4)", () => {
  it("merge, extend (at most 3) and union", () => {
    expect(merge(J316, J318)).toEqual({ book: "John", chapter: 3, v1: 16, v2: 18 });
    expect(merge(J316, { book: "John", chapter: 3, v1: 0, v2: 0 })).toEqual({ book: "John", chapter: 3, v1: 0, v2: 0 });
    expect(merge(J316, J41)).toBeNull();
    expect(extend([J316], J318)).toEqual([{ ...J316, v2: 18 }]);
    expect(extend([J316], J41)).toEqual([J316, J41]);
    const three = [J316, J41, { book: "Ps", chapter: 23, v1: 0, v2: 0 }];
    expect(extend(three, { book: "Jude", chapter: 1, v1: 3, v2: 3 })).toBe(three);
    expect(union([J316], [J41, J316])).toEqual([J316, J41]);
    expect(union(null, null)).toBeNull();
    expect(validTag(J316)).toBe(true);
    expect(validTag({ ...J316, v1: 0 })).toBe(false);
  });
  it("labels: short book name, en dash range, whole chapter, single-chapter book", () => {
    expect(label({ ...J316, v2: 18 })).toBe("John 3:16–18");
    expect(label({ book: "Ps", chapter: 23, v1: 0, v2: 0 })).toBe("Psalm 23");
    expect(label({ book: "Jude", chapter: 1, v1: 5, v2: 5 })).toBe("Jude 5");
  });

  it("1: the first character typed tags the block with the context; empty blocks have none", () => {
    const e = ed();
    e.ctx(J316);
    expect(e.tags(0)).toBeNull();
    e.type("H");
    expect(e.tags(0)).toEqual([J316]);
    expect(e.state.doc.textContent).toBe("H"); // never in the text (6)
    e.run((s) => s.tr.delete(1, 2));
    expect(e.tags(0)).toBeNull();
  });

  it("2: composing extends (same chapter grows, other chapter adds); 3: frozen after the caret leaves", () => {
    const e = ed();
    e.ctx(J316);
    e.type("Hello");
    e.ctx(J318);
    expect(e.tags(0)).toEqual([{ ...J316, v2: 18 }]);
    e.ctx(J41);
    expect(e.tags(0)).toEqual([{ ...J316, v2: 18 }, J41]);
    // a second paragraph; the caret leaves the first -> the first is frozen
    e.run((s) => s.tr.split(s.selection.head));
    e.ctx({ book: "Ps", chapter: 23, v1: 0, v2: 0 });
    expect(e.tags(0)).toEqual([{ ...J316, v2: 18 }, J41]);
    // fixing a typo later never retags
    e.at(2);
    e.type("x");
    expect(e.tags(0)).toEqual([{ ...J316, v2: 18 }, J41]);
  });

  it("2: 60s without typing ends composing", () => {
    const e = ed();
    e.ctx(J316);
    e.type("a");
    clock += COMPOSE_IDLE_MS + 1;
    e.ctx(J318);
    expect(e.tags(0)).toEqual([J316]);
  });

  it("4: Enter mid-block keeps the tags on both halves; a join gives the union; a tag jump ends composing", () => {
    const e = ed();
    e.ctx(J316);
    e.type("abcd");
    e.at(3);
    e.cmd((s, d) => { d!(s.tr.split(3)); return true; });
    expect(e.tags(0)).toEqual([J316]);
    expect(e.tags(1)).toEqual([J316]);
    // a third block in another passage, then join it into the second (Backspace at its start)
    e.at(e.state.doc.content.size - 1);
    e.cmd((s, d) => { d!(s.tr.split(s.doc.content.size - 1)); return true; });
    e.ctx(J41);
    e.type("z");
    expect(e.tags(2)).toEqual([J41]);
    const start = e.state.doc.child(0).nodeSize + e.state.doc.child(1).nodeSize + 1;
    e.at(start);
    expect(e.cmd(keepTags(joinBackward, -1))).toBe(true);
    expect(e.state.doc.childCount).toBe(2);
    expect(e.tags(1)).toEqual([J316, J41]);
    expect(e.state.doc.textContent).toBe("abcdz");
    // a jump made by clicking a tag ends composing: the next context change extends nothing
    e.type("y");
    e.end();
    e.ctx(J318);
    expect(e.tags(1)).toEqual([J316, J41]);
  });

  it("5: remove and add are undoable steps; a re-wrapped paragraph keeps its tags", () => {
    const e = ed();
    e.ctx(J316);
    e.type("note");
    e.end();
    e.ctx(J41);
    expect(e.cmd(tagWithContext)).toBe(true);
    expect(e.tags(0)).toEqual([J316, J41]);
    // to a bulleted list and back: the tags move with the block, never retagged
    e.ctx({ book: "Ps", chapter: 23, v1: 0, v2: 0 });
    e.end();
    e.cmd(toggleList(schema.nodes.bullet_list));
    expect(e.state.doc.child(0).type.name).toBe("bullet_list");
    expect(e.tags(0)).toEqual([J316, J41]);
    e.cmd(toggleList(schema.nodes.bullet_list));
    expect(e.state.doc.child(0).type.name).toBe("paragraph");
    expect(e.tags(0)).toEqual([J316, J41]);
  });

  it("6: typing never changes the text, and a chip row shows once per run of identical tags", () => {
    const e = ed();
    e.ctx(J316);
    const words = ["one", "two", "three"];
    words.forEach((w, i) => {
      if (i) e.run((s) => s.tr.split(s.selection.head));
      e.type(w);
    });
    expect(e.state.doc.textContent).toBe("onetwothree");
    // the 2nd and 3rd were new paragraphs typed while reading the same verse: one run
    expect(chipRuns(e.state.doc).map((r) => r.index)).toEqual([0]);
    e.end();
    e.run((s) => s.tr.split(s.selection.head));
    e.ctx(J41);
    e.type("four");
    expect(chipRuns(e.state.doc).map((r) => r.tags)).toEqual([[J316], [J41]]);
  });
});

describe("attrs from any route stay within the backend's rules (W2A-N1, N2)", () => {
  const bl = schema.nodes.bullet_list, ol = schema.nodes.ordered_list;
  it("the schema refuses bad tags and order on every node it creates", () => {
    for (const tags of ["AAAA", { x: 1 }, [{ book: "Gospel of Thomas", chapter: 1, v1: 0, v2: 0 }],
      [{ ...J316, x: 1 }], [J316, J318, J41, J316], [J316, null]]) {
      expect(() => bl.create({ tags }), JSON.stringify(tags)).toThrow(RangeError);
      expect(validTags(tags)).toBe(false);
    }
    for (const order of [1e9, -1, 1.5, "3", null, Number.NaN]) expect(() => ol.create({ order }), String(order)).toThrow(RangeError);
    expect(bl.create({ tags: [J316] }).attrs.tags).toEqual([J316]);
    expect(ol.create({ order: 99_999 }).attrs.order).toBe(99_999);
    expect(validTags(null)).toBe(true);
    // a stored day with an out-of-range order is unreadable, never mounted
    expect(parseDoc('{"type":"doc","content":[{"type":"ordered_list","attrs":{"order":1000000},"content":[{"type":"list_item","content":[{"type":"paragraph"}]}]}]}')).toBeNull();
  });
  it("asTags reads anything that isn't a list of tags as none (the tagger never throws)", () => {
    for (const v of ["AAAA", { x: 1 }, 5, undefined, null]) expect(asTags(v)).toEqual([]);
    expect(asTags([J316, { book: "<b>" }])).toEqual([J316]);
  });
  it("cleanSlice: pasted blocks carry no tags, order is clamped, lone surrogates become U+FFFD", () => {
    const item = schema.node("list_item", null, [schema.node("paragraph", { tags: [J316] }, [schema.text("a\uD83Db")])]);
    const s = cleanSlice(new Slice(Fragment.from(ol.create({ order: 42, tags: [J318] }, [item])), 0, 0));
    const n = s.content.firstChild!;
    expect(n.attrs).toEqual({ order: 42, tags: null });
    expect(n.firstChild!.firstChild!.attrs.tags).toBeNull();
    expect(n.textContent).toBe("a�b");
  });
  describe("W2A-N4: a block moved inside the editor keeps its own tags; outside paste still gets none", () => {
    const para = (text: string, tags: PassageTag[] | null) => schema.node("paragraph", { tags }, [schema.text(text)]);
    const slice = () => new Slice(Fragment.from(para("moved", [J316])), 0, 0);
    const stripped = () => new Slice(Fragment.from(para("moved", null)), 1, 1);  // what the selection 1..6 copies: open at both ends
    const plugin = () => {
      const state = notesState(schema.node("doc", null, [para("moved", [J316]), para("other", null)]), opts());
      const p = state.plugins.find((x) => x.props.transformPasted)!;
      const view = (extra: object = {}) => ({ state: state.apply(state.tr.setSelection(TextSelection.create(state.doc, 1, 6))), dragging: null, ...extra }) as never;
      return { p, view };
    };
    const tagsOf = (s: Slice) => s.content.firstChild!.attrs.tags;
    it("an internal drag (view.dragging) keeps the tags", () => {
      const { p, view } = plugin();
      const sl = slice();
      expect(tagsOf(p.props.transformPasted!.call(p, sl, view({ dragging: { slice: sl } }), false))).toEqual([J316]);
    });
    it("cut then paste inside the editor keeps the tags (the clipboard HTML carries none)", () => {
      const { p, view } = plugin();
      const v = view();
      p.props.handleDOMEvents!.cut!.call(p, v, new Event("cut") as ClipboardEvent);
      expect(tagsOf(p.props.transformPasted!.call(p, stripped(), v, false))).toEqual([J316]);
    });
    it("a paste from outside, or of other text, still gets no tags", () => {
      const { p, view } = plugin();
      expect(tagsOf(p.props.transformPasted!.call(p, slice(), view(), false))).toBeNull();
      const v = view();
      p.props.handleDOMEvents!.copy!.call(p, v, new Event("copy") as ClipboardEvent);
      const other = new Slice(Fragment.from(para("not what was copied", null)), 0, 0);
      expect(tagsOf(p.props.transformPasted!.call(p, other, v, false))).toBeNull();
    });
    it("keep still clamps order and fixes surrogates; without keep the tags go", () => {
      const item = schema.node("list_item", null, [schema.node("paragraph", null, [schema.text("a\uD83Db")])]);
      const s = cleanSlice(new Slice(Fragment.from(ol.create({ order: 42, tags: [J318] }, [item])), 0, 0), true);
      expect(s.content.firstChild!.attrs).toEqual({ order: 42, tags: [J318] });
      expect(s.content.firstChild!.textContent).toBe("a�b");
      expect(cleanSlice(s).content.firstChild!.attrs.tags).toBeNull();
    });
  });
  it("wellFormed replaces only lone surrogates; docText is the day's words, one block per line", () => {
    expect(wellFormed("😀 x\uDE00\uD83D")).toBe("😀 x��");
    expect(wellFormed("Ἐν ἀρχῇ")).toBe("Ἐν ἀρχῇ");
    const d = schema.node("doc", null, [schema.node("paragraph", null, [schema.text("one")]),
      schema.node("bullet_list", null, [schema.node("list_item", null, [schema.node("paragraph", null, [schema.text("two")])])])]);
    expect(docText(d)).toBe("one\ntwo");
  });
  it("docJSON never sends a lone surrogate (serde_json refuses them)", () => {
    const d = schema.node("doc", null, [schema.node("paragraph", null, [schema.text("abc\uD83D def")])]);
    const j = docJSON(d);
    expect(j).not.toMatch(/\\ud83d/i);
    expect(JSON.parse(j).content[0].content[0].text).toBe("abc� def");
  });
});

describe("autosave (8.6)", () => {
  function fakeTimers() {
    let now = 0, id = 0;
    const q = new Map<number, { at: number; fn: () => void }>();
    const t: Timers = { set: (fn, ms) => { q.set(++id, { at: now + ms, fn }); return id; }, clear: (i) => { q.delete(i); } };
    const advance = async (ms: number) => {
      const end = now + ms;
      for (;;) {
        const next = [...q.entries()].sort((a, b) => a[1].at - b[1].at)[0];
        if (!next || next[1].at > end) break;
        q.delete(next[0]);
        now = next[1].at;
        next[1].fn();
        await Promise.resolve(); await Promise.resolve(); await Promise.resolve();
      }
      now = end;
      for (let i = 0; i < 5; i++) await Promise.resolve();
    };
    return { t, advance };
  }

  it("saves 500ms after the last change, once, with the latest text", async () => {
    const { t, advance } = fakeTimers();
    const saves: string[] = [];
    const s = createSaver({ save: async (d, doc) => { saves.push(`${d}:${doc}`); }, timers: t });
    let text = "a";
    s.change("2026-10-27", () => text);
    await advance(300);
    text = "ab";
    s.change("2026-10-27", () => text);
    await advance(499);
    expect(saves).toEqual([]);
    await advance(1);
    expect(saves).toEqual(["2026-10-27:ab"]);
    expect(s.dirty).toBe(0);
  });

  it("broken (the editor failed, W2A-N1): Not saved from then on, flush false even after good saves", async () => {
    const { t, advance } = fakeTimers();
    const s = createSaver({ save: async () => {}, timers: t });
    s.broken("the editor hit an error (x)");
    expect(s.status).toEqual({ kind: "failed", reason: "the editor hit an error (x)" });
    s.change("2026-10-27", () => "a");
    await advance(500);
    expect(s.status.kind).toBe("failed");
    expect(await s.flush()).toBe(false);
    s.broken("a second error");
    expect(s.status).toEqual({ kind: "failed", reason: "the editor hit an error (x)" });
  });

  it("flush (blur, closing the panel or the app) saves at once", async () => {
    const { t } = fakeTimers();
    const saves: string[] = [];
    const s = createSaver({ save: async (d, doc) => { saves.push(`${d}:${doc}`); }, timers: t });
    s.change("2026-10-27", () => "x");
    s.change("2026-10-26", () => "y");
    expect(await s.flush()).toBe(true);
    expect(saves.sort()).toEqual(["2026-10-26:y", "2026-10-27:x"]);
  });

  it("a failed save says Not saved, keeps the text dirty and retries every 5s", async () => {
    const { t, advance } = fakeTimers();
    let fail = true;
    const saves: string[] = [];
    const statuses: SaveStatus["kind"][] = [];
    const s = createSaver({
      save: async (_d, doc) => { if (fail) throw "disk full"; saves.push(doc); },
      timers: t, onStatus: (st) => statuses.push(st.kind),
    });
    s.change("2026-10-27", () => "v1");
    await advance(500);
    expect(s.status).toEqual({ kind: "failed", reason: "disk full" });
    expect(s.dirty).toBe(1);
    expect(await s.flush()).toBe(false);
    await advance(4999);
    expect(saves).toEqual([]);
    fail = false;
    await advance(1);
    expect(saves).toEqual(["v1"]);
    expect(s.status.kind).toBe("saved");
    expect(statuses).toEqual(["failed", "saved"]);
  });

  it("Saving… shows only when a save takes longer than 1s", async () => {
    const { t, advance } = fakeTimers();
    let release = () => {};
    const statuses: string[] = [];
    const s = createSaver({ save: () => new Promise<void>((r) => { release = r; }), timers: t, onStatus: (st) => statuses.push(st.kind) });
    s.change("d", () => "x");
    await advance(500 + 999);
    expect(statuses).toEqual([]);
    await advance(1);
    expect(statuses).toEqual(["saving"]);
    release();
    await advance(0);
    expect(statuses).toEqual(["saving", "saved"]);
  });
});

describe("days (8.1, 8.2)", () => {
  afterEach(() => vi.useRealTimers());
  it("the local date, and a note keeps the day it was started on across midnight", () => {
    expect(localDay(new Date(2026, 9, 27, 23, 59, 59))).toBe("2026-10-27");
    expect(localDay(new Date(2026, 9, 28, 0, 0, 1))).toBe("2026-10-28");
    expect(msToMidnight(new Date(2026, 9, 27, 23, 59, 0))).toBe(60_000);
    // before midnight: Today is the note being written
    expect(dayOrder(["2026-10-25"], "2026-10-27", "2026-10-27")).toEqual(["2026-10-27", "2026-10-25"]);
    // after midnight: a new empty Today above it; the note stays on its own day
    expect(dayOrder(["2026-10-25"], "2026-10-28", "2026-10-27")).toEqual(["2026-10-28", "2026-10-27", "2026-10-25"]);
    expect(dayOrder([], "2026-10-28", null)).toEqual(["2026-10-28"]);
  });
  it("headers: Today, Yesterday, the weekday; no year for this year's older days", () => {
    expect(dayHeading("2026-10-22", "2026-10-22", "en-GB")).toEqual({ name: "Today", date: "Thursday, 22 October 2026", full: "Thursday, 22 October 2026" });
    expect(dayHeading("2026-10-21", "2026-10-22", "en-GB").name).toBe("Yesterday");
    const older = dayHeading("2026-10-20", "2026-10-22", "en-GB");
    expect(older).toEqual({ name: "Tuesday", date: "Tuesday 20 October", full: "Tuesday, 20 October 2026" });
    expect(dayHeading("2025-12-31", "2026-01-01", "en-GB")).toMatchObject({ name: "Yesterday", date: "Wednesday, 31 December 2025" });
  });
});

describe("the reading context (8.4)", () => {
  const base = { book: "John", chapter: 3, lastVerse: 36 };
  it("the current verse when it is on screen, else the visible verses, else the whole chapter", () => {
    expect(readingContext({ ...base, current: 16, visible: [14, 15, 16, 17] })).toEqual(J316);
    expect(readingContext({ ...base, current: 2, visible: [14, 15, 16] })).toEqual({ book: "John", chapter: 3, v1: 14, v2: 16 });
    expect(readingContext({ ...base, current: null, visible: Array.from({ length: 36 }, (_, i) => i + 1) })).toEqual({ book: "John", chapter: 3, v1: 0, v2: 0 });
    expect(readingContext({ ...base, current: null, visible: [] })).toBeNull();
  });
  it("a verse counts when at least half of it is visible", () => {
    const rects = [{ v: 1, top: -50, bottom: 10 }, { v: 2, top: 10, bottom: 50 }, { v: 3, top: 80, bottom: 120 }];
    expect(halfVisible(rects, 0, 100)).toEqual([2, 3]);
  });
});

describe("smooth caret geometry (8.5)", () => {
  it("centred on the boundary, vertically centred on the line, rounded to device pixels", () => {
    const g = caretGeometry({ left: 110.3, top: 200, bottom: 224 }, { left: 100, top: 150 }, 17.25, 1.5);
    expect(g.x).toBeCloseTo(Math.round(9.3 * 1.5) / 1.5);
    expect(g.y).toBeCloseTo(Math.round((212 - 150 - 8.625) * 1.5) / 1.5);
  });
  it("snaps on a pointer move, focus, IME, the first draw and more than 3 lines; glides otherwise", () => {
    const b = { forced: false, dy: 24, lineHeight: 24, composing: false, first: false };
    expect(snaps(b)).toBe(false);
    expect(snaps({ ...b, dy: 72 })).toBe(false);
    expect(snaps({ ...b, dy: 73 })).toBe(true);
    expect(snaps({ ...b, forced: true })).toBe(true);
    expect(snaps({ ...b, composing: true })).toBe(true);
    expect(snaps({ ...b, first: true })).toBe(true);
  });
});
