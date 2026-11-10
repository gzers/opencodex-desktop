param([Parameter(Mandatory)][string]$Plan,[Parameter(Mandatory)][string]$Digest,[Parameter(Mandatory)][string]$Journal,[switch]$Recover,[int]$CrashAfter=0)
$ErrorActionPreference='Stop'
Add-Type -Path (Join-Path $PSScriptRoot Native.cs),(Join-Path $PSScriptRoot Recovery.cs),(Join-Path $PSScriptRoot FiveRecovery.cs)
$ops=[Official299Proof.FiveRecovery]::Apply($Plan,$Digest,$Journal,[bool]$Recover,0,$CrashAfter,$null)
$ops|ConvertTo-Json
