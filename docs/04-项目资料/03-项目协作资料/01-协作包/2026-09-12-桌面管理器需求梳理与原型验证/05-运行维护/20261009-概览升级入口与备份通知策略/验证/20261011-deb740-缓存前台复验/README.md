# 0.1.10 deb740 缓存前台复验与候选交接

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。源码为 main@deb740c09857b5f0bef7c9b4090fe5e8587bb4b3，文档归属 docs/governance-main。本文是局部验证记录，不是发布或版本通过证书；前序 c1e3 的无效轮次和用户操作导致的前台切换记录继续保留。

## 缓存重启与页面进入

用户让出前台后，独立重算同 PID 的探针、前台 / 可见窗口采样、AX 页面和文件摘要，得到以下结果：

| 场景 | 有效证据 | 结论 |
|---|---|---|
| 10 次缓存重启 | 10 个独立 PID；每轮 47 个采样，单调时钟至少 46.335297 秒 | 全部采样前台且窗口可见；每轮只有 1 条 probe_loaded，主进程边界调用尝试 0 |
| 100 次概览进入 | 设置 / 概览交替，各 100 次；AX URL 序号 1–200 一致 | 动作时间窗全部落在 271.711039 秒、2493 个前台可见采样范围内；同 PID 边界尝试 0 |
| 缓存与截止 | 管理器 stable、面板缓存与调度共 3 个文件 | 每轮与末次大小 / SHA256 相同；记录的截止在各轮观测结束时仍位于未来 |

10 个 PID 依次为 20593、20777、20960、21441、21621、21948、22131、22320、22437、22639。逐轮证据位于 evidence/probe-restart-valid-01 至 10，汇总见 [foreground-summary](evidence/foreground-summary.json)；页面动作 AX 原件见 [routes actions](evidence/routes-100-actions.jsonl.gz)，独立结果见 [routes verification](evidence/routes-100-verification.json)。

准备态概览已挂载，Node / npm 已发现而 OpenCodex 缺失。该记录不替代就绪态升级入口和真实更新验收。探针覆盖注入主进程的 DNS / connect / exec / spawn 调用尝试，不是成功请求数或全系统网络抓包；此前 c1e3 的自检和真实手动查询阳性对照见 [历史方法与阳性对照](../20261011-c1e3-缓存探针与前台门禁/README.md)，本轮未追加新的 deb740 阳性轮次。

前台仪器用 NSWorkspace 的 frontmost PID 和屏幕上 layer-zero 窗口；重启约每秒采样，页面组约每 0.1 秒采样。只能证明这些采样及时间窗覆盖，不能证明采样间每一瞬间或启动至仪器开始的连续前台。注入轮次不用于 CPU、启动或交互性能结论。

第 2 轮曾误将监控 PID 写成 20962，readiness 超时 exit 2；真实应用 PID 为 20777。错误原件 [wrong PID](evidence/probe-restart-valid-02-foreground.json)与更正监控 [corrected](evidence/probe-restart-valid-02-foreground-corrected.json)一起保留，不记为应用焦点故障、不计错误样本通过。

## 退出后恢复与实例边界

第 10 个注入实例以原生 ⌘Q 正常退出；独立确认测试进程消失、原锁文件非阻塞独占锁可取得，没有删除 / 截断锁文件。随后显式同根恢复普通非注入实例，末次核验 UTC 2026-10-11T15:22:44.636615+00:00：PID23060、正确隔离根 / 锁 metadata、锁持有，DYLD / probe 环境缺席，只有一个测试实例，缓存 / 调度及测试 EXE 摘要稳定。见 [退出](evidence/probe-final-quit.json)、[普通实例核验](evidence/ordinary-final-verification.json)与 [最终窗口](evidence/ordinary-final.jpg)。

隔离根为 /private/tmp/ocx-0110-deb740-foreground/test-data；副本 identifier 为 com.gzers.opencodex.desktop.sandbox0110deb740，ad hoc 副本 EXE SHA256 为 d2b14777050edce579c58e7064bcbbb41f6c11d73da8b601582adcbb9ab5c315。该副本不能替代默认 Application Support 根、正式签名或真实安装升级。日常 /Applications/OpenCodeX Desktop.app 的 PID75764、Fri Oct 9 22:00:24 2026 启动身份未变；未替换日常安装。

前台复验已结束，不再自动占用用户前台。上述实例状态是末次快照，不承诺之后用户操作或外部状态不变；普通实例日志按归档时字节快照留存。

## Windows 同 SHA 候选

[CI 38147288262](https://github.com/gzers/opencodex-desktop/actions/runs/38147288262)五个 job 全部成功；查重后 UTC 15:02:45.283067 只触发一次 [Windows candidate 38149645814](https://github.com/gzers/opencodex-desktop/actions/runs/38149645814)，UTC 15:16:58 completed / success。源码同为 deb740，没有创建 Release 或 tag。

本机未执行 Windows 安装器，独立重算两份 ZIP、MSI / NSIS 大小与 SHA256、ZIP CRC、成员路径和回执一致性。核验见 [independent verification](evidence/windows-installer-independent-verification.json)。候选与证据 artifact 的两份回执字节相同：1970 bytes / SHA256 72d32fb8da5dcd1e336c8300555d0e040e9f4ee790a7dd563c35114fcf4bea84。

| 制品 | 大小 | SHA256 |
|---|---:|---|
| candidate ZIP，artifact 11706329022 | 7562877 bytes | 9bab91aeace36b5bb6f75661587b867b91226cd943b2ac4efed04d833b9afbb5 |
| evidence ZIP，artifact 11706169265 | 1681112 bytes | 71f8879ad1726972c9b2b6ed7ea34d1b909e4d9de6659175ffa39b7b0d972bf1 |
| MSI | 4538368 bytes | 079b9ffabd655ce7335a63af518352e08b57463fa2d9423ae182eb722667d90f |
| NSIS | 3231824 bytes | 59042244b98d09e732982581b040d59a453368ed7859e736eb8b39c259a3f105 |

两份 artifact 到期分别为 UTC 2026-10-25T15:16:47Z / 15:16:50Z。运行页 Artifacts 可下载，安装器原件留在外部临时目录，文档只保留摘要。

Runner 回执中 MSI / NSIS 新装、首屏和卸载 exit 0 / pass，卸载后 EXE removed；各观测 30 秒。关于版本 0.1.10 / Windows、主 UI 挂载、源码身份、沙箱初始化、准备态与关闭 CLI 一致。CDP phase complete，WebView2 153.0.4234.48。安装后 EXE 不在 artifact 成员中，本机未独立提取；其 SHA256 仅作为 runner 回执：MSI 为 3f556299972f208ee5f85a7436096e884e255816f380cf4506747c8bd9f42677，NSIS 为 98b6700d0e3152e0a0de3c3f369b03665af9167e79284edf8033156408776123。

startup-diagnostics 的权限字段来自 PowerShell harness 当前 token，不能证明目标应用进程普通 / 管理员权限。沙箱根、DPR=1 和强制收尾不替代默认安装位置 data、物理 DPI、正常退出 / 锁释放。CDP 图仍有透明侧栏黑底和部分低对比，根因未确认，必须补同状态整窗对照；云母 / 原生外观未验收。codeSigned / releaseAccepted 为 false。原始首屏见 [MSI](evidence/windows/msi-startup/main-window.png)及 [NSIS](evidence/windows/nsis-startup/main-window.png)。

## 归档与剩余门禁

[归档核验](archive-verification.json)：139 项证据摘要、138 份来源无损 roundtrip 全部通过，8 项编译产物 / 安装器仅保留摘要；138 个无关脏路径未变。明确选择的 JSON / PNG / JPEG 原字节保留，日志 / AX / JSONL 为 mtime=0 gzip；[来源](evidence/source-provenance.json)、[SHA256SUMS](evidence/SHA256SUMS)与 methods 外部方法源可复核。不复制数据根、偏好 / 备份载荷或 WebView profile。

修复与完整 UI 474 / Rust 866 项回归见 [时钟边界记录](../20261011-调度时钟边界/README.md)。本轮闭合缓存场景的 10 次有效重启和 100 次有效概览进入局部检查；八项 TASK.required_checks 仍开放，整体 RCP failed 表达版本验收未完成，不是本轮局部测试失败。

后续依次推进同机官方 0.1.9 配对冷 / 热各 ≥20 次、启动可交互 p95 增量 ≤100ms 且 ≤5%、十分钟 CPU 增量 ≤0.5 百分点 / RSS ≤10MiB、主线程无 >50ms 任务；旧探索性 +3.66913 百分点仍为待调查信号。真实 stable 24h / beta 6h、睡眠恢复与跨启动、最新 SHA 离线 / 慢网 / 超时 / 并发 / 通道隔离，以及双平台原生 / 安装 / 真实更新 / 事件 / 正式签名仍待验。模拟时钟、缓存前台或 runner smoke 均不替代这些门禁。

后续真机项目及顺序见 [跨平台交接](../20261011-跨平台候选与真机交接/README.md)。IMP / TASK 保持 in_progress；未晋升核心事实、创建关闭证书、迁移真实用户数据、替换日常安装或公开发布，release_authorized / release_accepted 为 false。
