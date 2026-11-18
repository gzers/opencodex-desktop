param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$EvidenceRoot,[Parameter(Mandatory)][string]$BuildEvidence)
$ErrorActionPreference='Stop'
$Source='36b70383eac5c44dffc51d34a4c4673cd8a3dbf7'
$verifierParent='673d1166334c4fb78d5094bc09ef130e578e0991'
$baseline='f3564fbb34713beee97ed221fb4797b422b11d57'
$testSha='1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8'
$verifier='test/windows024-native/verify-r7-host-receipts.py'
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
function GitLine([string[]]$Arguments) {
    $lines=@(git -C $CodeRoot @Arguments)
    if ($LASTEXITCODE -ne 0 -or $lines.Count -ne 1) { throw "Unexpected Git result: $($Arguments -join ' ')" }
    return $lines[0]
}
function BindGitBlob([string]$Path) {
    $expected=GitLine @('rev-parse',('{0}:{1}' -f $Source,$Path))
    $actual=GitLine @('hash-object',('--path={0}' -f $Path),(Join-Path $CodeRoot $Path))
    if ($expected -cne $actual) { throw "Committed input differs: $Path" }
    return $actual
}
$image=Get-Content -LiteralPath (Join-Path $BuildEvidence 'image.json') -Raw | ConvertFrom-Json
$fixed=Get-Content -LiteralPath (Join-Path $BuildEvidence 'sources.json') -Raw | ConvertFrom-Json
$exe=Get-Item -LiteralPath $image.path -Force
if ($exe.PSIsContainer -or ($exe.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
    $exe.Length -ne 104751616 -or $exe.Length -ne $image.bytes -or $image.sha256 -cne $testSha -or
    (Sha $exe.FullName) -cne $testSha -or (Sha (Join-Path $prior $bundleName)) -cne $bundleSha) { throw 'Fixed EXE or official bundle differs' }
if ((GitLine @('rev-parse','HEAD')) -cne $Source -or
    (GitLine @('rev-parse',($Source+'^'))) -cne $verifierParent -or
    (GitLine @('rev-parse',($verifierParent+'^'))) -cne $baseline -or
    $fixed.baseline -cne $baseline -or
    (GitLine @('diff','--name-only',$baseline,$verifierParent,'--')) -cne $verifier) { throw 'Fixed Git lineage differs' }
$paths=@($fixed.sources | ForEach-Object {$_.path} | Sort-Object)
$commitPaths=@(git -C $CodeRoot diff --name-only $verifierParent $Source -- | Sort-Object)
if ($LASTEXITCODE -ne 0 -or $paths.Count -ne 7 -or ($paths -join '|') -cne ($commitPaths -join '|')) { throw 'Seven recorded build inputs differ from commit paths' }
$binding=@(foreach($file in $fixed.sources) {
    $fileItem=Get-Item -LiteralPath (Join-Path $CodeRoot $file.path) -Force
    if ($fileItem.PSIsContainer -or ($fileItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        $fileItem.Length -ne $file.bytes -or (Sha $fileItem.FullName) -cne $file.sha256) { throw "Raw build input drift: $($file.path)" }
    @{path=$file.path;bytes=$file.bytes;sha256=$file.sha256;gitBlob=BindGitBlob $file.path}
})
$standalone=@(foreach($path in @('test/windows024-native/launch-r7-partial-owner.cmd',$verifier)) {
    $item=Get-Item -LiteralPath (Join-Path $CodeRoot $path) -Force
    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Unexpected controller input: $path" }
    @{path=$path;bytes=$item.Length;sha256=Sha $item.FullName;gitBlob=BindGitBlob $path}
})
$account=Get-LocalUser -Name 'marenben'
if(!$account.Enabled -or $account.SID.Value -cne $sid){throw 'Existing ordinary account does not match'}
$sessionInfo=[Diagnostics.ProcessStartInfo]::new('C:\Windows\System32\quser.exe','marenben')
$sessionInfo.UseShellExecute=$false
$sessionInfo.CreateNoWindow=$true
$sessionInfo.RedirectStandardOutput=$true
$sessionInfo.RedirectStandardError=$true
[Text.Encoding]::RegisterProvider([Text.CodePagesEncodingProvider]::Instance)
$sessionInfo.StandardOutputEncoding=[Text.Encoding]::GetEncoding([Globalization.CultureInfo]::InstalledUICulture.TextInfo.OEMCodePage)
$sessionInfo.StandardErrorEncoding=$sessionInfo.StandardOutputEncoding
$query=[Diagnostics.Process]::Start($sessionInfo)
try {
    $lines=$query.StandardOutput.ReadToEndAsync()
    $err=$query.StandardError.ReadToEndAsync()
    $query.WaitForExit()
    $session=$lines.GetAwaiter().GetResult()
    $sessionError=$err.GetAwaiter().GetResult()
    $sessionRaw=$query.ExitCode
} finally { $query.Dispose() }
SaveJson (Join-Path $EvidenceRoot 'session-query.json') @{raw=$sessionRaw;stdout=$session;stderr=$sessionError;codePage=$sessionInfo.StandardOutputEncoding.CodePage}
$active=@($session -split '\r?\n' | Where-Object {$_ -cmatch '^\s*marenben\s+\S+\s+7\s+(?:Active|\u8fd0\u884c\u4e2d)\s+'})
if ($sessionRaw -ne 0 -or $sessionError.Length -ne 0 -or $active.Count -ne 1) { throw 'Existing desktop is absent or changed' }
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
$inputs["input/verify-r7-host-receipts.py"]=Join-Path $CodeRoot $verifier
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
$expectedMembers=@($inputs.Keys)+@('input','out01','input/manifest.json')
$members=@(Get-ChildItem -LiteralPath @($public,(Join-Path $public 'input'),$out) -Force | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Unexpected staged reparse point: $($_.FullName)" }
    $_.FullName.Substring($public.Length+1).Replace('\','/')
})
if ((($expectedMembers | Sort-Object) -join '|') -cne (($members | Sort-Object) -join '|')) { throw 'Unexpected staged membership' }
foreach ($file in @($binding | Where-Object {$_.path -ceq 'test/windows024-native/launch-r7-partial-owner.ps1'})+@($standalone)) {
    $leaf=Split-Path -Leaf $file.path
    $staged=@($files | Where-Object { (Split-Path -Leaf $_.path) -ceq $leaf })
    if ($staged.Count -ne 1 -or $staged[0].bytes -ne $file.bytes -or $staged[0].sha256 -cne $file.sha256) { throw "Staged source binding mismatch: $leaf" }
}
$stagedExe=Join-Path $public 'input/fixture-test.exe'
$stagedBundle=Join-Path $public $bundleName
if ((Sha $stagedExe) -cne $testSha -or (Sha $stagedBundle) -cne $bundleSha) { throw 'Final EXE or bundle SHA differs' }
SaveJson (Join-Path $EvidenceRoot 'source-binding.json') @{source=$Source;verifierParent=$verifierParent;baseline=$baseline;builtInputs=$binding;standaloneInputs=$standalone}
SaveJson (Join-Path $EvidenceRoot 'staging.json') @{source=$Source;publicRoot=$public;profileFixtureAbsent=$true;profileFixture=$base;outputEmpty=$true;members=$members;session=$session;acl=$acl;files=$files;runtimeBundle=$bundleSha;testExe=$image.sha256}
Copy-Item -LiteralPath (Join-Path $public 'input/manifest.json') -Destination (Join-Path $EvidenceRoot 'staged-manifest.json')