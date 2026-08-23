Add-Type -TypeDefinition @"
using System.Runtime.InteropServices;
public struct XIVib { public ushort L; public ushort R; }
public class XI {
  [DllImport("xinput1_4.dll")] public static extern uint XInputSetState(uint idx, ref XIVib v);
}
"@
$v = New-Object XIVib
$v.L = 65535
$v.R = 65535
$codes = 0..3 | ForEach-Object { [XI]::XInputSetState($_, [ref]$v) }
Write-Output ("set-state results per slot (0 = ok): " + ($codes -join ", "))
Start-Sleep -Seconds 2
$v.L = 0
$v.R = 0
0..3 | ForEach-Object { [XI]::XInputSetState($_, [ref]$v) } | Out-Null
Write-Output "rumble stopped"
