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

<span
  class="tag"
  class:tag--private={variante === "private"}
  class:tag--team={variante === "team"}
  style:--_tag-dot={colore || undefined}
  role={onclick ? "button" : undefined}
  tabindex={onclick ? 0 : undefined}
  onclick={onclick}
  onkeydown={(e) => {
    if (onclick && (e.key === "Enter" || e.key === " ")) onclick();
  }}
>
  {#if colore}
    <span class="dot"></span>
  {/if}
  {@render children()}
</span>

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

  .tag[role="button"] {
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
