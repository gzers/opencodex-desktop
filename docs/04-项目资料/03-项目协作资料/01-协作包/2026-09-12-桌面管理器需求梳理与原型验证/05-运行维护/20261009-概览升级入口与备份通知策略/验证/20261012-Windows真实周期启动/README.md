# Windows 真实周期启动，终态仍待验

2026-10-12，唯一 `main@afff490` 普通PID38924、监督器44780、同一隔离根。管理员链路已结束，未重叠运行第二应用。当前15119/session2重新检查为Disconnected、应用foreground=false；停止桌面输入，不将断开会话算成连续前台周期通过。

## Beta 6h 初始化失败

原件：`.adg/evidence/OCX-0110-20261012-WIN-REAL-BETA-CYCLE/`。

17:57:20尝试以真实beta查询建立成功基线，原生命令返回failed，管理器错误明确为更新下载/连接失败。源调度保留manager-beta failures=1、last_success=null；不是成功6h截止。立即恢复stable/auto-check false，偏好原字节一致，没有启动6h监测器。

随后使用本机已认证GitHub API只读查询beta发布tag，HTTP404/Not Found；认证仅在内存，公开证据没有凭据。生产beta endpoint仍为固化`releases/download/beta/latest.json`，没有改端点、注入成功结果、创建tag/发布、下调周期。此前外部网页读取没有可核验返回，不作为404依据；结论以归档API响应和实际原生命令失败为准。

归档校验12个文件通过只是确认失败准备和恢复证据完整；beta实际6h周期未启动、未通过。

## Stable 与面板真实24h基线

启动证据：`.adg/evidence/OCX-0110-20261012-WIN-REAL-STABLE-PANEL-CYCLE/`。18:01:07.936真实stable手动初始化查询up_to_date，18:01:09.537真实npm面板初始化查询成功取得2.83.0。然后通过真实偏好命令开启自动检查并重载生产store；没有系统时钟、timer、业务响应或截止注入。

| 对象 | 原生last_success（UTC epoch） | 原生next_due | 实际名义截止（UTC+8） |
|---|---:|---:|---|
| manager-stable | 1791799267 | 1791885667 | 2026-10-13 18:01:07 |
| panel-latest | 1791799269 | 1791885669 | 2026-10-13 18:01:09 |

两项failures=0、next_due-last_success=86400。初始化后不再发手动检查，不修改截止。此时foreground=false，未来须观察真实恢复/自动终态；未来时间或启用记录不能代替周期结果。

## 活监测与接续

监测exec session93170、PID18944（创建UTC10:04:20.663），保留应用38924及监督器44780的实际进程句柄，以防PID复用。18:05:26独立重新打开句柄确认监测/应用存活；不以旧状态文件单独认定liveness。

连续记录存放在代码工作树的忽略运行目录 `test/runtime/windows0110/cycles/stable-panel-20261012/`，不把仍在追加的记录当作冻结证据提交。`observations.ndjson`含只读原生状态与真实调度文件快照；`monitor-state.json`是可重建过程视图。每5秒核验持有句柄，正常每5分钟读一次原生状态，名义截止附近加密观察；不会输入界面、发检查、重启产品或改变时钟/计时器。观察失败不等同终止，会重观测同一生命周期。

最长26h观测：若出现下一次真实成功截止，仍只标记“需最终审计”；不自行通过全周期、原始网络/子进程计数或性能。若截止到达但仍无前台，记录等待eligible foreground。终态后尝试恢复原stable/auto-check false；失败需另行接续，不擅自重启应用。源程序监督器的48h上限保留。

启动冻结清单12个文件重验通过，不包括仍活监测的后续终态。需继续保留同一应用，避免手动更新、切通道、改调度或停止后冒充本轮连续观测；如实际中断，原件保留并按原契约重建样本，不缩短周期。

本轮原预算、既有性能failed、beta6h缺口、同机0.1.9配对性能、管理员完整交互/其余DPI、安装更新卸载、全事件与TASK原件/脚本二次确认继续开放。当前auto-check在隔离测试根为true仅用于真实周期观测；日常版和真实数据未替换。IMP in_progress，不颁发证书、不关闭或发布。
