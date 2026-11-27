# TEAM83 Windows 主线普通账户回归终态

待证伪判断为[旧 Windows 分支凭据首次保存失败](../team83-20261127-ci-45956890861-r1/result.md)在主线复现。此轮同一用例通过，无法据此确定旧首错的根因。

源码 `37a141fbee8e357db3d42f2aa830c6be2c8be195`，Actions [45956736730](https://github.com/gzers/opencodex-desktop/actions/runs/45956736730)，Windows 后端作业 [140364838974](https://github.com/gzers/opencodex-desktop/actions/runs/45956736730/job/140364838974) 终态通过。`windows-startup-evidence` ZIP `15182792716` 共 2,830,044 字节，本机复算 SHA-256 与 GitHub 账本一致。[ZIP 及成员摘要](raw/sha256.txt)

| 检查 | 状态 | 结果 |
|---|---|---|
| 旧凭据用例 | pass | 以测试名称匹配主线日志，`renamed_file_is_rejected_instead_of_serving_another_account` 在 `test-22` 通过；主线与旧失败分支的凭据源码相同。 [原日志](raw/test-22.stdout.log) · [普通令牌与退出码](raw/test-22.json) |
| 普通账户后端 | pass | 51 个二进制均尝试，原退出 0；库套件 1802 通过、0 失败、19 忽略。[子套件回执](raw/ordinary-suite.json) |
| Release EXE 与普通账户启动 | pass | MSVC 构建及两个最终 EXE 冒烟步骤通过，普通账户产品未提升。[普通账户回执](raw/ordinary-final-exe-summary.json) |
| Node 26／22 裸 EXE 运行准备 | pass | 两种驱动使用 0.2.24 裸 EXE，缺随包描述时拒绝且保全选择和配置。[Node 26](raw/unbundled-node26.json) · [Node 22](raw/unbundled-node22.json) |
| CI 临时账户与 Profile 清理 | pass | 后端临时 SID 无残余进程，账户与 Profile 已移除，ACL 已还原。最终 EXE 的独立临时账户、Profile 记录、注册表和目录均不存在。[后端清理](raw/rust-ordinary.json) · [最终 EXE 清理](raw/ordinary-final-exe-cleanup.json) |
| 最新主线端口争用原断言 | pending | `main@e66336922` 的相关作业仍执行普通账户后端。 |
| 同源 MSI／NSIS 与本机验收 | blocked | 本作业使用裸 EXE，未验收已安装产品的供应、首次 GUI Status、真实启停与固定 2.99 安装恢复。 |

本作业首错无。原失败及 `store a: NotConfigured` 保留。此轮只读取 CI 原件，本机未运行安装或新建账户，原 30／5／10 秒与未知拒绝保持。W2／双 SID 暂停，未发布，`accepted:false`。
