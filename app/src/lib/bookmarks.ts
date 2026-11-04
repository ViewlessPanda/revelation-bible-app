// Automatic bookmarks (docs/design/bookmarks.md): the relative date phrase,
// the words each place shows, and the "counts as read" state machine (2).
// One bookmark per book, the last chapter actually read; nothing here touches
// verse text.

export interface Bookmark {
  /** OSIS book ID */
  book: string;
  chapter: number;
  /** Unix seconds */
  read_at: number;
}

/** --bookmark-dwell, --bookmark-scroll, --bookmark-undo-hold (tokens.css v0.7, 11). */
export const DWELL_MS = 10_000;
export const SCROLL_SCREENS = 1;
export const UNDO_HOLD_MS = 6000;
/** A scroll counts as the reader's only within this long after an intent (2.4). */
export const INTENT_MS = 250;

/**
 * The date phrase (5): "today", "yesterday", "on Tuesday", "on 3 October",
 * "on 3 October 2025", comparing local calendar dates, never 24-hour windows.
 * A date in the future (clock changed) is "today".
 */
export function lastRead(readAt: number, now: Date = new Date(), locale?: string): string {
  const then = new Date(readAt * 1000);
  const day = (d: Date) => Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()) / 86_400_000;
  const ago = day(now) - day(then);
  if (ago <= 0) return "today";
  if (ago === 1) return "yesterday";
  if (ago <= 6) return `on ${new Intl.DateTimeFormat(locale, { weekday: "long" }).format(then)}`;
  const opts: Intl.DateTimeFormatOptions = then.getFullYear() === now.getFullYear()
    ? { day: "numeric", month: "long" }
    : { day: "numeric", month: "long", year: "numeric" };
  return `on ${new Intl.DateTimeFormat(locale, opts).format(then)}`;
}

/** The words for one bookmark (4, 5, 7). `single`: a one-chapter book (no chapter named). */
export function bookmarkWords(name: string, b: Bookmark, single: boolean, now?: Date) {
  const when = lastRead(b.read_at, now);
  return {
    /** picker book cell tooltip */
    bookTip: single ? `Bookmark · last read ${when}` : `Bookmark: chapter ${b.chapter} · last read ${when}`,
    /** picker chapter cell tooltip */
    chapterTip: `Bookmark · last read ${when}`,
    /** visible end text of a Go to row */
    rowSub: `${single ? "Bookmarked" : `Chapter ${b.chapter}`} · last read ${when}`,
    /** accessible-name suffix after the book name (book cells, Go to rows) */
    bookSr: single ? `, bookmarked, last read ${when}` : `, bookmarked at chapter ${b.chapter}, last read ${when}`,
    /** a chapter cell's full accessible name */
    chapterLabel: `${name} ${b.chapter}, bookmarked, last read ${when}`,
  };
}

export interface TrackerHooks {
  /** performance.now() (monotonic) */
  now: () => number;
  /** Upsert this book's bookmark at this chapter, read now (2.5). */
  write: (book: string, chapter: number) => void;
}

interface Visit {
  book: string;
  chapter: number;
  attendedMs: number;
  userScrollPx: number;
  armed: boolean;
  qualified: boolean;
  /** start of the current attended stretch, or null while the clock is stopped */
  lastTick: number | null;
}

/**
 * What counts as read (bookmarks.md 2). The host reports chapter changes,
 * the first trusted input, whether the reader is attending (window visible
 * and focused, no pause condition), user scrolls and the end coming into
 * view. A visit qualifies on 10 s of attended time, one pane height of user
 * scrolling, or reaching the end by a user scroll; it then writes once, and
 * again when it ends.
 */
export class ReadTracker {
  private visit: Visit | null = null;
  private attending = false;

  constructor(private hooks: TrackerHooks, private dwellMs = DWELL_MS, private screens = SCROLL_SCREENS) {}

  /** The visit's state, for tests and debugging. */
  get state(): Readonly<Visit> | null {
    if (this.visit) this.accrue();
    return this.visit;
  }

  /**
   * A chapter is now shown (any cause). The same book and chapter carries the
   * current visit on (a verse jump, a translation switch). `armed: false` for
   * the launch restore only (2.3).
   */
  show(book: string, chapter: number, armed: boolean) {
    const v = this.visit;
    if (v && v.book === book && v.chapter === chapter) return;
    this.end();
    this.visit = { book, chapter, attendedMs: 0, userScrollPx: 0, armed, qualified: false, lastTick: null };
    this.restartClock();
  }

  /** The first trusted keydown, pointerdown or wheel after launch arms the restored visit. */
  arm() {
    const v = this.visit;
    if (!v || v.armed) return;
    v.armed = true;
    this.restartClock();
  }

  /** Window visible and focused, and no pause condition (command box, picker, modal, menus). */
  setAttending(on: boolean) {
    if (on === this.attending) { this.tick(); return; }
    this.accrue();
    this.attending = on;
    this.restartClock();
  }

  /** Called on a timer: accrues attended time and qualifies by dwell. */
  tick() {
    this.accrue();
    const v = this.visit;
    if (v && !v.qualified && v.attendedMs >= this.dwellMs) this.qualify();
  }

  /** A user scroll of `deltaPx` (either direction) in a pane `paneHeight` tall (2.4, rule B). */
  userScroll(deltaPx: number, paneHeight: number) {
    const v = this.visit;
    if (!v) return;
    v.armed = true; // a user scroll arms the visit
    this.restartClock();
    v.userScrollPx += Math.abs(deltaPx);
    if (!v.qualified && paneHeight > 0 && v.userScrollPx >= this.screens * paneHeight) this.qualify();
  }

  /** The end-of-chapter block became at least half visible during a user scroll (rule C). */
  endReached() {
    const v = this.visit;
    if (!v) return;
    v.armed = true;
    if (!v.qualified) this.qualify();
  }

  /** The visit ends (another chapter, app close): a qualified visit refreshes its read_at. */
  end() {
    const v = this.visit;
    if (v?.qualified) this.hooks.write(v.book, v.chapter);
    this.visit = null;
  }

  private qualify() {
    const v = this.visit!;
    v.qualified = true;
    this.hooks.write(v.book, v.chapter);
  }

  private accrue() {
    const v = this.visit;
    if (!v || v.lastTick === null) return;
    const t = this.hooks.now();
    v.attendedMs += Math.max(0, t - v.lastTick);
    v.lastTick = t;
  }

  private restartClock() {
    const v = this.visit;
    if (!v) return;
    if (v.armed && this.attending) { if (v.lastTick === null) v.lastTick = this.hooks.now(); }
    else { this.accrue(); v.lastTick = null; }
  }
}

/** Whether an event came from the reader (tests replace this; synthetic events are untrusted). */
export const trust = { isTrusted: (e: Event) => e.isTrusted };

/** Keys that scroll the reading pane (2.4). */
export const SCROLL_KEYS = new Set([" ", "PageUp", "PageDown", "ArrowUp", "ArrowDown", "Home", "End"]);
