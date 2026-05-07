<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    estraiSegnaposti,
    compila,
    contaCompilati,
  } from "$lib/template";
  import { Button, Toast } from "$lib/components";

  interface TagInfoFE {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptPerCompilatore {
    id: string;
    titolo: string;
    body: string;
    target_model: string;
    tags: TagInfoFE[];
  }

  interface Props {
    prompt: PromptPerCompilatore;
    onchiudi: () => void;
  }

  let { prompt, onchiudi }: Props = $props();

  let valori = $state<Record<string, string>>({});
  let formato = $state<"testo" | "markdown" | "json">("testo");
  let toastVisibile = $state(false);

  // ─── Rating post-uso (Fase 4 Step 2) ───
  let mostraRating = $state(false);
  let ratingDato = $state<-1 | 0 | 1 | null>(null);
  let timerRating: ReturnType<typeof setTimeout> | null = null;

  async function aggiungiRating(valore: -1 | 0 | 1) {
    ratingDato = valore;
    try {
      await invoke<string>("rating_aggiungi", {
        nuovo: {
          prompt_id: prompt.id,
          rating: valore,
          nota: null,
          used_with_model: prompt.target_model || null,
        },
      });
    } catch {
      // Silenzioso: il rating è opzionale, non vogliamo blocco UX.
    }
    // Auto-dismiss più rapido dopo l'azione (1s feedback).
    if (timerRating) clearTimeout(timerRating);
    timerRating = setTimeout(() => {
      mostraRating = false;
      ratingDato = null;
    }, 1000);
  }

  // ─── Espansione import (Fase 3 Step 8) ───
  let bodyEspanso = $state<string | null>(null);
  let loadingEspandi = $state(false);
  let errEspandi = $state("");

  /// True se il body sorgente contiene almeno un `{{import "..."}}`.
  const haImport = $derived(/\{\{\s*import\s+"[^"]+"\s*\}\}/.test(prompt.body));

  /// Body corrente per il flow di compilazione: espanso se l'utente ha
  /// chiesto l'espansione, altrimenti sorgente.
  const bodyCorrente = $derived(bodyEspanso ?? prompt.body);

  async function toggleEspansione() {
    errEspandi = "";
    if (bodyEspanso !== null) {
      // Già espanso → torna alla sorgente.
      bodyEspanso = null;
      return;
    }
    loadingEspandi = true;
    try {
      bodyEspanso = await invoke<string>("prompt_compila", { id: prompt.id });
    } catch (e) {
      errEspandi = String(e);
    } finally {
      loadingEspandi = false;
    }
  }

  const segnaposti = $derived(estraiSegnaposti(bodyCorrente));
  const compilato = $derived(compila(bodyCorrente, valori));
  const numCompilati = $derived(contaCompilati(segnaposti, valori));
  const progresso = $derived(
    segnaposti.length > 0 ? numCompilati / segnaposti.length : 1,
  );
  const tokenStimati = $derived(Math.ceil(compilato.length / 4));

  function testoPerCopia(): string {
    switch (formato) {
      case "markdown":
        return "```\n" + compilato + "\n```";
      case "json":
        return JSON.stringify(
          {
            prompt: compilato,
            ...(prompt.target_model
              ? { model: prompt.target_model }
              : {}),
            parameters: Object.fromEntries(
              segnaposti
                .filter((s) => valori[s.nome]?.trim())
                .map((s) => [s.nome, valori[s.nome].trim()]),
            ),
          },
          null,
          2,
        );
      default:
        return compilato;
    }
  }

  async function copia() {
    await navigator.clipboard.writeText(testoPerCopia());
    invoke("prompt_registra_uso", { id: prompt.id }).catch(() => {});
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 2000);
    // Mostra il toast rating dopo il copy. Auto-dismiss a 5s se l'utente
    // non interagisce (timer abbreviato a 1s se invece clicca).
    ratingDato = null;
    mostraRating = true;
    if (timerRating) clearTimeout(timerRating);
    timerRating = setTimeout(() => {
      mostraRating = false;
    }, 5000);
  }

  function renderCompilazione(
    body: string,
    vals: Record<string, string>,
  ): string {
    const esc = body
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    return esc.replace(/\{\{\s*(\w+)\s*\}\}/g, (_, nome: string) => {
      const valore = vals[nome]?.trim();
      if (valore) {
        const valEsc = valore
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;");
        return `<span class="val-inserito">${valEsc}</span>`;
      }
      return `<span class="ph"><span class="br">{{</span>${nome}<span class="br">}}</span></span>`;
    });
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      copia();
    }
  }}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="scrim"
  onmousedown={(e) => {
    if (e.target === e.currentTarget) onchiudi();
  }}
>
  <div
    class="modale"
    role="dialog"
    aria-modal="true"
    aria-label="Compila prompt"
  >
    <header class="modale-header">
      <div class="header-info">
        <h2>Compila</h2>
        <span class="header-titolo">{prompt.titolo}</span>
      </div>
      <Button variante="ghost" dimensione="sm" onclick={onchiudi}
        >✕</Button
      >
    </header>

    <div class="modale-body">
      <!-- ── Colonna form ── -->
      <div class="col-form">
        {#if segnaposti.length > 0}
          <div class="form-sezione">
            <h3>
              Segnaposti
              <span class="form-count"
                >{numCompilati}/{segnaposti.length}</span
              >
            </h3>
            <div class="progress-wrap">
              <div
                class="progress-bar"
                style:width="{progresso * 100}%"
              ></div>
            </div>
          </div>

          <div class="form-campi">
            {#each segnaposti as s, i}
              <div class="form-campo">
                <label for="comp-{s.nome}">{s.nome}</label>
                <input
                  id="comp-{s.nome}"
                  type="text"
                  bind:value={valori[s.nome]}
                  placeholder="Valore per {s.nome}"
                  autofocus={i === 0}
                />
              </div>
            {/each}
          </div>
        {:else}
          <div class="form-vuoto">
            <p class="vuoto-titolo">Nessun segnaposto</p>
            <p class="vuoto-hint">
              Il prompt è pronto per essere copiato
            </p>
          </div>
        {/if}
      </div>

      <!-- ── Colonna anteprima ── -->
      <div class="col-preview">
        <div class="preview-head">
          <h3>Anteprima</h3>
          {#if haImport}
            <button
              class="import-toggle"
              class:import-toggle--attivo={bodyEspanso !== null}
              onclick={toggleEspansione}
              disabled={loadingEspandi}
              type="button"
            >
              {loadingEspandi
                ? "Espansione…"
                : bodyEspanso !== null
                  ? "Mostra sorgente"
                  : "Espandi import"}
            </button>
          {/if}
          <div class="formato-toggle">
            <button
              class="fmt-btn"
              class:fmt-btn--attivo={formato === "testo"}
              onclick={() => (formato = "testo")}
              type="button">Testo</button
            >
            <button
              class="fmt-btn"
              class:fmt-btn--attivo={formato === "markdown"}
              onclick={() => (formato = "markdown")}
              type="button">Markdown</button
            >
            <button
              class="fmt-btn"
              class:fmt-btn--attivo={formato === "json"}
              onclick={() => (formato = "json")}
              type="button">JSON</button
            >
          </div>
        </div>

        {#if errEspandi}
          <div class="import-errore">{errEspandi}</div>
        {/if}

        {#if formato === "testo"}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="preview-box">
            {@html renderCompilazione(bodyCorrente, valori)}
          </div>
        {:else}
          <pre class="preview-box preview-raw">{testoPerCopia()}</pre>
        {/if}

        <div class="preview-meta">
          <span class="token-count">~{tokenStimati} token</span>
        </div>
      </div>
    </div>

    <footer class="modale-footer">
      <span class="footer-hint">
        <kbd>Ctrl</kbd>+<kbd>⏎</kbd> copia
      </span>
      <Button variante="ghost" onclick={onchiudi}>Annulla</Button>
      <Button variante="primary" onclick={copia}>Copia</Button>
    </footer>
  </div>
</div>

{#if mostraRating}
  <div
    class="rating-toast"
    role="region"
    aria-label="Valuta questo prompt"
  >
    <span class="rating-toast-msg">
      {#if ratingDato !== null}
        Grazie per il feedback!
      {:else}
        Com'è andata con questo prompt?
      {/if}
    </span>
    <div class="rating-bottoni">
      <button
        type="button"
        class="rating-btn rating-btn--neg"
        class:rating-btn--attivo={ratingDato === -1}
        onclick={() => aggiungiRating(-1)}
        disabled={ratingDato !== null}
        aria-label="Negativo"
        title="Non ha funzionato">👎</button
      >
      <button
        type="button"
        class="rating-btn rating-btn--neu"
        class:rating-btn--attivo={ratingDato === 0}
        onclick={() => aggiungiRating(0)}
        disabled={ratingDato !== null}
        aria-label="Neutro"
        title="Così così">😐</button
      >
      <button
        type="button"
        class="rating-btn rating-btn--pos"
        class:rating-btn--attivo={ratingDato === 1}
        onclick={() => aggiungiRating(1)}
        disabled={ratingDato !== null}
        aria-label="Positivo"
        title="Ottimo risultato">👍</button
      >
    </div>
  </div>
{/if}

<Toast variante="success" visibile={toastVisibile}>
  ✓ Copiato negli appunti
</Toast>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
  }

  .modale {
    display: flex;
    flex-direction: column;
    width: min(880px, 96vw);
    height: min(640px, 90vh);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .modale-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-4) var(--sp-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .header-info {
    display: flex;
    align-items: baseline;
    gap: var(--sp-3);
    min-width: 0;
  }

  .header-info h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    flex-shrink: 0;
  }

  .header-titolo {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modale-body {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1.2fr;
    overflow: hidden;
  }

  /* ── Colonna form ── */

  .col-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding: var(--sp-4) var(--sp-5);
    overflow-y: auto;
    border-right: 1px solid var(--border-subtle);
  }

  .form-sezione {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .form-sezione h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .form-count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-weight: normal;
  }

  .progress-wrap {
    height: 4px;
    background: var(--bg-overlay);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent-team);
    border-radius: 2px;
    transition: width var(--motion-normal);
  }

  .form-campi {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .form-campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .form-campo label {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .form-campo input {
    height: 36px;
    padding: 0 var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    outline: none;
    transition: border-color var(--motion-fast);
  }

  .form-campo input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .form-campo input::placeholder {
    color: var(--text-subtle);
  }

  .form-vuoto {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--sp-2);
    text-align: center;
  }

  .vuoto-titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    margin: 0;
  }

  .vuoto-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin: 0;
  }

  /* ── Colonna anteprima ── */

  .col-preview {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-4) var(--sp-5);
    overflow: hidden;
  }

  .preview-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .import-toggle {
    appearance: none;
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background var(--motion-fast),
      border-color var(--motion-fast),
      color var(--motion-fast);
    margin-left: auto;
  }
  .import-toggle:hover:not(:disabled) {
    border-color: var(--accent-team);
    color: var(--accent-team);
  }
  .import-toggle:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .import-toggle--attivo {
    background: var(--accent-team-soft, rgba(80, 120, 200, 0.15));
    border-color: var(--accent-team);
    color: var(--accent-team);
  }

  .import-errore {
    background: rgba(220, 80, 80, 0.12);
    color: #c83;
    border: 1px solid rgba(220, 80, 80, 0.4);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
    margin: 0 0 8px 0;
  }

  .preview-head h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .formato-toggle {
    display: flex;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .fmt-btn {
    appearance: none;
    padding: 4px 10px;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    background: var(--bg-input);
    border: none;
    cursor: pointer;
    transition: all var(--motion-fast);
  }
  .fmt-btn + .fmt-btn {
    border-left: 1px solid var(--border-default);
  }
  .fmt-btn--attivo {
    color: var(--text-strong);
    background: var(--bg-overlay);
    font-weight: var(--fw-medium);
  }

  .preview-box {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-loose);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-4);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-y: auto;
    user-select: text;
    -webkit-user-select: text;
  }

  .preview-raw {
    margin: 0;
    font-size: var(--fs-xs);
  }

  :global(.preview-box .val-inserito) {
    color: var(--success, #16a34a);
    background: color-mix(in oklch, var(--success, #16a34a) 12%, transparent);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    font-weight: var(--fw-medium);
  }

  :global(.preview-box .ph) {
    display: inline;
    font-family: var(--font-mono);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }

  :global(.preview-box .ph .br) {
    opacity: 0.55;
    font-weight: var(--fw-regular);
  }

  .preview-meta {
    display: flex;
    justify-content: flex-end;
  }

  .token-count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  /* ── Footer ── */

  .modale-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-3);
    padding: var(--sp-3) var(--sp-5);
    border-top: 1px solid var(--border-subtle);
  }

  .footer-hint {
    margin-right: auto;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    display: flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .footer-hint kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 3px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  /* ── Toast rating (Fase 4 Step 2) ── */
  .rating-toast {
    position: fixed;
    bottom: var(--sp-4);
    right: var(--sp-4);
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-3);
    padding: var(--sp-2) var(--sp-3);
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    z-index: var(--z-toast, 1000);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    animation: rating-slide-in var(--motion-fast) var(--easing-standard);
  }

  @keyframes rating-slide-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .rating-toast-msg {
    color: var(--text-default);
  }

  .rating-bottoni {
    display: flex;
    gap: var(--sp-1);
  }

  .rating-btn {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 4px 8px;
    font-size: 18px;
    cursor: pointer;
    line-height: 1;
    transition:
      background var(--motion-fast) var(--easing-standard),
      transform var(--motion-fast) var(--easing-standard);
  }

  .rating-btn:not(:disabled):hover {
    background: var(--bg-overlay);
    transform: translateY(-1px);
  }

  .rating-btn--attivo {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
  }

  .rating-btn:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }
</style>
