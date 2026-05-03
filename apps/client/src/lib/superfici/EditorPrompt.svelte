<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { EditorView } from "@codemirror/view";
  import { basicSetup } from "codemirror";
  import {
    segnapostoHighlight,
    segnapostoTheme,
  } from "$lib/codemirror/placeholder-highlight";
  import { estraiSegnaposti } from "$lib/template";
  import { Button } from "$lib/components";

  interface TagInfoFE {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptPerEditor {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    tags: TagInfoFE[];
  }

  interface Props {
    prompt: PromptPerEditor | null;
    onchiudi: () => void;
    onsalvato: () => void;
  }

  let { prompt, onchiudi, onsalvato }: Props = $props();

  const contenutoIniziale = prompt?.body ?? "";

  let titolo = $state(prompt?.titolo ?? "");
  let descrizione = $state(prompt?.descrizione ?? "");
  let body = $state(contenutoIniziale);
  let visibilita = $state<"private" | "workspace">(
    (prompt?.visibilita as "private" | "workspace") ?? "private",
  );
  let tagNomi = $state<string[]>(prompt?.tags.map((t) => t.nome) ?? []);
  let tagInput = $state("");
  let salvando = $state(false);
  let promptId = $state<string | null>(prompt?.id ?? null);
  let statoSalvataggio = $state<"" | "salvataggio" | "salvato">("");

  let editorEl = $state<HTMLDivElement | null>(null);

  let tuttiITag = $state<string[]>([]);
  let timerAutosave: ReturnType<typeof setTimeout>;

  const segnaposti = $derived(estraiSegnaposti(body));

  const suggerimentiTag = $derived(
    tagInput.trim()
      ? tuttiITag
          .filter(
            (t) =>
              t.toLowerCase().includes(tagInput.toLowerCase()) &&
              !tagNomi.includes(t),
          )
          .slice(0, 5)
      : [],
  );

  $effect(() => {
    invoke<TagInfoFE[]>("libreria_tag_lista")
      .then((tags) => (tuttiITag = tags.map((t) => t.nome)))
      .catch(() => {});
  });

  $effect(() => {
    if (!editorEl) return;
    const view = new EditorView({
      doc: contenutoIniziale,
      extensions: [
        basicSetup,
        segnapostoHighlight,
        segnapostoTheme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            body = update.state.doc.toString();
            pianificaAutosave();
          }
        }),
        EditorView.theme({
          "&": { height: "100%", fontSize: "13px" },
          ".cm-scroller": { overflow: "auto" },
          ".cm-content": {
            fontFamily: "var(--font-mono)",
            minHeight: "200px",
          },
          ".cm-gutters": {
            background: "var(--bg-surface)",
            borderRight: "1px solid var(--border-subtle)",
            color: "var(--text-subtle)",
          },
          "&.cm-focused": { outline: "none" },
        }),
      ],
      parent: editorEl,
    });
    return () => {
      view.destroy();
    };
  });

  function pianificaAutosave() {
    if (!promptId) return;
    clearTimeout(timerAutosave);
    timerAutosave = setTimeout(() => salvaInBackground(), 2000);
  }

  function aggiungiTag(nome?: string) {
    const n = (nome ?? tagInput).trim();
    if (n && !tagNomi.includes(n)) {
      tagNomi = [...tagNomi, n];
      pianificaAutosave();
    }
    tagInput = "";
  }

  function rimuoviTag(indice: number) {
    tagNomi = tagNomi.filter((_, i) => i !== indice);
    pianificaAutosave();
  }

  function gestisciTagKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (suggerimentiTag.length > 0) {
        aggiungiTag(suggerimentiTag[0]);
      } else {
        aggiungiTag();
      }
    } else if (e.key === "Backspace" && !tagInput && tagNomi.length > 0) {
      tagNomi = tagNomi.slice(0, -1);
      pianificaAutosave();
    }
  }

  async function salvaInBackground() {
    if (!titolo.trim() || !body.trim()) return;
    statoSalvataggio = "salvataggio";
    try {
      if (promptId) {
        await invoke("prompt_aggiorna", {
          dati: {
            id: promptId,
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
          },
        });
      } else {
        promptId = await invoke<string>("prompt_crea", {
          dati: {
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
          },
        });
      }
      statoSalvataggio = "salvato";
      setTimeout(() => (statoSalvataggio = ""), 1500);
    } catch {
      statoSalvataggio = "";
    }
  }

  async function salva() {
    if (!titolo.trim() || !body.trim()) return;
    salvando = true;
    clearTimeout(timerAutosave);
    try {
      if (promptId) {
        await invoke("prompt_aggiorna", {
          dati: {
            id: promptId,
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
          },
        });
      } else {
        await invoke<string>("prompt_crea", {
          dati: {
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
          },
        });
      }
      onsalvato();
    } catch (e) {
      console.error("Errore salvataggio:", e);
    } finally {
      salvando = false;
    }
  }

  function renderPreview(testo: string): string {
    const esc = testo
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    return esc.replace(
      /\{\{\s*(\w+)\s*\}\}/g,
      (_, n: string) =>
        `<span class="ph"><span class="br">{{</span>${n}<span class="br">}}</span></span>`,
    );
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
    if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      salva();
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
    aria-label={prompt ? "Modifica prompt" : "Nuovo prompt"}
  >
    <header class="modale-header">
      <h2>{prompt ? "Modifica prompt" : "Nuovo prompt"}</h2>
      <Button variante="ghost" dimensione="sm" onclick={onchiudi}>✕</Button>
    </header>

    <div class="modale-body">
      <!-- ── Colonna editor ── -->
      <div class="col-editor">
        <div class="campo">
          <label for="ed-titolo">Titolo</label>
          <input
            id="ed-titolo"
            bind:value={titolo}
            oninput={pianificaAutosave}
            placeholder="Titolo del prompt"
            autofocus
          />
        </div>
        <div class="campo">
          <label for="ed-desc">Descrizione</label>
          <input
            id="ed-desc"
            bind:value={descrizione}
            oninput={pianificaAutosave}
            placeholder="Breve descrizione (opzionale)"
          />
        </div>
        <div class="campo campo-grow">
          <label>Corpo del prompt</label>
          <div class="editor-wrap" bind:this={editorEl}></div>
        </div>
      </div>

      <!-- ── Colonna metadati ── -->
      <div class="col-meta">
        {#if segnaposti.length > 0}
          <div class="meta-sezione">
            <h3>
              Segnaposti <span class="meta-count">{segnaposti.length}</span>
            </h3>
            <div class="segnaposti-lista">
              {#each segnaposti as s}
                <span class="segnaposto-pill">{`{{${s.nome}}}`}</span>
              {/each}
            </div>
          </div>
        {/if}

        <div class="meta-sezione">
          <h3>Tag</h3>
          <div class="tag-input-wrap">
            {#each tagNomi as nome, i}
              <span class="tag-pill">
                {nome}
                <button
                  class="tag-rm"
                  onclick={() => rimuoviTag(i)}
                  type="button">✕</button
                >
              </span>
            {/each}
            <input
              class="tag-input"
              bind:value={tagInput}
              onkeydown={gestisciTagKeydown}
              placeholder={tagNomi.length ? "" : "Aggiungi tag…"}
            />
            {#if suggerimentiTag.length > 0}
              <div class="suggerimenti">
                {#each suggerimentiTag as sug}
                  <button
                    class="suggerimento"
                    onclick={() => aggiungiTag(sug)}
                    type="button"
                  >
                    {sug}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <div class="meta-sezione">
          <h3>Visibilità</h3>
          <div class="vis-toggle">
            <button
              class="vis-btn"
              class:vis-btn--attivo={visibilita === "private"}
              onclick={() => {
                visibilita = "private";
                pianificaAutosave();
              }}
              type="button">Privato</button
            >
            <button
              class="vis-btn"
              class:vis-btn--attivo={visibilita === "workspace"}
              onclick={() => {
                visibilita = "workspace";
                pianificaAutosave();
              }}
              type="button">Team</button
            >
          </div>
        </div>

        <div class="meta-sezione meta-sezione-grow">
          <h3>Anteprima</h3>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="preview-box">
            {@html renderPreview(body)}
          </div>
        </div>
      </div>
    </div>

    <footer class="modale-footer">
      {#if statoSalvataggio}
        <span class="autosave-status">
          {statoSalvataggio === "salvataggio" ? "Salvataggio…" : "Salvato ✓"}
        </span>
      {/if}
      <Button variante="ghost" onclick={onchiudi}>Annulla</Button>
      <Button
        variante="primary"
        onclick={salva}
        disabled={!titolo.trim() || !body.trim() || salvando}
      >
        {salvando ? "Salvataggio…" : "Salva"}
      </Button>
    </footer>
  </div>
</div>

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
    width: min(960px, 96vw);
    height: min(720px, 92vh);
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

  .modale-header h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .modale-body {
    flex: 1;
    display: grid;
    grid-template-columns: 1.3fr 1fr;
    overflow: hidden;
  }

  /* ── Colonna editor ── */

  .col-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-4) var(--sp-5);
    overflow: hidden;
    border-right: 1px solid var(--border-subtle);
  }

  .col-meta {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding: var(--sp-4) var(--sp-5);
    overflow-y: auto;
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .campo label {
    font-size: 11px;
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-family: var(--font-mono);
  }

  .campo input {
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

  .campo input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .campo input::placeholder {
    color: var(--text-subtle);
  }

  .campo-grow {
    flex: 1;
    min-height: 0;
  }

  .editor-wrap {
    flex: 1;
    min-height: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-input);
  }

  /* ── Metadati (colonna destra) ── */

  .meta-sezione {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .meta-sezione h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .meta-count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-weight: normal;
  }

  .segnaposti-lista {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .segnaposto-pill {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-weight: var(--fw-medium);
  }

  /* ── Tag picker ── */

  .tag-input-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    min-height: 36px;
    align-items: center;
    cursor: text;
    position: relative;
    transition: border-color var(--motion-fast);
  }

  .tag-input-wrap:focus-within {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 6px;
  }

  .tag-rm {
    appearance: none;
    border: none;
    background: none;
    padding: 0;
    margin-left: 2px;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 10px;
    line-height: 1;
  }
  .tag-rm:hover {
    color: var(--danger);
  }

  .tag-input {
    border: none;
    outline: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    flex: 1;
    min-width: 80px;
    padding: 0;
  }
  .tag-input::placeholder {
    color: var(--text-subtle);
  }

  .suggerimenti {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10;
    overflow: hidden;
  }

  .suggerimento {
    appearance: none;
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-default);
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
  }
  .suggerimento:hover {
    background: var(--bg-surface);
  }

  /* ── Visibilità toggle ── */

  .vis-toggle {
    display: flex;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .vis-btn {
    appearance: none;
    flex: 1;
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-muted);
    background: var(--bg-input);
    border: none;
    cursor: pointer;
    transition: all var(--motion-fast);
  }
  .vis-btn + .vis-btn {
    border-left: 1px solid var(--border-default);
  }
  .vis-btn--attivo {
    color: var(--text-strong);
    background: var(--bg-overlay);
    font-weight: var(--fw-medium);
  }

  /* ── Anteprima ── */

  .meta-sezione-grow {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .preview-box {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    line-height: var(--lh-loose);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-y: auto;
    min-height: 120px;
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

  /* ── Footer ── */

  .modale-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-3);
    padding: var(--sp-3) var(--sp-5);
    border-top: 1px solid var(--border-subtle);
  }

  .autosave-status {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin-right: auto;
  }
</style>
