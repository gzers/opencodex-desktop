# Desktop 所有权收缩：渠道、模型与客户端配置归还 OpenCodex

日期：2026-11-22。适用版本：0.2.24。代码：main 958029879（基线 cfb3c8aa7）。

## 决定

渠道、模型、协议、密钥和 OpenCodex 原生配置由 OpenCodex 自己管理。Desktop 只负责 Skill、扩展、独立同步、运行包、端口、更新、诊断和备份。

因此从 0.2.24 起：

- Desktop 删除自己的渠道编辑、新增、导入、保存、回退与模板管理能力，模型页只保留「在 OpenCodex 中管理」入口。
- WebDAV 与 CLI 同步只同步 Desktop 自己的偏好与扩展设置，不再携带渠道、模型或凭据。
- Desktop 不再创建 OpenCodex 原生配置锁，也不再写回 config.json；安装、修复、卸载前保留只读完整备份。

## 撤回理由

这两条链路各自都有问题，叠加在一起还会互相干扰。

一是重复实现。Desktop 自己在 manager-state 里维护一套渠道和模型对象，再投影到 OpenCodex 的配置文件，等于把 OpenCodex 已有的能力重做一次，却要和官方 2.99 的字段、刷新时机、运行键约束长期保持同步。这段时间出现的名称不生效、保存基准漂移、2.99 配置导入失败，都属于这条重复链路的维护成本。

二是触碰真实用户目录。此前为让 Desktop 的渠道写入能落地，加了整条 ~/.codex 搬移与崩溃恢复链：接管真实目录、保管原件、失败后恢复。这条链路本身需要完整的安全描述符、跨管理器 owner 交接和 supervisor 缺席证明，Mac 和 Windows 都没能在本版补齐，Windows 侧的首 Start 还在原预算内超时。它保护的是一个本不该由 Desktop 写的目录。

## 替代做法

启动 OpenCodex 时固定两个变量：

- OPENCODEX_HOME = Desktop 自管目录 ~/.codex-desktop/opencodex/home
- CODEX_HOME = OPENCODEX_HOME/codex-client

OpenCodex 2.99 在缺少 CODEX_HOME 时会回落到真实 ~/.codex，所以每个启动路径都显式传入，包括启停、状态、版本、doctor、shim、终端入口、卸载和 Windows 原生后端。真实 ~/.codex 全程只读。

端口设置从原生配置移到 manager-state/runtime-port-settings.json，自动分配由 Desktop 自己检查并保存，不再读写 config.json 的 port 字段。

## 影响范围

删除的代码：

- 命令层 model_config、model_config_v2、model_transfer、native_config_import、config_sync_v2、config_file_transfer_v2、scope_backup_v2、data_paths_v2、client_recovery。
- 基础设施层全部 macos_client_*、official_client_coordination、native_client_coordination、native_config_write_guard，以及 Windows 侧的 windows_client_originals024、windows_owner_host024、windows_supervisor024。
- 界面层 features/models 目录、渠道同步与文件传输组件、客户端恢复界面，以及只针对这些页面的浏览器审计与前端测试。

保留并调整的代码：

- modules/backup、modules/skills、extensions、sync、runtime、updates、diagnostics 保持不变。
- 新增 modules/model_config/native_config_backup.rs，只保留只读备份与升级前保护，不含任何写回路径。
- 普通启停恢复为已有认证的 Start / Stop；Windows 在正常零退出、监听与记录清理、程序未变都核实后允许后继启停。

## 验证

- cargo check、cargo clippy --locked --workspace --all-targets -- -D warnings、cargo fmt --check 通过。
- vue-tsc --noEmit、vitest run、vite build 通过；发布门禁快照 3 通过 / 0 失败。
- 隔离启动三次：数据根落在沙箱内，真实 ~/.codex 的 inode、mtime、权限前后一致。
- 未执行真实业务同步，未改日常配置，未发布。

## 遗留

本机没有完成官方 2.99 经产品路径的真实 Start / Status / Stop。沙箱启动按设计跳过集成运行包供应，因此隔离启动只能证明 HOME 隔离和不写原生配置，不能替代真实服务验证。真实官方 2.99 的启停与 Windows 原生生命周期仍需在具备条件的机器上补验。
