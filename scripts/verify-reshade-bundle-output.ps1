# Fail if Tauri build did not place ReShade DLLs at the expected resource paths.
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "reshade-common.ps1")
$RepoRoot = Split-Path -Parent $PSScriptRoot
$ReleaseDir = Join-Path $RepoRoot "src-tauri\target\release"
$MinDllBytes = 65536

function Test-ValidDll([string]$Path) {
    return (Test-Path $Path) -and (Get-Item $Path).Length -ge $MinDllBytes
}

$expected = Join-Path $ReleaseDir "presets\reshade\bin\dxgi.dll"
if (-not (Test-ValidDll $expected)) {
    $flat = Join-Path $ReleaseDir "presets\reshade\dxgi.dll"
    $size = if (Test-Path $expected) { (Get-Item $expected).Length }
            elseif (Test-Path $flat) { (Get-Item $flat).Length } else { 0 }
    Write-Error @(
        "ReShade dxgi.dll missing from Tauri build output at expected path:",
        "  $expected",
        "Found flat layout at $flat : $(Test-Path $flat) ($size bytes).",
        "Use directory resource mappings in tauri.conf.json (not ** globs)."
    )
    exit 1
}

Write-Host "ReShade bundle output OK: $expected" -ForegroundColor Green
exit 0
