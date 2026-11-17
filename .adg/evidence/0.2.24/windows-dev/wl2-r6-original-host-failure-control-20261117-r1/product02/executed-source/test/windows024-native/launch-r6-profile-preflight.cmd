@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0launch-r6-profile-preflight.ps1" -Root "%~dp0."
echo.
echo Preparation finished. Keep the ordinary desktop open and report completion.
pause
