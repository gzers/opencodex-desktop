# 0.2.24 Mac停止期间的派生与执行镜像变更

revision: 1；2026-11-12。归IMP-34 P24-04，承接[恢复计划准备](restore-plan-product-r1.md)。代码main afa4fca0393c2dffeeda7070ec4b2cf548100e02；本批仅补原生单进程退出证明的变更拒绝，accepted=false，不开放实际恢复许可。

## 已复现与修正

原ExitWatch只订阅NOTE_EXIT/NOTE_EXITSTATUS。真实合成进程在订阅后fork、或exec替换镜像，再以0退出时，旧实现仍签发单PID退出证明。两个新增反例在原生产算法下均失败，原stdout/stderr与binary摘要保留。

现同时订阅NOTE_FORK/NOTE_EXEC，并在检查退出状态前处理两类变更。变更可能与退出事件合并；发现后永久失效，分别返回lifecycle_exit_membership_changed或lifecycle_exit_image_changed。重复查询不能因原事件已被消费而转成通过。当前真实Stop入口已使用同一ExitWatch，不另造仅测试生效的实现。原UID/出生代次、非零退出/信号拒绝及原截止时间保持。

Darwin SDK明确：NOTE_FORK不把子PID交付给实际kevent，NOTE_TRACK/NOTE_TRACKERR/NOTE_CHILD自10.5不支持。因此本批只拒绝已观察变更，不能用这些事件声称已保留完整进程树。订阅之前逃逸/脱离的后代、所有监听与原件写者仍待闭合；MutationPermit仍无生产构造器，未调用实际apply。

## 验证及夹具时序

首轮补丁回归6通过/3失败：posix_spawn返回后，测试订阅有时早于首次镜像执行，正例意外接到初始exec事件。失败原件保留。后继合成进程先通过管道输出明确就绪字节，再由父进程订阅；跨父进程正例改为纯Rust等待及叶进程就绪握手，避免shell循环调用sleep本身制造fork。产品算法和断言未为夹具放宽。

最终10项原生定向检查通过：正常0退出、非零/信号拒绝、错误UID/出生代次、自身/已退出PID拒绝、原预算、非法状态、真实非子进程，以及退出事件合并/主进程仍存活时的fork/exec永久拒绝。编译、格式及全目标严格Clippy通过。

只追加一轮真实隔离官方2.99受影响链，不重跑全量：Start15664ms、Status1119ms、Stop含计划登记8609ms，原30/5/10秒预算未变。正常Stop得到原内核exit0，未被变更门误拒；独立索引为RestorePrepared/revision6，plan及全部阶段摘要核对，stage原生身份一致。客户端catalog/cache/audit尚未恢复，原完整断言仍失败exit101/accepted=false，动作仍client_original_restore_pending。没有把这次启停通过改成实际恢复或整版通过。

## 留证与后继

.adg/evidence/0.2.24/macos-exit-events-r1/保留两个原算法失败、首轮夹具失败、最终结果、真实启停trace、原索引/阶段/plan/stage、源码与binary摘要、SDK边界、manifest和清理回执。日志只含自有合成配置；未留runtime secret/admin token。

完整Darwin TCP表framing核对原主端点缺席，自有运行镜像为空；精确新N/K/history数据库分别取得/释放排他锁后删除。自有程序/profile/测试binary/索引/stage及已核对的重复证据根已清理。未改日常配置、Shell、正式安装或官方包，无模型/WebDAV业务请求，未出新.app/DMG或发布。

下一步仍是完整受控集合/监听/写者证明，之后才可实际恢复、读回并关闭原会话；全客户端范围、通用渠道图、显式中断恢复等按IMP-34原范围继续。Windows独占贡献与最终W2、NAME、npm System代理、官方发布验证和固定候选保持未完，不关闭目标。
