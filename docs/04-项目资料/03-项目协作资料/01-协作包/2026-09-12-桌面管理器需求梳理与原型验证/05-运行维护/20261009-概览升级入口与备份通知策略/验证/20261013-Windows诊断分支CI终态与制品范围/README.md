# Windows 诊断分支CI终态与制品范围

2026-10-13 14:00:39（北京时间）重新通过已认证API核验`feature/0.1.10-update-center-focus@be7395acfce0ec1e36b83300b576db408f8c35bb`的精确push run38352525092为completed/success。原失败run38349316806/d975b23和本机两组完整主库失败对照均保留，不能用本次成功覆盖历史失败或声称原故障已定位。

原件归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-DIAGNOSTIC-CI/`，41份全集合/字节摘要冻结重验通过。

## 同SHA的实际CI结果

frontend、backend、backend-windows、release-tools、windows-smoke-evidence五项均success。Windows的前端构建、fmt、Clippy、完整后端回归、release构建及最终EXE smoke均success；仅main执行的macOS build在feature上skipped，保留该状态。

Windows主库505 passed/0 failed/4 ignored；日志的34条cargo结果合计601 passed/0 failed/7 ignored，含各integration/空目标等，不把ignored计入通过。原失败的三个版本进程树场景本次均通过。

新CI宿主实际4逻辑处理器、RUST_TEST_THREADS/RUST_TEST_TIME_UNIT未设置、Node24.21.0、Windows26100，镜像20260925.250.1。测试通过时Rust捕获并隐藏其stderr，因此日志没有新增阶段耗时，不能从CI success倒推原失败处的Node启动/恢复线程或文件就绪原因。诊断源码预算和并发未改，原因仍未隔离。

## 制品与smoke身份

同run制品11792440963的ZIP SHA256为`3fae1e04e42f85ee681356c27464fbf6e4a93bd8d50ecaa612874fe385eca808`，匹配API digest；165项路径/重复/大小和CRC检查通过，选取11份原件逐字保存。测试EXE SHA`bcb9d346a829da51b50b4b85ae32258488e66aa9c25d5868dace0d3a21276a76`独立匹配CI测试摘要。

CI最终桌面EXE的smoke摘要为`8651ba8375835ca22802bfc2d6192da588f184be5a2395d0f417182db3c8c251`，startup.expectedCommit及沙箱app.log均匹配be7395a；实际运行30秒、主HWND存在且响应、移除HOME、沙箱初始化及overview首屏挂载通过。

startup实际应用令牌elevated=true/elevationType1，但模式any/applicationModeMatched=false；这是真实CI宿主令牌观察，不能抵本机普通/管理员配对门禁。CDP version端点一次超时保留，后续list目标及overview检查成功；不删除部分失败诊断。

包内**没有最终桌面EXE或安装器**，只有测试EXE、回执和启动证据，不能以回执摘要声称本机已有可运行的修复制品。归档图经查看为runner的overview WebView帧：缺少Node.js、其余项待检查；它不是物理本机DPI/云母/标题栏验收，也没有更新详情焦点复验。未触发候选打包、安装、发布或替换原周期实例。

## 本机普通诊断阶段样本

仅提取验证后的测试EXE到隔离软件目录，按白名单HOME/AppData/temp、普通真实false/type3、Node24.21运行一次7线程`windows_tests::`：9 passed/0 failed/0 ignored/500 filtered，其中7项进程树及2项Windows路径。三个版本场景均取得runtime-ready、spawned、assigned-and-resumed、waited-and-terminated的完整四阶段，共12条日志。

本次assigned-and-resumed实际376.356/382.1924/381.0161ms，版本成功568.0681ms、非零557.5246ms、超时2.0033878s。原2秒预算、4秒收尾及fixture不改，没有外层cleanup；这个局部成功证明诊断输出可用，不抵原runner故障复现、全库负载或版本原生门禁。已知本轮fixture命令路径没有Node残留，不扩大为全机或网络计数结论。

原eae普通应用37276/监督器24300/周期监测3920的真实保留句柄与创建时间保持，日常EXE摘要及偏好/根绑定/备份策略/调度四项字节不变；周期快照182份/0观察失败、原截止2026-10-13 22:55:39/40不变。CI watcher38920/exec30799在14:00:42正常观察终态后退出；不是原周期终态，不重启该观察器。

main当前仍eae，焦点特性/诊断尚未集成；原故障根因和最终同SHA制品原生验证继续开放。TASK182原件、FZ50显式脚本许可、安全安装/独立Profile、物理DPI/管理员矩阵、配对性能、真实周期和历史failed均保留。12:50后无前台输入，晚间须收到人的新开始信号；IMP in_progress，无正式RCP/关闭证书，不降低门禁、不关闭或发布。
