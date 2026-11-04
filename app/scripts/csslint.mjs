// Colour-lock and motion lint (red-letters-and-highlights.md 3.4.5 part 1,
// reader.md 14.3). Static, on CSS text: the app's source CSS (tests) and the
// built CSS (after `vite build`). Returns a list of human-readable problems.
//
// Rules:
// 1. No rule may declare `color` or a glyph-painting property (3.4.2) on a
//    selector whose subject could match an element inside `.vtext`, unless the
//    (selector, property, value) is one of R1-R6, `.verse > .vnum`, the
//    current-verse bar's empty `content` or E1-E4 (3.4.3).
//    "Could match" = the subject compound is `*`, the type `span`, has no type
//    and no class (only pseudo-classes/elements and/or attributes), or names a
//    class from: vtext wj add nd nd-sc w pilcrow verse.
// 2. Any selector containing `.wj` may declare only `color` (3.1).
// 3. R1-R6, the verse-number rule and E1-E4 must be present (a deleted red
//    rule, hover underline or selected line fails).
// 4. Motion: @keyframes and transition / transition-property animate only
//    opacity and transform.
// 5. Hover underline and selected line (3.4.3, 3.5): E1-E4 (`content: ""`
//    and `opacity` on `.vtext .w::after` / `::before`) are the only
//    content/opacity allowed inside `.vtext`. A `.w::after` or `.w::before`
//    background is exactly var(--color-word-underline) (or, under forced
//    colours only, CanvasText for ::after and Highlight for ::before); a
//    text-decoration colour that can reach verse text is exactly that token.
// 6. No layout shift: hover/focus/active/data-wrapped/aria-expanded rules
//    whose subject can be verse text declare only paint properties.
// 7. The pen's paint layer (highlights-pen.md 10 item 7, red-letters 3.4.2):
//    rules whose subject is `.hl-ink` or `.hl-box` declare only position,
//    insets, size, transform, z-index, pointer-events, user-select,
//    border-radius, background-color, opacity, transition, outline(-*) and
//    custom properties, never color or content; `.verse[data-ranges]` declares
//    only `isolation: isolate`; E5 and the pen cursor rule declare only
//    `display: none` and `cursor: inherit`.

const VERSE_CLASSES = ["vtext", "wj", "add", "nd", "nd-sc", "w", "pilcrow", "verse"];
const BANNED = [
  "color", "-webkit-text-fill-color", "-webkit-text-stroke", "-webkit-text-stroke-width",
  "-webkit-text-stroke-color", "text-shadow", "opacity", "filter", "mix-blend-mode",
  "background-clip", "-webkit-background-clip", "text-emphasis", "text-emphasis-style",
  "text-emphasis-color", "-webkit-text-emphasis", "visibility", "text-transform", "content",
];
const MOTION_OK = new Set(["opacity", "transform"]);

// [selector, property, value] after normalisation
export const ALLOWED = [
  [".wj", "color", "var(--color-red-letter)"], // R1
  [":root[data-red-letters=off] .wj", "color", "inherit"], // R2
  [".vtext .w", "color", "inherit"], // R3
  [".vtext::selection", "color", "var(--color-text)"], // R4
  [".vtext ::selection", "color", "var(--color-text)"],
  [".vtext .wj::selection", "color", "var(--color-red-letter)"], // R5
  [".vtext .wj ::selection", "color", "var(--color-red-letter)"],
  [":root[data-red-letters=off] .vtext .wj::selection", "color", "var(--color-text)"], // R6
  [":root[data-red-letters=off] .vtext .wj ::selection", "color", "var(--color-text)"],
  [".verse>.vnum", "color", "var(--color-verse-number)"],
  [".verse[aria-current=true]::before", "content", '""'],
  // the jump flash (wave-2-layout.md 6.8): an empty overlay under the text, opacity animated
  [".verse.flashing::after", "content", '""'],
  [".verse.flashing::after", "opacity", "0"],
  // E1-E4 (3.4.3): the empty Strong's hover-underline and selected-line boxes; they hold no text
  [".vtext .w::after", "content", '""'], // E1
  [".vtext .w::after", "opacity", "0"], // E1
  [".vtext .w:hover:not(:focus-visible)::after", "opacity", "1"], // E2
  [".vtext .w::before", "content", '""'], // E3
  [".vtext .w::before", "opacity", "0"], // E3
  [".vtext .w[aria-expanded=true]:not(:focus-visible)::before", "opacity", "1"], // E4
];
const REQUIRED = ALLOWED.filter(([s, p]) => p === "color" || s.startsWith(".vtext .w"));

// The lines' colour (3.4.3 allow-list): any background on a `.w`'s ::after
// or ::before, and any text-decoration colour that can reach verse text, is
// exactly the token; under forced colours only, the ::after may use
// CanvasText and the ::before Highlight (3.5).
const UNDERLINE = "var(--color-word-underline)";
const W_AFTER = /\.w(?![\w-])[^ >+~]*::after$/;
const W_LINE = /\.w(?![\w-])[^ >+~]*::(after|before)$/;

// No layout shift (3.4.3): a rule that applies only in a hover / focus /
// active / data-wrapped / aria-expanded (selected) state and whose subject can
// be verse text may declare only paint properties. Transforms only on a
// pseudo-element; `display: none` only on the out-of-flow `.w` line boxes.
const STATE = /:(hover|focus|focus-visible|focus-within|active)(?![\w-])|\[data-wrapped|\[aria-expanded/;
const PAINT_ONLY = new Set([
  "color", "background-color", "outline", "outline-color", "outline-style", "outline-width",
  "outline-offset", "border-radius", "box-shadow", "cursor", "pointer-events", "forced-color-adjust",
  "opacity", "transition", "transition-property", "transition-duration", "transition-timing-function",
  "transition-delay", "text-decoration", "text-decoration-line", "text-decoration-color",
  "text-decoration-style", "text-decoration-thickness", "text-decoration-skip-ink",
  "text-underline-offset", "text-underline-position",
]);
const PSEUDO_PAINT = new Set(["transform", "transform-origin", "translate", "scale", "rotate"]);

const INK_OK = new Set([
  "position", "inset", "inset-inline", "inset-block", "inset-inline-start", "inset-inline-end", "inset-block-start",
  "inset-block-end", "left", "top", "right", "bottom", "width", "height", "transform", "z-index", "pointer-events",
  "user-select", "-webkit-user-select", "border-radius", "background-color", "opacity", "transition", "transition-property",
  "transition-duration", "transition-timing-function", "transition-delay", "outline", "outline-offset", "outline-style",
  "outline-width", "outline-color",
]);
const INK = /\.hl-(ink|box)(?![\w-])/;
// [selector, the only (property, value) it may declare]
const ONLY = [
  [".verse[data-ranges]", "isolation", "isolate"],
  [":root[data-pen] .vtext .w::after", "display", "none"], // E5
  [":root[data-pen] .vtext .w", "cursor", "inherit"],
];

export function normSelector(sel) {
  return sel
    .replace(/\s+/g, " ")
    // minifiers write legacy single-colon pseudo-elements
    .replace(/(^|[^:]):(before|after|first-letter|first-line)(?![\w-])/g, "$1::$2")
    .replace(/\s*([>+~])\s*/g, "$1")
    .replace(/\[\s*([^\]=\s]+)\s*=\s*(["']?)([^\]"']*)\2\s*\]/g, "[$1=$3]")
    .trim();
}
const normValue = (v) => v.replace(/\s+/g, " ").replace(/\s*!important\s*$/i, "").replace(/'/g, '"').trim();

function splitTop(s, sepChar) {
  const out = [];
  let depth = 0, quote = null, cur = "";
  for (const ch of s) {
    if (quote) { cur += ch; if (ch === quote) quote = null; continue; }
    if (ch === '"' || ch === "'") { quote = ch; cur += ch; continue; }
    if (ch === "(" || ch === "[") depth++;
    if (ch === ")" || ch === "]") depth--;
    if (ch === sepChar && depth === 0) { out.push(cur); cur = ""; continue; }
    cur += ch;
  }
  if (cur.trim()) out.push(cur);
  return out;
}

/** Parse CSS into { rules: [{selectors, decls}], keyframes: [{name, decls}] }. */
export function parseCss(css) {
  css = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const rules = [], keyframes = [];
  let i = 0;
  function readBlock() {
    // css[i] is just after '{'; returns body text up to the matching '}'
    let depth = 1, start = i, quote = null;
    for (; i < css.length; i++) {
      const ch = css[i];
      if (quote) { if (ch === quote && css[i - 1] !== "\\") quote = null; continue; }
      if (ch === '"' || ch === "'") { quote = ch; continue; }
      if (ch === "{") depth++;
      else if (ch === "}" && --depth === 0) { const body = css.slice(start, i); i++; return body; }
    }
    throw new Error("unbalanced braces in CSS");
  }
  const decls = (body) =>
    splitTop(body, ";").map((d) => {
      const k = d.indexOf(":");
      return k < 0 ? null : { prop: d.slice(0, k).trim().toLowerCase(), value: d.slice(k + 1).trim() };
    }).filter(Boolean);
  function parseList(text, inKeyframes, kfName, media = "") {
    const saved = css, savedI = i;
    css = text; i = 0;
    while (i < css.length) {
      const open = css.indexOf("{", i);
      if (open < 0) break;
      const prelude = css.slice(i, open).trim().replace(/^;+/, "").trim();
      i = open + 1;
      const body = readBlock();
      if (inKeyframes) { keyframes.push({ name: kfName, decls: decls(body) }); continue; }
      if (prelude.startsWith("@")) {
        const at = /^@([\w-]+)\s*(.*)$/s.exec(prelude);
        const name = at ? at[1].toLowerCase() : "";
        if (name.endsWith("keyframes")) parseList(body, true, at[2].trim());
        else if (name === "media") parseList(body, false, undefined, `${media} ${at[2]}`.trim());
        else if (["container", "supports", "layer", "scope", "document"].includes(name)) parseList(body, false, undefined, media);
        else rules.push({ selectors: [], decls: decls(body), at: name, media });
        continue;
      }
      rules.push({ selectors: splitTop(prelude, ",").map((s) => s.trim()).filter(Boolean), decls: decls(body), media });
    }
    css = saved; i = savedI;
  }
  parseList(css, false);
  return { rules, keyframes };
}

function subjectCompound(sel) {
  const parts = [];
  let depth = 0, cur = "";
  for (const ch of sel) {
    if (ch === "(" || ch === "[") depth++;
    if (ch === ")" || ch === "]") depth--;
    if (depth === 0 && (ch === " " || ch === ">" || ch === "+" || ch === "~")) { parts.push(cur); cur = ""; continue; }
    cur += ch;
  }
  parts.push(cur);
  return parts.filter((p) => p !== "").pop() ?? "";
}

export function subjectMayBeVerseText(sel) {
  const c = subjectCompound(normSelector(sel));
  if (c.startsWith("*")) return true;
  const classes = [...c.matchAll(/\.(-?[_a-zA-Z][\w-]*)/g)].map((m) => m[1]);
  if (classes.some((k) => VERSE_CLASSES.includes(k))) return true;
  // classes inside :is()/:where()/:not() count too
  if (/:(is|where|not|has)\(/.test(c) && VERSE_CLASSES.some((k) => new RegExp(`\\.${k}(?![\\w-])`).test(c))) return true;
  const withoutFns = c.replace(/\([^)]*\)/g, "");
  const typeMatch = /^([a-zA-Z][\w-]*)/.exec(withoutFns);
  if (typeMatch) return typeMatch[1].toLowerCase() === "span";
  const hasClassOrId = /[.#]/.test(withoutFns.replace(/\[[^\]]*\]/g, ""));
  return !hasClassOrId; // only pseudo-classes / pseudo-elements / attributes
}

const isBanned = (prop, value) =>
  BANNED.includes(prop) || prop.startsWith("-webkit-text-stroke") || prop.startsWith("text-emphasis") ||
  (prop === "background-clip" && /text/.test(value)) || (prop === "-webkit-background-clip" && /text/.test(value));

function transitionProps(value) {
  return splitTop(value, ",").map((item) => item.trim().split(/\s+/)[0].toLowerCase());
}

export function lintCss(cssText) {
  const problems = [];
  const { rules, keyframes } = parseCss(cssText);
  const seen = new Set();
  for (const rule of rules) {
    for (const rawSel of rule.selectors) {
      const sel = normSelector(rawSel);
      for (const { prop, value } of rule.decls) {
        const v = normValue(value);
        const allowed = ALLOWED.some(([s, p, val]) => s === sel && p === prop && val === v);
        if (allowed) seen.add(`${sel}|${prop}|${v}`);
        if (/\.wj(?![\w-])/.test(sel) && prop !== "color") {
          problems.push(`${rawSel} { ${prop}: ${value} }: only colour may target .wj (3.1)`);
        }
        const reaches = subjectMayBeVerseText(sel);
        if (isBanned(prop, v) && reaches && !allowed) {
          problems.push(`${rawSel} { ${prop}: ${value} }: may paint verse text (colour lock 3.4.2)`);
        }
        const forced = /forced-colors\s*:\s*active/.test(rule.media ?? "");
        const forcedOk = forced && v.toLowerCase() === (W_AFTER.test(sel) ? "canvastext" : "highlight");
        if (W_LINE.test(sel) && (prop === "background" || prop === "background-color") && v !== UNDERLINE && !forcedOk) {
          problems.push(`${rawSel} { ${prop}: ${value} }: the hover and selected lines are ${UNDERLINE} (3.4.3)`);
        }
        // minifiers add -webkit- copies of text-decoration longhands
        const base = prop.replace(/^-webkit-(?=text-decoration|text-underline)/, "");
        // a shorthand without a colour would draw in currentColor (text or red)
        const decoBad = base === "text-decoration-color" ? v !== UNDERLINE
          : base === "text-decoration" ? v !== "none" && !v.includes(UNDERLINE) : false;
        if (reaches && decoBad) {
          problems.push(`${rawSel} { ${prop}: ${value} }: decoration on verse text is ${UNDERLINE} (3.4.3)`);
        }
        if (INK.test(subjectCompound(sel)) && !prop.startsWith("--") && !INK_OK.has(prop)) {
          problems.push(`${rawSel} { ${prop}: ${value} }: the paint layer may not declare this (highlights-pen.md 10.7)`);
        }
        const only = ONLY.find(([s]) => s === sel);
        if (only && (prop !== only[1] || v !== only[2])) {
          problems.push(`${rawSel} { ${prop}: ${value} }: only ${only[1]}: ${only[2]} is allowed here`);
        }
        if (reaches && STATE.test(sel)) {
          const pseudo = /::[\w-]+$/.test(subjectCompound(sel));
          const ok = PAINT_ONLY.has(base) || (pseudo && PSEUDO_PAINT.has(base)) ||
            (base === "display" && v === "none" && W_LINE.test(sel));
          if (!ok) problems.push(`${rawSel} { ${prop}: ${value} }: may shift layout between rest and hover (3.4.3)`);
        }
      }
    }
    for (const { prop, value } of rule.decls) {
      if (prop === "transition" || prop === "transition-property") {
        for (const p of transitionProps(value)) {
          if (!MOTION_OK.has(p)) problems.push(`${rule.selectors.join(", ")} { ${prop}: ${value} }: animates ${p} (motion lint)`);
        }
      }
    }
  }
  for (const kf of keyframes) {
    for (const { prop } of kf.decls) {
      if (!MOTION_OK.has(prop)) problems.push(`@keyframes ${kf.name}: animates ${prop} (motion lint)`);
    }
  }
  for (const [s, p, v] of REQUIRED) {
    if (!seen.has(`${s}|${p}|${v}`)) problems.push(`required rule missing: ${s} { ${p}: ${v} }`);
  }
  return problems;
}
