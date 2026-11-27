# Windows TEAM85 普通账户复验交接

治理基线：docs/governance-main@fa336d097。固定源码：9a452c37d3005119042d81ee1ae8f62c4920c4d0。同源 CI：46040314845。Windows 构建及制品：46049421165 / 15224497695。

验收状态：accepted:false。断开桌面网络按合同跳过。使用现有普通账户 marenben 的 RDP 会话，账户和 Profile 保留。

| 验证项 | 状态 | 证据与首错 |
| --- | --- | --- |
| ZIP、MSI、NSIS 摘要 | pass | 本机重算并核对通过，见 raw/digest-verification.json。首次下载定位错误 Missing artifact redirect，后使用已认证重定向成功取得同一制品。 |
| NSIS 运行夹具的候选来源 | pass | 从已验证 NSIS 提取，八个核心成员逐个比对治理摘要与长度，另外保留两份资源声明。见 raw/payload.json。这一步没有运行安装器。 |
| 完整包首开自动准备、步骤、无需联网提示及终态 | blocked | 等待真实普通桌面首开回执，已准备同源码观察器。 |
| 缺包暂缓、重开及确认前零 GET | blocked | 独立夹具入口已准备，未取得真实桌面交互与原生传输证据。 |
| 主动确认、同一 attemptId 及公开下载 | blocked | 等待普通桌面交互。IPC 观察只覆盖 WebView 到本机桥接，不能代替原生 HTTPS 响应。 |
| 首次 Status、5 秒预算、未知拒绝、重试及诊断入口 | blocked | 等待完整包普通桌面首开回执。 |
| Doctor、首错和旧结果保留 | blocked | 等待同一普通桌面。 |
| PID-only 拒绝 | blocked | 仅核对固定源码中的定向断言，尚无普通桌面实测。 |
| 端口设置写锁拒绝 | blocked | 既有自动审批拒绝了本机实写锁探针；保留该决定。固定源码中的缩短窗口单测不能代替生产预算验收。 |
| Start → Status → Stop（30／5／10 秒） | blocked | 尚无可信空闲证据。 |
| 清理 | blocked | 本轮公共暂存及入口保留供继续实测。编制本回执时接收目录为空；普通桌面夹具和监听尚无可归属回执。既有账户、Profile 与历史证据保留。 |

待证伪判断见 hypotheses.md，夹具入口及只读辅助观察源码在 raw/。首次状态返回与界面采样须分开核对。强制清理不能签作正常 Stop 或 Closed。
