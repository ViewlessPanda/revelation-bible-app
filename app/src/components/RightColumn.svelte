<script lang="ts">
  // The right column (wave-2-layout.md 17): Commentary on top, Study (Lexicon)
  // underneath, one column on the right window edge. It owns the column's
  // width splitter (the existing vertical splitter, once, on the inner edge)
  // and the horizontal divider between the panels. The numbers and keys are
  // lib/rcol.ts. Opening and closing are App.svelte's controllers: the column
  // slides as one drawer (lib/panel-motion.ts); a panel opening or closing
  // inside it is lib/rcol-motion.ts. This file only draws and measures.
  import type { Book, Lookup } from "../lib/types";
  import type { CmtView, VerseRef } from "../lib/commentary";
  import type { PanelLayout } from "../lib/panel-motion";
  import { RC, clampComm, clampWidth, commHeight, createDrag, dividerAria, dividerKey, splitOf, widthKey } from "../lib/rcol";
  import CommentaryPanel from "./CommentaryPanel.svelte";
  import StudyPanel from "./StudyPanel.svelte";

  interface Props {
    /** "drawer" while the column slides (out of flow), "track" at rest */
    layout: PanelLayout;
    /** each panel is in the DOM while open or fading out; `lift` = fading out, out of flow */
    commVis: boolean;
    studyVis: boolean;
    commLift: boolean;
    studyLift: boolean;
    view: CmtView;
    books: Book[];
    lookup: Lookup;
    widthPx: number;
    minPx: number;
    maxPx: number;
    /** Commentary's share of the column height less the divider (right_split) */
    split: number;
    onCloseComm: () => void;
    onCloseStudy: () => void;
    onResize: (px: number, done: boolean) => void;
    onResetWidth: () => void;
    /** the reader moved the divider (drag end, key, Enter, double-click): save this split */
    onSplit: (split: number) => void;
    onCmtRef: (to: VerseRef) => void;
    onLexRef: (key: string, from: string) => void;
  }
  let {
    layout, commVis, studyVis, commLift, studyLift, view, books, lookup, widthPx, minPx, maxPx, split,
    onCloseComm, onCloseStudy, onResize, onResetWidth, onSplit, onCmtRef, onLexRef,
  }: Props = $props();

  let rcol: HTMLElement;
  let cp: ReturnType<typeof CommentaryPanel> | undefined = $state();
  let sp: ReturnType<typeof StudyPanel> | undefined = $state();
  let wDragging = $state(false);
  let hDragging = $state(false);
  /** Commentary's clamped height for the ARIA values (updated at rest, not on every drag frame) */
  let ariaH = $state(0);
  let ariaAvail = $state(0);

  const both = $derived(commVis && studyVis && !commLift && !studyLift);
  const rem = () => parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
  const avail = () => (rcol?.clientHeight ?? 0) - RC.dividerPx;
  const aria = $derived(dividerAria(ariaH, ariaAvail, rem()));

  export function focusComm() { cp?.focusStart(); }
  export function focusStudy() { sp?.focusStart(); }
  export function element(): HTMLElement | null { return rcol ?? null; }
  /** The divider is being dragged with the pointer (Esc cancels it). */
  export function cancelDrag(): boolean {
    if (!drag.active) return false;
    drag.cancel();
    endDrag();
    return true;
  }
  /** Re-apply the split to the column's current height (window resize, open, close): the saved split is kept, only the px is re-clamped. */
  export function reflow() {
    const a = avail();
    if (!rcol || a <= 0) return;
    const h = commHeight(split, a, rem());
    rcol.style.setProperty("--rc-comm-h", `${h}px`);
    ariaAvail = a;
    ariaH = h;
  }

  // width via CSSOM (allowed by the CSP; a style attribute would not be)
  $effect(() => { rcol.style.setProperty("width", `${widthPx}px`); });
  // the split follows its own changes and the panels coming and going
  $effect(() => { void split; void both; void layout; reflow(); });

  // ---- width: the vertical splitter on the inner edge ----
  function wDown(e: PointerEvent) {
    try { (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId); } catch { /* no capture: the move still arrives */ }
    wDragging = true;
    document.body.classList.add("no-select");
  }
  function wMove(e: PointerEvent) {
    if (!wDragging) return;
    onResize(clampWidth(rcol.getBoundingClientRect().right - e.clientX, window.innerWidth, rem()), false);
  }
  function wUp() {
    if (!wDragging) return;
    wDragging = false;
    document.body.classList.remove("no-select");
    onResize(widthPx, true);
  }
  function wKey(e: KeyboardEvent) {
    const px = widthKey(e.key, e.shiftKey, widthPx, window.innerWidth, rem());
    if (px === null) return; // Enter is not bound here (17.1)
    e.preventDefault();
    onResize(px, true);
  }

  // ---- height: the divider (17.2) ----
  const currentH = () => commHeight(split, avail(), rem());
  const drag = createDrag({
    height: currentH,
    clamp: (h) => clampComm(h, avail(), rem()),
    write: (h) => rcol.style.setProperty("--rc-comm-h", `${h}px`),
    commit: (h) => onSplit(splitOf(h, avail())),
    raf: (cb) => requestAnimationFrame(cb),
    caf: (id) => cancelAnimationFrame(id),
  });
  function endDrag() {
    hDragging = false;
    document.documentElement.classList.remove("rc-drag");
    reflow();
  }
  function hDown(e: PointerEvent) {
    if (e.button) return;
    try { (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId); } catch { /* no capture: the move still arrives */ }
    drag.start(e.clientY);
    hDragging = true;
    document.documentElement.classList.add("rc-drag");
  }
  function hMove(e: PointerEvent) { drag.move(e.clientY); }
  function hUp(e: PointerEvent) {
    if (!drag.active) return;
    drag.end(e.clientY);
    endDrag();
  }
  function hKey(e: KeyboardEvent) {
    const to = dividerKey(e.key, e.shiftKey, currentH(), avail(), rem());
    if (to === null) return; // other keys keep their meaning (Esc, Tab, ...)
    e.preventDefault();
    onSplit(to === "reset" ? RC.splitDefault : splitOf(to, avail()));
  }
</script>

<div class="rcol" class:drawer={layout === "drawer"} class:two={both} id="rcol" bind:this={rcol}>
  <!-- the column's width splitter: a focusable separator is the ARIA window-splitter widget -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="splitter"
    class:dragging={wDragging}
    role="separator"
    aria-orientation="vertical"
    aria-controls="rcol"
    aria-label="Resize right column"
    aria-valuenow={Math.round(widthPx)}
    aria-valuemin={Math.round(minPx)}
    aria-valuemax={Math.round(maxPx)}
    tabindex="0"
    onpointerdown={wDown}
    onpointermove={wMove}
    onpointerup={wUp}
    onpointercancel={wUp}
    ondblclick={onResetWidth}
    onkeydown={wKey}
  ></div>
  {#if commVis}
    <CommentaryPanel bind:this={cp} {view} {books} lift={commLift} onClose={onCloseComm} onRef={onCmtRef} />
  {/if}
  {#if both}
    <!-- the divider between the panels: neither panel contains it (17.2) -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <div
      class="rc-div"
      class:dragging={hDragging}
      role="separator"
      aria-orientation="horizontal"
      aria-controls="p-comm"
      aria-label="Resize commentary and lexicon"
      aria-valuenow={aria.now}
      aria-valuemin={aria.min}
      aria-valuemax={aria.max}
      aria-valuetext={aria.text}
      tabindex="0"
      onpointerdown={hDown}
      onpointermove={hMove}
      onpointerup={hUp}
      onpointercancel={hUp}
      ondblclick={() => onSplit(RC.splitDefault)}
      onkeydown={hKey}
    ></div>
  {/if}
  {#if studyVis}
    <StudyPanel bind:this={sp} {lookup} lift={studyLift} onClose={onCloseStudy} onRef={onLexRef} />
  {/if}
</div>
