#Requires -Version 5.1
<#
.SYNOPSIS
    Script di creazione del pacchetto di installazione User-Level per Windows 11.
.DESCRIPTION
    Compila il binario release di Accent su Windows 11, crea il bundle ZIP di installazione
    utente (con install-user.cmd / install-user.ps1) e, se Inno Setup 6 (ISCC.exe) e' presente
    sul sistema, genera anche l'installer grafico autonomo Accent-0.7.0-win11-x64-user-setup.exe.
#>

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..\..")
$Version = (Get-Content (Join-Path $RepoRoot "VERSION") -Raw).Trim()
$DistDir = Join-Path $ScriptDir "dist"

Write-Host "==> Compilazione di Accent v$Version per Windows 11 (x86_64)..." -ForegroundColor Cyan
Push-Location $RepoRoot
try {
    cargo test --workspace
    cargo build --release --locked -p accent-ui
} finally {
    Pop-Location
}

New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
$StagingDir = Join-Path $DistDir "Accent-$Version-win11-x64-user"
if (Test-Path $StagingDir) { Remove-Item -Recurse -Force $StagingDir }
New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null

# Copia artefatti nel bundle user-installer
$BuiltExe = Join-Path $RepoRoot "target\release\accent-ui.exe"
$LocalExe = Join-Path $ScriptDir "accent.exe"
Copy-Item $BuiltExe $LocalExe -Force
Copy-Item $BuiltExe (Join-Path $StagingDir "accent.exe") -Force
Copy-Item (Join-Path $RepoRoot "img\brand\accent_logo.ico") (Join-Path $StagingDir "accent_logo.ico") -Force
Copy-Item (Join-Path $RepoRoot "img\brand\accent_logo.png") (Join-Path $StagingDir "accent_logo.png") -Force
Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $StagingDir "README.md") -Force
Copy-Item (Join-Path $RepoRoot "VERSION") (Join-Path $StagingDir "VERSION") -Force
Copy-Item (Join-Path $ScriptDir "install-user.ps1") (Join-Path $StagingDir "install-user.ps1") -Force
Copy-Item (Join-Path $ScriptDir "uninstall-user.ps1") (Join-Path $StagingDir "uninstall-user.ps1") -Force
Copy-Item (Join-Path $ScriptDir "install-user.cmd") (Join-Path $StagingDir "install-user.cmd") -Force

# Compila l'installer/disinstallatore nativo in puro Rust (Accent-Setup-<Version>-x86_64.exe)
$SetupExe = Join-Path $ScriptDir "Accent-Setup-$Version-x86_64.exe"
$env:ACCENT_VERSION = $Version
& rustc --edition 2024 -O (Join-Path $ScriptDir "installer.rs") -o $SetupExe
Copy-Item $SetupExe (Join-Path $StagingDir "Accent-Setup-$Version-x86_64.exe") -Force
Write-Host "==> Creato Installer/Disinstallatore .exe User-Level in puro Rust:" -ForegroundColor Green
Write-Host "    $SetupExe"

$ZipPath = Join-Path $DistDir "Accent-$Version-win11-x64-user-installer.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path "$StagingDir\*" -DestinationPath $ZipPath

Write-Host "==> Creato pacchetto ZIP User-Level per Windows 11:" -ForegroundColor Green
Write-Host "    $ZipPath"

# Se Inno Setup 6 (ISCC.exe) e' installato, genera anche il setup .exe
$IsccCandidates = @(
    "ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
)
$IsccExe = $IsccCandidates | Where-Object { Get-Command $_ -ErrorAction SilentlyContinue } | Select-Object -First 1

if ($IsccExe) {
    Write-Host "==> Trovato Inno Setup ($IsccExe): compilazione dell'installer .exe User-Level..." -ForegroundColor Cyan
    & $IsccExe (Join-Path $ScriptDir "accent-installer.iss")
    Write-Host "==> Creato installer grafico User-Level per Windows 11:" -ForegroundColor Green
    Write-Host "    $(Join-Path $DistDir "Accent-$Version-win11-x64-user-setup.exe")"
} else {
    Write-Host "Nota: Inno Setup 6 (ISCC.exe) non rilevato nel PATH; il pacchetto ZIP con install-user.cmd e' pronto in $ZipPath." -ForegroundColor Yellow
}
