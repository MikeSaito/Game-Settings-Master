# Hint: ReShade DLL status in GSM dev bundle.
$RepoRoot = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $RepoRoot "src-tauri\presets\reshade\bin"

Write-Host ""
Write-Host "=== ReShade DLL (required for install) ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Run: .\scripts\fetch-reshade-binaries.ps1"
Write-Host "   Or copy from https://reshade.me to:"
Write-Host "   $BinDir"
Write-Host ""
Write-Host "   dxgi.dll           - DX12 / DXGI (UE5, Forza, most AAA)"
Write-Host "   d3d11.dll          - DX10/11"
Write-Host "   d3d9.dll           - DX9"
Write-Host "   opengl32.dll       - OpenGL"
Write-Host "   ReShade64.dll + ReShade64.json - Vulkan (6.7+ implicit layer)"
Write-Host ""
Write-Host "GSM rejects stubs (< 64 KB). See presets/reshade/ATTRIBUTION.txt"
Write-Host ""

if (Test-Path $BinDir) {
    Get-ChildItem -Path $BinDir -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
        $ok = $_.Length -ge 65536
        $tag = if ($ok) { "OK" } else { "STUB" }
        Write-Host ("  {0,-22} {1,10} bytes  {2}" -f $_.Name, $_.Length, $tag)
    }
}
