---
id: "IMP-OPENCODEX-DESKTOP-34"
object_kind: "implementation.change"
state: "in_progress"
completion_summary: null
title: "0.2.24 发布后修复与双线集成计划"
target_version: "0.2.24"
demand_ids: ["DMD-OPENCODEX-POST023"]
task_ids: []
summary: "0.2.24共享与Mac原生实现持续集成；Windows W0已接，W1及最终候选未完，完整剩余范围按实施表推进。"
---
# 0.2.24 发布后修复与双线集成计划

revision: 94；2026-11-13。用户要求本机与Windows可独立开发部分并行、Windows分批提交、最终再通过Git交接Windows验收。本文件为本版实际实施顺序/依赖的唯一来源；[TEAM](TEAM-0.2.24.md)只拥有执行角色/写集/回交，不另维护进度。范围及既有采用规则承接[开发前边界](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-后继修复开发前边界确认.md)和[同源矩阵](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-0.2.24开发前收口矩阵.md)。本对话已按用户授权执行开发/测试/集成，Windows W0已回交并注册；当前不授权发布或改日常配置。

初始代码集成基线main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；当前开发检查点main fed7d21ad（非冻结候选），发行基线v0.2.23 e3728109e93498bfbd6e993d49904d57dfd75315；目标0.2.24。两个基线不能互相冒用。后继实际开工时核对main新提交；发现新增共享变化由本对话承接。新版本范围已由DMD-POST023承接，本计划已进入in_progress；尚无新正式TASK或关闭证书，不挂旧已发行TASK，也不为排期篡改旧证书。

## 实施与并行顺序

| 编号 | 当前状态 | 工作与输出 | 完成标准 | 证据／未完 |
| --- | --- | --- | --- | --- |
| P24-00 | 开发中 | 本机固定唯一实际程序根、当前版本/安装清单、同HOME、跨平台调用与错误语义；复用2.99原包身份和原生生命周期预算。 | 共享Rust入口/类型由本机单写；Windows按真实代码接口接入，无双重路径猜测/自造JSON清单。 | main9125f7134/3d3c90943：启动供应、来源/依赖/真实磁盘版本已接，当前选择/内部入口与用户CLI分离；93项相关检查和真实Mac官方包供应/版本探测通过。更新切换/CLI/卸载合同未完，W1未完整放行。 |
| P24-W0 | 已接收并注册，产品接线/失败目录清理未完 | Windows将r5验证过的TokenUser/受保护记录创建/保留进程句柄/Job集合/正常退出/原生监听与路径事实整理为独立Rust平台后端，带同源原生定向测试。 | 真实Win32事实而非桩；未知拒绝；不要求旧维护包lifetime字段，不强杀补通过；生产模块可被独立测试宿主编译。 | 贡献b2e89665d/证据0eef59a01已逐项核对，主线2d68a28bc按净树集成注册；4个Windows失败目录仍在，未伪报清理或产品整链通过。r5输入保持。 |
| P24-01 | Mac保留配置的卸载/重装及原登记同版本修复已接；其它来源/跨平台原生与整包验收未完 | 本机实现POST-01/02程序/配置分离、Mac标准单.app、用户可写供应、安装/更新/回退同一来源；POST-07磁盘/运行/历史版本分开识别。 | 默认与自定义根、只读资源、来源漂移、校验失败保全；清单/切换/保护单一来源。 | 依P24-00；Windows W0并行，不等完整UI或名称路线。 |
| P24-02 | 共享保存/手动备份/恢复与现有安装事务保护已接；版本化升级接线及候选验收未完 | 本机POST-04保存重核、POST-08原生JSON单层导入/完整备份/回退，复用真实配置事务。 | 保留草稿/密钥意图、有限等价变化不误锁，真实冲突拒绝；导入仅保留本机port和模式，其它按源替换，备份失败零覆盖。 | 后端8例及原型已定；不把Read的模版匹配暗当无副作用刷新。 |
| P24-03 | Mac终端/通知/网络/内置准备及版本事实/更新前重核已接；Windows/PATH和原生候选未完 | 本机POST-03同版本同HOME CLI核心及Mac入口；POST-05环境阶段/就绪区分、POST-06应用网络/更新说明；已采用UI接线。 | CLI显式启停/冲突不覆盖；内置环境不依赖系统Node；npm/GitHub错误准确；原型一致。 | 03路径/CLI接线依01；网络/UI可先独立推进。 |
| P24-W1 | 原生PATH/安装器024资源部分已放行；完整共享接线未完 | Windows后端接共享调用合同；Windows当前用户CLI/PATH和自定义安装根/版本目录/卸载平台适配。 | 实际运行版本/HOME/安装根一致；用户级入口/PATH有归属与撤销，安装更新不误删配置或别用户对象。 | W0可先提交；共享mod/入口/版本/工作流由本机接线；不能先猜清单文件名再返工。 |
| P24-04 | Mac普通Stop及显式中断恢复已接；已有Codex但缺Grok/未用专用清单路径已核验；首次空配置及Grok备份可逆恢复已接；缺根目录归属/同锁登记与原缺席原件捕获已接，整根可逆归档/独立movement登记与缺根只读核验、真实Guard约束的整根读回事务已接，生产缺根启停/整根授权收口/一般范围/候选预算/缺Ready/Windows未完 | 本机接收W0/W1，核对来源与写集，接共享模块/命令/配置和生命周期/事务；Windows修本线集成编译问题。 | 最终合并树跑真实共享流程；分线通过不当集成通过；同30/5/10秒和未知拒绝保持。 | 小批次回交，不等Windows末轮才合并；保持开发重叠。 |
| P24-NAME | 剩余技术方案未定，局部等待 | 非直接合法运行键名称及多协议派生组名称展示/官方刷新保持；普通合法英文改名已有能力不重复开发。 | 原始模型/认证/能力保持，明确改名可改实际键及引用；正式显示出口、默认客户端/刷新/重启/回退有证据。 | 10组真实保存确认，中文等新增/改名均转内部键；不私改原样2.99、不用combo降能力/双写轮询。只阻塞依赖它的名称改动和整版最终验收。 |
| P24-05 | Mac真实024工程.app及正常首次供应已验证；旧工程DMG早于最新源码，非冻结候选，Windows/NAME/完整原生未完 | 本机完成共享逻辑/页面/原型对照、Mac真实原包与必要安装/CLI/升级回退；构建并固定Windows集成候选。 | 给精确代码SHA、官方包版本/SHA、安装器SHA、调用合同和一次完整受影响原生验收表。 | 共享页面不重复交Windows；Mac可在Windows开发期间做有用检查。 |
| P24-W2 | 最后交接，当前不跑 | 用户把本对话生成的最终简短提示词交Windows；针对固定集成候选，一轮采集所有受影响原生项状态/证据。 | 双SID真实桌面并行、受保护生命周期/恢复及原预算、路径/CLI/相关安装更新卸载；失败保留，未知不通过。 | 按最终diff选必需项。不是旧r5第六轮，也不在未集成源码上提前宣称产品通过。 |
| P24-06 | 待最终回交 | 本机复核Windows完整表，批量修全部可并行失败项，再做受影响增量验；名称未闭合仍不发版。 | 所有本版必需项有最终来源证据、清理、文档/原型对照和有效发布授权；无假放行。 | Windows成功结果不覆盖旧失败；测试来源变更先评估依赖，不一处失败一轮全跑。 |

关键路径：00→01→W1/03接线→04→05→W2→06。W0与00/01/02/网络UI同时进行；02不等待Windows，W0不等待Mac完整开发；NAME单独解决，未定前不阻塞独立文件的功能开发，但不能悄悄删出本版交付。

## 固定接口语义，避免互相等待

- 本机给Windows的程序根/稳定入口/实际版本/HOME是已解析的受控输入，Windows不重新扫描旧路径或全机寻找其它服务。
- Windows平台后端返回TokenUser SID、原始进程创建身份和保留句柄/Job、原生文件/权限/监听事实。记录序列化、官方health验签与来源准入、应用状态机和事务总预算由共享层持有；不要求Windows复制一套共同状态。
- Windows W0可先设计进程本地的Rust类型/函数并在本线独立宿主实测，第一批提交即给本机审查；这些类型不冒充已冻结的IPC/持久Schema。本机消费时统一转换，稳定后才接生产入口。
- 先共享接口提交后同步SHA放行W1。若接口仍缺，只暂停对应接线；Windows继续W0可独立工作，Mac继续02/网络/UI。涉及另一线独占文件只能给最小补丁建议，由文件责任方应用。

## 验证与交付节奏

开发期只跑与本批改动有关的编译、单元/组件和必要定向原生检查。r5单用户“技术可行性”证据复用；移入产品后实际身份/配置/持久事务与原预算仍要验。保留两稳定锁的有限例外，不扩大缓存/审计/备份残留范围。

最终Windows环境按用户采用优先真实控制台管理员一会话＋另一账户RDP会话，必须不同真实SID、同时Active、真实桌面；同SID多会话不能代替。托管runner可做编译/离线测试，不能把无桌面托盘Access denied计为产品通过。账号/桌面不满足时明确该项未执行，不以已有单用户证据代签。

最终表覆盖实际变化所触及的当前用户默认/自定义安装、相关NSIS/MSI合同、用户级CLI/终端/冲突/卸载、自有运行包供应/就绪与实际端口、路径/IPC/查询、正常Stop完整恢复、升级失败回退/安全清理；测试前声明要证伪的判断。每步独立记录pass/fail/blocked及首错，尽可能继续独立步骤，依赖不满足标blocked，不执行危险后继。诊断轮accepted:false；只有最终整组独立验收才可通过。拿完整表后一次处理所有可并行项，不回到一关一轮。

本对话负责本机开发与受影响测试及Git接收；Windows通过独占Git分支交回，当前不另建对话或替用户发送消息。版本发布另依有效授权。

## 2026-11-10执行授权与当前入口

用户已明确授权本对话按顺序进入目标模式，替代本文r1“仅编排/未开始生产开发”的当时状态。范围/验收已由新[DMD-POST023](../01-需求管理/需求/DMD-OPENCODEX-POST023.md)承接，旧版本需求/证书不改。P24-00已在main工作区开始：统一ProgramLocations只读解析/版本路径类型，Mac运行程序根改为当前用户Application Support，Windows按实际可执行文件父目录；阶段测试/源码提交后及时回写。本轮未创建/关闭正式TASK，不能宣称机器派发已存在；Windows是否接令以实际Git回交为准。


### P24-00第一批检查点

main 9f1ffe677已推送，共享程序位置与启动解析已落代码，[接口r1](0.2.24-contracts/program-layout-r1.md)及44项测试留证。Mac标准.app位置不再因其父目录是Applications被拒绝，程序根改为用户Application Support；Windows继续实际exe父目录。此批不是实际新包安装验收；版本激活/安装清单与剩余消费者尚未接线，P24-00继续开发，W1暂不放行。r1旧编排文字以最新执行入口覆盖，不另造完成状态。


### 2026-11-11 Windows W0已交回并接收

[接收复核](0.2.24-contracts/windows-w0-receive-review-20261111.md)固定b2e89665d/0eef59a01，核对34原证据文件/3批manifest及实际源码，保留.gitignore行尾差额和4个失败目录清理拒绝。main2d68a28bc净树接入13个贡献文件并注册父模块，未导入target历史，不在Mac冒Windows原生执行。产品调用待P24-04接线，W1与W2未放行，W0清理未完保持。


### 2026-11-11 版本选择与初次供应核心

main4524c84fe/ef4c46767新增受管当前选择/版本镜像校验及版本化首次供应；[共享合同检查点](0.2.24-contracts/program-layout-r1.md)已补字段和真实完成边界。31供应＋6选择保护＋1标准app路径检查通过，源/日志/失败/清理在program-selection-r1。尚无真正官方2.99新bundle，生产startup/install/update等消费者未全部切换，升级/回退/显式恢复仍待实现，因此P24-00/01不关闭、W1不放行。本轮未启动官方服务或修改日常配置。


### 2026-11-11 真实官方包和生产消费者检查点

main3d3c90943固定官方2.99及024构建资源入口；main9125f7134接版本化启动供应/来源/依赖/环境和认证查询入口。详[消费者合同r2](0.2.24-contracts/program-consumers-r2.md)，93项相关检查、2项真实官方供应/环境检查及7项只读执行通过，临时测试残留已清理；Mac实际完整bundle已构建，Windows及原包发布未完成。P24-00/01仍继续更新切换、CLI/卸载等合同，P24-03的环境检测接线已推进但UI未验收。

Windows独占接口新增[HOME/Profile分离修正](0.2.24-contracts/windows-w0-integration-findings-r1.md)：现有W0强制HOME=OPENCODEX_HOME会丢失产品两个不同输入，上层传环境也会被覆盖；不修改Windows原件或伪造原生故障，待本线受影响组件修正，相关产品接入不能放行。其它本机任务继续，不重跑旧r5。


### 2026-11-11 渠道保存的等价变化核验

main b058d9071：实际v2草稿在服务端固定去除已知运行记账后的配置摘要；普通保存、预览、暂存Key在锁内核验JSON排版及已知记账变化，原始字节仍按读到的最新值执行事务CAS。真正的地址、Key、未知字段、运行策略和文件缺失仍拒绝且保留草稿；历史恢复、带原件身份的副本与草稿导出不采用等价放行。修复同名渠道引用重算时发现记录自碰撞。前端只凭同一草稿与凭据暂存回执更新其它待保存Key的来源标记，并复核节点/认证目标。

隔离Rust回归67项、前端定向58项通过，证据.adg/evidence/0.2.24/model-save-baseline-r1/；初次摘要格式与后续只读预览自碰撞失败均保留。本机测试残留已清理。实际冲突的显式无损重核入口与原生导入/备份仍待实施，不在本阶段标整项通过。Windows W0两目录输入修正仍待回交，现无新增Windows提交；其它本机开发继续。


### 2026-11-11 原生配置单层导入与备份

main 00cfc36a4：模型工具栏在“读取现有渠道”和“新增渠道”之间提供“导入配置”。选文件时用固定官方 2.99 校验原文件与保留本机端口后的候选，发放一次性选择 ID；点击“导入”不再弹第二层。提交前在配置锁内复核源路径、源摘要、当前 HOME、端口记录与模型状态，并要求已证明本机服务停止。对目标原字节创建并复验完整备份，再通过原有模型文件事务写入和读回；源中其它字段、内联 Key、密钥池和 headers 原样保留，不从旧目标合并或复制外部认证文件。以源文件重建模型来源账本，不运行保存编译器或自动匹配模版；返回的 DTO 不含凭据。

6 项隔离事务测试、1 项未修改官方 2.99 实际校验与合成事务、20 项前端回归及类型检查通过。前置失败及修正依据保存在 .adg/evidence/0.2.24/native-config-import-r1/；隔离目录已清理。真实官方包测试未启动服务，停止证明由测试注入，不能作为产品平台验收。备份已可核对原字节；独立整文件回退入口、真实冲突的显式无损重核与 Windows 最终候选仍未完成。

### 2026-11-11 原生配置独立整文件回退

代码主线 e30294a85f6c9ef1104a32796ca2f04dfb05b07f 在「设置 → 数据与备份」加入独立回退，当前列出原生导入及原生回退生成、且指向实际 HOME 的完整备份。确认页显示目标与端口变化；确认时复核备份摘要、大小、权限、选定官方运行包和本机基线，证明服务已停止后完整备份当前文件，再按原字节恢复并读回。会恢复文件中的原端口；Desktop 单独的端口模式记录不回退，生效模式按该记录与回退后的端口核对。取消及字节相同不新增备份、不替换配置。

10 项 Rust 隔离事务测试（4 项回退新增）、44 项前端回归与类型检查通过；摘要、测试日志与夹具清理记录在 .adg/evidence/0.2.24/native-config-rollback-r1/。尚未纳入手动及升级保护备份，也未做 Windows 实机 ACL 或集成候选验收；P24-02 仍未完成，下一项为保存真实冲突的显式重核。

### 2026-11-11 保存来源显式重核与单渠道边界

main 11bc469b496098a6827ca3db704d67fd789017b4 增加独立「核对配置来源」，在保存或只读投影的真实来源冲突后可进入预览。确认只更新草稿基准，保留本地渠道/模型编辑、待暂存或已暂存 Key、原复制来源；之后用户仍需手动保存。没有调用自动模版匹配或把 Read 当刷新。详情见[重核合同](0.2.24-contracts/model-source-reconcile-r1.md)。

本次修正单渠道保存边界：其它已管理渠道以已核对磁盘观察为准，但不自动接管无关原生渠道；不落盘另一渠道的暂存凭据，仍保留其草稿。重核凭据/地址与模型参数的独立变动有专门正例，同源重叠、删除来源、预览后的磁盘/草稿变动及未完成事务保持拒绝；不以修改摘要冒充已处理冲突。复制来源仍绑定其原记录，新增完整源渠道指纹后仅能在原源渠道全部内容未变时复用。

120 项后端隔离事务回归、67 项前端定向回归、类型检查、cargo fmt 和严格 Clippy 通过。前置失败、最终日志、沙箱边界、原包未改及测试残留清理事实保存在 .adg/evidence/0.2.24/model-source-reconcile-r1/。两处原有等价写法警告单独提交 main973dbf92b，不改断言或产品语义。未操作日常配置或启动真实业务服务。本阶段不等于 P24-02 整项完成；下一步接手动/升级完整保护备份、版本化程序生命周期与 CLI。Windows W0 两目录输入修正仍未有新回交，W1/W2 不放行。

### 2026-11-11 完整原生配置手动备份及现有安装事务保护

main0eea7274c 加入「设置 → 数据与备份 → 备份完整配置」，保存已落盘的完整原字节（含配置内凭据/池/headers/端口和未知字段），版本/镜像身份仅记录可验证事实，未确认则保持未知。手动、导入前、回退前及升级前配置备份共用当前 HOME、摘要、大小、链接/权限校验及独立回退入口。有效备份可恢复损坏或缺失的当前配置；损坏原件另存，原件不存在时不伪造保护备份 ID；备份中未指定的端口保持原字节，不强制补入新端口。

main11dbbdd9c 在现有安装事务的程序替换前接入完整配置保护，安装窗口重装也不能绕过；配置与备份在激活来源提交前再核对。备份失败不换程序，忽略共享锁的外部配置改动导致本次程序事务回退，外部配置原件不被补偿覆盖。Desktop 偏好保护和清理规则保留。详[完整原生配置保护合同](0.2.24-contracts/native-config-protection-r1.md)。

120 项隔离后端定向回归（原生配置16、安装35、备份57、升级偏好4、端口8）和56项前端回归、类型/格式/严格Clippy通过。失败、最终输出、二进制摘要、沙箱边界及清理事实在 .adg/evidence/0.2.24/native-config-protection-r1/。当前安装事务测试使用合成包/探针，不冒充官方2.99原生集成通过；版本化程序安装/切换/回退、正常启停归属和CLI仍待接线，P24-01/02不关闭。下一步转入版本化程序生命周期，之后固定候选。Windows分支仍无新贡献，不提前放行W1/W2或发布。

### 2026-11-11 版本选择的可恢复激活事务

main60e9fc640 新增 program_activation：对稳定入口、current.json 与本机 runtime.json 保存严格原件/后像及摘要，序列化写入，完整读回、保留终态回执后才清 pending。首次固定离线供应已接入；普通来源读取继续拒绝未完成事务，不自行恢复或猜其它版本。显式完成/回退 API 只操作原事务已绑定的三项元数据，保留旧/候选版本目录、未知外部值与失败记录；另一个 Desktop 根在创建来源锁之前就拒绝。共享接口见[版本激活合同](0.2.24-contracts/program-activation-r1.md)。

47 项合成隔离回归通过，包含首次/已有版本四阶段中断×继续/回退的16场景；另外使用原固定官方2.99完整离线资源实际展开、核验及重开1项通过，未启动Node/CLI/服务，原合成HOME字节保持。官方资源检查耗时72.2秒，不能冒充启停30秒、查询5秒、路径10秒预算通过；最后修订仅恢复异根准入顺序与模块排序，供应路径未变，最终源码重新构建并重跑47项合成测试。证据 .adg/evidence/0.2.24/program-activation-r1/ 保留初次镜像字段命名失败、最终输出与清理事实。

本阶段只接首次供应和共享Rust事务；常规安装/更新候选装配、版本化恢复UI、启停归属接线、卸载及CLI仍待完成。不将P24-00/01关闭，不放行W1/W2或发布。Windows分支检查仍为代码b2e89665d、证据0eef59a01，无新回交。

### 2026-11-11 版本化安装与更新消费者接线

main b617717c3 将 versioned handle 的安装窗口和官方更新命令转入版本目录候选装配，固定当前 Node/npm 到新版本依赖，官方源归档先复制/复核并用相对 sources 规范安装，再核对原包文件树与实际版本。Bun 1.4.4 的正常npm占位入口按真实锁定归档摘要识别后替换为对应平台二进制；不改官方OpenCodex源码，不默认执行依赖脚本。候选发布与来源/history切换复用共享激活事务，程序与配置原件保留。详[版本化安装合同](0.2.24-contracts/versioned-install-r1.md)。

60项最终隔离回归、格式和严格Clippy通过。另1项真实官方2.99/Npm/Node/Bun联网获取后离线缓存复验通过，两次同版本核验不切换配置，使用专门临时HOME及127.0.0.1:3060代理，无业务服务/模型请求；停止事实由测试注入，不替代平台验收。官方实测后仅补候选根链接拒绝、排除临时下载目录和原包被改反例，最终重建并复跑相关合成测试，原实测源码边界在证据中保留。证据 .adg/evidence/0.2.24/versioned-install-r1/；临时候选/缓存/重复回执清理，原结果保留。

本次版本切换要求真实停止证明，准备前与激活过程中均核对，避免丢失旧运行实例归属；这只是安装准入接线，现有服务归属仍有flat路径假设待改，下一步接待恢复事务产品入口与版本化归属。候选落位后未激活的重试只复用与新候选整树相同的目录，不猜旧来源。同版本程序已被外部改变不能通过普通安装覆盖，另需显式恢复。离线缓存不全仍报错，不隐式联网。W1/W2与P24-NAME边界保持，整版未完成。

### 2026-11-11 中断版本切换的产品恢复入口

main063af3a96 将设置安装页的恢复入口接到真实版本事务：只读观察原pending，显示原/目标版本及程序/配置位置；确认继续或回退时绑定原操作、实际I/D/H及三个目标的当前原字节，过期确认拒绝。应用存储/安装租约、管理器更新状态、真实服务停止查询和当前完整配置备份均在命令编排中；关闭仅退出观察，重开不重复提交。详[恢复入口合同](0.2.24-contracts/activation-recovery-ui-r1.md)。

mainaff0d9952 同步修正共享服务归属的路径来源：版本化RuntimeHandle提供已准入选择的轻量身份观察，归属不再猜flat目录；旧flat进程路径和选择漂移拒绝。此批不宣称Windows W0生命周期或实际Stop/查询预算整链通过。

30项后端定向回归、90项前端回归、类型/格式/严格Clippy通过；实际共享组件在Chromium以1024×740与580×480窗口验证预览/回退/结果，无横向溢出。证据 .adg/evidence/0.2.24/activation-recovery-ui-r1/ 保留初次本机监听被沙箱拒绝、前端夹具双Pinia原因和最终图像/摘要；临时测试目录/浏览器/服务已清理，无日常配置或官方包修改。

当前恢复只处理未完成的版本选择事务，不将“镜像被外部修改的同版本修复”、官方运行实例恢复或跨平台候选验收标完成。实际CLI、卸载、版本/网络UI、Windows独占接线与NAME仍按计划继续，W1/W2未放行。

### 2026-11-11 Mac终端官方ocx入口

maine9b00583c 新增显式终端命令预览/启停与设置卡，独立于既有ocxd本机IPC开关。当前Mac命令 ~/.local/bin/ocx → 当前程序根runtime/bin/ocx → Desktop --ocxd-runtime-cli，转交在Tauri/GUI启动之前执行；每次读取唯一当前程序选择及持久D/H，传原参数和退出码，保留真实用户HOME，只绑定OPENCODEX_HOME。Shell和文件修改带精确原件/后像及当前归属，冲突/外部修改不覆盖。详[终端入口合同](0.2.24-contracts/terminal-cli-r1.md)。

18项后端定向回归、41项前端测试、类型/格式/全目标严格Clippy通过，另外2项显式实测用新构建Desktop二进制：合成程序证明无GUI启动/参数退出码23保真；原样官方2.99从自定义用户集合根和HOME执行 --version 及 config get port --json，版本/端口一致、配置原字节不变，停用后自有入口和新建Shell文件移除。zsh/bash新会话识别、被后来加入的其他ocx优先保持、特殊路径/参数、损坏pending拒绝均有测试。证据 .adg/evidence/0.2.24/terminal-cli-r1/，只在隔离用户/临时目录执行，测试残留清理，未改真实Shell或日常配置。

当前更换D/H时需先停用已登记终端入口，防止命令孤立在旧用户集合；该检查在现有绑定写入准入内。Mac支持zsh/bash，对不支持Shell或自定义ZDOTDIR明确拒绝自动修改。Windows原生命令/PATH独占后端及卸载时撤销未接入，不能用Mac通过代替Windows完成。真实原包安装器、镜像修复、版本/网络UI、NAME、W0/W1和最终W2仍未完成；保持完整范围，后续推进版本化卸载与终端撤销联动。

### 2026-11-11 版本镜像完整目录摘要

mainbcb081ee4 为新建版本镜像加入必需treeSha256，覆盖相对路径、普通文件原字节/大小/执行位和空目录，排除根镜像清单自身并由current的imageSha256单独绑定。首次固定资源供应和新安装候选分别在落位后生成；安装、激活、显式恢复在动作边界核验完整镜像，不增加高频状态查询扫描。原Node包内已按固定清单核验的npm/npx文件链接先实体化为独立文件，大小/数量/路径保护保持，不接受未知链接。详[完整镜像目录合同](0.2.24-contracts/program-image-tree-r1.md)。

63项最终隔离回归（镜像3/选择7/事务7/供应33/安装4/终端9）、格式及全目标严格Clippy通过。另3项显式原生/原包检查通过：官方2.99实际供应并核验完整目录，新构建Desktop转交及官方CLI自定义HOME只读读取后完整目录摘要不变。官方供应约76.5秒，只是离线展开核验，不冒充启停/查询原预算。供应实测早于最后反例和文件实体化断言补充，生产目录算法未再变化；最终源码重新构建并重跑合成回归及真实CLI。原失败/实测源摘要和清理事实保存在 .adg/evidence/0.2.24/program-image-tree-r1/。

此批是版本化卸载/镜像修复的准入基础，不执行目录删除、不将卸载或所有权认证标为完成；旧开发镜像缺少新字段保持不可确认，不自动补摘要接管。下一步仍是卸载计划、终端撤销和可恢复移除事务；其它版本/网络UI与Windows/NAME验收范围保持。

### 2026-11-11 可继续的精确版本移除核心

main5e75b0c86 新增 program_removal：只接调用方已准入的明确版本集合，冻结原文件清单/current/入口/来源，配置完整保护和自有终端撤销后才进入删除；只删除逐项核对的普通文件和已空目录，不用递归猜删。中断后仅承接原冻结范围，已删除对象可保持缺失，存活对象任何新增/改动、外部入口或另一D均拒绝。七阶段中断后继续、实际Mac终端撤销/外部替换拒绝、原配置/备份/无关程序保持均有测试。详[移除核心合同](0.2.24-contracts/program-removal-core-r1.md)。

完成记录和未完成记录区分，普通来源读取拒绝pending；完成后启动先核对卸载事实，不自动解包重新供应。修正versioned初始化在无current时改写runtime.json时间字段的问题，避免破坏已完成事务后像。71项隔离后端定向回归及格式/全目标严格Clippy通过，证据 .adg/evidence/0.2.24/program-removal-core-r1/ 保留原启动失败和最终结果，测试只删除合成程序根内冻结文件，临时根已清理，未改日常配置或官方包。

这是Rust核心，不将现有卸载窗口/命令标已接通。生产仍待：可信版本集合登记源、所有命名程序在真实OS上不在用的证明、UI预览/继续入口、与用户选择范围及官方清理/数据选项编排、卸载后的显式内置重装。Windows原生事实不得用注入unused回调替代；W0/W1、镜像修复、版本/网络UI和NAME仍继续，P24-01不关闭。

### 2026-11-11 可信版本登记与Mac程序占用证明

main0954ec6f0 将首次供应和更新候选发布接入显式image-admissions登记，在后续版本探针之前持久记录已准入镜像，失败候选也有原来源。卸载生产准备入口prepare_registered先消费该登记再比对实际版本目录，确认/中断继续绑定原catalog revision，不扫描任意镜像赋予删除权限。main2ab15a9e0 增加Mac同UID程序占用证明，覆盖其它HOME、重命名程序与原路径已移走的存活镜像，前后核对PID创建代次；未知或不可读不当缺席，Windows跨SID范围明确未接。详[登记及占用合同](0.2.24-contracts/program-catalog-native-r1.md)。

72项最终定向回归、格式及全目标严格Clippy通过；真实固定官方2.99供应/登记与重开1项通过，配置原字节保持，未执行服务。native临时系统程序需ad-hoc重签才可在复制后的独立inode存活；仅改测试夹具，签名/权限/未知拒绝断言不放宽。原失败、夹具诊断及最终原包摘要在 .adg/evidence/0.2.24/program-catalog-native-r1/；一条过宽进程参数诊断已在留证前明确删去，未留无关进程argv，其失败断言仍保留。所有测试/诊断程序与临时根已清理，无业务同步/模型或日常配置改动。

版本登记和Macunused API已可由卸载编排消费，实际卸载窗口/命令和完整官方清理、卸载后显式内置重装仍待接；不将本阶段标P24-01完成。WindowsW0 HOME/Profile修正和W1/W2、镜像修复、版本/网络UI及NAME保持后继范围。

### 2026-11-11 保留配置的卸载产品入口

main716020fac 将设置程序卡的卸载入口接到真实版本登记、冻结原计划、一次性服务端预览票据和原终端撤销摘要。当前默认按已采用原型移除运行版本及自有终端入口，完整备份当前配置、保留用户配置/备份，不再从本窗口带入旧full/清空HOME/global npm范围。预览时不备份或删除，确认后持有应用存储/安装预约与真实Mac全部程序占用证明；预览过期、目录或终端变化、服务未停止时拒绝。窗口关闭重开复用原请求，重启后的partial/marker操作重新预览同一原卸载，不新造删除范围。详[卸载产品合同](0.2.24-contracts/program-removal-product-r1.md)。

43项后端定向回归、45项前端/设置集成、类型/格式/全目标严格Clippy通过；实际共享组件1040×760及580×480确认/继续/取消/结果检查通过。另用固定官方2.99完整镜像实际供应、登记、预览及删除1项通过：13393文件/645549518字节移除，配置原字节与备份保持、终端撤销，删除编排约20.7秒；unused证明在该原包测试中仍为隔离注入，不当真实原生GUI或生命周期预算验收。证据 .adg/evidence/0.2.24/program-removal-product-r1/保留前置失败/最终输出和清理，未改日常配置或官方包。

本批接通Mac保留配置的默认卸载，不将完整官方客户端/服务登记清理、可选数据删除或Windows共享程序/PATH撤销标完成。用户先停止服务，生产查询不确认则拒绝且无强杀。旧设置测试按当前原型重写为实际scope/无副作用/原操作继续回归，旧后端测试不删除。卸载后显式内置重装及坏镜像修复、WindowsW0/W1、版本/网络UI/通知/NAME和固定候选验收仍继续，P24-01未关闭。

### 2026-11-11 读取现有渠道的注册通知

main b6dc941cf 将明确读取后的模版匹配数量接到后端注册通知；删除渠道页常驻“上次成功读取”汇总。实际 guarded read 返回并读回草稿后，在同一存储写者内发布一次数字结果，Toast 可收起，铃铛/通知历史可回看；说明仍需保存配置才生效。失败发布固定分类，不复用旧成功摘要或记录渠道名/模型ID/地址/密钥。普通快照、打开页面和通知历史加载不伪造新读取通知。详[读取结果通知合同](0.2.24-contracts/channel-read-notifications-r1.md)。

通知刷新改为等待当前及排队读回的同一promise，announce串行聚合新事件，解决旧列表加载时遗漏新Toast的竞态。49项前端/通知回归与类型检查通过；新增3项实际读取/持久通知、43项注册表回归最终通过，已有24项实际渠道读取保存回归通过且未重复跑；格式与全目标严格Clippy通过。1040×760、420×640浏览器真实共享组件确认完整数字、收起、回看和不重复提示，无横向溢出/页面异常；详情容器与list IPC为显式夹具，不当原生候选验收。证据 .adg/evidence/0.2.24/channel-read-notifications-r1/ 保留首轮类型/注册失败、后续41项注册失败与最终结果，临时根/页面夹具已清理，不改日常配置或执行业务请求。

卸载后显式重装仍未实现。已核实旧image-admissions需作为卸载证据保留，不能只删REMOVED标记便复用首次供应；后继须以可恢复原操作归档旧登记并发布固定资源，不能把未知目录当新安装。Windows分支检查仍为代码b2e89665d、文档0eef59a01，没有新回交；W0 HOME/Profile、W1/W2、同版本镜像修复、版本/网络UI、NAME和固定候选验收继续，不关闭P24-01或整版目标。

### 2026-11-11 卸载后显式内置重装

main f180ee980 实现已完成登记卸载后的显式固定资源重装及原操作继续，设置提供重装入口。I/D/H及固定pin/原记录摘要在取得程序写者后复核，先完整保护当前配置，再不覆盖地归档原卸载marker和image-admissions；复用同一官方归档/镜像builder，新live登记与entry/current/来源只使用原候选和激活原字节，不从变化后的来源生成新基线。普通启动/读取和普通激活恢复不能继续重装内部事务。配置/备份与其它工具保持，不自动启用terminal，未知占用/外部改动/其它用户上下文拒绝。详[显式重装合同](0.2.24-contracts/program-reinstall-product-r1.md)。

本轮74项相关后端检查（含1项完整官方原包往返）、47项前端/设置回归、类型/格式/全目标严格Clippy通过；12个明确中断点可继续同一原操作，过期确认在读取归档前拒绝，损坏资源不改变已卸载状态。实际官方2.99供应→卸载→重装→真实版本输出约194.3秒完成，原配置/卸载证据/catalog保持；unused为隔离注入，不冒充真实GUI/生命周期验收。共享组件1040×760和580×480无溢出/页面错误。证据 .adg/evidence/0.2.24/program-reinstall-product-r1/ 保留Clippy、缺失根预览、原包版本输出前缀及错误夹具失败；全部测试程序/配置根、页面夹具和重复临时回执已清理，日常配置/官方2.99未改，无业务请求。

显式重装不替代同版本损坏镜像修复或完整官方客户端/服务清理，也不完成标准单.app/最终打包。Windows W0 HOME/Profile更正与W1/W2、版本/网络UI、NAME和固定候选验收继续。后继先做损坏镜像的原归属/保护及恢复范围，再推进余下共享界面与跨平台接线；不关闭P24-01或整版目标。

### 2026-11-11 原登记同版本镜像修复

main8a1fda555 接通设置「修复程序」和runtime_builtin_repair_preview/repair；与卸载后重装分开准入，复用固定资源完整builder和异步观察窗口。current与原image-admissions先证明原选择/官方来源，其它已登记版本仍核验；预览冻结原程序/来源/入口和完整损坏现场，确认/继续不从新现场重建许可。先保护完整config，再将原损坏目录及入口原件保存在repair-history，不删除额外普通文件或用户配置；发布相同原镜像、修复稳定入口，current/runtime.json/live登记保持原字节。未知占用/外部修改/不匹配资源拒绝；repair pending阻止普通启动/其它恢复窗口绕过。详[同版本修复合同](0.2.24-contracts/program-repair-product-r1.md)。

62项相关后端（含1项固定官方完整镜像真实破坏/修复/版本输出）、42项前端/设置回归、类型/格式/全目标严格Clippy通过；浏览器1040×760、580×480实际共享组件无溢出/页面错误。官方原包链约184.4秒，损坏Node原件、完整配置与来源记录保持；unused仍为隔离注入，不当原生GUI/生命周期或Windows验收。证据 .adg/evidence/0.2.24/program-repair-product-r1/保留首轮E0107及最终结果；程序/配置/页面夹具与重复临时回执已清理，未改日常配置或官方包。

当前修复只适用于与固定完整资源完全一致的原登记镜像；其它来源/版本不能用内置2.99降级或覆盖，若最终范围要求其重建仍须接相应可信资源，不声称任意版本已支持。原损坏镜像会额外占用存储，无自动删除原归档。Windows检查仍为b2e89665d/0eef59a01，无新回交；HOME/Profile修正与W1/W2、版本/网络UI、外部版本重核、NAME、完整官方清理/单.app及固定候选验收保持。下一步推进环境/版本/网络UI与Windows接口闭合，P24-01和整版目标均未关闭。

### 2026-11-11 双目的地网络诊断与说明反馈

main029eaa4ff 将设置检查连接改为当前保存代理下GitHub仓库API与官方npm元数据独立检查，状态/HTTP/有限重试建议/检查时间分别显示；实际身份/总时间/大小/重定向/TLS边界保持。共同HTTP构建应用手动代理与免代理项，None不继承环境，诊断无配置或版本写入。maind4d091ad0 将说明请求失败区分超时、限流、其它HTTP、网络、身份和超限，明确缓存所属版本与原获取时间；刷新失败仅保留精确同版本且未过期的原已核验缓存，原缓存不被失败覆盖。当前updater正文保持其当前来源与时间，不被旧缓存替换。详情失败有重试说明/网络设置，重试不触发检查更新或安装。详[网络与说明反馈合同](0.2.24-contracts/network-notes-feedback-r1.md)。

25项后端（network4/notes21）、66项前端/设置更新回归、类型/格式/全目标严格Clippy通过。真实loopback HTTP代理转发及免代理直连、限流/HTTP/超时/错误身份/重定向/重复JSON/越限反例均验证；缓存原字节保留及当前updater优先/时间来源补反例。浏览器1040×760、580×640真实共享组件和内存IPC显示独立结果、精确缓存和说明单独重试，无溢出/页面错误，截图已查看。证据 .adg/evidence/0.2.24/network-notes-feedback-r1/ 保留前端hook/迟到回执及视觉夹具失败和修正结果，所有测试根/页面夹具与重复临时回执已清理；无实网GitHub/npm、日常配置写入或业务请求。

这批只闭合网络诊断/说明反馈接线，不将POST-05内置供应阶段/重试、POST-07磁盘/运行/历史版本区分及外部更新重核标完成。其它HTTP消费者的免代理一致性继续核对，WindowsW0 HOME/Profile/W1/W2、NAME、完整官方清理、单.app/原生候选与最终验收保持，P24-03及整版目标未关闭。

### 2026-11-11 内置环境实际阶段与明确重试

maina1a3e6d5d 在RuntimeHandle内记录本启动的真实准备阶段，固定builder直接发布校验/展开/依赖装配/激活边界，失败保留实际failedPhase；只有current可读且实际worker存储/安装租约释放后才Supplied。IPC只读状态不写安装事实，明确retry只接受上次失败attemptId，沿同一保护/供应准入；已有current/版本/pending/卸载/repair/reinstall不自动覆盖，转安装配置。页面观察真实阶段，Supplied后独立运行现有依赖检查，准备不当服务已启动；观察不可读不转缺少系统Node。详[内置准备合同](0.2.24-contracts/integrated-preparation-r1.md)。

47项相关后端（含1项完整固定官方2.99首次供应worker约77.8秒）、28项前端/环境回归、类型/格式/全目标严格Clippy通过。真实官方供应证明原配置保持、选择读回、Supplied及租约释放，未启动服务；浏览器1040×760和580×640真实组件/内存IPC确认阶段/失败/一次明确重试和环境/服务区分，无溢出或页面异常。证据 .adg/evidence/0.2.24/integrated-preparation-r1/保留缺D夹具与错误首错文案断言、旧页面预期及最终结果；测试程序/配置/页面夹具和重复临时回执已清理，无日常配置/官方包改动或业务请求。

POST-05本次完成状态/重试产品接线，不将正式原生候选或整版启动预算标通过。POST-07磁盘/当前进程/历史版本及外部来源重核UI继续；其它HTTP消费者免代理一致性、WindowsW0 HOME/Profile/W1/W2、NAME、完整官方清理及单.app/最终候选仍保持。Windows最新检查仍为b2e89665d/0eef59a01，无新增回交，整版目标未关闭。

### 2026-11-11 磁盘/运行实例/历史版本与更新重核

maindadf16e48 新增只读runtime_version_facts：实际current定位package.json的磁盘声明、当前H受认证运行进程版本、上次安装记录及时间分开投影。versioned本地版本查询不再回写安装历史，文件/选择/pending前后读回不一致拒绝混合快照。新鲜双challenge同连接/HMAC/PID/port/uptime及记录回读可观察磁盘已更新但旧进程仍运行；此观察不产生启动/停止/ready许可，既有health_proof精确版本保护保持，无记录不推断已停止。详[版本事实合同](0.2.24-contracts/runtime-version-facts-r1.md)。

更新窗口显示三种版本、核对状态和时间，来源不一致/读取失败时不称可更新或已最新；失败旧结果标上次观察。打开窗口/回前台/明确检查刷新，generation拒绝迟到回执。确认先冻结原摘要并重读，变动在备份/安装前拒绝；后端必须expectedObservation，在远端查询前及共同安装worker取得预约后重核，执行许可额外全镜像核验，非关键用户新增文件不被展示级检查放行。备份等待不换新的晚到基线，current/runtime.json/配置保留。

17项后端（disk4/attestation9/安装4）、40项前端/更新/调度回归、类型/格式/全目标严格Clippy通过；最后展示依赖调整受影响2文件25项补验通过，未重跑无影响已过项。浏览器1040×760/580×640真实组件和内存IPC分别显示2.99磁盘/2.98实例/2.97历史和失败上次观察，无溢出/页面错误，截图已查看。证据 .adg/evidence/0.2.24/runtime-version-facts-r1/保留新API/缺事实夹具及等待重核前置失败与最终结果；全部程序/配置/页面夹具、重复临时回执已清理，无日常配置/官方包改动或真实升级/重启/业务请求。

当前POST-07完成只读事实/刷新/执行前拒绝漂移接线，不自动将外部更新包重新赋予可信身份或承诺未知包可继续升级；最终签名候选需验证实际完整调用链。WindowsW0 HOME/Profile/W1/W2、NAME、完整官方清理、标准单.app与其它HTTP免代理一致性和整版候选验收保持，目标未关闭。

### 2026-11-11 单.app装配及W1部分放行

main318b75e53/17f608590/2bfca20d1 更新Release装配为024官方2.99 stage及单.app DMG，移除旧维护运行包生成；源.app签名/全树及固定资源核对、只读挂载再核验、不覆盖产物和自己的卷/临时清理保持。新增tag/实际Tauri/UI/Cargo/policy版本预检，当前实际版本仍023，明确拒绝作为024出包，不偷改版本或触发发布。旧publish verifier不再允许024套用maintained-runtime门禁，新的官方集成交付验证仍未完成，当前明确拒绝发布。详[单应用包装合同](0.2.24-contracts/single-app-packaging-r1.md)。

19项包装/stage/历史包装回归及1项发布门禁反例通过，YAML/Python解析通过。真实签名.app夹具携带完整固定官方资源，实际hdiutil制作/只读挂载267890286字节DMG，核对单.app、Applications/安装说明、原签名字节和官方SHA通过，约23.3秒；这不是编译真实Desktop或产品启动/安装验收，fixtureOnly/productAcceptance=false。证据 .adg/evidence/0.2.24/single-app-packaging-r1/保留预检正则SyntaxError、修正后真实版本未冻结拒绝及最终结果，自己的mount/.app/DMG夹具和重复临时回执清理，未碰日常软件或配置，未发tag/工作流/发布。

TEAM递增revision5，按[W1原生接口r1](0.2.24-contracts/windows-w1-native-interface-r1.md)放行W0 HOME/Profile修正、W1-A PATH真实原语/反例、W1-B MSI/NSIS024官方资源/路径保留三项可独立工作；主线负责共享CLI事务与父注册/生命周期接线。当前Windows回交仍b2e89665d/0eef59a01，不能把提示词/Git已推送当已接令。最终W2仍等候选SHA/制品摘要及受影响表，不提前跑全量或重跑旧r5。

实际产品单.app构建/安装、Windows新回交/完整接线、NAME、完整官方清理、其余HTTP免代理一致性、版本/README冻结、官方新发布验证和最终候选验收保持，P24-01及整版目标均未关闭。

### 2026-11-11 Mac真实024工程构建及隔离首窗

maina58f03cb9 将Tauri/UI/Cargo与两锁版本统一为0.2.24，版本预检通过；这是开发版本非最终冻结。实际TauriCLI2.11.4 offline Rust/custom-protocol/固定官方资源构建真实production.app约255.2秒成功，Info.plist0.2.24及严格codesign通过，ocxd --help可用。新包装器核对真实产品资源/只读挂载/单.app树和原签名，工程DMG277329903字节、SHA39782f66a24b158600810c67653388219150ea94c08f710524fb396276e3d60e。实际binarySHA fda508bfbe7ac3e6ee71512c827d1aeb07ecdd0e0280295e8d9034afef26ea20，构建来源为09392b38e+确切版本补丁，未伪称嵌入后提交SHA。详[Mac工程包合同](0.2.24-contracts/macos-engineering-build-r1.md)。

CUA两次inventory超时后未猜锁屏，既有PID限定AX/CG采集器证明桌面已解锁/权限有效。工作树.app在精确sandbox因resource_dir unknown path退出；同一密封.app原样复制到隔离Applications、保持日常用户根/Keychain/出站拒绝后真实首窗渲染/截图成功。独立realpath在原policy下对workspace exe/Resources返回EPERM、对隔离路径成功，定位为测试路径隔离冲突，原失败和因果证据保留，没有放宽产品检查或去签名。Sandbox身份按既有合同跳过供应，因此这里只证明首窗，不冒充正常用户首次供应/生命周期全链。

证据 .adg/evidence/0.2.24/macos-engineering-build-r1/保存source原字节、build/签名/包/AX/截图、首错/realpath及清理；own进程组已空、所有副本.app/私有配置/辅助工具/临时卷与重复回执清理，仅target保留一份工程.app/DMG。未改日常配置/官方2.99，不跑服务启停/升级/同步/模型，不触发发布。

W0/W1贡献检查仍b2e89665d/0eef59a01；共享层可继续免代理/生命周期整合及NAME有界技术责任，但不能以工程包替代W2固定候选。README/版本发布说明最终同步、官方新交付发布验证、完整真实服务与客户端恢复及NAME仍是收口必需，整版目标未关闭。

### 2026-11-11 更新下载/npm消费者免代理一致性

main80afa4418 补齐管理器更新器metadata/download共用保存proxy/noProxy快照的configure_client钩子，以及运行包npm pack/install与npm view的NO_PROXY/npm_config_noproxy环境。Direct清除外部代理，Manual只用保存地址/免代理且非法不回退，System保持库客户端原选择；签名/TLS/候选/安装预约原流程不变。固定updater2.13.1及其reqwest0.13.5钩子API，测试rustls/ring只镜像插件真实初始化，没有升级传递版本。详[代理消费者合同](0.2.24-contracts/proxy-consumers-r1.md)。

31项去重后端相关检查（updater3/npmEnv1/安装4/metadata7/真实npm1/manager15）、格式/全目标严格Clippy通过。真实固定Node26.8.1/npm11.19.0对loopback合成registry执行view，分别实际收到代理绝对URI和例外直连，合成npmrc/HOME保持，不pack/install或访问真实registry；更新器钩子真实reqwest0.13请求验证metadata/artifact及三模式。首轮TLS fixture缺插件provider初始化、Clippy大枚举和旧超时测试ps EPERM失败保留；该唯一旧超时case在私有HOME/TMP、无OSsandbox但仅原临时shell/PID限定ps下原断言通过，不放宽产品或全组隔离。证据 .adg/evidence/0.2.24/proxy-consumers-r1/ 保留全部失败/修正及清理，测试根和重复回执已移除，未改日常代理/配置/官方包，无业务/真实升级请求。

此批闭合保存的Manual/Direct免代理传递，npm System模式仍缺原生平台代理解析，不声称已完成；Mac工程.app/DMG早于本提交，最终候选须从最新树重构。Windows最新检查仍b2e89665d/0eef59a01，无新回交；W0 HOME/Profile/W1/W2、NAME、完整服务/客户端还原、官方新交付发布验证及最终候选保持未完，整版目标不关闭。

### 2026-11-11 Mac正常首次供应与缺失状态废止

main45f813d79修正已供应后仍保留旧NotFound：来源已resolved而本次服务查询失败，仅将旧缺失结论废止为Unreachable/未知事实，不编造Stopped/Running；其它瞬时失败/未知来源/真正缺失合同保持。19项相关状态回归、格式和全目标严格Clippy通过。main7be8d2658新增test/smoke/macos024-first-supply.py，用真实密封.app、私有当前v2配置根、无原runtime且不设置OPENCODEX_SANDBOX执行生产首次供应；内核拒绝日常用户根/Keychain/全部网络，PID+exe限定AX/CG观察实际阶段及UI就绪，真实读回Node/npm/OCX版本，无服务/同步/模型操作。详[原生首次供应合同](0.2.24-contracts/macos-first-supply-native-r1.md)。

原工程输入的attempt1因测试未初始化v2根拒绝、attempt2因探针继承被隔离cwd失败均保留；attempt3供应链约86.2秒通过但截图发现旧缺失caption，不冒用为状态通过。最终以干净产品commit45f813d79构建真实.app约247.544秒，binary SHA d398c4e3f67b8ead28fe17f6595cd96731fc5cab32518302d3e6ec1abfcd00eb，严格签名通过，实际启动日志嵌入该完整SHA。第四次正常供应约127.647秒通过：77.694秒current出现、82.105秒Supplied、85.855秒环境门消失，随后真实状态采样收敛至“未运行”；这不是原生注入失败Unreachable证明，也不宣称瞬时刷新或5秒预算。Node v26.8.1、npm11.19.0、OCX2.99.0输出通过，配置原字节保持，无runtime-port记录。

.adg/evidence/0.2.24/macos-first-supply-native-r1/保留四次原件/失败、构建/签名/单测及manifest；夹具副本.app/程序/配置/助手根和逐字节核对后的重复临时回执已清理，自己的GUI进程组为空。target仅保留原工程.app和更旧DMG；旧DMG不能当本次新.app的包。未改日常配置/官方包，无业务请求，不是假新OS用户。Windows检查仍b2e89665d/0eef59a01，无新回交；W0 HOME/Profile/W1/W2、NAME、真实完整服务/客户端还原、npm System平台代理解析、官方新交付发布验证与最终候选保持未完，整版目标不关闭。

### 2026-11-11 当前版本化生命周期实际接线与全表诊断

main336647e3e修正execute_lifecycle绕过RuntimeHandle而误读旧平铺目录的问题：改用当前provider完整程序准入，另核对根/入口，未知和Node漂移仍在端口写入/启动前拒绝；无旧版接管。2项相关反例通过，最终编译/格式/全目标严格Clippy通过。实际官方2.99/versioned完整资源、私有profile及内核隔离下调用生产动作/来源/停止；最终启动4453ms、状态1096ms、正常停止2372ms，原30/5/10秒不变。详[Mac版本化生命周期合同](0.2.24-contracts/macos-versioned-lifecycle-r1.md)。

整轮仍failed/accepted=false：config.toml实际改变后原字节恢复，但catalog原字节、cache/audit原缺席未恢复；Grok备份仅缺席保持，无Agent/全树/新SID证明。先前缺native/history协调权限和裸测试线程无Tokio句柄的失败保留并分别定位；最终只补精确私有HOME协调路径和实际运行时前置，未改产品/官方准入或断言。原生第三轮只留逐文件字节相等结果，原字节未在清理前输出的证据局限明确记录；最终测试补捕获钩子，编译但未无假设重跑服务。

.adg/evidence/0.2.24/macos-versioned-lifecycle-r1/保留各轮首错、独立原包foreground/restore诊断、真实生产日志/来源摘要和检查。自有进程镜像/组已空；原主监听经完整Darwin TCP表核对，lsof无关卷告警不算缺席。精确新协调数据库取得/释放排他事务后删除，夹具程序/profile/副本/重复回执均清理，未强杀服务补通过、未改日常配置或官方包。当前工程.app/DMG未重构，不冒称含新补丁。

下一步先补Desktop客户端恢复事务及后像归属/协调/持久冲突保护，再用单轮受影响恢复链验证，不重复单独启动/状态。WindowsW0 HOME/Profile、W1/W2仍等新Git贡献；NAME、npm System代理解析、新官方交付发布验证和最终固定候选保持未完。整版目标不关闭，不触发发布。

### 2026-11-11 客户端目录恢复共享内容判定

main604f2763e增加client_artifact_policy，对catalog/cache/audit的原件/ready/pre-stop/正常停止后内容做整批有界判定，未知/ready文件代次或保护漂移/新用户模型或缓存/重复JSON/审计前缀丢失拒绝。native slug允许含斜杠，路由归属按启动前独立完整声明，None未知不当空集；原缓存已还原则保留不透明原字节。审计仅补充一致性，不能以PID/hash/日志代替原生写者/退出或官方锁。详[共享内容判定合同](0.2.24-contracts/client-artifact-policy-r1.md)。

17项定向反例及1项已接收Windows r5真实四阶段字节回放通过，编译/格式/全目标严格Clippy通过；纯数据回放从原manifest/六件源摘要和stage/cache-scope读回核对，不是本轮Windows原生服务验收。失败原件/辅助编译错误和首轮错误HOME投影拒绝保留；首错输入被同名结果覆盖的局限明确，最终正确投影单独保存。证据.adg/evidence/0.2.24/client-artifact-policy-r1/，自有测试二进制/夹具/重复回执已清理，无日常配置或官方包改动，不跑服务或业务请求。

这一层未写盘、未接Start/Stop，不把上轮catalog/cache/audit恢复失败改成通过。下一批继续原件保护/原生快照、持久事务及CAS、官方claim/N/K协调和正常退出后的整组补偿；config/Grok/Agent、服务重开/活跃刷新/崩溃恢复适用边界仍必须覆盖。Windows分支检查仍b2e89665d/0eef59a01无新回交，W0 HOME/Profile/W1/W2、NAME、npm System代理、新官方交付发布验证及固定候选保持；目标未关闭、不发版。

### 2026-11-11 Mac客户端原件原生快照及不可覆盖阶段留存

maina19669114提供macos_client_originals：固定catalog/cache/audit三件的真实FD原字节/缺席、dev/inode/birthtime、mtime/ctime、owner/mode/flags/ACL/xattr快照；逐层openat不跟随链接并保持目录身份/安全元数据。当前D内0700/0400原件及四阶段frame按O_EXCL/fsync/父目录同步发布，原件不重建/不覆盖，重开必须独立原摘要/各阶段摘要，已有未知阶段/半文件不被接管。详[Mac原件留存合同](0.2.24-contracts/macos-client-originals-r1.md)。

18项真实Mac文件系统检查（含实际子进程exit91重开、原生ACL/xattr保持、源/祖先/权限改变及链接/未知/不可读拒绝）通过，最终编译/格式/全目标严格Clippy通过。首轮libc字段/trait分类、后继遮蔽原错、OS provenance误拒绝/数字ACL名和Clippy字面量失败均保留。最终仅允许Mac自动附加的有界com.apple.provenance且不移除/改ACL；未知journal属性仍拒绝。native18后只把测试0改0o000，原源摘要及同值边界说明保留，未重复全组。证据.adg/evidence/0.2.24/macos-client-originals-r1/，自有进程/测试二进制/夹具/重复回执清理，无日常配置或服务/业务操作。

本批未向客户端写回，也未接Start/Stop；缺失CODEX_HOME目录仍错误，不误作空集，正式首次启动的目录缺席/创建/撤销要继续接。外层持久摘要/正常退出身份、官方claim/N/K、客户端CAS/原子补偿与失败恢复，以及config/Grok/Agent仍待实现，上轮三个文件未还原不改通过。Windows最新仍b2e89665d/0eef59a01无新回交；W0 HOME/Profile/W1/W2、NAME、npm System代理、新官方发布验证及固定候选保持，目标未关闭。

### 2026-11-11 官方claim/N/K实际排他及锁内原件读取

main6dece428a以当前完整程序/tgz身份及全镜像准入运行Desktop内嵌helper，直接import原样官方2.99的claim→N→K原语，作用域有界借用/真实Child/上下文/锁dev-inode及held/released/正常退出核对；已有不安全锁先拒绝，不修补旧mode；回调过期/失败不转成功，无服务/模型/同步调用。详[官方协调合同](0.2.24-contracts/official-client-coordination-r1.md)。

5项协议/来源/真实管道边界和1项真实官方完整资源协调检查通过，编译/格式/全目标严格Clippy通过。真实另三个SQLite连接在持锁期间均busy，原件实际在锁内捕获；正常/失败/过期后可重新取得排他，原稳定锁身份及客户端原字节保持，外来HOME与不安全旧权限拒绝。首个协调4705ms（测试显式5sdeadline），不是完整Stop10秒通过；原生轮总94.596秒含供应，仅测试后来新增pipe检查不重复供应。作用域借用不是官方同进程permit或服务/客户端写授权，真正补偿仍需其余完整证明。

.adg/evidence/0.2.24/official-client-coordination-r1/保留原假设、实际原件frame、来源/binary摘要、完整结果及manifest；自有helper/测试进程已空，精确新协调DB排他取得/释放后清理，副本runtime/profile/binary/重复回执移除，无日常配置或官方包改动，未出新.app/DMG。下一批继续原生客户端写前核对/持久恢复/回退并接Start/Stop的原件/阶段/服务代次；config/Grok/Agent、新目录缺席、WindowsW0/W1/W2、NAME、System代理及发布/固定候选仍未完，整版目标不关闭。

### 2026-11-11 Mac客户端可恢复机械发布及中断验证

mainfc0361bc1增加macos_client_restore：按四阶段独立摘要/共享内容判定冻结唯一plan；原字节/ACL/xattr暂存并保留原后像，实际native交换/不覆盖移动、整批先查未知、逐项读回、显式Complete/Rollback及terminal固定决定。13项Mac实际文件系统反例通过，其中真实子进程exit91覆盖11个intent/native/step/archive/terminal中断边界；同plan重新载入仍回原字节/缺席/保护，外部目标/归档/替换inode/别的plan均拒绝。最终源码/binary摘要、编译/格式/全目标严格Clippy通过，证据与首错 .adg/evidence/0.2.24/macos-client-restore-r1/；自有测试原件/stage/archive/程序与重复回执清理，无日常配置/官方包改动或服务/业务请求。详[Mac机械恢复合同](0.2.24-contracts/macos-client-restore-r1.md)。

生产MutationPermit没有构造器/签发器，未接Stop，不将前述三文件未还原改为通过。核对后rename不是预期inode的内核CAS，外部helper租约可在native工作中途过期；必须共同证明活跃写者排他/原生发布生命周期，不能用开始时check或增加预算代替。跨卷归档EXDEV、原flags非零及新CODEX_HOME目录缺席仍保留并拒绝，不冒称完整范围已支持。下一步继续这两处许可/生命周期闭合及config/Grok/Agent/外层持久绑定，再做受影响真实恢复链；WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新官方发布验证和固定候选保持未完，目标不关闭。

### 2026-11-11 Rust同进程保持官方协调锁直到工作栈退出

mainc6bda9595增加native_client_coordination，将机械恢复prepare/apply的锁参数从外部helper Lease切至Rust原生NativeLease。先由原样官方2.99做真实准入/初始化，带原数据库dev/inode、实际官方路径、原owner journal不透明摘要及原selected身份；Rust交接时按原claim/N/K重新取得SQLite事务并完整核对窗口内未变。无另建namespace/schema或busy重试。SQLite连接与源fd由同一栈持有，过期只判失败、不从外部提前解锁；先关SQLite再关所有原生pin，VFS HAS_MOVED与NOFOLLOW核对真实数据库连接代次。详[同进程协调合同](0.2.24-contracts/native-client-coordination-r1.md)。

14项相关定向（13项实际检查＋探针入口）及1项原样官方真实交接验证通过，最终编译/格式/全目标严格Clippy通过。真实另一OS进程在原件快照读取、旧helper2秒窗口之后仍被三锁排除，回调结束后可重获；100ms过期栈仍持锁、部分取得/竞争/旧权限/归属变化及VFS替换拒绝均证明。首轮错误新夹具被官方正确拒绝native残留，原错及独立官方CodexCoordinatorLegacyAmbiguousError证据保留；只补实际官方新协调器初始化前置。原8.697秒成功后去除中间重复全镜像扫描，仍保持整个操作前后两次完整核验及同selected身份，最终新HOME/原程序复用检查6.900秒（含人为2.1秒工作），不当真实Stop10秒通过或减法估算通过。

证据.adg/evidence/0.2.24/native-client-coordination-r1/保留三个轮次、源/binary、原件和manifest；自有进程、独立排他读回后的精确协调DB、程序/profile/binary与重复回执清理，无日常配置/官方包或服务/业务请求。外部helper提前释放窗口本批闭合；非合作写者check/rename竞争、正常退出/代次MutationPermit签发及Start/Stop补偿接线仍未完，生产签发器继续关闭。config/Grok/Agent/全树、活跃刷新/新目录/跨卷及原预算、WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新发布验证和固定候选均保持，不发版、不关闭目标。

### 2026-11-12 发布竞争实测修正与突退内容保全

mainbfb264eb8复现并修正最后state/check后真实native/archive gap的迟到写者：首轮3项均失败，陌生catalog/cache/stage被移出原路径。现先以原plan摘要＋实际双对象原字节/身份/保护留create-only私有conflict，再在双对象未变时尝试一次无损交换或EXCL归还；二次变化不覆盖/不重试/不产生成功outcome。归还成功操作仍失败，同plan不把未知内容接管。详[发布竞争保全合同](0.2.24-contracts/macos-client-restore-race-r1.md)。

19项最终算法原生组及1项新增conflict回执后实际exit91测试通过（去重20项），含实际另一OS进程在最后检查后替换、新目标再写、stage/archive损坏、原11个发布中断、ACL/xattrs原件保持；新退出保留陌生字节且重开Complete/Rollback均拒绝，无成功终态。为真实回执留原字节单独定向补验外部写者，不重复全部官方供应。最终编译/格式/全目标严格Clippy通过，source/binary准确摘要、首轮失败和合成实际conflict回执 .adg/evidence/0.2.24/macos-client-restore-race-r1/。所有自有进程/夹具/stage/archive/conflict及重复临时回执清理，未动官方/日常配置或运行服务/业务请求。

这不是内核CAS或任意持续写者线性化，不能保证所有瞬间文件位置不变；只对已验证边界保全内容/有条件归还、未知保持失败。生产MutationPermit仍无签发器，Stop未接，原三个真实文件恢复失败不转通过；下一步补正常服务退出/真实代次及完整恢复编排，保持原30/5/10预算。config/Grok/Agent/全树、新目录/活跃刷新/跨卷及flags、WindowsW0 HOME/Profile/W1/W2、NAME、System代理和发布/固定候选保持未完，目标不关闭。

### 2026-11-12 Mac普通Stop要求原进程内核正常退出事件

main57c201bd7新增macos_process_exit并接入stop_lifecycle：官方Stop前绑定已核对的原PID/uid/出生秒微秒，kqueue注册/二次代次核对；只接受原订阅真实NOTE_EXITSTATUS的exit0，错代次/信号/非零/注册前缺席及迟到事件拒绝。原官方HMAC/能力/记录、HOME其它进程/manager/port及绝对预算保持，原进程消失不能单独正常Stop通过。详[Mac正常退出合同](0.2.24-contracts/macos-normal-exit-r1.md)。

7项真实内核测试（含同uid非直接子进程、exit7/SIGTERM/原deadline拒绝）与10项既有归属回归通过，编译/格式/全目标严格Clippy通过。实际官方2.99隔离生产Start4433ms/Status1092ms/Stop2392ms通过原30/5/10秒，当前生产日志明确original-native-exit confirmed code=0；整轮仍exit101/accepted=false，config.toml已还原但catalog/cache/audit仍未恢复，原完整断言/原字节各阶段保留，不以局部服务Stop关闭全客户端问题。

证据.adg/evidence/0.2.24/macos-normal-exit-r1/，原包/source/binary摘要及manifest保持；测试/运行/launcher及非直接子fixture精确PID均退出，完整TCP表原端点缺席，精确协调DB独立排他取得/释放后删除；自有副本程序/profile/binary与重复回执清理，无官方/日常配置改动或真实业务请求，未出新.app/DMG。下一步将真实退出代次与启动前原件/phase/plan/上下文持久绑定，补生产恢复许可/全链；单PID证明不代替后代/所有监听/完整恢复，WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新发布验证及固定候选仍未完，目标不关闭。

### 2026-11-12 普通Stop结果保留原代次引用与原预算

main5a5732524将stop_lifecycle内部提取为verified结果，保留原runtime/profile/HOME/D路径、checked_owned运行记录摘要、Mac内核NormalExitProof及原有效Stop截止时间；既有包装API/DTO保持。AlreadyStopped分支无新记录/退出权限，不可被当startup原件授权。后继引用比较必须同runtime Arc/上下文/D/hash/PID出生代次，原deadline已过不能续开10秒窗口。详[完成Stop引用交接合同](0.2.24-contracts/completed-stop-binding-r1.md)。

4项内部引用测试与10项既有归属回归通过，编译/格式/全目标严格Clippy通过。合成外层结构只检引用比较，子进程内核exit0是真实测试事实，但不冒充官方服务Stop发行/恢复权限；AlreadyStopped用明确FakeHost只证明不能授新权。上轮官方4433/1092/2392ms保留原57c201bd7来源，不因结构重构无假设重跑供应或冒当前整组验收。证据.adg/evidence/0.2.24/completed-stop-binding-r1/含source/binary/manifest及清理，自有进程/binary/目录/重复回执移除，无真实服务或业务请求、日常/官方改动。

本轮尚未持久绑定启动前Originals与完成Stop结果，自动客户端恢复仍未接；原三件恢复失败不转通过，MutationPermit签发器仍关。下一步接startup会话/原件/phase/plan独立摘要与真实正常停止结果，目录代次及完整后代/监听/config/Grok/Agent/全树等仍要证明；WindowsW0 HOME/Profile/W1/W2、NAME、System代理、新发布验证和固定候选保持未完，目标不关闭、不发布。

### 2026-11-12 持久启动原件/实际就绪及正常Stop会话绑定

main4b6943ca7新增macos_client_session：NativeLease上下文一致且真实Stopped才捕获Original；create-only scope绑定D/profile/HOME/client目录原生身份、CurrentSelection及原件摘要。Ready必须同原保护记录/原生进程managed image/argv/HOME/出生代次和新鲜官方签名，startup-live只存摘要/PID/port/uid/birth无secret。独立scope/phase/live引用重开后PreStop重验真实进程，CompletedStop需同原runtime/context/D/hash/Generation/原deadline，封Stopped前后运行记录明确缺席。普通应用execute_lifecycle自动选择/持久活动Session及MutationPermit恢复工厂仍未接。详[会话绑定合同](0.2.24-contracts/macos-client-session-r1.md)。

本轮发现并另以d85f9d1d3修复helper首次成功N初始化rollback成空DB：只对原缺席/0字节、官方构造准入且成功三锁工作、gen0/null未变的首次事务调用原官方commit，已有/失败rollback保持，无复制schema/旧状态接管。另修Session有效客户端目录检查，actual cleared生产Mac环境为canonical(profile/.codex)，不能只信给协调器的另一个路径；新目录创建等正式范围仍待实现。首次readiness失败、第二轮nativeResidue及精确0DB/官方拒绝原件保留，两次失败私有服务仅确切image/record/双HMAC同连接官方Stop清理，不补通过。

7项Session引用/未知反例＋15项NativeLease相关（含探针入口）及最终1项原样官方实际绑定整链通过，最终编译/格式/全目标严格Clippy通过。attempt3启动含绑定13444ms/停止7695ms为较窄来源；新增Ready前后原生managed归属门后attempt4实际13675ms/7833ms过原绝对30/10秒，独立原引用/六件frame/准确source/binary摘要保持。三件原字节恢复仍false，accepted=false，只标绑定链通过，不将测试宿主调用冒软件自动恢复入口。初次出生秒极值测试与PID字符串Clippy首错保留。

.adg/evidence/0.2.24/macos-client-session-r1/保留四轮、官方有效路径/受限三键上下文/N拒绝及初始化事实、完整scope/live/phase/references和manifest；自有进程/精确排他读回后新协调DB/副本程序/profile/binary/重复回执清理，未动官方/日常配置/正式安装或业务请求。后继继续自动当前会话写者/持久索引、崩溃恢复与客户端恢复工厂；config/Grok/Agent/全树、新目录/活跃刷新/跨卷/flags、WindowsW0 HOME/Profile/W1/W2、NAME、System代理和发布/固定候选均未关闭，目标不关闭。

### 2026-11-12 应用启停自动登记当前原件会话

main595850a24将CurrentSession Index和StartupSession接入真实Mac官方2.99 ControlledProcessRunner的execute_lifecycle：新Start先Preparing/Captured、原ensure/ready后独立提交Ready；Stop自动按同原引用PreStop、真实普通Stop/kqueueexit0、Stopped封存和独立提交。索引schema1、D/profile/HOME/client原目录绑定，整个动作原生锁排他、每阶段独立SQLite FULL事务/原文档revisionCAS及append事件读回，原件不会因观察者消失/进程崩溃被默默重拍。详[应用当前原件会话合同](0.2.24-contracts/macos-current-session-product-r1.md)。

实际产品入口本轮Start15616ms、Status1108ms、Stop8591ms，原30/5/10秒不变；自动indexedReady/Stopped=true、索引revision5及五次已提交状态/原件引用逐件真实。文件补偿工厂仍未接，动作准确返回client_original_restore_pending（服务已停止、原件恢复待完成）且整轮原完整断言保留exit101/accepted=false；catalog/cache/audit仍false，不宣称恢复已完成。当前Stopped未补偿会话阻止再拍新基线及Restart，属于工程待收口状态，不能作为发布体验。缺引用/PreStop不确认仍先按原普通受认证Stop停止服务、再保留会话错误，不创建假原件。

9项索引原生/进程互斥/突退/引用与保护反例、1项保留产品现场最终锁强化重开检查、2项来源回归及9项动作DTO通过，最终编译/格式/全目标严格Clippy通过。完整native轮后只有action锁kind/flags/保护强化及新负例，source/binary分别保存，未无假设再跑整轮服务；Stopped现场以最终代码独立读回并拒绝基线覆盖。证据.adg/evidence/0.2.24/macos-current-session-product-r1/保留原SQLite/事件原字节、独立原引用匹配六件frame、产品前/后像和trace，精确NDB排他释放、原端口/nativeTCP及自有进程清理证明；所有副本程序/profile/binary/索引与重复回执移除，无日常/官方改动或业务请求。

下一步继续真正Stopped原会话许可/自动补偿、成功关闭后才放行新Start，以及崩溃显式恢复/活跃刷新/首次client目录/跨卷/flags、全部config/Grok/Agent/全树与后代监听证明。WindowsW0 HOME/Profile/W1/W2、NAME、System代理及发布/固定候选仍未完，目标不关闭、不发布。

### 2026-11-12 原Stopped会话恢复计划准备及独立引用

maind47a431dc在实际Stop同NativeLease/原deadline内封Stopped后，按原四阶段冻结恢复计划并提交独立id/hash至索引RestorePrepared/revision6。未签发MutationPermit、不执行apply；本轮草拟单PID许可在复核不足以证明完整集合后撤回，最终恢复核心无变更。独立启动base路由候选按官方slug编码/碰撞拒绝，live保存原生观察父代次仅供audit辅助，不冒完整retained进程集。索引schema2拒绝schema1，无迁移。详[恢复计划产品合同](0.2.24-contracts/restore-plan-product-r1.md)。

21项Session/Index及20项既有恢复回归通过，实际Mac四阶段只读内容回放通过、原样官方Schema/codec纯接口核对，编译/格式/全目标严格Clippy通过。首轮真实Stop成功但无plan，定位官方省略fastRows补true被误判未知；仅接受合法布尔对base声明的等价，实际Ready未声明Fast/effort行仍拒绝。保留第一轮Stopped/revision5，不覆盖失败；第二轮新profile复用完整校验原包，真实Start13413ms/Status1105ms/Stop含plan8491ms过原30/5/10秒，恢复plan/原件stage与独立索引/phase摘要匹配且可重开。三件恢复仍false，整轮exit101/accepted=false、动作pendingRestore不改。早期工具/fixture错误的原stderr未全捕获边界及隔离TMPDIR修正明确留证。

.adg/evidence/0.2.24/restore-plan-product-r1/保留原索引/phase、最终plan/preparing/stage原字节及独立引用、两轮产品trace和manifest；完整TCP原端点/自有进程缺席，精确协调DB排他取得/释放后删除；全部副本程序/profile/binary/索引/stage和重复回执清理，无日常/官方改动或业务请求，未出新.app/DMG。下一步继续完整集合/监听/原件写者许可再接apply和成功关闭，通用图/别名/Fast、config/Grok/Agent/全树、活跃刷新/新目录/跨卷/flags及显式恢复均未关闭；WindowsW0 HOME/Profile/W1/W2、NAME、System代理和发布/固定候选保持，目标不关闭。


## 2026-11-12 Mac停止变更事件的原生拒绝

mainafa4fca03修复单PID退出证明遗漏：订阅后fork/exec即永久拒绝，即便随后exit0或事件与退出合并，不能重复查询转成通过。两个原算法反例真实失败；首次正例夹具的初始exec时序错误保留，改为明确就绪握手后最终10项通过，格式/全目标严格Clippy通过。详[原生变更合同](0.2.24-contracts/macos-exit-events-r1.md)。Darwin不支持NOTE_TRACK，当前不是完整retained进程集合，MutationPermit仍未开放。

一轮受影响原包链Start15664ms/Status1119ms/Stop含计划8609ms，原30/5/10秒保持；原内核exit0及RestorePrepared/revision6、独立plan/阶段摘要核对。实际三件恢复仍false，整轮原assert保留exit101/accepted=false；无全量重跑。证据.adg/evidence/0.2.24/macos-exit-events-r1/含失败/最终/trace/原索引和manifest，原端点缺席、新协调DB排他取得释放后及全部自有程序/profile/binary/stage/重复证据根已清理。完整集合/监听/写者和实际恢复、全树/通用图/显式恢复及Windows/NAME/System代理/发布验证/固定候选继续；不改日常配置/官方包，不发布或关闭目标。


## 2026-11-12 服务停止观测与恢复失败的产品分离

main532298981修正真实Stop后错误回退Running的结果：CompletedLifecycleStop独立签发动作时点观测；恢复未完成返回stopped/result=null/clientRecoveryRequired及三动作门槛false，普通错误码无该权限。前端当前实例保持独立恢复标记，失败Restart不冒启动失败，晚到stopped轮询/未知动作不清恢复义务；概览及端口入口禁用相应动作，后继真实运行服务仍可普通Stop。详[停止与恢复观测合同](0.2.24-contracts/stopped-recovery-observation-r1.md)。重新打开/托盘/持久状态快照、显式恢复入口与实际apply仍未完成，不能将此批当全入口验收。

13项后端定向/59项前端、类型/编译/格式/严格Clippy通过。原包Start15616ms/Status1104ms/Stop含计划8515ms过原30/5/10；原内核exit0及RestorePrepared/revision6、独立phase/plan/stage读回，stoppedRecoveryObservation=true。实际三件恢复仍false，整轮原assert保留exit101/accepted=false。早期网络全禁夹具首错、前端旧fixture失败及改动前生产源码复核均留证，边界仅增加loopback/明确idle回执，测试修正独立main8ef251554。证据.adg/evidence/0.2.24/stopped-recovery-observation-r1/含原件/摘要/manifest与清理；全部自有程序/profile/binary/stage/协调DB及重复证据根清理。完整原生恢复、Windows/NAME/System代理/发布验证/固定候选仍按原范围继续，未改日常配置/官方包或发布。


## 2026-11-12 原索引只读观察与重建后的恢复门槛

mainb92dc31f4新增受保护原索引的immutable只读观察，先共享action flock再接SQLite，busy/遗留日志/旧结构/绑定或权限未知不当无会话。真实跨OS进程探针证实同进程busy观察不释放原SQLite写锁；遗留原件但索引/引用丢失时读侧未知、写侧不重建基线。状态来源显式D，独立client_recovery枚举不由官方JSON/缓存/失败清除；采集器、查询/IPC、原生托盘/菜单与UI同门槛，新实例仍required，真实运行状态保持独立。详[只读恢复观察合同](0.2.24-contracts/recovery-readonly-view-r1.md)。Windows原生观察未接明确unknown，不冒Mac或整链通过。

57项后端相关/64项前端、类型/编译/格式/严格Clippy通过，早期编译/类型/测试初始化错误原输出保留。真实第1轮Stop正常exit0但索引revision4、无plan，reopened794ms正确required；内部错误原被pending映射隐藏，未确认根因。补固定phase/code诊断后新profile复用完整校验原镜像，第2轮Start13611ms/Status1148ms/Stop9904ms、RestorePrepared/revision6及独立phase/plan/stage核对，reopened825ms；原30/5/10秒未变，Stop贴近预算，未给apply另开预算。首错未再现但不称已消除；两轮实际三件恢复仍false，原assert均exit101/accepted=false。

证据.adg/evidence/0.2.24/recovery-readonly-view-r1/含两轮原件/锁探针/摘要及manifest；原端点、自有镜像/测试和全部精确新协调DB/程序/profile/binary/索引/stage/重复证据根清理。后继实际apply/完整进程集合监听写者/显式恢复及全树/通用图/新目录/跨卷/flags保持；Windows/NAME/System代理/发布验证/固定候选不删除，目标未关闭，未改日常配置/官方包或发布。


## 2026-11-12 状态路径与实际端点事实修正

mainf5fc1fef0修正上批真实快照暴露的config.json被当HOME、停止仍显示保存端口：核对原官方status.ts，明确paths.config为文件而listen.port不等于运行监听。生产采集器以启动时固定D/H覆盖状态payload路径，并在成功/去抖/失败中保持；纯文件路径不猜HOME。只有明确停止/未发现时清运行port/PID，未知沿用不额外推定缺席，配置保存端口不改。详[路径与运行端点合同](0.2.24-contracts/status-context-paths-r1.md)。

32项相关检查/编译/格式/严格Clippy通过；真实隔离子进程命令夹具消费生产Source/collector，明确不是官方原包服务验收。本轮只核对原tgz源码及前批真实快照，未再启动整套服务。旧test把config文件当HOME的错误断言首错保留、更正后单项通过，其它断言未删；launcher相对路径错误仅原工具输出留证的边界如实记录。证据.adg/evidence/0.2.24/status-context-paths-r1/及manifest、隔离拒绝/清理完整，自有测试/命令/profile/binary/重复根清理。原生产品reopened路径/端点断言加强但本轮未执行，实际恢复及revision4首错仍未闭合；Windows/NAME/System代理/发布验证/固定候选继续，目标未关闭，不发布。


## 2026-11-12 npm原生静态代理与旧配置覆盖

main526007952将系统静态代理快照接到安装pack/install及面板npm view共用受控入口；Mac SystemConfiguration/Windows WinHTTP只读，None/Manual不查系统，离线不解析系统代理，显式凭据仍原临时0600文件且环境快照不覆盖它。原包npm不支持两协议独立代理/PAC/CIDR/简单主机例外等语义，当前明确拒绝不可等价配置，不能把此拒绝当完整System已做完。详[npm静态系统代理合同](0.2.24-contracts/npm-system-static-r1.md)。

真实npm暴露Direct的空环境被忽略而使用旧npmrc代理，修为false/null及明确空noproxy覆盖；18项相关检查、5场景实际loopback npm路由/原配置不变、实际Mac只读查询无代理分类、编译/格式/严格Clippy通过。Windows只做最终共享adapter的MSVC目标独立编译（其它DTO结构stub），未实机或整包通过。首CF类型/lint、测试server非阻塞时序及Direct产品首错原输出保留，末轮因Direct/bypass反例改变而重验五路，未删失败；0test误筛选不计通过。证据.adg/evidence/0.2.24/npm-system-static-r1/含摘要/manifest/清理，自有工具副本/进程/profile/npmrc/cache/binary/跨编译target和重复根移除，无日常/系统代理/官方包改动，无真实registry安装或业务请求。

完整System复杂路由、Windows新贡献/原生补验、NAME、实际恢复/完整集合监听写者与revision4首错、官方发布验证/固定候选保持未完；本轮未跑完整服务或出新包，不发布、不关闭目标。


## 2026-11-12 npm代理传输及原生字典补证

main4a7980904在原静态代理支持范围内补固定npm HTTPS/HTTP CONNECT与SOCKS5h HTTP/HTTPS远端DNS的实际协议检查，strict-ssl=true，使用本次合成CA正例及不信任CA负例（原npm UNABLE_TO_VERIFY_LEAF_SIGNATURE/exit1）；三个正常路径exit0、原npmrc不变。Mac原读取解析提取到同算法macos_map，两项真实CF字典类型/共同代理/IPv6 SOCKS/例外及拒绝规则检查通过，未修改系统代理。详[代理隧道与原生字典合同](0.2.24-contracts/npm-proxy-transports-r1.md)。

3个去重函数、内部4种新传输场景以及编译/格式/严格Clippy通过。首lint读字节数错误保留并用read_exact修正；最终源码提取及负例加强后，仅重验受影响的4传输和新字典项，没有plain HTTP五场景/完整服务无假设重跑。证据.adg/evidence/0.2.24/npm-proxy-transports-r1/保存准确源/binary/公共证书/摘要/manifest；原测试工具/profile/cache/binary/CA及server私钥、代理/registry/Node进程与重复根清理。没有官方/日常/系统代理改动或真实registry安装/业务请求。完整System/PAC分协议复杂例外、Windows、NAME、实际恢复/完整集合监听写者与revision4首错、发布验证/固定候选继续，不发布、不关闭目标。


## 2026-11-12 原件帧绑定Codex树清单

main94ed6dc11将描述符相对.codex树身份/保护/普通内容SHA清单写入每阶段schema2帧，独立原摘要因而绑定完整观察范围；tree与原三件字节/身份互相核对，普通内容哈希流式zeroize、不把哈希冒字节备份。固定claim与精确3sidecars为元数据协调条目，不开关其SQLite inode、不增恢复/删除权限。主线实际仍缺Grok/外部Agent、全树可恢复字节和写回许可，边界16MiB文件/128MiB总等超界拒绝，不能缩范围称完成。详[树清单合同](0.2.24-contracts/client-tree-inventory-r1.md)。启动固定阶段诊断独立main51c1f7dbf。

88项相关回归及最终8树增量、编译/格式/严格Clippy通过。真实其它进程目录替换及活跃SQLite锁保留通过，旧帧不补齐；开发目录link/schema/错误阶段assert首错保留。首两轮原包Start前原件失败、索引revision1，阶段诊断和空claim BEGIN EXCLUSIVE实际journal反例共同定位sidecar过度拒绝；修正后新profile第三轮Start13918ms/Status1087ms/Stop含计划8402ms，原30/5/10不变，四schema2 phase原摘要/tree/plan/stage读回、RestorePrepared/revision6。实际三件仍未还原，完整断言保留exit101/accepted=false；新collector仍Required但运行Loading/reopenedGate=false，未归因、不得当已过，前批revision4首错亦保持。

证据.adg/evidence/0.2.24/client-tree-inventory-r1/含三轮原件与失败/各来源/manifest；原主端点、自有镜像/测试与全部精确新协调DB/程序/profile/binary/索引/stage/旧日志/重复根已清理，日常/官方/业务无改动。后继可恢复全范围字节/完整集合监听写者/显式恢复及reopened故障继续；Windows/NAME/System/发布验证/固定候选不删，目标active，不发布。


## 2026-11-12 停止诊断准入与原字段检查修复

main70d2a2778闭合新collector的Required但Loading回归：展示映射正确移除config文件→HOME及Stopped旧PID后，no-record准入仍依赖清洁映射，既误拒正常报告又可能漏拒原残留PID。现直接按原字段角色核对所有HOME/config路径及原ready/running/PID，正常own stopped可读、矛盾别名/非nullPID/其它用户运行仍拒绝。原认证记录、来源及无记录前后检查/5秒预算保持，不从状态字符串授恢复权限。详[停止诊断准入合同](0.2.24-contracts/stopped-cli-admission-r1.md)。

修改前既有3项准入1过2失败；修后31项相关/编译/格式/严格Clippy通过。新的实际原包Start15245ms/Status1092ms/Stop含计划8412ms过原30/5/10，新collector782ms明确Stopped+Required、D/H及空运行端点正确、三动作false/reopenedGate=true；独立RestorePrepared/revision6/phase/tree/plan/stage吻合。修复这个新回归，不外推旧revision4首错已消失；实际三件未恢复、原assert仍exit101/accepted=false，完整恢复许可未开放。

证据.adg/evidence/0.2.24/stopped-cli-admission-r1/及manifest保留原失败与新快照/原件/准确来源；原主端点、自有镜像/测试/全部精确新协调DB/程序/profile/binary/索引/stage/重复根清理，未改日常或官方/业务无操作。完整可恢复范围/进程集合监听写者/Grok外部Agent/显式恢复等原范围及Windows/NAME/System/发布验证/固定候选继续，目标active，不发布。


## 2026-11-12 阶段树字节归档与原生准入计时

main39fe025d5将schema3阶段帧引用的每个.codex普通文件保存为原attempt内0400/create-only/SHA对象，原FD/保护/长度/哈希/全树前后核对、对象及父目录fsync；同内容去重，已有对象也重新核对同步，不修补缺失/损坏原件。原阶段读取可独立取回config/Agent字节，重开必须所有原对象匹配。schema1/2不迁移。协调claim/sidecars仍metadata-only，真实锁内归档外部probe仍busy。详[树字节归档合同](0.2.24-contracts/client-tree-bytes-r1.md)，完整Grok/外部Agent/写回及许可尚未接，不能缩范围。

首原包因client_coordination_expired在归档前失败，源码/时钟检查确认spawn导入被误算为Rust helper借用窗口这一过度拒绝（不能凭单错误码认定唯一原因）。独立mainda2fa5939将私有native准入固定为仅取metadata、正常released/exit0后再原生重获全部锁；公开借用Rust两秒、物理helper两秒和原30/5/10/完整性/busy门槛保持，helper字节未改。99相关项及最终增量/编译/格式/严格Clippy通过，首错/来源不覆盖。

新profile第2轮真实Start13760ms/Status1105ms/Stop含归档计划9374ms，新collector805ms Stopped+Required、三门槛false。四schema3 frame/独立索引revision6/plan/stage及11内容对象SHA/0400/长度匹配，最终算法一致仅注释后改。实际三件仍未还原，原assert保留exit101/accepted=false，Stop贴近10秒，后继apply仍同预算。证据.adg/evidence/0.2.24/client-tree-bytes-r1/含匹配表及固定合成config，不输出动态令牌原字节；自有程序/profile/全部blob/索引/stage/binary/精确协调DB/重复根清理，日常/官方/业务未改。完整进程集合监听写者/实际补偿及原全范围、旧revision4首错、Windows/NAME/System/发布验证/固定候选继续，目标active，不发布。


## 2026-11-12 Grok原件范围与明确Profile接线

main6d34836ed新增schema4 Profile原生身份及.grok缺席/完整有界普通树状态，每阶段原字节归档后发布；缺目录不创建、晚到目录不改原缺席，错Profile/链接/无权限/缺档拒绝，不迁移旧frame。生产begin/open明确使用ProcessContext Profile，便利旧签名仅test夹具，不作运行路径fallback。树角色区分使Grok中同名claim文件仍普通归档，不偷用Codex协调例外。详[Grok原件合同](0.2.24-contracts/grok-originals-r1.md)。

87相关回归、编译/格式/严格Clippy通过，首测试权限字面量lint保留并等价修正。真实全新Profile含有效Grok原配置/已有备份/内嵌Agent：Start16473ms/Status1102ms/Stop含双树归档9813ms均原30/5/10内，Grok配置确实79→8655→79字节，原SHA恢复、备份和Agent保持，独立原档可读；四schema4 frame/Profile/索引revision6/plan/stage及15内容对象吻合，新collector801ms Stopped+Required。Codex三件仍未恢复、原assert保留exit101/accepted=false，不能把Grok窄证据冒整版/恢复许可完成；后继实际apply仍同原10秒。

证据.adg/evidence/0.2.24/grok-originals-r1/及manifest保留固定合成原件和匹配表/来源，不输出动态令牌原件；自己的程序/Profile/双客户端树/全部blob/frame/index/stage/test及精确协调DB/重复根清理，日常/官方/业务未改。完整进程集合监听写者/实际补偿/显式恢复、外部Agent和一般路径等原范围及Windows/NAME/System/发布验证/固定候选继续，目标active，不发布。


## 2026-11-12 原服务代次TCP监听核对

main1a7133e40用目标Darwin SDK C桥/完整FD两观察核对原UID/PID/birth的TCP IPv4/6监听，主运行port必须实际归属此代次；普通Stop后所有捕获端口都须在原TCP全局表明确缺席，次要端口/查询未知不通过。CompletedStop保留原监听Generation，缺监听不能靠单PID exit0补齐。18相关检查及编译/格式/严格Clippy通过，原预算/认证/完整性/无强杀不变；仍不是UDP/Unix/后代转移或持续写者证明，不开放MutationPermit/apply。详[TCP归属合同](0.2.24-contracts/macos-tcp-ownership-r1.md)。

首原包脚本错误使用旧binary089f（新cc依赖使实际产物7a），工作树快照不等可执行源证明；错误轮原件保留并明确不计新TCP验证。改为读取本轮build路径及匹配18项结果的SHA再执行，新profile完整核验同一官方镜像：Start13958ms/Status1138ms/Stop含归档9996ms，捕获4个不同TCP且原exit0后4项全部检查缺席，索引revision6/四frame/plan/stage/字节及新collector794ms门槛正确。原30/5/10保持，几乎无余量，实际补偿仍false/原assert exit101/accepted=false，优化关键路径仍必需，不能给apply另开预算。

证据.adg/evidence/0.2.24/macos-tcp-ownership-r1/及manifest保留首错/错误binary分类/有效来源/原件/端口数量trace；两原主端点、自有进程/全部精确协调DB/程序/Profile/blob/frame/index/stage/test及重复根清理，未改日常/官方或业务。完整进程集合/持续写者许可/实际恢复/显式恢复及Windows/NAME/System/发布验证/固定候选等原范围继续，目标active，不发布。


## 2026-11-12 停止协调完整镜像校验成本

main c025dd83d先加入固定阶段耗时，独立官方2.99隔离轮Stop10200ms（内部10196），停止后的协调5983ms中约4760ms在两端程序准入/完整镜像校验。main d7b4921b8新增read_complete：每次新完整inventory同时核对tree与六个关键角色摘要，替代两端同边界“关键文件读取+全目录再读”；current/manifest/入口/pending/链接/大小/执行位与原生锁/原件检查全保留，无缓存或官方包改动。详[停止校验成本](0.2.24-contracts/stop-admission-cost-r1.md)。

30项debug、同组30项release、编译/格式/严格Clippy通过。新Profile debug实测Start12287/Status1232/Stop9997ms，索引RestorePrepared/revision6；实际release优化级新Profile为18962/2295/10048ms，正常exit0/4项捕获TCP缺席但prepare-plan预算耗尽，索引Stopped/revision5。两轮新collector分别865/1406ms、Stopped+Required/三门槛false；三轮完整断言仍exit101/accepted=false，实际catalog/cache/audit未恢复。原30/5/10不变，预算仍未解决，不能拿单次减少或release编译通过代替整链通过；旧revision4首错亦未外推关闭。

证据.adg/evidence/0.2.24/stop-admission-cost-r1/保留原超时、全部轮次准确源码/build/binary、独立阶段/索引/plan/stage/35项内容引用及manifest。三个原主端点、自有镜像/进程均缺席；九个精确协调DB排他取得/释放后，三轮所有程序/Profile/blob/index/stage/测试副本和重复临时回执清理。未改日常配置或官方包，无业务请求、未出新.app/DMG或发布。后继预算、完整进程集合/持续写者/实际恢复及显式恢复/原全范围、WindowsW0/W1/W2、NAME、System代理、官方发布验证/固定候选继续，目标active。


## 2026-11-12 Mac ARM64摘要后端与停止预算余量

main17ac170c4仅在Mac ARM64给原锁定sha2 0.10.9启用上游CPU检测硬件后端，缺硬件仍软件回退；不强制全局CPU特性、不减少任何字节/完整镜像/角色摘要或HMAC校验、不改原预算/官方2.99。锁只新增sha2-asm0.6.4，原依赖版本保持；Windows目标特性无asm。先写假设的独立release宿主同256MiB摘要软件947ms/硬件98ms，与Python独立值/标准向量/HMAC一致；产品另验，不冒独立宿主依赖图为产品锁。详[Mac SHA-256后端](0.2.24-contracts/macos-sha256-backend-r1.md)。

45项debug及同组45项release、编译/格式/全目标严格Clippy通过。准确binary的两轮新Profile实际debug Start8623/Status230/Stop4775ms，release5546/222/3559ms；各原PID内核exit0、4项捕获TCP全缺席、四frame/索引RestorePrepared/revision6/plan/stage及35项内容引用匹配，新collector353/376ms Stopped+Required且三门槛false。原30/5/10不变，实际release窄样本为后续apply留下6441ms；catalog/cache/audit仍未补偿、完整断言仍exit101/accepted=false。前批release10048ms失败原证据不覆盖，不以本样本冒一般路径/大树/完整恢复预算通过。

证据.adg/evidence/0.2.24/macos-sha256-backend-r1/及manifest保留探针/真实CPU/上游源摘要/产品平台特性与源码、两原包轮原件及日志。两主端点、自有服务/launcher/test/probe镜像均缺席，6个精确新协调DB排他取得/释放后与全部自有Profile/程序/blob/index/stage/副本binary/独立host target和重复临时回执清理。未改日常配置/系统设置/官方包，无业务请求、新.app/DMG或发布。当前样本性能已改善，后继完整进程集合/持续写者/实际恢复与显式恢复/全范围、旧revision4首错、WindowsW0/W1/W2、NAME、System代理、官方发布验证/固定候选保持，目标active。


## 2026-11-12 原启动父进程与服务双正常退出

main e62fc8a35新增PairWatch，从已独立绑定Ready Live读取两代，在PreStop/官方停止前核对UID/PID/出生时间及父子关系并保留双ExitWatch；原服务Stop证明之后，两者都须本次内核normal0才可继续seal_stopped/prepare_restore。未退/非零/信号/fork/exec/错代次拒绝，原30/5/10秒和正常官方Stop/4捕获TCP等门不变，无强杀。PairExitProof不可序列化，不当完整进程集合/其它监听/持续写者许可；实际apply仍关。详[启动父进程退出](0.2.24-contracts/macos-launcher-exit-r1.md)。

40相关函数通过（含1辅助入口，39功能判定），编译/格式/严格Clippy通过。先前pair和旧nonchild临时根/private/var拒绝、helper回收lint首错保留；修正夹具，不降判定，旧测试单独main2fc3cb04a。新Profile当前原包一轮Start11317/Status231/Stop4947ms，原父与服务normal0真实trace、四frame/索引RestorePrepared/revision6/plan/stage及35内容引用匹配，新collector371ms Stopped+Required且三门槛false。catalog/cache/audit仍未补偿，原完整assert仍exit101/accepted=false；本批未重构release/app/DMG，不借前批release样本代签。

证据.adg/evidence/0.2.24/macos-launcher-exit-r1/及manifest保留首错/来源/原件；所有自有helper/Node/Bun/launcher/test镜像为空、原主端点完整TCP表缺席，3个精确新协调DB排他取得/释放后，全部自有程序/Profile/blob/index/stage/副本和重复回执清理。未改日常/官方或业务，无发布。完整集合/写者/实际恢复与显式恢复/全范围、旧revision4首错、Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 官方写者锁覆盖反证与L接入

固化官方2.99审计证明client/connect直接catalog写入取用户级L/C而绕过K，当前claim/N/K不足以排除该类写者。main2f3c7dbfd将helper及同进程NativeLease改为claim→N→L→K，原官方L通过实际OS用户运行根解析，schema2固定4锁，旧/缺L回执拒绝；不改官方或增加旧兼容。原权限/身份/pin/VFS/owner/镜像前后及deadline保持，故不靠布尔/单PID证明开放apply。详[官方写者排他](0.2.24-contracts/official-writer-exclusion-r1.md)。

17项相关函数/编译/格式/严格Clippy通过；实际官方K catalog/cache/audit mutator及L直接目录写入在原生持锁时busy/callback不执行/字节不变，释放后相同入口均写成功；伪造/已撤销许可拒绝，跨旧2秒窗口锁仍在，原生操作4059ms。前两新Profile因sandbox mode拒绝、test探针模块路径错误而失败，原件保留，经精确诊断/仅夹具修正后第三Profile完成，不计早期错误为排他通过。

普通原包新ProfileStart8784/Status234/Stop4996ms过原30/5/10；原服务与父normal0、4捕获TCP全缺席、四frame/独立索引revision6/plan/stage/35内容引用匹配，collector360ms Stopped+Required且三门槛false。实际catalog/cache/audit仍未补偿，完整assert保留exit101/accepted=false，不把L组件闭合冒完整恢复许可或候选通过。

证据.adg/evidence/0.2.24/official-writer-exclusion-r1/及manifest保留源锚点、原失败和真实writer原输出。四自有Profile/程序/进程/原件blob/index/stage/副本和9个精确新协调DB排他取得释放后清理；既有用户globalL的inode/字节/0600/owner与父目录0700保持且独立排他重获释放，不删该原文件或留sidecar。未改日常配置/官方包，无业务/新.app/DMG/发布。完整集合/持续写者许可/实际恢复/显式恢复/全范围及旧revision4、Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 当前停止会话首次实际三项补偿

main67b2513f4在原真实Stop/Ready父子normal0及同进程claim/N/L/K临界区中，由私有借用ArtifactRestoreAuthority接Pending实际Complete；每次文件变更前核对本Originals/原Live/两代退出/Stop摘要及同一原deadline、四锁/owner/VFS和运行记录缺席，前后新OS归属必须Stopped。原批并发拒绝/迟到写者内容保全/无覆盖归档与突退算法不改；不从布尔/磁盘outcome发许可。只完成计划授权三件，整会话终态继续Required。详[当前三项实际补偿](0.2.24-contracts/current-session-artifact-restore-r1.md)。

61项相关去重函数及编译/格式/严格Clippy通过；旧external-writer夹具临时根首错保留，同binary只定向重验失败1项，不重跑已过19项，不降断言。新Profile原包实际Stop7473ms（内部7467）过原10秒，原服务与父normal0及4捕获TCP全缺席，三项真实apply/readback、catalog原退休模型字节恢复、cache/audit原缺席恢复；7目标路径nativeClientRestored全true。四frame/索引revision6/plan/35内容引用及实际发布inode/归档后像/outcome逐项匹配。当前仍RestorePrepared/Stopped+Required/三门槛false，原完整assert保留exit101/accepted=false，不能冒整个Stop成功。

只读字节/模式整树诊断Grok无差额、Codex只见claim事务journal消失和新owner稳定锁；按已采用原文，两锁精确是CodexHOME的claim.sqlite及owner.sqlite，非外部N/K。额外独立竞争/释放/身份安全诊断通过但在Stop计时后，未当生产同预算全树/锁终态通过。下一步须把整树安全属性/两稳定锁释放及余下生命周期门接原秒表及持久成功关闭；不靠直接移除pending报成功。未构建本批release/app/DMG。

证据.adg/evidence/0.2.24/current-session-artifact-restore-r1/及manifest保留原件/失败/实际补偿；自有镜像/helper/原主端点缺席，3精确新协调DB排他取得释放后与自有程序/Profile/双客户端blob/index/stage/archive/测试副本及重复回执清理，既有globalL完整保留。无日常/官方/业务修改或发布。原全范围、旧revision4、Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 完整客户端读回与两稳定锁释放

main c5a24573d接TreeSnapshot/Grok原完整路径/原字节/Protection（ACL/xattrs/flags等）比较、仅精确claim/owner例外、独立Stopped/Original稳定ID核对；原四锁正常释放后两个已存在稳定锁按原pin/VFS重获EXCLUSIVE并正常释放，重读全树后create-only0400回执绑定原References/plan/outcome/完整快照SHA/两锁Identity。前后原Stop/父子退出/当前HOMEStopped保持，Restart外预算显式clamp原Stop.deadline，不续原30/5/10。详[完整客户端读回](0.2.24-contracts/client-full-readback-r1.md)。

首实际轮整树及锁release过；第二新Profile在协调held阶段拒绝/index revision4原失败保留。另确定实验旧release最终名先见空文件使helper51ms即拒绝，定位真实生产竞态；改目录FD内完整暂存fsync→无覆盖rename-excl→目录sync/身份读回，不降2秒helper上限。第二轮/旧revision4的更细首错仍未证同因，不自动关闭。新增2原子通知正反例、6整树/锁反例及复用相关组总47去重函数/编译/格式/严格Clippy过，全部首错留存。

最终新Profile Start7658/Status362/Stop8074ms，原服务与父normal0/4TCP全缺席、三项实际补偿/七路径原字节/完整树安全属性及两稳定锁释放在原10秒内，完整completion回执实际生成；另独立当前树原生快照SHA与回执逐字匹配，35内容引用/index/plan核对。当前索引仍RestorePrepared/revision6、collector348ms Stopped+Required且三门槛false，完整assert保留exit101/accepted=false；不把新增前置通过冒会话终态/完整进程集合/一般范围完成，本批无release/app/DMG。

证据.adg/evidence/0.2.24/client-full-readback-r1/及manifest保留三轮/确定race/原失败与独立最终树。三个原主端点、自有进程/镜像缺席，9精确新协调DB排他取得释放后与全部自有程序/Profile/双客户端blob/index/stage/archive/control副本及重复回执清理；globalL身份/字节/安全模式原样保留。Windows远端仍b2e89665d/0eef59a01。无日常/官方/业务修改或发布，索引终态/恢复/全范围/旧revision4、Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 会话持久终态与再次启动闭环

main00d01df24以schema3独立Index登记Restored+completionSha256，close只接真正ReadbackProof，绑定原References/plan/data/client/原deadline；再验原件四frame/Scope/Live/完成receipt/outcome后CAS提交并读回。观察端只读独立验证、损坏缺失Unknown，不扫描/迁移/自动apply。已核对Restored才能新ticket/递增全局revision，旧events/原件保留。原预算后才完成的索引提交安全降级回pending，不把超时留成启动许可。普通Stop实际完成后才返回成功，Restart共享流已接但本批未执行单独Restart验收。详[会话终态](0.2.24-contracts/client-session-close-r1.md)。

36去重函数/编译/格式/严格Clippy过；新增反例初同revision唯一键失败只修测试并保留首错。新Profile真实原样官方2.99第一次Start8842/Status226/Stop8646ms，索引Restored/revision7，七路径全恢复；重开1234ms Stopped+NotRequired/canStart=true。显式第二次Start6677/Stop9013ms、新ticket/Restored revision14、七原字节全true。每次normal0父子/4TCP缺席、补偿/整树/两锁/回执及终态均在原30/5/10内，原集成函数首次完整exit0/resultpassed；外层诊断acceptedfalse，不冒整版或release候选验收。

8个只读完整性场景：合法/坏receipt缺失/坏receipt/outcome/scope/live分别接受或拒绝，索引字节不改；自有fixture还原及完成后合法用户配置编辑仍可观察Restored，不重放旧恢复。独立索引核对14events、两套原件/Scope/Live/plan/receipt/outcome/content全部匹配，最终两树原生快照SHA与第二份完成receipt一致。只读helper后加不改产品算法，未重跑服务。

证据.adg/evidence/0.2.24/client-session-close-r1/及manifest保留来源/初失败/双会话和反例。两个真实服务主端点、自有进程/helper缺席，3精确新协调DB排他重获释放后，与全部自有程序/Profile/双套blob/index/stage/archive/control测试副本和重复回执清理，既有globalL原样保留。未改日常/官方/系统配置，无业务/发布；未出新release/app/DMG。一般范围/完整集合/显式恢复/Restart命令及旧revision4、Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 单次Restart与原Stop入口预算

main82e2d21f1将managed action入口一次建立outer/stop_deadline，Restart总30秒且从原入口计Stop10秒，覆盖source准入/load/PreStop/官方停止/退出/监听/实际恢复/整树/两锁/索引终态，不在较晚StopProof起点重新赠10秒。固定stop-phase trace留真实阶段耗时；共享平台调用一致收紧，Windows独占模块未改/未原生通过。详[Restart预算](0.2.24-contracts/native-restart-budget-r1.md)。

21项相关函数/编译/格式/严格Clippy通过；新增前置耗7秒仅余3秒的预算断言。新Profile原样官方2.99一次真实Restart：初始Start8529/Status232ms，Restart14996ms，内旧完整Stop阶段8449ms，新实际PID10551→10591/端口62448、Ready revision10；最终Stop9004ms（段trace8997）Restored revision14/七原字节全true，重开1298ms Stopped+NotRequired/canStart=true。原30/5/10保持，原集成函数exit0/resultpassed，外层诊断acceptedfalse，不冒整版或release验收。

证据.adg/evidence/0.2.24/native-restart-budget-r1/及manifest含准确源/binary、真实Restart原输出、两段预算、schema3索引14events/两原件各自frame/plan/receipt/outcome/content、最终两树原生快照SHA匹配。两个主端点/自有镜像/进程缺席，3精确新协调DB排他取得释放后与自有程序/Profile/双套blob/index/stage/archive/control副本和重复回执清理；原globalL完整保留。无日常/官方/系统/业务修改或发布，无新release/app/DMG。一般范围/显式恢复/完整集合及旧revision4、Windows/NAME/System代理/发布验证/固定候选仍继续，目标active。


## 2026-11-12 静态显示别名图的恢复声明

maine6619359d按未改官方2.99源码及pure4组实测核清：defaultModelAliases/defaultAliases及modelAliases改显示名，不改provider的磁盘catalogModelSlug；对外Fast行不是本链磁盘路由。declaration允许有界合法显示别名与省略默认布尔、跳过明确disabled渠道，仍只从真实provider键/selectedModels独立声明，非法类型/碰撞/动态等未知图拒绝。原exactReady集合/归属/四锁/补偿和预算不变，不抄官方aliasregex。详[静态别名路由](0.2.24-contracts/static-display-alias-routes-r1.md)。

27相关函数/编译/格式/严格Clippy过；pure无发现/网络调用并有sandbox拒绝探针。新Profile真实3模型、globalAlias=true/渠道省略/显式custom-sol/Fast=true：Start9054/Status232/Stop8872ms，启动前3真实slug与Ready逐项匹配，实际display为opus/custom-sol/g3p，无Fast/display别名被虚构为恢复路由。原父子normal0/4TCP缺席、7路径全还原、完整树/两锁及Restored revision7；重开1283ms NotRequired/canStart=true。原集成函数exit0/resultpassed，外层acceptedfalse，不冒一般图/整版，NAME剩余不随本批关闭。

证据.adg/evidence/0.2.24/static-display-alias-routes-r1/及manifest含固化官方源锚点/pure与实际scope-Ready显示对照、原件35引用/完整树/准确binary。3精确新协调DB排他取得释放后连同自有程序/Profile/blob/index/stage/archive/control副本/pure约414MB导入及src/重复回执清理，原主端点/自有进程缺席，globalL原样保留。无日常/官方/系统/业务修改、新release/app/DMG或发布。动态/一般选择与自定义组合图/全客户端/显式恢复/旧revision4等原范围及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 非空选择清单的动态发现恢复

main8fd4bbf45按固化官方retained-sync/build-entries非空selectedModels过滤源码，允许live=true/省略且有明确非空清单独立声明路由；仍只取启动前JSON，不学习发现/Ready/cache，不改原精确集合/上下文/锁/恢复/预算。空动态选择/null/非法类型和其它未知图拒绝。原算法明确选择仍None的失败先验已保留。详[已选动态路由](0.2.24-contracts/selected-live-routes-r1.md)。

28项相关/编译/格式/严格Clippy过。测试专用loopbackGET/models返回4个合成ID含1未选、静态models故意过期；第一新Profile未识别请求/0发现，Ready为空而恢复门正确拒绝（Stopped revision5），首错保留。只读官方request形状与夹具方法/path正规解析后，第二新Profile实际1次GET /v1/models、0其它/业务请求；Start6591/Status232/Stop8815ms，3声明与Ready逐项相等、未选/过期ID排除，七原件全恢复、整树/两锁/Restored revision7，重开1256ms NotRequired/canStarttrue。原函数exit0/resultpassed，外层acceptedfalse，不冒开放动态/一般图或整版。

证据.adg/evidence/0.2.24/selected-live-routes-r1/及manifest留旧算法/夹具编译/第一轮拒绝、准确源/binary/实际发现和选择对照、两轮frame/index各35内容引用、成功plan/receipt/归档原后像与最终树SHA匹配。两个服务+两个模拟发现端点、自有进程/线程缺席，6精确新协调DB排他重获释放后与两套自有程序/Profile/blob/index/stage/archive/control副本/源提取和重复回执清理，globalL原样保留。无日常/官方/系统/真实业务修改、新release/app/DMG或发布。开放选择与其它一般图/客户端/显式恢复/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 静态全选名单与禁用模型声明

mainee1d607bd按固化官方configuredIds与最终disabledModelKeys规则，空/省略静态选择使用models∪retainModels去重名单，models为空才补defaultModel；非空选择仍优先，开放动态仍None。disabledModels按官方完整slug编码等价过滤，原碰撞先拒绝再过滤，不借禁用隐瞒歧义，不影响另一渠道同ID。原exactReady/原件/身份/锁/预算保持，combos/custom等未知继续。详[静态全选过滤](0.2.24-contracts/static-all-selected-routes-r1.md)。

3个原算法失败先验保留，修后31相关/编译/格式/严格Clippy过。新Profile真实名单2+retain2含重复、全选空数组、禁用1项：Start9066/Status230/Stop8663ms，启动前声明与Ready都精确2启用路由，retain-only存在/禁用项排除；七原字节全true/完整树/两锁/Restored revision7，重开1247ms NotRequired/canStarttrue。原函数exit0/resultpassed，外层acceptedfalse，未重复无依赖旧验证或冒整版。

证据.adg/evidence/0.2.24/static-all-selected-routes-r1/及manifest保留旧失败/源锚点/准确binary与实际名单对照、四frame/index/plan/receipt/35内容引用及最终树SHA匹配。原主端点/自有镜像/helper缺席，3精确新协调DB排他取得释放后连同自有程序/Profile/blob/index/stage/archive/control副本/源提取/重复回执清理，globalL原样保留。无日常/官方/系统/业务修改、新release/app/DMG或发布。其它一般图/全客户端/显式恢复/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 普通customModels覆盖与新增路由

mainccb6d16d5按固化官方routed-gather/build-entries真实provider/modelId的custom覆盖/新增及最终选择/禁用语义扩展declaration。仅启动前有界JSON授权，同ID覆盖不增slug，静态全名单加入custom新增ID；选择/禁用仍优先，坏容器/缺渠道/重复custom/编码歧义拒绝，连未选/禁用碰撞也不豁免。元数据保持官方职责，无组合替代或能力降级。详[普通自定义路由](0.2.24-contracts/custom-model-routes-r1.md)。

2个原算法失败留证，修后33相关函数/编译/格式/严格Clippy过。新Profile静态custom覆盖base、加1原生ID、禁用另1：Start8550/Status240/Stop8578ms过原30/5/10；before-start与Ready恰2路由，Custom base/context196608/text/low-medium及Custom added/context262144/text实际匹配，禁用custom未出现。七原字节全true/完整树/两锁/Restored revision7，重开1261ms NotRequired/canStarttrue。原函数exit0/resultpassed，诊断acceptedfalse，不冒其它custom/一般组合图或整版，NAME剩余不关闭。

证据.adg/evidence/0.2.24/custom-model-routes-r1/及manifest保留旧失败/源锚点/准确binary/实际ID能力对照、frame/index/plan/receipt/35内容引用/最终树SHA。原主端点/自有镜像/helper缺席，3精确新协调DB排他取得释放后与自有程序/Profile/blob/index/stage/archive/control副本/源提取/重复回执清理，globalL原样保留。无日常/官方/系统/业务修改、新release/app/DMG或发布。其它一般图/全客户端/显式恢复/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 有限目标组合与公开路由声明

mainc12c8d030按固化官方comboPublicModelId与deriveComboCatalogModel规则，从原JSON给有限、已独立绑定全部target的组合声明canonical combo/id或合法公开alias；保持目标路由不改，禁止影子遮蔽/physicalcombo冲突/重复缺失嵌套target/非法alias/nativeAlias接管等未明图。最终原exactReady/原件/锁/退出/恢复/预算不变，元数据由官方派生，不用combo替代NAME。详[有限组合路由](0.2.24-contracts/finite-combo-routes-r1.md)。

原有效combo声明失败留证，修后35相关/编译/格式/严格Clippy过。新Profile2静态能力目标+多目标mixed+单目标Public/fixture+禁用Public/hidden：Start8689/Status227/Stop8701ms，启动前/Ready恰4真实路由，mixed131072/text/仅medium与公开alias196608/text-image/low-medium实际符合原目标交集。七原字节全true/完整树/两锁/Restored revision7，重开1264ms NotRequired/canStarttrue。原函数exit0/resultpassed，诊断acceptedfalse，不冒其它组合/原生图、整版或NAME完成。

证据.adg/evidence/0.2.24/finite-combo-routes-r1/及manifest保留源锚点/旧失败/准确binary与实际4路由能力交集、frame/index/plan/receipt/35引用和最终树SHA匹配。原主端点/自有进程/helper缺席，3精确新协调DB排他取得释放后与自有程序/Profile/blob/index/stage/archive/control副本/源提取和重复回执清理，globalL原样保留。无日常/官方/系统/业务修改、新release/app/DMG或发布。其它一般图/全客户端/显式恢复/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 显式恢复只读诊断前置

main2ce09eb97注册client_recovery_preview无路径IPC及前端严格类型API，用inspect_map在原shared action观察租约内核对索引/完整原件/Scope/Live/plan，不在锁外二次读取；固定DTO只输出阶段/revision/attempt及证据状态，未登记completion不收养，unknown不冒不存在。continue/rollback一律false，不自动apply，不把预览当OS退出或写许可。其它平台unsupported，尚无界面组件/执行器，不标显式恢复完成。详[恢复诊断](0.2.24-contracts/client-recovery-preview-r1.md)。

49后端+2前端合同/类型/编译/格式/严格Clippy通过，4新增用例覆盖不初始化/持久阶段/全程shared锁/损坏原件与Scope/缺plan拒绝/未登记完成无权限/输出无私密原文。原新夹具目录假设和Vitest返回mock清理首错保留，仅测试修正，产品断言不降。未跑原包/GUI或Windows执行，不重复无假设服务轮。

证据.adg/evidence/0.2.24/client-recovery-preview-r1/及manifest保存源/binary/首错/结果；4新增只为清理在精确自有TMPDIR重验后确认为空并删除，去重统计不增加；所有精确helper命令缺席，本批无外部协调DB/程序/Profile/官方或日常配置改动、业务、新.app/DMG或发布。Windows远端仍b2e89665d/0eef59a01。后继显式执行/确认/UI与其它一般图/全客户端/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 已完成补偿的显式metadata收尾

main317eb9dee新增原文件已完整恢复但index未关闭分支：preview绑定原Head/树/身份/plan/outcome/receipt指纹，request只原attempt/revision/digest；Tauri新显式finalize经操作/更新/storage门与Index.open_existing（不创建缺失锁/index/空库），重新准入完整镜像/OS当前HOMEStopped/原父子PID与port缺席及整树/两锁事实，才登记原终态。没有Pending.apply或从旧磁盘回执重建正常退出许可，不写客户端文件；旧receipt坏值拒绝。continue/rollback/UI仍未接，不冒完整中断恢复完成。详[显式收尾](0.2.24-contracts/client-explicit-finalize-r1.md)。

71去重后端+3前端合同/类型/编译/格式/严格Clippy过。新ownedProfile真实官方Start11864/Status240/Stop8425ms，实际七文件补偿/整树/两锁后故障seam在index关闭前拒绝，原动作exit101/failed/Required/index6保留。另进程最终源重新当前准入的显式metadata正例6631ms/index7 Restored，错attempt/revision/digest、外部config编辑和陌生runtime记录均拒绝保留，重复确认拒绝，原普通失败不改写。客户端文件未被重放覆盖；最终树原生SHA与原completion一致，三项归档旧后像与Stopped字节匹配。两阶段binary/源码边界明确，open_existing后加仅收紧显式路径，不冒旧binary含最终新补丁。

证据.adg/evidence/0.2.24/client-explicit-finalize-r1/及manifest保留原失败/index6和显式另动作/index7、所有来源/正反例/35内容引用/最终树。原端点/自有镜像/helper缺席，3精确新协调DB排他取得释放后连同自有程序/Profile/blob/index/stage/archive/control副本/重复回执清理，globalL原样保留。无日常/官方/系统/业务修改、新release/app/DMG或发布。部分中断继续回退/确认UI与其它一般图/全客户端/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 客户端收尾确认界面与状态闭环

main d14125821在诊断中心Doctor后接ClientRecovery真实组件/持久store，入口显式preview、仅原完整已补偿现场确认metadata收尾；关闭/重挂不重复提交，回执精确匹配attempt/revision+1/status，成功后新live同D/H且Stopped/NotRequired/可Start才解除原UIlatch。普通迟到状态仍不清原失败latch；未知/位置变化/刷新失败保留限制，成功回执可只读刷新，不重试mutation。部分continue/rollback不显示。原唯一总原型新client-recovery024模块与2行loader/css同步入口/文案/状态/560px，无真实调用。详[收尾界面](0.2.24-contracts/client-recovery-ui-r1.md)。

WebKit新QA真复现summary后Tab跳BODY，main ed63c511f修共享UiDialog每步Tab显式有序移动（仅top层/未消费事件），原失败焦点轨迹/截图保留。96前端相关含其它dialog消费者通过；最后文案/unknowncompletionguard增量40项/build过。Chromium和WebKit各15软件与原型mock场景过，键盘取消/焦点9步、明暗/900×600无溢出、执行关闭重开1mutation、refresh失败只读重试均证实；最终WebKit2受影响refresh场景增量过。一次错误原型路径失败保留、修真实路径再过，不冒早期原型已验。生产dist排除审计mock并含真正UI/IPC契约，类型/build过，未新建native包或拿mock代原生。

证据.adg/evidence/0.2.24/client-recovery-ui-r1/及manifest保存所有源/原型index观察SHA、原失败/截图/双引擎summary/生产构建。所有9轮browser/Vite及精确audit脚本/Vitest退出，无本批native服务/外部DB/日常系统或业务操作；只复制核验证据后清重复临时根，正常dist保留。prototype其它用户index/原型未改未提交，本批只自己的新模块/loader/css。无发布。部分中断继续回退/其它一般图/客户端/完整集合/旧revision4及Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 中断恢复整组现场核对与原预算

main82764eabc将RestorePrepared的3目标/暂存/归档、intent/terminal与原生身份全部只读核对，指纹绑定原plan/root，不收养外部修改或发部分重放许可；登记Restored后正常用户编辑不触发旧后像重核。新增收尾拒绝阶段/耗时诊断，无路径或私密内容。详[整组核对合同](0.2.24-contracts/client-recovery-review-r1.md)。

23原生事务含11真实exit91中断边界、21会话、6读回检查及最终格式/编译/严格Clippy过。新Profile原官方Start8771/Status225/Stop8795ms，7文件全恢复后按预设index关闭前中断，原失败/index6保留。新核对首收尾拒绝，定向trace确认close-index10624ms超原10秒；原失败目录保留。合并同观察内重复读取且保留全部不同事实门后，仅同保留现场收尾8898ms/index7 Restored通过，未重跑服务或已过反例；合法后续合成cache编辑保持Restored及原输入。诊断accepted=false，不冒整版或候选验收。

证据.adg/evidence/0.2.24/client-recovery-review-r1/及manifest保留两次收尾失败/准确来源/成功与两独立index、35内容引用/15blob/七文件核对。自有进程/原主端口缺席、3精确新协调DB排他重获释放后与程序/Profile/全部夹具清理，既有globalL身份/字节/权限保持。Windows远端仍b2e89665d/0eef59a01。无日常/官方/业务改变、新.app/DMG或发布；部分继续回退与原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方发布验证/固定候选继续，目标active。


## 2026-11-12 部分中断恢复显式继续与界面

main37e486fad接独立client_recovery_continue、原索引/plan/确认及三文件NativeLease借用权限，当前OS停止/原父子PID/port/manager及记录缺席，批次前后完整枚举、每次publication锁/代次/原字节/权限复核；不重建历史正常退出，不改官方或旧预算。计划之外全Codex/Grok原树及两稳定锁绑定，未知编辑拒绝；释放锁后整树/回执/index关闭才Restored。mainbc887457f接真实确认/关闭重开单promise/成功新live同D/H解除限制，唯一总原型同入口和三内存场景同步。详[显式继续合同](0.2.24-contracts/client-recovery-continue-r1.md)。

69后端相关、27前端及类型/build/格式/严格Clippy过。新Profile官方Start8654/Status229/Stop5662ms在首目标交换后按预设中断，cache/audit尚未恢复、原exit101/index6保留。独立continue错确认/外部编辑拒绝保持索引；首10195ms、去重复锁外preview后10272ms失败均保留，原10秒不变。定位全OS枚举被逐步重复后归批次前后且保留每步不同事实核对，仅同现场继续9528ms/index7 Restored、重复确认拒绝；无重新安装/服务重跑。独立完整两树SHA匹配完成receipt、35内容引用/15blob及7客户端文件全还原。Chromium7/WebKit7界面与原型场景、末次WebKit3受影响详情/互斥guard场景过，生产排除mock；不冒mock为原生或整版验收。

证据.adg/evidence/0.2.24/client-recovery-continue-r1/及manifest保留所有失败、独立index/帧/计划/原件和准确来源边界。自有进程/主端口缺席、3精确协调DB排他重获释放后和程序/Profile/全部夹具清理，既有globalL身份/字节/权限保持，无日常/官方/业务改变或新.app/DMG/发布。Rollback终态/确认、更早捕获阶段及原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方发布验证/固定候选继续，目标active。


## 2026-11-12 恢复回退独立终态与原对象重新准备

mainea78757ea新增原生rollback执行器/完整停止后像读回、decision绑定不可序列化证明与RestoreRolledBack索引终态；仍Pending/Required，不能用回退证明关闭Restored或重新Start。新增rearm只create-only发布新逻辑plan、引用原父plan SHA及同一批原生对象，不拍新基线、不复制stage、保留旧不可变Rollback/outcome/回执；父链有界/新索引先登记后才可显式Continue，未登记meta计划不收养、不产生未知客户端对象。main3de2c6370只接准确只读回退状态，不注册rollback/rearm产品IPC/按钮，产品确认交互待下一批。详[回退与重准备合同](0.2.24-contracts/client-recovery-rollback-r1.md)。

75后端去重函数通过，普通和派生plan各11个真实exit91发布中断边界通过；首测试assertDebug编译错误保留并仅改比较，最终格式/编译/严格Clippy通过。一个新Profile原官方Stop首目标交换后按预设中断，原exit101/index6保留；同现场新回退8130ms/index7，真实状态Required/三动作false；重准备9557ms/index8、另一明确确认Continue9434ms/index9 Restored，每次原10秒，无重装或服务重启。两阶段独立完整树SHA匹配各回执；旧Rollback回执逐字/摘要与index7一致，新派生完成回执匹配index9，35内容引用/15blob/最终7文件原字节核对，真实最终状态NotRequired/仅Start。28前端/生产build及软件原型WebKit2只读状态场景过，不冒原生执行已可从UI操作。

证据.adg/evidence/0.2.24/client-recovery-rollback-r1/及manifest保留初始失败和index6/7/9、派生计划/原件/完整回执与准确分阶段源。自有进程/原端点缺席，3精确协调DB排他重获释放后和程序/Profile/全部夹具清理，globalL身份/字节/权限保持；无日常/官方/业务改变或新.app/DMG/发布。下一批产品rollback/rearm确认接线，以及早期捕获/原一般范围/完整集合/旧revision4/Windows/NAME/System代理/发布验证/固定候选继续，目标active。


## 2026-11-12 恢复撤销与重新准备产品确认闭环

main026529269注册真实rollback/rearm命令，沿现有运行/更新/binding门调用原生执行器；Preview给continue/rollback/rearm各自摘要与能力，一次整组核对派生方向，不减少事实门或改原秒表。main522d042e1接同UiDialog明确撤销/重新准备/返回、在途关闭重挂单promise；rollback/rearm新receipt+新live同D/H/Required/三动作false及精确preview核对后才展示下一步，绝不清latch或自动Continue；最后另行Continue成功+新live才可Start。状态读取失败只重读，不重发mutation。唯一总原型同流程同步，未碰其它并行原型/index。详[产品确认合同](0.2.24-contracts/client-recovery-rollback-ui-r1.md)。

22后端session、44前端/类型/生产build/格式/严格Clippy通过；新增测试busy枚举首build失败保留，改既有applying后过。WebKit/Chromium各22软件原型场景通过，补三明确确认链与原型慢操作关闭重开后各12受影响检查过，明暗/窄窗无溢出、返回零写入、错误保留限制、前两步不自动恢复、最后新状态解除限制，pageerror为空。生产排除mock。原生执行器与预算未变，明确按摘要复用上批8130/9557/9434ms原生证据，不无假设重跑服务；未新建.app/DMG或冒签名候选GUI→IPC整链已过。

证据.adg/evidence/0.2.24/client-recovery-rollback-ui-r1/及manifest保存首错/准确源/原型观察摘要/四轮截图和复用源。全部browser/Vite/audit进程退出，unit TMPDIR空，本批无native服务/协调DB/日常或业务改动；重复临时根归档核对后清理。Windows远端仍b2e89665d/0eef59a01，无新回交，不跑W2。早期中断阶段/一般客户端和路由图/完整集合/旧revision4以及Windows/NAME/System代理/官方发布验证/固定候选继续，目标active，不发布。


## 2026-11-12 早期中断原件完整保持的原生收尾

main97bd68a53接Captured/Ready/StopPrepared/Stopped/带独立引用Failed且无plan的完整原件不变分支：原scope/frames/完整树与安全属性、最新frame两锁存在及原生身份/可选0-2锁真实EXCLUSIVE重获释放、当前完整官方镜像/OS停止及记录缺席后，create-only revision独立回执+Proof/CAS登记Unchanged；不写客户端、不重造旧正常退出、不从Preparing无引用或未知文件发许可。独立观察坏回执Unknown，合法关闭后用户编辑不回放；新启动须核对旧闭合引用。无新产品IPC/按钮，下一批接确认。详[早期完整原件收尾合同](0.2.24-contracts/client-early-unchanged-r1.md)。

真实新Profile原官方Start3283ms在Captured前置后按预设中断，服务未启动/客户端未改、原exit101/index2/旧canStarttrue回执保持。独立finish错确认/外部编辑/陌生runtime记录拒绝不改索引；正例2415ms/index3 Unchanged，独立完整两树SHA/7路径原字节及5blob/原scope摘要吻合，真实状态NotRequired/仅Start。发现旧启动失败回执未带持久pending限制，main590536842补只读索引门：pending/unknown关闭三动作并带恢复限制，保留StartingFailed而不伪报Stopped；仅DTO/索引反例验证后加门，不改写早期原生证据。65后端相关及44前端/类型/build/格式/严格Clippy过；初编译可见性错误原件保留。

证据.adg/evidence/0.2.24/client-early-unchanged-r1/及manifest保留所有失败/index2/3/原件/准确源。自有进程缺席，2实际新协调DB排他重获释放后与程序/Profile/全部夹具清理、globalL身份/字节/权限保持，未改日常/官方/业务或新建.app/DMG/发布。早期完整原件产品确认、Preparing/不完整引用、已改写未plan早期恢复及原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方发布验证/固定候选继续，目标active。


## 2026-11-12 早期原件保持的产品确认入口

main7d6ecdf13注册finish_unchanged真实命令和预览canFinishUnchanged/独立摘要，沿原运行/更新/binding门进入已验证原生执行器；未知/Preparing无引用/已有plan或原件变化均不给能力。Ready持久引用不冒活进程证据，原件完整时预览Pending、执行仍新OS停止证明。未登记早期回执只能与同Head/当前全树逐字一致，坏值Unknown保留，不自动收养。main5b3d9e797接同UiDialog明确“核对并结束会话”→确认/返回、原件保持而未回写的文案、在途关闭重挂1promise、正确unchanged回执+新live同D/H才清限制，读取失败只刷新不再mutation。唯一总原型同步同4场景。详[早期产品确认](0.2.24-contracts/client-early-unchanged-ui-r1.md)。

24后端session去重用例全部有对应证据：首23过/旧Active断言1失败保留，改当前合同后仅定向重验失败1项，新增坏未登记回执和方向摘要early1项定向过；编译/格式/严格Clippy通过。61前端/类型/生产build过。WebKit首轮在原型busy隐藏关闭处失败保留，软件7项/原型1项局部不冒整组通过；本模块显式modal模式修正后WebKit/Chromium各12软件原型场景过，明暗/窄窗、取消零mutation、重开1mutation、失败限制/只读状态重试及旧Finalize键盘通过，pageerror空。原生算法和预算未变，按摘要复用上批2415ms/完整树/真实状态证据，本批无native重跑或.app/DMG/最终GUI→IPC候选验收。

证据.adg/evidence/0.2.24/client-early-unchanged-ui-r1/及manifest保留首失败/准确源/双引擎截图和复用native摘要；所有browser/Vite/audit终止、unit TMPDIR空，本批无native服务/协调DB/日常/官方/业务改变，归档核对后清重复临时根。Windows远端仍b2e89665d/0eef59a01无新回交，不跑W2。Preparing缺引用/未完捕获、已变化未plan早期恢复及原一般范围/完整集合/旧revision4/Windows/NAME/System代理/发布验证/固定候选继续，目标active，不发布。


## 2026-11-12 完全未创建原件保管目录的Preparing取消

main2c6cd6e05新增preparation取消：独立Preparing无refs/plan且整个client-originals缺席才可确认，descriptor-relative拒绝空/部分/链接或任何既存vault，不扫描/收养/删除原件；当前完整两树/原scope指纹、当前官方完整镜像/OS服务停止及记录缺席，原10秒metadata动作，create-only ticket/revision回执+Proof/CAS登记CancelledBeforeCapture，非原件恢复/Unchanged证明。晚到vault保持Unknown拒绝下一次Start，合法新Start另开ticket且旧事件保留。main1524b6a97接真实命令/预览互斥能力和明确取消确认/返回/单promise关闭重开，正确独立取消回执+新live同D/H才清限制，失败保留，读取失败只状态重试；唯一原型同步。详[捕获前取消](0.2.24-contracts/client-prepare-cancel-r1.md)。

25索引相关及71前端/类型/build/格式/严格Clippy过。新Profile官方准备161ms在捕获前按预设中断，原exit101/index1/refsnull/vaultabsent保留，真实StartingFailed三动作false+恢复限制证明前批回执门已生效。独立取消错确认/partial vault/陌生runtime记录拒绝保留；正例2722ms/index2，原vault从未创建/7当前文件保持/完整两树SHA匹配独立取消回执，真实新状态NotRequired/仅Start。服务从未启动，不跑无关启停或双SID。首两引擎green各9项只覆盖软件5真实流程+原型4静态打开，复核截图发现原型分支缺口，证据不改且明确降级范围；修脚本分支后另存各9完整软件/原型确认/返回/slow重开/失败重查通过，pageerror空，不冒首green为全过。

证据.adg/evidence/0.2.24/client-prepare-cancel-r1/及manifest保留所有来源边界/原失败/覆盖局限/真实成功/独立index1/2/完整树。自有进程/隔离程序Profile清理，0新协调DB创建（预列路径不存在）、globalL身份/字节/权限保持，四轮browser/Vite终止、unit TMPDIR空；无日常/官方/业务变化、新.app/DMG/发布。Windows远端仍b2e89665d/0eef59a01无新回交，不跑W2。任何既存vault的Preparing/部分捕获未登记/已变化未plan早期恢复及原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续，目标active。


## 2026-11-12 已登记闭合历史原件下的准备取消

main9ba2fc655从原独立SQLite连接读有界连续/canonical events，逐attempt最后引用须Restored/Unchanged、原件/scope/live/plan/完成receipt可验证，再将精确闭合attempt集合与vault目录成员/原生身份/保护比较；多/缺目录、unknown partial、损坏回执或历史rebase拒绝，不扫描生成引用或删除保留原件。新取消回执绑定历史/retainedVault、当前两树与原reservation，保持原权限/OS停止/完整镜像/原10秒。mainae3d08c5c软件和唯一原型改准确文案“本次准备尚未产生新的原件目录/保持当前配置和已保留原件”。详[历史原件核对](0.2.24-contracts/client-vault-history-r1.md)。

27后端去重/71前端/类型/build/格式/strictClippy、WebKit3受影响文案软件原型检查通过。新Profile普通Start9175/Status240ms；Stop原文件/父子/4TCP/全树/两锁通过但close-index超原10秒至10539ms，原exit101/index6失败保留，**预算未通过，不能发版**。不重跑整轮，独立显式metadata收尾9753ms/index7；随后一次明确Start在capture前中断961ms/index8，无新服务。已有真实闭合vault的独立取消8643ms/index9通过，错确认/未知partial目录/runtime记录拒绝不改索引。原Stop失败不被后继动作抵消；原秒表及完整性断言未放宽。

证据.adg/evidence/0.2.24/client-vault-history-r1/及manifest保留原失败/独立index6/7/8/9和准确分阶段源。独立完整两树SHA匹配取消receipt，29历史文件bytes/inode/mode完全保持，15blob与7当前文件核对、真实最终NotRequired/仅Start；自有进程/原端口缺席、3精确DB排他重获释放后和程序/Profile清理，globalL身份/字节/权限保持、browser/Vite终止/unitTMP空，无日常/官方/业务改变或.app/DMG/发布。下一步优先定向关闭Stop预算10539ms；真正部分捕获/已变化未plan、一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续，目标active。


## 2026-11-12 原件前驱链重复核验与普通Stop预算

main3e3bb6ab5将Originals.open和Pending.captures的同次4帧前驱递归重复改为每次新鲜完整prefix，仍逐帧独立digest/原生上下文/前驱/全部blob核验，不跨动作缓存；后补live.scope重复Original与session.open双scope、Pending.load多余Original读取，所有不同事实门/原锁/退出/端口/全树/权限/index及30/5/10不变。详[完整prefix成本与证据](0.2.24-contracts/stop-fresh-prefix-r1.md)。

105相关去重函数和只读probe1项通过，新增逐帧/逐blob坏值在后续open拒绝、held live原件/scope/ready/归档改变拒绝，restore普通/rearmed各11真实exit91边界保持；初probe测试root shadow/move编译错误保留并仅改helper，最终格式/编译/strictClippy过。首prefix-only新ProfileStart10396/Status267/Stop10296ms在完整readback阶段超10秒，原exit101/index6失败保留；同现场只读成本定位后才合并剩余重复，再开单轮新ProfileStart10239/Status271/Stop9415ms（入口段9407），真实父子normal0/4TCP缺席/三文件补偿/完整两树权限/两锁释放/receipt/index7 Restored全在原预算，重开1020ms NotRequired/仅Start、原函数exit0。外层acceptedfalse，原10539/10296失败不改写；余量585ms非一般大树或候选稳定性结论。

证据.adg/evidence/0.2.24/stop-fresh-prefix-r1/及manifest保留两轮/成本/原失败来源/准确binary及独立index6/7、成功完整树SHA、两轮共14原文件字节/缺席和blob/归档。两原主端口/自有进程缺席，6精确新DB排他重获释放后与两程序/Profile/原件/全部夹具清理，globalL身份/字节/权限保持、unitTMP空，无日常/官方/业务改动或.app/DMG/发布。Windows远端仍b2e89665d/0eef59a01；一般范围/候选预算/真正部分捕获/已变化未plan/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续，目标active。


## 2026-11-12 写入前预留捕获归属及部分原件取消

main47ad91a93在任何新attempt目录/blob前独立提交CaptureReserved(ticket)，生产借用同Index/action锁的不可序列化reservation，原head/根重核后EXCL创建；Captured refs必须原ticket，无预留便利begin仅test。中断无Original refs；取消只快照保全精确预留槽，不解析/收养为恢复源、不写删客户端或原件。CancelledCapture回执绑定原head/历史/完整partial树，未知另槽/链接/变更拒绝，历史保留部分证据后下一明确Start新ticket。main562d655ef软件/唯一原型区分部分捕获取消/捕获前取消/恢复完成，正确独立回执+新状态才清限制，保持确认/返回/关闭重开单promise。详[捕获预留合同](0.2.24-contracts/client-capture-reservation-r1.md)。

77后端去重、73前端/类型/生产build/格式/strictClippy通过；新增4个实际exit91捕获边界重开保留/错slot/错refs拒绝，两引擎各10软件原型场景过。新Profile原官方Start2279ms在Codex归档后中断、Original未发布/2blob保留、原exit101/index2 CaptureReserved/refsnull保留；独立取消3163ms/index3、错确认/未知另槽/runtime记录拒绝，真实NotRequired/仅Start、完整当前树SHA/取消receipt一致。再用户明确同现场普通Start9441/Stop9455ms（入口Stop9448）完整新原件恢复/index11 Restored过原预算；新attempt绑定新ticket、旧部分证据身份/字节/权限保持，独立全树/新receipt/真实状态及7文件核对过。不冒自动启动或一般目录/候选预算全过。

证据.adg/evidence/0.2.24/client-capture-reservation-r1/及manifest含index2/3/11、完整新原件和部分证据/准确分阶段源；自有服务/原端口/测试进程缺席，3精确DB排他重获释放后和程序/Profile/全部夹具清理，globalL身份/字节/权限保持、unitTMP空、browser/Vite退出，无日常/官方/业务变化或.app/DMG/发布。无预留旧残留继续拒绝，不做迁移；已变化未plan早期恢复/一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续，Windows远端仍b2e89665d/0eef59a01，目标active。


## 2026-11-12 四阶段齐备Stopped缺失恢复计划的显式准备

mainfe5be4b97对已独立Stopped/4帧和Live齐备、无plan分支新增只读policy准入/独立指纹和当前原生准备：当前三文件须精确Stopped后像，计划之外全Codex/Grok原件/保护及两稳定锁身份保持，任何未知stage/外部编辑/路由不明拒绝；完整官方准入/当前OS停止/原PID端口缺席及同进程锁下只暂存原件/计划并CAS登记RestorePrepared，不重建旧正常退出，不回写目标或自动恢复。main5870c915a接专用命令/预览和“准备恢复计划”明确确认/返回/单promise关闭重开，成功新live仍Required及同plan preview后才展示Continue，后续另行确认恢复；唯一原型同3场景。详[缺失计划准备合同](0.2.24-contracts/client-stopped-missing-plan-r1.md)。

60去重后端和82前端/类型/build/格式/strictClippy通过；括号/可见性、合成scope身份误用和错误回原字节断言首失败均保留，仅修代码/测试边界，不降精确native/change或准入。两引擎各7软件/原型场景过，pageerror空；生产排除mock。新Profile原官方Start8956/Status236/Stop3529ms按预设在独立Stopped index6、首次stage/plan前中断，原exit101/Required及尚未恢复文件保持。独立prepare3412ms/index7目标字节不改，另一明确Continue6372ms/index8 Restored，原每次10秒；错确认/外部config拒绝不改索引，无重跑服务。独立完整两树SHA及原件/plan/receipt/35引用15blob/index8和7原文件、新live NotRequired/仅Start核对通过，不用后继成功改判原Stop完成。

证据.adg/evidence/0.2.24/client-stopped-missing-plan-r1/及manifest保留首错/准确分阶段源/原中断和成功/截图。自有进程/原端点缺席，3精确DB排他重获释放后与程序/Profile/全部原件stagearchive夹具清理，globalL身份/字节/权限保持、unitTMP空/browser退出，无日常/官方/业务修改或.app/DMG/发布。Pending.prepare若在暂存后索引前中断仍因未登记stage拒绝并保留，未冒已恢复；下一批补计划准备归属/恢复及缺阶段早期恢复，原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续。Windows远端仍b2e89665d/0eef59a01，目标active。


## 2026-11-12 计划登记前只在事务目录暂存与中断重试

main55bc44300将生产Pending.prepare暂存位置移到已登记原件的私有事务目录，计划及原stage identity独立登记后才可向固定客户端目标原生交换。中断仅留私有未登记事务，新明确准备另id、原件原四帧/精确Stopped/全树余项/OS停止/同进程锁等门仍在，不收养/删除/覆盖旧文件；journal布局不忽略客户端任何stage，跨卷先拒绝。stage读回新增创建fd到路径原生身份/nlink绑定，字节权限相同的外部替换亦拒绝保留；原Complete/Rollback/rearm/归档冲突保全及10秒不变，UI/IPC不改。详[私有暂存与证据](0.2.24-contracts/client-journal-staging-r1.md)。

90相关去重用例/最终格式编译strictClippy通过：7个实际exit91准备边界保持客户端Stopped/无stage、重试后所有旧文件身份字节保护不变；登记后新journal与夹具client布局普通/真实rearmed各11共44边界过。审计旧测试父设rearm flag但child未消费，原green不能证明真实rearm路径，旧证据不改且本批实际走Complete中断→Rollback→rearm再退出，覆盖局限记录保留。新布局ACL/xattr及rollback独立终态/rearm→Complete读回定向过。

新Profile原官方Stop在私有stage_bytes处按预设中断，原exit101/index6 Stopped/无plan保留、客户端根无stage，两私有残留preparing+partialstage保持。新明确PreparePlan4765ms/index7，四目标不改、新id/schema3/journalStages绑定；另明确Continue6550ms/index8 Restored，原每次10秒、不重跑服务、不改判旧Stop。两旧残留文件bytes/inode/mode与前置相同，独立新完整两树SHA/四帧scopeLivePlanReceipt/index8/15blob及7原路径匹配、真实NotRequired/仅Start。准确首次/guard后续source和binary分开，不冒早期binary含后加guard。

证据.adg/evidence/0.2.24/client-journal-staging-r1/及manifest保存原中断/成功/旧覆盖局限/原件和所有残留/准确来源；自有进程原端口缺席，3精确DB排他重获释放后与程序/Profile/全部夹具清理，globalL身份字节权限保持、unitTMP空，无日常/官方/业务变化或.app/DMG/发布。缺阶段已变化早期恢复、原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续，Windows远端b2e89665d/0eef59a01，目标active。


## 2026-11-12 缺Stopped历史后像的独立当前恢复观察

mainf318a9e47对已有Original/Ready/PreStop/Live原独立refs2的StopPrepared，新增schema4 plan内RecoveryObservation/sourceHead/当前capture-tree-Grok，而不封写/采用历史stopped.json或给refs补阶段；原内容分类/完整余项/稳定锁/审计和当前官方镜像/OS停止/原PID端口缺席/同进程锁不变。准备后再次按plan内观察重算确认，哪怕后来是另一份合法native subset也须新确认；record plan绑定原sourceHead/CAS记RecoveryPrepared，原refs仍2。Continue/Finalize/Rollback/rearm/完整读回按该原plan和独立观察执行，终态保留原refs2；普通4frame仍原规则。main027c408cf软件/唯一原型分明当前核对与历史停止，正确新回执/阶段数及fresh live限制保持，不自动Continue/不改判原Stop。详[当前观察合同](0.2.24-contracts/client-current-observation-r1.md)。

92后端去重及84前端/类型/build/格式/strictClippy通过；source错revision/增造Stopped引用拒绝、unindexed损坏stopped.json不读取不回写、观察确认变更拒绝，refs2/refs3完整Rollback→rearm→Complete独立终态回归过。双引擎各10软件原型确认/返回/slow重开/失败和旧plan场景过，pageerror空，生产排除mock。新Profile官方Start8668/Status236/Stop2063ms在原父子normal0/4TCP缺席后、Stopped发布前按预设中断，原exit101/index5 StopPrepared/refs2保持。新独立当前计划prepare4099ms/index6 RecoveryPrepared目标不变，再另明确Continue6600ms/index7 Restored；原每次10秒，不假造过去退出证明或补Stopped。独立两树SHA、新schema4 plan/sourceHead、原三frameScopeLive字节保持/refs2/index5→6→7及7客户端原路径、真实NotRequired/仅Start匹配，stopped.json始终不存在。

证据.adg/evidence/0.2.24/client-current-observation-r1/及manifest保存原失败/新观察/准确源/独立摘要和截图；原生后仅加sourceHead validate收紧并定向反例/strictClippy，不冒旧binary含后加检查。自有进程原端口缺席，3精确DB排他重获释放后与程序/Profile/全部夹具清理，globalL身份字节权限保持/unitTMP空/browser退出，无日常/官方/业务变化或.app/DMG/发布。缺Ready或PreStop阶段、原一般范围/完整集合/旧revision4/Windows/NAME/System代理/官方验证/固定候选继续；Windows远端b2e89665d/0eef59a01无新回交，目标active。


## 2026-11-12 仅Ready历史的明确当前恢复与停止状态投影

main462171987承接缺Stopped观察：已独立登记Original/scope/Ready/Live、refs1而PreStop/Stopped缺失时，新的明确PreparePlan用实际Original+Ready+当前观察分类，sourceHead逐字段绑定原Ready头，schema4计划只列原两frame+当前观察三个摘要，原refs仍1。普通Stop仍须真实Ready==PreStop，Unknown先拒绝；不把Ready复制成历史PreStop、不封写缺阶段、不从当前内容推原生权限。当前全Codex/Grok余项、审计/路由声明/安全、原PID端口缺席、官方镜像/OS停止和同进程写者排他照旧。新观察只能准备计划，另行Continue/Finalize/Rollback/rearm及完整读回仍须新当前许可，原10秒不变。

定向原生首次还发现并修复状态衔接：历史Ready不证明服务现在运行。collector与本次已观察进程态合并，Running才Active、Stopped为Required、观察失败/未确认为Unknown；不改索引、不凭缓存清恢复、不改进程事实。软件与唯一原型同步仅Ready的准备/确认/返回/慢重开/失败，RecoveryPrepared阶段数严格允许1或2，普通RestorePrepared仍3；正确新回执与fresh live保持Required后才出现另行Continue，最终Restored+新live才可启动。详[仅Ready恢复合同](0.2.24-contracts/client-ready-current-recovery-r1.md)。

138项相关后端函数最终覆盖通过（其中首次restore组31过1失败，根因是执行未设置测试要求的/private/tmp，原失败保留，同断言独立正确夹具复验过；后改的两个prefix用例单独验证），86前端/类型/build/格式/strictClippy过。原普通历史一致性/unknown、未登记坏PreStop/Stopped不收养、当前合法内容变化仍需新确认、refs1/2/3完整撤销→重新准备→继续及独立终态均覆盖。两浏览器各7软件/原型场景通过，实际查看软件暗色和唯一原型确认截图，pageerror空；不扩为整版视觉/原包GUI验收。

新隔离Profile官方2.99 Start8739/Status227ms，按预设在PreStop前中断；生产正常Stop fallback确认原native exit0/4TCP缺席，Stop1289ms返回原失败，index4 Ready/refs1，PreStop/Stopped均不存在。原测试exit101、重开初次错误Active事实原件保留。修正collector后只在同一保留现场换准确新binary，未重跑服务：只读Ready+当前stopped真实状态Required；新的明确prepare4812ms/index5 RecoveryPrepared，目标不变；另一明确Continue5966ms/index6 Restored。独立完整Codex/Grok树SHA、原frame/scope/live字节和refs1、schema4 sourceHead/三摘要、15blob与原事件前缀全部核对，最终真实NotRequired/无PID/port/仅Start。后继新恢复成功不改判原Stop通过。

证据.adg/evidence/0.2.24/client-ready-current-recovery-r1/及165文件manifest保留首错、原中断/错误Active、不同阶段准确source/binary、新计划/完整读回/截图。归档逐字核对后自有进程与主端口缺席、3精确新协调DB排他重获释放再清理，程序/Profile/全部测试根删除；globalL身份/字节/权限/owner保持、排他重获释放/无sidecar，unitTMP空、browser/Vite退出，无日常/官方/业务变化或新.app/DMG/发布。

未完仍是本版完整范围：缺Ready/启动归属未登记的中断，首次缺根和一般客户端/路由图，完整进程集合及官方刷新协调，旧revision4首错，Windows贡献/产品接线，NAME非合法键/派生组、复杂System代理、官方发布验证器、README/版本和固定候选整链/W2；原普通Stop9415ms只证明同类样本，候选及一般树预算未冒完成。当前目标active，不以本批恢复工作替代其它版本任务。


## 2026-11-12 首次客户端状态诊断与未用专用清单的正常收尾

main d98d39e8b修复catalog/cache/audit四个真实阶段始终明确缺席时仍强求Ready清单包含路由条目的问题。只有先通过原context/unknown和普通历史一致性、三件始终Absent，才返回三项AlreadyOriginal；不生成模型投影、不授予原生权限或忽略其余文件。FullTree/原件/当前OS停止和写者隔离、原30/5/10秒仍照旧。20个policy函数/编译/格式/strictClippy通过，未知任阶段/缺context/外来新文件拒绝反例保留。原生驱动改按本轮真实原件与缺席核验Grok，并先持久生命周期结果，避免固定夹具断言遮蔽真实产品首错。

四个不同初始状态按事先假设各跑一次原包，无同状态盲目重跑。前三个原driver均在补充Grok归档断言失败、exit101保留：已有Codex/无Grok的产品已完整Stop6644ms、index8 Restored，独立native两树和真实NotRequired状态成功；Codex空目录/无Grok实际Stop3059ms后index6 Stopped无plan，新增config.toml为单换行且原先不存在；Grok首次备份场景Stop7000ms后index7 RestorePrepared，三件已还原但新config.toml.bak-opencodex保留，完整读回正确拒绝。均原native正常exit0/原4TCP缺席，不用driver失败把所有产品阶段混称失败，也不改判原driver通过。

另一个新Profile专门验证本次实际修复：已有Codex配置、无专用模型清单/缓存/审计、无Grok，复用前一个夹具已完整校验的原样官方2.99镜像而非重建维护包。新准确binary Start6495/Status264/Stop5744ms，native整driver exit0/result passed，Original/Ready/PreStop/Stopped三件都Absent，计划三Keep，完整恢复index8，Grok仍Absent；独立完整Codex/Grok当前原生树SHA匹配completion，原件/Scope/Live/plan引用逐摘要核对，正确实际Program上下文的重开状态NotRequired/仅Start。额外只读status helper错用了本Profile不存在的program而得到NotFound!=Stopped，原失败保留且不计通过；不以它替代原driver已通过的正确运行上下文状态。详[首次客户端路径合同/结果](0.2.24-contracts/client-first-use-r1.md)。

证据.adg/evidence/0.2.24/client-first-use-r1/及236文件manifest含完整状态表、前三份原失败/真实ledger/原件与合成残留、第四份新来源/通过/完整读回及官方相关源摘要。先归档逐字核验，再清理4个隔离Profile/程序/所有测试夹具；12精确新协调DB实际排他重获释放后删除，自有进程/主端口缺席，globalL身份/字节/owner/权限保持且排他重获释放/无sidecar，unitTMP空，无日常/官方/业务变化或.app/DMG/发布。

下一步必须处理同会话首次生成、已有独立原件缺席证明的空Codex配置和Grok备份的可逆补偿；Grok写入须接它自己的官方config-write-lock，不拿Codex锁冒同一权限。禁止以“空文件/新备份可忽略”扩大两稳定锁例外。整个.codex根不存在现由canonicalize在启动前拒绝，源码已确认、尚未开发，不能用本次空目录或无清单样本冒覆盖；须保留首次创建归属/原缺席事实。缺Ready/一般图/全进程集合/活跃刷新/旧revision4、Windows接线/NAME/System代理/发布验证/README/固定候选/W2继续，目标active。


## 2026-11-12 首次空Codex配置作为第四对象可逆恢复及官方写入锁

main b60bd1079补首次空目录Stop残留config.toml：Original明确无该文件、Ready独立记录普通配置、实际PreStop若存在须和Ready记录一致，Stopped/明确当前观察仅含4096内ASCII空白且原保护相同，才schema5计划额外绑定精确当前文件Image。新文件identity/字节/Protection必须匹配已独立历史tree witness，不从当前缺记录输出猜来源。原件/缺失历史不变，pre_stop/stopped缺口不补写。原三件分类和真实原生权限保持；第四对象复用同一state/pair/native-gap冲突反向保全/原生RENAME_EXCL移到私有stage/旧对象归档、Complete/Rollback/rearm/完整读回及独立终态。它不是新增忽略例外；最终普通全Codex树仍严格匹配Original，空文件必须实际缺席。详[首次空配置恢复合同](0.2.24-contracts/client-empty-config-r1.md)。

初原生内容恢复Start8477/Status255/Stop5920ms/index8 Restored，随后复核发现config.toml有独立官方文件写入锁，既有claim/N/L/K不能代替；该首pass原件保留但明确不作为最终写者隔离完成证明。补Desktop adapter借用原样官方withConfigWriteLock/prompt-lock，以专门子进程阻塞父管道持锁到callback结束，没有2秒自动解锁；当前Rust仍持原native lease，回调每次核对同锁Image/子进程未退出，忙/损坏/替换/超时拒绝；正常release回执、子exit0、锁缺席后才返回成功。父死亡仅pipe EOF触发官方finally，失败清理强停专用helper不能算正常成功，不信PID活着就认锁。

55相关后端函数最终通过：新增6（包括4个真实exit91 native/archive/terminal边界、refs1/2/3同对象撤销重准备/外部替换保全/缺Ready及原文件拒绝/降级计划拒绝），完整终态函数扩为已有配置与原缺席两类各refs1/2/3共6情形，旧restore32/tree9/completion7保持。初编译类型/guard私有字段误访与旧client-staging夹具1过3失败保留说明，改夹具用真实journal staging不放宽产品门。最终格式/编译和strictClippy过，未改UI所以不重复页面测试。

原生锁定向用真实官方另一进程证明持锁时locked/二次holder拒绝，超过2100ms仍held、正常释放后官方writer通过，config仍Absent。实际父进程exit91后无Rust drop，helper经EOF退出（ESRCH）、锁缺席，无信号/强杀补证明。加入锁后的准确新binary、新Profile复用核验官方镜像，Start6314/Status245/Stop6654ms，原30/5/10不变，主driver exit0、完整Restored8/原4frame/Ready与PreStop配置同一独立记录、Stopped单换行Image和schema5 plan/old-3原对象绑定，最终config与Grok均Absent。独立完整两树SHA匹配completion、原件/Scope/Live/phase/plan逐摘要/官方guard正常释放与正确实际Program上下文Stopped/NotRequired/仅Start均通过，源与首轮binary分开。

证据.adg/evidence/0.2.24/client-empty-config-r1/及148文件manifest保留初错/内容首pass局限/锁验证/父死/最终pass及原件。归档逐字核对后2隔离Profile/程序/所有测试根清理；6精确新协调DB实际EXCLUSIVE重获释放后删除，config锁及claims缺席/自有进程主端口缺席，globalL身份/字节/owner/权限保持、排他重获释放/无sidecar，unitTMP空。无官方/日常/业务修改或新.app/DMG/发布。

剩余仍须Grok首次备份的精确可逆恢复及它自己的官方写入排他、整个.codex缺根归属/原缺席、缺Ready/一般客户端与路由图/全进程集合及活跃刷新/旧revision4、Windows W1产品接线/NAME/System代理/发布验证/README/固定候选W2。Windows远端本轮重新fetch仍b2e89665d/0eef59a01，无新交接；本批原生只证明新增空配置合同，不代表整版accepted或所有竞争写者已覆盖。目标active。


## 2026-11-12 Grok首次备份可逆恢复与两配置写者隔离

main d5970bc55补Grok首次生成config.toml.bak-opencodex：Original必须明确无backup且有原config，Ready backup须字节与保护匹配Original config，实际PreStop/Stopped须保持同一已记录对象，当前Image/根identity匹配可信tree才schema6 plan登记可选grokBackup。固定第五目标在原profile/.grok，而Codex可选空配置仍第四目标；indices只枚举存在的目标，不能靠位置枚举把Grok备份当Codex文件。原三件和空配置共用同一精确对象状态机/RENAME_EXCL移入journal/archive/冲突反向保全、Rollback/rearm和完整读回。既有backup不动，未知新文件/新根/权限/复制内容或代次变化拒绝，不扩残留忽略例外。详[Grok备份合同](0.2.24-contracts/client-grok-backup-r1.md)。

共享管道holder只允许两个固定canonical target：实际Codex根或当前profile/.grok；保持HOME/Profile/OPENCODEX_HOME和原官方源码准入。可选Codex和Grok恢复各借用目标config.toml自己的官方锁，保持每步同native许可/child/锁Image检查、正常release/exit0/锁缺席；双目标时两把都持有，任一失败仍失败，按反序释放。当前complete仍整Codex+整Grok严格比Original；Rollback由plan精确backup和原余项共同核验，不能清启动限制。缺PreStop/Stopped仍仅按已有明确当前观察/sourceHead规则，不补造历史；错误确认或新备份均须新明确核对。

65相关后端函数最终过（新Grok6、旧restore38、终态1、completion7/Grok4/tree9）：新真实exit91在第五对象交换/归档/终态边界重开原plan继续；新旧/替换/外部gap/根代次/历史PreStop变化拒绝；终态函数现4种配置来源×refs3/2/1共12情形，完整Rollback读回→独立撤销头→rearm→Complete→Restored保持。初闭包引用lifetime和plan字段部分move编译错误由本轮结构化记录及chat原输出保留，不冒原始独立日志。最终格式/编译/strictClippy过；无UI变化，不重复页面验证。

一个新隔离Profile原样官方2.99实际Start9025/Status234/Stop9189ms，原30/5/10秒不变，整driver exit0/result passed/index8 Restored。Original backup缺席、Ready/PreStop/Stopped同一backup的字节/保护/identity，匹配Original config；schema6/grokBackup root和Stopped Image、old-4归档对象/原4frame/Scope/Live/plan摘要均独立核对，最终backupAbsent且原config/Agent保持。独立完整Codex/Grok当前原生两树SHA匹配completion，正确Program上下文Stopped/NotRequired/无PID/port/仅Start。停止贴近预算，不能从单样本过推一般树/最终候选余量达标；原首次备份旧失败在client-first-use-r1保留，不被本次新通过覆盖。

同一保留现场另执行无服务动作的两锁定向：真实官方另一writer在仅Grok锁时Grok locked/Codex可写，同时持两锁时两者locked，先正常释放Grok只放行Grok，再正常释放Codex才放行Codex；两锁与claims均实际缺席。不把N/K/L当Grok锁、不从子PID活性推断持锁。证据.adg/evidence/0.2.24/client-grok-backup-r1/及89文件manifest保存精确源/最终binary一致性/原件/真实trace和两锁。先归档逐字核验再清理本Profile/程序/全部夹具；3精确新协调DB实际EXCLUSIVE重获释放后删除，自有进程/主端口及两配置锁/claims缺席，globalL身份/字节/权限/owner保持并排他重获释放/无sidecar，unitTMP空，无日常/官方/业务变化或新.app/DMG/发布。

下一步整个.codex缺根创建归属/原缺席、缺Ready中断、一般客户端/路径/图/完整进程集合/活跃官方刷新与旧revision4，Windows产品接线及W1贡献、NAME非合法键/派生组、复杂System代理、官方发布验证/README/版本和固定候选W2继续；不把本批备份恢复等同整版完成。Windows本轮fetch仍b2e89665d/0eef59a01无新提交，目标active。


## 2026-11-12 缺根目录的原生创建归属原语与未登记拒绝

main f2c1d1c75新增root_creation原语，尚未接生产Start：在实际profile下独立确认.codex原缺席，持有与Index同client-session-action.lock/flock，先create-only0400 intent和独立intent引用，绑定uid/data/profile原生identity及profile保护。私有事务槽排他创建空candidate、记录created身份/保护并保留原fd；同卷原生RENAME_EXCL发布到.codex、fsync/当前路径与原fd身份/安全/空目录复验后才写published并返回不可反序列化的PublishedRoot。没有在新目录上拍假的“原来就存在”Original，也不启动服务、删除目录或给恢复许可。详[缺根创建归属合同](0.2.24-contracts/client-root-custody-r1.md)。

防止半接线误收养：主Index.open和只读inspect在发现root-intent或reservation槽时拒绝，不能把临时新.codex当旧配置重新建基线；action_pin只扩大到本原件模块内复用，实际验证逻辑不变。已有目标/目录或symlink/新并发目标、外来candidate、旧index/原件、改权限或引用、改根代次、非空/超时都拒绝保全。磁盘中的created/published不能重建进程内创建句柄，崩溃后不自动恢复或抹残留；原未登记文件继续不被收养。

35个相关函数最终通过（root8、index27/2ignored），格式/编译/strictClippy通过。一个真实第二进程被同action flock拒绝；4个真实child exit91分别在intent后、created后、原生发布后、published后，原缺席或已创建根事实分别保留，再新调用仍拒绝，记录文件字节/identity/mode不变且没有session index/Original frame。提取4场景原JSON/摘要/元数据并独立核对intent引用、created/published链及rootPresent边界。首fmt先于测试文件写完报缺文件，补齐完整测试后通过，原chat输出和结构化说明保留，不冒独立原始日志。

证据.adg/evidence/0.2.24/client-root-custody-r1/及16文件manifest保存假设/准确源和binary/4崩溃现场记录、主线拒绝、旧index回归和清理。仅本机原生文件系统/子进程测试，没有官方服务/GUI或真实业务调用，不冒完整缺根启动验收；测试子进程均实际wait到成功或91，TempDir清自有夹具，unitTMP空，归档逐字核验后删重复临时根，无日常/官方配置或全局协调DB变化，没有.app/DMG或发布。

缺根用户路径仍未解决：Active.capture仍在client canonicalize处拒绝，root_creation.reserve不由生产调用。下一步必须同一flock内转交CreatedRoot归属并独立登记原缺席/实际新根引用，然后接首次Start、正常Stop原根状态恢复与中断明确处理；不能用本原语pass冒完整启停，也不能释放/重拿锁后猜目录归属。之后才跑一个有假设的原官方缺根Start30/Status5/Stop10及状态/完整恢复验收。缺Ready/一般客户端图路径/完整进程集合/活跃刷新/旧revision4、Windows W1产品接线/NAME/System代理/官方交付验证/README/固定候选W2保持未完，目标active。


## 2026-11-12 创建根同锁转交、原缺席与Preparing原子登记

main 1c967e1bf接Index::register_first_root：只接受当前进程PublishedRoot，在原deadline内复验data/profile/实际root后移动原action flock File与created fd，未unlock/重新按路径认领。现有三表SQLite事务内同时写包含原根Absent独立引用的binding和revision1 Preparing首事件，ticket必须等于创建nonce；同一个COMMIT后才返回Index/Head。root_refs仅是事实，磁盘文件或序列化引用不能构造创建/服务权限。后继同根重开只能按已登记binding复核RootIntent/Created/Published摘要/身份/保护/上下文和首事件，不从sidecar自行补源。详[根登记合同](0.2.24-contracts/client-root-registration-r1.md)。

既有根仍用原scope binding，不自动转换；新根用明确RootBinding envelope，数据库user_version3/三表不变，是本版两种初始化事实而非旧版迁移。根创建未独立提交时，即使已有空index或rollback journal，普通open也在SQLite恢复/初始化前拒绝保留，read-only同样不触碰；已提交时只可重开受限Preparing。删除origin降级普通binding、改创建字节/根代次/源ticket或context拒绝，Index活跃check也复核binding和首事件。until root-aware Original/取消/根恢复接线，reserve_capture/advance到普通捕获/旧取消与完成均拒绝；preview不给普通cancel按钮，不让“目录现在存在”抹掉原缺席或误清启动限制。未接入Active.capture，缺根完整用户路径仍未解决。

42相关函数通过（登记6、原创建8、index27/2ignored、旧Preparing取消1），fmt/编译/all-targets strictClippy通过。真实第二进程分别在fd已转交、COMMIT前、COMMIT后三点探同action flock均被排除，实际回调计数3。3个真实child exit91验证提交前仅残留/拒绝且字节保持，提交后只能见Preparing1且不能普通捕获；原件frame均未创建。提取原合成文件/SQLite字节，并离线immutable SQLite独立验证提交后的RootBinding与首事件同nonce/同引用、context scope和原缺席JSON摘要链；离线投影不冒活体native归属。首Instant未import E0433由结构化说明与chat原输出保留，不称有独立首错原日志。

证据.adg/evidence/0.2.24/client-root-registration-r1/及32文件manifest含假设/准确源binary/三崩溃字节投影/独立核验/拒绝与旧路径回归。所有子进程实际wait，TempDir清自有现场、unitTMP空，归档逐字核验后删重复临时根；无官方服务/业务/日常配置或全局协调DB变化，无新.app/DMG/发布。未跑无假设的旧启停，也未以文件系统索引测试代签真实原包启动。

下一步root-aware原件事实与终态恢复：独立索引已有原根缺席，但现有Frame/Capture仍描述目录树，必须把二者准确区分后才允许原件捕获；接正常Stop原根状态恢复、受保护原件/两锁归档及根消失后的只读状态/后继新启动，并接初始化中断的明确处理。随后用新隔离Profile原官方2.99验缺根Start30/Status5/Stop10。缺Ready/一般客户端图路径/完整进程集合及活跃刷新/旧revision4、Windows产品贡献/NAME/System代理/官方交付验证/README和固定候选W2仍完整保留，目标active。


## 2026-11-13 原根Absent事实进入独立原件链与Captured登记

main 4aa0d1ace为已独立root-registration增加专用reserve_first_root_capture，必须当前Index仍持原created fd、源nonce/revision1、空执行根且未超原创建deadline，预留CaptureReserved2先于任何原件目录。schema5帧显式originalRootAbsence，独立原根Absent与捕获的初始化执行tree不混同；Original只能含实际初始claim/held-sidecar协调元数据，不接受用户配置/认证/服务owner等新内容。每个实际后继帧继承同来源引用，原件独立SHA/predecessor链/目录保护/内容归档仍核对，缺失阶段不补写。普通原件构造/重开不能从root marker或新目录把它收养成既有配置。详[根原缺席原件合同](0.2.24-contracts/client-root-originals-r1.md)。

同锁Index::record_first_root_originals在原deadline内核对Original原摘要、schema5原根来源和startup-scope同data/profile/home/client identity，CAS登记Captured3/独立refs，普通advance不能跳Ready。重开root-aware原件仅按当前独立RootBinding+Head里的原摘要及scope，不扫描目录新算baseline；Index.head和只读inspect亦复核该原件来源/上下文，scope被改保持拒绝。预览用同action读锁内传入的origin显示Pending/Captured、original verified，但不给普通取消/原件未变结束/继续/收尾按钮。完整tree读回不是整根Absent恢复，complete/rollback仍明确要求尚未实现的whole-root证明，不产成功receipt。生产Active.capture未启用缺根路径，本批不是原包缺根启停通过。

首轮0过7失败定位到上一批终态拒绝误放在通用Index.write，连本次专用预留也被挡；原log完整保留。将拒绝移到close_readback，root Index.write仅允许明确Preparing→CaptureReserved、专用CaptureReserved→Captured及Failed，旧捕获/取消/完成捷径继续拒绝。首新增argument及闭包变量shadow编译错误由hypothesis与chat原输出保留，修正测试调用/命名，不降门槛。原件阶段checkpoint再次验证创建custody/原deadline，登记提交前后复核时间，超时后已写引用保留而不谎称未写入。

108个去重相关函数通过（root-originals9、普通originals23、index27/2ignored、root-registration原6+deadline1、completion7、tree9/session26）；最终fmt/check和strictClippy通过。新增实际SQLite EXCLUSIVE事务产生claim+journal，证明可捕获的只是metadata而非用户Original，同时服务owner拒绝。4个真实child exit91分别在原件目录创建/Codex归档/Grok归档/Original发布，独立索引始终CaptureReserved2/refsnull，磁盘即使有完整Original也不被重开收养，记录字节保持；原创建句柄丢失/过期同样拒绝继续捕获。独立提取原JSON/原件及SQLite字节投影，核对RootBinding原缺席、两事件和未登记Original；正例Captured3的Original/scope摘要与Head、原根引用一致，原件链无捏造历史。离线副本不冒原生身份或服务权限，synthetic scope不冒官方来源/OS启动证明。

证据.adg/evidence/0.2.24/client-root-originals-r1/及103文件manifest含原失败、分阶段回归/准确最终源binary、四崩溃字节投影、独立核验和清理；后加deadline及只读预览检查有各自定向结果，不声称较早日志执行过后来改动。所有child实际wait，TempDir清自有现场/unitTMP空，归档逐字核验后删重复临时根。无官方服务/真实业务/日常HOME或全局协调DB变化，无新.app/DMG/发布。

下一步完整根恢复：安全停止并保全实际初始化目录/两稳定锁与原件，将当前根恢复为Absent，独立索引/只读来源在根已移出后仍能核对，再接新Start及初始化中断的明确恢复。完成这些证明后才能开放Ready/生产缺根Start30/Status5/Stop10，不能仅解除当前拒绝。缺Ready/一般客户端图路径/完整进程集合及活跃刷新/旧revision4，Windows产品贡献/NAME/System代理/官方发布验证/README/固定候选W2仍在原范围。Windows本轮fetch仍b2e89665d/0eef59a01无新回交，目标active。


## 2026-11-13 整根可逆归档与缺根只读重开存储原语

main 947fc8139新增root_retirement存储原语：已登记原根Absent、原件执行树已还原、当前完整Codex/Grok匹配Original后，create-only plan绑定原source Head/RootBinding refs/当前完整执行树，固定同卷root-retired-<id>槽。RENAME_EXCL移动完整目录对象到原件私有vault，原文件/锁都保留而非按路径删除；可回移同一对象。晚到源根替换先写冲突元数据再一次EXCL反向归还，目标/归档/内容变化/无原movement intent都拒绝，读回文件写后新根出现也不返回成功。详[整根归档合同](0.2.24-contracts/client-root-retirement-r1.md)。

Originals新增仅内存retired_root_slot物理锚点；原schema5 frame/root/source引用不变，logical_client始终原profile/.codex。移动后验证同一root identity/Protection、原帧/Scope/归档字节及前驱链，不拿归档路径改写scope或拍新Original；归档view不能seal新历史。open_archived_originals只依据调用方应由独立索引提供的ArchiveReferences/source Head/原RootBinding/plan摘要打开准确固定槽，不扫描备份、不从现在文件学习新摘要；source revision/hash/profile错误拒绝。它是只读事实入口，不能重建移动或服务许可。

73去重相关后端函数过（新retirement8/root-originals9/普通originals23/session26/completion7），fmt/check/strictClippy过。新实际child exit91在movement intent/原生移动后/readback前/readback后4边界，父实际wait91，准确原plan和retained对象可继续Retire、回移ExecutionRoot、再Retire；原独立index保持Captured3，未写RootRestored或给Start许可。正例同root/原frame字节/逻辑scope保持，新增外部目录/归档/读取后新根/来源hash不明拒绝；已移动外部替换可反向保全且失败。离线5场景原JSON/SQLite字节投影核对planSHA/sourceHead与独立Captured3、origin/frame/scope引用链和readback树摘要，副本不冒原生权限。后加公开archive bootstrap只读入口有自己的定向结果/strictClippy，不冒较早binary已含该入口。

证据.adg/evidence/0.2.24/client-root-retirement-r1/及95文件manifest保留假设/准确分阶段source与binary/4真实崩溃+正例投影/原读回及清理。所有测试child实际结束，TempDir清自有现场、unitTMP空，归档逐字核验后删重复临时根；未运行官方服务/业务或日常HOME修改，未触全局协调DB，无.app/DMG或发布。

关键未完明确：MutationPermit没有生产构造函数，测试回调不是OS停止/写者排他证明；本批不由Start/Stop调用，不宣称缺根启停已修好。下一步接当前真实service/launcher/TCP停止与官方写者排他/稳定锁释放，并处理root移出后许可的物理锚点变化；把movement plan/方向与真实Readback独立登记到索引，使普通状态在profile/.codex已缺席时仍可验证终态，明确中断后的用户动作，再接下一次Start。完成整链后才开放Ready和跑原官方2.99缺根30/5/10原生验收。原缺Ready/一般客户端图路径/全进程集合及活跃刷新/旧revision4、Windows产品W1/W2/NAME/System代理/发布验证/README/固定候选继续，目标active。


## 2026-11-13 整根归档独立登记与缺根状态核验

main 7131fdadc在移动前CAS登记RootRetirementPrepared，绑定原source revision/Head、不可变plan id/SHA、原RootBinding和Original引用；完成只接受当前apply返回的不可反序列化AuthorizedReadback，并绑定原动作deadline、data/profile身份、来源/方向/计划。caller不能延长permit期限，过期不关闭；提交后过期则保留事件并退回待核对。RootAbsent仅记录存储层原逻辑根确实缺席且准确归档/读回已核对，不冒服务停止或新Start许可。详[独立登记合同](0.2.24-contracts/client-root-index-r1.md)。

只读观察先打开受保护独立SQLite，再按准确source event/RootBinding/plan定位原逻辑.codex或固定archive；逻辑scope不改，物理root identity和完整tree/Protection仍核对。普通已存在根不能冒原根Absent；原路径重新出现新用户目录、归档/读回/来源变化保持拒绝且不覆盖。缺根后readonly和持排他action锁的writable reopen都可核对原账本，不扫描备份或重算采用新摘要。root-specific预览保持Pending，普通继续/收尾/回退/Start操作都不放行；软件与原型显示同样只读诊断。

84个去重后端相关函数、88项前端、fmt/diff-check/最终all-targets严格Clippy通过；WebKit/Chromium各6项合成页面与原型检查通过，实际查看软件与原型截图。三个真实child exit91覆盖计划登记后/整根移出后/终态登记后，父实际wait，独立索引分别保持Prepared4/Prepared4/Absent5，readback已写但未登记时不当完成。原JSON与SQLite字节投影独立复验4场景的事件链/来源/plan/Original/scope及终态读回SHA；离线字节不是原生权限证明，移动授权factory仅在测试存在。

首轮2过2失败为旧physical-overlap规则误拒私有archive，仅有chat原输出与结构化记录；第二轮2过4失败为投影末尾仍比较physical scope，有原log。二者修正为准确来源约束下的logical scope及末尾原根仍Absent复核，保留所有失败。Clippy冗余pattern及新增private deadline后漏测试构造字段的编译失败也保留，修正后最终严格检查通过；较后专项测试不冒称较早日志执行了后续源码。

证据.adg/evidence/0.2.24/client-root-index-r1/含104文件manifest、分阶段source/binary与验证范围、失败、截图、3崩溃+正例原字节和可重放离线核验。归档逐字校验后清理自有重复临时根；unitTMP空，browser/Vite与测试child均结束。无官方服务/真实业务/日常配置/全局协调DB变化，无新.app/DMG或发布。

生产MutationPermit仍无factory，NativeLease/官方写者锁和SQLite路径在根移动后的当前权限尚待接线，next Start仍拒绝；下一步优先完成真实当前Stop/launcher/TCP缺席与写者排他及重定位后的持续校验，再接完整缺根启停和用户明确中断恢复，按原官方2.99/30-5-10预算做隔离原生验证。本批没有新增Windows原生影响，不另跑W2；Windows W1可继续现有独占任务，固定集成候选后才交最终受影响Windows验收。缺Ready/一般路径与客户端图/完整进程集合和活跃刷新/旧revision4首错、NAME/System代理/发布验证/README/固定候选等完整范围不缩减，目标active。


### 2026-11-13 根移动后的锁权限可行性定向实验

证据.adg/evidence/0.2.24/client-root-relocation-feasibility-r1/保留原假设与结果。macOS实际/dev/fd/<directoryfd>/sample读取ENOENT，故不能用该路径绕开原逻辑路径变化。合成SQLite在原连接正常rollback/close后，用原retained fd取得全文件fcntl排他，真实独立子进程在目录移动前后均database is locked，原inode/mode/字节保持，释放后可正常获取。负控确认打开并关闭同inode另一fd会释放进程关联锁，子进程随即获取；因此后继不能在转交期间调用会open/close claim inode的通用读回。系统Python SQLite3.54.0，不冒生产rusqlite/原官方启停验收。

核对原包SHA与未修改官方源码：config writer acquisition会canonicalize绝对路径，现Desktop holder并无重定位协议；NativeLease仍核对原VFS路径。后继需独立设计当前claim锁转交/重新准入与no-auxiliary-open生命周期、N/L/K持续排他、官方writer真实holder在移动后准确路径/token核对与正常释放。此实验不批准直接改变handle.path、不删除现VFS检查、不增加生产MutationPermit factory，尚无生产代码变更。完整实验自有目录和fd/连接/子进程清理，证据逐字归档后删重复根，无日常HOME/全局DB/官方运行修改，目标active。


## 2026-11-13 随原目录移动的OFD排他原语

main94c897bbf新增ClaimGate，使用Darwin F_OFD_SETLK whole-file排他，与项目实际bundled SQLite跨进程互斥。先prepare原私有fd，再由调用方正常关闭claim SQLite，acquire借用原pin且失败不关闭fd；不创建/修复原文件、不等待/降级。物理anchor必须仍是原parent identity/Protection及claim inode，期限过后保持锁但操作拒绝，显式finish/drop或真实进程退出才释放。此批尚未接生产NativeLease或根移动factory，不能拿锁原语替代Stop/官方writer证明。详[OFD合同](0.2.24-contracts/client-claim-ofd-r1.md)。

最终9函数通过（8实质案例+1独立SQLite子进程探针），原8项先期日志保留。真实跨进程验证目录移动前后/额外File关闭/另一SQLite连接关闭后均仍排他，原identity/字节不变；外国持锁立即拒绝，本进程未释放SQLite时失败也不误解锁；父目录/inode/mode/链接变化及新用户根拒绝，到期不提前解锁。实际child在移动后exit91，父wait91，系统释放准确原inode，无清理信号。fmt/diff-check/strictClippy过；证据.adg/evidence/0.2.24/client-claim-ofd-r1/已逐字归档并清理自有临时根，无官方服务/业务/日常配置/全局DB/新包修改。

证据勘误：上一批root-index的source-final误将应用测试宿主摘要标为库测试binary，原记录不改，新增previous-source-correction记明。旧库摘要未在重建前保存，不能以本批摘要代补；其源码commit/逐文件hash、原测试输出及独立字节投影仍保留。本批按实际执行日志定位库宿主并记录准确摘要。

下一步以OFD原语接真实NativeLease和目录重定位，保留N/L/K、原官方writer与当前正常Stop/launcher/TCP事实；随后接缺根Start/Stop和next Start及明确中断处理，再做原包隔离原生验收。默认生产锁路径/VFS检查未放宽；其它完整剩余范围和固定候选/Windows W2顺序保持，目标active。


## 2026-11-13 NativeLease的claim交接与原官方协调验证

main0b8b1e74e增加显式with_native_relocation_coordination，共用原官方准入及来源前后核验；普通入口保持四SQLite事务。预存原OFD pin先于SQLite，handoff仅正常关闭claim，N/L/K持续排他，空档Pending拒绝所有操作。抢占者获得claim导致交接失败也不能当自己的许可；失败清理不冒成功。移动后只允许明确reanchor原directory/claim identity和Protection，logical client仍原路径，新用户根出现拒绝；owner/profile/HOME/期限及仍持有N/L/K原VFS检查保持。详[NativeLease交接合同](0.2.24-contracts/client-native-handoff-r1.md)。

19个常规函数通过（含2个子进程探针入口），官方ignored入口另1项真实通过；原阶段回归保留。实际子进程证明交接空档N/L/K持续忙而lease拒绝、外国claim抢占不放行、移动及额外fd关闭后四锁仍忙、释放后可获取；未知anchor/新用户根/owner/VFS/过期仍拒绝。未修改官方2.99真实准入/K/L写入入口在移动前后均busy且不执行callback，释放并回移后正控成功；原协调动作2054ms<10000ms，全runner69.944s不冒动作预算。fmt/diff-check/strictClippy通过，最终源码/执行库binary与真实原官方轮完全对应。

证据.adg/evidence/0.2.24/client-native-handoff-r1/含假设、源码/binary、完整日志、官方writer输出、global L前后身份/权限/字节及核验脚本。全部child实际wait，精确新N/K各在exclusive重获后删，预存global L保持identity/mode/bytes不变并重获释放，测试Profile/program和重复临时根清理，无日常配置/服务/业务请求/新包或发布。

本批仅协调层，官方config writer holder尚无移动协议，生产root MutationPermit/正常Stop完整证明/独立根关闭/next Start/缺Ready等仍未完。下一步接原holder准确物理路径与token/正常释放，再接真实根恢复许可与原包隔离启停。Windows W1与其它完整IMP范围不缩减，W2仍等固定候选，目标active。


## 2026-11-13 官方配置写锁的移动连续性与正常释放

maine3085eb87将Desktop holder接到未修改官方acquireConfigWriteLock/release公开API，保留原活体root/lock fd及原token/字节；先登记唯一Absent目的地，再在NativeLease已OFD reanchor后核对同一目录/锁和原路径缺席。消息绑定operation/PID/targetSHA与固定阶段，失败poison拒绝，不从序列化字段构造移动许可。EOF只在原/唯一目的地核对准确原对象并官方释放；新根/相同token新inode拒绝保全。详[写锁移动合同](0.2.24-contracts/client-writer-relocation-r1.md)。

首原包轮exit101保留，后继直探initial-check定位APFS目录nlink因自己创建锁/claims项改变，不能当root identity。修正只核对root代次/uid/gid/mode，原锁single-link/inode/字节仍严格，未放宽文件或权限。最终19常规函数（含2探针）、fmt/diff-check/node语法/strictClippy通过，原包7场景＋父进程真实exit91通过。正常移动/释放、三个EOF边界、同token替换保全、新用户根保护、非法命令拒绝均有证据；EOF实际wait正常exit1不冒成功且无清理信号。最长场景4738ms含刻意持锁2100ms<原10秒，供应/整轮90.665s不冒动作预算。Codex/Grok双helper直探证明两锁独立，分开正常release退出0且配置字节不变。

证据.adg/evidence/0.2.24/client-writer-relocation-r1/保留失败/直探/最终源码与实际库binary/官方原始输出/可重放核验和清理；所有accepted false。两轮child实际结束，测试Profile/program/直接probe目录及精确新N/K清理，预存global L身份/权限/字节保持并exclusive重获释放，重复临时根逐字归档后删除。无服务/日常配置/真实业务/新包/发布。

生产整根许可仍未构造；root-retirement当前完整树匹配不能直接忽略本次活体写锁。下一步把仅本动作的Guard原Image纳入临时对象连续性，移动后官方正常释放并确认其Absent及完整预期树，才可独立关闭原根缺席；再接正常Stop/launcher/TCP、原件与原deadline、next Start及明确中断入口。不能增永久残留例外或从磁盘重建holder。缺Ready/其它完整IMP范围及固定候选Windows W2仍未完成，目标active。


## 2026-11-13 活体写锁约束的整根事务与完整读回

mainc576a3b75将实际Guard Image绑定到schema2根计划，Original不含该路径、投影只移除准确本动作写锁，其余完整树/Grok和独立原件严格核对。普通入口不能处理guarded计划；当前OFD＋匹配活体Guard＋私有MutationPermit才可移动、明确重锚、官方正常release，再要求精确released tree/读回/独立关闭。磁盘事实不能构造Guard或正常释放许可，原根/锁/数据改变拒绝保全。重新打开live计划也重核Original，不能重写私有baseline采用新增用户文件。详[根临时对象事务合同](0.2.24-contracts/client-root-transient-r1.md)。

首原包组合轮prepare拒绝保留，后继真实清单诊断确认512字节claim-journal；原件比较正确。将正常claim→OFD交接移到拍计划前，config Guard与N/L/K保持，不新增journal例外。只读复核另发现重锚先失败会漏冲突，现先识别实际移动对象并create-only写conflict，补偿仍须当前权限，不明则两对象保全/失败；before_rename在最后检查后注入，真实Guard late例验证无终态。换inode用例旧marker移出client树，单独证明inode不符拒绝。

22个最终相关函数、fmt/diff-check/strictClippy通过；4个真实child exit91在登记/移动/释放/读回边界均保留Prepared4、不从磁盘回执结案。最终原包正例3760ms完成真实Guard/NativeLease＋合成原件/生命周期许可的移动、释放、精确读回和Absent5；真实晚到替换2201ms记录冲突、保全两对象且Prepared4/无终态，均<原10秒。未运行服务或正常Stop，不冒Start30/Status5/Stop10接受。

证据.adg/evidence/0.2.24/client-root-transient-r1/及374文件manifest含失败/差异、最终7源和实际库binary、独立原JSON/SQLite投影、原根/临时Image/释放树摘要事件链与可重放verify_all。原生payload无损gzip保留全字节，Bun缓存派生重复副本清理；诊断cleanup的预期移除字段有勘误，最终真实清理另证。六轮child/holder终止，精确新N/K移除，预存global L身份/权限/字节不变并重获释放；失败Profile/复用运行包/重复临时根在证据归档核验后清理，无日常配置/真实业务/新候选/发布。

下一步接真实当前正常Stop/launcher/TCP与原件来源，构造跨本次物理重锚持续核验的生产许可，再接完整缺根启停、next Start与明确中断操作；当前factory仍缺、Ready/capture guard不解。Windows W1、缺Ready/一般客户端图路径/完整进程集合与刷新/NAME/System代理/发布验证/README/固定候选等原完整范围保持，目标active。


## 2026-11-13 当前正常Stop根恢复许可与独立登记绑定

mainfed7d21ad新增生产RootStopAdmission/RootRetirementAuthority及唯一具体类型factory：完整OS absence在创建config Guard前，持Guard阶段绑定同session/Scope/Live/真实Stop/Pair/NativeLease，正常释放后再完整收尾；字段私有、无serde，不能从磁盘事实/callback制造生产许可。每次checkpoint按plan准确live/archive核对原logical scope/原件/Ready→PreStop→Stopped链、当前程序来源、原record/generation、主和全部捕获TCP/manager/记录缺席及原deadline。详[根正常Stop权限合同](0.2.24-contracts/client-root-stop-authority-r1.md)。

强制RootRetirementRegistration借用真实Index，只从当前RootRetirementPrepared构造；每次复核完整source event、plan、ticket/refs/restorePlan和当前CAS代次，未登记Head副本、错source、已关闭旧凭证拒绝。admit/prepare/retire明确分开，避免把source Head误当已登记Head，保留原归档定位。实现入口已落代码，但还未从完整缺根产品状态机调用，不因此开放Ready/capture/next Start。

32去重相关函数通过（root-stop4/completed-stop7/root-movement23，2重复）、fmt/diff-check/strictClippy通过。内核真实双端口子进程捕获/正常exit0/wait进入显式合成外层Stop谓词，验证secondary端口/未知/manager/记录/owner变化拒绝，不冒官方Stop证书；合成帧根binding在移动前后复验原件Scope/Live/历史/plan，新用户根/篡改拒绝；实际SQLite登记凭证验证未登记副本拒绝、source完整匹配及关闭失效。早期Fixture可见性编译失败、Clippy参数过多及修正原log保留；只读复核指出并确认Guard完整扫描顺序和先登记强约束，未代签执行。

证据.adg/evidence/0.2.24/client-root-stop-authority-r1/含22文件manifest、最终8源/真实库binary、测试日志/失败/复核及核验脚本；所有测试child与子审查已结束，unitTMP空、自有重复临时根在逐字归档后清理。无官方包/服务/全局DB/日常配置/真实业务/新候选或发布。

下一步接缺根原件capture与Active、受证明约束的Ready/PreStop/Stopped及根恢复状态编排，顺序为admit→Guard/plan→真实Index登记→当前许可移动/正常释放/读回/关闭，处理next Start与明确中断，再跑原官方2.99完整隔离30/5/10/30验收。当前factory未做该整链验证，P24-04/05及Windows/NAME/其它原完整范围保持未完，目标active。
