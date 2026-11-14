# W-L1生产调用点接线与严格未完项

2026-11-14；Asia/Shanghai。精确消费main35f41ecc0e700a3a77d121cc2dccf7258c327bc1、docs/governance-main832df5a85c9da241568fc94d0048aaaa2f92b094，TEAM revision11、IMP-34 revision137、windows-production-lifecycle-interface-r1。代码merge检查点529562728，文档merge检查点f4490c14a；未改官方2.99、日常配置/机器策略、旧lifetime或发布门。新代码首批219ad3e94bef547a91307fee00f37e72ae09f656已推送，最终源码/制品/result见本目录最终清单。

## 先有假设，再收独立项

hypothesis.json在新增实现前记录：实际命令入口能消费唯一LaunchPlan/ActionWindow与typedHTTP，将原创建句柄/Job和服务成员来源/listener绑定；HTTP成功不能替代所有原成员正常退出、客户端原件恢复或跨管理器连续性。后继hypothesis03根据原Job total286/active3及conhost真实事实补充完成端口捕获假设，不改变原断言/预算。每轮保留不同源码和原输出，不把debug旧失败改成release成功。

新增windows_official_lifecycle024适配器，实际process_action_with_runner的Windows官方2.99分支进入commands/windows_lifecycle024，复用原port::prepare_start、LaunchPlan和ActionWindow。原创建Node挂起时关联私有匿名Job，保留Profile/H/程序/输出文件及Node/Bun/script句柄，校准来源后resume；不是先运行再补Job。GUI/IPC/status用同一进程内registry，registry锁只保护短时查找，owner动作使用try_lock，不在Status等待持有者；新的runtime Arc/H不能搁置旧同程序根owner后又开另一owner。失败/取消/runner Drop不强杀或丢owner。

Ready/Status复用typed HMAC/ready/record/source，再核原Job中服务PID/FILETIME/SID/当前精确Bun文件身份、loopback listener及全部活成员来源。包内node_modules/bun/bin/bun.exe与独立dependencies Bun分别为已准入镜像成员，不按名称收养；系统conhost仅Win32返回的精确System32文件可用组件硬链接，仍核卷/文件ID，原runtime/package单链接拒绝保持。Status失败显式Unknown/noport，light路径不走旧端口缓存。客户端恢复状态在缺持久协议时保持Unknown/Required，不凭owner存在造Active。

Stop通过同typedHTTP原连接和两个RetainedService发送边界；其后仍要求原Job全部成员正常exit0、总创建数与保留句柄历史一致、原监听端点缺席、runtime record/pid缺席、末尾来源/预算复核。Job完成端口在分配根之前关联，捕获worker在resume前就绪；NEW_PROCESS消息本身不是正常退出证明，捕获失败/消息缺失/代次不符或accounting不等仍拒绝。测试显式force_owned_cleanup有独立类型/事件，只能用于失败后精确测试Job清理，不变成正常Stop。

**本批不具备完整Windows客户端原件持久恢复、supervisor缺席和跨管理器保留owner协议。** native退出若完成也只保留单独事实，不返回完整LifecycleResult::Stopped，不发后继Restart/第二Start许可；回执区分服务停止与恢复Required。旧SystemServiceOwnerProofProvider对准确官方024/来源未知保持Unknown，不能回退全机进程扫描或旧desktop_lifetime.Stop。其它平台及维护来源语义保留。

## 原始失败和修正

| 轮次/项 | 原结果 | 处置与限制 |
| --- | --- | --- |
| debug out01 | Start30218ms未确认；Status654/662/652ms为Unknown；Stop/Restart/第二Start未完成 | 原Job3个成员仍活，最后只做单独记账的测试失败强清；首个panic在Start断言，原stdout/stderr/events留存 |
| debug out02 | Start30205ms未确认，诊断stage=job-member-image；Status798/787/750ms为Unknown | 包内Bun已校准，Job含System32/conhost，total286/active3，轮询历史不完整；门禁已收紧，客户端恢复Required，Start/Restart均禁用 |
| 原客户端读回 | .codex/config.toml从30字节变238字节；文件ID/安全描述符也可变化，另有官方生成文件 | 第二轮完整声明.codex/.grok树记录存在性、SHA、卷/ID/security摘要；没有手动覆盖回原样冒充恢复 |
| 原生监控定向首错 | conhost系统组件硬链接被普通runtime单链接检查拒绝 | 只对精确Win32 System32/conhost允许系统硬链接；runtime仍单链接，原失败保留 |
| 监控修正编译首错 | 一次临时String静态生命周期E0716、一次反斜杠字面量E0765 | 原日志均保留；改为拥有型io::Error和无歧义规范化，再定向通过 |
| 监控受影响3项 | 挂起绑定/原非零退出、根退子活拒绝完整退出、双栈监听原代次 | monitor-final-directed.json全部exit0；只因本批监控变更回归，不跑旧W0全量/旧r5 |
| 产品bins首错 | ocxd读取rlib E0786、Windows1455页面文件太小 | 不改页面文件/策略；退出101原样保留，串行-j1重试通过；不会把它当产品逻辑错误 |
| release构建01/02 | 明确取消旧源码构建，exit-1 | 已确认PID/创建时间/命令，原因是新实现使旧源失效及并行内存峰值；非观测超时重启；不用于实际预算结论 |
| release最终定向 | 以release-test-build03及后继release运行原件为准 | 原opt-level=s/lto/codegen-units参数，test harness panic语义与生产abort分开记；不冒release候选 |

debug-independent-audit.json独立检查两轮原事件：没有Started/Stopped成功、没有后继Start、Status无旧端口、客户端已变化及失败强清独立事实。失败响应第1轮canStart等过于宽松已作为真实缺陷在第2轮收紧，原回执不改写。W-L1脚本不把HOME的secret/token值或原文件正文写入公开证据，原运行记录/配置只归档摘要。原客户端内容均为隔离合成输入，无真实用户认证或上游请求。

## 固定输入与来源

官方tgz输入固定SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；Windows bundle274033721字节，SHA14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7；descriptor SHA42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a。复用旧Public只读输入，隔离普通账户在本轮新I供应完整当前镜像；供应/读全树在动作Start预算外，真实启动的完整准入仍在原ActionWindow内。原2.99文件不改，不做MSI安装或CLI参数/只读复跑。

source01/02/03/04和executed*逐字记录各阶段源码，不将未完成/取消构建当最终制品；source04/05/06分别在对应release构建前固定；实际最后原包动作使用source06，后继source07只改动作回执权限。Git blob与Windows执行字节另列，CRLF/LF不冒充逐字相同。实际产品bins与测试EXE的SHA/bytes/工具链/profile见最终制品清单；任何旧MSI-only EXE都没有用于本批新生产调用验证。

## 缺共享接口的最小需求

shared-minimum-request.json给出三项具体需求与当前main源码摘要，未写第二个持久Schema：

1. Windows客户端原件：共享typed capture/ready/pre-stop/stopped/restore-readback许可；绑定实际SID、D/H/I/current、原文件句柄/卷ID/security或原缺席、两锁有限例外、来源/预算、持久代次与后继资格。原Mac Active/StartupSession/Index依赖macos_client_originals，不能在Windows复制POSIX语义。真实测试已证明Start修改Codex内容/权限。
2. 跨管理器owner：指定长寿命原创建者、SID私有IPC与句柄dup协议，绑定manager PID/FILETIME、H/source/代次和原Job；原owner存活到显式正常Stop/失败清理。bin/ocxd与新跨进程协议不在W-L1自动写集，无原保留句柄时保持Unknown，不从记录/PID重建权力。
3. 官方supervisor缺席：typed当前用户且精确H/I/current命名空间只读证明，继承原deadline，不用全机进程扫描或修改系统服务。StopAccepted只有官方响应pid/port/digest，原Job空不证明外部supervisor缺席。

这些是完整W-L1/整版未完，不因当前进程内接线编译或native局部通过而删除。完整Restart/第二Start及管理器退出/更新/H切换连续性均不能宣称通过。共享算法/HMAC安全门和原30/5/10/Restart30 Stop子10没有放宽。

## 清理与旧缺口

本轮新的普通测试SID1041/1042均已在精确Job清理证明后移除账户，输入ACL与临时桌面ACL恢复；第1轮DeleteProfile返回true且原生登记消失但目录残留，第2轮按失败保留规则保留Profile登记/原目录，后续只删账户对象不改失败场景。只读清单：out01目录10628文件624037698字节，out02目录12064文件646769406字节；不是ProfileRemoved。原强清事件不当成普通Stop成功，原现场不覆盖。后继release账户/制品及精确清理结果另见最终result。

旧MSI普通用户验收未完成；管理员MSI通过证据已由集成线接收，不重跑。旧SID1040 Loaded/error32/System/Registry占用、Public根及其它自动审批拒绝均不重试、不强卸hive、不杀系统进程、不改策略/重启系统或服务。固定候选前未跑W2/双SID、真实业务同步或模型请求，未发布。

## release首轮后的原生增量

source04固定release构建03通过（9m43s），out03真实Start30028ms仍未确认，Status682/645/664ms均Unknown/noport，最终stage=service-image，所有后继动作被恢复门禁拒绝；原source04/EXE/完整失败日志留存。该失败不当成debug性能推测。

随后单独原生反例先证明24个短命同源子进程可由完成端口保留所有代次与正常退出；再用原Job内精确System32/cmd.exe复现系统组件硬链接被普通包文件的单链接规则误拒绝。只读进程观察现对Win32返回系统目录下无重定向的辅助映像保留卷/文件ID；它们不因此被准入为官方服务，实际活跃服务仍须是当前精确Bun，包内Node/Bun/script单链接准入保持。系统辅助反例还发现已保留成员可能在活跃PID表读后退出，现仅返回WouldBlock在原deadline内重读，绝不替换句柄/收养同PID。

最终五个受影响原生用例exit0（native-final-directed.json）：系统辅助镜像/退出、短命同源历史、挂起身份/非零退出、根退子活、双栈监听；先前辅助反例marker首错、硬链接拒绝、退出竞态失败原样留存。新增源码af334cba1ec965ea50e3c0039621ce9dd49d3213已推送，source05在后继release构建04前逐字冻结；不把这批组件通过外推为新原包生产通过。

out03测试失败Job独立强清完成，账户SID1043按精确回执删除，Profile登记Loaded=false及失败目录保留。没有重跑MSI或旧CLI/旧W0全量。最终release04实测和最终产品bins结果另见本目录最终result/制品清单，旧失败不会回写成成功。

## 完整历史显露的共享退出角色要求

release out04原Job计数286与保留后代285逐项闭合（post-start-native-diagnostic error=null），不再是漏采历史。release04-independent-audit.json列出原sc.exe exit1060和schtasks.exe exit1等非零诊断进程，全部源自该原Job，未跳过或改写。按本合同非零成员拒绝的现规则，即使服务主体后续退出也不能凭这些记录宣告全成员正常Stop。共享最小需求补充：若需按官方诊断进程与服务主体角色解释退出码，应由共同合同冻结来源绑定的typed角色/退出语义和supervisor缺席证明；本线没有自行加exit白名单。

活成员路径比较另修Win32路径分隔符等价（正斜杠版本路径与系统反斜杠镜像路径），只消除字面拼写差异，不代替PID/FILETIME/SID/卷ID/文件ID/HMAC/来源门。新增等价与不同路径反例随release构建05验证。首次内联测试路径被转义成CR导致rustfmt拒绝，编译尚未开始；改char::from(92)构造后格式通过，工具错误单列path-test-format-first-error.json，没有借此改超时或真实动作断言。

## 最终回交状态（以此节及status-matrix.json为准）

最终代码分支codex/024-windows-native HEAD **3a9b71f7c20a404e136652e584c8f29de5efb226**，所有批次已推送。实际release原包动作代码 **360183c5906b941b19b48ed05cf532ea318d944f/source06.json**；release-test-build05通过，测试EXE **44592640字节，SHA256 b293c6041e74d89b24aa1468746d8a422f7901f037b46dcbf1597e9238a3fcd3**。

| 最终独立项 | 结果 | 原证据 |
| --- | --- | --- |
| 全产品MSVC bins | 通过 | product-build06 exit0，debug。最终Desktop75384832字节，SHA25640dfbfb3bc629e98fd67ec5dc00df7b9fd710f0ddae342935c987b47ec669b56；ocxd/ocx另见product-artifacts06，不是release候选。 |
| 仓库原release参数测试目标 | 通过 | release-test-build05 exit0；原opt-level=s/lto/codegen1，测试harness panic与产品abort语义分开记录。 |
| 真实生产Start | 通过 | out05 **24965ms**，同实际命令入口/runner/原创建Job和typed readiness；record PID64904，loopback port56328，官方2.99。 |
| 高频Status | 通过 | out05 **848/855/849ms**，同PID/端口/当前2.99与原owner；非旧缓存。 |
| PID/port/record反例 | 3项拒绝通过 | 实际Ready owner的native-binding-negatives；不接受错原生绑定。 |
| 受影响原生监控 | 5项通过 | native-final-directed；组件事实不替代完整Stop。 |
| 预算/缺owner/路径等价 | 定向通过 | release owner测试2项；不重置Restart30/Stop子10，无原owner不adopt。 |
| 普通Stop | **未通过** | out05 **10005ms**未在原10秒确认，result=None；不能将最终Job空或客户端字节恢复当原窗口成功。 |
| 最后Job读回 | 完整捕获并最终清空 | total334，retained descendants333，active0，最后一轮未强清；含sc1060/schtasks1，不能声称全0正常退出。 |
| 客户端恢复 | **未完成** | .codex/config.toml字节30已恢复，但文件ID/security摘要变；auth.json/.grok/settings.json原字节/ID/security保持；新增官方文件/锁仍在原完整快照，不自定豁免。 |
| Restart/第二Start | 拒绝后继 | 461/240ms，旧owner与恢复未闭合时不丢owner、不继续启动。不是正向Restart通过。 |
| 取消/更新/H切换/Auto冲突重选 | 未完整验收 | hook/来源门/旧owner保管已接；真实取消/更新矩阵未跑，失败代次关闭与原件恢复协议缺失时不重选/后继Start。 |
| 管理器重启连续性、supervisor缺席 | **缺共享接口** | 进程内保留不冒跨管理器host/私有IPC；StopAccepted及Job空不生造外部supervisor缺席。 |
| CLI/MSI/普通用户MSI | 原状态保持 | 未重跑旧已过；普通用户MSI仍未验，管理员MSI接收不替代。 |
| W2/双SID/发布 | 未执行 | 五轮是顺序单SID开发用例，不是双SID矩阵。 |
| allAccepted | **false** | 源码接入不等于完整W-L1或整版收口。 |

原out05确认Started的回执误将显式Stop也禁用，这是真实回执缺陷。后继72b9f74d6只改两个命令结果投影文件：确认Started允许显式Stop，恢复未确认仍禁Start/Restart并标clientRecoveryRequired；action-gate.json针对该语义exit0。原生/HMAC/来源/动作预算源码与source06逐字相同，复用out05原包证据，不重复真实运行。product-build06在source07下全bins再编译exit0；最后3a9b71f7c仅更新native README。没有把out05旧回执改写成已经修正，也未宣称最后产品EXE经过GUI交互实机验收。

独立final-independent-audit.json核对正向Start/3次Status、3个拒绝、原Job历史、Stop/后继失败与无强清事实。release05-client-readback.json单列原声明合成文件读回；没有持久原件引用/恢复许可，不能仅因config字节恢复给ClosedEligibility。共享最小需求包括typed Windows原件捕获/还原、长寿命owner/私有IPC、精确supervisor证明，以及官方诊断子进程的来源绑定角色/退出语义；未经集成冻结，不新增journal或退出白名单。

最终清理：5个新普通账户全部移除，5份原Job最终清空，所有临时桌面ACL及输入ACL恢复。前4轮明确测试失败强清，与正常Stop分立；最后out05未强清。5个失败Profile目录保留，共58877文件 **3211038283字节**；其中首轮旧控制器已尝试DeleteProfile并移除原生登记，部分原Profile内容未逐字归档，只有当时原快照/摘要及残留树，此限制不能写成完整失败目录原貌；第2..5轮原生登记Loaded=false、目录现场保持。

新Public根 **129文件319071336字节** 非EXE原始字节已归档/核验，五份EXE分别记录SHA/bytes。归档后精确删除被自动审批blocked by policy拒绝，命令未执行、未重试、未换机制绕过；失败Profile与所有旧拒绝根均排除在该删除动作外。复用w1c-product新增release缓存及原大资源保留，未伪报全机无残留，也未重算旧根整体字节。没有改日常配置/PATH/机器策略/官方2.99，未发模型或真实业务同步请求、未发布。
