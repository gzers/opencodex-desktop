# beforeResume记录精确设计（实现前决定）

采用r7允许的“v3原节点事件＋类型明确附件”，不修改旧v3严格union或旧current。独立记录标识固定windows-before-resume-recovery-v1，附属同一原会话，不能从磁盘记录构造PreparedAbort、QuiescentLauncher或AbortRestored。

每条记录deny_unknown_fields/camelCase：schema、sessionId、failedActionId、manifestDigest、bindingDigest、storagePlanDigest、securityPolicyDigest、failedPrefixDigest、recoveryActionId、sequence、previousRecordDigest、startedQpc、deadlineQpc、qpcFrequency、event。UUID为非nil规范小写，摘要64位小写hex；QPC为规范非负十进制字符串，frequency>0，deadline>started且不超过原10秒。真实Instant/原parent窗口仍由持有原Owner的工厂独立校验，记录不重建deadline或权威。

事件为严格tagged union，tag=kind：AbortRecoveryIntent；LauncherQuiesced{pid,creationFiletime,forced,rawExit,jobTotal,jobActive}（必须原total1/active0/forcedtrue/raw242，不能拿数据自身签静止）；WorkRetireIntent{nodeId,workFileId,parentId}；WorkRetiredVerified{nodeId,workFileId,parentId}；OriginalReturnIntent{nodeId,originalFileId,parentId}；OriginalReturnedVerified{nodeId,originalFileId,parentId}；AbortRestoreVerified{originalNodes,coordinationObjects,normalStop,closed,publicStartOpen}（节点数/协调数正数且三个bool恒false）；AbortRecoveryPending{stage,rawError}。无Ready、StopAccepted、Stopped、Closed事件或旧格式转换。

附件链重放验证：first sequence1/previous=null且Intent先出现；同一原绑定、failedPrefix与fresh恢复action/QPC窗口恒定；下一条sequence精确+1及前条原字节摘要匹配；跨scope/action、跳序/重放/终态后追加、未quiesce的节点动作、未配对intent/result、重复节点、返回工作对象错误ID/parent一律拒绝；Pending只记录失败前缀并终结当前动作，不能由新10秒自动续；RestoreVerified须quiesced、无未完intent、全部已退休节点对应原件返回。全原树/锁/8/端点证明只能来自native工厂，不由这些序列检查替代。

负例先验证上述所有缺字段、未知字段/variant、重复字段、旧schema、invalid UUID/hash/QPC、超过10秒、wrong previous/identity/order/replay与Closed伪证。实现只输出DATA校验，不自动接入旧宿主或开放Start。原native partial anchor、原Job及恢复范围后续同一Windows写集实现并保留独立实测缺口。
