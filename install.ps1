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
$Asset = "bsv-$Target.zip"

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

$Url = "https://github.com/$Repo/releases/download/$Tag/$Asset"

# --- choose install dir ------------------------------------------------------
if ([string]::IsNullOrEmpty($BinDir)) {
    $BinDir = Join-Path $env:LOCALAPPDATA 'Programs\bsv'
}
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# --- download and install ----------------------------------------------------
$Tmp = New-Item -ItemType Directory -Force -Path (Join-Path $env:TEMP ("bsv-" + [System.Guid]::NewGuid().ToString()))
try {
    $zip = Join-Path $Tmp $Asset
    Info "Downloading $Asset ($Tag)..."
    Invoke-WebRequest -Uri $Url -OutFile $zip

    # Verify checksum if the .sha256 asset is published alongside the archive.
    try {
        $shaFile = "$zip.sha256"
        Invoke-WebRequest -Uri "$Url.sha256" -OutFile $shaFile -ErrorAction Stop
        $expected = ((Get-Content $shaFile -Raw).Trim() -split '\s+')[0]
        $actual = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
        if ($expected.ToLower() -ne $actual) {
            throw "Checksum mismatch (expected $expected, got $actual)"
        }
        Info 'Checksum verified.'
    } catch [System.Net.WebException] {
        # No checksum published; continue without verification.
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
