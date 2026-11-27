# TEAM83 Windows 端口截止时间与普通账户终态

待证伪判断是 Start 的端口写锁争用可能超出原操作截止、发布设置或取得 Owner，以及旧分支的凭据首次保存错误可能在主线复现。

源码 `e663369226337c9a9cad594ccd3cafd1719c3711`，Apps 树 `fe77ddf82f5696620609d4463a99305b1b6b48b8`。[Actions 45965743318](https://github.com/gzers/opencodex-desktop/actions/runs/45965743318) 的 Windows 作业 `140393871165` 终态通过。[作业回执](job.json)

| 检查 | 状态 | 结果 |
|---|---|---|
| Windows Start 端口写锁争用 | pass | 普通账户执行 `start_port_lock_exhausts_action_window_without_publishing_settings`。750 毫秒夹具窗口返回 Timeout，满足 2 秒硬断言，设置未发布、HOME 为空、无 Owner。[原日志](raw/ordinary-library.stdout.log) |
| 共享端口设置保全及释放锁后重试 | pass | `automatic_port_wait_honors_start_deadline_and_keeps_original_settings` 在 250 毫秒夹具窗口返回 Timeout，满足 2 秒硬断言，原设置字节相同。释放后重新分配了不同端口，可回读相同 revision。[原日志](raw/ordinary-library.stdout.log) |
| 旧凭据首错回归 | pass | `webdav_and_encryption_use_different_files` 通过。该用例包含旧 `credentials.rs:600:47` 的 `store a`。此次未复现 [NotConfigured 首错](../team83-20261127-ci-45956890861-r1/result.md)，根因未证实。[原日志](raw/ordinary-library.stdout.log) |
| 普通账户 Rust 后端 | pass | 定向用例所在库二进制原退出 0，1806 通过、19 忽略。整个普通账户二进制范围退出 0，账户／Profile 删除，源或制品 ACL 与桌面 ACL 已复原。[库回执](raw/ordinary-library.json) [全二进制回执](raw/rust-ordinary.json) |
| 受管升级及离线重装 | pass | CI 夹具使用真实 npm 和版本探测，覆盖 2.90.0／2.91.0，日常配置未触碰，生成请求 0。固定 2.99 实机项仍待验证。[原回执](raw/managed-live.json) |
| 最终裸 EXE 的普通账户冒烟 | pass | 观察窗口存在且响应，两个记录均关联相同源码和 EXE 摘要。沙箱 HOME 均移除，两名 CI 临时账户及其 Profile 记录、注册表和目录均不存在。[步骤 18](raw/startup-first.json) [清理](raw/startup-first-cleanup.json) [步骤 24](raw/startup-ordinary.json) [清理](raw/startup-ordinary-cleanup.json) |
| 同源安装器及实机 | pending | Release Actions `45973489918` 已在同 SHA 启动 Mac、Windows 构建，采证时安装器仍未交付。完整包离线准备另缺可独立断网且保留交互控制的桌面。 |

本作业通过项无首错。裸 EXE 冒烟仅观测启动，原记录的环境门禁为 `integrated_runtime_not_ready`、准备阶段为 `idle`。[WebView 原观察](raw/webview-ordinary.json) 首次 Status 的请求到 UI 终态时序、超时未知与 Start 禁用，以及实际 Start → Status → Stop 均未由此冒烟验证。夹具窗口不改变产品 Start／Status／Stop 的 30／5／10 秒预算。

启动证据 ZIP `15186605240` 与 MSVC 裸 EXE ZIP `15185049734` 均按 GitHub 制品元数据核对源码、字节数及 SHA-256。本机复算 EXE 为 `2b40ed56b9fff9d3b58ea8c0ca3aecb6e805f86ad063410b3cd137e84b2b0e44`，与[制品清单](raw/candidate.json)及两轮冒烟一致。推送所选原始成员与[字节摘要](raw/sha256.txt)，两个原 ZIP 留在本机取证目录。本机未启动本批产品或创建测试账户。W2／双 SID 暂停，未发布，`accepted:false`。
