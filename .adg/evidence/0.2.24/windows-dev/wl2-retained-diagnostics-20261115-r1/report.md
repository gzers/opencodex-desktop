# W-L2 保留原生对象、D退出复核与S路径缺席修复

2026-11-15 Asia/Shanghai。TEAM13/IMP139/r3承接r2；fetch main dbf974097d067e75c4de68f9fc681af8ab447b09 / docs a6d9184b851521194477ddfeecd988f40d513e5b未变。allAccepted=false、productionExecuted=false、actualGUI=false。本轮是原生及S/D增量，不把前轮空host/fixturemanager成功升级为真实GUI或四边界通过。

四笔代码均推codex/024-windows-native：4562217649da1c822f07affc9caf279558f78665（保留原native image/token对象、原进程句柄和单调forced历史），1007ad5f774133973d36be506de71d383e4317ad（D分类使用原保留代次、原始raw/角色/完整历史核验），7f68125d7db9cbadfdd91c3b0f987172a64184d5（S父路径pin/错误阶段与raw保留及junction反例），eb8d7e949f42e6bd80178e4242d721f648634066（分离定向构建入口）。最终9文件均在合同Windows/test写集，没有改共享trait/预算、Mac、安装器、官方包或日常配置；原untracked schema保持。

## 假设、首错和修复

先写hypotheses01后一次收集独立项。D旧complete_family要求全部退出，却调用active-only镜像检查，真实退出对象必然被拒。首轮新增退出后验证仍调用通用observe，native01 raw101：原子测试内after-exit检查报Win3231，另一条强清反例已通过。没有逐API instrumentation，不能编造“某一个内部API恰好报31”。原trace/执行源/原场景完整保留。

修正为原进程仍活时取得并保留准确token只读句柄、镜像只读denywrite/delete句柄和原父目录pin。终态只对同一原process handle读取PID/FILETIME/Wait/raw，对同一原token/image对象读回SID/session/elevation/fileID并复核父路径。活进程仍做fresh nativeobserve；不改active-only service准入，不把JSON/旧PID变权限。原根挂起准入失败仍在原异常分支单独清理，退出历史与命令捕获必须在活窗口完成；缺失/漂移保持Unknown。

Job monitor先后采集的同一代次必须相同；若有界family观察先持有原句柄，随后monitor送来该原代次command/source时保留后者，不能静默丢掉命令历史。强清调用前置monotonic标记，包括已空Job和失败请求；后来raw0不能抹掉曾请求强清。D最终normalproof重核完整数量/唯一代次、原raw、角色语义与scope/deadline/force；未放宽诊断非零成0或未证明的子角色。精确父参数只接受当前已实现的完整语法子集，不再用任意参数包含CLI/phase判断；完整family仍未完。

S检查准确H staged叶子前固定最近已存在父目录及其全部非reparse祖先；只有此原父路径稳定的叶子NotFound可Absent。父pin错误即使原raw2/3也Unknown，保留原始2/3/5/32及父/叶阶段，不压成通用0或伪缺席。service-state及默认镜像复用同一约束。

library01三项过、junction准备命令syntax失败；library02缺CommandExt/E0599，测试未执行；library03四项过、同junction准备仍exit1。完整source/stdout/stderr/result分别保留。hypotheses04后复用仓库windows_private_create既有PowerShell New-Item Junction方法，系统准确PowerShell路径，路径用独立环境值传入，无shell拼接/策略/特权变更；原reparse拒绝断言完全保持。library04五项通过，真实junction创建raw0；supervisor01七项通过，重复的3项新S用例单列。

## 结果与来源

native02 2项过，query01受影响短命查询1项过；library04新D/S 5项过，supervisor01受影响7项过（其中3项重叠）。合计12个不同用例、15次成功用例执行。query捕获原SCM1060/Scheduler1及父代次/准确argv；fixture无官方call-site仍Unknown，不能输出生产NormalExit。原权限/预算/未知拒绝都不变。

product01 MSVC debug全部bins exit0/179337ms，69warnings原样保留。最终36Rust与query01/library04/supervisor01/product01每组逐字节匹配；native02编译的backend及独立宿主/Cargo15输入与最终相同，但快照中另两份S源码后有改动，明确不是全36匹配。后继Git提交只封存相同字节，不冒已按最终Git SHA重建。source.json保存真实编译head/dirtydiff/命令/工具链，final-source-manifest含最终Git blob和CRLF等价核对。

均debug/test，非签名release或冻结候选。精确制品如下，库及原生测试EXE摘要另见library-artifact/native-artifacts。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75456512 | b5f3782a1aa0dfe071c1f021d829ff2efdbac2050cbb6ec11e8b61a8e93a1410 |
| ocxd.exe | 5398528 | 5f058007a91f3e6a5c1c6f75d7d7d388bf1f0a0533d67a431cc8e6f26b4a52a6 |
| ocx.exe | 306176 | be27db2cb26ee7c1ac25460f323dd32c3051ce3f014d6d0a4b94d57ca5469310 |

## 完整状态表

| 项目 | 实现 | 定向结果 | 限定 |
| --- | --- | --- | --- |
| original-exited-source | fixed | native02 passed | Retain original image/token/parent objects while live; verify original process handle PID/FILETIME and same native objects after exit; live admission still active-only. |
| foreign-generation-and-image | native refusal | native02 passed | Wrong PID/FILETIME/SID/fileID and equal-byte different object refused; rename/write of retained image denied. |
| force-history | monotonic veto | native02 passed | Explicit force request recorded even already empty raw0 Job; no erasure by eventual zero exit. |
| original-query-history | retained native facts | query01 1 passed | Exact sc/schtasks argv,parent and raw1060/1 retained. Fixture has no official source => Unknown role. |
| D-post-exit-classification | fixed to retained source verifier | native primitive + product compilation | No post-exit process/token reopen assumption; never from PID/JSON. |
| D-normal-exit-check | strengthened | pure exit matrix passed | Same scope/deadline/full original history/raw exits/no duplicates/no force; still no production NormalExit positive. |
| D-parent-grammar | exact bounded subset | 2 pure D tests passed | Only exact [image,officialCLI,phase] supported; valid expanded official argv and required/service family remain incomplete; flags/eval/another script refused. |
| S-staged-parent | non-reparse parent chain pinned | native junction counterexample passed | Missing nested leaf below junction/file remains Unknown; exact regular missing leaf Absent. |
| S-error-preservation | separate parent/leaf failure | raw2/3/5/32 counterexamples passed | Parent error never leaf absence; exact native raw code preserved. |
| S-existing-regressions | same source/namespace gates | 7 tests passed | Includes3 new S cases +4 affected existing cases; exact SCM1060/scheduler0x80070002 observed independently. |
| MSVC-product | final Rust compiled | exit0 /179337ms | MSVC debug bins,69 raw warnings retained; no release budget/signed candidate claim. |
| original-failure-evidence | retained | native01 101/raw31; library01/02/03 101 | Original31 internal API not instrumented; no invented exact API. Library setup/trait errors separately retained. |
| failed-native-fixture-cleanup | archive verified | deletion blocked by automatic review | Exact7files670767bytes archived; command not executed, no retry/alternate API; root retained. |
| successful-task-cleanup | RAII exact synthetic roots | native02/query01 passed | No new users/Profile/Public/runtime services; successful fixture cleanup asserts; test-only explicit emptyJob force separately declared. |
| O-ClientOriginals | incomplete | not run/reused ordinary fullSD gap | Same-volume custody/current-ref CAS/complete restore/Closed remains incomplete; no capture capability issued. |
| H-empty-host | previous component pass reused | not repeated | actualGUIfalse, officialOwnerfalse; prior real ordinary session/noJob facts preserved. |
| H-production-owner-GUI | incomplete | not run | Original Owner routing, actual GUI and official service integration not accepted. |
| S-full-production | partial | not accepted | Owner/source proof types exist; full production positive matrix not exercised. |
| D-full-family | incomplete | not accepted | Service/OtherRequired/launcher descendant role constructors not implemented; unknown roles continue veto. |
| P-production-budget | refused | not run | No official Start until O/fullSD +four production boundaries; original30/5/10 and Restart30/Stop10 unchanged. |
| ordinary-MSI | not accepted | unchanged evidence reused | Administrator MSI pass not ordinary-user acceptance. |
| historical-cleanup | incomplete | unchanged | Old Profiles/Public/schema rejected cleanup paths not retried. |
| constraints | maintained | noW2/dualSID/publish | Official2.99 and daily config/PATH/policy unchanged; no real model/business sync. |

## 清理与未完

native01 failure guard在原失败后显式清空原自有synthetic Job，单独记forced，不补签正常退出。新native02强清反例只向已空的精确synthetic Job发请求，原raw0保留但normal资格否决。新成功fixture正常结束并精确RAII清理，测试junction只移除链接自身、目标仍在再由任务TempDir收尾；无新账户/Profile/Public根/官方服务。

失败树C:/Users/15119/AppData/Local/Temp/ocxd024-process-18124-1的7文件670,767字节全部原样归档核验。随后exact Remove-Item经自动审批拒绝blocked by policy，命令未执行、计划的内部回执未创建；原工具拒绝事实单独记录cleanup-native01-delete-rejected.json。不重试、不换机制，原目录保留。全部历史失败Profile/Public/schema和普通MSI缺口原样；overallCleanupComplete=false。

O完整原件custody/current-ref CAS/恢复/Closed、H真实GUI与原官方Owner生产路径、S完整生产matrix、D完整Service/OtherRequired/family角色仍未完成。r3完整SD含SACL的普通令牌正例环境仍未满足，不重复同前提1314，不提高权限/改ACL/策略或偷偷缩范围。四trait已冻结，不冒称共享声明缺失；在它们及O前提齐备前生产Start保持拒绝。无官方2.99/日常PATH配置修改，无真实模型/业务同步，无W2/双SID/发布。

离线入口：python verify_evidence.py。只校验本轮清单/原字节/源码快照/原始退出及限定，不运行Windows进程、策略、PATH或清理。
