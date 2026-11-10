param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$ExpectedHostSha256)
$ErrorActionPreference='Stop'
$dir=[IO.Path]::GetFullPath($Directory)
if((Split-Path -Parent $dir) -ine 'C:\ProgramData' -or (Split-Path -Leaf $dir) -cnotmatch '^ocxd299-proof-[a-f0-9]{10}$'){throw 'unowned run directory'}
$c=Get-Content -LiteralPath (Join-Path $dir 'controller.json') -Raw|ConvertFrom-Json
$r=[ordered]@{kind='separate-early-failure-cleanup';account=$c.account;profile=$c.profile;deleted=$false;error=$null}
try {
 $source=Join-Path $dir 'code/host.ps1'
 if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ine $ExpectedHostSha256 -or $c.hostHashes.host -cne $ExpectedHostSha256){throw 'actual attempted source differs'}
 if($c.ordinaryOriginalExit -ne 1 -or $c.ordinary.stage -cne 'ordinary-identity' -or $c.ordinary.firstError.line -ne 40 -or $null -ne $c.ordinary.start -or $c.ordinary.firstError.type -cne 'System.Management.Automation.SessionStateUnauthorizedAccessException'){throw 'not the exact pre-process HOME-variable failure'}
 $r.noRuntimeProof=@{exactSourceSha256=$ExpectedHostSha256;failureLine=40;nativeConstructorLine=96;ordinaryOriginalExit=$c.ordinaryOriginalExit;processCreationNotReached=$true}
 $a=Get-LocalUser -SID $c.account.sid
 if($a.Name -cne $c.account.name -or $a.Name -cnotmatch '^ocx299[a-f0-9]{10}$' -or $a.Enabled){throw 'exact disabled new account required'}
 $profiles=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.account.sid+"'"))
 if($profiles.Count -ne 1 -or $profiles[0].Loaded -or $profiles[0].Special -or $profiles[0].LocalPath -ine $c.profile){throw 'Profile deletion gate refused'}
 $r.profileBefore=@($profiles|Select-Object SID,LocalPath,Loaded,Special);$profiles[0]|Remove-CimInstance
 if(@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.account.sid+"'")).Count -ne 0 -or (Test-Path -LiteralPath $c.profile)){throw 'Profile still present'}
 Remove-LocalUser -SID $c.account.sid
 $r.deleted=(@(Get-LocalUser|Where-Object {$_.SID.Value -ceq $c.account.sid}).Count -eq 0);$r.profileAfter=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.account.sid+"'"));$r.profileDirectoryExists=Test-Path -LiteralPath $c.profile
}catch{$r.error=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
[IO.File]::WriteAllText((Join-Path $dir 'early-cleanup.json'),($r|ConvertTo-Json -Depth 12))
if(!$r.deleted){exit 1}
