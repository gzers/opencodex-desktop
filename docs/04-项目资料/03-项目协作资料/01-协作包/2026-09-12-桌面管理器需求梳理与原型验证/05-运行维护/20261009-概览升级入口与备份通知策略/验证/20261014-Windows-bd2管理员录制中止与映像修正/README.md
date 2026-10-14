# Windows bd2 管理员录制中止与映像修正

2026-10-14，Asia/Shanghai；关联 IMP-20 / TASK-OPENCODEX-DESKTOP-182。用户明确批准“批准管理员后台 ETW 录制并完成提权”。本次完成真实提权并尝试一次录制，但采集器启动校验失败，未取得原查询失败场景证据。完整目标及八项门禁保持开放。

## 本次实际结果

采集器为 `abc3a9140fc6f1b5d2aa6e739299af260f0f5d94`，原库来自失败 PR CI `38537166560`，EXE SHA256 `497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3`。官方 Node 24.21.0 摘要仍为 `ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32`。准入沿用已冻结的实际 PR 检出与 bd2 整树绑定，不换用成功 push 的 EXE。

20:48:34.7995244 创建后台提升控制器 PID 26048；20:48:35.2841537 开始执行。`WindowsPrincipal.IsInRole(Administrator)` 为 true，实际控制器映像和 creation 已保存。以 `Start-Process -Verb RunAs -WindowStyle Hidden` 发起，没有自动操作 UAC、安全提示或终端 UI。

20:48:37.665303 原测试根 PID 55376 创建。内核启用调用已返回，随后 `process.MainModule.FileName` 的映像比较报 `Root image mismatch.`。旧版本未保存该调用的实际返回值，不能确定具体误判原因。异常发生在事件消费者启动之前，不能把零事件解释为被测进程没有活动。

原库实际启动数 **1**、完成数 **0**；`testExitCode=null`，外层安全收尾 `forcedOuterCleanup=true`。没有完整 stdout/stderr，也没有可评估的查询结果；两份空的入口重定向文件不冒充原测试输出。不能用这次中止判断七查询通过、失败或重现历史低 CPU 等待。

唯一会话 `OpenCodex-BD2-15be5566681e4798928fec4a58e27a73` 的 native STOP 返回 0，最终 session loss/buffer loss 均 0，writer 排空成功；事件消费者未启动、root ProcessStart 未见、JSONL 为 0 字节。这里只证明该次自有会话收尾成功，**不证明完整覆盖、过滤正确或目标失败取证成功**。测试根实际令牌尚未采得，不能由控制器提权代替。

收尾后以只读 session 查询及新进程快照复核，无本次会话、控制器、原库、官方 Node 或桌面实例残留；不终止同名无关进程。日常 EXE、隔离 preferences/root/backup policy/schedule 前后摘要一致，没有新桌面实例或自动周期。

首次证据校验没有通过：收尾快照中的裸 PID 26048 已于 20:51:58.289154 被另一个进程复用。原快照保留，改按原 creation 与精确映像联合核验，确认原控制器不在、复用项不属于本次家族。没有对复用 PID 进行终止或其他操作；修正后的 postcheck 与首次 verifier 失败记录一并封存。

## 修正与验证

代码提交 `629b9c6f913b5e04badc9ce747f6bb8ba247fd89` 已推送到 `feature/0.1.10-windows-query-etw`。仅修改原三个 `test/smoke/windows-owned-etw*` 工具，产品、原夹具、工作流、锁和共享文件相对 bd2 不变。

根映像改由已持有的 `Process.Handle` 调用 `QueryFullProcessImageNameW`，不等待初始化、不枚举 loader modules、不预热或重启原库；新结果在校验前写入本次自有根记录。Windows 官方 API 文档分别说明模块表初始化期间枚举可能不可靠，以及进程句柄查询映像的用途；它们支持设计选择，但不能补回本次缺失的实际返回值。

测试启动后立即异步读取 stdout/stderr，启动校验异常也各自有界排空；新增 setup/run error、输出保存状态、实际正常完成/强制终止及收尾后 exit code 字段。原外层 120 秒、默认并发/capture 和内部 2/4/5/8 秒保持。

本机 Add-Type 编译、原 53 项纯检查、新增 5 项当前控制器句柄映像检查、摘要/目录/普通权限拒绝检查通过。新增检查只查询自身句柄，原库/Node/ETW 新执行数 0。它们不替代修正后的真实启动、事件过滤、消费者排空或失败覆盖验证。首次提交因当前 shell 未配置 Git identity 失败，随后仅使用与前序提交相同的命令级 `Codex/codex@local` 完成，未改全局配置。

同 `629b9c6f` 的 push `38567345341`、PR `38567352891` 在首次观测时仍 `in_progress`；frontend/release-tools/windows-smoke-evidence 已成功，backend/backend-windows 仍运行。只保存该时刻快照，不计新 SHA 双 CI 终态通过。PR #5 标题/正文已更新并独立读回，仍 draft/open/unmerged、无 auto_merge；没有重派 CI 或构建新桌面候选。

## 证据与下一项

原件、启动器、授权、准备验证、PR 读回和 CI 初始快照位于 `.adg/evidence/OCX-0110-20261014-WIN-BD2-ETW-RUN/`。`recording-original/` 保存本次 runtime 全部七份原件，逐路径 bytes/SHA256 与来源一致；清单和 verifier 使用相对路径字符串，支持 Windows/macOS 校验。失败原件及旧准备入口均不改。

这次批准的一次原库已经启动，未自动进行第二次。后续先核验修正 SHA 的 CI 终态，再取得下一次有界录制的明确批准，使用全新输出目录。原失败 case、活 Node 身份、调度/文件 I/O 及根因仍待证据，Mac 继续后台，尚未正式交接原生测试。

同证据目录的 `record-request-imagefix.ps1` 已默认 review-only 检查，固定新 HEAD、旧失败 EXE/Node/decoder 摘要、新 runtime 输出根及一次原库/120 秒。实际执行另要求真实管理员令牌、`-ApprovedByUser` 和同级新证据目录 `OCX-0110-20261014-WIN-BD2-IMAGEFIX-CI/ci-admission.json`：同 SHA 的 push/PR 均成功，实际检出树及原日志已核验。当前尚无该准入文件，没有用旧授权或旧 CI 启动第二次；未来 CI 准入材料不会写进本次冻结目录。

原 CPU/RSS failed、原预算/样本数/计量口径、八项 required checks、TASK revision2/in_progress/pending、FZ50 未获脚本许可及日常数据边界保持；不关闭 TASK、颁发证书、合并、签名、晋升或发布。
