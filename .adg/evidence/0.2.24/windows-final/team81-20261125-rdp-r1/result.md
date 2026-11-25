# TEAM81 完整包 RDP 续验预检

2026-11-25。用户只能使用 RDP，并确认本次续验无需判断是否断网。当前为 15119 的单用户活动 RDP session 2，Windows 网络配置报告可访问互联网。原 TEAM81 断网项继续标为未验证。固定候选 a69ae55a3fe21b58f415da727b0d1bdff8f37568，Actions 45630909723、Windows artifact 15030939242，合同 1b590767766070b395e2afddc52e1c04e5eca411。本轮 accepted:false。[预检](raw/rdp-preflight.json) · [原 TEAM81 结果](../team81-20261125-r1/result.md)

本次待证伪判断：完整随包资源在全新沙箱自动解包并校验，界面展示进度。环境就绪后，首次 Status 从请求发出起 5 秒内给出明确状态或进入「运行状态未知」。未知时可重试或查看诊断，Start 禁用。仅在明确空闲后执行 Start → Status → Stop，预算 30／5／10 秒。

| 项目 | 结果 | 观察 |
| --- | --- | --- |
| 候选 ZIP、MSI、NSIS 摘要及完整载荷 | pass，复用 | 原制品摘要和独立副本十项载荷核验可复用。[原制品摘要](../team81-20261125-r1/raw/digest-verification.json) |
| 完整包联网自动准备、校验及进度 | blocked | 前一次合并启动与 WebView 采集的命令被自动审批拒绝，返回 blocked by policy。本次只读核对未找到获准的 RDP 启动入口，未再次启动。原离线入口仍限定本机控制台。 |
| 完整包断网自动准备 | 未验证 | 用户本次要求继续使用联网 RDP。 |
| 缺包暂缓、重开、主动确认及未就绪时禁用 Start | pass，复用 | [原控件观察](../team81-20261125-r1/raw/missing-ui-observations.json) |
| 缺包主动下载 | fail，复用 | 固定候选原首错为下载前 managed_dependency_missing。本次未重跑。[原源码分支](../team81-20261125-r1/raw/runtime-install-guard.txt) |
| 未主动触发下载的网络证明 | blocked，复用 | 原观察代理未收到请求，较早的 WebView 443 连接用途未确定。 |
| 首次 Status 与未知交互 | blocked | 未启动新实例，无 Status 请求和耗时。 |
| 明确空闲后 Start → Status → Stop | blocked | 未获得可信空闲状态，未执行。 |
| 本次新增测试残留 | 已核对 | 本轮 RDP 目录为空，无沙箱和启动回执，两处完整候选路径未查到运行中的实例。已核验的完整副本、TEAM80、此前缺包现场和日常实例保留。 |

自动审批拒绝发生在候选启动之前，是本次续验首个阻塞。通过指定 127.0.0.1:3060 代理尝试 fetch，Schannel 与 OpenSSL 均在 TLS 握手失败。记录中的分支位置来自本地已知引用，远端是否超前尚无法确认。[Git 传输记录](raw/git-proxy-transport.txt)
