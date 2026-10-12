# Windows 最终eae失败备份不轮换

2026-10-13 01:15:49–01:15:50（UTC+8），精确 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 的唯一普通应用PID37276执行实际生产备份命令。原件归 `.adg/evidence/OCX-0110-20261013-WIN-EAE-BACKUP-POLICY-FAILURE/`；RDP断开，不作为鼠标/键盘或完整备份UI验收。

## 策略准入失败与成功触发

原库存45条，原manual/N10/D30、system/stable/auto-check true保持。实际创建3份本轮备份，D30内预览无候选；只把新3个ID的created_at设置2026-09-01夹具，原库存日期、载荷、pin及关联保护不改。

| 实际操作 | 独立结果 |
|---|---|
| 原生命令保存automatic策略 | 库存前后完全相同，不立即删除 |
| 在同一Windows备份协作锁下，将隔离策略schema_version从1暂设99 | 原automatic字节先保存；仅改变策略schema，偏好/根绑定/调度不改 |
| 原生命令create_preferences_backup | 实际拒绝unsupported cleanup policy schema or retention；库存完全不变，3份夹具仍是候选，没有新记录或轮换 |
| 读取preferences_backup_policy | 同样拒绝；损坏现存策略没有静默重置或回落manual |
| finally精确修复原automatic文件字节 | 独立SHA及原字节核对一致 |
| 实际用原偏好载荷再次创建备份 | 成功后只轮换3份新旧夹具，新成功备份保留，原45条DTO一致 |
| finally实际保存原manual策略 | 原策略文件字节及偏好/根绑定/调度保持，最终预览为空 |

所有文件夹具限定原隔离根、拒绝reparse和越界、原子写入并持有与产品相同的Windows锁；历史日期不是实际流逝D30。只删新3份的动作由实际生产自动清理完成，没有直接文件删除。策略schema夹具是在准入阶段失败，本轮不涵盖磁盘满、写入中断/断电、备份完成后的清理自身失败或全部保留安全矩阵。

## 同候选真实异常解除

新Backup/local/execution失败通知绑定候选UUID `9aa90412-c979-4783-aa1b-62a90b356ed0`。失败时resolved=false，修复后同一偏好载荷真实成功，于2026-10-13 01:15:50.373383（UTC+8）解除；原有通知的resolved及resolvedAt全部保持。

失败/成功原始event-scope的根object_id与该Backup分区UUID/指纹完全相同。原偏好载荷只读保存，并独立按精确源码的`manual-preferences + NUL + 原始payload`重算SHA，指纹匹配4bc0ff99…；不是将JSON DTO重序列化或仅将载荷摘要当作指纹。manager/panel远端分区不变，未调用手工通知解除、注入事件或替换原生桥。

该结果与此前Restore连续同候选解除、候选世代变化不解除的独立场景相补充；不扩展为全对象/动作/阶段/通道及在途安装隔离通过。

## 原件、周期和门禁保持

原库存所有文件路径/大小/SHA逐字节不变，最终只多一份真实成功备份的manifest及preferences载荷。原pin及失败事务保护仍保留。偏好、根绑定、backup-cleanup-policy和update-schedule原字节前后相同；偏好SHA674ad619…、调度6e51fe41…、根绑定8ae75370…。

应用37276/监督器24300、真实24h监测3920的实际句柄/创建时间/路径仍活，只一个桌面实例；候选EXE SHA f5e4a2ee…、日常EXE SHA43e06bc3…保持。stable/auto-check true和真实截止2026-10-13 22:55:39/40（UTC+8）不变，没有手动更新检查、切通道/根、安装脚本、重启或修改时钟/timer。

57份原件的全集合/原始字节哈希、策略拒绝/无轮换、成功仅清新3份、同分区指纹/UUID及实际解除、原件/周期保持断言冻结并重验通过。完整备份失败/UI、全事件矩阵、物理DPI、安装更新与卸载、官方019同机配对性能和真实周期仍待验；原TASK182、脚本二次许可和隔离Profile环境缺口、原预算及既有failed保持。IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。
