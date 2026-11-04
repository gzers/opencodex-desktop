# AC02 改名生产接线

工作根 ocx-0219-fixes / codex/0.2.21-fixes；实现直接写在原生产文件，未以 G0 spike 代替。新增 channel_rename.rs 通过 config_file.rs 的 path module 注册，不需要修改共享 mod.rs。

## 主线接口

入口：config_file::save_with_context(root: &Path, context: &ProcessContext, request: SaveRequest) → ModelResult<SaveResult>。所有已有字段和返回 DTO 保持；在真实保存事务内更新 Agent providerId / connectionDigest。commands/model_config.rs 的 model_file_save 可在 spawn_blocking 前克隆 context，改用该入口。此 command 不属于本 worker 的独占写集，本 worker 未修改。

原 save(root, home, request)、save_observed、save_observed_checked 也已支持改名与持久化恢复。没有 ProcessContext 的调用路径更新 providerId / channelId，并将旧 connectionDigest 清空；现有 execution_directory::bind 在实际运行身份下重建，避免继续持有旧身份摘要。

普通保存的 callback 仍保留；改名分支在 callback 前准备完整事务，在 callback 后重验所有输入，并一次提交 raw/layout/preferences/既存 source-ids 与安全账户。保存时间事实由主线按 syncworker 接口接入 config_file，不在此模块自造时间方案。

features/models/channelRename.ts 导出 channelRenameError / channelRenamePreview / 名称与标识编辑意图工具。ChannelEditor 已使用，当前渠道保存会就地阻止非法／冲突目标；全页后端同样执行完整校验。ChannelsView 若需在全页预览中提前显示同一错误，可复用 channelRenameError，不必复制校验。原保存成功重启提示由既有消费者承接。

## 事务与恢复

- 同一 channel.id 原对象改 provider 键，不删除重建渠道；当前默认、原顺序与禁用引用的完整 provider 段、当前清单观察、布局、Agent 偏好和既存 source-ids 同时更新。未知 provider/root 字段保持，型号、节点、模版／采用身份保持。
- provider-owned keychain 引用与 Key 池迁移前验证归属；新账户先创建读回，文件和目录消费者读回成功后持久化 committed 决定，再清理旧账户。Key 内容、池 slot.id／顺序／label／addedAt／活动选择保持。显式 replace/clear 仍按原编辑意图处理。
- 事务入口使用 manager-state/provider-rename-pending.json.lock，读快照／普通保存／恢复均先取该锁。文件顺序为 config→layout，再 preferences 既有事务锁、source-ids、protocol mapping。协议分组按稳定 channelId／协议映射保持其自身身份，不按新名称重建。
- 精确 before/after 图像位于 backups/model-config/rename-<UUID>/，目录 0700、文件 0600；pending 仅记录目标枚举、摘要、账户归属与提交决定，不记录 Key 值。
- snapshot/save 首先恢复 pending：未提交仅回滚仍属于该事务的字节，全部文件恢复后才移除暂存账户；committed 只前向补齐旧账户清理。外部新写入不覆盖，返回 rename_recovery_required；提交后清理失败返回 rename_cleanup_pending。两类状态保留日志，不伪装成功。
- 没有旧 provider/alias 兼容，也没有历史会话迁移；便携传输仍使用稳定 channel.id，未新增源 providerId 跨设备复制。

## 当前验证与交接

首轮生产入口隔离测试已实际通过；最终日志和汇总在本目录 logs/ 与 result.json。测试只使用 tempfile、子进程内存 credential builder；不访问日常配置／钥匙串，不真实联网，不启动或重启日常客户端。

主线仍负责：上述 command context 接线、保存时间事件钩子、ChannelsView／保存提示消费者、隔离客户端重启后的实际目录路由及双平台原生验收。不能把本分支通过的 Rust/DOM 测试写成这几项已经完成。本 worker 未提交或推送。
