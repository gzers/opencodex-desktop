@echo off
if not "%~1"=="DiagnosticAfter" exit /b 91
echo invoked>>"%~dp0invoked.log"
powershell.exe -NoLogo -NoProfile -Command "Start-Sleep -Milliseconds 5100"
copy /b "%~dp0synthetic.json" "%~dp0DiagnosticAfter-result.json" >nul
exit /b 0
