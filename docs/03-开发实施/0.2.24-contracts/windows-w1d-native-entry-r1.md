# W1-C接收、脚本准入修复与W1-D原生入口合同

revision: 1；2026-11-13；IMP-34 P24-03/W1。代码交付e004259feba4a077375db01a79f851470ed0ff74、证据25b32380106ecd2b6423b431a6284ef3ad8094b7；共同输入83b27e9ae/6ee2d2103、TEAM revision7。**W1-C未全通过**。本机接收后的代码main0d231aebc，完整源码及证据摘要见.adg/evidence/0.2.24/windows-w1c-receive-r1/。

## 全部结果与证据边界

| 项目 | 原回交与本机复核 | 未完 |
| --- | --- | --- |
| MSVC产品 | 真实MSVC14.44/SDK10.0.26100/rustc1.100，dev check/build bins+lib及测试no-run通过，精确exe与执行输入SHA归档 | 非release/固定候选，产品38/39及测试41条warnings保留，不冒严格Clippy。 |
| PrivateRecord修复 | 16MiB调用超过W0原1MiB上限，现按1MiB读并在publication前约束完整事务/后像；单项原生回归通过 | 未放宽W0，不修改Mac记录上限。 |
| PATH/恢复 | 真实隔离普通账户PATH启用、幂等、停用、后来条目保留、10格新进程继续、4类外部变化拒绝、通知中断保留及最终原值还原 | 单账户逐轮，不是双SID/W2；通知失败是明确注入，不冒OS真实广播故障。 |
| cmd | 真实产品ocxd→Node合成9.0.0脚本的完整混合参数、二进制stdin/stdout/stderr、exit23及OS Profile/自定义H通过 | 不用合成入口替代官方2.99。 |
| PowerShell | 默认.cmd混合参数exit1，stderr“| was unexpected at this time.”，无stdout | 仍失败，不能用cmd通过覆盖。 |
| 官方2.99 | 供应被runtime_selection_invalid“所选运行程序不可执行”拒绝，源码确认.mjs被传给Windows可执行扩展名检查 | 两条官方--version/config get尚未通过；不能说配置或原包损坏。 |
| NSIS hook | 真正PREUNINSTALL宏的无删除原生探针4格：未启用、pendingEnable拒绝保留、升级保留、非升级撤销 | 非完整卸载。 |
| MSI | 真实dev产品打包MSI/NSIS exit0；MSI类型1058/Impersonate/Return=check、3499<RemoveFiles3500；5格原生条件通过 | 尚未执行真实MSI卸载及失败保留，不能用表检查替代。 |
| 清理 | 本轮账户/Profile均缺席、隔离账户PATH原值还原、临时桌面ACL还原 | 5个生成根共26869205653字节仍在，自动审批blocked by policy未执行/未绕过；旧约2GiB和旧失败目录不改写。 |

265份贡献文件共1960818字节按Git blob逐字复制，原native raw-manifest139份逐一SHA/长度核验；10份最终源绑定7份逐字一致，3份（模板/既有Python）仅CRLF-LF差异，两摘要保留。out04原产品内容与最终产品修改一致；最终native测试另加PrivateRecord回归与成功官方结尾不吞argv失败断言，执行版另存，不声称执行过未走到的官方成功分支。

原测试exe产品本体/包仍在Windows，本机只核原摘要记录与日志/输入一致，未在Mac下载重算。账户Profile私有失败快照仅为隔离账户材料，无真实业务Key。原失败和首错/工具链环境缺件均原样保留。接收按两笔cherry-pick为abaa93ce2、b9742198b，不merge整条历史或带入源码之外生成物；导入仅约定证据与handoff，保留本机并行原型。

## 主线已修：解释器脚本角色

main7cf337d5a：program_selection的OS executable gate仅对Node/Bun。官方bin/ocx.mjs作为已准入Node的参数，按原受管版本边界/无链接普通文件句柄/单链接/大小上限/独立bin_sha256准入；入口扩展名或Unix x位不再冒充脚本内容证明。完整tree/current/清单/package.bin与所有关键文件摘要门保持，末段再核脚本摘要/清单，不把路径发现的任意脚本当合法。

新增2项反例集：600模式且原内容/树完整的.mjs正常准入，Node失去x位仍拒绝；脚本符号链接/硬链接/目录/缺失/内容换代全部拒绝。13项selection、21项terminal（2原ignored保持）、2项新原生转交host测试及all-targets严格Clippy/fmt通过。未在Mac宣称Windows官方命令已通过。

## W1-D采用的入口：原生ocx.exe

不继续尝试对.cmd多层转义打补丁。已准备main0d231aebc的src/bin/ocx.rs及Cargo bin：一个独立std控制台程序，**位置固定为实际安装I/ocx.exe**，直接调用同目录I/ocxd.exe --ocxd-runtime-cli并以OsString逐个传参、继承stdin/stdout/stderr/cwd/OS环境、等待原退出码。不运行cmd、PowerShell、GUI、PATH查找或自行读配置。ocxd仍是正式配置/选择/登记准入与官方Node的唯一转交者。未改官方包，未改变Mac ~/.local/bin/ocx。

本机2项仅证明Rust Command参数不重拼Shell、host子进程字节流及exit23；Windows target rustc metadata typecheck通过，未链接运行Windows程序，不当PowerShell验证。**当前产品终端事务仍为.cmd schema2，尚未切换新入口。** W1-D应把下列工作一次完成后验证，不能只运行未接线的新bin宣称解决。

1. 原生ocx.exe随MSI/NSIS作为安装器拥有的程序组件放在I，与ocxd.exe同目录；取实际I，不写死盘符或默认安装位置。实际打包清单要包含Desktop、ocxd、ocx，缺任何输入在构建前拒绝。无需新增helper复制到用户目录或命令启用时复制整份4.5MiB ocxd。
2. terminal_cli::Context的Windows命令/cli_entry为I/ocx.exe；用户PATH增量为I，只有显式启用才登记。原生shim为安装器拥有的现存程序，**启用/停用不创建、覆盖或删除该exe**。停用只撤销本应用新增PATH和私有终端登记；相同I已有等价PATH不取得删除权，外部变化拒绝/保留规则不变。与其它ocx同名命令仍冲突拒绝；不能为了通过裸命令解析抢覆盖PATH优先级。
3. Windows终端记录使用明确schema3并绑定当前Profile/D/H/I/manager与命令路径/entryKind。不可把开发schema2原脚本字节或任意exe内容当可执行授权；不开发schema2迁移/兼容，旧开发现场由原版本停用或精确保留失败后用新隔离账户验证。Mac schema1不变。新pending只含PATH/登记before-after和命令/console同次准入事实，不把整exe内容塞进1MiB私有JSON。每次预览及实际边界核对命令真实普通文件、单链接、无reparse、来源/身份/摘要与保留句柄；退出/未知保护不放宽。
4. 正常升级可替换安装器拥有的二进制而保留当前用户终端登记与PATH，不能把旧版本内容哈希硬编码成永久阻止新包的身份。新动作仍按当前明确安装来源核验；未知exe/根变化拒绝，别修改官方运行包的current/清单以登记Desktop shim。
5. 新shim一直存在不表示已启用：ocxd仍拒绝无登记/pending/位置变化；从未打开的卸载只读检查固定anchor与终端登记明确缺席即可成功，不创建数据，不因安装器预置ocx.exe而误认为未知用户脚本。数据卸载原“先撤销终端归属”的门保持。hook/新helper仍非升级先撤销、非0保留程序、升级保留；原固定程序生命周期卸载的Mac-only门不能顺手放开。

## 本批写集和定向执行

用户要求遇Windows工作停下给提示词，Mac在交付main0d231aebc后停止写本批范围。W1-D接原W1-C独占文件及临时共享CLI写集：terminal_cli.rs、terminal_cli_windows.rs、terminal_cli_path.rs、两份terminal_cli_windows_*tests.rs、commands/terminal_cli.rs、bin/ocxd.rs、新bin/ocx.rs；新增改program_layout.rs的Windows cli_entry及相应纯路径测试；Cargo.toml仅native ocx bin装配；test/windows024-native/**、windows三安装器文件。既有infrastructure/windows_cli024.rs/windows_official_runtime024/**可为受影响原生补验修复，不抢其它共享状态机/Vue/版本清单/工作流。共享program_selection修复已由Mac完成，若有新实际问题只给最小原因/证据，勿再放宽。

先承接最新main/文档，不覆盖本地工作。在专用普通测试账户验证：新终端裸ocx的真实解析目标（Get-Command/where与实际运行）、cmd及默认PowerShell的特殊/空/引号/Unicode参数逐项比较、二进制stdin/stdout/stderr/exit23；如PowerShell自身在到达exe之前丢参数，应分别记录版本/argument-passing模式和源解析，不能改默认模式/删断言假通过。原官方2.99 --version及自定义H config get port --json、Node/script/current精确来源与配置字节不变，禁止改官方脚本扩展名/内容或用合成版本代替。

对新schema3只跑受影响启用/停用/原值还原/外部编辑拒绝/pending新进程继续/升级保留与helper门，不重跑不受影响W0/旧r5。对真实本批dev MSI至少做一次专用测试身份/根的非升级卸载，核正常撤销及失败保留（错误阶段/exit/实际余留），不能仅重复MsiEvaluateCondition；NSIS已有宏证据按新入口受影响部分增量验证。全程不是W2/双SID/全量sweep，不发布、不改日常PATH/配置、不跑业务同步/模型请求。

先写本轮要证伪的假设，一轮记录所有独立项状态/首错/完整原输出；任何失败保持allAccepted=false，成功不覆盖原out04。复用已保留工具链和已核验官方原包/构建cache，减少重复大包副本；新证据先核验归档再清测试账户/Profile/原值与本轮精确残留。先前自动审批拒绝的清理保持未执行，不换工具绕过；未清大小另报。不复跑相同源相同假设，只按新事实修实际缺陷。分批及时推送代码及docs/024-windows-dev的新不可覆盖目录与原handoff，回SHA/状态表/首错/清理/未完。

这批只推进终端和官方脚本消费者。Windows生产生命周期/恢复完整接线、固定候选/W2、根未移中断/创建取消、一般客户端图/进程集合/刷新、NAME、复杂代理和官方交付门等IMP-34原范围不缩减，未发布。
