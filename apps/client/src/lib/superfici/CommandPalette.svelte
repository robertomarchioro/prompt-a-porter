<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Badge, Toast } from "$lib/components";
  import { estraiSegnaposti, compila, contaCompilati } from "$lib/template";
  import {
    contieneImport,
    espandiImportConToken,
  } from "$lib/util/palette-espansione";

  interface PromptRisultato {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    preferito: boolean;
    uso_count: number;
  }

  /// Risultato di prompt_cerca_ibrida: include score + rank lex/sem.
  /// Dal punto di vista UI lo trattiamo come PromptRisultato + flag
  /// "match semantico" per il badge.
  interface RisultatoIbrido extends PromptRisultato {
    score: number;
    rank_lex: number | null;
    rank_sem: number | null;
  }

  interface Preferenze {
    ricerca_semantica_abilitata: boolean;
    ricerca_alpha: number;
  }

  const finestra = getCurrentWindow();

  let query = $state("");
  let risultati = $state<PromptRisultato[]>([]);
  let indiceSelezionato = $state(0);
  /// Stato preferenze (caricato lazy a window.show), governa quale
  /// command Tauri viene chiamato per la ricerca.
  let prefRicercaSemantica = $state(false);
  let prefAlpha = $state(0.5);
  /// True se l'ultima query ha usato il comando ibrido (per badge UI).
  let usaIbrida = $state(false);
  /// True se almeno un risultato ha rank_sem !== null (qualcuno ha
  /// contributo semantico). Usato per disambiguare badge "ibrida con
  /// match sem" vs "ibrida ma solo lex".
  let qualcheMatchSem = $state(false);
  let modalita = $state<"ricerca" | "compila">("ricerca");
  let promptSelezionato = $state<PromptRisultato | null>(null);
  let valoriSegnaposti = $state<Record<string, string>>({});
  let toastVisibile = $state(false);
  let vaultChiuso = $state(false);
  let inputRicerca: HTMLInputElement | undefined = $state();
  // Issue #299: body con {{import "..."}} espansi via backend (specchio di CompilaModal #293).
  // null = espansione non ancora completata o non necessaria (nessun import nel body).
  let bodyEspanso = $state<string | null>(null);
  let erroreEspansione = $state<string | null>(null);
  // HIGH-1: contatore monotono per scartare risposte fuori-ordine (rapid switching).
  let expansionSeq = 0;
  // HIGH-2: true mentre prompt_compila_inline è in volo; usato per attesa in compilaECopia.
  let espansioneInCorso = $state(false);

  // Issue #299: usa il body espanso (import risolti) come sorgente per
  // segnaposti e output. Se l'espansione non è disponibile (nessun import
  // o errore), si usa il body raw come fallback.
  const bodyPerCompilazione = $derived(
    promptSelezionato ? (bodyEspanso ?? promptSelezionato.body) : "",
  );

  const segnaposti = $derived(
    promptSelezionato ? estraiSegnaposti(bodyPerCompilazione) : [],
  );

  const numCompilati = $derived(contaCompilati(segnaposti, valoriSegnaposti));

  const testoCompilato = $derived(
    promptSelezionato ? compila(bodyPerCompilazione, valoriSegnaposti) : "",
  );

  let timeoutRicerca: ReturnType<typeof setTimeout>;

  $effect(() => {
    void caricaPreferenze();
    // Focus initial sul campo ricerca al mount (sostituisce autofocus
    // attribute, sconsigliato a11y -> richiede focus controllato).
    inputRicerca?.focus();
    // Ricarica preferenze ogni volta che la palette torna visibile —
    // l'utente potrebbe aver cambiato il toggle in Impostazioni.
    // Stessa occasione: ri-focus campo ricerca se siamo in quella view
    // (utente atteso digiti subito senza dover cliccare).
    // La registrazione è asincrona: se il cleanup gira prima che la
    // .then assegni `unlisten`, il listener resterebbe orfano. Il flag
    // `annullato` copre quella finestra disiscrivendolo appena arriva.
    let unlisten: (() => void) | undefined;
    let annullato = false;
    finestra.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        void caricaPreferenze();
        if (modalita === "ricerca") {
          inputRicerca?.focus();
        }
      }
    }).then((u) => {
      if (annullato) {
        u();
      } else {
        unlisten = u;
      }
    });
    return () => {
      annullato = true;
      unlisten?.();
    };
  });

  // Action a11y-friendly per il loop segnaposti: sostituisce
  // l'attributo `autofocus={i === 0}` (vietato da svelte-check a11y).
  function focusIf(node: HTMLInputElement, shouldFocus: boolean) {
    if (shouldFocus) node.focus();
  }

  $effect(() => {
    clearTimeout(timeoutRicerca);
    const q = query;
    timeoutRicerca = setTimeout(() => cerca(q), 150);
  });

  $effect(() => {
    if (modalita === "ricerca") {
      const el = document.querySelector(
        `[data-indice="${indiceSelezionato}"]`,
      );
      el?.scrollIntoView({ block: "nearest" });
    }
  });

  async function cerca(q: string) {
    try {
      if (prefRicercaSemantica && q.trim().length > 0) {
        const ibridi = await invoke<RisultatoIbrido[]>("prompt_cerca_ibrida", {
          query: q,
          limit: 20,
          alpha: prefAlpha,
        });
        risultati = ibridi;
        usaIbrida = true;
        qualcheMatchSem = ibridi.some((r) => r.rank_sem !== null);
      } else {
        risultati = await invoke<PromptRisultato[]>("prompt_cerca", {
          query: q,
        });
        usaIbrida = false;
        qualcheMatchSem = false;
      }
      indiceSelezionato = 0;
      vaultChiuso = false;
    } catch {
      risultati = [];
      vaultChiuso = true;
      usaIbrida = false;
      qualcheMatchSem = false;
    }
  }

  async function caricaPreferenze() {
    try {
      const p = await invoke<Preferenze>("preferenze_carica");
      prefRicercaSemantica = p.ricerca_semantica_abilitata;
      prefAlpha = p.ricerca_alpha;
    } catch {
      /* default già settati */
    }
  }

  /**
   * Issue #299: espande i {{import "..."}} del body via backend, specchio di
   * CompilaModal.svelte. Usa `prompt_compila_inline` con il body raw e
   * l'id del prompt selezionato (per cycle detection corretta).
   *
   * HIGH-1: guard monotona su `expansionSeq` — risposte fuori-ordine vengono
   * scartate silenziosamente (rapid prompt switching).
   * HIGH-2: imposta `espansioneInCorso` true/false per permettere a
   * `compilaECopia` di attendere il completamento prima di copiare.
   * MEDIUM-2: guard usa `{{import "` (con virgoletta) per non colpire
   * segnaposti come `{{importanza}}`.
   *
   * Risultato:
   * - bodyEspanso = stringa espansa  →  segnaposti e output derivano da essa
   * - bodyEspanso = null, erroreEspansione = msg  →  body raw di fallback
   */
  async function espandiImport(rawBody: string, pid: string): Promise<void> {
    // HIGH-2: segnala inizio espansione (anche se non ci sono import, reset è istantaneo)
    espansioneInCorso = contieneImport(rawBody);
    if (!espansioneInCorso) {
      bodyEspanso = null;
      erroreEspansione = null;
      return;
    }
    // HIGH-1: stamp sequenziale prima dell'await
    const seq = ++expansionSeq;
    try {
      const risultato = await espandiImportConToken(
        rawBody,
        pid,
        (body, promptId) =>
          invoke<string>("prompt_compila_inline", { body, promptId }),
        seq,
        () => expansionSeq,
      );
      // HIGH-1: risultato null = risposta fuori-ordine, ignora
      if (risultato === null) return;
      bodyEspanso = risultato.bodyEspanso;
      erroreEspansione = risultato.erroreEspansione;
    } finally {
      // HIGH-2: disattiva solo se questo è ancora il token corrente
      // (se un'altra espansione è già partita, non toccare il suo stato)
      if (seq === expansionSeq) {
        espansioneInCorso = false;
      }
    }
  }

  function seleziona(prompt: PromptRisultato) {
    // Reset eager PRIMA di ogni await: il body espanso del prompt precedente
    // non deve restare visibile mentre l'espansione del nuovo prompt risolve
    // (lezione review #297, HIGH — no stale expansion).
    // HIGH-1: incrementa expansionSeq per invalidare le espansioni in volo.
    expansionSeq++;
    bodyEspanso = null;
    erroreEspansione = null;
    espansioneInCorso = false;
    promptSelezionato = prompt;
    valoriSegnaposti = {};
    modalita = "compila";
    void espandiImport(prompt.body, prompt.id);
  }

  function tornaARicerca() {
    // HIGH-1: cancella qualsiasi espansione in volo (le risposte pending
    // arriveranno con token vecchio e verranno scartate).
    expansionSeq++;
    modalita = "ricerca";
    promptSelezionato = null;
    valoriSegnaposti = {};
    bodyEspanso = null;
    erroreEspansione = null;
    espansioneInCorso = false;
    inputRicerca?.focus();
  }

  async function compilaECopia() {
    if (!promptSelezionato) return;
    // HIGH-2: se l'espansione import è ancora in volo, attendila prima di copiare.
    // Questo assicura che Ctrl+Enter subito dopo la selezione usi il body espanso
    // e non quello raw (che conterrebbe ancora i token {{import "..."}}).
    if (espansioneInCorso) {
      // Attesa attiva: polling sul flag fino a risoluzione.
      await new Promise<void>((resolve) => {
        const check = () => {
          if (!espansioneInCorso) {
            resolve();
          } else {
            requestAnimationFrame(check);
          }
        };
        requestAnimationFrame(check);
      });
    }
    // Issue #299: usa bodyPerCompilazione (body espanso se disponibile,
    // raw come fallback) invece del body raw diretto — allineato a CompilaModal.
    const testo = compila(bodyPerCompilazione, valoriSegnaposti);
    // MEDIUM-3: clipboard può rigettare (contesto insicuro / permesso negato).
    // Non silenziare il fallimento: mostra errore invece di fingere successo.
    try {
      await navigator.clipboard.writeText(testo);
    } catch (e) {
      erroreEspansione = `Copia fallita: ${String(e).replace(/^Error: /, "")}`;
      return;
    }
    toastVisibile = true;
    setTimeout(() => {
      toastVisibile = false;
      query = "";
      modalita = "ricerca";
      promptSelezionato = null;
      valoriSegnaposti = {};
      bodyEspanso = null;
      erroreEspansione = null;
      finestra.hide();
    }, 600);
  }

  function gestisciTastiera(e: KeyboardEvent) {
    if (modalita === "ricerca") {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          if (risultati.length > 0) {
            indiceSelezionato = Math.min(
              indiceSelezionato + 1,
              risultati.length - 1,
            );
          }
          break;
        case "ArrowUp":
          e.preventDefault();
          indiceSelezionato = Math.max(indiceSelezionato - 1, 0);
          break;
        case "Enter":
          e.preventDefault();
          if (risultati[indiceSelezionato]) {
            seleziona(risultati[indiceSelezionato]);
          }
          break;
        case "Escape":
          e.preventDefault();
          query = "";
          finestra.hide();
          break;
      }
    } else if (modalita === "compila") {
      if (e.key === "Escape") {
        e.preventDefault();
        tornaARicerca();
      } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        compilaECopia();
      }
    }
  }
</script>

<svelte:window onkeydown={gestisciTastiera} />

<div class="palette">
  {#if modalita === "ricerca"}
    <div class="palette-input-wrap">
      <svg
        class="palette-icona"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.3-4.3" />
      </svg>
      <input
        class="palette-input"
        bind:value={query}
        bind:this={inputRicerca}
        placeholder="Cerca prompt, tag o azione…"
      />
      {#if usaIbrida && qualcheMatchSem}
        <span
          class="palette-badge-sem"
          title="Ricerca ibrida lessicale + semantica attiva (α = {prefAlpha.toFixed(2)})"
          >🔎 sem</span
        >
      {/if}
    </div>

    <div class="palette-corpo">
      {#if vaultChiuso}
        <div class="palette-vuoto">
          <p class="muted">Sblocca il vault per cercare i prompt</p>
        </div>
      {:else if risultati.length === 0}
        <div class="palette-vuoto">
          <p class="muted">
            {query ? "Nessun risultato" : "Nessun prompt ancora"}
          </p>
          <p class="subtle" style="font-size: var(--fs-xs)">
            {query ? "Prova una ricerca diversa" : "Crea il tuo primo prompt dalla libreria"}
          </p>
        </div>
      {:else}
        <div class="palette-header-sezione">
          <span class="eyebrow"
            >{query ? "Risultati" : "Recenti"}</span
          >
        </div>
        <div class="palette-lista" role="listbox">
          {#each risultati as prompt, i}
            <button
              class="palette-item"
              class:palette-item--attivo={i === indiceSelezionato}
              data-indice={i}
              role="option"
              aria-selected={i === indiceSelezionato}
              onclick={() => seleziona(prompt)}
              onmouseenter={() => (indiceSelezionato = i)}
              type="button"
            >
              <div class="palette-item-info">
                <span class="palette-item-titolo">{prompt.titolo}</span>
                {#if prompt.descrizione}
                  <span class="palette-item-desc">{prompt.descrizione}</span>
                {/if}
              </div>
              <div class="palette-item-meta">
                <Badge
                  variante={prompt.visibilita === "private"
                    ? "warning"
                    : "info"}
                >
                  {prompt.visibilita === "private" ? "privato" : "team"}
                </Badge>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="palette-footer">
      <span class="palette-hint">
        <kbd>Esc</kbd> chiudi · <kbd>↑↓</kbd> naviga · <kbd>⏎</kbd> seleziona
      </span>
    </div>
  {:else if modalita === "compila" && promptSelezionato}
    <button class="palette-header-compila" onclick={tornaARicerca} type="button">
      <span class="palette-back">←</span>
      <span class="palette-header-titolo">{promptSelezionato.titolo}</span>
    </button>

    <div class="palette-corpo">
      {#if segnaposti.length > 0}
        <div class="palette-header-sezione">
          <span class="eyebrow">Segnaposti</span>
          <span class="palette-conteggio">
            {numCompilati} di {segnaposti.length} compilati
          </span>
        </div>

        <div class="palette-form">
          {#each segnaposti as s, i}
            <div class="palette-campo">
              <label class="palette-label" for="seg-{s.nome}"
                >{s.nome}</label
              >
              <input
                id="seg-{s.nome}"
                class="palette-field-input"
                type="text"
                placeholder="{s.nome}"
                bind:value={valoriSegnaposti[s.nome]}
                use:focusIf={i === 0}
              />
            </div>
          {/each}
        </div>
      {/if}

      <div class="palette-header-sezione">
        <span class="eyebrow">Anteprima</span>
      </div>
      {#if erroreEspansione}
        <p class="palette-errore-import" title={erroreEspansione}>
          Import non risolvibile: <code>{erroreEspansione}</code>
        </p>
      {/if}
      <pre class="palette-anteprima">{testoCompilato}</pre>
    </div>

    <div class="palette-footer">
      <span class="palette-hint">
        {#if espansioneInCorso}
          <span class="palette-hint-inCorso">espansione in corso…</span>
        {:else}
          <kbd>Esc</kbd> indietro · <kbd>Ctrl</kbd>+<kbd>⏎</kbd> compila e copia
        {/if}
      </span>
    </div>
  {/if}

  <Toast variante="success" visibile={toastVisibile}>
    ✓ Copiato negli appunti
  </Toast>
</div>

<style>
  .palette {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-raised);
    font-family: var(--font-ui);
    color: var(--text-default);
    overflow: hidden;
  }

  /* ── Input ricerca ── */

  .palette-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .palette-icona {
    color: var(--text-subtle);
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1;
    appearance: none;
    background: transparent;
    border: none;
    font-family: var(--font-ui);
    font-size: var(--fs-base);
    color: var(--text-strong);
    outline: none;
  }

  .palette-input::placeholder {
    color: var(--text-subtle);
  }

  .palette-badge-sem {
    margin-right: var(--sp-3);
    font-size: var(--fs-xs);
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-team-soft, rgba(80, 120, 200, 0.15));
    border: 1px solid var(--accent-team);
    color: var(--accent-team);
    font-family: var(--font-mono);
    white-space: nowrap;
    cursor: help;
  }

  /* ── Corpo ── */

  .palette-corpo {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2) 0;
  }

  .palette-vuoto {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-1);
    height: 100%;
    text-align: center;
    padding: var(--sp-6);
  }

  .palette-header-sezione {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-1) var(--sp-4);
  }

  .palette-conteggio {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-family: var(--font-mono);
  }

  /* ── Lista risultati ── */

  .palette-lista {
    display: flex;
    flex-direction: column;
  }

  .palette-item {
    appearance: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    width: 100%;
    padding: var(--sp-2) var(--sp-4);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    color: var(--text-default);
    transition: background var(--motion-fast);
  }

  .palette-item:hover,
  .palette-item--attivo {
    background: var(--bg-overlay);
  }

  .palette-item--attivo {
    border-left: 2px solid var(--accent-team);
    padding-left: calc(var(--sp-4) - 2px);
  }

  .palette-item-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .palette-item-titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .palette-item-desc {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .palette-item-meta {
    flex-shrink: 0;
  }

  /* ── Header compila ── */

  .palette-header-compila {
    appearance: none;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
    font-family: var(--font-ui);
    color: var(--text-default);
    width: 100%;
    text-align: left;
  }

  .palette-header-compila:hover {
    background: var(--bg-overlay);
  }

  .palette-back {
    color: var(--text-muted);
    font-size: var(--fs-lg);
  }

  .palette-header-titolo {
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  /* ── Form segnaposti ── */

  .palette-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-4);
  }

  .palette-campo {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .palette-label {
    width: 100px;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-align: right;
    flex-shrink: 0;
  }

  .palette-field-input {
    flex: 1;
    height: 28px;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 0 var(--sp-2);
    transition: border-color var(--motion-fast);
  }

  .palette-field-input:focus {
    outline: none;
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .palette-field-input::placeholder {
    color: var(--text-subtle);
    font-style: italic;
  }

  /* ── Anteprima ── */

  /* Issue #299: avviso espansione import fallita (specchio di CompilaModal) */
  .palette-errore-import {
    margin: 0 var(--sp-4) var(--sp-1);
    padding: 4px 8px;
    background: rgba(220, 80, 80, 0.08);
    color: var(--danger);
    font-size: var(--fs-xs);
    border-radius: var(--radius-sm);
  }

  .palette-errore-import code {
    font-family: var(--font-mono);
    background: transparent;
    padding: 0;
  }

  .palette-anteprima {
    margin: 0 var(--sp-4);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
    line-height: var(--lh-relaxed);
    max-height: 160px;
    overflow-y: auto;
    user-select: text;
    -webkit-user-select: text;
  }

  /* ── Footer ── */

  .palette-footer {
    display: flex;
    justify-content: center;
    padding: var(--sp-2) var(--sp-4);
    border-top: 1px solid var(--border-subtle);
  }

  .palette-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    display: flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .palette-hint kbd {
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

  /* HIGH-2: hint visivo durante espansione import in volo */
  .palette-hint-inCorso {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-style: italic;
  }
</style>
