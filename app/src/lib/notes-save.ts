// Daily notes: quiet autosave (wave-2-layout.md 8.6).
//
// A change marks its day dirty; the save runs `delay` (500ms) after the last
// change, and at once on `flush()` (blur, closing the panel, closing the app).
// Each save sends the whole day's document, read at save time, and the
// backend writes it in one transaction, so a crash loses at most the typing
// since the last pause. Saves run one at a time, in order. A failed save
// keeps the day dirty and retries every `retry` ms (5s); the status says
// "Not saved" until a save succeeds again. "Saving…" shows only when a save
// takes longer than `showAfter` (1000ms).

export type SaveStatus =
  | { kind: "saved" }
  | { kind: "saving" }
  | { kind: "failed"; reason: string };

export interface Timers {
  set(fn: () => void, ms: number): number;
  clear(id: number): void;
}

export interface SaverIO {
  save(day: string, doc: string): Promise<void>;
  onStatus?(s: SaveStatus): void;
  delay?: number;
  showAfter?: number;
  retry?: number;
  timers?: Timers;
}

export interface Saver {
  /** The day's document changed; `read` returns its JSON when the save runs. */
  change(day: string, read: () => string): void;
  /** Save every dirty day now. Resolves true when all of them are saved. */
  flush(): Promise<boolean>;
  /**
   * The editor itself failed (W2A-N1): what's on screen may not be what is
   * saved. The status says "Not saved" from now on and `flush` resolves
   * false (so closing asks), even when later saves succeed.
   */
  broken(reason: string): void;
  readonly status: SaveStatus;
  /** Days with changes not yet saved. */
  readonly dirty: number;
  dispose(): void;
}

const windowTimers: Timers = {
  set: (fn, ms) => window.setTimeout(fn, ms),
  clear: (id) => window.clearTimeout(id),
};

export function createSaver(io: SaverIO): Saver {
  const t = io.timers ?? windowTimers;
  const delay = io.delay ?? 500, showAfter = io.showAfter ?? 1000, retry = io.retry ?? 5000;
  const pending = new Map<string, () => string>();
  let status: SaveStatus = { kind: "saved" };
  let failed = false;
  let brokenReason: string | null = null;
  let timer = 0;
  let slow = 0;
  let chain: Promise<boolean> = Promise.resolve(true);
  let disposed = false;

  const set = (s: SaveStatus) => {
    if (s.kind === status.kind && (s.kind !== "failed" || (status.kind === "failed" && status.reason === s.reason))) return;
    status = s;
    io.onStatus?.(s);
  };
  const schedule = (ms: number) => {
    t.clear(timer);
    timer = t.set(() => { timer = 0; void flush(); }, ms);
  };

  async function runOnce(): Promise<boolean> {
    if (!pending.size) return !failed && !brokenReason;
    const batch = [...pending.entries()];
    pending.clear();
    t.clear(slow);
    slow = t.set(() => { if (!failed && !brokenReason) set({ kind: "saving" }); }, showAfter);
    let ok = true;
    let reason = "";
    for (const [day, read] of batch) {
      try {
        await io.save(day, read());
      } catch (e) {
        ok = false;
        reason = String(e);
        // keep it dirty unless a newer change for that day came in meanwhile
        if (!pending.has(day)) pending.set(day, read);
      }
    }
    t.clear(slow);
    if (ok) {
      failed = false;
      if (pending.size) schedule(delay);
      set(brokenReason ? { kind: "failed", reason: brokenReason } : { kind: "saved" });
    } else {
      failed = true;
      set({ kind: "failed", reason });
      if (!disposed) schedule(retry);
    }
    return ok && !pending.size && !brokenReason;
  }

  function flush(): Promise<boolean> {
    t.clear(timer);
    timer = 0;
    chain = chain.then(runOnce, runOnce);
    return chain;
  }

  return {
    change(day, read) {
      pending.set(day, read);
      if (!failed) schedule(delay); // while failing, the retry timer runs
    },
    flush,
    broken(reason) {
      brokenReason ??= reason;
      set({ kind: "failed", reason: brokenReason });
    },
    get status() { return status; },
    get dirty() { return pending.size; },
    dispose() {
      disposed = true;
      t.clear(timer);
      t.clear(slow);
    },
  };
}
