# Blueprint — "Ordito": log replicato per sync senza server e storage database-agnostic

> **Stato**: design / ideazione. Nessun codice ancora.
> **Obiettivo utente**: (1) sincronizzare i propri vault tra più dispositivi in
> modo intelligente **senza server centrale obbligatorio**; (2) rendere lo
> strato dati **database-agnostic** per la versione Enterprise (on-prem e
> cloud), con un unico protocollo di comunicazione.
> **Scope**: fasi F1-F3 = SKU Personale (multi-device, stesso utente);
> fase F4 = primo mattone dello SKU Enterprise v2.x.
> **Nome**: *Ordito* — il filo portante del telaio su cui ogni dispositivo e
> ogni backend tesse la propria trama (coerente con la convenzione tessile di
> [`stagioni-e-nomi-rilascio.md`](./stagioni-e-nomi-rilascio.md)).

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
  scelto; `vault-a-cartella.md` dichiara vault a cartella e sync server
  **mutuamente esclusivi** (questo blueprint riconcilia i due mondi, v. §Trasporto T1).
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
2. **Database-agnostic gratis**: FTS5 e sqlite-vec smettono di essere un
   problema di portabilità perché sono **indici derivati** — ogni backend usa
   il suo equivalente nativo (Postgres FTS + pgvector), rigenerandolo dal log.
3. **Audit trail crittografico**: ogni record è firmato dal dispositivo che
   l'ha prodotto. La paternità non è più un campo del body falsificabile
   (risolve alla radice la classe di problemi di #450).
4. **Time-travel e backup incrementale**: il log *è* la storia; l'export Git
   di `prompts-as-code.md` diventa l'ennesima proiezione.

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
  op:       "upsert" | "delete"        // delete = tombstone, mai hard-delete nel log
  entity:   "prompt" | "tag" | "folder" | "prompt_tag" | "version"
            | "rating" | "golden" | "global" | ...   // v. tabella copertura
  entityId: "prm-..."                  // ULID/UUIDv7, invariato rispetto a oggi
  fields:   { "Title": "...", "Body": "..." }   // SOLO i campi cambiati (field-level)
  hlc:      "0190c2f3-0007-dev_a1b2"   // Hybrid Logical Clock (v. sotto)
  device:   "dev_a1b2"                 // id dispositivo = fingerprint chiave Ed25519
  prev:     "b3-..."                   // hash BLAKE3 del record precedente DELLO STESSO device
  hash:     "b3-..."                   // BLAKE3 del record (esclusi hash e sig)
  sig:      "ed25519-..."              // firma del device su hash
}
```

Regole:

- **Hash chain per dispositivo** (`prev`): ogni device produce una catena
  lineare verificabile. Manomissioni o buchi nella catena sono rilevabili.
  Le catene dei diversi device formano un DAG ordinato dall'HLC.
- **Field-level, non row-level**: `fields` contiene solo i campi modificati.
  È ciò che rende il merge "intelligente" (v. §Semantica di merge).
- **`delete` = tombstone**: coerente con il soft-delete `DeletedAt` già in uso.
  Il purge fisico (cestino) è un'operazione *locale* alla proiezione, non
  viaggia nel log come hard-delete (v. §Compattazione per il GC del log).
- **`schema`**: i record dichiarano la versione di schema con cui sono stati
  scritti. Un peer con schema più vecchio bufferizza i record più nuovi
  finché non si aggiorna (v. §Edge case).

### HLC — Hybrid Logical Clock

Sostituisce `UpdatedAt` wall-clock come criterio di ordinamento. Tripla
`(physical_time_ms, logical_counter, device_id)`:

- a ogni evento locale: `physical = max(now(), physical_ultimo)`; se
  `physical == physical_ultimo` incrementa `logical`, altrimenti `logical = 0`;
- alla ricezione di un record remoto: `physical = max(now(), physical_locale,
  physical_remoto)` e `logical` di conseguenza (regole standard del paper HLC);
- confronto totale: `(physical, logical, device_id)` lessicografico — il
  `device_id` rompe i pareggi in modo deterministico su tutti i peer.

Proprietà: monotono anche con orologi sfasati, cattura la causalità
(un record ricevuto e poi modificato ha sempre HLC maggiore), nessun
coordinamento richiesto. Encoding: stringa ordinabile lessicograficamente
(48 bit ms hex + 16 bit counter hex + device id), così gli indici SQL
ordinano correttamente senza parsing.

Guardia anti-deriva: se `now()` locale è più avanti di oltre 10 minuti
rispetto al massimo `physical` visto dalla rete di peer, warning all'utente
("orologio del dispositivo sospetto") — mitiga il device con clock nel futuro
che "vince" ogni LWW.

## Semantica di merge

### Regola generale: LWW-map per campo

Per ogni `(entity, entityId, campo)` vince il record con **HLC più alto**.
Applicazione idempotente e commutativa: applicare due volte lo stesso record,
o record in ordine diverso, produce lo stesso stato (è un CRDT a mappa LWW).
In pratica la proiezione tiene, per riga, l'HLC dell'ultimo write applicato
per campo (tabella ausiliaria `OrditoApplied(entityId, field, hlc)`), e
scarta i record più vecchi.

Esempio del guadagno rispetto all'LWW attuale: il portatile cambia il titolo,
il fisso (offline) cambia i tag dello stesso prompt → **entrambe** le
modifiche sopravvivono. Oggi una delle due verrebbe scartata intera.

### Conflitto vero sul `Body`: il perdente diventa una variante

Caso: due device offline modificano **lo stesso campo `Body`** dello stesso
prompt. LWW da solo butterebbe via un lavoro potenzialmente prezioso.

Regola Ordito (la parte domain-native del design):

1. vince comunque l'HLC più alto → il `Body` "ufficiale" è deterministico
   e identico su tutti i peer;
2. il `Body` perdente **non viene scartato**: viene materializzato come
   **variante A/B** del prompt (`ParentPromptId` + `VariantLabel`
   auto-assegnata, es. `sync-2026-07-06-dev_a1b2`), feature già esistente (V011);
3. l'utente vede la variante nella UI varianti già esistente, la confronta,
   e la promuove o la elimina — **riusa la UI di promozione varianti al posto
   di una UI conflitti dedicata** (quella cancellata nel redesign non serve più);
4. opzionale (Fase Deluxe): se il prompt ha golden test, l'app li esegue su
   entrambe le versioni e suggerisce la vincente.

Soglia pratica: la variante-conflitto si crea solo se il `Body` perdente
differisce da quello vincente *e* dall'ultimo antenato comune (3-way check
usando `PromptVersions`); altrimenti è un falso conflitto e si scarta.

### Entità append-only: merge banale

`PromptRatings`, `AuditLog`, `PromptRunObservations` sono append-only per
design → il merge è l'unione degli insiemi (dedup per `Id`). Nessun
conflitto possibile. Sono i cittadini ideali del log.

### Copertura entità

| Entità | Nel log? | Merge | Note |
|---|---|---|---|
| Prompts | ✅ | LWW per campo + conflitto-Body→variante | cuore del sistema |
| PromptVersions | ✅ | append-only (immutabili) | UNIQUE(PromptId,Version): in caso di collisione di `Version` da due device, rinumerazione deterministica per HLC |
| Tags / PromptTags | ✅ | LWW per campo / add-wins set | `UNIQUE(WorkspaceId,Name)`: collisione nome → merge dei due tag (id vincente per HLC, riassegnazione PromptTags) |
| Folders | ✅ | LWW per campo | cicli da merge concorrente di `ParentFolderId`: detection e re-parent a root (v. Edge case) |
| PromptRatings | ✅ | unione (append-only) | |
| PromptGoldens / RunObservations | ✅ / ✅ | LWW / unione | le run sono osservazioni storiche |
| GlobalPlaceholders | ✅ | LWW per riga | |
| PromptImports | ✅ | derivata: ricostruita dal parsing del Body alla proiezione | non serve nel log |
| AuditLog | ✅ (locale→log) | unione | il log firmato È l'audit; la tabella diventa proiezione |
| ProviderConfig | ❌ **mai** | — | contiene API key: restano locali al device |
| SyncMeta / preferenze UI / UseCount, LastUsedAt | ❌ (default) | — | stato locale/effimero; `UseCount` eventualmente come contatore CRDT (somma di incrementi) in fase successiva |
| Users / Workspaces | ✅ (solo team) | LWW server-authoritative | nel P2P personale c'è un solo utente |

## Riconciliazione tra peer

Obiettivo: due peer scoprono *cosa manca all'altro* senza trasferire l'intero
log. Due meccanismi complementari:

1. **Checkpoint per coppia di peer** (alla CouchDB): per ogni device remoto
   noto si salva l'ultimo `(device, seq)` ricevuto per catena. Nel caso
   comune (peer che si parlano spesso) basta chiedere "dammi tutto dopo
   questi cursori" — un vettore di versione, O(1) round-trip.
2. **Riconciliazione range-based** (alla negentropy) come fallback robusto:
   se i cursori non bastano (peer nuovo, log compattato, cartella condivisa
   scritta da terzi), i peer confrontano fingerprint di range dell'insieme
   dei record ordinati per HLC, dividendo ricorsivamente i range che
   differiscono. Costo logaritmico, nessuno stato per-peer richiesto.
   Esistono implementazioni MIT in Rust e Go (negentropy) da riusare o
   riscrivere (l'algoritmo è pubblico e piccolo).

L'applicazione è **idempotente** (LWW-map), quindi ricevere due volte lo
stesso record è innocuo: la riconciliazione può essere approssimativa in
eccesso senza rischi di correttezza.

## Trasporti (il canale è stupido)

### T1 — Cartella condivisa (`.ordito/`) — costo minimo, valore immediato

Ogni device appende **segmenti di log** come file immutabili in una cartella
qualsiasi (Syncthing, Dropbox, NAS, chiavetta USB):

```
<cartella-sync>/.ordito/
├─ manifest.json                  # versione formato, elenco device noti (pubkey)
├─ dev_a1b2/
│  ├─ 000001.seg                  # segmento: batch di record CBOR, cifrato, ~1-4 MB
│  ├─ 000002.seg
│  └─ head.json                   # ultimo seq + hash, firmato
└─ dev_c3d4/
   └─ ...
```

- **Ogni device scrive SOLO nella propria sottocartella** → zero conflitti di
  file-locking, zero corruzione da SMB/NFS (il problema che `vault-a-cartella.md`
  teme per il `.db` su share **non esiste**: file append-only immutabili).
  Al peggio il servizio di file-sync duplica un file ("conflicted copy") che
  viene ignorato perché fuori catena.
- I segmenti sono **cifrati** (v. §Sicurezza): la cartella può stare su
  Dropbox senza esporre i prompt.
- Questo trasporto **riconcilia vault-a-cartella e sync**: il vault a
  cartella può ospitare `.ordito/` accanto ai `.md`, e il sync multi-device
  passa dai file, non da SQLite.

### T2 — LAN diretta

- Discovery **mDNS/DNS-SD** (`_pap-ordito._tcp`), come Syncthing.
- **Pairing**: l'identità del device è la sua chiave pubblica Ed25519.
  Accoppiamento con QR code o codice breve a 6 parole (verifica out-of-band
  del fingerprint, stile Syncthing/Signal). L'elenco device fidati vive nel
  vault e si sincronizza esso stesso via log.
- Canale: QUIC/TLS o Noise, con mutua autenticazione sulle chiavi device.
- Riuso possibile: **Iroh** (Rust, MIT/Apache) fornisce QUIC P2P +
  hole-punching + relay già pronti; alternativa minimale: TCP+Noise fatto in
  casa solo-LAN.

### T3 — Relay "muto" (store-and-forward, opzionale)

Per device mai accesi insieme e senza cartella condivisa: un servizio che
accetta e serve **blob cifrati opachi** per topic (= id vault), senza poterli
leggere. Chiunque può hostarlo; un futuro "PaP Cloud" può offrirlo come
servizio gestito **senza diventare trusted**. API: 3 endpoint
(`PUT /topic/{id}/seg`, `GET /topic/{id}/since/{cursor}`, `GET /health`).

## Sicurezza

Ricicla il design crittografico già scritto per Fase 5 Step 5 (E2E), con una
semplificazione: nel caso personale non serve lo scambio multi-utente.

- **Device key**: coppia Ed25519 per dispositivo, generata al primo avvio,
  privata nel keychain OS (`keyring`, già pianificato per `sync_token`).
  Firma ogni record e autentica i canali T2/T3.
- **Vault sync key (VSK)**: chiave simmetrica XChaCha20-Poly1305 per vault,
  generata alla prima attivazione del sync, distribuita ai device al pairing
  (cifrata X25519 verso la chiave del nuovo device). Cifra i segmenti su T1/T3.
  Su T2 (canale già mutuamente autenticato e cifrato) è difesa in profondità.
- **Revoca device**: rimozione dalla lista fidata (evento nel log, firmato da
  un device rimanente) + **rotazione VSK** distribuita ai superstiti. Il
  device revocato conserva ciò che aveva già (inevitabile) ma non decifra i
  segmenti futuri.
- Nel caso **team** (F4): la WMK per-workspace e il rituale membri restano
  quelli di `fase-5-enterprise.md` Step 5 — Ordito non li cambia, li trasporta.

### Due modelli di fiducia, un protocollo

| | Personale multi-device (F1-F3) | Team/Enterprise (F4) |
|---|---|---|
| Peer | i device dell'utente, tutti equivalenti | client + **peer con autorità** (server) |
| Trust | totale (stesso proprietario) | RBAC/SSO/approval applicati dal peer autoritativo |
| Topologia | mesh (T1/T2/T3) | stella verso il server (che internamente è solo un peer col DB grosso) |
| Autorizzazione | firma device sufficiente | il server valida ruolo/permessi PRIMA di accettare un record nel log del workspace; record rifiutati → risposta di reject al client |

Questa separazione evita il problema irrisolto del "RBAC in P2P puro":
non lo risolviamo, lo evitiamo — dove serve autorità, c'è un peer autoritativo.

## Il database come proiezione (obiettivo Enterprise)

### Applicazione del log

La proiezione consuma record in qualunque ordine e converge:

1. upsert idempotente per `(entity, entityId)`, guardato dalla LWW-map per campo;
2. FK mancanti tollerate: record "orfano" (es. prompt che referenzia una
   cartella non ancora vista) → applicato con FK a NULL + parcheggiato in
   `OrditoPending`; alla comparsa del target si ricollega. (Alternativa più
   semplice per F1: applicare i batch in ordine topologico per entità —
   folders → tags → prompts → associazioni.)
3. dopo l'apply: refresh degli indici derivati (FTS, embeddings) — logica già
   esistente, invariata.

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
naturale (il pattern _pure/_impl consolidato in M7). Il dialetto SQL si
neutralizza dove serve: `datetime('now')` → timestamp applicativi (già
necessari per l'HLC), indici parziali → equivalenti Postgres (li ha),
`INSERT OR REPLACE` → `ON CONFLICT DO UPDATE`.

**Cosa resta volutamente non portabile**: SQLCipher è il formato del *vault
personale* (file portable cifrato = requisito di prodotto), non dello strato
enterprise; on-prem/cloud la cifratura at-rest la dà il backend (TDE/volume)
o resta E2E sul log stesso.

### Il server enterprise diventa un consumer del log

`papsync` evoluto = peer autoritativo che: valida (RBAC, approval workflow),
appende al log del workspace, proietta su Postgres, notifica via WS.
Gli step Fase 5 (webhook, API pubblica, approval) si costruiscono **sopra il
log** (un webhook è un consumer; l'approval è uno stato che gate-a
l'accettazione dei record), non sopra un secondo protocollo. `SyncChangelog`
attuale è l'antenato diretto: migra da tabella server-only a formato condiviso.

## Compattazione del log

Il log cresce indefinitamente → GC a due livelli:

- **Snapshot + troncamento** (come i checkpoint WAL): periodicamente un
  device scrive uno *snapshot record* (stato completo compresso del vault a
  un HLC di taglio, firmato) e i segmenti interamente sotto il taglio
  diventano eliminabili. Un peer nuovo parte dallo snapshot + coda del log.
- **Squash per entità**: record dello stesso `(entity, entityId, campo)`
  soppiantati da HLC successivi sono rimovibili in riscrittura dei segmenti
  (mantenendo il primo e l'ultimo per l'audit, configurabile).
- Il taglio è **conservativo**: mai sotto il cursore del peer fidato più
  arretrato visto di recente; peer assente oltre N giorni → resync da snapshot.

## Edge case censiti

| Caso | Comportamento |
|---|---|
| Stesso record applicato due volte | no-op (LWW-map idempotente) |
| Record fuori ordine / FK mancante | parcheggio in `OrditoPending` + ricollegamento, o apply topologico per batch |
| Due device creano tag con lo stesso nome | merge deterministico: vince l'id con HLC più alto, PromptTags riassegnate |
| Merge concorrente crea ciclo di cartelle (A sotto B, B sotto A) | detection alla proiezione → il record con HLC più basso re-parenta a root + notifica |
| Collisione `PromptVersions.Version` da due device | rinumerazione deterministica per HLC (l'identità è l'`Id` della version, non il numero) |
| Device con clock nel futuro | guardia anti-deriva HLC (warning ≥10 min); il logical counter evita blocchi |
| Record con `schema` più nuovo del peer | bufferizzato, applicato dopo l'aggiornamento dell'app (il vault segnala "aggiorna per sincronizzare") |
| Segmento corrotto / catena rotta | scartato il suffisso dalla rottura in poi; re-fetch da altro peer (i dati esistono altrove per costruzione) |
| "Conflicted copy" creata da Dropbox in `.ordito/` | ignorata: fuori catena hash del device |
| Restore di un backup vecchio del vault | il device riparte dal suo ultimo `seq` noto: i peer gli rimandano il delta; i record che aveva prodotto e perso restano validi nel log altrui |
| Purge dal cestino | operazione locale alla proiezione; il tombstone resta nel log finché la compattazione non lo squasha |

## Comprare vs inventare

| Pezzo | Decisione proposta | Alternativa valutata |
|---|---|---|
| Formato record + semantica merge (LWW-map, conflitto→variante) | **inventare** (è piccolo ed è il cuore differenziante) | cr-sqlite (Apache/MIT): CRDT generico per SQLite, ma niente field-policy custom né conflitto-come-variante; ElectricSQL/PowerSync: richiedono Postgres centrale (contro l'obiettivo 1) |
| CRDT testo completo per il Body | **no** (LWW + variante basta ed è più spiegabile) | Automerge/Yjs: potenti ma pesanti, merge char-level poco prevedibile per prompt |
| HLC | **implementare** (≈100 righe dal paper) | — |
| Riconciliazione range-based | **riusare/portare** negentropy (MIT, esiste in Rust e Go) | Merkle Search Trees (più complesso) |
| Trasporto P2P (T2/T3) | valutare **Iroh** (Rust, MIT/Apache) vs TCP+Noise minimale solo-LAN | libp2p (più grosso del necessario) |
| Serializzazione | CBOR (`ciborium` Rust / `fxamacker/cbor` Go) | JSON (verboso ma debuggabile — resta per export) |
| Hash/firma | BLAKE3 + Ed25519 (`ed25519-dalek`, già nel design Fase 5) | — |

## Punti di tocco nel codice

| Area | Intervento | Sforzo |
|---|---|---|
| nuovo `ordito/` (crate o modulo Rust) | formato record, HLC, LWW-guard, append/apply, segmenti | alto (cuore) |
| tutti i comandi di mutazione (`prompt.rs`, `cartelle.rs`, `editor.rs`, …) | write-through: ogni mutazione appende al log **nella stessa transazione** SQLite | medio (meccanico, tanti punti) |
| `migrazione.rs` | tabelle `Oplog`, `OrditoApplied`, `OrditoPending`, `OrditoPeers` (V016+) | basso |
| `sync.ts` + `sync.rs` | il push mancante diventa "spedisci record"; pull = "applica record" | medio |
| nuovo `ordito_cartella.rs` | trasporto T1: segmenti, manifest, scansione | medio |
| nuovo `ordito_lan.rs` | trasporto T2: mDNS, pairing, canale | medio-alto |
| UI Svelte | schermata "I miei dispositivi" (pairing, stato, revoca); badge variante-conflitto | medio |
| `varianti.rs` | costruttore "variante da conflitto sync" (3-way check con `PromptVersions`) | basso-medio |
| `apps/server` | (F4) diventa consumer/peer autoritativo del log; Postgres come proiezione | alto (ma è LO step v2.0) |
| repository trait | (F4) estrazione `PromptRepo`/`SearchIndex`/`EmbeddingStore`/… dalle funzioni `_pure` | alto, meccanico |

## Fasi (incrementali, ognuna utile da sola)

- **F1 — Fondamenta (oplog + HLC)**: tabella `Oplog` scritta in transazione
  con ogni mutazione; HLC al posto di `UpdatedAt` come criterio di merge;
  ripara il push del client verso `papsync` attuale (push = spedire record).
  *(Valore anche senza P2P: sync bidirezionale corretto, audit firmato.)*
- **F2 — Trasporto cartella (T1)**: segmenti `.ordito/` cifrati su cartella
  condivisa; pairing implicito via VSK esportata (QR/frase). Primo sync
  multi-device **senza alcun server**, con l'infrastruttura che l'utente ha già.
- **F3 — LAN P2P (T2) + conflitto→variante**: mDNS, pairing device, UI
  "I miei dispositivi", riconciliazione negentropy, varianti da conflitto.
  *(Il relay T3 è un'appendice opzionale di F3.)*
- **F4 — Enterprise (v2.x)**: repository trait, proiezione Postgres,
  `papsync` peer autoritativo (RBAC/SSO/approval sopra il log). Primo mattone
  concreto dello SKU Enterprise; si apre solo col cliente (gate invariato di
  `v2.0-enterprise.md`).

## Rischi e vincoli

- **Doppia scrittura F1**: mutazione tabelle + append log DEVONO stare nella
  stessa transazione SQLite, o si diverge. È il vincolo di correttezza n. 1.
- **Crescita del log**: senza compattazione un vault molto attivo cresce di
  MB/mese. Snapshot+squash vanno consegnati entro F2, non "dopo".
- **Complessità percepita**: per l'utente il sync deve restare "accoppia i
  dispositivi e funziona". Tutta la meccanica (HLC, catene, segmenti) è
  invisibile; l'unica superficie nuova è "I miei dispositivi" + le varianti
  da conflitto.
- **Recupero chiavi**: perdere tutti i device = perdere la VSK. Mitigazione:
  frase di recupero stampabile alla creazione (stessa UX delle recovery
  phrase note).
- **Coesistenza col sync attuale**: durante F1-F2 `papsync` LWW resta il
  canale team. La migrazione del server al log è F4; niente doppio protocollo
  permanente.
- **Review crittografica**: prima di dichiarare stabile il trasporto cifrato
  (T1/T3) serve una review esterna, come già previsto per l'E2E di Fase 5.

## Cosa NON fare

- Non usare CRDT testuali char-level per il Body (merge imprevedibili sui
  prompt; la variante-conflitto è più onesta e più utile).
- Non costruire RBAC distribuito P2P: dove serve autorità c'è un peer
  autoritativo (il server). Il P2P puro resta single-user multi-device.
- Non sincronizzare mai `ProviderConfig` (API key) né stato effimero UI.
- Non fare del log un requisito: un vault senza sync attivo non paga nulla
  (l'oplog si attiva alla prima configurazione sync… oppure sempre-on ma
  con squash aggressivo — da decidere, v. Punti aperti).
- Non introdurre un ORM generico: trait per aggregato, mirati, quando serve
  la seconda implementazione (F4) — YAGNI prima.
- Non inventare trasporti nuovi: cartella, LAN, relay muto coprono tutto.

## Punti aperti (da decidere prima di F1)

1. **Oplog sempre-on o attivato col sync?** Sempre-on dà audit firmato e
   time-travel anche ai vault solo-locali, ma scrive di più. Propensione:
   sempre-on con squash aggressivo di default.
2. **`UseCount`/`LastUsedAt` nel log** come contatore CRDT (somma di
   incrementi) o locali per device? Propensione: locali in F1, contatore in F3.
3. **Vault cifrato + T1**: la cartella `.ordito/` può vivere *fuori* dal
   vault SQLCipher (è cifrata per conto suo). Confermare che non violi il
   modello mentale "tutto il vault è un file".
4. **Iroh vs Noise fatto in casa** per T2: dipende dal peso della dipendenza
   (Iroh è grande) vs costo di maturazione di un canale proprio.
5. **Nome pubblico della feature**: "Ordito" come nome interno/protocollo;
   per l'utente probabilmente solo "Sincronizzazione dispositivi".
