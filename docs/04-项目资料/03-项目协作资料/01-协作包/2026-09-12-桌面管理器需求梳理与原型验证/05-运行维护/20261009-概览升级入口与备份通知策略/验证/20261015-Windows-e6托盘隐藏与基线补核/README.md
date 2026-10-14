# Windows e6 托盘隐藏与官方 0.1.9 基线补核

2026-10-15，Asia/Shanghai。绑定 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182；本记录为未完成的原生测试阶段证据，不是版本验收回执。

源码 `e6a4b32f8b60e362911cc02bc67fc0a2f82e7963`，候选 run `38591604133`，MSI 只读提取的产品 EXE SHA256 `2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3`。同 SHA CI、制品、精确安装载荷的准入沿用上一段证据；没有启动新 CI、制作新候选或在宿主执行安装器。

## 普通实例及托盘阶段

00:45:43 启动唯一隔离普通实例 PID `56448`，创建时间 `2026-10-14T16:45:43.2096899Z`，真实产品令牌 `elevated=false/elevationType=3`，主 HWND `2229478`。本段配置根为代码 worktree 下 `test/runtime/foreground-tray-admin-e6-20261015/data`，不沿用日常状态；自动面板、自动检查、启动清理关闭，`keep_proxy_on_close=true`。

- 首屏可见，单系统标题栏，无应用菜单，DWM backdrop 为 2。实际远程会话 metric=1、native DPI=192，不算物理 200% 门禁。用户要求本轮不用处理 DPI，物理缩放仍待验。
- `Super_L+z` 后见数字提示，但状态捕获报 `no monitor found for window`，没有独立 Snap 窗口证据；不判 Snap 通过。
- 首次关闭点击后主窗口仍可见，原件保留；随后真实 Sky `Alt_L+F4` 使同 PID 主 HWND 隐藏，产品仍存活，DWM backdrop 为 1。隐藏偏好和日志回读保存。
- 隐藏后的 `Process.MainWindowHandle` 选到了 13×13 托盘 helper。该误读原件保留；只读方法改为持续绑定首次主 HWND，并重新核验其所属 PID、创建时间和实际产品令牌，未改产品源码。
- Sky 未暴露系统托盘和隐藏主窗口，按公开入口恢复报 `window id 2229478 was not found`。没有用原生显示窗口、其它 UI 输入工具或第二次启动绕过。
- 用户随后提供托盘图标截图；07:24:51 独立只读检查仍为原 PID、原创建时间、原主 HWND 隐藏。裁切图标本身不能证明 EXE 身份，身份由进程令牌及原候选绑定另行证明。

07:27:49 的保护检查确认本机 nativeCount=1，管理员产品尚未启动，普通实例未正常退出。本次恢复只做后台读取，没有执行前台输入。原前台 15 分钟时段已过，截图没有被解释为任意延长时段的授权。下一步需要用户从托盘“打开主界面”实际重开，取得同 PID 恢复证据并正常退出；实例归零后才进入顺序管理员对照，UAC 由用户确认。管理员控制器已准备但未执行。

此实例的自动检查关闭，存活时间不能作为真实 stable 24h / beta 6h 到期测试。未有新 30 分钟状态错误退避或真实周期证据。

## 官方 0.1.9 配对基线

官方 release `407422226` 的 MSI asset `623593980` 给出的摘要，与本地包 SHA256 `b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a`、大小 4,030,464 bytes 一致。标签 API 读取失败原件保留；独立 Git 远端读取获得 annotated tag `e5deeed6512c2cdaf7662a88f831bd0702a7c4ca`，peeled commit 为原官方基线 `de6862d40389d527527b9db8ef52e186a58b625f`。

原只读 MSI 提取回执与现存两个载荷逐项复核：产品 EXE 7,619,584 bytes，SHA256 `626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608`；ocxd 162,816 bytes，SHA256 `14cfc0a70470f54dbe959a5ee50c7577fe8a3e32c338077a2eccf6a22211b7fd`。未运行基线应用、安装或卸载，没有新增性能样本。

仍按原合同：每版冷/热各至少 20 次，启动可交互 p95 增量同时 ≤100ms、≤5%；每版闲置至少 600s，原计量口径的 CPU 增量 ≤0.5 百分点、RSS 增量 ≤10MiB；检查无 >50ms 主线程任务；fresh-cache 100 次切页/10 次重启零自动请求及查询子进程。真实 stable 24h / beta 6h、休眠恢复及跨启动另取证，不能以本次制品补核代替。

## 原件与未覆盖

证据目录：`.adg/evidence/OCX-0110-20261015-WIN-FG-TRAY-ADMIN/`。

`checkpoint-manifest-20261015.json` 固定 30 份历史文件；`verify-checkpoint.py` 校验该快照的摘要、普通令牌、主窗口隐藏及保护状态，显式返回 `fullNativeGatePassed=false`。后续新增原件为 append-only，需独立校验，不倒改此快照。

`baseline-git-proof-20261015/manifest.json` 固定 5 份补核原件；`verify-baseline-git-proof.py` 独立核对官方发布摘要、远端标签、提取回执和未执行边界。首次 API preflight 的失败结果保持，不以补核成功覆盖。

本证据目录的局部 `.gitattributes` 禁止文本换行转换，使原件及清单摘要在 Windows/macOS 检出时保持同一字节；不修改全局 Git 配置。

日常 EXE 及旧隔离 preferences/data-root/policy/schedule 四份摘要仍与原保护基线一致，原 TASK 3878 bytes/SHA256 `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93` 未改。用户已有删除不纳入提交。

托盘重开/退出、管理员产品令牌、Snap、物理 DPI、安装/升级卸载保留、实际更新进度/重启、备份恢复轮换固定、树表和事件、状态退避/真实周期及配对性能仍未完成。历史 bd2 query 失败没有取得实际失败时间窗 ETW，e6 PR 三项 query 失败和 Mac CPU/RSS 超预算保持。Mac 仍未取得原生平台交接；不关闭 TASK/IMP、自动合并、签名、晋升或发布。
