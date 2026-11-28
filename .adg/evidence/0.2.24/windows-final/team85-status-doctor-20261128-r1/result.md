# TEAM85 状态与诊断阶段回交

候选源码 9a452c37d3005119042d81ee1ae8f62c4920c4d0，Windows 制品 15224497695。当前 accepted:false。上一轮实机证据及安装器摘要见 [TEAM85 回交](../team85-20261128-9a452c37d-ordinary-r1/result.md)。

| 项目 | 状态 | 事实与待复核 |
| --- | --- | --- |
| 完整包准备 | pass | 普通桌面 253.281 秒完成解包、校验和激活，复用现有时间线。物理断网按用户决定跳过。 |
| Windows 原生回归 | pass | MSVC 测试制品编译通过。6 过、0 败、4 忽略。见 [原始日志](native-regression.log)。 |
| 首次 Status | blocked | 界面进入未知，原观察器未捕获首次 IPC 请求。后续只读采样 218 ms 返回不可达，Start 禁用。待同 SID 区分身份解析与无原生 Owner 槽。 |
| Doctor | fail | 3 ms 返回 application is not configured yet。源码先验证受控入口和工作目录。沙箱工作目录为 <sandbox>/home，运行 HOME 为 <sandbox>/data/opencodex/home，待普通 SID 验证两者实际存在性。 |
| 缺包延后与重开 | pass | 普通桌面已验证，Start 不可用。 |
| 确认前零 GET、受控供应与同一尝试 | blocked | 确认后失败 bundle_download_unavailable，阶段 downloading。独立 HEAD 是 404，原桌面 GET 状态未测得。 |
| Windows 普通桌面启停 30／5／10 秒 | blocked | 待可信空闲证据。 |
| 同一主线冻结及 W2／真实双 SID | blocked | 待双方完成同源定向验收。 |
| 清理 | pending | 保留旧现场，无新增桌面启动或强制终止。 |

请 Mac 集成线确认共享 Doctor 工作目录来源和脱敏诊断分类的改动归属，并交接受控供应的同源归档、摘要和测试入口。受控供应验证需核对确认前零 GET、确认后原生响应以及同一 attemptId 到终态。Windows 继续隔离现场的同 SID 验证。空闲接口及未知拒绝仍按[既有交接](../team85-20261128-9a452c37d-ordinary-r1/cold-start-handoff.md)执行。
