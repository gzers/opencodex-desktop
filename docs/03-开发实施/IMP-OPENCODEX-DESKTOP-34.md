---
id: "IMP-OPENCODEX-DESKTOP-34"
object_kind: "implementation.change"
state: "in_progress"
completion_summary: null
title: "0.2.24 发布后修复与双线集成计划"
target_version: "0.2.24"
demand_ids: ["DMD-OPENCODEX-POST023"]
task_ids: []
summary: "用户已授权目标模式执行；本机P24-00共享程序位置合同开发中，Windows贡献按独占写集Git承接。"
---
# 0.2.24 发布后修复与双线集成计划

revision: 42；2026-11-12。用户要求本机与Windows可独立开发部分并行、Windows分批提交、最终再通过Git交接Windows验收。本文件为本版实际实施顺序/依赖的唯一来源；[TEAM](TEAM-0.2.24.md)只拥有执行角色/写集/回交，不另维护进度。范围及既有采用规则承接[开发前边界](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-后继修复开发前边界确认.md)和[同源矩阵](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-0.2.24开发前收口矩阵.md)。本对话已按用户授权执行开发/测试/集成，Windows W0已回交并注册；当前不授权发布或改日常配置。

当前代码集成基线main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b，发行基线v0.2.23 e3728109e93498bfbd6e993d49904d57dfd75315；目标0.2.24。两个基线不能互相冒用。后继实际开工时核对main新提交；发现新增共享变化由本对话承接。新版本范围已由DMD-POST023承接，本计划已进入in_progress；尚无新正式TASK或关闭证书，不挂旧已发行TASK，也不为排期篡改旧证书。

## 实施与并行顺序

| 编号 | 当前状态 | 工作与输出 | 完成标准 | 证据／未完 |
| --- | --- | --- | --- | --- |
| P24-00 | 开发中 | 本机固定唯一实际程序根、当前版本/安装清单、同HOME、跨平台调用与错误语义；复用2.99原包身份和原生生命周期预算。 | 共享Rust入口/类型由本机单写；Windows按真实代码接口接入，无双重路径猜测/自造JSON清单。 | main9125f7134/3d3c90943：启动供应、来源/依赖/真实磁盘版本已接，当前选择/内部入口与用户CLI分离；93项相关检查和真实Mac官方包供应/版本探测通过。更新切换/CLI/卸载合同未完，W1未完整放行。 |
| P24-W0 | 已接收并注册，产品接线/失败目录清理未完 | Windows将r5验证过的TokenUser/受保护记录创建/保留进程句柄/Job集合/正常退出/原生监听与路径事实整理为独立Rust平台后端，带同源原生定向测试。 | 真实Win32事实而非桩；未知拒绝；不要求旧维护包lifetime字段，不强杀补通过；生产模块可被独立测试宿主编译。 | 贡献b2e89665d/证据0eef59a01已逐项核对，主线2d68a28bc按净树集成注册；4个Windows失败目录仍在，未伪报清理或产品整链通过。r5输入保持。 |
| P24-01 | Mac保留配置的卸载/重装及原登记同版本修复已接；其它来源/跨平台原生与整包验收未完 | 本机实现POST-01/02程序/配置分离、Mac标准单.app、用户可写供应、安装/更新/回退同一来源；POST-07磁盘/运行/历史版本分开识别。 | 默认与自定义根、只读资源、来源漂移、校验失败保全；清单/切换/保护单一来源。 | 依P24-00；Windows W0并行，不等完整UI或名称路线。 |
| P24-02 | 共享保存/手动备份/恢复与现有安装事务保护已接；版本化升级接线及候选验收未完 | 本机POST-04保存重核、POST-08原生JSON单层导入/完整备份/回退，复用真实配置事务。 | 保留草稿/密钥意图、有限等价变化不误锁，真实冲突拒绝；导入仅保留本机port和模式，其它按源替换，备份失败零覆盖。 | 后端8例及原型已定；不把Read的模版匹配暗当无副作用刷新。 |
| P24-03 | Mac终端/通知/网络/内置准备及版本事实/更新前重核已接；Windows/PATH和原生候选未完 | 本机POST-03同版本同HOME CLI核心及Mac入口；POST-05环境阶段/就绪区分、POST-06应用网络/更新说明；已采用UI接线。 | CLI显式启停/冲突不覆盖；内置环境不依赖系统Node；npm/GitHub错误准确；原型一致。 | 03路径/CLI接线依01；网络/UI可先独立推进。 |
| P24-W1 | 原生PATH/安装器024资源部分已放行；完整共享接线未完 | Windows后端接共享调用合同；Windows当前用户CLI/PATH和自定义安装根/版本目录/卸载平台适配。 | 实际运行版本/HOME/安装根一致；用户级入口/PATH有归属与撤销，安装更新不误删配置或别用户对象。 | W0可先提交；共享mod/入口/版本/工作流由本机接线；不能先猜清单文件名再返工。 |
| P24-04 | Mac当前版本化生命周期已接，实际启停/状态通过原预算；客户端完整恢复与Windows接线未完 | 本机接收W0/W1，核对来源与写集，接共享模块/命令/配置和生命周期/事务；Windows修本线集成编译问题。 | 最终合并树跑真实共享流程；分线通过不当集成通过；同30/5/10秒和未知拒绝保持。 | 小批次回交，不等Windows末轮才合并；保持开发重叠。 |
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
