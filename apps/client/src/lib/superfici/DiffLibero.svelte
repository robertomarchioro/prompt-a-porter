<script lang="ts">
  /**
   * DiffLibero — vista N-way per confrontare prompt arbitrari (V0.8 F5 PR-F).
   *
   * Scope drift della decisione designer #8: la tab "Import & Var." resta
   * limitata ad A/B/C dello stesso parent (F5 PR-E). Il confronto N-way
   * libero è qui in una superficie dedicata, attivata da Shell quando
   * l'utente seleziona ≥2 prompt nella ListPane (Cmd+click).
   *
   * F8 Palette aggiungerà l'azione "Confronta prompt selezionati…" che
   * accederà a questa stessa superficie.
   *
   * Riferimenti:
   * - Decisione designer #8 scope drift
   * - Blueprint F5 PR-F §6
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { X, Columns3 } from "lucide-svelte";

  interface PromptDettaglio {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    target_model: string;
  }

  interface Props {
    idPrompts: string[];
    onChiudi: () => void;
  }

  let { idPrompts, onChiudi }: Props = $props();

  let dettagli = $state<(PromptDettaglio | null)[]>([]);
  let caricamento = $state(true);

  async function carica(): Promise<void> {
    caricamento = true;
    const promesse = idPrompts.map(async (id) => {
      try {
        return await invoke<PromptDettaglio>("libreria_dettaglio", { id });
      } catch (e) {
        console.error("[diff-libero] dettaglio", id, e);
        return null;
      }
    });
    dettagli = await Promise.all(promesse);
    caricamento = false;
  }

  $effect(() => {
    void idPrompts.join("|");
    void carica();
  });

  function onKeydown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      onChiudi();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });

  // Limita a 4 colonne (decisione #8 / blueprint F5 PR-F)
  const visibili = $derived(idPrompts.slice(0, 4));
  const quante = $derived(visibili.length);
</script>

<div
  class="diff-libero"
  role="dialog"
  aria-modal="true"
  aria-label="Confronto libero prompt"
>
  <header class="header">
    <div class="titolo-wrap">
      <Columns3 size={16} />
      <span class="titolo">Confronto libero</span>
      <span class="conteggio">{quante} prompt</span>
    </div>
    <button
      class="chiudi"
      type="button"
      onclick={onChiudi}
      aria-label="Chiudi confronto"
      title="Chiudi (Esc)"
    >
      <X size={16} />
    </button>
  </header>

  {#if caricamento && dettagli.length === 0}
    <div class="vuoto">
      <p>Caricamento prompt…</p>
    </div>
  {:else}
    <div class="grid" style:grid-template-columns={`repeat(${quante}, 1fr)`}>
      {#each visibili as id, i (id)}
        {@const d = dettagli[i]}
        <article class="col" aria-label="Prompt {i + 1} di {quante}">
          {#if d}
            <header class="col-h">
              <span class="titolo-col">{d.titolo}</span>
              {#if d.descrizione}
                <span class="desc-col">{d.descrizione}</span>
              {/if}
              <div class="meta-col">
                <span
                  class="chip"
                  data-vis={d.visibilita}
                  aria-hidden="true"
                ></span>
                <span class="vis-label">
                  {d.visibilita === "private" ? "Privato" : "Team"}
                </span>
                {#if d.target_model}
                  <span class="muted">· {d.target_model}</span>
                {/if}
              </div>
            </header>
            <pre class="body-col">{d.body}</pre>
          {:else}
            <div class="errore">
              <p>Prompt non disponibile</p>
              <code>{id}</code>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .diff-libero {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal);
    display: flex;
    flex-direction: column;
    background: var(--bg-canvas);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    height: 48px;
  }

  .titolo-wrap {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--text-muted);
  }

  .titolo {
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-default);
  }

  .conteggio {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding: 1px 8px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
  }

  .chiudi {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .chiudi:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .vuoto {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .grid {
    flex: 1;
    display: grid;
    gap: 1px;
    overflow: hidden;
    background: var(--border-subtle);
  }

  .col {
    display: flex;
    flex-direction: column;
    background: var(--bg-canvas);
    overflow: hidden;
    min-width: 0;
  }

  .col-h {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .titolo-col {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desc-col {
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta-col {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--text-subtle);
  }

  .chip {
    display: inline-block;
    width: 5px;
    height: 5px;
    border-radius: var(--radius-full);
    background: var(--text-subtle);
  }

  .chip[data-vis="private"] {
    background: var(--accent-private);
  }

  .chip[data-vis="workspace"] {
    background: var(--accent-team);
  }

  .vis-label {
    color: var(--text-default);
  }

  .muted {
    color: var(--text-subtle);
  }

  .body-col {
    flex: 1;
    overflow: auto;
    margin: 0;
    padding: var(--sp-3);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.65;
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .errore {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    color: var(--text-muted);
  }

  .errore code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }
</style>
