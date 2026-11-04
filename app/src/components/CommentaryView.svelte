<script lang="ts">
  // Commentary blocks for one verse (pipeline/DESIGN.md 17.5; wave-2-layout.md
  // 7). Each block is drawn from its stored node tree: stored text goes in only
  // as Svelte text expressions (text nodes, never raw HTML), one `.ct` span per
  // stored text node (`data-n` = its node number, `data-block` on the block).
  // Elements become presentation wrappers only; verse and chapter milestones
  // (no children) draw nothing. A `reference` is a link to its range start.
  //
  // The template keeps no whitespace between inline tags on purpose: Svelte
  // would turn it into extra " " text nodes, i.e. characters the source
  // doesn't have. tests/commentary-render.test.ts checks every text node.
  import { attr, childrenOf, refTarget, scopeLabel, verseLabel, type CmtBlock, type CmtGroup, type CmtNode, type VerseRef } from "../lib/commentary";
  import type { Book } from "../lib/types";

  interface Props {
    groups: CmtGroup[];
    verseId: number;
    books: Book[];
    onRef: (to: VerseRef) => void;
  }
  let { groups, verseId, books, onRef }: Props = $props();

  const TAGS: Record<string, string> = { p: "div", title: "div", table: "div", row: "div", cell: "span", hi: "span" };
  const tag = (n: CmtNode) => TAGS[n.name ?? ""] ?? "span";
  const target = (n: CmtNode) => refTarget(n);
  const toId = (t: VerseRef) => t.book * 1_000_000 + t.chapter * 1000 + t.verse;
</script>

{#snippet node(b: CmtBlock, n: CmtNode)}{#if n.text !== null}<span class="ct" data-n={n.n}>{n.text}</span>{:else}{@const kids = childrenOf(b, n)}{@const t = target(n)}{#if t}<button type="button" class="cmt-ref" data-n={n.n} title="Go to {verseLabel(toId(t), books)}" onclick={() => onRef(t)}>{#each kids as c (c.n)}{@render node(b, c)}{/each}</button>{:else if kids.length}<svelte:element this={tag(n)} class="cmt-el el-{n.name}" data-n={n.n} data-type={attr(n, "type") ?? attr(n, "subType")}>{#each kids as c (c.n)}{@render node(b, c)}{/each}</svelte:element>{/if}{/if}{/snippet}

{#each groups as g, i (i)}
  <section class="cmt-group" aria-label={scopeLabel(g, verseId, books)}>
    <p class="cmt-scope">{scopeLabel(g, verseId, books)}</p>
    {#each g.blocks as b (b.id)}
      <div class="cmt-block" data-block={b.id}>{@render node(b, b.nodes[0])}</div>
    {/each}
  </section>
{/each}
