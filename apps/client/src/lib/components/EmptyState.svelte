<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    titolo: string;
    hint?: string;
    icona?: Snippet;
    azioni?: Snippet;
  }

  let { titolo, hint, icona, azioni }: Props = $props();
</script>

<div class="empty">
  {#if icona}
    <div class="empty-icon">{@render icona()}</div>
  {/if}
  <div class="empty-title">{titolo}</div>
  {#if hint}
    <div class="empty-hint">{hint}</div>
  {/if}
  {#if azioni}
    <div class="empty-actions">{@render azioni()}</div>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-3);
    padding: var(--sp-7) var(--sp-5);
    color: var(--text-muted);
    text-align: center;
  }

  .empty-icon {
    color: var(--text-subtle);
  }

  .empty-title {
    font-size: var(--fs-base);
    color: var(--text-strong);
    font-weight: var(--fw-medium);
  }

  .empty-hint {
    font-size: var(--fs-sm);
    max-width: 36ch;
  }

  .empty-actions {
    margin-top: var(--sp-2);
  }
</style>
