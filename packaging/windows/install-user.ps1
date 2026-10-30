#Requires -Version 5.1
<#
.SYNOPSIS
    Pacchetto di installazione User-Level per Windows 11 di Accent Markdown Editor.
.DESCRIPTION
    Installa Accent nello spazio utente (%LOCALAPPDATA%\Programs\Accent) senza richiedere
    privilegi di Amministratore (nessun prompt UAC). Crea i collegamenti nel Menu Start,
    registra l'associazione per i file .md/.markdown in HKCU e registra l'applicazione in
    "Impostazioni di Windows 11 -> App -> App installate".
#>

$ErrorActionPreference = "Stop"

$AppName = "Accent"
$AppFullName = "Accent Markdown Editor"
$AppVersion = "0.7.0"
$Publisher = "Accent Developers"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..\..") -ErrorAction SilentlyContinue

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Accent"
$StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$ShortcutPath = Join-Path $StartMenuDir "Accent Markdown Editor.lnk"

Write-Host "==> Installazione User-Level di $AppFullName v$AppVersion per Windows 11..." -ForegroundColor Cyan
Write-Host "    Cartella di destinazione: $InstallDir (nessun privilegio Admin richiesto)"

# 1. Individuazione del binario compilato
$CandidateBinaries = @(
    (Join-Path $ScriptDir "accent.exe"),
    (Join-Path $ScriptDir "accent-ui.exe")
)
if ($RepoRoot) {
    $CandidateBinaries += (Join-Path $RepoRoot "target\release\accent-ui.exe")
    $CandidateBinaries += (Join-Path $RepoRoot "target\x86_64-pc-windows-msvc\release\accent-ui.exe")
    $CandidateBinaries += (Join-Path $RepoRoot "target\x86_64-pc-windows-gnu\release\accent-ui.exe")
}

$SourceExe = $CandidateBinaries | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $SourceExe) {
    Write-Host "Binario release non trovato. Avvio compilazione con cargo build --release -p accent-ui..." -ForegroundColor Yellow
    if (-not $RepoRoot) {
        throw "Impossibile trovare accent.exe o il repository Cargo."
    }
    Push-Location $RepoRoot
    try {
        cargo build --release -p accent-ui
    } finally {
        Pop-Location
    }
    $SourceExe = Join-Path $RepoRoot "target\release\accent-ui.exe"
}

# 2. Creazione cartella %LOCALAPPDATA%\Programs\Accent e copia file
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$TargetExe = Join-Path $InstallDir "accent.exe"
Copy-Item -Path $SourceExe -Destination $TargetExe -Force

$IconCandidates = @(
    (Join-Path $ScriptDir "accent_logo.ico")
)
if ($RepoRoot) {
    $IconCandidates += (Join-Path $RepoRoot "img\brand\accent_logo.ico")
}
$SourceIcon = $IconCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
$TargetIcon = Join-Path $InstallDir "accent_logo.ico"
if ($SourceIcon) {
    Copy-Item -Path $SourceIcon -Destination $TargetIcon -Force
}

$UninstallScriptSource = Join-Path $ScriptDir "uninstall-user.ps1"
if (Test-Path $UninstallScriptSource) {
    Copy-Item -Path $UninstallScriptSource -Destination (Join-Path $InstallDir "uninstall-user.ps1") -Force
}

# 3. Creazione collegamento nel Menu Start di Windows 11
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $TargetExe
$Shortcut.WorkingDirectory = $env:USERPROFILE
$Shortcut.Description = "Editor e visualizzatore Markdown multipiattaforma in puro Rust"
if (Test-Path $TargetIcon) {
    $Shortcut.IconLocation = "$TargetIcon,0"
}
$Shortcut.Save()

# 4. Registrazione associazione file .md e .markdown in HKCU (senza UAC)
$ProgIdKey = "HKCU:\Software\Classes\AccentMarkdownFile"
New-Item -Path "$ProgIdKey\DefaultIcon" -Force | Out-Null
New-Item -Path "$ProgIdKey\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path $ProgIdKey -Name "(Default)" -Value "Documento Markdown"
if (Test-Path $TargetIcon) {
    Set-ItemProperty -Path "$ProgIdKey\DefaultIcon" -Name "(Default)" -Value "$TargetIcon,0"
}
Set-ItemProperty -Path "$ProgIdKey\shell\open\command" -Name "(Default)" -Value "`"$TargetExe`" `"%1`""

foreach ($Ext in @(".md", ".markdown")) {
    $OpenWithKey = "HKCU:\Software\Classes\$Ext\OpenWithProgids"
    New-Item -Path $OpenWithKey -Force | Out-Null
    Set-ItemProperty -Path $OpenWithKey -Name "AccentMarkdownFile" -Value ""
}

# 5. Aggiunta al PATH utente se non già presente
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $UserPath) { $UserPath = "" }
if (($UserPath -split ";") -notcontains $InstallDir) {
    $NewUserPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
}

# 6. Registrazione in Impostazioni di Windows 11 -> App -> App installate (HKCU Uninstall)
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AccentMarkdownEditor"
New-Item -Path $UninstallKey -Force | Out-Null
Set-ItemProperty -Path $UninstallKey -Name "DisplayName" -Value $AppFullName
Set-ItemProperty -Path $UninstallKey -Name "DisplayVersion" -Value $AppVersion
Set-ItemProperty -Path $UninstallKey -Name "Publisher" -Value $Publisher
Set-ItemProperty -Path $UninstallKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty -Path $UninstallKey -Name "DisplayIcon" -Value $TargetIcon
Set-ItemProperty -Path $UninstallKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall-user.ps1`""
Set-ItemProperty -Path $UninstallKey -Name "NoModify" -Value 1 -Type DWord
Set-ItemProperty -Path $UninstallKey -Name "NoRepair" -Value 1 -Type DWord

Write-Host "==> Installazione completata con successo!" -ForegroundColor Green
Write-Host "    Eseguibile: $TargetExe"
Write-Host "    Menu Start: $ShortcutPath"
