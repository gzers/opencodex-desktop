# L1 Agent实际v2接线：有界只读分析与下一批建议

2026-11-06，Asia/Shanghai。仅源码读取，没有改现有文件、运行CLI/模型/网络/测试。观察根：/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes，观察时HEAD 8fa93f6621565aac3198fc99381eaf2c3b2e845a；其中with_saved_config等是L0并行工作区字节，不将HEAD冒称包含全部内容。行号为本次观察定位，文件摘要另列manifest。

## 1. 直接阻塞与最小影响

| 位置（model_config下，除特别注明） | 确定现状 | 需切换的最小边界 |
| --- | --- | --- |
| execution_directory.rs:118～135、170～177 | build调用config_file::snapshot，仍经v1 Library/Policy；connection_matches比较已删除的connection.protocol；仅按逻辑channel.connection.provider_id找runtime行。 | v2目录必须使用with_saved_config真实保存来源，并按MappingFileV2的runtime provider→逻辑UUID关联；非anchor组不能丢行，也不能从旧channel协议回填。 |
| agent.rs:651～678；tasks.rs:184 | generate在请求之前调用Tasks::prepare，后者读ModelStore::load_library，schema2即失败。 | v2 prepare从真实LibraryV2和授权精确Ref建立后端任务，不调用Tasks::prepare。 |
| tasks.rs:31～68、239～257、636～637 | CreateTask/EditingBuffer/Target/Task为v1 TemplateContent/Revision，prepare/activate硬编码contractVersion1；contract_definition宣称longest literal suffix。 | prompt、任务和编辑缓冲必须v2 AST/Ref；不能只换prompt字符串或将条件压回suffix。 |
| agent.rs:717～777 | prompt用v1任务/contract；响应反序列Submission v1并调用v1 Tasks校验。 | v2 prompt→实际runtime生成→严格AgentProposalV2→f76 parse_validate_proposal_v2。 |
| agent.rs:200～223、307～378；tasks.rs:404～405、459、567 | latest/result/review/receipt/submit最终读取或写v1 ModelStore，老GeneratedRecord/Submission也是v1。 | 新v2运行/提案记录与回执入口；审阅保存交L0统一library+版本事务，不能回到save_internal_review/save_submission(v1)。 |
| broker.rs:258～327、337～355 | 唯一本机broker的contract/preflight/submit/receipt、bundle均委托Tasks v1；Json<Submission>在入口就按旧结构解析。 | 保留单listener/回环证明/撤销/过期/回执机制，按后端任务契约路由到v2 facade；须改typed入口，不能仅把bundle文字改为v2。路径/v1和descriptor version1是传输层，不应误当业务内容版本必须是1。 |
| commands/model_config.rs:414～446、475～552、617～649 | public accept/latest/external/directory/preferences-bind/generate/stop都委托以上旧路径。 | L0把这些实际消费者指向v2实现；旧高层不能作为schema2 fallback。preferences读写本身可保留，bind换v2；不能用“入口永久禁用”收尾。 |

这是实际调用边界清单，不是整个Agent或项目审计。

## 2. 能原样复用的真实接口（与新增建议分开）

~~~rust
// 官方CLI只读快照，路径/HOME均由ProcessContext给出，无v1管理库依赖。
upstream::snapshot(context:&ProcessContext) -> ModelResult<UpstreamSnapshot>; // async
upstream::read_command(executable:&Path,working:&Path,home:&Path,
    operation:ReadOperation) -> ModelResult<serde_json::Value>; // async

management::Management::bind(context:&ProcessContext) -> ModelResult<Management>; // async
Management::base_url(&self) -> &str;
Management::verify_binding(&self) -> ModelResult<()>;

agent::settings(root:&Path) -> ModelResult<Option<AgentSettings>>;
agent::save_settings(root:&Path,settings:AgentSettings) -> ModelResult<AgentSettings>;
research_receipts::verify(root:&Path,sources:&[ResearchSource]) -> ModelResult<()>;
operations::Registry::start(&'static self,id:&str)
    -> ModelResult<(tokio::sync::watch::Receiver<bool>,Running)>;
operations::Registry::stop(&self,id:&str) -> ModelResult<()>;

runtime::runtime_binding(context:&ProcessContext) -> ModelResult<String>;
execution_directory::connection_binding_for_raw(
    context:&ProcessContext,provider:&str,row:&Value) -> ModelResult<String>;
~~~

最后两个为pub(super)，同域新增模块可调用；它们不读v1库。旧connection_binding只含exe/H/provider/adapter/baseUrl/authMode/apiKeyTransport/responsesPath，不含v2映射、逻辑UUID、实际模型/字段选择，因此可复用为公共方向组成部分，不能单独充当完整v2执行绑定。

现有生成的真实执行器是**经证明的同一个官方OpenCodex runtime的Responses端点**：agent.rs:705先Management::bind，:730 POST其base_url+/v1/responses，body的model取实际目录namespaced selector，生成完成后:775 verify_binding。现有Desktop没有可直接复用、与v1任务无关的公开“任意prompt生成”函数。

不能误用两个看似接近的接口：

- Management::request(&self,method:reqwest::Method,path:&str,body:Option<Value>)->ModelResult<Value>虽公开，但management.rs:190～200白名单不含/v1/responses，直接调用会management_scope失败。不要擅自把通用管理白名单放开来绕过。
- ControlledProcessRunner/ProcessRunner是Start/Stop/Restart生命周期接口，ProcessCommand没有生成prompt/stdin/stdout协议；不能宣称它是已有模型生成器或发明CLI生成命令。假子进程用于测试ProcessContext和CLI快照/本机runtime服务，不是生产另造shell执行路径。

最小独立新增方案：agent_runtime_v2只做薄v2编排，仍调用Management::bind/base_url/verify_binding，沿原Responses协议/5s连接/120s生成/4MiB响应预算/no_proxy/禁redirect和取消保护，送到同一已证明runtime；不再注册另一套本机模型进程或推理服务。若L0要求Desktop transport函数也严格单源，可另由L0从agent.rs:705～775抽出与任务/库无关的execute_bound_prompt helper，旧/新编排共用；该helper目前不存在，不能伪称已可调用。

## 3. 不应为schema2重写的部分

saved_runtime_reconcile.rs::reconcile(management:&Management,context:&ProcessContext,renames:&BTreeMap<String,String>)->ModelResult<()>（async）读的是原版config.json的provider roster/default，并走官方editor owner；没有读取v1模板库。schema::*在此主要是ModelResult，不构成schema2阻断。可继续用于已有、明确确认的运行态同步/重启入口，Agent目录不应暗自触发它的管理写操作。

runtime.rs::runtime_binding同样可复用。相反runtime::prepare_scoped:1466读ModelStore::load，compile_materials:842走v1 compiler::project；v2提案审阅/保存不应绕回这两条旧写链。

agent_conversation.rs的本机历史/草稿输入存储只有templateId/baseRevision字符串、history/source文本，无TemplateContent v1，可按数据继续使用；不能据此把历史文字中的旧suffix语义作为新合同。research_receipts::verify可原样复用，citations(Task,Submission)则绑定v1类型，需v2薄校验：沿已核验source receipt约束新增引用/recordedAt，不接受模型自证资料来源。

## 4. 保存来源与目录绑定建议

实际已存在的源边界：

~~~rust
config_file::v2::with_saved_config<T>(
    root:&Path, home:&Path,
    read: impl FnOnce(&LibraryV2,&PolicyV2,&MappingFileV2,&Value,&str,&str)->Result<T>
)->Result<T>;
// closure参数依次为library,实际saved policy,runtime mappings,
// 原件Value,raw/source digest,root revision。
~~~

同步闭包内只构建owned、脱敏的SavedExecutionV2，不await/取第二次相同锁/读取draft。锁释放后才做CLI/HTTP；发送前和接收后重取已保存身份事实核对，不能把同步闭包跨await持锁。原件Value有秘密，不能整包返回UI或塞prompt；只摘方向事实/规则，必要凭据指纹在闭包内形成不透明绑定。

目录联合来源：保存PolicyV2/LibraryV2/MappingFileV2提供逻辑UUID、启用/选择、最终字段和引用；UpstreamSnapshot提供实际runtime provider/raw id/namespaced、当前可用能力/picker/native状态。两者必须同向匹配。运行provider可以是非anchor；显示仍逻辑渠道名/原始ID，实际selector只取官方行，不能由显示名拼接。不在live中的已保存模型可留不可用原因，不猜新模型，不静默换模型。

建议新增execution_directory_v2.rs（另配execution_directory_v2_tests.rs，由主文件cfg(path)注册）：

~~~rust
capture_saved(root:&Path,context:&ProcessContext)->Result<SavedExecutionV2>;
build(saved:&SavedExecutionV2,live:&UpstreamSnapshot)->Result<ExecutionDirectoryV2>;
bind(saved:&SavedExecutionV2,live:&UpstreamSnapshot,settings:&AgentSettings)
    ->Result<BoundExecutionV2>;
validate_binding(context:&ProcessContext,current:&SavedExecutionV2,
    live:&UpstreamSnapshot,bound:&BoundExecutionV2)->Result<()>;
~~~

BoundExecutionV2由后端构造（不Deserialize为调用权限），至少持逻辑channelId、实际runtimeProviderId/rawModelId/actualSelector、最终protocol、公共连接绑定、root/runtime身份、受影响模型规则/可用性摘要；运行快照rawDigest用于本次前后检查。不要将整个原件rawDigest当成永久设置的唯一身份，避免不相关runtime bookkeeping导致无意义重新选型。旧AgentSettings可以继续存在当前preferences命名空间，但执行前必须经v2 bind；旧缓存绑定不靠强制匹配偷偷恢复。

## 5. 生成/外部任务/审阅API建议（新增独立模块）

agent_runtime_v2.rs及agent_runtime_v2_tests.rs；复用现f76纯提案，不改现有core的内容类型。建议后端分两步发票据，让UI停止操作能拿到服务端生成runId，而不是UI指定origin或revision身份：

~~~rust
prepare_run(root:&Path,context:&ProcessContext,request:GenerateIntentV2)
    ->Result<RunTicketV2>;
async fn generate(root:&Path,context:&ProcessContext,run_id:&str)
    ->Result<GeneratedReviewV2>;
stop(run_id:&str)->Result<()>;
latest(root:&Path,template_id:Option<&str>)->Result<Option<GeneratedReviewV2>>;
prepare_accept(root:&Path,context:&ProcessContext,run_id:&str,
    expected_library:&str,reviewed_response:&[u8])->Result<ApprovedSavePlanV2>;
~~~

以上是建议签名，不冒称现有代码已实现。GenerateIntentV2仅用户意图：scene、精确targets、allowNew、requirements、history、已验证sources，以及需要时的typed v2编辑buffer；不含可执行路径/服务URL/HOME、origin、新revisionId或保存时间。RunTicket、允许目标、assignment、task/run来源和固定保存身份全由后端构造并持久记录；上下文不向模型或UI开放为可写权限。

prepare_run只从真实saved源选执行模型，prompt只放获授权目标的TemplateContentV2/Ref与v2 schema，不能把整库、所有渠道或原件认证发给模型。generate走实际runtime Responses→严格响应text→parse_validate_proposal_v2；结果持久为待审而非自动保存。accept复核原run/snapshot/hash/授权，调用plan_agent_save_v2；L0同一事务提交library+保存版本元数据+幂等回执，保存成功后才标reviewed。不走旧agent::accept_review或Tasks::submit的v1写库。

已有Scene可作为纯枚举复用。两个原功能边界须明确保留：

- CheckOnly：f76提案要求至少1项proposal，本身不是report-only响应入口。v2包装应按后端scene接严格报告DTO且零写，不把场景永久禁用或伪造一个无修改proposal。
- 未保存模板编辑buffer：旧EditingBuffer含v1 AST，是实际类型阻塞。应给typed v2 buffer+原Ref+buffer digest；buffer只能是数据/审阅基准，不成为执行模型runtime来源或伪造保存revision。f76当前锁校验对精确已存Ref，若保留“未保存buffer新增锁”场景，wrapper必须复核buffer锁/代次；可下一批明确暴露同源pure锁校验入口，不将buffer压回v1或自动保存它。

外部任务仍共用现broker listener/来源身份，不新开第二个port或任务守护进程。建议agent_runtime_v2提供prepare_external/context/preflight/prepare_external_accept/receipt薄facade，后端run record统一区分InternalAgent/ExternalTask来源；L0 broker与public commands路由到它。HMAC证明/loopback/禁代理/认证/expiry/revoke/同事件不同内容拒绝沿旧机制；提交字节在Json<Submission>旧typed extractor前改为v2严格入口。业务contractVersion2与broker transport version1可分层，但任务数据记录/激活校验不能再硬要求1。

## 6. 最小写集与L0接线

L1下一批建议独占新增四文件：execution_directory_v2.rs、execution_directory_v2_tests.rs、agent_runtime_v2.rs、agent_runtime_v2_tests.rs。先交真实目录+bind小片，再交v2 prompt/实际Responses/待审/plan小片；不重写runtime安装、process_runner、management listener或旧v1核心。

L0继续单写公共mod/commands、broker/tasks公共路由与权限、真实配置capture/统一template事务及运行来源服务。如选择共用execute_bound_prompt helper提取，需要另明确agent.rs单文件owner；在未分配前L1不修改现文件。旧研究citations与buffer锁泛化若需要共享helper，同样逐文件明确，不默扩写集。

## 7. 获准后的代表验证

使用临时D/H、假官方可执行子进程及仅回环fixture服务，不真实推理：假CLI按真实只读operation返回provider/live/order/catalog/status；真实Management绑定验证fixture runtime record及health challenge；捕获真实POST /v1/responses的v2 prompt并返回固定v2 AST响应；经真实f76校验出review，再由L0真实事务保存+保存时间并重开验证。

必需边界：非anchor映射/parked anchor/同ID跨渠道、原始slash ID/selector不猜、草稿与未保存凭据不进入执行源、改名/节点/协议/启停/选择变化导致原绑定拒绝、错PID/端口证明/executable/H变化、取消前不发送与迟到不保存、超时/HTTP失败/响应超限/坏JSON/V1响应保留结果、来源receipt/锁字段、库基准变化时不偷偷另存、同run审阅重放/冲突、CheckOnly零写、原有未发送输入/会话历史不丢、外部proof/preflight/receipt及撤销过期。

本轮只有只读分析，没有执行上述任何测试、网络或进程。没有源码commit，下一步等L0给完整基线/独占新增写集及必要公共helper归属后再编码；目标是实链切换，不以“Agent入口禁用”收口。

