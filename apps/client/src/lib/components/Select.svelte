<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";

  interface Props extends HTMLSelectAttributes {
    valore?: string;
    children: Snippet;
  }

  let {
    valore = $bindable(""),
    children,
    class: classeExtra = "",
    ...rest
  }: Props = $props();
</script>

<select class="select {classeExtra}" bind:value={valore} {...rest}>
  {@render children()}
</select>

<style>
  .select {
    width: 100%;
    height: 34px;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    cursor: pointer;
    appearance: none;
    background-image: linear-gradient(
        45deg,
        transparent 50%,
        var(--text-muted) 50%
      ),
      linear-gradient(135deg, var(--text-muted) 50%, transparent 50%);
    background-position:
      calc(100% - 14px) 50%,
      calc(100% - 9px) 50%;
    background-size: 5px 5px;
    background-repeat: no-repeat;
    padding-right: 28px;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }

  .select:hover {
    border-color: var(--border-strong);
  }
  .select:focus {
    outline: none;
    border-color: var(--accent-team);
    box-shadow: 0 0 0 3px var(--accent-team-soft);
  }
  .select:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
    opacity: 0.7;
  }
</style>
