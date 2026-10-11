# 2026-10-11 macOS 原生候选包隔离启动观察

## 结论

基于 main@861fdeea39d1c45fa2beddb2cba6659b67735d9f 的 0.1.10 macOS arm64 候选 .app 已在临时隔离根启动，首屏可读，临时数据根真实生效。该观察只闭合候选包的隔离启动与首屏可读性；优雅退出、锁清理、重启回读和真实 DMG 安装仍未闭合，不能作为 macOS 原生安装验收或发布授权。

## 观察范围

- 隔离根：/tmp/ocx-0110-native-install-graceful-20261011101622-65451
- 候选包：OpenCodeX Desktop.app
- 启动方式：候选包 wrapper 启动，并设置 OPENCODEX_SANDBOX=1、OPENCODEX_SANDBOX_ROOT 为上述临时根。
- bundle identifier：com.gzers.opencodex.desktop.sandbox0110graceful
- 代码身份：日志记录 version=0.1.10、commit=861fdeea39d1c45fa2beddb2cba6659b67735d9f。

## 已观察结果

1. 应用能够启动并显示真实首屏：OpenCodeX Desktop、概览、面板、拓展、诊断、设置和环境检查区域可读；Node.js 与 npm 已发现，OpenCodex 未发现，启动条件显示未满足。
2. 临时根下生成了 data/backups、data/cache、data/exports、data/logs、data/manager-state、data/opencodex-home 和 data/sync-state 等目录，说明候选运行时没有写入日常安装的数据根。
3. data/logs/app.log 记录：

       2026-10-11T02:16:23.257Z [manager] startup version=0.1.10 commit=861fdeea39d1c45fa2beddb2cba6659b67735d9f
       2026-10-11T02:18:41.661Z [manager] window dock:reopen: show=true focus=true dock=regular

4. 点击窗口关闭后窗口隐藏 / 重开，候选进程仍存活；这符合当前保持运行 / 托盘方向，但尚未证明退出动作的完整生命周期。
5. 本轮随后通过终端终止进程，不属于应用自身优雅退出。临时根仍有 data/manager-state/app.lock，内容为旧 PID 65472；再次启动时旧锁阻止了新的实例。因此本轮不能记为“正常退出、锁清理、重启回读通过”。

## 仍开放的门禁

- 通过应用退出路径完成退出，确认 app.lock 正常清理。
- 退出后重新启动，确认偏好、数据根、事件范围、更新计划和缓存状态可回读且无旧锁误报。
- 从 0.1.10 DMG 做真实临时安装、首次启动、升级保留和卸载边界验证；不得写入日常 /Applications/OpenCodeX Desktop.app。
- macOS 原生标题栏、云母 / 材质、动画、下拉框和统一树表的原生 UI 验收。
- 正式签名、公证、真实更新端点、下载校验、重启和失败恢复。

原始机器可读证据见 .adg/evidence/OCX-0110-20261011-MACOS-ISOLATED-STARTUP/。本记录不创建标签、不公开 Release、不晋升 stable、不替换日常 0.1.7，也不迁移真实用户数据。
