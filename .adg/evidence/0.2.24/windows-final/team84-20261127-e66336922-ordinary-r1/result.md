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
| 断网桌面 | skipped | 按 TEAM84 跳过，保留 TEAM83 原始状态。 |
| 完整包自动准备、进度和无需联网提示 | blocked | marenben 会话 3 断开且 Full 启动回执缺席。TEAM83 对照观察到 40.945 秒准备完成，不能替代本轮普通桌面。 |
| 缺包暂缓、重开、确认下载 | blocked | Missing 启动回执缺席。TEAM83 对照验证了暂缓、重开和弹窗取消，缺同一次确认后的请求及响应。 |
| 首次 Status 耗时和未知界面 | blocked | 缺普通桌面的实际请求及结果。对照确认过未知界面、重试、诊断和 Start 不可用，首次 Status 耗时没有原始计量。 |
| 正向启停 | blocked | 等待首次 Status 和可信空闲；未触发 Start。无保留 Owner 和记录缺席不能证明空闲。 |
| PID 文件拒绝副作用 | blocked | 同源组件测试已通过；普通桌面入口及实际副作用证据待收集。 |
| 写锁超时拒绝副作用 | blocked | 同源组件测试已通过；实机探针遇到工具审批拒绝，保留拒绝，原 30 秒实机断言未验证。 |
| 过程和清理 | blocked | 新夹具尚待运行，保留本轮启动入口和载荷。现有账户及 Profile 保留。历史对照管理器和原始证据留存。 |

[同源普通账户 CI 证据](../team83-20261127-ci-45965743318-token-r1/result.md) 中的 750／250 毫秒组件窗口只覆盖组件行为。历史对照的产品首错是 bundle_archive_missing，采集首错是 IGraphicsCaptureItemInterop.CreateForMonitor 0x80070057。本轮暂无普通桌面启动回执或产品首错。当前阻塞为 marenben 会话断开。入口和预检见 raw/ordinary-desktop-preflight.json，载荷清单见 raw/staged-payload.json。

accepted:false。暂停 W2、双 SID 和发布。
