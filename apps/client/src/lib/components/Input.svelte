<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends HTMLInputAttributes {
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

<input
  class="input {classeExtra}"
  bind:value={valore}
  aria-invalid={invalido || undefined}
  {...rest}
/>

<style>
  .input {
    width: 100%;
    height: 34px;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }

  .input::placeholder {
    color: var(--text-subtle);
  }
  .input:hover {
    border-color: var(--border-strong);
  }
  .input:focus {
    outline: none;
    border-color: var(--accent-team);
    box-shadow: 0 0 0 3px var(--accent-team-soft);
  }
  .input[aria-invalid="true"] {
    border-color: var(--danger);
    box-shadow: 0 0 0 3px var(--danger-soft);
  }
  .input:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
    opacity: 0.7;
  }
</style>
