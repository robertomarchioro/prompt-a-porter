<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { Star, GitFork, Download, PanelRight } from "lucide-svelte";
  import type { EditorView } from "@codemirror/view";
  import DetailTabs, { type TabId } from "$lib/components/DetailTabs.svelte";
  import EditorTab from "$lib/components/EditorTab.svelte";
  import EditorIndicator from "$lib/components/EditorIndicator.svelte";
  import MarkdownToolbar from "$lib/components/MarkdownToolbar.svelte";
  import RightRail from "$lib/components/RightRail.svelte";

  const META_KEY = "pap.detail.meta-collapsed";
  function caricaMetaCollapsed(): boolean {
    try {
      return localStorage.getItem(META_KEY) === "1";
    } catch {
      return false;
    }
  }
  function salvaMetaCollapsed(v: boolean): void {
    try {
      localStorage.setItem(META_KEY, v ? "1" : "0");
    } catch {
      /* ignore */
    }
  }

  interface TagInfo {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptDettaglio {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    target_model: string;
    folder_id: string | null;
    preferito: boolean;
    uso_count: number;
    creato_a: string;
    aggiornato_a: string;
    ultimo_uso: string;
    tags: TagInfo[];
  }

  interface Cartella {
    id: string;
    nome: string;
  }

  interface Props {
    promptId: string;
    /** Callback opzionale per chiudere il dettaglio (F8 wireuppera). */
    onChiudi?: () => void;
  }

  let { promptId, onChiudi: _onChiudi }: Props = $props();

  let dettaglio = $state<PromptDettaglio | null>(null);
  let titolo = $state("");
  let descrizione = $state("");
  let body = $state("");
  let statoSalvataggio = $state<
    "salvato" | "dirty" | "salvando" | "errore"
  >("salvato");
  let tabAttivo = $state<TabId>("editor");
  let editorView = $state<EditorView | null>(null);
  let righe = $state(1);
  let colonna = $state(1);
  let chars = $state(0);
  let cartelleCache = $state<Cartella[]>([]);
  let erroreCaricamento = $state<string | null>(null);
  let salvatoTs = $state<string | null>(null);
  let metaCollapsed = $state(caricaMetaCollapsed());

  let timerAutosave: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    salvaMetaCollapsed(metaCollapsed);
  });

  async function caricaDettaglio(id: string): Promise<void> {
    erroreCaricamento = null;
    try {
      const d = await invoke<PromptDettaglio>("libreria_dettaglio", { id });
      dettaglio = d;
      titolo = d.titolo;
      descrizione = d.descrizione;
      body = d.body;
      statoSalvataggio = "salvato";
      salvatoTs = d.aggiornato_a; // ultimo save noto: timestamp backend
      // F7: notifica StatusBar del prompt corrente
      window.dispatchEvent(
        new CustomEvent("pap:prompt-corrente", {
          detail: { id: d.id, titolo: d.titolo, visibilita: d.visibilita },
        }),
      );
    } catch (e) {
      console.error("[detail] caricaDettaglio", e);
      dettaglio = null;
      erroreCaricamento = "Prompt non trovato";
    }
  }

  // F7: dispatch save-stato ad ogni cambio per StatusBar
  $effect(() => {
    void statoSalvataggio;
    void salvatoTs;
    window.dispatchEvent(
      new CustomEvent("pap:save-stato", {
        detail: { stato: statoSalvataggio, salvatoA: salvatoTs ?? undefined },
      }),
    );
  });

  $effect(() => {
    void caricaDettaglio(promptId);
  });

  onMount(async () => {
    try {
      cartelleCache = await invoke<Cartella[]>("folder_lista");
    } catch {
      /* ignore */
    }
  });

  onDestroy(() => {
    if (timerAutosave) clearTimeout(timerAutosave);
  });

  function pianificaAutosave(): void {
    if (!dettaglio) return;
    statoSalvataggio = "dirty";
    if (timerAutosave) clearTimeout(timerAutosave);
    timerAutosave = setTimeout(salvaBackground, 2000);
  }

  async function salvaBackground(): Promise<void> {
    if (!titolo.trim() || !body.trim() || !dettaglio) return;
    statoSalvataggio = "salvando";
    try {
      await invoke("prompt_aggiorna", {
        dati: {
          id: promptId,
          titolo: titolo.trim(),
          descrizione: descrizione.trim(),
          body: body.trim(),
          visibilita: dettaglio.visibilita,
          tag_nomi: dettaglio.tags.map((t) => t.nome),
          target_model: dettaglio.target_model || null,
          folder_id: dettaglio.folder_id,
        },
      });
      statoSalvataggio = "salvato";
      salvatoTs = new Date().toISOString();
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (e) {
      console.error("[detail] save", e);
      statoSalvataggio = "errore";
    }
  }

  function onBodyChange(nuovo: string): void {
    body = nuovo;
    pianificaAutosave();
  }

  function onSelectionChange(info: {
    righe: number;
    colonna: number;
    chars: number;
  }): void {
    righe = info.righe;
    colonna = info.colonna;
    chars = info.chars;
  }

  function inserisciVariabile(): void {
    if (!editorView) return;
    const { from, to } = editorView.state.selection.main;
    const inserito = "{{nome}}";
    editorView.dispatch({
      changes: { from, to, insert: inserito },
      selection: { anchor: from + 2, head: from + 2 + 4 },
    });
    editorView.focus();
  }

  function inserisciImport(): void {
    if (!editorView) return;
    const { from, to } = editorView.state.selection.main;
    const inserito = '{{import "path"}}';
    editorView.dispatch({
      changes: { from, to, insert: inserito },
      selection: { anchor: from + 10, head: from + 10 + 4 },
    });
    editorView.focus();
  }

  function aggiungiTag(nome: string): void {
    if (!dettaglio || dettaglio.tags.some((t) => t.nome === nome)) return;
    dettaglio.tags = [
      ...dettaglio.tags,
      { id: `tmp-${nome}`, nome, colore: "" },
    ];
    pianificaAutosave();
  }

  function rimuoviTag(id: string): void {
    if (!dettaglio) return;
    dettaglio.tags = dettaglio.tags.filter((t) => t.id !== id);
    pianificaAutosave();
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    try {
      const t = new Date(iso).getTime();
      const ora = Date.now();
      const sec = Math.max(0, Math.floor((ora - t) / 1000));
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

  const folderNome = $derived(
    dettaglio?.folder_id
      ? cartelleCache.find((c) => c.id === dettaglio?.folder_id)?.nome ??
          dettaglio.folder_id
      : null,
  );
</script>

{#if erroreCaricamento}
  <div class="errore">
    <p>{erroreCaricamento}</p>
  </div>
{:else if dettaglio}
  <article class="detail-pane">
    <header class="detail-header">
      <div class="title-row">
        <input
          class="titolo"
          type="text"
          bind:value={titolo}
          placeholder="Titolo"
          oninput={pianificaAutosave}
        />
        <div class="actions">
          <button
            class="ico"
            type="button"
            class:fav-on={dettaglio.preferito}
            title="Preferito (F8)"
            aria-label="Preferito"
            onclick={() => console.log("F8 toggle fav")}
          >
            <Star
              size={14}
              fill={dettaglio.preferito ? "currentColor" : "transparent"}
            />
          </button>
          <button
            class="ico"
            type="button"
            title="Fork (F8)"
            aria-label="Fork"
            onclick={() => console.log("F8 fork")}
          >
            <GitFork size={14} />
          </button>
          <button
            class="ico"
            type="button"
            title="Esporta MD (F8)"
            aria-label="Esporta MD"
            onclick={() => console.log("F8 export MD")}
          >
            <Download size={14} />
          </button>
          <button
            class="primary"
            type="button"
            title="Compila (F8)"
            onclick={() => console.log("F8 compila")}
          >
            Compila
          </button>
          <button
            class="ico meta-toggle"
            type="button"
            title={metaCollapsed ? "Mostra metadata" : "Nascondi metadata"}
            aria-label="Toggle metadata"
            onclick={() => (metaCollapsed = !metaCollapsed)}
          >
            <PanelRight size={14} />
          </button>
        </div>
      </div>

      <textarea
        class="descrizione"
        bind:value={descrizione}
        placeholder="Descrizione…"
        rows="1"
        oninput={pianificaAutosave}
      ></textarea>

      <div class="meta-row">
        <span class="chip" data-vis={dettaglio.visibilita}>
          <span class="dot" aria-hidden="true"></span>
          {dettaglio.visibilita === "private" ? "Privato" : "Team"}
        </span>
        {#if folderNome}
          <span class="chip">📁 {folderNome}</span>
        {/if}
        {#if dettaglio.target_model}
          <span class="chip">🎯 {dettaglio.target_model}</span>
        {/if}
        <span class="chip muted">Usato {dettaglio.uso_count}×</span>
        <span class="chip muted">
          Aggiornato {tempoRelativo(dettaglio.aggiornato_a)}
        </span>
      </div>

      <DetailTabs {tabAttivo} onSeleziona={(t) => (tabAttivo = t)} />
    </header>

    <PaneGroup
      direction="horizontal"
      autoSaveId="detail-rail-v08"
      class="detail-pane-group"
    >
      <Pane defaultSize={70} minSize={50}>
        <div class="detail-body">
          {#if tabAttivo === "editor"}
            <MarkdownToolbar
              view={editorView}
              onInserisciVariabile={inserisciVariabile}
              onInserisciImport={inserisciImport}
            />
            <EditorTab
              {body}
              onChangeBody={onBodyChange}
              {onSelectionChange}
              {promptId}
              bind:editorView
            />
            <EditorIndicator {statoSalvataggio} {righe} {colonna} {chars} />
          {:else}
            <div class="tab-placeholder">
              <p>Tab "{tabAttivo}" — implementazione in F5</p>
            </div>
          {/if}
        </div>
      </Pane>
      {#if !metaCollapsed}
        <PaneResizer class="resizer" />
        <Pane defaultSize={30} minSize={20} maxSize={40}>
          <RightRail
            {promptId}
            {titolo}
            {body}
            visibilita={dettaglio.visibilita}
            targetModel={dettaglio.target_model}
            folderId={dettaglio.folder_id}
            tags={dettaglio.tags}
            onCambiaVisibilita={(v) => {
              if (dettaglio) dettaglio.visibilita = v;
              pianificaAutosave();
            }}
            onCambiaTarget={(t) => {
              if (dettaglio) dettaglio.target_model = t;
              pianificaAutosave();
            }}
            onCambiaFolder={(f) => {
              if (dettaglio) dettaglio.folder_id = f;
              pianificaAutosave();
            }}
            onAggiungiTag={aggiungiTag}
            onRimuoviTag={rimuoviTag}
            onApriTabImportVar={() => (tabAttivo = "import-var")}
          />
        </Pane>
      {/if}
    </PaneGroup>
  </article>
{:else}
  <div class="caricamento">
    <p>Caricamento…</p>
  </div>
{/if}

<style>
  .detail-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-canvas);
    min-width: 0;
  }

  .detail-header {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-3) 0 var(--sp-3);
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .titolo {
    flex: 1;
    background: transparent;
    border: 0;
    color: var(--text-strong);
    font-size: 20px;
    font-weight: var(--fw-semibold);
    font-family: var(--font-ui);
    padding: 0;
    outline: none;
  }

  .titolo::placeholder {
    color: var(--text-subtle);
  }

  .actions {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .ico {
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
    transition: background var(--motion-fast);
  }

  .ico:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .fav-on {
    color: var(--warning);
  }

  .meta-toggle {
    margin-left: var(--sp-2);
    border-left: 1px solid var(--border-subtle);
    padding-left: var(--sp-2);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  }

  .primary {
    display: inline-flex;
    align-items: center;
    padding: 6px 12px;
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
    transition: background var(--motion-fast);
  }

  .primary:hover {
    background: var(--accent-team-strong);
  }

  .descrizione {
    width: 100%;
    background: transparent;
    border: 0;
    color: var(--text-muted);
    font-size: 13px;
    font-family: var(--font-ui);
    padding: 0;
    outline: none;
    resize: none;
    min-height: 18px;
  }

  .descrizione::placeholder {
    color: var(--text-subtle);
  }

  .meta-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-1);
    padding-bottom: var(--sp-2);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 11px;
    color: var(--text-default);
  }

  .chip[data-vis="private"] .dot {
    background: var(--accent-private);
  }

  .chip[data-vis="workspace"] .dot {
    background: var(--accent-team);
  }

  .chip .dot {
    width: 5px;
    height: 5px;
    border-radius: var(--radius-full);
    background: var(--text-subtle);
  }

  .chip.muted {
    color: var(--text-subtle);
    border-color: transparent;
    background: transparent;
  }

  .detail-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .tab-placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .errore,
  .caricamento {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }
</style>
