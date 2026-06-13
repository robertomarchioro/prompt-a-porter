<#
.SYNOPSIS
  Pre-signing locale degli asset di una release GitHub di Prompt a Porter.

.DESCRIPTION
  Workflow: la CI (release.yml) produce gli asset unsigned in una release
  draft. Questo script — eseguito da workstation Windows con SimplySign
  Desktop logged-in — scarica gli asset, li firma Authenticode con
  signtool + virtual smart card Certum, verifica le firme, ricarica gli
  asset firmati (clobber) e opzionalmente promuove la release a Latest.

  Razionale: Certum SimplySign Cloud richiede GUI logged-in per esporre
  il cert via virtual smart card. Non esiste path headless documentato.
  Vedi docs/architettura/decisioni/authenticode-signing.md §"Strategia
  pre-signing locale" e docs/contribuire/release-signing-workflow.md.

.PARAMETER Tag
  Tag della release da firmare (es. "v0.9.0"). Deve corrispondere a una
  release esistente su GitHub (creata dalla CI, tipicamente in stato
  draft).

.PARAMETER CertThumbprint
  Thumbprint (40 hex) del certificato Certum esposto via SimplySign
  Desktop. Default: legge env var CERTUM_CERT_THUMBPRINT se settata,
  altrimenti chiede all'utente.

.PARAMETER Publish
  Se presente, dopo la firma + re-upload promuove la release da draft
  a published (con flag `--latest`). Default: lascia in draft per
  review finale manuale.

.PARAMETER Repo
  Repo GitHub owner/name. Default: "robertomarchioro/prompt-a-porter".

.PARAMETER WorkDir
  Cartella di lavoro temporanea. Default: $env:TEMP\pap-sign-<tag>.
  Viene creata se non esiste e (opzionalmente) ripulita a fine run.

.PARAMETER KeepWorkDir
  Se presente, NON cancella WorkDir a fine run (utile per debug).

.EXAMPLE
  .\scripts\sign-release.ps1 -Tag v0.9.0

  Firma gli asset del tag v0.9.0, ricarica firmati, lascia draft.

.EXAMPLE
  .\scripts\sign-release.ps1 -Tag v0.9.0 -Publish

  Firma + ricarica + promuove a Latest published.

.NOTES
  Prerequisiti:
  - Windows 10/11 con Windows SDK installato (signtool.exe)
  - SimplySign Desktop installato e logged-in PRIMA di lanciare lo script
  - GitHub CLI (gh) autenticato (`gh auth status` deve essere OK)
  - Cert Certum visibile in Cert:\CurrentUser\My (verifica:
    Get-ChildItem Cert:\CurrentUser\My | Where Thumbprint -eq <THUMB>)
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^v\d+\.\d+\.\d+(-[\w\.\-]+)?$')]
    [string]$Tag,

    [string]$CertThumbprint = $env:CERTUM_CERT_THUMBPRINT,

    [switch]$Publish,

    [string]$Repo = 'robertomarchioro/prompt-a-porter',

    [string]$WorkDir = (Join-Path $env:TEMP "pap-sign-$Tag"),

    [switch]$KeepWorkDir,

    # v1.0 M1.2 - re-signing Ed25519 Tauri Updater (opzione B).
    # I .sig generati dalla CI sono calcolati sui binari unsigned;
    # dopo signtool il contenuto cambia e i .sig non matchano piu',
    # rompendo il Tauri Updater (signature mismatch). Se passato
    # questo param, lo script rigenera .sig + latest.json sui
    # binari firmati. Se omesso, viene mostrato un warning chiaro
    # e si procede senza re-signing.
    [string]$UpdaterPrivKey = $env:TAURI_UPDATER_PRIVATE_KEY_PATH,

    [string]$UpdaterPrivKeyPassword = $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# 1. Preflight checks
Write-Host "==========================================================="
Write-Host "  Prompt a Porter - sign-release.ps1"
Write-Host "  Tag:     $Tag"
Write-Host "  Repo:    $Repo"
Write-Host "  WorkDir: $WorkDir"
Write-Host "  Publish: $Publish"
Write-Host "==========================================================="

# 1a. Preflight summary - visione d'insieme di tutti gli ingredienti.
# Non-blocking: i check critici successivi falliscono fast se mancano
# componenti essenziali. Serve a vedere a colpo d'occhio cosa manca.
function Show-PreflightSummary {
    Write-Host ""
    Write-Host "-- Preflight check --"

    $envThumb = [Environment]::GetEnvironmentVariable('CERTUM_CERT_THUMBPRINT', 'User')
    $envKey = [Environment]::GetEnvironmentVariable('TAURI_UPDATER_PRIVATE_KEY_PATH', 'User')
    $envPwd = [Environment]::GetEnvironmentVariable('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', 'User')

    $checks = [System.Collections.ArrayList]@()

    # CERTUM_CERT_THUMBPRINT
    if ($envThumb) {
        [void]$checks.Add(@{ Name = 'CERTUM_CERT_THUMBPRINT'; Status = 'OK'; Detail = $envThumb })
    } else {
        [void]$checks.Add(@{ Name = 'CERTUM_CERT_THUMBPRINT'; Status = 'MISSING'; Detail = '' })
    }

    # TAURI_UPDATER_PRIVATE_KEY_PATH (path env var must point to existing file)
    if (-not $envKey) {
        [void]$checks.Add(@{ Name = 'TAURI_UPDATER_PRIVATE_KEY_PATH'; Status = 'MISSING (opzione B disabilitata)'; Detail = '' })
    } elseif (-not (Test-Path $envKey)) {
        [void]$checks.Add(@{ Name = 'TAURI_UPDATER_PRIVATE_KEY_PATH'; Status = 'PATH_NOT_FOUND'; Detail = $envKey })
    } else {
        [void]$checks.Add(@{ Name = 'TAURI_UPDATER_PRIVATE_KEY_PATH'; Status = 'OK'; Detail = $envKey })
    }

    # TAURI_SIGNING_PRIVATE_KEY_PASSWORD (vuota = OK se chiave non cifrata)
    if ($envPwd) {
        [void]$checks.Add(@{ Name = 'TAURI_SIGNING_PRIVATE_KEY_PASSWORD'; Status = 'SETTATA'; Detail = '(valore nascosto)' })
    } else {
        [void]$checks.Add(@{ Name = 'TAURI_SIGNING_PRIVATE_KEY_PASSWORD'; Status = '(vuota - ok se chiave non cifrata)'; Detail = '' })
    }

    # tauri CLI
    $tauriCmd = Get-Command tauri -ErrorAction SilentlyContinue
    if ($tauriCmd) {
        [void]$checks.Add(@{ Name = 'tauri CLI'; Status = 'OK'; Detail = (& tauri -V) })
    } else {
        [void]$checks.Add(@{ Name = 'tauri CLI'; Status = 'MISSING (re-signing Ed25519 disabilitato)'; Detail = '' })
    }

    # signtool
    $st = (Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
           Sort-Object -Property FullName -Descending | Select-Object -First 1).FullName
    if (-not $st) {
        $st = (Get-ChildItem "$env:ProgramFiles\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
               Sort-Object -Property FullName -Descending | Select-Object -First 1).FullName
    }
    if ($st) {
        [void]$checks.Add(@{ Name = 'signtool.exe'; Status = 'OK'; Detail = $st })
    } else {
        [void]$checks.Add(@{ Name = 'signtool.exe'; Status = 'MISSING (run setup-windows.ps1)'; Detail = '' })
    }

    # gh CLI auth
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        & gh auth status 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            [void]$checks.Add(@{ Name = 'gh CLI'; Status = 'OK'; Detail = '(authenticated)' })
        } else {
            [void]$checks.Add(@{ Name = 'gh CLI'; Status = 'NOT_AUTHENTICATED (run: gh auth login)'; Detail = '' })
        }
    } else {
        [void]$checks.Add(@{ Name = 'gh CLI'; Status = 'MISSING'; Detail = '' })
    }

    # Cert visible in store (richiede SimplySign loggato)
    if ($envThumb) {
        $cert = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue |
                Where-Object { $_.Thumbprint -eq $envThumb }
        if ($cert) {
            $subj = $cert.Subject
            if ($subj.Length -gt 60) { $subj = $subj.Substring(0, 60) + '...' }
            [void]$checks.Add(@{ Name = 'Cert in Cert:\CurrentUser\My'; Status = 'OK'; Detail = $subj })
        } else {
            [void]$checks.Add(@{ Name = 'Cert in Cert:\CurrentUser\My'; Status = 'NOT_FOUND (apri SimplySign Desktop + login)'; Detail = '' })
        }
    } else {
        [void]$checks.Add(@{ Name = 'Cert in Cert:\CurrentUser\My'; Status = 'SKIP (thumbprint missing)'; Detail = '' })
    }

    foreach ($c in $checks) {
        $line = "  {0,-40} {1}" -f $c.Name, $c.Status
        if ($c.Detail) {
            $line += "  ($($c.Detail))"
        }
        Write-Host $line
    }
    Write-Host ""
}

Show-PreflightSummary

if ([string]::IsNullOrWhiteSpace($CertThumbprint)) {
    $CertThumbprint = Read-Host "Thumbprint cert Certum (40 hex)"
}
$CertThumbprint = $CertThumbprint.Trim().ToUpper() -replace '\s', ''
if ($CertThumbprint.Length -ne 40 -or $CertThumbprint -notmatch '^[0-9A-F]+$') {
    throw "CertThumbprint non valido (atteso 40 hex). Ricevuto: '$CertThumbprint'"
}

$ghCmd = Get-Command gh -ErrorAction SilentlyContinue
if (-not $ghCmd) {
    throw "gh CLI non trovato in PATH. Installa da https://cli.github.com/"
}
& gh auth status 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "gh CLI non autenticato. Esegui 'gh auth login' prima di rilanciare."
}
Write-Host "[OK] gh CLI autenticato"

$signtool = (Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
             Sort-Object -Property FullName -Descending |
             Select-Object -First 1).FullName
if (-not $signtool) {
    $signtool = (Get-ChildItem -Path "$env:ProgramFiles\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
                 Sort-Object -Property FullName -Descending |
                 Select-Object -First 1).FullName
}
if (-not $signtool) {
    throw "signtool.exe non trovato. Installa Windows SDK da https://developer.microsoft.com/windows/downloads/windows-sdk/"
}
Write-Host "[OK] signtool: $signtool"

$cert = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue |
        Where-Object { $_.Thumbprint -eq $CertThumbprint }
if (-not $cert) {
    Write-Host ""
    Write-Host "[FAIL] Certificato $CertThumbprint NON trovato in Cert:\CurrentUser\My"
    Write-Host ""
    Write-Host "  Probabile causa: SimplySign Desktop non e' logged-in."
    Write-Host "  Apri SimplySign Desktop -> Login (user + password + TOTP)"
    Write-Host "  poi rilancia questo script."
    Write-Host ""
    Write-Host "  Cert disponibili attualmente:"
    Get-ChildItem Cert:\CurrentUser\My | Format-Table Thumbprint, Subject, NotAfter
    throw "Cert non disponibile."
}
Write-Host "[OK] Cert trovato: $($cert.Subject)"
Write-Host "     Scade: $($cert.NotAfter)"

$releaseJson = & gh release view $Tag --repo $Repo --json tagName,name,isDraft,isPrerelease,assets 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "Release $Tag non trovata su $Repo. Output gh: $releaseJson"
}
$release = $releaseJson | ConvertFrom-Json
Write-Host "[OK] Release $Tag trovata (draft=$($release.isDraft), prerelease=$($release.isPrerelease))"
Write-Host "     Asset presenti: $($release.assets.Count)"
$release.assets | ForEach-Object { Write-Host "       - $($_.name)" }

# 2. Setup workdir
function Remove-WorkDirRobust {
    param([string]$Path)
    # Esci dalla dir per liberare handle CWD (capita dopo crash di un
    # run precedente con Set-Location dentro $WorkDir).
    Set-Location $env:USERPROFILE
    # Primo tentativo veloce
    Remove-Item -Path $Path -Recurse -Force -ErrorAction SilentlyContinue
    if (-not (Test-Path $Path)) { return }
    # Retry: tipicamente file ancora locked da Defender scan dopo
    # Expand-Archive. Attendi e riprova file-by-file (innermost first).
    Write-Host "Cleanup parziale - retry tolerante (Defender lock?)..."
    Start-Sleep -Milliseconds 1500
    Get-ChildItem -Path $Path -Recurse -Force -ErrorAction SilentlyContinue |
        Sort-Object -Property FullName -Descending |
        ForEach-Object {
            try { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction Stop }
            catch { Start-Sleep -Milliseconds 200; try { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction Stop } catch { } }
        }
    Remove-Item -Path $Path -Recurse -Force -ErrorAction SilentlyContinue
    if (Test-Path $Path) {
        throw "Impossibile pulire $Path. Chiudi processi che usano file dentro questa cartella (es. esplora risorse, antivirus scan) e rilancia. Workaround: cancella manualmente la cartella e rilancia lo script."
    }
}

if (Test-Path $WorkDir) {
    Write-Host ""
    Write-Host "WorkDir esiste gia': $WorkDir"
    $reuse = Read-Host "Riutilizzare contenuto esistente? [y/N]"
    if ($reuse -notmatch '^[yY]') {
        Remove-WorkDirRobust -Path $WorkDir
        New-Item -ItemType Directory -Path $WorkDir | Out-Null
    }
} else {
    New-Item -ItemType Directory -Path $WorkDir | Out-Null
}
Set-Location $WorkDir
Write-Host "[OK] WorkDir pronta: $WorkDir"

# 3. Download asset firmabili (+ updater artifacts se re-signing attivo)
# NB: l'MSI è stato rimosso dalla release (release.yml `--bundles nsis`):
# solo NSIS setup.exe per-utente + portable. Niente più pattern `*.msi`.
$signablePatterns = @('*.exe')
$zipPattern = '*portable*.zip'
$updaterPatterns = @('*.sig', 'latest.json')

$doUpdaterReSign = -not [string]::IsNullOrWhiteSpace($UpdaterPrivKey)
if ($doUpdaterReSign) {
    if (-not (Test-Path $UpdaterPrivKey)) {
        throw "UpdaterPrivKey path non esiste: $UpdaterPrivKey"
    }
    $tauriCmd = Get-Command tauri -ErrorAction SilentlyContinue
    if (-not $tauriCmd) {
        throw "tauri CLI non trovato. Installa con 'cargo install tauri-cli --version ^2' oppure 'pnpm i -g @tauri-apps/cli'."
    }
    Write-Host "[OK] Tauri Updater re-signing ATTIVO (key: $UpdaterPrivKey)"
} else {
    Write-Host ""
    Write-Host "[WARN] Tauri Updater re-signing SKIPPED."
    Write-Host "       I .sig generati dalla CI sono calcolati sui binari unsigned;"
    Write-Host "       dopo signtool non matchano piu' -> Tauri Updater rifiutera'"
    Write-Host "       gli update da questa release con 'signature mismatch'."
    Write-Host ""
    Write-Host "       Per abilitare il re-signing, passa -UpdaterPrivKey <path>"
    Write-Host "       (o setta env TAURI_UPDATER_PRIVATE_KEY_PATH)."
    Write-Host ""
    $continueAnyway = Read-Host "Procedere comunque senza updater re-signing? [y/N]"
    if ($continueAnyway -notmatch '^[yY]') {
        throw "Interrotto dall'utente."
    }
}

Write-Host ""
Write-Host "Scarico asset (.exe, .zip portable$(if($doUpdaterReSign){', .sig, latest.json'}))..."
$downloadPatterns = $signablePatterns + $zipPattern
if ($doUpdaterReSign) { $downloadPatterns += $updaterPatterns }
foreach ($pattern in $downloadPatterns) {
    # `gh release download` esce non-zero quando nessun asset combacia col
    # pattern; con $ErrorActionPreference='Stop' (+ PS 7.3 native error
    # action) ciò diventa terminante e abortisce. Alcuni pattern sono però
    # opzionali (es. nessun .msi: rimosso by design), quindi tolleriamo il
    # caso "no assets match" e ri-lanciamo solo errori reali. Il controllo
    # "$downloaded.Count -eq 0" più sotto copre il caso davvero vuoto.
    try {
        & gh release download $Tag --repo $Repo --pattern $pattern --skip-existing 2>&1 | Out-Null
    } catch {
        if ("$_" -notmatch 'no assets match') { throw }
        Write-Host "  (pattern '$pattern': nessun asset corrispondente, skip)"
    }
}

$downloaded = @(Get-ChildItem -Path $WorkDir -File |
                Where-Object { $_.Extension -in '.exe', '.msi', '.zip' })
if ($downloaded.Count -eq 0) {
    throw "Nessun asset firmabile scaricato. Controlla che la release abbia .exe / .msi / .zip."
}
Write-Host "[OK] $($downloaded.Count) asset scaricati:"
$downloaded | ForEach-Object { Write-Host "       - $($_.Name) ($([math]::Round($_.Length/1MB,2)) MB)" }

# 4. Estrai .zip portable per firmare l'.exe interno
$zipFiles = @($downloaded | Where-Object { $_.Extension -eq '.zip' })
$extractedExesFromZip = @()

foreach ($zip in $zipFiles) {
    $extractDir = Join-Path $WorkDir ("_extracted_" + [System.IO.Path]::GetFileNameWithoutExtension($zip.Name))
    if (Test-Path $extractDir) { Remove-Item $extractDir -Recurse -Force }
    New-Item -ItemType Directory -Path $extractDir | Out-Null

    Write-Host ""
    Write-Host "Estraggo $($zip.Name) in $extractDir..."
    Expand-Archive -Path $zip.FullName -DestinationPath $extractDir -Force

    $innerExes = @(Get-ChildItem -Path $extractDir -Recurse -Filter *.exe -File)
    if ($innerExes.Count -eq 0) {
        Write-Warning "Nessun .exe trovato dentro $($zip.Name) - verra' ri-uploadato as-is."
        continue
    }
    foreach ($innerExe in $innerExes) {
        $extractedExesFromZip += [PSCustomObject]@{
            ZipPath    = $zip.FullName
            ExtractDir = $extractDir
            ExePath    = $innerExe.FullName
        }
        Write-Host "  -> $($innerExe.FullName)"
    }
}

# 5. Lista finale di file da firmare
$toSign = @()
$toSign += ($downloaded | Where-Object { $_.Extension -in '.exe', '.msi' } | ForEach-Object { $_.FullName })
$toSign += ($extractedExesFromZip | ForEach-Object { $_.ExePath })

if ($toSign.Count -eq 0) {
    throw "Nessun file da firmare dopo download/estrazione."
}

Write-Host ""
Write-Host "-- Firma di $($toSign.Count) file --"
foreach ($file in $toSign) { Write-Host "  - $file" }

# 6. Firma signtool
foreach ($file in $toSign) {
    Write-Host ""
    Write-Host "Firmando $file..."
    & $signtool sign `
        /sha1 $CertThumbprint `
        /tr 'http://time.certum.pl' `
        /td sha256 `
        /fd sha256 `
        /a `
        $file
    if ($LASTEXITCODE -ne 0) {
        throw "Signing FALLITO per $file (signtool exit $LASTEXITCODE)"
    }
}

# 7. Verifica firma
Write-Host ""
Write-Host "-- Verifica firme --"
foreach ($file in $toSign) {
    & $signtool verify /pa /v $file
    if ($LASTEXITCODE -ne 0) {
        throw "Verifica firma FALLITA per $file"
    }
}
Write-Host "[OK] Tutte le firme verificate."

# 7b-pre. Costruisce le note "firmato/pubblicato" qui, prima del blocco
# latest.json, cosi' $publishedNotes e' disponibile sia per aggiornare
# latest.json.notes (sotto) sia per il gh release edit nel passo 10.
# Le note generate dalla CI descrivono lo stato draft PRE-firma ("Release
# in stato draft - signing pending / NON ancora firmati Authenticode").
# Sostituirle subito evita che Tauri Updater mostri un avviso fuorviante.
$notesTemplate = @'
Build firmata dal tag `__TAG__`.

Vedi [CHANGELOG.md](https://github.com/__REPO__/blob/main/CHANGELOG.md) per il dettaglio.

**Release firmata Authenticode (Certum Code Signing Open Source) e pubblicata.** I file `.sig` Ed25519 dell'updater sono rigenerati sui binari firmati.

**Download - due opzioni, entrambe senza privilegi admin:**

- **Installer NSIS** (`...-setup.exe`): installer per-utente in `%LocalAppData%`, nessun UAC/admin. Consigliato - riceve gli auto-update.
- **Portable** (`Prompt-a-Porter-portable-windows-x64-__TAG__.zip`): eseguibile standalone, niente installer. Estrai e lancia `Prompt a Porter.exe`.

Richiede WebView2 runtime (incluso in Windows 11; su Windows 10 scaricabile [da Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/)).

> Alla prima esecuzione Windows SmartScreen potrebbe ancora avvisare (reputation building del nuovo certificato): "Maggiori informazioni" -> "Esegui comunque". Sparisce con l'uso.

L'installer MSI e' stato rimosso di proposito (WiX/MSI installa in `Program Files` con UAC, contro la filosofia local-first single-user di PaP).

> **Nota:** i target macOS / Linux sono temporaneamente disabilitati nella build matrix (in corso di riabilitazione).
'@
$publishedNotes = $notesTemplate.Replace('__TAG__', $Tag).Replace('__REPO__', $Repo)

# 7b. Re-signing Ed25519 Tauri Updater (opzione B v1.0 M1.2).
# I .sig della CI sono calcolati sui binari unsigned; dopo signtool
# il contenuto e' cambiato e i .sig sono stale -> Tauri Updater
# rifiuterebbe gli update con 'signature mismatch'. Qui generiamo
# nuovi .sig sui binari firmati e ricomputiamo latest.json.
$updaterTargets = @()  # paths dei file Updater-relevant ri-firmati
$updaterArtifactsToUpload = @()  # paths di .sig + latest.json da uppare
if ($doUpdaterReSign) {
    Write-Host ""
    Write-Host "-- Re-signing Ed25519 Tauri Updater --"

    # Target Updater: NSIS setup.exe + MSI (NON il portable .exe,
    # NON il binario raw -> Tauri Updater su Windows segue
    # latest.json -> platforms.windows-x86_64.url, di default i
    # bundle NSIS/MSI).
    $updaterTargets = @($toSign | Where-Object {
        ($_ -like '*setup.exe') -or ($_ -like '*.msi')
    })
    if ($updaterTargets.Count -eq 0) {
        Write-Warning "Nessun target Updater (.msi / setup.exe) trovato fra i firmati. Skip re-signing."
    } else {
        # Mappa filename -> nuova signature (contenuto file .sig)
        $newSigContent = @{}  # filename "Prompt....msi" -> raw .sig text

        foreach ($file in $updaterTargets) {
            $sigPath = "$file.sig"
            if (Test-Path $sigPath) { Remove-Item $sigPath -Force }

            Write-Host "Re-sign Ed25519: $(Split-Path $file -Leaf)..."
            $tauriArgs = @('signer', 'sign', '-f', $UpdaterPrivKey)
            if (-not [string]::IsNullOrEmpty($UpdaterPrivKeyPassword)) {
                $tauriArgs += @('-p', $UpdaterPrivKeyPassword)
            }
            $tauriArgs += $file
            # Cattura stderr (tauri scrive errori su stderr): 2>&1 li
            # ridireziona su stdout cosi' diventano visibili.
            $tauriOutput = & tauri @tauriArgs 2>&1
            $tauriExit = $LASTEXITCODE
            if ($tauriExit -ne 0 -or -not (Test-Path $sigPath)) {
                Write-Host ""
                Write-Host "  [tauri output completo]:"
                $tauriOutput | ForEach-Object { Write-Host "    $_" }
                Write-Host ""
                Write-Host "  Possibili cause exit 1:"
                Write-Host "  - Password TAURI_SIGNING_PRIVATE_KEY_PASSWORD errata"
                Write-Host "  - Chiave $UpdaterPrivKey corrotta o non valida"
                Write-Host "  - Tauri CLI version incompatibile (verifica: tauri -V vs sintassi 'signer sign')"
                Write-Host ""
                Write-Host "  Debug manuale (testa la chiave su un file dummy):"
                Write-Host "    echo test > test.txt"
                Write-Host "    tauri signer sign -f '$UpdaterPrivKey' -p '<password>' test.txt"
                Write-Host ""
                throw "tauri signer sign FALLITO per $file (exit $tauriExit)"
            }
            $newSigContent[(Split-Path $file -Leaf)] = (Get-Content $sigPath -Raw)
            $updaterArtifactsToUpload += $sigPath
        }

        # Rigenera latest.json: scarica originale (se non gia' presente),
        # sostituisci signature in ogni platform, aggiorna pub_date.
        $latestJsonPath = Join-Path $WorkDir 'latest.json'
        if (-not (Test-Path $latestJsonPath)) {
            & gh release download $Tag --repo $Repo --pattern 'latest.json' --skip-existing 2>&1 | Out-Null
        }
        if (-not (Test-Path $latestJsonPath)) {
            throw "latest.json non scaricabile dalla release $Tag."
        }
        $latest = Get-Content $latestJsonPath -Raw | ConvertFrom-Json

        # latest.json schema: platforms.<key>.{signature, url}
        # signature = contenuto file .sig as-is. Importante: la CLI
        # `tauri signer sign` v2 emette gia' il .sig in formato base64
        # (encoding del minisign 4-line raw text). Tauri Updater client
        # si aspetta proprio quel base64 nel campo `signature`. NON
        # ri-encodare in base64 (sarebbe doppia encoding -> updater
        # rifiuterebbe con "invalid signature").
        foreach ($prop in $latest.platforms.PSObject.Properties) {
            $platform = $prop.Value
            $url = $platform.url
            $fname = ($url -split '/' | Select-Object -Last 1)
            if ($newSigContent.ContainsKey($fname)) {
                $platform.signature = $newSigContent[$fname].Trim()
                Write-Host "  latest.json platforms.$($prop.Name) -> nuova signature per $fname"
            } else {
                Write-Warning "  latest.json platforms.$($prop.Name) -> url '$fname' non in target ri-firmati, lascio signature stale"
            }
        }
        $latest.pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
        # Aggiorna notes con il testo "firmato/pubblicato" costruito prima
        # di questo blocco, cosi' Tauri Updater non mostra piu' il
        # disclaimer "signing pending / binari NON firmati" della CI.
        $latest.notes = $publishedNotes

        # Scrivi senza BOM UTF-8 (PS 5.1 Set-Content -Encoding utf8
        # aggiunge BOM; serde Rust + Tauri Updater non gradiscono).
        # .NET API e' compatibile sia PS 5.1 che PS 7+.
        $json = $latest | ConvertTo-Json -Depth 10
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($latestJsonPath, $json, $utf8NoBom)
        $updaterArtifactsToUpload += $latestJsonPath
        Write-Host "[OK] latest.json rigenerato (no BOM) + .sig ri-firmati ($($newSigContent.Count) target)"
    }
}

# 8. Re-zip portable (con .exe firmato dentro)
foreach ($entry in ($extractedExesFromZip | Group-Object ZipPath)) {
    $zipPath = $entry.Name
    $extractDir = ($entry.Group | Select-Object -First 1).ExtractDir

    Write-Host ""
    Write-Host "Re-zip $zipPath (con .exe firmato)..."

    Remove-Item -Path $zipPath -Force
    Compress-Archive -Path (Join-Path $extractDir '*') -DestinationPath $zipPath -Force

    $newSize = (Get-Item $zipPath).Length
    Write-Host "[OK] $($zipPath | Split-Path -Leaf) -> $([math]::Round($newSize/1MB,2)) MB"
}

# 9. Re-upload firmati con clobber (+ updater artifacts se ri-firmati)
Write-Host ""
Write-Host "-- Re-upload asset firmati (clobber) --"
$toUpload = @($downloaded | ForEach-Object { $_.FullName })
if ($doUpdaterReSign -and $updaterArtifactsToUpload.Count -gt 0) {
    $toUpload += $updaterArtifactsToUpload
}
foreach ($file in $toUpload) {
    $fileName = Split-Path $file -Leaf
    Write-Host "Upload $fileName..."
    & gh release upload $Tag $file --repo $Repo --clobber
    if ($LASTEXITCODE -ne 0) {
        throw "Upload FALLITO per $fileName (gh exit $LASTEXITCODE)"
    }
}
Write-Host "[OK] Tutti gli asset firmati ricaricati."

# 10. Publish opzionale
if ($Publish) {
    Write-Host ""
    Write-Host "-- Promuovo release a Latest published --"

    # $publishedNotes e' gia' costruito prima del blocco 7b (latest.json)
    # cosi' il campo notes di latest.json e il body della release usano
    # esattamente lo stesso testo. Nessuna ridefinizione necessaria qui.
    $notesFile = Join-Path $WorkDir 'published-notes.md'
    $utf8NoBomNotes = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($notesFile, $publishedNotes, $utf8NoBomNotes)

    $editArgs = @('release', 'edit', $Tag, '--repo', $Repo, '--draft=false', '--notes-file', $notesFile)
    if (-not $release.isPrerelease) {
        $editArgs += '--latest'
    }
    & gh @editArgs
    if ($LASTEXITCODE -ne 0) {
        throw "gh release edit FALLITO (exit $LASTEXITCODE)"
    }
    Write-Host "[OK] Release $Tag pubblicata (note aggiornate a stato firmato)."
} else {
    Write-Host ""
    Write-Host "Release resta in stato draft. Per pubblicare manualmente:"
    Write-Host "  gh release edit $Tag --repo $Repo --draft=false --latest"
    Write-Host "Oppure rilancia con -Publish."
}

# 11. Cleanup
Set-Location $env:USERPROFILE
if (-not $KeepWorkDir) {
    Write-Host ""
    Write-Host "Cleanup WorkDir $WorkDir..."
    Remove-Item -Path $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "==========================================================="
Write-Host "  DONE - Release $Tag firmata e ricaricata."
Write-Host "==========================================================="
