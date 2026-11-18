$ErrorActionPreference='Stop'
$publicRoot='C:\Users\Public\ocx-r7-private-native-20261118-r1'
$evidenceRoot=$PSScriptRoot
$codeRoot='C:\Users\15119\.codex\worktrees\r6-windows-native-resume\code'
$image=Join-Path $codeRoot 'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/ocxd.exe'
$expected='55cd6ef08653edc8bf2b5c754006473fba7b3fc8f4a0a04c486ea89222181d3e'
if((Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected){throw 'final exact broker artifact changed'}
$check=Get-Content -LiteralPath (Join-Path $evidenceRoot 'launcher-ps51-check.json') -Raw|ConvertFrom-Json
$launcher=Join-Path $codeRoot 'test/windows024-native/launch-r7-private-native-empty.ps1'
if($check.raw -ne 0 -or (Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash.ToLowerInvariant() -cne $check.scriptSha256){throw 'actual PS5-tested launcher changed'}
if(Test-Path -LiteralPath $publicRoot){throw 'prepared root already exists, no replace/adopt'}
$current=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$ordinary='S-1-5-21-587408064-856152790-2583328096-1003'
foreach($path in @($publicRoot,(Join-Path $publicRoot 'input'),(Join-Path $publicRoot 'out01'))){
 $rights=if((Split-Path $path -Leaf) -ceq 'out01'){'0x1301bf'}else{'0x1200a9'}
 $security=New-Object Security.AccessControl.DirectorySecurity
 $security.SetSecurityDescriptorSddlForm(('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)(A;OICI;{2};;;{1})' -f $current,$ordinary,$rights))
 [void][IO.Directory]::CreateDirectory($path,$security)
}
$target=Join-Path $publicRoot 'input/ocxd.exe';[IO.File]::Copy($image,$target,$false)
foreach($name in @('launch-r7-private-native-empty.ps1','launch-r7-private-native-empty.cmd')){[IO.File]::Copy((Join-Path $codeRoot ('test/windows024-native/'+$name)),(Join-Path $publicRoot $name),$false)}
$utf8=New-Object Text.UTF8Encoding($false)
$manifest=@{scope='isolated-private-control-v2-empty-fixture';expectedSid=$ordinary;expectedSession=7;files=@(@{name='ocxd.exe';bytes=(Get-Item -LiteralPath $image).Length;sha256=$expected});noOfficial=$true;noRuntimeOwner=$true;privateRoot='C:\Users\marenben\ocx024-r7-private-native-20261118-r1';taskFilesRetained=$true;oldFailuresUntouched=$true}
[IO.File]::WriteAllText((Join-Path $publicRoot 'input-manifest.json'),($manifest|ConvertTo-Json -Depth 8),$utf8)
$rows=@(Get-ChildItem -LiteralPath $publicRoot -Recurse -File|ForEach-Object {@{path=$_.FullName.Substring($publicRoot.Length+1);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}})
if($rows.Count -ne 4 -or (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected){throw 'prepared input membership/hash wrong'}
$stage=@{publicRoot=$publicRoot;privateRoot=$manifest.privateRoot;files=$rows;outDirectoryEmpty=@(Get-ChildItem -LiteralPath (Join-Path $publicRoot 'out01') -Force).Count -eq 0;account='marenben';expectedSid=$ordinary;expectedSession=7;actualDesktopExecution='Pending, user desktop required';currentR2Touched=$false;noOfficial=$true;artifactSha256=$expected;launcherPs51CheckSha256=$check.scriptSha256;acl=@(Get-Acl -LiteralPath $publicRoot|Select-Object Path,Sddl;Get-Acl -LiteralPath (Join-Path $publicRoot 'out01')|Select-Object Path,Sddl)}
[IO.File]::WriteAllText((Join-Path $evidenceRoot 'desktop-staged.json'),($stage|ConvertTo-Json -Depth 12),$utf8)
$stage|ConvertTo-Json -Depth 8
