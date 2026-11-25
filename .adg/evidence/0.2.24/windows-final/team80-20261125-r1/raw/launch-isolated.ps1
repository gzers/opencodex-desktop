param([ValidateSet('bundled','missing')][string]$Mode)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$sandbox=Join-Path $PSScriptRoot "$Mode-sandbox"
if(Test-Path -LiteralPath $sandbox){throw 'Sandbox already exists'}
$exe=Join-Path $PSScriptRoot "$Mode-image/opencodex-desktop.exe"
$env:OPENCODEX_SANDBOX='1'
$env:OPENCODEX_SANDBOX_ROOT=$sandbox
$env:WEBVIEW2_USER_DATA_FOLDER=Join-Path $sandbox 'EBWebView'
Remove-Item Env:HOME,Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS,Env:OPENCODEX_SANDBOX_BOUNDARY,Env:OPENCODEX_WINDOWS_SMOKE_CDP_PORT -ErrorAction SilentlyContinue
$started=[DateTime]::UtcNow.ToString('o')
$proc=Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -PassThru -RedirectStandardOutput (Join-Path $PSScriptRoot "$Mode-stdout.log") -RedirectStandardError (Join-Path $PSScriptRoot "$Mode-stderr.log")
$receipt=[ordered]@{ startedUtc=$started; environment='network attached'; sandboxEnabled=$true; sandboxAbsentBefore=$true; sandboxRoot=$sandbox; image=$exe; pid=$proc.Id; session=$proc.SessionId }
try {
 $receipt.token=Get-WindowsProcessToken -Process $proc -ExpectedExecutable $exe
 $owner=Invoke-CimMethod -InputObject (Get-CimInstance Win32_Process -Filter "ProcessId = $($proc.Id)") -MethodName GetOwnerSid
 $receipt.ownerSid=$owner.Sid
} catch { $receipt.firstError=$_.Exception.Message }
$receipt | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 -LiteralPath (Join-Path $PSScriptRoot "$Mode-launch.json")
$receipt | ConvertTo-Json -Depth 5
