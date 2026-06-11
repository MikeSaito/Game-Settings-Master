# Runs before `tauri build` — fails release bundle if ReShade resources are missing/stub.
# Skip locally: $env:GSM_SKIP_RESHADE_VERIFY = "1"

$ErrorActionPreference = "Stop"

if ($env:GSM_SKIP_RESHADE_VERIFY -eq "1") {
    Write-Host "ReShade bundle verify skipped (GSM_SKIP_RESHADE_VERIFY=1)." -ForegroundColor Yellow
    exit 0
}

$Verify = Join-Path $PSScriptRoot "verify-reshade-bundle.ps1"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File $Verify
exit $LASTEXITCODE
