# Vetrina — pubblicazione repo pubblico

Pipeline per pubblicare uno **snapshot pulito** del repo privato su un repo
GitHub pubblico ("vetrina"), a ogni release. Questa cartella `.vetrina/` e lo
script non vengono pubblicati (sono auto-esclusi).

## Modello

- **Repo privato** (questo): sviluppo, storia completa, roadmap/pianificazione,
  config interna.
- **Repo vetrina** (pubblico): solo prodotto + doc utente + changelog. Un commit
  `Release vX.Y.Z` per pubblicazione → storia pulita scandita dai rilasci.
- Lo script è una **proiezione pura**: copia (file tracciati − esclusioni) →
  gate segreti → snapshot → push. Niente riscrittura di contenuti: la sorgente
  è già "publish-clean".

## File

- `exclude.txt` — path esclusi dalla vetrina (prefissi dir o file esatti).
- `secret-allowlist.txt` — literali noti sicuri ignorati dal gate segreti.
- `../scripts/publish-vetrina.sh` — lo script.

## Uso

```bash
# Anteprima senza pubblicare (build + gate segreti):
DRY_RUN=1 ./scripts/publish-vetrina.sh v1.0.0

# Pubblicazione reale:
VETRINA_REPO=git@github.com:OWNER/REPO.git ./scripts/publish-vetrina.sh v1.0.0
```

Se la label è omessa usa il tag esatto su HEAD. Richiede working tree pulito.
Il gate aborta se trova pattern di segreti reali non presenti in allowlist.

## Da decidere quando si blinda il nome pubblico

- **Module path Go** (`go.mod`) e URL nel README/CONTRIBUTING/`tauri.conf.json`
  puntano ancora a `robertomarchioro/prompt-a-porter`. La proiezione li lascia
  invariati. Se il repo pubblico avrà un path diverso e si vuole `go get`
  remoto + link corretti, fare un **rename una-tantum nella sorgente** (non
  una trasformazione per-run).
- **Endpoint auto-update** (`tauri.conf.json`): vedi strategia release.

## Futuro: CI

Quando lo script è rodato, agganciarlo a un workflow su `release: published`
con un secret `VETRINA_PUSH_TOKEN` (PAT con push sul repo pubblico).
