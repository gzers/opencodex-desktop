---
id: "IMP-OPENCODEX-DESKTOP-34"
object_kind: "implementation.change"
state: "in_progress"
completion_summary: null
title: "0.2.24 发布后修复与双线集成计划"
target_version: "0.2.24"
demand_ids: ["DMD-OPENCODEX-POST023"]
task_ids: []
summary: "0.2.24 收缩 Desktop 所有权：渠道、模型、协议、密钥与原生配置归 OpenCodex；Desktop 只保留只读备份与自身独立范围，Windows 原生生命周期仍待补验。"
---
# 0.2.24 发布后修复与双线集成计划

revision: 223；2026-11-22。用户要求本机与Windows可独立开发部分并行、Windows分批提交、最终再通过Git交接Windows验收。本文件为本版实际实施顺序/依赖的唯一来源；[TEAM](TEAM-0.2.24.md)只拥有执行角色/写集/回交，不另维护进度。范围及既有采用规则承接[开发前边界](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-后继修复开发前边界确认.md)和[同源矩阵](../04-项目资料/01-项目支撑资料/06-问题记录/20261110-0.2.24开发前收口矩阵.md)。本对话已按用户授权执行开发/测试/集成，Windows W0及W1三批组件已回交并注册；当前不授权发布或改日常配置。

初始代码集成基线main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；当前共享检查点main 9daf5599151b65437f94ab2cadb1988060ee38c3（非冻结候选）；Windows实现检查点codex/024-windows-native e54a1156de42d445903faedb93aed4923f400b3a（旧r2实际载入357210891）（未整批集成main），发行基线v0.2.23 e3728109e93498bfbd6e993d49904d57dfd75315；目标0.2.24。两个基线不能互相冒用。后继实际开工时核对main新提交；发现新增共享变化由本对话承接。新版本范围已由DMD-POST023承接，本计划已进入in_progress；尚无新正式TASK或关闭证书，不挂旧已发行TASK，也不为排期篡改旧证书。

## 实施与并行顺序

2026-11-22 范围修正：Desktop 不再自管渠道、模型、协议、密钥与 OpenCodex 原生配置，也不再接管真实 ~/.codex。新增渠道按原型改、渠道改名不生效、2.99 配置导入迁移均归 OpenCodex，不在 Desktop 本版范围。详见[所有权收缩合同](0.2.24-contracts/desktop-owner-scope-correction-r1.md)。


| 编号 | 当前状态 | 工作与输出 | 完成标准 | 证据／未完 |
| --- | --- | --- | --- | --- |
| P24-00 | 开发中 | 本机固定唯一实际程序根、当前版本/安装清单、同HOME、跨平台调用与错误语义；复用2.99原包身份和原生生命周期预算。 | 共享Rust入口/类型由本机单写；Windows按真实代码接口接入，无双重路径猜测/自造JSON清单。 | main9125f7134/3d3c90943：启动供应、来源/依赖/真实磁盘版本已接，当前选择/内部入口与用户CLI分离；93项相关检查和真实Mac官方包供应/版本探测通过。更新切换/CLI/卸载合同未完，W1未完整放行。 |
| P24-W0 | 已接收并注册，产品接线/失败目录清理未完 | Windows将r5验证过的TokenUser/受保护记录创建/保留进程句柄/Job集合/正常退出/原生监听与路径事实整理为独立Rust平台后端，带同源原生定向测试。 | 真实Win32事实而非桩；未知拒绝；不要求旧维护包lifetime字段，不强杀补通过；生产模块可被独立测试宿主编译。 | 贡献b2e89665d/证据0eef59a01已逐项核对，主线2d68a28bc按净树集成注册；4个Windows失败目录仍在，未伪报清理或产品整链通过。r5输入保持。 |
| P24-01 | Mac保留配置的卸载/重装及原登记同版本修复已接；其它来源/跨平台原生与整包验收未完 | 本机实现POST-01/02程序/配置分离、Mac标准单.app、用户可写供应、安装/更新/回退同一来源；POST-07磁盘/运行/历史版本分开识别。 | 默认与自定义根、只读资源、来源漂移、校验失败保全；清单/切换/保护单一来源。 | 依P24-00；Windows W0并行，不等完整UI或名称路线。 |
| P24-02 | 已收口，撤回重复实现 | 保持只读原生配置备份（安装/修复/卸载前）与 Desktop 自有端口、偏好、扩展设置；渠道、模型、协议、密钥与原生 config.json 的编辑、导入、保存、回退归 OpenCodex。 | Desktop 不含任何写 OpenCodex 配置或接管真实 ~/.codex 的路径；安装前仍能生成完整只读备份，配置不可写时如实失败。 | main958029879 删除全部渠道/客户端接管命令与界面，新增只读 native_config_backup；详见 [所有权收缩合同](0.2.24-contracts/desktop-owner-scope-correction-r1.md)。 |
| P24-03 | 已修正 CLI 端点与停止后状态；真实候选启停通过 | 本机 POST-03 同版本同 HOME CLI 核心及 Mac 入口 | CLI 与 GUI 使用同一端点目录；停止后状态如实为未运行 | main b6785b486：端点共用同一相对路径常量并读取活动数据根；无本实例记录时不再调用官方 CLI。详见 [CLI 端点与状态合同](0.2.24-contracts/desktop-cli-endpoint-and-idle-status-r2.md)。 |
| P24-W1 | 已判定：Windows 最新交接不整批合并；需按新范围重建 | Windows 以 main b6785b486 保留程序根、入口与 PATH、安装器资源、真实正常启停 | 原 30/5/10 预算与未知拒绝不变；不含客户端原件接管 | 6532cbb6e/32b0483e3 净差为已撤回链；本机无 Windows 工具链，结论来自逐文件审查。详见 [交接处置](0.2.24-contracts/windows-handoff-scope-disposition-r1.md)。 |
| P24-04 | 已撤回 | Desktop 不再接管真实 ~/.codex 或保管客户端原件；OpenCodex 使用 Desktop 指定的 OPENCODEX_HOME/CODEX_HOME 自行管理客户端文件。 | 启停只保留已有认证的 Start / Stop；真实用户客户端目录全程只读。 | 撤回链留档 codex/0224-root-cancel-archive@436edfe63；隔离启动证明 ~/.codex 的 inode/mtime/权限未变。 |
| P24-05 | 官方 2.99 已通过产品路径 Start→Status→Stop（一次性 root，客户端 HOME 落在 Desktop 自管目录）；Windows 安装器缺原生入口来源文件的问题已修，真实打包与真机仍未验 | Mac 真实原包、必要安装/CLI、固定 Windows 集成候选 | 精确代码 SHA、官方包版本/SHA、受影响原生验收表 | main df4438a35 新增真实产品启停验证；main ec11a25e0 补上安装器要求的 ocx-native-entry.json 生成步骤。详见 [产品验证合同](0.2.24-contracts/desktop-product-verify-r1.md) 与 [原生入口来源合同](0.2.24-contracts/windows-native-entry-provenance-r1.md)。 |
| P24-W2 | 最后交接，当前不跑 | 用户把本对话生成的最终简短提示词交Windows；针对固定集成候选，一轮采集所有受影响原生项状态/证据。 | 双SID真实桌面并行、受保护生命周期/恢复及原预算、路径/CLI/相关安装更新卸载；失败保留，未知不通过。 | 按最终diff选必需项。不是旧r5第六轮，也不在未集成源码上提前宣称产品通过。 |
| P24-06 | 进行中：Mac 候选启停与 DMG 已验；测试与安装器缺口已修；Windows 真机、扩展写入口径与原型差异待闭环 | 收口所有必需项并固定候选 | 每项有可复现证据与清理，无假放行 | main 6dd13ccd6：恢复被误删的前端测试并修回路径刷新、端口刷新与通知去向。详见 [测试恢复合同](0.2.24-contracts/ui-test-restoration-r1.md)。 |

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


## 2026-11-13 首次缺根生产捕获和typed阶段接线，Stop10仍失败

main59ec750f6接Active首次NotFound创建custody/独立CaptureReserved/原Absent schema5 Original/当前Scope/Captured，Ready/PreStop/Stopped由私有typed RootPhaseProof原签名就绪/正常退出/监听证明在Index提交前后核验；普通advance拒绝不解除。Stop接原件恢复→Guard前完整admit→新Guard/根计划→实际Index登记/registration→当前normalStop根factory→根读回/关闭，所有协调释放和最终来源/预算仍须成功。canonical普通重开与诊断stop_ok已修；RootAbsent UI/nextStart/根中断处理尚未接。详[首次缺根生命周期合同](0.2.24-contracts/client-first-root-lifecycle-r1.md)。

四原包隔离轮首次Start<30s均通过，服务/launcher正常exit0、4捕获监听均缺席；完整Stop均失败。native10278ms/diagnostic10273ms停RestorePrepared7，trace确认artifact已恢复且writer正常release，post-apply-stopped超时。Index同调用重复coalescing后达到RootRetirementPrepared8但Stop11523ms仍timeout；目录fd双轮Protection优化a325f289a后Stop10338ms（manager10080ms）仍在root-retire拒绝，原root未移动、completionnull、无Retire读回。失败全部保留acceptedfalse，不把正常退出或登记计划冒完整Stop成功。

优化保持完整核验：Index checked_head一次完整resolve同observedHead、末尾重读相同，无跨调用cache；proof由原bind/seal产生，Index前后require。目录pin保留句柄/逐级nofollow相同inode与07777/owner/flags，双轮完整ACL/xattrs+末轮路径，避免重复open/第三次属性读取。0777误比导致0过2失败保留；只读审查发现扫描中ACL/xattrs窗口后补第二完整读取与确定性xattr负控。最终26快照/36索引/1canonical共63去重定向通过，fmt/diff/strictClippy过；最终test-only lint差额有exact test，生产源码对应最后失败原包轮。早期115过1ignored只属初接线阶段，不冒最终源码。

证据.adg/evidence/0.2.24/client-first-root-lifecycle-r1/及132文件manifest约1.08MB，四轮真实原始输出/固定阶段耗时、source/binary、无损原字节和独立SQLite链/Origin/Scope/Live/plan核验俱在。全部test/native/子审查句柄终止；精确12个新N/K/historyDB经exclusive重获删除，预存global L身份/权限/字节不变并重获释放；独立清理复核主端口，产品4监听证明不冒更广清理。先逐字归档四Profile原件后清四临时根/复用runtime/重复证据，unitTMP空，未改日常配置/官方源码/真实业务，无新包或发布。

下一步限定root-retire剩余耗时：按当前authority/Index/frame/native子阶段精确测量再修，同预算/判定/读回不放宽，无新假设不重跑；通过后接nextStart/RootAbsent UI/明确中断并整链复验。当前Stop门失败，P24-04/05未完，Windows W1/W2、NAME和其它原完整版本范围保持，目标active。


## 2026-11-13 Stop根许可成本测量与同次核验/持锁事务合并

mainea9220c70按真实计时合并同次重复：cost-diagnostic一次authority341ms（Index125/source215），root current_prefix复用同次完整已核验四帧避免open/Original/Ready/Stopped重读后source约68ms；原frames/前驱/Protection/独立SHA/blob/当前树/Grok/source仍完整。工厂仅删实际mutation gate之前的重复entry check，写/移动/释放/回执/最终absence周围检查保留，无跨动作cache。详[Stop成本合同](0.2.24-contracts/client-root-stop-cost-r1.md)。

首根Stop把完整admission移在artifact恢复前，一次准确Codex Guard贯穿原目标恢复和根retirement，到根正常release后最终full absence/完整读回才可能结案；普通会话独立restore不变，Grok仍用独立Guard，失败Drop不冒成功。RootAuthority同checkpoint用当前Index binding/initial source event/plan/Head/VFS/action事实夹住一次完整原件/四帧/scope/live/当前程序来源；初始registration/普通require仍完整，窄facts不独立授权限，Stop/Pair/全部端口/记录/期限保持。

四原包定向轮Stop仍失败：cost10310ms/prefix10372ms原root仍在；retained-writer10287ms已真实移动并relocated但无Retire读回；facts-coalesced10334ms已正常writer release、精确Retire readback存在，但Index仍Prepared8/null completion，期限内关闭未完成。最后一次authority约80ms而非341ms，局部成本减少不冒预算通过。四轮Start<30s、service/launcher exit0和4捕获监听缺席，不代替完整Stop。所有acceptedfalse，nextStart/根中断/状态UI仍未完。

最终52去重函数（4Stop/23move/26originals重1）、fmt/diff/strictClippy通过；复杂tuple及RefCell temporary编译失败日志保留，类型别名/局部结果修正，不禁lint。三次只读审查确认prefix/同Guard事务/事实夹完整观察未见直接弱化，未代签native。最终6源和实际库binary与最后失败原包轮完全对应。

证据.adg/evidence/0.2.24/client-root-stop-cost-r1/及122文件manifest约1.35MB，原计时/失败/源/独立原字节与SQLite/帧/plan/readbackSHA核验俱在。四轮child/子审查终止，精确12个新N/K/history经exclusive重获移除，预存global L身份/权限/字节不改并重获释放；独立清理复查主端口，4监听产品证明不冒全清理再验。原字节逐字归档后清四Profile/运行包/重复证据，unitTMP空，无日常配置/官方源码/真实业务/新候选/发布。

下一步精确拆分最后readback后的authority/full absence/Index-close及外层来源检查成本，形成新假设再跑，不盲重跑失败原轮；同10秒/实际mutation前后完整校验不放宽，整Stop通过前不标Restored或新Start可用。然后仍需nextStart/明确中断/状态UI及Windows/NAME/其它完整IMP项，目标active。


## 2026-11-13 根Stop收尾定位与提交前去重，原门仍未通过

mainf771c5e2b记录固定测试tail阶段并强化逐帧负控；46952673e将完整resolve已核验的Originals在同次Index观察中返回，registered_root_originals不重开同源原件；入口只删随后完整事务验证覆盖的重复检查。BEGIN前storage pins/VFS仍在，BEGIN内任何引用SQL前完整root/Head/binding，写后check/ROLLBACK、COMMIT后完整head及proof期限/demotion保持。详[Stop收尾合同](0.2.24-contracts/client-root-stop-tail-r1.md)。

两方案明确否决：shared-prefix前后context代替逐帧可漏中途改动再恢复，生产Originals变更完全撤回，额外新末尾fence亦撤回；helper前复用早先完整包准入会漏执行中途改动模块，未实施，read_complete保持。负控注入独立assert成功，Original改755后计划PreStop恢复必须在Ready先拒绝，逐帧规则未放宽。只读复核不代签native。

最终87去重函数（28snapshot/36index/23root，4ignored不计）、fmt/diff/strictClippy通过。四新Profile Stop均失败：否决prefix实验11027ms曾Absent9后过期退Prepared10（只原始定位不可验收）；两中间版本10379/10255ms；最终original-context10493ms（manager10253）停Prepared8，原root仍在/无终态。最终5源和实际库binary完全对应，首轮否决源仅hash/log未单存patch，不宣称完整源码可复建。旧实验更靠后的进度不代替最终来源通过，原Stop10不放宽、nextStart/状态UI/根中断仍未完。

证据.adg/evidence/0.2.24/client-root-stop-tail-r1/含127文件manifest、原假设/trace/失败/无损原字节与独立SQLite/RootBinding/phase核验、复核和来源。所有进程/子审查结束，逐字归档后清四Profile/运行包/重复临时根，unitTMP空，无日常配置/官方源码/真实业务/新候选/发布。清理审计缺口明确：首工具回执12精确新DB删除，复查误覆盖逐inode明细为空，归档不冒存在该明细；准确目标路径和再次全部Absent仍核验。global L身份/权限/字节保持并重获释放，独立清理只查导出主端口，不冒4监听再次全验。

下一步保持helper完整来源/逐帧动态检查和实际写入边界，测量完整inventory并评估同等核验下的并行/批量I/O；无可证伪方案不开新Stop轮，不采用旧摘要cache放行。完整原范围Windows/NAME/其它客户端/代理/版本发行收口保持，Stop和整版未完成，目标active。


## 2026-11-13 完整镜像有界并行核验与release首次缺根30/5/10通过

main97a3bf93b按隔离测量新增最多4 scoped reader的fresh完整inventory：原每文件/条目/空目录/可执行位/SHA与manifest排除保持，每job紧邻check_path，实际opened before.len读取前共享Atomic预算≤2GiB，原额外EOF1字节探测保留（坏在途最多4探测字节），全部spawn/Err/panic取消并join，不返回partial；统一seen避免名冲突，最初目录metadata及原枚举末检保留，所有reader完成后再核所有路径/目录mtime及Unix身份，Windowscreation/attrs只是附加不冒fileID。helper前完整内容准入与最终完整来源复验没有cache/跳过。详[并行镜像合同](0.2.24-contracts/image-inventory-parallel-r1.md)。

原官方15646项/13393文件/645549518字节，串行基准774801us；最终serial→parallel1970408→617526us，反向parallel→serial655807→694484us，清单逐项和完整树SHA均相等。缓存顺序影响明确，不宣传稳定3x；worker耗时累加不冒wall time。33去重debug回归（镜像10/来源11/repair5/removal7）、两release来源各21负控、fmt/diff/strictClippy过，新增并发预算/EOF增长/links/ancestor/path/目录变化/worker错误全部拒绝且cleanup前已join。

debug原包Start8877/Status228，Stop10754ms（manager10352）RootAbsent9已提交但最终来源复验超时，保留失败。额外交付profile（原Cargo release opt=s/LTO未改）同源码/同原30/5/10断言两次首次缺根完整启停通过：首release5991/225/7836ms；当前97a3bf93b release5979/224/7913ms。每轮正常service/launcher exit0、4TCP缺席、官方writer正常release、完整原件/Retire读回/RootAbsent9、外层协调release及完整来源复验均成立。acceptedfalse、非app/安装候选，debug失败不被覆盖；关键gate无debug开关降断言，library panic与实际app范围另记，固定app仍要验。

证据.adg/evidence/0.2.24/image-inventory-parallel-r1/含140文件manifest约311KB、不同source/binary/build配置、完整对照/原输出/失败＋release通过/负控和独立SQLite原字节。所有编译/test/native/agent句柄已实际结束；精确9新N/K/history逐轮exclusive重获后移除，预存global L身份/权限/字节保持并重获释放；清理独立查主端口不冒4监听额外全验。先逐字归档再清1benchmark＋3native根/复用runtime/复制binary/重复证据，unitTMP空，工程target保留，无日常配置/真实业务/官方源码/新appDMG/Windows候选或发布。

下一项明确nextStart：只读分析表明first-only marker/初binding/Preparing1/capturecreatedfd与代内origin.id==attempt、vault历史缺RootAbsent闭合支持不能直接放开reserve。拟保旧archive，每后继独立generation+attempt，先历史精确验证→index/events追加代际来源/前驱→create-only新代custody→atomicPreparing→同源readonly后继资格驱动状态/实际Start→两完整release原包循环及中断/CAS/新root反例；未实施。Stop回执canStarttrue不冒已支持第二次启动，RootAbsent状态Required不一致待一并接。缺Ready/一般图路径/进程集合与刷新/NAME/复杂代理/Windows W1W2/版本发行收口全目标保持，P24-04/05未关、目标active。


## 2026-11-13 根闭合历史独立核验，后继generation仍待登记

main0ac19863e新增仅事实的open_closed_history：原独立source/closed/origin、准确plan/intent/readback/completionSHA、原archive身份/Protection/完整树、schema5原件/phase/blob/scope/live及旧Grok归档严格核验；当前codex/Grok变化不当旧baseline。当前RootAbsent入口仍要求实际缺席/当前恢复，历史入口不创建Start/move许可。详[根闭合历史合同](0.2.24-contracts/client-root-closed-history-r1.md)。

Index借同SQLite/action核有界canonical完整events，所有事件首尾/阶段连接/refs/live前缀/注册决策检查；refsNone未闭合事件或未知generation不能被略过。source+1初Prepared与closed-1 prepared必须完整对应plan/source/ticket/refs/restorePlan，原source文档比不可变plan，exact vault names和观察后全部原件/索引再查保持。现只支持独立初origin首代，故明确拒绝其它代次ticket，不假称已经多次启动。

最终14新增历史场景、根恢复总37、索引36共73去重函数通过，4ignored不计，fmt/diff/strictClippy通过。新root/Grok与历史分离、空archive同内容换inode、原件/回执/属性/缺provenance或plan/blob/extra vault/refsNone事件/中间Prepared换hash以及投影时binding/action/events改变均拒绝保全。完整fourphase/live/schema2 marker释放为明确合成storage场景，非官方Stop证书。只读审查初两漏检已修复并补实测反例，不代签nextStart。

证据.adg/evidence/0.2.24/client-root-closed-history-r1/含30文件manifest约1.32MB、最终6源/真实库binary、日志/静态审查/原base64JSON与独立SQLite引用核验；早期7/8项仅其阶段证据。全部test/agent子进程终止，TempDir清理、unitTMP空、逐字归档后删重复根。未执行官方runtime/服务/全局DB，无日常配置/真实业务/新候选/发布。

下一步仍需newgeneration create-only custody、独立前驱关闭/本代origin/scope/Preparing原子登记及状态/实际Start同源资格，再至少两完整release原包循环和中断/新用户root/CAS/deadline负控。当前未开放RootAbsent→Start，首次release通过不冒第二次，P24-04/05与Windows/NAME/其它原全范围保持未完，目标active。


## 2026-11-13 第二代root创建与原子Preparing登记原语

main57ae63019新增Index持有的后继创建：准确首代RootAbsent/当前缺根/完整旧历史后，clone原action open-file-description/flock，create-only第二代槽/candidate/创建fd/原deadline/EXCL发布，rootGeneration（新origin/scope、初Preparing revision、前驱关闭revision/原编码SHA）和Preparing在同一Index事件/currentHead CAS事务中提交。原singleton/binding/旧events/原件/archives不改；Head可选字段仅Preparing序列化，References.generationSlot绑定本代UUID，SQLite仍原3表/userVersion3，无旧版迁移。详[第二代原语合同](0.2.24-contracts/client-root-generation-r1.md)。

提交前实际binding/全部旧闭合archive/候选newscope+provenance+空root+createdfd完整校验，SQL后再校验再COMMIT，提交后再检查；gen槽及generations/manager父目录在DB引用前sync。只读审查发现并修正SQL尾部漏查和父目录sync，fsync顺序不冒物理断电验收。读取按原初binding＋本代Preparing独立来源解析，scope前8上下文相同/clientidentity不同、前驱SHA和旧source/close严格，未知slot/更多未支持generation拒绝。

8新增后继/索引总44/根恢复37/首创建8共89去重函数过，4ignored不计，fmt/diff/strictClippy过。真实exit91五点reservation/rootpublish/eventinsert/headupdate/committed，独立bytes只旧Absent5或完整Preparing6，事务内rollback sidecar保留，磁盘重开不恢复createdfd/capture权；外部真实进程证明同action锁持续。四提交点×两实际变更（newfile/oldarchive）保证precommit两引用回滚，postcommit Preparing保留但Err且外部字节不删。旧binding/vault原字节不变，canonical source/plan/provenance/原件及scope均核验。

证据.adg/evidence/0.2.24/client-root-generation-r1/36文件manifest约2.55MB固定10源/实际库binary、原失败边界、独立SQLite/五现场raw-byte核验与清理；初Path类型错误仅chat原输出，unusedimport严格Clippy失败保留log。全部test/probe/agent句柄终止，TempDir和unitTMP清，逐字归档后删重复临时根；未执行官方服务/runtime/全局DB，无日常配置/真实业务/新候选/发布。

当前只第二代creation+Preparing原语，生产Start未调用；普通capture/reserve/首代capture仍拒绝第二代，第三代及多代history/状态资格未集成，不能声称两次原包启动通过。下一步专用第二代capture/Original/typed阶段链与多代历史reserved边界，再统一readonly资格接Start/状态并跑两完整release原包循环/中断反例；RootAbsent UI/更多循环和Windows/NAME/其它原全范围未完，目标active。


## 2026-11-13 第二代专用捕获与原件独立登记

main d4af2abb2新增专用 successor capture reservation/Original登记，保留首代入口与普通Start/Ready拒绝。只在准确本代初Preparing、独立前驱关闭SHA/本代origin/scope、原action/createdfd/原deadline/空根下预留CaptureReserved；rootGeneration仅保留最初不可变event，后继Head清空并按ticket解析。Original按本代active binding核scope和原件/范围摘要及HOME/根identity，首代singleton binding/旧events/旧归档不改。详[第二代捕获合同](0.2.24-contracts/client-root-successor-capture-r1.md)。

历史核验逐事件约束本代阶段和不可回退引用；旧闭合archive继续完整核对，整个vault必须恰为已闭合attempt加唯一已登记当前attempt。当前仓库仅投影private目录identity/protection/statversion/直接子名，不把partial bytes当原件；Captured仍完整核frame/blob/scope。聚合枚举采用独立有限条目预算，客户端单树/字节/文件准入不变。静态审查发现的单树预算误用与16384项上限已修，17MiB/16385项partial只能观察事实，create-only不采用。

原SQLite连接/action/CAS在SQL前、SQL后完整候选核验后、COMMIT读回后检查capture custody/deadline；四个真实边界注入证明SQL后过期或根被替换回滚event+Head，提交后返回Err并保留已提交事实/原件/外部根。摘要正确但HOME/client身份、逻辑路径或Original引用错误同样拒绝。

最终112个唯一受影响函数通过：第二代12、旧索引44（排除已单独运行的第二代12）、根归档37、首代原件9、树10；5项ignore不计，新增13项。strict all-target Clippy/fmt/diff通过。第二代schema5 Original和scope独立登记可事实重开；合成storage close保留两代归档/首代字节不变，不能冒正常服务Stop。三个实际子进程在reservation后、Original发布后及Captured提交后exit91，父wait91；原字节SQLite/JSON离线核验通过，重开不重建createdfd或capture权限。

证据.adg/evidence/0.2.24/client-root-successor-capture-r1/固定七源/实际library-test binary。binary在父57ae63019加当前七源差异下构建，源码映射本提交，不冒嵌入commit字段已重建。初轮只读夹具篡改准备失败保留原log，后续修夹具自身权限而非产品补权限；早期9/12项版本和最终聚合枚举版分别保留。无官方runtime/service/全局协调DB、日常配置/Shell/已安装app、业务WebDAV/模型请求或新固定候选；不宣称断电测试/整版验收或发布。归档逐字核验后清理本轮临时夹具和重复目录，工程target保留。

下一步扩展各代独立闭合历史与indexed generation槽集合，之后统一只读后继资格接生产Active/Start和状态，处理明确中断，再至少两完整release原官方包循环/受影响负控。本批生产Start未调用专用第二代入口、第三代仍拒绝，RootAbsent状态/缺Ready/其它客户端图路径/进程集合和刷新/NAME/System代理/Windows W1W2/版本发行与固定候选等原全范围不缩减，P24-04/05未关闭，目标active。


## 2026-11-13 多代来源、闭合审计与存储滚动

main ef00dd58c引入同一持锁SQLite canonical history的纯root_lineage，完整分代核首代singleton/Preparing1、每代唯一origin/ticket/slot及紧邻前代RootAbsent原编码SHA、保持scope前八项并拒绝复用任何旧root identity、代内阶段及Original/scope/live/phases/恢复决定不回退。重复关闭的早先completion也必须与同一固定plan/source最终receipt一致；事件4096、history4MiB、单事件65536及代数512上限保持。详[多代合同](0.2.24-contracts/client-root-lineage-r1.md)。

每代归档使用自己的origin/scope/source/初次plan登记/最终Prepared/readback，重复审查和全history/binding/Head读回；current .codex/.grok不作为历史事实。RootRefs只增加serde跳过的私有namespace事实上下文，持久五字段/原字节不变，Eq比较持久字段后每次leaf实查目录集合。封闭Lineage派生typed namespace；Transfer private seal、pending审计只返回空值；Frame使用owning origin，解码Plan匹配后继承上下文，provenance/每帧/prefix回调/独立归档文件返回前都复核名字。无文件能恢复该上下文或创建权限。

普通观察准确S，活体create-only Reservation/Transfer准确S+p；创建前clone原action并审查S，事务旧归档只接真实Transfer核对S加自身p，candidate写入后从全events重算S′，不学习未知slot、不新建第二本账或辅助SQLite fd。新旧capture/原deadline/createdfd/空根/fsync/CAS及已提交失败保留规则不变。

最终118个唯一受影响函数通过：rollover8、旧索引/捕获56（排除本轮rollover8）、根归档37、首代原件9、根创建8；4项ignore不计。strict all-target Clippy/fmt/diff通过。四代真实文件系统storage闭合证明首代binding/原events/每份旧archive不变及完整历史可重开，不当作服务Ready/normalStop。第三代三个真实子进程在published、SQL Head更新未COMMIT、COMMIT后exit91，父wait91；未提交完整Head与原predecessor逐字对应，提交后Preparing11与前驱原SQLite bytes SHA绑定，重开不重建custody。各代原件/scope/provenance/plan/readback和四代全raw files离线核验通过；不冒断电证明。

证据.adg/evidence/0.2.24/client-root-lineage-r1/固定10源/实际library-test binary；在父d4af2abb2加这些差异下构建，映射本提交不冒嵌入commit已重建。初始单storage和六项版本分别保留，最终8项/回归对应最终源；本轮无失败产品测试，草稿工具转义错误保留在tool transcript。归档逐字核验后清理本轮TempDir和重复临时根，工程target保留；没有官方runtime/service/全局协调DB、日常配置/Shell/安装app或真实WebDAV/模型请求、新固定候选/发布。

Windows远端仍b2e89665d/0eef59a01，无新贡献；不为Mac-only本批重跑Windows。下一步统一readonly后继资格接Active/生产Start和状态、明确中断入口，然后至少两完整release原官方包循环及受影响反例。缺Ready/一般客户端图路径/完整进程集合和活跃刷新/NAME/复杂代理/Windows W1W2/版本发行与固定候选等原全范围保持，P24-04/05未闭合，目标active。


## 2026-11-13 后继启动准备与状态同源接线

main 8b0dd88df新增Index::prepare_start_capture供真实Active入口使用：无index首根仍create-only首次登记；当前准确RootAbsent在原action owner内核资格并创建后继；既有普通根保持原路径。Active显式区分首代/后继的专用reservation和Original登记，未完成Preparing/CaptureReserved重开不能续捕获或刷新createdfd/deadline。详[后继启动合同](0.2.24-contracts/client-root-start-r1.md)。

held Index和单次readonly observer/OfficialStatusSource/recovery-preview调用同一verified source helper。准确闭合/全history/origin/current absence/namespace可用后，再以准确scope和最大新根数字/UUID/hash编码宽度预检下一完整常规九事件容量，保持原4096events/4MiB/65536单事件/512代，不新增低统一预算；容量不足在目录创建前拒绝。状态NotRequired和预览Restored与实际入口一致，新Preparing/Captured为Required，未知仍Unknown/拒绝；未闭合Prepared保持Pending，使用已有状态/控件，无新持久或IPC字段。

observer projection移入原只读SQLite事务，根状态提供全history；callback后完整resolve、binding/Head/history、VFS/paths/pins/namespace/archives再核，关闭后核返回边界。为避免immutable SQLite缓存掩盖外部同inode改写，通过开锁前原pin FD read_at核完整有界索引bytes+statversion并比较，无新辅助SQLite fd。已有同进程锁/共享action排他回归保持。持续变化拒绝，不声称消除任意外部最后瞬间竞争。

127个唯一受影响函数通过：产品准备8、旧索引/捕获56（排除产品8和多代8）、根归档37、多代8、命令4、状态来源14；5项ignore不计；strict all-target Clippy/fmt/diff通过。真实OfficialStatusSource的NotRequired→Required与预览/Start合成场景一致；第一/第三代闭合后准备到Captured，旧字节readonly不变。4095合法重复闭合事件反例在原界限内仍事实可读，但status/preview/实际准备不放行，无新root/slot/event。实际外部child成功改写同一inode、dev/ino不变，原readonly观察拒绝；测试显式自有canonical /private/tmp前缀，未在parent另开SQLite fd。

证据.adg/evidence/0.2.24/client-root-start-r1/固定8源/实际library binary，在父ef00dd58c加这些差异下构建，映射本提交不冒嵌入commit已重建；初6项与中间8项/最终8项分开保留。日志schema-canonical核验独立重算前驱Head编码SHA，确认新来源及revision关系；它是日志补充，不冒磁盘raw replay。无失败产品测试，review发现的容量/cache/外部fixture边界已补齐；source/hash和归档逐字核验后清理TempDir/本轮重复临时目录，工程target保留。

当前没有执行官方runtime/service/全局协调DB、日常配置/Shell/已安装app或真实WebDAV/模型请求，没有新固定候选/发布。下一步扩展现有release原包隔离用例到至少两完整Start30/Status5/normalStop10循环，验证真实第二代阶段链、正常退出、旧归档和状态；Stop动作完成回执的可启动门（尤其容量耗尽边界）及根中断/相关UI原型收口仍须随实际链核验，不能把本批接线/状态测试冒完整门禁已关。Restart30及Stop子预算不变；Windows W1W2/缺Ready/其它客户端图路径/完整进程集合与活跃刷新/NAME/复杂代理/发行验证/README和固定候选等原全范围保持，P24-04/05未关，目标active。


## 2026-11-13 原官方双轮启停诊断：第二次Stop失败保留

main 2e74359b4只扩展test host和cfg(test)连接访问器，产品基线8b0dd88df及原官方2.99未改。现有codex-absent用例可显式OCXD024_ROOT_CYCLES=2：首轮签名状态/H/PID/port及5秒、完整Stop DTO先准入；释放Index后读实际status/preview资格；再完整第二Start/Status/Stop。每轮核独立九阶段链，第二Ready document精确在最终events内；旧binding/events原document字节和完整archive内容/身份/权限在第二Start后和Stop后均比；readonly前后manager-state snapshot/config bytes一致，preview revision/attempt/phasecount与关闭源匹配。test-only optin unwind helper只在panic且该轮未尝试Stop时尝试正常Stop一次，结果不能补通过，已有失败Stop不重试。

唯一实际native轮使用release opt-level=s/LTO宿主与固定原包：首轮Start8199ms、签名Status224ms、normalStop8782ms，停止status1880ms/NotRequired/canStart及Restored预览通过；第二轮Start20700ms、Status2205ms通过，normalStop返回lifecycle_timeout，外层12878ms（产品内部11821ms），Stopped且clientRecoveryRequired=true/canStart=false/无Stopped result。独立Head停在RestorePrepared16，无rootRetirement/completion，第二根保留。两个原始launcher/service均正常exit0、记录已移除、两个导出primary端口内核表无监听；这些物理事实不替代正常Stop成功。整轮accepted=false/result failed，不放宽30/5/10，不重跑失败轮。

raw-byte离线核验16条SQLite事件/当前Head、两代provenance/Original/scope及Ready/PreStop/Stopped完整摘要链；首代完整plan/readback及原binding/前9条event字节不变，第二代没有伪造closed completion。实际旧archive完整snapshot在Start/Stop后保持。全局预存L的inode/uid/gid/mode/flags/ACL/xattrs/bytes/SHA及sidecar缺席在最终rw exclusive连接关闭后再核不变；不声称chmod未触ctime。新3个N/K/history按准确逻辑路径/长度前缀hash只允许本轮新对象，观察first-seen身份，原字节fsync回执后独占取回并精确清理；digest-adopt临时残留检查为空，L从未删除/重建/恢复覆盖。

新增运行器显式loopback bind正例、外网/日常HOME拒绝反例，source冻结前后hash与实际releasebinary摘要绑定，fresh supply固定2.99 descriptor e0464990/官方tgz b920ba02/外层bundle5b10c1a8，不用旧debug2.97资源或已删旧Profile。exact-one-test与双轮专用完成标记共同控制通过，失败本轮exit101/无通过标记保留。2次前置脚本失败在服务前分别遇无ACL返回/宿主Python无xattr接口，按实际文件身份与有界原生系统调用修正，独立目录保留；不是3次真实服务重跑。独立边界预检head-c0参数错误和初编译类型/导入错误保留，最终严格Clippy/fmt及4项release非原生命令回归通过。中间release构建因review改断言未运行，最终三源冻结后专门构建并比摘要；不冒固定.app或panic=abort主程序验证。

证据.adg/evidence/0.2.24/client-root-two-cycles-r1/保留所有失败、脚本/实际资源及binary/source身份、受控Profile74文件raw gzip、first-seen新协调文件原字节、完整manager/stderr阶段日志及独立验证。admin-api-token/runtime-port等敏感运行对象不导出raw；合成配置可保留。逐字归档复核后删除本轮三个运行/前置临时目录与边界预检/重复证据目录，工程target保留，无日常配置/Shell/已安装app/真实WebDAV/模型请求或新固定候选/发布。

下一假设已明确：第二代在当前每个操作检查点重复审查首代归档，Stopped独立登记由首轮约485ms增到约2140ms，restore-plan登记约359ms增到约2271ms，pre-Stop段也显著变长；目前只定位到这些边界，尚未逐syscall/frame计量根因。下一批先精确计数同一观察内重复frame/protection/path opens，选择仍完整覆盖实际变更前后来源/内容/保护及目录事实的有界优化；不使用跨动作摘要cache、跳过旧archive或增加/重置deadline，无新假设不开第二轮。根中断/动作回执容量边界、缺Ready/其它客户端图路径/进程集合和刷新/NAME/复杂代理/Windows W1W2/发行和固定候选等整版范围保持，P24-04/05未关，目标active。


## 2026-11-13 帧入口祖先链合并：局部改善，原包Stop失败保持

main e782afb6b只将帧入口data/profile/client/vault的重叠祖先按路径合并到调用内唯一集合，整组两遍nofollow pathname/fd身份及完整Protection核验，冲突identity/Protection在聚合阶段拒绝；raw父fd由原pins借用持续持有，无新辅助SQLite inode fd。private判定在fresh全部Protection与原pin相等后执行，profile/client准入不放宽。每帧完整root provenance/namespace、hash/pred/schema/root/Protection/Original/tree/Grok检查原样保留；无source cache、跨调用证明或持久格式变化。详[帧入口计量合同](0.2.24-contracts/client-prefix-observation-r1.md)。

先以cfg(test) thread-local metrics测完整四帧闭合：fullproof10/capture64/pinnedopen56/Protection6683。曾实现局部来源guard/首尾fullproof/namespace pins，但其单帧Original路径让相同串行capture从参考2.99–3.05s增至4.35–4.54s，修单帧后仍3.86–4.06s，独占原10s保管也出现失败；不能归因为并发负载。整套来源复用及其专属测试因此撤下，从未commit或跑官方包，保留原实验/失败/引用与最终撤回事实，不将其12%四帧局部数据包装成最终收益。

最终精简实现同binary/同fixture正反顺序reference90.75/89.93ms、ancestorunion87.79/86.30ms，均八帧，fullproof10/capture64/open56完全相同，Protection6683→6299减少5.75%，小样本均值3.65%变化。最终同binary串行capture约3.02s，reference约2.95–3.10s，来源次数同1089，属性略降；不宣称稳定通用提速或正常Stop通过。test标签沿用scoped-union是历史计量命名，最终没有来源复用。生产reference固定false，无env/serde入口；显式diagnostic env仅cfgtest模块。

最终186个唯一受影响函数通过：history1、目录pin31、首代原件9、索引/捕获56（5项ignore未计）、其余rootretirement36、restore44、rootcreation8、显式ignored对照诊断1；新增3个union负控。strict all-target Clippy/fmt/diff通过。冲突pins两序准确错误证明聚合拒绝；首遍已扫共享祖先后mode/xattr/rename/symlink变化在第二遍拒绝；父子/兄弟/重复及中文空格反斜杠路径和跨调用核验保持。旧源/auth/capture/phase/archive/恢复反例均保留。

证据.adg/evidence/0.2.24/client-prefix-observation-r1/固定最终7源/实际debug-librarybinary，在父2e74359b4加这些差异下构建，映射本提交不冒嵌入commit重建。所有中间编译/fixture/strictClippy与并发/独占失败按原日志留存，最终只计union-only日志；一次singleprobe改来源仍慢的证伪结果导致撤回方案而非修改deadline/断言。源码/binary及证据逐字核验后清理本轮TempDir/重复临时目录，工程target保留。没有官方runtime/service/全局协调DB、日常配置/Shell/已安装app/真实WebDAV或模型请求、新候选/发布。

原官方双轮第二Stop12878ms/RestorePrepared16失败继续有效。本批局部收益很小，不足以确认消除第二Stop瓶颈；下一批继续测同一完整历史观察中重复archive/plan/frame-prefix调用，保留独立来源/字节/权限/path及实际变更前后验证，不跳过旧history或cache跨边界，无新假设不开原包轮。原30/5/10及Restart30/Stop子预算、根中断/动作回执容量边界、缺Ready/一般客户端图路径/进程集合和刷新/NAME/复杂代理/Windows W1W2/发行与固定候选等完整范围保持，P24-04/05未关闭，目标active。


## 2026-11-13 历史只读计划认证收敛，前后完整复核保持

main 350bc0309把单次历史open开始的重复Plan认证从六次收敛为两次：私有open_archive_shell仅pins、不返回已认证历史；同一Pending::open_with_plan仍用原完整load得到本调用ownedPlan，完整initial.source比独立Head后消费；历史checkpoint后仍独立load、Plan相等及第二套archive/tree/Grok/intent/readback完整检查。每帧来源/auth/namespace/hash/pred/root/Protection/Original/scope/live、精确releasedtree与completion均不减；公开Pending::open及current/Stop/移动入口仍原完整认证，不用历史shortcut。namespace继承保留，不经序列化/枚举恢复，无跨观察cache。详[计划观察合同](0.2.24-contracts/client-plan-observation-r1.md)。

原六次load前四次之间无写入或外部callback，去掉重复不删独立before/after边界；上层历史前后各open一次仍保持，故该层12→4而不是整index只load2。测试reference只cfgtest TLS/RAII，生产固定false，无env/serde产品入口。保留前后完整认证及持续变化拒绝，不承诺所有不合作并发瞬时ABA都有相同命中时刻。

最终104个唯一受影响函数通过：3新例、完整schema2/fullframe比较1、其它rootretirement36、index56、多代8，5项ignore不计；strict all-target Clippy/fmt/diff通过。新source负控从早期ticket改动加强为保持身份/Original合法的source-scope/source-revision，并精确断言完整source不符；Plan字节/权限/未封存替换在真实readback checkpoint后拒绝，后续调用重新认证，坏digest/id/completion零改写。replacement-unsealed不冒合法同字节400 Plan文件inode连续性要求。

同binary/full4frame夹具reference→current→current→reference约86.89/86.32ms与81.10/82.19ms，plan6→2、八frame/两prefix/十次fullrootprovenance不变、capture64→60、Protection6299→5927，小样本均值5.73%变化；不宣称通用提速、原包normalStop10通过或固定候选验收。多代/进程exit/rootnativegap原反例均保留。证据.adg/evidence/0.2.24/client-plan-observation-r1/固定3源/实际debug-librarybinary，在父e782afb6b加这些差异下构建并映射本提交，不冒嵌入commit已重建；初test旧source反例日志只作原时点，不替代加强后最终覆盖。

本轮无失败测试或新原包轮，未触官方runtime/service/全局协调DB、日常配置/Shell/安装app或业务WebDAV/模型请求；所有test/clippy/fmt/agent句柄终止，unitTMP空、归档字节核验后清本轮临时根/重复目录，工程target保留，无候选/发布。Windows远端仍b2e89665d/0eef59a01无新回交，不要求Mac-only批重跑Windows。

下一项继续核上层双history-open与各自内部双archive-prefix的重复覆盖，尝试封闭的本调用观察对象将原before/after归档认证围住同一vault投影，结束时独立重读全部来源/内容/保护和路径；不学习磁盘来源、不cache跨边界，不直接删除结束复核。当前局部改善尚未证明消除原包第二Stop12878ms/RestorePrepared16，旧失败有效、无充分新假设不重跑。根中断/动作回执容量边界、缺Ready/其它客户端图路径/进程集合与刷新/NAME/复杂代理/Windows W1W2/发行README/固定候选等完整范围保持，P24-04/05未关，目标active。


## 2026-11-13 目录投影前后历史完整双遍观察

main 236ed43dd将vault历史审查原“两次open、各自两套完整读取”的四套遍历合并为同一投影前后两套。全部generation begin/各代scope→exact vault/current-attempt投影→全部finish/最终scope→currentdir/names/private/manager→外层独立index/Head/events/binding/slots，顺序不变，任一失败整体拒绝。私有不可Clone/serde的HistoricalObservation只存独立ClosedRefs/内存namespace、路径、authPlan/tree摘要及小intent/receipt，不保留Originals/fd/Plan/tree；begin callback仅借用三个scope目录且返回unit，finish消费对象独立重开/完整load/source比对、完整archive/每帧来源/content/root/Protection/Original/tree/Grok/scope/live/intent/readback，最后才返回最终Originals。公开单历史入口仍原完整两遍，current/Stop/移动路径不用新shortcut。详[投影观察合同](0.2.24-contracts/client-history-projection-r1.md)。

初设计的inspect(Originals)会通过其它owned-return方法逸出未完成原件，且逐代保留完整祖先fd会累计；实际实现未采纳该设计。capsule在begin内释放全部句柄/大对象，只保留先前已独立认证的相等摘要，finish不从摘要或磁盘学习来源，SHA只是完整freshauth后的相等比较。32capsules的FD检查由真实独立--exact/单线程子进程完成，初始/最终70，原件零写；避免仓库默认并发干扰测试证明，不新增较低代数限制。

107个唯一受影响函数通过：3新投影/资源例、fullschema2四帧比较1、其它rootretirement39、index56、多代8，5项ignore不计；strict all-target Clippy/fmt/diff通过。投影后Plan/receipt/frame/archive权限/namespace污染拒绝且不修复，scopecallback错误不返对象。多代/真实进程exit/来源/正常恢复间隙原反例保持。初控制日志的非隔离FD计数仅为原时点，最终隔离子进程日志独立保留。

同binary/同独立索引的四帧夹具reference→current→current→reference约180.66/173.85ms与90.56/91.64ms，wholepass4→2、frames16→8、fullrootprovenance20→10、plan4→2，保留一套完整before及一套独立after，小样本均值48.61%变化不直接认定原包Stop通过。每次索引观察创建新capsule，跨调用/动作不cache；不宣称所有generation原子快照或所有外部瞬态ABA可检测，前后持续变化拒绝边界保持。

证据.adg/evidence/0.2.24/client-history-projection-r1/固定4源/实际debug-librarybinary，在父350bc0309加这些差异下构建并映射本提交，不冒嵌入commit已重建。所有unit/clippy/fmt/agent句柄终止，unitTMP空、逐字归档后清本轮unit临时根/重复目录，工程target保留。该suite没有官方服务/全局协调DB、日常配置/Shell/已安装app或业务WebDAV/模型请求，无固定候选/发布。

已按新假设准备独立原包轮：冻结本提交clean tree与15个关键源，原Cargo release opt-level=s/LTO no-run构建完成后绑定实际binary，用全新Profile原30/5/10、相同沙箱/精确coordinator清理，验证两个完整正常循环。当前只在构建，尚无新native结论，前轮第二Stop12878ms/RestorePrepared16仍为有效失败；新结果另按原始证据回写，不改旧失败。根中断/动作回执容量边界、缺Ready/一般客户端图路径/进程集合和刷新/NAME/复杂代理/Windows W1W2/发行README与固定候选等完整目标保持，P24-04/05未关，目标active，发布另依有效授权。


## 2026-11-13 投影双遍优化后新原包轮：第二Stop仍失败，旧轮保留

冻结main236ed43dd clean tree及15关键源，在原Cargo release opt-level=s/LTO构建实际librarybinary SHA86d4036f3314457f2eb555e91d4787c8a71144af5c72b737e57c0fddd9ee4b6e。以“重复归档遍历减半应让第二normalStop回到原10秒”新假设启动唯一新native轮，原source/基线/沙箱/准确资源摘要/两cycle全部断言不减。前轮client-root-two-cycles-r1原失败证据不改，新证据独立[原包复验合同](0.2.24-contracts/client-history-projected-native-r1.md)。

首轮Start8202ms、同连接验签Status227ms、完整normalStop8749ms、stoppedStatus1375ms/NotRequired/canStart/Restored均通过。第二轮Start14368ms（前轮20700）、验签Status1453ms（前轮2205）通过；normalStop外层10736ms（前轮12878），产品内部10043ms，返回client_original_restore_pending、Stopped/无Stopped result/clientRecoveryRequired/canStartfalse，仍失败。内部恢复在root-artifacts-held处client_session_unconfirmed：该阶段累计5088ms、结束5375ms，尚未建root-retirement movement；独立Head仍RestorePrepared16，无completion，根留在原逻辑位置。不能把物理服务退出、局部耗时改善或已至Stop误判为通过。

两轮原始launcher/service实际normalexit0，每轮原生4listeners全消失；native runner的PID+birth终态与ownedexecutables均无残留、无observer错误/coordinator替换。旧binding/rawfirst9events/wholearchive在第二Start及Stop后保持，两代provenance/Original/scope/Ready/PreStop/Stopped完整摘要链从74文件raw gzip离线重验；首代原Plan/readback完备，第二代未伪造close。两个primary端口在清理前用内核TCP表独立确认不监听。全局预存L在最终rwexclusive连接后identity/owner/mode/flags/ACL/xattrs/bytes/SHA和sidecar缺席均原样，L从未删除/重建/恢复覆盖；3个准确新N/K/history先持久rawreceipt再独占取回和identity匹配清理，无digest-adopt残留。

证据.adg/evidence/0.2.24/client-history-projected-native-r1/保存运行脚本、冻结source/tree15hash、实际binary/完整release构建/4项非原生release命令回归、沙箱loopback正例/外网和日常HOME拒绝、完整stdout/stderr/manager/summary/rawprofile与coordinator原字节、失败核验和清理。只有这一实际新轮，无前置脚本失败或强杀/原轮重跑；官方tgzb920ba02/descriptore0464990/outerbundle5b10c1a8和整树供应保持，不改官方2.99，无日常配置/Shell/安装app或业务WebDAV/模型请求。归档逐字复核后移除本轮ownedProfile/程序包/复制testbinary及重复临时证据目录，工程target保留；无固定候选/发布。新单元与新原生证据分开，未将debug计量当release验收。

新假设被证伪一部分：重复历史优化确实改善Start和Stop，但没有使第二normalStop达标。下一步在现有Stop30/5/10原边界精确测Active::load/preStop/独立phase和restore-artifact admission，区分root-artifacts-held处deadline已耗尽与具体来源/图像/Guard检查拒绝；不能只凭client_session_unconfirmed认定是超时。已观测第二indexStopped累积2358ms、indexPlan2756、rootAdmission4283、rootArtifactsHeld5088，停止前load/preStop也较首轮增长。评估仅在原持锁事务同一次观察内围绕已闭合history保留完整前后验证、减少每层重复；不得skip当前root/provenance/Plan/frame/content/path/Protection/OSproof，不cache跨动作、不加预算或重置Stop。无新定位/假设不开下一native轮。根中断/动作回执容量边界、缺Ready/一般客户端范围/进程集合与刷新/NAME/复杂代理/Windows W1W2/发行README及固定候选等完整目标保持，P24-04/05未关，目标active。


## 2026-11-13 Ready取得的连续重复历史去重与Stop拒绝诊断

main 4163514fe新增封闭Index::open_ready_session供实际Active::load：私有open_inner_observed保留setup transaction及COMMIT后完整核验，直接消费后一次Head/Originals进入原scope/live/program/帧验证，去掉连续构造里再index.head和registered_root_originals两份重复fullhistory。返回前核profile/HOME/fullrefs/storage/rawHead；无公开中间Originals/token/callback或跨动作cache。原open/open_existing语义及公开open_indexed的非Ready范围保持，普通canonicalclient仍原逻辑，currentReady/PreStop/typedphase/后续事务与Guard不减。详[Ready取得合同](0.2.24-contracts/client-ready-acquisition-r1.md)。

ReadyAcquisitionError显式Index(io)与Session(AppError)，Active只对Index用原failure，原scope/live错误码透传。新reopen注入点仅origin-none，首根原3交接点/锁断言保持。cfgtest trace记录固定stage/区间微秒/同原deadline剩余预算/expired/code，覆盖入口load/preStop、admission、root-artifact与IO映射前permit/writer原因；不输出detail/path/config/secret，生产无日志，调用顺序/短路/结果/原IO种类文案不变，不增加或重置预算。

95个唯一受影响函数通过：Ready4、其它Session29、index56、命令4、step结果1、writer映射1，8ignore不计；最终strict all-target Clippy/fmt/diff通过。首代/第二代synthetic Ready source构造相同Head/refs，第二代historicalbegin/finish4→2、frame50→26、Protection77774→40442，不能当作实际Ready或Stop耗时保证。COMMIT后oldOriginal/receipt字节400权限、外部进程SQLitebinding/event、namespace变更被末次audit拒绝；假Live/PID不获currentReady，load后帧变化仍拒绝。source/live/program/nonReady拒绝保持。

初编译因Refs无Debug的assert_eq失败，改普通相等断言而非增加Debug；index原套件55pass/1failed来自新增testhook撞首根精确3阶段allowlist，修hook作用域后原测试不变精确通过，Readypostcommit负控也重测。core六源binary主要检查与最终七源binary仅新增writertrace等价/复验分别留证，未把旧日志冒一份最终binary全量。审查发现字节反例权限掩盖已修为写后400。证据.adg/evidence/0.2.24/client-ready-acquisition-r1/绑定两份源/binary及显式delta、假设、失败、复验和清理，归档逐字比对后清本轮自有TempDir/重复目录，工程target保留。无官方服务/全局协调DB、日常配置/Shell/安装app或业务WebDAV/模型请求。

前轮原包第二Stop10736ms/RestorePrepared16仍失败，本批未跑原包，不能宣布预算修复。下一步冻结release来源，用新阶段/remaining/code精确区分root-artifacts-held拒绝与deadline，验证入口load/preStop重复审查去除后的真实效应；保持原30/5/10及Restart30/Stop子预算、原件/归属/未知拒绝，旧失败原目录不覆盖。根中断/动作回执容量、缺Ready/其它客户端图路径/完整进程集合与刷新/NAME/复杂代理/Windows W1W2/发行README与固定候选全范围保持，P24-04/05未关，目标active，发布另依授权。


## 2026-11-13 Ready取得优化原包诊断：确认恢复原deadline耗尽

冻结main4163514fe clean tree/18关键源，原release opt-level=s/LTO宿主binary SHA43453c81da3d2ab09e48f5326d303f61b71a1893fc75ecd54601baf32077babd，实际原官方2.99/descriptor/outerbundle固定摘要不变。唯一新双轮诊断：首轮Start8839ms、同连接验签Status226ms、完整normalStop8608ms、stoppedStatus1381ms均通过；第二轮Start14110ms、Status1528ms通过，Stop10729ms仍失败，Stopped/无Stoppedresult/clientRecoveryRequired/canStartfalse/独立RestorePrepared16，无root movement/completion。旧10736/12878ms两轮失败证据均不覆盖。详[本轮原因诊断](0.2.24-contracts/client-ready-native-r1.md)。

新trace闭合了本轮根因：第二cycle loadReady636537us、preStop2092753us成功；进入artifactadmission仍908879us，Guardimage/Planopen/初authority全部在deadline内通过；applyretaining开始余696169us、实际耗735432us。最后一次authority.live开始只余1633us、耗40838us，结束remaining0/expiredtrue，紧随原pair/authority检查返回底层lifecycle_timeout；原PermissionDenied映射再包装为client_session_unconfirmed和动作client_original_restore_pending，未出现writer-refusal trace。因此本轮已确认为原10秒预算耗尽，不是根据总时长猜测来源/Guard损坏；该因果不冒自动追认旧无trace轮次。

两轮原launcher/service实际normalexit0、每轮4listeners全消失；所有trackedPID+birth及ownedexecutables终态无残留，coordinatoridentity稳定、无observationerror。oldbinding/rawfirst9events/旧archive在secondStart及Stop后均原样；两代provenance/Original/scope/Ready/PreStop/Stopped摘要链和首代Plan/readback从75文件rawgzip独立验证，第二代无伪造RootAbsent。局部artifact恢复可能已进入分步归档，不能从RestorePrepared16断言没发生任何artifact变化。原件/半成品均保留证据后仅清理owned fixture。

新增诊断不仅区分表面会话错误，原stage/code/remaining证明deadlinecrossing；cfgtest日志不输出errorDetail/path/config/secret。已有生产判定/原30/5/10及Restart30/Stop子预算未改，不用强制退出替代normalStop证据。4项同release非native命令回归通过，不把ignored目标当第二次通过；runner还要求实际one-test与完成marker，当前exit101/无passmarker保持整轮acceptedfalse。

证据.adg/evidence/0.2.24/client-ready-native-r1/保留source/tree18hash/实际binary、完整build、loopbackbind正例/外网和日常HOME拒绝、完整stdout/stderr/manager及独立trace核验、rawProfile75文件与新coordinator原字节。全局预存L在最终rwexclusive连接关闭后identity/uid/gid/mode/flags/ACL/xattrs/bytes/SHA及sidecar缺席不变，从未删除/重建/覆盖；准确3新N/K/history在持久rawreceipt/firstseenidentity核对后exclusive取回并清理，digest-adopt残留为空。两个导出primaryports经内核TCP表独立不监听。逐字归档核验后清本轮Profile/程序包/二进制副本和重复临时证据目录，工程target保留；没有日常配置/Shell/安装app/真实业务WebDAV/模型请求或官方2.99修改，无固定候选/发布。

下一步不改渠道或放宽拒绝，针对已确认耗时测量每帧rootprovenance连续3记录+1reference读取重复校验共享parentpin的成本；评估仅调用内封闭批量文件读取，在全组前后完整核parent路径链，每个文件仍完整nofollow/identity/version/type/singlelink/owner/ACL/xattr/bytes/hash/strictJSON及原交叉绑定，不cache跨帧/动作、不减少每帧调用或最终namespace。必须用单/多帧及认证期间parent/file变更反例证伪，若不能减少实际计数且无capture退化就不合入。旧失败保留，无新修复/假设不开原包轮。根中断/动作回执容量边界、缺Ready/一般客户端图路径/完整进程集合与刷新/NAME/复杂代理/Windows W1W2/发行README及固定候选全范围保持，P24-04/05未关，目标active。


## 2026-11-13 子目录pin建立去重，文件认证边界保持

main 6ca915392在child_private保留parent.private前验/mkdir/EEXIST，clone既有父DIRECTORYfd及identity/Protection期望，只open_at精确leaf，再完整parent+child union双遍、pureleafprivate，最终原parent.recheck仍位于childsecurity读取之后。复用的是目录handle，不是文件认证或Ready/Stop证明；无辅助SQLiteinodefd。普通capture_file主体与root_creation整文件相对父4163514fe逐字相同，每帧来源/内容/语义/namespace均原样。fixed4file来源分组草案因减少原中间parent采样拒绝边界而撤下，未编译/commit/native，不将其能力计入本批。详[子目录合同](0.2.24-contracts/client-child-pin-r1.md)。

145唯一函数通过：4新child控、fullhistory1、其它pin31、restore45、index56、rollover8，5ignore不计；strictClippy/fmt/diff通过。新FDflags断言先>=0再CLOEXEC；parentdrop后child完整pin有效，acquisition前后parent/child更换/权限/xattr/链接与原最后parenttail变更拒绝，既有unsafefile/link/755不修补。原syscall类型/内容hash/私有/位次/真实exit/恢复负控保持。

同binary同heldindex四帧ref87.98/89.70ms与current83.46/84.36ms，directoryopen68→30、Protection6430→6109，frame8/fullsource10/capture60/plan2不变，小样本均值5.55%仅局部debug观测且含计量/assert开销；不冒通用提速或Stop10通过，不承诺任意外部瞬态ABA采样时点完全相同。证据.adg/evidence/0.2.24/client-child-pin-r1/固定4源/actualdebugbinary，在父4163514fe差异构建并映射本提交；初3例与最终4例边界分开留证。所有test/clippy/fmt/agent句柄终止，归档逐字核验后清本轮临时根/duplicate证据，工程target保留，无官方服务/全局DB/日常配置/Shell/安装app/业务WebDAV模型或候选/发布。

原ready-native-r1第二Stop10729ms/deadline耗尽结论有效。本批小幅收益未验证原包，不盲重跑；继续针对恢复链重复路径/原件/authority校验测量，保持原30/5/10/Restart30及Stop子预算、逐帧完整认证和未知拒绝。根中断/动作回执容量边界、缺Ready/一般客户端图路径/进程集合和刷新/NAME/复杂代理/WindowsW1W2/发行README及固定候选等完整目标保持，P24-04/05未关，目标active。


## 2026-11-13 索引scope消费同次完整认证帧链

main 0f0e85fb3在root_binding.verify_capture使用原open_profile_origin_prefix返回的同次(originals,frames)立即传原verify_index_scope_prefix，避免原opener丢弃完整帧链后又由verify_closed_reference_files重读Original+Ready。frames与Originals来自同一refs/OriginalKeys的独立hash，已验证全部knownphase/source/provenance/root/保护/前驱/内容/archive和namespace，未改refs/known、不跨写或callback、不存跨调用状态；不接受外来任意frames，新增的是caller同源约束内的消费。scope/HOME/live/ReadySHA/process/parent字段校验逐项与原完整路径一致，当前program/签名Ready仍原独立职责。详[scope帧链合同](0.2.24-contracts/client-index-prefix-r1.md)。

98唯一函数通过：prefix筛选8（2新增+6既有根状态）、其它Session34、Index56；7ignore不计，最终语义加强函数去重；strictClippy/fmt/diff通过。同binary实际Index.open_existing ref→current→current→ref，首代frame8→4/约201.85–211.43ms→137.21–143.65ms；第二代frame24→20/413.41–418.73ms→361.67–369.39ms，fullhistorybegin/finish不变。debug合成局部收益不认normalStop通过。TLSreference仅cfgtest无产品env/serde入口，生产走完整同次prefixcaller；不声称去第二读取后所有外部瞬态采样时刻等同。

HOME/client/Readyhash/generation语义反例使用正确新文件SHA与400，scope变更同步live.scopeSHA/livefileSHA；拒绝后只改回目标字段并更新必要依赖必须成功，避免其它过期引用假绿。初HOME/client反例的旧live引用由review指出并修，早期通过日志不冒加强后证明；productfiles两个testbuild间逐字同源，仅测试代码加强，最终semantic例和clippy/fmt单独重跑。证据.adg/evidence/0.2.24/client-index-prefix-r1/固定3源、两actualbinary/source身份、counter/对照/显式testdelta与清理；归档核验后清本轮TempDir与重复证据根，工程target留存，无官方服务/全局协调DB/日常配置/Shell/安装app/业务WebDAV模型/固定候选或发布。

原ready-native-r1第二Stop10729ms/原deadline耗尽仍失败。本批合并同次帧证据未跑新原包，不能改判；下一步累计已测去重后冻结release源，按明确新假设验证两个完整原包循环/原30/5/10，若仍超时继续按trace修而非重置预算或省proof。根中断/动作回执容量边界、缺Ready/一般客户端图路径/完整进程集合与刷新/NAME/复杂代理/WindowsW1W2/发行README及固定候选等完整目标不缩减，P24-04/05未关，目标active。


## 2026-11-13 子目录/同次prefix累计优化原包轮：原件恢复成功，根计划登记后预算失败

main0f0e85fb3 clean tree/18关键源冻结，原releaseopt-level=s/LTO宿主binary SHA7174d3761349872aeec12585d7448067c69694627080ebc7f31be125e0fafbb3，原官方2.99/descriptor/outerbundle摘要不变。以“childpin减少重复路径打开＋currentRootScope复用同次prefix可释放预算”唯一新实际双轮，首轮Start8305ms/签名Status236ms/normalStop8285ms通过、stoppedStatus1351ms及preview/可再启动通过；第二Start13465ms/签名Status1220ms通过，normalStop12143ms失败（内部11443ms），Stopped/无Stoppedresult/clientRecoveryRequired/canStartfalse，独立RootRetirementPrepared17，无completion。所有旧10729/10736/12878ms失败目录保持，不用更晚的失败位置替代Stop成功。详[累计优化原包轮](0.2.24-contracts/client-prefix-native-r1.md)。

trace确认证据进展：secondloadReady515775us/preStop1861995us均成功；artifactadmission入口余1778855us，apply起始余1600561us耗1081034us且结束余519527us，postadmission/finaltime成功后仍余453634us。随后rootplan准备/独立登记/registration耗尽剩余预算，在root-retire入口admission.entry-time即remaining0/timeout；本轮无authority/writer-refusaltrace。恢复累计rootArtifactsHeld4606ms、rootPlan5931ms、rootRetire7818ms，下一需求是完整根移动/关闭预算仍不足，不是来源损坏或渠道写入失败。RootRetirementPrepared17只表示计划登记，root仍逻辑位置，没有Retire-readback/RootAbsent/completion。

独立raw77Profilefile核验17事件/Head、两代provenance/schema5Original/scope/ReadyPreStopStopped链、首代Plan/readback、第二rootretirePlanSHA/source16/schema2；firstbinding/raw9events/完整旧archive在secondStart/Stop保持。两轮原launcher/service真实normalexit0/各4listeners消失，但这些不替代normalStop。全trackedPIDbirth与ownedexecutable终态无残留，无observererrors/coordinator替换；globalexistingL最终rwexclusive关闭后身份/owner/mode/flags/ACL/xattrs/bytes/SHA及sidecars原样且释放，永不删重建覆盖；exact3新N/K/history先持久rawreceipt/firstseenidentity/exclusive检查再清理，digest-adopt残留空，两个primaryports内核TCP独立无监听。原件分步restore已完成并有新计划，但不自动声称所有后续恢复可用。

证据.adg/evidence/0.2.24/client-prefix-native-r1/保留clean source/tree18hash/实际binary/完整releasebuild/4项release非native回归、sandboxloopbackbind正例/外网与日常HOME拒绝、stdout/stderr/manager/summary、两个framechains及新计划rawbytes、进程/coordinator/TCP清理证据，原始失败result不变。只一新native轮，无强杀/重跑/加预算/改官方包；所有临时Profile/program/archivecopy/testbinary逐字归档核验后清理，工程target保留。无日常配置/Shell/安装app/真实业务WebDAV模型或固定候选/发布。只读agent同时盘点后继root中断与成功StopcanStart容量不一致，结论记录remaining-root-scope-audit.json，不是完成声明或新实现。

本假设部分有效：Start/Status和artifact阶段更快，但全Stop仍失败且累计耗时更长，不能以百分比或root进度夸大性能成果。下一步计量record_root_retirement/registration中已验证Original/Plan与beforeSQL/candidate/COMMITreadback/registration之间重复读取，设计封闭事务返回准确registration及已认证facts，保留所有实际边界的完整history/currentroot/provenance/frame/Plan/path/Protection/nativeproof，禁止跨动作cache/skip旧history或重置deadline。须真实反例/计量再开下一原包轮，原30/5/10及Restart30/Stop子预算保持。root中断专用authority/RootRetirementPrepared移动前后继续恢复/RootAbsent重close/成功Stopaction容量门、缺Ready/一般客户端图路径/进程集合和刷新/NAME/代理/WindowsW1W2/发行README与固定候选完整范围保持，P24-04/05未关，目标active。


## 2026-11-13 根退役登记复用同事务来源认证

main f10e732aa 将原事务外计划准入移入已有 SQL 事务的完整 source 审计之后、首次 SQL 之前；保留候选与提交后 full audit，随后直接返回当前登记凭据，实际 retire 入口 full require 不变。四个独立同形合成二代 fixture 对照旧／新／新／旧，完整历史观察 5→3、双遍审计 10→6，约 592→370ms。25 项不同受影响测试及严格工程检查通过；零项筛选尝试单独保留并以实际命令模块补验，不算通过。

原包第二 Stop12143ms／RootRetirementPrepared17 的失败不变。本批没有原包新轮或新候选；先定位根计划／退役剩余成本、形成覆盖缺口的新假设，才进行受影响真实轮。事务边界、反例、独立只读审阅、源／测试二进制摘要、证据逐字核验和测试临时根清理见 [client-retirement-transaction-r1](0.2.24-contracts/client-retirement-transaction-r1.md) 及 .adg/evidence/0.2.24/client-retirement-transaction-r1/。根恢复与缺 Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／Windows W1W2／交付验证与固定候选等本版全范围保持，目标 active，未发布。


## 2026-11-13 连续只读私有目录取得：完整模块回归通过

main acd9b871e 合并三处连续已有私有目录的取得核验，逐级 nofollow／完整 private 及最终双遍全链／直接 parent 尾部检查保持，不创建、不读取内容或输出中间 pin。原件模块单轮 290 项通过／9 原有忽略，版本化命令另 4 过／1 原有忽略，严格工程检查通过。初轮夹具尾斜杠导致注入未执行的失败单独保留，修夹具后原断言通过。

完整二代退役登记对照的属性读取 31030→29423，完整历史观察 3 次／内容双遍 6 遍及 frame、capture、plan、provenance 次数均保持，约 5% 收益；不以纯目录取得 156→56 的局部降幅代替。取得期间短暂变化的并发拒绝轨迹不保证与删去的中途扫描完全一致，最终期望和内容读取边界保持，实际 diff 已经只读审查。源／二进制摘要、全部压缩原日志、独立计量核验、清理及限制见 [client-descendant-pin-r1](0.2.24-contracts/client-descendant-pin-r1.md) 和 .adg/evidence/0.2.24/client-descendant-pin-r1/。

本批无新原包轮，原第二 Stop12143ms 失败不变，下一步核查 record_restore_plan 的事务外完整来源与计划准入能否消费同事务完整 Originals，先测量再形成新原包假设。Windows 精确远端 refs 仍为代码 b2e89665d／文档 0eef59a01，无新推送，不冒称 W1 已交回。根中断／缺 Ready／一般客户端图路径与完整进程集合／刷新／NAME／复杂代理／Windows W1W2／交付验证及固定候选等完整范围不缩减，目标 active，未发布。


## 2026-11-13 恢复计划登记并入同事务来源审计

main c3f2fc97d 将 root-aware record_restore_plan 的计划完整准入移入现有 write_checked_admitted，消费同次事务内完整来源认证的 Originals，仍先核 exact Head、后准入、再首次 SQL。候选／提交后的完整来源与索引读回、计划全部帧／引用校验、普通及 recovery-source 分支保持。首次／二代来源、SQL 前错误及九组边界篡改、登记后计划改动拒绝重新打开等新增 4 项通过，索引 28 项和命令 4 项回归通过，严格工程检查通过；编译期 Debug 断言错误的原轮保留。

四个独立同形二代夹具中完整历史观察 4→3，双遍内容审计 8→6，旧／新／新／旧约 592／465／463／590ms。登记是引用发布，不构成实际应用／正常停止成功；计划后续仍在应用前完整重新打开。来源／测试二进制摘要、独立验证与临时根清理见 [client-restore-registration-r1](0.2.24-contracts/client-restore-registration-r1.md) 及 .adg/evidence/0.2.24/client-restore-registration-r1/。

本批无原包新轮，原第二 Stop12143ms 失败保持。下一步有界审查完整只读帧前缀内重复 root provenance／namespace 观察，先证明返回与检查边界，再决定实施；不先删校验或重跑失败旧源。根中断／缺 Ready／一般客户端图路径、完整进程集合与刷新／NAME／复杂代理／Windows W1W2／交付验证与固定候选等完整范围不缩减，目标 active，未发布。


## 2026-11-13 保留逐帧来源门槛，合并其前置元数据观察

main 098ae4a15 未采用首末帧替代完整逐帧 provenance：来源内容／权限或 namespace 在中途改变又恢复，仍必须 Ready archive 前拒绝。新增三类瞬态反例与 archive 进入计数，要求仅 Original 已完成、后续恢复回调不可达。实际只将每帧来源校验之前的 profile/data/client 三组重叠目录元数据核验合为同政策鲜读＋双遍完整 union，之后才进入任何 namespace／来源记录内容。来源四文件读取、hash/private/动态绑定和逐帧／末尾 namespace 保持；代码补审无阻断项，不把它当测试证据。

58 项不同受影响测试及严格工程检查通过，五源文件／测试二进制摘要、旧方案否决、全部压缩原日志和 unit 临时根逐字归档清理见 [client-root-metadata-r1](0.2.24-contracts/client-root-metadata-r1.md) 与 .adg/evidence/0.2.24/client-root-metadata-r1/。同形二代恢复登记中动态 provenance38／namespace37／frame/archive26／完整历史3与双遍6不变，属性读取40322→39334，约2.5%；时间样本重叠，不宣称端到端明显提速。

当前以干净 main 098ae4a15 和21个关键源摘要开始 release 库测试宿主构建，后继累计改动原包双轮诊断须独立保留完整状态和原30/5/10预算，未因已开始构建或单轮通过而放行。原第二Stop12143ms失败保留，本批unit证据无新官方服务轮次。根中断／缺Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／Windows W1W2／交付验证及固定候选等完整范围保持，目标active，未发布。


## 2026-11-13 累计事务／目录／元数据优化原包轮：第二Stop仍超时

干净main098ae4a15、21关键源摘要和实际release库测试二进制1076a7ac…已核对，仅一轮新原包诊断。第一轮Start8405／签名Status233／Stop8638ms通过；第二轮Start13253／签名Status1164通过，完整Stop10888ms失败，产品内部10202ms，独立Head为RootRetirementPrepared17。原件恢复后仍余1024696us，rootplan/index/registration耗尽余量，retire入口拒绝；根未移，无Retire-readback/RootAbsent，不用服务正常退出或较旧轮更快代替成功。

两代原件与三phase完整链、首代binding/raw9events/vault保持，77原文件离线重验；两轮原launcher/service正常exit0、每轮4listeners缺席。三个精确新协调对象留原字节并核身份/exclusive后移除，globalL身份／保护／原字节保持并释放；PID+birth／exe与导出主端口独立缺席，逐字归档后清理自有Profile／程序／exe及临时证据根，工程target与旧失败保留。详见 [client-cumulative-native-r1](0.2.24-contracts/client-cumulative-native-r1.md) 和 .adg/evidence/0.2.24/client-cumulative-native-r1/。

下一步对真实root计划登记及retire/close完整分段测量，保留source/candidate/postcommit、逐帧archive前来源、原生/namespace/原件和10s预算；新依据前不重跑同源失败。根中断／缺Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证和固定候选等原完整范围保持，目标active，未发布。


## 2026-11-13 根退役登记分段：主耗时是完整文件认证，不是SQL

main264934c09仅增加测试计量与隔离原包诊断入口，保留原登记顺序／错误／预算。完整Stopped三phase合成第二代总538.924ms，来源123.497／准入56.493／候选175.556／提交后176.466ms，SQL+COMMIT仅0.789ms；互斥顶层覆盖99.9%以上，嵌套live-prefix157.157与tree/grok3.038ms不重复相加。三次完整历史观察与内容双遍6保持，少帧Captured先行结果不替代最终来源，合成表也不冒称真实运行现场。

9项不同受影响测试及严格工程检查通过，全部日志／五源文件与测试二进制摘要、证据逐字归档和临时根清理见 [client-retirement-cost-r1](0.2.24-contracts/client-retirement-cost-r1.md) 与.adg/evidence/0.2.24/client-retirement-cost-r1/。后继仅cfg(test)、限定owned临时Profile的ROOT_REGISTRATION_COST入口已具备，尚未执行新原包轮；应直接测真实candidate/postcommit/prefix成本证伪判断，不从合成表盲删校验。当前root_binding movement前后完整prefix重复是否能安全收敛仍待审，三事务边界与逐帧来源门槛保持。

原第二Stop10888ms失败不变。根中断／缺Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证与固定候选等原完整范围保持，目标active，未发布。


## 2026-11-13 原包真实登记分段与历史夹具校准

干净main264934c09、21源摘要及release库测试二进制9a5eb08a…核对，仅一轮新诊断。第一轮Start8129／Status226／Stop7604ms通过，第二轮Start12812／Status1231通过、Stop11681ms失败，内部10996ms，RootRetirementPrepared17根未移动。原件恢复后只余288171us，root登记耗尽预算，不重跑或提高10s。

实际第二代登记1053.745ms：来源278.369／准入84.635／候选338.963／提交后345.650ms，SQL+COMMIT仅0.532ms。互斥顶层覆盖99.9%以上，嵌套Protection868.906／liveprefix236.018／历史双遍495.298ms交叉，不能加总；三完整历史观察与内容双遍6保持。原合成Stopped夹具只有30帧，而实际48：首代历史合成从Captured退役仅Original，实际四phase，6遍×3差帧=18。下一步先校准完整四帧历史夹具，再评估不减检查的属性小值有界直接鲜读／ERANGE回退等底层成本；不能继续拿轻夹具预测原包达标。逐帧来源门槛和所有事务边界保持。

原77文件及两代摘要链、首代binding/raw9events/vault保持和第二Plan离线验证；两轮正常exit0/各4listeners缺席，精确新3协调对象留字节并identity/exclusive后清理，globalL原身份／保护／bytes保持释放；测试Profile/程序/exe和临时证据根逐字归档后清理。详见 [client-partition-native-r1](0.2.24-contracts/client-partition-native-r1.md) 与.adg/evidence/0.2.24/client-partition-native-r1/。停止动作canStart容量一致性及WindowsW1W2／根中断／缺Ready／一般图路径与完整进程集合／刷新／NAME／复杂代理／官方交付验证和固定候选等原完整范围保持，目标active，未发布。


## 2026-11-13 完整属性有界鲜读与四帧历史校准

main4ba715120实现普通小xattr完整直接鲜读，ERANGE／filledbuffer有界query+exact回退，ResourceFork始终保留原sized路径。初始依赖ERANGE方案被只读审查指出HFS/AppleDouble ResourceFork可成功返回前缀的P1，未提交该方案；修订后native全文／只改尾部／超限反例通过，最终补审无剩余阻断。错误／sizechange／aggregateMAX_SECURITY拒绝，ACL和所有Protection/source/frame/archive/namespace与三事务边界保持，无跨调用缓存或忽略属性。

完整首代四帧历史夹具9events关闭，第二代Stopped15／restore16／retirement17；48frame/archive、source61／namespace60／Protection61744／capture339／历史3/3/6各次数不变。native xattr calls187405→103631，旧／新／新／旧完整登记约767／494／493／775ms，约36%合成局部收益，不当原包预算证明。原轻Captured夹具和旧失败不回填。

原件305／协调27／命令4共336项不同受影响测试及严格工程检查通过，原有忽略保持。全部日志／P1与补审／四源和测试二进制摘要／unit临时根逐字归档清理见 [client-xattr-read-r1](0.2.24-contracts/client-xattr-read-r1.md) 和.adg/evidence/0.2.24/client-xattr-read-r1/。当前固定本提交重建release库宿主，后继唯一新原包双轮必须实测原30/5/10、root移动关闭和清理，未因开始构建或合成改善放行。根中断／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／交付验证及固定候选完整范围保持，目标active，未发布。


## 2026-11-13 属性鲜读原包轮：双根完成，协调收尾仍超总预算

干净main4ba715120、22关键源和release库测试二进制1313d1ec…核对，唯一新原包轮。第一轮Start8038／Status229／Stop6366ms通过；第二轮Start10241／Status819通过，完整Stop11345ms失败。真实登记657.561ms（旧诊断1053.745），帧48／archive48／历史3/3/6保持；第二根已移动及Retire-readback完成，独立RootAbsent18、closedGenerationCount2，79原文件离线核全部链和18events／旧binding/raw9events/vault保持。

失败位置后移并不放行：内部Stop10577ms，release-coordination标记下client_native_coordination_timeout。同源with_admission_mode归因是锁释放／完整程序来源重核后的最终deadline检查，不能说是锁未释放。动作仍失败、canStartfalse／recoveryRequiredtrue；第二只读状态／预览按其事实通过，保留真实磁盘闭合，不能抹掉它或把超时动作晋升成功。下一步分段协调lease末验／正常release／完整program读取与总余量，不多借时间、跳过程序hash或强行通过。

两轮真实normalexit0及每轮4listeners缺席；精确新3协调对象留原字节后核identity/exclusive清理，globalL原身份／保护／bytes保持释放；PID+birth／exe和主端口独立缺席。79原文件及全部材料逐字归档后清理Profile／程序／exe和临时证据根，工程target与旧失败保留。证据及严格失败verifier见 [client-xattr-native-r1](0.2.24-contracts/client-xattr-native-r1.md) 和.adg/evidence/0.2.24/client-xattr-native-r1/。

Windows远端仍b2e89665d／0eef59a01无新推送；根中断与动作资格／缺Ready／一般图路径／完整进程集合和刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原完整范围不缩减，目标active，未发布。


## 2026-11-13 完整镜像相邻路径查询去重与收尾分段

main3701ee391合并完整inventory必需现存路径中紧邻的存在性metadata查询，保留每个组件inspect、collect／worker读前／final原位置、全部内容hash/singlelink/nofollow/预算/线程join与目录前后身份mtime；不改read_complete或sharedbackup safety。55项不同受影响回归及严格工程检查通过，actual diff只读补审无阻断。少一次相邻采样不保证瞬态调度完全相同，native查询1+2d→1+d仅静态推导，inspect数不变。

官方2.99完整image只读ABBA一轮通过，13393files／15646entries／17901pathboundaries及全部rows和树摘要不变，旧／新／新／旧645／522／498／631ms约20%局部改善，无服务／协调对象。副本和unit临时根在结果逐字核对后清理。后继main0c988fc89仅加cfgtest+ownedProfile协调tail计量：锁末验／release／完整来源重读与deadline前后余量；image-only二进制与后继diagnostic源关联分别保存，不冒称旧binary含新代码。证据见 [image-path-admission-r1](0.2.24-contracts/image-path-admission-r1.md) 及.adg/evidence/0.2.24/image-path-admission-r1/。

本批无新原包服务轮，原第二Stop11345ms／RootAbsent18最终协调deadline失败保持，下一步固定release候选原包双轮+tail段判断实际收益；不能仅局部变快就收口。根中断／动作资格／缺Ready／一般图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／交付验证及固定候选完整范围不缩减，目标active，未发布。


## 2026-11-13 协调tail原包首次Stop提前失败与参数读取复核

干净main0c988fc89／24源／release库宿主dfac5586…固定后唯一新原包轮。Start7436／验签Status229ms通过，首次wait-os-absence读取procargs返回EINVAL，Stop失败duration1863ms，后续测试断言退出；HeadStopPrepared5，未得exit0／root恢复／第二轮，不能评价完整Stop10。回执当时写在断言之后所以summary缺stopDTO，本轮不补造；只取到Start协调tail完整来源约457／455ms，不当Stoptail。40原文件及Original/Ready/PreStop/provenance和5events已离线验证，精确3协调对象先留字节再identity/exclusive清理，globalL原身份／保护／bytes保持释放；PID+birth/exe与主端口独立缺席不补造退出证明。原字节核对后清Profile／程序／exe及临时证据根，不重跑失败源。

main4c2a826aa补Mac进程image/argv失败后始终原生BSD后置复核：仅明确ESRCHNone为缺席；live读错、observer错误、PID/UID/birth变更仍拒绝，同代zombie typed exiting仅Stop在原deadline内保持Unknown等待。原完整scan／manager／record／port/listeners／原ExitWatch exit0与endcheck不变。main382cf593f仅将停止测试DTO先写盘再执行原断言。40项不同相关回归及严格工程检查通过，privatecfgtest模块编译path与lint失败保留并修正、原规则未放宽，actualdiff只读审查无阻断。修复后尚无官方服务轮。

两组证据及限制／源／binary／临时清理见 [client-process-snapshot-r1](0.2.24-contracts/client-process-snapshot-r1.md) 和.adg/evidence/0.2.24/client-tail-native-r1/、client-process-snapshot-r1/。下一步固定新release一轮同时取得原30/5/10与Stoptail段，不能把子进程确定性反例当原包通过。原11345ms收尾超限／RootAbsent18和更早失败均保持。根中断／动作资格／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原全范围不缩减，目标active，未发布。


## 2026-11-13 参数复核修复原包轮：两代正常退出，第二close后证明拒绝

干净main382cf593f、25关键源及release库宿主95830240…固定，构建6m42s，唯一新原包轮。第一轮Start7937／Status231／Stop6205ms通过，第二轮Start10131／Status847通过、完整Stop11428ms失败；上轮参数读取EINVAL未再观察，两代服务/launcher都取得normalexit0与各4已捕获listeners缺席。第二根实际移动/readback且RootAbsent18曾提交，随后原post-close证明复核拒绝，原逻辑追加与pending17仅revision不同的pending19；总Stop内部10942ms。当前HeadRootRetirementPrepared19，不用临时18或磁盘事实回填两个历史已关闭通过。未单列原proof拒绝detail，不能仅凭泛型code指定具体内核字段原因。

新增tail输出共5份，仅第3份属于首次Stop：leasecheck1400／release426／完整source488578us，剩余3.806s；第二Stopcallback root-close失败，没有后续tail，不以Start/首次Stop计量替代。第二代登记721.603ms，48frame/archive／历史3/3/6保持，next应针对close事务与同次currentarchive认证的重复取得定位，保留所有source/candidate/postcommit、逐帧门槛、原readback/证明/deadline，不删最后证明或加预算。

79原文件离线核两代完整链与19events，首代binding/raw9events/vault保持，3新协调对象留字节+identity/exclusive清理，globalL原身份/Protection/bytes保持释放；PID+birth/exe与主端口独立缺席，源字节及全部材料逐字归档后清理Profile/program/exe和临时证据根，旧失败和工程target保留。详见 [client-exitfix-native-r1](0.2.24-contracts/client-exitfix-native-r1.md) 与.adg/evidence/0.2.24/client-exitfix-native-r1/。

只读侧审已给动作canStart/status容量资格同次postcommit私有projection方案，尚未实施或验证，不加预算外fullscan、不擦真实Stopped成功。根中断／动作资格／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原完整范围不缩减，目标active，未发布。


## 2026-11-13 保留后置Plan边界，采用租约目录元数据联合核验

实验删除archivedRootAbsent的inspect_retired虽通过两项静态损坏／计量，却不能证明之后Plan重读的时间边界等价，actual审查给出持续Plan损坏起点反例，因此未提交、全部候选source恢复baseline382cf593f，root_binding继续原三次来源观察。新增main5274b7048回归先成功inspect再改坏Plan，后续verifyreadback必须鲜读拒绝并保留篡改。候选hash／原测试日志／否决和临时清理独立留证，不算正式通过。

实际main5027b0ad4只合并NativeLease.check开始时physicalclient/profile/home三组相邻metadata为既有双遍union，saved identity/fullProtection／冲突拒绝保持，后续claim/pins/owner/SQLite/deadline原位。真实四库外进程竞争确认持锁blocked→normalrelease后unblocked，三根×三类变化两mode都拒绝。Protection134→106、xattr214→170，局部收益仅几百微秒，不足以单独开新原包轮。69项不同受影响测试及最终严格Clippy/fmt/diffcheck通过，证据见 [client-lease-directories-r1](0.2.24-contracts/client-lease-directories-r1.md) 及.adg/evidence/0.2.24/client-retired-inspection-rejected-r1/、client-lease-directories-r1/。unitTMP空，全部材料逐字归档后清理两批临时根。

下一步继续等价底层成本和动作canStart/status同源资格接线，不删后置Plan／帧／proof门槛，不为微小局部收益重跑原包。原第二Stop11428ms／pending19失败保持，本批无官方服务轮。根中断／动作资格／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证与固定候选原完整范围不缩减，目标active，未发布。


## 2026-11-13 正常Stop回执后继资格同源接线

main33d0a9fca将成功root Stop的canStart从通用Stopped状态改为本次close postcommit canonical history／indexednamespace的原算法资格候选，之后原完整history／currentarchive／Plan／receipt／scope／storage／Head和原AuthorizedReadback复核全部通过才返回私有typed值。外层协调／程序source／deadline失败不提取资格，仅正常Stop消费，Restart和普通目录沿原路径。容量否仍保留真实Stopped成功／failure无，只关canStart并recoveryRequired，status一致；未知／到期仍失败，不吞错误。后继Start自主核验，不把资格当启动授权；公共JSON和持久格式未改，原Stop10／Restart子预算不变。

六项新测试含4095events容量、候选后8类损坏／error、初始与候选后expiry原pendingdemotion、资格后新用户root仍Start拒绝；首代与完整四帧第二代计量无/有投影均12／72frames，完整第二history5/5/10和三事务source/candidate/postcommit次数相同，无新增fullscan。71项不同受影响测试及严格工程检查通过，actualdiff只读审查无阻断，fixture独占锁busy与testfactory私有编译失败均保留并修test，锁和断言未放宽。证据／8源与binary／临时根逐字归档清理见 [client-stop-eligibility-r1](0.2.24-contracts/client-stop-eligibility-r1.md) 和.adg/evidence/0.2.24/client-stop-eligibility-r1/。

本批实现资格差异，不宣称固定原包／Stop预算通过；无官方服务轮，原第二Stop11428ms／pending19失败保持。根中断／缺Ready／一般图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选完整范围不缩减，目标active，未发布。


## 2026-11-13 已索引代次目录的metadata扫描收敛

main287ee0c9d仅合并verify_generation_slots逐child无内容读取阶段的元数据扫描：parent原private前验／完整recheck后验，每child nofollow目录打开与固定初始identity/fullProtection、两轮named/held及ACL/xattr鲜读比较；外层UUID/sort/len511/names前后exact/generationsprivate/managerrecheck保留，之后才读取原来源资料。无childpin/token/内容逸出，不改任何Plan/frame/archive/source或三事务边界／proof／deadline。允许metadata采样时刻变化，明确child自身重复采样也减少，不宣称所有瞬态竞态等价；actualdiff只读补审无阻断。

完整48frame/archive/history3/3/6对照中capture339／source61／namespace60全保持，Protection61744→54373、xattr103631→91157约12%减少。初ABBA519/452/445/528ms与最终641/584/573/682ms全部保留，后者约12.6%局部收益，时间有噪声，不能选快值推断Stop10通过。目录反例含ACL／第二child／empty／limit及parent尾界；完整原件315＋协调29＋命令4共348项不同测试及严格Clippy/fmt/diffcheck通过，产品helper字节与最终test补充执行范围明确。证据／五源和binary／临时逐字归档清理见 [client-generation-directories-r1](0.2.24-contracts/client-generation-directories-r1.md) 与.adg/evidence/0.2.24/client-generation-directories-r1/。

本批无官方服务轮，原第二Stop11428ms／pending19失败保持，后继须完整关闭事务成本与累计原包总预算／资格接线复验，不用局部收益放行。Windows远端仍b2e89665d／0eef59a01无新推送。根中断／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选完整范围不缩减，目标active，未发布。


## 2026-11-13 固定源码原包正常双轮首次完整通过，转入剩余恢复范围

干净main287ee0c9d、28源和release库宿主9639b54d…核对，构建6m38s，唯一exact原包双轮exit0／completion marker／原全门槛通过。第一轮Start7160／验签Status231／完整Stop6016ms；第二轮Start9749／验签Status785／完整Stop9572ms，stopped status961/1840ms均<5s。两root移动/intent/readback/close完成，currentRootAbsent18、closed2、18events；成功动作canStart/status/preview一致，后继Start仍独立核验。此前11428ms/pending19及所有失败目录不变，不回写旧失败通过。

第二Stoptail leasecheck1140／release385／完整programsource523049us，结束余442132us；最终动作仅约428ms余量，定向通过不推导所有硬件/大树/更多代次压力足够。登记第二代599.753ms，48frame/archive/history3/3/6保持。两代normalexit0/各4listeners缺席，79原文件离线核完整Original/3phase/provenance/Plan/readback和旧binding/raw9events/vault保持，精确3新协调对象留字节+identity/exclusive后清理，globalL原身份/Protection/bytes保持释放；PID+birth/exe及导出主端口独立缺席，原字节与所有材料逐字归档后清理Profile/program/exe和临时证据根，工程target保留。详见 [client-generation-native-r1](0.2.24-contracts/client-generation-native-r1.md) 与.adg/evidence/0.2.24/client-generation-native-r1/。

已过正常双轮不再无依据重复运行，下一步接根已退休/readback存在但旧动作失败pending的独立显式metadata收尾：新确认+当前program/归档/currentGrok/进程manager/listeners缺席/真实双锁释放+独立RecoveryCompletion，不能延长旧normalStop proof或把旧失败变成功；只读实际方案及UI/后端缺口已留审查材料，尚未实施验证。root未移继续/回退、缺Ready/完整正常阶段和创建取消仍各自处理。NAME／一般图路径／完整进程集合与活跃刷新／复杂代理／WindowsW1W2与CLI安装升级卸载接线／官方交付verifier/README／固定appDMGMSINSIS等完整剩余范围保持，非整版接受，目标active，未发布。


## 2026-11-13 已退休根显式收尾接线与反例通过，原包专用验待结果

功能main848ca8c15／test入口cbc6174de新增独立RecoveryCompletion与精确root preview/UI门槛，只处理current pending且根已archive、现存Retireintent/readback完整、3phase/live均绑定、exactreleasedtree/currentGrok/逻辑根缺席的metadata收尾。digest从独立Plan推导既存bytes核对，不补文件或学习新基线；新用户确认domain绑定完整head/history/source/origin/archive/scope/live/HOME，单次新的stop10恢复预算不延长旧normalStop，不把旧失败变成功。当前fullprogram准入、旧Live相关进程/监听/HOME/manager缺席及双锁exclusive/release是新证明门槛，无法转成旧Stop/movement权限；source/candidate/postcommit和所有原件/帧/Plan门槛保持，提交后失败有安全pending保全，不伪称COMMIT回滚。紧邻同一确认重复完整核验后只返旧闭合revision，不加revision/公共schema。

actual审查P1（image扫描后缺currentfacts）修为check_closed再次核Head/fullreview/selection/OS/locks/deadline；P2（专用测试缺最终byte证据）补finalize/duplicate后7artifact完整快照/原Option字节相等，再独立注入afterimageGrok变化拒绝。5fixture/原件320＋协调29＋owner20＋命令4=373不同后端和前端94通过，严格Clippy/fmt/typecheck/diffcheck通过；初编译slice错误保留。Node/Vite119缓存文件原字节压缩核对后清理，unitTMP空，11源/debugbinary范围及全部证据逐字归档后清理临时根。详见 [client-retired-finalization-r1](0.2.24-contracts/client-retired-finalization-r1.md) 和.adg/evidence/0.2.24/client-retired-finalization-r1/。

当前固定cbc6174de／32关键源重建release库宿主，随后唯一原包专用轮会刻意保留原Stop中断失败，核新的explicitclose/duplicate10s/afterimagedrift refusal，尚无结果，不当成功。正常原包双轮之前287ee0c9d已过不重跑。根未移continue/rollback、缺Ready/阶段证据不足、创建取消／一般图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原完整范围保持，目标active，未发布。


## 2026-11-13 已退休根显式恢复原包专用验证通过

固定干净maincbc6174de／32关键源及release库宿主b22eb8b7…核对，构建6m42s，唯一exact原包专用轮exit0/专用marker与原全断言通过。官方2.99 Start7451／同连接验签Status238ms；真实正常退出与rootmove/readback后测试专用中断metadataclose，原Stop5204ms仍失败、pending8。新的publicfinalize2694ms<单次恢复10s关闭RootAbsent9，同一确切request重复2691ms<独立恢复10s返相同9且无新增revision。旧失败不洗成Stop成功，正常双轮此前287ee0c9d通过未重复运行。

程序完整scan后第三次duplicate注入合成currentGrok变化被当前复核拒绝并保留；before/finalize后/duplicate后7trackedsource存在性与完整bytes独立快照逐项相等且先于该后继变化，P1/P2反例路径均取到实际证据。9canonicalevents及原Original/3phase/provenance/Live/scope/artifactPlan/retirementPlan/intent/readback与素材blob链离线重验，exact3新协调对象先留字节+identity/exclusive清理，globalL原inode/Protection/bytes保持释放，PID+birth/exe与primaryport独立缺席；51原文件和全部材料逐字归档后清Profile/program/testexe及证据临时根，工程target/原失败不变。证据见 [client-retired-finalization-native-r1](0.2.24-contracts/client-retired-finalization-native-r1.md) 与.adg/evidence/0.2.24/client-retired-finalization-native-r1/。

本批关闭的是已退休且现存完整readback的显式metadata收尾/重复确认/scan后拒绝定向验证；未宣称旧19→20实机、大树/多代性能、root未移continue/rollback或阶段证据不足已通过。根未移/创建取消、缺Ready、一般客户端图路径/完整进程集合与活跃刷新、NAME、复杂代理、WindowsW1W2/安装CLI更新卸载接线、官方交付verifier/README与固定appDMGMSINSIS等原完整范围仍待完成，非整版接受，目标active，未发布。

## 2026-11-13 Windows W1三批接收与主线注册

实际远端代码b8f1bba91/文档61996e034，Windows已承接cbc6174de/61f0f99d8并按TEAM revision5交回HOME/Profile分离、当前用户PATH原语、MSI/NSIS024资源映射与独立探针。13个净文件逐项审阅后按四主题cherry-pick至main803b4abca，其树与贡献最终树一致；mainf8e3cf423再由本机单写父mod注册windows_cli024。82份原文档/证据逐字接收至docs3eda03484，未覆盖并行原型或旧失败。详[接收复核](0.2.24-contracts/windows-w1-receive-review-20261113.md)。

本机核52份原始清单、13份源绑定（10逐字一致/3模板仅CRLF-LF差异），descriptor/pin与官方2.99输入一致；7项process/8项PATH、MSI两格Costing/NSIS四格路径仅按Windows原日志范围接受，MSI/EXE为fixture装配且本体未传Mac，不当产品安装/W2。Mac独立宿主WindowsMSVC目标all-targets静态check、Mac严格all-targetsClippy/fmt及Python驱动语法过，无Windows运行或新产品包。

原工具/探针失败全部保留；Windows4个历史目录/11808文件共2108815151字节构建树及2份pycache仍未删，自动审批blocked by policy且命令未执行、未绕过；cleanupComplete保持false。本机没有新运行夹具，工程缓存保留。后继由本机接共享终端事务/撤销/恢复/卸载helper与生命周期产品消费者，独占Windows写集不抢改；需要实机时按用户要求停下给提示词，当前不重复W1或开W2。根未移/创建取消、一般客户端图路径/进程集合和刷新/NAME/复杂代理/发行验证/README和固定候选等原完整范围不缩减，P24-W1/04/05未关闭，未发布。

## 2026-11-13 Windows终端共享事务与W1-C交接

main22c1b0b19接独立Windows分支，复用同一terminal IPC/Context/private record/pending，不创建第二套页面或旧目录兼容。真实原生PATH接口负责TokenUser/类型/原字节/expected写后读回，shared policy负责精确增量/原值恢复/外部修改保留，cmd显式入口转交console ocxd专用早期模式及当前官方Node/脚本。完整镜像/选择/helper确认及末轮当前事实复核，中断仅重核后继续原before/after动作。main3db927764先保护尚有终端归属的D不被删除，提供--uninstall-terminal-cli --confirm单独撤销helper；main83b27e9ae补UI当前用户PATH范围。详[合同与W1-C写集](0.2.24-contracts/windows-terminal-transaction-r1.md)。

最终固定8个源文件，21项terminal（12新policy/transaction＋9原Mac；2项真实二进制ignored原样保留）、13项数据卸载、2项consoleIPC解析、11项前端及类型检查通过，严格all-targetsClippy通过，原格式失败与最终格式通过分别留存。原早期编译/工具输出失败只记录真实观察，不重建缺失stderr。Windows全产品cross-check在ring C依赖缺MSVC assert.h失败，尚未编译到本批产品Windows分支，不当成产品Rust通过/失败。本机没有Windows原生运行，真实cmd/PowerShell参数/stdio/退出码及ACL/注册表整链仍待。

本批单一TMPROOT内289份Node编译cache共4453578字节逐字压缩归档/解压摘要核对后删除精确根；测试Profile自动释放，无native服务、账户、日常PATH/Shell/配置、真实同步或模型操作；工程target及原Windows清理未完不动。证据.adg/evidence/0.2.24/windows-terminal-transaction-r1保留原日志、失败、冻结源、清理和可复核清单。

当前必须Windows原生接续：先MSVC产品编译，在隔离测试账户定向核终端/恢复/卸载helper，并将MSI/NSIS非升级卸载hook接到新helper，失败保留、升级不撤销。本机按用户约定在这里停止，TEAM revision7临时明确共享CLI单写到Git回交，提示词由用户复制，不自动派发。完整Windows生命周期生产消费者、程序卸载的完整进程门、W2/固定候选、根未移/创建取消/缺Ready/一般图路径/完整集合和刷新/NAME/复杂代理/官方发行verifier/README等原完整范围仍未完成，未发布。

## 2026-11-13 W1-C真实回交未全通过及W1-D原生入口

Windows源e004259fe/证据25b323801逐项接收，265份原文件1960818字节按blob复制、139份raw清单逐项SHA核对；10最终源7逐字/3行尾差额。两主题代码按abaa93ce2/b9742198b接主线：PrivateRecord1MiB调用/publication上限修正与真实隔离账户测试、非升级NSIS/MSI卸载hook。MSVCdev check/build/test no-run通过；10格新进程pending恢复、4类外部变化拒绝、cmd全参数/流/exit23、helper门与NSIS宏四格/MSI动作条件五格有原证据。原总exit101及allW1cAccepted=false保持。详[完整状态/修复与后继合同](0.2.24-contracts/windows-w1d-native-entry-r1.md)。

两个未通过项有独立原因：原官方.mjs被OS executable扩展名门误拒绝，PowerShell默认.cmd混合特殊参数exit1；新终端裸ocx及真实MSI卸载仍未验。main7cf337d5a仅纠正已核验Node的脚本角色，保留受管边界/no-link/普通文件/单链接/大小/全部摘要/完整树/清单门，Node/Bun原OS门不变；13项selection（2新反例集）、21项terminal、2项native shim host回归及严格Clippy/fmt过。main0d231aebc提供独立std native ocx bin，不经过Shell，Windows metadata typecheck过；未链接Windows运行、未切换产品原.cmd入口。

后继W1-D明确采用安装器拥有I/ocx.exe、仅PATH/登记schema3的启停事务、同目录ocxd专用提前模式及官方Node，Windows按TEAM revision8临时单写合同内CLI/路径/装配/安装器，补PowerShell/裸命令/原官方命令和真实MSI卸载定向验。本机在推送该输入后停止，不自动派发，不改官方或做schema2旧开发兼容，不扩到W2/双SID/全量。失败证据不覆盖。

本轮Windows测试账户/Profile零残留、隔离账户PATH原样还原、临时ACL还原；5新生成根26869205653字节仍因自动审批blocked by policy未删/未绕过，约25.0GiB与旧约2GiB另列保留，不说已归档即已清理。本机仅隔离host测试，证据/源摘要和清理见.adg/evidence/0.2.24/windows-w1c-receive-r1；未做真实业务同步/模型请求/日常配置写入。Windows生产生命周期/完整进程门、根未移/创建取消/一般图与集合/刷新/NAME/复杂代理/官方发行verifier/README/固定候选与W2等原完整目标不缩减，P24-W1/03/04/05未关闭，未发布。

## 2026-11-14 W1-D组件通过与MSI环境缺口

实际贡献b476fa933/证据7d8e5db85已逐项审阅；按三主题cherry-pick至a02b244e9，同树核对，211文件819674字节按blob接收，94组件raw与51MSI失败raw长度/SHA逐项重验。9执行源绑定2逐字/7仅CRLF差异明确保留，out04旧binary摘要缺失的限制不补造。mainfd2538e4d仅等价简化单目标循环，原Clippy101保留/严格后继通过、受影响current变更拒绝exact过，21terminal/3layout与fmt过。详[接收/唯一MSI补验合同](0.2.24-contracts/windows-w1d-receive-msi-followup-r1.md)。

Windows组件现在有原生事实：schema3不写/删安装器EXE，持句柄/source/身份复核；cmd与默认PowerShell7.6.9 Windows passing混合argv/流/exit23、新Token环境裸命令解析及停用缺席通过；原官方2.99版本和自定义H端口18241读取、原config字节/完整树验证通过；10格跨进程恢复/4外部冲突、schema2拒绝/已有PATH/通知注入及NSIS宏5格通过。不是固定候选/W2/完整NSIS卸载，不外推未测Shell版本。

真实MSI安装依次保留out01探针未返回/原退出-1、out02参数路径1619、out03系统DisableMsi=1的1625，**正常/失败卸载仍未执行，allAccepted=false**。下一步Windows在允许普通用户per-user MSI的隔离环境只补安装前提和这两项卸载；复用原包摘要，不重跑已过终端/官方消费者，不改当前机器策略/令牌/安装范围，不用helper或表条件替代。无可用环境只回明确环境事实，不能相同条件反复跑；本机推送后按用户约定停止给提示词，TEAM revision9临时补验范围生效。

本轮5个账户/Profile均缺席、隔离账户PATH原值和临时ACL恢复；新两根1944490289字节及unit3881472字节保留，自动审批blocked by policy拒绝删除且命令未执行、未绕过，旧约25GiB/W1约2GiB及原拒绝不覆盖。本机新TMPROOT归档核验后精确清，未创建Windows账户/服务/日常配置/真实同步或模型请求。证据.adg/evidence/0.2.24/windows-w1d-receive-r1。生产生命周期/完整进程门、发行sidecar最终来源生成/官方交付verifier/固定候选/W2、根未移/创建取消/一般图/完整集合/刷新/NAME/复杂代理/README等原完整目标仍未完，未发布。

## 2026-11-14 MSI补验环境回交，未执行卸载

实际Windows f66d9d59f树与mainfd2538e4d完全一致，无新产品/测试代码；文档24a4a4673的21份原件56573字节按blob接收、10份提升只读盘点raw逐项重验，原开发包在Windows再次SHA/长度核对一致。详[环境盘点接收](0.2.24-contracts/msi-environment-receive-20261114-r1.md)，本机证据.adg/evidence/0.2.24/msi-environment-receive-20261114-r1，未重复产品测试或创建空代码提交。

无现成隔离Windows测试环境：本机只读盘点Hyper-V已注册VM为0、Docker为Linux、无已配置远程目标；非全网结论。当前HKLM/HKCU Installer策略键32/64视图缺席，不伪称当前键仍为1，也不以缺席推断已允许安装；旧out03有效DisableMsi=1/1625失败原时点保留。msiExecutions=0、正常/失败卸载均not-executed、environmentReady/allAccepted=false；未改机器策略、令牌、安装范围、日常PATH/配置，未创建新账户/VM或运行安装/卸载/已过组件。

本轮新盘点Public根10文件5968字节仍因自动审批blocked by policy未删，命令未执行/未绕过，原历史拒绝不重试；无本轮账户/Profile/测试进程待清。TEAM revision10明确不再相同环境盘点/重试，等待用户提供可用隔离Windows环境或明确创建范围后才续原MSI-only验证。MSI对应验收和整版收口保持未完，其余完整本版未完范围不缩减、不发布。

## 2026-11-14 临时管理员MSI接收与生产生命周期接口准备

按用户指定fetch代码0f1dc195e/证据c84fc4a99，消费最新handoff；四笔仅10测试/账户/诊断/离线核验净差按ed01402a7/3eb32a3bd/42c030f6e/6b33016ba接入，181原文件826438字节按blob接收。固定原证明134份原件/133payload681196字节，9个内存坏证据反例全拒绝。实际管理员安装0、pending卸载1603/helper1原前后像保持、同SID原pending继续后卸载0/产品-1及D/H/runtime保留通过。原out01测试误判调度文字/build02宏语法101/null空串87都保留；不改旧普通1625事实、不把管理员补测当ordinary通过。详[管理员MSI接收](0.2.24-contracts/windows-admin-msi-receive-r1.md)。

普通用户验收/清理仍未完：新账户/两任务/MSI登记已移除，输入ACL与测试SID PATH及daily/machine PATH保持；SID尾1040 Profile Loaded/error32、System4/Registry328共享hive占用、85文件118556840字节保留；Public83文件184522112字节自动审批blocked by policy未删/未绕过，原其它拒绝不重试。ordinaryUserAcceptance=false、cleanupComplete=false、allAccepted=false，不重跑已过MSI/终端。Mac离线只核原Git证据，没有重算远端大包或启动Windows事务。

用户要求的下一共享接口已真实落码：main1da73491f复用原attested_status HTTP流程，新增typed当前观察与不依赖desktop_lifetime的retained-owner Stop边界；同TCP HMAC/private token/capabilities/record/source/POST保持，owner检查后再核record/source与原deadline，HTTP成功无normalExit/恢复权限。main35f41ecc0公开official_lifecycle024 LaunchPlan与ActionWindow，固定原2.99/current完整源、Node+mjs/Profile/H/白名单环境和继承绝对30/5/10/Restart30Stop子10预算，不拼Shell、不伪OS证明。产品Windows消费者尚未切换；不把接口测试说成生产已接。详[接口/写集/定向合同](0.2.24-contracts/windows-production-lifecycle-interface-r1.md)。

本机4launch/12transport/20原owner共36项去重测试通过，1原ignored保持；新增7项测试含ready/pending、错runtime/H/record、过期不连接、owner两个边界拒绝、原生检查中record漂移不发POST。严格Clippy首轮接口未公开消费者dead_code失败保留，通过统一公开re-export后最终all-targets -D warnings/fmt通过，未屏蔽lint或降断言。合成loopback/host原件临时根归档核对后精确清，工程target保留，无真实上游/模型/同步/日常配置写入。

TEAM revision11放行W-L1新增具体Windows owner与共享消费者Windows条件分支的精确写集，原包版本/完整bundle/descriptor摘要及一名隔离普通Profile定向表固定；Windows范围内连续排查批量修复定向验证及时推送，非必要不回Mac。跨管理器句柄连续性/独立客户端恢复持久协议缺共享格式时给最小草案，当前Unknown/Required保持，不暗降范围或自造第二事实源。固定候选前不W2/双SID、不发布；NAME、Mac根恢复/一般客户端图/完整集合/活跃刷新、复杂代理、发行最终sidecar清单/verifier/README/候选等原完整范围仍在，P24-W1/04/05未关闭。

## 2026-11-14 W-L1净差接收与四项W-L2接口冻结

revision138接收贡献3a9b71f7c/证据13ea948b2。7笔代码净树相同集成至6b3bd173f；380清单项3,046,760字节逐一一致，source06/07逐文件核对，CRLF两项单列，离线复核与原报告见[接收证据](../../.adg/evidence/0.2.24/windows-wl1-receive-r2/receive-result.json)及[当前合同r2](0.2.24-contracts/windows-lifecycle-shared-r2.md)。main12746c005cb2621d2f1f44d12c0af501613fef1a提供可编译四trait声明与已接生产Start/Status消费者，四项原生实现尚未完成，不称产品通过。

真实source06/360183c59 Start24965ms，Status848/855/849ms，同PID64904/port56328及3个native错绑定反例定向通过；source07仅动作回执增量保留Stop入口。Stop10005ms未确认，334原代次最终退出无force但sc1060/schtasks1仍在；config.toml30字节恢复、ID/SD不同。Restart461ms/第二Start240ms拒绝符合未决保护。完整Stop、Windows原件持久恢复、管理器跨进程连续性、supervisor缺席、Auto重选及取消/更新/H切换未验收，不用局部通过代替。

已消费shared-minimum-request.json：原件同卷原对象保管与不可覆盖代次、原创建者ocxd owner-host和SID私有fresh IPC、官方精确SCM/任务命名空间原生只读缺席、源/argv/父代次/原Job绑定的诊断角色四边界冻结。新角色合同单独说明产品语义：诊断非零保持raw，只有准确只读查询来源及独立native缺席均证明后才可作为CompletedDiagnostic；服务/root/未知角色/丢历史/force等仍否决，不改10秒或把exit1当不存在，不追溯放行旧失败。具体原语/反例先过再接生产；四trait无默认成功实现，Windows消费者原Required/Unknown保持到真实证明。

本机4项LaunchPlan/预算、12项验签传输测试通过，fmt/严格Clippy通过，仅共享主机检查；未在Mac冒做Windows实机。Windows接TEAMr12写集后可连续开发/定向核验及时推送，不再每个局部失败回Mac。普通MSI仍未验；本轮5账户移除、原Job最终空，5 Profile3,211,038,283字节、Public319,071,336字节与旧SID1040等清理未完。首个Profile部分删除及自动审批删除拒绝保留，不绕过。NAME、其它Mac/最终候选等整版原范围未缩减，当前不W2/双SID/发布。


## 2026-11-14 W-L2原语接收与普通令牌/启动环境澄清

revision139接收35e7336fcc64bcf3cab26ca8fa9688daba95a1f0/70ebd2a6728fbd352407950185dfcbd290e4c105。三笔按净差集成至301cb7e43，tree同贡献；371payload/2,715,343字节匹配，17项最终Git/工作字节及三组各26份实际Rust编译输入逐项复核。回执与离线重放见[接收证据](../../.adg/evidence/0.2.24/windows-wl2-receive-r3/receive-result.json)，[当前执行合同r3](0.2.24-contracts/windows-lifecycle-prerequisites-r3.md)增量承接r2，不重定四trait签名。main dbf974097d067e75c4de68f9fc681af8ab447b09另对齐两文件rustfmt，声明完整SD常量/环境约束；不是四适配已实现。

已接6项lib原语、1项真实短命query/parent/argv捕获及ocx.pid残留反例；raw1060/1和fixture无官方来源Unknown不变。最终MSVC debug bins0/160990ms仅贡献主机事实，14warnings与E0061首错保留。O/H/S模块仍cfg(test)，没有custody/私有broker/完整角色和source-bound Absent生产组合；productionExecuted/allAccepted=false。新隔离普通SID1046/elevatedfalse读取完整SD请求ACCESS_SYSTEM_SECURITY时原1314，只有owner/group/DACL部分读回未变；外层Job=true但寿命未知。原语进展不能抹掉这两项或用W-L1旧Start成功填新链空白。

普通用户范围正式明确：完整SD仍含audit SACL、label/attributes/CAP/trust/filter及OS返回控制信息，不缩DACL；句柄READ_CONTROL|ACCESS_SYSTEM_SECURITY=0x01020000，GetSecurityInfo(SE_FILE_OBJECT,BACKUP_SECURITY_INFORMATION=0x10000)。不可读/未请求/不支持/覆盖未知不是Absent，在首次原件变更前拒绝capture；pending后失败保留Required/custody，无Closed或后继。当前没有已确认满足完整SD的普通令牌环境，不能提升/启用特权/模拟服务或改策略凑通过；这类普通账户完整恢复生产Start可用性仍受阻，未来若缩范围需另行采用产品合同。本轮没有作该弱化。

broker验证入口确定为既有隔离普通交互登录的桌面Shell，bootstrap及挂起broker原句柄均须原生no-outer-Job，保持同SID/Profile/session/原创建句柄；GUI退出/新manager认证前后继续观测原broker存活且source/H/FILETIME一致。不以Explorer名称/DETACHED_PROCESS或单层Job无KILL推断，也不breakaway/改策略。该环境是已选待实证，不冒当前主机已就绪；Windows无桌面入口时只阻H实测，O1314也只阻依赖其完整正例/真实生产，不阻独立H/S/D适配开发。每轮有新假设才跑，不再重复同一普通前提失败。

本轮Windows账户1046移除，Profile登记移除但目录不可读/数量字节未知，Public26文件92,915,595字节仍在；早先87调用/删除后现场限制、自动审批拒绝及所有旧Profile/Public未完照存。后继归档失败须保留尚可访问的原账户/Profile，不能先删除再重建fixture冒原件。日常配置/PATH/策略未改，普通MSI未验。TEAMr13继续r2单写集并消费r3，固定候选前不W2/双SID，不发布。其它本版整版范围及NAME均保留。


## 2026-11-15 安全覆盖决定与Windows后继r4

revision140按用户要求消费c47953ccfe4c66a976285eacffd691e3a3b0c01b/fe1a54f6e299b96a6542c5b87ae384643bb9d163，handoff及历史两份shared-minimum-request；最新具体请求是current-reference/environment-decision-request.json。11个新轮次与handoff净3601文件导入，11清单3589payload/30,427,614字节逐一匹配；latest4份final-source以及library02/product01各39份Rust输入对c479核对。离线复核归[证据](../../.adg/evidence/0.2.24/windows-wl2-safety-receive-r4/receive-result.json)，复核范围不冒完整代码安全审计；Windows贡献仍在独占分支，未整树接入main，后续真实产品集成另净差审查。

H新证据已在普通marenben SID1003/session7实际桌面证明空broker原创建句柄/noJob、跨fixturemanager正常及故障/接续gen1..4同代存活；是H组件进展，actualGUI/officialOwner/production仍false，最后rawHostExitCode=null不补0。O已有单对象同卷移动/严格阶段/current CAS及partialwrite失败保留；完整manifest/目录整树custody/workcopy/实际move意图与引用/恢复/Closed仍未完。S/D仍未完成全生产组合。最新clean c479 debug bins0/156748ms、107warnings，最终lib8项；非release预算/严格Clippy/签名候选，partial测试不代完整原件证明。

采用[DMD AC-P24-08补充](../01-需求管理/需求/DMD-OPENCODEX-POST023.md)与[执行r4](0.2.24-contracts/windows-security-coverage-r4.md)：完整SACL需普通token未有的系统安全权限，没有已证明不改策略/令牌的完整普通环境；明确采用新代次windows-bounded-access-v1，READ_CONTROL和0x1f7的8个必需访问安全投影独立/合并成功才可捕获。完整审计SACL/整体SACL控制及未请求扩展ExcludedUnknown，不承诺full恢复；所含投影任一不可读仍拒绝。需要全部SACL一致性的场景不在新策略验收范围，不能用新名称掩盖保证变化。Mac分支不变，不启用特权/提升/改日常策略。

新manifest/phase原件schema-v2、current-v2及securityPolicyDigest固定范围，旧Full-v1/PartialDiagnostic不转换、不兼容迁移，真实不可构造的Bounded native proof仍待Windows实现。main2ebdd3ae0423903e9e713108a09a165ac36ead37只提交声明及注释，不切生产门；c479合入此main后继续本线。当前生产Start仍拒绝，直到真实整树内容/ID/8投影/存在性恢复、原进程/监督/来源和真实GUI闭合；不以r4合同存在当Closed或后继资格。

本机fmt/严格Clippy通过，仅共享声明编译；没有新增Windows执行或新scope8项正例。Windows按TEAMr14单写r2/r3既有范围及r4明确增量，先新mask/v2/原件/中断，H/S/D可并行，依赖齐再单SID原包闭环。原预算不变，旧失败不追溯放行、已过无影响项不重跑，不W2/双SID/发布。H成功两根22文件98,829,764字节归档清理已接，旧失败根/marenben账户Profile/7文件670767字节拒绝树与全部历史Profile/Public/普通MSI未完继续保留，清理自动审批拒绝不绕过。其它本版NAME/Mac/最终候选等范围未缩减。


## 2026-11-15 Windows r4连续增量接收与共享接口r5

revision141消费0549a660ee3f77ea243e798ec55b01f52545e492/a7baa88e1；最新产品源6e312f022，0549仅清理脚本不冒产品重编。14轮净5873文件导入，清单5854payload/57,943,378字节核验一致，GUI实际产品/测试输入对6e312、清理脚本对0549；[接收证据](../../.adg/evidence/0.2.24/windows-wl2-shared-receive-r5/receive-result.json)包含完整原局限。贡献保持Windows独占，未整批合入main/未安全审计全树，不冒整版验收。

真实GUI主窗口/native visible HWND/pageFinished/setup、普通marenben SID1003/session7、空broker31484原句柄与四代manager连续性已组件通过，observer0、manager0/1/0/0，空broker rawexit未知不填0；officialOwner/production仍false，旧失败broker40588退出未知/现场留。成功Profile子树685文件119,851,895字节原SID ZIP核验清，Public20文件178,669,957字节源仍保留，控制器8拒读/私有归档未独立复读，overallCleanupComplete=false，旧拒绝不重试。

O实际8投影、v2整树/阶段/current CAS、两锁native只读、Ready/PreStop工作快照、历史后继/Closed/Stop接线均有定向进展；完整官方副作用/NativeLease及真正原包H-S-D生产未完成，公共Start gate仍拒绝。B2 CAP/trust非空1314/5不补签，普通MSI/全部历史清理不关闭。r4有界安全保证/排除与原预算不变。

已落实两份最小共享请求：8e2699a91将Mac私有declaration纯解析/helpers/10旧测试提为modules/runtime/client_route_declaration，Mac复用；API先拒超2MiB，None不等空。358e7fc4717787616f049f3bdc1cfe02f6d353f9将Windows managed Status采样真实起点一次5秒传fetch/light/recovery，fresh来源debounce不重用授权，迟到/未知/失败不清恢复；Mac/fixtures默认旧路径。main只保守native桥接，合入时必须保留Windows既有原Closed recovery_state_at，唯一merge-tree冲突及解法见[r5合同](0.2.24-contracts/windows-shared-routes-status-r5.md)。本次不改Windows独占其它产品逻辑/不打开Start。

本机route11/status33/source10/Mac会话17共71通过，2原ignored保持；fmt/严格Clippy通过。首次纯提取import位置E0753修复，事实写证据。Windows须合入后MSVC及同deadline实际native回归，接口可编译不当原生证书。TEAMr15沿允许集继续官方副作用/NativeLease、原包闭环及实际GUI官方owner组合，不重做旧空GUI测试、不每关回Mac；原先NAME/Mac/最终候选等整体范围继续。无日常配置/策略/官方包修改、真实模型/同步，固定候选前不W2/双SID，不发布。


## 2026-11-16 r5回交接收与claim保管策略r6

revision142消费代码3e19075846dfea81ba8392bb60a3d3031afbf116/证据13e5c322351dba048fc8fe773aace56aae72e654。r5 RoutePin/同窗private StatusReceipt已由Windows接入，原receiver保留；4轮净2659文件导入，2654payload/31,356,233字节全核，4组产品输入114/116/118/118逐份对各自源核验。原生执行/失败/已忽略组件保持，接收是证据与来源复核，非整树安全审计或新生产验收。实现仍Windows独占未整批集成main，[复核证据](../../.adg/evidence/0.2.24/windows-wl2-claim-receive-r6/receive-result.json)及[后继合同r6](0.2.24-contracts/windows-claim-anchored-custody-r6.md)限定当前事实。

原claim/N/L/K四文件kernel ranges与固定Bun SQLite/真实rusqlite互斥、KnownFolder原Tokennamespace反例有定向通过；原封存Ready/PreStop/current三产物接共享内容视图，非产物节点仍严格不变。不能把content eligible或partial组件当NativeLease生产mutation、真实Closed采样或官方副作用完整性；原Start门仍拒绝，官方初始化/ownerreceipt/broker官方Owner/H-S-D生产仍未实跑。真GUI空broker前批通过复用。

真实目录rename五格已定位：baseline与仅metadata根句柄成功，开claim文件/file+metadata/实际lease均raw5失败，shareDelete也无效且已撤回；library07 raw0为诊断记录完成，不将失败格改过，原5/6失败保留、continuous-open正例blocked/ignored。固定tgz的main-account.ts:463→:394/native-main-claim.ts:167证明claim-only凭据writer存在，未执行真实凭据/网络，N/L/K不替代claim。

本对话采用r6 windows-anchored-directories-v1：原已存在根/子目录和准确两稳定文件留原位，普通原文件逐节点同卷保管，变更临界段内保持原claim/NLK，逐事件canonical intent/完成/退工作/还原及children核验。拒绝放claim仅持N/L/K移整根的空窗；租约分段不允许mutation权限逃逸，也不在官方运行/teardown要取claim时一直持锁自阻。新v3 originals/current+storagePlanDigest；r4安全8投影/ExcludedUnknown、内容/ID/存在性与预算不变，不迁移旧v2或新设锁例外。main fac8573e55441fe514919c63c7f1d65181ff3577仅提供声明，不实现/开启产品。

明确剩余：原Codex根缺席时初始化与最终准确退根仍需独立无空窗排他证明，不能保留目录/两锁冒Absent，也不能偷放claim。r6先解决existing-root策略，不删除首次空环境目标；该项未闭合整版不通过。Windows继续其余软件实现，不将已定策略再称缺共享trait，若缺席退根有新具体原生方案再回交，不重复原raw5实验。

最新clean3e1907584 debug MSVC bins0/77327ms、library08为10过2原ignored；本机仅声明fmt/Clippy通过，未新增Windows运行或原包正例。最新30scene/155文件1217258字节私有ZIP/原scene保留，前轮39scene/6scene分别留；ZIP不克隆NTFS元数据。旧broker40588/Public8拒读、CAP-trust非空/B2、普通MSI及全部删除拒绝/历史清理未关，overallCleanupComplete=false。TEAM16沿r2-r5写集和r6新布局继续，先原语/中断/整树后再单SID原包前提；无日常配置/策略/官方修改、业务同步/模型请求，固定候选前不W2/双SID、不发布，其它本版完整范围保持。


## 2026-11-17 r2首次Resume前超时：受限恢复与性能r7

revision143消费44bb9bdbabc78f1754b9df135f3d183205fcb9ec/e34f3fe8a，精确已载入产品357210891。最新两请求minimum-recovery-interface-request/shared-performance-request已读；远端净79018文件50轮，仅当前任务4关键轮次+handoff555文件导入接收，550payload/12,872,546字节全核，其它远端历史未整批接入/安全审计，不宣称全部Windows实现已集成main。[接收证据](../../.adg/evidence/0.2.24/windows-wl2-preresume-receive-r7/receive-result.json)与[r7恢复及性能合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)固定事实/允许集。

实际SID1003/session7 r2 Start30699ms/原30000超时，原Node21500/FILETIME134393657318681068未Resume，broker75744/FILETIME134393656723346634留；原Job1active/total1。canonical7e3d4697…seq9 Preparing/Released，3原文件(auth3/configtoml51/Grokconfig3字节)保管到2custody，3工作副本已发布，4anchors留位。OriginalSession tag未返回不代表无变更；原件已移动，不能只force或开新场景重采。原失败所有前缀/进程/custody及40588留，当前恢复并未执行。

已定当前受限外部任务修复：Windows先实现/合成验证及当前只读全预检、完整r2归档，形成准确2目标+3原件计划，再依据AGENTS高风险恢复单独确认在原Windows对话确认本次执行。旧50580/54692授权不能扩到21500/75744；原宿主已有Job控制可用性须真实核，不假设新image能过旧native认证。原Job明确静止后若旧broker的Store句柄阻止归还，准确broker失败forced也必须显式列计划/确认，绝不normalStop/Closed。新修复器只按批准synthetic任务范围和新native完整对象/8/字节/children/claimNLK独立核验归还准确原件，输出ExternalOriginalsRestoredBeforeResume、保留旧seq9及新外部修复记录，不复制ZIP重建ID、不接管旧Owner/Job、不授自动后继。

未来原宿主在首变更前安装PartialCaptureAnchor和可达恢复控制，失败无tag也保原部分Store；新增BeforeFirstResumeRecovery prepare_abort/quiesce_launcher/restore_originals，fresh独立恢复动作最多10秒，原Start30保持失败不可补时。原从未Resume/Job/source/实际prefix证明与新lease/原件完整读回必需；forced结果不兼容NormalExit/StopAccepted/Closed/后继。main 89e0195acb7a0c6e7549eafa376ec520ce981546只给trait/window和2期限反例，未热补当前宿主或实施恢复。

共享program_image完整校验副本四样本6079.433/6353.553/4805.450/8928.700ms，selection约3ms；源码事件推断至少8次不是原实际阶段计量/唯一因果，不能乘成30699ms生产下界或release通过。r7明确Windows单写共享program_image的Windows优化/计量与tests，Mac共享语义复核/集成；完整inventory/每byte/hash/安全/来源/真实操作边界扫描保留，无旧摘要缓存/元数据替代/删扫描/延预算。先实际计量再单pass优化；跨边界合并需要另行具体证明，本轮不自动许可。当前恢复前不重跑真实官方，可在自有副本做独立组件/编译。

本机新2个恢复期限反例与fmt/Clippy通过，仅共享声明，无Windows现场操作或性能算法优化。TEAM17精确新写集见r7；原r6原缺席退根/初始化/ownerreceipt/完整H-S-D/最大容量/普通MSI/CAP-trust/NAME/历史清理均未完，公共Start/全版accepted保持false，旧失败/删除拒绝不重试，原预算和r4范围保持，无日常/官方/业务/模型/W2/双SID/发布。


### 2026-11-17 revision144：异步并行与r7只读准备接收

按用户最新授权，本对话目标模式推进至发布前就绪，Mac与Windows按独占写集异步开发；不因Windows单线停点停止Mac独立工作，不授权本轮正式发布。Windows73d8eb155ae474940cecf4d8933c41d924b29a83 / docs48181b0b59eda60e32ce31dc670ef689c542183a本轮只完成r7只读工具准备：净503份证据导入，502载荷/6,714,147字节核验；10项最终源码逐项对Git，2项原字节一致、8项仅CRLF差异均保留原件。接收详[回执](../../.adg/evidence/0.2.24/windows-r7-readonly-receive-r8/receive-result.json)。旧handoff尚未追加r7，以不可变新轮status/SUMMARY为此次事实源；不是Windows整批安全审计或生产集成。

| 本批状态 | 结果与边界 |
| --- | --- |
| PS5.1只读查询/内容helper、归档fixture、解析/格式 | PASS；只证明工具准备，未触碰当前r2现场 |
| MSVC测试/产品 | BLOCKED：两次raw101，allocation/LLVM OOM；未执行新断言，未生成新产品 |
| 当前r2归档与native preflight | PENDING：19对象/25记录定位已有，未部署执行；计划executionReady=false/approvalRequested=false |
| r2原件恢复、Job当前证明、beforeResume生产恢复、扫描优化 | PENDING；75744/21500及custody不变，未授权/实施当前现场forced或恢复，不以新镜像绕过原认证 |
| 生产Start/Stop/Closed、W2/双SID、发布 | 未通过/未跑；不得把工具准备计为产品验收完成 |

Windows恢复编译环境后沿r7继续独占任务；不要同前提重复OOM构建，不提升普通验收token、不改日常策略。仅当现场归档/真实预检及精确恢复计划齐备时再单独请求当前r2恢复授权。Mac不改Windows scanner/原件/host写集，不把此停点转化成跨端往返的新整轮。

### 2026-11-17 Mac独立P24-05装配检查

main 462e2476a9275cdf82e9e3235ed84820ac2c28c7新增离线官方交付核验并接stage_runtime_bundle_024与Mac sealed .app打包：实际流式核对成员顺序/完整集合、文件摘要/模式、受限相对链接、锁定项目/依赖、恢复原包以及安装后的官方包全部文件与固定官方tgz原字节。重算描述符/外层摘要不能掩盖官方源码修改；追加gzip成员/非零TAR尾部、额外/缺少成员、越界链接及重复JSON拒绝。核验失败不输出后续打包配置，失败资源保留；装配证据上传workflow artifact。现有publish_release.py的0.2.24未完拒绝门保持，尚未实现整组最终发行制品/签名/公开资源验收，不宣称此离线检查取代它。

[定向证据](../../.adg/evidence/0.2.24/official-delivery-verifier-r1/result.json)：19项测试通过；真实Darwin包246,638,059字节、11,467成员/633,363,932展开字节、102个锁定依赖、2,664官方文件通过，descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3；原包b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。压缩尾部中间反例曾失败，修复说明及最终源码/日志保留。测试临时夹具自动清理，未解包执行真实程序、未写日常配置/同步/模型请求；此处不是新sealed .app/DMG验收。

下一步本机独立处理最终sidecar/已封装程序来源绑定与受影响Mac候选；NAME剩余路线仍须闭合，Windows沿当前写集独立继续，最终固定集成候选后才W2。当前不创建版本标签/发布。


### 2026-11-17 revision145：Mac三个程序来源与真实单appDMG

本机继续独立P24-05，main 98787e64f152515a6aa8e5b3c31b0330beaeca67新增Mac封装来源核验：直接消费同轮Tauri转交Cargo的成功compiler-artifact/build-finished回执，固定项目src/main.rs及src/bin/ocx.rs、ocxd.rs三个准确输出；源提交/产品工作树不变、封装MacOS成员集合恰为三项、原封装codesign严格通过。对各compiled/sealed临时副本去签名后统一无时间戳ad-hoc标识，再完整字节比较；原产物从不重签或改字节，操作后重核原SHA，副本清理。未知输出/重复或失败回执/旧helper/额外成员/链接/签名失败拒绝。

Tauri wrapper增独立标准输出证据（16MiB上限，截断不能通过），workflow传Cargo JSON并在Mac封装复核后消费唯一成功尝试、保存build-binding artifact；下载重试分类/原失败保留不变。5项新边界测试＋3项真实子进程重试测试通过；YAML解析与内嵌Python编译通过、diff检查通过，未冒actionlint或完整GitHub workflow运行。

[真实工程证据](../../.adg/evidence/0.2.24/macos-build-binding-r1/package-evidence.json)：产品源main462e2476a9275cdf82e9e3235ed84820ac2c28c7，以同轮Tauri实际Cargo输出构建/封装三个程序及官方资源，Release构建305593ms通过。sealed Desktop SHA9dd6cf70f6cad7c25ef597e4b585e0ab701a45fc13c3cc4c813383566e499913，ocxd c142ddbd73e17f55da247aaacc9ed0d149aaf1643947ce59d0290f0c4f882632，ocx 5a33b3d9ba6cee1192e95a454fae14ccc7a0a9f0fa1d886b1e0de0b9c011ffdf。verifier后继提交只改装配/测试，不冒产品由该后继SHA编译。

单appDMG278285528字节，SHA52097916684d23cb5ec4da592aa114b5559a3750b3bfacc8f19ea3ab382f090c；真实hdiutil只读挂载、完整原封装tree/签名/逐成员官方资源比对、单.app＋Applications精确链接＋Install.txt通过，原app不变/未安装未启动服务、临时mount/scratch退出，去签名诊断副本精确清理，工程包和历史app明确留作工程产物。非notarized/非updater签名/非最终集成候选，不交用户升级或公开发布。

中间错误保留：另行cargo build与Tauri配置不同的主程序比较先拒绝；同Tauri产物去签名后仍两字节__LINKEDIT签名分配元数据不同，实际位置/命令归诊断证据。最终方法在副本统一签名身份后完整一致，未按偏移忽略字节。原失败与新成功分别保留，不回填成功覆盖。

P24-05当前完成这一独立工程装配缺口，最终发行整组校验门仍关闭；NAME方案、Mac剩余一般客户端图/取消/缺Ready/真实集成及Windows生产恢复/预算未完成。本机继续独立工作；Windows无需重复本批共享/页面检查，按r7恢复与性能独占写集接续。


### 2026-11-17 revision146：精确Mac工程包首次供应复验

产品源462e2476a9275cdf82e9e3235ed84820ac2c28c7／sealed Desktop9dd6cf70f6cad7c25ef597e4b585e0ab701a45fc13c3cc4c813383566e499913同前述DMG，不换原包/预算/配置语义。[首次供应证据](../../.adg/evidence/0.2.24/macos-462e2476-first-supply-r1/summary.json)以当前普通私有Profile、生产startup、无OPENCODEX_SANDBOX执行，OS sandbox负控证明外网/日常HOME拒绝；先置当前格式合成配置只为关闭自动在线/同步/启动，不调用Start或业务。

73.419秒完成观察和清理：UI从展开包→核对固定依赖→核对入口→内置程序已就绪→概览，正确无陈旧“接入后可启动”；current真实版本2.99.0及原官方SHA、实际Node v26.8.1/npm11.19.0/ocx opencodex2.99.0 exit0；临时配置字节保持、runtimePortRecordAbsent、原app和副本binary不变。此耗时是首次离线供应＋GUI观察清理，不冒30秒服务Start预算通过。

截图/AX与实际调用、来源/程序镜像、合成启动配置/日志均留证。仅测试GUI结束用SIGTERM raw-15，不称正常服务Stop；无forced fallback、原生进程组残留为空，准确临时Profile/复制app/运行目录删除，日常配置/安装应用/真实同步/模型请求未动。accepted:false、frozenCandidate:false保持；源码未新增产品变化，Windows不需因此重跑已通过项。后续仍是NAME/Mac其它生命周期与最终候选/Windows生产恢复闭环，当前未发布。


### 2026-11-17 revision147：Windows异步请求接收与Job观测决定

收到最新代码2306f39880fde481a182e724c7a58bbbbf2a0310／证据571de2ee0ed0075229bd41a142ff5476c0949c4c。只导入新的r7-async轮61份及handoff增量；60载荷254,688字节、实际query-schema06的25源码对实施2f3b67b4169a587e3cf91c43ac014370b11b2b1d核原字节/hash/CRLF差额，[接收回执](../../.adg/evidence/0.2.24/windows-r7-async-receive-r9/receive-result.json)。同源安全查询与严格beforeResume DATA附件的7项MSVC测试raw0通过，仅controller夹具；原E0432/E0583/unknown字段失败保留。末尾98bd87926/cd427f938/2306f3988有扫描/只读采集器/命令谱系代码更新，当前这份证据未覆盖，不冒已测试或完整产品构建。未整批merge Windows产品。

已消费blocking-request.json。r7增补允许先用PSS单broker句柄完整快照＋仅JOB_OBJECT_QUERY本地DuplicateHandle的原生原型，唯一内核Job／成员／历史accounting与原从未Resume组合证据均核；禁止same-image绕过/注入/全机scan/提升/权限修补/外部杀Node，不修改shared接口。只有先通过原生夹具/所有拒绝测试才应用现有只读R0；如果不可读保持Unknown并继续独立线，不在Mac冒通过。原R1实际终止/原件恢复依旧需完整具体计划与用户单独确认，此决定不降低安全范围或给执行授权。Windows可自行取新docs继续，Mac不要求用户为每个只读工具步骤转交批准。

Mac P24-NAME新增原样官方函数动态边界核验：中文provider键被拒绝，官方显示名含半角/被拒绝；中文渠道·模型的显示投影可行且不改slug/保留能力sentinel，但这只是原函数验证，不是实际保存/自动刷新/默认客户端验收。原“渠道名/原始模型ID”格式的采用冲突已提给用户选择，未收到前不擅自以新分隔符替换。证据.adg/evidence/0.2.24/official-name-boundary-r1/；实际包633MB隔离提取目录已清理，无服务/真实同步/模型请求/日常配置修改。P24-NAME仍未关闭，Mac其它生命周期可独立继续。


### 2026-11-17 revision148：名称方案采用、真实保存与官方刷新

用户明确采用“接受原生显示格式，继续验证刷新与保存”。DMD AC-P24-09及UI规范29.1.1同步采用，[显示合同r1](0.2.24-contracts/official-channel-display-r1.md)记录同一事务/来源范围。旧“方案未定”状态由本条替代；本次未动并行未提交原型，未宣称默认Codex界面已加载。

main f7f5f8b1649d89181187692dd37cd93a31958509先修官方允许的带点运行名在Desktop被错误分配为内部ID：2新事务/映射检查＋21映射回归通过（其中1重复），原凭据/模型/未知字段与引用按普通保存投影保持，原始预期错误测试日志保留。[证据](../../.adg/evidence/0.2.24/dotted-channel-identity-r1/source-manifest.json)。

main 634fc376cda19ca039e9fc30d3be83699058414a接中文/派生组官方显示：模型map值“渠道名 · 原ID”仅显示/转／，人工非空显示名优先；读回以同policy/唯一ledger displayName target/hash及重算值核验自动来源，不从相似字符串猜归属；外部改名保留人工意图，回合法运行名后清自动map、人工项保持。来源信息进入原materialization字段列表，不新增轮询目录writer/维护包/能力规则。原始模型ID、路由与凭据不变；超128UTF-16/控制字符保存前拒绝，不截断，页面解释原因并保留输入。

原样官方真实刷新发现并修复独立产品缺陷：中文内部key＋协议/hash后缀可超过64，官方loadConfig因此备份invalid并fallback默认，最终目录丢渠道。新分配先为全部后缀预留长度，带点/首尾/保留字/ASCII/64约束与碰撞后长度全核，保留持久mapping；无旧映射自动兼容/迁移。native02原失败、synthetic输入/错误/目录保留，不用后续成功覆盖。

[本批证据](../../.adg/evidence/0.2.24/channel-display-native-r1/adoption-and-results.json)：100项Rust定向（file29/consumer27/source37/display1/protocol6）＋26项渠道页面测试/类型检查通过；分类边界后续补测与全targets严格Clippy/fmt通过，最终源码逐hash核对。native04实际Rust保存导出三代中文/改名/人工覆盖＋协议分组配置，每代新进程在OS sandbox禁止日常HOME/外网/Keychain，原样官方saveConfig/loadConfig＋两次syncCatalogModels完整目录刷新，共6次通过，slug唯一、名字正确/配置读回和合成Key/原ID保持。无服务/真实业务同步/模型调用；准确临时提取包/Profile均已清，旧失败保持。

本条只完成名称技术路线与生产保存/读取接线及官方原生目录函数链的定向验证，不关闭AC-P24-09：默认Codex实际GUI加载、最终应用包重启/相应备份回退仍待固定候选。之前462e工程DMG早于本次名称代码，不能作为新名称包验收。Windows恢复/完整预算/普通MSI/最终集成W2继续未完；本轮未发布，目标active。


### 2026-11-17 revision149：并行Windows进展与PSS最小参数决定

fetch Windows89f55e76d4f5a6e599b0e9db8cb359fcadce0254／docs540025ef6，净接7轮1567文件＋handoff追加，1560payload/45,124,509字节全核，原内容保持；[接收回执](../../.adg/evidence/0.2.24/windows-r7-parallel-receive-r10/receive-result.json)。载荷核验不冒Windows全实现代码审计/整批集成/原生重跑，Windows独占写集保持，main仍634fc376c。

完整Tauri库typecheck在原同一次进程404677ms raw0；新partial capture实际产品测试链接3项raw0/166883ms，原Projection再导出修复后通过，不冒产品三bin/release；扫描15项原生、固定副本完整入口6366ms通过但不冒整体Stop10/Start30/Status5。r2真实完整归档10701files/1664dirs/1192222294源字节与原前后native/44项已回交（原WMI-null拒绝及修双来源比较原样留），原件/custody/进程/seq9保持，R1执行仍未Ready/未请求批准。

已消费PSS24unknown的新请求，r7追加明确允许文档化0x8在自有单broker原型取完整类型、绝不解引用/记录ObjectName；OS内部会捕获名称的事实明确承认，非“没有采集”。原全遍历/未知拒绝/QUERY-only/对象去重/原历史/未Resume/预算不变，成功原型后才R0，现场forced/原件归还须原具体确认。Windows可自行fetch继续，不必用户中转普通只读试验。Mac继续独立产品缺口，最终候选/NAME实际GUI及Windows全部未完不关闭。


### 2026-11-17 revision150：完整scanner净差接入与本机实测

main 7fade183da7b43fa88fff3e459404aac704ff21b只从Windows已测64bf7aa73a61be2b50469657bbc7515d486eca00取program_image.rs、program_image_windows.rs及program_image_windows_tests.rs三文件，逐原字节与最新89f55完全相同；不merge后继Owner/capture/原件恢复/消费者来源文件，也未接未完成Windows产品整体。单写Windows贡献仍继续，本机只是按净差接入共享main；后继原Source/Stop deadline消费者仍在Windows本线，最终真实生命周期需同树再验。

[审查/来源/证据](../../.adg/evidence/0.2.24/windows-scan-integration-mac-r1/intake.json)：新Windows每同次inventory保留原对象、全字节hash/全目录集合/空目录/同原SD-ADS-path-ID等前后fence、4workers全部join与原deadline，Entry/树摘要和限额不变，无跨动作缓存/删扫描；Mac继续原Unix路径/完整读取与独立manifest绑定。本批是这三文件语义及来源审查，非Windows整树安全审计或完整产品验收。

本机定向35项通过（镜像13、选择13、激活回退7、登记2），严格全targets Clippy及fmt通过。另在精确新临时私有Profile、内核禁止日常HOME/外网/Keychain，原样官方2.99实际供应与完整扫描1项通过：15646条/645549518内容字节，13393真实文件全读/hash计数齐，tree a29dedc203311ca4d6d4005db94b4a10d4fb84409283669e25272f080756b790与固定官方源b920ba…一致；单次inventory602915微秒，整夹具供应/执行66.56秒，不把它当Start30/Status5/Stop10或Windows性能结论。测试宿主原签名/原字节未改，未启动服务，无业务同步/模型请求，准确本轮复制程序/资源/镜像/Profile整体清理且缺席核验，证据全部保留。

r7 PSS0x8参数答复已随docs5d5d5a3f8推送，Windows可自行取用并继续原型/否例与只读R0；无现场恢复授权。当前原r2归档已具但Job当前证明/受限修复具体前提、future完整恢复接口与原预算、默认客户端真实名称、Mac取消/一般客户端图/缺Ready、最终候选及发行整组门仍未完成，目标active；不W2/双SID/发布。代码主线新SHA只开放以上扫描来源，不要求Windows重跑无影响页面/CLI/旧归档。


### 2026-11-17 revision151：真实Mac保存/重开及首次纳管默认引用修复

main cb82a7a06bfd19e74872b2d5b3ef3b47c50c98c6修首次单渠道纳管保存：旧transform在将选定渠道插入受管集合前筛defaultChannelId，导致磁盘defaultProvider/defaultModel正确而saved policy丢默认；新begin按实际JSON读回产生幽灵“待保存修改”。同一旧7fade真实GUI已观察，隔离Rust严格整policy相等反例失败（原工具输出、固定差异说明保留），仅改先插入该scope再解析已观察默认；未受管无关默认仍不纳入，不改真实磁盘其它渠道/Key/引用。31文件事务测试、全targets严格Clippy与fmt过；main 29bdae6fba4f58653e26e43063d0e112292468e1只补真实保存状态断言/稳定AX输入方式，不冒产品由后继测试提交编译。

[本批原始证据](../../.adg/evidence/0.2.24/mac-model-save-reopen-r1/result.json)留旧7fade三次界面轮/原source构建绑定及新cb82两轮。新实际Tauri Release267337ms，sealed Desktop c851af6e356c56634883828b6d359229d98682b25ff9c89332f9ecbe435b9d0d，三个bin同轮Cargo回执与签名归一副本全字节一致；原app字节/签名不改，不是updater/notarization/最终集成候选。

新cb82原包GUI自有临时Profile：生产startup离线供应→模型读取现有渠道→精确原生AX字段中文改名→详情单渠道保存→实际JSON读回→退出→新GUI进程重开；两代均真实页面“未修改”，无待保存输入。中文显示maps、原始vendor/custom/gpt模型ID、合成Key、unknown字段及准确defaultModel引用均保持。内核sandbox拒外网、日常HOME及Keychain，未Start服务/真实业务同步/模型请求。两个自有GUI进程组SIGTERM -15且无残留，精确临时Profile/appcopy/程序目录清理，原signed app不变；旧工具AX初次注册/误等确认/选中文字不足失败保留，成功不覆盖失败。

此处是Desktop模型页真实写入与应用重开验，不是外部Codex默认选择器或OpenCodex服务重启；AC-P24-09仍需最终集成包外部客户端/回退，其它生命周期不借此签通过。Mac新建根的Preparing/CaptureReserved取消路径已重新核实：普通prepare_cancel显式拒creation evidence、Index.close_preparation拒root_registration，根lineage未含对应取消分支；当前生产尚缺根保留/退根与独立索引收口，不能删除目录或改ordinary断言冒通过。本批未修改这条缺口，列下一本机独立开发项。

Windows最新41043665d55f7f65bd550e1543701dd7c64192d9已fetch，只有PSS0x8原型变更无更新docs，尚不计当前Job或原型通过；继续其独占恢复线，旧R1未经具体确认。主线精确SHA已推，不放最终W2/双SID/发布，目标active。


### 2026-11-17 revision152：Mac取消前置与Windows三依赖决定

main adcf4f94edf73c6afb2675a34438f066be8b2eb8新增借原独立Index的首次目录取消只读审查：Preparing/CaptureReserved、原已注册初始缺席、原Head/history/目录native身份/精确HOME/profile/未过期窗口、root空且无任何client-originals存储、Grok完整状态绑定，两次一致读回；Review不序列化、不构造movement/恢复/Closed或后继许可。5项原生文件系统测试与8项创建/后继回归过，strict全targets Clippy/fmt过。首次root_start_tests过滤0项原日志保留且不计，通过用完整generation::start_tests过滤补验。证据.mac路径见[结果](../../.adg/evidence/0.2.24/mac-root-cancel-review-r1/result.json)。本批夹具用tempfile guard清理，无持久服务/日常配置；缺逐fixture独立清理回执不冒全历史清理证明。

此批未接实际退根/取消事件、服务缺席/新协调排他、已部分捕获/后继代次以及UI入口；普通取消的原拒绝仍保持。取消全链尚未完成，不以只读前置过当用户可取消。下一阶段继续原guard保护的精确目录保管/读回及取消记录接线，不删除用户新数据。

新fetch Windows57c982b683f065ca73e3afc718d0127c82eae775／docs00997673a，净12轮及handoff追加。实际payload数/总字节与固定code/docs摘要以[接收回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r11/receive-result.json)为准，逐原bytes/SHA/完整成员核；这次是载荷和具体请求接收，不冒整批源码安全审计/Windows native或完整集成。未来partial slot/lease/claim/QPC/附件/storage等新原语与生产库测试陆续通过；实际原Owner全trait、完整return/10秒及当前r2原件恢复未过，原57/EXE/测试来源分别保留。

三请求已于[r7文末](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)一次明确：未有manifest采用独立strictv2 canonical/preparation存在性，不伪造摘要、不转换v1/旧v3；活动lease由同一原worker连续持有，失败停驻不Drop伪释放/不重开10秒，后继失败处置另需具体证据；PSS0x8仍24unknown，允许documented NtQueryObject仅类型＋单broker零访问复制自有原型，缺权限/未知仍拒，禁止名字/全机/私有结构/提升/现场变更。main 78dc2b8149c4b2f948f6de0d649bea7b46dada02只加BEFORE_RESUME_RECOVERY_RECORD_V2标识，四trait签名及CONTRACT_VERSION4不变。格式标识不等实现或原生证明。

Windows可自行消费新main/docs继续已有独占集，非必要不回Mac。当前r221500/75744/原件/work/custody/anchors/协调及seq9保持，R1具体计划/确认尚未完成，任何实际forced/return不因本合同获得授权。NAME默认客户端/候选回退、Mac其它原生与完整候选/W2仍未完，不发布，目标active。


### 2026-11-17 revision153：取消审查新夹具精确清理

main 144391b95cc82cbc191e187537c736fd95284918仅完善本批取消审查的5项测试清理：关闭原Index/Context后显式TempDir.close，12个任务自有临时根逐项缺席断言并输出准确路径，接收端按同12路径再次核缺席。5项保持通过，全targets strict Clippy通过，diff无空白错误；[清理回执](../../.adg/evidence/0.2.24/mac-root-cancel-review-r1/cleanup-explicit/observed-absent.json)。原152的guard-only局限由这一新执行证据补齐，不回填旧日志、未删除任何历史失败或旧拒绝目录。

本批仅只读首次目录取消审查，非取消功能完成。shared v2标识/三Windows具体决定已推main78dc2b814及docs ea5bf8f59；后继main含Mac文件/测试，不改Windows运行时产品或原现场。下一Mac工作仍为实际guard排他/目录保管读回/索引取消事件及界面接入；Windows独占恢复线继续，最终候选及所有原验收不放宽，目标active、不发布。


### 2026-11-17 revision154：首次目录受锁保管与Windows净回交

main cfd41bf6e4cd055b69d9d27a4ca1ccc25f7332eb在原首次空根Review上绑定Grok及独立历史摘要，新增实际官方Guard/NativeLease保护的同对象保管内核。持原claim交接、准确Guard marker与原Head/binding/history/creation身份，先落不可变plan和move-intent，再用原父目录描述符RENAME_EXCL归档同一根；归档后重锚两实际协调对象、释放本次Guard、完整树及逻辑缺席读回。途中失败保原目录/归档/意图与新数据，不自动覆盖、删除或回滚新目录。

[精确代码与验证结果](../../.adg/evidence/0.2.24/mac-root-cancel-preservation-r1/result.json)：最终6项定向测试、strict lib/tests Clippy及fmt通过；默认过滤的1项原生test明确ignored，由原同源宿主在独立sandbox批次显式运行5场景：成功、移动前中断、移动后中断、后来新增内容、原路径新目录。五项实际官方2.99 Guard/claim均到达目标判断并通过，宿主SHA2e2ac366aa55599c6e074cdbda63b6166a7f9f47e6696de60ca96a230fd905d3不变；原native01语法失败、native02和final/旧检查均保留。首次供应成本与10秒保管窗口分开；不把整批耗时或存储test当完整Stop预算验收。5个任务根及共享程序根已清理并独立缺席核，无服务、日常配置、Keychain、业务同步或模型请求。

**这不是正式取消功能完成。** MovementAuthority在生产无构造工厂，仅cfg(test)可用。原独立索引仍保持Preparing/CaptureReserved，不生成Cancelled/Closed/后继；移动后的旧普通解析必须拒绝。下一步接原独立索引对取消计划的注册、live/archived/中断解析、实际服务/来源权限工厂及用户入口；不得靠伪造Original references或普通Stop复用绕过未完成捕获。目录保管测试通过不允许提前暴露移动按钮。

Windows9fdd4c618a57dbe5c9b52cee01680d6637300b5a／证据f6f8677a5724a527985b21eca374734c6c06c8c6新增native-return与object-type两轮，已逐原成员/长度/SHA接收，并消费原handoff追加；[回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r12/receive-result.json)。57文件1107196字节原件返回证据、25文件175689字节类型原型证据均匹配，非Windows实际重跑或整批源码安全审计。原返回首轮raw32失败与两树保留，修复后5实际前缀及3故障阶段通过；原10秒分别作用于各新夹具，2项17.79秒总和不冒一次完整恢复通过。现在只输出allOriginalFilesAtClient DATA，claim仍持有、locksReleased/normalStop/closed均false；最终释放、原Owner/fulltrait/完整10秒仍未完。

类型原型两遍各91捕获/91遍历/259结束，67实际类型、24零访问DuplicateHandle raw50；完整覆盖仍失败，未运行JobQUERY或当前r2。原loaded357源码确认旧CLI硬编码ExplicitRecovery且要求forced=true，不能借其探Status；[只读源码核查](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r12/loaded-client-route-review.json)。没有已确认的新文档路线，禁止重复同前提试验或将raw50当非Job；当前r2恢复依赖保持Unknown，未来worker/v2/最后锁释放仍按TEAM24/r7既有写集独立继续，不要求用户来回传普通进度。

NAME原生格式、官方保存/载入及六次刷新、Desktop保存后重开结论沿revision148/151；外部默认客户端、最终服务重启/回退仍未验。当前未固定整版候选，不W2/双SID、不发布，目标active。


### 2026-11-17 revision155：Windows原worker与失败保留接收

Windows代码bf16cad3f61259591a8fafff6f8fb2805f5e99c0／证据3b343c66ba0878a387e47353099bdc569e5883d4已净接收wl2-r7-owning-worker-20261117-r1，150项载荷1240837字节，完整成员/逐原长度与SHA匹配；8个最终源码逐Git原字节或明确CRLF归一核对，7个最终子进程模式的phase/raw/action/thread回执逐项核查。[接收与审查](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r13/review.json)。原product01编译失败、product02两保护目录夹具失败、保留树/只读归档和后来成功均原样留下；未运行或删除Windows现场。

新增实际原Owner在活动锁前转到唯一worker；registry只持control，Status独立trylock且前后核原end，普通Start/Stop/Restart保持拒绝。活动LeasePreparation/NativeLease在原worker创建，仍!Send/!Sync；失败、过期、panic、断连或部分释放失败撤销mutation并在同线程保留原资源，不靠Drop/autounlock或新10秒补通过。成功返回也仅ReturnedForValidation，原slot独立验证及专门AbortRestored尚未完成，不能当正常Stop/Closed/后继。

product07真实MSVC全Tauri库3父测试过，1专用子进程入口默认ignored由父测试exact显式调用7次；成功、失败、过期、panic、断连、部分释放、释放已过期七case均有原entered/observed/cleanup及stdout/stderr。成功raw0且原4范围释放/同原fileID和bytes返回；六失败raw242仅任务自有子进程的明确强制处置，不当产品正常释放或当前r2恢复授权。原native范围冲突/准确ThreadId/原action读回保留。最终EXE79d05a4b125205b5acb9a88f3a03567378415fed114ed6d8074ac7d3861b070d；本机只审原回执和源码绑定，不冒Windows本机执行或独立完整源码安全审计。

完整beforeResume driver仍需把prefix/附件/quiesce/逐原件return/全树与8/字节/children/精确release/AbortRestoreVerified贯成同原10秒，独立v2真实映射、可达版本化host命令和原官方产品Owner实测未过。TEAM25/r7允许集不变，Windows继续这些已授权独占项，Mac无需先提供新共享ABI。当前r2的24raw50/Job证明仍单独Unknown，不重复已失败同前提；原件/21500/75744/原seq9和旧40588均不动，未授权forced/归还。Mac索引取消接线进行中，尚未固定最终候选，不W2/双SID、不发布，目标active。


### 2026-11-17 revision156：首次取消计划原索引登记及中断重开解析

main e84cff4610c8b1f712747c874f9102904f2e30e6接入Preparing/CaptureReserved→RootCancellationPrepared准确转换：原独立SQLite同一CAS/events事务登记实际Guard计划的id/hash/sourceRevision，原source/head/history/Grok/创建身份绑定，登记先于任何目录移动。根移动引用复用原Head指针字段，但新phase只准首次无Original refs的取消计划目录，普通原件退休/恢复解析不混用；缺计划、伪造成功字段或候选读回失败回滚本次事件。不转换旧记录或凭目录发现接管计划。

读取只从独立索引引用解析固定计划：完整私有plan/intent/readback原身份/保护/字节及末端fence，原source事件与history、逻辑scope、Grok、根身份/完整tree均核；准确区分live与archived，双根/无根、缺原intent、新内容或namespace漂移均拒。支持原Index关闭后按同绑定重新打开已移动/中断目录，保持RootCancellationPrepared；前端新增“首次目录取消待完成”阶段，原件证据NotRecorded、plan Verified、所有普通恢复/后继操作仍关闭。没有借成功保管回执生成Cancelled/Closed或正常Stop。

[代码/结果](../../.adg/evidence/0.2.24/mac-root-cancel-indexed-r1/result.json)及base+完整source.patch可还原实际执行输入。最终strict lib/tests Clippy、7项定向检查、前端typecheck和86项交互检查过；新phase即使伪报普通mutation能力仍不开按钮。原生宿主d75a6f60ce747ec85c4ef55e8119e7f78a46cf8142fb175f910b0081fc3aafbd在隔离官方2.99/真实Guard/claim中运行5场景：成功、移动前中断、移动后中断，均关闭原Index后重开并得到pending/无后继；新数据、新逻辑目录两场景完整保留且重开拒绝。供应/构建与原10秒动作窗口分开，不以整批时长当完整生命周期预算通过。

首native01五项均因遗漏显式状态转换在移动前拒绝，反例未到达不计通过；补唯一转换后native02五项过，原失败/宿主/原始日志保留。首编译OsString遗漏与后来Clippy多余借用亦留说明/原日志；初编译完整原输出未归档如实注明，不能冒其存在精确原源码快照。native02最终来源12文件/hash、原Cargo回执及原宿主字节完整绑定。

受影响session_index回归首轮100过/3败/6默认ignored，原raw101及395秒整轮日志保留。三败逐原stderr确认都停在已有子进程/private/tmp准入：本轮harness漏设TMPDIR，默认var/folders与原测试合同不同；不改产品或断言，只给同原宿主三项各自新/private/tmp临时根并行补验，分别0.174/6.799/10.347秒全过。原退出91阶段/SQLite父锁等实际检查保持，100已过项不重跑；6项显式环境夹具不泛称通过，其中新取消原生入口已按上文显式跑5场景。

10个本轮native根、两共享程序根、3个定向补验根、26个明确打印的新单测根均复核缺席，原任务进程组无残留；只对准确本轮路径作清理结论，不冒历史全树已清。全部证据约6.2MB，未保留重复运行包，不接触日常配置/Keychain/外网/业务同步或模型请求。

**正式首次取消仍未完成**：MovementAuthority无生产工厂，尚需实际服务/来源准入、取消完成事件与最后释放/缺席证据、历史/后继资格及明确用户入口；已存在新内容/部分Original捕获不能复用本分支。根未移动的其它中断/一般客户端图、缺Ready、NAME外部客户端与最终重启/回退、Windows原Owner完整恢复/当前r2及最终候选仍未闭合。Windowsbf16cad3f/证据3b343c66b已接（revision155），本次Mac净差不改其恢复/ABI允许集，可自行合入继续，不W2/双SID、不发布，目标active。


### 2026-11-17 revision157：Windows释放后完整读回及独立收口接收

Windows44aa85f6c8e54fe58b9719be3d0706f838f912ae（含前笔e2173af83）／docs b2b25c26e5d644c905ca45e89fd8315802fec449已净接收release-readback轮，103载荷1143798字节，完整成员/长度/SHA逐原字节匹配，17执行源逐Git或明确CRLF归一核对。[接收回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r14/receive-result.json)。这是原始证据/具体代码绑定及handoff审查，不冒本机运行Windows或整批源码安全审计。

原worker的ReleasedLease四文件与原stableowner五范围同次独占可得/显式解锁读回已接，activeprobe仍!Send/!Sync；遇外部owner或任意失败，原probe/partial资源留原worker。节点return完成且无pending/failed后才有一次finalattempt，释放后两树/retiredwork/parents/custody/anchors的原ID、所有所含8、bytes/ADS/children、原Source/current/route/claimquiescence/附件前后完整核；durable AbortRestoreVerified以后还有finalnativefence，不能只凭记录产新权限。privatefactory已能由真实读回产专门AbortedOriginals，但不签正常Stop/Closed/后继或打开Start，旧超时与v3原链不变。

product01两test已过但Err-only反例阶段可能被早错掩盖，Windows仅增强分类/目标阶段/elapsed，product02最终两项原生通过。五实际前缀0/101/141/254open/255sealed各原10秒的新夹具分别410/680/1981/4052/3584ms，包含自有native launcher/Job242、lease/nodeReturn/附件/显式释放与五范围探测及fulltree；**未含真实产品Owner/官方Source准入成本，不当完整生产预算通过**。ownerbusy raw33、foreignmember InvalidData、finalrecord准确注入消息、afterfinalmember达到durablefinal后的末端拒绝、expired TimedOut均实际到达目标且无重试/补成功。原worker七case、v1 strict四项及原Send一项定向回归已过。EXE d6a3fc32429ee4655c086743a3be6c17e4274f723e2b18bca8d9557f31b297a8，27新根缺席；旧拒绝删除目录和raw32/DACL四失败根仍保留，未在Mac删除或重跑。

状态表现81项，完整产品Owner同原worker driver、原slot返回复核、过期后状态/历史、独立v2准备映射、partial work/capture/缺根全部覆盖、可达版本化privatecontrol/fullsharedtrait及真实普通Profile官方整链预算仍未完成。TEAM26/r7写集和四共享接口不变，Windows可独立继续，不等Mac新ABI；当前r2的24raw50仍只阻当前Job证明/具体受限恢复，不重复同前提PSS/zeroDuplicate，不用旧cleanup探Status。原21500/75744/原件/work/seq9与40588不变，R1未经本现场单独批准、不原包新Start、不W2/双SID、不发布。Mac首次取消实际服务/释放与收口接线进行中，最终候选未冻结，目标active。


### 2026-11-17 revision158：首次取消实际服务准入、四协调器释放与独立收口

main8140844dcb6a363263f679db5f6245102947f46a新增真实CurrentService工厂：在原Review仍空根、同Head/HOME/data/原deadline下完整准入当前官方镜像和OS服务缺席；进入实际Guard/claim后，完整进程扫描只排除该Guard保留的真实Child。排除同时核原pid/uid/创建代次、完整argv、真实程序路径及明确HOME，前后复核原Guard/原生锁；同名、其它同HOME进程、缺HOME或不可读对象仍拒。不从caller PID/DTO/stopped bool造许可，不签正常Stop。

普通native wrapper原有源核/释放语义保持，新带回执入口只在原四SQLite/OFD协调器显式释放、原最后完整镜像/选择仍通过且原pin已安全关闭之后签发nonserde/nonClone ReleasedCoordination。回执再核准确context/runtime/逻辑和实际根/四原file身份，所有rawpin先于新的SQLite连接、四范围同次BEGIN EXCLUSIVE可得及显式ROLLBACK/close/末端身份读回；不能由Drop、已有receipt、pid消失或复制布尔造出该类型。

实际MovementAuthority现仅由同原独立Index已登记计划、上述CurrentService与实际Guard/claim共同准入。原根同对象移动/Guard正常释放之后，还须完整OS/来源/四锁/根/Grok/保管receipt读回，才能形成专门Completed，并在原SQLite事务内和COMMIT后再核实际服务/全镜像/四范围/原当前head。独立终态为RootCancellationAbsent：原目录缺席和取消已核对，OriginalSession仍None，normalStop/通用Closed/后继资格均不授。只读重开、状态/前端标签已接，尚不开放新Start。

[结果与19文件/完整patch来源](../../.adg/evidence/0.2.24/mac-root-cancel-service-r1/result.json)：完整官方2.99/实际系统owner扫描/原Guard子进程/实际四锁的取消动作通过，原10000ms内**9102ms**，宿主b6e8c26479ae6d5857d3b87a7f2164ebe81e89848f5675abf3e321918daa1646原签名/字节不改。生成取消完成记录、关闭原Index后重开仍准确缺席，普通reserveStart/后继继续拒绝；不是整版candidate、生产UI入口或原服务正常Stop验收。约76秒整夹具含独立供包，不冒它在10秒内。

首native01成功场景Guard/移动/四锁均已到达且独立终态已经提交，但最终重复源扫描超过原10秒，按失败保原raw101、原数据库/receipt/来源/日志；未倒填为通过。优化只合并同一未变边界重复完整检查：close预检查只核原borrow/id/deadline，完整服务/全字节镜像/四锁核保留在SQL admission与COMMIT后；写DATA receipt后的中间边界用完整OS/record缺席与选择代次，下一SQL仍完整准入，不作跨动作缓存、不减字节/条目覆盖、不加时。

native01其余四场景已过并复用：移动前中断、移动后中断、实际额外同HOME Bun进程、移动后原路径出现新目录。中断/新根保现场、无错误terminal；额外进程在native初始化之前即拒，准确自有fixture子进程强制处置单列，不冒正常释放。后继只变成功收口函数/中间无副作用观察及runner文案，逐19源与前批核确认负分支未变，故不重跑四项。初Clippy等价map_or诊断原样保留，修等价谓词；最终strict lib/tests Clippy、7项定向检查、20项原owner回归、前端typecheck及87交互检查过；两专用native入口/一旧专用OS fixture默认ignored分别注明，不泛称全部原生通过。

6个本批native根、共享程序根、13个明确输出的单测根及自有进程组均缺席复核，约4MB证据保存，无重复运行包残留、日常配置/官方源修改、真实业务同步或模型调用。独立同HOME负例只杀本次新建的合成Bun fixture，未处置任何现有进程或Windows现场。

**余项未删减**：首次取消后的完整历史/后继root创建与捕获/状态同源资格、正式用户命令和确认入口尚待接；已部分Original捕获和其它代次不能复用本分支。Mac根未移中断/一般客户端图/缺Ready、NAME默认外部客户端与最终重启回退、Windows真实Owner整链/当前r2证明/最终候选与发布组验收仍未完成。Windows44aa85f6c／证据b2b25c26e已接（revision157），本次只改Mac/共用文件中的cfg-mac新增方法及前端标签，不碰Windows独占实现/四trait。最终不W2/双SID、不发布，目标active。


### 2026-11-17 revision159：首次取消用户入口与后继创建/捕获接线

main fcf04caecdd7c328f03fcf1a6ceda3e18517d9bc让RootCancellationAbsent作为有独立证明的后继前驱，与原RootAbsent分别验证；不假造Original refs或改普通Stop断言。谱系事件保持连续、原ticket/来源/head/容量上限不变，后继事务保同一action与实际新建根fd，原3代次索引闭环前后完整核。取消只读历史由其准确独立前缀/plan引用检查旧归档/源事件/receipt/原tree/保护/根身份及scope，结束再逐全证据复核；后继的当前.codex/Grok不会被拿来冒旧取消现场，无扫描发现收养，无目录丢失回退。

当前取消仍要求本代根完整/Grok与缺席；真正后继历史观察才不读后代当前客户端内容，同时原origin的namespace由已验证全索引/真实pending transfer提供。后代开始有新client-originals时，取消记录验证只容许其自身未有vault；其余vault成员仍按后继实际原件引用精确核，不能借取消豁免其它未知目录。旧保管/完成receipt中normalStop/successorEligible=false原字节不改；可启动资格来自新完整独立历史与当前事实验证，不重写旧DATA造权限。容量仍预留完整正常新代次9阶段，512代/4096事件/4MiB原上限保持。

原“取消启动准备”命令现按固定data的真实创建证据分派首次根动作，无新用户路径/PID参数。只读预览在同observer内生成created_root确认种类，完整空根/原history/Grok/绑定摘要与实际Index Review严格相同（Preparing/CaptureReserved两种源均核）；命令再取独立Index/当前service/actualGuard/claim/原程序，按原10秒完成两事件plan+terminal。原普通已有根取消仍走原分支；未知创建证据不fall back。页面同按钮显式确认，说明保管新建空目录并恢复原缺席、保留新内容、不自动启动，严格接受root_cancelled且revision+2；普通+1或混用status拒，最终实时Status未核对仍保Required，不凭回执直接开启Start。

[实际验证/15文件及完整patch](../../.adg/evidence/0.2.24/mac-root-cancel-successor-r1/result.json)：同原官方2.99、受限新Profile、实际配置Guard/四锁/完整OS和源核，从真实readonly preview拿摘要→先拒stale摘要→实际production root_cancellation::cancel→独立terminal，**9351ms/10000ms**；正常重开/status同源资格核后，新独立30秒动作实际创建后继空根→预留捕获→保存原件及原references→重开Captured **1680ms/30000ms**。后继scope只在明确测试helper里合成，不冒已启动官方服务或Source生命周期整版通过；原真实取消和后继存储捕获的证据层次分开。随后仅在该自有旧归档写入反例内容，实际重开拒绝，内容随raw fixture inventory留证。

原生宿主b8cc1e7c6227e3cdc7d22031c715cd7d6c36a05c95e1f58dea8bc7c889733dc2原字节/签名不改；约78秒夹具含供包，与原动作预算分开。7项首次准备检查、strict lib/tests Clippy通过；8项原Start/容量/状态同源回归、原谱系parser反例1项与旧归档漂移反例1项共10项同宿主通过，无重跑旧109全组；前端typecheck/90交互项过（显式确认、无自动动作、准确两事件receipt、错status/revision保Required）。中间Clippy对新测试helper多余借用的11诊断保原日志，修等价借用，无产品断言/预算变化。默认两native入口仍ignored，本批user-action scope显式执行其中实际当前服务入口，另一storage scope按前轮来源复用，不泛称两者全重跑。

本轮准确native根/进程组、3个受影响回归tmp根及13个明确输出单测根独立缺席复核，约0.96MB证据保留，无程序包残留/日常配置、Keychain、业务同步或模型调用。其它对话正在改文档核心/原型/协作包，均未纳本批提交，只提交IMP/TEAM和本轮.adg证据。

**剩余**：本批证明真实取消动作和后继文件/索引捕获，尚需最终Tauri原包的实际页面取消、后继官方service完整30/5/10/最终回退；其它代次准备取消或已有部分Original捕获仍不能复用首次空根分支。根未移中断/一般客户端范围/缺Ready、NAME默认外部客户端与最终重启回退、Windows完整产品Owner/privatehost/v2/当前r2及最终候选发布组门均未完成。Windows44aa85f6c/b2b25c26e本轮fetch无新增，继续其独占任务，不W2/双SID、不发布，目标active。


### 2026-11-17 revision160：Windows具体原worker驱动接收

Windowsfbac467cbf0b83472876f733d3fe5ec34f2582e8／证据379cde898741dc3b26484c0e390cb8c64ce839e9已净接收concrete-driver轮，140载荷1747888字节，全成员/逐字节长度SHA匹配，10个最终源码与Git原字节或明确CRLF归一逐项核；[接收回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r15/receive-result.json)。本机审查具体handoff/源码绑定，不冒Windows实际执行或整批安全审计。

具体canonical恢复callback已经原registry controlgate绑定唯一原window/StartFailure/原Owner→原worker贯通实际lease、准确action父pin、v1意图、quiesce、逐node原句柄return、明确释放、五范围probe和完整两树/8/bytes/children/Source/current/route/canonical/附件读回；private完成凭证先留原Owner再fallible复核。原锁方法/ReleasedLease保持private，错误仍保原worker/原槽，不改原10秒或旧v3超时事实。可调用具体driver不等可达版本化host、fulltrait或真实产品Owner已执行。

首两编译E0624分别因越private锁方法/ReleasedLease，改由原WorkerLocal内部完成而不加宽API；第三轮sealednative3509ms过但fixtureclose因返回pins仍存而失败，原24文件树/归档保留；第四轮STATUS_STACK_OVERFLOW原scene24文件保留。唯一原资源incoming/returned改Box堆所有权，未增stack/预算或加unsafeSend，activelease仍原worker；成功返回验证后join已无active资源worker、先关released再关fixture，失败parkedworker不join/unlock。

最终product05完整MSVC产品库raw0，一项原生集成含sealed/open实际自有worker pipeline分别3424/3535ms，各原10秒窗口内，包含真实原Job242/从未Resume/精确文件ID/所有原资源及claim线程绑定；没有真实产品Owner/官方Source准入成本，不能当完整官方整链预算通过。原worker七子case、origin五和Send一受影响回归过；EXE632dd4219b35dad9ad3f834b479d132a15924d565249bff9508fec16ebd164ce。最终2root/worker7root缺席，历史失败与新2失败root仍保留。

**新增清理缺口保留**：第四轮栈溢出后合成never-resumed-fixture.exe报告PID27960/parent14936仍在，只有CIM只读事实，原creation/nativeJob/scope处置证明未完。Windows须先准确只读身份/源/作用域和可审查计划，不凭PID猜杀/收养/Resume，不把其当原r2，不沿用旧50580/54692授权；本机未执行任何处置。该新残留不能写成已清理。

完整Owner官方普通Profile/原10秒、ReturnedOwner/proof独立验证及过期后只读历史、BeforeFirstResumeRecovery全trait/可达strict privatehost、独立v2准备与partial capture/work/缺根全故障覆盖仍未完成；当前r2Job24raw50仍Unknown，不重跑同前提PSS/旧cleanup探Status。Windows可持续原独占开发，TEAM28/r7写集与接口不变，不等Mac新ABI。Mac当前fcf04原包构建/实屏与官方后继验收进行中，最终候选仍未固定，不W2/双SID、不发布，目标active。


### 2026-11-17 revision161：Windows独立v2严格链与真实准备DATA接收

Windowsdd42d5e3871b71f860a5149bd22ea0166fd717b1（前笔a1470004b）／docsc17b031f1ad5e447ecedd40e7da91c816fdcfbd7已净接收v2-reference轮，88载荷1290956字节，完整成员/长度/原SHA核，14最终源逐Git原字节或明确CRLF归一核；[回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r16/receive-result.json)。本机只作来源/具体handoff审查，非本机Windows执行或整批安全审计。

独立V2 strict Record/Reference/Source/Component/Object/Event/chain按r7 A，拒缺字段/未知/隐式null/旧v1-v3/fallback、错前驱、覆盖缺失、replay、悬挂意图和Pending假终态。原capture在source观察前保唯一原session/host UUID，PartialCaptureAnchor从实际source/routes/pins/创建阶段/原两树与pending原件/journal成员投影保守DATA；只有实际create成功记NativeCreated，失败/过期保持IntentPublished，实际保原句柄才HandleRetained。原ID失败留raw/kind，安全/content未核仍null/Pending，DATA不造Owner/claim或恢复许可。

最终9strict DATA＋3实际native测试通过（product01/03/native04各原日志保留），旧abort4/partialcapture3/Send1定向回归过；最终EXE d16a5697f6b441900b6423eee7a76faafb6b35ca4cc91e3534f0f3714993a014为测试宿主，非候选。8新fixture准确缺席；旧partialregression只有Drop不泛称全历史清理。两源码采集断言错误如实留，不冒产品首错/独立安全审计。

96项状态表仍明确：真实产品Owner完整driver与slot返回独立验证/过期后只读status、fulltrait/版本化可达privatecontrol、v2全native准备/partialcapture/work/缺根仍未完，普通Profile固定官方整链原预算未过；synthetic27960及旧失败清理缺口保持。当前r2/旧40588与原件/协调/seq9不变，raw50对应依赖Unknown，无新R1执行批准，不重跑同前提PSS/旧cleanup探Status。TEAM28/r7独占集不变，Windows无需等Mac新ABI继续；本机正在复验真实取消界面后的状态刷新问题，未固定整版候选，不W2/双SID、不发布，目标active。


### 2026-11-18 revision162：实际打包取消与官方后继启停、状态等待修复

本批真实产品源45571c2b83024425bd7d019d3a4270b5dc73bb4e（修复80fcd4d2d），后继81f52452cb4cfafc64a80ca92d97a3ba5990c2fb仅实屏测试重查口径及精简终端摘要；不冒后继test SHA就是原包编译源。[全证据/结果](../../.adg/evidence/0.2.24/mac-root-cancel-integrated-r1/result.json)。原fcf04包先构建299167ms，新45571包258881ms，三sealed bin逐同轮Cargo产物签名归一副本全字节绑定且原件不改；新Desktop原SHA351807b2cf36227d04255510f581dee2f066c1b03dc96a291a2c05e759c76f48，原签名/内置官方2.99源不变。非最终Windows集成候选、updater签名/正式发布验收。

新集成测试seeder仅在准确新Profile用产品Index::prepare_start_capture生成真实Preparing1，无JSON拼receipt/伪造source/owner；实际应用先生产startup完整离线供包→退出→seeder登记→原signed副本重开→诊断/检查客户端恢复→明确确认。默认自动同步/更新/启动禁，OS sandbox拒日常HOME/Keychain/公网，只准loopback；合成provider固定127.0.0.1:1/无live发现/无模型请求。不触日常配置，窗口观察只遍历目标pid/准确exe，不读系统菜单。

首gui01原fcf04包已真实提交RootCancellationAbsent，但界面在取消后refresh等待期间保持executing，30秒观察失败；原AX/完整索引/来源/stream/截图保留。源码确认共享get_status_snapshot_with_collector原Mutex::lock可能把轮询/收尾刷新排队，且页面准确receipt后仍await刷新才退出executing；未采原live stack，不冒精确单次队列等时已证。修复：采样锁忙立即返回status_sample_in_progress，不开始第二采样/不报旧成功；页面准确receipt后结束mutation展示完成，独立refresh期间仍保原生命周期Required直到有效fresh snapshot，错误/重复点击不重放动作。4项snapshot（含实际持锁争用）与91前端交互（含延迟状态完成/保门槛）通过，strict lib+两实际integration宿主Clippy过，不改原30/5/10或状态断言。

第二gui02新包在动作前status准入遇忙，正确零变更拒绝，原Preparing1不变；测试未处理明确可重试状态，30秒等待失败，原源/证据保留。测试后继只在精确原Preparing仍相同且明确busy-before-mutation时最多4次重走“重新检查→取消→确认”，不是产品后台重试或权限绕过。实际gui03最终通过；同一旧seedHost绑定原fcf04+准确新测试源，45571对根创建/index/取消模块无净差，未假称seeder也由新source重新编译。

**gui03实际结果**：取消确认后页面显示“原目录缺席已恢复”，独立Head Preparing1→RootCancellationAbsent3、原逻辑.codex缺席；页面观察约6886ms包括AX轮询，严格动作预算由原后端检查，不能拿观察时间冒精确IPC性能。随后在同真实应用点击Start，原样官方服务pid46175/port54516真实Ready、后继ticket独立/refs完整，索引Ready7；再点击正常Stop，日志确认原service native exit code0、原launcher正常exit code0，捕获4个TCP监听全缺席，最终RootAbsent12/完成digest/原目录缺席。此为实际官方后继整链，与revision159合成scope捕获明确区别。

最后Stop时AX仍在等迟到状态投影，测试以完整canonical RootAbsent和生产正常退出/监听日志确认后结束，不冒已看见任务卡消失。独立核macOS proc_pidinfo对service及两个自有GUI pid均ESRCH，内核TCP全表framing复核导出的主port缺席；生产Stop另核全部4端口，不用ps/nc空值冒原全部监听证明。原GUI进程组退出-15只针对已经正常Stop后应用与首无服务进程，无force官方service。

三轮准确测试根/自有进程组均清，三条本次digest协调器每轮排他重新取得、原bytes/base64/digest留证后删除，preexisting共享全局L未删除/重建；本轮未单独记录其前后bytes，故不冒完整原共享L字节保持证明。原app/副本sealed字节未改。约24MB证据含原失败两轮/三截图与AX/同源构建，不存重复运行包。初seed格式检查raw1保原输出，格式化后build02 raw0，未更改产品源；最终runner新摘要不再终端重复base64。

**未完保留**：最终Tauri/Windows完整集成候选与签名/安装/升级回退发行门；取消后重复代次/部分捕获中断的其它分支、一般客户端图/缺Ready、NAME默认外部客户端与最终重启回退；最后Stop状态任务卡投影未等到收口。本批实际取消及官方后继Start/Ready/Stop通过不替代上述范围。Windowsdd42d5e/c17b031f1新v2回交已归revision161，完整Owner/v2/privatehost/普通Profile/当前r2及synthetic27960清理仍未闭合；本机不触其独占文件或旧现场。目标active，不W2/双SID、不发布。


### 2026-11-18 revision163：Windows独立V2原生持久化接收

Windowsc3b99c0fa704d278393bf9f854eec4de6968ba4c／docs954b4540c9bb705edbbbdd4990e8be82df61a402净接收v2-durability轮，29载荷678509字节，完整成员/逐原长度SHA匹配，4执行源逐Git原字节或明确CRLF归一核；[接收回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r17/receive-result.json)。本机核原数据与具体handoff，非Windows实际执行或整批安全审计。

独立V2记录命名/strict前驱与候选全链在CREATE_NEW前验证；实际新文件/目录先入原保留槽再作fallible保护/ID/bytes/ADS/完整members前后核。失败留原资源/部分前缀，不读旧目录收养、不重试/重开window；不调用v1 decoder或转换旧记录。该持久化仍只是DATA，不构造Owner/claim/quiescent/恢复许可，不能替代原生准备producer及完整partial恢复。

真实MSVC全Tauri库product01 6过/raw0/145459ms（native tests0.98秒），26新目录覆盖完整canonical DATA链、7链/引用/版本写前拒绝、5原生中断点、4过期/截止替换/非空拒绝、6实际member/SD/ADS/晚变化及3完整声明DATA/Pending/未知终态拒绝。完整声明ID为明确合成DATA，不冒native原件集合或产品Owner。EXE4b5e372f7c50b3b4d3be904a725d2d0b2bb371ee890bb3475d7fb0690ccdebea；26新root缺席，旧strict9/native准备3及无影响CLI/MSI/空GUI/归档复用不重复计。原112warning与工具/文档merge操作错误如实留，不冒新产品首错。

完整103项本线状态仍保真实Owner returned对象/完成proof原slot复验、截止后只读status/history、fulltrait/可达privatehost、全nativeV2准备及pre-manifest/partialcapture/work/缺根恢复、普通Profile原官方整链和预算未完。当前r2raw50依赖、synthetic27960清理和旧拒绝删除目录/失败源保持；不PSS重复同前提、不cleanup探Status、不原现场forced/归还或新原包Start。TEAM29/r7允许集与共享接口未变，Windows持续独占实现；Mac正在补停止进度卡实屏观察，不改已通过native断言，不W2/双SID、不发布，目标active。


### 2026-11-18 revision164：停止完成任务卡实屏自动收起补验

本批main0675a105017ae7fc82a4bd58dd43fe72031470df仅扩实屏验证器两种明确flow，**产品仍为45571c2b83024425bd7d019d3a4270b5dc73bb4e/原sealed Desktop351807b2cf36227d04255510f581dee2f066c1b03dc96a291a2c05e759c76f48**。stop-progress单独做实际正常Start/Ready/Stop，不重跑已过首次取消；既有源码和倒计时/超时断言未改。[结果与完整来源](../../.adg/evidence/0.2.24/mac-stop-progress-native-r1/result.json)。

先对照原任务卡store及NotificationCountdown：正常Stop准确回执会结束执行态，成功倒计时按既定规则在悬停/键盘焦点/隐藏时暂停。此前在RootAbsent刚提交即停止观察，不能据此推产品任务卡不收口。本轮gui01在供包结束后的UI可操作观察超时，尚未点击Start；实际原候选仍活，按准确PID/exe原生重观测按钮已可用，接同一进程继续，未重新供包/重启。same-scene continuation真实正常Stop/完成卡已见，但AXPress导航＋mouseMoved不足以证明所有暂停原因已解除，自动收起观察失败；原完整summary/AX/stream保留。

gui02在真实Ready后，旧AX路径上的按钮标签发生变化，原native helper在AXPress之前明确返回7/expected label mismatch，未执行该点击；该轮正常Stop清理成功，未按失败强制杀官方服务。验证器只对这种准确“未点击”拒绝重新观察同PID/原exe一次，不重放已准入动作，其它错误保持失败。只读pre-action UI readiness等待与内置供包分开，120秒是尚未开始任何生命周期的页面观察窗口，原Start30/Status5/Stop10保持。

最终gui03同原包、内核拒日常HOME/Keychain/公网、只loopback/合成provider，无模型或业务同步请求：实际Start→Ready→正常Stop→原RootAbsent→真实AX“停止已完成”→原生activate并点击现有概览导航，使键盘焦点及指针确实离开任务卡→倒计时自动收起，未点击卡上的关闭/收起按钮。末屏[截图](../../.adg/evidence/0.2.24/mac-stop-progress-native-r1/gui03/stop-card-dismissed.png)可见概览导航焦点、未运行/启动入口且任务卡已消失；原步骤AX与命令全部留存。22项既有notification-countdown/process-action测试通过，确认不因实屏补验改暂停规则。

独立内核复核最终服务与GUI PID均proc_pidinfo原返回0/ESRCH，导出主端口在合法有界TCP内核表缺席；产品日志另有原服务/launcher正常code0及捕获4监听全部消失，清理不以tasklist/ps成功推断服务停止。三个任务专用协调文件独占可重获、原字节/身份留档后删除，所有准确本轮Profile/appcopy及进程组缺席；初gui01原报告retained状态不回填，后续same-scene清理与独立复核另记。无日常配置或原signedapp变化。

**证据错误如实保留**：既有gui02已结束后，一次准备重跑在exclusive stdout创建处发现文件已存在、并未再次启动；该次准备此前误覆盖hypotheses02.json/runner02.py两文件。现明确标为事后分析/来源副本，beforeRun=false，不用于证明gui02原执行源码/事前假设，原gui02命令/AX/streams仍完整；最终gui03唯一新目录的runner03.py/hash/事前假设与真实native输出独立可核。原始gui01/continued/gui02失败均不覆盖、不当产品失败或已通过。约28MB证据，不保重复运行包。

此项补齐revision162“最终停止任务卡自动收起未观察”缺口，相关标记以后以本轮实际证据为准，不改旧结果。仍未完：Mac其它代次取消/部分捕获/根未移恢复/一般客户端范围/缺Ready，NAME默认外部客户端及最终回退，Windows完整Owner/slot/v2/privatehost/当前r2与synthetic27960清理、最终集成候选及发行签名/安装/升级回退门。Windowsc3b99c0fa/954b4540c回交已归revision163，本批只test脚本、不改其独占写集或共享产品接口，不要求无影响全组重跑，不W2/双SID、不发布，目标active。


### 2026-11-18 revision165：Windows独立返回复核与合成残留只读接收

Windows361cdedf032b20e57b7ca2421daf5ddf3044d631／docsf8552e56cc4ea0ddb8aaab143bc917e9a7d202df净接收synthetic-readonly及return-validation两轮，分别7载荷11629字节、123载荷1414795字节，完整成员/原长度SHA匹配；返回线13执行源逐Git原字节或明确CRLF归一核，[回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r18/receive-result.json)。本机只审来源/具体handoff，非Windows实机执行或整批安全审计。

原WorkerControl成功返回槽先保唯一Box原Owner、同原window与ReleasedLease再fallible核：前后transferbinding、registrySlot runtimeArc/安装根/HOME/Profile/hostUUID、原quiescence/private完成proof、adapter两次原树8/bytes/children与Source/route/canonical/附件读回相同。失败保原对象/revoke/原control占位，不重试；ReturnedValidated仍Required/runtimeAuthorityfalse，不造正常Stop/Closed/后继。截止后完整只读status/history仍未完，不把同窗口readback当永久权限。

首编译private测试入口与重复drop错误、后原workerfixture未保护source正确拒绝均原样留，后者新.tmpJEMT1O四文件readonly归档保留；只修新fixture私有子目录、不放宽原断言。最终MSVC全Tauri库一父四worker模式valid/raw5/wrongcutoff/expired通过；原树六模式在前轮已通过且源码未变复用，worker七子/释放十场景/concrete两pipeline及Send受影响回归过。最终EXEfb6c54fe2a79ca14fb9d92c9dd2a24b0d6fd950e557bde4451c58ac51a5cefc0；29成功root缺席，1新失败树及所有旧拒删/driver失败/残留保持，不冒历史全清。

synthetic27960新只读证据用QUERY_LIMITED_INFORMATION取得实际image、创建FILETIME134394012184503726、同句柄前后代次不变和任务文件SHA与旧ZIP一致，inSomeJob=true、parent14936/session2/exit259；这只确认当前进程身份，原Job完整成员和monotonic neverResume仍Unknown。处置计划ready=false，当前不请求执行批准，不凭PID/path/hash/当前Suspended补原creator能力；本机不终止/Resume/收养。此残留与r2/session7不同，旧清理授权不转用。

111项表仍保完整nativeV2准备producer/partialwork-capture/缺根、可达strictprivatehost/fulltrait、截止后status、真实Owner固定官方普通Profile整链原预算及容量/CAPtrust未完。当前r2raw50只停自身证明，21500/75744/原件/work/seq9与40588不变，不重跑PSS/旧cleanup，不forced/归还或原包新Start。TEAM29/r7写集不变，Windows继续原独占任务，Mac发行校验独立推进，不W2/双SID、不发布，目标active。


### 2026-11-18 revision166：Mac签名更新归档与完整封装内容绑定

mainfb755b8ea63d8a3fc9146dc4db491931919905b0新增verify_macos_updater_024.py并接CI/Release Mac装配。信任输入是明确checkout/tagged tauri.conf版本/identifier/updater公钥和固定官方平台policy；先真实minisign验detached base64签名，再真实codesign验证sealed app，流式读取整个.app.tar.gz而不extract或执行，把全部成员/字节SHA/模式/目录/安全链接与同轮sealed app精确相等；再复用原delivery核内置平台archive/descriptor/pin/原官方tgz逐文件与productionlock。验前后原签名app完整tree/xattrs和archive/signature/config原identity/stat/hash不变。输出始终releaseAcceptance=false，不能由这一部件回执签整版通过。

归档完整性包含单app准确根、精确目录父成员、去重/类型/模式、拒hardlink/设备/特殊文件、越界链接、额外成员/丢成员、PAX xattr overlay/未知metadata、global header和尾随第二归档。压缩文件上限1GiB、未压缩普通内容2GiB及100000成员；TarInfo在标准库处理longname/link/PAX正文前截断声明大小至16KiB，rawheader计数和整个解压流也有界，不能靠迭代前扩展头先申请大内存。读取固定块，不从归档磁盘执行源码或隐式提取文件。

[精确源码/验证](../../.adg/evidence/0.2.24/macos-updater-archive-r1/result.json)：最终9测试过，用真实minisign临时key做有效签名、错公钥/错误签名，并对内容/模式/丢失/额外/恶意成员/官方源码自洽篡改/隐藏trailer重新签名，确保有效签名不掩篡改；16GiB声明扩展头在读取正文前拒。Linux synthetic fixture仅codesign调用明确mock，完整crypto仍真实，不冒Mac签名测试。7项原delivery与30项publisher回归过，共46项；原publisher对>=0.2.24的拒绝门和旧版维护包合同未修改。两workflow经Ruby YAML.parse_file语法通过，未发起GitHub或发布任务。

真实本机原sealed45571包（Desktop351807b2cf36227d04255510f581dee2f066c1b03dc96a291a2c05e759c76f48）生成符合updater布局的GNU tar，仅用新临时测试key替换副本config公钥签名，实际系统codesign和最终有界parser完整过：outer19成员/289224644内容字节，压缩276791819字节/SHA7567162c1806982cfae8ab6d7cf8b3b385bd7cb0fd00988e847b161e250c5095；内置11467成员/633363932字节、2664原官方源码文件、102锁定包准确，官方源b920ba…不变。最终实测7204ms为离线校验/构造成本，非原生生命周期预算。两次自有私有根/临时签名私钥/约277MB测试archive全部删除并缺席核，保公钥/签名/config和摘要约47KB证据，无日常配置或源app变化。

**证据范围**：本地archive是从真实signed app新生成的tar，使用测试key，不是最终Tauri产出的正式updater或发行key实测。Release新步骤在实际tauri-action完成后、同源Mac binary绑定后要求恰好一个真实*.app.tar.gz及其.sig，用tagged原公钥执行同脚本，并上传macos-updater-delivery.json；缺包/签名/内容不等则失败，不改DMG或updater字节。只有以后真实固定候选CI过才计这一生产装配已验，不能以本机测试代签名发行验收。

Windows MSI/NSIS内容抽取/实际封装资源绑定及正式公共制品核验尚需完成；本机已发现7zz可用，但未以未证解析或模拟安装结果草率放行。publish_release.py整版硬拒绝继续保留，不把Mac校验接线当全部发行核验完成。Windows361cdedf0/f8552e56c已接revision165，完整产品Owner/privatehost/nativeV2/当前r2和synthetic27960处置依赖继续，Mac其它既列功能/NAME最终客户端/候选回退不删。目标active，不W2/双SID、不发布。


### 2026-11-18 revision167：Windows原生历史只读基础边界接收

Windowsf8feb3825a4f2194e4c5081a29a760d446473f61／docs801e53cc888ea73a5c9a72e19485371a64cd38ad的history-primitives轮净接收，100载荷1267628字节，完整成员/原长度与SHA相符；9执行源逐Git原字节或明确CRLF归一核，[回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r19/receive-result.json)。本机核原handoff/来源，不冒Windows实际执行或整批源码安全审计。

真实原QuiescentLauncher保存quiesce完成Instant在旧deadline之前；原截止后只读借原privateproof/creator/Job/绑定核total1active0/root242，原verify/Resume/reprepare仍拒旧过期，不靠DATA补权限。V1/V2 native storage保存真实完整append读回完成Instant；历史读取在新caller读取截止内只核原files/ID/所含8/bytes/ADS/members/严格原链，缺完成、partial/漂移/过期拒；旧append/read权限窗口不变，不从磁盘收养。此为原Job和原记录两基础只读能力，尚非完整产品Status或后继许可。

首编译测试模块path错误保原raw；后Tauri库12附件历史场景通过，但两Job fixture漏建私有H在CreateJob/Process之前被拒，22文件两个失败树精确只读ZIP/集合/hash前后核后保源；仅补fixture H。最终MSVC原Job按原1秒parent内242完成，旧verify过期仍TimedOut，新读回核同原Job，两个交叉proof各拒、各自原proof过/原marker未执行；附件12模式源码不变复用前轮，原V1/V2 storage/claimquiesce/完整workerpipeline13受影响回归过。EXE8ee6ad6e2d167d8fdb9bc42e56709571c9386468c10fa65b7c8bff9d43ea58f3，65成功root缺席；两个新失败树与所有旧拒删/driver失败/synthetic27960保持，不泛称历史全清。

119项本线状态仍待原Slot及时完成事实、原core/Source/route/canonical/两树/附件/privateproof汇成同一次Status5秒、strict可达privatehost/fulltrait、nativeV2partial/缺根、真实Owner普通Profile固定官方整链原预算和容量/CAPtrust/最终候选。当前r2raw50只停自身证明，不PSS重复、不旧cleanup探Status、不触21500/75744/原件/seq9/40588；旧场景清理授权不转用。TEAM30/r7独占集和共享接口保持，Windows继续本线；Mac仅只读研究Windows封装格式，不安装或运行旧installer，不W2/双SID、不发布，目标active。


### 2026-11-18 revision168：Windows签名安装器内嵌内容绑定与只读探测

main4ea82e8a61fe00c1f76334db5e39654f49f4342e新增verify_windows_installer_024.py及CI/Release接线，不改Windows独占产品/安装模板/四trait。显式当前tagged config版本/identity/updater公钥先真实minisign校验installer.sig，再完整准入当前Windows stage官方archive/descriptor/pin/原tgz/productionlock，读取installer的实际内嵌成员，分别绑定3个当前构建程序、native-entry和4运行包资源，共8项原长度/完整SHA，首尾archive/signature/config/全部输入和读取器原对象/字节复核，防止途中换包或换build输入。

MSI只允许当前固定Compound→app.cab→Cab结构及模板声明的准确File IDs；NSIS要求实际Nsis容器与完整logical路径，不泛匹配文件名。完整有界technical listing拒重复key、路径/case别名、遍历/控制字符、link/加密/未知大小（唯一NSIS生成uninstall.exe大小例外、不作为目标资源）、多容器/过大成员。实际7zip x -so -spd输出直接有界hash，不向磁盘解压；两pipe同步drain、stdout按准确声明长度、stderr1MiB、listing8MiB、成员10000/总2GiB和每进程120秒上限，超限或时间到只结束本次原读取器child，不执行installer/脚本，不覆盖用户文件。读取器本身固定显式路径，前后文件身份/hash绑定。

**不是安装布局证明**：MSI CAB的File ID能证明准确内嵌bytes，不能代Directory/Component table/native costing；NSIS文件内容不能代安装脚本实际目标/注册/卸载行为。输出明确installationDestinationVerified/nativeInstallUninstallVerified/releaseAcceptance=false，原真实Windows安装验收依旧必需。没有把安装成功tasklist或单hash当完整安全范围。

[离线验证](../../.adg/evidence/0.2.24/windows-installer-content-r1/result.json)：7新项含真实临时minisign、两容器明确模拟回复、8成员逐missing/篡改/大小/输入漂移、listing framing/特殊路径/类别拒绝，实际子进程两pipe有界输出/溢出/timeout/非零；9Mac updater＋7runtime delivery＋30publisher回归全部过，共53项。首unit01 NSIS最后空Size字段被通用strip误删空格，原raw1保留；仅改trim换行后unit02/final过，未放宽空Size一般规则。两workflow经Ruby YAML解析通过，PythonAST过；本机无PowerShell，未声称实际Windows装配步骤已执行。

真实容器探测用原发布0.2.22 MSI7467008字节/SHA204a9a…及NSIS5323941字节/SHA0b8246…原GitHub资产digest核后，只读7zz listing及主exe流，MSI实际Path ID、NSIS完整opencodex-desktop.exe，各15296000字节；新最终有界读取器与初独立探测摘要一致。旧两个exe实际SHA不同如实各存，未推它们是同一build或当前0.2.24包，不借旧版本证明新包合格。原完整命令/列表/摘要见[只读证据](../../.adg/evidence/0.2.24/windows-distribution-probe-r1/final-bounded-reader.json)。未运行、安装、解压写目录或改注册表。

最初尝试原0.2.23两包约580MB，读取速度低且格式研究不必大包，明确只对已核父子关系/准确命令的自有gh child发SIGTERM，不重试同一download；原raw-15/部分文件长度hash留证后改小022。最终4个旧资产/部分download共38569758字节全部从准确任务根清除、原根缺席核，保约47KB metadata/source/test证据，无重复安装器残留。中断不冒网络错误自然终止或下载成功。

Release Windows新步骤在实际tauri签名出包后各自要求唯一MSI/NSIS及.sig，用checkout公钥和实际build资源调用同脚本，两个kind均记passed/firstError再统一阻断并完整上传结果；读取工具从已安装7z.exe显式定位，minisign只在缺失时准备。实际候选尚未运行此步骤，当前只有工具/装配和旧读取法验证，不冒Windows0.2.24真实签名内容或安装路径/正常用户验收已过。publish_release.py>=024硬拒绝保持，公开制品最终输入/正式签名及整组发行验证尚待候选闭合，不提前发布。

Windowsf8feb3825/801e53cc8原生历史回交已接revision167，本批不改变其Owner/slot/nativeV2/privatehost/当前r2和synthetic27960清理依赖。Mac其它既列取消/中断/一般图/缺Ready、NAME最终外部客户端/回退和最终集成候选完整验收仍未完；不因发行校验增量缩小版本范围。目标active，不W2/双SID、不发布。


### 2026-11-18 revision169：Windows三阶段canonical trait与独立私有wire接收

Windows58f0d49f2b45ad108cf921ef04fae32f9980f133（含1f940af6c39ab38805c12e301f6b096c3ed74cbb）／docs0d6fa9c0dae16eb37327e8738ee2de0064dd549c净接收canonical-trait/private-wire两轮，97载荷850487字节、31载荷660933字节，全成员/原长度SHA核，5/4最终源逐Git或明确CRLF归一核；[接收回执](../../.adg/evidence/0.2.24/windows-r7-contract-receive-r21/receive-result.json)。本机只审来源与具体handoff，非Windows执行或全树安全审计。

WorkerLocal已实现原BeforeFirstResumeRecovery prepare→quiesce→restore，BoxAttempt先保唯一原session/action/scope/end/thread/nonce，原prefix/quiescent/tag在原槽先留再fallible核，claim borrow跨阶段释放但底层同lease claim/N/L/K连续持有。private非Clone/非serde关联tag只定位原native阶段，不能从record产权限；原失败revoke/worker保资源，不新window/unsafeSend。首native测试误断言primitive须丢弃已drained lease壳，原4范围释放/readback其实已过；保4file失败scene，只改新fixture检查真实4range可独占/原失活lease拒claim/releasedcheck，最终native过，两个metadata/type测试未变复用。12受影响测试过，20新成功root缺席；真实产品Owner官方整链未跑，不能用test trait通过开Start。

新windows-before-resume-private-control-v2是独立严格DATA codec，只支持带明确确认的begin-canonical-abort和两只读操作。所有字段/嵌套budget必需，拒旧v1/unknown/duplicate/任意PID-path-handle-proof/超长与尾随frame；原QPC扣耗时，Begin10秒/read5秒，instance/generation/challenge/scope和3类单次keys上限16384。transportActionId不冒真实Owner recoveryAction，locator不授nativepeer身份。6项DATA测试真实完整MSVC库过，未创建实际endpoint/Owner；格式转义准备错误及未生成script path错误原样单列，非产品运行失败。尚需protectedlocator/pipe、双端ordinary/noJob/sameimage/token认证和首次变更前实际可达dispatch，不把codec当可达能力。

最终两EXE6c94be0ddb710f3dd0c22290a3de3a22dbb924cba5e146800d666119be721d00、aa94e650a57fe06953b8586c78120c9627b7e88228f2c74b72e1784ddecdca57及所有raw/source/清理原件保留。141项状态仍保nativeV2partial/pre-manifest/缺根、真实普通Profile固定官方Owner整链原预算、容量/CAPtrust/普通MSI/最终候选未完；r2raw50及synthetic27960清理只停相应依赖，不重复query/旧cleanup或触原21500/75744/原件/seq9/40588。TEAM31/r7独占与四ABI不变，Windows可连续接线，不等Mac新接口；不W2/双SID、不发布，目标active。


### 2026-11-18 revision170：首次新根部分捕获的保留取消与后继存储链

main346b46a69b1d9951335359a334cda7bf21ddd132扩首次目录取消，读取原CaptureReserved的唯一ticket保管槽并绑定准确目录身份/Protection/全文件bytes与成员投影；它是部分内容保全DATA，不是OriginalSession、完整frame/scope或来源许可。Preparing仍要求无任何未预留vault，CaptureReserved只接受原预留槽/空vault的准确状态，未知另槽/链接/不可读/后续变化一律拒。新review-domain/plan schema2明确包含capture_vault，严格无v1fallback或迁移，不收养磁盘Original来补引用。

移动前后、独立计划/取消完成与历史后继均复核原部分槽。当前取消期间完整vault namespace必须相同；后继有新Original时，由全谱系分别期待旧取消保留slot和新真实attempt集合，历史只读仍两次验证旧部分内容，不拿后代当前树冒旧来源。未到Captured的published Original文件只保留字节，绝不读取它生成refs；普通已有目录取消/正常原件Stop仍走原分支。功能合同见[首次部分捕获取消](0.2.24-contracts/client-first-root-partial-cancel-r1.md)。

[结果与准确源码/patch](../../.adg/evidence/0.2.24/mac-first-root-partial-cancel-r1/result.json)：原样官方2.99＋实际OS owner/Guard/四锁准入，在真实StartupSession::begin_reserved四实际写入checkpoint注入中断：directory_created/codex_archived/grok_archived/original_published。测试仅cfgtest准确新/private/tmp根可注入，私有线程槽确认实际到达point，不能用泛Err让早错冒反例。原捕获30秒窗口结束后新显式取消独立10秒；四取消9488/8888/8957/9306ms，原预算均过。错摘要先拒且原head保留，确认后原CaptureReserved2→RootCancellationPrepared3→RootCancellationAbsent4，无Original refs，原目录缺席，实际四协调范围释放核。

同四场景随后另一次新30秒动作，实际后继目录创建/预留/原件存储捕获2736/2307/3089/3060ms，关闭后重开准确新Captured；旧部分槽前后全snapshot相同，随后只在自有旧槽加入合成反例字节，实际重开拒。该后继capture scope用已明确storage test合成，不冒官方服务后继启动/模型调用；本轮未Start任何服务。native hostafa8e44994045d68ab6007e909cb1fa2693224b13a33d1cfd95b2a7042c8cc9e原签名/字节不变，14最终源中Rust产品/测试源与实际编译回执原字节相同。

9项准备检查、原vault1/Start8/三个精确普通取消预约回归共12项受影响回归通过（后端去重共21）；首次ordinary filter只匹配manual ignored1/0pass原日志留，不计通过，之后准确三真实用例补跑。严格lib/testsClippy过；前端typecheck/94交互通过，新增created_root_capture只准CaptureReserved、created_root只准Preparing，匹配后才接受root_cancelled＋revision+2，wrongkind/phase拒。确认文案明示初始化记录/部分文件保留、不作为新的原件，不自动启动。最终UI仅在native执行后增强kind/phase拒绝并重测，非Rust动作变更，不重跑已过四场景。

初两个编译检查E0308因路径变量被pinned profile遮蔽，原raw保；移回未遮蔽边界，没有改变安全断言。新byte变更反例第一次对产品readonly部分文件写失败，按失败留原log；仅在合成fixture临时开写再还原原mode以测试bytes漂移，产品保护不改。四全部诊断场景一轮continue收集，无首错逐项重跑，供包一次且每场景完整重新准入；80秒第一场景含供应/构建，非10秒动作结论。

清理复核发现原smoke只清task Profile/program，遗漏外部每task-key的N/K协调DB；本轮四原Profile与创建intent严格对应的8文件已准确前后inode/uid/mode读取、SQLite EXCLUSIVE可取得并释放、原bytes/base64/SHA先留后删、独立缺席复核；不存在的H保持缺席，**全局共享L未选中/未修改**。原native01没有外部DB运行前缺席回执，不反填假证；精确唯一fresh Profile来源/当前identity与释放事实分别记。验证器后继加preflight exact key absence和结束后准确清理，不泛扫全局目录。5实际清理helper场景验证released成功/busy独立原子进程持锁/sidecar/symlink/错误mode拒绝，不重跑四产品动作。4测试根/共享program/18准确打印新单测根已缺席核，合成部分blob原字节及反例保留后清，证据约2.65MB不留重复程序包。

**未完**：本次新partial分支最终signed app真实界面→IPC→后继官方service启停、唯一原型新文案/状态对齐（原型文件正在其它对话工作，未并发写）；后继代次Preparing取消与已有plan中断的显式恢复仍独立未完，不能复用首次限定。一般客户端图/缺Ready、NAME外部客户端最终显示与回退、发行正式签名/封装门真实CI、Windows完整Owner/privatehost/nativeV2/当前r2和synthetic27960等原范围保持。代码首次push临时SSH断连raw128，原commit未重建，重试push成功后才记远端已接；不掩网络错误。

Windows58f0d49f2/0d6fa9c0d两回交已归revision169；本批只Mac原件/状态UI和smoke、无Windows独占路径/共享4trait变化，Windows可自行合入继续，候选未冻结，不W2/双SID、不发布，目标active。


### 2026-11-18 revision171：部分捕获原包实屏诊断与两处界面时序修复

本批工作止于发布前，**部分捕获GUI整链仍未验收通过**。[本轮结果](../../.adg/evidence/0.2.24/mac-first-root-partial-gui-r1/result.json)明确wholeGuiFlowAccepted/releaseAcceptance=false。产品源依次ebe80da860551c85c1ec53f1d57d0c8e2b0264b1、ea7e258bafcabc8128d98afa976fdba21cef9f4c、bb13a7f12570bff10cb117fabdeefcf1b30d27d5，三次Tauri app构建均实际完成并将包内三个程序绑定同轮Cargo回执；新sealed Desktop分别574b6269…、b9b5f207…、a37ea6e8…，完整摘要见原binding。当前main88b8fc33d5ec21b54c494cdb529fd68026b3f8b9仅后继测试窗口激活，不冒它是原包编译源。

新seeder先经真实产品Index创建/预留首次根，再在唯一ticket槽放明确synthetic-unpublished-fragment.json只读合成DATA；没有伪造Original/来源/Stop/完成权限。四真实capture中断点仍沿revision170原生证据，本轮只验证实际打包界面/IPC接线。helper与新包Rust root后端无净差，原source/hash和seed Cargo回执分开绑定。隔离新/private/tmp Profile、禁日常HOME/Keychain/公网、只loopback、无live模型/业务同步，无日常配置修改。

四轮结果分开保留：

- gui01：actual CaptureReserved2→GUI专用确认→RootCancellationAbsent4，片段原字节/ID/mode保留。后续状态门槛仍Required，未点击Start；同一原进程只读复核已关闭根仍无协调入口。保留现场后取完整原索引/片段/AX，再按准确无Start事实退出原测试GUI和清任务协调DB；原summary retained字段不回填，后续实际cleanup另证。
- gui02：重建后只读协调成功，官方后继达到Ready8。第一次Stop请求失败，测试等待45秒无完整RootAbsent；后来的failure cleanup在record消失时过早终止GUI组/删fixture，原canonical实际StopPrepared9。**不能将该cleanup当完整正常Stop/恢复已过**。原日志虽有service正常exit0及捕获4监听缺席，launcher/最终原件恢复未证；所有raw manager/frame/片段和原日志保留，失败不改通过。
- gui03：最新原包供包成功，但AX只有8节点/无WebArea，观察超时。实际截图页面完整渲染、原进程sample为NSApplication事件循环，因此是辅助功能观察问题，不能记“WebView空白/产品页面未加载”。未创建预约、取消或服务，准确root已清。截图/实际只读线程采样保留。
- gui04：同包、不重建，原生activate准确已登记测试进程后AX恢复，实际走到确认；取消前状态请求仍显示执行中并超观察窗口，原canonical仍CaptureReserved2/无plan，尚未触运行协调或Start。准确等待位置未取得，不能臆断同前三轮原因；无产品动作成功声明。原root/processgroup已清，保索引/合成片段/完整AX及streams。

两处产品修复已落盘且相关测试通过，但上述最后实屏全链不因单测转成功：

1. main0d44a4643：显式只读检查若索引已完整closedRoot，前后两个完整终态preview同phase/ticket/revision，夹住fresh live同data/HOME/Stopped/NotRequired/无pid-port/canStart快照，才协调旧UI latch；preview单独不放行、异HOME/变化/仍Required/busy均拒，绝不重放取消。99项恢复交互及typecheck过。
2. main0b77472be：新增processRequestPending，Ready快照可以展示事实，但原process_action invoke尚未返回时不能提前结束动作或接受第二个Stop/Restart（通知超时也仍拒），回执返回才结合最新真实snapshot收口。延迟Start回执/Ready交叉反例证实旧UI漏洞；不能据此冒gui02具体runtime lease错误已采到（原未记录该诊断/stack）。122项恢复/启停/倒计时相关交互与typecheck过。mainbb13a7f12仅将验证器failure cleanup从recordAbsent收紧到完整RootAbsent再允许退出/清理；没证明完整恢复则留现场。

最后88b8fc33d只激活准确PID/exe测试窗口后观察AX，编译与生命周期原30/5/10预算不变。原生后端/完整原件与安全覆盖未改，不为提高测试通过率删范围。7个准确已返回GUI/service PID经proc_pidinfo均ESRCH、4准确任务根及专用协调DB缺席复核；这只证明当前残留清理，**不补gui02的正常Stop/恢复证书**。原app/副本签名字节不变，完整失败证据约38.4MB、不保重复运行包/真实凭据。首次推送SSH瞬断raw128保记录，成功与否以真实push返回为准。

下一步本机只做gui04取消前status请求的定向阶段/原线程等待取证，确认具体阻塞点后才修对应后端；**没有新的已证假设，不再重跑整份供包/取消/启停**。此缺口与revision170部分文件算法通过分别记录。原型新文案对齐、后继代次取消/已有plan中断、一般客户端图/缺Ready、NAME真实默认客户端/最终回退、正式发行候选签名/资源校验和Windows完整Owner/privatehost/nativeV2/旧r2/synthetic27960等完整剩余保持。Windowsfetch仍58f0d49f2/0d6fa9c0d，无新回交；不W2/双SID、不发布，目标active。


### 2026-11-18 revision172：状态等待定向取证与异步轮询争锁修复

main17f07758e修后台轮询对collector的阻塞锁，51c8aca18加隔离阶段诊断与真实官方查询测试，bec99201e仅允许准确GUI任务根输出同一诊断。旧poll_once/next_wait在单worker运行时阻塞等待另线程持有的采样锁，持锁线程依赖同执行器timer才能释放；两入口各被实际watchdog以raw91截断。修后busy tick不调用来源、不清失败数、不发成功事件、不推进原due时间；调度仅按现有debounce重试。成功采样只取同一锁下投影，发事件前释放，不二次争锁。原Status5/Unknown拒绝保持。

[定向结果](../../.adg/evidence/0.2.24/mac-status-wait-diagnostic-r1/result.json)：修后两子场景/父测试、5轮询单测、6原deadline、4status投影和严格Clippy过。真实原样官方2.99在新任务Profile、实际CaptureReserved部分槽只读查询2056ms；另一独立任务在同一官方采样确实占锁时，实际后台poll_once与next_wait返回且无事件，完整查询2785ms，守原5000ms。后者host ef970752f49e448a6c4c6360bff481ab8dce9b2c086dee82cd639ed3db17cbf1；stage显示CLI读取2641ms、前后恢复检查各约15ms；返回Stopped/RecoveryRequired/canStartfalse，未取消或启停。两原group/准确任务根缺席，专用协调key均缺席，全局L未动。

诊断只在OCXD024_STATUS_TRACE=1及HOME为准确新/private/tmp/ocxd024-status-probe-*或ocxd024-gui-root-cancel-*/profile输出固定stage/相对微秒/threadID，不输出配置/载荷/凭据，不改权限或预算。首编译E0433、旧raw91、失败源与修后源/回执/清理保留。新增记录脚本的stdin编码准备错误发生在执行前，未修改产品或原证据。

**边界**：已证实具体调度缺陷与真实官方重叠查询通过；未采到gui04原线程栈，不能断言是原GUI等待唯一原因或签部分捕获GUI整链通过。下一步仅重建此源一包、定向真实WebView状态IPC/后台重叠观察，不执行取消/Start/Stop，不开无假设轮。NAME最终外部默认Codex客户端、一般图/恢复/候选与Windows原未完范围保持。不W2/双SID、不发布，目标active。


### 2026-11-18 revision173：Windows私有原生端点构建回交接收

已fetch并按净差接收codex/024-windows-native 077cbd56e103b9208450cbc3bc31274217a00656、docs/024-windows-dev 331ac32fa70ff570a2dd86c3f74e123e41789bce；功能源b6db6d582cdabd4f87329c6548526fc40db6bc30。原handoff的“077待push”为提交前时点事实，当前真实远端已核到077，不改原件。新wl2-r7-private-native-20261118-r1的109文件/108manifest项/1317391字节完整摘要过，10最终执行源逐Git仅行尾归一相等；[本机接收回执](../../.adg/evidence/0.2.24/windows-private-native-receive-r1/result.json)。这是Git/证据净差复核，没有本机Windows执行或独立原生安全认证，也未整批merge该Windows生产分支。

schema2 protectedlocator/first-instance SID管道先于ExistingQualification::prepare/OwnerStart创建；原ordinaryToken/session/exactimage/noOuterJob/nativepeer/locator/pin前后核、freshhello/nonce/replay及原QPC窗口不变。Begin仍只原registry canonicaltraitworker/ReturnedOwner复核，诊断metadata与fullhistory分开；ACK只交付数据不产权限。无旧v1回退、原loaded357热补、新image接旧pipe或自动force。新专用空fixture不绑定Runtime/Owner/官方，只用于两端原生身份与认证后拒绝检查。

Windows报告实际MSVC严格DATA8过、invalidargv7项raw2、原5受影响回归过。最终ocxd 18220032字节SHA55cd6ef08653edc8bf2b5c754006473fba7b3fc8f4a0a04c486ea89222181d3e；TEST EXE4fa4d5bdb510dce997aae19419c5e7a1be5242da82e423728cb9e7f1b6910991。四构建原首错/修后源/PS51入口检查与制品清单保留；最后nativepeer记录/ACK净差仅编译通过，不能冒原生执行通过。

**普通桌面实际端点仍Pending**：Public入口四输入摘要已核，marenben session7最后Disconnected，尚未执行launch-r7-private-native-empty.cmd。此项需该普通账户真实桌面；不提升token、不使用继承Job的控制环境绕过准入。用户参与只停本依赖，Windows可继续其它独立授权开发。151项状态不简写为“Windows完成”，真实普通Profile固定官方Owner原预算全链、V2 pre-manifest/partial/缺根、容量/CAPtrust/普通MSI/最终候选均未完。当前r2、40588、synthetic27960及拒删原现场不动，原件未还不跑新官方，原场景具体恢复需单独批准；不W2/双SID/发布。


### 2026-11-18 revision174：真实包状态IPC与首次部分取消后继正常启停收口

实际包仅重建一次：产品源bec99201e6ef4addaed61d6d50bf8b67a956d0e2，Tauri构建raw0/263087ms；包内Desktop/ocxd/ocx逐同轮Cargo回执规范化签名字节相等，严格codesign校验过。sealed Desktop 673e50ab378ceb93614b9d264dbd3b740b8a138519d3a14ef4831671623beeaa；这是本机验证包、未公证/非最终发行签名候选，不以构建成功代发布验收。后继436e6dd9a/841291a2a/0ae095327只有验证器变化，没有重建或冒更晚源是包源。

[真实只读状态GUI结果](../../.adg/evidence/0.2.24/mac-status-wait-diagnostic-r1/gui-result.json)：gui01验证器只等current.json出现便结束准备进程，落在current已发布、激活pending尚未清的中间点；seed在read_complete处正确拒runtime_selection_pending，未到Index/目标查询/服务。该失败不记产品缺陷、不改原判定；保raw manager/原current/pending后核无Index/客户端根/运行记录/自有进程，准确根与协调key已清。841291a2a改为current存在、pending缺席且真实概览就绪才结束准备。同一个包新gui02实际部分预留根经真实WebView检查及一次原生点击刷新进入Tauri，collector同thread acquired→return 775.519ms、UI观测2146ms；原head全相等、无服务动作、原两GUI组/准确根/专用协调key已清。它证明返回与原5秒预算，不用未知/忙状态当成功事实，不冒原gui04唯一因果已确证。

有上述新定向事实后才开[gui05完整本场景](../../.adg/evidence/0.2.24/mac-first-root-partial-gui-r1/gui05-review.json)：仍同包，实际首次CaptureReserved2/原ticket/合成未发布片段，真实界面确认取消。第一次状态busy在变更前拒绝且原head完全不变；显式重新核对后第二次完成RootCancellationAbsent4，界面观察7352ms。旧部分槽byte/device/inode/uid/gid/mode在取消后及最终正常Stop后完全相同，未收养Original refs。

随后真实显式新Start，原样官方2.99后继ticket不同，到Ready8；服务PID18678/实际端口58334。正常Stop取得原native service与launcher退出0、原4监听全部缺席、三个原件还原读回、根归档/四协调释放及最终RootAbsent13/完整completion摘要；实际stop总耗8662ms<=原10000ms，其中恢复6083ms，未用force、record缺席或tasklist替代证明。停止完成卡显示并按原倒计时自动收起，Start入口恢复。原30/5/10预算与未知拒绝未改，观察超时不是新产品预算。

[清理补证](../../.adg/evidence/0.2.24/mac-first-root-partial-gui-r1/gui05-final-cleanup.json)：两原GUI PID18241/18498和服务18678均原生proc_pidinfo返回ESRCH，准确测试根与十二task-key协调文件/sidecar缺席；已有global L未动。原sealed包/副本字节一致，原gui01~04失败与gui02旧不完整Stop清理不改成通过。新成功单独放gui05，本场景scenarioAccepted=true而releaseAcceptance=false；两组新GUI证据合计约28MB，不保重复app/运行包或真实密钥。原型同步仍属其它对话写集，未越权并发修改。

**后续不重复本场景**。首次部分取消及真实官方后继正常Stop缺口在本源关闭；后继代次Preparing取消、已有plan中断显式恢复、一般客户端图/缺Ready、NAME真实默认客户端加载及最终重启/回退、发行正式入口/密钥/实际候选与Windows完整Owner/nativeV2/普通桌面等独立未完保持。Windows最新接收见revision173，不借本批授权处置原r2/synthetic27960/40588。下一批按未完依赖选单个明确假设，不扩大sweep、不W2/双SID、不发布，目标active。


### 2026-11-18 revision175：Windows准备前缀原生读回回交接收

已fetch并净差核对代码96396f19bb03add0fdd29c56ec20fbeab189e6d3、证据3294b72b3138e45a97fbc1d0748864dc42560cb9，最新handoff与原push最终回执已接。新wl2-r7-preparation-read-20261118-r1的79文件/78manifest项/1021339字节逐摘要完整；6最终product08执行源与该Git只行尾归一相等。[接收回执](../../.adg/evidence/0.2.24/windows-preparation-read-receive-r1/result.json)只证明来源/证据一致，不冒本机Windows执行或整批生产集成。

原JournalPreparation只读原CREATE_NEW句柄中的empty/partial/full manifest/current与原expected准确前缀，ID/path/所含8/ADS/members/目录SD/pins/nativepeer前后核，每文件两读及整组复读，漂移拒。缺初始安全baseline保持securityDigest=null，不用当前安全描述符补原证明；Source/聚合coverage未知仍Pending，不写完整记录或清poison、不续旧Start。V2 actual journal固定manifest0/current1/aggregate2，原CaptureSlot/PartialAnchor只有限阶段映射，无从DATA构造Owner/恢复许可。

Windows报告4实际native测试去重30场景：10 initializerfault、9未知/漂移拒绝、1旧Start过期后原半manifest在新只读5秒观察，以及10 actualslot故障阶段映射。后继执行只复核变化项，不叠加轮次算通过；10受影响回归过。原rustfmt/编译/collector准备错误与修后源、全部raw保留，TEST EXE defec542f74cf2e87dc77ea8fbe5ea0e7fcc0d08e418cf56010a49350b880b03；55新成功fixture根缺席，旧partialcapture只FixtureDrop不冒全历史清理。

157项原状态表保持：这只是pre-manifest必要读回，不等fullSource/全部对象/partialwork/缺根恢复。普通桌面V2入口仍Pending，原Public输入55cd6ef…未覆盖；真实普通Profile官方Owner原预算全链/容量/CAPtrust/普通MSI/固定候选未完。当前r2/40588/synthetic27960/旧拒删现场未动，没有新force/归还授权；原四trait/r7允许写集不变，Windows可继续独立推进，不等Mac接口。不W2/双SID、不发布，目标active。


### 2026-11-18 revision176：NAME整文件备份回退与实际客户端目标确认

用户确认实际客户端是**ChatGPT内的本地Codex**，不是独立桌面版或终端CLI；需求AC-P24-09和[实际客户端验证边界](0.2.24-contracts/chatgpt-codex-name-validation-r1.md)已明确，不据客户端类型变更缩小最终验收范围。已安装ChatGPT标识com.openai.codex、版本26.1111.21308/build27194，仅从公开Info.plist读版本，未读取/修改日常配置、重启宿主或发送模型请求。

[本批结果](../../.adg/evidence/0.2.24/channel-name-rollback-r1/result.json)源码main18a126a5854c2a8aeedf1f3a56b8706b5a3828f4只增定向测试。真实Rust保存得到中文“备份渠道”＋单模型“人工名称”及协议拆组；调用原手动完整备份，保存为“修改后渠道”，再经原prepare_rollback/apply整文件回退。当前config.json原始字节与备份前完全相等、port41234/合成凭据保持，独立snapshot重开无草稿且渠道label恢复；回退前保护备份另有新ID。测试的stopped/validator闭包明确是文件事务隔离，不冒实际服务停止、官方校验或产品UI权限证明。

同一原样官方Darwin包5b10c1a8…，三份配置分别由三个新Bun进程执行官方saveConfig/loadConfig并各刷新两次，6次均过（1166/548/545ms为各进程总观察）。原生回退前后raw一致，官方生成slug/display_name/contextWindow/reasoning逐项相同；修改阶段只有自动名称改变，人工名称优先不变，原vendor/custom的斜线仅显示为／。无运行服务、上游模型请求或业务同步。Rust定向1用例/严格lib+tests Clippy均过；准确临时根已缺席，34项证据约121KB，执行源码逐当前Git完全相等，真实host摘要保留。driver新增成功条件在所有比较完成后才设置，任一失败保准确fixture，不把三子进程raw0掩盖最后比较失败。

**实际选择器仍未验**：[目标与限制回执](../../.adg/evidence/0.2.24/channel-name-rollback-r1/client-target.json)。CUA直接选择ChatGPT时返回“Computer Use is not allowed to use the app com.openai.codex for safety reasons”，未取得该界面、未点击/输入，不通过AppleScript/CGEvent/其它通道绕过，也未修改日常目录或重启当前宿主。官方文档确认model_catalog_json在启动加载、display_name是选择器标签；文档和本批六刷新不是ChatGPT已加载的证据。将合成输入/前后显示表保留供最终具备隔离运行条件的人工核对；不推造ChatGPT支持CLI的--profile启动旗或要求改日常config。

P24-NAME本次补齐配置事务＋官方新进程重新加载/目录回退组件证据；最终真实ChatGPT选择器/最终候选应用重启回退保持未验，不签整项或整版通过。Windows96396f19b/3294b72b3的新preparation读回接收归revision175，独占写集/原预算/旧现场权限不变。本机继续后继代次准备取消、已登记plan中断恢复/一般客户端图等独立未完；原型并行写集不触。候选未冻结，不W2/双SID、不发布，目标active。


### 2026-11-18 revision177：名称验证的外部任务协调文件清理补证

清理复核发现，revision176原临时根确已删除，但官方目录刷新还创建了临时根外、按准确CODEX_HOME派生的单个catalog-write-locks数据库。原round01结果/rootAbsent和源18a126a58均不修改，不将原报告扩大成外部协调对象已清。[独立补证](../../.adg/evidence/0.2.24/channel-name-rollback-coordination-r1/result.json)绑定原唯一新任务Profile，准确列出N/K/H三文件及sidecar共12路径；原K由本次源/随机任务路径确定，不扫描认领其它key。

该K原native身份/owner/mode600/nlink1/原字节先留证，经同SQLite文件BEGIN EXCLUSIVE/ROLLBACK证明无持锁者，重新核ID/字节后只删该准确任务对象；其余11路径保持缺席。原run01三官方进程全部terminal raw0，当前任务根与12外部路径缺席；global L未选中/未动，原目录和业务配置未改。原round01没有运行前外部路径缺席回执，明确记为缺证，不补造；该轮功能通过与清理的后续独立事实分开。

main15dbb8286只收紧验证器：开跑先记录该新Profile的12准确路径全缺席，已有任何对象则拒绝，不进入cargo/官方进程；成功后复用已原生验证的准确task-key清理helper。清理失败即allPhasesPassed=false/保fixture，三个child raw0不能覆盖末端失败。新定向负例实际放一个自有task K后执行原driver预检，真实拒绝且0子进程调用/原字节不变/失败fixture保留；明确测试清理后原根与12key缺席。Python语法/diff过；真实原K释放清理和已有helper范围复用，不重跑未变的名称事务及六次官方刷新。新driver源码/实际负例/全部原字节摘要保留。

产品后端与名称语义未改，Windows独占/四trait/r7和原30/5/10预算不变。NAME真实ChatGPT选择器工具禁用依赖、最终候选/重启回退、其它代次取消/中断与一般图、Windows完整原生和旧现场范围均按原计划保持。无正式发布或W2/双SID，目标active。


### 2026-11-18 revision178：同验证器三份历史名称测试的准确协调清理

按revision177已确认的同一生成路径，只补查channel-display-native-r1中native02/03/04三份已绑定结果；原结果逐Git字节一致、准确新/private/tmp/ocxd024-name-refresh-* Profile可追溯、根已缺席，原官方子进程均terminal（失败轮1、两成功轮各0/0/0），serviceStarted均false。每Profile只发现其catalog-write-locks单个任务K，其余N/H及sidecar缺席。

[三例独立补证](../../.adg/evidence/0.2.24/channel-name-rollback-coordination-r1/historical/result.json)先留原字节/identity/owner/mode/nlink，原SQLite排他取得并释放、重核后只删三个准确K；原三根及36准确路径全部缺席，global L未动，当前其它Profile/Windows现场未选中。运行前外部缺席回执原轮次未采，仍不补造；native02的功能失败和原清理记录不改成通过。原round01及本三轮的后续清理事实分别保留，不重复原保存/六刷新或扩跑全量验收。

本批只有事实/证据补充，main仍15dbb8286、TEAM39/r7与产品/Windows写集保持；实际ChatGPT选择器工具禁用和其余正式候选/原生/恢复未完项按revision176/177继续，目标active，不发布。


### 2026-11-18 revision179：Windows目录准备槽原生读回接收

已fetch并净差复核代码b1ef529722a503e4b36a0b3b99f9ca66e1ff1cec、证据7adf93a64d7b5f9afdd4c87963b744c4c39ca47f；原目录读取证据432afe278和后续push-receipt一起保留，不回写seal时pushPending/TLS失败历史。新目录77项manifest/875662字节全部匹配，6最终product08源与准确Git仅行尾归一相同，[接收回执](../../.adg/evidence/0.2.24/windows-directory-read-receive-r1/result.json)不冒本机Windows原生执行/生产全链或整批merge。

PrivateDirectoryPreparation只借原对象及actualTransferred事实；terminal-held才取准确path/ID/所含8/ADS/两读members/SD/pins与原parent复核，无terminal创建禁止从path补ID/Absent/新open或成员枚举，转交后不收养后续内容。initialSD未捕获继续securityDigest=null，currentSD仅diagnostic，完整Source/coverage保持Pending；没有restorefactory或权限扩大。V2固定context/custody/grok/目录slot不随subset缩号。

Windows报告新5native测试13fixture过、9受影响回归过；真实native后端check过且源不变。两编译错误仅明确别名/引用修正，全部失败raw/源保留，无意外native首错。TEST EXE547d50530139edcf728e4a1161847dbc413409c74b358aa60c076787b5b95dfe；34新成功根缺席，不冒旧拒删/失败/synthetic27960/40588历史全清。163项状态仍保完整initialSD/Source/partialcapture-work/nativeV2恢复/缺根、真实普通Profile官方Owner原预算全链/容量/CAPtrust/普通MSI/最终候选未完。普通桌面Publicout仍空/session7Disconnected，原55cd6ef…输入未覆盖。

当前r2 Job24raw50未知依赖只停自身，原21500/75744及原件/work/custody/anchors/协调/seq9未动，无新force/归还授权；不重复旧query/cleanup或旧拒删。Windows继续原r7单写，不等待Mac新接口；本机新增[后继准备取消合同](0.2.24-contracts/client-successor-prepare-cancel-r1.md)承接既有未完范围，先完整历史只读审查再原生保全/终态/恢复与实屏，不放宽首次限定或把组件当整项通过。不W2/双SID、不发布，目标active。


### 2026-11-18 revision180：后继Preparing取消完整历史审查与真实取消前驱

main845b2861924a4412a168a8ac148a8d1320f097d9新增generation::preparation_cancellation独立只读Review，7765271fff6c4e5deaad6a122b82bdc58145b439仅原生定向测试/准确driver。当前只读审查**不接产品预览或按钮、不赋移动/服务缺席/恢复/关闭/Start许可**，不是后继取消完整实现。[本批结果](../../.adg/evidence/0.2.24/mac-successor-cancel-review-r1/result.json)和[完整目标合同](0.2.24-contracts/client-successor-prepare-cancel-r1.md)分别保范围，不因组件通过删后继移动/计划中断未完。

Review借原Index/context/deadline，只接受当前后继Preparing＋原rootGeneration初始事件、无Original refs/计划/完成，原准确ticket/代次slot/RootOrigin/紧邻RootAbsent或RootCancellationAbsent前驱与原编码摘要。全有界独立lineage前后核，按根代次规则逐前代原件/归档/安全属性/成员/关闭来源完整读取，当前根仍空且Grok/Head/目录身份前后相等；同指纹两次观察后才返回确认摘要，recheck仍原deadline。首次review保原限制，CaptureReserved/新数据或未知历史不被收养。没有更改旧source/binding/revision或原预算。

6个新唯一测试通过：真实文件/索引的普通RootAbsent前驱正例，错Head/HOME/期限与后续CaptureReserved拒；新增用户文件、未登记vault成员、旧Original字节漂移拒且读操作不改现场；Grok变化使旧确认失效，新观察可反映当前Grok但原deadline真实经过3秒后仍拒。两执行修订共10准确fresh fixture均打印创建/删除路径且独立缺席核；已通过项不叠加验收，严格lib+tests Clippy过。测试启动的scope是明确合成存储，不冒官方/服务能力。

为覆盖另一允许前驱，只补一项[真实原生取消前驱](../../.adg/evidence/0.2.24/mac-successor-cancel-review-r1/native-review-result.json)：固定原样官方Darwin2.99＋真实CurrentService/Guard/四锁，首次新根Preparing1→RootCancellationAbsent3，原取消9705ms<=10000ms。再独立新Start准备生成后继Preparing4，新的只读审查窗口仍10秒，完整Review+recheck847ms，重开事实Index仍能审查；不捕获后继Original、不移动后继、不Start服务。附加旧取消归档改动后重核拒，原合成字节保留；该附加拒绝未单独采错误分类/末端耗时，不当完整独立反例证明，普通旧Original漂移已有上面新deadline负例。

native host807474a4eeb04706e8c708465b276b5e7a9f3c18225bff321b391fc1597e4a0b同轮Cargo回执/执行测试源逐Git相等；产品source845、native test source776分开。完整native运行81981ms包含先供包，非取消/审查预算；同一原进程等待期间只读sample证明仍在program_selection完整SHA读取，未取消/重跑，随后原进程raw0。driver真实10秒以产品动作判定，不用外层180秒观察冒通过；scope=successor-review若失败/超时保准确进程及fixture，不走旧自动kill/删根。两严格Clippy回执均0。

准确原native进程25686经proc_pidinfo确认ESRCH、其独立processgroup空、task根及准确N/K/H十二文件/sidecar缺席；运行前缺席回执与释放后原字节保留，global L未动，无真实凭据或业务请求。新增证据约0.83MB不保重复运行包。Windowsb1ef52972/7adf93a64接收归revision179，原写集/四trait/r7不变；本机继续后继保全移动/索引终态/多计划namespace与中断恢复，再真实用户动作/原包后继启停，不重复首次或NAME已过项。ChatGPT选择器、一般客户端图/缺Ready、最终候选及Windows原完整未完保持；不W2/双SID、不发布，目标active。

### 2026-11-18 revision181：从独立历史核对取消计划目录

main cd7cd0a9e62b3f5f88e0054c3498cbfe18616943 从有界独立历史导出完整计划目录集合，核对相邻源事件、登记、终态及计划引用唯一性。归档计划继续核自己的历史前缀，同时按当前完整索引核对目录成员。缺少、额外或复用计划均拒绝。该校验仅提供成员关系，后继保全、独立终态、登记后中断恢复和后继启停仍待实现和验证。

目录集合 2 项、首次审查 5 项、后继审查 6 项、首次登记 2 项、关闭历史 20 项通过，严格 Clippy 通过。首次登记 1 项需要隔离原生 Guard/NativeLease，定向轮保留 ignored。run02 保留因临时目录路径含 macOS 符号链接导致的失败及首错，验证后清理准确测试目录。run03 保留零匹配过滤器的回执，run04 另跑正确过滤器。四份被测文件摘要与提交逐一核对。[证据清单](../../.adg/evidence/0.2.24/mac-cancel-plan-namespace-r1/evidence-manifest.json)

### 2026-11-18 revision182：首次取消排除未登记目录槽

main 9daf5599151b65437f94ab2cadb1988060ee38c3 在零登记计划时同样核对目录集合。新槽由当前动作独占创建并固定身份，只能用于准确的下一次 RootCancellationPrepared 登记。针对性 Rust 测试 15 项通过，异常槽回归核对原槽字节和 SQLite Head 保持。

run05 原生显式取消在 6,982 ms／10,000 ms 到达 RootCancellationAbsent revision 3。陈旧确认与归档篡改均被拒绝，四项原生协调权限已释放。后继模拟捕获在 2,044 ms／30,000 ms 到达 Captured revision 6，服务未启动。测试根、进程组及准确协调路径均已清理，全局生命周期协调未动。回执的运行器基线和被测提交由原二进制净差及摘要绑定。该轮提供首次取消回归证据，后继取消移动、中断恢复、真实后继启停及整版验收仍待完成。[证据清单](../../.adg/evidence/0.2.24/mac-cancel-plan-namespace-r1/evidence-manifest.json)


## revision183：Windows r7目录、准备与扫描接收

代码 0816047f30f667e56bea4167ec012f6e12bfcbdd，证据 cecd17bda58ebbccc0bd0bb0a0a017a5df1ab5ba。接收全树成员、原准备绑定V2 Pending DATA、长路径查询和复用扫描缓冲的定向证据，未整合Windows生产分支。按固定Git对象核对四个清单的逐文件长度与SHA256：tree-membership 73件、bound-preparation 109件、full-msvc-scan-entry 37件、initial-start-worker-request 10件。实际固定副本MSVC完整扫描2634.174ms，16项回归通过，初次10079.394ms失败保留。缓存和宿主负载未控制，生产Start 30秒仍须定向实证。Public输入的活动比对来自Windows回交。普通桌面私有V2未取得输出，session 7断开。整链原件恢复、普通桌面生命周期和候选资格尚未通过。

初始Start持锁线程的所有权和后继恢复权限已在[Windows r7合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)冻结。Windows可继续本线开发，先在自有夹具证明同原线程和同范围。旧r2保持原状，固定候选前不进入W2。

## revision184：Windows r7 Journal 定向回交接收

代码 89e8c64478553c9715eb5babfe00f9eeed14af11，证据 420d7a948aeea76f198d6ffe8e2ee4a8b05499f0。按共同基线 9daf5599151b65437f94ab2cadb1988060ee38c3 核净差，并逐文件核对 Git blob 的长度和 SHA-256。定位摘要轮 43 件、最终命名数据流轮 40 件，均与清单相符。两轮基线均为 raw101。修复后定位摘要轮 4 项及关联 3 项通过，最终命名数据流轮 5 项通过。定位摘要绑定原保留目录实际路径，成员变化仍进入观测摘要。完整初始化 Journal 在末尾再次检查命名数据流。Windows 回交确认两轮分别清理了 32、31 个成功测试根，失败现场保留。两轮均为 productionAccepted:false，生产代码尚未合入。

[Windows r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)的写集、接口、预算与未知拒绝继续有效。初始 Start 持锁 worker、显式恢复和普通桌面 private V2 仍待交付。旧 r2 的 10 份只读导出一致，Job 仍有 24 项未知（raw50），恢复计划 executionReady=false。旧现场保持原状，固定候选后才进入 W2 或双 SID。


## revision185：Windows r7 首启保留与最终释放交接

首启准备释放代码 737bca2e3cd8899cda0d0d86bf8b5fbb446ebca1，证据 8b3ed982236014b0755538d7c3f00a3649c53fc3。核对 58 份 Git blob 的长度及 SHA-256，并复核 202 项状态。Windows 交接记录修正 Bun 夹具后准备套件 7/7、raw0，SQLite 负控 raw73；已提交的 stdout 仍是修正夹具前的 6/7。注入部分解锁错误后保持 ReleasePrepared，外部未取得原 claim。最终释放负例未取得注入的 raw5，固定 Bun 正例于 10,116 ms 返回 raw101（bounded-original-deadline）。成功但未获删除批准的两处合成根和六处失败根均保留。[证据目录](../../.adg/evidence/0.2.24/windows-dev/wl2-r7-start-owner-release-20261118-r1/)

复核后继代码 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2 的八处 Windows 文件。补丁将保留原租约接入最终恢复释放，在原保护边界内减少重复树读取，并增加注入部分解锁后的范围及 ReleasePrepared 断言。该 SHA 的原生回执待交，生产分支未集成。持久 worker、显式恢复、普通桌面私有 V2、普通用户 MSI 及原包单 SID 生命周期仍待闭合。公开 Start 保持拒绝，按原预算和未知拒绝继续验证。旧 r2 的 Node 未 Resume，三件原件留在 custody，十份只读导出一致，Job 仍有 24 项未知（raw50），executionReady=false。固定候选后再做 W2 或双 SID。


## revision186：Windows r7 最终释放计时与租约范围交接

已接收最终释放代码 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2、证据 f260a34da4dcabe4bf15fe733f4cd9a0686d80b9，以及租约范围代码 e54a1156de42d445903faedb93aed4923f400b3a、证据 e9e0522629f522e172650f3ea3734fa6b6cc4ae0。按固定 Git 对象复核最终释放 84 件、租约范围 31 件的长度与 SHA-256，并核对 210、216 项累计状态及执行源码。Windows 后继代码待集成。

最终释放在默认 MSVC 测试构建下耗时 10,070 ms，raw101。相同源码以测试 crate opt-level=1 单次通过，耗时 9.40 秒，raw0。缓存和主机负载未控制，生产 10 秒预算仍待正式构建验证。raw5 负控在 8.55 秒完成，原 claim 保持冲突。中断 1/1、释放 6/6，Bun 负控 raw73 保留，三项产品二进制构建通过。四处夹具删除请求被拒后保留。[固定计时证据](https://github.com/gzers/opencodex-desktop/tree/f260a34da4dcabe4bf15fe733f4cd9a0686d80b9/.adg/evidence/0.2.24/windows-dev/wl2-r7-final-release-timing-20261118-r1)

租约范围基线复现第三个角色释放 SQLite RESERVED 字节后仍通过准入，定向测试 raw101。修正后逐范围核对排他租约与获取线程，检查失败时撤销变更权限并保留仍持有的 claim。精确范围 5/5、分段 7/7、中断 1/1，修正测试 raw0。夹具清单记录 53 个根，其中 33 个存在、20 个缺席。[固定范围证据](https://github.com/gzers/opencodex-desktop/tree/e9e0522629f522e172650f3ea3734fa6b6cc4ae0/.adg/evidence/0.2.24/windows-dev/wl2-r7-lease-range-20261118-r1)

按 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)接续首启原线程持锁和显式恢复。原 worker 在首次原生获取前绑定 Start，过期变更权限维持失效，独立恢复权限仅覆盖原对象及连续持有的精确范围。公开 Start 维持拒绝。普通桌面 private V2、普通用户 MSI 和官方单 SID 生产生命周期仍待验证。旧 r2 的三件原件留在 custody，Node 从未 Resume，十份只读导出一致，24 项 Job 查询未知（raw50），executionReady=false。固定候选后进入 W2 与双 SID。

## revision187：Windows 首启工作线程回交与 Mac 后继取消预览

Windows 代码 a5ba9aee338ed69841a836e4a5be639115fa8e1c，证据 b8fe82f802b4df3d2a621b416e365ca609ec5671。14 个 Windows 文件的净差检查通过，27 个证据文件与清单一致。第二批控制测试与 7ca43ca47bb642a91e8c69444afaad673582a213 的 13 个源文件全部一致，2/2 通过。证据记有四次真实 SQLite raw33 冲突及 19 个测试根缺席。第二批 EXE SHA-256 为 11dd88f897ab5cc1fe141080c380d4bbeddc4283e56b65659bd3f1483ac9df70。第一批报告 5/5，但换行归一后仅 11/13 个源文件匹配，测试入口与 start_worker 测试模块不同，原 EXE 在记录摘要前被覆盖。保留原报告，待实际输入或更正回执绑定后再计入验证。三个产品 MSVC 二进制构建通过，14/14 源文件与代码提交一致，运行验收待做。[Windows 固定证据](https://github.com/gzers/opencodex-desktop/tree/b8fe82f802b4df3d2a621b416e365ca609ec5671/.adg/evidence/0.2.24/windows-dev/wl2-r7-start-worker-20261118-r1)

代码已在首次获取前登记 Owner，派发失败时保留 Owner 并拒绝普通动作。尚需证实原线程连续存活、同范围持有及显式恢复派发。登记本身只证明记录存在。默认 MSVC Stop 10,070 ms 失败仍有效，9.40 秒优化测试控制轮未完成生产验收。普通桌面私有 V2、普通用户 MSI、官方单 SID 生命周期、候选集成仍待完成。旧 r2 的三件原件、十份只读回执和 24 项 Job 未知保持原状，executionReady=false。公开 Start 保持拒绝。

Mac main 6bafdd342e69ee83e842f4b35a20bafa1320645a 用原 Index 和 SQLite 连接对齐后继准备取消的操作审查与产品预览，避免打开同一文件的第二个描述符。真实存储夹具的首次取消用时 6,787 ms，后继审查 792 ms，预览 164 ms，确认指纹一致。原 Guard 的持有、准备、移动与释放均有回执。准确测试根和外部协调路径已清理，全局协调未动。此轮只证明后继确认入口，后继保全、独立取消终态、中断接续及签名候选尚待闭合。聊天内 Codex 仍是名称选择器的实际验收目标。[Mac 证据清单](../../.adg/evidence/0.2.24/mac-successor-cancel-preview-r1/evidence-manifest.json)

## revision188：Windows r7 首启保留、准备附件与 Journal 交接

接收 Windows 代码 cfce007f0ae1de13c20e797b9448b5fcc1605875、证据 99520867ba3836b5443f61fc4620e6968083987c。复核新增 36 个代码路径，其中六个 Mac／共享路径与 main 6bafdd342e69ee83e842f4b35a20bafa1320645a 字节一致。五组清单共 219 个载荷逐件核对长度与 SHA-256。生产分支待按写集净差集成。EXE 摘要为证据记载值。

| 定向证据 | 接收判定 |
| --- | --- |
| retained-start-recovery，47 件 | 报告 20 项通过、44 个根缺席。test01 原字节仅 6/14、test02 仅 15/16 与对应 Git 一致。bins01 原字节 16/22，其余六份记为行尾归一一致。整组保留为历史记录，重叠结论采用后继绑定证据。 |
| preparation-pending，53 件 | 06aaf204fc 六条源输入匹配，报告 42/42、92 个根缺席。五个同名测试的绑定复测覆盖早先 EXE 被覆盖且源不匹配的回执。Pending 原字节在内存，durable=false。 |
| worker-result，40 件 | 37fd73ffa 三条源输入匹配，报告 18/18、27 个根缺席。 |
| recovery-scan-count，26 件 | 34a766bfd 六条源输入匹配。整链 10,144 ms，raw101，160 次 Journal 读与 82 次退役树读，读区间并集 3.78054 秒。部分解锁负控 10,039 ms，raw101，第 138 次 Journal 读未完成。两项失败和四个保留夹具根仍有效。 |
| journal-append-readback，53 件 | cfce007f0 三条源输入匹配，报告七组 15/15、三个 MSVC 产品构建、58 个夹具根缺席。复用同一次完整写前检查解析的 Current，保留字节 CAS、独立重放、完整写后回读、poison 和原期限。端到端计时及原生读收益待证。 |

[固定交接](https://github.com/gzers/opencodex-desktop/blob/99520867ba3836b5443f61fc4620e6968083987c/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)和 [r7 增量决定](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)记录未来隔离宿主严格 V2 部分捕获附件。普通桌面私有 V2、同原线程生产恢复原预算、普通用户 MSI 和官方单 SID 生命周期待完成。公开 Start 保持拒绝。旧 r2 的 Node 未 Resume，session 7 断开，三件原件在保管中，十份导出不变，24 项 Job 查询 raw50／Unknown，executionReady=false。


## revision189：Windows r7 Journal 读回计时交接

净差接收代码 6dbffae54c528750bfb6d5ad9b9e08dabaf78495、证据 8777e862f0b2017e94b77ad426ea7a6a324bcd9e。源码增量限三个 Windows 原件 Journal 文件的测试计时和相应用例。原生对象、安全描述符、流、成员集合、独立重放、追加读回和截止期限检查均在。两轮清单分别核对 54 和 49 份原始 Git 载荷的长度、SHA-256 和精确成员，执行源 2/2 和 3/3 分别绑定 ca1de1350a565b3a0be8557591b7332a60994c71 和 6dbffae54c528750bfb6d5ad9b9e08dabaf78495。各自最终 Journal 测试 5/5，报告各 14 个夹具根缺席。第一轮初始格式字符串编译失败 raw101 保留，修正编译 raw0。第二轮三个产品二进制构建 raw0。[固定交接](https://github.com/gzers/opencodex-desktop/blob/8777e862f0b2017e94b77ad426ea7a6a324bcd9e/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)

第一轮 12 次追加包含 24 次全读回，合计 88,909 μs，独立重放 21,159 μs。第二轮 25 次全读回合计 97,869 μs，其中每次重新检查记录及保留文件身份、属性和字节占 60.73%，当前记录字节和解析占 1.95%。该合成结果只供后续定位，生产完整恢复在原 10 秒内仍未证明，保留此前 10,144 和 10,039 ms 失败。[重放证据](https://github.com/gzers/opencodex-desktop/tree/8777e862f0b2017e94b77ad426ea7a6a324bcd9e/.adg/evidence/0.2.24/windows-dev/wl2-r7-journal-replay-cost-20261118-r1) [分段证据](https://github.com/gzers/opencodex-desktop/tree/8777e862f0b2017e94b77ad426ea7a6a324bcd9e/.adg/evidence/0.2.24/windows-dev/wl2-r7-journal-readback-stages-20261118-r1)

Windows 继续原 worker 的严格 V2 部分捕获附件与同线程恢复，公开 Start 保持拒绝。旧 r2 的 Node 从未 Resume，三个原件仍在保管中，Job 的 24 项查询为 Unknown（raw50），生产首错仍为 command-lineage raw64，executionReady=false。旧现场保持原状。普通桌面 private V2、普通用户 MSI、官方单 SID 生命周期及原预算仍待完成。固定签名候选后进入 W2／双 SID。

## revision190：Windows r7 准备失败的 V2 保全附件

Windows 代码 `0d0c9d29ccd9075d59f92350c158c256c835924b`，证据 `3155e8fa0b9f6f82d023d415996e9edabffdc3f4`。复核 11 个改动文件及最终执行源、67 项清单的字节数和 SHA-256、七组原始日志及 263 项矩阵。定向 28/28、最终测试和三个 MSVC 开发制品编译通过，51 个不同夹具根在本批前后经报告核实缺席。首次 Cargo 成功而包装器因生成 schema 拒绝的日志保留。原 `status.json` 回执把治理提交 ec5c597b77701811d60a2f1c394c3dd06cef0b43 写为 TEAM48／IMP187，实际为 TEAM49／IMP188，以原交接与提交内容为准。[固定证据](https://github.com/gzers/opencodex-desktop/tree/3155e8fa0b9f6f82d023d415996e9edabffdc3f4/.adg/evidence/0.2.24/windows-dev/wl2-r7-pending-durable-20261118-r1)

本次接收隔离宿主的持久化组件证据。原 worker 绑定原私有 D、普通令牌、原线程及动作，严格 V2 Intent／Pending 独占创建、刷盘、读回并复核后才报告 durable=true；失败保留准确前缀。本次未向 main 合入 Windows 分支。普通桌面私有 V2、实际原 Owner 回执、同线程整链 10 秒恢复、普通用户 MSI 和官方单 SID 生命周期待验。继续保留 10,144 与 10,039 ms 的整链超时。公开 Start 保持拒绝；旧 r2 现场维持原状，三件原件仍在 custody，Node 未 Resume，Job 24 项 unknown／raw50，executionReady=false。固定签名候选前不进入 W2／双 SID。

## revision191 Windows r7 保全状态、扫描与普通桌面准入

净差复核代码 7fb2f870683566487cd0fee236f03dfa90fee93c、证据 4cf484e08e13060f0e8d8687bd755f1c69c1025d。相对 0d0c9d29 增量两笔、六个 Windows 文件，完整镜像校验保留，计时仅用于测试。三组清单成员、原始 Git 字节长度与 SHA-256 逐项匹配，前两组六份执行源匹配。当前为交接接收。[固定交接](https://github.com/gzers/opencodex-desktop/blob/4cf484e08e13060f0e8d8687bd755f1c69c1025d/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)

| 证据 | 复核结果 |
| --- | --- |
| 准备保全状态，46 件 | 2/2，93 与 63 ms。真实附件 I/O、存活夹具 worker、合成 worker 状态。覆盖精确读回、同名、短写、flush、回读失败和失联。首错和两条记录保留，回执确认五个最终根及首个归档夹具缺席。 |
| 准备完整扫描，32 件 | 1/1，47 ms，三个 MSVC 开发制品编译通过。第三次完整校验拒绝等长字节变化，原 deadline 保留。回执确认夹具及生成 schema 缺席，真实 Owner 顺序和生产预算待验。 |
| 旧 r2 源码复核，4 件 | 原宿主提交 357210891f14a1851881e6192bd09d7031471707，树 a284da0e1bd517daa26848857b8189bbc88838c0。唯一 Resume 在捕获成功之后。实际捕获 30,699 ms，created-suspended，originalPhaseRetained=false。24 个 Job 观察为 Unknown／raw50。 |

后继隔离准入与注入点见 [r7 合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。仍缺普通桌面原 Owner V2 回执、同原线程完整恢复 10 秒、普通用户 MSI 和官方单 SID 生命周期。整链 10,144／10,039 ms 超时保留。旧 r2 保持原状，三件原件仍在 custody，Node 未 Resume，首错 command-lineage raw64，executionReady=false。公开 Start 保持拒绝，签名固定候选后再进入 W2／双 SID。

## revision192 Windows r7 完整镜像扫描组件复核

复核代码 `766d1f010be60865e7a425dab242ab5d861773b5`、证据 `f4bd13e48a82474b1cb23e000bfe86976733e4ce`。相对 IMP191 净差为两个 Windows 镜像扫描文件，两批共 81 件证据的成员、Git 字节长度、SHA-256 和执行源绑定一致。制品摘要来自 Windows 回执。[固定交接](https://github.com/gzers/opencodex-desktop/blob/f4bd13e48a82474b1cb23e000bfe86976733e4ce/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)

| 证据 | 复核结果 |
| --- | --- |
| 晚到目录 ADS，21 件 | 1/1。完整文件读取后新增目录 ADS 仍拒绝。初版 E0609 和修正后 MSVC raw0 均保留。 |
| 枚举类型提示，60 件 | 18 通过、3 忽略，测试和三个 MSVC 开发制品 raw0。文件与目录双向替换均拒绝。持有句柄仍核实际类型、非重解析、路径、卷、身份、八项安全投影、ADS、内容及两次独立成员复查。rustfmt 的 PATH 首错和固定工具链通过记录均保留。 |
| 固定副本配对计时 | 12 次扫描各哈希 10,586 个文件，均在 10 秒内。总中位 2694.602 → 2460.663 ms，枚举与固定中位 1627.553 → 1415.586 ms，保留 5855.232 ms 基线样本。回执记录夹具及生成 schema 已清理，固定只读副本仍保留。 |

本次接收组件证据。普通桌面原 Owner V2、同线程完整恢复 10 秒、普通用户 MSI 和官方单 SID 生命周期待验。沿用 TEAM52 的独立桌面准入和 pre-manifest 注入点，完整执行三次扫描。保留既有 10,144／10,039 ms 整链超时，旧 r2 现场维持隔离，公开 Start 保持拒绝。签名固定候选后才进入 W2／双 SID。

## revision193 Windows r7 原 Owner 测试入口复核

复核 Windows `18833e7b65c4cb48316f37ee15a3dc584650877e`、证据 `2a9c671969723fa9afca569b6ba2a163cb01a7e4`。主回执 94 件、暂存预检 15 件的 Git 字节长度和 SHA-256 匹配，另核 22 条源码 blob 绑定。固定原始记录见[Windows 回交](https://github.com/gzers/opencodex-desktop/blob/2a9c671969723fa9afca569b6ba2a163cb01a7e4/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)。

| 项目 | 复核结果 |
| --- | --- |
| 测试制品与组件 | 七组原生定向 13/13。测试制品 MSVC raw0／291,297 ms，三个产品制品 raw0／227,953 ms。PowerShell DATA 11/11，保留 E0308／raw101 和暂存预检模块路径 raw1。 |
| 清理 | 三份 stdout 各 24 根，合计 72 个唯一根，与 cleanup.json 一致，均由 Windows 报告缺席。交接勘误取代顶层遗留计数 1。专用账户、Profile 和 Public 暂存根也由 Windows 报告缺席。 |
| 真实普通桌面 | 现有账户方案按[隔离根合同](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)补验。仍缺原 Owner V2 同线程 10 秒回执、完整原件恢复、普通用户 MSI 和官方单 SID 生命周期。 |

旧 r2 保留原件及锁，24 项 Job 为 Unknown／raw50、command-lineage raw64、executionReady=false。公开 Start 保持拒绝，W2／双 SID 待签名候选冻结。名称验收对象为用户确认的 ChatGPT 内的 Codex，[实际模型选择器合同](0.2.24-contracts/chatgpt-codex-name-validation-r1.md)仍待隔离环境和最终候选证据。


## revision194 Windows r7 原宿主扫描门槛复核

Windows 代码 `4fc73cd1c26aa51fa288a3b436d2f0c050d36e1c`，证据 `34e36638acd7e94a0f414c96972eb121522308c1`。净差限测试 PowerShell 启动器。核对 27 件清单载荷的原始 Git 字节、长度、SHA-256 和精确成员。启动器摘要 `ea4559116be9e4f37079e72b6cad81a7fa6982ba1b38e314eaaa287a1cb61cee`，复用 18833e7b 的 MSVC 测试制品。[固定证据](https://github.com/gzers/opencodex-desktop/tree/34e36638acd7e94a0f414c96972eb121522308c1/.adg/evidence/0.2.24/windows-dev/wl2-r7-initial-scan-gate-20261118-r1)

启动器放行后续管理器前校验同一次 Start 的七条已计量完整扫描。初始化 `read_complete` 位于该计量记录之外。DiagnosticAfter 再核原宿主三条准备扫描，绑定失败动作、原镜像摘要与同一 10 秒期限。扫描门槛 15/15，DATA 11/11。保留 PowerShell 5.1 数组层级、继承模块路径及本地化错误文字三类失败。最终断言为原始 Windows 错误码 80。六件失败归档字节和摘要均匹配，原三处失败及两处成功夹具根由 Windows 回执核实缺席。

本次接收启动器组件证据。普通桌面原 Owner V2、整链恢复、普通用户 MSI 与官方单 SID 生命周期仍待。旧 r2 继续隔离，公开 Start 维持拒绝。签名候选冻结后进入 W2／双 SID。

## revision195 Windows r7 隔离入口与完整扫描期限复核

共享 main `1c69bff16b1f6d1edf35c91c93640ee236894312`，Windows `b4e23087e5c9c90f0e0731453c0b1421b6be37c5`，证据 `ccc5e84286c3824f5ff663f891e76fc13405160b`。对 Windows 隔离 Owner、九次计量、目录期限及文件读取期限四轮核对 Git 文件成员、原始字节长度、SHA-256 和源码绑定。完整桌面准入与动作合同见 TEAM56／[r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。

| 范围 | 复核结论 |
| --- | --- |
| 真实原 Owner 的隔离测试入口 | 五项原生负例和三个产品形状开发制品编译通过。原失败保留。负例清理由 Windows 回执报告，未逐项列出夹具路径。Public 暂存保留。 |
| 九次计量接线 | 增加两处测试计量，MSVC 通过，worker 3/3、启动器 16/16。尚未取得真实普通桌面 Host 的九次计量。 |
| 已获得原期限的 Windows 扫描器 | 目录定向 4/4，固定官方镜像 10,586 文件约 5.65 秒。文件读取首测 raw101，已固定文件偏移由 1 移到 0；修复后 3/3，固定官方镜像完整核验约 3.94 秒。失败日志和文件读取失败夹具保留，成功夹具已清理。 |
| 共享入口 | LaunchPlan 构造的完整读取与 verify_full 传入同一原期限。完整读取在进入、扫描前和成功返回前检查期限，Windows 完整清单收到同一个绝对时间。Mac 定向 14/14 与 4/4；Windows MSVC 及第三处入口由 Windows 接续。 |

实际单 SID 桌面同一原 Owner 的 raw31／V2 附件、九次首启扫描、完整恢复、官方生产生命周期和普通用户 MSI 仍待。会话显示连接只作为线索，实际普通令牌、SID、桌面和无外层 Job 尚需实证。旧 r2 继续保全，公开 Start 维持拒绝。W2／双 SID 在签名候选固定后开始。


## revision196 Windows r7 第三处期限接线与扫描器集成

固定 Windows 源码 `8266cf6a6b67402719c89689984b2154c8cc4599`、证据 `b0eaf2e54a42cffe5298cad6d5892ba1c2b02379`。核对 32 件 Git 原始载荷，共 2,279,467 字节，清单精确成员与五项源码 blob／SHA-256 绑定一致。MSVC 测试库及三个开发制品编译通过。隔离及不可 Resume 5/5，原 worker／Status 5/5，扫描 8/8，PowerShell 5.1 启动器 19/19。测试制品摘要 `af9d31a7faaa9f364038fb3c492ba994c7445f7d36e19f7bc04834b109a92221`。[固定证据](https://github.com/gzers/opencodex-desktop/tree/b0eaf2e54a42cffe5298cad6d5892ba1c2b02379/.adg/evidence/0.2.24/windows-dev/wl2-r7-shared-start-deadline-20261118-r1)

共享 main `040f411e6d48d97bde6e7b2e6c22860e547b6173` 接收三份扫描器文件，文件内容与上述 Windows 输入一致。保留整树成员、固定对象、权限和哈希核验。枚举提示独立校验，长路径可沿原方式打开，到期停止派发并收束已起线程。Mac 定向回归 13 项通过，官方镜像独立测时项维持显式忽略。先前派发负例在已逾期后仍执行一次读取，基线 raw101，修复后原生定向 4/4。官方固定镜像单次完整读取 10,586 个文件及 manifest，8,242,335 微秒，不能外推九次扫描总时长。[派发期限证据](https://github.com/gzers/opencodex-desktop/tree/e00ad101ee36f4276dbbd1fea560fc928472e222/.adg/evidence/0.2.24/windows-dev/wl2-r7-dispatch-deadline-20261118-r1)

Windows 分支保留第三次入口和四文件定向变更。交接时普通桌面 session 7 断开，两个配对均无 Host 结果。按 TEAM57 承接九次实际扫描和原宿主 V2。旧 r2 保持保全。完整原件归还、普通用户 MSI、正式单 SID 生命周期、W2／双 SID 和 ChatGPT 内 Codex 的实际选择器仍待验收。

## revision197 Windows r7 真实桌面首错与修复配对

| 范围 | 复核结论 |
| --- | --- |
| 首轮真实普通桌面 | session 7 的实际普通账户在 WinSta0/Default、无外层 Job 下启动 Host。PID 95724 在 6,460 ms 拒绝，首错 raw32，Status 为 Unreachable。三次入口完整扫描各 10,586 文件，分别为 2,154,243、2,114,464 和 2,184,376 µs。六次 worker 扫描、raw31 和后续 5／10／5 秒动作均未到达。Host 和失败现场保全。 |
| 隔离预检修复 | 测试夹具的严格叶句柄与完整预检请求 DELETE 的句柄冲突。daae8ffff7f304ec28b9c0214db26691e930a395 固定父范围和两个精确子句柄，完整预检后重查谱系与保护，再恢复严格叶句柄。失败保留已持有的子句柄并禁止复用描述。生产预检未变。MSVC 定向 6/6、595 ms，三个开发制品无夹具标记。接收组件。 |
| 原始回执核验器 | e19642724af81cc40c3dea6dbe4dc7d4caee9d21 只增加只读核验器。回执校验源码 SHA-256 f22ed1851e33a77a3557d7a57802e9c0f2c0a9aed501ea30b031f486a3b7e6b1 与 Git 字节一致。原始 r1 预期拒绝 raw32，合成 1 个通过、8 个拒绝。工具交叉核对 stdout、私有导出、进程回执、九条同期限计量及原 worker 和锁范围摘要。当前不证明宿主仍在线或后续仍持有范围。此轮四件记录由 Git 固定，未另附独立制品清单。 |
| 修复版真实桌面 | Windows 回执记录 out01 为空，待按 TEAM58 固定配对执行。旧原件恢复 r2 仍受 24 项 Job Unknown/raw50 和命令谱系 raw64 限制。继续保全三件原件及锁。普通用户 MSI 仍为 raw1625。 |

[首轮桌面与预检修复证据](https://github.com/gzers/opencodex-desktop/tree/3fa3c1c1de88905257fc7ddbb4ea4f0130a32032/.adg/evidence/0.2.24/windows-dev) · [只读核验器证据](https://github.com/gzers/opencodex-desktop/tree/a223e560670550b43c6bd042833ebc1d4a01fccf/.adg/evidence/0.2.24/windows-dev/wl2-r7-host-receipt-gate-20261118-r1)

公开 Start 继续拒绝，强制清理不能证明正常 Stop 或 Closed。ChatGPT 内 Codex 是渠道显示的实际验收入口。保存、磁盘读回、官方刷新和回退已有证据，实际客户端选择器仍待隔离验收。普通用户安装、生产单 SID 生命周期和签名候选仍待闭合，W2／双 SID 等候选固定后开始。


## revision198 Windows r7 外部恢复单文件与成对预检

| 组件 | 复核结论 |
| --- | --- |
| 单文件恢复原语 | 源码 `3ef31c9caae35f4d7cc6083de27bdf106ca0b560` 限于四个测试文件。12 件原始载荷共 34,024 字节，源码输入与载荷均与 Git 一致。最终 11/11，raw0，3,473 ms。涵盖独占意图与读回、同卷句柄移动、文件 ID／字节／八项安全投影，以及目标冲突 raw183、已有意图 raw80、共享冲突 raw32。四个隔离根报告已清理。 |
| 两文件预检 | 源码 `0e9f9cf486425d9fa4ba70ddcef376340f1abe5e` 相对上一条仅改测试文件 `external_repair_steps.rs`。源码 blob `bb58682b550a5afeefadb3023b0d846c33069c84` 与清单一致，核对八件原始载荷共 22,692 字节。MSVC x64 两轮各 7/7，raw0，3,421／3,524 ms。首次写入恢复意图前固定两个同卷文件句柄，核查四目录精确成员和八项安全投影。额外成员在写意图前拒绝，末件共享冲突保留 raw32 和夹具内容。清单记录 14 个隔离根均已清理。 |

[单文件证据](https://github.com/gzers/opencodex-desktop/tree/65e58f12d5ea3952cd63fc0e7230c8c70a8e816c/.adg/evidence/0.2.24/windows-dev) · [成对预检证据](https://github.com/gzers/opencodex-desktop/tree/87e664831c5774ee099a1d5929e0d217da03bf44/.adg/evidence/0.2.24/windows-dev/wl2-r7-external-pair-preflight-20261118-r1/)

## revision199 Windows r7 外部整树恢复与宿主回执核验

Windows 代码 a7d7284a3280d973aaacddf361adae6f6dcc8a3c，交接 f9bf34095e06e1fd5e267be89356e57fb96406e5。按固定 Git 对象复核各组清单、长度及 SHA-256。外部恢复相对 IMP198 增量限五个测试源码路径，其中一个为产品模块的测试代码，生产接线待做。

| 回交 | 复核结果 |
| --- | --- |
| 成对文件与原生租约 | 2afe0fa98，25 件证据，54 个清理标记。K 占用时首个 Intent 前 raw33，释放后独立重新获取。 |
| 整树移动 | bcf533209，19 件证据、80,736 字节。宿主 10/10，产品定向测试 12/12。六次移动、十二条刷盘记录、八个安全投影。raw183 保持七条记录的精确前缀。 |
| 整树租约 | 82f48aa05，14 件证据。宿主 10/10，产品定向测试 13/13。26 个检查点验证 raw33 与实际释放后重新获取。保留首次 E0502。 |
| 整树记录绑定 | 6e4e2e274，16 件证据、67,751 字节。宿主 10/10，产品定向测试 13/13。验证六步记录绑定、异常记录拒绝、中断前缀、精确读回及租约释放。保留首次 E0463。 |
| 宿主回执核验 | 最新代码净差只改回执核验脚本。九件原始证据共 13,862 字节。核验器重建 13 字段镜像清单摘要并绑定配对输入。合成正例及六项拒绝通过，旧脚本接受伪造 Node 摘要的回执已保留。真实 nine-r1 仍是 raw32，保留首次 Python 启动 raw9009 与后继 raw0。 |

nine-r2 继续使用已固定的源码与二进制配对 daae8ffff7f304ec28b9c0214db26691e930a395，新脚本独立核验。截至复核，out01 尚无回执。原 nine-r1 的 PID 95724 在 6,460 ms 后 raw32，仅三次入口扫描，未启动 worker。旧官方 r2 另有三件原件处于保管，24 个 Job 成员 Unknown/raw50，谱系 raw64，executionReady 与 approvalRequested 均为 false，继续保全。普通用户 MSI raw1625、原预算完整 Stop/Closed、生产单 SID 与固定候选仍待完成。公开 Start 保持拒绝。

[固定 Windows 交接](https://github.com/gzers/opencodex-desktop/blob/f9bf34095e06e1fd5e267be89356e57fb96406e5/docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md)

上述为组件通过。旧 r2 的三件真实原件和锁继续保全，24 项 Job 仍是 Unknown／raw50，命令谱系 raw64，`executionReady=false`，普通用户 MSI 为 raw1625。生产恢复仍需证明整树成员、进程已静止、claim/N/L/K 连续持有、持久记录、精确锁释放和原件正常归还。强制清理不能签署 Stop 或 Closed。真实桌面沿 TEAM58 的原配对，`nine-r2/out01` 尚无 Host 结果。

## revision200 Windows 释放后漂移复核与实际锁范围准入

组件源码 def21dfe1c3645c8d838b1ec01727b8dc392f546，固定证据 582e759fbd160ef1b0052d315fc7aff338f3288f。本机复核原日志、ZIP 总摘要及 24 件成员摘要和 28 条缺席记录。释放后修改归还的 config.toml，原端点仍允许写 completion.json，回归 raw101。修复后重查全成员、锚点、归还与退役对象身份、八项安全投影、字节和持久记录，拒绝漂移并保留十二件移动记录。MSVC 定向 14/14、raw0。EXE 摘要及测试根缺席均为 Windows 回执。[固定证据](https://github.com/gzers/opencodex-desktop/tree/582e759fbd160ef1b0052d315fc7aff338f3288f/.adg/evidence/0.2.24/windows-dev/wl2-r7-post-release-drift-20261118-r1)

新源码 f3564fbb34713beee97ed221fb4797b422b11d57 仅改测试 harness 的 external_repair_whole_tree.rs，把已准备节点及持久移动记录绑定到原 WholeTree，增加交叉夹具拒绝用例。净差及空白检查已复核。最新证据 ref 2e08a4463ec572c08787402cb8453b56a760a0f2 尚未绑定这份新源码的执行结果。

固定 nine-r2 真实普通会话允许一次 Host。当前 DiagnosticBefore 提供元数据与 Status 投影，实际范围仍需原线程检查。旧 nine-r1 PID 95724 独立观察 raw5，进程缺席与同 FILETIME 均未证明。Explicit 需独立进程代次证据和原 worker 的新鲜只读范围证明。具体合同见 TEAM61 和 [r7](0.2.24-contracts/windows-preresume-recovery-performance-r7.md)。旧 nine-r1 和独立的官方 r2 保全，生产恢复、普通 MSI、单 SID 生命周期及签名候选仍待验收。
## revision201 接收 Windows r7 原树绑定回归

Windows 测试源码 f3564fbb34713beee97ed221fb4797b422b11d57，证据 e479ab8a5b657fe90987868a109378ffad6b8f46。独立复核清单文件集合、大小和 SHA-256，修复前快照仅向 def21dfe1 插入两项反例，两个修复后快照均与所交源码一致。四份失败归档的总摘要和 61 个成员的大小与摘要一致。

修复前两项失败，raw101，2,927 ms。外树动作能写入完成记录，交换计划能移动源树 auth.json 并在目标留下四份记录。修复将已准备节点和持久移动记录绑定其原 WholeTree。MSVC 夹具 12/12、raw0、3,503 ms，链接产品库 16/16、raw0，实际测试 1.49 秒，含构建计时 222,171 ms。接收范围为合成夹具。Windows 报告 32 个夹具根已核缺席，四个已归档失败夹具和一个未跟踪生成 schema 因权限策略保留。本机复核 Git 证据和归档，物理清理状态取自 Windows 回执。

普通桌面仍无 nine-r2 合格 Host 回执。按 TEAM62 继续定向，保全 nine-r1 与独立旧官方 r2。普通用户 MSI、生产单 SID、正式恢复和签名候选仍待验收。公开 Start 保持拒绝。[固定证据](https://github.com/gzers/opencodex-desktop/tree/e479ab8a5b657fe90987868a109378ffad6b8f46/.adg/evidence/0.2.24/windows-dev/wl2-r7-external-tree-binding-20261118-r1)

## revision202 接收原 Start 线程范围探针草稿

Windows 核验器源码 673d1166334c4fb78d5094bc09ef130e578e0991，证据 644f8bc861bd7a1009aed9c5789699ff569f0895。复核 Git 对象 36 项文件长度与 SHA-256。核验器净差限测试脚本，其对固定 nine-r2 回执的适用性待证，现有 nine-r2 沿用 a7d7284a3。探针是 sourceCommit:null、productionAccepted:false 的测试草稿。最终 MSVC 定向 20/20，非测试生产库 cargo check raw0。PowerShell 启动门槛 1 项有效样本通过、10 项改写拒绝。测试 EXE SHA-256 为 1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8。原 LNK1104 保留。Windows 报告 268 个临时根已核缺席，两份合成夹具已归档并因删除被拒保留原目录。

最新交接时 nine-r2/out01 无 Host 回执。按 TEAM63／r7 采集固定配对单次 Host，旧制品缺原线程探针，取证后停在 Explicit 前。含探针的后继配对待绑定源码、制品和全量暂存清单，再执行真实普通桌面定向验证。旧 nine-r1 首错 6,460 ms／raw32，三次扫描且无 worker。独立旧官方 r2 三件原件仍在 custody，24 个 Job 成员 Unknown／raw50，命令谱系 raw64，executionReady=false。普通用户 MSI raw1625、生产单 SID、完整恢复和固定候选待验，公开 Start 维持拒绝。

[固定证据](https://github.com/gzers/opencodex-desktop/tree/644f8bc861bd7a1009aed9c5789699ff569f0895/.adg/evidence/0.2.24/windows-dev/wl2-r7-original-range-probe-20261118-r1)

## revision203 接收 Windows 原线程探针固定配对

固定源码 `36b70383eac5c44dffc51d34a4c4673cd8a3dbf7`，证据 `007dd301f16e47ed9afd59f91a17b3687e8b4dd7`。独立复核清单的 25 份载荷与 Git 树文件集合、长度和 SHA-256，并核对九项源码输入与提交对象，包含清单声明的换行规范化。Windows 暂存退出 raw0，随后独立读回六份暂存文件、两级目录、ACL 与 SHA-256。读回时 Profile 夹具缺席，两个 out01 均空。首次读回对 nine-r1 的扫描次数误计为一次，最终读回按原日志校正为三次。原生桌面、普通令牌、Job 和进程代次仍待现场证明。

固定测试配对位于 `C:/Users/Public/ocx-r7-isolated-owner-range-r1`。EXE SHA-256 `1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8`，运行时 SHA-256 `14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7`。源码改动沿 TEAM63／r7 的测试专用范围，三个路径常量 BASE、PUBLIC、BUNDLE 与启动器同批绑定。此处接收制品与暂存证据，真实 Host 与 5／10／5 秒动作按 TEAM64 取证。

nine-r1 在 6,460 ms 后 raw32，仅三次完整扫描且未建立原 worker。PID 95724 的 raw5 无法确认身份或缺席。独立旧官方 r2 三件原件仍在保管，24 个 Job 成员 Unknown/raw50，命令谱系 raw64，executionReady=false。普通用户 MSI raw1625、原件恢复、生产单 SID 及签名候选仍待验收。公开 Start 保持拒绝。[固定证据](https://github.com/gzers/opencodex-desktop/tree/007dd301f16e47ed9afd59f91a17b3687e8b4dd7/.adg/evidence/0.2.24/windows-dev/wl2-r7-probe-staging-20261118-r1)

## revision204 接收 Windows nine-r2 单宿主诊断

证据提交 cdf54c6a1cd095f8421c5e07a14929c30d6cc913，测试源码配对 daae8ffff7f304ec28b9c0214db26691e930a395，镜像 SHA-256 ab282752940b875b98e5e957a102ee8f4b1d32829f7524eb2dbe45959defc030。独立复核全部 14 项清单成员、字节数与 SHA-256。归档核验器与 a7d7284a3280d973aaacddf361adae6f6dcc8a3c 源码逐字节一致，本机离线重算退出码 0。

原生回执记录普通用户 marenben 的 session 7，WinSta0/Default，无外层 Job。九次完整扫描用时 20,064 ms，在首次 Intent 前返回 raw31 windows024_v3_capture_required。注入时两棵树已捕获，两个 Intent/文件对均缺席，Node 从未 Resume。宿主 PID 133584、创建 FILETIME 134394870078415414、worker ThreadId(51)。离线结果的 liveProcessVerified 和 laterRangeContinuityVerified 均为 false。该轮保留原现场，止于宿主诊断。

后继按 TEAM65 使用 IMP203 的固定 range-r1 配对独立补验。nine-r1 的 PID 95724 仍为 raw5，身份与缺席未知。独立旧官方 r2 仍保有三件原件，24 个 Job 成员 Unknown/raw50，谱系 raw64，executionReady=false。普通用户 MSI raw1625、生产恢复、生产单 SID 与签名候选仍待验。公开 Start 保持拒绝。

[原始证据](https://github.com/gzers/opencodex-desktop/tree/cdf54c6a1cd095f8421c5e07a14929c30d6cc913/.adg/evidence/0.2.24/windows-dev/wl2-r7-nine-r2-host-20261118-r1)

## revision205 接收独立回读计时缺口及定向观察脚本

证据 b0684d0a9 的实证交叉检查中，range-r1 核验器 04109655081281036a20f7965016007ffb5c7a6b2560ab8fc8b14855174ccc9d 接受按原路径和镜像摘要绑定的 nine-r2，拒绝将同一回执用于 range-r1，存活身份和后续范围连续性仍未证实。旧 launcher 只计管理器响应；独立回读 44 ms 与注入后 5,107 ms 均通过，证明计时没有覆盖实际附件核验。

复核 f2dc1b60fa2215f3102347953f1d26cf363d9ff0 清单 37 项，Git 字节及摘要一致。观察脚本的合成案例覆盖及时完成、5,309 ms 上界超限、launcher 非零保留首错、动作错绑和缺回执；控制端 session 2 拒绝。接受 [r7 增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 的测试专用保守计时和绑定条件。range-r1 尚无真实 Host 回执，观察脚本尚未暂存，现场结果待 TEAM66。原设定的期限、范围和拒绝条件继续适用。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/f2dc1b60fa2215f3102347953f1d26cf363d9ff0/.adg/evidence/0.2.24/windows-dev/wl2-r7-readback-timing-observer-20261118-r1)

## revision206 接收独立进程观察和计时脚本暂存

源码仍为 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7。独立复核 44502ee7c2d3038e37a313c86378c1724378c004 的 14 份与 b12b85232221cfdac3d8c0042a4131ee05e3b47b 的 4 份清单载荷，路径集合、字节数和 SHA-256 均一致。进程观察在 PowerShell 5.1 下接受精确 PID、FILETIME、会话和镜像并拒绝不匹配。控制端查询 nine-r2 返回 raw5，在普通桌面准入返回 raw1。独立观察入口已暂存于 `C:/Users/Public/ocx-r7-range-observers-20261118-r1`，现场输入为已核验的 Host-result.json 摘要，原失败和 claim/N/L/K 仍由原 worker 在 DiagnosticBefore 内确认。

指定的计时脚本已原样暂存于 `C:/Users/Public/ocx-r7-isolated-owner-range-r1/observe-r7-readback-timing.ps1`，6,394 字节，SHA-256 为 5567494d2174270d77bac6f862ab789c0d43eb3aaab7689e4e5a20e771a2f426。暂存 raw0，stderr 为空，原六份文件及 ACL 已复核。回执中 range-r1/out01 为空，现场 Host、DiagnosticBefore、Explicit 与 DiagnosticAfter 仍待。沿 TEAM66 的原期限和固定配对推进。普通用户 MSI、生产恢复、正常 Stop/Closed、生产单 SID 和签名候选仍待；保全原失败与旧现场。

[进程观察证据](https://github.com/gzers/opencodex-desktop/tree/44502ee7c2d3038e37a313c86378c1724378c004/.adg/evidence/0.2.24/windows-dev/wl2-r7-host-live-observer-20261118-r1) · [计时暂存证据](https://github.com/gzers/opencodex-desktop/tree/b12b85232221cfdac3d8c0042a4131ee05e3b47b/.adg/evidence/0.2.24/windows-dev/wl2-r7-team66-timing-stage-20261118-r1)

## revision207 接收 Windows 回读脚本配对证据

Windows 源码仍为 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7。复核证据 70c6d7f4f972ff93edd3d8b35ace4e5aad25cfe5 与 896bbd71105a08562943d56d39230db368486d7e，两份清单共 12 个载荷和 2 个归档文件的大小与 SHA-256 一致。PowerShell 5.1 合成探针证实原只读打开遇保留写句柄返回 raw32，修订后独立读取准确的 3,523 与 3,669 字节并核语义及摘要链。第二写者仍 raw32，重复导出 raw80 且保留原字节，动作错绑、额外成员、内容变更均拒绝。两份完整配对均通过摘要门，混配及 CMD 漂移均拒绝。Mac 从 Git 原字节独立重算同一修订摘要。

按 [r7 配对增量](0.2.24-contracts/windows-preresume-recovery-performance-r7.md) 与 TEAM68 固定实际暂存字节后再进入 Explicit。现有 Host 回执可直接消费，未有则只跑一次；原 worker 的范围证明能与脚本落地并行。合成结果只确认共享模式和脚本摘要门，现场 5 秒尚未证实。普通用户 MSI、生产恢复、正常 Stop/Closed、生产单 SID 和签名候选继续保留待验。

[独立回读及配对证据](https://github.com/gzers/opencodex-desktop/tree/896bbd71105a08562943d56d39230db368486d7e/.adg/evidence/0.2.24/windows-dev)

## revision208 接收 Windows 实际暂存与配对预检

复核源码 a8d7a08002c3af21b2c7ad8f9bcc06cda9d107d0 与证据 bd29ce9c75a259aa845d153f732bf3e8ddd2cccc。17 份载荷的路径、大小及 SHA-256 一致。暂存前后各七项，仅替换两份脚本，文件 ACL 一致，三处目录 ACL 均受保护。源码净差仅为测试 launcher 的三处替换。

实际 launcher 为 30,158 字节，SHA-256 为 6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990。证据分支中的观察脚本为 6,394 字节，SHA-256 为 f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f，绑定实际 launcher 和原 CMD。PowerShell 5.1 接受实装配对，混配拒绝并报告 `The fixed DiagnosticAfter launcher bytes differ.`。首次准备 raw1 与 stderr 已保留，修正后的准备、暂存与预检均 raw0，stderr 为空。

本次接收暂存与配对预检。range-r1 预检时没有 Host 回执，也未执行桌面动作。现场沿 TEAM69 推进。普通用户 MSI raw1625、生产恢复及正常 Stop/Closed、生产单 SID 和签名候选继续待验。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/bd29ce9c75a259aa845d153f732bf3e8ddd2cccc/.adg/evidence/0.2.24/windows-dev/wl2-r7-team68-readback-stage-20261119-r1)


## revision209 Mac 已取消前代的后继根取消通过

以 main 040f411e6d48d97bde6e7b2e6c22860e547b6173 和实际执行差异 SHA-256 33ffdfa284a37859e6e71a7e5eb002187a8f7f4b4d6787a3fd5eeda5903642a8 为输入。原样官方 Darwin 2.99 下，生产取消入口将已取消前代的后继从 Preparing revision 4 提交到 RootCancellationAbsent revision 6，保留后继票据及前代归档，并核验再次显式 Start 的资格。前代取消 7,018 ms，后继取消 8,634 ms，均在各自 10,000 ms 预算内。拒绝归档漂移、过期审查、错误 revision 或摘要。两次 Guard 依次到达 held、prepared、relocated、released。独立夹具、进程组及本轮专用协调文件已清理。

本轮覆盖已取消前代的生产动作。普通 RootAbsent 前代、已登记计划显式继续及固定候选的 Start → Ready → 正常 Stop 仍待验。服务未启动，releaseAcceptance=false。真实客户端已确认为 ChatGPT 内的 Codex，最终选择器继续沿既定隔离合同验证。

[动作及绑定证据](../../.adg/evidence/0.2.24/mac-successor-cancel-action-r1/evidence-manifest.json)

## revision210 Mac 普通关闭前驱的后继根取消通过

在合成普通 RootAbsent revision 5 的独立夹具中，使用原样官方 Darwin 2.99 执行生产取消。后继从 Preparing revision 6 到 RootCancellationAbsent revision 8，耗时 8,414 ms，预算 10,000 ms。保留前驱归档与后继票据，拒绝归档漂移、过期审查及错误 revision 或摘要。Guard 经过 held、prepared、relocated、released；独立夹具、进程组和本轮专用协调文件已清理，既有全局生命周期锁保持。下一次显式 Start 的资格检查通过，未实际启动服务。本轮不构成整版验收。

已登记 RootCancellationPrepared 的显式续接、固定候选的 Start → Ready → 正常 Stop，以及 ChatGPT 内 Codex 的真实选择器仍待验证。Windows 分支仍为 TEAM69／IMP208 已记录的两项 SHA。

[成功证据](../../.adg/evidence/0.2.24/mac-ordinary-successor-cancel-action-r1/evidence-manifest.json) · [独立构建失败证据](../../.adg/evidence/0.2.24/mac-ordinary-successor-cancel-build-failure-r1/evidence-manifest.json)


## revision211 Mac 后继取消代码检查点与回归复跑

后继取消、界面确认和定向验证入口已提交并推送 main cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2。20 个文件的执行净差 SHA-256 为 f82a9b54f1d2d45f6baec5083904a7024f9e54de1e3af23078604fc9e6067063。前述三份证据逐文件核对长度和 SHA-256。

串行回归首轮 2,627 通过、21 失败、59 忽略。失败集中于运行包安装、路径和卸载，原因是夹具落在受保护的 /private 前缀。用同源码构建的隔离测试宿主在 /Users/Shared 专用临时目录复跑失败项，21 项全通过。校验策略未改。原失败和复跑日志均留在本机 /private/tmp/ocxd024-regression.Z95TuT/，已确认复跑退出码 0、进程组缺席及夹具删除。21 项复跑单列，仍以两轮原始结果作证，不宣称另跑了全绿的全量测试。

该提交为共享检查点，releaseAcceptance=false。已登记计划的显式续接、固定候选正常启停、ChatGPT 内 Codex 实屏选择器和 Windows 生产验收继续开放。

## revision212 Mac 归档根原协调锁重取原语通过

使用 main cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2 与试验差异 SHA-256 0ab947ab8f2e32389c0b3c9b9124e2c802cb43494d9e57dcdac48a01d04c9a89。已核对构建输出中的测试可执行文件，实测摘要为 3735237073f6055a4851a250d786f6dce0f9051083345f9b64efa9a0d8aa557e。使用原样官方 Darwin 2.99，归档后重取原 Claim/N/L/K、进程外核对并正常释放耗时 14 ms，预算为 10,000 ms。锁争用、逻辑根重现、inode 替换、保护属性变化、owner 回执变化均拒绝进入回调。进程组、夹具和两个专用协调文件已核缺席，既有 UID 级 L 保持。

此为测试专用原语。独立计划的生产续接、四个中断阶段、真实启停与整版验收继续待验。本轮未启动服务。

[执行差异及证据](../../.adg/evidence/0.2.24/mac-native-reacquire-spike-r1/evidence-manifest.json)


## revision213 接收 Windows range-r1 原宿主诊断回执

接收 Windows 源码 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756、证据 b130bf93eaefb7f7a05494fa1f4571104b3958b3。源码双亲为已接收的 a8d7a08002c3af21b2c7ad8f9bcc06cda9d107d0 与 main cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2，独立重算合并树完全一致。本批证据新增归档及交接说明；从 Git 原始字节独立核对清单中 14 份载荷的长度及 SHA-256。固定现场 EXE 源码仍为 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7，SHA-256 为 1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8。

marenben 普通令牌、session 7、WinSta0/Default、无外层 Job。原 30 秒内九次完整镜像扫描走完，20,820 ms 到达预定首次 Intent 前 raw31，Node 未 Resume。核验器退出码 0，Host PID 25144／FILETIME 134395302283231479，原 worker ThreadId(51)。Host-result.json SHA-256 为 97d40caf5661aec2c16464d32d7ec4cbbff608d4e2d2b0ee757972c3fbac7346。保留首次归档共享冲突和后继只读快照。本轮只通过原 Host 诊断，独立活进程代次、原 worker 持锁连续性、Explicit、DiagnosticAfter、真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID 和签名候选仍待验。现场交接见 TEAM70。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/b130bf93eaefb7f7a05494fa1f4571104b3958b3/.adg/evidence/0.2.24/windows-dev/wl2-r7-range-host-20261119-r1)

## revision214 接收原 worker 与固定脚本验证

接收 Windows 源码 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756、证据 d88b1839e456751268ae2bf741cca64e665c548e。按 Git 原始字节核对 manifest 列出的 12 份文件，包含 .gitattributes。普通账户 marenben、SID 后缀 -1003、session 7 的独立原生查询确认同一宿主仍存活，PID 25144、FILETIME 134395302283231479、固定影像与原 Host 回执一致。原 ThreadId(51) 的 DiagnosticBefore 在 114 ms 内返回 raw0，证实原 raw31 FailedRetained/Capture、动作、claim/N/L/K 及原件延续。首次 Start 的期限已过，Node 从未 Resume。

实际暂存文件通过 ACL 和 reparse 核验。Launcher SHA-256 为 6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990，观察脚本 SHA-256 为 f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f。实际配对通过，混用旧件被拒。取证时未见后继动作回执。Required/Unknown 保留，runtimeAuthority、publicStartOpen、normalStop、closed 均为 false。

本轮满足定向 Explicit 的诊断前置。Explicit、独立 DiagnosticAfter、真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID 与签名候选仍待验。现场决策和输入见 TEAM71。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/d88b1839e456751268ae2bf741cca64e665c548e/.adg/evidence/0.2.24/windows-dev/wl2-r7-team70-original-worker-20261119-r1)


## revision215 接收单次 Explicit 超时并锁定只读取证

接收 Windows 源码 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756、证据 85f317434aab0663c5d282ee5edb0bdfb4532f0a。从 Git blob 独立核对 manifest 的 13 份载荷、长度、SHA-256 及目录成员。原 114 ms DiagnosticBefore 继续有效。单次 Explicit 于 9,999 ms 返回 TimedOut、raw=null，测试 unwrap panic 产生 raw101，launcher 首错为 Manager transport or refusal authority differs.。无有效原生回复、recoveryActionId 或 DiagnosticAfter，验收核验器为 passed:false。

请求是否准入、原 worker 是否收到、镜像扫描覆盖及 V2 Intent/Pending 持久状态仍为 Unknown。控制账户读私有日志得到 raw5。保留原现场和首错，按 TEAM72 与 r7 只取同 SID/session 的既有私有日志快照。真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID、签名候选及 ChatGPT 内 Codex 实际选模显示仍待验证。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/85f317434aab0663c5d282ee5edb0bdfb4532f0a/.adg/evidence/0.2.24/windows-dev/wl2-r7-team70-explicit-20261119-r1)


## revision216 接收原桌面部分取证

接收 Windows 证据 832378a92959783c11d99dcad4d4aef1a2722d2a，代码仍为 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756。按 Git blob 核对 manifest 三份载荷的长度和 SHA-256。原普通 SID 后缀 -1003、session 7、WinSta0/Default 两次确认 PID 25144／FILETIME 134395302283231479 存活。文件身份和保护稳定，两次独立读取 [0,5622) 摘要均为 4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819。

快照首错在 private-copy（RuntimeException，HRESULT -2146233087，raw=null）。缺少事件投影，原 Explicit 的 dispatch、扫描、V2 Intent/Pending、最末阶段和最终回传仍为 Unknown。按 TEAM73 与 r7 在同一环境做只读内存投影，保留原失败。真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID、签名候选及 ChatGPT 内 Codex 实际选模仍待验。

[Windows 原始证据](https://github.com/gzers/opencodex-desktop/tree/832378a92959783c11d99dcad4d4aef1a2722d2a/.adg/evidence/0.2.24/windows-dev/wl2-r7-team72-event-snapshot-20261119-r1)


## revision217 接收 Windows 日志投影与计时覆盖分析

证据固定至 ba54ebf76f7fff7a438606a329505da237118778，生产输入仍为 9a49ea219e8f34dfa1867ecebfc6050fa6dcb756。独立按 Git blob 核对投影 manifest 六份载荷的字节数和 SHA-256，以归档核验器复算回执并与入库 JSON 逐字节一致。原普通 SID 后缀 -1003、session 7、WinSta0/Default 下，PID 25144 与 FILETIME 134395302283231479 绑定同一日志稳定前缀 [0,5622)。来源摘要 4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819，脱敏投影摘要 8044e1de42b1f3ff21ddcfa0c45edc0c3ded4944f4cb41829de1f9a22dff4942。核验覆盖归档载荷与来源声明，私有原始字节保留现场。

同次 Explicit 的一条 intent 及一条 result 证明服务端至 dispatch 返回后记录 refused / TimedOut / raw=null。结合固定源码，预 dispatch 期限检查通过。114 ms DiagnosticBefore 的 ThreadId(51) 仅绑定该观察。Explicit 的 worker 接手、各独立扫描、V2 Intent/Pending 和最后回复 I/O 保持 Unknown。零计时事件不足以定位耗时阶段。

保留原 private-copy 首错、9,999 ms 超时和 raw101。Node 未恢复执行，三个原件仍在 custody，claim/N/L/K 仍持有。Windows 推送后待命，原现场只读。Mac 继续已登记取消的显式续接。真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID、签名候选及 ChatGPT 内 Codex 实际选模仍待验。

[只读事件投影](https://github.com/gzers/opencodex-desktop/tree/ba54ebf76f7fff7a438606a329505da237118778/.adg/evidence/0.2.24/windows-dev/wl2-r7-team73-event-projection-20261119-r2) · [计时覆盖分析](https://github.com/gzers/opencodex-desktop/tree/ba54ebf76f7fff7a438606a329505da237118778/.adg/evidence/0.2.24/windows-dev/wl2-r7-team73-timing-coverage-20261119-r1)


### IMP218 Windows 首次 Resume 前超时定向计量

承接 IMP217。已知原结果为服务端 dispatch 返回 TimedOut，原 worker 是否接手及 V2 持久状态仍未知。采用全新隔离配对，对同一原期限记录有界阶段前缀。正式验收缺项保持开放。固定 Windows 源码和证据、排他写集及单次验证输入见[定向诊断合同](0.2.24-contracts/windows-preresume-recovery-timing-r1.md)。

### IMP219 接收 Windows 计量证据并下发新隔离序列

接收源码 6532cbb6e612f2cc4e21684f685b28c92279ee51、证据 32b0483e33fef37370bc13d9c0807edb151c9eb5。按 Git 对象核对父提交、主线祖先、六文件写集、测试树 7ee052909df0c1e478cad78c8d33a98b00e667e6 及证据净差。Windows 回交记载 MSVC 测试构建和非测试库检查 raw0、定向断言 12/12，EXE 摘要 83fec3f05a6c356c25e0585b2199d15914baee30d4ad481080a9c5669adbd802。原日志和 EXE 留存在 Windows，本次核对范围为 Git 对象与记录的元数据。

新真实桌面序列仍待执行，首次 Explicit 超时的具体阶段为 Unknown，`accepted:false`。按 TEAM76 及[定向诊断合同 revision 2](0.2.24-contracts/windows-preresume-recovery-timing-r1.md)扩展最小测试写集，同一新样本保留原 30／10／5 秒与未知拒绝。真实原件归还、正常 Stop/Closed、普通用户 MSI、生产单 SID、签名候选与 ChatGPT 内 Codex 实际选模仍待验证。

[Windows 定向证据](https://github.com/gzers/opencodex-desktop/tree/32b0483e33fef37370bc13d9c0807edb151c9eb5/.adg/evidence/0.2.24/windows-dev/wl2-r7-team75-preresume-timing-20261120-r1)

### IMP220 Windows 待命及 Mac 停止诊断

截至本轮核对，Windows 远端源码为 6532cbb6e612f2cc4e21684f685b28c92279ee51，证据为 32b0483e33fef37370bc13d9c0807edb151c9eb5。定向 MSVC 结果沿用 IMP219。首次 Explicit 的具体耗时阶段仍为 Unknown，原件真实归还、正常 Stop/Closed 和正式候选验收保持开放。按 TEAM77 等待下次明确交接。

Mac 隔离实屏已验证取消和后继 Start 就绪。该后继的 Stop 约 10.8 秒，服务及监听已退出，原件恢复在 root-close 返回未确认，索引保留 root_absent rev12。该轮保留失败，测试清理单列，`accepted:false`。Mac 定向核验提交后读回、资格投影、最终期限检查及超时降级，依据首错决定修复位置。Start／Stop／读回 30／10／5 秒、完整检查和未知拒绝保持。独立客户端列表实测仍待 ChatGPT 内 Codex 复验。

### IMP221 Mac 第二代 Stop 超时与索引降级

同轮两代真实启停中，第一代 Stop 7,281 ms，第二代 11,642 ms。两代原生进程及监听退出，第二代提交 root_absent rev18 后降级为 root_retirement_prepared rev19，对外返回 client_original_restore_pending，accepted:false。前代归档无变化，测试残留清理单列。一次检查点内合并归档读取仍不足以满足第二代期限。接下来分解恢复与提交后证明耗时，保持 30／10／5 秒与独立读回。见[失败证据投影](../../.adg/evidence/0.2.24/mac-root-stop-archive-observation-r1/result.md)。

### IMP222 Mac ARM64 Release 两代正常启停通过

在 cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2 加冻结补丁、官方 OpenCodex 2.99.0 的隔离 ARM64 Release 原生宿主中，两代 Start 分别用时 7,278 ms 和 9,049 ms，正常 Stop 用时 5,317 ms 和 8,152 ms。独立 Status 均满足 5 秒预算，终态分别为 root_absent rev9 和 rev18。原件恢复、独立读回与前代归档完整性通过。测试进程退出 0，本轮夹具、进程、监听及所属协调器已清理。IMP221 的 Debug 失败独立保留。整版 accepted:false。[原生验证证据](../../.adg/evidence/0.2.24/mac-release-native-root-cycles-r1/result.md)

### IMP223 扩展落点边界修正

拓展页是所有权收缩后唯一还会写真实用户客户端配置的链路，其中 Codex 一格直接写真实 ~/.codex/config.toml 与 ~/.codex/skills。两处问题：与「真实 .codex 只读」冲突；运行中的服务以 CODEX_HOME=<OPENCODEX_HOME>/codex-client 启动，写进 ~/.codex 的 MCP 定义不会被读取。

代码 8b450a3fe 把 Codex 落点改到 Desktop 自管根，其余五个客户端保持各自用户级配置。ClientTarget 拆出 anchor（写入边界信任锚）与 home（落点所属用户主目录）；managed_codex_root() 只从已准入的数据根记录推导，绑定不成立返回只读视图；client_targets() 成为唯一目标矩阵入口，命令层、技能投射、同步、迁移、模型配置比较、便携资源和可打开路径统一使用。写入校验改为「落在各自 anchor 内」，外置 OPENCODEX_HOME 只在锚与数据根记录完全一致时放行。home_from_targets() 不再从 skills_dir 上溯推主目录。

另收一处：数据根直接挂在共享系统临时父目录下时不再绑定自管根——否则 <父目录>/opencodex/home 会被同机所有实例共用。

验证：fmt 与 clippy -D warnings 通过；modules::extensions 25 项、tests/extensions 39 项通过；全量 lib 2177 通过 / 5 失败，与基线 6784f89fe 相同，未新增。Mac 侧真实官方 2.99 启停结论不变；整版 accepted:false，仍待 Windows 按新目标矩阵重建候选后的真机启停与清理回交，以及正式候选验收。见[扩展落点边界合同](0.2.24-contracts/extension-client-home-boundary-r1.md)与[证据](../../.adg/evidence/0.2.24/extension-client-home-boundary-r1/summary.json)。

### IMP224 前端遗留用例与最终候选

IMP223 之后把 13 个失败的前端用例逐个对照产品代码，结论是测试过时而非功能回归：断言的是已撤回的入口（「诊断与恢复安装」）或从未存在过的 test id（runtime-uninstall、uninstall-ack、uninstall-confirm），以及拆分前根 store 的转发字段。代码 06ddcc7d1 改为断言真实界面：程序卸载走 runtime_removal_preview/apply 的复核与确认，进行中允许关窗且不重复提交；官方面板版本事实按 updates 切片断言；loadOfficialProject 现在同时读运行版本事实，断言随之补齐。

结果：vue-tsc 通过，vitest 142 文件 / 892 用例全部通过，此前为 13 失败。

据此重建候选并核验，未安装、未改动日常配置：应用二进制 40925331f82aef8a585f934c807b7431de813faf0255de78d2a26cadd78d8d89；无头首次供给通过（2.99.0、node v26.8.1、ocx doctor 报自管根、配置未变、无残留）；DMG b6d4f6a97faa10f43513c17f1192316c944ded641f1d737742e686a18ff22961（273,933,528 字节）只读挂载核验通过；发布输入预检 --tag v0.2.24 通过。

仍未完成：首次供给后的界面实屏观察（该机锁屏，系统拒绝 AX 观察与截屏）；Windows 按新扩展边界重建后的真机启停与清理回交。整版 accepted:false。

## 集成测试面（代码 06ddcc7d1）

除 lib 之外，全部集成测试二进制逐个跑完（--no-fail-fast）：38 个 test 二进制无失败，包括 extensions 41 项、client_selection_rename 21 项、client_npm_runtime 18 项、doctor 9 项、event_registry 43 项、status 系列与 tray 系列。唯一失败集合仍是 lib 的 5 项基线失败。

这一面覆盖了扩展落点、来源选择与重命名、托管运行、状态矩阵、托盘与窗口关闭，说明这次边界改动没有波及到别的子系统。

### IMP225 UI 发布门禁与最终候选确认

在最终候选 06ddcc7d1 上跑 UI 发布门禁（qa/gate-release.mjs）：typecheck、unit 892 项、build 三项全部通过，快照 3 通过 / 0 失败 / 0 跳过。此前基线里 unit 一项是失败的，恢复被误删的前端测试后门禁才具备完整意义。

本轮同时清理了 11 月 21 日遗留的陈旧构建目录（约 35 GB），磁盘可用空间由 24 GB 回到约 60 GB；工作树里 17 GB 的 apps/desktop/target 调试产物一并删除，它没有任何配置引用，且早于本次改动。两处都不涉及源码。

仍未完成：首次供给后的界面实屏观察（该机持续锁屏）；Windows 按新扩展边界重建后的真机启停与清理回交。整版 accepted:false。

### IMP226 日常目录未被触碰的现场核对

在最终候选构建与三次真实启动（两次 GUI 首次供给观察、一次产品路径 Start/Status/Stop）之后核对本机日常目录：

- `~/.codex/config.toml` 的修改时间仍是 2026-11-21 20:43，早于今天全部改动；SHA256 4b7102ff4397279173e12475b36f5d4f4813ae1d7b309052f5fdc349e499cd4b。
- `~/.codex-desktop/opencodex/home/codex-client` 在本机日常 profile 下不存在：本次没有经由日常配置启动服务，扩展写入只发生在隔离夹具里。

`~/.codex` 目录本身的 mtime 有更新，来自正在运行的 Codex 会话写 `thread_history_*` 与 `logs_2.sqlite`；这些不是 Desktop 写的，也不属于本次改动范围。逐条文件的 mtime 与上面两条一起看，才能区分「谁在写」。

### IMP227 最终代码上的产品路径启停

在最终代码 06ddcc7d1 上重跑 official299_product_lifecycle_uses_desktop_client_home（隔离根，官方 2.99.0 运行包）：Start 后 attested Status 报 running/ready，进程与端口都已记录；实读运行进程环境，CODEX_HOME 为隔离根下的 opencodex-home/codex-client；Stop 正常。真实 `~/.codex/config.toml` 的 mtime 仍为 2026-11-21 20:43。测试根用完即删，无残留进程。

这说明扩展落点的改动没有破坏服务启动、就绪与停止，也说明服务确实按自管根读客户端配置。

原型同步：原型里 Codex 的 Skills 与 MCP 路径改成了 `<OPENCODEX_HOME>/codex-client/...`，与软件一致；原型自测在改动前后同为 383 通过 / 201 失败（缺 jsdom 的既有失败），未新增。

### IMP228 Windows 交接送达正确分支

前几轮的交接与两份合同写在 `docs/024-extension-boundary`，但 Windows 实际消费的是 `docs/024-windows-dev` 及其派生分支。这两个分支不属于同一条历史（共同基底是 cfa4e6f0f，各自另有提交），所以交接没有真正送达——这与此前「Windows 收不到新内容」是同一类问题。

已新建 `docs/024-windows-boundary`（基于 `origin/docs/024-windows-dev` 的 32b0483e3），把 `windows-dev-handoff.md` 续写一段本轮交接，并把 `extension-client-home-boundary-r1.md` 与 `windows-rebuild-on-extension-boundary-r1.md` 一并放入该分支。交接写明重建基线为 main 81d823971、两条硬约束、保留写集与四项回交。

后续给 Windows 的交接都写在这个分支，避免再出现写在它读不到的地方。

### IMP229 磁盘回收与候选包独立保存

构建与多轮验证把本机可用空间压到 17 GB（99%）。为避免演示或验收时因空间不足失败，做了三件事：

- 最终 DMG（b6d4f6a97f…，273,933,528 字节）与 .app（二进制 40925331f8…）各自复制到独立目录后再删除对应构建缓存，哈希在复制后复核一致。
- 删除 `apps/desktop/tauri/target/debug`（266 GB 的编译缓存，需要时可重编）。保留 release 目录与资源，以便再次打包。
- 清理 11 月 21 日遗留的 `/private/tmp/ocxd024-*` 构建目录。

回收后可用空间约 38 GB。未删除任何源码、证据或本地未推送的原型工作。

### IMP230 Windows 目标的真实类型检查

此前「本机没有 Windows 工具链，Windows 结论只来自逐文件净差审查」一直是这块的缺口。本轮用 cargo-xwin + Homebrew LLVM 在本机对 `x86_64-pc-windows-msvc` 做了真实的 `cargo check`，不再只是读代码推断：

- `cargo xwin check --target x86_64-pc-windows-msvc --all-targets`：**0 错误**，包含 `windows_official_runtime024/**`、`windows_official_lifecycle024.rs`、`windows_private_create.rs`, 以及全部 Windows 专属测试目标。
- 只有既有警告（024 运行时模块的 re-export 未被使用、一处 `proof_deadline` 未使用），不是本次改动引入，也不影响类型正确性。
- 这证明本次扩展落点改动（ClientTarget 拆 anchor/home、client_targets 入口、validate_targets 签名、home_from_targets 改写）没有破坏 Windows 侧对这些类型的用法。

过程中遇到一个构建脚本按设计拒绝的情形，不是缺陷：跨平台检查时本地 `resources/runtime-bundle` 只放了 darwin 的描述符，build.rs 会以 `runtime descriptor target mismatch` 终止。临时移开该目录即可完成检查，检查后已原样放回，descriptor SHA256 仍为 e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3。

边界：这是编译期类型检查，**不是** Windows 真机运行。真机 Start/Stop、双账户自管根隔离、扩展写入落点与清理回交仍需 Windows 机器执行。

### IMP231 Windows 全部测试目标完成链接

在 IMP230 的类型检查之后，进一步用 cargo-xwin 把 `x86_64-pc-windows-msvc` 的**全部测试目标**链接成真实可执行文件：

- `cargo xwin build --target x86_64-pc-windows-msvc --all-targets`：0 错误。
- 产物为真正的 PE32+ x86-64 可执行文件，共 54 个 .exe，含库测试宿主与 `extensions`、`process_action`、`status_matrix` 等集成测试。

这比类型检查更强一层：链接阶段会暴露符号解析、跨 crate 接口不匹配、Windows 专属导入库缺失等问题，这些都没有出现。

仍然只是交叉编译证明，**不是** Windows 运行结果。PE 不能在本机执行，真机 Start/Stop、双账户自管根隔离、扩展写入落点与账户清理仍必须由 Windows 机器完成，验收表保持该项未完成。

收尾：runtime bundle 已原样放回（descriptor 摘要不变），tauri-build 为 Windows 目标生成的 `gen/schemas/windows-schema.json` 是构建产物（从未入库，与已入库的 macOS 版对应），检查后删除，工作树干净。

### IMP232 外置 OPENCODEX_HOME 下的扩展写入：已知限制

本轮尝试为「用户把 OPENCODEX_HOME 指到数据根之外」时的扩展写入补测试，发现这条分支**当前不可用**，并已撤回未收敛的改动，代码保持已验证状态（扩展集成 41 项全绿）。

现象：外置绑定存储的是规范化路径（macOS 下为 `/private/var/...`），而数据根的聚合目录来自 `/var/...`。写入前的边界校验用字面 `strip_prefix` 比较，两种写法对不上，合法落点会被判成「target context escapes its explicit root」，写入失败。

这不是本次改动引入的：外置 HOME 与扩展写入此前也没有端到端覆盖。但它是真实限制——设了外置 HOME 的用户，拓展页写入会失败，而不是写错位置，所以不会损坏配置。

为什么没有就地修掉：根与四条待检查路径必须同时统一写法才能通过，而其中 `codex-client` 首装时并不存在，规范化只对已存在的前缀生效；一旦某条路径规范化、另一条没有，字面比较又会失败。反复尝试了几种组合，都只能修好一部分用例而打破另一部分，没有收敛到可信的最小改动。把半成品合入比保留一个明确的上报更糟。

下一步需要的是把校验改成按「解析后的真实目录」比较（两侧都解析已存在前缀再比），而不是字面前缀。这会改变校验的核心语义，应当单独评审，并与「首装时目录尚不存在」的处理一起定。在此之前，外置 HOME 场景在验收表里记为已知限制。

### IMP233 外置 OPENCODEX_HOME 的写入选址已修复

IMP232 把该场景记为已知限制，根因判断是对的，但结论下早了：问题只出在路径写法，不需要改变校验语义。

根因：外置 OPENCODEX_HOME 的绑定存储为规范化路径（macOS 下 `/private/var/...`），而数据根的聚合目录来自 `/var/...`。写入前的边界校验用字面 `strip_prefix` 比较，根与落点写法不一致时，合法的后代被判成「target context escapes its explicit root」。

修法（代码 b684e1788）：校验前把边界根与各落点统一成同一写法——复用数据根已有的「解析已存在前缀、缺失后缀原样拼回」语义，因此首装时 `codex-client` 尚未创建也能通过；随后再用统一后的路径做包含判断与逐级 `check_path`。

回归 `external_opencodex_home_moves_the_codex_landing_with_it`：外置 HOME 下写入落在自管根、能从该文件读回 `[mcp_servers.external_probe]`，真实客户端目录未被创建。

验证：扩展集成 42 项全绿；fmt 与 clippy -D warnings 通过；lib 2177 通过 / 5 失败（仍是基线既有 5 项）；Windows 目标 `--all-targets` 检查 0 错误。

过程说明：IMP232 里提到的反复尝试确实存在，当时没有先取得「哪一行拒绝、两边的实际写法各是什么」的证据就连续改结构，导致改动发散。这次先落一处统一规范化、立刻用回归测试验证，才收敛。

### IMP234 最终候选（代码 b684e1788）

IMP233 之后代码又变了，此前记录的候选包已过期，因此重建并复核：

- 应用二进制 SHA256：eab691d9e1e56769efcea8e726f449694f285c05caa9e0f626307312f54714a1。
- 无头首次供给：程序根建出、current.json=2.99.0、node v26.8.1、ocx 2.99.0、ocx doctor 报告 effective Codex home 为自管根、预置配置字节未变、退出后无残留进程。
- DMG：SHA256 e66af6f14316bcbe76c9bc6056e5df67a70352e7a31cbfe58b132121ba5e6170，273,933,748 字节，只读挂载与载荷核验通过。
- 产物已复制到独立目录（/private/tmp/ocxd024-final-app、/private/tmp/ocxd024-dmg-b684e1788），复制后复核哈希一致。

回收：删除本轮产生的 debug（14 GB）与 x86_64-pc-windows-msvc（30 GB）编译缓存，可用空间从 18 GB 回到 56 GB；release 目录保留以便再次打包。源码、证据与本地未推送工作均未删除。

仍未完成：界面实屏观察（该机锁屏）；Windows 真机启停与清理。整版 accepted:false。

### IMP235 发布脚本自测在最终代码上全通过

在 b684e1788 上重跑发布链路的脚本测试（不构建、不安装、不改日常配置）：

- 打包与核验：test_package_macos_024、test_verify_macos_build_024、test_verify_macos_updater_024、test_verify_runtime_delivery_024、test_stage_runtime_bundle_024、test_verify_windows_installer_024 —— 40 项通过。
- 发布与清单：test_publish_release、test_collect_installer_replay、test_runtime_delivery_manifest、test_desktop_runtime_bundle —— 43 项通过、1 项跳过。

这说明本轮对扩展落点与边界校验的改动没有影响 macOS 打包、运行时交付校验、Windows 安装器校验或发布清单生成。

### IMP236 集成测试在最终代码上重跑

验收表里「集成测试无失败」那条是在外置 HOME 修复之前跑的，本轮在 b684e1788 上重跑（--no-fail-fast，含全部 test 目标）：

- 52 个测试目标通过，0 个失败。
- 唯一失败集合仍是 lib：2177 通过 / 5 失败 / 29 忽略，5 项与基线 6784f89fe 相同（macOS TCP listener fixture 3 项、data_root 的 `/private` 路径、受系统保护的临时路径）。
- extensions 42 项（含新增的外置 HOME 回归）在其内通过。

### IMP237 界面实屏观察通过（首次供给后）

该项此前一直被锁屏挡住。解锁后对最终候选（二进制 eab691d9e1…）跑 macos024-first-supply.py，在真实正常 profile、内核拒绝日常 HOME 与网络的隔离下完成首次供给并在实屏观察界面，结果 passed：

- 界面阶段可读：概览 → 展开运行包与依赖 → 核对程序与固定依赖 → 核对程序入口 → 内置程序已就绪 → 版本升级/文件同步/WebDAV 同步/最近事件。
- `uiEnvironmentReadyObserved` 为真，`staleAbsenceCaptionAfterReady` 为假：就绪后主标题不再是「接入后可启动」，不再有「未发现安装」的过期说法。
- 截图采集成功（captureExit 0）：界面显示「已发现安装，代理未运行，可以直接启动」，运行状态为未运行，版本升级/文件同步/WebDAV 同步三张卡片与最近事件都正常呈现，无「待接入」或报错。
- 运行包：current.json=2.99.0，node v26.8.1、npm 11.19.0、ocx 2.99.0 版本查询均通过。
- 预置配置字节未变，未生成 runtime-port 记录；源应用与副本二进制均未变；退出后无残留进程，临时根已删除；日常配置未被修改。

意义：这是首个把「安装后界面状态」与「运行包实际就绪」一起证明的实屏结果。此前只有无头供给与 CLI 层证据。

边界不变：本条只证明首次供给与就绪界面，不包含 Start/Stop 生命周期、业务同步或模型请求。Windows 真机项不受影响。

### IMP238 Windows 净差能否在 Mac 上重建：审查结论

目标要求 Mac 与 Windows 异步并行、由 Mac 完成净差集成。本轮就此做了一次实际评估，结论是**这条分支无法在 Mac 上干净重建**，证据如下。

以共同基线 cfb3c8aa7 对比 codex/024-windows-native（6532cbb6e），`apps/` 下共 215 个文件变化，其中 172 个属于已撤回的客户端原件保管与 owner-host 链（`windows_client_originals024` 171 个、`windows_owner_host024` 13 个）。

剩下约 43 个里，真正独立于该链的是运行层四个模块：

- `windows_official_runtime024/file_move.rs`（248 行）
- `windows_official_runtime024/peer.rs`（104 行）
- `windows_official_runtime024/pipe.rs`（363 行）
- `windows_official_runtime024/host_process.rs`（147 行）

它们都不引用 `windows_client_originals024`。但它们的上游依赖并不自足：`pipe` 依赖分支版的 `security::verify_private` 与 `PrivateDescriptor`，`file_move` 依赖分支版 `paths::validate`，`host_process` 通过 `use super::*` 依赖分支版 `mod.rs` 的整套 re-export。而分支版 `paths.rs`、`security.rs`、`process.rs` 各自又混入了保管链用的成员（如 `owned_file_identities`、`into_owned_files`、`locator_snapshot`、`preparation` 子模块）。

生命周期侧同理：`actions.rs`（424 行）表面上不引用保管链，但它 `mod recovery` 并 re-export 恢复相关类型；`start_worker.rs` 直接引用 `windows_client_originals024::recovery_worker` 与 `recovery_timing` 共 5 处；`windows_diagnostic_roles024.rs` 依赖 `windows_supervisor024::WindowsSupervisor`。

因此「只保留本版范围内的 Windows 代码」不是删文件，而是把同层原语从保管链语义中拆出来重写：`paths`/`security`/`process` 三个文件在分支里混合了两种用途，拆开需要重新定义私有句柄、私有描述符与路径准备的接口，并在 Windows 上重新验证 ACL、reparse、单链接等原生事实。这已经超出「净差集成」，属于一次新的 Windows 原生开发。

结论：这项由 Windows 按其写集完成——它掌握原生真机与既有验证脚本，Mac 侧无法代跑，也无法在不重写接口的前提下拆分。交接与四项回交已写在 `docs/024-windows-boundary`。

本轮的临时评估工作树与分支已删除，未改动任何产品代码。
