$ErrorActionPreference = 'Stop'
if (-not $env:CI) { throw 'Installer desktop preparation requires an isolated CI runner' }
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class GsmInstallerShell {
    [DllImport("user32.dll")]
    public static extern IntPtr GetShellWindow();
}
'@
# NSIS RunAsUser uses GetShellWindow to obtain the desktop user's token.
# Hosted runners can start without Explorer; installing succeeds but relaunch fails.
if ([GsmInstallerShell]::GetShellWindow() -eq [IntPtr]::Zero) {
    Start-Process -FilePath (Join-Path $env:WINDIR 'explorer.exe') -WindowStyle Hidden
}
$deadline = (Get-Date).AddSeconds(30)
while ([GsmInstallerShell]::GetShellWindow() -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 250
}
if ([GsmInstallerShell]::GetShellWindow() -eq [IntPtr]::Zero) {
    throw 'Windows desktop shell is unavailable; NSIS cannot relaunch as the desktop user'
}
Write-Host 'Windows desktop shell is ready for NSIS RunAsUser'
