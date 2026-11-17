param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-authorized-cleanup-20261117-r1'){throw 'exact authorized two-target entry required'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original marenben ordinary desktop session7 required'}
$private='C:\Users\marenben\ocx024-r6-authorized-cleanup-20261117-r1'
if(Test-Path -LiteralPath $private){throw 'one-time cleanup observation exists; no retry or alternate output'}
$publicOut=Join-Path $rootPath 'out01'
if(Test-Path -LiteralPath (Join-Path $publicOut 'cleanup-result.json')){throw 'immutable cleanup result already recorded'}
function WriteNewJson([string]$Path,$Value) {
 $bytes=[Text.UTF8Encoding]::new($false).GetBytes(($Value|ConvertTo-Json -Depth 16))
 $file=[IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush($true)}finally{$file.Dispose()}
}
function PathFacts {
 @('User','Machine')|ForEach-Object {
  $value=[Environment]::GetEnvironmentVariable('Path',$_);$digest=$null
  if($null -ne $value){$alg=[Security.Cryptography.SHA256]::Create();try{$digest=([BitConverter]::ToString($alg.ComputeHash([Text.Encoding]::Unicode.GetBytes($value)))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}
  @{scope=$_;present=$null -ne $value;sha256=$digest}
 }|ConvertTo-Json -Compress
}
$beforePath=PathFacts
$result=[ordered]@{sid=$identity.User.Value;session=7;scope='user explicitly authorized external failure cleanup of exact50580 then54692; all files retained';normalStop=$false;closed=$false;allCompanionsProven=$false;publicStartOpen=$false;allAccepted=$false;filesDeleted=$false;officialServiceStarted=$false;old40588Touched=$false;completed=$false}
$prepared=$null
try {
 $manifest=Get-Content -LiteralPath (Join-Path $rootPath 'input-manifest.json') -Raw -Encoding UTF8|ConvertFrom-Json
 if($manifest.scope -cne 'authorized-r6-two-target-forced-cleanup-v1' -or $manifest.expectedSid -cne $identity.User.Value -or $manifest.expectedSession -ne 7){throw 'exact prepared action/SID/session differs'}
 $seen=[Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
 foreach($row in $manifest.files){
  if($row.name -notin @('cleanup-r6-authorized-processes.cs','cleanup-r6-authorized-processes.ps1','cleanup-r6-authorized-processes.cmd')){throw 'unknown prepared input'}
  if(!$seen.Add([string]$row.name)){throw 'duplicate prepared input'}
  $path=Join-Path $rootPath $row.name;$item=Get-Item -LiteralPath $path -Force
  if(($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $row.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256){throw 'pinned cleanup input differs'}
 }
 if(@($manifest.files).Count -ne 3){throw 'complete input set required'}
 [void][IO.Directory]::CreateDirectory($private)
 $dir=Get-Item -LiteralPath $private -Force
 if($dir.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'new private output reparse refused'}
 Add-Type -Path (Join-Path $rootPath 'cleanup-r6-authorized-processes.cs')
 $prepared=[R6AuthorizedFailureCleanup]::PrepareExact()
 $result.preflight=($prepared.report|ConvertTo-Json -Depth 12|ConvertFrom-Json)
 WriteNewJson (Join-Path $private 'native-preflight.json') $prepared.report
 WriteNewJson (Join-Path $publicOut 'native-preflight.json') $prepared.report
 WriteNewJson (Join-Path $private 'forced-intent.json') @{time=[DateTimeOffset]::Now.ToString('o');targets=@(50580,54692);requestedExit=242;externalFailureCleanup=$true;normalStop=$false;filesRetained=$true}
 $result.cleanup=$prepared.CommitOnce()
 $result.completed=$result.cleanup.completed -and $result.cleanup.sourcesRecheckedAfter -and @($result.cleanup.terminatedPids).Count -eq 2 -and $result.cleanup.rawExits[0] -eq 242 -and $result.cleanup.rawExits[1] -eq 242
 if(!$result.completed){throw 'same-handle forced exits/source receipts unconfirmed'}
}catch {
 $result.firstError=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}
 if($null -ne $prepared){$result.cleanup=$prepared.report}
 $_|Out-String|Set-Content -LiteralPath (Join-Path $publicOut 'cleanup-error.log') -Encoding UTF8
}finally {
 $afterPath=PathFacts;$result.dailyPathBefore=ConvertFrom-Json $beforePath;$result.dailyPathAfter=ConvertFrom-Json $afterPath;$result.dailyPathUnchanged=$beforePath -ceq $afterPath
 if($null -ne $prepared){try{$prepared.Dispose();$result.freshObservationHandlesClosed=$true}catch{$result.disposeError=$_.Exception.ToString()}}
 if(Test-Path -LiteralPath $private){WriteNewJson (Join-Path $private 'cleanup-result.json') $result}
 WriteNewJson (Join-Path $publicOut 'cleanup-result.json') $result
}
if($result.completed -and $result.dailyPathUnchanged){Write-Host 'Authorized failure cleanup confirmed: Node242 then broker242. All files retained. Not NormalStop/Closed.';exit 0}
Write-Host 'Cleanup refused or incomplete. Do not rerun. See out01/cleanup-result.json; all evidence retained.'
exit 1
