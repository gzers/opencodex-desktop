param([Parameter(Mandatory)][string[]]$Controllers,[Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Evidence){throw 'fresh cleanup evidence required'}
[void][IO.Directory]::CreateDirectory($Evidence)
$results=@()
foreach($controller in $Controllers){
 $c=Get-Content -LiteralPath $controller -Raw|ConvertFrom-Json
 if(!$c.created -or $c.name -cnotmatch '^ocxw1c[0-9a-f]{9}$' -or $c.sid -cnotmatch '^S-1-5-21-[0-9-]+$' -or $c.profile -cne ('C:\Users\'+$c.name)){throw 'exact new account/Profile controller provenance required'}
 $users=@(Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue)
 if($users.Count){throw 'prior controller must remove its account before Profile cleanup'}
 $native=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'"))
 $row=[ordered]@{sid=$c.sid;profile=$c.profile;beforeExists=(Test-Path -LiteralPath $c.profile);profileServiceObject=$native.Count;archivedFiles=@()}
 if($native.Count){
  if($native.Count -ne 1 -or $native[0].Loaded -or $native[0].Special -or $native[0].LocalPath -cne $c.profile){throw 'Profile generation/state differs; retained'}
  $total=0L
  foreach($relative in @('.codex-desktop/desktop','w1c native/data/desktop','w1c native/custom H 雪','w1c native/I 空格 & !/runtime/bin')){
   $root=Join-Path $c.profile $relative
   try {$present=Test-Path -LiteralPath $root} catch [System.UnauthorizedAccessException] {
    # Preserve only these fixed new-account state roots using the admin backup
    # privilege. Source ACL/owner stay unchanged; never take ownership or grant
    # access to a daily user's path. The account is already deleted and unloaded.
    $backup=Join-Path $Evidence ($c.name+'/'+$relative)
    [void][IO.Directory]::CreateDirectory($backup)
    $log=Join-Path $Evidence ($c.name+'-'+($relative -replace '[^a-zA-Z0-9]','_')+'-backup.log')
    & robocopy.exe $root $backup /E /B /COPY:DAT /DCOPY:DAT /R:0 /W:0 /XJ /XJD /XJF /NP /NFL /NDL ('/LOG:'+$log)|Out-Null
    if($LASTEXITCODE -ge 8){throw ('exact new-profile backup failed: '+$LASTEXITCODE)}
    $root=$backup;$present=$true
   }
   if($present){foreach($file in Get-ChildItem -LiteralPath $root -File -Recurse -Force){
    if($file.Attributes -band [IO.FileAttributes]::ReparsePoint -or $file.Length -gt 16777216 -or ($total+$file.Length) -gt 33554432){throw 'private failure archive bound/reparse refusal'}
    $bytes=[IO.File]::ReadAllBytes($file.FullName);$total+=$bytes.Length
    $row.archivedFiles+=@{relative=$relative+'/'+[IO.Path]::GetRelativePath($root,$file.FullName);bytes=$bytes.Length;sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant();base64=[Convert]::ToBase64String($bytes)}
   }}
  }
  $row|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $Evidence ($c.name+'-before.private.json')) -Encoding utf8
  $native[0]|Remove-CimInstance -ErrorAction Stop
 }
 $row.afterExists=Test-Path -LiteralPath $c.profile
 $row.afterProfileServiceObject=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")).Count
 $row.accountStillPresent=@(Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue).Count -ne 0
 $row|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $Evidence ($c.name+'-result.private.json')) -Encoding utf8
 $results+=@{sid=$row.sid;profile=$row.profile;afterExists=$row.afterExists;serviceObjects=$row.afterProfileServiceObject;accountPresent=$row.accountStillPresent;archivedFiles=$row.archivedFiles.Count}
}
$results|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8
