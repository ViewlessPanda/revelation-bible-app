// The pen's pointer handling (highlights-pen.md 4.1-4.3): press, drag, click
// and release on the reading pane while pen mode is on. It only reads the
// verse-text DOM (through lib/ink.ts) and reports pieces to its host; the
// host writes them through the core and repaints. Nothing here touches a
// `.vtext` or a character.
import { charAt, spotAt } from "./ink";
import { clickRange, markAt, splitForErase, strokePieces, type Drawn, type MarkOut, type Piece, type Spot, type VerseInfo } from "./pen";
import type { HighlightColor } from "./types";

export type Tool = "pen" | "eraser";
export interface Preview { tool: Tool; color: HighlightColor; pieces: Piece[] }

export interface PenHost {
  scroller: HTMLElement;
  chapterEl: HTMLElement;
  /** the pen tool, or null when pen mode is off */
  tool: () => Tool | null;
  color: () => HighlightColor;
  verses: () => VerseInfo[];
  lang: () => string;
  marks: () => MarkOut[];
  /** the live preview while dragging (null: none) */
  preview: (p: Preview | null) => void;
  /** a finished drag: one stroke of pieces (never empty) */
  stroke: (tool: Tool, pieces: Piece[]) => void;
  /** a click on a drawn mark */
  clickMark: (tool: Tool, mark: Drawn) => void;
  /** a pen click on plain text: the word (or letter) under it */
  clickText: (piece: Piece) => void;
  /** a click on a verse number */
  clickVerseNumber: (tool: Tool, verse: number) => void;
  /** the drag reached past the chapter's end (4.1: clamped, Q31) */
  clamped: () => void;
}

const MOVE = 4; // px: below this a press is a click (4.1)
const BAND = 48; // px: the auto-scroll band at the pane's top and bottom
const MAX_SCROLL = 16; // px per frame

interface Drag {
  id: number; x0: number; y0: number; x: number; y: number;
  anchor: Spot | null; vnum: number | null; alt: boolean; moved: boolean; tool: Tool;
}

export interface PenControl {
  /** Esc during a stroke: drop it (nothing is written). True if there was one. */
  cancel: () => boolean;
  /** a stroke is in progress */
  active: () => boolean;
  destroy: () => void;
}

export function attachPen(h: PenHost): PenControl {
  const el = h.scroller;
  const doc = el.ownerDocument;
  const win = doc.defaultView ?? window;
  let d: Drag | null = null;
  let raf = 0;

  const column = () => h.chapterEl.closest<HTMLElement>(".reading") ?? h.chapterEl;
  const pieces = (x: number, y: number): Piece[] => {
    if (!d?.anchor) return [];
    const focus = spotAt(h.chapterEl, column(), x, y, false);
    if (!focus) return [];
    return strokePieces(h.verses(), d.anchor, focus, h.lang(), !d.alt);
  };
  const pastEnd = (x: number, y: number) => {
    const hit = doc.elementFromPoint?.(x, y) ?? null;
    if (hit?.closest(".chapter-end")) return true;
    const verses = h.chapterEl.querySelectorAll<HTMLElement>(".verse");
    const last = verses[verses.length - 1];
    const b = last?.getBoundingClientRect();
    return !!b && b.height > 0 && y > b.bottom;
  };
  const update = () => {
    if (!d || !d.moved) return;
    if (pastEnd(d.x, d.y)) h.clamped();
    h.preview({ tool: d.tool, color: h.color(), pieces: pieces(d.x, d.y) });
  };
  const scrollStep = () => {
    raf = 0;
    if (!d || !d.moved) return;
    const r = el.getBoundingClientRect();
    let depth = 0;
    if (d.y < r.top + BAND) depth = d.y - (r.top + BAND);
    else if (d.y > r.bottom - BAND) depth = d.y - (r.bottom - BAND);
    if (depth === 0 || r.height <= 2 * BAND) return;
    const v = Math.max(-MAX_SCROLL, Math.min(MAX_SCROLL, (depth / BAND) * MAX_SCROLL));
    const before = el.scrollTop;
    el.scrollTop += v;
    if (el.scrollTop !== before) update();
    raf = win.requestAnimationFrame(scrollStep);
  };
  const end = () => {
    if (raf) win.cancelAnimationFrame(raf);
    raf = 0;
    const was = d;
    d = null;
    if (was) {
      try { if (el.hasPointerCapture?.(was.id)) el.releasePointerCapture(was.id); } catch { /* already released */ }
    }
    return was;
  };

  function down(e: PointerEvent) {
    const tool = h.tool();
    if (!tool || e.button !== 0 || d) return;
    const t = e.target as Element | null;
    if (!t || !h.chapterEl.contains(t)) return;
    // the heading and the end-of-chapter buttons keep their own behaviour (3.3, 4.3.3)
    if (t.closest(".chapter-head, .book-title, .chapter-end")) return;
    const verse = t.closest<HTMLElement>(".verse");
    const vnum = verse && t.closest(".vnum") ? Number(verse.dataset.v) : null;
    const anchor = spotAt(h.chapterEl, column(), e.clientX, e.clientY, true);
    if (!anchor && vnum === null) return;
    e.preventDefault(); // no native selection or focus move (4.1)
    if (!el.contains(doc.activeElement)) el.focus({ preventScroll: true });
    try { el.setPointerCapture(e.pointerId); } catch { /* synthetic pointer */ }
    d = { id: e.pointerId, x0: e.clientX, y0: e.clientY, x: e.clientX, y: e.clientY, anchor, vnum, alt: e.altKey, moved: false, tool };
  }
  function move(e: PointerEvent) {
    if (!d || e.pointerId !== d.id) return;
    d.x = e.clientX; d.y = e.clientY; d.alt = e.altKey;
    if (!d.moved && Math.hypot(d.x - d.x0, d.y - d.y0) < MOVE) return;
    if (!d.anchor) return; // a press on a verse number with no text spot can only click
    d.moved = true;
    update();
    if (!raf) raf = win.requestAnimationFrame(scrollStep);
  }
  function up(e: PointerEvent) {
    if (!d || e.pointerId !== d.id) return;
    d.x = e.clientX; d.y = e.clientY; d.alt = e.altKey;
    const moved = d.moved;
    const list = moved ? pieces(d.x, d.y) : [];
    const was = end()!;
    if (moved) {
      h.preview(null);
      if (list.length) h.stroke(was.tool, list);
      return;
    }
    if (was.vnum !== null) { h.clickVerseNumber(was.tool, was.vnum); return; }
    const at = charAt(h.chapterEl, column(), was.x, was.y);
    if (!at) return;
    const m = markAt(h.marks(), at.verse, at.offset);
    if (m) { h.clickMark(was.tool, m); return; }
    if (was.tool !== "pen") return;
    const v = h.verses().find((x) => x.v === at.verse);
    const r = v && clickRange(v.text, h.lang(), at.offset, !was.alt);
    if (r) h.clickText({ verse: at.verse, start: r[0], end: r[1] });
  }
  // E9: the capture lost without a pointerup (window lost focus, etc.): cancel
  function lost(e: PointerEvent) {
    if (d && e.pointerId === d.id) cancel();
  }
  function cancel(): boolean {
    const was = end();
    if (!was) return false;
    if (was.moved) h.preview(null);
    return true;
  }

  el.addEventListener("pointerdown", down);
  el.addEventListener("pointermove", move);
  el.addEventListener("pointerup", up);
  el.addEventListener("pointercancel", lost);
  el.addEventListener("lostpointercapture", lost);
  return {
    cancel,
    active: () => d !== null,
    destroy() {
      cancel();
      el.removeEventListener("pointerdown", down);
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      el.removeEventListener("pointercancel", lost);
      el.removeEventListener("lostpointercapture", lost);
    },
  };
}

/**
 * What the paint layer draws: the drawn pieces, with the live preview laid
 * over them (pen: last write wins per character, so the preview cuts what is
 * under it; eraser: the part under the stroke shows at the erase opacity).
 */
export function inkPieces(drawn: Drawn[], p: Preview | null): (Drawn & { erasing?: boolean })[] {
  if (!p || !p.pieces.length) return drawn;
  const split = splitForErase(drawn, p.pieces);
  if (p.tool === "eraser") return split;
  const live: Drawn[] = p.pieces.map((x) => ({ ...x, id: -1, color: p.color }));
  return [...split.filter((x) => !x.erasing), ...live].sort((x, y) => x.verse - y.verse || x.start - y.start);
}

// ---- the pen and eraser cursors (3.3): 24 x 24, hotspots (2, 22) and (5, 19) ----
// Each shape twice: a halo stroke, then fill + outline. The halo and outline
// come from the tokens (--cursor-halo, --cursor-ink); the nib is the pen's
// swatch colour, read from the current theme. The body, collar, sleeve and
// rubber greys are the spec's fixed cursor art (3.3), not theme colours.
const PEN_SHAPES: [string, string | null][] = [
  ["7,13 11,17 21,7 17,3", "#E6E0D4"],
  ["4,16 8,20 11,17 7,13", "#7A746B"],
  ["2,22 4,16 8,20", null], // the nib: the swatch colour
];
const ERASER_SHAPES: [string, string][] = [
  ["7,11 13,17 22,8 16,2", "#7A746B"],
  ["2,16 8,22 13,17 7,11", "#F2EEE8"],
];
function cursorSvg(shapes: [string, string][], halo: string, ink: string): string {
  const poly = (pts: string, attrs: string) => `<polygon points="${pts}" ${attrs} stroke-linejoin="round"/>`;
  const halos = shapes.map(([p]) => poly(p, `fill="none" stroke="${halo}" stroke-width="3"`)).join("");
  const fills = shapes.map(([p, f]) => poly(p, `fill="${f}" stroke="${ink}" stroke-width="1.25"`)).join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">${halos}${fills}</svg>`;
}
const url = (svg: string) => `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
export function penCursor(swatch: string, halo: string, ink: string): string {
  return url(cursorSvg(PEN_SHAPES.map(([p, f]) => [p, f ?? swatch]), halo, ink));
}
export function eraserCursor(halo: string, ink: string): string {
  return url(cursorSvg(ERASER_SHAPES, halo, ink));
}
