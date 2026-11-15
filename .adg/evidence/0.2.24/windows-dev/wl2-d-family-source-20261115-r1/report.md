# W-L2 原启动参数与官方 Node→Bun / Service 来源增量

2026-11-15 Asia/Shanghai；TEAM13/IMP139/r3承接r2，继续main dbf974097d / docs a6d9184b85。此前保留native对象/S/D修复代码eb8d7e949、证据88d747a3d作为本轮基线。本轮allAccepted=false、productionExecuted=false、actualGUI=false，未启动官方服务，不复测普通完整SD旧前提。

## 实现及固定来源

先记hypotheses01后只读固定外包/内部官方2.99tgz，内部SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；八份精确源原字节及摘要单列，官方不改。实际bin是bin/ocx.mjs，而非不存在的bin/opencodex.mjs。原入口101/103/1557/1614行确认：Bun args=[准确cliPath,内部launch-proof43字符base64url参数,...原Node动作参数]，proof随机32字节。旧len3保守语法无法描述真实官方启动，本轮补完整固定链路而非按Bun/Node名称授权。

native捕获原根实际命令参数在CREATE_SUSPENDED返回的原process handle上、首次resume之前完成，沿原有界Unicode/argv解析器，私有Owner字段保留实际观察时点/raw/argv。新NativeArguments是观察数据不是proof，不接收IPC/JSON重建。原子代次argv/parent捕获仍要求原Job，原Root只在原创建者的挂起准入内读；任何失败沿原挂起准入异常处理，未开始客户端变更或官方用户代码。

D要求原根是准确当前Node+固定bin脚本+start --port canonicalu16；Bun actualargv须逐项承接原根动作/port并含恰好一份官方内部proof参数，准确native父代次等于原Node，捕获时点有序，两个原镜像fileID/currentsource/fixedcallsite/完整Job原历史一致。proof形状不认证，只是语法检查；没有复制HMAC或把arg值当权限，不输出真实proof值到本轮新日志/响应/持久授权。

原Bun启动参数在Stop时仍是start，因此查询子进程的动作阶段从原Owner捕获时点绑定，不能强求父argv被改成stop。SCM/Scheduler只有r2原冻结准确查询可取诊断角色，sc qc等未扩。Launcher为已证明原Node直接Bun启动链，仍raw0必需；Service另须匹配typed Ready/current检查后准确pre-stop原监听代次，绝不能用pid/string自填。未知helper（sentinel/console/priority等）仍Unknown，不以exit0发OtherRequired。

library01首版纯语法1项通过后，静态独立复核发现is_ok_and会把已知Service的missing/unreadable监听证据吞成Launcher，hypotheses02先记后修：只有真实service状态缺席是None；已知Service若缺原监听/身份冲突直接Err，原Ready证据未知不能退回Launcher。此最终运行体尚无生产正例；产品编译通过仅证明类型/装配，不伪称该路径已实机运行。

## 定向结果与来源界限

native01原根挂起读取/原引号Unicode参数/退出后保留1项通过；query01原短命查询实际argv/父代次/raw1060/1的1项通过，fixture没有官方source仍Unknown；library01新Node/Bun exact语法与逐参数漂移/缺proof/多eval/错port/非规范port拒绝1项通过。共3个不同定向用例，无失败运行；真实角色/服务生产验证未执行。上一批不受影响的raw-exit/准确query/S/H/CLI/MSI证据复用。

library01在Ready错误传播修正前编译；其pure grammar函数和test源码片段与最终逐字/换行等价，摘要见reused-pure-scope.json，所以复用该精确纯检查，不额外重建测试EXE冒全生产覆盖。native01/query01实际编译的backend与独立宿主/Cargo15项均与最终匹配；快照中的D/Owner两文件在后继修正，不冒全36一致。product01最终36Rust逐字对最终源，实际编译head eb8d7e949+dirtydiff/source/工具链均留原件。后继提交只封存相同字节。

MSVC debug全部bins0/156419ms，72warnings原样；非签名release/冻结候选，无原包预算正例。代码三笔均已推codex/024-windows-native：66c3a3e97d83488110e72bb1e34a10879ded5ea9（原根参数捕获及native测试）；a7025bb89fb7f730eee23da2e9bfedd9e3b80b11（固定父链/来源/阶段及原Ready服务代次）；75c0f7562f4d60fd0fbb704dc08dcf0cd3d47785（定向入口）。净6文件均在合同Windows/test写集，完整最终Gitblob/原字节在final-source-manifest.json。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75459072 | a885081683e23764eb824436dd4e78d8661b7335b829a2d9dc94b2cb40191587 |
| ocxd.exe | 5398528 | c7010fa17aeea92483bf38abb2963c271454a8ecc5d2fa4a62c0bdbd7fc6b812 |
| ocx.exe | 306176 | 258bee19a51f09065bc2ead8074be80e5efb7beb735593a12967657c0d1e35f7 |

原native测试/fixture和library01中间测试EXE的准确SHA另存artifacts-before-final-tests.json；不因后继编译替换而补造旧hash。

## 完整状态表

| 项目 | 实现 | 定向结果 | 限定 |
| --- | --- | --- | --- |
| original-root-argv | implemented native pre-resume capture | native01 passed | Exact original process handle, quotes/Unicode preserved; facts not an ownership proof. |
| original-short-lived-query-history | unchanged original parent/argv/raw | query01 passed | SCM1060/Scheduler1 kept; fixture no official source remains Unknown. |
| fixed-source-start-grammar | implemented Node->Bun chain | library01 1passed, exact pure scope reused | Final pure predicates and test bytes match captured source; native authority requires original parent/Job/current source, not proof shape. |
| query-phase | implemented original invocation + captured action phase | compiled; no real production case | An existing start Bun may create stop-phase queries; root argv never rewritten to stop. |
| Service-generation | implemented typed-ready pre-stop native generation lookup | final product compiled; runtime positive blocked | Known Service missing/contradictory original pre-stop listener evidence propagates Err; no fallback Launcher. This method was not exercised by pure grammar test. |
| Launcher-generation | implemented exact direct original Node->Bun chain | native root/grammar checks passed; production not run | raw0 remains required; arbitrary system helper/Bun names cannot create role. |
| diagnostic-grammar | unchanged two frozen queries | prior exact-query/raw-exit invariants reused unchanged | No sc qc or mutating target accepted; source-only discovery is not approval for grammar expansion. |
| OtherRequired-and-helper-family | incomplete | not accepted | Sentinel/console/priority helper source/argv/call-site combinations remain Unknown until proven; raw0 cannot bypass. |
| MSVC-product | final36Rust compiled | exit0/156419ms;72warnings retained | Debug only; original release budgets unchanged. |
| O-ClientOriginals | incomplete | no SD retest | Same-volume custody/current-ref CAS/complete restore/Closed remains unfinished. |
| H-real-GUI | incomplete | previous empty-host component reused | No actual GUI/official Owner/service rerun. |
| S-production | partial | previous directed proof reused | No owner-bound real production positive this round. |
| production-start-stop | refused | not run | Needs O fullSD and four production adaptations; 30/5/10 and Restart30/Stop10 retained. |
| ordinary-MSI | not accepted | not repeated | Earlier administrator transactions do not substitute ordinary acceptance. |
| cleanup | new task fixtures only | all new synthetic fixtures normally exited/RAII cleaned | No new accounts/Profile/Public or force requests this round. Prior exact rejected failure tree and all historical gaps remain; no retry. |
| constraints | maintained | noW2/dualSID/publish | No official/daily configuration/PATH/policy write; no models/business sync. |

## 清理及未完

本轮新synthetic测试仅自有目录/进程，native/query正常退出0且原Fixture清理断言完成，无新强清、账户/Profile/Public根/官方进程、日常配置或注册表PATH写。之前native01失败树7文件670767字节的自动审批blocked by policy拒绝及历史Profile/Public/schema缺口完全保持，未重试或换机制；本轮overallCleanupComplete=false。工程cache/实际EXE/本轮不可变证据留存。

O完整同卷custody/current-ref CAS/恢复/Closed、H真实GUI与原Owner生产路由、OtherRequired全helper来源/完整family、S/D实际生产矩阵尚未完成。普通原令牌完整SD含SACL前提仍未满足，不提升/改权限/改策略，不重复相同1314，不将本轮source检查升级为生产Start/Stop通过。若要求该普通环境可用，需要另行明确完整安全覆盖的产品方案；本轮没有改变合同。原30/5/10、Restart30含Stop子10和未知拒绝保持；无真实模型/业务同步、W2/双SID或发布。

离线重放 python verify_evidence.py 只验证本归档及限定，不启动Windows运行时动作。
