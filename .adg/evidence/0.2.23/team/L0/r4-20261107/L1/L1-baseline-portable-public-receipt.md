# L1 草稿观察基准与便携公开材料回交

2026-11-06，Asia/Shanghai。工作根/分支沿L1：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes，codex/0.2.23-model-matching。两个独立授权批次分别提交，不修改L0 gate、命令、L2 wrapper、transfer.rs或共享docs/.adg。

## 草稿观察基准

提交e387d2dcd0786d452f8acfbbe66845f68dd790dc，父4477dbc8bd40abc08a3d79ea532c0f99da76b1ec。仅store_v2.rs/consumer_v2_tests.rs。

DraftV2新增 #[serde(default)] pub observed_baseline:Option<PolicyV2>，wire为observedBaseline。只在草稿中持久，不进入便携包或发布Policy/ledger。所有L0生产写入须Some(当时真实、脱敏的分层实际观察)；缺字段/None仅限本版未发布schema2 fixture，不是v1兼容/迁移。

Store验证draft与baseline各自schema2、字段类型、唯一渠道/模型ID、运行provider身份、rawModelId/source匹配等。额外拒绝baseline同providerId在draft重绑为另一个管理UUID；允许合法人工连接/选择/order/source刷新差异、改名和draft-only新增对象，不把差异强制相等。CAS接口不变；整份Draft Eq包含baseline，baseline-only更改也必须generation递增，旧base不可重放覆盖。

正常应用crate快照18消费者测试通过（旧15+新增3），0失败/忽略。新增验证临时根往返保留独立baseline、无原Key/Policy或ledger半写；schema/source错配/ID重绑/未知apiKey字段拒绝；baseline-only CAS。最初新增测试因同一draft借用base同时move出现编译错误，已在测试clone修正，失败日志保留，不放宽产品校验。Clippy拒绝警告、格式/diff通过。

L0负责实际三方合并：actual与old baseline对比更新source；只保留draft相对baseline真实人工差异，初次import无saved metadata也有观察基准。此小批不自行实现门的合并策略或把wholeDraft恢复当作修复。

## 便携公开provider材料

提交2af475027cd91c5864b62882ca723f4a2641c45f，父e387d2dcd。仅portable_v2.rs/v2_tests.rs。

PortableConnectionV2新增：

~~~rust
#[serde(default, skip_serializing_if="Option::is_none")]
pub provider: Option<serde_json::Value>;

pub fn attach_public_sources_v2(
    package: &PortableModelsV2,
    sources: &BTreeMap<String, serde_json::Value>, // key=channelId
) -> Result<PortableModelsV2>;
~~~

普通export默认provider=None，不输出额外null或读取真实config。attach克隆原包，只向明确包内且有connection的非family渠道注入；所有源调用现provider_fields::validate_public，未知顶层字段/私有字段/嵌套secret拒绝，不静默过滤未知PUBLIC。baseUrl若存在必须属于该connection实际节点；输出重算payload digest并重新validate，createdAt不变。public.adapter只是来源材料，不替代ModelConfig.source/人工覆盖/精确引用的最终协议。

复用现transfer::restrict_models的纯字段裁剪函数，旧DTO桥仅传字段范围，不调用旧匹配器/运行解析、不改transfer.rs。模型子集按明确raw IDs收窄model* Map、models/selected/retain与已知*Models列表、defaultModel及模型别名等；保留被选modelCapabilities中的完整附加子字段。接收校验要求已附public再次裁剪不变，夹带未选模型定义/列表会报model_public_source_scope。family片段无connection/provider，不能通过附件绕开。

固定类型来源：只读 /private/tmp/post021-remaining-20261105/packages/opencodex-2.97.0.tgz，SHA256 3859a64198a05b0ebdbac0593b0cc6b675ec14f32b69ee100cda7a173404e552，package/src/types/provider.ts 及package/src/providers/model-presets.ts：

- autoReviewModelOverrides为按原模型ID键的Record<string,string>，value是review catalog selector。只按key裁，value（含外provider/fullslug/null清除值）保持，不当scope ID重写；支持选中slash ID的已知dash拼写，原实际列表若有同名dash原ID则精确身份优先，避免误携别行override。provider-wide autoReviewModel独立原样保留。
- liveModels是布尔发现开关，subset明确设false，不误当列表；完整channel不改。
- modelPreset是provider marker而非modelID map。隔离旧model*对象通用裁剪后保留既有marker元数据，subset mode=custom，行为对应原包markModelPresetDiverged的spread marker再置custom；不造appliedAt或删除原版本。完整channel不改marker。

正常应用crate快照37核心测试通过（旧31+新增6），0失败/忽略；Clippy --lib --tests --no-deps -- -D warnings、rustfmt、diff通过。新增验证可选附件/完整字段往返、hash变化和重复attach幂等、responsesPath/statelessResponses/嵌套扩展保全、所有已知model Map及否定列表裁剪、review selector值保留、接收越界拒绝、秘密/未知字段/family连接拒绝、public.adapter不抢人工协议。纯字段范围测试使用不透明公开JSON值，只证明裁剪/保全，不宣称每个vendor runtime参数已运行验证。

## 交接与限制

两个SHA均已直接回报L0；工作树干净，本地未push/发布。正常crate快照使用原Cargo.lock、实际源码与IO实现，额外仅本线模块注册声明；未导入L0未提交门或L2 wrapper。受测本批4个源/测试文件与Git提交逐字节核对。无日常配置/真实网络/上游，测试临时根自动清理，无服务/监听端口。

证据同目录：observed-baseline-consumer-tests.log（首次测试借用编译错误）、observed-baseline-consumer-final.log（18通过）、observed-baseline-clippy.log、portable-public-final-tests.log（37通过）、portable-public-clippy.log。

L0实际同步capture仍须在原件一致性保护下抽取已授权公共字段并调用attach，再按附public的新hash冻结；普通saved_state可保留None，不强迫每次保存导出原件字段。未知原config字段继续由原保存门保全，不能因为便携白名单拒绝就删除。L2负责接收overlay保全未选对象/public，authValidator使用真实source public方向；本批不修改wrapper或宣称实际跨端传输/应用完成。

