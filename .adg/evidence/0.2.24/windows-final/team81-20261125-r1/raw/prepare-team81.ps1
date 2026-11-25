$ErrorActionPreference='Stop'
$p='C:/Users/15119/ocx-team81-a69ae55-r1'
$verifyPath="$p/digest-verification.json"
$expected=[ordered]@{'verified-candidate.zip'='356747794908f6497350ed9874912745ef3c784adb8242caf1ee552d5f68eb67';'OpenCodeX Desktop_0.2.24_x64_en-US.msi'='f8bbece2817e40ae3b5528d2d00caa055cebc158617cad13569da15de6fe4fe8';'OpenCodeX Desktop_0.2.24_x64-setup.exe'='4999df3bcfbc47294ff08a4093673e1ca11d313e15dc091d9a6d6114266b8db4'}
$results=@(foreach($name in $expected.Keys){$file=Get-Item -LiteralPath "$p/$name"; $hash=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); [ordered]@{name=$name;bytes=$file.Length;sha256=$hash;expected=$expected[$name];match=$hash -ceq $expected[$name]}})
[ordered]@{observedUtc=[DateTime]::UtcNow.ToString('o'); runId=45630909723; artifactId=15030939242; source='a69ae55a3fe21b58f415da727b0d1bdff8f37568'; files=$results} | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8NoBOM -LiteralPath $verifyPath
if($results.Where({-not $_.match}).Count -ne 0){throw 'Candidate digest mismatch'}
Get-Content -LiteralPath $verifyPath
$target=Join-Path $p 'msi-admin'; if(Test-Path -LiteralPath $target){throw 'Extraction target already exists'}
$msi=(Get-Item -LiteralPath "$p/OpenCodeX Desktop_0.2.24_x64_en-US.msi").FullName
$log=Join-Path $p 'msi-admin.log'
$args='/a "{0}" /qn TARGETDIR="{1}" /l*v "{2}"' -f $msi,$target,$log
$sw=[Diagnostics.Stopwatch]::StartNew(); $proc=Start-Process -FilePath "$env:WINDIR/System32/msiexec.exe" -ArgumentList $args -WindowStyle Hidden -PassThru
if(-not $proc.WaitForExit(120000)){throw 'Administrative extraction still running after 120s'}
[ordered]@{exitCode=$proc.ExitCode;elapsedMs=$sw.ElapsedMilliseconds;mode='administrative extraction';pid=$proc.Id} | ConvertTo-Json | Set-Content -Encoding utf8NoBOM -LiteralPath "$p/msi-admin.json"
if($proc.ExitCode -ne 0){throw "Administrative extraction failed: $($proc.ExitCode)"}
$exe=@(Get-ChildItem -LiteralPath $target -Filter 'opencodex-desktop.exe' -File -Recurse)
if($exe.Count -ne 1){throw "Expected a single candidate desktop image, found $($exe.Count)"}
$complete=$exe[0].Directory.FullName
$bundle=Join-Path $complete 'runtime-bundle'; if(-not(Test-Path -LiteralPath $bundle -PathType Container)){throw 'Verified MSI payload did not contain runtime-bundle'}
$missing=Join-Path $p 'missing-image'; New-Item -ItemType Directory -Path $missing | Out-Null
Get-ChildItem -LiteralPath $complete -Force | Where-Object Name -cne 'runtime-bundle' | ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination $missing -Recurse -Force }
$inventory=@(Get-ChildItem -LiteralPath $complete -Recurse -File -Force | Sort-Object FullName | ForEach-Object {
  $rel=[IO.Path]::GetRelativePath($complete,$_.FullName); $t=Join-Path $missing $rel
  $sha=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()
  $other=if(Test-Path -LiteralPath $t -PathType Leaf){(Get-FileHash -LiteralPath $t).Hash.ToLowerInvariant()}
  if($rel -notlike 'runtime-bundle\*' -and $sha -cne $other){throw "Missing payload fixture changed $rel"}
  [ordered]@{path=$rel;bytes=$_.Length;sha256=$sha;missingFixtureSha256=$other}
})
if(Test-Path -LiteralPath (Join-Path $missing 'runtime-bundle')){throw 'Missing fixture contains runtime-bundle'}
[ordered]@{complete=$complete;missing=$missing;onlyExcludedTree='runtime-bundle';inventory=$inventory} | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8NoBOM -LiteralPath "$p/payload-inventory.json"
"Prepared $($inventory.Count) files; complete=$complete"

