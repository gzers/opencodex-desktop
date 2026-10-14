# Windows bd2 持续授权录制与容量修正

2026-10-14，Asia/Shanghai。用户明确授权“批准修正版后续都可以管理员后台 ETW 录制”，适用于本目标内后续必要的修正版后台有界诊断，无需逐轮再请求管理员录制许可。每轮仍先核验源码/CI/制品身份和当前进程，独立输出目录，一次原库/120 秒外层上限；不改原 2/4/5/8 秒、默认并发/capture、断言或性能预算，不无限重跑求绿，不自动操作 UAC。前台原生测试继续需要人的新开始信号。

## 629 实际完整录制

采集器 `629b9c6f913b5e04badc9ce747f6bb8ba247fd89` 已按同 SHA 双 CI 准入。使用原失败 PR `38537166560` 的测试 EXE/SHA256 `497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3`，官方 Node 24.21.0/SHA256 `ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32`。

21:23:33.6150603 创建提升控制器 PID 22652；21:23:35.9662199 创建原库 PID 55592。句柄映像查询与预期匹配，测试根实际令牌 `elevated=true/elevationType=2`，不是仅由控制器推测。消费者 21:23:35.807–21:23:46.869 的录制正常完成、root ProcessStart 已见，stdout/stderr 保存、consumer/writer 排空、唯一自有 session native STOP=0、kernel final loss/buffer loss=0，外层强制清理 false。不能据诊断用测试根管理员令牌声称产品双权限门禁已通过。

原完整库默认并发/capture、0 参数、无预热，完成 529 passed / 1 failed / 4 ignored / 0 filtered，约 10.50 秒、exit101。七 query 清理夹具均通过；唯一失败 `commands::workspace::tests::local_documents_are_frozen` 的 `license.opened`，仍是原制品编译时 runner 路径在本机不存在，不造目录或改断言。**没有复现原 bd2 七 query 失败。**

## 容量缺陷与覆盖限制

JSONL 183278 条、67291856 bytes，SHA256 `710bc329c01453fef1f8118eeeef46481fb008e16102429119a3844b8a7554aa`；无损 gzip 2889091 bytes，SHA256 `bf62ed9b7860310d8fc3716a0384c7bc60061b2fda3e54b6defa0017b687339f`，解压与 runtime 原件逐字节一致。全部原始字节保留。

已保存记录中测试根 178269 条，占约 97.27%；根测试的高频调度/文件 I/O 将 64 MiB 序列化额度在原时刻 21:23:35.9663028–21:23:36.6049333（约 0.64 秒）耗尽。`droppedRecords=1339173`，后续终止记录缺失；kernel loss=0 不能抵消这些应用序列化丢失。

另有真实上限实现错误：预算按 UTF-8 每行 +1 计量，Windows StreamWriter 实际 CRLF 每行 +2，文件超过 64 MiB **182992 bytes**。每行额外 CR 共 183278 bytes，扣除后 67108578 bytes 才在原声明限额内；不把实际文件宣称满足上限。

八条进程开始记录为测试根加七 Node。部分 Node 首次 API 观察已退出或不可打开，仅 ETW 生命周期；两条尚有 `exitFILETIME=0` 的句柄观察也不能替代完整终止/后续时间窗。原记录时间与到达延迟均保留，不据该成功场景推断旧失败根因或作性能样本。范围只持久化已准入自有家族和允许路径，未写整机 ETL、无关命令行/SID/其他调度参与者 ID；原始高频记录有丢失，完整过滤/失败覆盖结论不提升。

首次测试结果分析脚本仅选出一个夹具，初分析保留，按原 stdout 的两个精确 Windows 模块全名重新枚举，确认七项均通过。原件不改；机器摘要以最终分析及 verifier 为准。

## 修正与下一轮

代码提交 `3a91c175a6c42d1cec16ef3eeb1d799d9abe4aac` 已推送到 `feature/0.1.10-windows-query-etw`，仅改原三份 `test/smoke/windows-owned-etw*`。保留根进程身份、进程/线程/映像生命周期；高频调度和文件 I/O 限定已准入 Node/cmd 后代，省略的根高频事件单列计数，不据省略宣称根无活动。目标仍是查询子进程失败时间窗的原线程、调度、初始化及 I/O；不降低产品门禁。

writer 固定 UTF-8 无 BOM + LF，与真实输出字节共用预算，queue 未入列不消耗字节额度；仍 64 MiB、32768 queue/IRP、120 秒原库外层上限。新增实际内存写入汉字/LF/硬上限/queue 满后预算、PID 重用及延迟原时刻的后代范围检查；本机编译及 78 检查通过（原 58 +10 后代 +10 字节），摘要/目录/普通权限拒绝保持。只查询自身句柄和内存流，无原库/Node/ETW新执行，修正后实时行为待新 SHA 录制。

新 push `38572890687` / PR `38572902717` 初始仍 in_progress，PR #5 已更新到新范围并独立读回 draft/open/unmerged，无 auto_merge。新 review-only 入口 `record-request-descendant.ps1` 固定 3a/原失败 EXE/官方 Node/独立 runtime 根，消费新 `OCX-0110-20261014-WIN-BD2-VOLUME-CI/ci-admission.json`。待真实同 SHA 终态、实际检出/原日志准入后，按持续授权执行一次以验证已观察的采集器容量缺陷；不是重复完整库求绿或撤销旧失败。

原件、授权、分析、修正与准备验证封存在 `.adg/evidence/OCX-0110-20261014-WIN-BD2-ETW-IMAGEFIX-RUN/`。本次 session/自有根/控制器/官方 Node/desktop 收尾后均 0，以 PID+creation+映像核验，未操作无关进程。日常 EXE 与隔离 preferences/root/policy/schedule 前后摘要一致，没有新桌面实例或自动周期。

原 IMP-20/TASK-182 完整目标、八 required checks、macOS CPU/RSS failed、失败根因、原预算/样本数/计量口径、FZ50 脚本未获许可保持；Mac 继续后台，未正式原生交接，不关闭、合并、签名、晋升或发布。
