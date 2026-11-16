# 单phase批节点的严格序列规则

此为Windows内部r6实现记录，未建立第二事实源或新正式合同。旧v2/v3失败现场原样保留，不转换/接续；新批事件只由本代保留原对象的journal产生。

- 每个BatchEvent只占原不可覆盖phase链的一个sequence及一个event-xxxxxx.json，current.sequence含义不改。所有内部row带0起ordinal，严格连续，row原Event/RecoveryEvent.sequence等于本phase.sequence，previousEventDigest等于本phase.previousEventDigest；整向量序列化及batch plan digest绑定顺序，不能截断、交换或递归嵌套。
- complete intent计划必须正好覆盖本操作在StoragePlan或准确Stopped RecoveryPlan中的所有节点，按.codex后.grok及原relative序排列；每row保留原eventId/nodeId/rootId/operation/from-to/nativeID/all8/parent/expectedBefore-After/claimKey/epoch/原deadlineScope。所有row eventId与phase eventId全局不重复。
- intent先刷phase/current，然后实际原handle逐个执行；结果phase可声明严格连续的实际verified prefix增量。每一row completion仍经原单节点validator验证对应完整intent，不从prefix长度生成行。任何pending batch都禁止其它phase类别和旧单节点/lease/action/work/snapshot绕过。
- 一次result phase验证全部row后才原子更新纯数据Progress；最后一row结果完成前不得释放lease/进入Ready/Restored/Closed。原current只保存batchId/planDigest/kind/count/prefix摘要，仍4KiB；记录仍1MiB/512节点上限。
- anchors只允许原SnapshotVerified或ReleaseVerified观察批，不进入移动集合。原目录/两稳定文件在原处；缺席/未知/原件全部比较不变。actual0..N中断后可保留canonical intent pending及准确native prefix，不能自动补完成事件。
