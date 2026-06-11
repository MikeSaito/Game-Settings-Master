# Клонирует официальный пакет шейдеров ReShade в dev-бандл GSM.
# Also copies Textures/*.png from ShadersAndTextures when present (nvidia branch).

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$Dest = Join-Path $RepoRoot "src-tauri\presets\reshade\shaders"
$ShadersDir = Join-Path $Dest "Shaders"
$Temp = Join-Path $env:TEMP "reshade-shaders-fetch"

# Pin when updating — bump with fetch test if reshade-shaders layout changes.
$ShaderRepo = "https://github.com/crosire/reshade-shaders.git"
$ShaderCommit = if ($env:GSM_RESHADE_SHADERS_COMMIT) {
    $env:GSM_RESHADE_SHADERS_COMMIT
} else {
    "6b452c4a101ccb228c4986560a51c571473c517b"
}

if (Test-Path $Temp) {
    Remove-Item -Recurse -Force $Temp
}
New-Item -ItemType Directory -Force -Path $Temp | Out-Null

Write-Host "Fetching crosire/reshade-shaders @ $ShaderCommit ..."
Push-Location $Temp
git init | Out-Null
git remote add origin $ShaderRepo
git fetch --depth 1 origin $ShaderCommit
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Error "git fetch origin $ShaderCommit failed - bump pin or set GSM_RESHADE_SHADERS_COMMIT."
    exit 1
}
git checkout --quiet FETCH_HEAD
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Error "git checkout FETCH_HEAD failed."
    exit 1
}
Pop-Location

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
if (Test-Path $ShadersDir) {
    Remove-Item -Recurse -Force $ShadersDir
}

$EffectsSrc = Join-Path $Temp "ShadersAndTextures"
if (-not (Test-Path $EffectsSrc)) {
    Write-Error "ShadersAndTextures not found in clone - check reshade-shaders branch layout."
    exit 1
}
New-Item -ItemType Directory -Force -Path $ShadersDir | Out-Null
Copy-Item -Path (Join-Path $EffectsSrc "*") -Destination $ShadersDir -Recurse -Force

$texturesSrc = Join-Path $Temp "Textures"
if (Test-Path $texturesSrc) {
    $texturesDest = Join-Path $Dest "Textures"
    if (Test-Path $texturesDest) {
        Remove-Item -Recurse -Force $texturesDest
    }
    Copy-Item -Recurse $texturesSrc $texturesDest
}

Remove-Item -Recurse -Force $Temp

$fxCount = (Get-ChildItem -Path $ShadersDir -Filter "*.fx" -Recurse -ErrorAction SilentlyContinue).Count
Write-Host "Done. Shaders: $fxCount .fx files in $ShadersDir (commit $ShaderCommit)"
if ($fxCount -lt 1) {
    Write-Error "No .fx shaders found - check clone output."
    exit 1
}

exit 0
