<script lang="ts">
  interface Props {
    valore?: string;
    onchange?: (combo: string) => void;
  }

  let { valore = $bindable("Ctrl+Shift+P"), onchange }: Props = $props();

  let inAscolto = $state(false);

  const tasti = $derived(valore.split("+"));

  function iniziaAscolto() {
    inAscolto = true;
  }

  function gestisciTasto(e: KeyboardEvent) {
    if (!inAscolto) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === "Escape") {
      valore = "Ctrl+Shift+P";
      inAscolto = false;
      onchange?.(valore);
      return;
    }

    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

    const parti: string[] = [];
    if (e.ctrlKey || e.metaKey) parti.push("Ctrl");
    if (e.shiftKey) parti.push("Shift");
    if (e.altKey) parti.push("Alt");

    let tasto = e.key;
    if (tasto.length === 1) tasto = tasto.toUpperCase();
    else if (tasto === " ") tasto = "Space";

    parti.push(tasto);

    if (!e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey) return;

    valore = parti.join("+");
    inAscolto = false;
    onchange?.(valore);
  }
</script>

<svelte:window onkeydown={gestisciTasto} />

<button
  class="hotkey-input"
  class:hotkey-input--ascolto={inAscolto}
  onclick={iniziaAscolto}
  type="button"
>
  <div class="hotkey-tasti">
    {#each tasti as t}
      <kbd class="hotkey-kbd">{t}</kbd>
    {/each}
  </div>
  <span class="hotkey-hint">
    {#if inAscolto}
      Premi una combinazione… · Esc per resettare
    {:else}
      Clicca per cambiare
    {/if}
  </span>
</button>

<style>
  .hotkey-input {
    appearance: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-5);
    background: var(--bg-input);
    border: 2px solid var(--border-default);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
    width: 100%;
    font-family: var(--font-ui);
  }

  .hotkey-input:hover {
    border-color: var(--border-strong);
  }

  .hotkey-input--ascolto {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 4px var(--accent-team-soft);
  }

  .hotkey-input:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--bg-canvas), 0 0 0 4px var(--accent-team);
  }

  .hotkey-tasti {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .hotkey-kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 36px;
    height: 36px;
    padding: 0 var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-xl);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
  }

  .hotkey-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }
</style>
