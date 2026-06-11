# Fail CI/local build if ReShade bundle is missing or contains stubs.
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "reshade-common.ps1")
$RepoRoot = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $RepoRoot "src-tauri\presets\reshade\bin"
$ShadersDir = Join-Path $RepoRoot "src-tauri\presets\reshade\shaders\Shaders"
$ManifestPath = Join-Path $RepoRoot "src-tauri\presets\reshade\binary-hashes.json"
$MinDllBytes = 65536

function Test-ValidDll([string]$Path) {
    return (Test-Path $Path) -and (Get-Item $Path).Length -ge $MinDllBytes
}

$requiredDlls = @(
    "dxgi.dll", "d3d11.dll", "d3d9.dll", "opengl32.dll", "ReShade64.dll"
)
foreach ($name in $requiredDlls) {
    $path = Join-Path $BinDir $name
    if (-not (Test-ValidDll $path)) {
        $size = if (Test-Path $path) { (Get-Item $path).Length } else { 0 }
        Write-Error "ReShade bundle invalid: $name missing or stub ($size bytes). Run: npm run reshade:setup"
        exit 1
    }
}

$legacyVk = Join-Path $BinDir "VkLayer_reshade.dll"
if (Test-Path $legacyVk) {
    Write-Error "ReShade bundle invalid: remove legacy VkLayer_reshade.dll (ReShade 6.7+ uses ReShade64.json). Run: npm run reshade:setup"
    exit 1
}

$json = Join-Path $BinDir "ReShade64.json"
if (-not (Test-Path $json) -or (Get-Item $json).Length -lt 50) {
    Write-Error "ReShade bundle invalid: ReShade64.json missing or empty."
    exit 1
}

try {
    $manifest = Get-Content $json -Raw | ConvertFrom-Json
    if (-not $manifest.layer.name) {
        throw "missing layer.name"
    }
} catch {
    Write-Error "ReShade bundle invalid: ReShade64.json is not a valid Vulkan layer manifest."
    exit 1
}

if (Test-Path $ManifestPath) {
    $pins = Get-Content $ManifestPath -Raw | ConvertFrom-Json
    if ($pins.files) {
        foreach ($prop in $pins.files.PSObject.Properties) {
            $name = $prop.Name
            $expected = [string]$prop.Value
            if (-not $expected -or $expected.Trim().Length -eq 0) { continue }
            $path = Join-Path $BinDir $name
            if (-not (Test-Path $path)) {
                Write-Error "ReShade bundle invalid: $name missing (pinned in binary-hashes.json)."
                exit 1
            }
            $actual = (Get-Sha256Hex $path).ToLowerInvariant()
            if ($actual -ne $expected.ToLowerInvariant()) {
                Write-Error "ReShade bundle SHA256 mismatch for ${name}."
                exit 1
            }
        }
    }
}

$fxCount = (Get-ChildItem -Path $ShadersDir -Filter "*.fx" -Recurse -ErrorAction SilentlyContinue).Count
if ($fxCount -lt 1) {
    Write-Error "ReShade bundle invalid: no .fx shaders in $ShadersDir. Run: npm run reshade:setup"
    exit 1
}

Write-Host "ReShade bundle OK: $($requiredDlls.Count) DLLs, Vulkan manifest, $fxCount shader files." -ForegroundColor Green
exit 0
