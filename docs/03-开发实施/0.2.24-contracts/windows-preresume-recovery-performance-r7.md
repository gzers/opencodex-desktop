# 首次Resume前失败恢复与完整镜像优化 r7

revision: 7；2026-11-17 Asia/Shanghai；TEAM17/IMP143。Windows源码输入 **44bb9bdbabc78f1754b9df135f3d183205fcb9ec**，实际r2产品 **357210891f14a1851881e6192bd09d7031471707**，文档 **e34f3fe8a**完整SHA见本批integrated-source.json。共享代码输入89e0195acb7a0c6e7549eafa376ec520ce981546。承接r4安全8投影/ExcludedUnknown、r5原状态窗口及r6原目录留位，不修改官方、普通Token/日常策略、Start30/Status5/Stop10/Restart30含Stop子10或未知拒绝。本合同决定恢复路径和共享优化责任，不把新源码当已载入旧宿主的能力。

## 确定现场及复核范围

原SID S-1-5-21-587408064-856152790-2583328096-1003、marenben/session7/noJob；r2首Start **30699ms/30000ms Timeout**。broker **75744/FILETIME134393656723346634**、原Node **21500/FILETIME134393657318681068**。原创建者报告creatorResumed=false，原Job total1/active1、无后代；3原文件已进入custody，3工作副本已发布；canonical session **7e3d4697-dd04-4d8a-b40b-f4468fab88c0**，seq9、Preparing/Released、workMode=Open、readyGeneration/closed=null，OriginalSession tag未返回。没有tag不表示没有原件变更。所有PID/记录均只为精确定位，最新存活/权限/变化必须Windows同SID重新观察。

任务根 **C:/Users/marenben/ocx024-r6-existing-20261117-r2** 下I/D/H；准确manifest SHA **a3b159e09f7e6c48f75d9f597e632d0f0ad545439e4c7682d25196c903a2cd11**，current seq9 eventDigest **1659c2a9488db1b51a258e919202422a6c1a1b534393b17c49a6e952bbeecdc7**。二者是核对数据，绝不授Owner/Job/恢复许可。

| 归还原对象 | 原fileID（卷247287385） | 原字节 | custody目录ID |
| --- | --- | --- | --- |
| .codex/auth.json | 562949954857826 | 3 | 562949954956008 |
| .codex/config.toml | 562949954857823 | 51 | 562949954956008 |
| .grok/config.json | 562949954857829 | 3 | 281474978245353 |

两目录准确为Profile/.ocxd024-originals-c3db7afd-959c-44fb-8743-1cca384043c8-codex及同前缀-grok；每个target/nodeId/parentID/原内容hash/8投影直接取固定manifest并与原准备证据和**实际原生对象**逐个核对，完整白名单归本批receive-result.json。不以文件名相同或上述表中的ID数据凭空接管。四anchors留位，额外3个N/L/K协调原件保持原路径/原ID/8投影/内容，不新增“可删除SQLite”例外。

本轮远端净79018文件、50轮历史；**只导入并核验四个直接相关轮次与handoff共555文件**：当前r2失败接收、扫描计量、旧两目标获准清理回执、旧r1完整归档回执。4份manifest共550payload/12,872,546字节匹配；其余历史在固定远端docs Git对象保留，不宣称已全量集成或完成代码安全审计。minimum-recovery-interface-request.json和shared-performance-request.json已完整消费；原r1归档和50580/54692清理只能证明旧场景，不覆盖r2原件或21500/75744授权。

## 一、当前已加载宿主：选择受限外部任务修复，不热补/收养

选择独立、单次、原SID的外部任务修复器，针对上表**无凭据合成基线及准确r2现场**，输出 **ExternalOriginalsRestoredBeforeResume**。这是明确指定原件的任务修复，不是从磁盘重新创建NativeSession/OriginalSession、原Job或旧Owner权威。不能将新方法装入已加载357宿主，不能替换其I/ocxd.exe、注入进程/恢复线程、把运行中的程序换成新build或通过JSON重复Start获得旧Store。

当前宿主failure-control协议只有Status与强制原Job清理。Status需要原native同image/原连接fresh认证；旧同image客户端是否有可执行的只读Status入口须先核验，**不能假定新工具能通过旧宿主的exact-image准入**。已有精确I/ocxd的内部cleanup命令可在旧宿主内调用原registry/Job，但它只返回forced事实，clientsRestored/normalStop/closed=false、hostMayExit=false；它不释放所有custody句柄、不允许正常退出宿主，也不恢复3原文件。任何原通道不可达/身份不符就保留现场，不能降为PID cleanup。

### A. 先完成实现与只读预检，再确认实际恢复

Windows现在可实现修复器、在新自有合成场景验证下述全部断言、对当前r2只读预检并生成**精确可审查计划**。当前r2新的forced结束/外部恢复执行需要在原Windows对话单独确认；旧50580/54692授权不延伸到21500/75744。本项目AGENTS.md要求“高风险迁移、发布、恢复和全局同步必须单独确认”，当前用户请求是决定合同并交接，不在Mac执行或自动触发新的高风险现场操作。只在Windows这份完整计划/全归档/前置证据齐备后问一次，不以等待时间或旧清理回执当批准。

计划列出原SID/会话、两个准确PID/FILETIME/imageID/hash、从未Resume的证据组合、原Job清理方式、是否确需终止broker、3个原件及3个待退休工作对象各自ID/hash/8投影、完整目录membership、准确原/目标路径、预计动作10秒预算、归档位置/hash/原SID核验及所有失败保持行为。任何差异先记冲突、保持原样，不扩大到其它文件或40588。

### B. 全现场归档及原状态核验

1. 先在原普通SID准确Profile/session中，归档**本次r2**完整I/D/H、两custody、准确客户端全树/协调对象、控制locator与原输出/意图及原始setup；读分享不改变、逐member/hash与前后membership验证。保留ZIP只是字节归档，不克隆ID/8投影/ADS；原ADS如存在须原生单列，未知流拒绝。若仍有日志追加，记录每帧稳定snapshot边界及后续差异，再在进程静止后最终封存；不能拿旧r1 10641文件归档代本r2。
2. 持两目标**新观察句柄**，从OS取得原SID/session/FILETIME/镜像ID/hash/未提升/noJob，原Node的挂起状态与线程/命令谱系；这些句柄只是这次明确任务清理/修复的观察对象，不冒原creator handles/Job拥有者。关联归档的原creatorResumed=false、同原加载image代码没有failure后Resume路径、真实线程Suspended与原Job诊断前后，才认定适用before-first-Resume分支；任一可能已Resume/曾有后代/记录变化/证据缺失都拒绝此修复路径。不能单靠当前线程Suspended证明从未运行。
3. 原完整source/current及H/config、manifest/current/events1..9逐原hash/前驱/节点plan、全部原件对象、工作副本/anchors/非产物节点均实际读取。所有必需8投影/raw native状态保持r4；不修ACL/ownership、越权读、创建缺失协调文件或吞1314。原scope/source、外部编辑或unknown membership冲突即保持Required。

### C. 撤销失败进程访问能力，分开记录forced事实

在Windows用户确认的本次精确计划内，优先调用**原加载宿主已有命令/原私有通道**，由其实际持有的原Job作一次forced清理。新命令不重建Job。要求原Job total/active/保留成员与原未Resume launcher一致，结束后原Job空及原Node退出的原回执可验证，保存原raw242/forced=true。

若该通道不适用/不可达，当前方案**不自动改用外部终止Node**；需同一审查计划明确精确新观察句柄的外部forced范围和全部成员未知边界，再让用户确认。不能把只有两PID缺席当整个Job已清。若只有当前Node根退出证明而Job未知，禁止本恢复步骤继续。

原Job静止后旧broker仍持文件/Store，不具恢复/正常退出命令。若原现场不能在其保留句柄状态下以新native修复器独占核验/移动原对象，就需**明确终止准确broker75744的失败宿主进程对象**以释放其文件句柄；这是当前受限方案的一项独立forced操作，必须在上述计划/确认中列明（不能把hostMayExit=false改真）。在原Job已确认空、原件归档和全预检后，核验原持有观察handle仍同FILETIME/image/SID，才可按已确认范围执行。绝不Resume launcher、改旧程序、关闭别进程随机handle或结束40588/共享进程。forced后用同观察句柄等退出与raw码，核无新端点/新实际运行记录/成员，任何未知都停。

这里forced清理只建立**本次受控失败进程已静止**的外部前置事实，不能产NormalExit、StopAccepted、Stopped、Closed、ClosedEligibility/后继资格。原失败Start仍30699ms Timeout，永久保持失败，不因恢复成功改写为正常Start/Stop通过。

### D. 新native修复范围与精确归还

修复器从新原SID实际打开原父目录、两custody、三个准确原文件和三工作文件/anchors，独立保留native对象/8投影/内容/children；批准的精确任务范围加这些**实际原件验证**才形成本次修复能力。旧manifest/时间/PID只是定位与核对数据。不得为通用“旧记录恢复”建自动迁移/收养路径。

在新恢复动作的原10秒窗口内按准确原claim/N/L/K实际对象取得全部原ranges，清理/静止证明与source/current/原8/完整树重验；claim连续只在本变更临界段持有，不移动根/anchors。逐个先发布原工作副本RetireIntent，准确原handle同卷移至新SID私有退休区（不删、不覆盖）；再OriginalReturnIntent，原custody handle同卷返回原目标（必须Absent），核原ID/parent/全部bytes/8投影。任何工作文件被编辑/替换、额外文件/目录、原custody drift、sidecar/锁忙/流未知皆不覆盖、不重试当前动作，保留已发生准确前缀和全部原件。

实际full树读回恢复原10对象和本次原协调对象的身份/内容/8投影/children，准确原锁释放、无失败进程端点；私有退休区完整对象保留、原journal/失败证据不删。**新外部修复记录**schema=windows-external-preresume-repair-v1，只记录同一旧manifest/current摘要、确认范围、原进程静止来源、每节点真实前后、锁及新10秒deadline与结果，不能续写/修补旧seq9为Closed/Aborted，也不修改旧字段使其看似由原Owner完成。旧事务留为外部修复已关联的失败原件，缺任一结果保持RepairPending/Required。

成功输出ExternalOriginalsRestoredBeforeResume及normalStop=false、closed=false、allAccepted=false、publicStartOpen=false；不签自动再启动许可。后续另一个新隔离测试必须重新核验当前原件和旧失败会话不再拥有mutation权，保留全部外部修复与旧history，按明确qualification新任务开始；不能复用失败Store/将external回执当NativePredecessors、删除非空旧namespace或拿旧hash直接重试。没有这份接续边界或原件未还原前，**不再跑下一轮官方Start**。

## 二、后续宿主：首次Resume前失败的原生恢复接口

新增共享 **BeforeFirstResumeRecovery**，三个阶段prepare_abort→quiesce_launcher→restore_originals，关联PreparedAbort、QuiescentLauncher、AbortRestored为native字段私有/nonClone/nonserde类型；不与ClientOriginals::ClosedEligibility/NativeNormalExit/ServiceObservation兼容，无默认成功实现。**BeforeResumeRecoveryWindow::begin(started,parent)**固定独立显式恢复动作最多10秒/原parent更短，不接受未来start/已过期，不可clone后重设deadline。不是对旧Start30、Stop10的补时，也不是自动重试。

共享CONTRACT_VERSION=4只标明本适配合同，beforeResume恢复记录标识windows-before-resume-recovery-v1；不据此自动更改旧pipe/rendezvous/官方运行记录或v3原件schema。新恢复命令/严格字段须版本化并有拒绝旧协议的测试，旧加载宿主不会因此具备该命令。

在创建最初Owner和首次客户端变更**之前**，原宿主安装可达的private控制＋PartialCaptureAnchor（准确store/slot/session/source/原action/原创建句柄），所有fallible capture分步一旦产生原件/工作节点/锁/引用就先登记到原保留anchor，再返回错误或继续；没有OriginalSession tag时也可按准确原native prefix恢复。不能只在capture返回成功后才有恢复入口，不能Drop原Store或把None当无原件变更。

prepare_abort由原宿主fresh认证的新显式动作借原实际Owner：验证原进程/thread/原Job与monotonic creatorHasResumed=false，原source/H/partialStore/current/prefix/8投影和失败事实，无Ready/后代/别段动作。**不调用过期Start的mutation check来续原权**；只读验证原Origin元数据，创建新的受限recovery origin，始终绑定旧失败UUID/prefix，原窗口仍过期。暂停/撤销该旧Owner的Resume和一切普通调度，锁不能被其它请求重新激活；failed/pending部分存储先安装再检查，保证prepare失败仍保留。

quiesce_launcher可由原Job显式终止准确从未Resume launcher并记录forced/raw；不把forced变NormalExit。首次Resume状态或Job成员不确定则拒绝本接口（需不同显式恢复方案），不能补一个虚构Ready/HTTP200。原宿主始终活着保留Store/custody/原句柄，unless bounded shutdown explicitly proven；不为了释放普通文件handle先杀自己。

restore_originals在同一恢复10秒内，借原partialStore/原件handle与新实际claim/N/L/K、精确完整工作视图按v3 node前缀执行有界退工作/原件返回/anchors/全部8/字节/children与锁释放读回。它不要求不存在的Ready、StopAccepted、官方teardown，却必须证明从未运行官方launcher和没有该分支不可解释的副作用；新动作仅能返还原件，不能获得继续初始化/Resume权限。未完成准备/部分move也必须覆盖：既未动节点读回不变，Intent未Completed据原native后像精确核对，未知绝不推测补完成。

同一canonical链新增明确AbortRecoveryIntent/逐节点恢复/AbortRestoreVerified或AbortRecoveryPending事件，绑定原失败action/manifest/policy/storagePlanDigest、fresh recoveryActionId/deadline/failedPrefixDigest与实际原件对象；旧Start Timeout事实不可改。当前schema能否严格表示新union须在实现前给exact schema变更与负例；若旧v3严格解析不支持，采用显式next版本或当前节点事件+类型明确外部附件，不能吞未知字段/自动升级旧记录。当前已加载seq9不追溯此新工厂、不从JSON构造PreparedAbort。

成功为 **AbortedBeforeResumeOriginalsRestored**，允许只读核验/展示“启动失败，原件已恢复”；**不等同正常Stopped/Closed，不产生自动Start/Restart/Auto重选/NativePredecessors资格**。后继新测试单独fresh admission；若以后要复用此恢复到Start，另给明确新代捕获合同，不顺手放开。expired/锁忙/外改/日志失败/断连均留原件、准确prefix和Required，不自动force或循环开新10秒。

## 三、共享完整镜像扫描：责任与写集明确转交

责任分配：**Windows承担本批共享扫描的Windows实现、阶段计量与真实MSVC/native性能验证，Mac承担共享语义审查/净差集成与Mac回归**。本合同发布后，Mac暂停下列性能临时单写集，Windows可以连续实测与批量修复，不必每一处成本变化回Mac。当前现场恢复优先；控制器自有副本/编译可并行，原件未还或当前宿主未处理前不得再启动原普通原包测试。

临时允许写：

1. **modules/runtime/program_image.rs**：跨平台完整inventory核心和其中Windows分支、同文件测量/测试注册；可新增同目录program_image_windows.rs、program_image_windows_tests.rs（或子目录），测试成本指标及调用计数。只优化同一次完整校验的实现，不更改Entry/manifest摘要语义、受控文件集合、全字节读取或安全准入/边界。
2. Windows owner/source/claim/v3/existing_qualification/failure_control/相关tests、test/windows024-native/**：受限恢复与全过程计量、反例/同源回执，保持r2-r6已有写集。允许在实际调用点记录actionId/sourceDigest/ordinal/原因/开始结束/剩余预算/文件数/字节数/失败阶段及全量verify进入/退出，日志不含secret。
3. **program_selection.rs与official_lifecycle024.rs**本轮只允许cfg(test) cost hook/只读观测及必要注册，不改选择/版本/来源/LaunchPlan/HMAC准入语义。**backup/safety.rs、hash.rs、runtime_executable.rs、Windows共享路径/原生安全模块**如现API不足仅提交最小建议，不能默默放宽；可在program_image_windows局部实现等强原生只读准入供本轮消费并测试。根Cargo/锁/workflow/其它平台语义/公开UI/官方包/预算不在自动写集。

实现顺序：先对原入口做有界、低开销计量（每全量调用ordinal与真实阶段），确认路径安全检查/枚举/元数据/open/read/hash/threadjoin成本。单次完整verify四样本4.8–8.9秒支持Windows成本风险，源码至少8次不等于真实动态次数，不能乘成实际30699ms下界或唯一因果。不在当前活失败宿主注入新计量，也不以副本debug数据签release Start通过。

可优化：相同一次inventory中父链/目录项的重复检查、按原句柄/原目录ID与scope绑定的临时局部pin及前后复核、可靠同对象元数据/文件open-read-fstat与digest组合、有限并行hash/任务分配及缓冲；每个文件仍读完整每个byte并计算相同hash，完整目录集合/空目录/模式/边界/manifest无例外，所有线程明确收束，不让后台验证越动作进入恢复。保留同卷fileID/路径真实身份、硬链/reparse/ADS及原安全要求，局部优化不能只以pathname/mtime相同重用检查结果。

**本轮不准删除实际操作边界的完整扫描或合并跨边界扫描**，也不跨动作缓存摘要。若计量证实同一边界有完全重复扫描，可给出保持完整不可变原生句柄集合/拒写删覆盖/原current/source前后复验、每文件内容覆盖的具体批次证明再交Mac采用，当前不能凭“同action/已扫描过”直接删调用。原像在外部可能变化时必须重新完整读取，selection毫秒级复核不能替全树。

固定预算继续：scan若超过原动作期限，返回Timeout/Unknown并进入明确恢复；新scanner可加显式deadline传播/检查、join bounded协作，但不能静默新开时间或让已过期结果成功，不能跳余文件来“准时完成”。若同步OS读不能及时中断，记录真实超出与失败，不伪报严格预算通过。调worker数保持有界、计入原总量和线程归属；不修改机器页面文件/安全策略或用release断言不同掩盖失败。

## 后续执行顺序与验收

| 阶段 | Windows工作/必需证据 | 拒绝/限制 |
| --- | --- | --- |
| R0 当前只读 | 新r2全归档、全部原node/工作/anchors、原process/loaded-code/pipe能力及计划、source/当前prefix逐原native核 | 不代读旧ZIP、不拿旧两目标授权；未确认不force或还原本现场 |
| R1 当前任务修复 | 精确计划在原Windows对话一次确认后，原Job静止→必要broker独立forced→新native修复→3原ID/bytes/8/全tree/锁释放回执，所有退休物/旧seq9留 | forced≠Stop/Closed，不能覆盖外改/复制ZIP重建原ID，任何成员/历史未知拒绝继续 |
| R2 后续原宿主接口 | capture前注册partial anchor/控制，before-first-Resume的原创建/原Job证明，capture每个前缀及tag缺失故障下restore，新恢复action原10秒 | 误Resume/后代/错scope/replay/过期/断连/锁忙/部分归还保留，不虚构Ready/NormalExit/后继 |
| P0 同源计量 | 固定image每完整verify全阶段/文件字节数，至少worker1/原默认；复制源变更拒绝、还原全量读过 | 副本/debug不是原生产/release结论，已失败30699ms不改 |
| P1 单pass优化 | same inventory/digest与参考实现，删除/新增/空目录/改一byte/大小相同改byte/hash/权限/重命名/reparse/硬链/ADS/并发替换/线程失败/上限/取消反例 | 每节点/所有byte/full hash仍核，不缓存旧摘要/metadata替代/缩集合/放宽未知，失败线程全部归属收尾 |
| P2 定向产品 | 当前r2原件恢复且未来接口负例齐，精确新source/EXE/release profile；单SIDStart/同采样Status/Stop/后继等按原前提真实计量 | 未过前提不跑真实官方新轮；Start仍公开拒绝，仅原批准隔离通道，不W2/双SID/发布 |

后继提示词给代码/文档精确SHA。Windows范围内可连续完成恢复工具、未来接口与性能改动、批量定向验证及时推送；方案或原生权限/新不可读scope变更时带具体事实回交。原缺席Codex安全退根/初始化/ownerreceipt/最大batch容量/普通MSI/CAP-trust/GUI官方Owner/H-S-D/历史清理与NAME等整版未完不由本次关闭。全部失败目录/旧删除拒绝保持，不改官方或日常配置/策略、不真实业务/模型调用、不发布。


## 2026-11-17增补：旧宿主缺只读Status客户端的原Job查询边界

消费Windows r7-async的blocking-request.json/r7-loaded-job-status。已复核载入源357210891：failure-control服务端允许Status，但ocxd只暴露确认式failure-cleanup客户端；原Job为匿名CreateJobObjectW，原宿主/monitor保留其句柄。**不能凭源码中存在Status分支称客户端可用，不能把旧total1/active1当当前证明，也不能执行cleanup来试探当前状态。**

决定允许Windows先在**test/windows024-native/**新增独立轻量的、仅查询的原Job观察原型，优先使用有文档的PssCaptureSnapshot/PssWalkSnapshot句柄快照＋DuplicateHandle只读复制。它观察原内核Job对象，不伪造旧same-image客户端、不成为NativeOwner/恢复权限。此增补只解除“无可尝试的只读观测路线”，不是宣称已经能读取原Job；真实成功及全部否例仍须原生证明。产品四trait/Schema/预算/当前r2恢复确认均不变，不新增全产品构建依赖。

### 查询实现的有限写集与前置

1. 原型先只针对自己创建的broker/匿名Job/从未Resume launcher夹具，真实普通token、不提升/SeDebugPrivilege/改ACL或策略。只打开已经逐native核准身份的broker（当前现场对应75744/原FILETIME/source/SID1003/session7）和准确Node21500观察句柄；所需Process查询及DUP_HANDLE权限先最小值实测记录，不用PROCESS_ALL_ACCESS/PROCESS_VM_WRITE/CREATE_THREAD/SUSPEND_RESUME/TERMINATE。缺权限即Unknown，禁止换提升令牌或驱动。
2. 用PSS_CAPTURE_HANDLES及必要的handle基本/类型信息，完整遍历该**单一broker**的句柄快照，核返回标记/类型名称长度/完整终止状态。不得开VA_CLONE、线程上下文、内存dump或全机handle扫描；不读取/日志输出无关File/Token内容及名字。SDK若不能仅在这些flags下完整提供Job类型，先回实测阻塞，不转未声明NT解析或扩大读范围。raw错误/预算耗尽/截断/类型未知均不得跳过计成功。
3. 只对明确Job候选调用DuplicateHandle：目标为观察器本进程，desiredAccess仅JOB_OBJECT_QUERY，inherit=false、options=0；禁止DUPLICATE_SAME_ACCESS、DUPLICATE_CLOSE_SOURCE、远程关闭任何句柄、注入/重写入口或改变原宿主文件。本地副本只用于QueryInformationJobObject/IsProcessInJob/CompareObjectHandles，不调用SetInformationJobObject/TerminateJobObject或任何Resume。
4. 同一次观察内保留全部Job副本，用CompareObjectHandles识别重复句柄实际是否同一个内核Job；对原Node实际保留进程句柄执行IsProcessInJob，必须**唯一内核Job**匹配，不凭一个PID列表或命名/handle整数宣称原对象身份。完整两遍broker句柄枚举、对应副本对象比对与原broker/Node FILETIME/token/image/退出状态前后复核；原相关句柄或候选集合漂移/多个不同Job匹配/不支持比对/任一所需读失败均Unknown。
5. 从这个**实际原Job查询副本**取得BasicAccounting（TotalProcesses/ActiveProcesses/TotalTerminatedProcesses）、完整PIDList及ExtendedLimitInformation；在原预算内前后再次采样。当前beforeResume分支必须total=1、active=1、terminated=0、唯一实际成员为原Node；原Job禁breakaway的flags/原对象所属与源实现一致，原Node原线程/未Resume谱系及源/current/原件断言仍必须另核。计数不符、曾额外成员/退出历史、PID复用、静止/单线程不够、所有未知均拒绝此修复路径。**不把“当前只有1个PID”替代“历史total=1”**，也不把查询到Job替代从未Resume的组合证据。
6. 只读原型结束时只释放自己创建的snapshot/walkmarker和本地副本；原broker/Node/source/records/Job继续原样。原型观察句柄不持久化为权限，不给产品恢复mint proof。需证明自己的Job副本没有引入原对象最后句柄关闭副作用；源创建的Job配置及原broker存活仍在前后实际核对，原句柄从未关闭。

### 通过原型之后的当前现场与恢复边界

只有同源普通token夹具证明完整枚举/重复handle去重、正确Job匹配、从未Resume/历史退出/多Job/错PID-FILETIME/权限拒绝/候选变化/超时均严格分类后，Windows可在本轮已授权**只读R0**中对原现场应用相同原型，保留原字节源码/新exe hash/原生事实及完整r2归档。只读收集不要求用户为无副作用查询再授权；此处不授权执行R1终止/返回原件。

R1精确计划如继续采用原宿主已存在的cleanup命令，须把上述**同一个原Job的查询副本**保持到前后查询：先唯一原成员与total=1，再执行用户批准的原控制cleanup，之后同一查询句柄total=1/active=0/terminated计数与该forced事实一致、原Node同保留进程句柄退出/raw码、原宿主回执一并核验。**查询副本只替代不可达只读Status的观测来源，不取代原宿主的终止权，也不放行外部TerminateProcess(Node)**；原cleanup认证/路径不可达仍拒绝。

若释放原件句柄确需终止准确broker，仍必须在同一具体R1计划中单独列出并获原Windows用户确认；query副本可在broker退出前后复核同内核Job为空，但不是NormalStop/Closed。任何新Job/额外成员/原查询能力丢失、counter矛盾、source/current/原件变化均不恢复。CurrentR2权限仍未批准、旧50580/54692授权不沿用。无法满足本增补时保留该依赖阻塞，Windows继续future-host和扫描独立开发；不得再以“只差Status”重跑官方Start。

### 实现验证输入与输出

共享代码无需新增接口，本次Mac只改合同/接收证据/IMP/TEAM，不触碰Windows独占源码。Windows基线2306f39880fde481a182e724c7a58bbbbf2a0310（该末尾若有本线新提交增量承接），main98787e64f152515a6aa8e5b3c31b0330beaeca67只含Mac封装装配；本轮允许写test/windows024-native/**里的新Job-query原型、fixture/负例/执行器及本线证据与handoff，不改原已加载image/旧namespace/产品权限类型。不得把DATA负例或controller夹具当SID1003当前现场读取通过。

回交顺序：先同源原生原型/拒绝表；满足后只读R0完整实际现场、查询结果/未Resume组合、全归档及精确R1计划；此时再一次向用户请求本现场恢复。已有beforeResume和完整扫描写集不变，可同时连续开工，缺共享ABI时才另回最小请求。不W2/双SID/发布。

API依据（文档只能证明调用语义，不替代Windows实测）：[PssCaptureSnapshot](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/nf-processsnapshot-psscapturesnapshot)、[PSS_CAPTURE_FLAGS](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_capture_flags)、[PSS_WALK_HANDLES](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_walk_information_class)、[DuplicateHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle)、[CompareObjectHandles](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-compareobjecthandles)、[QueryInformationJobObject](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-queryinformationjobobject)。本机无Windows kernel，未声称原型/当前Job已执行通过。


## 2026-11-17增量答复：PSS未知类型与名称信息flag

收到r7-pss-incomplete-type-information（3c7bc980e／wl2-r7-pss-query-20261117-r1）：flags4/20/52的91项中24项TypeName缺失，Windows未跳过Unknown。允许在原test/windows024-native/**自有夹具的PSS调用增加**PSS_CAPTURE_HANDLE_NAME_INFORMATION(0x8)**，仅为取得完整类型信息；可与既有HANDLES/BASIC按最少组合先实测，TYPE_SPECIFIC不因本决定扩展。官方文档说明此flag会捕获对象名称，故它确实扩大快照内部采集，不声称OS完全未接触名称；观察器不得解引用ObjectName或复制/序列化/记录名称内容，不持久化PSS内存，及时释放snapshot/walkmarker。

只取有界TypeNameLength/TypeName、相应有效flags与实际Job只读副本；原句柄完整遍历、双采样、CompareObjectHandles/IsProcessInJob、原历史accounting/原未Resume证据、权限最小化/未知拒绝均保持。不得借0x8开VA clone/上下文/远程读内存、SeDebug/全机scan/NT未文档fallback。自有夹具先证明未提升令牌、全部captured=walked/end259、类型未知计数为0、无ObjectName输出/持久化及原Job不变、错身份/超时/重复/多Job/历史退出等原拒绝表；若新增flag仍有未知或拒绝，保留该停点，不拿明确的两Job当全部。

此项只允许查询原型参数增量，不改变产品权限/安全承诺、恢复流程或执行授权；通过原型后才按已有只读R0规则应用当前现场。r2归档及44项已过而不受影响部分复用，不重跑整归档；原Job当前证明与从未Resume组合仍未过，不force/归还原件/新Start，R1仍等完整精确计划及本现场用户确认。文档依据：[PSS_CAPTURE_FLAGS](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ne-processsnapshot-pss_capture_flags)和[PSS_HANDLE_ENTRY](https://learn.microsoft.com/en-us/windows/win32/api/processsnapshot/ns-processsnapshot-pss_handle_entry)；本机不冒Windows实际执行。


## 2026-11-17增量决定：未完成捕获记录、锁线程及PSS剩余类型

消费Windows三请求：wl2-r7-preparation-record-request-20261117-r1/interface-request.json、wl2-r7-lease-affinity-request-20261117-r1/interface-request.json及wl2-r7-pss-types-20261117-r1/blocking-request.json。允许以下独占实现接续，不允许现场R1执行；原4trait/CONTRACT_VERSION4不变。共享main78dc2b8149c4b2f948f6de0d649bea7b46dada02仅新增BEFORE_RESUME_RECOVERY_RECORD_V2字符串常量，原v1保持。

### A. 明确独立v2附件，不为缺manifest制造摘要

采用schema **windows-before-resume-recovery-v2**，使用独立strict RecordV2/ReferenceV2解码与工厂，不扩v1 decoder或v3/current/union、不将无效v1回退v2，不升级seq9/旧记录。不要求改现有v1正常canonical producer；Windows新future捕获可显式选择v2，所有recovery同动作始终同schema。只有原Owner/PartialCaptureAnchor的实际对象工厂能提供输入，DATA合法不授任何native权限。

V2必需顶层字段：schema、sessionId、failedActionId、recoveryActionId、securityPolicyDigest、reference、sequence、previousRecordDigest、startedQpc、deadlineQpc、qpcFrequency、event。UUID/64hex/QPC字符串/链严格性及原64KiB单记录、10秒/更短parent保持；nullable字段都须显式null而非缺省。重复/未知字段/非法variant拒绝。sessionId必须原捕获槽一次分配并持有的真实session UUID，不从后来的manifest倒推或另生造替代。

reference是kind标记的严格二选一：

| kind | 必需且仅有的字段 | 来源与限制 |
| --- | --- | --- |
| canonical | kind、manifestDigest、bindingDigest、storagePlanDigest、failedPrefixDigest | 全部来自完整已初始化的原Store/manifest/当前前缀真实字节，缺任一不得选此variant |
| preparation | kind、ownerInstanceId、source、intentPrefixDigest、nativeStagePrefixDigest、lastStage、components | 来自原宿主实例UUID/实际PartialCaptureAnchor已登记的准备前缀；不出现manifest/storagePlan字段，不填zero/占位hash；两个prefixDigest只可hash实际已存在的有限DATA投影/已持久前缀，序列长度和source scope必须覆盖原已登记集合 |

preparation.source是严格状态variant：unobserved{state}、observedAbsent{state,parentIdentityDigest,observationDigest}、captured{state,bindingDigest,imageDigest}。不存在完整Source时必须unobserved，不拿配置路径/旧Source摘要补权；observedAbsent只能原实际native父/准确缺席观察证明，不能以read失败转缺席。components为固定五角色有序数组（source/custody/codex/grok/journal），每项严格role、state、prefixDigest、objects；state仅unobserved/retained/verifiedAbsent，prefixDigest显式Option摘要，objects为按原native stage登记的唯一稳定objectId/role/原path定位摘要/identityDigest/securityDigest/contentDigest可空DATA记录。unobserved或任何必需identity/security/content未核实必须保持Pending；未打开/创建失败不产Absent。全列表上限512node/原帧字节上限延续，超限拒绝。exact native对象/prefix仍保原槽，DATA不含凭据明文或可绕scope的新绝对任意路径。

lastStage用已登记有限stage枚举，不接受任意诊断文本；给原准备节点intent发布、native创建/原句柄放槽、原8/内容读回、journal/current实际发布各阶段区分。记录字段要求的是如实可表达部分前缀，并不要求为了记录而创建原本缺失的manifest或额外成功事件。

事件保持原v1的严格意图/回读/forced区分，但V2须独立关联reference.kind。canonical的LauncherQuiesced/WorkRetire/OriginalReturn/Verified/Pending断言不变。preparation增严格PreparationRetainIntent及PreparationRetainedVerified（objectId、role、originalStage、identityDigest、retainedMembershipDigest），仅描述本次实际已建准备对象原位保留，不表示原件已恢复。原本未动任何原文件时originalNodes允许真实0，必须原完整登记集合/实际全tree与初始观察证明零变更；不能仅用无manifest/无refs/空array推断。无法核全部对象/锁/源即AbortRecoveryPending；PreparationRetainedVerified本身不能终结为AbortedBeforeResumeOriginalsRestored。

成功AbortRestoreVerified对canonical仍须真实所有原文件/工作副本/anchors/原8/完整children/锁释放后验证；对preparation仅当实际原Owner证明完整初始对象集合、所有前缀、全部必要源与安全事实并精确撤销/归还本次副作用后方可同名terminal，originalNodes可为0但不能漏未登记创建/源未知。缺Source但仍有未知原对象只可如实Pending，不能借V2降低准入。normalStop/closed/publicStartOpen固定false，绝不mint NativePredecessors/自动Start/Closed。准备用新manifest过渡canonical必须实际完成初始化并明确新事件绑定两前缀，不准为恢复先“补完Start”。

Windows写集承接before_resume_record.rs，可新before_resume_record_v2.rs/tests及v3/capture/preparation/journal的原native producer，mod仅注册；共享V2标识由Mac本笔给出。先DATA严格正负例与真实原槽字段映射，再原pre-manifest各故障点native readback，旧v1负例和实际canonical producer复用。必须新增缺字段/Option缺省/错variant/造manifest/未知component/重复或丢node/不完整Source/零node有隐藏对象/旧v1转译/失败超期/两action重放反例。未知保持Required，不把DATA测试代原Owner完整恢复。

### B. 同一个实际worker持活动lease，不跨线程转移/不Drop补释放

采用一个原宿主拥有的专用恢复worker：在取得任何活动LeasePreparation/NativeLease前，将可Send的原Owner/CaptureSlot的**唯一原对象所有权或排他借用**绑定此worker；原registry保其不可复制的control、原session/action标识和一次注册的固定deadline，正常调度即时标RecoveryInProgress/Required。不得clone Owner/Store、从文本重建、跨线程移动活动lease/OVERLAPPED或unsafe Send/Sync/transmute，原Windows单写类型/API范围保持。

一个显式恢复命令只创建一次窗口；worker同线程连续prepare→quiesce→完整return/readback→精确release，三trait阶段不能等待用户/新的RPC批准来续期。客户端断开只停止观察，不取消worker或新开10秒；状态请求只读当前实际worker投影，不拿JSON当持锁权限。原Owner可Send资源转交必须留明确slot/worker binding及前后scope/source核，转交失败无新lease，保持原槽。普通Start/Restart/Stop在期间拒绝，不用等待mutex无限阻塞高频Status；注册表用独立control状态反馈Required且原5秒budget不变。

成功路径在同一个原deadline内完整native证明全部变更返还、同claim/NLK精确释放/范围冲突负控、原对象/源/附件链读回后，才将实际已释放资源和专门AbortRestored回执交回原slot；不签normalStopped/Closed或后继许可。任何条件/OS调用超时只能失败，不能把同步调用占时伪装准时成功。

失败/过期时worker撤销所有mutation许可，保持原Owner/部分lease/files/ranges/前缀原对象在同线程停驻，仅接受无副作用状态读取。**不能依赖Rust Drop或线程退出自动释放仍有义务的锁后报告成功，也不能重新调用prepare用新deadline接旧动作。** worker panic/崩溃必须记录LostWorker/Unknown、原进程句柄/锁未知，禁止平台drop自动清理结果冒原正常释放；原异常不改变原失败事实。该保留状态不是主动后台循环恢复。

后继若需要释放失败持锁以便人工修复，须明确单独FailedLeaseDisposition：由原worker就其准确原held ranges/source/partialPrefix/实际进程静止和所有已发生副作用可证明状态形成可审查计划；未证明则Required。当前本合同不授权自动unlock/drop/force/替换工作进程，尤其不能放claim只靠NLK做目录变更。具体异常清理实现/计划先留证再决定，原实际现场高风险恢复仍单独确认。允许Windows先完成成功路径与“失败原worker保留、观察不续期”正负例，不把失败处置设计未完扩大成其它独立开发停点。

不新增共享trait签名，必要内部worker-control严格协议由Windows在windows_owner_host024/**本线实现并版本化，原loaded357不热补/不新image接旧pipe；Mac不需要先造一套平行Owner接口。测试需真实同线程ID、活动lease !Send/!Sync编译否例、registry仅control/Send原对象、断连/expire/重复action/panic失联分类、失败后原ranges仍真实冲突、无新窗口/无原件变化。不得为易通过只在DATA模拟锁或把controller夹具当marenben原现场。

### C. PSS名称flag仍未知：增加文档化类型查询原型，未知不跳过

0x8实测仍24unknown，现路线尚未通过，不继续扩大PSS flags或重跑已过同组。允许test/windows024-native/**单broker自有夹具新增**仅对象类型**的NtQueryObject(ObjectTypeInformation)路线，动态从系统Ntdll解析有文档函数，只读取PUBLIC_OBJECT_TYPE_INFORMATION.TypeName；不使用私有layout、reserved字段、ObjectNameInformation、远程内存或全机NtQuerySystemInformation。微软文档说明API可能变化，故缺导出/不支持/不合法buffer必须Unknown，不设未文档fallback。

类型输入只来自原PSS单broker完整handle快照。对每个原handle先DuplicateHandle到本观察进程：dwDesiredAccess=0、inherit=false、options=0，禁止SAME_ACCESS/CLOSE_SOURCE。保留原broker实际身份和存活及scope，所有复制原handle均不关闭；本地零权限副本只是内核同对象引用，不得调用数据读写、等待消费、信号/锁、socket操作或变更对象。受系统不支持复制的类型/无法复制/句柄变代/类型读取失败均Unknown，不把ERROR_INVALID_HANDLE或非Job特定错误当“不是Job”。源进程及对象原最后句柄仍保，观察前后原对象行为无变更；新副本close影响/回收必须自有fixture实测，不冒OS绝对无影响。

NtQueryObject参数固定ObjectTypeInformation、必要有限buffer（先固定4KiB，若原返回所需合法长度则最多64KiB、最多一次增大，原5秒更短parent不变）；只接受成功NTSTATUS与实际ReturnLength及头结构边界有效，UNICODE_STRING长度偶数/非0/≤512bytes、MaximumLength/Buffer全部在返回缓冲范围，无溢出/越界/悬指针，TypeName严格UTF-16。不得解reserved或把对象地址/handle整数作为授权。按类型识别Job后，再从同原broker原handle只QUERY复制并CompareObjectHandles与零权限副本一致，完整两遍集合/同内核对象去重/原Node成员/counter/未Resume组合前后仍按r7全文核。所有非Job也必须有实际类型，不吞24unknown或仅对已知两Job查询。

本增量只授权文档化类型查询的自有原型。先类型覆盖/未知句柄/高风险对象零访问/句柄复用/重复Job/历史退出/权限/超时/内存边界严格否例，再依现R0只读规则核当前现场；不满足继续保留CurrentJobUnknown，不以旧creation计数补签。任何更宽权限、绕same-image、外部终止Node/broker或当前原件归还仍不在此批准范围。

依据：[NtQueryObject / PUBLIC_OBJECT_TYPE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject)明确唯一可用类型字段TypeName；[DuplicateHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle)的零desired/options与禁止关闭源；这些文档支持API语义，不代真实普通令牌fixture/当前现场证据。三请求只解除对应接口未定；R1仍未批准，所有旧失败/拒绝、原件/当前进程保持，不W2/双SID/发布。


## 2026-11-17增量接收：零权限复制raw50与旧客户端边界

Windows9fdd4c618／docs f6f8677a的object-type原型已形成新事实：两次完整91句柄遍历中24个原句柄的零权限DuplicateHandle返回ERROR_NOT_SUPPORTED(50)，未取得本地副本；其余67已取得实际类型并不证明剩余不是Job。因此C路线未通过完整覆盖，既有“成功原型后只读R0”的条件未满足，不调用当前r2 JobQUERY，不重复同前提0x8或零权限轮次。尚无经源码/API证据确认的下一有界路线，不给无假设试跑、不扩大访问权限/SAME_ACCESS或跳未知。

另核实际loaded357的failure_control.rs第283—361行：request_cleanup固定Operation::ExplicitRecovery并要求facts.forced=true；服务器存在Status分支不等于已加载客户端有只读入口。不可执行旧cleanup来探状态，不用新image绕原same-image准入，不把构造相似RPC或修改旧二进制当已有能力。这里只作精确源码判读，未执行当前现场动作；证据见windows-r7-contract-receive-r12/loaded-client-route-review.json。

这项当前r2的恢复证明仍为Unknown，未形成R1Ready，更未授权forced/原件归还。原r2归档/44项不受影响证据复用，旧场景清理授权不转用。future-host原worker、独立v2附件、最终同claim/全树/8/字节/children/锁释放证据按A/B及既有r7允许集连续推进；不因旧现场只读路线停点而停止其它已授权实现。新路线或具体外部修复方案需先列原事实、覆盖与未证前提，再决定对应边界；不把部分原语通过写为生产Stop/Closed或整版通过。


## 2026-11-18增量决定：首启与恢复共用原worker

依据 cecd17bda58ebbccc0bd0bb0a0a017a5df1ab5ba 的 wl2-r7-initial-start-worker-request-20261118-r1/interface-request.json，在原B节基础上将worker的所有权起点前移。capture_held 与 capture_existing_with_predecessors 在原调用栈持有活动的 !Send/!Sync lease，失败后才启动恢复worker无法接回已释放的范围。

- 原宿主以唯一Owner/slot登记一个持久worker，在第一次native acquisition之前完成绑定。每个成功取得的原范围和已有副作用前缀即时归入同线程持有状态。初次Start返回首错或过期时，先撤销Start写权限，再发布Required及精确阶段。活动lease和部分取得句柄留在原线程。不得因提前返回、接收者断连或析构而释放尚有恢复义务的范围。worker失联或panic报告Unknown，保留最早可知事实并拒绝正常操作。
- 请求端只持控制许可。Start与显式恢复走同一worker，分别用入口签发并绑定action/slot的原绝对期限。排队计入期限，迟到请求不得开始写。原Start 30秒，Stop 10秒，Restart总30秒且Stop子窗最多10秒。持有活动lease或必须由Owner读取/修改的阶段经同一worker执行，其余共享编排留原入口。Required、Unknown或恢复处理中，普通Start/Stop/Restart立即拒绝。Status独立读受绑定的实际投影，在5秒内返回阶段及可证事实，记录失联、竞争或不可读时为Unknown，无须等worker锁。回执保留首阶段、实际结果和系统原始错误码。
- 同一线程区分原Start能力、只读保全能力、显式恢复能力。原Start到期永久失效。只读凭证核对原slot/worker/thread、原session/action、同源与同范围成员身份及锁。新恢复能力由原worker在一次显式请求中验证同一失败前缀、当前独立注册、原对象身份和实际连续持锁后生成，绑定原失败action、唯一新action和单一10秒或更短父限。能力只许可合同已证明的静止、原件返还、工作副本处置、读回与精确释放阶段。缺任一原范围或持有事实即Required。不得向原lease传新deadline重开Start权限，不clone/adopt，不释放再取得，不出现仅持NLK写入窗口。恢复过期维持原位保全，异常持锁处置仍需独立决定。恢复结论不签正常Stop、Closed或后继许可。
- Windows独占 windows_owner_host024/** 及r7已授权的Windows原件/生命周期/协调实现和 test/windows024-native/**。Windows基线 0816047f30f667e56bea4167ec012f6e12bfcbdd，共享main 9daf5599151b65437f94ab2cadb1988060ee38c3。四trait/CONTRACT_VERSION4及共享持久格式不变。新的共享依赖先交最小接口申请。原Owner绑定的V2/Pending可独立继续。
- 先用隔离自有夹具验证首个/部分/全范围取得后的真实失败、原Start超时后拒绝写、同原线程续用原句柄、外部冲突负控、显式恢复成功与过期、断连/重放/重复派发/worker失联、Status 5秒及普通请求拒绝。继续严格核验原件身份、安全、字节、children、锁释放、原预算和准确回执。旧r2现场、日常配置和官方包维持原状，本合同只放行上述新实现和隔离验证。
