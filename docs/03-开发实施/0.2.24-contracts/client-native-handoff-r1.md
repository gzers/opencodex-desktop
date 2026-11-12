# NativeLease 的 claim 交接与移动后核验

revision: 1；2026-11-13；IMP-34 P24-04；main0b8b1e74e。accepted=false，非冻结候选。承接[OFD原语](client-claim-ofd-r1.md)，完成协调锁层接线；未接服务Stop、官方配置writer holder重定位或生产root MutationPermit，完整缺根Start/Stop与next Start仍未完成。

## 显式协调入口

普通with_native_coordination仍用四个真实SQLite事务；claim连接单独保存，N/L/K三连接保持原顺序/强度和原VFS检查。新增with_native_relocation_coordination显式路径，共用未修改官方2.99准入、程序来源前后核验和原deadline。它仅在所有SQLite连接之前prepare原claim可写OFD pin，核对其device/inode与原准入pin一致；普通流程不额外启用此能力。

begin_claim_handoff先核对完整lease，进入HandoffPending，再正常rollback/close仅claim SQLite；N/L/K继续持有。随后从预存fd非阻塞获取OFD，不打开/关闭同inode额外pin。空档没有操作权限；失败留Pending，后继check及再次handoff拒绝，清理返回失败并释放剩余锁，不把外来claim锁误认成自己。它不证明该空档中其它进程未接触业务文件，后继完整movement授权仍须内容/代次复核与原正常Stop事实。

Ofd模式只有claim不再调用已关闭连接的VFS；其准确inode/Protection由ClaimGate持续核对。仍存活N/L/K连接全部保留原autocommit/VFS检查及pin。ownership receipt、profile、HOME、运行包来源、期限均照旧；不以claim迁移绕开其它来源条件。

reanchor_claim只接受仍是原directory identity/Protection及准确claim inode的物理路径，逻辑client始终原路径；原逻辑根重新出现对象即拒绝。移动后旧anchor拒绝，明确reanchor后继续核对原物理对象。此API仅是协调能力，调用者必须另据不可变movement plan授予移动权限，不能从目录相同就生成Stop、writer或文件恢复许可。普通lease不能handoff，未知anchor不被收养。

释放前完成核验；Ofd先显式解锁claim，N/L/K保持至各自正常rollback/close，所有SQLite连接关闭后才析构原pins。到期不会自动释放活跃锁，操作与过期finish仍失败，栈展开负责清理；Pending也不会回报成功。原子文件操作/未知拒绝与完整恢复合同不变。

## 合成与原官方验证

[证据及manifest](../../../.adg/evidence/0.2.24/client-native-handoff-r1/)记录最终19个常规函数通过（含2个子进程探针入口）；1个官方原包检查按显式ignored入口另行实际运行。常规回归包含四锁原顺序/竞争/期限/上下文/owner/VFS、部分获取失败清理，以及新7个交接场景。实际子进程在claim交接空档验证N/L/K仍忙、lease拒绝，而claim可被独立获取；抢占者确实进入空档持claim后，本次OFD获取失败并保持Pending。正常交接/整根移动/额外claim fd关闭后四锁仍排他，原claim字节不变；新用户根、其它anchor、owner或N的VFS替换、期限变化不放行。

新隔离Profile使用固定未修改官方2.99资源，原bundle descriptor与official tgz摘要复核，沙箱拒绝所有外网与日常HOME读取。官方构造器建立本轮N/claim，再由真实with_native_relocation_coordination取得准入和四锁；实际K/L写入入口在移动前后都报告busy且callback不执行，旧路径不会被重建，正常释放并回移后官方写入正控完成且过期permit拒绝。真实协调动作2054ms，小于该项10000ms，包含来源准入/交接/移动/末尾核验；整个供应/测试runner69.944s不冒动作预算。此处根移动是test-only合成动作，未执行服务，不能代替Start30/Status5/Stop10验收。

fmt/diff-check/all-targets strictClippy通过。最终源码逐文件SHA与官方轮source完全相同，实际执行库binary摘要9915ef0f00f9a138021bcbed3d1bc65a9406ef084d74c9ddf79fd16f8fd01d04，由执行日志定位；初期普通回归日志保留为阶段性证据。verify_evidence.py核对source/binary对应、最终测试、真实official K/L拒绝/正控、原global L前后事实及清理，不把记录摘要当服务权限。

官方runner全部实际wait结束，无服务或业务请求；精确本轮新N/K文件各经exclusive重获后删除，自有Profile/program/资源/执行binary临时根删除。预存全局client-desktop-lifecycle.sqlite的identity/mode/uid/gid/字节SHA完全相同，exclusive重获并释放，未删除预存文件；无其它全局DB或日常配置修改。unitTMP空，证据逐字归档后删除重复临时根，无新.app/DMG/Windows候选或发布。

## 后继

官方config writer Guard仍绑定原逻辑config路径；移动前后它的持有/释放事实尚待专门协议与原包验证。下一步接该holder物理anchor/原token/正常release，再由当前Stop/launcher/TCP和完整原件读回构造不可序列化movement permit，按原不可变plan移动与独立关闭，并处理next Start及明确中断。不能把本批协调层通过当上述产品范围完成。Windows W1/W2、NAME/复杂代理/其它IMP范围保持。
