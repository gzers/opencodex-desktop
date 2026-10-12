# Windows 最终eae恢复事件解除

2026-10-12，精确 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 的唯一普通应用PID37276。原件归 `.adg/evidence/OCX-0110-20261012-WIN-EAE-RESTORE-EVENT/`。本轮通过实际生产WebView调用原生命令，不注入事件、手工通知解除或伪造成功；RDP断开，鼠标/键盘及完整事件UI未验收。

## 实际失败、其他动作成功及连续候选恢复

当前system/stable/auto-check true/manual策略，新建真实备份 `bk_20261012151535_1301aad6`，载荷原字节与当前活动偏好相同（SHA674ad619…）。仅将该新测试snapshot载荷临时翻转一个字节；损坏未用作有效备份或周期状态。

| 实际操作 | 新Restore异常状态 |
|---|---|
| 恢复临时损坏的snapshot | 原生命令拒绝，活动偏好不变，新异常resolved=false |
| 真实创建另一偏好备份成功 | 没有执行另一Restore候选；该异常仍false，通知列表与失败后相同 |
| finally恢复原载荷字节，再真实重试原snapshot | 同候选恢复成功，该异常resolved=true |

新异常候选UUID为 `ba8e9fae-3831-4b3a-8a5c-f07321877b96`，于2026-10-12 23:16:30.380（UTC+8）实际解除。此前全部通知的resolved维度和resolvedAt未变，旧世代未解决异常未被新操作解除；原user pin保留。

备份载荷通过finally恢复，独立读取确认其SHA与原件一致；失败前、拒绝后和真实恢复后的活动偏好原始SHA均相同。restore返回refreshRequired=false/protectionReconciliationPending=false，新事务保护protected=false，生产store加载system/无错误。

## 候选身份核验

失败与成功后的event-scope整文件**并不相同**：实际Backup动作（含恢复保护）改变了自己的候选。独立按object=backup/action=restore/phase=execution/channel=local取分区，UUID/指纹和object_id完全相同，指纹重算为实际snapshot ID的SHA；通知ID绑定该同一UUID。

这正是动作隔离，不要求所有分区原始文件不变。没有通过编辑event-scope或通知文件取得解除结果；仅保存其原始只读快照。先前scope整文件比较结果false如实保留，不报成产品缺陷，也不删掉正负解除断言。

## 真实周期与门禁边界

本次snapshot始终保留stable/auto-check true，恢复后仍system；整个最终源周期文件前后SHA均6e51fe41…，名义截止仍2026-10-13 22:55:39/40（UTC+8）。独立持有应用/监测句柄核验PID37276/3920仍活，仅一个应用，没有手动更新查询、通道切换、clock/timer/deadline修改或产品重启。

没有执行安装器、第三方包脚本、真实数据迁移或日常替换；日常EXE摘要43e06bc3…保持。临时损坏是明确可复现fixture，不代表真实磁盘损坏/断电测试或所有失败路径。

16个原始文件全集合/哈希及Restore拒绝、其他Backup成功不解除、同候选真实解除、旧异常保持、分区UUID连续及周期保持断言冻结并重验通过。只闭合这个Restore/local/execution动作场景；其他动作、候选世代/跨通道、在途安装、全事件/UI与备份失败矩阵、完整原生/安装更新、配对性能和真实周期仍待验。TASK/脚本确认/测试环境缺口和原预算/既有性能failed保持，IMP in_progress，不降低门禁、不关闭或发布。
