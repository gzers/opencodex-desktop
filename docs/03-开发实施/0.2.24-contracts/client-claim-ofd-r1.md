# 原始 claim inode 的 OFD 排他原语

revision: 1；2026-11-13；IMP-34 P24-04；main 94c897bbf。accepted=false，非固定候选。仅实现原生锁原语，未接生产NativeLease、官方写锁重定位、缺根启停或下一次Start；不能据此构造root MutationPermit。

## 描述符生命周期

macOS的F_OFD_SETLK绑定open file description，whole-file F_WRLCK与项目实际链接的rusqlite事务互斥。无阻塞等待、无重试、无不支持时的POSIX降级。准备阶段仅打开已存在的准确claim文件：O_RDWR/NOFOLLOW/NONBLOCK/CLOEXEC，私有父目录、当前uid、0600、单链接、无未知flags/ACL/xattrs均核对，不创建/改权限/修复文件。

prepare必须先于本进程的claim SQLite连接，且原fd所有者须活到全部相关SQLite连接正常关闭之后。acquire方法借用原对象，不打开或关闭同inode额外fd；本进程原SQLite事务未释放时获取OFD失败，但prepared fd不被销毁，旧SQLite锁继续排他。完成正常rollback/close后才可获得OFD；期间N/L/K与官方配置writer等保护仍须后继owner负责，不从本原语推导它们存在。

字段/fd私有、无serde、非Send/Sync；check_at只核对当前实际parent与最初目录identity/Protection、原fd与相对claim路径identity/Protection/单链接一致，不能拿相同字节的新inode或新逻辑根冒充原对象。调用者负责由原不可变movement计划授权物理anchor，本原语不移动目录，也不依据路径事实授予生命周期权限。held对象期限不可延长；到期拒绝操作但继续持锁，显式finish或drop才释放，finish即使检查失败也执行清理但仍返回失败。所有权进程实际退出时内核关闭fd释放锁。

先前普通POSIX锁实验中的额外fd关闭风险仍真实；OFD原语经独立负面场景证明不受该类关闭影响，不能把此结果扩展成旧NativeLease的POSIX连接也安全。生产默认路径尚未替换，原VFS检查保持。

## 验证

[原日志、来源与manifest](../../../.adg/evidence/0.2.24/client-claim-ofd-r1/)保留最初8项及最终9项结果；最终为8个实质案例及1个独立SQLite子进程探针入口。每个探针使用本项目实际编译的库测试宿主、rusqlite和bundled SQLite，不依赖上一轮Python实验。

实际独立进程证实：目录移动前后BEGIN EXCLUSIVE均忙；额外File关闭和本进程另一个SQLite连接关闭后依旧忙；原目录代次/文件字节保持，正常finish后可获取。外国SQLite事务已持锁时立即拒绝（500ms断言），原持有者正常rollback/close后再取得OFD；本进程SQLite持锁时失败不误释放。inode/mode/父目录/链接/硬链接替换及新逻辑根拒绝，无修复/覆盖。到期后依旧排他但操作失败，drop/finish清理后可获取。真实子进程持OFD、移动目录、确认排他后process::exit(91)，父实际wait91，准确归档inode可正常获取，无清理信号。

fmt/diff-check与all-targets strict Clippy通过。测试TempDir与fd/子进程均结束，unitTMP空，逐字归档后删除重复临时根；无官方服务/运行包/业务请求/全局DB/日常配置/新安装器操作。

## 证据勘误

上一批client-root-index-r1/source-final.json的glob选择了应用测试宿主opencodex_desktop-8c45127ef563f082，却标成执行库测试的binary；库执行日志明确使用opencodex_desktop_lib-114f658f3f2e2b97。原文件与摘要保留，新增previous-source-correction.json记录错误与可继续使用的源码/测试日志/字节投影证据。旧库binary摘要未在重建前独立保存，不用本批新摘要冒旧摘要。本批从实际执行日志解析库binary路径并固定其摘要。

## 下一步

把原fd准备和正常SQLite→OFD交接接入真实NativeLease，保留N/L/K和官方writer持续排他；明确交接失败后的原锁/连接释放顺序与目录重定位检查。再接官方holder重定位/正常释放、不可变movement登记与当前正常Stop权限。原官方2.99、Start30/Status5/Stop10/Restart30预算与未知拒绝保持。其它IMP-34剩余范围不变，本批没有Windows原生影响，不开启W2或发布。
