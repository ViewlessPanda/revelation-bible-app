// Commentary panel helpers (docs/design/wave-2-layout.md 7; pipeline/DESIGN.md
// 17.5). Pure functions over the IPC payload of `get_commentary`
// (src-tauri/src/commentary.rs); the UI types none of the commentary's words.
import type { Book } from "./types";

export interface CmtAttr {
  name: string;
  value: string;
  /** reference@osisRef only: verse ids of the range ends, as written */
  ref_start?: number;
  ref_end?: number;
}
export interface CmtNode {
  n: number;
  parent: number | null;
  /** element name, or null for a text node */
  name: string | null;
  /** stored text exactly, or null for an element */
  text: string | null;
  attrs: CmtAttr[];
}
export interface CmtBlock { id: number; nodes: CmtNode[] }
export interface CmtGroup {
  /** first and last verse ids the anchor lists; null for the chapter fallback */
  first: number | null;
  last: number | null;
  blocks: CmtBlock[];
}
export interface CmtSource {
  title: string;
  /** label.status is exactly approved_by_owner */
  approved: boolean;
  author_line: string | null;
  date_line: string | null;
}
export type CommentaryResult =
  | { status: "ok"; source: CmtSource; verse_id: number; whole_chapter: boolean; groups: CmtGroup[] }
  | { status: "unavailable"; reason: string };

/** What the panel is showing. */
export type CmtView =
  | { kind: "empty" }
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "done"; result: CommentaryResult };

export interface VerseRef { book: number; chapter: number; verse: number }

/** `book*1000000 + chapter*1000 + verse` -> its parts. */
export function verseParts(id: number): VerseRef {
  return { book: Math.floor(id / 1_000_000), chapter: Math.floor(id / 1000) % 1000, verse: id % 1000 };
}

/** "John 3:16" (book name from the books list, as in the reader). */
export function verseLabel(id: number, books: Book[]): string {
  const p = verseParts(id);
  const b = books.find((x) => x.ord === p.book);
  return `${b?.name ?? `Book ${p.book}`} ${p.chapter}:${p.verse}`;
}

/** Scope line of one group (wave-2-layout.md 7): "On John 3:16", "On John 3:1–8",
 *  or for the chapter fallback "On John 3 (the whole chapter)". UI text, never
 *  from the commentary. */
export function scopeLabel(g: CmtGroup, verseId: number, books: Book[]): string {
  if (g.first === null || g.last === null) {
    const p = verseParts(verseId);
    const b = books.find((x) => x.ord === p.book);
    return `On ${b?.name ?? `Book ${p.book}`} ${p.chapter} (the whole chapter)`;
  }
  if (g.first === g.last) return `On ${verseLabel(g.first, books)}`;
  const a = verseParts(g.first), z = verseParts(g.last);
  if (a.book === z.book && a.chapter === z.chapter) return `On ${verseLabel(g.first, books)}–${z.verse}`;
  return `On ${verseLabel(g.first, books)}–${verseLabel(g.last, books)}`;
}

/** Where a reference link goes: its range start, as written (a descending
 *  range is never expanded, DESIGN.md 17.5). Null if it carries no ids. */
export function refTarget(n: CmtNode): VerseRef | null {
  if (n.name !== "reference") return null;
  const a = n.attrs.find((x) => x.name === "osisRef" && typeof x.ref_start === "number");
  return a && a.ref_start !== undefined ? verseParts(a.ref_start) : null;
}

/** Children of an element, in document order. */
export function childrenOf(block: CmtBlock, el: CmtNode): CmtNode[] {
  return block.nodes.filter((c) => c.parent === el.n);
}

export function attr(n: CmtNode, name: string): string | null {
  return n.attrs.find((a) => a.name === name)?.value ?? null;
}

/** Identity of the content shown: the same blocks keep their scroll and don't fade. */
export function contentKey(r: CommentaryResult): string {
  if (r.status !== "ok") return `x:${r.reason}`;
  return r.groups.map((g) => g.blocks.map((b) => b.id).join(",")).join("|");
}
