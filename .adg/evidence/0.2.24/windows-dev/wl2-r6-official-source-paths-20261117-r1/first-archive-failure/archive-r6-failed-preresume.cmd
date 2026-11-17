@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0archive-r6-failed-preresume.ps1" -Root "%~dp0."
echo.
echo Read-only archive finished. No process was stopped or test rerun.
pause
