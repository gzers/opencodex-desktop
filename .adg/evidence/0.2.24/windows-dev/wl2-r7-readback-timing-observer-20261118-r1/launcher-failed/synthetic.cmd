@echo off
if not "%~1"=="DiagnosticAfter" exit /b 91
echo invoked>>"%~dp0invoked.log"
copy /b "%~dp0synthetic.json" "%~dp0DiagnosticAfter-result.json" >nul
exit /b 17
