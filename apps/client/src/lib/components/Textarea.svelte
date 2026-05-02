<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";

  interface Props extends HTMLTextareaAttributes {
    valore?: string;
    invalido?: boolean;
  }

  let {
    valore = $bindable(""),
    invalido = false,
    class: classeExtra = "",
    ...rest
  }: Props = $props();
</script>

<textarea
  class="textarea {classeExtra}"
  bind:value={valore}
  aria-invalid={invalido || undefined}
  {...rest}
></textarea>

<style>
  .textarea {
    width: 100%;
    min-height: 96px;
    padding: 10px 12px;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-loose);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    resize: vertical;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }

  .textarea::placeholder {
    color: var(--text-subtle);
  }
  .textarea:hover {
    border-color: var(--border-strong);
  }
  .textarea:focus {
    outline: none;
    border-color: var(--accent-team);
    box-shadow: 0 0 0 3px var(--accent-team-soft);
  }
  .textarea[aria-invalid="true"] {
    border-color: var(--danger);
    box-shadow: 0 0 0 3px var(--danger-soft);
  }
  .textarea:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
    opacity: 0.7;
  }
</style>
