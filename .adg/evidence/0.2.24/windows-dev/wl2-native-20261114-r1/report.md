# W-L2 原生前提检查点（未完成生产适配）

2026-11-14，TEAM revision12 / IMP-34 revision138 / windows-lifecycle-shared-r2。accepted=false、allAccepted=false、productionExecuted=false。本轮在普通原令牌的完整安全描述符读取及外层Job连续性前提受阻；这是环境/合同读权限边界回交，不把四项共享trait重新列成“缺接口”。四项Windows具体trait实现、持久custody、原创建者broker与生产四边界组合仍未完成。

## 来源与已推送代码

- main：12746c005cb2621d2f1f44d12c0af501613fef1a
- docs/governance-main：34474df17bd4ee9c809e94640f8bab3b6b16754e
- Windows最终源码：35e7336fcc64bcf3cab26ca8fa9688daba95a1f0，tree 73f7ba8b084f9eb5fa1a91ec734d1997223fcc23。
- bb8d3d66b1fbfacb317d89b82b820e1e248bbb7f：原Job完成口捕获真实argv/父代次，缺失明确Unknown；没有按进程名或退出码签发角色。
- 2e1f0932b37bcc64bab2488950e1c1c130725d2a：准确H/ocx.pid缺席门用于Stop和stopped读回；同名目录/记录残留也拒绝。
- 35e7336fcc64bcf3cab26ca8fa9688daba95a1f0：仅测试宿主装配O/H/S前提与隔离/构建/清理入口；没有ocxd内部broker分派或生产四trait的假默认成功。

17文件净差均在r2写集；最终Rust逐字节与primitive-build02/native-build02/product-build03各26份Rust输入对齐。Git换行差异另记final-source-manifest.json。后补的清理控制脚本非编译源码，实际隔离账户运行用的PS副本完整保存在public-raw；清理第二次脚本SHA见cleanup02-hypothesis.json。旧未跟踪windows-schema.json保持。

## 首错、修复和当前阻断

首个构建环境调用误把MSVC Tools指向msvc-w1c而非msvc-w1c/msvc，cargo未运行；修正后承接源正式bins编译exit0/221062ms。第一轮两个编译入口均E0061/101：monitor.drain一处漏传commands表；完整源码/trace留在primitive-build01和native-build01，修正后第二轮通过。最终MSVC产品bins再次exit0/160990ms（debug，14条warning保留）。未运行严格Clippy或发行候选构建，未把debug/test的panic设置当产品release abort。

当前首个未解决错误在普通SID1046、elevated=false的Open-original-READ_CONTROL-ACCESS_SYSTEM_SECURITY阶段：Win32 1314，客户端没有所需的特权。请求access=0x81020000，后续完整SECURITY_INFORMATION=0x1ff；并未在失败后退回只读DACL。whoami /priv退出0，仅列SeChangeNotifyPrivilege、SeIncreaseWorkingSetPrivilege，未列SeSecurityPrivilege；原始OEM输出、解码视图与编码摘要均保留。可独立读取的owner/group/DACL及字节、volume/fileID在探针前后相同，完整SD从未被声称已读取。

另一个独立前提是IsProcessInJob(current,NULL)为true。当前未获得全部外层Job寿命/归属证明；没有把它说成已确认KILL_ON_JOB_CLOSE，也没有使用BREAKAWAY、新计划任务、服务或修改宿主策略来脱离它。H/codec/QPC正反例不等于native pipe认证或跨管理器owner连续性。

隔离用例先一次写出所有独立事实，再在完整SD断言失败；原始test exit101、控制器exit1，不是产品Stop错误或超时。实际官方启动次数0、原Job创建0、custody/phase/ClosedEligibility创建0。没有用已有W-L1 Start/Status成功拼成本轮新broker通过。

需要集成线明确或提供的最小前提详shared-minimum-request.json：其一是原普通令牌可满足完整SD读权限的既有隔离环境，或者明确冻结可读安全范围与不可读SACL等部分的严格语义；本线不得自行缩为DACL。其二是没有继承外层Job的普通启动上下文，或可证明原broker跨管理器存活的原生外层Job生命周期证据。未请求改动四trait签名、预算或原始退出语义，未在日常主机授予特权/修改策略/提升普通验收token。

## 实际测试范围

产品lib定向6项通过、1项隔离用例先ignored、1866项无关测试过滤；该隔离用例随后在新普通账户上明确执行并失败。独立native新增用例1项通过，捕获2/2真实短命查询argv、同原Job父PID/FILETIME、每项原始退出码sc1060/schtasks1，并拒绝向无官方来源fixture授予诊断正常退出例外。没有执行已有全量组件、旧r5、MSI/CLI、W2或双SID。

O/H/S模块当前以cfg(all(windows,test))装配，提供native读取前提和协议/预算反例，不是生产ClientOriginals/OwnerContinuity/SupervisorAbsence实现。D新增采集位于真实native Job模块，但DiagnosticRoles官方call-site/阶段绑定、完整NormalExit语义及错目标/delete/伪parent矩阵尚未实现。旧生产all-zero与Unknown/Required保护保留。精确SCM查询仅本机机器级opencodex-proxy-native，Task Scheduler COM仅根下opencodex-proxy；普通账户原始结果分别1060和0x80070002。它们只是独立命名空间观察，没有原owner/source绑定、stop前后重查或私有Absent能力；staged观察的根是合成测试目录，不冒充生产H。

## 完整状态表

状态是各项实际边界；passed-prerequisite-only与passed-refusal-only都不构成该组生产验收。

| 项目 | 状态 | 结果/限定 |
| --- | --- | --- |
| baseline-fetch | passed | main12746c005/docs34474df17 merged without reset; TEAMr12/IMP138/shared-r2 consumed |
| fixed-official-inputs | passed | bundle, descriptor and inner tgz digests independently recomputed; no original bytes modified |
| MSVC-baseline-bins | passed | exit0 221062ms; debug official product bins, not release candidate |
| MSVC-final-bins | passed | exit0 160990ms; actual final Rust bytes match final source; 14 warnings retained |
| compile01-E0061 | failed-fixed | monitor drain second argument missing; both first raw101 failures retained, corrected once before build02 |
| MSVC-library-primitives | passed | 6 passed, one isolated prerequisite test initially ignored; 1866 unrelated tests filtered |
| D-native-query-capture | passed-prerequisite-only | 2/2 real short-lived original-Job query argv and parent PID/FILETIME retained; raw sc1060/schtasks1 remain unchanged; no official source authority |
| D-unknown-source-no-normal-exit-exception | passed-refusal-only | synthetic launcher cannot mint official role; unrelated controller PID command evidence refused |
| D-official-source-role-trait | not-implemented | DiagnosticRoles/ProvenRole/NormalExit and negative target/delete/fake-parent matrix not completed; no whitelist/default successful implementation |
| pid-ocx-residual-negative | passed | legacy opencodex.pid absent while ocx.pid present refuses; directory same-name and runtime record residual refuse; same helper used Stop and stopped readback |
| O-partial-read-is-not-complete | passed-refusal-only | native1314 remains error; no owner/group/DACL fallback or original mutation |
| O-isolated-complete-security | blocked-environment-contract | ordinary SID1046/elevatedfalse ACCESS_SYSTEM_SECURITY open raw1314; SeSecurityPrivilege absent; no capture or official launch |
| O-original-readback | passed-unchanged-prerequisite-only | synthetic34bytes same fileID281474977958307/volume247287385 and partial SD digest; complete SD never claimed |
| O-same-volume-custody-journal-restore | not-implemented-blocked | no windows-client-originals-v1 journal/custody/phase/current reference or recovery proof created |
| O-existing-absent-cross-volume-lock-interruption-matrix | blocked-not-run | all original custody cases remain unverified; prerequisite blocked, not marked passed |
| H-strict-frame-negatives | passed-prerequisite-only | strict typed frame schema/duplicate keys/unknown op and fields/NUL/UTF8/truncated/oversize/trailing/numeric-shape refusal |
| H-QPC-original-budget | passed-prerequisite-only | same QPC frequency; original30/5/10 and Restart stop-sub10; wrong frequency/future/expired/overflow rejected; no IPC authentication claim |
| H-isolated-outer-Job | blocked-environment-observation | IsProcessInJob(current,NULL) true; outer lifecycle not proven. No KILL flag inferred, no breakaway or policy bypass |
| H-private-pipe-native-peer-replay-manager-generation | not-implemented-blocked | codec is not authenticated connection; private SID pipe/fresh peer process authentication/replay tables and atomic manager transfer not built |
| H-ocxd-owner-host-production | not-implemented-blocked | no --internal-runtime-owner024 dispatch; ocxd/GUI integration unchanged; original process-local owner still used |
| S-native-exact-SCM | passed-prerequisite-only | local manager CONNECT success; exact machine-level opencodex-proxy-native OpenService query raw1060; no native owner/source-bound Absent capability |
| S-native-exact-Scheduler | passed-prerequisite-only | fresh COM local connect/root GetTask exact opencodex-proxy raw0x80070002; no enumeration/control/delete |
| S-error-code-negatives | passed-refusal-only | access denied/pending-delete/error1/timeout and other HRESULT never classified absent; injected code classification tests only, no host registration created |
| S-staged-independent | passed-prerequisite-only | present synthetic staged cmd remains Present even when registrations absent; ordinary synthetic home exact five targets separately observed; not production H |
| S-SupervisorAbsence-trait-source-stage-recheck | not-implemented-blocked | no source/host generation-bound Absent minted; service-state/full H/parent-wrapper/post-stop recheck unverified |
| P-single-SID-Start-Status-Stop | blocked-not-run | productionExecutedfalse; no official launcher/HTTP/listener/retained product Job started after prerequisite failures |
| P-restoration-Restart-second-Start | blocked-not-run | ClosedEligibility not minted; no successor or Auto retry; cannot combine old W-L1 positive proof into new owner-host acceptance |
| P-manager-handoff | blocked-not-run | broker not implemented; GUI exit/new manager/fake pipe/source/H/SID/replay/stale generation matrix not run |
| P-cancel-update-H-switch-auto-conflict | blocked-not-run | no recovery qualification, no real source switches or successor actions |
| ordinary-prerequisite-test-result | failed-expected-gate | ignored test explicitly run under one fresh ordinary SID; raw test101/controller1; all independent facts written before first assertion on completeSecurityReadable |
| daily-PATH-input-desktop-ACL | passed | PATH hashes unchanged; input and temporary desktop ACL restored; no daily configuration or official package writes |
| cleanup-new-account | passed | SID1046 account removed; no official Job created/forced cleanup, no new task or MSI registration |
| cleanup-new-Profile | partial | first native call87 fixed actual NULL wrapper; second success0 removed Profile registration, exact directory remains inaccessible; count/bytes unknown, no full Profile preservation claim |
| cleanup-new-Public | blocked-automatic-review | 26files92915595bytes remain; exact recursive deletion rejected before execution; no retry or alternate mechanism |
| historical-MSI-CLI-cleanup | reused-unchanged-unfinished | old passed CLI/admin MSI not rerun; ordinary MSI and old failed Profile/Public/schema cleanup gaps remain; no old rejection retry |
| W2-dualSID-release-business | not-run | no W2/dualSID/frozen candidate/publish/model or business sync requests; one new ordinary test account only |

## 制品与输入

以下都是实际MSVC debug制品，未签名、未冻结、未发布；四项trait未闭合，不是可接受候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75404800 | 582b3afb06230e87d0dff2ed5e1af226abbaa6278469453ec583fc71aeab19fd |
| ocxd.exe | 4623872 | ab8299ca9a751058d06bded7784426d224767e2af728a8c08b9ad299ffb1bf50 |
| ocx.exe | 306176 | 5462cbb5188ee7339a31d5d7f7b63cb9124c306100a363a5d28d0b0afde3ca8f |
| 隔离原语测试宿主 | 92872192 | 988b248dd078f00ea4dd1bcfcf6904890973f6472f63da1000ff7ce55c7d3663 |

官方固定bundle274033721字节、descriptor与内嵌官方2.99 tgz已重新核SHA，见fixed-inputs.json，原包未解包修改。没有模型请求或真实业务同步。release原opt-level=s/lto=true/codegen1未改；前提失败后没有空跑生产或发行构建。

## 清理与证据限制

SID1046测试账户已移除；没有官方Job、测试计划任务或MSI登记。输入ACL、临时桌面ACL已恢复；日常User/Machine PATH摘要相同，未执行registry PATH写入。原件“未变化”仅指原语探针前/后的读回，不是Profile删除后的全量恢复证明。

清理首轮把PowerShell null经managed string传为empty，DeleteProfileW返回87，原receipt保留；C#内部传真正NULL后同API返回true/0、Profile登记消失，但C:/Users/ocxw1d6802b2e1c目录仍存在且枚举access denied，数量/字节未知，不将34字节合成文件大小冒充整个残留目录大小。没有强卸载hive、改ACL、替代递归删除或共享系统进程操作。原始公共日志/原件字节和可读身份摘要已归档；synthetic-original.reconstructed.bin明确从编译fixture字面量重建并与前后原生摘要匹配，不冒称控制器读到了不可读Profile，也不声称整个Profile完整保留。Native DeleteProfile已部分改变Profile现场，这一限制保留。

新Public根C:/Users/Public/ocx-wl2-20261114-r1，26文件92915595字节。25份非EXE逐字节归档核验，EXE原件精确保留于workspace且hash一致。精确递归删除被自动审批blocked by policy拒绝，整个命令未执行；未纠正后重试、未换API或其它机制绕过。提交的路径比较表达式含NUL转反斜线写法，此事实单列在public-delete-rejected.json，不假称检查已在shell内执行。

旧W-L1的5失败Profile、Public319071336字节、旧MSI SID1040/Profile/Public、其它旧失败目录/schema/空PATH清理缺口均未重复删除/改写。普通用户MSI仍未验收。新target构建缓存与EXE原件留用。cleanupComplete=false、allAccepted=false。新证据根为不可覆盖轮次；manifest逐项校验原始字节，.gitattributes禁止Git换行转换。
