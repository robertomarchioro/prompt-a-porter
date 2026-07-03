<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { urlDoc } from "$lib/aiuto/docs-links";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import { segnaPasso } from "$lib/aiuto/primi-passi.svelte";
  import { onMount, onDestroy } from "svelte";
  import {
    ChevronsLeft,
    Search,
    Plus,
    X,
    Rows3,
    Rows2,
    LayoutList,
    FileText,
    Play,
    Star,
    GitFork,
    Trash2,
    FolderInput,
    FileDown,
    Tag,
    Check,
  } from "lucide-svelte";
  import PromptCard from "./PromptCard.svelte";
  import {
    apriMenu,
    type VoceMenu,
  } from "$lib/stores/menu-contestuale.svelte";
  import { apriModale } from "$lib/stores/modale.svelte";
  import { scaricaBlob, slugFile } from "$lib/util/dati-export";
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
    /** Issue #141: body troncato a 800 char server-side, usato solo
     *  nella densità "anteprima". Vuoto se densità ≠ anteprima. */
    body_preview: string;
    /** #403: id del prompt principale se questo è una variante; null se
     *  è un principale. Già esposto da libreria_lista. */
    parent_prompt_id: string | null;
    /** Voto medio degli ultimi 90 giorni (stessa finestra dell'ordine
     *  "Migliori"), in [-1, 1]; null se nessun voto nella finestra.
     *  Mostrato in lista al posto del conteggio usi quando ordine = qualita. */
    rating_medio: number | null;
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
    /** F5 PR-F: set di prompt selezionati per Diff libero (Cmd/Ctrl+click) */
    selezioneMultipla?: Set<string>;
    onToggleSelezione?: (id: string) => void;
    onPulisciSelezione?: () => void;
    onConfronta?: () => void;
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
    selezioneMultipla,
    onToggleSelezione,
    onPulisciSelezione,
    onConfronta,
  }: Props = $props();

  // Cmd/Ctrl+click su card è gestito inline con onclickcapture sul
  // div .card-wrap (vedi template); il click "normale" arriva al
  // <PromptCard onclick={...}> tramite event bubble standard.

  let stato = $state<StatoLista>(caricaStato());
  let prompts = $state<PromptCardData[]>([]);
  // #403: mappa id→titolo per nominare il prompt padre nel tooltip variante.
  const titoliById = $derived(new Map(prompts.map((p) => [p.id, p.titolo])));
  // L'evidenziazione varianti (rientro + connettore "↳") ha senso solo con
  // ordine "A-Z", dove i titoli — e quindi le sister — tendono a stare
  // vicini. Con gli altri criteri la lista resta piatta.
  const mostraVarianti = $derived(stato.ordine === "alfabetico");
  function varianteTitle(p: PromptCardData): string {
    if (!p.parent_prompt_id) return "";
    const t = titoliById.get(p.parent_prompt_id);
    return t ? `Variante di "${t}"` : "Variante";
  }
  let cerca = $state("");
  let cercaDebounced = $state("");
  let cartelle = $state<Cartella[]>([]);
  // Tutti i tag del vault, per i submenu "Gestisci tag" / "Aggiungi tag a N".
  let tagsTutti = $state<TagInfo[]>([]);

  async function caricaAux(): Promise<void> {
    try {
      const [c, t] = await Promise.all([
        invoke<Cartella[]>("folder_lista"),
        invoke<TagInfo[]>("libreria_tag_lista"),
      ]);
      cartelle = c;
      tagsTutti = t;
    } catch {
      /* ignore */
    }
  }

  // Drag state — module-level fallback per dataTransfer null su browser strict.
  // $state perche' letto reattivamente nel template (class:dragging).
  let draggedId = $state<string | null>(null);
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
  // F8 PR-D1: ricarica stato densita/righePreview quando ImpostazioniModal
  // dispatcha pap:lista-densita-cambiata. Evita reload manuale.
  function onDensitaCambiata(): void {
    stato = caricaStato();
  }

  // Issue #146: Shell ascolta tray menu event "tray:nuovo-prompt" e
  // dispatcha pap:nuovo-prompt verso window. Qui in ListPane traduciamo
  // quell'event in chiamata a creaNuovoPrompt (che è definita più
  // sotto, fa il invoke prompt_crea + selezione automatica).
  function onNuovoPromptDaTray(): void {
    void creaNuovoPrompt();
  }

  onMount(async () => {
    await caricaAux();
    window.addEventListener("pap:lista-mutata", caricaLista);
    window.addEventListener("pap:lista-mutata", caricaAux);
    window.addEventListener("pap:lista-densita-cambiata", onDensitaCambiata);
    window.addEventListener("pap:nuovo-prompt", onNuovoPromptDaTray);
  });

  onDestroy(() => {
    window.removeEventListener("pap:lista-mutata", caricaLista);
    window.removeEventListener("pap:lista-mutata", caricaAux);
    window.removeEventListener("pap:lista-densita-cambiata", onDensitaCambiata);
    window.removeEventListener("pap:nuovo-prompt", onNuovoPromptDaTray);
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

  // Issue #140 + #141: 3 bottoni icon-only invece di chip-label per
  // occupare meno spazio quando la colonna è stretta. Anteprima ora
  // attiva (era `abilitata: false` come placeholder F3 PR-B che non
  // era mai stato cancellato).
  // Icone lucide rese inline nel template (switch case) perché
  // lucide-svelte non si compone con il tipo Component<...> di Svelte 5.
  const opzioniDensita: {
    id: Densita;
    label: string;
    abilitata: boolean;
  }[] = [
    { id: "compatta", label: "Compatta", abilitata: true },
    { id: "comoda", label: "Comoda", abilitata: true },
    { id: "anteprima", label: "Anteprima", abilitata: true },
  ];

  const opzioniOrdine: { id: Ordine; label: string }[] = [
    { id: "recente", label: "Recenti" },
    { id: "popolare", label: "Popolari" },
    { id: "qualita", label: "Migliori" },
    { id: "alfabetico", label: "A-Z" },
  ];

  function selezionaDensita(d: Densita): void {
    stato.densita = d;
  }

  // Issue #145: il bottone "+ Nuovo" era cabled a un placeholder
  // `console.log("F8 modale crea prompt")`. Ora chiama il cmd backend
  // `prompt_crea` esistente (editor.rs:146) con dati minimi default,
  // poi seleziona il nuovo prompt nel DetailPane per editing immediato.
  async function creaNuovoPrompt(): Promise<void> {
    try {
      const folderId =
        folderSelezionato && folderSelezionato !== "__nessuna__"
          ? folderSelezionato
          : null;
      const id = await invoke<string>("prompt_crea", {
        dati: {
          titolo: "Nuovo prompt",
          descrizione: "",
          body: "",
          visibilita: "private",
          tag_nomi: [],
          target_model: null,
          folder_id: folderId,
        },
      });
      // Notifica refresh lista (listener pap:lista-mutata già attivo
      // in onMount di questo stesso component) e seleziona il nuovo
      // prompt nel DetailPane.
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
      onSelezionaPrompt(id);
      segnaPasso("crea"); // Guida — checklist primi passi
    } catch (e) {
      console.error("[list-pane] crea prompt fallito", e);
    }
  }

  // ─── Menu contestuale card prompt (blueprint menu-contestuale §6.1) ───
  function refreshLista(): void {
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
  }

  async function togglePreferito(id: string): Promise<void> {
    try {
      await invoke("libreria_toggle_preferito", { id });
      refreshLista();
    } catch (e) {
      console.error("[list-pane] toggle preferito", e);
    }
  }

  async function forkPrompt(id: string): Promise<void> {
    try {
      const nuovoId = await invoke<string>("prompt_fork", { promptId: id });
      refreshLista();
      onSelezionaPrompt(nuovoId);
    } catch (e) {
      console.error("[list-pane] fork", e);
    }
  }

  async function eliminaPrompt(id: string, titolo: string): Promise<void> {
    if (!confirm(`Spostare "${titolo || "(senza titolo)"}" nel cestino?`)) {
      return;
    }
    try {
      await invoke("prompt_elimina", { id });
      refreshLista();
      // Se il prompt eliminato è aperto nel DetailPane, Shell lo deseleziona.
      window.dispatchEvent(
        new CustomEvent("pap:prompt-eliminato", { detail: id }),
      );
    } catch (e) {
      console.error("[list-pane] elimina", e);
    }
  }

  async function spostaInCartella(
    id: string,
    folderId: string | null,
  ): Promise<void> {
    try {
      await invoke<void>("prompt_sposta", {
        dati: { prompt_id: id, folder_id: folderId },
      });
      refreshLista();
    } catch (e) {
      console.error("[list-pane] sposta in cartella", e);
    }
  }

  async function esportaMarkdown(id: string, titolo: string): Promise<void> {
    try {
      const md = await invoke<string>("prompt_export_markdown", {
        promptId: id,
      });
      const blob = new Blob([md], { type: "text/markdown;charset=utf-8" });
      scaricaBlob(blob, `${slugFile(titolo)}.md`);
    } catch (e) {
      console.error("[list-pane] export markdown", e);
    }
  }

  function vociSposta(promptId: string): VoceMenu[] {
    const voci: VoceMenu[] = [
      {
        id: "mv-root",
        label: "Nessuna cartella",
        azione: () => spostaInCartella(promptId, null),
      },
    ];
    if (cartelle.length > 0) {
      voci.push({ separatore: true });
      for (const c of cartelle) {
        voci.push({
          id: `mv-${c.id}`,
          label: c.nome,
          azione: () => spostaInCartella(promptId, c.id),
        });
      }
    }
    return voci;
  }

  async function aggiungiTagAPrompt(
    promptId: string,
    tagNome: string,
  ): Promise<void> {
    try {
      await invoke<void>("prompt_tag_aggiungi", { promptId, tagNome });
      refreshLista();
    } catch (e) {
      console.error("[list-pane] aggiungi tag", e);
    }
  }

  async function rimuoviTagDaPrompt(
    promptId: string,
    tagNome: string,
  ): Promise<void> {
    try {
      await invoke<void>("prompt_tag_rimuovi", { promptId, tagNome });
      refreshLista();
    } catch (e) {
      console.error("[list-pane] rimuovi tag", e);
    }
  }

  // Submenu toggle: i tag già presenti sul prompt hanno la spunta e rimuovono;
  // gli altri aggiungono.
  function vociGestisciTag(p: PromptCardData): VoceMenu[] {
    if (tagsTutti.length === 0) {
      return [
        {
          id: "gt-vuoto",
          label: "Nessun tag nel vault",
          disabilitato: true,
          tooltip: "I tag si creano assegnandoli a un prompt",
        },
      ];
    }
    const presenti = new Set(p.tags.map((t) => t.nome));
    return tagsTutti.map((t) => ({
      id: `gt-${t.id}`,
      label: t.nome,
      icona: presenti.has(t.nome) ? Check : undefined,
      azione: () =>
        presenti.has(t.nome)
          ? rimuoviTagDaPrompt(p.id, t.nome)
          : aggiungiTagAPrompt(p.id, t.nome),
    }));
  }

  function vociPrompt(p: PromptCardData): VoceMenu[] {
    return [
      {
        id: "apri",
        label: "Apri",
        icona: FileText,
        azione: () => onSelezionaPrompt(p.id),
      },
      {
        id: "compila",
        label: "Apri in Compila",
        icona: Play,
        azione: () => apriModale({ tipo: "compila", promptId: p.id }),
      },
      { separatore: true },
      {
        id: "fork",
        label: "Duplica (fork)",
        icona: GitFork,
        azione: () => forkPrompt(p.id),
      },
      {
        id: "preferito",
        label: p.preferito ? "Rimuovi dai preferiti" : "Aggiungi ai preferiti",
        icona: Star,
        azione: () => togglePreferito(p.id),
      },
      { separatore: true },
      {
        id: "sposta",
        label: "Sposta in cartella",
        icona: FolderInput,
        figli: vociSposta(p.id),
      },
      {
        id: "gestisci-tag",
        label: "Gestisci tag",
        icona: Tag,
        figli: vociGestisciTag(p),
      },
      {
        id: "export",
        label: "Esporta come Markdown",
        icona: FileDown,
        azione: () => esportaMarkdown(p.id, p.titolo),
      },
      { separatore: true },
      {
        id: "elimina",
        label: "Elimina",
        icona: Trash2,
        pericolo: true,
        azione: () => eliminaPrompt(p.id, p.titolo),
      },
    ];
  }

  // ─── Menu contestuale selezione multipla (blueprint menu-contestuale §6.8) ───
  // I loop bulk usano `finally` per ripristinare SEMPRE lo stato UI (selezione
  // svuotata + lista ricaricata) anche su fallimento parziale, così l'utente
  // vede cosa è effettivamente cambiato invece di una selezione stale.
  async function spostaBulk(
    ids: string[],
    folderId: string | null,
  ): Promise<void> {
    try {
      for (const id of ids) {
        await invoke<void>("prompt_sposta", {
          dati: { prompt_id: id, folder_id: folderId },
        });
      }
    } catch (e) {
      alert(`Errore durante lo spostamento: ${String(e).replace(/^Error: /, "")}`);
    } finally {
      onPulisciSelezione?.();
      refreshLista();
    }
  }

  async function esportaBulkMarkdown(ids: string[]): Promise<void> {
    try {
      // Ordine = ordine di selezione (Set preserva l'inserimento), unite in un
      // unico file con separatore `---` (evita N download bloccati dal browser).
      const parti: string[] = [];
      for (const id of ids) {
        parti.push(
          await invoke<string>("prompt_export_markdown", { promptId: id }),
        );
      }
      const blob = new Blob([parti.join("\n\n---\n\n")], {
        type: "text/markdown;charset=utf-8",
      });
      scaricaBlob(blob, `prompt-a-porter-export-${ids.length}.md`);
    } catch (e) {
      console.error("[list-pane] export bulk markdown", e);
    }
  }

  async function eliminaBulk(ids: string[]): Promise<void> {
    if (!confirm(`Spostare ${ids.length} prompt nel cestino?`)) return;
    try {
      for (const id of ids) {
        await invoke<void>("prompt_elimina", { id });
        window.dispatchEvent(
          new CustomEvent("pap:prompt-eliminato", { detail: id }),
        );
      }
    } catch (e) {
      alert(`Errore durante l'eliminazione: ${String(e).replace(/^Error: /, "")}`);
    } finally {
      onPulisciSelezione?.();
      refreshLista();
    }
  }

  function vociSpostaBulk(ids: string[]): VoceMenu[] {
    const voci: VoceMenu[] = [
      {
        id: "mvb-root",
        label: "Nessuna cartella",
        azione: () => spostaBulk(ids, null),
      },
    ];
    if (cartelle.length > 0) {
      voci.push({ separatore: true });
      for (const c of cartelle) {
        voci.push({
          id: `mvb-${c.id}`,
          label: c.nome,
          azione: () => spostaBulk(ids, c.id),
        });
      }
    }
    return voci;
  }

  async function aggiungiTagBulk(ids: string[], tagNome: string): Promise<void> {
    // Continue-on-error: un fallimento su un prompt non blocca gli altri.
    let falliti = 0;
    for (const id of ids) {
      try {
        await invoke<void>("prompt_tag_aggiungi", { promptId: id, tagNome });
      } catch (e) {
        falliti++;
        console.error("[list-pane] aggiungi tag bulk", e);
      }
    }
    // La selezione resta: così si possono aggiungere più tag di fila.
    refreshLista();
    if (falliti > 0) {
      alert(`Impossibile aggiungere il tag a ${falliti} prompt su ${ids.length}.`);
    }
  }

  function vociAggiungiTagBulk(ids: string[]): VoceMenu[] {
    if (tagsTutti.length === 0) {
      return [
        {
          id: "bt-vuoto",
          label: "Nessun tag nel vault",
          disabilitato: true,
          tooltip: "I tag si creano assegnandoli a un prompt",
        },
      ];
    }
    return tagsTutti.map((t) => ({
      id: `bt-${t.id}`,
      label: t.nome,
      azione: () => aggiungiTagBulk(ids, t.nome),
    }));
  }

  function vociBulk(ids: string[]): VoceMenu[] {
    const voci: VoceMenu[] = [];
    // Confronta (Diff) supporta solo 2-4 prompt: mostriamo la voce solo quando
    // è azionabile (≥2 è già garantito dal branch in onContextCard).
    if (ids.length <= 4) {
      voci.push(
        {
          id: "confronta",
          label: `Confronta (${ids.length})`,
          azione: () => onConfronta?.(),
        },
        { separatore: true },
      );
    }
    voci.push(
      {
        id: "sposta",
        label: `Sposta ${ids.length} in cartella`,
        icona: FolderInput,
        figli: vociSpostaBulk(ids),
      },
      {
        id: "aggiungi-tag",
        label: `Aggiungi tag a ${ids.length}`,
        icona: Tag,
        figli: vociAggiungiTagBulk(ids),
      },
      {
        id: "export",
        label: `Esporta ${ids.length} come Markdown`,
        icona: FileDown,
        azione: () => esportaBulkMarkdown(ids),
      },
      { separatore: true },
      {
        id: "elimina",
        label: `Elimina ${ids.length}`,
        icona: Trash2,
        pericolo: true,
        azione: () => eliminaBulk(ids),
      },
    );
    return voci;
  }

  function onContextCard(e: MouseEvent, p: PromptCardData): void {
    e.preventDefault();
    const sel = selezioneMultipla;
    if (sel && sel.size >= 2 && sel.has(p.id)) {
      apriMenu(e.clientX, e.clientY, vociBulk([...sel]));
    } else {
      apriMenu(e.clientX, e.clientY, vociPrompt(p));
    }
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
        <ChevronsLeft size={14} />
      </button>
    </div>

    <div class="search-row">
      <div class="search-wrap" data-tour="ricerca">
        <Search size={14} class="search-ico" />
        <input
          class="search-input"
          type="search"
          placeholder="Cerca prompt..."
          bind:value={cerca}
        />
      </div>
      <AiutoLink chiave="ricerca-semantica" dimensione={18} />
      <button
        class="btn-nuovo"
        type="button"
        data-tour="nuovo-prompt"
        onclick={creaNuovoPrompt}
        title="Crea nuovo prompt"
      >
        <Plus size={14} />
        <span>Nuovo</span>
      </button>
    </div>

    <div class="toolbar-row">
      <div class="chip-densita" role="group" aria-label="Densità lista">
        {#each opzioniDensita as opt (opt.id)}
          <button
            class="chip chip-icona"
            class:chip-attivo={stato.densita === opt.id}
            disabled={!opt.abilitata}
            type="button"
            aria-label={opt.label}
            aria-pressed={stato.densita === opt.id}
            title={opt.label}
            onclick={() => selezionaDensita(opt.id)}
          >
            {#if opt.id === "compatta"}
              <Rows3 size={14} />
            {:else if opt.id === "comoda"}
              <Rows2 size={14} />
            {:else}
              <LayoutList size={14} />
            {/if}
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
        <p class="empty-hint">
          Sei all'inizio? <a
            class="empty-link"
            href={urlDoc("getting-started")}
            target="_blank"
            rel="noopener noreferrer">Leggi la guida ai primi passi ↗</a
          >
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
          class:variante={mostraVarianti && !!p.parent_prompt_id}
          class:selezionata-multi={selezioneMultipla?.has(p.id) || undefined}
          draggable="true"
          ondragstart={(e) => gestDragStart(e, p.id)}
          ondragover={(e) => gestDragOverCard(e, idx)}
          ondrop={(e) => gestDropCard(e, idx)}
          ondragend={gestDragEnd}
          oncontextmenu={(e) => onContextCard(e, p)}
          onclickcapture={(e) => {
            if ((e.metaKey || e.ctrlKey) && onToggleSelezione) {
              e.preventDefault();
              e.stopPropagation();
              onToggleSelezione(p.id);
            }
          }}
          role="presentation"
        >
          {#if mostraVarianti && p.parent_prompt_id}
            <span
              class="variante-marca"
              title={varianteTitle(p)}
              aria-label={varianteTitle(p)}>↳</span
            >
          {/if}
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
            bodyPreview={p.body_preview}
            ratingMedio={p.rating_medio}
            mostraRating={stato.ordine === "qualita"}
            onclick={() => onSelezionaPrompt(p.id)}
          />
        </div>
      {/each}
    {/if}
  </div>

  {#if selezioneMultipla && selezioneMultipla.size >= 2}
    <div class="multi-toolbar" role="toolbar" aria-label="Selezione multipla">
      <span class="multi-info">
        {selezioneMultipla.size} prompt selezionati
      </span>
      <button
        class="btn-secondary"
        type="button"
        onclick={onPulisciSelezione}
      >
        Annulla
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={onConfronta}
        disabled={selezioneMultipla.size > 4}
        title={selezioneMultipla.size > 4
          ? "Massimo 4 prompt confrontabili"
          : "Apri confronto libero"}
      >
        Confronta
      </button>
    </div>
  {/if}
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

  /* Issue #140: bottoni icon-only quadrati per occupare meno spazio
     orizzontale quando la colonna è stretta (era ~210px con 3 label,
     ora ~92px con 3 icone). */
  .chip-icona {
    display: inline-grid;
    place-items: center;
    width: 26px;
    height: 24px;
    padding: 0;
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

  .empty-link {
    color: var(--accent-private);
    text-decoration: none;
  }

  .empty-link:hover,
  .empty-link:focus-visible {
    text-decoration: underline;
  }

  .empty-link:focus-visible {
    outline: 2px solid var(--accent-private);
    outline-offset: 2px;
    border-radius: 2px;
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

  /* F5 PR-F: visual indicator selezione multipla per Diff libero */
  /* #403: card variante — rientro + connettore verso il prompt padre. */
  .card-wrap.variante {
    padding-left: var(--sp-5);
  }
  .variante-marca {
    position: absolute;
    left: var(--sp-2);
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-subtle);
    font-size: var(--fs-sm);
    line-height: 1;
    z-index: 1;
  }

  .card-wrap.selezionata-multi {
    background: var(--accent-team-soft);
    box-shadow: inset 3px 0 0 var(--accent-team);
  }

  .multi-toolbar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
  }

  .multi-info {
    flex: 1;
    font-size: var(--fs-sm);
    color: var(--text-default);
    font-weight: var(--fw-medium);
  }

  .btn-primary,
  .btn-secondary {
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .btn-primary {
    background: var(--accent-team);
    color: var(--accent-team-on);
    border: 0;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-team-strong);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
  }

  .btn-secondary:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
</style>
