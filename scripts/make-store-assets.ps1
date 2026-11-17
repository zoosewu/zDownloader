<#
.SYNOPSIS
    由 src-tauri/icons/source/icon.svg 產生瀏覽器商店的圖：商店標誌 300×300、宣傳圖塊 440×280。
    產物放在 store/，修改圖示後執行一次並提交。以 Edge headless 繪製，使用暫存的設定檔資料夾。

.EXAMPLE
    pwsh scripts/make-store-assets.ps1
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root 'store'
$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $edge) { throw '找不到 Microsoft Edge' }

$work = Join-Path $env:TEMP ("zoo-store-{0}" -f [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory $work | Out-Null
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item (Join-Path $root 'src-tauri\icons\source\icon.svg') $work

# Neutral dark surface and text, as the app's dark theme (styles/theme.ts).
$page = @'
<!doctype html><html><head><meta charset="utf-8"><style>
html, body { margin: 0; width: {W}px; height: {H}px; overflow: hidden; background: {BG}; }
.tile { display: flex; align-items: center; justify-content: center; gap: 22px; width: 100%; height: 100%;
        font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif; color: hsl(220, 6%, 91%); }
.name { font-size: 34px; font-weight: 600; letter-spacing: 0.2px; line-height: 1.15; }
.line { font-size: 15px; color: hsl(220, 5%, 64%); margin-top: 6px; }
</style></head><body>{BODY}</body></html>
'@

function Render([string]$name, [int]$w, [int]$h, [string]$bg, [string]$body) {
    $html = Join-Path $work "$name.html"
    $page.Replace('{W}', $w).Replace('{H}', $h).Replace('{BG}', $bg).Replace('{BODY}', $body) | Set-Content $html -Encoding utf8
    $png = Join-Path $out "$name.png"
    Remove-Item $png -ErrorAction SilentlyContinue
    & $edge --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --default-background-color=00000000 `
        --user-data-dir="$(Join-Path $work 'profile')" --window-size="$w,$h" --screenshot="$png" ("file:///" + ($html -replace '\\', '/')) 2>$null | Out-Null
    for ($i = 0; $i -lt 50 -and -not (Test-Path $png); $i++) { Start-Sleep -Milliseconds 100 }
    if (-not (Test-Path $png)) { throw "繪製 $name 失敗" }
    Write-Host "store/$name.png ($w x $h)"
}

try {
    # The logo is the icon itself on a transparent background, like the app icon.
    Render 'logo-300' 300 300 'transparent' '<img src="icon.svg" width="300" height="300" style="display:block">'
    Render 'tile-440x280' 440 280 'hsl(220, 5%, 12%)' `
        '<div class="tile"><img src="icon.svg" width="112" height="112"><div><div class="name">Zoo<br>Downloader</div><div class="line">Downloads, handed to the app</div></div></div>'
    Render 'marquee-1400x560' 1400 560 'hsl(220, 5%, 12%)' `
        '<div class="tile" style="gap:56px"><img src="icon.svg" width="240" height="240"><div><div class="name" style="font-size:84px">Zoo Downloader</div><div class="line" style="font-size:30px;margin-top:16px">Your downloads, handed to the desktop app.</div></div></div>'
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
