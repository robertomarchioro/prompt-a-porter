<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    selezionato?: boolean;
    children: Snippet;
    onclick?: () => void;
  }

  let { selezionato = false, children, onclick }: Props = $props();
</script>

<button
  class="list-item"
  aria-selected={selezionato || undefined}
  onclick={onclick}
  type="button"
>
  {@render children()}
</button>

<style>
  .list-item {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: 0 var(--sp-3);
    height: 32px;
    width: 100%;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-default);
    background: transparent;
    cursor: pointer;
    transition: background var(--motion-fast);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    text-align: left;
  }

  .list-item:hover {
    background: var(--bg-overlay);
  }
  .list-item[aria-selected="true"] {
    background: var(--bg-overlay);
    color: var(--text-strong);
  }
</style>
