@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0launch-r6-official-existing-r2.ps1" -Root "%~dp0."
set "qualification_rc=%errorlevel%"
echo.
if "%qualification_rc%"=="0" (echo Native qualification sequence passed. Public Start remains refused.) else (echo Qualification refused or failed. Do not rerun. See out01\launch-result.json.)
echo Keep the ordinary desktop open. Original files and broker are retained.
pause
exit /b %qualification_rc%
