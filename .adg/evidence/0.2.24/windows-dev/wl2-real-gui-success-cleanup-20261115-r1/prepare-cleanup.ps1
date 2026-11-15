param([Parameter(Mandatory)][string]$Code,[Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$Code=[IO.Path]::GetFullPath($Code);$Evidence=[IO.Path]::GetFullPath($Evidence)
if($Code -cne 'C:\Users\15119\.codex\worktrees\024-windows-native\code' -or $Evidence -cne 'C:\Users\15119\.codex\worktrees\official299-windows-evidence\docs-governance\.adg\evidence\0.2.24\windows-dev\wl2-real-gui-success-cleanup-20261115-r1'){throw 'exact owned worktrees required'}
$head=(& git -C $Code rev-parse HEAD).Trim()
if($head -cne '0549a660ee3f77ea243e798ec55b01f52545e492'){throw 'exact committed archive entry required'}
$controller=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
if($controller -cne 'S-1-5-21-587408064-856152790-2583328096-1001'){throw 'original controller required'}
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$root='C:\Users\Public\ocx-h024-gui-archive-20261115-r1'
if(Test-Path -LiteralPath $root){throw 'immutable new archive entry already exists'}
$acl=New-Object Security.AccessControl.DirectorySecurity
$acl.SetOwner([Security.Principal.SecurityIdentifier]::new($controller));$acl.SetAccessRuleProtection($true,$false)
foreach($who in @($controller,'S-1-5-18')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($who),[Security.AccessControl.FileSystemRights]::FullControl,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))}
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),[Security.AccessControl.FileSystemRights]::ReadAndExecute,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
[void][IO.Directory]::CreateDirectory($root,$acl)
$out=Join-Path $root 'out01';[void][IO.Directory]::CreateDirectory($out)
$outAcl=Get-Acl -LiteralPath $out
$outAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),[Security.AccessControl.FileSystemRights]::Modify,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
Set-Acl -LiteralPath $out -AclObject $outAcl
$payload=Join-Path $Evidence 'prepared-source';[void][IO.Directory]::CreateDirectory($payload)
foreach($name in @('archive-h024-fixed-success.cmd','archive-h024-fixed-success.ps1')){
 $source=Join-Path (Join-Path $Code 'test/windows024-native') $name
 [IO.File]::Copy($source,(Join-Path $root $name),$false)
 [IO.File]::Copy($source,(Join-Path $payload $name),$false)
 if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath (Join-Path $root $name) -Algorithm SHA256).Hash){throw 'prepared script bytes differ'}
}
$tokens=$null;$errors=$null
[void][Management.Automation.Language.Parser]::ParseFile((Join-Path $root 'archive-h024-fixed-success.ps1'),[ref]$tokens,[ref]$errors)
if(@($errors).Count -ne 0){throw 'prepared entry parse failed'}
$run=Join-Path (Split-Path $Evidence) 'wl2-real-gui-fixed-run-20261115-r1'
$export=Get-Content -LiteralPath (Join-Path $run 'same-sid-export01.json') -Raw|ConvertFrom-Json
if(!$export.complete -or $export.sid -cne $sid -or $export.session -ne 7 -or $export.base -cne 'C:\Users\marenben\ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35'){throw 'exact successful sameSID export required'}
$receipts=@()
foreach($name in @('launch-result.json','result.json','after-normal.json','after-hold.json','after-after-failure.json','after-finish.json','task-location.json','suspended-original.json')){
 $rows=@($export.files|Where-Object {$_.relative -ceq ('out/'+$name)})
 if($rows.Count -ne 1){throw 'missing or duplicate successful receipt'}
 $receipts+=@{relative=('out01/'+$name);sha256=$rows[0].sha256}
}
$receipts+=@{relative='out01/same-sid-export01.json';sha256=(Get-FileHash -LiteralPath (Join-Path $run 'same-sid-export01.json') -Algorithm SHA256).Hash.ToLowerInvariant()}
$receipts+=@{relative='input-manifest.json';sha256=(Get-FileHash -LiteralPath (Join-Path (Split-Path $Evidence) 'wl2-real-gui-profile-fix-20261115-r1/prepared-input-manifest.json') -Algorithm SHA256).Hash.ToLowerInvariant()}
$plan=@{schema=1;sid=$sid;session=7;successfulBase='C:\Users\marenben\ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35';publicRoot='C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1';receipts=$receipts;oldFailureTargets=@();brokerRawExit=$null;sourceHead=$head;scope='private sameSID full ZIPs and one exact successful Profile child deletion only; public source retained'}
$plan|ConvertTo-Json -Depth 9|Set-Content -LiteralPath (Join-Path $root 'archive-plan.json') -Encoding utf8
[IO.File]::Copy((Join-Path $root 'archive-plan.json'),(Join-Path $Evidence 'archive-plan.json'),$false)
$files=@(Get-ChildItem -LiteralPath $root -File -Recurse|ForEach-Object {@{path=$_.FullName.Substring($root.Length+1);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}})
@{sourceHead=$head;root=$root;files=$files;parseErrors=0;rootSddl=(Get-Acl -LiteralPath $root).Sddl;outSddl=(Get-Acl -LiteralPath $out).Sddl;runsStarted=0;deletionAttempted=$false;dailyPolicyChanged=$false;entry=(Join-Path $root 'archive-h024-fixed-success.cmd');accepted=$false}|ConvertTo-Json -Depth 9|Set-Content -LiteralPath (Join-Path $Evidence 'prepared-location.json') -Encoding utf8
Write-Output 'Prepared exact sameSID archive entry; no archive/deletion/desktop test started by controller.'
