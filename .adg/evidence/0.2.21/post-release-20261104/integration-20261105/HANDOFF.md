# 当前：已采用维护运行包，0.2.22发行验收进行中（2026-11-06）

当前实测：Mac17项真实OTA/同一自动新进程8文件读回通过；Windows两种安装器真实OTA42143838979通过；四原包安装格42141381151通过；Mac旧0.2.21读取升级后8文件副本与实际运行包回退上游均通过。12资产签名核验、公开beta0.2.22及Latest0.2.21保持已确认。稳定晋升仍等最终CI42141370911的Windows余项，原后端回归已过。

用户已明确采用本项目维护的2.97.0-post021.ownerquery.2离线运行包，SHA256 bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957。可复现构建与已验包逐字一致，旧版→修复包→失败保留→明确回退旧版的实际安装测试通过；不会自动替换日常来源。上游交付不再是本批前置，公开上游PR仍不执行。

桌面发行号冻结0.2.22；tag源1b5c8402dc340540711efa2c2c2037bce673a479。Release42138917336和同源CI42138893326正在执行，后续提交仅补验收脚本，不覆盖tag。最终原包、相邻升级/签名/更新清单/回退和公开发行仍待真实结果。当前TASK为TASK-OPENCODEX-022-FINAL-20261106，证据入口release-0.2.22-20261106/。Windows候选2定向任务已通过，无需重做；RDP绝对帧率硬件例外及历史失败保留。

下文旧“未采用来源/待冻结版本/请Windows补验”均为历史，不覆盖本段。

---

# 当前检查点：Windows候选2已接收，生产运行包采用待定（2026-11-06）

Windows代码63f9dbe26、文档98625f5a1已接收，117个原始路径逐字导入，manifest113项全部摘要一致；贡献TASK/RCP/CERT闭环有效，仅保留received-runtime副本。集成代码d78094369，apps树ca89905344d0d29e761bcb8ffa24d22769c9cb4c与ca46bc1f3及原MSVC候选相同，七个新增文件均为验收工具。

本轮真实管理器离线托管安装候选2完整版本/来源/2522文件、无效包失败保留、全新渠道改名后首次确认重启、取消、服务失败/同通知重试、其他账号及五份签名日常摘要检查通过。管理器严格配置字节守卫保持；原生实际代际及14个配置观测样本已核对。Windows证据只读断言回放与新增guard反例测试通过，不冒称本机执行Windows原生。

CI42056342409在ca46bc1f3的五项适用job已全部success，发行build按原条件skipped；仅相同产品树限定复用，合并头CI另按真实状态。旧材质结论与用户采用的RDP硬件例外保留，无需重跑本次定向已通过任务。

npm latest仍为上游2.97.0，尚不含候选修复；2.97.0-post021.ownerquery.2仅为已验测试修订，不能自动采用为日常来源。当前剩余：明确生产运行包交付/维护路径，再完成版本冻结及原安装器/升级/签名/更新清单/回退/统一发行。方案见windows-candidate2-targeted-20261105/integration-audit/PRODUCTION-RUNTIME-DECISION.md。共享DMD/IMP保持in_progress；未发布、未合并main、未公开上游PR、未改日常配置或运行真实同步/推理。下文按历史保留。

---

# 当前检查点：本机剩余修复ca46bc1f3，候选2交Windows定向

代码ca46bc1f3，安装器完整SemVer修复；源头provider发现基线补丁已交付，管理器全文件漂移拒绝保持。Mac实际托管安装候选.1/.2、入口/来源版本、失败保留及隔离卸载通过；34安装库、20严格协调、140上游路由、5构建器及12 Mac原生检查通过。CI42056342409待终态，不据旧CI代替新源。

Windows只补：取本轮同源MSVC候选，真实管理器离线安装2.97.0-post021.ownerquery.2，再从全新改名开始首次确认→默认重启→配置引用读回，并保留取消/失败/其它账号保护。详细步骤local-remaining-20261105/WINDOWS-TARGETED.md；旧查询补丁、增量基线补丁、candidate2-review和可复现构建器均在代码分支，制品不能只放Mac/tmp。

候选2归档SHA bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957。上游正式运行包/生产来源采用和最终发行门仍待，不发布/不公开PR/不改变日常配置或真实同步。下文旧轮次保持历史，不覆盖本段。

---

# 当前交接：已安装候选重启链通过，生产交付未完成

最新Windows代码705d69ca2、证据4d754fd85已接收，172项摘要一致；新管理器冷快照就绪修复合入bfcf85f0b。Mac本机31后端/3构建器/fmt通过，新CI42051476515待终态，新增路径Mac原生尚待。

Windows使用明确本地标识2.97.0-post021.ownerquery.1，真实安装后保存/错误重试/待改名继续/默认重启成功，新旧引用及日常摘要有读回。但尚无上游稳定发行；管理器托管安装拒绝预发布版本后缀，首次改名knownModels.updatedAt漂移仍需复核，不能把本地候选通过当整版可发布。完整结果/限制见windows-runtime-delivery-20261105/FINAL-HANDOFF.md和integration-audit/review.json。当前用户未授权公开上游PR或采用新的生产来源，本轮不发布/不改日常配置或真实同步。

旧CI42018439722只覆盖旧产品树，仍是历史有效通过，不能代替新冷快照修复集成验收。下面内容留作历史，不覆盖本段。

---

# 当前结果：477d7edea完整CI通过，运行包交付待（2026-11-05）

CI42018439722已completed/success。Windows普通账号最终EXE通过，菜单三帧稳定/即时命中/五次可信事件、取消/Escape及真实业务回归通过；Windows后端/凭据/运行包升级重装/MSVC/final EXE/Node24及22矩阵均通过。候选artifact13413501369、EXE SHA0ee6a1879b72028605de99a5efad8fca2439cb4531b395c928b9fa7e084ec2f5，内嵌4eb8cae1、apps树a8bb18f9一致；174份最终原生证据原字节保存，一个测试EXE只记哈希。旧4251失败与探针前后回归保持。

剩余关键依赖：上游正式运行包尚无两个owner查询修复，本地源候选成功不能代替生产依赖交付。已完成私有dev补丁/PR预览及限定本机检查，见upstream-runtime-prep-0a30c551/HANDOFF.md；对外fork/PR授权仍待。随后才进入实际默认重启、新发行版本/安装升级/签名更新/回退及统一发布。当前无live CI、不合并main、不发布、不改日常配置或真实同步。用户已采用RDP硬件例外，原绝对性能失败仍保留。

完整证据ci-menu-projection-477d7edea/review.json及validation-summary.json。下文全部保留历史，旧“CI待验/普通账号失败”不覆盖本段。

---

# 当前验证接续：477d7edea（2026-11-05）

上一轮4251e5c52 / CI42011443834最终普通账号原生菜单检查失败：按下前按钮发生36px位移，严格命中守卫拒绝，零按下；其它适用步骤通过。477d7edea仅补等待异步模型projection的真实有效值表，再采三帧/即时命中坐标，移动/遮挡拒绝及单次真实输入保持；20定向/73全协议通过，新CI42018439722已入队，尚无原生通过结论。产品apps树仍a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6，Mac及已完成Windows源证据保持限定复用。

审计和原始失败：ci-menu-projection-477d7edea/review.json；135份原始文件按字节保存，1份测试EXE只记哈希。旧CI状态不覆盖新轮次；正式上游运行包交付与公开PR授权仍未完成，不能提前合并/发布。下面b99a94f10回交结论及更早条目保持历史，最新执行取本段与validation-summary.json。

---

# 最新接收：Windows b99a94f10已集成（2026-11-05）

Windows代码8b9cca86a、证据b99a94f10已接收，883项manifest全通过；集成f97b2a92d / apps树a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6。完整CI42003613998五项适用job成功，Mac面板原生15检查及Rust定向18项通过。Windows切源/浮层/重试与首次精确云端范围已有有效证据，不重复要求已完成补验。

用户在本对话明确采用共享GPU/RDP硬件例外：绝对p95/长帧不再硬阻断发布，原失败数据不改写，不宣称性能优化；功能玻璃/回退/焦点/后台恢复要求保持。原冻结TASK不改，新TASK-INTEGRATION-HARDWARE-20261105承接最新采用。

当前剩余：正式运行包源补丁交付。Windows真实默认重启在本地源候选通过，但两个补丁还没进入@bitkyc08/opencodex正式包。当前凭据无上游lidge-jun/opencodex写权限，未公开提交PR/issue。详见integrated-b99a94f10-20261105/UPSTREAM-RUNTIME-HANDOFF.md、runtime-patch-review.json与review.json。取得可安装集成包后继续隔离默认重启及最终发行原包/升级/签名/回退/统一发布；尚未冻结新版本或合并main。

下文全部保留为历史，旧“仍待Windows切源/首次范围、必须原绝对帧率”等不得覆盖本段当前结论。

---

# 最新CI终态（2026-11-05）

c99071793对应run41913003716完整CI已success。Mac前端71协议/1291前端和Mac后端通过；Windows后端、原生凭据、运行包升级重装、MSVC构建、首轮/最终普通账号EXE、Node24与原始Node22.20安装失败/恢复/路径回归全部实际通过。原始绿CI报告174个文件原字节导入，归档摘要及提取文件SHA已验证，一个测试EXE只记哈希未入仓。新的原候选artifact13366782216已核验，EXE dd6f37cf…1a0184、构建127543d、head c99071793，Windows两条启动记录均同哈希且来源匹配/未提权。详情ci-ready-followup-c99071793/windows-review.json与msvc-candidate/review.json。

RDP性能原预算仍失败，Windows切服务origin快速重开/浮层竞态未有新实屏结果，正式默认客户端隔离重启、云端独有首次范围及发行版本/原包升级签名/回退/发布门仍缺。整版未完成，无完成证书。WINDOWS-FOLLOWUP.md顶部为当前有效候选与剩余项，下面阶段历史不覆盖此结论。

# 最新构建候选与CI（2026-11-05）

已取得MSVC原候选run41904417657/artifact13362926947并核验ZIP/EXE哈希、PE架构、构建87d5137和请求head31a28bfb4源码绑定。Windows首轮实际EXE冒烟通过；整轮后续在安装恢复后的WebDAV旧configured断言失败。c99071793仅修剩余ready消费者并纳入CI，71协议测试通过，完整Rust及ui/src与候选源保持一致。Node22及后续普通账号矩阵尚未执行，整轮不是绿；RDP性能、正式MSIX隔离重启和发行安装升级等门仍未齐。最新提示词固定该候选下载入口，产品未重建冒充新源，EXE未在本机执行。下面保留历史阶段事实，不覆盖本段。

# 最新接续状态（2026-11-05）

用户恢复目标并接受RDP、预算不变。代码31a28bfb4已完成本机面板恢复/加载来源修复、Vue Proxy探针标量投影、Windows驱动合并及MSVC候选留存。产品apps树ab09d800b27ce13e17c46ce68f9ae38f7d4fc21a与Mac原生6a3c7ef2d一致；63协议、1291前端、Rust面板12/Clippy/构建及Mac原生15通过，Rust全库1217通过/0失败/4原条件忽略。Windows新源切端口竞态未验证，RDP性能三项失败保留，正式默认重启仍缺。新回交54932fe8a已版本化保存，corrected RCP4将缺验及隔离失败真实标明；保留旧矛盾原件。报告披露误停日常服务PID30196，缺前后字节，不能宣称零副作用；本机未执行补偿操作。最新事实见IMP-30及两份audit/validation summary；下面为历史阶段交接，不覆盖本段。

# 双线集成交接（未发布）

Windows最终原始回交68c034362已接收，46原始blob/40manifest项SHA一致。详细采用边界见windows-final-code-review/evidence-audit.json。仍为RDP/GNU，正式默认adapter未重启；首次范围0渠道/17模版只能证明部分选择，主模型流程以旧WebDAV断言超时结束。Rust17个Node环境失败均被18定向重跑覆盖，不记为全workspace单次通过。来件RCP总体failed，却把MSVC/物理性能/正式重启checks误标passed，与正文冲突；原件隔离到received-runtime，不采用权威回执、不关闭AC。EXE启动脚本从另一checkout复制，缺原始内嵌commit及哈希拷贝链，运行身份待补。四个新TASK仍in_progress，无新CERT。

代码codex/post021-sync-fixes / c71a8cf1eef28db5de9ba82b5b599bdc1336678f，PR #69草稿。产品源23440509452fc1241324a8ba02ebdd6e52822c18；后续HEAD仅增加两份smoke脚本，apps树0eec77398bbd1cbe49f88dd3bd5e8046266e14c1一致。文档采用仍以DMD-POST021与IMP-30为准，未来原型脏文件未纳入。

Windows代码4b0c357b1已合入；文档911d51514的328独有路径字节导入，原manifest177+137项SHA一致，六TASK/六RCP保留失败及未完。材质恢复的根因与原生截图保留，不把性能失败改成通过。

本机结果详见validation-summary.json。集成前后端1289/1215及定向11、Clippy和构建通过。Mac六材质场景、渠道12、最终同步27与面板15检查通过；同步仅隔离HTTPS GET，业务状态摘要保持；面板为官方2.84原包真实子WKWebView，无脚本替换响应。返回面板410/353ms是复用返回观察，不是冷启动或首像素。Mac管理器通知中心会随外部侧栏点击收起，跨路由带通知的假设不成立；该原生覆盖未宣称通过，Windows既有/最终补验仍负责相应场景。

保留失败：100ms Rust超时测试初次并发请求未到服务，单项和低并发完整复验通过，未放宽deadline；亮低初次无WebArea；测试server stdout放在日常home禁读范围导致Node启动失败；缩写build SHA拒绝；通知入口/默认路由假设失败。最关键的实际产品失败是Mac首次导航前重开读取nil URL崩溃，cf412c336/234405094已修并补原生面板。另一个加载失败来自测试沙箱屏蔽系统mDNSResponder Unix socket：最小WK程序NSPOSIXErrorDomain1，同规则仅放行该socket即成功；最终产品也已独立通过，不用最小程序代验。所有尝试独立UUID根且保留原始报告。

发布门仍未齐：Windows最终同源CI/原生回归、RDP未通过的材质性能（需要物理桌面和MSVC候选包补验）、正式打包Codex桌面默认重启adapter、新版本冻结和原包安装升级/签名/更新通道/回退。没有发行版号变更、main合并、tag/发布、真实WebDAV或日常配置/Codex重启。当前TASK/DMD/IMP仍in_progress，无整版证书。Windows接续用WINDOWS-FOLLOWUP.md；用户已确认有本地Windows环境，采用继续交Windows对话补验；当前宿主工具仅列本机对话，未连接Windows宿主，未实际发送。提示词交由用户粘贴，不据此宣称Windows已开工。


最新交付代码f9def3300640f0871a93b6bbcec55de646e00567仍为同产品apps树，仅校正改名保存的恢复日志/写入前备份探针。35测试通过；Windows实际重跑待证。Windows最终交接以WINDOWS-FOLLOWUP.md新HEAD为准，旧CI失败保留。


接续CI显示名称旧合同失败：bd7c0fe8c实际原生已过剩余A保存及备份、排序/转移/对齐，再于021改名raw.name旧值断言失败。3aa789a6e仅适配name-only的持久化名称，整组59测试通过；产品apps树仍不变。原生重跑、后续未执行检查、Mac runner补跑和Windows实机仍未齐，详见ci-display-name-probe/analysis.json。旧source身份保留在原始证据，最新交付以WINDOWS-FOLLOWUP.md为准。


最终探针接续HEAD9850dbbae：原生新失败为持久化显示名称旧断言，3aa789a6e已修；继续逐项审计发现尚未执行的WebDAV旧探针与本轮启动只读检测冲突，9850dbbae按采用合同预先适配。新build必须有真实失败观察/修订/时间，只有config_sync_observe可额外准入，仍拒上传/下载/应用。历史独立运行ID界面保持旧预期。完整60测试通过，Windows实际重跑仍待，不把源码审计预测写成已复现。产品apps树与234405094不变。


补验注意：新增native-regression.mjs的restart-missing直接调用showSaved，仅能证明人工通知场景下的具体错误/重试，不能当实际保存或正式客户端adapter证据；首次范围脚本当前any/some断言也不能单独证明完整目录首次全选，需要报告精确预期对象集合及读回。原门禁继续，不据代码分支推送视为补验通过。


首次范围驱动已加固：sync-setup前在独立root放expected-first-scope.json，格式{"channelIds":["夹具本地渠道ID","夹具云端独有渠道ID"],"templateIds":["夹具本地模版ID","夹具云端独有模版ID"]}。必须由已知本地/云端夹具生成，不能从待验scope反推；起始scope/initialized文件必须不存在。新驱动逐一比较完整并集、拒少选/多选/重复，并保留读回及期望输入。此驱动覆盖渠道/模版；账户/Skill/MCP仍须对应证据。61协议测试通过，实机执行结果待交回。
