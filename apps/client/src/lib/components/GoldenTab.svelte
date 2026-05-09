<script lang="ts">
  /**
   * Tab Test golden del DetailPane (F5 PR-C).
   *
   * Tabella read-only dei golden test associati al prompt + bottone elimina.
   * Crea/modifica/esegui sono placeholder F8 (richiedono UI provider config).
   *
   * Riferimento blueprint: docs/roadmap/redesign-v08/blueprint-F5.md §3
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import { Play, Pencil, Trash2, Plus } from "lucide-svelte";

  interface Golden {
    id: string;
    prompt_id: string;
    etichetta: string;
    input_vars: string;
    expected_output: string;
    similarity_fn: string;
    soglia_tolleranza: number;
    creato_a: string;
    aggiornato_a: string;
  }

  interface Props {
    promptId: string;
    onConteggio?: (n: number) => void;
  }

  let { promptId, onConteggio }: Props = $props();

  let goldens = $state<Golden[]>([]);
  let caricamento = $state(false);

  async function carica(): Promise<void> {
    caricamento = true;
    try {
      goldens = await invoke<Golden[]>("golden_lista", {
        promptId,
      });
      onConteggio?.(goldens.length);
    } catch (e) {
      console.error("[golden] golden_lista", e);
      goldens = [];
      onConteggio?.(0);
    } finally {
      caricamento = false;
    }
  }

  $effect(() => {
    void promptId;
    void carica();
  });

  function gestListaMutata(): void {
    void carica();
  }

  onMount(() => {
    window.addEventListener("pap:lista-mutata", gestListaMutata);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", gestListaMutata);
  });

  async function elimina(id: string, etichetta: string): Promise<void> {
    if (!confirm(`Eliminare il golden test "${etichetta}"?`)) return;
    try {
      await invoke("golden_elimina", { id });
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[golden] golden_elimina", e);
    }
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    try {
      const sec = Math.max(
        0,
        Math.floor((Date.now() - new Date(iso).getTime()) / 1000),
      );
      if (sec < 60) return "ora";
      const min = Math.floor(sec / 60);
      if (min < 60) return `${min}m fa`;
      const h = Math.floor(min / 60);
      if (h < 24) return `${h}h fa`;
      const g = Math.floor(h / 24);
      return `${g}g fa`;
    } catch {
      return "";
    }
  }
</script>

<div class="golden-tab">
  <header class="header">
    <span class="titolo">Test golden</span>
    <span class="conteggio">{goldens.length}</span>
    <button
      class="primary"
      type="button"
      onclick={() => console.log("F8 modale crea golden")}
      title="Crea golden test (F8 modale)"
    >
      <Plus size={14} />
      <span>Golden</span>
    </button>
  </header>

  {#if caricamento && goldens.length === 0}
    <div class="vuoto">
      <p>Caricamento…</p>
    </div>
  {:else if goldens.length === 0}
    <div class="vuoto">
      <p class="muted">Nessun golden test definito per questo prompt.</p>
      <p class="sub">
        I golden ti permettono di confrontare l'output con un valore atteso e
        tracciare drift tra modelli o versioni.
      </p>
    </div>
  {:else}
    <ul class="lista" role="list">
      {#each goldens as g (g.id)}
        <li class="riga">
          <div class="riga-meta">
            <span class="etichetta">{g.etichetta}</span>
            <div class="badge-row">
              <span class="badge">{g.similarity_fn}</span>
              <span class="badge muted"
                >soglia {(g.soglia_tolleranza * 100).toFixed(0)}%</span
              >
              <span class="badge muted"
                >aggiornato {tempoRelativo(g.aggiornato_a)}</span
              >
            </div>
          </div>
          <div class="riga-azioni">
            <button
              class="ico"
              type="button"
              title="Esegui (F8 — richiede provider config)"
              aria-label="Esegui golden"
              onclick={() => console.log("F8 esegui golden", g.id)}
            >
              <Play size={14} />
            </button>
            <button
              class="ico"
              type="button"
              title="Modifica (F8 modale)"
              aria-label="Modifica golden"
              onclick={() => console.log("F8 modifica golden", g.id)}
            >
              <Pencil size={14} />
            </button>
            <button
              class="ico ico-danger"
              type="button"
              title="Elimina golden"
              aria-label="Elimina golden"
              onclick={() => elimina(g.id, g.etichetta)}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .golden-tab {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2);
    background: var(--bg-canvas);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-1);
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

  .primary {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 6px var(--sp-2);
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
  }

  .primary:hover {
    background: var(--accent-team-strong);
  }

  .vuoto {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    color: var(--text-muted);
    padding: var(--sp-4);
    gap: var(--sp-1);
  }

  .vuoto p {
    margin: 0;
    font-size: var(--fs-sm);
  }

  .muted {
    color: var(--text-muted);
  }

  .sub {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    max-width: 360px;
  }

  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .riga {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
  }

  .riga:hover {
    background: var(--bg-overlay);
  }

  .riga-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .etichetta {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge-row {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 4px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 6px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-default);
  }

  .badge.muted {
    color: var(--text-subtle);
    border-color: transparent;
    background: transparent;
  }

  .riga-azioni {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .ico:hover {
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  .ico-danger:hover {
    color: var(--danger);
  }
</style>
