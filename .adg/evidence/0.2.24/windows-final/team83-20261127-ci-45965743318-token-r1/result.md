# TEAM83 Windows 端口期限接入与独立冒烟

待证伪判断为 Windows Start 的端口争用可能超出原操作期限并写入配置，以及旧分支的普通账户凭据首次保存失败可能在主线复现。源码检查和独立冒烟已有结果。两项后端断言仍待原作业回执。

源码 `e663369226337c9a9cad594ccd3cafd1719c3711`。Actions [45965743318](https://github.com/gzers/opencodex-desktop/actions/runs/45965743318)。独立冒烟作业 `140393871161` 通过。ZIP `15183455031` 共 11,477 字节，本机复算与 GitHub 摘要一致。[ZIP 与原始成员摘要](raw/sha256.txt)

| 检查 | 状态 | 结果 |
|---|---|---|
| Windows Start 端口期限接入 | pass（源码） | Owner 和运行记录核验后，以 `window.deadline()` 准备端口。共享锁取原操作截止与锁上限中较早者，原子替换前复核截止时间。 |
| 端口争用与普通账户凭据回归 | pending | 同次 Actions 后端 Windows 作业 `140393871165` 步骤 10 仍在运行。源码已有期限拒绝、原配置保全及释放后的分配用例。本轮未执行本地锁测试。 |
| 原生身份和交接冒烟 | pass | [令牌](raw/process-token.json) 绑定本次主线 SHA。[身份](raw/native-process-identity.json) 验证过期代次拒绝及原子进程退出。[重启交接](raw/restart-handoff-result.json) 5 个夹具共 40 项检查通过，[文件保留夹具](raw/retention-checks.json) 5 项通过。 |
| 独立冒烟的临时普通账户和 Profile | pass | [Profile](raw/ordinary-profile.json) 与[清理回执](raw/ordinary-cleanup.json) 的 SID 一致。账户、Profile 记录、注册表和目录均不存在，无清理错误。 |
| 同源安装器与单用户实机 | blocked | Release 仍为旧候选，当前 CI 制品只有独立冒烟和前端证据。缺同源 MSI／NSIS，另缺可独立断网且保留交互控制的桌面。 |

独立冒烟首错无。[旧凭据首错](../team83-20261127-ci-45956890861-r1/result.md)与[端口期限请求及本地拒绝](../team83-20261127-port-deadline-review-r1/result.md)保留。本机仅下载、核验和归档原件，未创建测试账户、Profile 或运行夹具。单用户 Start／Status／Stop 保持 30／5／10 秒和未知拒绝。W2／双 SID 及发布暂停，`accepted:false`。
