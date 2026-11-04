# KJV versification overrides: printed-page check

Owner of this file: biblical scholar. Status: **checked against two scans by the scholar agent; human spot-check pending**. Last updated 2026-10-20.

## 1. What this checks

The versification plan (`sources/SOURCES.md` row
VRS-ENG-PALASO) uses the Paratext "English" table `eng.vrs` with two KJV overrides:

| Override | `eng.vrs` says | KJV override | Where the difference lies |
|---|---|---|---|
| 3 John 1 | 15 verses (`3JN 1:15`) | **14** | The KJV prints "Peace be to thee … Greet the friends by name" as part of verse 14 |
| Revelation 12 | 18 verses (`REV 12:18`) | **17** | The KJV prints "And I stood upon the sand of the sea" as the start of 13:1 |

The `eng.vrs` values above are the ones the source review reports. I didn't
re-read the pinned file, so they are context only. The KJV numbers below come from the printed pages.

## 2. Result

**Both overrides are confirmed** in two independent printed KJV editions, each viewed as a page
image.

| Override | Edition and page | What the page shows | Result |
|---|---|---|---|
| 3 John = 14 verses | **E1**, *Oxford Self-Pronouncing Testament* (c. 1913–15), printed p. 483, archive.org leaf **n484** (3 John 1–4 begin on p. 482, n483) | Verses numbered up to **14**. Verse 14 runs from "But I trust I shall shortly see thee" to "Greet the friends by name." Then the heading for Jude. No verse 15. | **14 confirmed** |
| 3 John = 14 verses | **E0**, Klopsch *Holy Bible: Red Letter Edition* (1901), as reproduced in a 2023 facsimile e-book, printed p. 1252, leaf **n1275** | The whole epistle is on one page. The last verse is numbered **14** and ends "Greet the friends by name." No verse 15. | **14 confirmed** |
| Revelation 12 = 17 verses | **E1**, printed p. 502, leaf **n503** | Verse **17** ends "the testimony of Jesus Christ." Next comes the heading "CHAPTER 13.", then verse 1, "And I stood upon the sand of the sea, and saw a beast rise up out of the sea …". No 12:18. | **17 confirmed** |
| Revelation 12 = 17 verses | **E0**, printed p. 1263, leaf **n1286** | The whole of chapter 12 is on one page. The last verse is numbered **17**. Then "Chapter 13", whose first verse (drop capital) begins "And I stood upon the sand of the sea". | **17 confirmed** |

The quoted words are boundary markers read from the scans (PLAN.md 2.4: never from memory).

**Cross-check against the pinned source (information only, not evidence):** CrossWire KJV 3.1
`kjv.osis.xml` (BIB-KJV-CW31) has 14 distinct verse IDs in `3John.1` and 17 in `Rev.12`, plus 18 in
`Rev.13`. I counted these with a script on 2026-10-20. That agrees with the printed pages, but it is
not independent evidence, because the source is the file under test.

## 3. Citations

- **E1.** *The Oxford Self-Pronouncing Testament … with the words of our Lord printed in red.* Oxford:
  University Press; London: Humphrey Milford. Printer: Horace Hart. Undated (inferred c. 1913–1915;
  to confirm). Copy: University of California Libraries. Scan:
  https://archive.org/details/oxfordselfpronou00oxforich. Pages used:
  `…/page/n483/mode/1up` (p. 482), `…/page/n484/mode/1up` (p. 483), `…/page/n503/mode/1up` (p. 502).
  Viewed 2026-10-20. This is an original copy, scanned in colour by the Internet Archive.
- **E0.** *The Holy Bible: Red Letter Edition.* New York: The Christian Herald; Louis Klopsch,
  Proprietor, Bible House, 1901, **as reproduced in** John DiCesare, *Rubricated American Heritage
  Bible eBook* (Museum and Library of the Bible Press, 2023). Scan:
  https://archive.org/details/rubricated-american-heritage-bible-e-book. Pages used:
  `…/page/n1275/mode/1up` (p. 1252) and `…/page/n1286/mode/1up` (p. 1263). Viewed 2026-10-20.
  **This is a modern facsimile, not an original copy.** See `red-letter-golden-samples.md`
  section 2 (E0) for the full caveat.

## 4. For the developer and the owner

- The override file in the repo can cite: "3 John 1 = 14 verses: E1 p. 483 (archive.org
  oxfordselfpronou00oxforich n484); E0 p. 1252 (n1275). Revelation 12 = 17 verses: E1 p. 502 (n503);
  E0 p. 1263 (n1286)."
- **Human spot-check pending.** A person should open the four page links above and confirm the last
  verse number in each case. That takes about 2 minutes.
- E2 (Oxford/Concordia, 1958–78) was **not** checked for these two chapters. It isn't needed, since
  two editions already agree.
