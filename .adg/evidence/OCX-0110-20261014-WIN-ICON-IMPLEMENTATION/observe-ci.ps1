param()
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = 'bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a'
$taskRoot = $PSScriptRoot
$taskPreviousPrompt = $env:GIT_TERMINAL_PROMPT
$taskPreviousInteractive = $env:GCM_INTERACTIVE
try {
    $env:GIT_TERMINAL_PROMPT = '0'
    $env:GCM_INTERACTIVE = 'never'
    $taskCredential = "protocol=https`nhost=github.com`n`n" | git -C $taskCode credential fill
    if ($LASTEXITCODE -ne 0) { throw 'Git credential helper unavailable' }
    $taskPassword = @($taskCredential | Where-Object { $_.StartsWith('password=') })
    if ($taskPassword.Count -ne 1) { throw 'Credential not returned' }
    $taskHeaders = @{
        Authorization = 'Bearer ' + $taskPassword[0].Substring(9)
        Accept = 'application/vnd.github+json'
        'X-GitHub-Api-Version' = '2022-11-28'
    }
    function Read-Api([string]$Path) {
        Invoke-RestMethod -Uri ('https://api.github.com/repos/gzers/opencodex-desktop/' + $Path) `
            -Headers $taskHeaders -TimeoutSec 25 -UserAgent 'ocx-windows-icon-ci-system-stack'
    }
    $taskRuns = Read-Api "actions/runs?head_sha=$taskSource&per_page=30"
    $taskRows = @()
    $taskDetails = @()
    foreach ($taskRun in $taskRuns.workflow_runs) {
        if ($taskRun.head_sha -cne $taskSource) { throw 'Run source mismatch' }
        $taskJobs = Read-Api "actions/runs/$($taskRun.id)/jobs?per_page=100"
        $taskArtifacts = Read-Api "actions/runs/$($taskRun.id)/artifacts?per_page=100"
        $taskDetails += [ordered]@{run=$taskRun;jobs=$taskJobs;artifacts=$taskArtifacts}
        $taskRows += [ordered]@{
            id=$taskRun.id;name=$taskRun.name;event=$taskRun.event
            status=$taskRun.status;conclusion=$taskRun.conclusion
            jobs=@($taskJobs.jobs | Select-Object name,status,conclusion)
        }
    }
    $taskOut = Join-Path $taskRoot ('ci-observation-system-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '.json')
    [ordered]@{atUtc=[DateTime]::UtcNow.ToString('o');source=$taskSource;runs=$taskDetails;mutationsIssued=$false} |
        ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $taskOut -Encoding utf8
    ConvertTo-Json -InputObject $taskRows -Depth 5 -Compress
} finally {
    $env:GIT_TERMINAL_PROMPT = $taskPreviousPrompt
    $env:GCM_INTERACTIVE = $taskPreviousInteractive
}
