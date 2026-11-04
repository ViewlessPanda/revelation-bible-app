<script lang="ts">
  // About Revelation (wave-2-layout.md 14; reader.md 2: app menu > About
  // Revelation). Everything is bundled, no network: the mark (public/brand,
  // made by scripts/make-icon.mjs from assets/brand/revelation-icon-1024.png),
  // the name set in type, the build's version, and two further views: Sources
  // and licences (every required attribution, lib/attributions.ts, checked
  // against sources/SOURCES.md by tests/about.test.ts) and the GNU GPL v3 text
  // (licenses/GPL-3.0.txt, shown as one text node).
  //
  // LOGO RIGHTS: the mark is not under the MIT License (assets/brand/LICENSE;
  // sources/SOURCES.md row BRAND-LOGO; made with ChatGPT).
  import { tick } from "svelte";
  import {
    APP_COPYRIGHT, APP_CREDIT, APP_DESCRIPTION, APP_LICENCE, APP_NAME, FONT_CREDITS, ICON_CREDIT, MHCC_CREDIT,
    KJV_CREDIT, KJV_LICENCE_NOTICE, MGNT_CREDIT, OSHB_CREDIT,
  } from "../lib/attributions";
  import GPL_TEXT from "../../../licenses/GPL-3.0.txt?raw";

  // Native modal <dialog>: the page behind is inert (focus stays inside),
  // Escape closes it. `onclose` lets the opener take focus back.
  let { onclose }: { onclose?: () => void } = $props();
  let dialog: HTMLDialogElement;
  let closeBtn = $state<HTMLButtonElement>();
  let heading = $state<HTMLHeadingElement>();
  let view = $state<"about" | "credits" | "gpl">("about");
  const version = __APP_VERSION__;

  export function open() {
    view = "about";
    dialog.showModal();
    dialog.scrollTop = 0;
    closeBtn?.focus(); // focus starts on Close (14)
  }
  async function show(v: typeof view) {
    view = v;
    await tick();
    dialog.scrollTop = 0;
    if (v === "about") closeBtn?.focus(); else heading?.focus();
  }

  const sections = [
    { heading: "Bible text", credits: [KJV_CREDIT] },
    { heading: "Lexicons", credits: [OSHB_CREDIT, MGNT_CREDIT] },
    { heading: "Commentary", credits: [MHCC_CREDIT] },
  ];
</script>

<dialog class="about" class:about-wide={view !== "about"} bind:this={dialog} aria-labelledby="about-title" aria-modal="true" {onclose}>
  {#if view === "about"}
    <div class="about-main">
      <img class="about-mark" src="brand/revelation-mark-192.png"
        srcset="brand/revelation-mark-192.png 1x, brand/revelation-mark-288.png 1.5x, brand/revelation-mark-384.png 2x"
        width="96" height="96" alt="" />
      <h2 id="about-title" class="about-name">{APP_NAME}</h2>
      <p class="about-version">Version {version}</p>
      <p class="about-desc">{APP_DESCRIPTION}</p>
      <p class="about-by">{APP_CREDIT}</p>
      <p class="about-notice">{ICON_CREDIT}</p>
    </div>
    <div class="about-foot">
      <button type="button" class="btn btn-outline" onclick={() => show("credits")}>Sources and licences</button>
      <button type="button" class="btn btn-primary" bind:this={closeBtn} onclick={() => dialog.close()}>Close</button>
    </div>
  {:else if view === "credits"}
    <h2 id="about-title" class="about-h" tabindex="-1" bind:this={heading}>Sources and licences</h2>
    <div class="about-body">
      <p>{APP_COPYRIGHT}. {APP_LICENCE}.</p>
      <p class="muted">The MIT License covers the code only. The Bible text, lexicons and fonts keep their own licences, credited below.</p>
      {#each sections as s (s.heading)}
        <h3>{s.heading}</h3>
        {#each s.credits as c (c.id)}
          <div class="about-credit" data-source={c.id}>{#each c.paragraphs as p, i (i)}<p>{p}</p>{/each}</div>
          {#if c.id === KJV_CREDIT.id}
            <p class="about-licence-notice"><strong>{KJV_LICENCE_NOTICE.label}</strong> {KJV_LICENCE_NOTICE.text}</p>
            <p><button type="button" class="btn btn-outline" onclick={() => show("gpl")}>Read the GNU General Public License</button></p>
          {/if}
        {/each}
      {/each}
      <h3>Fonts</h3>
      {#each FONT_CREDITS as f (f.id)}
        <p class="about-credit" data-source={f.id}>{f.name}: {f.copyright}. {f.licence}. {f.url}</p>
      {/each}
    </div>
    <div class="about-foot end">
      <button type="button" class="btn btn-outline" onclick={() => show("about")}>Back</button>
      <button type="button" class="btn btn-primary" onclick={() => dialog.close()}>Close</button>
    </div>
  {:else}
    <h2 id="about-title" class="about-h" tabindex="-1" bind:this={heading}>GNU General Public License, version 3</h2>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <pre class="about-gpl" tabindex="0" aria-label="GPL-3.0.txt">{GPL_TEXT}</pre>
    <div class="about-foot end">
      <button type="button" class="btn btn-outline" onclick={() => show("credits")}>Back</button>
      <button type="button" class="btn btn-primary" onclick={() => dialog.close()}>Close</button>
    </div>
  {/if}
</dialog>
