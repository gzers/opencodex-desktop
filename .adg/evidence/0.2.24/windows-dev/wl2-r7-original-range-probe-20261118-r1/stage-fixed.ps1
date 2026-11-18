param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$EvidenceRoot,[Parameter(Mandatory)][string]$Source)
$ErrorActionPreference='Stop'
$prior='C:\Users\Public\ocx-r7-isolated-owner-nine-r2'
$public='C:\Users\Public\ocx-r7-isolated-owner-range-r1'
$profile='C:\Users\marenben'
$base=Join-Path $profile 'ocx024-r7-isolated-owner-range-r1'
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$bundleName='input/runtime-bundle-0.2.24-win32-x64.tar.gz'
$bundleSha='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'
$out=Join-Path $public 'out01'
$utf8=[Text.UTF8Encoding]::new($false)
function SaveJson([string]$Path,$Data) {
    $f=[IO.File]::Open($Path,'CreateNew','Write','Read')
    try { $b=$utf8.GetBytes(($Data|ConvertTo-Json -Depth 12)); $f.Write($b,0,$b.Length);$f.Flush($true) } finally {$f.Dispose()}
}
function Sha($Path) {(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()}
function RequireAbsent($Path) {
    try { $present=Get-Item -LiteralPath $Path -Force -ErrorAction Stop } catch [System.Management.Automation.ItemNotFoundException] { return }
    if($present){throw "Fixed root exists $Path"}
}
foreach($path in @('C:\Users','C:\Users\Public',$profile,$prior)) {
    $item=Get-Item -LiteralPath $path -Force
    if(!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw "Unexpected parent $path"}
}
RequireAbsent $public
RequireAbsent $base
$image=Get-Content -Raw (Join-Path $EvidenceRoot 'fixed-serial/image.json')|ConvertFrom-Json
if ((Sha $image.path) -cne $image.sha256 -or (Sha (Join-Path $prior $bundleName)) -cne $bundleSha){throw 'Fixed exe or official bundle hash differs'}
$fixed=Get-Content -Raw (Join-Path $EvidenceRoot 'fixed-serial/sources.json')|ConvertFrom-Json
if ((git -C $CodeRoot rev-parse HEAD) -cne $Source -or (git -C $CodeRoot rev-parse HEAD^) -cne $fixed.baseline){throw 'Fixed source commit differs'}
foreach($file in $fixed.sources){
    if ((Sha (Join-Path $CodeRoot $file.path)) -cne $file.sha256) { throw "Source drift $($file.path)" }
}
$account=Get-LocalUser -Name 'marenben'
if(!$account.Enabled -or $account.SID.Value -cne $sid){throw 'Existing ordinary account does not match'}
$session=@(quser.exe marenben)
if($LASTEXITCODE -ne 0 -or -not ($session -match '^\s*marenben\s+\S+\s+7\s+Active\s+')) {throw 'Existing desktop is absent or changed'}
$oldBundle=Get-Item -LiteralPath (Join-Path $prior $bundleName) -Force
if($oldBundle.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Fixed bundle is reparse'}
$aclSections=[Security.AccessControl.AccessControlSections]::Access
function NewStageDirectory([string]$Path,[string]$Template) {
    [void](New-Item -ItemType Directory -Path $Path -ErrorAction Stop)
    $old=Get-Acl -LiteralPath $Template
    $acl=Get-Acl -LiteralPath $Path
    $expected=$old.GetSecurityDescriptorSddlForm($aclSections)
    $acl.SetSecurityDescriptorSddlForm($expected,$aclSections)
    Set-Acl -LiteralPath $Path -AclObject $acl
    $actual=(Get-Acl -LiteralPath $Path).GetSecurityDescriptorSddlForm($aclSections)
    if($actual -cne $expected){throw "ACL did not match fixed template: $Path"}
    return $actual
}
$acl=@{
    root=NewStageDirectory $public $prior
    input=NewStageDirectory (Join-Path $public 'input') (Join-Path $prior 'input')
    output=NewStageDirectory $out (Join-Path $prior 'out01')
}
$launchers=@('launch-r7-partial-owner.cmd','launch-r7-partial-owner.ps1')
$inputs=[ordered]@{'input/fixture-test.exe'=$image.path; $bundleName=$oldBundle.FullName}
foreach($name in $launchers){$inputs[$name]=Join-Path $CodeRoot "test/windows024-native/$name"}
foreach($entry in $inputs.GetEnumerator()){
    $to=Join-Path $public $entry.Key
    [IO.File]::Copy($entry.Value,$to,$false)
    if((Sha $entry.Value) -cne (Sha $to)){throw "Staged SHA differs $to"}
}
$manifest=@{schema=1;sourceCommit=$Source;sid=$sid;session=7;testSha256=$image.sha256;bundleSha256=$bundleSha}
SaveJson (Join-Path $public 'input/manifest.json') $manifest
$files=@(foreach($file in @($inputs.Keys)+@('input/manifest.json')){
    $item=Get-Item -LiteralPath (Join-Path $public $file)
    $dacl=(Get-Acl -LiteralPath $item.FullName).GetSecurityDescriptorSddlForm($aclSections)
    if($dacl -notmatch ('\(A;ID;0x1200a9;;;'+[regex]::Escape($sid)+'\)')){throw "Staged file is not read/execute only for ordinary account $file"}
    @{path=$item.FullName;bytes=$item.Length;sha256=Sha $item.FullName;acl=$dacl}
})
RequireAbsent $base
$output=@(Get-ChildItem -LiteralPath $out -Force)
if($output.Count -ne 0){throw 'Output directory filled before admission'}
SaveJson (Join-Path $EvidenceRoot 'staging.json') @{source=$Source;publicRoot=$public;profileFixtureAbsent=$true;profileFixture=$base;outputEmpty=$true;session=$session;acl=$acl;files=$files;runtimeBundle=$bundleSha;testExe=$image.sha256}
Copy-Item -LiteralPath (Join-Path $public 'input/manifest.json') -Destination (Join-Path $EvidenceRoot 'staged-manifest.json')