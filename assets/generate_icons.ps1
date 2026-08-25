# generate_icons.ps1 — regenerates the egui-app icon set in assets/
#
# The icon is a solid, full-coverage black square (no transparency, no other
# colors) rendered at multiple sizes and packed into the platform containers:
#
#   assets/icon.png         512x512 master (Linux / runtime)
#   assets/icon-32.png      32x32
#   assets/icon-128.png     128x128
#   assets/icon-256.png     256x256
#   assets/icon.ico         Windows multi-size (16..256, PNG entries)
#   assets/icon.icns        macOS multi-size (16..1024, PNG entries)
#
# Requires PowerShell 5.1+ on Windows (System.Drawing).
# Usage:  powershell -ExecutionPolicy Bypass -File assets\generate_icons.ps1

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$assetsDir = Join-Path $PSScriptRoot '.'
$tmpDir     = Join-Path $env:TEMP ("egui_icons_" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

# --- drawing ---------------------------------------------------------------

function New-IconBitmap([int]$size) {
    $bmp = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)

    # Full-coverage solid black, no transparency, no other colors.
    $black = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 0, 0, 0))
    $g.FillRectangle($black, 0, 0, $size, $size)

    $black.Dispose(); $g.Dispose()
    return $bmp
}

# --- containers ------------------------------------------------------------

function Write-BE16($bw, [uint16]$v) {
    $bw.Write([byte](($v -shr 8) -band 0xFF)); $bw.Write([byte]($v -band 0xFF))
}

function Write-BE32($bw, [uint32]$v) {
    $bw.Write([byte](($v -shr 24) -band 0xFF)); $bw.Write([byte](($v -shr 16) -band 0xFF))
    $bw.Write([byte](($v -shr 8) -band 0xFF));  $bw.Write([byte]($v -band 0xFF))
}

# Windows .ico — PNG-compressed entries (Vista+; Windows 10/11 target).
function New-IcoFile([int[]]$sizes, [string]$outPath) {
    $count = $sizes.Count
    $offset = 6 + 16 * $count
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)
    $bw.Write([uint16]0)                    # reserved
    $bw.Write([uint16]1)                    # type: icon
    $bw.Write([uint16]$count)
    foreach ($s in $sizes) {
        $dim = if ($s -ge 256) { 0 } else { $s }
        $bw.Write([byte]$dim); $bw.Write([byte]$dim)
        $bw.Write([byte]0); $bw.Write([byte]0)
        $bw.Write([uint16]1); $bw.Write([uint16]32)
        $png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir "icon-$s.png"))
        $bw.Write([uint32]$png.Length)
        $bw.Write([uint32]$offset)
        $offset += $png.Length
    }
    foreach ($s in $sizes) {
        $bw.Write([System.IO.File]::ReadAllBytes((Join-Path $tmpDir "icon-$s.png")))
    }
    $bw.Flush()
    [System.IO.File]::WriteAllBytes($outPath, $ms.ToArray())
    $bw.Dispose(); $ms.Dispose()
}

# macOS .icns — PNG-embedded chunks (macOS 10.7+).
function New-IcnsFile([object[]]$chunks, [string]$outPath) {
    $total = 8
    foreach ($c in $chunks) { $total += 8 + $c.Png.Length }
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)
    $bw.Write([Text.Encoding]::ASCII.GetBytes('icns'))
    Write-BE32 $bw ([uint32]$total)
    foreach ($c in $chunks) {
        $bw.Write([Text.Encoding]::ASCII.GetBytes($c.Type))
        Write-BE32 $bw ([uint32](8 + $c.Png.Length))
        $bw.Write($c.Png)
    }
    $bw.Flush()
    [System.IO.File]::WriteAllBytes($outPath, $ms.ToArray())
    $bw.Dispose(); $ms.Dispose()
}

# --- render ----------------------------------------------------------------

$allSizes = 16, 24, 32, 48, 64, 128, 256, 512, 1024
foreach ($s in $allSizes) {
    $bmp = New-IconBitmap $s
    $bmp.Save((Join-Path $tmpDir "icon-$s.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "rendered $s x $s"
}

# PNGs shipped in the repo
Copy-Item (Join-Path $tmpDir 'icon-512.png') (Join-Path $assetsDir 'icon.png')
Copy-Item (Join-Path $tmpDir 'icon-32.png')  (Join-Path $assetsDir 'icon-32.png')
Copy-Item (Join-Path $tmpDir 'icon-128.png') (Join-Path $assetsDir 'icon-128.png')
Copy-Item (Join-Path $tmpDir 'icon-256.png') (Join-Path $assetsDir 'icon-256.png')

# Windows .ico (16, 24, 32, 48, 64, 256)
New-IcoFile @(16, 24, 32, 48, 64, 256) (Join-Path $assetsDir 'icon.ico')

# macOS .icns (16, 32, 64, 128, 256, 512, 1024)
$icnsChunks = @(
    @{ Type = 'icp4'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-16.png')) },
    @{ Type = 'icp5'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-32.png')) },
    @{ Type = 'icp6'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-64.png')) },
    @{ Type = 'ic07'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-128.png')) },
    @{ Type = 'ic08'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-256.png')) },
    @{ Type = 'ic09'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-512.png')) },
    @{ Type = 'ic10'; Png = [System.IO.File]::ReadAllBytes((Join-Path $tmpDir 'icon-1024.png')) }
)
New-IcnsFile $icnsChunks (Join-Path $assetsDir 'icon.icns')

Remove-Item -Recurse -Force $tmpDir
Write-Host "Done. Icon set written to $assetsDir"
