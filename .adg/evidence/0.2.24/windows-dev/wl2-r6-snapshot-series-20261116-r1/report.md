# r6多次原生观察与原共享内容判断

TEAM16/IMP142/r6，源码512ff5de258cd209efac6b6018c26537114855c5/treee4fc5aed492a7db3334966174f28ce4cbd44ae63，承接eee0ad1a4/937fe6c94。四trait/原截止/8投影/ExcludedUnknown/原共享client_artifact_policy不变；官方与日常配置未改。

| 运行 | profile | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
| library01 | debug | 101 | 2/2/0 | 243345 |
| library02 | debug | 0 | 4/0/0 | 201541 |
| library03 | debug | 101 | 3/3/0 | 251160 |
| library04 | release | 0 | 6/0/0 | 1461282 |
| library05 | release | 101 | None/None/None | 165312 |
| library06 | release | 0 | 41/0/0 | 113187 |
| product01 | debug | 0 | None/None/None | 250388 |

首轮library01两过两败/raw101，四个Resume故障窗口及未知引用正确拒绝；完整多阶段测试在首次观察/resume后的再次release失败。旧release边界仍要求当前work字节等于最初work副本，遇到已登记变化即拒绝。保留初始work/original全部原断言，新增临时完整native release view：先按最近实际已链接archive精确比较ID/8投影/内容/children，再Prepared、原生unlock及后验/result；只在release结果canonical写完后撤下临时拒写句柄。失败view/pending保留，不把当前内容写回基线。原错误/EXE/日志/场景保留，未延长原10秒组件窗或重写原断言。

library02 debug四过；扩展library03 debug三过三败，首次Resume实际TimedOut，原10秒与断言保留，不重写通过。library04使用仓库原release参数，同原源码断言/10秒，无Cargo profile参数变化，六项全部通过；trace保留各阶段原剩余时间。该构建23分41秒；后续Cargo回归又触发重复优化编译，核全部Rust、rustc、编译flags和已通过EXE hash完全一致，仅驱动脚本增加过滤mode不同，确认compiler是本任务Cargo直接子进程后只停止该重复compiler。library05取消raw101/原stderr/source保留，无执行测试或新EXE。library06直接运行library04准确原EXE和library05原回归filters，执行前后hash相同、原Rust输入与编译flags核验，41过0败。这里没有放宽deadline/assertion/filter或更换测试实现，不是把取消当通过；具体证据归release-executable-reuse-proof/redundant-compiler-cancellation/run-command。release组件仍不代表actualOwner生产预算/冻结候选。共享policy直接调用原文件，source equality确认原算法未改；首个内联工具校验引号SyntaxError只影响证据helper，已保存脚本重核，原工具错误独立保留。

Resume必须当前Observed已由实际双根SnapshotWitness验证并链接该sealed observation sequence，先WorkResumeIntent/current，再ReOpenFile准确current工作对象的metadata observer，保留旧代次和不可变archive到history，再完成结果。非链接/旧序号/错误引用拒绝；同一观察重复链接仍拒绝，多次Observation每次独立序号与目录，最新链接不抹旧事件。current保留全部已链frame摘要和当前sealed序号；旧Ready等阶段按自身原action scope保存，不以新Stop scope改签旧快照。初始work_frame与原件plan永不替换。原子/普通写后每次完整观察仍核所有非产物与unknown children。

实际合成native序列取得三个独立完整frame和同卷archive，真实release/reacquire后进行下一段写入。旧archive/part/source metadata与old work generation持续保管、重核。内容调用用两根统一native前后barrier，逐次核全部original/custody/历史work对象/全部archive和当前完整frame；原纯policy调用仅在两个完整barrier之间重建已验证不可变字节，不再逐阶段重复扫描整集合，不缓存或跨调用复用许可。按原件来源ID/8投影/内容重建ArtifactFrame，source generation和archive-copy ID分开，调用原client_artifact_policy::assess，不复制解析/放宽规则。正例验证AlreadyOriginal和保留已捕获合成metadata的RestoreOriginal内容分支（后者没有实际恢复写）；反例在中间真实修改模型内容，原policy ChangedSinceReady拒绝；另实际Grok新增未知空目录在完整barrier拒绝，即使Codex内容本可通过也不给完整结论。原custody/frame保持。路由/PID/audit是明确合成输入，不是实际生产进程/配置资格。内容结论仍是DATA，不直接退休工作或给恢复/Ready/Closed许可。

新production stopped_content_observations selector仅在实际原Source/Owner/NativeAction、typed NormalExit/Absent/StopAccepted/原record/listener后验以及canonical Stopped下，取唯一Ready/PreStop/Stopped link序号；前后复核同claim/source，工厂只编译未真实调用。当前native测试全部Observation用途、生命周期Preparing，不冒充官方阶段。不造fake生产Owner/typed证明来填正例。

故障覆盖Resume意图后/原生句柄已交接后/结果event-current或真实部分current写，均保留准确Resuming与native历史前缀、poison拒绝追赶；未知历史archive空目录拒绝，不能靠只看最新活树忽略旧档案。受影响work/snapshot/lease/v3数据回归按表；不重复旧raw5/空GUI/首1314或无变化共享source/status算法。

未完保持：完整v3生产dispatcher/actualSource、正式phase全快照组合、config/auth/Grok源声明非产物变更、动态产物/新目录准确退休还原、全部normal exit/监督/端点/record/source及typed Closed/history/后继/实际原Owner H-S-D。公共Start无条件拒绝。原缺席Codex安全初始化/退根blocked单列，不能留下目录/两锁冒Absent。普通MSI/CAP-trust/历史清理不代签。不W2/双SID/模型业务请求/发布。

产品MSVCcustom-protocol debug bins成功，最终输入/testRust与准确工作/Git源码字节核验，前端32份复用；不签release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77739520 | 33625217d931a0af7ab9245bcdcfb6563a781d93363305d56581be5407106346 |
| ocxd | 5398528 | 998f3fdbc1cc052e8a5c986682a0aa6b371cee389b71bfa6c9c756f8ace28f55 |
| ocx | 306176 | c9f2de2330edb8f337401d6c8e60e56df78504ebadb2e9697019fa54f3520dea |

199场景私有内容ZIP核验；canonical多观察/Resume事件和same-volume source archive块/metadata原字节另归canonical-series-events/raw-scene-records，trace保留实际观察序号和结论。10处完成纯数据/native历史场景核完整文件与目录后清理；189失败或正式Closed缺席pending原场景、94 D日志根保留。ZIP非NTFS身份/安全克隆，旧自动删除拒绝/账户/Profile/Public/旧broker均不重试，overallCleanup=false。
