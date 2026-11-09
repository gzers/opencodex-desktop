$ErrorActionPreference='Stop'
$prep=$PSScriptRoot
$ci='protocol=https'+[Environment]::NewLine+'host=github.com'+[Environment]::NewLine+[Environment]::NewLine
$lines=@($ci|git credential fill)
$secret=$lines|Where-Object {$_ -like 'password=*'}|Select-Object -First 1
if(!$secret){throw 'GitHub credential unavailable'}
$artifacts=(Get-Content -LiteralPath (Join-Path $prep 'build-artifacts.json') -Raw|ConvertFrom-Json).artifacts
foreach($artifact in $artifacts){
 $program=$artifact.name.StartsWith('windows023-core-program-')
 $name=if($program){'program.zip'}else{'build-logs.zip'}
 $destination=Join-Path $prep $name
 if(Test-Path -LiteralPath $destination){throw 'Never overwrite a download'}
 $count=if($program){4}else{1}
 $parts=Join-Path $prep ($name+'.parts')
 New-Item -ItemType Directory -Path $parts|Out-Null
 $config=[Collections.Generic.List[string]]::new()
 for($i=0;$i -lt $count;$i++){
  if($i -gt 0){$config.Add('next')}
  $config.Add('url = "https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/'+$artifact.id+'/zip"')
  $config.Add('header = "Authorization: Bearer '+$secret.Substring(9)+'"')
  $config.Add('header = "X-GitHub-Api-Version: 2022-11-28"')
  $config.Add('proxy = "http://127.0.0.1:3060"')
  $config.Add('location');$config.Add('fail');$config.Add('silent');$config.Add('show-error')
  $config.Add('max-time = 600')
  $config.Add('output = "'+(Join-Path $parts ($i.ToString()+'.part')).Replace([char]92,[char]47)+'"')
  if($program){
   $start=[long][math]::Floor($artifact.size_in_bytes*$i/$count)
   $end=[long][math]::Floor($artifact.size_in_bytes*($i+1)/$count)-1
   $config.Add('range = "'+$start+'-'+$end+'"')
  }
  $config.Add('write-out = "%{http_code} %{size_download}\n"')
 }
 $config -join [Environment]::NewLine|& 'C:\Windows\System32\curl.exe' --parallel --parallel-immediate --parallel-max 4 --config - 1> (Join-Path $prep ($name+'.download.stdout.log')) 2> (Join-Path $prep ($name+'.download.stderr.log'))
 $exit=$LASTEXITCODE
 if($exit -ne 0){throw ('Artifact download failed; exit='+$exit+'; partial evidence retained')}
 $out=[IO.File]::Open($destination,[IO.FileMode]::CreateNew)
 try{
  for($i=0;$i -lt $count;$i++){
   $part=Join-Path $parts ($i.ToString()+'.part')
   if($program){
    $expected=[long][math]::Floor($artifact.size_in_bytes*($i+1)/$count)-[long][math]::Floor($artifact.size_in_bytes*$i/$count)
    if((Get-Item -LiteralPath $part).Length -ne $expected){throw 'Range size differs; do not trust partial archive'}
   }
   $inputStream=[IO.File]::OpenRead($part)
   try{$inputStream.CopyTo($out)}finally{$inputStream.Dispose()}
  }
 }finally{$out.Dispose()}
 $sha=(Get-FileHash -LiteralPath $destination).Hash.ToLowerInvariant()
 if((Get-Item -LiteralPath $destination).Length -ne $artifact.size_in_bytes -or 'sha256:'+$sha -cne $artifact.digest){throw 'Downloaded artifact differs from API identity'}
 @{artifactId=$artifact.id;bytes=(Get-Item -LiteralPath $destination).Length;sha256=$sha;proxy='http://127.0.0.1:3060';rangeCount=$count}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $prep ($name+'.download.json')) -Encoding utf8
 @{artifactId=$artifact.id;name=$name;verified=$true;bytes=$artifact.size_in_bytes}|ConvertTo-Json
 $config.Clear()
}
$secret=$null;$lines=$null
