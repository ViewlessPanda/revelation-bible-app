<script lang="ts">
  // Study panel (reader.md 8; wave-2-layout.md 15 and 17). The bottom half of
  // the right column (RightColumn.svelte owns the column). Holds
  // the Strong's entries for the selected word, or for a cross-reference
  // clicked in an entry; otherwise the empty state. Each lexicon's required
  // attribution (SOURCES.md) is shown under its entries. Esc is handled by App.svelte (it closes the panel and clears the
  // selected word, wave-2-layout.md 5).
  import { LEXICON_CREDITS } from "../lib/attributions";
  import type { Lookup } from "../lib/types";
  import LexEntryView from "./LexEntryView.svelte";

  interface Props {
    lookup: Lookup;
    /** closing inside the column: lifted out of flow while it fades */
    lift?: boolean;
    onClose: () => void;
    /** open the entry a cross-reference points to */
    onRef: (key: string, from: string) => void;
  }
  let { lookup, lift = false, onClose, onRef }: Props = $props();

  const credits = $derived.by(() => {
    if (lookup.kind !== "done") return [];
    const ids: string[] = [];
    for (const r of lookup.results) if (r.status === "ok" && !ids.includes(r.entry.lexicon)) ids.push(r.entry.lexicon);
    return ids.map((id) => LEXICON_CREDITS[id]).filter((c) => c !== undefined);
  });

  let body: HTMLDivElement;
  let closeBtn: HTMLButtonElement;
  let shown = ""; // the entry last shown, so a new one cross-fades in

  export function focusClose() { closeBtn?.focus({ preventScroll: true }); }
  /** Ctrl+J and the toggle: the entry's heading when one is shown, else the close button (wave-2-layout.md 5). */
  export function focusStart() {
    const h = body?.querySelector<HTMLElement>(".lex-key");
    if (h) h.focus({ preventScroll: true }); else focusClose();
  }

  // A new entry (another word, or a cross-reference): the body scrolls to its
  // top and the entry cross-fades in place (--dur-2, opacity; reader.md 8,
  // red-letters 3.4.3 "Selected word").
  $effect(() => {
    if (lookup.kind !== "done") return;
    const id = `${lookup.title}|${lookup.keys.join(" ")}`;
    if (id === shown) return;
    const first = shown === "";
    shown = id;
    body.scrollTop = 0;
    const lex = body.querySelector<HTMLElement>(".lex");
    if (first || !lex || typeof lex.animate !== "function") return;
    const cs = getComputedStyle(document.documentElement);
    lex.animate([{ opacity: 0 }, { opacity: 1 }], {
      duration: parseFloat(cs.getPropertyValue("--dur-2")) || 120,
      easing: cs.getPropertyValue("--ease-out").trim() || "ease-out",
    });
  });
</script>

<aside class="panel" class:lift id="p-study" aria-label="Study panel">
  <div class="panel-head">
    <span class="panel-title">Lexicon</span>
    <button type="button" class="ibtn sm" aria-label="Close study panel" title="Close · Esc · Ctrl+J" bind:this={closeBtn} onclick={onClose}>
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
    </button>
  </div>
  <div class="panel-body" bind:this={body}>
    {#if lookup.kind === "empty"}
      <div class="empty">
        <svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="M3 5h6a3 3 0 0 1 3 3v12a2 2 0 0 0-2-2H3z" /><path d="M21 5h-6a3 3 0 0 0-3 3v12a2 2 0 0 1 2-2h7z" /></svg>
        <span class="t">Nothing here yet</span>
        <span class="d">Click a word in the text to see its Strong's entry. Commentary, dictionary and notes will open here later.</span>
      </div>
    {:else if lookup.kind === "loading"}
      <div class="lex"><p class="lex-msg" role="status">Looking up {lookup.keys.join(", ")}…</p></div>
    {:else if lookup.kind === "error"}
      <div class="lex"><p class="lex-msg" role="status">{lookup.message}</p></div>
    {:else}
      <div class="lex">
        <div class="lex-src">{lookup.title}</div>
        {#each lookup.results as r, i (i)}
          {#if i > 0}<hr class="lex-sep" />{/if}
          {#if r.status === "ok"}
            <LexEntryView entry={r.entry} onRef={(k) => onRef(k, r.entry.key)} />
          {:else if r.status === "not_found"}
            <p class="lex-msg">No lexicon entry for {r.key}.</p>
          {:else}
            <section class="lex-entry" aria-label="Strong's {lookup.keys[i]}">
              <div class="lex-top"><h2 class="lex-key" tabindex="-1">{lookup.keys[i]}</h2></div>
              <p class="lex-msg lex-unavailable">The lexicon is unavailable: its file is missing or damaged, so this Strong's entry can't be shown. The Bible text is not affected. Please repair or reinstall Revelation.</p>
              <p class="lex-msg detail">Details: {r.reason}</p>
            </section>
          {/if}
        {/each}
        {#each credits as c (c.id)}
          <footer class="lex-credit" aria-label="Source of {c.title}">{#each c.paragraphs as p, j (j)}<p>{p}</p>{/each}</footer>
        {/each}
      </div>
    {/if}
  </div>
</aside>
