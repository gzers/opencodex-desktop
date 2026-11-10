# Remove only copied package/helper payloads, preserving every original failed output directory.
param([Parameter(Mandatory)][string]$EvidenceRoot,[Parameter(Mandatory)][string]$PreparationTempRoot,[string[]]$AttemptKeys=@('attempt-01','attempt-02','attempt-03','attempt-04'),[string[]]$PreparationLeaves=@('ocxd-official299-input-20261110','ocxd-official299-deps-20261110'))
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs')
$report=[ordered]@{utc=[datetime]::UtcNow.ToString('o');result='pending';attempts=@();preparation=@();error=$null;normalStopResultsUnchanged=$true}
try {
 foreach($prefix in $AttemptKeys){
  if($prefix -cnotmatch '^attempt-(?:r[245]-)?[0-9]{2}$'){throw 'explicit attempt identifier required'}
  $loc=Get-Content -LiteralPath (Join-Path $EvidenceRoot ($prefix+'-location.json')) -Raw|ConvertFrom-Json
  $dir=[IO.Path]::GetFullPath($loc.directory)
  if((Split-Path -Parent $dir) -ine 'C:\ProgramData' -or (Split-Path -Leaf $dir) -cnotmatch '^ocxd299-proof-[a-f0-9]{10}$'){throw 'payload cleanup escaped original fixture'}
  $c=Get-Content -LiteralPath (Join-Path $dir 'controller.json') -Raw|ConvertFrom-Json
  $source=Get-Content -LiteralPath (Join-Path $EvidenceRoot ($prefix+'/controller.json')) -Raw
  if($source -cne [IO.File]::ReadAllText((Join-Path $dir 'controller.json'))){throw 'archived controller differs from retained failure'}
  if($c.account.sid -cnotmatch '^S-1-5-21-[0-9-]+$' -or $c.account.name -cnotmatch '^ocx299[a-f0-9]{10}$'){throw 'not a new owned account'}
  if(@(Get-LocalUser|Where-Object {$_.SID.Value -ceq $c.account.sid}).Count -ne 0){throw 'test account remains'}
  $profiles=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.account.sid+"'"))
  if($profiles.Count -ne 0 -or (Test-Path -LiteralPath $c.profile)){throw 'test Profile or directory remains'}
  $ports=@();if($null -ne $c.ordinary.start){
   $text=[IO.File]::ReadAllText((Join-Path $dir 'ordinary/startup.stdout.log'))
   $numbers=[Collections.Generic.HashSet[int]]::new();[void]$numbers.Add([int]$c.ordinary.port)
   $history=Join-Path $dir 'ordinary/listener-history.json';if(Test-Path -LiteralPath $history){foreach($row in @(Get-Content -LiteralPath $history -Raw|ConvertFrom-Json)){if($row.sid -cne $c.account.sid){throw 'listener history escaped new account SID'};[void]$numbers.Add([int]$row.port)}}
   foreach($match in [regex]::Matches($text,'Claude intercept proxy active on http://127\.0\.0\.1:([0-9]+)')){[void]$numbers.Add([int]$match.Groups[1].Value)}
   foreach($port in $numbers){$owners=@([Official299Proof.Native]::Listeners($port));$ports+=@{port=$port;listenerOwners=$owners;scope='post-execution-cleanup-readback; does not retroactively satisfy any action budget'};if($owners.Count){throw 'previous test port currently has a listener; retain without killing it'}}
  }
  $removed=@()
  foreach($leaf in @('input','archive','powershell','code')){
   $path=[IO.Path]::GetFullPath((Join-Path $dir $leaf))
   if((Split-Path -Parent $path) -ine $dir){throw 'payload escaped exact run root'}
   $g=[Official299Proof.Guard]::new($path,$true);$g.Dispose()
   if((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'linked payload refused'}
   Remove-Item -LiteralPath $path -Recurse -Force
   $removed+=@{path=$path;existsAfter=(Test-Path -LiteralPath $path)}
   if(Test-Path -LiteralPath $path){throw 'payload remains after removal'}
  }
  $report.attempts+=@{attempt=$prefix;accountSid=$c.account.sid;accountExists=$false;profileRecords=$profiles;profileDirectoryExists=$false;ports=$ports;payloads=$removed;failedDirectoryPreserved=(Test-Path -LiteralPath $dir);originalResultPreserved=(Test-Path -LiteralPath (Join-Path $dir 'ordinary/result.json'));scheduledTask='no scheduled task creation call in this harness; not used'}
 }
 foreach($leaf in $PreparationLeaves){
  if($leaf -cnotin @('ocxd-official299-input-20261110','ocxd-official299-deps-20261110','ocxd299-r4-official','ocxd299-r4-deps','ocxd299-r5-official','ocxd299-r5-deps')){throw 'unregistered task preparation directory'}
  $tempRoot=[IO.Path]::GetFullPath($PreparationTempRoot);$path=[IO.Path]::GetFullPath((Join-Path $tempRoot $leaf))
  if((Split-Path -Parent $path) -ine $tempRoot -or $tempRoot -ine 'C:\Users\15119\AppData\Local\Temp'){throw 'preparation cleanup escaped explicitly created task directories'}
  $g=[Official299Proof.Guard]::new($path,$true);$g.Dispose();Remove-Item -LiteralPath $path -Recurse -Force
  $report.preparation+=@{path=$path;existsAfter=(Test-Path -LiteralPath $path)}
 }
 $report.result='pass'
}catch{$report.result='fail';$report.error=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
[IO.File]::WriteAllText((Join-Path $EvidenceRoot 'payload-cleanup.json'),($report|ConvertTo-Json -Depth 20))
if($report.result -cne 'pass'){exit 1}
