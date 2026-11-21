# Тихая установка Larpscord для текущего пользователя.
$setup = "E:\Larpscord\src-tauri\target\release\bundle\nsis\Larpscord_1.0.0_x64-setup.exe"
Write-Host "Запуск установки: $setup"
$p = Start-Process -FilePath $setup -ArgumentList "/S" -PassThru -Wait
Write-Host "Код выхода установщика: $($p.ExitCode)"

# Ищем установленный exe
$candidates = @(
  "$env:LOCALAPPDATA\Programs\Larpscord\Larpscord.exe",
  "$env:LOCALAPPDATA\Larpscord\Larpscord.exe"
)
foreach ($c in $candidates) {
  if (Test-Path $c) { Write-Host "INSTALLED_EXE: $c" }
}

# Ярлыки
$desktop = [Environment]::GetFolderPath("Desktop")
Get-ChildItem $desktop -Filter "*Larpscord*" -ErrorAction SilentlyContinue | ForEach-Object {
  Write-Host "DESKTOP_SHORTCUT: $($_.FullName)"
}
$startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
Get-ChildItem $startMenu -Recurse -Filter "*Larpscord*" -ErrorAction SilentlyContinue | ForEach-Object {
  Write-Host "STARTMENU_ENTRY: $($_.FullName)"
}
