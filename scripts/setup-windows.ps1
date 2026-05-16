<#
.SYNOPSIS
  Setup guidato workstation Windows per signing release Prompt a Porter.

.DESCRIPTION
  Verifica e installa tutti i componenti necessari per eseguire
  scripts\sign-release.ps1 su una macchina Windows 10/11 (anche
  IoT Enterprise). Configura env vars permanenti per il maintainer.

  Componenti gestiti (preferenza: winget se disponibile, fallback
  download diretto MSI per Windows IoT / Server senza Microsoft Store):
   - GitHub CLI (gh)             -> winget GitHub.cli | MSI da api.github.com
   - SimplySign Desktop          -> winget Certum.SmartSignSimplySignDesktop
                                    (se winget assente: install manuale guidato)
   - Node.js LTS                 -> winget OpenJS.NodeJS.LTS | MSI da nodejs.org
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

# Fallback per ambienti senza winget (es. Windows IoT Enterprise senza
# Microsoft Store). Scarica MSI direttamente e installa via msiexec.
# Usa -Verb RunAs per triggerare UAC: la maggior parte degli MSI
# installa in Program Files che richiede elevation (senza, exit 1603).
function Install-MsiFromUrl {
    param(
        [string]$Url,
        [string]$DisplayName
    )
    $rand = [System.IO.Path]::GetRandomFileName()
    $tmpMsi = Join-Path $env:TEMP "setup-$rand.msi"
    $logFile = Join-Path $env:TEMP "msi-install-$rand.log"
    Write-Host "  Download MSI: $Url"
    Invoke-WebRequest -Uri $Url -OutFile $tmpMsi -UseBasicParsing
    Write-Host "  Installazione msiexec (UAC prompt se richiesto, accetta)..."
    try {
        $proc = Start-Process -FilePath msiexec.exe `
            -ArgumentList "/i", "`"$tmpMsi`"", "/quiet", "/norestart", "/L*v", "`"$logFile`"" `
            -Verb RunAs -Wait -PassThru
    } catch {
        Remove-Item -Path $tmpMsi -Force -ErrorAction SilentlyContinue
        throw "msiexec lancio fallito (UAC negato?): $($_.Exception.Message)"
    }
    # 0 = OK; 3010 = OK, reboot required
    if ($proc.ExitCode -eq 0 -or $proc.ExitCode -eq 3010) {
        Remove-Item -Path $tmpMsi -Force -ErrorAction SilentlyContinue
        Remove-Item -Path $logFile -Force -ErrorAction SilentlyContinue
        Write-Host "  [OK] $DisplayName installato (exit $($proc.ExitCode))"
        return
    }
    # Fail: tengo MSI + log per debug
    Write-Host ""
    Write-Host "  [FAIL] msiexec exit $($proc.ExitCode)"
    Write-Host "    MSI scaricata (conservata):   $tmpMsi"
    Write-Host "    Log dettagliato msiexec:      $logFile"
    Write-Host "    Per la causa specifica cerca 'return value 3' nel log,"
    Write-Host "    oppure: Select-String -Path '$logFile' -Pattern 'Error|return value 3'"
    Write-Host ""
    Write-Host "    Workaround: doppio click sull'MSI per install GUI manuale."
    Write-Host ""
    throw "msiexec install fallito per $DisplayName (exit $($proc.ExitCode)). Vedi log: $logFile"
}

function Get-LatestGhMsiUrl {
    Write-Host "  Resolve URL MSI gh CLI latest..."
    $rel = Invoke-RestMethod -Uri 'https://api.github.com/repos/cli/cli/releases/latest' -Headers @{ 'User-Agent' = 'pap-setup' }
    $asset = $rel.assets | Where-Object { $_.name -like '*windows_amd64.msi' } | Select-Object -First 1
    if (-not $asset) { throw "Asset MSI gh CLI non trovato nella latest release" }
    return $asset.browser_download_url
}

function Get-LatestNodeLtsMsiUrl {
    Write-Host "  Resolve URL MSI Node.js LTS..."
    $idx = Invoke-RestMethod -Uri 'https://nodejs.org/dist/index.json' -Headers @{ 'User-Agent' = 'pap-setup' }
    $lts = $idx | Where-Object { $_.lts -and $_.lts -ne $false } | Select-Object -First 1
    if (-not $lts) { throw "Nessuna versione LTS Node.js trovata" }
    return "https://nodejs.org/dist/$($lts.version)/node-$($lts.version)-x64.msi"
}

# 1. Preflight: verifica winget (opzionale - fallback con download diretto MSI
#    per ambienti senza Microsoft Store, es. Windows IoT Enterprise)
Write-Section "1. Preflight"
$useWinget = Test-CommandExists 'winget'
if ($useWinget) {
    Write-Host "[OK] winget presente -> uso winget per gli install"
} else {
    Write-Host "[INFO] winget NON disponibile (probabile Windows IoT senza Store)"
    Write-Host "       Uso fallback: download MSI diretto + msiexec /quiet"
}

if ($SkipInstall) {
    Write-Host "[INFO] -SkipInstall passato: salto installazioni"
} else {
    Write-Section "2. Install software"

    # 2a. GitHub CLI
    if (Test-CommandExists 'gh') {
        Write-Host "[OK] gh CLI gia' presente: $((gh --version | Select-Object -First 1))"
    } elseif ($useWinget) {
        Install-WinGetPackage -PackageId 'GitHub.cli' -DisplayName 'GitHub CLI'
    } else {
        Write-Host "Installo GitHub CLI via MSI diretto..."
        Install-MsiFromUrl -Url (Get-LatestGhMsiUrl) -DisplayName 'GitHub CLI'
    }

    # 2b. SimplySign Desktop
    # Difficile da fallback (URL Certum versionato, soggetto a 404). Se
    # winget non disponibile e SimplySign non presente, lascio install
    # manuale all'utente con istruzioni.
    $simplySignExe = (Get-ChildItem -Path "C:\Program Files (x86)","C:\Program Files","$env:LOCALAPPDATA\Programs" `
        -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "*SimplySign*" } | Select-Object -First 1)
    if ($simplySignExe) {
        Write-Host "[OK] SimplySign Desktop gia' presente: $($simplySignExe.FullName)"
    } elseif ($useWinget) {
        Install-WinGetPackage -PackageId 'Certum.SmartSignSimplySignDesktop' -DisplayName 'SimplySign Desktop'
    } else {
        Write-Host "[WARN] SimplySign Desktop NON installato e winget non disponibile."
        Write-Host "       Scaricalo da:"
        Write-Host "         https://www.certum.eu/en/cert_expertise_signature_cards_drivers_simplysign_desktop/"
        Write-Host "       (download .msi richiede registrazione)."
    }

    # 2c. Node.js LTS
    if (Test-CommandExists 'node') {
        Write-Host "[OK] Node.js gia' presente: $(node -v)"
    } elseif ($useWinget) {
        Install-WinGetPackage -PackageId 'OpenJS.NodeJS.LTS' -DisplayName 'Node.js LTS'
    } else {
        Write-Host "Installo Node.js LTS via MSI diretto..."
        Install-MsiFromUrl -Url (Get-LatestNodeLtsMsiUrl) -DisplayName 'Node.js LTS'
    }

    # Refresh PATH per la sessione corrente (gli installer aggiungono a
    # User/Machine PATH solo per nuove shell; per usare node/npm subito
    # qui ricarichiamo).
    $env:PATH = [Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' +
                [Environment]::GetEnvironmentVariable('PATH', 'User')

    Write-Section "3. Setup pnpm + Tauri CLI"
    if (-not (Test-CommandExists 'corepack')) {
        Write-Host "[WARN] corepack non trovato in PATH. Riapri PowerShell e rilancia (Node appena installato)."
        Write-Host "       Workaround: chiudi questo terminale, riapri, rilancia: .\scripts\setup-windows.ps1 -SkipInstall"
        exit 0
    }

    # corepack enable scrive shim in C:\Program Files\nodejs\ -> richiede
    # admin. Lo lancio in sub-shell elevated via -Verb RunAs (UAC prompt).
    Write-Host "Abilito pnpm via corepack (UAC prompt: serve admin per scrivere in Program Files)..."
    $corepackCmd = 'corepack enable; corepack prepare pnpm@latest --activate'
    try {
        $proc = Start-Process -FilePath 'powershell' `
            -ArgumentList '-NoProfile', '-Command', $corepackCmd `
            -Verb RunAs -Wait -PassThru
    } catch {
        throw "Lancio corepack elevated fallito (UAC negato?): $($_.Exception.Message)"
    }
    if ($proc.ExitCode -ne 0) {
        throw "corepack enable fallito (exit $($proc.ExitCode)). Workaround: apri PowerShell come Admin e lancia manualmente: corepack enable; corepack prepare pnpm@latest --activate"
    }

    # Refresh PATH per intercettare pnpm shim appena creato in Program Files\nodejs
    $env:PATH = [Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' +
                [Environment]::GetEnvironmentVariable('PATH', 'User')

    if (-not (Test-CommandExists 'pnpm')) {
        throw "pnpm non disponibile dopo corepack enable elevated. Riapri PowerShell e rilancia con -SkipInstall."
    }
    Write-Host "[OK] pnpm: $(pnpm -v)"

    # pnpm alla prima esecuzione richiede PNPM_HOME settata + PNPM_HOME\bin
    # in PATH per i global package bin (tauri.cmd, ecc.). 'pnpm setup' fa
    # tutto ma ha output interattivo variabile; lo facciamo manuale.
    $pnpmHome = Join-Path $env:LOCALAPPDATA 'pnpm'
    $pnpmBin = Join-Path $pnpmHome 'bin'
    foreach ($dir in @($pnpmHome, $pnpmBin)) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    [Environment]::SetEnvironmentVariable('PNPM_HOME', $pnpmHome, 'User')

    # Aggiorna User PATH includendo sia PNPM_HOME (per cache pnpm e tool
    # che cercano qui) sia PNPM_HOME\bin (richiesto da pnpm per global bin).
    $userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
    if ($null -eq $userPath) { $userPath = '' }
    $pathParts = @($userPath -split ';' | Where-Object { $_ })
    $added = @()
    foreach ($dir in @($pnpmHome, $pnpmBin)) {
        if ($pathParts -notcontains $dir) {
            $pathParts += $dir
            $added += $dir
        }
    }
    if ($added.Count -gt 0) {
        $newUserPath = ($pathParts -join ';')
        [Environment]::SetEnvironmentVariable('PATH', $newUserPath, 'User')
        Write-Host "[OK] aggiunto a User PATH: $($added -join ', ')"
    }

    # Refresh PATH della sessione corrente (senza riavvio shell)
    $env:PNPM_HOME = $pnpmHome
    foreach ($dir in @($pnpmHome, $pnpmBin)) {
        if (-not ($env:PATH -like "*$dir*")) {
            $env:PATH = "$env:PATH;$dir"
        }
    }

    Write-Host "Installo @tauri-apps/cli globalmente..."
    pnpm i -g @tauri-apps/cli
    if ($LASTEXITCODE -ne 0) {
        throw "pnpm i -g @tauri-apps/cli fallito (exit $LASTEXITCODE)"
    }
    if (-not (Test-CommandExists 'tauri')) {
        Write-Host "[WARN] 'tauri' non ancora in PATH della sessione corrente."
        Write-Host "       L'install e' OK ($pnpmHome\tauri.cmd dovrebbe esistere)"
        Write-Host "       Chiudi e riapri PowerShell, poi: tauri -V"
    } else {
        Write-Host "[OK] tauri: $(tauri -V)"
    }
}

# 4. Verifica signtool (Windows SDK) + install automatico del solo
#    componente "Windows SDK Signing Tools for Desktop Apps" (~30 MB)
function Find-Signtool {
    $found = (Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
              Sort-Object -Property FullName -Descending |
              Select-Object -First 1).FullName
    if (-not $found) {
        $found = (Get-ChildItem -Path "$env:ProgramFiles\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
                  Sort-Object -Property FullName -Descending |
                  Select-Object -First 1).FullName
    }
    return $found
}

function Install-WinSdkSigningTools {
    # Microsoft pubblica un bootstrapper che permette di selezionare solo
    # i componenti voluti via /features OptionId.SigningTools (~30 MB).
    # URL stabile: fwlink che redireziona al Windows 11 SDK installer
    # corrente. Alternativa documentata: https://aka.ms/windowssdk
    $url = 'https://go.microsoft.com/fwlink/?linkid=2286561'
    $tmpExe = Join-Path $env:TEMP ("winsdksetup-" + [System.IO.Path]::GetRandomFileName() + ".exe")
    Write-Host "  Download Windows SDK bootstrapper..."
    Invoke-WebRequest -Uri $url -OutFile $tmpExe -UseBasicParsing
    Write-Host "  Install OptionId.SigningTools (UAC prompt: accetta)..."
    try {
        $proc = Start-Process -FilePath $tmpExe `
            -ArgumentList '/quiet', '/norestart', '/features', 'OptionId.SigningTools' `
            -Verb RunAs -Wait -PassThru
    } catch {
        Remove-Item -Path $tmpExe -Force -ErrorAction SilentlyContinue
        throw "Lancio Windows SDK installer fallito (UAC negato?): $($_.Exception.Message)"
    }
    Remove-Item -Path $tmpExe -Force -ErrorAction SilentlyContinue
    # 0 = OK; 3010 = OK, reboot required
    if ($proc.ExitCode -ne 0 -and $proc.ExitCode -ne 3010) {
        throw "Windows SDK Signing Tools install fallito (exit $($proc.ExitCode))"
    }
    Write-Host "  [OK] Windows SDK Signing Tools installato (exit $($proc.ExitCode))"
}

Write-Section "4. Windows SDK signtool"
$signtool = Find-Signtool
if ($signtool) {
    Write-Host "[OK] signtool: $signtool"
} elseif ($SkipInstall) {
    Write-Host "[INFO] signtool non presente, ma -SkipInstall attivo. Skip install."
    Write-Host "       Per installare manualmente:"
    Write-Host "         https://developer.microsoft.com/windows/downloads/windows-sdk/"
    Write-Host "         (seleziona solo 'Windows SDK Signing Tools for Desktop Apps')"
} else {
    Write-Host "[INFO] signtool NON trovato. Installo Windows SDK Signing Tools (~30 MB)..."
    Install-WinSdkSigningTools
    $signtool = Find-Signtool
    if ($signtool) {
        Write-Host "[OK] signtool: $signtool"
    } else {
        Write-Host "[WARN] signtool ancora non trovato dopo install. Riapri PowerShell e rilancia con -SkipInstall."
    }
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
