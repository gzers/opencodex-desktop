# TEAM83 Windows 原生分支 CI 最终回执

2026-11-26。TEAM83／IMP229 (`3e1a978a294c5490388185bafef6fa822732a923`) 接收 Windows 原生提交 `a288fc41457ffa682edb3b3df2e00500a61a5649`。主线仍为 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`。共享首次供应、关联界面、打包和共享严格检查归 Mac。

待证伪判断：独立 Windows 原生检查可以通过，同时共享严格检查失败会在原构建门槛阻止生成候选。若有诊断落在 Windows 独占文件，则在该写集内修复。

| 项目 | 结果 | 证据 |
| --- | --- | --- |
| Windows smoke、TokenUser、隔离凭据保护 | pass | Windows smoke 作业通过。TokenUser 13 项通过、1 项跳过。凭据保护三次定向测试均通过。[测试日志节选](raw/native-checks.log) |
| Windows 严格 Clippy | fail | 首错 `program_image.rs:540` 的 `Atomic::fetch_update` 已弃用。其余 Windows Clippy 诊断涉及镜像清单、恢复命令和程序移除三个共享文件。[原始连续错误段](raw/windows-clippy.log) |
| Windows MSVC 发行 EXE、MSI／NSIS、回归 | blocked | 发行 EXE 构建和 smoke 跳过，总构建跳过。Actions `45774644978` 上传了测试和诊断证据。[最终作业](raw/jobs.json) · [制品清单](raw/artifacts.json) |
| TEAM81 完整包断网准备 | blocked | 缺修复后安装器及可独立断网并保持操作的 Windows 桌面。[既有环境证据](../team81-20261126-resume-r1/result.md) |
| TEAM81 缺包确认下载 | fail（旧候选） | 确认后下载请求发出前返回 `managed_dependency_missing`，待新候选复验。[原实机证据](../team81-20261125-r1/result.md) |
| TEAM81 首次 Status、未知、30／5／10 秒启停 | blocked | 待修复候选。从环境就绪后实际页面发出的首次 `get_status_snapshot` 计时，区分前置选页探测。超过 5 秒时验证未知提示、重试、诊断与 Start 不可用。确认空闲后验证启停。 |
| W2／真实双 SID | blocked | 待两端定向通过、同一 main SHA 冻结及普通用户 per-user MSI 前提。[历史 W2](../team79-20261124-r1/sweep/summary.json) |

Windows CI 作业为 [139755250749](https://github.com/gzers/opencodex-desktop/actions/runs/45774644978/job/139755250749)。此前被自动审批拒绝的生成文件和缓存清理沿用[原回执](../team82-20261126-native-ci-r1/result.md)。本轮未启动候选、变更网络、安装策略或日常配置，保留原失败，`accepted:false`。
