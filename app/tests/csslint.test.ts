// Colour-lock CSS lint (red-letters-and-highlights.md 3.4.5 part 1) and the
// motion lint (reader.md 14.3), on the source CSS and, when present, the built CSS.
import { describe, expect, it } from "vitest";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { ALLOWED, lintCss, subjectMayBeVerseText } from "../scripts/csslint.mjs";

const read = (p: string) => readFileSync(resolve(process.cwd(), p), "utf8");
const APP = read("src/styles/app.css");
const TOKENS = read("src/styles/tokens.css");

describe("CSS lint", () => {
  it("the app's CSS passes", () => {
    expect(lintCss(TOKENS + "\n" + APP)).toEqual([]);
  });

  it("components carry no <style> blocks (no scoped class can reach verse text)", () => {
    const dir = resolve(process.cwd(), "src/components");
    const files = readdirSync(dir).filter((f) => f.endsWith(".svelte")).map((f) => `src/components/${f}`).concat(["src/App.svelte"]);
    for (const f of files) expect(read(f), f).not.toMatch(/<style[\s>]/);
    for (const f of files) expect(read(f), f).not.toMatch(/\{@html/);
  });

  it("tokens.css is a verbatim copy of docs/design/tokens.css", () => {
    const docs = resolve(process.cwd(), "../docs/design/tokens.css");
    if (existsSync(docs)) expect(TOKENS).toBe(readFileSync(docs, "utf8"));
  });

  it("the built CSS passes, if a build exists", () => {
    const dir = resolve(process.cwd(), "dist/assets");
    if (!existsSync(dir)) return;
    for (const f of readdirSync(dir).filter((x) => x.endsWith(".css"))) {
      expect(lintCss(readFileSync(resolve(dir, f), "utf8")), f).toEqual([]);
    }
  });

  it.each([
    ["a colour on supplied words", ".add { color: #123456; }"],
    ["a colour on divine-name small caps", ".vtext .nd { color: red; }"],
    ["a hover colour on Strong's words", ".vtext .w:hover { color: var(--color-link); }"],
    ["a UA-style element rule reaching spans", "span { color: blue; }"],
    ["a bare pseudo-class", ":hover { color: blue; }"],
    ["a universal rule", "* { color: inherit; }"],
    ["text-shadow on verse text", ".vtext { text-shadow: 0 0 1px red; }"],
    ["opacity on a verse", ".verse:hover { opacity: 0.8; }"],
    ["a weight on red letters", ".wj { font-weight: 600; }"],
    ["a background on red letters", ".vtext .wj { background: red; }"],
    ["a pseudo-element on red letters", ".wj::before { content: '\"'; }"],
    ["a legacy pseudo-element on verse text", ".vtext:after { content: 'x'; }"],
    ["text-transform on the divine name", ".nd { text-transform: uppercase; }"],
    ["a colour via :is()", ":is(.add, .nd) { color: red; }"],
    ["an attribute-only selector", "[data-strong] { color: red; }"],
    ["a transition on colour", ".btn { transition: color 120ms; }"],
    ["a keyframe moving layout", "@keyframes x { from { margin-left: 4px; } }"],
    // hover underline and selected line (3.4.3): only E1-E4 may declare content/opacity inside .vtext
    ["content on a pseudo-element of a mark", ".vtext .w .add::before { content: \"\"; }"],
    ["content on a marker of a word", ".vtext .w::marker { content: \"\"; }"],
    ["another opacity on the selected line", ".vtext .w::before { opacity: 0.4; }"],
    ["the selected line shown while keyboard-focused", ".vtext .w[aria-expanded=\"true\"]::before { opacity: 1; }"],
    ["text in the selected-line box", ".vtext .w::before { content: \"x\"; }"],
    ["a red selected line", ".vtext .w::before { background-color: var(--color-red-letter); }"],
    ["Highlight outside forced colours", ".vtext .w::before { background-color: Highlight; }"],
    ["Highlight on the hover line under forced colours", "@media (forced-colors: active) { .vtext .w::after { background-color: Highlight; } }"],
    ["a colour on the selected word", ".vtext .w[aria-expanded=\"true\"] { color: var(--color-link); }"],
    ["a weight on the selected word (layout shift)", ".vtext .w[aria-expanded=\"true\"] { font-weight: 600; }"],
    ["a background on the selected word", ".vtext .w[aria-expanded=\"true\"] { opacity: 0.9; }"],
    ["a selected decoration in link colour", ".vtext .w[aria-expanded=\"true\"] { text-decoration-color: var(--color-link); }"],
    ["opacity on the word itself", ".vtext .w:hover { opacity: 0.9; }"],
    ["another opacity on the underline box", ".vtext .w::after { opacity: 0.4; }"],
    ["the underline shown while keyboard-focused", ".vtext .w:hover::after { opacity: 1; }"],
    ["a colour on the underline box", ".vtext .w::after { color: red; }"],
    ["text in the underline box", ".vtext .w::after { content: \"x\"; }"],
    ["a red underline", ".vtext .w::after { background-color: var(--color-red-letter); }"],
    ["a link-coloured underline", ".vtext .w::after { background: var(--color-link); }"],
    ["CanvasText outside forced colours", ".vtext .w::after { background-color: CanvasText; }"],
    ["a decoration in the text colour", ".vtext .w:hover { text-decoration: underline; }"],
    ["a decoration in link colour", ".vtext .w[data-wrapped]:hover { text-decoration-color: var(--color-link); }"],
    ["a prefixed decoration in red", ".vtext .w:hover { -webkit-text-decoration-color: var(--color-red-letter); }"],
    ["hover padding (layout shift)", ".vtext .w:hover { padding-bottom: 2px; }"],
    ["a hover border (layout shift)", ".vtext .w:hover { border-bottom: 1px solid var(--color-word-underline); }"],
    ["a hover weight (layout shift)", ".vtext .w:hover { font-weight: 600; }"],
    ["hover margin via an ancestor state", ".verse:hover .w { margin-inline: 1px; }"],
    ["moving the word on hover", ".vtext .w:hover { transform: translateY(-1px); }"],
    ["putting the underline box in flow on hover", ".vtext .w:hover::after { position: static; }"],
    ["a wrapped word changing size", ".vtext .w[data-wrapped] { display: inline-block; }"],
  ])("rejects %s", (_name, rule) => {
    expect(lintCss(TOKENS + APP + "\n" + rule).length).toBeGreaterThan(0);
  });

  it("requires the red-letter rules to exist", () => {
    const without = APP.replace(".wj { color: var(--color-red-letter); }", "");
    expect(lintCss(TOKENS + without)).toContain("required rule missing: .wj { color: var(--color-red-letter) }");
  });

  it("requires the hover underline (E1, E2) to exist", () => {
    const without = APP.replace(/\.vtext \.w:hover:not\(:focus-visible\)::after \{[^}]*\}/, "");
    expect(without).not.toBe(APP);
    expect(lintCss(TOKENS + without)).toContain("required rule missing: .vtext .w:hover:not(:focus-visible)::after { opacity: 1 }");
  });

  it("requires the selected line (E3, E4) to exist", () => {
    const without = APP.replace(/\.vtext \.w\[aria-expanded="true"\]:not\(:focus-visible\)::before \{[^}]*\}/, "");
    expect(without).not.toBe(APP);
    expect(lintCss(TOKENS + without)).toContain(
      "required rule missing: .vtext .w[aria-expanded=true]:not(:focus-visible)::before { opacity: 1 }",
    );
  });

  it("allows content/opacity inside .vtext only for E1-E4 (and the gutter bar and jump flash, outside it)", () => {
    const glyphProps = ALLOWED.filter(([, p]) => p !== "color");
    expect(glyphProps).toEqual([
      [".verse[aria-current=true]::before", "content", '""'],
      [".verse.flashing::after", "content", '""'],
      [".verse.flashing::after", "opacity", "0"],
      [".vtext .w::after", "content", '""'],
      [".vtext .w::after", "opacity", "0"],
      [".vtext .w:hover:not(:focus-visible)::after", "opacity", "1"],
      [".vtext .w::before", "content", '""'],
      [".vtext .w::before", "opacity", "0"],
      [".vtext .w[aria-expanded=true]:not(:focus-visible)::before", "opacity", "1"],
    ]);
  });

  it("accepts the minified form of E1-E4 and the forced-colours lines", () => {
    const min = ".vtext .w:after{content:\"\";opacity:0;background-color:var(--color-word-underline)}" +
      ".vtext .w:hover:not(:focus-visible):after{opacity:1;transform:scaleX(1)}" +
      ".vtext .w:before{content:\"\";opacity:0;background-color:var(--color-word-underline)}" +
      ".vtext .w[aria-expanded=true]:not(:focus-visible):before{opacity:1;transform:scaleX(1)}" +
      ".vtext .w[data-wrapped]:hover:not(:focus-visible){-webkit-text-decoration-color:var(--color-word-underline);" +
      "-webkit-text-decoration-skip-ink:auto}" +
      "@media (forced-colors:active){.vtext .w:after{forced-color-adjust:none;background-color:canvastext}" +
      ".vtext .w:before{forced-color-adjust:none;background-color:highlight}}";
    const problems = lintCss(min);
    // only the (absent here) red-letter rules are reported; E1/E2 are found
    expect(problems.filter((p) => !p.startsWith("required rule missing"))).toEqual([]);
    expect(problems.filter((p) => p.includes(".vtext .w:") || p.includes(".vtext .w::"))).toEqual([]);
  });

  it("knows which selectors can reach verse text", () => {
    for (const s of [".wj", ".vtext .add", "span", "span.x", ":hover", "::selection", "*", "[data-x]", ".verse:hover", ".vtext .w::after"])
      expect(subjectMayBeVerseText(s), s).toBe(true);
    for (const s of [".btn", "kbd", ".opt-row kbd", "body", ".verse > .vnum", ".reading", ".pk-cell:hover"])
      expect(subjectMayBeVerseText(s), s).toBe(false);
  });
});
