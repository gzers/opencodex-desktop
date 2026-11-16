# r6同一原动作窗口租约释放与重新取得

TEAM16/IMP142/r6，源码e7c18e97ce5e7f65ed989ea2f15ac6a5d2cc62bc/tree13947bf9e07d0c46de3266c5884bc758b575a9c2。承接285be9fad/475f493e6；本轮fetch main/docs仍fac8573e5/578230717，未改正式合同/四trait/官方/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 0 | 20/0/0 | 276103 |
| library02 | 101 | 21/1/0 | 269086 |
| library03 | 101 | 0/1/0 | 154983 |
| library04 | 0 | 1/0/0 | 100826 |
| product01 | 0 | None/None/None | 231034 |

首轮20过。第二轮新严格数据反例与真实busy用例后21过1败：原断言要求Win32 raw33，但空合成SQLite的BEGIN EXCLUSIVE生成claim.sqlite-journal，整树完整children按合同提前InvalidData/rawNone拒绝。诊断03保留原断言，明确看到新sidecar；原截止还剩8609ms（后8603），current序20 Released，没有21 AcquirePrepared，不是超时或实际锁竞争已经成功。两轮raw101/原源码/EXE/失败现场保留，不把更早拒绝改成raw33通过。

只修夹具：合成rollback SQLite在native capture前初始化独立表，生产不初始化/不删或豁免sidecar。定向library04同原raw33断言通过，真正native acquire失败保留AcquirePrepared/current，foreign BEGIN EXCLUSIVE保持，由测试自行ROLLBACK；竞争消失后仍拒绝自动重试。其余21过复用library02，逐份生产Rust对最终字节相同，仅诊断及夹具setup(false保持原行为/true只新用例)改动；未重复全22套，也未重跑旧raw5/空GUI/共享route/deadline旧前提。

全部节点intent/result和LeaseReleasePrepared/LeaseReleasedVerified/LeaseAcquirePrepared/LeaseAcquiredVerified写同一不可覆盖phase链和准确current。manifest保留初始epoch/claim/deadline供整链重放，current保存当前epoch/leaseState/四域对象digest；不直接改内存绕过链。普通节点事件在Prepared/Released期间拒绝。每次Prepared只在两树完整Working或Restored、每原文件和工作代次完整时允许，不能把中途缺配置的树交官方使用。

释放：在同实际borrowed claim/NLK与原deadline核全树/原件custody，Prepared/current刷盘读回后才消费NativeLease，逐个UnlockFileEx并核范围空、原4文件ID/all8/字节/sidecar/Token/父pin，生成nonserde/nonClone非Send原生回执。回执保留原4文件及parent对象，不能从JSON或Drop造。先安装回执，再做后验与result IO；失败保留明确pending/native事实。重新取得：AcquirePrepared/current后，原回执单用、核原scope与原4对象内容/安全摘要，实际完整新租约重验两树后登记新epoch，再准节点变更。历史epoch不得复用，旧释放对象以后每次journal.check仍核。

正例在同一原10秒组件窗口，工作树完成后释放，固定Bun只导入bun:sqlite的claim-only回调此时raw0可取；原持锁时raw73，重新取全域后拒绝竞争。三原文件原件恢复及最后显式释放共36事件（原30节点+6租约事件），最终目录/两稳定文件/原件ID/all8/内容/完整children和退休工作对象核验，稳定释放范围证明仍独立。这里没有原包模块、业务/模型/网络请求或production Owner。

失败覆盖Prepared写失败（原锁不放）、native释放后/result或真实current部分写、new lease后/result或current部分写、Drop无回执、半树、外改字节/空目录、延长原deadline、claim/manifest/security/source/epoch等数据漂移；实际错误/后像归segment-traces和D journal-scene-records原字节，不能catchup/隐式重试。失效新lease在驱动中先安装，失败清理显式释放自有范围，不能冒作结果成功。

范围限制：本轮只同一原ActionWindow连续分段。后续不同Start/Status/Stop动作预算与source/phase许可不能拿此回执自动延长；仍需实际生产Source/route/阶段/Ready与PreStop/Stopped封存、允许副作用和工作文件对原包可写窗口、完整Stop/typed Closed/history/successor、broker原Owner/H-S-D。生产适配入口仍v2、公共Start无条件拒绝。原缺席Codex初始化/安全退根仍blocked，不能留目录/两锁冒Absent或释放claim只持NLK移根。旧v2/v3 pending不迁移/接管，旧ordinary MSI/CAP-trust/清理未代签。不W2/双SID、不发布。

产品MSVC custom-protocol debug bins与最终输入/测试Rust对准确源码字节核验；前端32份复用。非release预算、非冻结W2候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77688832 | ecd7bdb926915f6e961145de23974293801b1e15fc786686052b1ed2efa22ad7 |
| ocxd | 5398528 | ecd331e188254836500319a3e0c609ee02376e470f5eaf9c441f1a4efb522ac9 |
| ocx | 306176 | c38bc400a5666c26193db8719b01d082e273746be4746362d9e0d5b8afdaabe6 |

157场景私有ZIP内容核验；11处本轮已完成自有原语/纯数据场景核全部文件和目录后清理，146失败或正式phase未闭合的pending原场景、59处D日志根保留。清理选择器曾因通过测试后紧跟suite failures摘要而触发断言，未生成计划或执行删除；仅补原文failures分界，保留原stdout并重新核每处准确归属/内容/目录，原工具失败见cleanup-selector-first-error。成功同窗release也没有正式Closed，不删或冒后继。ZIP不是NTFS身份/安全克隆；旧拒绝删除/账户/Profile/Public与旧broker不重试。overallCleanupComplete=false。
