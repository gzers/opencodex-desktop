# TEAM81 Windows 同源候选实机前置复核

2026-11-27（北京时间）。基于 `main@7580b0d79289864cb105e6d56d668b608b93495b`，包含 Windows 原生 `a288fc41457ffa682edb3b3df2e00500a61a5649`，治理引用 `3e1a978a294c5490388185bafef6fa822732a923`。`accepted:false`。本轮为单用户定向证据，W2／双 SID 暂停，未发布。

| 核验项 | 状态 | 原始证据与限制 |
| --- | --- | --- |
| 同源制品和摘要 | pass | 同一主线构建 `45863614604`，Windows `15141032029`、Mac `15139469228`。ZIP、MSI、NSIS 及两份 `.sig` 的复算见[摘要回执](raw/digest-verification.json)。 |
| MSI 管理解包 | pass | 首次正斜杠路径返回 `1619`，MSI 服务记录 `2343`；同一安装器改用 Windows 原生路径后退出 `0`，耗时 2,046 ms。[首错](raw/msi-admin.log) · [重试](raw/msi-admin-r2.log) |
| 完整与缺包夹具 | pass | 两个完整副本各 10 个文件；缺包副本只省略固定 `.tar.gz`，9 个保留文件的摘要逐项相同，固定清单与摘要文件保留。[载荷清单](raw/payload-inventory-r2.json) · [只读预检](raw/fixture-preflight.json) |
| 完整包离线自动准备、校验、进度 | blocked | 当前真实桌面为带默认路由的 RDP session 2，尚无可独立断网的交互桌面。离线夹具未启动。联网对照另用全新根。 |
| 缺包暂缓、重开、确认门槛及请求 | blocked | 新候选桌面尚未启动。独立 HEAD 请求返回 404，只证明预检时公网端点未返回资源；不是 GUI 请求的证据。[HTTP 预检](raw/release-asset-preflight.json) |
| 首次 Status／未知提示与禁用 Start | blocked | 尚无环境就绪后的真实 UI 请求；以请求发出计 5 秒。 |
| 确认空闲后的 Start → Status → Stop | blocked | 尚未确认空闲；未发出 Start。预算 30／5／10 秒。 |
| 清理 | pending | 本轮未启动候选进程或本地观察端口。三套全新夹具和候选文件留供续验，旧失败现场保留。 |

首错是管理解包测试入口的路径格式；已用同一制品复现修正后的成功。当前没有新候选的产品首错。无网络环境仍缺；联网 RDP 仅可做本地载荷与真实 GUI 对照。若缺包确认后实机证实已发请求但不能取包，集成线需提供同一固定摘要的未发布测试资源方案；不发版来解锁测试。

[待证伪判断](raw/hypotheses.md) · [Windows CI 回读](raw/ci-run-r2.json)。CI 的 Windows smoke 成功；Windows 后端在读取时仍在运行。
### Windows CI 与随包归档复核

同一候选的随包归档为 274,033,721 字节，SHA-256 与运行时描述符一致；描述符 SHA-256 同时匹配随包摘要文件，并可在候选管理器二进制中读回相同字节串。缺包副本保留同一描述符，只省略固定归档。[原始复核](raw/runtime-bundle-verification.json)。这项只读核验不证明离线运行或在桌面实际解包。

同一主线 Actions 45862219344 的 Windows 后端回归、实际 TokenUser 所有权及隔离原生凭据步骤均已通过。读取时 Windows 作业尚在执行托管安装升级／离线重装；整条工作流未完成，其他平台的 backend/frontend 已取消。仍以实机验收证明首次 Status 和生命周期。[步骤回执](raw/ci-run-r3.json)

2026-11-27 01:45（北京时间）复核：步骤 13 的官方托管安装升级／离线重装已通过（489 秒），步骤 14 已采集原生依赖诊断。此时 Windows 作业仍在构建 release 二进制，整体作业无终态。[原始作业回执](raw/ci-run-r6.json)。此项 CI 测试与候选随包环境的真实桌面离线准备分开记录；上表实机状态不变。

### 冻结源码的 Windows 前端定向验证

2026-11-27（北京时间），从同一固定 main SHA 导出 UI 与 Tauri 配置，使用 Windows Node 24.22.3 和 Vitest 5.0.0。4 个测试文件、30 项断言通过，包含准备进度、缺包暂缓与再次打开、仅在确认后发起重试 IPC，以及 Status 5 秒截止、迟到响应丢弃和未知状态呈现。[通过回执](raw/pinned-source-check-r2.log)。首次导出遗漏了相邻 Tauri 配置，4 个套件在载入时都未执行断言；补齐同一提交的配套源文件后通过。[测试夹具首错](raw/pinned-source-check-r1.log)。没有启动候选桌面或发送产品下载请求，实机状态不变。

定向测试副本及其复制的依赖仍在本机证据根的 pinned-source-check-r1 目录。自动审批以 blocked by policy 拒绝递归删除该副本，保留原位。三套候选 HOME 未使用，历史失败现场保留。

### Windows CI 终态与冒烟首错

Actions 45862219344 的 Windows 作业 140055878175 已结束。MSVC release 编译和原始 EXE 上传通过；步骤 18 Windows final EXE smoke 失败，后续原生 Node 矩阵与普通用户冒烟跳过。[作业终态](raw/ci-windows-terminal.json)

EXE 在固定 30 秒观察后主窗口存在并响应，已观测到未提升令牌和 WebView2 的回环监听。首个失败断言为 `Sandbox startup log missing; see sandbox-file-inventory.json`（windows-startup.ps1:126）。随后应执行的源码 commit 核验及 WebView 断言均未执行。单独的进程 stderr 记录托盘外观刷新拒绝访问（os error 5）。[启动回执](raw/ci-final-exe-startup.json) · [进程和监听](raw/ci-final-exe-startup-diagnostics.json) · [首错](raw/ci-final-exe-ordinary-stderr.log) · [进程 stderr](raw/ci-final-exe-startup-stderr.log)

原始诊断制品 15141979226，ZIP SHA-256 为 f5ed4f85a76979c86f1560d76fb53ec06d95bc49c5ed7c47f3791a4d0bf6126a。制品内缺少 sandbox-file-inventory.json，公开作业日志端点返回 403。待证伪判断是冒烟脚本仅把 0.2.23 定位到 data/desktop，而 0.2.24 的产品启动测试中同样要求该根。后续须重验真实 CI；此作业使用独立 release EXE，TEAM81 实机状态表继续有效。

### Windows 冒烟脚本的日志路径与后继断言

Windows 分支 8a5f00a4ddb35a35baa55550935ceb65677899a4 仅修改两个 Windows PowerShell 冒烟脚本，0.2.24 从沙箱的 data/desktop 读取日志及数据根登记。冻结候选的产品文件字节保持一致。0.2.22、0.2.23、0.2.24 各两项路径断言通过；进程令牌脚本九项通过。[路径](raw/ci-startup-layout-check.json) · [令牌](raw/ci-startup-token-check.json)

分支 Actions 45909098994 的 Windows 原生工具作业通过。后端还在 Clippy，尚未执行最终 EXE 冒烟。[作业回执](raw/ci-layout-branch-progress.json)

待证伪判断为 0.2.24 的 Windows WebView 脚本仍会采用旧目录和版本入口。首个只读探针误用了不符合接纳条件的路径，改用固定绝对 Windows 路径后，旧根断言拒绝 data/desktop，正确根通过；旧驱动和仅支持 0.2.23 的驱动均拒绝 0.2.24。该探针未启动 EXE，真实 CI 的后继首错待确认。[首次探针](raw/ci-startup-legacy-probe.json) · [更正夹具回执](raw/ci-startup-legacy-probe-r2.json)

### 0.2.24 启动观察接线与实机续验读回

Windows 分支 `ea6f8fcd04d501a38cb3b1f7851bbc29651ffb56` 让启动冒烟从显式沙箱读取生产数据根和运行准备快照。仅接纳 `code:14` 且原因为 `discovery_runtime_unresolved` 或 `discovery_runtime_pending` 的未就绪拒绝。0.2.24 回执范围为 `startup-observation`。首轮 Node 定向 29 过、1 失败（旧测试把调用的空格写死），修正后 30 项全过。产品源码树与固定主线一致。[首轮原日志](raw/ci-webview-directed-r1.log) · [通过原日志](raw/ci-webview-directed-r2.log)

2026-11-27 07:20（北京时间）复核：Actions 45909098994 的 Windows smoke 工具作业成功，Windows 后端回归仍在执行。Actions 45910157474 的 Windows 最终 EXE 冒烟尚未执行。步骤 20 和 22 的安装恢复仍排在其后，现有脚本针对旧布局和 2.91／2.93，待真实结果确认后按 0.2.24 生产入口处理。[旧分支步骤回执](raw/ci-layout-branch-progress-r2.json) · [旧分支最新作业](raw/ci-followup-45909098994.json) · [新分支最新作业](raw/ci-followup-45910157474.json)

07:21 回读，三个隔离根均只有 program 子目录，无启动回执，按本轮固定镜像路径查询也未发现进程。RDP 所在主机依然有默认路由，完整离线验证仍缺独立离线桌面。Windows 的 OCXD024_STATUS_TRACE 在该固定源码中是空实现，环境准备快照只能证明准备状态。实机 Status 仍须采集从首个 get_status_snapshot 请求发出至 UI 得出结论的时间，未确认空闲前禁止 Start。三个夹具留待续验。[本机只读回执](raw/interactive-readback-r2.json)

### 真实 Windows 桌面续验

RDP session 2 中使用同一固定 EXE 和两个全新隔离根。断线后读回原进程，未重启候选。以下结果取代上表相应前置状态。

| 核验项 | 结果 | 证据与限制 |
| --- | --- | --- |
| 完整包联网对照自动准备 | pass | 后端自启动到供应完毕 65.801 秒。界面曾显示归档校验完成、正在展开以及「无需联网」。[应用日志](raw/desktop-r1/full-online-app.log) · [准备进度](raw/desktop-r1/full-online-uia-progress.txt) |
| 完整包真实离线准备 | blocked | 当前 RDP 主机有默认路由，断网夹具未启动。仍缺可独立断网的桌面。[现场读回](raw/desktop-r1/readback.json) |
| 缺包提示和初始启动门槛 | pass | 程序保留原件并报告预期的 bundle_archive_missing，界面有下载、查看安装配置、稍后处理，运行控件禁用。[应用日志](raw/desktop-r1/missing-app.log) · [缺包界面](raw/desktop-r1/missing-active-r1.txt) |
| 缺包暂缓、重开、确认后发出下载 | blocked | 首次自动操作在断线时返回 GetCursorPos 拒绝访问 0x80070005。重连后元素点击无坐标，截图点击返回 unknown screenshotId；再次读回仍为原缺包页。没有确认下载。[元素点击](raw/desktop-r1/missing-defer-error-active-r1.txt) · [截图点击](raw/desktop-r1/missing-defer-error-active-r2.txt) · [读回](raw/desktop-r1/missing-after-coordinate-r2.txt) |
| 环境就绪后的 Status 未知界面 | pass | 重连后实测显示「运行状态未知」「重试状态」「查看诊断」，无 Start 入口。前次采样在后端完成后仍显示展开，后续同一窗口已退出准备页。[旧快照](raw/desktop-r1/full-online-provisioned.txt) · [重连快照](raw/desktop-r1/full-online-active-r1.txt) |
| 首次 Status 耗时与首个失败阶段 | blocked | 固定候选在 Windows 未记录首次 IPC 起点、结束或超时原因。未知界面证明门槛生效，无法从 65.801 秒供应时间或断线期间的快照判定 Status 是否耗满 5 秒。 |
| 确认空闲后的 Start → Status → Stop | blocked | 当前未确认空闲，未调用 Start。预算 30／5／10 秒。 |
| 清理 | pending | 两个候选原窗口和本轮隔离树保留续验，历史失败保留。 |

Windows 后继 CI 两个作业已通过后端回归和原生凭据步骤，正在执行托管升级／离线重装；最终 EXE 冒烟和恢复断言待跑。[作业读回](raw/desktop-r1/ci-job-readback.json)。整版 accepted:false，W2／双 SID 暂停，未发布。
