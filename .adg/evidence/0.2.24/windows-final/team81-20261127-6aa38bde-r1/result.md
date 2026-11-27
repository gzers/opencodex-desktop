# Windows 0.2.24 主线接入回执

2026-11-27，源码 `6aa38bde9f27a179ed8d9d97c3533d8b5bea607d`，CI `45949909762`，`accepted:false`。

核对两项判断：主线是否包含原生 Status 与 Stop 的截止时间修复，是否已有从该 SHA 产出的同源安装器。

| 项目 | 判定 | 证据 |
| --- | --- | --- |
| 原生截止时间提交接入 | pass | `7889ecc3586eb811cffc4e80ec1d6f97998caaae` 与 `97f100b32e03b7d8952933a4bdbeaf519e270907` 均为主线祖先。自后者到本次主线的差异仅涉及测试文件。 |
| 主线 CI | in progress | 前端、发布工具、独立 Windows 冒烟及 Linux 后端已通过。Windows 严格 Clippy 已通过，后端回归继续运行。 |
| 同源安装器 | blocked | 本次查询该 SHA 只返还 CI `45949909762`。等候 Mac／Windows 同源制品及 MSI／NSIS 摘要。 |
| Windows 2.99 安装失败恢复 | blocked | 新驱动为 0.2.24 裸 EXE 验证依赖缺失时的拒绝和基线保全。已供应 2.99 的失败恢复仍等候可重放夹具及有效停止证明。 |
| Windows 单用户实机 | blocked | 等候双平台同源制品、可独立断网的 Windows 桌面、可达的固定下载资源及首次 GUI Status 和空闲证明。 |

旧 CI 首错为 `Unsupported native model UI acceptance version`。[旧日志与现场](../team81-20261127-7580b0d-r1/result.md)保留。本轮仅复核 Git 和 CI，未启动产品、下载运行时或创建测试现场。[主线作业和制品回执](raw/main-ci-receipt.json)

独立下载并复算 Windows 冒烟 ZIP `15176338274`，共 11,484 字节，SHA-256 为 `049ca349066a13b509d6c922e2f38b300d141bf8900e4182edf96c584e49c2f9`，与 Actions 公布的摘要一致。[ZIP 与 21 个成员摘要](raw/smoke-15176338274-verification.json)

冒烟回执证实进程代际不符会被拒绝、原子进程已退出，临时普通账户及 Profile 已删除。重启移交用例全部通过，夹具清理无错误，产品未启动。[Actions 原始日志摘录](raw/windows-smoke-140343111783.log)

06:38 UTC，主线 Linux 后端作业结束并通过。元数据进程树拒绝用例单独通过，后续主库回归 2,180 项通过、29 项忽略。Windows 后端已通过严格 Clippy，仍在执行回归。[Linux 终态回执](raw/backend-140343112025-terminal.json) · [Actions 原始日志摘录](raw/backend-140343112025.log) · [Windows 步骤回执](raw/backend-windows-140343112026-progress.json)

06:55:04 UTC，主线 Windows 作业的后端回归失败，后续 TokenUser 检查仍在运行。查询 Actions 步骤日志返回 `404 BlobNotFound`，检查注释为空。现只能确认失败步骤，首个测试断言与归属等待原始日志。[作业步骤回执](raw/backend-windows-140343112026-regression-failure.json) · [日志接口回执](raw/backend-windows-140343112026-logs-unavailable.txt)

## 主线 Windows 作业终态

CI `45949909762` 的 Windows 后端作业 `140343112026` 最终失败。步骤 10 的普通账户测试执行了全部 51 个二进制，其中 50 个返回 0。子项 24 `opencodex_desktop_lib` 返回 101，1,801 项通过、1 项失败、19 项忽略。首错为 `windows_cli024.rs:563` 的 `assertion failed: f.key().read().is_err()`，发生在私有注册表夹具的畸形 PATH 值测试。TokenUser、原生凭据和证据上传通过，Release EXE 与后续验证跳过。候选安装器仍缺席。普通账户、Profile 均已删除，无该 SID 残留进程，源和桌面 ACL 均恢复。

Actions 启动证据 ZIP `15178672325` 已独立复算，620,356 字节，SHA-256 为 `047275f03bf6387351343197dfe3c0954915d20be60ee15fa3b25cd87d064328`。归档保留 [Windows 作业终态](raw/backend-windows-140343112026-terminal.json)、[wrapper 首错摘录](raw/backend-windows-140343112026-terminal.log)、[普通账户总回执](raw/ordinary-main/rust-ordinary.json)、[套件回执](raw/ordinary-main/ordinary-suite.json)及[子项 24 原始输出](raw/ordinary-main/test-24.stdout.log)。

判断是畸形字符串夹具的写入不稳定，验证方法为在新的私有注册表键写入相同的未终止 UTF-16 字符，仅改变声明长度以外的缓冲区。本机 Win32 探针读回尾部为零时存储 4 字节，非零时存储原 2 字节，复核后已删除该私有键。CI 首错没有记录失败输入的类型或长度，探针证明了可触发首错的机制，未重构原 CI 进程的内存布局。正式 PATH 和日常配置未写入。
