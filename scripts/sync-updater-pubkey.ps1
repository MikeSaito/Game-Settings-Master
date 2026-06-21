# Copy public key from ~/.tauri/*.pub into src-tauri/tauri.conf.json

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$pubPath = Join-Path $env:USERPROFILE ".tauri\game-settings-master.key.pub"
$confPath = Join-Path $root "src-tauri\tauri.conf.json"

if (-not (Test-Path $pubPath)) {
    Write-Error "Missing $pubPath. Run .\scripts\generate-updater-keys.ps1 first"
}

$pubkey = (Get-Content $pubPath -Raw).Trim()
$content = Get-Content $confPath -Raw
if ($content -match '"pubkey"\s*:\s*"[^"]*"') {
    $content = $content -replace '"pubkey"\s*:\s*"[^"]*"', ('"pubkey": "' + $pubkey + '"')
} else {
    Write-Error "plugins.updater.pubkey not found in tauri.conf.json"
}

$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($confPath, $content.TrimEnd() + "`n", $utf8NoBom)

Write-Host "Updated pubkey in $confPath" -ForegroundColor Green
