@echo off
title OpenCodex H exact failure evidence
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0collect-h024-desktop.ps1" -Root "C:/Users/Public/ocx-h024-desktop-20261114-r1"
echo Failure evidence collection finished. The test was not restarted.
pause
