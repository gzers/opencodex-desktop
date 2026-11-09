# Disposable default-uninstall sentinel only. Refuse an existing aggregate;
# removal requires the complete exact tree and content originally created here.
function New-InstallerRetentionFixture([string]$Aggregate) {
 $root=[IO.Path]::GetFullPath($Aggregate)
 if(Test-Path -LiteralPath $root){throw 'Retention fixture requires an absent aggregate'}
 $directory=Join-Path $root 'desktop/sweep-retention'
 New-Item -ItemType Directory -Path $directory|Out-Null
 $file=Join-Path $directory ([guid]::NewGuid().ToString('N')+'.sentinel')
 [IO.File]::WriteAllText($file,'owned installer default user-data retention fixture',[Text.UTF8Encoding]::new($false))
 @{root=$root;directory=$directory;path=$file;hash=(Get-FileHash -LiteralPath $file).Hash;createdAggregate=$true}
}
function Remove-InstallerRetentionFixture($Fixture) {
 if(!$Fixture.createdAggregate){throw 'Unowned retention aggregate refused'}
 $root=[IO.Path]::GetFullPath($Fixture.root)
 $directory=Join-Path $root 'desktop/sweep-retention'
 if([IO.Path]::GetFullPath($Fixture.directory) -cne [IO.Path]::GetFullPath($directory) -or [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($Fixture.path)) -cne [IO.Path]::GetFullPath($directory) -or [IO.Path]::GetFileName($Fixture.path) -notmatch '^[a-f0-9]{32}[.]sentinel$'){throw 'Retention fixture path binding changed'}
 $expected=@($directory,(Join-Path $root 'desktop'),$root)
 $items=@(Get-Item -LiteralPath $root -Force)+@(Get-ChildItem -LiteralPath $root -Recurse -Force)
 if($items.Count -ne 4){throw 'Unexpected retained data; aggregate preserved'}
 foreach($item in $items){
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Linked retained data; aggregate preserved'}
  if($item.PSIsContainer){if($item.FullName -notin $expected){throw 'Unexpected retained directory; aggregate preserved'}}
  elseif($item.FullName -cne [IO.Path]::GetFullPath($Fixture.path) -or (Get-FileHash -LiteralPath $item.FullName).Hash -cne $Fixture.hash){throw 'Changed retained sentinel; aggregate preserved'}
 }
 # No recursive deletion. Any unexpected/racing child makes empty-dir removal
 # fail; nothing else is removed or permission-repaired to make the test pass.
 Remove-Item -LiteralPath $Fixture.path
 foreach($path in $expected){[IO.Directory]::Delete($path,$false)}
 if(Test-Path -LiteralPath $root){throw 'Owned retention fixture cleanup incomplete'}
 @{removed=$true;scope='exact new sentinel and empty parents after default-uninstall hash proof';productRetentionProved=$true}
}
