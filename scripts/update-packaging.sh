#!/usr/bin/env bash
# Regenerate the in-repo Scoop manifest for a release.
#
# Usage:
#   scripts/update-packaging.sh <version> <SHA256SUMS>
#
#   <version>     Release version without a leading "v" (e.g. 2026.9.1).
#   <SHA256SUMS>  The release's SHA256SUMS file ("<hash>  <filename>" per line),
#                 as published by the release workflow.
#
# Run from the repository root. The Homebrew formula (Formula/bsv.rb) is written
# by the release-kit release workflow (scripts/release.py formula); this script
# only covers Scoop, and is run by .github/workflows/release-extras.yml after a
# successful release. It can also be run locally to refresh the manifest.
set -euo pipefail

VERSION="${1:?usage: update-packaging.sh <version> <SHA256SUMS>}"
SUMS="${2:?usage: update-packaging.sh <version> <SHA256SUMS>}"
VERSION="${VERSION#v}"

REPO="grahambrooks/bsv"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSET="bsv-v${VERSION}-x86_64-pc-windows-msvc.zip"

win_x86="$(awk -v a="$ASSET" '$2 == a { print $1 }' "$SUMS")"
if [ -z "$win_x86" ]; then
    echo "error: no checksum for ${ASSET} in ${SUMS}" >&2
    exit 1
fi

# --- Scoop manifest ----------------------------------------------------------
cat > "${ROOT}/bucket/bsv.json" <<EOF
{
    "version": "${VERSION}",
    "description": "Backstage Entity Visualizer - TUI for exploring catalog-info.yaml files",
    "homepage": "https://github.com/${REPO}",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/${REPO}/releases/download/v${VERSION}/${ASSET}",
            "hash": "${win_x86}",
            "bin": "bsv.exe"
        }
    },
    "checkver": {
        "github": "https://github.com/${REPO}"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/${REPO}/releases/download/v\$version/bsv-v\$version-x86_64-pc-windows-msvc.zip"
            }
        }
    }
}
EOF

echo "Updated bucket/bsv.json to version ${VERSION}"
