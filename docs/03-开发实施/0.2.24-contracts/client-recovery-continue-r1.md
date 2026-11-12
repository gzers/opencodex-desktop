# 部分中断客户端恢复的显式继续

revision: 1；2026-11-12。归IMP-34 P24-04，承接[整组核对](client-recovery-review-r1.md)与[收尾界面](client-recovery-ui-r1.md)。后端main37e486fad，前端mainbc887457f。accepted=false、非冻结候选。本批接通同一已独立登记RestorePrepared计划的继续；回退终态和更早捕获阶段仍未实现，不能用继续冒完整中断恢复交付。

## 生产行为与授权事实

预览在原共享锁内核对原frames/scope/live/plan，只有terminal缺席、completion缺席、全组目标/暂存/归档仍属原计划，且三目标/精确plan暂存以外的整个Codex树及Grok树匹配原件、两稳定锁原身份一致，才返回canContinue与确认指纹。其它未知/合法后续编辑冲突均拒绝保留。仅精确plan的3目标及其3暂存分别由Pending原生身份/字节/保护核对，不忽略外来stage、其它文件或sidecar。leased=true只在同进程真实NativeLease核验时允许当前claim rollback journal，不扩大最终两稳定锁例外。

新client_recovery_continue命令只接原attempt/revision/confirmationDigest，不接路径/内容/选择性覆盖。复用运行/更新/storage门，Index.open_existing绝不补建缺失存储，第一权威确认在排他action锁内。经原样官方完整镜像准入与claim→N→L→K真实同进程锁，重新比对原确认后才能签发借用的ExplicitRestoreAuthority。它不同于原普通Stop权限，不从磁盘Live/回执重造历史正常退出，证明的是当前独立OS停止事实和当前写者排他。

批次前后保留完整当前用户OS进程/环境扫描、原service/launcher和port缺席、manager及运行记录复核；每次native publication仍核验同一NativeLease/owner/VFS/context/期限、原service/launcher缺席、原port缺席、manager与runtime/pid记录缺席，Pending同时保留整组先查/逐项原生身份/字节/保护与冲突保全。没有用记录缺失单独推出已停止。完整全机枚举是批次边界，不在每个相同写前步骤重复执行；窄per-step函数不能单独签发权限或代替批次前置。

执行只调用原plan的Complete，不重新拍基线、不另建plan、无强杀/旧格式兼容。NativeLease正常释放后核对全客户端原字节/权限/两稳定锁释放并独立读回，最后原Index关闭才返回Restored。新显式动作从入口计原10秒，不借用旧失败Stop或另续预算；失败可能已经完成部分publication，错误只说明“未确认完成，原件与当前文件保留”，不声称未写文件。阶段/耗时诊断不输出私密原文。

## 界面与总原型

复用诊断中心当前恢复入口/UiDialog，pending已核对计划显示“客户端配置恢复中断，可以继续原恢复计划”，确认说明恢复到本次启动前配置、保留归档、服务保持停止。按钮“确认继续恢复”；metadata已完成分支仍为“确认完成收尾”，未知/歧义证据不出现执行按钮。两个能力互斥，rollback能力未接也拒绝矛盾DTO。

Pinia持有唯一在途promise，关闭重挂/重复点击不再次mutation；成功必须原attempt、revision+1且Restored，再读同D/H新live Stopped/NotRequired及无PID/port才解除原前端限制，不自动启动。失败保留限制并要求重新检查，不能连续重发同请求。总原型仅修改既有client-recovery-024.js，新增continue/continue-slow/continue-error内存状态，不动其它并行原型/index、不调用真实API/文件或持久化。

## 本批验证与保留失败

- 后端去重69项：tree9含精确名单/外来stage/外部config/权限拒绝，session21，restore23含11真实子进程exit91边界，owner15既有加1新增函数（10种record/pid/process/launcher/port/manager/unknown/context/identity/clear分支）。新窄host反例是明确mock合同，不冒原生OS服务证据。最终格式/编译/全目标严格Clippy通过。前端27项覆盖专用无路径命令、用户确认、关闭重开单mutation、矛盾DTO、失败限制及新状态核对；类型/生产build通过。
- 单个全新隔离Profile用原样官方2.99，Start8654ms/Status229ms/Stop5662ms；原父子正常退出/4TCP缺席后，在第1目标原生交换之后注入中断。catalog已恢复，cache/audit仍未恢复，原完整测试exit101/RestorePrepared revision6/Required全部保留，未伪报普通Stop通过。
- 新进程同保留现场确认：错attempt/revision/digest以及外部config改动均拒绝，原索引不变/外部内容保留；这些已过反例后继不重复。首实际continue10195ms失败，去掉重复锁外preview后同现场10272ms仍失败，均在原10秒拒绝保留；按阶段定位后将全OS枚举归批次前后，保持每步全部不同事实核对，再仅同现场重验，9528ms Restored revision7，重复确认拒绝。未为重试重装/重启服务或重跑完整流程。
- 原临时参数不含中断seam的继续进程，受OS sandbox限制，仅loopback允许、外网与日常HOME读取拒绝。无真实模型/业务请求。新动作不重建旧NormalExitProof。
- 独立只读native完整Codex/Grok快照SHA与新completion逐字匹配；index7/7events/receipt登记摘要、4帧/plan/35内容引用与15原blob核对；另7客户端路径原字节/原缺席全部匹配。
- Chromium7与WebKit7软件/原型mock场景通过；明暗1180×760及900×600、不溢出、取消零mutation、关闭重开1mutation、失败限制保持。最终详情“尚未登记”及互斥guard后WebKit3受影响场景通过；截图实际检查。生产dist排除DEV窗口handle/合成UUID/mock文本，含真正continue命令与确认文案。未新建native.app/DMG。

证据[本批目录](../../../.adg/evidence/0.2.24/client-recovery-continue-r1/)及manifest包含原中断/两次continue超时/最后成功的不同目录、来源和可执行SHA。success用owner-tests构建，最终源后加host反例、测试细化与失败文案包装另有编译/Clippy；不把后加源码声称已包含于先前可执行，也不为不影响算法的变化无假设重跑服务。独立读回日志解析首以行首查JSON失败，修解析现有原输出后完成，未重跑原生服务；该日志仍原字节保留。

自有进程/原主端口缺席，3精确新协调DB经排他取得/释放后和程序/Profile/全部原件/stage/archive/测试副本清理；既有global L的inode/字节/模式/owner保持、可排他重获释放、无sidecar。两引擎与增量轮browser/Vite均退出，unit TMPDIR为空，重复临时根在归档核对后删除。未改日常/官方/系统或业务配置，不发布。

## 后继

下一项为Rollback停止后像终态/持久恢复与用户确认，不能记成Restored；更早Preparing/Captured等阶段、首次缺目录/一般客户端图/完整进程集合/旧revision4首错仍需推进。Windows贡献、NAME、复杂System代理、官方发布验证/README及固定候选与W2保持原范围；本批未收新Windows回交，不跑W2。
