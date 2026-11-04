// Slide-out side panels (docs/design/wave-2-layout.md section 3, v0.2).
//
// One state machine per collapsible panel (Study today; Notes and Commentary
// use the same controller later). It never touches the DOM itself: a host
// (lib/panel-dom.ts for the real page, a fake in tests) does the one-frame
// layout changes and plays the animation group. The rule it encodes:
// layout changes in one frame; only transform and opacity animate; while
// anything moves the reading pane keeps the WIDER of its two widths:
//
//   open:   layout "drawer" (overlay, no track) -> slide in -> layout "track"
//   close:  layout "drawer" (track removed at once) -> slide out -> "hidden"
//   reduced motion (--panel-slide 0): commit at once, then a 120 ms fade
//
// A toggle mid-move reverses the running group, so everything turns round
// from where it is; the layout for the final target is applied when the
// reversed group finishes. Starting a move on one panel while another
// panel's group is running finishes (and commits) that group first.

/** Where the panel is in the layout. */
export type PanelLayout = "hidden" | "drawer" | "track";
export type Phase = "closed" | "opening" | "open" | "closing";
export type MoveKind = "slide-in" | "slide-out" | "fade-in" | "fade-out";

/** The part of a Web Animation the controller needs. */
export interface Anim {
  onfinish: ((this: Anim, ev: unknown) => unknown) | null;
  reverse(): void;
  finish(): void;
  cancel(): void;
}

export interface MotionHost {
  /** Reduced motion is on (`--panel-slide` is 0): fade instead of slide. */
  reduced(): boolean;
  /** One-frame layout change (with the scroll anchor kept). */
  layout(to: PanelLayout): void;
  /** Start the group for a move: every animation shares duration and curve. */
  play(kind: MoveKind): Anim[];
  /** After a move settles: cancel the group, drop transforms, inline sizes, will-change. */
  clear(): void;
}

/** A running move of any panel (only one at a time, wave-2-layout.md 3.3). */
let running: { finishNow(): void } | null = null;

export interface PanelController {
  readonly phase: Phase;
  /** Where the panel is heading (true = open). Saved as the panel's open state. */
  readonly target: boolean;
  open(): void;
  close(): void;
  toggle(): void;
  /** Jump to the end of a running move and commit it (no-op when idle). */
  finishNow(): void;
}

export function createPanel(host: MotionHost, initiallyOpen: boolean, onPhase?: (p: Phase) => void): PanelController {
  let phase: Phase = initiallyOpen ? "open" : "closed";
  let target = initiallyOpen;
  let group: Anim[] | null = null;
  let left = 0; // animations of the group still running in the current direction
  let gen = 0; // stale onfinish callbacks (after finishNow) are ignored

  const setPhase = (p: Phase) => { phase = p; onPhase?.(p); };
  if (initiallyOpen) host.layout("track");

  function settle() {
    group = null;
    if (running === self) running = null;
    host.layout(target ? "track" : "hidden");
    host.clear();
    setPhase(target ? "open" : "closed");
  }

  function start(kind: MoveKind) {
    const g = ++gen;
    const anims = host.play(kind);
    if (!anims.length) { settle(); return; }
    group = anims;
    left = anims.length;
    running = self;
    for (const a of anims) {
      a.onfinish = () => { if (g === gen && --left === 0) settle(); };
    }
  }

  function set(open: boolean) {
    if (open === target) return; // already there or heading there
    target = open;
    if (group) {
      // turn the whole group round from where it is (3.3 Interruptions)
      left = group.length;
      for (const a of group) a.reverse();
      setPhase(open ? "opening" : "closing");
      return;
    }
    if (running && running !== self) running.finishNow(); // two changes at once (3.3)
    if (open) {
      if (host.reduced()) { host.layout("track"); setPhase("opening"); start("fade-in"); }
      else { host.layout("drawer"); setPhase("opening"); start("slide-in"); }
    } else {
      host.layout("drawer"); // the track goes at once; the panel stays where it was, lifted
      setPhase("closing");
      start(host.reduced() ? "fade-out" : "slide-out");
    }
  }

  const self: PanelController = {
    get phase() { return phase; },
    get target() { return target; },
    open: () => set(true),
    close: () => set(false),
    toggle: () => set(!target),
    finishNow() {
      if (!group) return;
      gen++; // the finish events that follow belong to nobody
      for (const a of group) a.finish();
      settle();
    },
  };
  return self;
}

/** Test-only: forget the module-level running move. */
export function _resetRunning() { running = null; }
