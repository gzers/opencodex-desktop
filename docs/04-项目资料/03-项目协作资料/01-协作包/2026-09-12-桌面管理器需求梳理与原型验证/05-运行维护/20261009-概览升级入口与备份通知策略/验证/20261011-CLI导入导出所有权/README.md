# CLI 导入导出所有权验证

### 40f19871 CLI 导入导出所有权与显式 HOME

导入 / 导出移入 owned worker，持有存储准入至真实结果与脱敏审计完成；取消观察者不取消已准入任务。导入使用显式 dependencies.home，修正之前错误使用工作目录的来源。保留现有加密口令限制、exit 13、snake_case 与导入 / 导出 DTO；未新增迁移注册终态，不能称为全域事件已覆盖。

最终 Rust 34 组 / 820 通过 / 0 失败 / 5 忽略，IPC 定向 16/16、fmt / Clippy all-targets -D warnings / staged diff 检查通过。初轮定向日志保留，最终仅改为 Clippy 要求的夹具初始化表达。往返仅核验拓展配置保存，未执行真实 Skills / MCP 客户端资产投射；20ms executor 响应与取消测试为同进程实锁，不能替代跨进程或原生性能预算。注册表计数不变，无新增计时器或路径，未改 UI / 未重跑原生。

证据见协作包验证/20261011-CLI导入导出所有权及 .adg/evidence/OCX-0110-20261011-CLI-MIGRATION-OWNERSHIP。本轮更新 CLI 导入 / 导出所有权与 HOME 待验结论；CLI 启停 / 重启、迁移终态及全域事件、损坏保护受控修复、双平台 / 安装器、真实更新 / 重启和配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。

所有夹具均为临时隔离根，未迁移真实用户数据。代码提交：40f19871c75d3b7d8332a2cfbbce8831cf9307e9，分支 feature/0.1.10-maintenance。
