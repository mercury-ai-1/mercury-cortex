<#
.SYNOPSIS
    Install the mercury-cortex CLI from GitHub Releases into a per-user location
    (Windows PowerShell 5.1 and PowerShell Core 7+).

.DESCRIPTION
    Downloads the correct prebuilt .zip for this CPU architecture from the
    latest (or a pinned) GitHub release, verifies its SHA-256 checksum against
    `checksums.txt`, extracts it, installs it under the current user's profile,
    and updates the user's PATH if required.

    It never builds from source and never executes downloaded content.

.PARAMETER Version
    Pin a specific release tag, e.g. "-Version v0.5.2".
    Defaults to the latest release from the GitHub API.

.PARAMETER Repo
    Owner/repository, e.g. "mercury-ai-1/mercury-cortex".
    Defaults to "mercury-ai-1/mercury-cortex".

.PARAMETER InstallDir
    Override the destination directory. Defaults to
    $env:LOCALAPPDATA\Programs\mercury-cortex-bin.

.PARAMETER NoPath
    Do NOT modify the user's PATH.

.EXAMPLE
    .\install.ps1
    irm https://raw.githubusercontent.com/mercury-ai-1/mercury-cortex/main/scripts/install.ps1 | iex
#>
[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$Repo = "mercury-ai-1/mercury-cortex",
    [string]$InstallDir = "",
    [switch]$NoPath
)

$ErrorActionPreference = 'Stop'

$Program = "mercury-cortex"

# ---------------------------------------------------------------------------
# Detect CPU architecture (Rust target arch fragment).
# ---------------------------------------------------------------------------
function Get-TargetArch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { return 'x86_64' }
        'ARM64' { return 'aarch64' }
        default {
            Throw "Unsupported CPU architecture: $env:PROCESSOR_ARCHITECTURE (supported: x86_64, aarch64)"
        }
    }
}

# ---------------------------------------------------------------------------
# Resolve the version to install.
# ---------------------------------------------------------------------------
function Resolve-Version {
    param([string]$Version, [string]$Repo)
    if ($Version) {
        if (-not $Version.StartsWith('v')) { $Version = 'v' + $Version }
        $versionTag = $Version
        Write-Host "Installing pinned version $versionTag"
    }
    else {
        $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
        Write-Host "Resolving latest release from $apiUrl"
        $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = "$Program-installer" }
        $versionTag = $release.tag_name
        Write-Host "Latest release: $versionTag"
    }
    return $versionTag
}

# ---------------------------------------------------------------------------
# Compute the SHA-256 of a file.
# ---------------------------------------------------------------------------
function Get-FileSha256 {
    param([string]$Path)
    $hash = Get-FileHash -Algorithm SHA256 -Path $Path
    return $hash.Hash.ToLowerInvariant()
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
$arch = Get-TargetArch
$versionTag = Resolve-Version -Version $Version -Repo $Repo
$triple = "$arch-pc-windows-msvc"
$archiveName = "$Program-$versionTag-$triple.zip"
$sumFileName = "checksums.txt"
$baseUrl = "https://github.com/$Repo/releases/download/$versionTag"
$archiveUrl = "$baseUrl/$archiveName"
$sumUrl = "$baseUrl/$sumFileName"

Write-Host ""
Write-Host "Installing $Program"
Write-Host "  version : $versionTag"
Write-Host "  target  : $triple"
Write-Host "  source  : $archiveUrl"
Write-Host ""

# ---------------------------------------------------------------------------
# Secure temporary working area (unique per process).
# ---------------------------------------------------------------------------
$tempRoot = Join-Path $env:TEMP "$Program-work"
if (Test-Path $tempRoot) { Remove-Item -Recurse -Force $tempRoot }
New-Item -ItemType Directory -Path $tempRoot | Out-Null

try {
    # --- Download -------------------------------------------------------------
    Write-Host "Downloading archive..."
    $archivePath = Join-Path $tempRoot $archiveName
    $netParams = @{ Uri = $archiveUrl; OutFile = $archivePath; UseBasicParsing = $true }
    if ($PSVersionTable.PSVersion.Major -ge 6) { $netParams.Remove('UseBasicParsing') }
    Invoke-WebRequest @netParams
    if (-not (Test-Path $archivePath)) { throw "Failed to download $archiveUrl" }

    Write-Host "Downloading checksums..."
    $sumPath = Join-Path $tempRoot $sumFileName
    $netParams = @{ Uri = $sumUrl; OutFile = $sumPath; UseBasicParsing = $true }
    if ($PSVersionTable.PSVersion.Major -ge 6) { $netParams.Remove('UseBasicParsing') }
    Invoke-WebRequest @netParams

    # --- Verify checksum ------------------------------------------------------
    Write-Host "Verifying SHA-256 checksum..."
    $expected = $null
    foreach ($line in Get-Content $sumPath) {
        if ($line -match [regex]::Escape($archiveName)) {
            $expected = ($line -split '\s+')[0].ToLowerInvariant()
            break
        }
    }
    if (-not $expected) { throw "No checksum entry for $archiveName in $sumFileName" }

    $actual = Get-FileSha256 -Path $archivePath
    if ($expected -ne $actual) {
        throw "Checksum mismatch for $archiveName.`n  expected: $expected`n  actual:   $actual`nRefusing to install (possible corruption or tampering)."
    }
    Write-Host "Checksum OK"

    # --- Extract --------------------------------------------------------------
    Write-Host "Extracting archive..."
    $extractRoot = Join-Path $tempRoot 'extracted'
    Expand-Archive -LiteralPath $archivePath -DestinationPath $extractRoot

    $bin = Get-ChildItem -Path $extractRoot -Recurse -Filter "$Program.exe" -File |
           Select-Object -First 1
    if (-not $bin) { throw "Binary '$Program.exe' not found inside the archive" }

    # --- Install location -----------------------------------------------------
    if ($InstallDir) {
        $dest = $InstallDir
    }
    else {
        $dest = Join-Path $env:LOCALAPPDATA "Programs\mercury-cortex-bin"
    }
    $dest = [System.IO.Path]::GetFullPath($dest)
    New-Item -ItemType Directory -Path $dest -Force | Out-Null

    if (Test-Path (Join-Path $dest "$Program.exe")) {
        Write-Host "Replacing existing installation at $(Join-Path $dest "$Program.exe")"
    }

    # --- Install (copy over, overwriting) --------------------------------------
    Copy-Item -LiteralPath $bin.FullName -Destination $dest -Force
    $installedExe = Join-Path $dest "$Program.exe"
    if (-not (Test-Path $installedExe)) { throw "Installation to $dest failed" }

    # --- PATH update -----------------------------------------------------------
    if (-not $NoPath) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if ($userPath -notmatch [regex]::Escape($dest)) {
            $delimiter = [System.IO.Path]::PathSeparator
            if ($userPath) {
                $newPath = $userPath.TrimEnd($delimiter) + $delimiter + $dest
            }
            else {
                $newPath = $dest
            }
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
            Write-Host "Added $dest to your user PATH."
        }
        else {
            Write-Host "$dest is already on your user PATH."
        }
        Write-Warning "PATH changes apply to new terminal sessions. Restart your terminal before using 'mercury-cortex'."
    }
    else {
        Write-Host "Skipping PATH update (-NoPath)."
        Write-Host "Add $dest to your PATH manually to use 'mercury-cortex'."
    }

    # --- Report installed version ---------------------------------------------
    Write-Host ""
    Write-Host "Successfully installed $Program to $installedExe"
    $installedVersion = & $installedExe version 2>$null
    if (-not $installedVersion) { $installedVersion = & $installedExe --version 2>$null }
    if ($installedVersion) {
        Write-Host ("Installed version: {0}" -f $installedVersion.Trim())
    }
}
finally {
    # --- Cleanup temp files -----------------------------------------------------
    if (Test-Path $tempRoot) { Remove-Item -Recurse -Force $tempRoot -ErrorAction SilentlyContinue }
}