# KJV label metadata (M1-4)

Owner of this file: biblical scholar. Status: **final recommendation, for owner approval**. First draft
2026-10-19; finalised 2026-10-20, after M1-1 pinned the source (BIB-KJV-CW31).

## 1. Recommended wording (final)

| Where | Wording |
|---|---|
| **Translation label** (pickers, headers, About) | **King James Version (1769)** |
| **Text source line** (About / translation info) | Text: CrossWire KJV 3.1, which describes itself as "King James Version (1769)". |
| **Red-letter provenance line** (About / translation info, and next to the red-letter toggle if there is room) | **Words of Jesus in red (New Testament): from the digital source's markup, which its makers matched to Louis Klopsch's 1901 Red Letter Edition.** |
| Optional short form of the red-letter line | Red letters: CrossWire's markup, matched to the 1901 Klopsch edition (New Testament only). |
| Optional history line (About only) | The King James Version was first published in 1611. |

The wording is short and neutral. Every part of it rests on a cited statement (section 2). It
doesn't rank the KJV or any edition.

### Why this wording, and what changed since the draft

1. **"King James Version (1769)" is the source's own name for its text.** The pinned file's OSIS
   header gives the work title "King James Version (1769) with Strongs Numbers and Morphology"
   (`kjv.osis.xml`, `<work osisWork="KJV"><title>`). CrossWire's module page uses the same name
   (fact 6). The label takes the source's words and drops the tagging description. That follows the
   draft's rule: the label shouldn't claim more than the cited source supports.
2. **No "Oxford" or "Blayney" on the label.** The source's title doesn't say either. Its revision
   notes do compare readings with "Blayney's 1769 text" (2023-06-20 entry) and say "Blayney has" for
   Exodus 17:15 and 1 John 2:23 (2023-05 entries). That shows Blayney was a reference for
   corrections. It isn't a statement that the file reproduces Blayney's edition. The draft's
   "standard 1769 Oxford text" therefore stays unused. (If the owner wants "standard 1769 text",
   that wording is still supported by facts 2 and 6. It's a style choice. "King James Version
   (1769)" is closer to the source.)
3. **"1611" moves to an optional history line.** It is true of the translation (fact 1), but the
   file's text is a 1769-based text, and a label reading "1611" next to it can mislead a new reader
   about which printing they are reading.
4. **The red-letter line says "New Testament" and "matched to".** The source's header says
   "Updated Words of Christ (red letter markup) to match Louis Klopsch's 1901 edition" (revisionDesc,
   2016-02-07, resp "DMS"). That supports "matched to Klopsch 1901". Two observed facts mean the line
   must not say the app's red letters simply *are* Klopsch's:
   - Klopsch 1901 also prints Old Testament passages in red. Its title page says so, and Psalm 22:1 is
     printed red (`red-letter-golden-samples.md`, E0 and RL-42). The pinned source has **no**
     words-of-Jesus markup in the Old Testament (0 `<q>` tags before Matthew 1:1; counted
     2026-10-20). So the app's red letters are New Testament only.
   - Agreement was checked at 99 sample verses only, and all 99 agree at the word level (samples
     file, section 9). That is good evidence for the claim. It doesn't prove every verse matches.
   The example wording ("Red letters follow the 1901 Klopsch edition, as marked by the digital
   source") is close to this. I recommend the version in the table, because it names the New
   Testament limit and attributes the match to the source's makers. That's what the evidence
   supports.

## 2. The facts behind the label, with sources

| # | Fact | Source | Confidence |
|---|---|---|---|
| 1 | The translation was first published in **1611**. | David Norton, *A Textual History of the King James Bible* (Cambridge University Press, 2005). Wikipedia, "King James Version", cites Norton throughout. | Settled. **Exact page in Norton to confirm.** |
| 2 | Most current printings follow the **1769 Oxford edition edited by Benjamin Blayney**, which "became the Oxford standard text, and is reproduced almost unchanged in most current printings". | Wikipedia, "King James Version", section on the standard text of 1769, citing Norton 2005 and A. S. Herbert, *Historical Catalogue of Printed Editions of the English Bible 1525–1961* (1968). | Well attested. **Pages and entry numbers to confirm.** |
| 3 | The 1611 and 1769 editions were **not printed red-letter**. Red-letter KJV printing began with Louis Klopsch's *Christian Herald* New Testament (1899) and Bible (1901). | Wikipedia, "Red letter edition"; Crossway, "The Origins of the Red-Letter Bible" (2006). The 1901 date and Klopsch's name are confirmed on the 1901 title page and its verso, viewed in E0 (leaves n24, n25), and in Klopsch's "Explanatory Note" dated "New York City, November, 1901" (p. xvi, leaf n39). | Well attested; title page viewed. |
| 4 | **Pinned source, own title:** "King James Version (1769) with Strongs Numbers and Morphology". | `sources/raw/BIB-KJV-CW31/kjv.osis.xml`, `<header><work osisWork="KJV"><title>` (SHA-256 `4bcbfe45…0e3f5`), read 2026-10-20. | As stated in the file. |
| 5 | **Pinned source, red-letter provenance:** "Updated Words of Christ (red letter markup) to match Louis Klopsch's 1901 edition." | Same file, `<revisionDesc resp="DMS">`, `<date>2016.02.07</date>`. Also recorded in SOURCES.md BIB-KJV-CW31 from the CrossWire wiki. | As stated in the file. |
| 6 | CrossWire's SWORD module page names the module "King James Version (1769) with Strongs Numbers and Morphology and CatchWords", version 3.1 (2023-07-19). | https://www.crosswire.org/sword/modules/ModInfo.jsp?modName=KJV (accessed 2026-10-19) | As stated on the page. |
| 7 | **The source records edits against its 1769 base spelling.** The revision note dated 2023-07-10 reads "All æ change for e." The note dated 2023-07-06 reads, as the bytes now stand, "Change the spelling of 'Cesar' to 'Cesar' (and 'Cesarea' to 'Cesarea')". That is presumably 'Cæsar' to 'Cesar', with the header itself rewritten by the æ edit. **To confirm with CrossWire.** | Same file, header, read 2026-10-20 (the bytes contain no "æ" in that line). | As stated in the file. The parenthetical is my reading, marked "to confirm". |
| 8 | **Printed editions keep "Cæsar".** E0 (Klopsch 1901, facsimile) prints "Cæsar's" at Matthew 22:21 (p. 1006, leaf n1029). | Page image viewed 2026-10-20. | Observed. |

## 3. Consequence for the owner (fact 7 and 8)

The app's KJV will show "Cesar" (and "e" for every "æ"), as the pinned source has it. Printed KJVs,
including the edition the red letters were matched to, show "Cæsar". Under PLAN.md 2.4 and 2.5 the
app shows the source as it is. This is a property of the source, not something for the app to
change. I recommend `SOURCES.md` BIB-KJV-CW31 records it in one line, for example: "Spelling note: the
source replaces 'æ' with 'e' throughout (revisionDesc 2023-07-10)." The label stays as above. "1769"
describes the text's base, and the source uses that wording itself.

## 4. To confirm

1. The Norton (2005) pages for facts 1 and 2, and the Herbert (1968) entry numbers for the 1611 and
   1769 editions, checked in the books themselves.
2. Fact 7: the exact original wording of the 2023-07-06 note (CrossWire's git history for
   `kjv.osis.xml` would show it).
3. Whether the owner prefers "King James Version (1769)" (the source's wording, recommended) or "King
   James Version · standard 1769 text" (also supported). Both are accurate. "Oxford" is not
   supported by the source's own title.

## 5. Citations for `SOURCES.md`

- CrossWire KJV 3.1, `kjv.osis.xml`, pinned at commit `d490be7e34762deb2c76cb2c1306d4808e27890d`
  (https://gitlab.com/crosswire-bible-society/kjv/-/raw/d490be7e34762deb2c76cb2c1306d4808e27890d/kjv.osis.xml),
  SHA-256 `4bcbfe45722fca396adbeac9c81025bbb173cb1e30296da90864f0400cb0e3f5`. Header: work title,
  and revisionDesc entries of 2016-02-07, 2023-06-20, 2023-07-06 and 2023-07-10.
- CrossWire, SWORD module KJV, version 3.1 (2023-07-19). https://www.crosswire.org/sword/modules/ModInfo.jsp?modName=KJV (accessed 2026-10-19).
- *The Holy Bible: Red Letter Edition.* New York: The Christian Herald; Louis Klopsch, Proprietor, Bible House, 1901. Viewed in the 2023 facsimile e-book https://archive.org/details/rubricated-american-heritage-bible-e-book, leaves n24 (title page), n25 (verso: "Copyright, 1901, by Louis Klopsch"), n39 (Explanatory Note, p. xvi), n1029 (p. 1006). Accessed 2026-10-20. **A modern facsimile, not an original copy.**
- Norton, David. *A Textual History of the King James Bible.* Cambridge: Cambridge University Press, 2005. (Pages to confirm.)
- Herbert, A. S. *Historical Catalogue of Printed Editions of the English Bible 1525–1961.* London: British and Foreign Bible Society; New York: American Bible Society, 1968. (Entry numbers to confirm. The publisher details are from my own knowledge and must be checked against the book's title page.)
- Wikipedia, "King James Version", section on the standard text of 1769. https://en.wikipedia.org/wiki/King_James_Version (accessed 2026-10-19).
- Wikipedia, "Red letter edition." https://en.wikipedia.org/wiki/Red_letter_edition (accessed 2026-10-19).
- Crossway, "The Origins of the Red-Letter Bible," 23 March 2006. https://www.crossway.org/articles/red-letter-origin/ (accessed 2026-10-19).
