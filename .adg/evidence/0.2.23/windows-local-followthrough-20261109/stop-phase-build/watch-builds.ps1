$ErrorActionPreference='Stop'
$root=$PSScriptRoot
$expected='4d783585645206c52b49bce8656475d26940c1d7'
$ci='protocol=https'+[Environment]::NewLine+'host=github.com'+[Environment]::NewLine+[Environment]::NewLine
$lines=@($ci|git credential fill);$secret=$lines|Where-Object {$_ -like 'password=*'}|Select-Object -First 1
$h=@{Authorization='Bearer '+$secret.Substring(9);Accept='application/vnd.github+json';'User-Agent'='ocxd-stop-phase'}
$complete=@{};$last=@{};$until=[datetime]::UtcNow.AddMinutes(40)
try{
 while($complete.Count -lt 2){
  foreach($id in @(42653885333,42653892698)){
   if($complete.ContainsKey($id)){continue}
   $r=Invoke-RestMethod ('https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/'+$id) -Headers $h -Proxy 'http://127.0.0.1:3060' -HttpVersion 1.1 -TimeoutSec 20
   if($r.head_sha -cne $expected){throw 'Build source changed'}
   $r|ConvertTo-Json -Depth 25|Set-Content (Join-Path $root ('run-'+$id+'.json'))
   $j=Invoke-RestMethod ('https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/'+$id+'/jobs') -Headers $h -Proxy 'http://127.0.0.1:3060' -HttpVersion 1.1 -TimeoutSec 20
   $j|ConvertTo-Json -Depth 30|Set-Content (Join-Path $root ('jobs-'+$id+'.json'))
   $active=@($j.jobs|Where-Object {$_.conclusion -ne 'skipped'}|ForEach-Object {$_.steps|Where-Object status -eq 'in_progress'|ForEach-Object {$_.name}})-join ', '
   $status=$r.status+' '+$r.conclusion+' '+$active
   if($last[$id] -cne $status){Write-Output ($id.ToString()+' '+$status);$last[$id]=$status}
   if($r.status -ne 'completed'){continue}
   $a=Invoke-RestMethod ('https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/'+$id+'/artifacts') -Headers $h -Proxy 'http://127.0.0.1:3060' -HttpVersion 1.1 -TimeoutSec 20
   $folder=Join-Path $root $(if($id -eq 42653885333){'program-build'}else{'fixture-build'});New-Item -ItemType Directory -Path $folder|Out-Null
   $a|ConvertTo-Json -Depth 20|Set-Content (Join-Path $folder 'build-artifacts.json')
   $r|ConvertTo-Json -Depth 25|Set-Content (Join-Path $folder 'build-run-final.json')
   $j|ConvertTo-Json -Depth 30|Set-Content (Join-Path $folder 'build-jobs-final.json')
   if($id -eq 42653885333){
    Copy-Item -LiteralPath 'D:\ocx13-stop-build-20261109-220455\download-artifacts.ps1' -Destination $folder
    & (Join-Path $folder 'download-artifacts.ps1')
    $program=@($a.artifacts|Where-Object name -eq ('windows023-core-program-'+$expected))
    if($program.Count -eq 1){$program[0]|ConvertTo-Json -Depth 15|Set-Content (Join-Path $folder 'program-artifact.json');Expand-Archive -LiteralPath (Join-Path $folder 'program.zip') -DestinationPath (Join-Path $folder 'program')}
    Expand-Archive -LiteralPath (Join-Path $folder 'build-logs.zip') -DestinationPath (Join-Path $folder 'logs')
   }else{
    foreach($artifact in $a.artifacts){
     $name=if($artifact.name.StartsWith('windows023-local-provision-logs-')){'logs'}else{'tool'}
     $path=Join-Path $folder ($name+'.zip')
     Invoke-WebRequest $artifact.archive_download_url -Headers $h -Proxy 'http://127.0.0.1:3060' -HttpVersion 1.1 -TimeoutSec 60 -OutFile $path
     $sha=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()
     if((Get-Item -LiteralPath $path).Length -ne $artifact.size_in_bytes -or 'sha256:'+$sha -cne $artifact.digest){throw 'Fixture artifact SHA/bytes mismatch'}
     @{id=$artifact.id;bytes=$artifact.size_in_bytes;sha256=$sha;source=$expected;run=$id;proxy='http://127.0.0.1:3060'}|ConvertTo-Json|Set-Content (Join-Path $folder ($name+'-download.json'))
     Expand-Archive -LiteralPath $path -DestinationPath (Join-Path $folder $name)
    }
   }
   $complete[$id]=$r.conclusion
   Write-Output ($id.ToString()+' evidence downloaded '+$r.conclusion)
  }
  if([datetime]::UtcNow -gt $until){throw 'Build monitoring envelope elapsed'}
  if($complete.Count -lt 2){Start-Sleep -Seconds 15}
 }
 $complete|ConvertTo-Json|Set-Content (Join-Path $root 'downloads-finished.json')
}finally{$h.Clear();$secret=$null;$lines=$null}
