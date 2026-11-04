// The slide-out state machine (lib/panel-motion.ts) against a fake host, and
// the Web Animations keyframes lib/panel-dom.ts builds (wave-2-layout.md 3).
import { beforeEach, describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { _resetRunning, createPanel, type Anim, type MotionHost, type MoveKind, type PanelLayout } from "../src/lib/panel-motion";
import { deltas, fadeFrames, slideFrames } from "../src/lib/panel-dom";
import { lintCss } from "../scripts/csslint.mjs";

class FakeAnim implements Anim {
  onfinish: Anim["onfinish"] = null;
  reversed = false;
  state: "running" | "finished" | "cancelled" = "running";
  reverse() { this.reversed = !this.reversed; }
  finish() { if (this.state !== "running") return; this.state = "finished"; this.onfinish?.call(this, {}); }
  cancel() { this.state = "cancelled"; }
}

function fakeHost(reduced = false, animCount = 3) {
  const log: string[] = [];
  let anims: FakeAnim[] = [];
  const host: MotionHost & { log: string[]; anims: () => FakeAnim[]; endAll: () => void } = {
    log,
    reduced: () => reduced,
    layout: (to: PanelLayout) => log.push(`layout:${to}`),
    play: (k: MoveKind) => { log.push(`play:${k}`); anims = Array.from({ length: animCount }, () => new FakeAnim()); return anims; },
    clear: () => log.push("clear"),
    anims: () => anims,
    endAll: () => { for (const a of anims) { a.state = "running"; a.finish(); } },
  };
  return host;
}

beforeEach(() => _resetRunning());

describe("panel state machine (3.1, 3.2)", () => {
  it("open: drawer, slide in, then the track is committed at the end", () => {
    const h = fakeHost();
    const phases: string[] = [];
    const p = createPanel(h, false, (x) => phases.push(x));
    p.open();
    expect(h.log).toEqual(["layout:drawer", "play:slide-in"]);
    expect(p.phase).toBe("opening");
    h.anims()[0].finish();
    h.anims()[1].finish();
    expect(p.phase).toBe("opening"); // waits for the whole group
    h.anims()[2].finish();
    expect(h.log).toEqual(["layout:drawer", "play:slide-in", "layout:track", "clear"]);
    expect(p.phase).toBe("open");
    expect(phases).toEqual(["opening", "open"]);
  });

  it("close: the track goes at once (drawer), slide out, then hidden", () => {
    const h = fakeHost();
    const p = createPanel(h, true);
    expect(h.log).toEqual(["layout:track"]); // restored open: simply there
    p.close();
    expect(h.log.slice(1)).toEqual(["layout:drawer", "play:slide-out"]);
    h.endAll();
    expect(h.log.slice(-2)).toEqual(["layout:hidden", "clear"]);
    expect(p.phase).toBe("closed");
    expect(p.target).toBe(false);
  });

  it("a toggle mid-open reverses the group and ends hidden with no track ever laid out", () => {
    const h = fakeHost();
    const p = createPanel(h, false);
    p.open();
    const g = h.anims();
    p.toggle();
    expect(g.every((a) => a.reversed)).toBe(true);
    expect(p.phase).toBe("closing");
    expect(h.log).toEqual(["layout:drawer", "play:slide-in"]); // nothing new played or laid out
    h.endAll();
    expect(h.log.slice(-2)).toEqual(["layout:hidden", "clear"]);
    expect(h.log).not.toContain("layout:track");
    expect(p.phase).toBe("closed");
  });

  it("a toggle mid-close reverses and re-commits the track when the reversed group finishes", () => {
    const h = fakeHost();
    const p = createPanel(h, true);
    p.close();
    p.open();
    expect(h.anims().every((a) => a.reversed)).toBe(true);
    h.endAll();
    expect(h.log.slice(-2)).toEqual(["layout:track", "clear"]);
    expect(p.phase).toBe("open");
  });

  it("two reversals turn round twice and keep the first direction", () => {
    const h = fakeHost();
    const p = createPanel(h, false);
    p.open(); p.close(); p.open();
    expect(h.anims().every((a) => !a.reversed)).toBe(true);
    h.endAll();
    expect(p.phase).toBe("open");
  });

  it("opening an open panel (or closing a closed one) does nothing", () => {
    const h = fakeHost();
    const p = createPanel(h, false);
    p.close();
    expect(h.log).toEqual([]);
    p.open(); h.endAll();
    const n = h.log.length;
    p.open();
    expect(h.log.length).toBe(n);
  });

  it("reduced motion: open commits at once and fades; close lifts and fades (3.4)", () => {
    const h = fakeHost(true);
    const p = createPanel(h, false);
    p.open();
    expect(h.log).toEqual(["layout:track", "play:fade-in"]);
    h.endAll();
    p.close();
    expect(h.log.slice(-2)).toEqual(["layout:drawer", "play:fade-out"]);
    h.endAll();
    expect(h.log.slice(-2)).toEqual(["layout:hidden", "clear"]);
  });

  it("no animations (no Web Animations): the move settles at once", () => {
    const h = fakeHost(false, 0);
    const p = createPanel(h, false);
    p.open();
    expect(h.log).toEqual(["layout:drawer", "play:slide-in", "layout:track", "clear"]);
    expect(p.phase).toBe("open");
  });

  it("two changes at once: the running group of another panel finishes and commits first (3.3)", () => {
    const a = fakeHost(), b = fakeHost();
    const pa = createPanel(a, false), pb = createPanel(b, false);
    pa.open();
    pb.open();
    expect(a.log.slice(-2)).toEqual(["layout:track", "clear"]);
    expect(pa.phase).toBe("open");
    expect(pb.phase).toBe("opening");
    expect(a.anims().every((x) => x.state === "finished")).toBe(true);
  });

  it("finishNow commits; late finish events are ignored", () => {
    const h = fakeHost();
    const p = createPanel(h, false);
    p.open();
    const g = h.anims();
    p.finishNow();
    expect(p.phase).toBe("open");
    const n = h.log.length;
    for (const x of g) x.onfinish?.call(x, {});
    expect(h.log.length).toBe(n);
  });
});

describe("keyframes animate only transform and opacity (12.3)", () => {
  const props = (frames: Keyframe[]) => frames.flatMap((f) => Object.keys(f).filter((k) => !["offset", "easing", "composite"].includes(k)));

  it("slide and fade builders", () => {
    for (const kind of ["slide-in", "slide-out"] as const) {
      for (const side of ["left", "right"] as const) {
        const f = slideFrames(kind, side, 352);
        for (const fr of [f.panel, f.column, f.start, f.centre, f.end]) {
          expect(fr).toHaveLength(2);
          expect(new Set(props(fr))).toEqual(new Set(["transform"]));
        }
      }
    }
    for (const kind of ["fade-in", "fade-out"] as const) {
      const f = fadeFrames(kind, 0.5);
      expect(new Set([...props(f.panel), ...props(f.settle)])).toEqual(new Set(["opacity"]));
    }
  });

  it("geometry: a right panel moves the column by -W/2 and the end group by -W; it slides from +100%", () => {
    expect(deltas("right", 352)).toEqual({ drawer: "translateX(100%)", column: -176, start: 0, end: -352 });
    expect(deltas("left", 320)).toEqual({ drawer: "translateX(-100%)", column: 160, start: 320, end: 0 });
    const open = slideFrames("slide-in", "right", 352);
    expect(open.panel).toEqual([{ transform: "translateX(100%)" }, { transform: "none" }]);
    expect(open.column).toEqual([{ transform: "none" }, { transform: "translateX(-176px)" }]);
    const close = slideFrames("slide-out", "right", 352);
    expect(close.panel).toEqual([{ transform: "none" }, { transform: "translateX(100%)" }]);
    expect(close.column).toEqual([{ transform: "translateX(-176px)" }, { transform: "none" }]); // FLIP
  });

  it("no source file calls .animate() outside the panel host and the entry cross-fades", () => {
    const src = resolve(process.cwd(), "src");
    const files: string[] = [];
    const walk = (d: string) => { for (const e of readdirSync(d, { withFileTypes: true })) e.isDirectory() ? walk(join(d, e.name)) : files.push(join(d, e.name)); };
    walk(src);
    const callers = files.filter((f) => /\.animate\(/.test(readFileSync(f, "utf8"))).map((f) => f.slice(src.length + 1).replace(/\\/g, "/"));
    expect(callers.sort()).toEqual(["components/CommentaryPanel.svelte", "components/StudyPanel.svelte", "lib/panel-dom.ts", "lib/rcol-motion.ts"]);
    for (const f of ["components/StudyPanel.svelte", "components/CommentaryPanel.svelte"]) {
      const sp = readFileSync(join(src, f), "utf8");
      expect(sp).toMatch(/\.animate\(\[\{ opacity: 0 \}, \{ opacity: 1 \}\]/);
      expect(sp.match(/\.animate\(/g)!.length).toBe(1);
    }
  });
});

describe("motion lint: no layout transitions (12.3)", () => {
  it("rejects transitions and keyframes on width, grid tracks, margins and insets", () => {
    for (const css of [
      ".panel { transition: width 280ms; }",
      ".app { transition: grid-template-columns 280ms ease; }",
      ".reading { transition-property: margin-left; }",
      ".panel { transition: inset-inline-end 220ms; }",
      ".panel { transition: right 220ms, opacity 120ms; }",
      ".panel { transition: all 220ms; }",
      "@keyframes x { from { width: 0; } }",
      "@keyframes y { to { left: 10px; } }",
    ]) {
      expect(lintCss(css).some((p: string) => p.includes("motion lint")), css).toBe(true);
    }
    expect(lintCss(".panel { transition: transform 220ms, opacity 120ms; }").filter((p: string) => p.includes("motion lint"))).toEqual([]);
  });

  it("the app's CSS has no layout transition and the drawer rule moves nothing by itself", () => {
    const app = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");
    expect(lintCss(app).filter((p: string) => p.includes("motion lint"))).toEqual([]);
    expect(app).not.toMatch(/transition[^;{}]*\b(width|grid-template|margin|inset|left|right)\b/);
    expect(app).not.toMatch(/\.panel\.entering|panel-in/);
  });
});
