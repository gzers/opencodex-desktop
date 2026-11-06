# L2 无 ETag 配置同步合同 v1

状态：可供 L0 冻结的完整合同；本文件本身不授予 R1 写权。2026-11-06，Asia/Shanghai。归属 IMP-31 P23-00/06、AC-P22-05/06。文档资料基线 `956e4cb4accbe4e80857f2e49da336e4127202fd`；只读参考代码 `0c57500f8f46ea50655c782c3f04334a39167abb`。采用本次总控裁决，替代初稿中的未决初始化方案；未修改原始 G0 回交。

本包的 `dto.schema.json` 定义闭合 DTO；本正文定义 JSON Schema 之外的关联不变量和执行顺序。两者共同构成合同。`error-policy.json`、`budgets.json`分别固化错误动作与预算。`examples.json` 是合成序列化与摘要向量；模型载荷仅为不含秘密的测试文本，不代表可导入配置。`manifest.json` 登记文件摘要和验证范围。生产源码、运行配置和共享仓库均未改。

## 1. 所有权和三层版本

- L2：代次/提交图、远端传输、冻结操作日记、回执、恢复计划。
- L1：模型 package 的编码、解码、完整字段、协议来源、不可变模版 id/revision/digest 和依赖校验。L2不得复制匹配器或重解释模型规则。
- L0：活动用户根/受控路径与连接解析、凭据引用、共享保存/备份、公共命令和错误注册；`infrastructure/webdav_client.rs`、Depth1/href/HTTP 细节由 L0 单写。L3消费DTO。

新文件前缀 `config-v2` 区隔现行 config-v1 域；Generation/Commit 元数据首版 `formatVersion=1`；payloadSchema 单独声明便携外壳、对象文档和模型包版本。不得用 transportVersion 推断模型 Schema，不根据字节相似性降级。PayloadSchemaRef 的实际支持组合由 L0 注册 L1 编码器交付的版本表；未注册组合返回 `sync_payload_schema_unsupported`。注册表是执行依赖，不改变本合同传输格式。

## 2. 字节、身份与文件

UTF-8 JSON，无 BOM；字段 camelCase，枚举 snake_case；对象拒绝未知字段和重复字段；UUID 必须小写规范 UUIDv4；摘要为 `sha256:` 加64位小写十六进制。整数非浮点且不超过u32，payloadBytes除外但受64MiB硬限。可选字段缺省用省略，不以 null/空字符串混用。UTC时间为带 `Z` 的RFC3339，表示观察/核对时间，不能作为业务保存时间或提交排序。

写者先冻结准确字节，默认无空白紧凑JSON；读者接受符合上述规则的合法JSON字节，不重新序列化再算摘要。远端完整性摘要绑定实际文件字节；测试向量的紧凑序列化仅是可复现示例。

| 远端文件 | 内容和语义 |
| --- | --- |
| `config-v2-generation-<G>.json` | Generation：format=`OCXD_CONFIG_GENERATION`、formatVersion=1、generationId=G。独立不可变文件。 |
| `config-v2-data-<G>-<O>.ocxdconf` | 本操作第一次冻结的完整载荷字节。包含基线中未选对象，所选对象按共用编码器更新；不传播整对象删除。 |
| `config-v2-commit-<G>-<O>.json` | Commit：format=`OCXD_CONFIG_COMMIT`、formatVersion=1、generationId、operationId、parents、payloadDigest、payloadBytes、payloadSchema、intent，resolve时另有resolution。最后发送。 |

G/O分别为UUIDv4，不以设备ID/时间代替。路径仅从已校验G/O派生，不接收远端自由URL。CommitRef=`{operationId,digest}`，digest是对应commit文件精确字节摘要。GenerationRef=`{generationId,digest}`绑定代次文件。父引用排序去重且同G；拒绝重复ID不同摘要、跨代父、环和缺失闭包。新写者不得修改任何已存在且不匹配的文件；同ID重试仅可复用原字节。UUID+本机操作锁是合规写者约束，不是服务器防篡改或条件创建。

`intent=update`的parents为零个或一个；零个仅限当前G确无提交。`intent=resolve`须至少两个父，恰为用户预览确认的全部已观察头。resolution含 selectionMode=`head|merged`、scopeDigest、previewDigest；head模式必须有selectedHead且属于parents，merged模式不得有selectedHead。合并由现有共享对象服务产生，L2不发明模型合并规则。全部未选范围的影响也必须入预览。

## 3. 只读观察和视图摘要

observe_remote、空目录预览、prepare的远端方法只有Depth1 PROPFIND/GET；不发PUT，不创建G。prepare可以在受控本机事务目录保存冻结计划/字节，不写业务配置、不建立用户备份。取消未执行的prepare只清理本机未提交临时材料。

成功、完整的根列举才能判空；集合自身排除。根404、鉴权失败、部分207、未知业务文件、旧域文件、坏JSON不等于空。新格式不读取旧latest、不隐式迁移。当前唯一G存在时，未带代次marker的其它G迟到data/commit只列为stale，不进入当前图；新的计划仍必须显式绑定当前观察到的G，旧计划不自动重绑定。

先列举元数据，再精确GET提交和父引用补全闭包，最后再次列举元数据集合；集合变化则重试有限次数，不能得到稳定观察时返回incomplete。两轮相同只证明本次观察稳定。已见引用按端点/G持久保留，漏列不能把已见提交忘成空；已知GET404返回不完整。实现按受限迭代图遍历，禁止无界递归。

RemoteState：`empty`（目录确空）、`generation_empty`（唯一G无有效提交/无无效提交）、`ready`（唯一有效头）、`conflict`（同G多头）、`initialization_ambiguous`（多个G可见）、`incomplete`（已知文件暂缺/列举不稳定）、`invalid`（格式/摘要/闭包不合法）。generation_empty可有明确列出的孤立payload但它不是当前配置。发现无效提交时不得以零有效头返回generation_empty。

结构化摘要使用canonical_v1：对象键按ASCII字典序递归排列（本合同全部键为ASCII），数组保留规定顺序，紧凑UTF-8 JSON，无BOM/尾换行，不转义普通Unicode/斜杠；引号/反斜杠/控制字符按JSON最短标准转义，整数十进制无前导零，无浮点/null/重复键；无Unicode规范化。SHA256针对此编码。它只用于本机派生视图/计划/执行摘要，远端文件摘要始终针对原文件字节。

viewDigest材料固定为对象 `{hashDomain:"OCXD_CONFIG_VIEW_V1",bindingId,state,generations,commits,heads,fileNames,problems}`：generations为按G排序的GenerationRef；commits/heads均为按(G,O)排序的`{generationId,operationId,digest}`；commits含全部已知有效提交，heads含各已验证G的头（歧义时顶层DTO的普通heads为空，hash材料仍保留各G的恢复头）；fileNames为相关协议及未知业务文件名去重排序；problems为`{code,step}`去重并按(code,step)排序。文件名使用已见并集，缺失结果体现在problems。不含时间、ETag、URL/用户名、载荷内容或展示文案。包含所有已知提交，不能只hash最新头。bindingId由L0提供不透明绑定摘要，不由UI自行推导真实URL。

planDigest材料为各Plan DTO去掉planDigest字段后的完整对象的canonical_v1摘要；preparedAt/expiresAt也被冻结。Journal.executionDigest则为`{hashDomain:"OCXD_CONFIG_EXECUTION_V1",operationId,planDigest,generation,commitDigest,payloadDigest}`的canonical_v1摘要；首次确认才生成G并固化commit，不更改用户已确认的planDigest。参见examples中的hashMaterials与准确向量。

## 4. 初始化和代次保护

空目录UploadPlan返回 `initialization=on_confirm`，不包含generation；UI明确“首次上传将初始化此空目录”。已有唯一G返回 `initialization=none` 及GenerationRef。用户确认的planDigest包含这一差别。首次执行锁内重验空目录/viewDigest/来源/范围后，才生成一个G，持久保存Generation精确字节和G，记下可能已发送阶段，然后PUT。重启和重试不得生成第二个G。

每次远端PUT前、PUT后、下载应用前重验G及已知代次集合。首次G写入后直接GET原文件核对、再列举；只有观测为本G唯一才继续载荷。多个G一旦可见，立即停止普通上传/下载应用，原材料全部保留。不得删marker、覆盖active-generation、按UUID/时间选胜者或自动清空。

接受的能力边界：无服务器原子互斥时，迟到初始化可能在稍后才被发现；已观察失效拒写，旧G迟到文件不进入新G视图。不承诺清空任意交错下零字节写入，也不承诺全设备永久最新。服务器最终完整可列举是发现未知并发提交的必要前提。

## 5. 上传阶段和同操作恢复

phase闭合枚举：`prepared`、`initialization_may_be_written`、`generation_verified`、`payload_may_be_written`、`payload_verified`、`commit_may_be_written`、`settled`、`cancelled_before_write`。failure记录可在任意非cancelled阶段停止，不以改成failed丢失原发送阶段。`lastWriteStep=none|generation|payload|commit`单调推进，初次网络发送前原子写日记，`mayHaveWritten=(lastWriteStep!=none)`。

执行顺序：prepared →（首次初始化）initialization_may_be_written → generation_verified → payload_may_be_written → payload_verified → commit_may_be_written → settled；已有G可直接generation_verified。每次请求前持久记录可能已发送；payload PUT后精确GET校验摘要/长度，再写提交；commit PUT后精确GET提交/载荷，吻合才能storageStatus=stored。generation创建的可能副作用不等于配置存储成功。

确认/新写入准入同时检查：planDigest、原绑定和配置修订、所选已保存来源摘要、范围摘要、冻结MCP来源客户端/定义摘要、凭据/保护意图、G及父集合、计划有效期。基准变更不自动换父/重取载荷。首次执行时才生成G，随后冻结Commit字节；本机日记引用固定payload，不再从当前编辑器或Key重建。

`verify_original_upload`始终只读远端和原日记，绝不PUT。只依赖原绑定、原G/O、冻结摘要、原精确commit字节和安全连接解析；不依赖当前MCP选择/当前范围/当前来源仍存在。即使本机后来保存或更改了勾选，也能确认原O的存储事实。读到原载荷但无可确认提交仍unknown；明确不同字节为invalid；只有404/断网/延迟不能推定未提交。

verify先检查当前代次集合，再GET原文件；代次变化仍可保留历史stored证据，但不能把旧文件残留算当前G成功。多个G可见时可只读确认某原O的精确文件，relation.remote标initialization_ambiguous，普通写入仍停止。GET结果与日记身份不符返回碰撞，不覆盖。

只读verify不要求本机原冻结大载荷仍在：只要受保护日记中的身份/摘要/精确元数据可信完整，即可直接核远端；resume则必须具备原冻结大载荷字节。日记损坏不能靠当前来源/Key重建原操作证据。

`resume_original_upload`是单独确认的允许写动作：先verify，已stored则零PUT；否则要求原冻结字节可用、原G仍唯一、当前来源/范围/MCP基准仍匹配、当前已观察父集合与冻结父一致，且用户确认最新expectedViewDigest。只补发原O的原字节；剔除本操作已写的G/payload造成的自有变化后，不允许把外部变化悄悄重基准。失效原操作仍可verify或转新预览，不能以resume生成新G/O。

## 6. 取消、下载和初始化歧义专用恢复

同O锁串行化取消和发送。lastWriteStep=none时取消转cancelled_before_write；一旦可能已发，cancel只阻止后续发送，保留阶段并提示核对，不能宣称远端撤回。底层取消/超时不能证明服务端未收。verify不受取消标志阻断。

普通prepare_download只允许唯一G/唯一已确认头，冻结文件后校验，再观察；apply下载前重验G/头/viewDigest及本机目标摘要、范围/凭据/备份意图。外部变化重新预览，备份失败零业务写入；源文件一致不替代本机目标保护。

初始化歧义时仅允许 `prepare_recovery`：用户指定sourceBindingId、GenerationRef、CommitRef和mode=`download|export`；分别校验所选G下完整闭包/载荷/Schema，不能合并多G当正常视图。计划kind=`recovery_download|recovery_export`；apply_recovery复验原文件/视图和本机目标，download仅经既有保护写本机，export沿已有文件导出服务写用户选择的位置；两者对远端零写入，不记录普通同步完成。解密/凭据选择与范围仍走共用服务。坏分支不假装可恢复；其它可验证分支独立提供动作。

原歧义目录保持；恢复后仅提示 `select_empty_directory`，由用户明确选择另一个已核实空目录，再走正常预览及首次确认初始化。此动作是打开既有连接目录编辑/选择，不会自动创建集合、搬迁、删除或上传。合同无清空按钮、无原目录初始化决议图、无MKCOL/DELETE能力。

## 7. DTO 事实语义

storageStatus=`not_attempted|unknown|stored|invalid`。not_attempted表示未尝试配置data/commit写入，可能已尝试初始化；unknown表示可能已发送但无法确认；stored只由原commit和payload吻合得到；invalid表示确认到身份/摘要/格式错误。历史stored不可因一次断网撤销，保留storedAt及最后证据；新观察错误单列，不伪造新确认。

relation独立为结构体，避免同一时刻“本机变了且远端冲突”互相覆盖：local=`not_compared|observed_equal|changed|unknown`；remote=`unknown|observed_equal|descendant|conflict|initialization_ambiguous|generation_changed`。verify不重读当前来源，local返回not_compared；调用方可另用assess_current_relation按冻结范围比较当前已保存事实，其失败不会阻断/抹掉原stored。该比较不重建带Key的传输包；当前MCP来源或范围变了时local=changed，需新预览。

remote判定先检查代次：多个G→initialization_ambiguous，原G失效/改变→generation_changed；原G唯一时，多头→conflict，唯一头恰为原O→observed_equal，唯一头且原O在其有效祖先闭包→descendant；其余或观察失败→unknown，不猜成descendant。local比较使用共用source_fact的已保存内容事实；范围或MCP来源摘要不相同即changed。scopeDigest冻结所选对象/子范围、选中凭据ID及接收映射，不含凭据值；MCP来源客户端和定义快照摘要另入mcpSourcesDigest；备份/保护选项由Plan字段及其planDigest共同绑定。所有摘要由后端重算，不信任客户端提供值。

OperationFact含dtoVersion、operationId、kind、phase、lastWriteStep、storageStatus、localApplyStatus、relation、generation可选、payloadDigest、commit可选、storedAt可选、observation可选、failure可选、recoveryActions。localApplyStatus=`not_applicable|not_attempted|unknown|applied|invalid`；上传固定not_applicable；下载的storageStatus仅表明源提交的远端存储核验，localApplyStatus单独表明本机应用结果，不能借stored声称下载已应用。下载O是本机操作ID，commit引用远端源O，二者不要求相等；上传的commit.operationId必须等于operationId。phase=settled仅表示本次执行/核对步骤停止，不等于两端永远一致。read_transfer_result只读本机；网络观察/verify必须独立命令。DTO不得含真实URL、Key、认证Header、口令、凭据值或本机绝对路径。

Journal是仅本机存储的私有DTO，不暴露于命令响应；rootRevision/connectionRevision由L0计算，payloadHandle为受控文件句柄ID，不是任意路径。generationBytesUtf8/commitBytesUtf8保存第一次冻结的小元数据原字节；后者一旦存在即须有executionDigest。Journal/plan/fact的operationId必须相同；previousFailures超64条时保留首次原因和最近63条，本机归档由既有保留服务管理，不动远端文件。

## 8. 稳定错误和 recoveryActions

错误结构为 `{code,step,retryability,cause?,httpStatus?,recoveryActions}`；cause仅枚举，不拼接URL/服务器响应正文。step=`observe|initialize|source_guard|payload_put|payload_get|commit_put|commit_get|verify|download_guard|apply_local|persist|decode|recovery`。HTTP 507仍保留状态；GET404与根目录404分清。

| code | 含义 | 可返回动作（仍需后端逐次准入） |
| --- | --- | --- |
| sync_generation_ambiguous | 多G | prepare_recovery_download、prepare_recovery_export、select_empty_directory、verify_original_upload（已有O时） |
| sync_generation_changed | 原G失效/消失 | verify_original_upload、refresh_remote、repreview_upload |
| sync_listing_incomplete | 暂缺/不稳定/部分列举 | refresh_remote、retry_later |
| sync_remote_invalid | 远端元数据/摘要/图损坏 | inspect_error_details、prepare_recovery_export（可验证分支） |
| sync_remote_conflict | 同G多头 | choose_conflict_source、refresh_remote、verify_original_upload |
| sync_operation_unknown | 发送结果无法确认 | verify_original_upload、retry_later；满足准入后可resume_original_upload |
| sync_operation_collision | 同O不同内容 | inspect_error_details、repreview_upload；绝无覆盖动作 |
| sync_source_changed / sync_scope_changed | 当前来源/MCP/范围变 | repreview_upload；原O可verify |
| sync_endpoint_changed | 原绑定不匹配 | return_to_original_connection、refresh_remote |
| sync_payload_schema_unsupported | 载荷版本组合未注册 | inspect_error_details；不提供降级导入 |
| sync_payload_invalid | 载荷或精确依赖校验失败 | inspect_error_details、repreview_upload |
| sync_limit | 任一硬预算耗尽 | inspect_error_details、select_empty_directory（显式保护恢复后） |
| sync_remote_full | 507/容量不足 | free_remote_space、verify_original_upload；应用不自动删文件 |
| sync_auth_required | 401/403认证授权 | return_to_original_connection、retry_later |
| sync_network | 断连/超时/TLS/协议传输 | retry_later、verify_original_upload（可能已发时） |
| sync_collection_missing | 根目录不存在 | return_to_original_connection；不隐式创建 |
| sync_passphrase_required / sync_passphrase_invalid | 解密缺口令/认证失败 | unlock_payload |
| sync_plan_expired / sync_plan_changed | 未发计划过期/确认摘要不符 | repreview_upload或refresh_remote |
| sync_local_changed / sync_backup_failed | 下载目标变化/保护失败 | repreview_download、inspect_local_protection |
| sync_journal_invalid / sync_local_state_io | 日记损坏/落盘失败 | inspect_local_protection、verify_original_upload（原身份仍可验证时） |
| sync_cancelled | 未发送取消 | repreview_upload；不自动恢复 |

Retryability=`never|after_refresh|same_operation_read_only|same_operation_guarded`。RecoveryAction为后端产生的闭合kind，带operationId/recoveryPlanId/generation/head等必要不透明标识，不带执行URL或shell命令；展示动作不等于自动执行。GET错误原样保留，不把认证/摘要/超限统一变成unconfirmed，更不误导查本机目录权限。

## 9. 硬预算

| 项 | v1值 |
| --- | --- |
| 单载荷文件 / 解包文档 | 64MiB / 32MiB，沿现有配置便携边界 |
| 单Generation/Commit元数据 | 各64KiB；payloadBytes先验且实际流式检查 |
| 当前G提交数 / 父数 / 头数 | 4096 / 64 / 64 |
| 一次观察代次marker数 | 64；两个可见即歧义，超上限也不能截断成唯一G |
| 单次列举条目 / XML体 | 10000 / 16MiB，集合自身计入传输限额但不计业务条目 |
| 一次观察元数据累计GET字节 / 请求数 | 32MiB / 8192；缓存命中不计GET字节，但计图节点预算 |
| 同时网络请求 | 最多4；同绑定发布准入本机串行，同O单写锁 |
| 单请求 / 一次显式观察或verify总时限 | 30秒 / 90秒，先到者生效；总期限不可因重试延长 |
| 稳定列举 / 暂缺精确GET | 最多3轮双列举 / 每文件最多3次，间隔100ms、300ms；只重读，不自动PUT |
| 未发送的上传/下载/恢复预览有效期 | 15分钟；可能已发送的O永不过期成新操作，仍允许原O只读verify |
| 原因/动作/展示文本 | code枚举；最多8个恢复动作；不保存远端错误正文 |

各预算取最先触发者；超过即停止为sync_limit/incomplete，不截断成单头/空目录，不GC、不清空、不静默删除历史。32MiB累计元数据可能早于4096个各64KiB文件触发，属有意的组合上限。预算增加须升合同修订并复验；正常同步不对所有历史大载荷逐个GET。当前选用、恢复、原O确认的payload必须完整精确GET。

## 10. 公共业务接口（IDL，无实现）

下列名称在L2模块内部固定；L0命令门面可映射既有命令名，但不可合并verify/publish或为读回执附加网络。SyncResult<T>为Result<T, SyncFailure>；所有入参是schema中的DTO，公共命令不接收原始载荷字节、秘密或路径。

```rust
async fn observe_remote(cx: &SyncContext, request: ObserveRequest) -> SyncResult<RemoteObservation>;
async fn prepare_upload(cx: &SyncContext, request: PrepareUploadRequest) -> SyncResult<UploadPlan>;
async fn publish_upload(cx: &SyncContext, request: ConfirmUploadRequest) -> SyncResult<OperationFact>;
async fn verify_original_upload(cx: &SyncContext, request: OperationRequest) -> SyncResult<OperationFact>;
async fn resume_original_upload(cx: &SyncContext, request: ResumeUploadRequest) -> SyncResult<OperationFact>;
fn cancel_upload(cx: &SyncContext, request: OperationRequest) -> SyncResult<OperationFact>;
fn assess_current_relation(cx: &SyncContext, request: OperationRequest) -> SyncResult<Relation>;
fn read_transfer_result(cx: &SyncContext, request: OperationRequest) -> SyncResult<OperationFact>;
async fn prepare_resolution(cx: &SyncContext, request: PrepareResolutionRequest) -> SyncResult<UploadPlan>;
async fn prepare_download(cx: &SyncContext, request: PrepareDownloadRequest) -> SyncResult<DownloadPlan>;
async fn apply_download(cx: &SyncContext, request: ConfirmDownloadRequest) -> SyncResult<OperationFact>;
async fn prepare_recovery(cx: &SyncContext, request: PrepareRecoveryRequest) -> SyncResult<RecoveryPlan>;
async fn apply_recovery(cx: &SyncContext, request: ConfirmRecoveryRequest) -> SyncResult<RecoveryReceipt>;
```

SyncContext由L0装配：活动根/根revision、受控journal路径/原子落盘/锁、按原binding解析的连接handle及配置revision、只读模型/对象事实与共享保存/备份服务、PayloadCodec注册表、受限网络trait。handle不跨IPC序列化；旧路径不得回退扫描。原连接不再可解析则明确endpoint_changed，不拿当前新连接查询旧O。

L1边界通过共用PayloadCodec门面注入，IDL：`supported_schemas() -> Vec<PayloadSchemaRef>`；`freeze_selected(scope, expected_source) -> FrozenPayload`；`inspect_and_validate(bytes, expected_schema, secret_handle) -> ValidatedPayload`；`merge_selected(base, selected, dependencies) -> FrozenPayload`。FrozenPayload含私有字节handle、准确摘要/长度、PayloadSchemaRef、公开范围/来源摘要；ValidatedPayload交给既有接收服务。秘密handle仅内部，L1不接远端网络，L2不解析模型字段。

## 11. L0 网络 trait 精确需求

增量独立trait，不修改旧同步域的ETag条件接口语义；无自动PUT重试/重定向，不提供DELETE/MKCOL。类型属于L0网络模块，L2只消费。

```rust
#[async_trait]
trait ConfigSyncTransport: Send + Sync {
    async fn list_children(&self, connection: &ConnectionHandle, limits: ListLimits)
        -> Result<ChildListing, SyncTransportError>;
    async fn get_file(&self, connection: &ConnectionHandle, path: &SyncRelativePath, limits: ReadLimits)
        -> Result<FileRead, SyncTransportError>;
    async fn put_file(&self, connection: &ConnectionHandle, path: &SyncRelativePath,
        frozen_body: &[u8], limits: WriteLimits) -> Result<PutResponse, SyncTransportError>;
}
```

list_children：仅配置集合自身、PROPFIND Depth1，请求必需DAV:resourcetype，不使用ETag；返回ChildListing=`{entries: Vec<ChildEntry>, http_status: u16, response_bytes: usize}`；ChildEntry=`{name: String, is_collection: bool, content_length: Option<u64>}`。只有严格成功且完整的207解析才返回Ok；任何子项整体失败或必需resourcetype的propstat失败、XML截断、超限、无法确认href归属均Err。可选getcontentlength若单独404只置None，不把可选属性缺失误判成子项消失；长度仍只作预算提示，不能判内容身份。Depth1服务端没有“全局完整性证明”，Ok只表示本响应解析完整。

ListLimits=`{max_entries:u32,max_xml_bytes:u64,timeout_ms:u32}`，由本合同预算填入；L0执行Content-Length和流式XML字节/条目限制。L2负责90秒总deadline、累计请求/元数据预算、有限重读和同操作发送前日记，L0不自行延长或自动重试写请求。

href可绝对或相对，由L0按响应URL解析；认证不得发送到外域。不跟随重定向；限制同scheme/host/port、同配置集合的直接子项，检查一次百分号解码后的路径组件，拒绝编码分隔符/反斜杠/点段/跨目录/查询/fragment/重复身份；允许集合自身但只计传输限额，不返回为子文件。禁DOCTYPE/实体扩展。raw href不直接传给L2 GET。SyncRelativePath只能由本协议校验G/O后构造，不能任意URL。

get_file：FileRead=`{body: Vec<u8>, http_status: u16}`，仅接受200完整文件，不接受206；ReadLimits=`{max_bytes:u64, timeout_ms:u32}`；在Content-Length和流式块两处检查字节限额；可请求no-cache但不据此承诺无缓存。

put_file：PutResponse=`{http_status:u16}`，仅接受200/201/204，发送冻结字节、禁If-Match/If-None-Match和ETag前置条件；请求前L2已经落盘可能发送事实。WriteLimits=`{max_bytes:u64,timeout_ms:u32}`；此函数不能自行盲重传。

SyncTransportError=`{kind,http_status?,delivery}`，kind=`not_found|unauthorized|forbidden|timeout|disconnected|tls|redirect|invalid_listing|invalid_href|partial_response|too_large|insufficient_storage|server|protocol`；delivery=`not_sent|may_have_been_sent|response_received`。L2在已持久标可能发送后，即使适配器估计not_sent也保守保留原O，不能据单次错误生成新O。禁止原始URL、认证头、响应正文入日志/DTO；HTTP状态与方法/步骤必须保真。

## 12. 冻结与交付条件

本包已落实本次总控的全部L2裁决，原始方案不再有初始化决议图备选或清空按钮。模型支持版本表、L0服务具体Rust类型和命令注册由各owner在共同基线装配，不影响此闭合DTO/传输合同。R1需要L0发布本包摘要、执行代码SHA、L2独占checkout/逐文件写集、L0网络接口落点及所用PayloadCodec注册版本；本次仅提交合同产物，不自授编码权。

验证范围仅本包JSON Schema、合成例子的结构/摘要/父图/状态及动作不变量。不是生产并发测试、真实WebDAV能力证明或产品验收。未访问真实WebDAV/Key/日常配置，未清空、同步、构建发行或发布。
