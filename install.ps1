<#
.SYNOPSIS
    bsv installer for Windows.

.DESCRIPTION
    Downloads a prebuilt bsv release from GitHub Releases and installs it into a
    user-writable directory, adding that directory to the user PATH.

.EXAMPLE
    irm https://raw.githubusercontent.com/grahambrooks/bsv/main/install.ps1 | iex

.PARAMETER Version
    Version tag to install (default: latest release).

.PARAMETER BinDir
    Install directory (default: %LOCALAPPDATA%\Programs\bsv).
#>
[CmdletBinding()]
param(
    [string]$Version = $env:BSV_VERSION,
    [string]$BinDir = $env:BSV_BIN_DIR
)

$ErrorActionPreference = 'Stop'
$Repo = 'grahambrooks/bsv'
$Target = 'x86_64-pc-windows-msvc'

function Info($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# --- resolve version ---------------------------------------------------------
if ([string]::IsNullOrEmpty($Version)) {
    Info 'Resolving latest release...'
    $latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $latest.tag_name
}
if ([string]::IsNullOrEmpty($Version)) {
    throw 'Could not determine latest version; pass -Version explicitly.'
}
$Tag = if ($Version.StartsWith('v')) { $Version } else { "v$Version" }

# Releases since release-kit v2 name archives bsv-<tag>-<target>.zip and publish
# one SHA256SUMS file; older releases used bsv-<target>.zip with a .sha256 file
# beside each archive. Try the current layout first.
$Base = "https://github.com/$Repo/releases/download/$Tag"
$Asset = "bsv-$Tag-$Target.zip"
$LegacyAsset = "bsv-$Target.zip"

# --- choose install dir ------------------------------------------------------
if ([string]::IsNullOrEmpty($BinDir)) {
    $BinDir = Join-Path $env:LOCALAPPDATA 'Programs\bsv'
}
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# --- download and install ----------------------------------------------------
$Tmp = New-Item -ItemType Directory -Force -Path (Join-Path $env:TEMP ("bsv-" + [System.Guid]::NewGuid().ToString()))
try {
    Info "Downloading bsv $Tag for $Target..."
    try {
        $zip = Join-Path $Tmp $Asset
        Invoke-WebRequest -Uri "$Base/$Asset" -OutFile $zip -ErrorAction Stop
    } catch {
        $Asset = $LegacyAsset
        $zip = Join-Path $Tmp $Asset
        Invoke-WebRequest -Uri "$Base/$Asset" -OutFile $zip -ErrorAction Stop
    }

    # Verify the checksum when the release publishes one (SHA256SUMS, or the
    # older per-archive .sha256 file); continue without verification otherwise.
    $expected = $null
    try {
        $sums = Join-Path $Tmp 'SHA256SUMS'
        Invoke-WebRequest -Uri "$Base/SHA256SUMS" -OutFile $sums -ErrorAction Stop
        foreach ($line in Get-Content $sums) {
            $parts = $line.Trim() -split '\s+'
            if ($parts.Count -ge 2 -and $parts[1] -eq $Asset) { $expected = $parts[0] }
        }
    } catch {
        try {
            $shaFile = "$zip.sha256"
            Invoke-WebRequest -Uri "$Base/$Asset.sha256" -OutFile $shaFile -ErrorAction Stop
            $expected = ((Get-Content $shaFile -Raw).Trim() -split '\s+')[0]
        } catch {
            # No checksum published.
        }
    }
    if ($expected) {
        $actual = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
        if ($expected.ToLower() -ne $actual) {
            throw "Checksum mismatch (expected $expected, got $actual)"
        }
        Info 'Checksum verified.'
    }

    Expand-Archive -Path $zip -DestinationPath $Tmp -Force
    Copy-Item -Path (Join-Path $Tmp 'bsv.exe') -Destination (Join-Path $BinDir 'bsv.exe') -Force
}
finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}

Info "Installed bsv.exe to $BinDir"

# --- ensure BinDir is on the user PATH ---------------------------------------
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $BinDir) {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$BinDir", 'User')
    Info "Added $BinDir to your user PATH (restart your shell to pick it up)."
}

& (Join-Path $BinDir 'bsv.exe') --version
