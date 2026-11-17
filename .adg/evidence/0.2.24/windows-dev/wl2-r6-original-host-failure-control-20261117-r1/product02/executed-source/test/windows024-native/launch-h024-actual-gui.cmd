@echo off
powershell.exe -NoLogo -NoProfile -File "%~dp0launch-h024-actual-gui.ps1" -Root "%~dp0."
echo.
echo Actual GUI probe finished. Original logs and task scene were retained.
pause
