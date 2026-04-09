#!/bin/sh
set -e

REPO="CosPie/meican_cli"
BIN="meican"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { printf "${CYAN}%s${NC}\n" "$1"; }
success() { printf "${GREEN}%s${NC}\n" "$1"; }
error() { printf "${RED}%s${NC}\n" "$1" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported OS: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Map to Rust target triple
get_target() {
    local os="$1"
    local arch="$2"
    case "${os}-${arch}" in
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
        macos-x86_64)   echo "x86_64-apple-darwin" ;;
        macos-aarch64)  echo "aarch64-apple-darwin" ;;
        windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
        *) error "Unsupported platform: ${os}-${arch}" ;;
    esac
}

# Get latest version from GitHub API
get_latest_version() {
    if command -v curl > /dev/null 2>&1; then
        curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget > /dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "curl or wget is required"
    fi
}

# Download file
download() {
    local url="$1"
    local output="$2"
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget > /dev/null 2>&1; then
        wget -q "$url" -O "$output"
    fi
}

main() {
    local os arch target version

    os=$(detect_os)
    arch=$(detect_arch)
    target=$(get_target "$os" "$arch")

    info "Detected platform: ${os} ${arch} (${target})"

    version=$(get_latest_version)
    if [ -z "$version" ]; then
        error "Failed to get latest version. Check https://github.com/${REPO}/releases"
    fi
    info "Latest version: ${version}"

    local ext="tar.gz"
    if [ "$os" = "windows" ]; then
        ext="zip"
    fi

    local filename="${BIN}-${target}.${ext}"
    local url="https://github.com/${REPO}/releases/download/${version}/${filename}"

    info "Downloading ${url}..."
    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    download "$url" "${tmpdir}/${filename}"

    info "Extracting..."
    if [ "$ext" = "tar.gz" ]; then
        tar xzf "${tmpdir}/${filename}" -C "$tmpdir"
    else
        unzip -q "${tmpdir}/${filename}" -d "$tmpdir" 2>/dev/null || {
            # fallback for systems without unzip
            error "unzip is required on Windows/MSYS2. Install with: pacman -S unzip"
        }
    fi

    # Install
    if [ "$os" = "windows" ]; then
        local win_dir="${USERPROFILE}/.local/bin"
        mkdir -p "$win_dir"
        cp "${tmpdir}/${BIN}.exe" "$win_dir/${BIN}.exe"
        success "Installed to ${win_dir}/${BIN}.exe"
        echo "Add ${win_dir} to your PATH if not already done."
    else
        if [ -w "$INSTALL_DIR" ]; then
            cp "${tmpdir}/${BIN}" "${INSTALL_DIR}/${BIN}"
            chmod +x "${INSTALL_DIR}/${BIN}"
        else
            info "Installing to ${INSTALL_DIR} (requires sudo)..."
            sudo cp "${tmpdir}/${BIN}" "${INSTALL_DIR}/${BIN}"
            sudo chmod +x "${INSTALL_DIR}/${BIN}"
        fi
        success "Installed to ${INSTALL_DIR}/${BIN}"
    fi

    success "Done! Run 'meican --version' to verify."
}

main
