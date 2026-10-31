#!/usr/bin/env bash
set -euo pipefail

# Cross-compiles the native Windows 11 GUI binary (accent.exe) and the self-contained
# Windows 11 Local Installer + Uninstaller (Accent-Setup-<version>-x86_64.exe) from Arch Linux.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VERSION="$(tr -d '[:space:]' < "${REPO_ROOT}/VERSION")"

echo "==> Building Accent v${VERSION} for Windows 11 (x86_64-pc-windows-gnu)..."

SYSROOT_DIR="/tmp/mingw-sysroot"
if [[ ! -f "${SYSROOT_DIR}/usr/x86_64-w64-mingw32/lib/crt2.o" ]] || [[ ! -f "${SYSROOT_DIR}/usr/bin/x86_64-w64-mingw32-dlltool" ]]; then
    echo "    Downloading mingw-w64 sysroot components into ${SYSROOT_DIR}..."
    mkdir -p "${SYSROOT_DIR}"
    for pkg in mingw-w64-crt mingw-w64-winpthreads mingw-w64-binutils; do
        url="$(pacman -Sp "$pkg")"
        curl -sL "$url" | tar -C "${SYSROOT_DIR}" --zstd -xf -
    done
    gcc_url="$(pacman -Sp mingw-w64-gcc)"
    curl -sL "$gcc_url" | tar -C "${SYSROOT_DIR}" --zstd -xf - --wildcards 'usr/lib/gcc/x86_64-w64-mingw32/*/libgcc*.a'
    cp "${SYSROOT_DIR}"/usr/lib/gcc/x86_64-w64-mingw32/*/libgcc*.a "${SYSROOT_DIR}/usr/x86_64-w64-mingw32/lib/"
fi

rustup target add x86_64-pc-windows-gnu >/dev/null 2>&1 || true

LINKER_WRAPPER="/tmp/x86_64-w64-mingw32-gcc"
cat << 'EOF' > "${LINKER_WRAPPER}"
#!/usr/bin/env bash
SYSROOT_LLD="$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld"
MINGW_LIB="/tmp/mingw-sysroot/usr/x86_64-w64-mingw32/lib"
ARGS=()
IS_SHARED=0
for arg in "$@"; do
    case "$arg" in
        -Wl,*)
            IFS=',' read -ra PARTS <<< "${arg#-Wl,}"
            for p in "${PARTS[@]}"; do
                ARGS+=("$p")
            done
            ;;
        -mwindows)
            ARGS+=("--subsystem" "windows")
            ;;
        -mconsole)
            ARGS+=("--subsystem" "console")
            ;;
        -mdll|-shared)
            IS_SHARED=1
            ARGS+=("--shared")
            ;;
        -nolibc|--unwindlib=*|-m64|-nostartfiles|-nodefaultlibs|-fno-*|-f*)
            ;;
        *)
            ARGS+=("$arg")
            ;;
    esac
done
if [[ "$IS_SHARED" -eq 0 ]]; then
    exec "$SYSROOT_LLD" -m i386pep "$MINGW_LIB/crt2.o" -L"$MINGW_LIB" "${ARGS[@]}"
else
    exec "$SYSROOT_LLD" -m i386pep "$MINGW_LIB/dllcrt2.o" -L"$MINGW_LIB" "${ARGS[@]}"
fi
EOF
chmod +x "${LINKER_WRAPPER}"

export PATH="${SYSROOT_DIR}/usr/bin:${PATH}"
cd "${REPO_ROOT}"

CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="${LINKER_WRAPPER}" \
    cargo build --release --target x86_64-pc-windows-gnu -p accent-ui

x86_64-w64-mingw32-strip "${REPO_ROOT}/target/x86_64-pc-windows-gnu/release/accent-ui.exe"
cp "${REPO_ROOT}/target/x86_64-pc-windows-gnu/release/accent-ui.exe" "${SCRIPT_DIR}/accent.exe"

SETUP_EXE="${SCRIPT_DIR}/Accent-Setup-${VERSION}-x86_64.exe"
ACCENT_VERSION="${VERSION}" rustc --edition 2024 -O \
    --target x86_64-pc-windows-gnu \
    -C linker="${LINKER_WRAPPER}" \
    "${SCRIPT_DIR}/installer.rs" \
    -o "${SETUP_EXE}"
x86_64-w64-mingw32-strip "${SETUP_EXE}"

echo "==> Build Windows 11 completata:"
echo "    Binario standalone:      ${SCRIPT_DIR}/accent.exe"
echo "    Setup & Disinstallatore: ${SETUP_EXE}"
