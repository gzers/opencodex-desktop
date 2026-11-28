# TEAM88 Windows 冷启动定位

accepted: false

- 清场 pass。旧候选 9a452c37d 已作废，PID 108100 已退出，原 SID/session 3 回执无剩余进程、监听，两把锁均可独占打开。正常 Stop 与 Closed 均为 false。原日志保留。
- 待证伪一：固定 1dac7c065 的首次 Status 在解析安装来源后，因当前进程无 owner 槽而返回未知。用 OCXD024_STATUS_TRACE=1 的首个拒绝阶段判断。
- 待证伪二：补齐隔离 cwd 后，Doctor 已进入子进程等待。需分别取得来源解析、入队、创建、退出、stdout EOF 和超时时刻。历史 7.5 秒只证明端到端失败，尚不能归因。
- 待证伪三：仅有空槽、记录缺席与可用端口无法证明冷启动空闲。Windows 在 Status 和 Start 共用原生证明，维持 5/30 秒上限；未知时不得写端口或创建进程。Stop 保持 10 秒。

## 给 Mac 集成线

固定 1dac7c065 只有 Status 原生 trace，Doctor 未分离 spawn、exit 和 stdout EOF。请在共享 Doctor 写集中提供阶段观察和定向验证，保留 5 秒等待，默认只读本地诊断。Windows 单轮固定二进制取证需要能独立观察进程和管道事件，若缺该观察能力，记录为 blocked，避免用代理现象给历史失败归因。受控下载供应继续由 Mac 单写。

## 单轮普通桌面取证

固定诊断源码 `1dac7c065413347ebc56b6123a8cf10c517449aa`，管理器 EXE SHA-256 `46769ea6c88fa6a43816fecffab423757a4eec8410ad2ae5896905869b46847e`。marenben 普通令牌、session 3，共用一轮新隔离根。收集器成功，应用首错另列。

| 项目 | 状态 | 实测 |
| --- | --- | --- |
| 定位首次 Status 空槽拒绝 | pass | IPC 307.9 ms，返回 `unreachable`、`canStart:false`。首拒绝阶段 `Lookup(NoMatchingSlot)` 为 23 µs，重试同阶段 11 µs。桌面显示未知、重试和诊断。 |
| 冷启动确认空闲 | fail | 没有可信空闲证明。 |
| Doctor 无 cwd | fail | 3.1 ms，`not_configured`，code 1。 |
| Doctor 补隔离 cwd | fail | 5405.1 ms，`timeout`，code 7。10:02:24.913Z 已创建 cmd.exe，随后 Node、Bun，以及 PowerShell、schtasks 和 sc。最后一个 PowerShell 样本在 10:02:30.309Z，观察窗内未见 launcher/Node/Bun 正常退出。首个可确认的失败发生在创建成功后的等待。 |
| Doctor 各阶段完整归因 | blocked | 固定二进制缺来源解析、入队、进程退出和 stdout EOF 的独立事件。采样无法区分命令执行和管道结束；5405.1 ms 是本轮端到端实测，历史 7.5 秒分开保留。 |
| 新候选实机验收 | blocked | 等原生可信空闲、同 SHA 双平台 CI 及安装器。 |

原始回执 SHA-256：

- `team88-combined-desktop-r1-20261128T0959066253888Z.json` `4f41dc162b295631dde12a8aa7264283d3b22704768977f289c20989aa76d3e2`
- `team88-combined-desktop-r1-20261128T0959066253888Z-ipc.json` `7efc0710b0ca76ba84bef3c95b807c08a92031865e391b07823fff64a87b9985`

新诊断管理器 PID 106544 单列清理，旧 108100 清场回执保持独立。原失败现场保留。
