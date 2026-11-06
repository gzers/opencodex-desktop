# L1 R1 第一批回交

日期：2026-11-06（Asia/Shanghai）。角色L1；MATCH-02/03纯核心贡献，待L0集成。不是整项AC或实际命令/保存链通过。

## 来源与提交

- 文档合同：0c7999b57e0591bcacc1dfa80602bf1cbdffa9e8采用OCXD-023-MATCH-G0-r1，原摘要1db45cc502033c28fcc77a07d25aceab8d30da7bd8ed611dd48dabdc9e1ca7b5。
- 代码起点：0c57500f8f46ea50655c782c3f04334a39167abb。
- 工作根：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes。
- 分支：codex/0.2.23-model-matching。
- 首阶段：fa07451c9cb742e33a095bcbefd330c462d6f730；L0已集成其等价提交04c3ef68e并注册模块（总控回报，L1未代称已检验总控测试）。
- 增量：4b34851dddd41f341691f6e0e338d09f2eb920d7，可在首阶段已集成树上单独cherry-pick。工作树干净；没有push、发布或新tag。

## 实际写集及功能

仅apps/desktop/tauri/src/modules/model_config/中的schema_v2.rs、matching_v2.rs、source_v2.rs、portable_v2.rs、seed_v2.json、v2_tests.rs。mod.rs/旧schema、matching、compiler、store、配置文件/命令、docs/.adg未改。

- 严格schema/AST、字段来源、Manual/AutoRead、显式覆盖和DTO；局部数字槽位/完整名称片段、重复/重叠/版本摘要校验。
- 从实际provider.adapter/精确modelAdapters读取逐模型协议证据；公开参数保持原映射、colon族/case匹配、modelCapabilities及否定保护，不输出认证材料。
- 读取→草稿纯计划、最新readId/全部基准CAS保护、失败/空/晚回、未匹配保留、计数、幂等与显式人工操作。
- 不可变revision追加计划、基准校验、原revision不改、Agent锁字段保护、实际变更记录。
- 运行物化纯计划、失效人工pin可明确清除修复；旧有效值不可解时beforeKnown=false，L0须在实际文件锁内读取旧值，不能以None伪称旧字段不存在。
- ledger生成/配对/重读来源恢复，防止物化值反认来源或人工；clear恢复原底值。finalize要求逐模型11字段目标完整，实际目标位置只能由L0映射器提供，拒绝认证字段路径。
- 协议分组只按最终协议及wireProfile，Chat/Responses独立；认证能力必须来自L0实际允许集，纯计划不分配运行provider身份、不复制凭据。
- portable v6确定性编码、格式/依赖闭包/摘要/子集/族片段校验、原生渠道禁止导出、本机token/凭据引用不便携；同revision不同内容拒绝，已有默认不被导入覆盖。
- 新seed由固定候选SHA256 aeeec2078ff1416f587019c376cc06b17259550e47db199f1e59a7fff8aae68b生成；锁/来源补自固定release-seed-rules.json（原代码0c57500f8的种子投影）。沿用templateId，新revisionId是candidateRevisionId命名空间下UUIDv5('OCXD-023-production-content-v2-r1')。没有自动激活日常库。

## 检查与证据

最终29项测试通过（0失败/跳过），含246行/152不同ID：133族归类、79记录命中已知覆盖、58参数/锁逐项等值、57版本族边界、8名称位置、4参数外溢反例、28已选潜在匹配、19未匹配逐项原配置保留，以及来源/手工/自动/ledger/分组/便携边界。

运行方式是独立Cargo驱动按绝对path直接编译本线真实源码和原schema.rs，未复制模型实现或mock v2函数。驱动仅给原schema.rs的无关AppError转换提供最小占位类型；不验证应用存储错误转换。独立Cargo.lock已保留，不声称等同应用整crate或原锁文件检查。

命令：

~~~sh
cargo test --offline --locked --manifest-path /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/r1-core-driver/Cargo.toml --lib
cargo clippy --offline --locked --manifest-path /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/r1-core-driver/Cargo.toml --lib --tests -- -D warnings
rustfmt --edition 2021 --check <本线5个Rust文件>
git diff --check
~~~

测试、Clippy拒绝警告、格式、diff检查均通过。最后只改了独立29号未匹配测试后重跑全29与Clippy/格式/diff，没有修改测试后的产品源码。没有运行服务、端口、真实发现/推理、日常配置或上游API。

证据均在本文件同目录：r1-core-final-test.log（最终29通过）、r1-core-stage1-test.log（阶段20通过）、r1-core-stage2-test.log（阶段27通过）、r1-core-stage2-final-test.log（阶段28通过）、r1-core-second-test.log（早期18通过2失败，已修复，保留）。首轮4通过16失败因seed生成缺必需lockedFields/sources，早期compile出现Vec类型推断错误；均未伪报通过，原工具输出在聊天保留。后两回放断言错误分别为label误当familyKey、版本等价遗漏系列前缀，修正测试而非放宽引擎。生成脚本曾在Python换行转义阶段未执行即失败，未影响来源原件。

驱动/target与测试日志是受控证据，保留供L0复核；没有临时进程/监听端口，无需运行态清理。中间补丁片段不构成产品源码来源，实际源码以Git提交为准。

## 注册与本批接口增量

L0已注册4模块及v2_tests，无需L1再改mod.rs。新增：schema_v2::save_revision_plan_v2(&LibraryV2,&SaveRevisionRequestV2)->Result<LibraryV2>。

source_v2新增：

~~~rust
fn finalize_materialization_ledger_v2(
    intended: &PolicyV2, written_file: &serde_json::Value,
    targets: &[MaterializedTargetV2],
) -> Result<MaterializationLedgerV2>;
fn reconcile_source_v2(
    channel_id: &str, saved: &SourceModelV2, imported: &SourceModelV2,
    ledger: Option<&MaterializationLedgerV2>,
    expected_source_policy_digest: &str, actual_document: &serde_json::Value,
) -> Result<SourceModelV2>;
fn validate_protocol_group_auth_v2(
    groups: &[ProtocolGroupPlanV2], bindings: &[ProtocolAuthBindingV2],
) -> Result<()>;
~~~

reconcile比首阶段多expected_source_policy_digest参数，验证ledger与可信已保存策略相配。传入的是ledger对应的已保存意图摘要，不能用尚未提交的新草稿摘要。ModelFieldWriteV2新增beforeKnown；LibraryV2加入默认空submissionReceipts，保留既有库回执维度。其余首阶段API保持。

## 下一批最小逐文件写集建议（待L0逐项放行）

为形成实际消费者链，建议本线新增两个文件、扩展两个现有文件，并继续拥有当前v2文件/测试；L0给集成SHA后在本线更新至明确来源，不自行合入其它贡献分支。

| 文件（model_config下） | L1接手内容 | 与L0接点 |
| --- | --- | --- |
| provider_import.rs（现存，新增v2入口） | 受锁原版provider配置→真实模型库存/source/v2状态；精确adapter证据、source/explicit/ledger重读恢复、导入方向/节点/认证证据核对。v1入口先保持兼容，生产读取由L0明确切v2。 | L0传原始Value、可信管理元数据/ledger、稳定渠道UUID/provider绑定及读取基准；原件/秘密只在L0受控输入中，输出脱敏DTO。 |
| compiler_v2.rs（新增） | 实际原版provider JSON物化：写modelAdapters等字段，恢复clear、生成ledger目标，按最终协议分组；根据L0给出的明确运行provider映射及改名结果保持原始ID/选择/未知字段，不另执行名称事务。 | L0 config_file::save门调用；L1返回下一原件Value+Policy/ledger/映射bytes，L0在同事务写入，不另写第二份配置。 |
| protocol_groups.rs（现存，新增v2入口） | 最终协议/wireProfile→稳定分组身份的load/plan/bytes；删除v2路径对channel默认协议依赖，原provider anchor仅依显式持久映射；认证允许集校验。 | 真正分组映射写入与改名事务由L0保存门统一提交；不在preview/mappings_v2中暗写。v1路径待共同切换再退役。 |
| store_v2.rs（新增） | 接实际根参数的库/策略/草稿/ledger读取、v2版本/配对校验、库revision原子保存与读回、draft代次CAS及准备配置保存的受控材料API；复用已有锁/atomic_write/版本保存服务，不扫描旧根、不迁移。 | L0提供解析后的确切文件路径/锁序/写入准入及共同事务接口。涉及原版JSON+Policy+ledger+分组的多对象提交仍由L0拥有，不让store再单独提交半份配置。 |

独立测试继续v2_tests.rs，若文件驱动需要真实temp-root fixture可另申请consumer_v2_tests.rs，由L0注册。schema_v2/matching_v2/source_v2/portable_v2及seed仅为真实接线必要的小幅扩展，不自行碰schema.rs/matching.rs/compiler.rs/store.rs。

L0继续独占：config_file*.rs/config_file_mapping.rs、channel_rename.rs/rename_effects.rs、execution_directory.rs/saved_runtime_reconcile.rs、commands/model_config.rs及公共注册/错误/总线、统一文件保存门。L2继续独占transfer及远端状态机；L3使用同一v2 DTO。下一批首条代表流程应为L0实际读取→L1 v2 source/match→实际编辑→保存门物化/ledger→重开读回，随后验证改名/分组/原始ID/选中保留；不能以新增模块存在代替消费者切换。

本建议尚非独占写权；本轮第二阶段已完成，等待L0下批SHA、路径/事务签名和逐文件放行后继续实际接线。

