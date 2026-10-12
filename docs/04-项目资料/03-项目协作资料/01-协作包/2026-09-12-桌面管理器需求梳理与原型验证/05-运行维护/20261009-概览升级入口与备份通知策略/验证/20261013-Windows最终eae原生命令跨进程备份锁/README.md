# Windows 最终eae原生命令跨进程备份锁

2026-10-13 01:34:03–01:34:16（UTC+8），精确 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 的唯一普通应用PID37276实际调用生产备份命令，由独立普通辅助进程持有同一`.backup-w2.lock`的Windows全范围排他锁。原件归 `.adg/evidence/OCX-0110-20261013-WIN-EAE-NATIVE-W2-LOCK/`。

## 六个实际命令与锁持有窗口

先保存原46条库存及所有备份/偏好/根绑定/策略/调度摘要，只新建一份本轮恢复源。逐场景以真实生产Tauri invoke调用，不重编译/替代产品，不启动第二桌面；每个辅助持锁进程以实际句柄、原始创建FILETIME和自身真实令牌false/type3核验。

| 实际原生命令 | 两次pending观察间隔 | 锁释放后结果 |
|---|---:|---|
| create_preferences_backup | 1187.3ms | 成功新建一份，不清库存 |
| list_preferences_backups | 1168.4ms | 成功列出实际48条 |
| set_preferences_backup_pinned | 1161.6ms | 实际固定本轮恢复源 |
| save_preferences_backup_policy | 1229.0ms | 实际保存原manual策略 |
| restore_preferences_backup | 1177.5ms | 实际light→原system恢复 |
| cleanup_preferences_backups | 1190.2ms | 实际只删本轮3份旧日期夹具 |

每场景先独立取得真实锁，再并发调用一个原生命令；两次只读WebView记录确认同一命令status=pending、finishedAtMs=null、开始时间一致。持锁期间辅助只读摘要与取得锁时的备份/偏好/根绑定/策略/调度文件完全一致，原调用进程和持锁进程实际句柄仍活；未把“仍有进程”单独当作命令已阻塞证明。

明确写入自有release.request后助手自然解锁并退出，命令才完成；六个助手和六个调用进程均以原句柄确认终态、exit0及创建/退出时间，未按进程名/PID清理或用看门狗解除产品操作。控制器45304/exec74199实际exit0，六场景原件完整。

这里观察的是六个生产命令与真实Windows协作锁的序列化，不是所有文件/事件元数据无变化：事件范围与通知属于实际终态投递，本次“持锁文件不变”范围明确只包含备份载荷/manifest、偏好/根绑定/策略/调度。没有替换原生桥或模拟响应、路由/UI输入。

## 不同值恢复与精确收尾

恢复场景在锁前实际保存light，但保持stable/auto-check true；持锁时活动偏好SHA与原system不同，解锁后真实Restore返回refreshRequired=false/protectionReconciliationPending=false。独立核验新restore-protection载荷为light/不同SHA、transaction_state=committed、related_backup_id为本轮源，活动偏好回到原system字节，无fallback偏好保存。

清理场景仅对真实创建返回的3个新ID原子设置9月1日日期夹具，原生预览只含这3份，持锁期间不删，解锁后只删这3份。日期不是实际流逝D30；finally确认无存活需恢复的新夹具，没有直接文件删除。最终撤回本轮源pin、生产store加载system/error=false，原46条DTO及原用户pin/事务保护保持。

原所有备份文件路径/大小/SHA不变，最终仅多本轮恢复源、一份实际create及一份真实restore-protection，共3条/6文件。原偏好、根绑定、策略和update-schedule原字节完全一致，偏好SHA674ad619…、调度6e51fe41…、根绑定8ae75370…。

## 生命周期与适用边界

唯一应用37276/监督器24300、真实24h监测3920的实际句柄/创建时间/路径保持，候选EXE SHA f5e4a2ee…、日常EXE SHA43e06bc3…保持。真实截止仍2026-10-13 22:55:39/40（UTC+8），没有手动更新查询、切根/通道、安装脚本、时钟/timer修改或产品/周期重启。

131份原件的全集合/字节摘要、六次真实pending及文件保持、普通真实令牌/进程句柄终态、不同值恢复/保护和精确原件恢复断言冻结并重验通过。RDP断开，没有鼠标/键盘或原生像素验收；本轮不覆盖管理员协作锁、非协作写者、锁持有者异常终止恢复、磁盘失败、完整备份/UI或全事件/通道/安装在途矩阵。

官方019同机配对性能、物理DPI及完整原生/安装更新/卸载、最终源30分钟退避与真实stable/面板24h及beta6h周期仍待验。原TASK182、脚本二次授权和隔离Profile环境缺口，原预算及既有failed保持；IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。
