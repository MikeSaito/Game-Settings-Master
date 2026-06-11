$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "reshade-common.ps1")

function Invoke-SetupScript {
    param(
        [Parameter(Mandatory)]
        [string]$ScriptPath
    )

    if (-not (Test-Path $ScriptPath)) {
        Write-Error "Setup script not found: $ScriptPath"
        exit 1
    }

    # Child .ps1 via call operator does not reliably set $LASTEXITCODE in Windows PowerShell.
    # Run in a subprocess so exit codes from fetch-reshade-*.ps1 propagate correctly.
    $psExe = Get-ReShadePowerShellExe
    & $psExe -NoProfile -ExecutionPolicy Bypass -File $ScriptPath
    if ($LASTEXITCODE -ne 0) {
        return $false
    }
    return $true
}

$scripts = @(
    "fetch-reshade-shaders.ps1",
    "fetch-reshade-binaries.ps1",
    "verify-reshade-bundle.ps1"
)

foreach ($name in $scripts) {
    $path = Join-Path $PSScriptRoot $name
    if (-not (Invoke-SetupScript $path)) {
        if ($name -eq "fetch-reshade-binaries.ps1" -or $name -eq "verify-reshade-bundle.ps1") {
            $hint = Join-Path $PSScriptRoot "reshade-dll-hint.ps1"
            if (Test-Path $hint) {
                & (Get-ReShadePowerShellExe) -NoProfile -ExecutionPolicy Bypass -File $hint
            }
        }
        exit 1
    }
}

Write-Host "ReShade bundle ready for tauri build." -ForegroundColor Green
exit 0
