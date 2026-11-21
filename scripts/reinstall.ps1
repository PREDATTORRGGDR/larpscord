# Reinstall Larpscord silently and verify. ASCII only.
$setup = "E:\Larpscord\src-tauri\target\release\bundle\nsis\Larpscord_1.1.0_x64-setup.exe"
$p = Start-Process -FilePath $setup -ArgumentList "/S" -PassThru -Wait
Write-Host "INSTALLER_EXIT: $($p.ExitCode)"
$exe = "$env:LOCALAPPDATA\Larpscord\larpscord.exe"
if (Test-Path $exe) {
  Write-Host "INSTALLED: $exe $((Get-Item $exe).LastWriteTime)"
} else {
  Write-Host "NOT_INSTALLED"
  exit 1
}
$desktop = [Environment]::GetFolderPath("Desktop")
$sh = New-Object -ComObject WScript.Shell
$sc = $sh.CreateShortcut("$desktop\Larpscord.lnk")
Write-Host "SHORTCUT_ICON: $($sc.IconLocation)"
Write-Host "SHORTCUT_TARGET: $($sc.TargetPath)"
