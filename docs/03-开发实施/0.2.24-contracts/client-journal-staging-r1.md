# 恢复计划登记前的事务目录暂存

revision: 1；2026-11-12。归IMP-34 P24-04，代码main55bc44300。承接[Stopped缺失计划](client-stopped-missing-plan-r1.md)、[回退与重准备](client-recovery-rollback-r1.md)。accepted=false，非冻结候选。本批修复准备中断会在客户端留下未登记暂存而不能重试的问题；没有扩大恢复来源、旧版本迁移或自动覆盖授权。

## 改动与不变量

生产Pending.prepare原本在Codex客户端根创建.ocxd-restore-<id>-<i>，完整plan落盘且索引登记前可能中断，之后preplan全树检查会拒绝这个未登记stage。新生产prepare只在已独立登记Original的0700事务目录创建原件暂存、准备记录及固定plan。计划登记前，客户端目录不写暂存文件，不交换/移动/删除目标；哪怕在写入部分字节、应用权限、fsync或plan发布后进程退出，也仅留私有未登记事务文件。

再次明确准备仍按原四阶段独立摘要、当前精确Stopped后像/内容补偿分类、完整原件余项及当前官方OS停止/原service与launcher/port缺席、同进程claim/N/L/K写者排他和完整镜像准入核对。生成新随机plan id和新stage，EXCL写入，不扫描/采用/删除旧stage或未登记plan，不将旧文件的新摘要登记为来源。旧事务残留保留作证；若客户端本身有任何未知stage/其它内容变化，仍拒绝，不以此修复为理由豁免。

初始新plan schema3显式journalStages=true，原plan独立hash同时绑定布局及每个stage原生identity；Pending重开必须布局与缓存的plan-bound根相符。schema1仅用于既有本版机械事务夹具布局，未新增旧软件路径兼容/迁移或缺省收养。schema2 rearmed逻辑计划沿其原parent继承真正的stage位置，不复制/移动对象或覆盖旧决定。所有实际stage路径固定为原件事务目录或既有固定客户端根，不接用户任意路径。

登记后明确Complete/Rollback才可从plan绑定的stage根执行原生RENAME_SWAP/EXCL；目标仍是固定客户端三文件。事务内所有pair/state/归档/冲突返回路径均沿同一绑定stage根，原字节/ACL/xattr/flags、dev-inode-birth、全批先查、每步原件/权限/当前许可检查不减少。原后像仍保留归档，不unlink猜测对象；跨卷在准备前拒绝，保留原有同卷支持边界，不复制删除回退。新stage位于journal时，客户端完整树不再忽略任何plan stage名，所有客户端未知stage一律拒绝，最终两稳定锁例外保持。

另补强stage创建读回：O_EXCL创建并写入的fd始终保留，校验nlink=1和最终路径原生identity必须属于该fd，末次fd复核仍同对象；不能在创建读回前接受同字节/同权限替换的新inode。发现替换保留现场，未绑定stage不获写许可。原有源内容/保护、父目录pin及fsync保持。

普通Stop、显式PreparePlan/Continue/Rollback仍使用同原deadline/权限和独立索引CAS；新布局没有自动恢复、自动重试或从文件成功布尔签权。prepare中断仍返回失败/Stopped，不把未登记plan当成功；新明确动作可重试，成功只RestorePrepared，最后须另行Continue。UI/IPC和唯一原型流程不变，本批未重复无影响页面测试或新建原型。

## 实际检查与旧证据边界

90相关去重后端用例：restore30（初组29再加新stage-fd反例）、session26、index27、completion7。新journal准备真实子进程exit91覆盖7个边界：preparing记录、stage_created、stage_bytes、stage_protected、stage_synced、staged和plan_published。每次重开仅使用独立原件refs；当前客户端仍精确Stopped且无stage。新plan准备/恢复前后全部旧未登记文件原字节/身份/保护不变，未被选为新stage。精确已登记stage/目标同字节替换和native_gap晚到外部对象保全拒绝通过；新增fd到路径替换即使字节/权限相同也拒绝。

测试审计发现既有publication-boundary函数虽父进程设置OCXD024_RESTORE_REARM，child未消费，因此以前两次布局相同的输出不能证明rearmed中断路径。原证据保留，本批child显式Complete中断→Rollback→新逻辑rearm，再到指定退出点；分别普通/真实rearmed各11边界。另wrapper用journal布局运行同矩阵，总计44个真实登记后exit91场景（2布局×2方向计划×11边界）。新7个准备中断另计，不用函数计数冒场景数或把过去两次普通流程记为rearmed。具体边界/原局限记录rearm-coverage-audit.json。

新布局真实ACL/不透明xattr还原单测及完整Rollback readback/index7→rearm/index8→Complete/index9闭环定向通过；旧回退回执原文保持，错decision不能闭合另一终态。最终格式、编译、全目标strictClippy通过。原组其余已过用例依据未变行为复用，未为末次fd guard重跑整个服务或所有已过组；7准备中断按受影响guard再次定向通过。

## 真实隔离的准备中断、重试与恢复

一个新隔离Profile使用未改官方2.99，在普通Stop准备计划的stage_bytes阶段注入错误：只在私有原件目录写了尚未应用原保护/fsync的暂存，尚无完整plan或索引引用。Start/Status/Stop确切耗时见native/native/summary.json；原service与launcher正常0退出、4捕获TCP缺席，原测试exit101/failed、index6 Stopped/restorePlannull保留，客户端catalog/cache/audit仍未补偿。客户端根没有stage，保存原frame/引用和两个未登记文件（preparing+不完整stage），没有把它们作为完成证据。

同保留现场新进程先验证旧revision/错摘要和外部原config改动拒绝（保留索引/外部内容，测试自行恢复合成fixture取新确认），随后新明确PreparePlan4765ms，index6→7 RestorePrepared，当前四项目标字节/缺席保持，客户端根无stage，生产新plan schema3和journalStages已核对。原两份未登记文件的SHA/字节长度/inode/mode与准备前一致，新plan id不同，没有收养旧文件。另一次明确Continue6550ms，原native许可/全树/权限/两锁/回执/index全部闭合到Restored revision8。每次各原10秒，不拿两动作时间拼成原Stop成功、不重新启动服务或请求模型。

独立只读native完整Codex/Grok树SHA匹配新completion；四frame/scope/live/plan/receipt原摘要与index8/8events一致；15原内容blob按摘要核对，完整journal包含孤儿/新stage归档/计划/原件分别保存。7客户端路径原字节/缺席全匹配，真实新状态NotRequired、无PID/port、仅canStart。未登记stage和准备记录在完整恢复后仍身份/字节/权限不变，原中断index6及失败结果不改写。

来源界限：初中断使用build-native/source，后加stage-fd绑定检查和反例后形成准确新构建，真实重试+Continue使用retry-and-continue/source.json与对应binary，包含最终产品guard；最终产品代码无后加改动。UI/命令未改，不冒最终签名候选GUI→IPC整链。本批未构建.app/DMG或发布。

证据[本批目录](../../../.adg/evidence/0.2.24/client-journal-staging-r1/)及manifest保留原假设、7/44退出场景输出/旧覆盖局限、所有来源边界、失败index6/成功index8/原件与残留/完整树/真实状态。OS sandbox拒绝日常HOME/外网，仅loopback，无真实业务或模型请求。自有服务/launcher/test进程与原主端口缺席，3精确新协调DB实际EXCLUSIVE重获释放后和程序/Profile/所有夹具清理；globalL inode/字节/权限/owner保持、排他重获释放且无sidecar。unit TMPDIR空，归档逐字核对后删除重复临时根，正常target保持。

后继仍有缺少已独立登记Ready/PreStop/Stopped后像且客户端已变化的早期恢复、首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错、Windows贡献和集成、NAME、复杂System代理、官方交付验证/README/固定候选及W2。所有本版必须项保持，目标active；Windows远端仍b2e89665d/0eef59a01无新回交。
