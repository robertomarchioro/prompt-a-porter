<script lang="ts">
  /**
   * F8 PR-D1 — Modale Impostazioni (scaffold + 4 macro + Avanzate placeholder).
   *
   * Layout 2 colonne: nav sezioni (sx) + dettaglio (dx). ⌘K cerca filtra
   * le voci nav. Sub-sezioni Avanzate (Provider AI, Embeddings, Audit, Sync,
   * Hotkey) arrivano in PR-D2.
   *
   * Riferimenti:
   * - Blueprint: docs/roadmap/redesign-v08/blueprint-F8.md §4 (decisione #12)
   */
  import { invoke } from "@tauri-apps/api/core";
  import {
    enable as autostartEnable,
    disable as autostartDisable,
    isEnabled as autostartIsEnabled,
  } from "@tauri-apps/plugin-autostart";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy, untrack } from "svelte";
  import {
    Palette,
    List as ListIcon,
    Pencil,
    Lock,
    Search,
    Check,
    Copy,
    Bot,
    Sparkles,
    ScrollText,
    RefreshCw,
    Keyboard,
    ListChecks,
    Download,
    Globe,
    Plus,
    Trash2,
    FlaskConical,
    FolderOpen,
    FileDown,
    Eraser,
    CloudDownload,
    Database,
    Info,
    Power,
    HelpCircle,
  } from "lucide-svelte";
  import Modale from "$lib/components/Modale.svelte";
  import AiutoSezione from "$lib/aiuto/AiutoSezione.svelte";
  import AboutSezione from "$lib/aiuto/AboutSezione.svelte";
  import PannelloProviderConfig from "$lib/components/PannelloProviderConfig.svelte";
  import PannelloLinter from "$lib/components/PannelloLinter.svelte";
  import HotkeyInput from "$lib/components/HotkeyInput.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import { nomeFileExport, scaricaBlob } from "$lib/util/dati-export";
  import {
    statoTema,
    salvaTemaTono,
    statoEditor,
    salvaEditor,
    AUTOSAVE_DELAY_MIN,
    AUTOSAVE_DELAY_MAX,
    FONT_SIZE_MIN,
    FONT_SIZE_MAX,
  } from "$lib/stores/preferenze.svelte";
  import {
    caricaStato as caricaStatoLista,
    salvaStato as salvaStatoLista,
    type Densita,
    type StatoLista,
  } from "$lib/stores/densita";
  import {
    syncGetState,
    syncOnChange,
    syncOra,
    syncLogout,
    type SyncState,
  } from "$lib/sync";

  // Riordino sidebar: tutte le voci sono di primo livello (niente più
  // accordion "Avanzate") e raggruppate per dominio tramite `gruppo`.
  type SezioneId =
    | "aspetto"
    | "vista"
    | "editor"
    | "globali"
    | "linter"
    | "dati"
    | "provider"
    | "embeddings"
    | "audit"
    | "sicurezza"
    | "sync"
    | "sistema"
    | "hotkey"
    | "aggiornamenti"
    | "sviluppo"
    | "guida"
    | "info";

  type GruppoId =
    | "personalizzazione"
    | "contenuti"
    | "ai"
    | "sicurezza-sync"
    | "sistema"
    | "aiuto";

  interface VoceSezione {
    id: SezioneId;
    label: string;
    gruppo: GruppoId;
    keywords: string[];
  }

  interface Props {
    onChiudi: () => void;
    sezioneIniziale?: SezioneId;
  }

  let { onChiudi, sezioneIniziale = "aspetto" }: Props = $props();

  // Snapshot iniziale: il parent passa sezioneIniziale solo per aprire
  // la modale su una sezione specifica; dopo, il modale gestisce
  // internamente il routing tra sezioni. untrack evita che successive
  // mutazioni del prop sezioneIniziale resettino la navigazione utente.
  let sezione = $state<SezioneId>(untrack(() => sezioneIniziale));
  let query = $state("");

  // ─── M6 PR-4: sezione "Dati" import/export markdown ──────────────
  // Repo URL hardcoded per link doc utente (no env var per evitare
  // dipendenza build, valore stabile).
  const REPO_ORG_REPO = "robertomarchioro/prompt-a-porter";

  interface ImportRisultato {
    nomeFile: string;
    id: string;
    titolo: string;
  }

  interface ImportErrore {
    nomeFile: string;
    errore: string;
  }

  let inputFileMarkdown: HTMLInputElement | undefined = $state();

  let datiImport = $state<{
    inCorso: boolean;
    risultati: ImportRisultato[];
    errori: ImportErrore[];
  }>({
    inCorso: false,
    risultati: [],
    errori: [],
  });

  let datiExport = $state<{
    inCorso: boolean;
    ultimo:
      | {
          totale: number;
          byteCount: number;
          filename: string;
        }
      | null;
    errore: string;
  }>({
    inCorso: false,
    ultimo: null,
    errore: "",
  });

  // --- Import/Export JSON (backup round-trip lossless) ---

  type ModalitaImportJson = "skip" | "overwrite" | "rename";

  interface ImportReportJson {
    nuovi: number;
    aggiornati: number;
    conflitti: number;
    errori: string[];
  }

  let inputFileJson: HTMLInputElement | undefined = $state();

  let modalitaImportJson = $state<ModalitaImportJson>("skip");

  let datiImportJson = $state<{
    inCorso: boolean;
    report: ImportReportJson | null;
    errore: string;
  }>({
    inCorso: false,
    report: null,
    errore: "",
  });

  let datiExportJson = $state<{
    inCorso: boolean;
    ultimo: { byteCount: number; filename: string } | null;
    errore: string;
  }>({
    inCorso: false,
    ultimo: null,
    errore: "",
  });

  async function handleSelezioneMarkdown(e: Event): Promise<void> {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;
    datiImport = {
      inCorso: true,
      risultati: [],
      errori: [],
    };
    // Loop frontend: per ogni file leggi text via File API + invoke
    // prompt_import_markdown. La directory bulk via plugin-dialog non
    // e' installato; il pattern multi-file copre comunque l'80% dei
    // casi (Obsidian/Foam: user seleziona vault folder content).
    for (let i = 0; i < files.length; i++) {
      const f = files[i];
      try {
        const testo = await f.text();
        const id = await invoke<string>("prompt_import_markdown", {
          testo,
          nomeFile: f.name,
        });
        // Recupera titolo finale post-parse via libreria_dettaglio
        // (best-effort: se fallisce mostra il nome file come fallback).
        let titolo = f.name;
        try {
          const det = await invoke<{ titolo: string }>("libreria_dettaglio", {
            id,
          });
          titolo = det.titolo;
        } catch {
          /* fallback al nome file */
        }
        datiImport.risultati = [
          ...datiImport.risultati,
          { nomeFile: f.name, id, titolo },
        ];
      } catch (err) {
        datiImport.errori = [
          ...datiImport.errori,
          {
            nomeFile: f.name,
            errore: String(err).replace(/^Error: /, ""),
          },
        ];
      }
    }
    datiImport.inCorso = false;
    // Notifica lista per refresh Libreria
    window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    // Reset input per permettere ri-selezione stessi file (browser
    // ignora change su stesso valore).
    target.value = "";
  }

  async function esportaVaultZip(): Promise<void> {
    datiExport = { inCorso: true, ultimo: null, errore: "" };
    try {
      const res = await invoke<{
        bytes: number[];
        totale_esportati: number;
      }>("vault_export_markdown_zip", { folderId: null });
      const filename = nomeFileExport("zip", new Date().toISOString());
      const blob = new Blob([new Uint8Array(res.bytes)], {
        type: "application/zip",
      });
      scaricaBlob(blob, filename);
      datiExport.ultimo = {
        totale: res.totale_esportati,
        byteCount: res.bytes.length,
        filename,
      };
    } catch (err) {
      datiExport.errore = String(err).replace(/^Error: /, "");
    } finally {
      datiExport.inCorso = false;
    }
  }

  async function esportaVaultJson(): Promise<void> {
    datiExportJson = { inCorso: true, ultimo: null, errore: "" };
    try {
      const json = await invoke<string>("vault_export_json");
      const filename = nomeFileExport("json", new Date().toISOString());
      const blob = new Blob([json], { type: "application/json" });
      scaricaBlob(blob, filename);
      datiExportJson.ultimo = {
        byteCount: new TextEncoder().encode(json).length,
        filename,
      };
    } catch (err) {
      datiExportJson.errore = String(err).replace(/^Error: /, "");
    } finally {
      datiExportJson.inCorso = false;
    }
  }

  async function handleSelezioneJson(e: Event): Promise<void> {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;
    const file = files[0];
    datiImportJson = { inCorso: true, report: null, errore: "" };
    try {
      const json = await file.text();
      const report = await invoke<ImportReportJson>("vault_import_json", {
        json,
        modalita: modalitaImportJson,
      });
      datiImportJson.report = report;
      // Notifica lista per refresh Libreria
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
    } catch (err) {
      datiImportJson.errore = String(err).replace(/^Error: /, "");
    } finally {
      datiImportJson.inCorso = false;
      // Reset input per permettere ri-selezione dello stesso file.
      target.value = "";
    }
  }

  const sezioni: VoceSezione[] = [
    // ── Personalizzazione ──
    {
      id: "aspetto",
      label: "Aspetto",
      gruppo: "personalizzazione",
      keywords: [
        "tema",
        "dark",
        "light",
        "auto",
        "tono",
        "palette",
        "zinc",
        "slate",
        "stone",
        "colori",
      ],
    },
    {
      id: "vista",
      label: "Vista lista",
      gruppo: "personalizzazione",
      keywords: [
        "densità",
        "compatta",
        "comoda",
        "anteprima",
        "righe",
        "preview",
        "lista",
      ],
    },
    {
      id: "editor",
      label: "Editor",
      gruppo: "personalizzazione",
      keywords: [
        "editor",
        "autosave",
        "wrap",
        "wrapping",
        "tasti",
        "code",
        "font",
        "indent",
        "riga",
      ],
    },
    // ── Contenuti ──
    {
      id: "globali",
      label: "Segnaposti globali",
      gruppo: "contenuti",
      keywords: [
        "segnaposti",
        "globali",
        "placeholder",
        "default",
        "variabili",
        "globale",
        "autore",
      ],
    },
    {
      id: "linter",
      label: "Linter",
      gruppo: "contenuti",
      keywords: [
        "linter",
        "lint",
        "regole",
        "diagnosi",
        "avvisi",
        "warning",
        "pii",
        "segnaposti",
      ],
    },
    {
      id: "dati",
      label: "Dati",
      gruppo: "contenuti",
      keywords: [
        "import",
        "export",
        "importa",
        "esporta",
        "markdown",
        "md",
        "backup",
        "obsidian",
        "foam",
        "zip",
        "front-matter",
        "json",
      ],
    },
    // ── AI ──
    {
      id: "provider",
      label: "Provider AI",
      gruppo: "ai",
      keywords: [
        "provider",
        "ai",
        "anthropic",
        "openai",
        "ollama",
        "openai-compat",
        "gemini",
        "api key",
        "endpoint",
        "modello",
      ],
    },
    {
      id: "embeddings",
      label: "Ricerca & Embeddings",
      gruppo: "ai",
      keywords: [
        "ricerca",
        "semantica",
        "embeddings",
        "minilm",
        "alpha",
        "ibrida",
        "reindex",
        "vettore",
      ],
    },
    {
      id: "audit",
      label: "Audit log AI",
      gruppo: "ai",
      keywords: ["audit", "log", "csv", "export", "cleanup", "retention"],
    },
    // ── Sicurezza & Sync ──
    {
      id: "sicurezza",
      label: "Sicurezza",
      gruppo: "sicurezza-sync",
      keywords: [
        "vault",
        "password",
        "master",
        "key",
        "lock",
        "blocca",
        "cifratura",
        "elimina",
      ],
    },
    {
      id: "sync",
      label: "Sync",
      gruppo: "sicurezza-sync",
      keywords: ["sync", "sincronizza", "logout", "stato", "remoto"],
    },
    // ── Sistema ──
    {
      id: "sistema",
      label: "Sistema",
      gruppo: "sistema",
      keywords: [
        "sistema",
        "avvio",
        "avvia",
        "login",
        "boot",
        "startup",
        "automatico",
        "tray",
        "icona",
      ],
    },
    {
      id: "hotkey",
      label: "Hotkey",
      gruppo: "sistema",
      keywords: ["hotkey", "scorciatoia", "tasti", "palette", "ctrl", "shift"],
    },
    {
      id: "aggiornamenti",
      label: "Aggiornamenti",
      gruppo: "sistema",
      keywords: [
        "aggiornamenti",
        "update",
        "updater",
        "versione",
        "nuova",
        "github",
        "installa",
      ],
    },
    {
      id: "sviluppo",
      label: "Sviluppo",
      gruppo: "sistema",
      keywords: [
        "sviluppo",
        "debug",
        "log",
        "diagnostica",
        "telemetria",
        "bug",
        "issue",
        "esporta",
        "zip",
        "tracing",
        "beta",
      ],
    },
    // ── Aiuto ──
    {
      id: "guida",
      label: "Guida e aiuto",
      gruppo: "aiuto",
      keywords: [
        "guida",
        "aiuto",
        "help",
        "tutorial",
        "tour",
        "documentazione",
        "docs",
        "manuale",
        "scorciatoie",
        "faq",
      ],
    },
    {
      id: "info",
      label: "Informazioni",
      gruppo: "aiuto",
      keywords: [
        "informazioni",
        "about",
        "versione",
        "version",
        "licenza",
        "license",
        "agpl",
        "credits",
        "crediti",
        "codename",
        "repository",
        "github",
      ],
    },
  ];

  // Ordine e label dei gruppi nella sidebar.
  const GRUPPI: { id: GruppoId; label: string }[] = [
    { id: "personalizzazione", label: "Personalizzazione" },
    { id: "contenuti", label: "Contenuti" },
    { id: "ai", label: "AI" },
    { id: "sicurezza-sync", label: "Sicurezza & Sync" },
    { id: "sistema", label: "Sistema" },
    { id: "aiuto", label: "Aiuto" },
  ];

  const sezioniFiltrate = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sezioni;
    return sezioni.filter(
      (s) =>
        s.label.toLowerCase().includes(q) ||
        s.keywords.some((k) => k.includes(q)),
    );
  });

  // Sidebar raggruppata: gruppi senza voci che matchano la ricerca
  // vengono nascosti.
  const gruppiFiltrati = $derived(
    GRUPPI.map((g) => ({
      ...g,
      voci: sezioniFiltrate.filter((s) => s.gruppo === g.id),
    })).filter((g) => g.voci.length > 0),
  );

  $effect(() => {
    if (
      sezioniFiltrate.length > 0 &&
      !sezioniFiltrate.find((s) => s.id === sezione)
    ) {
      sezione = sezioniFiltrate[0].id;
    }
  });

  // ─── Aspetto ───
  function cambiaTema(nuovo: string): void {
    statoTema.tema = nuovo;
    void salvaTemaTono(nuovo, statoTema.tono);
  }

  function cambiaTono(nuovo: string): void {
    statoTema.tono = nuovo;
    void salvaTemaTono(statoTema.tema, nuovo);
  }

  // ─── Vista lista ───
  let statoVista = $state<StatoLista>(caricaStatoLista());

  function aggiornaDensita(d: Densita): void {
    statoVista = { ...statoVista, densita: d };
    salvaStatoLista(statoVista);
    notificaListaCambio();
  }

  function aggiornaRighePreview(n: number): void {
    statoVista = { ...statoVista, righePreview: n };
    salvaStatoLista(statoVista);
    notificaListaCambio();
  }

  function notificaListaCambio(): void {
    window.dispatchEvent(new CustomEvent("pap:lista-densita-cambiata"));
  }

  // ─── Sicurezza ───
  let vaultPath = $state<string>("");
  let copiato = $state(false);
  let mostraCambioPassword = $state(false);
  let vecchiaPassword = $state("");
  let nuovaPassword = $state("");
  let confermaPassword = $state("");
  let errorePassword = $state("");
  let statoOpPassword = $state<"" | "in_corso" | "ok">("");

  $effect(() => {
    void caricaVaultPath();
  });

  async function caricaVaultPath(): Promise<void> {
    try {
      vaultPath = await invoke<string>("vault_percorso");
    } catch (e) {
      console.error("[impostazioni] vault_percorso", e);
    }
  }

  async function copiaPath(): Promise<void> {
    if (!vaultPath) return;
    try {
      await navigator.clipboard.writeText(vaultPath);
      copiato = true;
      setTimeout(() => (copiato = false), 1500);
    } catch (e) {
      console.error("[impostazioni] copia path", e);
    }
  }

  async function bloccaVault(): Promise<void> {
    try {
      await invoke("vault_lock");
      // Chiudi la modale e riporta l'app allo stato "vault bloccato":
      // App.svelte ascolta `pap:vault-bloccato` e rimonta Onboarding,
      // che rileva il vault esistente+cifrato e mostra lo sblocco
      // (issue #273: prima il lock backend avveniva ma la UI restava
      // sulla Shell, dando l'impressione che il pulsante non facesse
      // nulla).
      onChiudi();
      window.dispatchEvent(new CustomEvent("pap:vault-bloccato"));
    } catch (e) {
      console.error("[impostazioni] vault_lock", e);
    }
  }

  // ─── Elimina vault (issue #274) ───
  // Azione distruttiva con doppia conferma: (1) toggle che rivela il
  // form, (2) l'utente deve digitare ELIMINA per abilitare il bottone.
  // Il backend `vault_elimina` chiude la connessione e cancella i file
  // su disco; poi riportiamo l'app a Onboarding (che, non trovando piu'
  // il vault, avvia il wizard di setup).
  const TESTO_CONFERMA_ELIMINA = "ELIMINA";
  let mostraEliminaVault = $state(false);
  let confermaEliminaTesto = $state("");
  let erroreElimina = $state("");
  let statoOpElimina = $state<"" | "in_corso">("");

  function annullaEliminaVault(): void {
    mostraEliminaVault = false;
    confermaEliminaTesto = "";
    erroreElimina = "";
  }

  async function eliminaVault(): Promise<void> {
    erroreElimina = "";
    if (confermaEliminaTesto !== TESTO_CONFERMA_ELIMINA) {
      erroreElimina = `Digita ${TESTO_CONFERMA_ELIMINA} per confermare.`;
      return;
    }
    statoOpElimina = "in_corso";
    try {
      await invoke("vault_elimina");
      onChiudi();
      // Stesso canale di routing del lock: App.svelte rimonta
      // Onboarding, che non trovando piu' meta+db avvia il setup.
      window.dispatchEvent(new CustomEvent("pap:vault-bloccato"));
    } catch (e) {
      statoOpElimina = "";
      erroreElimina = String(e).replace(/^Error: /, "");
    }
  }

  // ─── Avanzate: state ───
  type StatoEmbeddings =
    | { stato: "non_scaricato"; model_id: string; path_atteso: string }
    | { stato: "pronto"; model_id: string; path: string; size_mb: number }
    | { stato: "caricato"; model_id: string; dimensione: number };

  interface PreferenzeFull {
    profilo: string;
    hotkey: string;
    tema: string;
    tono: string;
    lingua: string;
    onboarding_completato: boolean;
    crea_prompt_esempio: boolean;
    sync_server_url: string;
    sync_email: string;
    sync_token: string;
    sync_intervallo_sec: number;
    sync_abilitato: boolean;
    ricerca_semantica_abilitata: boolean;
    ricerca_alpha: number;
    idle_unload_secondi: number;
    debug_log_abilitato: boolean;
    updater_abilitato: boolean;
  }

  let prefsFull = $state<PreferenzeFull | null>(null);
  let embStatus = $state<StatoEmbeddings | null>(null);
  let embErrore = $state("");
  let embOpInCorso = $state<"" | "download" | "init">("");
  let embProgressDownload = $state<{
    file: string;
    bytes: number;
    total: number | null;
  } | null>(null);
  let embUnlistenDownload: UnlistenFn | null = null;

  let auditCleanupGiorni = $state(365);
  let auditExportInCorso = $state(false);
  let auditCleanupInCorso = $state(false);
  let auditMessaggio = $state("");

  let syncState = $state<SyncState>(syncGetState());
  let syncOraInCorso = $state(false);

  let hotkeyValore = $state("Ctrl+Shift+P");
  let hotkeySalvata = $state(false);

  // ─── Sviluppo → Debug log (v0.8.7 PR-B) ───
  interface FileLog {
    name: string;
    size_bytes: number;
    modified_at: string;
  }
  interface InfoDebugLog {
    path_corrente: string;
    directory: string;
    files: FileLog[];
  }
  let debugInfo = $state<InfoDebugLog | null>(null);
  let debugErrore = $state("");
  let debugOpInCorso = $state<"" | "esporta" | "pulisci">("");
  let debugMessaggio = $state("");

  async function caricaDebugInfo(): Promise<void> {
    debugErrore = "";
    try {
      debugInfo = await invoke<InfoDebugLog>("debug_log_info");
    } catch (e) {
      debugErrore = String(e);
    }
  }

  async function toggleDebugLog(): Promise<void> {
    if (!prefsFull) return;
    const nuovo = !prefsFull.debug_log_abilitato;
    prefsFull = { ...prefsFull, debug_log_abilitato: nuovo };
    try {
      await invoke("preferenze_salva", { preferenze: prefsFull });
      await invoke("debug_log_imposta_livello", { abilitato: nuovo });
      debugMessaggio = nuovo
        ? "Debug log attivato. I prossimi eventi verranno scritti su file."
        : "Debug log disattivato.";
      setTimeout(() => (debugMessaggio = ""), 3000);
    } catch (e) {
      // Rollback locale in caso di errore
      prefsFull = { ...prefsFull, debug_log_abilitato: !nuovo };
      debugErrore = `Errore toggle debug log: ${String(e)}`;
    }
  }

  async function apriCartellaLog(): Promise<void> {
    debugErrore = "";
    try {
      await invoke("debug_log_apri_cartella");
    } catch (e) {
      debugErrore = `Apertura cartella fallita: ${String(e)}`;
    }
  }

  async function pulisciLog(): Promise<void> {
    if (!window.confirm("Pulire il file di log corrente?\n\nI file rotati (vecchi) NON verranno toccati.")) {
      return;
    }
    debugErrore = "";
    debugOpInCorso = "pulisci";
    try {
      await invoke("debug_log_pulisci");
      debugMessaggio = "File di log corrente svuotato.";
      setTimeout(() => (debugMessaggio = ""), 3000);
      await caricaDebugInfo();
    } catch (e) {
      debugErrore = `Pulizia fallita: ${String(e)}`;
    } finally {
      debugOpInCorso = "";
    }
  }

  async function esportaLogZip(): Promise<void> {
    debugErrore = "";
    debugOpInCorso = "esporta";
    try {
      const zipPath = await invoke<string>("debug_log_esporta_zip");
      debugMessaggio = `ZIP esportato: ${zipPath}`;
    } catch (e) {
      debugErrore = `Export ZIP fallito: ${String(e)}`;
    } finally {
      debugOpInCorso = "";
    }
  }

  function formattaBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(2)} MB`;
  }

  // ─── Sviluppo → Aggiornamenti (v1.0 M1.4b) ───
  // Wrapping di tauri-plugin-updater. Check on-demand, niente
  // auto-check al boot. Vedi docs/utente/auto-update.md per la policy
  // e docs/architettura/decisioni/authenticode-signing.md §M1.5 per
  // le garanzie di sicurezza.
  type StatoUpdater =
    | { kind: "idle" }
    | { kind: "checking" }
    | { kind: "no-update" }
    | { kind: "available"; version: string; date: string; notes: string }
    | { kind: "installing" }
    | { kind: "error"; message: string };

  let updaterStato = $state<StatoUpdater>({ kind: "idle" });

  async function verificaAggiornamenti(): Promise<void> {
    updaterStato = { kind: "checking" };
    try {
      // Import dinamico per evitare di caricare il plugin se non serve
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (!update) {
        updaterStato = { kind: "no-update" };
        return;
      }
      updaterStato = {
        kind: "available",
        version: update.version ?? "?",
        date: update.date ?? "",
        notes: update.body ?? "",
      };
    } catch (e) {
      updaterStato = { kind: "error", message: String(e) };
    }
  }

  async function installaAggiornamento(): Promise<void> {
    if (updaterStato.kind !== "available") return;
    const ok = window.confirm(
      `Installare la versione ${updaterStato.version}?\n\nL'app verrà chiusa e riavviata. I tuoi dati restano intatti.`,
    );
    if (!ok) return;
    updaterStato = { kind: "installing" };
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (!update) {
        updaterStato = {
          kind: "error",
          message: "Update non più disponibile (forse già installato?)",
        };
        return;
      }
      await update.downloadAndInstall();
      // Dopo install riuscito, Tauri Updater chiude l'app per riavviarla.
      // Se il control torna qui significa che download/install non hanno
      // applicato (caso edge), riporto a idle.
      updaterStato = { kind: "idle" };
    } catch (e) {
      updaterStato = { kind: "error", message: String(e) };
    }
  }

  async function toggleUpdaterAbilitato(): Promise<void> {
    if (!prefsFull) return;
    const nuovo = !prefsFull.updater_abilitato;
    prefsFull = { ...prefsFull, updater_abilitato: nuovo };
    try {
      await invoke("preferenze_salva", { preferenze: prefsFull });
      // Reset stato updater al toggle (caso: utente disabilita mentre
      // c'è un update.available aperto)
      if (!nuovo) updaterStato = { kind: "idle" };
    } catch (e) {
      // Rollback locale
      prefsFull = { ...prefsFull, updater_abilitato: !nuovo };
      updaterStato = { kind: "error", message: String(e) };
    }
  }

  // ─── Segnaposti globali (issue #159) ───
  interface PlaceholderGlobale {
    name: string;
    value: string;
    updated_at: string;
  }
  let globaliLista = $state<PlaceholderGlobale[] | null>(null);
  let globaliErrore = $state("");
  let globaleNuovoNome = $state("");
  let globaleNuovoValore = $state("");
  // Mappa per edit inline: name → valore corrente (potenzialmente dirty)
  let globaliEdit = $state<Record<string, string>>({});

  async function caricaGlobali(): Promise<void> {
    globaliErrore = "";
    try {
      const lista =
        await invoke<PlaceholderGlobale[]>("globale_placeholder_lista");
      globaliLista = lista;
      const edit: Record<string, string> = {};
      for (const g of lista) edit[g.name] = g.value;
      globaliEdit = edit;
    } catch (e) {
      globaliErrore = String(e).replace(/^Error: /, "");
    }
  }

  async function aggiungiGlobale(): Promise<void> {
    globaliErrore = "";
    const name = globaleNuovoNome.trim();
    if (!name) {
      globaliErrore = "Nome obbligatorio.";
      return;
    }
    if (!/^\w+$/.test(name)) {
      globaliErrore =
        "Il nome può contenere solo lettere, cifre e underscore.";
      return;
    }
    try {
      await invoke<void>("globale_placeholder_aggiorna", {
        dati: { name, value: globaleNuovoValore },
      });
      globaleNuovoNome = "";
      globaleNuovoValore = "";
      await caricaGlobali();
    } catch (e) {
      globaliErrore = String(e).replace(/^Error: /, "");
    }
  }

  async function salvaGlobale(name: string): Promise<void> {
    globaliErrore = "";
    try {
      await invoke<void>("globale_placeholder_aggiorna", {
        dati: { name, value: globaliEdit[name] ?? "" },
      });
      await caricaGlobali();
    } catch (e) {
      globaliErrore = String(e).replace(/^Error: /, "");
    }
  }

  async function eliminaGlobale(name: string): Promise<void> {
    globaliErrore = "";
    try {
      await invoke<void>("globale_placeholder_elimina", { name });
      await caricaGlobali();
    } catch (e) {
      globaliErrore = String(e).replace(/^Error: /, "");
    }
  }

  async function caricaPrefsFull(): Promise<void> {
    try {
      prefsFull = await invoke<PreferenzeFull>("preferenze_carica");
      hotkeyValore = prefsFull.hotkey || "Ctrl+Shift+P";
    } catch (e) {
      console.error("[impostazioni] preferenze_carica", e);
    }
  }

  async function caricaEmbStatus(): Promise<void> {
    embErrore = "";
    try {
      embStatus = await invoke<StatoEmbeddings>("embeddings_status");
    } catch (e) {
      embErrore = String(e).replace(/^Error: /, "");
    }
  }

  async function scaricaModello(): Promise<void> {
    embErrore = "";
    embOpInCorso = "download";
    try {
      await invoke("embeddings_download");
      await caricaEmbStatus();
    } catch (e) {
      embErrore = String(e).replace(/^Error: /, "");
    } finally {
      embOpInCorso = "";
      embProgressDownload = null;
    }
  }

  async function inizializzaModello(): Promise<void> {
    embErrore = "";
    embOpInCorso = "init";
    try {
      await invoke("embeddings_init");
      await caricaEmbStatus();
    } catch (e) {
      embErrore = String(e).replace(/^Error: /, "");
    } finally {
      embOpInCorso = "";
    }
  }

  async function aggiornaAlpha(nuovo: number): Promise<void> {
    if (!prefsFull) return;
    prefsFull = { ...prefsFull, ricerca_alpha: nuovo };
    try {
      await invoke("preferenze_salva", { preferenze: prefsFull });
    } catch (e) {
      console.error("[impostazioni] alpha", e);
    }
  }

  async function toggleRicercaSemantica(): Promise<void> {
    if (!prefsFull) return;
    prefsFull = {
      ...prefsFull,
      ricerca_semantica_abilitata: !prefsFull.ricerca_semantica_abilitata,
    };
    try {
      await invoke("preferenze_salva", { preferenze: prefsFull });
    } catch (e) {
      console.error("[impostazioni] toggle ricerca", e);
    }
  }

  async function esportaAudit(): Promise<void> {
    auditMessaggio = "";
    auditExportInCorso = true;
    try {
      const csv = await invoke<string>("audit_export_csv", { filtro: null });
      const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `audit-${new Date().toISOString().slice(0, 10)}.csv`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      auditMessaggio = "Esportato.";
    } catch (e) {
      auditMessaggio = `Errore: ${String(e)}`;
    } finally {
      auditExportInCorso = false;
    }
  }

  async function eseguiAuditCleanup(): Promise<void> {
    auditMessaggio = "";
    auditCleanupInCorso = true;
    try {
      const eliminate = await invoke<number>("audit_cleanup_oltre_giorni", {
        giorni: auditCleanupGiorni,
      });
      auditMessaggio = `Eliminate ${eliminate} voci più vecchie di ${auditCleanupGiorni} giorni.`;
    } catch (e) {
      auditMessaggio = `Errore: ${String(e)}`;
    } finally {
      auditCleanupInCorso = false;
    }
  }

  async function avviaSyncOra(): Promise<void> {
    syncOraInCorso = true;
    try {
      await syncOra();
    } finally {
      syncOraInCorso = false;
    }
  }

  async function eseguiSyncLogout(): Promise<void> {
    try {
      await syncLogout();
    } catch (e) {
      console.error("[impostazioni] sync logout", e);
    }
  }

  async function salvaHotkey(): Promise<void> {
    if (!prefsFull) return;
    prefsFull = { ...prefsFull, hotkey: hotkeyValore };
    try {
      await invoke("preferenze_salva", { preferenze: prefsFull });
      hotkeySalvata = true;
      setTimeout(() => (hotkeySalvata = false), 1500);
    } catch (e) {
      console.error("[impostazioni] hotkey", e);
    }
  }

  // prefsFull serve a embeddings (ricerca semantica/alpha), sviluppo
  // (toggle debug) e aggiornamenti (toggle updater): caricalo pigro
  // appena si entra in una di queste sezioni.
  $effect(() => {
    if (
      (sezione === "embeddings" ||
        sezione === "sviluppo" ||
        sezione === "aggiornamenti") &&
      prefsFull === null
    ) {
      void caricaPrefsFull();
    }
  });

  $effect(() => {
    if (sezione === "embeddings" && embStatus === null) {
      void caricaEmbStatus();
    }
  });

  $effect(() => {
    if (sezione === "globali" && globaliLista === null) {
      void caricaGlobali();
    }
  });

  $effect(() => {
    if (sezione === "sviluppo" && debugInfo === null) {
      void caricaDebugInfo();
    }
  });

  // ─── Sistema → Avvio automatico (autostart) ───
  // Lo stato on/off vive a livello di SO (gestito dal plugin); il toggle
  // riflette isEnabled(). In versione portable l'opzione è esclusa: il path
  // dell'exe non è stabile e la voce di autostart si romperebbe se l'utente
  // sposta la cartella.
  let avvioPortable = $state(false);
  let avvioAutomatico = $state(false);
  let avvioCaricato = $state(false);
  let avvioInCorso = $state(false);
  let avvioErrore = $state("");

  async function caricaStatoAvvio(): Promise<void> {
    avvioErrore = "";
    try {
      avvioPortable = await invoke<boolean>("app_is_portable");
      avvioAutomatico = avvioPortable ? false : await autostartIsEnabled();
    } catch (e) {
      avvioErrore = `Impossibile leggere lo stato dell'avvio automatico: ${String(e)}`;
    } finally {
      avvioCaricato = true;
    }
  }

  async function toggleAvvioAutomatico(): Promise<void> {
    if (avvioPortable || avvioInCorso) return;
    avvioInCorso = true;
    avvioErrore = "";
    try {
      if (avvioAutomatico) {
        await autostartDisable();
      } else {
        await autostartEnable();
      }
      avvioAutomatico = await autostartIsEnabled();
    } catch (e) {
      avvioErrore = `Errore nell'impostare l'avvio automatico: ${String(e)}`;
    } finally {
      avvioInCorso = false;
    }
  }

  $effect(() => {
    if (sezione === "sistema" && !avvioCaricato) {
      void caricaStatoAvvio();
    }
  });

  onMount(() => {
    syncOnChange(() => {
      syncState = syncGetState();
    });
    void (async () => {
      embUnlistenDownload = await listen<{
        file: string;
        bytes: number;
        total: number | null;
      }>("embeddings:download:progress", (e) => {
        embProgressDownload = e.payload;
      });
    })();
  });

  onDestroy(() => {
    embUnlistenDownload?.();
  });

  async function cambiaPassword(): Promise<void> {
    errorePassword = "";
    if (!vecchiaPassword || !nuovaPassword) {
      errorePassword = "Compila vecchia e nuova password.";
      return;
    }
    if (nuovaPassword.length < 8) {
      errorePassword =
        "La nuova password deve essere lunga almeno 8 caratteri.";
      return;
    }
    if (nuovaPassword !== confermaPassword) {
      errorePassword = "La conferma non corrisponde alla nuova password.";
      return;
    }
    statoOpPassword = "in_corso";
    try {
      // I nomi dei parametri devono combaciare con la firma del comando
      // Rust `vault_cambia_password(password_vecchia, password_nuova)`.
      // Tauri converte snake_case → camelCase, quindi servono
      // `passwordVecchia`/`passwordNuova` (non `vecchia`/`nuova`):
      // con i nomi sbagliati l'invoke fallisce sempre (issue #272).
      await invoke("vault_cambia_password", {
        passwordVecchia: vecchiaPassword,
        passwordNuova: nuovaPassword,
      });
      statoOpPassword = "ok";
      vecchiaPassword = "";
      nuovaPassword = "";
      confermaPassword = "";
      setTimeout(() => {
        statoOpPassword = "";
        mostraCambioPassword = false;
      }, 1500);
    } catch (e) {
      statoOpPassword = "";
      errorePassword = String(e).replace(/^Error: /, "");
    }
  }
</script>

<Modale titolo="Impostazioni" larghezza="xl" {onChiudi}>
  <div class="layout">
    <aside class="nav-pane">
      <div class="search-box">
        <Search size={14} />
        <input
          type="search"
          placeholder="Cerca…"
          bind:value={query}
          aria-label="Cerca impostazioni"
        />
      </div>
      <nav>
        {#each gruppiFiltrati as g (g.id)}
          <div class="nav-gruppo">{g.label}</div>
          <ul>
            {#each g.voci as s (s.id)}
              <li>
                <button
                  type="button"
                  class:attiva={sezione === s.id}
                  onclick={() => (sezione = s.id)}
                >
                  {#if s.id === "aspetto"}<Palette size={14} />
                  {:else if s.id === "vista"}<ListIcon size={14} />
                  {:else if s.id === "editor"}<Pencil size={14} />
                  {:else if s.id === "globali"}<Globe size={14} />
                  {:else if s.id === "linter"}<ListChecks size={14} />
                  {:else if s.id === "dati"}<Database size={14} />
                  {:else if s.id === "provider"}<Bot size={14} />
                  {:else if s.id === "embeddings"}<Sparkles size={14} />
                  {:else if s.id === "audit"}<ScrollText size={14} />
                  {:else if s.id === "sicurezza"}<Lock size={14} />
                  {:else if s.id === "sync"}<RefreshCw size={14} />
                  {:else if s.id === "sistema"}<Power size={14} />
                  {:else if s.id === "hotkey"}<Keyboard size={14} />
                  {:else if s.id === "aggiornamenti"}<CloudDownload size={14} />
                  {:else if s.id === "sviluppo"}<FlaskConical size={14} />
                  {:else if s.id === "guida"}<HelpCircle size={14} />
                  {:else if s.id === "info"}<Info size={14} />
                  {:else}<FlaskConical size={14} />{/if}
                  <span>{s.label}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/each}
        {#if gruppiFiltrati.length === 0}
          <p class="vuoto-nav">Nessuna voce</p>
        {/if}
      </nav>
    </aside>

    <section class="dettaglio">
      {#if sezione === "aspetto"}
        <h3>Aspetto</h3>
        <div class="campo">
          <span class="campo-label">Tema</span>
          <div class="seg-control" role="radiogroup" aria-label="Tema">
            {#each ["auto", "light", "dark"] as t (t)}
              <button
                type="button"
                role="radio"
                aria-checked={statoTema.tema === t}
                class:attivo={statoTema.tema === t}
                onclick={() => cambiaTema(t)}
              >
                {t === "auto" ? "Auto" : t === "light" ? "Chiaro" : "Scuro"}
              </button>
            {/each}
          </div>
          <p class="hint">
            "Auto" segue le impostazioni del sistema operativo.
          </p>
        </div>

        <div class="campo">
          <span class="campo-label">Tono palette</span>
          <div class="seg-control" role="radiogroup" aria-label="Tono palette">
            {#each ["zinc", "slate", "stone"] as t (t)}
              <button
                type="button"
                role="radio"
                aria-checked={statoTema.tono === t}
                class:attivo={statoTema.tono === t}
                onclick={() => cambiaTono(t)}
              >
                {t}
              </button>
            {/each}
          </div>
          <p class="hint">
            Variazione neutra dei grigi (effetto sottile sui background).
          </p>
        </div>
      {:else if sezione === "vista"}
        <h3>Vista lista</h3>
        <div class="campo">
          <span class="campo-label">Densità default</span>
          <div class="seg-control" role="radiogroup">
            {#each ["compatta", "comoda", "anteprima"] as d (d)}
              <button
                type="button"
                role="radio"
                aria-checked={statoVista.densita === d}
                class:attivo={statoVista.densita === d}
                onclick={() => aggiornaDensita(d as Densita)}
              >
                {d}
              </button>
            {/each}
          </div>
          <p class="hint">
            "Compatta" mostra solo titolo; "comoda" titolo + meta; "anteprima"
            include un estratto del body.
          </p>
        </div>

        <div
          class="campo"
          class:disabilitato={statoVista.densita !== "anteprima"}
        >
          <span class="campo-label">
            Righe preview
            <strong class="num">{statoVista.righePreview}</strong>
          </span>
          <input
            type="range"
            min="1"
            max="8"
            bind:value={statoVista.righePreview}
            onchange={() => aggiornaRighePreview(statoVista.righePreview)}
            disabled={statoVista.densita !== "anteprima"}
            aria-label="Righe preview"
          />
          <p class="hint">
            Numero di righe del body mostrate quando densità = anteprima.
          </p>
        </div>
      {:else if sezione === "editor"}
        <h3>Editor</h3>

        <div class="campo">
          <span class="campo-label">Autosave delay</span>
          <div class="slider-row">
            <input
              type="range"
              min={AUTOSAVE_DELAY_MIN}
              max={AUTOSAVE_DELAY_MAX}
              step="250"
              value={statoEditor.autosaveDelayMs}
              oninput={(e) =>
                void salvaEditor({
                  autosaveDelayMs: Number(
                    (e.currentTarget as HTMLInputElement).value,
                  ),
                })}
              aria-label="Ritardo autosave in millisecondi"
            />
            <span class="slider-value">{statoEditor.autosaveDelayMs} ms</span>
          </div>
          <p class="hint">
            Quanto attende l'editor dopo l'ultima modifica prima di salvare
            (debounce). Più basso = salvataggio più aggressivo.
          </p>
        </div>

        <div class="campo">
          <span class="campo-label">Line wrapping (soft wrap)</span>
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={statoEditor.lineWrapping}
              onchange={(e) =>
                void salvaEditor({
                  lineWrapping: (e.currentTarget as HTMLInputElement).checked,
                })}
            />
            <span>Manda a capo le righe lunghe</span>
          </label>
          <p class="hint">
            Quando attivo, le righe lunghe vanno a capo invece di richiedere
            scroll orizzontale. Disattiva se preferisci uno stile da editor di
            codice.
          </p>
        </div>

        <div class="campo">
          <span class="campo-label">Dimensione indent</span>
          <div class="radio-row">
            <label>
              <input
                type="radio"
                name="indent-size"
                value="2"
                checked={statoEditor.indentSize === 2}
                onchange={() => void salvaEditor({ indentSize: 2 })}
              />
              2 spazi
            </label>
            <label>
              <input
                type="radio"
                name="indent-size"
                value="4"
                checked={statoEditor.indentSize === 4}
                onchange={() => void salvaEditor({ indentSize: 4 })}
              />
              4 spazi
            </label>
          </div>
        </div>

        <div class="campo">
          <span class="campo-label">Dimensione font</span>
          <div class="slider-row">
            <input
              type="range"
              min={FONT_SIZE_MIN}
              max={FONT_SIZE_MAX}
              step="1"
              value={statoEditor.fontSize}
              oninput={(e) =>
                void salvaEditor({
                  fontSize: Number(
                    (e.currentTarget as HTMLInputElement).value,
                  ),
                })}
              aria-label="Dimensione font editor in pixel"
            />
            <span class="slider-value">{statoEditor.fontSize} px</span>
          </div>
        </div>

        <div class="campo">
          <span class="campo-label">Numeri di riga</span>
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={statoEditor.showLineNumbers}
              onchange={(e) =>
                void salvaEditor({
                  showLineNumbers: (e.currentTarget as HTMLInputElement)
                    .checked,
                })}
            />
            <span>Mostra i numeri di riga nel gutter</span>
          </label>
        </div>

        <div class="campo">
          <span class="campo-label">Evidenzia riga attiva</span>
          <label class="toggle-row">
            <input
              type="checkbox"
              checked={statoEditor.highlightActiveLine}
              onchange={(e) =>
                void salvaEditor({
                  highlightActiveLine: (e.currentTarget as HTMLInputElement)
                    .checked,
                })}
            />
            <span>Sfondo leggero sulla riga sotto il cursore</span>
          </label>
        </div>
      {:else if sezione === "sicurezza"}
        <h3>Sicurezza</h3>

        <div class="campo">
          <span class="campo-label">Posizione vault</span>
          <div class="path-row">
            <code class="path-code" title={vaultPath}>{vaultPath || "—"}</code>
            <button
              type="button"
              class="btn-ghost"
              onclick={copiaPath}
              disabled={!vaultPath}
              title="Copia percorso"
            >
              {#if copiato}<Check size={14} />{:else}<Copy size={14} />{/if}
            </button>
          </div>
          <p class="hint">
            Database SQLite cifrato (SQLCipher) salvato localmente.
          </p>
        </div>

        <div class="campo">
          <span class="campo-label">Blocca vault</span>
          <button type="button" class="btn-warn" onclick={bloccaVault}>
            Blocca ora
          </button>
          <p class="hint">
            Richiede la master password al prossimo accesso. La modale viene
            chiusa automaticamente.
          </p>
        </div>

        <div class="campo">
          <span class="campo-label">Master password</span>
          {#if !mostraCambioPassword}
            <button
              type="button"
              class="btn-ghost"
              onclick={() => (mostraCambioPassword = true)}
            >
              Cambia password
            </button>
          {:else}
            <div class="form-pwd">
              <input
                type="password"
                placeholder="Vecchia password"
                bind:value={vecchiaPassword}
                autocomplete="current-password"
              />
              <input
                type="password"
                placeholder="Nuova password (≥ 8 caratteri)"
                bind:value={nuovaPassword}
                autocomplete="new-password"
              />
              <input
                type="password"
                placeholder="Conferma nuova"
                bind:value={confermaPassword}
                autocomplete="new-password"
              />
              {#if errorePassword}
                <p class="msg-err">{errorePassword}</p>
              {/if}
              {#if statoOpPassword === "ok"}
                <p class="msg-ok">Password aggiornata.</p>
              {/if}
              <div class="riga-azioni">
                <button
                  type="button"
                  class="btn-primary"
                  onclick={cambiaPassword}
                  disabled={statoOpPassword === "in_corso"}
                >
                  {statoOpPassword === "in_corso" ? "Aggiorno…" : "Aggiorna"}
                </button>
                <button
                  type="button"
                  class="btn-ghost"
                  onclick={() => {
                    mostraCambioPassword = false;
                    vecchiaPassword = "";
                    nuovaPassword = "";
                    confermaPassword = "";
                    errorePassword = "";
                  }}
                >
                  Annulla
                </button>
              </div>
            </div>
          {/if}
        </div>

        <div class="campo campo-danger">
          <span class="campo-label">Elimina vault</span>
          {#if !mostraEliminaVault}
            <button
              type="button"
              class="btn-ghost btn-danger"
              onclick={() => (mostraEliminaVault = true)}
            >
              <Trash2 size={14} />
              Elimina vault…
            </button>
            <p class="hint">
              Cancella definitivamente il database cifrato e tutti i prompt.
              <strong>Operazione irreversibile.</strong> Esporta un backup
              dalla sezione "Dati" prima di procedere.
            </p>
          {:else}
            <div class="form-pwd">
              <p class="hint hint-danger">
                Questa azione elimina <strong>tutti</strong> i tuoi prompt e
                non è reversibile. Per confermare digita
                <code>{TESTO_CONFERMA_ELIMINA}</code> qui sotto.
              </p>
              <input
                type="text"
                placeholder="Digita {TESTO_CONFERMA_ELIMINA}"
                bind:value={confermaEliminaTesto}
                autocomplete="off"
                aria-label="Conferma eliminazione vault"
              />
              {#if erroreElimina}
                <p class="msg-err">{erroreElimina}</p>
              {/if}
              <div class="riga-azioni">
                <button
                  type="button"
                  class="btn-warn btn-elimina"
                  onclick={eliminaVault}
                  disabled={statoOpElimina === "in_corso" ||
                    confermaEliminaTesto !== TESTO_CONFERMA_ELIMINA}
                >
                  {statoOpElimina === "in_corso"
                    ? "Elimino…"
                    : "Elimina definitivamente"}
                </button>
                <button
                  type="button"
                  class="btn-ghost"
                  onclick={annullaEliminaVault}
                  disabled={statoOpElimina === "in_corso"}
                >
                  Annulla
                </button>
              </div>
            </div>
          {/if}
        </div>
      {:else if sezione === "dati"}
        <h3>Dati</h3>
        <p class="dati-intro">
          Due formati: <strong>Markdown</strong> (interoperabile con
          Obsidian/Foam, archivio zip di file <code>.md</code> con
          front-matter YAML) e <strong>JSON</strong> (backup round-trip
          completo del vault: storico versioni, tag, cartelle, fork). Vedi la
          <a
            href="https://github.com/{REPO_ORG_REPO}/blob/main/docs/utente/markdown-import-export.md"
            target="_blank"
            rel="noopener">guida Markdown</a
          >
          e il
          <a
            href="https://github.com/{REPO_ORG_REPO}/blob/main/docs/utente/formato-export-json.md"
            target="_blank"
            rel="noopener">formato JSON</a
          > per i dettagli.
        </p>

        <div class="dati-card">
          <header class="dati-card-h">
            <span class="dati-card-title">Importa Markdown</span>
            {#if datiImport.inCorso}
              <span class="dati-card-status">Importazione in corso…</span>
            {/if}
          </header>
          <p class="dati-card-desc">
            Seleziona uno o più file <code>.md</code>/<code>.markdown</code>. Per ogni file
            viene creato un nuovo prompt; front-matter <code>title</code>,
            <code>description</code>, <code>target_model</code>,
            <code>visibility</code> letti se presenti, altrimenti
            usati default.
          </p>
          <input
            type="file"
            accept=".md,.markdown,text/markdown"
            multiple
            bind:this={inputFileMarkdown}
            onchange={handleSelezioneMarkdown}
            style="display:none"
          />
          <button
            type="button"
            class="dati-btn"
            onclick={() => inputFileMarkdown?.click()}
            disabled={datiImport.inCorso}
          >
            Seleziona file…
          </button>
          {#if datiImport.risultati.length > 0 || datiImport.errori.length > 0}
            <div class="dati-report">
              {#if datiImport.risultati.length > 0}
                <p class="dati-report-ok">
                  ✓ {datiImport.risultati.length} prompt importati
                </p>
                <ul class="dati-report-list">
                  {#each datiImport.risultati as r (r.id)}
                    <li><code>{r.nomeFile}</code> → "{r.titolo}"</li>
                  {/each}
                </ul>
              {/if}
              {#if datiImport.errori.length > 0}
                <p class="dati-report-err">
                  ✗ {datiImport.errori.length} falliti
                </p>
                <ul class="dati-report-list">
                  {#each datiImport.errori as e (e.nomeFile)}
                    <li>
                      <code>{e.nomeFile}</code>: {e.errore}
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </div>

        <div class="dati-card">
          <header class="dati-card-h">
            <span class="dati-card-title">Esporta Vault → Zip Markdown</span>
            {#if datiExport.inCorso}
              <span class="dati-card-status">Esportazione in corso…</span>
            {/if}
          </header>
          <p class="dati-card-desc">
            Crea un archivio zip con tutti i prompt del vault come file
            <code>.md</code> con front-matter YAML. La struttura cartelle
            del vault viene preservata. Compatibile con Obsidian/Foam come
            backup leggibile.
          </p>
          <button
            type="button"
            class="dati-btn dati-btn-primary"
            onclick={esportaVaultZip}
            disabled={datiExport.inCorso}
          >
            {datiExport.inCorso ? "Esporto…" : "Esporta zip"}
          </button>
          {#if datiExport.ultimo}
            <p class="dati-report-ok">
              ✓ Esportati {datiExport.ultimo.totale} prompt
              ({(datiExport.ultimo.byteCount / 1024).toFixed(1)} KB) →
              <code>{datiExport.ultimo.filename}</code>
            </p>
          {/if}
          {#if datiExport.errore}
            <p class="dati-report-err">✗ {datiExport.errore}</p>
          {/if}
        </div>

        <div class="dati-card">
          <header class="dati-card-h">
            <span class="dati-card-title">Importa JSON</span>
            {#if datiImportJson.inCorso}
              <span class="dati-card-status">Importazione in corso…</span>
            {/if}
          </header>
          <p class="dati-card-desc">
            Seleziona un file <code>.json</code> esportato da Prompt a Porter
            (ripristina vault completo: prompt, versioni, tag, cartelle).
            Scegli come gestire i prompt già presenti con lo stesso ID.
          </p>
          <div class="campo">
            <span class="campo-label">Conflitti</span>
            <div
              class="seg-control"
              role="radiogroup"
              aria-label="Gestione conflitti import JSON"
            >
              {#each [{ v: "skip", l: "Salta esistenti" }, { v: "overwrite", l: "Sovrascrivi" }, { v: "rename", l: "Rinomina duplicati" }] as opt (opt.v)}
                <button
                  type="button"
                  role="radio"
                  aria-checked={modalitaImportJson === opt.v}
                  class:attivo={modalitaImportJson === opt.v}
                  disabled={datiImportJson.inCorso}
                  onclick={() =>
                    (modalitaImportJson = opt.v as ModalitaImportJson)}
                >
                  {opt.l}
                </button>
              {/each}
            </div>
          </div>
          <input
            type="file"
            accept=".json,application/json"
            bind:this={inputFileJson}
            onchange={handleSelezioneJson}
            style="display:none"
          />
          <button
            type="button"
            class="dati-btn"
            onclick={() => inputFileJson?.click()}
            disabled={datiImportJson.inCorso}
          >
            Seleziona file JSON…
          </button>
          {#if datiImportJson.report}
            <div class="dati-report">
              <p class="dati-report-ok">
                ✓ Import completato — {datiImportJson.report.nuovi} nuovi,
                {datiImportJson.report.aggiornati} aggiornati,
                {datiImportJson.report.conflitti} conflitti
              </p>
              {#if datiImportJson.report.errori.length > 0}
                <p class="dati-report-err">
                  ✗ {datiImportJson.report.errori.length} errori
                </p>
                <ul class="dati-report-list">
                  {#each datiImportJson.report.errori as err, i (i)}
                    <li>{err}</li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
          {#if datiImportJson.errore}
            <p class="dati-report-err">✗ {datiImportJson.errore}</p>
          {/if}
        </div>

        <div class="dati-card">
          <header class="dati-card-h">
            <span class="dati-card-title">Esporta Vault → JSON</span>
            {#if datiExportJson.inCorso}
              <span class="dati-card-status">Esportazione in corso…</span>
            {/if}
          </header>
          <p class="dati-card-desc">
            Salva un singolo file <code>.json</code> con l'intero vault in
            formato lossless (storico versioni, tag, cartelle, fork). Ideale
            come backup completo o per migrare tra installazioni.
          </p>
          <button
            type="button"
            class="dati-btn dati-btn-primary"
            onclick={esportaVaultJson}
            disabled={datiExportJson.inCorso}
          >
            {datiExportJson.inCorso ? "Esporto…" : "Esporta JSON"}
          </button>
          {#if datiExportJson.ultimo}
            <p class="dati-report-ok">
              ✓ Esportato ({(datiExportJson.ultimo.byteCount / 1024).toFixed(
                1,
              )} KB) →
              <code>{datiExportJson.ultimo.filename}</code>
            </p>
          {/if}
          {#if datiExportJson.errore}
            <p class="dati-report-err">✗ {datiExportJson.errore}</p>
          {/if}
        </div>
      {:else if sezione === "provider"}
        <h3>Provider AI</h3>
        <PannelloProviderConfig />
      {:else if sezione === "embeddings"}
        <h3>Ricerca &amp; Embeddings</h3>
        {#if embStatus === null && !embErrore}
          <p class="hint">Caricamento stato modello…</p>
        {:else if embErrore}
          <p class="msg-err">{embErrore}</p>
        {:else if embStatus}
          <div class="campo">
            <span class="campo-label">Stato modello (MiniLM)</span>
            {#if embStatus.stato === "non_scaricato"}
              <p class="hint">
                Modello non scaricato. Path atteso:
                <code>{embStatus.path_atteso}</code>
              </p>
              <button
                type="button"
                class="btn-primary"
                onclick={scaricaModello}
                disabled={embOpInCorso !== ""}
              >
                {embOpInCorso === "download"
                  ? "Scarico…"
                  : "Scarica modello"}
              </button>
              {#if embProgressDownload}
                <p class="hint">
                  {embProgressDownload.file}:
                  {embProgressDownload.bytes} /
                  {embProgressDownload.total ?? "?"} byte
                </p>
              {/if}
            {:else if embStatus.stato === "pronto"}
              <p class="hint">
                Pronto ({embStatus.size_mb} MB) — non ancora
                caricato in memoria.
              </p>
              <button
                type="button"
                class="btn-primary"
                onclick={inizializzaModello}
                disabled={embOpInCorso !== ""}
              >
                {embOpInCorso === "init"
                  ? "Inizializzo…"
                  : "Inizializza"}
              </button>
            {:else}
              <p class="hint">
                Caricato in memoria (dim {embStatus.dimensione}).
              </p>
            {/if}
          </div>
        {/if}

        {#if prefsFull}
          <div class="campo">
            <label class="campo-row">
              <span>Ricerca semantica abilitata</span>
              <input
                type="checkbox"
                checked={prefsFull.ricerca_semantica_abilitata}
                onchange={() => void toggleRicercaSemantica()}
              />
            </label>
            <p class="hint">
              Quando attiva, la ricerca usa anche embeddings
              vettoriali (richiede modello inizializzato).
            </p>
          </div>
          <div class="campo">
            <span class="campo-label">
              Hybrid alpha (lessicale ↔ semantico)
              <strong class="num">
                {prefsFull.ricerca_alpha.toFixed(2)}
              </strong>
            </span>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={prefsFull.ricerca_alpha}
              onchange={(e) =>
                void aggiornaAlpha(
                  parseFloat(
                    (e.currentTarget as HTMLInputElement).value,
                  ),
                )}
            />
            <p class="hint">
              0 = solo BM25 lessicale · 1 = solo coseno semantico ·
              0.5 bilanciato.
            </p>
          </div>
        {/if}
      {:else if sezione === "audit"}
        <h3>Audit log AI</h3>
        <div class="campo">
          <span class="campo-label">Esporta cronologia</span>
          <button
            type="button"
            class="btn-ghost"
            onclick={esportaAudit}
            disabled={auditExportInCorso}
          >
            <Download size={14} />
            <span>
              {auditExportInCorso ? "Esporto…" : "Esporta CSV"}
            </span>
          </button>
          <p class="hint">
            Tutte le voci audit del vault in CSV.
          </p>
        </div>
        <div class="campo">
          <span class="campo-label">Cleanup retention</span>
          <div class="riga-inline">
            <input
              type="number"
              min="1"
              max="3650"
              bind:value={auditCleanupGiorni}
              class="num-input"
              aria-label="Giorni retention"
            />
            <span class="hint">giorni</span>
            <button
              type="button"
              class="btn-warn"
              onclick={eseguiAuditCleanup}
              disabled={auditCleanupInCorso}
            >
              {auditCleanupInCorso ? "Eseguo…" : "Elimina più vecchi"}
            </button>
          </div>
          <p class="hint">
            Rimuove voci audit antecedenti al numero di giorni
            indicato. Operazione irreversibile.
          </p>
        </div>
        {#if auditMessaggio}
          <p
            class="msg-info"
            class:msg-err={auditMessaggio.startsWith("Errore")}
          >
            {auditMessaggio}
          </p>
        {/if}
      {:else if sezione === "sync"}
        <h3>Sync</h3>
        <div class="campo">
          <span class="campo-label">Stato</span>
          <p class="hint">
            <strong>{syncState.stato}</strong>
            {#if syncState.ultimoSync}
              · ultimo sync {syncState.ultimoSync.slice(0, 16)}
            {/if}
            {#if syncState.conflitti > 0}
              · {syncState.conflitti} conflitti
            {/if}
          </p>
          {#if syncState.errore}
            <p class="msg-err">{syncState.errore}</p>
          {/if}
        </div>
        <div class="campo">
          <div class="riga-azioni">
            <button
              type="button"
              class="btn-primary"
              onclick={avviaSyncOra}
              disabled={syncOraInCorso ||
                syncState.stato === "non_configurato"}
            >
              {syncOraInCorso ? "Sincronizzo…" : "Sincronizza ora"}
            </button>
            <button
              type="button"
              class="btn-ghost"
              onclick={eseguiSyncLogout}
              disabled={syncState.stato === "non_configurato"}
            >
              Logout
            </button>
          </div>
          {#if syncState.stato === "non_configurato"}
            <p class="hint">
              Sync non configurata. Per configurare login al server
              remoto usa la superficie legacy
              <em>Impostazioni → Sincronizzazione</em>.
              Configurazione redesign-first prevista in v0.9.
            </p>
          {/if}
        </div>
      {:else if sezione === "hotkey"}
        <h3>Hotkey</h3>
        <div class="campo">
          <span class="campo-label">Apri palette globale</span>
          <HotkeyInput bind:valore={hotkeyValore} />
          <div class="riga-azioni">
            <button
              type="button"
              class="btn-primary"
              onclick={salvaHotkey}
            >
              {hotkeySalvata ? "Salvato ✓" : "Salva"}
            </button>
          </div>
          <p class="hint">
            Registrato a livello sistema. La modifica diventa
            effettiva al prossimo riavvio dell'app.
          </p>
        </div>
      {:else if sezione === "linter"}
        <h3>Linter</h3>
        <PannelloLinter />
      {:else if sezione === "globali"}
        <h3>Segnaposti globali</h3>
        <div class="campo">
          <p class="hint">
            Definisci valori di default per segnaposti
            <code>{`{{global nome}}`}</code>. Quando un prompt
            usa un segnaposto globale, il suo valore viene
            pre-riempito automaticamente in Compila e
            l'eventuale modifica viene salvata come nuovo
            default.
          </p>
        </div>
        <div class="campo">
          <span class="campo-label">Aggiungi segnaposto</span>
          <div class="globale-form">
            <input
              type="text"
              class="num-input globale-nome"
              placeholder="nome (es. autore)"
              bind:value={globaleNuovoNome}
              aria-label="Nome segnaposto globale"
            />
            <input
              type="text"
              class="globale-valore"
              placeholder="valore di default"
              bind:value={globaleNuovoValore}
              aria-label="Valore segnaposto globale"
            />
            <button
              type="button"
              class="btn-primary"
              onclick={aggiungiGlobale}
            >
              <Plus size={14} />
              <span>Aggiungi</span>
            </button>
          </div>
        </div>
        {#if globaliErrore}
          <p class="msg-err">{globaliErrore}</p>
        {/if}
        <div class="campo">
          <span class="campo-label">
            Segnaposti definiti
            {#if globaliLista}
              <strong class="num">({globaliLista.length})</strong>
            {/if}
          </span>
          {#if globaliLista === null}
            <p class="hint">Caricamento…</p>
          {:else if globaliLista.length === 0}
            <p class="hint">
              Nessun segnaposto globale definito.
            </p>
          {:else}
            <div class="globali-tabella" role="table">
              {#each globaliLista as g (g.name)}
                <div class="globali-riga" role="row">
                  <code class="globali-nome" title={g.name}>
                    {`{{global ${g.name}}}`}
                  </code>
                  <input
                    type="text"
                    class="globali-input"
                    bind:value={globaliEdit[g.name]}
                    aria-label={`Valore di ${g.name}`}
                  />
                  <button
                    type="button"
                    class="btn-ghost"
                    onclick={() => salvaGlobale(g.name)}
                    disabled={globaliEdit[g.name] === g.value}
                    title="Salva valore"
                  >
                    <Check size={14} />
                  </button>
                  <button
                    type="button"
                    class="btn-ghost btn-danger"
                    onclick={() => eliminaGlobale(g.name)}
                    title="Elimina segnaposto globale"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if sezione === "sistema"}
        <h3>Sistema</h3>
        <p class="hint">Integrazione con il sistema operativo.</p>

        <div class="campo">
          <div class="sviluppo-card">
            <div class="sviluppo-card-h">
              <span class="campo-label">Avvia all'avvio del computer</span>
              <label class="sviluppo-toggle">
                <input
                  type="checkbox"
                  checked={avvioAutomatico}
                  onchange={() => void toggleAvvioAutomatico()}
                  disabled={avvioPortable || avvioInCorso || !avvioCaricato}
                  aria-label="Avvia Prompt a Porter all'avvio del computer"
                />
                <span>{avvioAutomatico ? "Attivo" : "Disattivo"}</span>
              </label>
            </div>
            {#if avvioPortable}
              <p class="hint">
                Non disponibile nella versione <strong>portable</strong>: il
                percorso dell'eseguibile non è stabile (se sposti la cartella,
                l'avvio automatico si romperebbe). Usa l'installer per questa
                funzione.
              </p>
            {:else}
              <p class="hint">
                Quando attivo, Prompt a Porter parte automaticamente al login e
                si avvia <strong>ridotto nel tray</strong> (icona accanto
                all'orologio): clicca l'icona per aprire la finestra.
              </p>
            {/if}
            {#if avvioErrore}
              <p class="msg-err">{avvioErrore}</p>
            {/if}
          </div>
        </div>
      {:else if sezione === "sviluppo"}
        <h3>Sviluppo</h3>
        <p class="hint">
          Strumenti di diagnostica e funzioni beta. Utili per indagare
          comportamenti anomali e fornire informazioni dettagliate quando
          apri un'issue su GitHub.
        </p>

        <div class="campo">
          <div class="sviluppo-card">
            <div class="sviluppo-card-h">
              <span class="campo-label">Debug log</span>
              <label class="sviluppo-toggle">
                <input
                  type="checkbox"
                  checked={prefsFull?.debug_log_abilitato ?? false}
                  onchange={() => void toggleDebugLog()}
                  disabled={prefsFull === null}
                  aria-label="Abilita debug log"
                />
                <span>
                  {prefsFull?.debug_log_abilitato ? "Attivo" : "Disattivo"}
                </span>
              </label>
            </div>
            <p class="hint">
              Quando attivo, l'app registra su file gli eventi critici
              (livello DEBUG). Quando disattivo, solo errori e warning.
              Modifica effettiva immediata, no riavvio.
            </p>

            {#if debugInfo}
              <div class="sviluppo-info">
                <div class="sviluppo-info-row">
                  <span class="sviluppo-info-label">Cartella log</span>
                  <code class="sviluppo-path" title={debugInfo.directory}>
                    {debugInfo.directory}
                  </code>
                </div>
                <div class="sviluppo-info-row">
                  <span class="sviluppo-info-label">File attivi</span>
                  <span class="sviluppo-info-val">
                    {debugInfo.files.length}
                    {#if debugInfo.files.length > 0}
                      ({formattaBytes(
                        debugInfo.files.reduce(
                          (a, f) => a + f.size_bytes,
                          0,
                        ),
                      )} totali)
                    {/if}
                  </span>
                </div>
              </div>

              {#if debugInfo.files.length > 0}
                <details class="sviluppo-elenco">
                  <summary>Mostra elenco file</summary>
                  <ul>
                    {#each debugInfo.files as f (f.name)}
                      <li>
                        <code>{f.name}</code>
                        <span class="sviluppo-meta">
                          {formattaBytes(f.size_bytes)}
                          {#if f.modified_at}
                            · {f.modified_at.slice(0, 16)}
                          {/if}
                        </span>
                      </li>
                    {/each}
                  </ul>
                </details>
              {/if}
            {:else if debugErrore}
              <p class="msg-err">{debugErrore}</p>
            {:else}
              <p class="hint">Caricamento info log…</p>
            {/if}

            <div class="riga-azioni">
              <button
                type="button"
                class="btn-ghost"
                onclick={apriCartellaLog}
                title="Apri la cartella nel file manager"
              >
                <FolderOpen size={14} />
                <span>Apri cartella</span>
              </button>
              <button
                type="button"
                class="btn-primary"
                onclick={esportaLogZip}
                disabled={debugOpInCorso !== ""}
                title="Crea uno ZIP con tutti i file di log + metadata per allegarlo a un'issue GitHub"
              >
                <FileDown size={14} />
                <span>
                  {debugOpInCorso === "esporta"
                    ? "Esporto…"
                    : "Esporta ZIP per issue"}
                </span>
              </button>
              <button
                type="button"
                class="btn-warn"
                onclick={pulisciLog}
                disabled={debugOpInCorso !== ""}
                title="Svuota il file di log corrente (i file rotati restano)"
              >
                <Eraser size={14} />
                <span>
                  {debugOpInCorso === "pulisci" ? "Pulisco…" : "Pulisci log"}
                </span>
              </button>
            </div>

            {#if debugMessaggio}
              <p class="msg-info">{debugMessaggio}</p>
            {/if}
            {#if debugErrore && debugInfo}
              <p class="msg-err">{debugErrore}</p>
            {/if}

            <details class="sviluppo-viewer">
              <summary>Visualizza log live</summary>
              <p class="hint">
                Mostra le ultime 200 righe del file con auto-refresh
                ogni 2 secondi. Filtra per livello o regex.
              </p>
              <LogViewer />
            </details>
          </div>
        </div>
      {:else if sezione === "aggiornamenti"}
        <h3>Aggiornamenti</h3>
        <!-- v1.0 M1.4b — card Aggiornamenti -->
        <div class="campo">
          <div class="sviluppo-card">
            <div class="sviluppo-card-h">
              <span class="campo-label">
                <CloudDownload size={14} />
                Aggiornamenti
              </span>
              <label class="sviluppo-toggle">
                <input
                  type="checkbox"
                  checked={prefsFull?.updater_abilitato ?? true}
                  onchange={() => void toggleUpdaterAbilitato()}
                  disabled={prefsFull === null}
                  aria-label="Abilita verifica aggiornamenti"
                />
                <span>
                  {prefsFull?.updater_abilitato === false
                    ? "Disabilitato"
                    : "Abilitato"}
                </span>
              </label>
            </div>
            <p class="hint">
              Quando abilitato, puoi verificare manualmente se è
              disponibile una nuova versione. L'app non contatta GitHub
              automaticamente all'avvio. Vedi
              <a
                href="https://github.com/robertomarchioro/prompt-a-porter/blob/main/docs/utente/auto-update.md"
                target="_blank"
                rel="noopener"
              >
                docs auto-update
              </a>
              per la policy completa.
            </p>

            {#if prefsFull?.updater_abilitato !== false}
              <div class="riga-azioni">
                <button
                  type="button"
                  class="btn-primary"
                  onclick={verificaAggiornamenti}
                  disabled={updaterStato.kind === "checking" ||
                    updaterStato.kind === "installing"}
                >
                  <CloudDownload size={14} />
                  <span>
                    {updaterStato.kind === "checking"
                      ? "Verifica in corso…"
                      : "Verifica aggiornamenti"}
                  </span>
                </button>
              </div>

              {#if updaterStato.kind === "no-update"}
                <p class="msg-ok">Sei aggiornato all'ultima versione.</p>
              {:else if updaterStato.kind === "available"}
                <div class="sviluppo-info">
                  <div class="sviluppo-info-row">
                    <span class="sviluppo-info-label">Versione</span>
                    <strong class="sviluppo-info-val">
                      {updaterStato.version}
                    </strong>
                  </div>
                  {#if updaterStato.date}
                    <div class="sviluppo-info-row">
                      <span class="sviluppo-info-label">Rilasciata</span>
                      <span class="sviluppo-info-val">
                        {updaterStato.date.slice(0, 10)}
                      </span>
                    </div>
                  {/if}
                  {#if updaterStato.notes}
                    <details class="sviluppo-elenco">
                      <summary>Note di rilascio</summary>
                      <pre class="updater-notes">{updaterStato.notes}</pre>
                    </details>
                  {/if}
                </div>
                <div class="riga-azioni">
                  <button
                    type="button"
                    class="btn-primary"
                    onclick={installaAggiornamento}
                  >
                    <CloudDownload size={14} />
                    <span>Installa e riavvia</span>
                  </button>
                </div>
              {:else if updaterStato.kind === "installing"}
                <p class="hint">
                  Download e installazione in corso… L'app si riavvierà
                  automaticamente a fine processo.
                </p>
              {:else if updaterStato.kind === "error"}
                <p class="msg-err">{updaterStato.message}</p>
              {/if}
            {/if}
          </div>
        </div>
      {:else if sezione === "guida"}
        <AiutoSezione vaiInfo={() => (sezione = "info")} />
      {:else if sezione === "info"}
        <AboutSezione />
      {/if}
    </section>
  </div>
</Modale>

<style>
  .layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: var(--sp-3);
    min-height: 480px;
  }

  .nav-pane {
    border-right: 1px solid var(--border-subtle);
    padding-right: var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .search-box input {
    flex: 1;
    border: 0;
    background: transparent;
    color: var(--text-default);
    font-size: var(--fs-sm);
    outline: none;
  }

  nav {
    display: flex;
    flex-direction: column;
  }

  nav ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* Intestazione di gruppo nella sidebar (Personalizzazione, AI, …). */
  .nav-gruppo {
    padding: var(--sp-2) var(--sp-2) 4px;
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .nav-gruppo:not(:first-child) {
    margin-top: var(--sp-2);
  }

  nav button {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    padding: 6px var(--sp-2);
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    text-align: left;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  nav button:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  nav button.attiva {
    background: var(--bg-input);
    color: var(--text-strong);
    font-weight: var(--fw-medium);
  }

  .vuoto-nav {
    padding: var(--sp-2);
    color: var(--text-muted);
    font-size: var(--fs-sm);
    font-style: italic;
  }

  .dettaglio {
    padding: 0 var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .dettaglio h3 {
    margin: 0 0 var(--sp-1) 0;
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .campo.disabilitato {
    opacity: 0.55;
  }

  /* M10 — controlli sezione Editor */
  .slider-row {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .slider-row input[type="range"] {
    flex: 1;
    accent-color: var(--accent-team);
  }

  .slider-value {
    font-variant-numeric: tabular-nums;
    font-size: var(--fs-sm);
    color: var(--text-strong);
    min-width: 60px;
    text-align: right;
  }

  .radio-row {
    display: flex;
    gap: var(--sp-3);
  }

  .radio-row label,
  .toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-sm);
    color: var(--text-default);
    cursor: pointer;
  }

  .campo-label {
    font-size: var(--fs-sm);
    color: var(--text-default);
    font-weight: var(--fw-medium);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .num {
    font-variant-numeric: tabular-nums;
    color: var(--text-strong);
  }

  .seg-control {
    display: inline-flex;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    overflow: hidden;
    align-self: flex-start;
  }

  .seg-control button {
    padding: 6px var(--sp-3);
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    cursor: pointer;
    border-right: 1px solid var(--border-subtle);
  }

  .seg-control button:last-child {
    border-right: 0;
  }

  .seg-control button:hover {
    background: var(--bg-overlay);
  }

  .seg-control button.attivo {
    background: var(--accent-team);
    color: var(--accent-team-on);
  }

  .hint {
    margin: 0;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  input[type="range"] {
    accent-color: var(--accent-team);
    max-width: 320px;
  }

  .path-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .path-code {
    flex: 1;
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .btn-ghost,
  .btn-primary,
  .btn-warn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 6px var(--sp-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    cursor: pointer;
    align-self: flex-start;
  }

  .btn-ghost {
    background: var(--bg-input);
    color: var(--text-default);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--bg-overlay);
  }

  .btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent-team);
    color: var(--accent-team-on);
    border-color: transparent;
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-warn {
    background: transparent;
    color: var(--accent-warning, var(--warning, #d2a85f));
    border-color: var(--accent-warning, var(--warning, #d2a85f));
  }

  .btn-warn:hover {
    background: var(--accent-warning, var(--warning, #d2a85f));
    color: var(--bg-canvas);
  }

  .form-pwd {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 360px;
  }

  .form-pwd input[type="password"] {
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
  }

  .msg-err {
    margin: 0;
    color: var(--accent-danger, #d9534f);
    font-size: var(--fs-xs);
  }

  .msg-ok {
    margin: 0;
    color: var(--accent-success, #6cb86c);
    font-size: var(--fs-xs);
  }

  .riga-azioni {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .campo-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
  }

  .num-input {
    width: 80px;
    padding: 4px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-variant-numeric: tabular-nums;
  }

  .riga-inline {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }

  .msg-info {
    margin: 0;
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  /* ── Issue #159: Segnaposti globali ── */
  .globale-form {
    display: grid;
    grid-template-columns: 180px 1fr auto;
    gap: 6px;
    align-items: center;
  }

  .globale-nome,
  .globale-valore,
  .globali-input {
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
  }

  .globale-nome {
    width: 100%;
    font-family: var(--font-mono);
  }

  .globali-tabella {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .globali-riga {
    display: grid;
    grid-template-columns: 220px 1fr auto auto;
    gap: 6px;
    align-items: center;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
  }

  .globali-riga:hover {
    background: var(--bg-overlay);
  }

  .globali-nome {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    background: var(--accent-team-soft);
    color: var(--accent-team-strong);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .btn-danger {
    color: var(--accent-danger, #d9534f);
  }

  .btn-danger:hover:not(:disabled) {
    background: var(--accent-danger, #d9534f);
    color: var(--bg-canvas);
  }

  /* ── Elimina vault (issue #274): danger zone ── */
  .campo-danger {
    margin-top: var(--sp-3);
    padding-top: var(--sp-3);
    border-top: 1px solid var(--border-subtle);
  }

  .form-pwd input[type="text"] {
    padding: 6px var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--accent-danger, #d9534f);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
  }

  .hint-danger {
    color: var(--accent-danger, #d9534f);
  }

  .btn-elimina {
    color: var(--accent-danger, #d9534f);
    border-color: var(--accent-danger, #d9534f);
  }

  .btn-elimina:hover:not(:disabled) {
    background: var(--accent-danger, #d9534f);
    color: var(--bg-canvas);
  }

  .btn-elimina:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ── v0.8.7 Sezione Sviluppo → Debug log ── */
  .sviluppo-card {
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .sviluppo-card-h {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
  }

  .sviluppo-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    user-select: none;
    font-size: var(--fs-sm);
    color: var(--text-default);
  }

  .sviluppo-toggle input[type="checkbox"] {
    accent-color: var(--accent-team);
    cursor: pointer;
  }

  .sviluppo-toggle input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }

  .sviluppo-info {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: var(--sp-2);
    background: var(--bg-canvas);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .sviluppo-info-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-xs);
  }

  .sviluppo-info-label {
    color: var(--text-muted);
    min-width: 100px;
  }

  .sviluppo-info-val {
    color: var(--text-default);
    font-variant-numeric: tabular-nums;
  }

  .sviluppo-path {
    flex: 1;
    font-family: var(--font-mono);
    color: var(--text-default);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sviluppo-elenco summary {
    cursor: pointer;
    font-size: var(--fs-xs);
    color: var(--text-muted);
    user-select: none;
  }

  .sviluppo-elenco ul {
    margin: 6px 0 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sviluppo-elenco li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: 2px 6px;
    background: var(--bg-canvas);
    border-radius: var(--radius-sm);
    font-size: var(--fs-xs);
  }

  .sviluppo-elenco code {
    font-family: var(--font-mono);
    color: var(--text-default);
  }

  .sviluppo-meta {
    color: var(--text-subtle);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
  }

  .sviluppo-viewer {
    margin-top: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
    padding-top: var(--sp-2);
  }

  .sviluppo-viewer summary {
    cursor: pointer;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    user-select: none;
    margin-bottom: 6px;
  }

  .updater-notes {
    max-height: 200px;
    overflow: auto;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: var(--sp-2);
    margin: 6px 0 0 0;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-default);
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* M6 PR-4: sezione Dati */
  .dati-intro {
    margin: 0 0 var(--sp-3) 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.6;
  }
  .dati-intro code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.9em;
  }
  .dati-intro a {
    color: var(--accent-team);
    text-decoration: underline;
  }

  .dati-card {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    margin-bottom: var(--sp-3);
  }
  .dati-card-h {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--sp-2);
  }
  .dati-card-title {
    font-weight: var(--fw-medium);
    color: var(--text-default);
  }
  .dati-card-status {
    font-size: var(--fs-xs);
    color: var(--accent-team);
  }
  .dati-card-desc {
    margin: 0 0 var(--sp-2) 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }
  .dati-card-desc code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.9em;
  }

  .dati-btn {
    padding: 6px 14px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    cursor: pointer;
  }
  .dati-btn:hover:not(:disabled) {
    background: var(--bg-surface);
  }
  .dati-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .dati-btn-primary {
    background: var(--accent-team);
    color: var(--accent-team-on, white);
    border-color: transparent;
  }
  .dati-btn-primary:hover:not(:disabled) {
    background: var(--accent-team-strong, var(--accent-team));
  }

  .dati-report {
    margin-top: var(--sp-2);
    padding: var(--sp-2);
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
  }
  .dati-report-ok {
    margin: 6px 0;
    color: var(--accent-success, #2c8a2c);
    font-weight: var(--fw-medium);
  }
  .dati-report-err {
    margin: 6px 0;
    color: var(--danger);
    font-weight: var(--fw-medium);
  }
  .dati-report-list {
    margin: 4px 0 0 0;
    padding-left: var(--sp-3);
    color: var(--text-muted);
    font-size: var(--fs-xs);
    list-style: disc;
  }
  .dati-report-list li {
    margin: 2px 0;
  }
  .dati-report-list code {
    font-family: var(--font-mono);
    background: transparent;
    padding: 0;
  }
</style>
