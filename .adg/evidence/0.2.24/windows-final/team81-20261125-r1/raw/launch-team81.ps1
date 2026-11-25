param([Parameter(Mandatory=$true)][ValidateSet('missing','bundled')][string]$Mode)
$ErrorActionPreference='Stop'
$root=$PSScriptRoot
$payload=Get-Content -LiteralPath (Join-Path $root 'payload-inventory.json') -Raw | ConvertFrom-Json
$inventory=$payload.inventory | Where-Object { $_.path -ceq 'opencodex-desktop.exe' }
$exe=Join-Path $payload.$(if($Mode -eq 'missing'){'missing'}else{'complete'}) 'opencodex-desktop.exe'
$expectedRoot=[IO.Path]::GetFullPath($root).TrimEnd('\')+'\'
if(-not ([IO.Path]::GetFullPath($exe)).StartsWith($expectedRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Candidate image must be inside the TEAM81 root'}
$hash=(Get-FileHash -LiteralPath $exe).Hash.ToLowerInvariant()
if($hash -cne $inventory.sha256){throw 'Candidate image digest mismatch'}
$sandbox=Join-Path $root "$Mode-sandbox"
$receipt=Join-Path $root "$Mode-launch.json"
if((Test-Path -LiteralPath $sandbox) -or (Test-Path -LiteralPath $receipt)){throw 'TEAM81 launch already has a sandbox or receipt'}
$route=@(Get-NetRoute -ErrorAction Stop | Where-Object DestinationPrefix -in @('0.0.0.0/0','::/0') | Select-Object InterfaceAlias,AddressFamily,DestinationPrefix)
$up=@(Get-NetAdapter | Where-Object Status -eq Up | Select-Object -ExpandProperty Name)
$profiles=@(Get-NetConnectionProfile | Select-Object InterfaceAlias,IPv4Connectivity,IPv6Connectivity)
$network=[ordered]@{profiles=$profiles;activeAdapters=$up;defaultRoutes=$route}
$network | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8NoBOM -LiteralPath (Join-Path $root "$Mode-network-preflight.json")
if($Mode -eq 'bundled' -and @($route | Where-Object { $_.InterfaceAlias -in $up }).Count -gt 0){throw 'Active default route present. Bundled candidate was not launched.'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$processSession=(Get-Process -Id $PID).SessionId
if($processSession -eq 0){throw 'Interactive desktop required'}
$env:OPENCODEX_SANDBOX='1'
$env:OPENCODEX_SANDBOX_ROOT=$sandbox
$env:WEBVIEW2_USER_DATA_FOLDER=Join-Path $sandbox 'EBWebView'
Remove-Item Env:HOME,Env:CODEX_HOME,Env:OPENCODEX_HOME,Env:OPENCODEX_SANDBOX_BOUNDARY,Env:OPENCODEX_WINDOWS_SMOKE_CDP_PORT,Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
$started=[DateTime]::UtcNow.ToString('o')
$proc=Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -PassThru -RedirectStandardOutput (Join-Path $root "$Mode-stdout.log") -RedirectStandardError (Join-Path $root "$Mode-stderr.log")
$observed=[ordered]@{mode=$Mode;launchedUtc=$started;account=$identity.Name;sid=$identity.User.Value;session=$proc.SessionId;pid=$proc.Id;image=$exe;imageSha256=$hash;sandboxRoot=$sandbox;sandboxAbsentBefore=$true;processCreatedUtc=$proc.StartTime.ToUniversalTime().ToString('o')}
$observed | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8NoBOM -LiteralPath $receipt
$observed | ConvertTo-Json -Depth 5

