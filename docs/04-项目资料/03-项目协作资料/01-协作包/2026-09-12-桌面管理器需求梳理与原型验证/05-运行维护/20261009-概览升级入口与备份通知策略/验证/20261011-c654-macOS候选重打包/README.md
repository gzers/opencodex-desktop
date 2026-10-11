---
id: VFY-OCX-0110-20261011-C654-MACOS-CANDIDATE
kind: implementation.validation
state: partial
source_commit: c654287c06da3e621091c1051ebc19ce1c0b1077
---

# 2026-10-11 c654 macOS arm64 候选重打包

本记录对应 `feature/0.1.10-maintenance@c654287c06da3e621091c1051ebc19ce1c0b1077`，用于区分此前 `main@861fdeea` 的 macOS 0.1.10 构建记录。构建命令为：

```text
cargo tauri build --bundles app,dmg -- --locked
```

命令退出码为 0，前端构建转换 170 个模块；候选应用的 `CFBundleIdentifier` 为 `com.gzers.opencodex.desktop`，`CFBundleShortVersionString` 和 `CFBundleVersion` 均为 `0.1.10`，可执行文件为 arm64 Mach-O。

| 制品 | 复核结果 | 摘要 |
| --- | --- | --- |
| `apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app` | 通过身份复核 | 可执行文件 SHA-256 `e94b6cb758b4fbd4ecc67cd4af7f3dcbc024071be3b3d4d61f3b18589b574ec6`；Info.plist SHA-256 `80f28d5ee5bbb972e9ff1f4b0b92d437f72abb53df70923f1b45f3bb4f8f47e5` |
| `.app` canonical ditto zip 表示 | 已记录 | 4,262,901 bytes；SHA-256 `87f939f19003285a9e9e296cf5097ad33f0944552fcb3b243a2771c9da28617f` |
| `apps/desktop/tauri/target/release/bundle/dmg/OpenCodeX Desktop_0.1.10_aarch64.dmg` | 容器校验通过 | 4,383,007 bytes；SHA-256 `f2d903261bd7b6627d2e8872157e5bd03ee0ce18d7d66a1cea6ff5d33809bb25`；`hdiutil verify` 通过 |

当前应用为 ad hoc、linker-signed，`TeamIdentifier` 未设置；`codesign --verify --deep --strict` 报告 `code has no resources but signature indicates they must be present`，因此未通过正式签名校验。该候选包不能作为已签名、公证或公开发布制品。

本轮进程整理后仅保留一个隔离候选测试实例（PID `86306`）；日常安装实例（PID `75764`）保持运行且未操作。进程数量观察只用于测试环境卫生，不替代 macOS 原生 UI、安装升级、退出重启和发布验收。

源码门禁沿用同一 `c654287c` 复核结果：Rust 全量 `861 passed / 0 failed / 5 ignored`，UI `94` 个测试文件、`459` 项通过，metadata、fmt、Clippy `-D warnings`、类型检查、生产构建和 Windows bundle identity smoke 均通过。源码通过不等于双平台原生验收通过。

仍开放的门禁包括 macOS 正式签名 / 公证、真实安装升级 / 重启回读、Windows 普通与管理员权限及安装路径 / 数据根 / DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、自动检查性能与 stable 24h / beta 6h 周期、真实更新端点、全域事件执行证据和发布授权。状态保持 `in_progress`，`release_authorized: false`；不创建标签、不公开发布、不晋升 stable、不替换日常 0.1.7。

## 同日补充：唯一测试实例与原生 UI 局部观察

以上 PID `86306` 为此前记录时点。后续已通过 SIGTERM 退出该旧测试实例；此操作不能计为正常退出或锁清理验收通过。当前仅保留 PID `17410`，路径为 `/private/tmp/ocx-0110-dmg-install-tuk3m7/c654/OpenCodeX Desktop.app`，bundle identifier 为 `com.gzers.opencodex.desktop.sandbox0110c654`，数据根为 `/private/tmp/ocx-0110-dmg-install-tuk3m7/data`。日常安装 PID `75764` 保持运行，未操作。

保留实例由 c654 候选复制、修改测试身份并本地 ad hoc 重签。测试副本的签名不能覆盖上文原候选严格签名失败；该实例也不包含当前 `main@861fdeea39d1c45fa2beddb2cba6659b67735d9f` 新增的 Windows IPC 实现。

| 观察范围 | 实际结果与边界 |
| --- | --- |
| 真实偏好备份 | `backups/2026/10/manual-preferences/bk_20261011055615_caf8ad73/` 含 manifest 与 preferences 两个文件。UTC 创建时间 `2026-10-11T05:56:15Z`；偏好文件 1371 bytes，SHA-256 `58e1c8cc33f43243cd1dee8db4a45da712106e44c29c1ad6a0f97288843e3c85`，与清单一致。仅核对隔离根中的偏好备份，未恢复或清理。 |
| 备份树表 | 初始根与年份展开、月份收起；月份、事务分类、备份包可展开；两个文件均显示用途及打开操作。刷新后展开层级保留。焦点保留未验证；该树无复选框，不能替代复选框 / 箭头 / 文字对齐矩阵。未打开文件或目录。 |
| 设置更新入口 | 检查后打开共用更新中心，管理器及面板可进入详情并返回。管理器当前 `0.1.10`、可用版本 `—`；面板本地未发现，当前未知、远端可用 `2.83.0`，界面提示版本查询失败。两者安装按钮禁用。 |
| 更新说明 | 两类详情均展示无发行说明回退提示；面板有官方网页详情入口，未点击。当前无可执行更新，真实下载进度、长说明内部滚动、安装及重启回读继续待验。 |
| 概览光层 | 未接入 OpenCodex 的准备态、当前视口顶部和左右未见明显内层边距截断。未完成主题 / 三档 / CSS / WebGL / 就绪态完整矩阵，不提升为全部外观通过。 |

原生证据来自本会话 CUA AX 树及截图观察；截图未落盘。进程快照、测试身份、备份元数据和局部结果保存在 `.adg/evidence/OCX-0110-20261011-C654-NATIVE-UI-PARTIAL/`，不归档偏好正文。

本轮回写后保持版本实施 `in_progress`。下一轮须以当前 main 的明确候选身份补齐 Windows 双权限安装 / 数据根 / DPI / 原生外观 / IPC、macOS 正式签名公证及真实更新重启、自动检查性能和真实周期、全域事件执行证据。当前设备 / 签名及更新条件不足，且相同门禁已在连续执行轮次重复出现；目标执行转入等待外部条件的 `blocked`，不靠重跑已通过源码门禁或增加前台测试实例推进。0.1.10 正式发布仍需单独授权。
