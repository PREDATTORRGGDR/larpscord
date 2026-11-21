# Refresh icon cache + re-pin shortcut with the new icon. ASCII only.

$desktop = "C:\Users\CC\Desktop"
$exe = "C:\Users\CC\AppData\Local\Larpscord\larpscord.exe"

# 1. Recreate shortcut pointing to installed exe with explicit icon.
$ws = New-Object -ComObject WScript.Shell
$sc = $ws.CreateShortcut("$desktop\Larpscord.lnk")
$sc.TargetPath = $exe
$sc.WorkingDirectory = "C:\Users\CC\AppData\Local\Larpscord"
$sc.IconLocation = "$exe,0"
$sc.Save()
Write-Host "SHORTCUT_SAVED: target=$exe icon=$exe,0"

# 2. Extract fresh .ico next to exe as alternate explicit icon source
#    (exe,0 may be cached; direct ico path refreshes reliably).
# The NSIS-installed app has icon embedded in exe; additionally write ico copy:
Copy-Item "E:\Larpscord\src-tauri\icons\icon.ico" "C:\Users\CC\AppData\Local\Larpscord\larpscord.ico" -Force
$sc2 = $ws.CreateShortcut("$desktop\Larpscord.lnk")
$sc2.IconLocation = "C:\Users\CC\AppData\Local\Larpscord\larpscord.ico,0"
$sc2.Save()
Write-Host "SHORTCUT_ICON_NOW: $($sc2.IconLocation)"

# 3. Flush icon cache databases and restart explorer.
$iconCacheDirs = @(
  "$env:LOCALAPPDATA\Microsoft\Windows\Explorer\iconcache_*.db",
  "$env:LOCALAPPDATA\Microsoft\Windows\Explorer\thumbcache_*.db"
)
foreach ($pattern in $iconCacheDirs) {
  Remove-Item $pattern -Force -ErrorAction SilentlyContinue
}
ie4uinit.exe -show 2>$null
Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
  Start-Process explorer.exe
}
Write-Host "ICON_CACHE_FLUSHED explorer_restarted"
