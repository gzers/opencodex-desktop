# 0.2.24 当前停止会话的实际三项补偿

revision: 1；2026-11-12。归IMP-34 P24-04，承接[官方写者排他](official-writer-exclusion-r1.md)、[原父子退出](macos-launcher-exit-r1.md)和[恢复竞争保全](macos-client-restore-race-r1.md)。代码main67b2513f4。accepted=false、非冻结候选；本批首次把已评估三项补偿接到实际普通Stop，不将三项完成冒整次会话完成。

## 许可与实际调用

前批已完成本次正常Stop原服务代次/4项捕获TCP、Ready原父子normal0，以及同进程claim/N/L/K持续排他。目录实际K写者、官方连接L写者均已有真实busy及释放后写入正例。因此三项文件的许可以当前实际写者排他为基础；不必把单PID消失或磁盘outcome布尔当许可，也不伪称所有历史后代/转移描述符等完整集合已证明。

StartupSession唯一构造ArtifactRestoreAuthority，借用同一session、ProcessContext、CompletedLifecycleStop、PairExitProof、NativeLease及原动作deadline。MutationPermit只能由这个私有字段类型构造，无公开构造器/default/反序列化，不从持久回执或错误码复建，不延续到锁释放后的另一个操作。

每次机械发布/归档前沿原apply_checked调用permit.check：本Originals内存对象必须属于原session，原件/独立Live/范围、两代退出、Stop原记录摘要/上下文/原deadline、四锁pin/owner/VFS均保持；当前runtime-port.json与ocx.pid必须确实缺席。实际apply前后另外以现有OS归属提供者重新观察当前HOME为Stopped；未知或重新发布记录拒绝。所有这些检查与文件变更仍在同一NativeLease和原10秒内，不为恢复续时。

仅处理既有策略完整授权的catalog/cache/audit三件，不写config/Grok/Agent或任意整文件覆盖。计划必须是当前独立索引已登记的id/SHA，Pending::open复核全部原/Ready/PreStop/Stopped引用与准入决策，仍逐批核对来源身份/字节/安全属性。原冲突回执、迟到写者的内容保全/有条件归还、无覆盖归档、跨卷拒绝、崩溃/继续/回退等算法不改。三件已知停止后像归档到私有vault，独立原件/所有阶段保留，最终原字节/保护/原缺席读回后才写绑定planSHA的complete outcome。

许可只覆盖这三项当前已知文件操作，不是整个客户端树或完整生命周期集合的终态证书。当前Active在同一原生协调中登记plan后实际apply/readback，固定日志three-artifact-originals restored and readback verified；索引仍RestorePrepared且普通Stop仍client_original_restore_pending，Start/Restart不提前解锁。后继须把整树/已采用稳定锁释放/其它生命周期条件与成功终态同一计时接齐；本批没有缩掉这些范围或重定义总成功。

## 定向检查

61项去重相关函数通过：restore20、session9、native11、owner15、pair6（含辅助探针入口）；3个显式原包manual入口ignored不计通过。恢复核心的异常/外部实际进程竞争/冲突原字节保全、权限ACL/xattrs、12个实际突退边界等原断言保持。

首restore组19过1失败，是旧外部writer helper强制/private/tmp而默认tempdir落到/private/var；原父stderr未带子stdout的局限保留。只给同一准确binary的该失败项指定TMPDIR=/private/tmp，完整断言及conflict回执通过；已过19项不重跑。其余4组按同根执行通过。产品源码/断言未因夹具变动放宽，编译/格式/全目标严格Clippy通过。

## 首次实际三项补偿

新合成Profile、原样固定官方2.99完整供应，OS sandbox禁止日常HOME/外网读写，只允许自有根/精确自有协调DB及既有用户级L锁临时借用。准确最终build binary SHA58bb8688af9ad7aa9ff95d36e7f913b306a5be621958306d5f1406631db4dc36与上述61项及运行副本匹配。

- 原包启动/查询及Stop执行原30/5/10预算；本轮实际Stop7473ms（内部7467ms），日志包含原服务与父normal0、4捕获TCP全缺席及真实三项补偿读回完成。
- catalog原有退休gpt-5.4原字节确实恢复，cache/audit原缺席确实恢复，不能以删样例/保留残留替代。本轮7路径nativeClientRestored全部true（Codex config/catalog/cache/audit、Grok config/旧备份/Agent）。
- 独立四schema4 frame/索引revision6/plan摘要和35内容引用核对；三项实际目标为原字节/缺席，所有stage均缺席，catalog发布inode正是plan预备原件inode，三个被替换/移除后像归档字节逐件等于原Stopped帧。complete outcome0400且planSHA/decision精确一致。
- 原全链assert没有放宽：整次Stop尚不报告成功，仍exit101/accepted=false，reopened仍Stopped+Required且三门槛false。

本批没有构建新release/app/DMG，不用前批release性能代签本次新恢复路径。

## 整树诊断及正确例外

补偿后另外做只读全路径/字节/模式诊断：Grok无差额；Codex无普通文件字节/模式差额，只见原claim的事务journal已消失，以及新.opencodex-native-main.owner.sqlite。该诊断没有逐项核对ACL/xattrs/flags、内核集合或生产动作内的锁重获，不当完整树证书。

重新对照用户采用原文（开发前边界确认第171行）核清：两稳定锁例外精确是当前Codex HOME的.opencodex-native-main.claim.sqlite与.opencodex-native-main.owner.sqlite，**不是外部N/K数据库**。本轮没有请求或扩大保留例外。两文件及精确N/K另做独立OS进程竞争/释放诊断：当前进程取EXCLUSIVE时另一进程busy，释放后同一进程可取锁，文件inode/字节/0600/owner保持；但这发生在原Stop计时后，不能冒已接生产或同10秒终态通过。

下一步把正式整树安全属性读回、精确两稳定锁释放及余下生命周期条件接到原计时和持久成功关闭，不能仅把当前client_original_restore_pending改成成功来凑通过。

## 证据与清理

.adg/evidence/0.2.24/current-session-artifact-restore-r1/保留假设、首错/同binary定向修正、准确source/build/binary、原真实启停原件、独立plan/index/三项publication/archive/outcome及35内容匹配、只读整树差额和锁诊断原输出。没有runtime secret/admin token，业务请求为false。

原服务/launcher/Node/Bun/test及外部writer helper命令缺席，完整Darwin TCP表framing核对原主端点缺席；额外TCP仅引用准确产品trace。3个精确新协调DB各重获/释放EXCLUSIVE后连同自有程序/Profile/双客户端原件blob/index/stage/archive/测试副本清理；既有globalL的身份/字节/0600/owner保持且独立排他重获释放，未删除或留新sidecar。证据逐文件SHA/长度核对后删除重复临时根，日常配置/官方包未改，无发布。

当前这批真实补偿进展不等于原版全部能力闭合；整树/一般图/显式中断恢复/活跃刷新/完整集合及旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/固定候选继续，目标active。
