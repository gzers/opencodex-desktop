# 0.1.10 macOS 应用菜单、候选与退出观察

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182；源码 `main@03e0ad942c16fe2ef75473eb6693012b3ae684ac`。这里只记录局部执行证据，完整原生、性能、更新与发布门禁仍未闭合。

## 修复与定向验证

`fix(macos): 恢复应用菜单退出与隐藏入口` 已提交推送。macOS 首个应用菜单恢复原生隐藏、隐藏其他、显示全部与退出，Windows 菜单处理保持原有逻辑。fmt、tray 单元 3 项、tray integration 4 项、window_close integration 6 项和 workspace all-targets/all-features Clippy `-D warnings` 均退出 0；定向测试共 13 项。命令及耗时见 `test-results.json`，原始输出见 `1.raw.gz` 至 `4.raw.gz`，可读视图见对应 `.log`。

## 候选身份与安装边界

干净 main 工作树执行 `cargo tauri build --bundles app,dmg -- --locked`，退出 0、耗时 121.254 秒。应用版本与构建号均为 0.1.10，arm64，原 EXE 内嵌完整源码 SHA。

| 原件 | 字节数 | SHA256 |
|---|---|---|
| DMG | 4,384,633 | `1bc69e1b42676dfff93c4c402ffe8bb5affd234ade924ed5f4b33600a6fa3ab4` |
| EXE | 7,897,776 | `196596e388f4f57d63c66c1067d70ac1be1879ec2a9096f657efbe2dc4590c0f` |

原 `.app` 与 DMG 留存在 `/private/tmp/ocx-0110-03e0-candidate-3wpphyvk/`；这是本地临时留存路径，不是用户下载地址。DMG 容器校验通过；原 `.app` 严格签名退出 1：`code has no resources but signature indicates they must be present`。它仍为 linker ad hoc，正式签名和公证未通过。`result-summary.json` 的 `ui_launched: false` 是构建完成时的快照，后续启动另见下列记录。

只读挂载 DMG，核对其中 EXE 与原件哈希一致后，复制到候选目录的 `test-install/` 并卸载卷。仅测试副本改 identifier 为 `com.gzers.opencodex.desktop.sandbox011003e0`、ad hoc 重签；测试副本严格签名通过不能替代原件失败。副本 EXE 哈希 `a9de09589d4631df9e5046332fb263e33d776c64cddf133410d2bd6091bf0151`。这只是隔离复制安装，不是 `/Applications` 真实安装升级或 Gatekeeper / 公证验收。

## 原生退出与同根重启

| 动作 / 观察 | 结果与证据边界 |
|---|---|
| 清理旧实例 PID17410 | 精确 SIGTERM；记录于 `old-instance-cleanup.json`，不算正常退出通过 |
| 首次启动 PID41949 | 测试副本使用新沙箱根 `test-data`，完整源码身份与锁 PID 一致，见 `launch-1.json` |
| 点击原生应用菜单退出 | 菜单 AX 含 hide / hide others / show all / terminate；原 PID41949 消失，新根锁可非阻塞取得，未删除锁文件 |
| 退出后 UI 状态读取 | 观察期间出现新 PID42076，沿用了旧沙箱根 `/private/tmp/ocx-0110-dmg-install-tuk3m7/data`；不能算同新根重启。初读 `after-menu-quit.json` 的 false 与后续精确原 PID / 锁观察同时保留 |
| 对 PID42076 按 ⌘Q | 无信号 monitor 观察原 PID 退出，随后 AX 报 procNotFound / -600；两个测试根的锁均释放，测试进程数变为 0，未删除锁文件 |
| CLI 同新根重启 | 同一测试副本再次使用 `test-data`，PID42681，锁 PID 与环境一致；同根重启成功，前台只保留这一测试实例 |

退出后 UI 观察伴随重开，原因尚未独立定位；不能把它直接归因为应用自动重启或工具已确认的行为，也不能据此记完整正常生命周期通过。已确认的是原 PID 的退出与锁释放，以及最后 CLI 指定同根重启。⌘Q 后原始完整异常没有保存，`cmd-q-ui-observation-note.json` 明确只保留上下文记录；退出另有独立进程与锁证据。

`menu-quit-lock-observation.json`、`cmd-q-original-pid-observation.json`、`after-cmd-q-processes-and-locks.json`、`same-root-restart.json` 与 `current-instance-snapshot.json` 分别记录上述观察。日常 `/Applications/OpenCodeX Desktop.app` 的 PID75764 保持运行，未操作日常安装或真实数据。启动日志原始快照为 `test-data-app.raw.gz`。

## UI 证据与 CI 快照

`application-menu.ax.txt` 记录真实原生菜单。该菜单截图返回 unavailable，没有保存可用截图；不能记截图通过。`same-root-restart.jpg` 为真实返回的 JPEG，初始误用 `.png` 后缀，已仅改后缀、没有重编码；`ui-evidence-provenance.json` 记录字节哈希。`same-root-restart.ax.txt` 是 AX diff，不是完整树。

2026-10-11 07:13 UTC，[CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 与完整 03e0 SHA 匹配：frontend / release-tools / backend 成功，backend-windows 正在后端回归，整体 in_progress，不能记完整通过。旧 [Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 属于 `3d856fb561aa8dd21d0dd1347d8736c7b704f3d7`，正在 NSIS / MSI 构建；不代替 03e0 候选。观察时间与原始字节见 `log-provenance.json` 的 0713 快照条目。

原日志与 API 快照按原始字节 gzip 保存；可读日志只规范行尾，可读 JSON 只重新序列化。`log-provenance.json` 与 `checksums.json` 提供对应哈希。大型制品留在上述目录，仓库只保存可审阅证据。

IMP / TASK 保持 in_progress，`release_authorized: false`、`release_accepted: false`。后续仍需同 SHA 全 CI 与 Windows 安装器、真机双权限 / 物理 DPI / 默认数据根、完整 macOS 原生、0.1.9 配对性能与真实周期、真实更新与事件生命周期、正式签名及发布审阅；本轮未创建标签、Release 或晋升 stable。
