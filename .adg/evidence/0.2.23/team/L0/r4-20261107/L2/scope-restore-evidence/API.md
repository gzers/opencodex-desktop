# L2 v2 范围备份与恢复接口

模块 sync_local_v2::restore。公开函数全部接收宿主固定的规范化 D/H，无远端、旧 incoming/package/state、UI 文件路径或载荷字节权限。

- list(D,H) -> ModelResult<Vec<BackupStatusV2>>，条目为 id/enabled/ready/groups。
- preview(D,H,originalO,&[group]) -> ModelResult<RestorePreviewV2>，返回 restoreId/planDigest/groups。
- restore(D,H,restoreId,planDigest) -> ModelResult<RestoreReceiptV2>，返回 restoreId/originalOperationId/groups/status/completedGroups/refreshFileSnapshot/failureCode。

status 使用 applied/partial。只有全部实际完成时 refreshFileSnapshot=true；partial 为 false，界面保持原 O 和当前会话，不 refresh/ensure/begin 新草稿。成功后可触发 saved-only FileSnapshot 刷新建议。模块本身不 snapshot/begin/discard 任何草稿。只读 preview 不读取、采用或清理任何草稿，格式未知也不影响 saved-only 核图；实际 restore 触及模型组时在真实写锁内对任何草稿明确拒绝，不解析或猜测 clean。preview 自有计划持久化和 restore 均取得 global storage admission。

组保留 templates/channels/preferences/extensions/webdav/accounts/mcp/skill:name。未知、重复或空组明确拒绝。模型组使用原 typed images；对只选 templates 的组合先验证实际剩余 Policy 的引用、ledger、映射等依赖，不合法就拒绝，不扩大为 channels。manager/assets 使用原参与者的固定目标，允许不选模型的恢复保留后来模型编辑。所选目标有后来编辑则明确 stale；不做 JSON 三方覆盖。

## 生命周期与证明

固定目录 D/backups/transfer-scope-v2/<originalO>/record.json。backup=false 留 enabled=false 的元数据，不存 scope 私有图像。backup=true 在原全部 protect 与第二轮实际 participant verify、mapping guard 成功之后、首 model.apply_once 之前冻结 before，重试不替换。保留未完成 O 的原 before 和保护记录，ready=false，不自动恢复。历史无新 hook 的操作不臆造 before。

真实 model scope_backup_images 来自实际 ReceiveFileHost/Namespace/ServiceFactory/RemainingModel 的原保护 owner，before/after 均为其冻结 typed writes，剔除原 OperationReceipt。模型接收必须由真实回执确认；没有 owner 图像的测试模型 default None 只保留不可 ready 的备份。模型图像还与 model_transaction_v2::history 内带精确原 O receipt 的、已提交且根绑定的 typed 事务独立交叉核对；不猜最新事务，不自行解析模型 journal。

manager after 从原 applied_files 回执得出，MCP after 从 completed_mcp_bytes 得出，Skill 从 completed_skill_target 的实际 digest 核验后读取，未修改的第二目标仍保持 before。普通文件 after 直接来自原回执，生成 after 不读取今天的 H/config；目录另保原完成树 guard。全部实际参与者完成并写好 LocalProgress::Applied 后才可 ready。

非模型 before 的每个路径/字节/可执行位与 local.journal.targets 原树 hash 交叉校验，模型 before 与 typed 事务图像交叉校验。仅重算 scope record 自身 checksum 不能获得新的恢复权。UI 得到的只有上述脱敏 DTO。

## 写入、失败与幂等

preview 冻结原备份 digest、所选范围及当前目标树 guard，restore 重新核对来源和目标 CAS。首业务写前为所有所选当前目标重新保护并核验；任一失败不得开始恢复写。模型组调用专用 model_transaction_v2::commit_protected_restore，完整 typed metadata 使用同一事务，原来不存在的 OfficialConfig 或 allowlist Manager 文件恢复为真实 None，不用空 JSON 代替；普通 commit 禁删规则不变。

manager/assets 复用 transfer_backup 的纯 Image/image/write_image/owned_intermediate helpers，不读 v1 incoming/package/state。每个完成参与项持久化进度；单参与项失败时只补偿其可证明由 helper 写入的中间图像。跨参与项不是全局原子；返回 partial+真实 completedGroups/failureCode，原备份与新保护保留。同 restoreId 可以续做；已完全成功的同 restoreId 只返回原 receipt，不覆盖成功后新编辑。若 typed 引擎自身留下 pending marker，明确要求先由既有模型事务恢复机制完成该 pending，再续原范围恢复，不接管别的事务。

## 验证边界

所有测试使用隔离 tempfile D/H、实际 LocalApplication/ReceiveFileHost/typed transaction/portable_manager/portable_assets。故障仅在真实回执之后注入；没有 fakeApplied、真实 WebDAV、日常配置、远端清空或发布。源码提交 26484af3066c95e20b0c8116ad0d87f5f725e863；trait-only 接口提交 1552e1793550611307ef3295a973c406447705d0。精确命令和最终结果见 receipt.json 与原日志。
