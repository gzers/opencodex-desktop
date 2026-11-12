# 缺少历史Stopped时的独立当前恢复观察

revision: 1；2026-11-12。归IMP-34 P24-04。后端mainf318a9e47、界面main027c408cf。承接[缺失计划](client-stopped-missing-plan-r1.md)与[私有暂存](client-journal-staging-r1.md)。accepted=false、非冻结候选；仅闭合Original/Ready/PreStop/Live独立引用完整而Stopped缺失的一类中断，不代表缺Ready/PreStop等所有阶段恢复完成。

## 事实与许可区分

StopPrepared独立头保留原Original/scope/Ready/PreStop与Live摘要；当前完整官方镜像、受保护运行记录、原service/launcher PID与端口缺席、完整当前OS归属及原生claim/N/L/K写者排他仍必须重新证明。不能从磁盘Live/原日志重建过去NormalExitProof，也不把当前文件封成历史Stopped。

本版新schema4恢复plan包含RecoveryObservation，明确schemaVersion、原sourceHead、现在observedAt和当前capture/tree/grok。原sourceHead仍StopPrepared、同attempt和Original、只有Ready/PreStop两份阶段；新观察独立摘要参与plan.phases第四项，plan文件本身独立SHA再由索引登记。它不是Phase::Stopped frame，不调用Originals.seal(Stopped)，不往原refs.phases加不存在的阶段，不扫描/认领未登记stopped.json。若目录已有未登记历史文件，即使合法样子也不作来源；反例用损坏的unindexed stopped.json证实新方案不读取或改写它。

内容仍复用原client_artifact_policy全组三文件分类：before=Original，ready=Ready，pre_stop=原PreStop，当前observation独立作为本次恢复后像。必须满足原declaredRoutes/原绑定PID集合/审计HOME和时间范围、原Ready与PreStop一致、catalog/cache/audit内容和保护约束；未知或新客户端变化不从当前输出猜权限。其它Codex完整原件余项、Grok原件和两稳定锁仍必须匹配原件/PreStop原生身份；任何客户端stage/链接/不可读/外部内容变化拒绝保留。新观察所有引用内容先create-only归档并按原摘要复验，计划必须保持journalStages布局，不写目标。

确认绑定的当前三文件capture与补偿decision在只读review产生；实际锁内准备完后，再用计划内精确observation重新计算同sourceHead/decision/原件身份确认，必须等于用户确认摘要。即使后来内容仍是合法的另一个native subset，也不能沿用旧确认。新增反例先生成原确认，再变另一份符合分类的原生内容，准备plan的确认必不同；原件/目录安全/两锁身份和完整余项仍核对，未知保持失败。

## 持久状态和执行

record_restore_plan新增StopPrepared分支：必须plan内sourceHead逐字段等于当前原独立头、refs原文匹配，原frames及observation归档可读，才CAS追加RecoveryPrepared事件。原refs仍两阶段，只有新plan digest登记。普通Stopped4frame路径继续RestorePrepared，不能互换回执以伪造历史完成。

原Continue/Finalize/Rollback可处理已核验RecoveryPrepared，但权限仍是当前新动作的同进程锁/真实OS停止，不依据plan布尔授权；新plan.load复核原三frame链、sourceHead/refs和observation完整安全上下文/树/归档、分类/plan阶段哈希。完整恢复读回用plan里明确的current tree提供本次两稳定锁身份，不访问缺失历史Stopped。ReadbackProof绑定原refs2/plan/root/deadline；Restored/RestoreRolledBack终态允许精确refs2但独立登记验证必须同时核对schema4计划的原source引用。普通plan仍要求refs3，不能把两阶段嵌入普通plan蒙混通过。

回退与重新准备保持原不可变Rollback及新逻辑plan链；refs2重新准备后阶段仍RecoveryPrepared。超时降级回原expected.state而不是制造普通RestorePrepared/Stopped事实。最终Restored只是新明确恢复动作完整文件读回成功，不替代历史原Stop成功，原失败事件和结果保持。每次原10秒/完整性门不变，无强杀、自动恢复或预算借用。

## 产品与原型

preparePlan现支持stop_prepared且phaseCount2，预览仍原件verified/plan未记录，确认摘要为这次当前观察。返回只允许recovery_prepared+同attempt/revision+1，不能接普通restore_prepared来冒历史齐备；新状态preview必须phaseCount2，普通restore_prepared必须3，Finalize/Continue/Rollback能力各检查该边界。准备后的fresh live仍Required、同D/H/无PID和port/三动作false及精确preview/revision，才展示另行Continue；不清latch或自动恢复，最后新Restored+fresh live才允许启动。rearm按refs2返回recovery_prepared，同样不冒完整历史。

界面说明“上次停止未完成核对，配置仍待恢复”；确认“核对服务现在已停止，再按当前状态准备恢复计划。原件和上次操作结果保留”，内部schema/历史证明不塞进用户操作文案。唯一总原型同步plan-current/slow/error及既有plan场景，固定内存/原按钮确认返回/进度关闭重开，不读真实文件/API，不改其它原型或总index。

## 验证

92个去重后端函数：restore32（初31过再加确认内容变化反例1）、session26、index27、completion7。新例证明原三frame依旧按独立摘要、缺Stopped不读/补/认领、sourceHead错revision/refs多添Stopped拒绝；部分恢复、Rollback完整读回/独立回退头/rearm再Complete闭合均同时按refs2和refs3跑，旧refs不可rebase，坏回执/归档仍拒绝。后加sourceHead完整validate收紧后原观察用例定向过、最终格式/全目标strictClippy通过。没有用syntheticPID当真实正常退出证据。

84前端/类型/生产build通过：当前观察独立回执/阶段数、普通结果不可替代、准备后仍Required、不自动Continue、最后明确恢复、错误/关闭重挂/只读重试保持。两引擎各10软件/唯一原型场景通过，明暗1180×760及900×600、确认返回/慢重开/失败、旧4frame计划保持，pageerror为空；软件暗色与真正原型确认截图实际查看。生产排除DEV handle/固定UUID/mock文本，含正确新阶段/专用说明。

一个新隔离Profile使用未改官方2.99：Start8668ms/Status236ms；原普通Stop在服务+launcher实际normal0、4TCP缺席后，但Stopped frame/索引发布前按预设中断，Stop2063ms，原完整测试exit101/failed/Required/index5 StopPrepared，refs两份、不存在stopped.json。原返回停止事实来自当次真实Stop内核证据，不从后继plan重建。

另进程同现场review/prepare：错revision/digest、外部原config变更拒绝且索引不变/输入保留；测试自行恢复合成夹具后取新确认。当前观察prepare4099ms，只暂存和登记plan schema4/recoveryObservation/sourceHead，目标字节不变，index5→6 RecoveryPrepared；另一明确Continue6600ms，新当前权限下恢复/完整树安全/两锁/回执/index闭合Restored7。原每次10秒不变，不借新动作改判旧Stop成功，无服务重启或真实模型请求。

独立只读native完整Codex/Grok当前树SHA匹配新completion，plan SHA和原scope/live/frame/receipt引用匹配index7/7events；plan.observation.sourceHead逐字段等于保留index5，event6精确RecoveryPrepared/原refs2。原3frame+scope/live原文保持，stopped.json始终缺席，未生成或登记假的历史。7客户端路径原字节/缺席匹配，15内容blob逐摘要核对，新真实状态NotRequired/无PID/port/仅Start。

原生执行source/build-native准确binary在native和current-recovery回执分别记录；最后仅加sourceHead字段完整validate收紧，定向反例/strictClippy另证，不声称早期binary包含后加防御。未构建新.app/DMG或固定候选GUI→IPC原生整链，本轮局部passed不等整版accepted。

证据[本批目录](../../../.adg/evidence/0.2.24/client-current-observation-r1/)及manifest含原Stop失败index5/原件、当前plan/sourceHead、最终index7/完整树/独立摘要/真实状态/截图。OS sandbox拒绝外网及日常HOME，仅loopback，无真实模型或业务请求。自有进程/原端点缺席，3精确新协调DB实际EXCLUSIVE重获释放后与程序/Profile/全部原件stagearchive/test夹具清理；globalL inode/字节/权限/owner保持、排他重获释放、无sidecar。unitTMP空，browser/Vite退出，归档逐字核对后清重复临时根，target/dist保持，无日常/官方/系统配置改动或发布。

下一步缺少已独立登记Ready或PreStop的中断；当前三frame先决不能用本批窄证据替代。首次缺根/一般客户端及路由图/完整进程集合/旧revision4首错、Windows贡献集成/NAME/复杂System代理/官方发布验证/README和固定候选/W2继续，Windows远端b2e89665d/0eef59a01无新回交，目标active。
