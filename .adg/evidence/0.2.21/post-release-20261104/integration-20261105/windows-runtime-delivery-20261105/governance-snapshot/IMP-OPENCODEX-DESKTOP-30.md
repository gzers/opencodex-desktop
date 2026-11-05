---
id: "IMP-OPENCODEX-DESKTOP-30"
object_kind: "implementation.change"
title: "0.2.21发布后双线修复与集成发布计划"
state: "in_progress"
completion_summary: null
dmd_refs: ["DMD-OPENCODEX-POST021"]
demand_ids: ["DMD-OPENCODEX-POST021"]
task_ids: ["TASK-OPENCODEX-POST021-INTEGRATION-HARDWARE-20261105", "TASK-OPENCODEX-POST021-LOCAL-20261104", "TASK-OPENCODEX-POST021-CHANNELS-20261104", "TASK-OPENCODEX-POST021-CHANNEL-PROTOTYPE-20261104", "TASK-OPENCODEX-POST021-INTEGRATION-20261105", "TASK-OPENCODEX-POST021-WINDOWS-HUB-20261104", "TASK-OPENCODEX-POST021-WINDOWS-MATERIALS-20261104", "TASK-OPENCODEX-POST021-WINDOWS-PANEL-20261104", "TASK-OPENCODEX-POST021-WINDOWS-RESTART-20261104", "TASK-OPENCODEX-POST021-WINDOWS-RESTART-FEEDBACK-20261104", "TASK-OPENCODEX-POST021-WINDOWS-STICKY-20261104", "TASK-OPENCODEX-POST021-WINDOWS-FINAL-20261105", "TASK-OPENCODEX-POST021-WINDOWS-FINAL-PROBE-20261105", "TASK-OPENCODEX-POST021-WINDOWS-FINAL-RENAME-PROBE-20261105", "TASK-OPENCODEX-POST021-WINDOWS-FINAL-SYNC-PROBE-20261105"]
summary: "Windows回交已集成，最终完整CI及普通账号原生通过；仍待上游运行包交付与发行原包门。"
---
# 0.2.21发布后双线修复与集成发布计划


## 2026-11-05 菜单就绪探针最终原生复验通过

477d7edea的CI42018439722已completed/success，五个适用job全部成功，发行build按原分支条件skipped。Windows全部必需步骤包括普通账号最终EXE已通过：原菜单三帧/即时命中守卫、五个可信pointer/mouse/click事件落在Fast按钮、内部菜单与所属dialog的Escape、取消不改配置/备份、改名/通知/WebDAV真实只读失败等通过。普通账号令牌elevated=false，构建4eb8cae1与原EXE SHA 0ee6a1879b72028605de99a5efad8fca2439cb4531b395c928b9fa7e084ec2f5一致；候选artifact13413501369的ZIP摘要01795175…b1faa576、PE架构、apps树和构建父子关系已核验。原生归档13412734814的175项逐摘要核对，174份非二进制原字节保存，一份测试EXE只记哈希。旧4251普通账号36px位移失败与先红后绿测试保留，不降命中/取消/身份断言。

当前CI缺验闭合，正式运行包交付未闭合：两个owner查询补丁在2.84源候选原生通过，但用户可安装上游仍无修复。本机已备固定dev可回放补丁和PR预览，109定向+12兼容启动、类型检查通过；5项Windows专属/完整dev与Git索引门未验，不表示正式包完成。当前公开PR授权仍待，未对外提交。按硬件例外采用保持功能材质结果；下一步上游运行包交付及隔离默认重启、版本冻结/发行安装升级/签名/回退和统一发布。整版DMD/IMP/TASK仍in_progress。

证据ci-menu-projection-477d7edea/review.json、ci-477d7edea-run-final.json及final-artifact-import-manifest.json；validation-summary.json已更新，无正在运行的本轮CI。

## 当前接续：b99a94f10已接收，运行包交付仍待（2026-11-05）

最新验证接续：4251e5c52的CI42011443834最终failure，普通账号model-menu点击前严格守卫拒绝：连续三帧坐标y552.25，鼠标尚未按下时目标上移36px至top499.25，旧坐标落到summary。viewport/scroll未变，零按下事件；保留原始轨迹和135份原生证据，一份测试EXE只记哈希。前端1291/71协议、Mac完整后端、Windows回归/凭据/运行包升级重装/MSVC/final EXE及Node24/22恢复均通过，不能替代最后普通账号失败。

模型详情初始挂载不代表异步projection完成；ModelRuleFields的占位说明在effective到达后消失并出现有效值表。477d7edea仅补原生驱动等待真实“当前草稿有效值与来源”caption，再采坐标；保留三帧稳定、按下前命中、单次真实点击和移动/遮挡拒绝，不加坐标偏移或重试。两个新增反例在修复前失败，修复后20定向/73全协议通过；夹具的急切帧推进已与真实wait先检查的语义一致，原严格帧号断言保持，中间失败留存。此就绪条件修复不冒称已排除所有布局变化，Windows原生仍待CI42018439722终态。产品apps树与f97相同，Mac有效原生证据按未变源复用；上游补丁授权和正式运行包交付仍未完成。审计ci-menu-projection-477d7edea/review.json。


以下为最新采用，后文按时间留作历史。用户当前明确确认“是，采用该硬件例外”；共享GPU/RDP绝对帧率不再阻断本批，旧failed数值及未做生产性能优化的结论保留，功能玻璃/回退/交互/后台恢复仍必需。AC-P21-02已经回写，新集成TASK-OPENCODEX-POST021-INTEGRATION-HARDWARE-20261105绑定新口径；原冻结集成TASK不重绑、不虚假关闭。

Windows代码8b9cca86a与文档b99a94f10按883项SHA核验，888新增路径按字节接收，4份运行对象只隔离到received-runtime，不覆盖权威旧对象。无冲突合入f97b2a92d，apps树a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6。新MSVC首导航手动重试约646ms恢复，切60444/60446快速重开、两组各5轮浮层恢复；云端独有首次精确2渠道18模板、后续2/3及18/19不扩选、明确子集复开保持均有有效原生证据。旧失败/错标签/GNU限制仍保留。

本机最终CI42003613998五项适用job全部success（发行build按原分支条件skipped）；Mac真实官方子WKWebView最终15检查通过、配置字节不变，Rust面板定向18/0与前端构建通过。Windows新候选正式重启在隔离Administrator账号旧桌面48960→新92496，并读回配置65547；这是加两个源补丁的运行包候选，不是原发布包。上游仍未集成，当前凭据对lidge-jun/opencodex无写权限，不能在此仓库合并两个test补丁就宣称修好了生产运行依赖。

后续顺序：先完成可审阅的上游交接并确定补丁提交/运行包交付路径，取得可安装集成包后验实际默认重启；再冻结本批发行版号并执行原安装器、从v0.2.21升级、签名/更新清单/回退和统一发布。禁止静默改npm来源、覆盖上游包版本或直接改用户已装运行包；本次未发布、未合并main、未跑真实同步/推理、未改日常配置。证据integrated-b99a94f10-20261105/review.json。


## 2026-11-05完整CI已通过，实测与发行门仍保留

run41913003716 / head c99071793已completed/success。Windows126865197400全部必需步骤成功，唯一跳过的是不适用的旧ETW诊断分支准备。后端、原生凭据、官方运行包升级重装、MSVC构建、首轮EXE、安装失败/恢复/路径、原始Node22.20同流程、最终普通账号EXE均通过。真实普通账号令牌未提权，启动commit及EXE哈希一致；WebDAV恢复后使用真实version-check失败观察，source/revision/time/可见ready卡片与managed来源保护均通过。旧探针失败及相应原件保留，不抹历史。

新候选artifact13366782216原ZIP SHA abf87a793d96ace994e16f458c42719a58aaedf8e19dfcff9a49f6087367a438；EXE dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184、PE32+ AMD64、15,293,440字节，内嵌构建127543d/head c99071793/apps8c34bbc8均已核验。原始报告归档13367191554摘要bca61b2a…d7b9c76a，175文件逐项SHA核对，174证据原字节保存，一个测试EXE仅记哈希不入仓。Mac前后端同源已通过，Windows完整结果与候选review见ci-ready-followup-c99071793/windows-review.json及msvc-candidate/review.json。

自动化检查缺验现已闭合，但整版AC-P21-06尚未完成：RDP原预算性能仍失败，切端口快速路由/浮层恢复新候选实屏复验、正式MSIX默认重启可丢弃实例、云端独有首次范围、版本冻结/发行安装器/升级/更新签名/回退及发布仍待。旧隔离事件不作零副作用认证。采用WINDOWS-FOLLOWUP.md顶部新候选，后续取得真实证据再推进G3/G4，集成TASK/DMD/IMP保持in_progress，无整版CERT或发布。


## 2026-11-05 MSVC原候选已取得（仍非整版通过）

新面板产品CI run41904417657已产生artifact13362926947，windows-msvc-candidate。下载原ZIP摘要b6cb8245b2c1ae0caf7cd98d3f28dfc35412966e0a27e209b1fc264719895792与GitHub元数据一致；EXE摘要b8b2f4a57a05ce036dfd30d51c9339697b036f099f76240f265d99015d07385d，15,293,440字节、PE32+ AMD64。二进制内嵌87d5137311c716ebadb1c88cc4477ff6fc8a7195，父head31a28bfb4，apps树ab09d800已核验。review-candidate.py实际检查归档、清单、原/复制哈希和构建源码绑定；本机未执行Windows EXE，二进制仅留独立tmp，不提交文档仓库。

后端回归、原生凭据、运行包升级重装、MSVC构建、候选留存及首轮实际EXE冒烟步骤均success；后续安装恢复路径、Node版本及普通账号矩阵仍在执行，不写整轮绿。候选身份通过不代表RDP性能、正式默认重启、发行安装器或OTA通过。

WINDOWS-FOLLOWUP.md已给原候选入口和完整哈希，Windows据同一二进制继续切源竞态、RDP原预算及正式默认adapter复验。证据ci-panel-followup-31a28bfb4/windows-candidate-step-review.json及msvc-candidate/review.json。整版任务/证书停点保持。

## 2026-11-05恢复接续：Windows补充回交与本机修复

用户在本对话重新启动目标，并明确“接受 RDP，保留原性能预算”。本批材质验收允许RDP，替代下文历史物理桌面前置；不豁免p95≤2B、长帧、背景停绘/恢复、MSVC真实制品、正式客户端默认adapter和发行门。MSVC证据可来自同源CI，不额外要求在本地安装cl；需要取得同一原包候选并完成原生复验，CI编译本身不代表安装/升级或性能通过。

接收代码b2cb82cde与文档54932fe8a，版本化保存196份原始资料，主manifest189项及RDP清单20项SHA均一致，未覆盖68c034362旧原件。更正RCP revision4将isolation/MSVC/native/default-restart/performance真实标为failed，原矛盾回执继续保留；新回执先作为收到的运行资料，不据局部结果关闭原任务。完整模型UI、改名备份/镜像、版本四态及渠道真实清单/保存/重开有新增证据。首次范围仅本地1渠道17模版、remoteObjects=0，仍未证明云端独有对象并集；本机保留独立expected-first-scope清单校验，未采用来件按kind字段解析云目录的弱断言。

RDP高档30秒静止/滚动/弹窗p95约162.6/144/171.7ms，均超33.33ms；后台停绘及恢复通过，性能仍失败。面板60444切60446且10/30ms快速路由返回时失败，真实重试后恢复，根因尚不能直接归于空URL。报告披露固定PID清理误停日常安装目录Bun PID30196，后观察新服务66908；本对话已明确披露，缺越界前后日常配置字节，不能认证零副作用，未执行补偿重启/日常配置改写。后续清理必须同时证明测试路径、PID、创建时间和归属。

本机在原实现新增测试，复现“浮层关闭先于原生Hide返回，恢复Show被busy判断丢弃”。697b9e382保留最后一次待恢复Show、每命令独立revision、离开页面不重放，并过滤非当前子视图/非受限origin/about:blank的加载回调。该复现解释相关恢复竞态，不冒称完整Windows切源失败已解决。原生Windows需新候选重验。cb423a9df修正上轮探针直接返回Vue Proxy导致CDP取回空对象的问题，保留revision/source/time等严格断言；真实V8接口复现并回归。6a3c7ef2d集成渠道/面板驱动，保留独立首次范围合同。

当前交付31a28bfb4，产品apps树ab09d800b27ce13e17c46ce68f9ae38f7d4fc21a。前端197文件/1291测试、类型/构建、63协议测试、Rust面板12及Clippy通过，旧失败日志保留。Mac原生候选6a3c7ef2d与最终apps树一致，官方原包子WKWebView加载/重开15检查通过；不代替Windows切源或注入浮层竞态。Rust全库复验1217通过、0失败、4按原条件忽略，原始日志已留。CI新增windows-msvc-candidate，构建后留原EXE副本、原/复制SHA、构建commit/请求head/apps树/target；不是安装器或更新签名，不发布。

证据：integration-20261105/windows-final-followup-54932fe8a/integration-audit.json与local-panel-followup-20261105/validation-summary.json。下一步先交付新MSVC候选做Windows切源/浮层恢复、RDP性能剖析、正式客户端在一次性账号中的默认重启；其后再冻结发行版本和原包验收。整批TASK/DMD/IMP保持in_progress，未生成整版证书。

## 2026-11-05最新Windows回交审计

已接收文档68c034362的46个原始blob，40项manifest校验全部一致。静态apps树与f9def3300一致；报告中的d55e6064是Rust src子树。启动脚本从另一checkout复制EXE，缺内嵌commit原始启动日志和复制前后SHA链，运行身份仍待补。

地址分隔、读取失败文案、人工showSaved缺记录错误与重试有部分证据。首次范围结果为0渠道/17模版，主模型流程以旧WebDAV configured状态超时结束，不能计完整通过。Rust981通过/17失败/4忽略；17个失败均被18项定向重跑覆盖，保留原始失败，不称全workspace单次通过。RDP高档两项p95为31.5/28.8ms，仍不代替物理桌面前后矩阵；当前无MSVC/SDK。正式默认adapter仅发现MSIX，listProcesses=null表示未知，并未执行重启。

来件RCP总体failed、全部AC failed，却把MSVC/物理性能/正式重启checks标passed，与HANDOFF及source-manifest矛盾。原字节隔离至windows-final-code-review/received-runtime，不采用到权威RCP目录；四个TASK仍in_progress，不生成CERT。部分证据不能关闭任何十项整版门。

审计见integration-20261105/windows-final-code-review/evidence-audit.json；WINDOWS-FOLLOWUP.md给出物理桌面/MSVC、一次性Windows账号隔离正式adapter、精确首次范围、完整UI与版本状态、运行身份及回执修正要求。原始资料保留，后续接收真实补验后统一发布。

2026-11-04。范围/验收取[DMD-POST021](../01-需求管理/需求/DMD-OPENCODEX-POST021.md)，逐项状态回写[原问题清单第8节](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.21-bugs.md)。本计划接续本轮用户的双线决定，旧IMP-29不改。修复版本待集成冻结；分支名使用post021批次，不预设0.2.22。

## 分工、分支与写集

两线从共同代码基线7a8441f7e647e4b3540d5907e44ac4f186f90998建立分支，先取远端并核对；复用合适工作区，保留其它对话未提交内容。下列分支为计划名，建成后才记实际提交。

| 执行线 | 问题与验收 | 计划分支 / 责任写集 | 回交与停点 |
| --- | --- | --- | --- |
| W：Windows材质 | S21-POST-02，AC-P21-02 | 代码codex/post021-windows-materials。独占公共材质tokens/materials/effects/base样式、UiDialog/AppModal/ModalContent及确需调整的共用浮层材质实现/定向测试。原生窗口/渲染配置若确需变更先列具体路径；与主入口等共享文件冲突时回交最小补丁由集成者落地。 | 修复、Windows原生视觉/交互/性能定向验证，提交推送代码分支。文档使用docs/post021-windows-materials贡献分支，只写本线TASK/回执及独有证据目录。回交分支、SHA、根因、前后截图、检查/失败及未完；不合并main、不发布。 |
| L：本机同步与集成 | S21-POST-01/03/04/05，AC-P21-01/03/04/05；最终AC-P21-06 | 代码codex/post021-sync-fixes，复用本机ocx-0219-fixes工作区。负责sync/model_config后端与命令、同步业务组件/store/observation、OverviewRoute/SyncRoute、启动只读检测和对应测试。独占原问题清单、DMD/IMP、适用原型差分、版本/CHANGELOG/发布配置及最终治理回写。 | 先完成本机四项及Mac验证，等待W回交；统一集成、跨平台最终验收后发布。Windows集成后原生复测可再交W执行，但最终结论由L回写。 |

Windows不是仅改同步弹窗，也不是接管同步方向逻辑。L不并改W负责的公共材质；W不改同步判断、范围/版本元数据、业务状态/启动流程。OverviewRoute/ModelTransfer等业务文件若只需材质局部修改，W回交选择器或补丁，L合入，避免两端直接改同一文件。共用依赖/配置/主入口由L维护，不因某条线方便而自行升级工具链。

## 唯一顺序与依赖

| 阶段 | 工作 | 完成条件 |
| --- | --- | --- |
| G0：计划与交接 | 固定上述代码基线、DMD采用边界、双线写集与Windows提示词；文档提交可由Windows取用。开工分别建真实TASK并绑定本IMP和适用AC。 | 本次只完成计划/交接，不提前写实现通过；原型差分需要时按适用项补齐。 |
| G1-W：全局材质排查/修复 | 在Windows实际管理器中建立表面消费者清单，复现无磨砂；定位变量/覆盖、native dialog顶层/遮罩、WebView2合成和渲染环境中的实际原因，再修共享配方或最小平台路径。 | 原因有原生前后证据；正常高/中档真实磨砂，低档/无法模糊有可读实底；实际受影响表面均覆盖，不以一张弹窗截图关闭整项。 |
| G1-L：同步四项 | POST-01先验证真实UI范围初始化和内容变化，POST-03先注入首次提交/读回异常；联动POST-05区分版本语义及启动只读检测；POST-04修显示拼接。 | 隔离存储/凭据/测试服务中正常、取消、失败、并发与重开读回符合各AC；保留反例。 |
| G2：各线自验与交接 | 两条G1可并行。L完成适用Mac原生及前后端检查；W按下方矩阵完成Windows原生并推送两类分支/提交。 | 每线提交身份、实际运行身份及证据一致；无关变化不混入；原问题保持待集成，不以单线完成关闭整版。 |
| G3：统一集成与跨端复验 | 依赖G2-W与G2-L。L审查并合入W代码/独有文档证据，解决冲突；重新冻结最终源，并在同一提交重跑受影响检查，Mac验证共享材质，Windows复验材质和四项同步业务。 | 五项全部实际通过；合并后如再改共享实现，补受影响两端验证。缺Windows回交或原生复验时不能提前发布。 |
| G4：版本/原包/发布 | 依赖G3。统一冻结未占用的新发行版本，保持发现版0.2.21和原问题编号；维护版本文件、CHANGELOG、PR、构建/签名及更新清单，完成原包安装和从正式0.2.21升级后读回，保留回退依据，再由L统一发布。 | 按本次有效授权及仓库发布门禁执行；制品、tag、更新通道与最终源码一致。旧发布证书和开发包不代替新原包验证，W不得单独发版。 |

## W线全局材质验证矩阵

先按真实源码消费者盘点“应使用玻璃配方”的表面，并记录适用/不适用原因；不把无模糊设计的页面外壳强改玻璃。至少覆盖：

| 表面 | 代表消费者与必要观察 |
| --- | --- |
| 原生dialog/公共模态 | 同步预览、WebDAV连接、导入/导出、节点检测、设置/安装/更新等实际已有宿主；嵌套、滚动、X/Escape/焦点守卫保留。打开查看窗口不发真实同步。 |
| 通知/Toast/任务卡 | 已有通知中心、保存通知和任务进度；材质一致，计时/主动暂停及关闭观察语义保持。 |
| 下拉/快捷浮层 | 实际UiSelect/菜单/快捷浮层，核对portal/Teleport与不同背景；不只改dialog。 |
| 卡片/面板/控件 | 使用同一玻璃变量的卡片、面板、按钮/标签/选择器和吸顶面；核对层叠采样，不能出现局部修好、其它表面仍清晰透底。 |

每类覆盖亮/暗与高/中/低档；可在同场景同时采集多个表面减少重复操作。额外核对前台/失焦/恢复、关闭重开及切换档位；以100%为基线，补现有缩放边界，字号和布局不扩改。记录实际Windows版本、WebView2版本、应用commit/build、显示缩放、档位与GPU/硬件加速/远程桌面等可取得环境信息。只声明实际测试的系统和环境，不把缺验环境写成通过。

CSS.supports/计算样式/无头浏览器通过只作辅助。原生同场景修复前后截图中应能分辨背景细节是否真的被模糊；必要时提供隔离高对比背景和像素采样。回退应验证真实适用条件或明确隔离注入，不能只强设低档冒充运行时故障回退。保留已有性能预算，检查静止/滚动/后台明显卡顿或持续重绘，避免为玻璃引入无界轮询或全页滤镜。

## 回交与集成约束

- W代码：推送codex/post021-windows-materials，只含本线实现/测试；可建草稿PR但不自行合并。不得写docs/.adg到代码分支。
- W证据：在docs/post021-windows-materials写.adg/evidence/0.2.21/post-release-20261104/windows-materials-20261104/，包括HANDOFF.md、表面盘点/矩阵、环境/源码身份、修复前后截图和测试结果；本线运行控制对象使用独有ID。不要修改共享原清单、DMD/IMP、索引或旧证书；这些由L在集成时统一回写。
- L文档：继续docs/governance-main，只提交本批明确文件；其它对话原型/协作包脏改动保留，不整目录暂存，不以当前原型HEAD整包代替冻结采用集。
- 若发现真实根因超出允许写集，W完成独立取证并交回具体最小修改建议；保持已授权范围内进展，不静默修同步逻辑或新加产品功能。
- 所有执行使用隔离应用数据、合成配置/凭据和测试网络。不修改日常配置、不跑日常真实WebDAV、不重启日常Codex；公开发布前仍完成本计划全部集成门。

本轮交接就绪不等于Windows已开始。交接时计划为planned、task_ids为空，以下接续只记录实际启动与验证；不推定W线已执行。


2026-11-04本机线启动：用户明确“接下来可以进行本机的任务，目标模式进行”。L线代码分支已从约定main / 7a8441f7e建立codex/post021-sync-fixes；负责AC-P21-01/03/04/05，Windows材质独占写集不改。两线最终集成/发布门保持，当前不计AC-P21-02或06完成。


本机G1-L/G2接续：已建立TASK-OPENCODEX-POST021-LOCAL-20261104，绑定AC-P21-01/03/04/05及本IMP。适用原型增量已采用并完成纯内存/双引擎检查。实现进入codex/post021-sync-fixes；前端全量1278项、双引擎软件各16项及Clippy已通过。Rust全库首次1208通过/1失败/4忽略，唯一失败为未改动about/remote.rs的200ms进程夹具PID文件缺失，保留原日志并独立复验/受控并发重跑。Mac隔离原生尚待，W及整版集成发布尚待；不提前关闭。


2026-11-04本机L线验证回写：产品提交f8c34cfcc04ffd9f2b1aa8c035986eae5b6887c3已推送codex/post021-sync-fixes，草稿PR #69。首次目录先初始化及云端独有模版ID保留、ETag发布后只读确认、地址分隔与版本只读观察四项已实现。Vitest1279、Rust全库1209通过/4按原条件忽略、Clippy/构建及3项启动/层边界通过；软件双引擎各16、原型各7项，最终Mac候选原生27项通过，14次GET/零远端写、业务摘要保持。根因与验证限制以.adg/evidence/0.2.21/post-release-20261104/local-sync-20261104/validation-summary.json和HANDOFF.md为准。原始失败已保留并按原因修正或复验，未降低保护；用户真实Windows现场与真实服务响应头仍未采集。远端CI当时待运行完成。本机局部验证完成不关闭五项整版反馈；Windows材质及最终同提交双端/版本/原包安装升级/发布门继续待办，DMD/IMP保持in_progress。

本机最终交接HEAD f3289e2c11fb670f156c2c44237109f0b95afe38仅补实际挂载范围流程的测试，产品源与原生f8c34cfcc相同；49项受影响测试通过。L线验收回执按本批TASK关闭，DMD/IMP与五项整版问题保持待集成；不包含W材质和AC-P21-06。


## 新五项执行分工与依赖（2026-11-04，当前采用）

范围取DMD-POST021新增AC-P21-07至10，逐项状态仍写0.2.21清单8.4/8.5。用户本轮要求先回写及提供Windows提示词，尚未进行新实现。以下是对原双线责任的明确扩大，优先于上文旧“W不改业务”在新增责任路径内的限制；原同步/材质分工其余部分保持。

| 接续线 | 新问题 / 验收 | 允许写集与交接边界 |
| --- | --- | --- |
| L2 本机渠道与发现 | S21-POST-06/07/08，AC-P21-07/08 | NewChannelDialog/ChannelEditor及渠道命名/校验、compiler/config_file_mapping中适用内部身份逻辑、discovery/measurements及对应API/错误映射和测试；移除用户运行标识门槛与渠道私网准入。沿本机代码线接续，原本机同步TASK已关闭，不重开或改绑定；开工另建TASK。公共材质仍不改，涉及共享入口由集成统一处理。 |
| W2 Windows重启与面板 | S21-POST-09/10，AC-P21-09/10 | 沿codex/post021-windows-materials接续但提交与原材质问题分开。允许修改cache_refresh及必要management/client重启相关路径、SaveResultNotice、commands/panel.rs、PanelRoute、features/panel及专属测试/原生驱动。模型命名/发现/同步范围与版本逻辑仍由L独占；app根store、入口/默认配置等共享文件先回交最小补丁，由L集成。 |

- W2开工从docs/governance-main读取本节和新AC，另建独有TASK绑定AC-P21-09/10；不得修改原W材质或已关闭L任务的冻结验收。普通保存/改名、服务未运行/运行但未开面板/已开面板、运行记录缺失/过期须分别复现，核对实际错误路径；不凭用户猜测断定“未打开面板导致”。
- 重启修复与面板性能分笔提交、分开报告根因及验证。面板先采点击至可见的阶段耗时，覆盖建子WebView的show返回前等待；修复后用相同条件比较，不只加长超时或重启服务。Windows原生实际运行身份、前后证据与恢复验证齐全后回交。
- W证据继续docs/post021-windows-materials，在原独有证据目录下新增restart-panel/子目录，含HANDOFF、问题映射/环境、阶段耗时、原始失败、前后截图和实际重启/读回。L统一回写原清单、DMD/IMP与索引。
- L2与W2可并行，依赖适用原型和各自任务绑定；原G3改为等待原W材质、L2、W2及原L交付全部满足后统一合并。同一最终提交补Mac受影响重启/面板及Windows渠道/发现/同步/材质回归，原G4仍最后统一发行，不让一条贡献线独立发布。
- 本轮不改产品、不自动扩大到日常配置/秘密/真实WebDAV或重启日常Codex。新增功能使用隔离根/测试网络和自有候选验证；不为新决定倒改旧证书，修复版本尚未冻结。


2026-11-04 L2已获用户“同意进行，目标模式执行”授权，沿codex/post021-sync-fixes / f3289e2c接续。新增TASK-OPENCODEX-POST021-CHANNELS-20261104绑定AC-P21-07/08；原L任务保持完成，W2与材质独占写集不改。先采用适用原型增量，再完成公共渠道/发现实现和隔离/原生验收，最后提交交接，不独立发版。


2026-11-04 L2实现/验证接续：0e604cb2b6be706af767d9937532fe056cac7ca6已推送本机代码线。名称唯一/内部键管理及渠道模型获取/节点检测取消私网门槛已实现；适用原型59项及双引擎各4项，软件双引擎各12项、Vitest1280项、候选Mac原生12检查通过（2次实际loopback模型清单GET、零生成）。Rust全库最终重跑及Clippy最后复核仍在运行；初次旧显示名称不写raw断言已按新合同改为只更新name。详见.adg/evidence/0.2.21/post-release-20261104/local-channels-20261104/。原L任务不重绑，当前两项新任务仅在证据齐备后关闭；W与整版集成发布不在本机局部验收内。


2026-11-04 L2最终本机回归通过：代码0e604cb2b6be706af767d9937532fe056cac7ca6已推送，Vitest1280、Rust全库1211/0失败/4原条件忽略、Clippy -D warnings与构建通过；59条适用原型回归、双引擎原型各4/软件各12及Mac实际原生12检查通过。两个L2任务按自身范围关闭，本批DMD/IMP及十项整版仍待W回交/最终同提交集成。远端CI41845962425前端、发布工具、Windows冒烟已通过，双后端仍待结束，不据本机结果代写远端通过。未发布，未操作日常配置或真实渠道/同步。


2026-11-05集成接收：用户报告Windows已推送；已取代码4b0c357b1和文档911d51514。W交接明确材质性能失败（远程Server原包也超预算），真实正式Codex打包壳默认adapter尚未验；不据用户“完成”覆盖这些缺验。代码两线无冲突合入本机集成分支；328份Windows独有文档/证据按字节导入，未stash/reset并行原型。PR69 Windows失败实际为旧探针寻找已退役运行标识，需适配后重跑最终源CI。正式集成任务TASK-OPENCODEX-POST021-INTEGRATION-20261105承接全部采用AC及发布门，未完成。


## 2026-11-05最终集成复核

- 实际代码：W/W2 4b0c357b12d331de8dd49f8209056394e93a728d无冲突合入L/L2，合并提交56acb5a59；aa54d4b2a修正退役运行标识的Windows/Mac原生探针，不降低配置读回/相邻对象保护。用户报告已推送不等于贡献验收全通过。
- 实际文档：从W文档911d51514按原始字节导入328个独有路径，提交ba61be83e；材质manifest177项和W2 manifest137项全部SHA一致。六TASK/六RCP合同有效，原失败/未完成保留，无CERT伪造；未合并或暂存并行原型内容。
- Mac首轮集成aa54d4b2a：亮暗×高中低六场景原生观察完成，节点居中弹窗X/Escape、详情、保存通知与缺runtime-port.json具体错误/重试已验。亮色低档首次无AXWebArea，保留失败后同包新根复验通过。Vitest197文件/1289项、类型/构建/生产滤镜检查、Windows探针34项通过。
- Rust首次全库1214通过/1失败/4忽略，100ms端点超时测试在并发压力下请求未到自有服务；单项复验通过，未改100ms与路径断言；同产品源低并发全库1215通过/4原条件忽略，保留原始失败。
- 新增Mac面板阻断：实际候选在首次导航未完成时再Show读取existing.url()，触发wry-0.57.0对WKWebView.URL为nil的unwrap并退出。cf412c336改用已限定的服务origin身份；234405094补齐旧view关闭失败时保留归属，防重试误复用；面板11项通过。官方页面加载被测试沙箱拒绝的现象另行隔离，不把它冒称产品修复：最小WK程序在同规则下NSPOSIXErrorDomain=1，放行单一系统mDNSResponder Unix socket后加载成功，公网IP仍拒绝。实际管理器最终原生重跑结果见同源证据，不据最小程序替代。
- 发布停点：W材质远程Server性能预算明确失败（原包亦失败），缺可用物理桌面/MSVC正式包补验；W2只验证自有GUI壳，正式Codex打包桌面默认发现/退出/启动adapter未验。最终源Windows材质/同步/新建发现/面板回归、CI、版本冻结、新原包安装/升级/签名/更新清单/回退仍待。新Mac面板修复也需Windows受影响复验。G3/G4、DMD/IMP及集成TASK保持in_progress，未合并main、未发布。

证据统一在.adg/evidence/0.2.21/post-release-20261104/integration-20261105/；原始运行身份及失败不可覆盖。L旧已关闭任务/证书不重绑，W冻结回执不改写为通过。下一步先完成可执行Mac最终复验，再交Windows按集成源补缺项；不得用历史0.2.21发行证书代替本批。


2026-11-05 Mac面板最终补验：产品源23440509452fc1241324a8ba02ebdd6e52822c18的实际候选通过15检查（native-panel-user-entry/），含官方2.84原包子WKWebView加载、两次离开/重开、双WebArea和配置不变。返回路由至AX就绪约410/353ms，均为复用返回，不冒称冷启动/首像素。Mac通知中心在侧栏路由点击后按现有合同收起，未注入store制造带浮层场景，Mac该原生覆盖未计；Windows既有证据与最终同源补验继续保留。两次错误的通知触发假设和首次默认路由假设失败均保留。Clippy最终源通过，面板11定向项通过。交付HEAD c71a8cf1e只新增两份验收脚本，apps树与原生234405094逐字相同；不重新构建冒充新产品。后续Windows交接详见integration-20261105/WINDOWS-FOLLOWUP.md。


2026-11-05最终同步原生复验也已通过27检查：候选234405094，真实UI首次全选、只读版本各状态与连接显示有证据，隔离HTTPS14次GET、零远端写、业务摘要保持。详见integration-20261105/validation-summary.json；同HEAD仅验收脚本变化不影响产品身份。材质/渠道及全库检查按已核对未变路径复用，最终面板/同步及定向11/Clippy补齐受影响项，不宣称所有检查重新跑在c71a8cf1e。远端CI仍按实际状态，Windows及发行缺验未关闭。


2026-11-05继续审计CI：41862893068 / aa54d4b2a的Windows最终EXE冒烟在剩余A改名保存后报备份目录数2≠1；实际原生日志已保留。源码表明改名事务另存rename-UUID恢复镜像，不应将其与唯一写入前config.json备份混算。bd7c0fe8c仅修探针：普通保存仍严格一份备份/零恢复日志，显式改名必须额外恰一份格式正确的恢复日志，并校验配置精确前后镜像；旧备份/取消/相邻对象断言保留。35项探针测试通过，含错误postimage/未知文件反例。旧artifact没有沙箱备份镜像，暂不据源码推断实际内容通过；最终Windows原生复验仍待，新CI按新HEAD运行。产品apps树未改、Mac证据有效；发布门未关闭。原始失败及分析在integration-20261105/ci-rename-probe/，Windows提示词已指向新HEAD。


2026-11-05用户确认有可用本地Windows环境，采用继续交Windows对话补验。接续范围仍取WINDOWS-FOLLOWUP.md：最新代码bd7c0fe8c、同源Windows原生/CI、物理桌面材质性能/MSVC构建、正式客户端默认重启adapter；不绕过原门禁。当前本机宿主对话工具只返回local，无Windows对话地址，提示词已备妥但未实际发送/未据此写已开工。接收真实提交和原生证据后由L继续最终发布收口。


2026-11-05 CI基础设施缺验：最终HEAD bd7c0fe8c / run41868797161的Mac frontend126709885314、backend126709885804均未获runner/未执行steps，于16:50:25Z被取消；GitHub annotations明确macOS ARM容量不足，多次分配失败。按原job单独重跑返回不能重跑，父run中Windows仍运行，故待终态再补原检查，不换架构、不降测试、不把取消写通过。Windows本轮后端回归仍进行；前一同产品源Windows回归、凭据保护、官方运行包升级/离线重装已通过但最终EXE仍编译，不替代最新探针验证。证据ci-capacity/review.json；Windows实机与全部发行门未变。


2026-11-05 CI续验：最终提交bd7c0fe8c的Windows后端回归、原生凭据保护和官方运行包安装升级/离线重装步骤已过，release EXE编译仍在执行，最终原生冒烟未有结论。本机补跑与frontend CI相同的七组CDP协议套件共58项通过，日志integration-20261105/ci-rename-probe/full-cdp-tests.log；只补探针回归，不替代Windows原生/远端Mac runner缺验。未因运行慢或临时API EOF重启/取消原任务，仍查询run41868797161/job126709886228。


2026-11-05 Windows最终EXE CI出现下一处旧合同失败：run41868797161 / bd7c0fe8c已通过后端、原生凭据、运行包安装升级和release构建；原生排序、剩余A保存/精确备份及恢复镜像、转移与UI对齐通过，之后021渠道改名读回要求raw.name仍为旧值，实际为采用后的新名称。现场差异记录在ci-display-name-probe/；3aa789a6e只修探针，按真实界面有无独立运行ID区分历史合同和name-only合同，后者严格要求name=新名称，整个文件/Key/模型引用/稳定ID/未知字段保护不变。整组59测试通过，含旧/缺名称、Key丢失及未知/相邻对象变化反例；不以纯测试代替新原生重跑。后续021检测/WebDAV尚未执行，Mac CI仍缺runner待补；Windows提示词指向新HEAD，同产品源Mac证据继续有效，发布门未齐。


2026-11-05同批探针消费者审计：尚未实际执行的旧WebDAV探针要求启动无观察且IPC不得config_sync_observe，与已采用L线bootstrap只读检测冲突。9850dbbae按已核对的原型/产品合同预先适配，历史独立ID界面仍验旧预期；当前name-only界面必须产生合成.invalid端点的真实读取失败、正确revision/source/观察时间、零伪造保存时间。观察器只显式额外放行config_sync_observe，上传/下载/应用及测试连接继续拒绝；未注入store/替换IPC。完整60协议测试通过。这是源码审计发现而非新原生失败复现，最终同源原生仍待。Windows交接已指向9850dbbae，产品树与234405094相同，原失败/局部通过均保留。


2026-11-05接收Windows最终补验代码faf8dd926（源自bd7c0fe8c），合并为be337d5b3：有界可见/启用动作等待及新原生回归驱动已采用；重叠显示名称与启动版本检查保留本机历史/当前双合同及较严格revision/settled/time保护，没有退回无条件更名或弱状态校验。驱动补assertModelSandbox不可变根准入，人工showSaved通知场景写明边界。60协议测试及语法通过，apps树仍与234405094一致。Windows文档/原生报告分支尚未取到，不计补验完成。审计另记：人工保存通知不能证明真实保存/正式adapter；首次范围any/some检查不足以证明完整并集，回交须有精确对象集合证据。详见windows-final-code-review/review.json。新提示词指向be337d5b3，材质性能、正式重启、最终源原生/CI和发行门继续待办。


2026-11-05补验驱动审计处置：f9def3300将新设备首次范围检查由any/some补为独立夹具完整channelIds/templateIds并集比较，要求UI开工前无scope和initialized，并读入expected-first-scope.json；缺项/多项/重复均拒，期望文件不变。61协议项通过，原生仍待执行；不把仅渠道/模版范围的驱动扩称账户/Skill/MCP全部通过。产品apps未变；新交接说明已给出夹具清单格式，人工showSaved和正式adapter缺验证据边界保留。


2026-11-05 CI队列整理：最终f9def3300 / run41879603626完整五项CI保留。核对三个祖先提交3aa789a6e、9850dbbae、be337d5b3的runs41878328232/41878677161/41879180603及全部jobs均为queued、无steps后，提交取消请求清理过时重复排队；API返回取消请求已受理，但后续查询仍queued，尚不称已取消。没有终止最终运行或把取消计为通过，未修改运行器/工作流和测试范围。回执ci-capacity/superseded-queue-cleanup.json。Windows最终原始报告/性能与正式adapter证据仍未收到，代码分支仍faf8dd926，待真实回交。


2026-11-05最终源Mac前端远端检查已有实际通过证据：run41879603626 / job126748378977，检出PR合并提交8d764df0b7034aa61e719d1121533f6d354b9db1（父7a8441f7e、f9def3300），已fetch验证apps树与最终head完全一致。61协议、197文件/1289 Vitest、typecheck/build与Complete job步骤均success，日志于17:44:23Z结束；job列表仍延迟显示in_progress，因此只确认这些已执行检查，不据此写整轮CI绿。证据ci-final-f9def3300/frontend-review.json及原始解码日志。最终Mac后端剩余workspace和Windows后端回归仍执行，Windows实机原始报告未到，整版仍待验。旧缺容取消保留，未更换运行器或降范围。


2026-11-05最终源Mac后端CI也已completed/success（run41879603626/job126748379204），检出同一PR合并8d764df、apps树与f9def3300一致。格式/Clippy、单独有界元数据进程守卫及剩余workspace回归通过；库1215通过/0失败/4原条件忽略/1已单跑排除。原始日志与backend-review.json已保存。Mac前端任务也已completed/success，最终Mac远端检查齐备；不据此关闭Windows原生/性能/正式默认adapter和发行门。


2026-11-05性能原始数据续审：RDP滚动场景背景draws=0仍p95=144ms，三场景longTasks为空，尚不能定位为背景着色器或JS长任务。7c2559055仅给原生采样器补rAF回调成本、时间戳到入口延迟和每秒实际滚动范围；原30秒/B/p95≤2B/over6B/前台及渲染档位均保持。语法与diff检查通过，没有声称新RDP结果；ui/src与tauri/src同31a28bfb4，既有产品候选身份仍准确记录。下一步按新增指标/必要合成GPU trace定位，再做经原生证据验证的修复，不降玻璃或预算。证据performance-followup-20261105/analysis.json。


2026-11-05新面板产品Mac远端回归已通过：run41904417657的frontend126835126378及backend126835127037均success，前端63协议/1291 Vitest/类型/构建，后端库1216通过/0失败/4原忽略/1已单跑进程守卫排除、其余workspace均通过。原始日志与review见ci-panel-followup-31a28bfb4。检出合并87d5137（父7a8441f7e、31a28bfb4），已核验完整tauri子树及ui/src与当前7c2559055一致，后续只有QA采样器差异；不据此把当前7c全轮CI或Windows计为通过。直接no-cache任务查询先确认终态，run列表延迟仍保留为观察限制。Windows当前仍后端回归、无MSVC候选，RDP性能及正式客户端重启/发行门保持未完。


2026-11-05原候选全轮CI终态已核对：run41904417657/backend-windows126835127187为completed/failure，MSVC构建/候选留存及首轮实际EXE冒烟success；后续windows-install-recovery在ready WebDAV旧configured断言失败，实际failed是采用后的合成无效端点启动只读结果。8fc5d179c修剩余ready消费者，按真实检测版本动作区分历史/当前，等待已保存修订对应的真实观察；不改环境ready/managed来源/可见卡片/UI异常/无写入合同。c99071793将该消费者回归纳入CI，八套71测试通过；完整tauri及ui/src与候选源31a28bfb4一致，可用原MSVC EXE配新驱动复验。失败原件及后续Node22/普通账号跳过保留，不能将构建或首轮通过扩大为全轮绿。最新Windows下载/哈希和脚本接续在WINDOWS-FOLLOWUP.md，审计ci-panel-followup-31a28bfb4/install-recovery-old-contract/review.json。


2026-11-05安装恢复消费者最新前端CI已通过：run41913003716/job126865197605，head c99071793、测试合并127543d，八套71协议、197文件1291前端测试、typecheck/build及完成步骤全部success，原始日志和job元数据保存于ci-ready-followup-c99071793。此项证明新消费者测试已纳入并运行，不替代Windows原生安装恢复/后续Node22普通账号矩阵；双后端仍在执行，原MSVC候选身份和RDP/正式重启/发行缺验维持。


2026-11-05最新驱动头c99071793的Mac后端CI已completed/success（run41913003716/job126865197293，合并127543d）。库1216通过/0失败/4原忽略/1已单跑守卫排除，其余workspace及doctests通过；前端71协议/1291测试也已通过。原始后端日志、job和review在ci-ready-followup-c99071793，源码核验产品子树与既定候选一致。Windows后端仍回归，后续安装恢复消费者新原生结果、RDP性能及正式客户端默认重启/发行门未齐，整版不关闭。
