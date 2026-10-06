# macOS 原生 UI、安装回归与合并打包门禁

2026-10-06，Asia/Shanghai。承接 [IMP-18](../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-18.md) 与 [Windows 双权限首屏复验](06-Windows普通与管理员首屏复验.md)。用户授权同步、继续 macOS 回归、回写证据、提交推送；发布另行确认。

**固定 9262a5b 的 macOS 原生首屏、Tauri 回读、偏好重启恢复、隔离应用副本升级和真实官方包安装往返通过。正式安装器、托盘/代理完整生命周期及签名 OTA 门禁仍开放；本轮未合并 main 或发布。**

## 同步与构建身份

Mac fetch 时 origin/docs/governance-main 已包含 Windows 文档提交 **0075a81c9498a097bdee7e2e6b2b6d006bd4236e**，原文档目录已 fast-forward 接收。06 中认证阻断保留为 Windows 当时快照；传输已经完成，无需重复 cherry-pick。

代码在独立工作树固定 **feature/0.1.8-windows-fixes@9262a5bedbb1a5f454f32afb7418b959fa8c721f**，本轮未改实现；main 仍为 **5a0859ad2e6d44f4ca5751eedfb5a1b7eb8c5d09**。文档在 docs/018-macos-regression 工作树产生，提交后推送到 docs/governance-main，避开共享目录中其他任务尚未提交的文档改动。

环境为 macOS 27.0.1 / 26A434 / arm64，Node 26.8.1、npm 11.19.0、Rust/Cargo 1.95.0、tauri-cli 2.11.4。前端 npm ci 与生产构建通过，cargo tauri build --bundles app -- --locked 成功生成本地 .app。

复用此前 Mac 验证的未跟踪 Cargo.lock，SHA-256 为 b03242c077b3b27792655210ee8c08b344dc0818aae3c45c04f78d37d1e6affb；Tauri 2.12.1 / tauri-runtime-wry 2.12.1 / wry 0.57.0。这是本次解析身份，不宣称新克隆自动得到同一锁文件。最终出包必须记录并固定实际依赖解析。[依赖](../../../../../../../../.adg/evidence/MAC-018-20261006/dependencies.json)、[环境](../../../../../../../../.adg/evidence/MAC-018-20261006/environment.json)、[构建日志](../../../../../../../../.adg/evidence/MAC-018-20261006/app-build-pinned.log)。

原始候选主程序 SHA-256 为 **6c3ea8d719706c2cf8caa61e1c63c6beb6741f7e4a824b0888f9ce2647886ffc**；在独立沙箱存活 76.57 秒，日志回读 0.1.8 与完整 9262a5b。此项只证明启动，首屏另有 UI 实证。[原始程序观察](../../../../../../../../.adg/evidence/MAC-018-20261006/exact-process.json)。

## 原生 UI 实测

为保留日常 0.1.7，测试副本使用独立 bundle identifier、显示名称与 ad-hoc 重签；HOME、OPENCODEX_SANDBOX_ROOT 都指向测试目录。重签改变文件摘要与签名/link-edit 信息，不能把包装副本当作正式制品；Mach-O 所有有文件内容的 section 均与原候选相同。[副本身份](../../../../../../../../.adg/evidence/MAC-018-20261006/candidate-identity.json)、[section 比对](../../../../../../../../.adg/evidence/MAC-018-20261006/candidate-section-identity.json)。没有改测试副本的前端或后端代码。

界面操作在真实 macOS 窗口/WKWebView 执行，使用无官方运行时的新根，Node/npm 通过明确的测试 HOME 候选链接发现。

| 实测 | 结果与证据 |
|---|---|
| 首屏与桥接 | Vue 概览显示 Node/npm 已发现、OpenCodex 未发现，2/3 条件及安装指引；关于页回读 v0.1.8、macOS · Tauri v2；安装配置回读独立数据根和内部 OPENCODEX_HOME。[首屏 AX](../../../../../../../../.adg/evidence/MAC-018-20261006/native-first-screen.ax.txt)、[首屏截图](../../../../../../../../.adg/evidence/MAC-018-20261006/native-first-screen.png)、[关于 AX](../../../../../../../../.adg/evidence/MAC-018-20261006/native-about.ax.txt)、[关于截图](../../../../../../../../.adg/evidence/MAC-018-20261006/native-about.png) |
| 偏好持久化 | 原生设置把缩放从 100 改为 110 并显示保存成功；结束该测试 PID、重启后原生控件恢复 110。[回读](../../../../../../../../.adg/evidence/MAC-018-20261006/native-restart-preferences.ax.txt) |
| 路由与恢复 | 扩展、诊断页渲染正常；受控异常显示兜底条，点击重新加载后原生树恢复诊断页。[扩展 AX](../../../../../../../../.adg/evidence/MAC-018-20261006/native-extensions.ax.txt)、[异常 AX](../../../../../../../../.adg/evidence/MAC-018-20261006/native-error-recovery.ax.txt) |
| 安装确认 | 私有前缀在测试数据根；未确认位置与源时下一步禁用。只检查向导，未执行 GUI 最终安装。[AX](../../../../../../../../.adg/evidence/MAC-018-20261006/native-install-confirmation.ax.txt)、[截图](../../../../../../../../.adg/evidence/MAC-018-20261006/native-install-confirmation.png) |
| 离线选择器 | 原生 open-panel 显示“选择 OpenCodex 离线包”；定位测试目录，未选包时 Open 禁用；取消选择器及向导成功。[观察摘录](../../../../../../../../.adg/evidence/MAC-018-20261006/native-file-dialog.json) |

文件选择器 JSON 是本轮 CUA 观察的摘录，非原始 AX 文件；没有归档选择器中的其他个人目录。窗口关闭/托盘重开未取得可独立判定的证据，**不列为通过**，留给完整生命周期验收。

构建仍有 STATIC_VCRUNTIME deprecated 警告，本地启动仍有 sandbox_extension_issue_file_to_process 警告；实际渲染与 bridge 已核验。日志保留，不推导为正式签名/Gatekeeper 通过。

## 应用副本安装与升级

新安装实测将候选 .app 复制到独立测试安装目录，并用新数据根启动。升级实测复制已安装 0.1.7 到另一个测试目录，以同一测试 bundle identifier/数据根运行；保存原副本后，把测试安装位置换为 0.1.8 候选再启动。

原生关于页从 v0.1.7 变为 v0.1.8；125% 缩放与浅色主题恢复。0.1.7 已初始化的 manager-state/preferences.json 和 backups、exports、opencodex-home 下三个合成测试文件，升级后 SHA-256 全部一致。[文件比对](../../../../../../../../.adg/evidence/MAC-018-20261006/upgrade-checks.json)、[前版本](../../../../../../../../.adg/evidence/MAC-018-20261006/upgrade-before-about.png)、[后版本](../../../../../../../../.adg/evidence/MAC-018-20261006/upgrade-after-about.png)、[偏好截图](../../../../../../../../.adg/evidence/MAC-018-20261006/upgrade-after-preferences.png)。

这证明本地 .app 副本替换与选定数据保留；**不代替 DMG 安装、公证隔离属性、正式身份的钥匙串行为、全量配置迁移或签名 OTA**。没有拿日常用户数据做实验。

## 真实托管安装回归

全新 HOME/冷 npm 缓存实际运行已有 runtime_managed_real 集成测试；未设置保留安装模式，也未把跳过算通过。结果 **1 passed / 0 failed / 0 ignored**，33.01 秒；真实安装与入口 --version 均为官方 **2.78.0**。

覆盖 registry 下载、入口版本/托管来源回读、一级卸载及备份、真实 npm pack 离线包导入与探针、系统保护目录/坏包拒绝、外部改写后拒绝卸载、临时区清理。[命令与摘要](../../../../../../../../.adg/evidence/MAC-018-20261006/verification.json)、[完整日志](../../../../../../../../.adg/evidence/MAC-018-20261006/runtime-real.log)。本轮未重跑整套后端；同源码历史/CI 数量保持独立。此 1 项不代表 GUI 安装到官方面板及代理启停全流程通过。

## 合并与打包门禁

| 阶段 | 必须满足的条件 | 当前边界 |
|---|---|---|
| 合并前 | 审阅 9262a5b 相对 main 的差异及原 0.1.7 回归；保持全部现有测试、无 HOME、Vue 渲染、Tauri invoke、来源匹配和诊断门禁；共享文件一致 | 特性测试/CI 有证据，未合并 |
| main 集成 | 实现仅合 main；CHANGELOG.md 只在 main 写实际 0.1.8 范围、Windows IPC 未支持等限制；docs/.adg 不随实现合入 | 本轮不改 main |
| 最终 SHA | 最终合并提交重跑应有 CI、本机原生和最终可执行 smoke；记录 OS/权限、工具链、依赖锁、来源及制品摘要 | feature 通过不冒充最终制品通过 |
| macOS 正式包 | 正式身份签名、公证、Gatekeeper；DMG 新安装及 0.1.7 升级；Dock/托盘、关闭重开、数据/偏好/配置/钥匙串保留及失败恢复 | 本轮只过隔离 .app 复制和选定数据检查 |
| Windows 正式包 | NSIS/MSI 新装/升级，Win10/11 普通/管理员、DPI、桌面/开始菜单/任务栏/快捷方式图标及缓存，卸载数据边界 | Server 2025 双权限 EXE 首屏不能替代 |
| 真实生命周期 | 隔离官方代理启动/停止/重启、面板加载、窗口关闭/托盘退出、状态/端口释放；macOS CLI 实测；Windows CLI/IPC 明确未支持 | 安装探针已过，完整链路待验 |
| 签名 OTA | 测试副本 0.1.7→0.1.8 的签名/通道/版本判断、下载安装重启及配置/偏好/数据保留；错误签名、失败/回退按现有契约验收 | 未执行，复制替换不能替代 |
| 发布 | 最终身份各门禁有证据，发布范围/制品可审阅；标签、发布、更新元数据、通道晋升另行确认 | 本轮无发布授权 |

按上述顺序区分 main 集成与实际安装制品验收。正式安装器、发布及全局同步依项目规则单独确认；局部通过不降低门禁。后续继续使用测试副本与隔离根，不覆盖或停掉日常 0.1.7。

## 清理与归档

测试 PID 58125、58265、58267 均已退出，退出前未监听 TCP。日常 /Applications/OpenCodeX Desktop.app 仍为 0.1.7；二进制/Info.plist 摘要、PID 83345、2026-10-05 07:18:00 启动时间都一致。只证明实际核对项，未补造日常全量数据前置摘要。[基线](../../../../../../../../.adg/evidence/MAC-018-20261006/daily-baseline.json)、[后置检查](../../../../../../../../.adg/evidence/MAC-018-20261006/postcheck.json)。

[清单](../../../../../../../../.adg/evidence/MAC-018-20261006/manifest.json) 记录文件摘要；PNG 原字节保留，文本家目录脱敏为 <USER_HOME>，原输入摘要留存。没有归档二进制、官方包、完整缓存、用户配置或源码。Windows 证据与原 CI 失败保留，不改变 06 对原失败根因的限制。
