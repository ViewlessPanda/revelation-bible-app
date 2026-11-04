// Shared render-test checks (PLAN.md 2.6 / 2.8): used by render.test.ts
// (placeholder fixture) and kjv-render.test.ts (real KJV golden). Each check
// returns a list of problems; empty = pass. Offsets are code points, as stored.

export const cp = (s: string) => Array.from(s);
export const cpSlice = (s: string, a: number, b: number) => cp(s).slice(a, b).join("");
const ALLOWED_CLASSES = new Set(["wj", "add", "nd", "nd-sc", "w", "pilcrow"]);
const W_ATTRS = ["class", "data-strong", "role", "tabindex", "data-wrapped", "aria-expanded", "aria-controls"];

/** The stored facts a check needs (a golden verse has at least these). */
export interface StoredVerse {
  v: number;
  text: string;
  red: [number, number][];
}

export function textNodes(root: Node): Text[] {
  const out: Text[] = [];
  const walk = document.createTreeWalker(root, 4 /* SHOW_TEXT */);
  for (let n = walk.nextNode(); n; n = walk.nextNode()) out.push(n as Text);
  return out;
}

/** Structural and text assertions for one verse. */
export function checkVerse(verseEl: HTMLElement, g: StoredVerse): string[] {
  const p: string[] = [];
  const vt = verseEl.querySelector<HTMLElement>(":scope > .vtext");
  if (!vt) return [`v${g.v}: no .vtext`];
  if (vt.textContent !== g.text) p.push(`v${g.v}: DOM text differs from stored text`);
  if (verseEl.querySelector(".wj .vnum, .wj .sr-only, .vtext .vnum")) p.push(`v${g.v}: verse number inside verse text`);
  for (const el of vt.querySelectorAll("*")) {
    if (el.tagName !== "SPAN") p.push(`v${g.v}: <${el.tagName.toLowerCase()}> inside .vtext`);
    if (el.classList.length !== 1 || !ALLOWED_CLASSES.has(el.classList[0])) p.push(`v${g.v}: class "${el.className}" inside .vtext`);
    if (el.hasAttribute("style")) p.push(`v${g.v}: style attribute inside .vtext`);
    if (el.hasAttribute("tabindex") && !el.classList.contains("w")) p.push(`v${g.v}: tabindex outside .w`);
    // 3.4.1: attributes on .w are class, data-strong, role, tabindex, the
    // hover handler's data-wrapped and, on the selected word only,
    // aria-expanded="true" with aria-controls="p-study"; every other span
    // carries only its class
    const isW = el.classList.contains("w");
    const ok = isW ? W_ATTRS : ["class"];
    for (const a of el.getAttributeNames()) if (!ok.includes(a)) p.push(`v${g.v}: attribute ${a} on .${el.className}`);
    if (isW && (el.hasAttribute("aria-expanded") || el.hasAttribute("aria-controls")) &&
        (el.getAttribute("aria-expanded") !== "true" || el.getAttribute("aria-controls") !== "p-study")) {
      p.push(`v${g.v}: aria-expanded/aria-controls on .w must be "true" and "p-study", together`);
    }
  }
  // .wj spans == stored pieces, in order, exactly
  const wjs = Array.from(vt.querySelectorAll<HTMLElement>(".wj"));
  if (wjs.length !== g.red.length) p.push(`v${g.v}: ${wjs.length} .wj spans, ${g.red.length} stored pieces`);
  wjs.forEach((w, i) => {
    const r = g.red[i];
    if (r && w.textContent !== cpSlice(g.text, r[0], r[1])) p.push(`v${g.v}: .wj ${i} text differs from stored range`);
    if (w.closest(".vtext") !== vt || w.parentElement?.closest(".wj")) p.push(`v${g.v}: .wj nesting`);
  });
  // every text node wholly inside one stored piece (and under .wj) or wholly outside (and not under .wj)
  let pos = 0;
  for (const t of textNodes(vt)) {
    const len = cp(t.data).length;
    const a = pos, b = pos + len;
    const inside = g.red.find(([s, e]) => s <= a && b <= e);
    const overlaps = g.red.some(([s, e]) => a < e && s < b);
    const underWj = !!t.parentElement?.closest(".wj");
    if (overlaps && !inside) p.push(`v${g.v}: text node [${a},${b}) straddles a red boundary`);
    if (!!inside !== underWj) p.push(`v${g.v}: text node [${a},${b}) red=${underWj}, stored=${!!inside}`);
    pos = b;
  }
  return p;
}

/** Code points of `.vtext` under an element with class `cls`. */
function coveredBy(vt: HTMLElement, cls: string): Set<number> {
  const out = new Set<number>();
  let pos = 0;
  for (const t of textNodes(vt)) {
    const len = cp(t.data).length;
    const el = t.parentElement?.closest(`.${cls}`);
    if (el && vt.contains(el)) for (let i = pos; i < pos + len; i++) out.add(i);
    pos += len;
  }
  return out;
}

function rangesSet(rs: [number, number][]): Set<number> {
  const out = new Set<number>();
  for (const [s, e] of rs) for (let i = s; i < e; i++) out.add(i);
  return out;
}

const sameSet = (a: Set<number>, b: Set<number>) => a.size === b.size && [...a].every((x) => b.has(x));

/** `.add` / `.nd` cover exactly the stored marks; `.w` spans = stored words and keys. */
export function checkMarksAndWords(
  verseEl: HTMLElement,
  g: StoredVerse & { add: [number, number][]; nd: [number, number][]; words: [number, number, string][] },
): string[] {
  const p: string[] = [];
  const vt = verseEl.querySelector<HTMLElement>(":scope > .vtext")!;
  if (!sameSet(coveredBy(vt, "add"), rangesSet(g.add))) p.push(`v${g.v}: .add differs from stored added marks`);
  if (!sameSet(coveredBy(vt, "nd"), rangesSet(g.nd))) p.push(`v${g.v}: .nd differs from stored divine-name marks`);
  const ws = Array.from(vt.querySelectorAll<HTMLElement>(".w"));
  // a word that crosses a red boundary is split into one .w per side; join by position
  const merged: { s: number; e: number; keys: string }[] = [];
  let pos = 0;
  const posOf = new Map<Text, number>();
  for (const t of textNodes(vt)) { posOf.set(t, pos); pos += cp(t.data).length; }
  ws.forEach((w, i) => {
    const nodes = textNodes(w);
    if (!nodes.length) { p.push(`v${g.v}: empty .w`); return; }
    const s = posOf.get(nodes[0])!;
    const e = s + cp(w.textContent ?? "").length;
    const keys = w.dataset.strong ?? "";
    const last = merged[merged.length - 1];
    const splitByRed = i > 0 && w.closest(".wj") !== ws[i - 1].closest(".wj");
    if (last && last.e === s && last.keys === keys && splitByRed) last.e = e;
    else merged.push({ s, e, keys });
  });
  const want = g.words.map(([s, e, keys]) => ({ s, e, keys }));
  if (JSON.stringify(merged) !== JSON.stringify(want)) p.push(`v${g.v}: .w spans differ from stored words/keys`);
  return p;
}

/** Computed colour of every text node's parent (happy-dom cascade). */
export function checkColours(verseEl: HTMLElement, g: StoredVerse, red: string, text: string): string[] {
  const p: string[] = [];
  const vt = verseEl.querySelector<HTMLElement>(":scope > .vtext")!;
  let pos = 0;
  for (const t of textNodes(vt)) {
    const len = cp(t.data).length;
    const inside = g.red.some(([s, e]) => s <= pos && pos + len <= e);
    const got = getComputedStyle(t.parentElement!).color.toLowerCase();
    const want = (inside ? red : text).toLowerCase();
    if (got !== want) p.push(`v${g.v}: text at ${pos} is ${got}, expected ${want}`);
    pos += len;
  }
  const num = verseEl.querySelector<HTMLElement>(":scope > .vnum");
  if (num && getComputedStyle(num).color.toLowerCase() === red.toLowerCase()) p.push(`v${g.v}: verse number is red`);
  return p;
}
