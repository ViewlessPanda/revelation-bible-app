// T20 hover / focus pass (red-letters-and-highlights.md 3.4.6 assertion 5) on
// every real-KJV golden chapter, red letters on and off: in each forced state
// every colour, every text node and every layout-affecting computed property
// is unchanged from rest, and the wrapped-word decoration is the token colour
// exactly when hovered or selected without the focus ring. The selected
// states (assertion 6, v0.3.1) put aria-expanded on every .w. Harness:
// hover-pass.ts.
import { describe, expect, it } from "vitest";
import { kjv } from "./kjv-golden";
import { FORCED, renderChapter, restPass, setCss, statePass, unrender, useHarness, type State } from "./hover-pass";

useHarness();

const STATES: [string, State][] = [
  ["hover", { hover: true, fv: false, wrapped: false }],
  ["hover + focus-visible", { hover: true, fv: true, wrapped: false }],
  ["hover, wrapped words", { hover: true, fv: false, wrapped: true }],
  ["hover + focus-visible, wrapped words", { hover: true, fv: true, wrapped: true }],
  // the selected pass (3.4.6 assertion 6, v0.3.1): aria-expanded on every .w
  ["selected", { hover: false, fv: false, wrapped: false, selected: true }],
  ["selected + hover", { hover: true, fv: false, wrapped: false, selected: true }],
  ["selected + focus-visible", { hover: false, fv: true, wrapped: false, selected: true }],
  ["selected, wrapped words", { hover: false, fv: false, wrapped: true, selected: true }],
  ["selected + focus-visible, wrapped words", { hover: false, fv: true, wrapped: true, selected: true }],
];

describe("T20 hover / focus pass on the real KJV", () => {
  it.each(kjv.chapters.map((c) => [`${c.osis} ${c.chapter}`, c] as const))(
    "%s: colours, text and layout unchanged in every state",
    (_label, ch) => {
      setCss(FORCED);
      for (const red of ["on", "off"] as const) {
        document.documentElement.dataset.redLetters = red;
        const reading = renderChapter(ch);
        expect(reading.querySelectorAll(".vtext .w").length).toBeGreaterThan(0);
        const rest = restPass(reading, ch, red === "on");
        for (const [name, s] of STATES) expect(statePass(reading, rest, s), `${red} ${name}`).toEqual([]);
        unrender();
      }
    },
    60_000,
  );
});
