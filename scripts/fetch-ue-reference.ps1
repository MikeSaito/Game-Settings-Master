# Copy UE reference snapshots from a local Epic engine clone (git checkout per release tag).
param(
    [string]$EngineRoot = "",
    [string[]]$Versions = @(),
    [switch]$AutoTags
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$destRoot = Join-Path $root "tools\ue-reference"
$setupDoc = "docs/epic-clone-setup.md"

function Write-SetupHint {
    param([string]$Message)
    Write-Error @"
$Message

See setup guide: $setupDoc
  Epic account + GitHub link, clone, then re-run:
  .\scripts\fetch-ue-reference.ps1 -AutoTags
"@
}

function Resolve-EngineRootAuto {
    $candidates = @(
        $EngineRoot,
        $env:UE_ENGINE_ROOT,
        "D:\UnrealEngine",
        "C:\UnrealEngine",
        (Join-Path $env:USERPROFILE "UnrealEngine")
    ) | Where-Object { $_ -and $_.Trim() -ne "" } | Select-Object -Unique

    foreach ($candidate in $candidates) {
        $resolved = Resolve-Path -LiteralPath $candidate -ErrorAction SilentlyContinue
        if (-not $resolved) { continue }
        $path = $resolved.Path
        $cfg = Resolve-EngineConfigDir $path
        $gitDir = Resolve-GitRoot $path
        if ($cfg -or (Test-Path (Join-Path $gitDir ".git"))) {
            return $path
        }
    }
    return $null
}

function Resolve-EngineConfigDir([string]$engineRoot) {
    $candidates = @(
        (Join-Path $engineRoot "Engine\Config"),
        (Join-Path $engineRoot "UnrealEngine\Engine\Config")
    )
    return $candidates | Where-Object { Test-Path (Join-Path $_ "BaseEngine.ini") } | Select-Object -First 1
}

function Resolve-GitRoot([string]$engineRoot) {
    if (Test-Path (Join-Path $engineRoot ".git")) { return $engineRoot }
    $parent = Split-Path $engineRoot -Parent
    if ($parent -and (Test-Path (Join-Path $parent ".git"))) { return $parent }
    return $engineRoot
}

function Resolve-ReleaseTag([string]$version, [string[]]$tags) {
    $escaped = [regex]::Escape($version)
    $releaseTags = @(
        $tags | Where-Object { $_ -match "^$escaped(\.\d+)*-release$" }
    )
    if ($releaseTags.Count -eq 0) {
        $candidates = @(
            "$version-release",
            "$version-Release",
            "$version.0-release",
            "$version.0.0-release"
        )
        foreach ($c in $candidates) {
            if ($tags -contains $c) { return $c }
        }
        return $null
    }
    return ($releaseTags | Sort-Object {
        try { [version]($_ -replace '-release$', '') } catch { [version]'0.0.0' }
    } | Select-Object -Last 1)
}

function Copy-BaseIni([string]$configDir, [string]$outDir) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    Copy-Item -LiteralPath (Join-Path $configDir "BaseEngine.ini") -Destination (Join-Path $outDir "BaseEngine.ini") -Force
    Copy-Item -LiteralPath (Join-Path $configDir "BaseScalability.ini") -Destination (Join-Path $outDir "BaseScalability.ini") -Force
}

function Copy-SourceFiles([string]$engineRoot, [string]$outDir) {
    $sourceOut = Join-Path $outDir "source"
    New-Item -ItemType Directory -Force -Path $sourceOut | Out-Null

    $engineSource = Join-Path $engineRoot "Engine\Source\Runtime\Engine"
    $files = @(
        @{ Source = Join-Path $engineSource "Private\Scalability.cpp"; Dest = "Scalability.cpp" },
        @{ Source = Join-Path $engineSource "Private\GameUserSettings.cpp"; Dest = "GameUserSettings.cpp" },
        @{ Source = Join-Path $engineSource "Classes\GameFramework\GameUserSettings.h"; Dest = "GameUserSettings.h" },
        @{ Source = Join-Path $engineSource "Public\GameFramework\GameUserSettings.h"; Dest = "GameUserSettings.h" }
    )

    foreach ($file in $files) {
        if (Test-Path -LiteralPath $file.Source) {
            Copy-Item -LiteralPath $file.Source -Destination (Join-Path $sourceOut $file.Dest) -Force
        }
    }
}

if (-not $EngineRoot -or $EngineRoot.Trim() -eq "") {
    $EngineRoot = Resolve-EngineRootAuto
    if ($EngineRoot) {
        Write-Host "Auto-detected Epic clone: $EngineRoot"
    } else {
        Write-SetupHint "Epic Unreal Engine clone not found. Checked: D:\UnrealEngine, C:\UnrealEngine, UE_ENGINE_ROOT, $env:USERPROFILE\UnrealEngine"
    }
} elseif (-not (Test-Path -LiteralPath $EngineRoot)) {
    Write-SetupHint "EngineRoot path does not exist: $EngineRoot"
}

$configDir = Resolve-EngineConfigDir $EngineRoot
$gitDir = Resolve-GitRoot $EngineRoot
$useGit = Test-Path (Join-Path $gitDir ".git")

if (-not $configDir -and -not $useGit) {
    Write-SetupHint "Engine Config folder not found and no git repo under: $EngineRoot"
}

if ($AutoTags -or $Versions.Count -eq 0) {
    if (-not $useGit) {
        Write-SetupHint "AutoTags requires a git clone at EngineRoot (see $setupDoc)."
    }
    $versionsFile = Join-Path $root "tools\ue-catalog-builder\ue_versions.json"
    $versionsJson = Get-Content -LiteralPath $versionsFile -Raw -Encoding UTF8 | ConvertFrom-Json
    $Versions = @($versionsJson.versions)
}

$tags = @()
if ($useGit) {
    Push-Location $gitDir
    try {
        git fetch --tags --quiet 2>$null
        $tags = @(git tag -l)
        Write-Host "Git tags available: $($tags.Count)"
    } finally {
        Pop-Location
    }
}

$resolvedTags = @{}
foreach ($ver in $Versions) {
    if ($useGit) {
        $tag = Resolve-ReleaseTag $ver $tags
        if ($tag) { $resolvedTags[$ver] = $tag }
    }
}
Write-Host "Resolved release tags: $($resolvedTags.Count) / $($Versions.Count)"
foreach ($ver in $Versions) {
    $tag = $resolvedTags[$ver]
    if ($tag) { Write-Host "  UE $ver -> $tag" }
    else { Write-Host "  UE $ver -> (no tag)" }
}

$worktreeRoot = $null
if ($useGit) {
    $worktreeRoot = Join-Path $env:TEMP "gsm-ue-reference-worktrees-$([guid]::NewGuid().ToString())"
    New-Item -ItemType Directory -Force -Path $worktreeRoot | Out-Null
}

$copied = 0
foreach ($ver in $Versions) {
    $out = Join-Path $destRoot "UE_$ver"
    if ($useGit) {
        $tag = $resolvedTags[$ver]
        if (-not $tag) {
            Write-Warning "No git tag for UE $ver - skipping"
            continue
        }
        $worktree = Join-Path $worktreeRoot "UE_$ver"
        try {
            Write-Host "Creating temporary worktree for $tag ..."
            $prevEap = $ErrorActionPreference
            $ErrorActionPreference = "Continue"
            git -C $gitDir worktree add --detach --quiet $worktree $tag 2>&1 | Out-Null
            $ErrorActionPreference = $prevEap
            if ($LASTEXITCODE -ne 0) {
                Write-SetupHint "git worktree add $tag failed. Try: git fetch --tags in $gitDir"
            }
            $cfg = Resolve-EngineConfigDir $worktree
            if (-not $cfg) {
                Write-SetupHint "Config missing after checkout $tag"
            }
            Copy-BaseIni $cfg $out
            Copy-SourceFiles $worktree $out
            Write-Host "Copied UE_$ver from tag $tag"
            $copied++
        } finally {
            if (Test-Path -LiteralPath $worktree) {
                git -C $gitDir worktree remove --force $worktree 2>$null | Out-Null
            }
        }
    } else {
        Copy-BaseIni $configDir $out
        Copy-SourceFiles $EngineRoot $out
        Write-Host "Copied UE_$ver (single tree, no git)"
        $copied++
    }
}

if ($worktreeRoot -and (Test-Path -LiteralPath $worktreeRoot)) {
    Remove-Item -LiteralPath $worktreeRoot -Recurse -Force -ErrorAction SilentlyContinue
}

if ($copied -eq 0) {
    Write-SetupHint "No UE snapshots copied. Ensure clone has release tags (git fetch --tags)."
}

Write-Host "Done: $copied UE snapshots copied. Run: npm run catalog:build"
