import { describe, expect, it } from "vitest";
import { headingLabel, normalizeNumber, parseReference, refLabel, locationLabel, type Parse, echo } from "../src/lib/reference";
import type { Book } from "../src/lib/types";
import { kjv } from "./kjv-golden";

// Test data only: a few canonical books with made-up chapter/verse counts, and
// the placeholder fixture's books. (The app gets these from the core.)
const mk = (ord: number, osis: string, name: string, abbrevs: string[], chapters: number, v = 30): Book => ({
  ord, osis, name, heading: name, main_title: null, abbrevs, chapters, verses: Array.from({ length: chapters }, () => v),
});
const BOOKS: Book[] = [
  mk(6, "Josh", "Joshua", ["josh", "jos", "jsh"], 24),
  mk(18, "Job", "Job", ["jb"], 42),
  mk(19, "Ps", "Psalms", ["ps", "psa", "psalm", "pss", "psm"], 150),
  mk(22, "Song", "Song of Solomon", ["song", "sos", "sng"], 8),
  mk(29, "Joel", "Joel", ["jl"], 3),
  mk(32, "Jonah", "Jonah", ["jon", "jnh"], 4),
  mk(43, "John", "John", ["jn", "jhn", "joh"], 21, 36),
  mk(45, "Rom", "Romans", ["rom", "ro", "rm"], 16),
  mk(62, "1John", "1 John", ["1jn", "1jhn", "1jo", "1joh"], 5),
  mk(63, "2John", "2 John", ["2jn", "2jhn", "2jo"], 1, 13),
  mk(65, "Jude", "Jude", ["jde"], 1, 25),
];

const one = (p: Parse) => {
  expect(p.kind).toBe("refs");
  if (p.kind !== "refs") throw new Error();
  expect(p.refs).toHaveLength(1);
  const r = p.refs[0];
  return `${r.book.osis} ${r.chapter}${r.verse ? ":" + r.verse : ""}`;
};

describe("command box parser (reader.md 5.2)", () => {
  it.each([
    ["jn 3 16", "John 3:16"], ["jn 3:16", "John 3:16"], ["John 3.16", "John 3:16"], ["jhn3:16", "John 3:16"],
    ["JN 3 16", "John 3:16"], ["  jn   3  16  ", "John 3:16"],
    ["ps23", "Ps 23"], ["ps 23", "Ps 23"], ["Psalm 23", "Ps 23"], ["Psalms 23", "Ps 23"],
    ["Romans 8", "Rom 8"], ["rom 8", "Rom 8"], ["ro8", "Rom 8"],
    ["1 jn 4", "1John 4"], ["1jn4", "1John 4"], ["1 John 4", "1John 4"],
    ["jude 5", "Jude 1:5"], ["jn 3:16-18", "John 3:16"], ["Song of Solomon 2", "Song 2"],
    ["rom. 8", "Rom 8"], ["joh 1", "John 1"],
  ])("%s -> %s", (input, want) => {
    expect(one(parseReference(input, BOOKS))).toBe(want);
  });

  it("a book alone lists books (Enter goes to chapter 1)", () => {
    const p = parseReference("Romans", BOOKS);
    expect(p.kind === "books" && p.books.map((b) => b.osis)).toEqual(["Rom"]);
    const q = parseReference("1 ", BOOKS);
    expect(q.kind === "books" && q.books.map((b) => b.osis)).toEqual(["1John"]);
    const r = parseReference("jo", BOOKS);
    expect(r.kind === "books" && r.books.map((b) => b.osis)).toEqual(["Josh", "Job", "Joel", "Jonah", "John"]);
  });

  it("ambiguous prefixes list every book with a valid chapter, in canonical order", () => {
    const p = parseReference("jo 3", BOOKS);
    expect(p.kind).toBe("refs");
    if (p.kind === "refs") expect(p.refs.map(refLabel)).toEqual(["Joshua 3", "Job 3", "Joel 3", "Jonah 3", "John 3"]);
    const q = parseReference("jo 30", BOOKS); // only Job has 30 chapters
    expect(one(q)).toBe("Job 30");
  });

  it("errors use numbers from the data and never guess", () => {
    expect(parseReference("xyz 3", BOOKS)).toEqual({ kind: "text" });
    // not a reference: searched (wave-2-layout 6.2); bidi controls are left out of any echo (D5-F9)
    expect(parseReference("x‮yz⁦ 3", BOOKS)).toEqual({ kind: "text" });
    expect(echo("x‮yz⁦ 3")).toBe("xyz 3");
    expect(parseReference("rom 17", BOOKS)).toEqual({ kind: "error", line1: "Romans has 16 chapters." });
    expect(parseReference("jn 3:99", BOOKS)).toEqual({ kind: "error", line1: "John 3 has 36 verses." });
    expect(parseReference("jude 30", BOOKS)).toEqual({ kind: "error", line1: "Jude has 25 verses." });
    expect(parseReference("rom 0", BOOKS).kind).toBe("error");
    expect(parseReference("j 3", BOOKS).kind).toBe("text"); // one letter is not a prefix: searched
    expect(parseReference("jn3:16:2", BOOKS).kind).toBe("text");
    expect(parseReference("rmo 8", BOOKS).kind).toBe("text"); // no spelling correction: searched
    expect(parseReference("", BOOKS)).toEqual({ kind: "empty" });
  });

  it("works on the placeholder fixture's books (names from data)", () => {
    const fx = [mk(1, "Testbook", "Testbook", [], 3, 10), mk(40, "Shortbook", "Shortbook", [], 1, 5)];
    expect(one(parseReference("testbook 2", fx))).toBe("Testbook 2");
    expect(one(parseReference("te 3:4", fx))).toBe("Testbook 3:4");
    expect(one(parseReference("shortbook 5", fx))).toBe("Shortbook 1:5");
    expect(parseReference("testbook 4", fx)).toEqual({ kind: "error", line1: "Testbook has 3 chapters." });
  });

  it("labels", () => {
    expect(locationLabel(BOOKS[6], 3)).toBe("John 3");
    expect(locationLabel(BOOKS[10], 1)).toBe("Jude");
    expect(refLabel({ book: BOOKS[10], chapter: 1, verse: 5 })).toBe("Jude 5");
    expect(refLabel({ book: BOOKS[6], chapter: 3, verse: 16 })).toBe("John 3:16");
  });
});

// The real 66 books as the core sends them (names and abbreviations from the
// scholar's table, chapter/verse counts from the KJV; machine-generated golden).
const KJV = kjv.books;
const osis = (p: Parse) =>
  p.kind === "books" ? p.books.map((b) => b.osis) : p.kind === "refs" ? p.refs.map((r) => `${r.book.osis} ${r.chapter}${r.verse ? ":" + r.verse : ""}`) : p;

describe("ambiguous inputs show every choice, never a guess (kjv-book-names.md 5.2)", () => {
  it.each([
    // typed, books listed while typing (canonical order)
    ["jo", ["Josh", "Job", "Joel", "Jonah", "John"]],
    ["ju", ["Judg", "Jude"]],
    ["jud", ["Judg", "Jude"]], // `jud` was removed from Jude: ambiguous now
    ["ph", ["Phil", "Phlm"]],
    ["phi", ["Phil", "Phlm"]],
    ["phil", ["Phil", "Phlm"]], // SBL `phil` = Philippians first, Philemon one keystroke away
    ["ma", ["Mal", "Matt", "Mark"]],
    ["ez", ["Ezra", "Ezek"]],
    ["ha", ["Hab", "Hag"]],
    ["ze", ["Zeph", "Zech"]],
    ["1c", ["1Chr", "1Cor"]],
    ["2c", ["2Chr", "2Cor"]],
    ["1t", ["1Thess", "1Tim"]],
    ["2t", ["2Thess", "2Tim"]],
    ["sam", ["1Sam", "2Sam"]], ["samuel", ["1Sam", "2Sam"]],
    ["kings", ["1Kgs", "2Kgs"]],
    ["chr", ["1Chr", "2Chr"]], ["chronicles", ["1Chr", "2Chr"]],
    ["cor", ["1Cor", "2Cor"]], ["corinthians", ["1Cor", "2Cor"]],
    ["thess", ["1Thess", "2Thess"]], ["thessalonians", ["1Thess", "2Thess"]],
    ["th", ["1Thess", "2Thess"]],
    ["tim", ["1Tim", "2Tim"]], ["timothy", ["1Tim", "2Tim"]],
    ["pet", ["1Pet", "2Pet"]], ["peter", ["1Pet", "2Pet"]],
    ["john", ["John", "1John", "2John", "3John"]], // exact name first, then 1-3 John
    ["mi", ["Mic"]],
  ])("%s lists %j", (input, want) => {
    expect(osis(parseReference(input, KJV))).toEqual(want);
  });

  it.each([
    // typed with a chapter: every book with that chapter, in canonical order
    ["jo 3", ["Josh 3", "Job 3", "Joel 3", "Jonah 3", "John 3"]],
    ["ju 5", ["Judg 5", "Jude 1:5"]],
    ["jud 5", ["Judg 5", "Jude 1:5"]],
    ["ph 1", ["Phil 1", "Phlm 1:1"]],
    ["phi 2", ["Phil 2", "Phlm 1:2"]],
    ["ma 3", ["Mal 3", "Matt 3", "Mark 3"]],
    ["ez 3", ["Ezra 3", "Ezek 3"]],
    ["ha 1", ["Hab 1", "Hag 1"]],
    ["ze 1", ["Zeph 1", "Zech 1"]],
    ["1c 3", ["1Chr 3", "1Cor 3"]],
    ["2c 3", ["2Chr 3", "2Cor 3"]],
    ["1t 3", ["1Thess 3", "1Tim 3"]],
    ["2t 2", ["2Thess 2", "2Tim 2"]],
    ["sam 3", ["1Sam 3", "2Sam 3"]], ["samuel 3", ["1Sam 3", "2Sam 3"]],
    ["kings 2", ["1Kgs 2", "2Kgs 2"]],
    ["chr 5", ["1Chr 5", "2Chr 5"]], ["chronicles 5", ["1Chr 5", "2Chr 5"]],
    ["cor 4", ["1Cor 4", "2Cor 4"]], ["corinthians 4", ["1Cor 4", "2Cor 4"]],
    ["thess 2", ["1Thess 2", "2Thess 2"]], ["thessalonians 2", ["1Thess 2", "2Thess 2"]],
    ["th 2", ["1Thess 2", "2Thess 2"]],
    ["tim 2", ["1Tim 2", "2Tim 2"]], ["timothy 2", ["1Tim 2", "2Tim 2"]],
    ["pet 1", ["1Pet 1", "2Pet 1"]], ["peter 1", ["1Pet 1", "2Pet 1"]],
    ["sam 30", ["1Sam 30"]], // only 1 Samuel has 30 chapters: one row, still not a default
  ])("%s -> %j", (input, want) => {
    expect(osis(parseReference(input, KJV))).toEqual(want);
  });

  it.each([
    // unique or listed: one answer
    ["phil 2", "Phil 2"], ["philem 1", "Phlm 1:1"], ["phlm 1", "Phlm 1:1"],
    ["jn 3", "John 3"], ["jon 2", "Jonah 2"], ["john 3", "John 3"], ["mi 4", "Mic 4"],
    ["ti 2", "Titus 2"], ["es 4", "Esth 4"], ["pp 1", "Phil 1"], ["jde 3", "Jude 1:3"],
    ["cant 2", "Song 2"], ["mal 3", "Mal 3"], ["mat 3", "Matt 3"], ["mar 3", "Mark 3"],
    ["ezr 3", "Ezra 3"], ["eze 3", "Ezek 3"], ["1ch 3", "1Chr 3"], ["1co 3", "1Cor 3"],
    ["1th 3", "1Thess 3"], ["1ti 3", "1Tim 3"], ["isa 53", "Isa 53"], ["is 53", "Isa 53"],
  ])("%s -> %s", (input, want) => {
    expect(osis(parseReference(input, KJV))).toEqual([want]);
  });

  it("numbered-book forms readers type (5.3)", () => {
    for (const input of ["1 jn 4", "1jn4", "1 john 4", "i john 4", "I. John 4", "1. john 4", "1st john 4", "first john 4"]) {
      expect(osis(parseReference(input, KJV)), input).toEqual(["1John 4"]);
    }
    expect(osis(parseReference("iii jn 1", KJV))).toEqual(["3John 1:1"]);
    expect(osis(parseReference("ii kings 2", KJV))).toEqual(["2Kgs 2"]);
    expect(osis(parseReference("second samuel 1", KJV))).toEqual(["2Sam 1"]);
    expect(osis(parseReference("i sam 3", KJV))).toEqual(["1Sam 3"]);
    expect(normalizeNumber("isa 53")).toBe("isa 53");
    expect(normalizeNumber("is 53")).toBe("is 53");
    expect(normalizeNumber("i sam 3")).toBe("1 sam 3");
  });

  it("headings use the heading name: the book is Psalms, its chapters are headed Psalm (owner decision)", () => {
    const ps = KJV.find((b) => b.osis === "Ps")!;
    expect(headingLabel(ps, 23)).toBe("Psalm 23");
    expect([ps.name, ps.heading]).toEqual(["Psalms", "Psalm"]);
    expect(locationLabel(ps, 23)).toBe("Psalms 23"); // locations and references keep the book name
    const jude = KJV.find((b) => b.osis === "Jude")!;
    expect(headingLabel(jude, 1)).toBe("Jude");
    expect(KJV.find((b) => b.osis === "1Sam")!.name).toBe("1 Samuel");
  });
});
