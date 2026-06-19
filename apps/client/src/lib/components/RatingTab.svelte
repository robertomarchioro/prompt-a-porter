<script lang="ts">
  /**
   * Tab Valutazioni del DetailPane (issue #390).
   *
   * Mostra l'aggregato dei rating (positivi/neutri/negativi + media) per un
   * prompt, richiamando il comando Tauri `rating_aggregato`. Stato vuoto
   * quando nessun rating è ancora stato registrato.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { Smile, Meh, Frown, Star } from "lucide-svelte";
  import EmptyState from "./EmptyState.svelte";

  interface RatingAggregato {
    media: number | null;
    conteggio: number;
    positivi: number;
    neutri: number;
    negativi: number;
  }

  interface Props {
    promptId: string;
  }

  let { promptId }: Props = $props();

  let aggregato = $state<RatingAggregato | null>(null);
  let caricamento = $state(false);
  let errore = $state<string | null>(null);

  async function carica(): Promise<void> {
    caricamento = true;
    errore = null;
    try {
      aggregato = await invoke<RatingAggregato>("rating_aggregato", {
        promptId,
      });
    } catch (e) {
      console.error("[rating] rating_aggregato", e);
      errore = String(e).replace(/^Error: /, "");
      aggregato = null;
    } finally {
      caricamento = false;
    }
  }

  $effect(() => {
    // Ricarica quando cambia il promptId (navigazione tra prompt).
    void promptId;
    void carica();
  });

  function fmtMedia(media: number | null): string {
    if (media === null) return "—";
    const sign = media > 0 ? "+" : "";
    return `${sign}${media.toFixed(2)}`;
  }

  function percentuale(parte: number, totale: number): string {
    if (totale === 0) return "0%";
    return `${Math.round((parte / totale) * 100)}%`;
  }
</script>

<div class="rating-tab">
  {#if caricamento}
    <div class="spinner" aria-label="Caricamento..."></div>
  {:else if errore}
    <div class="errore">{errore}</div>
  {:else if aggregato && aggregato.conteggio > 0}
    <div class="sommario">
      <div class="media-card">
        <Star size={20} class="star-icon" />
        <span class="media-valore">{fmtMedia(aggregato.media)}</span>
        <span class="media-label">media su {aggregato.conteggio} vot{aggregato.conteggio === 1 ? "o" : "i"}</span>
      </div>
    </div>

    <div class="barre">
      <div class="barra-riga" aria-label="Positivi: {aggregato.positivi} ({percentuale(aggregato.positivi, aggregato.conteggio)})">
        <Smile size={14} class="icona-positivo" />
        <span class="barra-label">Positivi</span>
        <div class="barra-track" role="presentation">
          <div
            class="barra-fill positivo"
            style="width: {percentuale(aggregato.positivi, aggregato.conteggio)}"
          ></div>
        </div>
        <span class="barra-count">{aggregato.positivi}</span>
      </div>
      <div class="barra-riga" aria-label="Neutri: {aggregato.neutri} ({percentuale(aggregato.neutri, aggregato.conteggio)})">
        <Meh size={14} class="icona-neutro" />
        <span class="barra-label">Neutri</span>
        <div class="barra-track" role="presentation">
          <div
            class="barra-fill neutro"
            style="width: {percentuale(aggregato.neutri, aggregato.conteggio)}"
          ></div>
        </div>
        <span class="barra-count">{aggregato.neutri}</span>
      </div>
      <div class="barra-riga" aria-label="Negativi: {aggregato.negativi} ({percentuale(aggregato.negativi, aggregato.conteggio)})">
        <Frown size={14} class="icona-negativo" />
        <span class="barra-label">Negativi</span>
        <div class="barra-track" role="presentation">
          <div
            class="barra-fill negativo"
            style="width: {percentuale(aggregato.negativi, aggregato.conteggio)}"
          ></div>
        </div>
        <span class="barra-count">{aggregato.negativi}</span>
      </div>
    </div>
  {:else}
    <EmptyState
      titolo="Nessuna valutazione"
      hint="Usa la modale Compila per dare un voto al risultato del prompt."
    >
      {#snippet icona()}
        <Star size={32} />
      {/snippet}
    </EmptyState>
  {/if}
</div>

<style>
  .rating-tab {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding: var(--sp-4);
    min-height: 200px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-subtle);
    border-top-color: var(--accent-team);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    margin: var(--sp-6) auto;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .errore {
    color: var(--danger);
    font-size: var(--fs-sm);
    padding: var(--sp-3);
    background: rgba(220, 80, 80, 0.08);
    border-radius: var(--radius-sm);
  }

  .sommario {
    display: flex;
    justify-content: center;
  }

  .media-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-1);
    padding: var(--sp-4) var(--sp-6);
    background: var(--bg-overlay);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .media-valore {
    font-size: 2rem;
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    line-height: 1;
  }

  .media-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .barre {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .barra-riga {
    display: grid;
    grid-template-columns: 16px 70px 1fr 28px;
    align-items: center;
    gap: var(--sp-2);
  }

  .barra-label {
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .barra-track {
    height: 8px;
    background: var(--bg-overlay);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .barra-fill {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width 0.3s ease;
  }

  .barra-fill.positivo {
    background: var(--success);
  }

  .barra-fill.neutro {
    background: var(--text-subtle);
  }

  .barra-fill.negativo {
    background: var(--danger);
  }

  .barra-count {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  :global(.star-icon) {
    color: var(--accent-team);
  }

  :global(.icona-positivo) {
    color: var(--success);
  }

  :global(.icona-neutro) {
    color: var(--text-subtle);
  }

  :global(.icona-negativo) {
    color: var(--danger);
  }
</style>
