# Pre-commit: fast frontend checks (cargo test stays in CI only).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

npm test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

powershell -ExecutionPolicy Bypass -File (Join-Path $root "scripts\verify-types-sync.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
