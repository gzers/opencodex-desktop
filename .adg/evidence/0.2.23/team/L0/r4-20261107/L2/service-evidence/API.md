# L2 实际同步 Service 回交

只手改sync_service_v2.rs/tests.rs；Host必要修复按单独授权提交。L0公共commands/lib.manage、L3真实界面、identity/local/receive owner文件不由本线编辑。

## 生产构造与方法

`SyncServiceV2::production(D: PathBuf, H: PathBuf) -> SyncResult<Self>`：固定实际用户根/配置HOME，构造实际ProductionServiceBackendV2、FileWebDavTransport和D-bound凭据owner。L0可以一次manage工作者，并以storage admission覆盖本机写操作；active根变化需重建明确D/H工作者，不使用进程HOME或全局configure。

`with_file_transport(D, backend, Arc<FileWebDavTransport>, credentials)`用于L0已创建adapter及仅测试feature的CA；普通`new`支持确定性网络故障trait。所有实际网络调用统一4许可，Host/core的总deadline保留。真实捕获、原子记录和模型/资产应用置于spawn_blocking线程，网络await在其runtime block_on内执行；session的tokioMutex只锁本session，原O取消不排队等网络。

| 方法 | 公共接线 |
| --- | --- |
| catalog(readPassphrase) | config_v2_catalog({readPassphrase:null或string})；临时读取保护口令立即Zeroizing，不落scope或journal。 |
| read_scope() | 零网络，返回可选ScopeRequestV2。 |
| save_scope(ScopeRequestV2) | 顶层{selection,mcpClients,assetsReviewed}，不套额外request层。明确空保持未选，不变全选。 |
| preview(PreviewRequestV2, ServiceSecretsV2) | 服务端sessionId，读取云图/确切载荷、重捕获/冻结scope后prepare，远端零PUT。 |
| confirm({sessionId,planDigest}) | 用户确认原计划，上传才PUT，下载走实际LocalApplication/事务。 |
| result(operationId) | 只读本机原回执，零网络、不重捕获。 |
| verify(operationId,readPassphrase) | 原绑定和journal独立历史核对，不需要当前preview/source。 |
| resume_preflight(operationId,readPassphrase) | 只读远端预检，返回原ResumeUploadRequest及canResume/reason/observation；不用新preview digest冒充原plan。 |
| resume_original(ResumeUploadRequest,readPassphrase) | 跨重启同O、原planDigest和当前view；已Stored先只读核对，真正重传才恢复可信source/root/scope/G/父检查。不能恢复可信源时明确失败。 |
| cancel(sessionId) | 关闭未执行会话，上传沿core区分未发与可能已发，不宣称远端撤回。 |
| recovery_preview(RecoveryRequestV2,readPassphrase) | 仅多G歧义图中已验证candidate的明确G/head，原目录不清空，不挑latest。 |
| recovery_confirm({sessionId,planDigest}) | 沿core专用RecoveryPlan下载或导出；无远端PUT/DELETE。 |
| recovery_result(recoveryPlanId) | 原专用恢复回执，只读本机。 |

内部session resume不是唯一公共入口，UI不得为原操作恢复猜sessionId。未实现用message文本推断恢复动作；保留稳定错误与结构化actions。

## 实际公开DTO

请求均deny_unknown_fields，且不接受D/H/URL、任意handle、provider公开JSON或credentialRef作为UI权威。CatalogRequest仅Deserialize+Drop，无Debug/Serialize；ServiceSecrets为私有Zeroizing值，无Debug/Serialize和持久化。

- PreviewRequestV2：selection、mcpClients、assetsReviewed、selectedHead、forcedDirection(upload/download可选)、backupBeforeUpdate。
- ScopeRequestV2：selection、mcpClients、assetsReviewed，三者明确。
- CatalogV2：connectionState、entries、mcpSources、selected、initialized、cloudState、heads、recoveryActions、完整observation（含recoveryCandidates）。
- ObjectEntryV2：kind/id/label、availableLocal/Cloud、modelIds并集、models逐rawModelId/label/observed/selected/两域availability/familyId、familyIds、精确revisions RefV2及defaultRevision、secretAvailable。template引用绝不根据name猜ID；同revisionID不同digest拒绝。
- McpSourceEntryV2：name、sources:[{source:knownClient或managed,definitionDigest}]、differingFields。摘要由实际portable_assets明确来源导出结果调用mcp_source_digest产生，不另造hash；冲突来源不默认选一个。
- PreviewV2：sessionId、planDigest、chosenDirection、automaticReason、local/cloudSavedTime、noChange、backupBeforeUpdate、公开diff entries、local/cloudSummary、plan:{kind,plan}。没有payload/secret值，diff不公开秘密摘要值。
- ResumePreflightV2：request原ResumeUploadRequest、canResume、reason、observation。stored/read-only verify与后续source已改事实区分。
- RecoveryRequestV2：selection/mcpClients/assetsReviewed、generation/head、mode(download/export)、backupBeforeUpdate。

首次真实成功读取本机+云目录才写initialized及当时明确选区；读取失败/歧义不把截断目录写成初始化成功。之后新增对象只列目录，不扩已存范围。云端独有渠道/模型进入公共库存，首装可以正常选择下载。凭据库存只读取可用性事实，不为列举读未选Key；默认首次选择按明确允许的内容/凭据对象，未知同名MCP冲突仍需明确来源。

## 真实Owner组合

ProductionServiceBackend直接调用已提交capture、wrapper、overlay和identity owner，不复制匹配/编译/时间算法。空云上传直用owner.Upload；有效cloud的接收映射用owner.Download。强MappingPreview贯穿provider和session，不用只有hash的Weak记录自授资格。newLocal→portable使用稳定原UUID或既存known pair，不按名字合并。

ReceiveFactory的ModelApply.prepare接受core.prepare_download已产生的operationId，才调用同MappingPreview.after_adoption(id)构造真正ReceiveFileHostV2；模型、library/policy/ledger、版本、回执与同步身份映射沿实际固定八文件事务写入，不另分配O或绕回v1。

SourceSnapshotProvider桥的current用同MappedCaptureProvider，root只取actual with_saved_config的rootRevision，target用同完整LocalTargetBinding.digest。模型已Applied后，继续manager/assets必须先确认actualModelApply.status(id,handle)==Applied，再samepreview.verify_adopted(id)证明本O后继映射；不能重新验证old Absent或任意当前新map。

读取保护口令私有传入LocalApplication，原metadata records不带口令。确认备份选项沿Host.prepare_with_backup绑定core plan，默认仍true。

原绑定记录存endpoint/ref/id/revision和原选区、actualroot摘要，checksummed，密码不存。重启verify只还原可信原record+journal，不由active连接替换；原ref/password变更保守拒，坏记录拒绝联网。

## 已验证与未关闭范围

生产正常crate的14项聚合测试通过（详日志），包括实际saved上传、首装模型接收回执+映射、首装偏好/扩展/Skill/MCP模型后应用、真实加密首装、多个G候选及专用恢复下载；无远端写入的准备/下载/恢复以FakeTransport计数证明，实际host/capture/LocalApplication/ModelTxn非桩。Adapter真实HTTPS循环服务属于独立network owner既有证据，此批未访问真实业务WebDAV。

后继追加的真实备份失败/同O重试与恢复导出测试按最终receipt记录，不与14项旧集合重复计数。

仍未关闭：模型Applied但manager/assets未完后跨进程重启，需要identity owner受控restore_adopted和ModelApply证明getter；不能deserialize私有JSON恢复Weak issued权限。同UUID既有目标尚无namespace mapping时的显式adopt入口也等owner已提交新API，旧freeze严格碰撞未在Service绕过。普通生产后续同内容no-op会在该同ID准入补齐后补专项；现已有同语义Service/core零PUT测试，不将它冒充尚未验的owner碰撞场景。

LocalFile归独立file_transfer_v2 owner，包含WebDAV connection/password候选及受控installer；RemoteSync明确排自身连接，不为接线删选择或常驻禁用旧文件功能。Service不复制FileOwner。
