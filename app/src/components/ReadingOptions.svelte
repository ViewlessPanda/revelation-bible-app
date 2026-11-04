<script lang="ts">
  // Reading options popover (reader.md 4).
  import { onMount } from "svelte";
  import type { Settings } from "../lib/types";

  type Toggle = "verse_numbers" | "red_letters" | "supplied_italics";
  interface Props {
    settings: Settings;
    hasRedLetters: boolean;
    sizePx: number;
    onToggle: (key: Toggle) => void;
    onSize: (delta: number) => void;
    onClose: () => void;
  }
  let { settings, hasRedLetters, sizePx, onToggle, onSize, onClose }: Props = $props();
  let root: HTMLDivElement;

  onMount(() => root.querySelector<HTMLElement>("[role=switch]")?.focus());

  const rows: { key: Toggle; label: string }[] = [
    { key: "verse_numbers", label: "Verse numbers" },
    { key: "red_letters", label: "Words of Jesus in red" },
    { key: "supplied_italics", label: "Supplied words in italics" },
  ];

  const disabled = (k: Toggle) => k === "red_letters" && !hasRedLetters;
  const checked = (k: Toggle) => (disabled(k) ? false : settings[k]);

  function act(k: Toggle) {
    if (!disabled(k)) onToggle(k);
  }
  function rowKey(e: KeyboardEvent, k: Toggle) {
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); act(k); }
  }
  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); onClose(); }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="pop options" role="dialog" aria-label="Reading options" tabindex="-1" bind:this={root} {onkeydown}>
  {#each rows as r (r.key)}
    <div
      class="row"
      role="switch"
      tabindex="0"
      aria-checked={checked(r.key)}
      aria-disabled={disabled(r.key) ? "true" : undefined}
      aria-describedby={disabled(r.key) ? "red-desc" : undefined}
      onclick={() => act(r.key)}
      onkeydown={(e) => rowKey(e, r.key)}
    >
      <span class="row-text">
        <span class="row-label">{r.label}</span>
        {#if disabled(r.key)}<span class="row-desc" id="red-desc">Not available in this translation</span>{/if}
        {#if r.key === "red_letters" && !disabled(r.key)}
          <!-- visible only under a Windows contrast theme (app.css, D5-F2) -->
          <span class="row-desc fc-note">Hidden by your Windows contrast theme</span>
        {/if}
      </span>
      <span class="sw" aria-hidden="true"></span>
    </div>
  {/each}
  <hr class="sep" />
  <div class="size-row">
    <span class="lbl">Text size</span>
    <span class="size-val">{sizePx} px</span>
    <button type="button" class="ibtn sm" aria-label="Smaller text" title="Smaller · Ctrl −"
      aria-disabled={settings.text_step <= 1 ? "true" : undefined} onclick={() => onSize(-1)}>
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14" /></svg>
    </button>
    <button type="button" class="ibtn sm" aria-label="Larger text" title="Larger · Ctrl +"
      aria-disabled={settings.text_step >= 10 ? "true" : undefined} onclick={() => onSize(1)}>
      <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14M12 5v14" /></svg>
    </button>
  </div>
</div>
