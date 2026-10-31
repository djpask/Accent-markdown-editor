#!/usr/bin/env bash
set -euo pipefail

# Script per la generazione dell'Installatore in modalità utente per Windows 11
# con Disinstallatore integrato (Accent-Setup-<version>-x86_64.exe + bundle .zip).
# Utilizzo:
#   ./scripts/build-windows-installer.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WIN_DIR="${ROOT_DIR}/packaging/windows"
DIST_DIR="${WIN_DIR}/dist"
VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/VERSION")"

# 1. Cross-compila accent.exe e l'installer/disinstallatore Accent-Setup-<version>-x86_64.exe
"${WIN_DIR}/build-windows-from-linux.sh"

# 2. Crea il bundle ZIP completo per Windows 11 (con Setup .exe, binario e script PowerShell User-Mode)
mkdir -p "${DIST_DIR}"
STAGING_DIR="${DIST_DIR}/Accent-${VERSION}-win11-x64-user"
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

cp "${WIN_DIR}/Accent-Setup-${VERSION}-x86_64.exe" "${STAGING_DIR}/"
cp "${WIN_DIR}/accent.exe" "${STAGING_DIR}/"
cp "${ROOT_DIR}/img/brand/accent_logo.ico" "${STAGING_DIR}/"
cp "${ROOT_DIR}/img/brand/accent_logo.png" "${STAGING_DIR}/"
cp "${WIN_DIR}/install-user.cmd" "${STAGING_DIR}/"
cp "${WIN_DIR}/install-user.ps1" "${STAGING_DIR}/"
cp "${WIN_DIR}/uninstall-user.ps1" "${STAGING_DIR}/"
cp "${ROOT_DIR}/README.md" "${STAGING_DIR}/"
cp "${ROOT_DIR}/VERSION" "${STAGING_DIR}/"

ZIP_FILE="${DIST_DIR}/Accent-${VERSION}-win11-x64-user-installer.zip"
rm -f "${ZIP_FILE}"
if command -v zip >/dev/null 2>&1; then
    (cd "${STAGING_DIR}" && zip -qr "${ZIP_FILE}" .)
elif command -v bsdtar >/dev/null 2>&1; then
    (cd "${STAGING_DIR}" && bsdtar -a -cf "${ZIP_FILE}" .)
fi

echo "==> Artefatti Windows 11 (User-Mode + Disinstallatore integrato) pronti:"
echo "    Setup .exe autonomo: ${WIN_DIR}/Accent-Setup-${VERSION}-x86_64.exe"
echo "    Binario standalone:  ${WIN_DIR}/accent.exe"
if [[ -f "${ZIP_FILE}" ]]; then
    echo "    Archivio ZIP:        ${ZIP_FILE}"
fi
