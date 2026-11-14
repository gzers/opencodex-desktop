$ErrorActionPreference='Stop'
$root='C:/Users/Public/ocx-msi-admin-20261114-r1'
$sid='S-1-5-21-587408064-856152790-2583328096-1040'
$r=[ordered]@{at=[datetime]::Now.ToString('o');scope='read-only exact newly-created test SID profile unload diagnosis';events=@();errors=@()}
try{
 $key='HKLM:/SOFTWARE/Microsoft/Windows NT/CurrentVersion/ProfileList/'+$sid
 $r.profileRegistration=if(Test-Path -LiteralPath $key){Get-ItemProperty -LiteralPath $key|Select-Object ProfileImagePath,State,Flags,RefCount}else{$null}
 $r.userHiveMounted=Test-Path -LiteralPath ('Registry::HKEY_USERS/'+$sid)
 $r.classesHiveMounted=Test-Path -LiteralPath ('Registry::HKEY_USERS/'+$sid+'_Classes')
 $since=[datetime]::Parse('2026-11-14T15:00:00+08:00').ToLocalTime()
 foreach($log in @('Microsoft-Windows-User Profile Service/Operational','Application')){
  try{
   $filter=@{LogName=$log;StartTime=$since}
   if($log -eq 'Application'){$filter.ProviderName='Microsoft-Windows-User Profiles Service'}
   foreach($event in Get-WinEvent -FilterHashtable $filter -MaxEvents 300 -ErrorAction Stop){
    $xml=$event.ToXml()
    if($xml.Contains($sid) -or $xml.Contains('ocxw1d1d3a825b8')){$r.events+=@{log=$log;id=$event.Id;at=$event.TimeCreated;message=$event.Message;xml=$xml}}
   }
  }catch{$r.errors+=@{log=$log;error=$_.Exception.Message}}
 }
 $r.profile=Get-CimInstance Win32_UserProfile -Filter ("SID='"+$sid+"'")|Select-Object SID,LocalPath,Loaded,Special,Status
}finally{$r|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'profile-diagnosis04.json') -Encoding utf8}
