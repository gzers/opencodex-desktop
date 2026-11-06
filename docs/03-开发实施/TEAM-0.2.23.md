# 0.2.23 多对话团队协作文件

2026-11-06，Asia/Shanghai。**团队已进入开发与分批集成，Windows按唯一任务入口执行R2补验。** 前序R0/G0段落作为过程来源保留；本文拥有责任边界、启动条件和回交合同，不复制需求验收，不维护第二份任务状态。

执行阶段/整版顺序唯一归[IMP-31](IMP-OPENCODEX-DESKTOP-31.md#3-实施顺序与并行边界)，目录及Windows细目归[IMP-32](IMP-OPENCODEX-DESKTOP-32.md)，模版细目归[IMP-33](IMP-OPENCODEX-DESKTOP-33.md)。范围/验收归[DMD-POST022](../01-需求管理/需求/DMD-OPENCODEX-POST022.md)、[DMD-USER-DIRECTORY](../01-需求管理/需求/DMD-OPENCODEX-USER-DIRECTORY.md)、[DMD-TEMPLATE-MATCHING](../01-需求管理/需求/DMD-OPENCODEX-TEMPLATE-MATCHING.md)。本文件不授权发布、真实同步或修改日常配置。

## 1. 可行结论：一主四贡献，分阶段并行

建议共5个对话：本对话为集成主线，另开模版后端、同步后端、前端与原型、Windows适配4线。按领域而非按单个页面拆分；共享逻辑不按Mac/Windows复制两套。

- 适合并行：冻结合同后的匹配引擎和远端状态机、采用原型/接口确定后的UI、Windows专属适配与本机共享核心。
- 不适合独立决定：Channel/Model/Template Schema、协议来源、程序/用户根合同、WebDAV载荷版本、新运行包来源和公共组件。G0先收敛一份，由主线组织冻结。
- 不再细拆：新增/编辑/复制渠道合为前端一线；目录来源与安装恢复由主线统一；运行包名称投影也归主线，不另建运行包/发版线。
- 不是5线同时编码：G0可以并行只读分析/提方案，随后按各自依赖放行。等待接口时只继续本线不依赖项，不以假接口宣称实际链路通过。

## 2. 角色与唯一主责

| 角色 | 建议对话名/环境 | 对应清单 | 负责产物 | 启动前提 |
| --- | --- | --- | --- | --- |
| L0 集成主线 | 0.2.23 总控与目录运行包 / 本机 | G0共同冻结、DIR共享部分、P23-03/05、P23-08/09 | 共同合同、目录/程序解析、macOS适配、安装恢复核心、新维护运行包/名称投影、合并回写与交付 | 未闭合G0先完成设计，发布另依有效授权。 |
| L1 模版后端 | 0.2.23 模版匹配与模型协议 / 本机 | MATCH-01～05后端、AC-MATCH-01～07 | 族/版本/名称条件、来源/人工覆盖、读取自动匹配草稿、模型协议/有效配置、保存新revision、模版便携对象编码 | G0模版/来源/协议合同；实际文件接线依新目录基础。 |
| L2 同步后端 | 0.2.23 WebDAV文件协议 / 本机 | P23-00/06、AC-P22-05/06 | 无ETag文件/提交/代次、并发冲突保全、原操作核对、持久状态/回执/错误、唯一远端传输管道 | G0协议与L1载荷/依赖合同、本机路径接口；不再造模版解析器。 |
| L3 前端与原型 | 0.2.23 原型还原与业务界面 / 本机 | P23-02/04/07、DIR界面、MATCH界面、恢复UI | 总原型本版必要增量、条件覆盖/目录/恢复、无渠道协议向导、读取自动匹配/保存、真实组件页面验证 | 布局按已采用原型可先做；实际接线等对应字段/接口，不整体采用并行概览/对话探索。 |
| L4 Windows适配 | 0.2.23 Windows用户隔离与安装 / Windows | WIN23-00～04、DIR平台部分 | SID/Profile/ACL/凭据/锁/IPC、ocx.cmd/进程/端口、NSIS/MSI运行包/更新/卸载、实际两SID证明 | 平台核对可G0先行，适配编码等合同与共享接口，原生验收等可运行集成来源。 |

每线先自验并及时回交，L0核对实际写集/依赖/证据和最终合并影响。贡献通过只标待集成；验收与发布沿真实IMP/TASK/RCP，本文件不另打勾。

## 3. 写入边界与热点文件

以下路径相对各自代码checkout，为拟定写集，实际派发前按最终基线核验。未分配路径默认只读；新增消费者/跨域变更先报L0，不能自行扩大通配范围。**每个文件同一时间只有一个写入负责人。**

| 角色 | 可分配写集 | 不得直接修改 |
| --- | --- | --- |
| L0 | apps/desktop/tauri/src/modules/data_root、runtime、ipc、instance的共享实现；commands/data_root.rs、runtime.rs、upgrade.rs；model_config中的channel_rename.rs、rename_effects.rs、config_file*.rs、config_file_mapping.rs、execution_directory.rs、saved_runtime_reconcile.rs等逐项分配文件；运行包/发行工具。 | 其它贡献线进行中独占文件；合同变化先暂停相关项、明确转移归属。 |
| L1 | modules/model_config下schema.rs、matching.rs、compiler.rs、store.rs、seed-catalog.json、provider_import.rs、upstream.rs、discovery.rs、discovery_config.rs、protocol_groups.rs、portable_channels.rs、agent.rs、agent_conversation.rs、broker.rs及本线独立测试。 | config_file*.rs/名称事务、远端状态机、公共命令入口/模块注册、Vue/TS、日常模版库。与L0保存服务先约定接口。 |
| L2 | modules/model_config下config_remote.rs、sync_state.rs、remote.rs、transfer*.rs、file_objects*.rs、file_exports.rs、content_versions.rs、sync_version_tests.rs及modules/sync；本线独立协议测试。 | L1的schema.rs、portable_channels.rs及模版解析；公共命令入口、Vue/TS、真实WebDAV配置/文件。L1提供载荷对象，L2负责传输。 |
| L3 | apps/desktop/ui/src/features/models、features/sync；现存模版编辑器routes/ModelsRoute.vue、目录/恢复相关routes/SettingsRoute.vue、features/data-root/api.ts与store.ts、features/runtime/api.ts及其RuntimeInstallDialog.vue、必要features/backup/BackupFiles.vue按最终合同逐文件授权，另含独立UI测试。总原型本版模型/目录/恢复资源在文档贡献分支单写。 | Rust、公共组件/令牌/启动入口、版本文件、正式DMD/IMP/核心/索引、并行概览/对话/额度原型。 |
| L4 | infrastructure/windows_startup_root.rs、windows_restart.rs/.ps1、必要windows_query_job.rs/fixture、apps/desktop/tauri/windows/installer-hooks.nsh及独立Windows测试。实际分配按diff缩小。 | 共享data_root/IPC/instance、Schema、全平台runtime核心、主入口/构建版本、发行工作流、前端。跨入L0文件先回交具体补丁需求，由L0接入或转移单文件负责人。 |

### L0保留的共同入口

apps/desktop/tauri/src/lib.rs、state.rs、commands/mod.rs、commands/model_config.rs、model_transfer.rs、sync.rs、modules/model_config/mod.rs、tests.rs、全局错误注册、Tauri权限/配置、Cargo/npm清单与锁文件、.github、版本/发布说明和全分支共享文件均由L0单写。路径在最终基线不存在时记录实际入口，不创建同名第二实现。

L1/L2需要注册模块/命令时，回交函数签名、参数/错误和最小注册补丁建议，L0及时接线并发布共同基线，不能让贡献线长期只能用桩验证。src/stores/app.ts及前端启动/共享应用门面由L0单写；L3只回交249/384等实际调用点适配需求，不自行扩写。L3单写已分配的前端api.ts/store.ts等消费者；L0记录共同接口合同，L1单写生产模型Schema，双方不分别发明取值。

原型已有共享路径保护。**真正并行派发前**，L0按治理流程将当前DMD、根层IMP-31/32/33、TEAM、候选配置与总原型入口纳入有效共享路径/所有权计划，核对policy revision/digest；仅本版批准的原型文件授予L3独占权，其余只读。不手写残缺治理配置，也不声称本文件就是已生效机器锁。正式文档只由L0回写，不由贡献线同时改。

## 4. 放行点和合并合同

阶段顺序由IMP-31 A～F拥有，下表只说明角色启动前提。

| 放行点 | 可开展的贡献工作 | L0提供的前提 |
| --- | --- | --- |
| R0 交接准备 | L1/L2/L3/L4并行只读分析、接口/原型建议，不改共享文件。 | 可取得的当前资料、范围和拟定文件归属；未跟踪资料先显式纳入后才能跨端交接。 |
| R1 G0合同冻结 | L1纯匹配/协议核心，L2协议/故障夹具，L3采用原型布局，L4平台适配分批启动。 | 真实代码SHA、路径/来源/协议/载荷Schema、采用原型SHA/摘要、角色写集及必需检查；未冻结的相关实现仍等候。 |
| R2 共享基础可运行 | L1真实文件/运行接线，L2持久传输，L3实际API，L4启动/安装/两用户链。 | 新目录/入口与公共注册已进入共同基线；每线更新至明确SHA再验，不能混用旧接口。 |
| R3 逐项回交 | 每线达到可验项就提交/回交，L0逐项集成，不等四线全部结束才第一次合并。 | 实际写集/基线/证据审查和合并影响复验，明确依赖更新，不用“main最新”替代来源身份。 |
| R4 最终集成候选 | 本机/Windows只复验合并影响和必需原生/制品项。 | 同一代码树、运行包摘要、候选配置、采用原型；L0唯一维护P23-08/09和最终裁决。 |

L0先完成目录共享基础，匹配/同步/页面/平台按对应接口交错并行，不必等全部基础验收才开始纯逻辑。运行包名称投影使用L0单一维护来源，Windows验证同一包，不制作另一份同版本不同字节的包。

贡献线只合并L0公布的基线，不互相合入未集成分支，不直接推main、不force push或移动tag。L0按仓库历史选择merge/cherry-pick并保留来源；冲突由文件负责人解释语义，禁止简单ours/theirs隐藏接口差异。合同改变要发布新摘要，相关线更新并回验，旧证据注明实际失效范围。

## 5. 工作区、分支与资料交接

只读参考代码checkout：/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes，已知0.2.22基线0c57500f8f46ea50655c782c3f04334a39167abb。文档checkout：/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main，分支docs/governance-main。43c4068b5只是本轮调整前的旧文档HEAD，不包含当前团队/候选资料；本批已按用户指令准备提交推送，**实际接收以本次推送的docs/governance-main完整SHA及文件摘要为准，不用旧SHA启动R1**。派发前核对远端及本地，提供真实文档提交/文件摘要；不拉取覆盖并行用户修改。

建议分支：L0=codex/0.2.23-integration，L1=codex/0.2.23-model-matching，L2=codex/0.2.23-webdav，L3=codex/0.2.23-ui，L4=codex/0.2.23-windows。名称仅建议，尚未创建。优先复用适合的独立checkout，否则由宿主工作树工具建立；多对话不能共用可写目录/索引，Windows本机也同样遵守。

代码只在实现分支，docs及.adg只在文档分支。L3原型必须用独立文档贡献checkout，不把原型放入代码提交，也不直接并发写当前dirty文档树。其它线的证据经受控产物/各自文档贡献分支回交，路径为.adg/evidence/0.2.23/team/<L1或L2或L3或L4>/<批次>/，每批次唯一、不覆盖旧回执。只有L0改正式清单/核心/索引。

### 启动提示词骨架

派发时必须补代码SHA、文档SHA、合同/写集版本和实际工作区；未填不能当作直接执行包。用户已自行开启L1/L2/L3对话；本次不代创建新对话。

- **L1**：按TEAM的L1范围及IMP-33开发匹配/协议/读取自动草稿，基于指定代码/文档SHA。先核G0/写集，只改本线文件，公共注册向L0交需求。保留人工覆盖/原始ID，19未匹配不补齐；不改日常配置、不推理、不发布。逐项回交提交/检查/未完。
- **L2**：按TEAM的L2范围及P23-00/06实现已冻结无ETag协议/代次/并发保全/恢复，只用隔离合成服务，不真实同步或清空远端。载荷用共同合同，不改模版Schema/前端/公共入口。回交并发、失败恢复和清理证据。
- **L3**：按TEAM的L3范围完成本版已采用总原型及正式组件，不整体采用并行探索。条件覆盖、无渠道协议、读取自动匹配、目录/恢复/同步按合同；代码/原型分支分开，公共组件先交需求。验证保存/取消/失败/重开/窄窗后回交。
- **L4**：按TEAM及WIN23-00～04处理Windows用户隔离/目录/IPC/入口/安装器；共享核心只提具体需求，不重做匹配/同步/UI。验证两真实SID并行、同SID单实例、独立端口/凭据/进程归属和受影响安装更新卸载。隔离数据、保留失败/清理证据，不发布。

## 6. 回交和最终收口

每线回交短说明及可取得的代码/证据，至少包括：

~~~text
角色、清单编号、实际完成与未完
代码基线SHA、当前提交SHA、文档/原型/合同摘要
工作区、分支、实际改动文件列表
命令、检查数量/结果、失败/跳过/环境限制
实际制品/运行包摘要（涉及才提供）
截图/日志/脚本的可取得路径和摘要
临时文件/进程/端口清理结果
公共入口/跨写集需求、剩余依赖与下一动作
~~~

Mac绝对临时路径不能当Windows交付位置。通过已授权贡献分支或私有制品交付实际位置/摘要，不传Key、认证Header、真实配置或整包.adg；未上传明确“本地未同步”。口头完成不等于收到代码。

核对顺序为写集→基线/合同→证据可读取且匹配源码→首条代表流程按原型→合并树复验。两同用户RDP会话不当两SID；名称匹配不当能力/协议证明；失败/缺环境原样保留，重跑成功不删除原失败。

L0在全部适用AC、贡献检查与必需Windows边界通过后完成P23-08；发布由P23-09及有效授权决定。用户已自行启动前三个贡献对话，当前仅允许R0/G0只读核对；L3已正式回交G0缺口、接口需求、拟定写集和验证建议，并确认未建工作树、未改共享文件、未编码或执行测试，详见[回交摘要](../../.adg/evidence/0.2.23/team/L3/g0-20261106/receipt.md)。L1/L2具体回交以各线真实结果为准，不从用户已开对话推定完成。尚未发布R1冻结接口/代码基线或独占写集，L4未由本线启动；没有代创建工作树或执行任务，没有修改产品代码。

L2正式G0回交已接收并按原SHA256归档，见[完整方案](../../.adg/evidence/0.2.23/team/L2/g0-20261106/L2-G0-20261106.md)和[回交凭据](../../.adg/evidence/0.2.23/team/L2/g0-20261106/receipt.json)。这是待冻结建议，非编码/测试结果；初始化歧义恢复与服务器并发能力边界由L0结合共同合同处理。L2申请webdav_client.rs的Depth1/安全href/错误适配不在原写集，L0接入或明确单文件授权后才可改。R1尚未发布，当前L2/L3继续R0。

用户已授权总控目标模式文档/开发/测试/发布及直接协调既有L1/L2/L3，Windows走独立Git交接。当前L0已创建隔离集成工作区及干净原型贡献工作区，G0具体合同产物由各线回交，下一步按冻结合同逐项发R1写集；旧“只读/无开发授权”仅是前序状态，不取消本轮有效授权。共享路径策略已纳入根层IMP/TEAM/合同/候选，真实revision以提交文件为准。


### 2026-11-06 第二批实际派发与集成

用户完整授权持续有效。集成代码树为codex/0.2.23-integration，当前已集成L1两个贡献fa07451c9/4b34851dd及L2首批772e6424d，集成提交9e8fded1b。公共注册及生产网络适配正在正常crate复验，尚未完成保存/界面/安装/发行验收。

L1新增实际消费者批次独占：provider_import.rs、protocol_groups.rs仅新增v2入口，compiler_v2.rs、store_v2.rs、consumer_v2_tests.rs；原六个v2纯核心文件继续由L1持有。L0保留config_file、名称/配置统一事务、公共命令/注册。L1库/草稿可独立保存，Policy/ledger/映射只返回统一事务材料，不分开落运行状态。所有root参数为解析后的D，配置H单独传入；不扫描旧路径。锁序：全局写门、content_versions文件保存门、渠道名称门、config文件锁、model-v2事务锁。

L0内部辅助执行者分别独占data_root/{mod,bootstrap,layout,startup_v2}.rs及commands/data_root.rs，和runtime/install.rs及commands/runtime.rs；仍由L0负责审核与提交，不建立新的跨对话版本来源。其他L0文件由总控写入。此辅助分解不扩大L1/L2/L3/Windows写集。

扩大目录回归首轮68项为39过/29失败，主要暴露旧自动修复/旧绑定/版本及路径断言与新合同冲突；保留失败结果并继续逐项处理，不以局部新测试通过替代全模块验收。


### Windows R1 Git放行补记

代码codex/0.2.23-integration@9e69f4598512aac0c4e068b9f9f9c08ede906a28已真实推送；WINDOWS-0.2.23.md已由前序R0等待状态改为R1平台开发与隔离验证。新增Windows SID/Profile/ACL helper及endpoint.rs的Windows适配逐项授权见该唯一入口，L0保留platform.rs/公共注册。后继完整离线运行包/制品验收仍须另给明确摘要；当前不以版本显示或本机未提交文件作为Windows基线。


### 2026-11-07 Windows R2 真实推送交接

Windows R1代码及证据已集成；此前入口未随代码批次更新，已更正唯一任务入口WINDOWS-0.2.23.md至R2。当前已推送代码基线33d2c65b58ba8880d6deea7b52d69117238c3d33，包含实际Profile/ACL及IPC服务SID接线。Windows现在可继续MSVC和真实SID/凭据/pipe/端口原生边界补验；完整安装包与公共材质补丁待L0后继交付，不能将缺失制品项打勾。每批代码与文档及时推送，并提供远端确认的SHA；本机路径或只commit不算跨设备交付。
