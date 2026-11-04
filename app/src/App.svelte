<script lang="ts">
  // The reader screen (docs/design/reader.md). State, navigation, keyboard,
  // persistence. Verse text rendering lives in ChapterView + lib/verse-dom.
  import { flushSync, onMount, tick } from "svelte";
  import { api } from "./lib/api";
  import { locationLabel, refLabel, type Ref } from "./lib/reference";
  import {
    COLORS, COLOR_NAMES,
    type AppInfo, type Book, type Chapter, type HighlightColor, type Lookup, type Position, type SearchHit, type Settings, type TranslationInfo,
  } from "./lib/types";
  import CommandBox from "./components/CommandBox.svelte";
  import Picker from "./components/Picker.svelte";
  import ReadingOptions from "./components/ReadingOptions.svelte";
  import RightColumn from "./components/RightColumn.svelte";
  import type { CmtView, VerseRef } from "./lib/commentary";
  import ChapterView from "./components/ChapterView.svelte";
  import AboutDialog from "./components/AboutDialog.svelte";
  import { createPanel, type PanelController, type PanelLayout } from "./lib/panel-motion";
  import { createDomPanelHost } from "./lib/panel-dom";
  import { createColumnMotion, type Which } from "./lib/rcol-motion";
  import { RC, clampWidth, restoreSplit, restoreWidthRem, widthBounds } from "./lib/rcol";
  import { checkWrapped } from "./lib/verse-dom";
  import { INTENT_MS, ReadTracker, SCROLL_KEYS, UNDO_HOLD_MS, trust, type Bookmark } from "./lib/bookmarks";
  import PenTray from "./components/PenTray.svelte";
  import { caretOffset, paintInk } from "./lib/ink";
  import { PEN_KEYS, drawnPieces, snapshot as markSnapshot, strokePieces, type Drawn, type MarkIn, type MarkOut, type Piece, type Spot } from "./lib/pen";
  import { attachPen, eraserCursor, inkPieces, penCursor, type PenControl, type Preview, type Tool } from "./lib/pen-mode";
  import NotesPanel from "./components/NotesPanel.svelte";
  import { NotesStore } from "./lib/notes-store";
  import { halfVisible, readingContext } from "./lib/notes-days";
  import { tagText, type PassageTag } from "./lib/notes-schema";

  const STEPS = [16, 17, 18, 19, 20, 22, 24, 26, 28, 32];
  const DEFAULTS: Settings = {
    verse_numbers: true, red_letters: true, supplied_italics: true, text_step: 5, panel_open: false,
    commentary_open: false, right_width_rem: RC.widthDefault, right_split: RC.splitDefault, pen_color: "gold",
    notes_open: false, notes_width_rem: 20,
  };
  /** The notes panel's width limits in rem (wave-2-layout.md 1: --notes-w-min, --notes-w-max). */
  const NOTES_REM = { min: 16, max: 32 } as const;
  const root = document.documentElement;

  let info = $state<AppInfo | null>(null);
  let fatal = $state<{ title: string; detail: string } | null>(null);
  let userWarning = $state<string | null>(null);
  /** user.sqlite was refused at start-up (damaged, foreign, newer, incomplete): offer a new file (D5-F5). */
  let userRefused = $state(false);
  /** W2A-U1: the refused file is from a newer build: no "Start a new data file" */
  let userTooNew = $state(false);
  let userNotice = $state<string | null>(null);
  let translation = $state<TranslationInfo | null>(null);
  let books = $state<Book[]>([]);
  let chapter = $state<Chapter | null>(null);
  let highlights = $state<Record<number, HighlightColor>>({});
  let currentVerse = $state<number | null>(null);
  let settings = $state<Settings>({ ...DEFAULTS });
  let lookup = $state<Lookup>({ kind: "empty" });
  /** Where the right column is in the layout: one drawer for Commentary and Study (lib/panel-motion.ts drives it; wave-2-layout.md 17). */
  let colLayout = $state<PanelLayout>("hidden");
  let col: PanelController | null = null;
  let colMotion: ReturnType<typeof createColumnMotion> | null = null;
  let rightPx = $state(352);
  /** Which panels are in the column's DOM (open, or fading out) and which of those are lifted out of flow while they fade (17.3). */
  let commVis = $state(false);
  let studyVis = $state(false);
  let commLift = $state(false);
  let studyLift = $state(false);
  /** The selected Strong's word (wave-2-layout.md 15): its entry is in the panel. */
  let selectedWord: HTMLElement | null = null;
  let cmtView = $state<CmtView>({ kind: "empty" });
  /** The verse whose comment is shown or being fetched ("book.chapter.verse"). */
  let cmtKey = "";
  let cmtToken = 0;
  let followTimer = 0;
  let menuOpen = $state(false);
  let trOpen = $state(false);
  let optionsOpen = $state(false);
  let pickerOpen = $state(false);
  let scrolled = $state(false);
  let toast = $state<{ text: string; show: boolean }>({ text: "", show: false });
  let live = $state("");
  /** Automatic bookmarks by OSIS book ID (bookmarks.md); the picker and the dropdown read them when they open. */
  let bookmarks = $state<Record<string, Bookmark>>({});
  /** The last cleared bookmark while its Undo toast shows (bookmarks.md 6). */
  let undo = $state<Bookmark | null>(null);
  let undoTimer = 0;
  let cmdFocused = false;
  let windowFocused = typeof document.hasFocus === "function" ? document.hasFocus() : true;
  /** Bookmark writes waiting for the 500ms debounce (as resume, reader.md 12), by book. */
  const bmQueue = new Map<string, number>();
  let bmTimer = 0;
  /** User scroll intent (bookmarks.md 2.4): scrolls count until this time. */
  let intentUntil = -1;
  let lastScrollTop = 0;
  const tracker = new ReadTracker({ now: () => performance.now(), write: bookmarkRead });
  /** Range marks of the shown chapter in this translation (highlights-pen.md 5.3). */
  let marks = $state<MarkOut[]>([]);
  /** Pen mode (3): null off, else the tool in hand. A tool, never saved. */
  let pen = $state<Tool | null>(null);
  let preview = $state<Preview | null>(null);
  /** The tray hint's one-off message (4.1 clamp), once per session. */
  let penHint = $state<string | null>(null);
  let clampShown = false;
  let penCtl: PenControl | null = null;
  /** The tray stays for its exit fade after the pen goes off (3.2). */
  let trayLeaving = $state(false);
  let trayTimer = 0;
  /** Pen writes run one after another, in order (5.3, E5). */
  let penChain: Promise<unknown> = Promise.resolve();
  type HlChange =
    | { kind: "marks"; tr: string; book: string; ch: number; before: MarkIn[]; after: MarkIn[] }
    | { kind: "verse"; book: string; ch: number; v: number; before: HighlightColor | null; after: HighlightColor | null };
  /** Undo / redo of highlight changes, 20 steps, this session (3.1). */
  const past: HlChange[] = [];
  let future: HlChange[] = [];
  /** "Highlight removed · Undo" (4.1) while it shows. */
  let penUndo = $state(false);
  let penUndoTimer = 0;
  /** Daily notes (wave-2-layout.md 8): the left drawer, its documents and saver (kept while the panel is closed). */
  let notesLayout = $state<PanelLayout>("hidden");
  let notesCtl: PanelController | null = null;
  let notesPx = $state(320);
  let notesView = $state<ReturnType<typeof NotesPanel>>();
  let notesBtn = $state<HTMLButtonElement>();
  const notes = new NotesStore({ list: (b, n) => api.notesList(b, n), save: (d, doc) => api.noteSave(d, doc) });
  /** Verses at least half visible in the reading pane (measured on navigation and 300ms after scrolling stops). */
  let visibleVerses = $state<number[]>([]);
  let visibleTimer = 0;
  let closeAsk = $state<HTMLDialogElement>();

  let scroller = $state<HTMLDivElement>();
  let chapterEl = $state<HTMLDivElement>();
  let cmdBox = $state<ReturnType<typeof CommandBox>>();
  let rcolView = $state<ReturnType<typeof RightColumn>>();
  let panelBtn = $state<HTMLButtonElement>();
  let commBtn = $state<HTMLButtonElement>();
  let keysDialog = $state<HTMLDialogElement>();
  let aboutDialog = $state<ReturnType<typeof AboutDialog>>();
  let menuBtn = $state<HTMLButtonElement>();
  let navToken = 0;
  let lookupToken = 0;
  let saveTimer = 0;
  let toastTimer = 0;
  let wheelAcc = 0;
  let ready = false;
  let lastAnchor: Anchor = null;
  let anchorRaf = 0;

  const book = $derived(chapter ? books.find((b) => b.ord === chapter!.book) ?? null : null);
  const location = $derived(book && chapter ? locationLabel(book, chapter.chapter) : "");
  const neighbour = (dir: 1 | -1): { b: Book; ch: number } | null => {
    if (!book || !chapter) return null;
    const i = books.indexOf(book);
    if (dir === 1) {
      if (chapter.chapter < book.chapters) return { b: book, ch: chapter.chapter + 1 };
      return i + 1 < books.length ? { b: books[i + 1], ch: 1 } : null;
    }
    if (chapter.chapter > 1) return { b: book, ch: chapter.chapter - 1 };
    return i > 0 ? { b: books[i - 1], ch: books[i - 1].chapters } : null;
  };
  const prev = $derived(chapter ? neighbour(-1) : null);
  /** The passage being read, for the notes' automatic tags (wave-2-layout.md 8.4). */
  const notesCtx = $derived<PassageTag | null>(book && chapter
    ? readingContext({ book: book.osis, chapter: chapter.chapter, lastVerse: chapter.verses[chapter.verses.length - 1]?.v ?? 1, current: currentVerse, visible: visibleVerses })
    : null);
  const next = $derived(chapter ? neighbour(1) : null);
  const remPx = () => parseFloat(getComputedStyle(root).fontSize) || 16;
  // The column's width limits: 18 to 36 rem, never more than the window less the reading pane's minimum (wave-2-layout.md 17.1).
  const colMin = () => widthBounds(window.innerWidth, remPx()).min;
  const colMax = () => widthBounds(window.innerWidth, remPx()).max;
  const say = (t: string) => { live = ""; requestAnimationFrame(() => (live = t)); };

  // ---- presentation attributes on <html> (reader.md 4, 9) ----
  function applyRoot() {
    root.dataset.verseNumbers = settings.verse_numbers ? "on" : "off";
    root.dataset.redLetters = settings.red_letters ? "on" : "off";
    root.dataset.suppliedItalics = settings.supplied_italics ? "on" : "off";
    root.style.setProperty("--read-size", `var(--read-step-${settings.text_step})`);
  }

  // ---- anchor capture / restore (reader.md 9) ----
  type Anchor = { top0: true } | { v: number; f: number } | null;
  function captureAnchor(): Anchor {
    if (!scroller || !chapterEl) return null;
    if (scroller.scrollTop <= 0) return { top0: true };
    const top = scroller.getBoundingClientRect().top;
    for (const el of chapterEl.querySelectorAll<HTMLElement>(".verse")) {
      const r = el.getBoundingClientRect();
      if (r.bottom > top) return { v: Number(el.dataset.v), f: r.height ? Math.min(1, Math.max(0, (top - r.top) / r.height)) : 0 };
    }
    return null;
  }
  function restoreAnchor(a: Anchor) {
    if (!a || !scroller || !chapterEl) return;
    if ("top0" in a) { scroller.scrollTop = 0; return; }
    const el = chapterEl.querySelector<HTMLElement>(`.verse[data-v="${a.v}"]`);
    if (!el) return;
    const top = scroller.getBoundingClientRect().top, r = el.getBoundingClientRect();
    scroller.scrollTop += r.top - top + a.f * r.height;
  }
  function withAnchor(change: () => void) {
    const a = captureAnchor();
    change();
    restoreAnchor(a);
  }

  // ---- persistence (reader.md 12) ----
  function snapshot() {
    const a = captureAnchor();
    return {
      position: translation && chapter && book
        ? {
            translation: translation.abbrev, book: book.osis, chapter: chapter.chapter,
            anchor_verse: a && "v" in a ? a.v : null, anchor_frac: a && "v" in a ? a.f : null,
            current_verse: currentVerse,
          }
        : null,
      settings: {
        ...settings,
        right_width_rem: Math.round((rightPx / remPx()) * 100) / 100,
        right_split: restoreSplit(settings.right_split),
        notes_width_rem: Math.round((notesPx / remPx()) * 100) / 100,
      },
    };
  }
  function saveNow() {
    clearTimeout(saveTimer);
    if (!ready || userWarning) return;
    api.saveState(snapshot()).catch((e) => (userWarning = String(e)));
  }
  function scheduleSave() {
    clearTimeout(saveTimer);
    saveTimer = window.setTimeout(saveNow, 500);
  }

  // ---- automatic bookmarks (bookmarks.md) ----
  /** A visit counted as read: the bookmark now, the write after 500ms (flushed on close). */
  function bookmarkRead(book: string, chapter: number) {
    bookmarks = { ...bookmarks, [book]: { book, chapter, read_at: Math.floor(Date.now() / 1000) } };
    bmQueue.set(book, chapter);
    clearTimeout(bmTimer);
    bmTimer = window.setTimeout(flushBookmarks, 500);
  }
  function flushBookmarks() {
    clearTimeout(bmTimer);
    if (userWarning) { bmQueue.clear(); return; }
    for (const [book, chapter] of bmQueue) api.bookmarkSet(book, chapter).catch((e) => (userWarning = String(e)));
    bmQueue.clear();
  }
  async function refreshBookmarks() {
    if (userWarning) return;
    try {
      const rows = await api.bookmarksList();
      const next: Record<string, Bookmark> = Object.fromEntries(rows.map((b) => [b.book, b]));
      for (const [book, chapter] of bmQueue) next[book] = bookmarks[book] ?? { book, chapter, read_at: Math.floor(Date.now() / 1000) };
      bookmarks = next;
    } catch (e) { userWarning = String(e); }
  }
  /** The dwell clock runs only while the reader is attending (2.2): visible, focused, nothing pausing it. */
  function attending(): boolean {
    const paused = cmdFocused || pickerOpen || menuOpen || trOpen || !!document.querySelector("dialog[open]");
    return document.visibilityState === "visible" && windowFocused && !paused;
  }
  const updateAttending = () => tracker.setAttending(attending());
  async function clearBookmark(osis: string) {
    const b = books.find((x) => x.osis === osis);
    bmQueue.delete(osis);
    let old: Bookmark | null;
    try { old = await api.bookmarkClear(osis); } catch (e) { userWarning = String(e); return; }
    const next = { ...bookmarks };
    delete next[osis];
    bookmarks = next;
    if (!old) return;
    undo = old;
    clearTimeout(undoTimer);
    undoTimer = window.setTimeout(() => (undo = null), UNDO_HOLD_MS);
    say(`Bookmark cleared for ${b?.name ?? osis}. Ctrl+Z to undo.`);
  }
  async function undoClear() {
    const row = undo;
    if (!row) return;
    undo = null;
    clearTimeout(undoTimer);
    try { await api.bookmarkRestore(row); } catch (e) { userWarning = String(e); return; }
    bookmarks = { ...bookmarks, [row.book]: row };
    const b = books.find((x) => x.osis === row.book);
    say(`Bookmark restored: ${b?.name ?? row.book} ${row.chapter}.`);
  }

  // ---- navigation (reader.md 5.4, 7) ----
  async function goTo(ord: number, ch: number, opts: { verse?: number | null; dir?: number; animate?: boolean; anchor?: Anchor; restore?: boolean } = {}) {
    if (!translation) return;
    const token = ++navToken;
    const b = books.find((x) => x.ord === ord);
    if (!b) return;
    let c: Chapter;
    try {
      c = await api.getChapter(translation.abbrev, ord, ch);
    } catch (e) {
      fatal = { title: "Revelation can't show this chapter.", detail: String(e) };
      return;
    }
    const hl = await api.getHighlights(b.osis, ch).catch((e) => { userWarning = String(e); return []; });
    // range marks show only in their own translation (Q26); a read failure draws none
    const mk = await api.marksForChapter(translation.abbrev, b.osis, ch).catch(() => [] as MarkOut[]);
    if (token !== navToken) return;
    deselect(); // the selection ends with its chapter; the panel keeps its entry (15)
    penCtl?.cancel(); // a stroke never spans chapters (Q31)
    preview = null;
    marks = mk;
    chapter = c;
    // a new visit (bookmarks.md 2.1); the launch restore's starts un-armed (2.3)
    tracker.show(b.osis, ch, !opts.restore);
    highlights = Object.fromEntries(hl.map((h) => [h.verse, h.color]));
    currentVerse = opts.verse ?? null;
    await tick();
    if (!scroller || !chapterEl) return;
    if (opts.verse) {
      const el = chapterEl.querySelector<HTMLElement>(`.verse[data-v="${opts.verse}"]`);
      if (el) scroller.scrollTop += el.getBoundingClientRect().top - scroller.getBoundingClientRect().top - 3 * remPx();
    } else if (opts.anchor) {
      scroller.scrollTop = 0;
      restoreAnchor(opts.anchor);
    } else {
      scroller.scrollTop = 0;
    }
    measureVisible();
    if (opts.animate !== false) {
      chapterEl.classList.remove("entering");
      void chapterEl.offsetWidth;
      chapterEl.style.setProperty("--chapter-dir", String(opts.dir ?? 1));
      chapterEl.classList.add("entering");
    }
    scheduleSave();
  }
  const goRef = (r: Ref) => {
    goTo(r.book.ord, r.chapter, { verse: r.verse, dir: 1 });
    scroller?.focus();
  };

  // ---- jumping to a verse: search results, F3 (wave-2-layout.md 6.8) ----
  /** Show the verse (rendering its chapter if needed), make it current, flash it, announce it. */
  async function jumpTo(hit: Pick<SearchHit, "book" | "chapter" | "verse">, focusPane: boolean) {
    if (!chapter || chapter.book !== hit.book || chapter.chapter !== hit.chapter) {
      await goTo(hit.book, hit.chapter, { verse: hit.verse, dir: 1 });
    } else {
      currentVerse = hit.verse;
      await tick();
      const el = chapterEl?.querySelector<HTMLElement>(`.verse[data-v="${hit.verse}"]`);
      if (el && scroller) scroller.scrollTop += el.getBoundingClientRect().top - scroller.getBoundingClientRect().top - 3 * remPx();
      measureVisible();
      scheduleSave();
    }
    flash(hit.verse);
    if (focusPane) scroller?.focus();
  }
  function flash(v: number) {
    const el = chapterEl?.querySelector<HTMLElement>(`.verse[data-v="${v}"]`);
    if (!el) return;
    el.classList.remove("flashing");
    void el.offsetWidth; // a new jump restarts it
    el.classList.add("flashing");
  }
  /** A result chosen in the command box: focus moves to the reading pane. */
  function onJump(hit: SearchHit) {
    say(hit.reference);
    jumpTo(hit, true);
  }
  /** F3 / Shift+F3 outside the field: focus stays where it was. */
  async function searchStep(dir: 1 | -1) {
    const r = await cmdBox?.searchStep(dir);
    if (!r) { showToast("No search to step through · Ctrl+F"); return; }
    await jumpTo(r.hit, false);
    const where = `${r.hit.reference} · ${r.index + 1} of ${r.total}`;
    const text = r.wrapped ? `${dir === 1 ? "Back to the first result" : "Back to the last result"} · ${where}` : where;
    showToast(text);
    say(text);
  }
  function showToast(text: string) {
    toast = { text, show: true };
    clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => (toast = { ...toast, show: false }), 1000); // --toast-hold
  }
  const step = (dir: 1 | -1) => {
    const n = neighbour(dir);
    if (n) goTo(n.b.ord, n.ch, { dir });
  };

  // ---- toggles, size ----
  function toggle(key: "verse_numbers" | "red_letters" | "supplied_italics") {
    // red letters change colour only, so no anchor restore is needed, but it is harmless
    withAnchor(() => { settings[key] = !settings[key]; applyRoot(); });
    scheduleSave();
  }
  function setStep(n: number) {
    const s = Math.max(1, Math.min(10, n));
    if (s !== settings.text_step) {
      withAnchor(() => { settings.text_step = s; applyRoot(); });
      if (selectedWord) checkWrapped(selectedWord);
      scheduleSave();
    }
    const px = STEPS[s - 1];
    showToast(`Text size ${px} px${s === 10 ? " · largest" : s === 1 ? " · smallest" : ""}`);
    say(`Text size ${px} pixels`);
  }

  // ---- daily notes: the left drawer (wave-2-layout.md 3, 4, 5, 8) ----
  /** The notes panel's width limits in px: 16 to 32 rem, leaving the reading pane its minimum beside an open right column. */
  function notesBounds() {
    const rem = remPx(), min = NOTES_REM.min * rem;
    const right = colLayout === "track" ? rightPx : 0;
    return { min, max: Math.max(min, Math.min(NOTES_REM.max * rem, window.innerWidth - right - RC.readerMin * rem - 8)) };
  }
  const clampNotes = (px: number) => { const b = notesBounds(); return Math.max(b.min, Math.min(b.max, px)); };
  function makeNotes(open: boolean) {
    const host = createDomPanelHost({
      side: "left",
      panel: () => document.getElementById("p-notes"),
      pane: () => scroller ?? null,
      column: () => scroller?.querySelector<HTMLElement>(".reading") ?? null,
      bar: () => ({
        start: document.querySelector<HTMLElement>(".tb-start"),
        centre: document.querySelector<HTMLElement>(".tb-centre"),
        end: document.querySelector<HTMLElement>(".tb-end"),
      }),
      apply: (to) => withAnchor(() => {
        notesLayout = to;
        flushSync();
        if (selectedWord) checkWrapped(selectedWord);
      }),
      anchor: { capture: () => captureAnchor(), restore: (a) => restoreAnchor(a as Anchor) },
    });
    notesCtl = createPanel(host, open);
  }
  const inNotes = (el: Element | null) => !!el && !!document.getElementById("p-notes")?.contains(el);
  async function openNotes(focus = true) {
    if (settings.notes_open) { if (focus) notesView?.focusEditor(); return; }
    settings.notes_open = true;
    measureVisible();
    notesCtl?.open();
    scheduleSave();
    await tick();
    if (focus) notesView?.focusToday();
  }
  function closeNotes(returnTo?: HTMLElement | null) {
    const hadFocus = inNotes(document.activeElement);
    if (!settings.notes_open) return;
    settings.notes_open = false;
    void notes.flush(); // closing the panel saves (8.6)
    notesCtl?.close();
    scheduleSave();
    if (returnTo) returnTo.focus();
    else if (hadFocus) scroller?.focus();
  }
  // Ctrl+Shift+N and the toggle: closed -> open, caret at the end of today's note; open, focus elsewhere -> focus the editor; in Notes -> close (5)
  function toggleNotes(fromButton = false) {
    if (!settings.notes_open) { openNotes(); return; }
    if (fromButton) { closeNotes(notesBtn); return; }
    if (inNotes(document.activeElement)) closeNotes(scroller);
    else notesView?.focusEditor();
  }
  function resizeNotes(px: number, done: boolean) {
    withAnchor(() => { notesPx = clampNotes(px); flushSync(); });
    if (selectedWord) checkWrapped(selectedWord);
    if (done) scheduleSave();
  }
  /** The verses at least half visible (8.4 reading context). */
  function measureVisible() {
    if (!scroller || !chapterEl) return;
    const p = scroller.getBoundingClientRect();
    const rects = [...chapterEl.querySelectorAll<HTMLElement>(".verse")].map((el) => {
      const r = el.getBoundingClientRect();
      return { v: Number(el.dataset.v), top: r.top, bottom: r.bottom };
    });
    const next = halfVisible(rects, p.top, p.bottom);
    if (next.length !== visibleVerses.length || next.some((v, i) => v !== visibleVerses[i])) visibleVerses = next;
  }
  /** A notes tag (8.4 Click, 6.8): jump there, flash the range (at most 20 verses); focus stays in the editor. */
  async function notesJump(t: PassageTag) {
    const b = books.find((x) => x.osis === t.book);
    if (!b || t.chapter > b.chapters) { showToast(`${tagText(t, books)} isn't in this translation`); return; }
    const first = t.v1 || 1;
    say(tagText(t, books));
    await jumpTo({ book: b.ord, chapter: t.chapter, verse: first }, false);
    if (t.v1) for (let v = first + 1; v <= Math.min(t.v2, first + 19); v++) flash(v);
  }
  /** The window's close button (lib.rs holds it once): save the notes, the place and the bookmarks, then close; a failed notes save asks first. */
  async function closeRequested() {
    let ok = true;
    try { ok = await notes.flush(); } catch { ok = false; }
    tracker.end();
    flushBookmarks();
    clearTimeout(saveTimer);
    if (ready && !userWarning) await api.saveState(snapshot()).catch(() => {});
    // W2A-N3: only the notes' own result decides; an unrelated failed write
    // (the place, a bookmark, a highlight) never skips the question
    if (ok) { await api.appClose(false).catch(() => {}); return; }
    await api.appClose(true).catch(() => {});
    if (!closeAsk?.open) closeAsk?.showModal();
  }
  async function closeAnyway() {
    closeAsk?.close();
    await api.appClose(false).catch(() => {});
  }

  // ---- the right column: one drawer for Commentary and Study (wave-2-layout.md 3, 5, 7 and 17) ----
  const other = (w: Which): Which => (w === "comm" ? "study" : "comm");
  const isOpen = (w: Which) => (w === "comm" ? settings.commentary_open : settings.panel_open);
  const setVis = (w: Which, v: boolean) => { if (w === "comm") commVis = v; else studyVis = v; };
  const setLift = (w: Which, v: boolean) => { if (w === "comm") commLift = v; else studyLift = v; };
  const isVis = (w: Which) => (w === "comm" ? commVis : studyVis);

  function makeColumn(open: boolean) {
    const host = createDomPanelHost({
      side: "right",
      panel: () => document.getElementById("rcol"),
      pane: () => scroller ?? null,
      column: () => scroller?.querySelector<HTMLElement>(".reading") ?? null,
      bar: () => ({
        start: document.querySelector<HTMLElement>(".tb-start"),
        centre: document.querySelector<HTMLElement>(".tb-centre"),
        end: document.querySelector<HTMLElement>(".tb-end"),
      }),
      apply: (to) => withAnchor(() => {
        colLayout = to;
        if (to === "hidden") commVis = studyVis = commLift = studyLift = false; // the column is gone with whatever it held
        flushSync();
        rcolView?.reflow();
        if (selectedWord) checkWrapped(selectedWord);
      }),
      anchor: { capture: () => captureAnchor(), restore: (a) => restoreAnchor(a as Anchor) },
    });
    col = createPanel(host, open);
    colMotion = createColumnMotion({
      col: () => rcolView?.element() ?? null,
      panel: (w) => document.getElementById(w === "comm" ? "p-comm" : "p-study"),
      divider: () => document.querySelector<HTMLElement>(".rc-div"),
      done: (w) => { setVis(w, false); setLift(w, false); flushSync(); rcolView?.reflow(); },
    });
  }
  /** The panel `w` was just opened (its flag is already true): slide the column in, or settle it into the open column (17.3). */
  function showPanel(w: Which) {
    if (!col || !colMotion) return;
    if (isOpen(other(w))) {
      // the column is open (or sliding in): finish a slide, then the second panel fades in inside it
      if (col.phase !== "open") col.finishNow();
      colMotion.enter(w, () => { setVis(w, true); flushSync(); rcolView?.reflow(); });
      return;
    }
    colMotion.finishNow();
    // the column was sliding out holding another panel: let that finish, then start afresh
    if (col.phase === "closing" && !isVis(w)) col.finishNow();
    setVis(w, true);
    col.open(); // reverses a slide-out that still holds this panel
  }
  /** The panel `w` was just closed (its flag is already false): the other fills the column, or the last one slides out. */
  function hidePanel(w: Which) {
    if (!col || !colMotion) return;
    const onDivider = !!document.activeElement?.closest(".rc-div");
    if (isOpen(other(w))) {
      if (col.phase !== "open") col.finishNow();
      colMotion.leave(w, () => { setLift(w, true); flushSync(); rcolView?.reflow(); });
      // the divider is gone: focus goes to the remaining panel's close button (17.4)
      if (onDivider) document.querySelector<HTMLElement>(`#${other(w) === "comm" ? "p-comm" : "p-study"} .panel-head button`)?.focus();
      return;
    }
    colMotion.finishNow();
    col.close(); // the column slides out holding the panel as it was
  }

  function inComm(el: Element | null) {
    return !!el && !!document.getElementById("p-comm")?.contains(el);
  }
  /** The column's two separators (the width splitter and the divider). */
  function inColChrome(el: Element | null) {
    return !!el && !!el.closest(".splitter, .rc-div") && !!document.getElementById("rcol")?.contains(el);
  }
  async function openComm(focus = true) {
    if (settings.commentary_open) { if (focus) rcolView?.focusComm(); return; }
    settings.commentary_open = true;
    showPanel("comm");
    scheduleSave();
    scheduleFollow(0);
    await tick();
    if (focus) rcolView?.focusComm();
  }
  function closeComm(returnTo?: HTMLElement | null) {
    const hadFocus = inComm(document.activeElement);
    if (!settings.commentary_open) return;
    settings.commentary_open = false;
    hidePanel("comm");
    scheduleSave();
    if (returnTo) returnTo.focus();
    else if (hadFocus) scroller?.focus();
  }
  // Ctrl+Shift+C and the toggle: closed -> open and focus; open, focus elsewhere -> focus; in the panel -> close (5)
  function toggleComm(fromButton = false) {
    if (!settings.commentary_open) { openComm(); return; }
    if (fromButton) { closeComm(commBtn); return; }
    if (inComm(document.activeElement)) closeComm(scroller);
    else rcolView?.focusComm();
  }
  /** The column's width: the vertical splitter, its keys and double-click (17.1). */
  function resizeCol(px: number, done: boolean) {
    withAnchor(() => { rightPx = clampWidth(px, window.innerWidth, remPx()); notesPx = clampNotes(notesPx); flushSync(); });
    if (selectedWord) checkWrapped(selectedWord);
    if (done) scheduleSave();
  }
  /** The divider moved (17.2): the new share is remembered. */
  function setSplit(split: number) {
    settings.right_split = restoreSplit(split);
    scheduleSave();
  }
  /** The verse the commentary follows: the current verse, else the first verse visible at the top (7). */
  function followedVerse(): number | null {
    if (currentVerse !== null) return currentVerse;
    const a = captureAnchor();
    if (a && "v" in a) return a.v;
    return chapter?.verses[0]?.v ?? null;
  }
  function scheduleFollow(delay: number) {
    clearTimeout(followTimer);
    followTimer = window.setTimeout(follow, delay);
  }
  async function follow() {
    if (!settings.commentary_open || !chapter) return;
    const v = followedVerse();
    if (v === null) { cmtKey = ""; cmtView = { kind: "empty" }; return; }
    const key = `${chapter.book}.${chapter.chapter}.${v}`;
    if (key === cmtKey) return;
    cmtKey = key;
    const token = ++cmtToken;
    if (cmtView.kind !== "done") cmtView = { kind: "loading" };
    try {
      const result = await api.getCommentary(chapter.book, chapter.chapter, v);
      if (token === cmtToken) cmtView = { kind: "done", result };
    } catch (e) {
      if (token === cmtToken) { cmtKey = ""; cmtView = { kind: "error", message: `The commentary can't be shown: ${String(e)}` }; }
    }
  }
  /** A reference in the commentary: go to its range start (DESIGN.md 17.5). */
  function cmtRef(to: VerseRef) {
    jumpTo(to, false);
  }
  const panelOpen = () => settings.panel_open;
  function inPanel(el: Element | null) {
    return !!el && !!document.getElementById("p-study")?.contains(el);
  }
  /** Open Study; `focus` only when the reader asked for the panel itself (toggle, Ctrl+J). */
  async function openPanel(focus = true) {
    if (settings.panel_open) { if (focus) rcolView?.focusStudy(); return; }
    settings.panel_open = true;
    showPanel("study");
    scheduleSave();
    await tick();
    if (focus) rcolView?.focusStudy();
  }
  /** Close Study and release the selected word (15). Focus moves only off a vanishing panel. */
  function closePanel(returnTo?: HTMLElement | null) {
    const hadFocus = inPanel(document.activeElement);
    deselect();
    if (!settings.panel_open) return;
    settings.panel_open = false;
    hidePanel("study");
    scheduleSave();
    if (returnTo) returnTo.focus();
    else if (hadFocus) scroller?.focus();
  }
  // Ctrl+J and the toggle: closed -> open and focus; open, focus elsewhere -> focus; in the panel -> close (5)
  function togglePanel(fromButton = false) {
    if (!panelOpen()) { openPanel(); return; }
    if (fromButton) { closePanel(panelBtn); return; }
    if (inPanel(document.activeElement)) closePanel(scroller);
    else rcolView?.focusStudy();
  }

  // ---- the selected word (wave-2-layout.md 15; red-letters 3.4.3 E3/E4) ----
  // Attributes only: the text and the colour-lock spans are never touched.
  function select(w: HTMLElement) {
    if (selectedWord === w) return;
    deselect();
    selectedWord = w;
    w.setAttribute("aria-expanded", "true");
    w.setAttribute("aria-controls", "p-study");
    checkWrapped(w);
  }
  function deselect() {
    const w = selectedWord;
    selectedWord = null;
    if (!w) return;
    w.removeAttribute("aria-expanded");
    w.removeAttribute("aria-controls");
  }
  /** Click, Enter or Space on a word: select and open it, or on the selected word release it. */
  function activateWord(w: HTMLElement) {
    if (w === selectedWord && panelOpen()) { closePanel(); return; }
    select(w);
    lookupWord(w);
  }

  // Every key of the word is looked up and shown (a word can carry several).
  async function showKeys(title: string, keys: string[]) {
    const token = ++lookupToken;
    lookup = { kind: "loading", title, keys };
    await openPanel(false);
    try {
      const results = await Promise.all(keys.map((k) => api.getStrongs(k)));
      if (token === lookupToken) {
        lookup = { kind: "done", title, keys, results };
        say(`Strong's ${keys.join(", ")}`); // the entry heading, announced politely (15)
      }
    } catch (e) {
      if (token === lookupToken) lookup = { kind: "error", message: String(e) };
    }
  }
  function lookupWord(el: HTMLElement) {
    const keys = (el.dataset.strong ?? "").split(" ").filter(Boolean);
    if (!keys.length) return;
    return showKeys(`Lexicon for “${el.textContent ?? ""}”`, keys);
  }
  // A cross-reference inside an entry (w@src, strongsref) opens its target.
  const lookupRef = (key: string, from: string) => showKeys(`See ${key}, from ${from}`, [key]);

  // ---- highlights (PLAN.md 4 Highlights) ----
  async function highlight(color: HighlightColor) {
    if (selectionHighlight(color)) return; // Q30: a text selection is highlighted as a range
    if (!book || !chapter || currentVerse === null) {
      say("Click a verse first, then press 1 to 5 to highlight it.");
      return;
    }
    const v = currentVerse;
    await setVerseHighlight(v, highlights[v] === color ? null : color);
  }
  /** The whole-verse highlight of verse `v` of the shown chapter (reference-based, every translation). */
  async function setVerseHighlight(v: number, next: HighlightColor | null, record = true) {
    if (!book || !chapter) return;
    const before = highlights[v] ?? null;
    if (before === next) return;
    try {
      await api.setHighlight(`${book.osis}.${chapter.chapter}.${v}`, next);
      const h = { ...highlights };
      if (next) h[v] = next; else delete h[v];
      highlights = h;
      if (record) remember({ kind: "verse", book: book.osis, ch: chapter.chapter, v, before, after: next });
      say(next ? `Verse ${v} highlighted ${COLOR_NAMES[next]}` : `Highlight removed from verse ${v}`);
    } catch (e) {
      userWarning = String(e);
    }
  }

  // ---- pen-mode range highlights (highlights-pen.md 3, 4, 7) ----
  const LANG = "en"; // the translation's language for Intl.Segmenter; every installed text is English today
  const verseInfos = () => chapter?.verses.map((x) => ({ v: x.v, text: x.text })) ?? [];
  const drawn = $derived(drawnPieces(marks));
  const inked = $derived(inkPieces(drawn, preview));
  /** verses with drawn marks (the ", pen highlight" label) */
  const rangeVerses = $derived(new Set(drawn.map((p) => p.verse)));
  /** verses with a paint layer: drawn marks or the live preview */
  const inkVerses = $derived(new Set(inked.map((p) => p.verse)));

  function remember(c: HlChange) {
    past.push(c);
    if (past.length > 20) past.shift();
    future = [];
  }
  /** One write through the core; the chapter's marks come back. Queued, so rapid strokes land in order. */
  function penWrite(op: (tr: string, bk: string, ch: number) => Promise<MarkOut[]>, done?: (out: MarkOut[]) => void, record = true) {
    if (!translation || !book || !chapter) return;
    const tr = translation.abbrev, bk = book.osis, ch = chapter.chapter;
    penChain = penChain.then(async () => {
      const here = () => translation?.abbrev === tr && book?.osis === bk && chapter?.chapter === ch;
      const before = here() ? markSnapshot(marks) : null;
      try {
        const out = await op(tr, bk, ch);
        if (here()) marks = out;
        if (record && before) remember({ kind: "marks", tr, book: bk, ch, before, after: markSnapshot(out) });
        done?.(out);
      } catch (e) {
        const msg = String(e);
        showToast(msg); say(msg);
      }
    });
  }
  const penStroke = (tool: Tool, pieces: Piece[]) => {
    if (tool === "pen") {
      const c = settings.pen_color;
      penWrite((tr, bk, ch) => api.penApply(tr, bk, ch, c, pieces), () => say(`Highlighted ${COLOR_NAMES[c]}`));
    } else {
      penWrite((tr, bk, ch) => api.penErase(tr, bk, ch, pieces), () => say("Erased"));
    }
  };
  function penClickMark(tool: Tool, m: Drawn) {
    const c = settings.pen_color;
    if (tool === "eraser" || m.color === c) {
      penWrite((tr, bk, ch) => api.markRemove(tr, bk, ch, m.id), () => {
        say("Highlight removed. Ctrl+Z to undo.");
        penUndo = true;
        clearTimeout(penUndoTimer);
        penUndoTimer = window.setTimeout(() => (penUndo = false), UNDO_HOLD_MS);
      });
    } else {
      penWrite((tr, bk, ch) => api.markRecolor(tr, bk, ch, m.id, c), () => say(`Highlight recoloured ${COLOR_NAMES[c]}`));
    }
  }
  function penClickVerse(tool: Tool, v: number) {
    const now = highlights[v] ?? null;
    setVerseHighlight(v, tool === "eraser" || now === settings.pen_color ? null : settings.pen_color);
  }
  /** Q30: 1-5 outside pen mode with a selection in verse text highlights it as a range (grapheme snapping only). */
  function selectionHighlight(color: HighlightColor): boolean {
    const sel = document.getSelection();
    if (!sel || sel.isCollapsed || !sel.anchorNode || !sel.focusNode || !chapterEl) return false;
    const spot = (n: Node, off: number): Spot | null => {
      const vt = (n.nodeType === 1 ? (n as Element) : n.parentElement)?.closest<HTMLElement>(".vtext");
      if (!vt || !chapterEl!.contains(vt) || n.nodeType !== 3) return null;
      const o = caretOffset(vt, n, off);
      return o === null ? null : { verse: Number((vt.parentElement as HTMLElement).dataset.v), offset: o };
    };
    const a = spot(sel.anchorNode, sel.anchorOffset), f = spot(sel.focusNode, sel.focusOffset);
    if (!a || !f) return false;
    const pieces = strokePieces(verseInfos(), a, f, LANG, false);
    sel.removeAllRanges();
    if (pieces.length) penWrite((tr, bk, ch) => api.penApply(tr, bk, ch, color, pieces), () => say(`Selection highlighted ${COLOR_NAMES[color]}`));
    return true;
  }
  async function applyChange(c: HlChange, side: "before" | "after") {
    if (c.kind === "verse") {
      if (book?.osis === c.book && chapter?.chapter === c.ch) { await setVerseHighlight(c.v, c[side], false); return; }
      try { await api.setHighlight(`${c.book}.${c.ch}.${c.v}`, c[side]); } catch (e) { userWarning = String(e); }
      return;
    }
    const target = c[side];
    penChain = penChain.then(async () => {
      try {
        const out = await api.marksReplaceChapter(c.tr, c.book, c.ch, target);
        if (translation?.abbrev === c.tr && book?.osis === c.book && chapter?.chapter === c.ch) marks = out;
      } catch (e) { const msg = String(e); showToast(msg); say(msg); }
    });
    await penChain;
  }
  async function undoHighlight() {
    const c = past.pop();
    if (!c) { say("Nothing to undo"); return; }
    future.push(c);
    penUndo = false;
    await applyChange(c, "before");
    say("Undone");
  }
  async function redoHighlight() {
    const c = future.pop();
    if (!c) { say("Nothing to redo"); return; }
    past.push(c);
    await applyChange(c, "after");
    say("Redone");
  }

  function setPen(tool: Tool | null) {
    if (tool && !pen) closePopovers();
    penCtl?.cancel();
    preview = null;
    const was = pen;
    pen = tool;
    clearTimeout(trayTimer);
    trayLeaving = !tool && (!!was || trayLeaving);
    if (trayLeaving) trayTimer = window.setTimeout(() => (trayLeaving = false), 120); // --dur-exit
    if (tool) root.dataset.pen = tool; else delete root.dataset.pen;
    updateCursor();
    if (!tool) say("Highlighter off");
    else if (!was) say(`Highlighter on, ${COLOR_NAMES[settings.pen_color]}. Drag over words; Esc to stop.`);
    else say(tool === "eraser" ? "Eraser" : `Pen, ${COLOR_NAMES[settings.pen_color]}`);
  }
  function setPenColor(c: HighlightColor) {
    settings.pen_color = c;
    scheduleSave();
    setPen("pen");
  }
  /** --cursor-pen / --cursor-eraser from the theme's swatch colour (3.3). */
  function updateCursor() {
    if (!pen) return;
    const cs = getComputedStyle(root);
    const t = (n: string, d: string) => cs.getPropertyValue(n).trim() || d;
    const halo = t("--cursor-halo", "#FFFFFF"), ink = t("--cursor-ink", "#1C1B19");
    root.style.setProperty("--cursor-pen", penCursor(t(`--color-hl-${settings.pen_color}-swatch`, ink), halo, ink));
    root.style.setProperty("--cursor-eraser", eraserCursor(halo, ink));
  }
  function penClamped() {
    if (clampShown) return;
    clampShown = true;
    penHint = "Highlights stay within one chapter";
    say(penHint);
    window.setTimeout(() => (penHint = null), 2500);
  }
  /** Draw the paint layer (7.5): after render, on every change and on reflow. */
  function paint() {
    if (!chapterEl || !chapter) return;
    paintInk(chapterEl, inked, new Map(chapter.verses.map((x) => [x.v, x.text])));
  }

  // ---- copy: exact stored text plus reference (PLAN.md 2.5) ----
  function oncopy(e: ClipboardEvent) {
    const sel = document.getSelection();
    if (!sel || sel.rangeCount === 0 || !chapterEl || !book || !chapter || !translation) return;
    const range = sel.getRangeAt(0);
    if (!chapterEl.contains(range.commonAncestorContainer)) return;
    const parts: { v: number; text: string }[] = [];
    for (const vt of chapterEl.querySelectorAll<HTMLElement>(".vtext")) {
      if (!range.intersectsNode(vt)) continue;
      const full = vt.textContent ?? "";
      const pre = document.createRange();
      pre.selectNodeContents(vt);
      let s = 0, en = full.length;
      if (vt.contains(range.startContainer)) { pre.setEnd(range.startContainer, range.startOffset); s = pre.toString().length; }
      if (vt.contains(range.endContainer)) {
        const p2 = document.createRange(); p2.selectNodeContents(vt); p2.setEnd(range.endContainer, range.endOffset);
        en = p2.toString().length;
      }
      if (en > s) parts.push({ v: Number((vt.parentElement as HTMLElement).dataset.v), text: full.slice(s, en) });
    }
    if (!parts.length) return;
    const first = parts[0].v, last = parts[parts.length - 1].v;
    const ref = refLabel({ book, chapter: chapter.chapter, verse: first }) + (last !== first ? `–${last}` : "");
    e.clipboardData?.setData("text/plain", `${parts.map((p) => p.text).join("\n")}\n${ref} (${translation.abbrev})`);
    e.preventDefault();
  }

  async function startNewUserData() {
    try {
      const kept = await api.startNewUserData();
      userWarning = null;
      userRefused = false;
      userNotice = kept
        ? `A new data file was started. The old one was kept as ${kept}, in the same folder.`
        : "A new data file was started.";
      saveNow();
    } catch (e) {
      userWarning = String(e);
    }
  }

  // ---- WebView2 browser behaviour (D5-F4) ----
  // Tauri 2.12 doesn't expose WebView2's AreBrowserAcceleratorKeysEnabled /
  // AreDefaultContextMenusEnabled, so the page swallows them: reload, print,
  // save page, find, view source, history navigation, caret browsing. Copy
  // stays on Ctrl+C through the app's own handler (oncopy).
  function browserKey(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    if (["F3", "F5", "F7", "F12", "BrowserBack", "BrowserForward", "BrowserRefresh", "BrowserSearch", "BrowserHome"].includes(k)) return true;
    if (e.altKey && (k === "ArrowLeft" || k === "ArrowRight" || k === "Home")) return true;
    return ctrl && ["r", "p", "s", "f", "u", "n", "o", "h"].includes(k);
  }
  function oncontextmenu(e: MouseEvent) {
    if (!inTextField(e.target)) e.preventDefault(); // text fields keep Cut/Copy/Paste
  }

  // ---- keyboard map (reader.md 10) ----
  function inTextField(t: EventTarget | null) {
    return t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || (t instanceof HTMLElement && t.isContentEditable);
  }
  function onkeydown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    const k = e.key;
    const modal = !!document.querySelector("dialog[open]");
    // Ctrl+Z undoes a bookmark clear while its toast shows, from the picker or the command box (bookmarks.md 6)
    if (ctrl && !e.shiftKey && (k === "z" || k === "Z") && undo) {
      const a = document.activeElement;
      if (a?.id === "cmd-input" || !!a?.closest(".picker")) { e.preventDefault(); undoClear(); return; }
    }
    // Ctrl+F: search in the command box; F3 / Shift+F3 outside the field: step through results (wave-2-layout.md 6.7)
    if (ctrl && !e.altKey && (k === "f" || k === "F")) {
      e.preventDefault();
      if (!fatal && !modal) { closePopovers(); cmdBox?.focusSearch(); }
      return;
    }
    if (k === "F3") {
      e.preventDefault();
      if (!fatal && !modal && !inTextField(e.target)) searchStep(e.shiftKey ? -1 : 1);
      return;
    }
    // Ctrl+Shift+N: Notes (5), routed before the WebView2 swallow list (which blocks Ctrl+N)
    if (ctrl && e.shiftKey && !e.altKey && (k === "n" || k === "N")) {
      e.preventDefault();
      if (!fatal && !modal) { closePopovers(); toggleNotes(); }
      return;
    }
    if (browserKey(e)) { e.preventDefault(); return; }
    if (fatal) return;
    if (modal) return; // a modal is up: it owns the keyboard (Esc closes it natively)
    if (ctrl && (k === "l" || k === "L" || k === "g" || k === "G")) { e.preventDefault(); closePopovers(); cmdBox?.focusBox(); return; }
    if (ctrl && (k === "j" || k === "J")) { e.preventDefault(); togglePanel(); return; }
    if (ctrl && e.shiftKey && !e.altKey && (k === "c" || k === "C")) { e.preventDefault(); closePopovers(); toggleComm(); return; }
    if (ctrl && (k === "=" || k === "+")) { e.preventDefault(); setStep(settings.text_step + 1); return; }
    if (ctrl && (k === "-" || k === "_")) { e.preventDefault(); setStep(settings.text_step - 1); return; }
    if (ctrl && k === "0") { e.preventDefault(); setStep(5); return; }
    if (ctrl && k === "PageUp" && !inTextField(e.target)) { e.preventDefault(); step(-1); return; }
    if (ctrl && k === "PageDown" && !inTextField(e.target)) { e.preventDefault(); step(1); return; }
    if (k === "F6") { e.preventDefault(); cycleRegion(e.shiftKey ? -1 : 1); return; }
    // Esc during a pointer drag of the divider puts it back where the drag began (17.2)
    if (k === "Escape" && rcolView?.cancelDrag()) { e.preventDefault(); return; }
    if (k === "Escape" && (menuOpen || trOpen || optionsOpen || pickerOpen)) { e.preventDefault(); closePopovers(true); return; }
    const active = document.activeElement;
    const inPane = !!scroller && active instanceof Node && scroller.contains(active);
    const inTray = !!active?.closest(".pen-tray");
    // pen mode (highlights-pen.md 3.1): Esc cancels a stroke, then turns the pen off, before the Study panel closes
    if (k === "Escape" && pen && (inPane || inTray || active === document.body)) {
      e.preventDefault();
      if (!penCtl?.cancel()) { setPen(null); if (inTray) scroller?.focus(); }
      return;
    }
    // Ctrl+Z / Ctrl+Y (Ctrl+Shift+Z): undo / redo the last highlight change, pen on or off
    if ((inPane || inTray) && ctrl && !e.altKey && (k === "z" || k === "Z" || k === "y" || k === "Y")) {
      e.preventDefault();
      if ((k === "z" || k === "Z") && !e.shiftKey) undoHighlight(); else redoHighlight();
      return;
    }
    if ((inPane || inTray) && !ctrl && !e.altKey) {
      if (k === "h" || k === "H") { e.preventDefault(); setPen(pen ? null : "pen"); return; }
      if (pen && /^[1-5]$/.test(k)) { e.preventDefault(); setPenColor(PEN_KEYS[k]); return; }
      if (pen && (k === "e" || k === "E")) { e.preventDefault(); setPen(pen === "eraser" ? "pen" : "eraser"); return; }
    }
    // Esc in Notes or the notes editor only moves focus to the reading pane; the panel stays open (5)
    if (k === "Escape" && inNotes(active)) { e.preventDefault(); scroller?.focus(); return; }
    // Esc in the Commentary panel or on the column's separators only moves focus to the reading pane (5)
    if (k === "Escape" && (inComm(active) || inColChrome(active))) { e.preventDefault(); scroller?.focus(); return; }
    // Esc in the reading pane or the Study panel closes Study and clears the
    // selected word; focus goes to that word if it was in the panel (5)
    if (k === "Escape" && panelOpen() && (inPane || inPanel(active) || active === document.body)) {
      e.preventDefault();
      const word = selectedWord;
      closePanel(inPanel(active) ? (word?.isConnected ? word : scroller) : null);
      return;
    }
    if (inPane && !ctrl && !e.altKey) {
      if (k === "[") { e.preventDefault(); step(-1); }
      else if (k === "]") { e.preventDefault(); step(1); }
      else if (/^[1-5]$/.test(k)) { e.preventDefault(); highlight(COLORS[Number(k) - 1]); }
      else if ((k === "Enter" || k === " ") && active instanceof HTMLElement && active.matches(".vtext .w")) {
        e.preventDefault(); activateWord(active);
      }
    }
  }
  function cycleRegion(dir: 1 | -1) {
    const regions: (HTMLElement | null | undefined)[] = [
      settings.notes_open ? document.getElementById("p-notes") : null,
      document.getElementById("cmd-input"), scroller,
      settings.commentary_open ? document.querySelector<HTMLElement>("#p-comm .panel-title") : null,
      settings.panel_open ? document.querySelector<HTMLElement>("#p-study .panel-head button") : null,
    ];
    const list = regions.filter((x): x is HTMLElement => !!x);
    const cur = list.findIndex((r) => r.contains(document.activeElement) || (r === scroller && document.activeElement === r)
      || (r.closest("#p-study") !== null && r.closest("#p-study")!.contains(document.activeElement))
      || (r.closest("#p-comm") !== null && r.closest("#p-comm")!.contains(document.activeElement)));
    const nextEl = list[(cur + dir + list.length) % list.length];
    if (nextEl?.id === "p-notes") notesView?.focusEditor(); else nextEl?.focus();
  }
  function onwheel(e: WheelEvent) {
    if (!e.ctrlKey) return;
    e.preventDefault();
    wheelAcc += e.deltaY;
    while (Math.abs(wheelAcc) >= 100) {
      setStep(settings.text_step + (wheelAcc < 0 ? 1 : -1));
      wheelAcc -= Math.sign(wheelAcc) * 100;
    }
  }

  // ---- popovers ----
  function closePopovers(returnFocus = false) {
    const wasPicker = pickerOpen;
    menuOpen = trOpen = optionsOpen = pickerOpen = false;
    if (returnFocus && wasPicker) cmdBox?.focusBox();
  }
  function onpointerdown(e: PointerEvent) {
    const t = e.target as HTMLElement;
    if (!t.closest(".anchor") && !t.closest(".cmd")) menuOpen = trOpen = optionsOpen = pickerOpen = false;
  }

  // ---- reading pane clicks: words open the lexicon, verses become current ----
  function paneClick(e: MouseEvent) {
    if (pen) return; // pen mode: clicks are pen actions (4.4), handled on pointerup
    const t = e.target as HTMLElement;
    const w = t.closest<HTMLElement>(".vtext .w");
    if (w) { activateWord(w); return; }
    const v = t.closest<HTMLElement>(".verse");
    if (v && document.getSelection()?.isCollapsed !== false) {
      currentVerse = Number(v.dataset.v);
      scheduleSave();
    }
  }

  async function switchTranslation(t: TranslationInfo) {
    trOpen = false;
    if (translation && t.abbrev === translation.abbrev) return;
    const a = captureAnchor();
    const osis = book?.osis, ch = chapter?.chapter ?? 1, cv = currentVerse;
    translation = t;
    books = await api.listBooks(t.abbrev);
    const b = books.find((x) => x.osis === osis) ?? books[0];
    await goTo(b.ord, b.osis === osis ? Math.min(ch, b.chapters) : 1, { anchor: a, animate: false, verse: null });
    currentVerse = cv;
  }

  onMount(() => {
    (async () => {
      try {
        info = await api.appInfo();
      } catch (e) {
        fatal = { title: "Revelation couldn't start.", detail: String(e) };
        return;
      }
      if (info.content_error || info.translations.length === 0) {
        fatal = {
          title: "Revelation can't show the Bible text.",
          detail: info.content_error ?? "No content file is installed.",
        };
        return;
      }
      userWarning = info.user_error;
      userRefused = !!info.user_error;
      userTooNew = !!info.user_error && info.user_too_new === true;
      let saved: { position: Position | null; settings: Settings } = { position: null, settings: { ...DEFAULTS } };
      if (!userWarning) {
        try { saved = await api.loadState(); } catch (e) { userWarning = String(e); }
      }
      settings = { ...DEFAULTS, ...saved.settings };
      rightPx = clampWidth(restoreWidthRem(settings.right_width_rem) * remPx(), window.innerWidth, remPx());
      settings.right_split = restoreSplit(settings.right_split); // clamped to the column's height when drawn, never rewritten
      applyRoot();
      commVis = settings.commentary_open;
      studyVis = settings.panel_open;
      makeColumn(commVis || studyVis); // saved open panels are simply there, no slide
      notesPx = clampNotes((Number.isFinite(settings.notes_width_rem) ? Math.max(NOTES_REM.min, Math.min(NOTES_REM.max, settings.notes_width_rem)) : 20) * remPx());
      makeNotes(settings.notes_open);
      const pos = saved.position;
      translation = info.translations.find((t) => t.abbrev === pos?.translation) ?? info.translations[0];
      books = await api.listBooks(translation.abbrev);
      const b = books.find((x) => x.osis === pos?.book) ?? books[0];
      const ch = pos && b.osis === pos.book && pos.chapter <= b.chapters ? pos.chapter : 1;
      const anchor: Anchor = pos?.anchor_verse ? { v: pos.anchor_verse, f: pos.anchor_frac ?? 0 } : null;
      if (!userWarning) await refreshBookmarks();
      await goTo(b.ord, ch, { animate: false, anchor, restore: true });
      currentVerse = pos?.current_verse ?? null;
      ready = true;
    })();
    const onHide = () => { tracker.end(); flushBookmarks(); saveNow(); void notes.flush(); };
    // lib.rs holds the window's close once and calls this (W2-5: the notes are saved before the app closes)
    (window as unknown as { __revelationClose?: () => void }).__revelationClose = () => { void closeRequested(); };
    // the restored visit arms on the first trusted input (bookmarks.md 2.3)
    const onInput = (e: Event) => { if (trust.isTrusted(e)) tracker.arm(); };
    const onFocus = (e: Event) => { if (e.target === window) { windowFocused = true; updateAttending(); } };
    const onBlur = (e: Event) => { if (e.target === window) { windowFocused = false; updateAttending(); } };
    const clock = window.setInterval(updateAttending, 250);
    for (const t of ["keydown", "pointerdown", "wheel"]) window.addEventListener(t, onInput, { capture: true, passive: true });
    window.addEventListener("focus", onFocus);
    window.addEventListener("blur", onBlur);
    document.addEventListener("visibilitychange", updateAttending);
    const onResize = () => {
      rightPx = clampWidth(rightPx, window.innerWidth, remPx());
      notesPx = clampNotes(notesPx);
      flushSync();
      rcolView?.reflow(); // the split is a fraction: it is re-clamped to the new height (17.1)
      restoreAnchor(lastAnchor); // reader.md 9: window resize keeps the reader in the same verse
      if (selectedWord) checkWrapped(selectedWord);
    };
    window.addEventListener("pagehide", onHide);
    window.addEventListener("resize", onResize);
    window.addEventListener("wheel", onwheel, { passive: false });
    return () => {
      window.removeEventListener("pagehide", onHide);
      window.removeEventListener("resize", onResize);
      window.removeEventListener("wheel", onwheel);
      for (const t of ["keydown", "pointerdown", "wheel"]) window.removeEventListener(t, onInput, { capture: true });
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("blur", onBlur);
      document.removeEventListener("visibilitychange", updateAttending);
      clearInterval(clock);
      delete (window as unknown as { __revelationClose?: () => void }).__revelationClose;
      notes.saver.dispose(); // the app is going (tests): no retry outlives it
    };
  });

  // the commentary follows the current verse and the chapter (wave-2-layout.md 7)
  $effect(() => {
    void currentVerse; void chapter; void settings.commentary_open;
    scheduleFollow(0);
  });
  // the pause conditions change: the dwell clock stops or restarts at once (bookmarks.md 2.2)
  $effect(() => {
    void pickerOpen; void menuOpen; void trOpen;
    updateAttending();
  });
  // reading-pane listeners without a11y-noise: keyboard equivalents exist (reader.md 10)
  $effect(() => {
    const el = scroller;
    if (!el) return;
    // user scroll intent (bookmarks.md 2.4): wheel, touch, scrolling keys, the scrollbar
    const intent = (e: Event) => { if (trust.isTrusted(e)) intentUntil = performance.now() + INTENT_MS; };
    const onKey = (e: KeyboardEvent) => { if (SCROLL_KEYS.has(e.key) && !e.altKey) intent(e); };
    let onBar = false;
    const onDown = (e: PointerEvent) => { onBar = e.offsetX >= el.clientWidth || e.offsetY >= el.clientHeight; };
    const onMove = (e: PointerEvent) => { if (onBar && e.buttons) intent(e); };
    const onUp = () => { onBar = false; };
    const onScroll = () => {
      const delta = el.scrollTop - lastScrollTop;
      lastScrollTop = el.scrollTop;
      if (performance.now() <= intentUntil) {
        tracker.userScroll(delta, el.clientHeight);
        const end = el.querySelector<HTMLElement>(".chapter-end");
        if (end) {
          const r = end.getBoundingClientRect(), p = el.getBoundingClientRect();
          const seen = Math.min(r.bottom, p.bottom) - Math.max(r.top, p.top);
          if (r.height > 0 && seen >= r.height / 2) tracker.endReached();
        }
      }
      scrolled = el.scrollTop > 0;
      scheduleSave();
      if (!anchorRaf) anchorRaf = requestAnimationFrame(() => { anchorRaf = 0; lastAnchor = captureAnchor(); });
      if (currentVerse === null && settings.commentary_open) scheduleFollow(300); // 300ms after scrolling stops (7)
      if (settings.notes_open) { clearTimeout(visibleTimer); visibleTimer = window.setTimeout(measureVisible, 300); } // 8.4
    };
    el.addEventListener("click", paneClick);
    el.addEventListener("scroll", onScroll, { passive: true });
    el.addEventListener("wheel", intent, { passive: true });
    el.addEventListener("touchmove", intent, { passive: true });
    el.addEventListener("keydown", onKey);
    el.addEventListener("pointerdown", onDown);
    el.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      el.removeEventListener("click", paneClick); el.removeEventListener("scroll", onScroll);
      el.removeEventListener("wheel", intent); el.removeEventListener("touchmove", intent);
      el.removeEventListener("keydown", onKey); el.removeEventListener("pointerdown", onDown);
      el.removeEventListener("pointermove", onMove); window.removeEventListener("pointerup", onUp);
    };
  });
  $effect(() => {
    const el = chapterEl;
    if (!el) return;
    const end = (e: AnimationEvent) => {
      if (e.target === el) el.classList.remove("entering");
      else if (e.animationName.startsWith("verse-flash")) (e.target as HTMLElement).classList.remove("flashing");
    };
    el.addEventListener("animationend", end);
    return () => el.removeEventListener("animationend", end);
  });
  // the pen's pointer handling on the reading pane (highlights-pen.md 4)
  $effect(() => {
    const el = scroller, ch = chapterEl;
    if (!el || !ch) return;
    const ctl = attachPen({
      scroller: el, chapterEl: ch,
      tool: () => pen, color: () => settings.pen_color, verses: verseInfos, lang: () => LANG, marks: () => marks,
      preview: (p) => (preview = p), stroke: penStroke, clickMark: penClickMark,
      clickText: (p) => penStroke("pen", [p]), clickVerseNumber: penClickVerse, clamped: penClamped,
    });
    penCtl = ctl;
    return () => { ctl.destroy(); if (penCtl === ctl) penCtl = null; };
  });
  // repaint after every render and change (7.5); reflow repaints through the observer
  $effect(() => {
    void inked; void chapter; void settings.text_step; void settings.verse_numbers; void rightPx;
    tick().then(paint);
  });
  $effect(() => {
    const el = chapterEl;
    if (!el || typeof ResizeObserver !== "function") return;
    let raf = 0;
    const ro = new ResizeObserver(() => { if (!raf) raf = requestAnimationFrame(() => { raf = 0; paint(); }); });
    ro.observe(el);
    document.fonts?.ready.then(paint).catch(() => {});
    return () => { ro.disconnect(); cancelAnimationFrame(raf); };
  });
  // the cursor's nib follows the theme (3.3)
  $effect(() => {
    void pen; void settings.pen_color;
    updateCursor();
    const mq = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!mq) return;
    const on = () => updateCursor();
    mq.addEventListener?.("change", on);
    return () => mq.removeEventListener?.("change", on);
  });
</script>

<svelte:window {onkeydown} {onpointerdown} {oncopy} {oncontextmenu} />

{#if userWarning && !fatal}
  <div class="banner warn" role="alert">
    <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" /></svg>
    <span>Highlights and your place can't be saved right now: {userWarning}</span>
    {#if userRefused && !userTooNew}
      <button type="button" class="btn btn-outline banner-action" onclick={startNewUserData}>
        Start a new data file (the old one is kept as a backup)
      </button>
    {/if}
  </div>
{:else if userNotice && !fatal}
  <div class="banner" role="status">
    <span>{userNotice}</span>
    <button type="button" class="btn btn-quiet banner-action" onclick={() => (userNotice = null)}>Dismiss</button>
  </div>
{/if}

{#if fatal}
  <main class="fatal" aria-labelledby="fatal-title">
    <h1 id="fatal-title">{fatal.title}</h1>
    <p class="l1">
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" /></svg>
      Please repair or reinstall Revelation.
    </p>
    <p>Nothing is shown, because a damaged or replaced file could show words that aren't in the text.</p>
    <p class="detail">Details: {fatal.detail}</p>
  </main>
{:else}
  <div class="app">
    {#if notesLayout !== "hidden"}
      <NotesPanel
        bind:this={notesView}
        store={notes}
        {books}
        context={notesCtx}
        layout={notesLayout}
        widthPx={notesPx}
        minPx={notesBounds().min}
        maxPx={notesBounds().max}
        unavailable={userRefused ? userWarning : null}
        onClose={() => closeNotes(scroller)}
        onJump={notesJump}
        onResize={resizeNotes}
      />
    {/if}
    <section class="reader" class:scrolled aria-label="Reader">
      <header class="topbar">
        <div class="tb-start">
          <span class="anchor">
            <button type="button" class="ibtn" bind:this={menuBtn} aria-label="Menu" aria-haspopup="menu" aria-expanded={menuOpen} title="Menu"
              onclick={() => { const o = !menuOpen; closePopovers(); menuOpen = o; }}>
              <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            {#if menuOpen}
              <div class="pop menu-pop start" role="menu" aria-label="Menu">
                <button type="button" class="mi" role="menuitem" aria-disabled="true">Settings…</button>
                <button type="button" class="mi" role="menuitem" onclick={() => { menuOpen = false; keysDialog?.showModal(); }}>Keyboard shortcuts</button>
                <button type="button" class="mi" role="menuitem" onclick={() => { menuOpen = false; aboutDialog?.open(); }}>About Revelation</button>
              </div>
            {/if}
          </span>
          <button type="button" class="ibtn" aria-label="Notes panel" aria-pressed={settings.notes_open} aria-controls="p-notes"
            title="Notes · Ctrl+Shift+N" bind:this={notesBtn} onclick={() => toggleNotes(true)}>
            <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M13.4 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7.4" /><path d="M2 6h4" /><path d="M2 10h4" /><path d="M2 14h4" /><path d="M2 18h4" /><path d="M21.378 5.626a1 1 0 1 0-3.004-3.004l-5.01 5.012a2 2 0 0 0-.506.854l-.837 2.87a.5.5 0 0 0 .62.62l2.87-.837a2 2 0 0 0 .854-.506z" /></svg>
          </button>
        </div>

        <div class="tb-centre">
          <button type="button" class="ibtn tb-prev" aria-label="Previous chapter" title="Previous chapter · Ctrl+PgUp"
            aria-disabled={prev ? undefined : "true"} onclick={() => step(-1)}>
            <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M15 18l-6-6 6-6" /></svg>
          </button>
          <span class="anchor">
            <CommandBox
              bind:this={cmdBox}
              {books}
              {location}
              {pickerOpen}
              translation={translation?.abbrev ?? ""}
              searchError={info && info.search_available === false ? (info.search_error ?? "the search index is unavailable") : null}
              onGo={goRef}
              {onJump}
              onSay={say}
              {bookmarks}
              onClearBookmark={clearBookmark}
              onFocusChange={(f) => { cmdFocused = f; if (f) refreshBookmarks(); updateAttending(); }}
              onOpenPicker={() => { const o = !pickerOpen; closePopovers(); if (o) refreshBookmarks(); pickerOpen = o; }}
              onLeave={() => scroller?.focus()}
            />
            {#if pickerOpen && book && chapter}
              <Picker
                {books}
                currentOrd={book.ord}
                currentChapter={chapter.chapter}
                {bookmarks}
                onClearBookmark={clearBookmark}
                onChoose={(ord, ch) => { pickerOpen = false; goTo(ord, ch, { dir: 1 }); scroller?.focus(); }}
                onClose={() => { pickerOpen = false; cmdBox?.focusBox(); }}
              />
            {/if}
          </span>
          <button type="button" class="ibtn tb-next" aria-label="Next chapter" title="Next chapter · Ctrl+PgDn"
            aria-disabled={next ? undefined : "true"} onclick={() => step(1)}>
            <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M9 18l6-6-6-6" /></svg>
          </button>
        </div>

        <div class="tb-end">
          <span class="anchor">
            <button type="button" class="btn btn-quiet" aria-haspopup="menu" aria-expanded={trOpen} title={translation?.name}
              onclick={() => { const o = !trOpen; closePopovers(); trOpen = o; }}>{translation?.abbrev ?? ""}</button>
            {#if trOpen && info}
              <div class="pop menu-pop" role="menu" aria-label="Translation">
                {#each info.translations as t (t.abbrev)}
                  <button type="button" class="mi" role="menuitemradio" aria-checked={t.abbrev === translation?.abbrev}
                    onclick={() => switchTranslation(t)}>
                    <span class="chk" aria-hidden="true">{t.abbrev === translation?.abbrev ? "✓" : ""}</span>
                    <span><span>{t.abbrev}</span>{#if t.name !== t.abbrev}<span class="sub">{t.name}</span>{/if}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </span>
          <span class="anchor">
            <button type="button" class="ibtn" aria-label="Highlighter" aria-pressed={pen !== null} aria-keyshortcuts="H" title="Highlighter · H"
              onclick={() => setPen(pen ? null : "pen")}>
              <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 11-6 6v3h9l3-3" /><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4" /></svg>
            </button>
            {#if pen || trayLeaving}
              <PenTray color={settings.pen_color} eraser={pen === "eraser"} hint={penHint} leaving={!pen}
                onColor={setPenColor} onEraser={() => setPen(pen === "eraser" ? "pen" : "eraser")}
                onDone={() => { setPen(null); scroller?.focus(); }} />
            {/if}
          </span>
          <span class="anchor">
            <button type="button" class="ibtn" aria-label="Reading options" aria-haspopup="dialog" aria-expanded={optionsOpen} title="Reading options"
              onclick={() => { const o = !optionsOpen; closePopovers(); optionsOpen = o; }}><span class="aa" aria-hidden="true">Aa</span></button>
            {#if optionsOpen}
              <ReadingOptions
                {settings}
                hasRedLetters={translation?.has_red_letters ?? false}
                sizePx={STEPS[settings.text_step - 1]}
                onToggle={toggle}
                onSize={(d) => setStep(settings.text_step + d)}
                onClose={() => (optionsOpen = false)}
              />
            {/if}
          </span>
          <button type="button" class="ibtn" aria-label="Commentary panel" aria-pressed={settings.commentary_open} aria-controls="p-comm"
            title="Commentary · Ctrl+Shift+C" bind:this={commBtn} onclick={() => toggleComm(true)}>
            <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 7v14" /><path d="M16 12h2" /><path d="M16 8h2" /><path d="M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z" /><path d="M6 12h2" /><path d="M6 8h2" /></svg>
          </button>
          <button type="button" class="ibtn" aria-label="Study panel" aria-pressed={settings.panel_open} aria-controls="p-study"
            title="Study panel · Ctrl+J" bind:this={panelBtn} onclick={() => togglePanel(true)}>
            <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M15 4v16" /></svg>
          </button>
        </div>
      </header>

      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="reader-scroll" bind:this={scroller} tabindex="0" role="region" aria-label={location || "Chapter"}>
        <div class="reading">
          <div class="chapter" bind:this={chapterEl}>
            {#if chapter && book}
              {#key chapter}
                <ChapterView
                  {chapter}
                  {book}
                  {highlights}
                  {currentVerse}
                  ranges={rangeVerses}
                  ink={inkVerses}
                  prevLabel={prev ? locationLabel(prev.b, prev.ch) : null}
                  nextLabel={next ? locationLabel(next.b, next.ch) : null}
                  onPrev={() => step(-1)}
                  onNext={() => step(1)}
                />
              {/key}
            {/if}
          </div>
        </div>
      </div>

      {#if currentVerse !== null && !toast.show}
        <div class="hlbar" role="toolbar" aria-label="Highlight verse {currentVerse}">
          <span class="lbl">Verse {currentVerse}</span>
          {#each COLORS as c, i (c)}
            <button type="button" class="swatch" data-c={c} aria-pressed={highlights[currentVerse] === c}
              aria-label="{i + 1} {COLOR_NAMES[c]}" title="{COLOR_NAMES[c]} · {i + 1}" onclick={() => highlight(c)}>{i + 1}</button>
          {/each}
        </div>
      {/if}
      <div class="toast" class:show={toast.show} aria-hidden="true">{toast.text}</div>
      {#if undo}
        <div class="toast undo-toast show">
          <span>Bookmark cleared</span><span aria-hidden="true"> · </span>
          <button type="button" class="btn btn-quiet" onpointerdown={(e) => e.preventDefault()} onclick={undoClear}>Undo</button>
        </div>
      {:else if penUndo}
        <div class="toast undo-toast show">
          <span>Highlight removed</span><span aria-hidden="true"> · </span>
          <button type="button" class="btn btn-quiet" onpointerdown={(e) => e.preventDefault()} onclick={undoHighlight}>Undo</button>
        </div>
      {/if}
    </section>

    {#if colLayout !== "hidden"}
      <RightColumn
        bind:this={rcolView}
        layout={colLayout}
        {commVis}
        {studyVis}
        {commLift}
        {studyLift}
        view={cmtView}
        {books}
        {lookup}
        widthPx={rightPx}
        minPx={colMin()}
        maxPx={colMax()}
        split={settings.right_split}
        onCloseComm={() => closeComm(scroller)}
        onCloseStudy={() => closePanel(scroller)}
        onResize={resizeCol}
        onResetWidth={() => resizeCol(RC.widthDefault * remPx(), true)}
        onSplit={setSplit}
        onCmtRef={cmtRef}
        onLexRef={lookupRef}
      />
    {/if}
  </div>
{/if}

<div class="sr-only" aria-live="polite">{live}</div>

<AboutDialog bind:this={aboutDialog} onclose={() => menuBtn?.focus()} />

<dialog class="keys close-ask" bind:this={closeAsk} aria-labelledby="close-ask-title">
  <h2 id="close-ask-title">Your latest notes aren't saved</h2>
  <p>Revelation couldn't save your most recent changes to your notes, so closing now loses them. Keep Revelation open and it keeps trying every 5 seconds.</p>
  <div class="foot">
    <button type="button" class="btn btn-outline" onclick={() => closeAsk?.close()}>Keep Revelation open</button>
    <button type="button" class="btn btn-outline" onclick={closeAnyway}>Close anyway</button>
  </div>
</dialog>

<dialog class="keys" bind:this={keysDialog} aria-labelledby="keys-title" onclose={() => menuBtn?.focus()}>
  <h2 id="keys-title">Keyboard shortcuts</h2>
  <table>
    <tbody>
      <tr><td><kbd>Ctrl L</kbd> <kbd>Ctrl G</kbd></td><td>Go to a passage (command box)</td></tr>
      <tr><td><kbd>Ctrl F</kbd></td><td>Search the Bible (the command box; reopens the last search)</td></tr>
      <tr><td><kbd>F3</kbd> <kbd>Shift F3</kbd></td><td>Next / previous result of the last search</td></tr>
      <tr><td><kbd>Shift Delete</kbd> <kbd>Shift F10</kbd></td><td>On a bookmarked book or chapter: clear its bookmark (<kbd>Ctrl Z</kbd> undoes)</td></tr>
      <tr><td><kbd>↓</kbd> <kbd>Alt ↓</kbd></td><td>In the command box: book and chapter picker</td></tr>
      <tr><td><kbd>Esc</kbd></td><td>Cancel and close (a popup first, then a pen stroke, the pen, then the study panel)</td></tr>
      <tr><td><kbd>Enter</kbd> <kbd>Space</kbd></td><td>On a Strong's word: open its entry; on the selected word, close it</td></tr>
      <tr><td><kbd>Ctrl PgUp</kbd> <kbd>Ctrl PgDn</kbd></td><td>Previous / next chapter</td></tr>
      <tr><td><kbd>[</kbd> <kbd>]</kbd></td><td>Previous / next chapter, while the text has focus</td></tr>
      <tr><td><kbd>1</kbd>–<kbd>5</kbd></td><td>Highlight the current verse (same key again removes it); with text selected, highlight the selection</td></tr>
      <tr><td><kbd>H</kbd></td><td>Highlighter pen on / off, while the text has focus</td></tr>
      <tr><td><kbd>1</kbd>–<kbd>5</kbd> <kbd>E</kbd></td><td>With the pen on: pen colour / eraser (drag over words; <kbd>Alt</kbd> for letters)</td></tr>
      <tr><td><kbd>Ctrl Z</kbd> <kbd>Ctrl Y</kbd></td><td>Undo / redo the last highlight change, while the text has focus</td></tr>
      <tr><td><kbd>Ctrl +</kbd> <kbd>Ctrl −</kbd> <kbd>Ctrl 0</kbd></td><td>Larger / smaller / default text size</td></tr>
      <tr><td><kbd>Ctrl Shift N</kbd></td><td>Notes panel (today's note)</td></tr>
      <tr><td><kbd>Ctrl B</kbd> <kbd>Ctrl I</kbd> <kbd>Ctrl Shift 8</kbd> <kbd>Ctrl Shift 7</kbd></td><td>In a note: bold, italic, bulleted list, numbered list</td></tr>
      <tr><td><kbd>Ctrl Enter</kbd> <kbd>Shift F10</kbd></td><td>In a note: go to the paragraph's passage; the paragraph menu</td></tr>
      <tr><td><kbd>Ctrl Shift C</kbd></td><td>Commentary panel</td></tr>
      <tr><td><kbd>Ctrl J</kbd></td><td>Study panel</td></tr>
      <tr><td><kbd>F6</kbd> <kbd>Shift F6</kbd></td><td>Next / previous region</td></tr>
      <tr><td><kbd>←</kbd> <kbd>→</kbd> <kbd>Home</kbd> <kbd>End</kbd></td><td>On the right column's side splitter: resize, smallest, largest</td></tr>
      <tr><td><kbd>↑</kbd> <kbd>↓</kbd> <kbd>Home</kbd> <kbd>End</kbd> <kbd>Enter</kbd></td><td>On the divider between Commentary and Lexicon: move (Shift: bigger step), top, bottom, reset</td></tr>
    </tbody>
  </table>
  <div class="foot"><button type="button" class="btn btn-outline" onclick={() => keysDialog?.close()}>Close</button></div>
</dialog>
