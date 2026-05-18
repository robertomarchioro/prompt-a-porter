<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variante?: "default" | "private" | "team";
    colore?: string;
    children: Snippet;
    onclick?: () => void;
  }

  let {
    variante = "default",
    colore,
    children,
    onclick,
  }: Props = $props();
</script>

{#if onclick}
  <button
    type="button"
    class="tag tag--interactive"
    class:tag--private={variante === "private"}
    class:tag--team={variante === "team"}
    style:--_tag-dot={colore || undefined}
    onclick={onclick}
  >
    {#if colore}
      <span class="dot"></span>
    {/if}
    {@render children()}
  </button>
{:else}
  <span
    class="tag"
    class:tag--private={variante === "private"}
    class:tag--team={variante === "team"}
    style:--_tag-dot={colore || undefined}
  >
    {#if colore}
      <span class="dot"></span>
    {/if}
    {@render children()}
  </span>
{/if}

<style>
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    font-size: var(--fs-xs);
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    white-space: nowrap;
  }

  /* Reset stile button quando Tag e' interattivo (variante <button>) */
  button.tag {
    appearance: none;
    font: inherit;
    cursor: pointer;
  }

  .tag--private {
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-color: transparent;
  }
  .tag--team {
    color: var(--accent-team);
    background: var(--accent-team-soft);
    border-color: transparent;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--_tag-dot);
    flex-shrink: 0;
  }
</style>
