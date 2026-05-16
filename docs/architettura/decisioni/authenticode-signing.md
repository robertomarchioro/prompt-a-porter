# ADR — Authenticode signing per release Windows

**Stato**: ⚠️ AMENDED 2026-05-16 (approach C adottato) · **Data**: 2026-05-15 · **Branch sub-PR M1**: M1.1 (questo doc)

> **Storico decisione**: l'ADR originale (2026-05-15) adottava l'approccio
> **A — Windows runner CI** con SimplySign Desktop installato sul runner
> + login TOTP automatizzato. Dopo 4 iterazioni di test
> (`v0.8.9-test` → `v0.8.9-test4`) abbiamo confermato che SimplySign
> Desktop richiede interazione GUI per il login — non esiste un CLI
> headless documentato (gli argomenti `/silent /login /user /password
> /totp` non sono supportati nella release 2026 di SimplySign Desktop,
> contrariamente a quanto suggerito da alcune fonti community datate).
> Abbiamo quindi adottato l'**approccio C — pre-signing locale**
> (precedentemente scartato), che è l'unica opzione che funziona oggi
> con un cert Certum SimplySign Cloud.
>
> Vedi nuova §"Strategia pre-signing locale" sotto. Le sezioni
> "Workflow GitHub Actions — snippet pronto" e "Trade-off documentati"
> restano archiviate per riferimento storico (descrivono l'approccio
> A non più in uso).

## Decisione finale

Adottiamo **Certum SimplySign Cloud Code Signing (Open Source variante)** come provider di firma Authenticode per i bundle Windows di Prompt a Porter (`.exe` portable, `.msi`, `.exe` NSIS installer).

Integrazione: **approccio C — pre-signing locale**. La CI produce asset unsigned in release draft; il maintainer firma manualmente da workstation Windows con SimplySign Desktop logged-in tramite lo script `scripts/sign-release.ps1`, poi promuove la release a Latest published.

Il **Tauri Updater signing** (Ed25519 su `latest.json`) resta invece automatizzato in CI tramite secret `TAURI_SIGNING_PRIVATE_KEY` — è una chiave software locale, indipendente dal cert Certum.

## Strategia pre-signing locale (approccio C — corrente)

### Scoperta del blocker

Dopo aver implementato lo step di signing in CI (approccio A, sub-PR
M1.2 originale) e averlo testato su 4 tag consecutivi:

| Tag | Sintomo | Root cause |
|---|---|---|
| `v0.8.9-test` | 404 download installer Certum | URL hardcoded vendor-side cambiato (resolved via WinGet) |
| `v0.8.9-test2` | TOTP exception `Format specifier was invalid` | `[math]::Pow(10,6)` ritorna Double, `D6` non valido su Double (resolved con cast int) |
| `v0.8.9-test3` | `SimplySignDesktop.exe is not recognized` | Path hardcoded errato (resolved con path detection dinamica via GITHUB_ENV) |
| `v0.8.9-test4` | `Certificato non disponibile nello store dopo 30 secondi` | **SimplySign Desktop non ha CLI di login**. Gli argomenti `/silent /login /user /password /totp` non esistono. Il login richiede GUI. |

Ricerca community + verifica documentazione Certum 2026:

- **PDF Certum ufficiale** `Code Signing - signing the code using tools like Singtool and Jarsigner.pdf`: assume SimplySign Desktop già loggato. Non documenta nessun CLI di login.
- **[devas.life tutorial](https://www.devas.life/how-to-automate-signing-your-windows-app-with-certum/)** (2023): usa **SendKeys** per pilotare la GUI di SimplySign Desktop dopo aver aperto la finestra di login con start-process. Funzionante ma fragile (timing-dependent, ~50% success rate riportato).
- **[hpvb/certum-container](https://github.com/hpvb/certum-container)**: stacka SimplySign Desktop Windows in Wine + Xvnc per esporre la GUI via VNC; serve **login manuale via VNC ogni 2h** quando la sessione scade.
- **[defguard blog](https://defguard.net/blog/windows-codesign-certum-hsm/)**: usa cert HSM USB su self-hosted runner Linux con `p11-kit` + `osslsigncode`. Non applicabile al cert cloud.

**Conclusione**: non esiste oggi un metodo headless documentato per
loggarsi a SimplySign Cloud Code Signing. Tutti gli approcci CI
funzionanti richiedono o (a) GUI scripting fragile via SendKeys
oppure (b) container Linux con login VNC manuale periodico oppure
(c) cert HSM USB su runner self-hosted (cert diverso da quello che
abbiamo).

### Flusso adottato

```
┌────────────────────────────────────────────────────────────────┐
│ Workstation maintainer (Windows)                               │
│  • SimplySign Desktop GUI logged-in                            │
│  • signtool.exe (Windows SDK)                                  │
│  • gh CLI autenticato                                          │
└────────────────────────────────────────────────────────────────┘
                          ▲
                          │  scripts/sign-release.ps1 -Tag v0.9.0
                          │  - download asset draft
                          │  - signtool sign + verify
                          │  - re-zip portable (con .exe firmato)
                          │  - gh release upload --clobber
                          │  - gh release edit --draft=false --latest
                          │
┌─────────────────────────┴──────────────────────────────────────┐
│ GitHub Releases                                                │
│  • Release draft "v0.9.0" creata da CI                         │
│  • Asset: portable.zip (unsigned), latest.json, latest.json.sig│
└─────────────────────────▲──────────────────────────────────────┘
                          │  tauri-action (in CI)
                          │  - build Tauri
                          │  - genera asset
                          │  - sign Ed25519 latest.json (chiave Updater)
                          │
┌─────────────────────────┴──────────────────────────────────────┐
│ GitHub Actions (release.yml su tag v*)                         │
│  • Windows runner, NO SimplySign, NO signtool                  │
│  • Bundle unsigned + Updater signing Ed25519                   │
└────────────────────────────────────────────────────────────────┘
```

### Vantaggi

| | |
|---|---|
| **Affidabilità** | 100% determinismo: niente GUI scripting fragile, niente sessioni VNC, niente race condition fra Install-WinGet e login |
| **Costo zero infrastrutturale** | Nessun self-hosted runner, nessun container, nessuna VM dedicata |
| **Sicurezza** | Cert credentials NON in GitHub Secrets — restano solo sulla workstation del maintainer. Anche un compromise totale del repo GitHub non espone il cert |
| **Diagnostica facile** | Lo script gira sotto gli occhi del maintainer, errori visibili immediatamente, retry one-command |
| **Indipendenza dalla macchina di sviluppo** | Lo script non richiede il codice sorgente — basta gh CLI + signtool + SimplySign Desktop. Può girare su una qualunque workstation Windows (VM, secondo PC, ecc.) |

### Svantaggi accettati

| | |
|---|---|
| **Manualità per ogni release** | ~5 min attivi del maintainer + ~15 min wait CI. Per il cadenza release di PaP (mensile/bimestrale) è gestibile |
| **Dipendenza dalla disponibilità maintainer** | Se in viaggio senza workstation Windows accessibile, la release aspetta |
| **Promessa "release riproducibile da git tag" rotta** | L'asset firmato richiede intervento umano. Mitigation: la build CI è 100% riproducibile, il signing è l'unico step manuale e copre solo la "fiducia" verso Windows, non la sostanza dell'artifact |
| **Finestra 2h SimplySign** | Lo script verifica all'avvio che il cert sia disponibile e fallisce subito se la sessione è scaduta. Re-login GUI + re-run script |

### Quando rivedere questa decisione

Riaprire l'ADR se si verifica **una** di queste condizioni:

1. **Certum rilascia un'API headless** per SimplySign Cloud Code Signing (verifica annuale alla scadenza cert).
2. **Cresce la cadenza di release a >2/settimana** rendendo la manualità inaccettabile.
3. **Si presenta esigenza di release automatizzate da bot** (es. dependabot publish flow). Allora valutare cert HSM USB con self-hosted runner (approccio D).
4. **Si vuole supportare contributor esterni che pubblicano fork firmati**: serve un meccanismo diverso (ognuno con il proprio cert + script).

### Riferimenti operativi

- Procedura step-by-step: [`../../contribuire/release-signing-workflow.md`](../../contribuire/release-signing-workflow.md)
- Script: `scripts/sign-release.ps1`
- Setup cert una tantum: §"Setup procedura — UNA TANTUM" sotto

### Interazione con Tauri Updater Ed25519 (scoperta 2026-05-16, v0.8.9-test5)

**Problema**: tauri-action genera `latest.json` + `.sig` Ed25519 sui
binari prodotti dalla build CI, che sono **unsigned** (nessun
Authenticode). Quando il maintainer esegue `signtool sign` localmente,
il contenuto dei `.exe` / `.msi` cambia (~10-12 KB di firma aggiunti).
I `.sig` Ed25519 originali quindi **non matchano più** il contenuto
dei file ri-uploadati → Tauri Updater rifiuta gli update con
`signature mismatch`.

Bug rilevato durante test pipeline su tag `v0.8.9-test5` (lasciato
draft pre-release permanente come test artifact, mai promosso a Latest).

**Soluzione adottata (opzione B)**: lo script `sign-release.ps1`
include uno step di re-signing Ed25519 dopo `signtool`:

1. Per ogni target Updater (NSIS `setup.exe` + `.msi`), genera nuovo
   `<file>.sig` via `tauri signer sign -f <updater-privkey> <file>`.
2. Scarica `latest.json` originale, sostituisce
   `platforms.*.signature` con il contenuto base64 del nuovo `.sig`,
   aggiorna `pub_date` con timestamp corrente.
3. Re-uploada `latest.json` + tutti i `.sig` con `--clobber`.

**Requisiti aggiuntivi sulla workstation di signing**:
- Tauri CLI (`cargo install tauri-cli --version "^2"` o
  `pnpm i -g @tauri-apps/cli`).
- Chiave privata Ed25519 `pap-updater.key` accessibile localmente
  (path via env `TAURI_UPDATER_PRIVATE_KEY_PATH` o param
  `-UpdaterPrivKey`). Password via env
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` se la chiave è cifrata.

**Comportamento se chiave non passata**: lo script mostra warning
chiaro e chiede conferma esplicita prima di procedere (per evitare
release "silenziosamente rotte" per gli utenti con auto-update
attivo).

**Trade-off accettato**: la chiave privata Updater ora vive in due
posti — GitHub Secret (per CI) + workstation maintainer (per
re-signing). Mitigation: la chiave è custodita anche nel password
manager del maintainer ed è già la chiave "eterna" del progetto
(perderla = utenti non possono più ricevere update). Vedi
[`../../contribuire/setup-tauri-updater-keys.md`](../../contribuire/setup-tauri-updater-keys.md)
§"Backup chiave privata".

## Contesto

- Prompt a Porter è distribuito come bundle Windows portable + (in v1.0) NSIS installer per-user + Tauri Updater.
- Senza firma Authenticode valida: SmartScreen flagga l'eseguibile come "non riconosciuto" su prima esecuzione → friction utente significativa.
- L'auto-update di Tauri richiede `latest.json` firmato per verificare l'integrità di ogni patch.
- Bloccante storico: cert Certum non disponibile durante v0.3-v0.8 (KYC in corso). Sbloccato il 2026-05-15.

## Provider valutati

| Provider | Variante | Costo | CI-friendly | Note |
|---|---|---|---|---|
| **Certum** | SimplySign Cloud Open Source | ~€60/anno | ⚠️ Sì con workaround TOTP | Scelto. Cloud-based, no HSM USB |
| Certum | Standard HSM (token USB) | ~€60/anno | ❌ Difficile | Token fisico, serve self-hosted runner |
| Certum | EV Code Signing | ~€180/anno | ❌ Idem HSM | Elimina SmartScreen warning immediato, ma costo + HSM |
| DigiCert | Software Trust Manager / KeyLocker | $475+/anno | ✅ Sì | API native, ma costo proibitivo per OSS |
| SSL.com | eSigner | $169+/anno | ✅ Sì | API REST, alternativa valida ma più costosa di Certum |
| Sectigo | Standard EV | $200+/anno | ⚠️ Variabile | EV richiede HSM, no cloud per Standard |

**Razionale scelta Certum SimplySign**:
- Costo annuale più basso fra le opzioni valide.
- Variante Open Source dedicata a progetti OSS (Certum offre sconto OSS).
- Time-stamping server gratuito (`http://time.certum.pl`).
- Compatibile con `signtool.exe` Microsoft standard via virtual smart card.
- Esperienza community consolidata (rubyinstaller2, defguard, devas.life, hpvb/certum-container).

## Realtà tecnica del SimplySign Cloud

⚠️ **Importante**: "Cloud" non significa "API REST diretta". L'architettura è:

```
┌──────────────────────┐
│ GitHub Actions       │
│ windows-latest runner│
│                      │
│  ┌─────────────────┐ │
│  │ SimplySign      │ │
│  │ Desktop App     │ │
│  │ (emula smart    │ │
│  │ card virtuale)  │ │
│  └────┬────────────┘ │
│       │ login        │
│       │ user+pass    │
│       │ +TOTP (2FA)  │
│       ▼              │
│  ┌─────────────────┐ │     HTTPS
│  │ signtool.exe    │◄┼─────────────► Certum Cloud
│  │ /sha1 thumbprint│ │     auth      (smart card backend)
│  └─────────────────┘ │
└──────────────────────┘
```

**Conseguenze pratiche**:
- Serve l'**app SimplySign Desktop** installata sul runner (download `.msi` da Certum).
- Autenticazione **TOTP a 2 fattori** richiesta a ogni avvio dell'app (sessione max **2 ore**).
- `signtool.exe` parla con la virtual smart card emulata, non direttamente con il cloud.
- Per CI: serve **TOTP seed** in GitHub Secret per generare il codice OTP corrente al volo.

## Setup procedura — UNA TANTUM

Da eseguire **manualmente sul tuo computer**, non in CI.

### Step 1 — Attiva il certificato

1. Accedi a [https://simplysign.certum.eu](https://simplysign.certum.eu) con le credenziali ricevute via email.
2. Segui il wizard di attivazione (richiede smartphone con app Certum SimplySign o Google Authenticator per il primo TOTP setup).
3. Annota:
   - **Username** (la tua email Certum)
   - **Password** account
   - **Smartphone TOTP code** funzionante (verifica)

### Step 2 — Estrai la TOTP seed

Il QR code mostrato durante l'attivazione contiene la **seed segreta** della TOTP. Va estratta come stringa Base32 per poterla usare in CI.

**Opzione A — Tramite app autenticator desktop (raccomandato)**:
1. Installa [Aegis Authenticator](https://github.com/beemdevelopment/Aegis) (Android) o [Authy Desktop](https://authy.com/) o [1Password](https://1password.com/).
2. Scansiona il QR code di Certum dall'app desktop.
3. Aegis/1Password permettono di esportare la "seed/secret" come stringa Base32 (es. `JBSWY3DPEHPK3PXP...`).

**Opzione B — Lettura QR manuale**:
1. Salva il QR code come immagine.
2. Decodifica con tool come [zbarimg](https://github.com/mchehab/zbar) o [QR-Code Decoder online](https://zxing.org/w/decode.jspx).
3. Ottieni una URI tipo: `otpauth://totp/Certum:tu@email.com?secret=JBSWY3DPEHPK3PXP&issuer=Certum&algorithm=SHA1&digits=6&period=30`
4. Estrai il valore di `secret=`.

**Verifica seed**: testa con `oathtool --totp --base32 "JBSWY3DPEHPK3PXP"` (Linux) — deve generare un codice TOTP a 6 cifre che matcha quello dell'app smartphone.

### Step 3 — Recupera il cert thumbprint

Sul tuo computer, dopo aver loggato in SimplySign Desktop:

```powershell
# Windows PowerShell
Get-ChildItem Cert:\CurrentUser\My | Where-Object {$_.Subject -like "*Roberto Marchioro*"} | Format-List Thumbprint, Subject, NotAfter
```

Annota lo `Thumbprint` (40 caratteri hex). Esempio: `A1B2C3D4E5F6...`.

### Step 4 — Test signing locale

Prima di toccare il workflow GitHub, verifica che la firma funzioni in locale.

```powershell
# Login a SimplySign Desktop (aprila, autentica con username + password + TOTP)

# Firma un eseguibile di prova
signtool sign /sha1 <THUMBPRINT> /tr http://time.certum.pl /td sha256 /fd sha256 /a path\to\test.exe

# Verifica firma
signtool verify /pa /v path\to\test.exe
```

Se la verifica passa con `Successfully verified`, il cert è OK.

## GitHub Secrets da configurare

Su `github.com/robertomarchioro/prompt-a-porter` → Settings → Secrets and variables → Actions → New repository secret:

| Nome secret | Valore | Note |
|---|---|---|
| `CERTUM_USERNAME` | La tua email SimplySign | Es: `roberto@dominio.tld` |
| `CERTUM_PASSWORD` | Password account SimplySign | Plain text, marked secret |
| `CERTUM_TOTP_SEED` | Seed Base32 estratta nello Step 2 | Es: `JBSWY3DPEHPK3PXP...` |
| `CERTUM_CERT_THUMBPRINT` | Thumbprint estratto nello Step 3 | 40 caratteri hex, no spazi |

⚠️ **Mai committare questi valori nel repo**, nemmeno in `.env.example`. Restano solo in GitHub Secrets.

## Workflow GitHub Actions — snippet pronto

Da integrare in `.github/workflows/release.yml` nel job `build (windows-latest, x86_64-pc-windows-msvc)`, **dopo** `tauri-action` ma **prima** dell'upload artifact.

```yaml
- name: Install SimplySign Desktop (CI signing)
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  run: |
    Invoke-WebRequest -Uri "https://files.certum.eu/software/SimplySignDesktop/Windows/SimplySignDesktop-Setup.exe" -OutFile "$env:TEMP\simplysign-setup.exe"
    Start-Process -FilePath "$env:TEMP\simplysign-setup.exe" -ArgumentList "/quiet","/norestart" -Wait
    # Path tipico dopo install:
    echo "C:\Program Files (x86)\Certum\SimplySign Desktop" | Out-File -FilePath $env:GITHUB_PATH -Append

- name: Install oathtool for TOTP generation
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  run: |
    # PowerShell module per TOTP (alternativa a oathtool)
    Install-Module -Name OneTimePassword -Force -Scope CurrentUser
    Import-Module OneTimePassword

- name: Login to SimplySign and sign artifacts
  if: matrix.platform == 'windows-latest'
  shell: pwsh
  env:
    CERTUM_USERNAME: ${{ secrets.CERTUM_USERNAME }}
    CERTUM_PASSWORD: ${{ secrets.CERTUM_PASSWORD }}
    CERTUM_TOTP_SEED: ${{ secrets.CERTUM_TOTP_SEED }}
    CERTUM_CERT_THUMBPRINT: ${{ secrets.CERTUM_CERT_THUMBPRINT }}
  run: |
    # Genera TOTP corrente
    $totp = Get-OneTimePassword -SharedSecret $env:CERTUM_TOTP_SEED -Base32

    # Login via SimplySign Desktop CLI (verifica path effettivo nel doc Certum)
    & "C:\Program Files (x86)\Certum\SimplySign Desktop\SimplySignDesktop.exe" `
      /silent /login `
      /user:$env:CERTUM_USERNAME `
      /password:$env:CERTUM_PASSWORD `
      /totp:$totp

    # Aspetta che la virtual smart card sia disponibile
    Start-Sleep -Seconds 10

    # Firma tutti gli artifact Tauri prodotti
    $artifacts = Get-ChildItem -Recurse `
      -Path "apps\client\src-tauri\target\release\bundle" `
      -Include "*.exe","*.msi" `
      -File

    foreach ($file in $artifacts) {
      Write-Host "Signing $($file.FullName)"
      & signtool sign `
        /sha1 $env:CERTUM_CERT_THUMBPRINT `
        /tr http://time.certum.pl `
        /td sha256 `
        /fd sha256 `
        /a `
        $file.FullName
      if ($LASTEXITCODE -ne 0) {
        throw "Signing failed for $($file.FullName)"
      }
    }

    # Verifica firma
    foreach ($file in $artifacts) {
      & signtool verify /pa /v $file.FullName
    }
```

**Note operative**:
- I path SimplySign CLI possono cambiare fra release; verifica nella `Installation_of_the_SimplySign_application` PDF di Certum.
- Il modulo PowerShell `OneTimePassword` è una scelta fra le tante; alternative valide: `WinAuth`, `2fa` CLI Go portato, `oathtool` via WSL.
- Tauri-action già produce `latest.json` + `.sig` Ed25519 per l'updater; **`signtool` firma a parte gli `.exe` e `.msi`**, non il `latest.json` (che usa una signature Tauri-specifica).

## Trade-off documentati

### Finestra 2 ore
La sessione SimplySign dura max 2 ore. Le build Tauri Win durano 8-15 minuti, quindi entriamo nel limite.

**Mitigation**: se in futuro un singolo workflow dura > 2h (es. matrix multi-arch lunga), spezzare in job paralleli, ognuno con proprio login SimplySign all'inizio.

### Re-login automatico in caso di crash SimplySign Desktop
Se l'app crasha mid-job, il signtool fallisce con "no certificate available". Il workflow non gestisce il retry automatico.

**Mitigation accettata**: rare-event, accept failure + re-run manuale. Implementare retry sofisticato non vale la complessità.

### Rinnovo annuale
Cert Certum SimplySign valid ~1 anno (dal 2026-02-27 max 459 giorni per regole CA/Browser Forum).

**Mitigation**: calendario reminder a 60 giorni dalla scadenza per rinnovare + ripetere Step 1-4 di questa procedura per nuovo cert. Aggiornare i secret in GitHub.

### Sicurezza secret
`CERTUM_TOTP_SEED` + `CERTUM_PASSWORD` insieme sono equivalenti al cert privato. Se un workflow malicious (es. dependabot PR di untrusted contributor che modifica `release.yml`) accede a questi secret, firma malware con il nostro nome.

**Mitigation**:
- GitHub: `Settings → Actions → General → Fork pull request workflows → "Require approval for all outside collaborators"`.
- I secret `CERTUM_*` sono **environment secrets** del environment `release-windows`, non repo-level. L'environment richiede manual approval prima di esporre i secret al job.
- Ruotare TOTP seed se sospetti compromise (nuovo QR Certum + aggiornamento secret).

## Recovery procedure

### Cert compromesso / chiave esposta
1. Login a `simplysign.certum.eu` → revoca certificato corrente.
2. Richiedi re-emissione (Certum supporta re-issue in caso di compromise).
3. Step 1-4 della setup procedure con nuovo cert.
4. Ruota tutti i GitHub Secrets.

### Cert scaduto
Stessa procedura, ma senza revoke. Pianifica con 60 giorni di anticipo.

### Build firmata erroneamente (es. signtool ha firmato release sbagliata)
- Il cert resta valido.
- Pubblica nuova release con asset corretto + firma corretta.
- Le release "vecchie" firmate continuano a essere valide (signing è "verità storica"); se serve, marca la release errata come "deprecated" nel body.

## Alternative non scelte

### Approccio B — Container Linux + `osslsigncode`
[`hpvb/certum-container`](https://github.com/hpvb/certum-container) (MIT) incapsula SimplySign Desktop + Xvnc + `osslsigncode` + p11-kit in container Linux. Vantaggi: più CI-native (no Windows runner Desktop emulation), retry e session management più puliti. Svantaggi: tooling extra, build Win deve produrre artifact poi firmati separatamente in altro job.

**Decisione**: opzione B documentata come fallback. Adottiamo A perché il workflow `release.yml` esistente usa già `windows-latest` per la build Tauri, e tenere signing nello stesso job riduce complessità.

### Approccio C — Pre-signing locale + upload
Buildare in CI, scaricare artifact, firmarli sulla macchina dello sviluppatore (con SimplySign Desktop logged in interactively), ri-uploadare come release asset.

**~~Scartato~~ Adottato dopo amend 2026-05-16**: l'approccio A (Windows runner CI) si è rivelato non praticabile per la mancanza di un CLI headless di SimplySign Desktop. Approccio C è ora l'opzione corrente — vedi §"Strategia pre-signing locale (approccio C — corrente)" all'inizio del documento per il dettaglio operativo. Lo scrupolo originale ("rompe la promessa di release riproducibile da git tag") è mitigato dal fatto che la build CI resta 100% riproducibile: l'unico step manuale è il signing, che modifica metadati dell'asset ma non l'asset compilato.

### Approccio D — Cert HSM con runner self-hosted
Acquistare cert Standard HSM (token USB) + tenere un mini-PC dedicato come self-hosted runner GitHub con token montato.

**Scartato**: costo infrastrutturale (mini-PC + manutenzione) > differenza cloud vs HSM, complessità setup, single point of failure.

## Item rinviati (sub-PR M1 successivi)

Questa ADR copre la decisione architetturale + setup. Implementazione effettiva nelle sub-PR successive:
- **M1.2** — workflow `release.yml` con step signing (lo snippet sopra integrato + testato)
- **M1.3** — `tauri.conf.json` NSIS per-user no UAC
- **M1.4** — Tauri Updater client + `latest.json` endpoint
- **M1.5** — downgrade refuse + signature mismatch refuse
- **M1.6** — smoke test installer Win10/Win11 + test E2E updater
- **M1.7** — `docs/utente/auto-update.md` per l'utente finale

## Appendice — Tauri Updater key pair (Ed25519)

Authenticode firma il binario; **Tauri Updater firma il `latest.json`** che annuncia nuove release. Sono **due signing distinti** con due chiavi diverse:

| | Authenticode | Tauri Updater |
|---|---|---|
| Cosa firma | `.exe`, `.msi` | `latest.json` (metadata release) |
| Chiave | RSA, cloud Certum | Ed25519, generata localmente |
| Verifica | Windows + SmartScreen | Plugin Tauri lato client |
| Provider | Certum (CA pubblica) | Tu, generata `tauri signer generate` |

L'utente NON deve disinstallare/riinstallare l'app se la chiave Tauri Updater cambia (lock-in elevato): la chiave Ed25519 va custodita come segreto a lungo termine.

### Procedura UNA TANTUM — generazione chiavi Tauri Updater

Esegui sul tuo computer (NON in CI):

```bash
cd apps/client
pnpm tauri signer generate -w ~/.tauri/pap-updater.key
```

Verrà richiesto:
- **Password della chiave privata** (opzionale ma raccomandato). Mettilo se vuoi un layer extra di sicurezza.

Output:
- `~/.tauri/pap-updater.key` → **chiave privata** (NON COMMITTARE!)
- `~/.tauri/pap-updater.key.pub` → **chiave pubblica** (da inserire in `tauri.conf.json`)

### GitHub Secrets da configurare (in aggiunta ai 4 CERTUM_*)

| Nome secret | Valore | Note |
|---|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Contenuto di `~/.tauri/pap-updater.key` | Stringa base64 multi-linea. Copia tutto il file inclusi header/footer |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password scelta sopra (vuoto se non l'hai messa) | Se vuoto: imposta secret a stringa vuota, NON ometterlo |

### Aggiornamento `tauri.conf.json`

Sostituisci il placeholder in `plugins.updater.pubkey` con il contenuto **single-line** di `~/.tauri/pap-updater.key.pub`:

```bash
cat ~/.tauri/pap-updater.key.pub
# Output esempio:
# dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...
```

Copia la stringa (è una singola riga base64) e incollala come valore di `pubkey` in `tauri.conf.json`:

```json
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6...",
    "endpoints": [
      "https://github.com/robertomarchioro/prompt-a-porter/releases/latest/download/latest.json"
    ]
  }
}
```

⚠️ La chiave pubblica è committata nel repo (è pubblica per design). La chiave privata NO.

### Backup chiave privata

`~/.tauri/pap-updater.key` è la chiave **eterna** del progetto. Se la perdi, devi:
1. Generare nuova chiave
2. Cambiare `pubkey` in `tauri.conf.json` (breaking change: gli utenti con vecchia versione non potranno verificare i nuovi `latest.json` finché non installano manualmente la nuova versione)
3. Aggiornare GitHub Secrets

Per evitare questo dramma:
- Backup `~/.tauri/pap-updater.key` in un password manager (1Password, Bitwarden, etc.)
- Mai committare in clear nel repo, nemmeno per "trovarla in futuro"

### Recovery key persa

Se la chiave privata è persa MA hai release già pubblicate firmate con la vecchia chiave:
1. Genera nuova coppia.
2. Aggiorna `tauri.conf.json` con la nuova `pubkey`.
3. Aggiorna Secret `TAURI_SIGNING_PRIVATE_KEY`.
4. **Pubblica la nuova versione come release "manuale"** con note che indicano "aggiornamento manuale richiesto, l'updater automatico non funzionerà fino a installazione nuova versione".
5. Da quel punto in avanti, l'updater riprende a funzionare per chi ha installato la nuova versione.

### Garanzie di sicurezza dell'updater (M1.5)

`tauri-plugin-updater` v2 fornisce **automaticamente** le seguenti verifiche prima di applicare un update. Nessun codice utente è necessario per attivarle.

| Garanzia | Comportamento | Bypass possibile |
|---|---|---|
| **Signature verification** | Plugin verifica la firma Ed25519 di `latest.json` (campo `signature`) contro la `pubkey` configurata in `tauri.conf.json`. Mismatch → `Error("signature mismatch")`, update NON applicato | Solo se attaccante possiede la chiave privata |
| **Downgrade refuse** | Plugin compara semver `version` corrente (`env!("CARGO_PKG_VERSION")`) vs `version` in `latest.json`. Se non strictly newer → `shouldUpdate=false`, niente prompt utente | Solo se attaccante con chiave privata firma falso `latest.json` con `version` superiore |
| **HTTPS-only endpoint** | Config `plugins.updater.endpoints` accetta solo URL `https://`. URL `http://` rifiutati a parse-time del config | No (validato a build-time) |
| **MITM su payload binario** | Plugin scarica il binario indicato in `latest.json:platforms.<os>.url`, verifica integrità via firma Ed25519 di `latest.json:platforms.<os>.signature` (firma del binario stesso, non solo del manifesto) | Solo se attaccante con chiave privata |
| **Replay attack** (vecchio `latest.json` riproposto) | Non c'è nonce/timestamp esplicito, ma il downgrade refuse copre il caso pratico (vecchio `latest.json` = vecchia version → ignored) | Solo se vecchio `latest.json` contiene version newer del corrente |

### Policy di update applicata

Coerente con il principio "i dati restano sull'utente" + privacy-first:

1. **No auto-check al boot**. L'app non interroga l'endpoint updater senza azione esplicita dell'utente. Vedi `lib.rs` plugin init: nessuna chiamata `check()` automatica nello startup.
2. **Check on-demand utente**. La UI (futuro M1.4b) espone un bottone "Verifica aggiornamenti" in Impostazioni → Sviluppo (o sezione dedicata). L'invocazione è esplicita.
3. **Download e install con conferma**. Anche dopo il check, l'apply richiede consenso esplicito utente. Niente "applica e riavvia silenziosamente".
4. **No telemetria sul check**. L'app non invia user-agent identificabili oltre il default user-agent del runtime HTTPS Tauri. Niente fingerprinting, niente analytics sul controllo update.
5. **Disabilitazione**. L'utente può disabilitare il check (preferenza futura `updater_abilitato: bool` simile a `debug_log_abilitato`). Default: abilitato ma on-demand.

### Cosa NON copre l'updater

- **Compromise della chiave privata Ed25519** → l'attaccante con la chiave può firmare un payload malevolo con version superiore e l'updater lo applicherà. Mitigation = custodia chiave (vedi Appendice §"Backup chiave privata") + rotation se sospetto.
- **Compromise dell'endpoint GitHub Releases** (es. account GitHub hijack) → l'attaccante può sostituire `latest.json` con payload firmato. Mitigation = 2FA hardware key su GitHub account + audit log monitoring.
- **Vulnerabilità del runtime WebView2 / sistema OS** → fuori scope updater.
- **Phishing che porta utente a scaricare manualmente da URL ≠ GitHub Releases** → fuori scope updater (utente bypassa il sistema).

## Riferimenti

- [Certum SimplySign — Code Signing in the cloud (Signtool/Jarsigner manual PDF)](https://www.files.certum.eu/documents/manual_en/Signing_with_the_use_of_jarsigner_tool_and_signtool.pdf)
- [How to automate signing your Windows app with Certum's SimplySign](https://www.devas.life/how-to-automate-signing-your-windows-app-with-certum/) — tutorial 1Password TOTP
- [defguard — Secure Tauri/Windows Code Signing with Certum HSM](https://defguard.net/blog/windows-codesign-certum-hsm/) — variante HSM (storico)
- [hpvb/certum-container](https://github.com/hpvb/certum-container) — approccio B container Linux
- [rubyinstaller2 CertumCodeSigning wiki](https://github.com/oneclick/rubyinstaller2/wiki/CertumCodeSigning) — esempi CI con osslsigncode
- [Tauri 2 Windows Code Signing docs](https://v2.tauri.app/distribute/sign/windows/)
- [Tauri 2 Updater docs](https://v2.tauri.app/plugin/updater/) — plugin updater, signer generate, latest.json schema
- [Tauri tauri-action README](https://github.com/tauri-apps/tauri-action) — env vars TAURI_SIGNING_PRIVATE_KEY*

## Manutenzione di questo documento

- Quando la procedura di setup cambia (Certum aggiorna SimplySign Desktop): aggiornare §"Setup procedura".
- Quando cambia il workflow: aggiornare §"Workflow GitHub Actions — snippet pronto" + link alla revisione effettiva del file.
- Quando il cert si rinnova: aggiornare data in header.
- Quando emerge un trade-off nuovo (es. crash pattern in produzione): aggiornare §"Trade-off documentati".
