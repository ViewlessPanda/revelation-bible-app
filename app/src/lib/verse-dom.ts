// Builds the DOM inside `.vtext` from ready-made runs (DESIGN.md 5.1,
// red-letters-and-highlights.md 3.4.1). Text goes in ONLY as text nodes; the
// renderer never does offset arithmetic and never changes a character.
//
// Element tree per verse text:
//   one <span class="wj"> per stored red piece (runs sharing `rp`)
//     one <span class="w" data-strong="H1 H2"> per word (runs sharing `wn`)
//       <span class="add"> / <span class="nd"> around the run's text node
// Each span has exactly one class; no style attribute, no other element type.
import type { HitRun, Run } from "./types";

function groupBy<T, K>(items: T[], key: (t: T) => K): { key: K; items: T[] }[] {
  const out: { key: K; items: T[] }[] = [];
  for (const it of items) {
    const k = key(it);
    const last = out[out.length - 1];
    if (last && last.key === k) last.items.push(it);
    else out.push({ key: k, items: [it] });
  }
  return out;
}

function span(doc: Document, cls: string): HTMLSpanElement {
  const s = doc.createElement("span");
  s.className = cls;
  return s;
}

function runNode(doc: Document, r: Run): Node {
  let node: Node = doc.createTextNode(r.text);
  if (r.nd) { const s = span(doc, "nd"); s.appendChild(node); node = s; }
  if (r.add) { const s = span(doc, "add"); s.appendChild(node); node = s; }
  return node;
}

function wordNodes(doc: Document, runs: Run[]): Node[] {
  return groupBy(runs, (r) => r.wn).map((g) => {
    if (g.key === undefined) {
      const f = doc.createDocumentFragment();
      for (const r of g.items) f.appendChild(runNode(doc, r));
      return f;
    }
    const s = span(doc, "w");
    s.dataset.strong = (g.items[0].w ?? []).join(" ");
    s.setAttribute("role", "button");
    s.tabIndex = -1;
    for (const r of g.items) s.appendChild(runNode(doc, r));
    return s;
  });
}

/** Replace the children of `container` (a `.vtext`) with the verse's runs. */
export function renderVerseText(container: HTMLElement, runs: Run[]): void {
  const doc = container.ownerDocument;
  const frag = doc.createDocumentFragment();
  for (const g of groupBy(runs, (r) => r.rp)) {
    if (g.key === undefined) {
      for (const n of wordNodes(doc, g.items)) frag.appendChild(n);
    } else {
      const wj = span(doc, "wj");
      for (const n of wordNodes(doc, g.items)) wj.appendChild(n);
      frag.appendChild(wj);
    }
  }
  container.replaceChildren(frag);
}

/**
 * A search result's verse text (wave-2-layout 6.4, 6.5): the reader's tree
 * (one .wj per red piece, .add and .nd) without Strong's words, and each
 * matched part's text node inside <mark class="hit">, innermost, so a mark
 * crossing a red-letter or italic boundary is split at it and the colour
 * lock's selectors are untouched. Text nodes only; the runs are split by the
 * core, so nothing here does offset arithmetic.
 */
export function renderResultText(container: HTMLElement, runs: HitRun[]): void {
  const doc = container.ownerDocument;
  const node = (r: HitRun): Node => {
    let n: Node = doc.createTextNode(r.text);
    if (r.hit) { const m = doc.createElement("mark"); m.className = "hit"; m.appendChild(n); n = m; }
    if (r.nd) { const s = span(doc, "nd"); s.appendChild(n); n = s; }
    if (r.add) { const s = span(doc, "add"); s.appendChild(n); n = s; }
    return n;
  };
  const frag = doc.createDocumentFragment();
  for (const g of groupBy(runs, (r) => r.rp)) {
    if (g.key === undefined) {
      for (const r of g.items) frag.appendChild(node(r));
    } else {
      const wj = span(doc, "wj");
      for (const r of g.items) wj.appendChild(node(r));
      frag.appendChild(wj);
    }
  }
  container.replaceChildren(frag);
}

/** Svelte action: `<div class="rtext" use:resultText={hit.runs}>`. */
export function resultText(node: HTMLElement, runs: HitRun[]) {
  renderResultText(node, runs);
  return { update(next: HitRun[]) { renderResultText(node, next); } };
}

/**
 * Wrapped-word state for the hover underline (red-letters-and-highlights.md
 * 3.4.3): a `.w` split over several line boxes gets `data-wrapped` (CSS then
 * draws a static text-decoration instead of the ::after box); otherwise the
 * attribute is removed. Presentation only: it touches no text node, adds no
 * element, changes no layout, and only ever lands on a `.w` span.
 */
export function updateWrapped(e: Event): void {
  const container = e.currentTarget as HTMLElement | null;
  const target = e.target as Element | null;
  if (!container || !target || typeof target.closest !== "function") return;
  // pen mode suspends Strong's hover (highlights-pen.md 4.4); focus still updates
  if (e.type === "pointerover" && container.ownerDocument.documentElement.hasAttribute("data-pen")) return;
  const w = target.closest(".w");
  if (!w || !container.contains(w)) return;
  checkWrapped(w);
}

/** The same check for one `.w` (the selected word, after selection and every layout change). */
export function checkWrapped(w: Element): void {
  if (!w.classList.contains("w")) return;
  if (w.getClientRects().length > 1) {
    if (!w.hasAttribute("data-wrapped")) w.setAttribute("data-wrapped", "");
  } else if (w.hasAttribute("data-wrapped")) {
    w.removeAttribute("data-wrapped");
  }
}

/** Svelte action: `<span class="vtext" use:verseText={verse.runs}>`. */
export function verseText(node: HTMLElement, runs: Run[]) {
  renderVerseText(node, runs);
  node.addEventListener("pointerover", updateWrapped);
  node.addEventListener("focusin", updateWrapped);
  return {
    update(next: Run[]) { renderVerseText(node, next); },
    destroy() {
      node.removeEventListener("pointerover", updateWrapped);
      node.removeEventListener("focusin", updateWrapped);
    },
  };
}
