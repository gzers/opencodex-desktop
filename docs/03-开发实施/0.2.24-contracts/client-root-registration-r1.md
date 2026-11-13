# 首次目录归属的同锁索引登记

revision: 1；2026-11-12，IMP-34 P24-04，main 1c967e1bf。承接[目录创建归属](client-root-custody-r1.md)。accepted=false，非冻结候选。本批完成创建凭据转交与独立登记；生产缺根捕获/启停/原根状态恢复尚未接线。

## 由进程内创建凭据登记，避免空档

Index::register_first_root只收PublishedRoot，不接受任意序列化References。转交前复验原data/profile/实际Codex根与created fd、0400intent/created/published和原action身份，context不符或已有index/原件拒绝。真正移动原flock File和created fd给Index，整个过程不unlock或重开另一把锁。根证明原deadline继续覆盖SQLite注册；提交后超时只保留Preparing，不伪报未写入或完成。

SQLite仍是现有binding/current_session/events三表及user_version3。新根binding使用RootBinding(schemaVersion1, scope原字节, originalRoot独立引用)，普通已存在根保持原scope字节，不能根据新路径扫描自动改写旧binding。两者代表同版本不同初始事实，没有旧版配置迁移。原根Absent引用绑定创建nonce、intent/created/published SHA，原uid/data/profile路径与native identity/profile保护、实际新root身份和保护必须匹配。

新根初始化同一BEGIN IMMEDIATE/FULL synchronous事务内写binding、Preparing revision1和events1，首ticket必须等于创建nonce。一个COMMIT后才返回，消除根归属已登记却没有待处理会话的空档。创建事实永久留在独立binding；首事件也复核，不能删掉origin或换ticket冒成普通原本存在目录。created fd在初次Index内仍持有，后继重开只承认已登记的事实，不重建这个句柄或从磁盘给执行许可。

## 拒绝未知初始化与旧恢复捷径

未登记root sidecar仍不能初始化普通index。已有空index、大小不够或rollback/WAL/SHM侧文件时，在普通SQLite恢复/初始化前拒绝，原失败字节保留；只读观察亦不修复。root marker加普通旧scope binding同样拒绝，不允许降级。只有完整独立RootBinding和Preparing首事件都核对后，后继open_existing/inspect才可读到已登记的待处理会话。

活跃Index检查root binding仍为原字节、首Preparing仍绑定同创建nonce，重新复验所有独立来源与当前目录。Created字节/引用/root代次、原profile权限或context改变均拒绝，不修权限、不拍新基线。预览不得给旧“取消准备”捷径：那个流程仅证明客户端文件不变，不能恢复原先整个目录缺席。

本批故意保持安全边界：reserve_capture和普通Captured转移拒绝root registration；旧取消准备、完整readback收尾也不能关闭原根Absent。Active.capture仍未调用新创建/注册路径。当前用户设备若没有.codex，仍不能用这个中间能力直接完成启动；本批不将功能范围缩成“有目录就启动”或拿旧Original目录帧掩盖原缺席。

## 验证与证据

[32份证据与manifest](../../../.adg/evidence/0.2.24/client-root-registration-r1/)包含假设、源码/测试binary摘要、实际子进程和数据库记录、离线字节投影及清理。42个相关函数过：新增登记6、创建8、index27（2个显式manual ignored）和旧Preparing取消1，fmt/check及all-targets strictClippy通过。首遗漏Instant import导致E0433，补齐后通过；该观察在hypothesis与chat原输出保留，不声称另有原始日志。

真正的第二进程在root_lock_transferred、before_root_index_commit、root_index_committed三个回调点探同action flock，全被拒绝，实际计数3。正例证明只有一个Preparing首事件、binding原缺席引用正确，原created fd仍在，drop后只读查询与open_existing可读Preparing，但不能普通捕获/重开新基线；预览不提供普通取消/恢复按钮。

3个真实child exit91：转交后但index前、SQL提交前、COMMIT后。前两者readonly和普通open均拒绝且保留原文件字节，后者只能观察Preparing1、无Original，仍不能普通捕获。原日志保存所有合成来源文件和SQLite原字节的base64/长度/SHA；verified-crash-evidence逐一核对JSON引用链。提交后数据库在离线复制的immutable只读连接下独立验证user_version3、RootBinding scope、同原nonce/独立引用的Preparing1和唯一events1。离线副本只是字节事实，不冒原目录句柄/native权限，也未用副本代替真实锁探测。

context不符/created目录被换、binding删origin降级、created.json内容被改后恢复0400、first event及current ticket同时被改都拒绝。旧无root marker的普通索引、早期捕获/取消回归保持。不运行官方服务或真实业务，本批不能证明缺根原包启动/Stop预算或GUI→IPC整链。

所有测试子进程actual wait到成功或91，TempDir清原自有目录，unitTMP空；原证据核验后删除重复临时根。未修改日常HOME/官方包/系统设置，未运行真实WebDAV/模型请求，无全局协调数据库变化或新.app/DMG/发布。

## 后继不可省略的工作

- RootBinding有原根Absent事实；现有原件Frame中的目录树只是新执行根的捕获。必须明确这一差别并保持独立引用，再接root-aware capture，不能直接取消当前拒绝开关。
- 接正常Stop后的原根状态恢复，原件和两稳定锁在安全证明下保全，根移除/变化后独立索引和只读状态仍可核对；再接后继新Start。未知目录不得删除、移动或收养。
- 初始化未登记、已登记未捕获及捕获后中断应有明确可审查恢复行为，不能自动resume或用旧不变客户端取消清除原根缺席要求。
- 完成上述生产接线后，用新隔离Profile跑原官方2.99缺根Start30/Status5/Stop10和完整原状态/状态重开验证；本批文件系统/SQLite通过不替代它。

其余IMP-34范围继续，目标未完成。
