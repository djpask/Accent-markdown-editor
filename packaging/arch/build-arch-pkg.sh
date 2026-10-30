#!/usr/bin/env bash
set -euo pipefail

# Build script for the Arch Linux installation package (.pkg.tar.zst)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/VERSION")"

echo "==> Building Accent v${VERSION} Arch Linux package..."
cd "${SCRIPT_DIR}"

# Clean previous makepkg build artifacts
rm -rf "${SCRIPT_DIR}/src" "${SCRIPT_DIR}/pkg"

# Build the package using makepkg
makepkg -sf --noconfirm

PKG_FILE="$(ls -1 "${SCRIPT_DIR}"/accent-markdown-editor-"${VERSION}"-*.pkg.tar.* | head -n 1)"
echo "==> Pacchetto Arch Linux creato con successo:"
echo "    ${PKG_FILE}"
echo ""
echo "Per installarlo sul sistema esegui:"
echo "    sudo pacman -U \"${PKG_FILE}\""
