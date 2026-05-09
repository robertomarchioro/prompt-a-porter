<script lang="ts">
  /**
   * F8 PR-C — Modale Regressioni.
   *
   * Porting di Regressioni.svelte legacy (368 righe) come modale,
   * usando la primitive Modale + statoModale store.
   *
   * Tabella drift score per (prompt × provider × model) ultimi N giorni
   * + esportazione CSV. Slider 1-90gg sostituisce la select legacy
   * (7/30/90/365), come da blueprint F8 PR-C.
   *
   * Riferimenti:
   * - Blueprint: docs/roadmap/redesign-v08/blueprint-F8.md §3
   * - Cmd backend: src-tauri/src/regression.rs (regression_report,
   *   regression_report_csv)
   */
  import { invoke } from "@tauri-apps/api/core";
  import { Download } from "lucide-svelte";
  import Modale from "$lib/components/Modale.svelte";

  interface ReportRow {
    prompt_id: string;
    prompt_titolo: string;
    provider: string;
    model: string;
    num_run: number;
    num_passed: number;
    num_failed: number;
    similarita_media: number | null;
    similarita_ultima: number | null;
    ultima_run_at: string | null;
    drift_percentuale: number | null;
  }

  interface Props {
    onChiudi: () => void;
  }

  let { onChiudi }: Props = $props();

  let righe = $state<ReportRow[]>([]);
  let caricamento = $state(true);
  let errore = $state<string | null>(null);
  let giorni = $state(30);

  async function carica(): Promise<void> {
    caricamento = true;
    errore = null;
    try {
      righe = await invoke<ReportRow[]>("regression_report", { giorni });
    } catch (e) {
      errore = String(e);
      righe = [];
    } finally {
      caricamento = false;
    }
  }

  $effect(() => {
    void giorni;
    void carica();
  });

  async function esportaCsv(): Promise<void> {
    try {
      const csv = await invoke<string>("regression_report_csv", { giorni });
      const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `regressioni-${giorni}g-${new Date().toISOString().slice(0, 10)}.csv`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (e) {
      errore = String(e);
    }
  }

  function formatPercentuale(v: number | null): string {
    if (v === null) return "—";
    return `${v.toFixed(2)}%`;
  }

  function formatSimilarita(v: number | null): string {
    if (v === null) return "—";
    return v.toFixed(3);
  }

  function classeDrift(v: number | null): string {
    if (v === null) return "drift-nd";
    if (v > 5) return "drift-alto";
    if (v > 0) return "drift-leggero";
    if (v < -5) return "drift-migliorato";
    return "drift-stabile";
  }

  function classeStato(passed: number, totale: number): string {
    if (totale === 0) return "stato-nd";
    const ratio = passed / totale;
    if (ratio === 1) return "stato-ok";
    if (ratio >= 0.7) return "stato-misto";
    return "stato-ko";
  }
</script>

<Modale
  titolo="Regressioni"
  sottotitolo="Drift score per prompt × provider × model"
  larghezza="xl"
  {onChiudi}
>
  <div class="filtri-bar">
    <label class="slider-wrap">
      <span class="slider-lbl">
        Periodo: <strong>{giorni} {giorni === 1 ? "giorno" : "giorni"}</strong>
      </span>
      <input
        type="range"
        min="1"
        max="90"
        bind:value={giorni}
        aria-label="Giorni di periodo"
      />
    </label>
    <button
      type="button"
      class="btn-csv"
      onclick={esportaCsv}
      disabled={righe.length === 0}
      title="Esporta CSV"
    >
      <Download size={14} />
      <span>Esporta CSV</span>
    </button>
  </div>

  {#if caricamento}
    <div class="caricamento">Caricamento report…</div>
  {:else if errore}
    <div class="errore-box">
      <strong>Errore</strong>
      <span>{errore}</span>
    </div>
  {:else if righe.length === 0}
    <div class="vuoto">
      <strong>Nessuna regressione da mostrare</strong>
      <p>
        Esegui un golden dall'editor di un prompt per popolare il report.
        Vedi pannello Test nell'EditorPrompt.
      </p>
    </div>
  {:else}
    <p class="hint">
      {righe.length} combinazioni prompt × modello negli ultimi {giorni}
      {giorni === 1 ? "giorno" : "giorni"}. Ordinate per ultima esecuzione
      discendente. Drift positivo = peggioramento rispetto alla media del
      periodo.
    </p>

    <div class="reg-tabella">
      <div class="reg-head">
        <div class="reg-cell reg-cell--prompt">Prompt</div>
        <div class="reg-cell reg-cell--model">Provider · Model</div>
        <div class="reg-cell reg-cell--num">Run</div>
        <div class="reg-cell reg-cell--num">Pass</div>
        <div class="reg-cell reg-cell--num">Fail</div>
        <div class="reg-cell reg-cell--sim">Sim. media</div>
        <div class="reg-cell reg-cell--sim">Sim. ultima</div>
        <div class="reg-cell reg-cell--drift">Drift</div>
        <div class="reg-cell reg-cell--at">Ultima run</div>
      </div>

      {#each righe as r (r.prompt_id + r.provider + r.model)}
        <div class="reg-row reg-row--{classeStato(r.num_passed, r.num_run)}">
          <div class="reg-cell reg-cell--prompt" title={r.prompt_id}>
            {r.prompt_titolo}
          </div>
          <div class="reg-cell reg-cell--model">
            <span class="provider-pill">{r.provider}</span>
            <span class="model-name">{r.model}</span>
          </div>
          <div class="reg-cell reg-cell--num">{r.num_run}</div>
          <div class="reg-cell reg-cell--num reg-pass">{r.num_passed}</div>
          <div class="reg-cell reg-cell--num reg-fail">{r.num_failed}</div>
          <div class="reg-cell reg-cell--sim">
            {formatSimilarita(r.similarita_media)}
          </div>
          <div class="reg-cell reg-cell--sim">
            {formatSimilarita(r.similarita_ultima)}
          </div>
          <div
            class="reg-cell reg-cell--drift {classeDrift(r.drift_percentuale)}"
          >
            {formatPercentuale(r.drift_percentuale)}
          </div>
          <div class="reg-cell reg-cell--at" title={r.ultima_run_at ?? ""}>
            {r.ultima_run_at?.slice(0, 16) ?? "—"}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</Modale>

<style>
  .filtri-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    margin-bottom: var(--sp-3);
    padding-bottom: var(--sp-2);
    border-bottom: 1px dashed var(--border-subtle);
  }

  .slider-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    max-width: 480px;
  }

  .slider-lbl {
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .slider-lbl strong {
    color: var(--text-strong);
    font-variant-numeric: tabular-nums;
  }

  .slider-wrap input[type="range"] {
    width: 100%;
    accent-color: var(--accent-team);
  }

  .btn-csv {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    cursor: pointer;
    flex-shrink: 0;
  }

  .btn-csv:hover:not(:disabled) {
    background: var(--bg-overlay);
  }

  .btn-csv:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .caricamento {
    text-align: center;
    padding: var(--sp-5) 0;
    color: var(--text-muted);
  }

  .errore-box {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .errore-box strong {
    color: var(--text-strong);
  }

  .errore-box span {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    word-break: break-word;
  }

  .vuoto {
    padding: var(--sp-5);
    text-align: center;
    background: var(--bg-input);
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .vuoto strong {
    display: block;
    color: var(--text-strong);
    font-size: var(--fs-md);
    margin-bottom: 4px;
  }

  .vuoto p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .hint {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    margin: 0 0 var(--sp-2) 0;
  }

  .reg-tabella {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    overflow: hidden;
    font-size: var(--fs-sm);
  }

  .reg-head,
  .reg-row {
    display: grid;
    grid-template-columns: 2fr 2fr 60px 60px 60px 90px 90px 90px 140px;
    align-items: center;
  }

  .reg-head {
    background: var(--bg-input);
    color: var(--text-muted);
    font-weight: var(--fw-medium);
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .reg-row {
    border-top: 1px solid var(--border-subtle);
  }

  .reg-row--stato-ok {
    border-left: 3px solid var(--accent-success, #6cb86c);
  }

  .reg-row--stato-ko {
    border-left: 3px solid var(--accent-danger, #d9534f);
  }

  .reg-row--stato-misto {
    border-left: 3px solid var(--accent-warning, var(--warning, #d2a85f));
  }

  .reg-row--stato-nd {
    border-left: 3px solid var(--border-subtle);
  }

  .reg-cell {
    padding: var(--sp-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reg-cell--prompt {
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }

  .reg-cell--model {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .provider-pill {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 6px;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .model-name {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }

  .reg-cell--num,
  .reg-cell--sim,
  .reg-cell--drift {
    text-align: right;
    font-family: var(--font-mono);
  }

  .reg-cell--at {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .reg-pass {
    color: var(--accent-success, #6cb86c);
  }

  .reg-fail {
    color: var(--accent-danger, #d9534f);
  }

  .drift-alto {
    color: var(--accent-danger, #d9534f);
    font-weight: var(--fw-semibold);
  }

  .drift-leggero {
    color: var(--accent-warning, var(--warning, #d2a85f));
  }

  .drift-stabile {
    color: var(--text-muted);
  }

  .drift-migliorato {
    color: var(--accent-success, #6cb86c);
  }

  .drift-nd {
    color: var(--text-muted);
  }
</style>
