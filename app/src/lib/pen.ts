// Pen-mode range highlights, the pure part (highlights-pen.md 4.3, 6, 7.5).
// Offsets are Unicode code points of the stored verse text (the unit the core
// stores and checks; W2P-B3: never UTF-16 indices). Nothing here touches the
// DOM or changes a character: it only measures text and rectangles.
import type { HighlightColor } from "./types";

/** One piece of a stroke or a mark: `[start, end)` code points of one verse. */
export interface Piece { verse: number; start: number; end: number }
export interface PieceOut extends Piece { stale: boolean }
export interface MarkOut { id: number; color: HighlightColor; created_at: number; pieces: PieceOut[] }
/** A mark as an undo snapshot carries it (the core's `MarkIn`). */
export interface MarkIn { color: HighlightColor; created_at: number; pieces: Piece[] }

/** A point in the chapter: verse number and code-point offset (a caret position). */
export interface Spot { verse: number; offset: number }

export const cps = (s: string): string[] => Array.from(s);
export const cpLen = (s: string): number => cps(s).length;

/** UTF-16 index of code point `cp` in `s`. */
export function cpToU16(s: string, cp: number): number {
  let u = 0, n = 0;
  for (const ch of s) { if (n === cp) return u; u += ch.length; n++; }
  return u;
}
/** Code points before UTF-16 index `u` (a surrogate half counts as its pair's start). */
export function u16ToCp(s: string, u: number): number {
  let i = 0, n = 0;
  for (const ch of s) { if (i + ch.length > u) return n; i += ch.length; n++; }
  return n;
}

/** Unicode White_Space, exactly the core's `char::is_whitespace`. */
const WS = new Set([0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x20, 0x85, 0xa0, 0x1680, 0x2000, 0x2001, 0x2002, 0x2003, 0x2004,
  0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200a, 0x2028, 0x2029, 0x202f, 0x205f, 0x3000]);
export const isWs = (ch: string | undefined): boolean => ch !== undefined && WS.has(ch.codePointAt(0)!);

const MARK = /^\p{M}$/u;
const ZWJ = "\u{200D}";
/** B1 (as the core checks it): never before a combining mark, never next to a ZWJ. */
export function b1Ok(chars: string[], b: number): boolean {
  if (b <= 0 || b >= chars.length) return true;
  return !(MARK.test(chars[b]) || chars[b] === ZWJ || chars[b - 1] === ZWJ);
}

interface Seg { start: number; end: number; word: boolean }
const segCache = new Map<string, { g: number[]; w: Seg[] }>();
function segments(text: string, lang: string): { g: number[]; w: Seg[] } {
  const key = `${lang}\u0000${text}`;
  const hit = segCache.get(key);
  if (hit) return hit;
  const toCp = (segs: Iterable<{ index: number; segment: string; isWordLike?: boolean }>) => {
    const out: Seg[] = [];
    let cp = 0;
    for (const s of segs) { const n = cpLen(s.segment); out.push({ start: cp, end: cp + n, word: !!s.isWordLike }); cp += n; }
    return out;
  };
  const g = toCp(new Intl.Segmenter(lang, { granularity: "grapheme" }).segment(text)).map((s) => s.start);
  g.push(cpLen(text));
  const w = toCp(new Intl.Segmenter(lang, { granularity: "word" }).segment(text));
  const v = { g, w };
  if (segCache.size > 400) segCache.clear();
  segCache.set(key, v);
  return v;
}

/** Grapheme-cluster boundaries of `text` in code points, 0 and the length included. */
export function graphemeBounds(text: string, lang = "en"): number[] { return segments(text, lang).g; }

/** Round `b` down (or up) to a grapheme boundary that also passes B1. */
export function snapBound(text: string, lang: string, b: number, dir: "down" | "up"): number {
  const g = graphemeBounds(text, lang);
  const chars = cps(text);
  let i = dir === "down" ? findLast(g, (x) => x <= b) : g.findIndex((x) => x >= b);
  if (i < 0) i = dir === "down" ? 0 : g.length - 1;
  while (i > 0 && i < g.length - 1 && !b1Ok(chars, g[i])) i += dir === "down" ? -1 : 1;
  return g[i];
}
function findLast<T>(a: T[], f: (t: T) => boolean): number {
  for (let i = a.length - 1; i >= 0; i--) if (f(a[i])) return i;
  return -1;
}

/** The outer ends never start or end on whitespace (4.3 step 7). */
export function trimWs(text: string, s: number, e: number): [number, number] | null {
  const c = cps(text);
  while (s < e && isWs(c[s])) s++;
  while (e > s && isWs(c[e - 1])) e--;
  return s < e ? [s, e] : null;
}

/** 4.3 step 6: snap a start (down to its word's start; from a space, up to the next word). */
export function wordStart(text: string, lang: string, s: number): number {
  const segs = segments(text, lang).w;
  const seg = segs.find((x) => x.start <= s && s < x.end);
  if (!seg) return s;
  if (seg.word) return seg.start;
  if (isAllWs(text, seg)) return segs.find((x) => x.word && x.start >= seg.end)?.start ?? cpLen(text);
  return s; // punctuation the pointer passed over is kept
}
/** 4.3 step 6: snap an end (up to the end of the word holding the character before it). */
export function wordEnd(text: string, lang: string, e: number): number {
  if (e <= 0) return 0;
  const segs = segments(text, lang).w;
  const seg = segs.find((x) => x.start <= e - 1 && e - 1 < x.end);
  if (!seg) return e;
  if (seg.word) return seg.end;
  if (isAllWs(text, seg)) return findLastSeg(segs, (x) => x.word && x.end <= seg.start)?.end ?? 0;
  return e;
}
function findLastSeg(a: Seg[], f: (s: Seg) => boolean): Seg | undefined {
  const i = findLast(a, f);
  return i < 0 ? undefined : a[i];
}
function isAllWs(text: string, seg: Seg): boolean {
  const c = cps(text);
  for (let i = seg.start; i < seg.end; i++) if (!isWs(c[i])) return false;
  return true;
}

/** Snap one end of a range: words (default) or letters (Alt), then whole clusters and B1. */
export function snapStart(text: string, lang: string, s: number, words: boolean): number {
  return snapBound(text, lang, words ? wordStart(text, lang, s) : s, "down");
}
export function snapEnd(text: string, lang: string, e: number, words: boolean): number {
  return snapBound(text, lang, words ? wordEnd(text, lang, e) : e, "up");
}

/** The word (or, `words` false, the letter) at code point `at` for a click; null on whitespace. */
export function clickRange(text: string, lang: string, at: number, words: boolean): [number, number] | null {
  const len = cpLen(text);
  if (at < 0 || at >= len || isWs(cps(text)[at])) return null;
  if (words) {
    const seg = segments(text, lang).w.find((x) => x.start <= at && at < x.end);
    if (!seg || !seg.word) return null; // punctuation alone isn't a word
    return [snapBound(text, lang, seg.start, "down"), snapBound(text, lang, seg.end, "up")];
  }
  const a = snapBound(text, lang, at, "down");
  const b = snapBound(text, lang, at + 1, "up");
  return a < b ? [a, b] : null;
}

/** The verses of the chapter, in order, with their stored text. */
export interface VerseInfo { v: number; text: string }

/**
 * A stroke from `a` to `b` (either order) as per-verse pieces (4.3 "Pieces"):
 * the first verse from the snapped start to its end, whole verses between,
 * the last from 0 to the snapped end. Outer ends trimmed of whitespace; empty
 * pieces dropped. Returns [] for a no-op.
 */
export function strokePieces(verses: VerseInfo[], a: Spot, b: Spot, lang: string, words: boolean): Piece[] {
  const idx = new Map(verses.map((x, i) => [x.v, i]));
  const ia = idx.get(a.verse), ib = idx.get(b.verse);
  if (ia === undefined || ib === undefined) return [];
  let [s, e, is, ie] = [a.offset, b.offset, ia, ib];
  if (ia > ib || (ia === ib && a.offset > b.offset)) [s, e, is, ie] = [b.offset, a.offset, ib, ia];
  const first = verses[is], last = verses[ie];
  s = snapStart(first.text, lang, Math.max(0, Math.min(s, cpLen(first.text))), words);
  e = snapEnd(last.text, lang, Math.max(0, Math.min(e, cpLen(last.text))), words);
  const out: Piece[] = [];
  for (let i = is; i <= ie; i++) {
    const t = verses[i].text;
    const len = cpLen(t);
    let ps = i === is ? s : 0, pe = i === ie ? e : len;
    if (i === is) { const tr = trimWs(t, ps, len); if (!tr) continue; ps = tr[0]; }
    if (i === ie) { const tr = trimWs(t, 0, pe); if (!tr) continue; pe = tr[1]; }
    if (ps < pe && trimWs(t, ps, pe)) out.push({ verse: verses[i].v, start: ps, end: pe });
  }
  // a stroke is contiguous: if trimming dropped a first or last piece, the
  // remaining ones still run verse to verse; an inner gap can't happen
  return out;
}

/** The fresh (drawable) pieces of every mark, with the mark's colour and id. */
export interface Drawn extends Piece { id: number; color: HighlightColor }
export function drawnPieces(marks: MarkOut[]): Drawn[] {
  const out: Drawn[] = [];
  for (const m of marks) for (const p of m.pieces) if (!p.stale) out.push({ verse: p.verse, start: p.start, end: p.end, id: m.id, color: m.color });
  out.sort((x, y) => x.verse - y.verse || x.start - y.start);
  return out;
}

/** The drawn piece holding code point `at` of `verse`, if any. */
export function markAt(marks: MarkOut[], verse: number, at: number): Drawn | null {
  return drawnPieces(marks).find((p) => p.verse === verse && p.start <= at && at < p.end) ?? null;
}

/** An undo snapshot of the drawn marks (stale pieces are never part of one). */
export function snapshot(marks: MarkOut[]): MarkIn[] {
  return marks
    .map((m) => ({ color: m.color, created_at: m.created_at, pieces: m.pieces.filter((p) => !p.stale).map(({ verse, start, end }) => ({ verse, start, end })) }))
    .filter((m) => m.pieces.length > 0);
}

/** Split drawn pieces by an eraser preview: the parts under `erase` get `erasing`. */
export function splitForErase(pieces: Drawn[], erase: Piece[]): (Drawn & { erasing?: boolean })[] {
  const out: (Drawn & { erasing?: boolean })[] = [];
  for (const p of pieces) {
    const cuts = erase.filter((x) => x.verse === p.verse && x.start < p.end && p.start < x.end)
      .map((x) => [Math.max(p.start, x.start), Math.min(p.end, x.end)] as const)
      .sort((x, y) => x[0] - y[0]);
    let at = p.start;
    for (const [a, b] of cuts) {
      if (a > at) out.push({ ...p, start: at, end: a });
      out.push({ ...p, start: Math.max(a, at), end: b, erasing: true });
      at = Math.max(at, b);
    }
    if (at < p.end) out.push({ ...p, start: at, end: p.end });
  }
  return out;
}

// ---- measuring (7.5) ----

export interface Rect { left: number; top: number; right: number; bottom: number }

/**
 * Steps 1 to 3 of 7.5 for one piece's glyph rects (already relative to the
 * paint layer's origin): group by line (vertical overlap > half the smaller
 * height), join touching or overlapping rects on a line (gap <= 0.5px), give
 * every box its line's top and bottom, padded by `padBlock`. Returns boxes in
 * line order.
 */
export function lineBoxes(rects: Rect[], padBlock: number): Rect[] {
  const rs = rects.filter((r) => r.right - r.left > 0 && r.bottom - r.top > 0);
  const lines: Rect[][] = [];
  for (const r of rs) {
    const line = lines.find((l) => l.some((x) => {
      const ov = Math.min(x.bottom, r.bottom) - Math.max(x.top, r.top);
      return ov > Math.min(x.bottom - x.top, r.bottom - r.top) / 2;
    }));
    if (line) line.push(r); else lines.push([r]);
  }
  const out: Rect[] = [];
  lines.sort((a, b) => a[0].top - b[0].top);
  for (const l of lines) {
    const top = Math.min(...l.map((r) => r.top)) - padBlock;
    const bottom = Math.max(...l.map((r) => r.bottom)) + padBlock;
    l.sort((a, b) => a.left - b.left);
    const joined: Rect[] = [];
    for (const r of l) {
      const last = joined[joined.length - 1];
      if (last && r.left - last.right <= 0.5) last.right = Math.max(last.right, r.right);
      else joined.push({ left: r.left, right: r.right, top, bottom });
    }
    out.push(...joined);
  }
  return out;
}

/**
 * Step 4: the mark's outer ends grow by `padInline` only next to whitespace or
 * the verse edge, and pull back by half the join gap at a different colour.
 * `startEdge` / `endEdge` say what is beside the piece's first / last
 * character. In right-to-left text the start is the right-hand side.
 */
export type Edge = "pad" | "join" | "none";
export function padEnds(boxes: Rect[], startEdge: Edge, endEdge: Edge, padInline: number, joinGap: number, rtl = false): Rect[] {
  if (!boxes.length) return boxes;
  const out = boxes.map((b) => ({ ...b }));
  const amount = (e: Edge) => (e === "pad" ? padInline : e === "join" ? -joinGap / 2 : 0);
  const first = out[0], last = out[out.length - 1];
  if (rtl) { first.right += amount(startEdge); last.left -= amount(endEdge); }
  else { first.left -= amount(startEdge); last.right += amount(endEdge); }
  return out;
}

/** What sits beside a piece: whitespace or the verse edge (pad), a different-colour piece (join), or a letter. */
export function edgesOf(text: string, p: Drawn, all: Drawn[]): [Edge, Edge] {
  const c = cps(text);
  const touch = (at: number, before: boolean) =>
    all.some((q) => q !== p && q.verse === p.verse && q.color !== p.color && (before ? q.end === at : q.start === at));
  const side = (at: number, before: boolean): Edge => {
    if (touch(at, before)) return "join";
    const ch = before ? c[at - 1] : c[at];
    return ch === undefined || isWs(ch) ? "pad" : "none";
  };
  return [side(p.start, true), side(p.end, false)];
}

export const PEN_KEYS: Record<string, HighlightColor> = { 1: "gold", 2: "green", 3: "blue", 4: "rose", 5: "violet" };
