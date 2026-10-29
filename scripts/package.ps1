<#
.SYNOPSIS
    Packages Gank into a standalone redistributable zip archive for Windows.
.PARAMETER OutDir
    Output directory for the packaged release (defaults to 'dist').
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

# 2. Build release binary
Write-Host "Building Gank v$version in Release mode (LTO + opt-level=3 + strip)..." -ForegroundColor Cyan
Push-Location $rootDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to compile Gank in release mode."
    }
} finally {
    Pop-Location
}

$releaseExe = Join-Path $rootDir "target\release\Gank.exe"
if (-not (Test-Path $releaseExe)) {
    throw "Executable not found at $releaseExe"
}

# 3. Prepare staging folder
$distRoot = Join-Path $rootDir $OutDir
$bundleName = "gank-v$version-windows-x86_64"
$stageDir = Join-Path $distRoot $bundleName

if (Test-Path $stageDir) {
    Remove-Item $stageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null

Write-Host "Staging release files to $stageDir..." -ForegroundColor Cyan
Copy-Item $releaseExe -Destination $stageDir
Copy-Item (Join-Path $rootDir "README.md") -Destination $stageDir
Copy-Item (Join-Path $rootDir "LICENSE") -Destination $stageDir

# 4. Create ZIP archive
$zipPath = Join-Path $distRoot "$bundleName.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Write-Host "Compressing to $zipPath..." -ForegroundColor Cyan
Compress-Archive -Path "$stageDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

# 5. Generate SHA256 checksum
$hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
$hashFile = "$zipPath.sha256"
"$hash  $bundleName.zip" | Out-File -FilePath $hashFile -Encoding utf8

$zipItem = Get-Item $zipPath
$zipMb = [math]::Round($zipItem.Length / 1MB, 2)
$exeItem = Get-Item (Join-Path $stageDir "Gank.exe")
$exeMb = [math]::Round($exeItem.Length / 1MB, 2)

Write-Host "=================================================" -ForegroundColor Green
Write-Host "Gank v$version package successfully generated!" -ForegroundColor Green
Write-Host " - Executable: $exeMb MB"
Write-Host " - ZIP Bundle: $zipPath ($zipMb MB)"
Write-Host " - SHA256    : $hash"
Write-Host "=================================================" -ForegroundColor Green
