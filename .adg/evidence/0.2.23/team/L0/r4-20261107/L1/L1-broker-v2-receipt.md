L1 / 0.2.23 单broker v2最终增量回执

本批授权写集已完成并分片回交。工作区干净。正式发布、共享公共命令、真实library+版本元数据事务及整版验收仍由L0拥有。

代码工作区：/Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes
分支：codex/0.2.23-model-matching
本次复验HEAD：0104fbe7dcafd9fb275ebf11fb99f8985b5ab661
已公布共同基线：978f18e304813b2f7970b7a17c72333a550c3033
贡献提交：d3afc8e5874cffebeac8d93270cfe2fa3dd8b202（首可编译单broker片）；b76745a9422b0bf05c4821d35c410e34f8b64141（后端executor分流）；a1e46fe3b0f269bb04d3a347051d5d2fe94b55cd（授权/恢复边界）。总控已确认取得这些贡献。集成f0与978为明确公布基线的正常merge，无手工越写集。

本批只修改broker.rs、内部path注册的broker_v2_tests.rs，和授权的agent_runtime_v2.rs/tests必要readonly helper。未改commands、旧tasks、共享mod注册或日常配置；tasks旧proof退役及共享Clippy清理由L0提交。

公共接口（FsPath即std::path::Path）：

- async prepare_external_broker(root:PathBuf, context:ProcessContext, intent:GenerateIntentV2) -> ModelResult<ExternalBrokerBundleV2>
- Bundle仅含taskId、content、preparedTicket、expiresAt。preparedTicket是不进入复制内容的opaque能力；不接UI自报TaskGrant或执行上下文。
- activate_external(root, context, taskId, preparedTicket) -> ModelResult<()>
- revoke_external(root, taskId) -> ModelResult<()>
- with_external_authority(root, context, taskId, closure:FnOnce()->ModelResult<T>) -> ModelResult<T>
- agent_runtime_v2::requires_external_authority(root,runId) -> schema_v2::Result<bool>。只读私有后端record.executor；读取错误必须传播，不能fallback internal，不能从UI字段猜。

服务继续唯一listener、唯一descriptor manager-state/model-task-broker.json、同/v1/tasks/{id}传输路径及成熟64hex token来源。HMAC沿用ocxd-model-task-v1域串、UTF8 sha256:hex(SHA256(bearer))密钥、taskId/challenge/authority/pid绑定。未新造端口或proof体系。旧v1任务记录/请求体/prepare旧API明确task_contract_unsupported，不读v1库、不迁移suffix数据、不默退。

私有grant在D/manager-state/model-task-broker-v2，固定version2、task/run/root/后端ProcessContext、task和contract digest、source digest、expiry、激活/撤销与单payload身份；0600原子写入并回读，严格JSON/UUID/路径/64KiB。只保存bearer和opaque票据摘要。只有复制激活后才允许proof/context/contract/preflight/submit/receipt；未激活、跨任务token、Origin、错误Host、过期、撤销全部拒绝。

preflight/submit用4MiB限额raw Bytes再严格解析，拒重复key/未知字段/v1 payload。Auth与精确目标scope成功后才为合法submit预留submissionId+原始bodyDigest；同ID换任何字节409，原始字节重试幂等。实际submit只调用同v2纯审阅，持久Review或CheckOnly报告，不自动写库。HTTP回执携带submissionId、canonical submissionDigest与原始payloadDigest。重复submit修复已持久审阅的latest索引。撤销已accepted任务仅撤回transport，保留真实accepted revision与库。

L0公共accept必须先调用requires_external_authority，再将外部整个closure放入with_external_authority；closure内才可mark_accepted幂等快捷返回、prepare_accept、统一事务commit、mark_accepted。锁序grant→run→原保存门，不能持config/model事务锁反向进入。总控已确认采用此分流；本批不代替其公共入口集成验收。

生产源码直接验证（没有注册快照补丁）：broker HTTP 9/9，Agent相关21/21（13 runtime、7纯提案、1现有会话），合计30/30；完整cargo clippy --lib --tests --no-deps -- -D warnings通过；本批4文件rustfmt --check、git diff --check通过。初轮旧proof warning与f0基线9条共享测试lint日志保留；L0发布f0/978修复后正常merge重验，无allow或假引用屏蔽。

真实loopback HTTP证据覆盖：复制激活前拒绝；原HMAC proof；Host/Origin/bearer/跨task/expiry/revoke；合法v2 contract/preflight/submit只待审、raw重放与冲突；CheckOnly；strict JSON/body limit；旧v1body/旧持久task拒且无库变更；同一public prepare/resume listener不自动激活；原端口被占用时重定位、先proof再Bearer恢复回执；grant来源/严格版本/重复key/symlink；最终authority guard与撤销互斥；后端readonly executor不接受UI自报，external不能调用internal generate。

一个接受后撤销夹具只在TempDir模拟owner库写，用于验证已落revision保留与grant撤回；不声称该直接写法就是生产多对象事务。其余HTTP submit均证明library/draft不落盘、原配置不变。

检查命令与全部源码/日志SHA256详见L1-broker-v2-manifest.json。测试仅本机macOS临时D/H/context与loopback，不真实推理，不真实同步，不改日常配置、不发布。Fixture持TempDir、listener JoinHandle；测试进程已退出，进程/端口结束，临时数据由析构清理；仅保留受控日志和cargo target。Windows原生执行仍L4负责。产物当前本地未上传。
