# 05-Windows CI 首屏门禁复核与修正

2026-10-06，Asia/Shanghai。关联 [IMP-OPENCODEX-DESKTOP-18](../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-18.md)。用户明确授权回写复核结论、目标模式开发、本机验证、提交推送，并交给 Windows 设备复验。状态：**实施已授权；修正后的 Windows 验证待执行。**

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

待开发后补记源码提交、本机检查结果、远端 CI 状态及 Windows 复验提示词。没有 Windows 通过证据前，不声明首屏门禁已修复或版本可发布。文档与证据在 `docs/governance-main`；源码、测试与 CI 在 `feature/0.1.8-windows-fixes`。本轮授权不含合并 main、修改发布说明、制作安装器、打标签或发布。

## 官方依据

- [Microsoft WebView2 browser flags](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags)：环境变量、显式 AdditionalBrowserArguments 与权限行为。
- [Tauri WindowConfig](https://v2.tauri.app/reference/config/#windowconfig)：additionalBrowserArgs 和独立 dataDirectory；实现以项目 lockfile 对应源码为准。
- [GitHub hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)：Windows runner 权限；实际 token 和子进程参数仍需本轮诊断。
