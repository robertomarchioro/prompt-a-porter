<script lang="ts">
  /**
   * Modale Ritocco (feature "Ritocco", blueprint docs/roadmap/ritocco.md §3).
   *
   * Chiede a un provider AI configurato di suggerire migliorie al prompt,
   * tarate sul modello target del prompt (guide impacchettate lato backend).
   * Mostra suggerimenti + diff attuale/riscritto; "Accetta" delega al parent
   * (DetailPane) l'applicazione come nuova versione.
   *
   * Ospitata localmente da DetailPane (non dallo store globale) perché
   * l'accettazione scrive nell'editor e salva con snapshot: serve il contesto
   * di editing del parent.
   */
  import { invoke } from "@tauri-apps/api/core";
  import Modale from "$lib/components/Modale.svelte";
  import DiffViewer from "$lib/components/DiffViewer.svelte";
  import { providerHaModelliNoti, opzioniModello } from "$lib/modelli-provider";

  interface Props {
    promptId: string;
    onChiudi: () => void;
    onAccetta: (nuovoBody: string) => void;
  }

  let { promptId, onChiudi, onAccetta }: Props = $props();

  interface ProviderConfigItem {
    provider: string;
    base_url?: string | null;
    default_model?: string | null;
    abilitato: boolean;
  }
  interface Suggerimento {
    titolo: string;
    dettaglio: string;
  }
  interface RitoccoEsito {
    suggerimenti: Suggerimento[];
    prompt_migliorato: string;
    body_originale: string;
    tokens_used: number | null;
    costo_stimato: number | null;
    provider: string;
    model: string;
    troncato: boolean;
  }

  type Fase = "scelta" | "esecuzione" | "risultato";

  let fase = $state<Fase>("scelta");
  let providers = $state<ProviderConfigItem[]>([]);
  let providerScelto = $state("");
  let modelScelto = $state("");
  let errore = $state("");
  let gating = $state(true); // true finché non sappiamo se c'è un provider
  let esito = $state<RitoccoEsito | null>(null);

  // Carica i provider abilitati all'apertura (gating: serve almeno uno).
  $effect(() => {
    void caricaProvider();
  });

  async function caricaProvider(): Promise<void> {
    try {
      const lista = await invoke<ProviderConfigItem[]>("provider_config_lista");
      const abilitati = lista.filter((p) => p.abilitato);
      providers = abilitati;
      gating = abilitati.length > 0;
      if (abilitati.length > 0) {
        providerScelto = abilitati[0].provider;
        modelScelto = abilitati[0].default_model ?? "";
      }
    } catch (e) {
      errore = String(e).replace(/^Error: /, "");
      gating = false;
    }
  }

  // Al cambio di provider, reimposta il modello sul default di quel provider.
  function onCambiaProvider(nome: string): void {
    providerScelto = nome;
    const p = providers.find((x) => x.provider === nome);
    modelScelto = p?.default_model ?? "";
  }

  async function avvia(): Promise<void> {
    if (!providerScelto || !modelScelto.trim()) {
      errore = "Scegli provider e modello prima di avviare.";
      return;
    }
    errore = "";
    fase = "esecuzione";
    try {
      esito = await invoke<RitoccoEsito>("ritocco_esegui", {
        promptId,
        providerKind: providerScelto,
        model: modelScelto.trim(),
      });
      fase = "risultato";
    } catch (e) {
      errore = String(e).replace(/^Error: /, "");
      fase = "scelta";
    }
  }

  function accetta(): void {
    if (!esito || !esito.prompt_migliorato.trim()) return;
    onAccetta(esito.prompt_migliorato);
    onChiudi();
  }

  function fmtCosto(c: number | null): string {
    if (c == null) return "n/d";
    return c < 0.01 ? `~$${c.toFixed(4)}` : `~$${c.toFixed(2)}`;
  }
</script>

<Modale
  titolo="Ritocco — come migliorare il prompt"
  sottotitolo="Suggerimenti AI tarati sul modello del prompt"
  larghezza="lg"
  onChiudi={onChiudi}
>
  {#if !gating}
    <div class="ritocco-msg">
      <p>
        Per usare il Ritocco configura un provider AI (con le sue API key) in
        <strong>Impostazioni → Provider AI</strong> e abilitalo.
      </p>
      {#if errore}<p class="err">{errore}</p>{/if}
    </div>
  {:else if fase === "scelta"}
    <div class="ritocco-scelta">
      <p class="hint">
        Il Ritocco invia il prompt e le linee guida ufficiali del suo modello
        target al provider AI scelto, e propone una versione migliorata.
      </p>
      <label class="campo">
        <span>Provider</span>
        <select
          value={providerScelto}
          onchange={(e) => onCambiaProvider(e.currentTarget.value)}
        >
          {#each providers as p (p.provider)}
            <option value={p.provider}>{p.provider}</option>
          {/each}
        </select>
      </label>
      <label class="campo">
        <span>Modello</span>
        {#if providerHaModelliNoti(providerScelto)}
          <select bind:value={modelScelto}>
            {#each opzioniModello(providerScelto, modelScelto) as m (m)}
              <option value={m}>{m}</option>
            {/each}
          </select>
        {:else}
          <input
            type="text"
            bind:value={modelScelto}
            placeholder="nome del modello"
          />
        {/if}
      </label>
      {#if errore}<p class="err">{errore}</p>{/if}
    </div>
  {:else if fase === "esecuzione"}
    <div class="ritocco-loading" role="status" aria-live="polite">
      <span class="spinner" aria-hidden="true"></span>
      <p class="loading-titolo">Il sarto è al lavoro…</p>
      <p class="hint">
        Richiesta al modello AI in corso: può richiedere qualche secondo.
      </p>
    </div>
  {:else if fase === "risultato" && esito}
    <div class="ritocco-risultato">
      {#if esito.troncato}
        <p class="warn">
          ⚠️ Output forse troncato (prompt molto lungo): valuta un modello con
          più contesto o accorcia il prompt.
        </p>
      {/if}

      <section>
        <h4>Suggerimenti</h4>
        {#if esito.suggerimenti.length === 0}
          <p class="hint">Nessun suggerimento specifico restituito.</p>
        {:else}
          <ul class="sugg">
            {#each esito.suggerimenti as s, i (i)}
              <li><strong>{s.titolo}</strong> — {s.dettaglio}</li>
            {/each}
          </ul>
        {/if}
      </section>

      {#if esito.prompt_migliorato.trim()}
        <section>
          <h4>Modifiche proposte</h4>
          <div class="diff-scroll">
            <DiffViewer
              bodyA={esito.body_originale}
              bodyB={esito.prompt_migliorato}
              etichettaA="Attuale"
              etichettaB="Proposto"
            />
          </div>
        </section>
      {:else}
        <p class="hint">
          Il modello non ha proposto una riscrittura applicabile.
        </p>
      {/if}

      <p class="meta">
        {esito.provider} · {esito.model}
        {#if esito.tokens_used != null}· {esito.tokens_used} token{/if}
        · {fmtCosto(esito.costo_stimato)}
      </p>
    </div>
  {/if}

  {#snippet footer()}
    {#if !gating}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Chiudi</button>
    {:else if fase === "scelta"}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Annulla</button>
      <button type="button" class="btn-primary" onclick={() => void avvia()}>
        Avvia Ritocco
      </button>
    {:else if fase === "esecuzione"}
      <button type="button" class="btn-primary" disabled>In corso…</button>
    {:else}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Annulla</button>
      <button
        type="button"
        class="btn-primary"
        disabled={!esito?.prompt_migliorato.trim()}
        onclick={accetta}
      >
        Accetta suggerimenti
      </button>
    {/if}
  {/snippet}
</Modale>

<style>
  .ritocco-msg,
  .ritocco-scelta,
  .ritocco-risultato {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  .hint {
    color: var(--text-muted, #888);
    font-size: var(--fs-sm, 0.9rem);
    margin: 0;
  }
  .err {
    color: var(--danger, #d93f0b);
    font-size: var(--fs-sm, 0.9rem);
    margin: 0;
  }
  .warn {
    color: var(--warning, #b8860b);
    font-size: var(--fs-sm, 0.9rem);
    margin: 0;
  }
  .campo {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
  }
  .campo select,
  .campo input {
    padding: 0.4rem 0.5rem;
  }

  /* #507 — attesa risposta modello AI: indicatore visibile e inequivocabile. */
  .ritocco-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    padding: 2.5rem 0;
  }
  .loading-titolo {
    margin: 0;
    font-weight: var(--fw-medium, 600);
  }
  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-subtle, #ccc);
    border-top-color: var(--accent-team, #646cff);
    border-radius: 50%;
    animation: gira 0.8s linear infinite;
  }
  @keyframes gira {
    to {
      transform: rotate(360deg);
    }
  }

  /* #506 — la diff dev'essere scrollabile dentro i bordi della modale. */
  .diff-scroll {
    max-height: 42vh;
    overflow: auto;
    border: 1px solid var(--border-subtle, #ccc);
    border-radius: var(--radius-sm, 6px);
  }

  .sugg {
    margin: 0;
    padding-left: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.9rem;
  }
  h4 {
    margin: 0 0 0.4rem;
    font-size: 0.95rem;
  }
  .meta {
    color: var(--text-muted, #888);
    font-size: 0.8rem;
    margin: 0;
  }

  /* #506 — bottoni footer allineati allo stile standard delle modali
     (stessi token di CompilaModal: prima erano classi non definite). */
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
