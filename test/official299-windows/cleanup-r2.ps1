param([Parameter(Mandatory)][string]$EvidenceRoot,[Parameter(Mandatory)][string]$PreparationTempRoot)
& (Join-Path $PSScriptRoot 'cleanup-payloads.ps1') -EvidenceRoot $EvidenceRoot -PreparationTempRoot $PreparationTempRoot -AttemptKeys @('attempt-r2-01','attempt-r2-02')
