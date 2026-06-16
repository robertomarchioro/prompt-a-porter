<script lang="ts">
  /**
   * Tab "Import & Var." del DetailPane (V0.8 F5 PR-E).
   *
   * Due sezioni:
   *   1. Import composti — risolve `{{import "path"}}` via cmd backend
   *      `prompt_resolve_import_preview` mostrando titolo + snippet body.
   *      Click apre il prompt importato (CustomEvent pap:apri-prompt).
   *   2. Varianti A/B/C — pill da `varianti_lista`. Click switcha al prompt
   *      variante. Toggle "Confronta tutte" mostra grid 3-colonne con
   *      titolo + body affiancati (ridotto ad A/B/C, decisione #8).
   *
   * Riferimenti:
   * - Decisione designer #8 (riduzione N-way arbitrario → A/B/C parent)
   * - Blueprint F5 PR-E §5
   */
  import { invoke } from "@tauri-apps/api/core";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import { onDestroy, onMount } from "svelte";
  import { GitFork, FileText, Eye } from "lucide-svelte";
  import { estraiImports } from "$lib/util/estrai-imports";

  interface ImportPreview {
    id: string;
    titolo: string;
    body: string;
  }

  interface VariantInfo {
    id: string;
    parent_prompt_id: string;
    variant_label: string;
    titolo: string;
    body: string;
    uso_count: number;
    creato_a: string;
    aggiornato_a: string;
  }

  interface Props {
    promptId: string;
    body: string;
    onConteggio?: (n: number) => void;
  }

  let { promptId, body, onConteggio }: Props = $props();

  let importsRisolti = $state<Map<string, ImportPreview | null>>(new Map());
  let varianti = $state<VariantInfo[]>([]);
  let confrontaTutte = $state(false);

  const importsPath = $derived(estraiImports(body));

  async function risolviImports(paths: string[]): Promise<void> {
    const next = new Map<string, ImportPreview | null>();
    for (const p of paths) {
      try {
        const preview = await invoke<ImportPreview>(
          "prompt_resolve_import_preview",
          { path: p },
        );
        next.set(p, preview);
      } catch {
        next.set(p, null);
      }
    }
    importsRisolti = next;
  }

  async function caricaVarianti(): Promise<void> {
    try {
      varianti = await invoke<VariantInfo[]>("varianti_lista", {
        parentId: promptId,
      });
    } catch {
      varianti = [];
    }
  }

  $effect(() => {
    void importsPath.join("|");
    void risolviImports(importsPath);
  });

  $effect(() => {
    void promptId;
    void caricaVarianti();
  });

  $effect(() => {
    const n = importsPath.length + Math.max(0, varianti.length - 1);
    onConteggio?.(n);
  });

  function gestListaMutata(): void {
    void caricaVarianti();
    void risolviImports(importsPath);
  }

  onMount(() => {
    window.addEventListener("pap:lista-mutata", gestListaMutata);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", gestListaMutata);
  });

  function apriPrompt(id: string): void {
    window.dispatchEvent(new CustomEvent("pap:apri-prompt", { detail: id }));
  }

  function snippetBody(s: string, max = 200): string {
    const trimmed = s.trim();
    if (trimmed.length <= max) return trimmed;
    return trimmed.slice(0, max) + "…";
  }

  // Per "Confronta tutte": prendiamo fino a 3 varianti (A/B/C, decisione #8)
  const tracceConfronto = $derived(varianti.slice(0, 3));
</script>

<div class="iv-tab">
  <!-- Sezione Import composti -->
  <section class="sez">
    <header class="sez-h">
      <span style="display: inline-flex; align-items: center; gap: 6px;">
        <FileText size={12} />
        <span>IMPORT COMPOSTI</span>
        <AiutoLink chiave="prompt-componibili" dimensione={16} />
      </span>
      <span class="count">{importsPath.length}</span>
    </header>
    {#if importsPath.length === 0}
      <p class="vuoto">
        Nessun import nel body. Inserisci <code>{`{{import "path"}}`}</code>
        nell'editor per comporre prompt riusabili.
      </p>
    {:else}
      <ul class="lista" role="list">
        {#each importsPath as path (path)}
          {@const preview = importsRisolti.get(path)}
          <li>
            <button
              class="import-card"
              type="button"
              disabled={!preview}
              onclick={() => preview && apriPrompt(preview.id)}
              title={preview ? "Apri prompt importato" : "Import non risolto"}
            >
              <span class="ico" aria-hidden="true"><GitFork size={12} /></span>
              <span class="info">
                <code class="path">{path}</code>
                {#if preview}
                  <span class="titolo">{preview.titolo}</span>
                  <span class="snippet">{snippetBody(preview.body)}</span>
                {:else if importsRisolti.has(path)}
                  <span class="errore">Import non trovato</span>
                {:else}
                  <span class="muted">Risoluzione…</span>
                {/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- Sezione Varianti A/B/C -->
  <section class="sez">
    <header class="sez-h">
      <span style="display: inline-flex; align-items: center; gap: 6px;">
        <span>VARIANTI A/B/C</span>
        <AiutoLink chiave="varianti" dimensione={16} />
      </span>
      <span class="count">{varianti.length}</span>
      {#if varianti.length > 1}
        <label class="confronta-toggle">
          <input type="checkbox" bind:checked={confrontaTutte} />
          <span><Eye size={11} /> Confronta tutte</span>
        </label>
      {/if}
    </header>

    {#if varianti.length === 0}
      <p class="vuoto">
        Nessuna variante. Crea una variante per testare A/B con lo stesso
        parent prompt.
      </p>
    {:else}
      <div class="varianti-pill">
        {#each varianti as v (v.id)}
          <button
            type="button"
            class="pill"
            class:active={v.id === promptId}
            onclick={() => apriPrompt(v.id)}
            title={v.titolo}
          >
            {v.variant_label}
          </button>
        {/each}
      </div>

      {#if confrontaTutte && tracceConfronto.length >= 2}
        <div
          class="confronto"
          style:grid-template-columns={`repeat(${tracceConfronto.length}, 1fr)`}
        >
          {#each tracceConfronto as v (v.id)}
            <div
              class="col"
              class:active={v.id === promptId}
              role="article"
              aria-label={`Variante ${v.variant_label}`}
            >
              <header class="col-h">
                <span class="lbl">{v.variant_label}</span>
                <span class="titolo-col">{v.titolo}</span>
              </header>
              <pre class="body-col">{v.body}</pre>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </section>
</div>

<style>
  .iv-tab {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-canvas);
    display: flex;
    flex-direction: column;
  }

  .sez {
    padding: var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .sez:last-child {
    border-bottom: 0;
    flex: 1;
    min-height: 0;
  }

  .sez-h {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--text-subtle);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
  }

  .count {
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
    font-size: 11px;
    background: var(--bg-overlay);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    text-transform: none;
    letter-spacing: 0;
  }

  .confronta-toggle {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted);
    cursor: pointer;
    text-transform: none;
    letter-spacing: 0;
    font-weight: var(--fw-regular);
  }

  .confronta-toggle input {
    cursor: pointer;
  }

  .vuoto {
    color: var(--text-subtle);
    font-size: var(--fs-xs);
    margin: 0;
    line-height: 1.5;
  }

  .vuoto code {
    font-family: var(--font-mono);
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    color: var(--info);
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .import-card {
    display: grid;
    grid-template-columns: 18px 1fr;
    gap: var(--sp-2);
    width: 100%;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-default);
    text-align: left;
    cursor: pointer;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    transition: background var(--motion-fast);
    font-family: var(--font-ui);
  }

  .import-card:hover:not(:disabled) {
    background: var(--bg-overlay);
  }

  .import-card:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .ico {
    color: var(--info);
    align-self: start;
    padding-top: 2px;
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .path {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--info);
  }

  .titolo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .snippet {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.4;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    white-space: pre-wrap;
  }

  .errore {
    font-size: 11px;
    color: var(--danger);
  }

  .muted {
    font-size: 11px;
    color: var(--text-subtle);
  }

  .varianti-pill {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .pill {
    border: 1px solid var(--border-subtle);
    background: var(--bg-overlay);
    color: var(--text-muted);
    padding: 2px 10px;
    font-size: 11px;
    border-radius: var(--radius-full);
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .pill:hover {
    color: var(--text-default);
  }

  .pill.active {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
    color: var(--accent-team-strong);
  }

  .confronto {
    display: grid;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
    flex: 1;
    min-height: 0;
  }

  .col {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    overflow: hidden;
  }

  .col.active {
    border-color: var(--accent-team);
  }

  .col-h {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
  }

  .lbl {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--fw-semibold);
    color: var(--accent-team-strong);
    background: var(--accent-team-soft);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }

  .titolo-col {
    font-size: var(--fs-xs);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: var(--fw-medium);
  }

  .body-col {
    flex: 1;
    overflow: auto;
    margin: 0;
    padding: var(--sp-2);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
