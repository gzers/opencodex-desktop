@echo off
"%SystemRoot%\System32\chcp.com" 65001 >nul
setlocal DisableDelayedExpansion
"C:\Users\ocxw1c67eb51e2b\w1c native\I 空格 & !\runtime\versions\9.0.0\dependencies\node\node.exe" "C:\Users\ocxw1c67eb51e2b\w1c native\I 空格 & !\runtime\versions\9.0.0\opencodex\node_modules\@bitkyc08\opencodex\probe.cjs" %*
exit /b %errorlevel%
