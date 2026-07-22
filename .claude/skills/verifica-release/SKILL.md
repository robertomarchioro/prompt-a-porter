---
name: verifica-release
description: Verifica post-pubblicazione di una release firmata di Prompt à Porter — release Latest, latest.json coerente con gli asset ri-firmati, firma Ed25519 dell'updater valida, Authenticode embedded. Usala quando l'utente dice che ha pubblicato la release come Latest ("pubblicata, fai la verifica") dopo un /bump.
---

# /verifica-release — verifica post-pubblicazione

Invocazione: `/verifica-release vX.Y.Z`. Si esegue **dopo** che l'utente ha firmato sul
box Windows e promosso la release da draft a **Latest** (i passi manuali fuori dal
perimetro di `/bump`).

## Perché esiste

Dopo la firma, gli asset Windows e `latest.json` vengono **ri-generati e ri-caricati**
dallo script del box (`sign-release.ps1`): è il punto della pipeline dove una svista
rompe l'auto-update per tutti (gotcha storico #291, notes sbagliate perché il box non
aveva fatto `git pull`). La verifica controlla il risultato finale pubblicato, non il
processo.

## Esecuzione

Un solo comando, dalla radice del repo:

```bash
python3 .claude/skills/verifica-release/verifica.py vX.Y.Z
```

Richiede `gh` autenticato e python3 con `cryptography` (già presente sul box di
sviluppo; `minisign`/`osslsigncode` NON servono). Exit 0 = tutto ok.

Lo script controlla:

1. **Stato** — la release non è draft ed è la **Latest** del repo (quella che
   l'endpoint dell'updater serve ai client).
2. **`latest.json`** — `version` corretta; per ogni piattaforma il campo `signature`
   è **identico** al file `.sig` pubblicato (incluso quello Windows rigenerato dopo
   la ri-firma); tutte le `url` puntano al tag; `notes` presenti e riferite al tag.
3. **Firma updater Ed25519 (minisign)** — verifica crittografica reale della firma
   del `setup.exe` ri-firmato contro la pubkey in `tauri.conf.json`
   (payload BLAKE2b-512 prehashed + global signature sul trusted comment).
4. **Authenticode** — il PE del setup contiene la firma embedded
   (security directory non vuota).

## Interpretare l'esito

- **TUTTO OK** → registra l'esito: scrivi/aggiorna la memoria `release_vX_Y_Z`
  segnando la release come «Latest ✅ verificata» e riporta all'utente il riepilogo
  dei check.
- **FAIL su una signature o sulla firma Ed25519** → l'auto-update è a rischio per
  tutti i client: segnala SUBITO all'utente, non archiviare. Causa tipica: asset
  ri-firmato ma `.sig`/`latest.json` non ri-caricati, o box non allineato
  (`git pull` mancante prima della firma).
- **FAIL su notes/URL** → gotcha #291: il box ha firmato da un checkout vecchio.

## Limiti dichiarati

- L'Authenticode è verificato solo come **presenza** della firma embedded nel PE: la
  validazione completa della catena Certum si può fare solo su Windows
  (`Get-AuthenticodeSignature`). Storicamente è il comportamento atteso.
- La verifica Ed25519 copre il `setup.exe` (l'asset ri-firmato, il più critico);
  per gli altri asset ci si affida all'uguaglianza `signature` ↔ `.sig` del punto 2,
  dato che quei file non vengono toccati dalla ri-firma.
