# Windows 周期窗口前台恢复

2026-10-12 19:27（UTC+8），重新查询RDP session2为Active后，通过Computer Use重新选择并激活原候选窗口。证据归 `.adg/evidence/OCX-0110-20261012-WIN-CYCLE-FOREGROUND-RESUME/`。

仍为24h观测原PID38924、主HWND461032、旧afff490源码与EXE；没有启动新应用、手动查询、改偏好/调度或时钟/timer。生产app_foreground与effects.foreground/visible均true，原生DPI192/DWM attr2/foreground=true，真实截图核验云母恢复。

管理器lastCheckedAt仍2026-10-12T10:01:07.936639900+00:00，面板仍为初始2.83.0；完整调度文件与初始化基线一致，stable/面板next_due仍为2026-10-13 18:01:07/09（UTC+8），没有因这次前台唤醒重置截止。生产store观察记录尚无面板查询变化。

这些是实际状态/文件观察，没有原始网络或native命令计数仪器，不能据此声称全部自动请求次数为零。不是系统睡眠/恢复场景，也不是24h终态。周期监测继续持有同一应用生命周期；源码main已到4852820，但运行窗口仍旧afff，不据此认证新SHA原生门禁。

8个原始文件集合/哈希及原窗口/截止断言冻结并重验通过。其余原门禁、真实周期/同机性能、TASK原件和脚本二次确认保持，IMP in_progress，原预算/既有failed不变，不关闭或发布。
