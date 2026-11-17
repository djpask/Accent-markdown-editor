#!/usr/bin/env bash
set -euo pipefail

# Script per la generazione del pacchetto nativo Debian 13 "Trixie" (.deb)
# Utilizzo:
#   ./scripts/build-debian13-pkg.sh            # Genera il pacchetto .deb per Debian 13 (amd64)
#   ./scripts/build-debian13-pkg.sh --install  # Genera e installa il pacchetto su sistemi Debian con apt/dpkg

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/VERSION")"

"${ROOT_DIR}/packaging/debian/build-debian13-pkg.sh"

PKG_FILE="$(ls -1 "${ROOT_DIR}/packaging/debian"/accent-markdown-editor_"${VERSION}"-*_amd64.deb | head -n 1)"

if [[ "${1:-}" == "--install" ]]; then
    if command -v apt >/dev/null 2>&1; then
        echo "==> Installazione di ${PKG_FILE} tramite apt..."
        sudo apt install "${PKG_FILE}"
    elif command -v dpkg >/dev/null 2>&1; then
        echo "==> Installazione di ${PKG_FILE} tramite dpkg..."
        sudo dpkg -i "${PKG_FILE}"
    else
        echo "Errore: '--install' richiede un sistema Debian con 'apt' o 'dpkg' installato." >&2
        exit 1
    fi
fi
