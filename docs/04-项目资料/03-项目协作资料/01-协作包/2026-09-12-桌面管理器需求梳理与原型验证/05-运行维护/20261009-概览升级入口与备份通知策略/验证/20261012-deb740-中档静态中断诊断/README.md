# deb740 中档静态场景中断诊断

本轮只有调查线索，**整轮无效，不计性能验收**。概览准备态中档静态 / mesh 的前 129.921825 秒 RSS 基本稳定，随后失去前台；结束 CUA 报告锁屏，缺结束原生 UI。不能据此确认高档内存增长的原因，也不能替代[既有十分钟性能对照](../20261012-deb740-十分钟性能对照/README.md)的 CPU / RSS 超预算结论。

采集与回写均在 2026-10-12；本页时间全部为 UTC，Asia/Shanghai 为 UTC+8。关联 IMP-20 / TASK-182，应用实现源码 deb740c09857b5f0bef7c9b4090fe5e8587bb4b3；当前 main@fb34f5 的差异仅为 Windows 测试 / CI，不包含应用实现改动。本轮没有源码修改、真实迁移、日常安装替换、核心事实推广或发布。

## 场景与身份

- 复用唯一隔离测试 PID31501，测试 bundle `com.gzers.opencodex.desktop.perf0110oct12`。测试 app 与数据分别在 `/private/tmp/ocx-0110-performance-20261012/candidate0110/test-install/` 和同级 `test-data/`，边界为 `/private/tmp/ocx-0110-performance-20261012`。不创建第二个实例。
- 测试 EXE SHA256 `ab6b967da2936e66cb5ba481a462de7334e692573d2f6db0c539a73c7d61f9b4`；归档前独立读字节确认摘要与内嵌 deb740 源码身份。隔离副本为 ad hoc 签名，不能替代正式签名 / 公证。
- 本轮资源身份固定为 App31501、GPU31510、WebContent31511、Networking31512；只纳入经 resource coalition 验证的四个进程。首末应用快照均只有一个隔离实例；日常 PID75764 / `Fri Oct 9 22:00:24 2026` 身份保持不变。快照不证明全天状态。
- macOS 27.0.1 / arm64、AC 电源。浅色、100%、概览准备态、中档静态光、mesh。通过 Finder 打开现有隔离 app 项目恢复前台，身份仍为同 PID；属于场景准备，不是新的应用启动。

实际画质以 [settings-start.json](evidence/candidate0110/diagnostic-mid-mesh-finder-reopen/settings-start.json)、[settings-end.json](evidence/candidate0110/diagnostic-mid-mesh-finder-reopen/settings-end.json)及四个首末文件中的 preferences SHA 为准，均为 `visual_effects=mid`。`context.json.candidate.appearance` 来自旧静态清单，仍写 `high`，不是本轮真实画质；原件保留，不静默修正。首快照 00:32:16.479118 在预热期间、资源采样开始前取得；末快照 00:36:05.847338 在资源采样结束后取得。偏好摘要均为 `aed374985a30e077a7093a4ea5983b5f7b61bce40cee7ccbf5cc957f19e9e8aa`。

`settings-start-envelope-invalid.json` 保留错误的初始提取。`settings-end.json` 是有效 JSON，已原字节保存，没有“修复”。首 AX 只有“树未变化 / overview 焦点”的增量，不是独立完整 AX 树；首 JPEG 另保留。没有结束 AX 或 JPEG，不能宣称首末原生 UI 齐全。

## 资源数据与前台边界

场景准备从 00:30:56.229048 起，预热约 91.04 秒；资源采样每 10 秒一次，监控另按约 0.1 秒观察 NSWorkspace 前台 PID 与 on-screen layer-zero 窗口。没有注入探针或 UI 动作。

| 指标 | 整轮资源记录（无效） | 首次前台无效前的诊断子区间 |
| --- | --- | --- |
| 时间 | 00:32:27.512142–00:35:27.524211 | 00:32:27.512142–00:34:37.436545 |
| 单调时长 / 样本数 | 180.009340 秒 / 19 | 129.921825 秒 / 14 |
| 累计 CPU 增量 | 0.100000 秒 | 0.010000 秒 |
| 单核 CPU 比例 | 0.055553% | 0.007697% |
| RSS 首 → 末 | 408.140625 → 269.531250 MiB | 408.140625 → 407.953125 MiB |
| RSS 平均 | 378.823191 MiB | 408.015625 MiB |
| 验收用途 | 不计通过 | 仅诊断，未配对、不计通过 |

前台记录 2427 个采样，其中 2060 个满足前台且可见、367 个未满足。首次无效在 **00:34:47.150252**，`target_frontmost=false`、`target_window_on_screen=true`；整轮 `all_samples_frontmost_visible=false`。最大相邻采样的**单调 `elapsed_s` 间隔**为 0.145694167 秒，不能据此保证采样间每一瞬间。约末 40 秒资源区间没有满足前台条件。

本页子区间只选择首次无效时间之前的原资源样本，明确是约 130 秒，不能描述为 140 秒或整轮有效。末段 RSS 下降不归因于中档、页面卸载、锁屏回收或某一实现机制；首次前台丢失时间也不是已确认的锁屏起点。用户已说明之前焦点丢失由人工操作引起，不能写为软件焦点缺陷；本轮只记录观测，不推断新的原因。

四个偏好 / 更新缓存 / 调度文件大小、SHA256 与 mtime 首末一致。该事实不证明零网络。RSS 为四个进程 resident 之和，不是去重物理内存，也不证明泄漏。复用既有实例、画质切换与短时子区间未形成独立重复的高 / 中档因果实验；启动可交互、主线程长任务、网络均无新验收。

## 未就绪准备记录

三次准备阶段均未进入资源采样，错误为 `foreground visible window not ready within 10s`；每份就绪尝试 94 个采样，目标窗口可见但不是前台。另一个 preflight 也保留。不认定应用故障。

| 阶段 | 监控起点 | 保存内容 |
| --- | --- | --- |
| diagnostic-unlocked-mid-mesh | 00:14:24.496179 | 就绪失败、选择 / 概览 UI、初始无效准备与 launch 原件 |
| diagnostic-mid-mesh-active | preflight 00:18:25.829786；主监控 00:19:37.261070 | 两份就绪失败、概览 UI 与 launch 原件 |
| diagnostic-mid-mesh-unlocked-final | 00:25:47.497779 | 就绪失败、首 UI 与 launch 原件 |

用户回复解锁后，末次 CUA 捕获仍报告 Mac locked / 自动解锁失败。继续等待人工确认“桌面可操作”，没有反复解锁、打开额外测试窗口或用非原生控制工具绕过。本轮未使用 caffeinate，也未修改系统睡眠 / 锁屏设置。Finder 中代理创建的 test-install 标签页待桌面恢复后关闭，保留用户的下载 / 存储 / work 标签页。

## 原件、独立重算与剩余门禁

44 份来源无损复制、1 份明确标记的 admitted-only 归属衍生、1 份 gzip roundtrip、47 项 SHA256 manifest 核验通过，证据约 2.00 MB。归属原件 36 条，仅保留 8 条 admitted，排除的 28 条其他应用元数据仅留数量与原文件摘要。采集器、场景驱动、只读前台监控源码及归档 / 重算方法均保留；监控二进制只记录摘要。没有复制应用二进制、测试数据载荷、备份、WebView profile 或无关工作区。

- [归档核验](archive-verification.json)仅为证据完整性通过。
- [独立重算](evidence/diagnostic-recompute.json)分别读取外部原件与已归档输入，除归属衍生输入摘要外，所有计算结论一致；复核资源身份 / CPU / RSS、画质与稳定偏好、前台失效及子区间、四份未就绪监控。
- [SHA256SUMS](evidence/SHA256SUMS)与[source-map.json](evidence/source-map.json)固定证据字节和映射，不改写既有十分钟证据或 RCP。

下一步在同一隔离实例完成可操作桌面下的中档静态 / 高档 CSS / 必要的高档 mesh 控制场景，首末设置与 UI 齐全且持续前台，再建立复现、定位原因。任何修复须保留已确认的光层顶部 / 侧边距覆盖并做适用测试，随后重新进行官方 0.1.9 同机十分钟配对。

性能预算保持：CPU 增量 ≤0.5 百分点、RSS 增量 ≤10MiB；冷 / 热各至少 20 次、启动可交互 p95 增量 ≤100ms 且 ≤5%、检查无 >50ms 主线程任务。真实 stable 24h / beta 6h / 面板 24h、睡眠 / 跨启动、离线慢网超时并发通道隔离，以及双平台原生 / 安装 / 更新 / 事件 / 正式签名仍待完成。`review:scheduler-performance-0110` 保持 failed，八项门禁仍开放，TASK / IMP in_progress、acceptance_results 为空、发布未授权 / 未验收。
