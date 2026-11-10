param([Parameter(Mandatory)][string]$EvidenceRoot,[Parameter(Mandatory)][string]$PreparationTempRoot)
& (Join-Path $PSScriptRoot cleanup-payloads.ps1) -EvidenceRoot $EvidenceRoot -PreparationTempRoot $PreparationTempRoot -AttemptKeys @('attempt-r5-01') -PreparationLeaves @('ocxd299-r5-official','ocxd299-r5-deps')
