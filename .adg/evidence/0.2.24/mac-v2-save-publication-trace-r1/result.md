# Mac V2 渠道保存、官方发布与回退调用链复核

2026-11-21。Desktop 源码 `b33369498525854f0c9f7e335f11b3261cc6af41`。原样官方 OpenCodex 2.99.0 Darwin ARM64 包摘要 `5b10c1a888000d54d14783622d9d8330c33f4e1b14a5f163d007337d0424ec7f`。本轮为源码复核，未启动服务、调用 API 或重启客户端。

| 环节 | 已确认的行为 | 尚需实证 |
| --- | --- | --- |
| Desktop V2 保存 | `ChannelsView.vue:25,27` 调用 `store-v2.ts:214`，进入 `commands/model_config_v2.rs:411`。`config_file_v2.rs:1083,1345` 提交官方 JSON、管理器对象及凭据事务，并读回文件快照。保存命令内无运行服务绑定或目录发布确认。 | 从运行中的 GUI 走真实保存，核对磁盘代次与官方 live 输出。 |
| 用户确认重启 | `TaskProgressHost.vue:75` 显示「已写入」。用户确认「立即重启 Codex」后进入 `cache_refresh.rs:370`。旧改名准备依赖 `rename_effects`，生产记录来自旧 `channel_rename.rs:324–327`，V2 保存未生成该记录。无记录时跳过 `saved_runtime_reconcile`，转入 `cache_refresh.rs:593` 的官方重启路由。仅重建缓存时单独执行 `ocx sync-cache`。 | 核对 V2 保存对应的官方目录代次，并证明发布失败可安全重试。 |
| 官方 2.99 发布入口 | `provider-routes.ts:945–1024` 的 `POST /api/providers/reload` 要求专用 capability，重验全量磁盘快照后激活目标渠道并请求目录收敛。`provider-routes.ts:1027–1130` 的 `PUT /api/providers` 校验公开 DTO 的 `baseline/next`。两者一致时，成功路径也会接管 live 配置并收敛目录。旧 Desktop reconciliation 调用两入口并检查目录收敛。 | 在隔离真实服务验证 Desktop 绑定、认证、DTO、并发拒绝、旧 key 移除与目录读回。 |
| 整文件回退 | `commands/native_config_import.rs:286–351` 验证独立票据、官方包身份及确定停止。`native_config_import_v2.rs:554–602` 先备份并核验，再重验停止、来源与端口版本，提交配置和 Desktop 状态并读回。 | 分别验真实停止态回退、下一次启动消费并发布恢复配置、客户端再次读回。 |
| ChatGPT 内 Codex | IMP235 已在私有 HOME 的三个独立随包 app-server 观察到自动标签 A → B → A，slug 和人工名称保持。`test/smoke/macos024-display-refresh.py` 覆盖 Rust V2 落盘与原样官方目录组件。 | 固定候选上的真实 Electron 隔离与选择器，保留当前 ChatGPT 会话及日常配置。 |

官方 `src/server/management/provider-routes.ts` SHA256 `01ce02616e8cf680e7f9a2712e29c19230cb0b61675ba9a043e194150f7aa4e7`。公开 DTO 的定义与解析见 `src/server/auth-cors.ts:1307–1352`，文件 SHA256 `cbe1e432b359b39799b15214d5f6e5f3ace06b9ab280d80f01d0c0d7aefca664`。

复核未产生运行夹具。整版 `accepted:false`，TEAM79 待命规则继续有效。
