param([Parameter(Mandatory)][string]$Msi,[Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Evidence){throw 'fresh evidence directory required'}
New-Item -ItemType Directory -Path $Evidence|Out-Null
$report=[ordered]@{scope='read-only native MSI database and costing; no installation';passed=$false;installed=$false;cells=@()}
$installer=$null;$db=$null
try{
 $installer=New-Object -ComObject WindowsInstaller.Installer
 $installer.UILevel=2
 @{pid=$PID;step='database-open';at=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Evidence 'progress.json') -Encoding utf8
 $db=$installer.OpenDatabase([IO.Path]::GetFullPath($Msi),0)
 function Read-Table([string]$sql,[int]$columns){
  $view=$db.OpenView($sql);[void]$view.Execute()
  $rows=@()
  while($row=$view.Fetch()){$values=@();for($i=1;$i -le $columns;$i++){$values+=$row.StringData($i)};$rows+=,@($values)}
  [void]$view.Close();return ,$rows
 }
 $report.directories=Read-Table ('SELECT {0}Directory{0},{0}Directory_Parent{0},{0}DefaultDir{0} FROM {0}Directory{0}' -f [char]96) 3
 $report.files=Read-Table ('SELECT {0}File{0},{0}Component_{0},{0}FileName{0},{0}FileSize{0} FROM {0}File{0}' -f [char]96) 4
 $report.components=Read-Table ('SELECT {0}Component{0},{0}Directory_{0},{0}KeyPath{0} FROM {0}Component{0}' -f [char]96) 3
 $report.properties=Read-Table ('SELECT {0}Property{0},{0}Value{0} FROM {0}Property{0}' -f [char]96) 2
 $report.removeFiles=Read-Table ('SELECT {0}FileKey{0},{0}Component_{0},{0}FileName{0},{0}DirProperty{0},{0}InstallMode{0} FROM {0}RemoveFile{0}' -f [char]96) 5
 foreach($name in @('runtime-bundle-0.2.24-win32-x64.tar.gz','official-runtime.tgz','runtime-bundle.manifest.json','runtime-bundle.manifest.sha256')){
  if(@($report.files|Where-Object {($_[2] -split '[|]')[-1] -ceq $name}).Count -ne 1){throw ('one actual MSI resource required: '+$name)}
 }
 if(@($report.files|Where-Object {$_[2] -match 'maintained-runtime|0[.]2[.]23.*tar'}).Count){throw 'old maintained resource still packaged'}
 if(@($report.properties|Where-Object {$_[0] -ceq 'ALLUSERS'}).Count){throw 'machine-scope MSI property present'}
 $emptyFolderIds=@('DesktopFolder','INSTALLDIR','ApplicationProgramsFolder','OCX023Documents','OCX023RuntimeBundle','OCX023ProgramsFolder')
 if(@($report.removeFiles|Where-Object {$_[2] -ne '' -or $_[3] -cnotin $emptyFolderIds}).Count){throw 'unexpected wildcard/data deletion; only exact empty installer folders may be removed'}
 $default=[IO.Path]::Combine($env:LOCALAPPDATA,'Programs','OpenCodeX Desktop')
 $custom=[IO.Path]::Combine($Evidence,'custom 空格','Desktop')
 foreach($cell in @(@{mode='default';input=$null;expected=$default},@{mode='custom';input=$custom;expected=$custom})){
  @{pid=$PID;step='OpenPackage';mode=$cell.mode;at=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Evidence 'progress.json') -Encoding utf8
  $session=$installer.OpenPackage([IO.Path]::GetFullPath($Msi),1)
  if($cell.input){$session.Property('INSTALLDIR')=$cell.input}
  $actions=@();foreach($action in @('CostInitialize','FileCost','CostFinalize')){
   @{pid=$PID;step=$action;mode=$cell.mode;at=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Evidence 'progress.json') -Encoding utf8
   $outcome=$session.DoAction($action);$actions+=@{action=$action;result=$outcome};if($outcome -ne 1){throw ('native costing action failed: '+$action)}
  }
  $actual=$session.Property('INSTALLDIR').TrimEnd([char]92)
  if($actual -ine $cell.expected.TrimEnd([char]92)){throw ('resolved MSI '+$cell.mode+' I differs: '+$actual)}
  $report.cells+=@{mode=$cell.mode;resolved=$actual;expected=$cell.expected;actions=$actions;directoryCreated=(Test-Path -LiteralPath $actual)}
  [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($session)
 }
 $report.passed=$true
}catch{$report.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $Evidence 'first-error.log') -Encoding utf8}
finally{
 if($db){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($db)}
 if($installer){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($installer)}
 $report|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8
}
if(!$report.passed){exit 1}
