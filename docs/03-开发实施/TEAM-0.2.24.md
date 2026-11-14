# 0.2.24 Mac/Windows双线协作与Git交接

revision: 13；2026-11-14。实施顺序/状态唯一归[IMP-34](IMP-OPENCODEX-DESKTOP-34.md)，采用范围见其源链接。此文件只定义角色、排他写集、交接与提示词，不继承0.2.23 TEAM的旧派发状态。W0及W1三批组件已通过Git回交；管理员MSI补验与未完清理已接收，普通用户验收保持；当前W-L2原语已回交，SD/环境前提r3已明确，Windows按文末W-L2/r2写集及r3依赖连续实现，Mac不并行改该写集。实际状态归IMP-34。最终W2仍须固定候选后由用户转交提示词。本对话不额外开Mac贡献对话。

## 角色和排他写集

| 负责人 | 写集 | 其它边界 |
| --- | --- | --- |
| 本对话/Mac集成 | 跨平台路径/安装/状态/事务/保存/导入/网络、全部Vue/TS产品页、macOS打包/CLI、共同模块和命令注册、版本清单、.github、Cargo/npm根清单和锁、main集成、正式docs/计划索引。 | 不同时改Windows独占文件；从r5提取测试素材仅取精确源，不merge整条旧分支。 |
| Windows W0立即可做 | 新增apps/desktop/tauri/src/infrastructure/windows_official_runtime024/**；新增test/windows024-native/**（含独立原生测试宿主/Cargo清单锁/定向脚本）。 | 平台模块由独立宿主编译引用，真实Win32，不改产品父mod.rs注册；初期未接产品必须如实标记。无假host或全局进程扫描取代归属。 |
| Windows W1已部分放行，按下文r1接口 | 新增apps/desktop/tauri/src/infrastructure/windows_cli024.rs及test/windows024-native/**；按本机固定的程序根/稳定入口/清单修改apps/desktop/tauri/windows/installer-hooks.nsh、nsis023.nsi、wix023.wxs。 | 保留既有文件名不表示目标仍0.2.23；版本号由本机统一。默认/自定义根等共享语义先有真实接口提交，再改安装器/PATH。 |
| Windows W1-C，临时单写至Git回交 | 上述原生/安装器写集，以及明确共享CLI文件src/modules/runtime/terminal_cli.rs、terminal_cli_windows.rs、terminal_cli_path.rs、src/bin/ocxd.rs、src/commands/terminal_cli.rs；可新增terminal_cli_windows_native_tests.rs。 | 仅本批Windows编译/控制台转交/中断修复，完整边界见下文合同；Mac本批停止，不并行改这些文件；不改Mac语义、其它共享状态机或Vue。 |
| Windows W1-D，临时单写至下一次Git回交 | 原W1-C CLI/原生/安装器/test写集、两份terminal_cli_windows_*tests.rs、新bin/ocx.rs；program_layout.rs仅Windows cli_entry与纯路径测试；Cargo.toml仅native ocx bin装配。 | 具体合同windows-w1d-native-entry-r1，安装器拥有I/ocx.exe、登记schema3，不做旧开发记录兼容；program_selection已由Mac修，不放宽准入，不改Vue/其它状态机/工作流。 |
| Windows MSI唯一补验，按文末 | 可新增/改terminal_cli_windows_native_tests.rs的独立MSI-only用例、test/windows024-native/**；三个安装器文件仅实际日志证明缺陷时修。 | 已过组件不重跑；先确认允许per-user MSI的隔离环境，不改当前机器策略，不提升测试用户、不改安装范围；共享产品若有新缺陷先回证据。 |
| Windows W-L1生产生命周期，已回交；后继W-L2当前生效 | 原native/生产生命周期与tests；新增原件/owner-host/supervisor/diagnostic原生模块、ocxd内部模式及Windows消费者注册，逐文件按windows-lifecycle-shared-r2允许表。 | 四边界已冻结；Windows范围内连续开发验证推送，Mac暂停该写集。保持原预算/未知拒绝，不改Mac/原官方包/旧维护lifetime/其它业务。固定候选前不W2/双SID/发布。 |
| Windows证据贡献 | docs/分支仅.adg/evidence/0.2.24/windows-dev/<不可覆盖轮次>/**与docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md；最终验收另用windows-final-handoff.md和windows-final/<轮次>。 | 不改IMP/TEAM/README/共享索引；本对话接收后回写。路径创建时按真实结果，没有预填accepted。 |

每个文件同时只有一位写者。未分配旧文件默认只读，包括windows_private_create.rs、windows_query_job.rs、runtime_lifetime.rs；可在新模块复用现有接口，若必须改旧接口，回交具体原因/补丁，由本机接或明确转交单文件。不要因为文件名含windows就接管整条路径。

现有shared-path-policy revision7的共享入口保护继续生效。本表是人工明确写集，不宣称已生成OWNERSHIP或跨机锁。真正进入正式TASK前本对话须将新IMP/TEAM及合同登记到有效共享策略/所有权计划；不手工伪造runtime记录。Windows贡献不触及这些入口，正式状态由本对话维护。

## 来源与分支

- 代码主线：main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；Windows贡献建议codex/024-windows-native，独立干净checkout承接该基线。先fetch核对main，若主线已演进先报告并承接新的共同提交，不reset覆盖本地修改。
- r5技术输入：codex/official299-windows-proof的test交付cba03b42151843c753f25f53ca71a6b2b31dccef、实际执行源66afa61133d7654d3a2dfd172f49845bfb5272c3；只读提取test/official299-windows相关原型与不可变回执，不能全分支合并旧release/维护包改动。
- 当前文档主线：docs/governance-main；Windows证据贡献建议docs/024-windows-dev。代码和文档必须分支分离，代码不入docs分支，证据不入main。
- 最终Windows验收必须改用本机发出的集成代码SHA/安装器/官方包摘要，不能继续拿贡献分支的旧可执行包验收整版。

## 分批提交与接收

1. Windows先交W0第一批真正可编译/可审查的平台原语与定向证据，及时commit并push；不要攒到W1或末轮才交。
2. 每批回交代码分支/SHA、docs分支/SHA、基线与实际执行源码SHA、变更文件清单、实际命令/结果、环境与首错/未完、测试残留清理、共享入口最小接线建议。
3. 本机先核对实际diff/依赖/结果，再cherry-pick或按明确范围集成；由本机接共享入口并及时push共同main/集成基线。Windows继续独占模块，缺共享接口只停接线，不修改Mac负责的类型凑编译。
4. Mac集成引发Windows专属编译错误，Windows修本线文件后小批再交；不反复跑原r5技术可行性链，不把同一问题跨机中转一轮又一轮。
5. 所有代码分支只贡献，不合main、不打tag、不发版、不改官方2.99。最终main/release事实由本对话统一。

## W0首次启动话术（已执行，仅保留来源）

> 按docs/governance-main的docs/03-开发实施/TEAM-0.2.24.md revision1和IMP-34开展Windows W0开发，先fetch并核对基线main 0828a3955。只写新infrastructure/windows_official_runtime024/**和test/windows024-native/**，将官方2.99 r5已证明的SID/私有记录、保留进程句柄/Job、正常退出/监听与路径事实做成真实Rust原生后端和可编译定向测试。r5 test输入cba03b42/实际源66afa611，只取相关原型，勿全分支合并。原30/5/10秒、未知拒绝和两锁有限例外保持；共享mod/入口/Schema/持久格式/安装器先只给建议，等Mac接口提交再接W1。小批提交推送代码及独立docs证据并回SHA/结果/首错/清理；不跑全量/双SID最终验收，不改日常配置或官方包，不发布。

W0已按该提示交付，当前不要重复执行这段启动话术。该提示只曾放行W0独占范围，不让Windows在无接口时猜安装/CLI接线。Windows须自行先读完整TEAM/源合同；此处不是“测试一下旧原型”，输出是准备接生产的独立平台模块，定向测试服务于本次新实现。

## 最终验证交接

最终提示词由本对话在IMP-34 P24-05达成后生成，必须带当时真实Git SHA/制品摘要/受影响验收表/已过可复用项，不现在编造候选编号或承诺已通过。由用户复制到原Windows对话，沿Git回交完整表。本轮只提供当前开发话术；最终话术等集成候选固定后交付。


## 2026-11-10本机已开工及共享接口检查点

用户已授权本对话目标模式执行，本机P24-00路径核心代码main 9f1ffe677已推送，[program-layout-r1](0.2.24-contracts/program-layout-r1.md)为共享接口只读入口；此批只有位置解析/版本路径类型，未冻结current.json持久内容，W1仍等待。Windows W0不受此等待影响；Windows是否已开工不从Mac状态推断，以实际回交为准。

有效shared-path-policy由provider重封为revision6并读回核对，新增DMD-POST023/IMP34/TEAM024/共享合同受保护，原型共享路径保持。未伪造正式团队/OWNERSHIP/派发回执；已有r1提示词的独占W0写集保持，本修订不扩Windows路径。


当前Windows下一步：保留本线成果及4个清理未完的失败目录事实，等待本机完成共享供应/安装/CLI消费者后通过Git给W1精确接线合同；不重复原r5或W0已通过组件。当前本对话继续本机开发，不要求用户再次授权开工。最终W2提示词仍等固定集成候选后交用户。


## 2026-11-11 W0接口修正及当前共享提交

本机真实消费者已推进至main9125f7134，实际官方输入见main3d3c90943，[消费者r2](0.2.24-contracts/program-consumers-r2.md)为后继共享入口；版本更新/CLI/卸载未全齐，W1/W2未放行。内部ocx-runtime(.cmd)与用户显式ocx(.cmd)分离，Windows不要按r1早期稳定入口假定用户命令已启用。

W0新增可独立修正项：[user_profile与opencodex_home分别传入](0.2.24-contracts/windows-w0-integration-findings-r1.md)，本线原样模块当前把二者强制写成同一路径。继续在原独占写集按真实fixture补组件/反例并Git回交，不改共享层/官方包，不额外跑r5或最终双SID。这里只是Git材料已就绪，不冒充Windows已消费/执行。


共享路径策略已由provider递增至revision7并核验，仅新增本机单写的消费者合同r2及W0接口finding，未将Windows贡献handoff列为本机独占，不阻断正常Git回交。历史revision6/早期检查点按原时点保留。


## 2026-11-11 W1可并行开发放行

共同基线main 09392b38e09ea919566ccce831dcc95d16e7163b 已推送，当前正式接线输入为 [W1原生接口r1](0.2.24-contracts/windows-w1-native-interface-r1.md)。现可在既定独占文件并行做：W0 HOME/Profile修正；W1-A当前用户PATH原生读/精确expected替换/有界广播及反例；W1-B MSI/NSIS真实024官方资源映射/默认与自定义I及保留范围。共享事务/CLI包装/生命周期/命令注册由Mac承接，Windows不猜新JSON或自行改父mod/工作流。此前“等全部共享再开始”的历史文本按其时点保留，不再阻止这三个明确独立项。

当前W1-A只交native原语和定向证据，尚不是共享终端启用/恢复/卸载整链；W1-B旧wix023.wxs仍需改runtime-bundle-0.2.24-win32-x64.tar.gz及official-runtime.tgz。文件名保留不意味策略兼容023。实际平台descriptor由024stage生成，不拿Mac摘要冒充Win输入。先原语/资源小批提交及时push，完整W2仍等固定候选，不重跑旧r5/无假设全量。

> 先fetch main 09392b38e，按docs/governance-main的TEAM-0.2.24 revision5与windows-w1-native-interface-r1继续。先修W0区分user_profile/opencodex_home，再并行做当前用户PATH原语和MSI/NSIS的024官方资源映射；只写既定Windows独占文件，不改共享层/工作流/官方包。定向验证后分批及时提交推送代码和docs证据，回SHA、首错/未完与清理；不跑旧r5或W2全量，不发布。

这段由用户复制到原Windows对话；Git材料已推送不等于Windows已接收/执行，真实状态仍只归IMP-34。

## 2026-11-13 W1回交后的协作边界

本轮消费TEAM revision5的W1接口r1；后继共同代码基线mainf8e3cf423，代码来源codex/024-windows-native b8f1bba91，证据来源docs/024-windows-dev 61996e034。[接收复核](0.2.24-contracts/windows-w1-receive-review-20261113.md)区分组件证据、源码行尾差额、未完接线和清理；实施状态只归IMP-34。上文各历史提示词按原时点保留，当前不再派发其已交付的W0/W1组件任务。

Mac单写共享终端事务/命令注册/卸载helper与生命周期消费者，Windows继续保留现有独占文件；没有新的实机输入时不反复重跑组件。产品共享接线与固定候选就绪后，才按实际diff给Windows定向或W2提示词。用户新增约定为“需要Windows时停下给提示词，由原Windows对话沿Git执行”，不自动跨对话派发，也不在等待中擅改Windows独占路径。未放宽原判定/预算/双SID真实桌面或清理规则，不发布。

## 2026-11-13 W1-C终端原生接续

实际新基线main83b27e9ae7d0919f57298e38f5f52fd887d50650；详[终端共享事务与原生接续](0.2.24-contracts/windows-terminal-transaction-r1.md)。该合同及本表是本批临时单写范围，不意味着接管全部共享代码。Windows接收此批后先全产品MSVC编译，再在专用测试账户验证真实终端与PATH/恢复并消费新卸载helper；普通日常账户PATH不得写入。Mac按用户约定停在此处，直到收到Git回交再恢复本机共享开发。

需要验证的明确新事实：ocx.cmd→控制台ocxd.exe专用提前模式→同current官方入口，实际参数/stdio/退出码；五个before/after边界的pending继续及外部变化拒绝；正常非升级卸载先撤销终端、升级保留，非0保留程序/配置。尚非固定候选，不跑W2、双SID、全安装器sweep、真实业务同步或模型请求。不重复已接收W0/W1组件测试，除非本批实际变更影响其断言。原失败目录与自动审批清理拒绝仍保留，不绕过。

> fetch main83b27e9ae，按TEAM revision7与windows-terminal-transaction-r1接W1-C。先MSVC正式编译，再在隔离测试账户验证ocx参数/stdio/退出码、同版本同H、启停终端与pending恢复/外部冲突；MSI/NSIS非升级卸载接--uninstall-terminal-cli --confirm，失败保留。仅写合同指定文件，分批推送代码和原证据，回SHA/结果/首错/清理；不改官方2.99或日常PATH，不跑W2/双SID，不发布。

## 2026-11-13 W1-D原生入口与官方脚本定向收口

W1-C已收到e004259fe/25b323801；完整事实只归IMP-34与[接收/修复/W1-D合同](0.2.24-contracts/windows-w1d-native-entry-r1.md)，未全通过。新共同main **0d231aebc**：W1-C组件两笔已接，主线修.mjs解释器准入，并准备独立native ocx bin；产品事务尚未切native入口。W1-C原提示词按时点保留，当前只执行W1-D。

采用原生 **I/ocx.exe→同目录ocxd.exe专用模式→同current官方Node/脚本**，不经过.cmd重新解析。Windows负责schema3/命令归属/路径与安装器接线及真实终端/官方消费者定向验证；Mac停止本表临时写集直到Git回交，不再跨机“一处失败一轮”。普通W0/旧r5不复跑，W2/双SID/全量sweep不跑。真实dev MSI卸载定向覆盖正常/失败保留，不把原条件五格等同卸载实际通过。已有旧失败及清理拒绝均不覆盖、不绕过，源码/制品/执行环境与所有独立项原输出分开保留。

> fetch main0d231aebc，按TEAM revision8与windows-w1d-native-entry-r1完成W1-D：原生ocx.exe入口/安装器/schema3接线，补默认PowerShell混合参数与新终端裸ocx、官方2.99版本/自定义H读取、受影响PATH恢复及真实MSI卸载。隔离测试账户，不改官方或日常配置，不放宽断言；一次回完整状态表/首错/清理，分批推送代码和新原证据，不跑W2/双SID，不发布。

## 2026-11-14 W1-D接收后仅补MSI真实事务

代码b476fa933/证据7d8e5db85已回交，主线三笔8e2c74690/053b0a826/a02b244e9按同树接入，后继fd2538e4d仅等价修Clippy的单元素循环。完整事实及包SHA、环境要求、独立用例范围见[MSI唯一补验合同](0.2.24-contracts/windows-w1d-receive-msi-followup-r1.md)，实施状态只归IMP-34。上文W1-C/W1-D提示词保留原时点，不作为当前重复派发。

原生入口/PowerShell7默认参数/裸ocx/原官方2.99自定义H/10格pending与schema3/NSIS宏均已有定向原证据，不再完整重跑。真实MSI安装被当前测试机DisableMsi=1以1625阻止，正常/失败卸载尚未执行；不得改策略、提升测试用户或用组件helper/NSIS替代。请Windows使用本身允许per-user MSI的隔离测试环境，只补实际安装前提及两项卸载结果；没有环境就回环境事实，不能相同条件反复1625。Mac在交付后停止新写上述Windows范围，等待Git回交。

> fetch mainfd2538e4d，按TEAM revision9及windows-w1d-receive-msi-followup-r1，只在允许per-user MSI的隔离环境补真实正常/失败卸载及保留。复用已核包SHA，不改当前机器策略/测试令牌/安装范围；已过终端/官方读取不重跑，无可用环境就回环境事实。分批推代码和新原证据，回结果/首错/清理，不跑W2/双SID、不发布。

## 2026-11-14 MSI环境回交后的等待边界

Windows已按revision9“无环境回事实”交代码f66d9d59f/文档24a4a4673；代码树与mainfd2538e4d一致，无新实现可接。实际接收事实见[MSI环境盘点接收](0.2.24-contracts/msi-environment-receive-20261114-r1.md)，正式状态归IMP-34。没有可用现成隔离Windows环境，真实MSI两项仍未执行。当前四视图Installer策略键缺席与旧out03有效DisableMsi=1失败分时点保留，不能互相替代。

当前不要再复制上段提示词启动相同环境盘点/安装重试。先由用户提供可用隔离测试机/VM或明确创建环境的范围，Windows才恢复原MSI-only合同。没有新环境/授权时不创建VM、不改日常主机策略、不提升测试用户或改安装范围，不重复已过终端/官方消费者。Mac保留原单写责任与整个IMP未完范围，当前不新派Windows任务或宣称MSI接受。

## 2026-11-14 管理员MSI接收与W-L1生产接口放行

用户已在Windows原对话另行授权本机专用临时管理员场景，已按0f1dc195e/c84fc4a99接收；[接收复核](0.2.24-contracts/windows-admin-msi-receive-r1.md)区分管理员两项真实通过、普通用户未验和Profile/Public清理未完，不把当前例外追溯成旧普通场景通过。上文MSI-only/环境限制保留其原时点；当前无需再跑已过管理员MSI。

本次共同源 **main35f41ecc0e700a3a77d121cc2dccf7258c327bc1**，正式共享消费者接口/逐文件临时写集/原包输入SHA及受影响定向表见[Windows生产生命周期接口r1](0.2.24-contracts/windows-production-lifecycle-interface-r1.md)。modules::runtime::official_lifecycle024已可编译消费，产品Windows尚未切换；Windows自主把原创建句柄/Job接到实际Start/Status/Stop/Restart，沿同一30/5/10/Restart30及Stop子10预算，不再依赖原版没有的desktop_lifetime。实际原包/产品调用证据才算生产接线通过，native组件/HTTP返回/launchPlan均不能当正常退出或客户端恢复证明。

用户最新协作方式为Windows范围内持续排查→批量修复→定向验证→及时推送，非必要不回Mac；此合同内小修无需逐次请示。共享范围之外、新恢复/跨进程持久协议、改变既定方案或缺环境/授权时，带具体证据和最小方案回交；不得为效率放宽未知/超时/恢复/全进程要求。Mac在上述写集发布后停止并行写，直至Git回交；文档/其它Mac独立范围不归Windows接管。固定候选前不W2/双SID、不发布。

> fetch main35f41ecc0及本次docs/governance-main，按TEAM revision11/windows-production-lifecycle-interface-r1连续执行W-L1。消费真实LaunchPlan/ActionWindow/typed验签与保留owner Stop接口，把Win32原句柄/Job接生产Start/Status/Stop/Restart；用固定官方2.99和隔离普通Profile验证原预算/归属/端口/正常退出及失败保留，逐项回完整状态。仅改合同写集，小批及时Git推送；已过MSI/CLI不重跑，不改官方/日常配置，不跑W2/双SID，不发布。缺持久协议/范围外接口再交具体最小需求。

## 2026-11-14 W-L2四项共享边界，当前生效

Windows已交3a9b71f7c/13ea948b2，本对话已按净树接收；正式事实/未完只归IMP-34 revision138。后继共同代码main12746c005cb2621d2f1f44d12c0af501613fef1a；新[四项共享接口与W-L2合同r2](0.2.24-contracts/windows-lifecycle-shared-r2.md)取代文末原W-L1派发的当前执行口径，历史r1结果保留。trait仅声明/编译通过，不当原生能力已实现。

Windows单写合同内原native后端/生命周期文件；新增windows_client_originals024/**、windows_owner_host024/**、windows_supervisor024与windows_diagnostic_roles024及其tests；ocxd.rs只Windows internal-owner模式分派，可新增bin/windows_owner024.rs；Windows注册/消费者与必要Cargo Windows API feature逐文件按r2允许表。原件schema/owner-rendezvous格式/精确命名空间/诊断role语义已冻结，Windows无需每处试验回Mac，可以连续批量修复和定向验证，超写集/改共享算法/新策略或缺环境时才带最小证据回交。Mac在Git回交前不并行改本写集。

先四边界原语/反例与MSVC正式编译，再单SID生产owner-host/原件捕获、完整Stop和后继、GUI更替；原预算/未知拒绝、原官方包、失败保留和准确清理不变。角色精确证明与独立native缺席前，非零查询不能被当作通过；不做名称或退出码白名单。既有CLI/MSI非受影响不重跑；新broker改变的生产调用须在新source验证。固定候选前不W2/双SID/全量安装器，不发布。

> fetch main12746c005cb2621d2f1f44d12c0af501613fef1a及本次docs/governance-main，按TEAM revision12/windows-lifecycle-shared-r2执行W-L2：实现原件同卷保管恢复、原创建者ocxd owner-host、精确supervisor原生缺席与源绑定诊断角色，先原语反例再单SID生产启停/后继/管理器更替。保留30/5/10预算、raw退出码和未知拒绝，只写r2允许集；范围内持续开发定向验证及时分别push代码与新原证据，回完整状态/首错/摘要/清理。不改官方或日常配置，不W2/双SID，不发布。


## 2026-11-14 W-L2原语接收后，r3为当前前提合同

共同代码main dbf974097d067e75c4de68f9fc681af8ab447b09，证据35e7336fc/70ebd2a67已接；进度/缺口只归IMP-34 revision139。新[普通令牌SD与broker环境r3](0.2.24-contracts/windows-lifecycle-prerequisites-r3.md)澄清两个最小请求，r2四trait签名、逐文件Windows单写集/官方输入摘要不变。Mac交接后暂停本写集至Git回交；不把r3误作四适配已完成。

Windows在现有允许集持续完成具体O/H/S/D及生产接线；O完整SD要求不缩为部分可读投影，1314/5/覆盖未知在原件变更前拒绝，旧pending保持Required。现普通token不具完整SACL读取能力，不改token/机器策略。H从既有普通交互桌面启动并原生核bootstrap和挂起broker无外层Job；原broker必须经manager更替实测同代持续存活。无桌面入口或SD正例时只阻对应依赖，不因一个前提卡住独立H/S/D开发，不反复跑相同失败账户/控制器。

r2允许集继续，新增共享常量为精确API合同而非证据能力；Windows如需调整其使用先给最小平台依据，不能私降范围/改预算/增加特权。每轮有可证伪假设，一次采全独立项，批量修复；及时分别push代码/新原证据。Profile先实际归档核验再移除；旧自动审批拒绝不重试/绕过。固定候选前不W2/双SID/全量安装器，不发布。

> fetch maindbf974097及本次docs/governance-main，按TEAMr13/windows-lifecycle-prerequisites-r3增量承接r2。实现四项具体适配；完整SD不可读则原件变更前拒绝，保持普通token/日常策略；broker仅从原生确认无外层Job的普通交互桌面启动，补原manager退出后同broker存活/接管。O受阻不阻独立H/S/D开发，别重复相同1314/带Job控制器轮次。只写r2允许集，保持原预算/未知拒绝，分别推代码和完整新证据/首错/清理，不W2/双SID、不发布。
