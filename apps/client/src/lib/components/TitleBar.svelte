<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { statoTema, salvaTemaTono } from "$lib/stores/preferenze.svelte";
  import { apriModale } from "$lib/stores/modale.svelte";
  import { fmtShortcut } from "$lib/util/shortcut";
  import { Sun, Moon, Settings, HelpCircle } from "lucide-svelte";

  // #404: versione reale dell'app (no piu' la stringa hardcoded errata).
  let versione = $state("");
  onMount(async () => {
    try {
      versione = await getVersion();
    } catch {
      versione = "";
    }
  });

  function toggleTema(): void {
    const successivo = statoTema.tema === "dark" ? "light" : "dark";
    statoTema.tema = successivo;
    void salvaTemaTono(successivo, statoTema.tono);
  }

  function apriImpostazioni(): void {
    apriModale({ tipo: "impostazioni" });
  }

  function apriGuida(): void {
    apriModale({ tipo: "impostazioni", sezione: "guida" });
  }
</script>

<header class="titlebar">
  <div class="brand" title="Prompt a Porter">
    <span class="glyph">P</span>
    {#if versione}
      <span class="version-tag">v{versione}</span>
    {/if}
  </div>
  <div class="actions">
    <button
      type="button"
      class="icon-button"
      aria-label="Cambia tema"
      onclick={toggleTema}
    >
      {#if statoTema.tema === "dark"}
        <Sun size={16} />
      {:else}
        <Moon size={16} />
      {/if}
    </button>
    <button
      type="button"
      class="icon-button"
      data-tour="aiuto"
      aria-label="Guida e aiuto"
      title="Guida e aiuto"
      onclick={apriGuida}
    >
      <HelpCircle size={16} />
    </button>
    <button
      type="button"
      class="icon-button"
      aria-label="Impostazioni"
      title="Impostazioni ({fmtShortcut('mod+,')})"
      onclick={apriImpostazioni}
    >
      <Settings size={16} />
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--titlebar-h);
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
  }

  .glyph {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    background: var(--accent-team);
    color: var(--accent-team-on);
    font-weight: var(--fw-bold);
    font-size: var(--fs-xs);
  }

  .version-tag {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding: 2px var(--sp-2);
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  .icon-button:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
