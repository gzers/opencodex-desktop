# 0.2.23 模型模版通用匹配候选配置

2026-11-06。用户要求提前写好模型模版配置，并明确纳入0.2.23。配置数据已准备，**当前生产读取器不支持新格式，不能直接覆盖日常model-family-templates.json或seed-catalog.json**。原型及软件接线尚未开发；本文件不是已经生效的配置或发布资产。

## 文件与权威

- [候选JSON](model-templates.0.2.23.candidate.json)：format=ocxd-model-template-candidate-v1，targetVersion=0.2.23，stage=prepared_not_activated。文件自带生产读取支持待实施、来源提交/摘要及读取动作自动匹配到草稿/不自动持久写入的约束。
- 本JSON是本批候选配置的唯一可编辑数据源；前轮分析seed快照仅为固定证据，不是另一套活动配置。
- 原始种子来源0c57500f8f46ea50655c782c3f04334a39167abb；见[来源](../../../../../.adg/evidence/discussions/model-template-matching-20261106/source-identity.json)。17个模版沿用原templateId，candidateRevisionId为新候选身份，不覆盖原不可变revision。
- 范围/验收见[DMD-TEMPLATE-MATCHING](../../../../01-需求管理/需求/DMD-OPENCODEX-TEMPLATE-MATCHING.md)，顺序见[IMP-33](../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-33.md)，经IMP-31 P23-11进入整版验收。

## 已写入的配置

| 内容 | 数量/规则 |
| --- | --- |
| 模版 | 17个现有内置模型族，未扩充用户同意不处理的其它族。 |
| 族包含 | 34条通用规则及必要精确例外，替代原78条精确枚举。Qwen等使用ASCII数字边界，保留现有厂商路径/名称精确例外。 |
| 已知参数 | 58组原型号覆盖全部保留，参数和锁定字段与固定来源逐项一致；不新增未经验证的能力值。 |
| 条件覆盖 | 47组可表达为声明布局+完整数字版本+名称片段+精确剩余规格；11组不适用数字布局的保留精确ID条件。 |
| 符号等价 | 仅声明的数字版本槽位内.和-等价；字面片段可在版本前后，条件为AND，精确保留尾部规格/构建，未知布局不猜。 |
| 大小写 | 新候选匹配视图明确不区分大小写，原始ID保留；这是新revision行为，不改旧caseSensitive规则。 |
| 渠道前缀 | GZGG-NewApi下显式前缀映射候选；实际采用前绑定当时真实channel UUID并审阅，不把本机渠道名自动装入所有用户配置。 |
| 未知/冲突 | 未知型号只继承族通用值，等价重复及同优先级冲突报错；19项未匹配保留原配置，不补齐或纠错。 |

名称条件的position=any允许前后位置；已知型号参数仍通过声明布局和literalTail限制具体身份，避免把gpt-6-sol参数无差别应用到gpt-6-sol-mini或未来版本。字面后缀只是可选方式，不代表所有文字条件。

## 校验结果与使用边界

实际读取JSON的离线解释器回放246条记录/152个不同ID，族归类保持133个；58组来源参数均可由其原ID唯一选中，79条本地记录命中已知型号覆盖。57项版本/族边界、8项名称位置、6项坏配置拒绝、4项参数不外溢反例通过。已知型号参数来自已有种子，未据此宣称已获得新的供应商能力认证。

[校验回执](../../../../../.adg/evidence/discussions/model-template-matching-20261106/candidate-config-validation.json)与[逐模型结果](../../../../../.adg/evidence/discussions/model-template-matching-20261106/candidate-config-model-results.json)保留。校验方法是严格字段/来源等值检查和数据驱动候选解释器，不是当前产品JSON Schema或Rust读取器通过。

后续先冻结正式Schema和同一总原型的条件覆盖/预览，再在代码分支接正式读取、保存、实际解析及导入导出/同步/Agent编辑等消费者，生成新的发布模版数据。候选文件本身不激活当前软件；未来新版读取现有渠道成功时自动匹配到草稿，正常保存后才更新用户配置，不要求额外采用确认。当前不修改用户库、不更新产品版本或发布。复现构建脚本仅用于从固定基线重建初稿，今后编辑候选后需重新验证，不以脚本重复执行覆盖人工修改。

## 渠道协议移除补充（0.2.23目标）

用户已取消渠道默认/自定义协议。候选semantics已记录channelProtocolProperty=false及协议来源顺序：已采用模版通用→命中条件覆盖→单模型显式值；无渠道回退，列表发现适配独立，原版runtime provider adapter由最终模型协议派生。此项不会改变族识别结果，不是实际发现/请求链已经验证。

当前17模版通用protocol仍为来源中的inherit，58组参数也按原来源保留，不能在删除渠道默认后宣称所有模型都已获得可执行协议。正式采用前由MATCH-01核实当前受支持协议资料和实际导入模型的明确来源，或由用户逐模型自定义；缺失明确提示，不凭族名自动填协议。配置仍是待接线候选，不可直接覆盖用户库。

读取入口最新决定：semantics.autoAdoptTemplates=true，触发仅successful_read_existing_channels，目标unsaved_draft，automaticPersistentWrite=false；这是未来行为合同，prepared_not_activated仍保证本文件不执行真实配置变更。按明确引用或指定默认保存版本，不取无默认最新候选；单模型明确人工值保留，普通来源参数不当手工覆盖，冲突/未匹配保留原配置。
