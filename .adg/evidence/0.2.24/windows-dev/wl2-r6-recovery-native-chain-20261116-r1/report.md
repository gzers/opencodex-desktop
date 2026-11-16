# r6真实节点恢复与canonical中断窗口

源码139de12617ebfdb2f5d78c27d93a47abd177eae6/treec3767f6979051314dc616b38be377cb62cf60a20；承接55a6d0010/848509580，TEAM16/IMP142/r6。仅Windows允许集，保留原claim/NLK/目录及两稳定文件/all8/原预算/未知拒绝。

| 运行 | profile | raw退出 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
| library01 | debug | 101 | None/None/None | 55576 |
| library02 | debug | 101 | 19/4/0 | 179225 |
| library03 | debug | 101 | 21/2/0 | 200088 |
| library04 | release | 0 | 2/0/0 | 655508 |
| product01 | debug | 0 | —/—/— | 116896 |

首library01 E0432/E0603/raw101，为Witness跨private模块导出错误，无新EXE。library02编译后19过4败：测试合成Stop阶段误用生产重取锁入口，原实际Owner要求正确拒绝；仅新增cfg(test)组件receipt原窗重取，不改生产逻辑。library03 21过2败，完整返回及返回中断在原10秒bounded-original-deadline超时，原现场/raw/EXE均保留。library04仅这两失败用例按完全相同114份Rust/断言/原截止与仓库原release参数定向2过，未延长计时/降读回/跳过场景。19既有数据/日志成功及2新中断成功复用。不W2/双SID/候选冻结/发布。

RecoveryWitness只借实际两树，由anchored内部构造，不能serde/Clone或从路径重建。每节点前后完整核原件/初始work观察/当前frozen/已退休对象/全部历史archive/准确目录children/ID/all8/bytes；同actual claim/NLK连续。新事件movement绑定准确from/to/原卷/nativeID/target parent/expectedBefore/After及完整plan row摘要。私有日志driver在intent/current已持久后才调用实际same-handle move，随后双树后验、result/current。两个driver只被cfg(test)入口调用，生产无恢复许可。

完整正例实际6current退休、3original返回同一不可变canonical链及current；原current catalog为明确合成atomic替换、3originalAbsence文件退休，原根/所有目录/两稳定对象留原位。每步rusqlite claim-only竞争均busy。全树返回后实际rawlease显式释放、全部原件/retired集合/稳定锁512范围另读回；此rawrelease尚未接canonical final-release/history/Closed，current不冒称Closed。合成阶段/无凭据artifact内容不构成实际Source/Ready/NormalExit/contentpolicy资格。

11注入：Plan不可变记录→current前、真实current部分写、plan intent/current后；退休和还原各beforeMove/afterActualMove/beforeCurrentAfterImmutableResult/realPartialCurrent。独立读取全部真实原/工作位置ID/bytes、最后不可变event与current，确认native已动/结果缺席或phase领先current的精确前缀。错误后原journal和两树poison，拒绝检查不补写。7debug中断观察与5release观察组成12精确trace（1完整+11中断），离线再核actual current/所有原event原字节。

真实Source/routes/原进程/audit+原内容policy必须在转移current frame之前完成，才可开放production driver；完整原释放/后验/history/typedClosed/后继/Status5/实际Owner-H-S-D继续未完。官方config/auth/Grok与动态节点副作用未知拒绝。Start无条件拒绝，原缺席Codex安全无缝初始化/退根blocked；普通MSI/CAP-trust/历史Profile/Public/失败broker清理未关闭。无旧raw5/空GUI/相同1314重跑、无官方或日常策略配置改变/业务网络请求。

本轮88场景均私有ZIP核验，10完成纯数据场景核准确fresh路径/完整文件hash和目录membership后清理，78失败/pending及30个D根保留。1539份canonical/快照原字节另存raw-scene-records；ZIP非NTFS元数据/身份克隆。所有历史拒绝删除不重试，overallCleanupComplete=false。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77756416 | d873bd3d2fba160b839b3f0b8097ff8f98cfc4d143aa76c1102bcbf6e1ea5d71 |
| ocxd | 5398528 | 01c45b2af8bf8b83aa3adcc7921e8b6e9542df105d57906c3b0853ce24a967f2 |
| ocx | 306176 | e688ba14e0325d43236ccc6e43bc93b97f956d54351dca2632db3f2f162f6755 |
