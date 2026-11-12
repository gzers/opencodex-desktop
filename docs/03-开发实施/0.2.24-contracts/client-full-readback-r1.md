# 0.2.24 完整客户端树读回与两稳定锁释放

revision: 1；2026-11-12。归IMP-34 P24-04，承接[当前三项补偿](current-session-artifact-restore-r1.md)。代码main c5a24573d；accepted=false、非冻结候选；已接真实Stop收尾前置，独立索引成功关闭尚未接入，不把全部前置已过变成伪造完成。

## 完整读回边界

新的TreeSnapshot::restored_matches比较完整相对路径集合、根与目录原生身份、root/entry完整Protection（uid/gid/mode/ACL/xattrs/flags）、普通文件长度和SHA。文件由已知原件替换后inode/时间合法变化，内容和保护不能变；目录身份仍必须原样。缺文件、多文件、空目录、残留stage/backup、Agent或扩展属性变化都拒绝。Grok原缺席与目录出现不同，普通Grok树没有锁豁免。

仅已采用的Codex顶层claim.sqlite和owner.sqlite可留。原claim在初始化EXCLUSIVE中记录的真实临时journal是metadata-only协调观察，不恢复它的字节；最终必须不存在。最终claim/owner的journal/wal/shm均拒绝，不把任意sidecar、cache/audit或备份纳入例外。两个锁的最终身份必须与独立Stopped帧一致，原Original帧已有锁时也必须保持该原代次。生成自有证明不会把其它临时文件变成新基线。

普通三项补偿及原NativeLease完全正常释放后，complete_readback通过原pinned client相对路径读取整个Codex/Grok树，校验原件一致；再先持有两个已存在的CoordinationFilePin，按原有inode重获SQLite EXCLUSIVE（busy_timeout0，不CREATE）。任一仍被占用、代次替换/权限未知即失败，所有raw pin先于任一SQLite连接打开，所有连接正常ROLLBACK/close先于任一pin关闭，避免POSIX record锁被另一个FD关闭误释放。VFS HAS_MOVED及pin安全检查保持。随后重读整树与Grok/稳定ID确认没有变化。无强杀、无删除锁、无改权限或超时等待。

最终create-only0400 completion-readback.json绑定schema1、原References、当前plan id/SHA、实际complete outcome SHA、Codex/Grok最终完整快照SHA、两稳定锁Identity；写入/fsync/readback后才返回不可序列化ReadbackProof。该Proof只供后继索引事务消费，当前没有从磁盘receipt自动发放成功。原服务/父normal0、当前HOME真实Stopped/原记录缺席检查在读回前后保留。Restart外层预算较长时显式min(外层,原Stop.deadline)，不能用额外时间借给停止收尾。

当前Active在同一原停止计时内调用上述步骤，固定trace full-tree protection and two stable-lock releases verified。完整进程集合/转移监听和更一般客户端适用范围未因此得到证明；Index仍RestorePrepared、clientRecovery仍Required、三动作关闭。后继须将独立receipt参考和所有仍未满条件结合后才能正式关闭，不能只改pending文案/布尔。

## 已证实的释放通知竞态与修复

最终源码第二个新ProfileStop后在native-coordination阶段拒绝client_coordination_held，索引revision4、未实际补偿，首错保留。其旧协议不含更细原因，不能断言该历史轮或更旧revision4首错都是同一原因。

定向对未修改官方发行及Desktop旧helper协议做确定实验：父发布最终release路径后延迟50ms再补nonce，helper在51ms读到size0即failed/held，hold预算仍2000ms。这证实旧request_release“先创建最终名、再写完整字节”的生产竞态，单纯多等/延长租约无效。

PinnedDirectory的新封装操作在原私有control inode下创建独占.release-UUID.tmp，全部nonce字节写完/fsync后，以renameatx_np RENAME_EXCL相对原目录发布最终release，再sync目录和fstatat核对inode/0600/owner/nlink/长度。最终名在完整发布前始终缺席；已存在目标不覆盖，中断暂存保留且不当完成，不写自定义官方补丁或续2秒租约。模块仅暴露完整publication操作，未开放私有目录FD。

初实现误从另一个模块访问private file方法的5编译错误保留，改到拥有该pin的模块封装；后续未使用Write导入lint修正为仅测试导入，均不放宽产品判定。新增2项测试验证延迟暂存最终名仍缺席、完整字节/单链接/0600、目标不覆盖及中断无final。

## 定向验证与实际原包三轮

去重47函数覆盖：新completion6、既有tree8/Grok4/session9、Stop预算1、official协调8（含原子发布2）/native协调11；3个显式manual入口不计执行通过。初结果脚本选到tree子进程的1项summary，改读取最后外层8项summary，原日志不变，未凭脚本统计造通过。初布尔表达式lint等价简化及后续import/编译首错都保留。最终编译/格式/全目标严格Clippy通过，按变更只重验受影响集合。

| 轮次 | 范围与结论 |
| --- | --- |
| native | 第一次真实补偿+整树+两锁同原秒表通过；独立原件/index/plan/35内容引用、completion及另一次只读原生快照读回匹配。整版仍exit101/accepted=false，非最终budget clamp/原子发布源码。 |
| native2 | 加原Stop预算clamp后的新Profile；Start6698ms/Status240ms/Stop4147ms，原服务及父normal0/4TCP缺席，但协调held拒绝，索引revision4，无补偿/receipt；失败原目录保留。 |
| native3 | 原子发布及最终源码新Profile，完整重核同一原样官方镜像；Start7658ms/Status362ms/Stop8074ms（内部8070ms），原30/5/10保持。三项实际补偿、七路径原字节、完整树Protection/两稳定锁重获释放/receipt均通过；索引仍revision6、collector348ms Stopped+Required、三门槛false，原完整assert仍exit101/accepted=false。 |

准确delivery build回执的binary SHA 2cab38830d0b571ea1e2a4965c65d38fb87781edc2bcd43c4b2e0da9d90eaa6a与25项定向检查、native3实际执行副本一致，不冒前批产物。最终6completion/8official/11native是25函数，8tree/4Grok/9session/1预算依赖未变的已过22函数复用，总去重47。两个真实成功轮的独立read-only测试重新读取Codex/Grok完整原生快照，其SHA与该轮completion receipt逐字相符，原件四frame/索引plan/35内容引用均核对；失败轮27内容引用和revision4完整保留。独立快照不是额外成功发证或重跑服务。

本批未构建新release/app/DMG；native3窄样本通过预算不替代一般大树/缺根/外部Agent/组合图及最终候选真实验收。Windows回交仍b2e89665d/0eef59a01，不冒已接新贡献。

## 证据、清理与未完

证据.adg/evidence/0.2.24/client-full-readback-r1/保留先写假设、全部构建/首错、确定race原输出、3轮准确source/binary及原frame/index/plan/receipt、成功轮独立最终树与匹配摘要/失败轮原现场。summary/manifest逐长度SHA核验，不含runtime secret/admin token。证据重复拷贝遇原0400 plan拒绝的工具输出边界已记，改为现有字节一致就跳过，不覆盖/修改源。

三轮自有helper/Node/Bun/launcher/test镜像缺席，三个原主端点由完整Darwin TCP表framing核对；附加TCP缺席仅原准确产品trace，不假独立重放未导出的端口列表。9个精确新外部协调DB逐个EXCLUSIVE重获释放后，与全部自有程序/Profile/双客户端blob/index/stage/archive/control测试副本和确定racecontrol清理；原globalL inode/字节/0600/owner保持，独立排他重获释放后保留、无新sidecar。所有重复临时回执根清理，日常配置/官方包/系统设置未改，无业务请求或发布。

下一步仍须独立绑定回执的索引终态/崩溃恢复、完整生命周期集合/一般图/全客户端范围等原要求；旧revision4首错不因本次确定race复现就自动关闭。WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/固定候选保持，目标active。
