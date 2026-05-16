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

    [switch]$KeepWorkDir
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

# 3. Download asset firmabili
$signablePatterns = @('*.exe', '*.msi')
$zipPattern = '*portable*.zip'

Write-Host ""
Write-Host "Scarico asset firmabili (.exe, .msi, .zip portable)..."
foreach ($pattern in @($signablePatterns + $zipPattern)) {
    & gh release download $Tag --repo $Repo --pattern $pattern --skip-existing 2>&1 | Out-Null
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

# 9. Re-upload firmati con clobber
Write-Host ""
Write-Host "-- Re-upload asset firmati (clobber) --"
$toUpload = $downloaded | ForEach-Object { $_.FullName }
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
    $editArgs = @('release', 'edit', $Tag, '--repo', $Repo, '--draft=false')
    if (-not $release.isPrerelease) {
        $editArgs += '--latest'
    }
    & gh @editArgs
    if ($LASTEXITCODE -ne 0) {
        throw "gh release edit FALLITO (exit $LASTEXITCODE)"
    }
    Write-Host "[OK] Release $Tag pubblicata."
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
