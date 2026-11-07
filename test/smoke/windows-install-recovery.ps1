param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$NpmCli,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [string]$DriverNode=''
)
$ErrorActionPreference='Stop'
if(!$IsWindows){throw 'Native Windows required'}
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskNpm=(Resolve-Path -LiteralPath $NpmCli).Path
if(!$DriverNode){$DriverNode=$Node}
$taskDriver=(Resolve-Path -LiteralPath $DriverNode).Path
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh evidence directory required'}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
$taskSandbox=Join-Path $taskOutput 'OpenCodeX Native 中文 Spaces'
$taskSmokeVersion=(Get-Content -LiteralPath (Join-Path $PSScriptRoot '../../apps/desktop/tauri/tauri.conf.json') -Raw|ConvertFrom-Json).version
$taskDependencyContract='legacy'
$taskData=if($taskSmokeVersion -ceq '0.2.23'){Join-Path $taskSandbox 'data/desktop'}else{$taskSandbox}
New-Item -ItemType Directory -Path (Join-Path $taskSandbox 'home') -Force|Out-Null
if($taskSmokeVersion -ceq '0.2.23'){
 # Sandbox intentionally skips bundle provisioning. Supply the explicit native
 # dependency pair at the production fixed paths, never through host PATH.
 $taskTools=Join-Path $taskSandbox 'program/runtime/node'
 $taskNpmRoot=Split-Path -Parent (Split-Path -Parent $taskNpm)
 if((Split-Path -Leaf $taskNpm) -cne 'npm-cli.js' -or !(Test-Path -LiteralPath (Join-Path $taskNpmRoot 'package.json'))){throw 'Explicit complete npm module required'}
 New-Item -ItemType Directory -Path (Join-Path $taskTools 'node_modules') -Force|Out-Null
 Copy-Item -LiteralPath $taskNode -Destination (Join-Path $taskTools 'node.exe')
 Copy-Item -LiteralPath $taskNpmRoot -Destination (Join-Path $taskTools 'node_modules/npm') -Recurse
 $taskPins=Get-Content -LiteralPath (Join-Path $PSScriptRoot '../../.github/runtime-platforms-023.json') -Raw|ConvertFrom-Json
 $taskActualNode=(& $taskNode --version).Trim()
 if($LASTEXITCODE -ne 0){throw 'Explicit native Node version read failed'}
 $taskActualNpm=(Get-Content -LiteralPath (Join-Path $taskNpmRoot 'package.json') -Raw|ConvertFrom-Json).version
 if($taskActualNode -ceq ('v'+$taskPins.node.version) -and $taskActualNpm -ceq $taskPins.node.npmVersion){$taskDependencyContract='pinned-ready'}
 elseif($taskActualNode -ceq 'v22.20.0'){$taskDependencyContract='node22-compatibility-not-ready'}
 else{throw '023 ready probe requires exact production dependencies; only explicit Node22 matrix is admitted as negative readiness compatibility'}

}else{
 $taskTools=Join-Path $taskSandbox 'home/.local/bin'
 New-Item -ItemType Directory -Path $taskTools -Force|Out-Null
 Copy-Item -LiteralPath $taskNode -Destination (Join-Path $taskTools 'node.exe')
 $taskWrapper='@echo off'+[Environment]::NewLine+'"%~dp0node.exe" "'+$taskNpm+'" %*'+[Environment]::NewLine
 [IO.File]::WriteAllText((Join-Path $taskTools 'npm.cmd'),$taskWrapper,[Text.UTF8Encoding]::new($false))
}
$taskListener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskListener.Start();$taskPort=$taskListener.LocalEndpoint.Port;$taskListener.Stop()
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe
$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe
$taskInfo.UseShellExecute=$false
$taskInfo.CreateNoWindow=$true
$taskInfo.WindowStyle='Hidden'
$taskInfo.RedirectStandardError=$true
$taskInfo.RedirectStandardOutput=$true
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskSandbox
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
foreach($taskVariable in @('HOME','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
 [void]$taskInfo.Environment.Remove($taskVariable)
}
$taskRecord=[ordered]@{
 source=(git rev-parse HEAD).Trim();executable=$taskExe;sha256=(Get-FileHash -LiteralPath $taskExe).Hash
 node=$taskNode;npmCli=$taskNpm;dataRoot=$taskData;dependencyContract=$taskDependencyContract;actualNode=$taskActualNode;actualNpm=$taskActualNpm;fixedDependencies=($taskSmokeVersion -ceq '0.2.23');workingDirectory=$taskInfo.WorkingDirectory;sandbox=$taskSandbox;port=$taskPort
 startedAt=[DateTimeOffset]::Now.ToString('o');result='fail'
}
$taskProcess=$null
try{
 $taskProcess=[Diagnostics.Process]::Start($taskInfo)
 $taskRecord.pid=$taskProcess.Id
 $taskOut=$taskProcess.StandardOutput.ReadToEndAsync()
 $taskErr=$taskProcess.StandardError.ReadToEndAsync()
 # The protocol client needs Node >=24; the app uses the explicit native Node above.
 & $taskDriver (Join-Path $PSScriptRoot 'windows-install-recovery.mjs') "http://127.0.0.1:$taskPort" $taskOutput $taskSandbox $taskDependencyContract *> (Join-Path $taskOutput 'execution.log')
 if($LASTEXITCODE -ne 0){throw 'Native WebView install/recovery assertions failed; see execution.log'}
 $taskRecord.result='pass'
}finally{
 if($taskProcess -and !$taskProcess.HasExited){$taskProcess.Kill($true);$taskProcess.WaitForExit()}
 if($taskProcess){
  [IO.File]::WriteAllText((Join-Path $taskOutput 'application-stdout.log'),$taskOut.GetAwaiter().GetResult())
  [IO.File]::WriteAllText((Join-Path $taskOutput 'application-stderr.log'),$taskErr.GetAwaiter().GetResult())
 }
 $taskAppLog=Join-Path $taskData 'logs/app.log'
 if(Test-Path -LiteralPath $taskAppLog){
  $taskStartup=Get-Content -LiteralPath $taskAppLog | Select-String 'startup version=([^ ]+) commit=([0-9a-f]+)' | Select-Object -First 1
  if($taskStartup){
   $taskRecord.startupVersion=$taskStartup.Matches[0].Groups[1].Value
   $taskRecord.startupCommit=$taskStartup.Matches[0].Groups[2].Value
   $taskRecord.applicationTree=(git rev-parse "$($taskRecord.startupCommit):apps").Trim()
   $taskRecord.driverApplicationTree=(git rev-parse 'HEAD:apps').Trim()
   $taskRecord.applicationSourceMatchesDriver=$taskRecord.applicationTree -eq $taskRecord.driverApplicationTree
  }
 }
 $taskRecord.endedAt=[DateTimeOffset]::Now.ToString('o')
 $taskRecord|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $taskOutput 'launcher.json') -Encoding utf8
}
