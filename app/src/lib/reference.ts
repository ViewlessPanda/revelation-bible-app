// Command-box reference parser (reader.md 5.2, 5.3). Local, synchronous, no
// fuzzy matching. Book names and abbreviations come from the core (`Book`),
// never from this file.
import type { Book } from "./types";

export interface Ref {
  book: Book;
  chapter: number;
  /** null = top of the chapter */
  verse: number | null;
}

export type Parse =
  | { kind: "empty" }
  | { kind: "books"; books: Book[] }
  | { kind: "refs"; refs: Ref[] }
  | { kind: "error"; line1: string; line2?: string }
  /** Not a reference: searched (wave-2-layout 6.2). */
  | { kind: "text" };

const MAX_ROWS = 6;

export const bookKey = (name: string) => name.toLowerCase().replace(/\s+/g, "");

/** "John 3", or the book name alone for a single-chapter book. */
export function locationLabel(book: Book, chapter: number): string {
  return book.chapters === 1 ? book.name : `${book.name} ${chapter}`;
}

/** The chapter heading (reader.md 3.2): like locationLabel, with the book's heading name. */
export function headingLabel(book: Book, chapter: number): string {
  return book.chapters === 1 ? book.heading : `${book.heading} ${chapter}`;
}

/** "John 3:16", "Jude 5" (single-chapter books show the verse only). */
export function refLabel(r: Ref): string {
  if (r.book.chapters === 1) return r.verse ? `${r.book.name} ${r.verse}` : r.book.name;
  return `${r.book.name} ${r.chapter}${r.verse ? ":" + r.verse : ""}`;
}

// [digit] [letters (words)] [chapter [sep verse]] [- range end, ignored]
const RE =
  /^([1-3])?\s*([a-z]+(?:\s+[a-z]+)*)\.?\s*(?:(\d{1,3})(?:\s*[:.\s]\s*(\d{1,3}))?(?:\s*[-–]\s*\d{1,3}(?:\s*[:.]\s*\d{1,3})?)?)?$/;

/**
 * Books a typed book part can mean, in canonical order. No guessing
 * (reader.md 5.2; docs/scholar/kjv-book-names.md 5.2): an exact full name or
 * listed abbreviation wins; otherwise every book whose name starts with the
 * key (at least `minLen` characters, a typed number counts); otherwise, when
 * no number was typed, every numbered book whose name after the number starts
 * with the letters ("sam" = 1 and 2 Samuel, never just 1).
 */
function candidates(books: Book[], digit: string | undefined, letters: string, minLen: number): Book[] {
  const key = (digit ?? "") + letters;
  const exact = books.filter((b) => bookKey(b.name) === key || b.abbrevs.includes(key));
  if (exact.length) return exact;
  if (key.length < minLen) return [];
  const prefix = books.filter((b) => bookKey(b.name).startsWith(key));
  if (prefix.length || digit !== undefined) return prefix;
  return numberedBase(books, letters);
}

/** Numbered books whose name after the number starts with `letters`. */
const numberedBase = (books: Book[], letters: string) =>
  books.filter((b) => /^[1-3]/.test(bookKey(b.name)) && bookKey(b.name).slice(1).startsWith(letters));

/**
 * The book list while typing (no number yet): the candidates, and after an
 * exact match the other books the letters also start, so `phil` lists
 * Philippians then Philemon and `john` lists John then 1-3 John
 * (kjv-book-names.md 5.2). Enter takes the first row.
 */
function typingBooks(books: Book[], digit: string | undefined, letters: string): Book[] {
  const first = candidates(books, digit, letters, 1);
  const key = (digit ?? "") + letters;
  const exact = first.some((b) => bookKey(b.name) === key || b.abbrevs.includes(key));
  if (!exact) return first;
  const more = books.filter((b) => bookKey(b.name).startsWith(key));
  if (digit === undefined) more.push(...numberedBase(books, letters));
  const out = [...first];
  for (const b of more) if (!out.includes(b)) out.push(b);
  return out;
}

const ORDINALS: Record<string, string> = {
  first: "1", "1st": "1", i: "1", second: "2", "2nd": "2", ii: "2", third: "3", "3rd": "3", iii: "3",
};

/**
 * Numbered-book forms readers type (kjv-book-names.md 5.3): a leading
 * `first`/`1st`/`i`, `second`/`2nd`/`ii`, `third`/`3rd`/`iii` becomes 1, 2, 3,
 * and `.` after a leading numeral is a space, but only when a space or a
 * period follows, so `isa 53` and `is 53` stay Isaiah.
 */
export function normalizeNumber(s: string): string {
  const m = /^(first|1st|second|2nd|third|3rd|iii|ii|i)(?:\.\s*|\s+)(?=\S)/.exec(s);
  if (m) return `${ORDINALS[m[1]]} ${s.slice(m[0].length)}`;
  const d = /^([1-3])\.\s*(?=\S)/.exec(s);
  return d ? `${d[1]} ${s.slice(d[0].length)}` : s;
}

export function parseReference(input: string, books: Book[]): Parse {
  const raw = input.trim();
  if (raw === "") return { kind: "empty" };
  const s = normalizeNumber(raw.toLowerCase());

  // "1 " alone: books starting with that number
  const digitOnly = /^([1-3])$/.exec(s);
  if (digitOnly) {
    const bs = books.filter((b) => bookKey(b.name).startsWith(digitOnly[1]));
    return bs.length ? { kind: "books", books: bs.slice(0, MAX_ROWS) } : noBook(raw);
  }

  const m = RE.exec(s);
  if (!m) return noBook(raw);
  const [, digit, words, chStr, vStr] = m;
  const letters = words.replace(/\s+/g, "");

  if (chStr === undefined) {
    const bs = typingBooks(books, digit, letters);
    return bs.length ? { kind: "books", books: bs.slice(0, MAX_ROWS) } : noBook(raw);
  }

  const found = candidates(books, digit, letters, 2);
  if (!found.length) return noBook(raw);
  const n1 = Number(chStr);
  const n2 = vStr === undefined ? null : Number(vStr);

  const refs: Ref[] = [];
  let firstError: Parse | null = null;
  for (const b of found) {
    let chapter: number;
    let verse: number | null;
    if (b.chapters === 1 && n2 === null) {
      chapter = 1; verse = n1; // "jude 5" = Jude 1:5
    } else {
      chapter = n1; verse = n2;
    }
    if (chapter < 1 || chapter > b.chapters) {
      firstError ??= {
        kind: "error",
        line1: `${b.name} has ${b.chapters} ${b.chapters === 1 ? "chapter" : "chapters"}.`,
      };
      continue;
    }
    const maxV = b.verses[chapter - 1] ?? 0;
    if (verse !== null && (verse < 1 || verse > maxV)) {
      firstError ??= {
        kind: "error",
        line1: `${b.chapters === 1 ? b.name : `${b.name} ${chapter}`} has ${maxV} ${maxV === 1 ? "verse" : "verses"}.`,
      };
      continue;
    }
    refs.push({ book: b, chapter, verse });
  }
  if (refs.length) return { kind: "refs", refs: refs.slice(0, MAX_ROWS) };
  return firstError ?? noBook(raw);
}

// Bidi controls (U+061C, U+200E/F, U+202A-202E, U+2066-2069) the reader typed
// would reorder an echoed message; they are left out of the echo only (D5-F9).
const BIDI = /[\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/g;

/** What the reader typed, safe to echo inside a message. */
export const echo = (raw: string) => raw.replace(BIDI, "");

/** Not a reference: the command box searches it ("No book matches" is retired, wave-2-layout 6.2). */
function noBook(_raw: string): Parse {
  return { kind: "text" };
}
