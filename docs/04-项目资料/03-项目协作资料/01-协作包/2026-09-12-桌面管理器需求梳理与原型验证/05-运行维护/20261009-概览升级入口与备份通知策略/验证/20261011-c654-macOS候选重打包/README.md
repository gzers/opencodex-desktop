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
