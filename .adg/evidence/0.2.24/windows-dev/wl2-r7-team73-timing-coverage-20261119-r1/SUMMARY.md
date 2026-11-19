# TEAM73 固定制品的恢复计时覆盖

待证伪假设：既有 Host stdout 中缺少准备扫描计时行，可以排除这次 Explicit 已进入原线程扫描。只读复核固定源码及已归档输入。原场景、产品源码、已归档文件和原期限均未改动。

- 固定源码 `36b70383eac5c44dffc51d34a4c4673cd8a3dbf7`，测试 EXE SHA-256 `1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8`，固定 launcher 源码 `a8d7a08002c3af21b2c7ad8f9bcc06cda9d107d0`。
- 证据 `d88b1839e` 的 `wl2-r7-range-host-20261119-r1/Host.stdout.log` 为 29,287 字节，SHA-256 `38c6bfde673ce1b7fcffc77b700d56ed646897316d32f67dd445a05eed470980`，匹配两个 `r7-initial-image-scans=` 标记、零个 `r7-recovery-timing=`、零个 `r7-preparation-image-scans=`。上批记录的现场只读双读与该归档一致。
- 同次 Explicit 由 `d7a03cd938b7add9667cce2d21009e9a1cfdde5a` 的 `wl2-r7-team73-event-projection-20261119-r2` 绑定。服务端记录 `refused / TimedOut / raw=null`，保留客户端 9,999 ms、测试 raw101 和 launcher 原首错。

以下 Rust 文件相对于 `apps/desktop/tauri/src/infrastructure/windows_client_originals024/`，行号基于固定源码。

| 固定位置 | 可观察的边界 |
| --- | --- |
| `recovery_worker.rs:462`、`:585`、`:596` | 期限检查、Running 发布、原线程和隔离校验均在计时 guard 创建之前。测试仅在部分捕获且已有待持久化附件时创建 guard。 |
| `recovery_timing.rs:167`、`:179`、`:79` | 单次完整扫描返回后才加入缓冲。guard Drop 才输出 `r7-recovery-timing`，即使缓冲为空也输出该标记。仅存在扫描行时才额外输出 `r7-preparation-image-scans`。 |
| `v3/capture/preparation/bound.rs:81`、`:90`、`:91`、`:98`、`:105` | 三个完整 source 围栏夹住两次完整 V2 引用读回和同动作／时钟比对。共用 deadline 只证明时限来源相同，对象身份、覆盖和前后条件仍需单独证明。 |
| `recovery_worker.rs:621`、`:628`、`:639`、`:663` | 计时 guard 涵盖附件持久化、写后引用与原范围复查和投影发布。独立附件回读与服务端最终重验在别处完成。 |
| `test/windows024-native/launch-r7-partial-owner.ps1:237`、`:344`、`:416`（launcher 提交） | Host stdout 由异步复制写入，复制任务的等待发生在宿主退出后。现有活宿主日志只能证明已收录字节中的标记数。 |

假设被固定源码反证。缺行兼容未登记、未接手、计时启动前退出、正在计时范围内或输出尚未收录。无法由此定位 worker 实际接手、任一完整扫描或 V2 附件持久化。现有证据也不足以移除分隔两次引用读回的任一完整 source 围栏。

请集成线在**新隔离样本**区分阶段进入与返回。优先保留服务端注册结果、原线程首次消费和隔离校验、三次围栏进入及返回、两次引用读回、附件 flush／读回及写后绑定复核，最后是到期重验及回复。观测应在原单调期限内使用有界测试缓冲，服务端结束或超时时可快照已发布前缀，明确未观察、进入未返回和已返回。不要让前缀的可读性依赖原 worker 走到 Drop、正常返回或附件生成。保留原文件内容校验、10 秒期限、原始首错及未知拒绝。
