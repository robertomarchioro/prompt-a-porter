# Blueprint operativo — Ordito F1 "Fondamenta"

> **Stato**: pronto per l'implementazione. Nessun codice ancora.
> **Prerequisito**: [`../ordito-sync-log.md`](../ordito-sync-log.md) **v3**
> (design generale + 7 Decisioni chiuse del 2026-07-07). Questo documento è
> il taglio esecutivo della fase F1: firme esatte, DDL, algoritmi, ordine
> delle PR, piano di test. In caso di conflitto vince il design generale.
> **Obiettivo F1**: oplog sempre-on scritto in transazione da un choke-point
> unico, HLC, indici derivati incrementali fuori TX, key storage con
> fallback, GC locale + retention ~90gg, push riparato verso il `papsync`
> attuale, property test di convergenza. **Valore anche senza P2P.**
> **Fuori scope F1**: cifratura segmenti, master seed/VSK/recovery phrase,
> trasporti T1/T2/T3, snapshot multi-peer a quorum, varianti-da-conflitto,
> staging team (→ F2/F3/F4).

## Fatti di codice su cui è costruito questo blueprint

Verificati il 2026-07-07:

- `VaultState::with_conn` (`vault.rs:96`) passa `&Connection` (non `&mut`):
  le transazioni esplicite si aprono con `conn.unchecked_transaction()`.
- I comandi mutanti attuali girano **senza transazione esplicita** — es.
  `prompt_aggiorna` (`editor.rs:264`) esegue UPDATE + sincronizza_tags +
  snapshot + FTS + embedding + imports + audit come statement separati in
  auto-commit. Un crash a metà lascia stato parziale già oggi: il
  choke-point con TX è un miglioramento anche a prescindere dal log.
- `prompt_aggiorna` è **UPDATE full-row senza read-before-write**;
  `sincronizza_tags` (`editor.rs:91`) è DELETE-all + re-INSERT;
  `ricostruisci_fts` (`editor.rs:208`) è un **rebuild totale** della tabella
  FTS a ogni mutazione; `aggiorna_embedding` può invocare il runtime ML
  dentro `with_conn`.
- 95 `#[tauri::command]` totali; i mutanti sono concentrati in ~12 moduli
  (editor, cartelle, cestino, versioning, varianti, fork, rating, golden,
  regression, import_export, segnaposti_globali, provider_ai*).
  (*provider_ai NON entra nel log — Decisione: `ProviderConfig` mai loggata.)
- Id: `prm-`/`tag-`/`fld-` + 12 hex di timestamp ms + 8 hex random
  (`genera_id`, `editor.rs:78`). Timestamp DB: `datetime('now')` TEXT UTC.
- Migrazioni: `migrations/V001..V015__*.sql` embedded via `include_str!`,
  applicate in ordine da `migrazione.rs`, idempotenti, senza down.
- `SyncMeta` esiste (V001) ma è inutilizzata; `sync.ts` è pull-only;
  `preferenze.rs:190-193` ha il TODO keychain per `sync_token`.

## Architettura del modulo

Nuovo modulo `apps/client/src-tauri/src/ordito/` (modulo, non crate: evita
il refactor a workspace; l'estrazione a crate è rimandata a quando servirà
la condivisione col server, F4):

```
src/ordito/
├─ mod.rs        // re-export API pubblica: apply_change, OrditoCtx
├─ hlc.rs        // Hlc: tipo, encoding, tick locale, receive, anti-deriva
├─ record.rs     // OrditoRecord: struct, CBOR encode/decode, hash BLAKE3
├─ chain.rs      // catena per-install: prev/hash/sig, verify, equivocation
├─ keys.rs       // device key Ed25519 + install_nonce: keyring + fallback
├─ apply.rs      // apply_change (produzione) + applica_remoto (consumo F1: push/pull papsync)
├─ diff.rs       // diff field-level old/new, mappe campo→valore
├─ gc.rs         // GC locale + retention (Decisione 1/6, caso K=1)
└─ dirty.rs      // coda reindex FTS/embeddings fuori TX
```

Nuove dipendenze (tutte da pinnare come da policy Dependabot/canary):
`ed25519-dalek` (firma), `blake3` (hash), `ciborium` (CBOR),
`keyring` (device key — chiude anche il TODO `sync_token`),
`proptest` (dev-dependency). NIENTE `snow`/`mdns-sd` (F3), niente
compressione segmenti (F2).

## Migrazione V016 — `V016__ordito_fondamenta.sql`

```sql
-- Log canonico locale (Decisione 3: il log autoritativo vive nel vault).
CREATE TABLE Oplog (
    Seq       INTEGER PRIMARY KEY AUTOINCREMENT,  -- ordine locale di append
    Hlc       TEXT    NOT NULL,                   -- encoding ordinabile (v. §HLC)
    Vault     TEXT    NOT NULL,                   -- id vault (anti replay cross-vault)
    SchemaV   INTEGER NOT NULL,                   -- versione schema alla scrittura
    Op        TEXT    NOT NULL CHECK (Op IN ('upsert','delete')),
    Entity    TEXT    NOT NULL,
    EntityId  TEXT    NOT NULL,                   -- composito "a|b" per PromptTags
    Fields    BLOB,                               -- mappa campo→valore, CBOR (NULL per delete)
    Install   TEXT    NOT NULL,                   -- dev_<fp>.<nonce>
    Prev      TEXT,                               -- hash del record precedente stessa install (NULL = primo)
    Hash      TEXT    NOT NULL UNIQUE,            -- BLAKE3 del record canonico
    Sig       TEXT    NOT NULL                    -- Ed25519 su Hash
);
CREATE INDEX idx_oplog_hlc     ON Oplog(Hlc);
CREATE INDEX idx_oplog_entity  ON Oplog(Entity, EntityId);
CREATE INDEX idx_oplog_install ON Oplog(Install, Seq);

-- LWW-guard per campo (+ pseudo-campo '__tombstone__').
CREATE TABLE OrditoApplied (
    Entity   TEXT NOT NULL,
    EntityId TEXT NOT NULL,
    Field    TEXT NOT NULL,
    Hlc      TEXT NOT NULL,
    PRIMARY KEY (Entity, EntityId, Field)
) WITHOUT ROWID;

-- Record remoti in attesa (FK mancante / create mancante / schema futuro).
CREATE TABLE OrditoPending (
    Hash      TEXT PRIMARY KEY,
    Motivo    TEXT NOT NULL CHECK (Motivo IN ('fk','create','schema')),
    AttesaDi  TEXT,                               -- entityId atteso (per fk/create)
    Record    BLOB NOT NULL,                      -- CBOR integrale
    RicevutoA TEXT NOT NULL
);

-- Cursori dei peer (F1: solo il server papsync; F2+: le altre install).
CREATE TABLE OrditoPeers (
    PeerId      TEXT PRIMARY KEY,                 -- 'papsync' | install id
    CursoreSeq  INTEGER NOT NULL DEFAULT 0,       -- ultimo Oplog.Seq spedito/confermato
    CursoreHlc  TEXT,
    UltimoVisto TEXT
);

-- Coda reindex derivati, marcata in-TX, consumata fuori TX.
CREATE TABLE OrditoDirty (
    Entity    TEXT NOT NULL,
    EntityId  TEXT NOT NULL,
    Tipo      TEXT NOT NULL CHECK (Tipo IN ('fts','embedding','imports')),
    MarcatoA  TEXT NOT NULL,
    PRIMARY KEY (Entity, EntityId, Tipo)
) WITHOUT ROWID;

-- Stato del motore (chiave/valore).
CREATE TABLE OrditoMeta (
    Chiave TEXT PRIMARY KEY,
    Valore TEXT NOT NULL
);
-- Chiavi previste: 'install_id', 'hlc_ultimo', 'retention_giorni' (default '90'),
-- 'vault_id', 'schema_v'. La device key NON sta qui (v. §Key storage).
```

Note di migrazione:

- **Nessun backfill**: il log parte vuoto; lo stato corrente del vault è la
  base implicita (equivale a uno snapshot al momento di V016). Time-travel e
  audit firmato partono da qui; la cronologia `PromptVersions` preesistente
  resta intatta e non c'entra con la retention del log.
- `vault_id`: generato in V016 se assente (`vlt-` + genera_id), salvato in
  `OrditoMeta` — entra nel payload firmato di ogni record.

## HLC — spec di implementazione (`hlc.rs`)

```rust
pub struct Hlc { pub physical_ms: u64, pub counter: u32, pub install: String }
```

- **Encoding** (ordinabile come stringa, coerente con gli indici TEXT):
  `{:012x}-{:08x}-{install}` → es. `01890b2f3a7c-00000003-dev_a1b2.7f3e`.
  48 bit ms + 32 bit counter (Decisione da review: 16 bit overflowava sui
  bulk import).
- **Tick locale** (`Hlc::tick`): `physical = max(now_ms, ultimo.physical)`;
  se uguale incrementa `counter`, altrimenti `counter = 0`. L'ultimo HLC
  emesso è persistito in `OrditoMeta('hlc_ultimo')` **nella stessa TX** del
  record che lo usa (riavvio ⇒ mai HLC duplicati o regressivi).
- **Receive** (`Hlc::receive(remoto)`): `physical = max(now, locale, remoto)`
  con regole standard del paper; chiamato dal layer di apply per OGNI record
  remoto, su qualunque trasporto (in F1: la risposta di pull da papsync).
- **Bulk**: i comandi di import chiamano `tick_blocco(n)` che riserva n HLC
  monotoni in una sola persistenza di `hlc_ultimo`.
- **Anti-deriva**: se `now_ms > max_physical_visto + 10 min` → log warn +
  evento UI non bloccante (F1: solo log; la UI arriva col pannello sync).

## Key storage (`keys.rs`)

Scopo F1: la sola **device key** (Ed25519) + `install_nonce`. Master seed,
VSK e recovery phrase sono F2.

- **Percorso primario**: crate `keyring` — service `com.pap.client`,
  entry `ordito-device-key`. La chiave è **machine-bound per design**: NON
  vive nel vault, così il restore del file vault su un'altra macchina non
  porta con sé l'identità (⇒ niente equivocation da restore; la nuova
  macchina genera install nuova, come da design v3).
- **Fallback** (keychain assente o in errore — Linux headless, DPAPI rotto):
  file `ordito-device-key.enc` nella **app-data locale** (mai nel vault, mai
  su share), cifrato XChaCha20-Poly1305 con chiave derivata **Argon2id dalla
  password del vault** (richiesta comunque all'apertura; parametri = quelli
  di `vault.rs`). Per vault non cifrati: file `0600` in chiaro + warning una
  tantum (stessa postura dell'attuale `sync_token`, dichiarata onestamente).
- `install_id = "dev_" + hex(BLAKE3(pubkey))[..8] + "." + nonce4hex`,
  persistito in `OrditoMeta('install_id')`; il nonce si rigenera se la
  device key viene ri-generata (chiave persa ⇒ nuova identità, mai perdita
  dati).
- **Bonus di fase**: `sync_token` migra dallo stesso modulo (chiude il TODO
  di `preferenze.rs:190` e il debito #455/rinvii §sync_token).

## `apply_change` — il choke-point (`apply.rs`, `diff.rs`)

Firma e contratto:

```rust
pub enum Mutazione<'a> {
    Upsert { old: Option<&'a CampiMappa>, new: &'a CampiMappa },
    Delete { old: &'a CampiMappa },
}

/// UNICO punto di scrittura per le entità loggate. Da chiamare DENTRO una
/// transazione già aperta (tx). Non apre TX da solo: il comando chiamante
/// possiede la transazione (v. pattern sotto).
pub fn apply_change(
    tx: &Connection,          // dentro unchecked_transaction()
    ctx: &OrditoCtx,          // install_id, vault_id, schema_v, signer, hlc
    entity: Entita,           // enum tipizzato, non stringa libera
    entity_id: &str,
    mutazione: Mutazione<'_>,
) -> Result<EsitoApply, PapErrore>
```

Algoritmo (produzione locale):

1. **Diff field-level**: `diff::calcola(old, new)` → mappa dei soli campi
   cambiati (confronto per valore normalizzato: trim dove il comando già
   trimma). **Diff vuoto ⇒ `EsitoApply::NessunCambiamento`, nessun record,
   nessun UPDATE** (idempotenza by-design, testata come property).
2. **HLC**: `ctx.hlc.tick()` (o slot dal blocco bulk).
3. **Record**: costruisci `OrditoRecord` (CBOR canonico → BLAKE3 → firma
   Ed25519; `prev` = `Hash` dell'ultimo record di questa install, letto da
   `Oplog` con `MAX(Seq) WHERE Install = ?`).
4. **Scritture in TX** (tutte o nessuna):
   `INSERT INTO Oplog` → `UPDATE/INSERT/soft-DELETE` sulla tabella proiettata
   → upsert `OrditoApplied` per ogni campo (per delete: `__tombstone__`) →
   `INSERT OR REPLACE INTO OrditoDirty` (fts sempre; embedding solo se Body/
   Name cambiati; imports solo se Body cambiato) → `OrditoMeta('hlc_ultimo')`.
5. L'`AuditLog` applicativo resta (già append-only): registrato dal comando
   come oggi, dentro la stessa TX.

Pattern di migrazione di un comando (esempio `prompt_aggiorna`):

```rust
state.with_conn(|conn| {
    let tx = conn.unchecked_transaction()?;          // ← NUOVO: TX esplicita
    let old = leggi_prompt_campi(&tx, &dati.id)?;    // ← NUOVO: read-before-write
    let new = campi_da_aggiornamento(&dati);
    ordito::apply_change(&tx, &ctx, Entita::Prompt, &dati.id,
        Mutazione::Upsert { old: Some(&old), new: &new })?;
    sincronizza_tags_diff(&tx, &ctx, &dati.id, &dati.tag_nomi)?;  // v. sotto
    if dati.crea_snapshot { snapshot_versione(&tx, ...)?; }       // → apply_change(Version, create)
    audit::registra(&tx, ...);
    tx.commit()?;
    Ok(())
})
// FUORI dalla TX e fuori da with_conn: il worker dirty consuma FTS/embedding.
```

Cosa **esce** dalla transazione rispetto a oggi: `ricostruisci_fts` (rebuild
totale → riga incrementale via dirty), `aggiorna_embedding` (runtime ML →
dirty), `aggiorna_imports` (parsing → dirty). La TX diventa più corta di
quella implicita attuale nonostante l'append del log.

### `sincronizza_tags` → diff add/remove

Sostituita da `sincronizza_tags_diff`: legge le associazioni correnti,
calcola `da_aggiungere`/`da_rimuovere`, e per ciascuna emette
`apply_change(Entita::PromptTag, "{pid}|{tid}", Upsert/Delete)` (semantica
LWW-element-set per membro, entityId composito — design v3). La creazione di
un tag nuovo passa da `apply_change(Entita::Tag, ..., create)` dentro
`upsert_tag_id`. `tag_aggiungi_pure`/`tag_rimuovi_pure` (menu contestuale)
riusano le stesse primitive.

### Consumo (F1: `applica_remoto`)

Per il pull da `papsync` (che oggi parla full-row LWW, non record): il delta
ricevuto viene convertito in upsert row-level applicati con il **guard
OrditoApplied** al posto dell'attuale guard `UpdatedAt < ?` di
`sync_applica_delta` (`sync.rs`). I record così applicati NON rientrano nel
log locale come produzioni proprie (niente echo verso il server): si marca
il cursore in `OrditoPeers('papsync')`. Il consumo di record Ordito nativi
(da altre install) è F2 — ma `applica_remoto` è scritto già contro
`OrditoRecord` per non riscriverlo.

## Indici derivati incrementali (`dirty.rs`)

- **FTS per riga**: `fts_aggiorna_riga(conn, prompt_id)` = `DELETE FROM
  PromptsFts WHERE PromptId=?` + `INSERT ... WHERE p.Id=?` (stessa SELECT di
  oggi, filtrata). Il rebuild totale resta solo come comando di riparazione
  (Impostazioni → Sviluppo).
- **Worker**: task asincrono (tauri `async_runtime`) che drena `OrditoDirty`
  a batch: per ogni riga prende `with_conn` breve, esegue il reindex
  idempotente, cancella la riga dirty **solo a successo** (retry con backoff
  a fallimento; l'embedding senza Session fa graceful-skip lasciando la riga
  per il retry successivo, comportamento odierno preservato).
- Debounce: righe dirty della stessa entità coalescono per PK — un autosave
  ogni 2s non accoda 30 reindex.

## GC locale + retention (`gc.rs`) — Decisioni 1 e 6, caso peer-singolo

Nel vault senza peer (F1) la compattazione degenera nel caso semplice:

- La **proiezione è lo snapshot** (K=1, autofirmato: nessuna controfirma).
- Job periodico (all'apertura vault + ogni 24h): `DELETE FROM Oplog WHERE
  Hlc < taglio(retention_giorni)`, escludendo la **coda non spedita** ai
  peer registrati (`Seq > min(CursoreSeq)` di `OrditoPeers`) — con papsync
  configurato il log non viene mai tagliato oltre il cursore di push.
  `OrditoApplied` conserva gli HLC di guardia anche dopo il taglio, quindi
  lo stato proiettato resta completo e protetto.
- Tombstone: in peer-singolo sono tagliabili con la stessa regola (nessuna
  install terza può far risorgere nulla). La GC causale multi-peer è F2.
- `retention_giorni` (default 90) in `OrditoMeta`, esposta in Impostazioni →
  Dati con il testo onesto: "la cronologia locale copre gli ultimi X giorni".
- All'attivazione del sync (F2) la GC locale si disattiva a favore del
  regime causale: la transizione è già prevista qui per non dover toccare
  `gc.rs` due volte (flag: `OrditoPeers` con install ≠ 'papsync' ⇒ regole F2).

## Push riparato verso `papsync` (chiude il gap pull-only)

F1 NON cambia il protocollo del server (il server oplog-nativo è F4): usa
l'API esistente `/sync/push` full-row, alimentata dal log:

1. nuovo comando `sync_prepara_push()` → legge `Oplog` con
   `Seq > OrditoPeers('papsync').CursoreSeq`, raggruppa per entityId,
   proietta lo **stato corrente** delle entità toccate (il server è LWW
   full-row: gli si manda la riga intera, `Visibility='workspace'` only,
   come da suo filtro) e ritorna il `SyncPushRequest` TS-compatibile;
2. `sync.ts::eseguiSync()` diventa push→pull: POST `/sync/push`, aggiorna
   `OrditoPeers` col nuovo cursore **solo su 2xx**, poi pull come oggi
   (via `applica_remoto`); i `conflicts` della risposta alimentano il
   contatore `syncState.conflitti` (che oggi è sempre 0);
3. `SyncMeta` viene finalmente usata (LastSyncAt/LastError) al posto dello
   stato in `preferenze.json` — sana l'incoerenza censita nella review.

## Sequenza delle PR (atomiche, TDD, auto-merge su CI verde)

| PR | Contenuto | Dipende da | Stima |
|---|---|---|---|
| F1-PR1 | V016 + `ordito/` scheletro: `hlc.rs`, `record.rs`, `chain.rs`, `diff.rs` + `keys.rs` (keyring+fallback) — nessun comando toccato; property test HLC/record/chain | — | media |
| F1-PR2 | `apply.rs` + `dirty.rs` + worker; `editor.rs` migrato (prompt_crea/aggiorna, sincronizza_tags_diff, tag_aggiungi/rimuovi); FTS incrementale | PR1 | alta |
| F1-PR3 | `cartelle.rs` + `cestino.rs` + `versioning.rs` migrati (folder LWW, soft-delete→tombstone, version→create) | PR2 | media |
| F1-PR4 | `varianti.rs` + `fork.rs` + `rating.rs` + `golden/regression` + `segnaposti_globali` migrati | PR2 | media |
| F1-PR5 | `import_export.rs` migrato (HLC a blocco, import massivi) | PR2 | media |
| F1-PR6 | `gc.rs` (GC locale + retention) + impostazione UI in sezione Dati | PR2 | bassa |
| F1-PR7 | push papsync (`sync_prepara_push` + `sync.ts` push→pull + `applica_remoto` con guard OrditoApplied + `SyncMeta`) | PR3 | media |
| F1-PR8 | simulatore 2-install in-memory + property test di convergenza end-to-end; hardening dei casi emersi | PR7 | media |

Nota: `prompt_registra_uso` (UseCount/LastUsedAt) NON passa dal log
(Decisione 1/copertura: locale in F1, G-counter in F3) — resta com'è,
annotato nel codice.

Ogni PR: test prima (RED→GREEN), coverage `ordito/` ≥80% dal giorno 1
(`cargo llvm-cov`), CI `client-build` verde prima del merge (i path
`apps/client/**` la attivano, mappa path→workflow già censita in
`docs/contribuire/ci-workflows.md`).

## Piano dei property test (proptest)

Strategie generatrici: storie casuali di mutazioni (create/update/delete/
tag add-remove) su 1-3 install simulate, con clock sfasati e counter burst.

Proprietà da dimostrare (ognuna un test):

1. **Convergenza**: qualunque permutazione/interleaving di apply dello
   stesso insieme di record ⇒ proiezione identica byte-a-byte (dump
   ordinato delle tabelle).
2. **Idempotenza**: applicare due volte qualunque prefisso ⇒ stato identico.
3. **Diff vuoto ⇒ nessun record**: `apply_change(old==new)` non scrive nulla
   (né Oplog né UpdatedAt).
4. **Delete semantics**: tombstone domina HLC ≤; upsert con HLC > resuscita;
   mai resurrezione da record con HLC <.
5. **HLC**: monotonia locale attraverso riavvii simulati (persistenza
   `hlc_ultimo`); receive mai regressivo; nessun duplicato sotto burst
   (tick_blocco).
6. **Catena**: `verify_chain` accetta ogni storia prodotta; rifiuta
   troncamenti/bit-flip; rileva equivocation (due record stesso prev).
7. **LWW-element-set tag**: add/remove concorrenti convergono; il remove con
   HLC più alto vince (niente tag fantasma).
8. **Round-trip record**: CBOR encode→decode→encode stabile (hash invariato).

Golden test deterministici (non property): fixture di record `schema=16`
salvate come file — diventeranno il primo caso del registry di upcaster in
F2+.

## Criteri di completamento F1

- [ ] Ogni comando mutante delle entità loggate passa da `apply_change`
      dentro una TX esplicita (grep di guardia in CI: nessun
      `execute("UPDATE Prompts` fuori da `ordito/` e dai moduli migrati).
- [ ] `ricostruisci_fts` totale non è più chiamata da nessun comando di
      mutazione (solo riparazione manuale).
- [ ] Vault nuovo e vault migrato da V015 funzionano identici (smoke test
      migrazione su fixture).
- [ ] Push+pull round-trip col `papsync` attuale: modifica locale → push →
      visibile a un secondo client → pull inverso (test di integrazione,
      riusa il pattern di `apps/server/internal/integration_test.go`).
- [ ] Retention: vault con log >90gg viene compattato senza perdita di stato
      proiettato; la coda non spedita non viene mai tagliata.
- [ ] Coverage `ordito/` ≥80%; coverage client complessiva non regredisce
      sotto il gate corrente.
- [ ] Latenza di salvataggio prompt (autosave) non peggiorata rispetto a
      main (benchmark prima/dopo: la TX perde il rebuild FTS e guadagna
      l'append — atteso pari o migliore).

## Rischi operativi e mitigazioni

- **`unchecked_transaction` su `&Connection`**: corretto qui perché il Mutex
  di `VaultState` serializza l'accesso (una sola TX viva per volta); da
  documentare nel codice per i posteri.
- **Firma in-TX**: Ed25519 firma in ~50µs, BLAKE3 idem — trascurabili. Il
  costo vero era l'embedding, che esce dalla TX.
- **Migrazione V016 su vault grandi**: crea solo tabelle vuote — istantanea.
- **Doppia scrittura divergente** (vincolo di correttezza n.1 del design):
  impossibile per costruzione se ogni scrittura passa dal choke-point; la
  guardia è il grep CI del criterio 1 + il property test 1.
- **Worker dirty che resta indietro**: la ricerca FTS può essere stale di
  qualche secondo dopo un burst di import — accettato e documentato (oggi è
  sincrona ma al costo di un rebuild totale per salvataggio).
- **`keyring` su piattaforme reali**: il fallback è specificato sopra; i
  test CI usano sempre il fallback file (niente Secret Service sui runner).

## Rimandi espliciti

- Cifratura dei segmenti, master seed, recovery phrase, viste firmate,
  heartbeat, snapshot a quorum, GC causale multi-peer → **blueprint F2**.
- Variante-da-conflitto derivata e Vista conflitti → **F3** (la proiezione
  qui prepara solo `OrditoApplied`, che è il prerequisito).
- Protocollo oplog-nativo del server → **F4**.
