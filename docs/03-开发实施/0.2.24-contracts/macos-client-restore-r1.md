# 0.2.24 Mac客户端可恢复原件发布核心

revision: 1；2026-11-11。归IMP-34 P24-04，承接client-artifact-policy-r1、macos-client-originals-r1、official-client-coordination-r1；代码mainfc0361bc17d52476a238d78484de3ceafa9fd149。当前仅机械发布/显式继续回退核心及合成文件原生测试，生产许可未开放、Stop未接，accepted=false。

## 本批实现

infrastructure/macos_client_restore.rs作为macos_client_originals子模块使用原FD/身份/元数据读回；只处理原已登记的catalog、models_cache及catalog-audit三件，不接受任意目标路径。Pending::prepare先加载原Original/Ready/PreStop/Stopped四阶段及独立摘要，调用已验证共享内容判定，冻结原plan/id/phase摘要/路由声明/保留PID与审计辅助上下文，不能从变化后的文件生成新原件。

在客户端目录create-only建立本plan唯一原件暂存，每个原件写入/fsync并核对原字节和uid/gid/mode/原生ACL/全部有界xattrs。原文件flags非零等暂不支持的元数据直接拒绝，不清flags凑成功；flags=0时原ACL/xattr保持验证。暂存前先写受保护preparing记录，失败的已知暂存保留；没有凭文件名猜测删除。正式restore plan、intent及terminal outcome保存于原0700/0400保护目录，O_EXCL/fsync/读回，原plan摘要及各phase摘要必须独立传入才能重开。

每次执行先核对全组三件目标/暂存/归档。未知target、同字节替换inode、第三方cache、损坏归档/额外归档对象均拒绝，不能先写第一件再发现预先存在的其它冲突。逐项操作使用同一PinnedDirectory/原dev-inode-birth/保护/字节核对和原生renameatx_np：存在目标与已知原件暂存交换RENAME_SWAP；原本缺席的对象移出目标用RENAME_EXCL；不会unlink未知客户端对象或复制覆盖已有目的地。每步后读回已知前/后两代对象，失败不写终态；原时钟字段会被本进程rename改变，因此恢复过程中按已记录对象原生代次/内容/保护识别，不把变化后的新inode当同一个文件。

Complete恢复原字节/原保护或原缺席；Rollback显式回到本plan冻结的Stopped后像，不生成新的晚到基线。中断在交换/移动之后、观察回执之前，仍可按原plan中已保留的两代对象继续，不能因没有step回执就假设未写。原后像移到受保护原事务目录保留，不删除；已完成的terminal decision不能换成另一决定。完整最终读回必须三件同时满足原件/缺席及保护。

## 生产授权保持关闭的原因

MutationPermit有私有字段，未提供构造器/default/反序列化或当前生命周期签发器。公开apply要求该未开放许可及实际官方协调Lease；本轮测试通过私有prepare_checked/apply_checked，明确为自有合成目录的机械验证，不伪造服务正常退出许可，也没有在产品Start/Stop中调用。

必须继续关闭两个关键空隙后才接生产：

1. **路径核对后rename不是预期inode条件的内核CAS。** 原生交换保留双方对象，但不能仅凭先前检查声称并发非合作写者绝不在syscall间隙改变目标。真正签发器需保证有效写者排他、原文件/祖先保护与冲突处理覆盖实际发布，不能用测试checkpoint外部编辑已被检查挡住的结果证明所有竞态。
2. **外部锁helper的借用可能过期。** check前后均验证、失败保留pending，不允许失败转成功；但不能因为开始时helper持锁就授权一个延迟写者在释放后继续写。官方锁持有及实际native mutation必须有共同执行/退出生命周期证据，不能删2秒窗口或扩大原Stop10秒来回避。

本批没有声称这两项已解决，因此不能让现有三个文件未还原故障转为通过。原生代次/private JSON/hash/审计PID均不是独立服务归属授权，生产已有服务/重启原件绑定、正常退出/Job/监听、外层摘要持久保存、活跃官方刷新、config/Grok/Agent和全客户端树仍需实现。

## 持久边界与限制

客户端与D在不同卷时，归档原生rename可能EXDEV；当前保留原件/stage/plan并拒绝终态，没有自行退回copy后unlink路径。跨卷、外部CODEX_HOME尚未验通，不作为已支持全部目录的证据。原本缺失CODEX_HOME目录也仍需正式创建/撤销合同，不能要求用户自己建目录替代产品支持。

既有outcome必须是同decision及原plan摘要，intent及plan私有保护保持。用户另改目标、暂存、归档或目录身份不能被修复/接管；任何陌生状态都保留现场。本批只归档显式固定三件的原后像，不能据此开放清理任意日常cache/audit或把审计当不重要直接删除。

## 实测

13项真实Mac文件系统测试通过，含实际ACL/xattr原字节/保护恢复、原catalog恢复与cache/audit原缺席、同inode/第三方文件冲突、不同inode同字节和symlink拒绝、整批先查未知归档、归档损坏拒绝、回调失效拒绝终态、显式回退及归档中断后继续/回退、独立plan/phase摘要重开。

其中实际子进程exit91覆盖11个中断点：intent后1、原生动作后第1/2/3、step后第1/2/3、归档后第1/2/3、terminal后1。父进程使用子进程退出前独立写出并fsync的attempt/original/phase/plan摘要重开同plan，最终三件原字节/缺席/保护一致，未换源。都是自有合成文件/原生rename，不是虚拟HashMap，但不包含官方服务或真实生命周期签发器。

初轮缺测试文件导致fmt停止、首轮投影closure E0271编译错有明确分类（原stderr未捕获）；后继Clippy range-loop及未用fixture字段错误原日志保留，无关闭lint。相关算法/反例增量修正后最终编译/格式/全目标严格Clippy通过，13项完整原生测试与最终准确源码摘要/binary SHA一致。原10项/12项中间通过结果按目录保留，不外推未覆盖的后继改动；没有本轮官方服务/模型/业务同步请求。

OS sandbox拒绝网络/日常HOME读和夹具外写入，只有自己的测试profile/原件/stage/归档可写；chmod仅作用于新建合成ACL源文件。证据.adg/evidence/0.2.24/macos-client-restore-r1/保留各次检查/失败及最终13项/真实子进程结果、来源摘要和manifest。全部自有测试进程已退出，复制binary/合成原件/stage/archive/根和核对后重复临时回执已清理，日常配置/官方包/正式安装未动，未构建新.app/DMG。

下一步先闭合官方锁和原生写入共同生命周期、正常退出/代次许可及当前源码适用范围，再接Stop全链；不能只给MutationPermit补一个布尔构造器让当前测试变成“已集成”。Windows仍沿TEAM独占Git，本批Mac机械核心不替代W0 HOME/Profile/W1/W2，也不关闭NAME、System代理、正式发布验证和固定候选项。
