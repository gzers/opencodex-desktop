L1 / 复制、历史与队列导出最终收尾回执

本批已完成并分片回交：copy d34775a37d78c10c719dd178844248805caf1e99；本版事务history 904cfb221f594c26754e4bc307dd6be31ce91dc5；queued capture da4217a5b2bdfa158e872973fe7ecd41041855c0。总控已确认分别集成1b0f78bba、20c00f899及0aa02778c。

本线最终HEAD 851948bd4fc281982c69dfaaedcfccf2052deb99，codex/0.2.23-model-matching，/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes，工作区干净。正式依赖be0af917e(owned intents)/4c148a644(commands/resume)、ceda8f016(全crate格式)、e1a663c110(命令冗余closure)均正常merge，不含dirty依赖。

最后队列修正仅export child/tests两文件。ConfigDraftFileCaptureV2.capture_with_intents消费owned DraftFileIntentsV2{edits,channelIntents}，在实际draft/base/source/root/library/observedbaseline锁保护下复用SaveRequest纯render/compile再v6 scoping/public capture。模型override/alias/exactbinding来自当前本地typed队列快照，进入冻结导出文件；不接UIpolicy、不rematch、不write source/draft/library/version/WAL、不为无Key导出先stage未存Key。普通capture empty队列继续原scoped-only路径。

FileOwner真实dispatch/codec已覆盖。修正原owner fixture的业务边界：全部原D/H文件路径/hash保留，0删除/0变更，仅允许这个operation UUID的manager-state/file-transfer-v2/<id>.sealed新增；没有删除NoBusinessWrite断言，也没有把生产owner退成无持久操作。新fixture注入MemoryRecoveryKey避免实际keyring；sealed只能有version/id/nonce/ciphertext，无plaintextsecret；实际reopen_export验证认证解封，篡改ciphertext明确file_recovery_authentication拒绝。比较失败只输出hash/path，不dump业务内容。

最终较宽回归113/113：配置边界90、FileTransferOwner23；此前12export定向全过。完整cargo clippy --lib --tests --no-deps -- -D warnings、cargo fmt --all -- --check、git diff --check全通过。较宽测试HEAD 417cad78798d442998d7f433f84f840c20432646；后合e1只将同一history list函数的冗余closure换为函数指针，8个列明feature源文件与较宽测试HEAD逐字节一致，最终Clippy/全格式在最终HEAD通过。未重复全量1855测试冒充整版验收。

较宽复验命令：

~~~sh
cargo test --offline --locked --manifest-path apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib modules::model_config::config_file::v2
cargo test --offline --locked --manifest-path apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib modules::model_config::file_transfer_v2
cargo clippy --offline --locked --manifest-path apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib --tests --no-deps -- -D warnings
cargo fmt --manifest-path apps/desktop/tauri/Cargo.toml --all -- --check
~~~

日志queued-export-config-regression.log、queued-export-owner-regression.log、queued-draft-export-final-clippy.log；原跨写集Clippy失败queued-draft-export-clippy.log和L0全量第三轮失败/tmp/ocxd023-full-lib-third.log保留。原copy/history证据仍各自回执拥有，不覆盖。全部源SHA、日志SHA与明确来源见L1-final-consumer-manifest.json。

L3已收到reuseSourceKey显式false默认、history DTO/只读prepare与guarded恢复draft保存合同、queued export顶层edits/channelIntents及失败/取消保留本地queue建议；正式命令与owner注入由L0完成，本线不修改UI/共享命令。日期字段缺失仍null，不补mtime。

测试临时D/H与MemoryRecoveryKey，不真实推理/同步，不修改日常配置、不发布。TempDir析构与测试进程退出清理，保留target和受控日志。本批授权工作全部回交；Windows、软件UI、全量整合及正式制品由总控继续。产物本地未上传。
