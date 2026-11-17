param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath($Evidence)
$target=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'target'))
if([IO.Path]::GetDirectoryName($root) -cne $target -or [IO.Path]::GetFileName($root) -cnotmatch '^r6-archive-proof-[a-z0-9-]+$'){throw 'fresh directed test target required'}
if(Test-Path -LiteralPath $root){throw 'immutable test output exists'}
[void][IO.Directory]::CreateDirectory($root)
. (Join-Path $PSScriptRoot 'archive-r6-content.ps1')
$result=[ordered]@{scope='controller-owned synthetic content trees only; exact helper consumed by ordinary archive script';powershell=$PSVersionTable.PSVersion.ToString();passed=$false;ordinaryTokenAcceptance=$false;processAction=$false;sourceDeleted=$false;sourcePaths=@();negatives=@()}
$held=$null
try{
 foreach($n in @('A','B','Empty','A/nested','A/nested/empty','extra')){[void][IO.Directory]::CreateDirectory((Join-Path $root $n))}
 for($i=0;$i -lt 64;$i++){[IO.File]::WriteAllBytes((Join-Path $root ('A/nested/f{0:00}.json' -f $i)),[Text.Encoding]::UTF8.GetBytes(('fixture-'+$i)))}
 [IO.File]::WriteAllBytes((Join-Path $root ('B/'+[char]0x4E2D+[char]0x6587+'.txt')),[Text.Encoding]::UTF8.GetBytes('literal unicode filename'))
 [IO.File]::WriteAllBytes((Join-Path $root 'B/zero.sqlite'),[byte[]]@())
 $bytes=New-Object byte[] 1048576;[Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes);[IO.File]::WriteAllBytes((Join-Path $root 'A/payload.bin'),$bytes)
 foreach($n in @('one.sqlite','two.sqlite')){[IO.File]::WriteAllBytes((Join-Path $root ('extra/'+$n)),[byte[]]@())}
 $log=Join-Path $root 'B/held.log';$held=[IO.File]::Open($log,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
 $raw=[Text.Encoding]::UTF8.GetBytes('held-open stable task log');$held.Write($raw,0,$raw.Length);$held.Flush()
 $roots=@(@{source=(Join-Path $root 'A');relative='task'},@{source=(Join-Path $root 'B');relative='client'},@{source=(Join-Path $root 'Empty');relative='empty'})
 $extras=@(@{source=(Join-Path $root 'extra/one.sqlite');relative='locks/one.sqlite'},@{source=(Join-Path $root 'extra/two.sqlite');relative='locks/two.sqlite'})
 $zip=Join-Path $root 'proof.zip'
 $archive=Invoke-R6ContentArchive -Roots $roots -ExtraFiles $extras -ZipPath $zip
 $result.archive=$archive
 if(!$archive.archiveVerified -or $archive.files.Count -ne 70 -or $archive.fileErrors.Count){throw 'actual full70member ZIP did not verify'}
 $result.archive=$archive;$result.sourcePaths=@($roots|ForEach-Object {$_.source})
 # Same input collection type that failed under PS51 must sum exactly.
 [long]$manual=0;foreach($row in $archive.files){$manual+=[long]$row.bytes}
 if($manual -ne $archive.totalSourceBytes){throw 'bounded explicitInt64 total differs from byte receipts'}
 $duplicateRefused=$false
 try{[void](Invoke-R6ContentArchive $roots $extras $zip)}catch{$duplicateRefused=$true;$result.negatives+=@{case='existingZIP';error=$_.Exception.Message}}
 if(!$duplicateRefused){throw 'existing ZIP was overwritten'}
 foreach($case in @('duplicate','relativeEscape','missing')){
  $invalidRoots=$roots;$invalidExtras=$extras
  if($case -eq 'duplicate'){$invalidRoots=@($roots[0],$roots[0])}
  if($case -eq 'relativeEscape'){$invalidRoots=@(@{source=$roots[0].source;relative='../escape'})}
  if($case -eq 'missing'){$invalidExtras=@(@{source=(Join-Path $root 'absent');relative='missing'})}
  $path=Join-Path $root ($case+'.zip');$refused=$false
  try{[void](Invoke-R6ContentArchive $invalidRoots $invalidExtras $path)}catch{$refused=$true;$result.negatives+=@{case=$case;error=$_.Exception.Message}}
  if(!$refused -or (Test-Path -LiteralPath $path)){throw 'invalid complete inventory created ZIP or was accepted'}
 }
 $result.manualByteSum=$manual;$result.passed=$true
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{
 if($null -ne $held){$held.Dispose()}
 $result.helperSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'archive-r6-content.ps1') -Algorithm SHA256).Hash.ToLowerInvariant()
 $result|ConvertTo-Json -Depth 16|Set-Content -LiteralPath (Join-Path $root 'result.json') -Encoding utf8
}
if(!$result.passed){throw 'directed archive proof failed; originals and result retained'}
