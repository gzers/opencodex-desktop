# Snapshot of the read-only command submitted directly to exec_command.
# This file records that command; it is not an additional product invocation.
$batch='D:\ocxu1-20261109-194449'
$deadline=[datetime]::UtcNow.AddMinutes(16)
do{
 @{observedUtc=[datetime]::UtcNow.ToString('o');sessionInventory=@(quser)}|ConvertTo-Json -Compress -Depth 5|Add-Content -LiteralPath (Join-Path $batch 'rdp-session-samples.jsonl') -Encoding utf8
 if(Test-Path -LiteralPath (Join-Path $batch 'controller-finished.json')){break}
 Start-Sleep -Seconds 10
}while([datetime]::UtcNow -lt $deadline)
$events=@(Get-WinEvent -FilterHashtable @{LogName='Microsoft-Windows-TerminalServices-LocalSessionManager/Operational';StartTime=[datetime]'2026-11-09T19:20:00';Id=21,22,23,24,25,40} -ErrorAction SilentlyContinue|Sort-Object TimeCreated|ForEach-Object{@{id=$_.Id;recordId=$_.RecordId;timeUtc=$_.TimeCreated.ToUniversalTime().ToString('o');timeLocal=$_.TimeCreated.ToString('o');message=$_.Message}})
@{collectedUtc=[datetime]::UtcNow.ToString('o');logName='Microsoft-Windows-TerminalServices-LocalSessionManager/Operational';events=$events}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $batch 'rdp-session-events.json') -Encoding utf8
@{monitorFinishedUtc=[datetime]::UtcNow.ToString('o');controllerFinished=(Test-Path -LiteralPath (Join-Path $batch 'controller-finished.json'));samples=@(Get-Content -LiteralPath (Join-Path $batch 'rdp-session-samples.jsonl')).Count}|ConvertTo-Json
