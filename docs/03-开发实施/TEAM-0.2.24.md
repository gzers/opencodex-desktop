# 0.2.24 Mac/Windows双线协作与Git交接

revision: 76；2026-11-22。实施顺序/状态唯一归[IMP-34](IMP-OPENCODEX-DESKTOP-34.md)，采用范围见其源链接。此文件只定义角色、排他写集、交接与提示词，不继承0.2.23 TEAM的旧派发状态。W0及W1三批组件已通过Git回交；管理员MSI补验与未完清理已接收，普通用户验收保持；当前r6实际r2首Start超时且原件已保管，Mac已定beforeResume受限恢复与共享扫描r7；Windows先当前现场修复准备再未来接口/性能，未整批集成Windows生产代码，Mac不并行改该写集。实际状态归IMP-34。最终W2仍须固定候选后由用户转交提示词。本对话不额外开Mac贡献对话。

## 2026-11-22 范围收缩：渠道与客户端配置不再由 Desktop 承担

Desktop 不再自管渠道、模型、协议、密钥与 OpenCodex 原生配置，也不再接管真实 ~/.codex。新增渠道按原型改、渠道改名不生效、2.99 配置导入迁移归 OpenCodex，不在本版 Mac 或 Windows 写集内。

启动时固定 OPENCODEX_HOME 到 Desktop 自管目录，并用 CODEX_HOME 指向其下的 codex-client；真实用户客户端目录只读。安装、修复、卸载前仍生成只读完整备份，但 Desktop 不创建原生配置锁、不写回 config.json。

Windows 写集因此收窄为程序根、当前用户入口和原生生命周期，不再包含客户端原件保管/恢复与 supervisor 缺席证明。普通 Stop 已恢复为已有认证的 Stop；真机启停仍待补验，不因本范围修正改成已验收。

详见 [所有权收缩合同](0.2.24-contracts/desktop-owner-scope-correction-r1.md)。实施状态仍只归 IMP-34。

## 2026-11-22 Windows 交接处置，当前生效

Windows 最新交接（代码 6532cbb6e、证据 32b0483e3）不整批合并：相对共同基线 cfb3c8aa7 的净差主要是已撤回的客户端原件保管、恢复、owner-host 与 supervisor 缺席证明。

Windows 以 main b6785b486 重建分支，只保留程序根、当前用户入口与 PATH 事务、安装器资源映射，以及真实正常 Start / Status / Stop / Restart（原 30/5/10 预算、未知拒绝、失败保留）。不得再引入客户端原件接管或恢复代码。

Mac 侧已完成官方 2.99 经候选 .app 的真实启停，并修好 CLI 端点与停止后状态。Windows 真机结果仍只归 IMP-34。详见 [Windows 交接处置](0.2.24-contracts/windows-handoff-scope-disposition-r1.md)。

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


## 2026-11-15 当前入口：采用有界访问安全r4

用户把SD/SACL覆盖评估交Mac并要求明确决定。当前[安全覆盖r4](0.2.24-contracts/windows-security-coverage-r4.md)替代r3关于所有新会话必须完整SD/SACL的前提；正式采用范围归DMD AC-P24-08，真实实施状态只归IMP-34 revision140。共享main2ebdd3ae0423903e9e713108a09a165ac36ead37只给新scope/schema常量与注释，Windows实现c47953ccfe4c66a976285eacffd691e3a3b0c01b仍独占分支，合入main后继续；本次不把未完实现整合主线，不拿main旧EXE代c479。

Windows继续r2/r3逐文件独占写集，允许新BoundedAccessV1 native读取/整树manifest/phase-v2/current-v2/policyDigest与真实capture/custody/restore/Closed、H真实GUI/官方owner和S/D组合；Mac暂停这些文件。共享trait签名/预算不变，新的安全保证范围严格按r4，不能自行删必需投影或退0x7。旧Full-v1或PartialDiagnostic不转换许可，审计SACL明确范围外未知，不声称full恢复；当前Start仍拒绝直到真实新证明完成。范围外再变/其它平台/官方包/日常策略仍回具体最小原因，Windows范围内持续开发批量核验及时push。

H普通桌面空组件的noJob/原creator/manager更替证据可复用；actualGUI仍需真接线，不重新要求用户跑同一空宿主。新mask/新schema有新假设才定向核验，O新范围正例不由旧partial补签；已过无影响CLI/MSI不重跑。Profile失败现场保留/清理原规则，所有删除拒绝不绕过。固定候选前不W2/双SID，不发布。

> fetch main2ebdd3ae0423903e9e713108a09a165ac36ead37及本次docs/governance-main，在c479上承接TEAMr14/IMP140/windows-security-coverage-r4。实现固定有界8投影与新v2原件/引用，所含不可读未知拒绝；审计SACL范围外不作恢复证明，旧Full/partial不转换。继续整树custody/恢复/Closed、真实GUI与S/D生产组合，完成前保持Start拒绝。只写r4承接的允许集，保留原预算/普通token/日常策略/失败证据，定向批量核验及时分别push，回完整状态/首错/实际source及EXE摘要/清理；不W2/双SID、不发布。


## 2026-11-15 当前入口：共享Route与同采样5秒r5

实施状态只归IMP141；本次[接口r5](0.2.24-contracts/windows-shared-routes-status-r5.md)提供main358e7fc4717787616f049f3bdc1cfe02f6d353f9，在Windows0549a660ee3f77ea243e798ec55b01f52545e492或其干净后继合入。代码含8e2699a91纯declaration提取及358e7fc47同窗口状态接口；不整合万行未完Windows实现、不把GUI空broker过当生产通过。

r2/r3/r4 Windows单写集保持，新增明确消费共享client_route_declaration::declaration、StatusSource::_at/deadline方法及status_source.rs Windows分支定向接线权限，原共享算法/Mac/HMAC/预算不另改。合并预演仅windows_official_lifecycle024.rs冲突：该main差异只是保守桥接；保留Windows完整文件/原生recovery_state_at，不用main桥接替换，不做全仓ours/theirs。Mac后续暂停Windows写集至Git回交。

先MSVC/共享纯例及真实同窗receiver，再原配置pin→declaration→完整官方副作用/NativeLease与H-S-D生产闭环，前提未齐公共Start仍拒绝。真实GUI空宿主通过证据复用，不重复旧fixture或未变化CLI/MSI；CAP/trust非空、普通MSI/历史清理保持未完。窗口期限、rawexit/未知拒绝、r4排除和v2不变，原有新假设/一次完整表/批量修复/失败现场规则继续，固定候选前不W2/双SID、不发布。

> fetch main358e7fc4717787616f049f3bdc1cfe02f6d353f9及本次docs/governance-main，按TEAM15/IMP141/windows-shared-routes-status-r5在0549后继合入。唯一native冲突保留Windows完整recovery_state_at；消费共享纯route及fetch/light/recovery同一个原5秒deadline。继续官方副作用/NativeLease和H-S-D原包闭环，前提不齐Start仍拒绝。沿允许集连续开发批量定向核验及时分别push完整状态/首错/source与EXE/清理，不重跑旧GUI空组件，不W2/双SID、不发布。


## 2026-11-16 当前入口：claim连续排他、原目录留位r6

正式进度只归IMP142。Windows代码3e19075846dfea81ba8392bb60a3d3031afbf116/本轮docs已接来源与证据；后继共享main fac8573e55441fe514919c63c7f1d65181ff3577只增storageStrategy/新v3格式声明，不含Windows本线万行实现。[r6执行合同](0.2.24-contracts/windows-claim-anchored-custody-r6.md)决定existing-root客户端根/子目录与两稳定对象留原位、其它原文件逐节点同卷custody，claim保持于每个变更临界段，禁止仅持N/L/K的claim空窗。

r2-r5逐文件单写集继续，新增明确originals目录内anchored/v3节点计划/phase/current/progress/coordination/adapter/history及其native tests，Windows负责实现完整节点/根children读回和中断/原件恢复，不改四trait/HMAC/Route/Status算法、Mac、官方、安装器/workflow或日常策略。Mac交接后不并行写。v2失败/pending保持不迁移，新storagePlanDigest与r4安全策略均入canonical链；本次不是安全范围再次降低。

先原存在根有真实claim竞争的单文件/目录锚点与v3反例，再整树/lease分段/恢复后继。原根缺席初始化与最终退根仍未解决，不把旧Absent断言降为留目录，保持对应gate拒绝和最终未验收；有新排他方案带最小原对象/时序回交，不重新跑无新假设的raw5/shareDelete。其它独立软件任务继续，已过空GUI/Route/Sample/Bun非影响不重跑。原预算/Unknown/rawexit/scene保留、普通MSI/B2/旧拒绝清理全保持，固定候选前不W2/双SID、不发布。

> fetch main fac8573e55441fe514919c63c7f1d65181ff3577及本次docs/governance-main，在3e1907584后继按TEAM16/IMP142/windows-claim-anchored-custody-r6继续。采用原已存在目录/两稳定文件留位、其它原文件逐节点同卷custody与v3 canonical进度，临界段保持claim/NLK，禁止释放claim只靠N/L/K移根。先原语/中断/整树验证，前提未齐Start拒绝；原缺席Codex安全退根仍单列未完，新具体排他方案再交。沿允许集持续批量修复核验分别push完整表/首错/source与EXE/清理，不重跑旧raw5/空GUI，不W2/双SID、不发布。


## 2026-11-17 当前入口：r7受限恢复与完整扫描性能

正式状态只归IMP143；当前Windows44bb9bdba、已载入r2产品357210891、docs两请求已消费。[r7合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)对应main 89e0195acb7a0c6e7549eafa376ec520ce981546新增BeforeFirstResumeRecovery及独立最多10秒恢复动作窗口，不热补旧进程/不签Closed。Mac未整批合入Windows实现，Windows在44bb后继合入共享声明，原r2 I/EXE/记录必须保持，不替换其正在加载的镜像。

先实现/合成验证受限外部修复器、完成当前r2原SID全归档/只读native预检，给准确75744/21500和3原件计划；本次新forced/外部恢复须在原Windows对话按AGENTS单独确认，旧两目标授权不扩。原件未还或无明确隔离采用前不重开官方新轮；原通道只清Job不还原/不允许broker正常退出，不能用新程序PID/JSON收养旧Owner。恢复输出独立ExternalOriginalsRestoredBeforeResume，Start超时事实不改，不自动后继。

未来原宿主capture前装partialStore恢复控制/原从未Resume证明，落实trait prepare/quiesce/restore与新恢复origin，保持r4/r6完整对象与claim/NLK、预算/未知拒绝。r2-r6 Windows单写集继续，新增程序镜像临时单写 **modules/runtime/program_image.rs及其Windows局部模块/定向tests**；program_selection/official_lifecycle仅cfg(test)只读costhook。Windows负责实现和实际MSVC性能/反例，Mac暂停该性能写集负责共享语义复核与集成；backup/safety/hash等未放行，不变HMAC/来源算法/其它平台/官方包/根构建。共享policy不得因效率删full扫描或改缓存语义。

当前恢复与只读计划优先，自有副本可平行计量/编译；原件未还不真实原包重跑。完整校验/每byte/hash/原权限/目录与source-current/原操作边界保持，Windows先全阶段计量再单pass优化，不跨动作缓存/元数据代hash/延预算。整个范围内连续批量修复及时分别push；新范围/现场执行确认在具体计划就绪时一次完成，不每个小修回Mac。既有未完/失败证据/旧删除拒绝全部保持，不W2/双SID、不发布。

> fetch main 89e0195acb7a0c6e7549eafa376ec520ce981546及本次docs/governance-main，在44bb后继按TEAM17/IMP143/windows-preresume-recovery-performance-r7继续。先准备当前r2只读全归档/native预检及精确修复计划，在原Windows对话一次确认新两目标forced/3原件归还后执行，不能沿用旧清理授权；不签Stop/Closed或后继。未来宿主在首变更前装partial恢复接口；共享program_image Windows优化/tests已单写授权，先量阶段再等强优化，full inventory/每byte/hash/原预算未知拒绝不变。原件未还不原包重跑，及时分别push完整状态/首错/source与EXE/清理，不W2/双SID、不发布。


## revision18 异步并行接续

用户已授权本对话目标模式至0.2.24发布前就绪。Windows73d8eb155/docs48181b0b5交回的是r7只读工具准备及编译OOM，实际状态唯一见IMP-34 revision144；本次不改r7恢复接口/扫描允许集，不整批集成未完成Windows产品。

- Windows继续原件/只读现场预检、beforeResume恢复、host与扫描优化独占集；环境可构建前不原前提重复OOM。现场恢复须先有可审查的归档/真实预检/具体计划再请求单独授权，旧场景forced授权不转用；普通token/日常策略/原预算/未知拒绝不变。
- Mac并行维护.github装配与Mac程序来源/封装验收、共享页面与NAME剩余路线，不改Windows独占恢复/扫描文件。本次main 462e2476a9275cdf82e9e3235ed84820ac2c28c7仅离线交付核验/装配及相关测试；Windows合入main只承接这些装配变更，无需重跑已过原生项。
- 双方一批完成即推送各自代码/证据分支。缺共享接口只暂停对应接线，其余继续；不每次准备动作另开全轮/改合同。Windows新handoff须区分工具/新产物/实际部署/生产完成；本次新r7轮次尚未补入旧handoff的事实由IMP回执注明。
- 不提前W2/双SID或发布。最终候选清单、签名/来源和受影响原生验收由Mac固定后一次给Windows，旧失败证据不覆盖。


## revision19 原Job只读观察接续

r7-async源码/实际执行源分开见IMP-34 revision147，不将后继无证据提交算生产完成。对旧357无Status客户端的阻塞，由[r7文末增补](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)提供先原型后R0的只读Job原生查询路线：Windows临时允许test/windows024-native/**新PSS/仅QUERY复制/原生反例，不改原已加载image及产品权限ABI；既有future-host/完整scanner独占写集继续。

当前不需Mac新产品接口SHA；main98787e64f已推只是Mac封装装配，Windows可增量合入，不因此重跑已有原生项。先交原型真实拒绝表，满足后当前R0全现场/唯一原Job当前历史/归档/精确R1计划；实际forced/恢复仍需用户对本现场单独确认。无权读取则停对应依赖，不提升/改策略/绕same-image；其余开发连续推进。当前尚无本机或Windows该新增查询原型通过的证明，不W2/双SID/发布。


## revision20 名称共享提交接续

Mac已按用户采用的官方显示格式完成共享模型保存/读回/协议组名称接线，代码634fc376cda19ca039e9fc30d3be83699058414a（前笔带点名称f7f5f8b1649d89181187692dd37cd93a31958509）；测试和未完只归IMP-34 revision148。Windows可增量合入，不碰本批模型配置/Vue写集，不需重跑本批普通页面或重复官方函数目录测试；原生候选受影响集最终固定后再给。

本批未改Windows原件/Job/进程/安全投影/扫描ABI，r7原Job只读查询增补及future-host/性能独占写集不变。Windows继续轻量原生查询/拒绝表，满足后只读R0，准备精确恢复方案再一次请求本现场R1批准；不以Mac名称单测/目录函数链代Windows恢复、默认客户端GUI或整版验收。新工程包须明确晚于名称源，旧462e包不冒候选；不W2/双SID/发布。


## revision21 PSS请求答复与并行接续

Windows89f55e76d／最新540025ef6证据净接收见IMP149，不改产品接口或既有独占写集。r7文末允许PSS HANDLE_NAME_INFORMATION(0x8)在test/windows024-native/**自有原型中用于完整类型，禁读/日志ObjectName；原Unknown拒绝/唯一同内核Job等不变。原型真实过后可只读R0，未授权本现场恢复，旧归档/44项复用不重跑。future部分捕获sink/新恢复origin/claimNLK/完整return与scanner继续本线，Mac不并行改。默认客户端真实展示与Mac生命周期缺口由Mac独立继续，最终集成候选仍未固定，不W2/双SID/发布。


## revision22 三文件scanner接收

Mac main 7fade183da7b43fa88fff3e459404aac704ff21b已按净差接入64bf7aa73三scanner文件，与Windows89f55原字节同树；本机不更改Windows独占算法/恢复/Source/Stop消费者。本批35项Mac回归＋1真实完整镜像/清理与Clippy/fmt归IMP150；贡献本线可增量merge此main，三scanner内容相同不应自行改回参考旧实现。只有真实冲突由双方针对文件处理，不用全仓ours/theirs。

r7恢复/PSS0x8补充/partial原件/预算与全部前提保持；后继Windows独占继续，不因Mac接这三文件放开Start或当完整Windows已集成。Mac继续本机独立剩余功能与最终Mac候选，Windows最终W2仍等全体集成源码/制品固定后一次交验；不重复无影响已过项，不发布。


## revision23 Mac真实模型页保存接续

Mac main cb82a7a06bfd19e74872b2d5b3ef3b47c50c98c6修首次单scope纳管默认引用，29bdae6fba4f58653e26e43063d0e112292468e1为本机AX验证器后继；实际源码/31测试/真实应用保存重开证据只归IMP151。均无Windows独占文件/接口变化，本线可自行增量合入，不重跑无影响原件/Job/CLI/MSI。根creation取消与NAME最终外部客户端/回退、最终候选由Mac继续，Windows按r7/PSS0x8/future恢复/scanner独占集并行；当前r2实际恢复仍需完整计划及用户单独确认。


## revision24 三请求一次接续

main 78dc2b8149c4b2f948f6de0d649bea7b46dada02新增独立v2附件标识（四trait/CONTRACT_VERSION4不改），adcf4f94edf73c6afb2675a34438f066be8b2eb8是Mac只读取消前置，非新实际恢复能力；状态归IMP152。r7文末已明确Windows未完成捕获variant、同线程原worker/失败持锁处置、文档化对象类型查询三请求，不再等Mac另造产品接口。Windows在57c982后继增量合入，沿自身before_resume_record_v2、v3/capture/journal、原ownerworker/control及test/windows024-native独占集实施/定向负例；Mac不并行改。

旧v1/oldv3不迁移或fallback，原deadline/所有8投影/未知拒绝不变，DATA不授native/Closed。query只原型通过后可只读R0；当前r2实际force/归还仍需精确计划及独立确认。未有activelease跨线程/unsafeSend/自动Drop解锁许可，不把新recordconstant当工厂已实现。Mac继续真正目录取消/一般图/候选及NAME外部客户端，Windows恢复线独立继续；不W2/双SID/发布。


## revision25 目录保管与Windows新停点接续

Mac main cfd41bf6e4cd055b69d9d27a4ca1ccc25f7332eb只含Mac首次目录取消保管内核及隔离验证器，生产许可/独立索引取消终态尚未开放；实际状态只归IMP154。Windows可自行增量合入，不重复无影响原件/MSI/CLI/GUI；Mac继续取消索引/解析/生产权限及UI接入，不碰Windows恢复单写集。

已净消费Windows9fdd4c618／docs f6f8677a的native-return/object-type两轮，原件返回后的最终release/原Owner整链仍由Windows单写继续；TEAM24的v2与同线程worker决定保持，无新共享ABI依赖。零访问DuplicateHandle24项raw50的新事实只暂停当前r2原Job证明/受限恢复准备，不阻future worker/v2/最后释放。r7新增说明禁止把旧loaded357的cleanup命令当Status探针；没有新的有据路线前不反复换旗重跑。此处不放宽类型完整覆盖、读取权限或same-image，也不授权r2forced/归还；具体外部修复仍须可审查方案及用户对当前现场单独确认。固定候选前不W2/双SID，不发布。


## revision26 首次取消索引与Windows原worker接续

Mac main e84cff4610c8b1f712747c874f9102904f2e30e6仅首次目录取消的原索引计划登记/读取/中断重开、Mac状态枚举和共享前端阶段标签/拒绝验证；不提供生产移动工厂、取消终态或后继。实际状态/失败及补验结果只归IMP156。Windows可自行增量合入，不改Windows独占原件/worker/协调/来源/Job/扫描文件，不变四trait与CONTRACT_VERSION4，不重跑无影响MSI/CLI/既过原生项。

Windowsbf16cad3f/3b343c66b的7原worker场景与范围接收归IMP155；保持TEAM25/r7写集继续唯一原Owner完整恢复driver、独立v2准备前缀与可达私有host接线。当前r2的24raw50只阻其Job证明/具体恢复准备，不将其扩大成其它开发停点；没有新权限扩大、同旧image绕过或现场forced/归还许可。Mac继续实际取消的服务/来源授权、完成与后继、UI及其它Mac独立缺口。最终候选固定前不W2/双SID、不发布，旧失败不覆盖。


## revision27 实际首次取消收口与Windows释放回交接续

Mac main8140844dcb6a363263f679db5f6245102947f46a已接实际CurrentService/Guard角色识别、明确四协调器释放回执和RootCancellationAbsent独立收口；原预算与失败/来源只归IMP158。后继资格/用户命令和入口未接，不将RootCancellationAbsent当正常Stopped/Closed或Start许可。Windows可自行合入：cfg-mac新增owner方法、Mac native/Index和前端标签，不改Windows独占文件、四trait/CONTRACT_VERSION4或r7允许集，不要求无影响原生重跑。

Windows44aa85f6c/b2b25c26e的五范围释放后全读/专门AbortedOriginals原语已接，真实Owner全driver、独立v2准备、可达私有host和完整原预算仍由Windows独占推进；r2 Job raw50未知依赖及本现场单独恢复批准保持。Mac继续首次取消后继/入口和其它本机缺口，最终候选固定后才W2/双SID，不发布。


## revision28 首次取消入口与后继资格接续

Mac mainfcf04caecdd7c328f03fcf1a6ceda3e18517d9bc接首次取消用户命令/确认预览和Vue状态、准确root_cancelled两事件回执，以及独立取消前驱的历史验证/后继创建捕获/实时状态资格。原包原生动作和存储捕获的范围、预算及未验真实界面/service只归IMP159；不是最终候选或整版放行。Windows可自行增量合入本机文件/共享命令的Mac分支/前端字段，不改Windows原件/worker/host/协调/扫描独占集与四trait/CONTRACT_VERSION4，不要求重跑无影响原生项。

Windows维持44aa85f6c后继独占，完整Owner driver/原slot验证/v2/privatecontrol及真实官方整链继续；r2raw50依赖/本现场单独恢复批准不变。Mac后续集中最终原包界面/官方服务后继、其它取消/中断及既列缺口，不为已有通过项重复空轮。最终源码/制品固定前不W2/双SID，不发布。


## revision29 实屏后状态采样与独立Windows接续

共享main80fcd4d2d修get_status_snapshot_with_collector的排队等待：try_lock忙返回明确不确定错误，不复用旧成功、不开始第二采样，不改变原budget/collector映射/Windows Owner。45571c2b8只加Mac真实GUI验证器，81f52452c只修测试遇busy-before-mutation的明确重新确认/精简输出。真实sealed包来源只认45571，本批状态归IMP162；非最终候选。

Windows可自行增量合入。共享commands/mod.rs若与本线注册发生冲突逐段保留双方真实净差，不全文件ours/theirs；原私有Owner/恢复/扫描独占集、四trait/CONTRACT_VERSION4和r7无变化。本次status API在busy时现在明确Err，Windows既有定向调用须按原Unknown/Required处理，不能吞错放Start；固定候选时再验真实并发调用，不提前W2/双SID或重复无影响MSI/CLI/扫描/旧归档。

Mac实际原包首次取消和官方后继Ready/正常Stop已验，仍需其它既列缺口/最终状态投影与候选；Windowsdd42d5e后继完整Owner driver/slot复核/v2全native/privatehost继续，r2raw50依赖及synthetic27960清理计划按原边界，不因本批观察修复放宽。继续异步Git回交，非必要用户不中转；不发布。


## revision30 Mac更新归档装配接续

Mac mainfb755b8ea63d8a3fc9146dc4db491931919905b0仅.github/scripts新Mac signed-updater校验器、对应离线反例及CI/Release装配；状态/真实本机测试与正式发行证据边界归IMP166。Windows可自行增量合入，不要求重跑已有Owner/CLI/MSI/扫描或Mac页面；没有Windows独占文件/四trait/CONTRACT_VERSION4变化。新Mac步骤必须读取真实tauri输出archive/sig和tagged公钥，缺任何事实保持失败；发布总拒绝门仍在，非正式发行或放宽。

Windows361cdedf0/f8552e56c回交与合成残留只读事实归IMP165，原nativeOwner/returnslot/privatehost/v2独占继续；当前r2与synthetic27960恢复/清理仍需准确原事实和具体计划，不借本批发行工具授权处置。Mac继续Windows封装读取方式/发行核验及其它剩余本机需求；最终完整源码/制品固定前不W2/双SID，不发布。


## revision31 Windows签名封装内容装配接续

Mac main4ea82e8a61fe00c1f76334db5e39654f49f4342e仅新增.github Windows签名installer内嵌内容校验及CI/Release接线和离线tests；真实探测/签名fixture/未完成实际候选边界只归IMP168。不改安装器模板、Owner/恢复/扫描独占文件、四trait/CONTRACT_VERSION4。Windows可增量合入，不要求因此重跑既过native恢复/MSI/CLI/旧归档；下一真实最终候选安装器出包后会新增这层校验，不在当前未冻结产品图跑W2/双SID。

新输出仅8项当前构建资源/程序bytes绑定，MSI目录表/costing及NSIS安装行为仍由原Windows native安装验收拥有，不以embedded-payload pass替代。流水线分kind收集两结果后统一失败，publish总门仍拒，未授权新发布。工具依赖installed7zip/minisign/当前stage内容和签名，缺任一保持失败，不用旧022探测结果放候选。

Windowsf8feb3825/801e53cc8已归IMP167，继续原独占完整Status5/Owner/nativeV2/privatehost/真实原预算及清理计划；当前r2raw50和synthetic27960处置不因这批Git接线获得force/归还许可。Mac继续发行最终入口/其它需求，非必要用户不中转，及时push，不发布。


## revision32 首次部分捕获取消接续

Mac main346b46a69b1d9951335359a334cda7bf21ddd132只首次新根部分槽的取消/独立历史/后继、Mac确认状态及测试准确清理；数据合同与真实native预算只归IMP170。新plan/review schema2明确保部分vault投影，不读旧v1/fallback/收养；测试目录之外DB清理只准确task-key，不改共享全局L或其它机器现场。最终signedapp部分分支与原型一致性还未验，不将source测试/后继storage capture冒官方服务验收。

Windows可增量merge这些Mac文件/UI，不重跑无影响Owner/MSI/CLI/扫描。原smoke新增cfgtest捕获checkpoint只在task根生效、不改变生产代码路径；Windows若遇相同文件diff逐段保其真实注册，不全文件ours/theirs。原Windows独占stage/worker/privatewire/v2、四trait/CONTRACT_VERSION4不变，继续本线。58f0d49f2与0d6fa9c0d回交已接IMP169，codec不是native可达权限、当前r2/synthetic27960仍按原计划与独立批准边界，不W2/双SID、不发布。


## revision33 部分捕获实屏诊断后接续

Mac测试ebe80da86、只读重核修复0d44a4643、pendingRequest修复0b77472be及后继测试bb13a7f12/88b8fc33d均可增量合入；源、三包实际绑定、四失败轮次及未完唯一归IMP171。Rust取消/Windows原生后端/四trait均无新变化；前端明确已结案的只读协调和原invoke结算前不释放UI动作，不因此提前开放真实Start。Windows无需重跑无影响MSI/CLI/scan/归档/原Owner测试，继续原独占privatewire/native端点/V2恢复。

Mac下一本机任务只针对gui04取消前status等待的真实阶段/线程取证，未知不重跑全流程；gui02旧cleanup不足完整RootAbsent的问题已收紧后继测试，不用当前无残留代签原正常Stop。其它既列功能/原型与最终候选仍未验，当前两端禁止W2/双SID/发布边界保持。Windows最新58f0d49f2/0d6fa9c0d已接，无新接口依赖；不借本批授权处置原r2或synthetic27960。


## revision34 状态轮询争锁与只读诊断接续

共享main17f07758e/51c8aca18/bec99201e只改状态polling、Mac准确隔离stage及定向测试，实际结果和未完GUI边界只归IMP172。Windows可自行增量合入；busy不发事件、不改原due时间或成功事实，原Status5/Unknown/Required保持。无Windows原件/Owner/worker/privatewire/scan独占文件或四trait变化，不重跑无影响MSI/CLI/旧归档/原Owner；实际共享并发集成在固定候选核。

Mac继续单个真实包的只读状态IPC观察，不复跑供包/取消/启停。Windows077cbd56e/331ac32fa已fetch，尚待净差及原证据接收；原生端点桌面项Pending不能当通过。本批不授权r2/synthetic27960处置，原恢复/清理批准边界保持；不W2/双SID、不发布。


## revision35 Windows私有原生端点回交后接续

已接Windows077cbd56e/331ac32fa，精确摘要、151项状态和普通桌面Pending唯一归IMP173。原shared四trait/CONTRACT_VERSION4/r7写集不变；本机未整批merge原生实现。Windows继续唯一写原worker/privatecontrol/nativeV2/现有资格入口，不等Mac新接口。桌面只验证空fixture双端认证/四准确拒绝，不能当原产品Owner恢复或启停已过；原账户原普通令牌/noJob/同image/单次nonce和原5/10秒保持。

当前需要普通账户实际桌面运行已摘要固定的launch-r7-private-native-empty.cmd；在此之前不提升、不从管理器Job绕过、不使用旧image/pipe/cleanup替代。此依赖不阻其它独立授权开发。原r2/synthetic27960/40588及拒删现场仍保留，不借本批接收授权force或归还。Mac独立继续状态IPC/NAME/恢复与候选，最终冻结前不W2/双SID、不发布。


## revision36 Mac真实状态IPC及部分取消后继闭环后接续

共享main最新0ae095327，实际验证包编译源bec99201e；两者区分不冒最终冻结。Mac真实查询775.519ms/界面2146ms、首次部分取消和官方后继Ready→正常Stop/RootAbsent13已闭合，实际原预算及旧失败保留只归IMP174。436e6dd9a/841291a2a/0ae095327仅定向GUI验证器和激活完成前置，不改Windows独占文件/共享四trait；Windows可增量合入，无需重跑无影响MSI/CLI/scan/旧归档。

Windows继续revision35/r7原单写与普通桌面Pending；本机不等待此依赖，转剩余一般图/中断与NAME外部客户端/最终包回退。首次部分取消完整成功不等Windows产品恢复/整版可发布，也不授权旧现场force；原型文件继续原单写者，不同时改。最终图固定前不W2/双SID，不发布。


## revision37 Windows准备前缀读回接续

Windows96396f19b/3294b72b3已接，组件实际范围与157项状态唯一归IMP175；原四trait/CONTRACT_VERSION4/r7排他写集不变。本机未整批merge其生产分支。Windows继续fullSource与原nativeV2各覆盖/原Owner/privateendpoint实际普通桌面工作；不把带null安全baseline的Pending观察升级许可，不补旧Start窗口。

普通桌面入口仍用原四输入/55cd6ef…独立执行，不为本新增source覆盖未验产物；当前r2/40588/synthetic27960及拒删权限边界保持。Mac继续NAME隔离回退与其它本机独立项，不因本批要求重跑无影响GUI/MSI/CLI/scan。最终候选冻结前不W2/双SID、不发布。


## revision38 NAME备份回退及ChatGPT目标接续

main18a126a58只有config_file_v2新ignored文件事务测试、Darwin官方刷新验证器的backup-rollback分支及准确task前缀准入，无产品语义或Windows独占写集/四trait变化。Windows可增量合入，不重跑无影响nativeOwner/MSI/CLI/扫描；Mac实际组件与客户端仍未验边界只归IMP176。

最终名称显示对象按用户答复固定ChatGPT内本地Codex。工具禁止控制ChatGPT自身界面，不用其它UI技术绕过、不改日常配置或重启当前宿主；最终需具备独立隔离配置的实际客户端人工证据，不能以终端或官方目录刷新代签。此项只停真实客户端依赖，Mac继续后继代次取消/中断恢复/一般图，Windows继续revision37/r7，原r2/synthetic27960/40588与拒删边界不变。不W2/双SID、不发布。


## revision39 名称验证准确外部协调清理接续

main15dbb8286只改Darwin名称验证器的task-key预检/末端清理，不改产品/Windows独占文件或共享接口。实际原K释放与原字节、预检拒绝/0子进程负例及缺席补证唯一归IMP177；原round01的初始外部缺席未记录不补造。Windows无需重跑无影响nativeOwner/MSI/CLI/scan，继续revision37/r7本线。

原ChatGPT自身UI工具禁用不尝试旁路，人工客户端证据仍未完；这只停该依赖，本机继续其它尚未验收功能，候选未固定、不W2/双SID、不发布。


## revision40 Windows目录准备槽接收与Mac后继审查并行

Windowsb1ef52972/7adf93a64已接，准确范围与163项状态只归IMP179。原四trait/CONTRACT_VERSION4/r7写集不变，Mac不并发改目录/partial/source原生文件，未整批merge生产分支。Windows继续完整Source/initialSD/partialwork/nativeV2/原Owner与普通桌面独立依赖，原Public输入及旧r2/synthetic27960/40588不动。

Mac按client-successor-prepare-cancel-r1先补后继Preparing完整历史审查，再独立移动/索引/plan中断恢复；仅Macroot文件/共享必要字段归本机，前端和原型当前并行写集先核归属。首次取消、名称回退等未变通过项不重复；ChatGPT自身UI禁用依赖保持，不W2/双SID、不发布。


## revision41 后继准备只读组件接续

main845b28619只新增Mac generation的后继Preparing完整历史Review，7765271ff仅原生定向测试/driver。实际普通与真实取消关闭前驱、6测试/847ms审查、原9705ms首次取消及未完只归IMP180。无Windows独占文件/四trait/CONTRACT_VERSION4变化；Windows可增量合入，不要求重跑无影响MSI/CLI/scan/旧归档，仍按r7本线推进。

Mac接下来实现后继保全移动/独立终态、多计划namespace和已登记plan中断恢复；只读组件不开放按钮或Start，产品预览/前端/真实GUI仍待后续。原型文件保持其唯一写者，本机不并发覆盖。固定候选/ChatGPT选择器与Windows实际普通桌面/Owner缺口各保留，不W2/双SID、不发布，旧现场处置无新授权。

main cd7cd0a9e62b3f5f88e0054c3498cbfe18616943 增量为 macOS 独立取消计划目录校验，Windows 写集和共享合同不变。结果、证据和后继未完范围见 IMP181。

main 9daf5599151b65437f94ab2cadb1988060ee38c3 是 macOS 首次取消目录槽修复。原生回执、清理证据和未完范围见 IMP182。该批 Windows 写集和共享合同不变。


## revision44 Windows初始Start worker接续

共同代码基线main 9daf5599151b65437f94ab2cadb1988060ee38c3，Windows输入 0816047f30f667e56bea4167ec012f6e12bfcbdd，证据 cecd17bda58ebbccc0bd0bb0a0a017a5df1ab5ba。接收结果归IMP183。按[本次r7合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)继续Windows原宿主worker、生命周期/捕获/协调及定向测试。Mac继续后继准备取消。复用未受影响的扫描、CLI和MSI证据。

## revision45 Windows r7 Journal 交接接收

共同代码基线 9daf5599151b65437f94ab2cadb1988060ee38c3，Windows 代码 89e8c64478553c9715eb5babfe00f9eeed14af11，证据 420d7a948aeea76f198d6ffe8e2ee4a8b05499f0。IMP184 记录 Journal 两轮定向证据。按现有 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)继续原宿主 worker、显式恢复和普通桌面 private V2，沿用单写归属与既有验收预算。Mac 并行继续后继准备取消。既有旧 r2 现场保持原状。


## revision46 Windows r7 首启保留与最终释放交接

共同基线 9daf5599151b65437f94ab2cadb1988060ee38c3。准备释放代码 737bca2e3cd8899cda0d0d86bf8b5fbb446ebca1，证据 8b3ed982236014b0755538d7c3f00a3649c53fc3，后继最终释放源码 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2。状态以 IMP185 为准。Windows 继续按 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)和现有写集，用独立夹具复验最终释放与原持久 worker，接续显式恢复。Mac 并行承接后继准备取消。旧现场处置仍须单独授权。


## revision47 Windows r7 最终释放与租约范围接续

共同基线 9daf5599151b65437f94ab2cadb1988060ee38c3。最终释放代码 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2、证据 f260a34da4dcabe4bf15fe733f4cd9a0686d80b9。租约范围代码 e54a1156de42d445903faedb93aed4923f400b3a、证据 e9e0522629f522e172650f3ea3734fa6b6cc4ae0。结果归 IMP186。Windows 延续 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)现有写集，继续同原线程持锁、显式恢复及生产预算验证。共享接口沿用现行合同。Mac 并行接续后继准备取消，旧现场处置仍须单独授权。

## revision48 Windows 首启工作线程与 Mac 后继取消接续

共同基线 6bafdd342e69ee83e842f4b35a20bafa1320645a。Windows 代码 a5ba9aee338ed69841a836e4a5be639115fa8e1c，证据 b8fe82f802b4df3d2a621b416e365ca609ec5671。复核结果归 IMP187。按 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 与现行独占写集继续，Mac 同步承接 [后继取消合同](0.2.24-contracts/client-successor-prepare-cancel-r1.md)。共享接口沿用现行合同。

> fetch 最新 main 和 docs/governance-main，按 TEAM48／IMP187／r7 承接 a5ba9aee3。补交第一批 5/5 的实际测试源和原 EXE 摘要或更正回执。用独立夹具一次覆盖原 worker 连续存活、同线程同范围保留、失效 Start 权限、显式恢复派发和原预算，回交源码与 EXE 摘要、定向状态及清理证据。保留公开 Start 拒绝与旧 r2 现场。固定候选前继续单 SID 定向开发和验证。

## revision49 Windows r7 准备附件与原线程恢复接续

共同 main 6bafdd342e69ee83e842f4b35a20bafa1320645a，Windows 代码 cfce007f0ae1de13c20e797b9448b5fcc1605875，证据 99520867ba3836b5443f61fc4620e6968083987c。验收状态归 IMP188。Windows 继续独占 windows_owner_host024/**、r7 已授权的 Windows 原件／生命周期／协调文件和 test/windows024-native/**。沿用四个共享 trait、CONTRACT_VERSION4 和共享持久格式。Mac 并行承接后继取消和实际客户端验证。

> fetch 最新 main 和 docs/governance-main，按 TEAM49／IMP188／r7 从 cfce007f0 接续。先验证原 worker 在未来隔离 D 创建严格 V2 部分捕获附件，覆盖准确绑定、回读、超时和 I/O 失败。再完成原线程生产恢复及原预算计时，补普通桌面私有 V2、普通用户 MSI、官方单 SID 生命周期。保留首错和夹具事实，公开 Start 维持拒绝，旧 r2 保持原状，固定签名候选前不跑 W2／双 SID。


## revision50 Windows r7 Journal 计时接续

共同基线 6bafdd342e69ee83e842f4b35a20bafa1320645a，Windows 代码 6dbffae54c528750bfb6d5ad9b9e08dabaf78495，证据 8777e862f0b2017e94b77ad426ea7a6a324bcd9e。复核结果归 IMP189。Windows 按现有独占写集继续原 worker、Mac 并行承接后继准备取消。四个共享 trait、CONTRACT_VERSION4 与验收预算沿用 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。

> fetch 最新 docs/governance-main，按 TEAM50／IMP189／r7 从 6dbffae5 接续。完成原 worker 的严格 V2 部分捕获附件创建及准确读回，再验证同原线程的显式恢复和完整原预算。Journal 计时只作诊断，继续执行全量验证。保留旧 r2、公开 Start 拒绝与既有失败证据，固定候选前按单 SID 定向验证。

## revision51 Windows r7 持久化组件接续

共同 main 6bafdd342e69ee83e842f4b35a20bafa1320645a，Windows 代码 0d0c9d29ccd9075d59f92350c158c256c835924b，证据 3155e8fa0b9f6f82d023d415996e9edabffdc3f4。组件复核归 IMP190。四个共享 trait、CONTRACT_VERSION4、写集和时限沿用 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。Mac 并行承接后继取消及最终客户端验收。

> fetch 最新 docs/governance-main，按 TEAM51／IMP190／r7 接续原 worker。复用隔离 D 的 V2 持久化组件证据，继续真实普通桌面私有 V2、实际原 Owner 同线程完整恢复、普通用户 MSI 及官方单 SID 原预算验收。保留原超时和旧 r2 现场，公开 Start 维持拒绝，固定签名候选前不跑 W2／双 SID。

## revision52 Windows 原 Owner 普通桌面准入

共享 main 6bafdd342e69ee83e842f4b35a20bafa1320645a，Windows 代码 7fb2f870683566487cd0fee236f03dfa90fee93c，证据 4cf484e08e13060f0e8d8687bd755f1c69c1025d。复核归 IMP191。[r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)规定独立普通交互桌面、隔离 I/D/H、原私有 D 预绑定及准确的 pre-manifest 注入点。沿用 Windows 独占写集和四个共享 trait。

> fetch 最新 docs/governance-main，按 TEAM52／IMP191／r7 从 7fb2f8706 接续。用新普通交互账户隔离夹具测原 Owner 同线程 V2 Pending，真实捕获后、首次 manifest 意图前注入。回交三次完整扫描、原对象及锁、首错与预算。独立显式动作验证附件独占创建、刷盘、精确回读及否例。公开 Start 和旧 r2 维持原状，固定签名候选前保持单 SID 定向范围。

## revision53 Windows 完整扫描组件接续

共享 main `6bafdd342e69ee83e842f4b35a20bafa1320645a`，Windows 代码 `766d1f010be60865e7a425dab242ab5d861773b5`，证据 `f4bd13e48a82474b1cb23e000bfe86976733e4ce`。组件复核归 IMP192。沿用 TEAM52／[r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)的原生准入、Windows 独占写集、四个共享 trait、CONTRACT_VERSION=4 及原预算。

> fetch 最新 docs/governance-main，按 TEAM53／IMP192／r7 从 766d1f010 接续。先用新普通交互账户、真实 SID／会话、无外层 Job、独占 I/D/H、预绑定私有 D 与官方 2.99 夹具，记录 CaptureStore::capture_held 在 custody、前驱及 namespace 围栏后、首次 manifest 意图前注入时的原 Owner／worker、原件／锁、未 Resume、三次完整扫描、首错和时间。独立 10 秒动作只创建并核对原绑定 V2 附件。随后按既定合同验证同线程恢复、普通 MSI 及官方单 SID 生命周期。保持公开 Start 拒绝及旧 r2 现场，固定签名候选后进入 W2／双 SID。

## revision54 Windows r7 现有账户隔离补验

共享 main `6bafdd342e69ee83e842f4b35a20bafa1320645a`，Windows `18833e7b65c4cb48316f37ee15a3dc584650877e`，证据 `2a9c671969723fa9afca569b6ba2a163cb01a7e4`。本批独占文件和验证边界见 [r7 隔离根增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)，接收事实归 IMP193。首个可执行回交须包含入口预检和无 Resume 负例，完成后再安排普通桌面。

> fetch docs/governance-main，按 TEAM54／IMP193／r7，从 18833e7b6 接续。先实现并验证仅测试制品可用的隔离引导，绑定 marenben 的真实普通交互身份和全新合成客户端、协调、custody、I/D/H；入场前排除默认根和旧 r2，并证明缺失注入也无法 Resume。预检齐备后仅跑单 SID 原 Owner raw31 及同线程 V2，回交三次完整扫描、30／5／10 秒、失败与清理证据。共享接口依赖或隔离失证先回交。签名候选冻结前保持定向范围。


## revision55 Windows r7 扫描门槛接续

共享 main `6bafdd342e69ee83e842f4b35a20bafa1320645a`，Windows `4fc73cd1c26aa51fa288a3b436d2f0c050d36e1c`，证据 `34e36638acd7e94a0f414c96972eb121522308c1`。接收结论归 IMP194。启动器组件可作为后续独立入口的验证依据，实际普通桌面仍按 [r7 隔离根合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 准入。

> fetch 最新 docs/governance-main，按 TEAM55／IMP194／r7 从 4fc73cd1 接续。先完成仅测试制品可用的隔离引导和结构性不可 Resume 负例。准入齐备后，用 marenben 的交互式普通令牌、WinSta0/Default 和无外层 Job，在独立根跑单 SID 原 Owner raw31。回交 Start 计量扫描、恢复准备三次完整扫描、原线程及 V2 Intent/Pending 刷盘精确回读。本轮沿用 30／5／10 秒、四个 trait 与 CONTRACT_VERSION=4，保留旧 r2 现场。固定候选前保持定向范围。

## revision56 Windows r7 完整扫描原期限与普通桌面

共享 main 1c69bff16b1f6d1edf35c91c93640ee236894312，Windows b4e23087e5c9c90f0e0731453c0b1421b6be37c5，证据 ccc5e84286c3824f5ff663f891e76fc13405160b。接收结论归 IMP195，执行边界见 [r7 期限增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。Mac 提交覆盖前两处共享入口。第三处 Windows 入口、实机构建与验证由 Windows 单写。

> fetch 最新 docs/governance-main／main。按 TEAM56／IMP195／r7，从 Windows b4e23087e 接入 main 1c69bff16，补第三处 execute_admitted 原 deadline，再用 marenben 普通真实桌面验证九次首启扫描、原 Owner raw31、未 Resume 和 V2 Intent/Pending 刷盘读回。沿用 30／5／10 秒和原完整扫描断言，回交源码、制品、原始时间、首错与清理。七计量暂存可按原配对独立执行。候选固定前保持单 SID 定向范围，保全旧 r2。


## revision57 Windows r7 九次完整扫描实机回执

共享 main `040f411e6d48d97bde6e7b2e6c22860e547b6173`，Windows `8266cf6a6b67402719c89689984b2154c8cc4599`，证据 `b0eaf2e54a42cffe5298cad6d5892ba1c2b02379`。组件复核归 IMP196，实机条件见 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。Windows 继续独占生命周期与定向夹具写集；扫描器本批已在共享 main，原 trait、格式和预算沿用。

> fetch main／docs/governance-main，按 TEAM57／IMP196／r7 使用 8266cf6a 已固定九扫描配对。重连 marenben 普通桌面后先验证实际 SID、普通令牌、WinSta0/Default、无外层 Job 与独立 I/D/H，再跑 Host。记录同一 30 秒内九次完整扫描、首错、原 Owner、未 Resume、原线程和 claim/N/L/K。证明到达 raw31 且范围仍持有后再执行 5／10／5 秒 V2 定向诊断和精确回读。若超时，回交实际覆盖和首错。保留旧七扫描配对及旧 r2；固定签名候选前保持单 SID，不发布。

## revision58 Windows r7 修复配对的真实桌面验证

共享 main 040f411e6d48d97bde6e7b2e6c22860e547b6173，固定配对源码 daae8ffff7f304ec28b9c0214db26691e930a395，回执核验器 e19642724af81cc40c3dea6dbe4dc7d4caee9d21，最新证据 a223e560670550b43c6bd042833ebc1d4a01fccf。组件接收见 IMP197，Windows 继续独占定向验证写集。本轮保留现有配对，核验器独立使用。

> fetch docs/governance-main，按 TEAM58／IMP197／r7，用 Public 下已固定的 nine-r2 配对，在 marenben 的 session 7 真实普通桌面执行一次 Host。证伪目标是新夹具清除 raw32，在原 30 秒完成九次全量扫描并到达 pre-manifest raw31、Node 未 Resume。用 e19642724 核验原回执，另证同一存活 PID/FILETIME、原 Owner/线程和原 claim/N/L/K 仍持有，再做原 5／10／5 秒观察及同线程 V2 Intent/Pending 刷盘、逐字节读回。首错或未知按原合同拒绝并保留现场。旧原件恢复 r2 与本配对独立保全。回交 Git 证据，暂不做 W2／双 SID。


## revision59 接收外部恢复原语与成对预检

Windows 源码 `0e9f9cf486425d9fa4ba70ddcef376340f1abe5e`，证据 `87e664831c5774ee099a1d5929e0d217da03bf44`。组件接收和生产缺口见 IMP198。Windows 保持定向写集，组件接收无需重跑。真实 Host 沿 TEAM58 已固定的 nine-r2 配对和原回执核验器，原 30 秒和后续 5／10／5 秒预算照旧。新手工回执仍待交接。旧 r2 独立保全，候选冻结后再做 W2／双 SID。


## revision60 整树恢复组件与宿主回执独立核验

Windows 源码 a7d7284a3280d973aaacddf361adae6f6dcc8a3c，证据 f9bf34095e06e1fd5e267be89356e57fb96406e5。接收范围归 IMP199。Windows 继续独占原生与定向夹具写集，Mac 继续后继准备取消及客户端验收准备。

> fetch 最新 docs/governance-main，按 TEAM60／IMP199／r7 复用 daae8ffff7f304ec28b9c0214db26691e930a395 的 nine-r2 固定制品，用 a7d7284a3 核验回执。在 marenben 普通交互 session 7、WinSta0/Default、无外层 Job、Public 下 ocx-r7-isolated-owner-nine-r2 隔离根继续原定向试验。证伪目标是原 30 秒内九次完整扫描并到达首次 Intent 前 raw31，Node 未 Resume。独立证明同一存活 PID/FILETIME、原 Owner/线程和 claim/N/L/K 后，再做 5／10／5 秒诊断与刷盘读回。保留 nine-r1 raw32 和独立的旧官方 r2 现场。回交 Git 原始证据，固定签名候选前保持单 SID 定向范围。

## revision61 nine-r2 宿主与同线程范围验证

共享 main 040f411e6d48d97bde6e7b2e6c22860e547b6173，Windows 源码 f3564fbb34713beee97ed221fb4797b422b11d57，证据 ref 2e08a4463ec572c08787402cb8453b56a760a0f2。接收范围归 IMP200。Mac 与 Windows 分别推进，读探针限 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 的测试写集。

> fetch docs/governance-main，按 TEAM61／IMP200／r7 复用固定 nine-r2，在 marenben 普通桌面只执行一次 Host。验证原 30 秒九次完整扫描、首次 Intent 前 raw31、Node 未 Resume。独立核同一存活 PID/FILETIME，并在原 DiagnosticBefore 5 秒内由原 worker 实查 claim/N/L/K。两项齐备再执行原 10 秒 Explicit 与 5 秒回读。现有制品无范围探针时停在已取证阶段，另固定含探针的配对并保留本轮。回交 Git 原始证据。保全旧现场，候选冻结前保持单 SID 定向范围。

## revision62 接收原树绑定回归，继续普通桌面 Host

Windows 源码 f3564fbb34713beee97ed221fb4797b422b11d57，证据 e479ab8a5b657fe90987868a109378ffad6b8f46，组件接收见 IMP201。桌面输入继续使用 TEAM61／r7 已固定配对。Mac 独立推进后继准备取消。

> fetch docs/governance-main，按 TEAM62／IMP201／r7。若 out01 仍空，在 marenben 普通 session 7 单跑固定 nine-r2 Host，用 a7d7284a3 核验 30 秒内九次完整扫描、首次 Intent 前 raw31、Node 未 Resume。先独立核同一存活 PID/FILETIME，并在原 DiagnosticBefore 5 秒内由原 worker 实查 claim/N/L/K，再做 10 秒 Explicit 与 5 秒读回。固定制品缺探针时保留 Host 回执并停在 Explicit 前，按 r7 另固定测试配对。原失败现场保全，签名候选冻结前保持单 SID，原始证据走 Git。

## revision63 固定原线程探针的测试配对

共享 main 040f411e6d48d97bde6e7b2e6c22860e547b6173，Windows 核验器源码 673d1166334c4fb78d5094bc09ef130e578e0991，证据 644f8bc861bd7a1009aed9c5789699ff569f0895。接收结果见 IMP202。按 [r7 增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 配对调整三个测试路径常量及启动器，沿用原线程探针写集。Mac 并行承接后继取消。固定签名候选前保持单 SID 定向验证。

> fetch 最新 docs/governance-main，按 TEAM63／IMP202／r7 推进。先查 nine-r2/out01，空时用固定制品在 marenben 普通 session 7 只跑一次 Host，用 a7d7284a3 核九次全量扫描、30 秒、首次 Intent 前 raw31、Node 未 Resume。旧制品取证后停在 Explicit 前。同步固定含原线程探针的新隔离配对及全量输入摘要。新配对先独立证明同一存活 PID/FILETIME，再由原 worker 在 5 秒内核原失败与 claim/N/L/K，齐备才跑 10 秒 Explicit 和 5 秒读回。保留原失败及旧 r2，回交 Git 证据。

## revision64 接收探针配对，等待真实普通桌面回执

共享 main 040f411e6d48d97bde6e7b2e6c22860e547b6173。原线程探针源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7，证据 007dd301f16e47ed9afd59f91a17b3687e8b4dd7，固定路径和摘要见 IMP203。写集沿 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 与 TEAM63。Mac 并行推进后继准备取消。

> fetch 最新 docs/governance-main，按 TEAM64／IMP203／r7 执行。先消费已有 nine-r2/out01，为空时才在 marenben 普通 session 7、WinSta0/Default、无外层 Job 单跑固定 daae8ffff7f304ec28b9c0214db26691e930a395 的 Host。用 a7d7284a3280d973aaacddf361adae6f6dcc8a3c 核验原 30 秒内九次完整扫描、首次 Intent 前 raw31 且 Node 从未 Resume。保全该轮，停在 Explicit 前。新 range-r1 配对须现场确认真实桌面、令牌、Job 和父目录身份，独立证明同一存活 PID/FILETIME，并由原 worker 在 5 秒内复核原失败和准确 claim/N/L/K。齐备才跑 10 秒 Explicit 与 5 秒读回，证明唯一持久 V2 Intent/Pending 和独立字节回读。保留原失败及旧 r2，固定签名候选前保持单 SID 定向范围。回交 Git 原始证据。

## revision65 nine-r2 证据已接收，转入 range-r1

nine-r2 证据 cdf54c6a1cd095f8421c5e07a14929c30d6cc913 已按 IMP204 接收。沿 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 和 TEAM64 的单 SID、固定配对和写集继续。Mac 并行推进后继取消。

> fetch 最新 docs/governance-main，按 TEAM65／IMP204／r7，保全已完成的 nine-r2。用 IMP203 的 range-r1 固定配对在 marenben session 7 真实普通桌面单跑 Host，现场核 WinSta0/Default、普通令牌、无外层 Job 及父目录身份。独立确认同一存活 PID/FILETIME，并在原 DiagnosticBefore 5 秒由原 worker 核原失败和准确 claim/N/L/K。齐备才做 10 秒 Explicit 和 5 秒读回，证明唯一持久 V2 Intent/Pending 和独立字节读回。首错或未知保全并回交 Git。旧现场继续保全，固定签名候选前维持单 SID 定向验证。

## revision66 固定独立读回计时，继续 range-r1

接收 Windows 源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7、证据 f2dc1b60fa2215f3102347953f1d26cf363d9ff0，审查见 IMP205。按 [r7 增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 暂存指定观察脚本，Mac 并行推进后继取消和渠道名称验收。

> fetch docs/governance-main，按 TEAM66／IMP205／r7。在 marenben 普通 session 7 单跑固定 range-r1 Host，先独立确认同一存活 PID/FILETIME，再由原 worker 在 5 秒内核原失败和 claim/N/L/K。齐备后做 10 秒 Explicit、核唯一持久 V2、固定回执摘要，再用指定脚本计时 DiagnosticAfter，要求实际独立字节读回和语义核验在原 5 秒内。回交 Git 原始证据、计时绑定及首错。保全旧现场，候选冻结前保持单 SID 定向范围。

## revision67 接收 range-r1 观察入口暂存

接收 44502ee7c2d3038e37a313c86378c1724378c004 和 b12b85232221cfdac3d8c0042a4131ee05e3b47b，复核见 IMP206。沿 TEAM66／r7 的固定制品、原预算、普通桌面和独立取证继续。消费已有 Host 回执，out01 仍空时只跑一次固定 Host。将已核验 Host 回执的 SHA-256 传给独立进程观察入口，随后由原 worker 在 5 秒内证明原失败和 claim/N/L/K。齐备后做一次 10 秒 Explicit，绑定唯一 V2 回执，再由指定路径的计时脚本核独立字节与语义读回，原始 QPC 差严格小于五倍频率。保全首错与旧现场，通过 Git 回交。签名候选冻结前维持单 SID 定向范围。

## revision68 交接 Windows 回读脚本配对与现场验证

基线 Windows 源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7，证据 896bbd71105a08562943d56d39230db368486d7e。按 [r7 配对增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 由原 Windows 对话独占落两份脚本并复核实际暂存文件，接收结果归 IMP207。Host 和原 worker 的只读诊断可以先行。Mac 并行处理后继代次取消及选模验收。

> fetch docs/governance-main，按 TEAM68／IMP207／r7 固定并推送 launcher 与观察脚本配对。消费已有 range-r1 Host 回执，空时才在 marenben 普通 session 7 单跑一次。独立核存活 PID/FILETIME，原 worker 在 5 秒内核原失败与 claim/N/L/K。配对、ACL 和两项前置全部确认后才做一次 10 秒 Explicit 和一次严格小于 5 秒的 DiagnosticAfter，绑定唯一持久 V2、原字节和语义。保全首错与旧现场，通过 Git 回交；候选冻结前维持单 SID 定向范围。

## revision69 接收暂存预检并继续 range-r1 现场验证

Windows 源码 a8d7a08002c3af21b2c7ad8f9bcc06cda9d107d0、证据 bd29ce9c75a259aa845d153f732bf3e8ddd2cccc，实装脚本配对、ACL 与原 CMD 的审查见 IMP208。保全旧官方 r2 和原始失败证据，按 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 继续现场。公共 Start 维持拒绝，W2 和双 SID 留待固定签名候选。

> fetch docs/governance-main，按 TEAM69／IMP208／r7。使用 marenben 普通账户、SID 后缀 -1003、session 7、WinSta0/Default、无外层 Job。消费 range-r1/out01 内有效 Host 回执，空时才跑一次固定 Host。独立证实同一存活 PID/FILETIME，再由原 worker 在原 5 秒内通过 DiagnosticBefore 证实原失败和 claim/N/L/K。两项齐备后只做一次 10 秒 Explicit，再用固定配对执行 DiagnosticAfter，核唯一持久 V2 Intent/Pending 和独立逐字节及语义读回，原始 QPC 差严格小于五倍频率。保全首错和旧现场并通过 Git 回交。


## revision70 接收 Host 回执，续做独立活进程与原线程验证

按 IMP213 接收 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756 与 b130bf93eaefb7f7a05494fa1f4571104b3958b3。固定现场、脚本配对与排他写集仍沿 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。Mac 并行处理已登记取消的显式续接与隔离选模验证。

> fetch docs/governance-main，按 TEAM70／IMP213／r7 消费 range-r1 已有 Host 回执。保留 marenben 普通 session 7 原现场，独立核存活 PID 25144／FILETIME 134395302283231479，再由原 worker 在 5 秒内通过 DiagnosticBefore 核原失败及 claim/N/L/K。两项、固定脚本配对及 ACL 均通过后，跑一次 10 秒 Explicit 和一次严格小于 5 秒的独立 DiagnosticAfter，核唯一持久 V2 Intent/Pending、原字节和语义。首错或未知即停，保全本轮及旧 r2 并推送原证据。维持公开 Start 拒绝，候选固定前维持单 SID 定向范围。

## revision71 同一宿主的定向 Explicit 与独立读回

按 IMP214 接收源码 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756、证据 d88b1839e456751268ae2bf741cca64e665c548e。范围和排他写集沿 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。Mac 并行推进已登记取消的显式续接。

> fetch docs/governance-main，按 TEAM71／IMP214／r7 接续 range-r1。先找已有 Explicit 和 DiagnosticAfter 回执，已执行就消费结果。未执行时，现场确认 marenben 普通 session 7、PID 25144／FILETIME 134395302283231479 与原 worker ThreadId(51) 仍连续，复核原件、固定配对与 ACL 仍符合合同。齐备后只做一次 10 秒 Explicit，绑定唯一持久 V2 Intent/Pending，固定原始 Explicit 回执 SHA-256。随后只做一次独立 DiagnosticAfter，将实际逐字节及语义读回纳入 QPC 区间，差值严格小于五倍频率。首错或未知即停并回交 Git 原始证据，保全首失败与独立旧 r2。保持公共 Start 拒绝，不另起 Host／Start，固定签名候选前不做 W2／双 SID，不发布。


## revision72 单次 Explicit 超时后保全现场与取证

按 IMP215 接收 85f317434aab0663c5d282ee5edb0bdfb4532f0a。本轮 Windows 仅在原普通账户 session 7 读取既有私有日志，Git 写集为新证据目录与 windows-dev-handoff.md。Mac 并行推进已登记取消的显式续接。

> fetch docs/governance-main，按 TEAM72／IMP215／[r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 取证。保持 range-r1 原普通 SID/session，在原 PID 25144／FILETIME 134395302283231479、worker ThreadId(51) 的上下文只读快照既有 before-resume-private-control-v2-events.jsonl。回交采集前后文件身份和范围、原字节长度及摘要、首错，以及能绑定到同次 Explicit 的最末阶段。未知保持 Unknown。证据脱敏后提交新目录和 handoff，保全原件与旧失败。不重试 Explicit、DiagnosticAfter、Host 或 Start，不改权限、不强清、不启动 W2 或双 SID、不发布。


## revision73 原桌面稳定前缀与只读内存投影

按 IMP216 接收 832378a92959783c11d99dcad4d4aef1a2722d2a 的部分取证。Windows 只在原桌面读取日志并提交脱敏回执，Mac 继续原件恢复接线。

> fetch docs/governance-main，按 TEAM73／IMP216／[r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 接续。仅以原普通 SID/session 7 复核原 PID 25144／FILETIME、日志身份和保护，双读已证实的 5,622 字节前缀，用内存解析导出可绑定 ThreadId(51) 同次 Explicit 的白名单事件。无法绑定保持 Unknown。新证据和 handoff 走 Git，原失败及私有日志保留。不发新动作、不改权限、不强清、不跑 W2 或双 SID、不发布。


## revision74 接收投影后 Windows 待命

按 IMP217 消费 ba54ebf76f7fff7a438606a329505da237118778，Windows 保全原现场和既有失败，保持待命。Mac 继续按[后继代次准备取消合同](0.2.24-contracts/client-successor-prepare-cancel-r1.md)推进显式续接。下一次 Windows 定向诊断的环境与计量约束见 [r7 执行增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。


## TEAM75 Windows 定向计量与 Mac 并行

按 IMP218 消费最新 Windows 回交，Windows 从固定 Windows 源码准备一组新隔离配对，仅运行单次首次 Resume 前恢复定向诊断。Mac 同步推进后继准备取消显式续接。源码、写集、原期限及回交见[定向诊断合同](0.2.24-contracts/windows-preresume-recovery-timing-r1.md)。

## TEAM76 Windows 单轮真实桌面诊断与 Mac 并行

按 IMP219 接收 Windows 代码和定向证据。Windows 立即按[定向诊断合同 revision 2](0.2.24-contracts/windows-preresume-recovery-timing-r1.md)固定新隔离配对并补齐 V2 内部计量，复核后只执行一次新桌面序列。Mac 独立推进后继准备取消续接。真实运行时最先耗尽期限的阶段待取证。

## TEAM77 Windows 推送后待命，Mac 接续

依用户最新安排，Windows 保留已推送的源码、证据和原现场，等待 Mac 的下一次 Git 交接。此条替代 TEAM76 的立即执行安排。[定向诊断合同 revision 2](0.2.24-contracts/windows-preresume-recovery-timing-r1.md)留作后续明确下发时的执行依据。当前不新建诊断配对，不重跑原失败，不运行 W2 或双 SID。

Mac 核验后继代次的正常 Stop，固定下一交接的源码与证据。见 IMP220。

Mac 第二代真实 Stop 超时且终态降级，见 IMP221。Windows 延续待命，待 Mac 固定后继输入并明确 Git 交接。

Mac 后继的 ARM64 Release 原生宿主双代启停通过，见 IMP222。打包 GUI 与原生宿主证据分别记录。

## TEAM78 扩展落点归还自管根，Mac 收口待 Windows 真机

所有权收缩后仍留下一条会写真实用户配置的链路：拓展页把 Skill 与 MCP 分发到六个客户端，Codex 一格拼的是真实 ~/.codex。这同时违反「真实 .codex 只读」，也写不到运行中服务读取的 CODEX_HOME。代码 8b450a3fe 把 Codex 落点改到 Desktop 自管的 <OPENCODEX_HOME>/codex-client，其余五个客户端保持各自用户级配置，并把目标矩阵收敛为单一入口。边界、额外收紧与验证见[扩展落点边界合同](0.2.24-contracts/extension-client-home-boundary-r1.md)。

Windows 仍需按新目标矩阵重建候选并做真机启停与清理回交；Mac 侧真实官方 2.99 启停、DMG 候选和全量回归不变。

Windows 重建输入已固定为 main 81d823971，新增两条硬约束（Codex 落点是自管根；共享 %TEMP% 下不绑定自管根），写集与回交项见 [Windows 重建输入](0.2.24-contracts/windows-rebuild-on-extension-boundary-r1.md)。
