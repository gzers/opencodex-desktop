# TEAM84 采集路径勘误与 Windows 观测复核

待证伪判断是旧回执中的运行记录缺席来自候选实际绑定的 HOME，以及冻结 EXE 现有的 Windows smoke 能直接取得真实首次 Status 的完整计时。复核源码 e663369226337c9a9cad594ccd3cafd1719c3711；再次 fetch 后，主线和 TEAM84 合同未变。

旧采集器 `collect-team84-r2.ps1` 和旧启动器的 Collect 分支检查的是 `<sandbox>/data/opencodex`。候选的 `resolve_opencodex_home` 与 `DataRootLayout::with_aggregate_root` 都把 Inside HOME 定义为 `<sandbox>/data/opencodex/home`。因此原始回执中 `ocx.pid`、`runtime-port.json` 的 `present:false` 仅描述父目录。本轮当前令牌读取 marenben 的正确私有目录收到 Access denied，实际记录仍为未知。保留旧回执，不以文件缺席证明空闲。新一轮采集应先通过 `get_data_root_config` 核对实际 HOME 在本轮根内，再检查该 HOME 下的记录。

现成 Windows smoke 在显式沙箱内可以启用 loopback CDP 与独立 WebView 数据目录。`runtime_preparation_status` 仅读取准备快照，其 attemptId、revision、phase 可以与界面正在显示的准备阶段和「无需联网」提示对应。缺包场景可记录暂缓、重开前后的快照，并在用户主动确认时观察变化。快照与 UI 本身不提供实际 HTTP 响应，也不能证明离线。

现有 `windows-webview.mjs` 在页面挂载后才读取状态。后加的 `get_status_snapshot` 是独立样本，无法代替应用的自动首次请求。即使观察到界面由等待切到未知，也只能记录界面落定，不能确定后端耗尽 5 秒，还是在预算内返回不可达。保持共享交接的首次 Status 关联追踪要求，沿同一次请求记录发出、后端阶段、返回与 UI 落定。Doctor 继续沿 [已有共享交接](doctor-source-readback-r1.md) 处理。

现有 CDP 发现与 D/H 夹具检查 14 项通过，含 0.2.24 正确 HOME 接受、早期拒绝保留、非 loopback 拒绝。见 [原始测试输出](raw/windows-observer-layout-tests-r1.txt)。本轮未启动新的产品实例，未执行 Doctor、下载、生命周期或清理操作。保留 TEAM84 的断网跳过记录；当前目标要求的独立离线实机尚未验证。Start／Status／Stop 保持 30／5／10 秒，未确认空闲继续拒绝 Start。accepted:false。

补采入口 [collect-team84-home-r1.ps1](raw/collect-team84-home-r1.ps1) 仅在既有普通账户与原会话读取 Full／Missing 的保存位置和 v2 Inside 元数据，核对物理目录后才观察实际 HOME 下两个记录。外部 HOME、路径歧义或不可读保留为未知。回执仅记录状态与失败阶段，不导出 PID／端口记录内容。记录缺席只表明这次目录观察，不授予启动资格。Windows PowerShell 5.1 [夹具输出](raw/collect-team84-home-r1-tests.txt) 通过 9 项检查并确认夹具已删除。

2026-11-28 00:13:37（北京时间），marenben 在原会话 3 完成[只读补采](raw/collect-home-20261127T1613375446385Z.json)。原始回执的文件所有者 SID、回执 SID 与本机账户 SID 一致，回执记录管理员组缺席。回执声明的冻结源码和采集器摘要均与前置证据一致。Full 和 Missing 均绑定各自的 `<data>/opencodex/home`，两个 HOME 中的 ocx.pid 与 runtime-port.json 均缺席，无绑定或读取错误。回执仍为 provesIdle:false。首次 Status 的耗时、底层首错及 Doctor 的底层首错仍待同次调用取证。
