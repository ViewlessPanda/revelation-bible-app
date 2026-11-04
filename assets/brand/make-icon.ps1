param([string]$Src, [string]$Out)
Add-Type -AssemblyName PresentationCore, WindowsBase
$frame = [System.Windows.Media.Imaging.BitmapDecoder]::Create([uri]$Src, 'None', 'OnLoad').Frames[0]

# Mark (star + book) measured at x 331-919, y 225-691; text starts at y 734.
$crop = New-Object System.Windows.Media.Imaging.CroppedBitmap($frame, (New-Object System.Windows.Int32Rect(181, 160, 888, 565)))
$scale = 1.19
$cw = 888 * $scale; $ch = 565 * $scale
$markCx = (625 - 181) * $scale; $markCy = (458 - 160) * $scale
$x = 512 - $markCx; $y = 512 - $markCy

$fill = [System.Windows.Media.Color]::FromRgb(0x08, 0x0B, 0x12)
$dv = New-Object System.Windows.Media.DrawingVisual
$dc = $dv.RenderOpen()
$dc.DrawRectangle((New-Object System.Windows.Media.SolidColorBrush($fill)), $null, (New-Object System.Windows.Rect(0, 0, 1024, 1024)))
# Soft edges (horizontal and vertical masks multiply) so the crop blends into the fill.
function New-Fade($vertical, $a, $b) {
  $g = New-Object System.Windows.Media.LinearGradientBrush
  if ($vertical) { $g.StartPoint = '0,0'; $g.EndPoint = '0,1' } else { $g.StartPoint = '0,0'; $g.EndPoint = '1,0' }
  foreach ($s in @(@('Transparent',0.0),@('Black',$a),@('Black',$b),@('Transparent',1.0))) {
    $g.GradientStops.Add((New-Object System.Windows.Media.GradientStop([System.Windows.Media.Colors]::($s[0]), $s[1]))) }
  $g }
$dc.PushOpacityMask((New-Fade $false 0.12 0.88))
$dc.PushOpacityMask((New-Fade $true 0.10 0.95))
$dc.DrawImage($crop, (New-Object System.Windows.Rect($x, $y, $cw, $ch)))
$dc.Pop()
$dc.Pop()
$dc.Close()

$rtb = New-Object System.Windows.Media.Imaging.RenderTargetBitmap(1024, 1024, 96, 96, [System.Windows.Media.PixelFormats]::Pbgra32)
$rtb.Render($dv)
$enc = New-Object System.Windows.Media.Imaging.PngBitmapEncoder
$enc.Frames.Add([System.Windows.Media.Imaging.BitmapFrame]::Create($rtb))
$fs = [System.IO.File]::Create($Out)
$enc.Save($fs)
$fs.Close()
"wrote $Out"
