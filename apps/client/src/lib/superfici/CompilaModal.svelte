<script lang="ts">
  /**
   * Modale Compila (V0.8 F8 PR-A).
   *
   * Porting ridotto di CompilatorePrompt.svelte legacy. Form segnaposti
   * type-aware (testo/multilinea per ora) + preview live + output 3 formati
   * + rating ±1 con icone Frown/Meh/Smile (decisione #6) + nota collassata.
   *
   * Riferimenti:
   * - Decisione designer #6 (rating ±1 + nota preservato)
   * - Blueprint F8 PR-A §1
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, untrack } from "svelte";
  import { Frown, Meh, Smile, Copy, Check, Globe } from "lucide-svelte";
  import { estraiSegnaposti, compila, contaCompilati } from "$lib/template";
  import Modale from "$lib/components/Modale.svelte";
  import { fmtShortcut } from "$lib/util/shortcut";

  interface PromptDettaglio {
    id: string;
    titolo: string;
    body: string;
    target_model: string;
  }

  // Issue #159: backend `globale_placeholder_lista` ritorna questo shape
  interface PlaceholderGlobale {
    name: string;
    value: string;
    updated_at: string;
  }

  interface Props {
    promptId: string;
    onChiudi: () => void;
  }

  let { promptId, onChiudi }: Props = $props();

  type FormatoOutput = "testo" | "markdown" | "json";
  type Rating = -1 | 0 | 1;

  let dettaglio = $state<PromptDettaglio | null>(null);
  // Issue #293: body con {{import}} espansi via backend (specchio di AnteprimaTab).
  // null = espansione non ancora completata o non necessaria (nessun import nel body).
  let bodyEspanso = $state<string | null>(null);
  let erroreEspansione = $state<string | null>(null);
  let valori = $state<Record<string, string>>({});
  // Issue #159: resolver separato per `{{globale nome}}` — pre-fill dal DB.
  let valoriGlobali = $state<Record<string, string>>({});
  // Snapshot dei valori globali al caricamento, per detect dirty su salvataggio.
  let globaliPersistiti = $state<Record<string, string>>({});
  let formato = $state<FormatoOutput>("testo");
  let copiato = $state(false);
  let ratingScelto = $state<Rating | null>(null);
  let nota = $state("");
  let ratingInviato = $state(false);
  let erroreCaricamento = $state<string | null>(null);

  // M3 PR-4: dropdown switch tra varianti senza chiudere la modale.
  // `promptIdAttivo` viene usato per TUTTE le invoke (libreria_dettaglio,
  // rating_aggiungi) -> il parent passa il promptId iniziale come prop
  // ma le interazioni successive seguono la variante selezionata.
  interface VariantInfo {
    id: string;
    parent_prompt_id: string;
    variant_label: string;
    titolo: string;
  }
  // Snapshot iniziale del prop reattivo; untrack evita warning state_referenced_locally.
  let promptIdAttivo = $state(untrack(() => promptId));
  let varianti = $state<VariantInfo[]>([]);

  /**
   * Issue #293: espande i {{import}} del body via backend, specchio di
   * AnteprimaTab.svelte. Usa `prompt_compila_inline` con il body raw e
   * l'id del prompt attivo (per cycle detection corretta).
   *
   * Risultato:
   * - bodyEspanso = stringa espansa  →  segnaposti e output derivano da essa
   * - bodyEspanso = null, erroreEspansione = msg  →  body raw di fallback
   */
  async function espandiImport(rawBody: string, pid: string): Promise<void> {
    if (!rawBody.includes("{{import")) {
      bodyEspanso = null;
      erroreEspansione = null;
      return;
    }
    try {
      const espanso = await invoke<string>("prompt_compila_inline", {
        body: rawBody,
        promptId: pid,
      });
      bodyEspanso = espanso;
      erroreEspansione = null;
    } catch (e) {
      bodyEspanso = null;
      erroreEspansione = String(e).replace(/^Error: /, "");
    }
  }

  /**
   * Carica dettaglio + globali per `promptIdAttivo`.
   *
   * @param preservaValori M3 PR-4: se true (switch variante), preserva i
   *   valori dei segnaposti per le chiavi presenti in entrambi (l'utente
   *   ha gia' digitato qualcosa: non vogliamo che il toggle variante
   *   cancelli il suo lavoro). Default false al primo mount (reset puro).
   */
  async function carica(preservaValori = false): Promise<void> {
    try {
      const [det, glob] = await Promise.all([
        invoke<PromptDettaglio>("libreria_dettaglio", { id: promptIdAttivo }),
        invoke<PlaceholderGlobale[]>("globale_placeholder_lista"),
      ]);
      dettaglio = det;

      // Issue #293: espandi gli import PRIMA di estrarre i segnaposti,
      // così il form mostra i segnaposti del body finale (compresi quelli
      // introdotti dagli import) e l'output li include correttamente.
      await espandiImport(det.body, promptIdAttivo);

      // Usa il body espanso (se disponibile) per estrarre i segnaposti,
      // così il form mostra i campi giusti anche per import annidati.
      const bodyPerSegnaposti = bodyEspanso ?? det.body;
      const seg = estraiSegnaposti(bodyPerSegnaposti);
      const initN: Record<string, string> = {};
      const initG: Record<string, string> = {};
      const mapGlob = new Map(glob.map((g) => [g.name, g.value]));
      const vecchiValori = preservaValori ? valori : {};
      const vecchiGlobali = preservaValori ? valoriGlobali : {};
      for (const s of seg) {
        if (s.globale) {
          initG[s.nome] =
            vecchiGlobali[s.nome] ?? mapGlob.get(s.nome) ?? "";
        } else {
          initN[s.nome] = vecchiValori[s.nome] ?? "";
        }
      }
      valori = initN;
      valoriGlobali = initG;
      if (!preservaValori) {
        // Snapshot riferimento solo al primo caricamento: dopo un
        // switch variante, mantenere lo snapshot originale evita di
        // perdere persistenza pendente sui globali.
        globaliPersistiti = { ...initG };
      }
    } catch (e) {
      console.error("[compila-modal] caricaDettaglio", e);
      erroreCaricamento = "Prompt non disponibile";
    }
  }

  // M3 PR-4: carica anche la lista varianti (sister + main) al mount.
  // Usa parent_id quando promptId attuale e' gia' una variante; backend
  // gestisce entrambi i casi (parent diretto o nipote -> grandparent).
  async function caricaVarianti(): Promise<void> {
    try {
      const lista = await invoke<VariantInfo[]>("varianti_lista", {
        parentId: promptId,
      });
      varianti = lista;
    } catch {
      varianti = [];
    }
  }

  onMount(async () => {
    await carica();
    await caricaVarianti();
  });

  async function switchVariante(nuovoId: string): Promise<void> {
    if (nuovoId === promptIdAttivo) return;
    promptIdAttivo = nuovoId;
    // Reset rating state: il prossimo voto va sulla variante nuova,
    // non sulla precedente.
    ratingScelto = null;
    ratingInviato = false;
    nota = "";
    await carica(true);
  }

  // Issue #293: usa il body espanso (import risolti) come sorgente per
  // segnaposti e output. Se l'espansione non è disponibile (nessun import
  // o errore), si usa il body raw come fallback.
  const bodyPerCompilazione = $derived(
    dettaglio ? (bodyEspanso ?? dettaglio.body) : "",
  );

  const segnaposti = $derived(estraiSegnaposti(bodyPerCompilazione));
  const totaleSegnaposti = $derived(segnaposti.length);
  const compilati = $derived(
    contaCompilati(segnaposti, valori, valoriGlobali),
  );
  const tuttiCompilati = $derived(
    totaleSegnaposti === 0 || compilati === totaleSegnaposti,
  );

  const output = $derived.by(() => {
    if (!dettaglio) return "";
    const testo = compila(bodyPerCompilazione, valori, valoriGlobali);
    if (formato === "testo") return testo;
    if (formato === "markdown") {
      return `\`\`\`\n${testo}\n\`\`\``;
    }
    return JSON.stringify(
      {
        prompt_id: promptIdAttivo,
        titolo: dettaglio.titolo,
        target_model: dettaglio.target_model,
        body: testo,
        valori,
        valori_globali: valoriGlobali,
      },
      null,
      2,
    );
  });

  /**
   * Issue #159: persiste via UPSERT i globali modificati rispetto allo
   * snapshot iniziale. "verrà automaticamente aggiornato anche il valore
   * di default". Best-effort: log su errore senza bloccare la copia.
   */
  async function persistiGlobaliSeModificati(): Promise<void> {
    const dirty: Array<[string, string]> = [];
    for (const [name, value] of Object.entries(valoriGlobali)) {
      if (globaliPersistiti[name] !== value) {
        dirty.push([name, value]);
      }
    }
    if (dirty.length === 0) return;
    try {
      await Promise.all(
        dirty.map(([name, value]) =>
          invoke<void>("globale_placeholder_aggiorna", {
            dati: { name, value },
          }),
        ),
      );
      globaliPersistiti = { ...valoriGlobali };
    } catch (e) {
      console.error("[compila-modal] persistGlobali", e);
    }
  }

  async function copiaOutput(): Promise<void> {
    try {
      await persistiGlobaliSeModificati();
      await navigator.clipboard.writeText(output);
      copiato = true;
      setTimeout(() => (copiato = false), 1500);
    } catch (e) {
      console.error("[compila-modal] copy", e);
    }
  }

  async function inviaRating(scelto: Rating): Promise<void> {
    if (!dettaglio) return;
    // M3 PR-2: voto negativo apre modale dedicato per nota (le note su
    // voti negativi sono il segnale piu' utile per migliorare il prompt;
    // chiediamo esplicitamente invece di affidarci al details collassato).
    // Voti neutri/positivi mantengono il flow diretto.
    if (scelto === -1) {
      ratingScelto = scelto;
      modaleNotaNegativa = {
        aperto: true,
        nota: nota.trim(),
        salvataggio: false,
      };
      return;
    }
    await persistRating(scelto, nota.trim() || null);
  }

  async function persistRating(
    scelto: Rating,
    notaFinale: string | null,
  ): Promise<void> {
    if (!dettaglio) return;
    ratingScelto = scelto;
    try {
      await invoke<string>("rating_aggiungi", {
        nuovo: {
          prompt_id: promptIdAttivo,
          rating: scelto,
          nota: notaFinale,
          used_with_model: dettaglio.target_model || null,
        },
      });
      ratingInviato = true;
      setTimeout(() => onChiudi(), 800);
    } catch (e) {
      console.error("[compila-modal] rating", e);
    }
  }

  // Stato modale "Aggiungi nota" su voto negativo (M3 PR-2).
  let modaleNotaNegativa = $state<{
    aperto: boolean;
    nota: string;
    salvataggio: boolean;
  }>({
    aperto: false,
    nota: "",
    salvataggio: false,
  });

  async function confermaVotoNegativo(conNota: boolean): Promise<void> {
    modaleNotaNegativa.salvataggio = true;
    const testo = modaleNotaNegativa.nota.trim();
    await persistRating(-1, conNota && testo.length > 0 ? testo : null);
    modaleNotaNegativa.aperto = false;
    modaleNotaNegativa.salvataggio = false;
  }

  function chiudiModaleNotaSenzaInviare(): void {
    if (modaleNotaNegativa.salvataggio) return;
    modaleNotaNegativa.aperto = false;
    ratingScelto = null; // rilascia il rating se utente esce senza scegliere
  }

  function gestKeydown(e: KeyboardEvent): void {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void copiaOutput();
    }
  }
</script>

<svelte:window onkeydown={gestKeydown} />

<Modale
  titolo="Compila prompt"
  sottotitolo={dettaglio?.titolo}
  larghezza="lg"
  {onChiudi}
>
  {#if erroreCaricamento}
    <p class="errore">{erroreCaricamento}</p>
  {:else if !dettaglio}
    <p class="muted">Caricamento…</p>
  {:else}
    {#if varianti.length > 1}
      <label class="variante-bar">
        <span class="variante-bar-label">Variante</span>
        <select
          class="variante-select"
          value={promptIdAttivo}
          onchange={(e) => void switchVariante(e.currentTarget.value)}
          aria-label="Scegli variante del prompt"
        >
          {#each varianti as v (v.id)}
            <option value={v.id}>{v.variant_label} — {v.titolo}</option>
          {/each}
        </select>
      </label>
    {/if}
    <div class="grid">
      <div class="form">
        <header class="sez-h">
          <span>SEGNAPOSTI</span>
          <span class="count">{compilati}/{totaleSegnaposti}</span>
        </header>
        {#if totaleSegnaposti === 0}
          <p class="muted">Nessun segnaposto: questo prompt è statico.</p>
        {:else}
          {#each segnaposti as s (s.globale ? `g:${s.nome}` : s.nome)}
            <div class="campo" data-globale={s.globale || undefined}>
              <label
                class="lbl"
                for={`f-${s.globale ? "g-" : ""}${s.nome}`}
              >
                {#if s.globale}
                  <span class="lbl-globale-tag" title="Segnaposto globale">
                    <Globe size={11} />
                    globale
                  </span>
                {/if}
                <code>{s.globale
                    ? `{{globale ${s.nome}}}`
                    : `{{${s.nome}}}`}</code>
              </label>
              {#if s.globale}
                <textarea
                  id={`f-g-${s.nome}`}
                  class="input"
                  rows="2"
                  bind:value={valoriGlobali[s.nome]}
                  placeholder={`Valore globale per ${s.nome}…`}
                ></textarea>
              {:else}
                <textarea
                  id={`f-${s.nome}`}
                  class="input"
                  rows="2"
                  bind:value={valori[s.nome]}
                  placeholder={`Valore per ${s.nome}…`}
                ></textarea>
              {/if}
            </div>
          {/each}
        {/if}

        <header class="sez-h">
          <span>FORMATO OUTPUT</span>
        </header>
        <div class="formati">
          {#each ["testo", "markdown", "json"] as f (f)}
            <button
              type="button"
              class="chip-fmt"
              data-attivo={formato === f || undefined}
              onclick={() => (formato = f as FormatoOutput)}
            >
              {f}
            </button>
          {/each}
        </div>
      </div>

      <div class="preview">
        <header class="sez-h">
          <span>ANTEPRIMA</span>
          <button
            class="copia"
            type="button"
            onclick={copiaOutput}
            title="Copia ({fmtShortcut('ctrl+enter')})"
          >
            {#if copiato}
              <Check size={12} />
              <span>Copiato</span>
            {:else}
              <Copy size={12} />
              <span>Copia</span>
            {/if}
          </button>
        </header>
        {#if erroreEspansione}
          <p class="errore-espansione" title={erroreEspansione}>
            Import non risolvibile: <code>{erroreEspansione}</code>
          </p>
        {/if}
        <pre class="output">{output}</pre>
      </div>
    </div>

    <details class="rating-wrap" open>
      <summary class="rating-summary">
        Valuta il risultato
        {#if ratingInviato}<span class="ok">— grazie!</span>{/if}
      </summary>
      <div class="rating-row">
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === -1 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(-1)}
          aria-label="Negativo"
        >
          <Frown size={16} />
          <span>Negativo</span>
        </button>
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === 0 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(0)}
          aria-label="Neutro"
        >
          <Meh size={16} />
          <span>Neutro</span>
        </button>
        <button
          class="rating-btn"
          type="button"
          data-attivo={ratingScelto === 1 || undefined}
          disabled={ratingInviato}
          onclick={() => inviaRating(1)}
          aria-label="Positivo"
        >
          <Smile size={16} />
          <span>Positivo</span>
        </button>
      </div>
      <details class="nota-wrap">
        <summary class="nota-summary">Aggiungi nota</summary>
        <textarea
          class="input nota"
          rows="2"
          placeholder="Cosa ha funzionato / non ha funzionato (opzionale)"
          bind:value={nota}
        ></textarea>
      </details>
    </details>
  {/if}

  {#snippet footer()}
    <span class="footer-hint">
      <kbd>{fmtShortcut("ctrl+enter")}</kbd> Compila & copia
    </span>
    <button class="btn-secondary" type="button" onclick={onChiudi}>
      Chiudi
    </button>
    <button
      class="btn-primary"
      type="button"
      disabled={!tuttiCompilati || !dettaglio}
      onclick={copiaOutput}
    >
      Compila & copia
    </button>
  {/snippet}
</Modale>

{#if modaleNotaNegativa.aperto}
  <Modale
    titolo="Cosa non ha funzionato?"
    sottotitolo="Una nota aiuta a capire dove migliorare il prompt"
    larghezza="sm"
    onChiudi={chiudiModaleNotaSenzaInviare}
  >
    <div class="nota-neg-body">
      <textarea
        class="nota-neg-input"
        rows="4"
        placeholder="Es. output troppo generico, manca il tono richiesto, formato sbagliato…"
        bind:value={modaleNotaNegativa.nota}
        disabled={modaleNotaNegativa.salvataggio}
      ></textarea>
      <p class="nota-neg-hint">
        Opzionale: puoi anche solo registrare il voto negativo.
      </p>
    </div>
    {#snippet footer()}
      <button
        class="btn-secondary"
        type="button"
        onclick={() => void confermaVotoNegativo(false)}
        disabled={modaleNotaNegativa.salvataggio}
      >
        Salta e registra voto
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={() => void confermaVotoNegativo(true)}
        disabled={modaleNotaNegativa.salvataggio || modaleNotaNegativa.nota.trim().length === 0}
      >
        {modaleNotaNegativa.salvataggio ? "Salvataggio…" : "Salva voto + nota"}
      </button>
    {/snippet}
  </Modale>
{/if}

<style>
  /* M3 PR-4: dropdown variante */
  .variante-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
    padding: 6px 10px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }
  .variante-bar-label {
    font-size: var(--fs-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    flex-shrink: 0;
  }
  .variante-select {
    flex: 1;
    min-width: 0;
    padding: 4px 8px;
    background: var(--bg-canvas);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
    min-height: 320px;
  }

  .form,
  .preview {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    min-width: 0;
  }

  .sez-h {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--text-subtle);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
  }

  .count {
    margin-left: auto;
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
    font-size: 11px;
    text-transform: none;
    letter-spacing: 0;
    background: var(--bg-overlay);
    padding: 1px 6px;
    border-radius: var(--radius-full);
  }

  .copia {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-transform: none;
    letter-spacing: 0;
    font-weight: var(--fw-regular);
  }

  .copia:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .lbl {
    font-size: 11px;
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .lbl code {
    font-family: var(--font-mono);
    background: var(--accent-private-soft);
    color: var(--accent-private);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }

  /* Issue #159: differenziazione visiva segnaposto globale */
  .campo[data-globale] .lbl code {
    background: var(--accent-team-soft);
    color: var(--accent-team-strong);
  }

  .lbl-globale-tag {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    background: var(--accent-team-soft);
    color: var(--accent-team-strong);
    border-radius: var(--radius-full);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .input {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    padding: 6px 8px;
    resize: vertical;
  }

  .input:focus {
    outline: none;
    border-color: var(--accent-team);
  }

  .formati {
    display: inline-flex;
    background: var(--bg-overlay);
    border-radius: var(--radius-sm);
    padding: 2px;
    align-self: flex-start;
  }

  .chip-fmt {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 2px 10px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    text-transform: capitalize;
    font-family: var(--font-ui);
  }

  .chip-fmt[data-attivo] {
    background: var(--bg-surface);
    color: var(--text-default);
  }

  .output {
    flex: 1;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: var(--sp-2);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-default);
    margin: 0;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 320px;
  }

  .errore {
    color: var(--danger);
    text-align: center;
  }

  /* Issue #293: avviso espansione import fallita */
  .errore-espansione {
    margin: 0 0 var(--sp-1);
    padding: 4px 8px;
    background: rgba(220, 80, 80, 0.08);
    color: var(--danger);
    font-size: var(--fs-xs);
    border-radius: var(--radius-sm);
  }

  .errore-espansione code {
    font-family: var(--font-mono);
    background: transparent;
    padding: 0;
  }

  .muted {
    color: var(--text-subtle);
  }

  .rating-wrap {
    margin-top: var(--sp-3);
    border-top: 1px solid var(--border-subtle);
    padding-top: var(--sp-3);
  }

  .rating-summary {
    cursor: pointer;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--text-default);
    margin-bottom: var(--sp-2);
  }

  .ok {
    color: var(--success);
    font-weight: var(--fw-regular);
    font-size: var(--fs-xs);
  }

  .rating-row {
    display: flex;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }

  .rating-btn {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-1);
    padding: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .rating-btn:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--text-default);
  }

  .rating-btn[data-attivo] {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
    color: var(--accent-team-strong);
  }

  .rating-btn:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .nota-wrap {
    margin-top: var(--sp-1);
  }

  .nota-summary {
    cursor: pointer;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin-bottom: 4px;
  }

  .nota {
    width: 100%;
  }

  .footer-hint {
    margin-right: auto;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    color: var(--text-default);
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

  /* M3 PR-2: modale nota su voto negativo */
  .nota-neg-body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .nota-neg-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    resize: vertical;
    min-height: 80px;
  }

  .nota-neg-input:focus {
    outline: 2px solid var(--accent-team);
    outline-offset: -1px;
  }

  .nota-neg-hint {
    margin: 0;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }
</style>
