# TEAM81 Windows 首次 Status 界面预算核对

源码 `feature/024-windows-native-ci@7889ecc3586eb811cffc4e80ec1d6f97998caaae`。本轮要证伪的判断是后端排队会推迟界面 5 秒期限，或轮询事件会提前替代首次命令的结论。只读核对产品链并运行已有定向断言，未启动产品桌面或服务。

| 项目 | 结果 | 证据 |
| --- | --- | --- |
| 界面期限覆盖 IPC 等待 | pass（源码及模拟时钟） | `useEnvironmentObservation.ts:70–75` 在环境就绪后触发观察，`app/lifecycle/store.ts:374–401` 在 IPC 前启动计时。未完成的 IPC 在 4999 ms 仍显示等待，5000 ms 进入未知，Start 被禁用。 |
| 首次命令与轮询结果区分 | pass（模拟 IPC） | 定向断言覆盖未知事件后返回已确认命令、已确认事件后返回未知命令，事件均不提前结束首个请求。 |
| 共享后端队列 | 只读事实 | `commands/mod.rs:670–695` 先调度阻塞任务，再锁定收集器。`modules/status/mod.rs:739–773` 进收集器才创建自己的采样期限。界面期限会先落位，后端本次调用仍可继续。 |
| 同源候选真实首个 Status | blocked | 现有 Windows 阶段诊断为空实现，仍缺同次请求的界面开始、结果和时间。原机上首错 `windows024_owner_unknown` 为原生采样观察，界面分类尚未实测。 |
| Start → Status → Stop | blocked | 尚无明确的隔离 HOME 空闲证明。 |

定向 Vitest 3 通过，21 按筛选条件跳过，退出 0，首错无。[命令与控制台摘录](raw/vitest-excerpt.txt)。未创建桌面夹具或服务，无新增实机残留。完整离线准备、下载恢复和实机验收沿既有交接。`accepted:false`。
