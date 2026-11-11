# 0.2.24 Mac应用当前原件会话索引与实际启停接线

revision: 1；2026-11-12。归IMP-34 P24-04，承接macos-client-session-r1、completed-stop-binding-r1。实现main595850a24cf4e98235e7c1efe8f16996ae07158c。当前应用入口已自动登记原会话，文件补偿仍待完成，accepted=false、非冻结候选。

## 真实产品入口

commands/client_session_lifecycle.rs接入现有execute_lifecycle共同入口，应用/Tauri、IPC、托盘调用实际ControlledProcessRunner时参与Mac官方2.99原件会话。真实runner声明manages_client_originals；不执行真实客户端动作的确定性ProcessRunner夹具保留其明确无原件副作用合同，不能把这些夹具通过当生产已执行。Windows条件路径不接Mac接口，当前新原生Windows接线与验收仍待贡献。

新Start先在当前D登记Preparing，再用实际NativeLease及StartupSession捕获原件/创建scope，独立引用提交Captured。官方ensure及现有签名就绪门通过后，重新在真实官方协调下绑定Ready及live引用，独立索引提交成功才返回Started。整个动作沿原30秒deadline，无新秒表。已有运行记录的重复Start需要当前索引是Ready且实际原live仍验签/原生归属匹配；不捕获第二份“启动前原件”或将当前已改变客户端当原始状态。

Stop从索引读回同一Ready原引用，打开原StartupSession、重新校验原就绪Generation/HOME/受保护记录，封PreStop并提交StopPrepared。沿现有stop_lifecycle_verified的官方HMAC/能力/原生归属/manager/端口/当前HOME进程缺席及内核exit0门；返回的原Stop证明再与当前Session匹配、真实NativeLease下封Stopped并独立提交索引。原10秒总预算包含引用读取/PreStop/正常Stop/封存/索引，不能阶段各开10秒。

本批没有给尚缺的MutationPermit工厂添加布尔开关，也没有执行三件客户端原字节恢复。真实服务停止且Stopped引用提交后，动作明确返回client_original_restore_pending：“服务已正常停止，客户端配置原件恢复尚未完成；原会话和现场已保留”。公共DTO保留此准确失败码和操作文案，不转成Started/Stopped整组成功，不把原模型文件失败忽略。动作状态机在失败时保持动作前观测；下一次状态采集仍使用真实服务事实，不能从这个保留的旧门状态推断服务仍活着。

若会话/引用读取或PreStop核对失败，仍先执行一次原普通受认证Stop以免留下本HOME服务，再返回原会话错误；不创建替代原件或据此恢复未知文件。若普通Stop本身失败，其真实失败优先。当前Restart停止后同样遇到待恢复门而拒绝再次启动；这份工程实现暂时阻止未恢复原件的重新启动，直到完整补偿/显式恢复接通，不能把这种中间态作为可发布最终体验。

## 独立持久索引

macos_client_originals::session_index::Index在D/manager-state/client-session-index.sqlite保存自己的schema1，不复用或复制官方协调DB schema。index内binding固定当前uid/D/profile/HOME/client路径和原目录身份；current_session保存原ticket/单调revision/阶段/References；events由API只追加每次已提交原文档，current读回必须与同revision事件一致。原scope/Original/phase/live文件仍保存在独立保护原件目录，不能扫描该目录为索引生成新References。

独立action.lock在整个操作作用域持有，跨应用进程/CLI动作排他；每次引用提交是单独synchronous=FULL的SQLite原子事务，按照原revision和完整原文档CAS。这样进程在Start后崩溃时，已提交Captured/Ready仍持久存在，而不是随一个从未commit的长事务丢失。IPC观察者消失不能释放仍在实际worker内的Index所有者。

路径全部从D受保护目录派生，action锁openat O_NOFOLLOW/CLOEXEC/非阻塞，不截断；普通当前uid单链接600、原身份及权限/ACL/flags/xattr不变才允许使用。FIFO/目录/符号链接或已有未知mode不接管/不修补。SQLite READ_WRITE/NOFOLLOW，无未知数据库采用/自动迁移；仅原确实空白的全新schema初始化可建本项目表，已有不支持版本/内容、错误binding及sidecar保护拒绝。数据库实际VFS HAS_MOVED和原生文件pin核对真实打开inode，所有SQLite连接在原生pin关闭前结束，避免fcntl释放陷阱。

阶段只允许Preparing→Captured→Ready→StopPrepared→Stopped，异常可保留Failed；阶段引用递增不允许更改Original/scope/attempt或已保存live/hash。不完整、失败、停止但尚未补偿的会话均阻止reserve_start，不能“清空后再拍一份就正常”。本批没有关闭会话/删除原件API，没有崩溃后扫描自动接管，Preparing/Captured/StopPrepared/Stopped的显式恢复入口仍需后继实现。目录代次或HOME/profile变更不能复用旧索引。

## 验证

最终9项Index原生检查通过：实际另一OS进程全动作互斥，owner退出后可重开；子进程exit91后Captured仍持久且拒绝新原件基线；顺序/旧revision/原引用rebase/缺阶段拒绝；Failed不当空状态；跨HOME/客户端目录代次/未知DB/action symlink/FIFO/权限变化不修补；event与current原文档不符拒绝。另对真实产品Stopped现场独立进程重开1项通过，最终action锁强化仍识别原revision5、三阶段和原Original摘要，并拒绝新启动，未运行服务或改写基线。

2项当前版本化动作准入回归、9项动作DTO回归通过，含待恢复准确消息与无成功结果；编译/格式/全目标严格Clippy通过。增加的action锁普通文件/flags/保护条件和负例在最终准确二进制中实测，原已过现场读回定向补验；这些强化发生在完整产品native轮后，完整native Source与最终source-final.json分开，不冒称最终二进制整轮启停已重复验收。

本轮新私有profile携带固定完整官方2.99资源，直接调用实际process_action_with_runner/ControlledProcessRunner，**没有手动调用Session捕获/Ready/Stop阶段接口**。准确源文件/binary在native/source.json中，供应/程序归属/正常Stop保持原样：

| 项目 | 实际结果 |
| --- | --- |
| Start含自动原件/索引Ready | 15616ms，Started，原30秒；indexedReady=true，independentOriginalsReferenced=true。 |
| Status | 1108ms，实际PID50116/port55593、ready=true，原5秒。 |
| Stop含自动PreStop/真实exit0/索引Stopped | 8591ms，原10秒；普通内核exit0日志、运行/PID记录缺席；索引revision5，Preparing/Captured/Ready/StopPrepared/Stopped五个已提交事件全部真实存在。 |
| 产品动作结果 | client_original_restore_pending，result=null；准确告知服务已停止但客户端原件恢复待完成，不伪报完成。 |
| 客户端原件 | config.toml原字节恢复；catalog/cache/audit原字节/缺席仍false，Grok备份仅缺席保持。 |
| 整轮 | 原完整客户端恢复assert未削弱，exit101/accepted=false，原失败保留，不把索引通过代替恢复通过。 |

官方tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c、descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3；原固定包未改。native轮约103.539秒含真实新供应，非实际新.app/DMG验收或新OS SID。OS sandbox拒绝日常HOME读/外网，仅自有根/loopback及此私有HOME hash的精确N/K/history DB/sidecar/N采用路径可写，无模型请求/WebDAV业务同步。

## 留证及后继

证据.adg/evidence/0.2.24/macos-current-session-product-r1/保存精确私有产品索引SQLite原件、只读SQL原行/事件原字节摘要和base64导出、Original/scope/live/四阶段原件（分别匹配独立索引引用且0400/uid/单链接）、before/after字节、产品trace、准确source/binary及各项检查/manifest。自有测试/helper/Node/Bun进程组与镜像均退出，完整Darwin TCP表framing核对原主端口缺席，精确新协调DB分别独立取得/释放排他后删除；副本程序/profile/binary/锁/索引/原件与核对后的重复临时回执清理。没有日常配置/官方源码/正式安装修改，未出新.app/DMG或发布。

下一步必须让Stopped原会话经过完整原件许可/文件补偿、保留冲突和中断恢复，成功后才能关闭会话并放行新Start/Restart；不能只加“忽略待恢复”按钮或清索引凑通过。当前默认mac客户端目录必须已存在，正式首次用户目录缺席/创建撤销还未做；现存服务无对应Original/失败捕获/活跃模型刷新、跨卷/flags、全部config/Grok/Agent/全树、后代/监听等仍在完整范围内。WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新官方交付发布验证及固定候选保持IMP-34后继，不将本批当整版完成。
