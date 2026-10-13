# Windows 原周期到候选切换的只读审计

补齐原eae周期结束后到be7395a候选顺序启动的核对工具，避免将监测状态文件、窗口关闭或新的候选摘要直接当作原拥有者已收尾。工具只读现有证据、隔离数据及进程，不执行停止、启动、IPC、前台输入或配置写入。原件归`.adg/evidence/OCX-0110-20261013-WIN-CYCLE-TRANSITION-GUARD/`。

## 只读准入条件

工具复用已冻结且摘要绑定的真实周期审计器，继续检查原86400秒截止、单调时间、同源实例、稳定/面板分区和cached成功事实。切换前另外要求：

1. 原监测实际写入成功观察终态及finishedAtUtc，无未审阅的观察错误或提前成功。
2. 原native-cleanup回执属于eae/37276，恢复初始stable/auto=false；磁盘偏好也实际一致。
3. 原监督器自己的session记录为application-exited或stop-requested并已有finishedAtUtc。
4. 三个原进程的精确创建时间/实际句柄已终止，或OpenProcess明确返回不存在的ERROR_INVALID_PARAMETER87；访问拒绝等其他错误不能当已停止。
5. 独立全机CIM桌面进程查询实际为0，没有用只读worktree范围过滤漏掉其它桌面。

满足这些条件也只返回`read_only_prerequisites_met_requires_operator_review`，不授权前台、不自动切换，不抵拥有者exec80657/46484真实终态结果和人工范围审阅，不计完整周期或原生门禁通过。候选再次启动前仍须核对同SHA制品及真实令牌，只运行一个桌面实例。

14项内存工具夹具验证：运行中/中断监测、缺少success或cleanup、恢复标志与读回不一致、监督器仍在运行、原句柄未退出、查询权限拒绝、另一桌面、错误源/错误cleanup拥有者和观察错误均不能准入。合格的工具夹具也只返回审阅态，未把夹具注入产品、改时钟或业务响应。

## 本机实际结果

2026-10-13 14:53:57及14:58:18（北京时间）分别审计192/193份完整原周期前缀。实际37276/24300/3920保留句柄和创建时间均匹配且活，独立全机desktop count=1；原监测仍observing-real-deadlines、监督器running、无finished/cleanup回执。原stable/panel截止仍2026-10-13 22:55:39/40，26h上限保持，原观察失败0。

结果严格为`pending_original_owners_or_zero_instance`。磁盘auto=true是当前尚在观察的预期状态，工具不会为满足条件提前恢复；监测拥有者只有在其真实终态决策后才执行原cleanup，监督器仍须另行顺序收尾。没有发送stop.request、杀进程、重启或替换周期。

## 晚间使用

保留冻结源码目录，后续输出写新的独立证据目录；示例命令仅演示读取，当前未执行终态操作：

```powershell
& 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' `
  '.adg\evidence\OCX-0110-20261013-WIN-CYCLE-TRANSITION-GUARD\audit-transition.py' `
  --external-snapshot --output-name after-original-owner-terminal-20261013
```

工具拒绝重用输出名，源码归档冻结后拒绝继续写入归档内部，既有证据和原始日志不覆盖。若期满时尚未查到terminal，继续检查同一个真实拥有者句柄，不能因为观察超时创建替代运行。

22:50–23:10前台窗口仍须收到人的新开始信号；自动续行、工具审计或时钟到点不会取得焦点权限。先观察原22:55:39/40到期结果并审阅原终态，再决定是否有足够窗口完成顺序切换。原eae周期证据始终限定eae，不能把新be7395a源码或候选身份接上去宣称新候选已完成真实周期。

## 仍缺执行前提

本次只读环境查询中，Hyper-V模块可用，但Get-VM在普通身份下权限拒绝，CIM只返回宿主EZIO-MINI。不能据此声称无VM、已有可用VM或安装隔离就绪；未创建/启动VM、提升权限或改ACL。候选账户ocx0110_accept与独立目录仍不存在，官方0.1.9的真实Known Folder/Profile隔离尚未准入，配对性能不能启动。

原TASK182依旧缺失，FZ50第三方脚本显式许可未收到；当前只读提取候选摘要3a26098e…保持但未启动。安全安装环境、双令牌/物理DPI/云母/标题栏、数据保留、焦点/更新进度、备份/事件矩阵、最终源30min/beta6h/其余真实周期、原配对预算及历史failed继续开放。

27份全集合/字节摘要冻结重验通过，仅说明工具与本次审计证据完整。IMP in_progress，无正式RCP/关闭证书，不降低门禁、不提前关闭或发布。
