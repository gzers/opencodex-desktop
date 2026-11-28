# Windows TEAM85 普通账户复验交接

治理基线 fa336d097。固定源码 9a452c37d3005119042d81ee1ae8f62c4920c4d0。同源 CI 46040314845，Windows 构建 46049421165，制品 15224497695。

验收状态 accepted:false。断开桌面网络按约定跳过。实测使用既有普通账户 marenben，RDP session 3，令牌无本地管理员组。保留原 Full 窗口、原测试根和历史证据。

| 验证项 | 状态 | 证据与首错 |
| --- | --- | --- |
| ZIP、MSI、NSIS 摘要 | pass | 本机重算匹配，见 raw/digest-verification.json。首次下载定位错误 Missing artifact redirect，随后通过已认证重定向取得同一制品。 |
| NSIS 运行夹具来源 | pass | 从已核验 NSIS 提取。八个核心成员按长度和摘要核对，另保留两份资源声明。见 raw/payload.json。本轮通过该夹具启动，安装器未运行。 |
| 完整包首开自动准备、阶段和无需联网提示 | pass | 普通桌面真实首开，应用日志用时 253.281 秒后记录 provisioned and verified，当前镜像为 win32-x64 2.99.0。同一 attempt 由 preparing 经 verifying、extracting、assembling、activating 至 supplied，无阶段错误。观察区间内展开阶段约 194.6 秒、组装约 52.9 秒。见首开回执、采集和时间线。 |
| 缺包首开提示与拒绝启动 | pass | 同一普通账户 session 3 的独立 Missing 根。初次 attempt 在观察开始后 260 ms 采到 missing_archive，界面出现下载和稍后处理，Start 不出现。见 raw/missing。 |
| 缺包暂缓 | pass | 新隔离 Missing-r2 中，marenben 在真实桌面点「稍后处理」，界面转为「运行环境待处理」，提供「查看环境准备」，Start 不出现。03:26:47.692Z 的 WebView 快照同步捕获重开入口及 Start 缺席。见 raw/missing-r2。 |
| 缺包重开 | pass | marenben 从暂缓页点「查看环境准备」。03:29:18.485Z 的 WebView 快照显示缺包准备页、下载与稍后处理重新出现，Start 仍缺席。见 raw/missing-r2/ipc-reopened-snapshot.jsonl。原 Missing 于 03:24:32Z 确认退出，最后日志为窗口失焦，退出原因未知，原回执见 raw/collect-20261128T0324323717112Z.json。 |
| 确认前原生零 GET | blocked | 03:44:54.135Z 确认框已显示，观察于 03:49:54.190Z 结束。WebView 观察未覆盖原生 HTTPS，不能签零 GET。见 raw/missing-r2/ipc-confirm-observer.jsonl。 |
| 主动确认、下载阶段终态与拒绝启动 | pass | 用户在 Missing-r2 确认后报告失败。03:54:17.932Z 只读回执显示新 attempt e185ddd6-1ab5-40c7-befc-f42a20b55693，revision 7，failedPhase=downloading，failure=unavailable，retryable=true，Start 不出现。见 raw/missing-r2/preparation-after-confirm.json。 |
| 公开运行包供应 | fail | 03:52:12.008Z 原生日志首错 bundle_download_unavailable，官方地址未提供完整运行资源。固定源码只在 HTTP 响应落入不支持的状态分支时产生该详情，日志未记录具体状态码。见 raw/missing-r2/app-log-after-confirm.json。后续从 agent shell 经既定代理对固定资产地址及发布标签 API 发出独立 HEAD，均返回 404。该探测没有桌面 attemptId，不能反填该次 GET 的状态码。见 raw/missing-r2/public-release-head.json。 |
| 下载确认、传输与终态的同一 attemptId | blocked | 确认观察结束早于失败日志，未捕获确认 IPC 或原生传输，后验 attemptId 只能证明终态所属尝试。 |
| 首次 Status 原始 5 秒计时 | blocked | 界面于 00:40:07.889Z 采到等待首次观测，00:40:08.196Z 采到未知。没有首次命令的请求和响应时间戳。见 raw/ipc-observation.jsonl。 |
| 状态未知拒绝、重试与诊断入口 | pass | 环境就绪后出现运行状态未知、重试状态及查看诊断，Start 不出现。对原页面做后续只读 Status，218 ms 返回 live / unreachable，canStart、canStop、canRestart 均为 false。PID 与端口记录缺席。该次返回在预算内，属于不可达。见 raw/post-preparation-readback.json。 |
| Doctor 实际报告 | fail | 原页面调用 3 ms 返回 code 1、application is not configured yet，未产出报告。旧结果保留未实测。 |
| PID-only 拒绝 | blocked | 已定位固定源码中的预期 windows024_owner_unknown，尚无独立普通桌面实测。 |
| 端口设置写锁拒绝 | blocked | 自动审批拒绝了本机实写锁探针，保留该决定。源码中缩短窗口单测不作生产预算证据。 |
| Start → Status → Stop，30／5／10 秒 | blocked | 后续只读 Status 返回不可达，缺少可信空闲确认，未调用 Start。 |
| 清理 | blocked | Full 与 Missing-r2 窗口及任务根留作复查，原 Missing 已退出，退出原因未知。公共暂存、既有账户/Profile 与历史证据保留。本次只读采集，未操作进程或清理失败现场。 |

首个影响功能的结果是运行环境就绪后 Status 仍不可达。固定源码的 Windows 2.99 路径在验证托管运行环境后读取进程内 owner 槽。槽缺席、锁竞争、owner 缺席等均映射不可达，现有回执只能定位到共同返回，具体分支待证。Doctor 另将多种入口或执行错误折叠为 NotConfigured，原始过程错误当前不可见。

候选代码没有改动。待证伪判断见 hypotheses.md。raw/receipt-sha256.json 记录从接收目录归档的原始文件摘要。强制清理不能签作正常 Stop 或 Closed。
