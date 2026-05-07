<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, EmptyState } from "$lib/components";
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
  }

  interface Props {
    onchiudi: () => void;
  }

  let { onchiudi }: Props = $props();

  let dati = $state<Statistiche | null>(null);
  let caricamento = $state(true);
  let errore = $state<string | null>(null);

  async function carica() {
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
    return valore === "private" ? "Privato" : valore === "workspace" ? "Team" : valore;
  }
</script>

<div class="modale-overlay">
  <div class="modale">
    <header class="modale-head">
      <h2>Insight</h2>
      <Button variante="ghost" onclick={onchiudi}>Chiudi</Button>
    </header>

    <div class="modale-corpo">
      {#if caricamento}
        <div class="caricamento">Caricamento statistiche…</div>
      {:else if errore}
        <EmptyState titolo="Errore" hint={errore} />
      {:else if dati}
        <!-- ── Totali ── -->
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
          </div>
        </section>

        <!-- ── Top usati ── -->
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

        <!-- ── Non usati > 90g ── -->
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

        <!-- ── Distribuzione per tag ── -->
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

        <!-- ── Distribuzione per modello target ── -->
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

        <!-- ── Distribuzione per visibilità ── -->
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

        <!-- ── Prompt più importati (v0.6.0 Step 4) ── -->
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

        <!-- ── Lint health (v0.6.0 Step 4) ── -->
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

        <p class="privacy-nota">
          Tutte le statistiche sono calcolate localmente sul vault. Nessun dato
          esce dal tuo computer.
        </p>
      {/if}
    </div>
  </div>
</div>

<style>
  .modale-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg, rgba(0, 0, 0, 0.4));
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modale {
    width: min(900px, 92vw);
    max-height: 92vh;
    background: var(--bg-canvas);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modale-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-default);
  }

  .modale-head h2 {
    font-size: var(--fs-lg);
    font-weight: 600;
    margin: 0;
  }

  .modale-corpo {
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .caricamento {
    text-align: center;
    color: var(--text-muted);
    padding: 40px;
  }

  .sezione h3 {
    font-size: var(--fs-md);
    font-weight: 600;
    margin: 0 0 12px 0;
    color: var(--text-strong);
  }

  .vuoto {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    margin: 0;
    padding: 8px 0;
  }

  /* KPI grid */
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
  }

  .kpi {
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 12px 16px;
  }

  .kpi-num {
    font-size: 28px;
    font-weight: 700;
    line-height: 1;
    color: var(--text-strong);
  }

  .kpi-lbl {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    margin-top: 4px;
  }

  /* Bar list */
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
    gap: 12px;
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
    background: var(--accent-team-soft, var(--accent-team));
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

  /* Lista inattivi */
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
    padding: 6px 12px;
    background: var(--bg-input);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
  }

  .riga-titolo {
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 12px;
  }

  .riga-meta {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .privacy-nota {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-align: center;
    margin: 0;
    padding-top: 8px;
    border-top: 1px solid var(--border-default);
  }
</style>
