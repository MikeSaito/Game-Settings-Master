$ErrorActionPreference = 'Stop'
$fixtureRoot = Join-Path $PWD 'test-results/updater-fixtures'
New-Item -ItemType Directory -Path $fixtureRoot -Force | Out-Null
$keyPath = Join-Path $fixtureRoot 'test.key'
npm exec -- tauri signer generate --ci -w $keyPath -p native-test-password
if ($LASTEXITCODE -ne 0) { throw 'Test key generation failed' }
$publicKey = (Get-Content -LiteralPath "$keyPath.pub" -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -LiteralPath $keyPath -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = 'native-test-password'
try {
  foreach ($version in @('1.4.0', '1.4.1')) {
    $configPath = Join-Path $fixtureRoot "$version.json"
    $config = @{
      version = $version
      identifier = 'com.mike.gsm.native-updater-test'
      productName = 'GSM Native Updater Test'
      plugins = @{ updater = @{ pubkey = $publicKey; endpoints = @('http://127.0.0.1:17891/latest.json'); dangerousInsecureTransportProtocol = $true; windows = @{ installMode = 'quiet' } } }
    }
    $config | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $configPath -Encoding utf8
    npm exec -- tauri build --debug --features native-test --config $configPath
    if ($LASTEXITCODE -ne 0) { throw "Build $version failed" }
    $installers = @(Get-ChildItem "src-tauri/target/debug/bundle/nsis/*_${version}_x64-setup.exe")
    if ($installers.Count -ne 1) { throw "Expected one installer for $version" }
    Copy-Item -LiteralPath $installers[0].FullName -Destination (Join-Path $fixtureRoot "$version.exe")
    Copy-Item -LiteralPath ($installers[0].FullName + '.sig') -Destination (Join-Path $fixtureRoot "$version.exe.sig")
  }
} finally {
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
  # Test private material is never uploaded as an artifact.
  Remove-Item -LiteralPath $keyPath -ErrorAction SilentlyContinue
}
