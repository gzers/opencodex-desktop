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
