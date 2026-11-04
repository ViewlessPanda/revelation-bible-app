// Strong's hover underline (red-letters-and-highlights.md 3.4.3 v0.3, 3.5):
// the rules as specified, the token colours per theme, the T20 hover pass on
// highlighted verses and that the pass bites, and the data-wrapped handler.
// The T20 hover/focus pass over every golden chapter is kjv-hover.test.ts;
// the harness and what happy-dom can't do are described in hover-pass.ts.
import { describe, expect, it } from "vitest";
import type { HighlightColor } from "../src/lib/types";
import { normSelector, parseCss } from "../scripts/csslint.mjs";
import { kjv } from "./kjv-golden";
import { checkVerse, textNodes } from "./render-check";
import {
  APP, FORCED, TOKENS, renderChapter, restPass, setCss, statePass, token, unrender, useHarness, type State,
} from "./hover-pass";

useHarness();
const TINTS: HighlightColor[] = ["gold", "green", "blue", "rose", "violet"];
// the shortest golden chapter that has red letters
const RED_CH = kjv.chapters.filter((c) => c.verses.some((v) => v.red.length)).sort((a, b) => a.verses.length - b.verses.length)[0];

/** Declarations of the rules with exactly this (normalised) selector, in a media context. */
function declsOf(css: string, sel: string, media: RegExp | null = null): Record<string, string> {
  const out: Record<string, string> = {};
  for (const r of parseCss(css).rules) {
    const m = r.media ?? "";
    if (media ? !media.test(m) : m !== "") continue;
    if (!r.selectors.some((s: string) => normSelector(s) === sel)) continue;
    for (const d of r.decls) out[d.prop] = d.value.replace(/\s+/g, " ").trim();
  }
  return out;
}

describe("Strong's hover underline: the rules (3.4.3, 3.5)", () => {
  it("E1: the empty underline box at rest, exactly as specified", () => {
    expect(declsOf(APP, ".vtext .w::after")).toEqual({
      content: '""',
      position: "absolute",
      "inset-inline": "0",
      top: "calc(100% + var(--word-underline-gap))",
      height: "var(--word-underline-thickness)",
      "background-color": "var(--color-word-underline)",
      transform: "scaleX(0)",
      "transform-origin": "50% 50%",
      opacity: "0",
      "pointer-events": "none",
      transition: "opacity var(--dur-exit) var(--ease-in), transform 0s linear var(--dur-exit)",
    });
    expect(declsOf(APP, ".vtext .w")).toEqual({ color: "inherit", position: "relative", cursor: "pointer" });
  });

  it("E2: hover grows from the centre and fades in; never while keyboard-focused", () => {
    expect(declsOf(APP, ".vtext .w:hover:not(:focus-visible)::after")).toEqual({
      transform: "scaleX(1)",
      opacity: "1",
      transition: "opacity var(--dur-underline-fade-in) var(--ease-linear), transform var(--dur-underline-in) var(--ease-out)",
    });
    expect(declsOf(APP, ".vtext .w:hover")).toEqual({}); // the old outline rule is gone
    expect(declsOf(APP, ".vtext .w:focus-visible")).toEqual({
      outline: "var(--focus-ring)", "outline-offset": "1px", "border-radius": "2px",
    });
    // timings from the tokens (v0.3.1): enter 205 ms scale / 140 ms fade, leave 120 ms fade
    expect(token("--dur-underline-in")).toBe("205ms");
    expect(token("--dur-underline-fade-in")).toBe("140ms");
    expect(token("--dur-exit")).toBe("120ms");
    expect(token("--ease-linear")).toBe("linear");
    // neither line uses the old shared durations any more
    for (const sel of [".vtext .w:hover:not(:focus-visible)::after", ".vtext .w[aria-expanded=true]:not(:focus-visible)::before"])
      expect(declsOf(APP, sel).transition, sel).not.toMatch(/--dur-2\b|--dur-3\b/);
  });

  it("E3: the empty selected-line box at rest: same colour and position as E1, 1px thicker", () => {
    const e1 = declsOf(APP, ".vtext .w::after");
    const e3 = declsOf(APP, ".vtext .w::before");
    expect(e3).toEqual({ ...e1, height: "var(--word-underline-thickness-selected)" });
    // 3.4.6 assertion 6: the selected line's height is the hover line's plus 1px
    expect(TOKENS).toContain("--word-underline-thickness-selected: calc(var(--word-underline-thickness) + 1px);");
    expect(token("--word-underline-thickness-selected")).toBe(`calc(${token("--word-underline-thickness")} + 1px)`);
  });

  it("E4: selected (aria-expanded) grows in with the hover timings; never while keyboard-focused", () => {
    expect(declsOf(APP, ".vtext .w[aria-expanded=true]:not(:focus-visible)::before")).toEqual(
      declsOf(APP, ".vtext .w:hover:not(:focus-visible)::after"),
    );
    // no other rule keys on the selected state except the wrapped-word fallback
    const selRules = parseCss(APP).rules.flatMap((r: { selectors: string[] }) => r.selectors).map(normSelector)
      .filter((s: string) => s.includes("aria-expanded") && s.includes(".w"));
    expect(selRules.sort()).toEqual([
      ".vtext .w[aria-expanded=true]:not(:focus-visible)::before",
      ".vtext .w[data-wrapped][aria-expanded=true]:not(:focus-visible)",
      ".vtext .w[data-wrapped][aria-expanded=true]:not(:focus-visible)",
    ]);
  });

  it("reduced motion: both lines are plain fades (never scaled)", () => {
    for (const pe of ["after", "before"]) {
      expect(declsOf(APP, `:root:not([data-motion=full]) .vtext .w::${pe}`, /prefers-reduced-motion:\s*reduce/))
        .toEqual({ transform: "none" });
      expect(declsOf(APP, `:root[data-motion=reduce] .vtext .w::${pe}`)).toEqual({ transform: "none" });
    }
    // fades are kept in reduced mode: the fade-in and exit durations are never zeroed
    const blocks = [...TOKENS.matchAll(/:root:not\(\[data-motion="full"\]\)\s*\{([^}]*)\}|:root\[data-motion="reduce"\]\s*\{([^}]*)\}/g)]
      .map((m) => m[1] ?? m[2]);
    expect(blocks.length).toBeGreaterThanOrEqual(2);
    for (const b of blocks) expect(b).not.toMatch(/--dur-2\s*:|--dur-exit\s*:|--dur-underline-fade-in\s*:/);
  });

  it("wrapped words: both boxes hidden; a static decoration in the token colour, thicker when selected", () => {
    expect(declsOf(APP, ".vtext .w[data-wrapped]::after")).toEqual({ display: "none" });
    expect(declsOf(APP, ".vtext .w[data-wrapped]::before")).toEqual({ display: "none" });
    const deco = {
      "text-decoration-line": "underline",
      "text-decoration-color": "var(--color-word-underline)",
      "text-decoration-thickness": "var(--word-underline-thickness)",
      "text-underline-position": "under",
      "text-underline-offset": "var(--word-underline-gap)",
      "text-decoration-skip-ink": "auto",
    };
    expect(declsOf(APP, ".vtext .w[data-wrapped]:hover:not(:focus-visible)")).toEqual(deco);
    // the selected fallback shares the hover rule, then a later rule thickens it
    expect(declsOf(APP, ".vtext .w[data-wrapped][aria-expanded=true]:not(:focus-visible)")).toEqual({
      ...deco, "text-decoration-thickness": "var(--word-underline-thickness-selected)",
    });
  });

  it("forced colours: highlights outlined, both lines kept visible (selected = Highlight), chips ringed (3.5)", () => {
    const fc = /forced-colors:\s*active/;
    expect(declsOf(APP, ".verse[data-hl]", fc)).toEqual({ outline: "2px dashed CanvasText", "outline-offset": "0" });
    expect(declsOf(APP, ".vtext .w::after", fc)).toEqual({ "forced-color-adjust": "none", "background-color": "CanvasText" });
    expect(declsOf(APP, ".vtext .w::before", fc)).toEqual({ "forced-color-adjust": "none", "background-color": "Highlight" });
    expect(declsOf(APP, ".swatch::before", fc)).toEqual({ "forced-color-adjust": "none", outline: "1px solid CanvasText" });
  });

  it.each(["dark", "light", "sepia"])("%s: the underline colour is its own token, not red, link, accent, focus or text", (theme) => {
    document.documentElement.dataset.theme = theme;
    const u = token("--color-word-underline");
    expect(u).toMatch(/^#[0-9a-f]{6}$/);
    for (const other of ["--color-red-letter", "--color-link", "--color-accent", "--color-focus", "--color-text"]) {
      expect(token(other), other).not.toBe("");
      expect(u, other).not.toBe(token(other));
    }
    expect(token("--word-underline-gap")).toBe("1px");
    expect(token("--word-underline-thickness")).toBe("max(1px, round(0.06em, 1px))");
  });
});

describe("T20 hover pass on highlighted verses, and that the pass bites", () => {
  it.each(TINTS)("highlight %s on every verse: hover changes no colour, text or layout", (tint) => {
    setCss(FORCED);
    const reading = renderChapter(RED_CH, Object.fromEntries(RED_CH.verses.map((v) => [v.v, tint])));
    expect(reading.querySelector(`.verse[data-hl="${tint}"]`)).not.toBeNull();
    const rest = restPass(reading, RED_CH, true);
    expect(statePass(reading, rest, { hover: true, fv: false, wrapped: false })).toEqual([]);
    expect(statePass(reading, rest, { hover: true, fv: false, wrapped: true })).toEqual([]);
    expect(statePass(reading, rest, { hover: false, fv: false, wrapped: false, selected: true })).toEqual([]);
    expect(statePass(reading, rest, { hover: false, fv: false, wrapped: true, selected: true })).toEqual([]);
  }, 60_000);

  it("catches a hover colour, red lost on hover, a hover box change, a wrong decoration colour", () => {
    const run = (css: string, s: State) => {
      setCss(css);
      const reading = renderChapter(RED_CH);
      const problems = statePass(reading, restPass(reading, RED_CH, true), s);
      unrender();
      return problems;
    };
    const hover = { hover: true, fv: false, wrapped: false };
    expect(run(FORCED, hover)).toEqual([]);
    for (const planted of [
      ".vtext .w[data-force-hover] { color: var(--color-link); }",
      ".wj .w[data-force-hover] { color: var(--color-text); }",
      ".vtext .w[data-force-hover] { padding-bottom: 2px; }",
      ".verse[data-force-hover] .w { font-weight: 600; }",
    ]) expect(run(FORCED + "\n" + planted, hover).length, planted).toBeGreaterThan(0);
    const wrongDeco = FORCED.replace("text-decoration-color: var(--color-word-underline)", "text-decoration-color: var(--color-link)");
    expect(wrongDeco).not.toBe(FORCED);
    expect(run(wrongDeco, { hover: true, fv: false, wrapped: true }).length).toBeGreaterThan(0);
    // the selected pass bites too
    const selected = { hover: false, fv: false, wrapped: false, selected: true };
    expect(run(FORCED, selected)).toEqual([]);
    for (const planted of [
      ".vtext .w[aria-expanded=\"true\"] { color: var(--color-link); }",
      ".vtext .w[aria-expanded=\"true\"] { font-weight: 600; }",
      ".vtext .w[aria-expanded=\"true\"] { padding-bottom: 1px; }",
    ]) expect(run(FORCED + "\n" + planted, selected).length, planted).toBeGreaterThan(0);
    expect(run(wrongDeco, { ...selected, wrapped: true }).length).toBeGreaterThan(0);
  }, 60_000);
});

describe("data-wrapped handler (lib/verse-dom.ts)", () => {
  const ev = (type: string) =>
    type === "focusin" ? new FocusEvent("focusin", { bubbles: true })
      : typeof PointerEvent === "function" ? new PointerEvent(type, { bubbles: true }) : new MouseEvent(type, { bubbles: true });
  const rects = (n: number) => () => Array.from({ length: n }, () => new DOMRect(0, 0, 10, 10)) as unknown as DOMRectList;

  it("sets data-wrapped only on a .w split over lines, touching no text and no other element", () => {
    const ch = RED_CH;
    const reading = renderChapter(ch);
    const before = reading.innerHTML;
    const nodes = textNodes(reading), data = nodes.map((t) => t.data);
    // a word inside a red run, entered through a nested mark if there is one
    const w = reading.querySelector<HTMLElement>(".wj .w:has(.add)") ?? reading.querySelector<HTMLElement>(".wj .w")!;
    const inner = w.querySelector<HTMLElement>(".add, .nd") ?? w;

    Object.defineProperty(w, "getClientRects", { value: rects(2), configurable: true });
    inner.dispatchEvent(ev("pointerover"));
    expect(w.hasAttribute("data-wrapped")).toBe(true);
    expect(Array.from(reading.querySelectorAll("[data-wrapped]"))).toEqual([w]);
    expect(reading.innerHTML).not.toBe(before);
    expect(reading.innerHTML.replace(/ data-wrapped=""/g, "")).toBe(before);
    const now = textNodes(reading);
    expect(now.length).toBe(nodes.length);
    expect(now.every((t, i) => t === nodes[i] && t.data === data[i])).toBe(true);
    const verseEls = Array.from(reading.querySelectorAll<HTMLElement>(".verse"));
    expect(verseEls.flatMap((el, i) => checkVerse(el, ch.verses[i]))).toEqual([]);

    // one line box again: removed on the next pointerover / focus
    Object.defineProperty(w, "getClientRects", { value: rects(1), configurable: true });
    w.dispatchEvent(ev("focusin"));
    expect(w.hasAttribute("data-wrapped")).toBe(false);
    expect(reading.innerHTML).toBe(before);

    Object.defineProperty(w, "getClientRects", { value: rects(3), configurable: true });
    w.dispatchEvent(ev("focusin"));
    expect(w.hasAttribute("data-wrapped")).toBe(true);
    w.dispatchEvent(ev("pointerover"));
    expect(w.hasAttribute("data-wrapped")).toBe(true);
  });

  it("never marks anything but a .w (verse text between words, red runs, verse numbers)", () => {
    const reading = renderChapter(RED_CH);
    const before = reading.innerHTML;
    for (const el of reading.querySelectorAll<HTMLElement>(".vtext, .wj, .vnum, .verse")) {
      Object.defineProperty(el, "getClientRects", { value: rects(2), configurable: true });
      el.dispatchEvent(ev("pointerover"));
      el.dispatchEvent(ev("focusin"));
    }
    expect(reading.innerHTML).toBe(before);
  });

  it("the structure check rejects data-wrapped (or any foreign attribute) off a .w", () => {
    const ch = RED_CH;
    const reading = renderChapter(ch);
    const i = ch.verses.findIndex((v) => v.red.length > 0);
    const el = reading.querySelectorAll<HTMLElement>(".verse")[i];
    el.querySelector(".wj")!.setAttribute("data-wrapped", "");
    expect(checkVerse(el, ch.verses[i]).length).toBeGreaterThan(0);
    el.querySelector(".wj")!.removeAttribute("data-wrapped");
    expect(checkVerse(el, ch.verses[i])).toEqual([]);
    el.querySelector(".w")!.setAttribute("data-x", "1");
    expect(checkVerse(el, ch.verses[i]).length).toBeGreaterThan(0);
  });
});
