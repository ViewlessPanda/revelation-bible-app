# SOURCES.md: pinned content sources for Revelation

This file lists the outside files that go into Revelation: the Bible text,
lexicons, commentary, versification table, fonts, icons and the project logo.
For each one it records which file it is, where it came from, its licence (or
public-domain basis), its SHA-256 and what credit it requires.

How it is used:

- `.github/scripts/fetch-sources.sh` downloads each pinned file from its
  `Exact file URL` and checks it against `sources/checksums/<ID>.sha256`.
- The pipeline refuses to read a source whose SHA-256 is not in its row here.
- `app/tests/about.test.ts` checks that the in-app credits equal the
  `Required attribution` text below.

A source's SHA-256 is its pin. A changed file needs its row updated.

---

## Index

| ID | Type | Name |
|---|---|---|
| BIB-KJV-CW31 | Bible (Strong's + red letters) | CrossWire KJV 3.1, `kjv.osis.xml` |
| LEX-HEB-OSHB | Lexicon (Hebrew) | Open Scriptures `HebrewStrong.xml` |
| LEX-GRC-MGNT19 | Lexicon (Greek) | MorphGNT `strongsgreek.xml` v1.9 |
| VRS-ENG-PALASO | Versification | SIL libpalaso / Paratext "English" `eng.vrs.txt` |
| CMT-MHCC-CW20 | Commentary | CrossWire MHCC 2.0, `mhcc.osis.xml` (Matthew Henry's Concise) |
| FONT-LITERATA | Font (reading, default) | Literata 3.103 |
| FONT-INTER | Font (UI) | Inter 4.1 |
| ICON-LUCIDE | Image (icon set) | Lucide 1.59.0 |
| BRAND-LOGO | Image (project logo and app icon) | Owner-supplied logo (made with ChatGPT) and derived icon, `assets/brand/` |

Copyrighted translations (ESV, NIV, NASB and similar) are never bundled.

---

## Bibles

### BIB-KJV-CW31: CrossWire KJV 3.1 (OSIS source)
- **Type:** Bible, with Strong's tagging and words-of-Jesus markup in the same file
- **Name and edition:** King James Version (1769) with Strong's Numbers and Morphology; CrossWire Bible Society module **KJV, version 3.1** (released 2023-07-19 per `kjv.conf`). Intended as an e-text of Benjamin Blayney's 1769 Oxford edition (CrossWire KJV wiki). Protocanon only (Gen to Rev).
- **Publisher / maintainer:** CrossWire Bible Society, https://gitlab.com/crosswire-bible-society/kjv (contact: modules@crosswire.org)
- **Exact file URL:** https://gitlab.com/crosswire-bible-society/kjv/-/raw/d490be7e34762deb2c76cb2c1306d4808e27890d/kjv.osis.xml
- **Mutable URL?** No. The commit hash is in the path.
- **Format:** OSIS 2.1.1 XML, UTF-8. Verses use milestones (`<verse sID/>…<verse eID/>`). No DOCTYPE.
- **Version / date:** module 3.1 (2023-07-19); file last changed 2023-07-10
- **Size (bytes):** 28,043,075
- **License or public-domain basis:**
  - Underlying KJV text: public domain outside the UK. In the UK the Authorized Version is under Crown letters patent.
  - CrossWire's contributions: `kjv.conf` says *"Any copyright that might be obtained for this effort is held by CrossWire Bible Society © 2003-2023 and CrossWire Bible Society hereby grants a general public license to use this text for any purpose."* and sets `DistributionLicense=GPL` (https://crosswire.org/ftpmirror/pub/sword/raw/mods.d/kjv.conf). CrossWire's conf-file spec links the value "GPL" to the GNU GPL (https://wiki.crosswire.org/DevTools:conf_Files). No GPL version is named.
  - Revelation distributes the KJV data (the derived `kjv.sqlite` included) under the **GNU General Public License, version 3**; the text is in `licenses/GPL-3.0.txt` and the notice is in `licenses/KJV-NOTICE.md`. The app's own code stays under the MIT License.
- **Required attribution:** (About / Sources screen, and the installer's third-party notices)
  > King James Version (1769) text with Strong's numbers and words-of-Christ markup from the CrossWire Bible Society KJV module, version 3.1 (https://gitlab.com/crosswire-bible-society/kjv). Any copyright that might be obtained for this effort is held by CrossWire Bible Society © 2003-2023, which grants a general public license to use this text for any purpose (distributed under the GNU General Public License). OT Strong's numbers from The Bible Foundation; NT Strong's data from CrossWire's KJV2003 Project, with Greek data from Dr. Maurice A. Robinson; formatting markup by Michael Paul Johnson (eBible.org). Red-letter markup follows Louis Klopsch's 1901 red-letter edition, per CrossWire. Converted into Revelation's database format by the Revelation pipeline; the text and markup boundaries are unchanged.
  >
  > Rights in the Authorized (King James) Version in the United Kingdom are vested in the Crown.
- **Share-alike or other obligations:** GNU GPL, version 3: pass the licence on, keep the notices, mark the conversion as a modification with a date, and make the corresponding source (this pinned OSIS file plus the pipeline that converts it) available with each release that contains the KJV.
- **SHA-256:** `4bcbfe45722fca396adbeac9c81025bbb173cb1e30296da90864f0400cb0e3f5` (28,043,075 bytes). Checksum file: `sources/checksums/BIB-KJV-CW31.sha256`.

---

## Lexicons

### LEX-HEB-OSHB: Strong's Hebrew Dictionary, Open Scriptures Hebrew Bible Project XML
- **Type:** Lexicon (Hebrew/Aramaic), Strong's dictionary entries only
- **Name and edition:** James Strong, *A Concise Dictionary of the Words in the Hebrew Bible* (from *The Exhaustive Concordance of the Bible*, 1890), in the OSHB "HebrewStrong.xml" edition, with OSHB corrections and duplicate entries recombined (readme)
- **Publisher / maintainer:** Open Scriptures Hebrew Bible Project, https://github.com/openscriptures/HebrewLexicon
- **Exact file URL:** https://raw.githubusercontent.com/openscriptures/HebrewLexicon/21c9add13bc727d3a951361778e97e3ff7afd1ce/HebrewStrong.xml
- **Mutable URL?** No (commit hash in path)
- **Format:** XML (namespace `http://openscriptures.github.com/morphhb/namespace`, schema `StrongSchema.xsd`); one `<entry id="H1">` per number; 8,674 entries
- **Size (bytes):** 2,749,042
- **License or public-domain basis:** the repo readme says "These files are released under the Creative Commons Attribution 4.0 International license. The actual text of Brown, Driver, Briggs and Strong's Hebrew dictionary remain in the public domain. For attribution purposes, credit the Open Scriptures Hebrew Bible Project." (https://github.com/openscriptures/HebrewLexicon/blob/21c9add13bc727d3a951361778e97e3ff7afd1ce/readme.md). CC BY 4.0 legal code: https://creativecommons.org/licenses/by/4.0/legalcode
- **Required attribution:** (shown in-app on the Lexicon tab's source line or the About / Sources screen, and in the installer's third-party notices; CC BY 4.0 §3(a))
  > Strong's Hebrew Dictionary (James Strong, 1890; public domain), XML edition by the Open Scriptures Hebrew Bible Project (https://github.com/openscriptures/HebrewLexicon), licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). Provided "as is" without warranties. Converted to Revelation's database format; entry text unchanged.
- **Share-alike or other obligations:** none (CC BY has no share-alike). Keep the licence notice and the warranty-disclaimer reference; say if the text is modified.
- **SHA-256:** `a628f4f89f8bdaf2483fd3faf1abc8653cc6717758dfc9f24beb7571d9bdd0c4` (2,749,042 bytes). Checksum file: `sources/checksums/LEX-HEB-OSHB.sha256`.

### LEX-GRC-MGNT19: Strong's Greek Dictionary in XML with real Greek, v1.9
- **Type:** Lexicon (Greek), Strong's dictionary entries
- **Name and edition:** James Strong, *Dictionary of the Greek Words* (from *The Exhaustive Concordance*, 1890), XML edition by Dr. Ulrik Sandborg-Petersen, **version 1.9 (2019-02-05)**
- **Publisher / maintainer:** MorphGNT, https://github.com/morphgnt/strongs-dictionary-xml
- **Exact file URL:** https://raw.githubusercontent.com/morphgnt/strongs-dictionary-xml/dd6758b82f46b620804a8ea677d715844679eea1/strongsgreek.xml
- **Mutable URL?** No (commit `dd6758b8…` "Release version 1.9.")
- **Format:** XML with an internal DTD subset (`<!DOCTYPE strongsdictionary [ … ]>`); `<entry strongs="00001">` with `strongs_derivation`, `strongs_def`, `kjv_def`, `greek` (BETA, Unicode, translit), `pronunciation`, `strongsref`
- **Size (bytes):** 2,369,537
- **License or public-domain basis:** README: "Released under the Creative Commons CC0 waiver … Ulrik Sandborg-Petersen has waived all copyright and related or neighboring rights to *Strong's Dictionary in XML with real Greek*." (https://github.com/morphgnt/strongs-dictionary-xml/blob/dd6758b82f46b620804a8ea677d715844679eea1/README.md); release note for 1.7: "Release explicitly under the CC0 copyright waiver (Public Domain)". The file's prologue: "Public Domain -- Copy Freely". The underlying 1890 text is public domain.
- **Required attribution:** none required (CC0). Courtesy credit (About / Sources screen):
  > Strong's Greek Dictionary (James Strong, 1890; public domain), XML edition "Strong's Greek Dictionary in XML with real Greek" v1.9 by Ulrik Sandborg-Petersen (CC0), from an e-text by Michael Grier distributed by The Bible Foundation.
- **Share-alike or other obligations:** none
- **SHA-256:** `91474213b208a4e487d35f85c10e5c90737acc69082dad15dec5431406003545` (2,369,537 bytes). Checksum file: `sources/checksums/LEX-GRC-MGNT19.sha256`.

---

## Versification

### VRS-ENG-PALASO: Paratext "English" versification, SIL libpalaso copy
- **Type:** versification (table of chapters per book and maximum verse per chapter)
- **Name and edition:** `eng.vrs` "Versification "English"", `# Version=2.0` (header). This is Paratext's standard "English" versification, "the versification used by most English (e.g. RSV) and Spanish Bibles". It is used as the expected verse counts for the KJV check, with the two KJV overrides in `pipeline/data/kjv-versification-overrides.txt`.
- **Publisher / maintainer:** SIL Global, `sillsdev/libpalaso`, `SIL.Scripture/Resources/eng.vrs.txt`
- **Exact file URL:** https://raw.githubusercontent.com/sillsdev/libpalaso/f1f70e93a79d55b1769a4e20fb36047f2fa08d99/SIL.Scripture/Resources/eng.vrs.txt
- **Mutable URL?** No (commit hash in path)
- **Format:** plain text, ASCII, LF line endings. One line per book (`GEN 1:31 2:25 …`, maximum verse per chapter), then a verse-mapping section (`=` lines) that is not used.
- **Version / date:** Version=2.0
- **Size (bytes):** 18,787
- **License or public-domain basis:** MIT. https://github.com/sillsdev/libpalaso/blob/master/LICENSE ("Copyright (c) 2007-2026 SIL Global")
- **Required attribution:** the file is not vendored or shipped. Courtesy line: "Expected verse counts: Paratext 'English' versification (eng.vrs, SIL libpalaso, MIT), with the KJV overrides listed in SOURCES.md VRS-ENG-PALASO."
- **Share-alike or other obligations:** none
- **SHA-256:** `003981c7f43c69b73b60d40a3f35f72e7ee017a686a6fb206f19a1b721157541` (18,787 bytes). Checksum file: `sources/checksums/VRS-ENG-PALASO.sha256`.

---

## Commentaries

### CMT-MHCC-CW20: Matthew Henry's Concise Commentary, CrossWire MHCC 2.0 (OSIS source)
- **Type:** commentary
- **Name and edition:** "Matthew Henry's Concise Commentary on the Whole Bible", CrossWire Bible Society SWORD module **MHCC, version 2.0** (2021-02-14). A one-volume abridgement of Henry's *Exposition*. Whole Bible, Gen to Rev.
- **Publisher / maintainer:** CrossWire Bible Society, https://gitlab.com/crosswire-bible-society/mhcc
- **Exact file URL:** https://gitlab.com/crosswire-bible-society/mhcc/-/raw/b15ff40aab9556005fd3d38d5388040a90b99145/mhcc.osis.xml
- **Mutable URL?** No. The commit hash is in the path.
- **Format:** OSIS 2.1.1 XML, UTF-8 (no BOM, LF, pure ASCII; only entity `&amp;`), no DOCTYPE.
- **Version / date:** module 2.0, file committed 2021-02-14
- **Size (bytes):** 5,318,822
- **License or public-domain basis:** public domain. Matthew Henry died in 1714 and the abridgement was published by the Religious Tract Society in 1838–1840. The digitiser's statement (pinned `mhcc.conf`): `DistributionLicense=Public Domain`; About: "Public Domain--Copy Freely. This text matches the printed edition as published by Moody Press, 28th printing, no Copyright displayed. ISBN: 0-8024-5190-X." The conf's `TextSource` is the Christian Classics Ethereal Library's plain text (https://ccel.org/ccel/h/henry/mhcc/cache/mhcc.txt), whose header says "Rights: Public Domain".
- **Required attribution:** none is required (public domain). For provenance and accuracy, the About / Sources screen says:
  > Matthew Henry's Concise Commentary on the Whole Bible (public domain), from the CrossWire Bible Society MHCC module, version 2.0 (https://gitlab.com/crosswire-bible-society/mhcc, commit b15ff40), which was prepared from the Christian Classics Ethereal Library's text. Converted into Revelation's database format by the Revelation pipeline; text unchanged.
- **Share-alike or other obligations:** none
- **SHA-256:** `b00780749c2d6130f41441db2b5cb16df490231e6a315a7d2d9eabcfb36a9fd7` (5,318,822 bytes). Checksum file: `sources/checksums/CMT-MHCC-CW20.sha256`.

---

## Fonts and images

### Standing font conditions F1–F6 (apply to every FONT-* row)
- **F1. Unmodified upstream files only.** Use the binaries the foundry or
  project built: TTF/OTF, or a WOFF2 the foundry itself published. No
  subsetting, variable-font instancing, format conversion, re-hinting,
  merging or name-table edits. For a font with Reserved Font Names, a
  modified file may not show any RFN word in the name readers see (OFL 1.1
  condition 3).
- **F2. Licence next to the fonts.** Copy the family's licence file verbatim
  to `/assets/fonts/<family>/` and into the installer's third-party notices.
- **F3. In-app credit.** The About / Sources screen lists each family:
  copyright line, "SIL Open Font License 1.1", upstream link, and any
  trademark sentence listed in the row.
- **F4. Check on fetch.** Record SHA-256, name ID 5 (version), name ID 0
  (copyright) and name IDs 13/14 (licence). If the licence differs, or there
  is a Reserved Font Name the row doesn't list, don't use the file.
- **F5. Pin.** Release assets can be replaced, so the SHA-256 is the pin.
- **F6. Ship only the files we use.** Leaving out whole files is fine. Cutting
  glyphs out of a file is not.

Shared field values for all FONT-* rows: **Type:** font. **Share-alike or
other obligations:** OFL 1.1 conditions 1–5: not sold by itself, notice and
licence with each copy, Reserved Font Name restriction, no promotional use of
the authors' names, fonts stay under the OFL. The app code is unaffected.

### FONT-LITERATA: Literata (reading font, default)
- **Name and edition:** Literata 3, variable (`opsz`, `wght`) roman + italic
- **Publisher / maintainer:** The Literata Project Authors (TypeTogether for Google), https://github.com/googlefonts/literata
- **Exact file URL:** https://github.com/googlefonts/literata/releases/download/3.103/3.103.zip
- **Mutable URL?** release asset (replaceable); pinned by SHA-256. Tag `3.103` = commit `0c2761b727a1b3a7cffd313c37f0f5163dfc7a63`
- **Format:** TTF (zip) · **Version / date:** 3.103, 2023-05-19 · **Size (bytes):** 17,546,949 (zip)
- **License:** SIL OFL 1.1. `OFL.txt` at `0c2761b7`: "Copyright 2017 The Literata Project Authors (https://github.com/googlefonts/literata)"
- **Reserved Font Names:** none
- **Required attribution:** F2/F3 with the copyright line above
- **SHA-256:** zip `3.103.zip` f7fb973cafb26cf785cbebaeaf51c18f87c15a3bcf4d82a7d4857564db5b056d; `Literata[opsz,wght].ttf` b41138c9373112f32abb589cc22e8674b06ed4048b0c513be922bdd26f274440; `Literata-Italic[opsz,wght].ttf` d483dfaeba9cbf4ce71d32a52ee65df82f7e35b15fff8d1011cdb242d1fcd465; `OFL.txt` 8742963604cd89dc81437811a850018fc03b2bfad686d7422c8235967c87614e (also in `assets/fonts/fonts.sha256`)

### FONT-INTER: Inter (UI font)
- **Name and edition:** Inter 4, variable (`opsz`, `wght`) roman + italic
- **Publisher / maintainer:** The Inter Project Authors (Rasmus Andersson), https://github.com/rsms/inter
- **Exact file URL:** https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip
- **Mutable URL?** release asset; pinned by SHA-256. Tag `v4.1` = commit `e3a3d4c57d5ecc01453a575621882a384c1995a3`
- **Format:** TTF/WOFF2 (zip) · **Version / date:** 4.1, 2024-11-16 · **Size (bytes):** 33,707,794 (zip)
- **License:** SIL OFL 1.1. `LICENSE.txt` at the tag: "Copyright (c) 2016 The Inter Project Authors (https://github.com/rsms/inter)"
- **Reserved Font Names:** none
- **Required attribution:** F2/F3
- **SHA-256:** zip `Inter-4.1.zip` 9883fdd4a49d4fb66bd8177ba6625ef9a64aa45899767dde3d36aa425756b11e; `InterVariable.ttf` 4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf; `LICENSE.txt` 262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a (also in `assets/fonts/fonts.sha256`)

### ICON-LUCIDE: Lucide icon set (UI icons)
- **Type:** image (icon set, SVG)
- **Name and edition:** Lucide 1.59.0
- **Publisher / maintainer:** Lucide Icons and Contributors, https://github.com/lucide-icons/lucide
- **Exact file URL:** https://github.com/lucide-icons/lucide/releases/download/1.59.0/lucide-icons-1.59.0.zip
- **Mutable URL?** release asset; pinned by SHA-256. Tag `1.59.0` = commit `1ef60af944de207e0d5176905a0759ba12e6ee8d`
- **Format:** SVG (zip) · **Version / date:** 1.59.0, 2026-10-17 · **Size (bytes):** 1,427,722 (zip)
- **License:** `LICENSE` at the tag: **ISC**, "Copyright (c) 2026 Lucide Icons and Contributors", **plus MIT** "for the icons listed above" (about 120 icons "derived from the Feather project"), "Copyright (c) 2013-present Cole Bemis"
- **Required attribution:** the whole upstream `LICENSE` file (ISC + MIT) in third-party notices. About screen: "Lucide icons (ISC), including icons from Feather (MIT)"
- **Share-alike or other obligations:** none. Changes (stroke, colour, crop) are allowed. Keep the notices.
- **SHA-256:** none recorded: no Lucide file is vendored; the few icon shapes used are drawn in `app/src/lib/notes-tags.ts`

---

## Project brand artwork

### BRAND-LOGO: Revelation logo and app icon
- **Type:** image (project logo; app icon derived from it)
- **Name and edition:** "Revelation" logo: a four-pointed gold star with short rays above an open book, with the serif wordmark "Revelation" below, on a dark navy background. App icon: the star and book only, centred on a 1024×1024 `#080B12` canvas with soft edges.
- **Publisher / maintainer:** supplied by the project owner, C. Ethan Key, on 2026-10-24. **Made with ChatGPT**; it is an AI-generated image. The only later changes are the scripted ones below.
- **Files:**
  - `assets/brand/revelation-logo-original.webp`: the original, unaltered.
  - `assets/brand/revelation-icon-1024.png`: derived from the original by `assets/brand/make-icon.ps1` (crop to the mark, scale, centre, soft-edge masks; Windows WPF imaging, no third-party tools or content).
  - `assets/brand/make-icon.ps1`: our script (project code, MIT).
- **License or public-domain basis:** AI output under OpenAI's Terms of Use (https://openai.com/policies/terms-of-use/), which give the user ownership of the output. The logo is **not under the project's MIT licence**; see `assets/brand/LICENSE` and the licence section of the root README. The project does not describe the logo as hand-drawn or designer-made.
- **Required attribution:** none
- **SHA-256:**
  - `revelation-logo-original.webp`: `2bdbece3e616643eb4d4cf98343dd39a0432958d86d09c4dbfdf2c9b05936712`
  - `revelation-icon-1024.png`: `40b8b2195b8816d3d514c0580ae97962cf008e5098c77dc0cab4951e230bb4b8`
  - `make-icon.ps1`: `65f2c126cde1693c0433d251626c28622cdc4e65756e4ec4400f1bb883007661`
