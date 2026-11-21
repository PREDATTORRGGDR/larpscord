# Verify v1.1.0: launch, window, close-to-tray (process stays), shortcut intact. ASCII only.
$desktop = [Environment]::GetFolderPath("Desktop")
$lnk = Join-Path $desktop "Larpscord.lnk"
$sh = New-Object -ComObject WScript.Shell
$sc = $sh.CreateShortcut($lnk)
Write-Host "SHORTCUT_TARGET: $($sc.TargetPath)"

$exe = "$env:LOCALAPPDATA\Larpscord\larpscord.exe"
if (-not (Test-Path $exe)) { Write-Host "ERROR: exe missing"; exit 1 }
Write-Host "EXE_VERSION_DIR: $((Get-Item $exe).LastWriteTime)"

Invoke-Item $lnk
Start-Sleep -Seconds 7
$p = Get-Process -Name Larpscord -ErrorAction SilentlyContinue
if (-not $p) { Write-Host "ERROR: not launched"; exit 1 }
Write-Host "LAUNCHED: PID $($p[0].Id)"

# Close the window (as user would) — process must stay in tray.
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W32x {
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeout(IntPtr h, uint m, IntPtr w, IntPtr l, uint flags, uint timeout, out IntPtr result);
}
"@
# WM_CLOSE = 0x0010
$h = $p[0].MainWindowHandle
Write-Host "HWND: $h"
[W32x]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
Start-Sleep -Seconds 4

$p2 = Get-Process -Name Larpscord -ErrorAction SilentlyContinue
if ($p2) {
  Write-Host "TRAY_ALIVE: process survived window close (PID $($p2[0].Id))"
  $hidden = ($p2[0].MainWindowHandle -eq 0)
  Write-Host "WINDOW_HIDDEN: $hidden"
} else {
  Write-Host "ERROR: process exited on close"
  exit 1
}
