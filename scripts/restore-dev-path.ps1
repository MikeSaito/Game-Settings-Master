# Restore User PATH for local dev tools after Windows reset.
# Run: powershell -ExecutionPolicy Bypass -File scripts/restore-dev-path.ps1
# Then restart terminal / Cursor (Developer: Reload Window).

$ErrorActionPreference = "Stop"

$candidates = @(
    "$env:USERPROFILE\.cargo\bin",
    "C:\Program Files\nodejs",
    "C:\Program Files\Git\cmd",
    "C:\Program Files\Git\bin",
    "C:\Program Files\Git\mingw64\bin",
    "C:\Program Files\GitHub CLI",
    "$env:LOCALAPPDATA\Programs\Python\Python310",
    "$env:LOCALAPPDATA\Programs\Python\Python310\Scripts",
    "$env:LOCALAPPDATA\Programs\cursor\resources\app\bin",
    "C:\ffmpeg-master-latest-win64-gpl-shared\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8\bin",
    "C:\TensorRT\bin",
    "C:\Program Files\NVIDIA Corporation\NVIDIA App\NvDLISR"
)

$detectedPaths = foreach ($p in $candidates) {
    if (Test-Path $p) { $p.TrimEnd('\') }
}

$existingUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$existingPaths = @()
if ($existingUserPath) {
    $existingPaths = $existingUserPath -split ';' | Where-Object { $_ -and $_.Trim() -ne "" } | ForEach-Object { $_.TrimEnd('\') }
}

$paths = @($existingPaths + $detectedPaths) | Select-Object -Unique
$userPath = $paths -join ';'
[Environment]::SetEnvironmentVariable("Path", $userPath, "User")

Write-Host "User PATH updated ($($detectedPaths.Count) detected dev entries, $($paths.Count) total entries):"
$detectedPaths | ForEach-Object { Write-Host "  $_" }

$profileSnippet = @'
### Game Settings Master dev PATH refresh: begin
# Refresh PATH from registry (Cursor/IDE terminals often keep stale env from app launch).
$user = [Environment]::GetEnvironmentVariable('Path', 'User')
$machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
if ($user -and $machine) {
    $env:Path = "$user;$machine"
} elseif ($user) {
    $env:Path = $user
}
### Game Settings Master dev PATH refresh: end
'@

foreach ($profilePath in @(
        "$env:USERPROFILE\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1",
        "$env:USERPROFILE\Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
    )) {
    $dir = Split-Path $profilePath -Parent
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    try {
        $existingProfile = if (Test-Path $profilePath) { Get-Content -LiteralPath $profilePath -Raw } else { "" }
        $begin = "### Game Settings Master dev PATH refresh: begin"
        $end = "### Game Settings Master dev PATH refresh: end"
        $pattern = "(?s)\r?\n?$([regex]::Escape($begin)).*?$([regex]::Escape($end))\r?\n?"
        if ($existingProfile -match [regex]::Escape($begin)) {
            $updatedProfile = [regex]::Replace($existingProfile, $pattern, "`r`n$profileSnippet`r`n").TrimEnd() + "`r`n"
        } else {
            $updatedProfile = ($existingProfile.TrimEnd() + "`r`n`r`n" + $profileSnippet + "`r`n").TrimStart()
        }
        Set-Content -LiteralPath $profilePath -Value $updatedProfile -Encoding UTF8 -Force
        Write-Host "Profile updated: $profilePath"
    } catch {
        Write-Warning "Could not write profile $profilePath : $($_.Exception.Message)"
    }
}

# Apply to current session immediately
$env:Path = "$userPath;" + [Environment]::GetEnvironmentVariable('Path', 'Machine')

Write-Host ""
Write-Host "Open a NEW terminal tab, then verify: node -v; cargo -V; git --version; gh --version"
