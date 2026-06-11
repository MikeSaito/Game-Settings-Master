# Extract ReShade addon DLLs from official Setup.exe into GSM bundle (dev + release build).
# Usage: .\scripts\fetch-reshade-binaries.ps1 [path-to-ReShade_Setup_*_Addon.exe]
# Requires 7-Zip. Without a local path, downloads the pinned addon build from reshade.me.
# ReShade 6.7+ ships ReShade64.dll + ReShade64.json for Vulkan (no VkLayer_reshade.dll).

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $RepoRoot "src-tauri\presets\reshade\bin"

function Find-7Zip {
    @("C:\Program Files\7-Zip\7z.exe", "C:\Program Files (x86)\7-Zip\7z.exe") |
        Where-Object { Test-Path $_ } |
        Select-Object -First 1
}

$7z = Find-7Zip
if (-not $7z) {
    $choco = Get-Command choco -ErrorAction SilentlyContinue
    if ($choco) {
        Write-Host "7-Zip not found - installing via Chocolatey..."
        & choco install 7zip -y --no-progress | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Chocolatey failed to install 7-Zip (exit $LASTEXITCODE)."
            exit 1
        }
        $7z = Find-7Zip
    }
}
if (-not $7z) {
    Write-Error "7-Zip is required. Install from https://www.7-zip.org/ or run: choco install 7zip"
    exit 1
}

# Pin when ReShade releases - update ATTRIBUTION.txt / binary-hashes.json if bumped.
$DefaultSetupUrl = "https://reshade.me/downloads/ReShade_Setup_6.7.3_Addon.exe"

function Test-ValidReShadeDll([string]$Path) {
    return (Test-Path $Path) -and (Get-Item $Path).Length -ge 65536
}

function Find-ExtractedFile([string]$Root, [string]$Name) {
    $direct = Join-Path $Root $Name
    if (Test-Path $direct) { return $direct }
    Get-ChildItem -Path $Root -Filter $Name -Recurse -File -ErrorAction SilentlyContinue |
        Select-Object -First 1 -ExpandProperty FullName
}

function Test-SetupArchive([string]$SetupPath) {
    $probe = Join-Path $env:TEMP "reshade-setup-probe"
    Remove-Item $probe -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path $probe -Force | Out-Null
    & $7z x $SetupPath "-o$probe" -y | Out-Null
    if ($LASTEXITCODE -ne 0) { return $false }
    $dll = Find-ExtractedFile $probe "ReShade64.dll"
    $json = Find-ExtractedFile $probe "ReShade64.json"
    $ok = (Test-ValidReShadeDll $dll) -and (Test-Path $json) -and ((Get-Item $json).Length -ge 50)
    Remove-Item $probe -Recurse -Force -ErrorAction SilentlyContinue
    return $ok
}

$Setup = $args[0]
if ($Setup -and -not (Test-Path $Setup)) {
    Write-Error "Setup path not found: $Setup"
    exit 1
}

if (-not $Setup) {
    $downloadUrl = if ($env:GSM_RESHADE_SETUP_URL) { $env:GSM_RESHADE_SETUP_URL } else { $DefaultSetupUrl }
    $downloadDest = Join-Path $env:TEMP "ReShade_Setup_Addon.exe"
    Write-Host "Downloading ReShade addon from $downloadUrl ..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $downloadDest -UseBasicParsing
    $Setup = $downloadDest
}

if (-not (Test-Path $Setup)) {
    Write-Error "ReShade Setup.exe not found. Pass addon Setup path as argument or check network."
    exit 1
}

if (-not (Test-SetupArchive $Setup)) {
    Write-Error @"
$Setup is not a valid ReShade addon Setup (missing ReShade64.dll / ReShade64.json).

Use the addon build from https://reshade.me (ReShade_Setup_*_Addon.exe), not the unsigned build.
"@
    exit 1
}

$ManifestPath = Join-Path $RepoRoot "src-tauri\presets\reshade\binary-hashes.json"
if (Test-Path $ManifestPath) {
    $manifest = Get-Content $ManifestPath -Raw | ConvertFrom-Json
    $expectedSetupSha = $manifest.setup.sha256
    if ($expectedSetupSha -and $expectedSetupSha.Trim().Length -gt 0) {
        $actualSetupSha = (Get-FileHash -Path $Setup -Algorithm SHA256).Hash.ToLowerInvariant()
        $expectedNorm = $expectedSetupSha.ToLowerInvariant()
        if ($actualSetupSha -ne $expectedNorm) {
            Write-Error "ReShade Setup SHA256 mismatch. Expected $expectedNorm, got $actualSetupSha. Bump binary-hashes.json or use a matching Setup.exe."
            exit 1
        }
    }
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$Extract = Join-Path $env:TEMP "reshade-setup-extract"
Remove-Item $Extract -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $Extract -Force | Out-Null
& $7z x $Setup "-o$Extract" -y | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error "7-Zip extraction failed (exit $LASTEXITCODE)."
    exit 1
}

$ReShade64 = Find-ExtractedFile $Extract "ReShade64.dll"
$ReShadeJson = Find-ExtractedFile $Extract "ReShade64.json"

if (-not (Test-ValidReShadeDll $ReShade64)) {
    $fallback = $env:GSM_RESHADE_DLL_FALLBACK
    if ($fallback -and (Test-ValidReShadeDll $fallback)) {
        Write-Host "Using GSM_RESHADE_DLL_FALLBACK: $fallback" -ForegroundColor Yellow
        $ReShade64 = $fallback
    } else {
        Write-Error @"
Could not extract ReShade64.dll from Setup.exe.

Options:
  1. Install 7-Zip and re-run this script
  2. Pass path to ReShade_Setup_*_Addon.exe as argument
  3. Set `$env:GSM_RESHADE_DLL_FALLBACK` to a valid ReShade64.dll (>= 64 KB)
"@
        exit 1
    }
}

if (-not $ReShadeJson -or -not (Test-Path $ReShadeJson) -or (Get-Item $ReShadeJson).Length -lt 50) {
    Write-Error "ReShade64.json not found in Setup.exe extraction (required for Vulkan layer manifest)."
    exit 1
}

# Remove legacy mistaken copy from older GSM fetch scripts.
Remove-Item (Join-Path $BinDir "VkLayer_reshade.dll") -Force -ErrorAction SilentlyContinue

Copy-Item $ReShade64 (Join-Path $BinDir "ReShade64.dll") -Force
foreach ($name in @("dxgi.dll", "d3d11.dll", "d3d9.dll", "opengl32.dll")) {
    Copy-Item $ReShade64 (Join-Path $BinDir $name) -Force
}
Copy-Item $ReShadeJson (Join-Path $BinDir "ReShade64.json") -Force

Write-Host "ReShade binaries installed to $BinDir" -ForegroundColor Green
Get-ChildItem "$BinDir\*.dll", "$BinDir\ReShade64.json" -ErrorAction SilentlyContinue |
    ForEach-Object {
        $ok = if ($_.Extension -eq ".dll") { $_.Length -ge 65536 } else { $_.Length -ge 50 }
        $tag = if ($ok) { "OK" } else { "BAD" }
        Write-Host ("  {0,-22} {1,10} bytes  {2}" -f $_.Name, $_.Length, $tag)
    }

if (Test-Path $ManifestPath) {
    $manifest = Get-Content $ManifestPath -Raw | ConvertFrom-Json
    if ($manifest.files) {
        foreach ($prop in $manifest.files.PSObject.Properties) {
            $name = $prop.Name
            $expected = [string]$prop.Value
            if (-not $expected -or $expected.Trim().Length -eq 0) { continue }
            $path = Join-Path $BinDir $name
            if (-not (Test-Path $path)) {
                Write-Error "ReShade bundle hash check: missing $name"
                exit 1
            }
            $actual = (Get-FileHash -Path $path -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($actual -ne $expected.ToLowerInvariant()) {
                Write-Error "ReShade bundle hash mismatch for ${name}: expected $expected, got $actual"
                exit 1
            }
        }
    }
}

exit 0
