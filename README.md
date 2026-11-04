# Revelation

Revelation is a free, open-source desktop app for reading Scripture,
understanding it, and writing your thoughts down, in a simple, calm,
dark-first interface.

It runs fully offline in its own window: no browser, no server, no account,
no ads and no tracking. Windows comes first; macOS and Linux may follow later.

## Status

**Pre-alpha.** There is no installer yet; you build the app yourself (see
[Building](#building)). The full plan is in [PLAN.md](PLAN.md).

What works today:

- **Reader:** the King James Version with red-letter words of Jesus, chapter
  and verse navigation, and a dark-first theme.
- **Strong's lexicon:** hover a word to see its underline; click it for the
  Hebrew or Greek entry.
- **Commentary:** Matthew Henry's Concise Commentary, stacked with the lexicon
  in one right-hand column with an adjustable divider.
- **Pen highlights:** switch to pen mode and mark any stretch of text, in
  several colours that keep the text readable.
- **Phrase search:** type a phrase such as `Jesus wept`, or a reference such
  as `John 11:35`, in the top bar.
- **Daily notes:** a notes panel that saves as you type, kept by day and
  tagged with the passage you are reading.
- **Automatic bookmarks:** the app remembers where you left off in each book.

Being built now: a first-run setup, the Berean Standard Bible, optional ESV
reading through Crossway's ESV API with your own free key, notes by day with
search, and commentary highlighting.

## Accuracy first

The project's priorities, in order: **accuracy of the text, then reliability,
then a smooth interface, then speed.**

- **No AI in the content path.** Scripture, commentaries and lexicons are
  shown exactly as their sources have them. Nothing is summarized,
  paraphrased, corrected or generated.
- **Verified sources.** Every Bible and study text is built from a pinned,
  checksummed source file listed in [`sources/SOURCES.md`](sources/SOURCES.md).
  The build checks each verse against an independent reader of the same
  source and against hand-checked samples, and refuses to publish anything
  that differs.
- **Your data stays yours.** Highlights, notes and bookmarks live in one local
  file (`user.sqlite`) that only the app writes. They point at the text; they
  never change it.

## Building

You need, on Windows:

- [Rust](https://rustup.rs) (stable, MSVC toolchain), with the
  [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for
  Windows (Microsoft C++ Build Tools and WebView2);
- [Node.js](https://nodejs.org) 24 or later;
- [Git for Windows](https://git-scm.com), whose Git Bash is used to fetch the
  sources;
- optionally, [Python](https://www.python.org) 3, for the colour-contrast
  check in `verify` (it is skipped without Python).

**1. Fetch the pinned sources.** The Bible and study texts are not stored in
this repository. This script downloads each one from the URL listed in
`sources/SOURCES.md` and checks its SHA-256 before keeping it. In Git Bash,
from the repository root:

```bash
bash .github/scripts/fetch-sources.sh sources/raw
```

**2. Build the content databases.** In PowerShell, from the repository root:

```powershell
New-Item -ItemType Directory -Force pipeline\out | Out-Null
cargo run --release --locked -p revelation-pipeline -- build sources\raw pipeline\out
```

It prints `published ... sha256` lines when every check passes, or
`FAILED (nothing published)` if any check fails.

**3. Check and run the app.** (`npm.cmd` is used because Windows blocks
`npm.ps1` when PowerShell scripts are turned off.)

```powershell
cd app
npm.cmd ci
npm.cmd run verify
npm.cmd run tauri dev
```

`verify` runs every local check: type checks, unit tests, Rust tests and the
content-integrity tests.

**4. Optional: a release build.** `npm.cmd run release` builds
`target\release\revelation.exe` and copies the checked content files next to
it. It first checks that no other account on the PC can modify the files the
build trusts, and stops if one can; see [app/README.md](app/README.md).

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) explains how to
build and test, the accuracy rules (scripture text is never altered), and what
a pull request should include.

## Security

To report a vulnerability, use GitHub's private vulnerability reporting; see
[SECURITY.md](SECURITY.md). Please do not open a public issue for it.

## License

The source code and documentation are under the MIT License; see
[LICENSE](LICENSE). Copyright (c) 2026 C. Ethan Key.

The MIT License does **not** cover the content (Bible texts, commentaries and
lexicons), which keep their own licences, listed below and in
[`sources/SOURCES.md`](sources/SOURCES.md). Nor does it cover the Revelation
name, logo or app icon (see [App icon](#app-icon)).

### Content in this repository and its credits

- **King James Version.** The text (CrossWire KJV 3.1, with Strong's numbers
  and words-of-Jesus markup) is distributed under the GNU General Public
  License, version 3; see [`licenses/GPL-3.0.txt`](licenses/GPL-3.0.txt). It
  is built into its own data file, `kjv.sqlite`, from CrossWire's
  `kjv.osis.xml` (pinned in [`sources/SOURCES.md`](sources/SOURCES.md), row
  BIB-KJV-CW31) by the pipeline in [`pipeline/`](pipeline/). Each release
  that includes it also offers that source as a download. The app's code
  remains under the MIT License.
- **Strong's Hebrew Dictionary.** Strong's Hebrew Dictionary (James Strong,
  1890; public domain), XML edition by the Open Scriptures Hebrew Bible
  Project (https://github.com/openscriptures/HebrewLexicon), licensed under
  CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). Provided "as is"
  without warranties. Converted to Revelation's database format; entry text
  unchanged.
- **Strong's Greek Dictionary.** James Strong, 1890 (public domain), in the
  "Strong's Greek Dictionary in XML with real Greek" edition by Ulrik
  Sandborg-Petersen (MorphGNT, CC0), from an e-text by Michael Grier
  distributed by The Bible Foundation.
- **Matthew Henry's Concise Commentary on the Whole Bible.** Public domain,
  from the CrossWire Bible Society MHCC module, version 2.0
  (https://gitlab.com/crosswire-bible-society/mhcc), which was prepared from
  the Christian Classics Ethereal Library's text. Converted into Revelation's
  database format; text unchanged.
- **No ESV text is in this repository.** ESV is a trademark of Crossway; the
  ESV tests use placeholder text only.
- **Test data.** The golden and excerpt files carry the licences of their
  sources and are not MIT: those under `app/src-tauri/tests/golden/`,
  `pipeline/tests/`, `docs/pipeline/` and `docs/scholar/` hold short excerpts
  of the KJV (GPL-3.0), the Hebrew lexicon (CC BY 4.0, credited above), the
  Greek lexicon (CC0) and Matthew Henry (public domain).
- **Fonts.** Inter and Literata, under the SIL Open Font License 1.1; each
  family's licence file is in [`assets/fonts/`](assets/fonts/), and the
  copyright lines are in [`sources/SOURCES.md`](sources/SOURCES.md).

### Third-party software

The licences of the software dependencies are listed in `Cargo.lock` and
`app/package-lock.json`. A full third-party notices file will come with the
first binary release.

### App icon

The app icon and the mark in the About dialog are made from
[`assets/brand/revelation-icon-1024.png`](assets/brand/), which was created
with an AI image tool (ChatGPT); see [`sources/SOURCES.md`](sources/SOURCES.md),
row BRAND-LOGO, and [`assets/brand/LICENSE`](assets/brand/LICENSE).

The name, logo and app icon (the files in `assets/brand/`,
`app/public/brand/` and `app/src-tauri/icons/`) identify this project. You may
not use them to brand other software or to suggest endorsement without
permission. Forks and modified versions must use a different name and icon.
