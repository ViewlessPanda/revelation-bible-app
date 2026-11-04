// The paint layer for range highlights (highlights-pen.md 7.2, 7.5) and the
// pointer-to-offset mapping (4.3). Reads the verse-text DOM, never writes to
// it: boxes go only into `.hl-ink`, a sibling after `.vtext` that holds empty
// `<span class="hl-box">` elements (no text node, ever). `.vtext` and its
// text nodes are exactly as the renderer built them.
import { cpLen, cpToU16, edgesOf, lineBoxes, padEnds, u16ToCp, type Drawn, type Piece, type Rect, type Spot } from "./pen";

interface Table { nodes: Text[]; starts: number[]; len: number }
const tables = new WeakMap<HTMLElement, Table>();

/** The text nodes of a `.vtext` in order with each one's code-point start (built once). */
export function textTable(vt: HTMLElement): Table {
  const hit = tables.get(vt);
  if (hit) return hit;
  const nodes: Text[] = [], starts: number[] = [];
  let cp = 0;
  const walk = vt.ownerDocument.createTreeWalker(vt, 4 /* SHOW_TEXT */);
  for (let n = walk.nextNode(); n; n = walk.nextNode()) {
    nodes.push(n as Text);
    starts.push(cp);
    cp += cpLen((n as Text).data);
  }
  const t = { nodes, starts, len: cp };
  tables.set(vt, t);
  return t;
}

/** 4.3 step 2: a caret (text node, UTF-16 offset) inside `.vtext` as a code-point offset. */
export function caretOffset(vt: HTMLElement, node: Node, u16: number): number | null {
  const t = textTable(vt);
  const i = t.nodes.indexOf(node as Text);
  if (i < 0) return null;
  return t.starts[i] + u16ToCp(t.nodes[i].data, u16);
}

/** Glyph rects of `[start, end)` of a verse, one Range per text node part (7.5). */
export function pieceRects(vt: HTMLElement, start: number, end: number): DOMRect[] {
  const t = textTable(vt);
  const doc = vt.ownerDocument;
  const out: DOMRect[] = [];
  t.nodes.forEach((n, i) => {
    const a = t.starts[i], b = a + cpLen(n.data);
    if (b <= start || end <= a) return;
    const r = doc.createRange();
    r.setStart(n, cpToU16(n.data, Math.max(start, a) - a));
    r.setEnd(n, cpToU16(n.data, Math.min(end, b) - a));
    if (typeof r.getClientRects === "function") out.push(...Array.from(r.getClientRects()));
  });
  return out;
}

type Caret = { node: Node; offset: number } | null;
function caretFromPoint(doc: Document, x: number, y: number): Caret {
  const d = doc as Document & {
    caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null;
    caretRangeFromPoint?: (x: number, y: number) => Range | null;
  };
  if (typeof d.caretPositionFromPoint === "function") {
    const p = d.caretPositionFromPoint(x, y);
    return p ? { node: p.offsetNode, offset: p.offset } : null;
  }
  if (typeof d.caretRangeFromPoint === "function") {
    const r = d.caretRangeFromPoint(x, y);
    return r ? { node: r.startContainer, offset: r.startOffset } : null;
  }
  return null;
}

const verseOf = (el: Element | null) => el?.closest<HTMLElement>(".verse") ?? null;
const vtextOf = (verse: HTMLElement) => verse.querySelector<HTMLElement>(":scope > .vtext");

/**
 * 4.3 steps 1 and 3: the chapter spot under a point. `x` is first clamped
 * into the reading column (1px inside), so the margin and gutter land on the
 * nearest line. On a verse number: offset 0 of its verse. Between verses:
 * offset 0 of the verse below if above its first line, else the end of the
 * verse above. `start` true: a press, which can't start outside a verse.
 */
export function spotAt(chapterEl: HTMLElement, column: HTMLElement, x: number, y: number, start: boolean): Spot | null {
  const doc = chapterEl.ownerDocument;
  const col = column.getBoundingClientRect();
  const cx = col.width > 2 ? Math.min(Math.max(x, col.left + 1), col.right - 1) : x;
  const hit = doc.elementFromPoint?.(x, y) ?? null;
  const hv = verseOf(hit);
  if (hit && hv && chapterEl.contains(hv) && (hit.closest(".vnum") || hit.closest(".sr-only"))) {
    return { verse: Number(hv.dataset.v), offset: 0 };
  }
  const c = caretFromPoint(doc, cx, y);
  if (c) {
    const v = verseOf(c.node.nodeType === 1 ? (c.node as Element) : c.node.parentElement);
    const vt = v && vtextOf(v);
    if (v && vt && chapterEl.contains(v) && vt.contains(c.node) && c.node.nodeType === 3) {
      const off = caretOffset(vt, c.node, c.offset);
      if (off !== null) return { verse: Number(v.dataset.v), offset: off };
    }
  }
  if (start && !hv) return null;
  if (start && hv && !chapterEl.contains(hv)) return null;
  // between verses, in padding, or (during a stroke) outside: the nearest verse
  const verses = Array.from(chapterEl.querySelectorAll<HTMLElement>(".verse"));
  if (!verses.length) return null;
  for (let i = 0; i < verses.length; i++) {
    const vt = vtextOf(verses[i]);
    const first = vt?.getClientRects()[0] ?? verses[i].getBoundingClientRect();
    const box = verses[i].getBoundingClientRect();
    if (y < first.top) return { verse: Number(verses[i].dataset.v), offset: 0 };
    if (y <= box.bottom) return endOf(verses[i]);
  }
  return endOf(verses[verses.length - 1]);
}
function endOf(v: HTMLElement): Spot {
  const vt = vtextOf(v);
  return { verse: Number(v.dataset.v), offset: vt ? textTable(vt).len : 0 };
}

/** The code point a click falls on: the character after the caret, or before it if the point is past that one. */
export function charAt(chapterEl: HTMLElement, column: HTMLElement, x: number, y: number): Spot | null {
  const s = spotAt(chapterEl, column, x, y, true);
  if (!s) return null;
  const v = chapterEl.querySelector<HTMLElement>(`.verse[data-v="${s.verse}"]`);
  const vt = v && vtextOf(v);
  if (!vt) return s;
  const len = textTable(vt).len;
  if (s.offset >= len) return { verse: s.verse, offset: len - 1 };
  const r = pieceRects(vt, s.offset, s.offset + 1)[0];
  if (r && s.offset > 0 && (x < r.left || x > r.right)) {
    const p = pieceRects(vt, s.offset - 1, s.offset)[0];
    if (p && x >= p.left - 1 && x <= p.right + 1) return { verse: s.verse, offset: s.offset - 1 };
  }
  return s;
}

export interface InkPiece extends Drawn { erasing?: boolean }

function cssLength(value: string, fontPx: number): number {
  const v = value.trim();
  const n = parseFloat(v);
  if (!Number.isFinite(n)) return 0;
  return v.endsWith("em") ? n * fontPx : n;
}

/**
 * Draw `pieces` into the `.hl-ink` of their verses (7.5 step 5: every read
 * first, then every write, so one layout). Verses whose `.hl-ink` is missing
 * are skipped; every `.hl-ink` not given pieces is emptied.
 */
export function paintInk(chapterEl: HTMLElement, pieces: InkPiece[], texts: Map<number, string>): void {
  const doc = chapterEl.ownerDocument;
  const rootStyle = getComputedStyle(doc.documentElement);
  const byVerse = new Map<number, InkPiece[]>();
  for (const p of pieces) (byVerse.get(p.verse) ?? byVerse.set(p.verse, []).get(p.verse)!).push(p);
  const writes: { ink: HTMLElement; boxes: { r: Rect; c: string; erasing: boolean }[] }[] = [];
  for (const ink of chapterEl.querySelectorAll<HTMLElement>(".verse > .hl-ink")) {
    const verse = ink.parentElement as HTMLElement;
    const vt = vtextOf(verse);
    const list = byVerse.get(Number(verse.dataset.v)) ?? [];
    const boxes: { r: Rect; c: string; erasing: boolean }[] = [];
    if (vt && list.length) {
      const o = ink.getBoundingClientRect();
      const st = getComputedStyle(vt);
      const font = parseFloat(st.fontSize) || 16;
      const padBlock = cssLength(rootStyle.getPropertyValue("--hl-pad-block") || "0.08em", font);
      const padInline = cssLength(rootStyle.getPropertyValue("--hl-pad-inline") || "0.08em", font);
      const gap = cssLength(rootStyle.getPropertyValue("--hl-join-gap") || "2px", font);
      const rtl = st.direction === "rtl";
      const text = texts.get(Number(verse.dataset.v)) ?? "";
      for (const p of list) {
        const rel = pieceRects(vt, p.start, p.end).map((r) => ({
          left: r.left - o.left, right: r.right - o.left, top: r.top - o.top, bottom: r.bottom - o.top,
        }));
        const [se, ee] = edgesOf(text, p, list);
        for (const r of padEnds(lineBoxes(rel, padBlock), se, ee, padInline, gap, rtl)) {
          boxes.push({ r, c: p.color, erasing: !!p.erasing });
        }
      }
    }
    writes.push({ ink, boxes });
  }
  for (const { ink, boxes } of writes) {
    const frag = doc.createDocumentFragment();
    for (const b of boxes) {
      const el = doc.createElement("span");
      el.className = "hl-box";
      el.dataset.c = b.c;
      if (b.erasing) el.setAttribute("data-erasing", "");
      el.style.transform = `translate(${round(b.r.left)}px, ${round(b.r.top)}px)`;
      el.style.width = `${round(b.r.right - b.r.left)}px`;
      el.style.height = `${round(b.r.bottom - b.r.top)}px`;
      frag.appendChild(el);
    }
    ink.replaceChildren(frag);
  }
}
const round = (n: number) => Math.round(n * 100) / 100;

/** The verses `pieces` touch. */
export const versesOf = (pieces: Piece[]): Set<number> => new Set(pieces.map((p) => p.verse));
