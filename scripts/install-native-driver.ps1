$ErrorActionPreference = 'Stop'
cargo install tauri-driver --locked --version 2.1.0
if ($LASTEXITCODE -ne 0) { throw 'tauri-driver installation failed' }
$roots = @(${env:ProgramFiles(x86)}, $env:ProgramFiles, $env:LOCALAPPDATA) | Where-Object { $_ }
$runtimes = @(foreach ($root in $roots) {
    $application = Join-Path $root 'Microsoft/EdgeWebView/Application'
    if (Test-Path -LiteralPath $application) {
        Get-ChildItem -LiteralPath $application -Directory | Where-Object {
            $_.Name -match '^\d+\.\d+\.\d+\.\d+$' -and
            (Test-Path -LiteralPath (Join-Path $_.FullName 'msedgewebview2.exe'))
        }
    }
})
$runtime = $runtimes | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
if (-not $runtime) { throw 'Cannot locate installed WebView2 Runtime' }
$version = $runtime.Name
Write-Host "Using WebView2 $version at $($runtime.FullName)"
# Pin the runtime as well as its matching driver, even if Edge updates separately.
"WEBVIEW2_BROWSER_EXECUTABLE_FOLDER=$($runtime.FullName)" >> $env:GITHUB_ENV
$archive = Join-Path $env:RUNNER_TEMP 'edgedriver.zip'
$destination = Join-Path $env:RUNNER_TEMP 'edgedriver'
Invoke-WebRequest "https://msedgedriver.microsoft.com/$version/edgedriver_win64.zip" -OutFile $archive
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
& (Join-Path $destination 'msedgedriver.exe') --version
if ($LASTEXITCODE -ne 0) { throw 'EdgeDriver verification failed' }
$destination >> $env:GITHUB_PATH
