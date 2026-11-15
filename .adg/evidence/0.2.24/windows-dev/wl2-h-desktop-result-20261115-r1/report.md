# H修复后真实桌面结果与精确清理

2026-11-15 Asia/Shanghai。消费 TEAM r13 / IMP-34 r139 / prerequisites-r3（承接r2）。main/dbf974097d 与 docs/a6d9184b85 fetch未变。实际H执行源 ab87ad1261f927bf77a1c757342989105a40ad0a；清理脚本源 d31616d7d026054880794cacec17113c5c4c1645。新结果只接收空宿主H组件：allAccepted=false、productionExecuted=false、actualGUI=false、officialOwnerCreated=false。

用户从 marenben 普通桌面/session7运行一次修复入口。旧原空宿主PID748/FILETIME134391474259863429经exact旧manager镜像、同SID/session/fresh native peer认证显式退出；原旧observer创建句柄已丢失，此次新QUERY_LIMITED_INFORMATION|SYNCHRONIZE句柄只观察精确原身份并等待退出，非Owner接管。旧manager raw0、旧host raw0、nativeWaitConfirmed=true、forced=false。受限退出时旧host自己的日志追加认证/重放拒绝/显式退出事件，退出前原始日志已独立逐字归档；不声称旧活日志从未追加。lookup副本仅发现数据，未改旧原件/ACL，首次失败原场景保留。

新observer raw0。原挂起broker PID22216/FILETIME134391493883001324/卷247287385/fileID844424931380260由observer持续持有原创建句柄；bootstrap和beforeResume均原生noJob=true，同SID1003/session7/elevated=false/elevationType1。fixture manager generation1正常exit0，generation2只终止该原测试manager/exit1，generation3新manager认证后exit0，三次broker仍活且完整原身份一致/noJob=true。generation4请求空宿主显式退出，四代同连接旧请求各拒绝。最后原句柄观察到退出；新broker未另存raw exit code，不能填写0。它未创建官方Owner、服务或官方Job，也未证明真实GUI跨管理器连续性。

原第一次observer101与manager原生32、首次采集partial3及修后export02完整10原件全部保留于前轮。原生共享反例定位PrivateRecord创建writer与deny-write reader冲突；封存只读发布并保留身份/权限/bytes复核后，3新native、1原record、2受影响phase、1bootstrap全部通过。最终MSVC debug bins exit0/143952ms，62 warnings原样保留。五组各36份实际Rust输入与最终Rust一致。诊断library01非最终源的限制仍保留，不追溯为最终验证。

本结果 old recovery导出10项、新修复同SID导出25项，全部Base64/长度/SHA逐字核验；新导出complete=true、fileErrors=[]、stableLength全true。原始输出中的taskBaseRetained=true表示测试完成/导出时点；其后独立清理回执另行记账。

用户从原普通桌面运行一次精确清理入口。回执complete=true：成功场景128429…的19文件98,827,930字节、发现副本b989…的3文件1,834字节均先按归档清单核验后删除，合计22文件98,829,764字节。未删首次失败13ae…、现有账号/Profile或Public输入，未强杀进程、未重试旧自动审批拒绝路径。控制账户不据此冒称独立读取了私有失败目录。Public输入/输出与构建缓存有意留存；全部历史Profile/Public/schema清理缺口仍未关闭，overallCleanupComplete=false。

完整状态表如下；机器可读表见status-matrix.json，实际源码/制品摘要见source-and-artifacts.json。均MSVC debug/test，不是签名release候选。离线执行 python verify_evidence.py，只读本轮归档，不触发Windows运行时动作。

| 项目 | 实现 | 结果 | 限定 |
| --- | --- | --- | --- |
| original-failure | failure preserved | observer101; manager Win32 32 | Original failed scene13ae and original exports retained; no historical pass rewrite. |
| original-log-collection | complete | export02 10/10; no fileErrors | export01 partial3/sharing failure also retained; no ACL changes. |
| sharing-fix | implemented | native02 3 passed | Read-only sealed publication; locator snapshots return data only. |
| record-regression | implemented | record01 1 passed | Original immutable identity/content/pins/mutation refusals retained. |
| phase-regression | partial architecture | phase01 2 passed | No full native custody/CAS/Closed authority. |
| bootstrap-regression | implemented | library02 1 passed | Strict frame/version/bounds refusals retained. |
| MSVC-product | compiled | exit0 / 143952ms | Debug bins; raw 62 warnings retained; no release budget acceptance. |
| old-empty-host-exit | restricted diagnostic recovery | manager0 / host0; nativeWait confirmed | Old image fresh authenticated; readonly observation handle, no Owner adoption or forced broker exit. |
| repaired-native-token | native ordinary desktop | SID1003/session7/elevationType1 | Bootstrap and suspended broker ordinary, no impersonation/elevation. |
| repaired-noJob | native original handle checks | bootstrap true; beforeResume true | Before resume and after each manager boundary; no outer-Job lifetime assumption. |
| normal-manager-exit | fixture managers only | generation1 exit0; broker alive | Original creation handle retained; same PID/FILETIME/SID/image/fileID. |
| manager-failure | fixture manager Child only | generation2 raw1; broker alive | Forced only original synthetic manager; fault separately recorded. |
| next-manager-attach | fixture manager only | generation3 exit0; broker same/alive | Fresh native peer authentication; not actual GUI handoff. |
| connection-replay | implemented empty-host auth | generations1..4 each refused | Fresh connection/generation semantics; no lifecycle permission granted. |
| new-empty-host-exit | explicit diagnostic exit | generation4; original handle observed exit | Broker raw exit code not separately captured; do not infer 0. |
| sameSID-export | complete | 25/25 length/SHA/stable; no fileErrors | Old recovery export10 separately verified; original encodings preserved. |
| completed-task-cleanup | exact two newly created roots | 22 files / 98829764 bytes removed | All plan hashes validated before deletion; immutable sameSID receipt complete=true. |
| failed-scene-retention | preserved | receipt originalFailedBaseRetained=true | Controller did not enumerate private failed base; no account/Profile deletion. |
| Public-inputs-build-cache | intentionally retained | not deleted | Exact test binaries and scripts retained; old Public/Profile/schema cleanup gaps unchanged. |
| O-ClientOriginals | incomplete | production not run | Full native same-volume custody/current-ref CAS/restore/Closed incomplete; ordinary full-SD positive environment not demonstrated. |
| H-production-GUI | incomplete | not run | Actual empty ocxd host + library fixture managers only; no actual GUI/official Owner/Job/service. |
| S-SourceSupervisor | partial | earlier primitives reused | Complete source/owner/stage-bound production positive/negative proof matrix not accepted. |
| D-DiagnosticRoles | partial | earlier primitives reused | Full required/service family roles and production Stop integration not accepted; raw exits/Unknown preserved. |
| production-Start-Status-Stop-Restart | incomplete/refused | not run this round | O full SD plus all four implementations required; original 30/5/10 and Restart30/Stop10 unchanged. |
| ordinary-MSI-and-historical-cleanup | incomplete | unchanged evidence reused | Administrator MSI success does not close ordinary acceptance or historical Profile/Public cleanup. |
| constraints | maintained | no W2/dualSID/publish | No official2.99/daily PATH/config/policy changes; no real model or business-sync request. |

剩余实现仍由Windows写集负责：ClientOriginals同卷custody/current-ref CAS/恢复/Closed、H真实GUI与official Owner生产接线、S真实source/owner/stage组合proof、D完整family/required-service角色及Stop组合。四trait已冻结，不能把这些说成共享签名缺失。

生产验收还缺满足r3完整SD（含SACL）读取的原普通令牌环境；本轮H环境通过不解除此独立门。没有新条件不重跑相同1314。若产品要在该普通令牌上继续Start，需另外明确授权改变安全覆盖合同；本轮没有弱化范围、提升令牌、改ACL或策略。未更改官方2.99/日常配置/PATH，未请求模型/真实业务同步，不跑W2/双SID，不发布。

离线replay首轮因旧恢复回执的SID位于originalWitness而非顶层，校验器KeyError/exit1；原失败校验器、stdout/stderr/result完整保留replay-attempt01。修正按真实结构取SID，未改变原始回执或产品断言，最终两轮离线复核均exit0。
