# Firma e notarizzazione macOS

> **Stato**: procedura eseguita e validata il 2026-07-08/09 (tag di prova
> `v0.8.35-mactest`: firma Developer ID + notarizzazione **Accepted**).
> A differenza di Windows (Authenticode pre-signing locale, v.
> [`../contribuire/release-signing-workflow.md`](../contribuire/release-signing-workflow.md)),
> la firma macOS avviene **interamente in CI** dentro `release.yml`:
> il box Windows di firma non tocca gli artifact darwin (`sign-release.ps1`
> è platform-generico e lascia intatte le loro firme Ed25519 CI).

## Cosa produce la pipeline

Per ogni tag `v*`, il job `macos-14` della matrix di `release.yml` builda
`--target universal-apple-darwin --bundles app,dmg` (binario **universale**
arm64+x86_64, funziona sia su Apple Silicon sia su Mac Intel) e, con i
secret `APPLE_*` presenti:

1. importa il certificato nel keychain effimero del runner;
2. firma l'app con l'identity **Developer ID Application** (codesign);
3. invia l'app al servizio notarile Apple e attende il responso
   (`Notarizing Finished with status Accepted` nei log — è il verdetto
   autoritativo: Gatekeeper aprirà l'app senza avvisi);
4. carica sulla release draft il `.dmg` e l'`.app.tar.gz` universali
   (+ `.sig` Ed25519 per l'updater) e le entry `darwin-aarch64` **e**
   `darwin-x86_64` in `latest.json` (entrambe verso lo stesso artifact,
   così l'auto-update copre Intel e Apple Silicon).

> **Storia**: fino a v0.8.35 la pipeline compilava solo `aarch64`
> (Apple Silicon); su Mac Intel l'app veniva rifiutata con "non è
> supportato su questo Mac". Dal fix del 2026-07-11 (v0.8.36) il target è
> universale.

Tempi: la build è ~10 min; la coda notarile Apple è variabile (dai 5 minuti
alle ore — il primo run ha impiegato ~2h30). Non è un errore: il job aspetta.

## Setup una tantum (eseguito il 2026-07-08)

Prerequisito: iscrizione **Apple Developer Program** attiva (account
individuale ⇒ sei Account Holder, ruolo necessario per i cert Developer ID).
Il **Team ID** (10 caratteri) è in Membership details.

Tutto senza Mac, da Linux con OpenSSL:

```bash
# 1. Chiave privata + CSR (la chiave NON lascia la macchina)
mkdir -p ~/.tauri/apple && chmod 700 ~/.tauri/apple && cd ~/.tauri/apple
openssl genrsa -out pap-developer-id.key 2048 && chmod 600 pap-developer-id.key
openssl req -new -key pap-developer-id.key -out pap-developer-id.csr \
  -subj "/emailAddress=<email>/CN=<Nome Cognome>/C=IT"

# 2. Portale: Certificates → "+" → sezione Software →
#    "Developer ID Application" (G2 Sub-CA) → upload CSR → download .cer
#    (NON "Developer ID Installer" né "Apple Distribution")

# 3. p12 con catena completa.
#    ⚠️ GOTCHA CRITICO: con OpenSSL 3.x serve `-legacy`, altrimenti il
#    `security import` sul runner macOS fallisce con l'errore FUORVIANTE
#    "MAC verification failed during PKCS12 import (wrong password?)" —
#    non è la password, sono i cifrari di default (AES/SHA-256) che il
#    keychain non digerisce; -legacy usa SHA1-3DES/RC2.
openssl x509 -inform DER -in developerID_application.cer -out cert.pem
curl -sSf -o DeveloperIDG2CA.cer https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
openssl x509 -inform DER -in DeveloperIDG2CA.cer -out intermediate.pem
openssl rand -base64 24 > p12-passphrase.txt && chmod 600 p12-passphrase.txt
openssl pkcs12 -export -legacy -inkey pap-developer-id.key -in cert.pem \
  -certfile intermediate.pem -out pap-developer-id.p12 -passout file:p12-passphrase.txt
base64 -w0 pap-developer-id.p12 > pap-developer-id.p12.base64

# 4. App-specific password per la notarizzazione:
#    account.apple.com → Accesso e sicurezza → Password specifiche per le
#    app → nuova (etichetta "pap-notarize"). NON è la password dell'account.
```

## I 6 secret GitHub

| Secret | Valore |
|---|---|
| `APPLE_CERTIFICATE` | contenuto di `pap-developer-id.p12.base64` |
| `APPLE_CERTIFICATE_PASSWORD` | contenuto di `p12-passphrase.txt` |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: <NOME> (<TEAMID>)` — esatto dal subject CN del cert (`openssl x509 -in cert.pem -noout -subject`) |
| `APPLE_ID` | email dell'account Apple |
| `APPLE_PASSWORD` | l'app-specific password (mai la password dell'account) |
| `APPLE_TEAM_ID` | il Team ID (compare anche come OU nel cert) |

Caricamento senza esporre valori in chat/shell history:
`gh secret set NOME < file` per i primi due; gli altri via prompt
interattivo `gh secret set NOME` (incolla nascosto).

## Custodia e scadenze

- `~/.tauri/apple/` contiene chiave privata, cert, p12 e passphrase:
  **backup nello stesso luogo sicuro della chiave updater `pap-updater`**.
  I secret GitHub sono una copia cifrata lato GitHub, non un backup.
- Il certificato **scade il 2031-07-09** (5 anni). Rinnovo: stessa
  procedura dal punto 1 (o riuso della stessa chiave/CSR); ricaricare
  `APPLE_CERTIFICATE`. L'iscrizione Developer Program è annuale: se
  scade, il cert resta valido ma la notarizzazione smette di funzionare.
- **Revoca/rotazione**: dal portale si revoca il cert; le app già
  notarizzate restano valide (il ticket di notarizzazione sopravvive alla
  revoca del cert, per design Apple).

## Troubleshooting

| Sintomo | Causa reale |
|---|---|
| `MAC verification failed during PKCS12 import (wrong password?)` | p12 esportato con OpenSSL 3.x senza `-legacy` (v. gotcha sopra). Rigenerare con `-legacy`, aggiornare `APPLE_CERTIFICATE`, `gh run rerun <id> --failed` (i secret vengono riletti al rerun). |
| Notarizzazione lenta (ore) | Coda Apple, non un bug. Il job attende; timeout runner GH = 6h, mai raggiunto finora. |
| `Invalid` dal notary | Leggere i dettagli nell'output tauri-action (o `xcrun notarytool log <id>` da un Mac); cause tipiche: binario non firmato dentro il bundle, entitlements mancanti. |
| Verificare la compilazione senza firmare | workflow `macos-smoke.yml` (dispatch-only): builda unsigned senza toccare release. |
