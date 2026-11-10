param([Parameter(Mandatory)][string]$EvidenceDirectory,[Parameter(Mandatory)][string]$ExpectedCommit,[Parameter(Mandatory)][string]$ToolchainEnvironment)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$actual=(& git -C $root rev-parse HEAD).Trim()
if($actual -cne $ExpectedCommit){throw 'actual source commit differs'}
if(Test-Path -LiteralPath $EvidenceDirectory){throw 'new evidence directory required'}
New-Item -ItemType Directory -Path $EvidenceDirectory|Out-Null
. $ToolchainEnvironment
$manifest=Join-Path $PSScriptRoot Cargo.toml
$report=[ordered]@{scope='W0 standalone actual Rust platform module only';expectedCommit=$ExpectedCommit;actualCommit=$actual;accepted=$false;productIntegrated=$false;finalAcceptanceExecuted=$false;releaseExecuted=$false;commands=@();firstError=$null}
try{
 $report.toolchain=@{cargo=(& cargo --version);rustc=(& rustc --version);os=[Runtime.InteropServices.RuntimeInformation]::OSDescription;hostSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;elevated=([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)}
 foreach($kind in @('fmt','test','clippy')){
  $arguments=switch($kind){'fmt'{@('fmt','--manifest-path',$manifest,'--','--check')};'test'{@('test','--manifest-path',$manifest,'--locked','--offline','--','--test-threads=1','--nocapture')};'clippy'{@('clippy','--manifest-path',$manifest,'--locked','--offline','--all-targets','--','-D','warnings')}}
  $sw=[Diagnostics.Stopwatch]::StartNew();& cargo @arguments 1> (Join-Path $EvidenceDirectory ($kind+'.stdout.log')) 2> (Join-Path $EvidenceDirectory ($kind+'.stderr.log'));$exit=$LASTEXITCODE
  $report.commands+=@{program='cargo';arguments=$arguments;exitCode=$exit;elapsedMs=$sw.ElapsedMilliseconds}
  if($exit -ne 0){throw ('targeted '+$kind+' check failed')}
 }
 $report.accepted=$true
}catch{$report.firstError=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{
 $files=@(& git -C $root -c core.quotepath=false ls-files -- apps/desktop/tauri/src/infrastructure/windows_official_runtime024 test/windows024-native)
 $report.sourceFiles=@(foreach($rel in $files){$path=Join-Path $root $rel;@{path=$rel;bytes=(Get-Item -LiteralPath $path).Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}})
 $report.workingTreeStatus=@(& git -C $root status --porcelain -- apps/desktop/tauri/src/infrastructure/windows_official_runtime024 test/windows024-native)
 [IO.File]::WriteAllText((Join-Path $EvidenceDirectory result.json),($report|ConvertTo-Json -Depth 12))
}
if(!$report.accepted){exit 1}
