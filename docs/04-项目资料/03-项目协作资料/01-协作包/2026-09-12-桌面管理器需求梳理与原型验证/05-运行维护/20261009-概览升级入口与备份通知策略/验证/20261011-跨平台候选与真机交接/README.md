# 0.1.10 跨平台候选与真机交接

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。本文保存候选构建和后续验收交接，不是版本通过证书或稳定核心事实。代码归属 main；本文和证据归属 docs/governance-main。

最新源码基线已推进至 `main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27`，树表对齐 / 刷新焦点、完整 CI 及同源码候选见下文“main@369ab12d 最新候选与树表验证”。ADCC / 3D856 / 03E0 段落保留为历史轮次，已通过项不能自动替代最新 SHA 验收。

## main@adccca8e 的结果

源码身份为 `adccca8e92d28c95d4fa83c11d994cb6bf3445f6`。在干净工作树执行 `cargo tauri build --bundles app,dmg -- --locked`，退出码 0。

| 项目 | 实测结果 | 边界 |
|---|---|---|
| macOS 包身份 | 版本 / 构建号均为 0.1.10，arm64；EXE 包含完整源码 SHA | 只验证身份 |
| DMG | 4,383,725 bytes；SHA256 `059f85310a1b400d0755a524234e83d0eb066ad4854fbba384a1816f0ebc779d` | `hdiutil verify` exit 0 |
| macOS EXE | 7,897,760 bytes；SHA256 `5e54dee8ab172f1321e54da0ba14c1d9b113189de9c61a64fa58236764c3acd3` | 与其他 SHA 候选不可混用 |
| 严格签名 | `codesign --verify --deep --strict --verbose=2` exit 1；资源未封装，linker ad hoc | 未通过；当前宿主 0 个有效签名身份，公证未验证 |
| CI 38117383632 | frontend / release-tools / backend 成功；backend-windows 失败；build 跳过 | 不准入下一步 Windows 候选构建 |

原始摘要、命令和哈希清单见 `.adg/evidence/OCX-0110-20261011-ADCC-MACOS-CANDIDATE/`。DMG 留存于 `/private/tmp/ocx-0110-adcc-candidate-cSZKEZ/OpenCodeX Desktop_0.1.10_aarch64.dmg`，未安装或启动。本地留存路径不是用户下载地址。构建输出中的原 `.app` 可被后续构建覆盖，追溯该候选时使用对应 DMG 与哈希。

[失败 CI](https://github.com/gzers/opencodex-desktop/actions/runs/38117383632) 的 Windows Clippy 通过；Windows 后端库回归为 **485 passed / 1 failed / 4 ignored**。失败项为 `cli_binding_owns_runtime_and_exclusive_storage_until_terminal_audit`：运行操作锁已经可以取得时，未保存绑定的存储门仍冻结。运行操作锁是 worker 局部变量，而绑定门仍为闭包捕获，释放顺序存在窗口。保留这轮失败，修复需要重新取得 Windows 回归和候选证据；不删除断言、不增加重试掩盖失败、不降 Clippy 门禁。

## main@3d856fb5 的释放顺序修复基线

`main@3d856fb561aa8dd21d0dd1347d8736c7b704f3d7` 已提交推送：让绑定守卫在工作线程内先结算，随后才释放运行操作锁；保存后的待重启绑定仍冻结，未保存绑定正常解除。未删除或重试原竞态断言。

本地 fmt、IPC 定向 44 项、Rust workspace integration-test 34 组（861 passed / 0 failed / 6 ignored，其中库 694 passed / 4 ignored）、严格 all-targets / all-features Clippy 均退出 0。忽略项包括真实网络 / 钥匙串、子进程辅助测试和需要原生主线程的托盘集成，完整名称保留在回执，不记为这些场景通过。命令原件、原失败日志及哈希见 `.adg/evidence/OCX-0110-20261011-IPC-BINDING-RELEASE-ORDER/`。

新 [CI 38118226895](https://github.com/gzers/opencodex-desktop/actions/runs/38118226895) 的 headSha 已核对一致。2026-10-11 06:56 UTC 核查已完成，frontend / release-tools / backend / backend-windows / build 五个 job 全部成功。同 SHA [Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 已仅触发一次，headSha 与 main 一致，观察时为 in_progress；尚未取得安装器通过证据。最终 CI 和候选触发快照见 `.adg/evidence/OCX-0110-20261011-3D856-CI-AND-CANDIDATE-DISPATCH/`。06:41 / 06:53 UTC 的历史快照和失败轮次另存保留，不覆盖；不得使用旧 c654 安装包代替。

## main@3d856fb5 的 Windows CI 原生启动结果

Windows job 已完成严格 Clippy、后端回归、release EXE 构建及最终 EXE smoke。artifact 为 `windows-startup-evidence`（ID `11694523976`）；EXE SHA256 为 `ab023fa342b360d39c0298c85b14a844d716877d44ea8f0109369bcfda73902b`，启动回执的完整源码身份与修复提交一致。30 秒观察中窗口存在且响应，WebView 主 UI 已挂载，关于信息为 0.1.10 / Windows，隔离数据根已初始化。未发现 OpenCodex 时显示准备态，CLI 未启用。

回执、关于信息、runner 环境、窗口截图、artifact 元数据、新 CI 快照及 SHA256 清单见 `.adg/evidence/OCX-0110-20261011-3D856-WINDOWS-CI-SMOKE/`。JSON 原始字节另存 `*.raw.gz`；可读 JSON 仅重新序列化，`source-provenance.json` 记录原始字节摘要。旧快照和失败轮次不覆盖。

本结果只闭合 CI Windows job 和最终 EXE 启动检查，不是 MSI / NSIS 安装、真机双权限、物理 DPI、新装默认安装目录数据根、正常退出 / 锁清理、完整外观或性能通过证据。完整 CI 已在后续核查通过，同 SHA Windows 安装候选仍待收齐。

## main@3d856fb5 的 macOS 候选

在干净 main 工作树执行 `cargo tauri build --bundles app,dmg -- --locked`，退出码 0，耗时 116.398 秒。应用版本 / 构建号均为 0.1.10，identifier 为 `com.gzers.opencodex.desktop`，arm64 EXE 内嵌完整源码 SHA。

| 制品 / 检查 | 实测结果 | 验收边界 |
|---|---|---|
| DMG | 4,383,683 bytes；SHA256 `bbe9ef00f9ea5a46e69c2ae09b10b611b9757f18a0b8b4e080680696708eede4` | `hdiutil verify` exit 0 |
| 原 EXE | 7,897,760 bytes；SHA256 `af154f85fd7116f681b951c331bd1b99ad49f589ed8da1107e68d2179cd550c7` | 原 `.app` 完整复制留存，不与测试重签副本混用 |
| 严格签名 | exit 1：`code has no resources but signature indicates they must be present` | linker ad hoc；没有 TeamIdentifier / sealed resources，正式签名未通过 |
| 可用签名身份 / 公证 | 宿主 0 个有效签名身份；公证未验证 | 需要正式签名环境才能闭合 |
| 安装 / 原生 UI | 此候选未安装、未启动 | c654 旧实例观察不能作为该 SHA 的原生通过证据 |

原件和校验清单见 `.adg/evidence/OCX-0110-20261011-3D856-MACOS-CANDIDATE/`。原 `.app` 与 DMG 留存于 `/private/tmp/ocx-0110-3d856-candidate-2aw35kxm/`，本地路径不是下载地址；不得通过给原件重签覆盖原始失败记录。当前仍只有旧 c654 隔离测试实例；本轮尝试退出快捷键后该进程仍在运行，因此未启动第二个实例，也不记为正常退出 / 锁清理通过。

## main@03e0ad94 的 macOS 候选与原生观察

`03e0ad942c16fe2ef75473eb6693012b3ae684ac` 已推送，恢复 macOS 原生应用菜单隐藏与退出入口。fmt、13 项定向测试及严格全目标 / 全特性 Clippy 通过。干净源码构建 app / DMG 退出 0、耗时 121.254 秒，arm64，版本 / 构建号为 0.1.10，完整源码 SHA 内嵌 EXE。

| 制品 / 检查 | 实测结果 | 边界 |
|---|---|---|
| DMG | 4,384,633 bytes；SHA256 `1bc69e1b42676dfff93c4c402ffe8bb5affd234ade924ed5f4b33600a6fa3ab4`；容器校验通过 | 原件留存，不用于公开发布 |
| 原 EXE | 7,897,776 bytes；SHA256 `196596e388f4f57d63c66c1067d70ac1be1879ec2a9096f657efbe2dc4590c0f` | 完整原 `.app` 单独保留 |
| 严格签名 / 公证 | 原件签名 exit 1，资源未封装；公证未验证 | 测试副本 ad hoc 重签通过不能代替正式签名 |
| DMG 隔离复制安装 | 只读挂载、核对其中 EXE、复制测试副本并卸载卷通过 | 没有覆盖日常安装，不是系统目录真实安装升级验收 |
| 菜单退出 | PID41949 使用新 `test-data` 根，菜单原生 terminate 可用；点击后原 PID 消失、锁释放 | 退出后 UI 观察出现 PID42076 沿用旧测试根，来源未独立定位；不能记完整生命周期通过 |
| ⌘Q | 对 PID42076 按快捷键，无信号 monitor 确认退出；两个测试根锁可取得 | 该 PID 使用旧测试根，未把它记为同新根重启 |
| CLI 同新根重启 | 未删除锁文件，再启动新根为 PID42681，环境与锁 metadata 匹配 | 只保留这一测试实例；日常 PID75764 未操作 |

原件留存 `/private/tmp/ocx-0110-03e0-candidate-3wpphyvk/`。测试副本 identifier 为 `com.gzers.opencodex.desktop.sandbox011003e0`，EXE SHA256 `a9de09589d4631df9e5046332fb263e33d776c64cddf133410d2bd6091bf0151`；只修改该副本，不覆盖原件签名失败。旧 PID17410 精确 SIGTERM 清理不记正常退出通过。

证据见 `.adg/evidence/OCX-0110-20261011-03E0-MACOS-APPLICATION-MENU/`：原生菜单有 AX 记录但无可用菜单截图；同根重启有 JPEG 截图及 AX diff。⌘Q 后读取 AX 报 procNotFound / -600，完整原始异常未保存，只有上下文说明；退出另由进程监控和锁观察确认。初次退出后窗口仍出现的原始记录继续保留，不能覆盖为成功快照。

2026-10-11 07:13 UTC，[CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 的完整 headSha 与 03e0 一致，frontend / release-tools / backend 成功，backend-windows 正在回归，整体 in_progress。旧 [Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 仍在 NSIS / MSI 构建，属于 3d856 SHA；后续使用最新 SHA 候选，不能混用。这轮局部通过不解除物理 DPI、双权限、默认数据根、性能、真实更新 / 事件或正式签名门禁。

## 2026-10-11 07:32 UTC 最新局部验收与 CI

继续沿用 PID42681，一个隔离测试实例。已真实生成偏好备份 `bk_20261011072259_f9e7a1ac`，载荷 1371 bytes，SHA256 独立重算与清单相符；真实文件只有 preferences.json 与 backup-manifest.json。根 / 年默认展开、月份收起；实际箭头展开、键盘焦点至清单文件以及刷新保留展开层级已有 AX / 截图。无复选框场景视觉对齐只覆盖当前 macOS；复选框、半选、禁用、物理 DPI、刷新移除行时焦点恢复仍待验。

设置可进入共享更新中心及两类详情。管理器当前 0.1.10、可用 —，显示已是最新，缺说明且安装禁用；面板当前未知、可用 2.83.0，查询失败，缺说明回退与官方网页按钮可见，安装禁用。尚未执行网页跳转、安装或重启；准备态未发现 OpenCodex，不能替代概览就绪态入口或完整真实更新验收。

最新 [03e0 CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 的 frontend / release-tools / backend / backend-windows 成功，macOS build 仍运行，整体 in_progress。旧 [3d856 Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 已 completed / success，构建、隔离 runner 安装验证、上传成功；未下载独立核验安装器，也不能代替 03e0 候选。

原件见 `.adg/evidence/OCX-0110-20261011-03E0-NATIVE-BACKUP-DETAILS/`：15 组原生截图 / AX / metadata、备份校验、CI 原始 JSON 与 provenance / checksums。04 的文件名含 expanded 但该帧未展开，08 未证明焦点成功；不改写失败尝试为通过。未恢复、清理、固定备份或修改策略，日常安装未操作。

## 候选生成顺序（最新基线）

1. 已完成：统一树表首行对齐、刷新焦点修复已推送 main@369ab12d；[CI 38124355301](https://github.com/gzers/opencodex-desktop/actions/runs/38124355301) completed / success，五个 job 全部成功。
2. 已完成候选资产核验：2026-10-11 17:04（UTC+8）只触发一次 [Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287)，17:17:48 completed / success，headSha 与 main@369ab12d 一致。锁定依赖回归、MSI / NSIS 构建、隔离 runner 安装 / 首屏 / 卸载和上传成功；ZIP / MSI / NSIS 本地大小与 SHA256 核验通过。安装后 EXE 哈希仍为 runner 回执，真机验收待做。旧 03e0 留作历史，不能混用。
3. macOS 最新候选已构建并留存原件，隔离复制安装和原生树表局部观察见下文；只有一个测试实例。最新 SHA 的完整生命周期、默认根 / 真实安装升级及全密度 / 画质矩阵仍待验，正式签名未通过。
4. 完成下面的设备、性能和真实更新验收，补齐 TASK 所需检查，再进行发布审阅。候选工作流不创建 Release 或版本标签。

最新 CI / 查重 / 单次 dispatch 原件见 .adg/evidence/OCX-0110-20261011-369A-CI-CANDIDATE-DISPATCH/；macOS 原件与树表观察见 .adg/evidence/OCX-0110-20261011-369A-MACOS-TREE-CANDIDATE/。保留旧状态，临时 runner 安装 smoke 不作为 Windows 真机完整验收。

## 2026-10-11 15:44—15:50（UTC+8）后续验收

同 SHA Windows artifact 已下载并重算 ZIP / 安装器 SHA256，候选与证据 artifact 的 installer-verification.json 字节一致。临时 runner 的 MSI / NSIS 新装、最终 EXE 首屏与卸载回执均 pass；关于信息 0.1.10 / Windows、主 UI 已挂载、沙箱根初始化、准备态、CLI 关闭。最终 EXE 与预期 Tauri bundle marker 的哈希一致，这是 runner 回执，本机未提取安装后 EXE 独立重算。两个 artifact 预计 2026-10-25 15:49（UTC+8）到期。

| 安装器 | 本地 SHA256 |
|---|---|
| MSI，4538368 bytes | `1c803172d7cca9d294c011f40e0b58c1e057f33680b32cc29759f41e649e3527` |
| NSIS，3231828 bytes | `c6c0520c37d3957d66132f086c4f0fb85a9ed2df37d7e4b9fe02437781357cde` |

Windows 图片为 CDP WebView 捕获，左侧透明区域呈黑色、部分文字对比度低，原因未确认；应补真机整窗与 CDP 对照，不能直接认定云母外观通过。startup-diagnostics 中权限字段属于 PowerShell harness 当前 token，不是目标应用进程；后续双权限必须记录实际目标 token。沙箱启动与脚本最终强杀不能证明默认安装目录数据根、普通退出或锁释放。codeSigned / releaseAccepted 仍 false。

macOS 对原唯一测试 PID42681 原生 ⌘Q 后，独立确认进程退出且锁可取得；不删除 / 不截断锁文件。显式同根重启为 PID75489，锁重新由该 PID 持有；四次快照的原备份载荷与清单哈希不变。重启后设置回读原根、根 / 年展开、月份 10 收起，原记录 `bk_20261011072259_f9e7a1ac`（15:22:59 / 1.3KB）可见；日常 PID75764 和启动时间不变。只闭合同根隔离生命周期。原件见 `.adg/evidence/OCX-0110-20261011-03E0-MACOS-SAME-ROOT-LIFECYCLE/`，保留完整退出后 AX 异常和误点 / 裁切尝试；不覆盖先前来源不明的退出后窗口记录，也不替代原件正式签名、默认 Application Support 根及真实安装升级。

## 下一步及完成条件

| 顺序 | 工作 | 完成条件 |
|---|---|---|
| 1 | 收齐最新 369ab12d 的 Windows 安装候选 | 已完成同 SHA CI / 候选与 ZIP / MSI / NSIS 独立核验；最终 EXE 身份由 runner 回执证明，设备安装后独立重算仍保留 |
| 2 | 双平台隔离原生与安装验收 | Windows 双权限 / 数据根 / 物理 DPI / 外观 / IPC；macOS 正常退出、安装升级、树表焦点及对齐；均保存同 SHA 证据 |
| 3 | 自动检查性能与真实周期 | 下述 0.1.9 配对预算、缓存命中零请求、慢网离线、stable 24h / beta 6h 实测通过 |
| 4 | 真实更新和事件生命周期 | 两类更新详情 / 进度 / 备份阻断 / 重启回读、异常精确解除与通道隔离有执行证据 |
| 5 | 签名与发布资产审阅 | macOS 签名 / 公证、Windows Authenticode 和更新源 / 资产校验闭合；正式发布另行确认 |

前四步的候选与隔离验收属于当前目标执行范围；真实用户迁移、替换日常安装、创建版本标签、公开 Release 和 stable 晋升仍需单独授权。

## Windows 真机交接

使用 Windows 隔离测试机 / VM 或独立测试账户，保留日常安装和真实用户数据。测试包与源码必须同 SHA；记录 Windows / WebView2 版本、安装器哈希、安装后 EXE 哈希、实际启动日志的源码身份。按顺序执行，前台只保留一个测试实例。

| 验收项 | 必须留下的证据 |
|---|---|
| 普通 / 管理员权限 | 实际进程 token，启动 / 正常退出 / 锁清理，不能只写启动方式 |
| 安装与全部存储 | 自选实际安装位置的新装默认 `<安装目录>\data`；旧根 / 显式自定义根优先；偏好、备份、日志、缓存、托管面板 HOME 都回读同一活动根 |
| 权限与失败路径 | 不可写安装目录明确失败，无静默 AppData 回退；隔离夹具验证切根拒绝忙写者、失败解除、成功待重启与重启回读 |
| 原生外观 | 标题栏、云母、窗口控制与拖动、动效、下拉玻璃、浅深主题 / 高中低画质、顶部左右光层边界 |
| CDP 黑底观察 | 保存同一准备态的真机整窗和 CDP 对照；定位透明侧栏黑底 / 低对比度是否为原生合成或捕获差异，不用 WebView 截图代替云母验收 |
| DPI 与树表 | 实际显示缩放 100 / 125 / 150 / 200%；复选框 / 半选 / 禁用 / 箭头 / 叶占位 / 长文字与焦点、刷新保留层级 |
| IPC | GUI 先显式启用；同 SHA 的 ocxd 只读查询核验目标实例；断开观察者后操作与审计所有权、重启世代和释放顺序 |
| 事件 | 按注册表核对现有发出点，成功只解除精确匹配异常；切通道保留面板状态；进度 tick 不重复通知 |

`test/smoke/windows-appearance.ps1` 需要 PowerShell 7，分别用 `-Mode ordinary` / `administrator` 校验实际 token。它强制 WebView scale 1 / 1.25 / 1.5 / 2，只是渲染 DPR 矩阵，不能代替物理 DPI 验收；脚本结束强制收尾不能当作正常退出或锁清理通过。它设置独立沙箱根，因此也不能代替新装默认数据根测试。

`test/smoke/windows-installer.ps1` 仅允许临时 GitHub Actions Windows runner，不能绕过 guard 在用户设备运行。Windows named pipe 身份按账户派生，不能靠不同沙箱根区分；IPC 测试需要先确认目标，避免连到同账户日常实例。安装器是否包含 ocxd 以资产清单为准，需要另编 CLI 时记录同 SHA 来源。

## 性能与真实更新

继续使用实施前预算，测试后不得下调：同机 0.1.9 配对，冷 / 热启动各至少 20 次；启动可交互 p95 增量 **≤100ms 且 ≤5%**，闲置 10min CPU 增量 **≤0.5 百分点**、RSS 增量 **≤10MiB**，检查不产生 **>50ms** 主线程任务。缓存命中 100 次切页 / 10 次重启应为零网络和查询子进程；慢网、离线、单次检查分别取样，stable 24h / beta 6h 真实周期单独取证。

共用更新流程要补真实端点、内部说明滚动 / 网页、备份失败阻断、阶段动画、安装 / 重启后版本回读、失败与精确恢复证据。候选构建、源码测试和加速时钟都不能替代这些实测。macOS 正式签名 / 公证和真实安装升级同样待验。

当前 `release_authorized: false`、`release_accepted: false`，IMP / TASK 保持 `in_progress`。本轮没有启动额外测试软件、迁移真实数据、替换日常安装、创建标签、公开发布或晋升 stable。

## 原始日志与可读视图

ADCC / 3D856 macOS 构建日志、IPC 定向日志及 Windows 失败摘录的原始字节保存为对应 `*.raw.gz`。同名 `.log` 为去除行尾空格和末尾空行的可读视图，未修改消息；各目录的 `log-provenance.json` 记录原始字节数、解压后 SHA256 与视图 SHA256，`checksums.json` 覆盖归档和视图。签名失败与测试失败均保留，不能通过视图规范化改变验收结论。

## main@369ab12d 最新候选与树表验证

源码 `main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27` 已推送。统一 UiTreeTable 采用 16×22px 选择槽、16px 选择框，箭头 / 图标 / 叶占位与文字按 22px 首行对齐；多行名称不再使控件居中于整段。异步卸载前保存焦点及可见祖先，回退至可用选择框、同行主控件、可见祖先、首行；外部焦点不抢夺，错误移至重试入口，过期回调受保护。复选框方向键接入树导航，Space 保持原生选择。

扩展用例对父提交生产组件的基线为 8 passed / 6 failed，失败原件保留；修复后树表 15 项、全 UI 94 文件 / 466 项通过，typecheck / 构建 / diff check 通过。生产浏览器夹具浅深色、300 / 420px 名称列的首行中心偏差 11→0 CSS px；混合态选择框刷新保焦与 ArrowDown 实测通过，console 无错误。临时 QA 标签和 Vite PID4947 已清理；证据见 .adg/evidence/OCX-0110-20261011-369A-TREE-ALIGNMENT-FOCUS/，不代替完整原生 / 物理 DPI。

同源码 [CI 38124355301](https://github.com/gzers/opencodex-desktop/actions/runs/38124355301) completed / success，release-tools、frontend、backend、backend-windows、macOS build 五项成功。核对远端 main、最近 50 个候选查重后，于 17:04（UTC+8）只触发一次 [Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287)，headSha 一致。后续快照锁定依赖回归成功，NSIS / MSI 构建中，安装 smoke / 上传 / 本地核验待完成；原件见 .adg/evidence/OCX-0110-20261011-369A-CI-CANDIDATE-DISPATCH/。03e0 已通过项保留历史，不自动继承为最新 SHA 通过。

macOS 同源码 app / dmg 构建 exit 0，用时 121.016s；arm64、版本 / 构建号 0.1.10，完整 SHA 内嵌核验通过。DMG 4384841 bytes，SHA256 `7ccb4927be704711997674ad58dba8342c80a70fcf0b62de080440cea351f743`，hdiutil verify exit 0；原 EXE 7897776 bytes，SHA256 `3e670b4e2e7517e4d10897919b56769030bde3eac944d4642fe5c0e5ded9f52e`。原件保存于 /private/tmp/ocx-0110-369a-candidate-tgpmm_zq/original/；严格 codesign exit 1，资源未封装、只有 linker ad hoc，宿主 0 个有效签名身份，公证未验证。测试副本独立 identifier sandbox0110369a，ad hoc 严格校验通过，EXE SHA256 `8bb3920180d03c54e78392df71d8e0e57fefda0ddb023d60f2879625e71f4018`；不能代表原件正式签名。

旧测试 PID75489 原生 ⌘Q 退出、锁非阻塞可取得且字节不变；只读挂载 DMG 复制安装后，最新 PID9422 为唯一测试实例，test-data 根、环境与锁 metadata 匹配。日常 PID75764 与启动时间未变。真实偏好备份 `bk_20261011082942_7318022d` 仅含 preferences.json / backup-manifest.json，载荷 1371 bytes，独立摘要与清单一致；不含完整面板配置，未恢复。

原生根 / 年默认展开、月份收起；建立焦点后方向键到清单、Left 返回父箭头、刷新保留展开层级和浅深色局部可读已有证据。无复选框场景；刷新焦点位于 HTML 内容，不能记原生异步行焦点恢复通过。00—10 的 JPEG / AX / metadata 保留误点、无焦点按键和裁切尝试；08 只证明展开保留，09 树不在截图范围、10 滚动后才可见。见 .adg/evidence/OCX-0110-20261011-369A-MACOS-TREE-CANDIDATE/；原日志 gzip 按字节保存并记录摘要，各目录 checksums.sha256 独立校验。

继续完成最新 Windows 候选资产核验、实际目标 token 双权限 / 物理 DPI / 默认数据根 / 原生外观，以及 macOS 完整树表密度 / 画质矩阵、最新 SHA 生命周期 / 真实安装升级、同机 0.1.9 配对性能 / 真实周期、真实更新 / 事件和正式签名。性能门禁不调整；核心事实推广仍待后续确认。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`。本轮文档提交只包括 0.1.10 证据与协作包，未夹带 0.2.0 资料、迁移真实数据、替换日常安装或公开发布。

### 2026-10-11 main@369ab12d Windows 安装候选最终核验

最新 [Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287) 于 17:17:48（UTC+8）completed / success，完整源码身份为 369ab12da55448fbfdc4d0c9bf03970b56fd6c27；该 SHA 只触发一次。锁定依赖回归、MSI / NSIS 构建、隔离 runner 安装 / 最终 EXE 首屏 / 卸载及上传成功。17:28（UTC+8）本地独立重算两个 ZIP 的大小、SHA256，与 GitHub API digest 一致，ZIP CRC 通过；候选 ZIP 只有两个安装器和回执。两份 artifact 的 installer-verification.json 字节一致（1970 bytes，SHA256 3144d002a110caca807595c2dd9ec7cb6c50e68de4d0d84f1e820a61b1ff4857）。

| 资产 | bytes | 独立 SHA256 |
|---|---:|---|
| 候选 ZIP，artifact 11696829726 | 7561927 | 77d4ca63098b18a93589761edbe639c6808fd1dc253e61d805c4b5e6513cc002 |
| 证据 ZIP，artifact 11696909705 | 1685851 | bae9aa0b6b1264d2c9df9d5f8fb1cec032ffc9a52f3b56559418a75337f68a5b |
| MSI | 4538368 | f149f01f3096fc7aafa8bc574358ab11fd6c77479db28d822ee414a57fed2fc3 |
| NSIS | 3231540 | c5a3b9dc9ed87e486933a64ff4402fb47c5d1eb9ecca46fcad5e090de6b3dd0f |

安装器摘要与 runner 回执一致。MSI / NSIS 安装后 EXE 哈希、预期 bundle marker、启动源码 / 关于信息 / WebView 回执匹配；artifact 没有安装后 EXE，该部分仍是 runner 证据，本机未独立提取重算。30 秒首屏观察均已挂载 UI、0.1.10 / Windows、准备态、CLI 禁用、沙箱根初始化，安装 / 卸载 exit 0 且卸载后 EXE 移除。两个 artifact 预计 2026-10-25 17:17（UTC+8）到期，可从 run 的 Artifacts 下载最新候选；本地 ZIP / 安装器留存于 /private/tmp/ocx-0110-369a-windows-3sb0v4xp，未执行安装器。

最新两张 CDP WebView 截图仍可见左侧透明区域黑底、部分文字对比度低，根因未确认；Windows 真机须保留同状态整窗 / CDP 对照，不能据图认定云母或完整原生外观通过。startup-diagnostics 权限字段属于 PowerShell harness 当前 token，不证明目标应用进程权限；沙箱根、DPR=1 与强制收尾也不替代默认安装目录 data、物理 DPI、正常退出或锁释放。

原件见 .adg/evidence/OCX-0110-20261011-369A-WINDOWS-INSTALLER-CANDIDATE/：精选 PNG 与 JSON 原始 gzip 保留 artifact 字节，JSON、日志及 API / 下载报告均附原始摘要和可读视图，checksums.json 覆盖其余文件；没有把 315 项证据中的缓存、偏好载荷或大安装器复制进仓库。此前“构建中”快照继续保留，以上为最终核验结果。后续推进双平台真机 / 最新 SHA 生命周期、配对性能 / 真实周期、真实更新 / 事件和正式签名；IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

### 2026-10-11 main@369ab12d macOS 同根正常退出与重启

最新隔离 DMG 测试副本在 17:37（UTC+8）原生 ⌘Q 后测试进程消失、非阻塞独占锁可取得，锁文件未删除 / 截断。随后 UI 工具自动重开 PID96522 时沿用旧沙箱根；该失败尝试及 after-quit.ax.txt 保留，不计退出证明或同根重启通过。再次正常退出后，于 17:38:37 显式指定同一 test-data 根重启，open exit 0，PID97688 的环境、根、内嵌完整源码 SHA、锁 metadata 一致。

原生设置回读正确根，根 / 年默认展开、月份收起，原备份 bk_20261011082942_7318022d 可见；退出前 / 后、显式重启后及 17:49 最终快照中 preferences.json 与 backup-manifest.json 大小 / SHA256 全部不变。日常 PID75764 及启动时间不变，最终只有一个测试实例。原始快照、只读锁探针方法、AX、失败重开与截图映射见 .adg/evidence/OCX-0110-20261011-369A-MACOS-SAME-ROOT-LIFECYCLE/；截图实为 JPEG，只改扩展名、未改字节。

仅闭合当前 SHA 的隔离同根生命周期局部验证；ad hoc 副本不替代原件正式签名 / 公证、默认 Application Support 根、真实安装升级或更新重启。Windows 实际目标双权限 / 物理 DPI / 默认根 / 原生外观、完整画质矩阵、配对性能 / 真实周期、真实更新 / 事件及正式签名继续待验。IMP / TASK 保持 in_progress，release_authorized: false、release_accepted: false。
