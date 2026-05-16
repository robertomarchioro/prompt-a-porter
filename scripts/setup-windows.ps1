<#
.SYNOPSIS
  Setup guidato workstation Windows per signing release Prompt a Porter.

.DESCRIPTION
  Verifica e installa tutti i componenti necessari per eseguire
  scripts\sign-release.ps1 su una macchina Windows 10/11 (anche
  IoT Enterprise). Configura env vars permanenti per il maintainer.

  Componenti gestiti:
   - GitHub CLI (gh)             -> winget GitHub.cli
   - SimplySign Desktop          -> winget Certum.SmartSignSimplySignDesktop
   - Node.js LTS                 -> winget OpenJS.NodeJS.LTS
   - pnpm                        -> via corepack (incluso in Node.js >=16)
   - Tauri CLI (@tauri-apps/cli) -> pnpm i -g
   - Windows SDK Signing Tools   -> NON installabile da winget; lo script
     verifica solo la presenza di signtool.exe e linka il download manuale.

  Setup interattivo:
   - Copia chiave privata Tauri Updater (pap-updater.key) nella posizione
     standard %USERPROFILE%\.tauri\
   - Configura env vars permanenti (User scope):
       CERTUM_CERT_THUMBPRINT
       TAURI_UPDATER_PRIVATE_KEY_PATH
       TAURI_SIGNING_PRIVATE_KEY_PASSWORD (opzionale)

.PARAMETER SkipInstall
  Salta l'install dei componenti, esegue solo verifica + setup env vars.

.PARAMETER UpdaterKeyPath
  Path sorgente al file pap-updater.key (es. da pen drive o backup).
  Se omesso, viene richiesto interattivamente.

.PARAMETER CertThumbprint
  Thumbprint del cert Certum (40 hex). Se omesso, viene richiesto.

.EXAMPLE
  .\scripts\setup-windows.ps1

  Setup interattivo completo.

.EXAMPLE
  .\scripts\setup-windows.ps1 -SkipInstall -CertThumbprint <40-hex>

  Salta install, solo configura env vars.

.NOTES
  Eseguire come utente normale (NON come Administrator). winget usera'
  UAC per chiedere conferma quando serve.

  Dopo l'esecuzione, chiudi e riapri PowerShell per caricare le env
  vars aggiornate e i nuovi tool nel PATH.
#>

[CmdletBinding()]
param(
    [switch]$SkipInstall,
    [string]$UpdaterKeyPath,
    [string]$CertThumbprint
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host "==========================================================="
    Write-Host "  $Title"
    Write-Host "==========================================================="
}

function Test-CommandExists {
    param([string]$Command)
    return [bool](Get-Command $Command -ErrorAction SilentlyContinue)
}

function Install-WinGetPackage {
    param(
        [string]$PackageId,
        [string]$DisplayName
    )
    Write-Host "Verifica $DisplayName ($PackageId)..."
    $listed = winget list --id $PackageId --exact --accept-source-agreements 2>&1
    if ($LASTEXITCODE -eq 0 -and $listed -match $PackageId) {
        Write-Host "  [OK] gia' installato"
        return
    }
    Write-Host "  Installazione via winget..."
    winget install `
        --id $PackageId `
        --exact `
        --accept-package-agreements `
        --accept-source-agreements `
        --silent `
        --disable-interactivity
    $exit = $LASTEXITCODE
    # 0 = OK, -1978335215 = already installed
    if ($exit -ne 0 -and $exit -ne -1978335215) {
        throw "winget install fallito per $PackageId (exit $exit)"
    }
    Write-Host "  [OK] installato"
}

# 1. Preflight: verifica winget disponibile (App Installer da Microsoft Store)
Write-Section "1. Preflight"
if (-not (Test-CommandExists 'winget')) {
    throw "winget non disponibile. Installa 'App Installer' dal Microsoft Store: https://aka.ms/getwinget"
}
Write-Host "[OK] winget presente"

if ($SkipInstall) {
    Write-Host "[INFO] -SkipInstall passato: salto installazioni"
} else {
    Write-Section "2. Install software via winget"
    Install-WinGetPackage -PackageId 'GitHub.cli' -DisplayName 'GitHub CLI'
    Install-WinGetPackage -PackageId 'Certum.SmartSignSimplySignDesktop' -DisplayName 'SimplySign Desktop'
    Install-WinGetPackage -PackageId 'OpenJS.NodeJS.LTS' -DisplayName 'Node.js LTS'

    # Refresh PATH per la sessione corrente (winget aggiunge a User PATH
    # solo per nuove shell; per usare node/npm subito qui ricarichiamo).
    $env:PATH = [Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' +
                [Environment]::GetEnvironmentVariable('PATH', 'User')

    Write-Section "3. Setup pnpm + Tauri CLI"
    if (-not (Test-CommandExists 'corepack')) {
        Write-Host "[WARN] corepack non trovato in PATH. Riapri PowerShell e rilancia (Node appena installato)."
        Write-Host "       Workaround: chiudi questo terminale, riapri, rilancia: .\scripts\setup-windows.ps1 -SkipInstall"
        exit 0
    }
    Write-Host "Abilito pnpm via corepack..."
    corepack enable
    corepack prepare pnpm@latest --activate
    if (-not (Test-CommandExists 'pnpm')) {
        throw "pnpm non disponibile dopo corepack enable. Riapri PowerShell."
    }
    Write-Host "[OK] pnpm: $(pnpm -v)"

    Write-Host "Installo @tauri-apps/cli globalmente..."
    pnpm i -g @tauri-apps/cli
    if (-not (Test-CommandExists 'tauri')) {
        # pnpm global bin puo' non essere in PATH la prima volta
        $pnpmBin = (pnpm bin -g).Trim()
        Write-Host "[WARN] 'tauri' non in PATH. pnpm global bin: $pnpmBin"
        Write-Host "       Aggiungilo al PATH utente:"
        Write-Host "       [Environment]::SetEnvironmentVariable('PATH', `"`$env:PATH;$pnpmBin`", 'User')"
        Write-Host "       Poi riapri PowerShell."
    } else {
        Write-Host "[OK] tauri: $(tauri -V)"
    }
}

# 4. Verifica signtool (Windows SDK)
Write-Section "4. Verifica Windows SDK signtool"
$signtool = (Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
             Sort-Object -Property FullName -Descending |
             Select-Object -First 1).FullName
if (-not $signtool) {
    $signtool = (Get-ChildItem -Path "$env:ProgramFiles\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
                 Sort-Object -Property FullName -Descending |
                 Select-Object -First 1).FullName
}
if (-not $signtool) {
    Write-Host "[FAIL] signtool.exe NON trovato."
    Write-Host ""
    Write-Host "  Scarica Windows SDK da:"
    Write-Host "  https://developer.microsoft.com/windows/downloads/windows-sdk/"
    Write-Host ""
    Write-Host "  Durante l'installazione seleziona solo:"
    Write-Host "    [x] Windows SDK Signing Tools for Desktop Apps"
    Write-Host ""
    Write-Host "  Poi rilancia questo script con -SkipInstall per validare."
} else {
    Write-Host "[OK] signtool: $signtool"
}

# 5. Verifica gh CLI auth
Write-Section "5. Verifica gh CLI auth"
if (Test-CommandExists 'gh') {
    & gh auth status 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[INFO] gh CLI non autenticato. Lancia: gh auth login"
        $doLogin = Read-Host "Eseguire 'gh auth login' adesso? [Y/n]"
        if ($doLogin -notmatch '^[nN]') {
            gh auth login
        }
    } else {
        Write-Host "[OK] gh CLI autenticato"
    }
} else {
    Write-Host "[WARN] gh non in PATH (forse appena installato). Riapri PowerShell e rilancia con -SkipInstall."
}

# 6. Setup env var CERTUM_CERT_THUMBPRINT
Write-Section "6. Env var CERTUM_CERT_THUMBPRINT"
$existing = [Environment]::GetEnvironmentVariable('CERTUM_CERT_THUMBPRINT', 'User')
if ($existing) {
    Write-Host "[OK] gia' settata: $existing"
    $overwrite = Read-Host "Sovrascrivere? [y/N]"
    if ($overwrite -notmatch '^[yY]') { $CertThumbprint = $existing }
}
if (-not $CertThumbprint -or $CertThumbprint -eq $existing) {
    if (-not $CertThumbprint) {
        Write-Host ""
        Write-Host "  Per recuperare il thumbprint:"
        Write-Host "  1. Apri SimplySign Desktop e fai login (user+pwd+TOTP)"
        Write-Host "  2. Get-ChildItem Cert:\CurrentUser\My | Where Subject -like '*Roberto Marchioro*' | FL Thumbprint,Subject"
        Write-Host ""
        $CertThumbprint = Read-Host "Thumbprint cert Certum (40 hex, lascia vuoto per skip)"
    }
}
if ($CertThumbprint) {
    $CertThumbprint = $CertThumbprint.Trim().ToUpper() -replace '\s', ''
    if ($CertThumbprint.Length -ne 40 -or $CertThumbprint -notmatch '^[0-9A-F]+$') {
        Write-Host "[WARN] Thumbprint non valido (atteso 40 hex). Skip."
    } else {
        [Environment]::SetEnvironmentVariable('CERTUM_CERT_THUMBPRINT', $CertThumbprint, 'User')
        Write-Host "[OK] CERTUM_CERT_THUMBPRINT settata (User scope, persistente)"
    }
}

# 7. Setup chiave privata Tauri Updater
Write-Section "7. Chiave privata Tauri Updater"
$tauriDir = Join-Path $env:USERPROFILE '.tauri'
$keyDest = Join-Path $tauriDir 'pap-updater.key'

if (Test-Path $keyDest) {
    Write-Host "[OK] chiave gia' presente: $keyDest"
} else {
    Write-Host "Chiave NON presente in $keyDest"
    Write-Host ""
    Write-Host "  Devi copiarla da:"
    Write-Host "  - tua macchina di sviluppo principale (~/.tauri/pap-updater.key)"
    Write-Host "  - oppure dal password manager dove l'hai backuppata"
    Write-Host "  - oppure da una pen drive con il file"
    Write-Host ""
    if (-not $UpdaterKeyPath) {
        $UpdaterKeyPath = Read-Host "Path sorgente alla chiave (lascia vuoto per skip)"
    }
    if ($UpdaterKeyPath -and (Test-Path $UpdaterKeyPath)) {
        if (-not (Test-Path $tauriDir)) {
            New-Item -ItemType Directory -Path $tauriDir | Out-Null
        }
        Copy-Item -Path $UpdaterKeyPath -Destination $keyDest
        Write-Host "[OK] copiata in $keyDest"
    } elseif ($UpdaterKeyPath) {
        Write-Host "[FAIL] path non esistente: $UpdaterKeyPath"
    } else {
        Write-Host "[INFO] skip copia chiave. Copia manualmente in $keyDest poi rilancia con -SkipInstall."
    }
}

if (Test-Path $keyDest) {
    [Environment]::SetEnvironmentVariable('TAURI_UPDATER_PRIVATE_KEY_PATH', $keyDest, 'User')
    Write-Host "[OK] TAURI_UPDATER_PRIVATE_KEY_PATH settata (User scope)"

    # Password chiave (opzionale)
    $existingPwd = [Environment]::GetEnvironmentVariable('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', 'User')
    if ($null -eq $existingPwd) {
        Write-Host ""
        Write-Host "Se la chiave Tauri Updater ha una password (passphrase),"
        Write-Host "settala come env var. Lascia vuoto se la chiave non e' cifrata."
        $pwd = Read-Host "Password chiave Tauri Updater (vuoto = nessuna)"
        if ($pwd) {
            [Environment]::SetEnvironmentVariable('TAURI_SIGNING_PRIVATE_KEY_PASSWORD', $pwd, 'User')
            Write-Host "[OK] TAURI_SIGNING_PRIVATE_KEY_PASSWORD settata (User scope)"
        }
    } else {
        Write-Host "[OK] TAURI_SIGNING_PRIVATE_KEY_PASSWORD gia' settata"
    }
}

# 8. Riepilogo finale
Write-Section "8. Riepilogo"
Write-Host ""
Write-Host "Componenti:"
Write-Host "  gh CLI:        $(if(Test-CommandExists 'gh') {'OK'} else {'MISSING'})"
Write-Host "  Node.js:       $(if(Test-CommandExists 'node') {(node -v)} else {'MISSING'})"
Write-Host "  pnpm:          $(if(Test-CommandExists 'pnpm') {(pnpm -v)} else {'MISSING'})"
Write-Host "  Tauri CLI:     $(if(Test-CommandExists 'tauri') {(tauri -V)} else {'MISSING'})"
Write-Host "  signtool:      $(if($signtool) {'OK'} else {'MISSING (install Windows SDK)'})"
Write-Host ""
Write-Host "Env vars (User scope, persistenti):"
$tp = [Environment]::GetEnvironmentVariable('CERTUM_CERT_THUMBPRINT','User')
$kp = [Environment]::GetEnvironmentVariable('TAURI_UPDATER_PRIVATE_KEY_PATH','User')
$pw = [Environment]::GetEnvironmentVariable('TAURI_SIGNING_PRIVATE_KEY_PASSWORD','User')
Write-Host "  CERTUM_CERT_THUMBPRINT:              $(if($tp){$tp}else{'(non settata)'})"
Write-Host "  TAURI_UPDATER_PRIVATE_KEY_PATH:      $(if($kp){$kp}else{'(non settata)'})"
Write-Host "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD:  $(if($pw){'(settata, valore nascosto)'}else{'(non settata)'})"
Write-Host ""
Write-Host "Prossimi step:"
Write-Host "  1. CHIUDI E RIAPRI PowerShell (per caricare env vars + nuovi tool in PATH)"
Write-Host "  2. Apri SimplySign Desktop e fai login (user+pwd+TOTP)"
Write-Host "  3. cd <path-al-repo-prompt-a-porter>"
Write-Host "  4. .\scripts\sign-release.ps1 -Tag vX.Y.Z"
Write-Host ""
Write-Host "Riferimento: docs/contribuire/release-signing-workflow.md"
