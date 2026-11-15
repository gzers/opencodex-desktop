@echo off
title OpenCodex H repaired component verification
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0repair-h024-desktop.ps1"
echo Exact empty-host recovery and verification records have been retained. No official service was requested.
pause
