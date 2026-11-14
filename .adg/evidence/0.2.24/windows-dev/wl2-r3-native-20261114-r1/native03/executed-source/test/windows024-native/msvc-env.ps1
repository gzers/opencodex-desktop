param([Parameter(Mandatory)][string]$Tools,[Parameter(Mandatory)][string]$RustRoot)
$ErrorActionPreference='Stop'
$toolsRoot=(Resolve-Path -LiteralPath $Tools).ProviderPath
$rustRootPath=(Resolve-Path -LiteralPath $RustRoot).ProviderPath
$vc=@(Get-ChildItem -LiteralPath (Join-Path $toolsRoot 'VC/Tools/MSVC') -Directory)
$sdk=@(Get-ChildItem -LiteralPath (Join-Path $toolsRoot 'Windows Kits/10/Include') -Directory)
if($vc.Count -ne 1 -or $sdk.Count -ne 1){throw 'one explicit MSVC/SDK version required'}
$vcRoot=$vc[0].FullName;$sdkVersion=$sdk[0].Name
$sdkRoot=Join-Path $toolsRoot 'Windows Kits/10'
$env:CARGO_HOME=Join-Path $rustRootPath 'cargo'
$env:RUSTUP_HOME=Join-Path $rustRootPath 'rustup'
$env:VCToolsInstallDir=$vcRoot+[IO.Path]::DirectorySeparatorChar
$env:VCINSTALLDIR=(Join-Path $toolsRoot 'VC')+[IO.Path]::DirectorySeparatorChar
$env:WindowsSdkDir=$sdkRoot+[IO.Path]::DirectorySeparatorChar
$env:WindowsSDKVersion=$sdkVersion+[IO.Path]::DirectorySeparatorChar
$env:VSCMD_ARG_HOST_ARCH='x64';$env:VSCMD_ARG_TGT_ARCH='x64'
$env:INCLUDE=(@((Join-Path $vcRoot 'include'))+@('ucrt','shared','um','winrt','cppwinrt'|ForEach-Object {Join-Path $sdk[0].FullName $_})) -join ';'
$env:LIB=@((Join-Path $vcRoot 'lib/x64'),(Join-Path $sdkRoot "Lib/$sdkVersion/ucrt/x64"),(Join-Path $sdkRoot "Lib/$sdkVersion/um/x64")) -join ';'
$env:PATH=@((Join-Path $rustRootPath 'cargo/bin'),(Join-Path $vcRoot 'bin/Hostx64/x64'),(Join-Path $sdkRoot "bin/$sdkVersion/x64"),$env:PATH) -join ';'
$env:CC_x86_64_pc_windows_msvc='cl.exe'
$env:CXX_x86_64_pc_windows_msvc='cl.exe'
$env:AR_x86_64_pc_windows_msvc='lib.exe'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=Join-Path $vcRoot 'bin/Hostx64/x64/link.exe'
if(!(Test-Path -LiteralPath $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER) -or !(Test-Path -LiteralPath (Join-Path $sdk[0].FullName 'ucrt/assert.h'))){throw 'real Microsoft compiler/linker/SDK headers required'}
