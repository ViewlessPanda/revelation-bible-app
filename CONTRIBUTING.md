# Contributing to Revelation

Thank you for your interest. Revelation is a free, open-source, offline Bible study
app (Tauri 2, Svelte 5, Rust, SQLite). The project's priorities, in order, are
**accuracy of the text, then reliability, then a smooth interface, then speed**. A
contribution that helps the first two is always welcome; one that trades them away
for the last two will not be merged.

## Building and testing

The build steps are in the [README](README.md#building): fetch the pinned sources,
build the content databases, then check and run the app. Before you open a pull
request, run the full local check from `app/`:

```powershell
npm.cmd ci
npm.cmd run verify
```

`verify` prints one PASS or FAIL line per gate (formatting, UI tests, type checks,
build, clippy, Rust tests, colour contrast, database hashes, content bytes and
tracked files). All of them must pass. It uses your environment's Rust, Node and
Python as they are; set `REVELATION_TOOLS_HOME` only if you keep the toolchains in
one folder (`rustup`, `cargo`, `node`, `tmp`) and want it to use them.

## Accuracy rules

These come from [PLAN.md](PLAN.md), Section 2, and the tests enforce them.

- **Never alter scripture text.** Bible, commentary, lexicon and dictionary text is
  shown exactly as its source has it: no smart quotes, no normalisation (except
  inside the search index), no trimming, no "fixing", no paraphrase.
- **No AI-generated content in the content path.** Code and documentation may be
  written with help, but nothing generated may become scripture, commentary,
  lexicon or dictionary text.
- **Pinned content is not edited casually.** Files under `sources/`, `**/golden/`,
  `pipeline/tests/expected/` and `pipeline/tests/fixtures/` are compared byte for
  byte by CI. If a change to one of them is really needed, say why in the pull
  request and show the test that proves it is correct.
- **A new source needs a pinned URL, a SHA-256 and a row in
  [`sources/SOURCES.md`](sources/SOURCES.md)**, with its licence and the credit
  text it requires. Nothing is added to the app whose licence is unclear.
- **Content files are opened read-only**, and only `user.sqlite` is ever written,
  through one small module.
- **Keep it simple.** One main window, minimal dependencies. A new crate or npm
  package must be justified (maintenance, licence, size) in the pull request.

If you think a verse, commentary entry or lexicon entry shows wrong text, please
use the "Text accuracy report" issue form.

## Pull requests

- One topic per pull request, and keep it small.
- Add or update tests with the change. Anything that touches content needs
  verification tests (verse counts, round trip, golden samples).
- Follow the existing style; `cargo fmt` and the repository's `.editorconfig` set
  the basics.
- Fill in the pull request template checklist.

By contributing you agree that your contribution is licensed under the
[MIT License](LICENSE). The Revelation name, logo and app icon are not covered by it;
see [`assets/brand/LICENSE`](assets/brand/LICENSE).

## Reporting a security problem

Please do not open a public issue. See [SECURITY.md](SECURITY.md).
