// Daily notes: the editor's state (keys, input rules, plugins) and the static
// rendering of earlier days (wave-2-layout.md 8.1, 8.3).
//
// Nothing here builds markup from strings: the live editor is ProseMirror's
// own DOM from the schema, and earlier days are drawn by ProseMirror's
// DOMSerializer from the same schema (document.createElement + text nodes),
// so a stored note can only ever show as text with the four formats.
import { DOMSerializer, Fragment, Node as PMNode, Slice, type NodeType } from "prosemirror-model";
import { EditorState, Plugin, type Command } from "prosemirror-state";
import type { EditorView } from "prosemirror-view";
import { history, redo, undo } from "prosemirror-history";
import { keymap } from "prosemirror-keymap";
import { baseKeymap, chainCommands, deleteSelection, joinBackward, joinForward, selectNodeBackward, selectNodeForward, toggleMark } from "prosemirror-commands";
import { inputRules, wrappingInputRule } from "prosemirror-inputrules";
import { liftListItem, sinkListItem, splitListItem, wrapInList } from "prosemirror-schema-list";
import { emptyDoc, MAX_ORDER, schema, validOrder, validTags, type PassageTag, type Tags } from "./notes-schema";
import { caretTags, chipRow, chipRuns, keepTags, tagPlugin, type TagOptions } from "./notes-tags";
import { caretPlugin, type CaretOptions } from "./notes-caret";

const { bullet_list, ordered_list, list_item, hard_break } = schema.nodes;

/** The list type around the caret, if any (for the toolbar's pressed state). */
export function listAt(state: EditorState): NodeType | null {
  const $f = state.selection.$from;
  for (let d = $f.depth; d > 0; d--) {
    const t = $f.node(d).type;
    if (t === bullet_list || t === ordered_list) return t;
  }
  return null;
}

/** Bulleted / numbered list (toggle): off if the caret is in that list, converted if in the other one, else wrapped. */
export function toggleList(type: NodeType): Command {
  const lift = keepTags(liftListItem(list_item));
  const wrap = keepTags(wrapInList(type));
  return (state, dispatch, view) => {
    const cur = listAt(state);
    if (cur === type) return lift(state, dispatch, view);
    if (cur) {
      const $f = state.selection.$from;
      for (let d = $f.depth; d > 0; d--) {
        if ($f.node(d).type === cur) {
          if (dispatch) dispatch(state.tr.setNodeMarkup($f.before(d), type, { ...$f.node(d).attrs, ...(type === ordered_list ? { order: 1 } : {}) }));
          return true;
        }
      }
    }
    return wrap(state, dispatch, view);
  };
}

/** A mark is active at the caret or over the whole selection (aria-pressed). */
export function markActive(state: EditorState, name: "strong" | "em"): boolean {
  const type = schema.marks[name];
  const { from, $from, to, empty } = state.selection;
  if (empty) return !!type.isInSet(state.storedMarks || $from.marks());
  return state.doc.rangeHasMark(from, to, type);
}

const inList = (state: EditorState) => listAt(state) !== null;
/** Tab / Shift+Tab indent and outdent inside lists only; elsewhere Tab leaves the editor (8.3). */
const sink: Command = (s, d, v) => inList(s) && sinkListItem(list_item)(s, d, v);
const lift: Command = (s, d, v) => inList(s) && keepTags(liftListItem(list_item))(s, d, v);
const hardBreak: Command = (state, dispatch) => {
  if (dispatch) dispatch(state.tr.replaceSelectionWith(hard_break.create()).scrollIntoView());
  return true;
};

export interface EditorOptions {
  tags: TagOptions;
  caret: CaretOptions;
  /** Ctrl+Enter: go to this block's first tag. */
  jump(t: PassageTag): void;
}

/**
 * The reader's own text with every lone surrogate (half of an emoji, say)
 * replaced by U+FFFD, so the day stays valid JSON for the backend (W2A-N2).
 * Used only on note text, never on Scripture.
 */
export const wellFormed = (s: string) =>
  s.replace(/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/g, "�");

/** A numbered list's start as the backend accepts it (0 to 99,999). */
const clampOrder = (v: unknown): number => {
  if (validOrder(v)) return v;
  const n = Number(v);
  return Number.isFinite(n) ? Math.max(0, Math.min(MAX_ORDER, Math.trunc(n))) : 1;
};

/**
 * A pasted slice rebuilt in the notes schema (W2A-N1/N2): pasted blocks carry
 * no tags (`null`; tags come only from the tagger, from the reading context),
 * a numbered list's start is a whole number 0 to 99,999, and text is well
 * formed. A clipboard's `data-pm-slice` context can't set anything else.
 * With `keep` (only for a slice that came from this same editor, W2A-N4) a
 * block keeps its own tags if they are valid; the rest is cleaned the same.
 */
export function cleanSlice(slice: Slice, keep = false): Slice {
  const node = (n: PMNode): PMNode => {
    if (n.isText) {
      const t = wellFormed(n.text!);
      return t === n.text ? n : schema.text(t, n.marks);
    }
    const spec = n.type.spec.attrs ?? {};
    const attrs: Record<string, unknown> = {};
    // keep (W2A-N4): the slice came from this editor, so a block keeps its own
    // tags, but only if they pass the same check as everything else
    if ("tags" in spec) attrs.tags = keep && validTags(n.attrs.tags) ? n.attrs.tags : null;
    if ("order" in spec) attrs.order = clampOrder(n.attrs.order);
    return n.type.create(attrs, frag(n.content), n.marks);
  };
  const frag = (f: Fragment): Fragment => {
    const out: PMNode[] = [];
    f.forEach((c) => { out.push(node(c)); });
    return Fragment.from(out);
  };
  return new Slice(frag(slice.content), slice.openStart, slice.openEnd);
}

/**
 * Paste and drop (W2A-N1/N2/N4). Text from outside the editor is always
 * cleaned with its tags removed. A slice that came from this same editor keeps
 * its own tags (validated again): an internal drag is `view.dragging`'s own
 * slice; a cut or copy is remembered here, and a paste counts as "the same"
 * only if, once cleaned of tags, it equals what was copied. The clipboard's
 * HTML never carries tags, so nothing outside can ask to keep any.
 */
function createPastePlugin(): Plugin {
  let copied: Slice | null = null;
  const remember = (view: EditorView) => {
    try {
      copied = view.state.selection.empty ? null : cleanSlice(view.state.selection.content(), true);
    } catch { copied = null; }
  };
  return new Plugin({
    props: {
      handleDOMEvents: {
        copy: (view) => { remember(view); return false; },
        cut: (view) => { remember(view); return false; },
      },
      // never let a paste throw inside ProseMirror: nothing is pasted instead
      transformPasted: (slice, view) => {
        try {
          if (view.dragging && view.dragging.slice === slice) return cleanSlice(slice, true);
          const plain = cleanSlice(slice);
          if (copied && cleanSlice(copied).eq(plain)) return copied;
          return plain;
        } catch { return Slice.empty; }
      },
    },
  });
}

export function notesPlugins(o: EditorOptions): Plugin[] {
  const jumpFirst: Command = (state) => {
    const t = caretTags(state)[0];
    if (!t) return false;
    o.jump(t);
    return true;
  };
  return [
    inputRules({
      rules: [
        wrappingInputRule(/^\s*([-*])\s$/, bullet_list),
        wrappingInputRule(/^(\d{1,5})\.\s$/, ordered_list, (m) => ({ order: Number(m[1]) }),
          (m, node) => node.childCount + (node.attrs.order as number) === Number(m[1])),
      ],
    }),
    keymap({
      "Mod-b": toggleMark(schema.marks.strong),
      "Mod-i": toggleMark(schema.marks.em),
      "Mod-Shift-8": toggleList(bullet_list),
      "Mod-Shift-7": toggleList(ordered_list),
      "Mod-z": undo,
      "Mod-y": redo,
      "Mod-Shift-z": redo,
      "Shift-Enter": hardBreak,
      "Mod-Enter": jumpFirst,
      Enter: splitListItem(list_item),
      Tab: sink,
      "Shift-Tab": lift,
      Backspace: chainCommands(deleteSelection, keepTags(joinBackward, -1), selectNodeBackward),
      Delete: chainCommands(deleteSelection, keepTags(joinForward, 1), selectNodeForward),
    }),
    keymap(baseKeymap),
    history(),
    createPastePlugin(),
    tagPlugin(o.tags),
    caretPlugin(o.caret),
  ];
}

export function notesState(doc: PMNode, o: EditorOptions): EditorState {
  return EditorState.create({ schema, doc, plugins: notesPlugins(o) });
}

/**
 * A stored document (JSON) as a ProseMirror node, or null if it isn't one
 * this schema can show (the backend already refuses such rows; this is the
 * second gate). Tags that aren't well formed make the whole day unreadable.
 */
export function parseDoc(json: string | null): PMNode | null {
  if (json === null) return null;
  try {
    const node = PMNode.fromJSON(schema, JSON.parse(json));
    node.check();
    let ok = true;
    node.descendants((n) => {
      const t = n.attrs.tags as Tags | undefined;
      if (t !== undefined && !validTags(t)) ok = false;
      return ok;
    });
    return ok ? node : null;
  } catch {
    return null;
  }
}

/**
 * The day as the backend receives it. The schema already holds only valid
 * attrs; this is the save-time second gate with notes.rs's rules (W2A-N2):
 * tags that aren't 0 to 3 well-formed tags become null, `order` is clamped to
 * 0..99,999, and lone surrogates in the text become U+FFFD. The editor's
 * document itself is not changed here.
 */
export function docJSON(doc: PMNode): string {
  const clean = (v: unknown, key: string): unknown => {
    if (typeof v === "string") return key === "text" ? wellFormed(v) : v;
    if (Array.isArray(v)) return v.map((x) => clean(x, ""));
    if (!v || typeof v !== "object") return v;
    const out: Record<string, unknown> = {};
    for (const [k, x] of Object.entries(v as Record<string, unknown>)) {
      if (k === "attrs" && x && typeof x === "object") {
        const a = { ...(x as Record<string, unknown>) };
        if ("tags" in a && !validTags(a.tags)) a.tags = null;
        if ("order" in a) a.order = clampOrder(a.order);
        out[k] = a;
      } else out[k] = clean(x, k);
    }
    return out;
  };
  return JSON.stringify(clean(doc.toJSON(), ""));
}

/** The day's words as plain text (blocks on their own lines), for "Copy this day's text". */
export const docText = (doc: PMNode) => wellFormed(doc.textBetween(0, doc.content.size, "\n", "\n"));
export const isEmptyDoc = (doc: PMNode) => doc.textContent.length === 0;
export { emptyDoc };

/**
 * An earlier day, drawn read-only into `host` (8.1 "static HTML"): the
 * schema's DOM plus a chip row before each run of identically tagged blocks.
 * Chips are normal buttons here (in the Tab order).
 */
export function renderStatic(host: HTMLElement, doc: PMNode, h: { label(t: PassageTag): string; jump(t: PassageTag): void }) {
  host.replaceChildren();
  const ser = DOMSerializer.fromSchema(schema);
  const runs = new Map(chipRuns(doc).map((r) => [r.index, r.tags]));
  doc.forEach((node, _pos, index) => {
    const tags = runs.get(index);
    if (tags) host.appendChild(chipRow(tags, h, false, index === 0));
    const dom = ser.serializeNode(node) as HTMLElement;
    const own = node.attrs.tags as Tags;
    if (own && own.length) dom.setAttribute("aria-description", `Passage: ${own.map(h.label).join(", ")}`);
    host.appendChild(dom);
  });
}

/** Put `text` on the clipboard (the async API, else a selected textarea). True if it worked. */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.className = "sr-only";
    document.body.appendChild(ta);
    ta.select();
    let ok = false;
    try { ok = document.execCommand("copy"); } catch { ok = false; }
    ta.remove();
    return ok;
  }
}
