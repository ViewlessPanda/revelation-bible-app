// Automatic bookmarks (docs/design/bookmarks.md): the "counts as read" state
// machine (2) on a fake clock, and the relative date phrase (5).
import { describe, expect, it } from "vitest";
import { ReadTracker, bookmarkWords, lastRead } from "../src/lib/bookmarks";

function rig() {
  let t = 0;
  const writes: string[] = [];
  const tr = new ReadTracker({ now: () => t, write: (b, c) => writes.push(`${b} ${c}`) });
  /** advance the clock, ticking every 250ms as the app does */
  const wait = (ms: number) => { for (let i = 0; i < ms; i += 250) { t += Math.min(250, ms - i); tr.tick(); } };
  return { tr, writes, wait };
}

describe("counts as read (bookmarks.md 2)", () => {
  it("rule A: 10 s of attended time qualifies, writes once, and again when the visit ends", () => {
    const { tr, writes, wait } = rig();
    tr.setAttending(true);
    tr.show("Luke", 12, true);
    wait(9_750);
    expect(writes).toEqual([]);
    wait(250);
    expect(writes).toEqual(["Luke 12"]);
    wait(30_000);
    expect(writes).toEqual(["Luke 12"]); // never again on its own clock
    tr.show("Luke", 13, true);
    expect(writes).toEqual(["Luke 12", "Luke 12"]); // the end-of-visit refresh
  });

  it("quick jumps write nothing, however many chapters are passed through", () => {
    const { tr, writes, wait } = rig();
    tr.setAttending(true);
    for (const ch of [23, 24, 25, 26]) { tr.show("Ps", ch, true); wait(1_000); }
    tr.end();
    expect(writes).toEqual([]);
  });

  it("a pause stops the clock without resetting it (6 s + away + 4 s qualifies)", () => {
    const { tr, writes, wait } = rig();
    tr.setAttending(true);
    tr.show("Luke", 13, true);
    wait(6_000);
    tr.setAttending(false); // another app, the command box, the picker, a dialog
    wait(30_000);
    expect(writes).toEqual([]);
    expect(tr.state?.attendedMs).toBe(6_000);
    tr.setAttending(true);
    wait(3_750);
    expect(writes).toEqual([]);
    wait(250);
    expect(writes).toEqual(["Luke 13"]);
  });

  it("the launch restore starts un-armed: untouched it never writes; armed, it counts from then", () => {
    const { tr, writes, wait } = rig();
    tr.setAttending(true);
    tr.show("Luke", 3, false);
    wait(20_000);
    expect(writes).toEqual([]);
    expect(tr.state?.attendedMs).toBe(0);
    tr.arm();
    wait(9_750);
    expect(writes).toEqual([]);
    wait(250);
    expect(writes).toEqual(["Luke 3"]);
  });

  it("the same chapter shown again (a verse jump, a translation switch) carries the visit on", () => {
    const { tr, writes, wait } = rig();
    tr.setAttending(true);
    tr.show("John", 11, true);
    wait(6_000);
    tr.show("John", 11, true);
    wait(4_000);
    expect(writes).toEqual(["John 11"]);
  });

  it("rule B: one pane height of user scrolling, either direction, qualifies at once", () => {
    const { tr, writes } = rig();
    tr.show("Ps", 24, true);
    tr.userScroll(300, 800);
    tr.userScroll(-300, 800);
    expect(writes).toEqual([]);
    tr.userScroll(200, 800);
    expect(writes).toEqual(["Ps 24"]);
  });

  it("rule C: reaching the end by a user scroll qualifies; a user scroll arms an un-armed visit", () => {
    const { tr, writes } = rig();
    tr.show("Ps", 117, false);
    tr.endReached();
    expect(writes).toEqual(["Ps 117"]);
  });

  it("nothing is written when there is no visit", () => {
    const { tr, writes } = rig();
    tr.userScroll(5000, 100);
    tr.endReached();
    tr.end();
    tr.tick();
    expect(writes).toEqual([]);
  });
});

describe("the date phrase (bookmarks.md 5)", () => {
  const at = (y: number, m: number, d: number, h = 12, min = 0) => new Date(y, m - 1, d, h, min).getTime() / 1000;
  const now = new Date(2026, 9, 25, 0, 5); // Sunday 25 October 2026, 00:05 local

  it("compares local calendar dates", () => {
    expect(lastRead(at(2026, 10, 25, 0, 1), now)).toBe("today");
    expect(lastRead(at(2026, 10, 30), now)).toBe("today"); // future: clock changed
    expect(lastRead(at(2026, 10, 24, 23, 50), now)).toBe("yesterday"); // 15 minutes ago
    expect(lastRead(at(2026, 10, 23), now, "en-GB")).toBe("on Friday");
    expect(lastRead(at(2026, 10, 19), now, "en-GB")).toBe("on Monday"); // 6 days
    expect(lastRead(at(2026, 10, 18), now, "en-GB")).toBe("on 18 October"); // 7 days
    expect(lastRead(at(2026, 10, 18), now, "en-US")).toBe("on October 18");
    expect(lastRead(at(2025, 10, 3), now, "en-GB")).toBe("on 3 October 2025");
  });

  it("gives every place its words", () => {
    const w = bookmarkWords("Luke", { book: "Luke", chapter: 12, read_at: at(2026, 10, 24) }, false, now);
    expect(w.bookTip).toBe("Bookmark: chapter 12 · last read yesterday");
    expect(w.chapterTip).toBe("Bookmark · last read yesterday");
    expect(w.rowSub).toBe("Chapter 12 · last read yesterday");
    expect(w.bookSr).toBe(", bookmarked at chapter 12, last read yesterday");
    expect(w.chapterLabel).toBe("Luke 12, bookmarked, last read yesterday");
    const j = bookmarkWords("Jude", { book: "Jude", chapter: 1, read_at: at(2026, 10, 25, 0, 1) }, true, now);
    expect(j.bookTip).toBe("Bookmark · last read today");
    expect(j.bookSr).toBe(", bookmarked, last read today");
  });
});
