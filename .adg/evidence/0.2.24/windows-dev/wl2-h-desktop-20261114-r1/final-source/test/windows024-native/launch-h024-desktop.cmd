@echo off
title OpenCodex H native component test
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0launch-h024-desktop.ps1" -Root "C:/Users/Public/ocx-h024-desktop-20261114-r1"
echo Test records remain in this task's out01 directory. No official runtime was requested.
pause
