# Todo Fase 4 — Workflow Avanzati & Quality Assurance

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

- [ ] Fase 3 chiusa: `v0.3.0` taggata, ricerca semantica + linting + cartelle + import funzionanti
- [ ] Versioning Fase 2 funzionante e stabile
- [ ] Embeddings disponibili (necessari per similarity nei golden examples)
- [ ] Crea branch `fase-4` da `main`

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

Un prompt può avere N "varianti" che condividono lo stesso *intento* ma testano formulazioni diverse.

**Modello dati**:

```sql
ALTER TABLE Prompts ADD COLUMN ParentPromptId TEXT REFERENCES Prompts(Id);
ALTER TABLE Prompts ADD COLUMN VariantLabel TEXT;  -- "A", "B", "Formal", ...
ALTER TABLE Prompts ADD COLUMN IsVariant INTEGER NOT NULL DEFAULT 0;
```

In alternativa: tabella separata `PromptVariants` per non sporcare `Prompts`. Da decidere — preferenza: stessa tabella + view `PromptsMain` per query "solo principali".

- [ ] **UI Editor**: pulsante "Crea variante" dentro pannello prompt → duplica con `VariantLabel = "B"` (o successivo)
- [ ] **UI Libreria**: prompt con varianti mostrati come gruppo collassabile (icona ramificazione)
- [ ] **UI dedicata "Confronto varianti"**: layout 2 o 3 colonne fianco a fianco con body, segnaposti, ultimo uso, rating, commenti
- [ ] **Renderer**: dropdown variante in alto, switch al volo tra varianti mantenendo i valori del form
- [ ] **Statistiche**: ogni variante traccia il proprio `UseCount`, `Rating`, etc., così emerge naturalmente la migliore
- [ ] **Promozione variante**: pulsante "Promuovi a principale" che scambia il prompt main con la variante (mantiene storia)

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

- [ ] **UI**: dopo il copy nel Renderer, toast con 3 reazioni discrete in basso a destra; click registra rating; auto-dismiss 5s
- [ ] Opzionale: modale "Aggiungi nota" per spiegare un voto negativo
- [ ] **Aggregazione**: nel dettaglio prompt mostra rating medio (es. "👍 87% su 23 usi negli ultimi 30 giorni")
- [ ] **Sort by quality**: nuova vista "Migliori prompt" in Libreria, ordinata per rating medio recente
- [ ] **Privacy**: i rating personali sono privati di default; in workspace team gli admin vedono aggregati ma non singoli rating con note

## Step 3 — Diff tra versioni (stile git)

Storia versioni dalla Fase 2; aggiungiamo il diff visivo.

- [ ] Libreria: scegli `diff-match-patch` (Google, MIT) per algoritmo character/word-level
- [ ] Componente Svelte `<VersionDiff />` con 3 modalità: side-by-side, inline, unified
- [ ] Sintassi highlighting preservato: i segnaposti `{{...}}` e gli `{{import "..."}}` rimangono evidenziati anche dentro il diff
- [ ] **UI cronologia**: lista versioni in pannello dettaglio Libreria, click su una versione apre diff vs versione corrente
- [ ] **Confronto N-vs-M**: dropdown "compare with" per scegliere qualsiasi due versioni
- [ ] **Esportazione diff**: copia diff come testo Markdown (utile per code review fuori app)
- [ ] **Performance**: diff su body di 10k caratteri < 50ms

## Step 4 — Confronto fianco-a-fianco

Generalizzazione del Diff (Step 3) per confrontare anche **prompt diversi**, non solo versioni dello stesso prompt.

- [ ] Selezione multipla in Libreria (Cmd/Ctrl+click) → pulsante "Confronta selezione" attivo da 2 elementi
- [ ] Vista a colonne: 2-3 prompt fianco a fianco con metadata, body, segnaposti, rating, commenti
- [ ] Utile per: scegliere variant migliore, audit interno, code review prompt
- [ ] Diff rosso/verde tra colonne come "differenze rilevanti" (parole/frasi che cambiano)

## Step 5 — Fork/Clone

Permettere a un User di duplicare un prompt team nel proprio spazio privato per sperimentare.

- [ ] Comando Tauri `prompt_fork(promptId)` → crea copia con:
  - Nuovo `Id`
  - `Visibility = 'private'`
  - `WorkspaceId` = workspace personale dell'utente
  - `ForkOfPromptId = original.Id` (campo nuovo)
  - `Title = original.Title + ' (fork)'`
- [ ] Tracciabilità: nel dettaglio prompt forkato, badge "Forked from: [link al prompt originale se ancora visibile]"
- [ ] Lato originale (workspace team): contatore "5 fork attivi" visibile agli Admin
- [ ] **UI**: pulsante "Fork in privato" nel pannello azioni del dettaglio prompt
- [ ] **Pull request leggera (opzionale, valuta)**: dal fork posso "proporre modifica" all'originale → diventa un commit/version pending in approval workflow (Step 6)

## Step 6 — Approval workflow

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

**UI Editor — pannello "Test"**:
- [ ] Tab "Golden examples" affiancato a "Cronologia" e "Diagnosi"
- [ ] Lista golden con etichetta, input vars (preview), expected output (preview), soglia
- [ ] Pulsante "+ Aggiungi golden" → form con vars + expected + similarity function + soglia
- [ ] Pulsante "Esegui ora" su un golden → richiama provider, mostra `ActualOutput`, calcola similarità, salva osservazione
- [ ] Pulsante "Esegui tutti i golden" → batch run, riassunto pass/fail
- [ ] **Risultato run**: highlight delle differenze tra expected e actual con diff inline (riusa componente Step 3)

**UI Libreria — vista "Regressioni"**:
- [ ] Nuova sezione in sidebar "Regressioni" (badge con count se ci sono fail recenti)
- [ ] Tabella: prompt con golden fail negli ultimi N giorni, ordinata per drift (% di degrado)
- [ ] Trend per modello: "questo prompt era al 92% similarity con `claude-sonnet-4.5`, è sceso al 71% con `claude-sonnet-4.6`"
- [ ] Trend per versione: "v12 era picco qualità (95%), oggi siamo a v18 con 78%"
- [ ] Esportazione report: CSV con tutte le run e similarità

**Similarity functions**:
- [ ] **`cosine`**: embedding di `expected` e `actual`, cosine similarity. Default. Richiede embeddings di Fase 3.
- [ ] **`llm-judge`**: chiama il modello con un meta-prompt rubric "Valuta da 0 a 1 se actual rispetta expected per criteri X, Y, Z". Costo aggiuntivo, ma più sfumato.
- [ ] **`exact-match`**: 1.0 se identico, 0.0 altrimenti. Per output JSON strict, regex, ecc.
- [ ] **`regex`**: l'`expected` è una regex; pass/fail.

**Privacy & sicurezza**:
- [ ] Le chiavi API sono cifrate nel vault con la stessa derivazione (Argon2id) usata per il vault password
- [ ] Mai loggare chiavi API negli audit log o nei file di debug
- [ ] Test mai eseguiti in automatico senza azione utente esplicita (no costo nascosto)

**Step lavoro**:

- [ ] Migration v6 con `PromptGoldens`, `PromptRunObservations`, `ProviderConfig`
- [ ] Modulo Rust `lib_provider/` con trait `AIProvider` + impl per Anthropic, OpenAI, Google, Ollama, OpenAI-compatible
- [ ] Modulo Rust `lib_similarity/` con le 4 funzioni
- [ ] Comando Tauri `golden_crea(promptId, ...)`, `golden_aggiorna(...)`, `golden_elimina(...)`, `golden_lista(promptId)`
- [ ] Comando Tauri `golden_esegui(goldenId)` → run + salva observation
- [ ] Comando Tauri `regression_report(workspaceId, days)` → trend per prompt/modello
- [ ] **UI**: pannelli sopra descritti
- [ ] **CLI integration**: `pap test <promptId> [--golden=...]` per CI/CD
- [ ] **MCP integration** (anticipo Fase 5): `pap_test_prompt(promptId)` come tool MCP per agenti
- [ ] **Test**: workspace di prova con 5 prompt × 3 golden, esegui contro Ollama locale (mockabile in CI), verifica observation correttamente salvate

## Step 9 — Quality gate Fase 4

- [ ] Test coverage ≥ 70% sui moduli varianti, rating, approval, fork, ACL cartelle, golden/regression
- [ ] Test E2E flusso approvazione completo
- [ ] Test sync workspace team con prompt in stato `pending_review` (deve essere visibile solo a chi di dovere)
- [ ] Performance: diff su 50 versioni di un prompt grande < 200ms in lista
- [ ] Smoke test manuale: variants × rating × approval × fork × ACL × regression
- [ ] Test ACL cartelle: scenari complessi (admin/editor/user, inheritance on/off, override) tutti corretti
- [ ] Test regression: pipeline mock con Ollama, verifica detect di drift simulato (modello restituisce output diverso)

## Step 10 — Documentazione e release

- [ ] Aggiorna `docs/architettura/schema-dati.md` con nuove tabelle e colonne (varianti, rating, ACL cartelle, golden, observations)
- [ ] Nuovo `docs/workflow-approvazione.md`
- [ ] Nuovo `docs/varianti-prompt.md` con esempi pratici
- [ ] Nuovo `docs/permessi-cartelle.md` con matrice di scenari
- [ ] Nuovo `docs/regression-testing.md` con guida ai golden examples + provider setup
- [ ] Aggiorna matrice RBAC in `docs/architettura/overview.md` con permessi nuovi (chi può approvare, chi può forkare, chi può gestire ACL cartelle)
- [ ] Aggiorna `docs/utente/cli.md` con `pap test` e altri nuovi comandi
- [ ] CHANGELOG `v0.4.0`
- [ ] Aggiorna `docs/roadmap/fase-5-enterprise.md` (già esistente, già rivisto in linea con questa nuova roadmap)
- [ ] Tag `v0.4.0`

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
