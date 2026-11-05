# Windows PowerShell5.1采集器解析错误与修复

用户11:33确认管理员就绪后，重新绑定新MSVC测试PID52416、session2，PlanOnly在原PS7宿主成功，原30秒静止采样器等待600秒。用户实际管理员WindowsPowerShell执行RUN-ADMIN时，在collect-gpu-trace.ps1 line100出现UnexpectedToken An及缺右括号，未进入采集。该失败是本线采集工具兼容性遗漏，不能归为用户未配合，也不能算产品性能结果。

根因：脚本UTF8无BOM含中文regex，WindowsPowerShell5.1用ANSI解析时破坏了词法边界。修复aa52a17b201de7df1c9a4feee608047839d6a522使整份脚本ASCII化，中文状态以regex Unicode转义匹配，保留WPR未知状态拒绝和独立实例命名。已推送，产品apps树未改。

实际使用C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe（5.1.26100.33438）解析并运行PlanOnly，返回exit0/result plan-validated-no-recording，仍精确对应活PID52416；没有录制。初次验证子进程继承PS7module path导致Get-FileHash缺失，此验证失败日志保留；仅对子进程PSModulePath使用WindowsPowerShell内建模块路径后通过，未修改机器/用户持久配置。

修复后曾请用户重跑同一入口，但至本地11:44:37.247未检测到capture成功启动，采样器600秒到期exit1，未采样、无ETL。11:44:57核对WPR not recording及无collector-started标记后，按target.json原PID/路径/createUTCticks/session/owner及原生句柄时间核对停止52416，入口改为立即拒绝过期身份；原入口保留。修复后用户是否重跑及输出仍待答复，不凭无本机标记推断其操作。正式性能修复/默认Codex隔离重启继续未完成，TASK in_progress/RCP failed，无CERT。
