<#
.SYNOPSIS
    由 src-tauri/icons/source 的 SVG 產生 app、工作列、瀏覽器擴充與網頁的圖示。修改 SVG 後執行一次並提交產物。

    24 px 以下用 icon-small.svg（箭桿實心、托盤加粗），其餘用 icon.svg。
    以 Edge headless 繪製，使用暫存的設定檔資料夾，不影響使用者的 Edge。
    png / icns / Square*Logo 交給 `tauri icon`；icon.ico 自己組（才能放小尺寸版，32 px 放第一層）。

.EXAMPLE
    pwsh scripts/make-icon.ps1
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$source = Join-Path $root 'src-tauri\icons\source'
$icons = Join-Path $root 'src-tauri\icons'
$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $edge) { throw '找不到 Microsoft Edge' }

$work = Join-Path $env:TEMP ("zoo-icon-{0}" -f [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory $work | Out-Null
Copy-Item (Join-Path $source '*.svg') $work

function Render([int]$size) {
    $svg = if ($size -le 24) { 'icon-small.svg' } else { 'icon.svg' }
    $html = Join-Path $work "icon-$size.html"
    Set-Content $html "<!doctype html><html><body style=`"margin:0;background:transparent`"><img src=`"$svg`" width=`"$size`" height=`"$size`" style=`"display:block`"></body></html>" -Encoding utf8
    $png = Join-Path $work "icon-$size.png"
    & $edge --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --default-background-color=00000000 `
        --user-data-dir="$(Join-Path $work 'profile')" --window-size="$size,$size" --screenshot="$png" ("file:///" + ($html -replace '\\', '/')) 2>$null | Out-Null
    for ($i = 0; $i -lt 50 -and -not (Test-Path $png); $i++) { Start-Sleep -Milliseconds 100 }
    if (-not (Test-Path $png)) { throw "繪製 $size px 失敗" }
    $png
}

try {
    # png、icns、Square*Logo
    $master = Render 1024
    & (Join-Path $root 'node_modules\.bin\tauri.cmd') icon $master -o $icons | Out-Host
    if ($LASTEXITCODE) { throw 'tauri icon 失敗' }
    # 只做桌面版，不留行動平台的產物
    Remove-Item -Recurse -Force (Join-Path $icons 'android'), (Join-Path $icons 'ios') -ErrorAction SilentlyContinue

    # icon.ico：ICONDIR + 每層一筆 ICONDIRENTRY，影像資料直接放 PNG；32 px 放第一層（開發時 Windows 取第一層）
    $sizes = 32, 16, 20, 24, 40, 48, 64, 96, 128, 256
    $layers = foreach ($s in $sizes) { [pscustomobject]@{ Size = $s; Bytes = [IO.File]::ReadAllBytes((Render $s)) } }
    $ms = New-Object IO.MemoryStream
    $w = New-Object IO.BinaryWriter $ms
    $w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$layers.Count)
    $offset = 6 + 16 * $layers.Count
    foreach ($l in $layers) {
        $dim = if ($l.Size -ge 256) { 0 } else { $l.Size }
        $w.Write([byte]$dim); $w.Write([byte]$dim); $w.Write([byte]0); $w.Write([byte]0)
        $w.Write([uint16]1); $w.Write([uint16]32)
        $w.Write([uint32]$l.Bytes.Length); $w.Write([uint32]$offset)
        $offset += $l.Bytes.Length
    }
    foreach ($l in $layers) { $w.Write($l.Bytes) }
    $w.Flush()
    [IO.File]::WriteAllBytes((Join-Path $icons 'icon.ico'), $ms.ToArray())

    # 瀏覽器擴充
    foreach ($s in 16, 32, 48, 128) {
        Copy-Item (Render $s) (Join-Path $root "browser-extension\icons\icon$s.png") -Force
    }

    # 網頁 favicon
    Copy-Item (Join-Path $source 'icon.svg') (Join-Path $root 'public\favicon.svg') -Force

    Write-Host '已產生 src-tauri/icons、browser-extension/icons、public/favicon.svg'
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
