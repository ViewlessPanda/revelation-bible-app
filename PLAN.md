# Revelation: Project Plan

> **Revelation** is a free, open-source **native desktop app** for reading Scripture,
> understanding it, seeking wisdom, and writing thoughts down, in a simple,
> beautiful, dark-first interface.
>
> Installable and fully offline in its own window: no browser, no server, no
> account. Windows first; macOS/Linux possible later.
>
> An original project. It is not affiliated with any other Bible software, and
> contains none of their code, assets or data.

---

## 1. Priorities (in order, used to break every tie)

1. **Accuracy of text.** Scripture and commentaries are shown *exactly* as the
   source has them. Nothing is misquoted, summarized, paraphrased, "corrected"
   or generated. There is no AI anywhere in the content path.
2. **Reliability.** No buggy window behavior and no lost notes. Durable and
   well crafted. **KISS**: when in doubt, build the simpler thing.
3. **Silky-smooth UI.** Beautiful modern typography, a satisfying caret and
   motion (MS Word / Monkeytype-level feel), dark mode by default.
4. **Performance.** Instant startup, instant navigation, instant search.

If a feature threatens a higher priority, the feature loses.

---

## 2. Text-integrity rules (Priority 1)

These are hard rules for code, data and reviews.

1. **Two databases, never mixed.**
   - `content.sqlite` holds Bibles, commentaries, dictionaries and lexicons. It is
     **read-only** at runtime: opened with SQLite's read-only flag, and the app has
     no code path that writes to it. Translations added later arrive as their own
     read-only, checksummed pack files (see *Adding translations*). They're
     never merged into `content.sqlite`.
   - `user.sqlite` holds notes, highlights, last position and settings. It's the
     only database the app writes to.
2. **Content is built, not edited.** `content.sqlite` is produced by a
   deterministic import pipeline (`/pipeline`) from pinned source files. Nobody
   hand-edits content. The same inputs always produce the same output.
3. **Pinned sources.** Every source file is listed in `SOURCES.md` with its URL,
   version or date, license, and **SHA-256 checksum**. The pipeline refuses to
   run if a checksum doesn't match.
4. **No silent corrections.** If a source has a typo, we keep it as the source
   has it. Changing that requires a written errata policy, evidence from a
   scan of the printed original, and a visible note in the app. v1 makes **no**
   corrections; we choose the cleanest sources instead.
5. **Display = storage.** The renderer only adds *presentation* (fonts, spacing,
   verse numbers, red letters, italics for supplied words). It never changes
   characters. In particular:
   - no "smart quote" or auto-typography on content text;
   - no Unicode normalization of displayed text (Hebrew/Greek points and
     accents are kept byte-for-byte). Normalized copies exist **only** inside the
     search index;
   - copy/paste copies the exact stored text plus a reference.
6. **Automated verification (CI blocks merge on failure):**
   - verse counts per book and chapter match the declared versification
     (e.g. KJV = 31,102 verses);
   - **round-trip test**: export every verse from `content.sqlite` and compare it
     to the parsed source; any difference fails the build;
   - **render test**: the rendered DOM text of sampled and full chapters equals the
     database text (minus presentation-only elements);
   - golden samples: a hand-checked list of well-known verses (Gen 1:1, Ps 23,
     John 3:16, etc.) compared to the printed edition;
   - content DB checksum recorded per release; the app verifies it on startup and
     refuses to show text from a corrupted file (shows a "repair/reinstall"
     message instead).
7. **No generated content ever.** No AI summaries, no "explain this verse", no
   machine translation. The app shows sources; the reader does the thinking.
8. **Red letters are held to the same standard as the words themselves.**
   Red lettering tells the reader "Jesus himself said this", so the exact
   red/black boundaries are content, not styling.
   - Red letters come **only** from the translation's own words-of-Jesus markup
     (e.g. USFM `\wj … \wj*`, OSIS `<q who="Jesus">`). The app never infers,
     guesses, extends or trims them, and never copies red-letter spans from
     one translation to another.
   - The pipeline stores each red span as exact character boundaries within the
     verse, exactly as the source marks them, including spans that start or end
     mid-verse or cross verse boundaries.
   - If a translation has no words-of-Jesus markup, that translation shows **no**
     red letters (and the red-letter toggle says "Not available in this
     translation"). It never borrows another edition's markup.
   - The red-letter toggle only switches the color on or off. It never changes
     which words are marked.
   - Red-letter markup in public-domain editions comes from a specific digital
     edition (the 1611 KJV wasn't printed red-letter), so `SOURCES.md` records
     which edition each translation's red letters come from.
   - **Verification (CI blocks merge on failure):**
     - round-trip: for every verse, the set of red character ranges in the
       database equals the source markup exactly (count, start, end);
     - render test: the DOM's red spans contain exactly the same characters as
       the database ranges, with nothing extra red and nothing missing;
     - totals per book (number of red spans and red characters) match the
       source, so any dropped or added span fails loudly;
     - golden samples hand-checked against a printed red-letter edition,
       including tricky boundaries (quotes within narration, mid-verse
       starts, Old Testament quotations inside Jesus' words, and passages
       like John 3:16–21 where editions differ);
     - imported files get the same red-letter checks in their validation report.

---

## 3. Content sources (candidate list, **license must be verified before import**)

Nothing goes into the repo until its row in `SOURCES.md` is filled in and checked.

| Type | Candidates (to verify) | Expected status |
|---|---|---|
| Bible (with Strong's) | KJV 1769 text with Strong's tagging (e.g. CrossWire KJV module, eBible.org KJV) | Public domain in the US* |
| Bibles | ASV 1901, WEB, YLT, Darby, Webster 1833, **Berean Standard Bible** | Public domain |
| Hebrew / Greek texts | OpenScriptures Hebrew Bible (OSHB), STEPBible TAHOT/TAGNT, Nestle 1904, Byzantine, Scrivener TR | Mix of PD and CC BY 4.0 |
| Lexicons | Strong's (original), STEPBible TBESH/TBESG, Thayer's, BDB | PD / CC BY (verify the specific digitization) |
| Commentaries | Matthew Henry, Gill, Clarke, Barnes, JFB, Wesley, TSK cross-refs | PD texts; verify the *digitization's* terms |
| Dictionaries | Easton's (1897), Smith's (1863), Hitchcock's Bible Names | Public domain |
| Images | Wikimedia Commons (PD / CC only), 19th-century PD Bible-dictionary illustrations | Per-image license |

\* The KJV is under Crown letters patent in the UK. We'll note this in `SOURCES.md`.

**Excluded:** NIV, ESV, NASB, NKJV, NLT, CSB, AMP and other copyrighted
translations; BDAG, HALOT, Mounce, newer Vine's editions and other modern
references; anything from other Bible software's or BibleSupport module files. A later
version may let users **import their own** files; we never ship them.

---

## 4. Features (v1 scope)

### First-run setup
The first time Revelation opens, a calm, three-step welcome lets the reader
choose their defaults. It takes about a minute, and everything can be changed
later in Settings.

1. **Reading font.** Each option shows a live sample with real verse text, read from
   `content.sqlite` (never hard-coded), in that font on the dark theme.
2. **Bible translation.** Listed with its **publication date**, e.g.
   *American Standard Version (1901)*. The preview updates to show the chosen
   translation.
3. **Commentary.** Listed with the **Christian tradition next to the author's
   name**, e.g. *Matthew Henry · Presbyterian*, plus the date. A short preview
   shows that commentary's actual text for a familiar verse.

Then the app opens to Genesis 1 (and from then on, to where the reader left off).

- **Skip** is always available and sets the defaults: Literata, KJV, Matthew Henry.
- Keyboard-friendly (Enter/Back), smooth step transitions, no account, no network.
- Choices are saved to `user.sqlite` in one transaction at the end. If the app is
  closed mid-setup, setup starts again next time (no half-saved state).
- **Every font option is bundled** (all SIL OFL), so whatever the reader picks works
  offline.

**Translation and commentary labels are content, so priority 1 applies.**
Names, dates and traditions are stored as metadata in `content.sqlite`, each
with a cited source in `SOURCES.md` (the edition's own front matter for dates;
a standard biographical reference for traditions). The UI never hard-codes them.
Traditions use short, neutral, factual wording, with no ranking or commentary
on them.

Candidate translations (dates to confirm from each edition's front matter):

| Translation | Date shown |
|---|---|
| King James Version | 1769 (label approved by the owner 2026-10-22: "King James Version (1769)", per `docs/scholar/kjv-label.md`) |
| Webster's Bible | 1833 |
| Young's Literal Translation | 1862 (revised 1887, 1898) |
| Darby Translation | 1890 |
| American Standard Version | 1901 |
| World English Bible | 2020 (stable edition). The name is a trademark, which is fine because we never alter the text |
| Berean Standard Bible | *to confirm*: sources disagree (2016 vs 2020); renamed in 2022, public domain since April 2023 |
| Geneva Bible *(candidate)* | 1599 |
| Douay-Rheims, Challoner revision *(candidate)* | 1749–1752 |

Candidate commentaries, with each author's tradition (to confirm against the
cited references):

| Commentary | Tradition shown next to the author | Date |
|---|---|---|
| Matthew Henry (Complete / Concise) | Presbyterian (English Nonconformist) | 1706–1721 (finished by others after his death in 1714) |
| John Gill | Baptist (Particular Baptist) | 1746–1763 |
| Adam Clarke | Methodist | 1810–1826 |
| Albert Barnes | Presbyterian | 1830s–1870s |
| Jamieson-Fausset-Brown | Church of Scotland · Anglican · Free Church of Scotland (one per author) | 1871 |
| John Wesley | Anglican (founder of Methodism) | 1755 (NT), 1765 (OT) |
| Matthew Poole | Presbyterian (English Nonconformist) | 1683–1685 |
| John Calvin | Reformed | 1540s–1560s; English translation 1840s–1850s |
| Keil and Delitzsch (OT) | Lutheran | 1860s–1870s (English edition) |
| C. H. Spurgeon, *Treasury of David* (Psalms) | Baptist (Particular Baptist) | 1869–1885 |
| Marvin Vincent, *Word Studies* (NT) | Presbyterian (began in the Methodist ministry) | 1887–1890 |
| Geneva Bible notes | Reformed (English exiles in Geneva) | 1599 edition |

The same "Author · Tradition" label appears wherever a commentary is picked,
not just during setup.

### Reader
- Opens **exactly where you left off**: book, chapter, verse, scroll offset and
  open side tab. Saved continuously (debounced) and on close.
- Fast navigation: type `jn 3 16`, `ps23` or `Romans 8` into one command
  box; plus a simple book/chapter picker.
- Verse numbers, red letters, italics for supplied words; all toggleable.
- Strong's numbers are hidden by default; hovering a word shows a gentle tooltip, and clicking it opens the
  Lexicon tab.
- Parallel view (2 translations side by side). That's the only "multi-view" in v1.
- **Automatic bookmarks**: one per book, the last chapter actually
  read in that book (open and focused for a while, or scrolled; quick jumps and searches
  don't count). The book picker shows a small bookmark ribbon on such books and on the
  bookmarked chapter, so a reader going through several books sees where they left off in
  each. Stored in `user.sqlite` (book → chapter, timestamp); local only. No manual bookmarks
  for now.

### Study side panel (one panel, tabs, no floating windows)
- **Commentary**: follows the current verse; pick from installed commentaries
  (shown as "Author · Tradition").
- **Dictionary**: search by word or topic; entries can show **curated images**
  (see below); scripture references in entries are clickable.
- **Lexicon**: the Strong's entry for the clicked word, plus "find every verse
  with this word".
- **Notes**: Study notes (per verse), Topic notes (titled pages) and Journal
  (dated entries).

### Dictionary images (safe and accurate by design)
- **Curated, not live-searched.** Each image is hand-linked to a dictionary
  entry (e.g. *Fig* → photo of a fig tree), reviewed by a person against a
  checklist: relevant, accurate, non-explicit, and a respectful subject.
- v1 sticks to **plants, animals, objects, places, maps and artifacts**. No
  depictions of God or Christ (many traditions object to them, and they aren't
  needed for understanding).
- Every image carries source, author, license and attribution in the database;
  attribution is shown under the image.
- Bundled offline as compressed WebP. No network calls, no tracking.

### Notes (the writing experience)
- One beautiful editor for all three note types: bold, italic, headings, lists,
  quotes, and **scripture links** (`Jn 3:16` becomes a hover-preview link).
- Autosave on every pause (≈500 ms), with SQLite WAL so a crash or power loss loses
  at most a moment of typing.
- Plain-folder Markdown export plus automatic daily backups of `user.sqlite`
  (last 14 kept).
- Notes **quote** scripture by reference and show live text; they never store a
  copied-and-editable verse that could drift.

### Highlights
- **Whole-verse highlights** in a small palette of colors designed for dark mode
  (e.g. gold, green, blue, rose, violet). Each is a soft tint behind the verse,
  so text contrast never drops below WCAG AA.
- **Whole-verse** highlights belong to the **verse reference**, not a translation, so a verse
  highlighted in the KJV stays highlighted in the ASV or BSB. Range highlights show only in
  their own translation (design Q26, default).
- Applied with one click or a keyboard shortcut (1–5); removed the same way. With a text
  selection, 1–5 highlight the selected words as a range (design Q30, default).
- A "Highlights" list (filter by color) jumps straight to any highlighted verse.
- Stored in `user.sqlite`: whole-verse highlights as `(reference, color, created_at)`; range
  highlights as `mark` + `mark_piece` rows (schema v2, additive; the v1 table stays the
  whole-verse table). Highlights never touch the text itself.
- **Range highlights**: a highlighter pen (H, or the toolbar button)
  paints any span of words, even across verses, in the same five colours. Each piece is
  stored as `(translation, book, chapter, verse, start, end)` in Unicode code points of the
  verified text, plus a fingerprint of that verse. They are drawn on a separate layer and
  never touch the text. If a translation's text changes, they are never moved; they are
  listed for review instead. See `docs/design/highlights-pen.md`.

### Memorize
- **Save any verse or range** (e.g. Rom 8:28, Ps 23:1–6) to the **Memorize**
  folder with one click; organize it into sub-folders (e.g. "Comfort", "Gospel").
- Each saved item stores the **reference + translation**, never a copied
  string. Its text is always read live from `content.sqlite`, so it can't drift or
  be accidentally edited.
- A simple built-in review mode: show the reference, reveal the verse, then mark
  "got it" or "again".
- **Export to Anki** (`.apkg`), one deck per folder (sub-folders become
  sub-decks):
  - Front: reference + translation (e.g. *John 3:16 (KJV)*). Back: the
    exact verse text, with verse numbers for ranges. Optional reverse cards.
  - Text comes straight from `content.sqlite` at export time, and the export test
    re-reads the generated deck and checks every card against the database.
  - Re-exporting updates existing cards instead of duplicating them (stable
    note IDs from the reference + translation), so Anki keeps your review
    history.
  - Built with a small, well-tested `.apkg` writer (candidate: the
    `genanki-rs` crate, to be evaluated), with plain **TSV export** as a fallback
    (Anki's File → Import also reads it).

### Adding translations
One **Add translation** button (in the translation picker and in Settings)
opens a simple window with two ways in:

1. **From the Revelation library** (one click):
   - A list of translations the project has already verified through the
     pipeline (Section 2), each with name, date, language, size and license.
   - Click **Install**. The pack downloads from the project's GitHub Releases,
     its **signature and SHA-256** are checked, and it appears in the translation
     picker immediately. A failed check means nothing is installed and the reader
     sees a clear message.
   - The app makes only **two** kinds of network call: this library (only when the reader
     opens it) and the signed update check (Section 9, backlog idea 5; owner approved
     2026-10-22). The app works fully offline: a failed, slow or blocked update check
     never blocks or degrades anything. There's no other background traffic and no tracking.
   - Packs can also be installed **offline** by opening a downloaded `.revpack` file.
2. **From a file you have** (for texts the reader has the right to use):
   - Standard open formats: **USFM**, **USX**, **OSIS** and **Zefania XML**; later
     possibly other Bible software's own module formats for the reader's own files.
   - The importer copies text **byte-faithfully**, with no cleanup or
     "fixing". Before anything is added, the reader sees a **validation report**
     (books, chapters and verse counts against the declared versification;
     missing or extra verses listed). Nothing is added until they confirm.
   - Imported translations are labelled **"Imported by you"**, along with the file's
     own name, date and copyright line. They stay on that computer and are never
     uploaded or shared.

**One format, one code path (KISS).** Library packs and imported files both
become the same pack: a read-only SQLite file plus a small manifest (name,
abbreviation, date, language, versification, license, checksum). Installing,
removing and verifying work the same way for both. Removing a translation
never touches the reader's notes, highlights or Memorize items; those keep the
reference and simply show the text from another translation.

### Search
- Word or phrase search across a translation, plus Strong's-number search.
- SQLite FTS5 index built by the pipeline (search-only normalized text).

### Out of scope for v1 (to keep it simple)
Docking or floating windows, reading plans, printing, a downloader for
commentaries and dictionaries (translations only in v1), sync, mobile, and AI
features (never).

---

## 5. Architecture (KISS)

```
┌─────────────────────────── Desktop app (Tauri 2) ───────────────────────────┐
│  UI (Svelte 5 + TypeScript)                                                 │
│   Reader · Side panel tabs · Note editor (ProseMirror) · Command box        │
│                         │  small typed command API                         │
│  Core (Rust, thin)      ▼                                                   │
│   content.rs (read-only queries) · user.rs (notes, state) · backup.rs      │
│        │                                   │                               │
│  content.sqlite (read-only)        user.sqlite (WAL, backups)              │
└─────────────────────────────────────────────────────────────────────────────┘
     ▲ built by
  /pipeline (Rust CLI): fetch pinned sources → verify SHA-256 → parse →
  validate → write content.sqlite + FTS index → round-trip test
```

**Why these choices**
- **Tauri 2** gives a small, fast app on WebView2 (Windows), with macOS/Linux later.
  It also has a mature window-state plugin (saves and restores size and position, and
  clamps to a visible monitor, which fixes "window opens off-screen" bugs).
- **Svelte 5** compiles to plain JS. Small, fast and easy to read.
- **ProseMirror** is a proven editor core (used by many big editors). It
  stores notes as structured JSON (lossless), with Markdown export for portability.
- **SQLite** is one file, transactional, extremely durable and very fast.
- **Rust for the pipeline and core** gives strict parsing and strong tests in one language with
  the app backend. The core stays thin: queries in, plain data out.

**Repo layout**
```
/app         Tauri + Svelte desktop app ("Revelation")
/pipeline    content import CLI + verification tests
/sources     SOURCES.md, checksums, (raw files are downloaded, not committed)
/assets      fonts (OFL), curated images + image manifest
/docs        PLAN.md, design notes, errata policy
```

---

## 6. Design system (Priority 3)

**Dark-first palette.** No pure black or white: a soft charcoal background,
warm off-white text, one restrained accent. Light and sepia themes come later
from the same tokens.

**Typography**
- Reading font: chosen by the reader at first run from eight bundled OFL serifs
  (Literata default). UI font: *Inter*.
- Hebrew: *Ezra SIL* / *Noto Serif Hebrew*; Greek: *Gentium Plus*. All SIL OFL.
- Reading column 60–72 characters wide, line-height ≈ 1.6, real small caps
  for LORD, superscript verse numbers with comfortable spacing, and hanging
  verse numbers in paragraph mode.
- Font size changes in smooth steps (Ctrl +/−); layout never jumps.

**Motion and feel**
- **Smooth caret** in the editor: the native caret is hidden and a custom caret
  glides between positions (~80 ms ease-out) and blinks gently; the Monkeytype/Word
  feel. It turns off automatically when *Reduce motion* is on.
- Animations use only transform/opacity at 60+ fps. Nothing animates layout.
- Smooth, momentum-respecting scrolling; chapter changes cross-fade in
  ≤120 ms; side-panel tabs slide; tooltips fade in after a short intent delay.
- Keyboard-first: everything reachable without a mouse; visible focus states.
- Accessibility: WCAG AA contrast, screen-reader labels, and respect for OS text
  scaling and reduced-motion settings.

---

## 7. Reliability practices (Priority 2)

- **One main window**, one side panel and resizable splitters with min/max
  widths. No floating windows in v1.
- Window and panel state saved and restored safely (validated, clamped to screen).
- All writes go through one small `user.rs` module using transactions; schema
  migrations are versioned and tested on real old databases.
- Startup self-check: content checksum, `user.sqlite` integrity check, and
  automatic restore from the latest backup if corruption is found (asks first).
- Errors never crash silently: user-friendly message plus a local log file.
  No telemetry.
- Tests: Rust unit tests, pipeline verification, UI end-to-end tests
  (open → navigate → write note → kill app → reopen → note is there).
- CI on every PR: lint, tests, text-integrity suite, Windows build.

---

## 8. Performance targets (Priority 4)

| Action | Target |
|---|---|
| Cold start to text on screen | < 1 s |
| Chapter change | < 50 ms |
| Lexicon / commentary lookup | < 30 ms |
| Full-Bible word search | < 150 ms |
| Typing latency in editor | < 16 ms (one frame) |
| Installer size (KJV + core library) | < 60 MB |

Measured in CI where possible; regressions are bugs.

---

## 9. Milestones

Each milestone ends with a working, tested app. Accuracy work comes **before**
UI work.

| # | Milestone | Done when |
|---|---|---|
| 0 | **Foundations** | Repo `revelation`; MIT `LICENSE` with author credit; data licenses per source; `SOURCES.md`; CI skeleton |
| 1 | **Content pipeline: KJV + Strong's** | `content.sqlite` built from pinned source; verse counts, round-trip and golden tests pass |
| 2 | **Reader MVP** | Dark theme, typography, navigation, command box, resume where you left off, window state; first-run setup (font + translation steps) |
| 3 | **Lexicon** | Strong's tooltips, Lexicon tab, "every verse with this word" |
| 4 | **Commentaries** | First PD commentaries imported and verified, with cited tradition labels; verse-synced tab; first-run commentary step |
| 5 | **Dictionaries + images** | Easton's/Smith's imported; first ~200 curated images with attribution |
| 6 | **Notes** | Study, topic and journal notes; smooth-caret editor; autosave, backups, export |
| 7 | **Highlights + Memorize** | Whole-verse highlights; Memorize folders and review mode; verified Anki `.apkg` + TSV export |
| 8 | **Search** | FTS5 word/phrase and Strong's search |
| 9 | **Adding translations** | `.revpack` format; signed library packs on GitHub Releases; USFM/USX/OSIS/Zefania import with validation report |
| 10 | **Polish and v1.0 release** | Performance pass, accessibility pass, installer, signed release on GitHub |

### Ideas backlog (owner, 2026-10-22)

These are ideas the owner has asked for. None of them is scheduled yet. Each
must keep to Section 1 (accuracy, then reliability, then UI, then speed) and
Section 2. They are slotted into milestones after the first demo.

1. **Daily rewards for opening the app.** Revelation has no accounts, so a
   "login" means opening the app (or reading) on a given day.
   - It's offline and local only, and the record is kept in `user.sqlite`. It
     never uploads anything or tracks the reader.
   - Rewards are cosmetic or comfort items, such as highlight colours, caret
     styles, theme accents or page-turn sounds. They never lock or unlock
     Scripture, commentaries or any study feature.
   - It's gentle: there's no guilt or "streak lost" pressure, missed days are
     forgiven, and it can be switched off in Settings. Animations follow
     *Reduce motion*.
   - Open questions: what counts as a day (opening the app,
     reading one chapter, or a time spent), and which rewards to offer.
   - Likely slot: after Milestone 7 (Highlights + Memorize), with the designer.
2. **Page-turn reading mode.** In **Reading options**, the reader chooses
   **Scroll** (one long, continuous page, as today) or **Pages** (paginated
   spreads with a satisfying page-curl turn, like Apple Books).
   - Turns are driven by clicking, the arrow keys, swiping on the trackpad, or
     dragging the page corner. The curl follows the pointer.
   - It uses only transform and opacity (or WebGL/canvas) at 60+ fps
     (Section 6). Pagination never reflows or alters the text; a page is just
     a view of the same verified verses. Reduce motion swaps the curl for a
     quick cross-fade.
   - "Opens where you left off" works in both modes, including the page
     number after a resize or font change.
   - Likely slot: Milestone 2 (Reader) follow-up or Milestone 10 polish,
     after a designer prototype.
3. **More translations and commentaries.** Grow the verified library beyond
   KJV: the Section 3 candidates first (ASV, WEB, YLT, Darby, Webster,
   Berean Standard Bible; Matthew Henry, Gill, Clarke, Barnes, JFB, Wesley,
   TSK), then more after a license search (for example, other PD and
   openly licensed English Bibles, non-English Bibles, and more PD
   commentaries across traditions). Every addition goes through the pipeline,
   with a cited date or denomination and a `SOURCES.md` row. There are no
   exceptions to Section 2.
   - Slot: Milestones 4 and 9, which can be widened.
4. **More translations, and optional own-key reading.** The Berean Standard Bible and the WEB (both public
   domain) come first. Readers will also be able to import a text file they have the right to use. An
   optional way to read a publisher's text (for example the ESV, through Crossway's ESV API) with the
   reader's own free key is planned, under each publisher's terms. Revelation never bundles copyrighted
   texts without permission and never circumvents DRM.
5. **Automatic updates.** The app finds, downloads and installs new versions
   by itself, so the reader never has to download an installer by hand.
   - It uses Tauri's updater, which pulls from the project's GitHub Releases.
     Every update is **signed**, and the app refuses one whose signature or
     SHA-256 doesn't match, keeping the current version. Content files in an
     update are still opened only through the hash-checked loader, so an
     update can never slip in altered text.
   - It's quiet: the app checks at most once a day, downloads in the
     background, and installs on the next restart (or "Restart now"). Release
     notes are shown, and the reader can turn automatic checks off and check
     by hand instead.
   - **Owner decision (2026-10-22): approved** as a second network call, on the
     condition that the app keeps working fully offline (a failing update check
     never blocks or degrades anything). Section 4's network rule is updated.
     The update check sends nothing about the reader beyond what any download does.
   - Needs: a signing key kept off the repo (M10), the installer (M10), and
     a security review of the update path. Likely slot: Milestone 10.
6. **Reading stats.** Everything is local only, in `user.sqlite`, and never
   uploaded. It can be turned off and cleared from Settings.
   - Time spent per book and chapter, grouped by testament and by section
     (Law, History, Wisdom, Prophets, Gospels, Letters and so on), with
     good-looking charts.
   - Notes per book (which book you write the most about), highlights per
     book, and your average reading-session length, plus your longest and
     total time.
   - Reading time counts only while the window is focused and you're
     active. It stops after a short idle pause, so leaving the app open
     overnight doesn't count.
   - Likely slot: after Milestone 6 (Notes), with the designer.
7. **Progress heat map, badges and rewards.** A calendar heat map (like a
   contribution graph) of the days you read, plus badges such as "read every
   chapter of John" or "30 days in the Psalms" and the rewards from idea 1.
   - You can view it in Settings, or **pin** a compact version to the bottom
     left of the window. It is merged with idea 1 into one gentle,
     no-guilt system that never locks content.
   - Likely slot: with ideas 1 and 6.

---

## 10. Dev environment

The project can live on any drive. Rust, Node and npm keep their toolchains and caches under your user
profile by default; to keep them elsewhere (for example on a small system drive), set `RUSTUP_HOME`,
`CARGO_HOME` and `npm config set cache` before installing anything. `npm run verify` uses your environment
as it is; set `REVELATION_TOOLS_HOME` to a folder that holds `rustup`, `cargo`, `node` and `tmp` if you
want it to use those. Downloaded raw sources go in `sources/raw` (git-ignored). WebView2 (part of Windows 11)
and the Visual Studio C++ Build Tools are required on Windows.

---

## 11. Decisions

| Decision | Choice |
|---|---|
| Project name | **Revelation** |
| App type | Native desktop app (Tauri 2), offline, Windows first |
| Code license | **MIT**: anyone may use, modify, fork and redistribute (even commercially), as long as the copyright notice crediting the author stays in every copy |
| Data licenses | Each source keeps its own license (PD / CC BY / etc.), listed in `SOURCES.md` |
| Copyright holder | **C. Ethan Key** (`Copyright (c) 2026 C. Ethan Key`) |
| Font, translation, commentary | **The reader chooses** in first-run setup (defaults if skipped: Literata, KJV, Matthew Henry) |
| Reading fonts bundled | Literata, Source Serif 4, EB Garamond, Crimson Pro, Lora, Newsreader, Spectral, Gentium Book Plus (all SIL OFL) |
