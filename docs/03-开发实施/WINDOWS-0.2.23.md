# Windows Git协作任务：0.2.23

2026-11-07，用户已授权整版开发/测试/发布，Windows独立Git协作，由L0统一集成。本文是L4唯一可重读任务入口；范围与验收仍归IMP-32 WIN23-00～04。

**当前已放行 R2 Windows 集成影响与原生边界补验。** R1贡献已接收并集成，前序R1独占编码派发仅作为来源保留；本轮操作与写集以本节R2为准。代码已真实推送，不以旧R1基线或软件显示版本判断是否取得新实现。

## 当前基线与获取

- 仓库：gzers/opencodex-desktop。
- 代码：远端codex/0.2.23-integration，明确基线f3f54b7bb36e644cea0b832f5b09c1276b79c8ee（测试执行器Windows反斜杠准入修正；新候选42291427105实际验证，原候选终态及原因见R12）。
- 文档：远端docs/governance-main，以包含本条R2指令的提交为准；读取本文件、TEAM、IMP-31/32及0.2.23-contracts/L0-paths-runtime-v1.md。
- Windows代码贡献分支：codex/0.2.23-windows；文档/脱敏证据使用独立docs/0.2.23-windows分支。若已有自己的改动，先保留并合并上述明确代码基线，不重置或覆盖工作树。
- 两种产物必须分开checkout：代码分支没有docs，不据此误判资料缺失；docs分支没有新产品实现，不据此编译。先fetch远端上述两分支，再确认git rev-parse HEAD与祖先关系。
- 此为0.2.23实施基线，尚非最终安装候选；版本清单仍待L0统一更新。是否取得新代码以完整SHA及实际diff为准，不能仅看软件显示版本。

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
