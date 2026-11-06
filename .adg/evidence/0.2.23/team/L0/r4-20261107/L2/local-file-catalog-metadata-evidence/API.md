# LocalFile catalog 账户与凭据可用性

提交 193fa41abfabeb163d4845a59f6fae8101ebed4f；父提交 7f39cf5f9b5d2ca3c3db23aaeac2f615e66db0b3。只修改 sync_service_v2.rs 和 sync_service_v2_tests.rs。

LocalFileCatalogV2.accounts 为 Vec<portable_accounts::CatalogEntry>，直接复用真实 catalog。JSON 为 {account:{id,provider,subject,label,email,origin?},loginTransfer,localSource,informationSupported}。origin 包含 version/accountPlatform/owner/authKind/sourceNamespace/sourceAccountId/tenant。informationSupported 是现有信息导出规则；旧来源缺 origin 保持缺失，不按名称猜测。不可用来源仍列目录供界面展示，entries 仅保留原有可导出范围。没有 token、credentialRef 或账户密钥权限。

ObjectEntryV2 增 keyAvailable / authorizationAvailable，旧 JSON 缺字段默认 false。分别来自本机现有 key/auth 集合、已验证云端分区，并在合并目录时分别并集；secretAvailable 保留。MCP/Skill 用现有公开 catalog 元数据补充 secretAvailable，不读取未选凭据值。所有 Rust 字面量都显式补两字段，不加全局 allow。

本批 3 个 LocalFile catalog、2 个 DTO 定向测试通过；正常应用 crate 离线 strict Clippy --lib --tests --no-deps -- -D warnings 通过，rustfmt 和 diff check 通过。NeverCredentials 三个方法仍全 panic，测试通过。没有重跑长耗时 Service 集，没有新增 scopeprefs 模块、FileOwner 或主线公共入口。

本线临时 target 已在进程退出后删除；原始 /tmp 日志与此归档逐字节一致。共享缓存及其他贡献线目录保留。未执行真实同步、远端清空或发布。
