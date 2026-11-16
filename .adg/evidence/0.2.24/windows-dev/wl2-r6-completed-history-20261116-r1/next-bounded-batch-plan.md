# 后继：合同已允许的单phase有界节点事件批处理

本文件是当前CPU成本证据导出的Windows内部实现方案，不是新增正式合同或已完成事实；不申请放宽预算、安全断言或原件范围。r6原文已允许“可在单phase封装有界原子节点事件”，因此不把已有existing-root布局再次交Mac决策，也不等待范围外共享trait。

当前事实：完整37筛选同EXE仍raw101/31过6败，采样子CPU约94%单核、全机总体忙约10.8%；单个新完整case0诊断曾raw0。不能将完整失败归咎全机忙，更不能用新计时或单项成功补签。新receipt源码候选未提交，准确tree/patch/所有执行源/EXE及旧失败已冻结。

后继先fetch真实最新main/docs并复核候选差异，然后按同一Windows写集实现以下不可省略边界：

1. 新phase封套只容纳有界、非递归节点事件，仍是原单一manifest/current/不可覆盖phase链，不引入第二journal或从旧pending续写。记录serialized大小继承原1MiB上限，全节点数继承原512上限；超限在native变更前拒绝。
2. phase序列/previousEventDigest与每内部row的eventId/nodeId/rootId/原native ID/security/parent/from-to/expected-before-after/claimKey/leaseEpoch/原deadlineScope都明确绑定。采用前先单列phase sequence与row ordinal的严格规则，拒绝重复、缺row、跨node/root/session、错序/逆序、递归封套及未知字段，不能模糊原current序列含义。
3. 允许一组ordinary current退休/原件移动或工作创建的完整有序意图在一次phase/current内刷盘，再按准确原handle顺序逐项native执行；每一步仍持原claim/NLK、前后完整BOTH-tree/实际对象和原全量比较。结果记录只包含已由native返回及完整读回证实的准确前缀，不从expected数组或计数补结果。
4. 原件返回必须等待所有声明current工作节点准确退休；全部Restored/anchor释放/Closed条件保持。任何intent/current/第N个native/结果封套/current部分写失败都保留全部原/工作/退后像、native句柄和实际prefix，poison且不catch-up。旧v2及前批v3失败现场不转换、不清理以掩盖差异。
5. 首先纯进度反例与保留实际claim的两根合成组件，逐0..N前缀/封套缺项/交错/半current原始负例；再重新验证同原10秒fixture完整恢复、原件history/句柄关闭和新跨原窗只读历史的13场景。原27数据断言、原native恢复/关闭前缀及未知拒绝复用/受影响重验，失败原码保留。
6. 上述通过前，不提交未通过原预算的源码为已完成实现，不接生产权限或打开Start。源码候选可在本地继续改；证据独立及时提交推送。最终生产Source/Owner/typedStop/contentpolicy/原process与supervisor历史/完整inventory/typedClosed/Status5/后继仍按原合同接合。

无W2/双SID/发布，无官方/日常配置/机器策略修改；原缺席安全初始化/退根及历史清理缺口保持。目标继续active，下一步属于授权范围内的软件改进，不是缺环境或要求用户修改策略。
