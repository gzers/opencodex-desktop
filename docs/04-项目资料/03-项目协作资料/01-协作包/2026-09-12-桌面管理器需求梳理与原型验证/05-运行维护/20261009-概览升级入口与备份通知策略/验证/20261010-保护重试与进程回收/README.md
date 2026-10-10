# 面板保护本地重试与 Unix 进程回收

源码 feature/0.1.10-maintenance@56c8462c2f306d4531c1c2e9dfc6687cb52b2e0c，版本仍为 0.1.9 开发态，已推送。b974e61a 调整两个测试的模拟初始化时机；0be9d8b3 修复 Unix 双重回收；56c8462c 接入状态 / 本地重试与精确范围终态事件。

设置显示 pending、verified_pending、unreadable，读取不写盘、不核验或解除。显式重试持有运行变更准入及备份事务，以捕获的精确关联核验；关联消失、被替换、激活证据不足或摘要不符不解除。候选从尝试 / 版本 / 关联摘要构建，重试失败与真实成功使用相同范围，错误对象 / 动作 / 候选不能解除。晚到读取不能恢复已处理旧状态；没有新增定时查询、网络或安装命令。损坏记录保留，未提供强删 / 强解锁；受控修复尚待实现。

全量 Rust 初次终态回归失败：shell exit 3 被记录为 Cancelled。单独重跑通过不能消除缺陷；8 路并发回归复现了 Tokio Child 丢弃后孤儿回收与裸 waitpid 的竞争。修复后 Unix 使用 std Child，单线程持有并 wait；512 次退出保留 Failed。非 Unix 仍用原 async waiter，超时 / 取消 / 信号边界未放宽。

最终 macOS arm64：Rust 33 组 / 765 通过 / 0 失败 / 4 忽略，fmt 和 Clippy all-targets -D warnings 通过；UI 94 文件 / 447 项、类型检查、生产构建通过。UI 日志在最终 Rust 改动之前，其 UI 内容与提交一致。单测 / JSDOM 结果不等于原生显示或像素验收。

原始命令、失败与修复后日志、身份、适用限制及摘要：.adg/evidence/OCX-0110-20261010-PROTECTION-RETRY-REAPER。名为 concurrent-before 的历史日志实际仍为单工作者测试，不计为并发证据；真正复现来自 eight-before。

当前注册表 8 triggers / 29 jobs / 58 events（56 active、2 planned）/ 16 policies / 64 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。新增保护重试仅 user 触发，未增加周期任务。原生更新 / 重启、损坏关联受控修复、全域发出点覆盖、Windows、安装器及配对性能 / 真实周期继续未完成；TASK / IMP in_progress，不回写稳定核心，不发布。
