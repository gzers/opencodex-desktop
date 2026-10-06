# 05-Windows CI 首屏门禁复核与修正

2026-10-06，Asia/Shanghai。关联 [IMP-OPENCODEX-DESKTOP-18](../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-18.md)。用户明确授权回写复核结论、目标模式开发、本机验证、提交推送，并交给 Windows 设备复验。状态更新：**续修已推送；修正后的 Windows CI 与 Windows 普通/管理员首屏复验均通过**。本节保留 Mac 续修及 10:20 的历史快照，实际 Windows 回执另见 [06-双权限首屏复验](06-Windows普通与管理员首屏复验.md)。

## 已确认的失败边界

原始源码为 `feature/0.1.8-windows-fixes@785e80dada6928c9643ee69f16056d414cd7304e`。GitHub [CI run 37397271349](https://github.com/gzers/opencodex-desktop/actions/runs/37397271349) 的前端、macOS 后端、Linux 发布工具均通过；Windows 的格式、Clippy、后端回归及 release EXE 构建通过，失败发生在 `Windows final EXE smoke`：`Main WebView CDP target did not become available`。主进程、窗口句柄、启动日志及隔离数据初始化检查此前通过。

该失败说明测试未取得调试连接；现有证据不足以认定产品首屏绘制失败，也不是 NSIS/MSI 安装器打包失败。原始 Windows 真机成功记录和托管 runner 失败记录分别保留，不以其中一项替代另一项。

## 复核结论与实施口径

1. WebView2 支持 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`；Tauri/wry 没有直接读取变量，不等于 SDK 不支持。微软文档说明管理员权限可能使该变量被忽略。GitHub Windows runner 的权限是应记录的排查线索，尚未由本轮 artifact 证明为唯一根因。
2. 原脚本预选的是非零调试端口。没有 `DevToolsActivePort` 文件不能据此证明参数失效；本轮保留明确端口，不把未经验证的 port=0 文件发现方式列成修复结论。
3. 在 WebView 创建前，通过 Tauri 的浏览器参数配置明确传入 smoke 端口；入口限定为专用 smoke 开关、有效端口和独立沙箱，普通 `OPENCODEX_SANDBOX=1` 不自动开启调试入口。继续隔离 WebView 数据。
4. 补齐权限、端口监听、WebView2 子进程命令行、`/json/version`、`/json/list` 与最后请求错误证据，失败时仍输出诊断。请求有超时，连接异常不再被空 catch 吞掉。
5. 保留 Vue 首屏、Tauri invoke、构建来源和截图断言；不将门禁降为“进程活着即可”。若托管环境确实不支持，应迁移测试环境并保留发布前硬门禁。

本机 Mac 可以执行前端、通用 Rust、脚本协议与配置边界测试；不能把 WKWebView 结果当成 Windows WebView2 验证。Windows 必须从修正后的同一提交重建最终 EXE 并执行现有 smoke；至少比较普通权限与管理员权限两种启动。

## 实施与验证回写

源码终点为 `feature/0.1.8-windows-fixes@9262a5bedbb1a5f454f32afb7418b959fa8c721f`，两笔提交已推送：

- `d2c380104e6f91dfe73ce001aa104689247e1a55`：显式传入首屏测试的 WebView2 调试配置。仅 `OPENCODEX_WINDOWS_SMOKE_CDP_PORT` 与有效隔离 sandbox 同时成立时延迟 main 自动创建，再用窗口 builder 传入参数和绝对数据目录；普通启动与普通 sandbox 不开启 CDP。增加 4 项纯配置回归。
- `9262a5bedbb1a5f454f32afb7418b959fa8c721f`：保留首屏门禁并补全失败证据。CDP discovery 限定 HTTP loopback、非零端口和有界重试，保留版本、目标、响应及最后错误；脚本保存实际权限 token、自己启动的进程树及监听端口，失败仍上传证据、返回非零。新增 7 项协议回归并接入 CI。

脚本清除子进程继承的 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 和 `WEBVIEW2_USER_DATA_FOLDER`，明确验证代码传参；保留无 HOME、构建来源、窗口句柄、Vue 渲染、Tauri invoke、隔离数据、CLI 未启用和截图检查。权限差异仍是待实证假设，不写成已证明的唯一根因。

本机是 macOS，Node 26.8.1、Rust/Cargo 1.95.0，临时 PowerShell 7.6.6 仅用于语法解析。实际检查结果：

| 检查 | 结果与边界 |
|---|---|
| 前端 typecheck、测试、生产 build | 通过，375 项测试；未改前端生产源码 |
| Rust fmt、Clippy（all-targets、警告即失败）、integration-test | 通过，574 项通过、5 项原有忽略；包含新增 4 项配置测试 |
| CDP 协议回归 | 7 项通过：延迟 target、HTTP 错误、非法 JSON、非数组/空列表、停滞超时、连接拒绝和非 loopback 拒绝 |
| Node 脚本语法、PowerShell Parser | 通过；不代表 PowerShell Windows API 与 WebView2 原生执行通过 |
| diff 空白检查 | 通过 |
| Windows final EXE smoke | 本机未执行，交 Windows 设备从该提交重建并复验 |

本机离线生成的未跟踪 Cargo.lock 解析到 Tauri 2.12.1、tauri-utils 2.10.1、tauri-runtime-wry 2.12.1、wry 0.57.0；本轮没有新增受跟踪锁文件。WebView 创建与参数链路已核验对应本机依赖源码，仍需 Windows 构建验证。

2026-10-06 10:20（Asia/Shanghai）远端快照：[CI run 37403376481](https://github.com/gzers/opencodex-desktop/actions/runs/37403376481) 对应上述源码终点，状态为 `in_progress`（frontend=success；release-tools=success；backend-windows=in_progress；backend=in_progress），Windows 首屏尚无通过结论。机器记录见 [WIN-018-CDP-20261006](../../../../../../../../.adg/evidence/WIN-018-CDP-20261006/manifest.json)。

文档与证据在 `docs/governance-main`；源码、测试与 CI 在 `feature/0.1.8-windows-fixes`。本轮未合并 main、未修改 CHANGELOG、未制作安装器、未打标签或发布。Windows 首屏通过后，仍保留 macOS 原生 UI/安装、安装器和 OTA 等既有集成门禁。

## Windows 复验

在已有仓库同步特性分支与文档分支，确认代码是 `9262a5bedbb1a5f454f32afb7418b959fa8c721f`。使用 Node 24 或更高版本、PowerShell 7，从该提交重建 release EXE。先执行本地协议和后端回归，参照 `.github/workflows/ci.yml` 设置 `OPENCODEX_TEST_NODE`；不复用旧 EXE。

从仓库根构建与执行普通权限 smoke：

```powershell
npm --prefix apps/desktop/ui ci
npm --prefix apps/desktop/ui run build
cargo build --manifest-path apps/desktop/tauri/Cargo.toml --release --features custom-protocol --bins
pwsh -NoProfile -File test/smoke/windows-startup.ps1 -Executable apps/desktop/tauri/target/release/opencodex-desktop.exe -Node (Get-Command node).Source -OutputDirectory test/out/windows-cdp-ordinary -ExpectedCommit 9262a5bedbb1a5f454f32afb7418b959fa8c721f
```

再从管理员 PowerShell 7 执行同一脚本与同一 EXE，将输出目录改为 `test/out/windows-cdp-admin`。两次均使用独立 sandbox；依据 `startup-diagnostics.json` 的实际 token 判断是否覆盖普通与提升权限，不能仅依据窗口标题判定。

保存两组 `startup.json`、`startup-diagnostics.json`、`cdp-diagnostics.json`、`webview.json`、`main-window.png`、启动日志及 EXE SHA-256；若失败阶段尚未生成某文件，保留实际存在的全部证据。按 CDP 版本/目标、监听及子进程参数区分原因，不删除或弱化首屏门禁。将结果与证据回写文档分支本专题和对应 `.adg/evidence/`。不要替换日常 0.1.7，也不要在本次测试中合并或发布。

可直接交给 Windows 开发代理的提示词：

> 同步 `feature/0.1.8-windows-fixes` 和 `docs/governance-main`，按分析文档 05 复验代码 `9262a5b`。从该提交重建 release EXE，在普通与管理员 PowerShell 7 下分别运行 `test/smoke/windows-startup.ps1`，使用两个独立输出目录。保留首屏断言及全部诊断、截图、日志和 EXE 身份，失败按证据定位，结果回写文档分支。不要替换日常 0.1.7，不合并、不发布。

## Windows 本机复验回执（2026-10-06）

2026-10-06 Windows 回写：从同一 `9262a5b` 重建 release EXE，PowerShell 7.6.5 的普通与管理员组均按实际 token 覆盖并通过原首屏断言；本轮 CDP 协议 7 项、Windows 后端 335 项及格式/Clippy 通过。CI run `37403376481` 已为 success，Windows 首屏步骤明确成功。详细诊断、两组截图、EXE 身份和边界见 [06-双权限首屏复验](06-Windows普通与管理员首屏复验.md)；未修改代码、未替换日常 0.1.7、未合并或发布。原失败的唯一根因仍不由本次新版本通过结果倒推。

## 官方依据

- [Microsoft WebView2 browser flags](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags)：环境变量、显式 AdditionalBrowserArguments 与权限行为。
- [Tauri WindowConfig](https://v2.tauri.app/reference/config/#windowconfig)：additionalBrowserArgs 和 dataDirectory；绝对 sandbox 数据目录通过 builder 设置，实现已核验本机解析依赖源码。
- [GitHub hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)：Windows runner 权限；实际 token 和子进程参数仍需本轮诊断。
