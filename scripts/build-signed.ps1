# Local release build with updater signing (password-protected key).

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$keyPath = Join-Path $env:USERPROFILE ".tauri\game-settings-master.key"
if (-not (Test-Path $keyPath)) {
    Write-Error "Key not found. Run: .\scripts\generate-updater-keys.ps1"
}

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $keyPath -Raw

if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    $secure = Read-Host "Signing key password" -AsSecureString
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try {
        $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
    } finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
    }
}

Write-Host "Building with updater signature..." -ForegroundColor Cyan
npm run tauri build

if ($LASTEXITCODE -eq 0) {
    $sig = Get-ChildItem "src-tauri\target\release\bundle\nsis\*.sig" -ErrorAction SilentlyContinue
    if ($sig) {
        Write-Host "Signature created: $($sig.FullName)" -ForegroundColor Green
    } else {
        Write-Host "Warning: no .sig file. Check password and pubkey in tauri.conf.json" -ForegroundColor Yellow
    }
}
