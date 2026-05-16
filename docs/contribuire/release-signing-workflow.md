# Release Signing Workflow — Procedura Maintainer

> Procedura operativa step-by-step per firmare Authenticode una release
> di Prompt a Porter dopo che la CI ha generato gli asset unsigned.
>
> Razionale architetturale (perché pre-signing locale e non in CI):
> vedi [`../architettura/decisioni/authenticode-signing.md`](../architettura/decisioni/authenticode-signing.md)
> §"Strategia pre-signing locale".

## TL;DR

```powershell
# 1. push tag (CI parte da sola)
git tag v0.9.0
git push origin v0.9.0

# 2. attendi che la CI completi (build + draft release)
gh run watch --repo robertomarchioro/prompt-a-porter

# 3. apri SimplySign Desktop, fai login (user + pwd + TOTP)

# 4. firma Authenticode + re-sign Ed25519 Updater + ricarica + pubblica
#    Assume env CERTUM_CERT_THUMBPRINT + TAURI_UPDATER_PRIVATE_KEY_PATH gia' settate.
.\scripts\sign-release.ps1 -Tag v0.9.0 -Publish
```

Senza re-signing Updater (sconsigliato — Tauri Updater rifiuterà ogni update da questa release):
```powershell
# Senza env TAURI_UPDATER_PRIVATE_KEY_PATH lo script chiede conferma
.\scripts\sign-release.ps1 -Tag v0.9.0 -Publish
```

## Prerequisiti — una tantum

### Sulla workstation Windows usata per il signing

Può essere la macchina di sviluppo o una workstation Windows dedicata
(es. una VM Windows, un PC secondario, un laptop Windows). Lo script
non ha dipendenze dal codice sorgente di Prompt a Porter — gli basta
gh CLI + signtool + SimplySign Desktop.

| Software | Come installarlo |
|---|---|
| **Windows 10 (1803+) o Windows 11** | — |
| **Windows SDK** (per `signtool.exe`) | [Download Microsoft](https://developer.microsoft.com/windows/downloads/windows-sdk/). Basta il componente "Windows SDK Signing Tools" |
| **SimplySign Desktop** (Certum) | `winget install --id Certum.SmartSignSimplySignDesktop` oppure download da [pagina Certum](https://www.certum.eu/en/cert_expertise_signature_cards_drivers_simplysign_desktop/) |
| **GitHub CLI** (`gh`) | `winget install --id GitHub.cli` oppure [cli.github.com](https://cli.github.com/) |
| **PowerShell 5.1+** o **PowerShell 7+** | Preinstallato su Windows 10/11 |
| **Tauri CLI** (per re-signing Ed25519 Updater) | `cargo install tauri-cli --version "^2"` oppure `pnpm i -g @tauri-apps/cli`. Necessario per **opzione B** (re-signing post-Authenticode). Senza, lo script chiede conferma e procede ma Tauri Updater rifiuta gli update da quella release. |

### Setup credenziali una tantum

1. **gh CLI auth**:
   ```powershell
   gh auth login
   # scegli GitHub.com -> HTTPS -> browser auth
   ```
2. **Cert Certum thumbprint**: una volta loggato in SimplySign Desktop,
   recupera il thumbprint del certificato:
   ```powershell
   Get-ChildItem Cert:\CurrentUser\My |
     Where-Object { $_.Subject -like "*Roberto Marchioro*" } |
     Format-List Thumbprint, Subject, NotAfter
   ```
   Annota il thumbprint (40 caratteri hex). Per evitare di reinserirlo
   ogni volta, esportalo come env var permanente:
   ```powershell
   [Environment]::SetEnvironmentVariable('CERTUM_CERT_THUMBPRINT', '<TUO_THUMB>', 'User')
   # poi riapri PowerShell perche' la nuova env var sia visibile
   ```
3. **Tauri Updater private key** (per re-signing Ed25519 - opzione B):
   copia la chiave privata `pap-updater.key` dalla tua macchina di
   sviluppo principale (o dal tuo password manager) sulla workstation
   di signing. Esempio: `C:\Users\<user>\.tauri\pap-updater.key`.
   Esportala come env var permanente:
   ```powershell
   [Environment]::SetEnvironmentVariable('TAURI_UPDATER_PRIVATE_KEY_PATH', 'C:\Users\<user>\.tauri\pap-updater.key', 'User')
   # se la chiave ha password, settala con:
   [Environment]::SetEnvironmentVariable('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', '<password>', 'User')
   # riapri PowerShell.
   ```
   ⚠️ **Mai committare questa chiave**. Vedi `setup-tauri-updater-keys.md`
   §"Backup chiave privata" per la gestione long-term del segreto.

## Procedura per ogni release

### Step 1 — Tag e push (su qualunque macchina)

```bash
# Linux/Mac/Windows con git installato
git checkout main
git pull
git tag v0.9.0
git push origin v0.9.0
```

Il push del tag triggera `.github/workflows/release.yml` su GitHub Actions.

### Step 2 — Attendi build CI

La CI:
- Builda Tauri (~10-15 min) per Windows
- Genera `.exe` portable + `.msi` + `.exe` NSIS setup (se abilitati nel
  matrix; in v0.8.x è temporaneamente solo portable)
- Genera `latest.json` + `.sig` Ed25519 (Tauri Updater signing, **separato
  da Authenticode**, fatto in CI dato che la chiave Ed25519 è in secret)
- Crea una **release draft** su GitHub con tutti gli asset uppati

Monitora:
```bash
gh run watch --repo robertomarchioro/prompt-a-porter
# oppure visualizza nel browser:
gh run list --repo robertomarchioro/prompt-a-porter --limit 3
```

A fine build, verifica che la release draft esista:
```bash
gh release view v0.9.0 --repo robertomarchioro/prompt-a-porter
```

### Step 3 — Login SimplySign Desktop

**Sulla workstation Windows**:

1. Avvia SimplySign Desktop (icona system tray oppure menu Start)
2. Click "Login"
3. Inserisci:
   - Username (email Certum)
   - Password account
   - Codice TOTP (6 cifre dall'app smartphone Certum/Google Authenticator)
4. Una volta loggato, SimplySign Desktop monta il certificato come
   virtual smart card in `Cert:\CurrentUser\My`.

⚠️ **La sessione SimplySign dura max 2 ore**. Se passi più di 2h fra
login e signing, devi rifare il login. Lo script `sign-release.ps1`
verifica all'avvio che il cert sia visibile e fallisce subito con un
messaggio chiaro se non lo è.

### Step 4 — Esegui sign-release.ps1

Dal repo Prompt a Porter clonato sulla workstation Windows:

```powershell
cd path\to\prompt-a-porter

# Variante "safe" (lascia draft, ti da' chance di review prima di pubblicare).
# Assume env TAURI_UPDATER_PRIVATE_KEY_PATH settata -> re-signing Updater attivo.
.\scripts\sign-release.ps1 -Tag v0.9.0

# Variante "all-in-one" (firma + ricarica + pubblica come Latest)
.\scripts\sign-release.ps1 -Tag v0.9.0 -Publish

# Override esplicito path chiave Updater (se non hai usato env var):
.\scripts\sign-release.ps1 -Tag v0.9.0 -UpdaterPrivKey C:\Users\<user>\.tauri\pap-updater.key -Publish
```

Lo script:
1. Verifica gh CLI, signtool, cert disponibile, release esiste, (se re-signing attivo) tauri CLI + chiave Updater
2. Crea workdir temp `%TEMP%\pap-sign-v0.9.0\`
3. Scarica asset firmabili (`.exe`, `.msi`, `*portable*.zip`, + `.sig` e `latest.json` se re-signing attivo)
4. Estrae il `.zip` portable per firmare l'`.exe` interno
5. Firma tutti i file con `signtool sign /sha1 <thumb> /tr http://time.certum.pl /td sha256 /fd sha256 /a`
6. Verifica le firme con `signtool verify /pa /v`
7. **(Opzione B, se `-UpdaterPrivKey` o env var presente)** Re-firma Ed25519 i target Updater (setup.exe NSIS + .msi) con `tauri signer sign`, ricomputa `latest.json` con le nuove signature + `pub_date` corrente. Senza questo step, Tauri Updater rifiuta gli update con `signature mismatch` perché i `.sig` di CI sono calcolati sui binari unsigned.
8. Re-zippa il portable con `.exe` firmato dentro
9. Re-uploada gli asset firmati (+ `.sig` + `latest.json` se re-signing attivo) con `gh release upload --clobber`
10. (Opzionale, se `-Publish`) promuove la release da draft a Latest
11. Cleanup workdir (a meno che `-KeepWorkDir`)

### Step 5 — (Se non hai usato `-Publish`) review e pubblica

Apri la release draft nel browser:
```bash
gh release view v0.9.0 --web
```

Controlla:
- [ ] Il file portable `.zip` ha la dimensione corretta (~70-80 MB)
- [ ] Scarica il `.zip`, estrai, click destro su `Prompt a Porter.exe` →
      Proprietà → Tab "Firme digitali" → verifica firma valida con CA
      Certum
- [ ] (Opzionale) doppio click sull'`.exe` e verifica che la finestra
      SmartScreen menzioni "Open Source Developer, Roberto Marchioro"
      come publisher

Tutto OK → pubblica:
```bash
gh release edit v0.9.0 --draft=false --latest
```

## Troubleshooting

### "Cert non trovato"

Lo script fallisce con `[FAIL] Certificato <thumb> NON trovato in Cert:\CurrentUser\My`.

**Causa**: SimplySign Desktop non è logged-in oppure la sessione è
scaduta (>2h).

**Fix**: rifai login in SimplySign Desktop (Step 3) e rilancia lo
script. Se l'errore persiste anche dopo login, verifica il thumbprint:
```powershell
Get-ChildItem Cert:\CurrentUser\My | Format-Table Thumbprint, Subject
```
Confronta con la env var `CERTUM_CERT_THUMBPRINT`.

### "signtool.exe non trovato"

**Causa**: Windows SDK non installato o installato senza i Signing Tools.

**Fix**: scarica Windows SDK e durante l'installazione assicurati che
"Windows SDK Signing Tools for Desktop Apps" sia selezionato.

### "gh CLI non autenticato"

**Causa**: prima volta su questa workstation, oppure token gh scaduto.

**Fix**:
```powershell
gh auth login
```

### "Release <tag> non trovata"

**Causa**: tag pushato ma CI non ha ancora finito, o build CI fallita.

**Fix**:
```bash
gh run list --repo robertomarchioro/prompt-a-porter --limit 3
gh run view <run-id>
```
Se CI è fallita, fixa l'errore prima di firmare. Se è in corso, aspetta.

### "Signing FALLITO ... signtool exit ..."

**Causa**: tipicamente errore di comunicazione con virtual smart card
(sessione SimplySign scaduta a metà script) oppure cert revocato.

**Fix**:
1. Verifica che SimplySign Desktop sia ancora logged-in
2. Riapri SimplySign Desktop, login, ri-esegui lo script
3. Se persiste: verifica scadenza cert (`Get-ChildItem Cert:...`) e
   stato cert su [https://simplysign.certum.eu](https://simplysign.certum.eu)

### Verificare manualmente la firma su un file scaricato

```powershell
$signtool = (Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
             Sort-Object -Property FullName -Descending |
             Select-Object -First 1).FullName
& $signtool verify /pa /v "Prompt a Porter.exe"
```

Output atteso: `Successfully verified: Prompt a Porter.exe`.

## Limiti noti

- **Manualità**: ogni release richiede un intervento manuale (~5 min
  attivi + ~15 min wait CI). Non è automatizzabile finché Certum non
  rilascerà un'API headless per SimplySign Cloud.
- **Dipendenza dalla workstation**: serve sempre una macchina Windows
  con SimplySign Desktop e il cert. Se il maintainer è in viaggio senza
  accesso alla workstation, la release deve aspettare (oppure il
  maintainer porta una VM Windows con sé).
- **Finestra 2h**: se per qualche ragione lo script impiega più di 2h
  (network lentissimo, tante release file gigantesche), la sessione
  SimplySign può scadere a metà. Lo script verifica all'avvio ma non
  re-verifica fra step.

## Manutenzione di questo documento

Aggiornare quando:
- Cambia la procedura SimplySign Desktop (UI Certum cambia menu)
- Cambia `sign-release.ps1` (nuovi parametri, nuovi step)
- Si aggiungono nuovi target (es. riabilitiamo macOS / Linux signing)
- Il cert scade e si rinnova (aggiornare thumbprint e link Certum)

## Vedi anche

- ADR signing: [`../architettura/decisioni/authenticode-signing.md`](../architettura/decisioni/authenticode-signing.md)
- Setup chiavi Tauri Updater: [`./setup-tauri-updater-keys.md`](./setup-tauri-updater-keys.md)
- Workflow CI release: [`../../.github/workflows/release.yml`](../../.github/workflows/release.yml)
- Doc utente auto-update: [`../utente/auto-update.md`](../utente/auto-update.md)
