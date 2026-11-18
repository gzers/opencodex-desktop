# r7 准备失败读取与原包完整扫描

调用链核对基于 `0d0c9d29ccd9075d59f92350c158c256c835924b`。文件名相对于 `apps/desktop/tauri/src`。这里记录源码调用顺序；r2 保留现场沿用 `wl2-r7-r2-readonly-20261117-receive-r1`，未进入该入口。

| 顺序 | 实际调用与覆盖 | 全树完整扫描 |
|---|---|---|
| 1 | `windows_official_lifecycle024/start_worker.rs:331` 核原 Start worker、原 Owner、session、scope、action、launcher 和未 Resume。`windows_client_originals024/recovery_worker.rs:554` 进入 partial 分支后调用 `prepare_before_resume_process`，归入原 Job／launcher 准备事实。 | 0 |
| 2 | `windows_client_originals024/adapter/before_resume.rs:150` 检查原 Owner、未 Ready、未登记后继及原 capture session。 | 0 |
| 3 | `windows_client_originals024/v3/capture/preparation/bound.rs:77` 第一次 `original_read_fence`，核原进程、anchor、失败动作、Source、routes、journal context，最后复核进程。 | 1，若 Source 已登记且前置核对通过 |
| 4 | `bound.rs:78` 读取绑定的新 action 和失败 action／时钟。第一次 `v2_reference_data` 读取原捕获成员和引用。 | 0 |
| 5 | `bound.rs:81` 第二次 `original_read_fence`，覆盖与第 3 步相同。随后第二次 `v2_reference_data`，比较两次引用、action、失败 action 及原时钟。 | 1，同前提 |
| 6 | `bound.rs:89` 第三次 `original_read_fence`，覆盖与第 3 步相同。以同一原 Owner 生成 V2 Intent，再编码 Pending。 | 1，同前提 |
| 7 | `recovery_worker.rs:565` 登记内存记录并核对 Owner binding。已有私有 D parent 时核对恢复 action 与失败 action、同线程创建私有附件、持久写入并逐字节读回两份记录，最后复查原 Owner 和截止。 | 0 |

`windows_client_originals024/source.rs:248` 完整顺序为 `Owner.check_current` → broker 无外层 Job／镜像 → 固定 2.99 SHA 和 Owner scope → `program_selection::verify_current` → `program_image::verify_until` → 原 Current 句柄、字节及其安全投影 → 原 Profile/D/H/I 与绑定序列化一致性 → `Owner.check_current`。Owner 通过 `windows_official_lifecycle024.rs:655` 的 `source` 检查令牌、作用域 pin、安装选择、稳定入口和原进程绑定。它只带有界选择元数据及原进程对象核验。routes、journal context 和 `v2_reference_data` 也没有完整整包验证调用。

同一次动作只有一个 `BeforeResumeRecoveryWindow::deadline()`。每一次完整调用使用该 deadline，失败会截断后续调用。源码只支持“若已登记 Source 且一直通过，达到三次”的结论。三次之间执行实际的原件引用读取与前后比较。目前没有跨越这两次读取、同时绑定整包对象集合与原字节的连续保留句柄证据。本轮保留全部三次及其前后检查。

完整文件加 manifest 的固定副本单扫为 2634.174 ms，证据见 `wl2-r7-full-msvc-scan-entry-20261118-r1`。仿真整链和部分解锁分别 raw101／10144 ms 和 raw101／10039 ms，首错分别为最终释放 `bounded-original-deadline` 和 Journal 第 138 次读回未完成，证据见 `wl2-r7-recovery-scan-count-20261118-r1`。这些组件数据无法推导真实 Owner 同一线程的恢复耗时。

原 Owner 的 `windows_official_lifecycle024.rs:1190` 只从旧 r2 `ExistingQualification` 拿取私有 D pin。`windows_owner_host024/existing_qualification.rs:49` 精确绑定 `BASE`、SID、session 7、Profile、I/D/H 和原 broker 镜像。`windows_client_originals024/existing_qualification.rs:350` 和 `existing_qualification/fresh_fixture.rs:95,139` 还要求原合成基线、任务协调对象与已提交历史摘要相符。当前 R0 的 Job 仍有 24 项未知、raw50，三件原件仍保管。真实回执所需输入为一次按合同入场的普通桌面实际 Owner、独立新任务文件、准确原件基线和由原 Start worker 保全的 V2 捕获前缀。当前 R1 仍需独立证明和授权。
