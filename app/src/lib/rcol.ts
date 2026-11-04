// The right column's numbers and rules (docs/design/wave-2-layout.md 17):
// Commentary on top, Study underneath, a draggable divider between them.
// Everything here is pure (no DOM, no state) so the clamp, the keys, the
// reset and the rAF-throttled drag are tested on their own
// (tests/rcol.test.ts); RightColumn.svelte and App.svelte only call it.

/** Sizes in rem, as tokens.css section 12 (--rc-*). */
export const RC = {
  widthDefault: 22,
  widthMin: 18,
  widthMax: 36,
  /** window minus this is the most the column may take (--reader-w-min, 26rem) plus the splitter's 8px */
  readerMin: 26,
  minComm: 14,
  minStudy: 10,
  floor: 6,
  splitDefault: 0.5,
  step: 1,
  bigStep: 4,
  /** the 1px divider row */
  dividerPx: 1,
} as const;

export type Mode = "none" | "comm" | "study" | "both";

/** Which panels are in the column's flow (the open one fills the column, 17.1). */
export function rowsMode(comm: boolean, study: boolean): Mode {
  return comm && study ? "both" : comm ? "comm" : study ? "study" : "none";
}

// ---- width ---------------------------------------------------------------

/** The column width on restore: clamped to 18..36 rem; anything else is the default. */
export function restoreWidthRem(v: unknown): number {
  const n = typeof v === "number" && Number.isFinite(v) ? v : RC.widthDefault;
  return Math.max(RC.widthMin, Math.min(RC.widthMax, n));
}

/** Width limits in px for a window `winW` wide at `rem` px per rem (17.1). */
export function widthBounds(winW: number, rem: number): { min: number; max: number } {
  const min = RC.widthMin * rem;
  return { min, max: Math.max(min, Math.min(RC.widthMax * rem, winW - RC.readerMin * rem - 8)) };
}

export function clampWidth(px: number, winW: number, rem: number): number {
  const { min, max } = widthBounds(winW, rem);
  return Math.max(min, Math.min(max, px));
}

/** The vertical splitter's keys (17.1): as the panel splitters were, except Enter is not bound. */
export function widthKey(key: string, shift: boolean, px: number, winW: number, rem: number): number | null {
  const { min, max } = widthBounds(winW, rem);
  const step = (shift ? 4 : 1) * 16;
  if (key === "ArrowLeft") return Math.min(max, px + step);
  if (key === "ArrowRight") return Math.max(min, px - step);
  if (key === "Home") return min;
  if (key === "End") return max;
  return null;
}

// ---- height split --------------------------------------------------------

/** Commentary's height limits for a column with `avail` px (column height less the divider). */
export function splitBounds(avail: number, rem: number): { min: number; max: number; short: boolean } {
  let minC = RC.minComm * rem, minS = RC.minStudy * rem;
  const short = minC + minS > avail;
  if (short) minC = minS = RC.floor * rem; // a large OS text size on a short window
  return { min: minC, max: Math.max(minC, avail - minS), short };
}

export function clampComm(h: number, avail: number, rem: number): number {
  const { min, max } = splitBounds(avail, rem);
  return Math.max(min, Math.min(max, h));
}

/** The saved split as a fraction, 3 decimals; an invalid value is the default. */
export function restoreSplit(v: unknown): number {
  return typeof v === "number" && Number.isFinite(v) && v >= 0 && v <= 1 ? Math.round(v * 1000) / 1000 : RC.splitDefault;
}

/** Commentary's px height for a split: clamped; the saved split itself is never rewritten by this. */
export function commHeight(split: number, avail: number, rem: number): number {
  return clampComm(split * avail, avail, rem);
}

/** The split to save for a px height (3 decimals). */
export function splitOf(h: number, avail: number): number {
  return avail > 0 ? restoreSplit(h / avail) : RC.splitDefault;
}

/** The divider's keys (17.2): px height to go to, "reset" for Enter, or null for a key it ignores. */
export function dividerKey(key: string, shift: boolean, h: number, avail: number, rem: number): number | "reset" | null {
  const { min, max } = splitBounds(avail, rem);
  const step = (shift ? RC.bigStep : RC.step) * rem;
  if (key === "ArrowUp") return Math.max(min, Math.min(max, h - step));
  if (key === "ArrowDown") return Math.max(min, Math.min(max, h + step));
  if (key === "Home") return min;
  if (key === "End") return max;
  if (key === "Enter") return "reset";
  return null;
}

/** aria-valuenow / min / max / text from the clamp, as percentages of `avail` (17.2). */
export function dividerAria(h: number, avail: number, rem: number) {
  const { min, max } = splitBounds(avail, rem);
  const pc = (x: number) => (avail > 0 ? Math.round((x / avail) * 100) : 0);
  const now = pc(h);
  return { now, min: pc(min), max: pc(max), text: `Commentary ${now}%` };
}

// ---- pointer drag (17.2) -------------------------------------------------

export interface DragIO {
  /** Commentary's current height in px (at pointer down) */
  height(): number;
  clamp(h: number): number;
  /** the single style write: `--rc-comm-h` */
  write(h: number): void;
  /** the end of the drag: save the split for this height */
  commit(h: number): void;
  raf(cb: () => void): number;
  caf(id: number): void;
}

/**
 * Smooth drag: `move` only stores the latest y and asks for one frame; the
 * frame writes the height once. `end` cancels a pending frame, applies the
 * final y and commits. `cancel` (Esc) puts the divider back where it began.
 */
export function createDrag(io: DragIO) {
  let y0 = 0, h0 = 0, latest = 0, pending = 0, active = false;
  const frame = () => { pending = 0; io.write(io.clamp(h0 + latest - y0)); };
  const stop = () => { active = false; if (pending) { io.caf(pending); pending = 0; } };
  return {
    get active() { return active; },
    start(y: number) { active = true; y0 = latest = y; h0 = io.height(); },
    move(y: number) { if (!active) return; latest = y; if (!pending) pending = io.raf(frame); },
    end(y: number) {
      if (!active) return;
      stop();
      const h = io.clamp(h0 + y - y0);
      io.write(h);
      if (h !== h0) io.commit(h); // a click without a move saves nothing
    },
    cancel() { if (!active) return; stop(); io.write(h0); },
  };
}
