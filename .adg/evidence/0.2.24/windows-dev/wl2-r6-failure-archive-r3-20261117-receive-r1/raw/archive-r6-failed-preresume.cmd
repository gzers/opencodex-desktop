@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -File "%~dp0archive-r6-failed-preresume.ps1" -Root "%~dp0."
set "archive_rc=%errorlevel%"
echo.
if "%archive_rc%"=="0" (echo Archive verified successfully.) else (echo Archive FAILED. See out01\archive-result.json.)
echo No process was stopped or test rerun.
pause
exit /b %archive_rc%
