L1 / 0.2.23 运行与外部审阅增量回执

本批已完成可集成代码，整版验收与生产公共接线仍由L0负责。未重复旧G0分析。

代码位置：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes
分支：codex/0.2.23-model-matching
共同基线：f9a79297192e1c8ee1e66569ae51299128e8ff8e
内部运行提交：e59cfae468e75d43852fa299d532c9138948f6ee
外部门面提交：fdda071c54ff295f6ba0543c3f12f1df9ce1ed1b
前置提交：4cefab410（目录）、304e1a1e4（断言）、d97436061（同源锁helper）。

本批写集：agent_runtime_v2.rs/tests；内部提交还包括窄授权schema_v2.rs、agent_proposal_v2.rs。未写共享mod.rs、commands、旧agent/tasks/broker/runtime/management、保存门、日常配置或版本文件。

生产接口（完整类型以源码为准）：

- prepare_run(root, context, GenerateIntentV2) -> RunTicketV2
- async generate(root, context, runId) -> GeneratedReviewV2
- stop(runId), latest(root, templateId?)
- prepare_accept(root, context, runId, expectedLibrary, reviewedBytes) -> ApprovedSavePlanV2
- mark_accepted(root, context, runId, responseDigest) -> GeneratedReviewV2
- prepare_external(root, context, GenerateIntentV2) -> ExternalTaskV2
- external_context(root, taskId) -> ExternalTaskV2
- preflight_external(root, context, taskId, submissionBytes) -> GeneratedReviewV2
- submit_external_review(root, context, taskId, submissionBytes) -> ExternalReceiptV2
- receipt_external(root, taskId, submissionId) -> Option<ExternalReceiptV2>
- revoke_external(root, taskId) -> ()

外部submission严格JSON为 {contractVersion:2,submissionId:UUID,response:严格v2提案或CheckOnly报告}。每个task只固定一次有效submission；同id与同canonical response重复返回相同回执，其他后继提交冲突。预检不写记录，分配ID在提交时才固定。外部任务不依赖内部Agent偏好或推理服务。submit只持久Review或报告，不能业务保存。所有facade调用须先经L0现有broker认证、激活、proof、有效期与撤销门；taskId不是鉴权凭证。撤销须同时更新broker状态并调用revoke_external；最终提交期间授权/撤销与事务的协调由L0统一保存门接入。

内部执行真实使用temp fake官方CLI驱动现有upstream::snapshot，再通过真实Management证明核验与同runtime /v1/responses。仅指定模版Refs、requirements、history、source receipts及buffer进入prompt。测试证明原始config Key和整库没有进入prompt。CheckOnly严格报告，不能调用提案保存。取消、坏证明、超限、源变更、旧Ref和未知JSON全部拒绝。

buffer信任来自任务固定的typed content + exact savedRef + digest/generation。saved→buffer仅允许新增锁与未锁内容的人工编辑；buffer→proposal保护全部根/条件锁，响应不能新增锁权。新增带锁condition及新建onlyCreate均有正向测试。保存共用append流程，仅一次真实revision，父仍savedRef，历史不变，origin始终可信InternalAgent或ExternalTask。普通无buffer保存入口规则保持。mark_accepted核对真实落库的内容digest、backend revision ID/time/origin/parent及root/runtime身份。外部提交摘要与人工审阅后摘要分别保留。

L0接线要求：

1. 公共mod.rs注册 pub mod execution_directory_v2; pub mod agent_runtime_v2; tests已内部path注册。参考agent-runtime-registration-only.patch。
2. facade均在统一保存门之前调用，内部会取得私有record锁并捕获saved配置；不要在持有config/model事务锁时反向进入这些捕获接口。L0对ApprovedSavePlanV2的expected library/source/root完成CAS、library+content版本/元数据/回执统一事务，成功后调用mark_accepted。实际业务事务不是本批测试覆盖内容。
3. 同一个现有broker接入外部v2分发，不新造server/token/HMAC；命令和错误注册由L0完成。本批仅验证数据门面，未声称HTTP授权路径已接通。

验证：20/20（12 runtime、7纯proposal、1旧conversation）；严格Clippy --lib --tests --no-deps -- -D warnings通过；6个相关Rust文件rustfmt --check及git diff --check通过。前置目录6项通过日志保留。完整真实crate快照中所有已跟踪tauri文件与生产树逐字节一致，唯一差异是快照补的两个模块注册。完整Cargo.toml/lock与生产相同，未模拟核心或IO。日志与源码SHA256见同目录L1-agent-runtime-v2-manifest.json。初轮失败保留，不删除或覆盖。

环境边界：本机macOS/Unix fake CLI，Windows真实执行由L4负责；临时数据目录/loopback端口/CLI进程仅在测试夹具，TempDir和JoinHandle析构清理。没有真实上游推理、真实同步、日常配置更改、发布或推main。受控产物只在本机，未上传。
