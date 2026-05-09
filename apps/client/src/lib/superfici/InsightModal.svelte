<script lang="ts">
  /**
   * F8 PR-B — Modale Insight.
   *
   * Porting di Insight.svelte legacy (573 righe) come modale, usando la
   * primitive Modale + statoModale store.
   *
   * Tutte le statistiche sono calcolate localmente sul vault (cmd
   * statistiche_query). Aggiunto `token_medi` (V014) come KPI in
   * Panoramica.
   *
   * Riferimenti:
   * - Blueprint: docs/roadmap/redesign-v08/blueprint-F8.md §2
   * - Cmd backend: src-tauri/src/statistiche.rs
   */
  import { invoke } from "@tauri-apps/api/core";
  import Modale from "$lib/components/Modale.svelte";
  import { etichettaPerValore } from "$lib/modelli-target";

  interface Totali {
    prompt_attivi: number;
    prompt_eliminati: number;
    tag_attivi: number;
    creati_ultimi_30g: number;
    aggiornati_ultimi_30g: number;
    totale_versioni: number;
  }

  interface PromptUsato {
    id: string;
    titolo: string;
    uso_count: number;
    ultimo_uso: string | null;
  }

  interface PromptInattivo {
    id: string;
    titolo: string;
    aggiornato_a: string;
    giorni_inattivo: number;
  }

  interface DistribuzioneTag {
    id: string;
    nome: string;
    colore: string;
    conteggio: number;
  }

  interface DistribuzioneStringa {
    valore: string;
    conteggio: number;
  }

  interface PromptImportato {
    id: string;
    titolo: string;
    conteggio_importatori: number;
  }

  interface LintHealth {
    totale_prompt: number;
    prompt_senza_issue: number;
    percentuale_health: number;
    top_categorie: DistribuzioneStringa[];
  }

  interface Statistiche {
    totali: Totali;
    top_usati: PromptUsato[];
    non_usati: PromptInattivo[];
    per_tag: DistribuzioneTag[];
    per_target_model: DistribuzioneStringa[];
    per_visibilita: DistribuzioneStringa[];
    top_importati: PromptImportato[];
    lint_health: LintHealth;
    token_medi: number;
  }

  interface Props {
    onChiudi: () => void;
  }

  let { onChiudi }: Props = $props();

  let dati = $state<Statistiche | null>(null);
  let caricamento = $state(true);
  let errore = $state<string | null>(null);

  async function carica(): Promise<void> {
    caricamento = true;
    errore = null;
    try {
      dati = await invoke<Statistiche>("statistiche_query");
    } catch (e) {
      errore = String(e);
    } finally {
      caricamento = false;
    }
  }

  $effect(() => {
    void carica();
  });

  function maxConteggio(items: { conteggio: number }[]): number {
    return items.reduce((m, it) => Math.max(m, it.conteggio), 1);
  }

  function etichettaModello(valore: string): string {
    if (valore === "(non specificato)") return valore;
    return etichettaPerValore(valore) || valore;
  }

  function etichettaVisibilita(valore: string): string {
    return valore === "private"
      ? "Privato"
      : valore === "workspace"
        ? "Team"
        : valore;
  }
</script>

<Modale
  titolo="Insight"
  sottotitolo="Statistiche locali sul vault — nessun dato esce dal computer"
  larghezza="lg"
  {onChiudi}
>
  {#if caricamento}
    <div class="caricamento">Caricamento statistiche…</div>
  {:else if errore}
    <div class="errore-box">
      <strong>Errore</strong>
      <span>{errore}</span>
    </div>
  {:else if dati}
    <!-- Panoramica -->
    <section class="sezione">
      <h3>Panoramica</h3>
      <div class="kpi-grid">
        <div class="kpi">
          <div class="kpi-num">{dati.totali.prompt_attivi}</div>
          <div class="kpi-lbl">Prompt attivi</div>
        </div>
        <div class="kpi">
          <div class="kpi-num">{dati.totali.tag_attivi}</div>
          <div class="kpi-lbl">Tag</div>
        </div>
        <div class="kpi">
          <div class="kpi-num">{dati.totali.creati_ultimi_30g}</div>
          <div class="kpi-lbl">Creati 30g</div>
        </div>
        <div class="kpi">
          <div class="kpi-num">{dati.totali.aggiornati_ultimi_30g}</div>
          <div class="kpi-lbl">Aggiornati 30g</div>
        </div>
        <div class="kpi">
          <div class="kpi-num">{dati.totali.totale_versioni}</div>
          <div class="kpi-lbl">Versioni storiche</div>
        </div>
        <div class="kpi">
          <div class="kpi-num">{dati.totali.prompt_eliminati}</div>
          <div class="kpi-lbl">Cestinati</div>
        </div>
        <div class="kpi" title="Media char-count Body / 4 (proxy token cl100k)">
          <div class="kpi-num">~{dati.token_medi}</div>
          <div class="kpi-lbl">Token medi</div>
        </div>
      </div>
    </section>

    <!-- Top usati -->
    <section class="sezione">
      <h3>Top prompt usati (ultimi 30g)</h3>
      {#if dati.top_usati.length === 0}
        <p class="vuoto">Nessun prompt usato di recente.</p>
      {:else}
        {@const maxUso = maxConteggio(
          dati.top_usati.map((p) => ({ conteggio: p.uso_count })),
        )}
        <ul class="bar-list">
          {#each dati.top_usati as p (p.id)}
            <li class="bar-row">
              <div class="bar-label" title={p.titolo}>{p.titolo}</div>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--accent"
                  style:width="{(p.uso_count / maxUso) * 100}%"
                ></div>
              </div>
              <div class="bar-num">{p.uso_count}</div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Non usati -->
    <section class="sezione">
      <h3>Candidati a cleanup (> 90 giorni inattivi)</h3>
      {#if dati.non_usati.length === 0}
        <p class="vuoto">
          Nessun prompt è inattivo da oltre 90 giorni. Tutto in salute.
        </p>
      {:else}
        <ul class="lista-inattivi">
          {#each dati.non_usati as p (p.id)}
            <li class="riga-inattivo">
              <span class="riga-titolo" title={p.titolo}>{p.titolo}</span>
              <span class="riga-meta">{p.giorni_inattivo} giorni</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Distribuzione per tag -->
    <section class="sezione">
      <h3>Distribuzione per tag</h3>
      {#if dati.per_tag.length === 0}
        <p class="vuoto">Nessun tag in uso.</p>
      {:else}
        {@const maxTag = maxConteggio(dati.per_tag)}
        <ul class="bar-list">
          {#each dati.per_tag as t (t.id)}
            <li class="bar-row">
              <div class="bar-label">
                <span
                  class="tag-dot"
                  style:background={t.colore || "var(--text-subtle)"}
                ></span>
                {t.nome}
              </div>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--tag"
                  style:width="{(t.conteggio / maxTag) * 100}%"
                  style:background={t.colore || "var(--accent-team)"}
                ></div>
              </div>
              <div class="bar-num">{t.conteggio}</div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Distribuzione per modello target -->
    <section class="sezione">
      <h3>Distribuzione per modello target</h3>
      {#if dati.per_target_model.length === 0}
        <p class="vuoto">Nessun prompt nel vault.</p>
      {:else}
        {@const maxTm = maxConteggio(dati.per_target_model)}
        <ul class="bar-list">
          {#each dati.per_target_model as d (d.valore)}
            <li class="bar-row">
              <div class="bar-label">{etichettaModello(d.valore)}</div>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--model"
                  style:width="{(d.conteggio / maxTm) * 100}%"
                ></div>
              </div>
              <div class="bar-num">{d.conteggio}</div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Distribuzione per visibilità -->
    <section class="sezione">
      <h3>Distribuzione per visibilità</h3>
      {#if dati.per_visibilita.length === 0}
        <p class="vuoto">Nessun prompt nel vault.</p>
      {:else}
        {@const maxVis = maxConteggio(dati.per_visibilita)}
        <ul class="bar-list">
          {#each dati.per_visibilita as d (d.valore)}
            <li class="bar-row">
              <div class="bar-label">{etichettaVisibilita(d.valore)}</div>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--vis"
                  style:width="{(d.conteggio / maxVis) * 100}%"
                ></div>
              </div>
              <div class="bar-num">{d.conteggio}</div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Prompt più importati -->
    <section class="sezione">
      <h3>Prompt più importati</h3>
      {#if dati.top_importati.length === 0}
        <p class="vuoto">
          Nessun prompt è ancora importato da altri via
          <code>{`{{`}import "..."{`}}`}</code>.
        </p>
      {:else}
        {@const maxImp = Math.max(
          ...dati.top_importati.map((p) => p.conteggio_importatori),
        )}
        <ul class="bar-list">
          {#each dati.top_importati as p (p.id)}
            <li class="bar-row">
              <div class="bar-label" title={p.titolo}>{p.titolo}</div>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--imp"
                  style:width="{(p.conteggio_importatori / maxImp) * 100}%"
                ></div>
              </div>
              <div class="bar-num">{p.conteggio_importatori}</div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Lint health -->
    <section class="sezione">
      <h3>Lint health</h3>
      {#if dati.lint_health.totale_prompt === 0}
        <p class="vuoto">Nessun prompt da analizzare.</p>
      {:else}
        <div class="lint-health-summary">
          <div class="lint-health-percent">
            {dati.lint_health.percentuale_health.toFixed(1)}%
          </div>
          <div class="lint-health-label">
            {dati.lint_health.prompt_senza_issue} su {dati.lint_health
              .totale_prompt} prompt senza issue
          </div>
        </div>
        {#if dati.lint_health.top_categorie.length > 0}
          {@const maxCat = maxConteggio(dati.lint_health.top_categorie)}
          <ul class="bar-list">
            {#each dati.lint_health.top_categorie as c (c.valore)}
              <li class="bar-row">
                <div class="bar-label">{c.valore}</div>
                <div class="bar-track">
                  <div
                    class="bar-fill bar-fill--lint"
                    style:width="{(c.conteggio / maxCat) * 100}%"
                  ></div>
                </div>
                <div class="bar-num">{c.conteggio}</div>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </section>
  {/if}
</Modale>

<style>
  .caricamento {
    text-align: center;
    color: var(--text-muted);
    padding: var(--sp-5);
  }

  .errore-box {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    color: var(--text-default);
  }

  .errore-box strong {
    font-size: var(--fs-md);
    color: var(--text-strong);
  }

  .errore-box span {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    word-break: break-word;
  }

  .sezione + .sezione {
    margin-top: var(--sp-5);
  }

  .sezione h3 {
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    margin: 0 0 var(--sp-2) 0;
    color: var(--text-strong);
  }

  .vuoto {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    margin: 0;
    padding: var(--sp-1) 0;
  }

  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--sp-2);
  }

  .kpi {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-2) var(--sp-3);
  }

  .kpi-num {
    font-size: 28px;
    font-weight: var(--fw-bold);
    line-height: 1;
    color: var(--text-strong);
    font-variant-numeric: tabular-nums;
  }

  .kpi-lbl {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    margin-top: 4px;
  }

  .bar-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .bar-row {
    display: grid;
    grid-template-columns: 220px 1fr 40px;
    gap: var(--sp-2);
    align-items: center;
    font-size: var(--fs-sm);
  }

  .bar-label {
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .bar-track {
    height: 16px;
    background: var(--bg-input);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width 200ms ease;
  }

  .bar-fill--accent {
    background: var(--accent-team);
  }

  .bar-fill--model {
    background: var(--accent-team);
    opacity: 0.85;
  }

  .bar-fill--vis {
    background: var(--text-subtle);
  }

  .bar-fill--imp {
    background: var(--accent-team);
  }

  .bar-fill--lint {
    background: var(--warning, #f59e0b);
  }

  .bar-num {
    text-align: right;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .tag-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .lista-inattivi {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .riga-inattivo {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
  }

  .riga-titolo {
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: var(--sp-2);
  }

  .riga-meta {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .lint-health-summary {
    display: flex;
    align-items: baseline;
    gap: var(--sp-3);
    margin-bottom: var(--sp-3);
  }

  .lint-health-percent {
    font-size: 2rem;
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    font-variant-numeric: tabular-nums;
  }

  .lint-health-label {
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
