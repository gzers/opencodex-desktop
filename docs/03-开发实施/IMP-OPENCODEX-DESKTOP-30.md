---
id: "IMP-OPENCODEX-DESKTOP-30"
object_kind: "implementation.change"
title: "0.2.21发布后双线修复与集成发布计划"
state: "in_progress"
completion_summary: null
dmd_refs: ["DMD-OPENCODEX-POST021"]
demand_ids: ["DMD-OPENCODEX-POST021"]
task_ids: ["TASK-OPENCODEX-POST021-LOCAL-20261104", "TASK-OPENCODEX-POST021-CHANNELS-20261104", "TASK-OPENCODEX-POST021-CHANNEL-PROTOTYPE-20261104", "TASK-OPENCODEX-POST021-INTEGRATION-20261105", "TASK-OPENCODEX-POST021-WINDOWS-HUB-20261104", "TASK-OPENCODEX-POST021-WINDOWS-MATERIALS-20261104", "TASK-OPENCODEX-POST021-WINDOWS-PANEL-20261104", "TASK-OPENCODEX-POST021-WINDOWS-RESTART-20261104", "TASK-OPENCODEX-POST021-WINDOWS-RESTART-FEEDBACK-20261104", "TASK-OPENCODEX-POST021-WINDOWS-STICKY-20261104"]
summary: "L/L2与W/W2已集成；Mac原生复验发现面板空URL崩溃并修复，Windows性能及正式客户端adapter等发布门未通过。"
---
# 0.2.21发布后双线修复与集成发布计划

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
