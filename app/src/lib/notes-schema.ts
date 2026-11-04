// Daily notes (docs/design/wave-2-layout.md 8): the ProseMirror schema and
// the passage tags' pure rules.
//
// The schema is the wave-2 formatting set only (8.3, design README Q19):
// paragraphs, bulleted and numbered lists, bold, italic and hard breaks.
// Anything else is dropped on paste, because ProseMirror parses pasted HTML
// against this schema only: no links (so no `javascript:` URL can ever be a
// link), no images, no raw HTML, no headings or quotes this wave. The
// backend checks the same shape again on every save (src-tauri/src/notes.rs).
//
// Passage tags (8.4) are a node attribute of top-level blocks, `tags`, never
// text: `null` = never tagged (an empty block, or nothing being read), `[]`
// = the reader removed them, else 1 to 3 `{book, chapter, v1, v2}` (OSIS
// book ID; `v1 = v2 = 0` is the whole chapter). Display names come from the
// installed books, never from the note.
import { Schema, type DOMOutputSpec } from "prosemirror-model";
import type { Book } from "./types";

export interface PassageTag {
  book: string;
  chapter: number;
  v1: number;
  v2: number;
}
export type Tags = PassageTag[] | null;

/** At most this many tags on one block (8.4 rule 2). */
export const MAX_TAGS = 3;

/** At most this `order` on a numbered list (the backend's limit, notes.rs). */
export const MAX_ORDER = 99_999;

/** `tags` is null or 0 to 3 well-formed tags (W2A-N1: ProseMirror checks it on every node it creates). */
export function validTags(v: unknown): v is Tags {
  return v === null || (Array.isArray(v) && v.length <= MAX_TAGS && v.every(validTag));
}
/** `order` is a whole number 0 to 99,999. */
export const validOrder = (v: unknown): v is number => Number.isInteger(v) && (v as number) >= 0 && (v as number) <= MAX_ORDER;

// W2A-N1/N2: every node made with these attrs is checked, so a pasted
// `data-pm-slice` context (or any other route) can't put a value in the
// document that the tagger can't draw or the backend would refuse.
const tagsAttr = {
  tags: { default: null as Tags, validate: (v: unknown) => { if (!validTags(v)) throw new RangeError("tags: not passage tags"); } },
};
const orderAttr = {
  order: { default: 1, validate: (v: unknown) => { if (!validOrder(v)) throw new RangeError("order: out of range"); } },
};
const P: DOMOutputSpec = ["p", 0];

/** `<ol start>` from pasted HTML, kept within what the backend accepts. */
function olStart(dom: HTMLElement): number {
  const n = Number.parseInt(dom.getAttribute("start") ?? "1", 10);
  return Number.isFinite(n) ? Math.max(0, Math.min(MAX_ORDER, n)) : 1;
}

export const schema = new Schema({
  nodes: {
    doc: { content: "block+" },
    paragraph: {
      group: "block",
      content: "inline*",
      attrs: tagsAttr,
      parseDOM: [{ tag: "p" }],
      toDOM: () => P,
    },
    bullet_list: {
      group: "block",
      content: "list_item+",
      attrs: tagsAttr,
      parseDOM: [{ tag: "ul" }],
      toDOM: () => ["ul", 0],
    },
    ordered_list: {
      group: "block",
      content: "list_item+",
      attrs: { ...orderAttr, ...tagsAttr },
      parseDOM: [{ tag: "ol", getAttrs: (dom) => ({ order: olStart(dom as HTMLElement) }) }],
      toDOM: (node) => (node.attrs.order === 1 ? ["ol", 0] : ["ol", { start: String(node.attrs.order) }, 0]),
    },
    list_item: {
      content: "paragraph block*",
      defining: true,
      parseDOM: [{ tag: "li" }],
      toDOM: () => ["li", 0],
    },
    text: { group: "inline" },
    hard_break: {
      inline: true,
      group: "inline",
      selectable: false,
      parseDOM: [{ tag: "br" }],
      toDOM: () => ["br"],
    },
  },
  marks: {
    strong: {
      parseDOM: [
        { tag: "strong" },
        { tag: "b", getAttrs: (dom) => ((dom as HTMLElement).style.fontWeight !== "normal" ? null : false) },
        { style: "font-weight=400", clearMark: (m) => m.type.name === "strong" },
        { style: "font-weight", getAttrs: (v) => (/^(bold(er)?|[5-9]\d{2,})$/.test(v as string) ? null : false) },
      ],
      toDOM: () => ["strong", 0],
    },
    em: {
      parseDOM: [{ tag: "i" }, { tag: "em" }, { style: "font-style=italic" }, { style: "font-style=normal", clearMark: (m) => m.type.name === "em" }],
      toDOM: () => ["em", 0],
    },
  },
});

/** An empty day: one empty paragraph. */
export const emptyDoc = () => schema.node("doc", null, [schema.node("paragraph")]);

// ---- tags: pure rules (8.4) ----------------------------------------------

/** A block's tags as a list; anything that isn't a list counts as none (never throws, W2A-N1). */
export const asTags = (v: unknown): PassageTag[] => (Array.isArray(v) ? v.filter(validTag) : []);

export const whole = (t: PassageTag) => t.v1 === 0;
export const sameTag = (a: PassageTag, b: PassageTag) =>
  a.book === b.book && a.chapter === b.chapter && a.v1 === b.v1 && a.v2 === b.v2;
/** Two blocks share a chip row when their tags are identical, in order. */
export function sameTags(a: Tags, b: Tags): boolean {
  const x = asTags(a), y = asTags(b);
  return x.length === y.length && x.every((t, i) => sameTag(t, y[i]));
}

/** One tag covering both, when they are in the same chapter (rule 2: "the range grows"). */
export function merge(a: PassageTag, b: PassageTag): PassageTag | null {
  if (a.book !== b.book || a.chapter !== b.chapter) return null;
  if (whole(a) || whole(b)) return { book: a.book, chapter: a.chapter, v1: 0, v2: 0 };
  return { book: a.book, chapter: a.chapter, v1: Math.min(a.v1, b.v1), v2: Math.max(a.v2, b.v2) };
}

/** While composing, the reading context changed (rule 2): grow the same chapter's range, or add a tag (at most 3). */
export function extend(tags: PassageTag[], ctx: PassageTag): PassageTag[] {
  const i = tags.findIndex((t) => t.book === ctx.book && t.chapter === ctx.chapter);
  if (i >= 0) {
    const m = merge(tags[i], ctx)!;
    if (sameTag(m, tags[i])) return tags;
    return tags.map((t, j) => (j === i ? m : t));
  }
  return tags.length >= MAX_TAGS ? tags : [...tags, ctx];
}

/** Joining two blocks (rule 4): the union, first block's tags first, at most 3. */
export function union(a: Tags, b: Tags): Tags {
  if (a === null && b === null) return null;
  const out: PassageTag[] = [...asTags(a)];
  for (const t of asTags(b)) if (out.length < MAX_TAGS && !out.some((o) => sameTag(o, t))) out.push(t);
  return out.slice(0, MAX_TAGS);
}

const TAG_KEYS = ["book", "chapter", "v1", "v2"];
/** A well-formed tag (the backend checks the same, notes.rs `check_tags`: no other keys). */
export function validTag(t: unknown): t is PassageTag {
  if (!t || typeof t !== "object" || Array.isArray(t)) return false;
  const o = t as Record<string, unknown>;
  if (!Object.keys(o).every((k) => TAG_KEYS.includes(k))) return false;
  const int = (v: unknown, lo: number, hi: number) => Number.isInteger(v) && (v as number) >= lo && (v as number) <= hi;
  return typeof o.book === "string" && /^[A-Za-z0-9]{1,16}$/.test(o.book) && int(o.chapter, 1, 999)
    && int(o.v1, 0, 999) && int(o.v2, 0, 999)
    && ((o.v1 === 0 && o.v2 === 0) || ((o.v1 as number) >= 1 && (o.v2 as number) >= (o.v1 as number)));
}

/** "John 3:16", "John 3:16–18", "John 3", "Jude 5", "Psalm 23"; null if the book isn't installed. */
export function tagLabel(t: PassageTag, books: Book[]): string | null {
  const b = books.find((x) => x.osis === t.book);
  if (!b) return null;
  const range = t.v2 > t.v1 ? `${t.v1}–${t.v2}` : `${t.v1}`;
  if (b.chapters === 1) return whole(t) ? b.name : `${b.name} ${range}`;
  return whole(t) ? `${b.name} ${t.chapter}` : `${b.name} ${t.chapter}:${range}`;
}

/** A label that never fails: the stored book ID when the book isn't installed. */
export function tagText(t: PassageTag, books: Book[]): string {
  return tagLabel(t, books) ?? (whole(t) ? `${t.book} ${t.chapter}` : `${t.book} ${t.chapter}:${t.v1}${t.v2 > t.v1 ? `–${t.v2}` : ""}`);
}
