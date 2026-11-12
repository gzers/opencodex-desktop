# 仅Ready历史的明确当前恢复

revision: 1；2026-11-12。IMP-34 P24-04，代码main462171987。承接[缺Stopped观察](client-current-observation-r1.md)；accepted=false，非冻结候选，不修改官方2.99或日常配置。

## 准入与历史

只支持已有独立Original/scope/Ready/Live、refs1的Ready头。缺Ready/Live/原启动归属的情况尚不在本批；不能扫描认领目录中未登记文件，也不自动迁移。普通Stop的四阶段分类仍检查全部实际Snapshot未知状态并要求Ready与PreStop一致。新增CurrentRecoveryObservations只含before、ready、current，没有PreStop字段；它只判本次恢复的内容资格，不能证明丢失历史区间稳定或过去正常退出。

当前catalog/cache/audit仍须已声明路由、原绑定PID/审计HOME/时间范围、原生子集/缓存包络/审计前缀和保护约束。未知读取、外部新模型/新缓存/不明审计/权限变更拒绝。当前完整Codex其它文件、Grok、安全属性和两稳定锁原生身份仍须匹配可信历史；当前内容合格不等恢复执行许可。实际执行仍完整官方镜像准入、原PID/port缺席、当前OS停止/写者排他和同进程lease。

schema4计划内新RecoveryObservation绑定原Ready sourceHead与当前capture/tree/grok；plan.phases只有Original、Ready和当前观察三个摘要，原refs.phases永远只一项Ready。实现内部固定存储数组的空闲槽不进入当前分类、摘要或序列化，不冒PreStop帧。不可变计划及归档复验、创建fd身份、journalStages私有暂存、完整sourceHead CAS均保持。原目录即使有损坏的未登记pre-stop.json/stopped.json也不读取、替换或登记。

PreparePlan只是新明确操作：锁内当前观察与用户确认摘要再核对，不写客户端目标；成功登记RecoveryPrepared，原refs1。后来另一份合法原生内容仍需新确认。Continue/Finalize/Rollback/rearm分别按已绑定原plan和新当前原生许可执行；终态允许精确refs1/2/3并核对plan来源，不能拿普通完整历史计划冒充缺阶段来源。完整ReadbackProof仍绑定原refs/plan/所有树/安全/两稳定锁及原deadline；撤销不能产生可启动终态。预算每动作10秒，从入口计算，无新内部预算或强杀。

## 状态、软件与原型

原生首次中断暴露历史Ready被collector恒当Active。修正为与当前进程观察协调：Running才Active、Stopped转Required、观察失败或其它未确定状态Unknown。索引Ready原文和进程状态均不改；持久Required不能因进程又运行而消失，官方payload中的恢复字段不作为权限。去抖只复用既有进程事实与重新读取的独立恢复事实，不额外发CLI/网络请求。

准备计划入口接受ready+phaseCount1、stop_prepared+2或stopped+3，各自须独立原件verified、无plan/完成记录和本次确认摘要，排除矛盾能力。Ready/StopPrepared只收recovery_prepared回执，不能用restore_prepared或restored替代；后续RecoveryPrepared严格允许refs1/2，普通RestorePrepared仍3。rearm同原refs返回对应阶段，原latch保持。最后新Restored及fresh live同D/H/无PID/port/NotRequired才允许启动。

用户说明沿用“上次停止未完成核对，配置仍待恢复”，确认明确“核对服务现在已停止，再按当前状态准备恢复计划”；不承诺历史停止已完整。软件内存审计与唯一总原型新增plan-ready/slow/error，准备后仍需单独“确认继续恢复”，支持返回、进度关闭重开、错误保全，无真实API/文件操作。

## 定向证据与边界

[165份证据与manifest](../../../.adg/evidence/0.2.24/client-ready-current-recovery-r1/)含初始假设、失败和后续准确源。138相关后端函数覆盖，86前端/类型/build/格式/strictClippy通过；两个实际prefix反例扩展、refs1/2/3撤销/重准备/完整读回覆盖。首policy顺序回归Unknown误报ChangedSinceReady已修，初次restore组测试没设置/private/tmp使子进程guard失败，原记录保留，正确独立夹具同断言复验通过；Clippy范围表达式首错保留，不通过放宽断言解决。

真实官方2.99隔离Profile一次Start8739/Status227ms。测试在PreStop捕获前注入中断，生产fallback Stop1289ms真实native exit0及捕获4TCP缺席，但原动作返回失败并保留index4 Ready/refs1，无pre-stop.json/stopped.json。初次reopened wrongly Active和原整测试exit101保留；后继不重跑Start/Stop。

准确新binary对同现场只读核对Stopped+Required后，新明确Prepare4812ms/index5 RecoveryPrepared，四目标保持，另一Continue5966ms/index6 Restored。两个请求各原10秒，非自动继续。独立实际完整Codex/Grok树摘要与completion匹配、原事件前缀/原frames/scope/live和refs1保持，plan sourceHead逐字段等于原index4，三摘要和15blob均核对。最终真实NotRequired、无PID/port/仅Start。原失败不改判通过，普通Stop一般目录预算仍未闭合。

OS sandbox实际拒绝日常HOME/外网，仅loopback mock发现，无业务/模型请求。先归档逐字验证，再确认自有进程/主端口缺席，3精确新协调DB真实EXCLUSIVE重获释放后删除；程序/Profile/完整夹具和重复证据根清理。globalL inode/字节/权限/owner保持且排他重获释放，无sidecar；unitTMP空，browser/Vite退出。只验证该中断恢复路径，不代表完整进程家族、一般客户端/缺根/缺Ready、Windows、整包GUI→IPC或整版验收完成；未生成新.app/DMG或发布。
