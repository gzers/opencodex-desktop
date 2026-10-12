# Windows 最终 CI 与未闭合门禁审计

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182；本页为继续执行的审计记录，不是版本关闭、正式RCP或验收证书。证据归 `.adg/evidence/OCX-0110-20261012-WIN-FINAL-CI-AUDIT/`。

## 精确合并 CI 已结束

main `afff490a430a81f8c233d44928aa3681a23e91e0` 的 CI38213445710 attempt1 于2026-10-12 15:05:46（UTC+8）completed/success，最终读取为15:06:30。六项job全部completed/success：windows-smoke-evidence、backend-windows、release-tools、backend、frontend、build。macOS arm64 bundle实际构建步骤也通过，无条件跳过项冒充通过。首末状态保留在不同证据目录，不改写先前in_progress原件。

同SHA Windows候选38213528684已success，ZIP/MSI/提取EXE identity与本机startup源码均已验证，详见[合并制品主题原生回验](../20261012-Windows合并制品主题原生回验/README.md)。构建/runner安装通过不能替代Windows当前设备管理员、物理DPI、日常旁置安装或性能门禁。

当前仍是原容器的唯一普通权限合并测试PID29936，路径、创建时间、保留supervisor和源码一致；当前独立快照仅一个OpenCodex应用进程。日常0.1.9 EXE摘要仍为 `43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67`，没有替换日常安装或操作真实用户载荷。

## 完整目标审计仍为未达成

`requirement-audit.json`按原目标逐项列明已证、局部、缺失/不足的实际证据，保留普通/管理员、DPI/云母/标题栏、安装data/自定义根/卸载、真实更新详情与进度、备份恢复/轮换/固定/安全、树表全矩阵、事件与在途通道、真实30min状态退避、同机官方0.1.9配对性能、真实stable24h/beta6h/panel24h及睡眠/跨启动范围。没有围绕本轮已通过项缩小成功定义。

结论：

- 精确main CI、候选来源/资产摘要和当前单实例身份有真实证据；源码/文档提交依分支归属已推送。
- 备份及树表、持久主题/历史导入、具体事件恢复/已完成查询通道隔离均只按各自记录范围通过，不扩展成全部AC。
- 真实安装/更新仍被原scripts_required二次确认门禁阻断；自定义根引用切换保留runtime_unverified；junction夹具自动审批拒绝仍not_run。
- 管理员实机、其它物理DPI/标题栏操作、实际manager安装卸载、完整事务矩阵、真实30min退避、同机配对性能和真实周期缺完整证据。既有性能failed与原预算保持；加速状态、历史日期夹具、未来截止和后台存活均不抵真实长期/前台性能验收。

## 人工接续条件与桌面状态变化

原任务与脚本许可在前几轮接续中持续缺失；main CI现已结束。写入审计时的`query session`原始输出显示15119/session2已从先前Disconnected变为Active；再次独立读取也为Active。代理原先沿用“断开”的状态标注已在提交前纠正，以本轮原始会话输出为准；当前可以继续独立原生工作，不标记goal blocked。

1. 原 `.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json` 仍不存在。应从原执行机同步真实对象并验证其digest/scope/required_checks，不能依据旧RCP的digest重造或放宽合同。同步前不生成正式RCP/证书。
2. 15119/session2当前Active，可重新选择唯一目标窗口继续前台GUI与原生观测；管理员实际令牌与其它物理DPI仍分别待验。不将连接状态本身计为界面或性能通过。
3. 按IMP-01 FZ-50，需要对已经准备的单次精确 `@bitkyc08/opencodex@2.82.0`、包含 `bun@1.4.2 postinstall` 的脚本重跑作显式二次确认。计划与摘要见前次 `install-script-confirmation-plan.json`；本轮未获得许可、未开allowScripts、未执行脚本。一般目标续行不替代这项原合同的确认。

脚本和原TASK仍需人工补齐，但桌面恢复后应继续可独立完成的原生工作，goal保持active；不改变IMP的in_progress或TASK/AC门禁。目标范围不变。原实例和测试根保留，后续依实际进程状态决定复用或受控重启；不自动再打开并行实例。

## 连接恢复后的普通权限原生操作

重新选择唯一afff进程窗口HWND852848后，真实系统最大化按钮使IsZoomed=true；Alt+Space弹出真实系统菜单，R还原后IsZoomed=false；系统最小化按钮使原主HWND的IsIconic=true，document.visibility=hidden、effects.visible=false、ambientAllowed/animated=false；激活同一窗口后IsIconic=false、前台true、DWM云母属性2恢复。原生DPI仍192、DPR2，最大化WebView2560×1212、还原1180×760，均按实测记录。

保留工具边界：AX还原按钮输入没有使窗口还原，未计该输入通过；实际还原证明来自后续系统菜单。最小化后Process.MainWindowHandle误选26×26托盘窗口，原错误选窗结果保留且不参与结论；修正本地只读观测工具，以先前由Computer Use选定的主HWND并独立验证所属PID。上述仅覆盖普通/200%具体输入，管理员/其它物理DPI/拖动缩放/关闭托盘/Snap仍待验。

## 真实退避观测准备失败，不计30分钟通过

连接恢复后尝试在同一实例取得生产调度的真实错误与30min等待证据，临时写入测试调度文件schema99，未调整时钟。观测启动未成功：Tauri内部invoke属性writable=false/configurable=false，所谓透明调用记录器赋值没有生效；planCalls=0不能证明生产调度器没有运行。诊断与失败日志见 `.adg/evidence/OCX-0110-20261012-WIN-REAL-BACKOFF/`。

监测进程已terminal，原调度JSON逐字恢复、原自动检查false恢复，记录器未安装且原invoke保持；未因观测失败重启仍存活的产品进程，没有把这次prepare记作真实30min退避或性能证据。实际内部retry floor起点未观测，不能推断或绕过；后续先验证可观测边界，保留实际单调时钟、原native调用和保护。
