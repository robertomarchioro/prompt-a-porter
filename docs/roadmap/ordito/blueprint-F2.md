# Blueprint operativo — Ordito F2 "Trasporto cartella"

> **Stato**: design operativo. Nessun codice ancora. Presuppone F1 atterrata.
> **Prerequisito**: [`../ordito-sync-log.md`](../ordito-sync-log.md) v3 +
> [`blueprint-F1.md`](./blueprint-F1.md) (oplog, HLC, apply, keys, GC locale).
> **Obiettivo F2**: primo sync multi-device **senza alcun server** via
> cartella condivisa (T1): master seed + VSK a epoche + recovery phrase,
> segmenti cifrati, viste firmate per-device con heartbeat, consumo di
> record Ordito nativi, snapshot a quorum con GC causale e stale-out 90gg,
> vettori embedding nello snapshot (Decisioni 1/2/3/6).
> **Fuori scope F2**: LAN/mDNS/Noise (F3), varianti-da-conflitto e Vista
> conflitti (F3 — v. §Stale, parcheggio), relay T3, server enterprise (F4).

## Nuovi moduli e dipendenze

```
src/ordito/
├─ vsk.rs        // master seed, epoche, recovery phrase, envelope X25519
├─ segmenti.rs   // formato .seg: encode/decode, cifratura, rotazione
├─ t1.rs         // trasporto cartella: publish, scan, viste, heartbeat
├─ snapshot.rs   // snapshot multi-peer: copertura, attestazioni, derivati, controfirme
└─ gc.rs         // ESTESO: GC causale multi-peer + stale-out (da F1: caso K=1)
```

Dipendenze nuove: `chacha20poly1305` (XChaCha20-Poly1305, già nel design
Fase 5), `x25519-dalek` (envelope), `bip39` (recovery phrase, wordlist
standard), `zstd` (compressione segmenti/snapshot). Il KDF per le epoche NON
richiede dipendenze: `blake3::derive_key(context, master_seed)` con context
`"pap-ordito-vsk-epoch-{e}"`.

## Chiavi (`vsk.rs`) — Decisioni 3 e 6, design v3 §Sicurezza

- **Master seed**: 32 byte random alla prima attivazione del sync, salvato
  col medesimo meccanismo della device key (keychain + fallback F1).
- **Recovery phrase**: mnemonica BIP39 a 24 parole = codifica del master
  seed. Mostrata UNA volta alla creazione con flusso bloccante ("l'hai
  scritta?" + re-inserimento di 3 parole a campione). Ri-emessa
  obbligatoriamente dopo un rekey da revoca (nuovo master).
- **VSK per epoca**: `VSK_e = blake3::derive_key("pap-ordito-vsk-epoch-{e}",
  master_seed)`. L'epoca corrente vive in `OrditoMeta('vsk_epoca')`.
- **Pairing F2 (senza canale di rete)**: il nuovo device genera la propria
  device key e pubblica la propria `view.json` nella cartella; l'utente
  trasferisce il master seed **out-of-band** — QR mostrato dal device già
  accoppiato o inserimento della recovery phrase. Nessun envelope in
  cartella al primo pairing (il segreto non transita mai sul trasporto:
  QR/frase sono off-line).
- **Rekey su revoca**: nuovo master seed; envelope `keys/rekey-{epoca}-
  {install}.env` = master nuovo cifrato X25519 verso la chiave del
  destinatario (chiave X25519 pubblicata nella `view.json`, legata alla
  device key Ed25519 con firma di binding). Gli envelope restano finché il
  destinatario non li consuma (l'heartbeat lo prova).

## Formato segmento (`segmenti.rs`)

File immutabile `NNNNNN.seg` (numerazione per-install, zero-padded):

```
header CBOR (in chiaro):
  { magic: "PAPORD1", vault: "vlt-...", install: "dev_....",
    epoca_vsk: 3, seq_da: 1201, seq_a: 1450, nonce: <24B> }
payload:
  XChaCha20-Poly1305( zstd( CBOR array di OrditoRecord ) )
  con AAD = header serializzato   // header autenticato, non falsificabile
```

- Rotazione: chiudi il segmento a ~2 MB compressi o dopo 15 min di
  inattività con record pendenti.
- **Pubblicazione atomica**: scrivi `NNNNNN.seg.tmp` → flush → rename.
  Gli scanner ignorano `*.tmp` e i file fuori dalla numerazione contigua.
- Padding: dimensione arrotondata al KiB superiore per attenuare il
  fingerprinting da metadata (mitigazione dichiarata nel threat model).

## Trasporto T1 (`t1.rs`)

Layout come da design v3 (§T1). Comportamento:

- **Publish**: dopo ogni commit con record nuovi (o al massimo ogni 30s),
  il worker T1 serializza la coda `Oplog` non ancora pubblicata nel
  segmento corrente della propria sottocartella.
- **Scan**: polling della cartella ogni 30s + "Sincronizza ora" manuale
  (niente watcher `notify` sulla cartella remota: su share/Dropbox gli
  eventi sono inaffidabili; il polling è il baseline, il watcher resta
  un'ottimizzazione opzionale locale).
- **`view.json` firmata** (una per install, nella propria sottocartella):
  `{ install, pubkey_ed25519, pubkey_x25519, cursori: {install→seq},
  revoche: [...], heartbeat: <ts>, sig }`. Riscritta (write+rename) a ogni
  ciclo di scan: il cursore heartbeat è ciò che sblocca la GC causale.
- **Vista effettiva** = merge deterministico delle `view.json` valide
  (firma ok, install nota o in attesa di approvazione). Un'install nuova
  compare come "dispositivo in attesa" nella UI finché un device già
  fidato non la approva (evento firmato nella propria view).
- **Robustezza**: "conflicted copy" (pattern per provider: ` (1).seg`,
  `.sync-conflict-`) ignorate; segmento corrotto/AAD invalido → quarantena
  del suffisso + log; buchi di numerazione → si attende il re-scan (il
  file-syncer può consegnare fuori ordine), dopo 3 cicli si segnala.
- **Consumo**: i record decifrati passano a `applica_remoto` (F1) — che da
  F2 accetta record Ordito nativi: verifica `vault`, firma, catena
  (`prev`), equivocation; `Hlc::receive` PRIMA dell'apply; poi LWW-guard
  `OrditoApplied`, `OrditoPending` per orfani.

## Snapshot multi-peer (`snapshot.rs`) — design v3 §Compattazione

Directory `snapshot/` nella cartella di trasporto:

```
snapshot/
├─ snap-<hlc_taglio>.seg           # stato completo: zstd(CBOR), cifrato come i segmenti
├─ snap-<hlc_taglio>.manifest      # firmato: vettore copertura, hash dello snap, schema_v
└─ snap-<hlc_taglio>.ack-<install> # controfirma di ogni install che ha verificato
```

- Contenuto dello snap: per ogni entità viva → valori correnti + per ogni
  campo l'**attestazione** `(hash, sig, install)` del record vincente;
  tombstone non ancora GC-abili; sezione **derivati opachi**: blob
  embeddings `{modello, versione, vettori}` (Decisione 2 — cache non
  attestata, fuori dalle attestazioni).
- **Chi lo scrive**: l'install col cursore più avanzato quando il log
  supera una soglia (default: 30gg di record o 50 MB); lock ottimistico via
  nome file (se esiste già uno snap più recente, rinuncia).
- **Controfirma**: ogni install che fa scan verifica lo snap contro il
  proprio log (copertura + attestazioni a campione) e pubblica il proprio
  `.ack-<install>`. Con ack di **tutte le install non-stale** (K=tutte,
  Decisione 6) i segmenti interamente sotto il taglio diventano
  eliminabili; li rimuove chi li possiede, al ciclo successivo.
- Bootstrap device nuovo: ultimo snap con manifest valido + coda dei
  segmenti dopo il taglio; embeddings dal blob se il modello coincide.

## GC causale + stale-out (`gc.rs` esteso)

- Regola tombstone: rimovibile solo se `HLC < min(cursore confermato)` su
  **tutte** le install note non-stale (dai cursori delle `view.json`).
- **Stale-out**: install senza heartbeat da N giorni (default 90,
  `OrditoMeta('stale_giorni')`): esclusa dal calcolo, marcata stale nella
  vista. Preavvisi progressivi sugli altri device a 60 e 85 giorni
  ("Il dispositivo X non si sincronizza da Y giorni; tra Z verrà escluso"),
  con azione "posticipa" (estende una tantum di 30gg) o "rimuovi ora".
- **Rientro di un'install stale**: resync obbligatorio da snapshot. La coda
  d'uscita non pubblicata NON viene scartata né ri-emessa in automatico:
  viene **parcheggiata** in una tabella locale `OrditoStaleQueue`
  (migrazione V017-F2) con conteggio esposto in UI ("N modifiche di questo
  dispositivo non sono mai state sincronizzate"). La **revisione guidata**
  completa (scegli e ri-applica come record nuovi con HLC corrente) arriva
  con la Vista conflitti in F3; in F2 le azioni disponibili sono:
  "riapplica tutto" / "esporta come Markdown" / "scarta" — esplicite, mai
  silenziose (Decisione 6).

## UI F2 (Impostazioni → "Sincronizzazione dispositivi")

- Attivazione: scegli cartella di trasporto → genera master seed → flusso
  recovery phrase bloccante → prima publish.
- Aggiunta device: sul nuovo, "Unisciti a una sincronizzazione esistente"
  → scegli cartella → QR/frase → compare "in attesa" sugli altri →
  approvazione.
- Lista dispositivi: nome/età heartbeat/stato (attivo, in attesa, stale,
  revocato); revoca con doppio avviso (rotazione chiavi + "rimuovi anche
  l'accesso alla cartella condivisa: quello non lo facciamo noi").
- Stato sync: ultima publish/scan, dimensione log/cartella, snapshot e ack.
- Retention: passa automaticamente da "locale 90gg" a "regime causale"
  quando compaiono peer (transizione già predisposta in F1 `gc.rs`).

## Sequenza delle PR

| PR | Contenuto | Dipende da | Stima |
|---|---|---|---|
| F2-PR1 | `vsk.rs`: master seed, epoche, recovery phrase BIP39, envelope X25519; estensione `keys.rs` | F1 | media |
| F2-PR2 | `segmenti.rs`: formato, cifratura AAD, rotazione, padding; fuzz (troncamento, bit-flip, epoca mancante) | PR1 | media |
| F2-PR3 | `t1.rs`: publish/scan, `view.json`, vista effettiva, quarantene; `applica_remoto` esteso ai record nativi (firma/catena/equivocation) | PR2 | alta |
| F2-PR4 | property test di convergenza 2-3 install SU CARTELLA reale (tmpdir): permutazioni di publish/scan, partizioni simulate | PR3 | media |
| F2-PR5 | `snapshot.rs`: snap+manifest+ack, attestazioni, blob embeddings, bootstrap da snapshot | PR3 | alta |
| F2-PR6 | `gc.rs` causale + stale-out + preavvisi + `OrditoStaleQueue` (V017-F2) | PR5 | media |
| F2-PR7 | UI: attivazione, pairing QR/frase, lista dispositivi, revoca, stato | PR3 | media-alta |
| F2-PR8 | threat model metadata in docs + padding verificato + checklist per la review crittografica esterna | PR5 | bassa |

## Criteri di completamento F2

- [ ] Due install su cartella condivisa (Syncthing E Dropbox provati a
      mano) convergono da editing concorrente offline, LWW per campo.
- [ ] Device nuovo: bootstrap da snapshot < 60s su vault 10k prompt, con
      ricerca semantica attiva subito (blob embeddings).
- [ ] Revoca: il revocato non decifra i segmenti post-rotazione; un device
      offline-da-1-mese risale gli envelope e rientra.
- [ ] Recovery: vault ricostruito da snapshot + recovery phrase su macchina
      nuova senza alcun device precedente.
- [ ] GC: log e cartella non crescono oltre soglia con 3 install attive;
      un'install stale non blocca oltre i 90gg; nessuna resurrezione nei
      property test con tombstone GC-ati.
- [ ] Nessun contenuto in chiaro nella cartella (audit: solo header AAD,
      `view.json` e manifest — campi censiti nel threat model).
- [ ] Coverage `ordito/` ≥80% mantenuta; **review crittografica esterna
      pianificata** prima di dichiarare T1 stabile (gate di rilascio, non
      di merge).

## Rischi operativi

- **File-syncer lenti/parziali**: rename atomico + numerazione contigua +
  AAD coprono consegne parziali e fuori ordine; "cartella mai aggiornata"
  (client di sync spento) è indistinguibile da peer assente — corretto:
  valgono i preavvisi stale.
- **Case-insensitive filesystem** (Windows/macOS): nomi file solo
  lowercase-hex, nessuna collisione possibile.
- **Orologio del NAS irrilevante**: nessuna decisione usa mtime dei file;
  solo HLC interni e heartbeat firmati.
- **Due install che scrivono snapshot insieme**: lock ottimistico sul nome;
  al peggio due snap validi — vince il taglio più alto, l'altro decade
  senza ack.
