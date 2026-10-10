# CLI 查询与切根所有权验证

- 代码分支：feature/0.1.10-maintenance；提交：f1a0e00342c461b6d17287314195fbf701c520b0。
- 最终 IPC 定向 25 通过 / 0 失败；Rust 全量 34 组 / 829 通过 / 0 失败 / 5 忽略。fmt、Clippy all-targets -D warnings、staged diff 检查通过。
- 初轮定向 23 通过，保留 unused audit 警告原件；随后删除未使用字段并增加两项公共入口测试，最终日志对应本提交。

状态、数据根、备份列表、现有更新占位查询进入 owned worker；获准查询持有准入至脱敏审计。切根持有 GUI 共享运行时变更与独占存储准入，保存固定 app_data_root 引用后，即使观察者断开、返回错误或 panic，旧根保持冻结至重启；未保存时释放准入。偏好事务锁等待移出异步执行器。备份列表会创建合作锁，冻结时拒绝；其他只读元数据查询仍可用，但不写旧根审计。

公共状态入口等待真实 collector 锁时，20ms 异步计时器仍响应；观察者取消不释放 worker 的准入。公共切根入口等待真实偏好事务锁，取消后仍写入固定 app_root，而非 active root，审计成功后运行时准入释放、旧根写入保持冻结。终态 success / error / panic 及忙 / 冻结 / 未确认零执行矩阵通过。

以上为隔离目录、同进程实锁和 stopped shell 夹具验证；不能替代真实原生生命周期、跨进程绑定、Windows 安装路径或性能预算。CLI UpdateCheck 仍为 not_checked / pending 占位，本轮没有接通真实更新检查。注册表不变，无新增计时器 / 路径，未改 UI / 未重跑原生。TASK / IMP 保持 in_progress，不颁发完成证书，不回写稳定核心或发布。
