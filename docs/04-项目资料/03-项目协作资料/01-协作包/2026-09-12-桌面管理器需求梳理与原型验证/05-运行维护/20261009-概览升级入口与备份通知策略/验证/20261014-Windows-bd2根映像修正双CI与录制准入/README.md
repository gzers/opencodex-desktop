# Windows bd2 根映像修正双 CI 与录制准入

2026-10-14，Asia/Shanghai；关联 IMP-20 / TASK-OPENCODEX-DESKTOP-182。代码仍为 `feature/0.1.10-windows-query-etw@629b9c6f913b5e04badc9ce747f6bb8ba247fd89`，未启动修正后的第二次本机 ETW 录制。本文仅补同 SHA 的 CI 与实际检出证据。

## 已核验终态

push `38567345341` 和 PR `38567352891` 均 `completed/success`，五必需 job 成功、main-only build skipped。每轮 Mac 后端 35 组 / 900 passed / 6 ignored、Windows 后端 35 组 / 627 passed / 7 ignored。Windows 最终 EXE 首屏步骤成功；这是远端 runner 的通用检查，没有执行本机 C# collector、53 项纯检查或新增 5 项自身句柄检查，不替代真机门禁或实时采集。

push 的实际 backend/backend-windows checkout 均为 `629b9c6f913b5e04badc9ce747f6bb8ba247fd89`；PR 均为合成检出 `513cea4a790b5c7229eff3f8e44ba39031d28f46`。两者整树均 `30e3be09e903f9c9bc8adf2866f65ed3e85aedec`。实际 SHA 来自对应原 checkout 步骤日志；GitHub git commit 元数据分别保留并验证 tree 与本地固定源一致，不把合成提交直接叫作 head，也不由 tree 相同推定二进制相同。

完整原日志 ZIP：push 292369 bytes / SHA256 `efdd94d38e6cf1bea52217c6319a05d2f03e448c53fc94ffba73599df7e54081`，PR 296142 bytes / SHA256 `d6593e57811ea71d1bffef0a4681d1720e3da8d92ccd3667ad9624c9ebe2dd7f`。两份 CRC、安全条目和四类 backend checkout/test 原日志字节摘要均通过；冻结在 `.adg/evidence/OCX-0110-20261014-WIN-BD2-IMAGEFIX-CI/`。

有界只读观察器跟踪原两 run，API 间歇失败保留为读取错误，不推断远端停止；随后重新读取同 run，确认任务继续。中途 PR Windows 后端回归已成功、仍在 release 构建，最终 21:14:55 观测到两 run 均成功；观察器正常退出。没有重派 CI、重启任务或把观察截止当作终态。PR #5 只补正文终态，独立 GET 读回仍 draft/open/unmerged、无 auto_merge，head/base/title 保持。

## 准入与仍缺证据

`ci-admission.json` 仅表示新采集器源码的两轮通用 CI、实际 checkout/tree 和原日志已核验，供前一证据目录的 `record-request-imagefix.ps1` 消费。它不授予管理员录制权限，也不证明 C# 实时采集已通过。

此前 abc 的一次管理员录制实际启动原库后在根映像校验中止，消费者未启动、JSONL 0 条、强制收尾，旧 `MainModule` 返回值与原测试 stdout/stderr 缺失；原件仍在 `OCX-0110-20261014-WIN-BD2-ETW-RUN`，完整清单复核通过。629 的句柄映像查询及异常输出修正尚未真实录制，本次原库、Node、ETW、本机桌面新执行数均 0。机器结果中的执行计数限定本机，远端 CI 实际 Rust/EXE smoke 另由原日志证明。

本机桌面/官方 Node 仍 0，第二录制输出根不存在；日常 EXE 与隔离 preferences/root/policy/schedule 摘要保持，原 TASK 3878 bytes / SHA256 `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93` 不变。本次只核验新 CI，不再执行纯检查或成功对照求绿。

下一项需要用户对**修正后第二次单次管理员后台录制**的明确批准；默认入口只展示方案，实际运行仍需真实管理员令牌、独立新输出目录、一次原库/120 秒外层上限和原 2/4/5/8 秒 / 默认并发 / capture。UAC 由用户手动确认，前台原生测试仍需新的开始信号。Mac 继续后台，未原生交接。

原 bd2 PR 七 query 失败、失败时间窗内自有活 Node / 调度 / 文件 I/O / 根因、macOS 性能失败及所有完整版本门禁仍开放；不修改原预算、TASK required checks/revision2/in_progress/pending，不以新 CI 绿解除旧失败，不关闭、合并、签名、晋升或发布。
