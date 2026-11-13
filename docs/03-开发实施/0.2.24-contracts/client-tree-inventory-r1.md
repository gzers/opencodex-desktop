# 0.2.24 原件阶段绑定Codex客户端树清单

revision: 1；2026-11-12。归IMP-34 P24-04，承接[原件捕获](macos-client-originals-r1.md)、[恢复计划](restore-plan-product-r1.md)及[只读状态](recovery-readonly-view-r1.md)。代码main 94ed6dc11（独立启动诊断51c1f7dbf）。accepted=false、非冻结候选；本批是范围清单，不是实际文件恢复。

## 已接入产品的范围

Originals::begin及每次Ready/PreStop/Stopped封存都捕获当前.codex树的文件身份/权限/ACL/xattrs/长度/内容SHA、目录及空目录、原始相对路径。普通文件内容流式SHA256，64KiB临时缓冲zeroize，不把这些内容复制进树字段；已有三件catalog/cache/audit的原字节快照保持。原件帧schema2新增必需tree，原Original摘要及各phase独立摘要因而也绑定树字段，不能用后继磁盘重建旧树。原开发schema1明确拒绝，不兼容/自动补齐/迁移。

普通文件内容不存入tree，不等于“所有凭据永远不会入日志”：私有帧仍有真实路径及保护属性/xattrs，属性本身可能含敏感数据；没有全局脱敏保证。本批公开证据仅明确合成配置/目录，没有用户实际auth.json原件或完整配置凭据。树内容摘要不能代替可恢复字节备份；Grok、Codex外的Agent根、全树恢复写回还未实现，不能将原完整范围缩成只有一个.codex清单。

逐层保留目录FD，openat/O_NOFOLLOW/O_NONBLOCK打开普通文件，独立openat(.)/fdopendir/readdir读取原目录游标；不会用绝对read_dir跟随被替换的中间目录。每个条目fstatat no-follow与已打开FD身份/长度/mtime/ctime/链接数及保护前后匹配，目录条目集合/版本前后匹配，整树做两遍一致性检查。字面反斜杠在POSIX保持原样，不改成斜杠以致路径碰撞。新文件、同字节新inode、读取时改写/目录替换、无权限、symlink/hardlink/FIFO/特殊文件均拒绝，不把不读当缺席；已有普通多链接对象在open之前拒绝。该机制不是原子文件系统快照，也不证明任意同UID持续恶意写者的隔离。

边界为每文件16MiB、全体普通文件128MiB、16384条目、64层目录、路径总1MiB、保护元数据总4MiB；超界明确失败、不截断或静默跳过。此限制仍可能拒绝真实大Codex数据库/日志或有效链接目录，需要后继按完整范围处理，不能标整版全部客户端已支持。

## 协调数据库的特殊身份

原官方claim及其精确-journal/-wal/-shm名字仅登记stat原生身份/owner/mode/flags/大小，明确kind=coordination，没有内容摘要/ACL/xattrs/可恢复字节声明，也不能被清理或恢复引擎当普通文件。它们的真实原生锁准入仍归原NativeLease，没有给目录扫描授予新权限。孤立sidecar、错误用户/模式/类型或额外冒名均拒绝。这样的区分是为了不open/close同进程SQLite已持有锁的inode；不能将其扩成任意cache/audit忽略例外。

首轮真实产品在原件前失败。只读检查保留现场的树可读，因此不先认定遍历坏；第二轮新增固定begin阶段日志，定位capture-original。新单项用空官方claim执行BEGIN EXCLUSIVE，实际生成rollback journal并精确复现旧“sidecar拒绝”。此前已初始化数据库的锁保留测试未覆盖这个首次状态。修正只把这三个精确sidecar按协调身份记录，保留锁及原文件，没有造数据库或改官方源码。空claim+journal捕获后另一个OS进程BEGIN IMMEDIATE仍SQLITE_BUSY，正常释放后才可取得锁。

## 验证

88项相关检查通过：8树（另现场只读inspect入口明确ignored）、21原件、20恢复机械、9会话（1原包技术入口未跑）、20索引（1旧现场入口未跑）、10原生锁（含一个无参数探针入口，不当独立反例；1原包协调入口未重复）。嵌套子进程probe日志不加算测试数。真实其它进程将已打开中间目录移走并替换为外部symlink，捕获拒绝、原对象/外部文件原字节保持；普通文件晚改、literal路径、深度/大小/损坏清单及树与三件快照的SHA/身份交叉绑定均覆盖。最后另加ordinary-hardlink在open前拒绝及测试改名，最终8树项增量通过，所有源码全目标严格Clippy/编译/格式通过；不重复无变更整链。

前期开发首20项7通过13失败为目录链接数错误；第二次13通过7失败为内部schema1残留检查；修正后20项通过，原日志留存。描述符版本首替换目录反例实际拒绝，但断言期望“file”而非先发生的“directory”错误，保留失败后更正为实际拒绝点，没有放宽通过条件。空claim+journal反例首轮真实失败与后继成功全部留证。早期工具语法/rg正则错误仅会话输出，未伪称有独立stderr。

## 三轮真实产品结果

| 轮次 | 结果 | 原件与状态 |
| --- | --- | --- |
| native1 | Start3852ms，在原件capture前拒绝，未启动服务；原Preparing/revision1保留 | 树释放后只读inspect3条有效，不重建基线。 |
| native2 | 新profile，同一完整核验镜像；固定阶段日志定位capture-original，原Preparing/revision1保留 | 由独立空claim实验证实首次journal拒绝。 |
| native3 | 修正后新profile，Start13918ms/Status1087ms/Stop含计划8402ms，原30/5/10秒不变 | 真ready、原exit0、四个schema2树帧与三件快照绑定/独立原摘要匹配，RestorePrepared/revision6、plan及原stage身份读回。 |

第三轮四phase分别树条目4/10/9/7，普通字节210/7958/7958/3853，coordination条目2/2/1/2；原config.toml确实入树。实际恢复仍只有三件机械核心待授权，catalog/cache/audit均未还原，原完整断言exit101/accepted=false。没有把“原件范围清单已登记”当“文件已恢复”。

三轮重新创建的collector均得到client_recovery=required，但运行态仍loading；第三轮787ms查询返回未知，reopenedRecoveryGate=false。路径与端点正确、启停仍门控，不把这个失败改通过。该状态读取问题仍需定位；之前不同批次的正常Stop后revision4中断也是独立未归因问题。新固定日志只帮助定位本次capture失败，不能宣称也解决所有历史登记失败。

## 留证与清理

.adg/evidence/0.2.24/client-tree-inventory-r1/含各版首错/最终日志、原生测试FD/SQLite实际probe输出、三轮独立原索引/阶段/tree/plan/stage与匹配表、准确源码/binary、manifest及cleanup。第三轮真实源早于最终普通hardlink预检查，最终源增量8项和全检查单独记录，不冒充同一源码。三轮原证据互不覆盖，两个失败会话没有重拍原件。

Darwin完整TCP表framing确认原服务主端口缺席；自己的测试/helper/Node/Bun镜像为空。三轮精确新N/K/history数据库逐件取得/释放排他后删除；自有程序副本/profile/原件/stage/索引/binary、匹配后的旧临时日志及重复证据根清理。内核隔离拒绝日常HOME与外网，只开放loopback、自有根和精确新协调路径，没有模型或真实WebDAV请求。未改日常配置/Shell/正式安装/官方2.99，没有新.app/DMG或发布。

下一步是实际可恢复的完整范围/字节源与进程集合/监听/写者许可、显式继续/回退，以及本轮reopened状态失败。Grok/外部Agent、首次目录/跨卷/flags/通用路由图，WindowsW0 HOME/Profile/W1/W2、NAME、复杂System代理、官方交付发布验证和固定候选仍按IMP-34继续；不关闭目标。
