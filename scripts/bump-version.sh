#!/usr/bin/env bash
set -euo pipefail

# Accent Unified Version Bump Utility
# Usage: ./scripts/bump-version.sh <new-version> (e.g., 0.7.0 or 0.8.0)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ $# -ne 1 ]]; then
    echo "Utilizzo: $0 <versione> (es. 0.7.0)"
    echo "Versione attuale: $(cat "$ROOT_DIR/VERSION" | tr -d '[:space:]')"
    exit 1
fi

RAW_VER="$1"
# Normalize X.Y to X.Y.0 for SemVer Cargo compatibility
if [[ "$RAW_VER" =~ ^[0-9]+\.[0-9]+$ ]]; then
    SEMVER="${RAW_VER}.0"
elif [[ "$RAW_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    SEMVER="$RAW_VER"
else
    echo "Errore: formato versione non valido '$RAW_VER' (atteso X.Y o X.Y.Z)"
    exit 1
fi

echo "==> Aggiornamento versione di Accent a ${SEMVER}..."

# 1. VERSION file
printf "%s\n" "$SEMVER" > "$ROOT_DIR/VERSION"

# 2. Root Cargo.toml ([workspace.package] version)
sed -i -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"${SEMVER}\"/" "$ROOT_DIR/Cargo.toml"

# 3. Arch Linux PKGBUILD
if [[ -f "$ROOT_DIR/packaging/arch/PKGBUILD" ]]; then
    sed -i -E "s/^pkgver=[0-9]+\.[0-9]+\.[0-9]+/pkgver=${SEMVER}/" "$ROOT_DIR/packaging/arch/PKGBUILD"
fi

# 4. Windows 11 Inno Setup script
if [[ -f "$ROOT_DIR/packaging/windows/accent-installer.iss" ]]; then
    sed -i -E "s/^#define MyAppVersion \"[0-9]+\.[0-9]+\.[0-9]+\"/#define MyAppVersion \"${SEMVER}\"/" "$ROOT_DIR/packaging/windows/accent-installer.iss"
fi

# 5. Windows 11 PowerShell User Installer
if [[ -f "$ROOT_DIR/packaging/windows/install-user.ps1" ]]; then
    sed -i -E "s/^\$AppVersion = \"[0-9]+\.[0-9]+\.[0-9]+\"/\$AppVersion = \"${SEMVER}\"/" "$ROOT_DIR/packaging/windows/install-user.ps1"
fi

echo "==> Versione aggiornata con successo a ${SEMVER} in tutto il repository."
