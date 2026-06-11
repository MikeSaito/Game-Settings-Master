# Regenerate src-tauri/presets/reshade/binary-hashes.json from current bin/ + optional Setup.exe.
# Run after npm run reshade:setup when bumping ReShade version.

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "reshade-common.ps1")
$RepoRoot = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $RepoRoot "src-tauri\presets\reshade\bin"
$ManifestPath = Join-Path $RepoRoot "src-tauri\presets\reshade\binary-hashes.json"

$version = "6.7.3"
$setupUrl = "https://reshade.me/downloads/ReShade_Setup_6.7.3_Addon.exe"
if (Test-Path $ManifestPath) {
    $existing = Get-Content $ManifestPath -Raw | ConvertFrom-Json
    if ($existing.version) { $version = [string]$existing.version }
    if ($existing.setup.url) { $setupUrl = [string]$existing.setup.url }
}

$names = @(
    "ReShade64.dll", "ReShade64.json",
    "dxgi.dll", "d3d11.dll", "d3d9.dll", "opengl32.dll"
)
$files = @{}
foreach ($name in $names) {
    $path = Join-Path $BinDir $name
    if (Test-Path $path) {
        $files[$name] = (Get-Sha256Hex $path).ToLowerInvariant()
    }
}

$setupSha = ""
$setupPath = $args[0]
if (-not $setupPath) {
    $setupPath = Join-Path $env:TEMP "ReShade_Setup_Addon.exe"
}
if (Test-Path $setupPath) {
    $setupSha = (Get-Sha256Hex $setupPath).ToLowerInvariant()
}

$manifest = [ordered]@{
    version = $version
    setup   = [ordered]@{
        url    = $setupUrl
        sha256 = $setupSha
    }
    files = $files
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -Path $ManifestPath -Encoding utf8
Write-Host "Updated $ManifestPath ($($files.Count) file hashes, setup SHA256 pinned)." -ForegroundColor Green
