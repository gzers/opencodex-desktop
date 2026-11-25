# TEAM81 Windows 隔离启动复验

2026-11-25。固定候选源码 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，合同 `docs/governance-main@1b590767766070b395e2afddc52e1c04e5eca411`，Actions `45630909723`，Windows 制品 `15030939242`。本轮 `accepted:false`。只使用一个真实 Windows 桌面会话，未触发业务同步、模型请求、W2、双 SID 或发布。未改源码、网络策略、账号、日常配置和官方客户端。TEAM80 现场保留。[机器回执](summary.json) · [待证伪判断](raw/hypotheses.md)

| 核验项 | 结果 | 实测与证据 |
| --- | --- | --- |
| 候选 ZIP、MSI、NSIS 摘要 | pass | Windows 复算三项 SHA-256 均与固定候选一致。[摘要](raw/digest-verification.json) |
| MSI 管理解包和负样本 | pass | `msiexec /a` 退出码 0。独立负样本仅省略 `runtime-bundle`，保留文件摘要逐项一致。NSIS 本轮仅复算摘要。[解包日志](raw/msi-admin.log) · [载荷清单](raw/payload-inventory.json) |
| 完整包断网自动解包、校验、进度及「无需联网」 | blocked | 当前桌面经 RDP 连接，默认路由仍存在。Windows Sandbox 未找到，Hyper-V 查询权限不足。未找到可独立证实断网且保持交互的桌面。完整载荷还未启动，本轮完整包沙箱尚未创建。[离线预检](raw/offline-preflight.json) · [会话](raw/windows-sessions.txt) |
| 缺包沙箱路径 | pass | 真桌面 session 2 启动，安装根、Desktop 数据和 OpenCodex HOME 指向独立沙箱。[启动回执](raw/missing-launch.json) · [配置截图](raw/missing-installation.jpg) |
| 缺包暂缓、重开及主动确认门槛 | pass | 「稍后处理」进入「运行环境待处理」。通过「查看环境准备」重开。开始安装前要求用户勾选确认，环境未就绪时 Start 禁用。[控件观察](raw/missing-ui-observations.json) · [操作前截图](raw/missing-wizard-before.jpg) |
| 未主动触发时的网络证明 | blocked | 配置的环回观察代理未收到请求。点击前抓到 WebView 的 443 TCP 连接，未识别其用途。保留 UI 门槛的通过结果，下载流量的完整判定仍无证据。[点击前快照](raw/missing-before-action.json) · [代理观察](raw/missing-proxy-observation.json) |
| 主动触发下载 | fail | 勾选确认后点击「开始安装」，立即走入缺少集成运行程序分支，代理收到 0 个请求。[失败截图](raw/missing-wizard-after.jpg) · [源码分支](raw/runtime-install-guard.txt) |
| 就绪后首次 Status、5 秒超时、未知提示、重试与诊断 | blocked | 未观察到本轮环境就绪后的首次 Status，无耗时。缺包时的 Start 禁用仅属于未就绪阶段。[离线预检](raw/offline-preflight.json) · [控件观察](raw/missing-ui-observations.json) |
| 确认空闲后的 Start → Status → Stop | blocked | 未获得可信空闲结果，未执行该序列。预算保持 30／5／10 秒，未知拒绝。 |
| 本轮进程、监听清理 | pass | 正常关闭精确目标实例，PID 75624 及六个 WebView 子进程均已退出，观察代理端口 64081 无监听。TEAM80、TEAM79 和日常进程未操作。[关闭后快照](raw/process-cleanup.json) |
| 本轮夹具及下载残留清理 | blocked | 自动审批以 `blocked by policy` 拒绝删除命令。缺包现场两个目录和 38 个下载临时文件留在本机。缺包现场另已归档并校验全部 397 个成员。已核验的安装器、ZIP 和完整载荷留待离线复验。[归档清单](raw/missing-fixture-archive.json) · [残留清单](raw/file-cleanup-blocked.json) |

首个意外产品错误出现在 22:49:14（北京时间）主动点击安装后。界面显示「集成运行程序尚未准备完成，请先恢复内置资源」。固定源码 `commands/runtime.rs:572` 返回 `managed_dependency_missing`，发生在下载之前。22:43:49 的 `manage backups / os error 2` 来自故意移除资源的负样本启动，原日志也已保留。[启动日志](raw/missing-app.log) · [源码分支](raw/runtime-install-guard.txt)

离线复验还需要一个可在断网后继续交互的真实 Windows 控制台或独立 VM 桌面。缺包主动下载路径的产品失败需要集成线处理。