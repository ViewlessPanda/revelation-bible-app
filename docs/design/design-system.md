# Revelation design system

Status: v0.2, 2026-10-20 (Switch
and Popover components, the icon and branding rules).
Implements PLAN.md Sections 1, 4 and 6. The single source of values is
[`tokens.css`](tokens.css). This document explains them. When the two disagree,
`tokens.css` wins and this file is the bug.

Companion specs: [typography.md](typography.md) ·
red-letters-and-highlights.md ·
first-run.md · reader.md. Visual check: open
first-run.html or [reader.html](reader.html) in any browser (a full
type and colour specimen page is planned for a later pass).

**The feel in one line:** a quiet, warm, dark room with a good lamp. The text is the
brightest thing on screen; the interface steps back until it's needed.

---

## 1. Principles

1. **Text first.** Scripture is the highest-contrast element on every screen. UI
   chrome uses muted text, hairline borders and no ornament.
2. **One accent.** Slate blue (`--color-accent`) marks interaction: focus, the
   primary button, links, the current verse. Nothing else is coloured, except the
   red letters and the reader's own highlights, which belong to the text.
3. **Red is reserved.** Only words-of-Jesus spans use a red hue. Errors use ochre with
   an icon and words.
4. **Calm motion.** Short, soft and functional. Transform and opacity only. Every
   animation has a reduced-motion version.
5. **Keyboard-first and accessible.** WCAG 2.2 AA contrast as a floor (body text is
   AAA), visible focus everywhere, rem-based sizes that follow OS text scaling.
6. **Restraint.** If an element doesn't help reading, writing or finding, it goes.

---

## 2. Colour

### 2.1 Structure

There are no primitive palette names in components. Components use **semantic
tokens only** (`--color-bg`, `--color-text`, `--color-accent`, ...). Each theme block in
`tokens.css` defines the same set of names:

```
:root, [data-theme="dark"]   → dark (default, also when the attribute is absent)
[data-theme="light"]         → light (first draft)
[data-theme="sepia"]         → sepia (first draft)
```

Adding a theme means adding one block with the same names and running
`check-contrast.py`. No component changes.

### 2.2 Tokens

| Token | Dark | Light | Sepia | Use |
|---|---|---|---|---|
| `--color-bg` | `#1C1B19` | `#F7F4EE` | `#F2E8D5` | Page, reading column |
| `--color-surface` | `#232220` | `#EFEBE3` | `#E9DEC8` | Side panel, title bar, setup list |
| `--color-raised` | `#2B2A27` | `#FBF9F5` | `#F6EEDF` | Cards, menus, tooltips, popovers |
| `--color-border` | `#3A3834` | `#DCD5C9` | `#D6C8AE` | Decorative hairlines and dividers |
| `--color-border-strong` | `#7A746B` | `#8C857A` | `#8A7B66` | Input and control outlines (≥ 3:1) |
| `--color-text` | `#E6E0D4` | `#2A2723` | `#2C251D` | Scripture, body text, primary UI text |
| `--color-text-muted` | `#ABA59A` | `#5B564E` | `#5E5244` | Secondary text, captions, hints |
| `--color-text-disabled` | `#6E6A62` | `#A39D93` | `#A6987F` | Disabled controls only |
| `--color-verse-number` | `#ABA59A` | `#5B564E` | `#5E5244` | Verse numbers |
| `--color-accent` | `#8DB2DE` | `#285A93` | `#285780` | Primary button, selected state |
| `--color-accent-hover` | `#A5C3E6` | `#1F4A7B` | `#1E4666` | Primary button hover |
| `--color-on-accent` | `#1C1B19` | `#F7F4EE` | `#F2E8D5` | Text on accent fill |
| `--color-link` | `#8DB2DE` | `#285A93` | `#285780` | Links (always underlined) |
| `--color-focus` | `#8DB2DE` | `#285A93` | `#285780` | Focus ring |
| `--color-current-verse` | `#8DB2DE` | `#285A93` | `#285780` | 2px gutter bar on the current verse |
| `--color-selection` | `#2C3A4E` | `#C9D9EE` | `#CBD3D6` | `::selection` |
| `--color-red-letter` | `#EF8A7B` | `#A1251C` | `#982A1E` | Words of Jesus only |
| `--color-word-underline` | `#ABA59A` | `#5B564E` | `#5E5244` | Strong's word hover underline (v0.3, red-letters 3.4.3) |
| `--color-hl-{gold,green,blue,rose,violet}` | see red-letters-and-highlights.md | | | Whole-verse highlight tints (v0.3 set) |
| `--color-danger` | `#E2A65A` | `#8A5300` | `#844F00` | Errors and warnings (with icon and words) |
| `--color-success` | `#9CC69B` | `#2F6B3A` | `#2E6435` | Verified / installed confirmations |
| `--color-hover`, `--color-pressed` | text at 6% / 10% (dark), 5% / 9% (paper) | | | Overlays for hover and press on any surface |

The text colour is a warm off-white (`#E6E0D4`, 13.1:1). That's comfortably AAA
without the glare of pure white on charcoal, which causes halation for many readers,
especially those with astigmatism.

### 2.3 Accessibility notes

- Contrast for every pair is computed in
  red-letters-and-highlights.md §2 and its appendix.
- **Windows contrast themes** (`@media (forced-colors: active)`): don't fight them.
  Use system colours (`Canvas`, `CanvasText`, `LinkText`, `Highlight`), keep borders on
  all controls (they become visible), and draw focus rings with `outline` (it survives
  forced colours). Red letters follow the system text colour (Q6); highlights become a
  2px dashed `CanvasText` outline and the Strong's hover underline `CanvasText`
  (red-letters 3.5).

---

## 3. Type

Reading type is specified in [typography.md](typography.md). UI type is **Inter**, at
the sizes below. All sizes are in rem.

| Token | rem | px @100% | Weight | Leading | Use |
|---|---|---|---|---|---|
| `--text-ui-xs` | 0.75 | 12 | 500 | 1.45 | Keyboard hints, badges, counts |
| `--text-ui-sm` | 0.8125 | 13 | 400 | 1.45 | Secondary lines in lists, captions, attributions |
| `--text-ui-md` | 0.875 | 14 | 400 / 500 | 1.45 | Default UI text, buttons (500), tabs (500), list rows |
| `--text-ui-lg` | 1 | 16 | 600 | 1.25 | Panel and section headings, dialog body |
| `--text-ui-xl` | 1.25 | 20 | 600 | 1.25 | Dialog and window titles |
| `--text-ui-2xl` | 1.75 | 28 | 600 | 1.25 | First-run step titles (set in the reading font, see first-run.md) |

Inter settings: `font-feature-settings: "cv11", "ss01"` is **not** used (keep the
defaults; fewer surprises). Use `font-variant-numeric: tabular-nums` in counts,
tables and anything that updates live. All-caps labels are avoided; when unavoidable
(tiny section labels), use `--tracking-ui-caps` (0.06em).

---

## 4. Space, shape and depth

**Spacing**: a 4px grid, in rem so it scales with text.

| Token | `--space-0-5` | `-1` | `-2` | `-3` | `-4` | `-5` | `-6` | `-7` | `-8` |
|---|---|---|---|---|---|---|---|---|---|
| px @100% | 2 | 4 | 8 | 12 | 16 | 24 | 32 | 48 | 64 |

Rhythm: 8 between related items, 16 between groups, 24–32 between sections, 48+
around a whole screen's content.

**Radius** (px, so shapes don't balloon with text size): `--radius-sm` 4 (inputs,
tint corners) · `--radius-md` 6 (buttons, list rows) · `--radius-lg` 10 (cards,
popovers, tooltips) · `--radius-xl` 14 (dialogs, the Add translation window) ·
`--radius-full` (pills, swatches).

**Controls**: heights `--control-h-sm` 28 · `--control-h-md` 36 (default) ·
`--control-h-lg` 44 (primary actions in setup and dialogs). Horizontal padding 12 (sm)
or 16 (md/lg).

**Elevation.** On dark, depth comes mainly from surface lightness (`bg` → `surface` →
`raised`). Shadows only separate overlapping layers.

| Level | Surface | Shadow | Used by |
|---|---|---|---|
| 0 | `--color-bg` | none | Reading column |
| 1 | `--color-surface` | none, 1px `--color-border` edge | Side panel, title bar |
| 2 | `--color-raised` | `--shadow-2` | Menus, popovers, the command box results |
| 3 | `--color-raised` | `--shadow-3` | Tooltips, dialogs (with `--color-scrim` behind) |

---

## 5. Focus

```css
:focus { outline: none; }
:focus-visible {
  outline: var(--focus-ring);           /* 2px solid --color-focus */
  outline-offset: var(--focus-offset);  /* 2px */
}
```

- Only `:focus-visible`: keyboard users always see it, mouse clicks don't flash it.
- Ring contrast is ≥ 5.69:1 against every surface in every theme (AA needs 3:1).
- Inside lists where an outer ring would be clipped (scroll containers), use
  `outline-offset: -2px` so the ring is drawn inside the row.
- Focus never moves without the reader acting, except in two cases: on a step change in
  first-run setup, and when a dialog opens (focus goes to its first control and returns
  to the trigger when it closes).

---

## 6. Motion and feel

**Rules.** Only `transform` and `opacity` animate. Nothing animates width, height,
top, left, margin, padding or font-size. Entrances use `--ease-out`; exits are shorter
and use `--ease-in`. Animations never block input: a new action interrupts the running
one and starts from the current visual state.

| Token | Value | Use |
|---|---|---|
| `--dur-1` | 80ms | Caret glide, press feedback |
| `--dur-2` | 120ms | Hover, fades, chapter cross-fade, tooltip fade |
| `--dur-3` | 180ms | Tab content slide, tab indicator, reading-column FLIP |
| `--dur-4` | 240ms | Side panel open, first-run step entrance |
| `--dur-exit` | 120ms | All exits |
| `--ease-out` | `cubic-bezier(0.22, 1, 0.36, 1)` | Entrances, caret |
| `--ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | Exits |
| `--ease-in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | Things that move and stay (tab indicator) |
| `--ease-blink` | `cubic-bezier(0.45, 0, 0.55, 1)` | Caret blink |
| `--ease-linear` | `linear` | An opacity fade that rides on a scale grow (Strong's hover underline) |

### 6.1 Smooth caret (notes editor, command box)

- Native caret hidden (`caret-color: transparent`). A 2px-wide, `1.15em`-tall bar in
  `--color-text`, `border-radius: 1px`, absolutely positioned in an overlay layer.
- On every selection change, move it with
  `transform: translate(x, y)` over `--dur-1` (80ms) `--ease-out`. Across a line
  change it glides too (no special case). On large jumps (> 3 lines, or a click), it
  snaps with no glide, so it never sweeps across the page.
- Blink: solid for `--caret-hold` (500ms) after any keystroke or move, then a soft
  opacity cycle over `--caret-blink-period` (1060ms: 1 → 0 → 1 with `--ease-blink`,
  holding near full opacity for most of the cycle). Stop blinking after 30s idle
  (the caret stays solid), which saves power and is calmer.
- Selection (non-collapsed): the custom caret hides; the selection is drawn by the
  browser's `::selection` with `--color-selection`.
- Typing latency target is one frame (PLAN.md §8): the caret moves on the same frame as
  the text and is driven by `requestAnimationFrame` only for the blink.
- **Reduced motion:** `--caret-glide: 0`, so the caret jumps to its position. It keeps a
  plain on/off blink at the same period, or none if Windows' "Show text cursor blink"
  is off (to verify whether WebView2 exposes it; if not, follow the app setting).

### 6.2 Chapter change

- Target: new chapter on screen in < 50ms (PLAN.md §8). The animation is added on top
  and never delays it.
- The old chapter is removed at once. The new chapter's container fades in from
  `opacity: 0` and `translateY(var(--move-sm))` (4px) to rest over `--dur-2`
  (120ms) `--ease-out`. Same colour background, so it reads as a soft dissolve, not
  a flash.
- Scroll position resets to the top (or to the target verse) **before** the fade, so
  nothing scrolls visibly.
- **Reduced motion:** opacity fade only, 120ms, no movement.

### 6.3 Side panel and tabs

- **Panel open/close** (toggle with a shortcut, or drag the splitter): the panel
  slides in with `translateX(16px) → 0` plus opacity 0 → 1 over `--dur-4` (240ms). The
  panel's grid column takes its width in one frame. To stop the reading column jumping
  sideways, use FLIP: measure the column's position before and after, apply the
  inverse `translateX`, then transition it to 0 over `--dur-3` (180ms). Close: panel
  fades out over `--dur-exit` (120ms), then the column FLIPs back.
- **Splitter drag**: live, no animation (the reader is in control).
- **Tabs** (Commentary · Dictionary · Lexicon · Notes): the 2px active indicator under
  the tab label moves with `transform: translateX() scaleX()` over `--dur-3`
  `--ease-in-out`. The content cross-fades. The new content enters from
  `translateX(±8px)` in the direction of travel (right when moving to a tab on the
  right) with opacity 0 → 1 over `--dur-3`. The old content is removed at once.
- **Reduced motion:** the indicator jumps; content fades in over 120ms with no slide;
  the panel fades with no slide or FLIP.

### 6.4 Tooltips (Strong's words, icon buttons, highlights, range highlights)

- Show after `--delay-tooltip` (400ms) of hover intent, or at once on keyboard focus.
- If another tooltip closed less than `--delay-tooltip-skip` (300ms) ago, show at once
  (so moving along a line of words feels fluid, not sticky).
- Fade in over `--dur-2` (120ms) with `translateY(2px) → 0`. Hide after
  `--delay-tooltip-hide` (100ms) with a 80ms fade.
- Style: `--color-raised`, `--shadow-3`, `--radius-lg`, `--text-ui-sm`, max width 22rem,
  padding 8 × 12. Position above the word, flipping below near the top edge. Never
  cover the hovered word itself.
- Hovering the tooltip keeps it open (the Strong's tooltip has a "Open in Lexicon"
  link). Esc closes it.
- **Range highlights** (wave 2, highlights-pen.md) get the same
  highlight tooltip as a whole-verse highlight, naming the colour and its number:
  "Highlighted: Rose (4)"; over a range on a whole-verse tint it names both
  ("Highlighted: Gold (1) · pen: Gold (1)"). The paint layer has no pointer events, so the
  mark under the pointer is found by hit-testing (the caret offset under the pointer
  against the chapter's stored pieces, highlights-pen.md §4.3). Same timing and style as
  above. In pen mode the Strong's tooltip never opens (highlights-pen.md §4.4); the
  highlight tooltip still does, but never during a stroke, so it tells the reader what a
  click will recolour or remove without trailing the pen.
- **Reduced motion:** same timing, fade only.

### 6.5 Reduced motion: summary

Triggered by Windows "Animation effects: off" (`prefers-reduced-motion: reduce`) or
Settings → Reduce motion (`data-motion="reduce"`). The reader can also force full
motion (`data-motion="full"`). In reduced mode every movement distance token
(`--move-*`) is 0, `--dur-1` and `--dur-3` are 0, `--dur-4` is shortened to 120ms, the
caret doesn't glide and the FLIP is skipped. Fades of ≤ 120ms stay, because they move
nothing. The Strong's hover underline drops its `scaleX` grow and keeps its 120ms fade
(red-letters 3.4.3).

---

## 7. Core components (for every screen)

| Component | Spec |
|---|---|
| **Primary button** | Height 44 (setup, dialogs) or 36; padding 0 16; `--radius-md`; fill `--color-accent`, text `--color-on-accent`, `--text-ui-md` 500. Hover: a `::before` overlay in `--color-accent-hover` fades in (opacity, `--dur-2`), so no colour is ever animated. Pressed: `scale(0.98)` for `--dur-1`. One per screen at most |
| **Secondary button** | Same size; transparent fill, 1px `--color-border-strong` outline, `--color-text`. Hover: `--color-hover` overlay, same opacity fade |
| **Quiet button** | No outline; `--color-text-muted`. Hover: text switches to `--color-text` instantly, `--color-hover` overlay fades in. For Skip, Cancel and icon buttons |
| **Option row** (radio list) | Full-width row, `--radius-md`, padding 12 16, min height 56. Rest: transparent. Hover: `--color-hover` overlay (opacity fade, `--dur-2`). Selected: 1px inset `--color-accent` ring plus a 6px filled accent dot at the start, and `aria-checked="true"`. **Selected isn't shown by colour alone** (the dot) |
| **Text input** | Height 36, `--radius-sm`, 1px `--color-border-strong`, fill `--color-bg`. Focus: the focus ring |
| **Switch** | `role="switch"`, `aria-checked`. Track 32 × 18, `--radius-full`. Off: transparent, 1px inset `--color-border-strong` (≥ 3.10:1 on every surface), 12px thumb in `--color-text-muted`. On: track `--color-accent`, thumb `--color-on-accent`. The thumb moves `translateX(14px)` over `--dur-3` (180ms) `--ease-out`; the fills cross-fade by opacity over `--dur-2` (no colour transition). Reduced motion: `--dur-3` is 0, so the thumb jumps. Disabled: `aria-disabled="true"`, stays focusable, track and thumb `--color-text-disabled`, and the reason is shown in words beside it and linked with `aria-describedby`. Whole row is the click target |
| **Popover / menu** | `--color-raised`, `--shadow-2`, `--radius-lg`, padding 8. Opens: opacity 0 → 1 and `translateY(-4px) → 0` over `--dur-2` `--ease-out`; closes: opacity fade over `--dur-exit` `--ease-in`. Reduced motion: fade only. Esc closes and returns focus to the trigger; a click outside closes without moving focus |
| **Tooltip** | §6.4 |
| **Keyboard hint** | `<kbd>`: `--text-ui-xs` 500, 1px `--color-border`, `--radius-sm`, padding 1 6, `--color-text-muted` |
| **Icons** | 16px (UI) or 20px (toolbar), 1.5px stroke, rounded joins, `currentColor`. **Lucide**, pinned to one version (1.59.0): ISC, with MIT for the icons derived from Feather. Ship the whole upstream `LICENSE` (both notices) and credit "Lucide (ISC); includes icons from Feather (MIT)" on the About screen (`sources/SOURCES.md`, row ICON-LUCIDE). Recolouring and stroke changes are allowed. Always with a tooltip or a visible label |
| **Scrollbars** | Thin overlay style: 8px, thumb `--color-border-strong` at 60%, no track. Visible while scrolling and on hover of the scroll area |

---

## 8. App identity (early ideas only, for a later pass)

- **Wordmark**: "Revelation" set in the reading face's display cut (Literata),
  weight 500, slight positive tracking, sentence case. No ligature tricks, no cross
  motifs on letters.
- **Icon direction**: abstract and original, e.g. a single horizon line with light
  rising behind it ("dawn"), or the edge of an opening page catching light. Warm
  off-white on charcoal to match the theme. No depictions of God or Christ, no doves,
  halos or rays-from-a-figure, nothing echoing other Bible software.
- The app icon is original artwork, **never a Lucide icon**, and Revelation's
  branding never uses a font's Reserved Font Name or trademark.
