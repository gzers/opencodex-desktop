# L2 本批接口回交

本批实现提交：6e4eedac64a168f0363531ddc625494def0f8001、7f39cf5f9b5d2ca3c3db23aaeac2f615e66db0b3。仅修改 sync_service_v2.rs 与 sync_service_v2_tests.rs。共同基线包含 c688df2ae57af5652585efd4c02d66d20e5e6a8b 和实际接收修复 202e449aa54249a5983f17740e9bec20ee5182d7。

## 无 endpoint 的本机文件目录

ProductionServiceBackendV2::new(D, H, credentials) -> ModelResult<Self>；backend.local_file_catalog() -> ModelResult<LocalFileCatalogV2>。

返回 saved 来源的 entries、mcpSources、webdavConnections；连接公开 id/label/url/remotePath/username/passwordAvailable。复用实际 inventory_request 与 LocalFile capture，保留实际本机 UUID、模型/族、库版本和默认引用以及逐来源 MCP digest。不依赖活动 WebDAV，不调用网络。WebDAV 候选复用 portable_manager::export(webdav=true,passwords=false)；口令 availability 只核对 D 下精确私有文件元数据，不读口令值、不返回 ref/account-key 权限。当前接口只有 saved 来源；draft 不能据此宣称已实现。

## 同 UUID 采用

生产 Download preview 使用实际 owner 的 freeze_preview_adopting_same_ids。只有精确同 UUID 可占用采用，不按名称猜测其它 UUID。原 strict freeze_preview 保持。实际生产 Service 已验证第二次同步语义 no-op、PUT 不增加；外部修改 raw context window 后，明确 Download Apply 恢复原冻结值。

## 原 DownloadPlan 剩余接收

SyncServiceV2::resume_receive(operation_id: String, read_passphrase: Option<Zeroizing<String>>) -> SyncResult<OperationFact>。

读取原操作私有记录与冻结 context；H 来自生产 backend.configured_home，不能信任序列化路径。重新建立原实际 receiver，取得 LocalApplicationV2::local_model_journal_proof 与 receiver.applied_mapping_proof 两份实际 owner 类型证明，调用 restore_adopted，再以 new_mapped_remaining 完成原 prefs/Skill/MCP。保留原 operation、冻结字节、scope、target、备份选项。禁止新 model prepare/protect/apply、当前配置重捕获、网络或新映射。成功外层事实记 Applied，关系保持 Unknown，不宣称当前配置/远端已同步。

L0 的公共命令需薄注册本入口，或按原 result.kind 区分 Upload 与 Download；upload resume_original/preflight 不自动处理 Download。本批未实现 RecoveryPlan 专用 remaining 恢复，不将 DownloadPlan 结果扩大为全部恢复入口。

## 验证口径

6 个新定向案例通过：LocalFile 3、auth-none 1、同 UUID 1、实际双证明重启 1。普通应用 crate 的离线严格 Clippy 通过。源码当前含 23 个 Service 测试；本批不声称聚合重跑全部 23 个。全部使用隔离 tempfile D/H、实际生产 owner、假网络和合成凭据；未执行真实同步、远端清空或发布。

这些实证替代此前 service-evidence 中同 UUID 与模型 Applied 后重启尚未闭合的限制；历史原回执不改写。公共注册是否已在后继集成完成，以 L0 的实际接线回执为准。
