// The right column's rules (docs/design/wave-2-layout.md 17): split clamp, the
// divider's keys, reset, the rAF-throttled drag, the width limits, and the
// in-column motion keyframes (transform and opacity only). The DOM side is
// tests/right-column.test.ts.
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  RC, clampComm, clampWidth, commHeight, createDrag, dividerAria, dividerKey, restoreSplit, restoreWidthRem, rowsMode,
  splitBounds, splitOf, widthBounds, widthKey,
} from "../src/lib/rcol";
import { enterFrames, flipFrames, leaveFrames } from "../src/lib/rcol-motion";
import { lintCss } from "../scripts/csslint.mjs";

const REM = 16;
const AVAIL = 700; // a 701px column less the 1px divider

describe("split clamp (17.1)", () => {
  it("minimums are 14rem for Commentary and 10rem for Study", () => {
    expect(splitBounds(AVAIL, REM)).toEqual({ min: 224, max: 540, short: false });
    expect(clampComm(10, AVAIL, REM)).toBe(224);
    expect(clampComm(699, AVAIL, REM)).toBe(540);
    expect(clampComm(300, AVAIL, REM)).toBe(300);
  });

  it("a short column drops both minimums to the 6rem floor", () => {
    // 224 + 160 = 384 > 300: floor 96 each
    expect(splitBounds(300, REM)).toEqual({ min: 96, max: 204, short: true });
    expect(clampComm(0, 300, REM)).toBe(96);
    expect(clampComm(999, 300, REM)).toBe(204);
    // exactly fitting is not short
    expect(splitBounds(384, REM).short).toBe(false);
    expect(splitBounds(383, REM).short).toBe(true);
  });

  it("scales with the OS text size (rem)", () => {
    expect(splitBounds(1200, 20)).toEqual({ min: 280, max: 1000, short: false });
  });

  it("the split is a fraction of the column; a window resize re-clamps and never rewrites it", () => {
    const split = 0.2; // saved
    expect(commHeight(split, 700, REM)).toBe(224); // clamped up
    expect(commHeight(split, 2000, REM)).toBe(400); // a taller window: the same fraction, now unclamped
    expect(split).toBe(0.2);
    expect(commHeight(0.9, 700, REM)).toBe(540);
  });

  it("splitOf rounds to 3 decimals; bad values restore the default", () => {
    expect(splitOf(350, 700)).toBe(0.5);
    expect(splitOf(240, 700)).toBe(0.343);
    expect(splitOf(100, 0)).toBe(RC.splitDefault);
    for (const bad of [NaN, -0.1, 1.1, "0.5", null, undefined, Infinity]) expect(restoreSplit(bad)).toBe(0.5);
    expect(restoreSplit(0)).toBe(0);
    expect(restoreSplit(1)).toBe(1);
    expect(restoreSplit(0.12345)).toBe(0.123);
  });
});

describe("the divider's keys (17.2)", () => {
  const H = 350;
  it("arrows move 1rem, Shift 4rem, clamped", () => {
    expect(dividerKey("ArrowDown", false, H, AVAIL, REM)).toBe(366);
    expect(dividerKey("ArrowUp", false, H, AVAIL, REM)).toBe(334);
    expect(dividerKey("ArrowDown", true, H, AVAIL, REM)).toBe(414);
    expect(dividerKey("ArrowUp", true, H, AVAIL, REM)).toBe(286);
    expect(dividerKey("ArrowUp", true, 230, AVAIL, REM)).toBe(224);
    expect(dividerKey("ArrowDown", true, 520, AVAIL, REM)).toBe(540);
  });
  it("Home and End go to the minimums (224px and 160px at 100%); Enter resets", () => {
    expect(dividerKey("Home", false, H, AVAIL, REM)).toBe(224);
    expect(dividerKey("End", false, H, AVAIL, REM)).toBe(AVAIL - 160);
    expect(dividerKey("Enter", false, H, AVAIL, REM)).toBe("reset");
    expect(RC.splitDefault).toBe(0.5);
  });
  it("other keys are not taken (Esc, Tab, letters, sideways arrows)", () => {
    for (const k of ["Escape", "Tab", "a", "ArrowLeft", "ArrowRight", " "]) expect(dividerKey(k, false, H, AVAIL, REM)).toBeNull();
  });
  it("ARIA values: percentage of the column, from the clamp", () => {
    expect(dividerAria(350, AVAIL, REM)).toEqual({ now: 50, min: 32, max: 77, text: "Commentary 50%" });
    expect(dividerAria(224, AVAIL, REM).now).toBe(32);
    expect(dividerAria(0, 0, REM)).toEqual({ now: 0, min: 0, max: 0, text: "Commentary 0%" });
  });
});

describe("collapse: the open panel fills the column (17.1)", () => {
  it("rows by which panels are in flow", () => {
    expect(rowsMode(true, true)).toBe("both");
    expect(rowsMode(true, false)).toBe("comm");
    expect(rowsMode(false, true)).toBe("study");
    expect(rowsMode(false, false)).toBe("none");
  });
});

describe("column width (17.1)", () => {
  it("restore clamps to 18..36 rem; junk is the 22rem default", () => {
    expect(restoreWidthRem(10)).toBe(18);
    expect(restoreWidthRem(50)).toBe(36);
    expect(restoreWidthRem(27.5)).toBe(27.5);
    for (const bad of [NaN, Infinity, "22", null, undefined]) expect(restoreWidthRem(bad)).toBe(22);
  });
  it("limits: 18rem to 36rem, and the window less the reading pane's minimum", () => {
    expect(widthBounds(1600, REM)).toEqual({ min: 288, max: 576 });
    expect(widthBounds(760, REM)).toEqual({ min: 288, max: 336 }); // 760 - 416 - 8
    expect(widthBounds(500, REM)).toEqual({ min: 288, max: 288 }); // never below the minimum
    expect(clampWidth(100, 1600, REM)).toBe(288);
    expect(clampWidth(900, 1600, REM)).toBe(576);
  });
  it("keys are the old splitter's: arrows 16px (Shift 64), Home, End; Enter is not bound", () => {
    expect(widthKey("ArrowLeft", false, 352, 1600, REM)).toBe(368);
    expect(widthKey("ArrowRight", true, 352, 1600, REM)).toBe(288);
    expect(widthKey("Home", false, 352, 1600, REM)).toBe(288);
    expect(widthKey("End", false, 352, 1600, REM)).toBe(576);
    expect(widthKey("Enter", false, 352, 1600, REM)).toBeNull();
  });
});

describe("pointer drag is rAF-throttled: one style write per frame (17.2)", () => {
  function rig(h0 = 350) {
    const writes: number[] = [];
    const commits: number[] = [];
    const frames = new Map<number, () => void>();
    let next = 1;
    const io = {
      height: () => h0,
      clamp: (h: number) => clampComm(h, AVAIL, REM),
      write: (h: number) => writes.push(h),
      commit: (h: number) => commits.push(h),
      raf: (cb: () => void) => { frames.set(next, cb); return next++; },
      caf: (id: number) => { frames.delete(id); },
    };
    const frame = () => { const f = [...frames.values()]; frames.clear(); for (const cb of f) cb(); };
    return { drag: createDrag(io), writes, commits, frames, frame };
  }

  it("many moves between frames write once, with the latest y", () => {
    const r = rig();
    r.drag.start(100);
    for (const y of [101, 110, 120, 140]) r.drag.move(y);
    expect(r.frames.size).toBe(1);
    expect(r.writes).toEqual([]);
    r.frame();
    expect(r.writes).toEqual([390]); // 350 + 140 - 100
    r.drag.move(150);
    r.frame();
    expect(r.writes).toEqual([390, 400]);
    expect(r.commits).toEqual([]); // nothing is saved while dragging
  });

  it("the height is clamped to the minimums", () => {
    const r = rig();
    r.drag.start(500);
    r.drag.move(0);
    r.frame();
    r.drag.move(5000);
    r.frame();
    expect(r.writes).toEqual([224, 540]);
  });

  it("release cancels a pending frame, applies the final y and commits once", () => {
    const r = rig();
    r.drag.start(100);
    r.drag.move(160);
    expect(r.frames.size).toBe(1);
    r.drag.end(170);
    expect(r.frames.size).toBe(0);
    expect(r.writes).toEqual([420]);
    expect(r.commits).toEqual([420]);
    expect(r.drag.active).toBe(false);
    r.drag.end(200); // a stray second release does nothing
    r.drag.move(200);
    expect(r.commits).toEqual([420]);
    expect(r.frames.size).toBe(0);
  });

  it("Esc (cancel) puts the divider back and saves nothing", () => {
    const r = rig();
    r.drag.start(100);
    r.drag.move(180);
    r.frame();
    r.drag.move(190);
    r.drag.cancel();
    expect(r.frames.size).toBe(0);
    expect(r.writes.at(-1)).toBe(350);
    expect(r.commits).toEqual([]);
    expect(r.drag.active).toBe(false);
  });

  it("a click without a move saves nothing", () => {
    const r = rig();
    r.drag.start(100);
    r.drag.end(100);
    expect(r.commits).toEqual([]);
  });

  it("moves before a start are ignored", () => {
    const r = rig();
    r.drag.move(100);
    expect(r.frames.size).toBe(0);
  });
});

describe("in-column motion animates only transform and opacity, 8px from the panel's own side (17.3)", () => {
  const props = (fr: Keyframe[]) => [...new Set(fr.flatMap((f) => Object.keys(f)))].sort();
  it("enter and leave", () => {
    expect(enterFrames("comm", 8)).toEqual([{ opacity: 0, transform: "translateY(-8px)" }, { opacity: 1, transform: "none" }]);
    expect(enterFrames("study", 8)).toEqual([{ opacity: 0, transform: "translateY(8px)" }, { opacity: 1, transform: "none" }]);
    expect(leaveFrames("comm", 8)).toEqual([{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(-8px)" }]);
    expect(leaveFrames("study", 8)).toEqual([{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(8px)" }]);
    // reduced motion: --rc-enter-y is 0, the panel only fades
    expect(enterFrames("study", 0)).toEqual([{ opacity: 0, transform: "none" }, { opacity: 1, transform: "none" }]);
    for (const w of ["comm", "study"] as const) for (const fr of [enterFrames(w, 8), leaveFrames(w, 8)]) expect(props(fr)).toEqual(["opacity", "transform"]);
  });
  it("FLIP of the staying panel", () => {
    expect(flipFrames(-120)).toEqual([{ transform: "translateY(-120px)" }, { transform: "none" }]);
    expect(props(flipFrames(40))).toEqual(["transform"]);
  });
});

describe("the column's CSS has no height or width transition (12.3)", () => {
  const css = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");
  it("the rules exist and move nothing by transition", () => {
    expect(css).toMatch(/\.rcol\.two \{ grid-template-rows: var\(--rc-comm-h/);
    expect(css).toMatch(/\.rc-div::after \{[^}]*transition: opacity/);
    expect(lintCss(css).filter((p: string) => p.includes("motion lint"))).toEqual([]);
    for (const rule of css.match(/[^{}]*\.(rcol|rc-div|panel)[^{}]*\{[^}]*\}/g) ?? []) {
      expect(rule, rule).not.toMatch(/transition[^;}]*\b(height|width|grid-template|min-height|top|bottom)\b/);
      expect(rule, rule).not.toMatch(/@keyframes/);
    }
  });
  it("the lint rejects a height transition on the column", () => {
    for (const bad of [".rcol { transition: height 120ms; }", ".rcol { transition: grid-template-rows 120ms; }", ".rc-div { transition: top 120ms; }"]) {
      expect(lintCss(bad).some((p: string) => p.includes("motion lint")), bad).toBe(true);
    }
  });
});
