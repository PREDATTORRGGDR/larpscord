# Final desktop setup: verify shortcut + copy installer to Desktop. ASCII only.
$desktop = [Environment]::GetFolderPath("Desktop")
Write-Host "DESKTOP_DIR: $desktop"

# 1. Shortcut: recreate with explicit icon (guaranteed new L).
$exe = "$env:LOCALAPPDATA\Larpscord\larpscord.exe"
if (-not (Test-Path $exe)) { Write-Host "ERROR: exe not installed"; exit 1 }

$ws = New-Object -ComObject WScript.Shell
$lnkPath = "$desktop\Larpscord.lnk"
$sc = $ws.CreateShortcut($lnkPath)
$sc.TargetPath = $exe
$sc.WorkingDirectory = "$env:LOCALAPPDATA\Larpscord"
$sc.IconLocation = "$exe,0"
$sc.Description = "Larpscord - Discord Rich Presence Studio"
$sc.Save()
Write-Host "SHORTCUT_CREATED: $lnkPath"
Write-Host "SHORTCUT_ICON: $($sc.IconLocation)"

# 2. Installer to Desktop (shareable).
$setup = "E:\Larpscord\src-tauri\target\release\bundle\nsis\Larpscord_1.1.0_x64-setup.exe"
$dst = "$desktop\Larpscord_1.1.0_x64-setup.exe"
Copy-Item $setup $dst -Force
$mb = [math]::Round((Get-Item $dst).Length / 1MB, 2)
Write-Host "INSTALLER_COPIED: $dst ($mb MB)"

# 3. Final listing.
Get-ChildItem $desktop -Filter "*Larpscord*" | ForEach-Object { Write-Host "ON_DESKTOP: $($_.Name)" }
