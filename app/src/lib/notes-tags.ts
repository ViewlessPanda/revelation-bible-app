// Daily notes: passage tags as a ProseMirror plugin (wave-2-layout.md 8.4).
//
// Rules, as the spec numbers them:
// 1. A top-level block gets its tag from the reading context when its first
//    character is typed or pasted (a touched block with text and `tags: null`).
// 2. That block is "composing" until the caret leaves it or 60s pass with no
//    typing in it; a reading-context change while composing extends its tags
//    (same chapter: the range grows; another chapter: a second tag, at most 3).
// 3. When composing ends the tags are frozen: later edits never retag.
// 4. Enter mid-block keeps the tags on both halves (ProseMirror's split copies
//    the attributes); joining two blocks gives the union (the Backspace and
//    Delete wrappers here). A tag jump ends composing first.
// 5. Removing a tag or adding the current context are ordinary undoable steps.
// 6. Tags are attributes, never text: the chips are widget decorations
//    (`contenteditable="false"`), so typing never changes the document's text.
import { Plugin, PluginKey, type Command, type EditorState, type Transaction } from "prosemirror-state";
import { Decoration, DecorationSet, type EditorView } from "prosemirror-view";
import type { Node as PMNode } from "prosemirror-model";
import { ReplaceStep } from "prosemirror-transform";
import { asTags, extend, sameTags, union, type PassageTag, type Tags } from "./notes-schema";

/** Composing ends after this long without typing in the block (rule 2). */
export const COMPOSE_IDLE_MS = 60_000;

export interface TagState {
  /** The reading context (shown in the toolbar as "Tagging …"). */
  ctx: PassageTag | null;
  /** The composing block: its start position and when it was last typed in. */
  composing: { pos: number; at: number } | null;
}

type Meta = { ctx?: PassageTag | null; end?: true; auto?: true; refresh?: true };

export const tagKey = new PluginKey<TagState>("passage-tags");

export interface TagOptions {
  /** The chip text ("John 3:16"). */
  label(t: PassageTag): string;
  /** A chip was clicked: jump there (composing has already ended). */
  jump(t: PassageTag): void;
  now?(): number;
  /** The empty editor's placeholder (8.1). */
  placeholder?: string;
}

/** The top-level block containing `pos`: its index and start. */
export function topBlockAt(doc: PMNode, pos: number): { index: number; pos: number; node: PMNode } | null {
  if (!doc.childCount) return null;
  const $p = doc.resolve(Math.max(0, Math.min(pos, doc.content.size)));
  const index = $p.depth > 0 ? $p.index(0) : Math.min($p.index(0), doc.childCount - 1);
  const node = doc.child(index);
  return { index, pos: $p.posAtIndex(index, 0), node };
}

const hasText = (n: PMNode) => n.textContent.length > 0;

/** Tags found inside a block (a list's paragraphs), for a list that took over tagged paragraphs. */
function innerTags(n: PMNode): Tags {
  let out: Tags = null;
  n.descendants((d) => {
    const t = asTags(d.attrs.tags);
    if (t.length) out = union(out, t);
    return true;
  });
  return out;
}

/** Ranges of `doc` (after all `trs`) touched by those transactions. */
function touched(trs: readonly Transaction[]): [number, number][] {
  let ranges: [number, number][] = [];
  for (const tr of trs) {
    for (const map of tr.mapping.maps) {
      ranges = ranges.map(([a, b]) => [map.map(a, -1), map.map(b, 1)]);
      map.forEach((_s, _e, ns, ne) => { ranges.push([ns, ne]); });
    }
  }
  return ranges;
}

export function tagPlugin(o: TagOptions): Plugin<TagState> {
  const now = o.now ?? (() => Date.now());
  // decorations are rebuilt only when the doc (or a refresh) changes
  let cache: { doc: PMNode; set: DecorationSet } | null = null;

  return new Plugin<TagState>({
    key: tagKey,
    state: {
      init: () => ({ ctx: null, composing: null }),
      apply(tr, prev, _old, next): TagState {
        // W2A-N1: the tagger must never stop a transaction reaching the saver
        try { return applyTags(tr, prev, next); } catch { return { ctx: prev.ctx, composing: null }; }
      },
    },
    appendTransaction(trs, _old, state) {
      try { return appendTags(trs, state); } catch { return null; }
    },
    props: {
      decorations(state) {
        if (cache && cache.doc === state.doc) return cache.set;
        let set = DecorationSet.empty;
        try { set = DecorationSet.create(state.doc, decorate(state.doc, o)); } catch { /* no chips rather than no editor */ }
        cache = { doc: state.doc, set };
        return set;
      },
    },
  });

  function applyTags(tr: Transaction, prev: TagState, next: EditorState): TagState {
    const meta = tr.getMeta(tagKey) as Meta | undefined;
    if (meta?.refresh) cache = null;
    let ctx = prev.ctx;
    let composing = prev.composing;
    if (composing) {
      const r = tr.mapping.mapResult(composing.pos, 1);
      composing = r.deleted ? null : { pos: r.pos, at: composing.at };
    }
    if (meta?.end) composing = null;
    if (meta && "ctx" in meta) {
      ctx = meta.ctx ?? null;
      if (composing && now() - composing.at > COMPOSE_IDLE_MS) composing = null; // rule 2: 60s idle
    }
    const head = topBlockAt(next.doc, next.selection.head);
    // typing, deleting or pasting (a ReplaceStep; not a tag change, a list toggle or undo)
    const typed = tr.docChanged && !meta?.auto && !tr.getMeta("history$") && tr.steps.some((x) => x instanceof ReplaceStep);
    if (typed && head && hasText(head.node)) composing = { pos: head.pos, at: now() };
    // the caret left the composing block (rule 2)
    if (composing && (!head || head.pos !== composing.pos)) composing = null;
    if (composing) {
      const b = next.doc.nodeAt(composing.pos);
      if (!b || !b.isBlock) composing = null;
    }
    return { ctx, composing };
  }

  function appendTags(trs: readonly Transaction[], state: EditorState): Transaction | null {
    const st = tagKey.getState(state)!;
    const tr = state.tr;
    // rule 1: touched top-level blocks that gained text get the context; emptied ones lose their tags
    const ranges = touched(trs.filter((t) => !(t.getMeta(tagKey) as Meta | undefined)?.auto));
    if (ranges.length) {
      state.doc.forEach((node, pos) => {
        const end = pos + node.nodeSize;
        if (!ranges.some(([a, b]) => a <= end && b >= pos)) return;
        const tags = node.attrs.tags as Tags;
        if (!hasText(node)) {
          if (tags !== null) tr.setNodeAttribute(pos, "tags", null);
        } else if (tags === null) {
          const inner = node.type.name === "paragraph" ? null : innerTags(node);
          const next = inner ?? (st.ctx ? [st.ctx] : null);
          if (next) tr.setNodeAttribute(pos, "tags", next);
        }
      });
    }
    // rule 2: the context changed while composing -> extend the composing block's tags
    const ctxChanged = trs.some((t) => { const m = t.getMeta(tagKey) as Meta | undefined; return !!m && "ctx" in m; });
    if (ctxChanged && st.composing && st.ctx) {
      const node = state.doc.nodeAt(st.composing.pos);
      const tags = asTags(node?.attrs.tags);
      if (node && tags.length) {
        const next = extend(tags, st.ctx);
        if (next !== tags) tr.setNodeAttribute(st.composing.pos, "tags", next);
      }
    }
    if (!tr.docChanged) return null;
    const auto: Meta = { auto: true };
    tr.setMeta(tagKey, auto);
    if (ctxChanged) tr.setMeta("addToHistory", false); // an automatic extension isn't an undo step
    return tr;
  }
}

/** The context changed (navigation, or 300ms after scrolling stops). */
export function setContext(view: EditorView, ctx: PassageTag | null) {
  const cur = tagKey.getState(view.state)?.ctx ?? null;
  if (cur === ctx || (cur && ctx && sameTags([cur], [ctx]))) return;
  const m: Meta = { ctx };
  view.dispatch(view.state.tr.setMeta(tagKey, m).setMeta("addToHistory", false));
}

/** End composing now (a tag jump must never extend the block being written, rule 4). */
export function endComposing(view: EditorView) {
  const m: Meta = { end: true };
  view.dispatch(view.state.tr.setMeta(tagKey, m).setMeta("addToHistory", false));
}

/** Labels changed (books loaded, translation switched): redraw the chips. */
export function refreshTags(view: EditorView) {
  const m: Meta = { refresh: true };
  view.dispatch(view.state.tr.setMeta(tagKey, m).setMeta("addToHistory", false));
}

/** The run of consecutive top-level blocks with identical tags that contains block `index`. */
export function runAt(doc: PMNode, index: number): { from: number; to: number } {
  const tags = doc.child(index).attrs.tags as Tags;
  let from = index, to = index;
  while (from > 0 && sameTags(doc.child(from - 1).attrs.tags, tags)) from--;
  while (to + 1 < doc.childCount && sameTags(doc.child(to + 1).attrs.tags, tags)) to++;
  return { from, to };
}

function setRunTags(tr: Transaction, doc: PMNode, index: number, f: (t: PassageTag[]) => Tags) {
  const { from, to } = runAt(doc, index);
  let pos = 0;
  for (let i = 0; i < doc.childCount; i++) {
    const n = doc.child(i);
    if (i >= from && i <= to) tr.setNodeAttribute(pos, "tags", f(asTags(n.attrs.tags)));
    pos += n.nodeSize;
  }
}

/** Remove tag `i` from the run of blocks around block `index` (the chip's ×; rule 5, undoable). */
export function removeTag(view: EditorView, index: number, i: number) {
  const tr = view.state.tr;
  setRunTags(tr, view.state.doc, index, (t) => t.filter((_, j) => j !== i));
  view.dispatch(tr);
}

/** "Tag with the current passage" on the caret's block (rule 5, undoable). */
export const tagWithContext: Command = (state, dispatch) => {
  const ctx = tagKey.getState(state)?.ctx;
  const b = topBlockAt(state.doc, state.selection.head);
  if (!ctx || !b || !hasText(b.node)) return false;
  const tags = asTags(b.node.attrs.tags);
  const next = extend(tags, ctx);
  if (next === tags && tags.length) return false;
  if (dispatch) dispatch(state.tr.setNodeAttribute(b.pos, "tags", next));
  return true;
};

/** The caret block's tags (Ctrl+Enter, the paragraph menu). */
export function caretTags(state: EditorState): PassageTag[] {
  const b = topBlockAt(state.doc, state.selection.head);
  return asTags(b?.node.attrs.tags).slice();
}

/**
 * Wrap a command so that the top-level block at the caret afterwards keeps
 * the tags of the block(s) it came from: the union of the caret block and
 * its neighbour in `dir` (a join, rule 4), or just the caret block's (a list
 * toggle or lift, so a re-wrapped paragraph is never retagged).
 */
export function keepTags(cmd: Command, dir: -1 | 0 | 1 = 0): Command {
  return (state, dispatch, view) => {
    const b = topBlockAt(state.doc, state.selection.head);
    if (!b) return cmd(state, dispatch, view);
    let tags = b.node.attrs.tags as Tags;
    // a join of two top-level blocks: the caret at the start (Backspace) or end (Delete) of one
    const $h = state.selection.$head;
    const edge = state.selection.empty && $h.depth === 1
      && (dir < 0 ? $h.parentOffset === 0 : $h.parentOffset === $h.parent.content.size);
    if (dir !== 0 && edge) {
      const j = b.index + dir;
      if (j >= 0 && j < state.doc.childCount) {
        const other = state.doc.child(j).attrs.tags as Tags;
        tags = dir < 0 ? union(other, tags) : union(tags, other);
      }
    }
    if (!dispatch) return cmd(state, undefined, view);
    return cmd(state, (tr) => {
      const after = topBlockAt(tr.doc, tr.selection.head);
      if (after && tags !== null && hasText(after.node) && !sameTags(after.node.attrs.tags, tags)) {
        tr.setNodeAttribute(after.pos, "tags", tags);
      }
      // a paragraph moved into a list keeps nothing of its own (the list holds the tags)
      if (after && after.node.type.name !== "paragraph") {
        after.node.descendants((d, p) => {
          if (d.attrs.tags) tr.setNodeAttribute(after.pos + 1 + p, "tags", null);
          return true;
        });
      }
      dispatch(tr);
    }, view);
  };
}

// ---- decorations: chip rows, the description, the placeholder ----------------

const SVG = "http://www.w3.org/2000/svg";
/** Lucide icons (ISC), drawn with DOM calls: no markup strings. */
function icon(paths: string[], size: number): SVGSVGElement {
  const s = document.createElementNS(SVG, "svg");
  s.setAttribute("viewBox", "0 0 24 24");
  s.setAttribute("width", String(size));
  s.setAttribute("height", String(size));
  s.setAttribute("aria-hidden", "true");
  s.setAttribute("class", "i");
  for (const d of paths) {
    const p = document.createElementNS(SVG, "path");
    p.setAttribute("d", d);
    s.appendChild(p);
  }
  return s;
}
export const bookOpen = (size = 12) => icon(["M12 7v14", "M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z"], size);
const xIcon = () => icon(["M18 6 6 18", "m6 6 12 12"], 12);

export interface ChipHandlers {
  label(t: PassageTag): string;
  jump(t: PassageTag): void;
  remove?(i: number): void;
}

/**
 * One chip row (8.4 Display). Text only through textContent; buttons with
 * `tabindex=-1` in the live editor (`live`), normal buttons in static days.
 */
export function chipRow(tags: PassageTag[], h: ChipHandlers, live: boolean, first: boolean): HTMLElement {
  const row = document.createElement("div");
  row.className = first ? "tag-row first" : "tag-row";
  row.contentEditable = "false";
  tags.forEach((t, i) => {
    const name = h.label(t);
    const wrap = document.createElement("span");
    wrap.className = "tag";
    const chip = document.createElement("button");
    chip.type = "button";
    chip.className = "tag-chip";
    chip.setAttribute("aria-label", `Go to ${name}`);
    chip.title = live ? `Go to ${name} · Ctrl+Enter` : `Go to ${name}`;
    if (live) chip.tabIndex = -1;
    chip.append(bookOpen(12));
    const text = document.createElement("span");
    text.textContent = name;
    chip.append(text);
    // the editor keeps its focus and caret (8.4 Click)
    chip.addEventListener("mousedown", (e) => e.preventDefault());
    chip.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); h.jump(t); });
    wrap.append(chip);
    if (h.remove) {
      const x = document.createElement("button");
      x.type = "button";
      x.className = "tag-x";
      x.setAttribute("aria-label", `Remove tag ${name}`);
      if (live) x.tabIndex = -1;
      x.append(xIcon());
      x.addEventListener("mousedown", (e) => e.preventDefault());
      x.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); h.remove!(i); });
      wrap.append(x);
    }
    row.append(wrap);
  });
  return row;
}

/** The runs of top-level blocks that start a chip row: index, start position, tags. */
export function chipRuns(doc: PMNode): { index: number; pos: number; tags: PassageTag[] }[] {
  const out: { index: number; pos: number; tags: PassageTag[] }[] = [];
  let prev: Tags = null;
  doc.forEach((node, pos, index) => {
    const tags = asTags(node.attrs.tags);
    if (tags.length && (index === 0 || !sameTags(prev, tags))) out.push({ index, pos, tags });
    prev = tags;
  });
  return out;
}

function decorate(doc: PMNode, o: TagOptions): Decoration[] {
  const out: Decoration[] = [];
  for (const run of chipRuns(doc)) {
    const key = `tags:${run.index === 0 ? "f" : ""}${run.tags.map((t) => `${t.book}.${t.chapter}.${t.v1}.${t.v2}:${o.label(t)}`).join("|")}`;
    out.push(Decoration.widget(run.pos, (view) => chipRow(run.tags, {
      label: o.label,
      jump: (t) => { endComposing(view); o.jump(t); },
      remove: (i) => {
        const b = topBlockAt(view.state.doc, run.pos + 1);
        if (b) removeTag(view, b.index, i);
      },
    }, true, run.index === 0), { side: -1, key, ignoreSelection: true, stopEvent: () => true }));
  }
  doc.forEach((node, pos) => {
    const tags = asTags(node.attrs.tags);
    if (tags.length) {
      out.push(Decoration.node(pos, pos + node.nodeSize, { "aria-description": `Passage: ${tags.map(o.label).join(", ")}` }));
    }
  });
  const only = doc.childCount === 1 ? doc.firstChild : null;
  if (o.placeholder && only && only.type.name === "paragraph" && only.content.size === 0) {
    out.push(Decoration.node(0, only.nodeSize, { class: "is-empty", "data-placeholder": o.placeholder }));
  }
  return out;
}
