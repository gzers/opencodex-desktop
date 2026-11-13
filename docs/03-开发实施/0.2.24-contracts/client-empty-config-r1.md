# 首次空Codex配置的可逆恢复与官方写入排他

revision: 1；2026-11-12。IMP-34 P24-04，main b60bd1079。承接[首次客户端诊断](client-first-use-r1.md)与[当前恢复观察](client-ready-current-recovery-r1.md)。accepted=false，非冻结候选；本批不改变官方2.99或日常配置，不关闭本版其余范围。

## 新的明确恢复对象

原先空.codex目录正常Start后由官方写入config.toml；正常Stop清理路由后留下单换行，原完整tree读回因此正确拒绝。不能以“空文件无害”忽略，也不能直接unlink。现有私有恢复计划新增可选emptyConfig，记录本次停止后或明确当前观察的真实Image，schema5强制journalStages。普通前三件kinds/staged保持，第四target固定config.toml、原欲恢复值严格Absent，操作是Remove，不能从序列化输入换成其它路径。

准入从同一次已核验独立frame前缀提取config.toml的RecordedFile（identity、protection、长度和内容摘要）：Original必须明确不存在；Ready必须存在普通file；实际PreStop若有，必须与Ready的该记录一致；Stopped/current必须存在、保护和Ready相同，且当前准确Image与该独立tree记录完全匹配。当前字节不超过4096，只接受ASCII空格/Tab/CR/LF。缺Ready、新内容/新权限、同字节换inode、记录缺席后新出现文件均拒绝保留；已有原文件不进入第四对象路径，继续原全树严格核对。

明确恢复缺历史阶段时，仅用实际Original/Ready/实际可有PreStop和独立当前observation；空闲内部机械槽不作为历史配置证据，原refs/phases不变。计划load每次重新提取原tree witness并核对emptyConfig，删掉该字段伪降为旧三项计划也会被拒绝。确认摘要包含第四对象，晚到另一份合法空文件也须新的明确确认；不重新拍Original，不认领未登记文件。

## 复用同一原生可逆引擎

Pending的目标状态、暂存和归档识别精确文件identity/bytes/Protection。第四项与前三项共享每步权限核对、计划load、原生RENAME_EXCL、native_gap后检查、冲突create-only记录/一次可证明的反向移回，以及旧对象归档，不猜路径删除、不跨卷copy补成功。Complete将已记录空配置原对象移到私有stage再归档old-3，当前config必须Absent；Rollback把同一个停止后对象移回，不能覆盖外来新目标。rearm仍同object_id/旧原生对象、逻辑新plan，原Rollback历史不改。

全组预检再首次修改，未知任对象拒绝。review指纹含第四target/stage/archive，计划终态与现有对象一致才出确认。完整读回的remainder只在已load的第四对象明确覆盖范围排除它；它本身由同一精确对象引擎复核，不是通用忽略。最终Complete整tree仍直接比较Original，无第四对象例外；Rollback同原计划停止后对象和原余项，且不能清启动限制。ReadbackProof和独立索引终态仍绑定原refs、plan、安全属性、两稳定锁释放及原deadline。

## 官方config写入锁

复核未改官方源码确认config.toml还有独立的config.toml.ocx-write.lock。claim/N/L/K不能排除这条官方配置写入通路。因此内容首轮pass仅证明恢复内容，明确不作为最终写者排他证据。

Desktop自有client_config_guard299.mjs只import核验原官方2.99的withConfigWriteLock及prompt-lock，没有复制或修改其实现。专用子进程在官方callback中阻塞读父进程pipe，持锁直到收到准确release操作token；没有2s定时自动释放。Rust仍持原同进程NativeLease，每个修改check同时核对native许可、专用child仍在、锁完整Image/安全/PID/token仍是原来那份。仅持PID不是许可；输入HOME/Profile/官方package、canonical路径/完整版本选择先核对。锁忙/不可读/非普通/不安全或替换拒绝，不修现有权限、不接管旧锁、不等待绕过竞争。

callback结束后release消息、官方finally释放、独立released回执、child exit0与锁缺席都必须在原NativeLease/action deadline内，才成功返回。失败也请求正常释放，但不把清理成功改判恢复通过。父进程崩溃由OS pipe EOF触发官方finally并退出；清理兜底只可终止自己的专用helper并保留失败，不能作为正常释放证据，不能信定时器或子活性代替锁。现阶段在native锁后获取配置锁为同步fail-fast，若另一个官方writer先持该锁则拒绝，不等待造成锁序循环。

这把官方advisory锁不覆盖Codex本体或任意不协作进程；原native目标代次检测、冲突保全与未知拒绝仍必须保持。也不代表全进程家族/任意外部writer已经证明静默。Grok需要自己的config路径锁，此批未拿Codex锁冒它的排他。

## 验证与适用边界

[148份证据与manifest](../../../.adg/evidence/0.2.24/client-empty-config-r1/)分开保留阶段来源。55相关后端函数：新6、终态1（现循环已有原配置/原缺席×refs3/2/1六情形）、旧restore32/tree9/completion7最终通过，格式/编译/strictClippy通过。新增真实child exit91覆盖第四对象原生交换后、归档前、第四对象实际归档后、终态后；再次进程只用原独立refs/plan重开继续，不拍新baseline。外部替换在gap被完整反向保全且原操作失败；既有原件、不明Ready、新保护、缺记录和plan降级均拒绝。

初测试误用旧client-directory staging，1过3失败，产品按要求拒绝；改夹具用真实journal staging后过，断言不降级。首E0277推导切片与guard私有字段访问编译错误分别修显式Vec和精确只读config_write_lock_image边界；这些首错结构化记录和chat原输出说明保留，不称所有首错都有独立原始日志文件。

原生内容第一轮Start8477/Status255/Stop5920ms通过，但先于官方配置锁接线，first-native-scope-review明确不能作为最终排他证明。官方锁定向真实另一进程尝试同官方writer得到locked，第二holder亦拒绝；2100ms后仍锁住，正常释放后writer通过。原生父进程在持锁内直接exit91，没有Rust析构，子通过EOF官方finally释放并消失（ESRCH），锁Absent、未创建config，未用信号或强停证明。

最终准确带锁binary用另一个新Profile、复用完整核验的原样官方2.99镜像，真实Start6314/Status245/Stop6654ms、原30/5/10秒未改。driver exit0，Original config缺席、Ready/PreStop已独立同一记录、Stopped单换行的准确identity/保护/摘要绑定schema5，归档old-3保留该对象字节，最终config与Grok均Absent、index8 Restored。独立完整当前Codex/Grok原生树SHA匹配completion、Original/Scope/Live/phase/plan摘要全部核对；正确实际Program上下文状态Stopped/NotRequired/无PID或port/仅Start，锁holder正常释放是生产成功必要条件。

未改页面/原型/命令语义，沿已有用户明确恢复/撤销/重准备流程；没有重复无关浏览器测试，也没有构建新.app/DMG/固定候选GUI→IPC整链。Grok首次backup、整个.codex根不存在、缺Ready、一般客户端/图/路径/大树/链接、全进程集合/活跃刷新、旧revision4首错、Windows W1/W2、NAME、复杂System代理、官方发布验证/README等仍在IMP-34。

归档先逐字核对，再确认2个隔离Profile/程序/全部夹具和重复临时根清理；6精确新协调DB实际EXCLUSIVE重获释放后删除，自有进程和主端口缺席、config锁与claims缺席。globalL inode/字节/权限/owner保持且排他重获释放/无sidecar，unitTMP空。OS sandbox实际拒绝外网及日常HOME，只loopback，无真实业务/模型请求；官方包、日常配置与系统设置未变，没有发布。
