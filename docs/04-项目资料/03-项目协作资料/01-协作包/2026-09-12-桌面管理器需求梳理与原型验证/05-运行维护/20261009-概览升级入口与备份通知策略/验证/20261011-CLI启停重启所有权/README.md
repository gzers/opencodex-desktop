# CLI 启停重启所有权验证

代码分支：feature/0.1.10-maintenance；完整提交：14b85c11844f63a98f5549ebd54dd034e80bb206。

### 14b85c11 CLI 启停重启所有权与精确终态

Unix CLI 绑定 GUI 共享运行时变更准入，复用既有生命周期核验与注册通知；后台执行者持有运行时 / 存储准入至真实终态与脱敏审计完成，观察者取消不提前释放。仅 Start / Restart→Started、Stop→Stopped 成功；取消、失败、错误或不匹配终态失败，保留两字段 DTO、请求 ID 和显式执行环境。CLI 仍使用自己的 ControlledProcessRunner，未宣称与 GUI 共用 runner 实例。

同根世代 / 动作 / 候选的成功精确解除已有异常；通知偏好关闭不阻止已有异常解除。候选切换后恢复旧文件产生新世代，不能解除旧世代失败。初轮 19 通过 / 1 失败来自测试错误预期；仅修正测试，保留原件。最终 IPC 定向 20/20、process_action 8/8，Rust 34 组 / 824 通过 / 0 失败 / 5 忽略，fmt / Clippy all-targets -D warnings / staged diff 检查通过。

取消 / panic 与 20ms executor 响应测试使用注入 runner 和真实准入 / 审计，不替代真实原生生命周期或性能预算。注册表数量不变（8 triggers / 36 jobs / 74 events、72 active / 2 planned / 24 policies / 80 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup），无新增计时器或路径，未改 UI / 未重跑原生。证据见协作包验证/20261011-CLI启停重启所有权及 .adg/evidence/OCX-0110-20261011-CLI-LIFECYCLE-OWNERSHIP。

本轮更新 CLI 启停 / 重启所有权待验结论；CLI 查询 / 更新 / 数据根切换、迁移终态及全域事件、损坏保护受控修复、双平台 / 安装器、实际更新 / 重启与配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。
