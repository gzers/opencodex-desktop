# Windows bd2 basename 覆盖与完整家族录制

2026-10-14，Asia/Shanghai。关联 IMP-20 / TASK-OPENCODEX-DESKTOP-182。用户持续授权“批准修正版后续都可以管理员后台 ETW 录制”。本页记录 `e6a4b32f8b60e362911cc02bc67fc0a2f82e7963` 的本机管理员诊断，不构成版本或发布通过。

## 代码、CI 和范围

e6 仅修改 ETW 测试采集器：当 Windows ETW `Process/Start` 只提供 `cmd.exe` basename、argv 首参数也是裸 `cmd.exe` 时，在已归属父进程下以 `command-basename` 准入；随后仍用持有的进程句柄核验真实完整映像路径。未知映像、冲突 basename 或弱证据仍拒绝。

本机准备检查共 91 项通过。push CI `38583679031` 成功，Mac/Windows 分别 900/627 通过；PR CI `38583689702` 保留原三个 Windows version fixture 失败，Windows 527 passed / 3 failed / 4 ignored。PR 红灯来自原测试 fixture 的低执行进展，不是采集器在 CI 中实跑；该失败保留为已知失败诊断准入，未授予版本或发布通过。

## 实际管理员录制

使用原失败 EXE SHA256 `497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3` 和官方 Node 24.21.0 SHA256 `ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32`。控制器 PID 65712 为真实管理员令牌；测试根 PID 73328 的原库令牌为 elevated/type2。原库一次完成，exit101；默认并发/capture、无预热、原 2/4/5/8 秒及 120 秒外层上限保持。

录制结果：28,001 条 JSONL 事件、9,591,262 bytes，UTF-8/LF 实际计量低于 64 MiB；序列化丢弃、kernel loss、decoder error 均为 0，唯一自有 session native STOP=0，consumer/writer 排空成功，强制清理 false。

本轮发现并记录完整自有进程家族：31 个 process-start 与 31 个 process-stop，角色为测试根 1、Node 22、`command-basename` 8；所有开始记录均有对应结束记录，8 个命令中转均被采集。已知映像未准入计数为 0，修复了 0407 录制中的命令覆盖缺口。部分短命 Node 无法取得实时 API 句柄，但保留原 ETW 生命周期、创建时刻、父子关系和结束事件；不把句柄缺失伪装为完整 API 观察。

高频根调度/根 I/O 仍按范围省略并单独计数（调度 915533、文件 I/O initiation 311295），因此证据证明的是自有 Node/cmd 查询家族的完整生命周期和高频事件范围，不是测试根全部高频活动的完整序列。没有写整机 ETL、命令行、SID、无关路径或其他调度参与者 ID。

## 原测试结果和结论

原完整库为 529 passed / 1 failed / 4 ignored。七个 query cleanup fixture 全部通过；唯一失败仍是 `commands::workspace::tests::local_documents_are_frozen` 的 `license.opened`，原因是原制品编译时 CI runner 路径在本机不存在。未造路径、未改断言、未复现 bd2 历史三个 version fixture 的失败时间窗。

本轮取得了完整 Node/cmd 家族的真实线程、调度、映像、文件 I/O 和生命周期范围，但没有取得目标失败 case 的失败执行阶段，因为本机原库七 query 仍通过。历史 PR 失败、CI PR 红灯、macOS CPU/RSS 失败及全部版本门禁继续开放。

原件、授权、终态、原 stdout/stderr、压缩事件、进程清单和 verifier 位于 `.adg/evidence/OCX-0110-20261014-WIN-BD2-BASENAME-RUN/`。日常安装、真实数据、隔离状态摘要保持；收尾后 desktop、官方 Node、测试根和自有 session 均为 0。无合并、签名、发布、晋升或 TASK 关闭。
