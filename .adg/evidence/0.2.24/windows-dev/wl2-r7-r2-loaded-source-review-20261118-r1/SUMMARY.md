# r7 当前 r2 原载入版本的 Resume 路径复核

复核基于已有归档和 Git 对象。原输入清单的源码 357210891f14a1851881e6192bd09d7031471707、tree a284da0e1bd517daa26848857b8189bbc88838c0，以及 ocxd.exe SHA-256 568f9c7916a4d3c70ad8966cc54bcac263b2faa99fdb56c994d48dd42642bec0 一致。归档中 broker 75744 的创建时间、路径和映像文件摘要匹配原输入及后续只读记录。这里核的是映像文件和来源记录，未读取进程内存。

| 检查 | 证据和结论 |
| --- | --- |
| 原 Node 创建与唯一 Resume | windows_official_runtime024/process.rs:486–537 使用 CREATE_SUSPENDED 并在 resume 前绑定原 Job 和保持原 hThread。windows_official_lifecycle024.rs:911–965 先登记 Owner，成功 capture 后才能通过 before_resume 围栏调用原 Node 的唯一 Resume。host_process 和 windows_query_job 各自持有独立创建的进程与线程。 |
| 首错位置 | 原 start1 耗时 30,699 ms，原限额 30,000 ms，错误 windows024_originals_required/timed out。Owner stage 为 created-suspended 且 originalPhaseRetained=false。capture_originals 成功后才填充该字段，随后 Resume 路径前没有清除此字段的分支。首轮回执与 capture 未成功返回的控制流吻合。 |
| 原宿主失败后动作 | existing_qualification.rs:118–189 对首错立即返回，随后只进入失败控制。实际原事件止于 failure-control-listening。failure_control.rs:208–251 只分派只读 Status 和显式 forced cleanup。后者在原 Job 执行 forced242，不调用 Resume，不返回原件，也不生成正常 Stop 或 Closed。 |
| 后续独立观测 | 已归档的同 SID/session7 观察显示同一 Node 21500 创建代次仍在 Job 中，单线程 Wait/Suspended。历史 creatorResumed=false、active=1/total=1 与此前缀一致。当前仍缺原 Job 24 项类型及完整历史，raw50。单次线程状态、历史计数与映像文件摘要无法补足当前内核史。 |

源码复核项完成。完整从未 Resume 证据组合、R1 精确执行前提及单独授权仍待。保留原精确计划和全部现场，executionReady=false。此次没有重新运行入口、查询活进程、清理进程或移动原件。
