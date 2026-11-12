# 独立Stopped后像下缺失恢复计划的显式准备

revision: 1；2026-11-12。归IMP-34 P24-04，承接[显式继续](client-recovery-continue-r1.md)和[捕获预留](client-capture-reservation-r1.md)。后端mainfe5be4b97，界面main5870c915a。accepted=false，非冻结候选；仅处理已有全部4阶段/Live独立引用且Stopped无plan的分支，不代表缺阶段或所有早期恢复完成。

## 只读准入

当前head必须Stopped、原独立refs精确匹配StartupSession、无restorePlan；原Original/Ready/PreStop/Stopped四帧和所有blob按独立摘要重开，保留原schema/根/权限/原生身份/前驱保护。原restore_evidence从固定scope中的启动前declaredRoutes、原Live的service/launcher PID和HOME审计投影取内容准入参数，当前route声明仍须匹配原scope。过去磁盘Live不是当下退出或写权限。

复用原client_artifact_policy全组三文件补偿分类，所有未知动态路由、catalog/cache/audit格式/内容/审计边界或保护变化仍拒绝。当前Capture必须等于冻结Stopped完整原文/native/change字段；外部写过后即便恢复相同字节，也不重新拍Stopped或接受新ctime。指纹绑定四帧独立摘要、补偿decision、当前精确三文件、declaredRoutes/原PID集合/审计HOME与not_before；not_after只在当次assess用新观察上界验证，不作为UI等待期间静态来源字段，原冻结审计字节不可由此扩充。

原Codex树除3项已分类文件以外的内容/目录身份/安全属性都须匹配Original，Grok完整树也匹配原件；两稳定锁必须原Stopped身份且与Original一致。无plan时不忽略任何stage名字（包括猜测的“本计划”stage）；陌生暂存/其它文件拒绝保留。只在真实NativeLease内复核时允许当前claim journal，最终恢复两锁例外不变。

## 新的当前授权与持久计划

missing_plan.review在原shared action锁内读取原Head/原件并提供固定attempt/revision/digest，无路径/替换内容。prepare在既有Index.open_existing排他action锁内核对原请求，真实完整官方镜像准入和同进程claim/N/L/K锁后重新核对准入，当前HOME完整OS停止、原service/launcher PID及port缺席/运行记录和manager事实沿原门证明。没有从磁盘receipt重建NormalExitProof或扩大原10秒。

调用现有Pending.prepare只生成固定plan和原件暂存，目标config/catalog/cache/audit不被交换或删除；每步lease核对/目录原生identity/权限/原字节与停止后像/EXCL规则保持。释放协调锁、完整镜像后复核后，原索引record_restore_plan只允许同Stopped/同refs/原revisionCAS，一次append事件变RestorePrepared；超时或错误保留原件和现场，不标Restored/不许Start。成功仅restore_prepared，真正文件补偿由另一明确Continue在自己的原10秒执行。

仍存在已知未闭合边界：若Pending.prepare在stage写入或plan落盘后、独立索引登记前中断，本分支会因未登记stage拒绝再次准备，保留原件/暂存，不扫描认领、删除或重拍。该部分准备事务中断需要后继独立归属预留/恢复机制；本批不能称全部中断流程可继续。更早Captured/Ready/StopPrepared缺少完整后像也不人为补齐四阶段。

## 产品和原型

Preview新增canPreparePlan/planDigest，只在Stopped原件Verified、plan/completion未记录、3后继阶段完整、其它恢复能力不适用时给出。Tauri注册client_recovery_prepare_plan，沿既有运行排他租约/更新安装和待重启拒绝/binding-ready及blocking worker生命周期，参数仅attempt/revision/digest，其它平台unsupported。

真实ClientRecovery同UiDialog显示“服务已停止，客户端配置尚未恢复”“原件和停止后文件已核对，可以准备恢复计划”。点准备后明确同弹窗说明：重核原件/停止后文件/停止状态，当前目标文件保持、原件保留；成功后还需另点确认继续恢复，不自动恢复或启动，可返回。Pinia专用preparePlan，只有明确choice才可提交；同promise关闭重开/重复点击不重发，正确回执须同attempt/revision+1/status=restore_prepared。成功后仅以新live同D/H/Stopped/Required/三动作false及同preview revision/phase才展示继续，不清latch；状态读失败仅重查。最后另行Continue返回Restored+fresh live才能清限制。

唯一总原型client-recovery-024.js新增plan/plan-slow/plan-error固定内存流程，按钮/确认/返回/关闭重开/错误/准备完成后停止一致，不读真实文件/API/配置，无其它原型/index变更。

## 验证与首错

60去重相关后端：restore25（初24pass+新proposal用例失败后定向修夹具再通过）、tree9、session26；原普通/rearmed各11实际exit91发布边界保持。新proposal反例验证不写plan/stage、未知路由/外部postimage写入拒绝，恢复旧字节不能重建原change；preplan余项不同于已有plan，任何stage都拒绝。初代码括号编译错误proposal-test.log保留；首次把synthetic scope/PID审计fixture当真实session准入而失败proposal-fixed.log保留，拆开synthetic分类与真实身份验证，不给合成PID产品权限；review可见性错误restore-tests.log保留，改为独立锁内public review供native helper；restore-fixed.log保留错误“写回原字节应重获同fingerprint”断言失败，按原精确后像合同改拒绝，产品不放宽。最终编译/格式/全目标strictClippy过。

82前端/类型/生产build通过：无plan/四阶段/原件证据准入、专用path-freeAPI、明确choice/返回零mutation、错误Restored回执不能替代plan、准备成功仍Required且无自动Continue、close/remount1mutation、刷新失败只读重试、后续明确Continue新revision与状态解除限制。WebKit/Chromium各7软件/唯一原型场景过（明暗1180×760/900×600、确认/返回、慢操作重开、错误保留、准备完成只显示Continue），pageerror空；实际查看软件暗色和真正原型确认截图。生产dist排除DEV窗口handle/固定UUID/mock文本，含真实命令与确认。

一个新隔离Profile使用原样官方2.99，Start8956ms/Status236ms、Stop3529ms在独立Stopped index6后、首次plan/stage前注入中断；原服务与父normal0、4TCP缺席，客户端catalog/cache/audit仍未恢复，原完整assert exit101/failed/Required保持。原4帧/Live独立引用齐备，索引无plan、事务目录无restore-*；没有把当前后像冒新原件。

另进程同保留现场：错revision/digest、外部config编辑均拒绝，原索引不变/外部内容保留，测试自行恢复合成fixture后取新确认；实际prepare3412ms，独立index6→7 RestorePrepared，四项目标文件字节/缺席全部保持，原件暂存与计划已登记。随后另一明确Continue6372ms，index7→8 Restored，原两次各10秒预算不变，不拼预算冒原Stop通过；原Stop失败证据不改写，无重装/服务重启或模型请求。

独立只读原生完整Codex/Grok树SHA匹配新completion，fourFrames/scope/live/plan/receipt摘要匹配index8/8events，保留原index6及全部原件/新plan/归档/35内容引用和15blob；7客户端路径原字节/缺席匹配。实际新状态NotRequired/无PID和port/仅Start。核心真实验证用build-native源/准确binary，之后产品Preview/IPC/UI添加由最终编译/测试另证，核心算法未改；不借早期binary声称新产品GUI→IPC已经原生验收。未建新.app/DMG或候选。

证据[本批目录](../../../.adg/evidence/0.2.24/client-stopped-missing-plan-r1/)及manifest保留所有首错、原中断index6/独立prepare/Continue成功和准确来源、全树/真实状态/截图。OS sandbox拒绝外网和日常HOME，仅loopback，无真实模型或业务请求。自有进程/原端点缺席，3精确新协调DB真实EXCLUSIVE重获释放后和程序/Profile/原件/stage/archive/全部夹具清理；globalL inode/字节/权限/owner保持，可排他重获释放且无sidecar。unit TMPDIR空、browser/Vite退出，归档逐字核对后删除重复临时根，target/dist保留。

下一步计划准备自身的中断归属/恢复、缺少已登记Ready/PreStop/Stopped而客户端已有改动的情况；首次缺根/一般客户端和路由图/完整进程集合/旧revision4首错、Windows贡献集成/NAME/复杂System代理/官方交付验证/README及固定候选/W2继续。Windows远端b2e89665d/0eef59a01无新回交，目标active，不发布。
