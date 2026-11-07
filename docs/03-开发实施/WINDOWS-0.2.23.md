# Windows Git协作任务：0.2.23

2026-11-07，用户已授权整版开发/测试/发布，Windows独立Git协作，由L0统一集成。本文是L4唯一可重读任务入口；范围与验收仍归IMP-32 WIN23-00～04。

**当前已放行 R2 Windows 集成影响与原生边界补验。** R1贡献已接收并集成，前序R1独占编码派发仅作为来源保留；本轮操作与写集以本节R2为准。代码已真实推送，不以旧R1基线或软件显示版本判断是否取得新实现。

## 当前基线与获取

- 仓库：gzers/opencodex-desktop。
- 代码：远端codex/0.2.23-integration，明确基线7730073e96999382697fe2ad8dd8f877352c75a3（原生QA返修及人工门移除已集成；Mac实际native过具体范围，最终整套和平台制品仍待）。
- 文档：远端docs/governance-main，以包含本条R2指令的提交为准；读取本文件、TEAM、IMP-31/32及0.2.23-contracts/L0-paths-runtime-v1.md。
- Windows代码贡献分支：codex/0.2.23-windows；文档/脱敏证据使用独立docs/0.2.23-windows分支。若已有自己的改动，先保留并合并上述明确代码基线，不重置或覆盖工作树。
- 两种产物必须分开checkout：代码分支没有docs，不据此误判资料缺失；docs分支没有新产品实现，不据此编译。先fetch远端上述两分支，再确认git rev-parse HEAD与祖先关系。
- 此为0.2.23实施基线，尚非最终安装候选；版本清单仍待L0统一更新。是否取得新代码以完整SHA及实际diff为准，不能仅看软件显示版本。

## R21 release普通首装发现环境链未闭，继续修正（2026-11-07）

773release应用实际构建/包签名与整文件夹DMG只读挂载验证通过，DMG262723533bytes/SHA25692bd9d62a2969d76724eea99419ab71d667593a0ddcd661e1717623a8dd6774c，sealed source不变，own mount/scratch清理。此为候选，不是公开/更新签名制品通过。

Main在临时完整安装文件夹与独立HOME、OS拒绝网络/真实HOME下实际normal启动，未启用OPENCODEX_SANDBOX；v2U/D/H与指针正确，随包程序供应线程最终日志integrated runtime provisioned and verified，RuntimeStore managed路径为该I/runtime/bin/ocx，版本2.97.0-desktop023.names.1，无旧根创建。原先只等package/store落盘过早取样不够，已保留其局部pass而不当最终首装通过。

等待线程完成及3秒后实际原生首页仍“缺少Node.js/Homebrew旧指引”。定位commands::discovery默认null仍default_paths旧HOME/.nvm/.volta/brew候选，而不是本版实际I/runtime；runtime-source-changed监听只读来源卡片，未重新发现环境。该产品可用首装门不通过，L1独占commands/discovery及tests修固定受管程序解析，L3根壳刷新/旧指引按producer事实修，Main最终原生及release再验。不能改成依赖用户安装系统Node，也不扫描旧路径兜底。[r32release/实际normal首装及矛盾截图](../../.adg/evidence/0.2.23/team/L0/r32-release-firststart-managed-discovery-gap-20261107/manifest.json)。

前序47ebWindows候选42315908693回归步骤failure，正在保留凭据/证据；新773候选42319672971仍回归运行。未取原日志不猜产品失败原因、不改测试门或重新跑未修复源；全部未过项保持，未发布。

## R20 同源回归已过，新Windows完整候选仍待（2026-11-07）

明确源码7730073e96999382697fe2ad8dd8f877352c75a3完整前端214文件1576项、typecheck/build过；workspace全部binary51groups共2191过/0失败/36条件忽略，核心库1952过，strictClippy过。初始最后doctest因并行candidate构建切换共享Tauri依赖产物E0463失败，原整体exit101保留；不再并发构建后独立cargo--doc实际exit0(0tests)，只补失败范围，不重复2191项。[r31原日志与摘要](../../.adg/evidence/0.2.23/team/L0/r31-full-integrated-regression-20261107/manifest.json)。实际native UI/安装告警的证据仍归r27/r29，不把这份回归夸成发布通过。

新完整Windows候选42319672971已确认exact773，旧候选42315908693为47eb仍存活，各自真实结果按来源记录。新lib及人工门/页面消费者必须新源验证，旧candidate不能外推；两运行不因观察超时重开。当前Main串行准备exact773 release候选与后继DMG，原debugnative过不替代正式包/升级/签名。未合main/未tag/未发布，原None恢复同步语义仍待用户答复。

## R19 原生QA发现项已修正并复验（2026-11-07）

本批代码7730073e9（含da4f producer及f1236/83a4 UI）已推送。模型列表tab右状态/保存还原、搜索左/读取新增右、More保旧低频动作；新增560两步、阶段标题/进度/Key同行；同步file/WebDAV复用对象树+右凭据栏、搜索不折行、未设endpoint零目录/预览请求、唯一连接入口，人工review控件及额外compare步骤去除，global/source真实功能保低频显式入口。默认false wire照传，真实source/digest/CAS/secret/冲突/Ohash等不变，非UI偷偷填true。producerfile39/local46/strict过，UI132相关和4route40过，同源全UI214文件1576过/typebuild过。[r26后端/模型返修](../../.adg/evidence/0.2.23/team/L0/r26-native-ui-and-manual-gate-corrections-20261107/manifest.json)、[r28同步消费者](../../.adg/evidence/0.2.23/team/L0/r28-sync-native-parity-consumer-20261107/manifest.json)。

Main解锁后actualWKWebView已完成同源复验：模型空页布局、原生实际键入合成name/base/key→step2→cancel，无config写；file树/搜索/右凭据无复核项；未配WebDAV不再raw错误且唯一connectionbutton，实际开关连接窗无保存。OSdeny所有网络及真实用户HOME读写，私有进程和数据清理。该实际流程范围通过，不冒真实远端业务或最终签名制品。[r29原生截图/AX/源码事件](../../.adg/evidence/0.2.23/team/L0/r29-corrected-native-parity-20261107/manifest.json)。

安装容器拒绝已改在无frontend的原生就绪分支用应用自有NSAlert。实际显示/激活原窗口/Return确定退出1通过，未建用户数据或公共runtime，原签名应用字节保持；旧panic/parentless通知观测/未focus超时失败全部保留，r27。Mac显示检查不需Windows重做无关材质，但共享lib/人工门后继须定向复验。

新完整Rust/strict正在Main同树执行，旧47eb Windows完整候选42315908693仍实际运行；旧candidate不能证明新人工门/页面/启动后继最终验收，待终态后按diff最小补平台和最终制品。未发布，原None恢复同步语义仍待用户选择，不擅自实现跨设备删除。

## R18 Mac原生原型对照发现偏差，整版继续修正（2026-11-07）

用户屏幕现解锁，Main以OS阻断网络/真实HOME的任务候选2880实际WKWebView完成模型页、渠道两步、同步file/WebDAV和连接弹窗观察。取消新增/连接未生成config.json，私有沙箱与进程已清理。发现模型页工具栏/状态保存区和技术说明、wizard850宽/阶段标题/两步进度/Key同行未还原原型；file搜索label折行及技术checkbox，未配置WebDAV时rawsync_endpoint_changed和重复连接入口。原组件及CI通过不代表这些原生UI偏差已过，P23-04/07据实回未完成；截图/AX/证据见[r25](../../.adg/evidence/0.2.23/team/L0/r25-mac-native-prototype-audit-20261107/manifest.json)。L3按12f27未替换原型有界修ModelChannels/Wizard，后继sync页独立修，不改原型迎合实现。

后端复核确认assetsReviewed:true人工门是本版重引入，与DMD-UI-020 AC020-01及IMP28既有“连后端人工确认契约一起移除”决策冲突。不能UI静默填true；L2修真实fileTransfer/syncLocal门而保留MCP明确source/digest、路径、secret选择、原O/CAS/并发等保护。不能恢复020已移除的旧sideRef/“全局采用及渠道组顺序”标签冒本版全局refs语义。

另非sandbox原生测试安装容器提示触发AppActivity查询早于manage的panic，Main修早期纯内存注册、不创建D/H/runtime。原失败日志保留，重建后原生复验；当前候选Windows42315908693仍实际跑，但将来共享产品后继须按diff复验，不提前算最终源。

## R17 普通执行器原生预检已审计通过（2026-11-07）

真实短run42315398563精确HEAD47eb7cf7420ba9127f4d7047e4d83231f826035f终态success：Rustparent2/2和exactchild1/1、retained CreateProcess原生句柄exitCode为integer0、allBinariesAttempted=true；实际普通SID=TokenOwner/Profile/HOME/TEMP、sourceBytesUnchanged、全部DACL与桌面ACL恢复、account/Profile清理均通过。原null退出码缺口已关闭；预检只有一个小binary，不能外推49产品binary或实际安装通过。[r24原ZIP/回执/原生身份](../../.adg/evidence/0.2.23/team/L0/r24-ordinary-preflight-native-verified-20261107/manifest.json)。

47eb7修父wrapper观察对暂停态子进程的竞态：child先写实际原生身份账本再Resume，父层只核对原wrapper，不抢先用CIM扫描未装载映像；清理仍按原PID/SID/image/creation及child原账本，任何未跟踪liveSID进程拒绝删除。原跑测试过但账本未终结的run42314822371保留。派发期间另一次run42315236053实际取旧1ad源码，不计修后验证；先核远端SHA再派发42315398563，源已逐项确认。

同精确SHA完整候选42315908693已开始，继续全部49产品测试binary+独立doc+提权13/nativecred、NSIS/MSI、真实安装/卸载、两用户服务端口。旧完整候选42314821774仍存活不因本次观察超时重开或冒终止；它的源为1ad，结果只能证明其真实范围。当前不发布，等待完整产品验收；Mac实屏和原None删除语义仍待用户答复。

## R16 原生执行已实跑，退出码审计后继续完整候选（2026-11-07）

03b4d28f8通过同一retained CreateProcess原句柄读实际TokenUser、映像、创建时间和ordinary elevation；CIM缺失如实标明、若提供矛盾仍拒绝，原0..9时间粒度不变。启动前失败仅终止本次返回的精确原生句柄，避免遗留暂停进程，不按名称/端口猜进程。128项AST/managedinterop/纯fixture通过。

真实短run42314320984 job128223126807终态success，实际Rust parent2/2和exactchild1/1输出通过；TokenUser=TokenOwner/Profile/TEMP及DACL、桌面ACL、account/Profile清理、源码字节保持全过。但总控回执审计发现单项test1.exitCode=null，聚合int cast把null变0，不能据此关闭执行器门。原绿色工作流及此缺口一并保留，未按stdout冒退出权威。

1ad1bf5b3改用retained native handle的GetExitCodeProcess，等待真实signaledexit后只接受integer；null/text/partial拒绝，第一原始nonzero保持。后继短预检42314822371与完整产品候选42314821774已分别启动，精确源码1ad1bf5b3；后者仍强制全部49productbinary、doc独立执行、提权13/nativecred、实际安装及两SID端口，不用预检代替。[r23原ZIP/启动诊断/审计证据](../../.adg/evidence/0.2.23/team/L0/r23-native-preflight-exit-authority-20261107/manifest.json)。

未发布，Mac真实CI过范围保持，Mac实屏与原None恢复删除语义仍待用户答复；不因测试工具绿色擅自关闭这两项或生产配置边界。

## R15 普通执行器已到原生暂停态身份边界（2026-11-07）

后继91a97f99f修空JSON数组：pipeline在空进程账本时不生成tempfile，File.Move因此失败；同函数实际空数组复现，改InputObject并保持单项数组容器，不改产品。[r20](../../.adg/evidence/0.2.23/team/L0/r20-preflight-ledger-and-dacl-20261107/manifest.json)保留原ZIP与实际DACL原/读回证据。

1eb5c9225只接受D:→D:AI自动继承状态补记，全部ACE文本/顺序、Windows ACL二进制和保护位必须相同；报告normalized-auto-inherited-only/exact=false，不将真正访问权限或继承变更放过，116纯正反例通过。d5048038a将已验证父观测上下文贯穿暂停态启动函数，并再核当前OS SID。[r21](../../.adg/evidence/0.2.23/team/L0/r21-ordinary-launch-context-20261107/manifest.json)保留原回执。

实际短run42313538716已证明sourceOrArtifactAclRestored=true、accountAndProfileRemoved=true、sourceBytesUnchanged=true，真实SID=TokenOwner/Profile/TEMP/fresh directory owner通过。但未产生完成的Rust test回执；在暂停态进程retained kernel映像/时间及ordinary elevation通过后，CIM复合路径/拥有者/时间对照拒绝。不能猜具体哪字段或把空stdout当测试通过。下一后继直接用同一retained句柄读实际TokenUser，CIM阳性矛盾仍拒并保留诊断，实际快验后再进入全软件suite。[r22](../../.adg/evidence/0.2.23/team/L0/r22-ordinary-native-identity-boundary-20261107/manifest.json)原日志/ZIP保持。

目前所有改动限测试执行器/CI，不改产品权限或日常配置；软件49binary、安装/相邻和显式MSI切换、卸载保留及两SID端口未关闭，不发布。

## R14 快速原生预检的真实环境与清理缺口（2026-11-07）

第一次短run42310763513证实CreateProfile HRESULT0及actual KnownFolders通过，父子普通账号pwsh已启动，但未产生Rust suite回执；账户/Profile已清、源码未变，DACL读回不一致仍判cleanup-failed，不能算通过。[r18原ZIP/回执](../../.adg/evidence/0.2.23/team/L0/r18-ordinary-preflight-first-native-20261107/manifest.json)。

第二/三短run42311203133与42311736190通过有界诊断定位：真实IsWindows和ImageOS存在，RunAs子进程GITHUB_ACTIONS/RUNNER_ENVIRONMENT/RUNNER_TEMP未收到；不是KnownFolder重启（该分支本次false）、也不是环境名大小写问题。3个DACL差异均SetAcl调用完成后字符串不同，实际原/读回ACE与control尚待明细，未宽松判绿。[r19](../../.adg/evidence/0.2.23/team/L0/r19-preflight-child-environment-20261107/manifest.json)保留两原ZIP及实证布尔值。

最新eeb452913以父进程真实CI guard通过后持久的明确parent-observation交接，不伪造环境已继承；child先核实际SID/Profile、源码ID及本输入/证据位置，才接受这份显式上下文。原DACL严格恢复门不改，追加ordered ACE/control的原/读回DACL诊断；89项AST/纯fixture通过，包括差异仍拒绝/写失败独立识别。真实短run42312502433已开始，必须继续实际测试binary和ACL终态验证，不能把预检或诊断提交当软件验收。

未发布；全部49产品binary、两种实际安装/相邻或显式MSI切换/卸载保留、两真实SID服务端口仍必需。旧失败持续保留，代码和证据按批推送，不动日常配置和真实WebDAV。

## R13 普通执行器继续按真实错误修复（2026-11-07）

Mac真实CI42287979356 backend128133660321成功：严格Clippy/fmt、冷进程用例单跑及其余workspace共2189过/0失败/36条件忽略；前端已1570过。该PR合并树与采用源码树一致，不外推Windows。

Windows f3f CI回执显示49个测试二进制编译后、nativeSuiteExecuted=false，在策略JSON解析失败：npm packageLock.packages合法空键不支持PSCustomObject。主线同策略实证复现，9ea247411以共享字典解析保留完整结构，策略字节/摘要不改；111+85工具检查过。后继候选42294816371越过解析，但在CreateProfile返回非0 HRESULT时停止（stage provision-ordinary-account, line81），原错误未记录HRESULT，不伪称已知确切系统原因。78e769对齐原已实测260缓冲调用并记录HRESULT/innerHRESULT；仍待原生证实。详细实际日志/回执见[r16证据](../../.adg/evidence/0.2.23/team/L0/r16-mac-ci-and-runtime-policy-reader-20261107/manifest.json)。

为缩短执行器本身排障，新增2028c2ef2零依赖默认Rust libtest夹具，验证实际Profile/HOME/TEMP、敏感环境未继承、精确自身child及owned临时文件清理；本机2+1过及0doctest真实执行仅证明夹具，无fullSoftwarePassed。GitHub仅允许调度默认分支已注册的workflow和choice，因此快速预检统一走现有windows-candidate.yml：purpose=native-ui，admission_run=ordinary-preflight-023；该组合只运行ordinary-preflight job，旧native-ui job明确排除，非candidate安装流程。e5d7de53d已真实推送，实际短run42310763513已启动。原先新workflow调度404/新增choice422均保留为未启动，不记执行通过。

023保留安装也统一接现有入口purpose=installer-retention；admission_run为以mode=retention023开头的结构JSON，含精确candidate_run/attempt/head与artifact ID/bytes/SHA；旧019流程明确排除该标记。新单独workflow文件退役，无需提前合main才能测试。代码边界与所有产品/安装/双SID正式门保持，先完成真实短预检，再继续全部49产品binary；未发布。

## R12 Windows执行器路径误拒已修，新候选继续（2026-11-07）

原候选42286888610已终态failure：未进入普通SID整套Rust测试，也未创建该套件账号/生成安装器。原job128129908397实际在Assert-TwoUserLocalPath拒绝标准盘符反斜杠路径，字符类只转义斜杠，导致合法D:\a\repo也拒绝；不是这轮产品回归断言失败。提权private-create13项及三nativecred组实际通过。原ZIP714778072bytes，SHA256 88260d4cce4ce4dbbbaf3eb8f82b5d5c58d2efa269657edcdc7c534dace12ba2已原样下载核对，日志/有界文字回执见[r14](../../.adg/evidence/0.2.23/team/L0/r14-windows-native-path-admission-20261107/manifest.json)。连接器512MiB上限拒取时改用gh原字节下载，未截断或解压运行其中程序。

f3f54b7bb只修测试路径准入：纯字面校验先对分隔符等价核对，再由OS解析，绝对盘符/正反斜杠/混用/Unicode有效；relative/UNC/device/根/父遍历/ADS/control继续拒绝，不改产品权限、owner或日常目录。108项双用户检查及85项普通执行器检查过；新候选42291427105基于精确修复继续真实MSVC/普通SID全部binary、提权专项、安装及两用户端口，不复跑原失败配方。真实套件终态仍待，不能把脚本检查冒整版通过。

当前CI42287979356前端已success，Windows backend因同执行器误拒failure，Mac backend回归仍运行。后继新源码CI继续；未合main/未tag/未发布，日常配置/真实同步边界保持。

## R11 前端真实CI通过与候选资产名校验（2026-11-07）

真实CI42287979356的frontend job128133659058已success，214文件1570项、typecheck/build全过，包含未放宽阈值的像素检查。checkout为PR72合并提交0eb1bd2348e52ed0a255ed0c1a79bd0462218ef0，实际树73c196cee552f7f259f968b3ae1c1f5812b3b0ee与b23696b80精确一致；不是以本机结果替代CI。

代码a3d0be94e仅修相邻安装验收的候选原始文件名：固定bundler2.9.4在MSI/NSIS都直接使用product_name()，候选upload-artifact保留OpenCodeX Desktop空格；已发布022资产的OpenCodeX.Desktop点号名仍保持原PIN。不能把公开资产改名规则套到原候选。修前真实空格名fixture拒绝，修后16项通过，点号误改名/摘要/收据/成员不符仍拒；不改产品或旧格式策略。[r12证据](../../.adg/evidence/0.2.23/team/L0/r12-frontend-ci-and-installer-admission-20261107/manifest.json)保留原失败和同源证据。

Windows候选42286888610普通用户回归步骤已failure，提权创建专项success，原生凭据专项继续。日志尚未齐，不猜根因/不重启同源候选、不假称已取得安装器；待完整原日志和清理回执后做最小后继。两端后端CI及正式资产/相邻安装/签名仍未关闭，未发布。

## R10 像素采样错误已复现与修正（2026-11-07）

源码b23696b80655421a82e95d506c755723f6b0efcd已推送，本批仅ui-dialog-glass-v2测试文件。CI原artifact经实际ZIP/hash核验，Chromium窗口被runner限制为1182×684，但旧算法减requested900得到physicalTop=-216，把y138条纹误当probe；正确页面原点87映射probe到y441，原低效果该行range0、高效果阶跃0.05263。受控同684窗口原源码14失败/14通过，保留原失败，不重标CI绿。

后继abe49106e（集成b23696b80）以自有fixture三色标记校准真实截图原点/比例，核对probe位于可见区域及不含未捕获透明像素。全部28材质/对比度/控制断言及阈值循环块逐字不变，无skip、无产品样式修改；正常/684各28通过，小窗375/可见288的不可见probe负例明确拒绝，不能回退采背景冒绿。新CI42287979356在精确新SHA上运行；真实远端终态仍待。原ZIP、原因、校准几何与后继日志见[r11证据](../../.adg/evidence/0.2.23/team/L0/r11-pixel-registration-20261107/manifest.json)。

当前Windows候选42286888610及其普通SID suite仍实际运行，source41ab81694，未取消、未因观察超时重开。它的产品代码与本次后继完全相同，后继只有测试采样修正；实际制品/签名/相邻安装仍按原门核，不据此提前发布。

## R9 实际安装默认与普通SID套件已交新候选（2026-11-07）

明确代码41ab81694c1cce4e05bd78309b4ff2b8a5eb82da已真实推送，工作区干净。1673190e1采用CLI2.11.4/bundler2.9.4固定上游模板，保留MIT来源，仅改用户级Programs默认、明确自定义目录及卸载保留。15Python/10PS组、WiX3.14.1三分支严格Schema与完整NSIS3.13交叉编译通过；固定Windows3.11/WiXICE/真实UI和安装路径仍待候选，不用Mac静态结果冒原生通过。发布配方已固定同CLI2.11.4，避免新模板换打包器漂移。

旧022 MSI为per-machine，新023为per-user，Windows Installer不支持跨上下文自动major upgrade。新版先实际拒绝并指引卸载旧管理器保留配置，再安装；相邻MSI预期拒绝、MSI显式程序切换、两种fresh及NSIS相邻更新分五个独立runner场景。测试必须实际执行候选拒绝及旧程序/全数据哈希保持，不以harness自行报阻断代替。

ee1504b94普通SID执行器已冻结85项纯检查通过：父账户只编译，全部Cargo JSON精确测试binary交新真实普通SID/Profile/TEMP执行，默认libtest workers保持，不skip失败、不修foreignowner。docTests独立真实运行；提权私有创建13项及nativecred保留always。候选42286888610基于精确41ab81694已启动，需等待原生suite/安装器/两SID端口全部实际终态。

原候选3/4的1299过/368失败与CI2880像素断言失败继续保留。浏览器工具安装及workflow上下文修正已过actionlint，本机28像素过不能替代CI；L3正在对原CI截图/几何排障，不改产品材质或放宽阈值。Mac2880整文件夹DMG签名/挂载/源不变及隔离原生启动过，屏锁下实屏仍待。[r10证据](../../.adg/evidence/0.2.23/team/L0/r10-installers-and-ordinary-context-20261107/manifest.json)含精确摘要及清理事实。未发布，真实同步与日常配置边界保持。

## R8 原型补齐与真实CI失败后继（2026-11-07）

代码678fd312afbcfb01998e529244b0618baacd4af4已推送。模型条件预览已接真实同源Rust匹配视图/族/前置/全部原子三态，L3实际ModelsRoute与双引擎对照及原型默认折叠诊断通过；完整UI214文件1570项、类型/构建通过。冻结后端751d全workspace52目标2189项通过、0失败、36条件忽略；后继Mac启动消费8项、临时目录IPC17项与strictClippy通过。原失败、真实输入/输出和后继范围见[r9](../../.adg/evidence/0.2.23/team/L0/r9-prototype-and-ci-followup-20261107/manifest.json)。

第三候选42277944499基于cc92的实际Windows回归失败1299过/368失败/5忽略，未生成安装器。逐类核出106项明确owner不符、196项测试硬编码/tmp在Windows盘符路径不存在、4项NotConfigured及62项事务/断言类仍需实际后继确认。普通完整回归将放到新真实普通SID/Profile/TEMP执行，提权创建13项仍独立验证；不接受Admins作为用户、不repair已存在外owner、不删除或skip失败用例。678fd312a已将11处测试临时parent接Windows实际temp_dir，Unix仍保持短/tmp防IPC路径超限，未修改产品目录规则。

CI4f前端失败为没有声明安装真实Chromium/WebKit，非页面断言通过。10b5e1905已锁playwright-core1.64.0并在CI安装对应浏览器，28项原像素断言本机实跑通过；后继CI仍待实跑，原失败保留。Windows安装默认模板与MSI跨安装上下文处理继续按R7，不能因共享逻辑已绿而跳过本版触发的平台边界。

## R7 最后清单对照发现的安装默认路径缺口（2026-11-07）

本批代码7cbb0daf7已推送，含同源条件解释后端9c6da8e2c、原None恢复后真实方向测试及新的精确候选相邻安装验收工具5af6b66a6/工作流2d59b3615。后端177项与严格Clippy通过，最终UI展示仍由L3接线；不在Windows复制模型共享逻辑。

重要未完：现Tauri默认NSIS仍为LocalAppData直下、MSI仍ProgramFiles，尚未落地既定默认当前用户LocalAppData/Programs/OpenCodeX Desktop。旧工具已如实核出此差异，L0正补安装模板/配置与版本区分的验收，不能改合同、用强制/D或INSTALLDIR替代产品默认，也不能拿已手工stage到正确位置的两用户程序当安装器默认通过。候选3/4即使某部分通过，也不据此关闭本项。MSI跨上下文升级约束需要实际处理与证据，不添加旧配置自动迁移。

相邻安装新入口为windows023-retention.yml，只接受已成功的精确candidate run/attempt/head与artifact ID/大小/SHA；两个安装器各自独立新runner，原始0.2.22资产已有标签、ID、大小与实际摘要核对。真实执行尚未启动，现纯工具检查10+8项不等于Windows安装通过。

原None范围恢复的2项隔离验证同时发现：完整产品preview仍建议下载，不是已自动阻断；未确认后继下载、未写业务文件或真实远端。策略仍待用户选择。详见[r8证据](../../.adg/evidence/0.2.23/team/L0/r8-trace-and-absent-restore-20261107/manifest.json)。

## R6 真实双用户代理端口与候选后继（2026-11-07）

代码8bad37615增加显式VerifyRuntimePorts，仅在临时Actions runner中实际启动两个真实SID各自的随包服务，核对A占用端口时B拒绝且A原进程代次保持、B换独立端口后的双方健康重叠、各自精确停止及清理。92项静态/纯夹具通过只证明脚本边界，非Windows原生通过。接线48b3070b2要求实际fullVersionGatePassed及verified-runtime-ports；后继4f087eaea仅延长MSVC全流程构建job外层时段，未改产品启动/停止/性能预算。

第四候选42280143310基于精确48b3070b2，正在执行，尚未取得原生结果；第三候选42277944499继续保留，不能算最终源验证。相邻0.2.22安装升级与默认卸载保留数据另有受控工具补齐中，不能复用旧019→431脚本冒本版验收。后继实际资产仍须给可取用位置、源SHA和摘要。

Mac release候选07ed8b21b的完整包签名与隔离原生启动通过、实际新D/H/固定指针读回；锁屏下未做实屏验收。测试进程及沙箱数据已清理，候选程序保留待解锁复核。原始回执见[r7证据摘要](../../.adg/evidence/0.2.23/team/L0/r7-native-candidates-20261107/manifest.json)。本批结果不外推为整版发布门已过。

## R5 结果归属消费者与安装配方后继（2026-11-07）

当前代码07ed8b21b已推送集成分支，包含后端cc92eebf2只读结果归属与前端3d6284082四类原操作结果路由。前端214文件1563项通过、类型检查/构建通过，Chromium/WebKit各6条流程通过；此页面夹具证据不冒原生存储/权限证明。后端实际owner九项与workspace严格Clippy通过；核心库1915项通过、13忽略，集成扫测保留旧logs夹具失败，改为新D/H分区后app_startup_wiring和logs共8项通过。详细日志见[r6原字节及摘要](../../.adg/evidence/0.2.23/team/L0/r6-result-owner-integration-20261107/manifest.json)。

第二安装候选42276834374停在npm resolver version mismatch，未生成安装器；后继26b0e5dcb固定构建Node26.8.1/npm11.19。第三候选42277944499基于cc92eebf2，已通过完整离线运行包stage，正在Windows回归；必须等待实际终态与安装/双SID证明，不将本条当作安装通过。此前各失败保留，不重复运行原换行失败配方。

两真实用户的实际runtime端口验证与相邻0.2.22升级/卸载保留工具由L0继续补齐，最终候选须带明确源SHA和资产摘要再回交。当前无需重复已通过的共享模型或全站材质；Windows继续按R2独占范围及真实平台缺口执行，禁止真实业务同步/日常配置改写/自行发布。

## R4 当前集成候选与真实MSVC结果（2026-11-07）

代码基线已真实推送511463f75b10f5db9736adfa9be1ebefa7d98357，包含范围备份/原操作通知、实际用户创建与13项原生检查、平台Clippy后继修正、完整随包资源及候选安装配方。草稿PR #72统一接CI，不在旧PR #71反复核旧源码。

实际Windows/MSVC CI run42270824847：private-create 13/13通过，已记录提权TokenOwner=Administrators而刚创建owner=TokenUser；provider_auth_native、channel_rename_native、native_secure_store三组均通过。整体job仍失败于当时平台Clippy，不代表安装/两用户通过。现后继0635f8dfb已按平台收束imports、UTF16迭代及仅Unix使用的故障夹具；后继真实CI继续核，不以MacClippy替代Windows执行。

完整Windows候选首次run42275126707停在Reviewed patch changed，尚未生成安装包：是Windows checkout换行影响受摘要约束的补丁，修复511463f75固定patch LF、git apply autocrlf与跨平台POSIX相对路径。修后重新用固定原tgz实际双重构建，产物仍15406751 bytes、SHA256 afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b，无同版本字节漂移。第二候选run42276834374基于精确511463f75已启动，状态以Actions实际终态为准。

候选脚本仅在临时Actions runner创建两个临时真实普通账户，验证重叠GUI、Profile/配置/pipe/DPAPI隔离及同SID重复打开，完成后清理账号/进程/ACL；静态57项通过不当原生通过。完整代理端口隔离仍需实际runtime，不用synthetic DPAPI子范围冒整版完成。构建配方会随候选stage固定完整资源，不要求Windows读取Mac临时路径。保持本入口既有独占写集、无真实WebDAV/日常配置/发布的边界，原失败全部保留。

本批可拉取证据见[manifest](../../.adg/evidence/0.2.23/team/L0/r5-windows-candidate-20261107/manifest.json)。最终软件候选尚未发布；若贡献需要公共文件改动继续回交L0，勿恢复旧迁移或旧格式写入口。

## R3 追加：真实创建拥有者与最新集成（2026-11-07）

本节接续R2，不重复已完成贡献。Windows已推code400c4e7f4/证据baeb7e0bc，L0已接收原GNU108项、R2实际MSVC provider_auth_native失败及CI终态；原typed ACL修e011ae2ee已集成为87b10dd7e。原失败为新临时文件owner与TokenUser不符，提权TokenOwner是否Admins仍必须由实际日志证实，不能把解释写成已证根因。

代码现在提供d07079683显式TokenUser owner/User+SYSTEM私有DACL创建及abc374831完整私有发布者接线；原子写、锁、固定用户分区、事务镜像、凭据元数据和离线运行包解压都沿同一创建边界，不接管既有其它owner。已存在拥有者仍核对、外owner/重解析/hardlink拒绝，未通过接受Administrators/SYSTEM或关闭权限校验过门。原始Windows回交与后继编译/回归证据见[本批manifest](../../.adg/evidence/0.2.23/team/L0/windows-r3-handoff-20261107/manifest.json)。

Windows可继续：fetch并合入上述明确代码基线，按现R1专属写集补原生事实。先运行infrastructure::windows_private_create::tests（12项，包含TokenUser/TokenOwner/elevation/刚创建owner/DACL读回、已有外owner保持、重解析/hardlink/Unicode/非截断重开），再跑provider_auth_native及原凭据/rename/native_secure_store失败组。CI已加always实际创建测试，草稿PR #72用于当前集成树真实构建与回归；无需在旧PR #71反复验旧源码。L0下一批7bf84fe31已修Windows归属模块路径及新版Clippy诊断，不扩大用户数据权限。二账户/SID与正式安装器原有验收仍未关闭；缺受控第二账户环境如实回交，不用一个SID多会话冒充。

本机已通过atomic2、locks3、instance6、data_root42、modelTxn63、bundle25、actualLocalApply与完整workspace严格Clippy；这些不代替Windows实跑。完整离线供应stage已在仓库，后继正式安装候选仍按真实签名/摘要交付，不以Mac临时文件路径替代。后续最小公共材质复验仍按R2，不重复全站逻辑。保留所有旧失败和清理限制，不自行发布/真实同步/修改日常配置。

## R2 当前派发（2026-11-07）

已接收并集成Windows代码codex/0.2.23-windows@42d2a4a1770a5be1050dec3de641efab77a65917，证据docs/0.2.23-windows@f7c54ad2b7027b209ff99a4374a4d4f6cf9179d0；代码原提交911310f1、0c2d1a1d、42d2a4a1对应集成c8e7ac62e、0f3d670d5、7d1ace177。R1证据按原字节归档于[回交目录](../../.adg/evidence/0.2.23/team/L4/r1-20261106-01a1b0c8/)，集成文档提交1a45c44c92c00af9adac00ac66c0bdf821c6eda6。R1的97项GNU检查通过只证明其实际覆盖范围；同一SID的RDP会话不作为两用户证明，GNU复制EXE不作为正式安装包。

本次集成基线已包含04511769c的平台hook（实际Profile/ACL及连接前服务SID验证）与f0f76b7a4的Windows条件编译修正。先在独立代码树fetch并合入明确基线，确认该SHA是当前HEAD祖先；保留贡献分支已有改动，禁止reset覆盖。

1. **现在可执行**：使用当前集成源码，在Windows原生MSVC工具链编译/运行受影响检查，补验两真实SID并行、同SID跨会话/提权单实例、跨SID凭据拒绝、pipe及CLI身份验证、整机端口冲突与进程归属不误连误停。报告真实账号/SID关系，证据脱敏；缺少第二用户权限时明确未验，不以模拟通过代替。
2. **现在可执行**：检查本基线中已集成的R1实现与L0 hook是否正确组合。必要修复仅限R1已列Windows专属文件与test/smoke/windows023-*；platform.rs、lib.rs、runtime核心、Cargo/版本/发行工作流仍归L0。需要公共修正时提供精确接口/最小补丁需求，不并发改共享文件。
3. **待后继制品**：MSVC正式NSIS/MSI的安装、从0.2.22更新、卸载保留数据以及完整离线runtime供应，以L0后继交付的安装包/资源描述符摘要为准。本次源码交接不是最终安装候选；不能将隔离夹具或本机缺失的Mac临时路径记作制品交付。仓库已提供.github/runtime-platforms-023.json、.github/scripts/build_desktop_runtime_bundle.py与build_runtime_023.py，可读配方核对，stage/发行接线已在205d72b1a实现，可使用.github/scripts/stage_runtime_bundle_023.py按固定配方生成完整资源；最终版本/签名正式制品仍由L0统一收口。
4. **本批新增条件补验**：公共玻璃弹窗修复已在2bbe1aa92集成；保留模糊并增强正文/次级文字对比，低效果和无backdrop支持仍为实底。Mac Chromium/WebKit真实窗口像素及原回归64/64通过，最低正文5.56:1、次级4.88:1；请只对受影响WebView2弹窗核对亮暗/高低效果、长内容滚动与嵌套焦点，确认不是单纯透明或意外实底，不重做全站页面/模版逻辑。受既定RDP硬件例外约束时保留原失败事实，不伪造绝对帧率通过。

运行包名称版本已确定2.97.0-desktop023.names.1，包SHA256为afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b。本条用于后继制品对照，不表示tgz或完整bundle已上传可下载；L0必须提供真实可取位置后再做制品验收。

平台检查保持隔离，不改日常配置、不做真实WebDAV或上游推理、不接管其它用户端口/服务、不自行发布。回交到代码codex/0.2.23-windows及证据docs/0.2.23-windows，明确依赖基线、完整SHA、命令/结果、未完项与清理回执。每批可交付成果完成即提交推送，并更新本入口；只在本机commit或聊天报“完成”不算交付。

## 共享基础已经具备

U=当前用户HOME/.codex-desktop，D=U/desktop，H=U/opencodex/home；固定指针U/desktop/manager-location.json，v2元数据不重复活动根。启动、切换、IPC、卸载已转新版规则，旧迁移入口明确拒绝。运行程序位于实际安装根I/runtime，与D/H分离；managed-only解析器要求固定Node及激活证明，不扫描旧安装、旧HKCU、PATH或独立ocx。

固定依赖：Windows I/runtime/node/node.exe、I/runtime/node/node_modules/npm/bin/npm-cli.js；入口I/runtime/bin/ocx.cmd，包前缀I/runtime/opencodex。主线正在准备完整离线供应，不要求你读取日常旧程序来冒充集成。平台R1可使用明确构造的隔离夹具；实际完整安装/更新制品项待L0交后继制品SHA补验，不能记为已通过。

本机共享检查：data_root 55、IPC 48、workspace 8通过；安装器38项普通检查及固定I布局真实npm6项通过。首轮全Rust回归1333通过、10失败、10忽略：其中4项正是windows_startup_root旧规则测试，另5项目录/IPC消费者已由本机修正，1项200ms进程夹具在隔离重跑通过；保留首轮失败，不把这些数字当整版验收。

## R1 历史写集与来源（不作为R2重复开发指令）

以下为R1已派发与接收的写集；R2的可改范围见前述新派发节：

1. apps/desktop/tauri/src/infrastructure/windows_startup_root.rs：退役注册表/安装旁data/旧绑定链恢复逻辑及旧测试，接当前OS用户Profile/SID；保留lib仍使用的defer_main契约。旧四项默认路径测试改为真正验证新版规则及无旧扫描/无副作用，不通过ignore遮盖。
2. apps/desktop/tauri/src/infrastructure/windows_user_scope.rs（按需新增，由windows_startup_root.rs内部path模块引入）：实际Token SID、Profile/LocalAppData、目录拥有者及当前用户ACL辅助。固定待接接口：windows_startup_root::current_user_profile()->std::io::Result<PathBuf>；protect_current_user_path(path:&Path,mode:u32)->std::io::Result<()>。保护新用户数据及私有文件，拒绝接管其它用户根；不修改日常配置。L0据此接infrastructure/platform.rs，不直接由L4改该共享文件。
3. apps/desktop/tauri/src/modules/ipc/endpoint.rs：本批所有权交L4，但只改Windows路径/账户身份帮助及相关测试，Unix行为保持。当前Windows pipe_name仍从USERNAME派生，需要改成真实SID；显式验证客户端SID/管道访问归属，拒绝不同用户误连，保留沙箱与实际协议。模块注册/公共业务服务不归本批。
4. apps/desktop/tauri/windows/installer-hooks.nsh：按新版默认当前用户安装根/数据根修正受影响安装、更新和卸载，不扫描、迁移或删旧数据，不删其它用户/其它拥有者目录。完整runtime-bundle生产部署由L0统一接；需要具体接口先给最小补丁需求。
5. 按实际需要可改infrastructure/windows_restart.rs及同目录已存在对应ps1、windows_query_job.rs、windows_query_fixture.rs；仅Windows进程归属、入口/当前用户实例及本批隔离检查。无必要则不修改。
6. 新增独立平台检查可放test/smoke/windows023-user-scope.ps1及其同名前缀测试，或上述文件内cfg(windows/test)单测。回交实际逐文件清单。

共享lib.rs、data_root、runtime核心、commands、infrastructure/platform.rs、Cargo及锁、Tauri配置、版本、发行工作流、Vue/TS不归L4。需要Windows依赖feature或共享hook时，回交精确函数/feature和最小补丁，由L0接入；不要自行扩大写集。

## 验证和回交

证明两真实SID可并行、同SID多会话/提权仍单实例；固定Profile/根、ACL/DPAPI、pipe/客户端身份、端口占用与进程归属不误连误停。端口属于整机，冲突须明确报出，不能抢占其它用户服务。已有玻璃效果无需本批重测。

真实SID/安装器环境不足时如实分开报告，不用同用户两个RDP会话冒充两用户。普通UI/共享算法不重复做平台专项。隔离数据、保留失败/成功及清理证据；不运行真实业务同步/推理，不改日常配置或自动清理旧数据，不自行发布。

回交必须推到可取得的代码/文档贡献分支，给完整SHA、依赖基线、合同、环境/命令/结果、未完原生/制品项及清理回执。不能只给Windows本地临时路径。L0核验后统一集成；R2缺口及R4实际运行包/安装候选仍按前述新派发节继续。

后继交接补记（2026-11-07）：2bbe1aa92代码批次已推送，包含离线资源stage配方205d72b1a、设置37项新版路径回归cdf3585e3及公共材质修复。原33d2为R2首派来源，已由本段明确基线替代；新增同步Service/LocalFile与恢复后继仍由L0继续集成，不能据此提前发行。


## R22 主对话接管及Windows回归修复（2026-11-07）

采用用户最新单主对话要求，贡献线冻结，不再派发。代码54c5198a5分四笔：03f4cae6a统一模型接收/事务实际User+SYSTEM严格ACL，cfdd8e101引入U外部持久删除gate并在独占期释放D内句柄，3f208c759修正npm重复解包及准入锁测试夹具，54c5198a5改用原生retained handle等待退出再取码。权限验证未接受Admins/宽ACL，noFollow/单链接/实际TokenUser保留；禁止覆盖规则与原断言保留。Mac库1963过、14原忽略、strict过，PS130仅静态；不得当Windows通过。新候选及NSIS/MSI/双SID/保留五场景仍待验。

773候选42319672971终态失败：普通执行器第334行退出码读取异常，allBinariesAttempted=false；提权私有创建与原生凭据组成功，安装器后续未运行。47eb原证据整ZIP已完整SHA核验；773只按条目CRC/size抽取，不声称完整ZIP核验。[r33证据](../../.adg/evidence/0.2.23/team/L0/r33-main-only-repairs-20261107/manifest.json)保留原失败和跟进。


## R23 打包界面协议显式化（2026-11-07）

Mac release实际实屏白窗，虽后端供应成功但不通过UI门；编译fingerprint只有default，未含custom-protocol。b24b00be2让Windows候选taskBuildArgs、MacCI与正式release显式启用custom-protocol，重建实际rustc已观察该feature，46打包/发布工具检查过；实屏修后仍待。正在执行的54c5198a候选42325691178不含本修复及后继Windows接收GENERIC_READ，不作最终同源制品；保留其完整回归证据后再定后继候选。失败报告的后端pass不冒认界面pass，见r35。


## R24 原生白窗对照更正与当前候选（2026-11-07）

R23的缺custom-protocol因果判断撤回：同一已封装release包，未激活截图白窗，显式激活精确owned PID后正常首页；原default构建与显式custom-protocol构建均重复此现象。此次为直接启动执行器未激活产生的呈现问题，不认定产品资源加载失败。b24b00be2保留显式参数作构建约束。维护包普通首装、实际环境就绪首页与签名/DMG/源不变通过（r36），没有启动真实服务/同步。新Windows候选42327388977固定b24b00be2包含后继句柄和构建约束，完整回归/安装/双SID待；54候选保留为回归前序证据。


## R25 安装清理夹具改为当前V2（2026-11-07）

后继0c19e55d6只修test/smoke/windows-uninstall-data.ps1：原脚本仍构造V1注册表指针/安装旁data，和本版无旧扫描合同冲突。改用实际TokenUser注册Profile的全新U/desktop、V2固定指针和所选D，取消不写、确认删除、重复执行、外部HOME/旧路径保留、持久外部gate均保留断言。仅为预证不存在的本次合成夹具设置实际User/SYSTEM owner，既有对象不repair。31解析及真实Mac越界guard拒绝过，不冒Windows native过；需新来源候选完成安装清理。原候选只作为各自来源范围证据。见r37。


## R26 提权备份锁创建归属补全（2026-11-07）

7f5057366修正Windows备份/删除准入锁的创建：原safety标准API在提权进程可用Administrators默认owner，随后删除整树的实际用户归属复核拒绝；现走统一private create/open，验证实际TokenUser与User/SYSTEM，不修existing未知对象。非Windows保留原nofollow打开逻辑。两IPC锁等待夹具改为合法private创建，47IPC/42backup/12uninstall及strict本机过；always提权组保留原13项并新增真实backup-lock owner/DACL/重开字节检查。仅定向本机通过，不冒Windows实证；候选结果及新来源复验仍待，见r38。


## R27 完整候选失败明细与单主线后继（2026-11-07）

54候选42325691178：库1676过/16失败/5忽略，48其它binary记录成功；末库已执行完成，但终止时枚举先消失导致控制器line100异常与cleanup-failed，不假称全部完成/账户清理过。b24候选42327388977：49binary全部尝试，48成功，库1675过/17失败/5忽略，真实101保留；源码/制品ACL、desktopACL及账户Profile清理均过。两候选提权13过，原生改名凭据测试被“目录属于其它用户”挡住，安装器未进入通过。r40按条目CRC/size抽取，未声明完整715MB ZIP/SHA过。

后继67b08b2d5把非布局临时根gate留在根内，生产U/desktop删除/owner规则不变；7df4ec422把四个恢复marker/异根回执测试改为合法private创建，不松校验；b75452325以retained handle处理退出信号与枚举顺序，并保持仍活未知进程拒绝。同步9项Timeout及两项等待Elapsed尚未查清，dcb465d36只在Windows测试记录静态阶段名/耗时，预算/workers/全范围保持。

用户已批准恢复原None对象取消本机同步勾选。c2845bd7d接入原恢复CAS/保护/持久进度及本地UI回读，17真实恢复、1589UI、type/build/strict通过，原失败及修复证据r41。完整后继候选42333262869固定218b2262b，含上述存储/执行器、V2安装清理和提权备份锁、范围处理与诊断，当前仅运行，不标通过。无需再开贡献对话，Main继续取证修复，未发布。


## R28 云端独有MCP的恢复后重选补齐（2026-11-07）

f86620e83补真实消费者缺口：原None恢复取消选择后，cloud-only MCP目录无本机source选项，UI无法明确勾选下载。现仅该情形加入云端有效文件内exact定义摘要+managed目标入口；原本机冲突来源保持，标记availableLocal=false。四种原None对象（渠道/扩展/Skill/MCP）真实Service取消选择及明确重选下载4过，Service28、UI48/type/build/strict过。56a8a8dcc再阻止范围读回失败时沿用旧选择prepare/confirm/save。整套UI首轮1589过/1采样probe消失，独立原玻璃测试不改样式/阈值28过，精确新源全量续跑，r44。当前218Windows候选42333262869不含此普通共享增量，只证明其自身来源范围，最终制品需新源。
