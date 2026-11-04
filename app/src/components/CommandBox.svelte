<script lang="ts">
  // Command box (reader.md 5; wave-2-layout.md 6): ARIA 1.2 combobox with a
  // list popup. A reference goes there; words are searched; a bare book name
  // shows both. Results are whole verses read from kjv.sqlite by the core,
  // with match emphasis as presentation-only <mark> (verse-dom.ts).
  import { onDestroy, tick } from "svelte";
  import { api } from "../lib/api";
  import { echo, parseReference, refLabel, type Parse, type Ref } from "../lib/reference";
  import { PAGE, SearchSession, searchable, startsWithQuote } from "../lib/search-session";
  import { resultText } from "../lib/verse-dom";
  import type { Book, SearchHit } from "../lib/types";
  import { bookmarkWords, type Bookmark } from "../lib/bookmarks";
  import Ribbon from "./Ribbon.svelte";

  interface Props {
    books: Book[];
    location: string;
    /** translation abbreviation ("KJV"): the core's search names it */
    translation: string;
    /** null: search works; otherwise why it can't (index missing, damaged or stale) */
    searchError: string | null;
    onGo: (r: Ref) => void;
    /** A result chosen in the dropdown (wave-2-layout 6.8). */
    onJump: (hit: SearchHit) => void;
    onOpenPicker: () => void;
    onLeave: () => void;
    /** Polite live region. */
    onSay: (text: string) => void;
    pickerOpen: boolean;
    /** by OSIS book ID (bookmarks.md 4.3) */
    bookmarks: Record<string, Bookmark>;
    onClearBookmark: (osis: string) => void;
    /** The field gained or lost focus (the bookmark clock pauses while it has it). */
    onFocusChange: (focused: boolean) => void;
  }
  let { books, location, translation, searchError, onGo, onJump, onOpenPicker, onLeave, onSay, pickerOpen, bookmarks, onClearBookmark, onFocusChange }: Props = $props();

  type GoRow = {
    kind: "go"; id: string; label: string; sub?: string; ref: Ref; book: boolean;
    /** a bookmarked book row: its OSIS ID and words (bookmarks.md 4.3, 7) */
    bm?: { osis: string; sr: string };
  };
  type HitRow = { kind: "hit"; id: string; hit: SearchHit; index: number };
  type MoreRow = { kind: "more"; id: string };
  type Opt = GoRow | HitRow | MoreRow;

  const SEARCH_DELAY = 150; // --search-delay
  const SLOW = 300; // "Verses · searching…" after this
  const ANNOUNCE = 500; // live region, after typing stops
  const ENTER_WAIT = 1000; // Enter waits at most this long for a running search

  let input: HTMLInputElement;
  let listEl = $state<HTMLDivElement>();
  let focused = $state(false);
  let text = $state("");
  let listOpen = $state(false);
  let active = $state(-1);
  let parsed = $state<Parse>({ kind: "empty" });
  let edited = $state(false);
  /** Search mode (the chip): Ctrl+F or a leading quote mark (6.3). */
  let forced = $state(false);
  /** The last completed search (shown while the text still searches; Ctrl+F and F3 use it). */
  let session = $state<SearchSession | null>(null);
  /** Bumped when the session changes in place (paging), so the rows re-derive. */
  let version = $state(0);
  let slow = $state(false);
  let failed = $state<string | null>(null);
  let focusIntent: "search" | null = null;
  /** The words' date is computed when the dropdown opens (bookmarks.md 5). */
  let openedAt = $state(new Date());
  /** The one-item context menu of a bookmarked Go to row. */
  let menu = $state<{ osis: string; x: number; y: number } | null>(null);

  let debounce = 0;
  let slowTimer = 0;
  let announceTimer = 0;
  let lastInput = 0;
  let token = 0;
  /** The query waiting for its debounce, if any. */
  let queued: string | null = null;
  /** The search in flight, if any. */
  let pending: Promise<void> | null = null;

  const searchMode = $derived(forced || startsWithQuote(text));
  const ambiguous = $derived(!searchMode && parsed.kind === "books");
  /** Whether the Verses section belongs in the dropdown for this text. */
  const wantsVerses = $derived(
    edited && searchable(text) && (searchMode || parsed.kind === "books" || parsed.kind === "text"),
  );
  const unavailable = $derived(searchError ?? session?.unavailable ?? null);

  const goRows = $derived.by<GoRow[]>(() => {
    if (searchMode) return [];
    if (parsed.kind === "books")
      return parsed.books.map((b, i) => {
        const m = bookmarks[b.osis];
        const row: GoRow = {
          kind: "go", id: `cmd-opt-go-${i}`, label: b.name, book: true,
          sub: `${b.chapters} ${b.chapters === 1 ? "chapter" : "chapters"}`,
          ref: { book: b, chapter: 1, verse: null },
        };
        if (m && m.chapter <= b.chapters) {
          // Enter goes to the bookmarked chapter, and the row says so (Q35)
          const w = bookmarkWords(b.name, m, b.chapters === 1, openedAt);
          row.sub = w.rowSub;
          row.ref = { book: b, chapter: m.chapter, verse: null };
          row.bm = { osis: b.osis, sr: w.bookSr };
        }
        return row;
      });
    if (parsed.kind === "refs")
      return parsed.refs.map((r, i) => ({ kind: "go", id: `cmd-opt-go-${i}`, label: refLabel(r), ref: r, book: false }));
    return [];
  });

  const hitRows = $derived.by<HitRow[]>(() => {
    void version;
    if (!wantsVerses || unavailable || !session) return [];
    return session.shownHits.map((hit, index) => ({ kind: "hit", id: `cmd-opt-hit-${index}`, hit, index }));
  });
  const moreRow = $derived.by<MoreRow | null>(() => {
    void version;
    return hitRows.length && session?.hasMore ? { kind: "more", id: "cmd-opt-more" } : null;
  });
  const opts = $derived<Opt[]>([...goRows, ...hitRows, ...(moreRow ? [moreRow] : [])]);
  const activeOpt = $derived(active >= 0 ? opts[active] : undefined);
  /** The Verses section: results, or a message in its place. */
  const versesState = $derived.by<"none" | "unavailable" | "searching" | "results" | "empty">(() => {
    void version;
    if (!wantsVerses) return "none";
    if (unavailable) return "unavailable";
    if (failed) return "unavailable";
    if (!session) return slow ? "searching" : "none";
    if (session.total > 0) return "results";
    return ambiguous ? "none" : "empty";
  });

  export function focusBox() {
    input.focus();
    input.select();
  }

  /** Ctrl+F: search mode; after an earlier search, that query with its results (6.3). */
  export function focusSearch() {
    focusIntent = "search";
    if (document.activeElement === input) applySearchFocus();
    else input.focus();
  }

  function applySearchFocus() {
    focusIntent = null;
    forced = true;
    if (session && session.total > 0 && !session.unavailable) {
      text = session.query;
      edited = true;
      parsed = { kind: "text" };
      listOpen = true;
      version++;
      active = Math.max(0, session.last);
      tick().then(scrollActive);
    } else {
      text = "";
      edited = false;
      parsed = { kind: "empty" };
      listOpen = false;
      active = -1;
    }
    queueMicrotask(() => input.select());
  }

  /** F3 / Shift+F3 outside the field: the next / previous result of the last search (6.7). */
  export async function searchStep(dir: 1 | -1) {
    if (!session || session.total === 0) return null;
    const r = await session.step(dir);
    version++;
    return r ? { ...r, total: session.total } : null;
  }

  function onfocus() {
    focused = true;
    openedAt = new Date();
    onFocusChange(true);
    if (focusIntent === "search") { applySearchFocus(); return; }
    edited = false;
    forced = false;
    text = location;
    listOpen = false;
    queueMicrotask(() => input.select());
  }

  function onblur() {
    focused = false;
    menu = null;
    onFocusChange(false);
    listOpen = false;
    forced = false;
    text = location;
    cancelQueued();
  }

  function cancelQueued() {
    clearTimeout(debounce);
    clearTimeout(announceTimer);
    queued = null;
  }

  function oninput() {
    edited = true;
    lastInput = performance.now();
    parsed = searchMode ? { kind: "text" } : parseReference(text, books);
    active = goRows.length ? 0 : -1;
    listOpen = true;
    clearTimeout(announceTimer);
    if (wantsVerses) {
      queued = text;
      clearTimeout(debounce);
      debounce = window.setTimeout(fire, SEARCH_DELAY);
    } else {
      cancelQueued();
    }
  }

  /** Runs the queued search now (the debounce elapsed, or Enter). */
  function fire(): Promise<void> | null {
    clearTimeout(debounce);
    const q = queued;
    queued = null;
    if (q === null) return pending;
    const t = ++token;
    clearTimeout(slowTimer);
    slowTimer = window.setTimeout(() => { if (t === token) slow = true; }, SLOW);
    const run = (async () => {
      try {
        const s = await SearchSession.start(q, (query, offset) => api.search(translation, query, "bible", offset, PAGE));
        if (t !== token) return;
        failed = null;
        session = s;
        version++;
        if (!goRows.length) active = s.total > 0 ? 0 : -1;
        scheduleAnnounce(t);
      } catch (e) {
        if (t === token) failed = String(e);
      } finally {
        if (t === token) { clearTimeout(slowTimer); slow = false; pending = null; }
      }
    })();
    pending = run;
    return run;
  }

  function scheduleAnnounce(t: number) {
    clearTimeout(announceTimer);
    const wait = Math.max(0, lastInput + ANNOUNCE - performance.now());
    announceTimer = window.setTimeout(() => {
      if (t !== token || !listOpen || !session || !wantsVerses) return;
      const s = session;
      if (s.unavailable) return;
      if (ambiguous && goRows[0]) {
        if (s.total > 0) onSay(`${goRows[0].label}, a book. ${s.total} ${s.total === 1 ? "verse" : "verses"}.`);
      } else if (s.total === 1) {
        onSay(`1 verse: ${s.shownHits[0]?.reference ?? ""}`);
      } else if (s.total > 1) {
        onSay(`${s.total} verses. Showing ${s.shown}.`);
      } else {
        onSay(`No verses in the ${translation} contain “${echo(s.query.trim())}”.`);
      }
    }, wait);
  }

  function scrollActive() {
    const id = activeOpt?.id;
    const el = id ? document.getElementById(id) : null;
    el?.scrollIntoView?.({ block: "nearest" });
  }

  function setActive(i: number) {
    if (!opts.length) { active = -1; return; }
    active = Math.max(0, Math.min(opts.length - 1, i));
    tick().then(scrollActive);
  }

  /** How many options fit in the dropdown's height (PgUp / PgDn). */
  function pageSize(): number {
    const list = listEl?.closest<HTMLElement>(".cmd-pop");
    const rows = listEl ? Array.from(listEl.querySelectorAll<HTMLElement>('[role="option"]')) : [];
    const h = list?.clientHeight ?? 0;
    const avg = rows.length ? rows.reduce((n, r) => n + r.offsetHeight, 0) / rows.length : 0;
    return h > 0 && avg > 0 ? Math.max(1, Math.floor(h / avg)) : 5;
  }

  async function act(o: Opt) {
    if (o.kind === "go") {
      listOpen = false;
      onGo(o.ref);
    } else if (o.kind === "hit") {
      if (session) session.last = o.index;
      listOpen = false;
      onJump(o.hit);
    } else if (session) {
      const first = session.shown;
      await session.more();
      version++;
      await tick();
      const i = opts.findIndex((x) => x.kind === "hit" && x.index === first);
      if (i >= 0) active = i;
      // keep the scroll position: the new rows are appended below
      onSay(`${session.shown} of ${session.total} verses shown.`);
    }
  }

  async function enter() {
    if (activeOpt?.kind === "go" || (activeOpt && !queued && !pending)) { act(activeOpt); return; }
    if (!wantsVerses) { if (activeOpt) act(activeOpt); return; }
    // a search is still running: wait for it (at most 1s), then the first result
    const run = fire();
    if (run) await Promise.race([run, new Promise((r) => setTimeout(r, ENTER_WAIT))]);
    const first = opts.find((o) => o.kind === "go" || o.kind === "hit");
    if (first) act(first);
  }

  function onkeydown(e: KeyboardEvent) {
    const unchanged = !edited || text.trim() === "";
    const k = e.key;
    if (k === "ArrowDown") {
      e.preventDefault();
      if (e.altKey || (unchanged && !listOpen && !forced) || (listOpen && opts.length === 0 && unchanged && !forced)) {
        listOpen = false;
        onOpenPicker();
      } else if (opts.length) {
        listOpen = true;
        setActive(active + 1);
      }
    } else if (k === "ArrowUp") {
      if (opts.length) { e.preventDefault(); setActive(active - 1); }
    } else if (k === "PageDown" || k === "PageUp") {
      if (listOpen && opts.length) { e.preventDefault(); setActive(active + (k === "PageDown" ? 1 : -1) * pageSize()); }
    } else if ((k === "Home" || k === "End") && (e.ctrlKey || e.metaKey)) {
      if (listOpen && opts.length) { e.preventDefault(); setActive(k === "Home" ? 0 : opts.length - 1); }
    } else if (k === "Enter") {
      e.preventDefault();
      if (listOpen || edited) enter();
    } else if (k === "Escape" && !menu) {
      e.preventDefault();
      cancelQueued();
      forced = false;
      text = location;
      listOpen = false;
      onLeave();
    } else if (k === "Delete" && e.shiftKey && listOpen && activeOpt?.kind === "go" && activeOpt.bm) {
      // Shift+Delete clears the active row's bookmark (bookmarks.md 6); focus and the dropdown stay
      e.preventDefault();
      onClearBookmark(activeOpt.bm.osis);
    } else if (k === "Escape" && menu) {
      e.preventDefault();
      menu = null;
    } else if (k === "Backspace" && forced && text === "") {
      e.preventDefault();
      forced = false;
      parsed = { kind: "empty" };
    }
  }

  onDestroy(() => { cancelQueued(); clearTimeout(slowTimer); });

</script>

<div class="cmd">
  <div class="cmd-field" class:searching={searchMode && focused}>
    <svg class="i s16 cmd-icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="M20 20l-4-4" /></svg>
    {#if searchMode && focused}<span class="cmd-chip" aria-hidden="true">Search</span>{/if}
    <input
      bind:this={input}
      class="cmd-input"
      class:editing={focused}
      id="cmd-input"
      type="text"
      autocomplete="off"
      spellcheck="false"
      maxlength="200"
      role="combobox"
      aria-autocomplete="list"
      aria-expanded={listOpen}
      aria-controls="cmd-pop"
      aria-activedescendant={listOpen && activeOpt ? activeOpt.id : undefined}
      aria-invalid={listOpen && parsed.kind === "error" && !searchMode ? "true" : undefined}
      aria-describedby={listOpen && parsed.kind === "error" && !searchMode ? "cmd-err" : undefined}
      aria-label={searchMode && focused ? "Search the Bible" : "Go to a passage or search"}
      placeholder={searchMode && focused ? `Words or a "phrase" in the ${translation}` : "Go to a passage or search"}
      value={focused ? text : location}
      oninput={(e) => { text = e.currentTarget.value; oninput(); }}
      {onfocus}
      {onblur}
      {onkeydown}
    />
    <kbd class="cmd-kbd" aria-hidden="true">Ctrl L</kbd>
    <button
      type="button"
      class="ibtn sm cmd-chev"
      aria-label="Choose book and chapter"
      aria-haspopup="dialog"
      aria-expanded={pickerOpen}
      title="Choose book and chapter · ↓"
      onclick={onOpenPicker}
    >
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" /></svg>
    </button>
  </div>
  {#if listOpen}
    <div class="pop cmd-pop" id="cmd-pop">
      {#if parsed.kind === "empty" && !searchMode}
        <div class="cmd-hint">
          <div>Type a reference or words to search</div>
          <div class="ex"><code>jn 3 16</code><code>Romans 8</code><code>jesus wept</code></div>
          <div class="keys">↵ Go · Esc Cancel · ↓ Book list</div>
        </div>
      {:else if parsed.kind === "error" && !searchMode}
        <div class="cmd-err" id="cmd-err" role="status">
          <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" /></svg>
          <div>
            <div class="l1">{parsed.line1}</div>
            {#if parsed.line2}<div class="l2">{parsed.line2}</div>{/if}
          </div>
        </div>
      {:else}
        {#snippet goRow(row: GoRow, i: number)}
          <div
            class="opt-row"
            id={row.id}
            role="option"
            aria-selected={i === active}
            tabindex="-1"
            aria-keyshortcuts={row.bm ? "Shift+Delete" : undefined}
            onpointerdown={(e) => { e.preventDefault(); if (e.button === 0) act(row); }}
            onpointermove={() => { if (active !== i) active = i; }}
            oncontextmenu={(e) => {
              e.preventDefault();
              if (!row.bm) return;
              const o = (e.currentTarget as HTMLElement).closest(".cmd-pop")!.getBoundingClientRect();
              active = i;
              menu = { osis: row.bm.osis, x: e.clientX - o.left, y: e.clientY - o.top };
            }}
          >
            <span class="label">{row.label}</span>
            {#if row.bm}<Ribbon /><span class="sr-only">{row.bm.sr}</span>{/if}
            {#if row.sub}<span class="sub" aria-hidden={row.bm ? "true" : undefined}>{row.sub}</span>{/if}
            <span class="end"><kbd>↵</kbd></span>
          </div>
        {/snippet}
        <div class="cmd-list" role="listbox" aria-label="Go to or search" bind:this={listEl}>
          {#if goRows.length && !ambiguous}
            {#each goRows as row, i (row.id)}{@render goRow(row, i)}{/each}
          {:else if goRows.length}
            <div class="cmd-sec" role="group" aria-labelledby="cmd-h-go">
              <div class="cmd-h" id="cmd-h-go" role="presentation">Go to</div>
              {#each goRows as row, i (row.id)}{@render goRow(row, i)}{/each}
            </div>
          {/if}
          {#if versesState === "results" && session}
            <div class="cmd-sec" role="group" aria-labelledby="cmd-h-verses">
              <div class="cmd-h" id="cmd-h-verses" role="presentation">Verses · <span class="num">{session.total}</span> in the {translation}</div>
              {#each hitRows as row, j (row.id)}
                {@const i = goRows.length + j}
                {#if session.exactTotal > 0 && session.exactTotal < session.total && !session.phrase && (row.index === 0 || row.index === session.exactTotal)}
                  <div class="cmd-subh" role="presentation">
                    {row.index === 0 ? `Exact phrase · ${session.exactTotal}` : `All the words · ${session.total - session.exactTotal}`}
                  </div>
                {/if}
                <div
                  class="opt-row cmd-result"
                  id={row.id}
                  role="option"
                  aria-selected={i === active}
                  tabindex="-1"
                  onpointerdown={(e) => { e.preventDefault(); act(row); }}
                  onpointermove={() => { if (active !== i) active = i; }}
                >
                  <div class="ref">{row.hit.reference}<kbd>↵</kbd></div>
                  <div class="rtext" use:resultText={row.hit.runs}></div>
                </div>
              {/each}
              {#if moreRow}
                {@const i = goRows.length + hitRows.length}
                <div
                  class="opt-row cmd-more"
                  id={moreRow.id}
                  role="option"
                  aria-selected={i === active}
                  tabindex="-1"
                  onpointerdown={(e) => { e.preventDefault(); act(moreRow); }}
                  onpointermove={() => { if (active !== i) active = i; }}
                >
                  <span class="label">Show {Math.min(PAGE, session.total - session.shown)} more</span>
                  <span class="sub num">{session.shown} of {session.total}</span>
                  <span class="end"><kbd>↵</kbd></span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
        {#if menu}
          {@const m = menu}
          <!-- the field keeps focus (the dropdown stays open): the item acts on pointerdown -->
          <div class="pop bm-menu" role="menu" aria-label="Bookmark" style:left="{m.x}px" style:top="{m.y}px">
            <div class="mi" role="menuitem" tabindex="-1"
              onpointerdown={(e) => { e.preventDefault(); menu = null; onClearBookmark(m.osis); }}>
              <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2Z" /><path d="m14.5 7.5-5 5" /><path d="m9.5 7.5 5 5" /></svg>
              Clear bookmark
            </div>
          </div>
        {/if}
        {#if versesState === "searching"}
          <div class="cmd-h" role="status">Verses · searching…</div>
        {:else if versesState === "unavailable"}
          <div class="cmd-msg" role="status">
            <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" /></svg>
            <div>
              <div class="l1">Search is unavailable: its index is missing or damaged, so no results can be shown. The Bible text is not affected. Please repair or reinstall Revelation.</div>
              <div class="l2">Details: {unavailable ?? failed}</div>
            </div>
          </div>
        {:else if versesState === "empty" && session}
          <div class="cmd-msg" role="status">
            <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" /></svg>
            <div>
              <div class="l1">No verses in the {translation} contain “{echo(session.query.trim())}”.</div>
              <div class="l2">Check the spelling or try fewer words. To go to a passage, type a book and chapter, like Romans 8.</div>
            </div>
          </div>
        {:else if !opts.length && searchMode && !searchable(text)}
          <div class="cmd-hint"><div class="keys">Type at least 2 characters · Backspace leaves search</div></div>
        {/if}
      {/if}
    </div>
  {/if}
</div>
