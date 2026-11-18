@echo off
powershell.exe -NoLogo -NoProfile -File "%~dp0launch-r7-partial-owner.ps1" %*
exit /b %errorlevel%
