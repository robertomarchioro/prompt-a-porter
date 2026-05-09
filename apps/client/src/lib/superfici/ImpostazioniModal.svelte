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
    Palette,
    List as ListIcon,
    Pencil,
    Lock,
    Sliders,
    Search,
    Check,
    Copy,
  } from "lucide-svelte";
  import Modale from "$lib/components/Modale.svelte";
  import {
    statoTema,
    salvaTemaTono,
  } from "$lib/stores/preferenze.svelte";
  import {
    caricaStato as caricaStatoLista,
    salvaStato as salvaStatoLista,
    type Densita,
    type StatoLista,
  } from "$lib/stores/densita";

  type SezioneId = "aspetto" | "vista" | "editor" | "sicurezza" | "avanzate";

  interface VoceSezione {
    id: SezioneId;
    label: string;
    keywords: string[];
  }

  interface Props {
    onChiudi: () => void;
    sezioneIniziale?: SezioneId;
  }

  let { onChiudi, sezioneIniziale = "aspetto" }: Props = $props();

  let sezione = $state<SezioneId>(sezioneIniziale);
  let query = $state("");

  const sezioni: VoceSezione[] = [
    {
      id: "aspetto",
      label: "Aspetto",
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
      keywords: ["editor", "autosave", "wrap", "wrapping", "tasti", "code"],
    },
    {
      id: "sicurezza",
      label: "Sicurezza",
      keywords: [
        "vault",
        "password",
        "master",
        "key",
        "lock",
        "blocca",
        "cifratura",
      ],
    },
    {
      id: "avanzate",
      label: "Avanzate",
      keywords: [
        "provider",
        "ai",
        "anthropic",
        "openai",
        "ollama",
        "gemini",
        "embeddings",
        "ricerca",
        "audit",
        "log",
        "sync",
        "hotkey",
        "scorciatoia",
      ],
    },
  ];

  const sezioniFiltrate = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sezioni;
    return sezioni.filter((s) => {
      if (s.label.toLowerCase().includes(q)) return true;
      return s.keywords.some((k) => k.includes(q));
    });
  });

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
      onChiudi();
    } catch (e) {
      console.error("[impostazioni] vault_lock", e);
    }
  }

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
      await invoke("vault_cambia_password", {
        vecchia: vecchiaPassword,
        nuova: nuovaPassword,
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
      errorePassword = String(e);
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
        <ul>
          {#each sezioniFiltrate as s (s.id)}
            <li>
              <button
                type="button"
                class:attiva={sezione === s.id}
                onclick={() => (sezione = s.id)}
              >
                {#if s.id === "aspetto"}<Palette size={14} />
                {:else if s.id === "vista"}<ListIcon size={14} />
                {:else if s.id === "editor"}<Pencil size={14} />
                {:else if s.id === "sicurezza"}<Lock size={14} />
                {:else}<Sliders size={14} />{/if}
                <span>{s.label}</span>
              </button>
            </li>
          {/each}
          {#if sezioniFiltrate.length === 0}
            <li class="vuoto-nav">Nessuna voce</li>
          {/if}
        </ul>
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
        <div class="placeholder-card">
          <strong>Configurazione editor</strong>
          <p>
            Le opzioni di editing (autosave delay, line wrapping, indent)
            saranno disponibili in una prossima release. Per ora l'editor
            usa i default.
          </p>
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
      {:else if sezione === "avanzate"}
        <h3>Avanzate</h3>
        <div class="placeholder-card">
          <strong>In arrivo (F8 PR-D2)</strong>
          <p>
            Sub-sezioni: Provider AI, Ricerca &amp; Embeddings, Audit log AI,
            Sync, Hotkey. Per ora resta accessibile la superficie legacy
            <em>Impostazioni</em>.
          </p>
        </div>
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

  nav ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
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

  .placeholder-card {
    padding: var(--sp-3);
    background: var(--bg-input);
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .placeholder-card strong {
    display: block;
    color: var(--text-strong);
    margin-bottom: 4px;
    font-size: var(--fs-md);
  }

  .placeholder-card p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
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
</style>
