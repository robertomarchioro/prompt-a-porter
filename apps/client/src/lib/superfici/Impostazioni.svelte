<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, Toast } from "$lib/components";
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
    | "audit"
    | "lingua"
    | "info";

  const sezioni: { id: Sezione; etichetta: string; icona: string }[] = [
    { id: "account", etichetta: "Account", icona: "👤" },
    { id: "sync", etichetta: "Sincronizzazione", icona: "🔄" },
    { id: "hotkey", etichetta: "Scorciatoie", icona: "⌨" },
    { id: "aspetto", etichetta: "Aspetto", icona: "🎨" },
    { id: "vault", etichetta: "Vault", icona: "🔒" },
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
  let auditVoci = $state<VoceAudit[]>([]);
  let auditFiltro = $state<string | undefined>(undefined);

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

  async function caricaAudit() {
    try {
      auditVoci = await invoke<VoceAudit[]>("audit_lista", {
        limite: 200,
        tipoEntita: auditFiltro ?? null,
      });
    } catch {
      auditVoci = [];
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
        {:else if sezione === "audit"}
          <div class="sez">
            <h3 class="sez-titolo">Registro attività</h3>
            <p class="sez-desc">
              Cronologia delle operazioni eseguite nel vault
            </p>
            <div class="audit-toolbar">
              <div class="seg-control">
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={auditFiltro === undefined}
                  onclick={() => { auditFiltro = undefined; caricaAudit(); }}
                  type="button">Tutte</button
                >
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={auditFiltro === "Prompt"}
                  onclick={() => { auditFiltro = "Prompt"; caricaAudit(); }}
                  type="button">Prompt</button
                >
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={auditFiltro === "Vault"}
                  onclick={() => { auditFiltro = "Vault"; caricaAudit(); }}
                  type="button">Vault</button
                >
                <button
                  class="seg-btn"
                  class:seg-btn--attivo={auditFiltro === "Sync"}
                  onclick={() => { auditFiltro = "Sync"; caricaAudit(); }}
                  type="button">Sync</button
                >
              </div>
              <Button
                variante="ghost"
                dimensione="sm"
                onclick={caricaAudit}
              >
                Aggiorna
              </Button>
            </div>

            {#if auditVoci.length === 0}
              <div class="audit-vuoto">
                <p>Nessuna attività registrata</p>
              </div>
            {:else}
              <div class="audit-lista">
                {#each auditVoci as voce}
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
</style>
