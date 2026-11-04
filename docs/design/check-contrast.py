#!/usr/bin/env python3
"""Revelation colour check: WCAG 2.x contrast and colour-vision simulation.

Reads docs/design/tokens.css (the single source of colour) and prints the
tables used in red-letters-and-highlights.md, as Markdown.

    python check-contrast.py            # from docs/design/
    python check-contrast.py --strict   # exit 1 if any text pair < 4.5:1,
                                        # any non-text pair < 3:1, or any
                                        # highlight separation below its floor
No arguments other than --strict: it always reads tokens.css beside itself.

No dependencies beyond the Python 3.8+ standard library.

Method (so a reviewer can re-derive any number by hand):
  1. hex -> sRGB channel c in [0,1]  (value / 255)
  2. linearise: c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4
     (WCAG 2.x prints 0.03928 as the threshold; for 8-bit values both give
     identical results because no 8-bit value falls between them)
  3. relative luminance L = 0.2126 R + 0.7152 G + 0.0722 B
  4. contrast = (L_lighter + 0.05) / (L_darker + 0.05)
Colour-vision deficiency: Machado, Oliveira & Fernandes (2009) matrices at
severity 1.0 applied in linear RGB, then distance in OKLab (Ottosson 2020),
reported x100. Rough reading: < 3 looks identical, 3-6 subtle side by side,
6-10 noticeable, > 10 clearly different.

Wave 2 (v0.4): body text and red letters on the search match colour, verse
numbers on the verse flash (text, 4.5:1); the match edge on the match and on
the panel, the current-verse bar and focus ring on the flash, the focus ring
on the match (3:1). The match must stay SEP_FLOOR from the panel and
RED_FLOOR from red letters, and the flash SEP_FLOOR from the page. The flash
against each tint is printed for information only (the gutter bar carries
the cue there). Aliases such as `--color-flash: var(--color-selection)` are
resolved per theme.

Wave 2 (v0.5): search results now show in the command-box dropdown
(--color-raised): red letters on it, and body, red and muted text on its
active row (--color-hover composited over raised, "raised-hover"), all
4.5:1; the match edge on raised and raised-hover and the active-row accent
bar, 3:1; the match SEP_FLOOR from raised (match on the active row: info).

Wave 2 (v0.6, highlights-pen.md 8.4): the eraser preview shows the part of
a range mark being erased at --hl-erase-opacity (read from tokens.css)
over whatever is beneath it: the page or another tint (a whole-verse tint
under a range mark). Each of the 5 tints is blended over the page and over
the other four tints in sRGB, rounded to 8 bits as browsers paint (25
blends per theme), and body text, red letters and verse numbers must keep
4.5:1, the hover underline and focus ring 3:1, on every blend. An sRGB
blend of two hues can be darker than either, so this is computed, never
assumed. If a tint change ever fails here, raise --hl-erase-opacity.

Wave 2 (v0.7, bookmarks.md 9): the automatic-bookmark ribbon,
--color-bookmark, a filled glyph (non-text, 3:1) on the picker and the
command-box dropdown (raised), on their hover/active row (raised-hover)
and on a pressed cell (raised-pressed: --color-pressed over raised, as
raised-hover). It must stay RED_FLOOR from the red-letter colour and
SEP_FLOOR from the accent (focus, links, current book), the status
colour (danger) and the muted text beside it, in every vision type.

Highlight matrix (v0.3): for every theme and each of the five tints, the
strict run checks body text, red letters and verse numbers (text, 4.5:1) and
the Strong's hover underline and focus ring (non-text, 3:1) ON that tint.
It also fails if two tints, or a tint and the page, are closer than
SEP_FLOOR in any of the four vision types, or a tint is closer than
RED_FLOOR to the red-letter colour.
"""
import math
import pathlib
import re
import sys

TOKENS = pathlib.Path(__file__).with_name("tokens.css")

THEME_SELECTORS = {
    "dark": r':root,\s*:root\[data-theme="dark"\]\s*\{',
    "light": r':root\[data-theme="light"\]\s*\{',
    "sepia": r':root\[data-theme="sepia"\]\s*\{',
}
HL = ["gold", "green", "blue", "rose", "violet"]
SEP_FLOOR = 5.0   # OKLab x100: tint/tint and tint/page, every vision type
RED_FLOOR = 10.0  # OKLab x100: tint/red-letter colour, every vision type
# Wave 2 (v0.4, wave-2-layout.md 10): search match, its edge, the verse flash.
W2_TOKENS = ["match", "match-edge", "flash"]
W2_TEXT = [("text", "match"), ("red-letter", "match"), ("verse-number", "flash")]
W2_NONTEXT = [("match-edge", "match"), ("match-edge", "surface"),
              ("current-verse", "flash"), ("focus", "flash"), ("focus", "match")]
W2_SEP = [("match", "surface", SEP_FLOOR), ("match", "red-letter", RED_FLOOR),
          ("flash", "bg", SEP_FLOOR)]
# v0.5 (wave-2-layout.md 6.4, 10): search results moved into the command-box
# dropdown, which is --color-raised. Its active row is --color-hover (a
# color-mix of the text colour over transparent) composited over raised,
# plus the accent bar: "raised-hover" is computed per theme in
# load_themes(). The selected Strong's line reuses --color-word-underline,
# so it adds no pair. The dark --color-match moved to #45340A (v0.5) to
# keep SEP_FLOOR from raised; match on the active row is printed for info.
W5_TEXT = [("red-letter", "raised"), ("text", "raised-hover"),
           ("red-letter", "raised-hover"), ("text-muted", "raised-hover")]
W5_NONTEXT = [("match-edge", "raised"), ("match-edge", "raised-hover"), ("accent", "raised-hover")]
W5_SEP = [("match", "raised", SEP_FLOOR)]
# v0.6 (highlights-pen.md 8.4): eraser preview blends; pairs per blend.
ERASE_TEXT = ["text", "red-letter", "verse-number"]
ERASE_NONTEXT = ["word-underline", "focus"]
# v0.7 (bookmarks.md 9): the bookmark ribbon glyph.
W7_NONTEXT = [("bookmark", "raised"), ("bookmark", "raised-hover"), ("bookmark", "raised-pressed")]
W7_SEP = [("bookmark", "red-letter", RED_FLOOR), ("bookmark", "accent", SEP_FLOOR),
          ("bookmark", "danger", SEP_FLOOR), ("bookmark", "text-muted", SEP_FLOOR)]


def load_themes():
    css = TOKENS.read_text(encoding="utf-8")
    themes = {}
    for name, sel in THEME_SELECTORS.items():
        # Every block for this theme, in file order: the main block and any
        # appended version block (v0.4 wave 2) with the same selector.
        found = list(re.finditer(sel, css))
        if not found:
            sys.exit(f"theme block not found: {name}")
        base = dict(themes.get("dark", {}))
        for m in found:
            block = css[m.end(): css.index("}", m.end())]
            base.update(re.findall(r"--color-([a-z0-9-]+):\s*(#[0-9A-Fa-f]{6})\s*;", block))
        themes[name] = base
    # Plain colour aliases, `--color-a: var(--color-b);` (v0.4): they resolve
    # per theme because their targets do. color-mix() tokens are not aliases.
    aliases = re.findall(r"--color-([a-z0-9-]+):\s*var\(--color-([a-z0-9-]+)\)\s*;", css)
    for t in themes.values():
        for a, b in aliases:
            if a not in t and b in t:
                t[a] = t[b]
    # v0.5: --color-hover = color-mix(in srgb, var(--color-text) N%,
    # transparent), composited over --color-raised (sRGB, as browsers paint).
    for name, sel in THEME_SELECTORS.items():
        m = re.search(sel, css)
        block = css[m.end(): css.index("}", m.end())]
        t = themes[name]
        # v0.7 adds --color-pressed (same form) for a pressed picker cell
        for tok, key in (("hover", "raised-hover"), ("pressed", "raised-pressed")):
            pm = re.search(r"--color-" + tok + r":\s*color-mix\(in srgb,\s*var\(--color-text\)\s*([0-9.]+)%", block)
            if not pm:
                sys.exit(f"{name}: --color-{tok} color-mix not found")
            a = float(pm.group(1)) / 100
            mix = [round((1 - a) * r * 255 + a * x * 255) for r, x in zip(srgb(t["raised"]), srgb(t["text"]))]
            t[key] = "#" + "".join(f"{v:02X}" for v in mix)
    return themes


def erase_opacity():
    """--hl-erase-opacity from tokens.css (v0.6), a plain number in (0, 1]."""
    css = TOKENS.read_text(encoding="utf-8")
    m = re.search(r"--hl-erase-opacity:\s*([0-9.]+)\s*;", css)
    if not m:
        sys.exit("--hl-erase-opacity (v0.6) missing from tokens.css")
    a = float(m.group(1))
    if not 0 < a <= 1:
        sys.exit(f"--hl-erase-opacity out of range: {a}")
    return a


def blend(top, under, a):
    """`top` at opacity a over `under`, sRGB, rounded to 8 bits."""
    mix = [round((1 - a) * u * 255 + a * x * 255) for u, x in zip(srgb(under), srgb(top))]
    return "#" + "".join(f"{v:02X}" for v in mix)


def srgb(h):
    h = h.lstrip("#")
    return [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]


def linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lin_rgb(h):
    return [linear(c) for c in srgb(h)]


def lum_of(rgb):
    r, g, b = rgb
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def lum(h):
    return lum_of(lin_rgb(h))


def contrast_l(la, lb):
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def contrast(a, b):
    return contrast_l(lum(a), lum(b))


MACHADO = {
    "protan": [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
    "deutan": [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.011820, 0.042940, 0.968881]],
    "tritan": [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.303900]],
}
VISIONS = ["normal", "protan", "deutan", "tritan"]


def simulate(h, kind):
    rgb = lin_rgb(h)
    if kind == "normal":
        return rgb
    m = MACHADO[kind]
    return [min(1.0, max(0.0, sum(m[i][j] * rgb[j] for j in range(3)))) for i in range(3)]


def oklab(rgb):
    r, g, b = rgb
    l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
    m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
    s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
    l, m, s = (math.copysign(abs(x) ** (1 / 3), x) for x in (l, m, s))
    return (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s)


def delta(a, b, kind):
    pa, pb = oklab(simulate(a, kind)), oklab(simulate(b, kind))
    return 100 * math.sqrt(sum((x - y) ** 2 for x, y in zip(pa, pb)))


def main():
    strict = "--strict" in sys.argv
    themes = load_themes()
    erase_a = erase_opacity()
    failures = []
    for name, t in themes.items():
        missing = [k for k in W2_TOKENS + ["bookmark"] if k not in t]
        if missing:
            sys.exit(f"{name}: wave-2 (v0.4) colour tokens missing from tokens.css: {missing}")
        print(f"\n## Theme: {name}\n")
        print("### Relative luminance (working)\n")
        print("| Token | Hex | R lin | G lin | B lin | L |")
        print("|---|---|---|---|---|---|")
        shown = ["bg", "surface", "raised", "text", "text-muted", "verse-number", "link",
                 "red-letter", "word-underline", "selection", "on-accent", "accent", "danger", "success", "match", "bookmark"] + [f"hl-{h}" for h in HL]
        for k in shown:
            r, g, b = lin_rgb(t[k])
            print(f"| {k} | `{t[k]}` | {r:.4f} | {g:.4f} | {b:.4f} | {lum(t[k]):.4f} |")

        text_pairs = [
            ("text", "bg"), ("text", "surface"), ("text", "raised"),
            ("red-letter", "bg"), ("verse-number", "bg"), ("link", "bg"),
            ("text-muted", "bg"), ("text-muted", "surface"), ("text-muted", "raised"),
            ("link", "surface"), ("link", "raised"), ("on-accent", "accent"),
            ("danger", "bg"), ("danger", "surface"), ("danger", "raised"),
            ("success", "bg"), ("success", "surface"),
            ("text", "selection"), ("red-letter", "selection"),
        ]
        for h in HL:
            text_pairs += [("text", f"hl-{h}"), ("red-letter", f"hl-{h}"), ("verse-number", f"hl-{h}")]
        text_pairs += W2_TEXT + W5_TEXT
        print("\n### Text contrast (WCAG 1.4.3, needs 4.5:1)\n")
        print("| Foreground | Background | L fg | L bg | Ratio | AA 4.5 | AAA 7 |")
        print("|---|---|---|---|---|---|---|")
        for f, b in text_pairs:
            r = contrast(t[f], t[b])
            ok = r >= 4.5
            if not ok:
                failures.append((name, f, b, r))
            print(f"| {f} `{t[f]}` | {b} `{t[b]}` | {lum(t[f]):.4f} | {lum(t[b]):.4f} | "
                  f"{r:.2f} | {'pass' if ok else 'FAIL'} | {'pass' if r >= 7 else 'no'} |")

        print("\n### Non-text contrast (WCAG 1.4.11, needs 3:1)\n")
        print("| Element | Against | Ratio | 3:1 |")
        print("|---|---|---|---|")
        nontext = [("focus", "bg"), ("focus", "surface"), ("focus", "raised"),
                   ("border-strong", "bg"), ("border-strong", "surface"), ("border-strong", "raised"),
                   ("current-verse", "bg")]
        nontext += [(f"hl-{h}-swatch", "surface") for h in HL] + [(f"hl-{h}-swatch", "raised") for h in HL]
        # Strong's hover underline and focus ring on a word inside a
        # highlighted verse (red-letters-and-highlights.md 3.4.3)
        nontext += [("word-underline", "bg"), ("word-underline", "selection")]
        nontext += [("word-underline", f"hl-{h}") for h in HL] + [("focus", f"hl-{h}") for h in HL]
        nontext += W2_NONTEXT + W5_NONTEXT + W7_NONTEXT
        for f, b in nontext:
            r = contrast(t[f], t[b])
            ok = r >= 3
            if not ok:
                failures.append((name, f, b, r))
            print(f"| {f} `{t[f]}` | {b} `{t[b]}` | {r:.2f} | {'pass' if ok else 'FAIL'} |")

        print("\n### Colour-vision separation (OKLab distance x100)\n")
        print("| Pair | normal | protan | deutan | tritan | lum. ratio |")
        print("|---|---|---|---|---|---|")
        pairs = [("red-letter", "text"), ("red-letter", "link"), ("link", "text"),
                 ("red-letter", "verse-number"), ("red-letter", "danger")]
        pairs += [(f"hl-{h}", "bg") for h in HL]
        pairs += [(f"hl-{h}", "red-letter") for h in HL]
        for i, a in enumerate(HL):
            for b in HL[i + 1:]:
                pairs.append((f"hl-{a}", f"hl-{b}"))
        for a, b in pairs:
            ds = [delta(t[a], t[b], k) for k in VISIONS]
            if a.startswith("hl-"):
                floor = RED_FLOOR if b == "red-letter" else SEP_FLOOR
                if min(ds) < floor:
                    failures.append((name, a, b + " (separation)", min(ds)))
            print(f"| {a} / {b} | {' | '.join(f'{d:.1f}' for d in ds)} | {contrast(t[a], t[b]):.2f} |")
        for a, b, floor in W2_SEP + W5_SEP + W7_SEP:
            ds = [delta(t[a], t[b], k) for k in VISIONS]
            if min(ds) < floor:
                failures.append((name, a, b + " (separation)", min(ds)))
            print(f"| {a} / {b} (floor {floor:.0f}) | {' | '.join(f'{d:.1f}' for d in ds)} | {contrast(t[a], t[b]):.2f} |")
        ds = [delta(t["match"], t["raised-hover"], k) for k in VISIONS]
        print(f"| match / raised-hover (info) | {' | '.join(f'{d:.1f}' for d in ds)} | {contrast(t['match'], t['raised-hover']):.2f} |")
        for h in HL:  # informational, no floor
            ds = [delta(t["flash"], t[f"hl-{h}"], k) for k in VISIONS]
            print(f"| flash / hl-{h} (info) | {' | '.join(f'{d:.1f}' for d in ds)} | {contrast(t['flash'], t[f'hl-{h}']):.2f} |")

        print("\n### Highlight matrix: lowest value per tint\n")
        print("Text needs 4.5:1, underline and focus 3:1. Separation is the smallest OKLab x100 "
              f"distance to the page and the other four tints over all four vision types (floor {SEP_FLOOR:.1f}).\n")
        print("| Tint | Hex | Body text | Red letters | Verse number | Hover underline | Focus ring | Separation |")
        print("|---|---|---|---|---|---|---|---|")
        for h in HL:
            bgc = t[f"hl-{h}"]
            others = [t["bg"]] + [t[f"hl-{o}"] for o in HL if o != h]
            sep = min(delta(bgc, o, k) for o in others for k in VISIONS)
            cells = " | ".join(f"{contrast(t[f], bgc):.2f}" for f in
                               ("text", "red-letter", "verse-number", "word-underline", "focus"))
            print(f"| {h} | `{bgc}` | {cells} | {sep:.1f} |")

        print(f"\n### Eraser preview: each tint at {erase_a:.0%} over the page or another tint\n")
        print("Lowest of body text, red letters and verse number on the blend (needs 4.5:1); "
              "the hover underline and focus ring are checked too (3:1). Rows: the tint "
              "being erased. Columns: what lies beneath it.\n")
        unders = ["bg"] + [f"hl-{o}" for o in HL]
        print("| Erasing | " + " | ".join(unders) + " |")
        print("|---" * (len(unders) + 1) + "|")
        low = None
        for h in HL:
            cells = []
            for u in unders:
                if u == f"hl-{h}":
                    cells.append("-")  # a tint over itself is the tint (matrix above)
                    continue
                mix = blend(t[f"hl-{h}"], t[u], erase_a)
                rs = []
                for f in ERASE_TEXT:
                    r = contrast(t[f], mix)
                    rs.append(r)
                    if r < 4.5:
                        failures.append((name, f, f"hl-{h} {erase_a:.0%} over {u} {mix}", r))
                    if low is None or r < low[0]:
                        low = (r, f, h, u, mix)
                for f in ERASE_NONTEXT:
                    r = contrast(t[f], mix)
                    if r < 3:
                        failures.append((name, f, f"hl-{h} {erase_a:.0%} over {u} {mix}", r))
                cells.append(f"{min(rs):.2f}")
            print(f"| {h} | " + " | ".join(cells) + " |")
        r, f, h, u, mix = low
        print(f"\nLowest text pair: {f} on hl-{h} at {erase_a:.0%} over {u} (`{mix}`) = **{r:.2f}**.")

    if failures:
        print("\nFAILURES:")
        for f in failures:
            print("  {}: {} on {} = {:.2f}".format(*f))
    if strict and failures:
        sys.exit(1)


if __name__ == "__main__":
    main()
