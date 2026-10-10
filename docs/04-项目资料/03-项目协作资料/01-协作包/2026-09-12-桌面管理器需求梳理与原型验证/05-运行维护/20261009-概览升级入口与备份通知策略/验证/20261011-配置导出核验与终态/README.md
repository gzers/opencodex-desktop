# 配置导出核验与终态验证

### bd355e1c 配置导出回读核验与精确终态

目标锁在旧文件读取 / 备份前取得并持有到写入、精确回读和终态回调结束；只有实际文件与预期字节完全一致才成功。旧文件读取、备份、写入或回读失败报告失败；回读失败可能已有目标文件改变，不承诺自动回滚。源文档 / 容器构建与根 / 来源 / 锁拒绝发生在 begin 前，不虚构执行终态。

GUI / IPC 共用用户触发 ConfigExport、Action::Export 与 config-export-failed / succeeded；候选绑定完整文档摘要与无损目标路径，仅摘要持久化。同根世代 / 动作 / 候选的成功只解除捕获的失败；内容或目标切换不能解除之前失败。通知投递失败不覆盖实际导出结果；无 AppHandle 的库调用不虚构投递。generic Migration / Backup / Sync 仍保持 0.2.0 planned，不新增计时器 / 持久路径。

最终迁移定向 61/61、IPC 44/44、注册表 32/32；Rust 34 组 / 844 通过 / 0 失败 / 5 忽略，fmt / Clippy all-targets -D warnings / staged diff 检查通过。首轮注册表 31 通过 / 1 失败是新增测试误用 planned lookup 的返回契约；仅修正断言，原件保留。早期 migration 导出定向 9 项日志不绑定最终补丁。

生产注册表更新为 8 triggers / 37 jobs / 76 events（74 active / 2 planned）/ 25 policies / 82 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。真实临时文件腐败、备份拒绝、源 / 根 / 锁拒绝与持久异常重载重试验证通过，但不能代替原生 AppHandle 投递、跨存储一致快照或全事务原子保证。候选轮换后返回旧内容不复用旧 UUID。未改 UI / 未重跑原生。

证据见协作包验证/20261011-配置导出核验与终态及 .adg/evidence/OCX-0110-20261011-CONFIG-EXPORT；源码 bd355e1c93f1d7f856ca72bb124f916ddb634ab1。导入核验 / 终态、全域事件、损坏保护受控修复、双平台 / 安装器、实际更新 / 重启、配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，不回写稳定核心、不关闭或发布。
