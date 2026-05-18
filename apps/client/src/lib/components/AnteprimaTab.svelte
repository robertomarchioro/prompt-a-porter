<script lang="ts">
  /**
   * Tab Anteprima del DetailPane.
   *
   * M5 PR-1: refactor da preview statica (solo highlight segnaposti)
   * a split-view live con form valori inline + compilazione locale.
   *
   * Layout:
   * - Sinistra: form valori segnaposti + (collapsibile) globali
   * - Destra: preview body compilato (sostituzione segnaposti applicata)
   *
   * Limitazione M5 PR-1: gli `{{import "x"}}` sono ancora mostrati come
   * highlight statico (non espansi). L'espansione live richiede backend
   * `prompt_compila_inline` (M5 PR-2).
   *
   * Riferimento blueprint: docs/roadmap/redesign-v08/blueprint-F5.md §1
   */

  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { estraiSegnaposti, compila } from "$lib/template";

  interface PlaceholderGlobale {
    name: string;
    value: string;
    updated_at: string;
  }

  interface Props {
    body: string;
    promptId: string;
  }

  let { body, promptId }: Props = $props();

  // M5 PR-1: stato form valori. Reset su cambio promptId.
  let valori = $state<Record<string, string>>({});
  let valoriGlobali = $state<Record<string, string>>({});
  let globaliCaricati = $state(false);

  // Carica valori globali (UPSERT-driven, condivisi fra prompt).
  async function caricaGlobali(): Promise<void> {
    try {
      const lista = await invoke<PlaceholderGlobale[]>(
        "globale_placeholder_lista",
      );
      const map: Record<string, string> = {};
      for (const g of lista) map[g.name] = g.value;
      valoriGlobali = map;
    } catch {
      valoriGlobali = {};
    } finally {
      globaliCaricati = true;
    }
  }

  onMount(caricaGlobali);

  // Re-carica globali quando l'utente cambia prompt (potrebbe aver
  // modificato globali altrove); reset valori locali.
  $effect(() => {
    void promptId;
    valori = {};
    void caricaGlobali();
  });

  // Estrai segnaposti dal body corrente (reattivo: ad ogni edit
  // dell'EditorTab, l'elenco si aggiorna).
  const segnaposti = $derived(estraiSegnaposti(body));

  // Pre-popola valoriGlobali al primo caricamento per i segnaposti
  // globali presenti nel body. Riapplica se nuovi globali compaiono
  // nel body.
  $effect(() => {
    if (!globaliCaricati) return;
    for (const s of segnaposti) {
      if (s.globale && !(s.nome in valoriGlobali)) {
        valoriGlobali[s.nome] = "";
      }
    }
  });

  // Preview compilata: sostituisce segnaposti con valori utente.
  // Segnaposti non compilati restano come `{{nome}}` (placeholder
  // visivo nel preview, mostrato senza highlight speciale).
  const bodyCompilato = $derived(compila(body, valori, valoriGlobali));

  // Parser highlight per preview (mantiene import non risolti come
  // visualizzazione, segnaposti non compilati come placeholder).
  type Segmento =
    | { tipo: "testo"; testo: string }
    | { tipo: "segnaposto"; nome: string; globale: boolean }
    | { tipo: "import"; path: string };

  const RE_HIGHLIGHT =
    /(\{\{\s*import\s+"([^"]+)"[^}]*\}\})|(\{\{\s*(globale\s+)?(\w+)\s*\}\})/g;

  function parsaSegmenti(testo: string): Segmento[] {
    const acc: Segmento[] = [];
    let last = 0;
    let m: RegExpExecArray | null;
    RE_HIGHLIGHT.lastIndex = 0;
    while ((m = RE_HIGHLIGHT.exec(testo)) !== null) {
      if (m.index > last) {
        acc.push({ tipo: "testo", testo: testo.slice(last, m.index) });
      }
      if (m[2] !== undefined) {
        acc.push({ tipo: "import", path: m[2] });
      } else if (m[5] !== undefined) {
        acc.push({
          tipo: "segnaposto",
          nome: m[5],
          globale: m[4] !== undefined,
        });
      }
      last = m.index + m[0].length;
    }
    if (last < testo.length) {
      acc.push({ tipo: "testo", testo: testo.slice(last) });
    }
    return acc;
  }

  const segmentiPreview = $derived(parsaSegmenti(bodyCompilato));

  const totaleNonGlobali = $derived(segnaposti.filter((s) => !s.globale).length);
  const compilatiNonGlobali = $derived(
    segnaposti.filter((s) => !s.globale && valori[s.nome]?.trim()).length,
  );
  const totaleGlobali = $derived(segnaposti.filter((s) => s.globale).length);

  let mostraGlobali = $state(false);
</script>

{#if body.trim().length === 0}
  <div class="vuoto">
    <p>Body vuoto. Aggiungi contenuto nel tab Editor.</p>
  </div>
{:else}
  <div class="split">
    <!-- Sinistra: form valori -->
    <aside class="form">
      <header class="sez-h">
        <span>SEGNAPOSTI</span>
        <span class="count">{compilatiNonGlobali}/{totaleNonGlobali}</span>
      </header>

      {#if totaleNonGlobali === 0}
        <p class="muted">Nessun segnaposto: il prompt è statico.</p>
      {:else}
        <div class="campi">
          {#each segnaposti.filter((s) => !s.globale) as s (s.nome)}
            <label class="campo">
              <span class="campo-nome">{s.nome}</span>
              <input
                class="campo-input"
                type="text"
                placeholder={s.nome}
                bind:value={valori[s.nome]}
              />
            </label>
          {/each}
        </div>
      {/if}

      {#if totaleGlobali > 0}
        <header class="sez-h sez-h-globali">
          <button
            type="button"
            class="toggle-globali"
            onclick={() => (mostraGlobali = !mostraGlobali)}
            aria-expanded={mostraGlobali}
          >
            {mostraGlobali ? "▾" : "▸"} GLOBALI
          </button>
          <span class="count">{totaleGlobali}</span>
        </header>
        {#if mostraGlobali}
          <div class="campi">
            {#each segnaposti.filter((s) => s.globale) as s (s.nome)}
              <label class="campo">
                <span class="campo-nome campo-nome-globale">{s.nome}</span>
                <input
                  class="campo-input"
                  type="text"
                  placeholder={`globale ${s.nome}`}
                  bind:value={valoriGlobali[s.nome]}
                />
              </label>
            {/each}
          </div>
        {/if}
      {/if}
    </aside>

    <!-- Destra: preview compilato -->
    <section class="preview">
      <header class="sez-h">
        <span>ANTEPRIMA COMPILATA</span>
      </header>
      <pre
        class="anteprima">{#each segmentiPreview as seg, i (i)}{#if seg.tipo === "testo"}{seg.testo}{:else if seg.tipo === "segnaposto"}<span
              class="ph"
              class:ph-globale={seg.globale}
              title={seg.globale
                ? `Segnaposto globale non compilato: ${seg.nome}`
                : `Segnaposto non compilato: ${seg.nome}`}
              >{seg.globale
                ? `{{globale ${seg.nome}}}`
                : `{{${seg.nome}}}`}</span>{:else}<span
              class="imp"
              title="Import composto: risolto in fase di Compila & copia"
              >{`{{import "${seg.path}"}}`}</span>{/if}{/each}</pre>
    </section>
  </div>
{/if}

<style>
  .split {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) 1.5fr;
    gap: var(--sp-2);
    height: 100%;
    overflow: hidden;
  }

  .form {
    overflow-y: auto;
    padding: var(--sp-3);
    background: var(--bg-canvas);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .sez-h {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--fs-xs);
    font-weight: var(--fw-medium);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    color: var(--text-muted);
  }

  .sez-h-globali {
    margin-top: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
  }

  .toggle-globali {
    background: transparent;
    border: 0;
    color: inherit;
    font: inherit;
    text-transform: inherit;
    letter-spacing: inherit;
    padding: 0;
    cursor: pointer;
  }
  .toggle-globali:hover {
    color: var(--text-default);
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
  }

  .muted {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .campi {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .campo-nome {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .campo-nome-globale {
    color: var(--info, var(--accent-team));
  }

  .campo-input {
    padding: 6px 8px;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .campo-input:focus {
    outline: 2px solid var(--accent-team);
    outline-offset: -1px;
  }

  .preview {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preview > .sez-h {
    padding: var(--sp-2) var(--sp-3) 0;
    flex-shrink: 0;
  }

  .anteprima {
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.65;
    background: var(--bg-surface);
    color: var(--text-default);
    padding: var(--sp-3);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    flex: 1;
    overflow-y: auto;
  }

  .ph {
    background: var(--accent-private-soft);
    color: var(--accent-private);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    font-weight: var(--fw-medium);
  }

  .ph-globale {
    background: var(--info-soft, var(--accent-team-soft));
    color: var(--info, var(--accent-team-strong));
  }

  .imp {
    background: var(--info-soft);
    color: var(--info);
    border-radius: var(--radius-sm);
    padding: 1px 4px;
    text-decoration: underline dotted;
    font-weight: var(--fw-medium);
  }

  .vuoto {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .vuoto p {
    margin: 0;
  }
</style>
