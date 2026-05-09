<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SidebarMini from "$lib/components/SidebarMini.svelte";
  import ListPane from "$lib/components/ListPane.svelte";
  import DetailPane from "$lib/superfici/DetailPane.svelte";
  import DiffLibero from "$lib/superfici/DiffLibero.svelte";
  import CompilaModal from "$lib/superfici/CompilaModal.svelte";
  import { statoModale, chiudiModale } from "$lib/stores/modale.svelte";
  import {
    caricaStato,
    salvaStato,
    type StatoSidebar,
  } from "$lib/stores/sidebar-collapsed";

  // F2: stato collapsed sidebar + gruppi NavGroup, persistito in localStorage.
  let stato = $state<StatoSidebar>(caricaStato());

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void stato.sidebarCollapsed;
    void stato.gruppi.viste;
    void stato.gruppi.visibilita;
    void stato.gruppi.cartelle;
    void stato.gruppi.tag;
    void stato.gruppi.modelTarget;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => salvaStato(stato), 200);
  });

  // Stato filtri condivisi Sidebar ↔ ListPane (F2 + F3).
  let vistaCorrente = $state("recenti");
  let folderSelezionato = $state<string | null>(null);
  let tagSelezionato = $state<string | null>(null);
  let modelTargetSelezionato = $state("");
  let promptSelezionato = $state<string | null>(null);
  // Nota: cross-folder drop (drag prompt → cartella sidebar) sarà F3.x.
  // F3 PR-A copre solo drag-reorder dentro la lista.

  // F5 PR-F: selezione multipla per Diff libero (Cmd/Ctrl+click in ListPane)
  let selezioneMultipla = $state<Set<string>>(new Set());
  let mostraDiffLibero = $state(false);

  function toggleSelezione(id: string): void {
    const next = new Set(selezioneMultipla);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selezioneMultipla = next;
  }

  function pulisciSelezione(): void {
    selezioneMultipla = new Set();
  }

  function apriDiffLibero(): void {
    if (selezioneMultipla.size >= 2 && selezioneMultipla.size <= 4) {
      mostraDiffLibero = true;
    }
  }

  // F7: notifica StatusBar quando il prompt viene deselezionato
  $effect(() => {
    if (promptSelezionato === null) {
      window.dispatchEvent(
        new CustomEvent("pap:prompt-corrente", { detail: null }),
      );
      window.dispatchEvent(
        new CustomEvent("pap:save-stato", { detail: { stato: null } }),
      );
    }
  });

  // F5 PR-E + F6: navigazione cross-prompt via custom event
  // (Import composti, Varianti A/B/C → click apre nel detail)
  function onApriPrompt(e: Event): void {
    const id = (e as CustomEvent<string>).detail;
    if (typeof id === "string" && id) {
      promptSelezionato = id;
    }
  }

  onMount(() => {
    window.addEventListener("pap:apri-prompt", onApriPrompt);
  });

  onDestroy(() => {
    window.removeEventListener("pap:apri-prompt", onApriPrompt);
  });
</script>

<div class="shell-root">
  <TitleBar />
  <main class="shell-body">
    <PaneGroup direction="horizontal" autoSaveId="redesign-shell-v08">
      {#if stato.sidebarCollapsed}
        <Pane defaultSize={4} minSize={4} maxSize={4}>
          <SidebarMini
            onApriExpand={() => (stato.sidebarCollapsed = false)}
            onApriInsight={() => console.log("F8 modale Insight")}
            onApriRegressioni={() => console.log("F8 modale Regressioni")}
          />
        </Pane>
      {:else}
        <Pane defaultSize={20} minSize={14} maxSize={30}>
          <Sidebar
            {vistaCorrente}
            {folderSelezionato}
            {tagSelezionato}
            {modelTargetSelezionato}
            bind:gruppi={stato.gruppi}
            onSelezionaVista={(id) => (vistaCorrente = id)}
            onSelezionaFolder={(id) => (folderSelezionato = id)}
            onSelezionaTag={(id) => (tagSelezionato = id)}
            onSelezionaModelTarget={(m) => (modelTargetSelezionato = m)}
            onApriCollapse={() => (stato.sidebarCollapsed = true)}
            onApriInsight={() => console.log("F8 modale Insight")}
            onApriRegressioni={() => console.log("F8 modale Regressioni")}
          />
        </Pane>
      {/if}
      <PaneResizer class="resizer" />
      <Pane defaultSize={26} minSize={0} maxSize={40}>
        <ListPane
          {vistaCorrente}
          {folderSelezionato}
          {tagSelezionato}
          {modelTargetSelezionato}
          {promptSelezionato}
          onRimuoviFolder={() => (folderSelezionato = null)}
          onRimuoviTag={() => (tagSelezionato = null)}
          onRimuoviModelTarget={() => (modelTargetSelezionato = "")}
          onSelezionaPrompt={(id) => {
            promptSelezionato = id;
          }}
          onApriCollapse={() => console.log("F3.x list collapse")}
          {selezioneMultipla}
          onToggleSelezione={toggleSelezione}
          onPulisciSelezione={pulisciSelezione}
          onConfronta={apriDiffLibero}
        />
      </Pane>
      <PaneResizer class="resizer" />
      <Pane defaultSize={54}>
        {#if promptSelezionato}
          <DetailPane
            promptId={promptSelezionato}
            onChiudi={() => (promptSelezionato = null)}
          />
        {:else}
          <div class="placeholder-pane detail-placeholder">
            <p>Seleziona un prompt dalla lista</p>
          </div>
        {/if}
      </Pane>
    </PaneGroup>
  </main>
  <StatusBar />
</div>

{#if mostraDiffLibero}
  <DiffLibero
    idPrompts={Array.from(selezioneMultipla)}
    onChiudi={() => (mostraDiffLibero = false)}
  />
{/if}

<!-- F8: modali globali (apertura via apriModale dal store) -->
{#if statoModale.attiva?.tipo === "compila"}
  <CompilaModal
    promptId={statoModale.attiva.promptId}
    onChiudi={chiudiModale}
  />
{/if}

<style>
  .shell-root {
    display: grid;
    grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h);
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .shell-body {
    overflow: hidden;
  }

  .placeholder-pane {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .detail-placeholder {
    background: var(--bg-canvas);
  }

  :global(.resizer) {
    width: 1px;
    background: var(--border-subtle);
    position: relative;
    cursor: col-resize;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  :global(.resizer::after) {
    content: "";
    position: absolute;
    inset: 0 -3px;
  }

  :global(.resizer:hover),
  :global(.resizer[data-resize-handle-active]) {
    background: var(--accent-team);
  }
</style>
