// Daily notes: what the app holds between panel openings (wave-2-layout.md 8).
// The documents loaded so far (by day) and the one saver, so closing the
// panel never drops a change waiting to be saved (or retried), and closing
// the app can flush it whether the panel is open or not.
import type { Node as PMNode } from "prosemirror-model";
import { createSaver, type SaveStatus, type Saver, type Timers } from "./notes-save";
import { docJSON, parseDoc } from "./notes-editor";

export interface NotesIO {
  list(before: string | null, limit: number): Promise<{ days: { day: string; doc: string | null }[]; more: boolean }>;
  save(day: string, doc: string): Promise<void>;
  timers?: Timers;
}

/** Days loaded per page (8.1: "30 most recent days load first"). */
export const PAGE = 30;

export class NotesStore {
  /** Each loaded day: its document, or null when it can't be read (shown as such, never edited). */
  readonly docs = new Map<string, PMNode | null>();
  more = false;
  loaded = false;
  error: string | null = null;
  readonly saver: Saver;
  private listeners = new Set<() => void>();
  private statusListeners = new Set<(s: SaveStatus) => void>();
  private oldest: string | null = null;

  constructor(private io: NotesIO) {
    this.saver = createSaver({
      save: io.save,
      timers: io.timers,
      onStatus: (s) => { for (const f of this.statusListeners) f(s); },
    });
  }

  onChange(f: () => void): () => void { this.listeners.add(f); return () => this.listeners.delete(f); }
  onStatus(f: (s: SaveStatus) => void): () => void { this.statusListeners.add(f); return () => this.statusListeners.delete(f); }
  private changed() { for (const f of this.listeners) f(); }

  /** The first page, once. */
  async load(): Promise<void> {
    if (this.loaded) return;
    await this.page(null);
  }

  /** "Show earlier days". */
  loadMore(): Promise<void> {
    return this.page(this.oldest);
  }

  private async page(before: string | null) {
    try {
      const p = await this.io.list(before, PAGE);
      for (const d of p.days) {
        // a day edited in this session keeps the editor's version
        if (!this.docs.has(d.day)) this.docs.set(d.day, parseDoc(d.doc));
        if (this.oldest === null || d.day < this.oldest) this.oldest = d.day;
      }
      this.more = p.more;
      this.error = null;
      this.loaded = true;
    } catch (e) {
      this.error = String(e);
    }
    this.changed();
  }

  /** The editor changed `day`: keep it and save it after the pause. */
  edit(day: string, doc: PMNode) {
    this.docs.set(day, doc);
    this.saver.change(day, () => docJSON(this.docs.get(day) ?? doc));
  }

  /** Days that hold text, plus unreadable ones (shown with a note), newest first. */
  days(): string[] {
    return [...this.docs.entries()]
      .filter(([, d]) => d === null || d.textContent.length > 0)
      .map(([day]) => day)
      .sort((a, b) => (a < b ? 1 : a > b ? -1 : 0));
  }

  flush(): Promise<boolean> { return this.saver.flush(); }
}
