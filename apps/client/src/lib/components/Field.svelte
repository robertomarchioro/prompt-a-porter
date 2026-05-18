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
    <!--
      label wrappa children per associare automaticamente il control
      (input/textarea/select) all'etichetta. Pattern HTML5 valido che
      elimina il warning a11y_label_has_associated_control senza
      richiedere id/for esplicito su ogni callsite.
    -->
    <label class="field-row">
      <span class="field-label">{etichetta}</span>
      {@render children()}
    </label>
  {:else}
    {@render children()}
  {/if}
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

  /*
    Quando etichetta presente, la label wrappa testo + control e mantiene
    lo stesso layout colonna del .field. Pattern Svelte: la label e' un
    flex container interno per non rompere il gap visivo.
  */
  .field-row {
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
