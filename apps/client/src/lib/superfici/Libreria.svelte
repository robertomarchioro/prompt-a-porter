<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, EmptyState, NavItem, Tag } from "$lib/components";
  import { estraiSegnaposti } from "$lib/template";
  import CompilatorePrompt from "./CompilatorePrompt.svelte";
  import EditorPrompt from "./EditorPrompt.svelte";
  import Impostazioni from "./Impostazioni.svelte";

  interface PromptCard {
    id: string;
    titolo: string;
    descrizione: string;
    visibilita: string;
    preferito: boolean;
    uso_count: number;
    aggiornato_a: string;
    tags: TagInfoFE[];
  }

  interface PromptDettaglio {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    target_model: string;
    preferito: boolean;
    uso_count: number;
    creato_a: string;
    aggiornato_a: string;
    ultimo_uso: string;
    tags: TagInfoFE[];
  }

  interface TagInfoFE {
    id: string;
    nome: string;
    colore: string;
  }

  interface ConteggiViste {
    tutti: number;
    preferiti: number;
    privati: number;
    team: number;
  }

  let stato = $state<"caricamento" | "blocco" | "aperto">("caricamento");
  let vistaCorrente = $state("recenti");
  let tagSelezionato = $state<string | null>(null);
  let ordine = $state("recente");
  let cercaTesto = $state("");
  let idSelezionato = $state<string | null>(null);

  let prompts = $state<PromptCard[]>([]);
  let promptDet = $state<PromptDettaglio | null>(null);
  let conteggi = $state<ConteggiViste>({
    tutti: 0,
    preferiti: 0,
    privati: 0,
    team: 0,
  });
  let tags = $state<TagInfoFE[]>([]);
  let hotkeyCombo = $state("Ctrl+Shift+P");
  let passwordInput = $state("");
  let erroreUnlock = $state("");
  let mostraEditor = $state(false);
  let editorKey = $state(0);
  let promptPerEditor = $state<PromptDettaglio | null>(null);
  let mostraCompilatore = $state(false);
  let compilatoreKey = $state(0);
  let mostraImpostazioni = $state(false);

  const titoloVista = $derived(
    vistaCorrente === "recenti"
      ? "Recenti"
      : vistaCorrente === "preferiti"
        ? "Preferiti"
        : vistaCorrente === "tutti"
          ? "Tutti i prompt"
          : vistaCorrente === "privati"
            ? "Privati"
            : vistaCorrente === "team"
              ? "Team"
              : "Prompts",
  );

  const conteggioVista = $derived(
    vistaCorrente === "preferiti"
      ? conteggi.preferiti
      : vistaCorrente === "privati"
        ? conteggi.privati
        : vistaCorrente === "team"
          ? conteggi.team
          : conteggi.tutti,
  );

  const segnaposti = $derived(
    promptDet ? estraiSegnaposti(promptDet.body) : [],
  );

  let timeoutCerca: ReturnType<typeof setTimeout>;

  $effect(() => {
    inizializza();
  });

  async function inizializza() {
    try {
      const esiste = await invoke<boolean>("vault_esiste");
      if (!esiste) {
        stato = "aperto";
        return;
      }
      const aperto = await invoke<boolean>("vault_aperto");
      if (aperto) {
        stato = "aperto";
        await caricaDati();
        return;
      }
      const cifrato = await invoke<boolean>("vault_cifrato");
      if (!cifrato) {
        await invoke("vault_unlock", { password: "" });
        stato = "aperto";
        await caricaDati();
      } else {
        stato = "blocco";
      }
    } catch {
      stato = "blocco";
    }
  }

  async function sblocca() {
    try {
      await invoke("vault_unlock", { password: passwordInput });
      stato = "aperto";
      erroreUnlock = "";
      await caricaDati();
    } catch {
      erroreUnlock = "Password errata";
    }
  }

  async function caricaDati() {
    try {
      const prefs = await invoke<{
        hotkey: string;
        tema: string;
        tono: string;
      }>("preferenze_carica");
      hotkeyCombo = prefs.hotkey;
      document.documentElement.setAttribute("data-theme", prefs.tema);
      document.documentElement.setAttribute("data-tone", prefs.tono);
    } catch {
      /* preferenze non ancora salvate */
    }
    await Promise.all([caricaConteggi(), caricaLista(), caricaTags()]);
  }

  async function caricaConteggi() {
    try {
      conteggi = await invoke<ConteggiViste>("libreria_conteggi");
    } catch {
      /* vault non aperto */
    }
  }

  async function caricaLista() {
    try {
      prompts = await invoke<PromptCard[]>("libreria_lista", {
        filtro: {
          vista: vistaCorrente,
          tag_id: tagSelezionato,
          cerca: cercaTesto || null,
          ordine,
        },
      });
    } catch {
      prompts = [];
    }
  }

  async function caricaTags() {
    try {
      tags = await invoke<TagInfoFE[]>("libreria_tag_lista");
    } catch {
      /* nessun tag */
    }
  }

  async function caricaDettaglio(id: string) {
    try {
      const det = await invoke<PromptDettaglio>("libreria_dettaglio", {
        id,
      });
      if (idSelezionato === id) promptDet = det;
    } catch {
      /* prompt non trovato */
    }
  }

  async function togglePreferito() {
    if (!promptDet) return;
    const nuovo = await invoke<boolean>("libreria_toggle_preferito", {
      id: promptDet.id,
    });
    promptDet.preferito = nuovo;
    caricaConteggi();
    caricaLista();
  }

  function cambiaVista(v: string) {
    vistaCorrente = v;
    tagSelezionato = null;
    idSelezionato = null;
    promptDet = null;
    caricaLista();
    caricaConteggi();
  }

  function cambiaTag(id: string) {
    tagSelezionato = tagSelezionato === id ? null : id;
    vistaCorrente = "tutti";
    idSelezionato = null;
    promptDet = null;
    caricaLista();
  }

  function gestisciCerca() {
    clearTimeout(timeoutCerca);
    timeoutCerca = setTimeout(() => caricaLista(), 250);
  }

  function selezionaPrompt(id: string) {
    idSelezionato = id;
    caricaDettaglio(id);
  }

  function tempoRelativo(iso: string): string {
    if (!iso) return "";
    const d = new Date(
      iso.includes("T") ? iso : iso.replace(" ", "T") + "Z",
    );
    if (isNaN(d.getTime())) return "";
    const diff = Date.now() - d.getTime();
    const min = Math.floor(diff / 60000);
    if (min < 1) return "ora";
    if (min < 60) return `${min}m fa`;
    const ore = Math.floor(min / 60);
    if (ore < 24) return `${ore}h fa`;
    const giorni = Math.floor(ore / 24);
    if (giorni === 1) return "ieri";
    if (giorni < 7) return `${giorni}g fa`;
    return d.toLocaleDateString("it-IT", {
      day: "numeric",
      month: "short",
    });
  }

  function renderPreview(body: string): string {
    const esc = body
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    return esc.replace(
      /\{\{\s*(\w+)\s*\}\}/g,
      (_, n) =>
        `<span class="ph"><span class="br">{{</span>${n}<span class="br">}}</span></span>`,
    );
  }
</script>

{#if stato === "caricamento"}
  <main class="libreria-full">
    <p class="muted">Caricamento…</p>
  </main>
{:else if stato === "blocco"}
  <main class="libreria-full">
    <div class="unlock">
      <svg
        class="unlock-icona"
        width="40"
        height="40"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
      </svg>
      <h2 class="unlock-titolo">Vault bloccato</h2>
      <p class="unlock-desc">
        Inserisci la password per accedere ai tuoi prompt
      </p>
      <form
        class="unlock-form"
        onsubmit={(e) => {
          e.preventDefault();
          sblocca();
        }}
      >
        <input
          class="unlock-input"
          type="password"
          bind:value={passwordInput}
          placeholder="Password del vault"
          autofocus
        />
        {#if erroreUnlock}
          <p class="unlock-errore">{erroreUnlock}</p>
        {/if}
        <Button variante="primary" type="submit">Sblocca</Button>
      </form>
    </div>
  </main>
{:else}
  <div class="libreria">
    <!-- ── Sidebar ── -->
    <aside class="sidebar">
      <div class="ws-switcher">
        <div class="ws-avatar">P</div>
        <span class="ws-nome">Personale</span>
      </div>

      <div class="sb-gruppo">
        <div class="sb-label">Viste</div>
        <NavItem
          attivo={vistaCorrente === "recenti" && !tagSelezionato}
          conteggio={conteggi.tutti}
          onclick={() => cambiaVista("recenti")}
        >
          {#snippet icona()}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><circle cx="12" cy="12" r="10" /><polyline
                points="12 6 12 12 16 14"
              /></svg
            >
          {/snippet}
          Recenti
        </NavItem>
        <NavItem
          attivo={vistaCorrente === "preferiti" && !tagSelezionato}
          conteggio={conteggi.preferiti}
          onclick={() => cambiaVista("preferiti")}
        >
          {#snippet icona()}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><polygon
                points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
              /></svg
            >
          {/snippet}
          Preferiti
        </NavItem>
        <NavItem
          attivo={vistaCorrente === "tutti" && !tagSelezionato}
          conteggio={conteggi.tutti}
          onclick={() => cambiaVista("tutti")}
        >
          {#snippet icona()}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path
                d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
              /><polyline points="14 2 14 8 20 8" /></svg
            >
          {/snippet}
          Tutti i prompt
        </NavItem>
      </div>

      <div class="sb-gruppo">
        <div class="sb-label">Visibilità</div>
        <NavItem
          attivo={vistaCorrente === "privati" && !tagSelezionato}
          conteggio={conteggi.privati}
          onclick={() => cambiaVista("privati")}
        >
          {#snippet icona()}
            <span class="sb-dot sb-dot--private"></span>
          {/snippet}
          Privati
        </NavItem>
        <NavItem
          attivo={vistaCorrente === "team" && !tagSelezionato}
          conteggio={conteggi.team}
          onclick={() => cambiaVista("team")}
        >
          {#snippet icona()}
            <span class="sb-dot sb-dot--team"></span>
          {/snippet}
          Team
        </NavItem>
      </div>

      {#if tags.length > 0}
        <div class="sb-gruppo">
          <div class="sb-label">Tag</div>
          {#each tags as tag}
            <NavItem
              attivo={tagSelezionato === tag.id}
              onclick={() => cambiaTag(tag.id)}
            >
              {#snippet icona()}
                <span
                  class="sb-dot"
                  style:background={tag.colore || "var(--text-subtle)"}
                ></span>
              {/snippet}
              {tag.nome}
            </NavItem>
          {/each}
        </div>
      {/if}

      <div class="sb-spacer"></div>

      <div class="sb-gruppo">
        <NavItem onclick={() => (mostraImpostazioni = true)}>
          {#snippet icona()}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path d="M20 7h-9" /><path d="M14 17H5" /><circle
                cx="17"
                cy="17"
                r="3"
              /><circle cx="7" cy="7" r="3" /></svg
            >
          {/snippet}
          Impostazioni
        </NavItem>
      </div>
    </aside>

    <!-- ── Lista ── -->
    <section class="lista">
      <div class="lista-head">
        <div class="lista-riga1">
          <h2 class="lista-titolo">{titoloVista}</h2>
          <span class="lista-count">{conteggioVista}</span>
          <div class="lista-spacer"></div>
          <Button
            variante="primary"
            dimensione="sm"
            onclick={() => {
              promptPerEditor = null;
              editorKey++;
              mostraEditor = true;
            }}>+ Nuovo</Button
          >
          <select
            class="lista-sort"
            bind:value={ordine}
            onchange={() => caricaLista()}
          >
            <option value="recente">Recenti</option>
            <option value="popolare">Popolari</option>
            <option value="alfabetico">A-Z</option>
          </select>
        </div>
        <div class="lista-search">
          <svg
            class="lista-search-ico"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><circle cx="11" cy="11" r="8" /><path
              d="m21 21-4.3-4.3"
            /></svg
          >
          <input
            class="lista-search-input"
            bind:value={cercaTesto}
            oninput={gestisciCerca}
            placeholder="Cerca…"
          />
          {#if cercaTesto}
            <button
              class="lista-search-clear"
              onclick={() => {
                cercaTesto = "";
                caricaLista();
              }}
              type="button"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <div class="lista-corpo">
        {#if prompts.length === 0}
          <EmptyState
            titolo={cercaTesto ? "Nessun risultato" : "Nessun prompt ancora"}
            hint={cercaTesto
              ? "Prova una ricerca diversa"
              : "Crea il tuo primo prompt"}
          />
        {:else}
          {#each prompts as p}
            <button
              class="prompt-card"
              class:prompt-card--sel={p.id === idSelezionato}
              aria-selected={p.id === idSelezionato}
              onclick={() => selezionaPrompt(p.id)}
              type="button"
            >
              <div class="pc-head">
                {#if p.visibilita === "private"}
                  <svg
                    class="pc-vis pc-vis--private"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><rect
                      width="18"
                      height="11"
                      x="3"
                      y="11"
                      rx="2"
                      ry="2"
                    /><path d="M7 11V7a5 5 0 0 1 10 0v4" /></svg
                  >
                {:else}
                  <svg
                    class="pc-vis pc-vis--team"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path
                      d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"
                    /><circle cx="9" cy="7" r="4" /><path
                      d="M22 21v-2a4 4 0 0 0-3-3.87"
                    /><path d="M16 3.13a4 4 0 0 1 0 7.75" /></svg
                  >
                {/if}
                <span class="pc-title">{p.titolo}</span>
                <span class="pc-meta"
                  >{tempoRelativo(p.aggiornato_a)}</span
                >
              </div>
              {#if p.descrizione}
                <p class="pc-desc">{p.descrizione}</p>
              {/if}
              {#if p.tags.length > 0}
                <div class="pc-foot">
                  {#each p.tags as tag}
                    <Tag colore={tag.colore}>{tag.nome}</Tag>
                  {/each}
                </div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
    </section>

    <!-- ── Dettaglio ── -->
    <section class="dettaglio">
      {#if promptDet}
        <div class="det-head">
          <div class="det-top">
            <h1 class="det-titolo">{promptDet.titolo}</h1>
            <div class="det-azioni">
              <Button
                variante="ghost"
                dimensione="sm"
                onclick={togglePreferito}
              >
                {promptDet.preferito ? "★" : "☆"}
              </Button>
              <Button
                variante="ghost"
                dimensione="sm"
                onclick={() => {
                  promptPerEditor = promptDet;
                  editorKey++;
                  mostraEditor = true;
                }}>Modifica</Button
              >
              <Button
                variante="primary"
                dimensione="sm"
                onclick={() => {
                  compilatoreKey++;
                  mostraCompilatore = true;
                }}>Compila</Button
              >
            </div>
          </div>
          {#if promptDet.descrizione}
            <p class="det-desc">{promptDet.descrizione}</p>
          {/if}
          <div class="det-meta">
            <span>
              {#if promptDet.visibilita === "private"}Privato{:else}Team{/if}
            </span>
            {#if promptDet.uso_count > 0}
              <span
                >Usato {promptDet.uso_count}
                {promptDet.uso_count === 1 ? "volta" : "volte"}</span
              >
            {/if}
            <span>{tempoRelativo(promptDet.aggiornato_a)}</span>
          </div>
          {#if promptDet.tags.length > 0}
            <div class="det-tags">
              {#each promptDet.tags as tag}
                <Tag colore={tag.colore}>{tag.nome}</Tag>
              {/each}
            </div>
          {/if}
        </div>

        <div class="det-body">
          <div class="det-sezione">
            <h3>Corpo del prompt</h3>
          </div>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="body-preview">
            {@html renderPreview(promptDet.body)}
          </div>

          {#if segnaposti.length > 0}
            <div class="det-sezione">
              <h3>Segnaposti rilevati</h3>
              <span class="det-sezione-meta">
                {segnaposti.length}
                {segnaposti.length === 1 ? "parametro" : "parametri"}
              </span>
            </div>
            <div class="params-grid">
              {#each segnaposti as s}
                <div class="param">
                  <span class="pname">{`{{${s.nome}}}`}</span>
                  <span class="ptype">testo</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <div class="det-vuoto">
          <EmptyState
            titolo="Seleziona un prompt"
            hint="Clicca su un prompt nella lista per vedere i dettagli"
          />
        </div>
      {/if}
    </section>

    <!-- ── Status bar ── -->
    <footer class="statusbar">
      <div class="statusbar-lato">
        <span class="dot dot-ok"></span>
        <span>Locale</span>
      </div>
      <div class="statusbar-lato">
        <span>v0.1.0</span>
        <kbd class="statusbar-kbd">{hotkeyCombo}</kbd>
      </div>
    </footer>

    {#if mostraEditor}
      {#key editorKey}
        <EditorPrompt
          prompt={promptPerEditor}
          onchiudi={() => (mostraEditor = false)}
          onsalvato={() => {
            mostraEditor = false;
            caricaDati();
          }}
        />
      {/key}
    {/if}

    {#if mostraCompilatore && promptDet}
      {#key compilatoreKey}
        <CompilatorePrompt
          prompt={promptDet}
          onchiudi={() => {
            mostraCompilatore = false;
            caricaDati();
          }}
        />
      {/key}
    {/if}

    {#if mostraImpostazioni}
      <Impostazioni
        onchiudi={() => {
          mostraImpostazioni = false;
          caricaDati();
        }}
      />
    {/if}
  </div>
{/if}

<style>
  /* ── Loading & Lock ── */

  .libreria-full {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .unlock {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-3);
    max-width: 320px;
    text-align: center;
  }

  .unlock-icona {
    color: var(--text-subtle);
  }
  .unlock-titolo {
    font-size: var(--fs-xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    margin: 0;
  }
  .unlock-desc {
    font-size: var(--fs-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .unlock-form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    width: 100%;
    margin-top: var(--sp-3);
  }

  .unlock-input {
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
  .unlock-input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }
  .unlock-errore {
    font-size: var(--fs-xs);
    color: var(--danger);
    margin: 0;
  }

  /* ── Libreria grid ── */

  .libreria {
    display: grid;
    grid-template-columns: 240px 360px 1fr;
    grid-template-rows: 1fr 28px;
    height: 100vh;
    background: var(--bg-canvas);
    font-family: var(--font-ui);
    color: var(--text-default);
  }

  /* ── Sidebar ── */

  .sidebar {
    grid-row: 1;
    grid-column: 1;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-subtle);
    padding: var(--sp-3);
    gap: var(--sp-4);
    overflow-y: auto;
  }

  .ws-switcher {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .ws-avatar {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: linear-gradient(
      135deg,
      var(--accent-team),
      var(--accent-team-strong)
    );
    color: var(--accent-team-on);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: var(--fw-bold);
    flex-shrink: 0;
  }

  .ws-nome {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sb-gruppo {
    display: flex;
    flex-direction: column;
  }

  .sb-label {
    font-family: var(--font-mono);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-subtle);
    padding: var(--sp-2) var(--sp-3) 4px;
  }

  .sb-spacer {
    flex: 1;
  }

  .sb-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .sb-dot--private {
    background: var(--accent-private);
  }
  .sb-dot--team {
    background: var(--accent-team);
  }

  /* ── Lista ── */

  .lista {
    grid-row: 1;
    grid-column: 2;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-subtle);
    overflow: hidden;
  }

  .lista-head {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .lista-riga1 {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .lista-titolo {
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    margin: 0;
  }

  .lista-count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .lista-spacer {
    flex: 1;
  }

  .lista-sort {
    appearance: none;
    height: 26px;
    padding: 0 24px 0 8px;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background-color: var(--bg-overlay);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M1 1l4 4 4-4' fill='none' stroke='%236b7280' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    outline: none;
  }

  .lista-search {
    position: relative;
  }

  .lista-search-ico {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-subtle);
    pointer-events: none;
  }

  .lista-search-input {
    width: 100%;
    height: 32px;
    padding: 0 var(--sp-3) 0 32px;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    outline: none;
    box-sizing: border-box;
    transition: border-color var(--motion-fast);
  }
  .lista-search-input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }
  .lista-search-input::placeholder {
    color: var(--text-subtle);
  }

  .lista-search-clear {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    appearance: none;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0 5px;
    font-size: 11px;
    color: var(--text-muted);
    cursor: pointer;
    height: 18px;
    display: flex;
    align-items: center;
    line-height: 1;
  }

  .lista-corpo {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2);
  }

  /* ── Prompt card ── */

  .prompt-card {
    appearance: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    padding: var(--sp-3);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-ui);
    color: var(--text-default);
    transition: background var(--motion-fast);
  }
  .prompt-card:hover {
    background: var(--bg-overlay);
  }
  .prompt-card--sel {
    background: var(--bg-overlay);
    border-color: var(--border-default);
  }

  .pc-head {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .pc-vis {
    flex-shrink: 0;
    color: var(--text-muted);
  }
  .pc-vis--private {
    color: var(--accent-private);
  }
  .pc-vis--team {
    color: var(--accent-team);
  }

  .pc-title {
    flex: 1;
    min-width: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pc-meta {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-subtle);
    flex-shrink: 0;
  }

  .pc-desc {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    line-height: var(--lh-snug);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .pc-foot {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-top: 2px;
  }

  /* ── Dettaglio ── */

  .dettaglio {
    grid-row: 1;
    grid-column: 3;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .det-vuoto {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .det-head {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-4) var(--sp-5) var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .det-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
  }

  .det-titolo {
    margin: 0;
    font-size: var(--fs-2xl);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    letter-spacing: var(--tracking-tight);
    line-height: var(--lh-tight);
    flex: 1;
  }

  .det-azioni {
    display: flex;
    gap: var(--sp-2);
    flex-shrink: 0;
  }

  .det-desc {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    max-width: 60ch;
    margin: 0;
  }

  .det-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .det-meta span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .det-tags {
    display: flex;
    gap: var(--sp-2);
  }

  .det-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-5);
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
  }

  .det-sezione {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .det-sezione h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
  }

  .det-sezione-meta {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  .body-preview {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-loose);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-4);
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
    -webkit-user-select: text;
  }

  :global(.body-preview .ph) {
    display: inline;
    font-family: var(--font-mono);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }

  :global(.body-preview .ph .br) {
    opacity: 0.55;
    font-weight: var(--fw-regular);
  }

  .params-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
  }

  .param {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-3);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .pname {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--accent-private);
  }

  .ptype {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-subtle);
  }

  /* ── Status bar ── */

  .statusbar {
    grid-row: 2;
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 var(--sp-3);
    background: var(--bg-surface);
    border-top: 1px solid var(--border-subtle);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .statusbar-lato {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    display: inline-block;
  }
  .dot-ok {
    background: var(--success);
  }

  .statusbar-kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    padding: 0 4px;
    height: 16px;
    display: inline-flex;
    align-items: center;
  }
</style>
