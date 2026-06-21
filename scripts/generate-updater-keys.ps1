# Run in Windows Terminal or PowerShell (not Cursor background agent).
# Tauri will prompt for a password interactively.

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$keyPath = Join-Path $env:USERPROFILE ".tauri\game-settings-master.key"
$keyDir = Split-Path $keyPath -Parent
if (-not (Test-Path $keyDir)) {
    New-Item -ItemType Directory -Path $keyDir -Force | Out-Null
}

Write-Host ""
Write-Host "=== Game Settings Master updater keys ===" -ForegroundColor Cyan
Write-Host "Key file: $keyPath"
Write-Host ""
Write-Host "Tauri will ask for a password twice." -ForegroundColor Yellow
Write-Host "Characters may not show while typing. That is normal."
Write-Host ""

npm run tauri signer generate -- -w $keyPath --force

if ($LASTEXITCODE -ne 0) {
    Write-Host "Key generation failed." -ForegroundColor Red
    exit 1
}

& (Join-Path $PSScriptRoot "sync-updater-pubkey.ps1")

Write-Host ""
Write-Host "Done. Next steps:" -ForegroundColor Green
Write-Host "  1. GitHub Secret TAURI_SIGNING_PRIVATE_KEY = full content of $keyPath"
Write-Host "  2. GitHub Secret TAURI_SIGNING_PRIVATE_KEY_PASSWORD = your password"
Write-Host "  3. Local signed build: .\scripts\build-signed.ps1"
Write-Host ""
