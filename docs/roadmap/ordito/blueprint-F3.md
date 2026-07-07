# Blueprint operativo — Ordito F3 "LAN P2P + conflitto→variante"

> **Stato**: design operativo. Nessun codice ancora. Presuppone F1+F2.
> **Prerequisito**: [`../ordito-sync-log.md`](../ordito-sync-log.md) v3 +
> [`blueprint-F2.md`](./blueprint-F2.md) (record nativi, VSK, snapshot, GC
> causale).
> **Obiettivo F3**: sync live in LAN (T2: mDNS + Noise XX via `snow`,
> Decisione 4), riconciliazione negentropy, **variante-da-conflitto
> derivata** + **Vista conflitti** (incl. revisione rientro stale di F2),
> UseCount come G-counter. Appendice opzionale: relay muto T3.
> **Fuori scope F3**: Iroh/hole-punching (eventuale T4, Decisione 4),
> server enterprise (F4).

## Nuovi moduli e dipendenze

```
src/ordito/
├─ t2.rs         // mDNS advertise/browse, listener TCP, ciclo di sessione
├─ noise.rs      // wrapper snow: handshake XX, cifratura frame, SAS pairing
├─ negentropy.rs // riconciliazione range-based su (Hlc, Hash)
└─ conflitti.rs  // proiezione variante-da-conflitto + 3-way + Vista conflitti
```

Dipendenze nuove: `snow` (Noise Protocol), `mdns-sd` (discovery),
`negentropy` (implementazione Rust esistente, MIT — se inadeguata si porta
l'algoritmo: è piccolo e pubblico). `x25519-dalek` già presente da F2.

## Canale T2 (`noise.rs`, `t2.rs`) — Decisione 4

- **Identità di canale**: chiave statica **X25519 per install** (quella già
  pubblicata nella `view.json` in F2), legata alla device key Ed25519 da
  una **firma di binding** (`sig_ed25519("pap-noise-bind" ‖ x25519_pub)`).
  Il handshake autentica la chiave X25519; il binding la aggancia
  all'identità di catena.
- **Handshake**: `Noise_XX_25519_ChaChaPoly_BLAKE2s` via `snow`. Dopo lo XX
  ciascun lato invia il proprio certificato di binding; si accetta solo se
  l'install è nella vista fidata (o si entra nel flusso pairing, sotto).
- **Framing**: frame length-prefixed `u32 LE` (max 4 MiB), payload CBOR.
  Messaggi: `Hello{install, vault, cursori}`, `NegOpen/NegMsg` (negentropy),
  `Records{[...]}`, `Ack{seq}`, `Ping/Pong`, `Bye`. Keep-alive 30s, idle
  timeout 90s, riconnessione con backoff (1s→60s). Tutto codice nostro:
  coperto dai test di canale (loopback) come da §Verifica del design v3.
- **Discovery**: `_pap-ordito._tcp.local`, TXT record = `{vault_fp:
  blake3(vault_id)[..8], install}` — il vault id NON è in chiaro nel TXT
  (fingerprint troncato: sufficiente per matchare, non correlabile
  altrove). Porta effimera, listener attivo solo a vault aperto.
- **Ciclo di sessione**: Hello → confronto cursori (fast path) → negentropy
  se i cursori non bastano → scambio `Records` a batch (500/frame) →
  `Hlc::receive` + `applica_remoto` (identico a F2: un solo consumer per
  tutti i trasporti) → aggiorna `OrditoPeers` + heartbeat.

## Pairing in LAN (upgrade del flusso F2)

- Il device nuovo appare via mDNS come "non accoppiato"; l'utente avvia
  l'accoppiamento da un device fidato.
- Handshake XX senza fiducia → entrambi mostrano una **Short Authentication
  String**: 6 parole derivate da `blake3(handshake_hash)` sulla wordlist
  BIP39 italiana. Sul device nuovo le parole vanno **digitate** (non solo
  confermate) la prima volta — mitiga la conferma cieca.
- Alla conferma: il fidato invia il **master seed** in un envelope X25519
  dentro il canale (mai su disco), registra la nuova install nella propria
  `view.json`; il nuovo deriva le VSK e parte col bootstrap (da snapshot se
  presente su T1, altrimenti negentropy completa).
- Il QR/frase di F2 resta come fallback senza LAN.

## Variante-da-conflitto derivata (`conflitti.rs`) — design v3 §Merge

Trigger: in `applica_remoto`, un record `Body` perde il LWW (HLC minore del
guard) **oppure** vince spodestando un valore locale non ancora visto dal
mittente. In entrambi i casi il valore perdente è candidato variante.

Algoritmo (funzione pura del log, Invariante fondante):

1. **3-way**: antenato = valore `Body` del record con HLC massimo tra
   quelli `< min(HLC_vincente, HLC_perdente)` presenti nel log unito per
   quell'entità (fallback: antenato assente ⇒ conflitto reale). Se
   `perdente == vincente` o `perdente == antenato` → falso conflitto, stop.
2. **Materializzazione derivata** (migrazione V018-F3: colonna
   `Prompts.ConflittoSorgenteHash TEXT NULL`):
   `Id = "prm-" + hex(blake3(hash_record_perdente))[..16]`,
   `ParentPromptId = entità in conflitto`, `IsVariant = 1`,
   `VariantLabel = "sync-" + hlc_perdente`,
   `ConflittoSorgenteHash = hash_record_perdente`.
   Upsert idempotente: se la riga esiste già (stesso hash sorgente), no-op —
   ogni peer che proietta lo stesso log produce la stessa riga. La riga
   derivata NON genera record Oplog (mai emettere dal merge); viene
   ricostruita identica da chiunque rigiochi il log.
3. **Decadimento**: se il `Body` ufficiale diventa uguale a quello della
   variante (o l'utente la elimina), la variante decade (soft-delete
   locale della riga derivata — anch'esso non loggato: è proiezione).
4. **Promozione/modifica**: azioni utente sulla variante derivata la
   "adottano" (da lì in poi è un prompt normale: le azioni emettono record
   Oplog regolari con un create completo; `ConflittoSorgenteHash` resta
   come provenienza).

Property test dedicati: stessa storia di conflitto applicata in ogni
ordine/su ogni peer ⇒ stessa variante (stesso Id), una sola volta;
promozione su un peer + decadimento su un altro convergono.

## Vista conflitti (UI)

Nuova superficie aggregata (pattern Modale + `statoModale` consolidato):

- Elenco a livello vault: **varianti da conflitto** in attesa (per prompt:
  ufficiale vs variante, diff affiancato, azioni promuovi/elimina/apri) +
  **revisione rientro stale** (da `OrditoStaleQueue` F2: elenco modifiche
  parcheggiate, azioni per-riga riapplica/scarta; le ri-applicazioni sono
  record nuovi con HLC corrente — Decisione 6).
- Badge contatore su "Sincronizzazione dispositivi" e nel tray.
- Soglia di igiene (design v3): sopra N conflitti pendenti (default 10)
  prompt di revisione guidata.
- Opzionale dietro flag (Deluxe): "esegui i golden su entrambe" con
  suggerimento della vincente — solo se il prompt ha golden definiti.

## UseCount come G-counter (design v3 §Copertura)

- Nuova entità loggata `contatore`: `entityId = "prm-X|use"`, `fields =
  { <install_id>: <totale_progressivo_di_quella_install> }`.
- Ogni install scrive **solo il proprio campo**: la LWW-map per campo
  converge banalmente (unico scrittore per campo, valore monotono).
  Proiezione: `UseCount = Σ campi`; `LastUsedAt` = HLC massimo tra i campi
  (derivato, non loggato a parte).
- `prompt_registra_uso` migra su `apply_change` con questa entità.
- Lo snapshot ripiega i campi correnti (già coperto dal formato F2).

## Appendice opzionale — Relay muto T3

Solo se resta budget di fase; altrimenti slitta senza toccare il resto.

- Binario Go separato `pap-relay` (riusa lo scheletro chi/router di
  `papsync`, ma NESSUNA conoscenza di utenti/prompt): 3 endpoint
  (`PUT /topic/{id}/seg`, `GET /topic/{id}/since/{cursor}`,
  `GET /health`), storage su filesystem, quota per topic, TTL.
- `topic_id = hex(blake3::derive_key("pap-ordito-topic", master_seed))[..32]`
  — ruotabile con l'epoca (mitigazione correlazione, design v3 §T3).
- Auth minima: bearer `blake3::derive_key("pap-ordito-relay-auth",
  master_seed)` (chi ha il seed legge/scrive; il relay non decifra nulla).
- Lato client: `t3.rs` = stesso formato segmenti di T1, publish/poll HTTP.
  Il consumer resta `applica_remoto` — zero logica nuova di merge.

## Sequenza delle PR

| PR | Contenuto | Dipende da | Stima |
|---|---|---|---|
| F3-PR1 | `negentropy.rs` (crate o port) + property test riconciliazione (insiemi divergenti casuali → scambio minimo, convergenza) | F2 | media |
| F3-PR2 | `noise.rs`: handshake XX, binding Ed25519↔X25519, framing, keep-alive; test loopback + fuzz frame | F2 | media-alta |
| F3-PR3 | `t2.rs`: mDNS, sessione completa (Hello/cursori/negentropy/Records), riconnessione | PR1+PR2 | alta |
| F3-PR4 | pairing LAN con SAS 6 parole + invio master seed in canale; UI accoppiamento | PR3 | media |
| F3-PR5 | `conflitti.rs` + V018 (`ConflittoSorgenteHash`) + property test variante derivata | F2 | alta |
| F3-PR6 | Vista conflitti UI (varianti + revisione stale) + badge | PR5 | media-alta |
| F3-PR7 | UseCount G-counter (`prompt_registra_uso` → `apply_change`) | F2 | bassa |
| F3-PR8 | (opz) `pap-relay` Go + `t3.rs` client | PR3 | media |

## Criteri di completamento F3

- [ ] Due install nella stessa LAN convergono in <5s da una modifica (mDNS
      + sessione + apply), senza cartella condivisa configurata.
- [ ] Pairing LAN completo: SAS digitata sul nuovo, master seed trasferito
      solo in canale, install registrata nelle viste.
- [ ] Conflitto Body reale (editing offline concorrente) produce UNA
      variante identica (stesso Id) su tutti i peer, visibile nella Vista
      conflitti; la promozione da un peer converge ovunque.
- [ ] Revisione rientro stale operativa dalla Vista conflitti (le azioni
      emettono record nuovi con HLC corrente).
- [ ] UseCount coerente cross-device (somma dei sub-counter), nessun
      double-count nei property test di ri-consegna.
- [ ] Il canale T2 rifiuta: install revocate, vault diversi, binding
      mancante, frame malformati (fuzz) — sempre senza panico.
- [ ] Coverage `ordito/` ≥80%; property test di convergenza estesi a T2
      (3 install, partizioni e riconnessioni casuali).

## Rischi operativi

- **mDNS su reti ostili** (VLAN isolate, AP isolation): documentare il
  fallback "usa la cartella condivisa (T1) o il relay (T3)"; il TXT non
  espone il vault id in chiaro.
- **negentropy crate immaturo**: fallback dichiarato = port interno
  dell'algoritmo (pubblico e compatto); l'interfaccia `negentropy.rs` isola
  la scelta.
- **Sessioni simultanee** (A↔B e B↔A): tie-break deterministico — mantiene
  la sessione l'install con id lessicograficamente minore; l'altra chiude.
- **Vista conflitti che diventa un cimitero**: la soglia di igiene e il
  decadimento automatico sono parte dei criteri, non rifiniture.
