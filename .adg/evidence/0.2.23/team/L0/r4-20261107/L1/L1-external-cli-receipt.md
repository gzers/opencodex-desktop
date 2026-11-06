L1 / 外部CLI消费者有界核对回执

完成提交：f406e5c7b0f64021c1d3c6d6e9e556dd8f5a4f45
基线：33d2c65b58ba8880d6deea7b52d69117238c3d33；隔离分支codex/0.2.23-model-matching，工作区干净。
本轮写集仅broker.rs与broker_v2_tests.rs。未改UI、settings的13项旧断言、commands/tasks/共享注册或日常配置。

审核结论：先前9项已覆盖真实HTTP的same broker提交与恢复，但客户端是Rust reqwest；不能直接称为CLI子进程实证。导出文案仅称preflight valid，未指明普通提案的review.valid，且CheckOnly实际只有report、没有valid字段。本笔修正该消费缺口，补充外层{contractVersion:2,submissionId,response}、响应Schema定位、原始字节重试、待审回执身份及重读descriptor恢复说明。opaque preparedTicket只由管理器激活，不复制给外部Agent。

无新增重复测试：复用现有restart_reuses_one_descriptor_recovers_receipts_and_never_activates_prepared_tasks和reports_are_review_only_and_strict_json_scope_and_body_limits_fail_closed，在Unix改为真实/usr/bin/python3标准库CLI。已有9项仍9/9，通过3个独立CLI进程：普通提案submit、CheckOnly submit、换端口后的receipt恢复，全部exit=0。CLI直接读取原样导出task/transport，原proof/HMAC先于Bearer，每连接禁止代理/重定向，核对context/contract、preflight、taskId/submissionId与原始payloadDigest。恢复只GET原receipt，不POST submit、不写库、不假生成模型结果。现有库/draft不创建，原配置字节不变。

复验命令：

~~~sh
cargo test --offline --locked --manifest-path /Users/ezio/.codex/worktrees/ocxd-023-model-matching/ocx-0219-fixes/apps/desktop/tauri/Cargo.toml --target-dir /Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/batch2-crate-target --lib modules::model_config::broker -- --nocapture
~~~

完整严格cargo clippy --lib --tests --no-deps -- -D warnings、rustfmt --check与git diff --check通过。测试日志broker-v2-cli-first-tests.log直接记录3个真实子进程PID和exit，不含bearer/ticket。未重复运行本轮未改动的21项Agent回归；此前结果仍由旧回执拥有。

可收取脚本external_task_client.py由测试内EXTERNAL_CLI_REFERENCE逐字节导出，SHA256 0a9f6975d08057e5de49df5ac568c761d62744872cbd25c48f356914fd1c36f5。生产不依赖Python；它是可执行参考消费者，不是新推理进程。用法：把复制内容末尾JSON块放到私有client-input.json的bundle字段，submissionPath为事先制作、保持字节不变的严格v2请求文件绝对路径。不要把preparedTicket放进bundle。示意输入为{"bundle":复制JSON块,"submissionPath":"绝对路径/request.json"}。

~~~sh
python3 external_task_client.py submit < client-input.json
python3 external_task_client.py receipt < client-input.json
~~~

submit执行proof→context/contract→preflight→submit，只产待审回执；receipt重读discoveryFile、重新proof，仅查询原回执。若回执null，保持未知，脚本不自动重试提交。日志、脚本和源码SHA256见L1-external-cli-manifest.json。

已向L3回交精确消费建议：

- 等L0正式三命令注册名，用现有prepare_external_broker/activate_external/revoke_external合同，root/context由App注入，UI不自报executor或TaskGrant。
- prepare预览不激活；clipboard.writeText(content)成功后才activate(taskId,preparedTicket)。复制或激活失败分别表达，不假成功。仅复制content，opaque票据不进正文、日志、历史或同步。
- 外部任务不依赖内部模型偏好，不能因executionPreference未选而全禁外部操作。buffer先用model_v2_agent_buffer获得typed digest/generation，并固定exact Ref及expectedLibrary。
- 选定saved目标按真实templateId查model_v2_agent_latest；onlyCreate无真实savedTemplateId时传null且核对runId===bundle.taskId，不能使用前端临时ID或展示/接收另一任务结果。
- Review只表示待审，使用原runId/expectedLibrary及严格reviewedResponse走统一model_v2_agent_accept，由L0后端grant分流；CheckOnly report无保存按钮。撤销只撤能力，不删已accepted revision。

环境与边界：macOS、临时D/H与loopback；没有真实推理、真实同步、日常配置改动或发布。CLI进程已结束，fixture TempDir/监听端口由析构与测试进程退出清理，保留受控日志/target。Windows原生CLI执行由L4负责。本地未上传，可由L0收取归档。
