# Generates assets/capslang.ico and assets/capslang-off.ico from code, so the
# icons are reproducible and reviewable in the repo. Requires Windows.
Add-Type -AssemblyName System.Drawing

$OutDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function New-ArrowPath {
    param($X0, $X1, $Y, $Half, $HeadLen, $HeadHalf)
    $dir = if ($X1 -gt $X0) { 1 } else { -1 }
    $neck = $X1 - ($dir * $HeadLen)
    $pts = @(
        [System.Drawing.PointF]::new($X0,   $Y - $Half),
        [System.Drawing.PointF]::new($neck, $Y - $Half),
        [System.Drawing.PointF]::new($neck, $Y - $HeadHalf),
        [System.Drawing.PointF]::new($X1,   $Y),
        [System.Drawing.PointF]::new($neck, $Y + $HeadHalf),
        [System.Drawing.PointF]::new($neck, $Y + $Half),
        [System.Drawing.PointF]::new($X0,   $Y + $Half)
    )
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $p.AddPolygon($pts)
    return $p
}

function New-RoundedRect {
    param($X, $Y, $W, $H, $R)
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $R * 2
    $p.AddArc($X,             $Y,             $d, $d, 180, 90)
    $p.AddArc($X + $W - $d,   $Y,             $d, $d, 270, 90)
    $p.AddArc($X + $W - $d,   $Y + $H - $d,   $d, $d, 0,   90)
    $p.AddArc($X,             $Y + $H - $d,   $d, $d, 90,  90)
    $p.CloseFigure()
    return $p
}

function New-Master {
    param([string]$From, [string]$To)
    $bmp = New-Object System.Drawing.Bitmap 256, 256
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.InterpolationMode = 'HighQualityBicubic'
    $g.Clear([System.Drawing.Color]::Transparent)

    $bg = New-RoundedRect 12 12 232 232 52
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (New-Object System.Drawing.Point 0, 0),
        (New-Object System.Drawing.Point 256, 256),
        [System.Drawing.ColorTranslator]::FromHtml($From),
        [System.Drawing.ColorTranslator]::FromHtml($To))
    $g.FillPath($brush, $bg)

    $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $top = New-ArrowPath 62 196 104 13 44 34
    $bot = New-ArrowPath 194 60 156 13 44 34
    $g.FillPath($white, $top)
    $g.FillPath($white, $bot)

    $top.Dispose(); $bot.Dispose(); $bg.Dispose()
    $white.Dispose(); $brush.Dispose(); $g.Dispose()
    return $bmp
}

function Save-Ico {
    param([System.Drawing.Bitmap]$Master, [string]$Path)
    $sizes = @(16, 24, 32, 48, 64, 128, 256)
    $blobs = @()
    foreach ($s in $sizes) {
        $b = New-Object System.Drawing.Bitmap $s, $s
        $g = [System.Drawing.Graphics]::FromImage($b)
        $g.InterpolationMode = 'HighQualityBicubic'
        $g.PixelOffsetMode = 'HighQuality'
        $g.SmoothingMode = 'AntiAlias'
        $g.Clear([System.Drawing.Color]::Transparent)
        $g.DrawImage($Master, (New-Object System.Drawing.Rectangle 0, 0, $s, $s))
        $g.Dispose()
        $ms = New-Object System.IO.MemoryStream
        $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $blobs += ,@($s, $ms.ToArray())
        $ms.Dispose(); $b.Dispose()
    }

    $out = New-Object System.IO.MemoryStream
    $w = New-Object System.IO.BinaryWriter $out
    $w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$blobs.Count)
    $offset = 6 + (16 * $blobs.Count)
    foreach ($e in $blobs) {
        $dim = if ($e[0] -ge 256) { 0 } else { $e[0] }
        $w.Write([byte]$dim); $w.Write([byte]$dim)
        $w.Write([byte]0); $w.Write([byte]0)
        $w.Write([uint16]1); $w.Write([uint16]32)
        $w.Write([uint32]$e[1].Length); $w.Write([uint32]$offset)
        $offset += $e[1].Length
    }
    foreach ($e in $blobs) { $w.Write($e[1]) }
    $w.Flush()
    [System.IO.File]::WriteAllBytes($Path, $out.ToArray())
    $w.Dispose(); $out.Dispose()
    Write-Host "wrote $Path ($([math]::Round((Get-Item $Path).Length / 1kb, 1)) KB)"
}

$on = New-Master '#3B82F6' '#6366F1'
Save-Ico $on (Join-Path $OutDir 'capslang.ico')
$on.Dispose()

$off = New-Master '#9CA3AF' '#6B7280'
Save-Ico $off (Join-Path $OutDir 'capslang-off.ico')
$off.Dispose()
