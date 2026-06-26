# Validate merge_stats.json after full catalog fetch.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot

$statsPath = Join-Path $root "src-tauri\catalog\generated\merge_stats.json"



if (-not (Test-Path $statsPath)) {

    Write-Error "merge_stats.json not found. Run: npm run catalog:build"

}



$stats = Get-Content $statsPath -Raw | ConvertFrom-Json

$sources = @($stats.sources)

$total = [int]$stats.total_reference_entries

$versionCount = $sources.Count



Write-Host "Catalog validation"

Write-Host "  sources:          $($sources -join ', ')"

Write-Host "  total entries:    $total"

Write-Host "  scalability_tiers: $($stats.scalability_tiers)"

Write-Host "  sg_registry_count: $($stats.sg_registry_count)"

Write-Host "  gus_registry_count: $($stats.gus_registry_count)"

Write-Host "  deprecated_count:  $($stats.deprecated_count)"
Write-Host "  bare_stub_descriptions: $($stats.bare_stub_descriptions)"

Write-Host ""

Write-Host "introduced_by_version:"

$stats.introduced_by_version.PSObject.Properties | Sort-Object Name | ForEach-Object {

    Write-Host ("  {0,-6} {1}" -f $_.Name, $_.Value)

}

if ($stats.applicable_by_version) {
    Write-Host ""
    Write-Host "applicable_by_version:"
    $stats.applicable_by_version.PSObject.Properties | Sort-Object Name | ForEach-Object {
        Write-Host ("  {0,-6} {1}" -f $_.Name, $_.Value)
    }
}



if ($versionCount -ge 8) {
    if ($total -lt 700) {
        Write-Error "Expected total_reference_entries >= 700 when sources >= 8 (got $total). Re-run fetch + catalog:build."
    }
    if ([int]$stats.bare_stub_descriptions -gt 100) {
        Write-Error "Expected bare_stub_descriptions <= 100 (got $($stats.bare_stub_descriptions)). Improve tier_c in build.py."
    }
    if (-not ($stats.applicable_by_version.PSObject.Properties.Name -contains '5.4')) {
        Write-Error "merge_stats missing applicable_by_version.5.4 - re-run catalog:build."
    }
    if ([int]$stats.sg_registry_count -lt 12) {
        Write-Error "Expected sg_registry_count >= 12 (got $($stats.sg_registry_count)). Run tools/ue-catalog-builder/extract/sg_from_cpp.py --all-versions."
    }
    if ([int]$stats.gus_registry_count -lt 20) {
        Write-Error "Expected gus_registry_count >= 20 (got $($stats.gus_registry_count)). Run tools/ue-catalog-builder/extract/gus_from_header.py --all-versions."
    }
    Write-Host ""
    Write-Host "Full fetch mode ($versionCount sources, $total merged keys)."
    exit 0
}

if ($versionCount -gt 2 -and $total -lt 1000) {

    Write-Error "Expected total_reference_entries >= 1000 when sources > 2 (got $total, sources=$versionCount). Re-run fetch + catalog:build."

}



if ($versionCount -le 2) {

    Write-Host ""

    Write-Host "Bundled fixtures mode ($versionCount sources). Full fetch not applied; threshold 1000 skipped."

    if ($total -lt 548) {

        Write-Error "Bundled catalog below minimum ($total < 548)."

    }

    exit 0

}



$intro = $stats.introduced_by_version

$ue5Milestones = @("5.0", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8")

$present = @($ue5Milestones | Where-Object { $intro.PSObject.Properties.Name -contains $_ })

if ($present.Count -lt 3) {

    Write-Warning "Few UE 5.x introduced_by_version buckets ($($present -join ', ')). Check fetch tags."

}



Write-Host ""

Write-Host "Validation OK."


