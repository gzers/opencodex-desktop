# W-L1接收与W-L2四项共享边界 r2

revision: 2；2026-11-14。归IMP-34 P24-W1/04；当前执行写集取代r1，历史结果不改写。共享代码输入：12746c005cb2621d2f1f44d12c0af501613fef1a。模块 modules::runtime::windows_lifecycle024_contract 是可编译的接口声明，四个trait没有默认成功实现；不是四项原生能力已经实现。现有Windows生产消费者的Required/Unknown继续生效，直到实际适配及定向证据闭合。Windows接令后在下面写集内连续开发、批量修复、定向核验、及时push，无须逐处回Mac。

## 本次接收与事实边界

贡献代码3a9b71f7c20a404e136652e584c8f29de5efb226，证据13ea948b214ceffb3c3b06d8b1b6ae97b55c8ee5。7笔代码按净差集成至6b3bd173f，tree与贡献相同；17文件，1675增加/20删除。docs净差382文件仅包含本轮新证据与windows-dev-handoff；380个清单项、3,046,760字节逐一重算匹配。source06/07逐文件比对Git，infrastructure/mod.rs与account-w1d.ps1的捕获CRLF单列，不把换行差异算成未经说明的源码修改。可离线重放的接收入口在.adg/evidence/0.2.24/windows-wl1-receive-r2/verify_receive.py；没有重新运行Windows。

| 项目 | 接收结论 | 限定 |
| --- | --- | --- |
| 生产Start | 定向通过，24965ms | 实际source06/360183c59，非最终候选 |
| Status | 848/855/849ms通过 | 同PID64904/port56328，3个错误绑定反例拒绝 |
| 最后原Job | 334代次最终已退出，未强清 | 是失败后读回，不补签原10秒Stop通过 |
| Stop | 未确认，10005ms | 原始sc1060/schtasks1保留，不能改写为0 |
| 原件恢复 | 未完整 | config.toml字节30恢复，文件ID/安全描述符不同；两个其它合成原件相同 |
| Restart/第二Start | 保持拒绝 | 不丢旧owner、失败原件代次或重置预算 |
| 管理器重启连续性/supervisor | 未完成 | 进程本地owner与HTTP均不替代 |
| Auto冲突重试/取消/更新/H切换 | 尚未验收 | 后继受完整关闭资格限制 |
| 普通用户MSI | 未验收 | 本轮普通生命周期不替代先前管理员MSI或普通MSI |
| 清理 | 部分完成 | 5账户/原Job已处理；5 Profile 3,211,038,283字节与Public319,071,336字节保留；首个Profile部分删除限制保留 |
| W2/双SID/发布 | 未执行 | 当前不放行 |

source07/72b9f74d6仅改变动作回执：Started保留显式Stop，Start/Restart仍被恢复未决阻止。它的MSVC构建/动作gate结果独立接收，不把source06运行结果冒成source07全链已运行。旧out01..04/debug/release预算失败、首次E0786内存构建失败及取消记录、旧SID1040、清理自动审批拒绝均保留，不追溯改成成功。正式实施进度仍只在IMP-34维护。

## 四项接口共同约束

共享模块固定ContractVersion=1、官方包摘要与精确任务/服务名。NativeSession::check_current(deadline)绑定原创建者保留的句柄/Job、TokenUser SID、Profile、D/H/I、当前source/current摘要及session generation。ProcessGeneration只是身份数据，绝不是权限。所有trait的关联proof在具体Windows适配层字段私有、不可Clone/serde；唯一生产实现由原owner生成，不接受请求端自填bool、PID、序列化回执或任意路径生成proof。

具体Windows实现统一Owner与各关联proof类型：ClientOriginals::NativeNormalExit必须就是DiagnosticRoles::NormalExit；两者SupervisorAbsent必须就是SupervisorAbsence::Absent。禁止分别定义能互相冒用的替代类型。共享层不引入第二套HMAC/来源选择/恢复事实源，Windows实现四边界后再将当前生产入口接到它们，Mac适配保持。

ActionWindow只在公开入口产生一次：Start30s、Status5s、Stop10s、Restart30s且Stop子10s从同一动作起点算。IPC往返、捕获/刷盘、验证/还原、子进程监控全部计入原deadline，不能每个trait/重连/重试再加预算。只有明确新的用户动作才有新窗口；历史超时不重新解释为成功。

### 1. Windows原件持久恢复

接口顺序为ClientOriginals::capture(owner,window)→OriginalSession；record_ready(original,owner,ServiceObservation,deadline)→ReadySession；pre_stop(ready,owner,stop_deadline)→PreparedRestore；complete_stop(prepared,owner,NativeNormalExit,SupervisorAbsent,StopAccepted,stop_deadline)→ClosedEligibility；authorize_successor(closed,owner,window)。最后一项消费一次性关闭资格，重验同SID/H/I/source与代次；失败/取消的Start要走有证据的原件关闭路径才能重试，不能drop owner后换端口。

持久记录格式标识windows-client-originals-v1。D/official024/windows/<session UUID>/仅保存私有引用和不可覆盖阶段记录，不放公开凭据。固定字段：schema、sessionId、sequence、previousDigest、binding{SID、Profile/D/H/I的规范路径及原生目录ID、source/current摘要、owner PID/FILETIME}、phase、originalsManifestDigest、前阶段记录ID/摘要。phase只允许Captured、Ready、PreparedRestore、OfficialTeardown、Restored、Closed或Aborted；每份create-only，刷盘后再发布当前引用，引用CAS包含原文件ID/安全描述符/摘要。中断时任意前缀均可判定当前未决，缺段/重复/未知schema/回退引用一律拒绝，不恢复到默认空状态。

originals清单逐项标明相对路径、Absent或File/Directory、卷/文件ID、完整可读取安全描述符的原生字节摘要、内容摘要/长度、目录枚举及保管对象ID。内容和凭据留在SID私有的原生保管对象内；日志只记摘要。实际安全描述符不可完整读取则拒绝，不能只记录DACL后声称全部权限相同。现有两稳定锁是唯一有限例外，必须沿原约定逐个证明已释放/安全权限/无残留；sqlite-journal、备份、额外cache不能自动归入例外。

**恢复原件身份采用同卷原对象保管，不能复制重建后声称文件ID恢复。** 启动前在准确客户端父目录下建立SID私有custody，保留原文件对象并给官方提供内容等价的工作副本；必须在原件首次移动之前持久记录意图，移动前后逐项核对目录/文件ID与权限。D与Profile跨卷时，D只存引用；原对象仍在原卷custody，禁止跨卷copy/delete原件。不得通过放宽分享/修改旧原件权限让移动强行成功。文件被占用或reparse/多硬链/ADS等无法完整保管时明确拒绝Start，保留可恢复事务。

范围包含原声明Codex配置/认证/路由/目录、Grok根及实际生成物，不能只恢复截图三文件。优先复用现有Mac原件范围与有界树规则作为需求，Win32读取/移动/安全由Windows实现，绝不把POSIX身份硬套。原根缺席需捕获父目录身份及缺席，停后只退休本次有完整代次/内容证明的新对象；原根存在则保留原根身份/权限和未触及对象。额外未知文件、外部编辑、备份/引用漂移均保持Required/保留现场，不覆盖或删掉来凑读回一致。两根全量原像及声明受控副作用都在复核范围；不能把未列出的变化默默排除。

complete_stop在官方teardown之前保留PreStop，之后先得到正常退出/端点释放/supervisor/当前source的原生证明，才移动准确原对象回原路径并读回内容、ID、安全和存在性；官方先还回相同字节但ID不同仍需恢复本次保管的原对象。每一步有意图与完成记录，失败阶段不能因Rust值被消费而丢原持久引用。Closed只有全部读回匹配才发布，HTTP200或本轮最终Job空不能发布Closed。

### 2. 跨管理器owner连续性

OwnerContinuity::authenticate_manager(connection,deadline)→AttachedManager；with_owner(attached,deadline,operation)只在原owner内执行；detach_manager不终止服务。不从manager进程转移/重建Job权威。选用同一受控安装的ocxd.exe专用模式--internal-runtime-owner024作为长寿命原创建者，GUI/IPC runner只作为已认证客户端；不是系统服务、计划任务或需要管理员的新常驻服务。

broker先于官方launcher建立并持有完整W0原创建句柄/Job/完成历史及custody/phase句柄，普通GUI退出、IPC断开和runner Drop后继续存活；禁止KILL_ON_JOB_CLOSE和为观测结束而强杀。启动broker继承的外层Job若会杀后代，必须先证实其生命周期不破坏连续性，否则拒绝并单列；不绕过外部Job策略。受控故障清理仍独立报告forced，不能记普通Stop。

私有named pipe路径在D的受保护rendezvous记录内发现，名称只含SID/安装身份/随机实例摘要，不含秘密。使用本机首实例/拒绝远程、受保护DACL只允许当前TokenUser及明列必要系统主体，拒绝其它普通SID；句柄不继承，任何必要DuplicateHandle限制目标为已核验的manager/broker原生进程句柄及最小权限，不接受网络传来的handle数值。原owner留在broker，不需要向GUI复制Job来继续控制。

rendezvous格式windows-owner-rendezvous-v1，只含schema、instanceId、pipeName、broker PID/FILETIME、SID、D/H/I/source摘要、generation；它只定位，不授予权限。实际连接双方用GetNamedPipeClient/ServerProcessId及保留进程句柄核SID/FILETIME/准确镜像current，fresh随机256bit challenge→原生绑定响应（同一连接，单次使用），再签发内存AttachedManager。启动bootstrap通过私有继承句柄/stdin单次传递，秘密不入argv/日志/持久文件。

IPC帧为小端u32长度+严格UTF-8 JSON，上限64KiB；deny未知字段/版本/命令、重复键、非法整数、NUL、截断、超长和尾随帧。请求固定{schema:1,requestId,instanceId,managerGeneration,challenge,operation}，operation只允许status/start/stop/restart/detach/explicit-recovery；动作路径/程序/SID/token不能由请求更改。证据响应只含结果及非秘密身份摘要，不把native proof序列化成下一动作许可证。挑战/请求防重放表在broker保留，manager切换原子推进代次并拒绝旧连接继续写；并发动作仍走原同H互斥。

动作请求另有必需budget字段{actionId,startedQpc,deadlineQpc,stopDeadlineQpc,qpcFrequency}，全部整数以十进制字符串传送，避免JSON double损失；使用同机QueryPerformanceCounter/QueryPerformanceFrequency共同时间轴，不传Instant内存或墙钟。manager从公开动作开始取原窗口，broker核同一启动环境/频率、起点不在未来、截止不超过原动作上限及认证连接所绑定actionId，取原截止与本地允许截止较小值；过期/错频/重用动作拒绝。换算Instant剩余时间向下取整，接收/握手/重连不增加预算；服务端不得从收到请求时重新开始30/10秒。Status同样带原5秒截止。detach/握手有有界连接窗口，不得替代动作窗口。

同SID新GUI在原broker存在时可fresh认证接续；伪旧FILETIME/错误source/H/SID、重放/旧manager写入全部拒绝。broker缺席时即使原服务仍活或JSON自称可恢复，也只能Unknown/显式受限恢复，不能新建broker“接管”已有PID。旧owner未关闭时换I/H/source或开始后继拒绝。持久原件恢复在broker意外死亡后保留记录，必须独立重验实际事实，不能反序列化NativeNormalExit。

### 3. Supervisor缺席

SupervisorAbsence::observe_absent(owner,stop_deadline)→Absent，recheck_absent(owner,&Absent,stop_deadline)在最后恢复/Closed边界重新询问。Absent绑定SID/Profile/D/H/I/source、原owner代次及前后观测时点；不跨动作缓存。

从固定官方2.99原包确认：任务精确名opencodex-proxy（Scheduler根命名空间），SCM精确服务opencodex-proxy-native（机器级，不能说是用户级对象）；H下opencodex-service-task.xml、opencodex-service.cmd、opencodex-service-launcher.vbs、winsw/opencodex-proxy-native.xml与.exe，以及原官方service-state声明的同Profile/H关系。原包源码路径/摘要/核对行见official-source-review.json。Windows桌面监督来自实际父子链；本次未发现一个可凭空追加的HKCU Run键，不能发明新的命名空间去凑缺席。

使用原生只读SCM OpenSCManager/OpenService及QueryService*、Task Scheduler COM精确GetTask/实例查询，保留原HRESULT/Win32码。只有准确not-found且查询权限/命名空间已确认才算注册缺席；ERROR_ACCESS_DENIED、服务停止待删、Task返回1、空输出/超时绝不算缺席。原Job内wrapper/sentinel由原代次历史覆盖；它们缺席与注册缺席、精确H staged文件的状态相互独立。staged残留/外部定义/不属于本I/H的注册为Conflict或Unknown，本合同不授权停止/删除该注册、扫全机或跳过不可读对象。不能照抄官方walkWinswChain在exe/xml缺席时把SCM unknown变absent的宽松分支。

本次源码复核另确认运行PID文件准确为H/ocx.pid（src/service/desktop-supervision.mjs:22），不是opencodex.pid。当前W-L1消费者尚有后者检查，因其整体仍拒绝完整Stop未造成一次假放行；W-L2须在允许的windows_official_lifecycle024.rs内修为同原官方来源的准确目标，并加“opencodex.pid缺席而ocx.pid仍在”拒绝用例。runtime-port.json/ocx.pid缺席与supervisor/原Job证明仍各自独立，不能只改文件名便认为Stop闭合。

### 4. 诊断子进程角色与退出语义

DiagnosticRoles::classify(owner,retained_child,deadline)→ProvenRole；complete_family(owner,&SupervisorAbsent,stop_deadline)→NormalExit。枚举Launcher/Service/ScmRegistrationQuery/SchedulerRegistrationQuery/OtherRequired/Unknown，保留ChildExit::Exited(raw u32)/StillRunning/Unreadable。原始角色/命令/exit逐项落证据，不只有最后摘要。

角色证明至少包含原Job与父代次、PID/FILETIME/SID、原镜像文件ID/路径、实际原生argv与启动阶段、同current官方脚本/源码call-site来源摘要；在短命进程退出前有界捕获，不可读/已丢失历史则Unknown。不以名称/系统路径/一个退出码授予诊断角色；conhost/icacls/PowerShell也不一概豁免。精确只读命令语法仅SCM query opencodex-proxy-native及Scheduler /query /tn opencodex-proxy /xml（大小写/可证明等价路径规范化）；改变目标/增加动词/组合shell/额外参数即Unknown/OtherRequired。源码存在另一只读命令时先交具体来源/实际argv增量，不自动扩大规则。

**产品语义说明：本次新增的是按实际角色判断完成，原所有后代一概exit0无法描述官方的缺席查询。** Launcher/Service/OtherRequired必须正常退出0；所有原代次必须被完整记账、结束、端点释放且无force。已被完整证明的两个只读注册查询可以记为CompletedDiagnostic{rawExit}，但非零不变0、不单独判成功/缺席：还须同动作前后独立原生查询证明精确注册缺席、无访问拒绝/超时/错误目标，才可以不把该查询的“未找到”当服务异常退出。特别是schtasks1既可能不存在也可能拒绝访问，不能建码表豁免。Unknown role、证据断链、缺失argv或非查询的非零仍否决NormalExit。原W-L1失败不受此新角色合同追溯放行。Windows先完成角色原语/反例，再接此生产语义；无法取得完整证明就如实未通过。

## W-L2允许写集与执行顺序

当前窗口直到下一次Git回交由Windows单写，Mac不并行写以下范围。可在范围内持续试验，不需要每个失败再问；任何范围外修改先给最小原因与diff。共享trait签名被冻结，编译必须的小修可给最小补丁，不能静默改变其含义。

1. infrastructure/windows_official_runtime024/**、windows_official_lifecycle024.rs/同名_tests.rs；可新增infrastructure/windows_client_originals024/**、windows_owner_host024/**、windows_supervisor024.rs/同名_tests.rs、windows_diagnostic_roles024.rs/同名_tests.rs及test/windows024-native/**。该目录内实现上述持久格式，不能另开第二journal事实源。
2. src/bin/ocxd.rs仅Windows internal-owner模式分派与其测试；可新增src/bin/windows_owner024.rs，保留原terminal-native命令与stdio/退出码；src/lib.rs/state.rs/infrastructure/mod.rs仅Windows注册/持有入口。现有GUI/IPC公开合同不扩命令、权限或跨平台语义；相关原生调用改在内部broker传输完成。
3. commands/windows_lifecycle024.rs、commands/mod.rs、runtime_recovery_v2_owner.rs/runtime_recovery_v2_sources.rs；infrastructure/process_runner.rs/status_source.rs/attested_status.rs；modules/process/mod.rs仅Windows消费者/默认fixture兼容注入，不能改Mac或全局扫描兜底。可新增对应Windows定向tests文件。官方HTTP/当前来源验证算法不复制/放宽，消费现有types。
4. modules/runtime/windows_lifecycle024_contract.rs原则只消费，Windows必须的类型适配在新adapter完成；official_lifecycle024.rs/official_runtime024_transport.rs如发现真共享缺陷回交最小补丁，不自行改HMAC/就绪/预算。Cargo.toml仅确需的Windows API feature最小装配（如Pipes/Services），不升级依赖、不改release参数；Cargo.lock仅真实feature解析所需变化。构建workflow/安装器/全部Vue/模型同步网络/其它恢复协议均不在本写集。
5. docs只贡献新不可覆盖windows-dev/<轮次>/**和windows-dev-handoff.md；正式IMP/TEAM本对话回写。代码codex/024-windows-native从精确main承接，文档docs/024-windows-dev从本次docs/governance-main承接，及时分别push。不要整分支merge混入历史证据或代码。

顺序：先MSVC正式编译及四边界原语/负例（可并行独立实现）；再原owner-host/原件捕获接生产；然后一名隔离普通账户上同source验证Start/Status/Stop、后继与管理器更替。每轮前写本轮要证伪的假设，互不依赖项一次采集完整结果，依赖缺失标blocked；拿完整表批量修复再下一轮，不重跑无新假设的旧失败或未受影响通过项。source冻结前只开发定向，不W2/双SID/安装器全量，不发布。

## 固定输入及定向验证合同

代码为12746c005cb2621d2f1f44d12c0af501613fef1a；docs为本合同所在docs/governance-main实际提交（发给Windows的提示词给真实SHA）。官方2.99 tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；Windowsbundle 14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7/274033721字节；descriptor42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a。重用已保留原包/cache并重算摘要；每个新EXE记真实source/tree、工具链、profile、字节/sha，不拿旧EXE当新source。release定向沿opt-level=s/lto=true/codegen1，debug失败单列，测试panic配置不冒成产品abort。

| 组 | 正例/必须读回 | 必须拒绝/原始失败 |
| --- | --- | --- |
| O 原件 | 原根存在/缺席；D/Profile异卷；配置/凭据/路由/目录字节+fileID+SD；两锁逐个释放例外；各阶段断电/中断可判未决 | 同字节换ID或SD、篡改custody/ref、外部编辑、新未知文件、重解析/多链/ADS不完整、原件占用、旧代次/未知锁owner；失败不发布Closed |
| H owner | 同SID原broker在GUI退出后保留；新GUI fresh握手继续Status/Stop；原句柄/Job/phase不变，旧连接失权 | 错SID/FILETIME/source/H、pipe伪装、nonce/request重放、超长/重复键/截断、broker缺席但service活、并发manager；不从JSON/PID重建 |
| S supervisor | 原生准确任务与SCM缺席、H staged根核对；前后同源，当前Job/listener无残留 | task存在/外H、机器服务存在/待删、访问拒绝/超时、只HTTP/Job空/exit1/1060，均不能证明缺席 |
| D roles | 原真实短命query逐个来源/argv/父代次/Job保留；raw1060/1不改，独立缺席佐证后记CompletedDiagnostic | 同名不同目标、/delete等变更动词、伪parent/丢历史/错source、service非零、强清、仍活或不可读；必须否决 |
| P 生产 | 实际public→broker→原包；Start≤30/Status≤5/Stop完整≤10；同pid/port/source；原件闭合后Restart总≤30且Stop子≤10/第二Start | Stop任何未决后均无后继；Auto重选必须先闭合失败代次；显式端口冲突不换号；取消/更新/H切换不能丢owner/custody |

回交一次完整状态表、首行错误/阶段/原始trace、原件前/ready/prestop/post与phase引用、各原进程及rawexit/角色证明、原生supervisor HRESULT/Win32、GUI/broker原代次与交接、source/EXE摘要、普通账户/Job/Profile清理清单。诊断轮accepted:false；单项通过不宣布整组/整版通过。现有W-L1 Start/Status可复用为不受影响证据，但新broker路径改变后必须在新source重新验证受影响调用，不能拼成跨source总通过。

测试只使用任务自有I/D/H/Profile、合成客户端，不触碰日常PATH/配置，不运行真实模型/业务同步。先归档并核验失败现场，再清理准确自有进程/账户/树；tasklist不当无残留证明，不杀共享系统进程、不改主机策略、不提升普通验收token。旧自动审批拒绝不重试/绕过；新一轮证据目录不覆盖旧轮。未完清理如实保留数量/字节/原因，不能把普通MSI或旧失败清理悄悄标通过。
