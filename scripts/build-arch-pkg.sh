#!/usr/bin/env bash
set -euo pipefail

# Script per la generazione del pacchetto nativo Arch Linux (.pkg.tar.zst)
# Utilizzo:
#   ./scripts/build-arch-pkg.sh            # Genera il pacchetto .pkg.tar.zst
#   ./scripts/build-arch-pkg.sh --install  # Genera e installa il pacchetto con pacman -U

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/VERSION")"

"${ROOT_DIR}/packaging/arch/build-arch-pkg.sh"

PKG_FILE="$(ls -1 "${ROOT_DIR}/packaging/arch"/accent-markdown-editor-"${VERSION}"-*.pkg.tar.* | head -n 1)"

if [[ "${1:-}" == "--install" ]]; then
    echo "==> Installazione di ${PKG_FILE} tramite pacman..."
    sudo pacman -U "${PKG_FILE}"
fi
