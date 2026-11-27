# Windows 0.2.24 Stop 截止时间提交的 CI 回执

源码 `97f100b32e03b7d8952933a4bdbeaf519e270907`，Actions `45946693215`，`accepted:false`。

待证伪的判断：同一原截止时间在 Windows 停止路径生效后，严格检查、原生回归和候选启动观察仍能通过。Windows 后端终态待回读。

| 作业或制品 | 状态 | 范围 |
| --- | --- | --- |
| windows-smoke-evidence `140332897457` | pass | 进程移交 40 项、隔离检查和安装器矩阵脚本契约通过。产品未启动。 |
| frontend `140332897839` | pass | 142 个测试文件、913 项通过。 |
| release-tools `140332897835` | pass | 发布工具检查成功。 |
| backend `140332897727` | in progress | 运行其余工作区回归。 |
| backend-windows `140332897923` | in progress | 严格 Clippy 已通过，运行 Windows 后端回归。 |
| Windows 冒烟制品 `15174488719` | pass | Actions 上传日志和制品元数据给出同一 ZIP 摘要 `cd795d5abefcad46f40e65209f662784c27c703d5ae5959a84f93fa5f9c1565e`。范围为冒烟取证。 |

05:48 UTC 复核 `origin/main` 和 Mac 集成分支，两条原生修复均无等价补丁。正式 MSI／NSIS 及真实桌面验收待同源候选。此轮只读取 CI 与 Git，未启动产品、改动 Windows 本机策略或清理既有场景。

- [Windows 冒烟原始日志摘录](raw/windows-smoke-140332897457.log)
- [前端原始日志摘录](raw/frontend-140332897839.log)
