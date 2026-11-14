# W-L2接收：完整安全描述符与broker启动环境 r3

revision: 3；2026-11-14。归IMP-34 P24-W1/04，TEAM revision13。代码输入 **main dbf974097d067e75c4de68f9fc681af8ab447b09**。本合同增量承接[四边界r2](windows-lifecycle-shared-r2.md)，四trait签名、原预算/未知拒绝/原件恢复要求不变。本文只澄清共享最小请求的SD读取范围、环境与执行依赖，不把普通令牌/策略改变当修复，不将部分原语通过升级为四项生产适配通过。

## 接收结果

贡献代码35e7336fcc64bcf3cab26ca8fa9688daba95a1f0、证据70ebd2a67（完整SHA见同批integrated-source.json）；三笔新增贡献按净差接至301cb7e43，tree与贡献73f7ba8b084f9eb5fa1a91ec734d1997223fcc23一致。净17文件1137增加/21删除。新证据373文件仅本轮目录及handoff；371份payload2,715,343字节全部匹配。final-source-manifest的17项Git blob和实际字节（明确CRLF等价）逐项核对，primitive-build02/native-build02/product-build03各26份Rust实际编译输入逐一对源，不把后补控制脚本冒作实际执行版本。离线可重放入口：.adg/evidence/0.2.24/windows-wl2-receive-r3/verify_receive.py。

| 项目 | 已接事实 | 当前限制 |
| --- | --- | --- |
| MSVC bins | 基线/最终debug exit0；最终160990ms | 14 warnings保留，非签名release候选；Windows未跑严格Clippy |
| lib定向原语 | 6项通过 | 只是O/H/S测试装配，不是生产trait |
| 真实短命argv/parent | native1项通过，2/2查询保留raw1060/1 | fixture无官方call-site，仍Unknown，尚无DiagnosticRoles生产资格 |
| ocx.pid门 | 新精确文件名和残留/同名目录反例通过 | 完整Stop仍未闭合 |
| SD | 原普通SID1046/elevated=false返回1314 | 无SeSecurityPrivilege；只读owner/group/DACL不能替代完整SD |
| 原件探针 | 字节/ID/部分SD前后相同 | 未发生custody/phase/Closed，不是完整恢复证明 |
| 外层Job | IsProcessInJob(current,NULL)=true | 未证明Job寿命，不推断KILL_ON_CLOSE为true或false |
| SCM/Scheduler | 原生精确1060/0x80070002 | 无owner/source/阶段绑定Absent能力，不冒生产supervisor通过 |
| 生产四项适配 | 未完成；productionExecuted=false | 原包启动0，不能拼用W-L1成功冒本轮新broker成功 |
| 清理 | 新账户1046已移除、PATH不变/ACL复原 | Profile登记移除但目录不可读、数量未知；Public26文件92,915,595字节仍在，原自动审批拒绝不绕过 |

首轮E0061/101两份编译失败原件、隔离原语test101/controller1与清理87后真NULL成功/目录access denied均保留。接受代码表示接收实现/证据，allAccepted=false。集成后另修两处rustfmt纯格式，不变原生语义；新共享常量/注释不是四项Windows实现。

## 决定一：完整安全描述符的范围与权限

本轮**不缩为owner/group/DACL，也不把不可读的SACL视为不存在**。完整恢复的既有要求涉及同一文件对象的安全描述符全部内容，而非普通用户碰巧能读到的子集。共享常量固定在modules::runtime::windows_lifecycle024_contract，具体native读取在Windows允许集实现。

| 内容 | 读取合同 | 不可读/不确定的结果 |
| --- | --- | --- |
| Owner、primary group、DACL | READ_CONTROL；保留SID、ACL原生结构、ACE顺序/标志和null/空/缺席差别 | 原件捕获拒绝，不能修旧ACL后重试 |
| 完整SACL（含审计ACE） | ACCESS_SYSTEM_SECURITY；原调用令牌在打开句柄时实际具有所需权限 | 拒绝完整捕获；owner、字节或DACL相同不补签 |
| mandatory integrity label、resource attributes、CAP scope、process trust label、access filter等OS所支持的安全信息 | 属于完整描述符读回范围；保留操作系统返回的全部原生ACE/标志，不丢弃未识别字段 | 无法读取/无法证明覆盖的组件为Unknown；不能按“普通软件无需审计”省略 |
| SD revision/control与继承/保护、DACL/SACL present/defaulted等控制语义 | 从完整SD提取并保存，区分未请求、请求后证实缺席、empty/null/存在 | 解析失败、长度漂移或覆盖不明拒绝；只存SDDL/摘要不足以恢复原件 |

句柄安全查询访问掩码为 **READ_CONTROL | ACCESS_SYSTEM_SECURITY = 0x01020000**（FULL_DESCRIPTOR_QUERY_ACCESS），不是WRITE_DAC/WRITE_OWNER。原件内容读取所需FILE_READ_DATA/目录枚举等单独最小请求，不以GENERIC_READ代替SACL权限。路径、父目录pin、fileID、非reparse/单硬链/ADS及同卷保管前提沿r2，不因为取得SD便放行其它原件问题。

完整查询采用保留文件/目录句柄上的 **GetSecurityInfo(SE_FILE_OBJECT, BACKUP_SECURITY_INFORMATION=0x00010000)** 取得ppSecurityDescriptor；其余可选输出指针可为NULL，使用返回SD解析所需成员，LocalFree返回缓冲。BACKUP_SECURITY_INFORMATION按OS合同请求“all parts of the security descriptor”，不是给本程序启用备份特权。r2探针0x1ff是九个命名投影之和，不再凭这个数字本身声称对未来/扩展信息完整覆盖。禁止在API不支持或失败时退回0x7并签发原件许可。

完整结构须验证IsValidSecurityDescriptor、GetSecurityDescriptorLength及有界长度（承接256KiB）、修订/控制信息；同原句柄重复读取及原文件/父目录ID前后核对。必要时转换为自相对结构进行持久化，不能memcmp含绝对指针的内存地址；转换不得过滤ACE或重排语义。原对象同卷保管继续是恢复文件ID的依据，不依靠SetSecurityInfo重建新文件。重复读相等仅为本窗口事实，不宣称阻止所有外部安全写；阶段间变化按原CAS/冲突规则拒绝。

独立读取owner/group/DACL（0x7）的结果仅为诊断投影，日志明确coverage=partial、fullSecurityProven=false。label/attribute/scope个别读取可能在READ_CONTROL下成功，不意味着完整SACL已读。API返回成功但只申请子集仍是Partial；某ACE未出现在子集内不等于对象没有它。不得将安全描述符SHA、SID属主、路径owner或same-object alone当完整覆盖证明。

### 具体拒绝语义

- open或GetSecurityInfo返回5/1314：记录准确API/原始码/请求mask/原Token身份，分类security-unreadable，**在首次移动原件、创建官方工作副本或官方Start前拒绝capture**；无OriginalSession/ReadySession/ClosedEligibility。不要把预检失败说成Stop超时。
- 50/87或OS不支持该完整查询、缓冲异常、未知ACE覆盖、读取时变化：security-coverage-unknown，拒绝同上；不得按“该机没有此项”归Absent。未请求或不可读取与经完整查询确认不存在严格分开。
- 原件已经pending时的任何读失败：保留原custody/意图与阶段记录，恢复Required/Unknown，禁止普通Stopped/Restart/第二Start/Auto重选；不删除custody或回退默认值。只允许既有显式恢复入口按真实证明处理，不能因新请求失败丢旧owner。
- 有限两锁例外只处理原合同释放/原身份/权限证明，不豁免其未知安全信息，不扩展到journal/cache。

依据：[SECURITY_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-information)、[GetSecurityInfo](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo)。引用摘录/日期/下载摘要见security-reference-review.json。微软文档明确READ_CONTROL与ACCESS_SYSTEM_SECURITY的区别；取得SACL通常要求启用SeSecurityPrivilege，而本任务禁止更改验收令牌。因此**当前SID1046所代表的受测普通令牌没有满足完整SD前提；目前没有已证实可提供完整SD正例的普通环境**。不能靠反复新建相同权限账户、提升运行、AdjustTokenPrivileges、模拟特权服务、备份语义读取、grant privilege/ACL或修改日常策略将其标通过。

这是明确保留的产品可用性限制：在这类普通令牌下，要求完整原件恢复的生产Start仍应被拒绝。若未来要求其同时无特权可用，须另行采用“有界可读安全范围”的产品合同，并明确完整SACL不作证明；本轮没有默默采用该弱化，也不把剩余缺口声称已解决。

## 决定二：broker连续存活的可接受启动环境

选择**现有隔离普通账户的真实交互登录会话中，从该会话自己的桌面Shell/Explorer直接启动**（控制台或RDP均可，单SID即可）。不从当前被外层Job包裹的AI runner、CI、测试控制器的CreateProcess/Process.Start链直接派生。Explorer来源只是候选路径，**真正准入依据是原生no-outer-Job读回**，不是“RDP/Explorer通常无Job”的假设。

本轮最小可接受条件：

1. bootstrap与broker均为同一原普通TokenUser、elevated=false、真实Profile和同一交互Session；记录TokenElevation/TokenElevationType、TokenSessionId、PID/FILETIME与实际current镜像。不得以管理员令牌、runas /netonly、CreateProcessAsUser提升、借SYSTEM服务或新计划任务代替。既有临时账户的建立/清理控制权限不等于业务进程可用提升token。
2. bootstrap在创建broker前IsProcessInJob(GetCurrentProcess(),NULL)成功且FALSE；通过原始CreateProcessW以CREATE_SUSPENDED启动同current ocxd内部owner模式，保留原process/thread句柄，在resume前对这个broker句柄IsProcessInJob(...,NULL)成功且FALSE，再核SID/session/FILETIME/source/H。失败/true/查询拒绝即environment-unproven，未resume官方launcher/未开始客户端变更。
3. broker始终不被加入测试控制器/manager的Job；它创建并保留自己的私有子Job，只将官方launcher/受控后代放入该Job。子Job不启用KILL_ON_JOB_CLOSE，也不启用BREAKAWAY_OK/SILENT_BREAKAWAY_OK；父进程退出不触发应用自写的Drop杀服务。CREATE_NEW_PROCESS_GROUP/DETACHED_PROCESS、另开终端、Start-Process -WindowStyle Hidden均不是逃离外层Job的证明。
4. 若bootstrap或新broker为true，本轮拒绝，不凭查询到的某一个inner Job flags无KILL位推断全部enclosing Jobs安全。确需支持有Job启动环境时，须另交所有外层原对象/句柄、限制和控制者寿命证明；当前没有该证明，不能自动breakaway/更改外部Job或重启控制器来规避。
5. 正常GUI关闭/runner释放/IPC断开及同SID新GUI接管时，broker原进程句柄Wait仍未signaled、PID/FILETIME/SID/source/H相同；外部独立观测者保留创建时取得的broker handle（不是按旧PID重开），读到原子managerGeneration更替并验证旧连接失权。至少覆盖GUI正常退出及仅终止该原manager的失败试验，后者在事先核验精确创建句柄后执行，不强杀broker/官方服务；故障动作独立记账。新GUI仍做r2私有pipe/fresh peer/QPC验证，观察者JSON不能代替认证。
6. 在broker创建后、首次launcher前、每次manager attach和动作前后重验其no-outer-Job/原代次与native owner。如被外部添加Job或原owner消失，转Unknown不允许后继；不宣称轮询覆盖了未观察间隙。完整连续性依靠原创建句柄持续持有及broker自身不受manager的寿命控制，实测在声明动作窗口内证明它存活，而不是承诺无限期/注销/重启/系统杀进程后仍存活。

该环境目前是已选定、待Windows原生证实的执行入口，**不是已取得的主机可用证明**。Windows对话可先只读确认是否已有该普通交互会话；没有会话/桌面操作入口时报告缺口，请用户在Windows侧登录该专用账户并从桌面启动一次定向bootstrap，不自行改主机策略/注册任务/提升令牌。不要重复跑已知带Job的相同控制器。真实会话尚未就绪时，仍可开发四trait和codec/安全拒绝/原语组件，只有依赖该环境的H实测阻止。

当前code中outer_job_precondition只证明调用者，不替代“挂起broker原句柄noJob”或manager退出后的存活检查；两处必须实现。即使broker环境通过也不能掩盖SD1314，O和H独立门各自报告。

## 后续执行合同与写集

r2逐文件写集继续；本轮额外明确允许Windows修改共享windows_lifecycle024_contract.rs中本次三个常量的Windows API使用适配建议，但不能自行降低FULL范围/四trait签名/预算；真实必要修正先交最小依据。原O/H/S模块从cfg(test)切到cfg(windows)只能在具体实现就绪时做，private proof不可serde/clone、无默认成功实现；不把测试辅助模块注册后叫“生产已实现”。不改Mac/其它模型同步网络/安装器/workflow/旧lifetime/日常配置。原包2.99、bundle和descriptor及摘要沿r2固定，不重搬大包。

执行顺序：先实现可独立的native O错误/覆盖分类与持久状态架构、H broker/私有认证/noJob预检、S typed原生缺席前后重查、D source/argv/parent/Job角色；**O正例环境受阻不等于其它三项停止开发**。捕获完整SD失败的O生产代码应真实拒绝；可以在测试桩上检查阶段转换和中断恢复，但明确不是完整原生原件正例，不伪造已通过。

| 组 | 本批定向核验 | 不满足时 |
| --- | --- | --- |
| O权限/覆盖 | 原令牌精确access/backup查询；0x7成功不作完整、SACL不可读/1314/5、API50/87/长度/覆盖未知；capture前零client mutation，pending时保留Required/custody | 保留原失败，O正例/生产Start blocked，不重建相同账户反复1314；无新环境不重复旧前提轮 |
| H环境/连续性 | 无Job bootstrap＋挂起broker原句柄、普通SID/session；原broker跨旧manager退出和新manager认证仍活；错SID/FILETIME/source/H/pipe、replay/stale generation拒绝 | 有外层Job或无真实桌面只阻H实测；新broker虽起但认证/存活未验不算连续性通过 |
| S/D实现 | 复用已有namespace/argv原语，绑定原owner/source/阶段、短命原历史和独立Absent；原raw1060/1不改 | 无call-site/错目标/变更动词/缺历史仍拒绝NormalExit；fixture不冒官方角色 |
| P受影响生产链 | 只有O完整SD及H环境、四trait都就绪后才单SID原包Start≤30/Status≤5/Stop完整≤10；Restart30含Stop子10、第二Start/Auto前代关闭 | 任一必需证明不满足就不启动真实官方服务；旧W-L1正例不填本轮空白 |

每轮先写可证伪假设、编译＋所有独立定向项一次完整表，再批量修复；已通过且不受本批影响的CLI/MSI/native原语不重跑。debug原输出/失败/原始exit与release预算测试分开；最终预算仍原release参数。无新假设不再开相同失败轮，不以增加超时或把环境未就绪当pass收口。固定候选前不W2/双SID/安装器全量，不发布，无真实模型/业务同步。

完整回交须分别给implementation状态和test结果：真实四trait实现文件/调用点、source/tree/实际执行EXE摘要、SD请求/覆盖/API/rawError与是否发生首次变更、broker创建handle/noJob/原token/session/存活和managerGeneration转移、S/D原生proof/原exit、完整状态表/首错/阻止依赖。保留accepted:false直到整组必需项满足；不得因本合同已冻结就把原受阻前提消除。

## 现场与清理

本次新Profile原生DeleteProfile成功只表示登记移除，并不能证明目录/全部字节已清理；目录不可读明确unknown数量/字节，不复刻已丢原件为“完整原现场”。后继失败Profile清理前必须在原普通SID尚可访问时完成授权范围内实际归档/摘要核验；归档失败则保留账户/Profile供安全恢复，不能先删除账户再猜内容。归档不收集日常凭据。旧不可读Profile与Public自动审批拒绝保留，不通过改ACL/所有权、强卸载hive/共享进程操作或另一删除工具绕过。新临时原件/进程/账户清理分别记事实，用户/机器PATH和ACL前后核验；普通MSI未验及全部既有清理缺口不在此被关闭。
