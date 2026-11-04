<script lang="ts">
  // Book and chapter picker (reader.md 6), with automatic bookmarks
  // (bookmarks.md 4.1, 4.2, 5, 6, 7): a ribbon on the bookmarked book and
  // chapter, its tooltip, and "Clear bookmark" from a context menu or
  // Shift+Delete.
  import { onMount, tick } from "svelte";
  import type { Book } from "../lib/types";
  import { bookmarkWords, type Bookmark } from "../lib/bookmarks";
  import Ribbon from "./Ribbon.svelte";

  interface Props {
    books: Book[];
    currentOrd: number;
    currentChapter: number;
    /** by OSIS book ID, read when the picker opens */
    bookmarks: Record<string, Bookmark>;
    onChoose: (ord: number, chapter: number) => void;
    onClose: () => void;
    onClearBookmark: (osis: string) => void;
  }
  let { books, currentOrd, currentChapter, bookmarks, onChoose, onClose, onClearBookmark }: Props = $props();

  let root: HTMLDivElement;
  let view = $state<{ kind: "books" } | { kind: "chapters"; book: Book }>({ kind: "books" });
  let viewDir = $state(1);
  let digits = "";
  let digitTimer = 0;
  /** The tooltip shown, and the cell it describes. */
  let tip = $state<{ cell: HTMLElement; text: string; x: number; y: number } | null>(null);
  let tipTimer = 0;
  /** The one-item context menu. */
  let menu = $state<{ cell: HTMLElement; osis: string; x: number; y: number } | null>(null);
  let menuItem = $state<HTMLButtonElement>();

  const ot = $derived(books.filter((b) => b.ord <= 39));
  const nt = $derived(books.filter((b) => b.ord >= 40));
  // the words are computed when the picker opens (5: a list left open across midnight keeps them)
  const now = new Date();
  /** A book's bookmark, if it can be shown (a chapter beyond the book is kept but not shown, 4.1). */
  const bmOf = (b: Book) => {
    const m = bookmarks[b.osis];
    return m && m.chapter <= b.chapters ? m : undefined;
  };
  const words = (b: Book, m: Bookmark) => bookmarkWords(b.name, m, b.chapters === 1, now);

  function cells(): HTMLButtonElement[] {
    return Array.from(root.querySelectorAll<HTMLButtonElement>(".pk-grid .pk-cell"));
  }

  async function focusCell(pred: (el: HTMLButtonElement) => boolean) {
    await tick();
    const cs = cells();
    const el = cs.find(pred) ?? cs[0];
    cs.forEach((c) => (c.tabIndex = -1));
    if (el) { el.tabIndex = 0; el.focus(); }
  }

  onMount(() => {
    focusCell((el) => el.dataset.ord === String(currentOrd));
    return () => clearTimeout(tipTimer);
  });

  function chooseBook(b: Book) {
    if (b.chapters === 1) { onChoose(b.ord, 1); return; }
    viewDir = 1;
    view = { kind: "chapters", book: b };
    // the current book: its current chapter; a bookmarked book: its bookmarked chapter (Q34); else 1
    const cur = b.ord === currentOrd ? currentChapter : (bmOf(b)?.chapter ?? 1);
    focusCell((el) => el.dataset.ch === String(cur));
  }

  function backToBooks() {
    const ord = view.kind === "chapters" ? view.book.ord : currentOrd;
    viewDir = -1;
    view = { kind: "books" };
    focusCell((el) => el.dataset.ord === String(ord));
  }

  function columns(cs: HTMLButtonElement[], idx: number): number {
    // cells of the same grid share offsetParent; count those on the same row as idx
    const grid = cs[idx].parentElement!;
    const inGrid = cs.filter((c) => c.parentElement === grid);
    const top = inGrid[0].offsetTop;
    let n = 0;
    for (const c of inGrid) { if (c.offsetTop === top) n++; else break; }
    return Math.max(1, n);
  }

  // ---- tooltip (5): 400ms after hover or keyboard focus; closes on blur, the next key, Esc ----
  function tipText(cell: HTMLElement): string | null {
    return cell.dataset.tip ?? null;
  }
  function scheduleTip(cell: HTMLElement) {
    clearTimeout(tipTimer);
    const text = tipText(cell);
    if (!text) { tip = null; return; }
    tipTimer = window.setTimeout(() => {
      const r = cell.getBoundingClientRect(), o = root.getBoundingClientRect();
      tip = { cell, text, x: r.left - o.left + root.scrollLeft, y: r.bottom - o.top + root.scrollTop + 4 };
    }, 400); // --delay-tooltip
  }
  function hideTip() {
    clearTimeout(tipTimer);
    tip = null;
  }

  // ---- clearing (6) ----
  async function openMenu(cell: HTMLElement, osis: string, at?: { x: number; y: number }) {
    hideTip();
    const o = root.getBoundingClientRect();
    const r = cell.getBoundingClientRect();
    menu = at
      ? { cell, osis, x: at.x - o.left + root.scrollLeft, y: at.y - o.top + root.scrollTop }
      : { cell, osis, x: r.left - o.left + root.scrollLeft, y: r.bottom - o.top + root.scrollTop + 4 };
    await tick();
    menuItem?.focus();
  }
  function closeMenu(returnFocus = true) {
    const cell = menu?.cell;
    menu = null;
    if (returnFocus) cell?.focus();
  }
  function clearFrom(cell: HTMLElement, osis: string) {
    onClearBookmark(osis); // focus stays on the cell; the picker stays open
    cell.focus();
  }

  function onkeydown(e: KeyboardEvent) {
    if (menu) {
      if (e.key === "Escape" || e.key === "Tab") { e.preventDefault(); e.stopPropagation(); closeMenu(); }
      return;
    }
    const cs = cells();
    const idx = cs.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => {
      const t = Math.max(0, Math.min(cs.length - 1, to));
      cs.forEach((c) => (c.tabIndex = -1));
      cs[t].tabIndex = 0;
      cs[t].focus();
    };
    if (e.key === "Escape") {
      e.preventDefault(); e.stopPropagation();
      if (tip) { hideTip(); return; } // Esc closes the tooltip first, then the picker
      onClose();
      return;
    }
    hideTip();
    if ((e.key === "Backspace" || (e.altKey && e.key === "ArrowLeft")) && view.kind === "chapters") {
      e.preventDefault(); backToBooks(); return;
    }
    if (idx < 0) return;
    const cell = cs[idx];
    const osis = cell.dataset.bm;
    if (osis && ((e.key === "F10" && e.shiftKey) || e.key === "ContextMenu")) { e.preventDefault(); openMenu(cell, osis); return; }
    if (osis && e.key === "Delete" && e.shiftKey) { e.preventDefault(); clearFrom(cell, osis); return; }
    const cols = columns(cs, idx);
    const gridStart = cs.findIndex((c) => c.parentElement === cs[idx].parentElement);
    const rowStart = idx - ((idx - gridStart) % cols);
    switch (e.key) {
      case "ArrowRight": e.preventDefault(); move(idx + 1); return;
      case "ArrowLeft": e.preventDefault(); move(idx - 1); return;
      case "ArrowDown": e.preventDefault(); move(idx + cols); return;
      case "ArrowUp": e.preventDefault(); move(idx - cols); return;
      case "Home": e.preventDefault(); move(e.ctrlKey ? 0 : rowStart); return;
      case "End": e.preventDefault(); move(e.ctrlKey ? cs.length - 1 : rowStart + cols - 1); return;
    }
    if (view.kind === "books" && /^[a-z0-9]$/i.test(e.key) && !e.ctrlKey && !e.altKey) {
      const k = e.key.toLowerCase();
      for (let j = 1; j <= cs.length; j++) {
        const c = cs[(idx + j) % cs.length];
        if ((c.dataset.name ?? "").toLowerCase().startsWith(k)) { move((idx + j) % cs.length); break; }
      }
      e.preventDefault();
    } else if (view.kind === "chapters" && /^[0-9]$/.test(e.key)) {
      e.preventDefault();
      digits += e.key;
      clearTimeout(digitTimer);
      digitTimer = window.setTimeout(() => (digits = ""), 800);
      const n = Number(digits);
      const t = cs.findIndex((c) => c.dataset.ch === String(n));
      if (t >= 0) move(t);
    }
  }

  /** Pointer and focus handlers shared by the cells (tooltip, context menu). */
  function cellEvents(osis: string | undefined) {
    return {
      onpointerenter: (e: PointerEvent) => scheduleTip(e.currentTarget as HTMLElement),
      onpointerleave: () => hideTip(),
      onfocus: (e: FocusEvent) => scheduleTip(e.currentTarget as HTMLElement),
      onblur: () => hideTip(),
      oncontextmenu: (e: MouseEvent) => {
        e.preventDefault();
        if (osis) openMenu(e.currentTarget as HTMLElement, osis, { x: e.clientX, y: e.clientY });
      },
    };
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="pop picker" role="dialog" aria-label="Choose book and chapter" tabindex="-1" bind:this={root} {onkeydown}>
  {#if view.kind === "books"}
    {#key view}
      <div class="pk-view" data-dir={viewDir}>
        {#each [{ h: "Old Testament", list: ot }, { h: "New Testament", list: nt }] as g (g.h)}
          {#if g.list.length}
            <div class="pk-group-h">{g.h}</div>
            <div class="pk-grid">
              {#each g.list as b (b.ord)}
                {@const m = bmOf(b)}
                <button
                  type="button"
                  class="pk-cell"
                  tabindex="-1"
                  data-ord={b.ord}
                  data-name={b.name}
                  data-bm={m ? b.osis : undefined}
                  data-tip={m ? words(b, m).bookTip : undefined}
                  aria-current={b.ord === currentOrd ? "true" : undefined}
                  aria-keyshortcuts={m ? "Shift+F10 Shift+Delete" : undefined}
                  aria-describedby={tip && tip.cell.dataset.ord === String(b.ord) ? "pk-tip" : undefined}
                  onclick={() => chooseBook(b)}
                  {...cellEvents(m ? b.osis : undefined)}
                >
                  {b.name}{#if m}<Ribbon /><span class="sr-only">{words(b, m).bookSr}</span>{/if}{#if b.ord === currentOrd}<span class="dot" aria-hidden="true"></span>{/if}
                </button>
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    {/key}
  {:else}
    {@const book = view.book}
    {@const m = bmOf(book)}
    {#key view}
      <div class="pk-view">
        <div class="pk-head">
          <button type="button" class="btn btn-quiet" onclick={backToBooks}>‹ Books</button>
          <span class="pk-title">{book.name}</span>
        </div>
        <div class="pk-grid chapters">
          {#each Array.from({ length: book.chapters }, (_, i) => i + 1) as ch (ch)}
            {@const marked = m && m.chapter === ch ? m : undefined}
            <button
              type="button"
              class="pk-cell"
              tabindex="-1"
              data-ch={ch}
              data-bm={marked ? book.osis : undefined}
              data-tip={marked ? words(book, marked).chapterTip : undefined}
              aria-label={marked ? words(book, marked).chapterLabel : `${book.name} ${ch}`}
              aria-current={book.ord === currentOrd && ch === currentChapter ? "true" : undefined}
              aria-keyshortcuts={marked ? "Shift+F10 Shift+Delete" : undefined}
              aria-describedby={tip && tip.cell.dataset.ch === String(ch) ? "pk-tip" : undefined}
              onclick={() => onChoose(book.ord, ch)}
              {...cellEvents(marked ? book.osis : undefined)}
            >{ch}{#if marked}<Ribbon />{/if}</button>
          {/each}
        </div>
      </div>
    {/key}
  {/if}
  {#if tip}
    <div class="bm-tip" id="pk-tip" role="tooltip" style:left="{tip.x}px" style:top="{tip.y}px">{tip.text}</div>
  {/if}
  {#if menu}
    {@const m = menu}
    <div class="pop bm-menu" role="menu" aria-label="Bookmark" style:left="{m.x}px" style:top="{m.y}px">
      <button type="button" class="mi" role="menuitem" bind:this={menuItem}
        onclick={() => { closeMenu(false); clearFrom(m.cell, m.osis); }}>
        <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2Z" /><path d="m14.5 7.5-5 5" /><path d="m9.5 7.5 5 5" /></svg>
        Clear bookmark
      </button>
    </div>
  {/if}
</div>
