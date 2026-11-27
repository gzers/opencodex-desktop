# TEAM84 Windows 普通桌面复验前置回执

合同固定在 docs/governance-main@3f1b7da702f9e255bcc1f3071d9cc49f3139da75。候选源码 e663369226337c9a9cad594ccd3cafd1719c3711，构建 45973489918，Windows 制品 15187611477，macOS 制品 15186409765。

## 待证伪判断

1. 完整包首开无法在新隔离根自行校验并解包，或者没有准备进度和「无需联网」提示。按 TEAM84 跳过断网桌面。
2. 缺包在主动确认前发出下载请求，或者暂缓后无法重新打开准备页。确认后同一次尝试应能关联 attemptId、downloadConfirmed、响应和准备终态。公开地址不可用需保留实际结果。
3. 环境就绪后首次 Status 超过 5 秒仍显示等待，或者未知时缺少重试、诊断或允许 Start。只有可信空闲才走 Start → Status → Stop，预算 30／5／10 秒。
4. 仅有 ocx.pid 时 Start 写入了端口或更改数据根。端口设置写锁超过原 Start 截止时间时，Start 改动原有记录、产生进程、监听或启动回执。两个拒绝测试各用独立夹具。

## 当前证据

| 项目 | 结果 | 证据与限制 |
| --- | --- | --- |
| 安装器身份和摘要 | pass | MSI bbe3880bb1779ce90641cdba40b756c9516c777733b458c4ccf2c2b55f099bab。NSIS 45cb6d27d20c1a60f1167b9e4641ed5e862bc39398f0c90a7cf9fa830c0467aa。原始校验记录见 [TEAM83 摘要](../team83-20261127-e66336922-desktop-r1/raw/digest-verification.json)。 |
| 断网桌面 | skipped | 用户确认跳过，RDP 保持连接。保留 TEAM83 原始状态。 |
| 完整包自动准备 | pass | Full 在 marenben 普通会话 3 使用新隔离根。随包归档存在，运行指针为 2.99.0／win32-x64，管理器记录准备校验完成。启动至该日志为 170.005 秒。[原始采集](raw/full-r2-collector-receipt.json) |
| 完整包准备进度和无需联网提示 | blocked | 普通桌面采集发生在环境就绪后，准备期间的界面待取证。 |
| 缺包首开拒绝 | pass | Missing 在独立隔离根启动，PID 40624。归档和运行指针缺席，首错是夹具预期的 bundle_archive_missing。北京时间 20:46:29 的采集复核了两处原进程与日志。[原始回执](raw/collect-20261127T1246298602949Z.json) |
| 缺包暂缓、重开、确认下载 | blocked | 同账户采集仅见首开拒绝。缺暂缓、重开、主动确认及同一次下载响应。[同次响应取证交接](download-attempt-observability-r1.md) |
| 首次 Status 耗时和未知界面 | blocked | Full 准备校验完成。170.005 秒是管理器启动到环境就绪的墙钟时间。普通桌面的首次请求耗时及终态缺席。冻结候选的现有阶段追踪仅编入 macOS。[Windows 取证交接](first-status-trace-readback-r1.md) |
| 重开后的未知界面 | pass | 同一 Full 隔离根，截图显示「运行状态未知」，提供重试和诊断，概览未显示启动入口。重开不计入新根首次 Status 计时。 |
| 正向启停 | blocked | 等待首次 Status 和可信空闲；未触发 Start。无保留 Owner 和记录缺席不能证明空闲。 |
| PID 文件拒绝副作用 | blocked | 同源组件测试已通过；普通桌面入口及实际副作用证据待收集。 |
| 写锁超时拒绝副作用 | blocked | 同源组件测试已通过；实机探针遇到工具审批拒绝，保留拒绝，原 30 秒实机断言未验证。 |
| 过程和清理 | blocked | 原 Full PID 77372 与 Missing PID 40624 已退出。重开的 Full PID 103156 仍在 marenben 会话 3，两处测试根保留取证。既有账户、Profile 和历史证据保留。 |

[同源普通账户 CI 证据](../team83-20261127-ci-45965743318-token-r1/result.md) 中的 750／250 毫秒组件窗口只覆盖组件行为。历史对照的产品首错是 bundle_archive_missing，采集首错是 IGraphicsCaptureItemInterop.CreateForMonitor 0x80070057。本轮普通账户首错为预期的 bundle_archive_missing，见同账户原始回执。Full 日志与前次采集逐字节相同，仍缺首次 Status 的耗时与终态。入口和预检见 raw/ordinary-desktop-preflight.json，载荷清单见 raw/staged-payload.json。

北京时间 21:11 的 [同账户复采](raw/collect-20261127T1311382265875Z.json) 确认 Full 与 Missing 的日志字节、进程身份、运行指针和两处原生记录状态与前次相同，采集无读错。仍需观察 Full 的状态和启动按钮，以及 Missing 的确认下载结果。

北京时间 22:41，marenben 配置 Node 后从同一 Full 根重开。[启动回执](raw/full-reopen-20261127T1441380754827Z-0c765d725c7748d493fb24ff4b2a7b98.json) 记录冻结候选镜像摘要、PID 103156 和 PATH 解析到 node.exe，未取得 Node 版本。[用户截图](raw/full-reopen-unknown-ui-user.png)记录概览进入「运行状态未知」，显示「重试状态」和「查看诊断」，概览无启动入口。22:47 的[只读采集](raw/collect-20261127T1447152648524Z.json) 核对运行指针仍为 2.99.0／win32-x64、两处原生状态文件均缺席、采集无读错。重开后的 Full 日志只有管理器启动与窗口状态记录，未捕获 Status 首错或请求耗时。空闲仍未确认，没有执行 Start。Missing 本次未重开。

accepted:false。暂停 W2、双 SID 和发布。
