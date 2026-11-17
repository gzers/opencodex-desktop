# TEAM17／IMP143／r7：只读工具阶段与构建环境阻塞

本批源码 73d8eb155ae474940cecf4d8933c41d924b29a83，代码分支 codex/024-windows-native。已合入 main 89e0195acb7a0c6e7549eafa376ec520ce981546；文档分支合入 docs/governance-main 55e1a4a7ebca93fc220b79e6f55b466cd9104839。完整19项状态见 status.json；本批不是R0通过或恢复批准。

## 待证伪假设及本批结果

1. 原r1归档可否代表r2：不成立。本批精确归档范围另含r2 I/D/H、两custody、客户端全树、三协调文件、原Public输入输出/locator及原始setup；尚未实际运行，不复用旧ZIP签通过。
2. 查询新观察句柄是否无需终止权限：辅助探针通过。C#仅打开QUERY_LIMITED_INFORMATION|SYNCHRONIZE，未导入TerminateProcess；同代进程观察、错误FILETIME/映像摘要、ADS及多链接拒绝均在PS5.1的r4自有夹具验证。它不产原Owner/Job/恢复权。
3. 实际对象与既存8投影/字节/children是否一致：仍待普通桌面实测。固定19对象/25原记录从当前r2原始接收件生成，Rust观察器复用现有native_stamp；未把固定JSON当native能力。
4. 现宿主是否可完成MSVC定向编译：当前不成立。lib test在分配2097152字节时失败，raw101；较小的非测试产品构建另在LLVM阶段out of memory，raw101。没有语义报错不能当作编译通过，旧产物不能冒新制品。两轮原日志分别保留于build和product-build。

## 实现及验证边界

新增debug-only、版本明确且只接受before/after的只读入口。生产公开Start仍拒绝；不接旧pipe、不调用cleanup、不改当前r2对象。只读行逐项记录独立错误，全部所含8投影要求不变，审计SACL仍ExcludedUnknown；120秒是独立只读观察上限，不是Start/Status/Stop/恢复预算的延长。

PS5.1辅助探针r4通过，原始r1/r2/r3失败回执保留：r1系统PowerShell多链接被正确拒绝后改自有单链接cmd副本；r2测试null字符串封送为empty导致FILETIME对照错误，修正仅影响探针入口；r3旧.NET的ADS路径写入不支持，改PowerShell原生Stream操作。未放宽实际进程、流或链接断言。自有探针进程均普通退出raw0；三份复制的系统输入映像移入代码树ignored target保留，逐文件摘要与新位置见probe-input-retention.json，未提交系统二进制。小型自有ZIP及日志保留，未删除失败证据。

新归档/预检工具尚未向Public部署，也未让marenben运行；缺少成功编译的新观察器EXE。PowerShell三个入口解析、C#真实查询/反例、Rust新文件rustfmt静态检查和19/25清单结构检查通过；Rust单测未执行。generated schema原始输出已归档，tracked schema恢复HEAD，untracked schema移出代码工作区。

## 当前阻塞与下一步

环境快照见environment-blocker.json。首次故障后可用提交空间约4.7GiB，root test与非测试产品都发生真实内存耗尽；不修改页面文件、机器策略或结束日常进程，不在相同前提下重复空跑。需要用户自行释放日常应用占用的内存/提交空间，或提供同等授权的Windows构建环境。此前提改变后先构建并定向检查观察器，再一次交付原普通桌面的全r2归档及前后预检入口。

repair-plan-draft.json只列准确定位及顺序，executionReady=false、approvalRequested=false；不是可执行恢复计划。须收到本r2全归档、实际native8/完整源/current/所有节点、从未Resume证据组合及原Job历史边界，并完成修复器合成/中断/未知反例，才向用户一次询问精确执行确认。旧50580/54692授权不延伸至21500/75744。已加载同image只有cleanup dispatch而无只读Status客户端；该原Job当前观测缺口明确保留，未用新image尝试绕过认证。

当前21500/75744、所有工作文件/原custody、旧seq9和历史40588均保持，未force、未恢复、不签NormalStop/Closed。未来宿主beforeResume原生接口、严格schema负例及共享单次inventory优化尚未完成；原扫描成本证据仅复用既有副本计量，不声称当前Start超时的唯一因果。原件未还不再运行原包；不W2、双SID、发布，不改官方2.99、日常PATH/配置/策略。
