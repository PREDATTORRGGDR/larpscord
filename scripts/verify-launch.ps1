# Verify Larpscord launch via desktop shortcut (ASCII-only to avoid encoding issues).
$desktop = [Environment]::GetFolderPath("Desktop")
$lnk = Join-Path $desktop "Larpscord.lnk"
Write-Host "SHORTCUT: $lnk"

$sh = New-Object -ComObject WScript.Shell
$sc = $sh.CreateShortcut($lnk)
Write-Host "TargetPath: $($sc.TargetPath)"
Write-Host "Arguments: $($sc.Arguments)"
Write-Host "IconLocation: $($sc.IconLocation)"

if (-not (Test-Path $sc.TargetPath)) { Write-Host "ERROR: shortcut target missing"; exit 1 }

Invoke-Item $lnk
Start-Sleep -Seconds 7

$procs = Get-Process -Name "Larpscord" -ErrorAction SilentlyContinue
if ($procs) {
  Write-Host "PROCESS_ALIVE: PID $($procs[0].Id), memory $([math]::Round($procs[0].WorkingSet64/1MB,1)) MB"
} else {
  Write-Host "ERROR: Larpscord process not found"
  exit 1
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
  public delegate bool EnumWindowsProc(IntPtr h, IntPtr lp);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
}
"@
$found = $false
[Win]::EnumWindows({ param($h, $lp)
  if ([Win]::IsWindowVisible($h)) {
    $sb = New-Object System.Text.StringBuilder 256
    [Win]::GetWindowText($h, $sb, 256) | Out-Null
    if ($sb.ToString() -like "*Larpscord*") { $script:found = $true }
  }
  return $true
}, [IntPtr]::Zero) | Out-Null
Write-Host "WINDOW_FOUND: $found"
