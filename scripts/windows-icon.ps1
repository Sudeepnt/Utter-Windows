# Regenerates the Windows app icon from assets/icons/utter-logo.png.
param([string]$Out)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
if (-not $Out) { $Out = Join-Path $root "resources\windows\utter.ico" }
$glyph = [Drawing.Image]::FromFile((Join-Path $root "assets\icons\utter-logo.png"))
$sizes = 256, 64, 48, 40, 32, 24, 20, 16
$frames = foreach ($size in $sizes) {
    $bitmap = New-Object Drawing.Bitmap $size, $size
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = "AntiAlias"
    $graphics.InterpolationMode = "HighQualityBicubic"
    $graphics.PixelOffsetMode = "HighQuality"
    $graphics.DrawImage($glyph, 0, 0, $size, $size)
    $graphics.Dispose()

    $stream = New-Object IO.MemoryStream
    if ($size -ge 256) {
        # Vista and later read the large frame as PNG.
        $bitmap.Save($stream, [Drawing.Imaging.ImageFormat]::Png)
    } else {
        # Small frames stay classic 32-bit DIBs: header, bottom-up BGRA rows,
        # then an all-zero AND mask padded to 32-bit rows.
        $dib = New-Object IO.BinaryWriter $stream
        $dib.Write([UInt32]40); $dib.Write([Int32]$size); $dib.Write([Int32]($size * 2))
        $dib.Write([UInt16]1); $dib.Write([UInt16]32); $dib.Write([UInt32]0)
        $dib.Write([UInt32]($size * $size * 4)); $dib.Write([Int32]0); $dib.Write([Int32]0)
        $dib.Write([UInt32]0); $dib.Write([UInt32]0)
        for ($y = $size - 1; $y -ge 0; $y--) {
            for ($x = 0; $x -lt $size; $x++) {
                $pixel = $bitmap.GetPixel($x, $y)
                $dib.Write([byte]$pixel.B); $dib.Write([byte]$pixel.G)
                $dib.Write([byte]$pixel.R); $dib.Write([byte]$pixel.A)
            }
        }
        $dib.Write((New-Object byte[] ([Math]::Ceiling($size / 32) * 4 * $size)))
        $dib.Flush()
    }
    $bitmap.Dispose()
    , $stream.ToArray()
}
$glyph.Dispose()

# ICO container: header, one directory entry per frame, then the payloads.
$icon = New-Object IO.MemoryStream
$writer = New-Object IO.BinaryWriter $icon
$writer.Write([UInt16]0); $writer.Write([UInt16]1); $writer.Write([UInt16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $dimension = if ($sizes[$i] -ge 256) { 0 } else { $sizes[$i] }
    $writer.Write([byte]$dimension); $writer.Write([byte]$dimension)
    $writer.Write([byte]0); $writer.Write([byte]0)
    $writer.Write([UInt16]1); $writer.Write([UInt16]32)
    $writer.Write([UInt32]$frames[$i].Length); $writer.Write([UInt32]$offset)
    $offset += $frames[$i].Length
}
foreach ($frame in $frames) { $writer.Write($frame) }
$writer.Flush()
[IO.File]::WriteAllBytes($Out, $icon.ToArray())
"wrote $Out ($($icon.Length) bytes)"
