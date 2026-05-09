<script lang="ts">
  /**
   * Modale Compila (V0.8 F8 PR-A).
   *
   * Porting ridotto di CompilatorePrompt.svelte legacy. Form segnaposti
   * type-aware (testo/multilinea per ora) + preview live + output 3 formati
   * + rating ±1 con icone Frown/Meh/Smile (decisione #6) + nota collassata.
   *
   * Riferimenti:
   * - Decisione designer #6 (rating ±1 + nota preservato)
   * - Blueprint F8 PR-A §1
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Frown, Meh, Smile, Copy, Check } from "lucide-svelte";
  import { estraiSegnaposti, compila, contaCompilati } from "$lib/template";
  import Modale from "$lib/components/Modale.svelte";
  import { fmtShortcut } from "$lib/util/shortcut";

  interface PromptDettaglio {
    id: string;
    titolo: string;
    body: string;
    target_model: string;
  }

  interface Props {
    promptId: string;
    onChiudi: () => void;
  }

  let { promptId, onChiudi }: Props = $props();

  type FormatoOutput = "testo" | "markdown" | "json";
  type Rating = -1 | 0 | 1;

  let dettaglio = $state<PromptDettaglio | null>(null);
  let valori = $state<Record<string, string>>({});
  let formato = $state<FormatoOutput>("testo");
  let copiato = $state(false);
  let ratingScelto = $state<Rating | null>(null);
  let nota = $state("");
  let ratingInviato = $state(false);
  let erroreCaricamento = $state<string | null>(null);

  async function carica(): Promise<void> {
    try {
      dettaglio = await invoke<PromptDettaglio>("libreria_dettaglio", {
        id: promptId,
      });
      const seg = estraiSegnaposti(dettaglio.body);
      const init: Record<string, string> = {};
      for (const s of seg) {
        init[s.nome] = "";
      }
      valori = init;
    } catch (e) {
      console.error("[compila-modal] caricaDettaglio", e);
      erroreCaricamento = "Prompt non disponibile";
    }
  }

  onMount(carica);

  const segnaposti = $derived(
    dettaglio ? estraiSegnaposti(dettaglio.body) : [],
  );
  const totaleSegnaposti = $derived(segnaposti.length);
  const compilati = $derived(contaCompilati(segnaposti, valori));
  const tuttiCompilati = $derived(
    totaleSegnaposti === 0 || compilati === totaleSegnaposti,
  );

  const output = $derived.by(() => {
    if (!dettaglio) return "";
    const testo = compila(dettaglio.body, valori);
    if (formato === "testo") return testo;
    if (formato === "markdown") {
      return `\`\`\`\n${testo}\n\`\`\``;
    }
    return JSON.stringify(
      {
        prompt_id: promptId,
        titolo: dettaglio.titolo,
        target_model: dettaglio.target_model,
        body: testo,
        valori,
      },
      null,
      2,
    );
  });

  async function copiaOutput(): Promise<void> {
    try {
      await navigator.clipboard.writeText(output);
      copiato = true;
      setTimeout(() => (copiato = false), 1500);
    } catch (e) {
      console.error("[compila-modal] copy", e);
    }
  }

  async function inviaRating(scelto: Rating): Promise<void> {
    if (!dettaglio) return;
    ratingScelto = scelto;
    try {
      await invoke<string>("rating_aggiungi", {
        nuovo: {
          prompt_id: promptId,
          rating: scelto,
          nota: nota.trim() || null,
          used_with_model: dettaglio.target_model || null,
        },
      });
      ratingInviato = true;
      setTimeout(() => onChiudi(), 800);
    } catch (e) {
      console.error("[compila-modal] rating", e);
    }
  }

  function gestKeydown(e: KeyboardEvent): void {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void copiaOutput();
    }
  }
</script>

<svelte:window onkeydown={gestKeydown} />

<Modale
  titolo="Compila prompt"
  sottotitolo={dettaglio?.titolo}
  larghezza="lg"
  {onChiudi}
>
  {#if erroreCaricamento}
    <p class="errore">{erroreCaricamento}</p>
  {:else if !dettaglio}
    <p class="muted">Caricamento…</p>
  {:else}
    <div class="grid">
      <div class="form">
        <header class="sez-h">
          <span>SEGNAPOSTI</span>
          <span class="count">{compilati}/{totaleSegnaposti}</span>
        </header>
        {#if totaleSegnaposti === 0}
          <p class="muted">Nessun segnaposto: questo prompt è statico.</p>
        {:else}
          {#each segnaposti as s (s.nome)}
            <div class="campo">
              <label class="lbl" for={`f-${s.nome}`}>
                <code>{`{{${s.nome}}}`}</code>
              </label>
              <textarea
                id={`f-${s.nome}`}
                class="input"
                rows="2"
                bind:value={valori[s.nome]}
                placeholder={`Valore per ${s.nome}…`}
              ></textarea>
            </div>
          {/each}
        {/if}

        <header class="sez-h">
          <span>FORMATO OUTPUT</span>
        </header>
        <div class="formati">
          {#each ["testo", "markdown", "json"] as f (f)}
            <button
              type="button"
              class="chip-fmt"
              data-attivo={formato === f || undefined}
              onclick={() => (formato = f as FormatoOutput)}
            >
              {f}
            </button>
          {/each}
        </div>
      </div>

      <div class="preview">
        <header class="sez-h">
          <span>ANTEPRIMA</span>
          <button
            class="copia"
            type="button"
            onclick={copiaOutput}
            title="Copia ({fmtShortcut('ctrl+enter')})"
          >
            {#if copiato}
              <Check size={12} />
              <span>Copiato</span>
            {:else}
              <Copy size={12} />
              <span>Copia</span>
            {/if}
          </button>
        </header>
        <pre class="output">{output}</pre>
      </div>
    </div>

    <details class="rating-wrap" open>
      <summary class="rating-summary">
        Valuta il risultato
        {#if ratingInviato}<span class="ok">— grazie!</span>{/if}
      </summary>
      <div class="rating-row">
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === -1 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(-1)}
          aria-label="Negativo"
        >
          <Frown size={16} />
          <span>Negativo</span>
        </button>
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === 0 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(0)}
          aria-label="Neutro"
        >
          <Meh size={16} />
          <span>Neutro</span>
        </button>
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === 1 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(1)}
          aria-label="Positivo"
        >
          <Smile size={16} />
          <span>Positivo</span>
        </button>
      </div>
      <details class="nota-wrap">
        <summary class="nota-summary">Aggiungi nota</summary>
        <textarea
          class="input nota"
          rows="2"
          placeholder="Cosa ha funzionato / non ha funzionato (opzionale)"
          bind:value={nota}
        ></textarea>
      </details>
    </details>
  {/if}

  {#snippet footer()}
    <span class="footer-hint">
      <kbd>{fmtShortcut("ctrl+enter")}</kbd> Compila & copia
    </span>
    <button class="btn-secondary" type="button" onclick={onChiudi}>
      Chiudi
    </button>
    <button
      class="btn-primary"
      type="button"
      disabled={!tuttiCompilati || !dettaglio}
      onclick={copiaOutput}
    >
      Compila & copia
    </button>
  {/snippet}
</Modale>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
    min-height: 320px;
  }

  .form,
  .preview {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    min-width: 0;
  }

  .sez-h {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--text-subtle);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
  }

  .count {
    margin-left: auto;
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
    font-size: 11px;
    text-transform: none;
    letter-spacing: 0;
    background: var(--bg-overlay);
    padding: 1px 6px;
    border-radius: var(--radius-full);
  }

  .copia {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-transform: none;
    letter-spacing: 0;
    font-weight: var(--fw-regular);
  }

  .copia:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .lbl {
    font-size: 11px;
    color: var(--text-muted);
  }

  .lbl code {
    font-family: var(--font-mono);
    background: var(--accent-private-soft);
    color: var(--accent-private);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }

  .input {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    padding: 6px 8px;
    resize: vertical;
  }

  .input:focus {
    outline: none;
    border-color: var(--accent-team);
  }

  .formati {
    display: inline-flex;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    padding: 2px;
    align-self: flex-start;
  }

  .chip-fmt {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 2px 10px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    text-transform: capitalize;
    font-family: var(--font-ui);
  }

  .chip-fmt[data-attivo] {
    background: var(--bg-surface);
    color: var(--text-default);
  }

  .output {
    flex: 1;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: var(--sp-2);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-default);
    margin: 0;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 320px;
  }

  .errore {
    color: var(--danger);
    text-align: center;
  }

  .muted {
    color: var(--text-subtle);
  }

  .rating-wrap {
    margin-top: var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    padding-top: var(--sp-3);
  }

  .rating-summary {
    cursor: pointer;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    margin-bottom: var(--sp-2);
  }

  .ok {
    color: var(--success);
    font-weight: var(--fw-regular);
    font-size: var(--fs-xs);
  }

  .rating-row {
    display: flex;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }

  .rating-btn {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-1);
    padding: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .rating-btn:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .rating-btn[data-attivo] {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
    color: var(--accent-team-strong);
  }

  .rating-btn:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .nota-wrap {
    margin-top: var(--sp-1);
  }

  .nota-summary {
    cursor: pointer;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin-bottom: 4px;
  }

  .nota {
    width: 100%;
  }

  .footer-hint {
    margin-right: auto;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    color: var(--text-default);
  }

  .btn-primary,
  .btn-secondary {
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .btn-primary {
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-team-strong);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
  }

  .btn-secondary:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
