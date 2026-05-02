<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    testo: string;
    children: Snippet;
  }

  let { testo, children }: Props = $props();
</script>

<span class="tooltip-wrapper" data-tip={testo}>
  {@render children()}
</span>

<style>
  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
  }

  .tooltip-wrapper::after {
    content: attr(data-tip);
    position: absolute;
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 8px;
    font-size: var(--fs-xs);
    color: var(--text-strong);
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--motion-fast);
    z-index: var(--z-tooltip);
  }

  .tooltip-wrapper:hover::after {
    opacity: 1;
  }
</style>
