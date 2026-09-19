<script lang="ts">
  /**
   * Elenco delle collezioni curate con azioni Importa / Aggiorna.
   *
   * Condiviso fra la modale «Collezioni» (footer sidebar) e la card in
   * Impostazioni → Dati. Nessuna chiamata di rete finché non parte
   * `sfoglia()`: la modale lo chiama subito (aprirla È l'azione esplicita),
   * la card aspetta il click su «Sfoglia». Logica in `collezioni-logic.ts`,
   * backend in `collezioni.rs`.
   */
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    eseguiAggiorna,
    eseguiElenca,
    eseguiImporta,
    formattaDataImport,
    statoCollezione,
    type CollezioneVoce,
    type EsitoImport,
    type ImportReport,
  } from "./collezioni-logic";

  interface Props {
    /** Carica l'elenco appena montato invece di aspettare «Sfoglia». */
    caricaSubito?: boolean;
    /** Notifica il contenitore quando c'è un'operazione in corso. */
    onOccupato?: (occupato: boolean) => void;
  }

  let { caricaSubito = false, onOccupato }: Props = $props();

  const deps = {
    elenca: () => invoke<CollezioneVoce[]>("collezioni_elenca"),
    importa: (slug: string) =>
      invoke<ImportReport>("collezioni_importa", { slug }),
    aggiorna: (slug: string) =>
      invoke<ImportReport>("collezioni_aggiorna", { slug }),
    notificaListaMutata: () =>
      window.dispatchEvent(new CustomEvent("pap:lista-mutata")),
  };

  let collezioni = $state<CollezioneVoce[] | null>(null);
  let caricamento = $state(false);
  let erroreElenco = $state("");
  /** Slug della collezione in importazione/aggiornamento, o null. */
  let inCorso = $state<string | null>(null);
  let esiti = $state<Record<string, EsitoImport>>({});
  let errori = $state<Record<string, string>>({});

  const occupato = $derived(caricamento || inCorso !== null);
  $effect(() => {
    onOccupato?.(occupato);
  });

  export async function sfoglia(): Promise<void> {
    if (occupato) return;
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

  async function azione(voce: CollezioneVoce): Promise<void> {
    if (inCorso !== null) return;
    const slug = voce.slug;
    inCorso = slug;
    const { [slug]: _rimosso, ...altri } = errori;
    errori = altri;
    const esito =
      statoCollezione(voce) === "nuova"
        ? await eseguiImporta(slug, deps)
        : await eseguiAggiorna(slug, deps);
    if (esito.ok) {
      esiti = { ...esiti, [slug]: esito.esito };
      // Il backend registra l'impronta solo senza errori parziali: la voce
      // segue la stessa regola, così con errori il bottone resta attivo e
      // si può riprovare (review PR passo 2).
      if (esito.esito.errori.length === 0) {
        collezioni =
          collezioni?.map((c) =>
            c.slug === slug
              ? {
                  ...c,
                  importata_sha256: c.sha256,
                  importata_a: new Date().toISOString(),
                }
              : c,
          ) ?? null;
      }
    } else {
      errori = { ...errori, [slug]: esito.errore };
    }
    inCorso = null;
  }

  function etichettaAzione(voce: CollezioneVoce): string {
    if (inCorso === voce.slug) return "In corso…";
    switch (statoCollezione(voce)) {
      case "nuova":
        return "Importa";
      case "aggiornabile":
        return "Aggiorna";
      case "aggiornata":
        return "Aggiornata";
    }
  }

  // onMount e non $effect: l'effetto rileggerebbe lo stato che `sfoglia`
  // stesso scrive (regola di progetto sugli $effect, #170).
  onMount(() => {
    if (caricaSubito) {
      void sfoglia();
    }
  });
</script>

{#if caricamento}
  <p class="stato" aria-live="polite">Caricamento dell'elenco…</p>
{/if}

{#if erroreElenco}
  <p class="report-err" role="alert">✗ {erroreElenco}</p>
{/if}

{#if collezioni !== null}
  {#if collezioni.length === 0}
    <p class="stato">Nessuna collezione disponibile al momento.</p>
  {:else}
    <ul class="lista">
      {#each collezioni as c (c.slug)}
        {@const stato = statoCollezione(c)}
        <li class="voce">
          <div class="voce-testo">
            <span class="voce-titolo">{c.titolo}</span>
            <span class="voce-conteggio">{c.prompt} prompt</span>
            {#if stato === "aggiornabile"}
              <span class="badge badge-nuovo">aggiornamento disponibile</span>
            {:else if stato === "aggiornata"}
              <span class="badge">importata il {formattaDataImport(c.importata_a)}</span>
            {/if}
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
            {#if errori[c.slug]}
              <p class="report-err" role="alert">✗ {errori[c.slug]}</p>
            {/if}
          </div>
          <button
            type="button"
            class="btn"
            class:btn-primary={stato !== "aggiornata"}
            onclick={() => azione(c)}
            disabled={occupato || stato === "aggiornata"}
            aria-label={`${etichettaAzione(c)} la collezione ${c.titolo}`}
          >
            {etichettaAzione(c)}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
{/if}

<style>
  .stato {
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
  .badge {
    margin-left: 8px;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .badge-nuovo {
    color: var(--accent-team);
    border-color: var(--accent-team);
  }
  .voce-desc {
    margin: 4px 0 0 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.45;
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
