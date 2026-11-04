# Commentary label metadata: CMT-MHCC-CW20 (W2-C)

Owner of this file: biblical scholar. Status: **approved by the owner, 2026-10-27** (to-confirm items stay as noted). Written
2026-10-22. Source row: `sources/SOURCES.md` CMT-MHCC-CW20.

No commentary text is quoted or paraphrased in this file. The only quotations are from front
matter, module metadata and reference works, each under 15 words.

## 1. Recommended wording

| Where | Wording |
|---|---|
| **Title** (picker row, panel header) | **Matthew Henry's Concise Commentary** |
| **Author line** (picker row, panel header, under the title) | **Matthew Henry · Presbyterian (English Nonconformist), with Thomas Scott · Anglican** |
| **Date line** | **Abridgement c. 1838 (Religious Tract Society), from Henry's *Exposition* (1708–1710)** · "c. 1838" **to confirm** (§3.3) |
| **About / commentary info** (one sentence, added before the licence attribution in SOURCES.md) | This one-volume commentary was compiled by the Religious Tract Society (London, c. 1838), which said its comments were taken mostly from Matthew Henry and Thomas Scott, with other writers. |
| **Edition line** (About / commentary info) | Text: CrossWire MHCC module, version 2.0 (2021), prepared from the Christian Classics Ethereal Library's text. |
| Short author form (where space is tight, e.g. first-run step) | Matthew Henry, with Thomas Scott |

The wording is short and neutral. It doesn't rank either tradition. Every part rests on a cited
statement in §2–§3.

Approved by the owner, 2026-10-27.

### Why not "Matthew Henry · Presbyterian (English Nonconformist)" alone

1. **The *Concise* text is not Henry's own abridgement.** The file's text matches the Religious
   Tract Society (RTS) volume *Short Comments on Every Chapter of the Holy Bible* (fact 3), and that
   volume's own preface says its comments are, "for the most part", "taken from the works of Henry
   and Scott, and other writers" (fact 2). A label naming Henry alone would claim more than the
   source supports.
2. **The title already gives Henry's name.** "Matthew Henry's Concise Commentary" is the name the
   source uses for itself (fact 5), and the name readers know. Keeping it means the default
   commentary in PLAN §4 ("Matthew Henry") is still easy to find.
3. **Two people, two traditions.** PLAN §4 gives one tradition per author when a commentary has more
   than one (the JFB row). Henry and Scott belonged to different traditions (facts 7–9), so each name
   carries its own tradition. "with" signals that Scott's part is secondary and not separable
   comment by comment. The preface doesn't mark which comments come from whom.
4. **"Other writers" are not named** by the preface, so the label can't name them. The About
   sentence mentions them.

If the owner prefers a shorter author line, the next-best accurate option is
**"Matthew Henry and Thomas Scott (abridged)"** with the traditions shown in the info view only.
I don't recommend "Matthew Henry" alone.

## 2. The facts behind the label, with sources

| # | Fact | Source | Confidence |
|---|---|---|---|
| 1 | The pinned file names no author, compiler or edition. Its header has `<title>MHCC</title>`, `revisionDesc resp="cyrille"`, date 2021.02.14T10.19.58, "Converted from USFM source using u2o.py". | Pinned `mhcc.osis.xml` (commit `b15ff40a`), header read in memory 2026-10-22, not saved. | As stated in the file. |
| 2 | The RTS *Short Comments on Every Chapter of the Holy Bible* says in its preface (p. iii) that its comments are "for the most part" "taken from the works of Henry and Scott, and other writers holding the same principles". The preface names no compiler. The title page credits no person. | Internet Archive `shortcommentson00britgoog` (Google scan of an Oxford University copy, the same copy as Google Books `Fl8EAAAAQAAJ`, which CCEL cites as its page images). Preface read in the OCR text, 2026-10-22. | Read in the printed preface (via OCR). |
| 3 | **The pinned file's text is the RTS *Short Comments* text.** Compared in memory, 2026-10-22: 76.7% of the file's word 6-grams occur in the 19th-century OCR of fact 2. The OCR's own error rate predicts about 75.4% for identical text (95.4% of OCR tokens are clean words; 0.954⁶ = 0.754). The rate is even across all 66 books (0.70–0.82). A scrambled-text control scores 0.0%. Word counts are close (710,058 vs 724,953). | Method and numbers: this file; inputs as in facts 1 and 2. | Strong. It shows the same text, but not proof of word-for-word identity: small edits (e.g. the "roughly 1200 errors corrected" in fact 6) would not show at this level. |
| 4 | CCEL's ThML of the *Concise* gives as its page images Google Books `Fl8EAAAAQAAJ` and `jk5NAAAAcAAJ`. It credits "Henry, Matthew" as author, calls the work an "abridgment of the 6 volume" commentary, and records the electronic source as Logos Research Systems (ThML v1.0, 2001-06-20). | https://www.ccel.org/ccel/h/henry/mhcc.xml, header comments and `<DC>` block, read 2026-10-22. | As stated. CCEL names no compiler either. |
| 5 | CrossWire names the module "Matthew Henry's Concise Commentary on the Whole Bible", `Version=2.0`, `History_2.0` "New build with ccel txt file". Its About text (carried over from the older Bible Foundation e-text) says the text "matches the printed edition as published by Moody Press, 28th printing" (ISBN 0-8024-5190-X). | Pinned `mhcc.conf` at commit `b15ff40a` (https://gitlab.com/crosswire-bible-society/mhcc/-/raw/b15ff40aab9556005fd3d38d5388040a90b99145/mhcc.conf), read 2026-10-22. | As stated. **The Moody claim describes the older e-text.** Version 2.0 was rebuilt from CCEL's text, so whether 2.0 still "matches" Moody is not established. |
| 6 | A 1982 Moody hardcover with ISBN 0-8024-5190-X is listed as "Matthew Henry Concise Commentary on the Whole Bible by Matthew Henry and Thomas Scott". | Retailer listing, https://www.ebay.com/p/8038643221 (search result, 2026-10-22). | **Weak (retailer data). To confirm** from the Moody title page or a library record. Consistent with facts 2–3. |
| 7 | Matthew Henry (1662–1714): "nonconformist divine and commentator"; privately ordained in London, 9 May 1687; ministry at Chester from 2 June 1687, and at Mare Street, Hackney, from 18 May 1712. | *Dictionary of National Biography*, vol. 26 (1891), pp. 123–124, "Henry, Matthew", by Alexander Gordon (signed "A. G."). Read in the Wikisource transcription of the page scans, 2026-10-22. | Standard reference. |
| 8 | Henry was an "English nonconformist divine" who in 1687 "became minister of a Presbyterian congregation at Chester". | *Encyclopædia Britannica*, 11th ed. (1911), vol. 13, "Henry, Matthew". Wikisource transcription (index page 313 of the volume's scan), 2026-10-22. | Standard reference. Printed page number **to confirm**. |
| 9 | Thomas Scott (1747–1821), "commentator on the Bible", was instituted to the rectory of Aston Sandford, Buckinghamshire, on 22 July 1801, and died there in 1821, i.e. a Church of England clergyman. His commentary was written in weekly numbers from 2 January 1788. | *Dictionary of National Biography*, vol. 51, pp. 73–75, "Scott, Thomas (1747–1821)", by Alexander Gordon. Wikisource transcription, 2026-10-22. A Trinity College Cambridge archive authority record, based on the ODNB, heads him "Church of England clergyman and biblical scholar" (https://archives.trin.cam.ac.uk/index.php/scott-thomas-1747-1821-church-of-england-clergyman-and-biblical-scholar; title seen in search results only; the page refused connection). | Standard reference. Volume year (1897) **to confirm**. |
| 10 | Henry's *Exposition*: begun November 1704; first volume 1708; five volumes to the end of the Gospels in 1710; Acts finished before his death; Epistles and Revelation "prepared by thirteen nonconformist divines" after it. DNB also notes an RTS-era combined edition: G. Stokes, 1831–5, joining Henry with Scott's notes. | DNB vol. 26, as fact 7. EB1911 (fact 8) gives "(1708–1710)". | Standard reference. See §3.4 on PLAN's "1706–1721". |

## 3. Findings for the task questions

### 3.1 Who compiled the *Concise* text, and is Scott in it?

- **Compiler:** the Religious Tract Society, London, as corporate author. No individual compiler is
  named in the preface, the scan's title page, CCEL's record or Google Books' record (facts 2, 4).
  Who did the work inside the RTS is **unknown** (to confirm only if a later source names one; not
  needed for the label).
- **Thomas Scott: yes, his material is in it**, by the compilers' own statement (fact 2), and the
  pinned file's text is that RTS text (fact 3). The 1982 Moody listing credits Scott too (fact 6).
  How much of it is Scott's, and which comments, cannot be told from the preface. I did not compare
  the text against Henry's or Scott's full commentaries. That would be a research project, and the
  label doesn't need it.
- Calling the work "Matthew Henry's" is a long-standing publishing name (CCEL, CrossWire, Moody), not
  a statement of sole authorship. The title keeps it because it's the source's own name (fact 5).

### 3.2 Tradition

- **Henry: "Presbyterian (English Nonconformist)" is confirmed.** DNB 1891 calls him a nonconformist
  divine; EB1911 calls him an English nonconformist divine and minister of a Presbyterian
  congregation (facts 7–8). Note: he was born in Flintshire, Wales (DNB). "English Nonconformist"
  names the English dissenting tradition he ministered in (Chester, Hackney), not his birthplace. I
  could not open the ODNB (oxforddnb.com returned 403). Its headline is **to confirm** if the owner
  wants a 21st-century reference as well.
- **Scott: "Anglican"**, the word PLAN §4 already uses for Church of England writers (Wesley,
  Fausset). Supported by DNB (rector of Aston Sandford) and the ODNB-based Trinity record (fact 9).
  "Evangelical" would also be accurate, but it isn't needed and it adds a second label.

### 3.3 Date to show

- **Show "c. 1838" for the *Concise* text, marked to confirm.** Google Books dates the Oxford copy
  (`Fl8EAAAAQAAJ`) 1838 and CCEL relies on it. **But the Internet Archive's record for the same
  copy dates it "[185-?]".** The scan's title page is washed out: only "AND SOLD BY ALL
  BOOKSELLERS." is legible (leaf n9, viewed 2026-10-22), so no date can be read. The colophon names
  the printer, "John Childs and Son, Bungay", which doesn't settle the year. Google Books also lists
  RTS *The Miniature Commentary: Being Short Comments on Every Chapter of the Holy Bible*, dated 1840
  (`jk5NAAAAcAAJ`, `Hk9NAAAAcAAJ`, from the National Library of the Netherlands). I didn't
  compare that text with the file.
- Library catalogues that could settle it (Jisc Library Hub, WorldCat, HathiTrust, Bodleian) were
  blocked to my read-only access today (403s), and the Google Books API was over quota.
- **If it can't be confirmed**, use "1830s–1850s" or drop the year: "Religious Tract Society
  abridgement, 19th century". Either is safe. "c. 1838" is the best-supported single year.
- Keep this date separate from Henry's *Exposition*, which the date line gives as 1708–1710.

### 3.4 Note on PLAN §4's "1706–1721" for Henry's Complete commentary

DNB and EB1911 (fact 10) give **1708** for the first volume and 1708–1710 for Henry's own five
volumes. Neither gives 1706 or 1721. 1721 is probably the year of the posthumous sixth volume
(Epistles and Revelation), but I haven't seen a cited source for it. **To confirm** before the
Complete commentary (or this date line) is extended. For this commentary the date line uses only
what is cited: "(1708–1710)".

### 3.5 Edition statement

CrossWire MHCC module, version 2.0. The OSIS was converted 2021-02-14 (fact 1). The conf says 2.0
was built from CCEL's text file (fact 5), and CCEL's text came from Logos Research Systems (fact 4).
The Moody Press 28th printing (ISBN 0-8024-5190-X) is what the *older* CrossWire e-text claimed to
match (fact 5). It goes in the provenance note, not the label.

## 4. To confirm

1. **The date "c. 1838"**: from a legible title page or a library record of an RTS *Short Comments*
   or *Miniature Commentary* edition (Bodleian / Jisc Library Hub / WorldCat), resolving Google's
   "1838" against the Internet Archive's "[185-?]".
2. **Moody 1982 edition**: confirm "Matthew Henry and Thomas Scott" from its title page or a library
   record (fact 6 is retailer data).
3. ODNB headlines for Henry and Scott, if the owner wants a current reference beside DNB/EB1911.
4. EB1911 vol. 13 printed page number; DNB vol. 51 publication year.
5. PLAN §4's "1706–1721" for the Complete commentary (§3.4).

## 5. Machine-readable metadata (for `content.sqlite` meta)

```yaml
id: CMT-MHCC-CW20
display_name: "Matthew Henry's Concise Commentary"
authors:
  - name: "Matthew Henry"
    tradition: "Presbyterian (English Nonconformist)"
    life: "1662–1714"
    role: primary
  - name: "Thomas Scott"
    tradition: "Anglican"
    life: "1747–1821"
    role: secondary   # displayed as "with Thomas Scott"
author_label: "Matthew Henry · Presbyterian (English Nonconformist), with Thomas Scott · Anglican"
author_label_short: "Matthew Henry, with Thomas Scott"
tradition: "Presbyterian (English Nonconformist); Anglican"
date_label: "Abridgement c. 1838 (Religious Tract Society), from Henry's Exposition (1708–1710)"
date_label_status: to_confirm   # "c. 1838"; fallback "1830s–1850s"
compiler: "Religious Tract Society (London)"
about_note: "This one-volume commentary was compiled by the Religious Tract Society (London, c. 1838), which said its comments were taken mostly from Matthew Henry and Thomas Scott, with other writers."
edition_note: "Text: CrossWire MHCC module, version 2.0 (2021), prepared from the Christian Classics Ethereal Library's text."
provenance_note: "RTS Short Comments on Every Chapter of the Holy Bible (c. 1838) -> Moody Press printing (ISBN 0-8024-5190-X) -> Bible Foundation e-text -> Logos Research Systems e-text -> CCEL ThML/text (2001) -> CrossWire MHCC 2.0 OSIS (2021-02-14)."
citations:
  - "RTS, Short Comments on Every Chapter of the Holy Bible, London: Religious Tract Society, [1838? / 185-?], Preface p. iii. Internet Archive shortcommentson00britgoog (= Google Books Fl8EAAAAQAAJ, Oxford University copy). Accessed 2026-10-22."
  - "In-memory text comparison, pinned mhcc.osis.xml vs the IA OCR above, 2026-10-22 (6-gram hit 0.767; OCR-noise expectation 0.754; control 0.0). docs/scholar/commentary-labels.md fact 3."
  - "CrossWire MHCC mhcc.osis.xml header and mhcc.conf, commit b15ff40aab9556005fd3d38d5388040a90b99145, https://gitlab.com/crosswire-bible-society/mhcc. Accessed 2026-10-22."
  - "CCEL, Matthew Henry's Concise Commentary on the Bible, ThML header, https://www.ccel.org/ccel/h/henry/mhcc.xml. Accessed 2026-10-22."
  - "Dictionary of National Biography, vol. 26 (1891), pp. 123-124, 'Henry, Matthew', by Alexander Gordon. Via Wikisource page scans. Accessed 2026-10-22."
  - "Encyclopaedia Britannica, 11th ed. (1911), vol. 13, 'Henry, Matthew'. Via Wikisource. Accessed 2026-10-22."
  - "Dictionary of National Biography, vol. 51, pp. 73-75, 'Scott, Thomas (1747-1821)', by Alexander Gordon. Via Wikisource page scans. Accessed 2026-10-22."
  - "Retailer listing, Moody 1982, ISBN 0-8024-5190-X, 'by Matthew Henry and Thomas Scott', https://www.ebay.com/p/8038643221 (weak; to confirm)."
status: approved_by_owner
approved_by: "owner"
approved_on: "2026-10-27"
to_confirm:
  - "date c. 1838 (Google Books 1838 vs Internet Archive [185-?])"
  - "Moody 1982 attribution to Henry and Scott"
```

## 6. Citations for `SOURCES.md` (Label metadata citation, CMT-MHCC-CW20)

- *Short Comments on Every Chapter of the Holy Bible.* London: Religious Tract Society, n.d.
  (Google Books dates this copy 1838; the Internet Archive record says "[185-?]"). Printer: John
  Childs and Son, Bungay. Preface, p. iii. Oxford University copy, digitised by Google:
  https://archive.org/details/shortcommentson00britgoog (= https://books.google.com/books?id=Fl8EAAAAQAAJ).
  Accessed 2026-10-22.
- CrossWire Bible Society, MHCC 2.0: `mhcc.osis.xml` header and `mhcc.conf` at commit
  `b15ff40aab9556005fd3d38d5388040a90b99145`. Read 2026-10-22, nothing saved.
- Christian Classics Ethereal Library, `mhcc.xml` (ThML v1.0, 2001-06-20), header. Read 2026-10-22.
- Gordon, Alexander. "Henry, Matthew." *Dictionary of National Biography*, vol. 26. London: Smith,
  Elder, 1891, pp. 123–124.
  https://en.wikisource.org/wiki/Dictionary_of_National_Biography,_1885-1900/Henry,_Matthew
- "Henry, Matthew." *Encyclopædia Britannica*, 11th ed., vol. 13. Cambridge, 1911 (page to
  confirm). https://en.wikisource.org/wiki/1911_Encyclop%C3%A6dia_Britannica/Henry,_Matthew
- Gordon, Alexander. "Scott, Thomas (1747–1821)." *Dictionary of National Biography*, vol. 51
  (year to confirm), pp. 73–75.
  https://en.wikisource.org/wiki/Dictionary_of_National_Biography,_1885-1900/Scott,_Thomas_(1747-1821)
- Moody 1982 listing (weak, to confirm): https://www.ebay.com/p/8038643221
