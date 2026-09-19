#!/bin/sh
# bsv installer for macOS and Linux.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/grahambrooks/bsv/main/install.sh | sh
#   wget -qO- https://raw.githubusercontent.com/grahambrooks/bsv/main/install.sh | sh
#
# Environment variables:
#   BSV_VERSION   Version tag to install (default: latest release)
#   BSV_BIN_DIR   Install directory (default: ~/.local/bin, or /usr/local/bin if writable)
#
# This script downloads a prebuilt release binary from GitHub Releases. It uses
# curl or wget, whichever is available.
set -eu

REPO="grahambrooks/bsv"
BIN_NAME="bsv"

info() { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
err() { printf '\033[1;31merror:\033[0m %s\n' "$1" >&2; exit 1; }

# --- pick a downloader -------------------------------------------------------
if command -v curl >/dev/null 2>&1; then
    dl() { curl -fsSL "$1"; }
    dl_to() { curl -fsSL "$1" -o "$2"; }
elif command -v wget >/dev/null 2>&1; then
    dl() { wget -qO- "$1"; }
    dl_to() { wget -q "$1" -O "$2"; }
else
    err "neither curl nor wget found; please install one and retry"
fi

# --- detect platform ---------------------------------------------------------
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
    Darwin) os_part="apple-darwin" ;;
    Linux)  os_part="unknown-linux-gnu" ;;
    *) err "unsupported OS: $os (use the Windows PowerShell installer instead)" ;;
esac

case "$arch" in
    x86_64 | amd64) arch_part="x86_64" ;;
    arm64 | aarch64) arch_part="aarch64" ;;
    *) err "unsupported architecture: $arch" ;;
esac

target="${arch_part}-${os_part}"

# --- resolve version ---------------------------------------------------------
version="${BSV_VERSION:-}"
if [ -z "$version" ]; then
    info "Resolving latest release..."
    version="$(dl "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -n1 | cut -d'"' -f4)"
    [ -n "$version" ] || err "could not determine latest version; set BSV_VERSION explicitly"
fi
# Accept both "v1.2.3" and "1.2.3".
case "$version" in v*) tag="$version" ;; *) tag="v$version" ;; esac

# Releases since release-kit v2 name archives <bin>-<tag>-<target>.tar.gz and
# publish one SHA256SUMS file; older releases used <bin>-<target>.tar.gz with a
# .sha256 file beside each archive. Try the current layout first.
base="https://github.com/${REPO}/releases/download/${tag}"
asset="${BIN_NAME}-${tag}-${target}.tar.gz"
legacy_asset="${BIN_NAME}-${target}.tar.gz"

# --- choose install dir ------------------------------------------------------
bin_dir="${BSV_BIN_DIR:-}"
if [ -z "$bin_dir" ]; then
    if [ -w /usr/local/bin ] 2>/dev/null; then
        bin_dir="/usr/local/bin"
    else
        bin_dir="${HOME}/.local/bin"
    fi
fi
mkdir -p "$bin_dir"

# --- download and install ----------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

info "Downloading ${BIN_NAME} ${tag} for ${target}..."
if ! dl_to "${base}/${asset}" "${tmp}/${asset}" 2>/dev/null; then
    asset="$legacy_asset"
    dl_to "${base}/${asset}" "${tmp}/${asset}" || err "download failed: ${base}/${asset}"
fi

# Verify the checksum when the release publishes one (SHA256SUMS, or the older
# per-archive .sha256 file).
expected=""
if dl_to "${base}/SHA256SUMS" "${tmp}/SHA256SUMS" 2>/dev/null; then
    expected="$(awk -v a="$asset" '$2 == a { print $1 }' "${tmp}/SHA256SUMS")"
elif dl_to "${base}/${asset}.sha256" "${tmp}/${asset}.sha256" 2>/dev/null; then
    expected="$(cut -d' ' -f1 < "${tmp}/${asset}.sha256")"
fi
if [ -n "$expected" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
        actual="$(sha256sum "${tmp}/${asset}" | cut -d' ' -f1)"
    else
        actual="$(shasum -a 256 "${tmp}/${asset}" | cut -d' ' -f1)"
    fi
    [ "$expected" = "$actual" ] || err "checksum mismatch (expected $expected, got $actual)"
    info "Checksum verified."
fi

tar -xzf "${tmp}/${asset}" -C "$tmp"
install -m 0755 "${tmp}/${BIN_NAME}" "${bin_dir}/${BIN_NAME}"

info "Installed ${BIN_NAME} to ${bin_dir}/${BIN_NAME}"
case ":${PATH}:" in
    *":${bin_dir}:"*) ;;
    *) info "Add ${bin_dir} to your PATH to run '${BIN_NAME}' from anywhere." ;;
esac
"${bin_dir}/${BIN_NAME}" --version || true
