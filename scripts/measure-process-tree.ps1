param(
  [Parameter(Mandatory)][string]$Executable,
  [Parameter(Mandatory)][string]$Output,
  [int]$Seconds = 60,
  [switch]$Minimize
)
$ErrorActionPreference = 'Stop'
$targetPath = (Resolve-Path -LiteralPath $Executable).Path
$roots = @(Get-CimInstance Win32_Process -Filter "Name = 'ue-settings-master.exe'" | Where-Object { $_.ExecutablePath -eq $targetPath })
if ($roots.Count -ne 1) { throw "Expected one isolated application, found $($roots.Count)" }
$rootProcessId = [int]$roots[0].ProcessId
$rootCreated = $roots[0].CreationDate
$logicalProcessors = [Environment]::ProcessorCount
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class GsmPerformanceWindow {
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
}
'@
$window = (Get-Process -Id $rootProcessId).MainWindowHandle
if ($Minimize) {
  [void][GsmPerformanceWindow]::ShowWindow($window, 6)
  if (-not [GsmPerformanceWindow]::IsIconic($window)) { throw 'Application did not minimize' }
}
$previousCpu = @{}
$samples = [Collections.Generic.List[object]]::new()
$watch = [Diagnostics.Stopwatch]::StartNew()
$previousTime = 0.0
try {
  while ($watch.Elapsed.TotalSeconds -le $Seconds) {
    $snapshot = @(Get-CimInstance Win32_Process)
    if (-not ($snapshot | Where-Object { $_.ProcessId -eq $rootProcessId -and $_.CreationDate -eq $rootCreated })) { throw 'Measured application exited' }
    $ids = [Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($rootProcessId)
    do {
      $added = $false
      foreach ($entry in $snapshot) {
        if ($entry.CreationDate -ge $rootCreated -and $ids.Contains([int]$entry.ParentProcessId)) {
          if ($ids.Add([int]$entry.ProcessId)) { $added = $true }
        }
      }
    } while ($added)
    $cpuMs = 0.0; $privateBytes = 0L; $workingSet = 0L; $handles = 0; $threads = 0
    foreach ($entry in $snapshot | Where-Object { $ids.Contains([int]$_.ProcessId) }) {
      try { $process = Get-Process -Id $entry.ProcessId -ErrorAction Stop } catch { continue }
      try {
        $key = "$($entry.ProcessId):$($entry.CreationDate.Ticks)"
        $currentCpu = $process.TotalProcessorTime.TotalMilliseconds
        if ($previousCpu.ContainsKey($key)) { $cpuMs += [Math]::Max(0, $currentCpu - $previousCpu[$key]) }
        $previousCpu[$key] = $currentCpu
        $privateBytes += $process.PrivateMemorySize64
        $workingSet += $process.WorkingSet64
        $handles += $process.HandleCount
        $threads += $process.Threads.Count
      } finally { $process.Dispose() }
    }
    $elapsed = $watch.Elapsed.TotalSeconds
    $interval = $elapsed - $previousTime
    $samples.Add([pscustomobject]@{ seconds = $elapsed; cpu_ms = $cpuMs; cpu_percent = $(if ($previousTime -gt 0) { $cpuMs / ($interval * 1000) * 100 / $logicalProcessors } else { 0 }); private_mib = $privateBytes / 1MB; working_set_sum_mib = $workingSet / 1MB; handles = $handles; threads = $threads; processes = $ids.Count })
    $previousTime = $elapsed
    Start-Sleep -Seconds 4
  }
  [pscustomobject]@{ logical_processors = $logicalProcessors; duration_seconds = $watch.Elapsed.TotalSeconds; samples = $samples.ToArray() } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $Output -Encoding utf8
} finally {
  if ($Minimize) { [void][GsmPerformanceWindow]::ShowWindow($window, 9) }
}
