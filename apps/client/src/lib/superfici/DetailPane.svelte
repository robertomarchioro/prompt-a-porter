<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy, untrack } from "svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { Star, GitFork, Download, PanelRight, Save, Trash2 } from "lucide-svelte";
  import type { EditorView } from "@codemirror/view";
  import DetailTabs, { type TabId } from "$lib/components/DetailTabs.svelte";
  import EditorTab from "$lib/components/EditorTab.svelte";
  import EditorIndicator from "$lib/components/EditorIndicator.svelte";
  import MarkdownToolbar from "$lib/components/MarkdownToolbar.svelte";
  import RightRail from "$lib/components/RightRail.svelte";
  import AnteprimaTab from "$lib/components/AnteprimaTab.svelte";
  import DiagnosiTab from "$lib/components/DiagnosiTab.svelte";
  import GoldenTab from "$lib/components/GoldenTab.svelte";
  import CronologiaTab from "$lib/components/CronologiaTab.svelte";
  import ImportVarTab from "$lib/components/ImportVarTab.svelte";
  import Modale from "$lib/components/Modale.svelte";
  import Button from "$lib/components/Button.svelte";
  import { apriModale } from "$lib/stores/modale.svelte";
  import { statoEditor } from "$lib/stores/preferenze.svelte";

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
    // M3 PR-5: id del prompt principale se questo e' una variante,
    // null se questo e' il principale. Passato a RightRail per mostrare
    // il bottone "Promuovi a principale" solo sulle varianti.
    parent_prompt_id: string | null;
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
  let diagnosiCount = $state(0);
  let goldenCount = $state(0);
  let cronologiaCount = $state(0);
  let importVarCount = $state(0);

  let timerAutosave: ReturnType<typeof setTimeout> | undefined;

  // Issue #158: `dirty` traccia "modifiche autosave non confermate dal
  // salvataggio manuale". Autosave aggiorna body in DB ma NON crea
  // riga in PromptVersions; solo `salvaManuale()` crea cronologia.
  // Switch prompt o Compila con dirty=true → forziamo salvaManuale.
  // Beforeunload con dirty=true → dialog conferma chiusura.
  let dirty = $state(false);
  // Track del promptId precedente per gestire dirty al cambio prompt.
  let promptIdPrec = $state<string | null>(null);

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
    // Issue #158 + #167 + #170: switch prompt con dirty del precedente →
    // forza snapshot del PRECEDENTE prima di caricare il NUOVO.
    //
    // LA SOLA DIPENDENZA REATTIVA È `promptId`. Tutte le altre letture
    // (titolo/body/descrizione/dirty/dettaglio/promptIdPrec) DEVONO essere
    // in `untrack()` altrimenti Svelte 5 le tratta come dipendenze e
    // l'effect ri-esegue ad ogni keystroke dell'utente → caricaDettaglio
    // sovrascrive l'input. Issue #170 era esattamente questo loop: ogni
    // carattere digitato veniva immediatamente cancellato.
    const idNuovo = promptId;

    const {
      idPrec,
      dirtyPrec,
      dettaglioPrec,
      titoloPrec,
      descPrec,
      bodyPrec,
    } = untrack(() => ({
      idPrec: promptIdPrec,
      dirtyPrec: dirty,
      dettaglioPrec: dettaglio,
      titoloPrec: titolo,
      descPrec: descrizione,
      bodyPrec: body,
    }));

    untrack(() => {
      promptIdPrec = idNuovo;
    });

    void (async () => {
      if (
        idPrec !== null &&
        idPrec !== idNuovo &&
        dirtyPrec &&
        dettaglioPrec
      ) {
        // Cancel autosave pending del precedente: il save manuale lo include.
        if (timerAutosave) {
          clearTimeout(timerAutosave);
          timerAutosave = undefined;
        }
        await salvaConId(
          idPrec,
          titoloPrec,
          descPrec,
          bodyPrec,
          dettaglioPrec,
          true,
        );
      }
      await caricaDettaglio(idNuovo);
    })();
  });

  function onGotoLine(e: Event): void {
    const linea = (e as CustomEvent<number>).detail;
    if (typeof linea !== "number" || linea < 1) return;
    tabAttivo = "editor";
    // Scroll dell'editor alla riga (cross-tab navigation da F5 PR-B Diagnosi).
    // Defer al next tick per dare tempo al tab di rimontare l'EditorTab.
    setTimeout(() => {
      const view = editorView;
      if (!view) return;
      const lineCount = view.state.doc.lines;
      const target = Math.min(Math.max(linea, 1), lineCount);
      const lineObj = view.state.doc.line(target);
      view.dispatch({
        selection: { anchor: lineObj.from, head: lineObj.to },
        scrollIntoView: true,
      });
      view.focus();
    }, 50);
  }

  onMount(async () => {
    try {
      cartelleCache = await invoke<Cartella[]>("folder_lista");
    } catch {
      /* ignore */
    }
    window.addEventListener("pap:goto-line", onGotoLine);
    window.addEventListener("beforeunload", onBeforeUnload);
  });

  onDestroy(() => {
    if (timerAutosave) clearTimeout(timerAutosave);
    window.removeEventListener("pap:goto-line", onGotoLine);
    window.removeEventListener("beforeunload", onBeforeUnload);
  });

  function pianificaAutosave(): void {
    if (!dettaglio) return;
    statoSalvataggio = "dirty";
    dirty = true;
    if (timerAutosave) clearTimeout(timerAutosave);
    timerAutosave = setTimeout(
      () => void salvaBozza(),
      statoEditor.autosaveDelayMs,
    );
  }

  /**
   * Issue #158: salvataggio interno generico. `creaSnapshot=false` =
   * autosave (no riga in PromptVersions). `creaSnapshot=true` = save
   * manuale dell'utente (crea versione in cronologia).
   *
   * Issue #167: gli argomenti sono espliciti (no closure su reattive)
   * per evitare race in cui `promptId` è già il nuovo ma `body`/`titolo`
   * sono ancora del precedente (catastrofic data-loss).
   *
   * Ritorna true se l'invoke è andato a buon fine, false altrimenti.
   */
  async function salvaConId(
    idTarget: string,
    titoloTarget: string,
    descTarget: string,
    bodyTarget: string,
    dettaglioTarget: PromptDettaglio,
    creaSnapshot: boolean,
  ): Promise<boolean> {
    if (!titoloTarget.trim() || !bodyTarget.trim()) return false;
    statoSalvataggio = "salvando";
    try {
      await invoke("prompt_aggiorna", {
        dati: {
          id: idTarget,
          titolo: titoloTarget.trim(),
          descrizione: descTarget.trim(),
          body: bodyTarget.trim(),
          visibilita: dettaglioTarget.visibilita,
          tag_nomi: dettaglioTarget.tags.map((t) => t.nome),
          target_model: dettaglioTarget.target_model || null,
          folder_id: dettaglioTarget.folder_id,
          crea_snapshot: creaSnapshot,
        },
      });
      statoSalvataggio = "salvato";
      salvatoTs = new Date().toISOString();
      if (creaSnapshot) {
        // Solo il save manuale resetta dirty: autosave silenzioso lascia
        // dirty=true così l'utente sa che ci sono modifiche non snapshot-ate.
        dirty = false;
      }
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
      return true;
    } catch (e) {
      console.error("[detail] save", e);
      statoSalvataggio = "errore";
      return false;
    }
  }

  /**
   * Salva il prompt **corrente** (legge le reattive). Usare SOLO quando
   * non c'è ambiguità su quale prompt si sta salvando — cioè dal flow
   * utente diretto (autosave su input, click Salva). Per il caso "save
   * del precedente al cambio prompt" usare invece `salvaConId()` con
   * snapshot esplicito delle variabili (vedi $effect su promptId).
   */
  async function salva(creaSnapshot: boolean): Promise<boolean> {
    if (!dettaglio) return false;
    return salvaConId(
      promptId,
      titolo,
      descrizione,
      body,
      dettaglio,
      creaSnapshot,
    );
  }

  async function salvaBozza(): Promise<void> {
    await salva(false);
  }

  /**
   * Issue #158: salvataggio manuale dell'utente — crea snapshot in
   * PromptVersions, reset dirty. Triggato da bottone "Salva", da
   * Compila se dirty, da beforeunload (caso prompt corrente).
   */
  async function salvaManuale(): Promise<boolean> {
    // Cancel autosave pending: il manuale lo include.
    if (timerAutosave) {
      clearTimeout(timerAutosave);
      timerAutosave = undefined;
    }
    return salva(true);
  }

  /**
   * Issue #156: elimina prompt corrente con confirm dialog.
   * Backend prompt_elimina è soft-delete (DeletedAt timestamp).
   */
  // #303: stato del warning per cancellazione di prompt importati da altri.
  interface Dipendente {
    id: string;
    titolo: string;
  }
  let warnImport = $state<{
    aperto: boolean;
    deps: Dipendente[];
    lavorando: boolean;
  }>({ aperto: false, deps: [], lavorando: false });

  /** Soft-delete effettivo + cleanup + chiusura. Condiviso dai due percorsi. */
  async function eseguiSoftDelete(): Promise<void> {
    await invoke("prompt_elimina", { id: promptId });
    // Cancel autosave pending: eviterebbe di riscrivere il prompt
    // appena cancellato.
    if (timerAutosave) {
      clearTimeout(timerAutosave);
      timerAutosave = undefined;
    }
    dirty = false;
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    _onChiudi?.();
  }

  async function eliminaPrompt(): Promise<void> {
    if (!dettaglio) return;
    const titoloVis = dettaglio.titolo || "(senza titolo)";
    // #303: se altri prompt importano questo, mostra il warning dedicato.
    let deps: Dipendente[] = [];
    try {
      deps = await invoke<Dipendente[]>("prompt_dipendenti", { id: promptId });
    } catch (e) {
      console.error("[detail] prompt_dipendenti", e);
    }
    if (deps.length > 0) {
      warnImport = { aperto: true, deps, lavorando: false };
      return;
    }
    const ok = window.confirm(
      `Eliminare il prompt "${titoloVis}"?\n\nIl prompt verrà spostato nel Cestino, da cui potrai ripristinarlo o eliminarlo definitivamente.`,
    );
    if (!ok) return;
    try {
      await eseguiSoftDelete();
    } catch (e) {
      console.error("[detail] elimina", e);
      window.alert("Errore nell'eliminazione del prompt: " + String(e));
    }
  }

  /** #303: rimuove gli import dai dipendenti, poi cancella il prompt. */
  async function rimuoviImportECancella(): Promise<void> {
    warnImport.lavorando = true;
    try {
      await invoke("import_rimuovi_da_dipendenti", { targetId: promptId });
      await eseguiSoftDelete();
      warnImport = { aperto: false, deps: [], lavorando: false };
    } catch (e) {
      console.error("[detail] rimuovi import e cancella", e);
      window.alert("Errore durante la rimozione degli import: " + String(e));
      warnImport.lavorando = false;
    }
  }

  /**
   * Issue #158: apertura Compila con dirty force-snapshot prima di
   * lanciare la modale (così la versione compilata coincide con
   * quella in cronologia).
   */
  async function apriCompila(): Promise<void> {
    if (dirty) {
      const ok = await salvaManuale();
      if (!ok) {
        window.alert("Impossibile salvare il prompt prima di aprire Compila.");
        return;
      }
    }
    apriModale({ tipo: "compila", promptId });
  }

  /**
   * Issue #158: beforeunload (chiusura app) con dirty mostra dialog
   * standard del browser/webview. L'utente può scegliere "Annulla"
   * per restare oppure proseguire (in tal caso le modifiche autosave
   * sono già in DB ma niente snapshot in PromptVersions).
   */
  function onBeforeUnload(e: BeforeUnloadEvent): void {
    if (dirty) {
      e.preventDefault();
      e.returnValue = "";
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
  <article class="detail-pane" data-tour="editor">
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
            class="ico"
            class:dirty-pulse={dirty}
            type="button"
            title={dirty
              ? "Salva (modifiche non confermate in cronologia)"
              : "Salva (nessuna modifica)"}
            aria-label="Salva"
            disabled={!dirty}
            onclick={() => void salvaManuale()}
          >
            <Save size={14} />
          </button>
          <button
            class="ico danger"
            type="button"
            title="Elimina prompt"
            aria-label="Elimina prompt"
            onclick={() => void eliminaPrompt()}
          >
            <Trash2 size={14} />
          </button>
          <button
            class="primary"
            type="button"
            title="Compila (apre modale)"
            onclick={() => void apriCompila()}
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

      <DetailTabs
        {tabAttivo}
        badge={{
          diagnosi: diagnosiCount,
          golden: goldenCount,
          cronologia: cronologiaCount,
          importVar: importVarCount,
        }}
        onSeleziona={(t) => (tabAttivo = t)}
      />
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
          {:else if tabAttivo === "anteprima"}
            <AnteprimaTab {body} {promptId} />
          {:else if tabAttivo === "diagnosi"}
            <DiagnosiTab
              {promptId}
              {body}
              onConteggio={(n) => (diagnosiCount = n)}
            />
          {:else if tabAttivo === "golden"}
            <GoldenTab
              {promptId}
              onConteggio={(n) => (goldenCount = n)}
            />
          {:else if tabAttivo === "cronologia"}
            <CronologiaTab
              {promptId}
              onConteggio={(n) => (cronologiaCount = n)}
            />
          {:else if tabAttivo === "import-var"}
            <ImportVarTab
              {promptId}
              {body}
              onConteggio={(n) => (importVarCount = n)}
            />
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
            parentPromptId={dettaglio.parent_prompt_id}
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

{#if warnImport.aperto}
  <Modale
    titolo="Questo prompt è importato da altri"
    sottotitolo={dettaglio?.titolo || "(senza titolo)"}
    larghezza="sm"
    onChiudi={() => (warnImport = { aperto: false, deps: [], lavorando: false })}
  >
    <div class="warn-import">
      <p class="warn-testo">
        {warnImport.deps.length === 1
          ? "1 prompt importa questo prompt via {{import}}. Cancellandolo, quell'import si romperà:"
          : `${warnImport.deps.length} prompt importano questo prompt via {{import}}. Cancellandolo, quegli import si romperanno:`}
      </p>
      <ul class="warn-lista">
        {#each warnImport.deps as d (d.id)}
          <li>{d.titolo || "(senza titolo)"}</li>
        {/each}
      </ul>
    </div>
    {#snippet footer()}
      <Button
        variante="ghost"
        onclick={() => (warnImport = { aperto: false, deps: [], lavorando: false })}
        disabled={warnImport.lavorando}
      >
        Annulla
      </Button>
      <Button
        variante="danger"
        onclick={rimuoviImportECancella}
        disabled={warnImport.lavorando}
      >
        {warnImport.lavorando
          ? "Rimozione…"
          : warnImport.deps.length === 1
            ? "Rimuovi l'import e cancella"
            : `Rimuovi gli import dai ${warnImport.deps.length} prompt e cancella`}
      </Button>
    {/snippet}
  </Modale>
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

  /* Issue #156: bottone Trash header — colore danger su hover. */
  .ico.danger:hover {
    color: var(--accent-danger, #d9534f);
    background: var(--accent-danger-soft, rgba(217, 83, 79, 0.1));
  }

  /* Issue #158: bottone Salva header — pulse subtle quando dirty per
     attirare attenzione utente sulle modifiche non confermate. */
  .ico.dirty-pulse {
    color: var(--accent-team);
  }
  .ico.dirty-pulse:not(:disabled) {
    animation: dirty-pulse 2s ease-in-out infinite;
  }
  @keyframes dirty-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }
  .ico:disabled {
    opacity: 0.4;
    cursor: not-allowed;
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

  .warn-import {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .warn-testo {
    margin: 0;
    color: var(--text-default);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }

  .warn-lista {
    margin: 0;
    padding-left: var(--sp-4);
    max-height: 180px;
    overflow-y: auto;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
