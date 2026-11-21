# Diagnose: find the REAL visible desktop (OneDrive redirect?), place files there.
# ASCII only. Outputs every candidate path.

Write-Host "=== Registry Desktop redirect ==="
$reg = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders" -ErrorAction SilentlyContinue
Write-Host "UserShellFolders.Desktop: $($reg.Desktop)"
$reg2 = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders" -ErrorAction SilentlyContinue
Write-Host "ShellFolders.Desktop: $($reg2.Desktop)"

Write-Host "=== GetFolderPath ==="
Write-Host "Desktop: $([Environment]::GetFolderPath('Desktop'))"

Write-Host "=== Candidate desktop dirs ==="
$candidates = @(
  "C:\Users\CC\Desktop",
  "C:\Users\CC\OneDrive\Desktop",
  "C:\Users\CC\OneDrive\Рабочий стол",
  "C:\Users\CC\OneDrive\Рабочий стол"
)
foreach ($c in @("C:\Users\CC\Desktop", "C:\Users\CC\OneDrive\Desktop")) {
  if (Test-Path $c) {
    $count = (Get-ChildItem $c -ErrorAction SilentlyContinue | Measure-Object).Count
    Write-Host "EXISTS: $c (items: $count)"
    Get-ChildItem $c -Filter "*Larpscord*" -ErrorAction SilentlyContinue | ForEach-Object { Write-Host "  LARPSCORD_FILE: $($_.FullName)" }
  } else {
    Write-Host "MISSING: $c"
  }
}
# OneDrive russian desktop
$odRu = Get-ChildItem "C:\Users\CC\OneDrive" -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "Desk|Рабоч" }
foreach ($d in $odRu) {
  Write-Host "ONEDRIVE_DESKTOP_DIR: $($d.FullName)"
  Get-ChildItem $d.FullName -Filter "*Larpscord*" -ErrorAction SilentlyContinue | ForEach-Object { Write-Host "  LARPSCORD_FILE: $($_.FullName)" }
}

Write-Host "=== Installed exe version ==="
$exe = "C:\Users\CC\AppData\Local\Larpscord\larpscord.exe"
if (Test-Path $exe) {
  $vi = (Get-Item $exe).VersionInfo
  Write-Host "ExeTime: $((Get-Item $exe).LastWriteTime) ProductVersion: $($vi.ProductVersion) FileVersion: $($vi.FileVersion)"
} else {
  Write-Host "EXE_MISSING"
}

Write-Host "=== Running processes ==="
Get-Process -Name Larpscord,larpscord -ErrorAction SilentlyContinue | ForEach-Object {
  Write-Host "PROC: $($_.Id) $($_.Path) started $($_.StartTime)"
}
