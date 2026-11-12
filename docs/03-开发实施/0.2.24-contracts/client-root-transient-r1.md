# 活体写锁约束下的整根移动与完整读回

revision: 1；2026-11-13；IMP-34 P24-04；main c576a3b75。accepted=false，非冻结候选。承接[写锁连续性](client-writer-relocation-r1.md)。本批完成真实Guard/NativeLease与根存储事务组合；生产正常Stop许可构造、缺根Start/Stop及下一次Start仍未接。

## 临时写锁与用户原件分开

prepare_guarded必须有同一当前NativeLease、同一原逻辑root和活体Guard。先取准确原Image，再正常claim SQLite→OFD交接，N/L/K和config Guard保持，复核同一Image后才记录计划。SQLite自己的journal正常消失后再拍树，不忽略journal、不把它当永久残留例外。

guarded计划schema2绑定完整executionTree和准确writeGuard Image，原独立Original不改。Original必须不含该固定路径；投影只可去除config.toml.ocx-write.lock，且其inode/birth、完整Protection、大小、SHA与已登记Image逐项相符。没有通配前缀、文件名扩散或其它sidecar过滤；临时记录是事实，不能从JSON构造Guard。非guarded当前内部计划继续由现有入口产生，不做旧版本用户数据兼容/迁移。

准备、重新打开live计划及归档view都再次以独立Original核对投影后的完整树与Grok原件。即使有人改写私有计划、重新给出摘要并把当前新增auth文件拍进executionTree，登记与移动前仍拒绝，不能用新的私有baseline收养用户文件。换inode负例将旧marker移到client树外，单独验证相同字节/新inode不能通过，避免额外文件混杂。

## 真实持锁连续性到终态

apply_guarded仍必须持有字段私有的MutationPermit，且当前Guard Image与计划相同、NativeLease已经处于OFD。只允许Retire，不复活旧holder。它先向当前helper登记唯一目的地，在原deadline中移动同一目录；移动后先核实际对象身份，再明确重锚NativeLease和Guard。正常Guard.finish必须获得官方release/锁Absent/实际helper退出0和当前lease核验。随后才完整捕获released tree，必须精确等于计划去除本次临时锁后的树，写并复验Retire readback，再返回不能反序列化的AuthorizedReadback供独立Index关闭。

普通apply/旧存储入口拒绝guarded计划，不能以磁盘Image或回执冒充活体释放。根已经归档时，只读观察允许准确held tree或准确released tree两个有界事实：原helper可能在崩溃后经EOF释放。但独立阶段仍Prepared，磁盘已有readback也不自动结案；终态核验严格要求released tree。新增marker、相同字节新inode、其它文件或新逻辑根一概拒绝且不覆盖。

成功重锚前先比较实际移动对象并create-only记录冲突；之前after_native回调先拒绝会漏冲突记录的路径已调整。无覆盖补偿仍须当前权限确认，权限不明则保全归档中的外来目录与被移走的原目录，维持失败，不绕过权限强行归还。before_rename测试边界位于最后检查后，生产传空checkpoint；没有用测试回调提供生产许可。

## 证据与验证范围

[374文件manifest及可重放核验](../../../.adg/evidence/0.2.24/client-root-transient-r1/)固定最终7源文件和实际库binary。22个最终相关函数通过：原retirement9、independent index6、新transient7。fmt/diff-check/all-targets strictClippy通过。真实exit91四边界为计划登记、原生移动后、临时锁释放后、readback写后，父实际wait，独立head都保持Prepared4/null completion；正例独立关闭为Absent5。原JSON/SQLite逐字投影核验Original/scope/plan摘要、原根缺席来源、临时Image身份、释放树SHA和事件链，副本不冒活体权限。

首真实组合轮准备失败exit101保留；定向诊断轮原清单明确多出512字节claim.sqlite-journal，原拒绝正确。修复是把正常OFD交接前移，而非改变Original比较或增残留例外。两失败原目录、源码摘要、具体差异和原始输出保持。诊断轮曾保留同一不可变运行包输入供后续新Profile复验，不是活进程等待；原cleanup JSON的预期移除字段有专门cleanup-correction说明，最终实际清理另有确认，不能拿旧字段当已清理证据。

最终官方组合正例3760ms：实际未修改官方2.99准入、NativeLease/OFD、真实config Guard、完整移动/官方正常释放/读回/独立Absent5；晚到替换2201ms：实际Guard＋NativeLease在最后检查后遭替换，冲突先登记，两对象保全，head保持Prepared4、无终态回执，当前观察未知。两项均在原10000ms内，整轮供应/准备耗时不冒动作预算。这里的Original/scope与生命周期MutationPermit是显式合成夹具，没有运行服务/正常Stop或OS就绪证明，不能作为完整Start30/Status5/Stop10通过。

最终源码/binary与两最终原包轮完全对应，binary 7fc1f2f381550739c73ed0c000e08c72711ea23acd409afb1de3eb4e76284acb；前期20/21项及原包中间轮只作各自阶段证据。只读子审查发现并复核上述冲突/用例问题，不改文件、不代签测试；较后live Original核验有自己的最终回归/原包证据。

四份原生原字节payload无损gzip，原长度/SHA与压缩摘要、roundtrip均记录，原失败不覆盖。删除的是可重建的Bun-cache派生副本；原payload全字节保留。可选诊断后快照仅保留相关配置/索引原字节，额外Bun编译缓存留哈希而不复制，与完整.codex/Grok读回范围不同。verify_all在临时目录重放并自动清理，不修改归档。

六轮native实际句柄均终止，最终两个holder的内核ESRCH独立核对；每轮精确新N/K经exclusive重获后移除，预存global L身份/权限/字节保持并重获释放。所有临时Profile、复用运行包/资源、失败夹具和重复证据根在证据逐字归档后清理，unitTMP空；工程target保留。无日常配置、真实WebDAV/模型请求、服务、新.app/DMG/Windows候选或发布。

## 下一步

MutationPermit仍无生产factory。接当前正常Stop/launcher/TCP和原件/声明来源，在原预算内构造不可序列化、可跨本次物理重锚核验的许可；再开放生产缺根Start/Stop、下一次Start及明确中断恢复。不能去掉Ready/capture guard、从磁盘记录制造正常退出历史，或以本批组合测试缩减原IMP的其它客户端/路径/进程/刷新范围。Windows W1独占贡献继续，W2待固定集成候选；NAME/System代理/官方发布验证/README等完整剩余不变。
