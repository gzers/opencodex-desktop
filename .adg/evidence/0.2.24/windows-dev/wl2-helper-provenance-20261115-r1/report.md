# W-L2 Sentinel来源及O未决记录严格绑定

2026-11-15 Asia/Shanghai。沿TEAM13/IMP139/r3承接r2，fetch main dbf974097d067e75c4de68f9fc681af8ab447b09 / docs a6d9184b851521194477ddfeecd988f40d513e5b未变。代码从75c0f7562接续；本轮allAccepted=false、productionExecuted=false、actualGUI=false，非生产四边界收口。

## Sentinel准确来源

先写hypotheses01再只读固定外bundle14ef6025…及内官方2.99tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。六源原件/长度/SHA保存official-source；未改官方包。windows-process-priority.ts实际使用node:os setPriority(0,...)，不生成一个可凭空豁免的PowerShell进程；没有按“优先级helper”发新角色。

准确installed health sentinel是[bun原镜像, package/src/service/health-sentinel-main.ts]，不是任意同名程序或__health-sentinel参数。OtherRequired只有同时匹配原typedReady/原prestop Service父代次、原Node→Bun源链、原start捕获阶段、子/父同原镜像fileID及四份精确官方源码时才构造。四源为launch/main/windows/CLI，聚合digest明确分隔每路径/摘要；完整programImage前后仍复验。任何外image、额外参数、eval、其它目录同文件名、捕获阶段/原Service缺席都拒绝。其exit非零仍否决NormalExit，不把它当诊断query。

固定源码本身说明sentinel只用于managed service，普通foreground/desktop sidecar并不主动启动它。本轮没有打开该配置或启动official/sentinel；pure grammar/退出测试只覆盖新predicate与refusal，不能叫真实sentinel生产正例。conhost/其它完整helper角色仍Unknown，不靠同Job或raw0放行。

## O未决记录身份修复

original-record-hypothesis记录旧phase接受任意JSON对象（含空binding）的结构缺口；旧代码也未发native闭包，本轮不追溯声称发生过错误恢复。新binding.rs用deny_unknown_fields嵌套类型要求：原SID，Profile/D/H/I准确本地路径及卷号/字符串fileID，source/current摘要，owner PID/字符串FILETIME；严格规范整数、SID结构、SHA、UUID非nil/规范拼写。控制根不能相互重叠、与client .codex/.grok范围重叠或声明同目录对象；D/Profile异卷数据合法，仍无真实custody证明。

记录读取在原PrivateRecord/目录pin内继续；逻辑SID须同当前TokenUser，记录sessionId须同准确session目录叶名；缺字段/未知/嵌套重复键/回退/gap/replay/错误前digest均拒绝，未回退默认空值。原prefix/只有全部阶段依次出现的语法检查保持。原生路径语法校验仅加一个无I/O的公开syntax函数，成功绝非native目录ID/security或许可。

originals01四项通过后，native-path-review独立记录重复/尾随分隔符可能造成文本overlap别名的风险；最终canonical components校验及两个反例一并增加，只要求持久记录的路径拼写规范，不改变底层native路径等价。originals02四项再次全部通过，原结果不可覆盖。

这些serde类型只是身份声明数据，不是任一trait的NativeSession/OriginalSession/ClosedEligibility。即使结构完整链文字为Closed，read_pending仍只返回PendingOnly；没有移动原件、创建工作副本、发布current-ref或发后继资格。完整原件安全描述符权限、同卷custody/current-ref CAS/恢复/Closed仍未实现。本轮未用测试诊断投影代替fullSD，也没有新token1314空跑。

## 结果、来源与制品

helper library01角色suite4过（其中新sentinel1+受影响既有3）；originals01中间4过，originals02最终4过（新binding2+受影响prefix2）。合计8不同用例、12成功执行；本轮没有失败测试或编译，firstError=null。完整实机proof未生成，普通SD/原H桌面/CLI/MSI不受本批影响部分沿旧范围复用。

代码080b80c41cacc490587a1881c4fe98f52314d786（sentinel原source/Service父链）与cf03db58036780f8d02aaaa5bfd52654ee4bbab5（O typed binding/session与canonical路径）已分别push codex/024-windows-native，净6许可Windows文件。最终source/tree/Gitblob/原字节在final-source-manifest.json。没有共享trait/Cargo/release/安装器/旧lifetime或Mac修改，preexisting untracked windows-schema保持。

最终产品基于干净cf03db580编译，source.json实际head一致/dirtydiff为空；originals02/product01每组37Rust与最终逐字相同。helper library01在O改动前，captured36中的4源后来变化且不含newbinding；helper自身完整文件逐字相同，不能冒全36最终匹配。originals01的binding/phase后有canonical增量，明确非最终。三份实际库testEXE在对应时点单独记SHA，源码/EXE归属不混用。

MSVC debug bins exit0/128147ms，85warnings原样保留。非release预算测试、签名制品或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop.exe | 75458560 | 40a65cf0cefb57e053e7ca1c4b6b007a9557103a8bd45dba8ff50beeabfc00eb |
| ocxd.exe | 5398528 | 20dbf3e6760cdd0d51e60214e8f356408ed78b33b90cc44a731ffe0755c886c4 |
| ocx.exe | 306176 | adb2684b17e60bfa39fdccbabc345a892d246cda635d24ab5d88c813e35ff682 |

## 完整状态表

| 项目 | 实现 | 定向结果 | 限定 |
| --- | --- | --- | --- |
| fixed-source-review | 6source hashes verified | read-only official2.99 | Exact health sentinel entry + dispatch/launch/windows source; node:os priority has no child helper. |
| D-sentinel-OtherRequired | implemented original service parent and start phase | new pure grammar/exit refusal1passed | Four source hashes + native currentBun/image/Job/parent + originalReady/prestop required. No production sentinel positive this round. |
| D-regressions | raw required/diagnostic separation retained | 3 affected existing role tests passed | Launcher/Service/OtherRequired nonzero still veto. Unknown arbitrary helper raw0 cannot pass. |
| O-binding-structure | typed nested deny_unknown_fields | originals02 final4phase tests passed | Full identity fields/data-only; original prefixes/unknowns/duplicates/phase gaps retained. No NativeSession/OriginalSession/Closed authority minted. |
| O-binding-path-alias | canonical local path data check | originals02 4passed; includes alias2new counterexamples | Repeated/trailing separator aliases refuse in records; underlying native validator retains existing semantics. |
| O-session-and-SID | record scope matched to directory/currentSID | nestedduplicate/unknown/wrongsession counterexamples passed | No newSID or dualSID execution; synthetic declared identity never substitutes actual objects. |
| MSVC-product | clean final source compiled | exit0/128147ms;85warnings | MSVC debug bins, exactcf03 head/emptydiff; no release budget/frozen/signed candidate acceptance. |
| O-native-custody-and-currentRef | incomplete | not run | Same-volume original move, completeSD/currentref CAS/restore/Closed still missing; no production Start. |
| O-ordinary-fullSD | environment unavailable | known failure reused;0retests | No privilege/ACL/policy/token adjustment or narrowing coverage. |
| H-production-GUI | incomplete | prior empty-host fixture pass reused | Real GUI/original Owner integration not accepted. |
| S-production | partial | previous checks reused | Full real source-bound matrix not accepted. |
| D-full-family | incomplete | unknown helpers still veto | OS console/other source+argv+parent combinations unproven. Sentinel alone does not complete family. |
| ordinary-MSI | incomplete | unchanged evidence reused | Administrator MSI pass not ordinary acceptance. |
| cleanup | taskonly synthetic roots | testsRAII;no newaccounts/Profile/Public/force | Old native01 7files670767bytes and all historical Profile/Public/schema rejections retained, no retry. |
| constraints | maintained | noW2/dualSID/publish | No real model/business sync or daily config/PATH/official changes; original budgets unchanged. |

## 清理和未完

只有任务自有synthetic records/TempDir，成功后private记录句柄和目录pin释放、原TempDir guard收尾；没有失败临时现场、新account/Profile/Public根、官方进程或强清动作。没有日常配置/注册表PATH/机器策略写。工程cache/实际EXE/不可变证据保留。先前7文件670767字节失败树及全部历史Profile/Public/schema自动审批拒绝保持，不重试/不改机制，不宣告全部cleanup完成。

O保管/当前引用CAS/完整恢复，H实际GUI原Owner生产路由、D其它完整helper family、S/D整组实机矩阵仍未闭合，普通MSI未验保持。普通完整SD含SACL环境独立缺口仍在；在r3下不允许缩范围/提升令牌或改策略凑过，所以production Start保持拒绝。不把这项环境事实说成缺共享trait声明，也不将pure decode或helper语法当生产资格。

不W2/双SID、不发布，无真实模型/业务同步。离线python verify_evidence.py只核清单/原字节/实际源与结果限定，不运行Windows进程或清理。
