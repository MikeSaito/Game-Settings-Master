# Opens a new terminal window for interactive key generation.

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$script = Join-Path $PSScriptRoot "generate-updater-keys.ps1"

if (-not (Test-Path $script)) {
    Write-Error "Script not found: $script"
}

if (Get-Command wt.exe -ErrorAction SilentlyContinue) {
    $arg = "-NoExit", "-ExecutionPolicy", "Bypass", "-File", $script
    Start-Process wt.exe -ArgumentList "powershell", $arg
} else {
    Start-Process powershell.exe -ArgumentList "-NoExit", "-ExecutionPolicy", "Bypass", "-File", $script -WorkingDirectory $root
}

Write-Host "New terminal opened. Enter your password there." -ForegroundColor Cyan
Write-Host "Or run in this window: .\scripts\generate-updater-keys.ps1" -ForegroundColor Cyan
