# Generates Inno Setup wizard banner bitmaps from source image
Add-Type -AssemblyName System.Drawing

$rootDir = Split-Path -Parent $PSScriptRoot
$srcPath = "C:\Users\carll\.gemini\antigravity\brain\748d5eed-7eb0-4dd5-ad09-01cdbd5933d9\.user_uploaded\media_1793294131522_c730214f.png"
$pngDest = Join-Path $rootDir "assets\wizard_banner.png"
$bmp1xDest = Join-Path $rootDir "assets\wizard_banner.bmp"
$bmp2xDest = Join-Path $rootDir "assets\wizard_banner_2x.bmp"

Copy-Item -Force $srcPath $pngDest

$srcImg = [System.Drawing.Image]::FromFile($srcPath)

function Resize-And-Save-Bmp($src, [int]$w, [int]$h, [string]$dest) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.DrawImage($src, 0, 0, $w, $h)
    $g.Dispose()
    $bmp.Save($dest, [System.Drawing.Imaging.ImageFormat]::Bmp)
    $bmp.Dispose()
}

Resize-And-Save-Bmp $srcImg 164 314 $bmp1xDest
Resize-And-Save-Bmp $srcImg 328 628 $bmp2xDest
$srcImg.Dispose()

Write-Host "Wizard banner bitmaps generated successfully:"
Get-Item (Join-Path $rootDir "assets\wizard_banner*") | Select-Object Name, Length
