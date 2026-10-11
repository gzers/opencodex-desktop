---
id: MNT-OPENCODEX-DESKTOP-20261009-01
object_kind: maintenance.record
state: in_progress
title: 概览升级入口与备份通知策略（0.1.10）
col: COL-LOCAL-20260912-01
summary: 登记 0.1.10 更新、备份、统一事件、双平台目录、按钮与背景光及共享树表范围；完整实施与验收计划见 IMP-20，原型候选与实施合同已落盘，基础源码已分笔提交，生产实现及验收进行中，未发布。
source_refs:
  - COL-LOCAL-20260912-01
  - REL-OPENCODEX-DESKTOP-09
---

# 概览升级入口与备份通知策略（0.1.10）

登记于 2026-10-09；作为主协作包的运行维护附件，不改变原包 open 状态。受影响版本以用户 0.1.9 使用反馈为起点，目标维护版本 **0.1.10**；现已进入共用原型候选评审；按钮取消高光与背景光覆盖正文边距为已确认方向，升级 / 备份 / 通知流程供评审。已按目标模式启动生产实现；登记初期版本号仍为已发布 0.1.9，本轮已将源码身份固化为 0.1.10，尚未重新打包或发布。

2026-10-10 实施前确认：用户认可其余主要方向，补充所有树表使用统一组件，解决复选框、箭头、图标与文字垂直不齐。产品范围已纳入 [IMP-20 完整计划](../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-20.md)；目录遵循 macOS 默认、Windows 实际安装目录下 data，自定义活动根优先。自动检查性能需重点实测，未证明通过；具体契约、设备和性能预算在 P0 冻结，真实迁移与发布另行确认。稳定后才按用户后续确认回写核心事实，协作包不作为长期核心事实的替代。

| 事项 | 本次范围 | 待确认与验收重点 |
|---|---|---|
| 概览升级入口 | 概览与设置复用同一更新流程，区分管理器与面板，检查后按对象查看；内部滚动更新说明、网页出口与分阶段进度，备份作为确认选项 | 检查中、已最新、有更新、失败均有反馈；查询与安装分开，确定候选版本和来源，重复点击不重复执行 |
| 备份澄清与配置 | 明确备份对象、保存位置、触发条件、恢复方式、清理与轮换；评估固定保留版本 | 按对象展示实际覆盖范围；数量／时间保留规则与固定保留的关系需定稿；清理可说明且不删除固定保留项，恢复验证完整性 |
| 通知策略 | 统一注册所有可通知事件，由开发维护集中配置；本期无需用户自定义配置界面 | 支持启动、路由、定时、恢复等多触发源；列出一次性／状态变化／周期性质、分类级别、去重与恢复、投递方式；事件注册不等于每次弹窗 |
| 双平台目录规则 | Windows 默认数据目录随实际安装位置派生 data，macOS 保持 Application Support；保留自定义根、托管前缀与外置 HOME | 普通用户权限、旧版本迁移、全部存储重绑定、升级保留与卸载数据选择需独立实施和设备证据 |
| 按钮材质一致性 | 标准按钮取消渐变高光和顶部内亮线，保留强调 / 危险色及交互反馈 | 原型来源样式已修改，浅深主题 / 三档实屏、焦点和生产实现仍需验收 |
| 概览背景光留缝 | 按最新用户要求纳入 0.1.10；光层覆盖正文左右及顶部边距，文字 / 卡片保持原间距 | 原型 iframe 对称扩宽并按面板实际顶边扩展，Logo 位置补偿；标准 / 紧凑、就绪 / 环境准备共用；浅深主题、三档、CSS / WebGL 与滚动边界实屏待验 |
| 统一树表组件 | 所有生产树表统一组件；原型层级树表遵循同一合同，修正复选框 / 箭头 / 图标 / 文字对齐 | 全入口盘点、有无选择框 / 半选 / 长文字 / 缩放 / 键盘验收；0.2.0 同步原型仅统一组件，保留两个 tab 与业务范围 |

版本裁决更新：此前将背景光问题归于 0.1.9；本轮明确以 **0.1.10 为修复目标**，保留 0.1.9 的问题来源，不修改历史发布资产。0.1.3 更新通道专题是历史版本，不并入本次维护范围。

- [问题来源与版本裁决](问题来源记录/README.md)
- [现状、设计要点与待决策](分析/01-范围与设计要点.md)
- [统一事件注册与多触发源规划](分析/02-统一事件注册与多触发源规划.md)
- [更新详情来源与进度反馈](分析/03-更新详情来源与进度反馈.md)
- [双平台数据目录与迁移边界](分析/04-双平台数据目录与迁移边界.md)
- [备份目录树、文件作用与新增范围登记](分析/05-备份目录树与范围登记.md)
- [执行基线、合同与源码测试证据](分析/06-执行基线与契约.md)
- [完整实施、测试、稳定事实与发布计划（IMP-20）](../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-20.md)
- [0.1.10 原型设计与交接](原型/DESIGN.md)
- [原型 QA：状态与静态通过，视觉待验](原型/原型QA.md)
- [运行维护总索引](../README.md)
- [0.1.9 发布事实](../../../../../../03-开发实施/REL-OPENCODEX-DESKTOP-09.md)

2026-10-10 原型审计修正：异常按对象 / 动作 / 阶段 / 事务与成功证据解除；管理器通道切换保留面板缓存、退避和在途事务。41 项状态 / VM 交互测试通过，实屏仍待验，规则与证据见事件规划和原型 QA。事件语义稳定后再回写核心事实文档。

当前按 IMP-20 从 P0 契约推进 P1–P6；保留规则采用最近 N=10 或 D=30 天满足其一、固定项另外保护、默认手动预览清理，仍需落实事务关联与真实恢复合同。生产代码、设备测试、长期性能和发布状态分别取证；本文只引用实现分支与真实证据。该记录不构成发布授权。材料由 Ezio 与协作 Agent 持续维护，保留至维护闭环；稳定结论按后续确认回写需求与项目核心，运行结果回写 IMP／RUN／REL。

### 2026-10-10 面板保护核验补充

实现 5401dd5c 已推送：面板更新的偏好保护已有精确持久关联及启动本地重试，未决关联阻止目录切换 / 迁移。源码回归 Rust 753 通过 / 4 忽略、fmt / Clippy 通过；损坏回执的受控修复、原生更新与跨平台验收仍未完成。见 [证据与边界](验证/20261010-面板保护持久核验/README.md)。不晋升核心事实或发布状态。

### 2026-10-11 MCP 配置终态补充

代码分支 `feature/0.1.10-maintenance` 的 `a392afd9b6a81ea6d82eb5ba591600466765e0f4` 已推送。本轮收口 MCP 配置写入的安全终态：配置读取统一有界（16 MiB）、拒绝符号链接 / 非普通文件；多目标写入按稳定顺序持有目标锁；准备阶段读取的原件贯穿合并、备份和写入；原子写后精确回读，字节不一致失败；home、Skills / MCP / lock 路径、越界和重复物理目标执行前拒绝。

本地 Rust 全量为 690 passed / 0 failed / 4 ignored，fmt、Clippy 与 diff 检查通过。局部通过只覆盖 MCP 配置终态，不等于 0.1.10 完成；Skills / MCP 客户端资产投射、原生 / 安装器、Windows 双权限与安装位置、实际更新 / 重启、配对性能、真实 stable 24h / beta 6h 周期和全域事件仍按 IMP-20 保持待验。统一树表组件合同、复选框 / 箭头 / 文字对齐要求继续适用于所有树表入口。

对应证据与 CI 状态见 [20261011-MCP 配置终态](验证/20261011-MCP配置终态/README.md)。旧 CI `38076431305` 与新 CI `38077500332` attempt 2 的 Windows job 均在测试执行前因 `STATUS_ENTRYPOINT_NOT_FOUND`（`0xc0000139`）失败，不能计为 Windows 通过；release build 与 final EXE smoke 未执行。本文不回写稳定核心事实，不颁发完成证书，不构成发布授权。

### 2026-10-11 Windows 构建与候选安装补充

代码分支 feature/0.1.10-maintenance 的 0b402c154be65760092b6817b3027a2d0a1dc6dc 已修复 Windows 构建时 Tauri 默认 manifest 与项目 Common Controls v6 manifest 重复链接问题，改用 WindowsAttributes::new_without_app_manifest()。

普通 CI 38083572209 与候选打包 CI 38083572237 均已完成并成功。Windows backend regression、release build、final EXE smoke，以及 MSI / NSIS 隔离安装、首屏、卸载均通过；两种安装器卸载后 EXE 均已移除。MSI SHA256 为 b7c20061e97b47156c9c5a4693786843f01bb9b949d0825eea2c23c6d3599116，NSIS SHA256 为 9711d86321ac77d8723d1dfcae8d3e9c0a282debf6da20c738a953de89445438。

候选包 artifact 11682205024，候选证据 11681514176，普通 CI 启动证据 11682320162；大小与 ZIP SHA256 见对应 .adg/evidence 记录和验证 README。候选环境为 win25-vs2026 / 20260925.250.1 / Rust 1.99.0 / Node v24.21.0 / Tauri CLI 2.11.4。管理员权限启动证据已通过，但普通权限、DPI / 外观人工验收仍待完成。

本次候选版本仍为 0.1.9，不能作为 0.1.10 发布资产；codeSigned: false、releaseAccepted: false。macOS 原生安装、真实更新 / 重启、双平台自动检查性能、真实 stable 24h / beta 6h 周期、真实用户迁移、签名与发布门禁仍未闭合。历史 CI 38077500332 的失败记录继续保留，不被本次成功证据覆盖。协作包与 IMP 继续 in_progress，不构成发布授权。


### 2026-10-11 统一树表对齐与本地全量验证补充

代码分支 `feature/0.1.10-maintenance` 的提交 `8f5a76f867272730ee22ba672934fdd82807d218` 已推送。共享生产树表组件统一复选框、箭头占位、图标与文字的 22px 垂直对齐基线，移除原生复选框外边距补偿，不改变选择、展开、备份范围或树表数据语义。所有生产树表继续复用 `UiTreeTable.vue` / `treeTable.ts`，0.2.0 的同步原型只继承组件合同，不提前改变业务范围。

本地回归结果：Rust 全量 `690 passed / 0 failed / 4 ignored`；UI `94` 个测试文件、`459` 个测试通过；类型检查与生产构建通过，构建转换 `170` 个模块。Vite 的既有 `INEFFECTIVE_DYNAMIC_IMPORT` 警告和 jsdom canvas notice 均未阻断。详见 [20261011 本地全量验证](验证/20261011-本地全量验证/README.md) 与 `.adg/evidence/OCX-0110-20261011-LOCAL-VALIDATION/`。

本轮只闭合源码回归证据。macOS 原生 UI / 安装、Windows 普通权限与安装路径 / DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、真实更新 / 重启、自动检查性能、stable 24h / beta 6h、签名 / 发布资产 / 更新端点以及全域事件覆盖仍是未闭合门禁。现有 Windows 候选包实际版本仍为 0.1.9，不能作为 0.1.10 发布资产；本记录不构成发布授权。

### 2026-10-11 目标执行检查点

已按用户确认进入 0.1.10 目标模式执行。代码基线为 feature/0.1.10-maintenance / 8f5a76f8；文档与证据继续归属 docs/governance-main。本轮复核共享树表定向回归 2 个测试文件、12 项通过，既有 Rust / UI 全量源码证据保持有效。剩余原生、安装、性能、真实更新、签名、制品、全域事件和发布门禁仍需真实证据闭合，不形成发布授权。

详见 [20261011 目标执行检查点](验证/20261011-目标执行检查点/README.md)。

### 2026-10-11 更新调度触发与周期合同补充

代码分支 `feature/0.1.10-maintenance` 的 `24ed093aea0758b5fdf542fa38fc98a0c1d6b1e9` 已推送。概览进入使用 `foreground` 触发，持久化到期使用 `deadline`；旧 `app_update_check_interval_seconds` 只服务迁移与序列化，运行时周期不从该字段读取。manager stable / panel 固定 24 小时，manager beta 固定 6 小时。

本轮 UI 定向 6 个测试文件、34 项通过；Rust 调度定向 5/5；类型检查、生产构建、fmt、Clippy `-D warnings` 与 diff 检查通过。该记录只更新源码级合同证据，原生 UI、双平台安装 / 更新 / 重启、性能、真实周期观察、签名、全域事件调用点和发布门禁继续待验，协作包与 IMP 保持 `in_progress`。

详见 [20261011 目标执行检查点](验证/20261011-目标执行检查点/README.md) 与 [IMP-20](../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-20.md)。

### 2026-10-11 版本身份固化与全量源码门禁补充

代码分支 `feature/0.1.10-maintenance` 的提交 `9497b4e56f01b92abf977c6b244ec8e087ffa6b5` 已推送。该提交只统一 `tauri.conf.json`、Cargo 包 / 锁文件和 UI `package.json` / 锁文件的版本为 `0.1.10`，没有把测试夹具中的历史 `0.1.9 → 0.1.10` 场景改写。此前 Windows 候选包仍是 0.1.9 的事实继续保留；必须基于本提交重新生成并验证 0.1.10 候选，旧包不能直接晋升。

本轮源码验证通过：`cargo metadata --locked --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace --features integration-test`、`git diff --check`；Rust 全量测试退出码为 0，库目标 691 项通过 / 0 失败 / 4 忽略，二进制目标 2 项通过，集成测试目标均通过。UI `npm test -- --run` 为 94 个测试文件、459 项通过；`npm run typecheck`、`npm run build` 通过，生产构建转换 170 个模块；Windows bundle identity smoke 5/5 通过。

本轮只闭合源码版本身份和回归门禁。仍需重新生成 0.1.10 的 Windows / macOS 安装与签名制品，并完成普通权限、原生外观、自动检查性能、真实更新 / 重启、稳定 / beta 周期、全域事件调用点和发布门禁；`CHANGELOG.md` 只能在 `main` 集成阶段补写。协作包与 IMP 保持 `in_progress`，不构成合并、发布或 stable 晋升授权。

### 2026-10-11 运行态观测异常隔离

代码分支 feature/0.1.10-maintenance 的提交 c654287c06da3e621091c1051ebc19ce1c0b1077 完成运行态观测异常与启停执行异常的隔离。runtime-observation-recovered 只有在可执行文件、工作目录、OPENCODEX_HOME、对象、动作、阶段、通道和候选身份全部匹配时，才能解除同一运行上下文的观测异常；可解除的异常限定为 runtime-starting-failed、run-unreachable、run-at-risk、external-takeover。观测恢复不能解除 start / stop / restart 的 execution 生命周期失败。

本提交后的生产注册表统计为 8 triggers、39 jobs、81 events、27 notification policies、87 emission sites、8 scheduling sites、8 UI feedback sites、12 paths、4 cleanup policies。准确的注册表定向命令为：

    cargo test --locked --manifest-path apps/desktop/tauri/Cargo.toml --features integration-test --test event_registry -- --nocapture

结果为 33 passed、0 failed。该提交只闭合运行态观测恢复边界和对应源码定向回归，不表示全域事件调用点、双平台原生验收、安装制品或发布门禁已完成；协作包与 IMP 继续保持 in_progress。


### 2026-10-11 树表与事件注册审阅补充

代码分支 feature/0.1.10-maintenance 的提交 c654287c06da3e621091c1051ebc19ce1c0b1077 已推送。本轮按 0.1.10 目标模式完成树表与统一事件注册的静态审阅和定向回归。生产层级树表统一复用 apps/desktop/ui/src/components/ui/UiTreeTable.vue 与 treeTable.ts，入口为 BackupFiles.vue；SettingsRoute.vue:1158、1195 的 data-root-table 是数据根路径平面清单，不是层级树。复选框、箭头或占位、图标和文字继续遵循 16px / 22px / 18px / 22px 的对齐合同，备份树默认展开到月份层级；同步页 WebDAV 与配置文件导入两个 tab 保留。

当前注册表为 8 triggers / 39 jobs / 81 events，其中 79 active、2 planned；另有 27 notification policies、87 emission sites、8 scheduling sites、8 UI feedback sites、12 paths、4 cleanup policies。生产直接 app.emit / app.emit_to 集中在 event_delivery.rs，其余生产发出点经过统一 event_delivery 边界。process-cancel-signal、process-observation-signal、process-exit-signal、runtime-stdout-signal 四项明确登记为 internal_only 进程内部信号，不进入通知持久化、不提供前端订阅、不计为用户通知投递缺口。完整矩阵和盘点方法见 [树表与事件注册审阅证据](验证/20261011-树表与事件注册审阅/README.md) 与 .adg/evidence/OCX-0110-20261011-TREE-EVENT-AUDIT。

定向 event_registry 测试为 33 passed / 0 failed / 0 ignored；既有 Rust 694 passed / 0 failed / 4 ignored、UI 94 个测试文件 459 项通过、类型检查和生产构建通过的源码证据保持有效。本轮只闭合静态审阅与定向测试。macOS / Windows 原生 UI、安装、DPI、云母、标题栏、动效、透明下拉框、自动检查性能、真实更新 / 重启、签名、发布资产、更新端点和真实用户迁移仍未闭合；releaseAccepted: false，不构成完成证书、合并授权、公开发布或 stable 晋升。


### 2026-10-11 c654287c 最新源码全量复核

代码分支 `feature/0.1.10-maintenance` 的最新提交 `c654287c06da3e621091c1051ebc19ce1c0b1077` 已完成独立源码门禁复核。Rust metadata、fmt、Clippy `-D warnings` 均通过；`cargo test --locked` 为 34 个测试组共 `861 passed / 0 failed / 5 ignored`，其中 library `694 passed / 0 failed / 4 ignored`、binary `2 passed / 0 failed`。UI 全量为 94 个测试文件、459 项通过，类型检查与生产构建通过（170 modules transformed）；Windows bundle identity smoke 为 5/5 通过。

本轮继续绑定运行态观测异常与启停执行异常的隔离规则：只有可执行文件、工作目录、`OPENCODEX_HOME`、对象、动作、阶段、通道和候选身份全部匹配时才解除对应观测异常；不能解除 start / stop / restart 的 execution 生命周期失败。注册表快照为 8 triggers / 39 jobs / 81 events / 27 notification policies / 87 emission sites / 8 scheduling sites / 8 UI feedback sites / 12 paths / 4 cleanup policies。

详细命令、结果和限制见 [c654287c 源码复核](验证/20261011-c654源码复核/README.md) 与 `.adg/evidence/OCX-0110-20261011-C654-SOURCE-REVALIDATION/`。本轮只证明源码级回归；macOS / Windows 原生、安装、性能、真实更新 / 重启、签名、发布资产、真实迁移及发布门禁仍未闭合。协作包与 IMP 保持 `in_progress`，`release_authorization: false`，不构成完成证书、合并或发布授权。

### 2026-10-11 main 集成 CI 回写

feature/0.1.10-maintenance 已集成到 main，集成提交为 3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea，origin/main 已同步；CHANGELOG.md 已写入 0.1.10 未发布候选说明。GitHub Actions 38096673516（https://github.com/gzers/opencodex-desktop/actions/runs/38096673516）已完成并成功，release-tools、frontend、backend、backend-windows、build 全部通过。Windows CDP / backend regression、Windows release build、final EXE smoke 与 artifact upload 通过，macOS arm64 bundle build 通过。

本地门禁为通过：npm ci、UI 类型检查、生产构建、94 个测试文件 / 459 项 UI 测试；Cargo metadata、fmt、Clippy -D warnings、Rust 测试（library 694 passed / 0 failed / 4 ignored，binary 2 passed）和 git diff --check。回执见 .adg/evidence/OCX-0110-20261011-CI-38096673516/。

CI 成功只闭合集成源码、构建和 smoke 门禁。真实 macOS / Windows 原生 UI、安装与数据根、双权限、DPI / 云母 / 标题栏 / 动效 / 透明下拉框、自动检查性能和稳定周期、真实更新 / 重启 / 恢复、签名与发布门禁仍待验收；本协作包和 IMP 继续 in_progress，release_authorized: false。本次不创建发布标签、不公开发布、不替换日常 0.1.7。


### 2026-10-11 main 源码门禁证据

0.1.10 当前源码事实已绑定到 main@3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea，并确认 origin/main 已同步。Rust metadata、fmt、Clippy、workspace integration tests、UI 全量测试、类型检查、生产构建、Windows bundle identity smoke 和 git diff --check 全部通过；结果为 Rust library 694 passed / 0 failed / 4 ignored、binary 2 passed / 0 failed，UI 94 个测试文件 / 459 项通过，构建转换 170 个模块，identity smoke 5/5。

证据目录：.adg/evidence/OCX-0110-20261011-MAIN-SOURCE-GATES/；验证说明：验证/20261011-main源码门禁/README.md。该记录只证明 main 的源码、构建和身份门禁，旧 feature 分支与既有记录不改写。旧 Windows 候选包实际版本为 0.1.9，必须重新生成 0.1.10 双平台制品。

发布仍未授权：release_authorized: false，本协作包保持 in_progress。原生 UI、安装、双权限与数据根、自动检查性能和真实周期、真实更新 / 重启 / 恢复、签名、公证 / Authenticode、全域事件执行证据、真实用户迁移和发布门禁均待闭合。
