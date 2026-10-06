# 06-Windows 普通与管理员首屏复验

2026-10-06，Asia/Shanghai。承接 [05-Windows CI 首屏门禁复核与修正](05-Windows-CI首屏门禁复核与修正.md) 和 [IMP-OPENCODEX-DESKTOP-18](../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-18.md)。用户明确要求同步分支、从 `9262a5b` 重建 release EXE、分别以普通和管理员 PowerShell 7 执行原首屏脚本，保存证据并提交推送文档。**两组本机首屏复验均通过；对应 Windows CI 首屏步骤也已通过。**

本轮仅测试与回写。未修改源码或首屏断言，未替换日常 0.1.7、未合并 main、未制作安装器、未打标签或发布。原始 CI 失败记录保留，不将本次通过结果倒写为此前已通过。

## 固定源码与 EXE 身份

同步后的代码分支为 `feature/0.1.8-windows-fixes`，终点固定：

`9262a5bedbb1a5f454f32afb7418b959fa8c721f`

文档分支以 `docs/governance-main@df5cb8efa5ef930f9554ada6b71e64fcb9ee8432` 为本轮回写基线。两条分支只快进同步，没有合入 main。

| 项目 | 本次核对值 |
|---|---|
| 本机系统 | Microsoft Windows Server 2025 Datacenter，10.0.26100，x64 |
| PowerShell / Node / Rust | 7.6.5 / 24.19.0 / 1.99.0 |
| Tauri / tauri-utils / tauri-runtime-wry / wry | 2.12.1 / 2.10.1 / 2.12.1 / 0.57.0；本机未跟踪 Cargo.lock 的解析值 |
| 构建命令 | `cargo build --manifest-path apps/desktop/tauri/Cargo.toml --release --features custom-protocol --bins` |
| 候选 EXE | 独立代码工作树内 `apps/desktop/tauri/target/release/opencodex-desktop.exe` |
| 文件大小 / FileVersion / ProductVersion | 7,617,024 bytes / 0.1.8 / 0.1.8 |
| EXE SHA-256 | `d02e2d373043a113aa56989c45a7557e3f232fa1228eec3cd1b7acfd73e8c30c` |
| 子系统 / 构建用途 | Windows GUI；仅复验最终 EXE，没有 NSIS/MSI 或签名发布包 |

本机工具链为此前准备的用户缓存 Rust、LLVM 23.1.2 与 Windows SDK/CRT。release 编译成功，保留了便携 SDK 缺少 PDB 的调试信息告警；没有通过抑制首屏断言或忽略失败取得通过。EXE 元数据、构建日志与源文件指纹分别保存在 [exe-identity.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/exe-identity.json)、[release-build.log](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/release-build.log) 和 [source-identity.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/source-identity.json)。

## 两种权限对照

普通与管理员组均调用原提交中的 `test/smoke/windows-startup.ps1`，`ExpectedCommit` 明确传入完整提交，观察窗口均为 30 秒。管理员组由 `Start-Process -Verb RunAs` 启动同一 PowerShell 7；临时调度包装只记录调用上下文、日志和退出码，不改变测试脚本。权限依据脚本实际 token，不依据窗口标题。

| 项目 | 普通权限 | 管理员权限 |
|---|---|---|
| PowerShell 调度起止（本地时间） | 10:37:11–10:37:46 | 10:38:10–10:38:45 |
| effectiveAdministrator / tokenElevated | false / false | true / true |
| tokenElevationType | 3（受限 token） | 2（提升 token） |
| 独立输出目录 | `test/out/windows-cdp-9262a5b-20261006/ordinary` | `test/out/windows-cdp-9262a5b-20261006/administrator` |
| 测试进程 PID / CDP 监听进程 PID | 73464 / 67556 | 62712 / 51472 |
| 实际监听端点 | `127.0.0.1:58971` | `127.0.0.1:65317` |
| WebView2 浏览器身份 | `Edg/154.0.4258.53` | `Edg/154.0.4258.53` |
| `/json/version`、`/json/list` | 均 HTTP 200，首次发现尝试取得主 target | 均 HTTP 200，首次发现尝试取得主 target |
| 主 target | `http://tauri.localhost/#overview` | `http://tauri.localhost/#overview` |
| 首屏挂载 / 文本长度 | true / 452 | true / 452 |
| smoke / 调度退出码 | pass / 0 | pass / 0 |

两组使用同一 EXE SHA-256，但输出目录、sandbox、WebView 数据目录、端口与进程各自独立。诊断都记录六个属于测试进程树的 WebView2 子进程，浏览器命令行实际含对应的 `--remote-debugging-port`；监听所有者属于该进程树。未继承 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 或 `WEBVIEW2_USER_DATA_FOLDER` 来掩盖显式配置。

保留并实际通过的门禁包括：移除 HOME、持续存活、主窗口句柄与响应、启动日志构建来源匹配、隔离数据根初始化、Vue 首屏真实渲染、Tauri bridge 及关于/发现/偏好/数据根 invoke、Windows/0.1.8 元数据、CLI 未启用和首屏截图。未降为“进程存活即通过”。

两组启动日志分别记录：

```text
2026-10-06T02:37:12.486Z [manager] startup version=0.1.8 commit=9262a5bedbb1a5f454f32afb7418b959fa8c721f
2026-10-06T02:38:11.826Z [manager] startup version=0.1.8 commit=9262a5bedbb1a5f454f32afb7418b959fa8c721f
```

上述日志为 UTC，对应本地 10:37:12.486 与 10:38:11.826。首屏均显示 Node/npm 已发现、官方运行时待接入；这是实际未安装运行时的引导状态，不是绘制失败。本轮没有执行官方代理启停或安装。

- 普通组：[启动结果](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ordinary/startup.json)、[权限/进程/监听](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ordinary/startup-diagnostics.json)、[CDP 诊断](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ordinary/cdp-diagnostics.json)、[首屏截图](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ordinary/main-window.png)。
- 管理员组：[启动结果](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/administrator/startup.json)、[权限/进程/监听](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/administrator/startup-diagnostics.json)、[CDP 诊断](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/administrator/cdp-diagnostics.json)、[首屏截图](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/administrator/main-window.png)。

两组 `webview.json`、`app.log`、标准输出/错误、执行日志、调度上下文和结果同目录保存。证据原字节迁入文档分支 `.adg`，不依赖本机临时输出路径。

## 前置回归与远端 CI

| 检查 | 本轮结果 |
|---|---|
| CDP 协议 | 7 passed / 0 failed；异常响应、有界超时与 loopback 校验保持 |
| Windows 后端 integration-test | 335 passed / 0 failed / 5 ignored；含新增四项 smoke 配置测试 |
| Rust fmt、all-targets Clippy | 通过，Clippy 仍使用 `-D warnings` |
| PowerShell 脚本解析 | 通过；随后原 Windows API 在两种实际权限下均执行成功 |
| 前端类型检查 / 生产 build | 通过；本轮未重跑前端单测，375 项的历史和 CI 结果分别保留 |
| 本轮候选 EXE 构建 | 通过；从固定源码重建，未复用旧 EXE |

Windows 后端数量不包含 Unix 专属测试；既有真实外部服务测试的环境开关及忽略边界保持不变，不能将 335 项解释为所有外部服务均已实际验证。

2026-10-06 本轮读取 GitHub API 与运行页，[CI run 37403376481](https://github.com/gzers/opencodex-desktop/actions/runs/37403376481) 对应同一 `9262a5b`，已完成为 **success**。frontend、backend（macOS）、release-tools、backend-windows 均成功；Windows job 的 `Windows final EXE smoke` 明确 success。Windows job 完成时间为 `2026-10-06T02:31:29Z`，整个 run 状态更新于 `02:31:30Z`，对应本地 10:31:29/30。feature 分支的安装包 `build` job 按现有条件 skipped；不是制作安装器或发布完成。

完整 API 快照：[ci-run.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ci-run.json)、[ci-jobs.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/ci-jobs.json)。这更新了文档 05 在 10:20 的“进行中”快照，保留其原始时点，不覆盖旧 run `37397271349` 的失败。

## 结论与运行边界

续修版本在本机普通与管理员权限下，显式浏览器参数都形成了正确 loopback CDP 端点，并通过保留的首屏门禁；对应 CI 也通过。**本次没有复跑旧版本的权限 A/B，因此仍不能把原 CI 失败唯一归因为提升权限或环境变量被忽略。**

测试脚本结束后，本轮两组记录的测试进程和 CDP 监听均已释放，测试数据与证据保留。日常安装 EXE 仍为 0.1.7，SHA-256 保持 `058d924f762b8c1f7b1909e4b63ffb79ed5a4009663a80e5bc9825829798afba`；原快捷方式摘要未变，日常实例 PID 19216 与启动时间保持连续。main 仍为 `5a0859ad2e6d44f4ca5751eedfb5a1b7eb8c5d09`。前后核对见 [daily-baseline.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/daily-baseline.json) 和 [postcheck.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/postcheck.json)。

源码和原 smoke 脚本均未修改。安装器/升级、Windows 10/11 与 DPI、任务栏/快捷方式缓存、真实官方代理生命周期、OTA 和 macOS 原生 UI/安装仍属于后续集成范围；本次首屏通过不代替这些门禁。

证据总入口：[WIN-018-CDP-WINDOWS-20261006/manifest.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/manifest.json)，复验摘要见 [verification.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/verification.json)。本轮只归档诊断、日志、截图和身份摘要，不把候选 EXE、整个 WebView 缓存、测试数据根或源码复制到文档分支。

## 文档提交与推送状态

本轮复验结果与证据已在 `docs/governance-main` 本地提交。推送尝试被本机 GitHub 认证阻断：Git Credential Manager 的 GitHub 账号列表为空，非交互 `git push` 返回 `Cannot prompt because user interactivity has been disabled` 与 `unable to get password from user`。已检查可用连接通道，未取得可代替本机 Git 认证的已连接写入工具；没有要求或保存凭据值。

推送检查时，远端文档分支仍为 `df5cb8efa5ef930f9554ada6b71e64fcb9ee8432`，代码分支为固定 `9262a5b`，main 未变。此处不宣称本轮文档已推送；认证完成后只需推送已有 `docs/governance-main` 提交，无需重跑已通过测试。实际错误及远端快照见 [push-attempt.json](../../../../../../../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/push-attempt.json)。
