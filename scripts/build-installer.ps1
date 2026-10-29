<#
.SYNOPSIS
    Builds the Inno Setup installer executable (.exe) for Gank on Windows.
.PARAMETER OutDir
    Output directory for the compiled installer (defaults to 'dist').
#>
param (
    [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"
$rootDir = (Get-Item $PSScriptRoot).Parent.FullName

# 1. Extract version from Cargo.toml
$cargoToml = Get-Content (Join-Path $rootDir "Cargo.toml") -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
} else {
    $version = "0.1.0"
}

# 2. Always compile release binary to ensure latest changes are packaged
$releaseExe = Join-Path $rootDir "target\release\Gank.exe"
Write-Host "Compiling Gank v$version in Release mode (opt-level=3, LTO)..."
Push-Location $rootDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to compile Gank in release mode."
    }
} finally {
    Pop-Location
}

# 3. Locate ISCC.exe (Inno Setup Compiler)
$isccCandidates = @(
    "ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 7\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\InnoSetup\ISCC.exe",
    "C:\Program Files\Inno Setup 7\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 7\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($candidate in $isccCandidates) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) {
        $isccPath = $candidate
        break
    }
    if (Test-Path $candidate) {
        $isccPath = $candidate
        break
    }
}

if (-not $isccPath) {
    throw "Inno Setup Compiler (ISCC.exe) not found. Please install Inno Setup 6 or run in CI."
}

# 4. Compile Installer
$distDir = Join-Path $rootDir $OutDir
if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

$issScript = Join-Path $rootDir "scripts\installer.iss"
Write-Host "Compiling installer with Inno Setup ($isccPath)..."
& $isccPath "/DMyAppVersion=$version" "/O$distDir" $issScript

if ($LASTEXITCODE -ne 0) {
    throw "Failed to compile Inno Setup installer."
}

$installerExe = Join-Path $distDir "gank-v$version-windows-x86_64-setup.exe"
if (Test-Path $installerExe) {
    Write-Host "Calculating SHA256 of installer..."
    $hash = (Get-FileHash -Path $installerExe -Algorithm SHA256).Hash.ToLower()
    $hashFile = "$installerExe.sha256"
    Set-Content -Path $hashFile -Value "$hash  $(Split-Path -Leaf $installerExe)"
    Write-Host "`n==============================================="
    Write-Host "Installer built successfully!"
    Write-Host "File:   $installerExe"
    Write-Host "SHA256: $hash"
}
