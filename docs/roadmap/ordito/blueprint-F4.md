# Blueprint operativo — Ordito F4 "Enterprise: peer autoritativo e proiezione database-agnostic"

> **Stato**: design operativo **sotto gate domanda-driven**. Come da
> [`../v2.0-enterprise.md`](../v2.0-enterprise.md): nessun codice di
> produzione finché non esiste un cliente reale con richiesta scritta.
> Questo blueprint è ammesso dallo "stream design parallelo" di quel doc.
> **Prerequisito**: [`../ordito-sync-log.md`](../ordito-sync-log.md) v3 +
> F1-F3 atterrate. Si integra con gli step di
> [`../fase-5-enterprise.md`](../fase-5-enterprise.md) (0a single binary,
> 2 SSO, 5 E2E): Ordito F4 ne è lo strato dati/sync.
> **Obiettivo F4**: `papsync` evoluto in **peer autoritativo** del log
> (staging+ACK, catena workspace ri-firmata, certificati device→utente,
> RBAC/approval come gate di accettazione) e **proiezione database-agnostic**
> (SQLite e Postgres/pgvector dietro repository, equivalenza garantita da
> una conformance suite cross-language).

## Chiarimento architetturale (raffina la v3, non la cambia)

Il design v3 dice "repository trait + proiezione Postgres". Operativamente
la proiezione enterprise vive nel **server Go** (è lui che consuma il log e
serve i team), non nel client Rust:

- **Client Rust**: resta SQLite/SQLCipher. I trait per aggregato
  (`PromptRepo`, `SearchIndex`, `EmbeddingStore`, `OplogStore`) si
  estraggono SOLO dove il client ne trae beneficio reale (testabilità del
  choke-point, mock nei property test) — non come rito: YAGNI.
- **Server Go**: repository pattern con **due implementazioni** dietro la
  stessa interfaccia — `modernc.org/sqlite` (default zero-dipendenze,
  coerente con Fase 5 Step 0a "single binary senza Docker") e
  **Postgres/pgx + pgvector** per on-prem/cloud enterprise. È qui che si
  incassa il "database-agnostic" promesso dal blueprint.
- **Equivalenza cross-language**: la stessa sequenza di record deve
  proiettare lo stesso stato in Rust/SQLite, Go/SQLite e Go/Postgres.
  Garanzia: **conformance suite** su fixture di log condivise (file CBOR
  golden in `packages/ordito-fixtures/`), dump normalizzato e confronto in
  CI — stesso metodo già validato nello spike Argon2id Go==Rust
  byte-identico (2026-07-04).

## Lato client — staging e proposta (design v3 §Due modelli di fiducia)

Migrazione V019-F4: tabella `OrditoStaging`:

```sql
CREATE TABLE OrditoStaging (
    Seq        INTEGER PRIMARY KEY AUTOINCREMENT,
    Record     BLOB NOT NULL,          -- OrditoRecord CBOR (firmato dal device)
    Stato      TEXT NOT NULL CHECK (Stato IN ('locale','proposto','accettato','respinto')),
    MotivoKo   TEXT,                   -- dal server, se respinto
    PropostoA  TEXT
);
```

- In un **workspace team** le mutazioni locali NON appendono alla catena
  sincronizzata: `apply_change` scrive proiezione locale + `OrditoStaging`
  (stato `locale`). La UI mostra subito la modifica con badge "in attesa
  di sincronizzazione".
- Alla connessione: batch `proposto` → `POST /ordito/propose`. Risposta
  per-record: `accettato` (con posizione in catena workspace) o `respinto`
  (motivo: RBAC, approval richiesto, validazione).
- **Reject senza buchi**: il record respinto passa a `respinto`, il
  contenuto viene offerto come **bozza di recupero** (riusa il pattern
  della Vista conflitti F3); la proiezione locale fa rollback al valore
  della catena workspace. Nessuna catena per-install viene bucata: in team
  mode la catena autoritativa è quella del workspace (v3), e le proposte
  sono fuori catena finché non accettate.
- **Gossip vietato**: in workspace team i trasporti T1/T2/T3 sono
  disabilitati per quel vault (enforcement UI + guard nel codice), come da
  topologia a stella rigida.

## Lato server — catena workspace e pipeline di accettazione

Schema (nuove tabelle server, migrazione `002_ordito.sql`):

```sql
CREATE TABLE WorkspaceChain (
    Seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    WorkspaceId TEXT NOT NULL,
    Hash        TEXT NOT NULL UNIQUE,   -- BLAKE3 del record ri-firmato
    Prev        TEXT,                   -- catena PER workspace
    Record      BLOB NOT NULL,          -- record originale del client (firma device preservata)
    ServerSig   TEXT NOT NULL,          -- ri-firma del server sull'accettazione
    ProposedBy  TEXT NOT NULL,          -- install del proponente
    UserId      TEXT NOT NULL,          -- dal certificato device→utente
    AcceptedAt  TEXT NOT NULL
);
CREATE TABLE DeviceCerts (
    Install    TEXT PRIMARY KEY,
    UserId     TEXT NOT NULL REFERENCES Users(Id),
    PubkeyEd   TEXT NOT NULL,
    Cert       BLOB NOT NULL,           -- firma server su (install, pubkey, userId, ruolo)
    RevokedAt  TEXT
);
```

Endpoint nuovi (accanto ai legacy `/sync/*` durante la transizione):

- `POST /ordito/enroll` — pairing device: emette il **certificato
  device→utente** (v3 §Sicurezza) dopo auth JWT/OIDC.
- `POST /ordito/propose` — batch di record; risposta per-record.
- `GET  /ordito/chain?workspace=&since=` — serve la catena workspace
  (paginata); il client la applica con `applica_remoto` (stesso consumer
  di F2/F3 — verifica `ServerSig` + firma device originale).
- `WS   /ws` — notifica `ordito_update` (riusa l'hub esistente).

**Pipeline di accettazione** (ordine fisso, ogni passo può respingere):

1. authn: JWT/OIDC valido (SSO = Fase 5 Step 2, si aggancia qui);
2. certificato device valido e non revocato; firma record verificata;
   `UserId` derivato dal certificato — MAI dal payload (chiude davvero la
   classe #450: la paternità utente è `DeviceCerts.UserId`, non un campo
   del body);
3. autorizzazione: ruolo workspace + RBAC cartelle (`FolderPermissions`,
   Fase 5 Step "approval+RBAC") valutata sull'entità target;
4. approval workflow: se la cartella richiede review, il record entra in
   stato `pending_review` (accettazione differita: il proponente vede
   "in approvazione"; l'approvazione appende alla catena);
5. validazione dati (invarianti cross-campo, come la proiezione client);
6. append a `WorkspaceChain` (ri-firma) + proiezione + broadcast WS.

**E2E interplay** (Fase 5 Step 5): in workspace E2E il server valida solo
metadata (firma, cert, RBAC su cartella/id) — i passi 4-5 sui contenuti non
sono possibili su payload cifrati; il blob embeddings arriva dai client
(Decisione 2, obbligata qui). Documentare il trade-off nel contratto.

## Proiezione Go database-agnostic

```go
type ProiezioneRepo interface {   // per aggregato: Prompt, Tag, Folder, ...
    Apply(ctx, rec OrditoRecord) error   // upsert idempotente con LWW-guard
    Get/List/...                         // letture per l'API pubblica (Fase 5 Step 4)
}
type SearchIndex interface{ Reindex(id string) ... }   // FTS5 | Postgres tsvector
type VectorStore interface{ Upsert/KNN ... }           // no-op sqlite | pgvector
```

- Implementazione 1: `modernc.org/sqlite` (già in uso su CLI; il server
  migra da `mattn` come previsto da Fase 5 Step 0a — un lavoro solo).
- Implementazione 2: `pgx` + Postgres FTS (`tsvector`) + `pgvector`.
- `OrditoApplied` equivalente lato server (stessa semantica per-campo);
  upcaster registry Go verificato contro le fixture condivise.
- La **conformance suite** gira in CI su entrambe le implementazioni per
  ogni fixture: è il gate che rende onesto il claim "database-agnostic".

## Transizione dal sync legacy

- Il protocollo LWW full-row (`/sync/pull|push`) resta funzionante durante
  F4 e viene ritirato solo quando tutti i client di un workspace risultano
  ≥ versione Ordito (il server lo vede dai certificati emessi).
- Migrazione dati server: i workspace esistenti ottengono una catena
  inizializzata da uno **snapshot autoritativo** dello stato corrente
  (server-firmato, con attestazioni "migrated-from-legacy").
- `SyncChangelog` legacy resta come storico; le nuove scritture vivono in
  `WorkspaceChain`.

## Sequenza degli step (contingente al gate cliente)

| Step | Contenuto | Dipende da | Stima |
|---|---|---|---|
| F4-S1 | `packages/ordito-fixtures/` + conformance runner (Rust già verde su F1-F3; Go da zero su SQLite modernc) | F3 | media |
| F4-S2 | server: `WorkspaceChain` + `DeviceCerts` + enroll/propose/chain + pipeline 1-2-6 (senza RBAC/approval) | S1 | alta |
| F4-S3 | client: `OrditoStaging` (V019) + flusso propose/reject/bozza + disabilitazione T1-T3 sui vault team | S2 | alta |
| F4-S4 | RBAC cartelle + approval workflow nella pipeline (passi 3-4) — congiunto con gli step Fase 5 | S2 | alta |
| F4-S5 | repository Postgres/pgx + pgvector + conformance verde su entrambe | S1 | alta |
| F4-S6 | transizione legacy: snapshot autoritativo di migrazione + ritiro `/sync/*` gated | S3 | media |
| F4-S7 | hardening: chiude il backlog sync-server (#450-455, #462) sul nuovo protocollo; pen-test come Fase 5 Step 9 | S2-S5 | media |

## Criteri di completamento F4

- [ ] Conformance suite: ogni fixture proietta identico su Rust/SQLite,
      Go/SQLite, Go/Postgres (dump normalizzato, CI).
- [ ] Un reject RBAC non buca mai nulla: il client recupera la bozza, la
      catena workspace resta verificabile end-to-end da un client terzo.
- [ ] Paternità: un client malevolo non può attribuire record a un altro
      utente (il `UserId` viene solo da `DeviceCerts`) — test di attacco
      esplicito che rigioca lo scenario di #450.
- [ ] Approval: un record su cartella gated resta invisibile agli altri
      client fino all'approvazione; l'approvazione converge.
- [ ] Server single binary senza Docker (Step 0a) con SQLite; lo stesso
      binary con `--db postgres://` passa l'intera suite d'integrazione.
- [ ] I workspace legacy migrano senza perdita (snapshot autoritativo);
      i client vecchi continuano su `/sync/*` finché non aggiornano.
- [ ] Coverage server ≥80% sui package nuovi (`ordito`, `chain`, `certs`).

## Rischi operativi

- **Doppio protocollo temporaneo** (`/sync/*` + `/ordito/*`): confinato
  dalla regola di ritiro sopra; niente feature nuove sul legacy.
- **Divergenza upcaster Rust/Go**: la conformance suite è l'unico
  guardrail credibile — per questo è lo Step 1, non un'appendice.
- **Approval + offline**: un client può accumulare proposte su cartelle
  gated; lo staging le tiene ordinate ma l'UX "metà accettate, metà in
  review" va disegnata con cura (mock UI prima di S4).
- **Postgres FTS ≠ FTS5** (ranking/tokenizzazione diversi): accettato — i
  risultati di *ricerca* possono differire tra backend; la conformance
  copre lo STATO proiettato, non il ranking (dichiarato nel contratto).
- **Gate**: tutto questo blueprint resta carta finché non c'è il cliente
  (le 3 domande di `v2.0-enterprise.md`). L'unica eccezione ammessa è
  F4-S1 (fixtures + runner Rust), che serve comunque da regression harness
  per F2/F3.
