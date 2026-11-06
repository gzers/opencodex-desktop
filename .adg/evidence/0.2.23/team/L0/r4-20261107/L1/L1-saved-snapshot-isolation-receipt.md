# L1 saved-only snapshot隔离回交

2026-11-06，Asia/Shanghai。提交8594f201f139c88a589f3718d63e2bd8eaf89348，父b4c38f22cccb3884aaa04b183f39f02dd5e17a89，codex/0.2.23-model-matching。仅store_v2.rs/consumer_v2_tests.rs，工作树干净；未改L0 with_saved_config/capture/public gate/transaction、命令或UI。

接口：

~~~rust
StoreV2::snapshot_saved(&self)->Result<StoreSnapshotV2>;
LockedStoreV2::snapshot_saved_unlocked(&self)->Result<StoreSnapshotV2>;
~~~

原snapshot_unlocked和新增saved方法共享私有snapshot_with_draft_unlocked(include_draft)函数，不复制解析器。includeDraft=false在调用任何DRAFT_PATH路径/安全检查/metadata/open/read之前直接跳过；draft=None，其digests.files[DRAFT_PATH]只填hash(empty)占位。Library/Policy/ledger/mapping四个真实文件照原严格JSON/schema、类型与相互配对校验。文件缺失的seed/空Policy规则不变。

snapshot_saved只读，不恢复事务、不检查未知draft格式、不追随draft符号链接、不回写/修复任何业务文件。Store模型事务锁仍照原API获取；调用者已有config→model锁时用_unlocked避免重入。

完整snapshot与所有mutation/CAS仍includeDraft=true。save_library/stage_draft/prepare_save必须完整读取并校验草稿；saved digest占位不是写权限，也不能绕过坏draft准备半份Policy/ledger。

正常完整应用crate源码快照26消费者测试通过（原21+新增5），0失败/忽略，1305无关项过滤。新增真实temp-root验证：

- 四个真实saved文件+脏草稿：saved Policy/Library/ledger/mapping及其四份原字节digest与基线一致，draftNone，不泄漏未保存名称；完整snapshot仍可返回有效草稿。
- 草稿坏JSON/过新schema：saved读取成功且原草稿字节不变；完整snapshot失败。
- Unix草稿符号链接/悬空链接：saved不检查/追随目标，完整snapshot拒绝，链接与目标原件保持。
- 坏Library/Policy/ledger/mapping不因saved API跳过draft而被放过。
- 持model锁测试_unlocked；以saved占位digest调用prepare/stage/save_library仍因完整坏draft被拒，库/Policy/ledger不写。

严格Clippy --lib --tests --no-deps -- -D warnings、rustfmt/diff检查通过。两文件与受测完整crate快照逐字节核对，原Cargo.lock未改；无mock v2实现。证据saved-snapshot-isolation-tests.log、saved-snapshot-isolation-clippy.log同目录保留。临时根自动清理，无服务/监听端口、真实配置/网络/模型调用；未push/发布、未写共享docs/.adg。

L0后继：with_saved_config切snapshot_saved_unlocked；遇model_transaction pending只报告待核对，不在导出/同步capture预览隐式恢复配置。模型页完整snapshot在有授权保存事务下的恢复继续由L0保持，本批没有更改其策略。

前插任务前的Agent只读分析已回交：L1-agent-runtime-v2-wiring-analysis.md，SHA256 a9e92fa3eff8bf60a12bf391a855f8e3efb821dfa19af5d8b9acf37eb7169e7b。报告基于当时L0工作区观察，不是最终Git冻结；本批saved API是对其保存来源边界的补强。Agent runtime/execution-directory新增模块尚未编码，等待L0明确后继写集/基线，不以纯只读分析证明真实AI调用。

