<script lang="ts">
  import type { Snippet } from "svelte";
  import { ChevronRight, ChevronDown, Plus } from "lucide-svelte";

  interface Props {
    titolo: string;
    conteggio?: number;
    bottonAggiungi?: boolean;
    onAggiungi?: () => void;
    collapsed?: boolean;
    children: Snippet;
  }

  let {
    titolo,
    conteggio,
    bottonAggiungi = false,
    onAggiungi,
    collapsed = $bindable(false),
    children,
  }: Props = $props();

  function toggle(): void {
    collapsed = !collapsed;
  }
</script>

<section class="nav-group" data-collapsed={collapsed}>
  <header class="header">
    <button
      class="toggle"
      type="button"
      onclick={toggle}
      aria-expanded={!collapsed}
    >
      {#if collapsed}
        <ChevronRight size={12} />
      {:else}
        <ChevronDown size={12} />
      {/if}
      <span class="titolo">{titolo}</span>
      {#if conteggio !== undefined}
        <span class="conteggio">{conteggio}</span>
      {/if}
    </button>
    {#if bottonAggiungi}
      <button
        class="aggiungi"
        type="button"
        aria-label="Aggiungi {titolo.toLowerCase()}"
        onclick={onAggiungi}
      >
        <Plus size={14} />
      </button>
    {/if}
  </header>
  {#if !collapsed}
    <div class="contenuto">
      {@render children()}
    </div>
  {/if}
</section>

<style>
  .nav-group {
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    align-items: center;
    height: 24px;
    padding: 0 var(--sp-2);
    gap: var(--sp-1);
  }

  .toggle {
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    background: transparent;
    border: 0;
    color: var(--text-subtle);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    cursor: pointer;
    padding: 0;
    text-align: left;
  }

  .toggle:hover {
    color: var(--text-muted);
  }

  .titolo {
    text-align: left;
  }

  .conteggio {
    margin-left: auto;
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
  }

  .aggiungi {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .aggiungi:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .contenuto {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 var(--sp-1);
  }
</style>
