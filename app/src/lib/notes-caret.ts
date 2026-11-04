// Daily notes: the smooth caret (wave-2-layout.md 8.5, exact).
//
// The native caret is transparent (`.ProseMirror { caret-color: transparent }`,
// so screen readers and Magnifier still follow the real one) and a 2px bar is
// drawn in `.caret-layer`, an overlay inside the editor's scrolling content,
// so scrolling moves it with the text. Its position is written as a
// transform in the plugin view's `update`, which ProseMirror calls in the same
// frame as the DOM update (never a later rAF). Typing and arrow keys glide
// (80ms); a pointer click or drag, focus arriving, a move of more than 3
// lines, undo/redo, IME composition and resizes snap. Hidden when the editor
// isn't focused, when the selection isn't empty, and when the window is in
// the background. Motion is transform/opacity only; reduced motion is the
// tokens' `--caret-glide: 0` and the stepped blink in app.css.
import { Plugin, PluginKey } from "prosemirror-state";

export interface Rect { left: number; top: number; right?: number; bottom: number }

/** Round to device pixels (8.5 Position). */
export const toDevice = (v: number, dpr: number) => Math.round(v * dpr) / dpr;

/** The caret's transform for `coords` (from coordsAtPos) inside `layer`, with the caret `height` px tall. */
export function caretGeometry(coords: Rect, layer: { left: number; top: number }, height: number, dpr: number) {
  const x = toDevice(coords.left - layer.left - 1, dpr);
  const y = toDevice((coords.top + coords.bottom) / 2 - layer.top - height / 2, dpr);
  return { x, y };
}

/** A move snaps instead of gliding (8.5 Snap). */
export function snaps(o: { forced: boolean; dy: number; lineHeight: number; composing: boolean; first: boolean }): boolean {
  return o.forced || o.first || o.composing || Math.abs(o.dy) > 3 * o.lineHeight;
}

const caretKey = new PluginKey<{ snap: boolean }>("smooth-caret");

export interface CaretOptions {
  /** The overlay the caret is drawn in (positioned, inside the scrolling content). */
  layer(): HTMLElement | null;
  /** The window is in the foreground. */
  windowActive?(): boolean;
}

export function caretPlugin(o: CaretOptions): Plugin {
  return new Plugin({
    key: caretKey,
    state: {
      init: () => ({ snap: true }),
      // undo/redo and pointer selections snap (8.5)
      apply: (tr) => ({ snap: !!tr.getMeta("history$") || !!tr.getMeta("pointer") }),
    },
    view(view) {
      const el = document.createElement("div");
      el.className = "caret";
      el.setAttribute("aria-hidden", "true");
      let last: { x: number; y: number } | null = null;
      let force = true; // the next move snaps
      let active = o.windowActive?.() ?? true;

      const hide = () => { el.classList.add("hidden"); };

      function place(snapNow: boolean) {
        const layer = o.layer();
        if (!layer) return;
        if (el.parentElement !== layer) layer.appendChild(el);
        const sel = view.state.selection;
        if (!view.hasFocus() || !sel.empty || !active) { hide(); force = true; return; }
        let coords: Rect;
        try { coords = view.coordsAtPos(sel.head); } catch { hide(); return; }
        const lr = layer.getBoundingClientRect();
        // the caret takes the size of the font at the caret (a larger run gets a matching caret)
        const at = view.domAtPos(sel.head);
        const host = (at.node.nodeType === 1 ? at.node : at.node.parentElement) as HTMLElement | null;
        const cs = host ? getComputedStyle(host) : null;
        const font = cs ? parseFloat(cs.fontSize) || 15 : 15;
        const line = cs ? parseFloat(cs.lineHeight) || font * 1.6 : font * 1.6;
        const height = font * 1.15; // --caret-height: 1.15em
        el.style.fontSize = `${font}px`;
        const g = caretGeometry(coords, lr, height, window.devicePixelRatio || 1);
        const snap = snaps({ forced: snapNow || force, dy: last ? g.y - last.y : 0, lineHeight: line, composing: view.composing, first: !last });
        if (snap) el.classList.add("snap");
        el.style.transform = `translate(${g.x}px, ${g.y}px)`;
        if (snap) {
          void getComputedStyle(el).transform; // commit the jump before the glide comes back
          el.classList.remove("snap");
        }
        el.classList.remove("hidden");
        // blink restarts on every move: solid for --caret-hold, then the blink
        el.classList.remove("blink");
        void el.offsetWidth;
        el.classList.add("blink");
        last = g;
        force = false;
      }

      const onDown = () => { force = true; };
      const onFocus = () => { force = true; place(true); };
      const onBlur = () => hide();
      const onWin = (e: Event) => {
        if (e.target !== window) return;
        active = e.type === "focus";
        if (active) { force = true; place(true); } else hide();
      };
      view.dom.addEventListener("mousedown", onDown);
      view.dom.addEventListener("focus", onFocus);
      view.dom.addEventListener("blur", onBlur);
      window.addEventListener("focus", onWin);
      window.addEventListener("blur", onWin);
      const ro = typeof ResizeObserver === "function"
        ? new ResizeObserver(() => { force = true; place(true); })
        : null;
      const layer = o.layer();
      if (layer && ro) ro.observe(layer);
      place(true);

      return {
        update(v, prev) {
          const st = caretKey.getState(v.state);
          if (v.state.doc === prev.doc && v.state.selection.eq(prev.selection) && !st?.snap) {
            if (!v.hasFocus()) hide();
            return;
          }
          place(!!st?.snap);
        },
        destroy() {
          view.dom.removeEventListener("mousedown", onDown);
          view.dom.removeEventListener("focus", onFocus);
          view.dom.removeEventListener("blur", onBlur);
          window.removeEventListener("focus", onWin);
          window.removeEventListener("blur", onWin);
          ro?.disconnect();
          el.remove();
        },
      };
    },
  });
}

/** Test hook: the caret element of a view's layer. */
export const caretOf = (layer: HTMLElement) => layer.querySelector<HTMLElement>(".caret");
