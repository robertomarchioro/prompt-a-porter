<script lang="ts">
  /**
   * v0.8.7 PR-C — Viewer tail in-app per debug log.
   *
   * Mostra le ultime N righe del file `pap.log` (tauri-plugin-log)
   * con filtri level + regex + auto-refresh.
   *
   * Backend cmd `debug_log_leggi(n_righe)` ritorna `RigaLog[]` con
   * timestamp, level, target, message già parsati.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Pause, Play, RefreshCw, X } from "lucide-svelte";

  interface RigaLog {
    timestamp: string;
    level: string;
    target: string;
    message: string;
    raw: string;
  }

  type LivelloFiltro = "" | "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";

  const INTERVALLO_REFRESH_MS = 2000;
  const N_RIGHE_DEFAULT = 200;

  let righe = $state<RigaLog[]>([]);
  let errore = $state("");
  let livello = $state<LivelloFiltro>("");
  let regexInput = $state("");
  let autoRefresh = $state(true);
  let inAttesa = $state(false);
  let timerId: ReturnType<typeof setInterval> | undefined;

  async function ricarica(): Promise<void> {
    if (inAttesa) return;
    inAttesa = true;
    try {
      righe = await invoke<RigaLog[]>("debug_log_leggi", {
        nRighe: N_RIGHE_DEFAULT,
      });
      errore = "";
    } catch (e) {
      errore = String(e);
    } finally {
      inAttesa = false;
    }
  }

  function avviaTimer(): void {
    fermaTimer();
    timerId = setInterval(() => {
      if (autoRefresh) void ricarica();
    }, INTERVALLO_REFRESH_MS);
  }

  function fermaTimer(): void {
    if (timerId !== undefined) {
      clearInterval(timerId);
      timerId = undefined;
    }
  }

  onMount(() => {
    void ricarica();
    avviaTimer();
  });

  onDestroy(() => {
    fermaTimer();
  });

  const regexCompilata = $derived.by(() => {
    const s = regexInput.trim();
    if (!s) return null;
    try {
      return new RegExp(s, "i");
    } catch {
      return null;
    }
  });

  const regexErrore = $derived.by(() => {
    const s = regexInput.trim();
    if (!s) return "";
    try {
      new RegExp(s, "i");
      return "";
    } catch (e) {
      return String(e);
    }
  });

  const righeFiltrate = $derived.by(() => {
    let out = righe;
    if (livello) {
      out = out.filter((r) => r.level === livello);
    }
    if (regexCompilata) {
      const re = regexCompilata;
      out = out.filter(
        (r) =>
          re.test(r.message) ||
          re.test(r.target) ||
          re.test(r.raw),
      );
    }
    return out;
  });

  function classeRiga(level: string): string {
    switch (level) {
      case "ERROR":
        return "log-error";
      case "WARN":
        return "log-warn";
      case "INFO":
        return "log-info";
      case "DEBUG":
        return "log-debug";
      case "TRACE":
        return "log-trace";
      default:
        return "log-altro";
    }
  }

  function pulisciFiltri(): void {
    livello = "";
    regexInput = "";
  }
</script>

<div class="log-viewer">
  <header class="log-viewer-h">
    <div class="log-filtri">
      <label class="log-filtro">
        <span>Livello</span>
        <select bind:value={livello} aria-label="Filtro livello">
          <option value="">Tutti</option>
          <option value="ERROR">ERROR</option>
          <option value="WARN">WARN</option>
          <option value="INFO">INFO</option>
          <option value="DEBUG">DEBUG</option>
          <option value="TRACE">TRACE</option>
        </select>
      </label>
      <label class="log-filtro log-filtro-regex">
        <span>Regex</span>
        <input
          type="text"
          placeholder="es. salva|errore"
          bind:value={regexInput}
          aria-label="Filtro regex (case-insensitive)"
          class:log-regex-err={regexErrore !== ""}
        />
      </label>
      {#if livello || regexInput}
        <button
          type="button"
          class="log-btn-ghost"
          onclick={pulisciFiltri}
          aria-label="Pulisci filtri"
          title="Pulisci filtri"
        >
          <X size={12} />
        </button>
      {/if}
    </div>

    <div class="log-azioni">
      <span class="log-conta">
        {righeFiltrate.length}/{righe.length}
      </span>
      <button
        type="button"
        class="log-btn-ghost"
        onclick={ricarica}
        disabled={inAttesa}
        title="Aggiorna ora"
        aria-label="Aggiorna ora"
      >
        <RefreshCw size={12} />
      </button>
      <button
        type="button"
        class="log-btn-ghost"
        onclick={() => (autoRefresh = !autoRefresh)}
        title={autoRefresh ? "Pausa auto-refresh" : "Riprendi auto-refresh"}
        aria-label={autoRefresh ? "Pausa auto-refresh" : "Riprendi auto-refresh"}
      >
        {#if autoRefresh}
          <Pause size={12} />
        {:else}
          <Play size={12} />
        {/if}
      </button>
    </div>
  </header>

  {#if regexErrore}
    <p class="log-err">Regex non valida: {regexErrore}</p>
  {/if}
  {#if errore}
    <p class="log-err">{errore}</p>
  {/if}

  <div class="log-corpo" role="log" aria-live="polite">
    {#if righe.length === 0}
      <p class="log-vuoto">
        Nessuna riga di log. Abilita "Debug log" e usa l'app per generare
        eventi.
      </p>
    {:else if righeFiltrate.length === 0}
      <p class="log-vuoto">Nessuna riga corrisponde ai filtri.</p>
    {:else}
      {#each righeFiltrate as r (r.raw + r.timestamp)}
        <div class="log-riga {classeRiga(r.level)}">
          {#if r.timestamp}
            <span class="log-ts">{r.timestamp}</span>
          {/if}
          {#if r.level}
            <span class="log-level">{r.level}</span>
          {/if}
          {#if r.target}
            <span class="log-target">{r.target}</span>
          {/if}
          <span class="log-msg">{r.message || r.raw}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: var(--sp-2);
  }

  .log-viewer-h {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }

  .log-filtri {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex: 1;
    min-width: 0;
  }

  .log-filtro {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .log-filtro-regex {
    flex: 1;
    min-width: 0;
  }

  .log-filtro select,
  .log-filtro input {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-xs);
    padding: 2px 6px;
    font-family: var(--font-ui);
  }

  .log-filtro input {
    flex: 1;
    min-width: 100px;
    font-family: var(--font-mono);
  }

  .log-filtro input.log-regex-err {
    border-color: var(--accent-danger, #d9534f);
  }

  .log-azioni {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .log-conta {
    font-size: var(--fs-xs);
    font-variant-numeric: tabular-nums;
    color: var(--text-subtle);
    padding: 0 4px;
  }

  .log-btn-ghost {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-input);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .log-btn-ghost:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .log-btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .log-err {
    margin: 0;
    color: var(--accent-danger, #d9534f);
    font-size: var(--fs-xs);
  }

  .log-corpo {
    max-height: 320px;
    overflow: auto;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: var(--sp-1);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.5;
  }

  .log-vuoto {
    margin: 0;
    padding: var(--sp-2);
    color: var(--text-subtle);
    font-style: italic;
    text-align: center;
  }

  .log-riga {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 2px 4px;
    border-bottom: 1px solid var(--border-subtle);
    overflow-wrap: anywhere;
  }

  .log-riga:last-child {
    border-bottom: 0;
  }

  .log-ts {
    color: var(--text-subtle);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .log-level {
    font-weight: var(--fw-semibold);
    min-width: 48px;
    text-align: center;
    padding: 0 4px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .log-target {
    color: var(--text-muted);
    white-space: nowrap;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .log-msg {
    color: var(--text-default);
    flex: 1;
    min-width: 0;
  }

  .log-error .log-level {
    background: rgba(217, 83, 79, 0.18);
    color: var(--accent-danger, #d9534f);
  }

  .log-warn .log-level {
    background: rgba(210, 168, 95, 0.18);
    color: var(--accent-warning, #d2a85f);
  }

  .log-info .log-level {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .log-debug .log-level {
    background: var(--bg-overlay);
    color: var(--text-muted);
  }

  .log-trace .log-level {
    background: transparent;
    color: var(--text-subtle);
  }

  .log-error .log-msg {
    color: var(--accent-danger, #d9534f);
  }
</style>
