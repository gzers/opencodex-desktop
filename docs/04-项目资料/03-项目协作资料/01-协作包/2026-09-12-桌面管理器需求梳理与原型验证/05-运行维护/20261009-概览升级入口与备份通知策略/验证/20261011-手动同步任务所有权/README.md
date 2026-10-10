# 手动同步任务所有权

源码：feature/0.1.10-maintenance@153430bf517b8520255ef57e993c3d33e757e1e0。版本仍 0.1.9，未合入 main。本轮修复已有 GUI 手动同步的任务持有，不启用 planned 比较、自动同步或 0.2.0 功能。

## 实际改动

GUI run_sync_now 把配置与凭据读取、既有 runner 执行、冲突通知和状态 / DTO 投影放入 owned blocking worker。工作线程持有操作门与存储写准入直到返回；IPC 观察者取消不会提前释放任务与锁。连接测试、端点保存 / 删除复用同一入口。忙状态不等待操作锁、不排队执行同步任务；冻结数据根拒绝写入。读取与验证完成后才开始状态投影，开始时生成新 run_id。

上传失败仍返回命令成功的 failed DTO；其他 runner 错误仍返回原错误，冲突通知保持原行为。工作线程错误 / panic 释放两种准入，panic 释放锁的测试不表示原生 GUI 状态已获得终态投影。没有新增完整手动同步终态事件，也没有增加自动检查频率、网络轮询或计时器。

## 验证

- commands::sync 定向 14/14，含新增 owned_sync_tests 3/3及既有 probe 7/7：忙与冻结零任务执行、取消观察者后保持已提交写与锁直至终态、worker 错误 / panic 释放两种准入。
- Rust 全量 33 组 / 788 通过 / 0 失败 / 4 忽略（含零项平台 / doc-test 组）；event_registry 在全量内 31/31。
- cargo clippy --locked --all-targets -- -D warnings、cargo fmt --all -- --check、git diff --check 通过。
- 新增所有权测试使用注入闭包、临时文件与独立锁，不调用真实 WebDAV、钥匙串或原生 AppHandle。全量包含既有本地测试服务器；不能据此推定用户真实服务或 Windows 已通过。
- 未改 UI，未重跑 UI / 原生。注册表未变：8 triggers / 33 jobs / 68 events（66 active / 2 planned）/ 21 policies / 74 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。

命令工作目录为代码 worktree 的 apps/desktop/tauri。所有成功检查基于本提交内容；原始日志保存在 logs/，身份与摘要位于 .adg/evidence/OCX-0110-20261011-SYNC-OWNERSHIP。

## 无效轮次与限制

首次编辑在 tauri 工作目录误用仓库相对路径，FileNotFoundError，未修改源码；随后旧源码测试通过，保存为 initial-old-source.log，不作为补丁验证。正确路径编辑后 accepted.log 为 14/14；本轮没有最终源码的失败测试。证据写入准备时工具 JS SyntaxError 未执行任何命令，不算测试轮次。

此前 a3b84474 的“手动同步仍直接 await、取消所有权待补齐”是历史状态，本轮只在 GUI 命令入口补齐。CLI/IPC 仍直接调用 runner，取消与执行持有需另验。操作门为进程内，导入 / 外部配置写入和跨进程串行化没有完整保证；端点变更 / 手动同步注册终态、全域事件、损坏保护记录受控修复、Windows / 安装器、实际更新 / 重启及配对性能 / 真实周期门禁仍未完成。

TASK-182 / IMP-20 保持 in_progress，目标 active；不颁发完成证书，不回写稳定事件核心事实。没有真实用户数据迁移、日常安装替换、公开发布或 stable 晋升。
