# CLI 同步所有权与终态（2026-10-11）

实现：feature/0.1.10-maintenance@d1ab651c9bab783cae6f5268cbdbb208f4faa089。未并入 main，版本仍 0.1.9。

生产 Unix CLI 服务绑定运行中 GUI 的 AppHandle，复用 GUI 同步准入与核验执行体。冲突返回 TargetStateConflict，端点 / 凭据回读失败返回 ExecutionFailed，网络错误保留分类。GUI 上传失败 DTO 与配置读取错误契约保持；通知和状态使用同一存储。

请求取消后，已准入 owned worker 仍持有存储 / 同步准入到终态及脱敏审计完成。忙、冻结、未确认零执行 / 零执行审计。panic 写失败审计并释放准入；审计持久化失败记录日志，不覆盖真实执行结果。本轮取消测试使用注入执行体及真实文件系统、准入和审计，不能替代真实 WebDAV / 原生通知。没有证明跨进程互斥、远端持久化或全事务回滚。

最终后端 33 组 / 809 通过 / 0 失败 / 4 忽略，IPC 7/7、GUI 7/7，fmt / Clippy all-targets -D warnings / diff 通过。failed-compile 保留错误枚举名导致的失败，superseded-* 为共享设置错误契约补齐前轮次，不作为最终补丁证据。真实 WebDAV 集成无环境会提前返回，通过计数不能替代真实服务验收。未改 UI / 未重跑原生；UI 94 文件 / 459 项为 82c3479b 历史结果。

机器记录：.adg/evidence/OCX-0110-20261011-CLI-SYNC-OWNERSHIP/verification.json；原始日志在本目录，SHA256 在机器目录 files-sha256.json。注册表计数不变，无新增定时器 / 路径。其他 CLI 写入、全域事件、损坏保护受控修复、Windows / 安装器、实际更新 / 重启和配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，不回写稳定核心，不替换日常安装，不发布。

原始 cargo 日志保留末尾空行；全量 cached diff 检查仅报告原始日志 EOF 空行，非日志文档 / JSON 单独检查通过，不改写原件。
