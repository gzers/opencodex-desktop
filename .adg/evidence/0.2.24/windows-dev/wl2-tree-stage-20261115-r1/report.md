# Windows r4：整树阶段与两锁原生核验增量

2026-11-15。TEAM14 / IMP140 / windows-security-coverage-r4，承接 main2ebdd3ae0 / docs84ec482ee，不改共同声明、官方2.99、日常配置/策略或原预算。上一轮5601df969/7961253ff已交。

本轮代码分批推送：17560e2da（原句柄rename明确NUL存储）、6a24b3685（canonical v2树Intent/Completed）、5d90fb2f5（真实部分current写入poison反例）、7130ab83f（只读原生两锁）。最终7130ab83f98b7c0d9f908e9f3f756dc1c713f04c。

## 实现和准确范围

每次Codex/Grok根MoveToCustody/CreateWorkingCopy/RestoreOriginal都在同一canonical v2阶段链登记Intent，再原生操作，再实际ID/字节/8投影/位置核对后登记Completed；阶段create-only/sync/readback后同一排他current CAS。无第二journal。必需treeRecord拒绝缺字段的前期component-v2；旧/未知不迁移清空，Full-v1/PartialDiagnostic不转换。同phase步骤只能按有限原根状态推进，未完成意图不能跨phase，错根/重复/次序未知拒绝。

未执行move的意图不能发布Completed；单根中断保持seq3；首根已恢复、第二根外改时seq13明确Grok RestoreOriginal Intent，原Grok仍在custody、冲突文件留原处。中断不会自动rollback/catch-up/删除。真实排他current写入一个左括号并sync后注入I/O故障，phase2原件保留、同fileID、poison、再写/open拒绝且无phase3。故障注入仅cfg(test)，不是伪造OS错误。

两处稳定锁仅 .opencodex-native-main.claim.sqlite / .opencodex-native-main.owner.sqlite。最终实现用GENERIC_READ+READ_CONTROL保留文件句柄，对SQLite Windows rollback协议pending/reserved/shared的512字节范围同时LockFileEx(EXCLUSIVE|FAIL_IMMEDIATELY)，无生产SQLite open、无数据write/修ACL。原文件占用rawWin3233原样；持原生范围时真实SQLite竞争被SQLiteBusy拒绝。捕获返回前与restore Completed前核其释放、原始文件ID/8投影、全树内容与未知journal/wal/shm缺席；原缺席锁不创建。只证明当前窗口的rollback锁，不保证未来/mmap/恶意原始写入/任意WAL模式。全量内容和未知拒绝门保持。

初版locks02/library03曾使用SQLite open+BEGIN EXCLUSIVE/ROLLBACK并从VFS取HANDLE，合成对象读回无变化且测试通过；复核发现hot-journal潜在恢复可在查询前写数据库，故在生产使用前弃用。最终library04只读原生方案通过。旧原证据保留，不把初版当最后保证；没有在真实客户端执行初版。

## 结果与失败

| 执行 | 原码/结果 | 范围 |
| --- | --- | --- |
| library01 | 101；9过/1失败 | 第二Grok原根移动边界raw123，seq6意图保留 |
| native01 | 0；5过 | 对齐文件名存储往返、同ID/禁止覆盖/跨卷/ADS/链接/原deadline |
| library02 | 0；10过 | v2树阶段、原根/缺席恢复和混合失败现场 |
| current01 | 0；2过 | 真部分写入poison及正常排他CAS |
| locks01 | 101；0 tests | E0282原生HANDLE输出指针缺显式类型；首编译错保留 |
| locks02 | 0；3过 | 初版SQLite机制，后被只读方案替代 |
| library03 | 0；23过 | 初版捕获前锁检查与原错误类型；不是最终机制 |
| library04 | 0；23过 | 最终8投影/非空组件/树阶段/部分写入/只读两锁受影响组合 |
| product01 | 0；全部bins | 最终clean MSVC debug，179460毫秒，121条warning原样 |

首错library01在第二Grok原根入custody时raw123，旧缓冲区offset20+228字节恰好填满usize分配，没有显式NUL。推断为FileRenameInfo名字存储边界问题；没有把API包装缺失冒直接traced。显式增一个零WCHAR，FileNameLength仍为实际名字字节，不改原访问/不覆盖/同卷/未知拒绝；新对齐名字原生往返与新整树场景过。失败.tmpGvNMQ5先归档核验后保持，未在旧场景重试。首次归档器只匹配行首标记漏panic同一行场景，改成读取确切inline标记后补齐三场景，时间/限制记hypotheses02，不补造先归档事实。

锁首编译E0282修为明确HANDLE，并从静态复核修正原根DELETE句柄自身重开冲突（未声称运行过raw32）。最终原生方案没有此VFS输出指针。所有失败source/stdout/stderr/result保留。

## 未完及门禁

B1/B3为组件事实；B2 label/条件DACL/resource/filter非空证据复用，CAP/trust旧CREATE_NEW1314/5未重试。O整树及两锁有新增机制证明，但正式ClientOriginals/原official Ready/PreStop受控副作用、所有teardown/restore中断、ClosedEligibility和一次性后继仍未实现。H真实GUI与S/D原官方组合未执行，actualGUI=false；不能拿synthetic abort恢复或文本Closed签生产资格。require_durable_capture仍在，Start拒绝、无官方进程，无业务同步/模型请求/W2/双SID/发布。软件未完继续归Windows，不以桌面环境缺口代替其它实现责任。

marenben最后会话事实Disc，用户的交互登录答复仍待；旧empty-host/noJob原证据复用，不重跑旧Full1314/外Job/普通MSI。完整状态表归final-status.json。

## 来源、制品、清理

最终89份执行源逐字匹配工作源、Gitblob行尾等价；library04全部捕获输入与最终product01一致。native01只核对实际受影响的Windows原生backend和native输入，不冒此前整份快照都为最终。final-source.json / executed-final-coverage.json给范围，product01-artifacts.json给实际EXE字节/SHA256；非release预算/冻结候选。旧34条cfgtest警告原样，无strictClippy/安全审计声明。

本轮23处原SID私有归档场景，共147文件414769字节，zip/现场内容逐一复核；zip不克隆NTFS身份/安全，原对象/意图/current/外改留作失败和中断证据。成功fixture由原guard收尾。未再删除这些pending场景，不重试/绕过旧自动审批删除拒绝（含.tmpzyZqe4及历史Profile/Public/schema）；ordinaryMSI与旧清理缺口保持，overallCleanupComplete=false。没有账号/日常Profile/PATH/策略/官方改动。既有untracked windows-schema.json保留。清理细表及每场景原件/私有zip位置见final-status.json，scene-recheck.json给最后逐字复核。

离线verify_evidence.py仅验证原始payload/执行源/状态声明的字节完整性，绝不视为生产验收。
