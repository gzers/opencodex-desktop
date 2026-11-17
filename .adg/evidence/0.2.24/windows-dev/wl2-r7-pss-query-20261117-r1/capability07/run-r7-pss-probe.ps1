param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$evidencePath=[IO.Path]::GetFullPath($Evidence)
if(Test-Path -LiteralPath $evidencePath){throw 'fresh immutable capability evidence required'}
[void][IO.Directory]::CreateDirectory($evidencePath)
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. "$PSScriptRoot/msvc-env.ps1" -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$source=Join-Path $PSScriptRoot 'r7-pss-job-probe.cpp'
$header=Join-Path $env:WindowsSdkDir ('Include/'+$env:WindowsSDKVersion.TrimEnd([char]92)+'/um/ProcessSnapshot.h')
foreach($row in @(@{name='r7-pss-job-probe.cpp';path=$source},@{name='ProcessSnapshot.h';path=$header},@{name='run-r7-pss-probe.ps1';path=$PSCommandPath})){
 [IO.File]::Copy($row.path,(Join-Path $evidencePath $row.name),$false)
}
$build=Join-Path $PSScriptRoot 'target/r7-pss-query'
[void][IO.Directory]::CreateDirectory($build)
$id=[guid]::NewGuid().ToString('N')
$compilerOutput=Join-Path $build ('build-'+$id+'.exe')
$object=Join-Path $build ('build-'+$id+'.obj')
$watch=[Diagnostics.Stopwatch]::StartNew()
& cl.exe /nologo /std:c++17 /EHsc /W4 /MT /DUNICODE /D_UNICODE /D_WIN32_WINNT=0x0A00 "/Fe:$compilerOutput" "/Fo:$object" $source advapi32.lib kernel32.lib 1> (Join-Path $evidencePath 'build.stdout.log') 2> (Join-Path $evidencePath 'build.stderr.log')
$compiled=$LASTEXITCODE;$raw=$compiled;$exe=$null
if($compiled -eq 0){
 $exe=Join-Path $build ('run-'+$id+'.exe')
 $output=[IO.File]::Open($exe,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$bytes=[IO.File]::ReadAllBytes($compilerOutput);$output.Write($bytes,0,$bytes.Length);$output.Flush($true)}finally{$output.Dispose()}
 & $exe 1> (Join-Path $evidencePath 'stdout.log') 2> (Join-Path $evidencePath 'stderr.log')
 $raw=$LASTEXITCODE
}
$artifact=if($exe){@{path=$exe;bytes=(Get-Item -LiteralPath $exe).Length;sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()}}else{$null}
@{compiledRaw=$compiled;raw=$raw;elapsedMs=$watch.ElapsedMilliseconds;head=(git rev-parse HEAD);sourceSha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant();artifact=$artifact;scope='self-owned broker/Job PSS capability only; not accepted prototype or current R0';accepted=$false;currentR2Touched=$false;noElevation=$true;noVAClone=$true;noObjectNamesRead=$true;knownFixtureHandleNumbersNeverDuplicateAuthority=$true;fullTwoPassAndCounterProof=$false}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $evidencePath 'result.json') -Encoding utf8
exit $raw
