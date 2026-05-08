# Blueprint Backend V014 — drag-reorder + extend autore + extend Statistiche

> **Versione**: 1.0 · **Data**: 2026-05-08 · **Riferimento piano**: `docs/roadmap/redesign-v08.md` §4 · **Decisioni designer**: `decisioni-designer.md` (#5, #11, #13, #14 lato UI; nessuna decisione bloccante backend) · **Stima**: 1.5 giorni FT · **Parallelo a**: F0 UI (no overlap di file)

Blueprint operativo autoportante per il lavoro backend abilitante il redesign v0.8. Singola PR `feat/redesign-backend-v014` da `main`, può partire subito.

## Indice

1. [Obiettivo](#1-obiettivo)
2. [Strategia di delivery (1 PR)](#2-strategia-di-delivery-1-pr)
3. [Migrazione V014 — SortOrder](#3-migrazione-v014--sortorder)
4. [Comando `folder_riordina`](#4-comando-folder_riordina)
5. [Comando `prompt_riordina`](#5-comando-prompt_riordina)
6. [Extend `prompt_get_history` con autore](#6-extend-prompt_get_history-con-autore)
7. [Extend `Statistiche` con `token_medi`](#7-extend-statistiche-con-token_medi)
8. [Registrazione comandi in `lib.rs`](#8-registrazione-comandi-in-librs)
9. [Test attesi](#9-test-attesi)
10. [Edge case](#10-edge-case)
11. [Rischi specifici](#11-rischi-specifici)
12. [Exit criteria](#12-exit-criteria)
13. [Note out-of-scope (UI consumer-side)](#13-note-out-of-scope-ui-consumer-side)

---

## 1. Obiettivo

Sbloccare 3 feature del redesign UI (lato Rust):

1. **Drag-reorder cartelle e prompt** (decisioni #5, #11, #14): nuova colonna `SortOrder` su Folders e Prompts + 2 comandi Tauri per riordino atomico siblings.
2. **Cronologia con autore** (decisione #13): JOIN `Users` in `prompt_get_history` per esporre `DisplayName` + `Email` (l'avatar hash è calcolato client-side).
3. **Insight `token_medi`**: campo aggiunto alla struct `Statistiche` esistente, calcolato come proxy `AVG(LENGTH(Body)) / 4`.

**Output funzionale**:
- L'app può chiamare `folder_riordina(id, new_sort)` e `prompt_riordina(prompt_id, new_sort)` per cambiare ordering atomico nei siblings.
- `prompt_get_history` restituisce `autore_display_name` + `autore_email` per ogni revisione (l'UI calcolerà il color hash da email).
- `statistiche_query` restituisce `token_medi: i64` aggregato su `Prompts.Body`.

## 2. Strategia di delivery (1 PR)

- **Branch**: `feat/redesign-backend-v014`
- **Target**: `main` (NON `feat/redesign-v08`) — rationale: il backend è agnostico rispetto al refactor UI. Il merge su main lo rende disponibile sia alla UI legacy (che continua ad ignorare il campo nuovo, no-op) sia al branch redesign.
- **Effort**: 1.5 gg distribuiti come da §4 piano:
  - 0.25 gg V014 SQL + test migrazione
  - 0.5 gg `folder_riordina` + `prompt_riordina` + test
  - 0.25 gg extend `prompt_get_history` con JOIN
  - 0.25 gg extend `Statistiche` con `token_medi`
  - 0.25 gg buffer (review, fix CI)
- **Audit trail**: ogni mutazione registra audit log (`folder.riordinato`, `prompt.riordinato`).

---

## 3. Migrazione V014 — SortOrder

### File: `apps/client/src-tauri/migrations/V014__sort_order.sql` (NEW)

```sql
-- V014: SortOrder per drag-reorder cartelle e prompt nel redesign v0.8.
-- Additiva: DEFAULT 0 + backfill ROW_NUMBER per workspace+parent.
-- Indici composti per scan ordinato dei siblings.

ALTER TABLE Folders ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;
ALTER TABLE Prompts ADD COLUMN SortOrder INTEGER NOT NULL DEFAULT 0;

-- Backfill: ordine basato sul timestamp di creazione (stabilità sui vault esistenti).
-- ROW_NUMBER è 1-based; settiamo SortOrder = ROW_NUMBER - 1 per coerenza con il pattern
-- 0-based usato dalla UI F3.

UPDATE Folders SET SortOrder = (
    SELECT COUNT(*) FROM Folders f2
    WHERE f2.WorkspaceId = Folders.WorkspaceId
      AND COALESCE(f2.ParentFolderId, '') = COALESCE(Folders.ParentFolderId, '')
      AND f2.CreatedAt < Folders.CreatedAt
);

UPDATE Prompts SET SortOrder = (
    SELECT COUNT(*) FROM Prompts p2
    WHERE p2.WorkspaceId = Prompts.WorkspaceId
      AND COALESCE(p2.FolderId, '') = COALESCE(Prompts.FolderId, '')
      AND p2.CreatedAt < Prompts.CreatedAt
);

CREATE INDEX IF NOT EXISTS idx_folders_sort
    ON Folders(WorkspaceId, ParentFolderId, SortOrder);
CREATE INDEX IF NOT EXISTS idx_prompts_sort
    ON Prompts(WorkspaceId, FolderId, SortOrder);
```

**Note**:
- `COALESCE(...)` su `ParentFolderId`/`FolderId` necessario perché `NULL = NULL` è false in SQL: senza coalesce i siblings con parent NULL non si raggrupperebbero correttamente.
- Backfill basato su `CreatedAt`: stabile e deterministico sui vault esistenti — se 2 row hanno lo stesso `CreatedAt` (improbabile, granularità secondi), l'ordine fra loro è non specificato ma comunque coerente con qualunque scan.
- Idempotente: `IF NOT EXISTS` su CREATE INDEX evita errori se eseguito 2 volte (anche se `_Migrazioni` table previene seconda esecuzione naturale).
- Costo migrazione: O(N²) sui workspace con molte folder/prompt sotto stesso parent. Su vault tipico (≤500 prompt/workspace) il backfill è < 1s.

### Modifica `apps/client/src-tauri/src/migrazione.rs`

Aggiungere entry n.14 nell'array `MIGRAZIONI` (riga ~17, dopo V013):

```rust
Migrazione {
    versione: 14,
    nome: "sort_order",
    sql: include_str!("../migrations/V014__sort_order.sql"),
},
```

### Aggiornare assert nei test inline (riga ~141)

Cercare e aggiornare 2 punti nei test esistenti:

```rust
// Vecchio:
//   assert!(versione_corrente >= 13)
//   assert_eq!(MIGRAZIONI.last().unwrap().versione, 13)

// Nuovo:
assert!(versione_corrente >= 14);
assert_eq!(MIGRAZIONI.last().unwrap().versione, 14);
```

### Test nuovi in `migrazione.rs#[cfg(test)] mod test`

Aggiungere 2 test:

```rust
#[test]
fn v014_aggiunge_colonne_sortorder() {
    let conn = setup_db_test();
    // Verifica colonne presenti dopo migrazione completa
    let folders_cols: Vec<String> = conn.prepare("PRAGMA table_info(Folders)").unwrap()
        .query_map([], |r| r.get::<_, String>(1)).unwrap().flatten().collect();
    assert!(folders_cols.contains(&"SortOrder".to_string()));
    let prompts_cols: Vec<String> = conn.prepare("PRAGMA table_info(Prompts)").unwrap()
        .query_map([], |r| r.get::<_, String>(1)).unwrap().flatten().collect();
    assert!(prompts_cols.contains(&"SortOrder".to_string()));
}

#[test]
fn v014_backfill_sortorder_stabile_per_creato_at() {
    let conn = setup_db_test();
    // Setup: inserisci 3 folder nello stesso parent con CreatedAt distinti
    crea_workspace_e_user(&conn);
    conn.execute("INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f1','ws1',NULL,'A','/A','2026-01-01','2026-01-01')", []).unwrap();
    conn.execute("INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f2','ws1',NULL,'B','/B','2026-01-02','2026-01-02')", []).unwrap();
    conn.execute("INSERT INTO Folders (Id, WorkspaceId, ParentFolderId, Name, Path, CreatedAt, UpdatedAt) VALUES ('f3','ws1',NULL,'C','/C','2026-01-03','2026-01-03')", []).unwrap();

    // Re-applica V014 backfill (idempotente sui dati nuovi)
    let v014_sql = include_str!("../migrations/V014__sort_order.sql");
    // estrai solo le clausole UPDATE
    for stmt in v014_sql.split(';').filter(|s| s.trim().starts_with("UPDATE")) {
        conn.execute(stmt, []).unwrap();
    }

    let f1_sort: i64 = conn.query_row("SELECT SortOrder FROM Folders WHERE Id='f1'", [], |r| r.get(0)).unwrap();
    let f2_sort: i64 = conn.query_row("SELECT SortOrder FROM Folders WHERE Id='f2'", [], |r| r.get(0)).unwrap();
    let f3_sort: i64 = conn.query_row("SELECT SortOrder FROM Folders WHERE Id='f3'", [], |r| r.get(0)).unwrap();
    assert_eq!(f1_sort, 0);
    assert_eq!(f2_sort, 1);
    assert_eq!(f3_sort, 2);
}
```

(Helper `setup_db_test` e `crea_workspace_e_user` da definire seguendo il pattern esistente nei test di `cartelle.rs:388+`.)

---

## 4. Comando `folder_riordina`

### DTO input — `apps/client/src-tauri/src/cartelle.rs` (aggiungere)

```rust
#[derive(Debug, Deserialize)]
pub struct RiordinaFolder {
    pub id: String,
    pub new_sort: i64,
}
```

### Comando — `apps/client/src-tauri/src/cartelle.rs` (aggiungere accanto a `folder_sposta`)

```rust
/// Riordina una cartella all'interno dei siblings (stesso WorkspaceId + ParentFolderId).
/// Re-pack atomico: gli altri siblings vengono shiftati per fare spazio.
/// Se `new_sort` è oltre il count siblings, viene clampato all'ultimo posto.
#[tauri::command]
pub fn folder_riordina(
    dati: RiordinaFolder,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        atomicamente(conn, |c| {
            // Lookup workspace + parent della folder target
            let (workspace_id, parent_folder_id, current_sort): (String, Option<String>, i64) = c.query_row(
                "SELECT WorkspaceId, ParentFolderId, SortOrder FROM Folders WHERE Id = ?1 AND DeletedAt IS NULL",
                params![&dati.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).map_err(|_| PapErrore::Generico(format!("Cartella {} non trovata", dati.id)))?;

            // Conta siblings (escludendo self) per clampare new_sort
            let sibling_count: i64 = c.query_row(
                "SELECT COUNT(*) FROM Folders
                 WHERE WorkspaceId = ?1
                   AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3",
                params![&workspace_id, &parent_folder_id, &dati.id],
                |r| r.get(0),
            )?;
            let target_sort = dati.new_sort.clamp(0, sibling_count);

            if target_sort == current_sort {
                return Ok(()); // No-op: già al posto giusto
            }

            // Shift atomico: spazio per il target
            if target_sort < current_sort {
                // Spostiamo verso l'alto: shift +1 i siblings tra [target_sort, current_sort-1]
                c.execute(
                    "UPDATE Folders SET SortOrder = SortOrder + 1
                     WHERE WorkspaceId = ?1
                       AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
                       AND DeletedAt IS NULL
                       AND Id != ?3
                       AND SortOrder >= ?4 AND SortOrder < ?5",
                    params![&workspace_id, &parent_folder_id, &dati.id, target_sort, current_sort],
                )?;
            } else {
                // Spostiamo verso il basso: shift -1 i siblings tra [current_sort+1, target_sort]
                c.execute(
                    "UPDATE Folders SET SortOrder = SortOrder - 1
                     WHERE WorkspaceId = ?1
                       AND COALESCE(ParentFolderId, '') = COALESCE(?2, '')
                       AND DeletedAt IS NULL
                       AND Id != ?3
                       AND SortOrder > ?4 AND SortOrder <= ?5",
                    params![&workspace_id, &parent_folder_id, &dati.id, current_sort, target_sort],
                )?;
            }

            // Set target
            c.execute(
                "UPDATE Folders SET SortOrder = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
                params![target_sort, &dati.id],
            )?;
            Ok(())
        })?;
        crate::audit::registra(conn, "folder.riordinato", "Folder", &dati.id, None);
        log::info!("folder_riordina id={} new_sort={}", dati.id, dati.new_sort);
        Ok(())
    })
}
```

**Note**:
- `clamp(0, sibling_count)`: se la UI manda `new_sort` fuori range (es. drag fuori scope), lo riportiamo dentro senza errori.
- No-op early return: evita scrittura inutile quando `new_sort == current_sort`.
- `BEGIN/COMMIT` via helper `atomicamente`: garantisce che lo shift + set finale siano atomici.
- Audit fuori dalla transazione: il design di `audit::registra` non richiede transazione (vedi `cartelle.rs:288`). Se la transazione fallisce, l'audit non viene mai registrato.

---

## 5. Comando `prompt_riordina`

### DTO input — `apps/client/src-tauri/src/cartelle.rs` (aggiungere)

```rust
#[derive(Debug, Deserialize)]
pub struct RiordinaPrompt {
    pub prompt_id: String,
    pub new_sort: i64,
}
```

### Comando — `apps/client/src-tauri/src/cartelle.rs` (aggiungere accanto a `prompt_sposta`)

```rust
/// Riordina un prompt all'interno dei siblings (stesso WorkspaceId + FolderId).
/// Stessa logica di folder_riordina ma scope per FolderId invece che ParentFolderId.
#[tauri::command]
pub fn prompt_riordina(
    dati: RiordinaPrompt,
    state: State<'_, VaultState>,
) -> Result<(), PapErrore> {
    state.with_conn(|conn| {
        atomicamente(conn, |c| {
            let (workspace_id, folder_id, current_sort): (String, Option<String>, i64) = c.query_row(
                "SELECT WorkspaceId, FolderId, SortOrder FROM Prompts WHERE Id = ?1 AND DeletedAt IS NULL",
                params![&dati.prompt_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).map_err(|_| PapErrore::Generico(format!("Prompt {} non trovato", dati.prompt_id)))?;

            let sibling_count: i64 = c.query_row(
                "SELECT COUNT(*) FROM Prompts
                 WHERE WorkspaceId = ?1
                   AND COALESCE(FolderId, '') = COALESCE(?2, '')
                   AND DeletedAt IS NULL
                   AND Id != ?3",
                params![&workspace_id, &folder_id, &dati.prompt_id],
                |r| r.get(0),
            )?;
            let target_sort = dati.new_sort.clamp(0, sibling_count);

            if target_sort == current_sort {
                return Ok(());
            }

            if target_sort < current_sort {
                c.execute(
                    "UPDATE Prompts SET SortOrder = SortOrder + 1
                     WHERE WorkspaceId = ?1
                       AND COALESCE(FolderId, '') = COALESCE(?2, '')
                       AND DeletedAt IS NULL
                       AND Id != ?3
                       AND SortOrder >= ?4 AND SortOrder < ?5",
                    params![&workspace_id, &folder_id, &dati.prompt_id, target_sort, current_sort],
                )?;
            } else {
                c.execute(
                    "UPDATE Prompts SET SortOrder = SortOrder - 1
                     WHERE WorkspaceId = ?1
                       AND COALESCE(FolderId, '') = COALESCE(?2, '')
                       AND DeletedAt IS NULL
                       AND Id != ?3
                       AND SortOrder > ?4 AND SortOrder <= ?5",
                    params![&workspace_id, &folder_id, &dati.prompt_id, current_sort, target_sort],
                )?;
            }

            c.execute(
                "UPDATE Prompts SET SortOrder = ?1, UpdatedAt = datetime('now') WHERE Id = ?2",
                params![target_sort, &dati.prompt_id],
            )?;
            Ok(())
        })?;
        crate::audit::registra(conn, "prompt.riordinato", "Prompt", &dati.prompt_id, None);
        log::info!("prompt_riordina id={} new_sort={}", dati.prompt_id, dati.new_sort);
        Ok(())
    })
}
```

---

## 6. Extend `prompt_get_history` con autore

### Modifiche a `apps/client/src-tauri/src/versioning.rs`

#### Struct `VersioneStorica` (riga 13-25): aggiungere 2 campi finali

```rust
#[derive(Debug, Serialize, Clone)]
pub struct VersioneStorica {
    pub id: String,
    pub prompt_id: String,
    pub version: i64,
    pub titolo: String,
    pub descrizione: Option<String>,
    pub body: String,
    pub visibilita: Option<String>,
    pub target_model: Option<String>,
    pub creato_a: String,
    pub creato_da_user_id: String,
    // NUOVI campi V014
    pub autore_display_name: String,
    pub autore_email: Option<String>,
}
```

#### Query in `prompt_get_history` (riga 131-163): JOIN Users

```rust
#[tauri::command]
pub fn prompt_get_history(
    prompt_id: String,
    state: State<'_, VaultState>,
) -> Result<Vec<VersioneStorica>, PapErrore> {
    state.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT pv.Id, pv.PromptId, pv.Version, pv.Title, pv.Description, pv.Body,
                    pv.Visibility, pv.TargetModel, pv.CreatedAt, pv.CreatedByUserId,
                    u.DisplayName, u.Email
             FROM PromptVersions pv
             JOIN Users u ON u.Id = pv.CreatedByUserId
             WHERE pv.PromptId = ?1
             ORDER BY pv.Version DESC",
        )?;

        let rows = stmt.query_map(params![prompt_id], |row| {
            Ok(VersioneStorica {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                version: row.get(2)?,
                titolo: row.get(3)?,
                descrizione: row.get(4)?,
                body: row.get(5)?,
                visibilita: row.get(6)?,
                target_model: row.get(7)?,
                creato_a: row.get(8)?,
                creato_da_user_id: row.get(9)?,
                autore_display_name: row.get(10)?,
                autore_email: row.get(11)?,
            })
        })?;

        let mut acc = Vec::new();
        for r in rows { acc.push(r?); }
        Ok(acc)
    })
}
```

**Note**:
- `JOIN Users` (NOT `LEFT JOIN`) perché `PromptVersions.CreatedByUserId` è `NOT NULL REFERENCES Users(Id)` — la FK garantisce match.
- Su vault molto vecchi con orphan FK (improbabile ma possibile se Users è stato cancellato senza cascade), `JOIN` perde quelle righe. Acceptable: la cronologia con autore ignoto è meno utile della cronologia rotta.
- `Email` è `Option<String>` lato struct: schema V001 dice `Email TEXT` (nullable). Coerente.

---

## 7. Extend `Statistiche` con `token_medi`

### Modifiche a `apps/client/src-tauri/src/statistiche.rs`

#### Struct `Statistiche` (riga 14-29): aggiungere campo finale

```rust
#[derive(Debug, Serialize, Clone)]
pub struct Statistiche {
    pub totali: Totali,
    pub top_usati: Vec<PromptUsato>,
    pub non_usati: Vec<PromptInattivo>,
    pub per_tag: Vec<DistribuzioneTag>,
    pub per_target_model: Vec<DistribuzioneStringa>,
    pub per_visibilita: Vec<DistribuzioneStringa>,
    pub top_importati: Vec<PromptImportato>,
    pub lint_health: LintHealth,
    /// Media char-count del Body dei prompt attivi, divisa per 4 (proxy token cl100k).
    /// 0 se nessun prompt attivo. UI mostra come "~N token medi".
    pub token_medi: i64,
}
```

#### Helper nuovo (vicino agli helper esistenti `calcola_totali`, `top_usati`, ecc.)

```rust
fn calcola_token_medi(conn: &Connection) -> Result<i64, PapErrore> {
    // Proxy: 1 token ≈ 4 chars (heuristic OpenAI cl100k).
    // LENGTH(Body) in SQLite è char-count UTF-8, NON byte-count: già corretto.
    let avg_chars: f64 = conn.query_row(
        "SELECT COALESCE(AVG(CAST(LENGTH(Body) AS REAL)), 0.0)
         FROM Prompts WHERE DeletedAt IS NULL",
        [],
        |r| r.get(0),
    )?;
    Ok((avg_chars / 4.0).round() as i64)
}
```

**Note**:
- `CAST(... AS REAL)`: garantisce divisione floating-point invece che integer (SQLite altrimenti tronca).
- `COALESCE(..., 0.0)`: AVG() su 0 righe ritorna NULL → fallback a 0.
- Costo: 1 SCAN aggiuntivo su `Prompts`. Tabella tipica < 1000 righe per workspace, costo < 1ms.

#### Wireup in `statistiche_query` (riga 332+): aggiungere campo al return

```rust
#[tauri::command]
pub fn statistiche_query(
    state: State<'_, VaultState>,
) -> Result<Statistiche, PapErrore> {
    state.with_conn(|conn| {
        Ok(Statistiche {
            totali: calcola_totali(conn)?,
            top_usati: top_usati(conn)?,
            non_usati: non_usati(conn)?,
            per_tag: per_tag(conn)?,
            per_target_model: per_target_model(conn)?,
            per_visibilita: per_visibilita(conn)?,
            top_importati: top_importati(conn)?,
            lint_health: lint_health(conn)?,
            token_medi: calcola_token_medi(conn)?,
        })
    })
}
```

---

## 8. Registrazione comandi in `lib.rs`

### File: `apps/client/src-tauri/src/lib.rs`, riga ~298 (`tauri::generate_handler!`)

Cercare il cluster `cartelle::*` (riga 343-349 secondo audit), aggiungere 2 entry:

```rust
.invoke_handler(tauri::generate_handler![
    // ... comandi esistenti ...
    cartelle::folder_lista,
    cartelle::folder_crea,
    cartelle::folder_aggiorna,
    cartelle::folder_elimina,
    cartelle::folder_sposta,
    cartelle::folder_riordina,        // NUOVO V014
    cartelle::prompt_sposta,
    cartelle::prompt_riordina,        // NUOVO V014
    // ...
])
```

(Nessun cambio a `prompt_get_history` o `statistiche_query` — restano lì come prima, solo internal extended.)

---

## 9. Test attesi

### Test inline `cartelle.rs` (`#[cfg(test)] mod test`)

5 test nuovi:

```rust
#[test]
fn folder_riordina_no_op_se_stessa_posizione() {
    let conn = setup_db_test();
    crea_workspace_e_user(&conn);
    crea_folder(&conn, "f1", None, 0);
    crea_folder(&conn, "f2", None, 1);
    // Riordina f1 a SortOrder 0 (era già 0)
    folder_riordina_inner(&conn, "f1", 0).unwrap();
    assert_eq!(get_sort(&conn, "Folders", "f1"), 0);
    assert_eq!(get_sort(&conn, "Folders", "f2"), 1);
}

#[test]
fn folder_riordina_sposta_giu_shifta_siblings() {
    let conn = setup_db_test();
    crea_workspace_e_user(&conn);
    crea_folder(&conn, "f1", None, 0);
    crea_folder(&conn, "f2", None, 1);
    crea_folder(&conn, "f3", None, 2);
    // Sposta f1 da 0 a 2 → f2 e f3 shiftano a 0 e 1
    folder_riordina_inner(&conn, "f1", 2).unwrap();
    assert_eq!(get_sort(&conn, "Folders", "f1"), 2);
    assert_eq!(get_sort(&conn, "Folders", "f2"), 0);
    assert_eq!(get_sort(&conn, "Folders", "f3"), 1);
}

#[test]
fn folder_riordina_sposta_su_shifta_siblings() {
    // Setup: f1=0, f2=1, f3=2. Sposta f3 a 0 → f1 e f2 shiftano a 1 e 2
    // ... assert ...
}

#[test]
fn folder_riordina_clamp_oltre_range() {
    // Setup: 3 folder. Riordina f1 a SortOrder 999 → diventa 2 (ultimo)
    // ... assert ...
}

#[test]
fn folder_riordina_id_inesistente_errore() {
    let conn = setup_db_test();
    let res = folder_riordina_inner(&conn, "ghost", 0);
    assert!(matches!(res, Err(PapErrore::Generico(_))));
}
```

(Helper `folder_riordina_inner` che bypassa `State` per test diretto su `&Connection`. Pattern identico per `prompt_riordina_inner`.)

### Test `versioning.rs`

1 test nuovo:

```rust
#[test]
fn prompt_get_history_include_autore() {
    let conn = setup_db_test();
    crea_workspace_e_user(&conn);  // crea User u1 con DisplayName="Roberto", Email="r@example.com"
    crea_prompt_con_versione(&conn, "p1", "u1");

    let result = prompt_get_history_inner(&conn, "p1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].autore_display_name, "Roberto");
    assert_eq!(result[0].autore_email, Some("r@example.com".to_string()));
}
```

### Test `statistiche.rs`

1 test nuovo:

```rust
#[test]
fn calcola_token_medi_proxy_chars_diviso_4() {
    let conn = setup_db_test();
    crea_workspace_e_user(&conn);
    // 3 prompt: body length 16, 32, 48 chars → AVG=32 → token_medi = 32/4 = 8
    crea_prompt_con_body(&conn, "p1", "a".repeat(16));
    crea_prompt_con_body(&conn, "p2", "b".repeat(32));
    crea_prompt_con_body(&conn, "p3", "c".repeat(48));
    assert_eq!(calcola_token_medi(&conn).unwrap(), 8);
}

#[test]
fn calcola_token_medi_zero_se_no_prompt() {
    let conn = setup_db_test();
    assert_eq!(calcola_token_medi(&conn).unwrap(), 0);
}
```

### Test migrazione (`migrazione.rs`)

2 test già descritti in §3 (`v014_aggiunge_colonne_sortorder`, `v014_backfill_sortorder_stabile_per_creato_at`).

### Coverage target

- 9 test nuovi totali (5 cartelle + 1 versioning + 2 statistiche + 2 migrazione, escluso aggiornamento test esistenti per assert numerici).
- Stima coverage push: +0.5-1pt sul lib totale (currently 74.14%).
- **CI gate 70% mantenuto** (decisione v0.7.0).

---

## 10. Edge case

| # | Edge case | Comportamento atteso | Coperto da |
|---|---|---|---|
| 1 | `folder_riordina` su folder cancellata (`DeletedAt IS NOT NULL`) | Errore `PapErrore::Generico("Cartella X non trovata")` | Test inesistente sufficiente (filtra in lookup) |
| 2 | `folder_riordina` con `new_sort < 0` | Clamp a 0 (primo posto) | Test clamp |
| 3 | `folder_riordina` con `new_sort > sibling_count` | Clamp a sibling_count (ultimo posto) | Test clamp |
| 4 | `folder_riordina` con id che esiste in altro workspace | Errore "non trovata" — la query include ovviamente solo le folder del workspace di chi le ha richieste? **NO**: oggi il lookup non filtra workspace. Verificare se sufficiente aut tramite VaultState (presunta singola identità per vault). |
| 5 | Race: 2 client riordinano contemporaneamente nella stessa folder | SQLite lock + transazione = serializzazione. Il secondo vede stato post-primo. UI eventualmente refresha la lista (F3 responsabilità). |
| 6 | Folder con SortOrder duplicate dopo backfill (improbabile, vault con CreatedAt identico) | Riordino li distingue al primo update successivo. Niente crash. |
| 7 | `prompt_get_history` su prompt senza versioni (impossibile post-V002 backfill) | Ritorna `Vec` vuoto, no errore |
| 8 | Body con caratteri multi-byte (es. emoji 4-byte UTF-8) | `LENGTH(Body)` SQLite ritorna char-count corretto (NOT byte) — proxy `/4` rimane consistente |
| 9 | Body NULL (impossibile per schema, `Body NOT NULL`) | N/A |
| 10 | `Users` cancellato dopo creazione versione (improbabile, no CASCADE) | `JOIN` stretto perde la versione. Acceptable: history rotta senza autore = noise inferiore a errore generico |

---

## 11. Rischi specifici

| # | Rischio | Probabilità | Mitigazione |
|---|---|---|---|
| 1 | Vault esistenti con > 10k prompt — backfill V014 lento (O(N²) per parent) | LOW (workspace tipici < 500 prompt) | Backfill via subquery COUNT (O(N²) sì ma N è siblings dello stesso parent, tipicamente < 50) |
| 2 | Conflitto FK durante migrazione V014 (utente cancella prompt mentre upgrade gira) | LOW (migrazione runs at boot, no concurrent writes) | `_Migrazioni` table previene re-esecuzione |
| 3 | Cmd `folder_riordina` chiamato da UI legacy (preferences senza redesign attivo) | LOW (UI legacy non chiama il cmd) | Cmd è additivo: nessuna superficie attuale lo invoca |
| 4 | `audit::registra` fallisce dopo successo transazione (es. spazio disco esaurito) | LOW | `audit::registra` non è transazionale: log perso ma operazione applicata. Comportamento esistente di `folder_sposta`/`prompt_sposta` |
| 5 | `prompt_get_history` rotta da JOIN se `Users` ha orphan rows | LOW | Schema V001 ha FK strict. Test caso JOIN miss è acceptable (vedi edge case #10) |
| 6 | Calcolo `token_medi` con tokenizer reale richiesto dal designer post-merge | LOW | Proxy `LENGTH/4` documentato in commento Rust; switch a tiktoken-rs è cambio interno non breaking |
| 7 | UI consumer-side legge `SortOrder` ma non lo passa nelle altre cmd (`folder_aggiorna`) → reset a 0? | MED | NON cambiamo `folder_aggiorna`/`prompt_aggiorna` in V014. Solo `folder_riordina` e `prompt_riordina` modificano `SortOrder`. Lasciare invariati gli altri evita reset accidentali. |

---

## 12. Exit criteria

PR `feat/redesign-backend-v014` può fare merge solo se:

- [ ] `migrations/V014__sort_order.sql` creato con 2 ALTER + 2 UPDATE backfill + 2 CREATE INDEX
- [ ] `migrazione.rs MIGRAZIONI` aggiornato con entry n.14 + asserts test inline (`>= 14`, `last == 14`)
- [ ] `cartelle.rs` ha `folder_riordina` + `prompt_riordina` con DTO + audit + log
- [ ] `versioning.rs::VersioneStorica` esteso con `autore_display_name` + `autore_email`
- [ ] `versioning.rs::prompt_get_history` query con `JOIN Users`
- [ ] `statistiche.rs::Statistiche` esteso con `token_medi`
- [ ] `statistiche.rs::statistiche_query` chiama `calcola_token_medi`
- [ ] `lib.rs::generate_handler!` registra `cartelle::folder_riordina` + `cartelle::prompt_riordina`
- [ ] 9 test nuovi inline tutti verdi (5 cartelle + 1 versioning + 2 statistiche + 1 migrazione, oltre asserts updated)
- [ ] `cargo test --lib` verde
- [ ] `cargo clippy --all-targets -- -D warnings` verde
- [ ] `cargo llvm-cov --lib --summary-only --fail-under-lines 70` ✅
- [ ] CI client-build.yml verde (rust-test job)
- [ ] PR description menziona: V014 schema, 2 cmd nuovi, extend `VersioneStorica`+2 campi, extend `Statistiche`+1 campo, no breaking change client legacy

---

## 13. Note out-of-scope (UI consumer-side)

Quando F2/F3 della UI consumeranno questo backend, vanno aggiornati:

| File | Modifica necessaria | Fase |
|---|---|---|
| `cartelle.rs::folder_lista` (struct `Cartella`) | Aggiungere `SortOrder` al SELECT + ordinare `ORDER BY SortOrder ASC, Path COLLATE NOCASE` | F2 |
| `libreria.rs::libreria_lista` (o equivalente per Prompts) | Aggiungere `SortOrder` al SELECT + ordinare `ORDER BY SortOrder ASC, UpdatedAt DESC` | F3 |
| `apps/client/src/lib/types.ts` (TS types client) | Aggiungere `sort_order?: number` a `Cartella` e `Prompt` | F2/F3 |
| Component `RightRail/Cronologia` Svelte | Calcolare avatar hash da `autore_email` (SHA1+HSL) — vedi blueprint F5 | F5 |
| Component `Insight` Svelte | Mostrare `token_medi` con label "~N token medi" | F8 |

**NON è scope di V014 backend** modificare `folder_lista` o `libreria_lista`. Restano invariati e funzionano (ignorano `SortOrder` se non lo selezionano). Cambieranno solo quando F2/F3 li wirepup col nuovo ordering.

---

> **Stato blueprint**: 1.0 finale — pronto per esecuzione. Aggiornare se durante implementazione emergono cambi (es. tokenizer reale vs proxy `/4`, race condition non previste).
