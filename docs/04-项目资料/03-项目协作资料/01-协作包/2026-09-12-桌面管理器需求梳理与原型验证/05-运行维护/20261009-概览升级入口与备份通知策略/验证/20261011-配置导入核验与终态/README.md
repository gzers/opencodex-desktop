# 配置导入核验与精确终态

源码：`feature/0.1.10-maintenance` / `8e54b7f5c01eb439659bfcadf2a2ae2402062658`。TASK-OPENCODEX-DESKTOP-182 / IMP-20 继续 `in_progress`。

导入先验证容器、口令、显式 HOME、路径与锁准入，再捕获旧文件和创建保护备份；偏好、可选 WebDAV 配置、管理器拓展配置锁及 W2 事务锁持有至真实终态。只有 NotFound 可作为缺失处理，读取错误与损坏配置阻止执行。写后逐项精确回读，全部写完后再次核验；失败按逆序恢复已尝试目标，删除原本不存在的新文件并核验缺失。恢复失败明确报错，不报告成功。

GUI / IPC 共用用户触发 ConfigImport / Import / local 的失败与成功事件。候选绑定文档摘要与无损 HOME；同根世代、动作、候选的成功只解除捕获异常，异文档、异 HOME、候选轮换后返回旧内容不能解除旧异常。通知错误不覆盖业务结果，无 AppHandle 库调用不虚构持久事件。generic Migration / Backup / Sync 仍为 0.2.0 planned。

| 最终检查 | 结果 |
| --- | --- |
| Rust 全量 | 34 组，852 通过 / 0 失败 / 5 忽略 |
| migration | 68 通过 |
| extensions | 22 通过 |
| event_registry | 33 通过 |
| IPC | 44 通过 |
| fmt / Clippy all-targets -D warnings / staged diff | exit 0 |

新增 7 项导入测试涵盖多文件精确恢复、新文件清除、恢复失败、读取与保护备份拒绝、持久异常重载精确重试和准入拒绝；注册表新增错误动作 / 通道拒绝测试。最初 61 项日志不绑定最终补丁；新增测试首次 E0599 编译失败原件保留，使用实际 AppErrorPayload code 13 修正后重跑通过。5 个忽略项如实保留，不计入通过。

注册表：8 triggers / 38 jobs / 78 events（76 active / 2 planned）/ 26 policies / 84 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。没有新增计时器或持久路径。

这是合作锁下的逐文件核验与尽力恢复，不保证跨文件原子事务、崩溃回滚或非合作进程父路径突变安全。仅导入管理器拓展配置，不投射真实 Skills / MCP 客户端资产。本轮未改 UI、未重跑原生；不能替代 Windows 双权限、安装器、真实更新 / 重启、配对 CPU / RSS / p95 和真实 stable24h / beta6h 周期验收。全域事件盘点、损坏保护受控修复继续待验；未回写稳定核心、关闭或发布。

原始日志、命令与结果：`.adg/evidence/OCX-0110-20261011-CONFIG-IMPORT/verification.json`；文件摘要见同目录 `files-sha256.json`。
