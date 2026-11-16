param([Parameter(Mandatory)][string]$Msi,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'fresh MSI package facts output required'}
$path=(Resolve-Path -LiteralPath $Msi).ProviderPath
$hash=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
if((Get-Item -LiteralPath $path).Length -ne 316784640 -or $hash -cne '9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282'){throw 'MSI-only contract package bytes differ; retain original and review before running'}
$installer=$null;$database=$null;$view=$null
try{
 $installer=New-Object -ComObject WindowsInstaller.Installer
 $database=$installer.OpenDatabase($path,0)
 $q=[char]96
 $view=$database.OpenView(('SELECT {0}Property{0},{0}Value{0} FROM {0}Property{0}' -f $q));[void]$view.Execute()
 $properties=@{}
 while($row=$view.Fetch()){$properties[$row.StringData(1)]=$row.StringData(2)}
 [void]$view.Close()
 if($properties.ProductCode -cne '{67CF3D62-492D-4359-AAFE-8BE6FB17E647}' -or $properties.ProductName -cne 'OpenCodeX W1D Proof' -or $properties.ProductVersion -cne '0.2.24' -or $properties.ContainsKey('ALLUSERS')){throw 'MSI test identity/scope differs'}
 @{scope='read-only original MSI Product table; no installation';path=$path;sha256=$hash;bytes=(Get-Item -LiteralPath $path).Length;productCode=$properties.ProductCode;productName=$properties.ProductName;version=$properties.ProductVersion;allUsersPresent=$false}|ConvertTo-Json|Set-Content -LiteralPath $Output -Encoding utf8
}finally{
 if($view){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view)}
 if($database){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($database)}
 if($installer){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($installer)}
}
