// One search of the command box (wave-2-layout 6.4, 6.7): its true total, the
// results loaded so far (by position in the full ranked list, which the core
// keeps), how many the dropdown shows, and the one last jumped to, so "Show
// 20 more", Ctrl+F (reopen) and F3 / Shift+F3 all work from the same state.
import type { SearchHit, SearchResult } from "./types";

/** Results per page ("Show 20 more"). */
export const PAGE = 20;

/** Asks the core for results `offset .. offset + PAGE` of `query`. */
export type FetchPage = (query: string, offset: number) => Promise<SearchResult>;

export class SearchSession {
  total = 0;
  exactTotal = 0;
  phrase = false;
  /** Set when the index is unavailable: the reason (no results, ever). */
  unavailable: string | null = null;
  /** Results 0 .. shown-1 are loaded and shown in the dropdown. */
  shown = 0;
  /** Position of the result last jumped to, -1 before any. */
  last = -1;
  private byIndex = new Map<number, SearchHit>();

  private constructor(readonly query: string, private fetchPage: FetchPage) {}

  /** Runs `query` and loads its first page. */
  static async start(query: string, fetchPage: FetchPage): Promise<SearchSession> {
    const s = new SearchSession(query, fetchPage);
    await s.load(0);
    s.shown = Math.min(PAGE, s.total);
    return s;
  }

  private async load(offset: number) {
    const r = await this.fetchPage(this.query, offset);
    if (r.status === "unavailable") {
      this.unavailable = r.reason;
      this.total = this.exactTotal = 0;
      return;
    }
    this.total = r.total;
    this.exactTotal = r.exact_total;
    this.phrase = r.phrase;
    r.hits.forEach((h, i) => this.byIndex.set(r.offset + i, h));
  }

  /** The results the dropdown shows, in order. */
  get shownHits(): SearchHit[] {
    const out: SearchHit[] = [];
    for (let i = 0; i < this.shown; i++) {
      const h = this.byIndex.get(i);
      if (h) out.push(h);
    }
    return out;
  }

  get hasMore(): boolean {
    return this.shown < this.total;
  }

  /** "Show 20 more": the next page, appended. */
  async more(): Promise<void> {
    if (!this.hasMore) return;
    if (!this.byIndex.has(this.shown)) await this.load(this.shown);
    this.shown = Math.min(this.shown + PAGE, this.total);
  }

  /** Result `i` of the full list, loading its page if needed. */
  async at(i: number): Promise<SearchHit | null> {
    if (i < 0 || i >= this.total) return null;
    if (!this.byIndex.has(i)) await this.load(Math.floor(i / PAGE) * PAGE);
    // F3 past the shown results loads the next page silently and shows it too
    while (this.shown <= i && this.byIndex.has(this.shown)) this.shown = Math.min(this.shown + PAGE, this.total);
    return this.byIndex.get(i) ?? null;
  }

  /**
   * F3 (+1) / Shift+F3 (-1) from the result last jumped to. Past the true
   * last result F3 wraps to the first; before the first, Shift+F3 wraps to
   * the true last. `null` when there are no results.
   */
  async step(dir: 1 | -1): Promise<{ hit: SearchHit; index: number; wrapped: boolean } | null> {
    if (this.total === 0) return null;
    let i = this.last < 0 ? (dir === 1 ? 0 : this.total - 1) : this.last + dir;
    let wrapped = false;
    if (i >= this.total) { i = 0; wrapped = true; }
    else if (i < 0) { i = this.total - 1; wrapped = true; }
    const hit = await this.at(i);
    if (!hit) return null;
    this.last = i;
    return { hit, index: i, wrapped };
  }
}

/** Whether `text` has enough to search: 2 characters, not counting quote marks or spaces (6.2). */
export function searchable(text: string): boolean {
  return text.replace(/["'“”‘’\s]/g, "").length >= 2;
}

/** A leading quote mark makes the query an exact phrase and the box a search (6.2, 6.3). */
export const startsWithQuote = (text: string) => /^\s*["'“‘]/.test(text);
