#!/usr/bin/env bash
set -euo pipefail

# Kalk install script
# Installs the binary, desktop entry, and icon.
#
# Usage:
#   sudo ./install.sh                 # system-wide
#   ./install.sh --user               # user-local (~/.local)
#   ./install.sh --user --uninstall   # remove user-local install
#   sudo ./install.sh --uninstall     # remove system-wide install
#   PREFIX=/usr sudo ./install.sh     # custom prefix
#   ./install.sh --help               # show usage
#
# Platforms:
#   Linux  — installs the binary, desktop entry, and icon.
#   macOS  — installs the binary only (no XDG desktop entry / icon theme).

PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"

APP_NAME="kalk"
BINARY="target/release/${APP_NAME}"
DESKTOP_FILE="packaging/linux/${APP_NAME}.desktop"
ICON_FILE="packaging/linux/${APP_NAME}.png"

# --- pretty output ---

log()  { printf '\n\033[1;32m[+] %s\033[0m\n' "$*"; }
warn() { printf '\n\033[1;33m[!] %s\033[0m\n' "$*"; }
err()  { printf '\n\033[1;31m[✗] %s\033[0m\n' "$*"; }
info() { printf '     \033[1;37m%s\033[0m\n\n' "$*"; }

# --- helpers ---

die() {
    err "$*"
    exit 1
}

need_root() {
    if [ "$(id -u)" -ne 0 ]; then
        die "This operation needs root. Use sudo or run ./install.sh --user"
    fi
}

# Detect the platform. Linux installs everything (binary + desktop entry +
# icon); macOS installs the binary only, since it has no XDG desktop entries
# or hicolor icon theme. Other platforms are unsupported.
IS_MACOS=0
detect_os() {
    local os
    os="$(uname -s 2>/dev/null || echo unknown)"
    case "$os" in
        Linux) ;;
        Darwin) IS_MACOS=1 ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            die "Windows is not supported — this installer only targets Linux and macOS. Build manually with: cargo build --release"
            ;;
        *)
            die "Unsupported OS '${os}' — this installer only targets Linux and macOS."
            ;;
    esac
}

# --- install ---

install_binary() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    mkdir -p "${bindir}"
    install -m755 "${BINARY}" "${bindir}/${APP_NAME}"
    info "${bindir}/${APP_NAME}"
}

install_desktop() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    mkdir -p "${appdir}"
    sed "s|^Exec=.*|Exec=${PREFIX}/bin/${APP_NAME}|" "${DESKTOP_FILE}" \
        > "${appdir}/${APP_NAME}.desktop"
    chmod 644 "${appdir}/${APP_NAME}.desktop"
    info "${appdir}/${APP_NAME}.desktop"
}

install_icon() {
    # Install the raster PNG to the legacy pixmaps directory. The previous
    # "icon.svg" actually embedded a WebP image, which the GTK/librsvg icon
    # loader cannot decode — so no icon ever showed. pixmaps has no size
    # requirement and is searched by name, so Icon=kalk resolves to this PNG
    # across GNOME/KDE/XFCE without needing a perfectly-sized hicolor set.
    local pixmaps="${DESTDIR}${PREFIX}/share/pixmaps"
    mkdir -p "${pixmaps}"
    install -m644 "${ICON_FILE}" "${pixmaps}/${APP_NAME}.png"
    info "${pixmaps}/${APP_NAME}.png"

    # Remove any previously-installed broken SVG: the themed scalable dir is
    # searched before pixmaps, so a leftover unrenderable SVG would shadow the
    # PNG and still show no icon.
    rm -f "${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps/${APP_NAME}.svg"
}

# Refresh the icon cache and desktop database so the entry and icon appear
# without a re-login. Best-effort: missing tools or non-zero exits are ignored
# (the install itself already succeeded).
update_caches() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    local hicolor="${DESTDIR}${PREFIX}/share/icons/hicolor"
    if command -v gtk-update-icon-cache >/dev/null 2>&1 && [ -d "${hicolor}" ]; then
        gtk-update-icon-cache -q -f -t "${hicolor}" >/dev/null 2>&1 || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${appdir}" >/dev/null 2>&1 || true
    fi
}

# --- uninstall ---

uninstall_files() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    rm -f "${bindir}/${APP_NAME}"
    info "Removed ${bindir}/${APP_NAME}"

    # Desktop entry and icon only exist on Linux installs.
    if [ "$IS_MACOS" -eq 0 ]; then
        local appdir="${DESTDIR}${PREFIX}/share/applications"
        local pixmaps="${DESTDIR}${PREFIX}/share/pixmaps"
        local scalable="${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps"
        rm -f "${appdir}/${APP_NAME}.desktop"
        rm -f "${pixmaps}/${APP_NAME}.png"
        rm -f "${scalable}/${APP_NAME}.svg" # legacy: clean up the old broken SVG
        info "Removed ${appdir}/${APP_NAME}.desktop"
        info "Removed ${pixmaps}/${APP_NAME}.png"
        update_caches
    fi
}

# --- usage ---

usage() {
    cat <<EOF
Kalk installer — installs the binary, plus a desktop entry and icon on Linux.

Usage:
  sudo ./install.sh                 Install system-wide (PREFIX=/usr/local)
  ./install.sh --user               Install for the current user (~/.local)
  sudo ./install.sh --uninstall     Remove a system-wide install
  ./install.sh --user --uninstall   Remove a user-local install

Options:
  --user        Install/uninstall under ~/.local instead of system-wide
  --uninstall   Remove a previous install (user data is left untouched)
  -h, --help    Show this help and exit

Environment:
  PREFIX        Install prefix (default: /usr/local; ignored with --user)
  DESTDIR       Staging root prepended to every path (for packaging)

Platforms:
  Linux   binary + desktop entry + icon
  macOS   binary only (desktop entry and icon are Linux-only)
EOF
}

# --- main ---

# Handle help first, in any position — works without root or a supported OS.
for arg in "$@"; do
    case "$arg" in
        -h | --help)
            usage
            exit 0
            ;;
    esac
done

IS_USER=0

if [ "${1:-}" = "--user" ]; then
    IS_USER=1
    shift
fi

detect_os

if [ "${1:-}" = "--uninstall" ]; then
    if [ "$IS_USER" -eq 1 ]; then
        PREFIX="${HOME}/.local"
        DESTDIR=""
    else
        need_root
    fi
    log "Uninstalling Kalk from ${PREFIX}..."
    uninstall_files
    info "User data (~/.local/share/kalk/) was left untouched."
    exit 0
fi

if [ "$IS_USER" -eq 1 ]; then
    PREFIX="${HOME}/.local"
    DESTDIR=""
    log "Installing Kalk for current user (${PREFIX})..."
elif [ "$(id -u)" -eq 0 ]; then
    log "Installing Kalk system-wide (${PREFIX})..."
else
    need_root
fi

if [ "$IS_MACOS" -eq 1 ]; then
    warn "macOS detected: only the binary will be installed (no desktop entry or icon)."
fi

# Build if binary doesn't exist or source is newer
NEEDS_BUILD=0
if [ ! -f "${BINARY}" ]; then
    NEEDS_BUILD=1
elif [ -n "$(find src/ Cargo.toml Cargo.lock -newer "${BINARY}" 2>/dev/null)" ]; then
    NEEDS_BUILD=1
fi

if [ "$NEEDS_BUILD" -eq 1 ]; then
    log "Building release binary..."
    cargo build --release --locked
fi

install_binary
if [ "$IS_MACOS" -eq 0 ]; then
    install_desktop
    install_icon
    update_caches
fi

log "Kalk installed successfully."
if [ "$IS_MACOS" -eq 1 ]; then
    info "Run 'kalk' from your terminal."
else
    info "Run 'kalk' from your terminal, or find it in your application menu."
fi

# Portable absolute path to this script (macOS has no coreutils realpath).
script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
if [ "$IS_USER" -eq 1 ]; then
    info "Uninstall with: ./install.sh --user --uninstall"
else
    info "Uninstall with: sudo ${script_path} --uninstall"
fi
