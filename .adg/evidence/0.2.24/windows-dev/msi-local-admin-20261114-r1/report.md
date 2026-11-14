# 本机临时管理员 MSI 定向事务

2026-11-14，Asia/Shanghai。fetch共同main fd2538e4dae124b7c521b5b84b7ca5b5c6eeef6b、docs/governance-main 242f46e71e79501e11610c9a7aa7d6d7bc2e9fd8未变；消费TEAM revision10、IMP-34 revision136、MSI补验及环境接收合同。Windows原证据线7a042d41e承接；本轮代码首笔ad3dcce02c3ae89bbe14c4cf32103010918d24c7已推送，最终代码提交见result.json。

用户本轮明确修订环境方案：允许在本机使用新建专用临时管理员账户与真实独立Profile，实际提升令牌运行MSI两项事务；不使用已有Administrator的配置、不改机器策略。此授权替代旧合同对本次补充场景的“必须另机/普通令牌”限制，**不将管理员结果写成普通用户验收**。原普通用户入口及其拒绝保护保留，普通用户MSI缺口仍在。不是W2/双SID矩阵，不是release候选或发布。

hypothesis.json在修改和执行前保存待证伪假设：专用临时管理员能在不改策略/包/安装范围的条件下安装原per-user开发MSI，并在真实/x事务内证明pending Enable拒绝保留、显式继续后正常卸载。goal-revision.json保存用户要求的新目标文字；宿主create_goal拒绝覆盖未完成目标，update_goal仅支持状态，未虚标旧目标完成以绕过。实际执行采用本次用户新授权。

## 结果与来源

| 独立项 | 状态 | 证据与界限 |
| --- | --- | --- |
| MSVC产品lib定向编译 | 通过 | build01与最终build03 exit0，42条warnings保留；build02 JSON宏语法失败原样保留，修括号后通过 |
| 管理员ignored入口发现 | 通过 | discovery/discovery02，仅精确1项；随后只执行此入口 |
| 新账户/真实Profile/令牌 | 通过 | 唯一测试SID尾号1040，Profile ocxw1d1d3a825b8；实际elevated=true，session0；不复用日常管理员Profile |
| 原MSI身份与实际安装 | 通过 | out01真实/i exit0；原包ProductCode不变；per-user unmanaged查询0，machine查询1605，无ALLUSERS |
| pending Enable真实/x拒绝 | 事务观察通过，首版测试日志判读失败留存 | out01 /x exit1603，helper实际exit1；独立前后程序卷/文件ID/字节摘要、PATH、产品登记、pending/config、D/H哨兵与selection完全一致 |
| 拒绝发生在实际RemoveFiles前 | 复核通过 | 原日志调度cleanup后调度RemoveFiles；实际脚本先执行cleanup并失败，失败前没有实际RemoveFiles/FileRemove。independent-transaction-audit01与后继Rust原SID复核 |
| 原Enable显式继续 | 通过 | out02同SID原pending逐字重核、新预览摘要继续；没有重建pending、复制安装程序或重装MSI |
| 真实正常/x | 通过 | out02真实/x exit0；实际执行脚本cleanup在RemoveFiles前；MSI登记state=-1 |
| 程序/终端记录/pending移除 | 通过 | normal-after.private.json，四个安装器自有程序/来源组件缺席，terminal record/pending缺席 |
| D/H/config/live runtime保留 | 通过 | 哨兵/config摘要与out01一致，selection相同，完整合成版本树校验通过；不执行官方脚本 |
| 测试PATH还原/日常PATH | 通过 | 新SID PATH原始类型/字节一致；两轮daily user与machine PATH摘要未变 |
| Profile/HOME分离、当前用户PATH原语 | 复用既有通过 | W0/W1既有原生证据，未受本轮测试控制器变更影响 |
| PowerShell/cmd参数/stdio/退出码、裸ocx、原官方2.99/H读取 | 复用既有通过 | w1d-native-entry-20261113/out04，未重跑 |
| schema3/pending恢复/冲突及NSIS宏 | 复用既有通过 | 原10格恢复、4类冲突、out05与NSIS5格；未重跑 |
| 普通用户真实MSI卸载 | 未验 | 原1625失败保持；本轮提升令牌结果不替代 |
| 账户/任务/输入ACL/桌面ACL | 已处理，详见清理回执 | 原SID账户和两任务均移除，输入ACL恢复；管理员路径未修改交互桌面ACL |
| Profile最终删除 | 未完成 | cleanup05原生error32，Loaded=true；Restart Manager锁拥有者System PID4/Registry PID328；85文件118556840字节保留 |
| 本轮Public自有文件 | 删除自动审批拒绝 | 83文件184522112字节已归档/核验；blocked by policy，命令未执行，未绕过 |
| W2/双SID/业务同步/模型请求/发布 | 未执行 | 不在本轮范围 |
| 整版allAccepted | false | 普通用户/W1其它生产生命周期/固定候选与最终验收等不因本轮补充通过而关闭 |

实际运行的安装器仍为原W1-D开发包：316784640字节，SHA256 9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282，ProductCode {67CF3D62-492D-4359-AAFE-8BE6FB17E647}，名称OpenCodeX W1D Proof。原包与三个产品程序未重建或改写；本轮只重新编译产品lib测试目标，不把测试源码提交冒充原包生产源码。最终测试EXE为92022784字节，SHA256 1dcba506281e02bdbf3714779294f32752b5ff6251198bb2ede14ad5eba09622。首轮EXE单独记录为91996160字节、78a7cc5189a308ece465cbea0ac81032c4b80fc126e61ec9be1665a2c3e96192，保留首次失败的实际制品摘要。

## 首错与修正

首轮实际安装成功、真实失败卸载返回1603后，测试在旧日志断言处失败：它把 `Doing action: RemoveFiles` 当成已执行删除。原日志先生成安装脚本，再在InstallFinalize执行deferred动作，cleanup helper实际exit1后回滚。原out01完整stdout/stderr/MSI日志、独立前后像、账户保留事实均保存，没有覆盖失败。修正测试同时要求“cleanup调度早于RemoveFiles调度”和“cleanup实际执行报错前无实际RemoveFiles/FileRemove”，并保持程序身份/摘要及所有前后字节断言。没有改产品安装器顺序、返回处理或安全门。

build02在新增JSON后像表达式缺括号时编译失败，原stderr/exit101保留；修正后build03通过。out02在原SID上先逐字验证原pending和所有保留项、用修正日志判读复核原失败事务，再显式继续Enable并仅执行一次正常/x。没有重复安装或重跑旧终端组件。independent-transaction-audit02再独立核原前后像、正常后像与两份实际MSI日志。

后继清理暴露PowerShell/C#参数绑定问题：将PowerShell `$null` 传入C# string参数得到空串，不能替代DeleteProfile要求的NULL。null-marshalling-proof.json纯绑定测试确认；修正为C#内部传null的包装器。原DeleteProfile87、CIM报告Loaded=true及精确SID服务事件保留；正确原生调用仍由Windows判断是否允许删除，不强制卸载用户hive、不重启服务或机器、不递归擦Profile。控制器另修“Profile清理未完先保留账户”，该后续脚本修正静态解析通过，没有为此重跑MSI。

## 证据与限制

attempts/out01与out02分别保存原始输出，executed01保存首版运行脚本/config，source-manifest01保存首版Rust执行字节摘要（首版Rust全文在改动前未单独复制，明确保留此限制）；executed02及source-manifest02保存最终真实执行的完整Rust与脚本/config。Git blob与Windows执行字节摘要分立，CRLF/LF差异不冒充逐字相同。最终脚本清理修正另存executed-final及最终提交清单，不冒充out02使用了后继脚本。

清理结果及新增文件字节数见result.json/generated-inventory.json。cleanup05正确NULL调用返回error32，Profile服务仍Loaded=true；Restart Manager对精确NTUSER.DAT/UsrClass.dat列出System PID4/Registry PID328、Restartable=false，未调用RmShutdown/RmRestart。Profile85文件118556840字节保留，不触动共享系统进程。候选进程无该SID仅限记录的命名session0集合，不扩为全机证明。

新Public根83文件184522112字节归档和逐字核验后，精确递归删除被自动审批blocked by policy拒绝，命令未执行，详generated-cleanup-blocked.json；没有重试或换工具绕过。原自动审批拒绝的W0失败目录、installer-w1、W1-C/W1-D生成根、schema/空目录及Public盘点根不重试、不换机制绕过。新Public根仅为本轮脚本、两份测试EXE与原日志；复用旧Public MSI/Node/PowerShell，未复制大包/资源，未改官方2.99或日常配置。没有把tasklist exit0或无输出当进程/清理证明。
