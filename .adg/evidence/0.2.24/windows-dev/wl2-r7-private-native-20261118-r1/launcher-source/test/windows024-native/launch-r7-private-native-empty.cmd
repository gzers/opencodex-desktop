@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0launch-r7-private-native-empty.ps1" -Root "%~dp0."
echo Native V2 empty task exit code: %ERRORLEVEL%
pause
