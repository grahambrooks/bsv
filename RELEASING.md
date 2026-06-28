# Releasing

Releases are fully automated from a git tag. Everything needed lives in this
repository — there is no separate Homebrew tap or Scoop bucket repo to maintain.

## Cutting a release

Use the Makefile target, which gates on a green build and handles the version
bump, commit, tag, and push:

```bash
make release                  # version defaults to today's date, e.g. 2026.6.28
make release VERSION=2026.3.0  # or pin an explicit version
```

Versions are calendar-based (`year.month.day`), so a bare `make release` stamps
today's date. This (via [`scripts/release.sh`](scripts/release.sh)):

1. verifies the working tree is clean, you're on an up-to-date `main`, and the
   tag doesn't already exist (a same-day re-release needs an explicit `VERSION=`);
2. runs `make check` (format, clippy, tests, shellcheck, packaging smoke test);
3. bumps the version in `Cargo.toml`, refreshes `Cargo.lock`;
4. commits `Release vX.Y.Z`, creates an annotated tag, and pushes both.

Pushing the tag triggers the [`Release`](.github/workflows/release.yml) workflow,
which then:
   - builds release binaries for macOS (Intel + Apple Silicon), Linux (x86_64),
     and Windows (x86_64);
   - packages each as a `.tar.gz`/`.zip` with a matching `.sha256`;
   - creates the GitHub Release with all assets and generated notes;
   - regenerates `Formula/bsv.rb` and `bucket/bsv.json` with the new version and
     checksums via [`scripts/update-packaging.sh`](scripts/update-packaging.sh) and
     commits them back to `main`.

## What ships where

| Channel | File | Consumed by |
|---------|------|-------------|
| Homebrew (macOS/Linux) | `Formula/bsv.rb` | `brew install` |
| Scoop (Windows) | `bucket/bsv.json` | `scoop install` |
| Shell installer | `install.sh` | `curl`/`wget` \| `sh` |
| PowerShell installer | `install.ps1` | `irm … \| iex` |

## Updating manifests by hand

If you ever need to regenerate the manifests outside CI, download the release
archives' `.sha256` files into a directory and run:

```bash
make update-packaging VERSION=<version> CHECKSUMS=<dir-with-sha256-files>
# or directly:
scripts/update-packaging.sh <version> <dir-with-sha256-files>
```

`make verify-packaging` smoke-tests the generator without leaving any changes.

The placeholder `0.0.0` / all-zero checksums committed on `main` between releases
are expected; CI overwrites them on the next tagged build.
