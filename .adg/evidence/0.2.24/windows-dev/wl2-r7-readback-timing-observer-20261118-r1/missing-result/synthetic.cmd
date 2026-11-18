@echo off
if not "%~1"=="DiagnosticAfter" exit /b 91
echo invoked>>"%~dp0invoked.log"
exit /b 0
