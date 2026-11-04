// Presentation of a stored lexicon entry tree (pipeline/DESIGN.md 7, "as
// built"). The characters shown are the stored ones, as text nodes only; this
// file decides only grouping, labels and which attributes are display text.
//
// - Whitespace-only text nodes that are direct children of <entry> (parent
//   NULL: OSHB's CRLF + TAB indentation, MGNT's spaces and line feeds between
//   fields) are layout and are not drawn. Every other text node is drawn
//   exactly as stored, in document order.
// - Labels ("Meaning", "KJV usage") and the key shown on an empty
//   cross-reference (`<strongsref/>` shows its canonical key, e.g. "G25") are
//   UI text built from stored names/keys, like verse numbers (DESIGN.md 7).
import type { LexAttr, LexEntry, LexNode } from "./types";

/** Field labels for elements drawn as their own block. */
export const FIELD_LABELS: Record<string, string> = {
  source: "Source", meaning: "Meaning", usage: "Usage", note: "Note",
  strongs_derivation: "Derivation", strongs_def: "Definition", kjv_def: "KJV usage",
};

/** Elements that make up the headword line while no field has started. */
const HEAD = new Set(["w", "strongs", "greek", "pronunciation"]);

/** Attributes whose values are display text (DESIGN.md 7), per element. */
const DISPLAY_ATTRS: Record<string, string[]> = {
  w: ["xlit", "pron"],
  greek: ["unicode", "translit"],
  pronunciation: ["strongs"],
};

/** Layout whitespace: the characters the pipeline allows in such nodes. */
export const isLayoutWhitespace = (t: string) => /^[ \t\r\n]*$/.test(t);

export type LexBlock =
  | { kind: "head"; nodes: LexNode[] }
  | { kind: "field"; label: string; node: LexNode }
  | { kind: "para"; nodes: LexNode[] };

/**
 * Group the direct children of <entry> into blocks, in document order.
 * Whitespace-only children of <entry> are dropped as layout, except between
 * two nodes of the same running paragraph (MGNT text directly in <entry>),
 * where they are kept so no words run together.
 */
export function lexBlocks(entry: LexEntry): LexBlock[] {
  const blocks: LexBlock[] = [];
  let fieldSeen = false;
  let pending: LexNode[] = [];
  for (const n of entry.nodes) {
    if (n.parent !== null) continue;
    if (n.text !== null && isLayoutWhitespace(n.text)) { pending.push(n); continue; }
    const name = n.name;
    const last = blocks[blocks.length - 1];
    if (name !== null && name in FIELD_LABELS) {
      fieldSeen = true;
      blocks.push({ kind: "field", label: FIELD_LABELS[name], node: n });
    } else if (name !== null && HEAD.has(name) && !fieldSeen) {
      if (last && last.kind === "head") last.nodes.push(n);
      else blocks.push({ kind: "head", nodes: [n] });
    } else if (last && last.kind === "para") {
      last.nodes.push(...pending, n);
    } else {
      blocks.push({ kind: "para", nodes: [n] });
    }
    pending = [];
  }
  return blocks;
}

/** Child nodes of `n`, in document order. */
export function childrenOf(entry: LexEntry, n: LexNode): LexNode[] {
  return entry.nodes.filter((c) => c.parent === n.n);
}

/** The cross-reference target of an element (w@src, strongsref@strongs). */
export const refKey = (n: LexNode): string | null => n.attrs.find((a) => a.ref_key)?.ref_key ?? null;

/** Attributes of `n` that are display text, in stored order. */
export function displayAttrs(n: LexNode): LexAttr[] {
  const names = DISPLAY_ATTRS[n.name ?? ""] ?? [];
  return n.attrs.filter((a) => names.includes(a.name) && !a.ref_key);
}

export const lexiconName = (id: string) =>
  id === "LEX-HEB-OSHB" ? "Strong's Hebrew" : id === "LEX-GRC-MGNT19" ? "Strong's Greek" : id;
