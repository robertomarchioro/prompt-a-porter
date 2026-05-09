<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { ChevronsRight, Search, Plus, X } from "lucide-svelte";
  import PromptCard from "./PromptCard.svelte";
  import {
    caricaStato,
    salvaStato,
    type Densita,
    type Ordine,
    type StatoLista,
  } from "$lib/stores/densita";

  interface TagInfo {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptCardData {
    id: string;
    titolo: string;
    descrizione: string;
    visibilita: string;
    preferito: boolean;
    uso_count: number;
    aggiornato_a: string;
    tags: TagInfo[];
  }

  interface Cartella {
    id: string;
    nome: string;
  }

  interface Props {
    vistaCorrente: string;
    folderSelezionato: string | null;
    tagSelezionato: string | null;
    modelTargetSelezionato: string;
    onRimuoviFolder: () => void;
    onRimuoviTag: () => void;
    onRimuoviModelTarget: () => void;
    promptSelezionato: string | null;
    onSelezionaPrompt: (id: string) => void;
    onApriCollapse?: () => void;
  }

  let {
    vistaCorrente,
    folderSelezionato,
    tagSelezionato,
    modelTargetSelezionato,
    onRimuoviFolder,
    onRimuoviTag,
    onRimuoviModelTarget,
    promptSelezionato,
    onSelezionaPrompt,
    onApriCollapse,
  }: Props = $props();

  let stato = $state<StatoLista>(caricaStato());
  let prompts = $state<PromptCardData[]>([]);
  let cerca = $state("");
  let cercaDebounced = $state("");
  let cartelle = $state<Cartella[]>([]);

  // Drag state — module-level fallback per dataTransfer null su browser strict.
  let draggedId: string | null = null;
  let dropTargetIndex = $state<number | null>(null);
  let dropPosition = $state<"before" | "after" | null>(null);

  // Persisti stato lista (debounced 200ms)
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void stato.densita;
    void stato.ordine;
    void stato.righePreview;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => salvaStato(stato), 200);
  });

  // Debounce search 300ms
  let cercaTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void cerca;
    if (cercaTimer) clearTimeout(cercaTimer);
    cercaTimer = setTimeout(() => {
      cercaDebounced = cerca;
    }, 300);
  });

  $effect(() => {
    // Trigger reload su qualunque filtro o ordine cambi
    void vistaCorrente;
    void folderSelezionato;
    void tagSelezionato;
    void modelTargetSelezionato;
    void cercaDebounced;
    void stato.ordine;
    void caricaLista();
  });

  async function caricaLista(): Promise<void> {
    try {
      prompts = await invoke<PromptCardData[]>("libreria_lista", {
        filtro: {
          vista: vistaCorrente,
          tag_id: tagSelezionato,
          cerca: cercaDebounced || null,
          ordine: stato.ordine,
          target_model: modelTargetSelezionato || null,
          folder_id: folderSelezionato,
        },
      });
    } catch (e) {
      console.error("[list-pane] caricamento", e);
      prompts = [];
    }
  }

  // Cartelle in cache per label chip "Cartella: <nome>"
  onMount(async () => {
    try {
      cartelle = await invoke<Cartella[]>("folder_lista");
    } catch {
      /* ignore */
    }
    window.addEventListener("pap:lista-mutata", caricaLista);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", caricaLista);
  });

  const titoloVista = $derived.by(() => {
    switch (vistaCorrente) {
      case "preferiti":
        return "Preferiti";
      case "privati":
        return "Privati";
      case "team":
        return "Team";
      case "tutti":
        return "Tutti i prompt";
      default:
        return "Recenti";
    }
  });

  const folderLabel = $derived(
    folderSelezionato
      ? cartelle.find((c) => c.id === folderSelezionato)?.nome ??
          folderSelezionato
      : null,
  );

  const opzioniDensita: { id: Densita; label: string; abilitata: boolean }[] = [
    { id: "compatta", label: "Compatta", abilitata: true },
    { id: "comoda", label: "Comoda", abilitata: true },
    // Anteprima: F3 PR-B.
    { id: "anteprima", label: "Anteprima", abilitata: false },
  ];

  const opzioniOrdine: { id: Ordine; label: string }[] = [
    { id: "recente", label: "Recenti" },
    { id: "popolare", label: "Popolari" },
    { id: "qualita", label: "Migliori" },
    { id: "alfabetico", label: "A-Z" },
  ];

  function selezionaDensita(d: Densita): void {
    if (d === "anteprima") {
      console.warn("[list-pane] densità Anteprima rinviata a F3 PR-B");
      return;
    }
    stato.densita = d;
  }

  // ─── Drag & drop riordino ─────────────────

  function gestDragStart(e: DragEvent, id: string): void {
    draggedId = id;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/x-pap-prompt", id);
    }
    document.addEventListener("keydown", gestEscDuringDrag);
  }

  function gestDragEnd(): void {
    draggedId = null;
    dropTargetIndex = null;
    dropPosition = null;
    document.removeEventListener("keydown", gestEscDuringDrag);
  }

  function gestEscDuringDrag(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      gestDragEnd();
    }
  }

  function gestDragOverCard(e: DragEvent, idx: number): void {
    if (!draggedId) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const aMeta = e.clientY < rect.top + rect.height / 2;
    dropTargetIndex = idx;
    dropPosition = aMeta ? "before" : "after";
  }

  async function gestDropCard(e: DragEvent, idx: number): Promise<void> {
    e.preventDefault();
    if (!draggedId) return;
    const promptId = draggedId;
    const pos = dropPosition;
    gestDragEnd();
    if (!pos) return;

    const fromIdx = prompts.findIndex((p) => p.id === promptId);
    let targetIdx = pos === "before" ? idx : idx + 1;
    if (targetIdx > fromIdx) targetIdx -= 1; // adjust per rimozione self
    if (targetIdx === fromIdx) return; // no-op

    try {
      await invoke("prompt_riordina", {
        dati: { prompt_id: promptId, new_sort: targetIdx },
      });
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (err) {
      console.error("[list-pane] prompt_riordina", err);
    }
  }
</script>

<section class="list-pane" data-densita={stato.densita}>
  <header class="header">
    <div class="title-row">
      <span class="titolo-vista">{titoloVista}</span>
      <span class="count">{prompts.length}</span>
      <button
        class="collapse-btn"
        type="button"
        aria-label="Comprimi lista"
        title="Comprimi"
        onclick={onApriCollapse}
      >
        <ChevronsRight size={14} />
      </button>
    </div>

    <div class="search-row">
      <div class="search-wrap">
        <Search size={14} class="search-ico" />
        <input
          class="search-input"
          type="search"
          placeholder="Cerca prompt..."
          bind:value={cerca}
        />
      </div>
      <button
        class="btn-nuovo"
        type="button"
        onclick={() => console.log("F8 modale crea prompt")}
        title="F8 modale crea prompt"
      >
        <Plus size={14} />
        <span>Nuovo</span>
      </button>
    </div>

    <div class="toolbar-row">
      <div class="chip-densita">
        {#each opzioniDensita as opt (opt.id)}
          <button
            class="chip"
            class:chip-attivo={stato.densita === opt.id}
            disabled={!opt.abilitata}
            type="button"
            onclick={() => selezionaDensita(opt.id)}
          >
            {opt.label}
          </button>
        {/each}
      </div>

      <div class="filtri-attivi">
        {#if folderLabel}
          <span class="filtro-chip">
            Cartella: {folderLabel}
            <button
              type="button"
              aria-label="Rimuovi filtro cartella"
              onclick={onRimuoviFolder}
            >
              <X size={12} />
            </button>
          </span>
        {/if}
        {#if tagSelezionato}
          <span class="filtro-chip">
            Tag
            <button
              type="button"
              aria-label="Rimuovi filtro tag"
              onclick={onRimuoviTag}
            >
              <X size={12} />
            </button>
          </span>
        {/if}
        {#if modelTargetSelezionato}
          <span class="filtro-chip">
            Modello: {modelTargetSelezionato}
            <button
              type="button"
              aria-label="Rimuovi filtro modello"
              onclick={onRimuoviModelTarget}
            >
              <X size={12} />
            </button>
          </span>
        {/if}
      </div>

      <select
        class="sort"
        bind:value={stato.ordine}
        aria-label="Ordina per"
      >
        {#each opzioniOrdine as opt (opt.id)}
          <option value={opt.id}>{opt.label}</option>
        {/each}
      </select>
    </div>
  </header>

  <div class="body">
    {#if prompts.length === 0}
      <div class="empty">
        <p class="empty-msg">Nessun prompt trovato</p>
        <p class="empty-hint">
          Prova a rimuovere alcuni filtri o cerca con parole diverse.
        </p>
      </div>
    {:else}
      {#each prompts as p, idx (p.id)}
        <div
          class="card-wrap"
          class:drop-before={dropTargetIndex === idx &&
            dropPosition === "before"}
          class:drop-after={dropTargetIndex === idx &&
            dropPosition === "after"}
          class:dragging={draggedId === p.id}
          draggable="true"
          ondragstart={(e) => gestDragStart(e, p.id)}
          ondragover={(e) => gestDragOverCard(e, idx)}
          ondrop={(e) => gestDropCard(e, idx)}
          ondragend={gestDragEnd}
          role="presentation"
        >
          <PromptCard
            id={p.id}
            titolo={p.titolo}
            descrizione={p.descrizione}
            visibilita={p.visibilita}
            preferito={p.preferito}
            usoCount={p.uso_count}
            aggiornatoA={p.aggiornato_a}
            tags={p.tags}
            attivo={promptSelezionato === p.id}
            densita={stato.densita}
            righePreview={stato.righePreview}
            onclick={() => onSelezionaPrompt(p.id)}
          />
        </div>
      {/each}
    {/if}
  </div>
</section>

<style>
  .list-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-canvas);
    border-left: 1px solid var(--border-subtle);
    border-right: 1px solid var(--border-subtle);
    min-width: 0;
  }

  .header {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--bg-canvas);
    padding: var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    border-bottom: 1px solid var(--border-subtle);
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .titolo-vista {
    flex: 1;
    font-size: var(--fs-base);
    font-weight: var(--fw-semibold);
    color: var(--text-default);
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    padding: 1px var(--sp-2);
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
  }

  .collapse-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .collapse-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .search-row {
    display: flex;
    gap: var(--sp-2);
    align-items: center;
  }

  .search-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  :global(.search-ico) {
    position: absolute;
    left: var(--sp-2);
    color: var(--text-subtle);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 6px 8px 6px 28px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-team);
  }

  .btn-nuovo {
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

  .btn-nuovo:hover {
    background: var(--accent-team-strong);
  }

  .toolbar-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }

  .chip-densita {
    display: inline-flex;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .chip {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 2px var(--sp-2);
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
  }

  .chip:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .chip-attivo {
    background: var(--bg-surface);
    color: var(--text-default);
  }

  .filtri-attivi {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    flex-wrap: wrap;
  }

  .filtro-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 4px 2px 8px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 11px;
    color: var(--text-default);
  }

  .filtro-chip button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border: 0;
    background: transparent;
    color: var(--text-subtle);
    cursor: pointer;
    border-radius: var(--radius-full);
  }

  .filtro-chip button:hover {
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  .sort {
    margin-left: auto;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-xs);
    padding: 4px 6px;
    font-family: var(--font-ui);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-1) 0;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--text-muted);
    padding: var(--sp-4);
    gap: var(--sp-2);
  }

  .empty-msg {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    margin: 0;
  }

  .empty-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin: 0;
  }

  .card-wrap {
    position: relative;
    transition: opacity var(--motion-fast);
  }

  .card-wrap.dragging {
    opacity: 0.4;
  }

  /* Drop indicator: linea 2px accent-team con glow soft (decisione #5/#14) */
  .card-wrap.drop-before::before,
  .card-wrap.drop-after::after {
    content: "";
    position: absolute;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--accent-team);
    box-shadow:
      0 0 0 1px var(--accent-team),
      0 0 12px -2px color-mix(in oklch, var(--accent-team) 40%, transparent);
    pointer-events: none;
    z-index: 2;
  }

  .card-wrap.drop-before::before {
    top: -1px;
  }

  .card-wrap.drop-after::after {
    bottom: -1px;
  }
</style>
