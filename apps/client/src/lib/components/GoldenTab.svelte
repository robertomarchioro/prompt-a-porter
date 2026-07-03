<script lang="ts">
  /**
   * Tab Test golden del DetailPane (F5 PR-C).
   *
   * Lista dei golden test del prompt con crea, modifica, elimina,
   * esecuzione singola (con dettaglio risultato) ed esecuzione batch
   * ("Esegui tutti").
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

  // Risultato di un'esecuzione, come restituito da `golden_esegui`.
  // Campi in snake_case (serde default lato Rust).
  interface Observation {
    id: string;
    prompt_version_id: string;
    golden_id: string | null;
    provider: string;
    model: string;
    actual_output: string;
    similarita: number | null;
    passed: boolean;
    latenza_ms: number | null;
    tokens_used: number | null;
    costo_stimato: number | null;
    errore: string | null;
    ran_at: string;
    ran_by: string;
  }

  interface ProviderConfigItem {
    provider: string;
    base_url?: string | null;
    default_model?: string | null;
    abilitato: boolean;
  }

  interface RisultatoBatch {
    id: string;
    etichetta: string;
    passed: boolean;
    errore?: string;
  }

  // Ultimo esito per golden (id → esito), aggiornato da run singole e batch,
  // per mostrare un badge di stato accanto a ciascun golden nella lista.
  interface Esito {
    passed: boolean;
    similarita: number | null;
    errore: boolean;
  }

  interface Props {
    promptId: string;
    onConteggio?: (n: number) => void;
  }

  let { promptId, onConteggio }: Props = $props();

  let goldens = $state<Golden[]>([]);
  let caricamento = $state(false);
  let ultimoEsito = $state<Record<string, Esito>>({});
  // Ultimo provider/modello usati: pre-riempiono i picker di run.
  let ultimoProviderScelto = $state("");
  let ultimoModelScelto = $state("");

  function registraEsito(id: string, obs: Observation): void {
    ultimoEsito = {
      ...ultimoEsito,
      [id]: {
        passed: obs.passed,
        similarita: obs.similarita,
        errore: !!obs.errore,
      },
    };
  }

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

  // Carica i soli provider abilitati; usato sia dal batch sia dalla run
  // singola. Ritorna l'errore user-facing invece di lanciare.
  async function caricaProviderAbilitati(): Promise<
    | { ok: true; providers: ProviderConfigItem[] }
    | { ok: false; errore: string }
  > {
    try {
      const lista = await invoke<ProviderConfigItem[]>("provider_config_lista");
      const providers = lista.filter((p) => p.abilitato);
      if (providers.length === 0) {
        return {
          ok: false,
          errore:
            "Nessun provider AI abilitato. Configura un provider in Impostazioni → Provider AI.",
        };
      }
      return { ok: true, providers };
    } catch (e) {
      return { ok: false, errore: String(e).replace(/^Error: /, "") };
    }
  }

  // Sceglie provider/modello iniziali: preferisce l'ultimo usato se ancora
  // tra gli abilitati, altrimenti il primo della lista.
  function providerIniziale(providers: ProviderConfigItem[]): {
    provider: string;
    model: string;
  } {
    const ultimo = providers.find((p) => p.provider === ultimoProviderScelto);
    if (ultimo) {
      return { provider: ultimo.provider, model: ultimoModelScelto };
    }
    const primo = providers[0];
    return { provider: primo.provider, model: primo.default_model ?? "" };
  }

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
    const res = await caricaProviderAbilitati();
    if (res.ok) {
      const init = providerIniziale(res.providers);
      modaleBatch.providers = res.providers;
      modaleBatch.providerScelto = init.provider;
      modaleBatch.modelScelto = init.model;
    } else {
      modaleBatch.errore = res.errore;
    }
    modaleBatch.caricamentoProviders = false;
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
    ultimoProviderScelto = modaleBatch.providerScelto;
    ultimoModelScelto = modaleBatch.modelScelto;
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
        registraEsito(g.id, obs);
        risultati.push({
          id: g.id,
          etichetta: g.etichetta,
          passed: obs.passed,
          errore: obs.errore ?? undefined,
        });
      } catch (e) {
        risultati.push({
          id: g.id,
          etichetta: g.etichetta,
          passed: false,
          errore: String(e).replace(/^Error: /, ""),
        });
      }
    }
    modaleBatch.stato = { fase: "completato", risultati };
  }

  // --- Esecuzione singola con dettaglio risultato ---

  type StatoRun =
    | { fase: "scelta" }
    | { fase: "esecuzione" }
    | { fase: "completato"; obs: Observation };

  let modaleRun = $state<{
    aperto: boolean;
    golden: Golden | null;
    providers: ProviderConfigItem[];
    providerScelto: string;
    modelScelto: string;
    caricamentoProviders: boolean;
    stato: StatoRun;
    errore: string;
    mostraOutput: boolean;
  }>({
    aperto: false,
    golden: null,
    providers: [],
    providerScelto: "",
    modelScelto: "",
    caricamentoProviders: false,
    stato: { fase: "scelta" },
    errore: "",
    mostraOutput: false,
  });

  async function apriRun(g: Golden): Promise<void> {
    modaleRun = {
      aperto: true,
      golden: g,
      providers: [],
      providerScelto: "",
      modelScelto: "",
      caricamentoProviders: true,
      stato: { fase: "scelta" },
      errore: "",
      mostraOutput: false,
    };
    const res = await caricaProviderAbilitati();
    if (res.ok) {
      const init = providerIniziale(res.providers);
      modaleRun.providers = res.providers;
      modaleRun.providerScelto = init.provider;
      modaleRun.modelScelto = init.model;
    } else {
      modaleRun.errore = res.errore;
    }
    modaleRun.caricamentoProviders = false;
  }

  function chiudiRun(): void {
    if (modaleRun.stato.fase === "esecuzione") return; // no chiusura mid-run
    modaleRun.aperto = false;
  }

  function onCambiaProviderRun(): void {
    const p = modaleRun.providers.find(
      (x) => x.provider === modaleRun.providerScelto,
    );
    modaleRun.modelScelto = p?.default_model ?? "";
  }

  async function eseguiSingolo(): Promise<void> {
    const g = modaleRun.golden;
    if (!g || !modaleRun.providerScelto || !modaleRun.modelScelto.trim()) {
      return;
    }
    ultimoProviderScelto = modaleRun.providerScelto;
    ultimoModelScelto = modaleRun.modelScelto;
    modaleRun.errore = "";
    modaleRun.stato = { fase: "esecuzione" };
    try {
      const obs = await invoke<Observation>("golden_esegui", {
        goldenId: g.id,
        providerKind: modaleRun.providerScelto,
        model: modaleRun.modelScelto,
      });
      registraEsito(g.id, obs);
      modaleRun.stato = { fase: "completato", obs };
    } catch (e) {
      modaleRun.errore = String(e).replace(/^Error: /, "");
      modaleRun.stato = { fase: "scelta" };
    }
  }

  function fmtPercento(v: number | null): string {
    return v === null ? "—" : `${(v * 100).toFixed(0)}%`;
  }

  // --- Modale crea golden (#382) ---

  const SIMILARITY_FN_VALIDE = [
    "cosine",
    "llm-judge",
    "exact-match",
    "regex",
  ] as const;

  type SimilarityFn = (typeof SIMILARITY_FN_VALIDE)[number];

  // Lo stesso modale serve sia per creare sia per modificare: `modalita`
  // sceglie il comando backend, `id` è valorizzato solo in modifica.
  let modaleEditor = $state<{
    aperto: boolean;
    modalita: "crea" | "modifica";
    id: string | null;
    etichetta: string;
    input_vars: string;
    expected_output: string;
    similarity_fn: SimilarityFn;
    soglia_tolleranza: number;
    invio: boolean;
    errore: string;
  }>({
    aperto: false,
    modalita: "crea",
    id: null,
    etichetta: "",
    input_vars: "{}",
    expected_output: "",
    similarity_fn: "cosine",
    soglia_tolleranza: 0.85,
    invio: false,
    errore: "",
  });

  function apriEditor(): void {
    modaleEditor = {
      aperto: true,
      modalita: "crea",
      id: null,
      etichetta: "",
      input_vars: "{}",
      expected_output: "",
      similarity_fn: "cosine",
      soglia_tolleranza: 0.85,
      invio: false,
      errore: "",
    };
  }

  function apriEditorModifica(g: Golden): void {
    modaleEditor = {
      aperto: true,
      modalita: "modifica",
      id: g.id,
      etichetta: g.etichetta,
      input_vars: g.input_vars,
      expected_output: g.expected_output,
      similarity_fn: g.similarity_fn as SimilarityFn,
      soglia_tolleranza: g.soglia_tolleranza,
      invio: false,
      errore: "",
    };
  }

  function chiudiEditor(): void {
    if (modaleEditor.invio) return;
    modaleEditor = { ...modaleEditor, aperto: false };
  }

  async function salvaGolden(): Promise<void> {
    modaleEditor = { ...modaleEditor, invio: true, errore: "" };
    try {
      if (modaleEditor.modalita === "modifica" && modaleEditor.id) {
        await invoke("golden_aggiorna", {
          dati: {
            id: modaleEditor.id,
            etichetta: modaleEditor.etichetta.trim(),
            input_vars: modaleEditor.input_vars,
            expected_output: modaleEditor.expected_output,
            similarity_fn: modaleEditor.similarity_fn,
            soglia_tolleranza: modaleEditor.soglia_tolleranza,
          },
        });
      } else {
        await invoke("golden_crea", {
          dati: {
            prompt_id: promptId,
            etichetta: modaleEditor.etichetta.trim(),
            input_vars: modaleEditor.input_vars,
            expected_output: modaleEditor.expected_output,
            similarity_fn: modaleEditor.similarity_fn,
            soglia_tolleranza: modaleEditor.soglia_tolleranza,
          },
        });
      }
      modaleEditor = { ...modaleEditor, aperto: false, invio: false };
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      modaleEditor = {
        ...modaleEditor,
        invio: false,
        errore: String(e).replace(/^Error: /, ""),
      };
    }
  }

  // -------------------------------------------------

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
      onclick={apriEditor}
      title="Crea un nuovo golden test"
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
              {#if ultimoEsito[g.id]}
                {@const es = ultimoEsito[g.id]}
                <span
                  class="badge esito"
                  class:pass={es.passed && !es.errore}
                  class:fail={!es.passed || es.errore}
                  title={es.errore
                    ? "Ultima esecuzione: errore"
                    : es.passed
                      ? "Ultima esecuzione: passato"
                      : "Ultima esecuzione: fallito"}
                >
                  {es.errore
                    ? "⚠ errore"
                    : `${es.passed ? "✓" : "✗"} ${fmtPercento(es.similarita)}`}
                </span>
              {/if}
            </div>
          </div>
          <div class="riga-azioni">
            <button
              class="ico"
              type="button"
              title="Esegui questo golden"
              aria-label="Esegui golden"
              onclick={() => apriRun(g)}
            >
              <Play size={14} />
            </button>
            <button
              class="ico"
              type="button"
              title="Modifica golden"
              aria-label="Modifica golden"
              onclick={() => apriEditorModifica(g)}
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

{#if modaleRun.aperto}
  <Modale
    titolo="Esegui golden"
    sottotitolo={modaleRun.golden?.etichetta}
    larghezza="md"
    onChiudi={chiudiRun}
  >
    {#if modaleRun.caricamentoProviders}
      <p class="batch-info">Caricamento provider…</p>
    {:else if modaleRun.errore}
      <p class="batch-error">{modaleRun.errore}</p>
    {:else if modaleRun.stato.fase === "scelta"}
      <div class="batch-form">
        <label class="batch-field">
          <span class="batch-label">Provider</span>
          <select
            class="batch-input"
            bind:value={modaleRun.providerScelto}
            onchange={onCambiaProviderRun}
          >
            {#each modaleRun.providers as p (p.provider)}
              <option value={p.provider}>{p.provider}</option>
            {/each}
          </select>
        </label>
        <label class="batch-field">
          <span class="batch-label">Modello</span>
          <input
            class="batch-input"
            type="text"
            bind:value={modaleRun.modelScelto}
            placeholder="Es. claude-sonnet-4-6, gpt-4o, llama3.2…"
          />
        </label>
        {#if modaleRun.golden?.similarity_fn === "llm-judge"}
          <p class="batch-hint run-warn">
            Questo golden usa <code>llm-judge</code>, che richiede un provider
            giudice ancora non selezionabile da qui: l'esecuzione registrerà
            un errore finché non arriva quel supporto.
          </p>
        {/if}
      </div>
    {:else if modaleRun.stato.fase === "esecuzione"}
      <p class="batch-info">Esecuzione in corso… chiamata al provider AI.</p>
    {:else if modaleRun.stato.fase === "completato"}
      {@const obs = modaleRun.stato.obs}
      <div class="run-result">
        {#if obs.errore}
          <div class="run-esito err">
            <span class="run-esito-icon">⚠</span>
            <span>Esecuzione fallita</span>
          </div>
          <p class="run-error-msg">{obs.errore}</p>
        {:else}
          <div class="run-esito" class:pass={obs.passed} class:fail={!obs.passed}>
            <span class="run-esito-icon">{obs.passed ? "✓" : "✗"}</span>
            <span>{obs.passed ? "Passato" : "Fallito"}</span>
          </div>
          <div class="run-score">
            <div class="run-score-head">
              <span>Similarità <strong>{fmtPercento(obs.similarita)}</strong></span>
              <span class="run-score-soglia">
                soglia {fmtPercento(modaleRun.golden?.soglia_tolleranza ?? null)}
              </span>
            </div>
            <div class="run-bar">
              <div
                class="run-bar-fill"
                class:pass={obs.passed}
                class:fail={!obs.passed}
                style:width="{(obs.similarita ?? 0) * 100}%"
              ></div>
              <div
                class="run-bar-soglia"
                style:left="{(modaleRun.golden?.soglia_tolleranza ?? 0) * 100}%"
                title="Soglia"
              ></div>
            </div>
          </div>
        {/if}

        <div class="run-meta">
          <span class="run-meta-item">{obs.provider} · {obs.model}</span>
          {#if obs.latenza_ms !== null}
            <span class="run-meta-item">{obs.latenza_ms} ms</span>
          {/if}
          {#if obs.tokens_used !== null}
            <span class="run-meta-item">{obs.tokens_used} token</span>
          {/if}
        </div>

        {#if !obs.errore}
          <div class="run-io">
            <div class="run-io-block">
              <span class="run-io-label">Output ricevuto</span>
              <pre class="run-io-text">{obs.actual_output || "(vuoto)"}</pre>
            </div>
            <details class="run-io-atteso">
              <summary>Confronta con l'output atteso</summary>
              <pre class="run-io-text">{modaleRun.golden?.expected_output ?? ""}</pre>
            </details>
          </div>
        {/if}
      </div>
    {/if}
    {#snippet footer()}
      {#if modaleRun.stato.fase === "scelta"}
        <button class="secondary" type="button" onclick={chiudiRun}>
          Annulla
        </button>
        <button
          class="primary"
          type="button"
          onclick={eseguiSingolo}
          disabled={!modaleRun.providerScelto ||
            !modaleRun.modelScelto.trim() ||
            modaleRun.providers.length === 0}
        >
          Esegui
        </button>
      {:else if modaleRun.stato.fase === "esecuzione"}
        <button class="secondary" type="button" disabled>
          Esecuzione in corso…
        </button>
      {:else}
        <button
          class="secondary"
          type="button"
          onclick={() => (modaleRun.stato = { fase: "scelta" })}
        >
          Ri-esegui
        </button>
        <button class="primary" type="button" onclick={chiudiRun}>Chiudi</button>
      {/if}
    {/snippet}
  </Modale>
{/if}

{#if modaleEditor.aperto}
  <Modale
    titolo={modaleEditor.modalita === "modifica"
      ? "Modifica golden test"
      : "Nuovo golden test"}
    sottotitolo={modaleEditor.modalita === "modifica"
      ? "Aggiorna il caso di test atteso"
      : "Definisci un caso di test atteso per questo prompt"}
    larghezza="md"
    onChiudi={chiudiEditor}
  >
    <div class="editor-form">
      <label class="editor-field" for="editor-etichetta">
        <span class="editor-label">Etichetta</span>
        <input
          id="editor-etichetta"
          class="editor-input"
          type="text"
          bind:value={modaleEditor.etichetta}
          placeholder="Es. risposta breve, caso limite vuoto..."
          autocomplete="off"
        />
      </label>
      <label class="editor-field" for="editor-input-vars">
        <span class="editor-label">Variabili di input (JSON)</span>
        <textarea
          id="editor-input-vars"
          class="editor-input editor-textarea"
          bind:value={modaleEditor.input_vars}
          placeholder={"{}"}
          rows={4}
          spellcheck={false}
        ></textarea>
      </label>
      <label class="editor-field" for="editor-expected-output">
        <span class="editor-label">Output atteso</span>
        <textarea
          id="editor-expected-output"
          class="editor-input editor-textarea"
          bind:value={modaleEditor.expected_output}
          placeholder="Testo atteso, regex o prompt giudice..."
          rows={4}
          spellcheck={false}
        ></textarea>
      </label>
      <label class="editor-field" for="editor-similarity-fn">
        <span class="editor-label">Funzione di similarita</span>
        <select
          id="editor-similarity-fn"
          class="editor-input"
          bind:value={modaleEditor.similarity_fn}
        >
          {#each SIMILARITY_FN_VALIDE as fn (fn)}
            <option value={fn}>{fn}</option>
          {/each}
        </select>
      </label>
      <label class="editor-field" for="editor-soglia">
        <span class="editor-label">Soglia tolleranza (0-1, default 0.85)</span>
        <input
          id="editor-soglia"
          class="editor-input"
          type="number"
          min={0}
          max={1}
          step={0.01}
          bind:value={modaleEditor.soglia_tolleranza}
        />
      </label>
      {#if modaleEditor.errore}
        <p class="editor-errore" role="alert">{modaleEditor.errore}</p>
      {/if}
    </div>
    {#snippet footer()}
      <button
        class="secondary"
        type="button"
        onclick={chiudiEditor}
        disabled={modaleEditor.invio}
      >
        Annulla
      </button>
      <button
        class="primary"
        type="button"
        onclick={salvaGolden}
        disabled={modaleEditor.invio ||
          !modaleEditor.etichetta.trim() ||
          !modaleEditor.expected_output.trim()}
      >
        {modaleEditor.invio
          ? "Salvataggio..."
          : modaleEditor.modalita === "modifica"
            ? "Salva modifiche"
            : "Salva golden"}
      </button>
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

  /* Badge di ultimo esito nella riga golden */
  .badge.esito {
    font-family: var(--font-ui);
    font-weight: var(--fw-medium);
    border-color: transparent;
  }
  .badge.esito.pass {
    color: var(--accent-success, #2c8a2c);
    background: rgba(44, 138, 44, 0.12);
  }
  .badge.esito.fail {
    color: var(--danger);
    background: rgba(220, 80, 80, 0.12);
  }

  /* Esecuzione singola: dettaglio risultato */
  .run-warn code {
    font-family: var(--font-mono);
  }
  .run-result {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .run-esito {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-weight: var(--fw-semibold);
    font-size: var(--fs-base);
  }
  .run-esito.pass {
    color: var(--accent-success, #2c8a2c);
  }
  .run-esito.fail,
  .run-esito.err {
    color: var(--danger);
  }
  .run-esito-icon {
    font-size: 1.2em;
  }
  .run-error-msg {
    margin: 0;
    padding: 8px 10px;
    background: rgba(220, 80, 80, 0.08);
    border-radius: var(--radius-sm);
    color: var(--danger);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
  }
  .run-score {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .run-score-head {
    display: flex;
    justify-content: space-between;
    font-size: var(--fs-sm);
    color: var(--text-default);
  }
  .run-score-soglia {
    color: var(--text-muted);
  }
  .run-bar {
    position: relative;
    width: 100%;
    height: 8px;
    background: var(--bg-overlay);
    border-radius: var(--radius-full);
    overflow: hidden;
  }
  .run-bar-fill {
    height: 100%;
    border-radius: var(--radius-full);
  }
  .run-bar-fill.pass {
    background: var(--accent-success, #2c8a2c);
  }
  .run-bar-fill.fail {
    background: var(--danger);
  }
  .run-bar-soglia {
    position: absolute;
    top: -2px;
    width: 2px;
    height: 12px;
    background: var(--text-default);
    transform: translateX(-1px);
  }
  .run-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .run-meta-item {
    font-family: var(--font-mono);
  }
  .run-io {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .run-io-block {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .run-io-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
  }
  .run-io-text {
    margin: 0;
    padding: 8px 10px;
    max-height: 200px;
    overflow: auto;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    white-space: pre-wrap;
    word-break: break-word;
  }
  .run-io-atteso summary {
    cursor: pointer;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .run-io-atteso[open] summary {
    margin-bottom: 4px;
  }

  /* #382: modale editor golden */
  .editor-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .editor-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .editor-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
  }
  .editor-input {
    padding: 8px 12px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }
  .editor-textarea {
    font-family: var(--font-mono);
    resize: vertical;
    min-height: 80px;
  }
  .editor-errore {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--danger);
  }
</style>
