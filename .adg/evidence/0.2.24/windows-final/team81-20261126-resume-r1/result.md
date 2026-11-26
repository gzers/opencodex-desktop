# TEAM81 Windows 断网续验前置复核

2026-11-26。用户本轮重新要求完整包断网实测。固定候选源码 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，Actions `45630909723`，Windows 制品 `15030939242`，合同 `docs/governance-main@1b590767766070b395e2afddc52e1c04e5eca411`。通过指定代理 fetch，主线和治理合同未推进，Windows handoff `02be801131e028df45f3c4a2034e92ac41d22c27` 已含共享修复请求。`accepted:false`。

待证伪判断：完整载荷在断网的真实桌面上以全新隔离 HOME 自动解包和校验，展示进度；独立缺包夹具仅在用户主动确认后发出下载。环境就绪后从首个 Status 请求发出计时；5 秒无法确认，展示「运行状态未知」、重试及诊断，禁用 Start。仅明确空闲后执行 Start → Status → Stop，对应预算 30／5／10 秒。

| 项目 | 结果 | 证据 |
| --- | --- | --- |
| 固定 ZIP、MSI、NSIS 摘要 | pass | 本机复算三项 SHA-256 和字节数，与账本一致。[本次复算](raw/digest-recheck.json) |
| 断网自动解包、校验、进度 | blocked | 当前为联网 RDP session 2。Hyper-V 服务在运行，当前令牌执行 Get-VM 被拒绝，Windows Sandbox 命令不可用。未确认有可操作、可断网的 Windows 测试桌面。[本次前置](raw/desktop-preflight.json) |
| 缺包暂缓、重开、主动确认门槛、环境未就绪时禁用 Start | pass，复用 | [原实机证据](../team81-20261125-r1/result.md) |
| 未主动确认时无下载 | blocked，复用 | 观察代理收到 0 个请求，确认前 WebView 的 443 连接用途仍未知。[原网络观察](../team81-20261125-r1/raw/missing-proxy-observation.json) |
| 主动确认后下载 | fail，复用 | 候选在 `apps/desktop/tauri/src/commands/runtime.rs:571` 返回 `managed_dependency_missing`，早于发出下载请求。修复及新冻结候选待集成线回交。[原首错](../team81-20261125-r1/raw/runtime-install-guard.txt) |
| 首次 Status 与超时后的未知交互 | blocked | 候选本轮未启动，未穿过环境准备门槛。已有 UI 定向回归属于构建证据，实机请求耗时待测。 |
| 明确空闲后的 Start → Status → Stop | blocked | 未有真实 Status 确认空闲。 |
| 本轮清理 | pass | 本轮仅做只读复查，未启动候选、创建新测试夹具或变更网络。既有 RDP 尝试目录仍为空。TEAM80 和此前缺包失败现场保留。 |

本轮首个环境错误为 Get-VM 权限不足。上次 RDP 启动与采集命令被自动审批以 `blocked by policy` 拒绝，未重放。[上次 RDP 证据](../team81-20261125-rdp-r1/result.md)
