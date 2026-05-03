# Todo Fase 4 — Workflow Avanzati

> **Obiettivo**: trasformare PaP da "libreria di prompt" a "laboratorio di prompt". A questo punto il dataset utente è ricco (centinaia/migliaia di prompt), il team li usa quotidianamente, e i workflow diventano più sofisticati: confrontare versioni, sperimentare varianti, misurare qualità, formalizzare approvazioni.
>
> **Deliverable finale**: tag release `v0.4.0-fase4`.

---

## Filosofia di Fase

Penelope tesseva di giorno e disfaceva di notte: senza versioning e senza varianti, ogni esperimento sul prompt cancellava il precedente. Fase 4 dà all'utente la **macchina del tempo** (diff) e la **macchina dei "se"** (varianti A/B), così che non debba scegliere tra esplorare e conservare.

---

## Prerequisiti (Step 0)

- [ ] Fase 3 chiusa: `v0.3.0-fase3` taggata, ricerca semantica funzionante
- [ ] Versioning Fase 2 funzionante e stabile
- [ ] Crea branch `fase-4` da `main`

---

## Scope Feature Fase 4

| # | Feature | Modulo |
|---|---------|--------|
| 1 | Variants / A-B testing dello stesso prompt | client + server |
| 2 | Rating dopo l'uso + metrica qualità aggregata | client + server |
| 3 | Diff tra versioni (stile git) | client |
| 4 | Approval workflow opzionale | server + client |
| 5 | Fork/clone prompt team in spazio personale | client + server |
| 6 | Confronto fianco-a-fianco di prompt diversi | client |

---

## Step 1 — Variants / A-B testing

Un prompt può avere N "varianti" che condividono lo stesso *intento* ma testano formulazioni diverse.

**Modello dati**:

```sql
-- Estensione: ogni prompt ha un "ParentPromptId" opzionale
-- Le varianti sono prompt fratelli con stesso ParentPromptId
ALTER TABLE Prompts ADD COLUMN ParentPromptId TEXT REFERENCES Prompts(Id);
ALTER TABLE Prompts ADD COLUMN VariantLabel TEXT;  -- "A", "B", "Formal", ...
ALTER TABLE Prompts ADD COLUMN IsVariant INTEGER NOT NULL DEFAULT 0;
```

In alternativa: tabella separata `PromptVariants` per non sporcare `Prompts`. Da decidere.

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
    Note         TEXT,                                               -- opzionale, "ha funzionato male perché..."
    UsedWithModel TEXT,                                              -- es. "claude-sonnet-4.5"
    CreatedAt    TEXT NOT NULL
);
```

- [ ] **UI**: dopo il copy nel Renderer, toast con 3 reazioni (👎 / 👌 / 👍) discrete in basso a destra; click registra rating; auto-dismiss 5s
- [ ] Opzionale: modale "Aggiungi nota" per spiegare un voto negativo
- [ ] **Aggregazione**: nel dettaglio prompt mostra rating medio (es. "👍 87% su 23 usi negli ultimi 30 giorni")
- [ ] **Sort by quality**: nuova vista "Migliori prompt" in Libreria, ordinata per rating medio recente
- [ ] **Privacy**: i rating personali sono privati di default; in workspace team gli admin vedono aggregati ma non singoli rating con note

## Step 3 — Diff tra versioni (stile git)

Ora abbiamo storia versioni dalla Fase 2; aggiungiamo il diff visivo.

- [ ] Libreria: scegli `diff-match-patch` (Google, MIT) per algoritmo character/word-level
- [ ] Componente Svelte `<VersionDiff />` con 3 modalità: side-by-side, inline, unified
- [ ] Sintassi highlighting preservato: i segnaposti `{{...}}` rimangono evidenziati anche dentro il diff
- [ ] **UI cronologia**: lista versioni in pannello dettaglio Libreria, click su una versione apre diff vs versione corrente
- [ ] **Confronto N-vs-M**: dropdown "compare with" per scegliere qualsiasi due versioni
- [ ] **Esportazione diff**: copia diff come testo Markdown (utile per code review fuori app)
- [ ] **Performance**: diff su body di 10k caratteri < 50ms

## Step 4 — Approval workflow

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
- [ ] Schermata revisione: diff vs versione precedente, commenti, azioni "Approva", "Richiedi modifiche", "Rifiuta"
- [ ] **Notifiche**: WebSocket push agli Admin quando nuovo prompt in `pending_review`, all'Editor quando approvato/rifiutato
- [ ] **Visibilità**: prompt in `pending_review` visibili solo all'autore e agli Admin del workspace
- [ ] **Audit**: ogni transizione di stato logga in `AuditLog`
- [ ] **Test**: flusso completo Editor crea → Admin approva → tutti vedono

## Step 5 — Fork/Clone

Permettere a un User di duplicare un prompt team nel proprio spazio privato per sperimentare.

- [ ] Comando Tauri `prompt_fork(promptId)` → crea copia con:
  - Nuovo `Id`
  - `Visibility = 'private'`
  - `WorkspaceId` = workspace personale dell'utente
  - `ForkOfPromptId = original.Id` (campo nuovo)
  - `Title = original.Title + ' (fork)'`
- [ ] Tracciabilità: nel dettaglio prompt forkato, badge "📑 Forked from: [link al prompt originale se ancora visibile]"
- [ ] Lato originale (workspace team): contatore "5 fork attivi" visibile agli Admin
- [ ] **UI**: pulsante "Fork in privato" nel pannello azioni del dettaglio prompt
- [ ] **Pull request leggera (opzionale, valuta)**: dal fork posso "proporre modifica" all'originale → diventa un commit/version pending in approval workflow

## Step 6 — Confronto fianco-a-fianco

Generalizzazione del Diff (Step 3) per confrontare anche **prompt diversi**, non solo versioni dello stesso prompt.

- [ ] Selezione multipla in Libreria (Cmd/Ctrl+click) → pulsante "Confronta selezione" attivo da 2 elementi
- [ ] Vista a colonne: 2-3 prompt fianco a fianco con metadata, body, segnaposti, rating, commenti
- [ ] Utile per: scegliere variant migliore, audit interno, code review prompt
- [ ] Diff rosso/verde tra colonne come "differenze rilevanti" (parole/frasi che cambiano)

## Step 7 — Quality gate Fase 4

- [ ] Test coverage ≥ 70% su moduli varianti, rating, approval, fork
- [ ] Test E2E flusso approvazione completo
- [ ] Test sync workspace team con prompt in stato `pending_review` (deve essere visibile solo a chi di dovere)
- [ ] Performance: diff su 50 versioni di un prompt grande < 200ms in lista
- [ ] Smoke test manuale su tutte le combinazioni: variants × rating × approval × fork

## Step 8 — Documentazione e release

- [ ] Aggiorna `docs/schema-dati.md` con nuove tabelle e colonne
- [ ] Nuovo `docs/workflow-approvazione.md`
- [ ] Nuovo `docs/varianti-prompt.md` con esempi pratici
- [ ] Aggiorna matrice RBAC in `docs/architettura.md` con permessi nuovi (chi può approvare, chi può forkare)
- [ ] Changelog v0.4.0
- [ ] Crea `docs/todo-fase-5.md`
- [ ] Tag `v0.4.0-fase4`

---

## Decisioni discrezionali

1. **Varianti come stessa tabella o tabella separata?** Riutilizzare `Prompts` con `ParentPromptId` è più semplice ma confonde le query (filtro IsVariant ovunque). Tabella separata è più pulita ma duplica codice CRUD. Preferenza mia: stesso tabella + view `PromptsMain` per query "solo principali".
2. **Rating granularità**: 3 valori (-1/0/+1) o 5 stelle? Le 5 stelle danno più sfumature ma soffrono il bias culturale (italiano dà 3, americano dà 5). Preferenza: **3 valori** semplice.
3. **Approval workflow su prompt privati**: ha senso o solo team? Preferenza: **solo team**, i privati restano diretti per non penalizzare uso personale.
4. **Fork mostra link all'originale anche se cancellato?** Decidere policy retention.

---

## Riferimenti

- Fase 3 (precedente): `docs/todo-fase-3.md`
- Fase 5 (prossima): `docs/todo-fase-5.md`
- Algoritmo diff: https://github.com/google/diff-match-patch
