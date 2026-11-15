param([Parameter(Mandatory)][string]$Code,[Parameter(Mandatory)][string]$Evidence,[Parameter(Mandatory)][string]$ExpectedHead)
$ErrorActionPreference='Stop'
$root='C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1'
$old='C:\Users\Public\ocx-h024-actual-gui-20261115-r1'
if(Test-Path -LiteralPath $root){throw 'new immutable fixed GUI root already exists; do not overwrite'}
$Code=[IO.Path]::GetFullPath($Code)
$Evidence=[IO.Path]::GetFullPath($Evidence)
if($Code -cne 'C:\Users\15119\.codex\worktrees\024-windows-native\code'){throw 'exact source worktree required'}
if($Evidence -cne 'C:\Users\15119\.codex\worktrees\official299-windows-evidence\docs-governance\.adg\evidence\0.2.24\windows-dev\wl2-real-gui-profile-fix-20261115-r1'){throw 'exact evidence round required'}
$controller=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
if($controller -cne 'S-1-5-21-587408064-856152790-2583328096-1001'){throw 'original controller SID required, no elevation'}
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$head=(& git -C $Code rev-parse HEAD).Trim()
if($head -cne $ExpectedHead){throw 'committed source changed'}
$product=Get-Content -LiteralPath (Join-Path $Evidence 'product01-artifacts.json') -Raw|ConvertFrom-Json
$library=Get-Content -LiteralPath (Join-Path $Evidence 'library01-artifact.json') -Raw|ConvertFrom-Json
$sources=@(
 @{name='h-gui-tests.exe';path=$library.path;bytes=$library.bytes;sha256=$library.sha256},
 @{name='opencodex-desktop.exe';path=$product[0].path;bytes=$product[0].bytes;sha256=$product[0].sha256},
 @{name='ocxd.exe';path=$product[1].path;bytes=$product[1].bytes;sha256=$product[1].sha256}
)
foreach($row in $sources){
 $item=Get-Item -LiteralPath $row.path
 if($item.Length -ne $row.bytes -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or (Get-FileHash -LiteralPath $row.path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256){throw 'exact compiled artifact differs'}
}
$acl=New-Object Security.AccessControl.DirectorySecurity
$acl.SetOwner([Security.Principal.SecurityIdentifier]::new($controller));$acl.SetAccessRuleProtection($true,$false)
foreach($who in @($controller,'S-1-5-18')){
 $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($who),[Security.AccessControl.FileSystemRights]::FullControl,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
}
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),[Security.AccessControl.FileSystemRights]::ReadAndExecute,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
[void][IO.Directory]::CreateDirectory($root,$acl)
[void][IO.Directory]::CreateDirectory((Join-Path $root 'input'))
$out=Join-Path $root 'out01';[void][IO.Directory]::CreateDirectory($out)
$outAcl=Get-Acl -LiteralPath $out
$outAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),[Security.AccessControl.FileSystemRights]::Modify,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
Set-Acl -LiteralPath $out -AclObject $outAcl
foreach($row in $sources){
 $target=Join-Path (Join-Path $root 'input') $row.name
 [IO.File]::Copy($row.path,$target,$false)
 if((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256){throw 'copied input digest differs'}
}
$transforms=@()
foreach($name in @('launch-h024-actual-gui.cmd','launch-h024-actual-gui.ps1','collect-h024-actual-gui.ps1')){
 $source=Join-Path (Join-Path $Code 'test/windows024-native') $name
 $text=[IO.File]::ReadAllText($source)
 $fixed=$text.Replace($old,$root)
 if($name.EndsWith('.ps1') -and $fixed -ceq $text){throw 'expected exact hardcoded task root substitution missing'}
 [IO.File]::WriteAllText((Join-Path $root $name),$fixed,[Text.UTF8Encoding]::new($false))
 $transforms+=@{name=$name;source=$source;sourceSha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant();transformation='replace exact old task root only; all commands/guards/budgets unchanged'}
 $copy=Join-Path (Join-Path $Evidence 'prepared-scripts') $name
 [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($copy));[IO.File]::Copy((Join-Path $root $name),$copy,$false)
}
$manifest=@{files=@($sources|ForEach-Object {@{name=$_.name;bytes=$_.bytes;sha256=$_.sha256}});productSourceHead=$head;ordinaryDesktopRequired=$true;scope='actual product GUI and empty broker only; no official Owner or service'}
$manifest|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'input-manifest.json') -Encoding utf8
[IO.File]::Copy((Join-Path $root 'input-manifest.json'),(Join-Path $Evidence 'prepared-input-manifest.json'),$false)
$errors=@()
foreach($name in @('launch-h024-actual-gui.ps1','collect-h024-actual-gui.ps1')){
 $tokens=$null;$parseErrors=$null
 [void][Management.Automation.Language.Parser]::ParseFile((Join-Path $root $name),[ref]$tokens,[ref]$parseErrors)
 $errors+=@($parseErrors)
}
if($errors.Count -ne 0){throw 'prepared script parse failed'}
$files=@(Get-ChildItem -LiteralPath $root -File -Recurse|ForEach-Object {@{path=$_.FullName.Substring($root.Length+1);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}})
@{root=$root;sourceHead=$head;files=$files;scriptTransforms=$transforms;rootSddl=(Get-Acl -LiteralPath $root).Sddl;outSddl=(Get-Acl -LiteralPath $out).Sddl;inputHashesVerified=$true;parseErrors=0;runsStarted=0;oldTaskRootChanged=$false;dailyPolicyChanged=$false;launchEntry=(Join-Path $root 'launch-h024-actual-gui.cmd');accepted=$false}|ConvertTo-Json -Depth 9|Set-Content -LiteralPath (Join-Path $Evidence 'prepared-location.json') -Encoding utf8
Write-Output 'Prepared new fixed GUI root; no desktop process started and original failed task untouched.'
