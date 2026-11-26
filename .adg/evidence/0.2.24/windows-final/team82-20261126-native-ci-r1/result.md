# TEAM82 Windows 原生 CI 修复与共享交接

2026-11-26。基线 main `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，治理 `633f0d086438592aeacc2ce5ae8757b1c0fa3d9b`。Windows 源码 `a288fc41457ffa682edb3b3df2e00500a61a5649` 已推送 `feature/024-windows-native-ci`，供主线集成。

待证伪判断是 Windows 独占修改可以移除原生严格检查告警，同时保持 Owner 串行化、原期限、未知拒绝、原进程与监听证据。首次供应、关联 UI 和候选打包由 TEAM82 指定 Mac 独占；本提交只修改 Windows 原生文件和 Windows 专用测试。

| 项目 | 结果 | 证据 |
| --- | --- | --- |
| Windows 原生格式与差异检查 | pass | Rust 1.100.0 MSVC，格式检查与 Git 差异检查均返回 0。 |
| 前端构建 | pass | 使用 Node 24.22.3 重新构建。[版本](raw/ui-node.log) · [构建日志](raw/ui-build-node24.log) |
| Windows 独立原生测试 | pass | MSVC 22 项通过。包括进程成员与命令行身份、监听代次、未知超时、独立强制清理及隔离 PATH 测试。[原始输出](raw/native-tests-msvc.log) |
| 产品定向回归 | pass（5 项通过，3 项真实运行测试按原设置跳过） | [原始输出](raw/desktop-focused.log) |
| Windows 严格 Clippy | fail | 本地 MSVC 严格 `--workspace --all-targets` exit 101。原生文件告警消失，剩余诊断均为共享文件。首错 `program_image.rs:540` 使用已弃用的 `Atomic::fetch_update`。[原始输出](raw/clippy-msvc.log) |
| 代码分支 CI | Windows smoke 通过。Windows backend 的严格 Clippy 步骤已失败，后续独立检查在读取时仍运行。首次 CI 快照中的 backend 与 release-tools 已失败 | [Windows 作业](https://github.com/gzers/opencodex-desktop/actions/runs/45774644978/job/139755250749) · [首次读取](raw/ci-jobs.json) · [Windows 步骤读回](raw/ci-backend-windows-job.json) |
| TEAM81 完整包断网准备 | blocked | 缺少可独立断网且保持可操作的 Windows 桌面，未重复旧候选。[前置读回](../team81-20261126-resume-r1/result.md) |
| TEAM81 缺包确认下载 | fail（旧候选） | 已过暂缓、重开、主动确认 UI 和未就绪禁用 Start。确认后的原首错为 `runtime.rs:571 managed_dependency_missing`，早于下载请求。确认前 WebView 443 流量用途未知。[原始实机证据](../team81-20261125-r1/result.md) |
| TEAM81 首次 Status、未知降级与 30／5／10 秒启停 | blocked | 待新候选越过环境准备后，从界面真实发出请求计时；5 秒无法确认时需要重试、诊断和 Start 禁用证据。只在明确空闲后执行启停。 |
| W2／真实双 SID | blocked | 两端尚未从同一当前 main SHA 冻结；历史普通用户 MSI 首错 1625，注册表未显式配置仍不能证明前提已满足。[历史 W2 回执](../team79-20261124-r1/sweep/summary.json) |

请主线集成线明确剩余共享 CI 写集的归属并接入 Windows 源码。涉及 `modules/runtime/program_image.rs` 的弃用 API 和在 Windows 上无调用的清单函数，`modules/runtime/program_removal.rs` 的测试辅助方法，以及 `commands/runtime_recovery_v2_owner.rs` 的未使用 `proof_deadline`。处理时保持原证明期限和断言。共享首次供应修复完成后，请回交新候选主线 SHA、Actions run、Windows 制品 ID 与 MSI／NSIS 摘要。两端定向复验通过后再冻结同源制品。

已复算旧候选 Actions `45630909723`／Windows 制品 `15030939242`。旧源为 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，MSI SHA-256 `f8bbece2817e40ae3b5528d2d00caa055cebc158617cad13569da15de6fe4fe8`，NSIS SHA-256 `4999df3bcfbc47294ff08a4093673e1ca11d313e15dc091d9a6d6114266b8db4`。本次源提交属于 CI 修复，未生成可验收的新安装器。[旧制品复算原始回执](../team81-20261126-resume-r1/raw/digest-recheck.json)

首次本地工具调用沿用了默认 GNU target，依赖编译因 `dlltool.exe` 缺失退出，原日志已保留。随后显式选择 MSVC；保留了原记载失败。[首次 Clippy](raw/clippy.log) · [首次原生测试](raw/native-tests.log) · [MSVC 工具链](raw/rustc-msvc.log)

只读复查未发现本工作树的候选或测试程序进程。自动审批以 `blocked by policy` 拒绝了清理命令。两个生成 schema 文件的工作树变动和临时 Node 编译缓存保留在本轮工作树，没有进入源码提交。[清理读回](raw/cleanup.json) 历史失败、marenben 及 Profile 保留。本轮未启动候选、修改系统策略、发出业务同步或模型请求，未执行 W2 或发布。`accepted:false`。
