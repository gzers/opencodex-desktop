# Windows e6 普通权限前台子集

2026-10-14–15，Asia/Shanghai；关联 IMP-20 / TASK-OPENCODEX-DESKTOP-182。代码固定 `feature/0.1.10-windows-query-etw@e6a4b32f8b60e362911cc02bc67fc0a2f82e7963`。本轮验证同 SHA 桌面候选在远程会话中的普通权限首屏、系统标题栏、材质状态和正常退出，完整原生门禁继续开放。

## 前台授权和实际窗口

用户明确“可以开始 Windows 前台验证，我现在有 30 分钟。”，原窗口按 2026-10-14 23:39:28 至 2026-10-15 00:09:28 记录。原开始时本机没有 e6 桌面候选；此前把候选准备留到前台开始后，造成窗口主要用于构建，这是准备不足，不记为已完成 UI 验证。Windows 显示设置提示当前远程会话不能更改显示器设置；用户随后“**不用管dpi的问题**”，本轮跳过物理缩放，原物理 DPI 门禁保留待验。

00:00:05.9984359 创建唯一普通实例 PID 73920。首次界面工具调用遇到用户物理 Escape，立即停止 Computer Use，没有继续输入或绕过；该时刻只完成制品/普通令牌准入，没有界面验收。用户再明确“**继续前台验证**”，重新核实仍为同 PID、映像和创建时刻，按 00:16:41–00:31:41 的短窗口继续，没有启动第二实例。

界面操作仅使用 Windows Computer Use 的 `node_repl + @oai/sky`；读取进程令牌、窗口几何、DWM 属性的 PowerShell 方法不注入输入。没有运行含全局透明度注册表写入及 Win32 点击/按键的旧 `windows-native.ps1`，没有改系统 DPI、透明度、防护或全局 PATH。

## 同 SHA 候选准入

只请求一次隔离 Windows candidate workflow：run `38591604133`，实际 head e6，最终 `completed/success`。锁定依赖回归、NSIS/MSI 打包、隔离 runner 安装/最终 EXE 首屏/卸载及上传步骤成功；构建期间只观察同一 run，没有重派。曾准备读取已上传制品以避免只等待 post-cache，但实际下载时整 run 已成功，不把运行中任务当作终态。

候选 artifact `11909280359`：7626639 bytes / SHA256 `b8fe4304da47f29bc6c50ae73cae4433ed4792e5d4ed4e3ad85c35afd826161d`；evidence artifact `11909105485`：1691283 bytes / SHA256 `9267a2b41bc31e4aa3aa01ef45c4a2f88a1fd7719a432c723e6dd3c14855ad4e`。两份均匹配 API digest，ZIP CRC/条目边界通过；GitHub credential 仅用于 API，签名重定向下载未带 Authorization。

MSI 4567040 bytes / SHA256 `1f79fa7974d8cfc9c37d55a88ba4b92138e819d4fe1284dc62ca81e6b41383e9`；NSIS 3270239 bytes / SHA256 `c0a5a9be5ebe6eb44062dc49a5e68b28e96e51acdb5712c7b002f8cbcac6cb18`。只读 MSI database/CAB 提取桌面 EXE，没有执行本机安装器或卸载；提取 EXE SHA256 `2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3` 匹配 runner 精确 MSI marker/payload 回执，版本 0.1.10。包内 environment、原始 installer 回执及本机 startup log 均绑定 e6。

原 e6 PR CI `38583689702` 的三个 version fixture 失败和更早 bd2 七 query 失败仍保留，不能用新 candidate success 撤销或推定根因。候选不是发布、签名或版本通过授权。

## 实际行为及边界

测试根为新 `test/runtime/foreground-e6a4b32-20261014/data`，显式 sandbox/boundary；fresh source-default 偏好关闭 auto_panel、launch_with_codex、自动更新和 startup_cleanup，不制造缓存/截止时刻。本机未配置 Node.js，首屏如实显示“缺少 Node.js”；未安装官方包或执行 FZ50 生命周期脚本。

| 检查 | 真实证据及结果 |
|---|---|
| 普通令牌 | PID 73920 的持有句柄/API 映像及 creation；`elevated=false/elevationType=3`，多时刻回读一致 |
| 首屏源码 | startup `version=0.1.10 commit=e6…`、真实 WebView 截图、非空系统标题 |
| 系统标题栏 | 单一 `OpenCodeX Desktop` 标题栏，应用菜单未附着 |
| 最大化 | Sky 真实按钮操作后 `maximized=true`，客户区由 1180×760 变为 3200×1121 |
| 还原对照 | 首次 element-index 点击仍最大化；后续当前截图坐标 `(3128,13)` 点击成功，客户区精确回到 1180×760，外框也回到原值 |
| 最小化及恢复 | `minimized=true/foreground=false`，随后恢复为同实例 `minimized=false/foreground=true` |
| 材质状态 | 活动 DWM backdrop raw=2，最小化时 raw=1，恢复 raw=2；应用日志对应 `mica/mica`→`solid/inactive`→`mica/mica`，仅支持本次远程会话的状态证据 |
| 正常退出 | 真实设置 UI 将隔离 `shell.keep_proxy_on_close` 从 true 改为 false，文件回读；点击系统关闭按钮后 desktop/ocxd 进程 0，无 agent forced termination，未保留 exit code |

Sky element-index 还原及设置导航点击没有达到目标状态，截图坐标点击生效。原索引尝试、原生几何及坐标对照全部保留；不据索引输入差异直接认定产品还原故障，也不把失败尝试删除。

只读取证脚本最初以字符串比较原创建时间，PowerShell `ConvertFrom-Json` 将旧字段解析成 `System.DateTime`，导致误报 PID reuse。原审阅记录保存；修正为 UTC ticks 加持有句柄原生创建时间核验后一致。此修正只在新证据方法，不修改产品或老冻结原件。

00:31:23 后检查本机 native count=0，日常 EXE 及旧隔离 preferences/root/policy/schedule SHA 前后相同。测试写入仅新隔离根；没有对日常软件安装、卸载、迁移或替换，没有新自动周期。

## 仍开放

本轮未测试管理员产品实例、托盘实际重开/退出、Snap、浅深全主题及三档矩阵、安装目录默认 data/自定义根/升级卸载保留、更新详情/真实进度、备份恢复/轮换/固定、统一树表对齐、事件解除/通道隔离、最终源真实 30 分钟状态退避、真实周期、官方 0.1.9 配对性能。物理 DPI 按用户本轮方向跳过；不把观察到的远程 nativeDpi=192 当成物理 200% 验收。

证据 `.adg/evidence/OCX-0110-20261014-WIN-FG-REQUEST/` 的原回执、状态、截图、方法和清单可独立校验；二进制 ZIP/只读提取 EXE 保留在代码 worktree 忽略目录。所有原预算、八项 required checks、TASK revision2/in_progress/pending、macOS CPU/RSS failed、历史 query 失败及 Mac 依赖保持；不关闭、自动合并、签名、晋升或发布。
