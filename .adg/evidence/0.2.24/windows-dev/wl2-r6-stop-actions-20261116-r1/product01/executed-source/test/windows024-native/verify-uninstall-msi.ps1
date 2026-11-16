param([Parameter(Mandatory)][string]$Msi,[Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Evidence){throw 'fresh MSI uninstall evidence required'}
[void][IO.Directory]::CreateDirectory($Evidence)
$report=[ordered]@{passed=$false;scope='native MSI action database and condition evaluation only; no installation/uninstall executed';cells=@()}
$installer=$null;$db=$null;$package=0u
try{
 $installer=New-Object -ComObject WindowsInstaller.Installer;$installer.UILevel=2
 $db=$installer.OpenDatabase([IO.Path]::GetFullPath($Msi),0)
 function Row([string]$sql,[int]$columns){$view=$db.OpenView($sql);[void]$view.Execute();$r=$view.Fetch();if(!$r){throw 'required MSI row missing'};$values=@();for($i=1;$i -le $columns;$i++){$values+=$r.StringData($i)};if($view.Fetch()){throw 'ambiguous MSI action row'};[void]$view.Close();return ,$values}
 $quote=[char]96
 $action=Row ('SELECT {0}Type{0},{0}Source{0},{0}Target{0} FROM {0}CustomAction{0} WHERE {0}Action{0}=''OCX023RevokeTerminalCli''' -f $quote) 3
 $sequence=Row ('SELECT {0}Condition{0},{0}Sequence{0} FROM {0}InstallExecuteSequence{0} WHERE {0}Action{0}=''OCX023RevokeTerminalCli''' -f $quote) 2
 $remove=Row ('SELECT {0}Sequence{0} FROM {0}InstallExecuteSequence{0} WHERE {0}Action{0}=''RemoveFiles''' -f $quote) 1
 if([int]$action[0] -ne 1058 -or $action[1] -cne 'INSTALLDIR' -or $action[2] -cne '"[INSTALLDIR]ocxd.exe" --uninstall-terminal-cli --confirm'){throw 'deferred impersonating checked helper action differs'}
 if([int]$sequence[1] -ge [int]$remove[0]){throw 'terminal cleanup must precede RemoveFiles'}
 $report.action=@{type=[int]$action[0];source=$action[1];command=$action[2];condition=$sequence[0];sequence=[int]$sequence[1];removeFilesSequence=[int]$remove[0]}
 Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class W1cMsiConditions {
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] public static extern uint MsiOpenPackageExW(string path,uint options,out uint package);
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] public static extern uint MsiSetPropertyW(uint package,string name,string value);
 [DllImport("msi.dll",CharSet=CharSet.Unicode)] public static extern int MsiEvaluateConditionW(uint package,string condition);
 [DllImport("msi.dll")] public static extern uint MsiCloseHandle(uint handle);
 [DllImport("msi.dll")] public static extern uint MsiSetInternalUI(uint level,IntPtr hwnd);
}
'@
 [void][W1cMsiConditions]::MsiSetInternalUI(2,[IntPtr]::Zero)
 $code=[W1cMsiConditions]::MsiOpenPackageExW([IO.Path]::GetFullPath($Msi),1,[ref]$package)
 if($code -ne 0){throw ('read-only MSI session failed: '+$code)}
 foreach($cell in @(@{name='fresh-install';installed='';remove='';upgrade='';expect=0},@{name='normal-uninstall';installed='1';remove='ALL';upgrade='';expect=1},@{name='upgrade-removal';installed='1';remove='ALL';upgrade='{TEST-UPGRADE}';expect=0},@{name='repair';installed='1';remove='';upgrade='';expect=0},@{name='partial-feature-removal';installed='1';remove='ShortcutsFeature';upgrade='';expect=0})){
  foreach($pair in @(@('Installed',$cell.installed),@('REMOVE',$cell.remove),@('UPGRADINGPRODUCTCODE',$cell.upgrade))){if([W1cMsiConditions]::MsiSetPropertyW($package,$pair[0],$pair[1]) -ne 0){throw 'MSI property set failed'}}
  $actual=[W1cMsiConditions]::MsiEvaluateConditionW($package,$sequence[0])
  $report.cells+=@{name=$cell.name;expected=$cell.expect;actual=$actual}
  if($actual -ne $cell.expect){throw ('native MSI uninstall condition differs: '+$cell.name)}
 }
 $report.passed=$true
}catch{$report.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $Evidence 'first-error.log') -Encoding utf8}
finally{if($package){[void][W1cMsiConditions]::MsiCloseHandle($package)};if($db){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($db)};if($installer){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($installer)};$report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8}
if(!$report.passed){exit 1}
