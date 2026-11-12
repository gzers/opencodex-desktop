# 第二代根创建保管与原子初始登记原语

revision: 1；2026-11-13；IMP-34 P24-04；main 57ae63019。accepted=false，非固定候选。承接[闭合历史核验](client-root-closed-history-r1.md)。本批实现从准确首代RootAbsent创建第二代空目录，并原子登记其来源与Preparing；**没有接入第二代捕获、真实Start/Ready/Stop、第三代滚动或状态放行**，不把这项原语当完整再次启动。

## 每代来源和旧事实保全

Head新增可选rootGeneration，仅本代初始Preparing携带：preparingRevision、predecessorRevision、前驱完整关闭Head原编码SHA和本代root binding。后继阶段将通过独立ticket解析该初始事件，不从当前目录学习origin。当前普通Head字段缺省为None且不参与序列化，原首代事件原文和singleton binding不改；没有增第二份进度账本、没有旧格式用户数据兼容/迁移，SQLite仍user_version3和原三表。

原root_creation::References增加可选generationSlot，第二代必须等于本代canonical UUID。新证据槽固定manager-state/client-root-generations/<id>，intent/created/published和intent reference均create-only。首代固定client-root-reservation/根reference保持原样；first-only reserve/transfer明确拒绝新代遗留，不以未知槽初始化“新第一次”。当前原语只支持一个第二代槽，额外未知槽拒绝，不冒任意代数已经实现。

## 原action锁、创建fd和持久化顺序

Index::create_successor_root消费并持有原Index，要求当前完整Head精确为首代RootAbsent，当前原逻辑.codex实际缺席、data/profile/HOME与原context一致，完整闭合历史/旧archive/原件核验通过。字段私有CreationAuthority只能由该Index构造，root_creation不能从复制Head/磁盘回执重造它。

新Reservation使用原action fd的clone，共享同一open file description/flock，不释放再重获；外部真实进程probe证明创建前后都不能取锁，Index退出后才可取。原inode/Protection/action pin持续核对。新candidate为空、用户私有、同卷；保留其created fd与原deadline，RENAME_EXCL发布到逻辑.codex，不覆盖外部用户目录。创建/发布/回执与原first-root原生检查一致，未知/新文件保全并失败。

新建generation槽后同步generations与manager，记录写后同步vault/generations/manager；原publication profile/vault同步保持，数据库COMMIT前再次同步父目录。它是fsync调用与顺序证据，真实exit91只证明进程故障边界，不能冒物理掉电测试。

## 同一数据库事务登记

生成新Head时revision=前驱closed+1、ticket=新origin.id，state Preparing/refs null，rootGeneration包含新binding与前驱SHA；不改首代binding和任何旧event/Original/archive。BEGIN IMMEDIATE内先核当前旧Head、全部旧history和实际binding/旧闭合archive，再核准确新origin、空目录/created fd，才写新events行和current_session CAS。

两条SQL写完、所有测试故障边界后，COMMIT前完整resolve候选代次/旧source/旧close/archive/新scope，复核实际binding、new provenance/空根/created fd、storage与原deadline，失败回滚两条引用；提交后重新完整head及空根/fd检查。提交后才来的变更导致Err、保留已提交Preparing和外部文件，不返回有效创建权限或谎称未写入。

root_binding保留首代Preparing1校验，第二代初始事件按本代ticket从有界canonical events定位，前驱必须准确首代RootAbsent且完整关闭编码SHA相符。新scope前八项（uid/data/profile/HOME及逻辑client）必须相同，只有client identity必须是新的；本代origin/slot/ticket连接一致。读取时再次核前驱闭合归档。当前仅允许一个已登记后继初始事件，未知更多代次拒绝。重开可看见准确Preparing事实，但created fd/deadline不会从磁盘恢复，普通capture/首代capture/reserve_start依旧拒绝。

## 测试和审查修正

[36文件证据及manifest](../../../.adg/evidence/0.2.24/client-root-generation-r1/)约2.55MB；89去重相关函数通过：后继新增8、索引总44（含8后继）、根恢复37、首次创建8；4ignored不计。fmt/diff-check和最终all-targets strictClippy通过。最初Path入参类型编译失败只有chat原输出，hypothesis明确边界；unused测试import的Clippy失败原log保留，移除import不禁lint。

正例新旧root identity、attempt不同，singleton binding与全部旧vault原字节不变，新origin/Preparing同一事务出现，可实际重开读取，但普通capture与Start拒绝。新用户目录、过期/错误前驱、extra vault、未知generation槽、晚到文件、篡改前驱SHA和未登记source不能被采用。

真实exit91五点：generation reservation后、root published后、events插入后、current Head更新后、COMMIT后。父实际wait91；前四点独立DB只能看到完整旧RootAbsent5，提交后为完整Preparing6/原子rootGeneration。事务内两点保留rollback sidecar；测试只对原bytes作immutable只读投影，不以自动SQLite恢复覆盖失败。磁盘重开不恢复创建fd或捕获许可，新root/未登记槽原样保留。

在beforeCOMMIT、eventInserted、headUpdated、committed四点分别真实加入新root文件或改旧archive（8组合），提交前一律回滚旧Head，提交后保留Preparing但返回Err，外部字节保全。注入操作先unwrap/assert成功，不靠故障注入本身报错满足negative result。外部进程flock检查保持同action锁直到Index退出，非tasklist或文件存在推断。

只读审查发现的“SQL后尾部完整校验缺失”和“两层父目录未sync”已按上述顺序修正并补负控；最终静态确认仅覆盖这两项闭合，不代签执行或再次服务启动。早期4/7场景通过日志不冒最终提交尾部来源。

source-final固定最终10源SHA与实际执行库binary；verify_raw逐字解码五崩溃现场JSON/SQLite，核旧闭合source/plan/Original/scope/completion、新intent/created/published摘要链、根新身份及原子Preparing对前驱完整原文SHA、scope前八项相等。离线副本只补字节证据，不冒活体目录、Stop或Start许可。

## 清理与下一步

本批不执行官方runtime/服务或全局协调DB，没有日常配置、真实WebDAV/模型请求、新.app/DMG/Windows候选或发布。所有test/exit/probe和子审查句柄结束，TempDir现场清、unitTMP空；原字节归档逐字核验后清理重复临时根，工程target留存。

下一批将第二代初始revision、origin/scope解析与活体created fd接入专用capture reservation/Original登记和阶段链；历史审计扩展为各代独立关闭及当前reserved目录的明确边界，旧archive仍保留。随后统一只读后继资格驱动状态和实际Start，跑至少两完整release原包Start/Status/Stop和中断/CAS/新用户root/原deadline负控。第三代及更多循环不能用当前“第二代原语”冒接受；当前真实Start仍走原first-only拒绝，RootAbsent状态UI/中断恢复未完。Windows W1/W2、NAME、其它客户端/代理/发布验收等完整目标保持。
