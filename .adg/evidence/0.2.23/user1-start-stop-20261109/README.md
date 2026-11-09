# user1 启动确认／停止单项回交

2026-11-09，Asia/Shanghai。accepted:false。依据[固定交接合同](../../../../docs/03-开发实施/0.2.23-contracts/user1-start-stop-handoff-20261109.md)执行；未跑core-sweep、双SID、安装器、全量、旧失败轮或发布，未改产品／维护包／判定。实际产品仅Start一次；启动未确证成功，正常Stop零次，未发失败后的补充Stop。

程序源8445d65dca0af76c3275f8617a76c1a726fb8873，复用构建42602603906/artifact13667461176；ZIP304962844字节，SHA256048b4d344c1bd8bbf81adb0a73571f14eaa697e122aafdad7ea6aeda7e42f274及manifest9成员全部核验。运行包2.97.0-desktop023.lifecycle.2，15413257字节/SHA256c02f4710496775d8f4dfe271bd90d6afdf7dbf960f6e696637271ab9f37bc70b。代码回交codex/0.2.23-integration@dc919da846260b0c11bebc04c56fd32963929d45，仅test/**装配与采集，apps/**及运行策略与8445完全相同。

实际执行批次[ocxu1-20261109-192154](ocxu1-20261109-192154/four-part-report.json)：现有Windows工作站EZIO-MINI，Windows11构建26100，RDP会话2，user1为独立临时非管理员组账户SID S-1-5-21-587408064-856152790-2583328096-1009，实际应用未提升/type1，原生窗口可见；sandbox:false，真实Profile C:/Users/ocxaccu1a66ac3cb9，私有程序位于该账户LocalAppData，未使用日常配置／Key或真实业务同步。环境详见该批environment.json和host-environment.json。

|交接项|本轮实际结果|原证据|
|---|---|---|
|①启动回执原文／数量／唯一性／PID端口|实际process_action原回执共1条、成功0条；唯一返回的是失败回执，无唯一成功回执。未观察到受保护runtime-port.json，无服务PID／端口可核对，不能将cmd／Node／Bun启动链PID当服务PID。|user1/action-001.raw.json原文；action-001.json编号；user1-result.json；event-order.json按原数组位置编号，不重排。|
|②启动trace／失败分类|首个产品失败在app.log第8行：8028ms实际启动进程exit_code=1/lifecycle_start_exit。cmd109876、Node49576、Bun38360的保留原句柄都在清理前退出，真实exit1。没有ocxd-lifetime-admission-failed或lifetime_admission的stage/kind/win32，归属原因证据不足，不能推断归属已通过或被拒绝。之后原30秒确认窗口30102ms未取得健康证明，仅为后续现象，不能改写此前exit1为单纯确认超时。|user1/desktop-logs/app.log:8；user1-result.json.trace；owned-health-observations.json；original-exits-before-cleanup.json。|
|③停止trace／真实退出码|启动未成功，正常Stop未执行、无Stop回执／Stop命令退出码。真实启动链exit1、诊断driver15936 exit1分别保存；后续Manager／WebView精确清理不冒充Stop或客户端还原成功。|user1/driver-exit.json；original-exits-before-cleanup.json；child-finished.json。|
|④账户／Profile／用户目录清理|实际user1账户、CIM Profile和原用户目录均删除，残留进程列表空，identityUnproven:false，桌面ACL恢复，封存程序摘要不变。原bootstrap exit0只表示执行器收齐诊断，不表示Start成功；诊断driver实际exit1。|controller-finished.json：accountRemoved/profileRemoved/userDirectoryRemoved/desktopAclRestored/programUnchanged均true；residualProcesses=[]。|

启动回执原文（对应文件，不是另一次调用）：

    {"lifecycleState":"starting_failed","result":null,"canStart":true,"canStop":false,"canRestart":false,"failure":{"code":"lifecycle_start_exit","message":"服务启动进程已退出，请查看诊断日志后重试。"}}

首个产品失败原日志：

    2026-11-09T11:24:20.348Z [manager] start: failed (managed runtime operation failed: lifecycle_start_exit; 官方启动进程退出，服务未确认就绪；exit_code=1; duration_ms=8028)

## 前置装配失败保留

- ocxu1-20261109-190955：宿主PowerShell可执行文件ACL没有新临时SID的RX，凭据启动拒绝访问，Desktop未启动，Start0次。账户SID…1007/Profile/用户目录全删除，ACL恢复。原失败与0回执独立保存。
- ocxu1-20261109-191803：复制同字节PowerShell到独立目录避免修改宿主日常ACL后，Profile初始化完成；新薄层脚本误用只读HOME局部变量，bootstrap报SessionStateUnauthorizedAccessException，Desktop未启动、Start0次。账户SID…1008/Profile/用户目录全删除。改名并通过PS AST／保留变量检查后才进行上面首次产品执行。

上述是新增薄层入口的前置失败，未复跑历史42604447388或任何旧产品失败轮；账户串行创建，前一个删除后才进入下一个，每批仅一个临时user1，不是双SID测试。失败目录及执行时实际代码副本全部保留，原事件回执不会用后继结果覆盖。这里只回交诊断，后续修复位置由Mac核对后决定，不授权产品返修／全量／发布。

封存ZIP、程序副本及同字节执行器工具作为复现输入保留在本机D:/ocxu1-20261109-*；账户/Profile/应用私有安装及其H均已清理，不把保留输入冒充残留服务。Git证据不含运行记录secret、真实Key、token或认证Header；每批evidence-manifest.json绑定文件大小与SHA256。
