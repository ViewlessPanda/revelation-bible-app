# KJV short book names and command-box abbreviations (D6a task 2)

Owner of this file: biblical scholar. Status: **recommendation, for owner and developer review**,
2026-10-22. Feeds `app/src-tauri/src/books.rs` (provisional table) and `docs/design/reader.md` 5.2
and Q11. I did not edit either file.

## 1. Summary

- The source file stores only the long main titles (for example the first book's "THE FIRST BOOK
  OF MOSES CALLED GENESIS"). The reader also needs a **short name** (picker, chapter heading such
  as "John 3") and **abbreviations** for the command box.
- **Short names are content** (PLAN Section 4: labels are content, cited, neutral). Every short name
  below rests on a printed KJV edition that I viewed as a page image (section 2).
- **All 66 short names in `books.rs` match the printed short forms in E2's alphabetical table
  (p. 5), word for word.** The only difference is **numeral style**: both editions print roman
  numerals with a period ("I. Samuel", "III. John"); `books.rs` uses arabic ("1 Samuel"). That is a
  typographic choice, not a change of name. It is an **owner decision** (section 4.1).
- The older edition (S1, Oxford 1823) prints longer forms for five books in its table ("The Acts",
  "The Epistle to the Romans", "To the Hebrews", "The Epistle of James", "The Song of Solomon"),
  and its running heads use "S." before the Gospel names and "Solomon's Song". Those are recorded
  as alternatives (section 4.2). Neither changes the recommendation.
- **Abbreviations are UX, not content.** The SBL Handbook forms are given where they exist; the
  rest are marked "UX convention, not content". Two `books.rs` abbreviations collide with another
  book's prefix: `jud` (Judges and Jude) and `phil` (Philippians and Philemon). Section 5 lists all
  ambiguous inputs and how I'd resolve them.
- A machine-readable TSV is in section 6.

## 2. Editions cited

### S1 (primary, public domain, original copy): Oxford quarto, 1823

| Field | Value | Basis |
|---|---|---|
| Title | *The Holy Bible, containing the Old and New Testaments: translated out of the original tongues: and with the former translations diligently compared and revised, by His Majesty's special command. Appointed to be read in Churches.* | Title page, leaf n6 (OCR; title page not re-viewed for this task) |
| Imprint | Oxford: printed at the Clarendon Press, by Samuel Collingwood and Co., Printers to the University. Sold by Edward Gardner, Oxford Bible Warehouse, Paternoster Row, and others. "Pica Quarto. 1823. Cum Privilegio." | Title page OCR; archive.org metadata (publisher, date 1823) |
| Copy | University of Toronto, Robarts Library (call number ABR-4033). Scanned 2007 by the Internet Archive. Rights field: NOT_IN_COPYRIGHT. **An original copy, not a facsimile.** | https://archive.org/metadata/holybiblecontain00oxfouoft, read 2026-10-22 |
| Item | https://archive.org/details/holybiblecontain00oxfouoft | |
| Page link pattern | `https://archive.org/details/holybiblecontain00oxfouoft/page/n{N}/mode/1up`. Image viewed: `https://archive.org/download/holybiblecontain00oxfouoft/page/n{N}.jpg` (full size about 2480 x 3330 px). | |
| Names table | "The Names and Order of all the Books of the Old and New Testament, with the number of their chapters". Unpaginated front matter, **leaf n17** (between the preface "The Translators to the Reader" and Genesis at n18). | **Viewed at full resolution 2026-10-22; all 66 names read from the image** (the OCR has errors in the chapter numbers, e.g. Joshua) |
| Running heads checked | Ecclesiastes n557; Solomon's Song n563; S. Matthew n941; S. John n1033; The Acts n1051; Romans n1075; Hebrews n1132; James n1141; I. Peter n1145. Psalms: book title "The Book of PSALMS" and each psalm headed "PSALM I." etc., n475. | Viewed 2026-10-22 (top strip of each page) |
| Why this edition | A plain Oxford printing of the standard 1769-based text, from the University Press that issued the 1769 edition, in an original library copy. | The text-lineage claim is general knowledge about Oxford printings, **to confirm** if the owner wants it stated anywhere. It isn't needed for book names. |

### E2 (cross-check, the plain modern short forms): Oxford "Red Letter Edition" printed for Concordia

The same edition as E2 in `red-letter-golden-samples.md` section 2 (title page n7 and verso n8
already viewed there; printed by Vivian Ridler, Printer to the University, so 1958 to 1978; exact
date **to confirm**). Item: https://archive.org/details/B-001-001-277 (double-page scan).

| Page | Content | Basis |
|---|---|---|
| p. 4 (left half of **leaf n9**) | "The Names and Order of all the Books of the Old and New Testament with the page number and the number of their chapters" | Viewed at full resolution 2026-10-22 |
| p. 5 (right half of **leaf n9**) | "The Old and New Testament Books arranged in alphabetical order with the page number and the number of their chapters" (New Testament books in italics) | Viewed at full resolution 2026-10-22; all 66 names read |

E2 may still be in copyright as a printing. It is used only as a fact check (which short form an
Oxford KJV prints); nothing from it is copied beyond the book names, which are facts.

### SBL Handbook (abbreviations only)

*The SBL Handbook of Style*, 2nd ed. (Atlanta: SBL Press, 2014), section 8.3 (abbreviations of
biblical books). **I did not see the handbook itself.** The list was read 2026-10-22 from a
library guide that says it follows the latest SBL Handbook and labels the Old Testament list "SBL
8.3": University of Divinity Library, "Style guide: Bible", https://divinity.libguides.com/styleguide/bible.
The exact subsection numbers are **to confirm** against the handbook.

## 3. The 66 books

"S1 table" and "E2 p. 5" give the forms as printed (S1 sets the whole table in capitals; E2 in
upper and lower case). "Recommended" is what I'd show in the picker and chapter heading.

| # | OSIS | Recommended short name | S1 table (1823, n17) | E2 p. 5 (n9) | books.rs | Difference from books.rs |
|---|---|---|---|---|---|---|
| 1 | Gen | Genesis | GENESIS | Genesis | Genesis | none |
| 2 | Exod | Exodus | EXODUS | Exodus | Exodus | none |
| 3 | Lev | Leviticus | LEVITICUS | Leviticus | Leviticus | none |
| 4 | Num | Numbers | NUMBERS | Numbers | Numbers | none |
| 5 | Deut | Deuteronomy | DEUTERONOMY | Deuteronomy | Deuteronomy | none |
| 6 | Josh | Joshua | JOSHUA | Joshua | Joshua | none |
| 7 | Judg | Judges | JUDGES | Judges | Judges | none |
| 8 | Ruth | Ruth | RUTH | Ruth | Ruth | none |
| 9 | 1Sam | 1 Samuel | I. SAMUEL | I. Samuel | 1 Samuel | numeral style (4.1) |
| 10 | 2Sam | 2 Samuel | II. SAMUEL | II. Samuel | 2 Samuel | numeral style |
| 11 | 1Kgs | 1 Kings | I. KINGS | I. Kings | 1 Kings | numeral style |
| 12 | 2Kgs | 2 Kings | II. KINGS | II. Kings | 2 Kings | numeral style |
| 13 | 1Chr | 1 Chronicles | I. CHRONICLES | I. Chronicles | 1 Chronicles | numeral style |
| 14 | 2Chr | 2 Chronicles | II. CHRONICLES | II. Chronicles | 2 Chronicles | numeral style |
| 15 | Ezra | Ezra | EZRA | Ezra | Ezra | none |
| 16 | Neh | Nehemiah | NEHEMIAH | Nehemiah | Nehemiah | none |
| 17 | Esth | Esther | ESTHER | Esther | Esther | none |
| 18 | Job | Job | JOB | Job | Job | none |
| 19 | Ps | Psalms | PSALMS | Psalms | Psalms | none; see 4.3 for the chapter heading |
| 20 | Prov | Proverbs | PROVERBS | Proverbs | Proverbs | none |
| 21 | Eccl | Ecclesiastes | ECCLESIASTES | Ecclesiastes | Ecclesiastes | none |
| 22 | Song | Song of Solomon | THE SONG OF SOLOMON | Song of Solomon | Song of Solomon | none (S1 adds "The"; S1 running head "SOLOMON'S SONG", 4.2) |
| 23 | Isa | Isaiah | ISAIAH | Isaiah | Isaiah | none |
| 24 | Jer | Jeremiah | JEREMIAH | Jeremiah | Jeremiah | none |
| 25 | Lam | Lamentations | LAMENTATIONS | Lamentations | Lamentations | none |
| 26 | Ezek | Ezekiel | EZEKIEL | Ezekiel | Ezekiel | none |
| 27 | Dan | Daniel | DANIEL | Daniel | Daniel | none |
| 28 | Hos | Hosea | HOSEA | Hosea | Hosea | none |
| 29 | Joel | Joel | JOEL | Joel | Joel | none |
| 30 | Amos | Amos | AMOS | Amos | Amos | none |
| 31 | Obad | Obadiah | OBADIAH | Obadiah | Obadiah | none |
| 32 | Jonah | Jonah | JONAH | Jonah | Jonah | none |
| 33 | Mic | Micah | MICAH | Micah | Micah | none |
| 34 | Nah | Nahum | NAHUM | Nahum | Nahum | none |
| 35 | Hab | Habakkuk | HABAKKUK | Habakkuk | Habakkuk | none |
| 36 | Zeph | Zephaniah | ZEPHANIAH | Zephaniah | Zephaniah | none |
| 37 | Hag | Haggai | HAGGAI | Haggai | Haggai | none |
| 38 | Zech | Zechariah | ZECHARIAH | Zechariah | Zechariah | none |
| 39 | Mal | Malachi | MALACHI | Malachi | Malachi | none |
| 40 | Matt | Matthew | MATTHEW | Matthew | Matthew | none (S1 running head "S. MATTHEW", 4.2) |
| 41 | Mark | Mark | MARK | Mark | Mark | none |
| 42 | Luke | Luke | LUKE | Luke | Luke | none |
| 43 | John | John | JOHN | John | John | none (S1 running head "S. JOHN") |
| 44 | Acts | Acts | THE ACTS | Acts | Acts | none (S1 table and running head, and E2 p. 4, print "The Acts", 4.2) |
| 45 | Rom | Romans | THE EPISTLE TO THE ROMANS | Romans | Romans | none (S1 running head "ROMANS") |
| 46 | 1Cor | 1 Corinthians | I. CORINTHIANS | I. Corinthians | 1 Corinthians | numeral style |
| 47 | 2Cor | 2 Corinthians | II. CORINTHIANS | II. Corinthians | 2 Corinthians | numeral style |
| 48 | Gal | Galatians | GALATIANS | Galatians | Galatians | none |
| 49 | Eph | Ephesians | EPHESIANS | Ephesians | Ephesians | none |
| 50 | Phil | Philippians | PHILIPPIANS | Philippians | Philippians | none |
| 51 | Col | Colossians | COLOSSIANS | Colossians | Colossians | none |
| 52 | 1Thess | 1 Thessalonians | I. THESSALONIANS | I. Thessalonians | 1 Thessalonians | numeral style |
| 53 | 2Thess | 2 Thessalonians | II. THESSALONIANS | II. Thessalonians | 2 Thessalonians | numeral style |
| 54 | 1Tim | 1 Timothy | I. TIMOTHY | I. Timothy | 1 Timothy | numeral style |
| 55 | 2Tim | 2 Timothy | II. TIMOTHY | II. Timothy | 2 Timothy | numeral style |
| 56 | Titus | Titus | TITUS | Titus | Titus | none |
| 57 | Phlm | Philemon | PHILEMON | Philemon | Philemon | none |
| 58 | Heb | Hebrews | TO THE HEBREWS | Hebrews | Hebrews | none (S1 running head "HEBREWS") |
| 59 | Jas | James | THE EPISTLE OF JAMES | James | James | none (S1 running head "JAMES") |
| 60 | 1Pet | 1 Peter | I. PETER | I. Peter | 1 Peter | numeral style (S1 running head "I. PETER") |
| 61 | 2Pet | 2 Peter | II. PETER | II. Peter | 2 Peter | numeral style |
| 62 | 1John | 1 John | I. JOHN | I. John | 1 John | numeral style |
| 63 | 2John | 2 John | II. JOHN | II. John | 2 John | numeral style |
| 64 | 3John | 3 John | III. JOHN | III. John | 3 John | numeral style |
| 65 | Jude | Jude | JUDE | Jude | Jude | none |
| 66 | Rev | Revelation | REVELATION | Revelation | Revelation | none |

Order: both tables give the books in the order above, which is also the `books.rs` order. OSIS IDs
are those already in `books.rs` (the OSIS 2.1.1 book names; **to confirm** only if the pipeline
ever meets a different ID, which it would reject).

## 4. Notes on the short names

### 4.1 Numeral style (17 books): owner decision

Both editions print a roman numeral with a period: "I. Samuel", "II. Kings", "III. John". `books.rs`
uses arabic numerals and no period: "1 Samuel". The name is the same; only the numeral style
differs. Options:

- **A (recommended): arabic, as in `books.rs`.** Same name, modern rendering, matches what readers
  type (`1 jn`) and what the SBL list uses ("1–2 Sam"). Record in `SOURCES.md` that the printed
  form is roman and that the app renders the numeral in arabic.
- B: roman with period, as printed ("I. Samuel 3"). Most faithful to the page, but harder to scan in
  a picker and easy to misread ("I." can look like a letter).

Either is defensible. It only needs to be applied the same way everywhere (picker, heading, About).

### 4.2 Longer and alternative printed forms

These are real printed forms, recorded so nobody "corrects" the app later without seeing them. None
is recommended for the picker, because the short form in the same edition family (E2 p. 5) is
printed too.

| OSIS | Alternative printed form | Where |
|---|---|---|
| Acts | The Acts | S1 table (n17), S1 running head (n1051), E2 p. 4 |
| Rom | The Epistle to the Romans; The Romans | S1 table (n17); E2 p. 4 |
| Heb | To the Hebrews | S1 table (n17), E2 p. 4 |
| Jas | The Epistle of James; Epistle of James | S1 table (n17); E2 p. 4 |
| Song | The Song of Solomon; Solomon's Song | S1 table (n17); S1 running head (n563) |
| Matt, John (and probably Mark, Luke) | S. Matthew, S. John | S1 running heads (n941, n1033). Luke's running head reads "S. LUKE" in the OCR (n993), not viewed; Mark **not checked, to confirm**. |

The S1 running heads also show that Oxford's own compact forms for Romans, Hebrews and James are the
same as the recommended short names.

### 4.3 Psalms: book name vs chapter heading (design question)

The book is "Psalms" in both tables, but each chapter is headed "PSALM I.", "PSALM II." and so on in
S1 (leaf n475, viewed). So "Psalm 23" is the printed form for a single chapter, and "Psalms" is the
name of the book. reader.md builds the heading from book name + chapter, which would give
"Psalms 23". Suggestion for the developer and designer: a per-book chapter label, used only for the
heading (`Psalm` for Ps, the short name for every other book). E2's psalm headings were **not
checked (to confirm)**, but S1 is enough to cite.

### 4.4 Case

S1 sets the table in capitals and E2 in upper and lower case. Showing "Genesis" rather than
"GENESIS" is a rendering choice, not a change of name.

## 5. Command-box abbreviations (UX, not content)

### 5.1 What the SBL Handbook gives

SBL forms (section 2): Gen, Exod, Lev, Num, Deut, Josh, Judg, Ruth, 1–2 Sam, 1–2 Kgs, 1–2 Chr,
Ezra, Neh, Esth, Job, Ps/Pss, Prov, Eccl (or Qoh), Song (or Cant), Isa, Jer, Lam, Ezek, Dan, Hos,
Joel, Amos, Obad, Jonah, Mic, Nah, Hab, Zeph, Hag, Zech, Mal; Matt, Mark, Luke, John, Acts, Rom,
1–2 Cor, Gal, Eph, Phil, Col, 1–2 Thess, 1–2 Tim, Titus, Phlm, Heb, Jas, 1–2 Pet, 1–2–3 John,
Jude, Rev.

Every SBL form is already in `books.rs`, either as a listed abbreviation or because it equals the
book's full name (Ruth, Ezra, Job, Joel, Amos, Jonah, Mark, Luke, John, Acts, Titus, Jude), **except
`cant`** (SBL's alternative for Song of Solomon). I suggest adding `cant`. The SBL forms equal the
OSIS IDs for every book.

All other abbreviations in `books.rs` (for example `jn`, `mt`, `ge`, `sos`, `jsh`, `nb`) are **UX
convention, not content**. They're common in Bible software and study Bibles; I have no single
citation for them and none is needed.

### 5.2 Ambiguous inputs

reader.md 5.2 accepts a full name, a listed abbreviation, or a **unique** prefix of at least two
letters, with no guessing. I checked every two-, three- and four-letter prefix of the 66 names
(lower case, spaces removed) and every `books.rs` abbreviation against the other books' names.

| Typed | Matches | Recommendation |
|---|---|---|
| `jo` | Joshua, Job, Joel, Jonah, John | Ambiguous. Show the five choices. Don't map `jo` to any book. |
| `ju` | Judges, Jude | Ambiguous. Show both. |
| **`jud`** | Judges, Jude (prefix of both). **`books.rs` lists it for Jude.** | **Remove `jud` from Jude's list** and treat it as ambiguous. Jude keeps `jde` and its full name; Judges keeps `judg`, `jdg`, `jg`, `jdgs`. Neither SBL form is `jud`. |
| `ph`, `phi` | Philippians, Philemon | Ambiguous. Show both. |
| **`phil`** | Prefix of both Philippians and Philemon. **`books.rs` lists it for Philippians.** | **Keep** `phil` → Philippians: it is the SBL abbreviation for Philippians (Philemon is `Phlm`). Keep `philem` for Philemon. If the box shows choices while typing, list Philemon second so it stays one keystroke away. |
| `jn` | John (listed) | Keep. Common, and no book name starts with "jn". |
| `jon` | Jonah (listed); John's forms are `jn`, `jhn`, `joh` | Keep for Jonah. Some readers may mean John; low risk, because John never abbreviates to `jon` in SBL or `books.rs`. |
| `ma` | Malachi, Matthew, Mark | Ambiguous. Show the three. (`mal`, `mat`, `mar` are unique.) |
| `mi` | Micah only | Unique, fine. |
| `ez` | Ezra, Ezekiel | Ambiguous. (`ezr`, `eze` unique.) |
| `ha` | Habakkuk, Haggai | Ambiguous. |
| `ze` | Zephaniah, Zechariah | Ambiguous. |
| `th`, `ti` | `th` matches nothing on its own; `ti` is listed for Titus | Fine, as long as the parser doesn't strip the number from numbered books (see 5.3). |
| `1c`, `2c` | 1 Chronicles / 1 Corinthians (and 2) | Ambiguous. (`1ch`, `1co` unique.) |
| `1t`, `2t` | 1 Thessalonians / 1 Timothy (and 2) | Ambiguous. (`1th`, `1ti` unique.) |
| `sam`, `samuel`, `kings`, `chr`, `chronicles`, `cor`, `corinthians`, `thess`, `thessalonians`, `tim`, `timothy`, `pet`, `peter` (no number) | Both numbered books | Ambiguous. Show 1 and 2. Never default to 1. |
| `john` (no number) | The Gospel (full name, exact match) | Exact full name wins, so John's Gospel. If the box lists choices while typing, also offer 1, 2 and 3 John. |
| `es` | Esther (listed) | Fine for the 66-book canon. If Apocrypha are ever added, `es` and `esd` would need review (Esdras). |
| `pp` | Philippians (listed in `books.rs`) | Works, but `pp` usually means "pages". Optional to drop; UX call. |

### 5.3 Numbered books: forms readers type

Readers type the number many ways: `1 jn`, `1jn`, `1 john`, `i john`, `I. John`, `1st john`,
`first john`, `iii jn`. Suggested normalization before lookup (UX, not content):

1. Lower-case; treat `.` after a leading numeral as a space.
2. Replace a **leading** `first`/`1st`/`i`, `second`/`2nd`/`ii`, `third`/`3rd`/`iii` with `1`, `2`,
   `3`, **only when followed by a space or a period**. This keeps `isa` (Isaiah) and `is 53` from
   being read as `1 sa`. Example: `i sam` → `1sam`, but `isa` stays Isaiah.
3. Remove the space between the number and the book (`1 jn` → `1jn`), then match.

Accepting roman numerals also matches what the printed editions show (section 4.1).

### 5.4 Changes I'd make to the `books.rs` abbreviation lists

- Jude: remove `jud` (ambiguous with Judges).
- Song of Solomon: add `cant` (SBL alternative).
- Everything else: keep as is.

The TSV below already reflects these two changes. With them, all 170 abbreviations are unique and
none is another book's full name (checked 2026-10-22).

## 6. Machine-readable table

Tab-separated, UTF-8, one header row, 66 rows in canonical order. `short_name` uses option A
(arabic numerals; section 4.1). `abbreviations` is comma-separated, lower case, no spaces; full
names always match too and aren't repeated here. Citation keys: **S1** = Oxford, Collingwood, 1823,
archive.org `holybiblecontain00oxfouoft`; **E2** = Oxford for Concordia, 1958–78, archive.org
`B-001-001-277`; `n{N}` = archive.org leaf; "head" = running head.

```tsv
osis	short_name	citation	abbreviations
Gen	Genesis	E2 p.5 (n9); S1 table (n17)	gen,ge,gn
Exod	Exodus	E2 p.5 (n9); S1 table (n17)	exod,exo,ex
Lev	Leviticus	E2 p.5 (n9); S1 table (n17)	lev,le,lv
Num	Numbers	E2 p.5 (n9); S1 table (n17)	num,nu,nm,nb
Deut	Deuteronomy	E2 p.5 (n9); S1 table (n17)	deut,dt,de
Josh	Joshua	E2 p.5 (n9); S1 table (n17)	josh,jos,jsh
Judg	Judges	E2 p.5 (n9); S1 table (n17)	judg,jdg,jg,jdgs
Ruth	Ruth	E2 p.5 (n9); S1 table (n17)	rth,ru
1Sam	1 Samuel	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1sam,1sa,1sm
2Sam	2 Samuel	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2sam,2sa,2sm
1Kgs	1 Kings	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1kgs,1ki,1kg
2Kgs	2 Kings	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2kgs,2ki,2kg
1Chr	1 Chronicles	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1chr,1ch
2Chr	2 Chronicles	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2chr,2ch
Ezra	Ezra	E2 p.5 (n9); S1 table (n17)	ezr
Neh	Nehemiah	E2 p.5 (n9); S1 table (n17)	neh,ne
Esth	Esther	E2 p.5 (n9); S1 table (n17)	esth,est,es
Job	Job	E2 p.5 (n9); S1 table (n17)	jb
Ps	Psalms	E2 p.5 (n9); S1 table (n17); each psalm headed PSALM (S1 n475)	ps,pss,psa,psalm,psm
Prov	Proverbs	E2 p.5 (n9); S1 table (n17)	prov,pro,pr,prv
Eccl	Ecclesiastes	E2 p.5 (n9); S1 table (n17); S1 head (n557)	eccl,qoh,ecc,ec
Song	Song of Solomon	E2 p.4 and p.5 (n9); S1 table: The Song of Solomon (n17); S1 head: SOLOMON'S SONG (n563)	song,cant,sos,sng
Isa	Isaiah	E2 p.5 (n9); S1 table (n17)	isa,is
Jer	Jeremiah	E2 p.5 (n9); S1 table (n17)	jer,je,jr
Lam	Lamentations	E2 p.5 (n9); S1 table (n17)	lam,la
Ezek	Ezekiel	E2 p.5 (n9); S1 table (n17)	ezek,eze,ezk
Dan	Daniel	E2 p.5 (n9); S1 table (n17)	dan,da,dn
Hos	Hosea	E2 p.5 (n9); S1 table (n17)	hos,ho
Joel	Joel	E2 p.5 (n9); S1 table (n17)	jl
Amos	Amos	E2 p.5 (n9); S1 table (n17)	am
Obad	Obadiah	E2 p.5 (n9); S1 table (n17)	obad,ob
Jonah	Jonah	E2 p.5 (n9); S1 table (n17)	jon,jnh
Mic	Micah	E2 p.5 (n9); S1 table (n17)	mic,mc
Nah	Nahum	E2 p.5 (n9); S1 table (n17)	nah,na
Hab	Habakkuk	E2 p.5 (n9); S1 table (n17)	hab,hb
Zeph	Zephaniah	E2 p.5 (n9); S1 table (n17)	zeph,zep,zp
Hag	Haggai	E2 p.5 (n9); S1 table (n17)	hag,hg
Zech	Zechariah	E2 p.5 (n9); S1 table (n17)	zech,zec,zc
Mal	Malachi	E2 p.5 (n9); S1 table (n17)	mal,ml
Matt	Matthew	E2 p.5 (n9); S1 table (n17); S1 head: S. MATTHEW (n941)	matt,mat,mt
Mark	Mark	E2 p.5 (n9); S1 table (n17)	mrk,mk,mr
Luke	Luke	E2 p.5 (n9); S1 table (n17)	luk,lk
John	John	E2 p.5 (n9); S1 table (n17); S1 head: S. JOHN (n1033)	jn,jhn,joh
Acts	Acts	E2 p.5 (n9); E2 p.4, S1 table (n17) and S1 head (n1051) print The Acts	act,ac
Rom	Romans	E2 p.5 (n9); S1 head ROMANS (n1075); S1 table: The Epistle to the Romans	rom,ro,rm
1Cor	1 Corinthians	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1cor,1co
2Cor	2 Corinthians	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2cor,2co
Gal	Galatians	E2 p.5 (n9); S1 table (n17)	gal,ga
Eph	Ephesians	E2 p.5 (n9); S1 table (n17)	eph,ephes
Phil	Philippians	E2 p.5 (n9); S1 table (n17)	phil,php,pp
Col	Colossians	E2 p.5 (n9); S1 table (n17)	col
1Thess	1 Thessalonians	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1thess,1thes,1th
2Thess	2 Thessalonians	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2thess,2thes,2th
1Tim	1 Timothy	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1tim,1ti,1tm
2Tim	2 Timothy	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2tim,2ti,2tm
Titus	Titus	E2 p.5 (n9); S1 table (n17)	tit,ti
Phlm	Philemon	E2 p.5 (n9); S1 table (n17)	phlm,phm,philem
Heb	Hebrews	E2 p.5 (n9); S1 head HEBREWS (n1132); S1 table: To the Hebrews	heb
Jas	James	E2 p.5 (n9); S1 head JAMES (n1141); S1 table: The Epistle of James	jas,jm
1Pet	1 Peter	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention; S1 head I. PETER (n1145)	1pet,1pe,1pt
2Pet	2 Peter	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2pet,2pe,2pt
1John	1 John	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	1jn,1jhn,1jo,1joh
2John	2 John	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	2jn,2jhn,2jo
3John	3 John	E2 p.5 (n9) and S1 table (n17) print roman numeral; arabic is a display convention	3jn,3jhn,3jo
Jude	Jude	E2 p.5 (n9); S1 table (n17)	jde
Rev	Revelation	E2 p.5 (n9); S1 table (n17)	rev,re,rv
```

## 7. To confirm

1. Exact print date of E2 (1958–78 from the printer's dates; carried over from the red-letter work).
2. S1 running heads for Mark (not checked) and Luke (OCR only, "S. LUKE", n993).
3. SBL Handbook subsection numbers (I read the list through a library guide, not the handbook).
4. Psalm headings in E2 (S1 is checked and is enough to cite).
5. Whether the owner prefers option A or B for numerals (4.1), and whether the Psalms heading should
   say "Psalm" (4.3). Both are decisions, not open facts.

## 8. Citations for `SOURCES.md`

- Book short names: *The Holy Bible* (Oxford: Clarendon Press, Samuel Collingwood and Co., 1823),
  "The Names and Order of all the Books", unpaginated front matter. University of Toronto copy,
  https://archive.org/details/holybiblecontain00oxfouoft, leaf n17, viewed 2026-10-22. Cross-checked
  against *The Holy Bible, Red Letter Edition, Authorized King James Version* (Oxford: University
  Press, printed for Concordia Publishing House, n.d. [1958–78]), pp. 4–5,
  https://archive.org/details/B-001-001-277, leaf n9, viewed 2026-10-22.
- Abbreviations: *The SBL Handbook of Style*, 2nd ed. (SBL Press, 2014), section 8.3, as given by
  University of Divinity Library, "Style guide: Bible", read 2026-10-22. Other abbreviations: UX
  convention, not content.
