# 首次客户端状态与专用模型清单缺席

revision: 1；2026-11-12，IMP-34 P24-04。main d98d39e8b。accepted=false，非冻结候选。承接[完整读回](client-full-readback-r1.md)和[当前恢复](client-ready-current-recovery-r1.md)，保留本版完整范围。

## 诊断状态表

| 初始状态 | 原产品事实 | 原driver | 当前结论 |
| --- | --- | --- | --- |
| Codex配置/清单已存在，Grok根不存在 | native正常exit0/4TCP缺席；Stop6644ms，index8 Restored；Grok仍Absent；独立全树与真实状态通过 | exit101，在固定“Grok三件都存在”的补充断言失败 | 产品已支持该路径，原driver失败保留；修正为按初始存在/缺席核对 |
| Codex为空目录，Grok根不存在 | native正常exit0/4TCP缺席；Stop3059ms，index6 Stopped无plan；config.toml原不存在，现为一个换行 | exit101，同补充断言遮蔽后继状态 | 产品未完成；新空配置必须按原件缺席可逆恢复，不能忽略 |
| Grok配置/Agent存在，原backup不存在 | native正常exit0/4TCP缺席；Stop7000ms，index7 RestorePrepared；三件已补偿，新backup仍在，完整读回拒绝 | exit101，固定存在backup的补充断言错误 | 产品未完成；首次backup必须纳入原缺席补偿，不能扩大残留例外 |
| Codex原配置存在，但专用catalog/cache/audit全不存在，Grok根不存在 | 修复后新Profile Start6495/Status264/Stop5744ms；完整driver exit0/index8 Restored；四个真实阶段三件全Absent，计划三Keep | exit0，实际初始状态归档断言通过 | 本批修复已取得定向原生证据；不代表其它首次状态已解决 |
| 整个.codex根不存在 | commands/client_session_lifecycle的client()先canonicalize；不存在时在捕获/索引/服务启动前返回失败 | 本批无该状态原生启停 | 源码确认缺口，尚待有归属的首次根创建，不冒称原生通过 |

前三种初态各一轮，第四种是新的不同状态与新修复假设，不是重跑已通过的原状态。原失败目录证据及逐阶段binary/source分开保存；后继成功不覆盖原失败。第一种额外只读完整树和当前状态检查通过；第二种额外完整读回检查因无登记plan失败，该失败也保留。没有把native正常停止当完整原件恢复。

## 本批修复的精确范围

client_artifact_policy的ordinary assess仍先检查四个实际阶段的Unknown并要求Ready==PreStop。当前内容assessor仍验证原独立declaredRoutes、绑定PID、audit HOME/时间范围等context，并检查所有Snapshot的有效性。只有before/ready/current三个阶段的catalog、cache、audit九份观察全部明确Absent，才返回三个AlreadyOriginal。普通四阶段路径因此四阶段都Absent；未登记历史缺口仍按先前明确当前恢复规则处理，不合成历史。

没有文件可补偿时不能要求一个不存在的私有清单列出路由；也不能从返回NoOp推断服务停止或授予原生权限。计划之外的原配置/全树/Grok、安全保护/原生写者排他/锁和完整读回照旧。任一Unknown、context缺失或后来出现的外来文件不满足此分支，按原规则拒绝。20个受影响policy函数最终通过，原unknown/保护/路由/缓存/审计反例保留，编译/格式和全目标strictClippy通过。

原生驱动新增不同初态参数，并用真实before原始字节与独立Original SHA核验Grok状态。present项须有记录的普通file/digest且归档字节一致；absent项须在Original Grok Absent或directory entries中明确不存在，不能用读取报错冒缺席。生命周期事实先落report，再做补充归档断言。普通既有三件/原预算/Stop结果/完整断言未放宽。

## 定向原生与独立读回

第四个Profile复用第一夹具中完整校验的原样官方2.99运行镜像，当前user HOME/D/H与client仍全新隔离。实际Program在第一夹具的program路径，而非第四Profile目录。新binary与源摘要记录在catalog-absent/source.json及build-final-fixtures回执；前3轮是旧binary，仅增加不同初态测试准备，不能说包含后加的policy或归档断言修复。

真实Start6495ms/Status264ms/Stop5744ms，original30/5/10秒不变，完整driver exit0。原四frame中3可选文件均Absent、original Grok也Absent；计划kinds=[keep,keep,keep]，无虚构模型投影或对三件写回。完整Codex原配置字节和原缺席保持，原两稳定锁身份/释放规则保持，index8 Restored。独立原生完整两树SHA匹配completion，Original/Scope/Live/phase/plan摘要引用逐项匹配，正确实际Program上下文collector为Stopped/NotRequired/无PID或port、仅Start。

补充只读status helper硬编码到第四Profile自身program路径，该路径不存在，因此得到NotFound!=Stopped；原失败证据与输入错误说明保留、不计通过。它不能替代原native driver对实际复用Program上下文已成功的重开状态核验，也不把错误上下文的结果修饰为服务运行状态变化。独立完整文件树读回自身exit0，不声称它证明运行来源。

## 后续实现与不变的边界

1. 将原来不存在且由本会话首次生成的空Codex配置、Grok备份作为明确可逆恢复对象，绑定原独立Original/Ready/PreStop/Stopped或先前定义的当前恢复观察与原生代次，未知编辑保留。必须能继续、撤销、重准备、完整读回，不能直接unlink或把文件当新例外忽略。
2. Grok备份补偿需要它自己的当前官方config-write-lock。官方src/grok/inject.ts的copyBackupOnce使用COPYFILE_EXCL；实测首次备份在正常Stop后仍存在。官方src/codex/config-write-lock.ts按canonical configPath+'.ocx-write.lock'协调；不以Codex N/K/L锁冒它的排他许可。
3. .codex根不存在时需要有保护的首次创建归属及原缺席记录。不能先创建一个目录、再拍快照掩盖它原先不存在；也不能收养并发出现的目录/修权限。当前尚未实现，留在本版未完。
4. 缺Ready、一般图/客户端/路径/链接/大树、全进程集合/活跃官方刷新、旧revision4、Windows独占贡献接线、NAME、复杂System代理、官方交付验证/README与固定候选/W2仍继续，不由本批样本关闭。

[本批证据与236文件manifest](../../../.adg/evidence/0.2.24/client-first-use-r1/)保存全部原失败/真实ledger/独立原件、合成残留和新通过。OS sandbox实际拒绝外网/日常HOME，仅loopback、无业务或模型请求。归档逐字核对后4个自有Profile/程序/测试根和重复临时证据清理，12精确新协调DB实际EXCLUSIVE重获释放后删除；globalL inode/字节/权限/owner保持且排他重获释放/无sidecar，unitTMP空，自有进程/主端口缺席。未改官方包/日常配置，未构建新.app/DMG或发布。
