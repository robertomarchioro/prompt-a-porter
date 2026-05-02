<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    attivo?: boolean;
    conteggio?: number;
    icona?: Snippet;
    children: Snippet;
    onclick?: () => void;
  }

  let { attivo = false, conteggio, icona, children, onclick }: Props =
    $props();
</script>

<button
  class="nav-item"
  aria-current={attivo || undefined}
  onclick={onclick}
  type="button"
>
  {#if icona}
    <span class="ico">{@render icona()}</span>
  {/if}
  <span class="label">{@render children()}</span>
  {#if conteggio !== undefined}
    <span class="count">{conteggio}</span>
  {/if}
</button>

<style>
  .nav-item {
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

  .nav-item:hover {
    background: var(--bg-overlay);
  }
  .nav-item[aria-current="true"] {
    background: var(--bg-overlay);
    color: var(--text-strong);
  }

  .ico {
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }
</style>
