# TEAM81 Windows 同源候选实机前置复核

2026-11-27（北京时间）。基于 `main@7580b0d79289864cb105e6d56d668b608b93495b`，包含 Windows 原生 `a288fc41457ffa682edb3b3df2e00500a61a5649`，治理引用 `3e1a978a294c5490388185bafef6fa822732a923`。`accepted:false`。本轮为单用户定向证据，W2／双 SID 暂停，未发布。

| 核验项 | 状态 | 原始证据与限制 |
| --- | --- | --- |
| 同源制品和摘要 | pass | 同一主线构建 `45863614604`，Windows `15141032029`、Mac `15139469228`。ZIP、MSI、NSIS 及两份 `.sig` 的复算见[摘要回执](raw/digest-verification.json)。 |
| MSI 管理解包 | pass | 首次正斜杠路径返回 `1619`，MSI 服务记录 `2343`；同一安装器改用 Windows 原生路径后退出 `0`，耗时 2,046 ms。[首错](raw/msi-admin.log) · [重试](raw/msi-admin-r2.log) |
| MSI 当前用户组件登记与卸载动作顺序 | pass | 固定制品包表只读核验，实机卸载仍缺证据。[包表回执](raw/msi-candidate-database.json) |
| 完整与缺包夹具 | pass | 两个完整副本各 10 个文件；缺包副本只省略固定 `.tar.gz`，9 个保留文件的摘要逐项相同，固定清单与摘要文件保留。[载荷清单](raw/payload-inventory-r2.json) · [只读预检](raw/fixture-preflight.json) |
| 完整包联网对照 | pass | 全新隔离 HOME 自动准备，用时 65.801 秒。显示归档校验、展开进度和「无需联网」。[应用日志](raw/desktop-r1/full-online-app.log) · [进度](raw/desktop-r1/full-online-uia-progress.txt) |
| 完整包真实离线自动准备 | blocked | 当前 RDP 主机有默认路由，缺可独立断网并保持交互的测试桌面。[只读环境预检](raw/desktop-r2/offline-host-preflight.json) |
| 缺包提示、暂缓、重开和下载确认门槛 | pass | 同一窗口通过暂缓和重开。仅在用户确认后进入下载 HTTP 响应处理。[暂缓](raw/desktop-r2/missing-deferred.txt) · [重开](raw/desktop-r2/missing-reopened.txt) · [确认前](raw/desktop-r2/missing-confirmation-before.json) · [确认后](raw/desktop-r2/missing-confirmation-after.log) |
| 缺包下载、校验及安装 | blocked | `bundle_download_unavailable`，官方地址未提供完整运行资源。本次 GET 状态码未记录。五项未执行，原件保留，运行控件禁用。[应用日志](raw/desktop-r2/missing-confirmation-after.log) · [界面](raw/desktop-r2/missing-confirmation-after-ui.txt) · [文件清单](raw/desktop-r2/missing-confirmation-after.json) |
| 运行状态未知、重试、诊断及 Start 禁用 | pass | 环境准备完成后，真实窗口显示未知、重试和诊断，未提供 Start 动作。[界面](raw/desktop-r1/full-online-active-r1.txt) |
| 首次 GUI Status 耗时和首个失败阶段 | blocked | 固定候选未采集首次 GUI 请求起止。原生探针在 124 µs 返回 `windows024_owner_unknown`。完整原生采样用时 303,528 µs，结果为 `Unreachable`。[原生探针](raw/desktop-r2/status-owner-probe-msvc.log) |
| 确认空闲后的 Start → Status → Stop | blocked | 未确认空闲，未调用 Start。预算 30／5／10 秒。 |
| 清理 | pending | 原生探针隔离根已清理。两套桌面现场与历史失败保留。自动审批拒绝清理 `pinned-source-check-r1`，该目录仍在。[探针清理回执](raw/desktop-r2/status-owner-probe-cleanup.json) |

本轮首错为 MSI 管理解包入口路径格式，已用同一制品复验通过。缺包夹具预期首先返回 `bundle_archive_missing`，用户确认下载后的首错为 `bundle_download_unavailable`。已通过 Git 请求集成线提供与固定摘要相同的未发布测试资源、首次 GUI Status 关联事件和冷启动空闲证明。

[待证伪判断](raw/hypotheses.md)。下文保留各次实机与 CI 读回的历史证据。
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

RDP session 2 中使用同一固定 EXE 和两个全新隔离根。断线后读回原进程，未重启候选。下表是此次续验当时的读回，最新状态见页首。

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


### 缺包暂缓与重开实机读回

同一缺包进程 PID 87380，15119 RDP session 2。点击「稍后处理」后显示「运行环境待处理」和「查看环境准备」，运行控件禁用。点击「查看环境准备」后恢复缺包提示、下载入口及稍后处理选项，运行控件仍禁用。暂缓和重开通过。下载确认框与确认后产品请求待取证。此前的自动操作失败日志保留。

[暂缓后界面](raw/desktop-r2/missing-deferred.txt) · [重开后界面](raw/desktop-r2/missing-reopened.txt)

### 首次 Status 原生无 Owner 探针

待证伪判断为环境已就绪时，Windows 原生路径会在等待 5 秒前因缺少同进程 Owner 而结束。固定候选供应出的 2.99.0 程序通过身份准入，用时 321,327 µs。对全新隔离 HOME，直接原生 Status 用时 124 µs，完整采样 303,528 µs，轻量采样 295,018 µs，三者均为 `Unreachable`。原生首错为 `windows024_owner_unknown`。隔离 HOME 树未变化，未调用 Start。另有三项原生拒绝与预算反例通过，四项环境用例跳过。探针源码 `feature/024-windows-native-ci@df61a19f7c3e4184bbf1910e045eae75afae08b3`。[探针原日志](raw/desktop-r2/status-owner-probe-msvc.log) · [反例原日志](raw/desktop-r2/status-owner-regressions-msvc.log)

真实 GUI 首次请求起止与首错仍缺。新 HOME 下原生 Status 只查找已有 slot，创建 slot 的入口是 Start；证据支持冷启动门槛互锁。未知拒绝继续生效。等待集成线明确冷启动空闲证明及共享请求关联事件，本轮未执行生命周期和 W2，accepted:false。

### Windows CI 安装恢复首错与原生探针清理

| 作业 | 结果 | 首错与原始证据 |
| --- | --- | --- |
| 45909098994／140212863817 | 步骤 18 fail | 最终 EXE WebView 冒烟把 code 14 / discovery_runtime_unresolved 判成异常，后续恢复步骤跳过。[原始 stderr](raw/ci-native-recovery-r1/older-startup-stderr.log) · [作业](raw/ci-native-recovery-r1/job-140212863817.json) |
| 45910157474／140216305356 | 步骤 18 pass，步骤 20 fail | 最终 EXE 启动观察成功，安装恢复在 windows-install-recovery.mjs:89 首次断言 immutable sandbox mismatch 退出。Node22 和普通用户后续冒烟跳过。[启动](raw/ci-native-recovery-r1/final-exe-startup.json) · [WebView](raw/ci-native-recovery-r1/final-exe-webview.json) · [安装恢复结果](raw/ci-native-recovery-r1/recovery.json) · [首错](raw/ci-native-recovery-r1/recovery-execution.log) · [作业](raw/ci-native-recovery-r1/job-140216305356.json) |

Windows 分支 138aec09388d995b9ef47a74b2a6730d55081bb6 复用版本化准入，仍逐项核对 Windows、活动数据根、内部 HOME 和聚合根。语法检查和 51 项 Windows 定向断言通过，复验 CI 为 45917490865。安装恢复实际事务仍未验证。后续旧驱动调用 2.91.0／2.93.0，并依赖不接纳 0.2.24 的 UI 合同；需按固定 2.99.0 候选的真实受管更新入口和受批准的定向失败夹具验证，保持原恢复判据。[定向原日志](raw/ci-native-recovery-r1/driver-tests.log)

CI 诊断 ZIP 15162224555 和 15162378359 的 SHA-256 已复算并匹配 Actions API。原生探针仅有一个登记文件和三个子目录，各项和摘要核验后已逐路径清理，探针根读回缺席。当前两套桌面续验夹具仍在，历史失败保留。[探针清理原始回执](raw/desktop-r2/status-owner-probe-cleanup.json)

### 缺包下载入口前基线与共享 CI 首错

北京时间 08:40，同一个 15119 RDP 窗口 PID 87380 仍显示缺包和「下载运行环境」，运行控件禁用。缺包 HOME 内未找到 `.tar.gz`，应用日志仍只有初次 `bundle_archive_missing`，实际确认和下载请求待验证。[可访问性树](raw/desktop-r2/missing-before-download-entry.txt) · [日志](raw/desktop-r2/missing-before-download-entry.log) · [进程及归档清单](raw/desktop-r2/missing-before-download-entry.json)

Actions 45917490865 的 Windows 后端作业 140240116416 已通过 Clippy，步骤 10 回归仍在运行。独立共享前端作业 140240116135 失败，913 项测试通过，首错为 `tests/ui-dialog-glass-v2.test.ts:176` 的 Chromium `afterAll` 关闭钩子超过自身 10 秒。Git 留存原始首错片段与作业回执，完整日志仍在 Actions。此处超时属于测试钩子，与产品 Status 的 5 秒预算无关。按 TEAM83 交共享线核查 Chromium 清理卡点并保留现有断言。[首错片段](raw/ci-native-recovery-r2/job-140240116135-first-error.log) · [作业](raw/ci-native-recovery-r2/job-140240116135.json)


### 缺包下载确认框已打开

2026-11-27 08:50（北京时间），单次点击「下载运行环境」后，原 PID 87380 的确认框显示联网及校验提示，以及「稍后处理」「确认下载」。确认按钮尚未点击。同次采集的隔离 HOME 中没有下载归档，应用日志没有新增下载事件。此处采集的是应用日志与文件清单，未进行网络流量采集。[界面与原始回执](raw/desktop-r2/missing-confirmation-before.json) · [应用日志原件](raw/desktop-r2/missing-confirmation-before.log)

### 离线桌面前提复核

2026-11-27，本机 Windows Server 2025 Datacenter 未找到 Windows Sandbox 启动器。Hyper-V 服务运行，但当前令牌在只读 Get-VM 查询时返回权限不足，因此无法确认有可用的已授权隔离虚拟机。完整包离线实测仍为 blocked，需要一台可独立断网且保持交互控制的测试桌面。本次只读复核未变更虚拟机、主机网络或 RDP 设置。[环境原始回执](raw/desktop-r2/offline-host-preflight.json)

### 同一 Windows CI 的独立令牌与测试夹具

Actions 45917490865 的独立作业 140240116350 已通过，制品 15162463428 的 ZIP SHA-256 为 43e9e997220f80297e478e9a9b1bd3c06c614d092599a4fcd77e7e33f51d0c51，与 Actions 一致。临时普通 Profile 的 Known Folder 读回通过，原生自有句柄身份测试拒绝了变更的进程代际。重启移交夹具 40 项通过，未启动产品。安装器矩阵结果只覆盖脚本契约。上述作业不提供真实桌面首次 Status、安装恢复或 W2 验收。[摘要与原始成员](raw/ci-native-recovery-r2/token-evidence-15162463428/receipt.json)

### 缺包界面确认下载后的实机结果

2026-11-27 09:11（北京时间），用户在原缺包窗口确认下载。PID 87380、RDP session 2 的产品日志记录 bundle_download_unavailable，详细原因是「官方地址未提供完整运行资源」。固定源码在收到未被接纳的 HTTP 响应后返回该详细原因，日志未记录具体状态码。独立 HEAD 预检的 404 属于另一请求。

| 核验项 | 结果 | 现场 |
| --- | --- | --- |
| 暂缓、重开、用户确认后发出请求 | pass | 确认前日志无下载事件，点击后产品到达 HTTP 响应处理分支。 |
| 下载成功、校验及安装 | blocked | 界面显示「内置运行环境准备失败」，五项均为「待执行」，并提示保留原内容。 |
| 失败后禁用运行及文件留存 | pass | 运行动作禁用。固定归档、下载暂存目录和 runtime/current.json 均缺席，原描述符、摘要及夹具成员 official-runtime.tgz 仍在。 |

窗口与隔离树保留。未重复下载，未运行 Start。首次 GUI Status 计时、明确空闲、离线桌面和 W2 前提仍未满足。整版 accepted:false。[应用日志](raw/desktop-r2/missing-confirmation-after.log) · [可访问性树摘录](raw/desktop-r2/missing-confirmation-after-ui.txt) · [进程及文件回执](raw/desktop-r2/missing-confirmation-after.json)

### Windows CI 安装恢复重试终态

Actions `45917490865` 的 Windows 后端作业 `140240116416` 于 2026-11-27 01:24:38 UTC 结束，结论为 failure。源码为 `138aec09388d995b9ef47a74b2a6730d55081bb6`。[作业原始回执](raw/ci-native-recovery-r2/job-140240116416.json)

| 核验项 | 结果 | 证据范围 |
| --- | --- | --- |
| 严格 Clippy、Windows 回归、TokenUser、隔离凭据、托管升级／离线重装及 MSVC release | pass | 作业步骤 8、10–16 成功。步骤 13 属于既有托管测试夹具。 |
| 最终 EXE 启动 | pass | 步骤 18 挂载主界面，生产数据根为 `data/desktop`。范围为 `startup-observation`。发现返回 `code:14/discovery_runtime_unresolved`，运行未就绪。[WebView](raw/ci-native-recovery-r2/startup-evidence-15164911523/webview.json) · [启动日志回执](raw/ci-native-recovery-r2/startup-evidence-15164911523/startup.json) |
| 安装失败、元数据恢复与路径回归 | fail | 步骤 20 已越过前轮 `immutable sandbox mismatch`，布局回读为正确根。本轮首错为 `Unsupported native model UI acceptance version`，发生在 `test/smoke/windows-model-ui.mjs:34`，经旧运行时弹窗驱动进入。安装事务和恢复断言均未执行。[stderr 原件](raw/ci-native-recovery-r2/startup-evidence-15164911523/execution.log) · [恢复回执](raw/ci-native-recovery-r2/startup-evidence-15164911523/native-install-recovery.json) |
| Node 22.20 原生环境、普通账户最终 EXE | blocked | 步骤 22、24 因步骤 20 失败跳过。独立普通 Profile 的令牌检查已通过，证据见前节。 |

旧驱动还要求 2.91.0／2.93.0 和旧安装向导。0.2.24 需要经产品的固定 2.99 准备／更新入口提供可复现的失败夹具和恢复输入，再按原有失败留存、原件保全与恢复断言单独验证。真实桌面状态沿用上表，W2 暂停，整版 `accepted:false`。

MSVC ZIP `15165046187` SHA-256 为 `3f4660a6deafb259bb1777455893c67e8c37fe70274fc37148fbeb75af54c048`，其 EXE SHA-256 为 `34f722791f1ee2633c30fe3f1247883c1831aacb63433ca91f2b9ca06ed86410`，与恢复 launcher 一致。诊断 ZIP `15164911523` SHA-256 为 `4615e4a2166665c8b9838aade297688c5905c30393332e48a995c3c52cddc250`，8 份选取成员均按原字节核验。[MSVC 摘要](raw/ci-native-recovery-r2/msvc-candidate-15165046187-digest.json) · [诊断摘要与成员](raw/ci-native-recovery-r2/startup-evidence-15164911523/receipt.json)

### 首次 GUI Status 观察受阻

本轮待证伪判断是，在准备完成前接入 WebView2 回环观察，可以记录真实 UI 发出的首个 Status 请求、终态和未知提示出现时间。自动审批拒绝了新夹具准备、候选启动及观察的组合命令，首错为 `blocked by policy`。只读复核确认新夹具和观察端口均缺席，两个旧窗口仍在原会话，未新增清理项。本轮没有更换入口重试。[审批与本机读回](raw/desktop-r2/first-gui-status-observer-policy.json)

固定源码的 UI 在准备完成并重新核对环境后发起状态观测，观测上限为 5 秒。Windows 原生 2.99 状态路径会在缺 Owner 时提前拒绝。当前证据确认了界面未知及独立原生探针的提前拒绝，实际首次 GUI 请求的发出、返回和落位时序仍需同一次调用关联。请沿已交接的共享诊断入口回交同源候选，保留 30／5／10 秒、未知拒绝和 Start 空闲门槛。远端 main 仍为 `7580b0d79`。本轮未调用 Start，未跑 W2，未发布，`accepted:false`。

### 固定 MSI 的登记与撤销顺序

待证伪判断为固定候选仍缺任一当前用户组件登记，或非升级卸载把终端撤销排在删除文件之后。重算 MSI SHA-256 后以只读模式查询实际 MSI。产品码为 {BB69CCBC-44F3-416A-B577-4BFA017D1EB9}，包表没有 ALLUSERS 或 MSIINSTALLPERUSER，Registry 表在 HKCU 的 Software\gzers\OpenCodeX Desktop\023\Components 下含全部八项。OCX023RevokeTerminalCli 只对已安装产品的非升级完整卸载生效，序号 1401，早于 InstallInitialize 1500 和 RemoveFiles 3500。两项包表断言均通过。[包表原始回执](raw/msi-candidate-database.json)

读表首错为 MSI SQL OpenView 拒绝未加引号的列名，修正后输出包表；中间一次路径筛选表达式有转义错误，换为字面匹配后复核通过。本次未运行安装或卸载。历史产品 {A4D717F0-…} 的 HKLM 登记偏差成因仍未定，本次也未验证固定候选在干净普通账户上的卸载成功或失败保留。

### 固定 MSI 与 NSIS 的十项载荷复核

待证伪判断是同源的 NSIS 与 MSI 携带不同程序或运行资源。两份安装器的 SHA-256 复算匹配候选清单。从固定 NSIS 以只读流读取十项，与 MSI 管理解包原件核对。九项原字节一致，覆盖固定运行归档、清单与摘要边车、官方归档、原生入口、CLI、守护进程和许可资源。严格比较首错为 opencodex-desktop.exe 的 SHA-256 不同。[NSIS 原始列表](raw/nsis-candidate-payload-list.log) · [严格比较原始结果](raw/nsis-candidate-payload.json)

两个主程序均为 12,017,664 字节。差异只发生在 .rdata 的零基偏移 11253647 和 11253649。固定工作流使用 Tauri CLI 2.11.4，打包源码把同一标记位置分别替换成 MSI 与 NSS。首次标记探针因假定标记仅出现一次而失败，复查按实际差异偏移定位，证实仅在内存对齐这一处标记后全文件逐字节一致。原 EXE 与安装器摘要均保留。此次只读包内容，没有执行安装器或写出成员文件。离线桌面、真实安装与卸载状态沿用现有回执，accepted:false。[PE 字节差异](raw/nsis-candidate-main-exe-difference.json) · [打包源码与标记复核](raw/nsis-candidate-bundle-type-marker.json)
