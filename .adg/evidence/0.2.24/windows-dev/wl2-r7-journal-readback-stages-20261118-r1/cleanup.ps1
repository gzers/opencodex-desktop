param([Parameter(Mandatory)][string]$Run)
$ErrorActionPreference='Stop'
$runRoot=(Resolve-Path -LiteralPath $Run).ProviderPath
$fixtureRows=Get-Content -LiteralPath "$runRoot/fixture-state.json" -Raw | ConvertFrom-Json
$tempRoot=[IO.Path]::TrimEndingDirectorySeparator((Resolve-Path -LiteralPath ([IO.Path]::GetTempPath())).ProviderPath)
$dataRoot=(Resolve-Path -LiteralPath 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target').ProviderPath
$rows=@(foreach($row in $fixtureRows){
    $path=[IO.Path]::GetFullPath($row.path)
    $keep=$row.suiteRaw -ne 0 -or $row.existedBeforeSuite -or !$row.reportedByFixture
    $errorText=$null
    $members=@()
    if(!$keep -and (Test-Path -LiteralPath $path)){
        try{
            $parent=[IO.Path]::GetDirectoryName($path)
            $name=[IO.Path]::GetFileName($path)
            if(!(($parent -eq $tempRoot -and $name -cmatch '^[.]tmp[A-Za-z0-9]+$') -or
                 ($parent -eq $dataRoot -and $name -cmatch '^r6-v3-[0-9a-f-]{36}$'))){
                throw 'fixture lies outside the known synthetic roots'
            }
            $resolved=(Resolve-Path -LiteralPath $path).ProviderPath
            if($resolved -ne $path){throw 'fixture resolved name changed'}
            $rootItem=Get-Item -LiteralPath $resolved -Force
            if(!$rootItem.PSIsContainer -or ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)){
                throw 'fixture root is not a plain directory'
            }
            $queue=[Collections.Generic.Queue[string]]::new()
            $queue.Enqueue($resolved)
            $members=@(while($queue.Count){
                foreach($member in Get-ChildItem -LiteralPath ($queue.Dequeue()) -Force){
                    if($member.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'fixture child is a reparse point'}
                    if($member.PSIsContainer){$queue.Enqueue($member.FullName)}
                    $streams=@(Get-Item -LiteralPath $member.FullName -Stream * | Where-Object Stream -ne ':$DATA')
                    if($streams.Count){throw 'fixture has a named stream'}
                    [ordered]@{relative=[IO.Path]::GetRelativePath($resolved,$member.FullName); directory=$member.PSIsContainer;
                        sha256=$(if(!$member.PSIsContainer){(Get-FileHash -LiteralPath $member.FullName).Hash.ToLowerInvariant()})}
                }
            })
            if(@(Get-Item -LiteralPath $resolved -Stream * | Where-Object Stream -ne ':$DATA').Count){throw 'fixture root has a named stream'}
            Remove-Item -LiteralPath $resolved -Recurse -Force
        }catch{$errorText=$_.Exception.Message}
    }
    [ordered]@{path=$path; suite=$row.suite; suiteRaw=$row.suiteRaw; absent=!(Test-Path -LiteralPath $path);
        preservedByResult=$keep; error=$errorText; inventory=$members}
})
$schemaPath=[IO.Path]::GetFullPath('apps/desktop/tauri/gen/schemas/windows-schema.json')
$schema=$null
if(Test-Path -LiteralPath $schemaPath){
    $schemaRelative='apps/desktop/tauri/gen/schemas/windows-schema.json'
    if(git ls-files -- $schemaRelative){throw 'Windows schema is tracked'}
    if($LASTEXITCODE -ne 0){throw 'schema ownership check failed'}
    $schemaResolved=(Resolve-Path -LiteralPath $schemaPath).ProviderPath
    if($schemaResolved -ne $schemaPath){throw 'schema resolved name changed'}
    $archive=Join-Path $runRoot 'generated-windows-schema.json'
    [IO.File]::Copy($schemaResolved,$archive,$false)
    $sha=(Get-FileHash -LiteralPath $schemaResolved).Hash.ToLowerInvariant()
    if($sha -ne (Get-FileHash -LiteralPath $archive).Hash.ToLowerInvariant()){throw 'schema archive digest differs'}
    Remove-Item -LiteralPath $schemaResolved
    $schema=[ordered]@{path=$schemaResolved; archive=$archive; sha256=$sha; absent=!(Test-Path -LiteralPath $schemaResolved)}
}
[ordered]@{fixtures=$rows; schema=$schema; priorDeniedPathsRetried=$false; currentR2Touched=$false} |
    ConvertTo-Json -Depth 10 | Set-Content -LiteralPath "$runRoot/cleanup.json" -Encoding utf8NoBOM
$rows | Select-Object -Property path,suiteRaw,absent,error | Format-Table
