# macOS 两代正常启停定向复验

2026-11-21，测试进程退出码 101，`accepted:false`。源码基线 cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2，加下表固定的本机补丁。使用隔离测试目录与原样官方 OpenCodex 2.99.0，保留 Start／Stop／独立 Status 的 30／10／5 秒预算。

本轮假设是单个索引检查点内合并重复的归档计划与整树读回，可使前后两代正常 Stop 都在 10 秒内完成。后继失败证伪了该假设。

| 步骤 | 第一代 | 第二代 |
| --- | ---: | ---: |
| Start | 8,742 ms | 11,499 ms |
| 运行中 Status | 223 ms | 919 ms |
| Stop | 7,281 ms | **11,642 ms** |
| 停止后 Status | 1,278 ms | 1,564 ms |
| 最终索引 | root_absent rev9 | root_absent rev18 → root_retirement_prepared rev19 |

两代原生进程对正常退出且退出码为 0，监听均已消失，两次均抵达 final-absence-confirmed。第二代管理器约 11,237 ms 返回 client_original_restore_pending。旧归档未变化。夹具、进程、监听和三项本轮所属协调器已清理，原有协调器保留。

第二代从原生进程对退出到恢复返回约 7,959 ms，第一代约 4,883 ms。第一次 admission.entry-time 属于持有原件后的 artifacts.admission-before，剩余期限分别约 5,486 ms 和 3,489 ms。第二代最终关闭阶段约 1,593 ms，含超时降级。独立 source、candidate、提交后和授权读回各自的成本仍待分解，当前仅定位期限在关闭路径被发现已耗尽。后续先做定向计量，再固定新假设及候选输入。

原始失败日志、测试补丁、索引和归档保存于本机权限受限证据目录。以下为锚定原始文件的 SHA-256。

| 文件 | SHA-256 |
| --- | --- |
| source.patch | 00e2f2c5fb55bdf2ea16ba9eae6bb3eccb1777745e024eae3392a9e71927730f |
| test host | 8e290fa51b7e62e064b85835249f571f7c0d46151f88387b9608716920c1be0b |
| app.log | d8428e27f7cac28d4b7fbdd7dd2fd2aafa841eb1a277d81ef6daf364bb41b9ad |
| focused.log | d7aefa7e9e622178cb2508576a3eb5ed2c8657be3d28f4a85dcc9ce1d694f3e5 |
| root-cycles.json | 0a06e1bbb38c6ac646066be609cf7faf28e51ec925b30704637c9741782ec8e2 |
| two-cycle-index-events.json | 69edc316c1c6ba76e2a930f5e263db0bfb72de71b21dac6667c4d830bb91ec73 |
