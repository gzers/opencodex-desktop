# TEAM85 Windows 普通账户复验接收表

2026-11-28。验收结论 `accepted:false`。源码 `9a452c37d3005119042d81ee1ae8f62c4920c4d0`。真实普通账户在 RDP session 3 使用从已核验 NSIS 解包的独立夹具，安装器本身未运行。接收时从 Git 复算原清单 26 项及 Doctor R2 的 4 项 SHA-256，全部一致。[Windows 实机原回执](https://github.com/gzers/opencodex-desktop/blob/86f8aef7223d88b7aff3355b2bc8d2035dcf22e2/.adg/evidence/0.2.24/windows-final/team85-20261128-9a452c37d-ordinary-r1/result.md) · [冷启动与 Doctor 补充回执](https://github.com/gzers/opencodex-desktop/blob/86f8aef7223d88b7aff3355b2bc8d2035dcf22e2/.adg/evidence/0.2.24/windows-final/team85-20261128-9a452c37d-ordinary-r1/cold-start-handoff.md) · [同源安装器与签名](result.md)

| 项目 | 状态 | 证据与待办 |
| --- | --- | --- |
| 制品与夹具来源 | 通过 | Actions ZIP、MSI、NSIS 摘要与夹具成员均已核验。Minisign 通过，Authenticode 为 `NotSigned`。 |
| 完整包首开 | 通过 | 同一 attempt 展示本地准备各阶段和「无需联网」，应用日志 253.281 秒至 `supplied`。断开桌面网络按既定决定跳过。 |
| 缺包交互 | 通过 | 提示、暂缓、重开、主动确认后的可重试下载失败均有实机回执，Start 一直不可用。 |
| 原生下载与供应 | 阻塞 | 原桌面首错 `bundle_download_unavailable`。独立 shell HEAD 返回 404，原生 GET 的 HTTP 状态仍未知。确认前零原生 GET、确认与终态串联、同源归档供应均待证。 |
| 首次 Status 界面 | 通过 | 环境就绪后首次 IPC 241.1 ms 返回 `unreachable`、`canStart:false`，287.5 ms 采到未知、重试和诊断入口。原生探针复现无 owner 时不可达，首次 GUI IPC 的具体拒绝阶段未知。 |
| 可信空闲与启停 | 阻塞 | 未调用 Start。源码确认 Status 要求已有 owner，GUI 与 CLI 的共同 Start 在空槽处可仅凭两条运行记录缺席进入端口准备。未知时能否创建真实进程未执行验证。 |
| Doctor | 失败 | 隔离 cwd 缺席，1.6 ms 返回 `not_configured`。补齐独立夹具 cwd 后 IPC 7564 ms 返回 `timeout`，子进程等待预算 5 秒。合成探针区分进程退出和 stdout EOF，实机超时阶段与后代退场待证。外层采集器退出码未知，IPC 原件单独保留。 |
| PID-only 与端口写锁 | 阻塞 | 组件 6 过、4 跳过，写锁子窗口 750 ms。普通桌面尚无实测，自动审批拒绝本机实写锁探针。 |
| 清理 | 未完成 | 原现场保留，新 Full 管理器 PID 108100 原回执仍打开，Doctor 后代退场待证。自动审批拒绝删除四个诊断构建资源文件。强制清理不能作为正常 Stop 或 Closed 证据。 |

诊断分支 `1dac7c065413347ebc56b6123a8cf10c517449aa` 提供显式沙箱下的 owner 槽拒绝分类，绑定沙箱定向回归 7 过、4 跳过。诊断 EXE SHA-256 `46769ea6c88fa6a43816fecffab423757a4eec8410ad2ae5896905869b46847e`。普通桌面阶段回执、可信空闲和新 MSI／NSIS 均未完成。该分支作为诊断输入接收。[诊断提交](https://github.com/gzers/opencodex-desktop/commit/1dac7c065413347ebc56b6123a8cf10c517449aa)
