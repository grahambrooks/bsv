# Releasing

Releases are fully automated from a git tag. Everything needed lives in this
repository — there is no separate Homebrew tap or Scoop bucket repo to maintain.

## Cutting a release

1. Bump the version in `Cargo.toml` and commit it to `main`.
2. Tag the commit and push the tag:

   ```bash
   git tag v$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
   git push origin --tags
   ```

3. The [`Release`](.github/workflows/release.yml) workflow then:
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
scripts/update-packaging.sh <version> <dir-with-sha256-files>
```

The placeholder `0.0.0` / all-zero checksums committed on `main` between releases
are expected; CI overwrites them on the next tagged build.
