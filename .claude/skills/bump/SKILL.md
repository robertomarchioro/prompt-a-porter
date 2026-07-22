---
name: bump
description: Prepara la release di Prompt à Porter fino alla draft — bump versione nei 4 file, voce CHANGELOG, commit, tag annotato, verifica che release.yml parta. Usala quando l'utente dice "bump vX.Y.Z" o chiede di preparare una release. Con --solo-voci prepara solo la bozza delle voci changelog senza toccare file.
---

# /bump — Release draft di Prompt à Porter

Invocazione: `/bump vX.Y.Z "Titolo release"`. La versione è obbligatoria; se il titolo
manca, proponilo a partire dai cambi raccolti al passo 1 e chiedi conferma.
Variante `/bump --solo-voci`: esegui solo il passo 1 e presenta la bozza della sezione
changelog in chat, senza toccare alcun file.

Il bump NON porta codice: solo versione + changelog. Tutto il lavoro della release deve
essere GIÀ mergiato su main via PR.

## Precondizioni (verificarle, non assumerle)

1. `git checkout main && git pull --ff-only` — working tree pulito (`git status`).
2. La versione richiesta è la successiva coerente con l'ultimo tag
   (`git tag --sort=-creatordate | head -1`) e con l'heading in testa a `CHANGELOG.md`.
3. Se in `CHANGELOG.md` esiste un heading di versione **mai taggata**, consolidarlo
   nella nuova versione prima di procedere (precedente: v0.8.37 consolidata in v0.8.38,
   commit `17f85ae`). Una versione nel changelog deve sempre corrispondere a un tag.
4. Mai taggare un commit non ancora su main
   (anti-pattern documentato in `docs/contribuire/ci-workflows.md`).

## Passi

### 1. Raccogli i cambi

```bash
git log $(git describe --tags --abbrev=0)..HEAD --oneline --no-merges
```

Recupera i numeri di issue chiuse dalle PR mergiate. Raggruppa nelle categorie del
changelog: **Novità / Sicurezza / Fix / Manutenzione**.

### 2. Bump versione nei 4 file

La versione nei file è `X.Y.Z` **senza** prefisso `v`:

| File | Campo |
|---|---|
| `apps/client/package.json` | `"version"` |
| `apps/client/src-tauri/tauri.conf.json` | `"version"` |
| `apps/client/src-tauri/Cargo.toml` | `[package] version` |
| `apps/client/src-tauri/Cargo.lock` | entry `[[package]] name = "pap"` (a mano o `cargo update -p pap`) |

Poi verifica la coerenza: la nuova versione deve comparire in tutti e 4 i file
(`grep -rn "X.Y.Z" <i 4 file>`). L'allineamento del `Cargo.lock` non è cosmetico:
la CI builda con `--locked` e fallisce se non combacia col `Cargo.toml`.

### 3. Voce CHANGELOG

Nuova sezione **in testa** a `CHANGELOG.md` (subito sotto il titolo del file),
formato esatto delle release precedenti:

```markdown
## vX.Y.Z — Titolo (YYYY-MM-DD)

> Una-due frasi di contesto: cosa porta la release e per chi.

### Novità

- **Titolo voce** (#nnn): descrizione utente-centrica in italiano.

### Sicurezza

### Fix

### Manutenzione
```

Solo le sezioni non vuote, in quest'ordine. Le voci sono scritte per l'utente finale
(cosa cambia per lui), non per lo sviluppatore; i riferimenti issue/PR tra parentesi.

### 4. Sanity locale

Prima del commit, in `apps/client`: `pnpm run lint && pnpm run test && pnpm run build:frontend`
tutti verdi.

### 5. Commit e push su main

Commit direttamente su main (il bump non passa da PR):

```bash
git add -A
git commit -m "chore(release): bump vX.Y.Z — Titolo" -m "<riassunto breve dei cambi principali>"
git push
```

**Attendi il client-build verde su main** prima di taggare
(`gh run watch` sul run appena partito): è il gate `--locked` + lint/test/build.

### 6. Tag

Tag annotato con lo stesso formato dei precedenti (prima riga = titolo, poi bullet;
la firma avviene automaticamente dalla config git):

```bash
git tag -a vX.Y.Z -m "vX.Y.Z — Titolo" -m "- Voce principale 1 (#nnn)
- Voce principale 2 (#nnn)"
git push origin vX.Y.Z
```

### 7. Verifica la build di release

```bash
gh run list --workflow=release.yml --limit 1
```

Il push del tag avvia `release.yml`: build matrix 3 OS (~15–20 min) che produce la
**release DRAFT**. Attendi l'esito e controlla la draft: asset attesi per piattaforma
+ `latest.json`. Se la build fallisce, indaga con `gh run view <id> --log-failed`.

### 8. STOP — riporta all'utente

Link alla draft, esito build, e il promemoria dei passi manuali (sotto).

## Confine — cosa la skill NON fa

- **NON firma** Authenticode e **NON pubblica** la release / imposta Latest: sono passi
  manuali di Roberto sul box di firma.
- Promemoria per il box firma: `git pull` nel checkout del box **prima** di firmare
  (gotcha #291), poi `scripts/sign-release.ps1` (passphrase via DPAPI; lo script
  preserva le entry Linux in `latest.json`).
- **Mai** creare heading o voci changelog fuori da questo processo: se dei fix sono
  mergiati ma la release non è decisa, le voci si preparano con `/bump --solo-voci`
  e restano in chat, non nel file.
