# app

The Revelation desktop app: Tauri 2 + Svelte 5 (runes) + TypeScript, plain Vite
(no SvelteKit). It reads the real KJV built by the pipeline (`pipeline/out/kjv.sqlite`) and the real
lexicons (`content.sqlite`); a synthetic placeholder fixture is used by tests only. The verified-open
code is in the shared crate `crates/revelation-content` (used by the pipeline and the app).

```
app/
  index.html, vite.config.ts, svelte.config.js, tsconfig.json, package.json (exact pins)
  src/            UI: App.svelte, components/, lib/ (parser, verse DOM, IPC types), styles/
  src/styles/tokens.css   verbatim copy of docs/design/tokens.css (a test checks it)
  tests/          vitest + happy-dom: parser, render test, CSS colour-lock lint, lexicon
  scripts/        csslint.mjs (colour lock + motion lint), lint-built-css.mjs, vitest.mjs, make-icon.mjs,
                  copy-content.mjs (release content), sync-fonts.mjs (fonts from /assets/fonts)
  src-tauri/      Rust core, a member of the root workspace (one Cargo.lock at the root)
    src/content.rs   read-only queries; opens through revelation_content::open_verified (DESIGN.md 8, 10.3)
    src/books.rs     book names, headings, abbreviations from data/kjv-book-names.tsv
    data/kjv-book-names.tsv  byte copy of the table in docs/scholar/kjv-book-names.md 6 (a test checks it)
    src/runs.rs      stored ranges -> ready-made runs for the WebView (DESIGN.md 5.1)
    src/user.rs      user.sqlite: the only writer (rollback journal, transactions, migration v1)
    src/kjv_tests.rs tests on the real KJV (full-Bible run check, stale hash, render golden)
    src/lex_tests.rs tests on the real lexicons (every entry == raw rows, missing/tampered file)
    src/fixture.rs   placeholder fixture builder (cfg(test) only, not in shipped binaries)
    tests/fixtures/  the committed fixture databases and placeholder-render.json
    tests/golden/    kjv-render.json, lexicon-entries.json: machine-generated goldens
```

| Task | Command (in `app/`) |
|---|---|
| Install | `npm install --ignore-scripts` (no package needs install scripts) |
| UI tests | `npm test` (through a small wrapper script) |
| Type check | `npm run check` |
| Rust tests | `cargo test -p revelation` (from anywhere in the repo; `cargo test --workspace` at the root runs all three crates) |
| Rebuild fixture | `cargo test -p revelation -- --ignored regenerate_fixture`, paste the printed hashes into `src/fixture.rs`, run it again to write the golden JSON |
| Rebuild KJV render golden | `cargo test -p revelation -- --ignored regenerate_kjv_render_golden` |
| Rebuild lexicon golden | `cargo test -p revelation -- --ignored regenerate_lexicon_golden` |
| ACL preflight | `npm run preflight` (report only, exit 0); `npm run preflight:test` (its tests, on a scratch folder in `%TEMP%`) |
| Release | `npm run release` = `preflight:release` (fails on any finding) + `tauri build --no-bundle` + `copy-content` |
| Release exe only | `npx tauri build --no-bundle` (skips the preflight; never bundle: the installers download tools to C:) |
| Release content | `npm run copy-content` |

The exe lands in the root `target/release/revelation.exe`; `copy-content` puts
`content/kjv.sqlite` and `content/content.sqlite` beside it.

## Release preflight: who can write what the build trusts (M0-F1)

`../scripts/preflight-acl.ps1` (Windows PowerShell 5.1) checks who can write what a
release build trusts. It is **read-only**: it calls
`Get-Acl` and never changes an ACL. It checks the repository root,
`sources/checksums`, `pipeline/tests/expected`, `pipeline/out`, `target`,
`target/release` (exe and `content/` included), `%CARGO_HOME%\bin`,
`%RUSTUP_HOME%`, the Node and Build Tools folders under `REVELATION_TOOLS_HOME` when it is set, the
`cargo.exe`/`rustc.exe`/`node.exe` that resolve on PATH, and the Git
installation that `git.exe` resolves to (it says whether that folder is on the
system PATH). Missing paths (for example `target/release` before the first
build) are skipped.

A finding is an Allow ACE, explicit or inherited (inherit-only ACEs too, since
they apply to every file inside), that gives WriteData/AddFile,
AppendData/AddSubdirectory, DeleteSubdirectoriesAndFiles, Delete,
ChangePermissions, TakeOwnership, GENERIC_WRITE or GENERIC_ALL (so Write,
Modify and FullControl) to anyone except the current user, SYSTEM,
Administrators or TrustedInstaller: for example `BUILTIN\Users`,
`NT AUTHORITY\Authenticated Users`, `Everyone`, `INTERACTIVE`, service
accounts or another user's SID. An owner outside that set is also a finding
(an owner can rewrite the ACL). `CREATOR OWNER` is not a finding: no token
contains that SID, so the ACE grants nothing on the object itself; Windows
turns it into a grant for whoever creates a child, and creating one already
needs a grant the script reports.

For each finding it prints `icacls` commands to review and run as
administrator: give the current user explicit full control (under UAC the
everyday shell doesn't use the Administrators group), stop inheritance at the
folder to protect when the grant comes from above it (for example a drive root, so the
rest of the drive is untouched), and `/remove:g` the offending SID. It groups
the fixes at the repository root, the tools folder and the Git installation root.

| Mode | Exit code |
|---|---|
| `npm run preflight` (warn) | 0, findings printed |
| `npm run preflight:release` / `npm run release` | 1 on any finding or unreadable ACL; the build doesn't start |
| same, with `REVELATION_ALLOW_UNSAFE_ACL=1` | 0, with a `!!` banner: local demo builds only; any other value is ignored |

`scripts/test-preflight-acl.ps1 [-Root <folder>]` creates a scratch folder,
changes only that folder's ACL (inherited and explicit Authenticated Users
Modify, read-only grants, CREATOR OWNER, single write rights for other
principals), checks the exit codes and output, checks the preflight leaves the
SDDL unchanged and that running the printed fix clears the finding, and
deletes the folder.

## Content: where it is and how it is checked

- **Expected hash, compiled in.** `build.rs` reads
  `pipeline/tests/expected/output-sha256.txt` (comment lines and blank lines
  skipped, every other line strictly `<64 hex>  <file>`; the parser is
  `revelation_content::hashfile`), and writes `KJV_SQLITE` and
  `CONTENT_SQLITE` into the binary (`revelation_core::expected`). A missing or
  malformed file, or a missing line, fails the build. `rerun-if-changed`
  means a new pipeline hash is picked up by the next build automatically. The
  app never reads the `.sha256` sidecar beside a database.
- **Location (KISS).** Debug builds (`cargo test`, `tauri dev`) read
  `pipeline/out/kjv.sqlite` and `pipeline/out/content.sqlite` in this
  repository. The release exe reads `content\` next to itself.
  `npm run copy-content` puts both files there, copying only if every source's
  SHA-256 equals the expected value (checked before any copy; it re-hashes the
  copies too). `npm run release` = `tauri build --no-bundle` + that copy.
- **Opening.** Only `revelation_content::open_verified` (plain file only, no
  links or junctions; read once, size-capped, SHA-256 against the compiled-in
  value, `sqlite3_deserialize` that buffer, read-only, `query_only`). If the
  hash differs, for example because the pipeline rebuilt the DB and the app was
  not rebuilt, the app shows the repair/reinstall message and no text
  (`kjv_tests::stale_or_wrong_hash_gives_the_repair_state_and_no_text`,
  `tests/kjv-app.test.ts`). A missing or failing `content.sqlite` never blocks
  the Bible: the Strong's panel says the lexicon is not available and why.
- **Render golden.** After the DB changes, regenerate with one command:
  `cargo test -p revelation -- --ignored regenerate_kjv_render_golden`, and
  review the diff. `kjv_render_golden_is_up_to_date` fails until you do.
- **Translation label.** The DB has no `translation_name` yet (DESIGN.md 8),
  so the reader-facing name comes from one cited constant,
  `content::TRANSLATION_LABELS` (`KJV` -> "King James Version (1769)",
  owner-approved, docs/scholar/kjv-label.md). It moves into the DB's
  `meta.translation_name` in the pipeline later; a DB value already wins. The
  abbreviation stays `KJV` (translation button, copied references). Book
  names, headings and abbreviations come from `data/kjv-book-names.tsv` (the
  scholar's cited table): arabic numerals ("1 Samuel"), and Psalms chapters
  headed "Psalm 23" (owner decisions, `books.rs`). The title above chapter 1
  is the DB's `book.main_title` as stored.

## About and credits

Menu > About Revelation shows the app's copyright and MIT licence and the
required attribution of every bundled source, as static text
(`src/lib/attributions.ts`). `tests/about.test.ts` reads `sources/SOURCES.md`
and fails if any line differs from its row (BIB-KJV-CW31, LEX-HEB-OSHB,
LEX-GRC-MGNT19, FONT-LITERATA, FONT-INTER). The lexicon credits are also shown
under the entries in the Strong's panel.

## User data: `user.sqlite` (not WAL; deviation from PLAN.md)

PLAN.md (sections 5 and 7) says "SQLite WAL" for user data. This was changed
on purpose (review item D5-F1): `user.sqlite` uses
`journal_mode = DELETE` with `synchronous = FULL`.

- **Why.** In WAL mode a commit stays in `user.sqlite-wal` until a
  checkpoint. The app's writes (a highlight, the position, settings) are a few
  hundred bytes, never enough to trigger one, and Tauri ends the process
  without closing the connection, so `user.sqlite` stayed a 4 KB empty shell
  and every highlight lived only in the `-wal`. Copying only `user.sqlite`,
  a sync or backup tool that skips `*-wal`, or one damaged byte in the `-wal`
  silently reset all highlights. With a rollback journal every commit is in
  `user.sqlite` before the call returns, and a crash (or `taskkill /F`) leaves
  at most a hot `-journal` that SQLite rolls back on the next open. The writes
  are tiny and rare, so WAL's reader/writer concurrency buys nothing here.
- **Upgrading a file from the earlier build.** `UserDb::open` reads the header
  before SQLite opens the file, opens it with checkpoint-on-close turned off,
  and runs the integrity, identity and version checks. Only if they all pass
  does it run `PRAGMA wal_checkpoint(TRUNCATE)` (every committed frame into the
  main file), then `PRAGMA journal_mode = DELETE` (which removes the emptied
  `-wal`). If either fails, nothing is migrated and the reader sees the
  "can't be saved" banner.
- **Refused, never migrated over.** A file whose header says WAL but whose
  main file has no schema (no tables, `user_version` 0) means the data was in a
  `-wal` that is missing or damaged. It is refused (`UserError::MissingWal`),
  and the main file and any `-wal` are left byte-identical; the same refusal
  happens on every start.
- **Way out (D5-F5).** The banner for a refused file offers "Start a new data
  file". It renames `user.sqlite` and any `-wal`/`-shm`/`-journal` to
  `user-kept-<unix seconds>.sqlite[...]` in the same folder (never deleted)
  and creates a new file. Messages name the file only, never the full path.

Tests: `user::tests` (`committed_data_is_in_user_sqlite_alone`,
`highlight_survives_without_a_clean_close`,
`wal_file_from_the_previous_build_is_converted_with_every_row`,
`empty_wal_shell_is_refused_not_migrated`,
`a_refused_file_is_kept_and_a_new_one_started`).

## Window state

`tauri-plugin-window-state` 2.5.0 (cleared in the project's internal review notes 9;
adds only itself and `tauri-plugin`) restores size, position and the maximised
state (PLAN.md 7: "saved and restored safely, clamped to screen"). The plugin
restores the position only when the saved rectangle intersects a connected
monitor, so a window saved on a monitor that is gone opens centred. It is
driven from Rust only (save on close, restore on window creation); no WebView
permission is granted, so `capabilities/main.json` is unchanged. It writes
`.window-state.json` into the app's config folder (`%APPDATA%\local.revelation.reader`).

The plugin restores the saved size as is, so `lib.rs::clamp_to_monitor` runs
right after it (D5-F3): a window that doesn't fit its monitor's work area
(saved on a bigger screen, or a tampered file) gets the default 1200 x 800,
or the work area if smaller, centred. A maximised window is left alone.

## WebView2 hardening (release)

- `main.rs` removes `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`,
  `WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER` and `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`
  from the process environment before anything starts (D5-F7), so a
  `--remote-debugging-port` set there no longer opens the DOM and IPC.
- Tauri 2.12 doesn't expose WebView2's `AreBrowserAcceleratorKeysEnabled` or
  `AreDefaultContextMenusEnabled` (only wry does), so `App.svelte` swallows the
  browser's keys (F5, Ctrl+R, Ctrl+P, Ctrl+S, Ctrl+F, F3, F7, Alt+Left/Right
  and the like) and the right-click menu outside text fields (D5-F4). Copy
  stays on Ctrl+C through the app's own handler. Switch to the native settings
  when Tauri exposes them.

## Fonts

Literata (reading) and Inter (UI) are the unmodified upstream variable TTFs in
`/assets/fonts` (see its README and SOURCES.md FONT-LITERATA / FONT-INTER).
`scripts/sync-fonts.mjs` checks them against `assets/fonts/fonts.sha256` and
copies them, with their licence files, into `public/fonts` (ignored) before
every dev/build; after `vite build` it checks `dist/fonts` again.
`src/styles/fonts.css` declares them; the CSP allows `font-src 'self'` only.
