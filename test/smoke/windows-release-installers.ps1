param([Parameter(Mandatory=$true)][string]$OutputDirectory,[ValidateSet('msi','nsis')][string]$Kind,[ValidateSet('default','custom')][string]$Mode,[string]$Version='0.2.18',[Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedBuiltSha256)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop'){throw 'Restricted to project disposable Windows runner'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $taskOutput | Out-Null
$taskAssets=Join-Path $taskOutput 'assets'
New-Item -ItemType Directory -Force $taskAssets | Out-Null
if($Version -notmatch '^0[.]2[.](17|18)$'){throw 'Unsupported release acceptance version'}
$taskTag='v'+$Version
& git fetch --depth=1 origin ('refs/tags/'+$taskTag+':refs/tags/'+$taskTag)
if($LASTEXITCODE -ne 0){throw 'Target release tag not available'}
$taskCommit=(& git rev-parse ($taskTag+'^{commit}')).Trim()
if($taskCommit -notmatch '^[0-9a-f]{40}$'){throw 'Target release source commit invalid'}
$taskMsiName='OpenCodeX.Desktop_'+$Version+'_x64_en-US.msi'
$taskNsisName='OpenCodeX.Desktop_'+$Version+'_x64-setup.exe'
$taskRelease=(& gh api ('repos/gzers/opencodex-desktop/releases/tags/'+$taskTag))|ConvertFrom-Json
if($LASTEXITCODE -ne 0){throw 'Could not read target release identity'}
& gh release download $taskTag --repo gzers/opencodex-desktop --dir $taskAssets
if($LASTEXITCODE -ne 0){throw 'Could not read existing signed assets'}
$taskExpected=@{}
foreach($name in @($taskMsiName,$taskNsisName)){
 $asset=@($taskRelease.assets|Where-Object name -ceq $name)
 if($asset.Count -ne 1 -or $asset[0].digest -notmatch '^sha256:([0-9a-f]{64})$'){throw 'One immutable installer asset with server digest required'}
 $taskExpected[$name]=$Matches[1]
 if((Get-FileHash -LiteralPath (Join-Path $taskAssets $name)).Hash.ToLowerInvariant() -cne $taskExpected[$name]){throw 'Release installer digest mismatch'}
}
$taskBundle=Join-Path $taskOutput 'bundle'
foreach($taskBundleKind in @('msi','nsis')){New-Item -ItemType Directory -Force (Join-Path $taskBundle $taskBundleKind)|Out-Null}
Copy-Item -LiteralPath (Join-Path $taskAssets $taskMsiName) -Destination (Join-Path $taskBundle 'msi')
Copy-Item -LiteralPath (Join-Path $taskAssets $taskNsisName) -Destination (Join-Path $taskBundle 'nsis')
$taskPayload=Join-Path $taskOutput 'administrative-payload'
$taskMsi=Join-Path $taskAssets $taskMsiName
$taskProcess=Start-Process msiexec.exe -ArgumentList ('/a "'+$taskMsi+'" /qn TARGETDIR="'+$taskPayload+'" /L*v "'+(Join-Path $taskOutput 'administrative-extract.log')+'"') -Wait -PassThru
if($taskProcess.ExitCode -ne 0){throw 'MSI administrative payload extraction failed'}
$taskImages=@(Get-ChildItem -LiteralPath $taskPayload -Recurse -Filter opencodex-desktop.exe)
if($taskImages.Count -ne 1){throw 'One exact installed image is required'}
$taskOriginal=Join-Path $taskOutput 'original-built.exe'
# Recover the original marker, then require the first release's frozen EXE hash.
@'
const fs=require('fs'),crypto=require('crypto');
const [input,output,expected]=process.argv.slice(2),b=fs.readFileSync(input),m=Buffer.from('__TAURI_BUNDLE_TYPE_VAR_MSI');
if(!/^[0-9a-f]{64}$/.test(expected))throw Error('Original build digest required');const matches=[];
for(let i=b.indexOf(m);i>=0;i=b.indexOf(m,i+1)){
 const candidate=Buffer.from(b);Buffer.from('__TAURI_BUNDLE_TYPE_VAR_UNK').copy(candidate,i);
 if(crypto.createHash('sha256').update(candidate).digest('hex')===expected)matches.push(candidate);
}
if(matches.length!==1)throw Error('one exact hash-matching original built image required, found '+matches.length);
fs.writeFileSync(output,matches[0]);
'@ | Set-Content -LiteralPath (Join-Path $taskOutput 'recover-identity.cjs') -Encoding utf8
& node (Join-Path $taskOutput 'recover-identity.cjs') $taskImages[0].FullName $taskOriginal $ExpectedBuiltSha256
if($LASTEXITCODE -ne 0){throw 'Original release identity verification failed'}
& (Join-Path $PSScriptRoot 'windows-installer.ps1') -BundleDirectory $taskBundle -BuiltExecutable $taskOriginal -Node (Get-Command node).Source -OutputDirectory (Join-Path $taskOutput 'verification') -ExpectedCommit $taskCommit -Kind $Kind -Mode $Mode
[ordered]@{result='pass';tag=$taskTag;source=$taskCommit;kind=$Kind;mode=$Mode;existingAssetsUnchanged=$true;rebuilt=$false;originalBuiltSha256=$ExpectedBuiltSha256;installers=$taskExpected}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $taskOutput 'existing-assets-verification.json') -Encoding utf8
