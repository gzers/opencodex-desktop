# L1 R1 消费者批次回交

2026-11-06，Asia/Shanghai。角色L1；MATCH-02/03实际配置适配贡献。当前完成授权消费者批与L0要求的严格重复JSON键修复；公共命令、真实统一保存门/日记/补偿与界面切换由L0/L3继续接入，不以本批测试关闭整项AC。

## 基线、工作根与提交

- 代码根：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes。
- 分支：codex/0.2.23-model-matching。
- 按L0要求merge 1370afb551d6e5189050975ed5b54e4e0de9d20a，合并SHA 8df83593f7091ce816b26658e05db9bac551d1c4。
- 随后merge独立公共注册6e031e109，合并SHA 74649027f6d98b96a9a878c562afee47ea64976f；未读取/合入L0目录与运行包的未提交内容。
- 318786bef707732165480fe2ec2eb9c1f7609c4b：v2 provider读取、分组/配置物化、StoreV2与消费者测试。L0已回报cherry-pick并准备注册。
- 8523f7c90ef67df7eefa7e2c7ce813c976b4cdc1：编译入口校验完整映射；模型跨协议移动时保留公开per-model对象内附加字段。
- 3e32bb1e437168dc248177c40b49ebcfc08432ee：递归重复JSON键拒绝，Store/portable/seed使用严格解码，新增测试。

后两笔依次cherry-pick；不得只取最后严格解析笔而遗漏8523的编译/字段保全修正。全部提交在本机共享Git对象库可读，未push、未创建发布/tag。

## 精确写集

现存provider_import.rs/protocol_groups.rs只追加v2入口，原v1函数未改。新增compiler_v2.rs/store_v2.rs/consumer_v2_tests.rs。严格解码增量还改原授权schema_v2.rs/portable_v2.rs/v2_tests.rs。所有路径位于apps/desktop/tauri/src/modules/model_config/；其它源文件、公共mod.rs、命令、共享docs/.adg未写。

## 实际实现与接口

1. provider_import::read_policy_v2读取实际原版Value、明确的ProviderBindingV2稳定channelUUID/providerId、可信v2 Policy/ledger/mapping，输出source/explicit/Manual/AutoRead分层状态。实际原件的模型库存、选择、节点、名称、公开参数为来源，认证值不进入输出DTO。原始含斜杠ID保持，selectedModels运行slug优先映射到已知精确原ID，无唯一证据不做有损还原。受管物化值按ledger恢复原source；实际外改仍是source，不能变成人工覆盖。
2. protocol_groups新增MappingFileV2(schemaVersion=2)及RuntimeMappingV2(channelId/protocol/wireProfileDigest/providerId/anchor)。校验唯一身份和显式anchor，plan_mappings_v2只使用L0已分配身份，不创建名字或文件。v2路径不走渠道protocol回退。
3. compiler_v2::compile_file_v2消费同源library/before/intended、L0已执行明确改名/连接意图后的原件Value、已分配完整mapping及实际认证允许集，返回CompiledFileV2。真正生成分协议provider配置、modelAdapters等per-model参数、选择/顺序/默认引用及ledger targets。保留未知原件、其它渠道、认证和原始ID；clear恢复source。模型移回原组前转移PUBLIC per-model对象，保留modelCapabilities扩展字段。碰撞/认证不符拒绝，原件不变。
4. StoreV2只接已解析D根，不扫描目录。库缺失返回seed，policy缺失空schema2；有旧v1/损坏/未知版本即拒绝，不改写旧原件。库独立保存用同一model-v2锁、原子写和读回，draft独立CAS可持久写但不写运行文件。Policy/ledger/mapping保存只prepare返回PendingWriteV2，不自行提交半份配置。
5. 严格JSON：schema_v2::parse_strict_json_v2从字节递归拒绝对象重复键，包含Unicode转义后同名键、数组内对象重复键。保留32MiB预算、serde递归上限和尾随文档拒绝。错误code=model_json_duplicate_key，错误不回显恶意键名或原件。Store五类文件、portable v6、内置seed均先严格解码；没有宽松Value后再检查的旁路。

### Store接口（与L0已接受签名一致）

~~~rust
StoreV2::new(root:&Path)->StoreV2;
StoreV2::lock(&self)->Result<LockedStoreV2<'_>>;
StoreV2::with_locked(&self, FnOnce(&LockedStoreV2)->Result<T>)->Result<T>;
LockedStoreV2::snapshot_unlocked()->Result<StoreSnapshotV2>;
LockedStoreV2::save_library_unlocked(&SaveRevisionRequestV2)->Result<LibraryV2>;
LockedStoreV2::stage_draft_unlocked(expected:Option<&ReadBaseV2>, next:DraftV2)->Result<DraftV2>;
LockedStoreV2::prepare_save_unlocked(
    expected:&StoreDigestsV2, policy:&PolicyV2,
    ledger:&MaterializationLedgerV2, mappings:&MappingFileV2,
)->Result<Vec<PendingWriteV2>>;
~~~

StoreDigestsV2.library为canonical库摘要；files为五份持久文件原字节摘要，缺失hash空字节，不得混用。DraftV2={schemaVersion:2,base:ReadBaseV2,policy:PolicyV2}。Ledger落盘封装为{schemaVersion:2,ledger:MaterializationLedgerV2}；mapping封装为{schemaVersion:2,mappings:[...]}，旧数组格式拒绝。

固定路径：manager-state/model-family-templates.json、model-policy.json、model-config-draft.json、model-materialization-ledger.json、model-protocol-mappings.json。事务锁对root/manager-state/model-v2调用TargetFileLock::lock，生成其.lock；根须已存在。L0按全局admit→file_saves admit→rename admit→config锁→StoreV2锁顺序接入。LockedStoreV2持锁对象上的_unlocked方法不再重入锁。

PendingWriteV2={relative_path,before:Option<Vec<u8>>,after:Vec<u8>}，prepare当前返回需要变化的Policy/ledger/mapping三类写集。L0将next_config与这些字节置于同一日记提交/读回/补偿；draft在保存后如何标记新基准/清理，由统一保存门明确收口，不得将旧draft继续当作保存后新基准。

### 运行分组与名称接点

非anchor派生provider使用已有官方note字段保存v2身份marker。重编时已有目标provider marker不匹配报碰撞，不能以同名覆盖。原anchor保留原note；anchor选择必须由L0显式映射。新分组的认证复制仅在L0明确提供的同源认证允许集授权下进行，不从渠道默认协议猜凭据适配。旧非anchor组无模型后变disabled并清空本线已管模型字段；不删provider身份/认证及其它顶层材料。

编译器不执行改名事务。测试中由调用方先完成已分配provider键/根引用改名，L1再按稳定逻辑UUID编译；L0继续拥有channel_rename/rename_effects/execution_directory等实际名称/凭据/目录事务。

## 验证与实限

用完整应用crate源码测试快照、原Cargo.lock离线编译。快照由本线已提交基线git archive构成，再覆盖本批实际源码，唯一额外源码差异为mod.rs追加compiler_v2/store_v2/consumer_v2_tests三项注册。没有mock模型/文件锁/atomic_write/provider读取实现，没有真实网络、CLI或日常配置。测试D/H来自tempfile::tempdir。

最终43项通过：前批29核心+本批12消费者（含Store重复键）+2严格JSON/portable测试，0失败、0忽略；1247其它测试按过滤条件未执行。消费者覆盖：

- 实际配置读入→模版自动草稿→原版字段物化→Store准备写集→临时D/H模拟L0提交→重开恢复source→clear删除物化覆盖；检查prepare阶段业务文件零写。
- Chat/Responses实际拆分为两个provider；原始ID/选择/默认和picker路由保持，重复编译稳定，清除协议后回组；附加能力子字段来回保留。
- 原件认证/未知字段/其它渠道保留；source DTO不含认证值；源参数未转人工；外改capability仍是ExternalFileEdit来源。
- L0先改名后的配置编译、已知运行slug→精确原ID、映射碰撞与认证不符拒绝。
- 实际原子库保存/读回、原revision不改、stale库/草稿拒绝、同一事务锁竞争、symlink路径拒绝。
- 旧v1/损坏配对拒绝且原件不改。五类Store文件重复schemaVersion及嵌套source重复均拒绝；portable重复version/source拒绝，转义同名与递归深度/尾随文档边界验证。

命令（完整路径见manifest）：

~~~sh
cargo test --offline --locked --manifest-path <snapshot>/apps/desktop/tauri/Cargo.toml --target-dir <batch2-target> --lib v2_tests -- --skip sync_v2_tests
cargo clippy --offline --locked --manifest-path <snapshot>/apps/desktop/tauri/Cargo.toml --target-dir <batch2-target> --lib --tests --no-deps -- -D warnings
rustfmt --edition 2021 --check <本批Rust文件>
git diff --check
~~~

最终正常crate快照43测试与Clippy拒绝警告通过，格式/diff检查通过。原Clippy第一次失败因plan_mappings_v2只被测试调用，8523已让生产compile入口实际调用，不以allow(dead_code)压制；失败日志保留。最初快照生成命令遇到主机Python不支持tarfile.extractall(filter=...)，未编译即失败，后改为验证归档成员无绝对/越界/链接后解压固定Git归档；未改共享源码或配置。

实际Tauri公共命令/统一保存门、UI调用链与日记故障恢复仍由L0后续集成验证。这里的临时D/H提交helper只模拟调用方完成同批文件写入，不能称L0真实多文件原子提交/补偿已通过。本批不发布，不运行桌面服务或推理。

## L0后继接线

- 注册compiler_v2/store_v2与consumer_v2_tests（L0已回报正在执行，本线不改mod.rs）。
- 原件v2读取必须先parse_strict_json_v2(raw_bytes)再传provider_import::read_policy_v2；若先宽松serde_json::Value解析，重复键会在抵达本线前丢失。此config_file读取边界仍归L0。
- template公共save入口不接受UI提交revisionId/time/origin；L0后端生成并设origin=Manual，本线接受此权限边界。
- 实际保存门捕获原件字节digest/全部draft-source-policy-library-root基准，持有冻结锁序后调用编译和prepare；同一日记包含JSON与Policy/ledger/mapping，并明确收口draft。
- 继续按真实读取/保存重开/取消/失败恢复实际组件验证，不把本批适配测试等同界面或正式命令完成。

代码已直接回报L0。没有启动服务/监听端口，tempfile测试根已自动清理；受控源码测试快照、编译缓存和日志保留作证据。本地未上传。所有提交工作树干净，实际文件摘要另见本批manifest。

