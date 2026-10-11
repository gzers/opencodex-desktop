# 0.1.9 与 deb740 十分钟性能对照

归属 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。采样为 2026-10-12（UTC+8）；源码为官方 0.1.9 `de6862d40389d527527b9db8ef52e186a58b625f` 与 0.1.10 `deb740c09857b5f0bef7c9b4090fe5e8587bb4b3`。本轮性能预算比较**未通过**，不认定具体因果回归、内存泄漏或自动检查根因。

## 方法与结果

同一 macOS 27.0.1（26A434）arm64 宿主、18 个逻辑 CPU、AC 电源、无记录中的 thermal warning。顺序运行官方基线再运行候选，均使用隔离 ad hoc 副本及独立测试根；浅色 / 高档 / WebGL mesh / 100% 的概览准备态，Node/npm 已发现、OpenCodex 缺失。每轮自启动预热约 184.54 秒，再以 10 秒间隔记录 61 个样本。实际进程、环境、根、源码和二进制摘要见 [candidates.json](evidence/candidates.json)、两轮 `launch.json` 与 `context.json`。

采集器以精确应用路径和 resource coalition 归属纳入应用、WebKit GPU、WebContent、Networking，使用累计 CPU 秒与 RSS；61 个样本中的四个进程身份均稳定。CPU = 累计 CPU 秒差 / 实际单调 elapsed 秒 × 100%，以单核百分比计；RSS 是各进程 resident 之和，不是去重物理内存。

| 指标 | 官方 0.1.9 | 0.1.10 deb740 | 候选减基线 | 原预算 |
|---|---:|---:|---:|---:|
| 实际采样秒 | 600.0159545 | 600.0238034 | — | 10 分钟 |
| 累计 CPU 秒 | 290.05 | 300.96 | +10.91 | — |
| 平均单核 CPU | 48.340381% | 50.158010% | +1.817629 百分点 | ≤0.5 百分点 |
| 平均 RSS | 284.617572 MiB | 401.626537 MiB | +117.008965 MiB | 增量 ≤10 MiB |
| 结束 RSS | 284.859375 MiB | 505.531250 MiB | +220.671875 MiB | 增量 ≤10 MiB |
| 轮内 RSS 增长 | +0.484375 MiB | +211.390625 MiB | — | 调查信号 |
| 采集器自身 CPU 秒 | 0.317583 | 0.324500 | — | 单列，不计入应用 |

时间窗为 UTC 2026-10-11 16:23:24.286544–16:33:24.296692、16:38:47.753837–16:48:47.778817。CPU、平均 RSS、结束 RSS 三项比较均超预算，见独立重算 [comparison.json](evidence/comparison.json)。两轮记录的偏好、缓存与调度四个文件大小 / SHA256 / mtime 不变；这不证明零网络或自动检查没有发生。

WebContent RSS 在候选中由 103.953125 增至 309.468750 MiB，基线由 94.750000 增至 95.296875 MiB；候选应用 RSS 约 118.97–119.02 MiB。各角色累计 CPU 秒如下，不能仅凭进程角色判断具体功能原因。

| 角色 | 0.1.9 CPU 秒 | 0.1.10 CPU 秒 |
|---|---:|---:|
| 应用 | 42.30 | 35.78 |
| WebKit GPU | 150.37 | 175.61 |
| WebContent | 97.38 | 89.57 |
| Networking | 0 | 0 |

## 调查线索与有效性边界

十分钟后单独运行的 WebContent `sample` / `vmmap` 是调查线索，不属于配对采样。`sample`（00:51:28，UTC+8）主线程 1310 个观察中，约 884 个落在 timer / RemoteLayerTreeDrawingArea 链；约 701 个包含 SVG mask / stroke bounding box / CoreGraphics 路径计算，另约 103 个涉及 WebGL prepareForDisplay / GPU 等待。见 [webcontent-sample.txt.gz](evidence/candidate0110/idle-high/webcontent-sample.txt.gz)。观察次数不是精确耗时比例。

`vmmap`（00:53:48.670）记录 footprint 428.4M、peak 435.8M，WebKit Malloc resident 354.5M / dirty 329.6M；对应 malloc zone 3,633,438 allocations / 257.7M allocated。没有同时间、同方法的基线 vmmap，不能认定泄漏，也不能把 RSS 与 footprint 混用。原件见 [webcontent-vmmap.txt](evidence/candidate0110/idle-high/webcontent-vmmap.txt)。

设置页另有 90.020976 秒 / 10 样本诊断：累计 CPU +0.07 秒（0.077760% 单核），RSS 611.125 → 580.109375 MiB，身份及四文件摘要稳定。**该轮锁屏起点未知，缺同时结束 UI，也缺连续前台记录，不能把低 CPU 归因于页面卸载。** `actual_warmup_seconds` 约 1325.47 是距原启动的时间，不是该阶段预热；开始 AX 是 171 字节的差异视图。没有把本轮当设置页验收。后续解锁截图只能注明延迟结束快照。

两轮首末 AX / JPEG 能证明捕获时的准备态外观；基线开始 AX 135 字节为差异视图，另存完整 AX。没有连续全局前台 / 可见性、启动可交互、主线程长任务或网络仪器；10 秒轮询可能漏短命进程。只有一组顺序对照，未随机重复，未覆盖就绪态。因此预算未通过仍保留，但因果与整体性能验收未完成。用户主动切换前台不被记录为软件缺陷。

基线的 ⌘Q / 菜单退出未生效，后续精确核实路径后于 UTC 16:35:36.393367 仅对 PID29226 发 SIGTERM，helpers 消失；这是测试清理，不计正常退出验收。`cleanup.json` 的 returncode 1 为清理后 `ps` 无进程的结果，不是 signal 失败。候选于基线清理后才启动。UTC 17:18:16.449950 快照确认一个隔离测试 PID31501，日常 PID75764 / Oct 9 22:00:24 启动身份不变；该结论只对快照有效。后续 CUA 报告 Mac 锁屏，已请求人工解锁，没有另开测试实例。

## 归档与离线复核

[archive-verification.json](archive-verification.json) 核验 46 项 manifest、42 份来源无损复制、3 份明确筛选的归属衍生记录、3 份 gzip roundtrip、4 张 JPEG 仅扩展名修正，证据约 1.98 MB。原始 `attribution.json` 含其他应用拒绝项，仅保留 8 个 admitted records 及原件大小 / 摘要 / 计数；这三份**不是原件无损复制**，原件仍在外部隔离证据根。

`SHA256SUMS` 摘要为 `ff7c72fc892e72b6353b12ef6154d01ffbce91bade93442ab00375e61ba6c922`。归档输入独立重算与保存的 comparison 字节一致。方法和来源关系见 `evidence/methods/` 与 [source-map.json](evidence/source-map.json)。未归档二进制、应用 bundle、偏好 / 缓存 / 备份载荷、WebView profile 或其他应用元数据。

从本目录可离线复核，不依赖外部测试二进制：

```sh
python3 evidence/methods/recompute.py --root evidence --output /tmp/ocx-0110-comparison.json
```

`--verify-binaries` 仅在外部原隔离二进制仍存在时使用；默认离线复核校验采样、身份、方法及计算，不重新证明原件正式签名。归档完整性通过与版本验收未通过分别记录。

## 下一步与门禁

解锁后复用唯一候选，先补设置页延迟结束快照，再做中档静态概览、高档 CSS 概览的短时诊断；必要时回到高档 mesh。每次记录真实设置、首末 UI、进程身份及连续前台，并避免重叠采集器。用于隔离 Logo / SVG、mesh、光层布局与页面生命周期的线索；先建立可重现差异再修复，不推测性撤销已确认的顶部 / 左右光层覆盖。

修复后须以同机官方 0.1.9 配对重新评定十分钟 CPU / RSS，并补冷 / 热各 ≥20 次、启动可交互 p95 增量 ≤100ms 且 ≤5%、检查无 >50ms 主线程任务。真实 stable 24h / beta 6h、睡眠 / 跨启动、双平台原生 / 安装 / 真实更新 / 事件与正式签名门禁继续开放。缓存 10/10 重启与 100 次概览进入局部通过不替代这些检查。IMP / TASK 保持 in_progress，发布未授权 / 未验收，不生成关闭证书。
