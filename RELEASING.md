# Releasing

Releases are fully automated from a git tag. Everything needed lives in this
repository — there is no separate Homebrew tap or Scoop bucket repo to maintain.

## Cutting a release

The tag is the version. Cut a release from a green `main`:

```bash
make release-dry   # what would be tagged
make release       # tag origin/main with the next version and push the tag
```

Versions are calendar-based, `vYYYY.M.D` from today's UTC date; if that tag
already exists the last number is bumped until it is free, so a same-day
re-release needs no arguments. `make release` (from [`release.mk`](release.mk),
shared across grahambrooks' Rust tools via release-kit) fetches tags, tags
`origin/main`, and pushes only the tag. Nothing is bumped or committed locally,
and local uncommitted work or the current branch don't matter.

Pushing the tag triggers the [`release`](.github/workflows/release.yml) workflow
(release-kit v2; configured by [`.release.env`](.release.env)), which then:
   - stamps the tag's version into `Cargo.toml` and builds release binaries for
     macOS (Intel + Apple Silicon), Linux (x86_64 + arm64), and Windows (x86_64);
   - uploads each as `bsv-vX.Y.Z-<target>.tar.gz` (`.zip` on Windows) to the
     GitHub Release, plus one `SHA256SUMS` file;
   - regenerates `Formula/bsv.rb` via [`scripts/release.py`](scripts/release.py)
     and merges it, with the `Cargo.toml` version bump, to `main` through a
     pull request.

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
