$ErrorActionPreference='Stop'
Add-Type -Path 'C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code/test/windows024-native/cleanup-r6-authorized-processes.cs'
try{[void][R6AuthorizedFailureCleanup]::PrepareExact();throw 'production helper unexpectedly admitted controller'}catch{if($_.Exception.ToString() -notlike '*exact ordinary SID1003/session7/noJob required*'){throw};Write-Output 'Production helper compiled; wrong actual context refused before target/source access.'}
