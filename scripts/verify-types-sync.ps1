# Fail when src/lib/bindings.ts is out of sync with Rust DTOs (run after editing models.rs).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$bindings = Join-Path $root "src\lib\bindings.ts"
$backup = Join-Path $env:TEMP "gsm-bindings-backup-$([guid]::NewGuid().ToString()).ts"

Copy-Item -LiteralPath $bindings -Destination $backup -Force

Push-Location (Join-Path $root "src-tauri")
try {
    $PSNativeCommandUseErrorActionPreference = $false
    cargo test export_typescript_bindings -- --nocapture 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

$diff = & git -C $root diff --no-color -- src/lib/bindings.ts
if ($diff) {
    Copy-Item -LiteralPath $backup -Destination $bindings -Force
    Write-Error @"
src/lib/bindings.ts is out of sync with Rust models.
Run: npm run types:gen
Then commit the updated bindings.ts

$diff
"@
    exit 1
}

Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue
Write-Host "bindings.ts is in sync with Rust DTOs."
