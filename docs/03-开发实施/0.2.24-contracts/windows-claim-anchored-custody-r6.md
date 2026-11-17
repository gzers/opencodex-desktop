# Windows claim连续排他与原目录留位保管 r6

2026-11-17后继：[r7](windows-preresume-recovery-performance-r7.md)明确首次Resume前失败的受限原件恢复与共享扫描责任/临时写集；本布局、r4安全及原预算不变，新的恢复回执不当正常Stop/Closed。旧失败事务与原缺席退根未完仍保留。

revision: 6；2026-11-16 Asia/Shanghai；TEAM16/IMP142，承接r4安全覆盖及[r5共享接口](windows-shared-routes-status-r5.md)。Windows实现输入3e19075846dfea81ba8392bb60a3d3031afbf116；本次共享常量提交 **main fac8573e55441fe514919c63c7f1d65181ff3577**。四trait签名、普通Token、固定8投影/审计ExcludedUnknown及30/5/10/Restart30含Stop子10不变。本轮是具体Windows保管策略决定，不授权发布或把原失败改通过。

## 已接收事实

代码由0549推进至3e1907584，文档由676cec18a至13e5c3223，4轮新证据及handoff净2659文件；2654payload/31,356,233字节逐项核验。四组final product输入对各自提交核对（114/116/118/118文件，行尾差异明列），最新72份Rust实源以末轮记录为准。离线复核及完整失败/未完归.adg/evidence/0.2.24/windows-wl2-claim-receive-r6/；不是新的Windows执行或完整代码安全审计。代码继续Windows独占分支，尚未整批集成main。

| 项目 | 复核接收 | 尚未成立 |
| --- | --- | --- |
| Route/Sample | r5已合，原Windows receiver保留；RoutePin真实H/config ID/8投影/字节；同次私有样本回执绑定原Slot/session/scope/deadline | native Closed正例/并发切换/真实原包生产采样未执行，不以缺Owner测试代签 |
| NativeLease | 原claim/N/L/K四文件、范围/原Token及Bun与rusqlite真实互斥组件，持有拒绝/释放可用；namespace不受fake HOME影响 | 官方初始化/ownerreceipt/真实恢复和root交接不成立；组件不构成mutation许可证 |
| ArtifactContent | 原对象/Ready/PreStop/current封存视图接原纯policy，非产物树节点严格核不变 | 官方允许副作用＋NativeLease＋canonical恢复闭合仍未组合；content eligible不授恢复 |
| root rename | baseline与仅metadata根handle可移动；打开claim-file（即使shareDelete）/file+metadata/实际lease均raw5失败 | library07诊断raw0只表示记录各格完成，不把后三格失败算成功；原5/6等失败保留，continuous-open-claim用例仍blocked/ignored |
| Claim-only writer | 固定官方tgz的main-account.ts:463→:394及native-main-claim.ts:167，凭据提交仅claim排他，helper不自动取得N/L/K | 不执行真实凭据写入/网络；源码反例足以否定“N/L/K必替代claim”的假设，不制造新无锁窗口 |
| 最新产物 | clean3e1907584 custom-protocol MSVC debug bins0/77327ms，library08为10pass/2ignored，原字节及SHA保留 | 非release预算/候选，official production/StartOpen仍false |
| 清理 | 最新30scene/155文件1,217,258字节私有ZIP核验原scene留，前轮39scene及6scene分别保留 | ZIP不是NTFS身份安全克隆，overallCleanupComplete=false；旧broker40588/Public8拒读/普通MSI/CAP-trust/所有旧删除拒绝保持 |

## 决定：采用原目录留位、普通原文件逐节点保管

选择 **windows-anchored-directories-v1**。对原来存在的客户端目录（含原根和原子目录）保持准确原路径、原native directory ID、8投影及祖先pin；Codex的两稳定协调文件保持准确原路径/原对象。其它原始普通文件逐个通过准确原handle移动到同卷SID私有custody；官方工作文件在原来的目录里create-new，和原件对象分代。**不再要求移动包含已开claim的根或子目录**，也不关闭claim换路径来满足rename。

这是一种保管布局改变，不缩小原件内容/存在性/目录ID/8投影恢复范围，不把root ID相同当全树已恢复。Windows native目录有开文件不能移动的限制保留；不使用POSIX rename flags、ShareDelete新试验/强关其它handle、copy-delete替代原ID或修改官方协议。Grok没有稳定锁名称例外，不能按文件名复制Codex豁免。

被否决方案为“只释放claim并依靠N/L/K移动整根后重开”；实际官方claim-only writer不能被N/L/K排除。不存在一张Pending标记能让其它官方writer尊重Desktop journal；因此不引入ClaimRelocationPending的无锁写许可，不因原件hash相同认为空窗安全。已做shareDelete失败无需重跑，不增加超时。

## 协调域及锁的准确生命周期

1. 在原Token/SID、准确Profile/H/I/source/phase上下文中，先查原目录/各稳定文件归属、ID/8投影/rollback头/无sidecar/ADS。NativeLease依原顺序取得claim EXCLUSIVE与N/L/K原ranges；每个**允许的捕获、封存、恢复变更临界段**内claim不得中断、重开、降级为路径数据。N/L/K是各自协调域，不是claim替身。
2. 原目录及两协调文件不移动，lease绑定的client路径/ID保持；将原普通文件逐节点保管、发布工作副本，以及恢复/退休工作文件时，必须实际持有同一claim handle/ranges和其它必需锁，全段继承原deadline。锁忙/未知/范围读回或原件变更错误即poison/Required；不得释放重试、catch-up或把新handle当原连续持有。
3. **不是整段运行期一直独占claim**。完成临界段、刷盘并核完整工作视图/原件custody后，记录LeaseReleasePrepared；原生释放自己本次lease后记录明确结果。只有当本段不再执行客户端变更，才可让原包进入自己的claim协议；否则会自阻官方启动/teardown。这次释放是已完成临界段的终点，不是为了根移动制造的空窗。
4. Ready/PreStop/current/restore每次需新动作权限时，重新按实际当前对象取得完整协调域，再验证前后原scope/source/记录/内容/工作代次连续性；不跨释放窗口继承mutation权限、不把“曾持有”当还持有。官方teardown/HTTP可能自己取claim，不能在持claim时同步等待其取同锁；分段前必须留Prepared、原件保管与确切输出，分段后新lease验证真实官方结果/退出/副作用。预算不重开。
5. 恢复临界段结束后，证明原两协调对象身份/8投影、原lease已正常释放及原client/进程集合/监督/端点的后验。普通Stop不能靠Drop隐式释放或进程结束来补“已释放”；失败清理独立记账。所有来自租约的closure/callback只可借用活lease，生命周期/类型不能让原凭据写权限逃到unheld期。
6. 两稳定文件内容/SQLite状态沿原已采用的有限释放规则及当前NativeLease字节稳定检查，不新增“所有SQLite都可变化”的例外。本次布局不修改它们，不增删schema/PRAGMA、不打开SQLite做恢复/初始化；副作用若要求改变受保护锁字节仍须原范围判定。journal/wal/shm不是例外，任一新/未知sidecar拒绝，不能先删再验。

临界段外官方正常运行可能产生其已声明副作用，这些必须靠原AllowedContent/精确归属与fresh阶段比较证明；不能宣称claim在官方运行时一直被Desktop持有。原件custody始终准确对象保管并受私有权限/原handle/pin保护；任何外部原件编辑仍拒绝恢复覆盖。

## 完整节点计划及恢复算法

在新会话任何客户端变更前，capture原整树完整有界manifest，按原路径/原nodeID排序固定节点集合，kind只分DirectoryAnchor、CoordinatorAnchor、CustodiedFile、OriginalAbsent。每节点绑定原parent ID/原卷ID、8投影、文件原字节/hash/长度或目录原children，路径必须相对准确client根，拒绝链接/reparse/未知流/重叠/重复/父链不全。子目录也保留原位，不将其当可整体rename以再次遇到开文件限制。

- **DirectoryAnchor**：原存在的目录留准确位置；有原非DELETE/metadata/enumeration句柄和祖先pin维持身份，实际能力按Windows sharing读回。目录children按操作计划预测变化，不允许以“目录集合变化正常”忽略额外节点。8投影不变，timestamps不是本版8投影恢复证据。目录有其它不可读元数据则按原边界拒绝，审计SACL范围仍ExcludedUnknown。
- **CoordinatorAnchor**：只有原Codex准确两文件在其原根、原ID/归属成立且符合稳定锁有限合同才采用；claim/NLK实际锁权归NativeLease，不归字符串kind。owner稳定文件的释放证明仍独立，不能因为叫anchor就免检查。原缺席文件不能无记录地创建/收养；初始化仍是独立未完。
- **CustodiedFile**：每原文件同卷私有custody target与不可变nodeId一一对应，先OriginalMoveIntent刷盘/current CAS，再用原native filehandle准确rename，不覆盖target，随即核target原ID/字节/8投影和parent。copy仅创建独立工作副本，不替代原件。当前文件原ID取不到或已经打开而无法移动，拒绝该节点并保留已经发生的准确前缀，不改ACL/分享策略强行成功。
- **工作文件与新增目录**：WorkCreateIntent先持久，create-new准确父目录后登记新ID/字节/8投影/子集合，标明与原文件对象不同。官方新节点只有在源绑定副作用白名单/真实Ready与PreStop封存中被证明，才成为可退休的受控工作节点；未知新增物拒绝。每个原文件空缺期间属于Prepared/Preparing，不Resume官方、不发布Ready。目录仍原位不意味着途中配置可用于运行。

恢复时同一原lease内先核原树/custody、原受控工作视图及真实完整Stop/副作用许可。对每工作文件发布WorkingRetireIntent，准确handle同卷移至私有退休区，不删除/覆盖未知路径；核原退休对象全量内容/ID/8投影。原文件RestoreOriginalIntent刷盘后以custody原handle回原路径（目标必须Absent），核原parent/原ID/字节/8投影。原目录从未移位，按原children最终精确读回；原不存在但本代创建的工作目录在其所有受控后代退休、无未知/开放子对象且准确证明可移动时自底向上退休（非删除），原存在目录不退休。额外空目录也需判定，不能忽略。

各节点完成后才允许RootRestorationVerified；两根都满足原存在性/全部节点/父根/安全与两稳定锁有限释放事实，再结合真实native退出/监听/record/supervisor/source才能Closed。中断/权限/任一scope变化仍保留准确pending与custody/退休树，禁止Start/Restart/Auto后继，不凭文本恢复原生资格。

## 原缺席根及首次初始化：仍是独立必需路径

本方案直接解决**原已存在目录及原协调文件具备安全准入**的根移动冲突，不把此前OriginalAbsent改为“允许留一个目录/两锁”。原根/claim不存在时，NativeLease不能只打开缺席路径成功，也不能先创建对象再把原始缺席抹掉。须先在原父目录pin下记录原缺席，初始化创建每对象有独立create-intent及原新ID/安全/内容事实；初始化与官方协调范围若未完整来源证明，不进行生产Start。

最终若原根缺席要求回缺席，必须退休准确本代生成根（含所有受控协调文件），不能在claim还开着时rename，也不能在未排除claim-only writer的情况下关claim移动。**本r6没有授权这种空窗**，亦未证实一般no-gap原生退根机制。Windows应将该分支保持原gate拒绝、明列未验收；可继续原有/新增纯数据/无锁组件工作，但不能以现有根正例代表首次空环境。若原生实验/来源核查能给准确退根排他方案，再带最小原对象/句柄/操作顺序证据回交决定；无新假设不重复原raw5。

原Grok无claim且不存在时，可继续原已证的准确原代次退休路径，但仍须source/工作树/sideeffect/归属完整证据，不能据此推定Codex同样。整个版本首次空环境要求未删除；若这一分支未闭合，本版最终验收仍未完成。这是明确剩余产品能力，不用新增稳定锁例外或修改原存在性断言填补。

## 新canonical格式，不迁移旧移动记录

选用新原件layout和 **windows-client-originals-v3 / windows-client-current-v3**，storageStrategy=windows-anchored-directories-v1，securityPolicy仍r4 windows-bounded-access-v1/8投影/ExcludedUnknown。共享CONTRACT_VERSION=3只指适配合同；pipe/rendezvous与官方记录schema不动，四trait签名不改。v2 root-move/pending原样保留，不自动转换/续写，不从其文本Closed签v3许可；新session没有旧未决时才开始v3。

manifest新增storagePlan、storagePlanDigest；每phase/current同时绑定storagePlanDigest、securityPolicyDigest、manifestDigest及同SID/Profile/D/H/I/source/owner/session/代次。storagePlan列出完整原node目录树、原缺席/初始化依赖、准确稳定锚点及每原文件target，拒绝任意请求修改策略/节点排除/额外anchor。它是单一canonical journal的组成，不建第二恢复事实源。

逐节点事件仍写同一不可覆盖phase链（可在单phase封装有界原子节点事件），字段eventId、nodeId、rootId、operation、from/to相对路径及原native ID、parent ID、expectedBefore/After摘要、leaseEpoch/claimKey、deadlineScope、previousEventDigest。事件顺序限：NodeMoveIntent→NodeMovedVerified→WorkCreateIntent→WorkCreatedVerified；恢复限WorkRetireIntent→WorkRetiredVerified→OriginalRestoreIntent→OriginalRestoredVerified。Absent/new目录有明确Create/Retire事件，anchors只允许SnapshotVerified/ReleaseVerified，不允许Move。字段拒绝重复/未知/skip/逆序/cross-node/cross-session/claimKey漂移，不能凭数据mint实际claim权。

事件Intent刷盘并读回后才native动作；完成事件必须来自同原handle/leasedriver当前读回，不能仅写JSON。完成事件/current CAS失败后保留原事件与已发生native后像、poison，不默认rollforward/catchup。Ready/PreparedRestore必须**完整计划每原普通文件与工作节点**完成，Restored/Closed必须全部原对象回原处、声明工作节点退休且锚点/children/释放证实，不能只保持旧“双根已move”布尔。新节点进度须进入现有阶段与current，不可旁路旧RootProgress来让未完整树通过。

已有history/后继/source-route pin/同窗样本继续绑定本代strategy/manifest/父历史摘要与实际nativeproof；新后继重新capture当前用户原件，不把前代锚点缓存成当前授权。失败恢复只定位数据，原broker/lease丢失时Unknown/受限恢复，不从JSON重建；旧v2 history只按原严格边界读，不为兼容迁移降低检查。

## Windows后继写集与验证

Windows在3e1907584或无并行冲突的干净后继合入本批main共享常量；r2/r3/r4/r5逐文件允许集继续，新增明确windows_client_originals024/tree与v3节点计划/phase/reference/progress/coordination/adapter/history及其native tests内部实现。可新增同目录anchored/v3文件，由Windows父mod注册；旧v2实现/失败目录不删除以抹历史。共享trait/Status/route/HMAC/官方来源与其它平台不改，遇新范围/原缺席退根策略或权限环境变化带最小证据回交。Mac交接后不并行写Windows范围。

顺序：先原存在目录/锚点＋单原文件带真实claim互斥的组件；v3全计划纯反例与native中断可并行；再已有根Codex/Grok整树捕获/封存/恢复/后继。完成前原durable Start gate不解除。官方初始化/原缺席/ownerreceipt仍分列，真实原包闭环前必须组合NativeLease、声明副作用、canonical Stop、broker原Owner/H-S-D，r5同一采样5秒还需真Closed同代/并发正例，不借本决策跳过。旧GUI空组件/route/deadline/Bun已过不重跑，只有受本改影响的增量验证。

| 组 | 必需正例 | 反例/状态 |
| --- | --- | --- |
| A 连续claim | 原根/所有原目录/两锁ID与8投影留位，lease持续期间逐原文件往返，fixed Bun claim-only竞争从开始到最后native写均busy，释放后可取 | 同source来源但只N/L/K不得mutation；中途claim掉锁/换文件/失败重开/不完整sidecar拒绝 |
| B 节点计划 | root和nested目录非空/空、多个原文件/工作节点，同卷私有custody及D跨卷；恢复每原ID/内容/8投影/全children一致 | 同路径不同对象、额外空目录/未知节点、硬链/ADS/reparse、open普通文件移动失败保留、锚点被替换/移动均拒绝 |
| C 持久中断 | 每节点intent/完成/工作创建/恢复/退休、phase刷盘/current部分写等各窗口，真实原件前缀与准确引用一致或保持pending | 缺节点/错序/文本Closed/漏子集/外改/poison/多writer/stale history拒绝，不强补记录或删树 |
| D lease分段 | 原包own claim回调只在Desktop临界段释放后，Ready/Stop新lease同实际scope重验，所有操作用同一动作截止 | 等官方持claim造成自锁、释放期native写/lease callback逃逸/now+预算重置拒绝 |
| E 原缺席 | 原缺席事实/初始化依赖/新ID/所有创建过程记账；安全退根只有独立排他原生方案证明后才实测 | 当前未解决Codex原缺席退根，保持blocked；不留目录/两锁冒Absent，不关闭claim+只持N/L/K偷过 |
| P 生产 | 已有根component齐后才依完整前提运行单SID原包Start/Status/Stop/后继与真GUI官方owner/H-S-D，最终Start30/Status5/Stop10/Restart30含Stop子10 | 无生产前提不真实启动，旧组件不代生产；首次空环境/NativeLease初始化未完不得全版accepted |

每轮先可证伪假设，一轮独立全表/首错/实际trace/源码tree/工具链/EXE参数SHA；先批量修再新轮，不无假设重跑原raw5或首1314。普通原Token、不改日常配置/策略/官方，无真实凭据/模型/业务同步；只合成数据库及固定Bun组件。失败scene先在原SID可读时完整私有归档核验再清准确自有对象；所有旧自动审批拒绝不绕过，不把ZIP当NTFS元数据克隆。普通MSI/B2非空CAP-trust/全部历史Profile/Public/失败broker清理不在本决策关闭。

本合同只冻结layout选择与前后置断言，主线共享常量不实现保管/原生许可。Windows新实现和新真实正例仍待，不W2/双SID，不发布。未定的原缺席退根直接说明为未完，不再把已明确的existing-root布局叫缺共享接口。
