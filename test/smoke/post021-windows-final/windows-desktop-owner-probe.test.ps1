$ErrorActionPreference='Stop'
Add-Type @'
using System;
using System.Collections.Generic;
public static class OcxDesktopOwnerV1 {
 public sealed class Identity {public string Path;public string Sid;public long CreatedUtcTicks;}
 public static Dictionary<int,Identity> Values=new Dictionary<int,Identity>();
 public static Identity Read(int pid){if(pid==99)throw new InvalidOperationException("unknown owner");return Values.ContainsKey(pid)?Values[pid]:null;}
}
'@
# The test-only native type is already defined. The production script itself
# remains unchanged, and the real native probe is tested in a separate PS5 run.
function Add-Type {param($TypeDefinition)}
function Get-CimInstance {param($ClassName,$Filter,$ErrorAction);return $script:taskRows}
$taskInstallRoot='D:\Fixture\OpenAI.Codex'
$taskTicks=([datetime]'2026-11-05T08:00:00Z').ToUniversalTime().Ticks
$taskSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
function Set-Case($Rows,$Native){
 $script:taskRows=$Rows;[OcxDesktopOwnerV1]::Values.Clear()
 foreach($item in $Native){$v=New-Object OcxDesktopOwnerV1+Identity;$v.Path=$item.path;$v.Sid=$item.sid;$v.CreatedUtcTicks=$item.ticks;[OcxDesktopOwnerV1]::Values.Add($item.pid,$v)}
}
function Row([int]$Number,[string]$Path){[pscustomobject]@{ProcessId=$Number;ParentProcessId=1;ExecutablePath=$Path;CreationDate=[datetime]::new($taskTicks,[DateTimeKind]::Utc);SessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId}}
function Native([int]$Number,[string]$Path,[string]$Sid=$taskSid,[long]$Ticks=$taskTicks){@{pid=$Number;path=$Path;sid=$Sid;ticks=$Ticks}}
$taskImage=Join-Path $taskInstallRoot 'ChatGPT.exe'
$taskScript=Join-Path $PSScriptRoot 'windows-desktop-owner-probe.ps1'
Set-Case @((Row 10 $taskImage),(Row 11 $taskImage)) @((Native 10 $taskImage),(Native 11 $taskImage 'S-1-5-18'))
$taskResult=@(. $taskScript);if($taskResult.Count -ne 1 -or $taskResult[0] -notmatch '^10 1 '){throw 'Current owner filter failed'}
Set-Case @((Row 10 ($taskInstallRoot+'-evil	estook.exe'))) @()
if(@(. $taskScript).Count -ne 0){throw 'Sibling path was admitted'}
Set-Case @((Row 10 $taskImage)) @()
if(@(. $taskScript).Count -ne 0){throw 'Gone process was not ignored'}
$taskNegatives=@(
 @{rows=@((Row 10 $taskImage));native=@((Native 10 $taskImage $taskSid ($taskTicks+10)))},
 @{rows=@((Row 10 $taskImage));native=@((Native 10 ($taskInstallRoot+'-evil	estook.exe')))},
 @{rows=@((Row 10 $taskImage));native=@((Native 10 (Join-Path $taskInstallRoot 'Other.exe')))},
 @{rows=@((Row 10 $taskImage),(Row 99 $taskImage));native=@((Native 10 $taskImage))},
 @{rows=@((Row 10 ''));native=@()}
)
foreach($taskCase in $taskNegatives){
 Set-Case $taskCase.rows $taskCase.native
 $taskFailed=$false;$taskPartial=@()
 try{$taskPartial=@(. $taskScript)}catch{$taskFailed=$true}
 if(!$taskFailed -or $taskPartial.Count){throw 'Unsafe native identity returned partial success'}
}
Write-Output 'Owner probe policy passed: owner/SID, sibling boundary, gone, generation change, native path/image mismatch, unknown owner, no partial stdout'
