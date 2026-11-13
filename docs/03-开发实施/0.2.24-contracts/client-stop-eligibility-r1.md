# 停止回执与后继启动资格同源 r1

revision: 1；2026-11-13；IMP-34 P24-04；main33d0a9fca1bb81785fef1c6ef99b12e432f0cc72；accepted=false，非固定候选。承接 [只读资格审查](../../../.adg/evidence/0.2.24/client-exitfix-native-r1/successor-action-review.json) 与 [Plan门槛保留](client-lease-directories-r1.md)。本批不运行官方服务，不宣称正常Stop预算通过。

## 产品行为

此前成功Stop回执只按通用Stopped状态设置canStart=true；独立status却会核验当前RootAbsent、完整独立历史容量及namespace，可能要求处理原会话。现在正常root Stop接收本次关闭认证产生的私有ClosedRootStartEligibility，使用同一successor_eligible_checked算法。

| 完整关闭认证结果 | 停止动作 | 后继门槛 |
|---|---|---|
| 已核验，允许下一完整周期 | Stopped成功、failure无 | canStart=true，recoveryRequired=false |
| 已核验，历史容量不足 | 仍为Stopped成功、failure无 | canStart=false，recoveryRequired=true，与status一致 |
| 候选计算或后续来源／归档／读回／证明出错、过期 | 真实已停止观察仍保持；完整动作不确认成功 | result=null／failure有，全部动作门槛关闭 |

没有用错误路径with_stopped_client_recovery擦除真实成功result，容量不足也不发停止失败通知。普通既有客户端目录的成功Stop仍走原规则；Restart仍用原close和后继Start，不启用这项投影。公共JSON字段／数据库格式／前端未改。

## 同次完整认证边界

仅本次close的postcommit完整观察把包含新RootAbsent的canonical history及lineage绑定的indexed namespace交给私有候选计算；采用原容量算法的4096events／4MiB／单event65536／generation上限与实际序列化完整周期预留，不另写简化阈值。随后原fullhistory、逐帧provenance／archive、current root／Plan／Retire-readback／scope、namespace与binding/history回读、storage／identity／Head尾检全部继续。

只有上述完整write返回成功且原AuthorizedReadback::require再次成功，才构造字段私有、无Clone／serde／公开unchecked构造器的资格值。投影回调只返回()，不把Originals、Plan或mutation permit输出；候选为false同样不能跳过完整收尾。原source/candidate/postcommit三事务边界与过期pending demotion保持。

Active直到外层协调核验、释放、完整程序来源重读与原deadline成功后才提取该值，错误／超时不泄露资格。真正root Stop请求缺少投影明确失败，不能落回generic true。commands独立保留StoppedObservation供错误路径使用，仅成功路径消费资格。Stop10及Restart30的原Stop子deadline不重置，不在动作返回后额外调用完整status/fullscan。后继Start仍重新认证当前现场，资格不是下一次Start的许可或永久禁启标记。

## 验证

六项新测试覆盖：首代及完整四帧历史第二代动作/status一致；4095事件容量不足仍完整关闭并保留Stopped成功、status要求处理且后继Start拒绝；候选后8类frame／Plan／receipt／namespace／logical root／binding／event／明确投影错误均返回错误且无资格、保留已提交事实；初始proof过期拒绝；候选后到期保留原pending demotion；先前资格允许后出现新用户根，实际Start仍拒绝且保留用户文件。fixture时限只缩短原预算，不加时。

无投影／有投影首代均12帧、12archive，完整第二代均72帧、72archive，历史begin／finish各5、内容双遍10相同，source/candidate/postcommit各1。没有新增历史fullscan；候选自身的有界容量／目录观察仍有实际成本，不声称零开销或性能修复。独立实际diff只读审查无阻断，不代执行测试。

6新项＋root retirement index8＋root start8＋retirement transaction6＋命令4＋DTO11＋基础index28，共71项不同受影响测试通过；命令1项／基础index2项原有忽略保持。最终all-targets Clippy -D warnings、fmt及diff-check通过。初轮2项测试在持有action独占锁时发起独立status，被正确busy拒绝；修为先drop Index再status后通过，产品锁规则未改。扩展第二代计量时测试辅助factory可见性编译错误也保留，仅cfgtest父模块可见修正，未开生产绕过入口。

证据 [.adg/evidence/0.2.24/client-stop-eligibility-r1](../../../.adg/evidence/0.2.24/client-stop-eligibility-r1/manifest.json) 含开跑前假设、所有压缩原日志、错误／修正记录、首／次代计数、8源与测试二进制摘要、审阅范围及重验脚本。unit TMP为空，原字节压缩解压逐字核对后删除本批临时根，工程target保持。无日常配置／Shell／安装app／官方2.99或真实WebDAV／模型请求变更。

本批只关闭资格回执的实现差异，仍缺固定候选原包验证；此前第二Stop11428ms／pending19失败未解决，不因新单元测试通过而重复运行同失败源码或更改预算。根中断恢复／缺Ready／一般客户端图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选等原完整范围继续，目标active，未发布。
