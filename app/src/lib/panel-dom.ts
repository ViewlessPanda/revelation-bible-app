// The page side of the slide-out panels (docs/design/wave-2-layout.md 3):
// a MotionHost for lib/panel-motion.ts. It measures, applies the one-frame
// layout through the caller (Svelte state + flushSync), and plays the Web
// Animations group. Every keyframe it builds animates only `transform` or
// `opacity` (tests/panel-motion.test.ts checks the builders), and the CSS
// lint rejects any transition on width, grid tracks, margins or insets.
import type { Anim, MotionHost, MoveKind, PanelLayout } from "./panel-motion";

export type Side = "left" | "right";
export type Frames = Keyframe[];

export interface DomPanelOptions {
  /** Which window edge the panel slides from (Notes left; Commentary, Study right). */
  side: Side;
  panel(): HTMLElement | null;
  /** The reading pane's scroll container (`.reader-scroll`). */
  pane(): HTMLElement | null;
  /** The reading column (`.reading`). */
  column(): HTMLElement | null;
  /** Top-bar groups that ride along (reader.md 2). */
  bar(): { start: HTMLElement | null; centre: HTMLElement | null; end: HTMLElement | null };
  /** Other visible panels whose x changes with this one (wave-2-layout.md 3.1
   *  step 4): an inner right panel (Commentary when Study moves) moves with
   *  the pane's end edge. */
  others?(): HTMLElement[];
  /** Apply the layout in one frame (Svelte state + flushSync), keeping the scroll anchor. */
  apply(to: PanelLayout): void;
  /** Keep the reading position across a re-wrap. */
  anchor: { capture(): unknown; restore(a: unknown): void };
}

/** Pixel offsets for one move of a panel W px wide (3.1 step 4). */
export function deltas(side: Side, w: number) {
  const s = side === "right" ? 1 : -1;
  return {
    /** the drawer's off-screen transform, as a fraction of its own width */
    drawer: `translateX(${s * 100}%)`,
    /** the reading column and the centre group: the pane's centre moves by W/2 */
    column: (-s * w) / 2,
    start: side === "left" ? w : 0,
    end: side === "right" ? -w : 0,
  };
}

const tx = (px: number) => (px === 0 ? "none" : `translateX(${px}px)`);

/** Keyframes for a move; `from` and `to` are the two ends in time order. */
export function slideFrames(kind: "slide-in" | "slide-out", side: Side, w: number) {
  const d = deltas(side, w);
  const pair = (rest: string, moved: string): Frames =>
    kind === "slide-in" ? [{ transform: rest }, { transform: moved }] : [{ transform: moved }, { transform: rest }];
  return {
    // in: off-screen -> in place; out: in place -> off-screen
    panel: kind === "slide-in" ? [{ transform: d.drawer }, { transform: "none" }] : [{ transform: "none" }, { transform: d.drawer }],
    // in: glide to the new place; out: FLIP from the old place
    column: pair("none", tx(d.column)),
    start: pair("none", tx(d.start)),
    centre: pair("none", tx(d.column)),
    end: pair("none", tx(d.end)),
    zero: { start: d.start === 0, end: d.end === 0 },
  };
}

export function fadeFrames(kind: "fade-in" | "fade-out", settleFrom: number): { panel: Frames; settle: Frames } {
  return {
    panel: kind === "fade-in" ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 1 }, { opacity: 0 }],
    settle: [{ opacity: settleFrom }, { opacity: 1 }],
  };
}

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}
function ms(name: string, fallback: number): number {
  const v = token(name);
  const n = parseFloat(v);
  if (!Number.isFinite(n)) return fallback;
  return /ms$/.test(v) ? n : /s$/.test(v) ? n * 1000 : n;
}
const ease = (name: string, fallback: string) => token(name) || fallback;
const num = (v: string) => { const n = parseFloat(v); return Number.isFinite(n) ? n : 0; };

/** Content-box width of the reading column. */
function contentWidth(col: HTMLElement): number {
  const cs = getComputedStyle(col);
  return col.clientWidth - num(cs.paddingLeft) - num(cs.paddingRight);
}

export function createDomPanelHost(o: DomPanelOptions): MotionHost {
  let group: Animation[] = [];
  let extra: Animation[] = [];
  let moved: HTMLElement[] = [];
  let reflowed = false; // the last layout change re-wrapped the column
  let narrowed = false; // the column's max-width was set to its final width for this move

  const canAnimate = (el: HTMLElement | null): el is HTMLElement => !!el && typeof el.animate === "function";

  function run(el: HTMLElement | null, frames: Frames, opts: KeyframeAnimationOptions, into: Animation[]) {
    if (!canAnimate(el)) return;
    if (opts.fill !== undefined && frames.some((f) => "transform" in f)) { el.style.willChange = "transform"; moved.push(el); }
    into.push(el.animate(frames, opts));
  }

  function settleFade(col: HTMLElement | null, into: Animation[], duration?: number) {
    const from = num(token("--reflow-fade-from")) || 0.5;
    run(col, fadeFrames("fade-in", from).settle, { duration: duration ?? ms("--dur-2", 120), easing: ease("--ease-out", "ease-out") }, into);
  }

  return {
    reduced: () => token("--panel-slide") === "0",

    layout(to) {
      const col = o.column();
      const before = col ? contentWidth(col) : 0;
      o.apply(to);
      const after = col ? contentWidth(col) : 0;
      reflowed = Math.abs(after - before) > 0.5;
    },

    play(kind: MoveKind): Anim[] {
      group = [];
      extra = [];
      const panel = o.panel(), col = o.column(), pane = o.pane(), bar = o.bar();
      if (!canAnimate(panel)) return [];
      const w = panel.getBoundingClientRect().width;
      if (kind === "slide-in" || kind === "slide-out") {
        const opts: KeyframeAnimationOptions = kind === "slide-in"
          ? { duration: ms("--dur-panel-in", 280), easing: ease("--ease-panel-in", "ease-out"), fill: "both", composite: "replace" }
          : { duration: ms("--dur-panel-out", 220), easing: ease("--ease-panel-out", "ease-in-out"), fill: "both", composite: "replace" };
        const f = slideFrames(kind, o.side, w);
        if (kind === "slide-in" && col && pane) {
          // 3.1 step 3: re-wrap once, now, to the final line length
          const cs = getComputedStyle(col);
          const pad = num(cs.paddingLeft) + num(cs.paddingRight);
          const measure = num(cs.maxWidth) || Infinity;
          const cur = contentWidth(col);
          const final = Math.min(measure, pane.clientWidth - w - pad);
          if (cur > 0 && final > 0 && final < cur - 0.5) {
            const a = o.anchor.capture();
            col.style.setProperty("max-width", `${final}px`);
            narrowed = true;
            o.anchor.restore(a);
            settleFade(col, extra);
          }
        } else if (kind === "slide-out" && reflowed) {
          settleFade(col, extra); // 3.2 step 3: it re-wrapped in the commit frame
        }
        run(panel, f.panel, opts, group);
        run(col, f.column, opts, group);
        run(bar.centre, f.centre, opts, group);
        if (!f.zero.start) run(bar.start, f.start, opts, group);
        if (!f.zero.end) {
          run(bar.end, f.end, opts, group);
          for (const el of o.others?.() ?? []) run(el, f.end, opts, group);
        }
      } else {
        // 3.4 reduced motion: commit at once, the panel fades, the column settles
        const dur = ms(kind === "fade-in" ? "--dur-panel-in" : "--dur-panel-out", 120);
        const f = fadeFrames(kind, num(token("--reflow-fade-from")) || 0.5);
        run(panel, f.panel, { duration: dur, easing: ease(kind === "fade-in" ? "--ease-out" : "--ease-in", "ease"), fill: "both" }, group);
        run(col, f.settle, { duration: dur, easing: ease("--ease-out", "ease-out"), fill: "both" }, group);
      }
      return group as unknown as Anim[];
    },

    clear() {
      for (const a of [...group, ...extra]) a.cancel();
      group = [];
      extra = [];
      for (const el of moved) el.style.removeProperty("will-change");
      moved = [];
      const col = o.column();
      if (narrowed && col) {
        const a = o.anchor.capture();
        col.style.removeProperty("max-width");
        o.anchor.restore(a);
      }
      narrowed = false;
    },
  };
}
