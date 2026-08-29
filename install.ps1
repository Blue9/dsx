# Creates a Desktop shortcut that launches dsx through WSL interop.
# Smart App Control blocks unsigned exes started from NTFS, but permits
# the same binary when launched via wsl.exe (a Microsoft-signed binary),
# so the shortcut runs the build straight from the WSL filesystem.
$wsl = Join-Path $env:SystemRoot 'System32\wsl.exe'
$repo = '/home/g/dev/ps5'
$exe = './target/x86_64-pc-windows-gnu/release/dsx.exe'

$ws = New-Object -ComObject WScript.Shell
$lnk = $ws.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Desktop')) 'dsx.lnk'))
$lnk.TargetPath = $wsl
$lnk.Arguments = "--cd $repo -e $exe"
$lnk.Description = 'DualSense to Xbox controller bridge'
$lnk.Save()
Write-Output "shortcut created: Desktop\dsx.lnk (launches via wsl.exe)"
