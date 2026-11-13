@echo off
"%SystemRoot%\System32\chcp.com" 65001 >nul
setlocal DisableDelayedExpansion
"C:\Users\ocxw1c83d4b3a16\w1c native\I 空格 & !\runtime\versions\9.0.0\dependencies\node\node.exe" "C:\Users\ocxw1c83d4b3a16\w1c native\I 空格 & !\runtime\versions\9.0.0\opencodex\node_modules\@bitkyc08\opencodex\probe.cmd" %*
exit /b %errorlevel%
