@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0archive-r7-current-r2.ps1" -Root "%~dp0."
set "archive_rc=%errorlevel%"
echo.
if "%archive_rc%"=="0" (echo Read-only archive and native observations verified.) else (echo Read-only collection has conflicts. Do not rerun. See out01\archive-result.json.)
echo No process was stopped, resumed or repaired. Keep the ordinary desktop open.
pause
exit /b %archive_rc%
