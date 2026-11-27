# TEAM83 Windows 主线独立冒烟证据

待证伪判断：主线独立 Windows 冒烟制品能证明其临时普通账户和 Profile 已清理。若任一账户、Profile 记录、注册表项或目录仍存在，本项不通过。

源码 `3a844aef364830604da49268d3c470a318ecb3fe`。Actions [45963586935](https://github.com/gzers/opencodex-desktop/actions/runs/45963586935)，独立作业 [140386871329](https://github.com/gzers/opencodex-desktop/actions/runs/45963586935/job/140386871329)，ZIP 制品 `15181926704`。本机逐字节复算 ZIP 为 11,481 字节，SHA-256 与 GitHub 账本一致。执行结果和清理见原始成员，[ZIP 和成员摘要](raw/sha256.txt)。

| 检查 | 状态 | 结果 |
|---|---|---|
| 进程令牌冒烟及源码绑定 | pass | [原始回执](raw/process-token.json) 的 `sourceCommit` 匹配主线。9 项检查通过，含真实子进程令牌与权限、镜像错配及退出进程的拒绝。 |
| 原生进程身份 | pass | [原始回执](raw/native-process-identity.json) 证实使用保留句柄的本机时间戳，身份代次错配被拒绝，原子进程退出。 |
| 普通账户 Profile 和清理 | pass | [Profile 回执](raw/ordinary-profile.json) 与[清理回执](raw/ordinary-cleanup.json) 属于同一临时 SID。账户、Profile 记录、注册表项及目录均不存在，清理错误为空。 |
| 重启交接夹具 | pass | [原始结果](raw/restart-handoff-result.json) 的 5 个受控父进程与等待器用例、40 个检查通过，未启动产品。 |
| 主线普通账户后端，包括旧分支失败的凭据用例 | pending | 同一运行中的独立作业 [140386871266](https://github.com/gzers/opencodex-desktop/actions/runs/45963586935/job/140386871266) 在本次取证时仍执行步骤 10。其测试原件、首错和清理需从该作业另行获取。 |
| 同源 MSI／NSIS 及本机隔离 HOME 验证 | blocked | [最新 Release](https://github.com/gzers/opencodex-desktop/actions/runs/45863614604) 仍为 `7580b0d79289864cb105e6d56d668b608b93495b`。无当前主线安装器可复算，未执行本机安装、下载或生命周期。 |

本次独立冒烟首错无。保留 [旧 Windows 分支首次保存凭据失败](../team83-20261127-ci-45956890861-r1/result.md) 及 [共享端口截止时间交接](../team83-20261127-port-deadline-review-r1/result.md)。本机只读校验证据，没有创建测试账户或 Profile。原 Start／Status／Stop 30／5／10 秒、未知拒绝及暂停 W2／双 SID 均保持，未发布，`accepted:false`。
