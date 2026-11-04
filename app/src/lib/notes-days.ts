// Daily notes: days and the reading context (wave-2-layout.md 8.1, 8.2, 8.4).
// Pure: the clock, the locale and the visible verses are passed in.
import type { PassageTag } from "./notes-schema";

/** The local date, `YYYY-MM-DD` (the OS time zone; the note's key). */
export function localDay(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** Local midnight at the start of `day`. */
export function dayDate(day: string): Date {
  const [y, m, d] = day.split("-").map(Number);
  return new Date(y, m - 1, d);
}

/** The day before `day`. */
export function prevDay(day: string): string {
  const d = dayDate(day);
  d.setDate(d.getDate() - 1);
  return localDay(d);
}

/** ms until the next local midnight (when a new "Today" appears). */
export function msToMidnight(now: Date): number {
  const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
  return Math.max(1, next.getTime() - now.getTime());
}

/**
 * The groups shown, newest first (8.1): Today always; every stored day;
 * and the day being edited (a note keeps the day it was started on, so
 * after midnight it stays below the new, empty Today).
 */
export function dayOrder(stored: Iterable<string>, today: string, live: string | null): string[] {
  const set = new Set(stored);
  set.add(today);
  if (live) set.add(live);
  return [...set].sort((a, b) => (a < b ? 1 : a > b ? -1 : 0));
}

/** The date header (8.2): "Today" / "Yesterday" / the weekday, then the full date (no year for this year's older days). */
export function dayHeading(day: string, today: string, locale?: string): { name: string; date: string; full: string } {
  const d = dayDate(day);
  const t = dayDate(today);
  const full = new Intl.DateTimeFormat(locale, { weekday: "long", day: "numeric", month: "long", year: "numeric" }).format(d);
  const noYear = new Intl.DateTimeFormat(locale, { weekday: "long", day: "numeric", month: "long" }).format(d);
  const name = day === today ? "Today"
    : day === prevDay(today) ? "Yesterday"
    : new Intl.DateTimeFormat(locale, { weekday: "long" }).format(d);
  const date = day !== today && d.getFullYear() === t.getFullYear() ? noYear : full;
  return { name, date, full };
}

// ---- the reading context (8.4) --------------------------------------------

export interface ContextInput {
  /** OSIS book ID and chapter shown */
  book: string;
  chapter: number;
  /** verses in the chapter */
  lastVerse: number;
  /** the current verse, if set */
  current: number | null;
  /** verses at least 50% visible in the reading pane, ascending */
  visible: number[];
}

/**
 * The passage being read: the current verse if it is on screen; else the
 * visible verses ("John 3:1–12"); the whole chapter visible -> "John 3".
 */
export function readingContext(i: ContextInput): PassageTag | null {
  if (!i.book || i.chapter < 1) return null;
  if (i.current !== null && i.visible.includes(i.current)) {
    return { book: i.book, chapter: i.chapter, v1: i.current, v2: i.current };
  }
  if (!i.visible.length) {
    return i.current !== null ? { book: i.book, chapter: i.chapter, v1: i.current, v2: i.current } : null;
  }
  const v1 = i.visible[0], v2 = i.visible[i.visible.length - 1];
  if (v1 <= 1 && v2 >= i.lastVerse) return { book: i.book, chapter: i.chapter, v1: 0, v2: 0 };
  return { book: i.book, chapter: i.chapter, v1, v2 };
}

/** Verses at least half visible: `rects` are each verse's top and bottom, `top`/`bottom` the pane's. */
export function halfVisible(rects: { v: number; top: number; bottom: number }[], top: number, bottom: number): number[] {
  const out: number[] = [];
  for (const r of rects) {
    const h = r.bottom - r.top;
    const seen = Math.min(r.bottom, bottom) - Math.max(r.top, top);
    if (h > 0 && seen >= h / 2) out.push(r.v);
  }
  return out;
}
