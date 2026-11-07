# One kind per fresh GitHub-hosted job; no /D or INSTALLDIR overrides.
# Run only after windows023-retention-ci-admit.py. -Execute is deliberately explicit.
param([Parameter(Mandatory=$true)][string]$AdmissionPath,[switch]$Execute)
$ErrorActionPreference='Stop'

function Test-RetentionSamePath([string]$Left,[string]$Right) {
    # Rust canonicalize returns extended local-drive paths. Normalize only for
    # comparison; keep the actual native paths unchanged in execution/evidence.
    if($Left.StartsWith('\\?\') -and $Left.Substring(4) -match '^[a-zA-Z]:[\\/]'){$Left=$Left.Substring(4)}
    if($Right.StartsWith('\\?\') -and $Right.Substring(4) -match '^[a-zA-Z]:[\\/]'){$Right=$Right.Substring(4)}
    [IO.Path]::GetFullPath($Left).TrimEnd([IO.Path]::DirectorySeparatorChar).Equals(
        [IO.Path]::GetFullPath($Right).TrimEnd([IO.Path]::DirectorySeparatorChar),[StringComparison]::OrdinalIgnoreCase)
}
function Assert-RetentionPlainPath([string]$Path) {
    $cursor=[IO.Path]::GetFullPath($Path)
    while($cursor){
        if(Test-Path -LiteralPath $cursor){
            if((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse path refused'}
        }
        $cursor=[IO.Path]::GetDirectoryName($cursor)
    }
}
function Get-RetentionTree([string]$Root) {
    Assert-RetentionPlainPath $Root
    if(!(Test-Path -LiteralPath $Root -PathType Container)){throw 'Retained directory missing'}
    $rows=[Collections.Generic.List[object]]::new()
    $pending=[Collections.Generic.Stack[string]]::new();$pending.Push($Root)
    while($pending.Count){
        foreach($item in Get-ChildItem -LiteralPath $pending.Pop() -Force){
            if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Fixture link refused'}
            $relative=[IO.Path]::GetRelativePath($Root,$item.FullName)
            if($item.PSIsContainer){$rows.Add([ordered]@{path=$relative;directory=$true});$pending.Push($item.FullName)}else{
                $rows.Add([ordered]@{path=$relative;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName).Hash.ToLowerInvariant()})
            }
            if($rows.Count -gt 20000){throw 'Fixture inventory exceeded bound'}
        }
    }
    ConvertTo-Json -InputObject @($rows|Sort-Object path) -Depth 5 -Compress
}
function Assert-RetentionTree([string]$Root,[string]$Expected) {
    if((Get-RetentionTree $Root) -cne $Expected){throw 'Retained tree changed (paths, directories, bytes or hashes)'}
}
function Write-RetentionJson([string]$Path,$Value) {
    $Value|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $Path -Encoding utf8
}
function Get-RetentionRegistrations {
    $found=@()
    foreach($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)){
        foreach($hive in @([Microsoft.Win32.RegistryHive]::CurrentUser,[Microsoft.Win32.RegistryHive]::LocalMachine)){
            $base=[Microsoft.Win32.RegistryKey]::OpenBaseKey($hive,$view)
            try{
                $key=$base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Uninstall')
                if($key){try{foreach($name in $key.GetSubKeyNames()){
                    $entry=$key.OpenSubKey($name)
                    try{if($entry.GetValue('DisplayName') -ceq 'OpenCodeX Desktop'){
                        $found += [ordered]@{hive="$hive";view="$view";key=$name;version=$entry.GetValue('DisplayVersion');
                            location=$entry.GetValue('InstallLocation');icon=$entry.GetValue('DisplayIcon');uninstall=$entry.GetValue('UninstallString')}
                    }}finally{$entry.Dispose()}
                }}finally{$key.Dispose()}}
                # Do not clear remembered paths: a contaminated runner must stop.
                $remembered=$base.OpenSubKey('Software\gzers\OpenCodeX Desktop')
                if($remembered){try{
                    if(!$script:taskAttemptStarted -and ($null -ne $remembered.GetValue('InstallDir') -or $null -ne $remembered.GetValue(''))){throw 'Remembered product installation exists; fresh runner required'}
                }finally{$remembered.Dispose()}}
            }finally{$base.Dispose()}
        }
    }
    # HKCU\Software uninstall keys are shared across WOW64 views. Do not count
    # the same physical per-user registration twice; HKLM views stay distinct.
    foreach($group in @($found|Group-Object { $_.hive+'|'+$_.key+$(if($_.hive -ceq 'LocalMachine'){'|'+$_.view}) })){
        $first=$group.Group[0]
        foreach($entry in $group.Group){foreach($field in @('version','location','icon','uninstall')){
            if($entry[$field] -cne $first[$field]){throw 'Conflicting registry views for one product registration'}
        }}
        $first.views=@($group.Group|ForEach-Object {$_.view})
        $first
    }
}
function Get-RetentionExpectedRoot([string]$Kind,[string]$Version,[string]$Local,[string]$MachinePrograms) {
    if($Kind -cnotin @('msi','nsis')){throw 'Unsupported installer kind'}
    if($Version -ceq '0.2.23'){return Join-Path $Local 'Programs/OpenCodeX Desktop'}
    if($Version -ceq '0.2.22'){
        if($Kind -ceq 'nsis'){return Join-Path $Local 'OpenCodeX Desktop'}
        return Join-Path $MachinePrograms 'OpenCodeX Desktop'
    }
    throw 'Unsupported installer version'
}
function Get-RetentionContextBlock([string]$Kind,[string]$Scenario,$Registration) {
    if($Kind -ceq 'msi' -and $Scenario -cin @('adjacent','explicit-conversion')){
        if($Registration.hive -cne 'LocalMachine' -or $Registration.key -ine '{C5476FDD-CB86-46AF-A97F-96DD56A32CA0}'){
            throw 'Official baseline MSI machine registration differs'
        }
        return [ordered]@{result='blocked-installation-context';baselineContext='per-machine';candidateContext='per-user';
            baselineProductCode=$Registration.key;automaticReplacementSupported=$false;candidateRejectionVerified=$false;
            reason='Windows Installer major upgrades cannot cross installation context';
            source='https://learn.microsoft.com/en-us/windows/win32/msi/major-upgrades';
            reviewableAlternative='Explicitly uninstall 022 retaining all data, then separately install 023 per-user; not an automatic upgrade';
            oldDataMigrationRequested=$false}
    }
    return $null
}
function Read-RetentionMsiProperties([string]$Package) {
    # Read the digest-pinned MSI database, never execute it or activate its app.
    $installer=New-Object -ComObject WindowsInstaller.Installer
    $database=$null;$view=$null;$record=$null
    try{
        $database=$installer.OpenDatabase($Package,0)
        $view=$database.OpenView('SELECT `Property`, `Value` FROM `Property`');$view.Execute()
        $properties=@{}
        while($record=$view.Fetch()){
            $properties[[string]$record.StringData(1)]=[string]$record.StringData(2)
            $null=[Runtime.InteropServices.Marshal]::FinalReleaseComObject($record);$record=$null
        }
        @{productCode=$properties.ProductCode;version=$properties.ProductVersion;allUsers=$properties.ALLUSERS;upgradeCode=$properties.UpgradeCode}
    }finally{
        if($view){$view.Close()}
        foreach($owned in @($record,$view,$database,$installer)){if($owned){$null=[Runtime.InteropServices.Marshal]::FinalReleaseComObject($owned)}}
    }
}
function Find-RetentionInstall([string]$Version) {
    $registrations=@(Get-RetentionRegistrations)
    if($registrations.Count -ne 1 -or $registrations[0].version -cne $Version){throw 'Expected one exact-version actual installer registration'}
    $entry=$registrations[0];$paths=@()
    if($entry.location){$paths+=([string]$entry.location).Trim('"')}
    if($entry.icon -match '^"?([^"\r\n]+?\.exe)'){$paths+=Split-Path -Parent $Matches[1]}
    if($entry.uninstall -match '^"([^"\r\n]+uninstall\.exe)"'){$paths+=Split-Path -Parent $Matches[1]}
    $roots=@($paths|ForEach-Object {[IO.Path]::GetFullPath($_).TrimEnd('\')}|Sort-Object -Unique|Where-Object {
        Test-Path -LiteralPath (Join-Path $_ 'opencodex-desktop.exe') -PathType Leaf
    })
    if($roots.Count -ne 1){throw 'Actual default installation directory is ambiguous'}
    $root=$roots[0];Assert-RetentionPlainPath $root
    # Verify the two installer defaults independently. A changed template fails
    # visibly; a common forced TEMP directory cannot satisfy this acceptance.
    $expected=Get-RetentionExpectedRoot $taskAdmission.kind $Version $taskLocal $env:ProgramFiles
    $expectedHive=if($Version -ceq '0.2.22' -and $taskAdmission.kind -ceq 'msi'){'LocalMachine'}else{'CurrentUser'}
    if($entry.hive -cne $expectedHive){throw 'Actual installer context differs from the version contract'}
    if(!(Test-RetentionSamePath $root $expected)){throw "Actual $($taskAdmission.kind) default root differs from its reviewed template: $root"}
    $exe=Join-Path $root 'opencodex-desktop.exe'
    $item=Get-Item -LiteralPath $exe
    if($item.VersionInfo.ProductVersion -cne $Version){throw 'Installed PE product version differs'}
    [pscustomobject]@{root=$root;exe=$exe;registration=$entry;version=$item.VersionInfo.ProductVersion;sha256=(Get-FileHash -LiteralPath $exe).Hash.ToLowerInvariant()}
}
function Invoke-RetentionInstaller([string]$File,[string]$Arguments,[string]$Phase,[int[]]$ExpectedExitCodes=@(0)) {
    $step=[ordered]@{name=$Phase;startedUtc=[datetime]::UtcNow.ToString('o');exitCode=$null;timedOut=$false}
    $taskResult.steps+=,$step
    Write-RetentionJson (Join-Path $taskOutput 'execution-attempt.json') $taskResult
    $p=Start-Process -FilePath $File -ArgumentList $Arguments -PassThru
    $step.pid=$p.Id;$step.file=$File;$step.arguments=$Arguments;$step.expectedExitCodes=$ExpectedExitCodes
    try{
        if(!$p.WaitForExit(180000)){$step.timedOut=$true;throw 'Installer timeout; preserve state and do not retry'}
        $step.exitCode=$p.ExitCode
        if($p.ExitCode -notin $ExpectedExitCodes){throw "Installer failed or requested reboot: $($p.ExitCode)"}
    }finally{$step.finishedUtc=[datetime]::UtcNow.ToString('o');$p.Dispose()}
}
function Install-RetentionPackage([string]$Package,[string]$Phase) {
    $script:taskMayBeInstalled=$true
    if($taskAdmission.kind -ceq 'msi'){
        Invoke-RetentionInstaller "$env:SystemRoot\System32\msiexec.exe" ('/i "'+$Package+'" /qn /norestart /L*v "'+(Join-Path $taskOutput ($Phase+'.log'))+'"') $Phase
    }else{Invoke-RetentionInstaller $Package '/S' $Phase}
}
function Uninstall-RetentionPackage($Installation,[string]$Phase) {
    if($taskAdmission.kind -ceq 'msi'){
        if($Installation.registration.key -notmatch '^\{[0-9A-Fa-f-]{36}\}$'){throw 'Actual MSI product code missing'}
        Invoke-RetentionInstaller "$env:SystemRoot\System32\msiexec.exe" ('/x '+$Installation.registration.key+' /qn /norestart /L*v "'+(Join-Path $taskOutput ($Phase+'.log'))+'"') $Phase
    }else{
        $uninstaller=Join-Path $Installation.root 'uninstall.exe';Assert-RetentionPlainPath $uninstaller
        Invoke-RetentionInstaller $uninstaller '/S' $Phase
    }
    $deadline=[datetime]::UtcNow.AddSeconds(30)
    while((Test-Path -LiteralPath $Installation.exe) -and [datetime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 200}
    if((Test-Path -LiteralPath $Installation.exe) -or @(Get-RetentionRegistrations).Count){throw 'Default uninstall left executable or product registration'}
    $script:taskMayBeInstalled=$false
}
function Assert-RetentionCandidateMsiBlock($Baseline) {
    $beforeImage=Get-RetentionTree $Baseline.root
    $beforeUser=Get-RetentionTree $taskUser
    Write-RetentionJson (Join-Path $taskOutput 'before-msi-block-old-program.json') ($beforeImage|ConvertFrom-Json)
    Write-RetentionJson (Join-Path $taskOutput 'before-msi-block-U.json') ($beforeUser|ConvertFrom-Json)
    $log=Join-Path $taskOutput 'candidate-machine-context-block.log'
    Invoke-RetentionInstaller "$env:SystemRoot\System32\msiexec.exe" ('/i "'+$taskAdmission.candidatePackage+'" /qn /norestart /L*v "'+$log+'"') 'candidate-must-block-machine022' @(1603)
    if(!(Get-Content -LiteralPath $log -Raw).Contains('OCX023_MACHINE022_BLOCK:')){throw 'MSI failed without the specific product context-block message'}
    Assert-RetentionTree $Baseline.root $beforeImage
    Assert-RetentionTree $taskUser $beforeUser
    Assert-RetentionSeeds 'after-candidate-msi-block'
    $stillInstalled=Find-RetentionInstall '0.2.22'
    if($stillInstalled.sha256 -cne $Baseline.sha256){throw 'Rejected MSI changed the official old executable'}
    if(Test-Path -LiteralPath (Get-RetentionExpectedRoot 'msi' '0.2.23' $taskLocal $env:ProgramFiles)){
        throw 'Rejected MSI wrote the new program directory'
    }
    Write-RetentionJson (Join-Path $taskOutput 'candidate-machine-context-block.json') @{
        result='pass-expected-product-rejection';candidateExecuted=$true;expectedExitCode=1603;
        productMessage='OCX023_MACHINE022_BLOCK:';oldProgramTreeUnchanged=$true;allUserAndSeedHashesUnchanged=$true;
        automaticUpgradePassed=$false;legacyMigration=$false
    }
}
function New-RetentionSeed([string]$Path,[string]$Value) {
    Assert-RetentionPlainPath $Path
    if(Test-Path -LiteralPath $Path){throw 'Never overwrite an existing seed'}
    $null=New-Item -ItemType Directory -Path (Split-Path -Parent $Path) -Force
    [IO.File]::WriteAllText($Path,$Value,[Text.UTF8Encoding]::new($false))
    $script:taskSeeds += [ordered]@{path=$Path;bytes=(Get-Item -LiteralPath $Path).Length;sha256=(Get-FileHash -LiteralPath $Path).Hash.ToLowerInvariant()}
}
function Set-RetentionSyntheticOwner {
    # Elevated hosted runners otherwise create Administrators-owned fixtures.
    # Only our preflight-proven absent, newly seeded U is eligible here.
    $null=Get-RetentionTree $taskUser
    $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
    $system=[Security.Principal.SecurityIdentifier]::new('S-1-5-18')
    $items=@((Get-Item -LiteralPath $taskUser -Force))+@(Get-ChildItem -LiteralPath $taskUser -Recurse -Force)
    foreach($item in $items){
        $acl=if($item.PSIsContainer){[Security.AccessControl.DirectorySecurity]::new()}else{[Security.AccessControl.FileSecurity]::new()}
        $acl.SetOwner($sid);$acl.SetAccessRuleProtection($true,$false)
        $inherit=if($item.PSIsContainer){[Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit'}else{[Security.AccessControl.InheritanceFlags]::None}
        foreach($principal in @($sid,$system)){
            $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($principal,[Security.AccessControl.FileSystemRights]::FullControl,
                $inherit,[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
        }
        Set-Acl -LiteralPath $item.FullName -AclObject $acl
        if((Get-Acl -LiteralPath $item.FullName).GetOwner([Security.Principal.SecurityIdentifier]).Value -cne $sid.Value){throw 'Synthetic U/H owner assignment failed'}
    }
    $taskResult.syntheticOwnerSid=$sid.Value
}
function Assert-RetentionNoMigration([string]$Phase) {
    $inventory=Get-RetentionTree $taskUser|ConvertFrom-Json
    $oldHashes=@($taskSeeds|Where-Object {!($_.path.Replace('\','/')).StartsWith($taskUser.Replace('\','/').TrimEnd('/')+'/',[StringComparison]::OrdinalIgnoreCase)}|ForEach-Object {$_.sha256})
    if(@($inventory|Where-Object {$_.sha256 -and $_.sha256 -cin $oldHashes}).Count){throw 'Old synthetic data appeared in U/H'}
    Write-RetentionJson (Join-Path $taskOutput ($Phase+'-U-inventory.json')) $inventory
}
function Assert-RetentionSeeds([string]$Phase) {
    foreach($seed in $taskSeeds){
        Assert-RetentionPlainPath $seed.path
        if(!(Test-Path -LiteralPath $seed.path -PathType Leaf) -or (Get-Item -LiteralPath $seed.path).Length -ne $seed.bytes -or
            (Get-FileHash -LiteralPath $seed.path).Hash.ToLowerInvariant() -cne $seed.sha256){throw "Synthetic seed changed at $Phase"}
    }
    foreach($legacy in $taskLegacy){Assert-RetentionTree $legacy.path $legacy.tree}
    Write-RetentionJson (Join-Path $taskOutput ($Phase+'-seeds.json')) $taskSeeds
}
function Get-RetentionAuditEvents([datetime]$Since) {
    try{$events=@(Get-WinEvent -FilterHashtable @{LogName='Security';Id=@(4656,4663);StartTime=$Since} -MaxEvents 30001 -ErrorAction Stop)}catch{
        if($_.FullyQualifiedErrorId -notlike 'NoMatchingEventsFound*'){throw}
        $events=@()
    }
    if($events.Count -gt 30000){throw 'Security audit exceeded bound'}
    foreach($event in $events){
        [xml]$xml=$event.ToXml();$fields=@{}
        foreach($datum in $xml.Event.EventData.Data){$fields[[string]$datum.Name]=[string]$datum.'#text'}
        if($fields.ObjectType -ceq 'File' -and $fields.ObjectName){
            [ordered]@{recordId=$event.RecordId;eventId=$event.Id;atUtc=$event.TimeCreated.ToUniversalTime().ToString('o');
                path=$fields.ObjectName;process=$fields.ProcessName;pid=$fields.ProcessId;mask=$fields.AccessMask}
        }
    }
}
function Start-RetentionAudit {
    # Enable SeSecurityPrivilege for SACLs, only on this disposable hosted VM.
    Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class Ocx023AuditPrivilege {
 [StructLayout(LayoutKind.Sequential)] struct Luid {public uint Low; public int High;}
 [StructLayout(LayoutKind.Sequential)] struct Privilege {public uint Count; public Luid Id; public uint Attributes;}
 [DllImport("advapi32.dll", SetLastError=true)] static extern bool OpenProcessToken(IntPtr p,uint a,out IntPtr t);
 [DllImport("advapi32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern bool LookupPrivilegeValue(string s,string n,out Luid l);
 [DllImport("advapi32.dll", SetLastError=true)] static extern bool AdjustTokenPrivileges(IntPtr t,bool d,ref Privilege p,uint z,IntPtr a,IntPtr b);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 public static void Enable() {IntPtr t; Luid l;
  if(!OpenProcessToken(System.Diagnostics.Process.GetCurrentProcess().Handle,0x28,out t)) throw new Exception("Open audit token failed");
  try {if(!LookupPrivilegeValue(null,"SeSecurityPrivilege",out l)) throw new Exception("Audit privilege missing");
   var p=new Privilege{Count=1,Id=l,Attributes=2};
   if(!AdjustTokenPrivileges(t,false,ref p,0,IntPtr.Zero,IntPtr.Zero)||Marshal.GetLastWin32Error()!=0) throw new Exception("Audit privilege not assigned");
  } finally {CloseHandle(t);}
 }
}
'@
    [Ocx023AuditPrivilege]::Enable()
    $backup=Join-Path $taskOutput 'audit-policy.original.csv'
    & "$env:SystemRoot\System32\auditpol.exe" /backup ('/file:'+$backup) | Out-Null
    if($LASTEXITCODE -ne 0){throw 'Cannot preserve original audit policy'}
    $script:taskAuditBackup=$backup
    & "$env:SystemRoot\System32\auditpol.exe" /set '/subcategory:{0CCE921D-69AE-11D9-BED3-505054503030}' /success:enable /failure:enable | Out-Null
    if($LASTEXITCODE -ne 0){throw 'Cannot enable bounded legacy read audit'}
    $everyone=[Security.Principal.SecurityIdentifier]::new('S-1-1-0')
    foreach($legacy in $taskLegacy){
        $acl=Get-Acl -LiteralPath $legacy.path -Audit
        $script:taskAuditAcls+=@{path=$legacy.path;acl=$acl;auditSddl=$acl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Audit)}
        $audit=Get-Acl -LiteralPath $legacy.path -Audit
        $rule=[Security.AccessControl.FileSystemAuditRule]::new($everyone,[Security.AccessControl.FileSystemRights]::ReadData,
            [Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit',[Security.AccessControl.PropagationFlags]::None,
            [Security.AccessControl.AuditFlags]'Success,Failure')
        $audit.AddAuditRule($rule);Set-Acl -LiteralPath $legacy.path -AclObject $audit
    }
    $script:taskAuditSince=[datetime]::Now
    Confirm-RetentionAuditControls 'initial'
}
function Confirm-RetentionAuditControls([string]$Phase) {
    $before=@(Get-RetentionAuditEvents $taskAuditSince)
    $checkpoint=if($before.Count){($before|Measure-Object recordId -Maximum).Maximum}else{0}
    foreach($legacy in $taskLegacy){$null=[IO.File]::ReadAllBytes((Join-Path $legacy.path 'windows023-legacy-seed.txt'))}
    $deadline=[datetime]::UtcNow.AddSeconds(15)
    do{
        $events=@(Get-RetentionAuditEvents $taskAuditSince)
        $missing=@($taskLegacy|Where-Object {
            $control=Join-Path $_.path 'windows023-legacy-seed.txt'
            !@($events|Where-Object {(Test-RetentionSamePath $_.path $control) -and [Convert]::ToInt64($_.pid,16) -eq $PID -and
                $_.eventId -eq 4663 -and $_.recordId -gt $checkpoint}).Count
        })
        if($missing.Count){Start-Sleep -Milliseconds 250}
    }while($missing.Count -and [datetime]::UtcNow -lt $deadline)
    Write-RetentionJson (Join-Path $taskOutput ($Phase+'-legacy-audit-positive-controls.json')) $events
    if($missing.Count){throw 'Legacy read audit positive control missing; no no-scan claim allowed'}
}
function Test-RetentionLegacyRead($Event,$Roots,[string[]]$InstallRoots=@()) {
    $inLegacy=@($Roots|Where-Object {($Event.path.Replace('\','/').TrimEnd('/')+'/').StartsWith($_.path.Replace('\','/').TrimEnd('/')+'/',[StringComparison]::OrdinalIgnoreCase)}).Count
    $installedChild=@($InstallRoots|Where-Object {($Event.process.Replace('\','/').TrimEnd('/')+'/').StartsWith($_.Replace('\','/').TrimEnd('/')+'/',[StringComparison]::OrdinalIgnoreCase)}).Count
    $product=([IO.Path]::GetFileName($Event.process) -in @('opencodex-desktop.exe','ocxd.exe')) -or $installedChild
    $inLegacy -and $product -and ([Convert]::ToInt64($Event.mask,16) -band 1)
}
function Assert-RetentionNoLegacyRead([string]$Phase) {
    Confirm-RetentionAuditControls $Phase
    $events=@(Get-RetentionAuditEvents $taskAuditSince)
    Write-RetentionJson (Join-Path $taskOutput ($Phase+'-legacy-access.json')) $events
    foreach($legacy in $taskLegacy){
        $control=Join-Path $legacy.path 'windows023-legacy-seed.txt'
        if(!@($events|Where-Object {(Test-RetentionSamePath $_.path $control) -and [Convert]::ToInt64($_.pid,16) -eq $PID -and $_.eventId -eq 4663}).Count){
            throw 'Audit positive control lost or Security log rolled over; absence cannot pass'
        }
    }
    if(@($events|Where-Object {Test-RetentionLegacyRead $_ $taskLegacy $taskInstallRoots}).Count){
        throw 'Candidate attempted to list/read legacy data; original audit events retained'
    }
}
function Assert-RetentionReadback($Facts,$Installation,[string]$FreshLog) {
    if($Facts.about.version -cne '0.2.23' -or !(Test-RetentionSamePath $Facts.paths.managerExecutable.path $Installation.exe) -or
        !(Test-RetentionSamePath $Facts.paths.installationRoot.path $Installation.root) -or
        !(Test-RetentionSamePath $Facts.paths.desktopRoot.path $taskDesktop) -or
        !(Test-RetentionSamePath $Facts.paths.opencodexHome.path $taskHome) -or
        !(Test-RetentionSamePath $Facts.config.currentDataRoot $taskDesktop) -or
        !(Test-RetentionSamePath $Facts.config.currentOpencodexHome $taskHome)){
        throw 'Actual candidate version/I/U/H readback differs'
    }
    if($Facts.preferences.launch_with_codex -ne $false -or $Facts.preferences.cold_sync -ne $false -or
        $Facts.preferences.app_update_auto_check -ne $false){throw 'Synthetic no-sync/check preferences not active'}
    $identity='startup version=0.2.23 commit='+$taskAdmission.candidate.head
    if(!$FreshLog.Contains($identity)){throw 'Running candidate embedded SHA differs from admitted head'}
    $identity
}
function Read-RetentionCdp([int]$Port,[string]$Phase,[int]$RootProcessId) {
    $deadline=[datetime]::UtcNow.AddSeconds(40);$target=$null
    do{
        try{$targets=@(Invoke-RestMethod "http://127.0.0.1:$Port/json/list" -TimeoutSec 2);$target=@($targets|Where-Object {$_.type -ceq 'page' -and $_.webSocketDebuggerUrl})|Select-Object -First 1}catch{}
        if(!$target){Start-Sleep -Milliseconds 250}
    }while(!$target -and [datetime]::UtcNow -lt $deadline)
    if(!$target){throw 'Actual candidate WebView CDP unavailable'}
    # Bind the listener to the exact installed process tree before using CDP.
    $owned=[Collections.Generic.HashSet[int]]::new();$null=$owned.Add($RootProcessId)
    $processes=@(Get-CimInstance Win32_Process)
    do{$changed=$false;foreach($process in $processes){
        if($owned.Contains([int]$process.ParentProcessId) -and $owned.Add([int]$process.ProcessId)){$changed=$true}
    }}while($changed)
    $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction Stop)
    if(!$listeners.Count -or @($listeners|Where-Object {$_.LocalAddress -notin @('127.0.0.1','::1') -or !$owned.Contains([int]$_.OwningProcess)}).Count){
        throw 'CDP listener not owned by the candidate process tree'
    }
    $uri=[uri]$target.webSocketDebuggerUrl
    if($uri.Scheme -cne 'ws' -or $uri.Host -cne '127.0.0.1' -or $uri.Port -ne $Port){throw 'Nonlocal CDP target refused'}
    $ws=[Net.WebSockets.ClientWebSocket]::new();$cancel=[Threading.CancellationTokenSource]::new(90000)
    try{
        $ws.ConnectAsync($uri,$cancel.Token).GetAwaiter().GetResult()
        $expression='(async()=>{const i=window.__TAURI_INTERNALS__?.invoke;if(!i||!document.querySelector("#app")?.children.length)return null;return {about:await i("app_about"),paths:await i("model_paths_v2_snapshot"),config:await i("get_data_root_config"),preferences:await i("get_preferences")};})()'
        $readyDeadline=[datetime]::UtcNow.AddSeconds(60)
        do{
        $request=@{id=23;method='Runtime.evaluate';params=@{expression=$expression;awaitPromise=$true;returnByValue=$true}}|ConvertTo-Json -Depth 5 -Compress
        $bytes=[Text.Encoding]::UTF8.GetBytes($request)
        $ws.SendAsync([ArraySegment[byte]]::new($bytes),[Net.WebSockets.WebSocketMessageType]::Text,$true,$cancel.Token).GetAwaiter().GetResult()
        do{
            $stream=[IO.MemoryStream]::new()
            try{do{
                $buffer=[byte[]]::new(16384)
                $read=$ws.ReceiveAsync([ArraySegment[byte]]::new($buffer),$cancel.Token).GetAwaiter().GetResult()
                if($read.MessageType -ne [Net.WebSockets.WebSocketMessageType]::Text){throw 'Unexpected CDP frame'}
                $stream.Write($buffer,0,$read.Count)
                if($stream.Length -gt 4MB){throw 'CDP response too large'}
            }while(!$read.EndOfMessage)
            $reply=[Text.Encoding]::UTF8.GetString($stream.ToArray())|ConvertFrom-Json
            }finally{$stream.Dispose()}
        }while($reply.id -ne 23)
        ($reply|ConvertTo-Json -Depth 30 -Compress)|Add-Content -LiteralPath (Join-Path $taskOutput ($Phase+'-cdp.original.jsonl')) -Encoding utf8
        if($reply.error -or $reply.result.exceptionDetails){throw 'Actual program identity/path readback failed'}
        if(!$reply.result.result.value){Start-Sleep -Milliseconds 250}
        }while(!$reply.result.result.value -and [datetime]::UtcNow -lt $readyDeadline)
        if(!$reply.result.result.value){throw 'Actual program UI did not mount'}
        $reply.result.result.value
    }finally{$ws.Dispose();$cancel.Dispose()}
}
function Read-RetentionCandidate($Installation,[string]$Phase) {
    if($Installation.sha256 -cne $taskAdmission.candidateInstalledExeSha256){throw 'Installed candidate payload differs from admitted original'}
    $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
    $log=Join-Path $taskDesktop 'logs/app.log';$offset=if(Test-Path -LiteralPath $log){(Get-Item -LiteralPath $log).Length}else{0}
    $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Installation.exe;$info.WorkingDirectory=$Installation.root;$info.UseShellExecute=$false
    $info.Environment['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']="--remote-debugging-address=127.0.0.1 --remote-debugging-port=$port"
    # OS current-profile resolution is intentional: no fake HOME or sandbox.
    $p=[Diagnostics.Process]::Start($info);$creation=$p.StartTime.ToUniversalTime();$failure=$null
    $processCleanup=@{result='pending';error=$null;pid=$p.Id;creationUtc=$creation.ToString('o')}
    try{
        $facts=Read-RetentionCdp $port $Phase $p.Id
        Write-RetentionJson (Join-Path $taskOutput ($Phase+'-native-facts.original.json')) $facts
        if($p.HasExited){throw 'Installed candidate exited during identity readback'}
        $raw=[IO.File]::ReadAllBytes($log)
        if($raw.Length -le $offset){throw 'Fresh startup build identity missing'}
        $fresh=[Text.Encoding]::UTF8.GetString($raw,[int]$offset,$raw.Length-[int]$offset)
        [IO.File]::WriteAllText((Join-Path $taskOutput ($Phase+'-startup.original.log')),$fresh,[Text.UTF8Encoding]::new($false))
        $identity=Assert-RetentionReadback $facts $Installation $fresh
        Write-RetentionJson (Join-Path $taskOutput ($Phase+'-native-readback.json')) @{pid=$p.Id;creationUtc=$creation.ToString('o');image=$Installation;facts=$facts;startupIdentity=$identity;sandbox=$false}
    }catch{$failure=$_}finally{
        # Kill only our still-live process tree; do not uninstall/retry on failure.
        try{if(!$p.HasExited){
            if($p.StartTime.ToUniversalTime() -ne $creation -or !(Test-RetentionSamePath $p.MainModule.FileName $Installation.exe)){throw 'Process identity changed; cleanup refused'}
            $p.Kill($true);if(!$p.WaitForExit(15000)){throw 'Owned candidate tree cleanup timed out'}
        }
        $processCleanup.result='pass'
        }catch{$processCleanup.result='fail';$processCleanup.error=$_.Exception.Message;if(!$failure){$failure=$_}}
        $p.Dispose()
        Write-RetentionJson (Join-Path $taskOutput ($Phase+'-process-cleanup.json')) $processCleanup
    }
    if($failure){throw $failure}
}

if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7 -or $env:GITHUB_ACTIONS -cne 'true' -or
    $env:RUNNER_OS -cne 'Windows' -or $env:RUNNER_ENVIRONMENT -cne 'github-hosted' -or
    $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or $env:GITHUB_EVENT_NAME -cne 'workflow_dispatch' -or
    $env:GITHUB_RUN_ATTEMPT -cne '1'){throw 'Fresh GitHub-hosted Windows Actions first dispatch only'}
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskOutput=Join-Path $taskRepo 'test/out/windows023-installer-retention'
$taskDownloads=Join-Path $taskRepo 'test/software/windows023-installer-retention'
Assert-RetentionPlainPath $AdmissionPath
if(!(Test-RetentionSamePath $AdmissionPath (Join-Path $taskOutput 'admission.json'))){throw 'Exact task admission path required'}
$taskAdmission=Get-Content -LiteralPath $AdmissionPath -Raw|ConvertFrom-Json
$taskSource=(& git -C $taskRepo rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $taskSource -cne $taskAdmission.harnessSource -or $taskSource -cne $env:GITHUB_SHA -or
    $taskAdmission.schemaVersion -ne 23 -or $taskAdmission.scenario -cnotin @('adjacent','fresh023','explicit-conversion') -or $taskAdmission.kind -cnotin @('msi','nsis') -or
    ($taskAdmission.scenario -ceq 'explicit-conversion' -and $taskAdmission.kind -cne 'msi') -or
    $taskAdmission.baselineRelease -ne 435948372 -or $taskAdmission.baselineVersion -cne '0.2.22' -or
    $taskAdmission.candidateVersion -cne '0.2.23' -or $taskAdmission.candidate.head -cnotmatch '^[0-9a-f]{40}$' -or
    $taskAdmission.executionRun -cne $env:GITHUB_RUN_ID -or $taskAdmission.executionJob -cne $env:GITHUB_JOB -or
    !(Test-RetentionSamePath $taskAdmission.outputDirectory $taskOutput)){throw 'Admission/harness/run identity differs'}
foreach($name in @('baseline','candidate')){
    $package=$taskAdmission.($name+'Package');Assert-RetentionPlainPath $package
    if(!(Test-RetentionSamePath (Split-Path -Parent $package) $taskDownloads) -or
        (Get-Item -LiteralPath $package).Length -ne $taskAdmission.($name+'Bytes') -or
        (Get-FileHash -LiteralPath $package).Hash.ToLowerInvariant() -cne $taskAdmission.($name+'Sha256')){throw 'Task installer path/bytes/digest differs'}
}
if(& git -C $taskRepo status --porcelain --untracked-files=no){throw 'Harness tracked files changed'}
if($env:OPENCODEX_SANDBOX -or $env:OPENCODEX_SANDBOX_ROOT -or $env:CODEX_HOME -or $env:WEBVIEW2_USER_DATA_FOLDER){throw 'Inherited product overrides refused'}
$taskProfile=[Environment]::GetFolderPath('UserProfile');$taskLocal=[Environment]::GetFolderPath('LocalApplicationData')
if(!(Test-RetentionSamePath $taskProfile $env:USERPROFILE) -or !(Test-RetentionSamePath $taskLocal $env:LOCALAPPDATA)){throw 'OS current profile differs from runner environment'}
$taskUser=Join-Path $taskProfile '.codex-desktop';$taskDesktop=Join-Path $taskUser 'desktop';$taskHome=Join-Path $taskUser 'opencodex/home'
$taskOldRoots=@((Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'com.gzers.opencodex.desktop'),(Join-Path $taskLocal 'com.gzers.opencodex.desktop'),(Join-Path $taskProfile '.codex'))
$taskAttemptStarted=$false
if(@(Get-RetentionRegistrations).Count){throw 'Pre-existing product registration; do not alter it'}
foreach($path in @($taskUser)+$taskOldRoots+@((Join-Path $taskLocal 'OpenCodeX Desktop'),(Join-Path $env:ProgramFiles 'OpenCodeX Desktop'),(Join-Path $taskLocal 'Programs/OpenCodeX Desktop'))){
    Assert-RetentionPlainPath $path;if(Test-Path -LiteralPath $path){throw 'Existing product/profile/legacy data; fresh runner required'}
}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Product process already exists'}
if(!$Execute){@{result='prepared-only';installerExecuted=$false;windowsAcceptancePassed=$false;kind=$taskAdmission.kind}|ConvertTo-Json;return}
$attempt=Join-Path $taskOutput 'execution-attempt.json'
if(Test-Path -LiteralPath $attempt){throw 'Original attempt exists; no retry or overwrite'}
$taskResult=[ordered]@{schemaVersion=23;result='fail';harnessSource=$taskSource;candidate=$taskAdmission.candidate;kind=$taskAdmission.kind;
    baselineVersion='0.2.22';candidateVersion='0.2.23';scenario=$taskAdmission.scenario;baselineInstalled=($taskAdmission.scenario -cne 'fresh023');automaticUpgradePassed=$false;explicitProgramConversion=($taskAdmission.scenario -ceq 'explicit-conversion');steps=@();defaultPathOverrides=$false;
    legacyReadAuditScope='Candidate desktop, CLI and installed runtime reads/listing of seeded legacy roots';legacyScanVerified=$false;
    ordinaryAccountAcceptanceClaimed=$false;releaseRequested=$false}
Write-RetentionJson $attempt $taskResult
$taskAttemptStarted=$true;$taskMayBeInstalled=$false;$taskSeeds=@();$taskLegacy=@();$taskAuditBackup=$null;$taskAuditAcls=@();$taskFailure=$null;$taskInstallRoots=@();$taskStopAfterExpectedBlock=$false
$cleanup=[ordered]@{result='pending';policyRestored=$false;aclRestored=$false;fixtureDisposition='preserved-on-disposable-runner';installerRetry=$false;errors=@()}
try{
    if($taskAdmission.kind -ceq 'msi'){
        $msiProperties=Read-RetentionMsiProperties $taskAdmission.baselinePackage
        Write-RetentionJson (Join-Path $taskOutput 'baseline-msi-properties.original.json') $msiProperties
        if($msiProperties.productCode -ine '{C5476FDD-CB86-46AF-A97F-96DD56A32CA0}' -or
            $msiProperties.version -cne '0.2.22' -or $msiProperties.allUsers -cne '1'){throw 'Published baseline MSI context/identity differs'}
    }
    $baseline=$null
    if($taskAdmission.scenario -cne 'fresh023'){
        Install-RetentionPackage $taskAdmission.baselinePackage 'install-official-022'
        $baseline=Find-RetentionInstall '0.2.22';$taskResult.baselineInstallation=$baseline;$taskInstallRoots+=,$baseline.root
        $oldInstall=$baseline.root
    }else{$oldInstall=Get-RetentionExpectedRoot $taskAdmission.kind '0.2.22' $taskLocal $env:ProgramFiles}
    # Never start the old program: it must not migrate or discover any old data.
    $taskOldRoots+=Join-Path $oldInstall 'data'
    foreach($root in $taskOldRoots){
        if(Test-Path -LiteralPath $root){throw 'Unexpected old data after fresh baseline install'}
        New-RetentionSeed (Join-Path $root 'windows023-legacy-seed.txt') ('legacy must remain untouched '+[guid]::NewGuid())
        New-RetentionSeed (Join-Path $root 'manager-state/preferences.json') '{invalid legacy document: must never load or import}'
        $taskLegacy+=@{path=$root;tree=(Get-RetentionTree $root)}
    }
    New-RetentionSeed (Join-Path $taskDesktop 'manual/windows023-user-seed.txt') 'manual desktop data retained'
    New-RetentionSeed (Join-Path $taskDesktop 'backups/windows023-retained/seed.txt') 'manual backup retained'
    New-RetentionSeed (Join-Path $taskHome 'windows023-home-seed.txt') 'synthetic H retained; no business credentials'
    # Only synthetic runner preferences; prevent automatic synchronization/checks.
    $prefs=Join-Path $taskDesktop 'manager-state/preferences.json';$null=New-Item -ItemType Directory -Path (Split-Path -Parent $prefs) -Force
    Write-RetentionJson $prefs @{schema_version=2;shell=@{launch_with_codex=$false;auto_panel=$false};updates=@{app_update_auto_check=$false};sync=@{cold_sync=$false}}
    Set-RetentionSyntheticOwner
    $block=Get-RetentionContextBlock $taskAdmission.kind $taskAdmission.scenario $(if($baseline){$baseline.registration}else{$null})
    if($block){
        Assert-RetentionCandidateMsiBlock $baseline
        $block.candidateRejectionVerified=$true
        Write-RetentionJson (Join-Path $taskOutput 'installation-context-blocker.json') $block
        $taskResult.blocker=$block
        if($taskAdmission.scenario -ceq 'adjacent'){
            $taskStopAfterExpectedBlock=$true;$taskResult.result='pass-expected-block'
        }else{
            # Explicit test action, never embedded in product upgrade behavior.
            $retainedBeforeConversion=Get-RetentionTree $taskUser
            Uninstall-RetentionPackage $baseline 'explicit-uninstall-official-022-retain-data'
            Assert-RetentionTree $taskUser $retainedBeforeConversion
            Assert-RetentionSeeds 'after-explicit-old-msi-uninstall'
            Write-RetentionJson (Join-Path $taskOutput 'explicit-program-conversion.json') @{
                authorizedScenario='explicit-conversion';oldVersion='0.2.22';newVersion='0.2.23';
                oldDefaultUninstallExecuted=$true;UandHHashesPreserved=$true;oldDataSeedsPreserved=$true;
                automaticUpgrade=$false;configurationMigration=$false
            }
        }
    }
    if(!$taskStopAfterExpectedBlock){
    Start-RetentionAudit
    Assert-RetentionSeeds 'before-upgrade'
    $installPhase=switch -CaseSensitive ($taskAdmission.scenario){'adjacent'{'upgrade-to-023'} 'explicit-conversion'{'explicit-install-023'} default{'fresh-install-023'}}
    Install-RetentionPackage $taskAdmission.candidatePackage $installPhase
    $candidate=Find-RetentionInstall '0.2.23';$taskResult.candidateInstallation=$candidate;$taskInstallRoots+=,$candidate.root
    if($baseline -and !(Test-RetentionSamePath $baseline.root $candidate.root) -and (Test-Path -LiteralPath $baseline.exe)){throw 'Upgrade left the old executable'}
    Assert-RetentionSeeds 'after-upgrade'
    Read-RetentionCandidate $candidate 'upgraded'
    Assert-RetentionSeeds 'after-upgraded-readback'
    Assert-RetentionNoMigration 'after-upgraded-readback'
    Assert-RetentionNoLegacyRead 'after-upgraded-readback'
    $retained=Get-RetentionTree $taskUser;Write-RetentionJson (Join-Path $taskOutput 'before-default-uninstall-U.json') ($retained|ConvertFrom-Json)
    Uninstall-RetentionPackage $candidate 'default-uninstall'
    Assert-RetentionTree $taskUser $retained;Assert-RetentionSeeds 'after-default-uninstall'
    Install-RetentionPackage $taskAdmission.candidatePackage 'reinstall-023'
    $reinstalled=Find-RetentionInstall '0.2.23';$taskResult.reinstalledInstallation=$reinstalled
    Assert-RetentionTree $taskUser $retained;Assert-RetentionSeeds 'after-reinstall'
    Read-RetentionCandidate $reinstalled 'reinstalled'
    Assert-RetentionSeeds 'after-reinstalled-readback'
    Assert-RetentionNoMigration 'after-reinstalled-readback'
    Assert-RetentionNoLegacyRead 'after-reinstalled-readback'
    $retained=Get-RetentionTree $taskUser
    Uninstall-RetentionPackage $reinstalled 'final-default-uninstall'
    Assert-RetentionTree $taskUser $retained;Assert-RetentionSeeds 'after-final-default-uninstall'
    Assert-RetentionNoLegacyRead 'final'
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Product process remained after cleanup'}
    $taskResult.legacyScanVerified=$true;$taskResult.automaticUpgradePassed=($taskAdmission.scenario -ceq 'adjacent');$taskResult.result='pass'
    }
}catch{$taskFailure=$_;$taskResult.error=$_.Exception.Message;$taskResult.errorType=$_.Exception.GetType().Name
}finally{
    foreach($entry in $taskAuditAcls){try{
        Set-Acl -LiteralPath $entry.path -AclObject $entry.acl
        $restored=(Get-Acl -LiteralPath $entry.path -Audit).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Audit)
        if($restored -cne $entry.auditSddl){throw 'Original legacy SACL readback differs after cleanup'}
    }catch{$cleanup.errors+=($_.Exception.Message)}}
    $cleanup.aclRestored=$cleanup.errors.Count -eq 0
    try{if($taskAuditBackup){
        & "$env:SystemRoot\System32\auditpol.exe" /restore ('/file:'+$taskAuditBackup)|Out-Null
        $cleanup.policyRestored=$LASTEXITCODE -eq 0
        if($cleanup.policyRestored){
            $restoredPolicy=Join-Path $taskOutput 'audit-policy.restored.csv'
            & "$env:SystemRoot\System32\auditpol.exe" /backup ('/file:'+$restoredPolicy)|Out-Null
            $cleanup.policyRestored=$LASTEXITCODE -eq 0 -and !(Compare-Object (Get-Content -LiteralPath $taskAuditBackup) (Get-Content -LiteralPath $restoredPolicy))
        }
        if(!$cleanup.policyRestored){$cleanup.errors+='Audit policy restore failed'}
    }else{$cleanup.policyRestored=$true}
    }catch{$cleanup.policyRestored=$false;$cleanup.errors+=($_.Exception.Message)}
    $cleanup.installerStateMayRemain=$taskMayBeInstalled
    $cleanup.result=if($cleanup.errors.Count){'fail'}else{'pass'}
    if($cleanup.result -cne 'pass'){$taskResult.result='fail'}
    $taskResult.finishedUtc=[datetime]::UtcNow.ToString('o')
    Write-RetentionJson (Join-Path $taskOutput 'cleanup-receipt.json') $cleanup
    Write-RetentionJson (Join-Path $taskOutput 'retention-result.json') $taskResult
}
if($taskFailure){throw $taskFailure}
if($taskResult.result -cnotin @('pass','pass-expected-block')){throw 'Retention cleanup failed; original receipts retained'}
$taskResult|ConvertTo-Json -Depth 15
