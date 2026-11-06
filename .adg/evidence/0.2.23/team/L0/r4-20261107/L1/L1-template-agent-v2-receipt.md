# L1 模版操作与Agent v2纯提案回交

2026-11-06，Asia/Shanghai。仅完成本批授权纯逻辑，不写库文件、不调用模型/网络/CLI、不改公共mod/commands/UI或现核心实现。

代码根 /Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes，分支codex/0.2.23-model-matching。

- a68371c306c784bb3890e98d9abc133e8003ebc8（父2af475027）：只新增template_ops_v2.rs、template_ops_v2_tests.rs。
- f76bcbc1b7c8fd763020314e22e9a10da37cd102（父a68371c30）：只新增agent_proposal_v2.rs、agent_proposal_v2_tests.rs。

工作树干净，两笔依次可cherry-pick。L0分别注册pub mod template_ops_v2及agent_proposal_v2；测试由各主文件cfg(path)注册，本线未修改mod.rs。

## 精确模版版本操作

纯函数（schema_v2::Result）：

~~~rust
set_default(&LibraryV2, expected_library:&str, reference:&RefV2)->LibraryV2;
clear_default(&LibraryV2, expected_library:&str, reference:&RefV2)->LibraryV2;
usage(&LibraryV2, saved:&PolicyV2, draft:Option<&PolicyV2>,
      template_id:&str, revision_id:&str, additional_exact_refs:&[RefV2])->UsageReportV2;
delete_revisions(&LibraryV2, expected_library:&str, template_id:&str,
      revision_ids:&[String], saved:&PolicyV2, draft:Option<&PolicyV2>,
      additional_exact_refs:&[RefV2])->LibraryV2;
~~~

set/clear验证canonical库摘要和精确已存RefV2摘要。clear只能清当前所选精确默认或无默认的幂等状态，不能用另一个highlight版本悄悄取消当前default。原revision字节不改，不自动latest、不触发后台模型重配。

UsageReportV2含dtoVersion/reference/libraryDigest/savedPolicyDigest/draftPolicyDigest/isDefault/builtinInitial/references/models。引用state区分saved/draft/additional/library，kind区分global_reference/channel_reference/manual/auto_read/additional_exact/base_revision；每项定位channelId/rawModelId/dependentRevision。模型行分开命名matched、matchedConditionId、bindingKind、usingQueriedRevision和匹配issues；名称命中不是采用或能力/运行证明。历史/事务依赖只由L0传精确additional refs，不扫描目录/猜历史。

delete整批先验证：空/重复/缺失选择或旧expected拒绝；固定seed初始revision（不依赖客户端可伪造origin）保护；默认必须先clear或切默认；saved/draft精确引用、Manual/AutoRead及额外明确refs均保护；仍存活子revision的baseRevision父引用也保护。若父子都在同批可删除，最后一个用户revision删后移除空模板，剩余版本/default不自动改写。任一项失败原库不变。

稳定新错误：model_revision_selection_invalid、model_revision_builtin_protected、model_revision_is_default、model_revision_in_use、model_template_default_changed，均带具体恢复action。其它沿核心stale/dependency/digest错误。

6项真实AST测试通过：精确default/取消及不可变版本；纯命中与Manual/AutoRead/草稿使用区分；additional及baseRevision依赖；saved/draft/history/子版本保护与批次原子；内置/default保护及取消后不自动latest；stale/重复/缺失/摘要错配不改库。

## 严格Agent v2提案

响应AgentProposalV2固定为contractVersion=2、expectedLibrary、proposals数组（1～32）。每项ContentProposalV2={target:Option<RefV2>,label,contentVersion:2,content:TemplateContentV2}。target=None仅表示请求新建，不允许模型提供新templateId。响应origin/revisionId/createdAt/makeDefault及任意unknown字段拒绝；递归重复键严格拒绝；旧suffix/variants/v1响应不自动转换。

可信AgentProposalContextV2刻意不实现Deserialize，须由已授权的后端任务状态构造：expected_library、allowed_targets精确Ref列表、allow_create、TrustedAgentOriginV2（InternalAgent或ExternalTask及task_id）、逐项AssignedRevisionV2（后端template_id/revision_id/created_at）。校验UUID、时间、assignment数量/唯一性；旧目标必须获精确Ref授权并与assignment模板匹配，新模板必须不存在且获allow_create。model响应不能选择可信origin、身份或时钟。

API：

~~~rust
parse_validate_proposal_v2(library:&LibraryV2,response_bytes:&[u8],
    context:&AgentProposalContextV2)->Result<ProposalReviewV2>;
plan_agent_save_v2(同输入)->Result<AgentSavePlanV2>;
~~~

ProposalReviewV2={dtoVersion,expectedLibrary,valid,proposals,issues}；每项包含index、原v2 proposal、plannedRevision（仅可验证时）、differences、dependencies、issues。差异用JSON Pointer和Option<Value>保留缺失与显式null的区别。结构/旧库错误直接返回Err，内容/锁/目标问题给对应issues。plan_agent_save返回{library,review}，任一提案无效则整批不返回可保存的部分库。

真实保存合法性/锁字段复用既有save_revision_plan_v2；每项origin只来自可信context，make_default=false，原不可变revision/default不改。没有第二套匹配/保存引擎。批次内部canonical expected随候选库更新，但目标授权仍对原库精确Ref。

7项真实AST测试通过：严格v2差异/依赖及InternalAgent origin；后端ExternalTask origin与明确新建权限；顶层/嵌套origin/身份/时间/unknown字段注入拒绝；v1/suffix/重复键/旧库拒绝；精确目标范围和digest不可替换；真实root/condition锁；无效条件与混合批次不部分保存。没有mock引擎，没有实际AI调用。

## 验证、证据与后继

正常完整应用crate受控快照、原Cargo.lock、实际核心源码；仅快照新增这两个模块注册声明，生产共享mod.rs未改。template 6测试、Agent 7测试各通过，0失败/忽略，两次正常crate Clippy --lib --tests --no-deps -- -D warnings通过；rustfmt及diff检查通过。本批没有失败检查。四份源码/测试与受测快照逐字节核对。

日志同目录template-ops-first-tests.log、template-ops-clippy.log、agent-proposal-first-tests.log、agent-proposal-clippy.log。无业务文件/日常配置读写，无上游/模型/CLI、临时服务或监听端口，受控快照/构建缓存/日志保留；未push/发布，未写共享docs/.adg。

L0仍负责同一model事务提交library与保存版本元数据，捕获当前saved/draft及明确历史精确refs。L3消费正式默认/删除/usage/Agent DTO，不能以名称命中代替使用状态。真实Agent执行器须按此schema生成提示/接响应，在后端任务授权和审阅后构造可信context，再以实际公共门写入；进程/执行目录/取消/网络/AI真实调用和最终验收不由本批纯提案测试证明。

