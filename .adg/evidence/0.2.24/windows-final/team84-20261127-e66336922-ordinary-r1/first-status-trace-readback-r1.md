# TEAM84 首次 Status 计时复核

候选源码 e663369226337c9a9cad594ccd3cafd1719c3711，Windows 制品 15187611477。普通账户 marenben 会话 3 仍断开，回执目录为空。本次没有启动候选，未取得普通桌面产品首错或首次 Status 耗时。安装器摘要、逐项状态与清理结论见 [前置回执](result.md)。

待证伪判断为冻结 Windows EXE 可以通过现有 `OCXD024_STATUS_TRACE=1` 导出首次 Status 计时。`apps/desktop/tauri/src/infrastructure/status_source.rs` 中 `diagnostic_stage` 的实现由 `#[cfg(target_os = "macos")]` 限定，Windows 命中空实现。实际 Windows EXE 的 SHA-256 与入场校验一致。二进制中也未发现 `OCXD024_STATUS_TRACE` 和 `STATUS024_PHASE`，两个对照字符串均能检出。原始结果见 [复核记录](raw/first-status-trace-readback-r1.json)。原启动器无需为这个变量修改。

`apps/desktop/ui/src/app/lifecycle/store.ts` 的 `loadStatusSnapshot` 用同一 5 秒定时器覆盖请求和忙碌重试。当前请求返回空结果时，界面显示未知。包准备日志证明供应完成，首次请求、后端返回与界面落定仍缺同一次的计时。请共享集成线沿现有沙箱诊断为 Windows 补齐该次请求的关联标识、界面发出与结果、后端进入与返回及单调耗时。只记录阶段、状态及耗时，保持 5 秒和未知拒绝。冷启动空闲证明继续沿用证据提交 `581e8a7e9a7bea04b07e76dab23b3199e5fc9225` 的请求。

后续补验：Full 重开后，用户触发「环境诊断」，[界面截图](raw/full-reopen-doctor-unavailable-user.png)只记录通用失败提示。Doctor 是独立只读命令。固定源码中，托管 2.99 Status 先校验运行身份，再查找当前管理器进程内与此运行来源及 HOME 匹配的 Owner。若没有 Owner，直接映射为不可达。Windows 追踪需区分运行身份校验、Owner 查找、观测返回及同一 5 秒截止时间。Doctor 单独记录失败阶段与可用的退出码，沿用输出脱敏边界。

本次未触发 Start、下载、清理或变更网络。断网桌面按 TEAM84 跳过。accepted:false。
