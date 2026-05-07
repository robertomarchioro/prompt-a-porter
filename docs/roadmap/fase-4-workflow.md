# Todo Fase 4 — Workflow Avanzati & Quality Assurance

> **Stato**: ✅ **client-first track chiusa al 100%** (Step 1, 2, 3, 4, 5, 8). Step 6 (approval) e 7 (RBAC cartelle) rinviati a Fase 5 perché dipendono dal server team in produzione — vedi [`rinvii.md`](./rinvii.md).
>
> 13 PR mergiate (#58-#69 in serie continua, post v0.3.0). Coverage globale 69.53% line / 73.99% function. Tag `v0.4.0` previsto dopo Step 9 (quality gate) e Step 10 (doc + release).

> **Deliverable finale**: tag release `v0.4.0`.

## Direzione generale del progetto

Prompt a Porter è una libreria locale-first per prompt AI. Tutte le scelte tecniche seguono tre vincoli non negoziabili:

1. **I dati restano sull'utente.** Vault cifrato locale, feature cloud opt-in, niente telemetria.
2. **Niente lock-in.** Formati aperti (Markdown, JSON), licenza AGPL 3.0, export sempre disponibile, schema dati documentato.
3. **Integrazione via standard.** MCP, OIDC, OpenAPI 3.1, Native Messaging — niente API proprietarie chiuse.

Il progetto attraversa 5 fasi: dall'app standalone (Fase 1, chiusa) alle fondamenta solide e integrabili (Fase 2), all'intelligenza assistiva tutta locale (Fase 3), ai workflow avanzati con qualità misurabile (Fase 4, questa), all'ecosistema enterprise opt-in (Fase 5 → v1.0.0).

## Direzione di Fase 4

A questo punto il dataset utente è ricco (centinaia/migliaia di prompt), il team li usa quotidianamente, e i workflow diventano più sofisticati. Fase 4 trasforma PaP da **libreria di prompt** a **laboratorio di prompt**: confrontare versioni, sperimentare varianti, misurare qualità in modo riproducibile, formalizzare approvazioni, governare l'accesso ai contenuti per cartella.

Il differenziatore di questa fase è il **regression testing tramite golden examples**: nessun altro prompt manager ti dice "questo prompt si è rotto perché il modello sottostante è cambiato". Trasformare il prompt da testo a **contratto comportamentale verificabile** è ciò che separa "tool vezzoso" da "infrastruttura affidabile" per chi usa prompt in CI/CD o in workflow critici.

### Filosofia di Fase

> Penelope tesseva di giorno e disfaceva di notte: senza versioning e senza varianti, ogni esperimento sul prompt cancellava il precedente. Fase 4 dà all'utente la **macchina del tempo** (diff), la **macchina dei "se"** (varianti A/B), e la **macchina della verità** (regression testing su golden examples), così che non debba scegliere tra esplorare e conservare.

---

## Step 0 — Prerequisiti

- [x] Fase 3 chiusa: `v0.3.0` taggata 2026-05-06, ricerca semantica + linting + cartelle + import funzionanti
- [x] Versioning Fase 2 funzionante e stabile (PromptVersions popolato e usato dal diff Step 3)
- [x] Embeddings disponibili — usati da `similarity::cosine` per i golden cosine
- [x] Branch lavorato direttamente su feature branch da `main` (no `fase-4` long-running)

---

## Scope Feature Fase 4

| # | Step | Modulo |
|---|------|--------|
| 1 | Variants / A-B testing dello stesso prompt | client + server |
| 2 | Rating dopo l'uso + metrica qualità aggregata | client + server |
| 3 | Diff tra versioni (stile git) | client |
| 4 | Confronto fianco-a-fianco di prompt diversi | client |
| 5 | Fork/clone prompt team in spazio personale | client + server |
| 6 | Approval workflow opzionale | server + client |
| 7 | Permessi per cartella (RBAC) | client + server |
| 8 | Golden examples + regression testing | client (+ provider AI configurabile) |

---

## Step 1 — Variants / A-B testing

> ✅ Atterrato in PR #66 (2026-05-07). Schema V011 + modulo `varianti.rs` con 16 unit test (89.44% coverage).

- [x] Migration V011 `prompt_varianti.sql` con `ParentPromptId`/`VariantLabel`/`IsVariant` come ALTER su `Prompts` + 2 indici parziali. Decisione "stessa tabella" confermata, no view `PromptsMain` necessaria
- [x] **Backend**: `varianti.rs` con `crea_variante_pure`/`lista_pure` + 2 Tauri command. Riggancia automatica al grandparent se l'utente forka una variante
- [x] **UI Libreria**: bottone "+ Variante" nel detail pane + pillole "B/C/D" cliccabili sotto i tag
- [x] Hook FTS/embedding/imports applicati a ogni variante creata; snapshot v1 in `PromptVersions`
- 📋 **UI Editor "Crea variante"** dentro il pannello editor (oggi solo dalla Libreria) — `rinvii.md` candidato `v0.5.0`
- 📋 **Vista "Confronto varianti"** dedicata multicolonna — atterrabile riusando `ConfrontoPrompt` (Step 4) — `rinvii.md`
- 📋 **Renderer dropdown variante** con switch al volo mantenendo i valori del form — `rinvii.md`
- 📋 **Promozione variante a principale** (swap main ↔ variant) — `rinvii.md`

## Step 2 — Rating dopo l'uso

Dopo aver compilato e copiato un prompt, l'utente può dare un feedback.

**Modello dati**:

```sql
CREATE TABLE PromptRatings (
    Id           TEXT PRIMARY KEY,
    PromptId     TEXT NOT NULL REFERENCES Prompts(Id),
    UserId       TEXT NOT NULL REFERENCES Users(Id),
    Rating       INTEGER NOT NULL CHECK (Rating BETWEEN -1 AND 1),  -- -1, 0, 1
    Note         TEXT,                                               -- opzionale
    UsedWithModel TEXT,                                              -- es. "claude-sonnet-4.5"
    CreatedAt    TEXT NOT NULL
);
```

> ✅ Atterrato in PR #69 (2026-05-07). Schema V013 + modulo `rating.rs` con 15 unit test (94.74% coverage).

- [x] Migration V013 `prompt_ratings.sql` con CHECK su Rating ∈ {-1, 0, 1} + indice `idx_ratings_prompt_created`. Append-only (no UPDATE), `CreatedAt` di default `datetime('now')`
- [x] **UI Compilatore**: toast post-copy con 3 emoji 👎/😐/👍 in basso a destra. Auto-dismiss 5s, 1s dopo l'azione
- [x] **Aggregazione**: badge "👍 N% su K" nel detail pane Libreria con colorazione verde/giallo/rosso (≥80, ≥50, <50). Tooltip distribuzione completa
- [x] Errore silenzioso: rating opzionale, nessun blocco UX. Audit log `rating.aggiunto`
- 📋 **Modale "Aggiungi nota"** per voto negativo — campo `Note` già nello schema, manca solo UI — `rinvii.md`
- 📋 **Sort by quality** "Migliori prompt" in Libreria — richiede integrazione in `libreria_lista` — `rinvii.md`
- 📋 **Privacy team**: rating personali privati di default, admin vede aggregati senza note — Fase 5 con E2E

## Step 3 — Diff tra versioni (stile git)

> ✅ Atterrato in PR #65 (2026-05-07). Modulo `lib/diff/` (16 vitest) + componente `VersionDiff.svelte`.

- [x] Libreria scelta: **jsdiff** (`diff` 9.x npm, BSD-3) anziché `diff-match-patch` — API più ergonomica (`diffWords`, `diffLines`), tree-shakeable, niente WASM
- [x] Componente Svelte `<VersionDiff modalita="unified|side-by-side" />` riusabile (consumato anche da `ConfrontoPrompt` Step 4)
- [x] Segnaposti `{{...}}` preservati come token unitari grazie a `diffWordsWithSpace` (boundary `\b`)
- [x] **UI Cronologia**: toggle Body/Diff inline/Diff side-by-side, dropdown "Confronta con" che esclude la versione selezionata, badge +N/−N
- [x] **Esportazione diff Markdown** via `diffMarkdown(a, b, header)` con prefissi `+`/`-`/space; copy-to-clipboard nel pannello
- [x] Performance: jsdiff su body 10k caratteri ben sotto 50 ms (verificato a runtime)

## Step 4 — Confronto fianco-a-fianco

> ✅ Atterrato in PR #67 (2026-05-07). Componente `ConfrontoPrompt.svelte` riusa `VersionDiff` di Step 3.

- [x] Selezione multipla in Libreria via Cmd/Ctrl+click su prompt-card (legacy click apre il dettaglio come prima)
- [x] Bottone toolbar "Confronta (N)" attivo da 2 selezionati + "×" per svuotare
- [x] Modale 1200×720 a 2 colonne grid (CSS pronta per 3) con metadata + body + tag in `<pre>`
- [x] Toggle "Body affiancato" / "Diff colorato" che riusa `<VersionDiff modalita="side-by-side" />` con badge +N/−N
- [x] Indicatore visivo `prompt-card--check` (bordo sinistro accent) sulle card selezionate

## Step 5 — Fork/Clone

> ✅ Atterrato in PR #68 (2026-05-07). Schema V012 + modulo `fork.rs` con 16 unit test (89.90% coverage).

- [x] Migration V012 `prompt_fork.sql` con `ForkOfPromptId TEXT REFERENCES Prompts(Id) NULL` + indice parziale per query "fork attivi di X"
- [x] Comando Tauri `prompt_fork(prompt_id)` → nuovo Id, Visibility forzata `'private'`, `WorkspaceId='ws-personale'`, `AuthorUserId='usr-locale'`, `ForkOfPromptId=originale`, titolo `<orig> (fork)`
- [x] Tracciabilità: comando `fork_info(prompt_id)` ritorna `ForkOfInfo { original_id, original_titolo?, original_eliminato }` resiliente al soft-delete
- [x] **UI Libreria**: bottone "Fork" nel detail pane + banner "Fork di X" cliccabile per navigare all'originale (disabled se eliminato)
- [x] Tag, snapshot v1, hook FTS/embedding/imports applicati al fork
- 📋 **Contatore "N fork attivi"** lato originale per workspace team — schema già pronto via indice — `rinvii.md`
- 📋 **Pull request leggera** dal fork verso l'originale — dipende da Step 6 approval, naturale Fase 5

## Step 6 — Approval workflow

> ⏸ **Rinviato a Fase 5** (gate workspace team in produzione). Vedi [`rinvii.md`](./rinvii.md) § Fase 5.
>
> Lo Step richiede un server team in produzione per le notifiche WebSocket agli Admin, lo stato `pending_review` condiviso, e il flusso multi-utente. Senza un workspace team reale che lo richieda, non vale la pena anticiparlo nel client-first.

Per workspace ad alta governance (es. team che condividono prompt usati in produzione), introdurre flusso di approvazione opzionale.

**Configurazione per workspace**:

- [ ] Setting "Modalità di pubblicazione": `Diretta` (default, Editor pubblica subito) | `Con approvazione` (Editor crea bozza, Admin approva)
- [ ] Configurabile per visibilità: solo prompt Team in approvazione, prompt privati restano diretti

**Modello dati**:

```sql
ALTER TABLE Prompts ADD COLUMN Status TEXT NOT NULL DEFAULT 'published'
    CHECK (Status IN ('draft','pending_review','published','rejected','archived'));
ALTER TABLE Prompts ADD COLUMN ReviewedByUserId TEXT REFERENCES Users(Id);
ALTER TABLE Prompts ADD COLUMN ReviewedAt TEXT;
ALTER TABLE Prompts ADD COLUMN ReviewNote TEXT;
```

- [ ] **UI per Editor**: pulsante "Invia per revisione" invece di "Pubblica" quando workflow attivo
- [ ] **UI per Admin**: nuova vista "In revisione" in sidebar Libreria, badge con count
- [ ] Schermata revisione: diff vs versione precedente (Step 3), commenti, azioni "Approva", "Richiedi modifiche", "Rifiuta"
- [ ] **Notifiche**: WebSocket push agli Admin quando nuovo prompt in `pending_review`, all'Editor quando approvato/rifiutato
- [ ] **Visibilità**: prompt in `pending_review` visibili solo all'autore e agli Admin del workspace
- [ ] **Audit**: ogni transizione di stato logga in `AuditLog`
- [ ] **Test**: flusso completo Editor crea → Admin approva → tutti vedono

## Step 7 — Permessi per cartella (RBAC)

> ⏸ **Rinviato a Fase 5** (gate workspace team in produzione). Vedi [`rinvii.md`](./rinvii.md) § Fase 5.
>
> Lo Step richiede multi-utente reale per testare la matrice di scenari (admin/editor/user × inheritance × override). In workspace personale le cartelle restano organizzative come oggi (Fase 3 Step 7), nessun valore aggiunto dall'introdurre RBAC senza altri utenti.

Le cartelle introdotte in Fase 3 erano puramente organizzative. Ora, per workspace team, ogni cartella può avere ACL specifiche. **Solo per workspace team** — in workspace personale le cartelle restano organizzative.

**Schema**:

```sql
CREATE TABLE FolderPermissions (
    FolderId        TEXT NOT NULL REFERENCES Folders(Id),
    PrincipalType   TEXT NOT NULL CHECK (PrincipalType IN ('user','role','everyone')),
    PrincipalId     TEXT NOT NULL,                -- UserId, role name (Admin/Editor/User), o 'everyone'
    Permission      TEXT NOT NULL CHECK (Permission IN ('read','write','admin')),
    InheritFromParent INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (FolderId, PrincipalType, PrincipalId)
);
```

**Inheritance model**:
- Di default un sub-folder eredita ACL del parent (`InheritFromParent = 1`)
- Override esplicito → l'admin imposta ACL specifico per quel folder, l'inheritance può continuare ad applicarsi sopra il custom (additive)
- **Niente "deny"**: solo additive (semplifica enormemente). Se un utente ha read da `everyone` e write esplicito → effective = max(read, write) = write.
- Effective permissions = unione di (workspace ACL, ACL ereditati dai parent, ACL espliciti del folder)

**Azioni per permission level**:
- `read`: vedere il folder e i suoi prompt, eseguire/compilare
- `write`: read + creare/modificare prompt nel folder + spostarvi prompt
- `admin`: write + gestire ACL del folder + crearvi sub-folder

**Step**:

- [ ] Migration v5 con tabella `FolderPermissions`
- [ ] Comando Tauri `folder_get_permissions(folderId)` → ACL effettiva (con inheritance risolto) + esplicita (solo questo folder)
- [ ] Comando Tauri `folder_set_permission(folderId, principal, permission)` / `folder_revoke_permission(...)`
- [ ] Comando Tauri `folder_set_inherit(folderId, inherit: bool)` per sganciare inheritance
- [ ] **Server**: filter su tutti gli endpoint che listano prompt/cartelle: l'utente vede solo cartelle con read effettivo
- [ ] **UI**: click destro su cartella in workspace team → "Permessi" → modale con lista ACL, add/remove principal, toggle inheritance
- [ ] **UI**: badge "permessi custom" su folder con override esplicito vs eredità
- [ ] **UX visibility**: cartelle senza read effettivo NON appaiono nella tree (no info leak)
- [ ] **Audit**: ogni cambio ACL registrato in `AuditLog`
- [ ] **Test**: matrici di scenari (admin/editor/user × folder con/senza override × parent acl × inheritance on/off)

## Step 8 — Golden examples + regression testing

> ✅ **Atterrato in 7 PR consecutive #58-#64** (2026-05-06/07). Differenziatore strategico di Fase 4 chiuso al 100%. Vedi [`docs/utente/regression-testing.md`](../utente/regression-testing.md) (in arrivo Step 10).

**Differenziatore strategico**: nessun altro prompt manager esistente offre questa funzione. Trasforma il prompt da testo a contratto comportamentale verificabile.

**Premessa**: ogni prompt è un contratto: dato input X, ti aspetti output Y (con tolleranza). Quando il modello sottostante cambia (Claude 4.6 → 5.0), o quando edithi il prompt, **non sai se hai migliorato o peggiorato finché non lo testi a mano**. Soluzione: salviamo "golden examples" (input/output attesi), e PaP misura nel tempo se l'output reale aderisce al golden.

**Schema**:

```sql
CREATE TABLE PromptGoldens (
    Id              TEXT PRIMARY KEY,
    PromptId        TEXT NOT NULL REFERENCES Prompts(Id),
    Etichetta       TEXT NOT NULL,                -- "caso comune", "edge case lungo", ecc.
    InputVars       TEXT NOT NULL,                -- JSON delle variabili compilate
    ExpectedOutput  TEXT NOT NULL,
    SimilarityFn    TEXT NOT NULL DEFAULT 'cosine', -- cosine | llm-judge | exact-match | regex
    SoglieTolleranza REAL NOT NULL DEFAULT 0.85,
    CreatedAt       TEXT NOT NULL,
    UpdatedAt       TEXT NOT NULL,
    DeletedAt       TEXT
);

CREATE TABLE PromptRunObservations (
    Id              TEXT PRIMARY KEY,
    PromptVersionId TEXT NOT NULL,
    GoldenId        TEXT REFERENCES PromptGoldens(Id),  -- NULL = run libero senza golden
    Provider        TEXT NOT NULL,                       -- "anthropic" | "openai" | "ollama" | ...
    Model           TEXT NOT NULL,                       -- "claude-sonnet-4.6"
    ActualOutput    TEXT NOT NULL,
    Similarita      REAL,                                -- 0..1
    Passed          INTEGER NOT NULL,                    -- 1 se >= soglia
    LatenzaMs       INTEGER,
    TokensUsed      INTEGER,
    CostoStimato    REAL,                                -- USD, opzionale
    RanAt           TEXT NOT NULL,
    RanBy           TEXT NOT NULL                        -- UserId
);

CREATE INDEX idx_observations_prompt ON PromptRunObservations(PromptVersionId, RanAt DESC);
CREATE INDEX idx_observations_model ON PromptRunObservations(Provider, Model, Passed);
```

**Provider AI**: l'utente decide. Configurazioni supportate:
- **Anthropic API** (Claude): chiave API in Impostazioni > Provider AI, cifrata nel vault
- **OpenAI API** (GPT): idem
- **Google API** (Gemini): idem
- **Ollama locale** (qualsiasi modello locale): URL del server Ollama, default `http://localhost:11434`
- **OpenAI-compatible custom** (LM Studio, vLLM, ecc.): URL + chiave generica

**UI Editor — pannello "Test"** ✅ (PR #62):
- [x] Tab Golden affiancato a "Cronologia" e "Diagnosi" nell'EditorPrompt
- [x] Lista golden con etichetta + similarity_fn + soglia + icona stato (○/…/✓/✗)
- [x] Form crea/modifica inline con etichetta, input_vars JSON, expected, dropdown similarity, soglia
- [x] Pulsante "Esegui" per singolo golden con feedback live (running/ok/ko + similarità + latenza)
- [x] Output ricevuto in `<details>` collassabile (max 200px scroll)
- 📋 **"Esegui tutti i golden" batch** — atterrabile come quick win in v0.5

**UI Libreria — vista "Regressioni"** ✅ (PR #64):
- [x] Nuova superficie `Regressioni.svelte` accessibile dalla sidebar Libreria con NavItem dedicato
- [x] Tabella drift per (prompt × provider × model) ordinata per ultima run, con border-left colorato (ok/ko/misto)
- [x] Filtro periodo dropdown (7/30/90/365 giorni) con re-render automatico
- [x] Drift colorato: rosso > 5%, giallo 0-5%, verde se migliorato. Tooltip con conteggi
- [x] **Esportazione CSV** RFC 4180 via Blob + `URL.createObjectURL` (download client-side)

**Similarity functions** ✅ (PR #60+#63):
- [x] **`cosine`**: riusa `compute_embedding_opt` Fase 3, dot product L2-normalized
- [x] **`llm-judge`** (PR #63): meta-prompt rubric con parser tollerante (virgola decimale, prefisso testuale, normalizzazione percentuale)
- [x] **`exact-match`**: trim + ==
- [x] **`regex`**: `expected` come regex Rust, pass/fail

**Provider AI** ✅ (PR #59 Ollama, #63 Anthropic+OpenAI+OpenAI-compat):
- [x] **Anthropic**: `POST /v1/messages` con `x-api-key` + `anthropic-version`. Parser concatena solo blocchi `text`
- [x] **OpenAI**: `POST /v1/chat/completions` con `Authorization: Bearer`. Preferisce `completion_tokens` su `total_tokens`
- [x] **Ollama**: `POST /api/generate` non-streaming, default `localhost:11434`, override base_url
- [x] **OpenAI-compatible**: stesso codepath OpenAI con base_url custom (LM Studio, vLLM)
- 📋 **Google API (Gemini)**: non implementato in v0.4 — `rinvii.md` candidato `v0.5.0`

**Privacy & sicurezza** ✅:
- [x] API key in plaintext nel DB cifrato SQLCipher AES-256 (no doppia cifratura applicativa). UPSERT preserva chiave esistente se omessa
- [x] `provider_config_lista` Tauri command **NON** rinvia api_key al frontend
- [x] Test mai automatici: ogni run richiede click utente in UI o invocazione CLI
- 📋 **Mai loggare chiavi API**: audit conferma nessun log di `ApiKey` ma serve verifica formale (security-review) — `rinvii.md`

**Step lavoro** ✅:
- [x] Migration **V008** PromptGoldens, **V009** PromptRunObservations, **V010** ProviderConfig (PR #58, #63)
- [x] Modulo Rust `provider_ai.rs` con trait `AIProvider` + 4 impl (PR #59 Ollama, #63 remote)
- [x] Modulo Rust `similarity.rs` con le 4 funzioni + `SimilarityCtx` (PR #60, #63)
- [x] Comandi Tauri `golden_crea/aggiorna/elimina/lista` (PR #58)
- [x] Comando Tauri `golden_esegui` end-to-end con mock-able provider (PR #61)
- [x] Comandi Tauri `regression_report` + `regression_report_csv` (PR #64)
- 📋 **CLI integration** `pap test <promptId>` per CI/CD — `rinvii.md` candidato `v0.5.0`
- 📋 **MCP integration** `pap_test_prompt` come tool MCP per agenti — Fase 5 con MCP HTTP/SSE

## Step 9 — Quality gate Fase 4

> ✅ Atterrato in PR #71 (2026-05-07). Vedi `docs/operativo/coverage.md` § "Moduli Fase 4 vs target Step 9".

- [x] Coverage ≥ 70% sui moduli Fase 4: **rating 95.24%, regression 91.27%, fork 91.14%, varianti 90.36%, similarity 86.13%, provider_ai 77.17%**. Globale 69.91% line + 74.30% function
- [x] Test regression con pipeline mock (Step 8d MockProvider in PR #61): copre detect di drift senza dipendere da Ollama live
- [x] Performance diff: jsdiff su 50 versioni body 10k char misurato < 50 ms a runtime (PR #65) — bench formale rinviato a `v0.5.0`
- [x] 7 stress test sentinel anti-regressione: 100 varianti / 50 fork / 100 rating misti
- ⏸ E2E flusso approvazione + sync `pending_review` + ACL cartelle scenari → Fase 5 (Step 6+7 rinviati con server team)
- ⏸ Smoke test manuale variants × rating × approval × fork × ACL × regression — manuale, no PR

## Step 10 — Documentazione e release

> ✅ Atterrato 2026-05-07. Tag `v0.4.0` rilasciato dopo questa PR.

- [x] Schema-dati esteso con V008-V013 (PromptGoldens, PromptRunObservations, ProviderConfig, ParentPromptId/VariantLabel/IsVariant, ForkOfPromptId, PromptRatings)
- [x] `docs/utente/varianti-prompt.md`
- [x] `docs/utente/fork-prompt.md`
- [x] `docs/utente/rating-prompt.md`
- [x] `docs/utente/regression-testing.md`
- [x] CHANGELOG `v0.4.0`
- [x] Roadmap aggiornata: questo doc + `release-plan.md` + `rinvii.md` + cluster README (PR #70)
- ⏸ `docs/workflow-approvazione.md` — rinviato con Step 6 a Fase 5
- ⏸ `docs/permessi-cartelle.md` — rinviato con Step 7 a Fase 5
- ⏸ Aggiornare `docs/utente/cli.md` con `pap test` — quick win `v0.5.0`
- ⏸ Matrice RBAC in overview — rinviata con Step 7 a Fase 5
- [x] Tag `v0.4.0` post-merge di questa PR

---

## Decisioni discrezionali

1. **Varianti come stessa tabella o tabella separata?** Riutilizzare `Prompts` con `ParentPromptId` è più semplice ma confonde le query (filtro IsVariant ovunque). Tabella separata è più pulita ma duplica codice CRUD. Preferenza mia: stessa tabella + view `PromptsMain` per query "solo principali".
2. **Rating granularità**: 3 valori (-1/0/+1) o 5 stelle? Le 5 stelle danno più sfumature ma soffrono il bias culturale (italiano dà 3, americano dà 5). Preferenza: **3 valori** semplice.
3. **Approval workflow su prompt privati**: ha senso o solo team? Preferenza: **solo team**, i privati restano diretti per non penalizzare uso personale.
4. **Fork mostra link all'originale anche se cancellato?** Decidere policy retention.
5. **Provider AI default per regression testing**: nessun default — l'utente sceglie. Il primo provider configurato diventa il "preferito" automaticamente.
6. **Costo stimato per run**: includere prezzi token nel report? Utile ma il calcolo dipende dal provider e cambia. Preferenza: includere se provider espone pricing API, altrimenti no.

---

## Riferimenti

- Fase 3 (precedente): `docs/roadmap/fase-3-intelligence.md`
- Fase 5 (prossima): `docs/roadmap/fase-5-enterprise.md`
- Algoritmo diff: https://github.com/google/diff-match-patch
- LLM-as-judge: https://huggingface.co/learn/cookbook/llm_judge
- Anthropic API: https://docs.anthropic.com/
- OpenAI API: https://platform.openai.com/docs/
- Ollama API: https://github.com/ollama/ollama/blob/main/docs/api.md
