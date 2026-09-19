<script lang="ts">
  /**
   * Card «Collezioni» (Impostazioni → Dati): elenca le collezioni di prompt
   * curate dal progetto e ne importa una nel vault, in modalità `skip`.
   *
   * Nessuna chiamata di rete finché l'utente non preme «Sfoglia»: l'elenco
   * non si carica all'apertura delle impostazioni. Logica e formattazione
   * in `collezioni-logic.ts`; il backend è `collezioni.rs`.
   */
  import { invoke } from "@tauri-apps/api/core";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import {
    eseguiElenca,
    eseguiImporta,
    type CollezioneInfo,
    type EsitoImport,
    type ImportReport,
  } from "./collezioni-logic";

  const deps = {
    elenca: () => invoke<CollezioneInfo[]>("collezioni_elenca"),
    importa: (slug: string) =>
      invoke<ImportReport>("collezioni_importa", { slug }),
    notificaListaMutata: () =>
      window.dispatchEvent(new CustomEvent("pap:lista-mutata")),
  };

  let collezioni = $state<CollezioneInfo[] | null>(null);
  let caricamento = $state(false);
  let erroreElenco = $state("");
  /** Slug della collezione in importazione, o null. */
  let inImport = $state<string | null>(null);
  let esiti = $state<Record<string, EsitoImport>>({});
  let erroriImport = $state<Record<string, string>>({});

  async function sfoglia(): Promise<void> {
    caricamento = true;
    erroreElenco = "";
    const esito = await eseguiElenca(deps);
    if (esito.ok) {
      collezioni = esito.collezioni;
    } else {
      erroreElenco = esito.errore;
    }
    caricamento = false;
  }

  async function importa(slug: string): Promise<void> {
    if (inImport !== null) return;
    inImport = slug;
    const { [slug]: _e, ...altriErrori } = erroriImport;
    erroriImport = altriErrori;
    const esito = await eseguiImporta(slug, deps);
    if (esito.ok) {
      esiti = { ...esiti, [slug]: esito.esito };
    } else {
      erroriImport = { ...erroriImport, [slug]: esito.errore };
    }
    inImport = null;
  }
</script>

<div class="card">
  <header class="card-h">
    <span class="card-title">
      Collezioni di prompt
      <AiutoLink chiave="collezioni" dimensione={16} />
    </span>
    {#if caricamento}
      <span class="card-status">Caricamento…</span>
    {:else if inImport !== null}
      <span class="card-status">Importazione in corso…</span>
    {/if}
  </header>
  <p class="card-desc">
    Raccolte di prompt curate dal progetto, una per tipo di lavoro, scritte
    per mostrare segnaposti, import e varianti. Scaricate da GitHub solo
    quando premi <strong>Sfoglia</strong>; l'importazione non tocca mai i
    prompt che hai già (quelli già presenti vengono saltati). Finiscono nella
    cartella <code>Collezioni</code>.
  </p>

  <button
    type="button"
    class="btn"
    onclick={sfoglia}
    disabled={caricamento || inImport !== null}
  >
    {collezioni === null ? "Sfoglia collezioni…" : "Aggiorna elenco"}
  </button>

  {#if erroreElenco}
    <p class="report-err" role="alert">✗ {erroreElenco}</p>
  {/if}

  {#if collezioni !== null}
    {#if collezioni.length === 0}
      <p class="vuoto">Nessuna collezione disponibile al momento.</p>
    {:else}
      <ul class="lista">
        {#each collezioni as c (c.slug)}
          <li class="voce">
            <div class="voce-testo">
              <span class="voce-titolo">{c.titolo}</span>
              <span class="voce-conteggio">
                {c.prompt} prompt
              </span>
              <p class="voce-desc">{c.descrizione}</p>
              {#if esiti[c.slug]}
                <p class="report-ok">✓ {esiti[c.slug].riepilogo}</p>
                {#if esiti[c.slug].errori.length > 0}
                  <ul class="report-list">
                    {#each esiti[c.slug].errori as e, i (i)}
                      <li>{e}</li>
                    {/each}
                  </ul>
                {/if}
              {/if}
              {#if erroriImport[c.slug]}
                <p class="report-err" role="alert">✗ {erroriImport[c.slug]}</p>
              {/if}
            </div>
            <button
              type="button"
              class="btn btn-primary"
              onclick={() => importa(c.slug)}
              disabled={inImport !== null || caricamento}
              aria-label={`Importa la collezione ${c.titolo}`}
            >
              {inImport === c.slug
                ? "Importazione…"
                : esiti[c.slug]
                  ? "Importa di nuovo"
                  : "Importa"}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  /* Stessi token delle card «dati-*» di ImpostazioniModal: il componente è
     separato per non far crescere ancora quel file, gli stili sono scoped. */
  .card {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    margin-bottom: var(--sp-3);
  }
  .card-h {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--sp-2);
  }
  .card-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }
  .card-status {
    font-size: var(--fs-xs);
    color: var(--accent-team);
  }
  .card-desc {
    margin: 0 0 var(--sp-2) 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }
  .card-desc code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.9em;
  }

  .btn {
    padding: 6px 14px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    cursor: pointer;
    white-space: nowrap;
  }
  .btn:hover:not(:disabled) {
    background: var(--bg-surface);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-primary {
    background: var(--accent-team);
    color: var(--accent-team-on, white);
    border-color: transparent;
  }
  .btn-primary:hover:not(:disabled) {
    background: var(--accent-team-strong, var(--accent-team));
  }

  .vuoto {
    margin: var(--sp-2) 0 0 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }
  .lista {
    list-style: none;
    margin: var(--sp-2) 0 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .voce {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-2);
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
  }
  .voce-testo {
    min-width: 0;
    flex: 1;
  }
  .voce-titolo {
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }
  .voce-conteggio {
    margin-left: 8px;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .voce-desc {
    margin: 4px 0 0 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.45;
  }

  .report-ok {
    margin: 6px 0 0 0;
    font-size: var(--fs-sm);
    color: var(--accent-success, #2c8a2c);
    font-weight: var(--fw-medium);
  }
  .report-err {
    margin: 6px 0 0 0;
    font-size: var(--fs-sm);
    color: var(--danger);
    font-weight: var(--fw-medium);
  }
  .report-list {
    margin: 4px 0 0 0;
    padding-left: var(--sp-3);
    color: var(--text-muted);
    font-size: var(--fs-xs);
    list-style: disc;
  }
</style>
