param([ValidateSet('before','after')][string]$Phase = 'before')
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$roleSids = @{'S-1-5-21-587408064-856152790-2583328096-500'='console';'S-1-5-21-587408064-856152790-2583328096-1003'='rdp'}
$sid = $identity.User.Value
if (-not $roleSids.ContainsKey($sid)) { throw 'Use the Administrator console or the marenben RDP desktop.' }
$role = $roleSids[$sid]
$dest = Join-Path $PSScriptRoot ("receipts-$role\$Phase.json")
if ([IO.File]::Exists($dest)) { throw 'A receipt for this phase already exists.' }
$r = [ordered]@{utc=[DateTime]::UtcNow.ToString('o');phase=$Phase;processId=$PID;processImage=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName;sid=$sid;account=$identity.Name;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;role=$role;sessionName=$env:SESSIONNAME;interactive=[Environment]::UserInteractive;effectiveAdmin=([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator);profile=$env:USERPROFILE;errors=@();firstError=$null}
function Hash-Text([string]$Value) {
  $d = [Security.Cryptography.SHA256]::Create()
  try { return [BitConverter]::ToString($d.ComputeHash([Text.Encoding]::UTF8.GetBytes($Value))).Replace('-','').ToLowerInvariant() } finally { $d.Dispose() }
}
function Inspect-Path([string]$Path, [bool]$HashFile) {
  try { $item = Get-Item -LiteralPath $Path -Force -ErrorAction Stop }
  catch [System.Management.Automation.ItemNotFoundException] { return [ordered]@{path=$Path;state='absent'} }
  $state=[ordered]@{path=$item.FullName;state='present';attributes=[string]$item.Attributes;mtimeUtc=$item.LastWriteTimeUtc.ToString('o')}
  if ($HashFile -and -not $item.PSIsContainer) {
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Refusing to hash reparse point $Path" }
    $state.bytes=$item.Length; $state.sha256=(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
  }
  return $state
}
try {
  Add-Type -TypeDefinition @"
using System;
using System.ComponentModel;
using System.Text;
using System.Runtime.InteropServices;
public static class W2DesktopProof {
    [DllImport("kernel32")] static extern uint GetCurrentThreadId();
    [DllImport("user32",SetLastError=true)] static extern IntPtr GetThreadDesktop(uint tid);
    [DllImport("user32",SetLastError=true)] static extern IntPtr GetProcessWindowStation();
    [DllImport("user32",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool GetUserObjectInformation(IntPtr obj,int index,StringBuilder s,int size,out int needed);
    static string Name(IntPtr obj) { var s=new StringBuilder(256); int n; if (obj==IntPtr.Zero || !GetUserObjectInformation(obj,2,s,s.Capacity*2,out n)) throw new Win32Exception(); return s.ToString(); }
    public static string Current() { return Name(GetProcessWindowStation()) + "\\" + Name(GetThreadDesktop(GetCurrentThreadId())); }
}
"@
  $r.desktop=[W2DesktopProof]::Current(); $r.sessions=@(& quser.exe); $r.quserExit=$LASTEXITCODE
  $shells=@(Get-CimInstance Win32_Process -Filter "Name='explorer.exe'" | Where-Object SessionId -eq $r.session | ForEach-Object { $owner=Invoke-CimMethod -InputObject $_ -MethodName GetOwnerSid; [ordered]@{pid=$_.ProcessId;session=$_.SessionId;sid=$owner.Sid;ownerQueryCode=$owner.ReturnValue;image=$_.ExecutablePath} }); $r.shells=$shells
  if ($r.desktop -ine 'WinSta0\Default' -or -not $r.interactive -or @($shells | Where-Object {$_.sid -eq $sid -and $_.ownerQueryCode -eq 0}).Count -lt 1) { throw 'Desktop or Explorer TokenUser binding missing.' }
  if (($role -eq 'rdp' -and ($r.effectiveAdmin -or $r.sessionName -notlike 'RDP-*')) -or ($role -eq 'console' -and $r.sessionName -ine 'Console')) { throw 'Unexpected session transport or effective user role.' }
  foreach ($part in 'userPath','files','registrations','processes','policy') {
    try {
      switch ($part) {
        'userPath' { $key=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment'); try { $exists=($null -ne $key -and $key.GetValueNames() -contains 'Path'); $r.userPath=[ordered]@{present=$exists}; if ($exists) { $v=[string]$key.GetValue('Path',$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames); $r.userPath.kind=$key.GetValueKind('Path').ToString();$r.userPath.rawUtf8Sha256=Hash-Text $v;$r.userPath.chars=$v.Length } } finally { if($key){$key.Dispose()} } }
        'files' { $r.files=@(); foreach($entry in @('.codex/config.toml','.codex-desktop','.codex-desktop/desktop/manager-location.json','.codex-desktop/desktop/terminal-cli.json','.local/bin/ocx.exe','AppData/Local/Programs/OpenCodeX Desktop','ocx-w2-team79-r1')) { $r.files+=Inspect-Path (Join-Path $env:USERPROFILE $entry) $true } }
        'registrations' { $r.registrations=@(); $k='HKCU:/SOFTWARE/Microsoft/Windows/CurrentVersion/Uninstall'; if(Test-Path -LiteralPath $k) { $r.registrations=@(Get-ChildItem -LiteralPath $k | Get-ItemProperty | Where-Object DisplayName -like '*OpenCodeX*' | Select-Object PSChildName,DisplayName,DisplayVersion,InstallLocation,Publisher) }; $r.vendorKey=Test-Path -LiteralPath 'HKCU:/Software/gzers/OpenCodeX Desktop' }
        'processes' { $r.productProcesses=@(Get-CimInstance Win32_Process -Filter ("SessionId="+$r.session) | Where-Object {$_.Name -in 'ocxd.exe','ocx.exe','opencodex-desktop.exe','node.exe','bun.exe'} | ForEach-Object { $which=Invoke-CimMethod -InputObject $_ -MethodName GetOwnerSid; [ordered]@{pid=$_.ProcessId;image=$_.ExecutablePath;name=$_.Name;session=$_.SessionId;sid=$which.Sid;ownerQueryCode=$which.ReturnValue} }) }
        'policy' { $r.policies=@(); foreach ($h in 'LocalMachine','CurrentUser') { $regBase = if ($h -eq 'LocalMachine') {[Microsoft.Win32.Registry]::LocalMachine} else {[Microsoft.Win32.Registry]::CurrentUser}; $key=$regBase.OpenSubKey('SOFTWARE\Policies\Microsoft\Windows\Installer'); try { $p=[ordered]@{hive=$h;keyPresent=($null -ne $key);disableMsiPresent=($null -ne $key -and $key.GetValueNames() -contains 'DisableMSI')}; if ($p.disableMsiPresent) { $p.value=$key.GetValue('DisableMSI'); $p.kind=$key.GetValueKind('DisableMSI').ToString() }; $r.policies+=$p } finally { if($key){$key.Dispose()} } } }
      }
    } catch { $r.errors+=($part+' - '+$_.Exception.Message) }
  }
  if ($r.errors.Count) { $r.firstError=$r.errors[0] }
} catch { $r.firstError=$_.Exception.Message }
$r.finishedUtc=[DateTime]::UtcNow.ToString('o'); $json=$r | ConvertTo-Json -Depth 9
$bytes=[Text.UTF8Encoding]::new($false).GetBytes($json);$stream=[IO.File]::Open($dest,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None);try{$stream.Write($bytes,0,$bytes.Length)}finally{$stream.Dispose()}
if ($r.firstError){Write-Host $r.firstError;exit 1};Write-Host "$role $Phase recorded";exit 0