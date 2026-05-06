# Todo Fase 3 — Intelligenza & Authoring

> **Stato**: ✅ **chiusa al 100%**. Tag `v0.3.0` rilasciato il 2026-05-06.
>
> Tutti gli 11 step della roadmap completati. Sub-step / decisioni discrezionali rimasti aperti sono stati spostati in [`rinvii.md`](./rinvii.md) come candidati per `v0.5.0`. Il deliverable finale è la draft release pubblicata: https://github.com/robertomarchioro/prompt-a-porter/releases/tag/v0.3.0.

## Direzione generale del progetto

Prompt a Porter è una libreria locale-first per prompt AI. Tutte le scelte tecniche seguono tre vincoli non negoziabili:

1. **I dati restano sull'utente.** Vault cifrato locale, feature cloud opt-in, niente telemetria.
2. **Niente lock-in.** Formati aperti (Markdown, JSON), licenza AGPL 3.0, export sempre disponibile, schema dati documentato.
3. **Integrazione via standard.** MCP, OIDC, OpenAPI 3.1, Native Messaging — niente API proprietarie chiuse.

Il progetto attraversa 5 fasi: dall'app standalone (Fase 1, chiusa) alle fondamenta solide e integrabili (Fase 2), all'intelligenza assistiva tutta locale (Fase 3, questa), ai workflow avanzati con qualità misurabile (Fase 4), all'ecosistema enterprise opt-in (Fase 5 → v1.0.0).

## Direzione di Fase 3

Fase 3 trasforma PaP da **biblioteca passiva a sistema attivo**. Tutto deve girare client-side senza dipendenze cloud, essere opt-in, mantenere il dato in chiaro locale, e degradare con grazia se i modelli non sono disponibili.

Il valore distintivo di questa fase è l'allineamento con la filosofia **AI-native ma local-first**: ricerca semantica via embeddings ONNX locali, prompt componibili come moduli software, linting proattivo, organizzazione in cartelle. Quello che altri prompt manager fanno via cloud, PaP lo fa offline sul tuo laptop.

### Filosofia di Fase

> "Pegaso è bellissimo da vedere, ma vola solo se Bellerofonte sa cavalcarlo."

L'AI in PaP è uno strumento di scoperta e qualità, non di sostituzione. Ogni feature in questa fase deve:

1. **Funzionare offline** sul client desktop (zero dipendenze cloud)
2. Essere **opt-in** (utente attiva esplicitamente in Impostazioni)
3. Mantenere **dato in chiaro** locale, embeddings server-side opzionali
4. Avere **degradazione graziosa**: se l'embedding model non è disponibile, ricerca FTS5 continua a funzionare

---

## Step 0 — Prerequisiti

- [x] Fase 2 chiusa: `v0.2.0-foundations` taggata 2026-05-04, AGPL 3.0 attiva, MCP+CLI funzionanti
- [x] Modello dati supporta `TargetModel` (anticipato in v0.2.1 PR #23)
- [x] Decisione strategica embeddings: **client-side puro**. Server-side cache rinviata in `rinvii.md` § Fase 5 (decisione discrezionale 2)
- [x] Branch lavorato direttamente su feature branch da `main` (no fase-3 long-running)

---

## Scope Feature Fase 3

| # | Step | Modulo |
|---|------|--------|
| 1 | ONNX Runtime + modello embeddings | client (server opzionale) |
| 2 | sqlite-vec integration + tabella embeddings | client |
| 3 | Ricerca ibrida FTS5 + vettoriale | client |
| 4 | Auto-suggerimento tag su nuovo prompt | client |
| 5 | Linting prompt (lunghezza, segnaposti, PII, stile) | client |
| 6 | Modello-target dichiarato + filtri | client + server |
| 7 | Cartelle (organizzazione gerarchica, no ACL) | client + server |
| 8 | Prompt componibili `{{import "..."}}` | client |
| 9 | Statistiche di qualità prompt | client |

---

## Step 1 — Setup ONNX Runtime + modello embeddings

- [x] Crate `ort 2.0.0-rc.12` con `default-features=false + load-dynamic + api-23 + ndarray` (api-23 evita bug VitisAI di api-24)
- [x] Modello: **`paraphrase-multilingual-MiniLM-L12-v2`** (118 MB ONNX, 384 dim). Spike 3 v1+v2 confermano scelta. Vedi `docs/architettura/decisioni/embedding-model.md`
- [x] **Download on-demand** al primo uso, cache in `${data_dir}/models/multilingual-MiniLM-L12-v2/`. libonnxruntime in `${data_dir}/onnxruntime/<version>/`
- [x] Comando Tauri `embeddings_init` (download + Session ort), `embeddings_status`, `embeddings_compute`
- [x] Performance: ~25-30 ms per embedding query MiniLM su CPU desktop (vedi `docs/operativo/bench-ricerca-ibrida.md`)
- [x] Auto-init Session al boot client se modello già su disco (PR #47, no click manuale)
- [x] Idle-unload Session dopo soglia configurabile (PR #51, default 5 min)

## Step 2 — sqlite-vec integration

- [x] Crate `sqlite-vec 0.1` registrato come **auto-extension** statica via `sqlite3_auto_extension` con `std::sync::Once` (compatibilità con SQLCipher verificata in Spike 1)
- [x] Migration **V005** `embeddings.sql` (PromptsEmbeddings vec0 384 dim) e **V006** `tag_embeddings.sql` (TagsEmbeddings vec0)
- [x] Hook in `editor::aggiorna_embedding_prompt` su create/update prompt — fallback graceful se Session None (PR #49)
- [x] Backfill `embeddings_backfill` Tauri command con batch + progress events
- [x] Hook delete: pulisce embeddings dal vec0 al tombstone

## Step 3 — Ricerca ibrida (semantica + FTS5)

- [x] **Reciprocal Rank Fusion (RRF) pesata**: `score(d) = (1-α)·1/(k_rrf+rank_lex) + α·1/(k_rrf+rank_sem)` con `K_RRF=60`, `POOL_SIZE=50`
- [x] Comando Tauri `prompt_cerca_ibrida(query, limit, alpha)`, alpha clamped [0,1]
- [x] Default `alpha = 0.5`, configurabile in Impostazioni > Ricerca semantica (slider con preset Lessicale/Bilanciato/Semantico)
- [x] **Fallback automatico**: se Session None, `cerca_semantica` ritorna `vec![]` → degrada a FTS-only senza errore (PR #49)
- [x] **UI Command Palette**: chip lex/sem accanto a ogni risultato per mostrare in quale pipeline matcha
- [x] Bench P95 < 100 ms su 10k prompt: misurato 8.29 ms su lex+sem+RRF (PR #50)

## Step 4 — Auto-suggerimento tag

Quando l'utente crea un nuovo prompt, suggerire tag pertinenti.

- [x] Embedding tag dal nome (`TagsEmbeddings` vec0 V006), confronto cosine via L2 distance da `search_nearest_tags`
- [x] Comando Tauri `tags_suggest(testo, limit)` → `[{tag, score, sorgente}]`
- [x] **UI Editor**: chip cliccabili top-5 sotto il tag picker
- [x] Fallback: se `< MIN_TAG_PER_SEMANTIC=10` tag con embedding, oppure Session None → `tag_frequenti`
- [x] Soglia distanza L2 ≤ 1.0 (≈ cosine ≥ 0.5)
- [x] UX non-autoritativa: chip cliccabili per aggiungere, ignorabili

## Step 5 — Linting prompt

Avvisi proattivi che aiutano a scrivere prompt migliori, senza essere paternalistici.

**Categorie di lint**:

| Codice | Descrizione | Severità |
|--------|-------------|----------|
| `LEN001` | Body troppo lungo (> 4000 char) — possibile spreco token | warning |
| `LEN002` | Body troppo corto (< 30 char) — probabilmente incompleto | info |
| `PH001` | Segnaposto malformato (`{nome}` invece di `{{nome}}`) | error |
| `PH002` | Segnaposto definito ma non usato nel body | warning |
| `PH003` | Nome segnaposto contiene caratteri speciali | warning |
| `IMP001` | Import non risolto (`{{import "missing/path"}}`) | error |
| `IMP002` | Ciclo di import rilevato | error |
| `IMP003` | Profondità import > 5 livelli | warning |
| `PII001` | Possibile email rilevata nel body (regex) | warning |
| `PII002` | Possibile codice fiscale/SSN rilevato | warning |
| `PII003` | Possibile numero di carta di credito (Luhn check) | error |
| `PII004` | Possibile API key o token (pattern OpenAI, Anthropic, AWS, GitHub) | error |
| `STY001` | Body con molte ripetizioni (n-gram analysis) | info |
| `STY002` | Mancanza di istruzioni chiare (manca verbo imperativo nelle prime righe) | info |

- [x] Modulo `linting.rs` con 11 regole su 14: LEN001/002, PH001/003, PII001/003/004, STY001, IMP001/002/003
- [x] Comando Tauri `prompt_lint(body, prompt_id?)` → `[{code, severita, messaggio, linea, colonna}]`
- [x] **UI**: pannello Diagnosi collapsible nell'editor (PR #45)
- [x] Severità Error visibile in UI (non blocca salvataggio: l'utente decide se ignorare)
- [x] Regole IMP001/002/003 DB-aware con cycle detection + depth check (PR #48)
- 📋 **Inline marker CodeMirror 6** sui punti incriminati → rinviato a `v0.5.0` (`rinvii.md`)
- 📋 **Configurazione per-categoria** in Impostazioni → rinviato (oggi sempre attivo)
- 📋 **PH002 / PII002 / STY002** non implementate per scelta (semantica ambigua / regex IT complessa / NLP IT-EN troppo fragile) — vedi `docs/utente/linting-regole.md`

## Step 6 — Modello-target dichiarato

> Anticipato in `v0.2.1` (PR #23) come quick win di Fase 3.

- [x] Campo `Prompts.TargetModel` esistente in V001
- [x] 9 preset in `apps/client/src/lib/modelli-target.ts`
- [x] **UI Editor**: dropdown sopra Visibilità, autosave-aware
- [x] **UI Libreria**: gruppo "Modello target" in sidebar, badge nel detail panel
- 📋 **Custom free-text target** (oltre i 9 preset) — `rinvii.md`
- 📋 **Server endpoint `?target=...`** — Fase 5 (server team in produzione)

## Step 7 — Cartelle (organizzazione gerarchica)

Tag e cartelle coesistono ma servono scopi diversi: i **tag** sono etichette trasversali (un prompt può averne 5), le **cartelle** sono ubicazione canonica (un prompt sta in 1 sola cartella). Modelli ortogonali.

In Fase 3 le cartelle sono **solo organizzative**, senza ACL. I permessi per cartella arrivano in Fase 4 con il workflow di approvazione.

**Schema**:

```sql
CREATE TABLE Folders (
    Id              TEXT PRIMARY KEY,
    WorkspaceId     TEXT NOT NULL REFERENCES Workspaces(Id),
    ParentFolderId  TEXT REFERENCES Folders(Id),  -- NULL = root del workspace
    Name            TEXT NOT NULL,
    Path            TEXT NOT NULL,                -- denormalizzato: "/marketing/email/cold"
    CreatedAt       TEXT NOT NULL,
    UpdatedAt       TEXT NOT NULL,
    DeletedAt       TEXT
);
CREATE INDEX idx_folders_workspace_path ON Folders(WorkspaceId, Path);

ALTER TABLE Prompts ADD COLUMN FolderId TEXT REFERENCES Folders(Id);  -- NULL = root del workspace
```

> Anticipato in `v0.2.1` (PR #25 backend, PR #26 D&D + polish).

- [x] Migration **V004** schema `Folders` + `Prompts.FolderId`
- [x] 6 Tauri command: `folder_lista/crea/rinomina/sposta/elimina` + `prompt_sposta`
- [x] Path denormalizzato + rinomina cascata in transazione (helper `atomicamente`)
- [x] Anti-ciclo: bloccato spostamento dentro sé stessi o discendenti
- [x] Soft-delete cascade
- [x] **UI Sidebar Libreria**: tree gerarchico, "Senza cartella" come voce speciale
- [x] **Drag & drop** + filter chip + rinomina inline (PR #26)
- [x] Stress test 100 cartelle / depth 5 / invariante Path↔ParentFolderId (PR #52)
- 📋 **Esporta singola cartella** (oggi solo intero vault) — `rinvii.md`
- 📋 **`/sync/folders` endpoint** — Fase 5

## Step 8 — Prompt componibili `{{import "..."}}`

Trasforma il vault da "lista di prompt" a **sistema modulare di prompt componibili**. Power user e team che gestiscono famiglie di prompt evitano duplicazione, manutenibili come codice software.

**Sintassi** (coerente con segnaposti `{{...}}` esistenti):

```
{{import "sistema/ruolo-esperto" with ambito="finanza"}}

Analizza il bilancio fornito e identifica anomalie.

{{import "comune/output-json-strict" with schema=`
  {
    "anomalie": [{ "campo": "string", "severita": "low|med|high" }]
  }
`}}

Bilancio: {{bilancio}}
```

Dove `sistema/ruolo-esperto` è un altro prompt nella libreria (lookup per `Path` cartella + `Title` slug, o per `Id` se path ambiguo). Le variabili dell'import (`with`) si combinano con i segnaposti del prompt importato.

**Schema** (no nuove tabelle, tabella di indice per dependency graph):

```sql
CREATE TABLE PromptImports (
    ParentPromptId  TEXT NOT NULL REFERENCES Prompts(Id),
    ImportedPath    TEXT NOT NULL,                 -- "sistema/ruolo-esperto"
    ImportedPromptId TEXT REFERENCES Prompts(Id),  -- risolto, NULL se non risolvibile
    PinnedVersion   INTEGER,                       -- NULL = follow latest
    Position        INTEGER NOT NULL,              -- ordine nel body
    PRIMARY KEY (ParentPromptId, Position)
);
CREATE INDEX idx_imports_imported ON PromptImports(ImportedPromptId);  -- per "chi importa X"
```

- [x] Parser regex `{{import "..."}}` in `prompt_componibili::parse_imports` (no `with` né `version=` in v0.3)
- [x] Comando Tauri `prompt_compila(id)` — espansione ricorsiva con cycle/depth detection
- [x] **Resolver** `resolve_path`: cartella+titolo, fallback titolo solo. Path "/marketing/email/cold" → match esatto Folders.Path + Title NOCASE
- [x] **Depth-limit 5** + **MAX_OUTPUT_BYTES 1MB** anti-bomba di compilazione
- [x] IMP001 (non risolto), IMP002 (ciclo), IMP003 (depth) lint rules DB-aware (PR #48)
- [x] Tabella `PromptImports` (V007) come grafo dipendenze, popolata su save
- [x] Espansione live in `CompilatorePrompt.svelte` (PR #46)
- 📋 **Sintassi `with k=v`** per variabili scopate per import — `rinvii.md`
- 📋 **Pinning a versione** `{{import "x" version=N}}` — `rinvii.md`, schema `PromptVersions` già pronto
- 📋 **Editor UI doppia vista** Sorgente/Compilato (oggi separato in CompilatorePrompt) — `rinvii.md`
- 📋 **Hover preview import** + **ctrl+click navigazione** — `rinvii.md`
- 📋 **Cross-prompt linting** "questo prompt è importato da N altri" — `rinvii.md`
- 📋 **Markdown export con front-matter `imports`** per riproducibilità — `rinvii.md`

## Step 9 — Statistiche qualità prompt

> Anticipato in `v0.2.1` (PR #24).

- [x] Vista `Insight.svelte` con icona dedicata
- [x] Top prompt usati 30g, candidati cleanup (>90g), distribuzioni per tag/target/visibilità
- [x] Charts: SVG inline custom (no chart.js)
- [x] Privacy: aggregazione locale, disclaimer esplicito
- 📋 **Prompt più importati** — atterrabile post Fase 3 ora che Step 8 è chiuso, `rinvii.md`
- 📋 **Distribuzione per cartella** — atterrabile post Fase 3 ora che Step 7 è chiuso, `rinvii.md`
- 📋 **Distribuzione per autore** — Fase 5 (multi-user team)
- 📋 **Lint health %** + top categorie — atterrabile post Fase 3, `rinvii.md`

## Step 10 — Quality gate Fase 3

- [x] Coverage gate 60% line via cargo-llvm-cov nel CI (PR #53). Roadmap esplicita verso 70% in `docs/operativo/coverage.md`. Coverage attuale: 60.12%
- [x] Benchmark P95 ricerca ibrida 10k prompt: 8.29 ms misurati (PR #50, target <100 ms con margine ~2.5x includendo encoding query)
- [x] Idle-unload Session embeddings configurabile (PR #51, default 5 min, 0=off)
- [x] Dataset generator riproducibile in `examples/genera_dataset.rs` (PR #50)
- [x] Grace degradation verificata: backfill skippa se Session None invece di crashare (PR #49) + 5 smoke test
- [x] Stress test cartelle: 100 cartelle, depth 5, invariante Path↔ParentFolderId validato (PR #52)

## Step 11 — Documentazione e release

- [x] `docs/utente/ricerca-semantica.md` (questa PR)
- [x] `docs/utente/linting-regole.md` (questa PR)
- [x] `docs/utente/cartelle.md` (questa PR)
- [x] `docs/utente/prompt-componibili.md` (questa PR)
- [x] Aggiorna `docs/architettura/schema-dati.md` con `Folders`, `PromptsEmbeddings`, `TagsEmbeddings`, `PromptImports`, V004-V007
- [x] Aggiorna `docs/architettura/overview.md` con flusso ricerca ibrida + embedding lifecycle + resolver import
- [x] CHANGELOG `v0.3.0`
- [x] Tag `v0.3.0` rilasciato 2026-05-06 con build cross-OS + 8 asset (Linux deb/rpm/AppImage, macOS arm64 dmg/tar.gz, Windows NSIS/MSI/portable)

---

## Decisioni discrezionali

1. **Modello embedding**: ✅ deciso in Spike 3 v1 (2026-05-04), **confermato in v2 (2026-05-05)** dopo valutazione di alternative 2024-2025 — `paraphrase-multilingual-MiniLM-L12-v2` (118 MB ONNX). Vedi `docs/architettura/decisioni/embedding-model.md`. EmbeddingGemma-300m documentato come alternativa futura se vincoli size/perf si rilassano.
2. **Cache embeddings server-side per workspace team**: 📋 **rinviata a Fase 5** (vedi `rinvii.md` § 7). In `v0.3.0` tutti gli embedding sono client-side puri.
3. **Linting PII block-by-default o warn-only?**: ✅ deciso **warn-by-default** in v0.3. Block-mode per workspace E2E rinviato a Fase 5.
4. **Sintassi import**: ✅ `{{import "..."}}` (no `with` / `version=` in v0.3). Estensioni rinviate a Fase 4.
5. **Riload automatico Session post idle-unload**: 📋 rinviato (vedi `rinvii.md`). In `v0.3.0` la ricerca cade su FTS-only fino al riavvio del client se la Session è stata droppata.

---

## Riferimenti

- Fase 2 (precedente): `docs/roadmap/fase-2-foundations.md`
- Fase 4 (prossima): `docs/roadmap/fase-4-workflow.md`
- Modelli embedding: https://huggingface.co/BAAI/bge-small-en-v1.5
- sqlite-vec: https://github.com/asg017/sqlite-vec
- ONNX Runtime Rust: https://github.com/pykeio/ort
