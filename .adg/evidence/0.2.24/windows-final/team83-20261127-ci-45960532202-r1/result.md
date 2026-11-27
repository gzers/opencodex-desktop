# TEAM83 Windows 主线受管升级及离线重装回执

待核验判断为 Windows 主线接入已有 PID 记录的拒绝修复后，普通账户后端或受管升级回归失败。Actions [45960532202](https://github.com/gzers/opencodex-desktop/actions/runs/45960532202) 的 Windows 作业 `140377030882` 终态通过，源码 `a75d3b0af3e131e10b7ee8fcc2a7687378c7482a`。实际保留的 CI 制品是裸 EXE 和启动证据。

| 检查 | 状态 | 结果 |
|---|---|---|
| 受管升级及离线重装 | pass | CI 使用真实 npm、官方 registry 和版本探测，覆盖 2.90.0／2.91.0，验证来源记录可回读，记录日常配置未触碰、生成请求为 0。[原回执](raw/managed-live.json) |
| 普通账户后端 | pass | 所有测试二进制均有尝试，原退出 0，临时账户和 Profile 已移除，文件权限已还原。[原回执](raw/rust-ordinary.json) |
| 普通账户 EXE 清理 | pass | 最终 EXE 冒烟通过，独立临时账户、Profile 记录、注册表和目录均不存在。[原回执](raw/ordinary-cleanup.json) |
| 固定 2.99 安装恢复及同源 MSI／NSIS 实机验收 | blocked | 等待同源安装器及既有固定 2.99 实机夹具交接。 |
| 端口争用截止时间 | pending | 后续主线 `e66336922` 作业独立取证。 |

本作业无首错。2.90.0／2.91.0 回执范围仅限 CI 夹具。制品 ZIP 已复算 2,903,272 字节的 SHA-256 并与 GitHub 账本比对，原样保留三个成员。[摘要](raw/sha256.txt) 本机仅取证，未运行产品、未创建账户。原始预算和未知拒绝保持，W2／双 SID 暂停，未发布，`accepted:false`。
