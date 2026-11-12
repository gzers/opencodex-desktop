# 首次客户端目录的创建归属原语

revision: 1；2026-11-12，IMP-34 P24-04，main f2c1d1c75。accepted=false，非冻结候选。本批是缺根启动所需的前置能力及拒绝边界，尚未让产品支持缺根启停。承接[首次客户端状态](client-first-use-r1.md)与[独立原件索引](client-capture-reservation-r1.md)。

## 为什么先做归属

当前Active.capture先canonicalize(profile/.codex)，索引binding和原件Capture也要求这个目录已存在。直接mkdir再调用现有捕获会让Original看起来原先就是目录，丢失原缺席事实。目录创建和启动之间还可能中断或出现另一进程创建的目标，不能按同一路径收养。新原语先记录真实原缺席，再创建、保留原生身份；生产调用暂不启用，等待与独立索引同锁转交及完整恢复合同一并接线。

## 原生文件系统原语

root_creation::reserve只接受已存在、路径干净无链接代次变化的DataRoot与profile；DataRoot要求私有，profile要求本uid/不可被group或other写/无flags且保护代次保持，Codex目标与数据事务根不得互相包含。先确认profile/.codex确实不存在，打不开/链接/已有目录或文件都不当缺席。使用同一个manager-state/client-session-action.lock，与现有Index完全同action_pin身份/Protection检查和flock，持有到PublishedRoot凭据销毁或后继真实转交，第二进程不能同时拥有。

已有session index/sidecar、client-originals、root intent引用或reservation槽都拒绝新建，不修旧文件或迁移。固定client-root-reservation目录用mkdirat排他创建，不接受已有空槽。intent.json和client-root-intent.json引用create-only0400，包含schema/purpose、nonce、uid、DataRoot/profile路径与原生identity/profile保护、originalRootAbsent=true和独立SHA，落盘后才有candidate。

candidate在私有事务槽中mkdirat0700创建、打开并保持原fd，必须为空并与profile同卷。created.json绑定intentSHA和候选root identity/Protection，再同一action锁下重核候选/父根/目标缺席。renameatx_np(...RENAME_EXCL)发布到profile/.codex，不能替换并发出现的外部目录或文件。父目录/事务目录sync后，新的目标PinnedDirectory必须与保留fd一致且仍为空、安全，再写published.json绑定intent和createdSHA。

PublishedRoot不实现反序列化；references只返回当前进程内完整复验后的事实引用，不能通过读磁盘获得执行许可。每次检查原fd/当前路径的identity、安全属性、动作锁、原引用、上下文和deadline；目录内容已变也拒绝在旧确认上转交。任何失败保留记录、candidate或已发布根，不猜路径删除、修权限或默认继续。此模块没有service/native Stop权限，未拍Original，也未授予根删除/恢复许可。

## 生产拒绝边界

虽然新建路径尚未接Start，普通Index.open与只读inspect已识别client-root-intent.json或client-root-reservation存在就拒绝。这样未来或中断留下的新.codex不会因“现在存在”被普通捕获当成旧配置。只读失败不建index、不生成Original、不改原记录；原unknown拒绝和早期捕获流程保持。action_pin只扩大到macos_client_originals模块内部可复用，未放宽其uid/mode/ACL/flags/xattrs或身份校验。

## 验证与证据边界

[16份证据与manifest](../../../.adg/evidence/0.2.24/client-root-custody-r1/)保存假设、编译检查、源码/测试binary摘要和崩溃记录。最终root8/index27（2个显式manual ignored）通过，fmt/check和all-targets strictClippy通过。首fmt早于测试文件写完，报缺macos_client_root_creation_tests.rs，后补齐完整测试；该观察在hypothesis与chat工具原输出保留，不声称有另存的原始日志。

正例证明原缺席引用早于任何候选/目标，发布后仍没有session index或Original frame；第二个真实OS进程在同flock被拒绝。已有目录/文件/悬空链接、原生发布前并发创建目标、外来candidate、旧index、改profile权限/引用、创建或发布后换目录、非空/过期分别拒绝，不清理第三方输入。

4个真实child exit91分别在intent发布、created记录、原生发布、published记录边界：前两处.codex仍Absent，后两处已存在；父进程wait实际91，重新调用拒绝且记录文件字节/身份/mode保持，没有自动收养或新index。原日志打印合成现场各文件的rawUtf8、identity/mode/长度/SHA，verified-crash-evidence逐一核对4场景原JSON摘要、引用关系和rootPresent边界。记录清单是文件证据，不冒称一次完整目录安全属性读回或服务退出证明。

所有夹具只在本批TMPDIR，所有子进程实际wait完成，TempDir负责清理各自目录；unitTMP已确认空，归档逐字核验后删除重复证据根。无官方运行包、服务或GUI执行，无全局协调DB/日常HOME/业务数据修改，不运行真实WebDAV或模型请求，未构建新.app/DMG或发布。

## 接下来必须完成

1. 在不释放现有flock的情况下，把进程内PublishedRoot归属和引用交给独立生命周期索引。不能只是释放后重开同路径，不能用磁盘记录代替原句柄，更不能把新根拍成“原来存在”的Original。
2. 独立登记原根Absent与实际创建root身份/引用，并接真实启动、正常Stop后的原根状态恢复。原两稳定锁和所有客户端原件仍需当前原生安全证明，不能把新根当额外永久残留例外。根消失/被替换时的索引只读和后继新启动也需明确合同。
3. 把创建/登记/原件捕获间中断接入明确可审查恢复流程；不能自动收养本批无独立生命周期引用的残留，不能用局部文件存在猜服务归属。
4. 上述接线后，才在新隔离Profile按原官方2.99跑有假设的真实缺根Start30/Status5/Stop10、完整原状态读回/状态重开，并保存失败、清理残留。本批8项原语通过不能代替这一验收。

缺Ready、一般客户端/图/路径/完整进程集合/活跃刷新、旧revision4首错、Windows接线/NAME/System代理、官方交付验证/README和固定候选W2仍按IMP-34继续，目标未完成。
