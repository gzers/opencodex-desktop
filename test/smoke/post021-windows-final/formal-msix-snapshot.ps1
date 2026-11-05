param([string]$ExpectedUser='Administrator')
$ErrorActionPreference='Stop'
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
if($taskIdentity.Name.Split('\')[-1] -ine $ExpectedUser){throw 'Formal validation requires its exact Windows test owner'}
$taskSelf=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$taskOwn=@();$taskOther=@();$taskUnknown=0
foreach($taskP in @(Get-CimInstance Win32_Process -Filter "Name='ChatGPT.exe' OR Name='codex.exe' OR Name='opencodex-desktop.exe'")){
    $taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
    if($taskOwner.ReturnValue -ne 0){$taskUnknown++;continue}
    $taskName=$taskOwner.Domain+'\'+$taskOwner.User
    $taskRow=[ordered]@{pid=$taskP.ProcessId;parentPid=$taskP.ParentProcessId;path=$taskP.ExecutablePath;createdAtUtcTicks=[string]$taskP.CreationDate.ToUniversalTime().Ticks;sessionId=$taskP.SessionId;owner=$taskName}
    if($taskName -ieq $taskIdentity.Name){$taskOwn+=$taskRow}else{$taskOther+=$taskRow}
}
[ordered]@{owner=$taskIdentity.Name;sid=$taskIdentity.User.Value;sessionId=$taskSelf.SessionId;own=$taskOwn;other=$taskOther;unknownOwnerCount=$taskUnknown;atUtc=[datetimeoffset]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 5
