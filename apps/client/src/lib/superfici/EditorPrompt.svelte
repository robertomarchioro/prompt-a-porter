<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { EditorView } from "@codemirror/view";
  import { basicSetup } from "codemirror";
  import {
    lintMarkers,
    lintMarkersTheme,
    setLintIssues,
  } from "$lib/codemirror/lint-markers";
  import { leggiCategorieDisabilitate } from "$lib/preferenze-linter";
  import { importTokens } from "$lib/codemirror/import-tokens";
  import {
    segnapostoHighlight,
    segnapostoTheme,
  } from "$lib/codemirror/placeholder-highlight";
  import { estraiSegnaposti } from "$lib/template";
  import { Button, Select } from "$lib/components";
  import { MODELLI_TARGET } from "$lib/modelli-target";

  interface TagInfoFE {
    id: string;
    nome: string;
    colore: string;
  }

  interface PromptPerEditor {
    id: string;
    titolo: string;
    descrizione: string;
    body: string;
    visibilita: string;
    target_model?: string | null;
    folder_id?: string | null;
    tags: TagInfoFE[];
  }

  interface CartellaSel {
    id: string;
    path: string;
  }

  interface Props {
    prompt: PromptPerEditor | null;
    onchiudi: () => void;
    onsalvato: () => void;
    oncreaVariante?: (nuovoId: string) => void;
    /// v0.7.0 Step 4: invocato su Ctrl/Cmd+click su un token
    /// `{{import "..."}}` per aprire il prompt importato.
    onapriPrompt?: (promptId: string) => void;
  }

  let { prompt, onchiudi, onsalvato, oncreaVariante, onapriPrompt }: Props =
    $props();

  let creandoVariante = $state(false);
  let erroreVariante = $state("");

  async function creaVariante() {
    if (!prompt) return;
    creandoVariante = true;
    erroreVariante = "";
    try {
      const nuovoId = await invoke<string>("prompt_crea_variante", {
        parentId: prompt.id,
        etichetta: null,
      });
      oncreaVariante?.(nuovoId);
    } catch (e) {
      erroreVariante = String(e);
    } finally {
      creandoVariante = false;
    }
  }

  const contenutoIniziale = prompt?.body ?? "";

  let titolo = $state(prompt?.titolo ?? "");
  let descrizione = $state(prompt?.descrizione ?? "");
  let body = $state(contenutoIniziale);
  let visibilita = $state<"private" | "workspace">(
    (prompt?.visibilita as "private" | "workspace") ?? "private",
  );
  let targetModel = $state<string>(prompt?.target_model ?? "");
  let folderId = $state<string>(prompt?.folder_id ?? "");
  let cartelleDisponibili = $state<CartellaSel[]>([]);
  let tagNomi = $state<string[]>(prompt?.tags.map((t) => t.nome) ?? []);
  let tagInput = $state("");
  let salvando = $state(false);
  let promptId = $state<string | null>(prompt?.id ?? null);
  let statoSalvataggio = $state<"" | "salvataggio" | "salvato">("");

  let editorEl = $state<HTMLDivElement | null>(null);

  let tuttiITag = $state<string[]>([]);
  let timerAutosave: ReturnType<typeof setTimeout>;

  // ─── Tag suggeriti semantici (Fase 3 Step 4) ───
  interface TagSuggerito {
    id: string;
    nome: string;
    colore: string;
    score: number;
    sorgente: "vector" | "frequenza";
  }
  let tagSuggeriti = $state<TagSuggerito[]>([]);
  let timerTagSuggest: ReturnType<typeof setTimeout>;

  // ─── Lint diagnosi (Fase 3 Step 5) ───
  type Severita = "error" | "warning" | "info";
  interface LintIssue {
    code: string;
    severita: Severita;
    messaggio: string;
    linea: number | null;
    colonna: number | null;
  }
  let lintIssues = $state<LintIssue[]>([]);
  let timerLint: ReturnType<typeof setTimeout>;
  let mostraDiagnosi = $state(false);

  // ─── Golden examples (Fase 4 Step 8) ───
  type SimilarityFn = "cosine" | "exact-match" | "regex" | "llm-judge";
  interface Golden {
    id: string;
    prompt_id: string;
    etichetta: string;
    input_vars: string;
    expected_output: string;
    similarity_fn: SimilarityFn;
    soglia_tolleranza: number;
    creato_a: string;
    aggiornato_a: string;
  }
  interface Observation {
    id: string;
    prompt_version_id: string;
    golden_id: string | null;
    provider: string;
    model: string;
    actual_output: string;
    similarita: number | null;
    passed: boolean;
    latenza_ms: number | null;
    tokens_used: number | null;
    costo_stimato: number | null;
    errore: string | null;
    ran_at: string;
    ran_by: string;
  }
  let goldens = $state<Golden[]>([]);
  let mostraTest = $state(false);
  let modificaGolden = $state<Record<string, Golden | null>>({});
  let mostraNuovoGolden = $state(false);
  let nuovoEtichetta = $state("");
  let nuovoInputVars = $state("{}");
  let nuovoExpected = $state("");
  let nuovoSimFn = $state<SimilarityFn>("cosine");
  let nuovoSoglia = $state(0.85);
  let runMessaggio = $state<Record<string, string>>({});
  let runStato = $state<Record<string, "idle" | "running" | "ok" | "ko">>({});
  let ultimaObs = $state<Record<string, Observation>>({});
  let modelOllama = $state("llama3.2");

  async function ricaricaGoldens() {
    if (!promptId) {
      goldens = [];
      return;
    }
    try {
      goldens = await invoke<Golden[]>("golden_lista", { promptId });
    } catch {
      goldens = [];
    }
  }

  $effect(() => {
    // ricarica quando promptId atterra (dopo primo salvataggio).
    if (promptId) ricaricaGoldens();
  });

  async function aggiungiGolden() {
    if (!promptId || !nuovoEtichetta.trim() || !nuovoExpected.trim()) return;
    try {
      await invoke<string>("golden_crea", {
        dati: {
          prompt_id: promptId,
          etichetta: nuovoEtichetta.trim(),
          input_vars: nuovoInputVars.trim() || "{}",
          expected_output: nuovoExpected,
          similarity_fn: nuovoSimFn,
          soglia_tolleranza: nuovoSoglia,
        },
      });
      mostraNuovoGolden = false;
      nuovoEtichetta = "";
      nuovoInputVars = "{}";
      nuovoExpected = "";
      nuovoSimFn = "cosine";
      nuovoSoglia = 0.85;
      await ricaricaGoldens();
    } catch (e) {
      runMessaggio["__nuovo"] = String(e);
    }
  }

  async function salvaModifica(g: Golden) {
    const m = modificaGolden[g.id];
    if (!m) return;
    try {
      await invoke("golden_aggiorna", {
        dati: {
          id: g.id,
          etichetta: m.etichetta.trim(),
          input_vars: m.input_vars.trim() || "{}",
          expected_output: m.expected_output,
          similarity_fn: m.similarity_fn,
          soglia_tolleranza: m.soglia_tolleranza,
        },
      });
      modificaGolden[g.id] = null;
      await ricaricaGoldens();
    } catch (e) {
      runMessaggio[g.id] = String(e);
    }
  }

  async function eliminaGolden(id: string) {
    if (!confirm("Eliminare questo golden? L'azione è reversibile dal DB.")) return;
    try {
      await invoke("golden_elimina", { id });
      await ricaricaGoldens();
    } catch (e) {
      runMessaggio[id] = String(e);
    }
  }

  // ─── Esegui tutti i golden batch (v0.5.0 Step 4) ───
  let batchInCorso = $state(false);
  let batchProgresso = $state<{ fatti: number; totali: number } | null>(null);
  let batchSummary = $state<{
    passed: number;
    failed: number;
    error: number;
  } | null>(null);

  async function eseguiTuttiGolden() {
    if (batchInCorso || goldens.length === 0) return;
    batchInCorso = true;
    batchSummary = null;
    batchProgresso = { fatti: 0, totali: goldens.length };
    let passed = 0;
    let failed = 0;
    let error = 0;
    // Sequenziale per evitare rate limit del provider scelto.
    for (const g of goldens) {
      await eseguiGolden(g.id);
      const stato = runStato[g.id];
      if (stato === "ok") passed++;
      else if (stato === "ko") failed++;
      else error++;
      batchProgresso = {
        fatti: (batchProgresso?.fatti ?? 0) + 1,
        totali: goldens.length,
      };
    }
    batchSummary = { passed, failed, error };
    batchInCorso = false;
    batchProgresso = null;
  }

  async function eseguiGolden(id: string) {
    runStato[id] = "running";
    runMessaggio[id] = "";
    try {
      const obs = await invoke<Observation>("golden_esegui", {
        goldenId: id,
        providerKind: "ollama",
        model: modelOllama.trim() || "llama3.2",
        baseUrl: null,
      });
      ultimaObs[id] = obs;
      runStato[id] = obs.passed ? "ok" : "ko";
      if (obs.errore) {
        runMessaggio[id] = obs.errore;
      } else if (obs.similarita !== null) {
        runMessaggio[id] = `similarità ${obs.similarita.toFixed(3)} · ${obs.latenza_ms ?? "?"}ms`;
      }
    } catch (e) {
      runStato[id] = "ko";
      runMessaggio[id] = String(e);
    }
  }

  const segnaposti = $derived(estraiSegnaposti(body));

  const suggerimentiTag = $derived(
    tagInput.trim()
      ? tuttiITag
          .filter(
            (t) =>
              t.toLowerCase().includes(tagInput.toLowerCase()) &&
              !tagNomi.includes(t),
          )
          .slice(0, 5)
      : [],
  );

  $effect(() => {
    invoke<TagInfoFE[]>("libreria_tag_lista")
      .then((tags) => (tuttiITag = tags.map((t) => t.nome)))
      .catch(() => {});
  });

  $effect(() => {
    invoke<CartellaSel[]>("folder_lista")
      .then((cs) => (cartelleDisponibili = cs))
      .catch(() => {});
  });

  // Tag suggest debounced quando il body cambia. Riceve fallback
  // (sorgente "frequenza") quando la Session embedding non è loaded.
  $effect(() => {
    const corrente = body;
    clearTimeout(timerTagSuggest);
    if (!corrente.trim() || corrente.trim().length < 30) {
      tagSuggeriti = [];
      return;
    }
    timerTagSuggest = setTimeout(() => {
      invoke<TagSuggerito[]>("tags_suggest", { testo: corrente, limit: 5 })
        .then((s) => {
          // Filtro: non suggerire tag già presenti.
          tagSuggeriti = s.filter((t) => !tagNomi.includes(t.nome));
        })
        .catch(() => {
          tagSuggeriti = [];
        });
    }, 700);
  });

  // Lint debounced quando il body cambia.
  // Se il prompt è già salvato (promptId presente), passa l'id al
  // backend in modo che attivi anche le regole IMP* (import non
  // risolto / cicli / depth) sul grafo dei prompt componibili.
  $effect(() => {
    const corrente = body;
    const idCorrente = promptId;
    clearTimeout(timerLint);
    if (!corrente.trim()) {
      lintIssues = [];
      return;
    }
    timerLint = setTimeout(() => {
      // v0.6.0 Step 6: legge le categorie disattivate dall'utente da
      // Impostazioni → Linter (localStorage) e le passa al backend.
      const categorieDisabilitate = leggiCategorieDisabilitate();
      invoke<LintIssue[]>("prompt_lint", {
        body: corrente,
        promptId: idCorrente,
        categorieDisabilitate,
      })
        .then((issues) => {
          lintIssues = issues;
        })
        .catch(() => {
          lintIssues = [];
        });
    }, 600);
  });

  let editorView = $state<EditorView | null>(null);

  $effect(() => {
    if (!editorEl) return;
    const view = new EditorView({
      doc: contenutoIniziale,
      extensions: [
        basicSetup,
        segnapostoHighlight,
        segnapostoTheme,
        lintMarkers,
        lintMarkersTheme,
        importTokens({
          onapri: (promptId) => {
            onapriPrompt?.(promptId);
          },
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            body = update.state.doc.toString();
            pianificaAutosave();
          }
        }),
        EditorView.theme({
          "&": { height: "100%", fontSize: "13px" },
          ".cm-scroller": { overflow: "auto" },
          ".cm-content": {
            fontFamily: "var(--font-mono)",
            minHeight: "200px",
          },
          ".cm-gutters": {
            background: "var(--bg-surface)",
            borderRight: "1px solid var(--border-subtle)",
            color: "var(--text-subtle)",
          },
          "&.cm-focused": { outline: "none" },
        }),
      ],
      parent: editorEl,
    });
    editorView = view;
    return () => {
      view.destroy();
      editorView = null;
    };
  });

  // v0.6.0 Step 3: dispatch dei marker linter ogni volta che lintIssues
  // cambia. CodeMirror aggiorna le decoration in modo dichiarativo via
  // StateEffect, senza ricreare il view.
  $effect(() => {
    const issues = lintIssues;
    if (!editorView) return;
    editorView.dispatch({
      effects: setLintIssues.of(issues),
    });
  });

  function pianificaAutosave() {
    if (!promptId) return;
    clearTimeout(timerAutosave);
    timerAutosave = setTimeout(() => salvaInBackground(), 2000);
  }

  function aggiungiTag(nome?: string) {
    const n = (nome ?? tagInput).trim();
    if (n && !tagNomi.includes(n)) {
      tagNomi = [...tagNomi, n];
      pianificaAutosave();
    }
    tagInput = "";
  }

  function rimuoviTag(indice: number) {
    tagNomi = tagNomi.filter((_, i) => i !== indice);
    pianificaAutosave();
  }

  function gestisciTagKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (suggerimentiTag.length > 0) {
        aggiungiTag(suggerimentiTag[0]);
      } else {
        aggiungiTag();
      }
    } else if (e.key === "Backspace" && !tagInput && tagNomi.length > 0) {
      tagNomi = tagNomi.slice(0, -1);
      pianificaAutosave();
    }
  }

  async function salvaInBackground() {
    if (!titolo.trim() || !body.trim()) return;
    statoSalvataggio = "salvataggio";
    try {
      if (promptId) {
        await invoke("prompt_aggiorna", {
          dati: {
            id: promptId,
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
            target_model: targetModel.trim() || null,
            folder_id: folderId || null,
          },
        });
      } else {
        promptId = await invoke<string>("prompt_crea", {
          dati: {
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
            target_model: targetModel.trim() || null,
            folder_id: folderId || null,
          },
        });
      }
      statoSalvataggio = "salvato";
      setTimeout(() => (statoSalvataggio = ""), 1500);
    } catch {
      statoSalvataggio = "";
    }
  }

  async function salva() {
    if (!titolo.trim() || !body.trim()) return;
    salvando = true;
    clearTimeout(timerAutosave);
    try {
      if (promptId) {
        await invoke("prompt_aggiorna", {
          dati: {
            id: promptId,
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
            target_model: targetModel.trim() || null,
            folder_id: folderId || null,
          },
        });
      } else {
        await invoke<string>("prompt_crea", {
          dati: {
            titolo: titolo.trim(),
            descrizione: descrizione.trim(),
            body: body.trim(),
            visibilita,
            tag_nomi: tagNomi,
            target_model: targetModel.trim() || null,
            folder_id: folderId || null,
          },
        });
      }
      onsalvato();
    } catch (e) {
      console.error("Errore salvataggio:", e);
    } finally {
      salvando = false;
    }
  }

  function renderPreview(testo: string): string {
    const esc = testo
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    return esc.replace(
      /\{\{\s*(\w+)\s*\}\}/g,
      (_, n: string) =>
        `<span class="ph"><span class="br">{{</span>${n}<span class="br">}}</span></span>`,
    );
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onchiudi();
    if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      salva();
    }
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
    aria-label={prompt ? "Modifica prompt" : "Nuovo prompt"}
  >
    <header class="modale-header">
      <h2>{prompt ? "Modifica prompt" : "Nuovo prompt"}</h2>
      <div class="header-azioni">
        {#if prompt && oncreaVariante}
          <Button
            variante="ghost"
            dimensione="sm"
            onclick={creaVariante}
            disabled={creandoVariante}
            title="Crea una variante (B/C/…) di questo prompt"
          >
            {creandoVariante ? "..." : "+ Variante"}
          </Button>
        {/if}
        <Button variante="ghost" dimensione="sm" onclick={onchiudi}>✕</Button>
      </div>
    </header>
    {#if erroreVariante}
      <div class="errore-variante" role="alert">{erroreVariante}</div>
    {/if}

    <div class="modale-body">
      <!-- ── Colonna editor ── -->
      <div class="col-editor">
        <div class="campo">
          <label for="ed-titolo">Titolo</label>
          <input
            id="ed-titolo"
            bind:value={titolo}
            oninput={pianificaAutosave}
            placeholder="Titolo del prompt"
            autofocus
          />
        </div>
        <div class="campo">
          <label for="ed-desc">Descrizione</label>
          <input
            id="ed-desc"
            bind:value={descrizione}
            oninput={pianificaAutosave}
            placeholder="Breve descrizione (opzionale)"
          />
        </div>
        <div class="campo campo-grow">
          <label>Corpo del prompt</label>
          <div class="editor-wrap" bind:this={editorEl}></div>
          {#if lintIssues.length > 0}
            {@const errori = lintIssues.filter((i) => i.severita === "error").length}
            {@const avvisi = lintIssues.filter((i) => i.severita === "warning").length}
            {@const info = lintIssues.filter((i) => i.severita === "info").length}
            <div class="diagnosi">
              <button
                class="diagnosi-toggle"
                onclick={() => (mostraDiagnosi = !mostraDiagnosi)}
                type="button"
              >
                <span class="diagnosi-titolo">Diagnosi</span>
                {#if errori > 0}
                  <span class="diagnosi-pill diagnosi-pill--err"
                    >{errori} errori</span
                  >
                {/if}
                {#if avvisi > 0}
                  <span class="diagnosi-pill diagnosi-pill--warn"
                    >{avvisi} avvisi</span
                  >
                {/if}
                {#if info > 0}
                  <span class="diagnosi-pill diagnosi-pill--info"
                    >{info} info</span
                  >
                {/if}
                <span class="diagnosi-chevron">
                  {mostraDiagnosi ? "▾" : "▸"}
                </span>
              </button>
              {#if mostraDiagnosi}
                <ul class="diagnosi-lista">
                  {#each lintIssues as issue (issue.code + (issue.linea ?? 0) + (issue.colonna ?? 0) + issue.messaggio.slice(0, 32))}
                    <li
                      class="diagnosi-item diagnosi-item--{issue.severita}"
                    >
                      <span class="diagnosi-code">{issue.code}</span>
                      <span class="diagnosi-msg">{issue.messaggio}</span>
                      {#if issue.linea !== null}
                        <span class="diagnosi-pos"
                          >L{issue.linea}{#if issue.colonna !== null}:{issue.colonna}{/if}</span
                        >
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}

          {#if promptId}
            <!-- ── Pannello Test (Fase 4 Step 8e) ── -->
            <div class="test-pannello">
              <button
                class="diagnosi-toggle"
                onclick={() => (mostraTest = !mostraTest)}
                type="button"
              >
                <span class="diagnosi-titolo">Test</span>
                {#if goldens.length > 0}
                  <span class="diagnosi-pill diagnosi-pill--info">
                    {goldens.length} golden
                  </span>
                {/if}
                <span class="diagnosi-chevron">{mostraTest ? "▾" : "▸"}</span>
              </button>

              {#if mostraTest}
                <div class="test-corpo">
                  <div class="test-config">
                    <label class="test-label">
                      Modello Ollama
                      <input
                        type="text"
                        bind:value={modelOllama}
                        placeholder="llama3.2"
                        class="test-input-inline"
                      />
                    </label>
                    <span class="test-hint">
                      Provider remote (Anthropic, OpenAI) in arrivo nello Step 8f.
                    </span>
                    {#if goldens.length > 0}
                      <Button
                        dimensione="sm"
                        variante="primary"
                        disabled={batchInCorso}
                        onclick={eseguiTuttiGolden}
                      >
                        {#if batchInCorso && batchProgresso}
                          Esecuzione {batchProgresso.fatti}/{batchProgresso.totali}…
                        {:else}
                          Esegui tutti ({goldens.length})
                        {/if}
                      </Button>
                    {/if}
                  </div>
                  {#if batchSummary}
                    <div
                      class="test-batch-summary"
                      class:test-batch-summary--ok={batchSummary.failed === 0 &&
                        batchSummary.error === 0}
                      class:test-batch-summary--ko={batchSummary.failed > 0 ||
                        batchSummary.error > 0}
                      role="status"
                    >
                      ✓ {batchSummary.passed} passed
                      {#if batchSummary.failed > 0}
                        · ✗ {batchSummary.failed} failed
                      {/if}
                      {#if batchSummary.error > 0}
                        · ⚠ {batchSummary.error} errore
                      {/if}
                    </div>
                  {/if}

                  {#each goldens as g (g.id)}
                    {@const stato = runStato[g.id] ?? "idle"}
                    {@const isModifica = modificaGolden[g.id] != null}
                    <div class="test-item test-item--{stato}">
                      {#if isModifica}
                        {@const m = modificaGolden[g.id]!}
                        <div class="test-form">
                          <input
                            class="test-input"
                            bind:value={m.etichetta}
                            placeholder="etichetta"
                          />
                          <textarea
                            class="test-textarea"
                            bind:value={m.input_vars}
                            placeholder="JSON variabili (es: {`{}`})"
                            rows="2"
                          ></textarea>
                          <textarea
                            class="test-textarea"
                            bind:value={m.expected_output}
                            placeholder="output atteso"
                            rows="3"
                          ></textarea>
                          <div class="test-form-row">
                            <select
                              bind:value={m.similarity_fn}
                              class="test-select"
                            >
                              <option value="cosine">cosine</option>
                              <option value="exact-match">exact-match</option>
                              <option value="regex">regex</option>
                              <option value="llm-judge" disabled
                                >llm-judge (8f)</option
                              >
                            </select>
                            <input
                              type="number"
                              min="0"
                              max="1"
                              step="0.01"
                              bind:value={m.soglia_tolleranza}
                              class="test-input-inline"
                            />
                            <Button onclick={() => salvaModifica(g)}
                              >Salva</Button
                            >
                            <button
                              class="test-link"
                              onclick={() => (modificaGolden[g.id] = null)}
                              type="button">Annulla</button
                            >
                          </div>
                        </div>
                      {:else}
                        <div class="test-row">
                          <span class="test-icon-stato test-icon-stato--{stato}">
                            {stato === "ok"
                              ? "✓"
                              : stato === "ko"
                                ? "✗"
                                : stato === "running"
                                  ? "…"
                                  : "○"}
                          </span>
                          <span class="test-etichetta">{g.etichetta}</span>
                          <span class="test-meta">
                            {g.similarity_fn} · soglia {g.soglia_tolleranza.toFixed(2)}
                          </span>
                          <button
                            class="test-action"
                            onclick={() => eseguiGolden(g.id)}
                            disabled={stato === "running"}
                            type="button"
                          >
                            {stato === "running" ? "..." : "Esegui"}
                          </button>
                          <button
                            class="test-link"
                            onclick={() => {
                              modificaGolden[g.id] = { ...g };
                            }}
                            type="button">Modifica</button
                          >
                          <button
                            class="test-link test-link--danger"
                            onclick={() => eliminaGolden(g.id)}
                            type="button">Elimina</button
                          >
                        </div>
                        {#if runMessaggio[g.id]}
                          <div class="test-messaggio test-messaggio--{stato}">
                            {runMessaggio[g.id]}
                          </div>
                        {/if}
                        {#if ultimaObs[g.id] && (ultimaObs[g.id].actual_output || ultimaObs[g.id].errore)}
                          <details class="test-details">
                            <summary>Output ricevuto</summary>
                            <pre class="test-output">{ultimaObs[g.id]
                                .actual_output ||
                                ultimaObs[g.id].errore ||
                                ""}</pre>
                          </details>
                        {/if}
                      {/if}
                    </div>
                  {/each}

                  {#if mostraNuovoGolden}
                    <div class="test-item test-item--nuovo">
                      <div class="test-form">
                        <input
                          class="test-input"
                          bind:value={nuovoEtichetta}
                          placeholder="etichetta (es. 'caso comune')"
                        />
                        <textarea
                          class="test-textarea"
                          bind:value={nuovoInputVars}
                          placeholder={'{"variabile":"valore"}'}
                          rows="2"
                        ></textarea>
                        <textarea
                          class="test-textarea"
                          bind:value={nuovoExpected}
                          placeholder="output atteso"
                          rows="3"
                        ></textarea>
                        <div class="test-form-row">
                          <select
                            bind:value={nuovoSimFn}
                            class="test-select"
                          >
                            <option value="cosine">cosine</option>
                            <option value="exact-match">exact-match</option>
                            <option value="regex">regex</option>
                          </select>
                          <input
                            type="number"
                            min="0"
                            max="1"
                            step="0.01"
                            bind:value={nuovoSoglia}
                            class="test-input-inline"
                          />
                          <Button onclick={aggiungiGolden}>Crea</Button>
                          <button
                            class="test-link"
                            onclick={() => (mostraNuovoGolden = false)}
                            type="button">Annulla</button
                          >
                        </div>
                        {#if runMessaggio["__nuovo"]}
                          <div class="test-messaggio test-messaggio--ko">
                            {runMessaggio["__nuovo"]}
                          </div>
                        {/if}
                      </div>
                    </div>
                  {:else}
                    <button
                      class="test-aggiungi"
                      onclick={() => (mostraNuovoGolden = true)}
                      type="button">+ Nuovo golden</button
                    >
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <!-- ── Colonna metadati ── -->
      <div class="col-meta">
        {#if segnaposti.length > 0}
          <div class="meta-sezione">
            <h3>
              Segnaposti <span class="meta-count">{segnaposti.length}</span>
            </h3>
            <div class="segnaposti-lista">
              {#each segnaposti as s}
                <span class="segnaposto-pill">{`{{${s.nome}}}`}</span>
              {/each}
            </div>
          </div>
        {/if}

        <div class="meta-sezione">
          <h3>Tag</h3>
          <div class="tag-input-wrap">
            {#each tagNomi as nome, i}
              <span class="tag-pill">
                {nome}
                <button
                  class="tag-rm"
                  onclick={() => rimuoviTag(i)}
                  type="button">✕</button
                >
              </span>
            {/each}
            <input
              class="tag-input"
              bind:value={tagInput}
              onkeydown={gestisciTagKeydown}
              placeholder={tagNomi.length ? "" : "Aggiungi tag…"}
            />
            {#if suggerimentiTag.length > 0}
              <div class="suggerimenti">
                {#each suggerimentiTag as sug}
                  <button
                    class="suggerimento"
                    onclick={() => aggiungiTag(sug)}
                    type="button"
                  >
                    {sug}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          {#if tagSuggeriti.length > 0}
            <div class="tag-auto-section">
              <span class="tag-auto-label">
                Suggeriti
                {#if tagSuggeriti[0].sorgente === "vector"}
                  <span class="tag-auto-fonte" title="Da ricerca semantica"
                    >semantica</span
                  >
                {:else}
                  <span class="tag-auto-fonte" title="Da tag più frequenti"
                    >frequenza</span
                  >
                {/if}
              </span>
              <div class="tag-auto-pills">
                {#each tagSuggeriti as t (t.id)}
                  <button
                    class="tag-auto-pill"
                    onclick={() => aggiungiTag(t.nome)}
                    title="Score: {t.score.toFixed(2)}"
                    type="button"
                  >
                    + {t.nome}
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <div class="meta-sezione">
          <h3>Modello target</h3>
          <!-- v0.7.0 Step 3: combo input free-text + datalist preset.
               Sostituisce il vecchio Select per consentire modelli custom
               (es. claude-opus-5, gpt-6, modelli locali) oltre ai preset. -->
          <input
            class="meta-target-model-input"
            type="text"
            list="modelli-target-preset"
            bind:value={targetModel}
            oninput={() => pianificaAutosave()}
            placeholder="Preset o nome custom (es. claude-opus, gpt-4o, llama3.2…)"
            autocomplete="off"
          />
          <datalist id="modelli-target-preset">
            {#each MODELLI_TARGET as m (m.value)}
              <option value={m.value}>{m.label}</option>
            {/each}
          </datalist>
        </div>

        <div class="meta-sezione">
          <h3>Cartella</h3>
          <Select bind:valore={folderId} onchange={() => pianificaAutosave()}>
            <option value="">Nessuna (root)</option>
            {#each cartelleDisponibili as c (c.id)}
              <option value={c.id}>{c.path}</option>
            {/each}
          </Select>
        </div>

        <div class="meta-sezione">
          <h3>Visibilità</h3>
          <div class="vis-toggle">
            <button
              class="vis-btn"
              class:vis-btn--attivo={visibilita === "private"}
              onclick={() => {
                visibilita = "private";
                pianificaAutosave();
              }}
              type="button">Privato</button
            >
            <button
              class="vis-btn"
              class:vis-btn--attivo={visibilita === "workspace"}
              onclick={() => {
                visibilita = "workspace";
                pianificaAutosave();
              }}
              type="button">Team</button
            >
          </div>
        </div>

        <div class="meta-sezione meta-sezione-grow">
          <h3>Anteprima</h3>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="preview-box">
            {@html renderPreview(body)}
          </div>
        </div>
      </div>
    </div>

    <footer class="modale-footer">
      {#if statoSalvataggio}
        <span class="autosave-status">
          {statoSalvataggio === "salvataggio" ? "Salvataggio…" : "Salvato ✓"}
        </span>
      {/if}
      <Button variante="ghost" onclick={onchiudi}>Annulla</Button>
      <Button
        variante="primary"
        onclick={salva}
        disabled={!titolo.trim() || !body.trim() || salvando}
      >
        {salvando ? "Salvataggio…" : "Salva"}
      </Button>
    </footer>
  </div>
</div>

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
    width: min(960px, 96vw);
    height: min(720px, 92vh);
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

  .header-azioni {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .errore-variante {
    padding: var(--sp-2) var(--sp-5);
    color: var(--danger);
    background: var(--danger-soft);
    border-bottom: 1px solid var(--border-subtle);
    font-size: var(--fs-sm);
  }

  .modale-body {
    flex: 1;
    display: grid;
    grid-template-columns: 1.3fr 1fr;
    overflow: hidden;
  }

  /* ── Colonna editor ── */

  .col-editor {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-4) var(--sp-5);
    overflow: hidden;
    border-right: 1px solid var(--border-subtle);
  }

  .col-meta {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding: var(--sp-4) var(--sp-5);
    overflow-y: auto;
  }

  .campo {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .campo label {
    font-size: 11px;
    font-weight: var(--fw-medium);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    font-family: var(--font-mono);
  }

  .campo input {
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

  .campo input:focus {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .campo input::placeholder {
    color: var(--text-subtle);
  }

  .campo-grow {
    flex: 1;
    min-height: 0;
  }

  .editor-wrap {
    flex: 1;
    min-height: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-input);
  }

  /* ── Metadati (colonna destra) ── */

  .meta-sezione {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .meta-sezione h3 {
    margin: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .meta-target-model-input {
    height: 34px;
    padding: 8px 12px;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }
  .meta-target-model-input:hover {
    border-color: var(--border-strong);
  }
  .meta-target-model-input:focus {
    outline: none;
    border-color: var(--accent-team);
    box-shadow: 0 0 0 3px var(--accent-team-soft);
  }

  .meta-count {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-weight: normal;
  }

  .segnaposti-lista {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .segnaposto-pill {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-weight: var(--fw-medium);
  }

  /* ── Tag picker ── */

  .tag-input-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    min-height: 36px;
    align-items: center;
    cursor: text;
    position: relative;
    transition: border-color var(--motion-fast);
  }

  .tag-input-wrap:focus-within {
    border-color: var(--accent-team);
    box-shadow: 0 0 0 2px var(--accent-team-soft);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background: var(--bg-overlay);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 6px;
  }

  .tag-rm {
    appearance: none;
    border: none;
    background: none;
    padding: 0;
    margin-left: 2px;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 10px;
    line-height: 1;
  }
  .tag-rm:hover {
    color: var(--danger);
  }

  .tag-input {
    border: none;
    outline: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-strong);
    flex: 1;
    min-width: 80px;
    padding: 0;
  }
  .tag-input::placeholder {
    color: var(--text-subtle);
  }

  .suggerimenti {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10;
    overflow: hidden;
  }

  .suggerimento {
    appearance: none;
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-default);
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
  }
  .suggerimento:hover {
    background: var(--bg-surface);
  }

  /* ── Visibilità toggle ── */

  .vis-toggle {
    display: flex;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .vis-btn {
    appearance: none;
    flex: 1;
    padding: var(--sp-2) var(--sp-3);
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
    color: var(--text-muted);
    background: var(--bg-input);
    border: none;
    cursor: pointer;
    transition: all var(--motion-fast);
  }
  .vis-btn + .vis-btn {
    border-left: 1px solid var(--border-default);
  }
  .vis-btn--attivo {
    color: var(--text-strong);
    background: var(--bg-overlay);
    font-weight: var(--fw-medium);
  }

  /* ── Anteprima ── */

  .meta-sezione-grow {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .preview-box {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    line-height: var(--lh-loose);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--sp-3);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-y: auto;
    min-height: 120px;
  }

  :global(.preview-box .ph) {
    display: inline;
    font-family: var(--font-mono);
    color: var(--accent-private);
    background: var(--accent-private-soft);
    border-radius: var(--radius-sm);
    padding: 1px 6px;
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }

  :global(.preview-box .ph .br) {
    opacity: 0.55;
    font-weight: var(--fw-regular);
  }

  /* ── Footer ── */

  .modale-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-3);
    padding: var(--sp-3) var(--sp-5);
    border-top: 1px solid var(--border-subtle);
  }

  .autosave-status {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    margin-right: auto;
  }

  /* ── Tag suggeriti (Fase 3 Step 4) ── */

  .tag-auto-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: var(--sp-3);
    padding-top: var(--sp-3);
    border-top: 1px dashed var(--border-subtle);
  }

  .tag-auto-label {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tag-auto-fonte {
    font-family: var(--font-mono);
    text-transform: lowercase;
    letter-spacing: 0;
    background: var(--bg-input);
    border-radius: 999px;
    padding: 1px 6px;
    color: var(--text-muted);
  }

  .tag-auto-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tag-auto-pill {
    background: var(--bg-input);
    border: 1px dashed var(--border-default);
    color: var(--text-default);
    border-radius: 999px;
    padding: 2px 10px;
    font-size: var(--fs-xs);
    cursor: pointer;
    transition:
      background var(--motion-fast),
      color var(--motion-fast),
      border-color var(--motion-fast);
  }
  .tag-auto-pill:hover {
    background: var(--accent-team-soft, rgba(80, 120, 200, 0.15));
    border-color: var(--accent-team);
    color: var(--accent-team);
  }

  /* ── Diagnosi lint (Fase 3 Step 5) ── */

  .diagnosi {
    margin-top: var(--sp-2);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
  }

  .diagnosi-toggle {
    width: 100%;
    background: transparent;
    border: 0;
    color: var(--text-strong);
    padding: 8px 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--fs-sm);
  }

  .diagnosi-titolo {
    font-weight: var(--fw-semibold);
  }

  .diagnosi-chevron {
    margin-left: auto;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .diagnosi-pill {
    font-size: var(--fs-xs);
    padding: 1px 8px;
    border-radius: 999px;
    font-family: var(--font-mono);
  }
  .diagnosi-pill--err {
    background: rgba(220, 80, 80, 0.18);
    color: #c83;
  }
  .diagnosi-pill--warn {
    background: rgba(220, 160, 60, 0.18);
    color: #c83;
  }
  .diagnosi-pill--info {
    background: var(--bg-canvas);
    color: var(--text-muted);
  }

  .diagnosi-lista {
    list-style: none;
    margin: 0;
    padding: 0 0 8px 0;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .diagnosi-item {
    display: grid;
    grid-template-columns: 70px 1fr auto;
    align-items: baseline;
    gap: 8px;
    padding: 6px 12px;
    font-size: var(--fs-sm);
    border-bottom: 1px solid var(--border-subtle);
  }
  .diagnosi-item:last-child {
    border-bottom: 0;
  }
  .diagnosi-item--error {
    background: rgba(220, 80, 80, 0.05);
  }
  .diagnosi-item--warning {
    background: rgba(220, 160, 60, 0.05);
  }
  .diagnosi-item--info {
    background: transparent;
  }

  .diagnosi-code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-muted);
  }

  .diagnosi-msg {
    color: var(--text-default);
  }

  .diagnosi-pos {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
  }

  /* ── Test pannello (Fase 4 Step 8e) ── */
  .test-pannello {
    margin-top: var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    overflow: hidden;
  }

  .test-corpo {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-2);
  }

  .test-config {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-bottom: var(--space-2);
    border-bottom: 1px dashed var(--border-subtle);
    font-size: var(--fs-sm);
  }

  .test-label {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    color: var(--text-default);
  }

  .test-hint {
    color: var(--text-subtle);
    font-size: var(--fs-xs);
  }

  .test-batch-summary {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    background: var(--bg-overlay);
    color: var(--text-strong);
  }
  .test-batch-summary--ok {
    background: var(--success-soft);
    color: var(--success);
  }
  .test-batch-summary--ko {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .test-input,
  .test-input-inline {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    background: var(--bg-elevated);
    color: var(--text-default);
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
  }

  .test-input {
    width: 100%;
  }

  .test-input-inline {
    width: 80px;
  }

  .test-textarea {
    width: 100%;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    background: var(--bg-elevated);
    color: var(--text-default);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    resize: vertical;
  }

  .test-select {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    background: var(--bg-elevated);
    color: var(--text-default);
    font-size: var(--fs-sm);
  }

  .test-item {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs);
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    background: var(--bg-elevated);
  }

  .test-item--ok {
    border-color: var(--accent-success, #6cb86c);
  }

  .test-item--ko {
    border-color: var(--accent-danger, #d9534f);
  }

  .test-item--running {
    border-color: var(--accent-warn, #d2a85f);
  }

  .test-item--nuovo {
    border-style: dashed;
  }

  .test-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .test-icon-stato {
    width: 18px;
    text-align: center;
    font-weight: bold;
  }

  .test-icon-stato--ok {
    color: var(--accent-success, #6cb86c);
  }

  .test-icon-stato--ko {
    color: var(--accent-danger, #d9534f);
  }

  .test-icon-stato--running {
    color: var(--accent-warn, #d2a85f);
  }

  .test-icon-stato--idle {
    color: var(--text-subtle);
  }

  .test-etichetta {
    flex: 1 1 auto;
    font-weight: 500;
    color: var(--text-default);
  }

  .test-meta {
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    font-family: var(--font-mono);
  }

  .test-action {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-xs);
    background: var(--bg-surface);
    color: var(--text-default);
    cursor: pointer;
    font-size: var(--fs-sm);
  }

  .test-action:hover:not(:disabled) {
    background: var(--bg-elevated);
  }

  .test-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .test-link {
    background: none;
    border: none;
    color: var(--text-subtle);
    font-size: var(--fs-xs);
    cursor: pointer;
    padding: var(--space-1);
  }

  .test-link:hover {
    color: var(--text-default);
    text-decoration: underline;
  }

  .test-link--danger:hover {
    color: var(--accent-danger, #d9534f);
  }

  .test-messaggio {
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
    color: var(--text-subtle);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-xs);
  }

  .test-messaggio--ok {
    color: var(--accent-success, #6cb86c);
  }

  .test-messaggio--ko {
    color: var(--accent-danger, #d9534f);
  }

  .test-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .test-form-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .test-aggiungi {
    align-self: flex-start;
    background: none;
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-xs);
    color: var(--text-subtle);
    padding: var(--space-1) var(--space-2);
    cursor: pointer;
    font-size: var(--fs-sm);
  }

  .test-aggiungi:hover {
    color: var(--text-default);
    border-color: var(--text-default);
  }

  .test-details {
    margin-top: var(--space-1);
    font-size: var(--fs-xs);
  }

  .test-details summary {
    cursor: pointer;
    color: var(--text-subtle);
  }

  .test-output {
    margin-top: var(--space-1);
    padding: var(--space-2);
    background: var(--bg-surface);
    border-radius: var(--radius-xs);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 200px;
    overflow-y: auto;
  }
</style>
