# OCX-0110-20261011-MACOS-LIFECYCLE

## 证据定位

- 项目：opencodex-desktop
- 代码分支：main
- 代码提交：861fdeea39d1c45fa2beddb2cba6659b67735d9f
- 文档分支：docs/governance-main
- 记录时间：2026-10-11（Asia/Shanghai）
- 候选版本：0.1.10，macOS arm64
- 测试数据根：/tmp/ocx-0110-dmg-install-tuk3m7/data

## 已观察结果

1. 从 0.1.10 arm64 DMG 进行临时隔离安装并启动成功，真实首屏可读。
2. 候选使用独立 bundle identifier 和临时数据根，未替换 /Applications 日常安装。
3. 原生 UI 操作观察到窗口关闭后进程继续保活；随后通过应用退出路径退出，并重新启动成功。
4. 重新启动后的候选进程为 PID 86306，日志和 manager-state/app.lock 均生成；当前锁与进程匹配。
5. macOS Saved Application State 的恢复提示已选择 Don’t Reopen。
6. 当前仅保留一个隔离测试实例；日常安装 PID 75764 保持运行且未操作。

## 当前实例复核

processes.txt、app-lock.json 和 app.log 是当前保留实例的最小非敏感复核材料。数据根已观察到 backups、cache、exports、logs、manager-state、opencodex-home 和 sync-state 等目录；完整文件名见 data-root-inventory.txt。

## 边界与未闭合门禁

- 本证据只证明 macOS arm64 候选的临时隔离安装与局部生命周期观察，不是正式安装完成证书。
- 候选为 ad hoc / linker-signed，未完成正式签名、公证和发布身份验收。
- 原生标题栏、玻璃 / 背景光、动效、更新 / 重启、升级 / 卸载、自动检查性能与稳定周期仍待验收。
- Windows 0.1.10 制品、普通 / 管理员权限、安装路径、DPI、云母、原生标题栏、动效和透明下拉框仍需设备证据。
- 不回写稳定核心事实，不创建 Release / tag，不晋升 stable，不替换日常 0.1.7 / 0.1.9 安装。

历史的强制终止和残留旧锁记录仍保留在 OCX-0110-20261011-MACOS-ISOLATED-STARTUP；本记录只补充后续隔离候选的局部成功观察，不能覆盖历史失败。
