@echo off
title OpenCodex completed H task cleanup
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0cleanup-h024-completed.ps1"
echo Cleanup receipt retained. Original failed scene and account have not been removed.
pause
