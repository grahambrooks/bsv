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

Pushing the tag triggers the [`release`](.github/workflows/release.yml) workflow
(release-kit v2; configured by [`.release.env`](.release.env)), which then:
   - stamps the tag's version into `Cargo.toml` and builds release binaries for
     macOS (Intel + Apple Silicon), Linux (x86_64 + arm64), and Windows (x86_64);
   - uploads each as `bsv-vX.Y.Z-<target>.tar.gz` (`.zip` on Windows) to the
     GitHub Release, plus one `SHA256SUMS` file;
   - regenerates `Formula/bsv.rb` via [`scripts/release.py`](scripts/release.py)
     and merges it to `main` through a pull request.

When that succeeds, [`release-extras`](.github/workflows/release-extras.yml)
regenerates `bucket/bsv.json` from the published `SHA256SUMS` via
[`scripts/update-packaging.sh`](scripts/update-packaging.sh) and merges it the
same way.

## What ships where

| Channel | File | Consumed by |
|---------|------|-------------|
| Homebrew (macOS/Linux) | `Formula/bsv.rb` | `brew install` |
| Scoop (Windows) | `bucket/bsv.json` | `scoop install` |
| Shell installer | `install.sh` | `curl`/`wget` \| `sh` |
| PowerShell installer | `install.ps1` | `irm … \| iex` |

## Updating the Scoop manifest by hand

The Homebrew formula is only ever written by the release workflow. If you need
to regenerate the Scoop manifest outside CI, download the release's `SHA256SUMS`
and run:

```bash
make update-packaging VERSION=<version> CHECKSUMS=<path-to-SHA256SUMS>
# or directly:
scripts/update-packaging.sh <version> <path-to-SHA256SUMS>
```

`make verify-packaging` smoke-tests the generator without leaving any changes.

The placeholder `0.0.0` / all-zero checksums committed on `main` between releases
are expected; CI overwrites them on the next tagged build.
