# Todo Fase 3 — Intelligenza & Authoring

> **Deliverable finale**: tag release `v0.3.0`.

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

- [ ] Fase 2 chiusa: `v0.2.0` taggata, AGPL 3.0 attiva, MCP+CLI funzionanti, auto-update sperimentato
- [ ] Modello dati supporta `TargetModel` (anticipato in Fase 2 dentro lo schema export)
- [ ] Decisione strategica embeddings: **client-side puro** (consigliato) o **server-side opzionale** per workspace team
- [ ] Crea branch `fase-3` da `main`

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

- [ ] Aggiungi crate `ort` (ONNX Runtime Rust binding) al client desktop
- [ ] Modello scelto: **`paraphrase-multilingual-MiniLM-L12-v2`** (~118 MB ONNX, 384 dim, multilingue forte). Decisione presa in Spike 3 v1 (2026-05-04) e **confermata in v2 (2026-05-05)**: recall@5 97.5% sul mix IT/EN. Vedi `docs/architettura/decisioni/embedding-model.md`. EmbeddingGemma-300m (2025) valutato in v2 ma scartato per trade-off non giustificati (+180 MB download, 3.7× tempo per-embedding) per il modesto guadagno di +2.5 pt recall@5.
- [ ] **Bundling vs download**:
  - Bundle nel binario: +30-80 MB. Pro: zero setup. Contro: ogni upgrade modello richiede re-release.
  - Download al primo uso: scarica da repo HuggingFace o mirror. Pro: binario snello. Contro: serve connessione iniziale.
  - **Decisione consigliata**: download on-demand con cache in `~/.local/share/prompt-a-porter/models/`
- [ ] Comando Tauri `embeddings_init()` → carica modello in memoria (lazy, primo uso)
- [ ] Comando Tauri `embeddings_compute(text: String)` → returns `Vec<f32>` (384 dim)
- [ ] Performance target: < 50ms per embedding di un prompt medio (1000 token) su CPU moderna
- [ ] Threading: ONNX Runtime su thread dedicato, non blocca main loop
- [ ] Test: 1000 embeddings in batch < 30s su laptop standard

## Step 2 — sqlite-vec integration

- [ ] Aggiungi estensione `sqlite-vec` al setup SQLite client (carica come dynamic library al boot del vault)
- [ ] Migration v3 con nuova tabella virtuale:
  ```sql
  CREATE VIRTUAL TABLE PromptsEmbeddings USING vec0(
      PromptId TEXT PRIMARY KEY,
      Embedding FLOAT[384]
  );
  ```
- [ ] Hook su create/update prompt → ricalcola embedding del concat `Title + Description + Body` (debounced 2s per evitare ricalcolo durante typing)
- [ ] Job di backfill: al primo unlock con feature attivata, calcola embeddings per tutti i prompt esistenti (con progress bar in UI)
- [ ] Storage size estimation: 384 float32 = 1.5 KB per prompt → trascurabile su 10k prompt (15 MB)
- [ ] Hook su delete/tombstone → rimozione corrispondente da `PromptsEmbeddings`

## Step 3 — Ricerca ibrida (semantica + FTS5)

- [ ] Algoritmo: **Reciprocal Rank Fusion (RRF)** combina top-K da FTS5 e top-K da sqlite-vec
- [ ] Comando Tauri `search_hybrid(query, limit, alpha)` dove `alpha` ∈ [0,1] pesa semantico vs lessicale
- [ ] Default `alpha = 0.5` (bilanciato), configurabile in Impostazioni > Ricerca (slider con preset Lessicale/Bilanciato/Semantico)
- [ ] **Fallback automatico**: se embedding model non caricato → solo FTS5 senza errore
- [ ] **UI**: nessun cambio visibile per l'utente nella libreria. La ricerca è la stessa, ma trova "fratelli concettuali"
- [ ] **UI Command Palette**: badge "ricerca semantica attiva" piccolino se la query usa embeddings
- [ ] Highlight nei risultati: per FTS5 match testuale, per semantico nessun highlight (non c'è una keyword da evidenziare)
- [ ] Test qualitativi: dataset di 50 prompt, query "riscrivi email in tono formale" deve trovare anche prompt "trasforma messaggio in business style"

## Step 4 — Auto-suggerimento tag

Quando l'utente crea un nuovo prompt, suggerire tag pertinenti.

- [ ] Approccio: per ogni tag esistente nel workspace, calcola embedding del nome tag + descrizione (se presente). Confronta con embedding del nuovo prompt. Top-K per cosine similarity.
- [ ] Comando Tauri `tags_suggest(promptId or text)` → lista `[{tag, score}]` ordinata
- [ ] **UI nell'Editor**: sotto al tag picker, sezione "Tag suggeriti" con chip cliccabili (top 5)
- [ ] Fallback: se < 10 tag esistenti nel workspace, ritorna tag più frequenti (no embedding necessario)
- [ ] Soglia minima cosine similarity per suggerire: 0.55 (configurabile)
- [ ] **UX**: i suggerimenti non sono autoritativi, sono suggerimenti. L'utente deve poter ignorare facilmente.

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

- [ ] Modulo `lib_lint/` in client con regole pluggable
- [ ] Comando Tauri `prompt_lint(body, segnaposti)` → lista issues `[{code, severity, message, line, col}]`
- [ ] **UI**: pannello "Diagnosi" nell'editor (collapsible, default chiuso se zero issues)
- [ ] Inline marker in CodeMirror 6 (decoration) sui punti incriminati
- [ ] Severità error blocca il salvataggio (override possibile con conferma esplicita)
- [ ] Configurabile in Impostazioni > Linting: abilita/disabilita per categoria
- [ ] Test: dataset di prompt buoni e cattivi, verifica precision/recall delle regole PII

## Step 6 — Modello-target dichiarato

Ogni prompt può dichiarare per quale modello AI è ottimizzato.

- [ ] Migration: aggiunto `TargetModel` in Fase 1 — verificare che sia popolato correttamente
- [ ] Valori predefiniti: `claude-opus`, `claude-sonnet`, `claude-haiku`, `gpt-4`, `gpt-4-mini`, `gemini-pro`, `gemini-flash`, `llama-3`, `generic`
- [ ] **UI Editor**: dropdown nella sidebar destra con preset + opzione "Custom"
- [ ] **UI Libreria**: filtro nella sidebar "Per modello target" (count per modello)
- [ ] **UI Renderer**: badge che mostra il modello target del prompt corrente, warning se l'utente sta per copiarlo per un modello diverso (eventually)
- [ ] **Server**: filtro `?target=claude-opus` su endpoint search

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

- [ ] Migration v4 con schema sopra
- [ ] Comando Tauri `folder_crea(parentId, name)`, `folder_rinomina(id, name)`, `folder_sposta(id, newParentId)`, `folder_elimina(id, cascade)`
- [ ] Comando Tauri `prompt_sposta(promptId, folderId | null)`
- [ ] Path denormalizzato: query "tutti i prompt sotto `/marketing`" usa `WHERE Path LIKE '/marketing/%'` indicizzato
- [ ] Rinomina cartella → recompute Path dei discendenti (UPDATE ricorsivo, transazione)
- [ ] **UI Sidebar Libreria**: tree view "Cartelle" sotto le viste esistenti (Recenti/Preferiti/Tutti). Espandi/collassa ramo, contatore prompt per cartella
- [ ] **UI**: drag & drop prompt tra cartelle, drag & drop cartelle in altre cartelle
- [ ] **Click destro su cartella**: nuovo, rinomina, elimina, esporta
- [ ] **Sync server**: endpoint `/sync/folders` per workspace team (delta sync coerente con prompt sync)
- [ ] **Filtri Libreria**: filter chip "Cartella corrente" per ricerca limitata al ramo
- [ ] Test: creazione/rinomina/spostamento/cancellazione, path denormalizzato coerente in tutti gli scenari

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

- [ ] Parser nel modulo template: estendi `estraiSegnaposti` per riconoscere `{{import "..." [with k=v ...]}}` e ritornare struttura ricca
- [ ] Comando Tauri `prompt_compila(promptId, vars)` — risolve import ricorsivamente, restituisce testo finale + dependency tree
- [ ] **Resolver**:
  1. Path lookup per cartella+slug, fallback su Title match
  2. Risoluzione versione: pin esplicito (`{{import "..." version=12}}`) o latest
  3. Depth-limit 5 per evitare cicli accidentali
  4. Errore esplicito su import non risolto con `IMP001` (vedi linting Step 5)
  5. Errore su ciclo con `IMP002`
- [ ] **Editor UI**: doppia vista "Sorgente" (con `{{import}}`) e "Compilato" (testo finale espanso). Toggle in alto.
- [ ] **Editor UI**: hover su un import mostra preview del prompt importato
- [ ] **Editor UI**: navigazione "Vai al prompt importato" (ctrl+click)
- [ ] **Cross-prompt linting**: quando si modifica un prompt, mostra "Questo prompt è importato da N altri prompt. Vedi lista?". Click apre vista delle dipendenze inverse.
- [ ] **Versioning interaction**: pin esplicito a una versione = stabilità (l'import non cambia se il prompt referenced viene modificato). Default = follow latest = aggiornamenti propaganti.
- [ ] **Indicizzazione embeddings**: il body del prompt importatore è indicizzato sul testo *non compilato* (mantiene riferimento ai blocchi importati come metadati semantici)
- [ ] **Export**: il formato JSON include la struttura import. Markdown export ha una direttiva front-matter `imports: [...]` per riproducibilità.
- [ ] Test: prompt che importa altri 3 prompt → cambia uno → verifica propagazione. Cycle detection. Pin version stability.

## Step 9 — Statistiche qualità prompt

Aggregazione passiva dei dati d'uso (già raccolti da `UseCount`, `LastUsedAt`).

- [ ] Vista "Insight" in Libreria (icona dedicata nella sidebar)
- [ ] **Top prompt usati** ultimi 30 giorni
- [ ] **Prompt non usati** da > 90 giorni (candidati a cleanup)
- [ ] **Prompt più importati** (anticipa il valore di Step 8)
- [ ] **Distribuzione per tag**, **per cartella**, **per modello target**, **per autore** (per workspace team)
- [ ] **Lint health**: % prompt senza issues, top 10 categorie issue più frequenti
- [ ] Charts: usa SVG inline + librerie minimal (preferire SVG custom su `chart.js`)
- [ ] **Privacy**: nessun dato esce dal vault. Statistiche aggregate locali.

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
- [ ] Tag `v0.3.0` (post-merge di questa PR)

---

## Decisioni discrezionali

1. **Modello embedding**: ✅ deciso in Spike 3 v1 (2026-05-04), **confermato in v2 (2026-05-05)** dopo valutazione di alternative 2024-2025 — `paraphrase-multilingual-MiniLM-L12-v2` (118 MB ONNX). Vedi `docs/architettura/decisioni/embedding-model.md`. EmbeddingGemma-300m documentato come alternativa futura se vincoli size/perf si rilassano.
2. **Cache embeddings server-side per workspace team**: il server ricalcola gli embedding per condividerli? Pro: zero ricalcolo per ogni client. Contro: il server vede testo prompt in chiaro (necessario per il calcolo). Trade-off di privacy che entra in conflitto con E2E in Fase 5.
3. **Linting PII è block-by-default o warn-only?** Per workspace ad alta sensibilità (Fase 5 E2E) sarà block; per ora **warn-by-default** sembra ragionevole.
4. **Sintassi import**: decisione presa — `{{import "..."}}` coerente con segnaposti.

---

## Riferimenti

- Fase 2 (precedente): `docs/roadmap/fase-2-foundations.md`
- Fase 4 (prossima): `docs/roadmap/fase-4-workflow.md`
- Modelli embedding: https://huggingface.co/BAAI/bge-small-en-v1.5
- sqlite-vec: https://github.com/asg017/sqlite-vec
- ONNX Runtime Rust: https://github.com/pykeio/ort
