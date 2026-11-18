# Windows r7 同一操作边界校验

窗口仍来自原调用入口。文件操作及日志写入使用同一个绝对截止时间，超时原样返回。

| 边界 | 保留的校验 | 合并的重复工作 |
|---|---|---|
| 逐文件退休或返还 | RecoveryWitness 在移动前后对 .codex 和 .grok 扫描，核对同父目录身份、路径、原 claim、全部保留文件及工作副本的身份、安全投影、字节与 children。移动前再次核对目标缺席及原 claim，按原保留句柄执行无覆盖 rename。移动及两侧校验任一错误都 poison 两棵树。 | 两棵树校验之间，在 ctx 不变且无其他操作的情况下去掉内层对被移动树的第二遍完整扫描。独立单树测试入口仍自己执行扫描。 |
| 准备恢复计划 | checked_recovery_plan 复核当前 Journal、实际原 claim、两棵树的新观察结果、原 storage plan 和 progress。随后原路执行 durable append。 | 计划验证过程中返回本次刚验证的计划对象。无需在生成同一事件之前重新提取内容相同的计划。 |
| 完整树写入 intent/current、结果及释放准备 | 在 intent 前、intent 后及 verified 后均执行完整树校验。在读取计划前新观察原件，核对恢复成员和内容。释放前后的独立实际检查沿用。 | verify_retirement_restored 已执行 verify_retirement_prefix，复用这次完整扫描并在同一观察中构造计划。 |
| 每次 durable append | append_entry_inner 仍完整回读 Journal、重放原规则，保留写入后的独立完整回读。后置释放证明仍完整复查原件及锁。 | 无。 |

临时状态由原保留句柄和原 claim 限定在单次调用内。两棵树的完整检查涵盖原生身份、固定安全投影、内容与目录成员。移动失败、边界漂移或 deadline 到期仍以原入口的错误结束。没有跨写入、跨窗口或跨动作的缓存。改动只在 Windows 原件模块内，供 Mac 审阅。同一来源正例在最终释放调用内仍返回 bounded-original-deadline，本批未建立完整生产耗时证明。
