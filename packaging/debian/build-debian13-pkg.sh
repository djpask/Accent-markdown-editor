#!/usr/bin/env bash
set -euo pipefail

# Build script for the native Debian 13 ("Trixie", amd64) package (.deb).
# Supports both:
#   1. Native builds directly on Debian 13 (Trixie) / glibc <= 2.41 hosts.
#   2. Hermetic sysroot builds from newer Linux distros (e.g. Arch Linux with glibc > 2.41),
#      linking directly against SHA256-verified official Debian 13 (trixie) libraries so
#      the produced binary is 100% ABI-compatible with Debian 13 (libc6 2.41).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VERSION="$(tr -d '[:space:]' < "${REPO_ROOT}/VERSION")"
PKG_NAME="accent-markdown-editor"
DEB_REVISION="1~deb13u1"
DEB_VERSION="${VERSION}-${DEB_REVISION}"
DEB_ARCH="amd64"
DEB_FILENAME="${PKG_NAME}_${DEB_VERSION}_${DEB_ARCH}.deb"
DEB_OUTPUT="${SCRIPT_DIR}/${DEB_FILENAME}"
DEBIAN_MIRROR="${DEBIAN_MIRROR:-https://deb.debian.org/debian}"
DEBIAN_SUITE="trixie"
TARGET_MAX_GLIBC="2.41"

echo "==> Creazione pacchetto Debian 13 (${DEBIAN_SUITE}) per Accent v${VERSION} (${DEB_ARCH})..."

HOST_ARCH="$(uname -m)"
if [[ "${HOST_ARCH}" != "x86_64" ]]; then
    echo "Errore: architettura host '${HOST_ARCH}' non supportata (atteso x86_64 / amd64)." >&2
    exit 1
fi

# Sincronizza Cargo.lock offline con la versione corrente del workspace
(cd "${REPO_ROOT}" && cargo update --workspace --offline)

# Rileva la versione di glibc dell'host per decidere se compilare nativamente o tramite sysroot Debian 13
HOST_GLIBC="$(ldd --version 2>/dev/null | awk 'NR==1 && match($0, /[0-9]+\.[0-9]+$/) { v = substr($0, RSTART, RLENGTH) } END { print (v ? v : "99.99") }')"
USE_SYSROOT=0
if [[ "${FORCE_DEBIAN13_SYSROOT:-0}" == "1" ]]; then
    USE_SYSROOT=1
elif [[ "$(printf '%s\n%s\n' "${TARGET_MAX_GLIBC}" "${HOST_GLIBC}" | sort -V | tail -n 1)" != "${TARGET_MAX_GLIBC}" ]]; then
    USE_SYSROOT=1
fi

prepare_debian13_sysroot() {
    local sysroot_dir="$1"
    local stamp_file="${sysroot_dir}/.debian13_trixie_sysroot_ready_v2"

    if [[ -f "${stamp_file}" ]] \
        && [[ -f "${sysroot_dir}/usr/lib/x86_64-linux-gnu/Scrt1.o" ]] \
        && [[ -f "${sysroot_dir}/usr/lib/x86_64-linux-gnu/libc.so.6" ]] \
        && [[ -f "${sysroot_dir}/usr/lib/x86_64-linux-gnu/libgtk-3.so.0" ]] \
        && [[ -f "${sysroot_dir}/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0" ]]; then
        return 0
    fi

    echo "==> Host glibc (${HOST_GLIBC}) > Debian 13 glibc (${TARGET_MAX_GLIBC}): preparazione sysroot ermetico Debian 13 (${DEBIAN_SUITE})..."
    rm -rf "${sysroot_dir}"
    mkdir -p "${sysroot_dir}/usr/lib/x86_64-linux-gnu" "${sysroot_dir}/usr/lib64"
    ln -sfn usr/lib "${sysroot_dir}/lib"
    ln -sfn usr/lib64 "${sysroot_dir}/lib64"

    python3 - "${DEBIAN_MIRROR}" "${DEBIAN_SUITE}" "${sysroot_dir}" << 'PYEOF'
import hashlib
import lzma
import os
import re
import subprocess
import sys
import tempfile
import urllib.request

mirror, suite, sysroot = sys.argv[1], sys.argv[2], sys.argv[3]
required_pkgs = [
    "libc6",
    "libc6-dev",
    "libgcc-s1",
    "libglib2.0-0t64",
    "libgtk-3-0t64",
    "libgdk-pixbuf-2.0-0",
    "libcairo2",
    "libdbus-1-3",
    "libsoup-3.0-0",
    "libjavascriptcoregtk-4.1-0",
    "libwebkit2gtk-4.1-0",
]

idx_url = f"{mirror}/dists/{suite}/main/binary-amd64/Packages.xz"
print(f"    Scaricamento indice pacchetti: {idx_url}")
with urllib.request.urlopen(idx_url, timeout=60) as resp:
    raw_index = lzma.decompress(resp.read()).decode("utf-8", errors="replace")

found = {}
for stanza in raw_index.split("\n\n"):
    m_pkg = re.search(r"^Package:\s*(\S+)", stanza, re.M)
    if not m_pkg:
        continue
    pkg = m_pkg.group(1)
    if pkg in required_pkgs and pkg not in found:
        m_fn = re.search(r"^Filename:\s*(\S+)", stanza, re.M)
        m_sha = re.search(r"^SHA256:\s*(\S+)", stanza, re.M)
        m_ver = re.search(r"^Version:\s*(\S+)", stanza, re.M)
        if m_fn and m_sha:
            found[pkg] = (m_fn.group(1), m_sha.group(1), m_ver.group(1) if m_ver else "?")

missing = [p for p in required_pkgs if p not in found]
if missing:
    raise SystemExit(f"Errore: pacchetti mancanti nell'indice Debian {suite}: {missing}")

with tempfile.TemporaryDirectory() as tmpdir:
    for pkg in required_pkgs:
        rel_path, expected_sha, ver = found[pkg]
        url = f"{mirror}/{rel_path}"
        deb_file = os.path.join(tmpdir, f"{pkg}.deb")
        print(f"    Scaricamento {pkg} ({ver})...")
        with urllib.request.urlopen(url, timeout=60) as resp:
            data = resp.read()
        actual_sha = hashlib.sha256(data).hexdigest()
        if actual_sha != expected_sha:
            raise SystemExit(f"Errore SHA256 per {pkg}: atteso {expected_sha}, ottenuto {actual_sha}")
        with open(deb_file, "wb") as f:
            f.write(data)

        # Estrai data.tar.* dal pacchetto .deb verificato dentro il sysroot
        ar_list = subprocess.check_output(["ar", "t", deb_file], text=True).splitlines()
        data_member = next((m for m in ar_list if m.startswith("data.tar")), None)
        if not data_member:
            raise SystemExit(f"Errore: nessun membro data.tar trovato in {pkg}.deb")
        p_ar = subprocess.Popen(["ar", "p", deb_file, data_member], stdout=subprocess.PIPE)
        tar_flag = "-xJf" if data_member.endswith(".xz") else ("--zstd" if data_member.endswith(".zst") else "-xzf")
        if tar_flag == "--zstd":
            cmd = ["tar", "--zstd", "-xf", "-", "-C", sysroot]
        else:
            cmd = ["tar", tar_flag, "-", "-C", sysroot]
        rc = subprocess.call(cmd, stdin=p_ar.stdout)
        p_ar.wait()
        if rc != 0 or p_ar.returncode != 0:
            raise SystemExit(f"Errore durante l'estrazione di {pkg}.deb")
PYEOF

    local libdir="${sysroot_dir}/usr/lib/x86_64-linux-gnu"
    ln -sfn libgtk-3.so.0 "${libdir}/libgtk-3.so"
    ln -sfn libgdk-3.so.0 "${libdir}/libgdk-3.so"
    ln -sfn libglib-2.0.so.0 "${libdir}/libglib-2.0.so"
    ln -sfn libgobject-2.0.so.0 "${libdir}/libgobject-2.0.so"
    ln -sfn libgio-2.0.so.0 "${libdir}/libgio-2.0.so"
    ln -sfn libgdk_pixbuf-2.0.so.0 "${libdir}/libgdk_pixbuf-2.0.so"
    ln -sfn libcairo.so.2 "${libdir}/libcairo.so"
    ln -sfn libdbus-1.so.3 "${libdir}/libdbus-1.so"
    ln -sfn libsoup-3.0.so.0 "${libdir}/libsoup-3.0.so"
    ln -sfn libjavascriptcoregtk-4.1.so.0 "${libdir}/libjavascriptcoregtk-4.1.so"
    ln -sfn libwebkit2gtk-4.1.so.0 "${libdir}/libwebkit2gtk-4.1.so"
    printf '/* GNU ld script */\nGROUP ( /lib/x86_64-linux-gnu/libgcc_s.so.1 -lgcc )\n' > "${libdir}/libgcc_s.so"

    # Crea archivi statici vuoti per le dipendenze transitive GTK non referenziate direttamente da Accent
    # (scartate da --as-needed), così da non esporre mai /usr/lib dell'host al linker.
    local empty_obj="${libdir}/_empty_stub.o"
    printf '' | gcc -x c -c - -o "${empty_obj}"
    for stub_lib in pangocairo-1.0 cairo-gobject gdk_pixbuf-2.0 atk-1.0 pango-1.0 cairo harfbuzz z; do
        if [[ ! -e "${libdir}/lib${stub_lib}.so" ]] && [[ ! -e "${libdir}/lib${stub_lib}.a" ]]; then
            ar rcs "${libdir}/lib${stub_lib}.a" "${empty_obj}"
        fi
    done
    rm -f "${empty_obj}"

    touch "${stamp_file}"
}

BUILT_BIN=""
if [[ "${USE_SYSROOT}" -eq 1 ]]; then
    SYSROOT_DIR="${DEBIAN13_SYSROOT_DIR:-/tmp/debian13-trixie-sysroot}"
    prepare_debian13_sysroot "${SYSROOT_DIR}"

    SYSROOT_LLD="$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld"
    if [[ ! -x "${SYSROOT_LLD}" ]]; then
        SYSROOT_LLD="$(command -v ld.lld || command -v ld)"
    fi
    GCC_CRTBEGIN_S="$(gcc -print-file-name=crtbeginS.o)"
    GCC_CRTEND_S="$(gcc -print-file-name=crtendS.o)"
    GCC_LIBDIR="$(dirname "$(gcc -print-file-name=libgcc.a)")"

    LINKER_WRAPPER="/tmp/accent-debian13-x86_64-linker.sh"
    cat << EOF > "${LINKER_WRAPPER}"
#!/usr/bin/env bash
set -euo pipefail
SYSROOT="${SYSROOT_DIR}"
LLD="${SYSROOT_LLD}"
GCC_CRTBEGIN_S="${GCC_CRTBEGIN_S}"
GCC_CRTEND_S="${GCC_CRTEND_S}"
GCC_LIBDIR="${GCC_LIBDIR}"

ARGS=()
IS_SHARED=0
for arg in "\$@"; do
    case "\$arg" in
        -Wl,*)
            IFS=',' read -ra PARTS <<< "\${arg#-Wl,}"
            for p in "\${PARTS[@]}"; do
                [[ -n "\$p" ]] && ARGS+=("\$p")
            done
            ;;
        -shared)
            IS_SHARED=1
            ARGS+=("-shared")
            ;;
        -pie|-no-pie)
            ;;
        -L/usr/lib|-L/usr/lib64|-L/lib|-L/lib64)
            ;;
        -m64|-nodefaultlibs|-nostartfiles|-fuse-ld=*|-B*|-f*)
            ;;
        *)
            ARGS+=("\$arg")
            ;;
    esac
done

COMMON_FLAGS=(
    "--sysroot=\${SYSROOT}"
    "-nostdlib"
    "--build-id"
    "--eh-frame-hdr"
    "--hash-style=gnu"
    "-m" "elf_x86_64"
    "-L\${SYSROOT}/usr/lib/x86_64-linux-gnu"
    "-L\${GCC_LIBDIR}"
)

if [[ "\$IS_SHARED" -eq 0 ]]; then
    exec "\$LLD" "\${COMMON_FLAGS[@]}" \\
        -dynamic-linker /lib64/ld-linux-x86-64.so.2 \\
        -pie \\
        "\${SYSROOT}/usr/lib/x86_64-linux-gnu/Scrt1.o" \\
        "\${SYSROOT}/usr/lib/x86_64-linux-gnu/crti.o" \\
        "\${GCC_CRTBEGIN_S}" \\
        "\${ARGS[@]}" \\
        -lgcc --as-needed -lgcc_s --no-as-needed \\
        -lc \\
        -lgcc --as-needed -lgcc_s --no-as-needed \\
        "\${GCC_CRTEND_S}" \\
        "\${SYSROOT}/usr/lib/x86_64-linux-gnu/crtn.o"
else
    exec "\$LLD" "\${COMMON_FLAGS[@]}" \\
        "\${SYSROOT}/usr/lib/x86_64-linux-gnu/crti.o" \\
        "\${GCC_CRTBEGIN_S}" \\
        "\${ARGS[@]}" \\
        -lgcc --as-needed -lgcc_s --no-as-needed \\
        -lc \\
        -lgcc --as-needed -lgcc_s --no-as-needed \\
        "\${GCC_CRTEND_S}" \\
        "\${SYSROOT}/usr/lib/x86_64-linux-gnu/crtn.o"
fi
EOF
    chmod +x "${LINKER_WRAPPER}"

    echo "==> Compilazione release di Accent con ABI Debian 13 (glibc <= ${TARGET_MAX_GLIBC})..."
    (
        cd "${REPO_ROOT}"
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="${LINKER_WRAPPER}" \
            cargo build --release --locked --target x86_64-unknown-linux-gnu -p accent-ui
    )
    BUILT_BIN="${REPO_ROOT}/target/x86_64-unknown-linux-gnu/release/accent-ui"
else
    echo "==> Compilazione release nativa di Accent (host glibc ${HOST_GLIBC} <= ${TARGET_MAX_GLIBC})..."
    (
        cd "${REPO_ROOT}"
        cargo build --release --locked -p accent-ui
    )
    BUILT_BIN="${REPO_ROOT}/target/release/accent-ui"
fi

# Prepara la struttura del pacchetto Debian
BUILD_ROOT="${SCRIPT_DIR}/build"
STAGE_DIR="${BUILD_ROOT}/${PKG_NAME}_${DEB_VERSION}_${DEB_ARCH}"
rm -rf "${BUILD_ROOT}"
mkdir -p \
    "${STAGE_DIR}/DEBIAN" \
    "${STAGE_DIR}/usr/bin" \
    "${STAGE_DIR}/usr/share/applications" \
    "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps" \
    "${STAGE_DIR}/usr/share/icons/hicolor/512x512/apps" \
    "${STAGE_DIR}/usr/share/doc/${PKG_NAME}"

# 1. Binario principale + strip simboli superflui (Debian Policy §10.1) + symlink relativo (Debian Policy §10.5)
install -Dm755 "${BUILT_BIN}" "${STAGE_DIR}/usr/bin/accent"
strip --strip-unneeded "${STAGE_DIR}/usr/bin/accent"
ln -sf accent "${STAGE_DIR}/usr/bin/accent-ui"

# 2. Verifica rigorosa ABI ELF per Debian 13 (Trixie)
echo "==> Verifica compatibilità ABI ELF per Debian 13 (Trixie)..."
INTERP="$(readelf -l "${STAGE_DIR}/usr/bin/accent" | awk -F': ' '/Requesting program interpreter/ {gsub(/\]/, "", $2); print $2}' | tr -d '[:space:]')"
if [[ "${INTERP}" != "/lib64/ld-linux-x86-64.so.2" ]]; then
    echo "Errore ABI: interprete ELF inatteso '${INTERP}'." >&2
    exit 1
fi
if readelf -d "${STAGE_DIR}/usr/bin/accent" | grep -qE '\((RPATH|RUNPATH)\)'; then
    echo "Errore Debian Policy: il binario contiene RPATH/RUNPATH non consentiti." >&2
    exit 1
fi
MAX_REQ_GLIBC="$(objdump -T "${STAGE_DIR}/usr/bin/accent" | grep -oE 'GLIBC_[0-9]+\.[0-9]+(\.[0-9]+)?' | sed 's/GLIBC_//' | sort -uV | tail -n 1)"
echo "    Interprete ELF:           ${INTERP}"
echo "    Massima versione GLIBC:   GLIBC_${MAX_REQ_GLIBC} (Debian 13 fornisce GLIBC_${TARGET_MAX_GLIBC})"
if [[ "$(printf '%s\n%s\n' "${TARGET_MAX_GLIBC}" "${MAX_REQ_GLIBC}" | sort -V | tail -n 1)" != "${TARGET_MAX_GLIBC}" ]]; then
    echo "Errore ABI: il binario richiede GLIBC_${MAX_REQ_GLIBC}, superiore a GLIBC_${TARGET_MAX_GLIBC} di Debian 13!" >&2
    exit 1
fi

# 3. File Desktop, icone e documentazione
install -Dm644 "${SCRIPT_DIR}/accent.desktop" "${STAGE_DIR}/usr/share/applications/accent.desktop"
if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "${STAGE_DIR}/usr/share/applications/accent.desktop"
fi

install -Dm644 "${REPO_ROOT}/img/brand/accent_logo.svg" \
    "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps/accent.svg"
install -Dm644 "${REPO_ROOT}/img/brand/accent_logo.png" \
    "${STAGE_DIR}/usr/share/icons/hicolor/512x512/apps/accent.png"

install -Dm644 "${REPO_ROOT}/README.md" "${STAGE_DIR}/usr/share/doc/${PKG_NAME}/README.md"
install -Dm644 "${REPO_ROOT}/SPECS.md" "${STAGE_DIR}/usr/share/doc/${PKG_NAME}/SPECS.md"
install -Dm644 "${SCRIPT_DIR}/copyright" "${STAGE_DIR}/usr/share/doc/${PKG_NAME}/copyright"

CHANGELOG_DATE="$(LC_ALL=C date -R)"
cat << EOF | gzip -9n > "${STAGE_DIR}/usr/share/doc/${PKG_NAME}/changelog.Debian.gz"
${PKG_NAME} (${DEB_VERSION}) trixie; urgency=medium

  * Release of Accent Markdown Editor v${VERSION} packaged for Debian 13 (Trixie).

 -- Accent Developers <maintainers@accent-editor.org>  ${CHANGELOG_DATE}
EOF
chmod 0644 "${STAGE_DIR}/usr/share/doc/${PKG_NAME}/changelog.Debian.gz"

# Normalizza i permessi delle directory a 0755 (richiesto da lintian)
find "${STAGE_DIR}" -type d -exec chmod 0755 {} +

# 4. Metadati DEBIAN/control e DEBIAN/md5sums
INSTALLED_SIZE="$(du -sk --exclude="${STAGE_DIR}/DEBIAN" "${STAGE_DIR}" | awk '{print $1}')"

cat << EOF > "${STAGE_DIR}/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${DEB_VERSION}
Section: editors
Priority: optional
Architecture: ${DEB_ARCH}
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6 (>= 2.39), libgcc-s1 (>= 4.2), libglib2.0-0t64 (>= 2.70), libgtk-3-0t64 (>= 3.24), libwebkit2gtk-4.1-0, libjavascriptcoregtk-4.1-0, libsoup-3.0-0, libdbus-1-3, libgdk-pixbuf-2.0-0, libcairo2, libwayland-client0 (>= 1.20), libxkbcommon0 (>= 1.0), libxkbcommon-x11-0 (>= 1.0), libx11-6, libx11-xcb1, libxcb1, libxcursor1, libxi6, libxrandr2, libegl1, libgl1, fontconfig
Recommends: hicolor-icon-theme, shared-mime-info, desktop-file-utils, fonts-inter | fonts-cantarell | fonts-dejavu-core
Provides: accent
Conflicts: accent
Replaces: accent
Maintainer: Accent Developers <maintainers@accent-editor.org>
Homepage: https://github.com/accent-editor/accent
Description: Distraction-free multiplatform Markdown editor and viewer in pure Rust
 Accent is a fast, clean-room Markdown editor and viewer written in 100% pure
 Rust using eframe/egui, inspired by Apostrophe.
 .
 Features include Caret-Aware Live Preview (View-while-typing), Focus and Zen
 modes, native GPU-rasterized Mermaid diagrams without WebView, atomic saving
 with sub-300ms crash-recovery journaling, and zero-dependency native export to
 PDF, DOCX, ODT, LaTeX, EPUB, and HTML5.
EOF
chmod 0644 "${STAGE_DIR}/DEBIAN/control"

(
    cd "${STAGE_DIR}"
    find usr -type f -print0 | LC_ALL=C sort -z | xargs -0 md5sum > "${STAGE_DIR}/DEBIAN/md5sums"
)
chmod 0644 "${STAGE_DIR}/DEBIAN/md5sums"

# 5. Costruzione dell'archivio .deb (formato Debian 2.0)
rm -f "${SCRIPT_DIR}"/${PKG_NAME}_*_${DEB_ARCH}.deb
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "==> Assemblaggio pacchetto .deb tramite dpkg-deb..."
    dpkg-deb --root-owner-group -Zxz --build "${STAGE_DIR}" "${DEB_OUTPUT}"
else
    echo "==> Assemblaggio deterministico pacchetto .deb (Debian 2.0 ar + xz)..."
    DATA_STAGE="${BUILD_ROOT}/data_root"
    mkdir -p "${DATA_STAGE}"
    cp -a "${STAGE_DIR}/usr" "${DATA_STAGE}/"

    CONTROL_TAR="${BUILD_ROOT}/control.tar.xz"
    DATA_TAR="${BUILD_ROOT}/data.tar.xz"
    DEB_BIN_FILE="${BUILD_ROOT}/debian-binary"

    printf '2.0\n' > "${DEB_BIN_FILE}"
    tar --format=gnu --sort=name --owner=root:0 --group=root:0 -C "${STAGE_DIR}/DEBIAN" -cf - . | xz -6 > "${CONTROL_TAR}"
    tar --format=gnu --sort=name --owner=root:0 --group=root:0 -C "${DATA_STAGE}" -cf - . | xz -6 > "${DATA_TAR}"

    python3 - "${DEB_OUTPUT}" "${DEB_BIN_FILE}" "${CONTROL_TAR}" "${DATA_TAR}" << 'PYEOF'
import os
import sys
import time

out_deb = sys.argv[1]
members = [
    ("debian-binary", sys.argv[2]),
    ("control.tar.xz", sys.argv[3]),
    ("data.tar.xz", sys.argv[4]),
]
mtime = int(os.environ.get("SOURCE_DATE_EPOCH", int(time.time())))

with open(out_deb, "wb") as out:
    out.write(b"!<arch>\n")
    for name, path in members:
        with open(path, "rb") as f:
            payload = f.read()
        header = f"{name:<16}{mtime:<12d}{0:<6d}{0:<6d}{0o100644:<8o}{len(payload):<10d}`\n".encode("ascii")
        if len(header) != 60:
            raise SystemExit(f"Lunghezza header ar non valida per {name}: {len(header)}")
        out.write(header)
        out.write(payload)
        if len(payload) % 2 == 1:
            out.write(b"\n")
PYEOF
fi

rm -rf "${BUILD_ROOT}"

SHA256_SUM="$(sha256sum "${DEB_OUTPUT}" | awk '{print $1}')"
DEB_SIZE_H="$(du -h "${DEB_OUTPUT}" | awk '{print $1}')"

echo "==> Pacchetto Debian 13 (Trixie) creato con successo:"
echo "    File:   ${DEB_OUTPUT}"
echo "    Taglia: ${DEB_SIZE_H}"
echo "    SHA256: ${SHA256_SUM}"
echo ""
echo "Per installarlo su Debian 13 (Trixie) esegui:"
echo "    sudo apt install \"./$(basename "${DEB_OUTPUT}")\""
