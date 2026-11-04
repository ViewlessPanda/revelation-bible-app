<script lang="ts">
  // Commentary panel (wave-2-layout.md 7 and 17). The top half of the right
  // column (RightColumn.svelte owns the column, its width splitter and the
  // divider). Shows the comment on the verse being read (App.svelte passes it
  // in); the source bar shows the commentary's title and, only once the owner
  // has approved the labels, its author and date lines (pipeline/DESIGN.md
  // 17.7). The column slides in and out as one drawer (lib/panel-motion.ts).
  import { MHCC_CREDIT } from "../lib/attributions";
  import { contentKey, verseLabel, type CmtView, type VerseRef } from "../lib/commentary";
  import type { Book } from "../lib/types";
  import CommentaryView from "./CommentaryView.svelte";

  interface Props {
    view: CmtView;
    books: Book[];
    /** closing inside the column: lifted out of flow while it fades */
    lift?: boolean;
    onClose: () => void;
    onRef: (to: VerseRef) => void;
  }
  let { view, books, lift = false, onClose, onRef }: Props = $props();

  let body: HTMLDivElement;
  let heading: HTMLElement;
  let shown = ""; // the content last shown, so a new one fades in from the top

  /** Ctrl+Shift+C and the toggle put focus on the header (wave-2-layout.md 5). */
  export function focusStart() { heading?.focus({ preventScroll: true }); }

  // A new entry replaces the old with a 120ms opacity fade and the body
  // scrolls to its top (wave-2-layout.md 7). The same blocks (another verse of
  // the same anchor, or of a whole-chapter fallback) stay where they are.
  $effect(() => {
    if (view.kind !== "done") return;
    const id = contentKey(view.result);
    if (id === shown) return;
    const first = shown === "";
    shown = id;
    body.scrollTop = 0;
    const el = body.querySelector<HTMLElement>(".cmt");
    if (first || !el || typeof el.animate !== "function") return;
    const cs = getComputedStyle(document.documentElement);
    el.animate([{ opacity: 0 }, { opacity: 1 }], {
      duration: parseFloat(cs.getPropertyValue("--dur-2")) || 120,
      easing: cs.getPropertyValue("--ease-out").trim() || "ease-out",
    });
  });
</script>

<aside class="panel" class:lift id="p-comm" aria-label="Commentary panel">
  <div class="panel-head">
    <h2 class="panel-title" tabindex="-1" bind:this={heading}>Commentary</h2>
    <button type="button" class="ibtn sm" aria-label="Close commentary panel" title="Close · Ctrl+Shift+C" onclick={onClose}>
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
    </button>
  </div>
  {#if view.kind === "done" && view.result.status === "ok"}
    {@const r = view.result}
    <div class="cmt-source">
      <div class="cmt-title">{r.source.title}</div>
      {#if r.source.approved}
        {#if r.source.author_line}<div class="cmt-author">{r.source.author_line}</div>{/if}
        {#if r.source.date_line}<div class="cmt-date">{r.source.date_line}</div>{/if}
      {:else}
        <div class="cmt-pending">Labels awaiting review</div>
      {/if}
    </div>
  {/if}
  <div class="panel-body" bind:this={body}>
    {#if view.kind === "empty"}
      <div class="empty">
        <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 7v14" /><path d="M16 12h2" /><path d="M16 8h2" /><path d="M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z" /><path d="M6 12h2" /><path d="M6 8h2" /></svg>
        <span class="t">No verse selected</span>
        <span class="d">The commentary on the passage you're reading will appear here.</span>
      </div>
    {:else if view.kind === "loading"}
      <div class="cmt"><p class="lex-msg" role="status">Loading the commentary…</p></div>
    {:else if view.kind === "error"}
      <div class="cmt"><p class="lex-msg" role="status">{view.message}</p></div>
    {:else if view.result.status === "unavailable"}
      <div class="cmt">
        <p class="lex-msg lex-unavailable">This commentary is unavailable: its file is missing or damaged, so it can't be shown. The Bible text is not affected. Please repair or reinstall Revelation.</p>
        <p class="lex-msg detail">Details: {view.result.reason}</p>
      </div>
    {:else}
      {@const r = view.result}
      <div class="cmt">
        {#if r.groups.length === 0}
          <p class="lex-msg">{r.source.title} has no note on {verseLabel(r.verse_id, books)}.</p>
        {:else}
          {#if r.whole_chapter}
            <p class="lex-msg cmt-fallback">No note is listed for {verseLabel(r.verse_id, books)} itself, so this is the comment on the whole chapter.</p>
          {/if}
          <CommentaryView groups={r.groups} verseId={r.verse_id} {books} {onRef} />
        {/if}
        <!-- SOURCES.md CMT-MHCC-CW20 condition 3: the About / Sources text stays with the commentary -->
        <footer class="lex-credit cmt-credit" aria-label="Source of the commentary" data-source={MHCC_CREDIT.id}>{#each MHCC_CREDIT.paragraphs as p, j (j)}<p>{p}</p>{/each}</footer>
      </div>
    {/if}
  </div>
</aside>
