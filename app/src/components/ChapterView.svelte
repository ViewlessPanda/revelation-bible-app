<script lang="ts">
  // One chapter of verse text. DOM contract: red-letters-and-highlights.md
  // 3.4.1. Verse text goes in only through the `verseText` action (text
  // nodes only, never raw HTML). This component has NO style block on purpose: a
  // scoped class must never land on a verse-text element.
  // Range highlights (highlights-pen.md 7.2): a verse in `ink` gets
  // `data-ranges` and an empty `.hl-ink` AFTER `.vtext` (never inside it);
  // lib/ink.ts fills it with empty boxes. A verse in `ranges` (it has drawn
  // marks) gets ", pen highlight" in its screen-reader label. Nothing inside
  // `.vtext` changes.
  import { verseText } from "../lib/verse-dom";
  import { COLOR_NAMES, type Book, type Chapter, type HighlightColor } from "../lib/types";
  import { headingLabel } from "../lib/reference";

  interface Props {
    chapter: Chapter;
    book: Book;
    highlights: Record<number, HighlightColor>;
    currentVerse: number | null;
    /** verses with drawn range marks (the label suffix) */
    ranges?: Set<number>;
    /** verses with a paint layer: drawn marks or the live pen preview */
    ink?: Set<number>;
    prevLabel: string | null;
    nextLabel: string | null;
    onPrev: () => void;
    onNext: () => void;
  }
  let { chapter, book, highlights, currentVerse, ranges = new Set(), ink = new Set(), prevLabel, nextLabel, onPrev, onNext }: Props = $props();

  const hlTitle = (c: HighlightColor | undefined) =>
    c ? `Highlighted: ${COLOR_NAMES[c]} (${["gold", "green", "blue", "rose", "violet"].indexOf(c) + 1})` : undefined;
</script>

{#if chapter.chapter === 1 && book.main_title}
  <!-- the source's main title (book.main_title), exactly as stored, where the book opens -->
  <p class="book-title">{book.main_title}</p>
{/if}
<h1 class="chapter-head">{headingLabel(book, chapter.chapter)}</h1>
{#each chapter.verses as verse (verse.v)}
  <div
    class="verse"
    data-v={verse.v}
    data-hl={highlights[verse.v]}
    data-ranges={ink.has(verse.v) ? "" : undefined}
    title={hlTitle(highlights[verse.v])}
    aria-current={currentVerse === verse.v ? "true" : undefined}
  >
    <span class="vnum" aria-hidden="true">{verse.v}</span>
    <span class="sr-only">Verse {verse.v}{ranges.has(verse.v) ? ", pen highlight" : ""} </span>
    <span class="vtext" use:verseText={verse.runs}></span>{#if ink.has(verse.v)}<span class="hl-ink" aria-hidden="true"></span>{/if}
  </div>
{/each}
<nav class="chapter-end" aria-label="Chapters">
  {#if prevLabel}
    <button type="button" class="btn btn-quiet" onclick={onPrev}>‹ {prevLabel}</button>
  {:else}<span></span>{/if}
  {#if nextLabel}
    <button type="button" class="btn btn-quiet" onclick={onNext}>{nextLabel} ›</button>
  {:else}<span></span>{/if}
</nav>
