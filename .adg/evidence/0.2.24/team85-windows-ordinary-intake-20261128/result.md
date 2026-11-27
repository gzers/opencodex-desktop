# TEAM85 Windows 普通账户接收

2026-11-28。accepted:false。当前源码 main@9a452c37d3005119042d81ee1ae8f62c4920c4d0。[同源 CI 46040314845](https://github.com/gzers/opencodex-desktop/actions/runs/46040314845)。安装器待同源构建与核验。

## 接收 TEAM84 普通账户复验

复核了旧候选 `e663369226337c9a9cad594ccd3cafd1719c3711` 的 Windows 原始回执。观察器贡献 `b6f37e07e6129e243b1f5a2e1570cedc7cf90fc5` 已接入 main，净差为三个 `test/smoke` 文件。观察器只读取隔离 WebView 的准备状态和界面，不触发安装或启停。[TEAM84 原始回执](https://github.com/gzers/opencodex-desktop/blob/fbff469c63cd5a72f288b505543471a227084e32/.adg/evidence/0.2.24/windows-final/team84-20261127-e66336922-ordinary-r1/result.md)

| 项目 | 结果 | 证据与后续 |
| --- | --- | --- |
| 普通账户完整包自动准备 | pass | 新隔离根供应官方 2.99.0，启动到校验完成为 170.005 秒。此时长用于准备阶段。 |
| 准备进度及「无需联网」 | blocked | 采集始于就绪后。新候选用观察器从首开记录时间线。 |
| 缺包首开 | pass | 独立夹具返回预期的 `bundle_archive_missing`，未复现 `managed_dependency_missing`。 |
| 暂缓、重开及确认下载 | blocked | 缺确认前零 GET、主动确认后请求及同一尝试的终态。 |
| 首次 Status | blocked | 缺首个请求的耗时与结果。原生探针约 0.23 秒返回 `Unreachable`，空闲仍未确认。 |
| 重开后的未知界面 | pass | 显示未知、重试和诊断，概览无 Start。 |
| Doctor | fail | 界面提示暂不可用并保留上次结果。具体失败阶段待采证。 |
| 正向启停 | blocked | 空闲未确认，未触发 Start。 |
| PID-only 和端口写锁 | blocked | 组件级结果可参考，真实桌面拒绝及副作用尚待证明。 |
| 本轮清理 | blocked | 旧 Full PID 103156 与两个隔离根保留在原回执，清理审批曾被拒绝。新轮使用新根，原回执继续保留。 |
| 断网桌面 | skipped | 按用户决定跳过。 |
