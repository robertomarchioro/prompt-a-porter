# Blueprint — "Ordito": log replicato per sync senza server e storage database-agnostic

> **Stato**: design / ideazione. Nessun codice ancora.
> **Revisione**: v3 (2026-07-07) — i 7 punti aperti della v2 sono stati decisi
> (v. §Decisioni chiuse in coda) e le conseguenze riportate nelle sezioni.
> La v2 (2026-07-06) era la riscrittura post review avversariale a 3 lenti
> (correttezza distribuita, security, prodotto): merge come **funzioni pure
> del log** e compattazione a **snapshot con commitment separato**.
> **Obiettivo utente**: (1) sincronizzare i propri vault tra più dispositivi in
> modo intelligente **senza server centrale obbligatorio**; (2) rendere lo
> strato dati **database-agnostic** per la versione Enterprise (on-prem e
> cloud), con un unico protocollo di comunicazione.
> **Scope**: fasi F1-F3 = SKU Personale (multi-device, stesso utente);
> fase F4 = primo mattone dello SKU Enterprise v2.x.
> **Nome**: *Ordito* — il filo portante del telaio su cui ogni dispositivo e
> ogni backend tesse la propria trama. È il nome del **motore** (docs, sito,
> release notes); in UI la feature si chiama sempre **"Sincronizzazione
> dispositivi"** (v. Decisione 7).

## Perché (contesto e problemi del sync attuale)

Lo stato dell'arte nel repo (2026-07-06):

- Il sync attuale è **client-server** (`papsync`, Go): pull/push delta con
  **last-write-wins su `UpdatedAt` wall-clock** (`apps/server/internal/sync/handler.go`).
  Due macchine con orologi sfasati possono perdere scritture in silenzio.
- Il client desktop è di fatto **pull-only**: nessun codice chiama
  `/sync/push`; la UI conflitti (`ConflittoSync.svelte`) è stata rimossa nel
  redesign v0.8 (blueprint F9/F11).
- Lo storage è SQLite-first in modo pervasivo: ~590 punti di contatto SQL
  grezzo in 25 file Rust, con tre lock-in *strutturali*: **SQLCipher**
  (cifratura file), **FTS5** (full-text), **sqlite-vec** (semantica).
- `prompts-as-code.md` cita già il bet "CRDT local-first multi-device", mai
  scelto. `vault-a-cartella.md` dichiara vault a cartella e sync
  **mutuamente esclusivi in via permanente** (Decisione 5): la cartella di
  trasporto T1 è dedicata, NON è il vault a cartella.
- Lo schema client ha già mezzi ingredienti: tombstone `DeletedAt`, contatore
  `Version`, `AuditLog` e `SyncChangelog` append-only.

## Principio architetturale

> Non si sincronizza **lo stato del database**: si sincronizza un **log
> append-only di cambiamenti** (oplog), cifrato e firmato per dispositivo.
> Ogni database — SQLite locale, Postgres enterprise, export Markdown/Git —
> è una **proiezione ricostruibile** del log. I peer sono intelligenti,
> il canale di trasporto è stupido e intercambiabile.

Conseguenze:

1. **Sync senza server**: due dispositivi che si scambiano i record mancanti
   del log convergono allo stesso stato, su qualunque canale (LAN, cartella
   condivisa, relay muto).
2. **Database-agnostic**: FTS5 e sqlite-vec smettono di essere un problema di
   portabilità perché sono **indici derivati** — ogni backend usa il suo
   equivalente nativo (Postgres FTS + pgvector). FTS si rigenera dal testo;
   gli **embeddings viaggiano nello snapshot** come derivato opaco versionato
   per modello (Decisione 2): il device nuovo ha la ricerca semantica subito
   e ricalcola solo i prompt modificati dopo il taglio (o tutto, se usa un
   modello diverso).
3. **Audit trail a livello dispositivo**: ogni record è firmato dal device
   che l'ha prodotto. Nel caso personale la paternità è crittografica a
   livello device; nel caso team la paternità *utente* resta asserita dal
   peer autoritativo tramite un **certificato device→utente** firmato dal
   server al pairing (v. §Sicurezza). Questo *irrigidisce* la classe di
   problemi di #450, non la "risolve alla radice": la firma prova il device,
   non l'utente.
4. **Time-travel e backup incrementale**: il log *è* la storia; l'export Git
   di `prompts-as-code.md` diventa l'ennesima proiezione. Nei vault senza
   sync attivo la storia è limitata dalla retention locale (Decisione 1).

Ispirazioni (tutte a concetti aperti / implementazioni permissive):
Git (content-addressing, hash chain), CouchDB (checkpoint di replica per
coppia di peer), CockroachDB (Hybrid Logical Clock, dal paper Kulkarni et
al.), Nostr/negentropy e ATProto/Merkle Search Trees (riconciliazione
range-based di insiemi), Syncthing (identità dispositivo = chiave pubblica,
pairing), Ink & Switch "local-first" (server come peer non privilegiato).

## Il formato del record (`OrditoRecord`)

Un record per mutazione applicata, serializzato **CBOR** (compatto, binario,
schema-less; JSON per debug/export). Campi:

```
OrditoRecord {
  v:        1                          // versione formato record
  schema:   15                         // versione schema dati (= migrazione client V015)
  vault:    "vlt-..."                  // id del vault: lega il record al suo vault (anti replay cross-vault)
  op:       "upsert" | "delete"        // delete = tombstone, mai hard-delete nel log
  entity:   "prompt" | "tag" | "folder" | "prompt_tag" | "version"
            | "rating" | "golden" | "global" | ...   // v. tabella copertura
  entityId: "prm-..."                  // ULID/UUIDv7; per entità a chiave composita: id canonico
                                       // "{PromptId}|{TagId}" (v. §Set e chiavi composite)
  fields:   { "Title": "...", "Body": "..." }   // SOLO i campi cambiati (field-level)
  hlc:      "..."                      // Hybrid Logical Clock (v. sotto)
  device:   "dev_a1b2.7f3e"            // fingerprint pubkey Ed25519 + install_nonce (v. sotto)
  prev:     "b3-..."                   // hash BLAKE3 del record precedente DELLA STESSA install
  hash:     "b3-..."                   // BLAKE3 del record (esclusi hash e sig)
  sig:      "ed25519-..."              // firma del device su hash
}
```

Regole:

- **Hash chain per install** (`prev`): ogni install produce una catena
  lineare verificabile e **immutabile** (mai riscritta, mai squashata —
  v. §Compattazione). Manomissioni o buchi sono rilevabili.
- **Identità device = fingerprint(pubkey) + install_nonce**: il nonce viene
  rigenerato a ogni install/restore. Motivo: il restore di un backup che
  include la chiave privata (o il clone di una VM) produrrebbe **due
  scrittori sulla stessa catena** → fork non rilevabile (equivocation).
  Col nonce, un restore genera una *nuova* identità di catena; in più ogni
  peer che osserva **due record con lo stesso `prev` e stesso device**
  segnala l'equivocation e mette in quarantena il ramo con HLC minore.
- **`vault` nel payload firmato**: un record non è replayabile su un altro
  vault/topic; verificato all'apply.
- **Field-level, non row-level**: `fields` contiene solo i campi modificati.
  Vincolo: il **primo** record di un `entityId` è sempre un *create completo*
  (tutti i campi NOT NULL presenti); un field-update che arriva prima del
  create viene bufferizzato in `OrditoPending` (la proiezione non può fare
  INSERT parziale su colonne NOT NULL).
- **`delete` = tombstone row-level** con semantica definita (v. §Delete).
- **`schema`**: i record dichiarano la versione di schema con cui sono stati
  scritti — per sempre (v. §Evoluzione dello schema).

### HLC — Hybrid Logical Clock

Sostituisce `UpdatedAt` wall-clock come criterio di ordinamento. Tripla
`(physical_time_ms, logical_counter, install_id)`:

- a ogni evento locale: `physical = max(now(), physical_ultimo)`; se
  `physical == physical_ultimo` incrementa `logical`, altrimenti `logical = 0`;
- **a ogni ingest di record remoti, su QUALUNQUE trasporto** — incluso lo
  scan della cartella T1, che non ha una "ricezione" di rete: prima
  dell'apply, `physical = max(now(), physical_locale, physical_remoto)` e
  `logical` di conseguenza. È un **invariante del layer di applicazione**,
  non del trasporto: senza, un device che assorbe record da file può poi
  produrre HLC inferiori a ciò che ha già visto (violazione di causalità);
- confronto totale: `(physical, logical, install_id)` lessicografico —
  l'`install_id` rompe i pareggi in modo deterministico su tutti i peer.

Encoding: stringa ordinabile lessicograficamente — 48 bit ms hex + **32 bit
counter** hex + install id. Il counter a 32 bit assorbe i burst da import
massivo/backfill (>65k eventi/ms romperebbero un counter a 16 bit); in
alternativa i comandi bulk assegnano HLC monotoni a blocco.

Guardia anti-deriva: se `now()` locale è più avanti di oltre 10 minuti
rispetto al massimo `physical` visto dalla rete di peer, warning all'utente
("orologio del dispositivo sospetto") — mitiga il device con clock nel futuro
che "vince" ogni LWW.

## Semantica di merge

### Invariante fondante: il merge è una funzione pura del log

> Ogni regola di risoluzione (LWW, variante da conflitto, fusione tag
> duplicati, rottura cicli cartelle, numerazione versioni) è una **funzione
> deterministica dell'insieme dei record**, calcolata identicamente da ogni
> peer **alla proiezione**. Il merge **non emette mai nuovi record**: ogni
> id, label o timestamp che il merge "crea" è **derivato** dai record in
> input (hash, HLC), mai da `now()` o da un generatore locale.

Motivo (finding CRITICAL della review): se il merge creasse entità come
effetto collaterale dell'apply, ogni peer genererebbe id diversi per la
stessa risoluzione → le risoluzioni si ripropagherebbero come nuove mutazioni
→ divergenza e amplificazione combinatoria. Con merge puri, log identico ⇒
proiezione identica, sempre.

### Regola generale: LWW-map per campo

Per ogni `(entity, entityId, campo)` vince il record con **HLC più alto**.
La proiezione tiene, per riga, l'HLC dell'ultimo write applicato per campo
(tabella ausiliaria `OrditoApplied(entityId, field, hlc)` — il guard è
**per-campo**, mai per-riga) e scarta i record più vecchi. Applicare due
volte lo stesso record, o record in ordine diverso, produce lo stesso stato.

Esempio del guadagno rispetto all'LWW attuale: il portatile cambia il titolo,
il fisso (offline) cambia i tag dello stesso prompt → **entrambe** le
modifiche sopravvivono. Oggi una delle due verrebbe scartata intera.

**Onestà sul termine "CRDT"**: solo questo strato field-level è una LWW-map
CRDT (commutativa/idempotente). Sopra c'è uno strato di risoluzione
domain-native (variante, tag-merge, anti-ciclo) che NON è un CRDT: è reso
convergente dall'invariante di purezza qui sopra, e va verificato con
property test (v. §Strategia di verifica). Inoltre il field-level LWW può
comporre combinazioni di campi mai esistite su un singolo device: le
invarianti cross-campo (es. `Visibility` vs cartella) vanno validate alla
proiezione con regole deterministiche di riparazione.

### Delete, tombstone e resurrezione (semantica esplicita)

- `op:"delete"` scrive un **tombstone row-level** registrato in
  `OrditoApplied` come pseudo-campo `__tombstone__` col suo HLC.
- Il tombstone **domina ogni campo con HLC ≤ HLC_delete**: un update
  concorrente più vecchio del delete perde su tutta la riga.
- Un upsert con **HLC > HLC_delete resuscita** la riga (comportamento
  scelto: chi modifica *dopo* la cancellazione sta esprimendo l'intento più
  recente). La resurrezione ripristina i campi al valore LWW corrente.
  Non esiste "undelete implicito" da merge parziale: o l'upsert batte il
  tombstone su HLC, o la riga resta cancellata.
- Il **purge dal cestino** è locale alla proiezione; il tombstone resta nel
  log finché la GC causale non lo prova consegnato ovunque (v. §Compattazione).

### Conflitto vero sul `Body`: il perdente diventa una variante — derivata

Caso: due device offline modificano **lo stesso campo `Body`** dello stesso
prompt. LWW da solo butterebbe via un lavoro potenzialmente prezioso.

Regola (versione pura, corretta rispetto alla v1 del blueprint):

1. vince l'HLC più alto → il `Body` "ufficiale" è deterministico e identico
   su tutti i peer;
2. il `Body` perdente viene materializzato come **variante A/B** del prompt
   (feature V011) — ma come **riga derivata della proiezione**, non come
   record nuovo: `Id = "prm-" + BLAKE3(hash_record_perdente)` (identico su
   ogni peer), `VariantLabel = "sync-" + HLC_perdente` (derivata dal record,
   mai da `now()` locale). Ogni peer che proietta lo stesso log produce la
   stessa variante, senza emettere nulla;
3. la variante derivata è a tutti gli effetti un prompt: se l'utente la
   modifica o la promuove, *quelle* azioni sì producono record normali;
   se la ignora, resta una proiezione stabile del conflitto;
4. l'utente la vede nella UI varianti già esistente — **si riusa la UI di
   promozione varianti al posto di una UI conflitti dedicata**;
5. opzionale (Deluxe): se il prompt ha golden test, l'app li esegue su
   entrambe le versioni e suggerisce la vincente.

**Criterio 3-way** (quando creare la variante): si crea solo se il `Body`
perdente differisce sia dal vincente sia dall'**antenato comune**, definito
sul log unito come l'ultimo valore di `Body` (per HLC) visibile da *entrambe*
le catene prima della divergenza. Fallback obbligatorio: **antenato non
determinabile ⇒ si tratta come conflitto reale** (variante creata) — mai
scarto silenzioso di un Body divergente. I record che fungono da antenato a
divergenze aperte sono protetti dalla compattazione.

**Igiene dei conflitti** (accumulo silenzioso): la **Vista conflitti** aggrega
"N varianti da conflitto in attesa" a livello vault (non solo badge
per-prompt); le varianti derivate identiche al Body ufficiale decadono
automaticamente; sopra una soglia configurabile l'app propone una revisione
guidata. La stessa vista ospita la **revisione al rientro di un'install
stale** (Decisione 6).

### Set e chiavi composite (`PromptTags` e simili)

Il formato ha un solo `entityId`, ma `PromptTags` ha PK composita. Regola:

- `entityId` canonico composito: `"{PromptId}|{TagId}"`.
- Semantica **LWW-element-set per membro**: `op:"upsert"` = membro presente,
  `op:"delete"` = tombstone del membro; vince l'HLC più alto per membro.
  (Scartato l'add-wins puro della v1: rendeva impossibile togliere un tag
  toccato concorrentemente — tag "fantasma".)
- Nel client attuale `sincronizza_tags` fa DELETE-all + re-INSERT
  (`editor.rs`): per il log va convertito in **diff add/remove espliciti**
  per membro (prerequisito F1).

### Tag duplicati e cicli di cartelle: risoluzione alla proiezione

- **Due device creano tag con lo stesso nome** (`UNIQUE(WorkspaceId,Name)`):
  nessuna "riassegnazione" emessa. La proiezione applica una **fusione
  derivata**: l'id canonico del nome è quello col HLC di creazione più alto;
  le membership dell'id perdente sono *lette come* membership del canonico
  (mapping deterministico in proiezione). Il tag perdente resta nel log;
  nessun record nuovo.
- **Ciclo di cartelle da merge concorrente** (A→B→C→A): funzione pura — si
  **ignora in proiezione** l'edge con HLC *minimo* del ciclo (il più vecchio
  perde, coerente con LWW), che viene proiettato come figlio di root.
  Deterministico anche su cicli ≥3 e su cicli multipli (si itera in ordine
  HLC). Collisione di nome in root (vincolo `UNIQUE` di V004): suffisso
  deterministico derivato dall'id cartella.

### Versioni (`PromptVersions`): identità = Id, numero = etichetta

La v1 del blueprint prevedeva la "rinumerazione deterministica" in caso di
collisione di `Version` — contraddiceva l'immutabilità e rompeva i
riferimenti per numero (rollback UI, trend delle osservazioni,
`{{version=N}}`). Regola corretta:

- l'**identità** di una versione è il suo `Id` (immutabile, append-only);
- il **numero** è un'etichetta di visualizzazione assegnata alla proiezione
  in ordine HLC; due version-record concorrenti con lo stesso numero
  producono numeri di *display* distinti senza mutare i record;
- il vincolo `UNIQUE(PromptId, Version)` non vive più nella tabella
  proiettata come vincolo di verità (il numero è calcolato): la proiezione
  non può fallire per collisione;
- i riferimenti interni (`{{version=N}}`, rollback) risolvono il numero via
  proiezione locale e vengono **pinnati all'Id** al salvataggio: il numero è
  UX, l'Id è verità. (Nota di migrazione: gli import `{{version=N}}`
  esistenti restano risolti per numero-di-display; documentare l'ambiguità
  residua in vault multi-device.)

### Entità append-only: merge banale

`PromptRatings`, `AuditLog`, `PromptRunObservations` sono append-only per
design → il merge è l'unione degli insiemi (dedup per `Id`). Nessun
conflitto possibile. Sono i cittadini ideali del log.

### Copertura entità

| Entità | Nel log? | Merge | Note |
|---|---|---|---|
| Prompts | ✅ | LWW per campo + conflitto-Body→variante derivata | cuore del sistema |
| PromptVersions | ✅ | append-only (immutabili) | identità = Id; numero = etichetta di proiezione (v. §Versioni) |
| Tags | ✅ | LWW per campo | nome duplicato → fusione derivata alla proiezione |
| PromptTags | ✅ | LWW-element-set per membro | entityId composito `"{PromptId}\|{TagId}"` |
| Folders | ✅ | LWW per campo | cicli: edge con HLC minimo ignorato in proiezione (funzione pura) |
| PromptRatings | ✅ | unione (append-only) | |
| PromptGoldens / RunObservations | ✅ / ✅ | LWW / unione | le run sono osservazioni storiche |
| GlobalPlaceholders | ✅ | LWW per riga | |
| PromptImports | ❌ derivata | ricostruita dal parsing del Body alla proiezione | la logica di parsing è **versionata con `schema`**: proiezioni con parser diversi possono divergere temporaneamente, riallineate all'upgrade |
| AuditLog | ✅ (locale→log) | unione | audit firmato a livello device (v. claim §Principio p.3) |
| ProviderConfig | ❌ **mai** | — | contiene API key: restano locali al device |
| SyncMeta / preferenze UI | ❌ | — | stato locale/effimero |
| UseCount / LastUsedAt | ❌ in F1 | G-counter per-install in F3 | incrementi keyed-by-id (idempotenti alla ri-consegna); lo snapshot ripiega i sub-counter per install — progettato ora per non incastrarsi con la compattazione |
| Users / Workspaces | ✅ (solo team) | server-authoritative | nel P2P personale c'è un solo utente |

## Riconciliazione tra peer

Obiettivo: due peer scoprono *cosa manca all'altro* senza trasferire l'intero
log. Due meccanismi complementari:

1. **Checkpoint per coppia di peer** (alla CouchDB): per ogni install remota
   nota si salva l'ultimo `(install, seq)` ricevuto per catena. Nel caso
   comune basta chiedere "dammi tutto dopo questi cursori" — un vettore di
   versione, O(1) round-trip.
2. **Riconciliazione range-based** (alla negentropy) come fallback robusto:
   fingerprint di range dell'insieme dei record ordinati per HLC, divisione
   ricorsiva dei range che differiscono. Costo logaritmico, nessuno stato
   per-peer. Implementazioni MIT esistenti in Rust e Go.
   Nota: su T1/T3 i fingerprint si calcolano sui record **decifrati** →
   richiede il keyring VSK completo (v. §Sicurezza).

L'applicazione è **idempotente** (LWW-map per campo), quindi ricevere due
volte lo stesso record è innocuo. Attenzione però all'interazione con la
compattazione: la ri-consegna "in eccesso" di record già coperti da snapshot
è gestita dalla GC causale dei tombstone (v. §Compattazione) — è quella, non
l'idempotenza, a impedire le resurrezioni.

## Trasporti (il canale è stupido)

### T1 — Cartella condivisa (`.ordito/`) — costo minimo, valore immediato

Ogni device appende **segmenti di log** come file immutabili in una cartella
**di trasporto dedicata** (Syncthing, Dropbox, NAS, chiavetta USB):

```
<cartella-trasporto>/.ordito/
├─ dev_a1b2.7f3e/
│  ├─ 000001.seg                  # segmento: batch di record CBOR, cifrato (epoca VSK nel header), ~1-4 MB
│  ├─ 000002.seg
│  ├─ head.json                   # ultimo seq + hash, firmato
│  ├─ view.json                   # vista firmata: device noti, revoche, cursor heartbeat (v. sotto)
│  └─ keys/                       # envelope di rekey cifrati per-device (v. §Sicurezza)
└─ dev_c3d4.a911/
   └─ ...
```

- **Il log autoritativo vive DENTRO il vault** (tabelle `Oplog`/`OrditoApplied`/…
  nel `.db` SQLCipher): i segmenti nella cartella di trasporto sono una
  **replica rigenerabile** (Decisione 3). Cancellare la cartella non perde
  nulla — si ri-pubblica dal log locale o si ri-fetcha dai peer. Il modello
  mentale "il vault è un file, copialo e hai tutto" resta vero, log incluso.
- **Nessun file mutabile condiviso.** La v1 prevedeva un `manifest.json` in
  radice: contraddiceva il design (scrittura condivisa → conflicted copy,
  bersaglio di manomissione). Ora ogni informazione di controllo è nella
  `view.json` **firmata, per-device, nella propria sottocartella**; la vista
  effettiva (device fidati, revoche) è il merge deterministico delle viste
  firmate. Un eventuale `manifest.json` è solo cache derivata locale.
- **Cursor heartbeat**: ogni device pubblica nella propria `view.json` il
  proprio cursore di lettura firmato con timestamp — è ciò che rende
  calcolabile la compattazione su un trasporto senza handshake (v. §Compattazione).
- Ogni device scrive **solo nella propria sottocartella** → zero conflitti di
  file-locking, zero corruzione da SMB/NFS: file append-only immutabili. Le
  "conflicted copy" di Dropbox sono ignorate perché fuori catena hash.
- I segmenti sono **cifrati** (VSK a epoche, v. §Sicurezza): la cartella può
  stare su Dropbox senza esporre i prompt.
- **La cartella di trasporto NON è il vault a cartella.** È una posizione
  qualsiasi che contiene solo `.ordito/`. Il vault a cartella **non sarà mai
  sincronizzato via Ordito** (Decisione 5, esclusione permanente): i `.md`
  replicati dal file-syncer e l'oplog trasporterebbero la stessa modifica su
  due canali (echo, doppio conflict-resolver — Syncthing vs HLC — con esiti
  divergenti). Chi usa il vault a cartella sincronizza con i propri strumenti
  (git, Syncthing) accettando i limiti documentati in `vault-a-cartella.md`.

### T2 — LAN diretta

- Discovery **mDNS/DNS-SD** (`_pap-ordito._tcp`) via crate `mdns-sd`.
- **Pairing**: l'identità del device è la sua chiave pubblica Ed25519 (+
  install_nonce). Accoppiamento con QR code o codice breve a 6 parole
  (verifica out-of-band del fingerprint, stile Syncthing/Signal). L'elenco
  device fidati vive nelle viste firmate e converge come il resto del log.
- **Canale: TCP + Noise XX** via crate `snow` (implementazione consolidata
  del Noise Protocol Framework), mutua autenticazione sulle chiavi device
  (Decisione 4). Scope **solo-LAN**: il caso Internet è coperto in modo
  asincrono da T1 (cartella) e T3 (relay muto). Framing, keep-alive e
  riconnessione sono nostri e vanno testati (v. §Strategia di verifica).
- **Iroh è stato valutato e retrocesso a eventuale T4 futuro** (P2P live
  cross-Internet con hole-punching), da riaprire solo su domanda reale:
  dipendenza pesante, relay di default hostati da n0 (in contrasto col claim
  "senza server": andrebbero self-hostati o disabilitati), superficie della
  review crittografica molto più larga — e comunque non copre T3 (i relay
  Iroh instradano traffico live tra peer online insieme, non fanno
  store-and-forward per device mai accesi insieme).

### T3 — Relay "muto" (store-and-forward, opzionale)

Per device mai accesi insieme e senza cartella condivisa: un servizio che
accetta e serve **blob cifrati opachi** per topic, senza poterli leggere.
Chiunque può hostarlo; un futuro "PaP Cloud" può offrirlo **senza diventare
trusted**. API: 3 endpoint (`PUT /topic/{id}/seg`,
`GET /topic/{id}/since/{cursor}`, `GET /health`).

**Modello di minaccia metadata (T1/T3)**: anche a contenuto cifrato, l'host
del trasporto vede numero di device, dimensioni, orari di attività, e su T3
un topic-id stabile correlabile. Mitigazioni previste: padding dei segmenti
a taglie quantizzate, topic-id ruotabile derivato da `KDF(VSK_epoch)`,
`view.json` cifrata come i segmenti. Va dichiarato nella doc utente: "chi
ospita il trasporto vede *che* lavori e *quanto*, mai *cosa*".

## Sicurezza

Ricicla il design crittografico di Fase 5 Step 5 (E2E), con una
semplificazione (caso personale = nessuno scambio multi-utente) e con il
key-management corretto dopo la review.

### Identità e storage delle chiavi

- **Device key**: coppia Ed25519 per install, privata nel keychain OS.
  ⚠️ Realismo: il crate `keyring` **non è ancora una dipendenza** (il
  `sync_token` è tuttora in chiaro con un TODO in `preferenze.rs`), e il
  keychain è il punto fragile della piattaforma (incidenti DPAPI #467/#468 su
  Windows; Linux headless spesso senza Secret Service). Prerequisito F1:
  **spec di key-storage con fallback** — file cifrato con passphrase
  Argon2id quando il keychain non è disponibile o non affidabile; perdita
  della chiave = ri-pairing dell'install (mai perdita di dati: i record già
  pubblicati restano validi).
- **Certificato device→utente (solo team, F4)**: al pairing il server firma
  l'associazione `(device pubkey, userId, ruolo)`; i peer possono verificare
  la paternità utente end-to-end invece di fidarsi di un campo del body.

### VSK a epoche, derivate da un seed stabile

- **Master seed** per vault, generato alla prima attivazione del sync;
  la **recovery phrase** codifica il master seed — e quindi resta valida
  attraverso tutte le rotazioni ordinarie (v1 aveva il difetto: frase
  stampata al giorno 0, morta alla prima rotazione).
- **VSK per epoca**: `VSK_e = KDF(master_seed, e)`. Ogni segmento dichiara
  nel header l'epoca con cui è cifrato. Ruotare = incrementare l'epoca.
- **Distribuzione**: al pairing il nuovo device riceve il master seed
  (envelope X25519 verso la sua chiave) → può derivare **tutte** le epoche e
  leggere l'intera storia (la v1 distribuiva "la VSK" al singolare: un
  device accoppiato dopo una rotazione non avrebbe letto i segmenti storici).
- **Rekey su revoca**: la revoca invalida il master seed noto al revocato →
  **nuovo master seed**, distribuito ai superstiti come **envelope per-device
  cifrati X25519, fuori dalla VSK** (file in `keys/` su T1) — mai dentro
  segmenti cifrati con la chiave che si sta ruotando (la v1 era circolare).
  Gli envelope non vengono mai rimossi finché ogni device fidato non li ha
  consumati (heartbeat). Un superstite offline da mesi risale la catena di
  envelope a suo nome. La frase di recupero va **ri-emessa** dopo un rekey
  da revoca (nuovo master): l'app lo impone con un flusso bloccante.
- **Smaltimento del vault = crypto-shredding** (Decisione 3): "Elimina vault"
  distrugge il master seed (dal vault e, su richiesta, dai device accoppiati)
  → tutti i segmenti sparsi su trasporti terzi (Dropbox, NAS, relay)
  diventano **permanentemente illeggibili**. Il flusso UI lo dichiara e
  offre la pulizia manuale dei blob residui ("per rimuovere anche i file
  cifrati, cancella la cartella X / il topic Y").
- **Cosa la revoca NON fa (limiti dichiarati)**: (a) non protegge il passato
  — il revocato ha già letto ciò che aveva; (b) su T1 **non revoca l'accesso
  al filesystem**: il revocato può ancora cancellare o inquinare la cartella
  condivisa finché non viene rimosso dallo share (azione out-of-band che la
  UI deve richiedere esplicitamente). Difese in-band: i peer ignorano per
  costruzione segmenti firmati da install revocate (HLC di revoca nelle
  viste firmate), rilevano i buchi da cancellazione (catene) e ri-fetchano
  da altri peer; nessun range è considerato "solo su T1" finché non è
  ridondato su ≥2 peer.

## Due modelli di fiducia, un protocollo

| | Personale multi-device (F1-F3) | Team/Enterprise (F4) |
|---|---|---|
| Peer | i device dell'utente, tutti equivalenti | client + **peer con autorità** (server) |
| Trust | totale (stesso proprietario) | RBAC/SSO/approval applicati dal peer autoritativo |
| Topologia | mesh (T1/T2/T3) | **stella rigida**: nessun gossip client-client (v. sotto) |
| Autorizzazione | firma device sufficiente | il server valida ruolo/permessi prima di accettare |
| Catena | catena per-install è l'autorità | **catena workspace del server** è l'autorità |

**Reject senza buchi (fix del finding CRITICAL F4)**: un client team NON
appende le proprie mutazioni direttamente alla catena sincronizzata. Le
mutazioni locali vanno in **staging** (visibili subito nella UI locale,
riconciliabili); alla connessione il client le propone al server; il server
valida (RBAC, approval) e le **appende alla catena workspace ri-firmata**;
il client allinea. Un reject scarta la proposta (con notifica e recupero del
contenuto in bozza) senza mai bucare una catena. Conseguenza da rispettare:
in team mode il gossip diretto client-client è vietato — altrimenti record
mai accettati trapelerebbero ai peer. Il P2P puro resta la topologia del
caso personale.

## Il database come proiezione (obiettivo Enterprise)

### Applicazione del log

La proiezione consuma record in qualunque ordine e converge:

1. upsert idempotente guardato dalla LWW-map **per campo** (+ pseudo-campo
   `__tombstone__`);
2. record orfani (FK verso entità mai vista) → parcheggio in `OrditoPending`;
   la risoluzione del pending consulta lo **stato finale** del target,
   tombstone incluso: si ricollega solo se il target è vivo, altrimenti
   FK NULL/root (niente riattacchi a entità cancellate, niente dangling);
   field-update prima del create → bufferizzato (v. §Formato);
3. **atomicità dell'apply**: `apply record + update OrditoApplied` nella
   stessa transazione. Il refresh degli indici derivati (FTS, embeddings)
   NON può stare in quella transazione (gli embeddings chiamano un
   modello/provider): nella stessa TX si marca un **dirty-flag persistente**
   per entità, consumato fuori-TX da una coda di reindex idempotente con
   retry. L'apply non è "fatto" finché il flag non è consumato — mai
   divergenza silenziosa dell'indice.

**Costo della ricostruzione** (aggiornato con la Decisione 2): FTS si
rigenera dal testo (gratis). Gli **embeddings viaggiano nello snapshot**
come derivato opaco marcato `(modello, versione)`: un device nuovo che parte
dallo snapshot ha la ricerca semantica subito e ricalcola solo i prompt
modificati dopo il taglio. Il ricalcolo completo resta necessario solo se il
device usa un modello diverso da quello dichiarato nel blob (degradazione
pulita al comportamento pre-decisione). Nello scenario E2E enterprise questa
scelta è obbligata: il server non può calcolare embedding su dati che non
legge — sono i client a fornirli via snapshot.

### Evoluzione dello schema: upcaster permanenti

I record `schema=15` restano nel log **per sempre** (append-only; riscriverli
romperebbe le firme). La proiezione deve poterli rigiocare a qualunque schema
futuro. Questo NON è un buffer transitorio (come suggeriva la v1), è
infrastruttura permanente:

- **registry di upcaster versionati** `schema_v → schema_v+1`, funzioni pure
  e testate, mantenute a vita (anche per gli snapshot, che hanno anch'essi
  una versione schema);
- **policy sulle migrazioni**: vietate le migrazioni lossy sui campi loggati
  senza upcaster esplicito che ne documenta la perdita; rimozione/split/cambio
  di tipo di un campo = upcaster obbligatorio;
- **trasporto schema-agnostico**: un peer con app vecchia **inoltra e serve**
  i record che non sa parsare (relay opaco per la riconciliazione — un peer
  vecchio non deve mai diventare un buco nero di record); solo l'*applicazione
  in proiezione* è gated dalla versione ("aggiorna l'app per vedere le
  modifiche nuove").

### Strato repository (l'unico refactor grande)

Per l'Enterprise serve astrarre le ~590 query. Non un ORM: **trait per
aggregato**, implementazione SQLite come prima istanza, Postgres come seconda.

```
trait PromptRepo    { fn upsert(&self, p: &PromptFields, guard: &LwwGuard) -> ...; fn get(...); fn list(...); ... }
trait FolderRepo    { ... }
trait SearchIndex   { fn reindex(&self, id: &str); fn search(&self, q: &str) -> ...; }   // FTS5 | Postgres FTS
trait EmbeddingStore{ fn upsert(&self, id: &str, v: &[f32]); fn knn(...) -> ...; }        // sqlite-vec | pgvector
trait OplogStore    { fn append(...); fn range(...); fn fingerprint(...); }
```

Le funzioni `*_pure(&Connection)` già esistenti sono il punto d'estrazione
naturale (pattern _pure/_impl di M7). Il dialetto SQL si neutralizza dove
serve: `datetime('now')` → timestamp applicativi (già necessari per l'HLC),
indici parziali → equivalenti Postgres, `INSERT OR REPLACE` →
`ON CONFLICT DO UPDATE`.

**Cosa resta volutamente non portabile**: SQLCipher è il formato del *vault
personale* (file portable cifrato = requisito di prodotto), non dello strato
enterprise; on-prem/cloud la cifratura at-rest la dà il backend (TDE/volume)
o resta E2E sul log stesso.

### Il server enterprise diventa un consumer del log

`papsync` evoluto = peer autoritativo che: valida (RBAC, approval workflow),
appende alla catena workspace ri-firmata, proietta su Postgres, notifica via
WS. Gli step Fase 5 (webhook, API pubblica, approval) si costruiscono
**sopra il log** (un webhook è un consumer; l'approval è il gate di
accettazione delle proposte in staging), non sopra un secondo protocollo.
`SyncChangelog` attuale è l'antenato diretto.

## Compattazione del log (ridisegnata)

Vincolo appreso dalla review: **le catene per-install sono immutabili** —
riscrivere o rimuovere record intermedi rende la catena non verificabile
(indistinguibile da manomissione) e cancella le firme originali (con esse la
prova di paternità). Quindi: niente squash in-place. La compattazione è uno
**strato separato di snapshot con commitment**:

- **Snapshot record**: stato completo compresso del vault a un HLC di taglio,
  scritto come segmenti dedicati (`snapshot/`), con:
  - un **vettore di copertura** `[(install, seq, hash_head)]` — fin dove
    copre ogni catena, verificabile da chi possiede ancora i segmenti;
  - per ogni valore vincente, l'**attestazione originale** `(hash, sig,
    install)` del record che l'ha prodotto — la paternità sopravvive al
    taglio (senza questo, uno snapshot potrebbe attribuire contenuti a chi
    non li ha mai scritti, senza possibilità di smentita);
  - una sezione di **derivati opachi** (Decisione 2): i vettori embedding
    correnti marcati `(modello, versione)`, come **cache non attestata** —
    fuori dalle attestazioni di paternità, ricalcolabile e ignorabile da chi
    usa un modello diverso. Un vettore corrotto altera solo i risultati di
    ricerca fino al primo ricalcolo (rischio accettato nel trust model
    personale; nel team mode il server ricalcola o rifiuta i blob);
  - la propria versione `schema` (upcastabile come i record).
- **Controfirma a quorum — K = tutte le install note non-stale** (Decisione
  6): i segmenti sotto il taglio diventano eliminabili solo dopo che ogni
  install attiva ha verificato e controfirmato lo snapshot. Prima di allora
  lo snapshot accelera solo il bootstrap dei nuovi device. (Scartato il
  quorum a maggioranza: con la flotta tipica da 2 device degenera e riapre
  il finding S8.)
- **GC causale dei tombstone** (fix del bug di resurrezione): un tombstone è
  rimovibile solo quando il suo HLC è sotto il **minimo cursore confermato di
  TUTTE le install note** (heartbeat firmati su T1, ACK su T2/T3) — non
  "il peer più arretrato visto di recente" (v1): un peer assente potrebbe
  avere in coda d'uscita upsert vecchi che, senza il tombstone, farebbero
  risorgere il record ovunque.
- **Stale-out automatico — N = 90 giorni** (Decisione 6): un'install assente
  da oltre 90 giorni viene esclusa dal calcolo della GC e marcata **stale**,
  con preavvisi progressivi sugli altri device (che possono sempre
  posticipare). Al rientro, l'install stale fa resync da snapshot e le sue
  modifiche non pubblicate vanno in **revisione guidata** nella Vista
  conflitti — mai scarto silenzioso: l'utente vede "N modifiche di questo
  dispositivo non sono mai state sincronizzate" e sceglie cosa ri-applicare;
  le ri-applicazioni sono record **nuovi con HLC corrente** (nessuna
  resurrezione silenziosa: la prevenzione del finding C3 resta intatta).
- **GC locale e retention (vault senza sync)** (Decisione 1): con l'oplog
  sempre-on, un vault senza peer esegue in F1 la versione degenere della
  compattazione — snapshot locale senza quorum (K=1) e retention di default
  **~90 giorni** di storia; attivando il sync si passa al regime causale
  pieno. La finestra di retention è un'impostazione visibile (sezione Dati)
  con testo onesto: "la cronologia locale copre gli ultimi X giorni".
- I record-antenato di divergenze `Body` aperte e gli envelope di rekey non
  consumati sono esclusi dal taglio.

## Edge case censiti

| Caso | Comportamento |
|---|---|
| Stesso record applicato due volte | no-op (LWW-map per campo idempotente) |
| Record fuori ordine / FK mancante | `OrditoPending`; risoluzione contro lo stato finale del target (tombstone incluso) |
| Field-update prima del create | bufferizzato: il primo record applicato di un entityId è sempre un create completo (NOT NULL) |
| delete vs update concorrenti | tombstone domina i campi con HLC ≤ HLC_delete; upsert con HLC maggiore resuscita (policy esplicita) |
| Due device creano tag con lo stesso nome | fusione **derivata alla proiezione** (nessun record emesso); id canonico = HLC creazione più alto |
| Merge concorrente crea ciclo di cartelle | edge con HLC minimo ignorato in proiezione, figlio di root; collisione nome → suffisso deterministico |
| Collisione `PromptVersions.Version` | nessuna rinumerazione: identità = Id, numero = etichetta di proiezione |
| Device con clock nel futuro | guardia anti-deriva HLC (warning ≥10 min); counter 32 bit assorbe i burst bulk |
| Ingest da cartella T1 | receive-update dell'HLC obbligatorio prima dell'apply (invariante del layer, non del trasporto) |
| Record con `schema` più nuovo del peer | applicazione bufferizzata, ma **trasporto/relay sempre attivo** (il peer vecchio inoltra ciò che non sa parsare) |
| Record con `schema` vecchio, app nuova | upcaster permanente dal registry (mai riscrittura dei record firmati) |
| Segmento corrotto / catena rotta | scartato il suffisso dalla rottura; re-fetch da altro peer (ridondanza ≥2 peer prima di considerare un range eliminabile) |
| Due record con stesso `prev` e stessa install | **equivocation** (chiave riusata da restore/clone): allarme + quarantena del ramo con HLC minore; il restore corretto genera un nuovo install_nonce |
| "Conflicted copy" creata da Dropbox in `.ordito/` | ignorata: fuori catena hash |
| Restore di un backup vecchio del vault | nuova identità install (nonce); i record della vecchia install restano validi nel log altrui; il device riparte dai cursori |
| Install stale (assente > 90gg) | preavvisi progressivi prima dello stale-out; al rientro: resync da snapshot + **revisione guidata** delle modifiche non pubblicate nella Vista conflitti (ri-applicazioni = record nuovi con HLC corrente; mai scarto silenzioso) |
| Vault senza sync (peer singolo) | GC locale: snapshot senza quorum (K=1), retention di default ~90 giorni (Decisione 1) |
| Purge dal cestino | locale alla proiezione; il tombstone resta finché la GC causale non lo prova consegnato a tutte le install note |
| Record replayato su un altro vault | rifiutato: `vault` è nel payload firmato |
| Eliminazione del vault | crypto-shredding del master seed → segmenti su trasporti terzi permanentemente illeggibili; pulizia dei blob offerta in UI |

## Strategia di verifica

Il claim di convergenza è il rischio n.1 e va testato come proprietà, non
come esempi (coerente col gate coverage 80% del repo):

- **Property test di convergenza**: generatore di storie casuali multi-device
  (proptest) → applica ogni permutazione/sottoinsieme progressivo dei record
  → asserzione: proiezioni identiche byte-a-byte, incluse varianti derivate,
  fusioni tag, numeri di versione. È il test che avrebbe catturato il
  finding "merge con effetti collaterali".
- **Simulatore multi-peer** con clock sfasati, partizioni, code d'uscita
  ritardate: scenari di resurrezione tombstone, install stale (incl.
  revisione al rientro), equivocation.
- **Fuzz** sui segmenti (troncamento, bit-flip, conflicted copy, epoche VSK
  mancanti) → mai panico, sempre quarantena/refetch.
- **Test del canale T2** (framing, keep-alive, riconnessione Noise): sono
  codice nostro (Decisione 4), vanno coperti come il resto.
- **Test di upcaster**: fixture di record per ogni schema storico, rigiocati
  a ogni release (regressione permanente).
- **Golden dei merge domain-native**: casi 3-way su Body (con e senza
  antenato), cicli cartelle ≥3, delete-vs-update — attesi deterministici.
- Gate: il crate `ordito/` entra nel coverage gate 80% dal primo giorno.

## Comprare vs inventare

| Pezzo | Decisione | Alternativa valutata |
|---|---|---|
| Formato record + semantica merge (LWW-map, merge puri, conflitto→variante derivata) | **inventare** (è piccolo ed è il cuore differenziante) | cr-sqlite (Apache/MIT): CRDT generico per SQLite, ma niente field-policy custom né conflitto-come-variante; ElectricSQL/PowerSync: richiedono Postgres centrale (contro l'obiettivo 1) |
| CRDT testo completo per il Body | **no** (LWW + variante derivata basta ed è più spiegabile) | Automerge/Yjs: potenti ma pesanti, merge char-level poco prevedibile per prompt |
| HLC | **implementare** (≈100 righe dal paper; counter 32 bit) | — |
| Riconciliazione range-based | **riusare/portare** negentropy (MIT, esiste in Rust e Go) | Merkle Search Trees (più complesso) |
| Trasporto T2 | **deciso: TCP + Noise XX (`snow`) + `mdns-sd`, solo-LAN** (Decisione 4) | **Iroh** retrocesso a eventuale T4 domanda-driven (P2P live cross-Internet): dipendenza pesante, relay n0, non copre comunque T3; libp2p (più grosso del necessario) |
| Serializzazione | CBOR (`ciborium` Rust / `fxamacker/cbor` Go) | JSON (verboso ma debuggabile — resta per export) |
| Hash/firma | BLAKE3 + Ed25519 (`ed25519-dalek`, già nel design Fase 5) | — |
| Key storage | `keyring` + **fallback file cifrato Argon2id** (spec F1) | solo keychain (bocciato: DPAPI #467, Linux headless) |

## Punti di tocco nel codice

| Area | Intervento | Sforzo |
|---|---|---|
| nuovo `ordito/` (crate o modulo Rust) | formato record, HLC, LWW-guard, append/apply, segmenti, upcaster registry, **snapshot/GC locale + retention** (Decisione 1) | alto (cuore) |
| **choke-point di mutazione** `apply_change(entity, id, old, new)` | UN punto che legge lo stato precedente, calcola il diff per-campo e appende al log in-TX; i 95 comandi Tauri vi migrano sopra | **alto** (la v1 diceva "medio/meccanico": falso — oggi `prompt_aggiorna` fa UPDATE full-row senza read-before-write e `sincronizza_tags` fa DELETE-all+re-INSERT; senza choke-point il field-level non è realizzabile) |
| indici derivati (FTS/embeddings) | da rebuild-completo-in-TX a **incrementali fuori TX** con dirty-queue persistente | medio-alto (prerequisito F1: la TX di scrittura oggi è già pesante) |
| `migrazione.rs` | tabelle `Oplog`, `OrditoApplied`, `OrditoPending`, `OrditoPeers`, dirty-queue (V016+) | basso |
| `sync.ts` + `sync.rs` | il push mancante diventa "spedisci record"; pull = "applica record" | medio |
| nuovo `ordito_cartella.rs` | trasporto T1: segmenti, viste firmate per-device, heartbeat, scansione | medio |
| nuovo `ordito_lan.rs` | trasporto T2: `mdns-sd`, pairing, canale Noise XX (`snow`), framing/keep-alive/riconnessione | medio-alto |
| key storage (`keyring` + fallback) | device key, master seed, envelope rekey, crypto-shredding | medio (nuovo prerequisito) |
| UI Svelte | "I miei dispositivi" (pairing, stato, revoca + avviso rimozione share, preavvisi stale-out); **Vista conflitti aggregata** (varianti + revisione rientro stale); flusso ri-emissione recovery phrase; retention nella sezione Dati; flusso "Elimina vault" con crypto-shredding | medio-alto |
| proiezione varianti-da-conflitto | derivazione pura (id da hash, label da HLC), 3-way con fallback | medio |
| `apps/server` | (F4) staging+ACK, catena workspace ri-firmata, certificato device→utente, Postgres | alto (ma è LO step v2.0) |
| repository trait | (F4) estrazione `PromptRepo`/`SearchIndex`/`EmbeddingStore`/… dalle funzioni `_pure` | alto, meccanico |

## Fasi (incrementali, ognuna utile da sola)

- **F1 — Fondamenta**: choke-point `apply_change` + oplog **sempre-on** in
  transazione (Decisione 1); HLC; indici derivati incrementali fuori TX;
  key storage con fallback; **GC locale + retention ~90gg** per i vault
  senza peer; property test di convergenza dal giorno 1. Ripara il push del
  client verso `papsync` attuale (push = spedire record).
  *(Valore anche senza P2P: sync bidirezionale corretto, audit firmato,
  time-travel entro la retention.)*
- **F2 — Trasporto cartella (T1)**: segmenti cifrati (VSK a epoche da master
  seed), viste firmate per-device, heartbeat, **snapshot a quorum + GC
  causale + stale-out 90gg** (estensione multi-peer della GC locale di F1);
  vettori embedding nello snapshot. Primo sync multi-device **senza alcun
  server**.
- **F3 — LAN P2P (T2) + conflitto→variante**: `mdns-sd`, pairing, canale
  Noise, riconciliazione negentropy, varianti derivate + Vista conflitti
  (incl. revisione rientro stale), UseCount G-counter.
  *(Il relay T3 è un'appendice opzionale di F3.)*
- **F4 — Enterprise (v2.x)**: repository trait, proiezione Postgres,
  `papsync` peer autoritativo (staging/ACK, catena workspace, certificati
  device→utente, RBAC/SSO/approval sopra il log). Si apre solo col cliente
  (gate invariato di `v2.0-enterprise.md`).

## Rischi e vincoli

- **Correttezza n.1**: mutazione tabelle + append log nella stessa
  transazione (produzione) e apply + LWW-guard nella stessa transazione
  (consumo). Gli indici derivati sono eventually-consistent via dirty-queue,
  mai dentro la TX critica.
- **Crescita del log**: contenuta per costruzione — retention locale ~90gg
  nei vault senza sync (F1), snapshot a quorum + GC causale + stale-out 90gg
  con sync attivo (F2). La UI mostra quando un device assente sta bloccando
  la compattazione, coi preavvisi dello stale-out.
- **Complessità percepita**: per l'utente il sync resta "accoppia i
  dispositivi e funziona" — in UI si chiama **"Sincronizzazione
  dispositivi"** (Decisione 7). Superfici nuove: "I miei dispositivi", Vista
  conflitti, recovery phrase, retention. Tutto il resto è invisibile.
- **Recupero chiavi**: la recovery phrase codifica il master seed e
  sopravvive alle rotazioni ordinarie; dopo un rekey da revoca va ri-emessa
  (flusso bloccante in UI).
- **Coesistenza col sync attuale**: durante F1-F2 `papsync` LWW resta il
  canale team. La migrazione del server al log è F4; niente doppio
  protocollo permanente.
- **Review crittografica esterna** prima di dichiarare stabile il trasporto
  cifrato (T1/T3) e il canale Noise (T2), come già previsto per l'E2E di
  Fase 5.
- **Performance primo sync**: da misurare in F2 con vault sintetici; ordine
  di grandezza atteso: il testo comprime bene (10k prompt ≈ decine di MB di
  log; lo snapshot molto meno); i vettori nello snapshot pesano ~1,5 KB per
  prompt (≈15 MB su 10k prompt) e azzerano il re-embedding al bootstrap.
  Numeri reali nel blueprint di F2.

## Cosa NON fare

- Non usare CRDT testuali char-level per il Body (merge imprevedibili sui
  prompt; la variante-conflitto derivata è più onesta e più utile).
- **Non emettere mai record come effetto collaterale di un merge**: ogni
  risoluzione è derivata alla proiezione. (È il vincolo che tiene in piedi
  la convergenza — qualunque eccezione futura va dimostrata pura.)
- **Non riscrivere mai record firmati**: né per compattazione (→ snapshot
  con commitment) né per migrazione schema (→ upcaster permanenti).
- Non costruire RBAC distribuito P2P: dove serve autorità c'è un peer
  autoritativo; in team mode topologia a stella rigida, niente gossip
  client-client.
- Non sincronizzare mai `ProviderConfig` (API key) né stato effimero UI.
- Non introdurre un ORM generico: trait per aggregato, mirati, quando serve
  la seconda implementazione (F4) — YAGNI prima.
- Non inventare trasporti nuovi: cartella, LAN, relay muto coprono tutto.
- **Non sincronizzare mai un vault a cartella via Ordito** (Decisione 5,
  permanente): doppio canale → echo e doppio conflict-resolver. E non usare
  la cartella del vault-a-cartella come cartella di trasporto T1.
- Non mettere "Ordito" in un pulsante o in un titolo di impostazioni: è il
  nome del motore, non della feature (Decisione 7).

## Decisioni chiuse (2026-07-07)

I 7 punti aperti della v2, decisi dall'autore dopo analisi guidata
(contenuto, impatti, opzioni con pro/contro per ciascuno):

1. **Oplog sempre-on con retention corta.** Ogni vault scrive l'oplog dalla
   migrazione V016, firmato; nei vault senza sync la GC locale tiene la
   storia entro ~90 giorni (parametro visibile), col sync attivo si passa al
   regime causale pieno. Motivo: un solo code path di scrittura (i property
   test coprono ogni vault reale), niente backfill all'attivazione del sync,
   e il contro privacy/disco del sempre-on è risolto dalla retention.
   Conseguenza: la GC locale (snapshot K=1) entra nello scope di **F1**.
   - Scartato "attivato col sync": doppio code path da testare, backfill
     sintetico, edge case di riattivazione, F1 svuotata di valore.
2. **Vettori embedding nello snapshot** (non nel log, non ricalcolo puro):
   blob opaco `(modello, versione)` nella sezione derivati, fuori dalle
   attestazioni. Bootstrap con semantica immediata, log invariato, ~1,5 KB
   per prompt solo nello snapshot; degradazione pulita a ricalcolo se il
   modello differisce. Obbligata comunque nello scenario E2E enterprise.
   - Scartato "nel log": gonfia la struttura eterna con vettori di stati
     intermedi legati al modello del device scrivente.
3. **Cartella `.ordito/` fuori dal vault, come replica rigenerabile.** Il
   log autoritativo vive nel `.db` SQLCipher; i segmenti sono contenuto del
   tubo, cancellabili senza perdita. Il modello "il vault è un file" esce
   rafforzato; lo smaltimento del vault diventa **crypto-shredding** del
   master seed (i blob remoti diventano illeggibili per sempre).
   - Scartato "vault come cartella" (invita il `.db` su share SMB →
     corruzione) e "T1 vietato ai vault cifrati" (ammazza il trasporto più
     economico per il vault di default).
4. **T2 = TCP + Noise XX (`snow`) + `mdns-sd`, solo-LAN.** Internet
   asincrono via T1/T3. **Iroh retrocesso a eventuale T4** domanda-driven
   (P2P live cross-Internet), con condizione di riapertura esplicita:
   dipendenza pesante, relay n0 in contrasto col claim "senza server",
   review crypto più larga, e comunque non copre T3.
5. **Vault a cartella + Ordito: chiuso per sempre.** Esclusione permanente,
   non più "design futuro separato". Chi usa il vault a cartella sincronizza
   con i propri strumenti (git/Syncthing) accettando i limiti documentati.
   Implicazione accettata: il bet "prompts-as-code" perde la combinazione
   `.md`+sync intelligente; riaprirla richiederà una revisione esplicita di
   questa decisione.
6. **K = tutte le install non-stale; N = 90 giorni automatico.** Snapshot
   eliminabile solo con controfirma di tutte le install attive (garanzia
   anti-S8 piena); stale-out automatico a 90 giorni con preavvisi
   progressivi; al rientro **revisione guidata** delle modifiche non
   pubblicate nella Vista conflitti (ri-applicazioni = record nuovi con HLC
   corrente; mai scarto silenzioso).
   - Scartato "solo manuale" (crescita illimitata se il nudge viene
     ignorato) e "maggioranza + 30gg" (con 2 device degenera e riapre S8;
     30gg cade dentro le assenze normali).
7. **Naming ibrido.** In UI sempre **"Sincronizzazione dispositivi"** (mai
   "Ordito" in un pulsante o titolo); "Ordito" è il nome del **motore** in
   docs di architettura, sito e release notes ("basata sul motore di sync
   Ordito"), con voce breve nel glossario. Nota codename: nel pool di
   `stagioni-e-nomi-rilascio.md` esiste «Onirico Ordito» (lettera O) —
   l'omonimia è governata lì: se la release che debutta il motore sarà una
   cardine, quel codename ne è il candidato naturale (omonimia deliberata).
   Attenzione redazionale al quasi-omonimo vietato "Ardito".
