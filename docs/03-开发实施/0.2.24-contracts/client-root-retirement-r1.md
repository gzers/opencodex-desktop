# 整根可逆归档与缺根只读重开原语

revision: 1；2026-11-13，IMP-34 P24-04，main 947fc8139。承接[根缺席原件](client-root-originals-r1.md)。accepted=false，非冻结候选。此批为存储/原生目录移动原语，**没有生产启停许可构造、根终态独立索引登记或完整缺根Start/Stop接线**。

## 原目录缺席与当前目录对象保全

前置是独立RootBinding已经证明原用户根Absent，schema5 Original保留该来源、它的tree仅描述初始执行目录。计划只允许原件/已停止/已有恢复计划这些明确source阶段，source Head/Original摘要/原RootBinding需相符，实际当前Codex完整tree须按原严格读回匹配初始执行树（只承接原两稳定锁有限规则），Grok也匹配Original。未知用户新内容不能随根移动被隐式收养。保存当前完整executionTree与Grok快照，create-only0400不可变root-retirement计划，不更新Original。

归档槽固定在该原件vault下root-retired-<canonical id>，原用户逻辑路径固定profile/.codex，必须同卷，原生RENAME_EXCL移动整个目录对象而不是unlink内部文件。槽已有对象或目标新用户根存在时拒绝覆盖。原目录本身identity/Protection和全部原文件、两稳定锁都保留在私有槽；没有扩大永久残留例外，也没有用新空根替换原缺席。

Retire前要求完整源与固定plan匹配且归档槽不存在，先持久方向intent，再当前callback权限核对和源代次检查后移动；RestoreExecutionRoot则要求原逻辑路径Absent、准确归档对象与plan匹配，EXCL回移同一目录。已在目标位置也必须有原方向intent，不能把手动移动的目录自动认成完成。四阶段历史或当前恢复观察仍由原件来源决定，此模块不补封历史。

native gap发现实际移动了晚到外部替换时，先create-only冲突元数据记录预期/观察identity，再重核实际对象和空原路径，最多一次EXCL反向归还。若又有变化就保留两份对象及失败，不覆盖目标、不循环猜测。任何权限回调失败/超时、未知内容、归档变化或新根出现都不返回Readback；失败后已发生的移动保留原intent供后继明确处理，不谎称未写入。

## 物理归档锚点与原逻辑作用域

Originals中的retired_root_slot只是内存物理锚点，不是新原件字段。Archived view打开同一个已记录root对象，按原RootBinding的Intent/Created/Published身份和保护复验，原frame仍检查独立摘要/前驱/内容归档/root身份。logical_client始终原profile/.codex，startup-scope原SHA/内容不改；scope的路径验证读取逻辑路径，而真正字节捕获使用固定物理归档根。归档view不允许seal新阶段，避免在根已移出后制造历史。

public open_archived_originals需要ArchiveReferences（应来自后继独立索引的source Head、原root引用、planId/独立SHA），只打开该attempt/固定槽。immutable plan/source Head必须逐字段相同，原frame/scope/root全部匹配；不扫描其它备份、不从现在文件反算并采用新SHA。错误source revision、plan摘要或profile拒绝。References是可序列化事实，绝不因此产生MutationPermit。

独立Readback保存planSHA、方向、原source Head、originalRootAbsent、原logicalClient、固定archive槽、原root identity、当前完整executionTree和Grok摘要、当前root是否Absent。写前后都再核对真实源/归档/原路径：如果私有记录写完后外部新根出现，也必须返回失败。记录存在本身不能清启动限制。存储Readback不等于已登记根终态，也不证明服务/进程/写者已停止。

## 当前权限边界

MutationPermit字段私有，没有public构造或serde；本批没有生产factory。apply接口只能由未来持有当前正常Stop与root写者许可的owner构造，目前无法由生产调用。test专用apply_checked使用显式回调，只证明文件系统/保全/中断语义，不能拿synthetic callback冒OS停止、完整进程家族或官方写者排他。

后继必须在同动作原deadline中处理原claim/SQLite锁、配置写入锁、目录物理锚点变化与当前正常退出事实，并独立索引登记方向/计划/真实RootReadback。现在普通Index及状态读取在根已移出时尚不具备终态路径；测试保留的source index仍Captured3，故它不会宣称Restored或放行Start。生产缺根Start/Stop仍未开放，不能拿这一原语通过代替完整验收。

## 证据和验证

[95份证据与manifest](../../../.adg/evidence/0.2.24/client-root-retirement-r1/)含原假设、测试、原件和SQLite字节投影及最终源码/binary。73去重函数通过：新root retirement8、root-originals9、普通originals23、session26、completion7，fmt/check和all-targets strictClippy通过。较后增加的公开只读归档bootstrap有独立exact test和Clippy，先前日志不冒称执行后来源码。

正例同一目录Retire→RestoreExecutionRoot→Retire，原identity/Protection/frame原字节与逻辑scope保持，当前原根Absent、归档完整。Archived view可读原独立帧却不能写新历史。外部auth/config新内容、归档槽已有对象、原路径新目录、归档后来新增文件、plan摘要/来源revision/profile错误分别拒绝；读回文件写后新根出现也不放行。actual native gap替换对象被完整反向保全并保持失败，不丢弃外部内容。

4个真实child exit91在intent、after_native、before_readback、readback边界，父实际wait91，重开只使用原Source/plan/hash和同一retained root，继续归档后还能回移执行根并再归档；不拍新Original。原源index始终Captured3，不产生RootRestored。日志打印合成文件原字节base64/长度/SHA和原生identity/mode，offline-byte-projections及verified-evidence独立核对5个场景plan/source和SQLite当前Captured3、RootBinding/Original/scope摘要引用、根identity与readback树摘要。离线复制只证明字节一致，不作活体文件身份或进程证书。

测试回调权限丢失后，若目录已经移动就保留归档与intent且无成功Readback，后继明确回调才能继续；没有把清理成功变业务成功。文件系统rename不具通用内核CAS，仍保留前后代次与冲突处理限制，不宣称任意不协作外部进程已被排除。

所有自有child实际wait、TempDir清原现场、unitTMP空，证据归档逐字核对后删除重复临时根。无官方服务/运行包或GUI执行，无真实WebDAV/模型/业务请求、日常HOME或全局协调数据库变化，未构建新.app/DMG或发布。

## 下一步

接真实当前service/launcher正常退出和TCP缺席、官方写者排他/稳定锁释放与root重定位后的权限校验；独立索引注册movement plan和当前方向/完成证明，让根缺席状态下只读验证仍可追溯原HEAD与归档，并接用户明确中断处理与后继新Start。完整缺根原官方2.99 Start30/Status5/Stop10还须新隔离Profile验收。

缺Ready、一般客户端/图/路径/全进程集合及活跃刷新、旧revision4首错、Windows W1/W2产品贡献、NAME/复杂System代理、官方发布验证/README/版本及固定候选仍按IMP-34原范围，目标未完成。
