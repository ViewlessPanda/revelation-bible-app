// Motion inside the open right column (docs/design/wave-2-layout.md 17.3):
// a second panel opening or one of two closing. Transform and opacity only;
// no height ever animates (every height change is laid out in one frame and
// the motion plays on top of it). The column itself slides as one drawer
// through lib/panel-motion.ts, not through this file.
//
//   enter: measure the staying panel, commit the two-row layout, the entering
//          panel fades in and settles 8px from its own side; a staying panel
//          whose top moved FLIPs back to its old place, with the divider,
//          above the entering panel.
//   leave: measure, commit the one-row layout with the closing panel lifted
//          (absolute at its old rect), it fades out 8px toward its own side;
//          a staying panel whose top moved FLIPs, under the fading panel,
//          and the column clips for the animation's life only.

export type Which = "comm" | "study";
export type Frames = Keyframe[];

const other = (w: Which): Which => (w === "comm" ? "study" : "comm");
/** Commentary settles from above (-y), Study from below (+y). */
const side = (w: Which) => (w === "comm" ? -1 : 1);
const ty = (px: number) => (px === 0 ? "none" : `translateY(${px}px)`);

export function enterFrames(w: Which, y: number): Frames {
  return [{ opacity: 0, transform: ty(side(w) * y) }, { opacity: 1, transform: "none" }];
}
export function leaveFrames(w: Which, y: number): Frames {
  return [{ opacity: 1, transform: "none" }, { opacity: 0, transform: ty(side(w) * y) }];
}
/** The staying panel (and the divider) from its old place to its new one. */
export function flipFrames(dy: number): Frames {
  return [{ transform: ty(dy) }, { transform: "none" }];
}

export interface ColumnMotionIO {
  col(): HTMLElement | null;
  panel(w: Which): HTMLElement | null;
  divider(): HTMLElement | null;
  /** A leave finished: remove the lifted panel. */
  done(w: Which): void;
}

const cs = () => getComputedStyle(document.documentElement);
const token = (n: string) => cs().getPropertyValue(n).trim();
const num = (v: string, fb: number) => { const n = parseFloat(v); return Number.isFinite(n) ? n : fb; };
const ms = (n: string, fb: number) => { const v = token(n); const x = num(v, fb); return /ms$/.test(v) ? x : /s$/.test(v) ? x * 1000 : x; };
const ease = (n: string, fb: string) => token(n) || fb;
const canAnimate = (el: HTMLElement | null): el is HTMLElement => !!el && typeof el.animate === "function";

export function createColumnMotion(io: ColumnMotionIO) {
  let end: (() => void) | null = null;

  /** Jump to the end of a running in-column animation and clean up (17.3 Interruptions). */
  function finishNow() { end?.(); }

  function run(
    play: (add: (el: HTMLElement | null, f: Frames, o: KeyframeAnimationOptions) => void) => void,
    cleanup: () => void,
    lifted?: Which,
  ) {
    const anims: Animation[] = [];
    const moved: HTMLElement[] = [];
    let left = 0, over = false;
    const finish = () => {
      if (over) return;
      over = true;
      // cancel() rejects `finished` with an AbortError; nobody waits on it
      for (const a of anims) { a.finished?.catch(() => {}); a.cancel(); }
      for (const el of moved) el.style.removeProperty("will-change");
      cleanup();
      if (lifted) io.done(lifted);
      if (end === finish) end = null;
    };
    end = finish;
    play((el, f, o) => {
      if (!canAnimate(el)) return;
      el.style.willChange = "transform";
      moved.push(el);
      const a = el.animate(f, o);
      anims.push(a);
      left++;
      a.onfinish = () => { if (--left === 0) finish(); };
    });
    if (!anims.length) finish();
  }

  const reduced = () => token("--panel-slide") === "0";
  const y = () => num(token("--rc-enter-y"), 8);

  return {
    finishNow,

    /** The second panel `w` opens inside the open column. */
    enter(w: Which, commit: () => void) {
      finishNow();
      const stay = io.panel(other(w));
      const oldTop = stay?.getBoundingClientRect().top ?? 0;
      commit();
      const ent = io.panel(w);
      const div = io.divider();
      const newTop = stay?.getBoundingClientRect().top ?? 0;
      const dy = oldTop - newTop;
      const flip = !reduced() && Math.abs(dy) > 0.5;
      const dur = ms("--dur-rc-in", 180), easing = ease("--ease-out", "ease-out");
      const lift = [stay, div].filter((x): x is HTMLElement => !!x);
      if (flip) for (const el of lift) el.classList.add("rc-top");
      run((add) => {
        add(ent, enterFrames(w, y()), { duration: dur, easing });
        if (flip) { add(stay, flipFrames(dy), { duration: dur, easing }); add(div, flipFrames(dy), { duration: dur, easing }); }
      }, () => { for (const el of lift) el.classList.remove("rc-top"); });
    },

    /** One of two panels, `w`, closes inside the open column. */
    /** `commit` lays out one row in one frame with `w` lifted (out of flow). */
    leave(w: Which, commit: () => void) {
      finishNow();
      const col = io.col(), closing = io.panel(w), stay = io.panel(other(w));
      if (!col || !closing) { commit(); io.done(w); return; }
      const r = closing.getBoundingClientRect(), c = col.getBoundingClientRect();
      const oldTop = stay?.getBoundingClientRect().top ?? 0;
      commit();
      for (const [k, v] of [
        ["top", r.top - c.top - col.clientTop], ["left", r.left - c.left - col.clientLeft], ["width", r.width], ["height", r.height],
      ] as const) closing.style.setProperty(k, `${v}px`);
      const dy = oldTop - (stay?.getBoundingClientRect().top ?? 0);
      const flip = !reduced() && Math.abs(dy) > 0.5;
      if (flip) col.classList.add("rc-clip");
      run((add) => {
        add(closing, leaveFrames(w, y()), { duration: ms("--dur-rc-out", 120), easing: ease("--ease-in", "ease-in"), fill: "forwards" });
        if (flip) add(stay, flipFrames(dy), { duration: ms("--dur-rc-in", 180), easing: ease("--ease-out", "ease-out") });
      }, () => {
        col.classList.remove("rc-clip");
        for (const k of ["top", "left", "width", "height"]) closing.style.removeProperty(k);
      }, w);
    },
  };
}
