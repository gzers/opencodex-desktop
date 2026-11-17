@echo off
powershell.exe -NoLogo -NoProfile -File "%~dp0archive-h024-fixed-success.ps1"
echo.
echo Archive and cleanup attempt finished. Inspect retained receipt.
pause
