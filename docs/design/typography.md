# Typography

Status: v0.2, 2026-10-20 (v0.2: the
font licence facts in section 6, Gentium v7 names).
Tokens referenced here are defined in [`tokens.css`](tokens.css).

Principle: type does all the work in Revelation. One reading face at a time, set
generously, with presentation marks (verse numbers, small caps, italics, red letters)
that are quiet enough to disappear while reading and clear enough to find when
looking. **Nothing here changes a character of the text** (PLAN.md 2.5). Every rule
below is CSS presentation on top of the exact stored string.

---

## 1. Reading column

| Property | Value | Notes |
|---|---|---|
| Measure | `max-width: var(--measure)` = **66ch** (reader can choose 60 / 66 / 72ch) | `ch` is measured in the reading font, so the column follows each font's width. Target is 60–72 characters per line; verify in M2 by counting real lines (section 6) |
| Font size | `calc(var(--read-size) * var(--read-scale))` | `--read-size` default 1.25rem (20px at 100% OS text size); `--read-scale` per font (section 4) |
| Line height | `var(--read-leading)` ≈ 1.6 (1.55–1.65 per font) | Unitless, so it follows size changes |
| Paragraph spacing | `0.9em` between source paragraphs; no first-line indent | Paragraph breaks come only from the source's paragraph marks |
| Alignment | `text-align: start`; **no justification**, **no hyphenation** (`hyphens: manual`) | Justification makes rivers at this measure, and auto-hyphenation inserts visible hyphens that aren't in the text. The renderer must never add characters |
| Side margins | Column centred in the reader pane, minimum `--space-6` (32px) gutter on each side, plus `--verse-number-hang` on the left for hanging numbers | |
| Top of chapter | Chapter heading, then `--space-6` before the first verse | Heading style: reader.md §3 |
| Kerning and ligatures | `font-kerning: normal; font-variant-ligatures: common-ligatures;` (browser defaults) | Ligatures are glyph substitutions only; copy/paste and the DOM keep the original characters. No discretionary or historical ligatures (`dlig`, `hlig`): they read as decoration |
| Figures | Reading font default (often old-style in Garamond-like faces) | Verse numbers use Inter lining figures instead (section 2) |
| Font synthesis | `font-synthesis: none` once the real files are bundled | Prevents faux bold/italic if a file is missing; a missing file should fail the font-loading test instead |
| Optical size | `font-optical-sizing: auto` | Literata, Source Serif 4 and Newsreader have an `opsz` axis and will pick their text cut automatically |

### Font size steps (Ctrl + / Ctrl −)

`--read-step-1` … `--read-step-10` = 16, 17, 18, 19, **20**, 22, 24, 26, 28, 32px at
100% OS text size (all in rem). Ctrl+0 returns to 20. The size **switches in one
frame and is never animated** (animating font-size animates layout). What keeps it
smooth is that nothing jumps away: before the change, the app records the verse at the
top of the viewport and its offset; after the change it restores that verse to the
same place. A small size indicator ("20 px") fades in for 1s at the window's bottom
centre (`--dur-2` fade).

### OS text scaling

All type is in `rem`. The root font size is `100%`, and the Rust core reads Windows'
**Text size** setting (`UISettings.TextScaleFactor`) and sets
`html { font-size: calc(100% * factor) }`. WebView2 already follows display scaling
(DPI). Whether it follows the separate Text size setting on its own is **to verify in
M2**. Either way, nothing should use `px` for type.

---

## 2. Verse numbers

Verse numbers are presentation (they're not in the verse text). They should be easy
to find and easy to ignore.

| Property | Value |
|---|---|
| Typeface | `var(--font-ui)` (Inter), weight 500, `font-variant-numeric: lining-nums tabular-nums` |
| Size | `var(--verse-number-size)` = 0.6em of the reading size (12px at the 20px default) |
| Colour | `var(--color-verse-number)`, never red, never a highlight |
| Raise | `position: relative; top: var(--verse-number-raise)` (−0.5em). **Not** `vertical-align: super`, which can enlarge the line box and make lines uneven |
| Gap after the number | `margin-inline-end: var(--verse-number-gap)` (0.18em). It's a margin, not a space character, so the text stays exact |
| Accessibility | The number element has `aria-hidden="true"`, and a visually hidden "Verse 16" precedes the verse text, so screen readers announce it once. The render test treats that label as presentation, like the number |
| Selection and copy | Copy uses the app's own handler (exact stored text plus reference, PLAN.md 2.5), so verse numbers never leak into copied text |
| Toggle off | The number element is removed from layout (`display: none`); the gap goes with it |

### Two layout modes

**Verse-per-line mode (default).** Each verse is its own block. The number **hangs**
in a gutter to the left of the column, right-aligned, so every verse's text starts
on the same edge:

```css
.verse { display: block; position: relative; margin-block-end: 0.35em; }
.verse > .vnum {
  position: absolute; inset-inline-start: calc(-1 * var(--verse-number-hang));
  width: calc(var(--verse-number-hang) - 0.5em); text-align: end;
  top: 0.55em; /* sits on the first line's x-height zone */
}
```

**Paragraph mode.** Verses flow inside the source's paragraphs. Mid-paragraph numbers
are inline and raised (as in the table above). The number of the **first verse of each
paragraph hangs** into the same gutter, so the paragraph's left edge stays clean:

```css
.para > .verse:first-child > .vnum {
  display: inline-block; width: var(--verse-number-hang);
  margin-inline-start: calc(-1 * var(--verse-number-hang));
  padding-inline-end: 0.5em; text-align: end;
}
```

Chapter 1 / verse 1: the chapter heading already says where you are, so verse 1's
number is shown like any other (no drop numeral). It's simpler and avoids enlarged
first letters, which would be decoration of the text.

---

## 3. Source-marked presentation

| Source markup | Presentation | CSS | When toggled off |
|---|---|---|---|
| Words of Jesus (`\wj`, `<q who="Jesus">`) | Red-letter colour only | `.wj { color: var(--color-red-letter) }` | `color: inherit`. See red-letters-and-highlights.md for the hard rule |
| Supplied words (`transChange type="added"`, USFM `\add`) | Real italic of the reading font | `.add { font-style: italic }` | `font-style: inherit` (the word stays, upright) |
| Divine name (`divineName`, USFM `\nd`) | Real small caps | see 3.1 | Not toggleable (it's how the edition prints the name) |

The three compose freely, because each touches one property: a supplied word inside
Jesus' words is red **and** italic; *LORD* inside Jesus' words is red **and** small
caps.

### 3.1 Small caps for LORD

The edition prints the divine name as a capital L followed by small capitals. How we
get there depends on how the pinned source **stores** the characters. The renderer must
not change them (PLAN.md 2.5).

| Source stores | CSS | Result |
|---|---|---|
| Mixed case `Lord` inside `divineName` (CrossWire OSIS KJV does this) | `.nd { font-variant-caps: small-caps; }` | L + ORD in small caps. DOM and copy still say `Lord` |
| Upper case `LORD` inside `divineName` | Wrap the stored characters after the first in a child span: `<span class="nd"><span>L</span><span class="nd-sc">ORD</span></span>` with `.nd-sc { font-variant-caps: all-small-caps; }` | Same look. The DOM text is still exactly `LORD`, so the render test passes |

Never use `text-transform` for this. Depending on the browser, copying may give the
transformed text, and the screen would show characters that aren't in the store.

**Real vs synthesised small caps.** `font-variant-caps` uses the font's `smcp`/`c2sc`
features when they exist. When they don't, Chromium **synthesises** small caps by
shrinking capitals, which looks thin and light. So each bundled file must be checked
for `smcp` (section 6, a developer check; licences are cleared). If a chosen
font lacks real small caps, the fallback is Chromium's synthesis
(`font-synthesis-small-caps: auto` for `.nd` only), noted in the Settings font list as
"small caps simulated" (Q7: keep such a font with simulated small caps, or drop it).

Small-caps tracking: `letter-spacing: 0.04em` on `.nd-sc` / `.nd` only. Small caps read
tight without it. This is spacing between glyphs, not a glyph change, and `.nd` is not
a red-letter span. It's still a presentation choice for the owner to confirm (Q3).

### 3.2 Italic for supplied words

Use the font's **true italic** file, which all eight families ship. The italic must
not shift the baseline or change size. Colour is unchanged; a supplied word in a red
passage keeps the red. No brackets or other marks are added: the KJV's printed
convention is italic, and adding characters would break Display = storage.

---

## 4. The eight reading fonts: optical matching

Goal: at the same `--read-size`, every font should *look* the same size and colour on
the page, so switching fonts in setup or Settings changes the voice, not the scale.
We match **x-height** (the size of lower-case letters, which drives perceived size)
about three-quarters of the way to Literata, rather than fully. Fully matching would
blow up the ascenders and capitals of small-x-height faces like EB Garamond.

`--read-scale = (x_Literata / x_font) ^ 0.75`, rounded, then adjusted by eye for
width. **The x-heights below are my estimates from the families' published
specimens, not measured values.** They must be measured from the actual bundled files
in M2 (section 6) before the scales are final.

| Font (`data-read-font`) | Est. x-height (em) | `--read-scale` | `--read-leading` | `--read-tracking` | Character | Notes |
|---|---|---|---|---|---|---|
| **Literata** (`literata`, default) | ≈ 0.48 | 1.00 | 1.60 | 0 | Warm, sturdy, built for long screen reading | Reference face. `opsz` axis |
| Source Serif 4 (`source-serif-4`) | ≈ 0.475 | 1.00 | 1.60 | 0 | Crisp, modern transitional | `opsz` axis |
| EB Garamond (`eb-garamond`) | ≈ 0.40 | 1.12 | 1.55 | 0.005em | Classical, bookish, small x-height | Tall ascenders need less extra leading after scaling. Thin hairlines on dark; check at 16–18px |
| Crimson Pro (`crimson-pro`) | ≈ 0.42 | 1.10 | 1.55 | 0.005em | Old-style, elegant | |
| Lora (`lora`) | ≈ 0.50 | 0.97 | 1.65 | 0 | Soft, calligraphic, large x-height | Large x-height wants more leading |
| Newsreader (`newsreader`) | ≈ 0.46 | 1.03 | 1.60 | 0 | Editorial, compact | `opsz` axis |
| Spectral (`spectral`) | ≈ 0.42 | 1.06 | 1.65 | 0 | Light, wide, elegant on screen | Wide, so a smaller scale than its x-height suggests. Its regular weight is light; check it on dark at small sizes |
| Gentium Book (`gentium-book`; was Gentium Book Plus before v7) | ≈ 0.46 | 1.03 | 1.60 | 0 | Scholarly, even, huge character set | The Book weight is the right one for body text (Gentium is lighter) |

Weights: body text at 400 in every font and every theme. No per-theme weight tricks.
They complicate testing and are not needed at these contrasts.

---

## 5. Hebrew and Greek

Original-language text appears in the Lexicon, in interlinear views later, and in
Hebrew/Greek Bibles (OSHB, Nestle 1904 and others). Pairing rules:

| | Hebrew | Greek |
|---|---|---|
| Family | `var(--font-hebrew)`: **Ezra SIL** for pointed text with cantillation; **Noto Serif Hebrew** for small UI sizes (lexicon headwords under 16px) via `--font-hebrew-ui` | `var(--font-greek)`: **Gentium** (polytonic; called Gentium Plus before v7) |
| Select by | `:lang(he), :lang(hbo)` | `:lang(grc), :lang(el)` |
| Size | `calc(1em * var(--scale-hebrew))` = **1.2×** the surrounding reading size. Hebrew letters sit within a smaller visual body, and points need room | `calc(1em * var(--scale-greek))` = **1.04×** |
| Line height | `var(--leading-hebrew)` = **2.0** for blocks with cantillation; 1.7 for unpointed or consonantal text | `var(--leading-greek)` = 1.6 |
| Direction | `direction: rtl; unicode-bidi: isolate;` on every Hebrew run, `text-align: start` | ltr |
| Spacing | **Never** `letter-spacing` or `word-spacing` on Hebrew (breaks mark positioning and joins) | None |
| Features | Defaults only. No `font-feature-settings` | Defaults only |
| Normalisation | None. Text is shown byte-for-byte as stored (PLAN.md 2.5); the font must position combining marks as given | Same |

Inline Hebrew inside an English lexicon line is isolated (`<bdi lang="hbo">` or
`unicode-bidi: isolate`), so punctuation around it stays in English order.

Pairing note: Gentium and Gentium Book are the same design family. With any reading
font, Gentium Greek sits comfortably beside Latin text at 1.04×. Gentium Book also
contains Greek, so if bundle size matters, Gentium Book could serve Greek too (Q8).

**Names.** SIL renamed the families in v7.000 (June 2025): Gentium Plus became
**Gentium**, Gentium Book Plus became **Gentium Book**. The pinned version is v7.000
(font licence review §4.8), so the design docs and `tokens.css` v0.2 use the new names.
PLAN.md still uses the old ones. Whatever the version, the picker shows
the family name that's actually in the shipped file.

---

## 6. Verification method for M2 (developer)

Once the real files are in `/assets/fonts`:

1. **x-height and width**: in a test page, for each font render `xxxxxxxxxx` and the
   string `abcdefghijklmnopqrstuvwxyz` at 100px with `--read-scale: 1`, and measure
   the bounding box height of `x` and the width of the alphabet. Set
   `--read-scale = (x_Literata / x_font) ^ 0.75`, then adjust by up to ±0.03 so the
   alphabet width lands within ±6% of Literata.
2. **Characters per line**: at 20px and 66ch, count characters on 20 full lines of a
   real chapter from `content.sqlite`. The mean must sit within 60–72; if not, adjust
   that font's measure.
3. **Small caps**: check each file for the `smcp` and `c2sc` features
   (`fontTools`: `ttx -t GSUB`, look for the feature tags). Record the results in the
   table below.
4. **Hebrew**: render a pointed, accented sample from OSHB in Ezra SIL and Noto Serif
   Hebrew in WebView2 and check mark stacking. Compare against the same text in a PDF
   from the source.

### Font facts (licences cleared; features still to check)

Licence, version, RFN and notice answers come from the font licence review
(2026-10-20); section numbers below refer to it. All 12 families are SIL OFL 1.1 and cleared; "with conditions"
means the standing conditions F1–F6 (§2) plus what's noted.

| Font | Licence and version | RFN | Real small caps (`smcp`)? | Italic file | Variable? | To check (developer, M2) |
|---|---|---|---|---|---|---|
| Literata | OFL 1.1, release 3.103 (§4.1) | none | I believe yes | Yes | Yes (`wght`, `opsz`) | `smcp` in the file we bundle |
| Source Serif 4 | OFL 1.1, 4.005 (§4.2); "Source" is an Adobe trademark | **Source** | Yes, in Adobe's upstream release | Yes | Yes (`wght`, `opsz`) | Adobe-built files only (Desktop zip, or Adobe's own WOFF2 zip); never a WOFF2 we convert |
| EB Garamond | OFL 1.1, Octavio Pardo's repo pinned at a commit (§4.3); keep both copyright lines | none | Yes, upstream | Yes | Yes (`wght`) | Google Fonts builds from the same repo, so the old "Google strips features" worry becomes: check `smcp` in the pinned upstream files |
| Crimson Pro | OFL 1.1, pinned commit (§4.4) | none | **Unsure** | Yes | Yes (`wght`) | Check `smcp`; may need simulation |
| Lora | OFL 1.1, v3.021 (§4.5) | **Lora** | **Probably not** | Yes | Yes (`wght`) | Check `smcp`; likely simulated |
| Newsreader | OFL 1.1, pinned commit (§4.6); registered trademark | none | **Unsure** | Yes | Yes (`wght`, `opsz`) | Check `smcp` |
| Spectral (+ Spectral SC) | OFL 1.1, pinned commit (§4.7); registered trademark; Spectral SC is covered by the same row | none | **Unsure** | Yes | No (static) | If `smcp` is missing, `.nd` may use Spectral SC's files (licence already cleared; list the extra files in the SOURCES.md row) |
| Gentium Book (was Gentium Book Plus) | OFL 1.1, v7.000 recommended (§4.8) | **Gentium, SIL** | Yes (SIL fonts support `smcp`) | Yes | No (static) | Confirm `smcp`; verify the PGP signature if practical |
| Gentium (was Gentium Plus) | OFL 1.1, v7.000 recommended (§4.8, §4.12) | **Gentium, SIL** | Yes | Yes | No | Greek `smcp`/`c2sc` per v7 notes |
| Ezra SIL | **Version 2.51 only** (2.0 was not open source); OFL (1.1 per Debian) plus an MIT notice for the Hebrew layout code (§4.11) | **Ezra, SIL** (per Debian) | n/a | n/a | No | Read the licence file in `EzraSIL-2.51.zip` and confirm OFL 1.1, the RFNs and the MIT notice; if any differ, stop and review it. Mark positioning in Chromium. Ezra SIL doesn't follow canonical order, so NFC/NFD-normalised text won't display correctly (no normalisation on display anyway, PLAN.md 2.5) |
| Noto Serif Hebrew | OFL 1.1, 2.004 (§4.10); "Noto" is a Google trademark | none | n/a | n/a | Yes (`wght`) | Coverage of all cantillation marks used by OSHB. Fetch the Serif release, not the repo's "latest" (Noto Sans Hebrew) |
| Inter | OFL 1.1, 4.1 (§4.9) | none | n/a | Yes | Yes | None expected |

**Files are bundled unmodified (condition F1, §1–2).** Subsetting, converting to WOFF2,
instancing a variable font, re-hinting or renaming makes a Modified Version, which
can't be shown to readers under a Reserved Font Name, and the picker shows font names.
So: ship the upstream TTF/OTF files byte for byte (or a WOFF2 the foundry itself
published) and load them directly with `@font-face`. WebView2 doesn't need WOFF2.
Leaving whole files (weights or styles we don't use) out is fine (F6); cutting glyphs
out of a file is not. The licence file goes beside each family and in the third-party
notices, and the About screen credits each family (F2, F3). Any modified file needs a
new licence review first.
