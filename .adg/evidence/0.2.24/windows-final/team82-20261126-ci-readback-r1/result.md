# TEAM82 主线 CI 与 W2 环境读回

2026-11-26。主线为 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，治理为 `633f0d086438592aeacc2ce5ae8757b1c0fa3d9b`。本轮要证伪的判断：这一 SHA 的安装器构建与同源主线 CI 均通过，且本机注册表读回可确认普通用户 per-user MSI 前提。两项均无足够依据，整版 `accepted:false`。

| 项目 | 状态 | 证据 |
| --- | --- | --- |
| 同一 SHA 的 Windows 制品 | 构建成功，复用摘要 | [Release 45630909723](https://github.com/gzers/opencodex-desktop/actions/runs/45630909723) · [两包摘要](../team81-20261126-resume-r1/raw/digest-recheck.json) |
| 严格 Windows Clippy | fail | [backend-windows 139263517735](https://github.com/gzers/opencodex-desktop/actions/runs/45630906743/job/139263517735)。首错 `windows_official_runtime024/mod.rs:15:29` 的未使用导出。另有生命周期在保持 owner `MutexGuard` 时 `await` 的两处告警；日志未证明它们造成 TEAM81 的首次 Status 问题。[错误摘录](ci-windows-errors.txt) |
| 共享与 Mac CI | fail | 同次 `backend` 在 `program_image.rs:540` 命中工具链废弃警告；`frontend` 在 `production-boot.mjs:47` 的注册 ID 数组多一个 `null`；`release-tools` 的 Mac updater 扩展头拒绝用例失败。[错误摘录](ci-other-errors.txt) |
| 普通用户 MSI 前提 | blocked | HKLM 64／32 位视图和 `marenben` 已加载 hive 未读到 `DisableMSI`。该读回无法推翻 [TEAM79 实测 1625](../team79-20261124-r1/sweep/summary.json)，也无法证明有效策略允许安装。[注册表输出](msi-policy-readback.txt) |
| TEAM81／W2 | blocked | Mac 共享修复及新制品未交付。本机仍缺可独立断网并保持可操控的桌面，`marenben` 会话为 Disc。W2 等两端同一 main SHA 冻结及实测普通用户安装前提。[现有实机状态表](../team82-20261126-preflight-r1/result.md) |

严格 CI 失败是独立于实机缺包首错 `managed_dependency_missing` 的证据。TEAM82 的共享首次供应、UI 和打包继续由 Mac 修改。Windows 修复候选接入时复查 `backend-windows`；如仍失败，可从上述 Windows 独有诊断定位，保持 owner 串行化、原预算与未知拒绝。共享 `program_image.rs`、生产 UI 和 Mac updater 测试结果交集成线核对。

本轮只读远端、CI、会话及注册表，未运行安装器、改策略、启动 W2 或产生新测试夹具。历史失败和账户保留。
