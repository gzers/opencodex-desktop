param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true'){throw 'Restricted to an ephemeral Windows Actions runner'}
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $taskOutput|Out-Null
$taskName='ocxacc'+[guid]::NewGuid().ToString('N').Substring(0,8)
$taskSecret=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
$taskSecure=ConvertTo-SecureString ([Convert]::ToBase64String($taskSecret)+'aA1!') -AsPlainText -Force
$taskUser=$null
try {
 $taskUser=New-LocalUser -Name $taskName -Password $taskSecure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable OpenCodeX CI acceptance account'
 $taskUsers=Get-LocalGroup -SID 'S-1-5-32-545'
 Add-LocalGroupMember -Group $taskUsers -Member $taskUser
 & icacls.exe $taskOutput /grant ($taskName+':(OI)(CI)M') | Out-Null
 if($LASTEXITCODE -ne 0){throw 'Cannot provision isolated acceptance output directory'}
 $taskChild=Join-Path $taskOutput 'ordinary-child.ps1'
 $taskConfig=Join-Path $taskOutput 'ordinary-input.json'
 [ordered]@{executable=$taskExe;node=$taskNode;output=(Join-Path $taskOutput 'startup');script=(Join-Path $PSScriptRoot 'windows-startup.ps1');commit=(& git rev-parse HEAD).Trim()} | ConvertTo-Json | Set-Content -LiteralPath $taskConfig -Encoding utf8
 @'
param([string]$InputFile)
$ErrorActionPreference='Stop'
try{
 $c=Get-Content -LiteralPath $InputFile -Raw | ConvertFrom-Json
 & $c.script -Executable $c.executable -Node $c.node -OutputDirectory $c.output -Mode ordinary -ExpectedCommit $c.commit
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 exit 0
}catch{[Console]::Error.WriteLine($_.Exception.Message);exit 1}
'@ | Set-Content -LiteralPath $taskChild -Encoding utf8
 $taskCredential=[pscredential]::new(($env:COMPUTERNAME+'\'+$taskName),$taskSecure)
 $taskProcess=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $taskCredential -LoadUserProfile -WorkingDirectory $taskOutput -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$taskChild+'"'),'-InputFile',('"'+$taskConfig+'"')) -RedirectStandardOutput (Join-Path $taskOutput 'ordinary-stdout.log') -RedirectStandardError (Join-Path $taskOutput 'ordinary-stderr.log') -PassThru
 if(!$taskProcess.WaitForExit(300000)){$taskProcess.Kill($true);throw 'Ordinary native startup deadline expired'}
 if($taskProcess.ExitCode -ne 0){throw 'Ordinary native startup failed; see retained output'}
 $taskReport=Get-Content -LiteralPath (Join-Path $taskOutput 'startup/startup.json') -Raw|ConvertFrom-Json
 if($taskReport.result -cne 'pass' -or $taskReport.applicationToken.elevated -ne $false -or !$taskReport.applicationModeMatched){throw 'Ordinary application token evidence missing'}
 [ordered]@{result='pass';scope='temporary nonadministrator local account with genuine loaded Profile; isolated sandbox, final EXE and WebView2 business checks';accountSid=$taskUser.SID.Value;applicationElevated=$taskReport.applicationToken.elevated;applicationModeMatched=$taskReport.applicationModeMatched;dailyUserChanged=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'ordinary-summary.json') -Encoding utf8
} finally {
 if($taskUser){Remove-LocalUser -SID $taskUser.SID}
 [Array]::Clear($taskSecret,0,$taskSecret.Length)
}
