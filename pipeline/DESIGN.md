# Pipeline design

- **Audience:** developers and reviewers of the pipeline
- **Date:** 2026-10-20
- **Status:** implemented. Revised after an internal review, for the lexicon importer as built (6.2, 7, 8, 9.2, 11),
  the commentary import and `mhcc.sqlite` (section 17), and the search index (section 18).
- **Based on:** [`docs/pipeline/kjv-osis-survey.md`](../docs/pipeline/kjv-osis-survey.md) (all numbers below come from it unless marked "survey probe"), PLAN.md section 2 (rules 1–8), SOURCES.md rows BIB-KJV-CW31, LEX-HEB-OSHB, LEX-GRC-MGNT19, VRS-ENG-PALASO.
- **Survey probes:** three read-only counts made the survey's way (scratchpad Python run with `py -I`, paths as arguments, nothing written near the sources): the event form (`Start`/`End` or `Empty`) of every element in the three XML files (section 9.4; the element totals agree with the survey), the line classes of `eng.vrs.txt` (section 9.2, T2), and the exact XML declaration bytes of each file (section 9.2).

Words used precisely below:
- **code point**: a Unicode scalar value (what Rust's `char` and Python's `str` index, and what SQLite's `length()`/`substr()` count on TEXT).
- **range**: a half-open interval `[start, end)` of code points in one verse's stored text.
- **fail closed**: stop the build with an error naming the file, byte offset and (if known) verse; write no output.

## 1. Scope

**M1 builds** the KJV (BIB-KJV-CW31) with verse text, supplied-word italics (`transChange`), divine-name small capitals (`divineName`), red letters (`q who="Jesus"`) and Strong's links (`strong:` lemma tokens), plus the two Strong's lexicons (LEX-HEB-OSHB, LEX-GRC-MGNT19), and checks verse counts against VRS-ENG-PALASO with the two KJV overrides.

**Wave 2 adds** Matthew Henry's Concise Commentary (CMT-MHCC-CW20) as its own database, `mhcc.sqlite` (section 17).

**Not imported in M1** (the review's element list, or out of scope): `morph`, `lemma.TR`, `<note>` (study notes, `rdg`, `catchWord`), pilcrow milestones, `inscription` markup (its text *is* imported, it is verse text), titles (chapter, psalm, acrostic; the book **main** titles are imported since D2, as `book.main_title`, section 8), colophons, the header, and the search index (FTS5 comes later; search-only normalisation lives only there). Open question 1 asks to add the canonical psalm titles, acrostic headings and colophons.

## 2. Stages

```
sources/raw/<ID>/<file>  ──► 1 verify ──► 2 parse ──► 3 validate ──► 4 write ──► 5 check ──► 6 publish
sources/checksums/<ID>.sha256  (bytes in memory) (typed model)     (temp file)  (temp file, (rename DB,
                                                                                 in memory)  then .sha256)
```

1. **verify.** For each source the build needs, the pipeline reads `sources/checksums/<ID>.sha256` (one line: 64 lowercase hex digits, two spaces, file name, LF; anything else fails closed), applies the path rules (section 9.5), checks the file's size against the cap (9.1), reads the whole file into memory once, computes SHA-256 over those bytes and compares. Every later stage works on **that same buffer**, so the bytes parsed are the bytes hashed (no re-open, no time-of-check/time-of-use gap). A CI test also checks that each checksum appears in its SOURCES.md row, so the two can't drift.
2. **parse.** Pure functions from bytes to a typed model (`KjvBook`/`Verse`/`Mark`/`RedPiece`/`Word`, `LexEntry`). No I/O, no network, no globals. One parser per source format: KJV OSIS, OSHB lexicon, MGNT lexicon, `.vrs`.
3. **validate.** Every invariant in sections 3–7 and 9, the versification check, and the anomaly list. All errors are collected per file and reported together (bounded to the first 100), then the build stops.

**Stages 4–6, write, check and publish, are one rule** (M1-F1, M1-F2). The destination (`<dest>` = `kjv.sqlite` or `content.sqlite` in `<out-dir>`) changes only after the exact bytes that will sit there have passed every check. In order:

| Step | What | On failure |
|---|---|---|
| 0 | **Clean up.** At start-up, `build` deletes leftovers of earlier crashes in `<out-dir>`: names matching exactly `<dest>.tmp-<16 hex>` or `<dest>.sha256.tmp-<16 hex>`, each optionally followed by `-journal`, `-wal` or `-shm`. Nothing else is ever deleted. | stop |
| 1 | **Create the temp file** `<dest>.tmp-<16 hex>` in `<out-dir>` with `OpenOptions::create_new` (O_EXCL / `CREATE_NEW`), so an existing or planted file is never reused or followed. The 16 hex digits come from the process ID and the clock (std only); they must be unique, not secret, because `create_new` refuses any existing name. The empty file is then opened by SQLite as a new database. | delete temp, stop |
| 2 | **Write.** Before any table: `PRAGMA journal_mode = DELETE` (the returned mode is read back and must be `delete`; WAL is never used), `PRAGMA synchronous = FULL`, `PRAGMA foreign_keys = ON` (M1-F12), then the section 8 conventions. One transaction inserts everything in document order and commits. Then `PRAGMA integrity_check` (exactly one row, `ok`), `PRAGMA foreign_key_check` (zero rows), `VACUUM`, and `Connection::close` (its error is checked). | delete temp, stop |
| 3 | **No sidecars, then flush.** Assert that `<temp>-journal`, `<temp>-wal` and `<temp>-shm` don't exist, then open the temp file for writing (no truncate) and call `File::sync_all` (Windows needs write access for `FlushFileBuffers`). | delete temp, stop |
| 4 | **Check the temp file** (stage 5 below). | delete temp, stop |
| 5 | **Write the checksum** from the SHA-256 of the bytes just checked (not a re-read): `<dest>.sha256.tmp-<16 hex>`, created with `create_new`, same one-line format as `sources/checksums/*.sha256`, then `sync_all`. | delete both temps, stop |
| 6 | **Publish.** Refuse (do not delete) if `<dest>-wal`, `<dest>-shm` or `<dest>-journal` exists: the error names the file for a person to remove. Rename the temp DB to `<dest>`, then the temp checksum to `<dest>.sha256`. On Linux, `sync_all` the directory after the renames. | delete both temps, stop |

Notes on step 6:
- On Windows `std::fs::rename` is `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`. It fails if `<dest>` is open (the app, a `sqlite3` shell); that failure leaves the old `<dest>` and the old `<dest>.sha256` untouched, because the checksum is renamed only after the DB rename succeeded.
- A crash between the two renames leaves a new, fully checked `<dest>` next to the old `<dest>.sha256`. That mismatch fails closed (`check` reports it; the app never reads the sidecar anyway, section 10.2), and the next `build` replaces both.
- A failed build never leaves anything at `<dest>` that didn't pass stage 5, and on any failure the previous good build stays byte-identical (tested by T21).

**Stage 5, check, in detail** (step 4 above; also the `check` subcommand on a published file). It reads the whole file into memory (size cap 256 MiB), computes its SHA-256, and opens **that buffer** as a read-only in-memory database with `sqlite3_deserialize` (rusqlite `serialize` feature, read-only flag) plus `PRAGMA query_only = 1`: the same `open_verified` function the app core will use (section 10.3). An in-memory database has no `-wal`, `-shm` or `-journal`, so nothing beside the file can change what is checked, and the hash covers exactly the bytes checked (M1-F1). It then requires: header bytes 18 and 19 both `0x01` (rollback journal, never WAL); `sqlite_schema` equal to the pinned schema text of section 8; `PRAGMA integrity_check` = `ok`; `PRAGMA foreign_key_check` empty; the section 8 queries empty; and the round-trip and totals checks of section 11 against the parsed model. The `check` subcommand also fails if `<dest>.sha256` doesn't match or a sidecar of `<dest>` exists. If `serialize` turns out not to be usable in Phase B, the fallback is `file:<temp>?mode=ro&immutable=1` (`SQLITE_OPEN_URI`), with the hash taken from the same read and the no-sidecar assertion of step 3.

CLI (std only, no argument-parsing crate): `revelation-pipeline verify <sources-dir>`, `build <sources-dir> <out-dir>`, `check <sources-dir> <out-dir>`. `build` runs all six stages.

### 2.1 Verify-only, not fetch

The pipeline **does not download anything**. It verifies files that a person (or the CI workflow, with `curl` from the pinned URL in SOURCES.md) placed in `sources/raw/<ID>/`. Reasons:

- **The hash is the integrity guarantee, not the transport.** A pinned SHA-256 makes the file's origin irrelevant: a fetch step adds convenience but no safety. With verify-only, a tampered or truncated file is refused exactly as a fetched one would be.
- **Smaller attack surface in the tool that parses untrusted data.** No URL handling, redirects, proxies, TLS, certificate store or partial downloads inside the pipeline. The parser never sees anything that hasn't passed the hash check.
- **Determinism and offline builds.** The same inputs give the same output with the network unplugged (PLAN.md rule 2). A flaky mirror can't change a build, only stop the download step.
- **KISS and licences.** No HTTP/TLS crates (ureq, rustls, ring, webpki roots), so no extra ISC, BSD-3-Clause or CDLA notices and nothing more to keep patched.

The cost is one manual step. To make it painless, the README of `sources/` (or a small script outside the pipeline) lists one `curl -fL -o <path> <pinned URL>` line per source; CI runs the same lines and caches `sources/raw/`.

A new pin is accepted only after checking that its commit is reachable from the publisher's default branch, because `raw.githubusercontent.com/<owner>/<repo>/<sha>/…` also serves commits that exist only in a fork (M1-F16; all four current pins pass). The wording of that rule belongs in SOURCES.md.

## 3. Stored verse text

### 3.1 Definition

> A verse's stored text is the concatenation, in document order, of every character-data node that lies after the verse's `<verse sID="X"/>` milestone and before its `<verse eID="X"/>` milestone, **except** nodes inside a `<note>` subtree. The bytes are copied unchanged. Attribute values contribute nothing.

That is all. In particular the pipeline does **not**:
- trim, collapse, add or translate whitespace;
- apply any Unicode normalisation, case change (`divineName` stays `Lord`), quote or dash conversion;
- insert anything for markup: no pilcrow for `<milestone marker="¶"/>`, no quotation mark for `<q marker="">` (a non-empty `marker` fails closed, because using it would mean generating characters), no space where a note was removed;
- decode entities (the three XML inputs contain no `&`; any `&` in an XML input fails closed, section 9.2).

Because the KJV file contains no `&` and no CR, "character data" here is simply the raw bytes between tags (no end-of-line handling arises).

Character data **between** verses (after an `eID` and before the next `sID`) belongs to no verse and is dropped. In this file it is a single `\n` every time (survey section 4); the validator checks that anything there is whitespace or inside a known non-verse element (title, colophon), and fails closed on stray text.

### 3.2 Whitespace rules (validated, fail closed)

Inside a verse, whitespace between elements is ordinary character data, so `</w> <w>` contributes its space and `</w>, <q …>` contributes `, `. The survey found these properties, and the validator requires them for the KJV (a different source gets its own profile):

| Rule | KJV value |
|---|---|
| V1. Stored text is non-empty and at most 4,096 code points | shortest 11 (John 11:35), longest 528 (Esth 8:9) |
| V2. The only whitespace character is U+0020 (no TAB, LF, CR, NBSP or other `White_Space`) | 758,617 spaces, nothing else |
| V3. No leading space | 0 verses |
| V4. No two consecutive spaces (checked after notes are removed) | 0 verses |
| V5. Trailing space allowed, and the count is a pinned fact | **91 verses** (survey anomaly 2) |
| V6. No C0 or C1 control, U+FEFF, U+FFFD, U+2028/2029, or bidi control (U+202A–202E, U+2066–2069) | none |
| V7. Every non-ASCII character is in the source profile's allowlist | `’` U+2019, `–` U+2013, `—` U+2014, `Æ` U+00C6 |

V5 needs a word. The 91 trailing spaces are in the source's verse content (for example Matt 8:9 ends `…</transChange>. <verse eID="Matt.8.9"/>`), so under the no-trimming rule they are stored. The renderer's normal CSS whitespace handling makes them invisible; copy/paste and Anki export copy them as stored. If the owner prefers trimming, that is an explicit, documented exception to rule 2.4/2.5, not something the pipeline does quietly (open question 4).

V7 is the cheapest guard against mojibake or a wrong file: any character the profile hasn't seen stops the build for a person to look at.

### 3.3 Elements allowed inside a verse

| Element | Contributes text | Stored as | Must be |
|---|---|---|---|
| `w` | yes | Strong's word link (section 6) | has `lemma`; not nested in another `w` |
| `transChange` | yes | mark `added` (section 4) | `type="added"` (all 21,571 in verse text) |
| `divineName` | yes | mark `divine` | no attributes |
| `inscription` | yes | nothing in M1 (text is already capitals) | no attributes |
| `q` | yes | red pieces (section 5) | exactly `who="Jesus"` and `marker=""` |
| `milestone` | no (empty) | nothing in M1 | `type="x-p"`, `marker="¶"`, optional `subType="x-added"`, empty |
| `note` | **no** (whole subtree skipped) | nothing in M1 | `type="study"`; inside it only `catchWord`, `rdg`, `transChange`, `divineName` |

Any other element, attribute or attribute value inside a verse fails closed. `xml:lang` or similar attributes the survey didn't see in verses are not allowed until a profile says so. Each element's form is fixed too (section 9.4, M1-F3): `verse` and `milestone` only as `Empty`, `w` as either, everything else in this table only as a `Start`/`End` container, so an empty `<q …/>`, a milestone-form `q` or a container `<verse>…</verse>` fails closed.

`w` may carry `lemma` (required), `morph` and `src` (both ignored in M1) and `type` matching `x-split-<digits>` (ignored; it links the two halves of a split word, which keep their own lemmas).

## 4. Presentation-only ranges

Presentation never changes characters (PLAN.md 2.5). The pipeline records *where* presentation applies, as ranges over the stored text:

| `mark.kind` | Source | Renderer shows | KJV count |
|---|---|---|---|
| `added` | `transChange type="added"` | italics (supplied words) | 21,571 |
| `divine` | `divineName` | small capitals (`Lord` → LORD, `God` → GOD) | 6,887 |

Red letters are not a mark; they have their own table with stricter checks (section 5).

Rules (validated):
- P1. A mark's range is the code-point extent of the element's whole text content, including nested elements (`transChange` can contain `w` and `divineName`; `w` can contain `divineName`).
- P2. Marks are never empty (the survey found no empty `transChange` or `divineName`) and never cross a verse boundary (none is open at any verse milestone).
- P3. Marks may nest or touch; each is independent. Within one verse they are stored in document order of their start tags.
- P4. A mark is either entirely inside one red piece or disjoint from all red pieces. XML nesting guarantees this for one verse (`q` and `transChange` nest properly), and the validator checks it so the renderer can build a clean element tree.

Not stored in M1: `inscription` (its text is already in capitals), pilcrows, notes, titles.

## 5. Red letters

### 5.1 Offset unit: code points

Every red range (and every mark and word range) is stored as **code-point offsets** into the stored verse text, half-open `[start, end)`.

| Unit | For | Against |
|---|---|---|
| **Code points** (chosen) | SQLite's `length()` and `substr()` count code points on TEXT, so anyone can audit the database with the stock `sqlite3` shell: `substr(text, start + 1, end - start)` *is* the red text. Every integer is a valid boundary in every encoding, so a stored offset can never split a UTF-8 sequence or a surrogate pair; a wrong offset gives wrong text that the round-trip catches, never an invalid string. Rust (`char_indices`) and Python (`str`) index it natively, so the pipeline and the independent Python oracle (section 11) agree without conversion. Independent of any one consumer's string type. | JavaScript strings index UTF-16, so offsets can't be used with `String.slice` directly when a character is outside the BMP. |
| UTF-16 code units | Native to the WebView: `String.slice`, DOM `Range` and the CSS Custom Highlight API all use them. | Ties stored content to one consumer's encoding. Rust and SQLite don't index it, so the pipeline, the audit queries and the round-trip all need a conversion. An offset can point into the middle of a surrogate pair. |
| UTF-8 bytes | Native to Rust `str` and to the file. | SQLite `substr()` on TEXT disagrees with it; an offset can point mid-character; the renderer would need `TextEncoder` arithmetic. |

**How the WebView avoids the mismatch.** The renderer never does offset arithmetic. The Rust core turns a verse into a small tree of runs before it crosses the IPC boundary: `[Text("And Jesus … him, "), Red(1, [Text("Suffer "), Added("it to be so"), Text(" now: … righteousness.")]), Text(" Then he suffered him.")]`. The UI turns each `Red` into one `<span class="wj">` and each run into a text node or a styled span. The red-letter toggle only switches a class on the container. So the UTF-16 question never reaches the UI. If M2 ever needs UTF-16 offsets (for example the Custom Highlight API), one Rust function converts code points to UTF-16 units, tested with an astral-plane fixture; the stored data stays in code points.

For this KJV file code points and UTF-16 units are equal (no character outside the BMP) and UTF-8 bytes differ wherever `’`, `–`, `—` or `Æ` occurs. Because of that, a test fixture with an astral character (for example U+1D400) and a combining mark is required, or a units mix-up would pass every KJV test.

**B1. No boundary inside a grapheme (every profile, M1-F9).** Every integer is a valid boundary in every *encoding*, but not for *rendering*: a boundary between a base letter and its combining marks would split one grapheme across two spans (detached Hebrew points, dotted circles, odd selection edges). So no start or end of a red piece, mark or word, other than 0 and the verse length, may fall immediately before a character of General_Category `M*` (`Mn`, `Mc`, `Me`), or immediately before or after U+200D ZERO WIDTH JOINER. No Unicode crate is needed: the profile's character allowlist (V7) records the category of each allowed character (taken from the pinned Unicode data when the first profile with marks arrives). The KJV allowlist has no `M*` character and no U+200D, so B1 holds trivially there; T12 has a fixture with a boundary placed before a combining mark, which must fail closed.

### 5.2 From `q` elements to per-verse pieces

A `q` element may cover several verses (280 do; the longest covers 48). The database stores one **piece** per (q, verse) pair.

**Which events count as a `q` (M1-F3).** The event loop dispatches on the pair (event kind, full element name), never on the name alone (section 9.4). For `q` in the KJV profile:

| Event | Action |
|---|---|
| `Start` `q`, attributes exactly `who="Jesus"` and `marker=""` | opens a span (below) |
| `End` `q` | closes the open span |
| `Empty` `q` (`<q …/>`), in any form | **fail closed** |
| any `q` carrying `sID` or `eID` (milestone form) | **fail closed**, until a profile explicitly supports milestone `q` with its own algorithm, tests and pinned facts |

The Python oracle applies the same table: every `empty` token is checked against the profile's allowlist of empty forms and anything else raises. Otherwise an `Empty` `q` could be silently ignored by both implementations, the oracle and the parser would agree on "no red", and a whole imported translation's red letters could vanish with every check green. The pinned KJV has none (all 648 `q` are containers; survey section 5, survey probe), so these rules cost nothing for M1 and protect the import path.

```
for each <q> container in document order (q_ord = 1, 2, …):
    for each verse V the q touches:
        start = (q's start tag is inside V) ? offset of the start tag in V's stored text : 0
        end   = (q's end tag is inside V)   ? offset of the end tag in V's stored text   : length(V)
        store (V, start, end, q_ord)
```

Offsets are measured in the stored text as defined in section 3, so a note before the `q` doesn't shift it (notes contribute nothing), and nothing outside verses counts.

`q_ord` keeps the source's span identity: a cross-verse span is still one span, its pieces share a `q_ord`, and the round-trip compares the grouping too. No piece is merged with, extended into or trimmed against anything else.

The KJV gives 648 `q` → **2,035 pieces** in 2,026 verses.

### 5.3 Red-span invariants (validated, fail closed)

| Rule | Why | KJV value |
|---|---|---|
| R1. Pieces are non-empty (`start < end`) and inside the verse (`end ≤ length`) | an empty piece would mean a span starting just before `eID` or ending just after `sID`; the design doesn't guess which verse owns it | 0 empty |
| R2. `q` never nests, and never occurs in a note, between verses, or across a chapter boundary (KJV profile; a later profile may allow chapter crossing, the algorithm already does) | nested or stray spans have no defined meaning here | 0 / 0 / 0 / 0 |
| R3. Pieces in one verse don't overlap and are separated by at least one character | the renderer's one-span-per-piece mapping stays exact | 9 verses with 2 pieces, all separated by black text |
| R4. Every boundary is at a verse edge or next to U+0020 (KJV profile) | catches a shifted offset at once, cheaply | 612 + 36 starts, 111 + 537 ends |
| R5. No piece starts or ends with whitespace (KJV profile) | same | 0 |
| R6. The pieces of one `q_ord` lie in consecutive verses; every piece except the first starts at 0 and every piece except the last ends at `length` | the pieces must reassemble into the source span | holds |

R4 and R5 are facts of this source, not general rules: the KJV profile asserts them; a future source that breaks them fails closed so a person decides.

R1–R6 are checked on the parsed model (stage 3). R1, R3 and R6 are **also** run as SQL on the built database in stage 5 (section 8), so a writer bug is caught independently of the model it was written from (M1-F12). B1 (section 5.1) applies to red pieces too.

### 5.4 Round-trip and totals

- **Per verse:** the list of `(start, end, q_ord)` read back from the database equals the list computed from the source, exactly (count, start, end and grouping), for all 31,102 verses, including the 29,076 with none.
- **Per book:** number of `q` (by the book of the start tag), number of pieces, and red characters (`Σ (end − start)`, code points) equal the source and equal the pinned facts from the independent survey (Matt 146 / 646 / 69,530; Mark 131 / 285 / 27,482; Luke 185 / 589 / 62,696; John 153 / 422 / 39,461; Acts 21 / 28 / 2,479; 1Cor 2 / 2 / 181; 2Cor 1 / 1 / 77; Rev 9 / 62 / 8,596; all others 0; total 648 / 2,035 / 210,502).
- **Per `q`:** each `q_ord`'s pieces, read back from the database and joined in verse order, equal the oracle's per-`q` record in length and SHA-256 (`kjv-q.tsv`, section 11.1). That record is computed by a structurally different method (tag stripping), so it also catches a merge plus a split in one book, which per-book totals cancel out (M1-F4).
- **Mutation tests** prove the checks bite: fixtures that drop a span, add one, shift it by ±1, extend or trim one boundary, split one piece in two, merge two (at a verse edge and mid-verse), move a piece to the neighbouring verse, or let a span leak one character into the next verse each make the build fail, and each test asserts **which** check fires (T11).

## 6. Strong's links and key normalisation

### 6.1 Word links

Each `w` in verse text becomes one **word** row: its range (the code-point extent of its whole text content) and its ordered list of Strong's keys, one per `strong:` token in `lemma`, in token order.

- 6,331 `w` are empty (`<w …/>`, survey anomaly 3). They are stored as zero-length ranges (`start = end`) at their position, so "find every verse with G3588" sees them, as the source tags them. They can't be hovered. Zero length is allowed for words only, never for marks or red pieces.
- 18,142 `w` carry two or more keys (Gen 1:1 "created" = H853 + H1254); all are kept, in order.
- `lemma` is split on single U+0020 bytes only (never `split_whitespace`); an empty token (two spaces, or a leading or trailing space) or any other whitespace fails closed (M1-F15).
- `lemma.TR:` tokens are skipped (excluded in M1). Every other token must be `strong:` + key; anything else fails closed unless it is on the anomaly list (6.3).
- `w` in titles, colophons and notes are not imported (their text isn't).

### 6.2 One canonical key

The three sources spell the same number three ways. Only the **key** is normalised; display text never is (the lexicon's own `<strongs>1</strongs>` text, its headwords and every verse character are stored as the source has them).

| Source | Spelling | Example | Parser (in `src/strongs.rs`) |
|---|---|---|---|
| KJV `lemma` token | `strong:` + `H0`/`G` + digits; Hebrew always has one leading zero, Greek none | `strong:H07225`, `strong:H0430`, `strong:G1` | `StrongsKey::from_osis_lemma` |
| HebrewStrong.xml `entry/@id`, `w/@src` | `H` + digits, unpadded | `H1`, `H24` | `StrongsKey::from_prefixed` |
| strongsgreek.xml `entry/@strongs` | five digits, zero-padded, no letter | `00001` | `StrongsKey::from_digits(Greek, …)` |
| strongsgreek.xml `strongsref` | `language="GREEK"`/`"HEBREW"` + digits, Greek unpadded, Hebrew padded | `427`, `0175` | `StrongsKey::from_strongsref` |

**Canonical form:** letter + number without leading zeros: `H7225`, `H430`, `H1`, `G1`. Stored as TEXT. The grammar is strict (1–5 ASCII digits, value ≥ 1, nothing else; `H1234a` is rejected, not truncated) and its errors carry no input text.

**Per-source spelling (M1-F15).** `strongs.rs` accepts every padding (`strong:H7225`, `strong:H00430`, `strong:G01` all parse), which is right for a shared grammar but more lenient than any one source. So `profile.rs` also asserts each source's exact spelling from the table above, and a different or garbled spelling fails closed. For the KJV: Hebrew tokens are `strong:H0` + a number with no further leading zero (exactly one leading `0`, at most 5 digits in all), Greek tokens are `strong:G` + a number with no leading zero (at most 4 digits). Each lexicon's spellings are pinned the same way (OSHB `entry@id` and `w@src`: `H` + unpadded; MGNT `entry@strongs` exactly five digits; `strongsref` Greek unpadded, Hebrew `0` + unpadded). Four Hebrew `strongsref` values are unpadded in the source (D2 probe 2026-10-22: in entries G25 `5689`, G137 `5869`, G3160 `4992`, G4460 `7343`; the survey's shape table counts them under `HEBREW 9999`, its prose says "zero-padded"). They are kept as the source has them and listed in `lexicon.rs` like the anomaly list: keyed by entry and exact value, each must match exactly once, and any other unpadded Hebrew reference fails closed.

Checks: every KJV key resolves to a lexicon entry of the same language, and none to a Greek "Not Used" placeholder (all 14,089 distinct keys of the whole file do, survey section 6; verse text, which is what is imported, has 373,501 key links and 14,075 distinct keys, the other 14 occur only in psalm titles and colophons; D2 build: 0 unresolved). A failure lists every unresolved key with its number of links and stops the build; nothing is invented or fixed; every lexicon cross-reference resolves; Greek keys occur only in the NT and Hebrew keys only in the OT (KJV profile).

### 6.3 Known source anomalies

Anything the grammar rejects stops the build, except entries in a short, reviewed list in the pipeline source, each keyed by verse **and** the exact attribute value (so it can't match anything else) and each with one fixed action:

| Verse | Exact `lemma` value | Action | Why |
|---|---|---|---|
| 2Cor.8.10 | `strong:G1722 G5129 lemma.TR:εν lemma.TR:τουτω` | skip the token `G5129`; keep `strong:G1722` | the token has no `strong:` prefix (survey anomaly 1). The pipeline doesn't guess what a malformed token means |

Every list entry is printed in the build report, recorded in SOURCES.md and worth reporting upstream. A list entry that no longer matches anything also fails the build, so the list can't go stale.

## 7. Lexicons

Both lexicons are stored as **the entry's own XML tree, made safe and typed**, not as rendered HTML and not as re-worded fields. The UI decides how each element looks (presentation); the characters are the source's.

- One `lex_entry` row per entry: canonical key, lexicon ID, document order.
- One `lex_node` row per element or text node inside the entry, in document order, with its parent: element nodes carry the element name; text nodes carry their bytes **exactly**, including the hard line feeds in 2,900 Greek `strongs_def` texts, leading spaces, and whitespace-only nodes.
- One `lex_attr` row per allowed attribute. Display text that lives in attributes (`greek@unicode`, `greek@translit`, `pronunciation@strongs`, Hebrew `w@xlit`, `w@pron`) is stored exactly. Cross-reference attributes (`w@src`, `strongsref@language` + `@strongs`) also get the canonical target key in `lex_attr.ref_key`.

Allowed elements and attributes (anything else fails closed):

| Lexicon | Elements under `entry` | Attributes |
|---|---|---|
| OSHB | `w`, `source`, `meaning`, `usage`, `def`, `note` | `entry@id`; `w@pos`, `w@pron`, `w@xlit`, `w@xml:lang`, `w@src` |
| MGNT | `strongs`, `greek`, `pronunciation`, `strongs_derivation`, `strongs_def`, `kjv_def`, `latin`, `strongsref` | `entry@strongs`; `greek@BETA`, `greek@unicode`, `greek@translit`; `pronunciation@strongs`; `strongsref@language` (`GREEK`/`HEBREW`), `strongsref@strongs` |

Rules:
- Text is raw bytes; no end-of-line normalisation (quick-xml's content getters would turn OSHB's 51,928 CRs into LFs, so they are forbidden, section 9.3; T14 asserts that entry H1's whitespace nodes still contain `\r\n\t`). CR occurs only in OSHB's whitespace-only nodes between an entry's children (CRLF + tabs); the validator rejects CR anywhere else. The UI ignores whitespace-only nodes that are direct children of `entry` when it lays out the fields (presentation).
- MGNT: the 324 entries with text directly inside `<entry>` keep it as text nodes in place (it is content, for example entry 1's "Often used (usually …").
- The 101 Greek "Not Used" placeholders are stored like any other entry (they are in the source); nothing links to them.
- `<strongs>` text must equal the entry's number (checked, survey: always true); `entry@id` / `entry@strongs` are unique and ascending (checked).
- OSHB `<note>` elements are OSHB's documented corrections and are part of the lexicon text; they are kept.
- An empty reference element (`strongsref`, or MGNT `pronunciation` whose text is in an attribute) shows text the UI builds from stored data (the canonical key or the attribute value). Like verse numbers, that is presentation, not content.
- Prologue (MGNT) and namespace/schema attributes are not imported; credits come from SOURCES.md.

Round-trip: rebuilding each entry from the three tables gives exactly the parsed tree (names, attributes, text bytes, order), and the concatenation of an entry's text nodes equals the entry's character data in the source.

**As built (D2, `src/lexicon.rs`).**
- *Elements and parents.* Allowed only where the survey saw them. OSHB: `w` in `entry`, `source`, `meaning`, `usage`; `source`, `meaning`, `usage` in `entry`; `def` in `source`, `meaning`; `note` in `entry`, `source`, `meaning`. MGNT: `strongs`, `strongs_derivation`, `strongs_def`, `kjv_def` in `entry`; `greek`, `pronunciation` in `entry`, `strongs_derivation`, `strongs_def`; `latin` in `strongs_derivation`, `strongs_def`; `strongsref` in those three and `kjv_def`. Required attributes as the MGNT DTD declares them (`greek`: all three; `pronunciation@strongs`; `strongsref`: both). Outside entries: OSHB `<lexicon>` with exactly its three pinned namespace/schema attributes (inert strings), MGNT `strongsdictionary` → `prologue` then `entries`, once each, no attributes; only whitespace text, except the MGNT prologue (not imported).
- *Characters.* Text and attribute values: printable ASCII plus a pinned list of letter, mark and punctuation ranges (Latin-1 letters, Latin Extended-A/B and Additional, IPA and modifier letters, combining diacritics, Greek and Greek Extended, Hebrew points and letters, phonetic extensions, U+2010, U+2014, U+2019, U+2026). That excludes C1 controls, NBSP, soft hyphen, bidi and zero-width format characters, U+2028/2029, BOM, U+FFFD and private use. LF is allowed in text. CR and TAB only in OSHB whitespace-only nodes directly inside `<entry>` (and between entries); OSHB must be CRLF throughout (51,928 pairs, no bare CR or LF). Attribute values have no TAB, CR or LF.
- *Entry attribute.* `entry@id` / `entry@strongs` is checked (pinned spelling, unique, ascending, MGNT `<strongs>` text = the number) but not stored: under the pinned spelling it is a function of `lex_entry.key`, and T14 rebuilds it from the key.
- *Attribute order* is stored (`lex_attr.i`), so an entry can be rebuilt byte for byte: T14 rebuilds all 14,298 entries from the database alone (attributes in stored order, double-quoted; an element without child nodes in the empty form) and compares them with the source bytes at each entry's offset. All are identical.
- *References.* `lex_attr.ref_key` is set on `w@src` (OSHB) and on `strongsref@strongs` (MGNT; NULL on `@language`). `pronunciation@strongs` is display text, not a reference. Every reference resolves to an entry of its language that is not a placeholder.
- *Placeholders.* An MGNT entry whose only element is `<strongs>` and whose other text is `Not Used` (spaces and LFs around it) is a placeholder; the set must be exactly the pinned 101 (2717, 3203–3302). They are stored like any other entry; `lexicon.rs` marks them in the model only (T13, reference checks).
- *Pinned facts* (survey 9.1, 9.2): entries 8,674 / 5,624; element counts per lexicon; `w@pos` 8,674, `w@pron` 9,850, `w@xlit` 9,850, `w@xml:lang` 8,674, `w@src` 7,934; `greek@*` 6,139 each, `pronunciation@strongs` 5,652, `strongsref@language` GREEK 6,746 / HEBREW 317; 324 MGNT entries with text directly in `<entry>`. A fact that no longer holds fails the build.

## 8. Content schema

Two output files with the same conventions. The KJV (GPL) goes in its own read-only file; that also matches PLAN.md's "translations are packs", so the KJV becomes the first pack:

- `kjv.sqlite`: the KJV (tables `meta`, `source`, `book`, `verse`, `mark`, `red_piece`, `word`, `word_key`);
- `content.sqlite`: the lexicons (tables `meta`, `source`, `lex_entry`, `lex_node`, `lex_attr`).

If the owner wants one file instead, the same tables go into one database unchanged. The app opens both read-only and joins `word_key.key` to `lex_entry.key` across `ATTACH`; foreign keys can't cross files, so the pipeline's check stage verifies that join instead.

Conventions: `STRICT` tables (SQLite ≥ 3.37; the bundled library is 3.54), `PRAGMA encoding = 'UTF-8'`, `page_size = 4096`, `auto_vacuum = NONE`, `journal_mode = DELETE` (never WAL; header bytes 18/19 = `01 01`), `application_id = 0x52564C4E` ("RVLN"), `user_version` = schema version (1). Offsets are code points and the column names say so (`start_cp`, `end_cp`). SQLite ignores `REFERENCES` unless `PRAGMA foreign_keys = ON`, so the build turns it on and the check stage runs `PRAGMA foreign_key_check` (M1-F12). The schema below is one SQL constant in `db.rs`; the check stage compares `sqlite_schema` with it exactly, and the app later does the same for packs (section 10.3).

```sql
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
-- kjv.sqlite: schema_version, translation_abbrev = 'KJV', has_red_letters = '1',
-- red_letter_source = 'BIB-KJV-CW31', versification = 'VRS-ENG-PALASO + KJV overrides',
-- versification_overrides_sha256. content.sqlite: schema_version only.
-- Nothing from the build environment (commit, time, host) is stored: each file is a pure
-- function of the pinned inputs and the pipeline code (10.1, PM decision 2026-10-22).
-- Reader-facing labels (name, date) arrive with the first-run work (M2), from the scholar's
-- kjv-label.md, never hard-coded.

CREATE TABLE source (
  id     TEXT PRIMARY KEY,                                  -- SOURCES.md ID, e.g. 'BIB-KJV-CW31'
  file   TEXT NOT NULL,
  size   INTEGER NOT NULL,
  sha256 TEXT NOT NULL CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*')
) STRICT, WITHOUT ROWID;

CREATE TABLE book (
  ord      INTEGER PRIMARY KEY CHECK (ord BETWEEN 1 AND 66),
  osis     TEXT NOT NULL UNIQUE,                            -- 'Gen' (as in the source)
  usfm     TEXT NOT NULL UNIQUE,                            -- 'GEN' (as in eng.vrs)
  chapters INTEGER NOT NULL CHECK (chapters >= 1),
  main_title TEXT CHECK (main_title IS NULL OR length(main_title) BETWEEN 1 AND 256)
                                                            -- the main title element, raw character data
) STRICT;

CREATE TABLE verse (
  id      INTEGER PRIMARY KEY,                              -- book*1000000 + chapter*1000 + verse
  book    INTEGER NOT NULL REFERENCES book(ord),
  chapter INTEGER NOT NULL CHECK (chapter BETWEEN 1 AND 999),
  verse   INTEGER NOT NULL CHECK (verse BETWEEN 1 AND 999),
  text    TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 4096),   -- stored text, section 3
  CHECK (id = book * 1000000 + chapter * 1000 + verse),
  UNIQUE (book, chapter, verse)
) STRICT;

CREATE TABLE mark (                                         -- section 4
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),                 -- document order within the verse
  kind     TEXT NOT NULL CHECK (kind IN ('added', 'divine')),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE red_piece (                                    -- section 5
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  q_ord    INTEGER NOT NULL CHECK (q_ord >= 1),             -- which <q> in the source, 1-based
  CHECK (0 <= start_cp AND start_cp < end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX red_piece_by_q ON red_piece (q_ord, verse_id);

CREATE TABLE word (                                         -- section 6; zero length allowed
  verse_id INTEGER NOT NULL REFERENCES verse(id),
  n        INTEGER NOT NULL CHECK (n >= 0),
  start_cp INTEGER NOT NULL,
  end_cp   INTEGER NOT NULL,
  CHECK (0 <= start_cp AND start_cp <= end_cp),
  PRIMARY KEY (verse_id, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE word_key (
  verse_id INTEGER NOT NULL,
  n        INTEGER NOT NULL,
  k        INTEGER NOT NULL CHECK (k >= 0),                 -- token order within the lemma
  key      TEXT NOT NULL CHECK (key GLOB '[HG][1-9]*' AND key NOT GLOB '?*[^0-9]*'
                                AND length(key) BETWEEN 2 AND 6),   -- canonical, section 6.2
  PRIMARY KEY (verse_id, n, k),
  FOREIGN KEY (verse_id, n) REFERENCES word (verse_id, n)
) STRICT, WITHOUT ROWID;
CREATE INDEX word_key_by_key ON word_key (key, verse_id);

-- content.sqlite (after its own copies of meta and source, as above)
CREATE TABLE lex_entry (
  key     TEXT PRIMARY KEY CHECK (key GLOB '[HG][1-9]*' AND key NOT GLOB '?*[^0-9]*'
                                  AND length(key) BETWEEN 2 AND 6),   -- canonical: 'H1', 'G1'
  lexicon TEXT NOT NULL CHECK (lexicon IN ('LEX-HEB-OSHB', 'LEX-GRC-MGNT19')),
  ord     INTEGER NOT NULL CHECK (ord >= 1),                -- document order in its file
  CHECK (substr(key, 1, 1) = CASE lexicon WHEN 'LEX-HEB-OSHB' THEN 'H' ELSE 'G' END)
) STRICT, WITHOUT ROWID;

CREATE TABLE lex_node (                                     -- section 7
  key    TEXT NOT NULL REFERENCES lex_entry(key),
  n      INTEGER NOT NULL CHECK (n >= 0),                   -- document order within the entry
  parent INTEGER,                                           -- NULL = direct child of <entry>
  name   TEXT,                                              -- element name, or NULL for text
  text   TEXT,                                              -- text bytes, or NULL for an element
  CHECK ((name IS NULL) <> (text IS NULL)),
  CHECK (parent IS NULL OR parent < n),
  PRIMARY KEY (key, n),
  FOREIGN KEY (key, parent) REFERENCES lex_node (key, n)
) STRICT, WITHOUT ROWID;

CREATE TABLE lex_attr (
  key     TEXT NOT NULL,
  n       INTEGER NOT NULL,
  i       INTEGER NOT NULL CHECK (i >= 0),                  -- order within the element's start tag
  name    TEXT NOT NULL,
  value   TEXT NOT NULL,                                    -- exactly as in the source
  ref_key TEXT,                                             -- canonical target for cross-references
  PRIMARY KEY (key, n, name),
  FOREIGN KEY (key, n) REFERENCES lex_node (key, n)
) STRICT, WITHOUT ROWID;
```

Range ends can't be checked against `length(text)` in a `CHECK` (another table), so the check stage runs, and requires zero rows from:

```sql
SELECT 'mark' AS t, verse_id, n FROM mark JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
UNION ALL SELECT 'red_piece', verse_id, n FROM red_piece JOIN verse ON verse.id = verse_id WHERE end_cp > length(text)
UNION ALL SELECT 'word', verse_id, n FROM word JOIN verse ON verse.id = verse_id WHERE end_cp > length(text);
```

The red-span invariants R1, R3 and R6 (section 5.3) are run the same way on the built file, each required to return zero rows (M1-F12):

```sql
-- R1: every piece is non-empty and inside its verse
SELECT 'R1', r.verse_id, r.n FROM red_piece r JOIN verse v ON v.id = r.verse_id
 WHERE NOT (r.start_cp < r.end_cp AND r.end_cp <= length(v.text));

-- R3: in each verse n runs 0, 1, 2, … in position order, at least one character between pieces
SELECT 'R3', a.verse_id, a.n FROM red_piece a
  JOIN red_piece b ON b.verse_id = a.verse_id AND b.n = a.n + 1
 WHERE b.start_cp <= a.end_cp
UNION ALL
SELECT 'R3', verse_id, NULL FROM red_piece GROUP BY verse_id
HAVING min(n) <> 0 OR max(n) <> count(*) - 1;

-- R6: the pieces of one q_ord sit in consecutive verses, one per verse; inner edges are 0 and length
WITH p AS (
  SELECT r.q_ord, r.verse_id, r.start_cp, r.end_cp, length(v.text) AS len,
         lag(r.verse_id) OVER w AS prev_v, row_number() OVER w AS i,
         count(*) OVER (PARTITION BY r.q_ord) AS k
    FROM red_piece r JOIN verse v ON v.id = r.verse_id
  WINDOW w AS (PARTITION BY r.q_ord ORDER BY r.verse_id))
SELECT 'R6', verse_id, q_ord FROM p
 WHERE (i > 1 AND start_cp <> 0) OR (i < k AND end_cp <> len)
    OR (i > 1 AND (prev_v = verse_id
                   OR EXISTS (SELECT 1 FROM verse WHERE id > p.prev_v AND id < p.verse_id)));

-- q_ord runs 1..N with no gaps
SELECT 'q_ord', NULL, NULL FROM red_piece HAVING count(DISTINCT q_ord) <> max(q_ord);
```

**Book main titles (D2).** `book.main_title` is the character data of the book's `<title type="main">`, raw: the only element allowed inside is `<abbr expansion="…">` (5 titles, `ST.`), whose text is kept and whose attribute is not imported; any other element, a second main title, one after chapter 1, an empty title, a character outside the profile's allowlist or whitespace other than U+0020 fails closed (check `title`). The KJV profile requires all 66 (fixtures may have none, hence NULL is allowed). These are the source's long titles (`THE FIRST BOOK OF MOSES CALLED GENESIS`); short reader-facing names are not invented here (the scholar's cited table is separate). A test compares all 66 with a naive byte-level reading of the raw file.

**content.sqlite checks.** The check stage requires zero rows from: a node whose parent is a text node; an attribute on a text node; an entry whose nodes are not numbered 0..k−1; an entry with no nodes; a lexicon whose `ord` is not 1..N; a `ref_key` with no `lex_entry` (plus `integrity_check`, `foreign_key_check`, schema equality, and T14 against the parse).

And anyone can read red letters straight out of the file: `SELECT substr(v.text, r.start_cp + 1, r.end_cp - r.start_cp) FROM red_piece r JOIN verse v ON v.id = r.verse_id WHERE v.id = 43003016;` (John 3:16).

Never stored: rendered HTML, normalised text, generated quotation marks or pilcrows, anything from notes, `morph` or `lemma.TR`.

## 9. Untrusted input

Source files are untrusted data even after the hash check: the hash proves we have the file we reviewed, not that the file is harmless, and the same parsing code will later handle files readers import themselves (PLAN.md "Adding translations"). So the parser is defensive on its own, and the hash check comes first.

### 9.1 Order and caps

**Order.** path rules (9.5) → size cap → read once into memory → SHA-256 → strict UTF-8 check of the whole buffer (`std::str::from_utf8`) → pre-scan (9.2) → parse (9.3, 9.4). Nothing is parsed before its hash matches.

**Size and shape caps** (each one fails closed; the KJV's actual value in brackets):

| Cap | Limit |
|---|---|
| File size, checked from metadata before reading and again by reading at most cap + 1 bytes | KJV 64 MiB [28.0 MB]; each lexicon 16 MiB [2.7 MB, 2.4 MB]; `.vrs` 1 MiB [18.8 KB]; overrides file 64 KiB |
| Element nesting depth | 16 [8; lexicons 4 and 5] |
| Attributes per element / attribute value length | 16, counted by us: the 17th fails before any duplicate check (9.3) / 4 KiB |
| Name length (element or attribute) | 64 bytes |
| One text node | 64 KiB (largest expected: the MGNT prologue, about 3.3 KB) |
| Tokens per file | 4,000,000 [1,685,578] |
| Stored verse text | 4,096 code points [528] |
| Verses per file / `q` per file | 40,000 / 10,000 |
| Built database read into memory by the check stage | 256 MiB |

### 9.2 Pre-scan (XML inputs) and line formats

The pre-scan runs over the raw bytes of the three **XML inputs only**; the `.vrs` file and the overrides file are line formats with their own strict parsers (below). The DTD, entity and declaration rules are enforced a second time by the event loop (9.3).

- **Bytes.** No byte below 0x20 except TAB, LF and CR (the XML 1.0 `Char` rule), no U+FFFE or U+FFFF, and CR only in a profile that allows it (OSHB; the KJV and MGNT have none).
- **Declaration, pinned per profile** (M1-F15). The file must start with exactly these bytes (survey probe), and the XML parser is given only the bytes after the prefix:

  | Profile | Required prefix | Parser input starts at byte |
  |---|---|---|
  | KJV | `<?xml version="1.0" encoding="UTF-8"?>` (38 bytes) | 38 |
  | OSHB | `<?xml version="1.0" encoding="utf-8"?>` (38 bytes) | 38 |
  | MGNT | `<?xml version='1.0' encoding='utf-8' standalone='yes'?>` (55 bytes), LF, then the pinned DTD subset (bytes 56 to 1,090, end exclusive) | 1,090 |

  Because byte 0 must be `<`, a BOM (which quick-xml may skip by itself) can't pass. Any `Decl` or `DocType` event from the parser is a second declaration or DOCTYPE and fails. Error offsets add the start byte, so they are always file offsets (the "1,090-byte base" for MGNT).
- **No DTD, no entities, no external anything.** The KJV and OSHB files must contain no `<!` at all: no `DOCTYPE`, no `ENTITY`, no comment, no CDATA. MGNT's internal DTD subset (survey 9.2) is the one exception, and it is **skipped, never interpreted**: the pre-scan requires `<!DOCTYPE strongsdictionary [` at byte 56 and the subset to be exactly the pinned 1,034 bytes (SHA-256 `9b89a2867fda6f8bfcda951f9bad1ccfc5b06416eb6d38f84f8177078e9441f6`). Any other `<!` anywhere (in MGNT: anywhere from byte 1,090 on) fails. The survey and the review both confirmed the subset declares no entities, no parameter entities and no default or fixed attribute values, so skipping it changes nothing in the parsed data (its one enumerated type, `strongsref@language`, is checked against the allowlist instead of normalised). A different subset (a new MGNT release) fails closed and needs a review. Implemented as `XmlProfile.subset` (`xmlscan.rs`): the byte between the declaration and the subset must be one LF, the subset must start with `<!DOCTYPE strongsdictionary [` and hash to the pinned value, and the `<!` and `<?` scans and the parser start at byte 1,090. Text outside the root may contain CR only in the OSHB profile.
- **Any `&` in an XML input fails closed.** None of the three XML files contains one. (`eng.vrs.txt` does, at line 307, inside a `#!` directive (M1-F10); that is why this scan and the `<!` scan are scoped to XML inputs.) Later sources that use the five predefined entities or numeric character references get an explicit decoder for exactly those, never a general entity mechanism; offsets are then measured in the decoded text and V6 applies to decoded characters.
- `xsi:schemaLocation`, namespaces and any URL-looking value are inert strings: never opened, fetched or resolved.

**`.vrs` lines** (M1-F10). Every line of `eng.vrs.txt` is classified as exactly one of: a `#!` line (a Paratext mapping directive, **not** a comment), any other `#` line (comment), a blank line, a book line (`<BOOK> <c>:<n> <c>:<n> …`), or a mapping line (`<BOOK> <c>:<v>… = …`). A line that fits none fails closed. M1 uses only the 66 canonical book lines; the other 26 book lines (deuterocanonical and other books, after REV) are ignored by a pinned list of their codes; directives and mapping lines are recognised and ignored, because M1 doesn't map versifications. The pinned file (survey probe) has 452 LF-terminated lines and no CR: 89 comments, 1 `#!` directive (line 307), 1 blank, 92 book lines (66 + 26) and 269 mapping lines. T2 asserts these counts, so a directive can never be mistaken for a comment or a book line.

### 9.3 quick-xml 0.42.0: what may be used (M1-F5)

quick-xml is a tokenizer, not a well-formedness checker, and several of its helpers change bytes. Only the API in the "allowed" column may be used; anything else needs a design change. Names are from the 0.42.0 documentation (read by the review on 2026-10-20); Phase B confirms each against the pinned crate.

| Area | Allowed | Forbidden |
|---|---|---|
| Reader | `Reader` over the in-memory `&[u8]`, `read_event()` | `NsReader` and every namespace-resolving call; a `Reader` over a file or `BufRead`; the `encoding` feature |
| Names | `e.name().as_ref()`, the **full qualified name** bytes, compared to byte literals. So `<x:q who="Jesus">` is the unknown element `x:q` and fails closed instead of turning red | `local_name()`, `prefix()`, any namespace lookup |
| Text | the event's raw bytes (`Deref` / `into_inner()`), nothing else | `xml_content()`, `xml10_content()`, `xml11_content()`, `html_content()` (all normalise line ends; the 1.1 one also maps U+0085 and U+2028), `unescape*()`, `decode()`, `inplace_trim_start()`, `inplace_trim_end()` |
| Attributes | iterate `attributes()`, counting ourselves and failing at the 17th **before** our own duplicate check over the at most 16 collected names (quick-xml's own check may stay on, but the cap must not depend on its RUSTSEC-2026-0194 fix); the raw `Attribute::value` bytes; each value double-quoted in all three profiles (MGNT too: after the declaration it has no single-quoted value, D2 probe), checked on the raw start-tag bytes | `unescape_value*()`, `decode_and_unescape_value*()`, `normalized_value*()`, `html_attributes()` |

**Configuration, set explicitly and asserted by a test** (T15 reads `reader.config()` back field by field, so an upgrade that changes a default fails CI): `trim_text_start = false`, `trim_text_end = false`, `expand_empty_elements = false`, `check_end_names = true`, `allow_unmatched_ends = false`, `allow_dangling_amp = false`.

**Every event kind is handled** by an explicit `match` arm (if the enum is `#[non_exhaustive]`, the wildcard arm fails closed):

| Event | Action |
|---|---|
| `Start`, `End`, `Empty` | dispatch on (kind, full name) per 9.4; push or pop the own stack and compare the end name with the stack top |
| `Text` | raw bytes (above); must not contain `]]>`; outside the root element only whitespace the profile allows |
| `Decl`, `DocType` | fail (the declaration and MGNT's subset were consumed by the pre-scan, 9.2) |
| `CData`, `Comment`, `PI`, `GeneralRef` | fail |
| `Eof` | fail unless the root element was opened and closed and the own stack is empty |

**What the own stack adds**, because quick-xml doesn't enforce it: exactly one root element (a `Start` or `Empty` after the root closed fails); no non-whitespace text before or after the root; no `<` in any attribute value; no `]]>` in text; end names equal to the own stack's top (as well as `check_end_names`); the depth, token, attribute and length caps (9.1). The Python tokenizer already rejects `<` in attribute values and `]]>`; the two implementations keep one rejection list, and T15's fixtures run against both. quick-xml has no DTD processor and does not expand custom entities on its own; the pre-scan means we don't depend on that.

### 9.4 Element event forms (M1-F3)

The event loop dispatches on (event kind, full element name) for **every** element of the profile: each element is allowed as a `Start`/`End` container, as `Empty` (`<x …/>`), or as either. An element not in the table, or a form not allowed for it, fails closed, in the Rust event loop and in the Python oracle alike. Counts are the survey probe's (their totals agree with the survey):

| Profile | `Empty` only | Either | `Start`/`End` only (`Empty` fails) |
|---|---|---|---|
| KJV | `verse` (62,204 milestones), `milestone` (2,970) | `w` (6,331 empty, survey anomaly 3) | `q`, `transChange`, `divineName`, `note`, `inscription`, `catchWord`, `rdg`, `chapter`, `div`, `title`, `foreign`, `abbr`, `osis`, `osisText`; in the (not imported) header: `header`, `work`, `title`, `identifier`, `refSystem`, `scope`, `date`, `p`, `revisionDesc` |
| OSHB | none | `source` (1 empty, H5223, survey excerpt H2) | `lexicon`, `entry`, `w`, `def`, `meaning`, `usage`, `note` |
| MGNT | `greek` (6,139), `pronunciation` (5,652), `strongsref` (7,063) | none | `strongsdictionary`, `prologue`, `entries`, `entry`, `strongs`, `strongs_def`, `strongs_derivation`, `kjv_def`, `latin` |

In addition, any `q` with an `sID` or `eID` attribute fails closed (section 5.2). A future profile that needs milestone `q` adds it explicitly, with its own algorithm, tests and pinned facts.

### 9.5 Fail closed, paths and messages

**Fail closed, everywhere.** Unknown element, attribute, attribute value, event form, token kind, character outside the profile's allowlist, structure the survey didn't see (for example a `q` between verses), cap exceeded, or a checksum mismatch: the build stops, names the file, the byte offset and the verse if known, and writes nothing. The output is built in a temporary file, checked there, and published only after the check passes (section 2, stages 4–6), so a failed build never leaves an unchecked database where the app could find it.

**No paths, commands or URLs from content.** Input paths are `sources/raw/<ID>/<file>` where `<ID>` and `<file>` come from the repo's own checksum files and must match `^[A-Z0-9][A-Z0-9-]{0,40}$` and `^[A-Za-z0-9][A-Za-z0-9._-]{0,80}$` (no separators). Also rejected (M1-F11): `..` anywhere in a name; a trailing `.` (Windows strips it, so two names could alias one file); and the Windows device names `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, compared case-insensitively with the part before the first `.`, so `NUL.xml` and `con` fail too. Before opening, `std::fs::symlink_metadata` is read for `sources/raw/<ID>` and for the file: a symlink, a junction or any other reparse point (on Windows, `file_attributes()` has `FILE_ATTRIBUTE_REPARSE_POINT`) fails, and so does anything that isn't a plain directory or a regular file respectively. An alternate data stream name (`kjv.osis.xml:ads`) can't pass the pattern. Output paths come only from the command line, and `<out-dir>` gets the same reparse-point check. Nothing from inside a source file (`osisID`, `xml:lang`, `src`, attribute values, text) is ever used to build a path, a SQL string (all inserts use bound parameters), a URL or a command. The pipeline never executes anything and never touches the network.

**Error messages don't echo content.** They give positions and a short, escaped excerpt of at most 80 bytes (control characters shown as `\u{…}`), never raw bytes, so a hostile file can't inject terminal escapes into logs. `KeyError` in `strongs.rs` already carries no input text.

## 10. Determinism, expected hashes and opening content files

### 10.1 Determinism

- Same inputs + same pipeline commit + same `Cargo.lock` (which pins the bundled SQLite) → byte-identical database files. Inserts happen in document order in one transaction; no timestamps, random IDs, hash-map iteration order or locale-dependent formatting reach the file (the temp-file name of stage 4 never does). The build runs `VACUUM` before closing so free-page layout can't vary.
- **Content addressing, not provenance, inside the files (PM decision 2026-10-22).** The `source` table records each input's SHA-256 (and `meta` the overrides file's); nothing from the build environment is stored, not even the pipeline commit, so a database's SHA-256 is a pure function of the pinned inputs and the pipeline code (with `Cargo.lock`). Before D2 `meta.pipeline_commit` made the committed expected hash change with every commit.
- **Provenance lives beside the files.** The pipeline executes nothing, so the caller passes the commit in `REVELATION_PIPELINE_COMMIT` (the CI step or a developer script runs `git rev-parse HEAD` and `git status --porcelain`): 40 lowercase hex digits, with `-dirty` appended when the working tree has uncommitted changes; unset or malformed gives `unknown-dirty`. `build` prints it with the SHA-256 of `Cargo.lock` and the binary profile, and the header of `pipeline/tests/expected/output-sha256.txt` records the provenance of the committed values. With `REVELATION_RELEASE=1`, `build` refuses any value ending in `-dirty` before reading a source (M1-F12; `db::build_commit`, tested in T16).
- A test builds twice into two directories and compares the output hashes (T16).

The caps leave wide margins: the largest text node in the three files is 3,343 bytes (MGNT prologue), the longest attribute value 134 bytes, and no tag has more than 4 attributes.

### 10.2 Expected hashes (M1-F6)

- `<file>.sha256` beside each output is a **convenience** for people and CI (`sha256sum -c`). It proves nothing against tampering, because whoever can replace the database can replace the sidecar too. The app never reads it.
- The expected SHA-256 of each bundled content file is committed in `pipeline/tests/expected/output-sha256.txt`: one `<sha256>  <file>` line per file (`kjv.sqlite`, `content.sqlite`), `#` lines for the header and the build provenance (10.1). CI fails when a build's output differs from it, so every content change is a visible diff that needs the owner's review (section 11.1), and CI's Windows and Ubuntu builds must agree, which also proves the output doesn't depend on the platform. The app build (M2) compiles the same value into the signed binary, and the start-up check (PLAN.md 2.6) compares against that compiled-in value only.
- Library packs (M9): the expected hash comes from the signed pack manifest, never from a file beside the pack.

### 10.3 Opening content files (rules for the app core, M2, M8 and M9)

Recorded now because M2 builds the start-up check (M1-F1, M1-F8).

1. **One function, `open_verified(path, expected_sha256)`,** shared by the pipeline's check stage and the app core:
   - **Best (recommended):** read the whole file into memory (size cap), compute its SHA-256, compare with the expected value (10.2, never a value taken from beside the file), then open **that same buffer** with `sqlite3_deserialize`, read-only, plus `PRAGMA query_only = 1`. No sidecar can be read and there is no time-of-check/time-of-use gap; it is the pipeline's read-once-then-hash rule again. Cost: the file's size in RAM (the review estimates 30–60 MB for the KJV and lexicons) and roughly 0.1 s of hashing at start-up, inside the 1 s cold-start target.
   - **Acceptable:** open `file:<path>?mode=ro&immutable=1` (`SQLITE_OPEN_URI`), hash first, and refuse to start if `<db>-wal`, `<db>-shm` or `<db>-journal` exists. It uses less RAM but leaves a gap: the file could change after it was hashed, and `immutable` tells SQLite it can't.
   - **Never:** a plain `SQLITE_OPEN_READ_ONLY` open of a path. SQLite reads a `-wal` beside the file even on a read-only connection and even when the database was built in DELETE mode, and the file's hash still matches; the review demonstrated a changed verse text that way (M1-F1).
   - The pipeline's check stage uses Best, with the hash computed rather than compared (it is the value being produced). For the app, Best is the recommendation; whether large future packs (commentaries, M4+) may fall back to Acceptable is owner question 14.
2. **Every content file and pack** is opened with `PRAGMA trusted_schema = OFF`, `SQLITE_DBCONFIG_DEFENSIVE`, tight `sqlite3_limit`s, `PRAGMA cell_size_check = ON` and `PRAGMA query_only = 1`. `load_extension` is never enabled (M1-F17). (The rusqlite features this needs, such as `limits`, get the same "adds no crate" check as `serialize`, section 13.)
3. **Packs** (`.revpack`, M9; imported translations, M3+) are crafted SQLite files from outside. One is accepted only if its `sqlite_schema` equals a pinned copy exactly: no triggers, views or virtual tables. `PRAGMA integrity_check` runs before first use. FTS5 tables from a pack are never opened; the app builds its own search index from `verse.text`.
4. **Search display (M8)** always reads `verse.text` and computes highlights on the stored text. `snippet()` and `highlight()` output is never shown, because the index holds normalised text (PLAN.md 2.5).
5. **Offline `.revpack` files** need the same signature check as library packs, or a clear "Imported by you" path that re-runs the full validation report (T2–T9, T13).

## 11. Verification tests

All run in CI (`cargo test --workspace`, plus `revelation-pipeline build` on the real sources, which CI downloads with `curl` and verifies) and block merge. "Source" means the parse of the pinned file; "facts" means the pinned expected values from the independent survey (below).

### 11.1 Independent oracle

The round-trip (DB vs. the pipeline's own parse) proves the writer and schema lose nothing, but it can't catch a parser bug, because both sides come from the same parser. So CI also compares against values produced by a **different implementation**: the Python regex tokenizer in `docs/pipeline/survey-scripts/`, which shares no code with the Rust parser. Different code catches coding bugs, but both implement the same written definitions (sections 3.1 and 5.2) from the same author, so one oracle file also uses a **structurally different method**, and none of them can be hand-edited past CI (M1-F4).

**Files** (in `pipeline/tests/expected/`, generated from the pinned sources):

| File | One line per | Contents |
|---|---|---|
| `kjv-chapters.tsv` | chapter (1,189) | verse count; SHA-256 of the chapter's stored verse texts joined with LF; marks as `verse:start-end:kind`; red pieces as `verse:start-end:q_ord`; SHA-256 of the chapter's words and keys (`verse:n:start-end:key+key…`, LF-joined) |
| `kjv-q.tsv` | `q` (648) | `q_ord`; book of the start tag; the source-file byte offset of the start tag's `<` and of the byte after the end tag's `>` (checkable by a person with a hex viewer); verses touched; red length in code points; SHA-256 of the red text computed by **tag stripping**: the source bytes between the start and end tags, minus `<note>…</note>` subtrees, then minus every tag, then minus every LF (in this file text between verses is exactly one LF and verse text contains none, survey sections 4 and 6). The review derived the same 648 `q` / 210,502 code points this way |
| `lexicon-entries.tsv` | lexicon entry (8,674 + 5,624) | key; lexicon; `ord`; byte count and SHA-256 of the raw character data between `<entry …>` and `</entry>` with tags stripped (CR bytes kept); attribute count and SHA-256 of the attribute list (`element@name=value`, the entry's own attribute first, document order, LF-joined); SHA-256 of the **tree**: the tokens between the entry tags re-serialised without attributes (`<x>`, `</x>`, an empty element as `<x></x>`, text raw), which pins names, nesting and where each text node sits. Generated by `oracle_lexicons.py` (own strict regex tokenizer and walker per lexicon, pinned MGNT subset hash; `selftest_lexicons.py` checks its refusals) |

The Rust test (T5) serialises the database the same way and compares line by line, so a mismatch names the chapter, the `q_ord` or the entry; for `kjv-q.tsv` it joins each `q_ord`'s pieces in verse order. Against the review's table of red-span errors (M1-F4), `q_ord` and `kjv-q.tsv` catch a merge plus a split that cancel out in per-book totals, tag stripping catches a definition-level error shared by both milestone algorithms, and the word and lexicon hashes give T13 and T14 an independent comparison. A misreading shared by both implementations *and* the tag-stripping method is left to the golden samples (T18, T19).

**Generator rules** (M1-F4, M1-F13):
- Each file's header records the generator script's path and SHA-256, each source's ID and SHA-256, and the Python version.
- Every generator uses one shared `check_sha()` (checksum verified before anything is read) and one strict walker: end-tag names compared, nesting checked (a nested `q` raises), the empty-form allowlist of section 9.4, and `sID`/`eID` on `q` rejected. The survey scripts that lack these (`survey_kjv.py` and `basic_facts.py` don't check checksums; `survey_kjv_detail.py` doesn't compare end names or detect a nested `q`) must not generate expected files until fixed; that fix is a separate task in `docs/pipeline/survey-scripts/`.
- Run with `py -I` (CI: `python3 -I`), paths as arguments. Regenerated only when the source pin or a generator changes; never edited by hand.

**CI regeneration job.** A separate `oracle` job downloads and verifies the pinned sources the same way as the Rust job, runs the generators, and fails on `git diff --exit-code pipeline/tests/expected/`. It also runs the walker against the T15 fixtures marked "both" and requires each one to raise. So a hand-"fixed" expected file can't merge. How Python gets onto the runner (preinstalled `python3` or `actions/setup-python` pinned by SHA) is an owner/PM call, open question 11. The Rust tests themselves need no Python: they read only the committed files.

**Owner review.** Changes under `pipeline/tests/expected/**` (including `output-sha256.txt`, section 10.2) and `pipeline/tests/golden/**` need the owner's review: a `CODEOWNERS` rule plus branch protection. When T5 fails, "regenerate and commit" is never the fix without that review.

### 11.2 Test list

| ID | Test | Checks | PLAN rule |
|---|---|---|---|
| T1 | Source verification and paths | each file's SHA-256 equals `sources/checksums/<ID>.sha256`, and that value appears in the ID's SOURCES.md row; a one-byte-changed fixture and a truncated fixture are refused before parsing; checksum files with CRLF, uppercase hex, one space, no final LF, two lines or another file name are refused; IDs and names `NUL`, `CON`, `..`, `A/B`, `kjv.osis.xml.`, `kjv.osis.xml:ads`, `CON.xml`, and a symlink or junction in `sources/raw/<ID>/`, are refused (9.5); with `CI` set and `REVELATION_SOURCES` unset the source tests fail, not skip | 2.3 |
| T2 | Versification table | `eng.vrs.txt` lines classify exactly as pinned (89 comments, 1 `#!` directive, 1 blank, 92 book lines = 66 used + 26 ignored by code, 269 mapping lines; any other line fails, 9.2); the 66 used lines give the 66 books; the overrides file has exactly the reviewed lines (`3JN 1 14`, `REV 12 17`), each naming an existing chapter and changing its value; **the resulting table totals 1,189 chapters and 31,102 verses, asserted before the database is tested against it** | 2.6 |
| T3 | Verse counts | the database has 66 books in canonical order and, for every one of the 1,189 chapters, verses numbered exactly 1..n with n from the T2 table | 2.6 |
| T4 | Verse round-trip | every verse's stored text equals the source's, byte for byte, 31,102 of 31,102; the same for marks (kind, start, end, order) and words (range, keys, order) | 2.6 |
| T5 | Oracle | the database matches `kjv-chapters.tsv` in every chapter, `kjv-q.tsv` for every `q_ord`, and `lexicon-entries.tsv` for every entry (11.1) | 2.6, 2.8 |
| T6 | Verse-text invariants | V1–V7 (section 3.2) hold, including the pinned count of 91 trailing spaces; the range-end query in section 8 returns no rows; fixtures with CRLF, a lone CR, TAB, NBSP, U+2028, U+202E, U+200B, U+FEFF, U+FFFD or NUL in a verse, an empty verse, a 4,097-code-point verse, a leading space, and two spaces left by removing a note each fail | 2.5 |
| T7 | Red round-trip per verse | for every verse, `(start, end, q_ord)` from the database equals the source exactly: count, start, end, grouping | 2.8 |
| T8 | Red totals per book | `q` count, piece count and red characters per book equal the source **and** the facts table (section 5.4); the 39 OT books and the 19 NT books without red show 0 | 2.8 |
| T9 | Red invariants | R1–R6 (section 5.3) on the parsed model, and R1, R3, R6 as SQL on the built file (section 8); marks are inside or outside red pieces, never across (P4); B1 (5.1); fixtures: `q` across a chapter, in a note, between verses (start tag, and separately end tag), nested; an empty piece at a verse end or start; `</q><q>` with no separator (fails) and with one space (passes as 2 pieces); `transChange` containing a whole `q` (fails) | 2.8 |
| T10 | No inference | fixtures: narration such as "Jesus said, Follow me." without `q` gives no red; a translation fixture without words-of-Jesus markup gives zero `red_piece` rows and `has_red_letters = 0`; red rows exist only where a `q who="Jesus"` was parsed (the writer has no other code path that inserts them) | 2.8 |
| T11 | Mutation tests | fixtures that drop, add, shift by ±1, extend, trim, split, merge (at a verse edge and mid-verse), move to the neighbouring verse, or leak a red span across a verse each fail T5, T7, T8 or T9, and each test asserts **which** check fires; the same for a dropped, added or changed character in verse text (T4/T5) | 2.6, 2.8 |
| T12 | Units | a fixture with a character outside the BMP and a combining mark, inside a red piece and across a piece boundary, proves code points are used end to end (pipeline, `substr` audit query, the core's runs function, the code-point → UTF-16 converter); a boundary placed just before the combining mark fails closed (B1) | 2.8 |
| T13 | Strong's | key grammar (the unit tests in `strongs.rs`); the KJV profile's exact spelling (6.2): `strong:H7225`, `strong:H00430`, `strong:G01`, a double space and a TAB between tokens each fail; every KJV key resolves to a used lexicon entry; anomaly list entries all match; per-book counts of words and keys equal the source; per-chapter word/key hashes equal `kjv-chapters.tsv` | 2.6 |
| T14 | Lexicon round-trip | 8,674 + 5,624 entries; every entry's tree (names, attributes, text bytes, order) equals the source; text nodes concatenate to the entry's character data; every cross-reference resolves; entry H1's whitespace nodes equal the source bytes of survey excerpt H1 and contain `\r\n\t` (9.3); fixtures: a CR in a non-whitespace node, `H01` and `H1` in one file, `strongsref language="Greek"`, `strongs="0"`, U+202E in lexicon text each fail | 2.6 |
| T15 | Untrusted input | fixtures with a DOCTYPE, an ENTITY (billion laughs, an external `SYSTEM` entity, parameter entities), a changed or moved MGNT subset, a second DOCTYPE, `<!` after MGNT byte 1,090, `&amp;`, `&#65;`, `&#x202E;`, a lone `&`, CDATA, a comment, a PI, a second `<?xml?>`, a BOM, invalid UTF-8 (overlong, encoded surrogate, truncated at EOF), depth 17 and 100,000, 4,000,001 tokens, 17 and 1,000,000 attributes (stops at 17), a duplicate attribute, a 65-byte name, a 4 KiB + 1 value, a 64 KiB + 1 text node, a single-quoted attribute in the KJV profile, `<` in an attribute value, `]]>` in text, a second root element, text outside the root, an unknown element in a verse, `<x:q who="Jesus">`, `q` with another `who` (`jesus`, `Jesus `), an extra attribute, a non-empty `marker`, verse-structure faults (mismatched `osisID`/`sID`, unmatched or duplicate `eID`/`sID`, verses out of order or in the wrong chapter, a multi-verse `osisID`, a verse open across `</chapter>`), stray text between verses; and the M1-F3 set: an empty `q`, a milestone `q` (`sID`/`eID`), an empty `transChange`, an empty `divineName` and a container `<verse>…</verse>`. Each fails closed with no output file; the M1-F3 set and the `q` cases fail in **both** Rust and the Python walker. One test asserts the quick-xml configuration field by field (9.3). On Linux CI: a 2 GB sparse file and a file that grows after its size is read are refused without a full read | 2.2, 2.3 |
| T16 | Determinism | two builds produce byte-identical files and `.sha256` values; the output hashes equal `output-sha256.txt` on both Windows and Ubuntu (10.2); `meta` holds exactly its pinned keys (no build-environment value, 10.1); a release build refuses a `-dirty` commit | 2.2, 2.6 |
| T17 | Read-only and sidecars | the check stage opens files only through `open_verified` (in-memory, read-only; 10.3), and a write attempt through that connection fails; header bytes 18/19 are `01 01`; after a build no `-wal`, `-shm` or `-journal` exists beside the temp file or `<dest>`; with a crafted `-wal` (made from a twin database whose verse text differs) and, separately, a hot `-journal` planted next to a built file, `check` and `open_verified` show the original text or refuse, **never the WAL's text**, and publish refuses while a sidecar of `<dest>` exists (M1-F1). The app's half lands with `open_verified` in M2 | 2.1, 2.6 |
| T18 | Golden samples (red) | section 12 | 2.6, 2.8 |
| T19 | Golden samples (verse text) | Gen 1:1, Ps 23:1–6, John 3:16 and the other listed verses equal their hand-filled `source_expected` text (section 12) | 2.6 |
| T20 | Render test | M2: the reader's DOM text for sampled and full chapters equals the stored text, and the DOM's red spans contain exactly the stored red pieces. Containment is not enough, because a nested element (a Strong's link, `a`, `button`, `.nd`, `.add`, a hover style) can set its own colour: T20 also asserts the **computed colour** of the parent of every text node, red exactly inside stored pieces and the text colour outside, with red on and off, in hover, focus and selection states and with every highlight colour (M1-F7; the matching CSS lint and the `<span>` + `color: inherit` rule for Strong's words are in the designer's `docs/design/red-letters-and-highlights.md`). The core's runs function is unit-tested now | 2.6, 2.8 |
| T21 | Atomic publish | the stage 4–6 function takes a step callback: the CLI passes one that does nothing, and a test passes one that fails after each step 0–6 of section 2, and separately makes the rename fail by holding `<dest>` open (Windows): `<dest>` and `<dest>.sha256` stay byte-identical to the previous good build, and no temp file is left behind (or the next start-up removes it). At the one point between the two renames, the new `<dest>` passes `check` and the stale `.sha256` is reported as a mismatch (M1-F2) | 2.6 |

T20 needs the app (M2); T17's app half also lands in M2; everything else is pipeline-only and lands in M1 Phase B. Imported files (M3+) reuse T2–T9 and T13 as the validation report.

### 11.3 Adversarial cases

There are 24 groups of adversarial cases. Each lands in the test named below, as a byte-exact fixture in `pipeline/tests/fixtures/`, and fails closed with no output file, except the cases marked as passing. "Both" cases run against the Rust pipeline and the Python walker (11.1).

| Cases | Test | Cases | Test |
|---|---|---|---|
| 1 (`q` across a chapter) | T9 | 13 (non-BMP plus a combining mark) | T12 |
| 2 (`q` over 3 verses with a note, passes) | T7 | 14 (verse structure) | T15 |
| 3, 4 (empty and milestone `q`, both) | T15, T9 | 15 (stray text, empty / long / leading-space / double-space verse) | T15, T6 |
| 5 (`q` placement and attribute variants, `x:q`) | T9, T15 | 16, 17 (DTD, entities, constructs, caps) | T15 |
| 6, 7 (empty pieces; adjacent `q`) | T9; the passing half also T7 and T20 | 18 (sparse and growing files, Linux CI) | T15 |
| 8 (`transChange` and `q` nesting) | T9, T4 | 19 (MGNT subset variants) | T15 |
| 9 (mutation set, asserting which check fires) | T11 | 20, 21 (checksum files, paths) | T1 |
| 10 (planted `-wal` / hot `-journal`; kill or deny mid-write) | T17, T21 | 22 (`REVELATION_SOURCES` unset with `CI`) | T1 |
| 11 (BOM, control and bidi characters, bad UTF-8) | T15, T6 | 23 (lexicon cases) | T14 |
| 12 (character references) | T15 | 24 (`lemma` spacing and padding) | T13 |

## 12. Golden tests

### 12.1 The scholar's proposal

`docs/scholar/red-letter-golden-samples.md` section 8: two expectation columns. A person fills `source_expected` by hand from the raw pinned file; CI asserts the database equals `source_expected`; differences between the printed edition and `source_expected` go in a non-blocking "edition differences" report and in SOURCES.md, never "fixed".

### 12.2 My view: agree, with one blocking addition

I agree with the split, for three reasons:
1. **Only the source column can be enforced without breaking rule 2.4.** Where print and source differ (the survey and the scholar expect a few: Aramaic sayings, John 3:16–21, Rev 1:8, 1:11, 4:1, 22:14), the database must keep the source's boundaries. If CI blocked on print, it would stay red for ever or push someone to change content to match a book. Neither is acceptable.
2. **The source column is still an independent check, if a human fills it.** It's filled from the raw excerpts (`docs/pipeline/golden-source-excerpts.md`), not copied from pipeline output, so it catches a pipeline that drops, adds, shifts or trims a span at exactly the tricky places: mid-verse starts, carried spans, two spans in one verse, OT quotations inside Jesus' words. But the developer is an AI and the scholar's print checks were done by an AI reading images, so if the same kind of agent fills it, it is no longer independent of the pipeline's author (M1-F14). The owner (or another human) fills it, or at least spot-checks it, and the file's header names that person (open question 15).
3. **Words, not characters, against print** (scholar's point 3) is right: printed boundary punctuation can't be read reliably from scans, and exact characters are already enforced by T7, T5 and T8.

**Addition (blocking): no undocumented difference.** CI generates the print-vs-source comparison and requires that every difference is listed in `pipeline/tests/golden/edition-differences.tsv` (reference, one-line description, the SOURCES.md note it corresponds to) and that every listed difference still exists. That blocks only on *undocumented* or *stale* entries, never on the existence of a real edition difference, and it costs one small file. It keeps PLAN.md's intent ("golden samples … compared to the printed edition" in CI) without letting print override the source. The owner decides (open question 6).

### 12.3 Files and mechanics

- `pipeline/tests/golden/red-letter-source.tsv`: one row per reference in the scholar's YAML (99 rows): `ref`, number of red pieces, and per piece `start` (`verse_start` / `carried` / `mid`), first word, last word, `end` (`verse_end` / `mid`), plus the black text between pieces where the scholar records it. Filled by hand from the excerpts; the header records who filled it and who spot-checked it (by name, and whether a human or an agent), when, and the SHA-256 of the excerpt file used. Paths under `**/golden/**` are byte-exact (`.gitattributes`) and need the owner's review (11.1). Word-level rows don't pin punctuation (a boundary moving across a comma or closing quote keeps the same first and last word); that is T5's and T7's job.
- `pipeline/tests/golden/verse-text.tsv`: `ref` and the exact stored text expected, hand-copied from the raw excerpts (Gen 1:1, Ps 23:1–6, John 3:16 and the others the scholar lists). The same print comparison applies. Trailing spaces are kept exactly (V5), and a hand-copied row is likely to drop one, so the TSV parser shows trailing and other non-obvious whitespace escaped in every diff (`\x20` at the end, `\t`, `\u{a0}`). A mismatch there is fixed in the TSV after checking the excerpt, never by trimming the database.
- Derived values: for each piece, "first word" and "last word" are the first and last whitespace-separated tokens of `substr(text, start + 1, end − start)`; `carried` means `start = 0` and the piece is not the first of its `q_ord`; `verse_start` means `start = 0` and it is the first.
- CI checks that the TSV's references are exactly the YAML's references (a strict, read-only scan of the scholar's file; a mismatch fails), so the two files can't drift.
- The print report is written to `target/golden-print-report.md` and uploaded as a CI artifact.
- Until the human spot-check (scholar open item 1) is done, rows marked `verified-scan (agent)` count for T18's source column only; the print comparison runs but its differences file stays a draft.

## 13. Crates

The pipeline needs XML, SHA-256 and SQLite, and nothing else. Versions are the ones checked on 2026-10-19 (all permissive licences). **Not added to `Cargo.toml` yet**: without the MSVC linker nothing would build or link (M0-3).

| Crate | Version | Features | Why this one | Licence |
|---|---|---|---|---|
| `quick-xml` | 0.42.0 | default features off; **`encoding` off** (UTF-8 only; avoids `encoding_rs`) | mature, small (one dependency, `memchr`), low-level pull reader over `&[u8]`, no DTD processor; we use only `Reader` and raw events | MIT (+ memchr Unlicense/MIT) |
| `sha2` | 0.11.1 | defaults | the standard RustCrypto SHA-256; never write our own | MIT OR Apache-2.0 (+ digest, crypto-common, block-buffer, hybrid-array, typenum, cfg-if, cpufeatures) |
| `rusqlite` | 0.40.2 | `default-features = false`, `features = ["bundled", "serialize"]`; never `load_extension` | `serialize` gives `sqlite3_deserialize` for the in-memory, read-only check (section 2, stage 5; 10.3). It is a feature of an approved crate, expected to add no crate; Phase B confirms that with `cargo tree -e normal,build` before using it, and if it adds one, its licence is reviewed first (fallback: `immutable=1`). The standard SQLite binding; `bundled` compiles the SQLite amalgamation in (3.54.0 via `libsqlite3-sys` 0.38.2), so the version is pinned by `Cargo.lock` and the build is reproducible; turning the default `cache` feature off drops `hashlink`/`hashbrown`/`foldhash`, which we don't need | MIT; SQLite public domain; build-time `cc` MIT/Apache-2.0 |

Deliberately **not** used: an HTTP/TLS stack (section 2.1); `clap` (three subcommands parse with `std::env::args_os`); `serde`, `toml`, `serde_yaml` (the overrides file and the golden TSVs are line formats with strict hand parsers); `regex` (the grammars are tiny); Unicode crates (`char::is_whitespace`, `char::is_control` and the per-source allowlist cover the checks); `anyhow`/`thiserror` (one error enum); `tempfile` (a `create_new` temp file plus `std::fs::rename` in the same directory, section 2); test frameworks (`#[test]` only).

Proposed `pipeline/Cargo.toml` (Phase B):

```toml
[dependencies]
quick-xml = { version = "=0.42.0", default-features = false }
sha2 = "=0.11.1"
rusqlite = { version = "=0.40.2", default-features = false, features = ["bundled", "serialize"] }
```

**Advisories (M1-F17).** As of 2026-10-20 no RustSec advisory affects these versions (quick-xml's RUSTSEC-2026-0194 and -0195 are fixed in ≥ 0.41.0; the FTS5 CVEs are fixed in SQLite 3.53.2; 3.54.0 itself wasn't confirmed against NVD). `cargo audit` (later `cargo deny`) runs in CI once `Cargo.lock` has dependencies, and blocks merge on an advisory; installing it is open question 13.

Exact (`=`) versions plus the committed `Cargo.lock`; any bump is a reviewed change with a fresh licence check (`cargo tree -e normal,build` output attached), as the licence review requires. FTS5 (search, later milestone) is compiled into the bundled SQLite as far as I know; to confirm when search is built.

## 14. Code layout and Phase B order

```
pipeline/src/
  main.rs        CLI: verify | build | check (std only)
  lib.rs
  strongs.rs     key grammar and canonical form (exists; fmt + clippy pass; tests not yet run: no linker)
  sources.rs     checksum files, path rules (incl. device names, reparse points), size caps, read-once + SHA-256
  xmlscan.rs     pre-scan (9.2) + strict event loop over quick-xml (9.3, 9.4), own element stack, caps, positioned errors
  vrs.rs         eng.vrs line classes (9.2), the 66 canonical book lines + overrides file → chapter table; T2
  osis.rs        KJV OSIS → model (verses, marks, red pieces, words)
  lexicon.rs     OSHB and MGNT → entry trees; profiles, pinned facts, cross-reference and KJV-key resolution (D2)
  profile.rs     per-source facts and allowlists: KJV characters (with categories, B1), event forms, declaration,
                 Strong's spellings, pinned counts, anomaly list
  validate.rs    invariants V*, P*, R*, B1, Strong's resolution
  db.rs          schema (section 8) as one SQL constant; writer; stages 4–6 (temp file, sidecar checks, publish)
  open.rs        open_verified: read once, SHA-256, sqlite3_deserialize read-only (shared with the app core in M2)
  check.rs       stage 5: header, schema, integrity, foreign keys, SQL invariants, round-trip, totals, oracle comparison
  commentary.rs  commentary profile, parser, block serialiser, KJV resolution (section 17)
  labels.rs      the scholar's label YAML subset -> meta rows (17.7)
  runs.rs        verse + marks + red pieces → run tree (shared with the app core in M2)
pipeline/data/kjv-versification-overrides.txt   the two overrides, each with its printed-page citation (scholar)
pipeline/tests/fixtures/                        small hand-made OSIS/lexicon files, byte-exact
pipeline/tests/expected/                        oracle files (11.1: kjv-chapters.tsv, kjv-q.tsv, lexicon-entries.tsv) and output-sha256.txt (10.2); owner review
pipeline/tests/golden/*.tsv                     section 12; owner review
```

Tests that need the real sources read their location from `REVELATION_SOURCES`. When it is unset they **fail** if `CI` is set and are skipped (with a printed notice) otherwise, so a CI run can never look green by skipping them.

Phase B, in small reviewable steps, tests first for anything touching content: (1) `sources.rs` + T1; (2) `vrs.rs` + T2; (3) `xmlscan.rs` + T15 fixtures, including the quick-xml configuration test; (4) `osis.rs`, `profile.rs`, `validate.rs` + T6, T9, T10, T12, T13; (5) the oracle files and the CI regeneration job (the Python half, including the strict walker and the "both" fixtures, can be done before the linker exists) + T5; (6) `db.rs`, `open.rs`, `check.rs` + T3, T4, T7, T8, T11, T16, T17, T21; (7) `lexicon.rs` + T14; (8) golden files + T18, T19; (9) `runs.rs` unit tests (T20's pipeline half).

## 17. Commentary: Matthew Henry's Concise Commentary

Added 2026-10-24 for `CMT-MHCC-CW20`: CrossWire MHCC 2.0, `mhcc.osis.xml`, GitLab commit `b15ff40a`, SHA-256 `b00780749c2d6130f41441db2b5cb16df490231e6a315a7d2d9eabcfb36a9fd7` (5,318,822 bytes). Binding: that row of `sources/SOURCES.md` (conditions 1–6). Every number here comes from [`docs/pipeline/mhcc-osis-survey.md`](../docs/pipeline/mhcc-osis-survey.md) and is pinned in `commentary.rs` (`mhcc()`, `CmtFacts`): if one changes, the build fails (`cmt-facts`, condition 2). Code: `commentary.rs` (profile, parser, serialiser, KJV resolution), `labels.rs`, `db.rs` (`CMT_SCHEMA`, `write_cmt`), `check.rs` (`check_cmt`), `run.rs` (`prepare_mhcc`, `publish_mhcc`), `oracle.rs` (`compare_cmt`).

### 17.1 Import scope (condition 5)

Imported: comment paragraphs (`<p>`, `<p subType="x-introduction">`), section headings and book titles (`<title type="x-s2" | "x-IS" | "x-ms" | "main">`), chapter outlines (`<table>` / `<row>` / `<cell>`), book introductions (the `x-introduction` paragraphs before a book's first chapter), bold run-in headings (`<hi type="bold">`), and `<reference osisRef>` cross-references, each with the KJV verse ids of its two ends so the app can link them. The verse and chapter milestones inside a block stay in its tree (they are part of its bytes). Validated but **not** imported: the header, the 198 `milestone type="x-usfm-toc1"`/`2`/`3` book-name milestones, and the converter's 2,053 USFM marker comments (17.3). Anything else (an element, attribute or value the profile doesn't list, in any position) fails closed with a positioned error; a new element type needs a review before import. The pipeline fetches nothing (condition 1); CI fetches only the pinned URL (`.github/scripts/fetch-sources.sh`).

### 17.2 Profile, allowlists and event forms (M1-F3, M1-F5)

The same strict event loop as the KJV (`xmlscan.rs`, sections 9.1–9.4; quick-xml used only as section 9.3 allows), with the commentary's `XmlProfile` (`MHCC_XML`): declaration `<?xml version="1.0" encoding="utf-8"?>` exactly, LF only, double-quoted attributes, no DOCTYPE, no PI, no CDATA. Event forms (`MHCC_FORMS`): `verse`, `chapter` and `milestone` are `Empty` only; every other element (`osis`, `osisText`, `header`, `revisionDesc`, `date`, `work`, `description`, `type`, `identifier`, `refSystem`, `div`, `title`, `p`, `table`, `row`, `cell`, `hi`, `reference`) is a `Start`/`End` container only, so `<p />`, `<hi … />`, `<cell />`, `<title … />`, a container `<verse>…</verse>` and `<chapter …></chapter>` all fail (`form`).

**Empty-event dispatch.** The form is checked first, in `xmlscan`, on (event kind, full name). The commentary walker then handles `Start` and `Empty` in one branch: the only `Empty` events that can reach it are the three milestone elements, and a `Start` pushes the element on the block's own stack while an `Empty` does not. The paired-title check of a comment (17.3) runs after the form check for an `Empty` event, so an empty `<title />` after `<!-- s2 -->` reports its form, not the comment (T24 `empty-title`).

Where each element may sit (anything else: `cmt-structure`):

| Element | Allowed parent | Attributes (exactly these, in this order) |
|---|---|---|
| `div` | `osisText` (`type="book" osisID canonical="true"`: the books in KJV order, 66 in the real file) or a book (`type="introduction"`) | as listed |
| `p` | book or `div type="introduction"` | none, or `subType="x-introduction"` |
| `title` | book or `div type="introduction"` | `type` ∈ {`x-s2`, `x-IS`, `x-ms`, `main`} |
| `table` | book or `div type="introduction"` | none |
| `row` / `cell` | `table` / `row` | none |
| `hi` | `p` | `type="bold"` |
| `reference` | `p` | `osisRef` = `Book.C.V` or `Book.C.V-Book.C.V`, in one book (`cmt-ref`) |
| `verse` | `p` | `sID osisID n`, or `eID` (17.4) |
| `chapter` | book or `p` | `sID osisID n`, or `eID`; chapters `1, 2, …` in order, each closed by its own `eID` (`cmt-chapter`) |
| `milestone` | book | `type` ∈ {`x-usfm-toc1`, `x-usfm-toc2`, `x-usfm-toc3`}, `n` |

Text is allowed only inside blocks (whitespace between blocks), never directly in `table` or `row`; the header's leaf elements may hold text (not imported). Attribute names, order and closed value lists are checked by one function (`attrs_are`, `cmt-attr`), and every start, end and empty tag must have the canonical spelling the file uses (one space before each attribute and before `/>`, nothing else; `cmt-tag`), which is what lets T22 rebuild the bytes.

### 17.3 Untrusted input additions

Section 9 applies unchanged, with three commentary-only rules:

- **Characters.** The raw bytes are LF and printable ASCII (`0x20`–`0x7E`) only, checked before the XML scan (`cmt-char`): no tab, CR, NBSP, bidi control or other non-ASCII byte. (MHCC is ASCII throughout.)
- **`&amp;` is the one entity** (`XmlProfile::amp`, off for the KJV and the lexicons): 8 in the file, all in text, passed on by our own scanner as a text token `&` (quick-xml's unescape is not used), stored as `&` and written back as `&amp;` by the serialiser. `&lt;`, numeric references, a bare `&` and any entity in an attribute fail (`entity`).
- **Comments.** Exactly the four converter strings `<!-- s2 -->`, `<!-- is -->`, `<!-- ms -->`, `<!-- mt -->` are allowed (1,187 + 800 + 61 + 5 = 2,053), each directly followed by the `<title>` of its paired type (`x-s2`, `x-IS`, `x-ms`, `main`), and every such title must be directly preceded by its comment. Any other comment fails as a declaration (`dtd`); a gap, a wrong title or a title without its comment fails (`comment`, `cmt-comment`). Comments are validated, counted against the survey and not stored.

### 17.4 Blocks and anchors: stored exactly as the source lists them

A **block** is a `<p>`, `<title>` or `<table>` inside a book; blocks never nest. 16,255 blocks: 7,014 `p`, 4,657 `p subType="x-introduction"`, 2,531 `table`, 1,187 `title x-s2`, 800 `x-IS`, 61 `x-ms`, 5 `main`; 10 sit before their book's first chapter milestone. Each block records its book (KJV order 1–66), the chapter open at its start tag (`NULL` before the book's first chapter) and its anchor (or `NULL`).

An **anchor** is a `<verse sID … osisID … n … />` … `<verse eID … />` pair: 4,297 anchors (850 single-verse). The parser requires, and never repairs: the `sID` is the first thing in its block and names the open book and chapter as `Book.C.V` or `Book.C.V1-V2` with `V1 < V2`, without leading zeros; `osisID` lists exactly every verse of that range, in order, single-space separated; `n` is `V` or `V1-V2`; no anchor is already open; the `eID` equals the open `sID`, in the same chapter, and nothing but whitespace and a `chapter` milestone follows it in its block; an anchor never spans a chapter or book end. The blocks from the `sID`'s block to the `eID`'s block belong to the anchor, so every anchor is a non-empty run of whole, consecutive blocks (8,078 blocks are in anchors; 3,513 anchors span more than one block).

**The anchoring table is the source's `osisID` lists, verbatim** (`cmt_anchor_verse`, one row per listed verse, in list order): 28,968 rows, 28,718 distinct verses. Every listed verse and both ends of every `<reference>` (1,204) are verses of the table `kjv.sqlite` is checked against (VRS-ENG-PALASO + the KJV overrides; `resolve`, `cmt-kjv`): 0 unresolved. Nothing is inferred, filled in, merged or re-anchored. In particular, as the source has them (survey section 6, pinned):

| Source fact | Count | What the app sees |
|---|---|---|
| KJV verses in no anchor | 2,384 | no comment for that verse. Example: John 3:16. The source's John 3 anchors are 1–8 and 22–36; its comment on 9–21 is inside the 1–8 anchor's text. The chapter's blocks are still reachable by `(book, chapter)` |
| Chapters with no anchor at all | Lev 19, Ps 108 | chapter blocks only |
| Verses listed by two anchors | 250 | both anchors' blocks |
| A verse-1 anchor, then verse 1 listed again | 248 | as written |
| Chapters whose only anchor is a verse-1 anchor | 118 | as written |
| Reused `sID`s | 9: `Isa.4.1`, `Isa.56.1`, `Isa.57.1`, `Isa.58.1`, `Jer.4.1`, `Prov.10.1`, `Song.1.1`, `Song.5.1`, `Song.6.1` | anchors are numbered in document order; `sid` is not unique |
| Descending `<reference>` ranges | 4: `Exod.13.2-Exod.10.13`, `Ezek.3.12-Ezek.3.2`, `Jer.50.1-Jer.3.8`, `Jer.50.4-Jer.7.17` | `ref_start > ref_end`, as written; both ends are KJV verses |

### 17.5 Output file and schema (node-tree storage)

**One database per commentary source:** `mhcc.sqlite`, beside `kjv.sqlite` and `content.sqlite`, so the GPL'd KJV file stays separate (SOURCES.md BIB-KJV-CW31 condition 3) and a commentary can ship, update or be dropped on its own. It is built by the same stages 4–6 as the others (section 2: temp file, one transaction, `integrity_check`, `foreign_key_check`, `VACUUM`, `sync_all`, check of the exact bytes through `open_verified`, then `.sha256`, rename DB then checksum), with the section 8 conventions (`journal_mode = DELETE`, header bytes 18/19 = `01 01`, `application_id = 0x52564C4E`, `user_version = 1`, `STRICT` tables, foreign keys on). The schema is the constant `db::CMT_SCHEMA`; the check stage requires `sqlite_schema` to equal it exactly.

Tables: `meta (key, value)`; `source (id, file, size, sha256)`, two rows: `CMT-MHCC-CW20` and `VRS-ENG-PALASO`; `cmt_anchor (id, sid)`; `cmt_anchor_verse (anchor, k, verse_id)`, indexed on `(verse_id, anchor)`; `cmt_block (id, book, chapter, anchor)`, indexed on `anchor` and on `(book, chapter)`; `cmt_node (block, n, parent, name, text)`; `cmt_attr (block, n, i, name, value, ref_start, ref_end)`. `verse_id` is `kjv.sqlite`'s `verse.id` (book × 1,000,000 + chapter × 1,000 + verse).

**Node-tree storage.** A block is stored as its own XML tree, not as flattened text or HTML: node 0 is the block element; every further node, in document order, is either an element (`name`, with its attributes in `cmt_attr` in source order `i`) or a text node (`text` exactly as in the source, `&amp;` decoded to `&`), and `parent` is its element's `n`. Adjacent text is one node. A `reference@osisRef` attribute row also carries `ref_start` / `ref_end`. So the stored rows alone rebuild every block's source bytes (`serialise_block`: canonical tag spelling, `&` back to `&amp;`), which is T22, and the app renders from the tree without parsing markup at run time.

`meta` keys: `schema_version` (`1`), `commentary_source` (`CMT-MHCC-CW20`), `verse_ids` (the verse-id scheme anchors use), `versification_overrides_sha256`, and the label rows of 17.7 (`label.*`, `label_file`, `label_file_sha256`, `label_cited_from`). No build time, host or commit is stored (10.1).

**Check stage** (`check_cmt`; also the `check` subcommand): schema text, `application_id`, `user_version`, header bytes and indexes (`schema`); `integrity_check` and `foreign_key_check`; the `meta` and `source` rows equal the expected ones (`meta`); SQL invariants on the built file, each required to return no rows (a node's parent is an element node of the same block; attributes sit on element nodes only; a block's nodes are numbered `0..k-1` and every block has nodes; blocks and anchors are numbered `1..N`; every anchor lists at least one verse, `k = 0..k-1`; every anchor is a non-empty run of consecutive blocks and anchors follow one another in document order; an anchored block sits in its anchor's chapter; books and chapters never go backwards; reference verse ids sit exactly on `reference@osisRef`, `cmt-ref`); then T22, and with the KJV table every stored verse id and reference end is a KJV verse (`cmt-kjv`).

**For the app (the comments for the current verse):**

```sql
-- Blocks of every anchor that lists the verse, in document order.
SELECT b.id, b.anchor FROM cmt_anchor_verse v
  JOIN cmt_block b ON b.anchor = v.anchor
 WHERE v.verse_id = ?1
 ORDER BY b.id;
-- Each block's tree.
SELECT n, parent, name, text FROM cmt_node WHERE block = ?1 ORDER BY n;
SELECT n, i, name, value, ref_start, ref_end FROM cmt_attr WHERE block = ?1 ORDER BY n, i;
-- A chapter's blocks (introduction, outline, headings, every anchor), e.g. for a verse in no anchor.
SELECT id, anchor FROM cmt_block WHERE book = ?1 AND chapter = ?2 ORDER BY id;
```

The app opens `mhcc.sqlite` with `open_verified` against the hash compiled in from `output-sha256.txt` (10.2, 10.3) and shows no author, tradition or date label from it until the owner approves the label (17.7; SOURCES.md condition 4; the text itself, W2-C done, is shown under a gated heading).

**Whole-chapter fallback (app).** A verse in no anchor (2,384, e.g. John 3:16; also every verse of Lev 19 and Ps 108) gets no verse comment. The app then shows that chapter's blocks (the last query above) under a scope line that says it is the comment on the chapter ("On John 3 (the whole chapter)"), never a guessed anchor. A reference link with `ref_start > ref_end` (17.4, 4 cases) navigates to `ref_start` and is never expanded.

### 17.6 Verification

| Test | What | Where |
|---|---|---|
| Pinned facts | every count in 17.2–17.4 and the anomaly lists equal the survey, or the build fails (`cmt-facts`) | `commentary::mhcc()`, every build |
| T5 (oracle) | the database equals `pipeline/tests/expected/mhcc-blocks.tsv` (16,255 blocks: book, chapter, anchor, element, attributes, raw-byte length and SHA-256, tag-stripped text length and SHA-256) and `mhcc-anchors.tsv` (4,297 anchors: sID, first and last block, verse count, osisID as written). Both are regenerated in CI by `oracle_mhcc.py` (no XML parser: the regex tokenizer and allowlist walker of `mhcc_osis.py`, no code shared with Rust) and byte-compared with the committed files (11.1). A changed oracle row is reported by ordinal and column | `cmt_real.rs`; `ci.yml` oracle job |
| T16 | two builds are byte-identical and equal the `mhcc.sqlite` line of `output-sha256.txt`; CI's build is checked with `sha256sum -c` | `cmt_real.rs`; `ci.yml` |
| T22 (round trip) | every block rebuilt from its stored rows alone equals its source bytes; every block and anchor equals the parse | `check_cmt`: every build and `check` |
| Counts read back | blocks, anchors, anchor verses, distinct verses, anchored blocks, references, `hi`, descending ranges, the source row, the meta key set and the app's verse query, by plain SQL | `cmt_real.rs` |
| Database mutations | 17 edits of a built file (text; `&amp;` re-encoded; attribute order; an anchor verse moved or made non-KJV; a block moved out of its anchor or chapter; reference ids dropped or made non-KJV; a node re-parented; a block deleted; a meta or label value; the source size; a dropped index; `application_id`; `user_version`) each fail **the named check** (`T22`, `cmt-kjv`, `cmt-ref`, `meta`, `schema`) | `cmt_real.rs` |
| T23 (labels) | `pipeline/data/mhcc-labels.yaml` is byte for byte the one YAML block of `docs/scholar/commentary-labels.md`, and every label row is in `meta` exactly | `cmt_real.rs` |
| T24 (fail closed) | the synthetic `fixtures/mhcc/good.xml` builds a checked file; each of the 65 lines of `fixtures/mhcc/mutations.tsv` (import scope, event forms, malformed anchors, references, chapters, entities, comments, DOCTYPE/CDATA/PI, characters, tag spelling, quoting) is refused by its named check and nothing is published. The 63 lines marked `both` are also refused by the Python reader (`selftest_mhcc.py`, in CI); the 2 marked `rust` need the KJV table | `t24_cmt_fixtures.rs`; `ci.yml` |

### 17.7 Labels (the scholar's metadata)

The panel's name, author, tradition and date come from the scholar's cited metadata, not from SOURCES.md and not from code (SOURCES.md condition 4). `pipeline/data/mhcc-labels.yaml` is a byte copy of the YAML block in `docs/scholar/commentary-labels.md` §5 (T23). `labels.rs` reads it with a strict reader for the small YAML subset it uses (no YAML crate) and writes flat `meta` rows: `label.<key>`, list items as `label.<key>.<i>` (from 1), fields of list items as `label.<key>.<i>.<field>`, and a trailing `# comment` as `<row key>.comment`; values exactly as written, with only a quoted scalar's quotes removed. `label.id` must equal the source id. `label_file`, `label_file_sha256` and `label_cited_from` record where the rows came from. The date "c. 1838" is stored with `label.date_label_status = to_confirm` and its comment, and `label.status = recommended_pending_owner_approval`: the pipeline stores the scholar's values as they are, and the app gates display on the owner's approval.

**The approved value (W2P-A2).** `label.status` has exactly two allowed values: `recommended_pending_owner_approval` (the scholar's recommendation) and **`approved_by_owner`**, the single value that means the owner approved the labels (`labels::LABEL_APPROVED`; the app's `commentary::LABEL_APPROVED`). The reader refuses any other value and any `# comment` on `status`, so a comment can never read like approval. Only the owner's edit of the scholar's doc block (and with it the label file and the expected hash) changes it. While the value is not exactly `approved_by_owner`, the app's Commentary panel shows the text with a heading but no author, tradition or date line, and a muted "Labels awaiting review" line (the heading is `label.display_name` only if one of the `label.citations` quotes it verbatim, else the neutral "Commentary"); once it is, the full source bar of wave-2-layout §7 is drawn from these rows.

**Characters (W2P-A1).** Besides controls, the reader refuses every `General_Category` Cf, Zl and Zp character (bidi marks, embeddings, overrides and isolates, ZWSP/ZWNJ/ZWJ, U+2060, BOM, soft hyphen, tags: `labels::is_invisible_format`), anywhere in the file, fail closed. The committed file has none, so the database bytes and hash are unchanged.

Changing the label changes the doc block, the label file and the expected hash together. (Condition 4 says the metadata lives "in `content.sqlite`"; it is in `mhcc.sqlite`'s `meta`, beside the text it labels. The rule it protects, cited metadata from the database and never hard-coded, is unchanged.)

## 18. Search index: `kjv-search.sqlite`

Added 2026-10-24 (task W2-2p, pipeline half of phrase search; PLAN.md §4 Search, wave-2-layout §6). Code: `revelation_content::search` (normalisation and query, shared by the pipeline and the app), `pipeline/src/search.rs` (`load`, `write`, `SEARCH_SCHEMA`), `check.rs` (`check_search`, `search_norm_oracle`), `run.rs` (`load_search_input`, `publish_search`, `check_published_search`). No new crate or feature: FTS5 is in the bundled SQLite already (18.6).

### 18.1 Layout, and why

- **Its own file, `kjv-search.sqlite`,** beside `kjv.sqlite`. The index is KJV-derived, so it is treated as a GPL work and must not sit in `content.sqlite` or any non-GPL file. Keeping it out of `kjv.sqlite` too leaves that file's schema, hash and checks unchanged, keeps FTS5 (a virtual table with shadow tables) out of the file whose text is displayed, and lets the index be rebuilt or dropped on its own.
- **Built by the pipeline, not on the user's machine** (PLAN.md §4: "FTS5 index built by the pipeline"): the bytes are deterministic, checked against the text at build time and hash-pinned like every other content file, so the app needs no build step, no writable content file and no first-run delay. This replaces the "app builds it on the user's machine" plan of §10.3 item 3 and dependencies §8.4 *for the bundled KJV only*; imported packs (M9) still never bring a pre-built FTS index (18.6).
- **Built from the published `kjv.sqlite`,** read through `open_verified` with the hash its build just returned, never from the parse model: the index is a function of exactly the bytes the app displays. `meta.search_built_from_sha256` records that hash, and the app requires it to equal its compiled-in `kjv.sqlite` hash (`require_search_index`), so ids can't point into a different text.
- **One FTS5 table, `rowid` = `verse.id`** (book × 1,000,000 + chapter × 1,000 + verse). Scope (Bible, OT, NT, a book) is a `rowid BETWEEN` range, canonical order is `ORDER BY rowid`, and the results are verse ids, with no join and no second table.
- **The `ascii` tokenizer over pre-normalised text.** The column `norm` holds only `[a-z0-9]` words separated by single spaces (18.2), so the tokenizer just splits at spaces; FTS5's own folding (`unicode61` diacritics, case tables) never decides what matches, and the normalisation is one documented Rust function used for both the index and the query. `detail = full` (positions, needed for phrases); no `porter` stemming, no prefix indexes.

### 18.2 The normalisation (version `1`, `meta.search_normalisation`)

`revelation_content::search::fold`, one character at a time:

| Character | Becomes |
|---|---|
| `A`–`Z` | the lower-case letter |
| `a`–`z`, `0`–`9` | itself |
| `Æ`, `æ` (U+00C6, U+00E6; "Ænon", John 3:23) | `ae` |
| every other ASCII character (space, punctuation, symbols, controls), every `char::is_whitespace` character, and `SEPARATORS` (`« » ¶ · ‘ ’ ‚ ‛ “ ” „ ‟ … ′ ″ ‹ ›`, soft hyphen, the hyphens and dashes U+2010–U+2015, U+200B, U+FEFF) | a word break, dropped |
| anything else | kept in the word, which makes the word *foreign* |

The words, in order, joined by single spaces, are the normalised text (`normalise`). Consequences, all intended and tested: case and punctuation never matter ("Jesus wept." = "JESUS WEPT"); `LORD’s` is the two words `lord s`; `Beer–sheba` and `son-in-law` are `beer sheba` and `son in law` (so `beersheba` does not match them); numbers are words (`psalm 119`).

- **Index:** `normalise_indexed` fails, naming the verse and the code point, if a verse has a foreign character, and an empty result fails too (`search-norm`): the index never holds a word no query can reach. The real KJV's non-ASCII characters are exactly `Æ – — ’` (asserted on all 31,102 verses).
- **Query:** a word with a foreign character (`café`, Hebrew, an emoji, a combining accent) can never be in the index, so the query has no results and runs no SQL (no guessing what an unknown character "means").
- The normalised text is **search-only**: never displayed, never written anywhere else (PLAN.md 2.5).
- Any change to `fold` is a new `NORMALISATION` version, a new index hash and an owner-reviewed diff of `output-sha256.txt`.

### 18.3 Schema

`SEARCH_SCHEMA`: `meta (key, value)` and `source (id, file, size, sha256)` as in `kjv.sqlite` (one source row, `BIB-KJV-CW31`, copied from `kjv.sqlite`), and

```sql
CREATE VIRTUAL TABLE verse_fts USING fts5(norm, tokenize = 'ascii', detail = full);
```

`meta`: `schema_version` 1, `translation_abbrev` KJV, `search_built_from` kjv.sqlite, `search_built_from_sha256`, `search_normalisation` 1. Section 8 conventions (DELETE journal, `application_id`, `user_version`, header bytes) and the standard stages 4–6 (`db::publish`: temp file, one transaction with the rows in id order, then FTS5 `optimize` so the index is one segment, `integrity_check`, `foreign_key_check`, `VACUUM`, `sync_all`, check of the exact bytes through `open_verified`, `.sha256`, rename). Nothing is published if any check fails. 31,102 rows.

### 18.4 Query semantics (for the app, M8)

`revelation_content::search` (feature `sqlite`):

- `parse_query(input) -> Result<Option<Query>, SearchError>`: at most `MAX_QUERY_CHARS` = 200 code points (`TooLong` otherwise). The words are `tokens(input)`, normalised as in 18.2. A leading quote mark (`"`, `'`, `“`, `‘`, after spaces) makes the whole query `Query::Phrase` (these words, consecutive, in order); otherwise it is `Query::AllWords` (every word, whole words, any order). Quote marks anywhere else are punctuation. `None` means no words (nothing to search, total 0).
- **Safe `MATCH`.** The FTS5 string is built only from the normalised words, which are `[a-z0-9]+` by construction, each wrapped as an FTS5 string literal (`"w"`, any `"` doubled, although none can occur): phrase `"w1 w2 …"`, all words `"w1" "w2" …` (implicit AND). The reader's raw text never reaches `MATCH`, and the FTS5 string is a bound parameter, never SQL text. So `OR`, `AND`, `NOT` and `NEAR` are plain words; `*`, `^`, `-`, `+`, `:`, parentheses, column filters (`norm:`, `{norm}:`) and quotes are word breaks. Nothing a reader types can make FTS5 return a syntax error.
- **Order.** Exact-phrase matches first, then the other all-words matches, canonical (verse-id) order within each. A one-word query and a quoted phrase have only the first group.
- **Totals and paging.** `search(conn, input, scope, offset, limit) -> Result<SearchPage, SearchError>` returns the true total in scope, how many of them are exact-phrase matches, and the hits `offset .. offset + limit` (`PAGE` = 20). `ranked(conn, &Query, scope)` returns every id in order (the app may keep it and page without querying again).
- **Scope.** `Scope::{Bible, OldTestament, NewTestament, Book(1..=66)}`; a `Book` out of range is `BadScope`.
- **Display.** Results are verse ids. The app reads `verse.text` from `kjv.sqlite` and emphasises `match_spans(&verse.text, &query)`: code-point ranges of whole matching words (all words) or of each whole-phrase occurrence (phrase), computed on the stored text with the same `tokens`. FTS5 `snippet()`, `highlight()` and the `norm` column are never shown (§10.3 item 4).

Timings on the real index, debug build: `the` (24,091 results) 8 ms; `and the` 14 ms; `jesus wept` 0.3 ms (PLAN.md target: full-Bible search under 150 ms).

### 18.5 Check stage and verification

`check_search` (every build and the `check` subcommand), on the built bytes through `open_verified`:

- the pinned schema with FTS5's shadow tables (`schema`); `integrity_check` = ok, which also runs FTS5's own check of the index against its content (`integrity`); the `meta` and `source` rows (`meta`);
- SQL invariants returning no rows: every rowid is a well-formed verse id (`search-id`); every `norm` is non-empty `[a-z0-9]` words with single spaces (`search-norm`);
- every verse id of the published `kjv.sqlite` exactly once, in order, and nothing else (`search-id`, `search-count`); the FTS5 content and docsize tables have one row per verse (`search-count`);
- **`norm` equals the documented function of `verse.text` for every verse, recomputed independently** by `search_norm_oracle`, a separate map-then-collapse implementation that rejects any character outside the KJV's known set instead of sharing `fold`'s rules (`search-norm`);
- every verse is found by an FTS5 phrase query of its own whole normalised text, built by the app's `Query::fts_phrase` (`search-fts`).

Tests: `crates/revelation-content/tests/search.rs` (FTS5 compiled in and the SQLite version; a small index built there and opened with `open_bytes`; `jesus wept` gives John 11:35 first; case and punctuation; whole words, no prefix or stemming; quoted vs all words; phrase-first order, true totals and paging; scope; FTS5 syntax as literal text; a scan oracle; identity checks) and `pipeline/tests/search_real.rs` (the real build, checked, built twice byte-identical and equal to `output-sha256.txt`; every verse once and normalised as documented, by two oracles; John 11:35 first; **a brute-force oracle over all 31,102 verses, with its own tokenizer, agrees with FTS5 on the id set, the exact-phrase part and the order for 110 queries (49 ordinary, 61 hostile: FTS5 operators, column filters, quotes, SQL, controls, non-ASCII) × 6 scopes = 660 pairs**; every returned id is a `kjv.sqlite` verse with a non-empty emphasis span; whole words and true totals, OT + NT = Bible; ten database mutations each fire the named check; a verse the normalisation does not cover stops the build and publishes nothing).

### 18.6 M1-F8 and opening the index

- `kjv-search.sqlite` is **not a pack**: our pipeline builds it from our own verified file, and its SHA-256 is committed in `output-sha256.txt` and compiled into the app like `kjv.sqlite`'s (§10.2). §10.3 item 3 (never open FTS5 tables from a pack) is unchanged for packs and imported translations; their search, if any, is built by the app from their `verse.text`.
- The app opens it like every content file (§10.3 items 1–2): `open_verified` against the compiled-in hash (read once, hash, `sqlite3_deserialize`), `SQLITE_DBCONFIG_DEFENSIVE` (which also keeps FTS5's shadow tables read-only), `trusted_schema = OFF`, triggers and views disabled, `query_only`, `cell_size_check`, no extensions; then `require_content_identity` and `require_search_index(conn, KJV_SHA256)`. All the tests query through that open.
- FTS5 is compiled into the bundled SQLite (libsqlite3-sys `bundled` passes `-DSQLITE_ENABLE_FTS5`), proved by a runtime test (`PRAGMA compile_options`, `sqlite_compileoption_used`, creating an `ascii` table), which also requires the SQLite version that fixes the 2026 FTS5 memory-safety CVEs.
- Display never comes from the index (18.4).
