function Test-GpuForegroundHeartbeat($Candidate,$Identity,[datetimeoffset]$Now){
    try{
        if(!$Candidate -or $Candidate.foreground -ne $true -or
           $Candidate.pid -ne $Identity.pid -or
           [string]$Candidate.createdAtUtcTicks -cne [string]$Identity.createdAtUtcTicks){return $false}
        $taskAt=[datetimeoffset]::Parse($Candidate.at)
        $taskSince=[datetimeoffset]::Parse($Candidate.foregroundSince)
        $taskAge=($Now-$taskAt).TotalSeconds
        return $taskAge -ge 0 -and $taskAge -lt 2 -and ($taskAt-$taskSince).TotalSeconds -ge 3
    }catch{return $false}
}
