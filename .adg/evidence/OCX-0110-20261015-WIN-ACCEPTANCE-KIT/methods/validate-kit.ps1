param([ValidateSet('initial','process-remotesigned')][string]$RunMode='initial')
$ErrorActionPreference = 'Stop'
$taskEvidence = Split-Path -Parent $PSScriptRoot
$taskRecordPath = Join-Path $taskEvidence 'kit-method-checks.original.json'
if (Test-Path -LiteralPath $taskRecordPath) { throw 'Keep existing original checks' }
$taskBuild = Get-Content -LiteralPath (Join-Path $taskEvidence 'kit-build.original.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskKit = $taskBuild.kitPath
$taskManifest = Get-Content -LiteralPath (Join-Path $taskKit 'kit-manifest.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntimeRoot = Join-Path $taskCode ('test/runtime/acceptance-kit-method-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskRuntimeRoot -ErrorAction Stop | Out-Null
$taskBefore = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if ($taskBefore.Count) { throw 'Method-only validation requires native count zero' }
$taskChecks = [Collections.Generic.List[object]]::new()
$taskPolicyBefore = @(Get-ExecutionPolicy -List | Select-Object Scope,ExecutionPolicy)
$taskPwsh = Join-Path $taskKit 'runtime/powershell/pwsh.exe'
$taskSignature = Get-AuthenticodeSignature -LiteralPath $taskPwsh
if ($taskSignature.Status -ne 'Valid' -or $taskSignature.SignerCertificate.Subject -notlike '*Microsoft Corporation*') {
    throw 'Copied runtime does not retain its Microsoft signature'
}
function Invoke-KitMethod {
    param([string]$Name,[string]$Script,[string]$ExpectedDigest,[int]$ExpectedExit,[string]$ExpectedError='')
    $taskInfo = [Diagnostics.ProcessStartInfo]::new($taskPwsh)
    $taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true;$taskInfo.WindowStyle='Hidden'
    $taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true
    $taskInfo.WorkingDirectory=Split-Path -Parent $Script
    $taskInfo.Environment['PSModuleAnalysisCachePath']=Join-Path $taskRuntimeRoot ('module-cache-' + $Name)
    $taskInfo.Environment['PSModulePath']=Join-Path $taskKit 'runtime/powershell/Modules'
    $taskInfo.Environment['POWERSHELL_TELEMETRY_OPTOUT']='1'
    foreach ($taskArgument in @('-NoLogo','-NoProfile','-NonInteractive')) {
        $taskInfo.ArgumentList.Add($taskArgument)
    }
    if ($RunMode -ceq 'process-remotesigned') {
        $taskInfo.ArgumentList.Add('-ExecutionPolicy')
        $taskInfo.ArgumentList.Add('RemoteSigned')
    }
    foreach ($taskArgument in @('-File',$Script,
        '-ExpectedProfile','C:\Users\ocx0110_accept','-ExpectedKitManifestSha256',$ExpectedDigest)) {
        $taskInfo.ArgumentList.Add($taskArgument)
    }
    $taskProcess = [Diagnostics.Process]::Start($taskInfo)
    try {
        $taskOut = $taskProcess.StandardOutput.ReadToEndAsync()
        $taskErr = $taskProcess.StandardError.ReadToEndAsync()
        if (!$taskProcess.WaitForExit(30000)) {
            $taskProcess.Kill();[void]$taskProcess.WaitForExit(5000)
            throw 'Owned method child exceeded its bound'
        }
        $taskStdout=$taskOut.GetAwaiter().GetResult();$taskStderr=$taskErr.GetAwaiter().GetResult()
        $taskStdoutPath=Join-Path $taskEvidence ($RunMode + '-' + $Name + '.stdout.original.txt')
        $taskStderrPath=Join-Path $taskEvidence ($RunMode + '-' + $Name + '.stderr.original.txt')
        if ((Test-Path -LiteralPath $taskStdoutPath) -or (Test-Path -LiteralPath $taskStderrPath)) {
            throw 'Do not overwrite a prior method attempt'
        }
        [IO.File]::WriteAllText($taskStdoutPath,$taskStdout,[Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($taskStderrPath,$taskStderr,[Text.UTF8Encoding]::new($false))
        if ($taskProcess.ExitCode -ne $ExpectedExit -or ($ExpectedError -and !$taskStderr.Contains($ExpectedError))) {
            throw "Method contract did not hold: $Name"
        }
        if ($ExpectedExit -eq 0) {
            $taskReply=$taskStdout | ConvertFrom-Json
            if ($taskReply.status -cne 'rejected' -or $taskReply.reason -cne 'daily-account' -or
                !$taskReply.kitBytesVerified -or $taskReply.productStarted -or $taskReply.accountOrProfileCreated -or
                $taskReply.registryOrAclChanged -or $taskReply.launchPermissionGranted) {
                throw 'Daily identity must not be accepted as an isolated profile'
            }
        }
        $taskChecks.Add([ordered]@{name=$Name;result='passed';exitCode=$taskProcess.ExitCode;
            ownedProcessId=$taskProcess.Id;helperOnly=$true;productStarted=$false})
    } finally { $taskProcess.Dispose() }
}
Invoke-KitMethod 'daily-profile-refusal' (Join-Path $taskKit 'methods/inspect-profile.ps1') $taskBuild.kitManifestSha256 0
Invoke-KitMethod 'wrong-manifest-refusal' (Join-Path $taskKit 'methods/inspect-profile.ps1') ('0'*64) 1 'Kit manifest does not match'
$taskFixture = Join-Path $taskRuntimeRoot 'tampered-fixture'
New-Item -ItemType Directory -Path (Join-Path $taskFixture 'methods') -ErrorAction Stop | Out-Null
Copy-Item -LiteralPath (Join-Path $taskKit 'methods/inspect-profile.ps1') -Destination (Join-Path $taskFixture 'methods/inspect-profile.ps1')
[IO.File]::WriteAllText((Join-Path $taskFixture 'inventory.bin'),'changed',[Text.UTF8Encoding]::new($false))
$taskMock = [ordered]@{
    schemaVersion=1;source=$taskBuild.source;hostDailySidSha256=$taskManifest.hostDailySidSha256
    files=@([ordered]@{path='inventory.bin';bytes=7;sha256=('0'*64)})
}
[IO.File]::WriteAllText((Join-Path $taskFixture 'kit-manifest.json'),($taskMock | ConvertTo-Json -Depth 5),[Text.UTF8Encoding]::new($false))
$taskMockHash=(Get-FileHash -LiteralPath (Join-Path $taskFixture 'kit-manifest.json')).Hash.ToLowerInvariant()
Invoke-KitMethod 'changed-payload-refusal' (Join-Path $taskFixture 'methods/inspect-profile.ps1') $taskMockHash 1 'Kit file bytes do not match'
$taskAfter = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if ($taskAfter.Count) { throw 'Unexpected native product during method checks' }
$taskPolicyAfter = @(Get-ExecutionPolicy -List | Select-Object Scope,ExecutionPolicy)
if (($taskPolicyBefore | ConvertTo-Json -Compress) -cne ($taskPolicyAfter | ConvertTo-Json -Compress)) {
    throw 'Parent execution policies unexpectedly changed'
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskBuild.source;result='passed'
    scope='Prepared kit and cloned PowerShell runtime, profile preflight only'
    checks=$taskChecks.ToArray();checksPassed=$taskChecks.Count
    runtimeVersion=$PSVersionTable.PSVersion.ToString();copiedRuntimeSignatureStatus=[string]$taskSignature.Status
    childPolicyMode=$RunMode;parentExecutionPoliciesUnchanged=$true
    parentExecutionPoliciesBefore=$taskPolicyBefore;parentExecutionPoliciesAfter=$taskPolicyAfter
    kitManifestSha256=$taskBuild.kitManifestSha256;nativeBeforeCount=$taskBefore.Count;nativeAfterCount=$taskAfter.Count
    sourceMethodSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'inspect-profile.ps1')).Hash.ToLowerInvariant()
    profilePreparedOrAdmitted=$false;accountOrProfileCreated=$false;productStarted=$false
    registryOrAclOrKnownFoldersChanged=$false;performanceMeasured=$false;fullVersionGatePassed=$false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskRecordPath -Encoding utf8
[ordered]@{result='passed';checksPassed=$taskChecks.Count;productStarted=$false;profilePreparedOrAdmitted=$false} | ConvertTo-Json
