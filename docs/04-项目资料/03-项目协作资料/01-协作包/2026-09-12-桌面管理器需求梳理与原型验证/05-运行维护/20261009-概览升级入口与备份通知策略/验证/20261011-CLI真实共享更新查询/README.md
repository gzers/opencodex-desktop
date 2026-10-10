# CLI 真实共享更新查询验证

- 代码分支：feature/0.1.10-maintenance；提交：7a3f290e4fdc294983a9046af21a44d7624a3ae9。
- 最终管理器查询定向 14/14，IPC 28/28，Rust 全量 34 组 / 837 通过 / 0 失败 / 5 忽略。fmt、Clippy all-targets -D warnings、staged diff 检查通过。
- initial / intermediate 日志是补齐最终测试与状态暂存前的轮次，只保留原件，不绑定最终源码。

CLI UpdateCheck 接入真实 updater.check，共用 GUI 的 SharedUpdateStatus、通道、端点、调度预留、缓存与查询终态事件。查询观察者取消不取消后台任务，GUI / CLI 分别持有准入至终态投递 / 脱敏审计完成；磁盘与插件准备移入 blocking worker。旧通道世代不会写缓存、推进新通道截止时间或解除新通道异常。冻结准入前不执行 setup / 网络工厂。CLI 保留 snake_case 三字段 DTO，真实失败与被取代分别返回 ExecutionFailed / TargetStateConflict。

真实文件系统矩阵覆盖 stable / beta × available / up_to_date / failed、其他通道保持、内容有界、旧详情清空及持久化拒绝不报告成功；取消、错误、panic 验证 owned 收尾。以上为注入执行体与实盘验证，不替代原生 AppHandle、updater HTTP、安装与重启。

缓存与调度仍按既有顺序分别写入；不是双文件原子事务。若缓存保存后调度保存失败，缓存可能已变化。本轮持久化失败夹具阻断于缓存写入之前，不能证明第二次写入失败的全回滚。成功通知只在完整保存返回后发出；内存失败清除候选。

注册表不变，无新增定时器 / 路径，未改 UI / 未重跑原生。迁移注册终态、保护损坏受控修复、双平台 / 安装器、真实更新 / 重启、配对性能 / 真实周期继续待验。TASK / IMP 保持 in_progress，不颁发完成证书、不回写稳定核心或发布。
