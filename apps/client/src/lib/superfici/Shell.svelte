<script lang="ts">
  /**
   * Shell — root della UI redesign v0.8 (post v0.8.1 issue #137).
   *
   * Layout 3-pane via **CSS grid puro** come da prototipo originale
   * (`docs/architettura/redesign/prototype/redesign.css:117`):
   *
   *   .shell-body {
   *     grid-template-columns:
   *       var(--col-sidebar) 1px var(--col-list) 1px minmax(0, 1fr);
   *   }
   *
   * Drag handler manuali (pointermove/pointerup) aggiornano le var
   * `--col-sidebar` / `--col-list` clamped tra MIN/MAX. Persistenza in
   * localStorage via `shell-layout.ts`. Quando ListPane è collapsed,
   * lo slot resta visibile a 36px con un bottone "list-restore" per
   * ri-aprirlo.
   *
   * Sostituisce paneforge che non rispettava le proporzioni del proto
   * (248/320px) ed era confuso da `collapsedSize=0` (vedi issue #137:
   * drag invertito + lista che spariva del tutto).
   */
  import { onDestroy, onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { ChevronsRight } from "lucide-svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SidebarMini from "$lib/components/SidebarMini.svelte";
  import ListPane from "$lib/components/ListPane.svelte";
  import CestinoVista from "$lib/components/CestinoVista.svelte";
  import DetailPane from "$lib/superfici/DetailPane.svelte";
  import DiffLibero from "$lib/superfici/DiffLibero.svelte";
  import CompilaModal from "$lib/superfici/CompilaModal.svelte";
  import InsightModal from "$lib/superfici/InsightModal.svelte";
  import RegressioniModal from "$lib/superfici/RegressioniModal.svelte";
  import ImpostazioniModal from "$lib/superfici/ImpostazioniModal.svelte";
  import PaletteModal from "$lib/superfici/PaletteModal.svelte";
  import NuovaCartellaModal from "$lib/superfici/NuovaCartellaModal.svelte";
  import {
    statoModale,
    chiudiModale,
    apriModale,
  } from "$lib/stores/modale.svelte";
  import {
    caricaStato as caricaStatoSidebar,
    salvaStato as salvaStatoSidebar,
    type StatoSidebar,
  } from "$lib/stores/sidebar-collapsed";
  import {
    caricaStato as caricaStatoLayout,
    salvaStato as salvaStatoLayout,
    COL_SIDEBAR_MIN,
    COL_SIDEBAR_MAX,
    COL_LIST_MIN,
    COL_LIST_MAX,
  } from "$lib/stores/shell-layout";

  // F2: stato collapsed sidebar + gruppi NavGroup, persistito in localStorage.
  let stato = $state<StatoSidebar>(caricaStatoSidebar());

  // Larghezze 2 colonne resizable in pixel, persistite in localStorage.
  let layout = $state(caricaStatoLayout());

  // Stato collapse ListPane (separato dalla sidebar). In-memory only:
  // alla riapertura dell'app la lista riparte espansa, come da prototipo.
  let listCollapsed = $state(false);

  let saveSidebarTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void stato.sidebarCollapsed;
    void stato.gruppi.viste;
    void stato.gruppi.visibilita;
    void stato.gruppi.cartelle;
    void stato.gruppi.tag;
    void stato.gruppi.modelTarget;
    if (saveSidebarTimer) clearTimeout(saveSidebarTimer);
    saveSidebarTimer = setTimeout(() => salvaStatoSidebar(stato), 200);
  });

  let saveLayoutTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void layout.colSidebar;
    void layout.colList;
    if (saveLayoutTimer) clearTimeout(saveLayoutTimer);
    saveLayoutTimer = setTimeout(() => salvaStatoLayout(layout), 200);
  });

  // Stato filtri condivisi Sidebar ↔ ListPane (F2 + F3).
  let vistaCorrente = $state("recenti");
  let folderSelezionato = $state<string | null>(null);
  let tagSelezionato = $state<string | null>(null);
  let modelTargetSelezionato = $state("");
  let promptSelezionato = $state<string | null>(null);

  // F5 PR-F: selezione multipla per Diff libero.
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

  // F7: notifica StatusBar quando il prompt viene deselezionato.
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

  // F5 PR-E + F6: navigazione cross-prompt via custom event.
  function onApriPrompt(e: Event): void {
    const id = (e as CustomEvent<string>).detail;
    if (typeof id === "string" && id) {
      promptSelezionato = id;
    }
  }

  // F8 PR-D1/PR-E: shortcut globali per modali ricorrenti.
  function onShortcutGlobale(e: KeyboardEvent): void {
    if (!(e.metaKey || e.ctrlKey)) return;
    if (e.key === ",") {
      e.preventDefault();
      apriModale({ tipo: "impostazioni" });
    } else if (e.key === "k" || e.key === "K") {
      e.preventDefault();
      apriModale({ tipo: "palette" });
    } else if (e.shiftKey && (e.key === "P" || e.key === "p")) {
      e.preventDefault();
      apriModale({ tipo: "palette" });
    }
  }

  // ─── Drag handler resizer (issue #137) ───
  // Pattern: onpointerdown su .resizer cattura pointer, listener
  // pointermove + pointerup su window aggiornano la var corrispondente
  // clamped tra MIN/MAX. Niente paneforge → drag direzione corretta:
  // mouse a destra ⇒ pane a sinistra cresce.
  type ResizeKind = "sidebar" | "list";

  let dragKind = $state<ResizeKind | null>(null);
  let dragStartX = 0;
  let dragStartCol = 0;

  function onPointerDownResizer(kind: ResizeKind, e: PointerEvent): void {
    if (e.button !== 0) return;
    e.preventDefault();
    dragKind = kind;
    dragStartX = e.clientX;
    dragStartCol = kind === "sidebar" ? layout.colSidebar : layout.colList;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    window.addEventListener("pointermove", onPointerMoveResizer);
    window.addEventListener("pointerup", onPointerUpResizer, { once: true });
  }

  function onPointerMoveResizer(e: PointerEvent): void {
    if (!dragKind) return;
    const delta = e.clientX - dragStartX;
    const next = dragStartCol + delta;
    if (dragKind === "sidebar") {
      layout = {
        ...layout,
        colSidebar: Math.min(
          COL_SIDEBAR_MAX,
          Math.max(COL_SIDEBAR_MIN, Math.round(next)),
        ),
      };
    } else {
      layout = {
        ...layout,
        colList: Math.min(
          COL_LIST_MAX,
          Math.max(COL_LIST_MIN, Math.round(next)),
        ),
      };
    }
  }

  function onPointerUpResizer(): void {
    dragKind = null;
    window.removeEventListener("pointermove", onPointerMoveResizer);
  }

  // Issue #146: tray menu items (Nuovo prompt / Impostazioni) emettono
  // event Tauri verso il webview. Qui ci registriamo per tradurli in
  // azioni client. `mostra_libreria` e `apri_palette` sono gestiti
  // interamente backend-side, qui non serve nulla.
  let unlistenTray: UnlistenFn[] = [];

  onMount(() => {
    window.addEventListener("pap:apri-prompt", onApriPrompt);
    window.addEventListener("keydown", onShortcutGlobale);
    void (async () => {
      unlistenTray.push(
        await listen("tray:apri-impostazioni", () => {
          apriModale({ tipo: "impostazioni" });
        }),
      );
      unlistenTray.push(
        await listen("tray:nuovo-prompt", () => {
          // Delega a ListPane via window event: ListPane ha già il
          // contesto folder/filtri e la logica creaNuovoPrompt.
          window.dispatchEvent(new CustomEvent("pap:nuovo-prompt"));
        }),
      );
    })();
  });

  onDestroy(() => {
    window.removeEventListener("pap:apri-prompt", onApriPrompt);
    window.removeEventListener("keydown", onShortcutGlobale);
    window.removeEventListener("pointermove", onPointerMoveResizer);
    unlistenTray.forEach((u) => u());
    unlistenTray = [];
  });
</script>

<div class="shell-root">
  <TitleBar />
  <main
    class="shell-body"
    data-sidebar-collapsed={stato.sidebarCollapsed}
    data-list-collapsed={listCollapsed}
    style="--col-sidebar: {layout.colSidebar}px; --col-list: {layout.colList}px;"
  >
    {#if stato.sidebarCollapsed}
      <div class="pane sidebar-mini-pane">
        <SidebarMini
          onApriExpand={() => (stato.sidebarCollapsed = false)}
          onApriInsight={() => apriModale({ tipo: "insight" })}
          onApriRegressioni={() => apriModale({ tipo: "regressioni" })}
        />
      </div>
    {:else}
      <div class="pane sidebar-pane">
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
          onApriInsight={() => apriModale({ tipo: "insight" })}
          onApriRegressioni={() => apriModale({ tipo: "regressioni" })}
          onAggiungiCartella={() => apriModale({ tipo: "nuova-cartella" })}
        />
      </div>
    {/if}

    <!--
      svelte-ignore a11y_no_noninteractive_tabindex
      WAI-ARIA: resizable separator pattern - role=separator + tabindex=0
      e' la combinazione corretta per uno splitter ridimensionabile da
      tastiera (cfr. https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/).
      svelte-check non riconosce questo pattern e lo segnala come falso
      positivo.
    -->
    <div
      class="resizer"
      class:dragging={dragKind === "sidebar"}
      role="separator"
      aria-orientation="vertical"
      aria-label="Ridimensiona sidebar"
      tabindex="0"
      onpointerdown={(e) => onPointerDownResizer("sidebar", e)}
    ></div>

    <div class="pane list-pane" class:collapsed={listCollapsed && vistaCorrente !== "cestino"}>
      {#if vistaCorrente === "cestino"}
        <CestinoVista />
      {:else if listCollapsed}
        <button
          class="list-restore"
          type="button"
          aria-label="Riapri elenco prompt"
          title="Riapri elenco prompt"
          onclick={() => (listCollapsed = false)}
        >
          <ChevronsRight size={14} />
        </button>
      {:else}
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
          onApriCollapse={() => (listCollapsed = true)}
          {selezioneMultipla}
          onToggleSelezione={toggleSelezione}
          onPulisciSelezione={pulisciSelezione}
          onConfronta={apriDiffLibero}
        />
      {/if}
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_tabindex (vedi nota resizer sidebar sopra) -->
    <div
      class="resizer"
      class:dragging={dragKind === "list"}
      class:hidden={listCollapsed}
      role="separator"
      aria-orientation="vertical"
      aria-label="Ridimensiona lista prompt"
      tabindex="0"
      onpointerdown={(e) => onPointerDownResizer("list", e)}
    ></div>

    <div class="pane detail-pane">
      {#if vistaCorrente === "cestino"}
        <div class="placeholder-pane detail-placeholder">
          <p>Ripristina o elimina i prompt dal cestino.</p>
        </div>
      {:else if promptSelezionato}
        <DetailPane
          promptId={promptSelezionato}
          onChiudi={() => (promptSelezionato = null)}
        />
      {:else}
        <div class="placeholder-pane detail-placeholder">
          <p>Seleziona un prompt dalla lista</p>
        </div>
      {/if}
    </div>
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

{#if statoModale.attiva?.tipo === "insight"}
  <InsightModal onChiudi={chiudiModale} />
{/if}

{#if statoModale.attiva?.tipo === "regressioni"}
  <RegressioniModal onChiudi={chiudiModale} />
{/if}

{#if statoModale.attiva?.tipo === "impostazioni"}
  <ImpostazioniModal
    sezioneIniziale={statoModale.attiva.sezione as
      | "aspetto"
      | "vista"
      | "editor"
      | "sicurezza"
      | "avanzate"
      | "guida"
      | undefined}
    onChiudi={chiudiModale}
  />
{/if}

{#if statoModale.attiva?.tipo === "palette"}
  <PaletteModal onChiudi={chiudiModale} />
{/if}

{#if statoModale.attiva?.tipo === "nuova-cartella"}
  <NuovaCartellaModal onChiudi={chiudiModale} />
{/if}

<style>
  .shell-root {
    display: grid;
    grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h);
    height: 100vh;
    width: 100vw;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
    overflow: hidden;
  }

  .shell-body {
    display: grid;
    grid-template-columns:
      var(--col-sidebar, 248px)
      1px
      var(--col-list, 320px)
      1px
      minmax(0, 1fr);
    min-height: 0;
    background: var(--bg-canvas);
    overflow: hidden;
  }

  /* Sidebar collapsed (icon-only mini bar) */
  .shell-body[data-sidebar-collapsed="true"] {
    grid-template-columns:
      44px
      1px
      var(--col-list, 320px)
      1px
      minmax(0, 1fr);
  }

  /* List collapsed: pane resta a 36px, mostra solo bottone restore.
     Resizer a destra del list-pane viene nascosto (.hidden). */
  .shell-body[data-list-collapsed="true"] {
    grid-template-columns:
      var(--col-sidebar, 248px)
      1px
      36px
      0
      minmax(0, 1fr);
  }

  .shell-body[data-sidebar-collapsed="true"][data-list-collapsed="true"] {
    grid-template-columns: 44px 1px 36px 0 minmax(0, 1fr);
  }

  .pane {
    min-width: 0;
    overflow: hidden;
  }

  .list-pane.collapsed {
    background: var(--bg-surface);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: var(--sp-2, 12px);
  }

  .list-restore {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: 1px solid var(--border-default);
    background: var(--bg-canvas);
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .list-restore:hover {
    background: var(--bg-overlay);
    color: var(--text-strong);
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

  .resizer {
    background: var(--border-subtle);
    cursor: col-resize;
    position: relative;
    z-index: 10;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  /* Hit-area allargata ±3px senza alterare la grid track (1px) */
  .resizer::after {
    content: "";
    position: absolute;
    inset: 0;
    left: -3px;
    right: -3px;
  }

  .resizer:hover,
  .resizer.dragging {
    background: var(--accent-team);
    transition: none;
  }

  .resizer.hidden {
    pointer-events: none;
    background: transparent;
  }
</style>
