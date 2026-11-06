# L1 保存版本事务材料小批回交

2026-11-06，Asia/Shanghai。授权为content_versions.rs临时单写及新增测试；不修改统一保存门、命令、transaction或mod.rs。

提交4477dbc8bd40abc08a3d79ea532c0f99da76b1ec（父7abd03040768405b36ce6ad74ab7646b47154f88），分支codex/0.2.23-model-matching。仅apps/desktop/tauri/src/modules/model_config/content_versions.rs与content_versions_v2_tests.rs，后者由前者cfg(path)自注册。工作树干净，可单独cherry-pick到总控已集成7abd等价源码的树。

接口：

~~~rust
pub const SAVED_VERSIONS_PATH: &str = "manager-state/saved-content-versions.json";
pub struct PlannedVersionsV2 { pub before: Option<Vec<u8>>, pub after: Vec<u8> }
pub fn lock_versions(root: &Path) -> ModelResult<TargetFileLock>;
pub fn plan_save_at(
    root: &Path, before: &SaveState, after: &SaveState,
    time: &str, event_id: &str,
) -> ModelResult<PlannedVersionsV2>;
~~~

plan不取锁，不写盘，不读config/v1 Policy，不生成时间或event；只读版本map并以固定输入生成字节。调用方持外层file-save admission再取lock_versions，保留到统一事务提交/读回。锁是原版本目标的TargetFileLock：root/manager-state/saved-content-versions.json.lock；旧update也复用该锁函数，checkpoint与validate回调顺序不变。

map变换抽为私有apply_save_at，旧record_at_validated和plan共用；数量/8MiB预算与序列化共用encode_records。record_save/record_at_validated/observe/inherit旧行为、旧测试保留。没有第二份clock-floor规则；时间回拨后直到严格越过trusted floor才恢复可信，删除保留floor/tombstone，revision仍按旧saturating规则；同event同内容同原time重放不增加revision，冲突沿旧错误。

plan先用schema_v2严格JSON解码再按原BTreeMap<String,Version>类型解析，无schema封装/升级。原alias及缺scopeKey兼容按旧规则；只有真实状态变化才规范化。重复scope/嵌套键、坏类型/未知字段均拒绝且原件不改。

no-op：现有文件返回原before/after相同字节，不改空白或alias、不制造时间/revision/floor；缺文件返回before=None/after={}字节且不创建文件。L0若要求完全零创建，按业务SaveState前后相等跳过该write；不能把空map当作首次保存事件。原time/event必须在提交前冻结到统一日记，恢复只重放同字节，不以当前时间重造。

验证：完整应用crate受控源码快照，原Cargo.lock，实际TargetFileLock/atomic_write，只有临时D/H/版本测试根。20项content_versions测试通过（旧12+新增8），0失败/忽略；1281无关测试过滤。包括旧真实子进程退出恢复、版本锁超时后原时间重放、改名/丢回执/锁序回归；新增逐字节旧API/plan等价、无操作/重放/事件冲突、回拨/删除floor、严格解析/文件不改、版本锁竞争及plan连续调用不重入不写。

命令：cargo test --offline --locked --manifest-path <受控快照>/apps/desktop/tauri/Cargo.toml --target-dir <受控target> --lib modules::model_config::content_versions -- --test-threads=1。正常crate Clippy --lib --tests --no-deps -- -D warnings通过。rustfmt --config skip_children=true --check及git diff --check通过。

证据：同目录content-version-plan-first.log（新增8通过）、content-version-full-tests.log（20通过）、content-version-clippy.log（通过）。本批无失败测试、无skip。源码和测试与受测快照逐字节核对，临时根自动清理，无服务/监听端口。源码快照/日志/构建缓存受控保留。

未触真实配置/日常数据/上游，未push/发布、未写共享docs/.adg。本线仅返回准确版本字节；L0将其纳入config/Policy/ledger统一transaction及恢复，并负责版本时间实际消费者的最终验收。

