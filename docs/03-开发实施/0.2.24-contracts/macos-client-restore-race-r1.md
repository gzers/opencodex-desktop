# 0.2.24 Mac恢复发布竞争的内容保全与有条件归还

revision: 1；2026-11-12。归IMP-34 P24-04，承接macos-client-restore-r1及native-client-coordination-r1；代码mainbfb264eb84209d74a81362464661f0b376af2a9b。当前是机械恢复核心的实际竞争修正，生产MutationPermit签发器仍未开放、Stop未接，accepted=false。

## 已复现的具体缺陷

在最后一次state/check之后、真实rename syscall之前，用自有合成夹具明确注入非合作写者：

1. catalog目标被同用户替换为另一个实际inode；RENAME_SWAP把用户新内容移到暂存、把原件放回目标。旧实现虽报错，但没有把用户内容归还原位置。
2. cache从准备删除的已知对象被原位写入用户新内容；RENAME_EXCL移动目标到stage后，旧实现报错却使原目标缺席。
3. archive阶段的stage源在最后检查后被替换；旧实现把陌生源移进归档目录，报错后没有归回stage。

三个首轮反例均真实失败，完整stdout/stderr保留；这说明只有预检查和“失败不记成功”还不足以维持当前文件位置。没有因官方锁已经通过就删除这些非合作写者反例，也未修改官方2.99。

## 修正行为

每次真实发布/归档后，继续核对原plan中预期双方的原生代次/字节/保护。发现未匹配的普通可读对象时，先在原受保护事务目录create-only写入conflict回执：原plan摘要、固定对象索引及实际观察到的target/stage或archive/stage完整原字节/代次/保护。该记录0400、fsync和目录同步，不重建原件或把陌生对象加入恢复许可。

在原生协调/原预算检查仍有效时，最多一次有条件归还：

- 交换冲突：仅target仍是本plan已知发布原件、stage有完整可读陌生对象、其双对象与conflict采集时完全相同时，做一次无损反向交换，随后核对双方是否确实互换。
- 移出目标的冲突：target仍缺席、stage是同一已观察对象时，用RENAME_EXCL把它归回目标。若新写者又建立目标，则不覆盖。
- 归档源冲突：stage仍缺席且archive仍为刚观察的同一对象，用RENAME_EXCL归回stage；目标再次变化不覆盖。

归还成功也始终返回失败，不产生restore outcome；当前计划仍包含未解释的外部变化，不能自动继续成功。任一步二次变动、元数据/权限不可读、未知文件种类或归还条件不满足，保留对象及已有conflict回执，不开启循环重试、不猜路径删除、不用新现场重建基线。不允许一次冲突的成功归还抹掉另一个对象的外部修改。

## 不作虚假保证

这不是预期inode条件的内核CAS，也不是对任何同SID恶意/持续写者的全面隔离。实际syscall可能在原件发布后才检测到竞争，短暂位置变化不能宣称不存在。反向交换自身也不是不受竞争的魔法操作；前/后读回变化时保持失败与内容保全，不能承诺所有对象无条件回到原路径。当前验证能证明确定注入边界的陌生内容未删除、能归还的原位置归还、二次写者不被覆盖，以及无成功终态；不能外推全时序线性化或任意持续写者安全。

同进程NativeLease已经防止正常回调执行中外部helper租约提前释放，但仍不能替代服务正常退出/真实代次/原件所有权许可。MutationPermit无构造器/签发器的产品边界保持；本轮只调用私有合成测试seam，不把三个真实客户端文件尚未恢复的原失败改为通过。config/Grok/Agent和全树、活跃官方刷新、missing CODEX_HOME、跨卷归档及原flags适用范围仍未关闭。

## 验证

本轮最终生产算法下19项原生恢复测试通过，覆盖原13项原件/ACL/xattrs/中断/继续回退/冲突反例及新late-gap反例、二次写者、实际外部OS进程替换：

| 场景 | 证据结论 |
| --- | --- |
| native gap迟到catalog/cache | 用户新内容归还对应原目标；操作失败、不写outcome。 |
| archive gap迟到源 | 陌生源归还stage，不被当作原件提交。 |
| compensation gap再来第二个cache写者 | 第二份保持target，第一份保持stage，双方原字节均在；不覆盖新目标、不写outcome。 |
| 实际另一OS进程在最后检查后替换catalog | 真实新inode/内容返回目标；conflict回执0400，原plan摘要与实际陌生stage原字节核对；后继同plan仍拒绝未知状态。 |
| stage被外部替换 | 不把陌生已发布对象当本plan原件反向覆盖，不写outcome；双方内容保持在实际目标/stage。 |

后续只加一项conflict回执写完、归还前实际子进程exit91的定向测试，并给已过外部写者测试增加合成原始conflict JSON输出；生产算法字节未变。新增定向检查通过：父进程以独立原/phase/plan摘要重开，陌生原字节在stage、冲突回执保留，Complete/Rollback均拒绝，不生成成功outcome。连同原先11个实际发布/归档突退边界，本组仍保留12个真实退出边界的不同范围，不将新冲突退出当正常继续通过。

去重后20项相关测试通过，原19项完整组+新增退出项，外部写者仅为原始证据输出定向补验；最终编译/格式/全目标严格Clippy通过。各次native源/binary SHA独立保留：source.json对应19项算法；source-extra.json对应后继测试记录增强，算法相同。先前3项真实失败/原件均保存，没有删失败目录或弱化断言。

实际原始conflict回执以明确合成输入输出到stdout，留存conflict-raw-receipt.json及conflict-crash-receipt.json，含实际捕获的两代对象身份/字节/保护，不能当日常文件或当前服务身份凭据。来源、准确错误、边界假设、原生结果与manifest位于.adg/evidence/0.2.24/macos-client-restore-race-r1/。OS sandbox拒绝日常HOME读、网络和自有根外写入；所有测试/外部写者进程、复制binary、合成原件/stage/archive/conflict目录及逐字节核对后的重复回执根已清理。未动官方运行包/日常配置/正式安装，无真实服务、业务同步或模型请求，未构建新.app/DMG。

下一步是服务正常退出的真实代次/原件关联证明与完整恢复编排，再在原30/5/10预算下跑一次受影响实际恢复链。当前修正不构成全部发布竞争已线性化或生产恢复许可已可签发；产品入口仍须拒绝无法证明的工作。WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新官方交付发布验证及固定候选按IMP-34继续，不发布。
