# Point git at repo hooks (.githooks/pre-commit runs npm test + verify-types-sync).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

& git config core.hooksPath .githooks
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$hook = Join-Path $root ".githooks\pre-commit"
if (-not (Test-Path -LiteralPath $hook)) {
    Write-Error "Missing hook: $hook"
    exit 1
}

# Unix: ensure shell hook is executable.
if ($IsLinux -or $IsMacOS) {
    chmod +x $hook 2>$null
}

Write-Host "Git hooks installed (core.hooksPath=.githooks)."
Write-Host "Pre-commit runs: npm test, scripts/verify-types-sync.ps1"
