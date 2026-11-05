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
function Get-CimInstance {
 param($ClassName,$Filter,$ErrorAction)
 if($script:caseTopFailure){throw 'CIM unavailable'}
 if($Filter){return $script:caseRequery}
 return $script:caseRows
}
$taskBasenameMatch='(?i)(?:^|[\\/])codex\.exe(?:\s|$)'
$taskCodeModeMatch='(?i)codex-code-mode-host'
$caseTicks=([datetime]'2026-11-05T08:00:00Z').ToUniversalTime().Ticks
$caseSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
function Row([int]$Number,[string]$Command='D:\Fixture\codex.exe app-server'){
 [pscustomobject]@{ProcessId=$Number;ExecutablePath='D:\Fixture\codex.exe';CommandLine=$Command;CreationDate=[datetime]::new($caseTicks,[DateTimeKind]::Utc)}
}
function Native([int]$Number,[string]$Sid=$caseSid,[long]$Ticks=$caseTicks,[string]$Path='D:\Fixture\codex.exe'){@{pid=$Number;sid=$Sid;ticks=$Ticks;path=$Path}}
function Set-Case($Rows,$Native,$Requery=$null){
 $script:caseRows=$Rows;$script:caseRequery=$Requery;$script:caseTopFailure=$false
 [OcxDesktopOwnerV1]::Values.Clear()
 foreach($item in $Native){$v=New-Object OcxDesktopOwnerV1+Identity;$v.Path=$item.path;$v.Sid=$item.sid;$v.CreatedUtcTicks=$item.ticks;[OcxDesktopOwnerV1]::Values.Add($item.pid,$v)}
}
$caseScript=Join-Path $PSScriptRoot 'windows-appserver-owner-probe.ps1'
Set-Case @((Row 10),(Row 11),(Row 12 'D:\Fixture\hermes-codex-bridge-mcp.exe')) @((Native 10),(Native 11 'S-1-5-18'),(Native 12))
$caseResult=@(. $caseScript)
if($caseResult.Count -ne 1 -or $caseResult[0] -notmatch '^10\s'){throw 'Current SID or narrow candidate filtering failed'}
Set-Case @((Row 10)) @()
if(@(. $caseScript).Count){throw 'Gone process admitted'}
Set-Case @((Row 99)) @()
if(@(. $caseScript).Count){throw 'Disappeared process must not block query'}
$caseReused=Row 99;$caseReused.CreationDate=$caseReused.CreationDate.AddSeconds(1)
Set-Case @((Row 99)) @() $caseReused
if(@(. $caseScript).Count){throw 'Reused process must not be admitted'}
$caseNegatives=@(
 @{rows=@((Row 10));native=@((Native 10 $caseSid ($caseTicks+10)))},
 @{rows=@((Row 10));native=@((Native 10 $caseSid ($caseTicks-1)))},
 @{rows=@((Row 10));native=@((Native 10 $caseSid $caseTicks 'D:\Wrong\codex.exe'))},
 @{rows=@((Row 10),(Row 99));native=@((Native 10));requery=(Row 99)}
)
foreach($case in $caseNegatives){
 Set-Case $case.rows $case.native $case.requery
 $caseResult=@(. $caseScript)
 if($caseResult.Count -ne 1 -or $caseResult[0] -cne '__OCX_ENUM_INCOMPLETE__'){throw 'Unknown identity must emit only sentinel, never partial rows'}
}
Set-Case @() @();$script:caseTopFailure=$true
if(@(. $caseScript)[0] -cne '__OCX_ENUM_INCOMPLETE__'){throw 'Top-level query failure lost'}
Write-Output 'App-server probe policy passed: SID, narrow match, gone/reused, generation/path, live unknown, top-level failure, no partial output'
