# Setup chiavi Tauri Updater (procedura una-tantum)

> Guida operativa per il maintainer principale: generare la coppia di chiavi Ed25519 usata da Tauri Updater per firmare `latest.json`, configurare i GitHub Secrets, sostituire la chiave pubblica in `tauri.conf.json`.
>
> Procedura da eseguire **una sola volta** alla nascita del progetto (o al rinnovo/rotation della chiave). Operazione **locale sul computer del maintainer**, non in CI.

## Premessa

La firma del Tauri Updater è distinta dalla firma Authenticode dei binari Windows:

| | Authenticode (`.exe`/`.msi`) | Tauri Updater (`latest.json`) |
|---|---|---|
| Chiave | RSA, cloud Certum SimplySign | Ed25519, generata localmente |
| Provider | CA pubblica | Tu, custodita personalmente |
| Lock-in se persa | Basso (rinnovo cert nuovo) | **Alto** (utenti devono fare update manuale) |

Vedi [`docs/architettura/decisioni/authenticode-signing.md`](../architettura/decisioni/authenticode-signing.md) per il razionale architetturale completo.

⚠️ **La chiave privata Ed25519 è il segreto a lungo termine più importante del progetto.** Se la perdi e devi rigenerarla, **tutti gli utenti con la versione vecchia non potranno più ricevere update automatici** finché non installano manualmente la nuova versione. Vedi §"Recovery key persa" in fondo.

## Prerequisiti

- Repo clonato in locale.
- `pnpm` installato (`corepack enable && corepack prepare pnpm@latest --activate`).
- Tauri CLI disponibile via `pnpm tauri` nel progetto `apps/client/`.
- Password manager attivo (1Password, Bitwarden, KeePass, ...): la chiave privata + password vanno custoditi lì, non solo sul disco.

## Step 1 — Genera la coppia di chiavi Ed25519

Dal terminale:

```bash
cd apps/client
pnpm tauri signer generate -w ~/.tauri/pap-updater.key
```

Cosa succede:
- Crea la cartella `~/.tauri/` se non esiste.
- Ti chiede una **password** opzionale per criptare la chiave privata.
  - **Raccomandato**: imposta una password robusta (servirà come Secret GitHub).
  - **Se non vuoi**: premi Invio due volte (password vuota, comunque accettata).
- Output: 2 file
  - `~/.tauri/pap-updater.key` → chiave privata (NON committare mai)
  - `~/.tauri/pap-updater.key.pub` → chiave pubblica (committata nel repo)

Verifica:

```bash
ls -la ~/.tauri/
# pap-updater.key       (~200 bytes)
# pap-updater.key.pub   (~150 bytes)
```

## Step 2 — Backup IMMEDIATO

⚠️ Prima di proseguire, salva la chiave privata in luogo sicuro.

```bash
cat ~/.tauri/pap-updater.key
```

Copia tutto l'output (incluse le righe header tipo `untrusted comment:...`) e incollalo in:

1. **Password manager** come "Secure Note" titolo `"PaP Updater Private Key"`.
2. **Password Ed25519** che hai impostato in Step 1 → "Secure Note" separato, titolo `"PaP Updater Key Password"`.
3. (Opzionale) backup criptato su altro disco fisico.

Non basta avere `~/.tauri/pap-updater.key`: se il disco si rompe, il setup è compromesso.

## Step 3 — Configura i 2 GitHub Secrets

Vai su `github.com/robertomarchioro/prompt-a-porter` → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**.

### Secret 1: `TAURI_SIGNING_PRIVATE_KEY`

| Campo | Valore |
|---|---|
| **Name** | `TAURI_SIGNING_PRIVATE_KEY` |
| **Value** | Contenuto INTEGRALE di `~/.tauri/pap-updater.key`, incluse le righe `untrusted comment:` e tutto il blob base64. Rispetta i newline. |

Comando per copiare:

```bash
cat ~/.tauri/pap-updater.key | xclip -selection clipboard       # Linux
cat ~/.tauri/pap-updater.key | pbcopy                            # macOS
type %USERPROFILE%\.tauri\pap-updater.key | clip                 # Windows
```

Poi paste nel form GitHub.

### Secret 2: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

| Campo | Valore |
|---|---|
| **Name** | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |
| **Value** | La password scelta in Step 1 |

⚠️ Se hai scelto **password vuota** in Step 1: **crea comunque il secret** con valore vuoto (stringa "" salvata). NON omettere il secret, altrimenti tauri-action fallisce con "missing env".

## Step 4 — Sostituisci il placeholder in `tauri.conf.json`

### 4a. Stampa la chiave pubblica

```bash
cat ~/.tauri/pap-updater.key.pub
```

Output esempio:

```
untrusted comment: minisign public key: 1FD4AC8BE9649417
RWQXlGTpi6zUH7doydceNVzUSEPOdeLElpCLksc/jH7SPCKtWmb9E/RR
```

Il file ha 2 righe:
- Riga 1: commento (`untrusted comment:...`)
- Riga 2: **la chiave base64 vera** — è quella che ti serve

Estrai solo la seconda riga:

```bash
tail -n 1 ~/.tauri/pap-updater.key.pub
```

Output (esempio):

```
RWQXlGTpi6zUH7doydceNVzUSEPOdeLElpCLksc/jH7SPCKtWmb9E/RR
```

### 4b. Sostituisci nel file

Apri `apps/client/src-tauri/tauri.conf.json` e trova la riga:

```json
"pubkey": "PLACEHOLDER_GENERA_CON_TAURI_SIGNER_GENERATE",
```

Sostituisci con (esempio):

```json
"pubkey": "RWQXlGTpi6zUH7doydceNVzUSEPOdeLElpCLksc/jH7SPCKtWmb9E/RR",
```

### 4c. Crea PR + merge

Il valore di `pubkey` è pubblico per design (verifica firma lato client, non c'è bisogno di tenerla segreta). Si committa normalmente:

```bash
git checkout -b chore/tauri-updater-pubkey
git add apps/client/src-tauri/tauri.conf.json
git commit -m "chore(updater): sostituisci placeholder pubkey con chiave reale Ed25519"
git push -u origin chore/tauri-updater-pubkey
gh pr create --title "chore(updater): sostituisci placeholder pubkey con chiave reale Ed25519" \
             --body "Setup Tauri Updater completato. Vedi docs/contribuire/setup-tauri-updater-keys.md."
```

Merge appena CI verde.

## Verifica fine setup

Una volta che PR di sostituzione è mergiata, hai 2 opzioni per validare:

### Opzione A — Test signing locale (rapido)

```bash
cd apps/client
echo "test" > /tmp/test.txt

# Con password impostata in Step 1
pnpm tauri signer sign -f ~/.tauri/pap-updater.key -p "TUA_PASSWORD" /tmp/test.txt

# Se hai lasciato password vuota
pnpm tauri signer sign -f ~/.tauri/pap-updater.key /tmp/test.txt

# Verifica + cleanup
ls /tmp/test.txt.sig  # se esiste, la chiave funziona
rm /tmp/test.txt /tmp/test.txt.sig
```

⚠️ **Flag corretto è `-f` (file path), NON `-k`** che invece vuole la chiave come stringa base64 inline. Se usi `-k ~/.tauri/...` il parser interpreta il path come base64 e fallisce con `Invalid symbol 46, offset 14` (il punto del path non è base64 valido).

In alternativa, più sicuro per evitare la password nello shell history, usa env vars:

```bash
export TAURI_SIGNING_PRIVATE_KEY_PATH=~/.tauri/pap-updater.key
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="TUA_PASSWORD"
pnpm tauri signer sign /tmp/test.txt
unset TAURI_SIGNING_PRIVATE_KEY_PATH TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

Per riferimento, l'help del comando (Tauri 2):

```
Usage: tauri signer sign [OPTIONS] <FILE>

Options:
  -k, --private-key <PRIVATE_KEY>           Load the private key from a string
                                            [env: TAURI_SIGNING_PRIVATE_KEY=]
  -f, --private-key-path <PRIVATE_KEY_PATH> Load the private key from a file
                                            [env: TAURI_SIGNING_PRIVATE_KEY_PATH=]
  -p, --password <PASSWORD>                 Set private key password when signing
                                            [env: TAURI_SIGNING_PRIVATE_KEY_PASSWORD=]
```

Se il file `.sig` viene generato senza errori, la chiave è valida e funziona col toolchain.

### Opzione B — Tag di test su GitHub (end-to-end)

Conferma la pipeline completa (Authenticode + Tauri Updater):

```bash
git tag v0.8.9-test
git push origin v0.8.9-test
```

Triggera `release.yml`. Nei log job Windows verifica:
- Step **"Build & release Tauri"**: legge `TAURI_SIGNING_PRIVATE_KEY` env, produce `latest.json` + asset `.sig`.
- Step **"Firma Authenticode (Certum SimplySign)"**: firma `.exe`/`.msi`.

Asset attesi nella release draft:
- `latest.json` — manifesto firmato Ed25519
- `Prompt-a-Porter-portable-windows-x64-v0.8.9-test.zip` — portable con binario firmato Authenticode
- `Prompt.a.Porter_0.8.9-test_x64-setup.exe` — NSIS installer firmato Authenticode + `.sig` Ed25519
- `Prompt.a.Porter_0.8.9-test_x64_en-US.msi` — MSI firmato Authenticode + `.sig` Ed25519

Cleanup tag di test dopo verifica:

```bash
git tag -d v0.8.9-test
git push origin :refs/tags/v0.8.9-test
gh release delete v0.8.9-test --yes
```

## Checklist finale

- [ ] **Step 1**: `pnpm tauri signer generate -w ~/.tauri/pap-updater.key` con password robusta
- [ ] **Step 2**: chiave privata + password salvate in password manager
- [ ] **Step 3a**: Secret `TAURI_SIGNING_PRIVATE_KEY` configurato su GitHub
- [ ] **Step 3b**: Secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` configurato (anche se vuoto)
- [ ] **Step 4**: placeholder `PLACEHOLDER_GENERA_CON_TAURI_SIGNER_GENERATE` sostituito + PR mergiata
- [ ] **Verifica** (opzionale): tag `v0.8.9-test` triggerato + asset firmati controllati + tag cleanato

## Recovery — key persa o compromessa

### Scenario 1: chiave privata persa (es. disco rotto, file cancellato per errore)

Se non hai backup nel password manager, l'unica via è generare nuova chiave:

1. Genera nuova coppia con la procedura sopra.
2. Aggiorna `pubkey` in `tauri.conf.json` con la nuova chiave pubblica.
3. Aggiorna i 2 GitHub Secrets.
4. **Pubblica una nuova versione "manuale"**: nella release note avvisa che l'aggiornamento automatico non funzionerà per chi è su versioni precedenti, dovranno scaricare manualmente da GitHub Releases.
5. Da quel punto in avanti, l'updater riprende per chi ha installato la nuova versione.

### Scenario 2: chiave privata compromessa (es. leak su pubblico)

1. **NON pubblicare nuove release con la chiave vecchia** (un attaccante può sostituire `latest.json` con payload malevolo).
2. Genera nuova coppia.
3. Aggiorna `pubkey` + Secrets.
4. Pubblica release "manuale" con avviso esplicito.
5. Annuncia il compromise su canali pubblici (GitHub issue, social).

### Scenario 3: cambio maintainer + handover

Il maintainer attuale può:
- Esportare `~/.tauri/pap-updater.key` (e password) al nuovo maintainer attraverso canale sicuro (es. Signal, PGP-encrypted email).
- Il nuovo maintainer importa la chiave nella sua `~/.tauri/`.
- Il nuovo maintainer aggiorna i GitHub Secrets a suo nome (se il vecchio cede l'admin del repo).
- Niente downtime per gli utenti.

## Manutenzione di questo documento

- Quando la procedura `pnpm tauri signer generate` cambia output: aggiornare §Step 1.
- Quando aggiungiamo varianti (es. cert HSM): linkare nuovo doc parallelo.
- Quando la chiave viene rotata: aggiornare data in cima.

## Riferimenti

- [ADR Authenticode signing](../architettura/decisioni/authenticode-signing.md) — decisione architetturale completa, garanzie sicurezza, alternative scartate
- [Doc utente auto-update](../utente/auto-update.md) — cosa vede l'utente finale
- [Tauri 2 Updater plugin docs](https://v2.tauri.app/plugin/updater/) — riferimento ufficiale
