<script lang="ts">
  // The pen tray (highlights-pen.md 3.2): five colour radios, the eraser and
  // Done, under the top bar while pen mode is on. It never takes focus when it
  // opens; inside it, Left/Right move between its controls (roving tabindex)
  // and Tab leaves it.
  import { COLORS, COLOR_NAMES, type HighlightColor } from "../lib/types";

  interface Props {
    color: HighlightColor;
    eraser: boolean;
    onColor: (c: HighlightColor) => void;
    onEraser: () => void;
    onDone: () => void;
    /** a one-off message in place of the hint (4.1: the one-chapter clamp) */
    hint?: string | null;
    /** fading out (3.2: opacity over --dur-exit) */
    leaving?: boolean;
  }
  let { color, eraser, onColor, onEraser, onDone, hint = null, leaving = false }: Props = $props();

  let tray = $state<HTMLDivElement>();
  let active = $state(0); // index into the tray's controls for the roving tabindex
  const controls = () => Array.from(tray?.querySelectorAll<HTMLElement>("[data-roving]") ?? []);
  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    const list = controls();
    const i = list.indexOf(document.activeElement as HTMLElement);
    if (i < 0) return;
    e.preventDefault();
    const rtl = getComputedStyle(tray!).direction === "rtl";
    const step = (e.key === "ArrowRight") !== rtl ? 1 : -1;
    active = (i + step + list.length) % list.length;
    list[active].focus();
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div class="pen-tray" role="toolbar" aria-label="Highlighter" bind:this={tray} tabindex="-1" {onkeydown}
  class:leaving>
  <div role="radiogroup" aria-label="Pen colour">
    {#each COLORS as c, i (c)}
      <button type="button" class="swatch" data-c={c} role="radio" data-roving
        aria-checked={!eraser && color === c} aria-label={COLOR_NAMES[c]} title="{COLOR_NAMES[c]} · {i + 1}"
        tabindex={active === i ? 0 : -1} onpointerdown={(e) => e.preventDefault()} onclick={() => { active = i; onColor(c); }}>{i + 1}</button>
    {/each}
  </div>
  <button type="button" class="ibtn sm" aria-label="Eraser" aria-pressed={eraser} title="Eraser · E" data-roving
    tabindex={active === 5 ? 0 : -1} onpointerdown={(e) => e.preventDefault()} onclick={() => { active = 5; onEraser(); }}>
    <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21" /><path d="M22 21H7" /><path d="m5 11 9 9" /></svg>
  </button>
  <button type="button" class="btn btn-quiet pen-done" data-roving tabindex={active === 6 ? 0 : -1}
    onclick={onDone}>Done <kbd>Esc</kbd></button>
  <span class="pen-hint">{hint ?? "Drag over words · Alt for letters"}</span>
</div>
