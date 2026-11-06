# L1 G0 正式合同候选 r1

合同ID：OCXD-023-MATCH-G0-r1。日期：2026-11-06，Asia/Shanghai。
状态：可供L0冻结；尚非R1编码放行。本文件是L1独立受控产物，不写共享docs/.adg。采用时绑定本文件SHA256；语义变化生成后继文件，不能覆盖已采用字节。

## 1. 来源与已决定事项

- 文档：956e4cb4accbe4e80857f2e49da336e4127202fd中的TEAM、IMP-31、IMP-33和DMD-TEMPLATE-MATCHING。
- 参考代码：0c57500f8f46ea50655c782c3f04334a39167abb；不是尚待派发的R1工作根。
- 候选JSON SHA256：aeeec2078ff1416f587019c376cc06b17259550e47db199f1e59a7fff8aae68b。
- L0最新裁决：实际modelAdapters[rawModelId]优先，其次实际provider.adapter，解析为逐模型来源证据。它们不是新渠道属性，不根据模型名称推断协议。此裁决替代L1上轮对provider.adapter不参与底值的保守建议。
- 来源之上依次为模版通用、条件、明确人工覆盖；保留原始ID、认证、选择、启停及顺序。成功读取自动进入草稿，正常保存才生效。
- 17模版、34包含、58已知覆盖；246行/152ID，保持133族归类与28已选潜在覆盖，19未匹配不补齐。

## 2. 格式与版本

| 对象 | 固定值 | 规则 |
| --- | --- | --- |
| Library/Policy | schemaVersion=2 | 严格字段检查，不降级v1 |
| 新revision内容 | contentVersion=2 | 新UUID与摘要；不修改历史字节 |
| 模型便携包 | format=OCXD_MODEL_PACKAGE, formatVersion=6, schemaVersion=2 | 现有包版本1～5已占用；本版所有选区都用6 |
| 模型交互DTO | dtoVersion=2 | 展示字段不是提交权限 |
| 外层配置文件/保护容器/远端图 | 沿L0/L2单独合同 | 必须声明所含模型格式；不能由模型v6推定其它版本 |

JSON属性camelCase，枚举snake_case，tagged union使用kind/mode；UUID规范小写；SHA256为sha256:加64位小写hex。摘要算法：对label、contentVersion、content构成的对象递归排序键，数组保持顺序，无空白UTF-8 JSON；不含时间、digest自身。传输摘要另对精确字节计算。

TemplateV2保留id/label/source/defaultRevision/revisions。RevisionV2保留id/createdAt/savedLabel/content/digest/baseRevision/changes/origin，并要求contentVersion=2。LibraryV2保留templates及已有保存回执语义。PolicyV2保留channels/templateRefs/defaultChannelId及L0拥有的显式删除/恢复事务字段。所有可执行引用使用下文RefV2。

原v1库、旧revision、candidate-v1不后台转换、不覆盖、不重算摘要；v2读取器遇到这些格式拒绝。历史原件继续保全，不能重解释suffix为新条件，也不新增旧运行兼容器。新发布种子由冻结候选离线生成新revision，沿用templateId，不将candidateRevisionId冒称已保存版本。原版config.json作为明确读取来源不等于迁移旧Desktop库。

ChannelV2.connection保留providerId/nodes/activeNodeId/credentialRef/enabled/allowPrivateNetwork/authMode/sourceReadId/importedConfig，删除protocol。渠道defaults/familyOverrides只允许非协议人工参数；其ModelRules.protocol必须inherit，custom时报错而非静默删除。名称/连接/凭据/删除事务继续由L0拥有。

## 3. 最小Rust数据类型

以下是声明合同，不是本轮产品实现。沿用现有Field<T>/ModelRules/ModelSelection/ServiceNode/ModelNaming/SourceEvidence。serde与JsonSchema必须落实上节线格式，下面省略重复derive。

~~~rust
type Id = String;
type Digest = String;
type RuleKey = String; // 严格11字段白名单
type Result<T> = std::result::Result<T, MatchErrorV2>;

struct RefV2 { template_id: Id, revision_id: Id, digest: Digest }
enum ModelBindingV2 { // tag=kind；无引用用Option::None
    Manual { reference: RefV2 },
    AutoRead { reference: RefV2 },
}
enum ProtocolV2 {
    OpenaiChat, OpenaiResponses, Anthropic, Google, OllamaNative,
}
enum SourceKindV2 { ImportedFile, UpstreamRead, PortableImport, ExternalFileEdit }
struct SourceEvidenceV2 {
    kind: SourceKindV2, source_digest: Digest,
    provider_id: String, pointer: String, // 已知公开字段的JSON Pointer
}
struct SourceFieldV2 { value: serde_json::Value, evidence: SourceEvidenceV2 }
struct SourceModelV2 {
    raw_model_id: String,
    fields: std::collections::BTreeMap<RuleKey, SourceFieldV2>,
}
struct ModelConfigV2 {
    raw_model_id: String,
    label: String, observed: bool,
    source: SourceModelV2,
    explicit_overrides: ModelRules,
    binding: Option<ModelBindingV2>,
}
struct ChannelModelStateV2 {
    channel_id: Id, models: Vec<ModelConfigV2>,
    defaults: ModelRules, // 仅非协议人工渠道覆盖
    family_overrides: std::collections::BTreeMap<Id, ModelRules>,
}
enum FieldOriginV2 { // tag=kind
    Source { evidence: SourceEvidenceV2 },
    TemplateCommon { reference: RefV2 },
    TemplateCondition { reference: RefV2, condition_id: Id },
    ChannelDefault,
    ChannelFamily { template_id: Id },
    ExplicitModel,
    Unknown,
}
enum ProtocolStateV2 { // tag=kind
    Resolved { value: ProtocolV2, origin: FieldOriginV2 },
    Missing,
    Unsupported { value: String, origin: FieldOriginV2 },
}
struct EffectiveModelV2 {
    channel_id: Id, raw_model_id: String,
    selected: bool, enabled: bool, observed: bool,
    binding: Option<ModelBindingV2>, rules: ModelRules,
    protocol: ProtocolStateV2,
    field_origins: std::collections::BTreeMap<RuleKey, FieldOriginV2>,
    matched_condition_id: Option<Id>, notices: Vec<MatchIssueV2>,
}
~~~

RuleKey白名单：protocol/contextWindow/inputModalities/reasoningEfforts/defaultEffort/fast/pinnedEffort/effortMap/noReasoning/reasoningAuthoritative/fastWire。每个SourceField.value按对应字段类型验证，不能装整个provider/Key/认证头。缺来源字段用map中不存在表达；fastWire的显式null仍按Field<Option<FastWire>>保留。原始未知字段由L0保留在原件，不塞进规则来源对象。

ProtocolV2的wire值固定为openai-chat/openai-responses/anthropic/google/ollama-native，不能由Rust枚举名自动产生下划线值。来源中的未知字符串保留证据，只有它成为最终协议时报告unsupported；模版和人工提交只接受支持值。扩展协议须新合同及实际adapter检查，无隐式openai默认。

ChannelV2拥有原有身份、连接、选择、顺序、命名、人工非协议渠道覆盖和models；同一模型ID唯一。templateRefs继续表示人工选定的模版候选版本，模型binding额外区分Manual/AutoRead。computed rules/fieldOrigins不进入持久人工覆盖字段。

ChannelV2的模型库存还须逐项保留原观察标记和label：原discovered.rules转换为source事实后不再作为第二套有效配置权威。EffectiveModelV2及L3行DTO另外返回selected/enabled/observed三个只读布尔值，分别由原ModelSelection、connection.enabled及真实观察标记派生，读取匹配不得改写它们。未发现但明确手选的ID继续保留，不要求先发现才能保存。

## 4. 正式条件AST

TemplateContentV2字段：description/matching/nameLayouts/commonRules/conditionOverrides/lockedFields/sources。matching保留channelMode/channels/caseSensitive/priority/include/exclude，另有rejectRepeatedLiteralPrefix=true，落实候选解释器的重复族前缀拒绝；只检查已声明prefix规则p的p+p开头，不纠错任意ID。

~~~rust
enum PatternKindV2 { Exact, Prefix, Glob, AsciiNumericPrefix }
struct PatternV2 { kind: PatternKindV2, value: String }
struct MatchRulesV2 {
    channel_mode: ChannelMode, channels: std::collections::BTreeSet<Id>,
    case_sensitive: bool, priority: i32,
    include: Vec<PatternV2>, exclude: Vec<PatternV2>,
    reject_repeated_literal_prefix: bool,
}
struct NameLayoutV2 {
    id: String, literal_prefix: String,
    version_capture: String, // complete_numeric_components
    numeric_joiners: Vec<String>, // 仅[".", "-"]
    tail_capture: String, // literal_remainder
    trailing_date_policy: String, // retain_valid_YYYYMMDD_after_hyphen
    ambiguous_numeric_tail: String, // no_equivalence
}
enum LiteralPositionV2 { Any, BeforeVersion, AfterVersion }
enum SelectorAtomV2 { // tag=kind
    ExactId { value: String },
    NumericVersion { components: Vec<String> },
    LiteralSegment { value: String, position: LiteralPositionV2, boundary: String },
}
struct SelectorV2 {
    all: Vec<SelectorAtomV2>,
    layout_id: Option<String>, literal_tail: Option<String>,
}
struct ConditionOverrideV2 {
    id: Id, name: String, selector: SelectorV2, rules: ModelRules,
    locked_fields: std::collections::BTreeSet<String>,
    source_exact_id: String, scope: String, // known_identity_only
}
struct TemplateContentV2 {
    description: String, matching: MatchRulesV2,
    name_layouts: Vec<NameLayoutV2>, common_rules: ModelRules,
    condition_overrides: Vec<ConditionOverrideV2>,
    locked_fields: std::collections::BTreeSet<String>, sources: Vec<SourceEvidence>,
}
~~~

1. 渠道范围使用稳定UUID。namespace候选只有明确采用且绑定UUID后才生效；最长前缀至多去一次，只变匹配视图。不能按渠道显示名自动装给所有用户。
2. include为OR，exclude优先；glob只解释字面*和?，不执行regex。ascii_numeric_prefix要求前缀后紧跟ASCII数字。caseSensitive属于revision，新候选为false，原wire ID从不折叠改写。
3. all是非空AND。layoutId/literalTail成对；有layout恰有一个numeric_version。无layout不能使用数字版本或before/after位置；boundary唯一允许hyphen_or_string_edge。
4. prefix后的连续ASCII数字按.或-分段，components保持字符串、前导零及完整段数。至少3段、末分隔为-且末段8位有效公历YYYYMMDD时，将该段连同-留在tail。tail以.起始则布局失败；至少3段、含-且末段4/6位的歧义数字尾不做等价。与冻结候选解释器一致。
5. literalTail整体相等，规格和构建不丢；literal_segment匹配完整连字符边界片段。before/after相对声明数字槽位，any允许两侧。gpt-6-sol参数不流入gpt-6-sol-mini、日期别名或未来版本。
6. 模版按matching.priority取唯一最高者；最高同级多个命中报matching_conflict。条件没有额外priority：同模版同时命中多个条件报condition_overlap，即使赋值相同/字段不相交也不合并。
7. 保存时拒绝等价重复selector。AND原子先排序去重；布局身份键为折叠后的literalPrefix、完整components及literalTail，同身份键重复即拒绝，不能通过添加重复名称条件绕过。纯exact_id按折叠后的完整ID集合形成身份键；另对完整AST做规范签名。静态可证明重叠时直接拒绝；一般重叠在实际命中时拒绝，不将样本通过说成全域互斥证明。
8. 58组参数及锁字段保留；lockedFields是模版编辑/Agent的保护字段，不是逐模型人工标记。未知型号只继承common；19项不补齐、不禁用、不清空。

## 5. 来源与有效配置解析

L0从受锁实际原件提取协议来源：精确modelAdapters[rawModelId]存在且结构有效则用它，否则用实际provider.adapter。精确项存在但值不受支持时不能退provider.adapter。证据包含sourceDigest、providerId和正确转义的JSON Pointer。通用adapter被展开为逐模型来源事实；ChannelConnectionV2/PortableChannelV2没有protocol。

其它来源参数保持原版字段映射/实际选择器决胜，包括provider级/per-model值及fastEnabled=false等已有否定约束；wildcard规则只有按原版真实决胜才可产生来源，并记录真实pointer。协议最小合同仅为精确modelAdapters/实际adapter，其它协议选择器须L0核验实际语义后再扩展。来源类型错误是错误，不当缺字段兜底。

逐字段优先级从低到高：source → commonRules → 唯一condition → 非协议人工channel.defaults → 非协议人工familyOverrides → explicitOverrides。后三层必须是明确编辑或已验证v2人工元数据，不能由原JSON有值推定。protocol只走source/common/condition/explicitModel，渠道protocol custom拒绝。

false、空集合、fastWire=null均为显式custom。Manual引用只解析其精确revision，不参加其它模版竞争；失效时保留引用并报错，不换默认。AutoRead只在下一次明确成功读取时重新选择，普通snapshot/project/save不重选默认版本。

## 6. 读取草稿与L3 DTO

~~~rust
struct ReadBaseV2 {
    dto_version: u32, draft_id: Id, draft_generation: u64,
    source_digest: Digest, policy_digest: Digest, library_digest: Digest,
    root_revision: String, // L0不含路径的根身份
}
struct ReadChannelObservationV2 {
    read_id: Id, base: ReadBaseV2, channel_id: Id, models: Vec<SourceModelV2>,
}
enum MatchOutcomeV2 {
    Matched, ManualPreserved, Unmatched, MissingDefault, MissingDependency, Conflict,
}
struct ReadModelDtoV2 {
    raw_model_id: String, outcome: MatchOutcomeV2, changed: bool,
    binding_before: Option<ModelBindingV2>, binding_after: Option<ModelBindingV2>,
    effective: EffectiveModelV2, issues: Vec<MatchIssueV2>,
}
struct MatchCountsV2 {
    total: u32, matched: u32, manual_preserved: u32, unmatched: u32,
    missing_default: u32, missing_dependency: u32, conflict: u32,
    changed: u32, protocol_pending: u32,
}
struct ReadMatchDtoV2 {
    dto_version: u32, read_id: Id, base: ReadBaseV2,
    next_draft_generation: u64, changed: bool,
    models: Vec<ReadModelDtoV2>, counts: MatchCountsV2,
}
~~~

六种outcome互斥且计数和为total；changed/protocolPending是独立计数。匹配成功可以同时协议待配置。DTO不含Key/认证字段/任意provider原件。L3直接使用第4节条件AST、conditionId、规范字段路径和稳定原因码，不写匹配引擎。

读取顺序：

1. L0捕获source/policy/library/root/draft代次与最新readId，取得完整一致的实际观察；连接/认证/ID/选择等沿L0原观察对象保全。
2. 失败、认证错误、无接口、全空结果不应用，不清空草稿。成功观察才进入匹配。
3. 每个templateId按渠道明确保存引用→全局明确保存引用→明确defaultRevision选择；不取未保存编辑、候选或任意latest。明确引用缺失报依赖错误，不退默认。
4. 保留Manual绑定及人工explicitOverrides；成功自动匹配者写AutoRead精确Ref。未匹配/无默认/冲突/依赖缺失保留该模型原绑定和原有效配置，观察材料单独保全，不先覆盖该模型原配置。无关成功项可以形成草稿。
5. L0返回前CAS检查readId仍最新且draftGeneration/source/policy/library/root都未变；任一变化拒绝整次迟到应用。UI同样检查readId/代次，不能仅靠禁用按钮。
6. 相同来源值/引用/人工意图重复读取不增加语义差异。readId、观察时间、来源证据编号不参与changed；无变化不加草稿代次，真实变化只CAS增加一次。
7. 保存预览包含引用变化和真实有效参数差异；取消不提交。保存模版本身或更换默认版本不后台重配运行配置。

ReadMatchDtoV2是响应，不是可写Policy。L0持有服务端草稿，UI发送明确编辑操作；source/effective/fieldOrigins/auto binding不能由客户端提交获得写权。

## 7. 保存：区分source、人工意图与运行物化

~~~rust
enum ModelEditV2 { // tag=kind；只允许真实明确编辑
    SetOverride { channel_id: Id, raw_model_id: String, field: RuleKey, value: serde_json::Value },
    ClearOverride { channel_id: Id, raw_model_id: String, field: RuleKey },
    SetManualBinding { channel_id: Id, raw_model_id: String, reference: RefV2 },
    ClearManualBinding { channel_id: Id, raw_model_id: String },
}
struct SaveModelsRequestV2 {
    dto_version: u32, expected: ReadBaseV2, edits: Vec<ModelEditV2>,
    scope_channel_id: Option<Id>,
}
struct MaterializedFieldV2 {
    channel_id: Id, raw_model_id: String, field: RuleKey,
    target_pointer: String, value_digest: Digest, present: bool,
}
struct MaterializationLedgerV2 {
    source_policy_digest: Digest, written_file_digest: Digest,
    fields: Vec<MaterializedFieldV2>,
}
~~~

- SetOverride是逐模型字段进入人工custom的唯一入口；“有效值相同但设为自定义”仍属意图变更。ClearOverride恢复inherit，不复制下层值。手选模版写Manual，清除后置空，下一次明确读取才产生AutoRead。
- 服务端PolicyV2分开保存source、explicitOverrides、binding、非协议人工渠道覆盖。UI不可写source或AutoRead；正式人工Agent建议沿审核入口转成同类编辑操作。
- L0基于可信草稿与明确edits调用同一解析器，生成运行物化计划，写原版modelAdapters/modelContextWindows等。原JSON中的物化值只是运行表示，不反向成为人工元数据。禁止从保存diff推断“人工改过”。
- 原版文件、PolicyV2/source层、MaterializationLedgerV2和分组映射必须同一L0事务保护；锁/日志/恢复保证读者只看到配对已提交状态。失败保全原件，不能生成假的人工标记。
- 重读时，原字段若等于ledger最后物化值/缺失状态，恢复保存的原source层，不能把模版输出再导入为source。全文件摘要变化后逐受管字段比较；不相关变化不使已知来源失效。
- 字段被实际外改时，记录ExternalFileEdit来源及旧证据，依然不叫人工override；下次明确读取允许模版覆盖。ledger缺失/损坏、受管输出与元数据不能配对时，报source_provenance_unknown，保留原件，停止受影响模型自动重判/覆盖。首次独立原件导入可建ImportedFile；已知受管元数据损坏不能伪装首次导入。
- 取消模版/人工覆盖后按原source层恢复，原来不存在的字段恢复不存在；不能把模版物化值留作新底值。未知及不相关原件字段不删。
- 提交锁内再次核对全部expected和精确依赖；scope保存保留其它渠道；保存不能临时切换默认revision。
- 已选且启用的受影响模型最终协议missing/unsupported或认证不兼容，阻断本次运行应用并保留草稿，不自动禁用/取消选择。未选待配置模型可保留；但不能为生成必填adapter捏造协议。新渠道无任何可解析协议时只保留草稿，不创建可运行provider。

## 8. protocol_groups不再渠道回退

bucket只读取EffectiveModelV2.protocol：Resolved给精确协议；Missing/Unsupported报可定位问题。不读connection.protocol，不把Chat/Responses并到渠道协议，不按渠道名称或首模型猜anchor。

分组键固定为(channelUUID, finalProtocol, wireProfileDigest)。wireProfile包含已有reasoningAuthoritative/fastWire等实际需provider级隔离的有效值，不包含URL、名称、排序或选择。相同键复用持久运行身份；多个协议形成多个运行provider，adapter取本组finalProtocol。

逻辑channel↔运行provider映射由L0持有并同保存事务提交。原providerId要保留为anchor时必须有明确持久anchorProtocol记录；不能从channel默认/数组顺序推导。无法安全建立anchor/新组则拒绝应用，不覆盖已有provider。分组前后rawModelId不增删前缀。

专用认证/OAuth/本机无认证方向限制继续验证，不能将Key盲复制到不兼容adapter。预览只生成分组计划，不创建映射文件；凭据引用沿同一逻辑渠道受控解析。

## 9. 最小函数与L0接线

~~~rust
// L1纯逻辑，无文件/网络/CLI/草稿持久写入
fn validate_content_v2(content: &TemplateContentV2) -> Result<()>;
fn resolve_effective_model_v2(
    library: &LibraryV2, channel: &ChannelV2, model: &ModelConfigV2
) -> Result<EffectiveModelV2>;
fn match_read_to_draft_v2(
    library: &LibraryV2, draft: &PolicyV2, observation: &ReadChannelObservationV2
) -> Result<ReadMatchPlanV2>;
fn apply_explicit_edits_v2(draft: &PolicyV2, edits: &[ModelEditV2]) -> Result<PolicyV2>;
fn plan_materialization_v2(
    library: &LibraryV2, before: &PolicyV2, intended: &PolicyV2
) -> Result<ModelMaterializationPlanV2>;
fn plan_protocol_groups_v2(rows: &[EffectiveModelV2]) -> Result<Vec<ProtocolGroupPlanV2>>;
fn export_portable_models_v2(
    library: &LibraryV2, saved_policy: &PolicyV2, scope: &ModelExportScopeV2
) -> Result<PortableModelsV2>;
fn encode_portable_models_v2(value: &PortableModelsV2) -> Result<Vec<u8>>;
fn decode_portable_models_v2(bytes: &[u8]) -> Result<PortableModelsV2>;
fn validate_portable_models_v2(value: &PortableModelsV2) -> Result<()>;
~~~

ReadMatchPlanV2内部字段为expected:ReadBaseV2/nextPolicy:PolicyV2/dto:ReadMatchDtoV2。ModelMaterializationPlanV2含逐模型有效值、精确依赖、分组计划、预期意图摘要，不含任意可执行JSON路径。ProtocolGroupPlanV2含channelId/protocol/wireProfileDigest/rawModelIds；真实provider ID由L0映射服务分配。

L0逐项接线清单：

1. config_file_mapping.rs::read_policy：defaults_from_file/overrides_from_file输出进入source，人工层只来自v2管理元数据；删ChannelConnection.protocol赋值，建逐模型协议证据，保留原版实际规则决胜/否定保护。
2. config_file.rs::snapshot/snapshot_value：返回分层已保存状态和同源投影，不触发默认重选；显式读取动作另接match_read_to_draft并CAS应用。
3. commands/model_config.rs及channel_actions.rs：读取身份绑定全部基准，v2 DTO脱敏；保存用服务端draft身份+edits及原独立名称/连接/凭据意图，拒绝客户端source/effective/auto binding。
4. config_file_mapping.rs::write_inner：只消费L1物化计划，不从connection.protocol写adapter，不把effective反填explicit。保留未知字段/认证/原ID/选择/无关对象；clear恢复source。
5. config_file.rs::seed_imports：原件公开参数进入source，仅已验证v2人工意图进入explicit；provider_import.rs由L1调整为检查实际adapter证据+节点/认证，删除渠道协议比较。
6. config_file保存日志、metadata_digest、record_layout：JSON/Policy/source/ledger/protocol mappings同事务保护与恢复，增加library/metadata/root/draft并发校验。
7. execution_directory.rs/saved_runtime_reconcile.rs/名称事务：复用逻辑渠道↔分组映射，不从adapter回造渠道协议、不改rawModelId。
8. discovery_paths.rs/measurements/前端节点消费者：临时DiscoveryKind与请求ProtocolV2解耦，凭据同源/方向验证保留；发现结果不写渠道协议。DiscoveryKind默认未选，不便携持久化。
9. 公共命令/模块注册、全局错误和tests.rs由L0单写；上述跨写集文件由L0接入或逐文件转交。本文件不自行扩大L1写集。

## 10. L2便携编码边界

PortableModelsV2包含v2 templates、所选模型policy片段、source公开值/证据摘要、explicitOverrides、Manual/AutoRead精确引用与依赖闭包。PortableChannelV2无protocol；连接/凭据选择沿已有保护外壳。公开provider材料中的实际adapter只作来源材料，不能覆盖逐模型分层。

模型子集只携所选raw IDs及依赖；族片段无连接/凭据。普通同步仅已保存对象；未保存草稿只允许明确草稿导出动作，不能混入普通上传。每个Ref须同包或本地可验证的精确revision/digest；同ID不同内容拒绝、不覆盖旧revision。

接收保留Manual/AutoRead种类，重新绑定明确channel mapping。来源仍为source，标PortableImport并保留原证据链摘要，不变人工。namespace规则未明确采用则不激活。原本机路径/readId/draftGeneration/credentialRef/运行provider映射/ledger不便携；ledger由接收端真实保存生成。

L1拥有语义类型/确定性编码/校验，L2拥有外壳和传输。transfer.rs旧PortableChannel属于L2；新模型语义类型从L1批准schema/portable_channels文件引用，双方不各造一份。包formatVersion=6一律调用v2校验，不走旧1～5降级分支。

L2的保护加密生成一次后冻结精确字节；重试不重编码/重新加密。模型规则/依赖冲突与远端提交冲突分开，传输stored不代表模型可应用。外层预算不得因模型包版本变大而提高。

## 11. 错误、预算与检查

~~~rust
struct MatchIssueV2 {
    code: String, channel_id: Option<Id>, raw_model_id: Option<String>,
    field_path: Option<String>, condition_ids: Vec<Id>, reference: Option<RefV2>,
    message: String, recovery_actions: Vec<String>,
}
struct MatchErrorV2 { code: String, issues: Vec<MatchIssueV2> }
~~~

| 稳定code | 处理粒度 | recoveryActions |
| --- | --- | --- |
| model_schema_unsupported / model_unknown_field | 格式/请求拒绝，不丢字段 | upgrade_reader, inspect_source |
| model_source_invalid | 读取不应用 | inspect_source, retry_read |
| model_source_provenance_unknown | 受影响模型保留，不伪造层次 | inspect_source, reestablish_source |
| model_matching_conflict / model_condition_overlap | 模型级保留，相关保存应用拒绝 | edit_matching, choose_manual_template |
| model_selector_duplicate / model_selector_invalid | 模版保存拒绝 | edit_conditions |
| model_template_default_missing | 模型级保留 | set_saved_default, choose_manual_template |
| model_dependency_missing / model_revision_digest_mismatch | 读取该项保留；导入应用拒绝 | restore_exact_revision, choose_manual_template |
| model_protocol_missing / model_protocol_unsupported | 草稿可见，受影响运行应用拒绝 | configure_model_protocol, edit_template |
| model_channel_protocol_forbidden | 请求拒绝 | remove_channel_protocol |
| model_protocol_auth_incompatible / model_runtime_mapping_conflict | 保存应用拒绝，原件保全 | review_auth_binding, inspect_runtime_mapping |
| model_read_empty / model_read_unavailable | 本次读取不应用 | inspect_source, retry_read |
| model_read_stale / model_save_stale | 整次应用拒绝，保留新草稿 | refresh_and_retry |
| model_limit | 超限拒绝，不截断 | reduce_scope |

unmatched正常状态不报错误，没有补齐19项的action。message脱敏，不放Key/原件/绝对用户路径。L3按code/action枚举处理，不解析message判断行为；具体按钮动作只在对应能力实际可用时启用。

预算：rawModelId≤2048 UTF-8字节，include+exclude≤256，单pattern≤1024字节，layouts≤256，conditions≤256，单selector atoms≤32，description≤32768字节，sources≤128。便携文件/解包文档沿L2统一64MiB/32MiB，不提高。超限在分配/遍历前拒绝，无regex执行。

R1后验证项：冻结246/152回放、133/28覆盖、58参数与锁等值；57版本族边界、8名称位置、4外溢反例；重复/重叠/未知格式拒绝；modelAdapters优先/provider.adapter来源证据；Manual/AutoRead/explicit独立；同值设人工与clear恢复；失败/空/晚回/重复幂等；save→read→clear不残留物化值；外改/ledger损坏/事务中断保全；Chat/Responses真实分组及认证保护；模型子集/依赖便携往返；L0/L3实际API、组件及保存读回。

以上检查本轮未执行。冻结需L0采用版本、source/explicit/ledger、协议分组事务及L2/L3字段，公布本文件摘要、共同代码基线与逐文件独占写集后才放行R1。本轮无产品编码、上游调用、日常配置变更或发版。

