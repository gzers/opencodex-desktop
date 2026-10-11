# 20261011 macOS arm64 候选构建

日期：2026-10-11  版本：0.1.10  状态：candidate_build_passed

代码基线为 main@861fdeea39d1c45fa2beddb2cba6659b67735d9f，在 macOS 27.0.1 arm64 宿主执行 cargo tauri build --target aarch64-apple-darwin，退出码为 0。前端构建完成 170 个模块转换，并生成以下两个 0.1.10 候选制品：

| 制品 | 架构 | 大小 | SHA-256 | 结果 |
|---|---|---:|---|---|
| OpenCodeX Desktop.app（以 ditto zip 记录摘要） | arm64 | 4,265,674 bytes | 3221a952eb07c0cee57fc03486ce252e604c2470025bd04028d5d4dbfc0afd1f | 版本与 Mach-O 架构通过 |
| OpenCodeX Desktop_0.1.10_aarch64.dmg | arm64 | 4,390,813 bytes | e3f89379909f1dd7c185409f207f0c4223cf9d178b8244943760dc4cdcdef283 | hdiutil verify 通过 |

Info.plist 的 CFBundleShortVersionString 与 CFBundleVersion 均为 0.1.10，应用可执行文件为 arm64 Mach-O。当前应用为 ad hoc 签名，TeamIdentifier 未设置；本轮没有进行正式签名、公证、真实安装、升级、重启回读或更新端点验收。

证据原件：

- [.adg manifest.json](../../../../../../../../../.adg/evidence/OCX-0110-20261011-MACOS-ARM64-BUILD/manifest.json)
- [.adg commands.json](../../../../../../../../../.adg/evidence/OCX-0110-20261011-MACOS-ARM64-BUILD/commands.json)
- [.adg result-summary.json](../../../../../../../../../.adg/evidence/OCX-0110-20261011-MACOS-ARM64-BUILD/result-summary.json)

本记录只闭合当前 main 的 macOS arm64 打包门禁。macOS 原生 UI / 安装升级回归、Windows 0.1.10 制品与双权限 / 安装路径 / 原生外观验收、自动检查性能、真实更新 / 重启 / 恢复、正式签名公证 / Authenticode、全域事件执行证据和发布授权仍未闭合。状态继续为 in_progress，release_authorized: false；不创建标签、不公开发布、不晋升 stable、不替换日常安装。
