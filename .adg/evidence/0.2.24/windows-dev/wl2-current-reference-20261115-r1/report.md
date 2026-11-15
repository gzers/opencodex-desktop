# W-L2 同对象当前引用CAS与阶段刷盘顺序

2026-11-15 Asia/Shanghai；TEAM13/IMP139/r3承接r2，fetch main dbf974097d067e75c4de68f9fc681af8ab447b09 / docs a6d9184b851521194477ddfeecd988f40d513e5b未变。本轮从8958a59b8接续，只改许可Windows O/test路径，allAccepted=false、productionExecuted=false、actualGUI=false。

## 实现和安全范围

hypotheses01先记录同原对象排他CAS与中断拒绝，library01 7项通过；review01/hypotheses02再记录原session目录、独占重开前的安全观察及create-only阶段顺序，library02最终8项通过。前轮通过状态不覆盖，原源码/命令/stdout/stderr/result均分别归档。本轮无失败测试/编译，firstError=null。

reference.rs新增CurrentReference(Reference<Full>)私有生产类型。Full固定r3 FULL_DESCRIPTOR_QUERY_ACCESS与GetSecurityInfo BACKUP完整descriptor，不以0x7替代、不改令牌/策略/ACL。真实native CAS机制测试仅cfg(test) PartialDiagnostic O/G/D投影，类型与生产Full不兼容，也不会构造OriginalSession/Closed。没有执行新fullSD查询，原1314证据复用。可编译Full适配器不等于完整安全正例或原件捕获完成。

打开current.json先由原PrivateRecord取得准确当前用户私有文件ID/原字节，保持原拒写删reader时再从实际native句柄观察安全描述符；独占读写重开必须比较同fileID/原bytes/原descriptor，转移期间变化或失败拒绝。父目录仍保留原pin。引用序列化只含schema/session/sequence/phase前后SHA/bindingSHA/manifestSHA，不放凭据，不作第二journal或权限来源。fileID/descriptor及准确bytes都在内存保留，准确bytes比较包含摘要一致性所需条件；没有使用易竞态路径替换覆盖current。

每次advance先重核本原current对象和scope/deadline，下一Tip必须来自同一原native session目录、同SID/session/binding/manifest的实际私有阶段prefix，sequence只+1、previousPhaseDigest等于原tip。旧prefix、foreignsession/目录/重复/未知字段/回退一律拒绝。current句柄share0，实测第二writer/rename/delete拒绝；pointer本身仅定位原始恢复工作，不从JSON构造原owner/nativeproof。

PendingOnly::append_pending读取原immutable记录和完整prefix，重核前缀/同目录，validate nextphase/同binding/manifest及原prevdigest；PrivateRecord create-only/flush/readback后才返回下一prefix，CurrentReference入口此后才CAS。若后半步失败，额外阶段记录保留，不回滚删除、不自动选最新记录覆盖current；旧prefix不能继续append、读取失配保持Unknown/Required。

写current从第一字节开始poison；write/set_len/sync/readback/security/id/parent/deadline任何失败都不解除。实际partialwrite只留一枚左括号后，原阶段文件逐字保留，指针重开严格拒绝；没有defaultempty/隐式catchup/重试新动作。这里是原对象排他写和failclosed中断，不声称断电原子提交；真实native原件恢复资格从未由tip或文本Closed重建。

read_pending枚举准确phase前缀时允许同目录的准确current.json名字，仍只返回PendingOnly；current自身的结构/完整性验证归独立CurrentReference，不能凭忽略文件内容取得Closed。即使文本Closed chain完整，native恢复/退出/supervisor/source未证实就无后继资格。创建CurrentReference前Full目录检查保留，首次写前拒绝原fullSD失败。

## 验证和来源

首轮7项与最终8项（4phase/binding +3currentCAS +1纯mask/error）合计8不同用例、15成功执行；实际文件锁/fileID/读写/flush/partialwrite原生机制通过，但descriptor coverage明确partialDiagnosticOnly。production Full/type wrapper只编译、无runtime正例。synthetic schema里路径/ID是数据；其与实机原件未结合，不称OriginalSession或实际capture成功。

代码3a123fe979bbefba32c00272829318e09b9b5bfa与c47953ccfe4c66a976285eacffd691e3a3b0c01b均已推codex/024-windows-native，净4文件：O mod/phase/newreference及Windows测试入口。不改共享trait/HMAC/release/Cargo/安装器/Mac或官方2.99。最终source/tree/Gitblob/原byte在final-source-manifest；早期39Rust中的phase/reference后来变化不冒最终全匹配，library02/product01每组39Rust与最终逐字一致，测试EXE各时点hash分立。

最终产品clean c47953ccf MSVC debug全部bins exit0/156748ms，107原warnings保留。测试raw含cfgtest FFI声明/unused等warning均留原件，无strictClippy或安全审计通过声明。非release预算/签名冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75458560 | d136dce96df103f5bec219b5275689bc4a85c383848b8a1980e3fd12cd807eb1 |
| ocxd.exe | 5398528 | cc6d7e66c4f0ad6906f43d30d0516043217cbb55c8cb7fdd6ec06475e48520db |
| ocx.exe | 306176 | cf0c1a39371e9e5f7bd4c0f7a514d9b6106574835f70c05eafa87517de4ce5b8 |

## 完整状态表

| 项目 | 实现 | 定向结果 | 限定 |
| --- | --- | --- | --- |
| phase-tip | derived from original private phase prefix | library01 existing checks passed | Typed binding/session/currentSID plus exact sequence/prevdigest/manifest; never native proof. |
| reference-current-object | same native ID exclusive RW CAS | library01 native mechanism positive passed | No path replace, foreign writer/delete refuses; exact expected bytes compare stronger than digest alone. |
| production-security | Reference<Full> only | Full adapter compiled only; test-only partial mechanism passed | Full access/BACKUP fixed; no live fullSD query; incompatible PartialDiagnostic cfgtest cannot create production wrapper. |
| phase-before-reference | validated create-only append/sync thenCAS | library02 passed; actual create-only phase flush/readback before advance | Extra phase on later reference failure remains and stale oldprefix cannot append again. |
| foreign-scope | native session-directory ID checked | library02 native session-directory mismatch refused; no current created | Wrong directory/session rejected before new current file creation. |
| pointer-integrity | strict duplicate/unknown-tip fields | library02 duplicate/unknown/foreign-session/stale cases passed | Expected Tip must match exact private phase prefix; never reconstruct permission. |
| write-interruption | poisoned reference, phases retained | library02 actual partialwrite test passed | Private immutable phase preserved; reference Unknown/Required; no default/catch-up or native recovery rights. |
| MSVC-product | clean final39Rust compiled | exit0/156748ms;107warnings retained | Debug build only; not release budget, signed candidate or W2. |
| native-original-capture | incomplete | not run | No original manifest/custody/workcopy/completeSD/restore/Closed wiring yet. |
| H-S-D-production | partial/incomplete | prior evidence reused | ActualGUI/originalOwner/full family/sourcebound production matrix not accepted. |
| ordinary-fullSD-MSI | environment/acceptance incomplete | no known1314 repeat | No privilege/ACL/policy changes; administrator MSI not ordinary acceptance. |
| cleanup | synthetic task records only | successguards sofar | No newaccount/Profile/Public/force; prior exact rejected trees/history untouched; cache/evidence retained. |
| constraints | maintained | noW2/dualSID/publish | No real model/business sync/dailyconfig/PATH/official changes; original budgets preserved. |

## 清理及需要用户协调的前提

只使用新任务private records/TempDir，成功测试guard在释放原句柄/目录pin后收尾；本轮无失败临时树、account/Profile/Public根、强清、官方进程/业务模型请求或日常PATH配置/策略写。缓存/EXE/不可变证据有意留存。旧native01失败树7文件670767字节及全部历史Profile/Public/schema自动审批拒绝保留，无重试或替代删除；overallCleanupComplete=false。

完整原件manifest/capture/同卷custody和工作副本、实际pre-move意图/当前引用协议接原对象、目录整树恢复/Closed、H真实GUI原Owner生产路由及S/D全proof矩阵仍未实现/验收。此处推进的CurrentReference机制并非上述整组完成。

用户已收到最小协调问题：后续真实生产验收需要不提升/不改策略即可完整读取SD/SACL的隔离普通环境，或由Mac集成owner另行明确安全覆盖产品合同。r3目前明确保留full范围，不接受管理员捷径/0x7假证明；没有新条件不重复1314。具体输入见environment-decision-request.json和mac-decision-handoff.md。尚未收到答复/新合同，时间经过不是批准，生产Start拒绝保持；不能把其余未完Windows实现推成缺共享声明。

无W2/双SID/发布。离线python verify_evidence.py只验证归档原字节/执行源/退出与边界，不运行native动作。
