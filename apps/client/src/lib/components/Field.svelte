<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    etichetta?: string;
    hint?: string;
    errore?: string;
    children: Snippet;
  }

  let { etichetta, hint, errore, children }: Props = $props();
</script>

<div class="field">
  {#if etichetta}
    <label class="field-label">{etichetta}</label>
  {/if}
  {@render children()}
  {#if errore}
    <span class="field-error">{errore}</span>
  {:else if hint}
    <span class="field-hint">{hint}</span>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-size: var(--fs-xs);
    font-weight: var(--fw-medium);
    letter-spacing: var(--tracking-wide);
    color: var(--text-muted);
    font-family: var(--font-mono);
    text-transform: uppercase;
  }

  .field-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .field-error {
    font-size: var(--fs-xs);
    color: var(--danger);
  }
</style>
