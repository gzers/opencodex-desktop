$ErrorActionPreference='Stop'
$taskTestRoot=$PSScriptRoot
$taskOutput=Join-Path $taskTestRoot ('post021-preflight-'+[datetime]::UtcNow.ToString('yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N'))
& 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/formal-msix-preflight.ps1' -ExpectedUser Administrator -TestRoot $taskTestRoot -OutputDirectory $taskOutput
