# 首次缺根捕获与受证明约束的生命周期接线

revision: 1；2026-11-13；IMP-34 P24-04；main a325f289a、59ec750f6。accepted=false，非固定候选。承接[正常Stop根权限](client-root-stop-authority-r1.md)。生产首次Start/Ready/PreStop/Stopped和根恢复调用链已接，但真实Stop10预算仍失败，RootAbsent终态未完成；next Start、RootAbsent状态UI及根中断恢复仍未接受，不放行整版或Windows W2。

## 创建、捕获和阶段权限

Active.capture对实际用户profile/.codex读取symlink_metadata：仅NotFound进入原root_creation::reserve/publish与原created fd/排他custody，Index原子登记初始Preparing，再用专用first-root CaptureReserved预约。其它读取错误不当Absent；原已有路径仍走canonical既有根流程。原件捕获通过同一reservation取得独立Absent来源，schema5 Original保存初始执行树而不替代原缺席事实，scope和Captured3通过专用登记，未做旧路径迁移。

RootPhaseProof字段私有，无serde，不能用帧SHA/bool替代本次活体证明。first-root Ready执行原bind_ready的签名health/ready/health与进程代次/launcher验证；PreStop再次fresh_ready并实际seal；Stopped绑定原CompletedLifecycleStop、PairExitProof、原记录/四帧与全部已捕获TCP/manager/记录缺席。Index.record_first_root_phase在提交前后require当前证明、准确data/client/profile/source Head/refs，CAS与前驱/完整不可变引用保持。普通advance依旧拒绝root-aware会话，只有明确typed入口可进入新增三阶段。

root_binding事实重开现可承接Captured/Ready/StopPrepared/Stopped/RestorePrepared，依validate、独立RootBinding及原摘要/完整帧链核验，不从当前目录猜原件。StartupSession::open_indexed从实际Index的origin重开；普通分支继续effective_client canonical路径，避免有效符号链接普通客户端退化。新增canonical重开回归覆盖该差异。

## 正常Stop编排

原服务与launcher都正常退出后，首根路径使用显式relocation native协调。先typed Stopped与独立refs，再原artifact恢复计划/恢复及官方writer释放；完整root-stop admission在创建新Guard之前，随后新Guard/根计划/OFD、实际Index登记和不可序列化registration、当前Stop factory、原根移动/官方正常release/精确读回/独立RootAbsent关闭。普通既有根原流程保持。RootAbsent成功并非只看盘上终态：必须整个协调释放、最终程序来源和期限检查成功；诊断必须stop_ok及全部断言后才写passed，始终accepted:false。

目前真实生产调用已走到独立RootRetirementPrepared8，但在root-retire权限核验时耗尽预算，未发生根终态关闭；Stop返回恢复待处理/timeout、canStart=false。没有把服务/launcher退出0或root计划存在当完整Stop成功。首次Ready恢复预览仍Pending、终态UI/nextStart仍未接，记录为后继范围。

## 同动作等价开销优化

Index.checked_head以一次observedHead执行完整root_binding::resolve，末尾重读Head须逐字段相同，再返回该已核验事实；每独立调用仍重新核验，无跨动作cache。删除的是同调用内resolve已执行的初始event/verify_capture重复读取；action锁、数据库VFS与pins、CAS及提交读回保持。phase包装器执行原bind/seal，typed proof在Index提交前后require，不再先在包装器重复require一次。

目录pin的recheck不再反复open同一链上目录；每个路径分量fstatat NOFOLLOW必须与原retained fd身份、类型、birth/dev/inode、uid/gid/mode07777/flags相符，完整Protection(ACL/xattrs)通过同inode保留句柄读取两轮，末轮路径也重核，持续祖先替换或扫描中属性变化拒绝。没有依ctime推理或忽略ACL/xattrs。测试允许同目录正常增文件，但mode、新inode/symlink/祖先替换、首轮属性读后添加xattr都拒绝且现场保留。

首次优化测试曾误用0777而与原特殊权限位不符，0过2失败保留，改为完整07777后通过。静态审查指出末轮必须再次读完整ACL/xattrs，已补双轮及确定性xattr负控，不能把中间仅路径复核版本当等价接受。原三次Protection减为两次，保留语义边界。

## 真实诊断结果，四轮均失败保留

[132文件证据与manifest](../../../.adg/evidence/0.2.24/client-first-root-lifecycle-r1/)保留每轮启动/停止原输出、固定阶段/时间、源码binary、独立SQLite事件/Original/scope/live/phase/plan字节及清理。新隔离Profile、原官方2.99和原30/5/10预算，沙箱拒绝外网及日常HOME；仅loopback合成发现，无真实模型/业务同步请求。

| 轮次 | 首次Start | 服务/launcher与监听 | 独立停留状态 | Stop结果 |
| --- | --- | --- | --- | --- |
| native | <30秒，通过 | 两者exit0，捕获4TCP均缺席 | RestorePrepared7 | 10278ms，恢复未完成 |
| diagnostic | <30秒，通过 | 同上 | RestorePrepared7 | 10273ms；文件恢复及正常writer release后post-apply-stopped timeout |
| coalesced | <30秒，通过 | 同上 | RootRetirementPrepared8 | 11523ms；减少同调用Index重复后到根许可入口仍timeout |
| descriptor | <30秒，通过 | 同上 | RootRetirementPrepared8 | 10338ms（manager动作10080ms），root-retire拒绝，未移动/未完成 |

所有失败不重写为成功。每轮原始缺根schema5与独立origin一致，Preparing→CaptureReserved→Captured→Ready→StopPrepared→Stopped→RestorePrepared事件齐全；后两轮多出准确根计划登记8，completion仍null，原root还在，无根Retire readback。verify_native从无损压缩原字节重建只读SQLite，逐帧核验前驱/SHA、scope/live、原Absent来源及计划source，副本不作原生权限证明。首次Start通过不代替Stop、nextStart或中断恢复。

## 检查、复核和清理

最终定向63去重函数通过（26快照、36索引、1canonical）；中间115过/1ignored的root_较广筛选仅属于最初接线源码，不冒最终优化来源。最终只改测试nested-if的Clippy等价写法后，xattr exact test再跑并严格Clippy通过，生产源码与descriptor失败原包轮完全一致；source-final记录该唯一测试差额与执行binary，非同binary冒用。fmt/diff-check通过。文件可见性/门槛未降低，无超时延长。

两位只读审查分别找出并确认canonical/stop_ok、Index单调用复用与ACL/xattr末轮修正，不改文件、不执行native、不代签。所有子审查和test/native句柄已终止，四轮无残留进程；产品证明4TCP缺席，独立清理仅另查已导出的主端口，未冒全部监听集合复验。每轮精确新N/K/history共12个DB均exclusive重获后删除，预存global L原身份/权限/字节保持并重获释放，不删除预存对象。每轮完整自有Codex/Grok/manager-state/合成HOME原字节先归档，逐字核验后清四个Profile、复用runtime/program和重复临时证据根；unitTMP空，工程target留存。证据约1.08MB，无重复运行包归档。无日常配置、官方源码、真实业务、新.app/DMG/Windows候选或发布。

## 下一步，不能视为已通过

优先对root-retire当前Stop authority/Index/frame/native检查做精确同动作分段测量，确定剩余重复I/O或查询成本，再统一修复独立项，原10秒/严格读回保持。未有新假设不开新native轮；已通过Start不单独重复验收。完成Stop后再接RootAbsent的用户状态和nextStart安全滚动/中断恢复；现unfinished/未知继续拒绝。Windows W1独占贡献、W2固定候选、NAME/一般客户端图路径/完整进程集合与刷新/System代理/官方发布验证/README等原完整范围不缩减。
