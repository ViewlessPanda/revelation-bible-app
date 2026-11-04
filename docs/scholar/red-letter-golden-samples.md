# Red-letter golden samples (M1-3)

Owner of this file: biblical scholar. Status: **second pass done; draft; every row
still needs a human spot-check**. First pass 2026-10-19; second pass 2026-10-20; follow-up
(RL-42 in E2, E2 title page, Mt 4:19 in E1) 2026-10-20.

## 1. Purpose

PLAN.md Section 2, rule 8 says red letters are content. The app shows red only where the
translation's own words-of-Jesus markup puts it, with exact boundaries.

This file is an **independent check** on that pipeline. It records what printed red-letter KJVs
show, verse by verse, at the tricky boundaries. The record was read off page scans, not taken from
the digital source and not written from memory. A pipeline that drops, adds, shifts or trims a span
should disagree with it.

Two things this file does **not** do:

- It doesn't decide where the app's red letters go. The app follows the pinned digital source
  (BIB-KJV-CW31). Where that source differs from a printed edition, the difference is a **finding to
  record**, not something to "fix" (PLAN.md 2.4 and 2.8). Nothing here proposes changing the source.
- It doesn't supply Scripture text for tests. The few words quoted below are boundary markers only,
  read from the scans or copied from the machine-extracted excerpts. Word-for-word text tests must
  use the pinned source file.

**What changed in the second pass (2026-10-20):**

- Added **E0**, the 1901 Klopsch edition that the source says its red letters were matched to. It
  was viewed in a **2023 facsimile e-book**, not an original copy (section 2).
- Re-checked **all 99** verse-level rows against E0, plus one new Old Testament row (RL-42,
  Ps 22:1).
- Closed **Mark 8:2**: in E1 and E0 the whole verse is red, and the span runs on into 8:3.
- Compared the **pinned source** (via `docs/pipeline/golden-source-excerpts.md`) with E0 and E1 at
  word level (section 9).
- Rebuilt the machine-checkable block with `source_expected`, `print_E0` and `print_E1` columns
  (section 10).
- Versification overrides: see `kjv-versification-overrides.md`. KJV label: see `kjv-label.md`.

**Follow-up (2026-10-20, later the same day):**

- Closed **RL-42 in E2**: Ps 22:1 is printed **black** in E2, at p.611 (n312), not p.612 as the
  first pass assumed (section 3, E2 method note). The source agrees with E2 there; E0 still differs.
- Closed the **E2 title page and printer** lines by viewing the images (n7, n8). They agree with the
  OCR. E2's date is still open: neither page prints one.
- Re-viewed **Mt 4:19 in E1** magnified (p.10, n11). Same boundary as recorded; it is no longer a
  whole-page-only reading.
- Owner spot-check guide: `golden-spot-check-guide.md` (same folder).

## 2. Editions consulted

### E0 (the edition the source claims to follow): Klopsch, *Holy Bible: Red Letter Edition*, 1901, in a 2023 facsimile e-book

**Facsimile caveat (read first).** E0 was viewed **only** as page images in a 2023 digital
facsimile, the archive.org item that CrossWire's wiki links as a "scanned copy". A source review found it is a
modern e-book. It is **not** an original 1901 copy and
not a library scan. So:

- It shows one privately owned copy, as photographed or scanned and processed in or before 2023.
  I can't tell whether the images were retouched, colour-corrected or rebuilt, or whether that copy
  is the same printing or format as the hard copy CrossWire used in 2016. **To confirm.**
- Its colour reproduction is good enough to tell red from black at about 1:1 magnification
  (section 3). Its red is a dark crimson, which can look black when the image is reduced.
- I viewed only the reproduced 1901 pages (title page, verso, Explanatory Note and Bible text). The
  e-book's 2023 preface and other new material (leaves n0 to about n21, judging by the OCR, plus a
  photograph of the binding at n22) were **not** used or copied.
- An E0 reading tells you what this facsimile shows. It is evidence about Klopsch 1901, not proof
  of it.

| Field | Value | Basis |
|---|---|---|
| Title (reproduced 1901 title page) | *The Holy Bible. Red Letter Edition. Containing the Old and New Testaments, translated out of the original tongues: with former translations diligently compared and revised. Many thousands of parallel references and marginal readings.* | Leaf n24, viewed |
| Red-letter statement on the title page | Old Testament: "Passages and incidents quoted or referred to by our Lord and Saviour printed in red". New Testament: "The words recorded as having been spoken by our Lord and Saviour printed in red". Also: Old Testament passages "generally regarded as prophetic of our Saviour" are marked with a red star. | Leaf n24, viewed |
| Imprint | New York: The Christian Herald. Louis Klopsch, Proprietor, Bible House. 1901. Corner lines: "Self-pronouncing", "Helps and references", "Large type. Octavo", "Concordance and maps". | Leaf n24, viewed |
| Verso | "Copyright, 1901, by Louis Klopsch, New York City". "The Christian Herald Press, M. N. Cormack, Supt." | Leaf n25, viewed |
| Klopsch's own explanation | "Explanatory Note", p. xvi, signed "Louis Klopsch", dated "New York City, November, 1901". Point I: New Testament red marks the words "universally accepted as the utterances of our Lord and Saviour". Point II: Old Testament red marks passages He quoted or that relate directly to incidents He referred to. Points III and IV cover red in the reference column and the red star. | Leaf n39, viewed (paraphrased except the one short quotation) |
| Binding (photograph in the e-book, 2023) | Spine reads "Holy Bible / Red Letter Edition / The Christian Herald / Teachers' Edition". | Leaf n22, viewed. This is a modern photograph, recorded only to identify the copy. |
| Facsimile e-book | John DiCesare, *Rubricated American Heritage Bible eBook*. Museum and Library of the Bible Press, 2023-04-15. Archive.org metadata calls it "a historical background preface and a facsimile reproduction" of Klopsch's "Holy Bible: Red Letter Edition". | https://archive.org/metadata/rubricated-american-heritage-bible-e-book, read 2026-10-20 |
| Item | https://archive.org/details/rubricated-american-heritage-bible-e-book | |
| Page link pattern | `https://archive.org/details/rubricated-american-heritage-bible-e-book/page/n{N}/mode/1up`. Image actually viewed: the archive.org BookReader image service, full resolution (1801 × 2700 px), `https://ia601908.us.archive.org/BookReader/BookReaderImages.php?zip=/25/items/rubricated-american-heritage-bible-e-book/Rubricated%20American%20Heritage%20Bible%20eBook_jp2.zip&file=Rubricated%20American%20Heritage%20Bible%20eBook_jp2/Rubricated%20American%20Heritage%20Bible%20eBook_{NNNN}.jp2&id=rubricated-american-heritage-bible-e-book&scale=1&rotate=0`, where `{NNNN}` is the leaf number padded to 4 digits. The server name can change; the `details` link is the stable citation. | |
| Page/leaf mapping | For the Bible text, **printed page = leaf − 23** (for example n1004 = p. 981, n1277 = p. 1254, n644 = p. 621, n40 = p. 17). I read the folio on the cited pages, at overview resolution, and it was consistent throughout. The front matter uses roman numerals (Explanatory Note = p. xvi = n39). | Folios viewed |

### E1 (independent check): The Oxford Self-Pronouncing Testament, red-letter

| Field | Value | Basis |
|---|---|---|
| Title (from title page) | *The Oxford Self-Pronouncing Testament. The New Testament of Our Lord and Saviour Jesus Christ, translated out of the original Greek: and with the former translations diligently compared and revised by His Majesty's special command. Appointed to be read in Churches.* | Title page image, leaf n2, viewed |
| Red-letter statement (printed in red on the title page) | "With the words of our Lord printed in red" | Title page image, viewed |
| Imprint | Oxford, printed at the University Press. London: Humphrey Milford, Oxford University Press, Amen Corner. New York: 35 West 32nd Street. "Long Primer 16mo (Pron.) Refs." Cum privilegio. | Title page image, viewed |
| Printer (colophon) | "Oxford: Horace Hart, Printer to the University" | Final leaf n520, viewed |
| Date | **Not printed.** Library catalogue gives "[19--?]". Inferred **c. 1913–1915**: Milford was Publisher to the University from 1913 to 1945, and Hart was Printer from 1883 to 1915 (Wikipedia, "Humphrey S. Milford"; Wikipedia, "Horace Hart"). **To confirm** against an OUP bibliography. | Inference |
| Contents | New Testament only (510 pp.) | Archive.org metadata |
| Copy scanned | University of California Libraries; scanned in colour by the Internet Archive (Canon 1Ds). Archive.org rights field: NOT_IN_COPYRIGHT. **An original copy.** | Archive.org metadata |
| Scan | https://archive.org/details/oxfordselfpronou00oxforich | |
| Page link pattern | `https://archive.org/details/oxfordselfpronou00oxforich/page/n{N}/mode/1up` (image: `https://archive.org/download/oxfordselfpronou00oxforich/page/n{N}.jpg`). For the text pages used here, **printed page = N − 1**. I checked this against the running heads on every page cited. | |

E1 stays the **independent** printed check. Agreement with E0 shows only that the source matches the
edition it says it followed (source review 4.2).

### E2 (cross-check, and the Old Testament controls): Oxford "Red Letter Edition" Bible printed for Concordia

| Field | Value | Basis |
|---|---|---|
| Title | *The Holy Bible, Red Letter Edition … Authorized King James Version, with the words of the Lord Jesus Christ as recorded in the New Testament printed in red letters. With Chain References.* Printed for Concordia Publishing House, St. Louis, Mo., at the University Press, Oxford. "Long Primer 8vo Refs." | Title page image, leaf n7 (right-hand page), **viewed 2026-10-20**; it agrees with the OCR. "Red Letter Edition" and the line limiting red to the New Testament are themselves printed red. |
| Printer | "Printed in Great Britain at the University Press, Oxford by Vivian Ridler, Printer to the University", under "Cum privilegio" | Verso of title page, leaf n8 (left-hand page), **viewed 2026-10-20**. Ridler was Printer from 1958 to 1978 (Wikipedia, "Vivian Ridler"). |
| Date | Not established. The title page and its verso (n7, n8, viewed) print **no date**. Printed sometime in 1958–1978 by the printer's dates. **To confirm** (not closable from the pages viewed). | Inference |
| Scan | https://archive.org/details/B-001-001-277 (double-page colour scan; a private owner's copy with handwritten pen underlining. Printed red was distinguishable from the pen marks on the pages used.) | |
| Use here | Gen 1:1 and Ps 23:1 controls; cross-check of John 3 and Rev 1. Its title page limits red to the New Testament. | |

E2 is a modern printing whose apparatus may still be in copyright. It is used only as a fact check
(which words are red). Nothing from it is copied into the app.

### Candidates examined but not used

- *New Testament: Red Letter Edition* (World Publishing Co., Cleveland), archive.org `newtestamentredl0000vari`.
  The catalogue says 1901, but the imprint gives "2231 West 110th Street, Cleveland", which looks
  much later. **Date unreliable; to confirm.** Not used.
- *Holman Self-Pronouncing Red Letter Edition* New Testament (A. J. Holman, 1904), archive.org
  `newtestamentour00fitrgoog` (a Google scan). It has no usable OCR, and I didn't check whether its
  colour survived scanning. Not examined. It could serve as a further edition later.
- An **original** (non-facsimile) scan of Klopsch's 1899 New Testament or 1901 Bible: still **not
  found**. In the first pass I searched archive.org for Klopsch and *Christian Herald* red-letter
  editions. In the second pass the only Klopsch item I used was the facsimile above, found through
  CrossWire's link and the source review's note. Finding an original copy remains **to confirm**.

## 3. How the boundaries were read, and the limits

- **Method (E1, E2, first pass).** I located each verse with the scan's OCR, then opened the page
  image itself in a browser and viewed it magnified (about one column at a time). Close calls were
  viewed again at higher zoom. OCR was used only to find pages. It carries no colour, so no colour
  judgement rests on OCR.
- **Method (E0, second pass).** I located pages with the e-book's OCR text (an earlier session's
  copy of the archive.org search text, used only to find leaf numbers; every reading was then made
  from the image). I loaded each leaf at full resolution in the app's browser pane, then viewed
  crops at 0.75× to 1.6× native size. **Every red/black judgement at a boundary was made at about
  1:1 (0.9× to 1.6×).** E0's crimson can look black below about 0.7×, especially in right-hand
  columns, so whole-column overviews were used only to find verses. Where I recorded a verse as
  black, red text was visible in the same view for comparison, or I compared it with the black
  verse numerals at 1.6× (Rev 1:8, Rev 3:6).
- **Method (E2 Ps 22:1, 2026-10-20).** I loaded the full double-page image of leaf n312 (3424 × 2558
  px, `https://archive.org/download/B-001-001-277/page/n312.jpg`) in the browser pane, read the
  folios (610 left, 611 right) and the running heads (Psalms 19, 20 / Psalms 21, 22), and viewed
  Ps 22:1–4 at about 0.78× and 1.25× native size. As a second, mechanical check I counted dark
  pixels that are clearly red (R − G > 45 and R − B > 35) in the verse area: 0 of about 20,000 ink
  pixels in 22:1, and 0 on the whole spread. The same count on a known red page of the same scan
  (n546, John 3–4) found 38% and 16% of ink pixels red on its two pages. So the scan does carry
  red where the book printed it, and none at Ps 22:1. The pixel count supports the visual reading;
  it does not replace the human spot-check.
- **Who read them.** The readings were made by the scholar agent (an AI) viewing the page images.
  The owner asked for hand-checked golden samples, so **a person must spot-check every sample
  against the same pages** before CI treats them as binding (section 13).
- **Word-level boundaries only.** At screen resolution I can reliably tell which **words** are red.
  The colour of a single comma, colon or space at a boundary is often not reliable. So every boundary
  is stated at the word level. Punctuation and spaces at a boundary are **not asserted**.
- **Verse numbers and pilcrows (¶)** are printed black in E1 and E0 and are not counted as part of
  any span. Reference letters (superior a, b, c) inside red text are printed red. Ignore them.
- **Transcription convention for quoted boundary words.** E1 and E0 are self-pronouncing editions.
  Proper names carry accents, breves and hyphens. The boundary words quoted below drop those marks
  and hyphens. In every other respect, including spelling, they are as read from the scan. Example:
  E1 and E0 both print "their's" at Mt 5:3.
- **One copy per edition.** A different printing of the same title could differ.

## 4. Verification summary

| Status | E1/E2 (first pass) | E0 (second pass) |
|---|---|---|
| `verified against scan image` | 99 of 99 verse rows (Mk 8:2 end closed this pass at E1 p. 89, n90), plus RL-42 (Ps 22:1) in E2, closed 2026-10-20 at E2 p.611 (n312) | **99 of 99** verse rows, plus RL-42 (Ps 22:1). E0 is a modern facsimile, so its tag is `verified-scan-E0-facsimile`. |
| of which, read only at whole-page resolution, not magnified | 0 (Mt 4:19 in E1 re-viewed magnified, 2026-10-20) | 0 |
| `located in scan, color not verifiable with my tools` | 0 | 0 |
| `to confirm by a human against the printed/scanned edition` | **all rows** (recommended human spot-check) | **all rows** (recommended human spot-check) |

| Comparison (word level) | Result |
|---|---|
| E0 vs E1, 99 rows | **98 agree, 1 differs** (Rev 1:8). Plus one difference of scope: E0 prints Old Testament passages in red (RL-42). See section 7. |
| Source vs E0, 99 rows | **99 match, 0 mismatches.** RL-42 (Ps 22:1, outside the original 99) is a **mismatch**: E0 red, source unmarked. See section 9. |
| Source vs E1, 99 rows | **98 match, 1 mismatch** (Rev 1:8). |
| RL-42 (Ps 22:1), E2 | **E2 no red** (p.611, n312; closed 2026-10-20). So at Ps 22:1 the source agrees with E2 and differs from E0. |

All source-side values are **derived by me from the machine-extracted excerpts**, with the
tokenizer described in section 10. They are pending human confirmation.

## 5. The samples (as printed in E1, with E2 cross-checks)

Citation shorthand: **E1 p.P (nN)** means E1, printed page P, archive.org leaf nN. **E2 nN** means
E2, archive.org leaf nN. **E0 p.P (nN)** means E0, printed page P, leaf nN of the facsimile item.
Link patterns are in section 2.

"Black …" is narration or another speaker, printed black. "RED …" is printed red. Quoted words are
boundary markers only. Section 6 gives the E0 reading for every row.

| ID | Reference | Category | Why it's tricky | Boundary as printed (E1) | Citation | Status |
|---|---|---|---|---|---|---|
| RL-01 | Matt 3:15 | Mid-verse start **and** end | Jesus' words sit between narration and a narrative sentence | Black to "…said unto him," / RED "Suffer … righteousness." (italic *it to be so* is also red) / Black "Then he suffered him." | E1 p.9 (n10) | verified against scan image |
| RL-02 | Matt 3:17 | **Control:** another divine speaker | Voice from heaven ("This is my beloved Son…"). Not Jesus. | **No red** in verse | E1 p.9 (n10) | verified against scan image |
| RL-03 | Matt 4:4; 4:6; 4:7 | OT quotation inside Jesus' words; **control** of Scripture quoted by another speaker | 4:4 and 4:7 quote Deuteronomy inside Jesus' reply. In 4:6 the devil quotes Ps 91. | 4:4 Black "But he answered and said," / RED "It is written, … mouth of God." to verse end. 4:6 **no red**. 4:7 Black "Jesus said unto him," / RED "It is written again, … the Lord thy God." to verse end. | E1 p.9 (n10) | verified against scan image |
| RL-04 | Matt 4:10 | OT quotation inside Jesus' words | Imperative plus "for it is written" plus the quotation, all one span | Black "Then saith Jesus unto him," / RED "Get thee hence, … shalt thou serve." to verse end | E1 p.10 (n11) | verified against scan image |
| RL-05 | Matt 4:17 | Mid-verse start | Narration "began to preach, and to say," leads in | Black to "…and to say," / RED "Repent: … at hand." to verse end | E1 p.10 (n11) | verified against scan image |
| RL-06 | Matt 5:2–3 | Whole-verse baseline; start at a verse boundary | 5:2 is narration only. Red starts exactly at 5:3. | 5:2 **no red**. 5:3 whole verse RED ("Blessed are … kingdom of heaven."). | E1 p.11 (n12) | verified against scan image |
| RL-07 | Matt 5:12–14; 4:19 | Whole-verse baseline (carried across verses); simple mid-verse start | Sanity checks | 5:12, 5:13, 5:14 each whole verse RED. 4:19 Black "And he saith unto them," / RED "Follow me, … fishers of men." to verse end. | E1 p.11 (n12); 4:19 at E1 p.10 (n11) | verified against scan image (4:19 in E1 re-viewed magnified on 2026-10-20, at about 0.94× native of the 2144 × 3718 px image: same boundary; at 0.62× in E0) |
| RL-08 | Matt 9:6 | **Narration inside Jesus' speech** (two spans) | Parenthesis "(then saith he to the sick of the palsy,)" splits the speech | RED "But that ye may know … to forgive sins," (carried from 9:5) / Black "(then saith he to the sick of the palsy,)" / RED "Arise, take up thy bed, … thine house." to verse end | E1 p.20 (n21) | verified against scan image |
| RL-09 | Matt 22:21 | Another speaker, then narration, then Jesus, in one verse | Interlocutors' answer "Caesar's." must stay black | Black "They say unto him, Caesar's. Then saith he unto them," / RED "Render therefore … are God's." to verse end. (Also 22:24, the Sadducees quoting Moses: **no red**.) | E1 p.52 (n53) | verified against scan image |
| RL-10 | Matt 26:26–28 | Mid-verse starts; span carried across a verse | Last Supper. Red resumes after narration in 26:27, then runs whole-verse in 26:28. | 26:26 Black to "…and said," / RED "Take, eat; this is my body." 26:27 Black to "…saying," / RED "Drink ye all of it;". 26:28 whole verse RED. | E1 p.63 (n64) | verified against scan image |
| RL-11 | Matt 27:46 | **Aramaic with interpretive narration** | Is the translation part of the red span? | Black to "…with a loud voice, saying," / RED "Eli, Eli, lama sabachthani?" / **Black** "that is to say, My God, my God, why hast thou forsaken me?" (high zoom) | E1 p.68 (n69) | verified against scan image |
| RL-12 | Mark 5:41 | Aramaic with interpretive narration | Same question. The translation includes "I say unto thee". | Black to "…said unto her," / RED "Talitha cumi;" / **Black** "which is, being interpreted, Damsel, I say unto thee, arise." (high zoom) | E1 p.82 (n83) | verified against scan image |
| RL-13 | Mark 7:34 | Aramaic with interpretive narration | Single-word span | Black to "…saith unto him," / RED "Ephphatha," / **Black** "that is, Be opened." | E1 p.88 (n89) | verified against scan image |
| RL-14 | Mark 8:1–2 | Span starts exactly at a verse start after a narration-only verse | Boundary falls on the verse division | 8:1 **no red** (ends "…and saith unto them,"). 8:2 **whole verse RED**, from "I have compassion on…" to "…nothing to eat:", and the red runs on into 8:3. | E1 p.88 (n89) and **p.89 (n90)** | verified against scan image (**closed 2026-10-20**: verse end viewed at E1 p.89, n90, and in E0) |
| RL-15 | Mark 15:34 | Aramaic with interpretive narration | Parallel to Matt 27:46 | Black to "…with a loud voice, saying," / RED "Eloi, Eloi, lama sabachthani?" / **Black** "which is, being interpreted, My God, my God, why hast thou forsaken me?" | E1 p.110 (n111) | verified against scan image |
| RL-16 | Luke 4:17–21 | **OT quotation read aloud by Jesus** (Isa 61); whole verses; crosses a verse | 4:18–19 is Isaiah, read by Jesus. 4:17 and 4:20 are narration. | 4:17 **no red** (ends "…where it was written,"). 4:18 whole verse RED. 4:19 whole verse RED. 4:20 **no red**. 4:21 Black "And he began to say unto them," / RED "This day … in your ears." to verse end. | E1 p.124 (n125) | verified against scan image |
| RL-17 | Luke 23:46–47 | OT quotation (Ps 31:5); mid-verse end; **control** of another speaker | Narration follows the saying in the same verse | 23:46 Black to "…he said," / RED "Father, into thy hands I commend my spirit:" / Black "and having said thus, he gave up the ghost." 23:47 (centurion) **no red**. | E1 p.180 (n181) | verified against scan image |
| RL-18 | John 3:2–5 | "Verily, verily, I say unto thee" after narration; alternating speakers | Nicodemus (3:2, 3:4) must stay black | 3:2 **no red**. 3:3 Black "Jesus answered and said unto him," / RED "Verily, verily, … kingdom of God." to verse end. 3:4 **no red**. 3:5 Black "Jesus answered," / RED "Verily, verily, … kingdom of God." to verse end. | E1 p.189 (n190) | verified against scan image |
| RL-19 | John 3:9–15 | **Contrast block for John 3:16–21**; mid-verse start, then whole verses | Red resumes mid-verse in 3:10, then runs unbroken | 3:9 **no red**. 3:10 Black "Jesus answered and said unto him," / RED "Art thou a master of Israel, … these things?" to verse end (the span crosses a column break). 3:11, 3:12, 3:13, 3:14, 3:15 each whole verse RED. | E1 p.189 (n190) | verified against scan image |
| RL-20 | John 3:16–21 | **Where editions differ about the end of Jesus' words** | Some interpreters end the quotation at 3:15 (section 8) | 3:16, 3:17, 3:18, 3:19, 3:20, 3:21 each whole verse RED. 3:22 **no red**. E2 agrees: 3:16–21 red, 3:22 not red. | E1 p.189 (n190) and p.190 (n191); E2 p.1078 (n546) | verified against scan image (E1 and E2) |
| RL-21 | John 11:34–35 | Mid-verse end where another speaker follows; **control** of a short narrative verse | "Jesus wept." is narration about Jesus | 11:34 Black "And said," / RED "Where have ye laid him?" / Black "They said unto him, Lord, come and see." 11:35 **no red**. | E1 p.213 (n214) | verified against scan image |
| RL-22 | John 19:26–30 | Short sayings at mid-verse positions | Several one-clause spans with narration on either side | 19:26 Black to "…saith unto his mother," / RED "Woman, behold thy son!" to verse end. 19:27 Black "Then saith he to the disciple," / RED "Behold thy mother!" / Black "And from that hour…" to verse end. 19:28 Black to "…saith," / RED "I thirst." to verse end. 19:30 Black to "…he said," / RED "It is finished:" / Black "and he bowed his head, and gave up the ghost." | E1 p.232 (n233) | verified against scan image |
| RL-23 | Acts 1:4–5 | **Post-resurrection words; narration ("saith he") inside the speech** | Indirect speech turns into direct speech mid-verse | 1:4 Black "And, being assembled … depart from Jerusalem, but" / RED "wait for the promise of the Father, which," / Black (italic) "saith he," / RED "ye have heard of me." to verse end (high zoom). 1:5 whole verse RED. | E1 p.237 (n238) | verified against scan image |
| RL-24 | Acts 1:6–11 | Post-resurrection words; **controls**: disciples, narrative, angels | 1:9 is the narrative control requested for Acts | 1:6 **no red**. 1:7 Black "And he said unto them," / RED "It is not for you to know … in his own power." to verse end. 1:8 whole verse RED. 1:9, 1:10, 1:11 **no red** (1:11 is the "two men in white apparel" speaking). | E1 p.238 (n239) | verified against scan image |
| RL-25 | Acts 9:4–6 | Words of the ascended Christ, interleaved with Saul's replies | Two speakers alternate within 9:5 and 9:6 | 9:4 Black to "…saying unto him," / RED "Saul, Saul, why persecutest thou me?" to verse end. 9:5 Black "And he said, Who art thou, Lord? And the Lord said," / RED "I am Jesus whom thou persecutest: … against the pricks." to verse end. 9:6 Black "And he trembling and astonished said, … And the Lord said unto him," / RED "Arise, and go into the city, … what thou must do." to verse end. | E1 p.257 (n258) | verified against scan image |
| RL-26 | Acts 9:10 | One-word mid-verse span (the Lord in a vision) | A single vocative between narration and Ananias' reply | Black to "…said the Lord in a vision," / RED "Ananias." / Black "And he said, Behold, I am here, Lord." (high zoom) | E1 p.257 (n258) | verified against scan image |
| RL-27 | Acts 20:35 | **Saying of Jesus quoted by Paul** | Words of Jesus inside another speaker's speech | Black "I have shewed you all things, … the words of the Lord Jesus, how he said," / RED "It is more blessed to give than to receive." to verse end | E1 p.287 (n288) | verified against scan image |
| RL-28 | 1 Cor 11:24–26 | Words of institution quoted by Paul; **control** of Paul's own comment | 11:26 continues the thought but is Paul's | 11:24 Black "And when he had given thanks, he brake it, and said," / RED "Take, eat: … in remembrance of me." to verse end. 11:25 Black "After the same manner … when he had supped, saying," / RED "This cup is the new testament … in remembrance of me." to verse end. 11:26 **no red**. | E1 p.350 (n351) | verified against scan image |
| RL-29 | 2 Cor 12:9 | Words of the ascended Christ reported by Paul; mid-verse start and end; the verse crosses a column | Paul's response follows in the same verse | Black "And he said unto me," / RED "My grace is sufficient for thee: … made perfect in weakness." / Black "Most gladly therefore will I rather glory…" to verse end (top of the next column) | E1 p.376 (n377) | verified against scan image |
| RL-30 | Rev 1:7–8 | **Where editions differ:** who speaks in 1:8 | The attribution "saith the Lord" and the titles that follow it | 1:7 **no red** (including "Even so, Amen."). 1:8 RED "I am Alpha and Omega, the beginning and the ending," / **Black** "saith the Lord, which is, and which was, and which is to come, the Almighty." (high zoom). E2 agrees. **E0 differs: the whole of 1:8 is black** (section 7). | E1 p.486 (n487); E2 p.1246 (n630); E0 p.1254 (n1277) | verified against scan image (E1, E2 and E0) |
| RL-31 | Rev 1:11 | **Narration inside the speech (two spans)** | A one-word black "and," between two red spans | Black "Saying," / RED "I am Alpha and Omega, the first and the last:" / **Black** "and," / RED "What thou seest, write in a book, … and unto Laodicea." to verse end (high zoom). E2 agrees. E0 agrees. | E1 p.486 (n487); E2 p.1246 (n630); E0 p.1254 (n1277) | verified against scan image (E1, E2 and E0) |
| RL-32 | Rev 1:17–18 | Risen Christ; mid-verse start; carried into the next verse | | 1:17 Black "And when I saw him, … saying unto me," / RED "Fear not; I am the first and the last:" to verse end. 1:18 whole verse RED (1:19 and 1:20 also whole verse RED). | E1 p.487 (n488) | verified against scan image |
| RL-33 | Rev 2:1 | Letter to a church: the command "Unto the angel … write" | Is the address formula itself red? | Whole verse RED, including "Unto the angel of the church of Ephesus write; These things saith he that…" (carried from 1:20) | E1 p.487 (n488) | verified against scan image |
| RL-34 | Rev 3:6; 3:20; 3:22 | Letters to the churches; the refrain "what the Spirit saith" | The refrain speaks of the Spirit | 3:6 whole verse RED. 3:20 whole verse RED ("Behold, I stand at the door, and knock: … and he with me."). 3:22 whole verse RED. | 3:6 at E1 p.490 (n491); 3:20, 3:22 at E1 p.491 (n492) | verified against scan image |
| RL-35 | Rev 4:1 | **Control (edition-sensitive):** the trumpet-like voice of 1:10 | "Come up hither, and I will shew thee…" is a voice that 4:1 links to 1:10 | **No red** in verse | E1 p.491 (n492) | verified against scan image |
| RL-36 | Rev 22:7 | Mid-verse end | A beatitude follows the saying in the same verse | RED "Behold, I come quickly:" / Black "blessed is he that keepeth the sayings of the prophecy of this book." (22:6 ends black, so red starts at the verse start) | E1 p.517 (n518) | verified against scan image |
| RL-37 | Rev 22:11–14 | Whole verses, then a stop at 22:14 | 22:14 is often read as part of the speech | 22:11 **no red**. 22:12 whole verse RED (starts "And, behold, I come quickly;"). 22:13 whole verse RED. 22:14 **no red**. | E1 p.517 (n518) | verified against scan image |
| RL-38 | Rev 22:15–17 | Whole verse between controls | | 22:15 **no red**. 22:16 whole verse RED ("I Jesus have sent mine angel … bright and morning star."). 22:17 **no red**. | E1 p.518 (n519) | verified against scan image |
| RL-39 | Rev 22:20 | Mid-verse start **and** end | John's response follows in the same verse | Black "He which testifieth these things saith," / RED "Surely I come quickly." / Black "Amen. Even so, come, Lord Jesus." | E1 p.518 (n519) | verified against scan image |
| RL-40 | Gen 1:1 | **Control (OT)** | | **No red** (the whole first page of Genesis is black) | E2 p.9 (n11), right-hand page (folio read at 1:1 on 2026-10-20); E0 p.17 (n40) | verified against scan image (E2 and E0) |
| RL-41 | Ps 23:1 | **Control (OT)** | Ps 23 isn't red in E0 either | **No red** (the first pass also saw Psalm 22 black on this E2 page; that page holds only 22:13 onward. Ps 22:1 is on E2 p.611, n312: see RL-42) | E2 p.612 (n313); E0 p.621 (n644) | verified against scan image (E2 and E0) |
| RL-42 | Ps 22:1 | **New, second pass: OT passage quoted by Jesus** (Mt 27:46, Mk 15:34) | Klopsch reds Old Testament passages quoted by Jesus. E1 has no Old Testament, and E2 limits red to the New Testament. | **E0:** RED "My God, my God, why hast thou forsaken me?" (red drop capital) / Black "why art thou so far from helping me, and from the words of my roaring?", with a red star in the margin. **E1:** not applicable (NT only). **E2 (closed 2026-10-20):** **no red** in 22:1. The drop capital, "My God, my God, why hast thou forsaken me?" and "why art thou so far … my roaring?" are all black, with no star. Ps 22:1 is on E2 **p.611 (n312)**, top of the right-hand column, not p.612: p.612 (n313) begins at 22:13. | E0 p.620 (n643), at 1.13×; **E2 p.611 (n312)**, at about 0.78× and 1.25× | verified against scan image (E0 and E2) |

Groups RL-03, RL-07, RL-10, RL-14, RL-16 to RL-19, RL-21, RL-22, RL-24, RL-25, RL-28, RL-32, RL-34
and RL-37 each cover several verses. Section 10 lists one expectation per verse.

## 6. E0 re-check, row by row

E0 page and leaf for each group, what E0 shows, and whether it agrees with E1 at word level. Zoom is
the magnification at which the boundary was judged (native image = 1.0×).

| ID | Verses | E0 page (leaf) | E0 reading | vs E1 | Zoom |
|---|---|---|---|---|---|
| RL-01 | Mt 3:15 | p.981 (n1004) | Black to "…said unto him," / RED "Suffer … righteousness." / Black "Then he suffered him." | agrees | 0.66× (red and black side by side) |
| RL-02 | Mt 3:17 | p.981 (n1004) | No red | agrees | 0.7× (red 4:4 in same view) |
| RL-03 | Mt 4:4, 4:6, 4:7 | p.981 (n1004) | 4:4 RED from "It is written" to verse end; 4:6 no red; 4:7 RED "It is written again, … God." | agrees | 0.7× |
| RL-04 | Mt 4:10 | p.981 (n1004) | Black "Then saith Jesus unto him," / RED "Get thee hence, … serve." | agrees | 0.66× |
| RL-05 | Mt 4:17 | p.982 (n1005) | Black to "…and to say," / RED "Repent: … hand." | agrees | 1.19× |
| RL-06 | Mt 5:2–3 | p.982 (n1005) | 5:2 no red; 5:3 whole verse RED | agrees | 1.19× |
| RL-07 | Mt 5:12–14; 4:19 | p.982 (n1005) | 5:12–14 whole verses RED; 4:19 Black "And he saith unto them," / RED "Follow me, … men." | agrees | 0.91× (5:12–14), 0.62× (4:19, red clearly visible) |
| RL-08 | Mt 9:6 | p.987 (n1010) | RED "But that ye may know … forgive sins," / Black "(then saith he to the sick of the palsy,)" / RED "Arise, … house." | agrees | 0.73×–1.0× |
| RL-09 | Mt 22:21, 22:24 | p.1006 (n1029) | 22:21 Black "They say unto him, Cæsar's. Then saith he unto them," / RED "Render … God's."; 22:24 no red | agrees | 0.73× |
| RL-10 | Mt 26:26–28 | p.1012 (n1035) | 26:26 RED "Take, eat; this is my body."; 26:27 RED "Drink ye all of it;"; 26:28 whole verse RED | agrees | 0.73× |
| RL-11 | Mt 27:46 | p.1015 (n1038) | Black to "…saying," / RED "Eli, Eli, lama sabachthani?" / **Black** "that is to say, My God, my God, why hast thou forsaken me?" | agrees | 1.0× |
| RL-12 | Mk 5:41 | p.1023 (n1046) | Black to "…said unto her," / RED "Talitha cumi;" / **Black** "which is, being interpreted, Damsel, I say unto thee, arise." | agrees | 0.99× |
| RL-13 | Mk 7:34 | p.1026 (n1049) | RED "Ephphatha," (split across a line) / **Black** "that is, Be opened." | agrees | 1.35× |
| RL-14 | Mk 8:1–2 | p.1026 (n1049) | 8:1 no red, ends "…saith unto them,"; 8:2 whole verse RED to "…eat:"; 8:3 red | agrees (E1 end now viewed: same) | 0.99× |
| RL-15 | Mk 15:34 | p.1039 (n1062) | Black to "…saying," / RED "Eloi, Eloi, lama sabachthani?" / **Black** "which is, being interpreted, …" | agrees | 0.85× |
| RL-16 | Lk 4:17–21 | p.1047 (n1070) | 4:17 no red; 4:18 and 4:19 whole verses RED (4:18 crosses the column); 4:20 no red; 4:21 Black "And he began to say unto them," / RED "This day … ears." | agrees | 0.79× |
| RL-17 | Lk 23:46–47 | p.1079 (n1102) | 23:46 Black to "…he said," / RED "Father, into thy hands I commend my spirit:" (foot of left column) / Black "and having said thus, he gave up the ghost." (top of right column); 23:47 no red | agrees | 0.79× |
| RL-18 | Jn 3:2–5 | p.1084 (n1107) | 3:2 no red; 3:3 Black "Jesus answered and said unto him," / RED "Verily, …"; 3:4 no red; 3:5 Black "Jesus answered," / RED "Verily, …" | agrees | 0.76× |
| RL-19 | Jn 3:9–15 | p.1084 (n1107) | 3:9 no red; 3:10 Black "Jesus answered and said unto him," / RED "Art thou … things?"; 3:11–15 whole verses RED | agrees | 0.76× |
| RL-20 | Jn 3:16–22 | p.1084 (n1107) | 3:16–21 whole verses RED; 3:22 no red | agrees | 0.99× |
| RL-21 | Jn 11:34–35 | p.1098 (n1121) | 11:34 Black "And said," / RED "Where have ye laid him?" / Black "They said unto him, Lord, come and see."; 11:35 no red | agrees | 0.79× |
| RL-22 | Jn 19:26–30 | p.1108 (n1131) | Same spans as E1 at 19:26, 19:27, 19:28 and 19:30 | agrees | 0.85× |
| RL-23 | Acts 1:4–5 | p.1111 (n1134) | 1:4 Black to "…from Jerusalem, but" / RED "wait for the promise of the Father, which," / Black (italic) "saith he," / RED "ye have heard of me."; 1:5 whole verse RED | agrees | 1.19× |
| RL-24 | Acts 1:6–11 | p.1111 (n1134), p.1112 (n1135) | 1:6 no red; 1:7 Black "And he said unto them," / RED "It is not for you … power." (crosses the page); 1:8 whole verse RED; 1:9–11 no red | agrees | 0.79×–0.95× |
| RL-25 | Acts 9:4–6 | p.1123 (n1146) | Same spans as E1 | agrees | 0.76×–1.13× |
| RL-26 | Acts 9:10 | p.1123 (n1146) | Black to "…in a vision," / RED "Ananias." / Black "And he said, Behold, I am here, Lord." | agrees | 1.13× |
| RL-27 | Acts 20:35 | p.1140 (n1163) | Black to "…how he said," / RED "It is more blessed to give than to receive." | agrees | 0.85× |
| RL-28 | 1 Cor 11:24–26 | p.1175 (n1198), p.1176 (n1199) | 11:24 Black "…and said," / RED "Take, eat: … of me."; 11:25 Black "…saying," / RED "This cup … of me."; 11:26 no red | agrees | 0.74×–0.85× |
| RL-29 | 2 Cor 12:9 | p.1190 (n1213) | Black "And he said unto me," / RED "My grace … weakness." / Black "Most gladly therefore …" | agrees | 0.85× |
| RL-30 | Rev 1:7–8 | p.1254 (n1277) | 1:7 no red. **1:8 no red at all**: "I am Alpha and Omega, the beginning and the ending, saith the Lord, …, the Almighty." is all black. | **DIFFERS** | 0.85×, 1.13× and 1.58× (compared with the black verse numeral "8", and with red 1:11 at the same zoom) |
| RL-31 | Rev 1:11 | p.1254 (n1277) | Black "Saying," / RED "I am Alpha and Omega, the first and the last:" / Black "and," / RED "What thou seest, … Laodicea." | agrees | 1.13×–1.58× |
| RL-32 | Rev 1:17–20 | p.1254 (n1277), p.1255 (n1278) | 1:17 Black "… saying unto me," / RED "Fear not; I am the first and the last:" (top of p.1255); 1:18–20 whole verses RED | agrees | 0.76× |
| RL-33 | Rev 2:1 | p.1255 (n1278) | Whole verse RED, including "Unto the angel of the church of Ephesus write;" | agrees | 0.76× |
| RL-34 | Rev 3:6; 3:20; 3:22 | p.1256 (n1279); p.1257 (n1280) | All three whole verses RED | agrees | 1.58× (3:6); 0.79× (3:20, 3:22, with black 4:1 below) |
| RL-35 | Rev 4:1 | p.1257 (n1280) | No red, including "Come up hither, …" | agrees | 0.79× (red 3:22 in same view) |
| RL-36 | Rev 22:7 | p.1271 (n1294) | 22:6 no red; 22:7 RED "Behold, I come quickly:" / Black "blessed is he …" | agrees | 1.05× |
| RL-37 | Rev 22:11–14 | p.1272 (n1295) | 22:11 no red; 22:12, 22:13 whole verses RED; 22:14 no red | agrees | 0.74× (red and black side by side) |
| RL-38 | Rev 22:15–17 | p.1272 (n1295) | 22:15 no red; 22:16 whole verse RED (runs into the right column); 22:17 no red | agrees | 0.74× |
| RL-39 | Rev 22:20 | p.1272 (n1295) | Black "He which testifieth these things saith," / RED "Surely I come quickly." / Black "Amen. Even so, come, Lord Jesus." | agrees | 0.74× |
| RL-40 | Gen 1:1 | p.17 (n40) | No red | agrees with E2 | 1.05× |
| RL-41 | Ps 23:1 | p.621 (n644) | No red (both halves of the line checked) | agrees with E2 | 1.13× |
| RL-42 | Ps 22:1 | p.620 (n643) | RED "My God, my God, why hast thou forsaken me?" / Black "why art thou so far …" | E1 n/a; **E2 differs**: no red at 22:1, E2 p.611 (n312), viewed 2026-10-20 at about 1.25× | 1.13× |

**Note on zoom.** Rows judged at 0.74× to 0.85× had red and black text side by side in the same view,
and the contrast was clear. I still recommend the human spot-check view these pages at 100% or more,
because E0's crimson darkens at lower magnification.

## 7. E0 vs E1 differences

Stated as facts about these two copies. No edition is preferred.

| # | Where | E0 (Klopsch 1901, facsimile) | E1 (Oxford, c. 1913–15) | Also | Rows |
|---|---|---|---|---|---|
| 1 | **Rev 1:8** | Whole verse **black** (p.1254, n1277) | RED "I am Alpha and Omega, the beginning and the ending," then black "saith the Lord, …" (p.486, n487) | E2 agrees with E1 (p.1246, n630). The **source agrees with E0** (no `<q>` in Rev 1:8). | RL-30 |
| 2 | **Old Testament scope** | Old Testament passages quoted or referred to by Jesus are printed red (title page n24; Explanatory Note p. xvi, n39). Observed at Ps 22:1 (p.620, n643). | New Testament only; there is no Old Testament in E1 | E2's title page limits red to the New Testament (title page image n7, viewed 2026-10-20), and **E2 prints Ps 22:1 black** (p.611, n312; viewed 2026-10-20). The **source has no Old Testament markup** (0 `<q>` tags before Matthew 1:1). | RL-42 (new). RL-40 and RL-41 are black in all editions checked. |
| 3 | Old Testament prophecy stars and red reference-column entries | Present (title page, Explanatory Note points III and IV; red star seen beside Ps 22:1) | Not present | Apparatus, not words of Jesus. Outside the app's red-letter scope. Recorded for completeness. | — |

**No other differences** were found in the 99 rows. In particular, E0 agrees with E1 at every
priority category listed in the brief:

- **Aramaic sayings** (Matt 27:46, Mark 5:41, 7:34, 15:34): E0 reds only the Aramaic. The
  interpretive narration is black, as in E1.
- **Acts 1, Acts 9 and Acts 20:35, 1 Cor 11:24–25, 2 Cor 12:9**: E0 reds the same words as E1,
  including the "saith he" break in Acts 1:4 and the single word "Ananias." in Acts 9:10.
- **Rev 1:11, 4:1, 22:7, 22:14, 22:20**: same as E1. **Rev 1:8 differs** (item 1).
- **John 3:16–21**: red in E0, as in E1 and E2; 3:22 black.

**A footnote in E0 relevant to item 1** (reported, not interpreted). At the foot of p.1254
(n1277), the page that holds Rev 1:1–17, E0 prints a footnote beginning "The highest Biblical
authorities, while differing on other passages, unite in ascribing" to Jesus "the words printed in
red". I didn't locate its reference mark on the page. The Explanatory Note (p. xvi) also describes
New Testament red as the words "universally accepted as the utterances of our Lord and Saviour".
These are Klopsch's statements of method. I record them without drawing a conclusion about Rev 1:8.

## 8. Known edition differences and edition-sensitive passages

Stated as facts about specific editions, with sources. No edition is preferred.

1. **John 3:16–21.** E0, E1 and E2 all print 3:16–21 entirely in red, and 3:22 in black (RL-20). The
   source agrees. Interpreters differ on whether Jesus' words end at 3:15. The NRSV Updated Edition
   has a footnote saying "Some interpreters hold that the quotation concludes with verse 15" (quoted
   in Wikipedia, "Red letter edition"; **to confirm against a printed NRSVue**). I have still found
   no scanned KJV red-letter edition that stops the red at 3:15. Whether any does is **to confirm**.
2. **Revelation 1:8.** E1 and E2 red "I am Alpha and Omega, the beginning and the ending," and print
   the rest black. **E0 prints the whole verse black** (section 7, item 1). The source has no red in
   1:8. Commentators aren't certain who speaks here. Albert Barnes, *Notes* on Rev 1:8, writes that
   it "can not be absolutely certain that the writer meant to refer to the Lord Jesus specifically
   here" (read via biblehub.com; **to confirm against a printed Barnes**). So editions legitimately
   differ at this verse. The app follows its source, which matches E0.
3. **Revelation 1:11.** E0, E1 and E2 agree: two red spans with a black "and," between them (RL-31).
   The source agrees.
4. **Acts 20:35, 1 Cor 11:24–25, 2 Cor 12:9, Acts 9, Revelation.** E0 and E1 both red the words of
   Jesus outside the Gospels, including sayings quoted by Paul (RL-25 to RL-29). E2's title page
   limits red to the New Testament (title page image n7, viewed 2026-10-20). Whether some KJV red-letter editions red the Gospels
   only is **to confirm**. I have no cited example.
5. **Aramaic sayings with a translation (Matt 27:46, Mark 5:41, 7:34, 15:34).** E0 and E1 red only
   the Aramaic. The interpretive clause ("that is to say…", "which is, being interpreted…") is black,
   including "Damsel, I say unto thee, arise" at Mark 5:41 (RL-11 to RL-15). The source agrees: its
   `<q>` encloses only the Aramaic words in all four verses.
6. **The Old Testament.** Crossway (2006) reports that Klopsch's first red-letter Bible printed in red
   "Old Testament passages that Jesus quoted or that were directly related to incidents to which he
   referred". **E0 confirms this on its own title page** (leaf n24), and Ps 22:1 is printed red
   (RL-42). E2's title page limits red to the New Testament (title page image n7, viewed 2026-10-20), and E2 prints Ps 22:1 black
   (p.611, n312; viewed 2026-10-20). Gen 1:1 and Ps 23:1 are black in E0 and E2. The pinned source marks **no** Old Testament text, so the app's red letters are New
   Testament only. This is recorded as a source property in section 9.
7. **Rev 4:1 and Rev 22:14.** E0 and E1 print both black (RL-35, RL-37), and so does the source. **I
   have no cited edition that reds either one.**
8. **Spelling.** E0 and E1 print "their's" at Matt 5:3. E0 prints "Cæsar's" at Matt 22:21, where the
   source has changed "æ" to "e" (`kjv-label.md`, fact 7). Printings differ in small details of
   spelling and punctuation. Don't use any printed edition for word-for-word text tests.

## 9. Source vs E0 comparison

**What was compared.** For each of the 99 rows, I took the source's `<q who="Jesus">` containers from
the machine-extracted excerpts (`docs/pipeline/golden-source-excerpts.md`, source SHA-256
`4bcbfe45…0e3f5`) and derived the red words with a small tokenizer (section 10). I compared them with
E0's (and E1's) red words: number of spans, where each span starts and ends (verse start, carried,
mid-verse), the first and last red word of each span, and the black words between or after spans.
Comparison is at the **word level**. Case, punctuation and the curly versus straight apostrophe were
ignored, because print punctuation isn't asserted (section 3).

**Result: 99 of 99 rows match E0. There are no word-level mismatches in the original sample set.**

| Group | Source vs E0 | Source vs E1 | Notes |
|---|---|---|---|
| RL-01 to RL-29 (Matthew to 2 Corinthians, 76 verse rows) | match (all) | match (all) | Includes all four Aramaic sayings, Acts 1:4 (two spans, black "saith he,"), Matt 9:6 (two spans), Acts 9:10, 20:35, 1 Cor 11:24–25 and 2 Cor 12:9. Mark 8:2: source red runs to verse end and stays open into 8:3, the same as print. |
| RL-30 Rev 1:7 | match | match | |
| **RL-30 Rev 1:8** | **match** (both no red) | **MISMATCH**: source no red; E1 and E2 red "I am Alpha … the ending," | Finding for the owner. The source follows E0 here. |
| RL-31 to RL-39 (Revelation, 19 verse rows) | match (all) | match (all) | Rev 2:1: see structural note (a) |
| RL-40, RL-41 (Gen 1:1, Ps 23:1) | match (no red) | match (E2) | |
| **RL-42 Ps 22:1** (new; not in the excerpts file) | **MISMATCH**: E0 red "My God, my God, why hast thou forsaken me?"; source no red | E1 n/a (NT only). **E2: match** (no red at 22:1, E2 p.611, n312; closed 2026-10-20) | Finding for the owner. The source has no Old Testament markup at all. Basis: a whole-file count of `<q` tags before `Matt.1.1` is 0, made by me on 2026-10-20. Ps 22:1 is **not** in `golden-source-excerpts.md`, so the developer should add it to the excerpt list to make this row checkable in the same way. |

**Structural notes** (not word-level mismatches; recorded so the tests are written correctly):

- (a) **Rev 1:20 → 2:1.** In the source, the `<q>` that starts in Rev 1:17 closes at the end of 1:20,
  and a new `<q>` opens at the first word of 2:1. On the page (E0 and E1) the red runs on without a
  break. At word level both are "whole verse red, previous verse ended red", so `start: carried` is
  correct for colour. But a test that counts `<q>` elements, or expects one span to continue across
  the chapter break, would see two spans where the reader sees one run of red. The YAML records
  `q_open_at_start: false` for REV.2.1 so this is explicit.
- (b) Several "whole verse red" rows close the `<q>` at the end of a verse even where the following
  verse in print is black (Luke 4:19, John 3:8, 3:21, Acts 1:5, 1:8, Rev 3:22, 22:13, 22:16). That is
  consistent with print. It is noted only because `q_open_at_end` varies between whole-verse rows.
- (c) **Old Testament.** E0 prints Old Testament red (title page; RL-42). The source has none. Every
  Old Testament verse that E0 prints in red would be a source-vs-E0 mismatch. I checked only Ps 22:1.
  How many verses that affects is **not established**.

**What this does and doesn't show.** The source's red letters agree with this copy of Klopsch 1901 at
every New Testament boundary sampled, including the ones where Oxford printings differ (Rev 1:8).
That supports the source's statement that its markup was matched to Klopsch's 1901 edition, **for the
New Testament**. It doesn't extend to the Old Testament, where the source has no red. It covers 99
sampled verses, not the whole text, and it rests on a modern facsimile.

## 10. Machine-checkable expectations (for M1-7)

### How to read this block

Each row has three columns.

- **`source_expected`**: what the pinned source marks. **Derived by me (the scholar agent) from
  `docs/pipeline/golden-source-excerpts.md`, not by a person, and not by the pipeline.** Tag:
  `status: derived-by-scholar-from-excerpts, human: pending`. Before CI treats a row as binding, a
  person checks it against the excerpt's reading-aid block (section 11, step 1). This is the column
  CI should assert against `content.sqlite`.
- **`print_E0`**: what E0 (Klopsch 1901, 2023 facsimile) prints. Tag `verified-scan-E0-facsimile`.
- **`print_E1`**: what E1 prints. For GEN.1.1, PSA.23.1 and PSA.22.1 this column holds **E2**
  (`edition: E2`), because E1 is New Testament only. Rows marked `edition: E1+E2` were also checked in E2.
- `source_vs_E0` and `source_vs_E1` are my word-level comparison (section 9). They are for the
  non-blocking "edition differences" report, **not** for CI failure.

Field meanings (all columns):

- `spans`: number of separate red runs in the verse, at word level.
- `start`: `verse_start` (red begins at the verse's first word, and the previous verse ended
  black), `carried` (red begins at the first word, and the previous verse in canonical order ended
  red), or `mid` (red begins after one or more black words). This is about **colour**, not about
  `<q>` elements. See `q_open_at_start`.
- `end`: `verse_end` or `mid`.
- `first` / `last`: first and last red word of a single span. `span1_first`, `span1_last`,
  `span2_first`, `span2_last`: the same for two spans. `black_between`: the black words between two
  spans. `black_after_first`: the first black word after a span that ends mid-verse.
- Print columns give boundary words as read from the scan, with pronunciation marks removed. Source
  columns give them exactly as the excerpt's text (outside tags) has them, including attached
  punctuation. **Compare words, not punctuation.** Strip punctuation, fold case, and treat ’ and '
  as equal.
- `q_open_at_start` / `q_open_at_end` (source only): copied from the excerpt's machine-derived lines
  "`q` open at verse start/end". Use these to test the `<q>` structure itself.

How I derived `source_expected`: I took each excerpt's **reading-aid** `text` block. I tracked
`<q …>` / `</q>` depth, starting at depth 1 when the excerpt says "`q` open at verse start: yes". I
dropped all tags (`<w>`, `<transChange>`, `<divineName>`, `<milestone>`) but kept their text. I
split the text on whitespace. A word counts as red if its letters are inside a `<q>`. No word was
partly red and partly black, so no row needed a judgement call. `carried` versus `verse_start` uses
`q_open_at_start`, or the previous verse's last word where the `<q>` closed at the verse boundary
(Rev 2:1). The previous verse was always in the excerpt set, except Rev 22:6,
whose `q` state I read from the source file: it has no `<q>`. Script:
`src_spans.py` in the author's scratchpad (scratch, not part of the repo). The developer may re-implement it. The values must agree.

**Totals:** 99 original rows plus RL-42 (PSA.22.1), so 100 rows. `source_vs_E0`: 99 match, 1
MISMATCH (PSA.22.1). `source_vs_E1`: 99 match, 1 MISMATCH (REV.1.8). For PSA.22.1 the `print_E1`
column holds E2 (E1 is NT only), as for GEN.1.1 and PSA.23.1; E2 prints 22:1 black (closed
2026-10-20), so source and E2 match there.

```yaml
# Red-letter golden samples, KJV. Book codes are USFM.
# source = BIB-KJV-CW31 (kjv.osis.xml, SHA-256 4bcbfe45722fca396adbeac9c81025bbb173cb1e30296da90864f0400cb0e3f5)
# E0 = Klopsch 1901 Red Letter Edition, viewed in a 2023 facsimile e-book (archive.org rubricated-american-heritage-bible-e-book)
# E1 = Oxford Self-Pronouncing Testament, c.1913-15 (archive.org oxfordselfpronou00oxforich)
# E2 = Oxford/Concordia Red Letter Edition, 1958-78 (archive.org B-001-001-277)
# Every row: human: pending until a person spot-checks it (section 11).
- ref: MAT.3.15
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Suffer", last: "righteousness.", black_after_first: "Then", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Suffer", last: "righteousness.", end: "mid", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Suffer", last: "righteousness.", end: "mid", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.3.17
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.4
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "It", last: "God.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "It", last: "God.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "It", last: "God.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.6
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.7
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "It", last: "God.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "It", last: "God.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "It", last: "God.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.10
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Get", last: "serve.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Get", last: "serve.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Get", last: "serve.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.17
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Repent:", last: "hand.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Repent:", last: "hand.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Repent:", last: "hand.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.4.19
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Follow", last: "men.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Follow", last: "men.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Follow", last: "men.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.5.2
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.5.3
  source_expected: {spans: 1, start: "verse_start", end: "verse_end", first: "Blessed", last: "heaven.", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.5.12
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Rejoice,", last: "you.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.5.13
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Ye", last: "men.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.5.14
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Ye", last: "hid.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.9.6
  source_expected: {spans: 2, start: "carried", end: "verse_end", span1_first: "But", span1_last: "sins,", black_between: "(then saith he to the sick of the palsy,)", span2_first: "Arise,", span2_last: "house.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 2, start: "carried", span1_last: "sins,", black_between: "(then saith he to the sick of the palsy,)", span2_first: "Arise,", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 2, start: "carried", span1_last: "sins,", black_between: "(then saith he to the sick of the palsy,)", span2_first: "Arise,", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.22.21
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Render", last: "God’s.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Render", last: "God's.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Render", last: "God's.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.22.24
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.26.26
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Take,", last: "body.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Take,", last: "body.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Take,", last: "body.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.26.27
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Drink", last: "it;", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Drink", last: "it;", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Drink", last: "it;", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.26.28
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "For", last: "sins.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MAT.27.46
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Eli,", last: "sabachthani?", black_after_first: "that", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Eli,", last: "sabachthani?", end: "mid", black_after_first: "that", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Eli,", last: "sabachthani?", end: "mid", black_after_first: "that", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MRK.5.41
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Talitha", last: "cumi;", black_after_first: "which", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Talitha", last: "cumi;", end: "mid", black_after_first: "which", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Talitha", last: "cumi;", end: "mid", black_after_first: "which", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MRK.7.34
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Ephphatha,", last: "Ephphatha,", black_after_first: "that", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Ephphatha,", last: "Ephphatha,", end: "mid", black_after_first: "that", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Ephphatha,", last: "Ephphatha,", end: "mid", black_after_first: "that", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MRK.8.1
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MRK.8.2
  source_expected: {spans: 1, start: "verse_start", end: "verse_end", first: "I", last: "eat:", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", first: "I", last: "eat:", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", first: "I", last: "eat:", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: MRK.15.34
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Eloi,", last: "sabachthani?", black_after_first: "which", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Eloi,", last: "sabachthani?", end: "mid", black_after_first: "which", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Eloi,", last: "sabachthani?", end: "mid", black_after_first: "which", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.4.17
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.4.18
  source_expected: {spans: 1, start: "verse_start", end: "verse_end", first: "The", last: "bruised,", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.4.19
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "To", last: "Lord.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.4.20
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.4.21
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "This", last: "ears.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "This", last: "ears.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "This", last: "ears.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.23.46
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Father,", last: "spirit:", black_after_first: "and", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Father,", last: "spirit:", end: "mid", black_after_first: "and", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Father,", last: "spirit:", end: "mid", black_after_first: "and", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: LUK.23.47
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.2
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.3
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Verily,", last: "God.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Verily,", last: "God.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Verily,", last: "God.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.4
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.5
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Verily,", last: "God.", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Verily,", last: "God.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Verily,", last: "God.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.6
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "That", last: "spirit.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.7
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Marvel", last: "again.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.8
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "The", last: "Spirit.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.9
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.10
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Art", last: "things?", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Art", last: "things?", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Art", last: "things?", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.11
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Verily,", last: "witness.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.12
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "If", last: "things?", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.13
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "And", last: "heaven.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.14
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "And", last: "up:", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.15
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "That", last: "life.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.16
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "For", last: "life.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.17
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "For", last: "saved.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.18
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "He", last: "God.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.19
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "And", last: "evil.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.20
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "For", last: "reproved.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.21
  edition_sensitive: true
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "But", last: "God.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.3.22
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.11.34
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Where", last: "him?", black_after_first: "They", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Where", last: "him?", end: "mid", black_after_first: "They", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Where", last: "him?", end: "mid", black_after_first: "They", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.11.35
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.19.26
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Woman,", last: "son!", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Woman,", last: "son!", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Woman,", last: "son!", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.19.27
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Behold", last: "mother!", black_after_first: "And", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Behold", last: "mother!", end: "mid", black_after_first: "And", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Behold", last: "mother!", end: "mid", black_after_first: "And", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.19.28
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "I", last: "thirst.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "I", last: "thirst.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "I", last: "thirst.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: JHN.19.30
  source_expected: {spans: 1, start: "mid", end: "mid", first: "It", last: "finished:", black_after_first: "and", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "It", last: "finished:", end: "mid", black_after_first: "and", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "It", last: "finished:", end: "mid", black_after_first: "and", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.4
  source_expected: {spans: 2, start: "mid", end: "verse_end", span1_first: "wait", span1_last: "which,", black_between: "saith he,", span2_first: "ye", span2_last: "me.", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 2, start: "mid", span1_first: "wait", span1_last: "which,", black_between: "saith he,", span2_first: "ye", span2_last: "me.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 2, start: "mid", span1_first: "wait", span1_last: "which,", black_between: "saith he,", span2_first: "ye", span2_last: "me.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.5
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "For", last: "hence.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.6
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.7
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "It", last: "power.", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "It", last: "power.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "It", last: "power.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.8
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "But", last: "earth.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.9
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.10
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.1.11
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.9.4
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Saul,", last: "me?", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Saul,", last: "me?", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Saul,", last: "me?", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.9.5
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "I", last: "pricks.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "I", last: "pricks.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "I", last: "pricks.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.9.6
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Arise,", last: "do.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Arise,", last: "do.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Arise,", last: "do.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.9.10
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Ananias.", last: "Ananias.", black_after_first: "And", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Ananias.", last: "Ananias.", end: "mid", black_after_first: "And", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Ananias.", last: "Ananias.", end: "mid", black_after_first: "And", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: ACT.20.35
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "It", last: "receive.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "It", last: "receive.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "It", last: "receive.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: 1CO.11.24
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Take,", last: "me.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Take,", last: "me.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Take,", last: "me.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: 1CO.11.25
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "This", last: "me.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "This", last: "me.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "This", last: "me.", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: 1CO.11.26
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: 2CO.12.9
  source_expected: {spans: 1, start: "mid", end: "mid", first: "My", last: "weakness.", black_after_first: "Most", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "My", last: "weakness.", end: "mid", black_after_first: "Most", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "My", last: "weakness.", end: "mid", black_after_first: "Most", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.7
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.8
  edition_sensitive: true
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", first: "I", last: "ending,", end: "mid", black_after_first: "saith", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: MISMATCH
- ref: REV.1.11
  edition_sensitive: true
  source_expected: {spans: 2, start: "mid", end: "verse_end", span1_first: "I", span1_last: "last:", black_between: "and,", span2_first: "What", span2_last: "Laodicea.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 2, start: "mid", span1_first: "I", span1_last: "last:", black_between: "and,", span2_first: "What", span2_last: "Laodicea.", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 2, start: "mid", span1_first: "I", span1_last: "last:", black_between: "and,", span2_first: "What", span2_last: "Laodicea.", end: "verse_end", edition: E1+E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.17
  source_expected: {spans: 1, start: "mid", end: "verse_end", first: "Fear", last: "last:", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Fear", last: "last:", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Fear", last: "last:", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.18
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "I", last: "death.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.19
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Write", last: "hereafter;", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.1.20
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "The", last: "churches.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.2.1
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Unto", last: "candlesticks;", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.3.6
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "He", last: "churches.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.3.20
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "Behold,", last: "me.", q_open_at_start: true, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.3.22
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "He", last: "churches.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.4.1
  edition_sensitive: true
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.7
  source_expected: {spans: 1, start: "verse_start", end: "mid", first: "Behold,", last: "quickly:", black_after_first: "blessed", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", first: "Behold,", last: "quickly:", end: "mid", black_after_first: "blessed", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", first: "Behold,", last: "quickly:", end: "mid", black_after_first: "blessed", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.11
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.12
  source_expected: {spans: 1, start: "verse_start", end: "verse_end", first: "And,", last: "be.", q_open_at_start: false, q_open_at_end: true, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.13
  source_expected: {spans: 1, start: "carried", end: "verse_end", first: "I", last: "last.", q_open_at_start: true, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "carried", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "carried", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.14
  edition_sensitive: true
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.15
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.16
  source_expected: {spans: 1, start: "verse_start", end: "verse_end", first: "I", last: "star.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "verse_start", end: "verse_end", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "verse_start", end: "verse_end", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.17
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: REV.22.20
  source_expected: {spans: 1, start: "mid", end: "mid", first: "Surely", last: "quickly.", black_after_first: "Amen.", q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 1, start: "mid", first: "Surely", last: "quickly.", end: "mid", black_after_first: "Amen.", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 1, start: "mid", first: "Surely", last: "quickly.", end: "mid", black_after_first: "Amen.", edition: E1, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: GEN.1.1
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: PSA.23.1
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, status: derived-by-scholar-from-excerpts, human: pending}
  print_E0: {spans: 0, status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E2, status: verified-scan, human: pending}
  source_vs_E0: match
  source_vs_E1: match
- ref: PSA.22.1
  edition_sensitive: true
  source_expected: {spans: 0, q_open_at_start: false, q_open_at_end: false, basis: "no <q> tag anywhere before Matt.1.1 in kjv.osis.xml (whole-file count, 2026-10-20); not in golden-source-excerpts.md", status: derived-by-scholar-from-source-file, human: pending}
  print_E0: {spans: 1, start: "verse_start", first: "My", last: "me?", end: "mid", black_after_first: "why", status: verified-scan-E0-facsimile, human: pending}
  print_E1: {spans: 0, edition: E2, page: "E2 p.611 (n312)", note: "E1 is NT only; E2 used as for GEN.1.1 and PSA.23.1; viewed 2026-10-20", status: verified-scan, human: pending}
  source_vs_E0: MISMATCH
  source_vs_E1: match
```

## 11. Recommended test design (for the developer and the owner)

1. **CI asserts `source_expected`, after a person confirms it.** For each row, a person opens the
   excerpt in `golden-source-excerpts.md` and checks the `source_expected` values against the
   reading-aid block (about 100 rows, a few seconds each). Then they change `human: pending` to
   `human: confirmed <initials> <date>`. CI then asserts that `content.sqlite` matches
   `source_expected`. Because those values come from outside the pipeline, the test catches any
   pipeline bug that drops, adds, shifts or trims a span.
2. **Keep the print columns non-blocking.** Report `source_vs_E0` and `source_vs_E1` in a
   non-blocking "edition differences" report. Record the two findings below as properties of the
   source edition in `SOURCES.md`. Don't "fix" them (PLAN.md 2.4 and 2.8):
   - Rev 1:8: the source (like E0) has no red; E1 and E2 red the first clause.
   - Old Testament: the source has no red; E0 reds Old Testament passages such as Ps 22:1.
3. **Compare words, not characters, against print.** Printed boundary punctuation isn't reliable to
   read (section 3). Exact character ranges are already enforced by the round-trip test against the
   source markup (PLAN.md 2.8).
4. **Test the `<q>` structure separately.** Use `q_open_at_start` and `q_open_at_end`, and watch Rev
   1:20 → 2:1 (section 9, note a). A span count over `<q>` elements gives a different answer from a
   count of visible red runs across that chapter break.
5. **Add PSA.22.1 to the excerpt list**, so its `source_expected` can be confirmed from an excerpt
   like the other rows.
6. **Red letters only.** These rows test red-letter boundaries. Verse text tests use the pinned file.
7. **Before CI treats any print value as golden**, a person spot-checks it against the cited page.
   Any disagreement is resolved by looking at the page again, never by consulting the digital source.

## 12. Citations for `SOURCES.md`

- **E0.** *The Holy Bible: Red Letter Edition. Containing the Old and New Testaments …* New York: The
  Christian Herald; Louis Klopsch, Proprietor, Bible House, 1901. "Copyright, 1901, by Louis
  Klopsch." The Christian Herald Press, M. N. Cormack, Supt. **Viewed only in a modern facsimile:**
  John DiCesare, *Rubricated American Heritage Bible eBook* (Museum and Library of the Bible Press,
  2023-04-15), https://archive.org/details/rubricated-american-heritage-bible-e-book (accessed
  2026-10-20). Only the reproduced 1901 pages were used. Nothing was copied from the 2023 preface
  or other new matter.
- **E1.** *The Oxford Self-Pronouncing Testament. The New Testament of Our Lord and Saviour Jesus Christ … with the words of our Lord printed in red.* Oxford: University Press; London: Humphrey Milford; New York: 35 West 32nd Street. Printer: Horace Hart. Undated (inferred c. 1913–1915; to confirm). Copy: University of California Libraries. Scan: https://archive.org/details/oxfordselfpronou00oxforich (accessed 2026-10-19 and 2026-10-20).
- **E2.** *The Holy Bible, Red Letter Edition … Authorized King James Version … with Chain References.* Printed for Concordia Publishing House, St. Louis, at the University Press, Oxford; printer Vivian Ridler. Undated (1958–1978 by the printer's dates; to confirm). Scan: https://archive.org/details/B-001-001-277 (accessed 2026-10-19 and 2026-10-20; title page n7 and verso n8 image-checked 2026-10-20).
- **Source excerpts.** `docs/pipeline/golden-source-excerpts.md` (machine-extracted from BIB-KJV-CW31 `kjv.osis.xml`, commit d490be7, GPL; see SOURCES.md BIB-KJV-CW31).
- Wikipedia, "Red letter edition" (Klopsch, 1899 New Testament, 1901 Bible; NRSVue footnote at John 3:15). https://en.wikipedia.org/wiki/Red_letter_edition (accessed 2026-10-19). The article cites Steve Eng, "The Story Behind: Red Letter Bible Editions," International Society of Bible Collectors.
- Crossway, "The Origins of the Red-Letter Bible," 23 March 2006. https://www.crossway.org/articles/red-letter-origin/ (accessed 2026-10-19).
- Albert Barnes, *Notes on the New Testament*, on Revelation 1:8, read via https://biblehub.com/commentaries/barnes/revelation/1.htm (accessed 2026-10-19). To confirm against a printed edition.
- Wikipedia, "Horace Hart" (Printer to the University 1883–1915); "Humphrey S. Milford" (Publisher to the University 1913–1945); "Vivian Ridler" (Printer to the University 1958–1978). Accessed 2026-10-19. These should be upgraded to a standard reference (for example, the *Oxford Dictionary of National Biography*) before release.

## 13. What a human must still check

1. **Spot-check every row** in section 10 against the cited pages. That's about 30 E1/E2 page views
   and 30 E0 page views. View E0 at 100% or more (section 3). Until then, every row is
   `human: pending`. A shorter first pass (30 page views covering every category, with verified
   links) is in `golden-spot-check-guide.md`.
2. **Confirm `source_expected`** for each row against `golden-source-excerpts.md` (section 11,
   step 1).
3. **Rev 1:8 (the one E0/E1 difference).** Confirm at E0 p.1254 (n1277), E1 p.486 (n487) and E2
   p.1246 (n630). Then decide how `SOURCES.md` records it.
4. **Ps 22:1 and the Old Testament.** Confirm E0 p.620 (n643) (red) and E2 p.611 (n312) (black;
   the scholar closed the E2 reading on 2026-10-20). Decide whether the owner wants a statement such as "Old Testament red in the 1901
   edition is not marked in this source" in `SOURCES.md` (`kjv-label.md` already proposes "New
   Testament" in the provenance line).
5. **The facsimile itself.** If possible, compare a few E0 pages with an **original** 1901 Klopsch
   copy or a library scan (not found yet), and ask CrossWire which printing or format their 2016
   hard copy is. That question could go to CrossWire, if the owner decides to ask.
6. **Carry-overs from the first pass:** E2 date (no date on n7 or n8; title page itself now
   image-checked, 2026-10-20), E1 date (c. 1913–15), the NRSVue
   footnote, and Barnes in print.

## 14. Built database vs golden samples, 2026-10-21

Task D6a. A human-readable confirmation beside the automated test T18. The YAML block in section 10
was **not** changed by this check (only this section was added), so T18's input is the same.

**What was compared.**

- **Database side:** `docs/pipeline/golden-db-red-dump.tsv` (generated by
  `pipeline/examples/golden_red_dump.rs` from `kjv.sqlite` SHA-256 `92cae5e0…264e27`, equal to
  `pipeline/tests/expected/output-sha256.txt`). It lists, for each of the 100 references, every red
  piece with its code-point offsets, `q_ord` and the first and last four stored words, cut by the
  stored offsets. 100 references, 72 verses with red (75 pieces: three verses have two pieces), 28
  verses with no red, 6,349 red code points in all.
- **Golden side:** section 10, all three columns (`source_expected`, `print_E0`, `print_E1`), and the
  machine-extracted `docs/pipeline/golden-source-excerpts.md` (100 excerpts, now including PSA.22.1).
- **Method:** a throwaway script, run from standard input with nothing saved. It parsed the section
  10 YAML and the dump. It made each excerpt's plain verse text by removing `<note>` elements and
  then all tags. For each row it took from the dump the span count, the start (offset 0 or not), the
  end (end offset equal to the verse length or not), the first and last red word, the black words
  between two pieces, and the first black word after a piece that ends mid-verse. These were read
  from the excerpt text **at the database's offsets**. Against `source_expected` the words had to be
  identical, punctuation included. Against the print columns they were compared as words (case and
  punctuation folded, ’ = '), as section 10 says.

**Results.**

| Check | Agree | Disagree |
|---|---|---|
| Database vs `source_expected` (span count, start, end, every boundary word, `black_between`, `black_after_first`) | **100 / 100** | none |
| Database vs `print_E0` (E0, Klopsch 1901 facsimile) | **99 / 100** | PSA.22.1 (known, below) |
| Database vs `print_E1` (E1; E2 for GEN.1.1, PSA.23.1, PSA.22.1) | **99 / 100** | REV.1.8 (known, below) |
| Stored verse length equals the excerpt's plain-text length (code points) | 100 / 100 | none |
| Each piece's first and last four stored words equal the excerpt text at the same offsets | 75 / 75 pieces | none |
| Order of references in the dump equals the order in section 10 | yes | none |

So the database's red/black boundary **words** are the ones a reader sees in the printed editions at
every golden reference, except the two known edition differences. No new disagreement was found.

**The two known print-vs-source differences, as the database has them.**

1. **Rev 1:8.** The dump has one line with `-` (no red; verse length 128). That is exactly the
   source (no `<q>` in the excerpt) and E0 (all of 1:8 black). E1 and E2 print the first clause red
   ("I … ending,", then black from "saith"). The database follows the source, as PLAN.md 2.8
   requires. Recorded as a property of the source, not an error.
2. **Ps 22:1.** The dump has one line with `-` (no red; verse length 113). That is exactly the
   source: the excerpt now in `golden-source-excerpts.md` has 0 `<q>` start tags and 0 end tags,
   which confirms the whole-file count recorded in the row's `basis` field. E2 also prints 22:1
   black. E0 prints "My … me?" red. The database follows the source. Section 11, step 5 ("add
   PSA.22.1 to the excerpt list") is now done. The `basis` text in the YAML row still says "not in
   golden-source-excerpts.md". I left it unchanged so T18's input stays the same. It can be updated
   with the next deliberate edit of the YAML block (T18 ignores `basis`).

**Structure notes (observations only, no action needed).**

- Every row marked `carried` starts at offset 0. Its `q_ord` equals that of the previous verse's
  red piece where that verse is in the set (for example JHN.3.6–3.8 = 477, JHN.3.11–3.21 = 478,
  MAT.26.28 = 129, ACT.1.5 = 617, REV.22.13 = 646). So the database keeps one `<q>` element across
  those verse breaks, as the source does.
- **Rev 1:20 → 2:1:** colour carries across the chapter break (both verses fully red), but the
  `q_ord` changes from 642 to 643. That matches section 9, note a: the source closes one `<q>` at the
  end of 1:20 and opens a new one at 2:1. A count of `<q>` elements and a count of visible red runs
  give different answers here, as section 11, step 4 says.
- **Matt 22:21:** the stored red piece begins "Render therefore unto Cesar". That is the source's
  spelling (see `kjv-label.md`, fact 7). E0 prints "Cæsar". Word comparison against print is about
  the boundary words ("Render" … "God’s"), which agree. The spelling difference is a known property
  of the source, not a red-letter finding.

**Housekeeping.** Adding this section changes this file's SHA-256 from `46fd7cc6…6921a7`. The
headers of `golden-db-red-dump.tsv`, `pipeline/tests/golden/red-letter-source.tsv` and
`pipeline/tests/golden/verse-text.tsv` record the old hash. The YAML block they were made from is
unchanged. The developer may regenerate those headers when convenient.

Every row stays `human: pending`. This check is by the scholar agent, from machine-extracted files,
not a person's look at the scans (`golden-spot-check-guide.md`).
