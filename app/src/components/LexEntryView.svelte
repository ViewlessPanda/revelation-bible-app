<script lang="ts">
  // One Strong's entry (pipeline/DESIGN.md 7). Stored text goes in only as
  // Svelte text expressions (text nodes, never raw HTML), one `.lt` span per
  // stored text node (`data-n` = its node number). Display attributes are
  // `.la` spans with the stored value; the key shown on an empty
  // cross-reference is a `.lb` span (UI text built from the stored key).
  // Cross-references (w@src, strongsref) are buttons that open their entry.
  // Layout whitespace directly in <entry> is not drawn (lib/lexicon.ts).
  //
  // The template keeps no whitespace between inline tags on purpose: Svelte
  // would turn it into extra " " text nodes, i.e. characters the source
  // doesn't have. tests/lexicon-render.test.ts checks every text node.
  import { childrenOf, displayAttrs, lexBlocks, lexiconName, refKey } from "../lib/lexicon";
  import type { LexEntry, LexNode } from "../lib/types";

  interface Props {
    entry: LexEntry;
    onRef: (key: string) => void;
  }
  let { entry, onRef }: Props = $props();

  const blocks = $derived(lexBlocks(entry));
</script>

{#snippet node(n: LexNode)}{#if n.text !== null}<span class="lt" data-n={n.n}>{n.text}</span>{:else}{@const ref = refKey(n)}{@const kids = childrenOf(entry, n)}{#if ref}<button type="button" class="lex-ref" data-n={n.n} data-ref={ref} title="Open Strong's {ref}" onclick={() => onRef(ref)}>{#if kids.length}{#each kids as c (c.n)}{@render node(c)}{/each}{:else}<span class="lb">{ref}</span>{/if}</button>{:else}<span class="lex-el el-{n.name}" data-n={n.n}>{#if n.name === "w" && kids.length}<span class="lex-hw" lang="he" dir="rtl">{#each kids as c (c.n)}{@render node(c)}{/each}</span>{:else}{#each kids as c (c.n)}{@render node(c)}{/each}{/if}{#each displayAttrs(n) as a (a.name)}<span class="la" data-attr={a.name}>{a.value}</span>{/each}</span>{/if}{/if}{/snippet}

<section class="lex-entry" aria-label="Strong's {entry.key}" data-key={entry.key}>
  <div class="lex-top"><h2 class="lex-key" tabindex="-1">{entry.key}</h2><span class="lex-src">{lexiconName(entry.lexicon)}</span></div>
  {#each blocks as b, i (i)}
    {#if b.kind === "field"}
      <div class="lex-field"><span class="lex-label">{b.label}</span><div class="lex-body">{@render node(b.node)}</div></div>
    {:else if b.kind === "head"}
      <div class="lex-head">{#each b.nodes as n (n.n)}{@render node(n)}{/each}</div>
    {:else}
      <div class="lex-body lex-para">{#each b.nodes as n (n.n)}{@render node(n)}{/each}</div>
    {/if}
  {/each}
</section>
