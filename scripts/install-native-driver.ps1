$ErrorActionPreference = 'Stop'
cargo install tauri-driver --locked --version 2.1.0
if ($LASTEXITCODE -ne 0) { throw 'tauri-driver installation failed' }
$edge = Join-Path ${env:ProgramFiles(x86)} 'Microsoft/Edge/Application/msedge.exe'
if (-not (Test-Path -LiteralPath $edge)) { $edge = Join-Path $env:ProgramFiles 'Microsoft/Edge/Application/msedge.exe' }
$version = (Get-Item -LiteralPath $edge).VersionInfo.ProductVersion
if ($version -notmatch '^\d+\.\d+\.\d+\.\d+$') { throw 'Cannot determine Edge version' }
$archive = Join-Path $env:RUNNER_TEMP 'edgedriver.zip'
$destination = Join-Path $env:RUNNER_TEMP 'edgedriver'
Invoke-WebRequest "https://msedgedriver.microsoft.com/$version/edgedriver_win64.zip" -OutFile $archive
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
$destination >> $env:GITHUB_PATH
