<script lang="ts">
  /**
   * Modale «Cartamodello» — scompone uno o più prompt piatti in moduli
   * componibili (blueprint docs/roadmap/cartamodello.md).
   *
   * Fasi: scelta provider → analisi (chiamata al modello, nulla scritto) →
   * revisione (moduli con crea/riusa/scarta e titolo editabile; prompt con
   * sorgente, anteprima espansa e diff di fedeltà) → applicazione
   * (`cartamodello_applica`, una transazione: moduli creati, originali
   * salvati come nuova versione). Aperta dallo store globale
   * (`apriModale({ tipo: "cartamodello", promptIds })`) da DetailPane e dal
   * menu della selezione multipla.
   */
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Modale from "$lib/components/Modale.svelte";
  import DiffViewer from "$lib/components/DiffViewer.svelte";
  import AiutoLink from "$lib/aiuto/AiutoLink.svelte";
  import { providerHaModelliNoti, opzioniModello } from "$lib/modelli-provider";
  import {
    conScelta,
    conTitoloPrompt,
    costruisciRichiesta,
    ETICHETTA_TIPO,
    formattaCosto,
    formattaEsito,
    formattaSimilarita,
    motiviBloccanti,
    scelteIniziali,
    scelteInizialiPrompt,
    type EsitoApplica,
    type Proposta,
    type SceltaModulo,
    type SceltaPrompt,
  } from "./cartamodello-logic";

  interface Props {
    promptIds: string[];
    onChiudi: () => void;
  }

  let { promptIds, onChiudi }: Props = $props();

  interface ProviderConfigItem {
    provider: string;
    base_url?: string | null;
    default_model?: string | null;
    abilitato: boolean;
  }

  type Fase = "scelta" | "analisi" | "revisione" | "applicazione" | "fatto";
  type VistaPrompt = "sorgente" | "anteprima" | "diff";

  let fase = $state<Fase>("scelta");
  let providers = $state<ProviderConfigItem[]>([]);
  let providerScelto = $state("");
  let modelScelto = $state("");
  let gating = $state(true);
  let errore = $state("");
  let proposta = $state<Proposta | null>(null);
  let scelte = $state<SceltaModulo[]>([]);
  let sceltePrompt = $state<SceltaPrompt[]>([]);
  let vista = $state<Record<string, VistaPrompt>>({});
  let esito = $state<EsitoApplica | null>(null);

  const motivi = $derived(
    proposta ? motiviBloccanti(proposta, scelte, sceltePrompt) : [],
  );
  const nProposti = $derived(proposta?.prompt.length ?? 0);

  onMount(() => {
    // L'editor potrebbe avere una bozza non ancora autosalvata di uno dei
    // prompt del lotto: chiediamo a DetailPane di scriverla ora, così
    // l'analisi legge il testo vero e il reload finale non la perde
    // (review PR-2). La modale è un focus trap: da qui in poi nessuno
    // scrive nell'editor.
    window.dispatchEvent(new CustomEvent("pap:flush-autosave"));
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

  function onCambiaProvider(nome: string): void {
    providerScelto = nome;
    const p = providers.find((x) => x.provider === nome);
    modelScelto = p?.default_model ?? "";
  }

  async function analizza(): Promise<void> {
    if (!providerScelto || !modelScelto.trim()) {
      errore = "Scegli provider e modello prima di avviare.";
      return;
    }
    errore = "";
    fase = "analisi";
    try {
      const p = await invoke<Proposta>("cartamodello_analizza", {
        promptIds,
        providerKind: providerScelto,
        model: modelScelto.trim(),
      });
      proposta = p;
      scelte = scelteIniziali(p);
      sceltePrompt = scelteInizialiPrompt(p);
      vista = Object.fromEntries(p.prompt.map((x) => [x.id, "diff" as VistaPrompt]));
      fase = "revisione";
    } catch (e) {
      errore = String(e).replace(/^Error: /, "");
      fase = "scelta";
    }
  }

  async function applica(): Promise<void> {
    if (!proposta || motivi.length > 0) return;
    errore = "";
    fase = "applicazione";
    try {
      const r = await invoke<EsitoApplica>("cartamodello_applica", {
        richiesta: costruisciRichiesta(proposta, scelte, sceltePrompt),
      });
      esito = r;
      fase = "fatto";
      window.dispatchEvent(new CustomEvent("pap:lista-mutata"));
      for (const p of r.prompt_aggiornati) {
        window.dispatchEvent(
          new CustomEvent("pap:prompt-ripristinato", { detail: { promptId: p.id } }),
        );
      }
    } catch (e) {
      errore = String(e).replace(/^Error: /, "");
      fase = "revisione";
    }
  }

  function scelta(chiave: string): SceltaModulo | undefined {
    return scelte.find((s) => s.chiave === chiave);
  }
</script>

<Modale
  titolo="Cartamodello — scomponi in moduli"
  sottotitolo={promptIds.length === 1
    ? "Il prompt viene tagliato nei suoi pezzi riusabili e ricomposto con import e segnaposti."
    : `${promptIds.length} prompt: i pezzi in comune diventano moduli condivisi.`}
  larghezza="xl"
  onChiudi={onChiudi}
>
  {#if !gating}
    <div class="blocco">
      <p>
        Per usare Cartamodello configura un provider AI (con le sue API key) in
        <strong>Impostazioni → Provider AI</strong> e abilitalo.
      </p>
      {#if errore}<p class="err">{errore}</p>{/if}
    </div>
  {:else if fase === "scelta"}
    <div class="blocco">
      <p class="hint">
        Il testo dei prompt viene inviato al provider scelto con la sintassi di
        Prompt à Porter. Il modello propone i moduli (ruolo, vincoli, formato…) e
        i prompt ricomposti; tu rivedi tutto prima che venga scritto qualcosa.
        <AiutoLink chiave="cartamodello" dimensione={16} />
      </p>
      <label class="campo">
        <span>Provider</span>
        <select value={providerScelto} onchange={(e) => onCambiaProvider(e.currentTarget.value)}>
          {#each providers as p (p.provider)}
            <option value={p.provider}>{p.provider}</option>
          {/each}
        </select>
      </label>
      <label class="campo">
        <span>Modello</span>
        {#if providerHaModelliNoti(providerScelto)}
          <select bind:value={modelScelto}>
            {#each opzioniModello(providerScelto, modelScelto) as m (m.value)}
              <option value={m.value}>{m.etichetta}</option>
            {/each}
          </select>
        {:else}
          <input type="text" bind:value={modelScelto} placeholder="nome del modello" />
        {/if}
      </label>
      {#if errore}<p class="err">{errore}</p>{/if}
    </div>
  {:else if fase === "analisi" || fase === "applicazione"}
    <div class="attesa" role="status" aria-live="polite">
      <span class="spinner" aria-hidden="true"></span>
      <p class="attesa-titolo">
        {fase === "analisi" ? "Il sarto prende le misure…" : "Il sarto cuce…"}
      </p>
      <p class="hint">
        {fase === "analisi"
          ? "Richiesta al modello AI in corso: può richiedere qualche decina di secondi."
          : "Creazione dei moduli e nuova versione dei prompt."}
      </p>
    </div>
  {:else if fase === "revisione" && proposta}
    <div class="blocco">
      {#if proposta.troncato}
        <p class="warn">⚠️ Output forse troncato: la proposta potrebbe essere incompleta.</p>
      {/if}
      {#each proposta.avvisi as a, i (i)}
        <p class="warn">{a}</p>
      {/each}
      {#if errore}<p class="err" role="alert">{errore}</p>{/if}

      <section>
        <h4>Moduli proposti <span class="conteggio">{proposta.moduli.length}</span></h4>
        {#if proposta.moduli.length === 0}
          <p class="hint">Il modello non ha trovato pezzi riusabili.</p>
        {:else}
          <p class="hint">
            I moduli nuovi finiscono in <code>{proposta.cartella_moduli}</code>.
            {#if !proposta.riuso_semantico}
              Il confronto con i prompt esistenti è solo lessicale (modello di ricerca semantica non caricato).
            {/if}
          </p>
          <ul class="moduli">
            {#each proposta.moduli as m (m.chiave)}
              {@const s = scelta(m.chiave)}
              <li class="modulo" class:scartato={s?.azione === "scarta"}>
                <div class="modulo-testa">
                  <span class="badge">{ETICHETTA_TIPO[m.tipo]}</span>
                  <input
                    class="titolo"
                    type="text"
                    value={s?.titolo ?? m.titolo}
                    disabled={s?.azione !== "crea"}
                    aria-label="Titolo del modulo"
                    oninput={(e) => (scelte = conScelta(scelte, m.chiave, { titolo: e.currentTarget.value }))}
                  />
                  <span class="usato">usato da {m.usato_da.length || "?"}</span>
                </div>
                <div class="azioni" role="radiogroup" aria-label="Cosa fare del modulo">
                  <label>
                    <input type="radio" name={`az-${m.chiave}`} checked={s?.azione === "crea"}
                      onchange={() => (scelte = conScelta(scelte, m.chiave, { azione: "crea" }))} />
                    Crea nuovo
                  </label>
                  {#if m.riuso}
                    <label>
                      <input type="radio" name={`az-${m.chiave}`} checked={s?.azione === "riusa"}
                        onchange={() => (scelte = conScelta(scelte, m.chiave, { azione: "riusa" }))} />
                      Usa esistente «{m.riuso.titolo}» <span class="hint">({formattaSimilarita(m.riuso)})</span>
                    </label>
                  {/if}
                  <label>
                    <input type="radio" name={`az-${m.chiave}`} checked={s?.azione === "scarta"}
                      onchange={() => (scelte = conScelta(scelte, m.chiave, { azione: "scarta" }))} />
                    Scarta
                  </label>
                </div>
                {#if m.titolo_in_conflitto && s?.azione === "crea" && (s?.titolo ?? "").trim().toLowerCase() === m.titolo.trim().toLowerCase()}
                  <p class="warn">Esiste già un prompt con questo titolo: rinominalo o usa quello esistente.</p>
                {/if}
                {#each m.problemi as pr, i (i)}
                  <p class="err">{pr}</p>
                {/each}
                <details>
                  <summary>Corpo del modulo</summary>
                  <pre class="corpo">{m.corpo}</pre>
                </details>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section>
        <h4>Prompt ricomposti <span class="conteggio">{nProposti}</span></h4>
        {#each proposta.prompt as p (p.id)}
          {@const sp = sceltePrompt.find((x) => x.id === p.id)}
          <article class="prompt">
            <header class="prompt-testa">
              <input
                class="titolo"
                type="text"
                value={sp?.titolo ?? p.titolo_originale}
                aria-label="Titolo del prompt"
                oninput={(e) => (sceltePrompt = conTitoloPrompt(sceltePrompt, p.id, e.currentTarget.value))}
              />
              {#if p.titolo.trim() && p.titolo.trim().toLowerCase() !== p.titolo_originale.trim().toLowerCase()}
                <button
                  type="button"
                  class="suggerimento"
                  title="Usa il titolo proposto dal modello"
                  onclick={() => (sceltePrompt = conTitoloPrompt(sceltePrompt, p.id, p.titolo))}
                >
                  il modello propone «{p.titolo}»
                </button>
              {/if}
              {#if p.segnaposti.length > 0}
                <span class="hint">segnaposti: {p.segnaposti.join(", ")}</span>
              {/if}
            </header>
            {#each p.import_non_risolti as imp (imp)}
              <p class="err">L'import «{imp}» non risolve a nessun modulo.</p>
            {/each}
            {#each p.problemi as pr, i (i)}
              <p class="err">{pr}</p>
            {/each}
            <div class="tabs" role="tablist">
              {#each [["diff", "Diff con l'originale"], ["anteprima", "Anteprima espansa"], ["sorgente", "Sorgente"]] as const as [id, label] (id)}
                <button
                  type="button"
                  role="tab"
                  class:attiva={(vista[p.id] ?? "diff") === id}
                  aria-selected={(vista[p.id] ?? "diff") === id}
                  onclick={() => (vista = { ...vista, [p.id]: id })}
                >
                  {label}
                </button>
              {/each}
            </div>
            {#if (vista[p.id] ?? "diff") === "diff"}
              {#if p.anteprima_espansa !== null}
                <div class="diff-scroll">
                  <DiffViewer
                    bodyA={p.corpo_originale}
                    bodyB={p.anteprima_espansa}
                    etichettaA="Originale"
                    etichettaB="Ricomposto, espanso"
                    altezza="contenuto"
                  />
                </div>
                <p class="hint">
                  Un diff quasi vuoto vuol dire che la scomposizione è fedele: ha solo spostato testo in moduli e segnaposti.
                </p>
              {:else}
                <p class="hint">Anteprima non disponibile finché ci sono import non risolti.</p>
              {/if}
            {:else if (vista[p.id] ?? "diff") === "anteprima"}
              <pre class="corpo">{p.anteprima_espansa ?? "(import non risolti)"}</pre>
            {:else}
              <pre class="corpo">{p.corpo}</pre>
            {/if}
          </article>
        {/each}
      </section>

      {#if proposta.note.length > 0}
        <details>
          <summary>Note del modello ({proposta.note.length})</summary>
          <ul class="note">
            {#each proposta.note as n, i (i)}<li>{n}</li>{/each}
          </ul>
        </details>
      {/if}

      {#if motivi.length > 0}
        <div class="bloccanti" role="alert">
          <strong>Prima di applicare:</strong>
          <ul>
            {#each motivi as m, i (i)}<li>{m}</li>{/each}
          </ul>
        </div>
      {/if}

      <p class="meta">
        {proposta.provider} · {proposta.model}
        {#if proposta.tokens_used != null}· {proposta.tokens_used} token{/if}
        · {formattaCosto(proposta.costo_stimato)}
      </p>
    </div>
  {:else if fase === "fatto" && esito}
    <div class="blocco">
      <p class="ok">✓ {formattaEsito(esito)}</p>
      {#if esito.moduli_creati.length > 0}
        <p class="hint">
          Moduli creati: {esito.moduli_creati.map((m) => m.titolo).join(", ")}.
        </p>
      {/if}
      <p class="hint">
        Gli originali sono alla versione precedente nella tab Cronologia: se il risultato non ti convince, ripristina.
      </p>
    </div>
  {/if}

  {#snippet footer()}
    {#if !gating}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Chiudi</button>
    {:else if fase === "scelta"}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Annulla</button>
      <button type="button" class="btn-primary" onclick={() => void analizza()}>Analizza</button>
    {:else if fase === "analisi" || fase === "applicazione"}
      <button type="button" class="btn-primary" disabled>In corso…</button>
    {:else if fase === "revisione"}
      <button type="button" class="btn-secondary" onclick={onChiudi}>Annulla</button>
      <button type="button" class="btn-secondary" onclick={() => (fase = "scelta")}>Rianalizza</button>
      <button
        type="button"
        class="btn-primary"
        disabled={motivi.length > 0}
        title={motivi.length > 0 ? motivi[0] : "Crea i moduli e salva i prompt ricomposti come nuova versione"}
        onclick={() => void applica()}
      >
        Applica
      </button>
    {:else}
      <button type="button" class="btn-primary" onclick={onChiudi}>Chiudi</button>
    {/if}
  {/snippet}
</Modale>

<style>
  .blocco {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  .hint {
    color: var(--text-muted);
    font-size: var(--fs-sm);
    margin: 0;
  }
  .hint code {
    background: var(--bg-overlay);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .err {
    color: var(--danger);
    font-size: var(--fs-sm);
    margin: 0;
  }
  .warn {
    color: var(--warning, #b8860b);
    font-size: var(--fs-sm);
    margin: 0;
  }
  .ok {
    color: var(--accent-success, #2c8a2c);
    font-weight: var(--fw-medium);
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
  h4 {
    margin: 0 0 0.4rem;
    font-size: 0.95rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .conteggio {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    padding: 0 6px;
  }

  .attesa {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    padding: 2.5rem 0;
  }
  .attesa-titolo {
    margin: 0;
    font-weight: var(--fw-medium);
  }
  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-subtle);
    border-top-color: var(--accent-team);
    border-radius: 50%;
    animation: gira 0.8s linear infinite;
  }
  @keyframes gira {
    to {
      transform: rotate(360deg);
    }
  }

  .moduli {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .modulo {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    background: var(--bg-canvas);
  }
  .modulo.scartato {
    opacity: 0.6;
  }
  .modulo-testa {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .badge {
    font-size: var(--fs-xs);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .titolo {
    flex: 1;
    min-width: 0;
    padding: 0.3rem 0.45rem;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }
  .usato {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .azioni {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem 1rem;
    font-size: var(--fs-sm);
  }
  .azioni label {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
  }
  details summary {
    cursor: pointer;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }
  .corpo {
    margin: 0.3rem 0 0;
    padding: 0.5rem;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 32vh;
    overflow: auto;
  }

  .prompt {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-bottom: 0.6rem;
  }
  .prompt-testa {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
  .prompt-testa .titolo {
    flex: 1 1 14rem;
    font-weight: var(--fw-medium);
  }
  .suggerimento {
    border: 1px dashed var(--border-default);
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-full);
    padding: 2px 8px;
    font-size: var(--fs-xs);
    cursor: pointer;
  }
  .suggerimento:hover {
    color: var(--text-default);
    background: var(--bg-overlay);
  }
  .tabs {
    display: flex;
    gap: 0.25rem;
  }
  .tabs button {
    padding: 4px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-xs);
    cursor: pointer;
  }
  .tabs button.attiva {
    background: var(--bg-overlay);
    color: var(--text-default);
  }
  .diff-scroll {
    max-height: 42vh;
    overflow: auto;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }
  .note {
    margin: 0.3rem 0 0;
    padding-left: 1.1rem;
    font-size: var(--fs-sm);
    color: var(--text-muted);
  }
  .bloccanti {
    border: 1px solid var(--warning, #b8860b);
    border-radius: var(--radius-sm);
    padding: 0.6rem;
    font-size: var(--fs-sm);
  }
  .bloccanti ul {
    margin: 0.3rem 0 0;
    padding-left: 1.1rem;
  }
  .meta {
    color: var(--text-muted);
    font-size: 0.8rem;
    margin: 0;
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
