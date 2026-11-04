# MHCC OSIS survey (CMT-MHCC-CW20)

- **Date:** 2026-10-22
- **File:** `sources/raw/CMT-MHCC-CW20/mhcc.osis.xml`, 5,318,822 bytes, SHA-256 `b00780749c2d6130f41441db2b5cb16df490231e6a315a7d2d9eabcfb36a9fd7`, git blob SHA-1 `36a7a31b78da0cdc217182b31a466da662f4cf92` (row CMT-MHCC-CW20 in `sources/SOURCES.md`).
- **Method:** `docs/pipeline/survey-scripts/survey_mhcc.py`, run with `py -I -B` (Python 3.8.1) from the scratchpad, paths as arguments, never from inside `sources/raw`. It reads each file once and checks it against its checksum file (`osis_tokens.load_pinned`) before anything else, runs the whole MHCC file through the strict reader `mhcc_osis.py` (regex tokenizer plus allowlist walker, no XML parser, no entity mechanism; it refuses anything not listed below) and the KJV file through the existing strict KJV walker, and only then prints facts. The commands:

  ```
  py -I -B docs/pipeline/survey-scripts/survey_mhcc.py <mhcc.osis.xml> sources/checksums/CMT-MHCC-CW20.sha256 <kjv.osis.xml> sources/checksums/BIB-KJV-CW31.sha256
  py -I -B docs/pipeline/survey-scripts/oracle_mhcc.py <mhcc.osis.xml> sources/checksums/CMT-MHCC-CW20.sha256 pipeline/tests/expected
  ```

Every number below is pinned in the Rust profile (`pipeline/src/commentary.rs`, `mhcc()`) or in the oracle files, so the build fails if one changes. Nothing here was corrected; the anomalies in section 6 are stored as the source has them.

**Agreement with the earlier review.** Every fact it recorded (2026-10-22, in memory) was reproduced from the fetched file: size, SHA-256, blob SHA-1; UTF-8 with no BOM, LF only, pure ASCII, `&amp;` the only entity (8×), no DOCTYPE; 66 books, 1,189 chapters; 4,297 matched `verse` sID/eID pairs, each `osisID` listing every covered verse, no range across a chapter, 345 anchors in Psalms; 250 verse IDs covered by more than one anchor; 1,204 `reference` elements; title types `x-s2`, `x-IS`, `x-ms`, `main`; chapter outlines as `table/row/cell`; anchors crossing `p`/`div` boundaries; the Prov 10:1, Gen 21 and Ps 23 examples. Three things the review did not mention, none of which contradicts it: the 2,053 XML comments (section 1), the 9 reused `sID` values (6.5) and the 4 descending reference ranges (6.7).

## 1. Bytes

| Fact | Value |
|---|---|
| Declaration | exactly `<?xml version="1.0" encoding="utf-8"?>` (38 bytes, lowercase `utf-8`), then LF |
| Bytes | only LF (55,467) and printable ASCII 0x20–0x7E (81 distinct byte values): no CR, TAB, other control, or byte above 0x7E |
| `&` | 8, every one `&amp;`, all in text (the archaic "&c."); none in an attribute value |
| `<!` | 2,053, every one an XML comment and one of exactly four strings: `<!-- s2 -->` 1,187, `<!-- is -->` 800, `<!-- ms -->` 61, `<!-- mt -->` 5. Each sits directly before the `<title>` of type `x-s2`, `x-IS`, `x-ms`, `main` respectively, and every title of those types has its comment directly before it. They are the converter's USFM marker names (no content) |
| `<?` | 1 (the declaration); `]]>` 0; `>` in text 0 |
| Tokens / depth | 143,211 tokens (tags, text runs and comments); maximum element depth 7 |
| Longest text run / attribute value | 4,177 bytes / 719 bytes (an 80-verse `osisID`); at most 3 attributes on a tag |
| Tag spelling | inside books every tag is in one canonical form: `<name a="v" b="w">`, `<name a="v" />` (one space before `/>`), `</name>`; attribute order is fixed per element (section 2). Only the root `<osis>` start tag (attributes on three lines) and the indented header differ |

## 2. Elements and attributes

Event forms are the M1-F3 classification (DESIGN.md 9.4). Context `div:book` is `<div type="book">`, `div:intro` is `<div type="introduction">`.

| Element | Form | Count | Parent | Attributes (exact order) |
|---|---|---|---|---|
| `osis` | container | 1 | root | `xmlns`, `xmlns:xsi`, `xsi:schemaLocation` (pinned values) |
| `osisText` | container | 1 | `osis` | `osisIDWork="MHCC"`, `osisRefWork="Bible"`, `xml:lang="en"` |
| `header` | container | 1 | `osisText`, before the books | none |
| `revisionDesc`, `date`, `p` | container | 1 each | header | `revisionDesc@resp` |
| `work`, `title`, `description` ×66, `type`, `identifier`, `refSystem` | container | as listed | header | `work@osisWork`; `description@type`, `@subType`; `type@type`; `identifier@type` |
| `div` | container | 3,526 | `osisText` (66 books) or `div:book` (3,460 introductions) | book: `type="book"`, `osisID`, `canonical="true"`; introduction: `type="introduction"` |
| `milestone` | empty only | 198 | `div:book` | `type` (`x-usfm-toc1`/`2`/`3`, 66 each), `n` |
| `chapter` | empty only | 2,378 | `div:book` (97) or `p` (2,281) | `sID`, `osisID`, `n` or `eID` |
| `title` | container | 2,053 (+1 in the header) | `div:book` or `div:intro` | `type`: `x-s2` 1,187, `x-IS` 800, `x-ms` 61, `main` 5 |
| `p` | container | 11,671 (+1 in the header) | `div:book` or `div:intro` | none (7,014) or `subType="x-introduction"` (4,657) |
| `table` | container | 2,531 | `div:book` or `div:intro` | none |
| `row` | container | 2,531 | `table` | none; always exactly 2 `cell`s |
| `cell` | container | 5,062 | `row` | none |
| `hi` | container | 3,930 | `p` | `type="bold"` |
| `reference` | container | 1,204 | `p` | `osisRef` |
| `verse` | empty only | 8,594 | `p` | `sID`, `osisID`, `n` (4,297) or `eID` (4,297) |

`title`, `cell`, `hi` and `reference` contain text only. Text in `osis`, `osisText`, the header containers, `div`, `table` and `row` is spaces and LFs only. No `note`, `q`, `lg`, `l`, `list`, `item`, `w`, `seg`, `foreign`, `divineName`, `transChange`, `lb`, `figure` or `catchWord` occurs.

## 3. Books and chapters

- 66 `div:book`, in canonical order with the KJV's OSIS book names (`Gen` … `Rev`, `Ps`, `1John`).
- 1,189 chapters, each `<chapter sID="Book.C" osisID="Book.C" n="C" />` … `<chapter eID="Book.C" />`, numbered 1..n per book with no gap; 97 of the 2,378 milestones are direct children of a book `div`, the other 2,281 sit inside a `p` (the chapter boundary falls inside the last paragraph of the previous chapter, after its last verse anchor closed).
- 10 blocks (the `title type="main"` and an empty `p` in Obad, Phlm, 2John, 3John and Jude) come before their book's first chapter milestone.

## 4. Blocks

The paragraph-level elements `p`, `title` and `table` never nest; every one of them inside a book is a **block** (the unit stored and anchored, DESIGN.md 17). The `div type="introduction"` wrappers are a USFM-conversion artifact (review 3.4) and hold blocks of every kind.

| Fact | Value |
|---|---|
| Blocks | 16,255: `p` 11,671 (`subType="x-introduction"` 4,657), `table` 2,531, `title` 2,053 |
| Inside a verse anchor / outside | 8,078 / 8,177 |
| Whitespace-only blocks | 3,337 (`<p>` LF `</p>` and the like; stored like any other block) |
| Non-empty blocks outside every anchor | 257 `p subType="x-introduction"` (book introductions; the chapter comments of Lev 19 and Ps 108, 6.6), all 2,531 outline `table`s, all 2,053 `title`s |
| Largest block | 8,271 bytes; all blocks together 5,145,403 bytes, of which 3,949,070 are text |
| `reference` outside anchors | 11 (in book introductions) |

## 5. Anchors: how comments attach to KJV verses

- 4,297 anchors: `<verse sID="Gen.1.1-2" osisID="Gen.1.1 Gen.1.2" n="1-2" />` … `<verse eID="Gen.1.1-2" />`. 850 cover one verse (`sID="Ps.23.1"`, `n="1"`), the others a range; the largest lists 80 verses.
- **Each `osisID` lists every covered verse explicitly**, single-space separated, and equals the range in `sID` exactly (`Book.C.V` or `Book.C.V1-V2` with V1 < V2, book and chapter the open ones), and `n` is `V` or `V1-V2`. Checked for all 4,297; the pipeline stores the `osisID` list as written and computes nothing from `sID` or `n`.
- 28,968 verse references in all, 28,718 distinct verses. **All 28,718 are KJV verses** (each equals a `verse sID` of the pinned `kjv.osis.xml`); 2,384 KJV verses are in no anchor (the comments cover ranges; Lev 19 and Ps 108 have no anchor at all).
- Anchors never nest. Every sID is the first thing in its block (only spaces/LF before it) and every eID is followed in its block only by spaces/LF and chapter milestones, so **every anchor is a run of whole blocks**: 784 anchors are one block, 3,471 two (the `Verses 1-2` heading `p`, then the comment `p`), 1 three, 4 five, 13 seven, 15 nine, 7 eleven, 2 thirteen (outline-style sequences, 6.2). No block belongs to two anchors. 3,513 anchors (all but those 784) cross `p`/`div` boundaries (anomaly 4).
- Every anchor's first block contains its `hi type="bold"` heading: 3,446 `Verses N-N`, 484 `Verse N` (3,930 `hi` in all); the other 367 anchors have no heading.

## 6. Anomalies (stored as the source has them, counts pinned)

1. **Verses in more than one anchor (anomaly 1): 250.** No verse is in more than two. 248 of them are the pattern below; the other two are `1Sam.25.39` and `Ps.95.7`, each listed by two consecutive ranges.
2. **A single verse-1 anchor directly followed by another anchor that also lists verse 1: 248 chapters** (anomalies 1 and 2). The first anchor holds a chapter preface or a chapter outline written as plain paragraphs (Gen 21: `Chapter Outline` and the outline lines sit inside `<verse sID="Gen.21.1">`), the second the "Verses 1-8" comment. Both are kept; the panel shows both at verse 1.
3. **Chapters whose only anchor is a single verse-1 anchor: 118** (32 of them psalms; anomaly 3, e.g. Ps 23). The comment covers the whole chapter but the file anchors it to verse 1 only; it is shown at verse 1 only.
4. **Anchors crossing element boundaries: 3,513** (5 above).
5. **Reused sIDs: 9** sID values occur twice, each time as a separate, correctly paired anchor: `Prov.10.1`, `Song.1.1`, `Song.5.1`, `Song.6.1`, `Isa.4.1`, `Isa.56.1`, `Isa.57.1`, `Isa.58.1`, `Jer.4.1` (OSIS asks for unique milestone IDs; pairing here is unambiguous because anchors never nest). Anchors are therefore identified by document order, not by sID.
6. **Chapters without any anchor: 2**, Lev 19 and Ps 108: their comment paragraphs sit outside every verse milestone (Lev 19's first paragraph is the fragment `laws.`). They are stored as chapter-level blocks.
7. **Descending reference ranges: 4**, `Jer.50.1-Jer.3.8`, `Jer.50.4-Jer.7.17`, `Ezek.3.12-Ezek.3.2`, `Exod.13.2-Exod.10.13` (probably "50:1-3, 8" style references flattened by the converter). Stored as written; both ends are KJV verses.

## 7. Cross-references

1,204 `<reference osisRef="…">` with display text such as `Joh 6:48` or just `51`: 691 single verses (`Book.C.V`), 513 ranges (`Book.C.V-Book.C.V`, same book on both ends; 3 cross a chapter, 4 descend, 6.7). Every end of every reference is a KJV verse (checked against `kjv.osis.xml`). The pipeline stores the attribute as written plus the two KJV verse IDs of its ends; it does not reorder or expand anything.

## 8. Header (not imported)

`revisionDesc resp="cyrille"` with `date` `2021.02.14T10.19.58` and `p` "Converted from USFM source using u2o.py"; `work osisWork="MHCC"` with `title` `MHCC`, 66 `description type="usfm" subType="x-id"` (USFM book codes), `type type="OSIS"` `Bible`, `identifier type="OSIS"` `Bible.en.MHCC`, `refSystem` `Bible`. Validated against the allowlist, not imported (credits come from SOURCES.md). The `x-usfm-toc1/2/3` milestones (book names from the converter; `toc2`/`toc3` are always empty) are validated and not imported either.
