# 官方2.99渠道显示与保存来源 r1

revision: 4；2026-11-21。需求AC-P24-09由用户明确采用“原生显示格式，继续验证刷新与保存”。本文件拥有行为/来源合同，执行进度唯一归IMP-34；不改官方包，不变更Windows生命周期ABI或独占scanner/恢复写集。

- 普通官方合法渠道名（含点，最多64ASCII字符、首尾字母数字及保留字限制）实际改provider key并重写已管理引用；稳定channel ID/原始模型ID/凭据保持。中文等不能直接作键时保留合法内部路由ID。
- 当逻辑channel.label不等于实际目标providerID（含持久协议组）时，在官方modelDisplayNames精确原始模型ID处写“渠道名 · 模型ID”。显示串中/转／；不改selectedModels/models/modelAliases、请求或目录slug。生成值不超过官方128个UTF-16单元、无控制字符，超限或无法表示时事务写前拒绝，不截断。人工非空显示名优先；清空后回自动显示，无需改官方模型能力。
- compiler_v2与同一事务materialization ledger新增field=displayName的精确目标/存在性/值摘要，字段仍在原fields列表，无新增第二ledger或catalog文件。source规则消费认识这一种非能力字段但不把它转ModelRules；provider_import只在旧policy没人工显示名、重算自动值一致、同policy摘要、唯一同channel/model记录、目标pointer和实际值摘要全相等时视为自动，避免读回锁死旧名。无来源/外部改值/原人工同值都按人工保留，不迁移/收养未知生成记录。
- 同步改名后自动名称重算、人工名称不变；回合法provider key后默认自动map项清掉，人工项保留。协议组继续复用既有持久mapping，不因显示或活动URL改group身份。新分配provider key先为protocol/hash/碰撞后缀预留长度，再取父key前缀，保证64内；碰撞不覆盖其它provider。已有不合法映射拒绝，不自动迁移旧开发记录。
- provider原认证/apiKeyPool/能力、隐藏/顺序/default引用仍走原编译/凭据事务准入。显示名被外部改动按真实来源处理；保存基线冲突原样拒绝，不因显示功能放宽CAS。超限错误提供明确原因，原JSON/草稿和页面待保存输入不丢。
- 官方saveConfig/loadConfig及syncCatalogModels负责正式目录发布；Desktop不写第二份display overlay、不轮询覆盖catalog、不生成combo取代模型，不修改官方源码。每代在新进程加载、连续两次官方刷新检查真实目录的唯一slug/display_name和文件读回；这些测试不是默认Codex GUI已加载证明。

验证使用独立临时Profile＋内核禁止日常HOME/外网/Keychain、固定官方bundle、合成凭据且liveModels=false；test/smoke/macos024-display-refresh.py从真实Rust保存事务导出三代配置，再以原样官方Bun函数执行保存/读回/目录刷新。先保留原失败（组key>64导致官方default fallback），修新分配逻辑后原断言不放宽。所有测试残留归本次明确根清理，证据包含原源码hash、命令、stdout/stderr、每代合成配置、目录读回和清理；无服务/真实业务同步/模型调用。

当前不因本合同签整版完成：最终集成包、默认客户端真实加载/应用重启、相同显示映射的正式回退仍需固定候选证据，Windows平台无新增专属UI验证义务。版本正式发布门保持关闭。


默认客户端按用户确认为ChatGPT桌面内本地Codex，实际选择器证据与隔离边界见[客户端验证合同](chatgpt-codex-name-validation-r1.md)。整文件备份回退组件和未验范围只归IMP176，不用官方函数通过替代客户端真实加载。


### 保存、发布与回退核验

渠道 V2 保存的成功口径是磁盘事务提交并读回。对同一保存代次独立确认官方运行态接管、目录收敛和模型读回。以实际渠道页在运行中触发保存，验证合法 key 改名、自动标签更新、人工标签保持。模拟保存期间基线变化或服务绑定变化，确认没有误报当前代次已发布。发布使用官方入口，保留基线校验、凭据保护和可重试结果。客户端重启须由用户显式确认。

整文件回退在确定停止时执行。用同一固定候选核对回退磁盘事务、随后启动发布的恢复代次、实际客户端选择器三段结果。隔离后端测试的已有结论和客户端准入见[客户端验证合同](chatgpt-codex-name-validation-r1.md)，实施状态见 IMP237。

原样官方 2.99.0 定向诊断见 IMP238。先核对官方实际接纳的 Codex 二进制，再核对保存代次与真实目录。合法换键使用逐渠道 reload 后的官方整体 PUT 使目录收敛。发布确认包含唯一 slug、精确显示名及待移除 slug 已消失。固定候选继续验证 Desktop 的客户端发现与保存代次绑定。
