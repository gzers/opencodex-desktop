# 备份策略保存终态验证

日期：2026-10-10。源码：feature/0.1.10-maintenance@d24bd4d56d9c2ad6ddbabffd02573ce81916d8f4。版本仍为 0.1.9，未合入 main。

## 实际范围

已准入工作线程在共享备份锁内投递策略实际保存的失败 / 成功终态；完整 CleanupPolicy 序列化摘要隔离候选，成功只解除精确失败，不新增成功提醒。校验及锁拒绝只即时反馈；通知失败不更改实际保存结果。保存策略不清理任何备份，不新增计时器、网络查询或文件路径。

生产注册表现有 8 triggers / 32 jobs / 64 events（62 active / 2 planned）/ 19 policies / 70 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；原型注册表独立，planned_closed 两项继续关闭。

## 验证

在 apps/desktop/tauri 执行：

- cargo test --locked policy_event_tests：最终 4/4（focused-final.log）；12 份距今 40..51 天的备份在保存自动策略后完整清单保持不变。
- cargo test --locked --test event_registry：31/31。
- cargo test --locked：33 组，773 通过、0 失败、4 忽略。
- cargo clippy --locked --all-targets -- -D warnings、cargo fmt --all -- --check、git diff --check：通过。

测试覆盖真实隔离写入失败、历史加载与精确重试解除、不同数据根 / 不同策略不误解除、回到旧内容不复用旧候选、校验 / 锁拒绝不调用终态观察器、通知持久化失败后策略已真实落盘而异常保守保留。首次 focused.log 对应命令工作目录错误及未追加测试，实际 0 个定向测试，不作为通过证据或产品失败；focused-rerun.log 是增强 12 份备份断言前的 4/4，最终日志为本次权威结果。

本补丁未改 UI，未重跑 UI 或原生；上一 UI 基线 27e425bf 为 94 文件 / 454 项。没有 AppHandle 原生发出点验收。全域终态 / 准入 / 恢复矩阵、Windows、安装器与性能门禁继续未闭合；TASK / IMP 保持 in_progress。未迁移真实用户数据、替换日常安装、公开发布或晋升 stable，未回写稳定事件核心事实。
