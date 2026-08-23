# Copies the built dsx.exe to a local Windows folder and puts a shortcut
# on the Desktop. Re-run after every rebuild to update the installed copy.
$src = Join-Path $PSScriptRoot 'target\x86_64-pc-windows-gnu\release\dsx.exe'
$dst = Join-Path $env:LOCALAPPDATA 'dsx'
New-Item -ItemType Directory -Force -Path $dst | Out-Null
Copy-Item $src (Join-Path $dst 'dsx.exe') -Force

$ws = New-Object -ComObject WScript.Shell
$lnk = $ws.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Desktop')) 'dsx.lnk'))
$lnk.TargetPath = Join-Path $dst 'dsx.exe'
$lnk.WorkingDirectory = $dst
$lnk.Description = 'DualSense to Xbox controller bridge'
$lnk.Save()
Write-Output ("installed: " + (Join-Path $dst 'dsx.exe'))
Write-Output "shortcut created: Desktop\dsx.lnk"
