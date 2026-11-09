# RDP 重连后 user1 单次启动对照

2026-11-09，Asia/Shanghai。用户说明 RDP 已重新连接，并结合此前“再来一次”的要求，追加本独立批次；原批次不覆盖。诊断 accepted:false，产品启动仍未通过。

## 对照结论

Windows TerminalServices-LocalSessionManager 原事件证实：上一轮会话 2 在 19:22:15 断开，19:24:20 启动失败，19:37:04 重连。本轮控制器运行于 19:45:00—19:51:43，41 次会话采样全部 Active，该区间未记录会话连接、断开或重连事件；仍在 19:48:01 出现同一 lifecycle_start_exit，真实 exit_code=1，duration_ms=10325。

因此，RDP 断线不是本次失败复现的必要条件，不能仅凭重连把问题记为已修复；这不排除断线影响上一轮的表现，也不确定更深层退出原因。原 30 秒确认观察到 30174 ms 后结束，是后续现象，不能覆盖此前实际启动退出。

对照原证据：[rdp-comparison.json](user1/rdp-comparison.json)、[Windows 会话事件](user1/rdp-session-events.json)、[41 次原始采样](user1/rdp-session-samples.raw.log)。上一轮原 83 个封存文件大小及 SHA256 再核验全部一致，根 manifest.json 仍只绑定原批次，不冒充包含本补充目录。

## 环境与固定输入

- 现有本地工作站 EZIO-MINI，宿主实际 OS Caption 为 Microsoft Windows Server 2025 Datacenter，build 26100 / UBR 33438；以 host-environment.json 为准，不能只由内核构建号推断为 Windows 11。非临时 Actions VM。
- RDP 会话 2；唯一新建普通账户 ocxaccu1eca8aba48，SID S-1-5-21-587408064-856152790-2583328096-1010；真实 Profile C:/Users/ocxaccu1eca8aba48。应用 PID 51400，窗口原生可见，实际 elevated:false / elevationType:1；控制器 elevated:true / type:2，二者区分记录。
- 复用 source 8445d65dca0af76c3275f8617a76c1a726fb8873 / run 42602603906 / artifact 13667461176。ZIP 304962844 字节、SHA256 048b4d344c1bd8bbf81adb0a73571f14eaa697e122aafdad7ea6aeda7e42f274，manifest 9 个成员逐个核验；程序树摘要 76ae696b7f151ff520bfdd2360c573f7adb3e49b95639b5c9cba4fb567746f84。
- 运行包 2.97.0-desktop023.lifecycle.2，15413257 字节，SHA256 c02f4710496775d8f4dfe271bd90d6afdf7dbf960f6e696637271ab9f37bc70b。首次原生供应成功后调用实际 Desktop process_action。
- 诊断脚本原样复用 codex/0.2.23-integration@dc919da846260b0c11bebc04c56fd32963929d45；本轮没有产品或诊断驱动变更。当前读取合同为 docs/governance-main@949dec24685db599404e1249d1e2897caa00a455；采集器 environment.json 的 documentHandoff 保留首次交接来源 e6366f07287467c508592af18ca065935e22a451，本轮读取版本另见 comparison-provenance.json。

## 四项回交

| 项目 | 实际结果 | 原证据 |
|---|---|---|
| ① 启动回执及 PID/端口 | 本批 Start 共 1 次、原回执 1 条、成功 0 条，无唯一成功回执；未观察到受保护 runtime-port.json，没有服务 PID/端口可核对，启动链 PID 不能代替服务 PID。 | action-001.raw.json、action-001.json、user1-result.json、owned-health-observations.json |
| ② 启动 trace 与首个失败 | app.log 第 13 行，10325 ms 实际 exit1 / lifecycle_start_exit；未观察到固定 ocxd-lifetime-admission-failed 或 lifetime_admission 的 stage/kind/win32，不能推断准入成功或被拒。之后 30174 ms 未确认健康。 | desktop-logs/app.log:13、event-order.json、user1-result.json |
| ③ 正常停止与真实退出码 | Start 未成功，正常 Stop 未执行，无 Stop trace/退出码。清理前保留的 cmd PID 45512 原句柄 exit1；诊断 driver PID 29000 原句柄 exit1。本批未保留 Node/Bun 启动进程身份，不补造其退出码。控制器 bootstrap exit0 仅表示采集器完成。 | original-exits-before-cleanup.json、driver-exit.json、child-finished.json、controller-finished.json |
| ④ 账户/Profile 清理 | accountRemoved/profileRemoved/userDirectoryRemoved/desktopAclRestored/programUnchanged 均 true；identityUnproven:false，residualProcesses:[]。另只读核验账户和原用户目录均已消失，控制器已退出。 | controller-finished.json、post-cleanup-check.json |

上述 user1 文件位于 user1/ 子目录。启动回执原文：

```json
{"lifecycleState":"starting_failed","result":null,"canStart":true,"canStop":false,"canRestart":false,"failure":{"code":"lifecycle_start_exit","message":"服务启动进程已退出，请查看诊断日志后重试。"}}
```

## 接续处理边界

本轮实际 Start 仅一次，原启动 30 秒／停止 10 秒／状态 5 秒及身份、健康、退出和还原断言保持；启动失败后没有补充 Stop 或再次 Start。没有 core-sweep、双 SID、安装器、真实业务同步或发布，也没有日常配置变更。账户/Profile/私有安装清理完成；D:/ocxu1-20261109-194449 保留原诊断输出，前序封存 ZIP、程序及执行器作为复现输入保留。

下一步应沿官方启动链排查 exit1，区分启动入口、运行包和原生归属准入问题。当前没有具体准入阶段，不能直接认定 token-pointer 等既有缺陷已修复，也不能仅凭退出码定位修复点。先核对现有源码和原证据；若需在产品或维护包增加脱敏启动输出及准入诊断，先说明方案，再实施。本次不执行后继产品返修或更多轮次。
