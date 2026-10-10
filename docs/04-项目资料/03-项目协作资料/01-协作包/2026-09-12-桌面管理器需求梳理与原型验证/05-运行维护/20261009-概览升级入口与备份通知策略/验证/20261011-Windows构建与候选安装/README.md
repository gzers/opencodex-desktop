---
id: VFY-OCX-0110-20261011-WINDOWS-BUILD-INSTALL
kind: implementation.validation
state: partial
source_commit: 0b402c154be65760092b6817b3027a2d0a1dc6dc
---

# Windows 构建与候选安装验证

本记录回写 0.1.10 实施期间对 Windows 构建链和两种安装器的验证结果。代码来源为 feature/0.1.10-maintenance 的 0b402c154be65760092b6817b3027a2d0a1dc6dc。

0b402c15 将 Tauri 构建改为 WindowsAttributes::new_without_app_manifest()，避免 Tauri 默认 manifest 与项目 Common Controls v6 manifest 重复链接，解决 Windows 资源构建失败。

普通 CI [38083572209](https://github.com/gzers/opencodex-desktop/actions/runs/38083572209) 已完成并成功，frontend、release-tools、backend、backend-windows、Windows backend regression、Windows release build、Windows final EXE smoke 和启动证据上传均通过；其中条件跳过的 build job 不是失败。候选打包 CI [38083572237](https://github.com/gzers/opencodex-desktop/actions/runs/38083572237) 已完成并成功。

候选打包环境为 win25-vs2026、镜像 20260925.250.1、Rust 1.99.0、Cargo 1.99.0、Node v24.21.0、npm 11.19.0、Tauri CLI 2.11.4。Windows backend regression、release build 和 final EXE smoke 通过。

| 安装器 | 文件 | SHA256 | 安装 | 首屏 | 卸载 | EXE 移除 |
|---|---|---|---|---|---|---|
| MSI | OpenCodeX Desktop_0.1.9_x64_en-US.msi | b7c20061e97b47156c9c5a4693786843f01bb9b949d0825eea2c23c6d3599116 | pass | pass | pass | 是 |
| NSIS | OpenCodeX Desktop_0.1.9_x64-setup.exe | 9711d86321ac77d8723d1dfcae8d3e9c0a282debf6da20c738a953de89445438 | pass | pass | pass | 是 |

候选包 artifact 为 11682205024（opencodex-desktop-0.1.9-windows-x64-candidate，7,370,689 bytes）；候选证据为 11681514176（1,648,485 bytes）；普通 CI 启动证据为 11682320162（9,152,150 bytes）。对应 ZIP SHA256 已登记在 .adg/evidence/OCX-0110-20261011-WINDOWS-BUILD-INSTALL/artifact-sha256.json。

启动证据为管理员权限路径：30 秒内主窗口存在且响应，source commit 匹配，CDP HTTP 200，WebView2 153.0.4234.48，页面为 http://tauri.localhost/#overview。这证明构建、隔离安装、首屏和卸载链路通过，不能替代普通权限人工复验或 DPI / 外观验收。

本次候选的实际版本仍是 0.1.9。codeSigned: false、releaseAccepted: false，因此它不是 0.1.10 发布资产，也没有产生发布授权。macOS 原生安装、真实更新 / 重启、双平台自动检查性能、stable 24h / beta 6h 周期、Windows 普通权限、安装路径 / 自定义路径 / DPI / 外观、真实用户迁移、签名和公开发布仍保持未闭合。

证据明细见同目录下 .adg/evidence/OCX-0110-20261011-WINDOWS-BUILD-INSTALL/ 的 JSON 文件。IMP-20、本协作包和执行基线保持 in_progress。
