<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { GitFork, X, Sparkles } from "lucide-svelte";
  import { estraiSegnaposti } from "$lib/template";
  import { estraiImports } from "$lib/util/estrai-imports";
  import { MODELLI_TARGET } from "$lib/modelli-target";
  import Modale from "$lib/components/Modale.svelte";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";
  import Field from "$lib/components/Field.svelte";

  interface TagInfo {
    id: string;
    nome: string;
    colore: string;
  }

  interface Cartella {
    id: string;
    nome: string;
    path: string;
  }

  interface VariantInfo {
    id: string;
    parent_prompt_id: string;
    variant_label: string;
    titolo: string;
  }

  interface TagSuggerito {
    id: string;
    nome: string;
    colore: string;
    score: number;
    sorgente: string;
  }

  interface Props {
    promptId: string;
    titolo: string;
    body: string;
    visibilita: string;
    targetModel: string;
    folderId: string | null;
    tags: TagInfo[];
    onCambiaVisibilita: (v: string) => void;
    onCambiaTarget: (t: string) => void;
    onCambiaFolder: (f: string | null) => void;
    onAggiungiTag: (nome: string) => void;
    onRimuoviTag: (id: string) => void;
    onApriTabImportVar?: () => void;
  }

  let {
    promptId,
    titolo,
    body,
    visibilita,
    targetModel,
    folderId,
    tags,
    onCambiaVisibilita,
    onCambiaTarget,
    onCambiaFolder,
    onAggiungiTag,
    onRimuoviTag,
    onApriTabImportVar,
  }: Props = $props();

  let cartelle = $state<Cartella[]>([]);
  let varianti = $state<VariantInfo[]>([]);
  let tagSuggeriti = $state<TagSuggerito[]>([]);
  let tagInput = $state("");

  let timerSuggerimenti: ReturnType<typeof setTimeout> | undefined;

  onMount(async () => {
    try {
      cartelle = await invoke<Cartella[]>("folder_lista");
    } catch {
      /* ignore */
    }
    void caricaVarianti();
  });

  onDestroy(() => {
    if (timerSuggerimenti) clearTimeout(timerSuggerimenti);
  });

  async function caricaVarianti(): Promise<void> {
    try {
      varianti = await invoke<VariantInfo[]>("varianti_lista", {
        parentId: promptId,
      });
    } catch {
      varianti = [];
    }
  }

  // M3 PR-1: modale "Crea variante" — sostituisce il placeholder F8.
  // Backend prompt_crea_variante accetta etichetta opzionale: se vuota,
  // assegna auto la prossima libera (b, c, d...).
  let modaleVariante = $state<{
    aperto: boolean;
    etichetta: string;
    salvataggio: boolean;
    errore: string;
  }>({
    aperto: false,
    etichetta: "",
    salvataggio: false,
    errore: "",
  });

  function apriModaleCreaVariante(): void {
    modaleVariante = {
      aperto: true,
      etichetta: "",
      salvataggio: false,
      errore: "",
    };
  }

  function chiudiModaleVariante(): void {
    if (modaleVariante.salvataggio) return; // no chiusura mentre invoke pending
    modaleVariante.aperto = false;
  }

  async function confermaCreaVariante(): Promise<void> {
    modaleVariante.salvataggio = true;
    modaleVariante.errore = "";
    try {
      const etichetta = modaleVariante.etichetta.trim();
      const newId = await invoke<string>("prompt_crea_variante", {
        parentId: promptId,
        etichetta: etichetta.length > 0 ? etichetta : null,
      });
      await caricaVarianti();
      // Apri la nuova variante (stesso pattern di import: CustomEvent
      // intercettato da Shell/router).
      window.dispatchEvent(
        new CustomEvent("pap:apri-prompt", { detail: newId }),
      );
      modaleVariante.aperto = false;
    } catch (e) {
      modaleVariante.errore = String(e).replace(/^Error: /, "");
    } finally {
      modaleVariante.salvataggio = false;
    }
  }

  function gestisciKeydownModale(e: KeyboardEvent): void {
    if (e.key === "Enter" && !modaleVariante.salvataggio) {
      e.preventDefault();
      void confermaCreaVariante();
    }
  }

  async function caricaSuggerimenti(): Promise<void> {
    const testo = `${titolo} ${body}`.trim();
    if (testo.length < 10) {
      tagSuggeriti = [];
      return;
    }
    try {
      tagSuggeriti = await invoke<TagSuggerito[]>("tags_suggest", {
        testo,
        limit: 5,
      });
    } catch {
      tagSuggeriti = [];
    }
  }

  // Debounce 500ms su body+titolo per non spammare tags_suggest
  $effect(() => {
    void titolo;
    void body;
    if (timerSuggerimenti) clearTimeout(timerSuggerimenti);
    timerSuggerimenti = setTimeout(caricaSuggerimenti, 500);
  });

  // Reload varianti quando promptId cambia
  $effect(() => {
    void promptId;
    void caricaVarianti();
  });

  function aggiungiViaInput(): void {
    const n = tagInput.trim();
    if (!n) return;
    onAggiungiTag(n);
    tagInput = "";
  }

  function gestiscTagKeydown(e: KeyboardEvent): void {
    if (e.key === "Enter") {
      e.preventDefault();
      aggiungiViaInput();
    } else if (e.key === "Backspace" && !tagInput && tags.length > 0) {
      const ultimo = tags[tags.length - 1];
      if (ultimo) onRimuoviTag(ultimo.id);
    }
  }

  async function apriImport(path: string): Promise<void> {
    try {
      const preview = await invoke<{
        id: string;
        titolo: string;
        body: string;
      }>("prompt_resolve_import_preview", { path });
      window.dispatchEvent(
        new CustomEvent("pap:apri-prompt", { detail: preview.id }),
      );
    } catch (e) {
      console.error("[right-rail] import non risolvibile", path, e);
    }
  }

  function apriVariante(id: string): void {
    window.dispatchEvent(new CustomEvent("pap:apri-prompt", { detail: id }));
  }

  const segnaposti = $derived(estraiSegnaposti(body));
  const imports = $derived(estraiImports(body));
  const tagiNomi = $derived(new Set(tags.map((t) => t.nome)));
  const suggerimentiVisibili = $derived(
    tagSuggeriti.filter((s) => !tagiNomi.has(s.nome)),
  );
</script>

<aside class="right-rail">
  <!-- Sezione Metadati -->
  <section class="sezione">
    <header class="sezione-h">METADATI</header>
    <div class="campo">
      <label class="lbl" for="rr-vis">Visibilità</label>
      <select
        id="rr-vis"
        class="select"
        value={visibilita}
        onchange={(e) => onCambiaVisibilita(e.currentTarget.value)}
      >
        <option value="private">Privato</option>
        <option value="workspace">Team</option>
        <optgroup label="Disponibili in versioni future">
          <option value="" disabled>Workspace condiviso (v0.9+)</option>
          <option value="" disabled>Pubblico/Marketplace (v1.0+)</option>
        </optgroup>
      </select>
    </div>

    <div class="campo">
      <label class="lbl" for="rr-target">Modello target</label>
      <input
        id="rr-target"
        class="input"
        list="rr-target-models"
        value={targetModel}
        oninput={(e) => onCambiaTarget(e.currentTarget.value)}
        placeholder="Modello target"
      />
      <datalist id="rr-target-models">
        {#each MODELLI_TARGET as m (m.value)}
          <option value={m.value}>{m.label}</option>
        {/each}
      </datalist>
    </div>

    <div class="campo">
      <label class="lbl" for="rr-folder">Cartella</label>
      <select
        id="rr-folder"
        class="select"
        value={folderId ?? ""}
        onchange={(e) =>
          onCambiaFolder(e.currentTarget.value || null)}
      >
        <option value="">Nessuna cartella</option>
        {#each cartelle as c (c.id)}
          <option value={c.id}>{c.path}</option>
        {/each}
      </select>
    </div>

    <div class="campo">
      <label class="lbl" for="rr-tag-input">Tag</label>
      {#if tags.length > 0}
        <div class="tags-list">
          {#each tags as t (t.id)}
            <span
              class="tag-chip"
              style:--tag-c={t.colore || "var(--text-subtle)"}
            >
              <span>{t.nome}</span>
              <button
                type="button"
                aria-label="Rimuovi tag {t.nome}"
                onclick={() => onRimuoviTag(t.id)}
              >
                <X size={10} />
              </button>
            </span>
          {/each}
        </div>
      {/if}
      <input
        id="rr-tag-input"
        class="input"
        type="text"
        placeholder="Aggiungi tag, premi Invio"
        bind:value={tagInput}
        onkeydown={gestiscTagKeydown}
      />
      {#if suggerimentiVisibili.length > 0}
        <div class="suggerimenti">
          <span class="lbl-sub">
            <Sparkles size={11} /> Suggeriti
          </span>
          <div class="sugg-list">
            {#each suggerimentiVisibili as s (s.id)}
              <button
                type="button"
                class="sugg"
                onclick={() => onAggiungiTag(s.nome)}
                title="Punteggio {s.score.toFixed(2)} ({s.sorgente})"
              >
                + {s.nome}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </section>

  <!-- Sezione Segnaposti rilevati -->
  <section class="sezione">
    <header class="sezione-h">
      SEGNAPOSTI RILEVATI
      <span class="count">{segnaposti.length}</span>
    </header>
    {#if segnaposti.length === 0}
      <p class="vuoto">Nessun segnaposto.</p>
    {:else}
      {#each segnaposti as s (s.nome)}
        <div class="segnaposto">
          <code class="ph">{`{{${s.nome}}}`}</code>
          <span class="tipo">testo</span>
        </div>
      {/each}
    {/if}
  </section>

  <!-- Sezione Import composti -->
  <section class="sezione">
    <header class="sezione-h">
      IMPORT COMPOSTI
      <span class="count">{imports.length}</span>
    </header>
    {#if imports.length === 0}
      <p class="vuoto">Nessun import.</p>
    {:else}
      {#each imports as path (path)}
        <button
          class="import-row"
          type="button"
          onclick={() => apriImport(path)}
          title="Apri prompt importato"
        >
          <GitFork size={11} />
          <code>{path}</code>
        </button>
      {/each}
    {/if}
  </section>

  <!-- Sezione Varianti A/B -->
  <section class="sezione">
    <header class="sezione-h">
      VARIANTI A/B
      <span class="count">{varianti.length}</span>
    </header>
    <div class="varianti">
      {#each varianti as v (v.id)}
        <button
          type="button"
          class="pill"
          class:active={v.id === promptId}
          onclick={() => apriVariante(v.id)}
          title={v.titolo}
        >
          {v.variant_label}
        </button>
      {/each}
      <button
        type="button"
        class="pill add"
        onclick={apriModaleCreaVariante}
        title="Crea nuova variante di questo prompt"
      >
        + Variante
      </button>
    </div>
    {#if varianti.length > 1}
      <button type="button" class="link" onclick={onApriTabImportVar}>
        Confronta tutte
      </button>
    {/if}
  </section>
</aside>

{#if modaleVariante.aperto}
  <Modale
    titolo="Crea variante"
    sottotitolo={titolo}
    larghezza="sm"
    onChiudi={chiudiModaleVariante}
  >
    <div class="modale-variante-body" onkeydown={gestisciKeydownModale} role="presentation">
      <p class="modale-variante-help">
        Le varianti permettono di mantenere versioni alternative dello stesso
        prompt — utili per A/B test o adattamenti per modelli diversi. La
        nuova variante eredita titolo, descrizione, body, tag e cartella dal
        prompt corrente.
      </p>
      <Field
        etichetta="Etichetta variante"
        hint="Lascia vuoto per assegnazione automatica (b, c, d…)"
        errore={modaleVariante.errore}
      >
        <Input
          bind:valore={modaleVariante.etichetta}
          placeholder="Es. mobile, formale, GPT-4…"
          disabled={modaleVariante.salvataggio}
        />
      </Field>
    </div>
    {#snippet footer()}
      <Button
        variante="ghost"
        onclick={chiudiModaleVariante}
        disabled={modaleVariante.salvataggio}
      >
        Annulla
      </Button>
      <Button
        variante="primary"
        onclick={confermaCreaVariante}
        disabled={modaleVariante.salvataggio}
      >
        {modaleVariante.salvataggio ? "Creazione…" : "Crea variante"}
      </Button>
    {/snippet}
  </Modale>
{/if}

<style>
  .right-rail {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-subtle);
  }

  .sezione {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: var(--sp-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .sezione-h {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: 10px;
    font-weight: var(--fw-semibold);
    color: var(--text-subtle);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
  }

  .count {
    margin-left: auto;
    color: var(--text-subtle);
    font-weight: var(--fw-regular);
    font-size: 11px;
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .lbl {
    font-size: 11px;
    color: var(--text-muted);
  }

  .lbl-sub {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
  }

  .select,
  .input {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-default);
    font-size: var(--fs-sm);
    font-family: var(--font-ui);
    padding: 4px 6px;
  }

  .select:focus,
  .input:focus {
    outline: none;
    border-color: var(--accent-team);
  }

  .tags-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 4px;
  }

  .tag-chip {
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

  .tag-chip::before {
    content: "";
    width: 4px;
    height: 4px;
    border-radius: var(--radius-full);
    background: var(--tag-c, var(--text-subtle));
  }

  .tag-chip button {
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

  .tag-chip button:hover {
    background: var(--bg-canvas);
    color: var(--text-default);
  }

  .suggerimenti {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 6px;
  }

  .sugg-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .sugg {
    border: 1px dashed var(--border-default);
    background: transparent;
    color: var(--text-muted);
    padding: 2px 6px;
    font-size: 11px;
    border-radius: var(--radius-full);
    cursor: pointer;
  }

  .sugg:hover {
    background: var(--bg-overlay);
    color: var(--text-default);
    border-style: solid;
  }

  .segnaposto {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-canvas);
  }

  .ph {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--accent-private);
  }

  .tipo {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-subtle);
  }

  .import-row {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    width: 100%;
    border: 0;
    background: transparent;
    color: var(--text-default);
    font-size: var(--fs-xs);
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
  }

  .import-row:hover {
    background: var(--bg-overlay);
  }

  .import-row code {
    font-family: var(--font-mono);
    color: var(--info);
  }

  .vuoto {
    color: var(--text-subtle);
    font-size: 11px;
    margin: 0;
  }

  .varianti {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .pill {
    border: 1px solid var(--border-subtle);
    background: var(--bg-overlay);
    color: var(--text-muted);
    padding: 2px 8px;
    font-size: 11px;
    border-radius: var(--radius-full);
    cursor: pointer;
    font-family: var(--font-ui);
  }

  .pill:hover {
    color: var(--text-default);
  }

  .pill.active {
    background: var(--accent-team-soft);
    border-color: var(--accent-team);
    color: var(--accent-team-strong);
  }

  .pill.add {
    border-style: dashed;
  }

  .link {
    margin-top: var(--sp-1);
    border: 0;
    background: transparent;
    color: var(--accent-team);
    font-size: 11px;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
    text-align: left;
  }

  /* M3 PR-1: modale Crea variante */
  .modale-variante-body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .modale-variante-help {
    margin: 0;
    font-size: var(--fs-sm);
    color: var(--text-muted);
    line-height: var(--lh-relaxed, 1.5);
  }
</style>
