# Windows 恢复事件候选隔离

2026-10-12，同一隔离根、唯一 `main@afff490` 普通候选 PID29936。证据归 `.adg/evidence/OCX-0110-20261012-WIN-RESTORE-EVENT-SCOPE/`。全部恢复通过真实原生命令，不注入领域事件、不调用手工通知解除。

准备两个真实手动备份 A=`bk_20261012090242_b54fb46b`、B=`bk_20261012090242_371096c7`，载荷均与当前 canonical 偏好字节一致，SHA256均为 `8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a`。清理策略为manual，两次创建没有删除记录。

## 候选轮换保留历史异常

临时翻转A载荷一个字节，真实恢复拒绝并生成失败通知 `…:1695188a-60bd-44ad-829d-ca2ea77499fd`。成功恢复B后，该通知仍未解决；恢复A载荷原字节后再成功恢复A，该历史异常仍未解决。

这是原契约的候选轮换隔离：同对象/动作/阶段/通道的A→B→A会生成新UUID，返回旧摘要不恢复旧UUID。首次预期“返回A就解除”缺少这一前提，已按源码及执行契约纠正；不据此报产品缺陷或改变规则。原始 `native-matching-success.json` 文件名保留该次观测过程，实际语义为轮换后返回A，不能冒充连续同候选重试。

## 连续同候选成功解除

不插入其他恢复候选，再临时破坏A载荷，生成当前候选失败 `…:5a53223f-141f-4694-81e0-138255efddff`；还原原字节并立即成功恢复A。该条通知于2026-10-12 17:05:15.520（UTC+8）变为resolved=true，前一代异常仍为false，全部原有通知的resolved维度不变。保存的失败/成功后event-scope完全相同，证明当前UUID未旋转。

两轮活动偏好前后字节均不变，临时载荷全部恢复；三个成功恢复结果均refreshRequired=false、protectionReconciliationPending=false，对应新保护备份均protected=false，原固定备份仍pinned。策略manual、stable通道、auto-check false、theme system保持。没有通过原始文件修改来解除通知。

原始文件全集合和哈希冻结后，无参数验证16个文件及上述正负断言通过。只覆盖当前restore/local/execution及候选世代场景，不抵全部动作、阶段、通道、崩溃或跨存储矩阵。原TASK/脚本二次确认及其余原生、安装、更新、同机性能和真实周期门禁继续开放，预算/既有性能failed不变，IMP in_progress，不关闭或发布。
