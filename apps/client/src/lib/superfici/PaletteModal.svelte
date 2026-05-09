<script lang="ts">
  /**
   * F8 PR-E — Modale Palette (command palette interna).
   *
   * Sostituisce la window Tauri separata `CommandPalette.svelte` con
   * una modale interna alla shell. Usa la primitive Modale + statoModale.
   *
   * - Input ricerca con focus on mount + debounce 150ms
   * - Cmd: prompt_cerca_ibrida (se ricerca semantica abilitata) o
   *   prompt_cerca semplice
   * - ↑↓ ↵ navigation, ESC chiude (gestito da Modale)
   * - Selezione prompt → apriModale({tipo:"compila", promptId}) (delega
   *   a CompilaModal che sostituisce Palette nello stesso slot statoModale)
   * - Pannello "Filtri avanzati" collassato (decisione #7) con slider
   *   alpha hybrid; toggle stato persistito in localStorage
   *
   * Riferimenti:
   * - Blueprint: docs/roadmap/redesign-v08/blueprint-F8.md §5
   * - Cmd backend: src-tauri/src/prompt.rs (prompt_cerca),
   *   src-tauri/src/ricerca_ibrida.rs (prompt_cerca_ibrida)
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { ChevronDown, ChevronRight, Search } from "lucide-svelte";
  import Modale from "$lib/components/Modale.svelte";
  import { apriModale } from "$lib/stores/modale.svelte";
  import { fmtShortcut } from "$lib/util/shortcut";

  interface PromptRisultato {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    preferito: boolean;
    uso_count: number;
  }

  interface RisultatoIbrido extends PromptRisultato {
    score: number;
    rank_lex: number | null;
    rank_sem: number | null;
  }

  interface PreferenzeRicerca {
    ricerca_semantica_abilitata: boolean;
    ricerca_alpha: number;
  }

  interface Props {
    onChiudi: () => void;
  }

  let { onChiudi }: Props = $props();

  const STORAGE_FILTRI_KEY = "pap.palette.filtri-avanzati-aperti";
  const STORAGE_ALPHA_KEY = "pap.palette.alpha";

  let query = $state("");
  let risultati = $state<PromptRisultato[]>([]);
  let indiceSelezionato = $state(0);
  let prefRicercaSemantica = $state(false);
  let alphaUtente = $state(0.5);
  let filtriAperti = $state(false);
  let inputRicerca: HTMLInputElement | undefined = $state();
  let usaIbrida = $state(false);
  let timeoutRicerca: ReturnType<typeof setTimeout> | undefined;

  function caricaPreferenzeLocali(): void {
    try {
      filtriAperti = localStorage.getItem(STORAGE_FILTRI_KEY) === "1";
      const alpha = localStorage.getItem(STORAGE_ALPHA_KEY);
      if (alpha !== null) {
        const parsed = parseFloat(alpha);
        if (Number.isFinite(parsed) && parsed >= 0 && parsed <= 1) {
          alphaUtente = parsed;
        }
      }
    } catch {
      /* localStorage non disponibile */
    }
  }

  function salvaFiltriAperti(): void {
    try {
      localStorage.setItem(STORAGE_FILTRI_KEY, filtriAperti ? "1" : "0");
    } catch {
      /* ignore */
    }
  }

  function salvaAlpha(): void {
    try {
      localStorage.setItem(STORAGE_ALPHA_KEY, alphaUtente.toFixed(2));
    } catch {
      /* ignore */
    }
  }

  async function caricaPreferenzeBackend(): Promise<void> {
    try {
      const p = await invoke<PreferenzeRicerca>("preferenze_carica");
      prefRicercaSemantica = p.ricerca_semantica_abilitata;
      // Se l'utente non ha ancora toccato lo slider in questa sessione,
      // adotta l'alpha di default da Preferenze.
      if (localStorage.getItem(STORAGE_ALPHA_KEY) === null) {
        alphaUtente = p.ricerca_alpha;
      }
    } catch {
      /* default già settati */
    }
  }

  async function cerca(q: string): Promise<void> {
    try {
      if (prefRicercaSemantica && q.trim().length > 0) {
        const ibridi = await invoke<RisultatoIbrido[]>("prompt_cerca_ibrida", {
          query: q,
          limit: 20,
          alpha: alphaUtente,
        });
        risultati = ibridi;
        usaIbrida = true;
      } else {
        risultati = await invoke<PromptRisultato[]>("prompt_cerca", {
          query: q,
        });
        usaIbrida = false;
      }
      indiceSelezionato = 0;
    } catch {
      risultati = [];
      usaIbrida = false;
    }
  }

  function selezionaIndice(i: number): void {
    if (i >= 0 && i < risultati.length) {
      indiceSelezionato = i;
      apriCompila(risultati[i]);
    }
  }

  function apriCompila(r: PromptRisultato): void {
    apriModale({ tipo: "compila", promptId: r.id });
  }

  function gestisciTastiera(e: KeyboardEvent): void {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (risultati.length > 0) {
        indiceSelezionato = Math.min(
          indiceSelezionato + 1,
          risultati.length - 1,
        );
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      indiceSelezionato = Math.max(indiceSelezionato - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (risultati[indiceSelezionato]) {
        apriCompila(risultati[indiceSelezionato]);
      }
    } else if (e.key === "." && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      filtriAperti = !filtriAperti;
      salvaFiltriAperti();
    }
  }

  onMount(() => {
    caricaPreferenzeLocali();
    void caricaPreferenzeBackend();
    void cerca("");
    inputRicerca?.focus();
  });

  $effect(() => {
    if (timeoutRicerca) clearTimeout(timeoutRicerca);
    const q = query;
    timeoutRicerca = setTimeout(() => void cerca(q), 150);
  });

  $effect(() => {
    if (typeof document === "undefined") return;
    const el = document.querySelector(`[data-indice="${indiceSelezionato}"]`);
    el?.scrollIntoView({ block: "nearest" });
  });
</script>

<Modale
  titolo="Palette"
  sottotitolo="Cerca un prompt o un'azione"
  larghezza="md"
  {onChiudi}
>
  <div class="palette-input-wrap">
    <Search size={14} />
    <input
      type="search"
      bind:this={inputRicerca}
      bind:value={query}
      onkeydown={gestisciTastiera}
      placeholder="Cerca prompt…"
      aria-label="Cerca"
    />
    <button
      type="button"
      class="filtri-toggle"
      class:aperto={filtriAperti}
      onclick={() => {
        filtriAperti = !filtriAperti;
        salvaFiltriAperti();
      }}
      title="Filtri avanzati ({fmtShortcut('mod+.')})"
    >
      {#if filtriAperti}<ChevronDown size={14} />{:else}<ChevronRight
          size={14}
        />{/if}
      <span>Filtri</span>
    </button>
  </div>

  {#if filtriAperti}
    <div class="filtri-pane">
      <div class="campo">
        <span class="campo-label">
          Hybrid alpha (lessicale ↔ semantico)
          <strong class="num">{alphaUtente.toFixed(2)}</strong>
        </span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          bind:value={alphaUtente}
          onchange={() => {
            salvaAlpha();
            void cerca(query);
          }}
          disabled={!prefRicercaSemantica}
          aria-label="Hybrid alpha"
        />
        {#if !prefRicercaSemantica}
          <p class="hint">
            Ricerca semantica disabilitata. Attivala in Impostazioni →
            Avanzate → Ricerca &amp; Embeddings per usare il bilanciamento.
          </p>
        {/if}
      </div>
    </div>
  {/if}

  <ul class="risultati" role="listbox" aria-label="Risultati">
    {#each risultati as r, i (r.id)}
      {@const ibrido = r as RisultatoIbrido}
      <li>
        <button
          type="button"
          role="option"
          aria-selected={indiceSelezionato === i}
          class="riga"
          class:selezionata={indiceSelezionato === i}
          data-indice={i}
          onclick={() => selezionaIndice(i)}
          onmouseenter={() => (indiceSelezionato = i)}
        >
          <div class="testi">
            <div class="titolo">
              {r.titolo}
              {#if r.preferito}<span class="star" title="Preferito">★</span
                >{/if}
            </div>
            {#if r.descrizione}
              <div class="descr">{r.descrizione}</div>
            {/if}
          </div>
          <div class="meta">
            {#if usaIbrida && ibrido.rank_sem !== null}
              <span class="badge-sem" title="Match semantico">sem</span>
            {/if}
            <span class="uso">×{r.uso_count}</span>
          </div>
        </button>
      </li>
    {/each}
    {#if risultati.length === 0 && query.trim() !== ""}
      <li class="vuoto">Nessun risultato per "{query}".</li>
    {:else if risultati.length === 0}
      <li class="vuoto">Nessun prompt nel vault.</li>
    {/if}
  </ul>

  <footer class="hint-bar">
    <span><kbd>↑</kbd> <kbd>↓</kbd> naviga</span>
    <span><kbd>{fmtShortcut("enter")}</kbd> compila</span>
    <span><kbd>{fmtShortcut("mod+.")}</kbd> filtri</span>
    <span><kbd>{fmtShortcut("esc")}</kbd> chiudi</span>
  </footer>
</Modale>

<style>
  .palette-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .palette-input-wrap input {
    flex: 1;
    border: 0;
    background: transparent;
    color: var(--text-default);
    font-size: var(--fs-md);
    outline: none;
  }

  .filtri-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px var(--sp-2);
    border: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    font-size: var(--fs-xs);
    cursor: pointer;
  }

  .filtri-toggle:hover,
  .filtri-toggle.aperto {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .filtri-pane {
    margin-top: var(--sp-2);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .campo-label {
    font-size: var(--fs-sm);
    color: var(--text-default);
    font-weight: var(--fw-medium);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .num {
    font-variant-numeric: tabular-nums;
    color: var(--text-strong);
  }

  .hint {
    margin: 4px 0 0 0;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  input[type="range"] {
    accent-color: var(--accent-team);
    max-width: 320px;
  }

  .risultati {
    list-style: none;
    margin: var(--sp-2) 0 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 320px;
    overflow-y: auto;
  }

  .riga {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    width: 100%;
    padding: 6px var(--sp-2);
    border: 0;
    background: transparent;
    color: var(--text-default);
    border-radius: var(--radius-sm);
    text-align: left;
    cursor: pointer;
    font-size: var(--fs-sm);
  }

  .riga.selezionata {
    background: var(--bg-input);
  }

  .testi {
    flex: 1;
    min-width: 0;
  }

  .titolo {
    color: var(--text-strong);
    font-weight: var(--fw-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .star {
    color: var(--accent-warning, var(--warning, #d2a85f));
  }

  .descr {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 2px;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-xs);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .badge-sem {
    background: var(--accent-team);
    color: var(--accent-team-on);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font-size: 10px;
    font-weight: var(--fw-medium);
    text-transform: uppercase;
  }

  .uso {
    font-variant-numeric: tabular-nums;
  }

  .vuoto {
    padding: var(--sp-3);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-style: italic;
  }

  .hint-bar {
    display: flex;
    gap: var(--sp-3);
    margin-top: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .hint-bar kbd {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-default);
  }
</style>
