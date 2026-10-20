param([Parameter(Mandatory)][string]$Installer)
$ErrorActionPreference = 'Stop'
if (-not $env:RUNNER_TEMP) { throw 'Installer smoke tests require an isolated CI runner' }
$destination = Join-Path $env:RUNNER_TEMP ('gsm-install-' + [guid]::NewGuid())
$setup = Start-Process -FilePath (Resolve-Path -LiteralPath $Installer) -ArgumentList @('/S', "/D=$destination") -PassThru -Wait -WindowStyle Hidden
if ($setup.ExitCode -ne 0) { throw "Installer failed: $($setup.ExitCode)" }
$exe = Join-Path $destination 'ue-settings-master.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw "Installed application not found: $exe" }
foreach ($resource in @('games/known.json','catalog/display.json','catalog/ue4.json','catalog/ue_reference_index.json')) {
  if (-not (Test-Path -LiteralPath (Join-Path $destination $resource))) { throw "Installed resource not found: $resource" }
}
$app = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
try {
  Start-Sleep -Seconds 8
  $app.Refresh()
  if ($app.HasExited) { throw 'Installed application exited during startup' }
} finally {
  if (-not $app.HasExited) { Stop-Process -Id $app.Id -Force }
}
