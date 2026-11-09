param([Parameter(Mandatory=$true)][string]$ApiPath,[string]$OutputFile)
$ErrorActionPreference='Stop'
if(!$ApiPath.StartsWith('actions/')){throw 'Only repository Actions reads admitted'}
$taskText=('protocol=https'+[char]10+'host=github.com'+[char]10+[char]10) | git -C C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code credential fill
if($LASTEXITCODE -ne 0){throw 'Credential lookup failed'}
$taskMap=@{};foreach($line in $taskText){$pair=$line.Split('=',2);if($pair.Count -eq 2){$taskMap[$pair[0]]=$pair[1]}}
$taskHeaders=@{Authorization=('Bearer '+$taskMap.password);Accept='application/vnd.github+json';'User-Agent'='ocxd023-session-diagnostic'}
$taskUri='https://api.github.com/repos/gzers/opencodex-desktop/'+$ApiPath
if($OutputFile){
 if(Test-Path -LiteralPath $OutputFile){throw 'Never replace a download'}
 Invoke-WebRequest -Uri $taskUri -Proxy http://127.0.0.1:3060 -Headers $taskHeaders -HttpVersion 1.1 -OutFile $OutputFile
 Get-FileHash -LiteralPath $OutputFile | Select-Object Hash,Path
}else{
 $value=Invoke-RestMethod -Uri $taskUri -Proxy http://127.0.0.1:3060 -Headers $taskHeaders -HttpVersion 1.1
 $value | ConvertTo-Json -Depth 16
}
