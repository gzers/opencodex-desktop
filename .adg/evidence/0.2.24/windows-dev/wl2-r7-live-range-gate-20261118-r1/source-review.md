# r7 固定桌面制品的现场原锁核验

待证伪判断：固定的 nine-r2 制品能在显式动作入队前，由原工作线程重新读取原 claim/N/L/K，并给出与 Host 回执同代的现场证据。

输入为 main `040f411e6d48d97bde6e7b2e6c22860e547b6173`、治理 `65835a0b3726d50f18018fbb8d8b8f6375ad286b`、TEAM59／IMP198／r7 及固定宿主源码 `daae8ffff7f304ec28b9c0214db26691e930a395`。只读核对该提交中的调用路径。

| 入口 | 固定源码中的事实 |
| --- | --- |
| Host 注入 | `windows_client_originals024/v3/capture/test_failure.rs:128` 在注入回执提供 `originalClaimAndNlkVerified`。保存的回执对应注入时刻。 |
| DiagnosticBefore | `windows_owner_host024/recovery_control/server.rs:290` 读取 `before_resume_worker_status` 和同样本 Status。`windows_official_lifecycle024.rs:174` 与 `start_worker.rs:183` 从原登记读取工作线程投影。这里能验证原线程和有效登记仍可观测，返回值没有原锁的重新核验。 |
| Explicit | `recovery_control/server.rs:312` 接受单次显式请求。`recovery_worker.rs:592,613` 在原工作线程上读取原范围，并在写 V2 附件后复核。`coordination/retained_start.rs:25` 在这条路径上验证原对象并读取同代范围事实。 |

结论：当前制品具备初始回执、实时线程状态和显式动作内重新读取。能够在显式动作前提供原线程重新读取的入口尚未找到。Host 可独立按固定配对完成。若合同要求先有现场原锁读回，现有只读回执不足以进入 Explicit。需要确认是否采用初始原锁回执、同一存活原 Owner／线程、无恢复动作已派发与原线程持续持有对象的组合证据，还是须增加独立只读入口。后一方案可限于隔离测试制品，由原线程复核原失败绑定及同一 claim/N/L/K，在一个原 5 秒窗口内返回数据。保留公用 Status 当前投影、显式动作原 10 秒、公开 Start 拒绝和未知拒绝。该入口无需持久写入、恢复许可或范围释放。涉及重建时记录新源码和制品配对。

现场：`preflight-readonly.json` 记录实际会话、旧进程存在、暂存五文件与三目录核验，以及 nine-r2 输出为空。查询旧 PID 的存在性未验证精确创建 FILETIME。没有运行 Host、管理命令或旧现场动作，没有终止进程或删除文件。

后续输入为治理 `842057724a030d97901b5e1883eec5cf203ecb32`、TEAM60／IMP199，继续要求先独立证明同一 PID/FILETIME、原 Owner／线程和 claim/N/L/K，再执行 5／10／5 秒序列。暂存入口第 294–305 行在每个管理模式下校验 live PID、会话、FILETIME 和镜像。第 422–424 行核对 DiagnosticBefore 的线程投影及未登记 recovery worker；此处仍没有重新核验原生范围。管理端的独立回执核验脚本将两项现场证明显式返回 false，离线结果只用于准入检查的输入。

用现有只读 `archive-r6-process-facts.cs` 从控制端查询 nine-r1 的 PID 95724，原生返回 raw5。原结果见 `controller-query-nine-r1.json`。这是观察端权限失败，PID 存在仍不足以确认精确创建时间。没有提高权限，后续 nine-r2 需要普通桌面实际回执。
