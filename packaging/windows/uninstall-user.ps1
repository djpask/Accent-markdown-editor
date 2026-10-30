#Requires -Version 5.1
<#
.SYNOPSIS
    Disinstallatore User-Level per Windows 11 di Accent Markdown Editor.
#>

$ErrorActionPreference = "SilentlyContinue"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Accent"
$StartMenuShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Accent Markdown Editor.lnk"
$DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "Accent Markdown Editor.lnk"

Write-Host "==> Rimozione di Accent Markdown Editor dal profilo utente..." -ForegroundColor Cyan

# 1. Rimuovi collegamenti Menu Start e Desktop
if (Test-Path $StartMenuShortcut) { Remove-Item $StartMenuShortcut -Force }
if (Test-Path $DesktopShortcut) { Remove-Item $DesktopShortcut -Force }

# 2. Rimuovi chiavi di registro HKCU
Remove-Item -Path "HKCU:\Software\Classes\AccentMarkdownFile" -Recurse -Force
Remove-ItemProperty -Path "HKCU:\Software\Classes\.md\OpenWithProgids" -Name "AccentMarkdownFile" -Force
Remove-ItemProperty -Path "HKCU:\Software\Classes\.markdown\OpenWithProgids" -Name "AccentMarkdownFile" -Force
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AccentMarkdownEditor" -Recurse -Force

# 3. Rimuovi dal PATH utente
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath) {
    $CleanedPath = ($UserPath -split ";" | Where-Object { $_ -and ($_ -ne $InstallDir) }) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $CleanedPath, "User")
}

# 4. Rimuovi cartella %LOCALAPPDATA%\Programs\Accent
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
}

Write-Host "==> Disinstallazione di Accent completata." -ForegroundColor Green
