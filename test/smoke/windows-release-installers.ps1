param([Parameter(Mandatory=$true)][string]$OutputDirectory,[ValidateSet('msi','nsis')][string]$Kind,[ValidateSet('default','custom')][string]$Mode)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop'){throw 'Restricted to project disposable Windows runner'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $taskOutput | Out-Null
$taskAssets=Join-Path $taskOutput 'assets'
New-Item -ItemType Directory -Force $taskAssets | Out-Null
& gh release download v0.2.17 --repo gzers/opencodex-desktop --dir $taskAssets
if($LASTEXITCODE -ne 0){throw 'Could not read the existing signed draft assets'}
$taskExpected=@{
 'OpenCodeX.Desktop_0.2.17_x64_en-US.msi'='2680d2e807f240f60c1447be43b524f67f6a16cba31e5c6b349733b34a056e5c';
 'OpenCodeX.Desktop_0.2.17_x64-setup.exe'='efa5254c27308c8e0b06aa0aff45ae4be674bf424bac52efcb79d9735e60526d'
}
foreach($name in $taskExpected.Keys){
 if((Get-FileHash -LiteralPath (Join-Path $taskAssets $name)).Hash.ToLowerInvariant() -cne $taskExpected[$name]){throw 'Existing installer bytes changed; do not rebuild or overwrite the tag'}
}
$taskBundle=Join-Path $taskOutput 'bundle'
foreach($taskBundleKind in @('msi','nsis')){New-Item -ItemType Directory -Force (Join-Path $taskBundle $taskBundleKind)|Out-Null}
Copy-Item -LiteralPath (Join-Path $taskAssets 'OpenCodeX.Desktop_0.2.17_x64_en-US.msi') -Destination (Join-Path $taskBundle 'msi')
Copy-Item -LiteralPath (Join-Path $taskAssets 'OpenCodeX.Desktop_0.2.17_x64-setup.exe') -Destination (Join-Path $taskBundle 'nsis')
$taskPayload=Join-Path $taskOutput 'administrative-payload'
$taskMsi=Join-Path $taskAssets 'OpenCodeX.Desktop_0.2.17_x64_en-US.msi'
$taskProcess=Start-Process msiexec.exe -ArgumentList ('/a "'+$taskMsi+'" /qn TARGETDIR="'+$taskPayload+'" /L*v "'+(Join-Path $taskOutput 'administrative-extract.log')+'"') -Wait -PassThru
if($taskProcess.ExitCode -ne 0){throw 'MSI administrative payload extraction failed'}
$taskImages=@(Get-ChildItem -LiteralPath $taskPayload -Recurse -Filter opencodex-desktop.exe)
if($taskImages.Count -ne 1){throw 'One exact installed image is required'}
$taskOriginal=Join-Path $taskOutput 'original-built.exe'
# Recover the original marker, then require the first release's frozen EXE hash.
@'
const fs=require('fs'),crypto=require('crypto');
const [input,output]=process.argv.slice(2),b=fs.readFileSync(input),m=Buffer.from('__TAURI_BUNDLE_TYPE_VAR_MSI');
const expected='610fb4ccbc3e2984d5a8a311aac90d1a498089deaeb3208ece9adc39f5365776',matches=[];
for(let i=b.indexOf(m);i>=0;i=b.indexOf(m,i+1)){
 const candidate=Buffer.from(b);Buffer.from('__TAURI_BUNDLE_TYPE_VAR_UNK').copy(candidate,i);
 if(crypto.createHash('sha256').update(candidate).digest('hex')===expected)matches.push(candidate);
}
if(matches.length!==1)throw Error('one exact hash-matching original built image required, found '+matches.length);
fs.writeFileSync(output,matches[0]);
'@ | Set-Content -LiteralPath (Join-Path $taskOutput 'recover-identity.cjs') -Encoding utf8
& node (Join-Path $taskOutput 'recover-identity.cjs') $taskImages[0].FullName $taskOriginal
if($LASTEXITCODE -ne 0){throw 'Original release identity verification failed'}
& (Join-Path $PSScriptRoot 'windows-installer.ps1') -BundleDirectory $taskBundle -BuiltExecutable $taskOriginal -Node (Get-Command node).Source -OutputDirectory (Join-Path $taskOutput 'verification') -ExpectedCommit d7c7978f043278baae8f2fc8c2119315a4660f12 -Kind $Kind -Mode $Mode
[ordered]@{result='pass';tag='v0.2.17';source='d7c7978f043278baae8f2fc8c2119315a4660f12';originalReleaseRun=40636062852;kind=$Kind;mode=$Mode;existingAssetsUnchanged=$true;rebuilt=$false;originalBuiltSha256='610fb4ccbc3e2984d5a8a311aac90d1a498089deaeb3208ece9adc39f5365776';installers=$taskExpected} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskOutput 'existing-assets-verification.json') -Encoding utf8
