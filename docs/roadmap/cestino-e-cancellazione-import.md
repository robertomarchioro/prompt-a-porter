# Blueprint — Cestino prompt (#302) + Warning cancellazione import (#303)

> Stato: **design, no codice**. Due feature accoppiate dall'unico meccanismo
> di cancellazione (soft-delete) e dalla relazione `{{import}}`.
> Issue: **#302** (recupero prompt cancellati, P2) e **#303** (warning su
> cancellazione di prompt referenziati, P1).
> Decisioni prodotto 2026-06-15:
> - **#303**: primo taglio = *annulla* + *rimozione import massiva*. La
>   sostituzione con dropdown (opzione 3 dell'issue) è **rinviata**.
> - **#302**: cestino con *ripristino* + *eliminazione definitiva* (singola +
>   svuota). **Niente** auto-pulizia per anzianità.

## 1. Scoperte che riducono il lavoro

1. **La cancellazione è già soft-delete.** `editor.rs:264` `prompt_elimina`
   esegue `UPDATE Prompts SET DeletedAt = datetime('now') WHERE Id = ?1 AND
   DeletedAt IS NULL`. I prompt cancellati **non sono persi**, solo nascosti
   (tutte le query filtrano `DeletedAt IS NULL`). `statistiche.rs:104` li
   conta già. → **#302 è quasi solo UI + 2-3 query.**
2. **La relazione "chi importa chi" esiste.** Tabella
   `PromptImports(ParentPromptId, ImportedPromptId)`; IMP004 (`linting.rs:410`)
   fa già `SELECT COUNT(DISTINCT ParentPromptId) ... WHERE ImportedPromptId =
   ?1`. Per #303 basta restituire la **lista** dei genitori.
3. **La riscrittura dei body è già strutturata.** `ImportRef`
   (`prompt_componibili.rs:41`) espone `path`, `byte_start`, `byte_end`,
   `with`, `version` → rimozione precisa del token. `aggiorna_imports`
   ricalcola le righe `PromptImports` dopo ogni modifica al body.
4. **Il versioning esiste** (`prompt_get_history`/`prompt_rollback`) → le
   modifiche massive ai body altrui possono essere snapshot-ate e reversibili.
5. **La UX promette già un cestino.** Il confirm in `DetailPane.eliminaPrompt`
   recita *"spostato nel cestino … recuperabile"* — promessa oggi non
   mantenuta: #302 la onora.

## 2. Fondamenta condivise

### 2.1 Backend: lista dipendenti

```rust
// in prompt_componibili.rs o cartelle.rs (vicino alla logica import)
#[derive(Serialize)]
pub struct Dipendente { pub id: String, pub titolo: String }

#[tauri::command]
pub fn prompt_dipendenti(id: String, state: State<'_, VaultState>)
    -> Result<Vec<Dipendente>, PapErrore> {
    // SELECT DISTINCT pr.Id, pr.Title
    //   FROM PromptImports pi JOIN Prompts pr ON pr.Id = pi.ParentPromptId
    //  WHERE pi.ImportedPromptId = ?1 AND pr.DeletedAt IS NULL
    //  ORDER BY pr.Title
}
```

Usata da **#303** (popolare il warning) e da **#302** (in ripristino: "questo
prompt era importato da N altri" — informativo).

### 2.2 Ordine di realizzazione

Nonostante #303 sia P1, **#302 va per primo**: è la rete di sicurezza (ogni
cancellazione diventa recuperabile) ed è a basso rischio. Con il cestino in
place, #303 può offrire serenamente "cancella comunque" perché l'azione è
reversibile. Poi #303 sopra le fondamenta condivise.

## 3. #302 — Cestino prompt

### 3.1 Backend (`editor.rs` / nuovo `cestino.rs`)

```rust
#[derive(Serialize)]
pub struct PromptCancellato {
    pub id: String, pub titolo: String,
    pub eliminato_il: String,       // DeletedAt ISO
    pub importato_da: usize,        // count dipendenti (avviso ripristino)
}

#[tauri::command]
pub fn cestino_lista(state) -> Result<Vec<PromptCancellato>, PapErrore>;
// SELECT Id, Title, DeletedAt FROM Prompts WHERE DeletedAt IS NOT NULL ORDER BY DeletedAt DESC

#[tauri::command]
pub fn prompt_ripristina(id: String, state) -> Result<(), PapErrore>;
// UPDATE Prompts SET DeletedAt = NULL, UpdatedAt = datetime('now') WHERE Id = ?1 AND DeletedAt IS NOT NULL
// → poi aggiorna_imports(body) per ricucire PromptImports; audit "prompt.ripristinato"

#[tauri::command]
pub fn prompt_elimina_definitivo(id: String, state) -> Result<(), PapErrore>;
// DELETE FISICO: prima FK figli (PromptImports, PromptVersions, Ratings, Golden…), poi Prompts.
// Solo su righe con DeletedAt IS NOT NULL (non si può purgare un prompt vivo). audit "prompt.eliminato_definitivo"

#[tauri::command]
pub fn cestino_svuota(state) -> Result<usize, PapErrore>;
// elimina_definitivo per ogni riga del cestino; ritorna il conteggio
```

> **Sicurezza** (regola progetto: query parametrizzate, validazione boundary):
> il purge fisico agisce **solo** su `DeletedAt IS NOT NULL`; mai su prompt
> vivi. `PRAGMA foreign_keys = ON` ⇒ con FK attive il DELETE su `Prompts` fallisce
> se restano figli senza cascade. Solo `PromptTags` ha `ON DELETE CASCADE`;
> tutte le altre relazioni vanno gestite a mano (vedi ordine in §3.1bis).

### 3.1bis Ordine cascade del purge fisico (verificato su schema V001-V015)

`prompt_elimina_definitivo(X)` in **una transazione**, in quest'ordine:

```sql
-- figli con FK semplice (no cascade) → cancellare PRIMA di Prompts
DELETE FROM PromptVersions WHERE PromptId = X;
DELETE FROM PromptImports  WHERE ParentPromptId = X OR ImportedPromptId = X;
DELETE FROM PromptRatings  WHERE PromptId = X;
DELETE FROM PromptGoldens  WHERE PromptId = X;
-- self-ref (variante/fork): NULL per non bloccare la FK
UPDATE Prompts SET ParentPromptId = NULL WHERE ParentPromptId = X;  -- orfana le varianti
UPDATE Prompts SET ForkOfPromptId = NULL WHERE ForkOfPromptId = X;
-- side-tables non-FK ma da ripulire per coerenza
DELETE FROM PromptsFts        WHERE PromptId = X;   -- FTS5 (search)
DELETE FROM PromptEmbeddings  WHERE PromptId = X;   -- se presente (V005)
-- PromptTags si svuota da solo (ON DELETE CASCADE)
DELETE FROM Prompts WHERE Id = X AND DeletedAt IS NOT NULL;  -- guard: solo cestinati
```

### 3.2 Edge case ripristino

- **Nessun vincolo UNIQUE su `Prompts.Title`** (verificato: V001 lo definisce
  `TEXT NOT NULL` senza UNIQUE; nessuna migrazione lo aggiunge). → il ripristino
  **non può violare un constraint**. Titoli duplicati creano al più ambiguità
  nel resolve degli import, ma è già possibile oggi: il ripristino non
  introduce nulla di nuovo. Nessun suffisso necessario.
- **Import contenuti nel prompt ripristinato**: ri-eseguire `aggiorna_imports`
  così le sue dipendenze in uscita tornano tracciate.
- **Prompt ripristinato che altri importavano**: l'import rotto (IMP001) nei
  genitori torna automaticamente valido (resolve_path lo ritrova).
- **Varianti**: `prompt_elimina` non cascata alle varianti (restano con
  `ParentPromptId` al genitore cestinato). Comportamento preesistente,
  preservato: le varianti si ripristinano singolarmente dal cestino.

### 3.3 Frontend

- Voce di navigazione **"Cestino"** nella `Sidebar` (sotto le viste, con
  conteggio da `cestino_lista().length`).
- Vista lista cancellati (riusa lo stile `ListPane`/`PromptCard` in modalità
  read-only): per riga `[Ripristina]` `[Elimina ✕]`; in testa `[Svuota
  cestino]` (con conferma).
- Badge "importato da N" sulla riga se `importato_da > 0` (suggerisce che il
  ripristino ricuce import rotti altrove).
- Eventi: dopo ripristino/elimina → `pap:lista-mutata` (già in uso) per
  rinfrescare libreria e conteggi.

### 3.4 Aggiornare il messaggio di cancellazione

Il confirm di `eliminaPrompt` ora può dire con verità "recuperabile dal
**Cestino**" (non più "dal database fino al cleanup").

## 4. #303 — Warning cancellazione prompt referenziati

### 4.1 Flusso

Quando si cancella X, **prima** di `prompt_elimina`:

1. `const deps = await invoke("prompt_dipendenti", { id })`.
2. Se `deps.length === 0` → comportamento attuale (soft-delete diretto).
3. Se `deps.length > 0` → apri **modale di warning** (non `window.confirm`):

```
⚠ "Ruolo esperto" è importato da 3 prompt:
   • Email formale
   • Onboarding
   • Report settimanale
Cancellandolo, questi import si romperanno.

  [Annulla]
  [Rimuovi l'import dai 3 prompt e cancella]
```

### 4.2 Backend — rimozione import massiva

```rust
#[tauri::command]
pub fn import_rimuovi_da_dipendenti(
    target_id: String,        // X, il prompt che sto per cancellare
    state,
) -> Result<usize, PapErrore> {
    // Per ogni P in dipendenti(X):
    //   1. leggi P.Body
    //   2. parse_imports(body); filtra gli ImportRef il cui resolve_path == X
    //   3. rimuovi i token per byte range (DAL FONDO verso l'inizio, così gli
    //      offset restano validi); pulisci eventuali righe/spazi residui
    //   4. SNAPSHOT versione di P (prima della modifica) — reversibile
    //   5. prompt_aggiorna(P, nuovo_body) → ricalcola PromptImports
    // Ritorna il numero di prompt modificati. Transazione unica.
}
```

> Poi il comando lato UI esegue `prompt_elimina(X)` (soft-delete). Ordine:
> prima sgancia gli import, poi cancella X — così nessun istante con import
> rotti.

### 4.3 Edge case rimozione

- **Token con parametri** `{{import "X" with tono=formale}}` o `version=N`:
  si rimuove l'intero token via `byte_start..byte_end` (già esatti in
  `ImportRef`) — i modifiers spariscono col token. Nessun parsing speciale.
- **Più import dello stesso X** nello stesso body: rimuovere tutte le
  occorrenze, sempre dal fondo.
- **Import di X annidato** in un prompt Y a sua volta importato: si tocca solo
  il livello diretto (genitori di X); la catena più profonda si ricalcola da
  sé al prossimo lint.
- **Errore su un singolo P**: transazione → rollback totale, X non viene
  cancellato, messaggio chiaro. Niente stato a metà.
- **Whitespace orfano**: dopo la rimozione, collassare eventuale riga vuota
  lasciata dal token su riga propria (cosmesi, non rompere il markdown).

### 4.4 Frontend

- Nuova modale `WarningCancellazioneImport.svelte` (riusa il pattern `Modale`
  esistente). Sostituisce `window.confirm` **solo** quando `deps.length > 0`.
- Sequenza bottone "Rimuovi e cancella":
  `import_rimuovi_da_dipendenti(X)` → `prompt_elimina(X)` →
  `pap:lista-mutata`. Toast "Import rimossi da N prompt, prompt cancellato".

### 4.5 Rinviato (opzione 3 dell'issue)

Sostituzione con prompt Y da dropdown: stessa macchina di §4.2 ma invece di
rimuovere il token si riscrive `ImportRef.path` X→Y preservando `with`/
`version`. Comando futuro `import_sostituisci_in_dipendenti(target_id,
replacement_id)`. Fuori dal primo taglio.

## 5. Test plan

**Rust (#302)**:
- `cestino_lista` ritorna solo `DeletedAt IS NOT NULL`, ordinati per data.
- `prompt_ripristina` azzera DeletedAt e il prompt riappare in
  `libreria_lista`; ri-traccia gli import in uscita.
- `prompt_elimina_definitivo` rifiuta un prompt vivo; su un cancellato
  rimuove anche le righe figlie (no orfani in PromptImports/Versions).
- `cestino_svuota` conta e azzera.

**Rust (#303)**:
- `prompt_dipendenti` ritorna i genitori distinti, esclude i cancellati.
- `import_rimuovi_da_dipendenti`: body con `{{import "X"}}` semplice e con
  `with`/`version` → token rimosso, PromptImports aggiornata, snapshot creato;
  multi-occorrenza; rollback su errore.

**TS (vitest)**: il flusso UI sceglie warning vs confirm in base a
`deps.length`; cestino renderizza azioni e dispatcha gli eventi.

## 6. Breakdown PR

| PR | Issue | Scope | Rischio |
| --- | --- | --- | --- |
| **PR-1** | #302 | Backend cestino (`cestino_lista`/`prompt_ripristina`/`prompt_elimina_definitivo`/`cestino_svuota`) + test | basso |
| **PR-2** | #302 | UI Cestino in Sidebar + lista + ripristino/elimina/svuota + msg confirm aggiornato | basso-medio |
| **PR-3** | shared | `prompt_dipendenti` + test (fondamenta #303) | basso |
| **PR-4** | #303 | Backend `import_rimuovi_da_dipendenti` (snapshot + transazione) + test | medio |
| **PR-5** | #303 | UI warning modale + sequenza rimuovi-e-cancella | basso-medio |
| **PR-6** *(opz.)* | #303 | Opzione 3: `import_sostituisci_in_dipendenti` + dropdown | medio |

PR-1→2 chiudono #302. PR-3→5 chiudono #303 (primo taglio). PR-6 completa
l'issue #303 quando vorremo la sostituzione.

## 7. Domande aperte — RISOLTE (schema V001-V015, 2026-06-15)

- ✅ **UNIQUE su `Prompts`**: nessuno su `Title`. Il ripristino non può violare
  constraint. Logica "(ripristinato)" rimossa. (Il `UNIQUE (WorkspaceId, Name)`
  è sui **Tags**, non sui Prompts.)
- ✅ **Varianti**: `prompt_elimina` non cascata (`WHERE Id = ?1`). Le varianti
  restano cestinabili/ripristinabili singolarmente. Comportamento preesistente
  preservato.
- ✅ **FK CASCADE**: solo `PromptTags` ha `ON DELETE CASCADE`. Tutte le altre
  (`PromptVersions`, `PromptImports` ×2, `PromptRatings`, `PromptGoldens`,
  self-ref `ParentPromptId`/`ForkOfPromptId`) sono FK semplici con
  `foreign_keys = ON` → purge fisico con cascade manuale in transazione, ordine
  in **§3.1bis**.
