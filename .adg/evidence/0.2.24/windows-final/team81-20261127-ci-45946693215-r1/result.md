# Windows 0.2.24 Stop 截止时间提交的 CI 回执

源码 `97f100b32e03b7d8952933a4bdbeaf519e270907`，Actions `45946693215`，`accepted:false`。

待证伪的判断：同一原截止时间在 Windows 停止路径生效后，严格检查、原生回归和候选启动观察仍能通过。Windows 后端终态待回读。

| 作业或制品 | 状态 | 范围 |
| --- | --- | --- |
| windows-smoke-evidence `140332897457` | pass | 进程移交 40 项、隔离检查和安装器矩阵脚本契约通过。产品未启动。 |
| frontend `140332897839` | pass | 142 个测试文件、913 项通过。 |
| release-tools `140332897835` | pass | 发布工具检查成功。 |
| backend（Linux）`140332897727` | pass | 前置查询和元数据进程树断言通过。主库 2,180 项通过、29 项忽略、1 项单独执行。 |
| backend-windows `140332897923` | in progress | 严格 Clippy、后端回归、TokenUser、隔离凭据及托管安装升级与离线重装均已通过。 |
| Windows 冒烟制品 `15174488719` | pass | Actions 上传日志和制品元数据给出同一 ZIP 摘要 `cd795d5abefcad46f40e65209f662784c27c703d5ae5959a84f93fa5f9c1565e`。范围为冒烟取证。 |

05:48 UTC 复核 `origin/main` 和 Mac 集成分支，两条原生修复均无等价补丁。正式 MSI／NSIS 及真实桌面验收待同源候选。此轮只读取 CI 与 Git，未启动产品、改动 Windows 本机策略或清理既有场景。

- [Windows 冒烟原始日志摘录](raw/windows-smoke-140332897457.log)
- [前端原始日志摘录](raw/frontend-140332897839.log)

下载并独立复算冒烟 ZIP 字节数及摘要，核对四份回执。进程代际变化被拒绝，原进程已退出。重启移交夹具回执为 `productStarted:false`，无夹具清理错误。[制品复算和成员摘要](raw/smoke-15174488719-verification.json)

## 后续回读

Windows 严格 Clippy 通过，后端回归继续运行，终态和日志待回读。[作业步骤回执](raw/jobs-progress.json)

Linux 后端作业已成功结束。[原始日志摘录](raw/backend-140332897727.log) · [作业回执](raw/backend-140332897727.json)

06:23:06 UTC，Windows 后端回归通过。TokenUser 检查已开始，后续制品及真实验收仍待回读。[Windows 作业步骤原始回执](raw/backend-windows-140332897923-progress.json)

06:26:29 UTC，Windows 真实 TokenUser 所有权检查通过，隔离原生凭据步骤正在执行。Windows 作业仍在运行，候选安装及实机结果沿前置回执。[新增 Actions 步骤回执](raw/backend-windows-140332897923-token-user.json)

06:34:28 UTC，隔离原生凭据保护通过。Windows 作业已进入官方托管安装升级与离线重装检查，终态待回读。[Actions 步骤回执](raw/backend-windows-140332897923-native-credentials.json)

06:42:37 UTC，官方托管安装升级与离线重装 CI 用例通过。后续依赖诊断和 MSVC Release EXE 构建待回读。完整包首次离线准备继续等候同源安装器和可独立断网的实机桌面。[Actions 步骤回执](raw/backend-windows-140332897923-managed-reinstall.json)

## Windows 作业终态

Actions 作业 `140332897923` 最终失败。以下结果均绑定源码 `97f100b32e03b7d8952933a4bdbeaf519e270907`。

| 步骤 | 结果 | 证据 |
| --- | --- | --- |
| 7–13 格式、严格 Clippy、普通账户后端、TokenUser、隔离凭据、托管升级与离线重装 | pass | Actions 步骤回执。步骤 9 为专用诊断，按条件跳过。 |
| 15、18 MSVC Release EXE 构建及最终 EXE 冒烟 | pass | 制品 `15178143126` 的 ZIP 和 EXE 摘要已独立复算，EXE 12,013,056 字节，SHA-256 `6805154f9d966ae65ef2f12b3da86f4753ec929b5aa868d219f7636cc4f08e4a`。 |
| 20 Windows 原生安装失败及恢复 | fail | 2026-11-27 06:57 UTC，旧 UI 驱动在 `windows-model-ui.mjs:34` 断言 `Unsupported native model UI acceptance version`，未进入安装恢复检查。 |
| 22、24 Node 22.20 回归及普通账户最终 EXE | blocked | 步骤 20 失败后跳过。 |
| 25 失败证据上传 | pass | 启动证据 `15178970199`，1,181,360 字节，独立复算 ZIP SHA-256 `3f98e29fda394ae5d7290c60c55940d9eaa30f7e68fcae6824059c4fd919d3b3`。 |

新旧截止时间分支均在同一旧驱动断言失败。主线 `6aa38bde9` 已为 0.2.24 裸 EXE 单列固定依赖缺失拒绝和基线保全；固定 2.99 实包恢复仍需既有的停止证明及失败夹具。主线后端回归的失败是另一个步骤，待原始证据定位。仅复算 CI 制品并读回证据。本机未启动候选、未新建实机夹具。未取得 MSI／NSIS，`accepted:false`。

- [Actions 终态](raw/backend-windows-140332897923-final.json) · [作业日志摘录](raw/backend-windows-140332897923-first-failure.log)
- [EXE 制品复算](raw/candidate-15178143126-verification.json) · [启动证据复算](raw/startup-15178970199-verification.json)
- [原始断言日志](raw/windows-native-018-execution.log) · [启动回执](raw/windows-native-018-launcher.json) · [安装恢复回执](raw/windows-native-018-native-install-recovery.json)
