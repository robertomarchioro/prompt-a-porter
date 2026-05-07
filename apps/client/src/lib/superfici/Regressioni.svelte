<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, EmptyState } from "$lib/components";

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
    onchiudi: () => void;
  }

  let { onchiudi }: Props = $props();

  let righe = $state<ReportRow[]>([]);
  let caricamento = $state(true);
  let errore = $state<string | null>(null);
  let giorni = $state(30);

  async function carica() {
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
    carica();
  });

  async function esportaCsv() {
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

<div class="modale-overlay">
  <div class="modale">
    <header class="modale-head">
      <h2>Regressioni</h2>
      <div class="head-actions">
        <label class="head-filtro">
          Periodo
          <select bind:value={giorni}>
            <option value={7}>ultimi 7 giorni</option>
            <option value={30}>ultimi 30 giorni</option>
            <option value={90}>ultimi 90 giorni</option>
            <option value={365}>ultimo anno</option>
          </select>
        </label>
        <Button
          variante="ghost"
          onclick={esportaCsv}
          disabled={righe.length === 0}>Esporta CSV</Button
        >
        <Button variante="ghost" onclick={onchiudi}>Chiudi</Button>
      </div>
    </header>

    <div class="modale-corpo">
      {#if caricamento}
        <div class="caricamento">Caricamento report…</div>
      {:else if errore}
        <EmptyState titolo="Errore" hint={errore} />
      {:else if righe.length === 0}
        <EmptyState
          titolo="Nessuna regressione da mostrare"
          hint="Esegui un golden dall'editor di un prompt per popolare il report. Vedi pannello Test nell'EditorPrompt."
        />
      {:else}
        <p class="hint">
          {righe.length} combinazioni prompt × modello negli ultimi {giorni}
          giorni. Ordinate per ultima esecuzione discendente. Il drift positivo
          indica peggioramento rispetto alla media del periodo.
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
              <div class="reg-cell reg-cell--drift {classeDrift(r.drift_percentuale)}">
                {formatPercentuale(r.drift_percentuale)}
              </div>
              <div class="reg-cell reg-cell--at" title={r.ultima_run_at ?? ""}>
                {r.ultima_run_at?.slice(0, 16) ?? "—"}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .modale-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modale {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    width: min(95vw, 1280px);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
  }

  .modale-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .modale-head h2 {
    margin: 0;
  }

  .head-actions {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .head-filtro {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--fs-sm);
    color: var(--text-default);
  }

  .head-filtro select {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    background: var(--bg-surface);
    color: var(--text-default);
    font-size: var(--fs-sm);
  }

  .modale-corpo {
    overflow: auto;
    padding: var(--space-3) var(--space-4);
  }

  .caricamento {
    text-align: center;
    padding: var(--space-6) 0;
    color: var(--text-subtle);
  }

  .hint {
    color: var(--text-subtle);
    font-size: var(--fs-sm);
    margin: 0 0 var(--space-3) 0;
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
    background: var(--bg-surface);
    color: var(--text-subtle);
    font-weight: 500;
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
    border-left: 3px solid var(--accent-warn, #d2a85f);
  }

  .reg-row--stato-nd {
    border-left: 3px solid var(--border-subtle);
  }

  .reg-cell {
    padding: var(--space-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reg-cell--prompt {
    font-weight: 500;
    color: var(--text-default);
  }

  .reg-cell--model {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .provider-pill {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    padding: 2px 6px;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
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
    color: var(--text-subtle);
  }

  .reg-pass {
    color: var(--accent-success, #6cb86c);
  }

  .reg-fail {
    color: var(--accent-danger, #d9534f);
  }

  .drift-alto {
    color: var(--accent-danger, #d9534f);
    font-weight: 600;
  }

  .drift-leggero {
    color: var(--accent-warn, #d2a85f);
  }

  .drift-stabile {
    color: var(--text-subtle);
  }

  .drift-migliorato {
    color: var(--accent-success, #6cb86c);
  }

  .drift-nd {
    color: var(--text-subtle);
  }
</style>
