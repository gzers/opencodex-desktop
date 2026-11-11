# 0.2.24 Mac服务原进程正常退出证明

revision: 1；2026-11-12。归IMP-34 P24-04，承接native-client-coordination-r1及macos-client-restore-race-r1；代码main57c201bd77e9e08aa6b78773e29daf91d50d885b。当前已接Mac普通Stop的原进程内核退出门，完整客户端补偿仍未接，accepted=false、非冻结候选。

## 产品变化

infrastructure/macos_process_exit.rs新增Generation、ExitWatch和不可序列化NormalExitProof。Generation只观察明确PID的真实proc_pidinfo，要求当前实际uid、非僵尸、有效原生出生秒/微秒；错uid/错出生代次、自己进程、已消失/无法读取均拒绝。ExitWatch在停止前以kqueue EVFILT_PROC订阅NOTE_EXIT|NOTE_EXITSTATUS，注册回执及注册后第二次代次必须一致；不从进程名搜索、不发信号、不waitpid/reap服务或其launcher。

正常证明只来自同一订阅的真实PID、opaque cookie、proc filter、NOTE_EXIT/EXITSTATUS、非错误/EOF退出事件；native wait状态必须WIFEXITED且exit0。非零码/信号退出、未退出、注册前消失都不当正常退出；同observer可保留已取得事件，但每次检查仍受原动作绝对deadline约束，队列里有迟到成功事件也不能突破预算。Proof只证明这一个进程代次，不能证明所有后代/端口/HOME/客户端原件恢复。

commands/runtime_recovery_v2_owner::stop_lifecycle在原归属/记录/服务管理器检查后、官方POST之前订阅原服务PID；期望uid/出生代次来自刚读回的现有OS Host身份，原后继checked_owned再核对仍保留。后继官方同连接HMAC/能力/第二次验签/记录核对/正常teardown全不变。只有原进程、记录、当前HOME其它进程、原端口及服务管理器均证明缺席后，再要求内核正常退出事件；成功日志固定为stop-proof: original-native-exit confirmed code=0。没有删原Absent/Unknown约束，也没把端口消失替代正常退出。原30/5/10秒及Stop合并秒表保持。

Windows条件编译路径不变，不在Mac冒Windows验证；Linux等未支持平台不从此门获得额外授权。NormalExitProof不是整组生命周期MutationPermit，恢复签发器仍无生产构造器，不可用此单PID证明跳过控制集合/监听/原件绑定。

## 验证

7项真实Darwin退出检查通过：自有受控shell的exit0、exit7、SIGTERM区分；活进程不提前成功；错uid/birth/self拒绝；已退出不追认；原截止时间已过不接受队列或缓存成功；非法wait状态；实际同uid非直接子进程也取得内核exit0，未抢占launcher的waitpid。信号只用于该自建shell反例，不用于官方服务或凑正常停止。7项入口原生结果保存；新增固定日志后没有修改该内核算法。

原有10项归属回归通过：用户/创建时间/image/argv/HOME/记录/版本来源、当前uid进程扫描、端口事实、未知/外来记录、其它HOME程序占用等保持；原另一个attested模拟服务手动项本轮未重跑，不计通过。最终编译/格式/全目标严格Clippy通过。

对改动过的实际产品Stop另做一次原样官方2.99/private profile完整启停受影响检查。固定descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3、原tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；生产process_action_with_runner、RuntimeHandle、实际官方ensure与Stop、原5秒状态采集均真实执行，没有注入服务owner/exit成功桩。

| 项目 | 结果 |
| --- | --- |
| Start | 4433ms，Started且真实ready，原30秒。 |
| Status | 1092ms，同原PID43304/port61945，ready=true，原5秒。 |
| Stop | 2392ms，Stopped；生产日志出现原进程内核normal exit0证明，运行/PID记录缺席，原10秒。 |
| config.toml | 实际启动时改变，停止后原字节恢复。 |
| catalog/cache/audit | 三件原字节/缺席仍未恢复，原完整断言保持failed。 |
| 整轮 | exit101/accepted=false，不拿Stop局部通过替代整组客户端恢复。 |

该轮约84.682秒含真实供应，唯一输入来源见integrated/source.json及binary SHA，最终提交与实际产品算法字节一致。不重跑此前无影响的协调/窗口/安装器，仅沿同轮检查并保留known失败；five before/after-start/after-stop原字节本次已实际捕获，不拿上轮缺原字节证据代替。Grok备份仅原缺席保持，无Grok改变后恢复宣称；非新OS账户、不当双SID验证。OS sandbox拒绝外网/日常HOME，只有loopback、自有profile及精确三类协调DB摘要/sidecar/N采用临时路径可写，无真实模型、业务同步、日常配置或官方源码改动。

## 证据、清理和后继

证据.adg/evidence/0.2.24/macos-normal-exit-r1/含最初两种原生API实际回执探针、7项内核检查、10项归属回归、编译/Clippy、正式原包一轮全部结果、当前原件/后像、原生成功日志与source/binary摘要，manifest逐项核对。唯一非直接子进程fixture PID也经精确PID只读查询确认已消失；服务/测试/launcher自有进程组及实际镜像为空，完整Darwin TCP表framing确认原主端点缺席，各精确新协调DB取得/释放SQLite排他后清理。副本程序/配置/binary/辅助目录及核对后的重复回执全部移除，未动正式安装、日常配置或其它账户对象，未出新.app/DMG或发版。

下一步把这份正常退出代次证明与启动前Originals、独立phase/plan摘要、当前HOME/程序和完整恢复编排绑定后，才可签发生产补偿许可。单PID exit0不替代后代/其它监听退出/完整恢复；config/Grok/Agent/全树、活跃刷新、外部写者冲突保全、新目录/跨卷等适用范围继续。WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新官方发布验证及固定候选仍在IMP-34内，整版目标不关闭。
