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

# Windows可取用补丁（当前入口）

- 仓库gzers/opencodex-desktop，分支codex/post021-runtime-patch-handoff。
- 提交d5a6aaf991262ef6205e7dd68cefae69d62033d8。
- 文件test/smoke/post021-windows-final/patches/runtime-owner-dev-0a30c551.patch；35,703字节。
- SHA256：0d8eaaec8c361d914fb901db2c8ef6a36702ce3c7ec209cb3f255817f8e32bb7。
- manifest.json同目录含原/后像摘要、上游固定dev和未完项；不在管理器仓库应用补丁，须在隔离上游checkout核对基线后使用。

在Windows现有仓库fetch该分支后，用git archive --format=zip --output=runtime-patch.zip origin/codex/post021-runtime-patch-handoff test/smoke/post021-windows-final/patches，再用Expand-Archive解压及Get-FileHash核验；避免PowerShell文本重定向改写patch编码。Git blob与ZIP提取均已复验原SHA。用户仅授权可取用贡献分支，本次没有创建上游fork/PR/issue或发布。旧/private/tmp路径仅保留本机溯源，不再是唯一交付位置。

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

# 当前Windows接续：完整CI通过后的实际缺验

最新代码c990717931aa37ed66ca99e42bf5cf4f470069bb，CI run41913003716已completed/success，Mac前后端、Windows完整安装恢复/Node22.20/普通账号矩阵都通过。以下为最新有效入口，下面旧候选和失败历史仅留溯源。

候选：windows-msvc-candidate，artifact13366782216。
下载：https://github.com/gzers/opencodex-desktop/actions/runs/41913003716/artifacts/13366782216
新独立临时目录命令：gh run download 41913003716 -R gzers/opencodex-desktop -n windows-msvc-candidate
ZIP SHA-256：abf87a793d96ace994e16f458c42719a58aaedf8e19dfcff9a49f6087367a438。
EXE SHA-256：dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184（15,293,440字节，PE32+ AMD64）。
内嵌/清单构建提交127543d2d0d9c2725f1a357710cd43bc69e3c44d，PR head c99071793，apps树8c34bbc8de7cb8d6202c7c870ead4397bbfc44c1。原ZIP/EXE/复制哈希、构建及源码树均已核验；两条普通账号进程令牌及Node24/22安装恢复实际启动报告与同一EXE哈希一致。不要重建后冒称这份候选，也不要覆盖日常安装。

剩余工作按原授权在隔离环境执行：
1. 此新候选实屏复测服务60444切60446、首次加载快速离开/重开、浮层Hide尚未返回即关闭、正常重开和重试。CI通过不证明该手工竞态已解决；保留当前trace/原生日志/配置字节和截图。
2. RDP已被用户接受，预算保持。performance/performance-modal原30秒采样及后台停绘恢复，保留新增callbackCostMs/dispatchDelay/实际滚动范围；同环境同数据比较前后，采合成/GPU/RDP主线程trace后修瓶颈，不降档/降玻璃/改预算。
3. 用独立expected-first-scope.json证明本地+云端独有渠道/模版完整并集，已有范围保持及新增不扩选；已有remoteObjects0只能证明本地首次。
4. 正式MSIX默认adapter重启只在可证明一次性Windows账号/会话与进程owner/祖先链均为验收实例时执行；枚举null为未知，无法隔离继续只读停点，不用OwnedClient替代。
5. 固定PID清理误停日常Bun事件及更正失败回执保持原件。清理只匹配测试路径/PID/创建时间和归属，不补偿性重启或改日常配置；新回执逐项真实，未通过不生成完成证书。

本机接收原始证据后统一冻结发行版本、验原安装器/升级/更新签名/回退并发布。当前仍不合并main、不发布、不跑真实WebDAV/推理、不操作日常Codex配置。完整绿CI只关闭自动化缺验，不代替上述实测与发行门。

---

最新验收脚本接续c99071793：产品代码不变，仍使用下述已核验31a28bfb4 MSVC原候选，不必为工具更新重建产品。安装恢复后的就绪WebDAV消费者也已对齐启动只读检测：合成.invalid地址应真实失败，不再强要求configured。原首轮EXE冒烟已通过；整轮CI在该旧消费者断言失败，后续Node22/普通账号矩阵被跳过，仍待实测。71协议测试通过，该消费者测试已纳入CI。Windows完整安装恢复/后续矩阵请使用新脚本复验，原失败证据保留；不把新纯测试当原生成功。

## 已取得的MSVC原候选（下一轮Windows优先使用）

CI run41904417657，artifact13362926947，名称windows-msvc-candidate。
下载页：https://github.com/gzers/opencodex-desktop/actions/runs/41904417657/artifacts/13362926947
ZIP SHA-256：b6cb8245b2c1ae0caf7cd98d3f28dfc35412966e0a27e209b1fc264719895792。
EXE SHA-256：b8b2f4a57a05ce036dfd30d51c9339697b036f099f76240f265d99015d07385d；15,293,440字节，PE32+ AMD64。
构建提交87d5137311c716ebadb1c88cc4477ff6fc8a7195（PR合并），关联head31a28bfb4，apps树ab09d800b27ce13e17c46ce68f9ae38f7d4fc21a。内嵌提交、ZIP摘要、EXE原/复制哈希均已核验。

在新建独立临时目录执行：gh run download 41904417657 -R gzers/opencodex-desktop -n windows-msvc-candidate
先核对candidate.json与EXE哈希，再按既有隔离启动方案运行；不能覆盖或启动日常安装目录。7c2559055是验收/诊断脚本源，不是该二进制内嵌SHA；完整Rust和UI产品源码与候选一致，差异仅QA性能诊断。

构建与首轮原生EXE冒烟已通过，整轮Windows矩阵当时仍继续。该文件不是安装器、签名更新包或发布通过凭证。必须据同一候选继续切端口/快速路由/浮层恢复、RDP原预算和可安全隔离的正式Codex默认重启。证据ci-panel-followup-31a28bfb4/msvc-candidate/及windows-candidate-step-review.json。先前误停日常服务事件仍保留，清理须匹配测试路径、PID、创建时间与归属。

最新诊断补充7c2559055：只改apps/desktop/ui/qa/windows-materials.mjs，ui/src与tauri/src均与MSVC候选源31a28bfb4相同。对已核对同源的原生候选继续跑performance与performance-modal原30秒采样，保留失败JSON。新增diagnostics.callbackCostMs、callbackDispatchDelayMs及每秒实际滚动范围，避免“无WebGL draws也超预算”被错误归因为背景动画。回调成本不包含后续布局/绘制/GPU，延迟也不能单独定因；必要时继续采集合成/GPU trace。预算、档位、玻璃和通过条件均未改，此工具更新不是性能通过。详见performance-followup-20261105/analysis.json。

# 最新Windows接续：31a28bfb4

在本对话用户已确认：接受RDP，保留原预算。以下为当前有效补验，下面旧提示词仅保留交接历史。

- 先fetch并合入codex/post021-sync-fixes@31a28bfb4f0decd151269ce896f2a7364fb5b186及docs/governance-main最新IMP-30，沿已有Windows分支继续。新产品树ab09d800b27ce13e17c46ce68f9ae38f7d4fc21a；旧bd7c0fe8c二进制不能代验新面板修复。CI增加windows-msvc-candidate，取该最终源构建候选并核对candidate.json、run/commit/apps树、EXE SHA与运行启动日志。可用同源MSVC CI构建复验，不强制本机安装MSVC；仍不等于发行安装器/升级/签名通过。
- 重点复测60444→60446切源、首次导航未完成即离开/重开、浮层打开后快速关闭与Hide交错、正常重开和失败重试。697b9e382修相关丢失Show和过时/about:blank回调；本机测试及Mac面板已通过，Windows原竞态仍需实测，不提前写已解决。保留请求时序/当前trace/原生日志和失败帧，不以延长超时换通过。
- RDP性能沿B=16.667ms、p95≤2B等原预算，静止/滚动/弹窗至少30秒、前后台恢复。54932fe8a三项失败保留；记录CPU/GPU/DWM/RDP合成负载与主线程/合成/绘制trace，先区分宿主资源争用、远程帧投递、着色器与滤镜成本，再修实际瓶颈，不偷偷改低档、减弱玻璃或调整预算。新候选与原包在同会话同数据/尺寸/档位比较。
- 首次范围用UI开工前独立expected-first-scope.json，含本地及云端独有渠道/模版完整ID并集。来件remoteObjects=0只证明本地首次选中；补后续新增不扩选及已有范围保持。当前驱动拒绝缺项/多项/重复，不回退到kind解析或any/some判通过。
- 正式MSIX默认重启只在能证明一次性Windows账号/会话/进程owner和祖先链归属时执行；listProcesses=null为未知，不能当无进程。不能在日常同包范围直接测试或用OwnedClient代替。若仍无独立实例就报告缺验。
- 误停日常Bun PID30196事件和修订RCP4保留。清理每次匹配测试路径、PID、创建时间/归属，不能固定PID盲清理；不自行补偿重启/改配置。旧原件不覆写，新证据和回执逐项真实，未通过仍failed，无整版CERT。

本机已保存windows-final-followup-54932fe8a完整版本化资料，主189/独立RDP20项manifest SHA一致；新产品验证见local-panel-followup-20261105/。不发布、不合并main、不跑真实WebDAV/推理、不改日常配置。

---

# Windows最终补验提示词

## 68c034362回交后的定向补验

原始证据已接收，审计见windows-final-code-review/evidence-audit.json。继续已有Windows贡献分支，先合入最新集成源f9def3300；不重新创建同名分支，不以旧探针超时报告作最终通过。

1. 使用用户已确认可用的本地Windows控制台完成物理桌面性能/MSVC补验；当前回交仍为RDP。记录Git/apps树、MSVC与SDK、EXE构建commit、复制前后SHA及实际启动日志；不能只从旧feature目录取EXE后声明同源。
2. 正式Codex默认adapter采用一次性Windows测试账号/独立登录会话，先证明目标进程owner和祖先链都属于可丢弃实例，并确认官方adapter的当前用户过滤。从独立PowerShell/测试管理器执行，不能停止日常Codex。listProcesses=null需记录原始exec异常/超时，不能当无实例；不能证明隔离则继续只读停点，不伪造默认adapter返回值或以OwnedClient替代。
3. 用独立expected-first-scope.json重新验证首次本地+云端独有渠道/模版完整集合、已有范围及后续新增不扩选；旧结果0渠道17模版不证明完整并集。账户/Skill/MCP按本轮夹具另补对应证据。
4. 完整模型UI/节点弹窗/保存取消与同步流程重跑至终态，使用已修名称/版本探针。补已知/未知保存时间、无快照、失败状态及HTTP请求/业务文件不变证据；来件只有失败版本状态，缺记录重启截图来自人工showSaved。
5. 材质性能保留原预算和同场景前后矩阵，记录物理/RDP、GPU、DPI、WebView2和制品身份；旧RDP高档两项仅作观察。
6. 原RCP的MSVC/物理性能/正式重启checks与正文矛盾，已被隔离。原件保留，新回执或合法修订逐项填真实结果，不把缺验统一标passed，不生成完成证书。回交原始证据与明确未完，由本机统一收口。

用户已确认有本地Windows环境，继续由Windows对话补验（2026-11-05）。本机工具未连接Windows宿主，尚未实际发往该对话；请粘贴以下内容。

继续gzers/opencodex-desktop本批集成验收。先fetch并读取代码origin/codex/post021-sync-fixes（f9def3300640f0871a93b6bbcec55de646e00567，产品源234405094；已集成faf8dd926补验驱动，保留严格名称/版本合同及不可变根准入，须使用该最终探针），文档origin/docs/governance-main的IMP-30、DMD-POST021及0.2.21-bugs.md第8.7节。由集成分支另开codex/post021-windows-final，避免用原W分支替代L/L2；文档证据另开docs/post021-windows-final，仅写独有证据/新TASK回执，原冻结任务不重绑。

补三类实际缺验：
1. 同最终源回归Windows材质、首次范围/同步方向/ETag有界确认/地址/版本只读观察、名称唯一/无运行标识/无私网前置、重启和面板。新Mac面板空URL修复cf412c336/234405094涉及共享origin复用：补首次导航未完成即离开重开、服务地址变化、旧视图关闭失败/重试。保留代次/超时/浮层遮挡与配置读回。
2. 用物理Windows桌面和MSVC候选正式构建补材质滚动/静止/后台性能；沿用原预算和基线，记录OS/WebView2/GPU/DPI/远程状态。原RDP Server基线与候选均失败，不能改预算或当作通过；没有合适环境就报告具体缺验。
3. 用正式Codex打包桌面默认发现/退出/启动adapter完成独立根正常重启及配置读回；OwnedClient.exe壳只作旧证据，不替代本项。只能操作能证明独立配置和进程归属的实例；无法隔离即回报，不关闭日常Codex。

不发布、不合并main、不跑真实WebDAV、不改日常配置/Key；使用合成数据与自有loopback。提交推送代码/文档贡献、回交SHA/原生截图/检查与未完；最终由本机统一冻结版本、验证原包安装升级/签名更新和发布。


补验注意：新增native-regression.mjs的restart-missing直接调用showSaved，仅能证明人工通知场景下的具体错误/重试，不能当实际保存或正式客户端adapter证据；首次范围脚本当前any/some断言也不能单独证明完整目录首次全选，需要报告精确预期对象集合及读回。原门禁继续，不据代码分支推送视为补验通过。


首次范围驱动已加固：sync-setup前在独立root放expected-first-scope.json，格式{"channelIds":["夹具本地渠道ID","夹具云端独有渠道ID"],"templateIds":["夹具本地模版ID","夹具云端独有模版ID"]}。必须由已知本地/云端夹具生成，不能从待验scope反推；起始scope/initialized文件必须不存在。新驱动逐一比较完整并集、拒少选/多选/重复，并保留读回及期望输入。此驱动覆盖渠道/模版；账户/Skill/MCP仍须对应证据。61协议测试通过，实机执行结果待交回。
