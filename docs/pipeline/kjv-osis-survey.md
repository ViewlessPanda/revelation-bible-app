# KJV OSIS and lexicon format survey

- **Method:** scripts in `docs/pipeline/survey-scripts/`, run on the pinned files
- **Date:** 2026-10-20
- **Status:** complete for Phase A. Read-only: no source file was changed, and nothing here was typed from memory. Every count comes from a script run on the pinned files after its SHA-256 was checked.
- **Feeds:** [`pipeline/DESIGN.md`](../../pipeline/DESIGN.md)

## 0. Summary

| Fact | Value |
|---|---|
| KJV file | `sources/raw/BIB-KJV-CW31/kjv.osis.xml`, 28,043,075 bytes, SHA-256 `4bcbfe45…0e3f5` (matches `sources/checksums/BIB-KJV-CW31.sha256`) |
| Encoding | UTF-8 (strictly valid), no BOM, LF only (34,989 LF, 0 CR), no NUL or other C0 controls except 199 TABs (all in the header, none in verse text) |
| DTD, entities, CDATA, comments, PIs | none. The XML declaration is the only `<?`, and there are no `&` bytes at all |
| Verses | **31,102** `sID`/`eID` milestone pairs, all balanced, every chapter numbered 1..n, 66 books and 1,189 chapters in canonical order |
| Versification check | per-chapter counts equal **VRS-ENG-PALASO plus the two KJV overrides** in all 1,189 chapters (table total 31,102) |
| Words of Jesus | **648** `<q who="Jesus" marker="">` containers. No other `q` exists. Never nested, never inside a note, never between verses |
| Cross-boundary `q` | **280** cross at least one verse boundary (longest: Matt 24:4 to 24:51, 48 verses). **0** cross a chapter boundary |
| Red ranges per verse | 648 `q` become **2,035** per-verse ranges in 2,026 verses; **210,502** red characters (code points = UTF-16 units here; 210,668 UTF-8 bytes) |
| Candidate verse text | 4,113,007 bytes; the only whitespace is U+0020; no leading spaces, no double spaces; **91 verses end with a space** |
| Anomalies (reported, not fixed) | 1 malformed lemma token (2 Cor 8:10); 91 trailing spaces; 6,331 empty `<w/>` elements; en dash used for hyphen in 786 places (hyphen in 12); one `Æ` left in John 3:23 despite the header's "All æ change for e" |

"Candidate verse text" in this file means: all character data between a verse's `sID` and `eID` milestones, in document order, minus everything inside `<note>` subtrees. That is the definition proposed in DESIGN.md section 3; this survey measured it.

## 1. How this was produced

All scripts are in [`survey-scripts/`](survey-scripts/), run with `py -I -B` (Python 3.8.1) from outside the raw directories, with paths as arguments. `-B` (and `sys.dont_write_bytecode` in each script) keeps `__pycache__` out of the repo. None uses an XML parser: they tokenize bytes with strict regular expressions and raise on anything they don't recognise, so no DTD or entity is ever processed.

Since review items M1-F3 and M1-F13, every script that reads a pinned source checks it with the one shared `load_pinned()`/`check_sha()` in `osis_tokens.py` first: the file is read once, those bytes are hashed and compared with `sources/checksums/<ID>.sha256` (exact format, matching file name), and the script refuses on any mismatch. Every script that tokenizes the KJV then runs the one shared strict walker (`strict_tokenize()`) over the **whole** file before it writes anything. Output files are written only after all checks pass.

| Script | Output | What it does |
|---|---|---|
| `basic_facts.py` | encoding, line endings, control bytes, `<!`/`<?`/`&` counts, character and tag inventories | byte-level only; checks the SHA-256 first |
| `osis_tokens.py` | (library) | `load_pinned()`/`check_sha()`; strict tokenizer (refuses `&`, `<!`, `]]>` or a second `<?`); strict walker with the KJV profile: every element name and token kind is allowlisted (`verse` and `milestone` empty only, `w` either, everything else start/end only, so an empty `q`, `transChange`, `divineName`, `note` or `inscription` fails), end tags must match the open element, and a nested `q`, a `q` inside a note, a milestone `q` (`sID`/`eID`) or a `q` whose attributes aren't exactly `who="Jesus" marker=""` fails |
| `survey_kjv.py` | structure, element inventory, `q` statistics, per-book red totals, text facts | checks the SHA-256 and runs the strict walker first |
| `survey_kjv_detail.py` | versification comparison, inter-verse text, whitespace, `lemma` tokens, `q` boundary context | checks both SHA-256s and runs the strict walker first |
| `survey_lexicons.py` | both lexicons | checks both SHA-256s first; locates and skips MGNT's DTD subset by bytes after checking it declares no entities or defaults; checks end-tag names itself (the KJV walker doesn't apply to the lexicons) |
| `survey_misc.py` | inscription verses, trailing-space list, dash/apostrophe/bracket counts, Strong's keys against both lexicons, testament check, lexicon line breaks, size facts for the parser caps | checks SHA-256s and runs the strict walker on the KJV first |
| `survey_excerpts.py` | the appendix of this file | checks SHA-256s; copies excerpts byte for byte |
| `golden_excerpts.py` | [`golden-source-excerpts.md`](golden-source-excerpts.md) | checks the SHA-256 and runs the strict walker first; refuses if the golden-samples YAML block has no `ref:` |
| `selftest_strict.py` | PASS/FAIL lines | runs the synthetic fixtures in `survey-scripts/fixtures/` (placeholder words only, no Scripture): each `fail-*` fixture must be refused by the walker and by every KJV-reading script, with no output written; checksum refusals are tested the same way |

How to run (from the repository root; `<out>` is a directory outside the repo, for example the session scratchpad):

```text
py -I -B docs/pipeline/survey-scripts/selftest_strict.py docs/pipeline/survey-scripts/fixtures <new empty dir>
py -I -B docs/pipeline/survey-scripts/basic_facts.py sources/raw/<ID>/<file> sources/checksums/<ID>.sha256
py -I -B docs/pipeline/survey-scripts/survey_kjv.py sources/raw/BIB-KJV-CW31/kjv.osis.xml sources/checksums/BIB-KJV-CW31.sha256 <out>
py -I -B docs/pipeline/survey-scripts/survey_kjv_detail.py sources/raw/BIB-KJV-CW31/kjv.osis.xml sources/checksums/BIB-KJV-CW31.sha256 sources/raw/VRS-ENG-PALASO/eng.vrs.txt sources/checksums/VRS-ENG-PALASO.sha256 <out>/detail.txt
py -I -B docs/pipeline/survey-scripts/survey_lexicons.py sources/raw/LEX-HEB-OSHB/HebrewStrong.xml sources/checksums/LEX-HEB-OSHB.sha256 sources/raw/LEX-GRC-MGNT19/strongsgreek.xml sources/checksums/LEX-GRC-MGNT19.sha256 <out>/lex.txt
py -I -B docs/pipeline/survey-scripts/survey_misc.py sources <out>/misc.txt
py -I -B docs/pipeline/survey-scripts/survey_excerpts.py sources <out>/excerpts.md
py -I -B docs/pipeline/survey-scripts/golden_excerpts.py sources/raw/BIB-KJV-CW31/kjv.osis.xml sources/checksums/BIB-KJV-CW31.sha256 docs/scholar/red-letter-golden-samples.md <out>/golden.md
```

`basic_facts.py` and `survey_kjv.py` now take the checksum file as an extra argument (second position). After those changes, every script's output on the pinned files was byte-identical to its output before the changes (2026-10-20), except `golden_excerpts.py`, which had silently found 0 references after the golden-samples file changed its row style; it now reads both styles and refuses on 0.

A few details were located with throwaway probes (scratchpad Python with `py -I`, or `grep` on the raw file) rather than the scripts above: the example references in section 5's boundary list (Mark 12:1, Luke 10:21, John 13:7, Acts 1:5; the counts themselves come from `survey_kjv_detail.py`), the 32,110 `\n` nodes directly inside `<chapter>`, the five chapters without `chapterTitle`, and the absence of empty `transChange`/`divineName`/`q`/`inscription` elements. They are easy to fold into a script if anyone wants them reproducible.

`golden_excerpts.py` produced [`golden-source-excerpts.md`](golden-source-excerpts.md) (99 references) in the earlier session.

**Versification input.** The comparison in section 4 uses VRS-ENG-PALASO. I did not download it: a copy was already in the earlier session's scratchpad. Its SHA-256 (`003981c7…57541`) and git blob SHA-1 (`2e994259…cb8654`) both match the SOURCES.md row, so it is the pinned file. I copied it to `sources/raw/VRS-ENG-PALASO/eng.vrs.txt` (read-only) and added `sources/checksums/VRS-ENG-PALASO.sha256`. The two overrides (3 John 1 = 14, Rev 12 = 17) are applied in the script, not to the file. The scholar's printed-page confirmation of the overrides (condition 2) is still pending.

## 2. Bytes and encoding (KJV)

- UTF-8, strictly valid; no BOM; XML declaration `<?xml version="1.0" encoding="UTF-8"?>`, immediately followed by `<osis …>` on the same line.
- Line endings: 34,989 LF, 0 CRLF, 0 bare CR. The file ends with `</osisText>\n</osis>\n`.
- Control bytes: 0 NUL; 199 TAB, all in the `<header>` indentation; no other C0 bytes.
- Non-ASCII: 55 distinct characters in the whole file, mostly Greek in `lemma.TR:` attribute values and 22 Hebrew letters in Psalm 119's acrostic titles. **Inside candidate verse text only four occur:** U+2019 `’` (1,996), U+2013 `–` (786), U+2014 `—` (1, Exod 32:32 "sin—;"), U+00C6 `Æ` (1, John 3:23 "Ænon"). No character outside the BMP occurs anywhere, so code-point and UTF-16 offsets are equal for this file (UTF-8 byte offsets are not).
- Of 31,102 verses, 28,722 are pure ASCII.

## 3. Markup declarations and references (KJV)

| Construct | Count |
|---|---|
| `<!DOCTYPE`, `<!ENTITY`, `<!ELEMENT`, `<!ATTLIST`, `<!NOTATION`, `SYSTEM`, `PUBLIC` | 0 each |
| `<![CDATA[` | 0 |
| `<!--` comments | 0 |
| `<?` processing instructions | 1 (the XML declaration) |
| `&` (any entity or character reference) | 0 |
| XInclude | 0 |

The root has `xsi:schemaLocation` pointing at `http://www.crosswire.org/~dmsmith/osis/osisCore.2.1.1-cw-latest.xsd`. It must never be fetched (DESIGN.md section 9).

## 4. Verse markup and counts

- Every verse is a pair of empty milestones, `<verse osisID="X" sID="X"/>` … `<verse eID="X"/>` (excerpt K1). There are 31,102 of each; `osisID` always equals `sID`; every `eID` closes the verse that is open; verses never nest or overlap.
- Verse milestones are direct children of `<chapter>` in 29,715 cases and of `<q>` in 1,387 (the verse starts while a red span is open). `<chapter>` is a container; no verse milestone is ever outside its chapter.
- No `w`, `transChange`, `divineName` or `inscription` element is open at any verse milestone, so those never cross a verse boundary. Only `q` does.
- Chapters: 1,189, all numbered 1..n per book. 1,184 carry `chapterTitle` (the five that don't are the one-chapter books Obad, Phlm, 2John, 3John, Jude).
- Between verses there is only whitespace: every character-data node directly inside `<chapter>` outside a verse is a single `\n` (32,110 nodes). Titles sit between verses as elements (section 7).

**Per-book verse counts against VRS-ENG-PALASO plus overrides.** eng.vrs gives 1,189 chapters / 31,104 verses for the 66 books; with `3JN 1: 15 → 14` and `REV 12: 18 → 17` it gives 1,189 / **31,102**. The file matches the resulting table **in every chapter** of every book:

| Book | Ch | Verses | Book | Ch | Verses | Book | Ch | Verses |
|---|---|---|---|---|---|---|---|---|
| Gen | 50 | 1,533 | Isa | 66 | 1,292 | Rom | 16 | 433 |
| Exod | 40 | 1,213 | Jer | 52 | 1,364 | 1Cor | 16 | 437 |
| Lev | 27 | 859 | Lam | 5 | 154 | 2Cor | 13 | 257 |
| Num | 36 | 1,288 | Ezek | 48 | 1,273 | Gal | 6 | 149 |
| Deut | 34 | 959 | Dan | 12 | 357 | Eph | 6 | 155 |
| Josh | 24 | 658 | Hos | 14 | 197 | Phil | 4 | 104 |
| Judg | 21 | 618 | Joel | 3 | 73 | Col | 4 | 95 |
| Ruth | 4 | 85 | Amos | 9 | 146 | 1Thess | 5 | 89 |
| 1Sam | 31 | 810 | Obad | 1 | 21 | 2Thess | 3 | 47 |
| 2Sam | 24 | 695 | Jonah | 4 | 48 | 1Tim | 6 | 113 |
| 1Kgs | 22 | 816 | Mic | 7 | 105 | 2Tim | 4 | 83 |
| 2Kgs | 25 | 719 | Nah | 3 | 47 | Titus | 3 | 46 |
| 1Chr | 29 | 942 | Hab | 3 | 56 | Phlm | 1 | 25 |
| 2Chr | 36 | 822 | Zeph | 3 | 53 | Heb | 13 | 303 |
| Ezra | 10 | 280 | Hag | 2 | 38 | Jas | 5 | 108 |
| Neh | 13 | 406 | Zech | 14 | 211 | 1Pet | 5 | 105 |
| Esth | 10 | 167 | Mal | 4 | 55 | 2Pet | 3 | 61 |
| Job | 42 | 1,070 | Matt | 28 | 1,071 | 1John | 5 | 105 |
| Ps | 150 | 2,461 | Mark | 16 | 678 | 2John | 1 | 13 |
| Prov | 31 | 915 | Luke | 24 | 1,151 | 3John | 1 | 14 |
| Eccl | 12 | 222 | John | 21 | 879 | Jude | 1 | 25 |
| Song | 8 | 117 | Acts | 28 | 1,007 | Rev | 22 | 404 |

Per-chapter counts are in the script output (`chapter_counts.tsv`); the DESIGN.md versification test compares every chapter, not just book totals.

## 5. Words of Jesus: `<q who="Jesus">`

**Form.** Always exactly `<q who="Jesus" marker="">` … `</q>`: a container element, not a milestone (no `sID`/`eID` form of `q` exists in the file). All 648 have the same two attributes with the same values. `marker=""` means no quotation mark is to be generated. There is **no** `q` without `who="Jesus"`, and no other words-of-Jesus markup.

**Placement.** Every `<q>` start tag and every `</q>` end tag is inside a verse (0 open or close between verses). `q` never nests, never occurs inside `<note>`, and no `<note>` occurs inside a `q`.

**Crossing boundaries.** 368 `q` stay inside one verse; **280** cross one or more verse boundaries; **0** cross a chapter boundary. By number of verses touched: 1: 368, 2: 110, 3: 43, 4: 33, 5: 19, 6: 12, 7: 10, 8–48: 53. Longest: Matt 24:4–51 (48 verses), Matt 5:3–48 and Matt 25:1–46 (46), Matt 10:5–42 and Matt 23:2–39 (38). 1,387 verses start with a `q` already open, so their start tag is in an earlier verse (excerpt K4).

**Boundary context** (what is next to the tags, in candidate verse text):
- Before `<q>`: a U+0020 space in 612 cases; the verse start in 36.
- After `</q>`: the verse end in 537 cases; a U+0020 space in 111.
- So every red boundary sits at a verse edge or next to a space. No red range starts or ends with whitespace, and none is empty.
- First token inside `q`: `<w>` 623, empty `<w/>` 19, `<transChange>` 4, text 2 (Mark 12:1 "A ", Luke 10:21 "I "). Last token before `</q>`: text (punctuation) 646, empty `<w/>` 2 (John 13:7, Acts 1:5).

**Per book** (`q` counted in the book of its start tag; ranges and characters per verse):

| Book | `q` | cross a verse | verses with red | red ranges | red chars (code points) | UTF-8 bytes |
|---|---|---|---|---|---|---|
| Matt | 146 | 69 | 644 | 646 | 69,530 | 69,588 |
| Mark | 131 | 52 | 285 | 285 | 27,482 | 27,504 |
| Luke | 185 | 92 | 587 | 589 | 62,696 | 62,748 |
| John | 153 | 57 | 419 | 422 | 39,461 | 39,489 |
| Acts | 21 | 6 | 27 | 28 | 2,479 | 2,481 |
| 1Cor | 2 | 0 | 2 | 2 | 181 | 181 |
| 2Cor | 1 | 0 | 1 | 1 | 77 | 77 |
| Rev | 9 | 4 | 61 | 62 | 8,596 | 8,600 |
| **Total** | **648** | **280** | **2,026** | **2,035** | **210,502** | **210,668** |

UTF-16 code units equal code points in every row (no astral characters). No red in any Old Testament book. Nine verses have two red ranges: Matt 9:6, Matt 21:31, Luke 5:24, Luke 8:8, John 21:15, 21:16, 21:17, Acts 1:4, Rev 1:11.

**Inside `q`:** `w` 29,943; `transChange` 633; pilcrow `milestone` 68; `divineName` 3 (inside `w`); verse milestones (as above). Nothing else.

## 6. What occurs inside verses

**Elements** (start or empty tags between `sID` and `eID`):

| Element | Count | Notes |
|---|---|---|
| `w` | 355,295 | Strong's carrier. 6,331 of all `w` in the file are empty `<w …/>` (a Strong's number with no English word, excerpt K2), 1,335 of them inside `q` |
| `transChange` | 21,580 | 21,571 in verse text, all `type="added"` (italic supplied words); 9 inside notes (6 `type="amplified"`, 3 `added`) |
| `rdg` | 7,404 | only inside notes |
| `note` | 6,959 | all `type="study"`; 218,635 bytes of text; excluded from verse text in M1 (excerpt K10) |
| `catchWord` | 6,958 | only inside notes |
| `divineName` | 6,952 | 6,887 in verse text, 65 inside notes |
| `milestone` | 2,970 | all `type="x-p" marker="¶"` (1,407 also `subType="x-added"`); always at the very start of a verse's text; 68 inside `q` |
| `q` | 648 | section 5 |
| `inscription` | 14 | Exod 28:36, 39:30; Dan 5:25–28; Zech 14:20; the title on the cross (Matt 27:37, Mark 15:26, Luke 23:38, John 19:19); Acts 17:23; Rev 17:5, 19:16. Text already in capitals (excerpt K9) |

Paths (excluding the `osis/osisText/div/chapter` ancestors): `w` 325,289; `q/w` 29,943; `transChange` 20,938; `note/rdg` 7,396; `note` 6,959; `note/catchWord` 6,958; `w/divineName` 6,867; `milestone` 2,902; `q` 648; `q/transChange` 633; `q/milestone` 68; `note/rdg/divineName` 46; `inscription/w` 44; `transChange/w` 19; `note/catchWord/divineName` 19; `inscription` 14; `transChange/divineName` 13; `note/rdg/rdg` 8; `note/transChange` 6; `inscription/w/divineName` 3; `q/w/divineName` 3; `note/rdg/transChange` 2; `divineName` 1; `note/catchWord/transChange` 1. Maximum element depth in the file is 8.

**Where verse text sits** (bytes of candidate text by innermost element, `q` ignored): inside `w` 3,455,536; directly in the verse (spaces and punctuation between elements) 534,636; `transChange` 95,117; `divineName` 27,683; `inscription` 35.

**`divineName` text values:** `Lord` 6,461; `God` 308; `Lord’s` 108; `LORD` 5; `Jehovah` 4; `Jah` 1. The text is mixed case, and print shows it in small capitals (LORD, GOD, JEHOVAH, JAH). The `LORD` cases are the five inscriptions the header's MOD-448 note lists.

**`lemma` attribute.** Every `w` has one. Tokens: `strong:` 374,069; `lemma.TR:` 144,788 (not imported in M1); other 1. Shapes: Hebrew is always `H0` plus the number (`H07225`, `H0430`, `H01`): 5 digits 191,025, 4 digits 35,527, 3 digits 1,288, 2 digits 1,427, every one with a leading zero. Greek is never padded (`G3778`, `G1`): 4 digits 126,145, 3 digits 16,785, 2 digits 1,848, 1 digit 24. 18,142 `w` carry more than one `strong:` token (Gen 1:1 "created" = `strong:H0853 strong:H01254`). No `G` key appears in the Old Testament and no `H` key in the New. `morph` (199,999 values, `robinson:` and `strongMorph:` prefixes) is not imported in M1.

All 14,089 distinct keys exist in the lexicons (8,674 Hebrew, 5,415 Greek); none points to one of the Greek lexicon's 101 "Not Used" placeholders.

**Whitespace inside verses.** 770,863 text nodes inside verses, 256,343 whitespace-only. In candidate verse text the only whitespace character is U+0020 (758,617 of them): no TAB, LF, CR or NBSP; no verse starts with a space; no verse contains two consecutive spaces (also after notes are removed); **91 verses end with a space** (section 8).

## 7. Text outside verses (not verse text)

| What | Element | Count | Example |
|---|---|---|---|
| Generated chapter headings | `title type="chapter"` | 1,184 | `CHAPTER 1.`, `PSALM 3.` (CrossWire's, not KJV text) |
| Book titles | `title type="main"` | 66 | `THE FIRST BOOK OF MOSES CALLED GENESIS` |
| Psalm superscriptions | `title type="psalm" canonical="true"` | 116 | "A Psalm of David, when he fled from Absalom his son." (excerpt K11), with `w`/Strong's |
| Psalm 119 acrostic headings | `title type="acrostic" canonical="true"` | 22 | `<foreign xml:lang="hbo">א ALEPH.</foreign>` (excerpt K12) |
| Pauline colophons | `div type="colophon"` | 14 | Rom.c "Written to the Romans from Corinthus, …" (excerpt K13), with `w`/Strong's |
| Work title | `title` in `<header>` | 1 | "King James Version (1769) with Strongs Numbers and Morphology" |
| Revision history | `header/revisionDesc` | 11 | dated edit notes (useful provenance for the scholar) |

555 `w` elements are outside verses (in titles and colophons), plus 48 `transChange`, 22 `foreign`, 6 `divineName` and 5 `abbr expansion="Saint"`. **None of this is verse text and none of it is imported in M1** under the import scope. The psalm superscriptions, acrostic headings and colophons are printed in the KJV and marked `canonical="true"`, so leaving them out is a completeness gap in the reader; see DESIGN.md open question 3.

## 8. Anomalies (reported, not fixed)

1. **Malformed `lemma` token, 2 Cor 8:10** (excerpt K7): `lemma="strong:G1722 G5129 lemma.TR:εν lemma.TR:τουτω"`. The second token lacks the `strong:` prefix. It is the only token in the file that is neither `strong:…` nor `lemma.TR:…`. The pipeline must not guess what it means; DESIGN.md section 6 treats it as a listed, reviewed exception. Worth reporting to CrossWire.
2. **Trailing space before `eID` in 91 verses** (excerpt K6), e.g. Matt 8:9 `…<transChange type="added">it</transChange>. <verse eID="Matt.8.9"/>`. 90 are in the New Testament, plus 2 Kgs 12:16. 82 follow a `</transChange>`. The space is character data inside the verse, so it is part of the stored text (no trimming). None of these verses ends inside a red range.
3. **Empty `<w …/>` elements**: 6,331 carry Strong's numbers for source words with no English rendering (mostly the Greek article G3588), e.g. Matt 3:17 starts with `<w src="13" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/>`. They add no characters. Stored as zero-length word links (DESIGN.md section 6).
4. **En dash for hyphen.** U+2013 occurs 786 times in verse text, in compound names (`Tubal–cain`, Gen 4:22; `Beth–el`), while U+002D occurs 12 times (`God-ward`, `joint-heirs`, `Maran-atha`, …). Printed KJVs use a hyphen in both. It's a property of this digital edition, kept as is; the scholar may want to note it in SOURCES.md.
5. **`Æ` remains once.** The header's 2023-07-10 note says "All æ change for e", but John 3:23 still reads "Ænon". Kept as is; the scholar should know, since the header claims otherwise.
6. **Apostrophe is always U+2019** (1,996; no U+0027 in verse text). Consistent, not an error, but it means the search index must fold `’` and `'` (search-only normalisation, PLAN.md 2.5).
7. `1 John 2:23` contains literal square brackets, `[but]`, as in printed KJVs (the second half of the verse is in italics in print). Not an error; noted because brackets are characters here, not markup.
8. Header typo: "Change the spelling of 'Cesar' to 'Cesar'" (2023-07-06 note). Irrelevant to the text; noted for completeness.

Things checked and **not** found: nested `q`; `q` without `who="Jesus"`; `q` opening or closing between verses; empty `q`, `transChange`, `divineName` or `inscription`; `transChange` crossing a verse; text directly inside `<chapter>` other than `\n`; verses out of order or missing; Greek keys in the OT or Hebrew keys in the NT; Strong's numbers missing from the lexicons.

## 9. Lexicons

### 9.1 `HebrewStrong.xml` (LEX-HEB-OSHB)

- 2,749,042 bytes; SHA-256 matches. UTF-8, valid, no BOM, `<?xml version="1.0" encoding="utf-8"?>`.
- **CRLF** line endings throughout (51,928 CRLF, no bare LF or CR); no final newline (ends `</entry>\r\n</lexicon>`); 86,506 TABs (indentation). No `<!`, no comments, no CDATA, no `&`.
- Root `<lexicon>` with a default namespace `http://openscriptures.github.com/morphhb/namespace` and an `xsi:schemaLocation` (`StrongSchema.xsd`, a relative path, never to be opened).
- **8,674 entries**, `id="H1"` … `id="H8674"`, ascending, no gaps, no duplicates, no padding.
- Elements: `w` 17,784; `def` 15,469; `entry` 8,674; `usage` 8,673; `source` 8,668; `meaning` 8,435; `note` 167. Max depth 4. Paths: `entry/w` (headword, 8,674), `entry/source` (with `w` cross-references 8,755 and `def` 1,315), `entry/meaning/def` 14,154, `entry/usage` (336 `w` inside), `entry/note` 128, `entry/source/note` 38, `entry/meaning/note` 1.
- Headword `w`: attributes `pos` (52 distinct values, e.g. `n-m`, `v`, `n-pr-loc`), `pron`, `xlit`, `xml:lang` (`heb` 5,393, `x-pn` 2,628, `arc` 653). Cross-reference `w`: `src="H24"` (7,934), always resolving to an existing entry.
- Irregularities: one empty `<source/>` (H5223, excerpt H2); one entry without `<usage>` (H3390); 239 entries without `<meaning>`.
- The 167 `<note>` elements are OSHB's documented corrections ("number 250, corrected to 251", "lemma אי missing vowel, corrected to אִי"). They are part of the lexicon's own text.
- Text directly inside `<entry>` is whitespace only (CRLF and TABs between child elements).

### 9.2 `strongsgreek.xml` (LEX-GRC-MGNT19)

- 2,369,537 bytes; SHA-256 matches. UTF-8, valid, no BOM, `<?xml version='1.0' encoding='utf-8' standalone='yes'?>` (single quotes). LF only; 3 TABs (in the prologue). No `&`.
- **Internal DTD subset**: `<!DOCTYPE strongsdictionary [ … ]>` at bytes 56..1090 (1,034 bytes, SHA-256 `9b89a286…41f6`, excerpt G1): 12 `<!ELEMENT`, 4 `<!ATTLIST` (entry, greek, pronunciation, strongsref), 1 comment, **0 `<!ENTITY`, 0 parameter entities, 0 `SYSTEM`/`PUBLIC`, 0 `#FIXED`, 0 default attribute values** (all `#REQUIRED`). The one comment in the file is inside the subset. So a parser that skips the subset loses nothing: the only non-CDATA attribute is `strongsref@language` (an enumeration), and its values `GREEK`/`HEBREW` contain no whitespace to normalise. The pipeline must still reject the DTD as a construct and skip only this exact, hash-pinned subset (DESIGN.md section 9).
- Root `strongsdictionary` → `prologue` (the 1890/2006 credits and "Public Domain -- Copy Freely") and `entries`.
- **5,624 entries**, `strongs="00001"` … `strongs="05624"` (always five digits), ascending, no gaps, no duplicates. Each has `<strongs>` text equal to the number without padding (`<strongs>1</strongs>`). **101 are "Not Used" placeholders** (2717 and 3203–3302; excerpt G3).
- Elements: `strongsref` 7,063 (empty; `language="GREEK"` 6,746, `HEBREW` 317; `strongs` is unpadded for Greek but zero-padded for Hebrew, e.g. `0175`, `011`); `greek` 6,139 (empty; the Greek is in attributes `unicode`, `translit`, `BETA`); `pronunciation` 5,652 (empty; text in `@strongs`); `kjv_def` 5,522; `strongs_derivation` 5,510; `strongs_def` 5,507; `latin` 12. Max depth 5. Every `strongsref` resolves to an existing, used entry.
- **Mixed content matters here:** 324 entries have non-whitespace text directly inside `<entry>`, outside any child element (excerpt G2: entry 00001's "Often used (usually …"). 2,900 `strongs_def` texts contain hard line feeds from the original ASCII e-text, and several start with a space. All of it is display text and is kept exactly.
- Some display text is in attributes (`greek@unicode`, `greek@translit`, `pronunciation@strongs`). No attribute value contains TAB, CR or LF.

## 10. Commands run (2026-10-20)

From the repository root, scratch output directory `…\scratchpad\out`. This is the record of the original run; the current argument lists are in section 1 (`basic_facts.py` and `survey_kjv.py` now also take the checksum file).

```text
py -I docs/pipeline/survey-scripts/basic_facts.py <each of the 3 files>          exit 0 (PYTHONIOENCODING=utf-8)
py -I docs/pipeline/survey-scripts/survey_kjv.py <kjv> <out>                      exit 0; survey.txt byte-identical to the 2026-10-19 run
py -I docs/pipeline/survey-scripts/survey_kjv_detail.py <kjv> <kjv.sha256> <eng.vrs.txt> <vrs.sha256> <out>   exit 0
py -I docs/pipeline/survey-scripts/survey_lexicons.py <heb> <heb.sha256> <grc> <grc.sha256> <out>              exit 0
py -I docs/pipeline/survey-scripts/survey_misc.py sources <out>                   exit 0
py -I docs/pipeline/survey-scripts/survey_excerpts.py sources <appendix>          exit 0
sha256sum -c sources/checksums/<ID>.sha256   (in each raw dir)                    4 x OK
git hash-object sources/raw/VRS-ENG-PALASO/eng.vrs.txt                            2e994259e1df38a8c6410bd4c3a24df2f4cb8654 (matches SOURCES.md)
```

## Appendix: machine-extracted excerpts

Generated by `docs/pipeline/survey-scripts/survey_excerpts.py`; do not edit by hand. Each block is the exact bytes at the given 0-based offset of the named file (CR bytes, if any, are shown as they are in the file).

### K1. Gen 1:1, a whole verse (milestones, `w`, `lemma`, punctuation outside `w`)

`kjv.osis.xml` offset 4541, length 326, SHA-256 `7984137f2443b5c260ae118cdcc819490f55f6ec4913d620bcf491ef704239eb`

```xml
<verse osisID="Gen.1.1" sID="Gen.1.1"/><w lemma="strong:H07225">In the beginning</w> <w lemma="strong:H0430">God</w> <w morph="strongMorph:TH8804" lemma="strong:H0853 strong:H01254">created</w> <w lemma="strong:H08064">the heaven</w> <w lemma="strong:H0853">and</w> <w lemma="strong:H0776">the earth</w>.<verse eID="Gen.1.1"/>
```

### K2. Matt 3:17, start: an empty `<w/>` carrying a Strong's number with no English text

`kjv.osis.xml` offset 15267442, length 189, SHA-256 `5e0ca41181bf1b3cfe06e1f73762f4cdcb9a92fac2914613fb57647b30f20c20`

```xml
<verse osisID="Matt.3.17" sID="Matt.3.17"/><w src="13" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w>
```

### K3. Matt 3:15, a red span opening mid-verse (space before `<q>`, `marker=""`)

`kjv.osis.xml` offset 15263792, length 194, SHA-256 `b8f412d0bfc3f47263cea0188917c8ba980c9817360af729498980a3a3a6a056`

```xml
<w src="7" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G863 lemma.TR:αφες" morph="robinson:V-2AAM-2S">Suffer</w>
```

### K4. John 3:10 to 3:11, a red span crossing a verse boundary (`<q>` stays open across the milestones)

`kjv.osis.xml` offset 19930859, length 241, SHA-256 `9b6c8b4effb14bafcaa7c89371d5c70f677e01550cb446334d584cf88180e548`

```xml
<w src="14" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w>?<verse eID="John.3.10"/>
<verse osisID="John.3.11" sID="John.3.11"/><w src="1" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Verily</w>
```

### K5. John 3:21 to 3:22, a red span closing at verse end, then a pilcrow milestone and an empty `<w/>`

`kjv.osis.xml` offset 19949914, length 278, SHA-256 `27526f631405de332fd6be4319b0e3b20dbf29488a1a42d1f58eadfafe2d6e79`

```xml
<w src="17" lemma="strong:G2316 lemma.TR:θεω" morph="robinson:N-DSM">God</w>.</q><verse eID="John.3.21"/>
<verse osisID="John.3.22" sID="John.3.22"/><milestone type="x-p" subType="x-added" marker="¶"/><w src="11" lemma="strong:G3588 lemma.TR:την" morph="robinson:T-ASF"/>
```

### K6. Matt 8:9, end: a space before the verse's end milestone (one of 91 verses)

`kjv.osis.xml` offset 15507376, length 159, SHA-256 `073ad0078ebf8ecff61112b822950f9d1ec488848c02aaf529a6ee07ec3c55ad`

```xml
<w src="30" lemma="strong:G4160 lemma.TR:ποιει" morph="robinson:V-PAI-3S">he doeth</w> <transChange type="added">it</transChange>. <verse eID="Matt.8.9"/>
```

### K7. 2 Cor 8:10, the one malformed `lemma` token (`G5129` without `strong:`)

`kjv.osis.xml` offset 24477637, length 121, SHA-256 `cace5aee8ec9948f9a97ecbf76fa2ab589a6bd33a82fda52dbe75d4a0f09ab69`

```xml
<w src="3 4" lemma="strong:G1722 G5129 lemma.TR:εν lemma.TR:τουτω" morph="robinson:PREP robinson:D-DSN">herein</w>
```

### K8. Ps 23:1, start: `divineName` (stored as `Lord`, printed as small capitals)

`kjv.osis.xml` offset 9464461, length 101, SHA-256 `68c77d37c6509770df0fa7f49a280aa56217087d2f3e59e8c98ffbd053b4a2e3`

```xml
<verse osisID="Ps.23.1" sID="Ps.23.1"/><w lemma="strong:H03068">The <divineName>Lord</divineName></w>
```

### K9. Dan 5:25, `inscription` (text already in capitals)

`kjv.osis.xml` offset 14239428, length 278, SHA-256 `38b876527e5783ef66d6d33c306b61626fa5887f3a177240f5f5663d9b964cb9`

```xml
<inscription><w morph="strongMorph:TH8752" lemma="strong:H04484">MENE</w>, <w morph="strongMorph:TH8752" lemma="strong:H04484">MENE</w>, <w morph="strongMorph:TH8752" lemma="strong:H08625">TEKEL</w>, <w morph="strongMorph:TH8751" lemma="strong:H06537">UPHARSIN</w></inscription>
```

### K10. Exod 26:4, a study note inside a verse (excluded from verse text in M1)

`kjv.osis.xml` offset 1431332, length 151, SHA-256 `302e5f3af8d38a03d739f966c50f7ed0bbb258e0105ee148656bfc680bdae9f1`

```xml
<note type="study"><catchWord>selvedge</catchWord>: <transChange type="amplified">an edge of cloth so woven that it cannot unravel</transChange></note>
```

### K11. Psalm 3, chapter start: a generated chapter title and the canonical psalm title, both outside any verse

`kjv.osis.xml` offset 9313782, length 383, SHA-256 `f9f09478d785fc6fed4082fd7b21fe743645dd8f1859f8774455d2e79d11654b`

```xml
<chapter osisID="Ps.3" chapterTitle="PSALM 3.">
<title type="chapter">PSALM 3.</title>
<title type="psalm" canonical="true"><w lemma="strong:H04210">A Psalm</w> <w lemma="strong:H01732">of David</w>, <w morph="strongMorph:TH8800" lemma="strong:H01272">when he fled</w> <w lemma="strong:H06440">from</w> <w lemma="strong:H053">Absalom</w> <w lemma="strong:H01121">his son</w>.</title>
```

### K12. Psalm 119, first acrostic title (outside verses)

`kjv.osis.xml` offset 10277211, length 91, SHA-256 `fe087887167495edad5e68dc09419f989fb92cb7f1ee9db14ffeca6658d785fa`

```xml
<title type="acrostic" canonical="true"><foreign xml:lang="hbo">א ALEPH.</foreign></title>
```

### K13. Romans colophon, start (outside verses)

`kjv.osis.xml` offset 23603932, length 209, SHA-256 `a54d0a6bb4d3cfe1ce6baefbc480b328ff2481269641fb87f8019d5e46d57621`

```xml
<div type="colophon" osisID="Rom.c"><w src="15" lemma="strong:G1125 lemma.TR:εγραφη" morph="robinson:V-2API-3S">Written</w> <w src="13" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w>
```

### H1. HebrewStrong.xml, entry H1 (CRLF and tab indentation are in the file)

`HebrewStrong.xml` offset 264, length 334, SHA-256 `73eb81b081cacba9bfff098215d4165778694daaba3f89832c3653c93b245f78`

```xml
<entry id="H1">
		<w pos="n-m" pron="awb" xlit="ʼâb" xml:lang="heb">אָב</w>
		<source>a primitive word;</source>
		<meaning><def>father</def>, in a literal and immediate, or figurative and remote application</meaning>
		<usage>chief, (fore-) father(-less), × patrimony, principal. Compare names in 'Abi-'.</usage>
	</entry>
```

### H2. HebrewStrong.xml, entry H5223 (the one empty `<source/>`)

`HebrewStrong.xml` offset 1657902, length 244, SHA-256 `775af1cd0cdc286527891c492318130f9210daab926de89dc63fd448e02e7c16`

```xml
<entry id="H5223">
		<w pos="a" pron="naw-keh'" xlit="nâkeh" xml:lang="heb">נָכֶה</w>
		<source/>
		<meaning><def>smitten</def>, i.e. (literally) maimed, or (figuratively) dejected</meaning>
		<usage>contrite, lame.</usage>
	</entry>
```

### G1. strongsgreek.xml, the internal DTD subset (skipped, never interpreted)

`strongsgreek.xml` offset 56, length 1034, SHA-256 `9b89a2867fda6f8bfcda951f9bad1ccfc5b06416eb6d38f84f8177078e9441f6`

```xml
<!DOCTYPE strongsdictionary [
   <!ELEMENT strongsdictionary (prologue, entries) >
   <!ELEMENT prologue (#PCDATA) >
   <!ELEMENT entries (entry)+ >
   <!ELEMENT entry (#PCDATA|strongs|greek|pronunciation|latin|strongsref|strongs_def|strongs_derivation|kjv_def)* >
   <!ATTLIST entry strongs CDATA #REQUIRED >
   <!ELEMENT greek EMPTY >


   <!-- unicode is real Greek. translit is SBL-style transliteration. -->
   <!ATTLIST greek BETA CDATA #REQUIRED 
                   unicode CDATA #REQUIRED 
                   translit CDATA #REQUIRED >
   <!ELEMENT latin (#PCDATA) >
   <!ELEMENT kjv_def (#PCDATA|strongsref)* >
   <!ELEMENT strongs_def (#PCDATA|greek|latin|strongsref|pronunciation)* >
   <!ELEMENT strongs_derivation (#PCDATA|greek|latin|strongsref|pronunciation)* >
   <!ELEMENT pronunciation EMPTY >
   <!ATTLIST pronunciation strongs CDATA #REQUIRED >
   <!ELEMENT strongs (#PCDATA) >
   <!ELEMENT strongsref EMPTY >
   <!ATTLIST strongsref language (GREEK|HEBREW) #REQUIRED
                 strongs CDATA #REQUIRED >
 ]>
```

### G2. strongsgreek.xml, entry 00001 (text directly inside `<entry>`, hard line feeds, a leading space)

`strongsgreek.xml` offset 4484, length 708, SHA-256 `f2987e0cdf76b233f086892e2e081c29410706efa058dd251513ee8cf829143f`

```xml
<entry strongs="00001">

 <strongs>1</strongs>   <greek BETA="*A" unicode="Α" translit="A"/>   <pronunciation strongs="al'-fah"/>

 <strongs_derivation>of Hebrew origin;</strongs_derivation><strongs_def> the first letter of the alphabet; figuratively, only
 (from its use as a numeral) the first: </strongs_def><kjv_def>--Alpha.</kjv_def> Often used (usually
 <greek BETA="A)/N" unicode="ἄν" translit="án"/>, before a vowel) also in composition (as a contraction from <strongsref language="GREEK" strongs="427"/>) in
 the sense of privation; so, in many words, beginning with this letter;
 occasionally in the sense of union (as a contraction of <strongsref language="GREEK" strongs="260"/>).


</entry>
```

### G3. strongsgreek.xml, entry 02717 (one of 101 "Not Used" placeholders)

`strongsgreek.xml` offset 1157765, length 67, SHA-256 `d12d7772b47db0788090f70eb7783ba55b67d686749bab55284f5b508d3d6b88`

```xml
<entry strongs="02717">
 <strongs>2717</strongs>  Not Used
</entry>
```
