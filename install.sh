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

PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"

APP_NAME="kalk"
BINARY="target/release/${APP_NAME}"
DESKTOP_FILE="packaging/linux/${APP_NAME}.desktop"
ICON_FILE="packaging/linux/${APP_NAME}.svg"

# --- helpers ---

die() {
    echo "ERROR: $*" >&2
    exit 1
}

need_root() {
    if [ "$(id -u)" -ne 0 ]; then
        die "This operation needs root. Use sudo or run ./install.sh --user"
    fi
}

# --- install ---

install_binary() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    mkdir -p "${bindir}"
    install -m755 "${BINARY}" "${bindir}/${APP_NAME}"
    echo "  -> ${bindir}/${APP_NAME}"
}

install_desktop() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    mkdir -p "${appdir}"
    install -m644 "${DESKTOP_FILE}" "${appdir}/${APP_NAME}.desktop"
    echo "  -> ${appdir}/${APP_NAME}.desktop"
}

install_icon() {
    local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps"
    mkdir -p "${icondir}"
    install -m644 "${ICON_FILE}" "${icondir}/${APP_NAME}.svg"
    echo "  -> ${icondir}/${APP_NAME}.svg"
}

# --- uninstall ---

uninstall_files() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps"

    rm -f "${bindir}/${APP_NAME}"
    rm -f "${appdir}/${APP_NAME}.desktop"
    rm -f "${icondir}/${APP_NAME}.svg"

    echo "  Removed ${bindir}/${APP_NAME}"
    echo "  Removed ${appdir}/${APP_NAME}.desktop"
    echo "  Removed ${icondir}/${APP_NAME}.svg"
}

# --- main ---

IS_USER=0

if [ "${1:-}" = "--user" ]; then
    IS_USER=1
    shift
fi

if [ "${1:-}" = "--uninstall" ]; then
    if [ "$IS_USER" -eq 1 ]; then
        PREFIX="${HOME}/.local"
        DESTDIR=""
    else
        need_root
    fi
    echo "Uninstalling Kalk from ${PREFIX}..."
    uninstall_files
    echo "Done. User data (~/.local/share/kalk/) was left untouched."
    exit 0
fi

if [ "$IS_USER" -eq 1 ]; then
    PREFIX="${HOME}/.local"
    DESTDIR=""
    echo "Installing Kalk for current user (${PREFIX})..."
elif [ "$(id -u)" -eq 0 ]; then
    echo "Installing Kalk system-wide (${PREFIX})..."
else
    need_root
fi

# Build if binary doesn't exist or source is newer
NEEDS_BUILD=0
if [ ! -f "${BINARY}" ]; then
    NEEDS_BUILD=1
elif [ -n "$(find src/ Cargo.toml Cargo.lock -newer "${BINARY}" 2>/dev/null)" ]; then
    NEEDS_BUILD=1
fi

if [ "$NEEDS_BUILD" -eq 1 ]; then
    echo "Building release binary..."
    cargo build --release --locked
fi

install_binary
install_desktop
install_icon

echo ""
echo "Kalk installed successfully."
echo "Run 'kalk' from your terminal, or find it in your application menu."
if [ "$IS_USER" -eq 1 ]; then
    echo "Uninstall with: ./install.sh --user --uninstall"
else
    echo "Uninstall with: sudo $(realpath "$0") --uninstall"
fi
