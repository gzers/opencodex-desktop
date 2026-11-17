@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0cleanup-r6-authorized-processes.ps1" -Root "%~dp0."
set "cleanup_rc=%errorlevel%"
echo.
if "%cleanup_rc%"=="0" (echo Authorized forced cleanup verified.) else (echo Cleanup REFUSED or incomplete. Do not rerun. See out01\cleanup-result.json.)
echo All original files and archives were retained. This is not NormalStop.
pause
exit /b %cleanup_rc%
