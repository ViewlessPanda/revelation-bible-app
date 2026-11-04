<script lang="ts" module>
  /** Position a popup from script (CSSOM; the CSP allows no style attributes). */
  export function place(node: HTMLElement, at: { x: number; y: number }) {
    const set = (a: { x: number; y: number }) => { node.style.setProperty("left", `${a.x}px`); node.style.setProperty("top", `${a.y}px`); };
    set(at);
    return { update: set };
  }
</script>

<script lang="ts">
  // The notes panel (wave-2-layout.md 8): daily notes on the left. One
  // document per local day, newest first, Today on top with the live
  // ProseMirror editor; earlier days are drawn read-only (renderStatic) until
  // clicked. One live view at a time. Saving is the store's saver (500ms after
  // a pause, on blur, on closing the panel or the app). Opening and closing
  // are App.svelte's (lib/panel-motion.ts, side "left"); this file draws.
  import { onMount, tick, untrack } from "svelte";
  import { EditorView } from "prosemirror-view";
  import { TextSelection } from "prosemirror-state";
  import { toggleMark } from "prosemirror-commands";
  import type { Node as PMNode } from "prosemirror-model";
  import type { Book } from "../lib/types";
  import type { PanelLayout } from "../lib/panel-motion";
  import { schema, tagText, type PassageTag } from "../lib/notes-schema";
  import { caretTags, endComposing, refreshTags, removeTag, setContext, tagWithContext, topBlockAt } from "../lib/notes-tags";
  import { copyText, docText, emptyDoc, listAt, markActive, notesState, renderStatic, toggleList } from "../lib/notes-editor";
  import { dayHeading, dayOrder, localDay } from "../lib/notes-days";
  import type { NotesStore } from "../lib/notes-store";
  import type { SaveStatus } from "../lib/notes-save";

  interface Props {
    store: NotesStore;
    books: Book[];
    /** the passage being read (8.4), shown as "Tagging …" */
    context: PassageTag | null;
    layout: PanelLayout;
    widthPx: number;
    minPx: number;
    maxPx: number;
    /** user.sqlite can't be used: notes can't be loaded or saved */
    unavailable: string | null;
    onClose: () => void;
    onJump: (t: PassageTag) => void;
    onResize: (px: number, done: boolean) => void;
    /** the clock (tests) */
    now?: () => Date;
  }
  let { store, books, context, layout, widthPx, minPx, maxPx, unavailable, onClose, onJump, onResize, now = () => new Date() }: Props = $props();

  const PLACEHOLDER = "Write about what you're reading…";
  let panel: HTMLElement;
  let body: HTMLDivElement;
  let closeBtn: HTMLButtonElement;
  let today = $state(localDay(new Date()));
  /** The day of the live editor once it holds a note (null: Today's, not started yet). */
  let liveDay = $state<string | null>(null);
  const editorDay = $derived(liveDay ?? today);
  /** Redraw static days and the day list (the store is plain data). */
  let version = $state(0);
  let status = $state<SaveStatus>({ kind: "saved" });
  let statusOpen = $state(false);
  /** The editor hit an error this session (W2A-N1): the screen may be ahead of the document. */
  let broke = false;
  let copied = $state("");
  let announce = $state("");
  let pressed = $state({ strong: false, em: false, bullet: false, ordered: false });
  let menu = $state<{ x: number; y: number; tags: PassageTag[] } | null>(null);
  let dragging = $state(false);
  let toolIndex = $state(0);
  let view: EditorView | null = null;
  let viewDay = "";
  let viewToken = $state(0);
  let hasView = $state(false);
  /** The store's load state, mirrored for the template (the store is plain data). */
  let loadError = $state<string | null>(null);
  let more = $state(false);
  let loaded = $state(false);
  /** Focus asked for before the editor existed (the first page still loading). */
  let pendingFocus = false;
  let menuEl = $state<HTMLDivElement>();

  const label = (t: PassageTag) => tagText(t, books);
  const entries = $derived.by(() => {
    void version;
    return dayOrder(store.days(), today, liveDay).map((day) => ({ day, doc: store.docs.get(day), h: dayHeading(day, today) }));
  });
  const ctxLabel = $derived(context ? label(context) : null);
  const narrow = $derived(widthPx < 18 * 16);

  export function element(): HTMLElement | null { return panel ?? null; }
  export function flush(): Promise<boolean> { return store.flush(); }
  /** Ctrl+Shift+N, the toggle: the caret at the end of today's note (5). */
  export async function focusToday() {
    if (editorDay !== today) { liveDay = store.docs.get(today)?.textContent ? today : null; await tick(); }
    if (!view) { pendingFocus = true; return; }
    const end = TextSelection.atEnd(view.state.doc);
    view.dispatch(view.state.tr.setSelection(end).scrollIntoView());
    view.focus();
  }
  /** Ctrl+Shift+N with Notes open: focus the editor as it was. */
  export function focusEditor() {
    if (view) view.focus(); else closeBtn?.focus({ preventScroll: true });
  }
  export function hasFocus(): boolean { return !!panel?.contains(document.activeElement); }

  // ---- the live editor ----
  function readToolbar() {
    if (!view) return;
    const s = view.state, l = listAt(s);
    pressed = { strong: markActive(s, "strong"), em: markActive(s, "em"), bullet: l === schema.nodes.bullet_list, ordered: l === schema.nodes.ordered_list };
  }
  function mountView(host: HTMLElement, day: string) {
    const stored = store.docs.get(day);
    const doc: PMNode = stored ?? emptyDoc();
    const layer = document.createElement("div");
    layer.className = "caret-layer";
    const v: EditorView = new EditorView(host, {
      state: notesState(doc, {
        tags: { label, jump: (t) => onJump(t), placeholder: day === today && !stored ? PLACEHOLDER : undefined },
        caret: { layer: () => layer, windowActive: () => (typeof document.hasFocus === "function" ? document.hasFocus() : true) },
        jump: (t) => { endComposing(v); onJump(t); },
      }),
      attributes: { class: "note-editor", "aria-label": `Note for ${dayHeading(day, today).full}`, "aria-multiline": "true", role: "textbox", spellcheck: "true" },
      dispatchTransaction(tr) {
        // W2A-N1: the change reaches the saver before anything is drawn, and
        // any error in the editor shows "Not saved" (never a false "Saved").
        let st;
        try { st = v.state.apply(tr); } catch (e) { editorFailed(e); return; }
        if (tr.docChanged) {
          // the first keystroke creates today's note; it keeps that day past midnight (8.1)
          if (liveDay === null) liveDay = day;
          const fresh = !store.docs.get(day)?.textContent;
          store.edit(day, st.doc);
          if (fresh) version++;
        }
        try { v.updateState(st); readToolbar(); } catch (e) { editorFailed(e); }
      },
      handleDOMEvents: {
        blur: () => { void store.flush(); return false; },
        keydown: (_v, e) => {
          if ((e.key === "F10" && e.shiftKey) || e.key === "ContextMenu") { e.preventDefault(); openMenu(); return true; }
          return false;
        },
      },
    });
    host.appendChild(layer);
    view = v;
    viewDay = day;
    hasView = true;
    setContext(v, context);
    readToolbar();
    viewToken++;
    if (pendingFocus) {
      pendingFocus = false;
      v.dispatch(v.state.tr.setSelection(TextSelection.atEnd(v.state.doc)));
      v.focus();
    }
  }
  /** The editor failed: say "Not saved" until the app closes; the text stays on screen to copy. */
  function editorFailed(e: unknown) {
    broke = true;
    store.saver.broken(`the editor hit an error (${String(e).slice(0, 120)})`);
  }
  /** "Copy this day's text" (W2A-N2): a day that can't be saved is never lost. */
  async function copyDay() {
    let text = "";
    if (view && !broke) text = docText(view.state.doc);
    else if (view) {
      // the editor's document may be behind the screen: copy what the reader sees
      const c = view.dom.cloneNode(true) as HTMLElement;
      c.querySelectorAll(".tag-row, .caret-layer").forEach((n) => n.remove());
      text = [...c.querySelectorAll("p")].map((p) => p.textContent ?? "").join("\n");
    } else {
      const d = store.docs.get(editorDay);
      if (d) text = docText(d);
    }
    copied = (await copyText(text)) ? "Copied. Paste it somewhere safe." : "Couldn't copy. Select the text and press Ctrl+C.";
    announce = copied;
  }
  function unmountView() {
    if (!view) return;
    view.destroy();
    view = null;
    viewDay = "";
    hasView = false;
    version++;
  }
  /** The live day's container (one at a time): the view is made here and destroyed with it. */
  function editorHost(node: HTMLElement, day: string) {
    mountView(node, day);
    return {
      update(d: string) { if (d !== viewDay) { unmountView(); node.replaceChildren(); mountView(node, d); } },
      destroy() { unmountView(); },
    };
  }

  // an earlier day: drawn read-only until clicked (8.1)
  function staticDay(node: HTMLElement, arg: { day: string; v: number }) {
    const draw = (a: { day: string; v: number }) => {
      const doc = store.docs.get(a.day);
      if (doc) renderStatic(node, doc, { label, jump: (t) => { if (view) endComposing(view); onJump(t); } });
      else node.replaceChildren();
    };
    draw(arg);
    return { update: draw };
  }
  async function openDay(day: string, at?: { x: number; y: number }) {
    if (day === editorDay) return;
    void store.flush();
    liveDay = day === today && !store.docs.get(today)?.textContent ? null : day;
    await tick();
    if (!view) return;
    let sel = TextSelection.atEnd(view.state.doc);
    if (at) {
      const p = view.posAtCoords({ left: at.x, top: at.y });
      if (p) sel = TextSelection.create(view.state.doc, p.pos);
    }
    view.dispatch(view.state.tr.setSelection(sel));
    view.focus();
  }
  function staticClick(day: string, e: MouseEvent) {
    if ((e.target as HTMLElement).closest(".tag")) return; // a chip
    openDay(day, { x: e.clientX, y: e.clientY });
  }
  function staticKey(day: string, e: KeyboardEvent) {
    if (e.key === "Enter" && e.target === e.currentTarget) { e.preventDefault(); openDay(day); }
  }

  // ---- toolbar (8.3): one Tab stop, roving with the arrow keys ----
  const tools = [
    { id: "strong", label: "Bold", keys: "Ctrl+B", run: () => toggleMark(schema.marks.strong) },
    { id: "em", label: "Italic", keys: "Ctrl+I", run: () => toggleMark(schema.marks.em) },
    { id: "bullet", label: "Bulleted list", keys: "Ctrl+Shift+8", run: () => toggleList(schema.nodes.bullet_list) },
    { id: "ordered", label: "Numbered list", keys: "Ctrl+Shift+7", run: () => toggleList(schema.nodes.ordered_list) },
  ] as const;
  function useTool(i: number) {
    if (!view) return;
    tools[i].run()(view.state, view.dispatch, view);
    view.focus();
  }
  function toolKey(e: KeyboardEvent) {
    const n = tools.length;
    const to = e.key === "ArrowRight" ? (toolIndex + 1) % n : e.key === "ArrowLeft" ? (toolIndex + n - 1) % n
      : e.key === "Home" ? 0 : e.key === "End" ? n - 1 : -1;
    if (to < 0) return;
    e.preventDefault();
    toolIndex = to;
    (e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>(".ntool")[to]?.focus();
  }

  // ---- the paragraph menu (Shift+F10, Menu key): Go to · Remove tag · Tag with the current passage ----
  function openMenu() {
    if (!view) return;
    const c = view.coordsAtPos(view.state.selection.head);
    const r = panel.getBoundingClientRect();
    menu = { x: Math.max(8, c.left - r.left), y: c.bottom - r.top + 4, tags: caretTags(view.state) };
    tick().then(() => menuEl?.querySelector<HTMLButtonElement>("button")?.focus());
  }
  function closeMenu() { menu = null; view?.focus(); }
  function menuAct(f: () => void) { f(); closeMenu(); }
  function menuKey(e: KeyboardEvent) {
    const items = [...(menuEl?.querySelectorAll<HTMLButtonElement>("button:not([disabled])") ?? [])];
    const i = items.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); closeMenu(); }
    else if (e.key === "ArrowDown") { e.preventDefault(); items[(i + 1) % items.length]?.focus(); }
    else if (e.key === "ArrowUp") { e.preventDefault(); items[(i - 1 + items.length) % items.length]?.focus(); }
    else if (e.key === "Tab") { e.preventDefault(); closeMenu(); }
  }
  const caretBlock = () => (view ? topBlockAt(view.state.doc, view.state.selection.head) : null);

  // ---- width: the splitter on the panel's right edge (wave-2-layout.md 1) ----
  function wDown(e: PointerEvent) {
    if (e.button) return;
    try { (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId); } catch { /* the move still arrives */ }
    dragging = true;
    document.body.classList.add("no-select");
  }
  function wMove(e: PointerEvent) {
    if (!dragging) return;
    onResize(Math.max(minPx, Math.min(maxPx, e.clientX - panel.getBoundingClientRect().left)), false);
  }
  function wUp() {
    if (!dragging) return;
    dragging = false;
    document.body.classList.remove("no-select");
    onResize(widthPx, true);
  }
  function wKey(e: KeyboardEvent) {
    const step = (e.shiftKey ? 4 : 1) * 16;
    const to = e.key === "ArrowRight" ? widthPx + step : e.key === "ArrowLeft" ? widthPx - step
      : e.key === "Home" ? minPx : e.key === "End" ? maxPx : null;
    if (to === null) return;
    e.preventDefault();
    onResize(Math.max(minPx, Math.min(maxPx, to)), true);
  }

  const statusText = (s: SaveStatus) => (s.kind === "saved" ? "Saved" : s.kind === "saving" ? "Saving…" : "Not saved");

  // width via CSSOM (allowed by the CSP; a style attribute would not be)
  $effect(() => { panel.style.setProperty("width", `${widthPx}px`); });
  // the reading context reaches the live editor (8.4)
  $effect(() => { void viewToken; const c = context; if (view) setContext(view, c); });
  // labels follow the installed books
  $effect(() => { void books; untrack(() => { if (view) refreshTags(view); version++; }); });

  onMount(() => {
    const offChange = store.onChange(() => { loadError = store.error; more = store.more; loaded = store.loaded; version++; });
    const offStatus = store.onStatus((s) => {
      const was = status.kind;
      status = s;
      if (s.kind === "failed" && was !== "failed") announce = "Your latest notes couldn't be saved. Revelation retries every 5 seconds.";
      else if (s.kind === "saved" && was === "failed") { announce = "Notes saved"; statusOpen = false; }
    });
    today = localDay(now());
    status = store.saver.status;
    loaded = store.loaded;
    if (!unavailable) void store.load();
    // a new Today appears after midnight (8.1); checked often and when the window comes back
    const tickDay = () => { const d = localDay(now()); if (d !== today) today = d; };
    const timer = window.setInterval(tickDay, 30_000);
    document.addEventListener("visibilitychange", tickDay);
    window.addEventListener("focus", tickDay);
    return () => {
      offChange(); offStatus();
      clearInterval(timer);
      document.removeEventListener("visibilitychange", tickDay);
      window.removeEventListener("focus", tickDay);
      void store.flush(); // closing the panel saves (8.6)
    };
  });
</script>

<aside class="npanel panel" class:drawer={layout === "drawer"} id="p-notes" aria-label="Notes" bind:this={panel}>
  <div class="panel-head">
    <span class="panel-title">Notes</span>
    <span class="nh-end">
      {#if !unavailable}
        {#if status.kind === "failed"}
          <span class="anchor">
            <button type="button" class="save-state failed" aria-expanded={statusOpen} onclick={() => (statusOpen = !statusOpen)}>
              <svg class="i s14" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="10" /><path d="M12 8v4M12 16h.01" /></svg>
              <span>Not saved</span>
            </button>
            {#if statusOpen}
              <div class="pop save-pop" role="status">
                Your latest changes couldn't be saved ({status.reason}). Revelation retries every 5 seconds; your text stays here.
                <button type="button" class="btn btn-outline copy-day" onclick={copyDay}>Copy this day's text</button>
                {#if copied}<span class="copied">{copied}</span>{/if}
              </div>
            {/if}
          </span>
        {:else}
          <span class="save-state" data-kind={status.kind}>
            {#if status.kind === "saved"}<svg class="i s12" viewBox="0 0 24 24" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>{/if}
            <span>{statusText(status)}</span>
          </span>
        {/if}
      {/if}
      <button type="button" class="ibtn sm" aria-label="Close notes panel" title="Close · Ctrl+Shift+N" bind:this={closeBtn} onclick={onClose}>
        <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
      </button>
    </span>
  </div>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="notes-toolbar" role="toolbar" aria-label="Formatting" tabindex="-1" onkeydown={toolKey}>
    {#each tools as t, i (t.id)}
      <button type="button" class="ibtn ntool" tabindex={i === toolIndex ? 0 : -1} aria-label={t.label} title="{t.label} · {t.keys}"
        aria-pressed={pressed[t.id]} disabled={!hasView} onmousedown={(e) => e.preventDefault()} onclick={() => { toolIndex = i; useTool(i); }}>
        {#if t.id === "strong"}
          <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8" /></svg>
        {:else if t.id === "em"}
          <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M19 4h-9M14 20H5M15 4 9 20" /></svg>
        {:else if t.id === "bullet"}
          <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M3 12h.01M3 18h.01M3 6h.01M8 12h13M8 18h13M8 6h13" /></svg>
        {:else}
          <svg class="i s16" viewBox="0 0 24 24" aria-hidden="true"><path d="M10 12h11M10 18h11M10 6h11M4 10h2M4 6h1v4M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" /></svg>
        {/if}
      </button>
    {/each}
    {#if ctxLabel}
      <span class="tagging" title="New paragraphs are tagged with the passage you're reading">
        {#if narrow}
          <svg class="i s16" viewBox="0 0 24 24" aria-label="Tagging {ctxLabel}" role="img"><path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z" /><path d="M7.5 7.5h.01" /></svg>
        {:else}Tagging {ctxLabel}{/if}
      </span>
    {/if}
  </div>
  <div class="panel-body notes-body" bind:this={body}>
    {#if unavailable}
      <p class="notes-msg" role="status">Notes can't be opened or saved right now: {unavailable}</p>
    {:else if loadError}
      <p class="notes-msg" role="status">Your notes can't be loaded: {loadError}</p>
    {:else if !loaded}
      <p class="notes-msg" role="status">Opening your notes…</p>
    {:else}
      {#each entries as { day, doc, h } (day)}
        <section class="day" aria-labelledby="dh-{day}">
          <h2 class="date-head" id="dh-{day}" aria-label={day === today ? `Today, ${h.full}` : h.full}>
            <span class="dh-name">{h.name}</span> <span class="dh-date">{h.date}</span>
          </h2>
          {#if doc === null}
            <p class="notes-msg">This day's note can't be read, so it isn't shown. It was left exactly as it was.</p>
          {:else if day === editorDay}
            <div class="note-live" use:editorHost={day}></div>
          {:else if !doc || !doc.textContent}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_static_element_interactions -->
            <div class="note-static note-empty" tabindex="0" aria-label="Write today's note" onclick={() => openDay(day)} onkeydown={(e) => staticKey(day, e)}>{PLACEHOLDER}</div>
          {:else}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_static_element_interactions -->
            <div class="note-static" tabindex="0" aria-label="Note for {h.full}; press Enter to edit"
              use:staticDay={{ day, v: version }} onclick={(e) => staticClick(day, e)} onkeydown={(e) => staticKey(day, e)}></div>
          {/if}
        </section>
      {/each}
      {#if more}
        <div class="notes-more"><button type="button" class="btn btn-outline" onclick={() => store.loadMore()}>Show earlier days</button></div>
      {/if}
    {/if}
  </div>
  {#if menu}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="pop menu-pop note-menu" role="menu" aria-label="Paragraph" bind:this={menuEl} onkeydown={menuKey}
      use:place={menu}>
      {#each menu.tags as t, i (i)}
        <button type="button" class="mi" role="menuitem" onclick={() => menuAct(() => { if (view) endComposing(view); onJump(t); })}>Go to {label(t)}</button>
      {/each}
      {#each menu.tags as t, i (i)}
        <button type="button" class="mi" role="menuitem" onclick={() => menuAct(() => { const b = caretBlock(); if (b && view) removeTag(view, b.index, i); })}>Remove tag {label(t)}</button>
      {/each}
      <button type="button" class="mi" role="menuitem" disabled={!ctxLabel}
        onclick={() => menuAct(() => { if (view) tagWithContext(view.state, view.dispatch); })}>Tag with the current passage{ctxLabel ? ` (${ctxLabel})` : ""}</button>
    </div>
  {/if}
  <div class="sr-only" aria-live="polite">{announce}</div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="splitter nsplit"
    class:dragging
    role="separator"
    aria-orientation="vertical"
    aria-controls="p-notes"
    aria-label="Resize notes panel"
    aria-valuenow={Math.round(widthPx)}
    aria-valuemin={Math.round(minPx)}
    aria-valuemax={Math.round(maxPx)}
    tabindex="0"
    onpointerdown={wDown}
    onpointermove={wMove}
    onpointerup={wUp}
    onpointercancel={wUp}
    onkeydown={wKey}
  ></div>
</aside>
