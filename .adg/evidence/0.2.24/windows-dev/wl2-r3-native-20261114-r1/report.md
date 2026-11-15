# W-L2 / r3 Windows 增量回交（仍未完成）

TEAM revision13 / IMP-34 revision139 / windows-lifecycle-prerequisites-r3，增量承接r2。main dbf974097d067e75c4de68f9fc681af8ab447b09；docs a6d9184b851521194477ddfeecd988f40d513e5b。代码 3d028e82d731ceb2d307b0a954a93db467ca3f39，tree 70e0beb3ab9073008e76d51b3346b7bb377ac77e。allAccepted=false、productionExecuted=false、complete=false。

本轮没有重复普通账户1314/外层Job控制器前提失败，没有再建测试账户。独立H/S/D适配继续落码；它们的编译与原语通过不等于四边界或产品通过。最终MSVC debug bins exit0/137535ms；lib增量8项、native实际pipe4项通过。原30/5/10/Restart30及Stop子10、raw退出码/Unknown拒绝不变；未改Mac、官方2.99或日常配置/策略，未W2/双SID/发布。

## 本轮实现与实际调用点

| 边界 | 实现进展 | 真实限制 |
| --- | --- | --- |
| O / NativeSession | 原Owner增加当前token、Profile/D/H/I原生目录pin、原Job/root与session/source摘要；完整SD改为0x01020000和GetSecurityInfo(BACKUP=0x10000)，原生结构/长度/control及自相对字节检查，保留准确API/rawCode。execute_async在官方create_suspended前调用preflight及durable capture拒绝门。 | ClientOriginals trait尚未实现。同卷custody/工作副本/完整内容与ADS/current引用CAS/有限锁/恢复读回/ClosedEligibility尚未落齐。require_durable_capture保持明确拒绝，不因SD读取成功就跳过恢复协议。 |
| O / pending架构 | phase.rs用canonical schema的create-only PrivateRecord与原句柄识别连续前缀；各阶段中断、缺段/重放/摘要/seq/schema/binding漂移测试通过，文本Closed只产生PendingOnly。 | 这是架构和合成测试，尚无生产client-originals事务/持久current-ref；不从记录反序列化原生恢复资格。 |
| H / native组件 | 实际SID/System私有首实例local-only pipe、OS client/server PID及保留FILETIME/token/image句柄；bootstrap与挂起原broker普通token/noJob门、准确stdio继承列表；fresh 256bit challenge、原安装根镜像限制、QPC原始起点及deadline、单次动作、nonce/request/action表、manager原子代次/旧连接拒绝。 | OwnerContinuity具体类型在cfg(test)组合编译；原生组件在Windows后端。ocxd内部长驻分派、rendezvous/bootstrap、GUI/IPC生产传输仍未接，未启动实际broker或做真实manager替换。 |
| S | SupervisorAbsence真实trait类型注册Windows，Absent私有非Clone/serde；绑定原Owner/source与同一stop_deadline，准确SCM/Scheduler、H staged文件、H与Profile默认state镜像、原family完整退出，最终恢复边界fresh重查。 | 没有新source-bound Absent实机正例；当前Stop尚未调用它。任何state文件存在/不可读仍保守Unknown，未把声明关系解析成absence。 |
| D | DiagnosticRoles真实trait类型注册Windows，查询候选需原Job子代次、live捕获argv/父代次/时间/启动阶段、System32准确镜像、固定包/源码call-site摘要及原父Bun/CLI；rawexit保持。 | 完整service/OtherRequired角色分类及生产Stop组合未闭合；非查询/丢历史/未知仍拒绝NormalExit；没有实际官方ProvenRole/NormalExit生成。本轮syntax反例不能冒充官方因果正例。 |

以上界限机器版见implementation-status.json，下一步仍由Windows在既定写集继续，不要求Mac重新冻结已经明确的四trait或弱化完整SD。

## 原始结果与首错

1. 承接时两处mod文件发生纯格式/排序合并冲突；采用主线已有格式与Windows注册，无旧语义删除。合并提交0ab827265保留。
2. build01/raw日志有GetSecurityInfo ACL输出指针FFI重复声明签名warning，未压警告；第二轮改成与既有Win32声明一致的ACL指针，build02/03不再含该warning。没有本轮编译/测试非零失败；所有旧失败仍在原不可覆盖轮次。首个build01运行期间补入了phase架构文件，不能宣称全程冻结；早期几次诊断EXE已被后续构建覆盖且未逐个保存EXE hash，不补造摘要。详artifact-provenance-limits.json。最终build03/native03/product02冻结输入与最终源一致，实际EXE摘要已保存。
3. build01 2项lib通过（新增phase/replay前）；build02 6项通过；build03最终8项通过。native01/02各2项通过，源随boundedIO修复后native03最终4项通过，含silentpeer及queued-tail反例。早期源与最终不同的文件逐项记录，不将它们冒作最终同source全通过。
4. product01 debug bins0/141896ms，早于末次复核；product02最终debug bins0/137535ms。最终library54warnings、product71warnings照留，多为待接线类型/既有告警；没有严格Clippy或release通过声明，没有更改release优化/panic参数。
5. 当前未解决首前提仍是旧真实普通SID1046的完整安全open raw1314/缺SeSecurityPrivilege。旧请求access0x81020000；本轮r3新mask0x01020000与BACKUP读取只完成代码/纯错误分类，**未实测新mask会返回1314**。复用旧普通令牌缺口与r3正式限制，未把旧结果改成新调用通过，也未再建同权限账户复跑。

本轮前提拒绝/production gate主要是源码与合成测试结论，没有运行新的public Start，不能把“零官方进程创建”说成已实际跑完整Start拒绝矩阵。native管道与phase合成文件测试是实机Win32，但没有实际官方/模型/业务请求。

## Administrator 与 marenben

用户先提出使用Administrator，再提出marenben。只读Get-LocalUser、准确本地Administrators成员与quser盘点记录于marenben-session-inventory01.json：marenben/SID尾1003启用，未列入直接本地管理员成员，盘点时无其交互会话。该账号的实际TokenElevation/TokenSessionId/noJob尚未证明，不按组名推断已通过。Administrator已有会话1；本轮没有把管理员令牌当ordinary positive，没有提升/启用特权、借服务/计划任务或改策略。

已请求用户交互登录marenben并保持桌面，供后续r3普通bootstrap验证；没有索取或保存密码。既有marenben不会作为本任务临时账户被删除，也不读取/改写其日常客户端配置。只有在真实桌面普通token/noJob及安全工作路径证实后才进入依赖H的实测。旧普通SD失败可复用；它不会因换普通账号或H环境通过而消失。

## 完整状态表

| 项目 | implementation | test | 限定 |
| --- | --- | --- | --- |
| baseline-r3 | merged | verified | main dbf974097/docs a6d9184b8, TEAM13/IMP139/r3; r2 four signatures unchanged; two format/reorder conflicts resolved with incoming formatting |
| O-exact-security-API | implemented | compiled-and-pure-tests-passed | READ_CONTROL/ACCESS_SYSTEM_SECURITY0x01020000; GetSecurityInfo SE_FILE_OBJECT BACKUP0x10000; complete structure/length/control/self-relative bytes, exact API/raw-code preservation; no fallback or privilege change |
| O-old-ordinary1314 | unchanged-prerequisite-gap | reused-not-rerun | old real SID1046 open with0x81020000 raw1314, SeSecurityPrivilege absent; new r3 masks NOT actually queried on a new ordinary token, so no claim of new-mask real1314 result |
| O-before-client-mutation-gate | wired-in-production | compile-only | new Start must pass read-only complete original preflight AND durable capture gate before native official construction; no real public Start called in this round |
| O-phase-prefix | architecture-implemented | 2-new-tests-passed | canonical schema and create-only private records/held native handles; all phase prefixes discover pending, gaps/duplicates/schema/digest/sequence/binding drift rejected; textualClosed never creates native proof |
| O-custody-and-current-ref-CAS | NOT-implemented | not-run | no actual original move/working copies/content tree/ADS/durable current ref CAS/restoration; ClientOriginals trait NOT implemented; require_durable_capture remains unconditional refusal |
| O-full-native-positive | blocked | not-run | no proven permitted full-SD ordinary environment; no new accounts or same prerequisite loop |
| H-private-native-pipe | implemented-native-component | 4-tests-passed | protected currentSID/System ACL, local only, firstinstance, noninheritable handles and actual OS client/server PID + retained FILETIME/token/image; tests same-process peers, not actual managers |
| H-suspended-original-host | implemented-native-component | compiled-not-executed | creator noJob/ordinary; exact ocxd internal mode; private inherited bootstrappipe allowlist; originalprocess/thread noJob/token/currentimage beforeResume; no BREAKAWAY or Job adoption |
| H-OwnerContinuity-trait | implemented-in-cfg-test | compiled-not-live-authenticated | original Owner retained; native pipe and peer challenge/SID/session/fileID/currentI checks; attached proof private and non-Clone/serde; oldgeneration rejected; no real new manager attached |
| H-replay-and-budget | implemented | pure-tests-passed | request/action/256-bit nonce retained without eviction; one action per attachedproof; original QPC start/frequency/deadline and RestartStop-subwindow; sampleInstant precedes QPC to avoid extension |
| H-exact-installation | implemented | pure-negative-tests-passed | broker must I/ocxd.exe and manager I/opencodex-desktop.exe or I/ocxd.exe; arbitrary/current-other-I images rejected |
| H-delayed-native-cancel | implemented | silent-peer/accept-timeout-tests-passed | caller returns by inherited deadline; worker owns only pipe/event/OVERLAPPED/buffers through native cancellation, no owner/job or late authority; no deadline reset |
| H-trailing-frame | implemented | queued-extra-frame-test-passed | already queued trailing bytes/frames refused before auth. Future arriving bytes never authorize a second action on consumed attached proof; no crossmanager live test |
| H-long-lived-ocxd-dispatch-and-rendezvous | NOT-implemented | not-run | ocxd internal daemon/bootstrap/rendezvous and production GUI/IPC routing not wired; trait object does not by itself instantiate real broker |
| H-real-desktop-marenben | environment-unready-at-inventory | read-only-inventory-only | existing enabled SID1003, not found in direct local Administrators membership, no listed interactive session. This is NOT proof of runtime token or noJob; login requested from user; no account/profile mutation |
| H-Administrator | not-substituted-for-ordinary | not-rerun | user proposed Administrator as alternative; existing session1 may control environment preparation but no ordinary/elevated gate weakened or dailyprofile/config used |
| H-real-manager-replacement | not-wired/not-ready | not-run | no original broker creation handle/live samegeneration/noJob/native challenge replacement or oldmanager forced-exit test; cannot claim noJob from Explorer name or mere new process |
| S-SupervisorAbsence-trait | implemented-and-registered-Windows | compiled/namespace-classifier-tests-passed | private Absent binds original Owner scope and exact inherited stop deadline; native exact SCM/Scheduler, H staged targets/state/mirror, original family complete; final fresh requery, no serialized proof import |
| S-Profile-default-state-mirror | implemented | new-negative-test-passed | sameProfile .opencodex/service-state.json independently checked even customH. Missing customH state cannot override present mirror; same-name dir also rejected; no state/config content read from daily/marenben profile |
| S-present-or-unreadable-state | conservative-Unknown | compiled/pure-and-synthetic-tests | existing state declaration relation parsing still not implemented, so any present/unreadable record refuses Absent; not silently excluded |
| S-live-source-bound-absence | NOT-validated | not-run | no new exact native SCM/Scheduler queries after r3; old independent namespace facts reused, not minted as source-bound proof |
| D-DiagnosticRoles-trait | implemented-query-candidate-adapter | compiled/syntax-counterexamples-passed | requires original member handle/generation/parent argv capture time/phase, exact System32 image/target/syntax, current archive and pinned officialcallsite hash, admitted parent Bun CLI; rawexit unchanged |
| D-actual-query-capture-time | implemented | compiled | new monotonic captured_at bound by actual native capture; original parent must be retained and captured no later than query; fixture never gains official source authority |
| D-wrong-target-mutating-command | implemented | new-negative-tests-passed | foreign target, delete, extra shell tokens, barefilename, qc, /query missingxml and alteredpath rejected; no new grammar from observed source alone |
| D-full-family-role-accounting | incomplete-conservative-refusal | not-live-validated | non-query service/otherRequired classification incomplete; nativehistory/unknown/unreadable/stillrunning/requirednonzero remain veto; current productionStop strictall-zero still in force |
| D-pinned-source-readonly | read-only-reviewed | hash-verified | exact fixedtgz source retrieved read-only from originalbundle, no official edits. winsw.ts contains qc beyond frozen query grammar; no actual qc child observed here and grammar NOT expanded |
| P-real-official-Start-Status-Stop | blocked/dependent | not-run | no service/launcher/broker/account created; Opositive/Hdesktop/hostdispatch/fulladapters allrequired; no oldW-L1 positive replay as newpass |
| P-restore-Restart-secondStart-Auto | blocked/dependent | not-run | no OriginalSession/ReadySession/PreparedRestore/ClosedEligibility; no successor or budget extension |
| MSVC-first-checkpoint | passed-with-warnings | exit0 | two tests beforephase/replay additions; clashing GetSecurityInfo ACL-pointer signature warning retained and fixed in build02; no suppressed lint |
| MSVC-final-library | passed-with-warnings | 8-new-tests-exit0 | 54 warnings retained (mostly unwired types/legacy warnings); no strictClippy pass claimed; 1872 unrelated tests filtered |
| MSVC-final-native | passed | 4-native-tests-exit0 | sameSource actual pipepeer/currentprocess, firstinstance, silentpeer, oversized/trailingframes; not W2/twoSID/GUIacceptance |
| MSVC-product-before-review | passed-with-warnings | exit0/141896ms | actualdebugbins; earlier than finalerror/mirror/current-I/boundedIO review fixes, not promoted to final sourceartifact |
| MSVC-final-product | passed-with-warnings | exit0/137535ms | final debug products compiled from final Rust bytes; 71 warnings retained, no strictClippy or release pass claimed |
| cleanup-this-round | completed-for-executed-native-fixtures | testprocess-exit0 | no new account/Profile/Publicroot/service/broker/task/MSI; synthetic phase tempfiles normal RAII and testprocess exit0; buildtargets retained |
| cleanup-historical | unchanged-unfinished | not-retried | all old failed Profiles/Public/schema cleanup and ordinaryMSI gaps unchanged; old automatic rejection never retried/bypassed |
| constraints | maintained | not-run | noW2/dualSID/release/policy or token adjustment/dailyconfig write/business sync/model request; Administrator/marenben not used for business changes |

## 制品与实际源

所有制品均为MSVC debug/test，不是签名release候选。最终源码16个净差文件与Git逐项对齐（CRLF差异明确），build03/native03/product02每组34份实际Rust字节与最终源逐项相同，详final-source-manifest.json。最终EXE摘要：

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75435520 | b1a5fc1efa4d5009673b11b672a9b1cd06b92f8251c32be5cfef9050a67a96ae |
| ocxd.exe | 4627968 | a44e4348a0ecc00a3e16cba5da995a983c6ddb7bd0d9a179467ae88133afa8a1 |
| ocx.exe | 306176 | bf0d1b7fcec56a6b81303b86942154bf5dc8995ef6bd972bdd36996ad54d2fa1 |
| library-r3-tests | 93082112 | 0cff647f2e734a6661b158313030d4bdef9f6b166039a0bda254d3dddce52bd3 |
| native-r3-pipe-tests | 1687552 | fd26b28ecdb2f57f559352ba6347dabb122811ecc0c17f1b9030046907a72de6 |

已复核固定bundle/descriptor/内嵌官方2.99 tgz及所读5份官方源码摘要。源码state.ts说明H自定义时仍有同Profile默认service-state镜像，已加独立拒绝。winsw.ts还含qc只读调用，但本轮没有实际qc子进程证据，冻结query grammar未自动扩大；未知语法保持拒绝。原包未改。

## 清理与保持项

本轮新账户/Profile/Public根/实际broker/官方runtime/计划任务/MSI均0；只在任务native合成test进程内创建私有pipe/peer/phase临时文件，最终test进程exit0，其进程内句柄结束；合成phase临时根正常RAII清理。管道超时的迟到取消线程只保管pipe/event/OVERLAPPED/私有缓冲，没有Owner或Job，caller在原deadline返回，不能用迟到结果补签成功。没有用tasklist代替Job/句柄证明，没有force kill。

编译脚本只写Process PATH，未调用registry PATH写者；本轮没有发生输入/桌面ACL变更，旧日常PATH/ACL原证据按未受影响复用，不伪造本轮before/after快照。旧失败Profiles/Public/schema/普通MSI未验等缺口保持，旧自动审批拒绝未重试或换方式删除。工程target与实际EXE原件留用，历史cleanupComplete=false。

证据新轮次不可覆盖，.gitattributes保留raw原始字节，manifest逐项覆盖除自身外所有payload。正式IMP/TEAM未回写，本轮只贡献此新目录与windows-dev-handoff。
