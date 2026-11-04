# Tight crop of the app mark for the small icon sizes (wave-2-layout.md 14,
# "For W2-Q (icon set)"): at 16-32 px the full 1024 canvas leaves the star's
# rays and the book's page edge below a pixel, so those sizes are made from
# the mark's bounding box plus 6% padding on each side, on the same #080B12
# square. Built-in Windows WPF imaging only (no extra tools), like
# assets/brand/make-icon.ps1. The image is cropped, never recoloured or redrawn.
#
#   powershell -NoProfile -STA -File app\scripts\make-icon-small.ps1 `
#     -Src assets\brand\revelation-icon-1024.png -Out <dir>\mark-tight.png
#
# BRAND-LOGO (sources/SOURCES.md): not under the MIT License; see
# assets/brand/LICENSE.
param([string]$Src, [string]$Out, [int]$Threshold = 64, [double]$Pad = 0.06, [switch]$MeasureOnly)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName PresentationCore, WindowsBase
$frame = [System.Windows.Media.Imaging.BitmapDecoder]::Create([uri](Resolve-Path $Src).Path, 'None', 'OnLoad').Frames[0]
$bmp = New-Object System.Windows.Media.Imaging.FormatConvertedBitmap($frame, [System.Windows.Media.PixelFormats]::Bgra32, $null, 0)
$w = $bmp.PixelWidth; $h = $bmp.PixelHeight
$stride = $w * 4
$px = New-Object byte[] ($stride * $h)
$bmp.CopyPixels($px, $stride, 0)

# The mark: any pixel whose brightest channel is at least $Threshold above the
# background's (#080B12, brightest channel 0x12). The soft edges of the canvas
# stay far below that.
$minX = $w; $minY = $h; $maxX = -1; $maxY = -1
$limit = 0x12 + $Threshold
for ($y = 0; $y -lt $h; $y++) {
  $row = $y * $stride
  for ($x = 0; $x -lt $w; $x++) {
    $i = $row + $x * 4
    $m = [Math]::Max($px[$i], [Math]::Max($px[$i + 1], $px[$i + 2]))
    if ($m -ge $limit) {
      if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
      if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
    }
  }
}
if ($maxX -lt 0) { throw "no mark found above threshold $Threshold" }
$bw = $maxX - $minX + 1; $bh = $maxY - $minY + 1
$side = [int][Math]::Ceiling([Math]::Max($bw, $bh) * (1 + 2 * $Pad))
$cx = ($minX + $maxX + 1) / 2.0; $cy = ($minY + $maxY + 1) / 2.0
"mark bbox x $minX-$maxX, y $minY-$maxY ($bw x $bh); crop side $side"
if ($MeasureOnly) { return }

# Square crop centred on the mark, drawn onto the background colour (a crop
# that would run past the canvas edge is padded with the same colour).
$fill = [System.Windows.Media.Color]::FromRgb(0x08, 0x0B, 0x12)
$dv = New-Object System.Windows.Media.DrawingVisual
$dc = $dv.RenderOpen()
$dc.DrawRectangle((New-Object System.Windows.Media.SolidColorBrush($fill)), $null, (New-Object System.Windows.Rect(0, 0, $side, $side)))
$dc.DrawImage($frame, (New-Object System.Windows.Rect(($side / 2.0 - $cx), ($side / 2.0 - $cy), $w, $h)))
$dc.Close()
$rtb = New-Object System.Windows.Media.Imaging.RenderTargetBitmap($side, $side, 96, 96, [System.Windows.Media.PixelFormats]::Pbgra32)
$rtb.Render($dv)
$enc = New-Object System.Windows.Media.Imaging.PngBitmapEncoder
$enc.Frames.Add([System.Windows.Media.Imaging.BitmapFrame]::Create($rtb))
$fs = [System.IO.File]::Create($Out)
$enc.Save($fs)
$fs.Close()
"wrote $Out ($side x $side)"
