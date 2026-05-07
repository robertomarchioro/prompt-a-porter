<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    Button,
    PannelloLinter,
    PannelloProviderConfig,
    Switch,
    Toast,
  } from "$lib/components";
  import HotkeyInput from "$lib/components/HotkeyInput.svelte";
  import {
    syncGetState,
    syncOnChange,
    syncOra,
    syncLogout,
    type SyncState,
  } from "$lib/sync";

  interface Preferenze {
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
  }

  interface VoceAudit {
    id: string;
    azione: string;
    tipo_entita: string;
    id_entita: string;
    metadati: string;
    avvenuto_a: string;
  }

  interface Props {
    onchiudi: () => void;
    onapriLogin: () => void;
  }

  let { onchiudi, onapriLogin }: Props = $props();

  type Sezione =
    | "account"
    | "sync"
    | "hotkey"
    | "aspetto"
    | "vault"
    | "ricerca"
    | "provider"
    | "linter"
    | "audit"
    | "lingua"
    | "info";

  const sezioni: { id: Sezione; etichetta: string; icona: string }[] = [
    { id: "account", etichetta: "Account", icona: "👤" },
    { id: "sync", etichetta: "Sincronizzazione", icona: "🔄" },
    { id: "hotkey", etichetta: "Scorciatoie", icona: "⌨" },
    { id: "aspetto", etichetta: "Aspetto", icona: "🎨" },
    { id: "vault", etichetta: "Vault", icona: "🔒" },
    { id: "ricerca", etichetta: "Ricerca semantica", icona: "🔎" },
    { id: "provider", etichetta: "Provider AI", icona: "🤖" },
    { id: "linter", etichetta: "Linter", icona: "✏️" },
    { id: "audit", etichetta: "Registro attività", icona: "📋" },
    { id: "lingua", etichetta: "Lingua", icona: "🌐" },
    { id: "info", etichetta: "Informazioni", icona: "ℹ" },
  ];

  let sezione = $state<Sezione>("aspetto");
  let prefs = $state<Preferenze>({
    profilo: "personale",
    hotkey: "Ctrl+Shift+P",
    tema: "dark",
    tono: "zinc",
    lingua: "it",
    onboarding_completato: true,
    crea_prompt_esempio: true,
    sync_server_url: "",
    sync_email: "",
    sync_token: "",
    sync_intervallo_sec: 60,
    sync_abilitato: false,
    ricerca_semantica_abilitata: false,
    ricerca_alpha: 0.5,
    idle_unload_secondi: 300,
  });

  let vaultPercorso = $state("");
  let vaultCifrato = $state(false);
  let erroreHotkey = $state("");
  let mostraCambioPassword = $state(false);
  let vecchiaPassword = $state("");
  let nuovaPassword = $state("");
  let confermaPassword = $state("");
  let errorePassword = $state("");
  let confermaElimina = $state(false);
  let toastVisibile = $state(false);
  let toastTesto = $state("");
  let syncState = $state<SyncState>(syncGetState());
  interface AuditPaginato {
    voci: VoceAudit[];
    totale: number;
    limite: number;
    offset: number;
  }

  let auditVoci = $state<VoceAudit[]>([]);
  let auditFiltro = $state<string | undefined>(undefined);
  let auditAzione = $state("");
  let auditTesto = $state("");
  let auditDa = $state("");
  let auditA = $state("");
  let auditOffset = $state(0);
  let auditTotale = $state(0);
  const auditLimite = 50;
  let auditCleanupGiorni = $state(365);
  let auditMostraConfermaCleanup = $state(false);
  let auditEsportazioneInCorso = $state(false);

  // ─── Ricerca semantica (Fase 3) ───
  type StatoEmbeddings =
    | { stato: "non_scaricato"; model_id: string; path_atteso: string }
    | { stato: "pronto"; model_id: string; path: string; size_mb: number }
    | { stato: "caricato"; model_id: string; dimensione: number };

  let embStatus = $state<StatoEmbeddings | null>(null);
  let embErrore = $state("");
  let embOperazione = $state<"" | "download" | "init" | "backfill">("");
  let embProgressDownload = $state<{
    file: string;
    bytes: number;
    total: number | null;
    indice_file: number;
    totale_file: number;
  } | null>(null);
  let embProgressBackfill = $state<{
    tipo: string;
    processati: number;
    totale_stima: number;
    ultimo_id: string;
  } | null>(null);
  let embEsitoBackfill = $state<{
    prompt_processati: number;
    tag_processati: number;
    errori: number;
  } | null>(null);
  let embUnlistenDownload: UnlistenFn | null = null;
  let embUnlistenBackfill: UnlistenFn | null = null;

  // Import/export vault (Fase 2 Step 4)
  let importModalita = $state<"skip" | "overwrite" | "rename">("skip");
  let importInCorso = $state(false);
  let importReport = $state<{ nuovi: number; aggiornati: number; conflitti: number; errori: string[] } | null>(null);
  let importErrore = $state("");
  let exportInCorso = $state(false);

  $effect(() => {
    caricaDati();
    syncOnChange(() => {
      syncState = syncGetState();
    });
  });

  $effect(() => {
    if (sezione === "audit") {
      caricaAudit();
    }
  });

  $effect(() => {
    if (sezione === "ricerca" && embStatus === null) {
      void caricaStatoEmbeddings();
    }
  });

  $effect(() => {
    // Setup listener per progress events una sola volta.
    let attivo = true;
    (async () => {
      const u1 = await listen<typeof embProgressDownload>(
        "embeddings:download:progress",
        (e) => {
          if (!attivo) return;
          embProgressDownload = e.payload;
        },
      );
      const u2 = await listen<typeof embProgressBackfill>(
        "embeddings:backfill:progress",
        (e) => {
          if (!attivo) return;
          embProgressBackfill = e.payload;
        },
      );
      embUnlistenDownload = u1;
      embUnlistenBackfill = u2;
    })();
    return () => {
      attivo = false;
      embUnlistenDownload?.();
      embUnlistenBackfill?.();
    };
  });

  async function caricaDati() {
    try {
      prefs = await invoke<Preferenze>("preferenze_carica");
    } catch {
      /* preferenze default */
    }
    try {
      vaultPercorso = await invoke<string>("vault_percorso");
    } catch {
      /* vault non disponibile */
    }
    try {
      vaultCifrato = await invoke<boolean>("vault_cifrato");
    } catch {
      /* vault non esiste */
    }
  }

  async function salvaPreferenze() {
    try {
      await invoke("preferenze_salva", { preferenze: prefs });
    } catch {
      /* errore salvataggio */
    }
  }

  // ─── Ricerca semantica (Fase 3) ───
  async function caricaStatoEmbeddings() {
    embErrore = "";
    try {
      embStatus = await invoke<StatoEmbeddings>("embeddings_status");
    } catch (e) {
      embErrore = String(e);
    }
  }

  async function scaricaModello() {
    embErrore = "";
    embOperazione = "download";
    embProgressDownload = null;
    try {
      embStatus = await invoke<StatoEmbeddings>("embeddings_download");
    } catch (e) {
      embErrore = `Download fallito: ${e}`;
    } finally {
      embOperazione = "";
    }
  }

  async function inizializzaSession() {
    embErrore = "";
    embOperazione = "init";
    try {
      embStatus = await invoke<StatoEmbeddings>("embeddings_init");
    } catch (e) {
      embErrore = `Init fallito: ${e}`;
    } finally {
      embOperazione = "";
    }
  }

  async function eseguiBackfill() {
    embErrore = "";
    embOperazione = "backfill";
    embProgressBackfill = null;
    embEsitoBackfill = null;
    try {
      embEsitoBackfill = await invoke<{
        prompt_processati: number;
        tag_processati: number;
        saltati_no_session: number;
        errori: number;
      }>("embeddings_backfill");
    } catch (e) {
      embErrore = `Backfill fallito: ${e}`;
    } finally {
      embOperazione = "";
    }
  }

  function alphaPreset(p: "lessicale" | "bilanciato" | "semantico") {
    const v = p === "lessicale" ? 0.0 : p === "bilanciato" ? 0.5 : 1.0;
    prefs.ricerca_alpha = v;
    void salvaPreferenze();
  }

  function aggiornaAlpha(e: Event) {
    const target = e.target as HTMLInputElement;
    prefs.ricerca_alpha = parseFloat(target.value);
    void salvaPreferenze();
  }

  async function toggleRicercaSemantica() {
    prefs.ricerca_semantica_abilitata = !prefs.ricerca_semantica_abilitata;
    await salvaPreferenze();
  }

  function cambiaTema(tema: string) {
    prefs.tema = tema;
    document.documentElement.setAttribute("data-theme", tema);
    salvaPreferenze();
  }

  function cambiaTono(tono: string) {
    prefs.tono = tono;
    document.documentElement.setAttribute("data-tone", tono);
    salvaPreferenze();
  }

  function cambiaLingua(lingua: string) {
    prefs.lingua = lingua;
    salvaPreferenze();
  }

  async function gestisciHotkey(combo: string) {
    try {
      await invoke("registra_hotkey", { combo });
      await salvaPreferenze();
      erroreHotkey = "";
      toast("Scorciatoia registrata");
    } catch (e) {
      erroreHotkey = `Impossibile registrare: ${e}`;
    }
  }

  async function cambiaPassword() {
    errorePassword = "";
    if (nuovaPassword !== confermaPassword) {
      errorePassword = "Le password non coincidono";
      return;
    }
    if (nuovaPassword.length < 8) {
      errorePassword = "Minimo 8 caratteri";
      return;
    }
    try {
      await invoke("vault_cambia_password", {
        passwordVecchia: vecchiaPassword,
        passwordNuova: nuovaPassword,
      });
      mostraCambioPassword = false;
      vecchiaPassword = "";
      nuovaPassword = "";
      confermaPassword = "";
      toast("Password cambiata");
    } catch {
      errorePassword = "Password attuale errata";
    }
  }

  async function eliminaVault() {
    try {
      await invoke("vault_elimina");
      window.location.reload();
    } catch {
      /* errore eliminazione */
    }
  }

  async function copiaPercorso() {
    await navigator.clipboard.writeText(vaultPercorso);
    toast("Percorso copiato");
  }

  function buildAuditFiltro() {
    return {
      da: auditDa ? `${auditDa}T00:00:00Z` : null,
      a: auditA ? `${auditA}T23:59:59Z` : null,
      user_id: null,
      azione_like: auditAzione || null,
      tipo_entita: auditFiltro ?? null,
      testo: auditTesto || null,
      limite: auditLimite,
      offset: auditOffset,
    };
  }

  async function caricaAudit() {
    try {
      const res = await invoke<AuditPaginato>("audit_query", {
        filtro: buildAuditFiltro(),
      });
      auditVoci = res.voci;
      auditTotale = res.totale;
    } catch {
      auditVoci = [];
      auditTotale = 0;
    }
  }

  function applicaFiltri() {
    auditOffset = 0;
    caricaAudit();
  }

  function resetFiltri() {
    auditFiltro = undefined;
    auditAzione = "";
    auditTesto = "";
    auditDa = "";
    auditA = "";
    auditOffset = 0;
    caricaAudit();
  }

  function paginaPrev() {
    if (auditOffset >= auditLimite) {
      auditOffset -= auditLimite;
      caricaAudit();
    }
  }

  function paginaNext() {
    if (auditOffset + auditLimite < auditTotale) {
      auditOffset += auditLimite;
      caricaAudit();
    }
  }

  async function esportaAudit() {
    auditEsportazioneInCorso = true;
    try {
      const csv = await invoke<string>("audit_export_csv", {
        filtro: { ...buildAuditFiltro(), limite: 50000, offset: 0 },
      });
      const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      const ts = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      a.download = `audit-export-${ts}.csv`;
      a.click();
      URL.revokeObjectURL(url);
      toastTesto = "Audit log esportato";
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 3000);
    } catch (e) {
      toastTesto = `Errore esportazione: ${String(e)}`;
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 4000);
    } finally {
      auditEsportazioneInCorso = false;
    }
  }

  async function esportaVaultJson() {
    exportInCorso = true;
    try {
      const json = await invoke<string>("vault_export_json");
      const blob = new Blob([json], { type: "application/json;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      const ts = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      a.download = `pap-vault-${ts}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toastTesto = "Vault esportato in JSON";
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 3000);
    } catch (e) {
      toastTesto = `Errore esportazione: ${String(e)}`;
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 4000);
    } finally {
      exportInCorso = false;
    }
  }

  function apriImport() {
    importReport = null;
    importErrore = "";
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      importInCorso = true;
      try {
        const text = await file.text();
        importReport = await invoke("vault_import_json", {
          json: text,
          modalita: importModalita,
        });
        toastTesto = `Import completato: ${importReport!.nuovi} nuovi, ${importReport!.aggiornati} aggiornati`;
        toastVisibile = true;
        setTimeout(() => (toastVisibile = false), 4000);
      } catch (e) {
        importErrore = String(e);
      } finally {
        importInCorso = false;
      }
    };
    input.click();
  }

  async function eseguiCleanup() {
    try {
      const eliminate = await invoke<number>("audit_cleanup_oltre_giorni", {
        giorni: auditCleanupGiorni,
      });
      toastTesto = `${eliminate} righe eliminate`;
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 3000);
      auditMostraConfermaCleanup = false;
      auditOffset = 0;
      await caricaAudit();
    } catch (e) {
      toastTesto = `Errore cleanup: ${String(e)}`;
      toastVisibile = true;
      setTimeout(() => (toastVisibile = false), 4000);
    }
  }

  const azioneLabel: Record<string, string> = {
    "prompt.creato": "Prompt creato",
    "prompt.aggiornato": "Prompt aggiornato",
    "prompt.eliminato": "Prompt eliminato",
    "prompt.preferito": "Preferito",
    "vault.creato": "Vault creato",
    "vault.sbloccato": "Vault sbloccato",
    "vault.bloccato": "Vault bloccato",
    "vault.password_cambiata": "Password cambiata",
    "sync.delta_applicato": "Sync applicato",
  };

  function toast(testo: string) {
    toastTesto = testo;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 2000);
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
  }}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="scrim"
  onmousedown={(e) => {
    if (e.target === e.currentTarget) onchiudi();
  }}
>
  <div
    class="modale"
    role="dialog"
    aria-modal="true"
    aria-label="Impostazioni"
  >
    <header class="modale-header">
      <h2>Impostazioni</h2>
      <Button variante="ghost" dimensione="sm" onclick={onchiudi}
        >✕</Button
      >
    </header>

    <div class="modale-body">
      <!-- ── Sidebar sezioni ── -->
      <nav class="imp-sidebar">
        {#each sezioni as s}
          <button
            class="imp-nav"
            class:imp-nav--attivo={sezione === s.id}
            onclick={() => (sezione = s.id)}
            type="button"
          >
            <span class="imp-nav-ico">{s.icona}</span>
            {s.etichetta}
          </button>
        {/each}
      </nav>

      <!-- ── Contenuto sezione ── -->
      <div class="imp-content">
        {#if sezione === "account"}
          <div class="sez">
            <h3 class="sez-titolo">Account</h3>
            <p class="sez-desc">Profilo e identità locale</p>
            <div class="sez-riga">
              <label class="sez-label">Profilo</label>
              <span class="sez-valore">
                {prefs.profilo === "personale"
                  ? "Personale"
                  : "Team"}
              </span>
            </div>
            <div class="sez-riga">
              <label class="sez-label">Workspace</label>
              <span class="sez-valore">Personale</span>
            </div>
          </div>
        {:else if sezione === "sync"}
          <div class="sez">
            <h3 class="sez-titolo">Sincronizzazione</h3>
            <p class="sez-desc">
              Condividi prompt con il team tramite server
            </p>

            {#if prefs.sync_abilitato && prefs.sync_token}
              <div class="sync-stato">
                <div class="sync-stato-row">
                  <span
                    class="dot"
                    class:dot-ok={syncState.stato === "idle"}
                    class:dot-sync={syncState.stato === "syncing"}
                    class:dot-err={syncState.stato === "error"}
                  ></span>
                  <span class="sync-stato-testo">
                    {#if syncState.stato === "idle"}
                      Connesso
                    {:else if syncState.stato === "syncing"}
                      Sincronizzazione…
                    {:else if syncState.stato === "error"}
                      Errore
                    {:else}
                      Offline
                    {/if}
                  </span>
                </div>
                {#if syncState.ultimoSync}
                  <span class="sync-meta">
                    Ultimo sync: {syncState.ultimoSync}
                  </span>
                {/if}
                {#if syncState.errore}
                  <p class="sez-errore">{syncState.errore}</p>
                {/if}
              </div>

              <div class="sez-riga">
                <label class="sez-label">Server</label>
                <code class="sync-url">{prefs.sync_server_url}</code>
              </div>
              <div class="sez-riga">
                <label class="sez-label">Account</label>
                <span class="sez-valore">{prefs.sync_email}</span>
              </div>

              <div class="sync-btns">
                <Button
                  dimensione="sm"
                  onclick={() => syncOra()}
                >
                  Sincronizza ora
                </Button>
                <Button
                  variante="ghost"
                  dimensione="sm"
                  onclick={async () => {
                    await syncLogout();
                    await caricaDati();
                    toast("Disconnesso dal server");
                  }}
                >
                  Disconnetti
                </Button>
              </div>
            {:else}
              <div class="sync-non-connesso">
                <svg
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path
                    d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"
                  />
                  <path d="M3 3v5h5" />
                  <path
                    d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"
                  />
                  <path d="M16 16h5v5" />
                </svg>
                <p>Non connesso a nessun server sync</p>
                <Button
                  dimensione="sm"
                  onclick={onapriLogin}
                >
                  Connetti server
                </Button>
              </div>
            {/if}
          </div>
        {:else if sezione === "hotkey"}
          <div class="sez">
            <h3 class="sez-titolo">Scorciatoie</h3>
            <p class="sez-desc">
              Scorciatoia globale per aprire la palette da qualsiasi
              applicazione
            </p>
            <div class="hotkey-wrap">
              <HotkeyInput
                bind:valore={prefs.hotkey}
                onchange={gestisciHotkey}
              />
            </div>
            {#if erroreHotkey}
              <p class="sez-errore">{erroreHotkey}</p>
            {/if}
          </div>
        {:else if sezione === "aspetto"}
          <div class="sez">
            <h3 class="sez-titolo">Aspetto</h3>
            <p class="sez-desc">
              Tema e tonalità dell'interfaccia
            </p>
            <div class="sez-campo">
              <label class="sez-label">Tema</label>
              <div class="seg-control">
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.tema === "dark"}
                  onclick={() => cambiaTema("dark")}
                  type="button">Scuro</button
                >
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.tema === "light"}
                  onclick={() => cambiaTema("light")}
                  type="button">Chiaro</button
                >
              </div>
            </div>
            <div class="sez-campo">
              <label class="sez-label">Tono</label>
              <div class="seg-control">
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.tono === "zinc"}
                  onclick={() => cambiaTono("zinc")}
                  type="button"
                >
                  <span class="tono-dot" style:background="#71717a"
                  ></span>
                  Zinc
                </button>
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.tono === "slate"}
                  onclick={() => cambiaTono("slate")}
                  type="button"
                >
                  <span class="tono-dot" style:background="#64748b"
                  ></span>
                  Slate
                </button>
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.tono === "stone"}
                  onclick={() => cambiaTono("stone")}
                  type="button"
                >
                  <span class="tono-dot" style:background="#78716c"
                  ></span>
                  Stone
                </button>
              </div>
            </div>
          </div>
        {:else if sezione === "vault"}
          <div class="sez">
            <h3 class="sez-titolo">Vault</h3>
            <p class="sez-desc">Gestione del database locale</p>
            <div class="sez-riga">
              <label class="sez-label">Percorso</label>
              <div class="vault-path">
                <code>{vaultPercorso}</code>
                <Button
                  variante="ghost"
                  dimensione="sm"
                  onclick={copiaPercorso}>Copia</Button
                >
              </div>
            </div>
            <div class="sez-riga">
              <label class="sez-label">Cifratura</label>
              <span class="sez-valore">
                {vaultCifrato ? "AES-256 (SQLCipher)" : "Non cifrato"}
              </span>
            </div>

            {#if vaultCifrato}
              <div class="sez-divider"></div>
              <div class="sez-campo">
                <Button
                  dimensione="sm"
                  onclick={() =>
                    (mostraCambioPassword = !mostraCambioPassword)}
                >
                  {mostraCambioPassword ? "Annulla" : "Cambia password"}
                </Button>
              </div>
              {#if mostraCambioPassword}
                <div class="pwd-form">
                  <input
                    type="password"
                    bind:value={vecchiaPassword}
                    placeholder="Password attuale"
                    class="pwd-input"
                  />
                  <input
                    type="password"
                    bind:value={nuovaPassword}
                    placeholder="Nuova password"
                    class="pwd-input"
                  />
                  <input
                    type="password"
                    bind:value={confermaPassword}
                    placeholder="Conferma nuova password"
                    class="pwd-input"
                  />
                  {#if errorePassword}
                    <p class="sez-errore">{errorePassword}</p>
                  {/if}
                  <Button
                    variante="primary"
                    dimensione="sm"
                    disabled={!vecchiaPassword ||
                      !nuovaPassword ||
                      !confermaPassword}
                    onclick={cambiaPassword}
                  >
                    Conferma cambio
                  </Button>
                </div>
              {/if}
            {/if}

            <div class="sez-divider"></div>
            <div class="sez-campo">
              <label class="sez-label sez-label--danger"
                >Zona pericolosa</label
              >
              <div class="export-import">
                <h4 class="sez-sub-titolo">Esporta / Importa</h4>
                <p class="sez-sub-desc">
                  Backup completo del vault in formato JSON portabile (vedi
                  <a href="https://github.com/robertomarchioro/prompt-a-porter/blob/main/docs/utente/formato-export-json.md" target="_blank" rel="noopener">schema export</a>).
                </p>
                <div class="export-import-azioni">
                  <Button
                    variante="ghost"
                    dimensione="sm"
                    onclick={esportaVaultJson}
                    disabled={exportInCorso}
                  >
                    {exportInCorso ? "Esportazione…" : "Esporta JSON"}
                  </Button>
                  <Button
                    variante="ghost"
                    dimensione="sm"
                    onclick={apriImport}
                    disabled={importInCorso}
                  >
                    {importInCorso ? "Import in corso…" : "Importa JSON…"}
                  </Button>
                </div>

                <div class="import-modalita">
                  <span class="import-modalita-label">Modalità conflitti:</span>
                  <div class="seg-control seg-control--small">
                    <button
                      type="button"
                      class="seg-btn"
                      class:seg-btn--attivo={importModalita === "skip"}
                      onclick={() => (importModalita = "skip")}
                    >Skip</button>
                    <button
                      type="button"
                      class="seg-btn"
                      class:seg-btn--attivo={importModalita === "overwrite"}
                      onclick={() => (importModalita = "overwrite")}
                    >Sovrascrivi</button>
                    <button
                      type="button"
                      class="seg-btn"
                      class:seg-btn--attivo={importModalita === "rename"}
                      onclick={() => (importModalita = "rename")}
                    >Rinomina</button>
                  </div>
                </div>

                {#if importReport}
                  <div class="import-report">
                    <strong>Risultato:</strong>
                    {importReport.nuovi} nuovi · {importReport.aggiornati} aggiornati ·
                    {importReport.conflitti} conflitti
                    {#if importReport.errori.length > 0}
                      · {importReport.errori.length} errori
                    {/if}
                  </div>
                {/if}
                {#if importErrore}
                  <div class="import-errore">{importErrore}</div>
                {/if}
              </div>

              {#if !confermaElimina}
                <Button
                  variante="danger"
                  dimensione="sm"
                  onclick={() => (confermaElimina = true)}
                >
                  Elimina vault
                </Button>
              {:else}
                <div class="danger-confirm">
                  <p class="danger-warn">
                    Questa azione è irreversibile. Tutti i prompt
                    verranno eliminati permanentemente.
                  </p>
                  <div class="danger-btns">
                    <Button
                      variante="ghost"
                      dimensione="sm"
                      onclick={() => (confermaElimina = false)}
                      >Annulla</Button
                    >
                    <Button
                      variante="danger"
                      dimensione="sm"
                      onclick={eliminaVault}
                    >
                      Conferma eliminazione
                    </Button>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {:else if sezione === "ricerca"}
          <div class="sez">
            <h3 class="sez-titolo">Ricerca semantica</h3>
            <p class="sez-desc">
              Trova prompt per significato (non solo per parole esatte) usando
              embedding locali. Tutto offline, niente cloud, modello scaricato
              al primo uso (~150 MB).
            </p>

            {#if embErrore}
              <div class="ric-errore">{embErrore}</div>
            {/if}

            <div class="ric-card">
              <div class="ric-card-head">
                <span class="ric-card-titolo">Modello embedding</span>
                {#if embStatus?.stato === "non_scaricato"}
                  <span class="ric-badge ric-badge--off">Non scaricato</span>
                {:else if embStatus?.stato === "pronto"}
                  <span class="ric-badge ric-badge--ok">Pronto su disco</span>
                {:else if embStatus?.stato === "caricato"}
                  <span class="ric-badge ric-badge--attivo">Caricato in memoria</span>
                {:else}
                  <span class="ric-badge">Sconosciuto</span>
                {/if}
              </div>
              <div class="ric-card-body">
                {#if embStatus?.stato === "non_scaricato"}
                  <p class="ric-info">
                    Il modello (~150 MB tra ONNX + tokenizer + libonnxruntime)
                    verrà scaricato al primo uso da HuggingFace e Microsoft.
                  </p>
                  <Button
                    variante="primary"
                    onclick={scaricaModello}
                    disabled={embOperazione !== ""}
                  >
                    {embOperazione === "download"
                      ? "Scarico…"
                      : "Scarica modello"}
                  </Button>
                {:else if embStatus?.stato === "pronto"}
                  <p class="ric-info">
                    Modello scaricato ({embStatus.size_mb} MB su disco). Per
                    usarlo serve caricarlo in memoria — operazione una sola
                    volta a sessione, poi resta disponibile.
                  </p>
                  <Button
                    variante="primary"
                    onclick={inizializzaSession}
                    disabled={embOperazione !== ""}
                  >
                    {embOperazione === "init"
                      ? "Inizializzazione…"
                      : "Inizializza"}
                  </Button>
                {:else if embStatus?.stato === "caricato"}
                  <p class="ric-info">
                    Pronto per la ricerca semantica. Output {embStatus.dimensione}
                    dimensioni per prompt.
                  </p>
                {/if}

                {#if embProgressDownload && embOperazione === "download"}
                  <div class="ric-progress">
                    <div class="ric-progress-label">
                      File {embProgressDownload.indice_file}/{embProgressDownload.totale_file}:
                      <code>{embProgressDownload.file}</code>
                    </div>
                    <div class="ric-progress-bar">
                      {#if embProgressDownload.total}
                        <div
                          class="ric-progress-fill"
                          style:width="{(embProgressDownload.bytes / embProgressDownload.total) * 100}%"
                        ></div>
                      {:else}
                        <div class="ric-progress-fill ric-progress-fill--ind"></div>
                      {/if}
                    </div>
                    <div class="ric-progress-text">
                      {(embProgressDownload.bytes / 1024 / 1024).toFixed(1)} MB
                      {#if embProgressDownload.total}
                        / {(embProgressDownload.total / 1024 / 1024).toFixed(1)}
                        MB
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            {#if embStatus?.stato === "caricato"}
              <div class="ric-card">
                <div class="ric-card-head">
                  <span class="ric-card-titolo">Backfill embedding esistenti</span>
                </div>
                <div class="ric-card-body">
                  <p class="ric-info">
                    Calcola embedding per i prompt e tag già presenti nel vault.
                    Idempotente: salta quelli già processati.
                  </p>
                  <Button
                    variante="ghost"
                    onclick={eseguiBackfill}
                    disabled={embOperazione !== ""}
                  >
                    {embOperazione === "backfill"
                      ? "Elaborazione…"
                      : "Avvia backfill"}
                  </Button>
                  {#if embProgressBackfill && embOperazione === "backfill"}
                    <div class="ric-progress">
                      <div class="ric-progress-label">
                        {embProgressBackfill.tipo}: {embProgressBackfill.processati}
                        / {embProgressBackfill.totale_stima}
                      </div>
                      <div class="ric-progress-bar">
                        <div
                          class="ric-progress-fill"
                          style:width="{embProgressBackfill.totale_stima > 0
                            ? (embProgressBackfill.processati /
                                embProgressBackfill.totale_stima) *
                              100
                            : 0}%"
                        ></div>
                      </div>
                    </div>
                  {/if}
                  {#if embEsitoBackfill}
                    <div class="ric-esito">
                      ✓ {embEsitoBackfill.prompt_processati} prompt + {embEsitoBackfill.tag_processati}
                      tag elaborati
                      {#if embEsitoBackfill.errori > 0}
                        ({embEsitoBackfill.errori} errori)
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>
            {/if}

            <div class="ric-card">
              <div class="ric-card-head">
                <span class="ric-card-titolo">Bilanciamento ricerca</span>
              </div>
              <div class="ric-card-body">
                <p class="ric-info">
                  Quando la ricerca semantica è attiva, combina match
                  lessicale (parole esatte) e match semantico (significato)
                  via Reciprocal Rank Fusion. Sposta il cursore per privilegiare
                  uno dei due.
                </p>
                <div class="ric-alpha">
                  <div class="ric-alpha-preset">
                    <button
                      class="seg-btn"
                      class:seg-btn--attivo={prefs.ricerca_alpha === 0}
                      onclick={() => alphaPreset("lessicale")}
                      type="button">Lessicale</button
                    >
                    <button
                      class="seg-btn"
                      class:seg-btn--attivo={prefs.ricerca_alpha === 0.5}
                      onclick={() => alphaPreset("bilanciato")}
                      type="button">Bilanciato</button
                    >
                    <button
                      class="seg-btn"
                      class:seg-btn--attivo={prefs.ricerca_alpha === 1}
                      onclick={() => alphaPreset("semantico")}
                      type="button">Semantico</button
                    >
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={prefs.ricerca_alpha}
                    oninput={aggiornaAlpha}
                    class="ric-alpha-slider"
                  />
                  <div class="ric-alpha-valore">
                    α = {prefs.ricerca_alpha.toFixed(2)}
                  </div>
                </div>
                <div class="ric-toggle">
                  <Switch
                    attivo={prefs.ricerca_semantica_abilitata}
                    onchange={toggleRicercaSemantica}
                  />
                  <span>Usa ricerca semantica nelle query</span>
                </div>

                <div class="ric-idle-unload">
                  <label for="idle-unload-select">
                    Scarica modello dopo inattività
                  </label>
                  <select
                    id="idle-unload-select"
                    bind:value={prefs.idle_unload_secondi}
                    onchange={salvaPreferenze}
                  >
                    <option value={0}>Mai (resta sempre caricato)</option>
                    <option value={300}>5 minuti (consigliato)</option>
                    <option value={600}>10 minuti</option>
                    <option value={1800}>30 minuti</option>
                    <option value={3600}>1 ora</option>
                  </select>
                  <p class="sez-hint">
                    Libera ~150 MB di RAM quando il modello non è in
                    uso. Dopo lo scarico, la ricerca cade su FTS
                    lessicale fino al prossimo riavvio del client (il
                    riload automatico arriva in una versione successiva).
                  </p>
                </div>
              </div>
            </div>
          </div>
        {:else if sezione === "provider"}
          <div class="sez">
            <PannelloProviderConfig />
          </div>
        {:else if sezione === "linter"}
          <div class="sez">
            <PannelloLinter />
          </div>
        {:else if sezione === "audit"}
          <div class="sez">
            <h3 class="sez-titolo">Registro attività</h3>
            <p class="sez-desc">
              Cronologia delle operazioni eseguite nel vault
            </p>

            <div class="audit-filtri">
              <div class="audit-filtri-riga">
                <div class="seg-control">
                  <button
                    class="seg-btn"
                    class:seg-btn--attivo={auditFiltro === undefined}
                    onclick={() => { auditFiltro = undefined; applicaFiltri(); }}
                    type="button">Tutte</button
                  >
                  <button
                    class="seg-btn"
                    class:seg-btn--attivo={auditFiltro === "Prompt"}
                    onclick={() => { auditFiltro = "Prompt"; applicaFiltri(); }}
                    type="button">Prompt</button
                  >
                  <button
                    class="seg-btn"
                    class:seg-btn--attivo={auditFiltro === "Vault"}
                    onclick={() => { auditFiltro = "Vault"; applicaFiltri(); }}
                    type="button">Vault</button
                  >
                  <button
                    class="seg-btn"
                    class:seg-btn--attivo={auditFiltro === "Sync"}
                    onclick={() => { auditFiltro = "Sync"; applicaFiltri(); }}
                    type="button">Sync</button
                  >
                </div>
              </div>

              <div class="audit-filtri-riga">
                <input
                  type="text"
                  class="audit-input"
                  placeholder="Filtra per azione (es. creato, eliminato)"
                  bind:value={auditAzione}
                  onkeydown={(e) => e.key === "Enter" && applicaFiltri()}
                />
                <input
                  type="text"
                  class="audit-input"
                  placeholder="Cerca testo libero"
                  bind:value={auditTesto}
                  onkeydown={(e) => e.key === "Enter" && applicaFiltri()}
                />
              </div>

              <div class="audit-filtri-riga">
                <label class="audit-data-label">
                  <span>Da</span>
                  <input
                    type="date"
                    class="audit-input audit-input--data"
                    bind:value={auditDa}
                  />
                </label>
                <label class="audit-data-label">
                  <span>A</span>
                  <input
                    type="date"
                    class="audit-input audit-input--data"
                    bind:value={auditA}
                  />
                </label>
                <Button variante="primary" dimensione="sm" onclick={applicaFiltri}>
                  Applica filtri
                </Button>
                <Button variante="ghost" dimensione="sm" onclick={resetFiltri}>
                  Reset
                </Button>
                <Button
                  variante="ghost"
                  dimensione="sm"
                  onclick={esportaAudit}
                  disabled={auditEsportazioneInCorso}
                >
                  {auditEsportazioneInCorso ? "Esportazione…" : "Esporta CSV"}
                </Button>
              </div>
            </div>

            {#if auditTotale > 0}
              <div class="audit-paginazione">
                <span class="audit-paginazione-info">
                  {auditOffset + 1}–{Math.min(auditOffset + auditLimite, auditTotale)} di {auditTotale}
                </span>
                <div class="audit-paginazione-azioni">
                  <Button
                    variante="ghost"
                    dimensione="sm"
                    onclick={paginaPrev}
                    disabled={auditOffset === 0}
                  >
                    ← Precedente
                  </Button>
                  <Button
                    variante="ghost"
                    dimensione="sm"
                    onclick={paginaNext}
                    disabled={auditOffset + auditLimite >= auditTotale}
                  >
                    Successiva →
                  </Button>
                </div>
              </div>
            {/if}

            {#if auditVoci.length === 0}
              <div class="audit-vuoto">
                <p>Nessuna attività trovata</p>
              </div>
            {:else}
              <div class="audit-lista">
                {#each auditVoci as voce (voce.id)}
                  <div class="audit-riga">
                    <div class="audit-info">
                      <span class="audit-azione">
                        {azioneLabel[voce.azione] ?? voce.azione}
                      </span>
                      {#if voce.metadati}
                        <span class="audit-meta">{voce.metadati}</span>
                      {/if}
                    </div>
                    <div class="audit-dx">
                      <span class="audit-tipo">{voce.tipo_entita}</span>
                      <span class="audit-data">{voce.avvenuto_a}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

            <div class="audit-cleanup">
              <h4 class="audit-cleanup-titolo">Pulizia periodica</h4>
              <p class="audit-cleanup-desc">
                Cancella le voci più vecchie di N giorni. Operazione manuale, non recuperabile.
              </p>
              {#if !auditMostraConfermaCleanup}
                <div class="audit-cleanup-form">
                  <input
                    type="number"
                    min="30"
                    max="3650"
                    step="30"
                    class="audit-input audit-input--num"
                    bind:value={auditCleanupGiorni}
                  />
                  <span class="audit-cleanup-suffix">giorni</span>
                  <Button
                    variante="ghost"
                    dimensione="sm"
                    onclick={() => (auditMostraConfermaCleanup = true)}
                  >
                    Pulisci ora
                  </Button>
                </div>
              {:else}
                <div class="danger-confirm">
                  <p class="danger-warn">
                    Stai per eliminare tutte le voci più vecchie di {auditCleanupGiorni} giorni.
                    L'operazione è irreversibile.
                  </p>
                  <div class="danger-btns">
                    <Button
                      variante="ghost"
                      dimensione="sm"
                      onclick={() => (auditMostraConfermaCleanup = false)}
                    >Annulla</Button>
                    <Button
                      variante="danger"
                      dimensione="sm"
                      onclick={eseguiCleanup}
                    >
                      Conferma pulizia
                    </Button>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {:else if sezione === "lingua"}
          <div class="sez">
            <h3 class="sez-titolo">Lingua</h3>
            <p class="sez-desc">Lingua dell'interfaccia</p>
            <div class="sez-campo">
              <div class="seg-control">
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.lingua === "it"}
                  onclick={() => cambiaLingua("it")}
                  type="button">Italiano</button
                >
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={prefs.lingua === "en"}
                  onclick={() => cambiaLingua("en")}
                  type="button">English</button
                >
              </div>
              <p class="sez-hint">
                Il cambio lingua sarà effettivo al prossimo avvio.
              </p>
            </div>
          </div>
        {:else if sezione === "info"}
          <div class="sez">
            <h3 class="sez-titolo">Informazioni</h3>
            <p class="sez-desc">Prompt a Porter</p>
            <div class="info-grid">
              <span class="info-label">Versione</span>
              <span class="info-val">0.1.0 — Fase 1</span>
              <span class="info-label">Framework</span>
              <span class="info-val">Tauri 2 + Svelte 5</span>
              <span class="info-label">Database</span>
              <span class="info-val">SQLite + SQLCipher</span>
              <span class="info-label">Licenza</span>
              <span class="info-val">GPL 2.0</span>
            </div>
            <div class="sez-divider"></div>
            <p class="info-credits">
              Sviluppato con cura per gestire prompt AI in modo sicuro
              e locale.
            </p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<Toast variante="success" visibile={toastVisibile}>
  ✓ {toastTesto}
</Toast>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
  }

  .modale {
    display: flex;
    flex-direction: column;
    width: min(800px, 96vw);
    height: min(600px, 90vh);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .modale-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-4) var(--sp-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .modale-header h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .modale-body {
    flex: 1;
    display: grid;
    grid-template-columns: 200px 1fr;
    overflow: hidden;
  }

  /* ── Sidebar ── */

  .imp-sidebar {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-3);
    border-right: 1px solid var(--border-subtle);
    overflow-y: auto;
  }

  .imp-nav {
    appearance: none;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-default);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: background var(--motion-fast);
  }

  .imp-nav:hover {
    background: var(--bg-overlay);
  }

  .imp-nav--attivo {
    background: var(--bg-overlay);
    color: var(--text-strong);
    font-weight: var(--fw-medium);
  }

  .imp-nav-ico {
    font-size: 14px;
    width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  /* ── Contenuto ── */

  .imp-content {
    padding: var(--sp-5) var(--sp-6, var(--sp-5));
    overflow-y: auto;
  }

  .sez {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .sez-titolo {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .sez-desc {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    margin-top: calc(-1 * var(--sp-2));
  }

  .sez-campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .sez-riga {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
  }

  .sez-label {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-weight: var(--fw-medium);
  }

  .sez-label--danger {
    color: var(--danger);
  }

  .sez-valore {
    font-size: var(--fs-sm);
    color: var(--text-default);
  }

  .sez-errore {
    font-size: var(--fs-xs);
    color: var(--danger);
    margin: 0;
  }

  .sez-hint {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin: 0;
  }

  .sez-divider {
    height: 1px;
    background: var(--border-subtle);
  }

  /* ── Segmented control ── */

  .seg-control {
    display: flex;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    max-width: 320px;
  }

  .seg-btn {
    appearance: none;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-muted);
    background: var(--bg-input);
    border: none;
    cursor: pointer;
    transition: all var(--motion-fast);
  }
  .seg-btn + .seg-btn {
    border-left: 1px solid var(--border-default);
  }
  .seg-btn--attivo {
    color: var(--text-strong);
    background: var(--bg-overlay);
    font-weight: var(--fw-medium);
  }

  .tono-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* ── Hotkey ── */

  .hotkey-wrap {
    max-width: 360px;
  }

  /* ── Vault ── */

  .vault-path {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-width: 0;
  }

  .vault-path code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 300px;
  }

  .pwd-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    max-width: 320px;
  }

  .pwd-input {
    height: 36px;
    padding: 0 var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    outline: none;
    transition: border-color var(--motion-fast);
  }
  .pwd-input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }
  .pwd-input::placeholder {
    color: var(--text-subtle);
  }

  .danger-confirm {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-3);
    background: color-mix(in oklch, var(--danger) 8%, transparent);
    border: 1px solid color-mix(in oklch, var(--danger) 30%, transparent);
    border-radius: var(--radius-md);
  }

  .danger-warn {
    font-size: var(--fs-sm);
    color: var(--danger);
    margin: 0;
  }

  .danger-btns {
    display: flex;
    gap: var(--sp-2);
  }

  /* ── Sync ── */

  .sync-stato {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .sync-stato-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot-ok { background: var(--success); }
  .dot-sync { background: var(--warning); }
  .dot-err { background: var(--danger); }

  .sync-stato-testo {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
  }

  .sync-meta {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .sync-url {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 260px;
  }

  .sync-btns {
    display: flex;
    gap: var(--sp-2);
  }

  .sync-non-connesso {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-6);
    text-align: center;
    color: var(--text-subtle);
  }

  .sync-non-connesso p {
    margin: 0;
    font-size: var(--fs-sm);
    max-width: 30ch;
  }

  /* ── Audit ── */

  .audit-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
  }

  .audit-lista {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    max-height: 360px;
    overflow-y: auto;
  }

  .audit-riga {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface);
  }

  .audit-info {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-width: 0;
    flex: 1;
  }

  .audit-azione {
    font-size: var(--fs-sm);
    color: var(--text-strong);
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }

  .audit-meta {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .audit-dx {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    flex-shrink: 0;
  }

  .audit-tipo {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .audit-data {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    white-space: nowrap;
  }

  .audit-vuoto {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-6);
    color: var(--text-subtle);
    font-size: var(--fs-sm);
  }

  .audit-vuoto p {
    margin: 0;
  }

  .audit-filtri {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    margin-bottom: var(--sp-3);
  }

  .audit-filtri-riga {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }

  .audit-input {
    flex: 1;
    min-width: 0;
    padding: 6px 10px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .audit-input:focus-visible {
    outline: 2px solid var(--accent-team);
    outline-offset: 1px;
  }

  .audit-input--data {
    flex: 0 0 auto;
    width: 160px;
  }

  .audit-input--num {
    flex: 0 0 auto;
    width: 100px;
  }

  .audit-data-label {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .audit-paginazione {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-2) 0;
    border-bottom: 1px solid var(--border-subtle);
    margin-bottom: var(--sp-2);
  }

  .audit-paginazione-info {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .audit-paginazione-azioni {
    display: flex;
    gap: var(--sp-1);
  }

  .audit-cleanup {
    margin-top: var(--sp-5);
    padding: var(--sp-3);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .audit-cleanup-titolo {
    margin: 0 0 var(--sp-1);
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .audit-cleanup-desc {
    margin: 0 0 var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    line-height: var(--lh-relaxed);
  }

  .audit-cleanup-form {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .audit-cleanup-suffix {
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }

  .export-import {
    margin-top: var(--sp-4);
    padding: var(--sp-3);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    margin-bottom: var(--sp-4);
  }

  .sez-sub-titolo {
    margin: 0 0 var(--sp-1);
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .sez-sub-desc {
    margin: 0 0 var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    line-height: var(--lh-relaxed);
  }

  .sez-sub-desc a {
    color: var(--accent-team);
    text-decoration: none;
  }

  .sez-sub-desc a:hover {
    text-decoration: underline;
  }

  .export-import-azioni {
    display: flex;
    gap: var(--sp-2);
    margin-bottom: var(--sp-3);
  }

  .import-modalita {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }

  .import-modalita-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .seg-control--small .seg-btn {
    padding: 2px 8px;
    font-size: var(--fs-xs);
  }

  .import-report {
    padding: var(--sp-2);
    background: var(--accent-team-soft);
    border: 1px solid var(--accent-team);
    border-radius: var(--radius-sm);
    font-size: var(--fs-xs);
    color: var(--text-default);
  }

  .import-errore {
    padding: var(--sp-2);
    background: color-mix(in oklch, var(--danger) 15%, transparent);
    border: 1px solid var(--danger);
    border-radius: var(--radius-sm);
    font-size: var(--fs-xs);
    color: var(--text-strong);
  }

  /* ── Info ── */

  .info-grid {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: var(--sp-2) var(--sp-4);
  }

  .info-label {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .info-val {
    font-size: var(--fs-sm);
    color: var(--text-default);
  }

  .info-credits {
    font-size: var(--fs-sm);
    color: var(--text-subtle);
    margin: 0;
    font-style: italic;
  }

  /* ── Ricerca semantica (Fase 3) ── */
  .ric-errore {
    background: var(--accent-danger-soft, rgba(220, 80, 80, 0.15));
    color: var(--accent-danger, #c83);
    border: 1px solid var(--accent-danger, #c83);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    margin-bottom: var(--sp-3);
    font-size: var(--fs-sm);
  }

  .ric-card {
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--sp-4);
    margin-bottom: var(--sp-3);
  }

  .ric-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--sp-3);
  }

  .ric-card-titolo {
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .ric-badge {
    font-size: var(--fs-xs);
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    color: var(--text-muted);
  }
  .ric-badge--off {
    color: var(--text-subtle);
  }
  .ric-badge--ok {
    background: var(--accent-team-soft, rgba(80, 120, 200, 0.15));
    border-color: var(--accent-team);
    color: var(--accent-team);
  }
  .ric-badge--attivo {
    background: var(--accent-success-soft, rgba(80, 180, 120, 0.18));
    border-color: var(--accent-success, #5b8);
    color: var(--accent-success, #5b8);
  }

  .ric-card-body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .ric-info {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }

  .ric-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ric-progress-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }
  .ric-progress-label code {
    font-family: var(--font-mono);
    color: var(--text-strong);
  }
  .ric-progress-bar {
    height: 8px;
    background: var(--bg-canvas);
    border-radius: 4px;
    overflow: hidden;
  }
  .ric-progress-fill {
    height: 100%;
    background: var(--accent-team);
    border-radius: 4px;
    transition: width 200ms ease;
  }
  .ric-progress-fill--ind {
    width: 30% !important;
    animation: ric-indeterminate 1.5s ease-in-out infinite;
  }
  @keyframes ric-indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }
  .ric-progress-text {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-variant-numeric: tabular-nums;
  }

  .ric-esito {
    font-size: var(--fs-sm);
    color: var(--accent-success, #5b8);
    font-weight: var(--fw-semibold);
  }

  .ric-alpha {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .ric-alpha-preset {
    display: inline-flex;
    background: var(--bg-canvas);
    border-radius: var(--radius-sm);
    padding: 2px;
    width: fit-content;
  }

  .ric-alpha-slider {
    width: 100%;
    accent-color: var(--accent-team);
  }

  .ric-alpha-valore {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-variant-numeric: tabular-nums;
  }

  .ric-toggle {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border-subtle);
  }
</style>
