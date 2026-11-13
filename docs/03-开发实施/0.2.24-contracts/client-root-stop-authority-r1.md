# 当前正常 Stop 与独立登记绑定的根恢复许可

revision: 1；2026-11-13；IMP-34 P24-04；main fed7d21ad。accepted=false，非冻结候选。承接[整根临时对象事务](client-root-transient-r1.md)。本批新增生产许可构造与原会话绑定入口，尚未从完整缺根产品Start/Ready/Stop状态机调用或做原官方整链验收，不把静态接线与下述隔离谓词检查当产品通过。

## 唯一正常停止来源

RootStopRequest只是本动作的借用输入，不授予权限；字段包括原StartupSession/ProcessContext、CompletedLifecycleStop、PairExitProof、当前NativeLease、source Head与原动作deadline。admit_root_completed_stop在创建config Guard之前，核对原根Absent来源、Stopped或RestorePrepared、相同当前refs、完整Stopped帧链、scope/live与当前程序来源，要求真正本次normal Stop/服务和launcher正常退出/同记录摘要，并完整OS absence查询，之后才生成字段私有、无serde的RootStopAdmission。已有Stopped观察、缺Ready或缺Stopped、旧回执、复制PID/摘要/普通bool都不能生成该对象。

完整query不能在Guard创建后执行：自己的Bun holder带相同HOME，也会被正确判为占用。准入后持Guard阶段使用同一admission绑定的当前session/scope/live、lease、正常Stop、进程对和监听检查；Guard创建/正常释放仍归实际官方helper。结束Guard已正常release后，再做完整OS absence及当前绑定核验。该先后不是忽略helper占用或放宽扫描，而是只在原借用许可/排他锁内执行受限阶段。

## 停止与监听事实持续有效

require_original_absence复用具体owner的固定context，与原Stop记录digest、服务generation、native NormalExitProof及捕获TcpListeners绑定。检查原service/launcher无进程、主端口/manager/运行记录缺席，再对捕获过的全部TCP监听端口逐一核对；secondary重新占用或任一查询未知都失败。caller没有新预算，deadline始终min(原Stop deadline, caller)，Restart不能借出额外时间。原Scope、Live和四帧仍是事实，只与活体Stop/Pair/lease同时匹配才有本次许可。

## 强制独立索引先登记

prepare_root_completed_stop接受已有admission和准确Guard，再走原正常OFD交接/marker绑定计划；登记由调用方持有Index完成。retire_root_completed_stop必须接RootRetirementRegistration，不能仅交旧source Head或计划JSON。

registration字段私有、无serde，实际借用当前受action锁保护的Index。只从当前RootRetirementPrepared构造，require核对sourceRevision、plan id/SHA、ticket/refs/原restorePlan、当前Head完整代次，并从events表读取准确原source revision的完整文档与source逐字段相同。复制Head未真正登记、新摘要、不符source状态、已关闭/改变的索引一律拒绝。移动每个权限checkpoint均再require该登记；完成前不能丢失原归档定位引用。

## 移动后继续核对原会话

RootRetirementAuthority只在会话模块内由admission/当前registration创建，字段私有、无serde；root_retirement的唯一生产工厂只接受该具体类型，不能把任意callback转换成生产MutationPermit。测试仍有明确合成callback路径，其存在不冒正常停止权限。

校验按不可变plan打开准确live目录或准确archive，原logical client路径不改，不再沿已缺席旧物理路径读帧。独立原件/Scope/Live/Original→Ready→PreStop→Stopped链与归档字节均复验，scope/current program root/selection、generation、Ready摘要和原refs都须一致。当前NativeLease/owner/context/正常Stop/pair/全部捕获端口、runtime-port/ocx.pid缺席和原deadline在checkpoint持续核对；helper释放后完整查询收尾。它不补历史、不从现在文件反算采用新baseline，不扩大进程家族覆盖声称。

生成AuthorizedReadback仍须原guarded事务完成原目录移动、官方normal release及精确released tree，当前factory返回后才供同动作Index关闭。完整产品编排还未接，不能去掉现有Ready/capture guard或靠磁盘事实开放next Start。

## 检查与证据

[22文件manifest与核验脚本](../../../.adg/evidence/0.2.24/client-root-stop-authority-r1/)保留最终8文件SHA与实际执行库binary 131d71c2414e6451fdcf6361e9f27789cf38a149821944ff3d3c80baef8c5edc。32个去重相关函数通过（root-stop4、completed-stop7、root-movement23，有2重叠），fmt/diff-check和all-targets strictClippy通过。

真实子进程绑定IPv4/IPv6各一监听并由OS捕获，正常退出0/父实际wait；该内核事实进入显式合成的外层Stop谓词夹具，不进入生产factory或冒官方runtime。测试证明全部捕获端口被检查、次要端口/未知socket查询/manager/记录重现/owner变化拒绝；已有Stopped无fresh proof、错误digest/代次/context/期限不授予恢复。根binding测试使用明确合成Ready/Stopped/PID事实，证明移动前后原logical scope/plan/帧链仍核对，修改任一历史帧、scope/live、期望来源、摘要或新增用户根都拒绝。它不能证明官方Start/Stop已执行。

独立registration测试在实际SQLite账本中证明未登记的Head副本拒绝、原source event必须逐字段相同、准确登记在同root移动后可核验、关闭后旧凭证拒绝。已有22项根移动/写锁/中断回归复用，本轮无新官方包/服务执行。失败日志保留：跨测试模块Fixture私有导致编译拒绝，仅调整测试可见性；Clippy参数过多后整理为借用request和owner固定context，没有禁用lint。早期3项通过日志不冒称覆盖后继registration改动。

只读审查找出并复核修正上述Guard扫描顺序和强制登记两点，不编辑/运行测试，不代签产品验收。子审查已结束；测试进程正常wait、TempDir与unitTMP清净，证据逐字归档后删重复临时根。无日常配置、全局协调DB、官方运行包/服务、真实WebDAV/模型请求、新.app/DMG/Windows候选或发布。

## 下一步

将原根缺席capture/reservation与StartupSession/Active接线，受明确来源/当前证明约束地接Ready、PreStop、Stopped、恢复计划和RootRetirementPrepared/Absent；同一动作先admit、再Guard/plan、持Index注册凭证移动并在原Stop预算内关闭，处理next Start与明确中断入口。之后新隔离Profile运行原官方2.99完整缺根Start30/Status5/Stop10/Restart30，保留失败。当前新factory未由该产品路径调用，所以P24-04/05仍未完成。Windows W1/W2、一般客户端图路径/完整进程集合与刷新/NAME/System代理/发布验证/README/固定候选等原IMP范围保持。
