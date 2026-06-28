#!/usr/bin/env bash
# Cut a release: bump the version, run checks, commit, tag vX.Y.Z, and push.
#
# Usage:
#   scripts/release.sh <version>        # e.g. scripts/release.sh 2026.3.0
#   make release VERSION=<version>
#
# Pushing the tag triggers the Release workflow, which builds the per-platform
# binaries, publishes the GitHub Release, and commits the regenerated install
# packaging (Homebrew formula + Scoop manifest) with the new version/checksums.
set -euo pipefail

VERSION="${1:?usage: release.sh <version> (e.g. 2026.3.0)}"
VERSION="${VERSION#v}"
TAG="v${VERSION}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

current="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"
echo "==> Preparing release ${TAG} (current version ${current})"

# --- preconditions ----------------------------------------------------------
if [ -n "$(git status --porcelain)" ]; then
    echo "error: working tree is not clean; commit or stash first" >&2
    exit 1
fi
branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$branch" != "main" ]; then
    echo "error: releases are cut from main (currently on '$branch')" >&2
    exit 1
fi
git fetch --quiet origin main
if [ -n "$(git rev-list HEAD..origin/main)" ]; then
    echo "error: local main is behind origin/main; pull first" >&2
    exit 1
fi
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "error: tag $TAG already exists" >&2
    exit 1
fi

# --- gate on a green build --------------------------------------------------
make check

# --- bump version and refresh the lockfile ----------------------------------
VERSION="$VERSION" perl -i -pe \
    'BEGIN{$n=0} if(/^version = /){ $n++; s/".*"/"$ENV{VERSION}"/ if $n==1 }' Cargo.toml
cargo build --quiet

# --- commit, tag, push ------------------------------------------------------
git add Cargo.toml Cargo.lock
git commit -m "Release ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"
git push origin main "$TAG"

cat <<EOF

==> Pushed ${TAG}.
    The Release workflow will now build per-platform binaries, publish the
    GitHub Release, and commit updated Formula/bsv.rb + bucket/bsv.json
    (Homebrew + Scoop) with the new version and checksums.
EOF
