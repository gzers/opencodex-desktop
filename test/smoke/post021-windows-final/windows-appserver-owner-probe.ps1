# The candidate builder prepends the same reviewed native identity type used
# by the desktop probe. Original narrow matcher regexes are passed as data.
$ErrorActionPreference='Stop'
$taskRows=@()
try {
    $taskSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $taskMe=[Security.Principal.WindowsIdentity]::GetCurrent().Name
    foreach($taskP in @(Get-CimInstance Win32_Process -ErrorAction Stop)){
        if([string]::IsNullOrWhiteSpace($taskP.CommandLine)){continue}
        if(!($taskP.CommandLine -match $taskBasenameMatch -or $taskP.CommandLine -match $taskCodeModeMatch)){continue}
        $taskNative=$null
        try{$taskNative=[OcxDesktopOwnerV1]::Read([int]$taskP.ProcessId)}catch{
            $taskCurrent=Get-CimInstance Win32_Process -Filter ('ProcessId='+$taskP.ProcessId) -ErrorAction Stop
            if(!$taskCurrent -or $taskCurrent.CreationDate -ne $taskP.CreationDate){continue}
            throw 'Still-live candidate owner identity unavailable'
        }
        if(!$taskNative){continue}
        if($taskNative.Sid -cne $taskSid){continue}
        $taskDelta=$taskNative.CreatedUtcTicks-$taskP.CreationDate.ToUniversalTime().Ticks
        if($taskDelta -lt 0 -or $taskDelta -gt 9){throw 'App-server generation changed'}
        $taskImage=$taskNative.Path
        $taskSlash=[string][char]92
        if($taskImage.StartsWith($taskSlash+$taskSlash+'?'+$taskSlash)){$taskImage=$taskImage.Substring(4)}
        if(!$taskP.ExecutablePath -or $taskImage -ine $taskP.ExecutablePath.Replace('/',$taskSlash)){throw 'App-server image changed'}
        $taskCommand=$taskP.CommandLine.Replace([string][char]9,' ')
        $taskRows += ([string]$taskP.ProcessId+[char]9+$taskCommand+[char]9+$taskMe)
    }
    $taskRows|Write-Output
}catch{Write-Output '__OCX_ENUM_INCOMPLETE__'}
