param([ValidateSet('dispatch','runs')][string]$Mode='runs')
$ErrorActionPreference='Stop'
$taskCredentialText=('protocol=https'+[char]10+'host=github.com'+[char]10+[char]10) | git -C C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code credential fill
if($LASTEXITCODE -ne 0){throw 'Git credential unavailable'}
$taskCredential=@{}
foreach($line in $taskCredentialText){$parts=$line.Split('=',2);if($parts.Count -eq 2){$taskCredential[$parts[0]]=$parts[1]}}
$taskHeaders=@{Authorization=('Bearer '+$taskCredential.password);Accept='application/vnd.github+json';'User-Agent'='ocxd023-local-session-diagnostic'}
$taskUrl='https://api.github.com/repos/gzers/opencodex-desktop/actions/workflows/windows-candidate.yml/'
if($Mode -eq 'dispatch'){
 Invoke-RestMethod -Uri ($taskUrl+'dispatches') -Proxy http://127.0.0.1:3060 -HttpVersion 1.1 -Method Post -Headers $taskHeaders -ContentType 'application/json' -Body '{"ref":"codex/0.2.23-integration","inputs":{"purpose":"local-session-build"}}' | Out-Null
 @{dispatched=$true;workflow='windows023-local-session-build.yml';ref='codex/0.2.23-integration'} | ConvertTo-Json
}else{
 $taskResult=Invoke-RestMethod -Uri ($taskUrl+'runs?per_page=5') -Proxy http://127.0.0.1:3060 -Headers $taskHeaders
 @($taskResult.workflow_runs | Select-Object id,head_sha,status,conclusion,html_url) | ConvertTo-Json -Depth 4 | Tee-Object -FilePath (Join-Path $PSScriptRoot 'build-runs.json')
}
