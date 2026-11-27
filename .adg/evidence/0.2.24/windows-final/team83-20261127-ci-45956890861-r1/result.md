# TEAM83 Windows 普通账户 CI 首错

`accepted:false`。本轮检验两个判断：旧 Windows 分支在 PATH 夹具修正后能否通过普通账户套件，以及主线独立修正能否通过同一套件。

- 旧 Windows 分支源码 `134a83e8b02bd8ecf304356e86282c101adc25d4`，Actions `45956890861`，`backend-windows` 作业 `140365331558`。
- 主线对照 `37a141fbee8e357db3d42f2aa830c6be2c8be195`，Actions `45956736730`，`backend-windows` 作业 `140364838974`。
- 旧分支原始制品 `15181251979`（`windows-startup-evidence.zip`），本机复算 SHA-256 为 `a64640aec3b5876394b0d41f661ac7e5ef32bfa9a63e1c1a27250d23b39361b6`。下列原件由该 ZIP 按成员名提取并逐一校验字节数与 SHA-256。

| 项目 | 状态 | 证据 |
| --- | --- | --- |
| 旧分支普通账户后端 | fail | 51 个二进制均尝试，只有 `test-23`（`opencodex_desktop_lib`）由保留的进程句柄确认退出 101；库测试 1801 通过、1 失败、19 忽略，本轮未出现 PATH 断言失败。 |
| 旧分支 TokenUser 和独立凭据步骤 | pass | 两步均成功。独立凭据步骤选择 `provider_auth_native`、`channel_rename_native` 的忽略用例和 `native_secure_store_round_trip`，未选择失败的库用例。 |
| 主线普通账户后端、TokenUser | pass | 作业步骤均已成功。已对照两提交中的 `credentials.rs`、`windows_private_create.rs` 和 CI 路径，无差异；主线整步通过，尚未提取该用例的逐行回执。 |
| 旧分支后续构建与 EXE 验证 | blocked | 后端失败后，实际安装更新、MSVC Release 和最终 EXE 验证均跳过。 |
| 旧分支 CI 清理 | pass | 原回执确认临时账户和 Profile 已删除，输入与桌面 ACL 已恢复，源码字节未变。普通账户套件列出 64 条 `stopped-or-exited`。 |
| TEAM81 实机收口 | blocked | 截至本轮复核，主线 `a75d3b0af3e131e10b7ee8fcc2a7687378c7482a`；最近核到的打包 `45863614604` 仍用旧源码 `7580b0d79289864cb105e6d56d668b608b93495b`。尚未收到包含 Windows 端口修复的同源 MSI／NSIS。 |

首错 `infrastructure::credentials::tests::renamed_file_is_rejected_instead_of_serving_another_account`，`src\infrastructure\credentials.rs:600:47`，`store a: NotConfigured`。首次 `store_webdav_password` 已失败；尚未重命名文件，也未执行另一账户读取拒绝断言。其余四个 `credentials::tests` 用例在同一进程中通过。

共享回交请求：如主线再次遇到此首错，在现有普通账户用例中记录首次保存返回失败的位置和可用 OS 错误码，区分共享测试根、私有目录准备、权限设置、DPAPI 和原子写入。保存路径不调用显式根 `store_at` 的回读分支。该测试组共用一个根，损坏文件用例通过通用 `create_dir_all` 创建父目录；是否存在与首次保存的并发影响仍待验证。保持原断言、凭据保护和外部错误行为。诊断不记录口令、密文或日常配置。

原件见 [库 stdout](raw/test-23.stdout.log)、[库 stderr](raw/test-23.stderr.log)、[进程回执](raw/test-23.json)、[普通账户套件回执](raw/ordinary-suite.json)、[父回执](raw/rust-ordinary.json) 和 [SHA-256](raw/sha256.txt)。本轮仅提取 CI 证据，未启动本机候选或生成测试账户。
