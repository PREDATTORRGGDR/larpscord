# Tray test: launch, close window, verify process stays, read debug log. ASCII only.
param([string]$ExePath = "E:\Larpscord\src-tauri\target\debug\larpscord.exe")

Remove-Item "$env:APPDATA\Larpscord\tray-debug.log" -ErrorAction SilentlyContinue
Start-Process -FilePath $ExePath
Start-Sleep -Seconds 6

$p = Get-Process -Name larpscord -ErrorAction SilentlyContinue
if (-not $p) { Write-Host "NOT_RUNNING"; exit 1 }
Write-Host "PID: $($p[0].Id) HWND: $($p[0].MainWindowHandle)"

Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class C{[DllImport("user32.dll")]public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);}'
$h = $p[0].MainWindowHandle
if ($h -eq [IntPtr]::Zero) { Write-Host "NO_WINDOW_HANDLE"; exit 1 }
[C]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
Start-Sleep -Seconds 4

$p2 = Get-Process -Name larpscord -ErrorAction SilentlyContinue
if ($p2) {
  Write-Host "TRAY_ALIVE: survived window close"
  Write-Host "WINDOW_HIDDEN: $($p2[0].MainWindowHandle -eq [IntPtr]::Zero)"
} else {
  Write-Host "PROCESS_EXITED"
}
Write-Host "--- tray-debug.log ---"
Get-Content "$env:APPDATA\Larpscord\tray-debug.log" -ErrorAction SilentlyContinue
# Cleanup: kill test instance
Get-Process -Name larpscord -ErrorAction SilentlyContinue | Stop-Process -Force
