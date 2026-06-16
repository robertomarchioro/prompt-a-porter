<script lang="ts">
  /**
   * Tab Test golden del DetailPane (F5 PR-C).
   *
   * Tabella read-only dei golden test associati al prompt + bottone elimina.
   * Crea/modifica/esegui sono placeholder F8 (richiedono UI provider config).
   *
   * Riferimento blueprint: docs/roadmap/redesign-v08/blueprint-F5.md §3
   */
  import { invoke } from "@tauri-apps/api/core";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import { onDestroy, onMount } from "svelte";
  import { Play, PlayCircle, Pencil, Trash2, Plus } from "lucide-svelte";
  import Modale from "$lib/components/Modale.svelte";

  interface Golden {
    id: string;
    prompt_id: string;
    etichetta: string;
    input_vars: string;
    expected_output: string;
    similarity_fn: string;
    soglia_tolleranza: number;
    creato_a: string;
    aggiornato_a: string;
  }

  // Shape parziale: serve solo `passed` per la UI batch.
  interface Observation {
    passed: boolean;
  }

  interface ProviderConfigItem {
    provider: string;
    base_url?: string | null;
    default_model?: string | null;
    abilitato: boolean;
  }

  interface RisultatoBatch {
    etichetta: string;
    passed: boolean;
    errore?: string;
  }

  interface Props {
    promptId: string;
    onConteggio?: (n: number) => void;
  }

  let { promptId, onConteggio }: Props = $props();

  let goldens = $state<Golden[]>([]);
  let caricamento = $state(false);

  async function carica(): Promise<void> {
    caricamento = true;
    try {
      goldens = await invoke<Golden[]>("golden_lista", {
        promptId,
      });
      onConteggio?.(goldens.length);
    } catch (e) {
      console.error("[golden] golden_lista", e);
      goldens = [];
      onConteggio?.(0);
    } finally {
      caricamento = false;
    }
  }

  $effect(() => {
    void promptId;
    void carica();
  });

  function gestListaMutata(): void {
    void carica();
  }

  onMount(() => {
    window.addEventListener("pap:lista-mutata", gestListaMutata);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", gestListaMutata);
  });

  async function elimina(id: string, etichetta: string): Promise<void> {
    if (!confirm(`Eliminare il golden test "${etichetta}"?`)) return;
    try {
      await invoke("golden_elimina", { id });
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[golden] golden_elimina", e);
    }
  }

  // M3 PR-3: "Esegui tutti i golden" batch.
  // Loop seriale per evitare rate limit/cost spike del provider.
  // Backend `golden_esegui` chiamato per ogni golden con provider scelto;
  // raccogliamo Observation.passed (o errore) per il summary finale.
  type StatoBatch =
    | { fase: "scelta" }
    | { fase: "esecuzione"; correnteIdx: number; correnteEtichetta: string }
    | { fase: "completato"; risultati: RisultatoBatch[] };

  let modaleBatch = $state<{
    aperto: boolean;
    providers: ProviderConfigItem[];
    providerScelto: string;
    modelScelto: string;
    stato: StatoBatch;
    caricamentoProviders: boolean;
    errore: string;
  }>({
    aperto: false,
    providers: [],
    providerScelto: "",
    modelScelto: "",
    stato: { fase: "scelta" },
    caricamentoProviders: false,
    errore: "",
  });

  async function apriBatch(): Promise<void> {
    modaleBatch = {
      aperto: true,
      providers: [],
      providerScelto: "",
      modelScelto: "",
      stato: { fase: "scelta" },
      caricamentoProviders: true,
      errore: "",
    };
    try {
      const lista = await invoke<ProviderConfigItem[]>("provider_config_lista");
      modaleBatch.providers = lista.filter((p) => p.abilitato);
      if (modaleBatch.providers.length === 0) {
        modaleBatch.errore =
          "Nessun provider AI abilitato. Configura un provider in Impostazioni → Provider AI.";
      } else {
        const primo = modaleBatch.providers[0];
        modaleBatch.providerScelto = primo.provider;
        modaleBatch.modelScelto = primo.default_model ?? "";
      }
    } catch (e) {
      modaleBatch.errore = String(e).replace(/^Error: /, "");
    } finally {
      modaleBatch.caricamentoProviders = false;
    }
  }

  function chiudiBatch(): void {
    if (modaleBatch.stato.fase === "esecuzione") return; // no chiusura mid-batch
    modaleBatch.aperto = false;
  }

  function onCambiaProvider(): void {
    const p = modaleBatch.providers.find(
      (x) => x.provider === modaleBatch.providerScelto,
    );
    modaleBatch.modelScelto = p?.default_model ?? "";
  }

  async function eseguiTutti(): Promise<void> {
    if (
      !modaleBatch.providerScelto ||
      !modaleBatch.modelScelto.trim() ||
      goldens.length === 0
    ) {
      return;
    }
    const risultati: RisultatoBatch[] = [];
    for (let i = 0; i < goldens.length; i++) {
      const g = goldens[i];
      modaleBatch.stato = {
        fase: "esecuzione",
        correnteIdx: i + 1,
        correnteEtichetta: g.etichetta,
      };
      try {
        const obs = await invoke<Observation>("golden_esegui", {
          goldenId: g.id,
          providerKind: modaleBatch.providerScelto,
          model: modaleBatch.modelScelto,
        });
        risultati.push({ etichetta: g.etichetta, passed: obs.passed });
      } catch (e) {
        risultati.push({
          etichetta: g.etichetta,
          passed: false,
          errore: String(e).replace(/^Error: /, ""),
        });
      }
    }
    modaleBatch.stato = { fase: "completato", risultati };
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    try {
      const sec = Math.max(
        0,
        Math.floor((Date.now() - new Date(iso).getTime()) / 1000),
      );
      if (sec < 60) return "ora";
      const min = Math.floor(sec / 60);
      if (min < 60) return `${min}m fa`;
      const h = Math.floor(min / 60);
      if (h < 24) return `${h}h fa`;
      const g = Math.floor(h / 24);
      return `${g}g fa`;
    } catch {
      return "";
    }
  }
</script>

<div class="golden-tab">
  <header class="header">
    <span style="display: inline-flex; align-items: center; gap: 6px;">
      <span class="titolo">Test golden</span>
      <AiutoLink chiave="regression-testing" dimensione={16} />
    </span>
    <span class="conteggio">{goldens.length}</span>
    <button
      class="secondary"
      type="button"
      onclick={apriBatch}
      disabled={goldens.length === 0}
      title={goldens.length === 0
        ? "Nessun golden da eseguire"
        : `Esegui tutti i ${goldens.length} golden test in batch`}
    >
      <PlayCircle size={14} />
      <span>Esegui tutti</span>
    </button>
    <button
      class="primary"
      type="button"
      onclick={() => console.log("F8 modale crea golden")}
      title="Crea golden test (F8 modale)"
    >
      <Plus size={14} />
      <span>Golden</span>
    </button>
  </header>

  {#if caricamento && goldens.length === 0}
    <div class="vuoto">
      <p>Caricamento…</p>
    </div>
  {:else if goldens.length === 0}
    <div class="vuoto">
      <p class="muted">Nessun golden test definito per questo prompt.</p>
      <p class="sub">
        I golden ti permettono di confrontare l'output con un valore atteso e
        tracciare drift tra modelli o versioni.
      </p>
    </div>
  {:else}
    <ul class="lista" role="list">
      {#each goldens as g (g.id)}
        <li class="riga">
          <div class="riga-meta">
            <span class="etichetta">{g.etichetta}</span>
            <div class="badge-row">
              <span class="badge">{g.similarity_fn}</span>
              <span class="badge muted"
                >soglia {(g.soglia_tolleranza * 100).toFixed(0)}%</span
              >
              <span class="badge muted"
                >aggiornato {tempoRelativo(g.aggiornato_a)}</span
              >
            </div>
          </div>
          <div class="riga-azioni">
            <button
              class="ico"
              type="button"
              title="Esegui (F8 — richiede provider config)"
              aria-label="Esegui golden"
              onclick={() => console.log("F8 esegui golden", g.id)}
            >
              <Play size={14} />
            </button>
            <button
              class="ico"
              type="button"
              title="Modifica (F8 modale)"
              aria-label="Modifica golden"
              onclick={() => console.log("F8 modifica golden", g.id)}
            >
              <Pencil size={14} />
            </button>
            <button
              class="ico ico-danger"
              type="button"
              title="Elimina golden"
              aria-label="Elimina golden"
              onclick={() => elimina(g.id, g.etichetta)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if modaleBatch.aperto}
  <Modale
    titolo="Esegui tutti i golden"
    sottotitolo={`${goldens.length} test su questo prompt`}
    larghezza="md"
    onChiudi={chiudiBatch}
  >
    {#if modaleBatch.caricamentoProviders}
      <p class="batch-info">Caricamento provider…</p>
    {:else if modaleBatch.errore}
      <p class="batch-error">{modaleBatch.errore}</p>
    {:else if modaleBatch.stato.fase === "scelta"}
      <div class="batch-form">
        <label class="batch-field">
          <span class="batch-label">Provider</span>
          <select
            class="batch-input"
            bind:value={modaleBatch.providerScelto}
            onchange={onCambiaProvider}
          >
            {#each modaleBatch.providers as p (p.provider)}
              <option value={p.provider}>{p.provider}</option>
            {/each}
          </select>
        </label>
        <label class="batch-field">
          <span class="batch-label">Modello</span>
          <input
            class="batch-input"
            type="text"
            bind:value={modaleBatch.modelScelto}
            placeholder="Es. claude-sonnet-4-6, gpt-4o, llama3.2…"
          />
        </label>
        <p class="batch-hint">
          I golden verranno eseguiti in serie (uno alla volta) per evitare
          rate-limit. Stima ~5-30s per golden a seconda del provider.
        </p>
      </div>
    {:else if modaleBatch.stato.fase === "esecuzione"}
      <div class="batch-progress">
        <p class="batch-progress-label">
          Esecuzione {modaleBatch.stato.correnteIdx} di {goldens.length}:
          <strong>{modaleBatch.stato.correnteEtichetta}</strong>
        </p>
        <div class="batch-bar">
          <div
            class="batch-bar-fill"
            style:width="{((modaleBatch.stato.correnteIdx - 1) / goldens.length) * 100}%"
          ></div>
        </div>
      </div>
    {:else if modaleBatch.stato.fase === "completato"}
      {@const pass = modaleBatch.stato.risultati.filter((r) => r.passed).length}
      {@const fail = modaleBatch.stato.risultati.length - pass}
      <div class="batch-summary">
        <div class="batch-counts">
          <span class="batch-count pass">✓ {pass} pass</span>
          <span class="batch-count fail">✗ {fail} fail</span>
        </div>
        <ul class="batch-result-list" role="list">
          {#each modaleBatch.stato.risultati as r (r.etichetta)}
            <li class="batch-result" class:fail={!r.passed}>
              <span class="batch-result-icon">{r.passed ? "✓" : "✗"}</span>
              <span class="batch-result-name">{r.etichetta}</span>
              {#if r.errore}
                <span class="batch-result-err" title={r.errore}>{r.errore}</span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
    {#snippet footer()}
      {#if modaleBatch.stato.fase === "scelta"}
        <button class="secondary" type="button" onclick={chiudiBatch}>
          Annulla
        </button>
        <button
          class="primary"
          type="button"
          onclick={eseguiTutti}
          disabled={!modaleBatch.providerScelto ||
            !modaleBatch.modelScelto.trim() ||
            modaleBatch.providers.length === 0}
        >
          Esegui {goldens.length} test
        </button>
      {:else if modaleBatch.stato.fase === "esecuzione"}
        <button class="secondary" type="button" disabled>
          Esecuzione in corso…
        </button>
      {:else}
        <button class="primary" type="button" onclick={chiudiBatch}>
          Chiudi
        </button>
      {/if}
    {/snippet}
  </Modale>
{/if}

<style>
  .golden-tab {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2);
    background: var(--bg-canvas);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-1);
  }

  .titolo {
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-default);
  }

  .conteggio {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding: 1px 8px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
  }

  .primary {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 6px var(--sp-2);
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
  }

  .primary:hover {
    background: var(--accent-team-strong);
  }

  .vuoto {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    color: var(--text-muted);
    padding: var(--sp-4);
    gap: var(--sp-1);
  }

  .vuoto p {
    margin: 0;
    font-size: var(--fs-sm);
  }

  .muted {
    color: var(--text-muted);
  }

  .sub {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    max-width: 360px;
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .riga {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
  }

  .riga:hover {
    background: var(--bg-overlay);
  }

  .riga-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .etichetta {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge-row {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 4px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 6px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-default);
  }

  .badge.muted {
    color: var(--text-subtle);
    border-color: transparent;
    background: transparent;
  }

  .riga-azioni {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .ico:hover {
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  .ico-danger:hover {
    color: var(--danger);
  }

  /* M3 PR-3: header batch button */
  .secondary {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: var(--fs-xs);
    background: transparent;
    color: var(--text-default);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .secondary:hover:not(:disabled) {
    background: var(--bg-canvas);
  }
  .secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* M3 PR-3: modale batch */
  .batch-info,
  .batch-error,
  .batch-hint {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }
  .batch-error {
    color: var(--danger);
  }
  .batch-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .batch-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .batch-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
  }
  .batch-input {
    padding: 8px 12px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }
  .batch-progress-label {
    margin: 0 0 var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--text-default);
  }
  .batch-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-overlay);
    border-radius: var(--radius-full);
    overflow: hidden;
  }
  .batch-bar-fill {
    height: 100%;
    background: var(--accent-team);
    transition: width 200ms ease;
  }
  .batch-summary {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .batch-counts {
    display: flex;
    gap: var(--sp-3);
    font-weight: var(--fw-medium);
  }
  .batch-count.pass {
    color: var(--accent-success, #2c8a2c);
  }
  .batch-count.fail {
    color: var(--danger);
  }
  .batch-result-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 240px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .batch-result {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 10px;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
  }
  .batch-result.fail {
    background: rgba(220, 80, 80, 0.08);
  }
  .batch-result-icon {
    font-weight: var(--fw-bold);
  }
  .batch-result.fail .batch-result-icon {
    color: var(--danger);
  }
  .batch-result-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .batch-result-err {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    max-width: 50%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
