# 备份固定状态终态验证

日期：2026-10-10。源码：feature/0.1.10-maintenance@c4996219215ccac3538eebb5eaeda81e3f64c362。版本仍为 0.1.9，未合入 main。

## 实际范围

固定 / 取消固定在共享备份锁内先完成有界准入与保护核验，再写入 manifest 并同步目录；写入返回成功后必须精确回读完整 manifest 才记录成功。候选取 backup_id、action、sha256、bytes、pinned 的摘要，不持久化路径或用户内容。相同根世代与候选成功才解除失败，不添加成功提醒；不同备份、根或取消固定不能误解除固定失败。回到旧内容不复用已旋转候选。

无效 ID、非法锁路径、未决保护拒绝不投递写入终态；生产合作锁对占用会等待，不宣称抢锁失败。取消固定不能绕过 Restore / PreferencesProtection。通知持久化失败不撤销已经精确核验的固定状态，旧异常保守保留。未新增计时器、网络请求、路径或清理。

## 验证与轮次

在代码工作树 apps/desktop/tauri 执行：

- cargo test --locked pin_event_tests：最终 focused-accepted.log 为 5/5，退出 0。覆盖真实写入失败与精确重试、根 / 备份 / 状态隔离、保护与准入拒绝、写者返回成功但回读不匹配、通知失败不撤销落盘。
- cargo test --locked --test event_registry：31/31。
- cargo test --locked：33 组，778 通过、0 失败、4 忽略。
- cargo clippy --locked --all-targets -- -D warnings、cargo fmt --all -- --check、git diff --check：通过。

失败历史全部保留：focused.log 是错误工作目录导致 0 项定向，不是通过证据；focused-rerun.log 是 DTO 字段误用的编译失败；focused-final.log 为 4 通过 / 1 夹具失败（锁文件已存在）；focused-verified.log 为 4 通过后递归申请阻塞锁，精确测试子进程被 SIGTERM 中止，不是通过证据。最终用隔离临时根的非法锁路径测试真正的准入拒绝，生产锁逻辑未改变。

生产注册表现有 8 triggers / 32 jobs / 66 events（64 active / 2 planned）/ 20 policies / 72 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。原型表独立，两项 planned 继续关闭。

UI 与原生未重跑，没有 AppHandle 原生发出点验收。同步连接、端点保存 / 删除及结果映射仍有覆盖缺口，未以本轮测试宣称全域通过。Windows、安装器、实际更新 / 重启、性能与全域终态 / 准入 / 恢复门禁继续未闭合；TASK / IMP 保持 in_progress。未迁移真实用户数据、替换日常安装、公开发布、晋升 stable 或回写稳定事件核心事实。
