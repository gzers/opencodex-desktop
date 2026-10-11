# c1e3 缓存探针与前台门禁

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。本轮仅记录缓存探针、隔离生命周期及测试有效性，不是 0.1.10 验收通过或发布证书。

探针自检、真实查询阳性对照及证据无损归档通过。用户让出前台后的复测取得第二次有效重启样本，累计 2 / 10；200 次页面切换完成，但切换期间前台条件不满足，因此该组不计完整缓存零请求验收。最新普通隔离实例为 PID10336，注入实例已退出。

用户随后明确：“失去前台是我操作的原因，不是软件问题。”该说明作为用户报告的原因单独记录，不根据采样中的前台丢失认定应用焦点缺陷。原始记录与无效轮次保留；初始窗口未显示的采样也单独保留，不由该说明推导其原因。后续应在预先沟通的短时、不切换应用的窗口补测，避免无协调地重复前台测试。

## 源码与测量方法

| 对象 | 身份与用途 |
|---|---|
| 应用 / 候选基线 | main@c1e3c09d8ed33238543ce5943e439991b8099088；原件及安装候选见[上一轮候选核验](../20261011-c1e3-候选最终核验/README.md) |
| 探针工具提交 | main@3ed00487958185a520e61ed2ba2d706f95e9a384；只新增测试工具，不改变应用实现或候选包 |
| 探针源码 | [macOS 更新查询探针](https://github.com/gzers/opencodex-desktop/blob/3ed00487958185a520e61ed2ba2d706f95e9a384/test/smoke/macos-update-probe.c)、[自检程序](https://github.com/gzers/opencodex-desktop/blob/3ed00487958185a520e61ed2ba2d706f95e9a384/test/smoke/macos-update-probe-check.py) |
| 测量范围 | macOS 沙箱内 DYLD interposer，记录主进程边界调用尝试；字段仅 pid、mono_ns、wall_ns、event、kind |

2026-10-11 21:29:46（UTC+8）开始的工具自检退出 0，耗时 0.855883 秒。结果覆盖 connect、connectx、getaddrinfo、posix_spawn / posix_spawnp、execv / execvp / execve，以及非沙箱、符号链接保护和脱敏字段。只用本地 loopback，不启动应用、不访问外部更新源。自检记录的 source_head 为当时的 c1e3；归档逐字节核对工具源码与随后提交的 3ed00487 相同，不将旧 HEAD 字段改写为新值。

真实手动查询阳性对照记录 probe_loaded 1、DNS 4、connect 2、npm view 的 spawnp 1 / spawn 1，并生成真实管理器 stable 与面板 latest 缓存。工具记录的是调用尝试，不能解释成成功请求数；子进程 env_clear 会移除注入，因此自检不能替代真实查询对照，也不能声称捕获全系统网络。注入轮次不用于 CPU 或启动耗时对比。

## 真实缓存与零请求样本

缓存由阳性对照中的真实成功手动查询生成。没有修改缓存、调度截止时间或系统时钟；自动检查开启，stable 间隔 24 小时、启动延迟 45 秒，并要求可见且前台。

| 文件（相对隔离数据根） | 大小 | SHA256 |
|---|---:|---|
| manager-state/update-schedule.json | 182 bytes | 2bbdedb306dbf9b8c34b109014c00f130dba270a68978b24b0a6f95082f1676a |
| cache/updates/manager-stable.json | 217 bytes | 4e98783a3efd8d22c002bfb60c00289f9e9cd6829e8b204945bca643b2a98214 |
| cache/updates/panel-latest.json | 145 bytes | 32347ef6243354565e586a8d8a73bf6d715b4c4dbd6a6702170187e6f832a508 |

stable 管理器 last_success / next_due 为 1791722553 / 1791808953，面板为 1791722552 / 1791808952；均无失败且截止时间尚在未来。归档时三份文件的原始字节摘要仍与查询后快照一致。

| 轮次 / PID | 前台且可见采样 | 时长（秒） | 结论 |
|---|---:|---:|---|
| restart-01 / 329 | 0 / 47 | wall 46.389510 | 前台条件不满足，不计入 |
| restart-valid-01 / 2647 | 47 / 47 | monotonic 46.417726 | 第一次有效重启样本；加载探针 1、边界尝试 0，缓存与调度摘要未变 |
| restart-valid-02 / 4705 | 46 / 47 | monotonic 46.396460 | 首次窗口未显示，不计入；不是 47 / 47 通过 |
| valid-02-ready / 4705 | 0 / 47 | monotonic 46.411658 | 同 PID 再观察，前台条件不满足；不算新的有效重启 |
| restart-ready-02 / 5439 | 7 / 47 | monotonic 46.325718 | 初始就绪后前台条件不满足，不计入 |
| opportunity01 / 10029 | 47 / 47 | monotonic 46.432740 | 用户让出操作后的第二次有效重启样本；加载探针 1、边界尝试 0，缓存与调度摘要未变 |

采样间隔约 1 秒，即使 47 / 47 也不证明采样之间的连续状态。原始 JSON 中尚无 monotonic 字段的旧轮次只引用 wall 时长，不补造单调时间。无效轮次的“边界尝试 0”不能用于证明前台自动检查满足门禁。

PID2647 的页面组完成设置 100 次、概览 100 次，200 个 AX URL 全部匹配。完整监控为 259 个采样 / monotonic 260.303215 秒，其中 249 个非前台；动作时间窗内 202 个采样全部非前台。日志仅见 probe_loaded 1，缓存摘要未变，但不计入要求持续前台的 100 次概览进入验收。早期方法试验和失败组也已保存，不能只保留带 valid 文件名的轮次。

## 用户让出前台后的复测

用户明确允许再试一次前台测试。21:49:19（UTC+8）在独立确认 PID7538 正常退出、锁可取得后，同根启动注入 PID10029；原生 AX 就绪并执行窗口 Raise 后开始只读采样。21:49:41 至 21:50:27 的 47 个采样全部为前台且窗口可见，单调时长 46.432740 秒，结束时距启动 68.021502 秒。探针仅记录加载 1 次，主进程查询边界调用尝试 0；随后核验三份缓存 / 调度文件摘要一致，真实截止时间仍在未来。

已有 phase verifier 仅对 restart-* 名称合并前台字段，本轮名为 opportunity01，因此原始 verification JSON 的前台字段为 null。保留原件，通过[独立合并核验](evidence/opportunity01/merged-verification.json)按相同 PID 连接启动、前台采样、边界日志和缓存摘要；不把原始 null 改成通过。采样从窗口就绪后开始，不声称启动至采样开始的全部时间均持续前台，也不将此轮用于启动或 CPU 比较。

完成后已告知用户恢复操作，本次机会只执行一轮，没有继续自动占用前台。此前用户操作导致的前台切换按测试条件中断记录；首次窗口未显示原因仍未独立确认。

## 退出后恢复与实例边界

最后一轮注入实例通过 ⌘Q 退出，随后独立进程 / 非阻塞锁快照确认进程消失、锁可取得且字节不变；先取得独立快照再进行 UI 观察，避免观察接口重新启动应用。

21:31:32（UTC+8）按相同 c1e3 隔离副本启动非注入 PID7538；DYLD_INSERT_LIBRARIES / OPENCODEX_UPDATE_PROBE_LOG 明确为空。21:35:44 独立快照确认唯一测试实例、正确沙箱根与边界、锁 metadata PID7538 / 锁持有，读探针前后锁字节不变。测试 identifier 为 com.gzers.opencodex.desktop.sandbox0110c1e3，测试 EXE SHA256 为 b32c9a8d07955bcd5c0b3edaec34ba7ef738c9444bf0cfd7ac5af204077508f2；这是测试副本，不能替代原件签名验收。

测试根仍为 /private/tmp/ocx-0110-c1e3-candidate-hqjf10k6/test-data。已有备份清单 562 bytes / 90fb16b9acf4f8c2e1dd7cde2c3fcedb22e2ec85d7380311109e46249b9532a6、偏好载荷 1369 bytes / 1a5895d9c89e7144c3353cc3306ce6b0ee1028777c749bb6613adc1d7f1a86ad 摘要未变。日常应用 PID75764 及 Oct 9 22:00:24 启动时间不变；未替换日常安装、迁移真实用户数据或操作备份载荷。

以上 PID7538 为本次机会前的历史恢复实例。新增轮次的注入 PID10029 已通过 ⌘Q 正常退出，独立快照确认进程消失、锁可取得且锁字节未变。21:51:37 启动同根非注入 PID10336，21:54:45 的末次只读快照确认唯一隔离实例、沙箱根与边界正确、锁重新持有，备份摘要和日常应用身份不变。非注入启动记录明确确认两个注入环境变量为空或不存在。该快照是当时状态，普通实例继续运行的日志后续可能增长。

## 证据及未闭合门禁

[机器计算结果](evidence/metrics.json)、[来源与摘要](evidence/source-provenance.json)、[归档校验](archive-verification.json)、[SHA256 清单](evidence/SHA256SUMS)共同保存本轮依据。此前 65 份原始文件保留，新增 opportunity01 子目录下 17 份原始快照，共 82 份逐一按字节 / gzip 解压 roundtrip 核对，86 项证据摘要校验；dylib、三个 monitor 二进制和测试 EXE 共五项仅保留摘要。此前计算结果保存在[历史指标](evidence/metrics-before-opportunity01.json)，最新指标补入第二次有效重启及用户原因说明。方法源码保存原字节，探针源码引用 main 提交；不复制应用、偏好 / 缓存 / 备份载荷或 WebView profile。用户原因说明属于后续解释，单独标明来源，不修改原始采样。

仍需补足有效缓存 100 次概览进入 / 10 次重启零自动查询、受控 0.1.9 配对性能、stable 24h / beta 6h 真实到期及跨启动 / 睡眠、离线 / 慢网 / 超时 / 并发 / 通道隔离，以及双平台原生 / 安装 / 真实更新 / 事件与正式签名门禁。冷 / 热各至少 20 次、交互 p95 ≤100ms 且增量 ≤5%、十分钟 CPU 增量 ≤0.5 百分点 / RSS ≤10MiB、无主线程任务 >50ms 等预算不变。局部工具或 runner 通过不替代这些验收。

八项 TASK.required_checks 保持开放；IMP / TASK 为 in_progress，release_authorized / release_accepted 均为 false。RCP 的 failed 表达整体验收未完成，不表示用户切换应用是软件缺陷。不创建关闭证书，不晋升稳定核心事实，不公开发布。
