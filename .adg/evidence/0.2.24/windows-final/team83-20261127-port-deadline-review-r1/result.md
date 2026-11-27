# TEAM83 Windows Start 端口写锁的截止时间审查

待证伪判断：已有 Owner 和运行记录核验完成后，Windows Start 的 Auto 端口准备仍使用调用方原绝对截止时间。若剩余不足 3 秒且端口设置锁被占用，等待应受剩余时间限制，期限耗尽后不得写端口设置或启动运行程序。

| 检查 | 状态 | 结果 |
|---|---|---|
| Windows 端口顺序修复接入主线 | pass | Windows 来源为 `b490f29e38654457bd0466072f526369fc3ccdd2`，main 为 `3a844aef364830604da49268d3c470a318ecb3fe`。原生生命周期文件及其 Windows 命令入口与贡献提交一致，主线保留自身 PATH 夹具。 |
| 原截止时间传递到 Auto 端口写锁 | fail（源码审查） | 原生 Start 在调用前检查一次 `window.remaining()`，随后同步等待共享端口函数。`port::save` 的锁从调用点另计 3 秒，未消费原截止时间。若锁在原期限后、独立锁期限前释放，还可能先写设置，再由 LaunchPlan 拒绝过期启动。 |
| MSVC 争用复现 | blocked | 已确认上一轮测试 EXE 包含一般写锁争用用例。本轮执行这些测试并清理专用夹具的命令被自动审批以 `blocked by policy` 拒绝，未执行，未创建该命令中的夹具。未测得真实超时秒数。 |
| 主线 CI 与同源安装器 | blocked | [主线 CI 45963586935](https://github.com/gzers/opencodex-desktop/actions/runs/45963586935) 检查时仍运行，Windows smoke evidence 已通过。最新 [Release 45863614604](https://github.com/gzers/opencodex-desktop/actions/runs/45863614604) 的源码仍是 `7580b0d79289864cb105e6d56d668b608b93495b`。无本次主线的 MSI／NSIS 摘要可复算。 |

源码首个断点是 [Windows 原生 Start 调用](https://github.com/gzers/opencodex-desktop/blob/3a844aef364830604da49268d3c470a318ecb3fe/apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs#L414-L422)。它调用 [共享端口准备和保存](https://github.com/gzers/opencodex-desktop/blob/3a844aef364830604da49268d3c470a318ecb3fe/apps/desktop/tauri/src/modules/runtime/port.rs#L108-L170)，保存使用 [独立的 3 秒写锁及同步重试](https://github.com/gzers/opencodex-desktop/blob/3a844aef364830604da49268d3c470a318ecb3fe/apps/desktop/tauri/src/infrastructure/locking.rs#L14-L76)。外层 [原生入口](https://github.com/gzers/opencodex-desktop/blob/3a844aef364830604da49268d3c470a318ecb3fe/apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs#L330-L350) 同步 join。

请 Mac 集成线在共享端口模块提供接收绝对截止时间的启动准备入口，争用等待以原截止时间与现有 3 秒锁上限的较早者为准，并在获得锁后和提交写入前检查预算。保留独立用户设置保存的锁策略。Windows 消费该入口，保留 Owner／运行记录先核验及未知拒绝。定向测试应在锁持有跨过原期限时断言及时拒绝、原端口文件字节保持、无新运行程序，并保留及时释放时的正常分配正例。此请求不改变 Start／Status／Stop 的 30／5／10 秒。

本轮为只读源码核对，未运行产品或变更日常配置。完整包离线、缺包固定资源、真实首个 GUI Status 与正向空闲证明继续沿已有交接。W2／双 SID 暂停，未发布，`accepted:false`。
