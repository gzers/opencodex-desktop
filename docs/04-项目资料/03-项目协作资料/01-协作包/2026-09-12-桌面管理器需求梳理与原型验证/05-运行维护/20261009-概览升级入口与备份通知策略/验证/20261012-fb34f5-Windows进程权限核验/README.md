# fb34f5 Windows 应用进程权限与候选安装包核验

本轮代码为 `main@fb34f5b91c3a6feb5b41f11f9527171c4bf742f5`（`test(windows): 使用应用进程令牌核验权限回执`）。相对 deb740，仅修改 Windows smoke / token helper 及两条 workflow，共 7 个文件，没有应用或后端实现变化。CI 与候选安装流水线均已完成并成功；以下结论按下载制品、runner 回执和本机独立复核区分，不代表 0.1.10 整体验收通过。

## 同源码流水线

- [CI · 38162783294](https://github.com/gzers/opencodex-desktop/actions/runs/38162783294)：Windows / macOS 后端、前端及 Windows startup。
- [Windows candidate installer · 38163233949](https://github.com/gzers/opencodex-desktop/actions/runs/38163233949)：候选 MSI / NSIS 构建、新装、首屏与卸载。

最终 API 快照均为 `completed / success`，`headSha` 与上述完整源码身份相同。见 [CI 快照](ci-snapshot.json)、[候选快照](candidate-snapshot.json)。未重新触发流水线，也未把两个独立构建的 EXE 摘要相互代替。

| 日志范围 | 结果 | 统计边界 |
|---|---|---|
| CI Windows Rust | 585 passed / 0 failed / 6 ignored | 34 个 result group，包含零测试组 |
| CI macOS Rust | 866 passed / 0 failed / 6 ignored | 34 个 result group，包含零测试组 |
| CI 前端 | 95 个文件 / 474 passed | 按完整日志解析 |
| Candidate Windows Rust | 585 passed / 0 failed / 6 ignored | 34 个 result group，包含零测试组 |
| Windows token helper | 9 项通过 | harness 和 cmd.exe 子进程 helper 验证 |

ignored 不计为通过；见 [日志独立统计](job-log-analysis.json) 和 `ci/*.log.gz`、`candidate/run.log.gz`。helper 的 [原始核验](archive-verification.json) 保留原义，只证明 helper 制品和测试，不等于实际应用或安装验收。本轮最新 API 下载证据没有 macOS DMG，因此不声明独立核验了 fb34f5 的 macOS DMG。

## 下载制品及安装回执

独立复核 4 个 artifact 的完整源码身份、API 大小与 SHA256、ZIP CRC、安全条目路径；下载的候选 MSI / NSIS 再分别按回执核对字节数和摘要，包内与 evidence 内的安装回执逐字节一致。见 [应用与安装器核验](application-installer-verification.json)。

| 下载安装器 | 字节数 | SHA256 |
|---|---:|---|
| MSI | 4538368 | `834dba519f3c159762ceb2656f312137806dbdf22f926e6c270c06cc1422a771` |
| NSIS | 3232774 | `49f8d0e51fdcbce234ab0c70225d73e33486a2ef6fad517b3874597b504482b2` |

runner 两种安装器均记录版本 0.1.10、安装与卸载 exit 0、首屏通过、卸载后 EXE 消失。安装后的 EXE 不在下载 artifact 内，其摘要只由 runner 回执提供；本机没有重新运行 Windows 安装器，也没有独立计算安装后 EXE 的摘要。`codeSigned=false / releaseAccepted=false`，不能当作正式签名或发布验收。

## 应用进程权限核验范围

CI 与两种候选安装启动回执都已从实际应用 PID 读取 token，PID、完整 EXE 路径和源码身份一致。

| 回执 | 实际应用 PID | token | 模式验收 |
|---|---:|---|---|
| [CI startup](ci/startup/startup.json) | 8308 | elevated=true / elevationType=1 | mode:any，applicationModeMatched=false |
| [MSI startup](candidate/msi-startup/startup.json) | 2592 | elevated=true / elevationType=1 | mode:any，applicationModeMatched=false |
| [NSIS startup](candidate/nsis-startup/startup.json) | 2792 | elevated=true / elevationType=1 | mode:any，applicationModeMatched=false |

三份回执均为 `applicationTokenVerified=true`；这是 CI 提权环境下的实际应用观察，不能代替普通 / 管理员双权限验收。数据根使用随机 runner 临时沙箱和显式 override，没有验证默认 `<实际安装目录>\data`。DPR=1 与自动收尾不能代替物理 DPI、正常退出、真实用户数据保留或升级。云母、标题栏、菜单隐藏和 CDP 透明侧栏黑底 / 低对比仍需真机整窗验收，根因未确认。

## 归档复核与剩余门禁

归档只留选定回执、环境摘要和 6 份 gzip 日志；34 份来源文件逐字节复制或解压 roundtrip 一致。`source-map.json` 记录来源与变换，`SHA256SUMS` 覆盖 39 项，`archive-integrity.json` 单独记录清单摘要。安装器、测试二进制、用户载荷、沙箱及 WebView profile 留在仓库外，未整体复制 evidence ZIP。4 个 artifact 的下载摘要与安全性通过，不提升任何整体验收门禁。

macOS 官方 0.1.9 与 deb740 的[十分钟性能对照](../20261012-deb740-十分钟性能对照/README.md)仍为失败：平均 CPU +1.817629 百分点、平均 RSS +117.008965 MiB、结束 RSS +220.671875 MiB。fb34f5 没有应用实现改动，也没有新性能验收能消除该结果。SVG / WebKit 线索不是自动检查或泄漏的原因证明。

前台工具此前报告 Mac 锁屏，等待人工解锁；现有一个隔离测试实例继续复用。解锁后先做中档静态、高档 CSS 和必要的 mesh 返回受控诊断，记录首末 UI 与连续前台，再依据结果修复并重跑配对性能。保留已确认的光层边距覆盖。

冷 / 热各至少 20 次、启动可交互 p95 增量 ≤100ms 且 ≤5%、闲置 10min CPU 增量 ≤0.5 百分点 / RSS 增量 ≤10MiB、无 >50ms 主线程任务及真实 stable 24h / beta 6h 等原门禁不变。Windows 双权限 / 默认数据根 / 原生外观、双平台安装 / 真实更新 / 事件及正式签名仍需补齐。TASK / IMP 继续 in_progress，八项 required_checks 尚未整体闭合，acceptance_results 为空，无核心事实推广、真实迁移、日常安装替换、tag 或公开发布。
