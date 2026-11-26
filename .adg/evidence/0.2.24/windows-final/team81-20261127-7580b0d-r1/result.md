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
