<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Badge, Button, Field, Input, Toast } from "$lib/components";
  import { estraiSegnaposti, compila, contaCompilati } from "$lib/template";

  interface PromptRisultato {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    preferito: boolean;
    uso_count: number;
  }

  const finestra = getCurrentWindow();

  let query = $state("");
  let risultati = $state<PromptRisultato[]>([]);
  let indiceSelezionato = $state(0);
  let modalita = $state<"ricerca" | "compila">("ricerca");
  let promptSelezionato = $state<PromptRisultato | null>(null);
  let valoriSegnaposti = $state<Record<string, string>>({});
  let toastVisibile = $state(false);
  let vaultChiuso = $state(false);
  let inputRicerca: HTMLInputElement | undefined = $state();

  const segnaposti = $derived(
    promptSelezionato ? estraiSegnaposti(promptSelezionato.body) : [],
  );

  const numCompilati = $derived(contaCompilati(segnaposti, valoriSegnaposti));

  const testoCompilato = $derived(
    promptSelezionato ? compila(promptSelezionato.body, valoriSegnaposti) : "",
  );

  let timeoutRicerca: ReturnType<typeof setTimeout>;

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
      risultati = await invoke<PromptRisultato[]>("prompt_cerca", {
        query: q,
      });
      indiceSelezionato = 0;
      vaultChiuso = false;
    } catch {
      risultati = [];
      vaultChiuso = true;
    }
  }

  function seleziona(prompt: PromptRisultato) {
    promptSelezionato = prompt;
    valoriSegnaposti = {};
    modalita = "compila";
  }

  function tornaARicerca() {
    modalita = "ricerca";
    promptSelezionato = null;
    valoriSegnaposti = {};
    inputRicerca?.focus();
  }

  async function compilaECopia() {
    if (!promptSelezionato) return;
    const testo = compila(promptSelezionato.body, valoriSegnaposti);
    await navigator.clipboard.writeText(testo);
    toastVisibile = true;
    setTimeout(() => {
      toastVisibile = false;
      query = "";
      modalita = "ricerca";
      promptSelezionato = null;
      valoriSegnaposti = {};
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
        autofocus
      />
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
                autofocus={i === 0}
              />
            </div>
          {/each}
        </div>
      {/if}

      <div class="palette-header-sezione">
        <span class="eyebrow">Anteprima</span>
      </div>
      <pre class="palette-anteprima">{testoCompilato}</pre>
    </div>

    <div class="palette-footer">
      <span class="palette-hint">
        <kbd>Esc</kbd> indietro · <kbd>Ctrl</kbd>+<kbd>⏎</kbd> compila e copia
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
</style>
