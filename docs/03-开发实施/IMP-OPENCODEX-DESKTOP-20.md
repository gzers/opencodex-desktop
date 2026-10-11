---
id: IMP-OPENCODEX-DESKTOP-20
object_kind: implementation.change
state: in_progress
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-182"]
completion_summary: null
title: 0.1.10 运行维护完整实施与验收计划
summary: 用户已授权目标模式执行；在 feature/0.1.10-maintenance 推进契约、存储、调度、更新、备份、统一事件、共享树表与视觉实现；真实用户迁移、日常安装替换、公开发布和 stable 晋升另行确认。
---

# 0.1.10 运行维护完整实施与验收计划

2026-10-10，Asia/Shanghai。用户要求以目标模式规划从文档治理事实、开发、测试到发布的完整过程，并补充所有树表使用统一组件，修正复选框、箭头和文字垂直不齐。需求方向及默认目录规则已确认；具体契约、生产实现与设备验收仍须按阶段完成。

关联 [0.1.10 协作包](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/README.md)。本 IMP 为实施路线与技术决策入口；协作包保留讨论、设计和原型证据，需求与 AC 归需求管理，稳定产品规则归项目核心，运行与发布事实归 RUN / REL，任务和检查回执归 .adg。

## 1. 基线、权限和完成定义

- 源码基线为已发布 0.1.9：`main@de6862d40389d527527b9db8ef52e186a58b625f`，发布结论见 [REL-09](REL-OPENCODEX-DESKTOP-09.md)。开始开发时复核远端、工作区、版本及最新合法基线；新增提交必须记录，不能默认为仍是该 SHA。
- 文档继续使用 docs/governance-main；代码已在 feature/0.1.10-maintenance 开发，最终并入 main。复用既有代码 worktree，保护未提交原型，不在文档分支提交 apps / workflows，不在代码分支提交 docs / .adg。CHANGELOG 只在 main；共享文件若修改，按 AGENTS 同步。
- 本次已授权执行完整计划，当前 state 为 in_progress；执行绑定 TASK-OPENCODEX-DESKTOP-182、写入范围与检查，不把局部实现等同于版本验收或发布完成。真实数据迁移、破坏性恢复、公开发布和稳定通道晋升保留单独确认，0.1.9 发布授权不沿用为 0.1.10 授权。
- 日常安装（包括保留的 0.1.7）继续保留；候选使用隔离数据与可识别的安装位置。必须使用真实旧安装器 / OTA 的阶段，先确认隔离或专用测试设备方案，不能为取证自动覆盖日常软件。
- 版本完成须同时满足范围 / AC 映射、真实源码与制品身份、回归、双平台设备、性能、核心事实回写、发布决策和发布后验证。未覆盖项标明原因及阻塞，历史通过项不得计为新版本通过。

## 2. 冻结的范围边界

| 工作包 | 0.1.10 要交付的行为 | 明确边界 |
|---|---|---|
| W1 更新入口与详情 | 概览 / 设置共用管理器及面板的检查、状态行、结果 / 详情 / 确认、说明来源、网页出口、阶段进度与失败重试 | 面板更新仍为官方包受控更新；无更新说明时如实提示，不能编造详情 |
| W2 备份管理 | 偏好备份、实际清单树表、固定、保留策略、清理预览、恢复前保护及完整性检查 | 不宣称完整 OpenCodex 配置 / 会话 / 登录态备份；保留其他既有事务的保护门禁 |
| W3 统一事件 | 集中注册触发源、检查任务、领域事件、投递与解除规则；盘点全部入口，接入本期及现有适用能力 | 注册不等于已实现；0.2.0 规划事件显式未启用，不借此开发新同步 / WebDAV 能力；不做用户自定义通知配置页 |
| W4 数据目录 | macOS 保持默认；Windows 新默认由实际安装目录派生 data；活动自定义根优先；缓存、备份、日志、托管面板等统一解析 | 旧数据保留并显式迁移；系统凭据 / OS 管理文件、外部已有安装及用户导出目标有明确例外 |
| W5 材质和背景光 | 标准按钮取消高光；概览光层覆盖正文左右及顶部边距，保留文字 / 卡片间距 | 三档由软件偏好决定，不跟随系统减少动态；保留能力失败回退和后台暂停 |
| W6 共享树表 | 建立唯一树表组件，全部现有生产树表入口接入；原型所有树表遵循同一组件合同 | 文件 / WebDAV 两个 tab 和原有业务语义保留；仅更新 0.2.0 原型组件，不把未来功能纳入生产范围 |

背景光按用户最新裁决归 0.1.10，问题来源仍为 0.1.9。0.1.3 更新通道专题保持历史归属。未来新增备份范围须同时更新范围登记、manifest、树表用途、恢复、清理与事件；未来新增事件在稳定核心事实、生产注册和调用点中同步维护。

## 3. 阶段顺序与退出条件

P0 已开始，后续阶段按依赖执行并记录实际证据。P0 先统一契约；P1 / P2 形成基础，P3 / P4 / P5 复用基础；P6 可在 P0 后并行，但共享文件由一个集成入口维护。P7 在全部实现合入候选后运行，P8 才提升稳定事实；P9 需要本版本发布授权，P10 才关闭版本交付。

| 阶段 | 工作及产物 | 依赖与退出条件 |
|---|---|---|
| P0 契约与治理基线 | 复核源码 / 文档基线；把 W1–W6 映射到真实需求 / AC；冻结存储、更新 DTO、备份动作与恢复合同、事件字段、树表 API、测试设备与性能预算；建立生产调用点清单 | 形成可追溯 AC / 检查映射；歧义与不适用项有处置；不能用临时编号伪装既有 AC |
| P1 存储与安装基础 | 活动数据根唯一解析；持久状态 / 缓存分区；Windows 安装位置、写权限、升级保留、卸载选项；迁移与 reference-only 全存储重绑定 | 在隔离数据完成新装 / 升级 / 不可写 / 自定义根 / 复制失败 / 回滚，旧根不被自动删除；真实用户迁移另确认 |
| P2 调度与来源 | 集中 nextDue 调度、持久缓存 / 退避、对象锁 / 世代号、有限来源查询；更新说明按版本缓存 | 多源合并、通道隔离、手动模式、跨启动及进程超时测试通过；性能预算达标 |
| P3 共用更新流程 | 概览和设置共享事务；精致结果 / 详情弹窗、说明内滚动、网页链接、可选备份确认、真实阶段进度、取消和重启回读 | 两入口往返及迟到结果、签名 / 完整性失败、关闭弹窗、失败重试均有真实反馈；不以动画计时伪造百分比 |
| P4 备份与共享树表接入 | 生产共享树表、manifest 驱动目录 / 用途 / 打开确认；范围与动作发现修复、关联事务、保留 / 固定 / 清理 / 恢复保护 | 复选框 / 箭头 / 文字对齐；数量或时间保留正确；关联记录不被拆清；损坏 / 越界 / 链接逃逸拒绝，失败保留旧数据 |
| P5 事件全量映射 | 对现有 UI action、command result、state observer、job 做“入口 → 注册 → 投递 / 解除”映射，替换分散通知策略 | 无未注册投递，未知项拒绝；适用入口有证据；未来能力显式 planned；异常对象 / 动作 / 阶段严格匹配 |
| P6 视觉与组件一致性 | 全部树表接入盘点；标准按钮 / 光层边距修正；主题 / 三档 / CSS / WebGL / 标准紧凑 / 准备与就绪态 | 原型配对及真实 WebView 检查通过；不破坏命中区域、滚动、可访问性、原生标题栏和后台暂停 |
| P7 综合验收 | 全套前后端 / 发布工具 / 协议检查；macOS / Windows 原生、安装、升级、性能和长时 / 跨启动矩阵 | 当前 SHA、制品、环境、原始失败与最终回执完整；失败 / 缺设备不能判通过 |
| P8 集成与稳定事实 | 审阅候选、并入 main、重跑集成 CI / 候选打包；按实证及用户后续确认回写核心事实、需求 AC、IMP；准备 REL 与发布检查单 | main 与制品精确对应；事实 / 注册 / 调用点一致；核心回写审阅通过，不在此阶段自动发布 |
| P9 发布执行 | 用户确认明确版本 / 制品 / 通道后打标签、构建签名、公开 Release / stable 投递；更新链接及旧公钥验签 | 源码、tag、资产摘要、签名与端点匹配；公开下载及专用设备旧版 OTA 路径验收通过 |
| P10 观察与关闭 | 观察更新源、错误、目录 / 偏好保留、重复通知与进程资源；必要时停止投递、回滚通道；补全 REL / RUN / 回执 | 无未处置发布阻塞；满足完成定义并形成治理关闭证据，IMP 才记录完成 |

阶段耗时不先承诺固定天数：设备、真实周期观察与发布授权是外部依赖。失败回到对应阶段修复，再重跑受影响门禁；不得靠减少断言、删测试或改通过阈值闭合。

## 4. 数据目录与兼容契约

macOS 默认活动根为 `~/Library/Application Support/com.gzers.opencodex.desktop`；Windows 新安装默认活动根为 `<实际软件安装目录>\data`，与盘符无关。已有合法自定义活动根优先，升级不自行改根。Windows 旧 AppData 数据在明确迁移前继续留用；安装位置变化不隐式搬动已有活动根。

所有管理器控制的持久文件经唯一 DataRoot 服务解析：偏好和调度状态归 `manager-state/`，备份归 `backups/`，更新可丢弃元数据归 `cache/updates/`，其他 logs、runtime、托管 HOME、sync-state 按原契约统一重绑定。调度状态的具体文件名 / schema 在 P0 冻结；清元数据缓存不能同时清 nextDue、退避与提醒去重。OS 凭据存储、系统 / WebView 管理文件及用户主动选择的外部导出路径例外逐项登记，不能声称应用能控制系统所有写入。

Windows 基于安装记录 / 实际可执行文件位置确定默认根，不能用 cwd、临时 updater 或固定 C / D 盘推导。不可写时明确提示选择自定义根，不静默退回 AppData、不要求日常以管理员运行、不放宽全目录 ACL。普通 / 管理员启动解析到同一已配置根，管理员成功不能替代普通权限通过。

迁移按停写 / 暂停受控进程 → 目标、空间、归属与冲突检查 → 覆盖范围充分的保护快照 → 复制和逐文件验证 → 原子切换全部存储引用 → 重启 / 回读验收进行；失败保留旧根和旧绑定，成功也不自动删旧数据。reference-only 不复制但须完整切换引用，不能只改变偏好字符串或意外重置外置 HOME。保护目录规则按归属 / 权限重新设计，不能为适应 Program Files 直接删除现有安全保护。卸载默认保留用户数据；删除须明确选择且仅作用于经校验的自有目录。

## 5. 自动检查性能与缓存门禁

这里是实施预算与测量计划，尚无 0.1.10 实机性能结论。当前原型内存缓存、分钟评估 timer 和 app.ready 15s 不作为生产性能方案。

| 项目 | 生产目标 |
|---|---|
| 启动 | UI 就绪后 30–60s 内低优先级启动一次到期评估，先读持久状态；本次契约已取 45s，不能阻塞首屏或与安装 / 迁移抢资源 |
| 自动周期 | 管理器 stable 24h、beta 6h；面板 24h；手动模式关闭新自动检查；任务注册不自动给其他模块加周期 |
| 调度 | 一个集中 nextDue 单次定时器，事件驱动重算；不使用分钟网络轮询、不逐页面建 timer；挂起恢复只处理一次到期，不补跑历史周期 |
| 并发 | 每对象 single-flight 与 generation；自动查询串行；手动复用在途检查，明确更新阶段禁止新事务；通道改变只使管理器状态失效 |
| 网络与进程 | 只查有界版本元数据；面板避免连续多个 npm view 子进程，保留代理 / 来源 / 完整性语义；说明按需、按版本缓存 |
| 退避 | 5min → 30min → 2h，最多三次快速重试后回正常周期；网络恢复、路由切入不能绕过 nextAllowed；总 deadline、响应体上限与取消 / 子进程清理在 P0 定稿 |
| 自动行为 | 到期只检查和通知，不自动下载安装 / 重启；缓存命中立即展示，过期背景刷新；无新结果不反复发通知 |

源代码风险：0.1.9 面板远端查询可串行执行版本与完整性两次 npm 查询，每次超时上限 30s；这不是实测耗时，但意味着需单独控制最大等待、子进程与 UI 响应。生产缓存需校验 schema、来源 / 通道、版本、时间与大小，坏缓存安全重查；系统时钟前移 / 回拨和缓存跨版本兼容须覆盖，不能因异常时间永久不检查或每次启动风暴。

必须完成以下实测，禁止只用加速时钟判长期性能通过：

1. 未过期缓存，100 次概览 / 设置切入、10 次重启：自动网络查询和查询子进程均为 0；损坏 / 过期缓存另测。
2. 启动、路由、到期、恢复等同时到达，每对象只发一次；手动不叠加；管理器切通道后面板缓存 / 退避 / 在途事务保持。
3. 离线期间不循环派生查询；真实慢网 / 超时结束后子进程、锁、监听和 timer 无残留；安装 / 迁移期间调度延后，恢复不集中补跑。
4. 同一设备与 0.1.9 基线配对，测启动到可交互 p95、帧耗时 / 长帧、自动检查峰值 CPU / RSS、闲置 CPU / RSS、请求数与字节量、子进程数 / 存活时间。P0 记录工具、场景、重复次数与允许变化预算，P2 / P7 按预先预算判定，不能测后选阈值。
5. 加速状态测试覆盖 24h / 6h 周期逻辑；另记录至少一个真实 stable 24h 与 beta 6h 的到期窗口、休眠恢复和跨启动行为。长时数据不足则保留待验，不称完整性能门禁通过。

## 6. 更新详情与事务反馈

共用状态服务提供对象、通道、运行版本、候选版本、来源、说明状态 / 内容、可选网页 URL、发布时间和取得时间；检查完成后再查看详情，概览对象行也可单独查看。结果弹窗上下布局，详情的说明区域内部滚动、头部 / 操作区固定；设置入口使用同一组件和事务，不保留另一个简化流程。

管理器说明来自精确候选的 updater notes 或对应 GitHub Release body，用户可见发布说明以 main 的 CHANGELOG 为源生成。面板来自官方 npm 元数据及经核验的 repository / release 映射，说明缺失、获取失败和不支持必须区分；无可靠映射不编造链接。远端说明按受限 Markdown / 文本渲染，限制大小，禁止脚本、任意 HTML 和远端图片；外链经受控 HTTPS 来源校验。说明失败不混同签名失败，也不能绕过更新来源或签名校验。

流程为检查 → 详情 / 确认 → 可选管理器偏好备份 → 准备 → 下载 → 验证 → 应用 → 面板版本 / 入口回读，或管理器待重启 → 重启后版本回读。下载已知总字节才显示百分比；验证 / 应用无可信总量则显示阶段动画和文字。中 / 低档静态进度仍可读，高档动效继续按软件档位，不追加系统动态开关。

operationId、对象、候选、通道、序号与世代号防迟到结果覆盖。关闭弹窗只隐藏显示，跨路由复用进度；后台支持的可取消阶段才提供取消，写入阶段保留保护。安装就绪不能提前改运行版本；真正回读成功才发完成。备份默认勾选且说明实际偏好范围；选择备份失败阻断更新，取消确认不生成备份；既有强制写前保护不降低。

## 7. 备份与树表共同合同

偏好备份实际为全文件 `preferences.json` 快照和 `backup-manifest.json`，摘要及字节数在 manifest 内，不增加虚构 checksum 文件。现有 v1 每记录一个 payload，其他动作从真实清单确定来源，不固定成偏好文件；发现、列表、轮换与恢复均覆盖支持动作，包括核对 runtime-uninstall 与关联记录。完整迁移保护不能用偏好备份替代。

保留默认最近 N=10 份 **或** D=30 天内，满足其一保留，固定项另保留且不占 N；按已冻结的用途分组策略计算，并向用户说明不是最多十份。清理默认预览 + 手动确认，更改策略不立刻删除；若提供自动轮换，仅在新备份成功后按已选择策略执行。事务中的恢复前保护及关联组不能被拆删；成功后解除临时保护并回到普通保留策略，失败仍保留，用户固定另计。

树表外层始终显示，初始根 / 年展开、月份收起，后续刷新保留用户展开和焦点。名称 / 用途 / 操作三列从活动根与实际 manifest 构建；打开文件 / 目录前警告，原生端再验归属、存在、权限与链接逃逸。打开不代表只读，也不授予恢复 / 删除权限；系统打开失败须反馈。

### 7.1 唯一树表组件与全部入口

P0 盘点所有带层级行的生产入口、共享原型入口及已有业务组件，建立“入口 / 文件 / 是否含选择框 / 业务选择规则 / 组件接入 / 证据”矩阵，不能仅按名称含 Tree 搜索。2026-10-10 有界盘点完成：生产唯一层级表入口为 BackupFiles.vue + api.ts，接入 UiTreeTable.vue + treeTable.ts；设置 WebDAV / 迁移为平面表单，菜单 / 下拉 / 更新折叠区域不是树表。生产没有 SyncRoute 或导入比较树。共享原型 tree-table.js 已接入 maintenance-ui.js 备份、sync-config.js 同步范围 / 分组 / 导入比较、model-config.js 变更比较 / 策略差异 / 模板保存比较；文件 / WebDAV 两 tab 及事件桥保持。节点呈现与可见行计算共用，业务选择由调用者负责；原型像素与焦点验收待验。

- 生产共享组件统一负责列布局、缩进、展开、行状态、焦点、选择呈现与滚动；节点 ID 稳定，数据 / 操作由调用方提供。原型也复用一个共享树表实现；因运行环境不同可有适配层，但不能每页面再复制布局规则。
- 名称单元格统一槽位：缩进、选择框（可选）、展开箭头或叶节点占位、图标、文字；复选框、箭头和图标共用对齐行及命中尺寸，单行垂直居中。无箭头 / 无选择框时遵守统一占位策略，同层文字起点一致，不能靠每页面 padding-top 微调。
- 长文字按统一截断 / 换行规则显示；多行时控件对齐首行的共同中心，不随整个多行块高度漂移。列内容 / 操作对齐遵循相同首行基准，主题、字号、密度和缩放改变后仍一致。
- 选择和展开是不同操作：点击箭头不改变勾选，点击复选框不展开；单击 / Space / Enter、焦点与 aria-expanded / checked / mixed 保持一致。父子级联、依赖强选、过滤后全选、禁用节点及半选语义由业务合同明确注入，不把备份浏览树默认变成级联选择树。
- 使用语义表格加可访问按钮，或完整符合键盘合同的 treegrid；P0 冻结一种，不只增加 role 而缺失导航 / 层级语义。目录按钮可键盘展开，长列表保持滚动 / 焦点；数据量实测后决定是否需要虚拟化。
- 组件样例及测试覆盖无 / 有复选框、全选 / 半选 / 禁用、叶节点、深层、长名称 / 多行、空 / 加载 / 错误、单行操作按钮、过滤刷新、焦点保留。两平台浅深主题 / 三档 / 标准紧凑及实际缩放截图检查对齐；不能用 DOM 快照替代像素与人工操作验收。

所有现有生产树表接入同一组件后才算 W6 完成。未来新增树表直接复用；0.2.0 原型组件接入不构成其业务实施或验收。

## 8. 统一事件范围与触发安全

全产品登记覆盖更新、备份 / 恢复 / 清理、运行与面板、安装 / 修复 / 移除、文件 / WebDAV、模型 / 账户 / 扩展、诊断 / 设置 / 数据根以及公共事务反馈。由配置分别定义 triggers、jobs、events、分类级别、一次性 / 状态 / 周期、投递、去重、冷却、解除与脱敏；更新以外周期（如候选诊断 6h、WebDAV 15min）显式 planned / 关闭，不因全量登记开始运行。

发出点矩阵逐行标记现有能力已接入 / 不适用 / 未来未启用，提供真实文件、函数、对象 / 动作 / 阶段与错误码。发出通知前校验注册和来源；进度 tick 只更新任务显示，不刷通知中心。普通展开 / 导航 / 筛选 / 已读不再制造通知。后台无更新不弹窗，新候选去重进中心，自动失败按策略记录，手动检查即时反馈。

异常解除要求同对象、动作、阶段、通道 / 候选及相关操作的真实成功证据。检查成功不能解除应用失败，其他对象成功不能解除原对象失败，切通道不是异常恢复。管理器通道切换不清面板缓存、退避、确认或在途事务；更新写入 / 待重启阶段阻止冲突切换。通知不存凭据、原始异常、账户显示名或完整路径，诊断由受控事务 ID / 错误码定位。

稳定后统一事件成为项目核心中的单一长期事实源，后续新增事件同步登记；本轮不提前写入“生产已具备”的核心事实。

## 9. 测试矩阵、证据与审阅门禁

| 层级 | 必须覆盖 | 验收边界 |
|---|---|---|
| 静态 / 单元 / 集成 | 类型、前端构建、Rust fmt / Clippy、前后端全套回归、协议与发布工具；存储原子性、调度合并、候选世代、事件解除、清理保护、树表选择 / 展开与焦点 | 按当前源码和工具实际记录项数 / 忽略原因，不固定沿用旧版本数量 |
| 真实原生 UI | macOS Apple Silicon / Windows x64；概览与设置共用更新、树表对齐 / 键盘、弹窗滚动、主题 / 三档、光层全边距、CSS / WebGL / 静态回退 | Windows 普通 / 管理员分别测；实际 100% / 125% / 150% / 200% 缩放按设备能力覆盖，记录真实 DPI 与 WebView DPR，模拟不替代物理验收 |
| 生命周期 | 最小化 / 隐藏 / 挂起 / 网络恢复 / 退出 / 重启，托管面板与外部 HOME，通知跨启动去重、晚到回调、进度不丢失 | 0.1.9 原生标题栏、菜单迁移、关闭 / 托盘 / Snap 等相关回归保留；既有未覆盖项按适用性登记，不自动豁免 |
| 数据与失败 | 旧格式 / 自定义根 / 不可写 / 空间不足 / 冲突 / 损坏 / 非法路径 / 缓存清除 / 时钟改变、备份失败阻断、恢复中断与回滚、关联记录清理 | 用隔离可复现数据，验证文件内容和绑定，不只截图 toast；真实用户迁移单独确认 |
| 安装与升级 | macOS 当前候选安装 / 卸载；Windows MSI / NSIS 新装、0.1.9 升级、自定义位置与受保护路径、卸载保留 / 明确清理、运行版本首屏 | 精确制品 SHA 与版本；需旧版 OTA 时覆盖已承诺兼容路径，包括专用设备 0.1.7，不自动替换日常安装 |
| 性能 / 长时 | 第 5 节配对测量、缓存命中零请求、离线 / 慢网子进程收尾、真实周期到期与休眠恢复 | 单测、原型、真实周期与硬件测量四类证据分开；未测标待验 |
| 发布 | main 集成 CI、双平台 Release 构建、三种更新资产旧信任公钥密码学验签、公开端点与 Release SHA、真实下载 / OTA | updater 签名不等于 Apple 公证 / Authenticode，平台分发状态分别明确 |

阶段任务使用稳定检查名，例如 `review:scope-0110`、`review:storage-0110`、`review:scheduler-performance-0110`、`review:updates-0110`、`review:backup-tree-0110`、`review:event-coverage-0110`、`review:native-0110`、`review:installer-0110`、`review:core-writeback-0110`、`review:release-0110`。执行时通过治理 provider 建 TASK，required_checks 对应 RCP.checks 与真实证据；本次不伪造未开始阶段的通过回执或关闭证书。

每份回执带源码 SHA、制品 SHA / 版本、设备 / OS / 权限 / 实际缩放、隔离数据标识、命令 / 操作、结果、截图或日志及未覆盖原因；失败原件保留，修复后新增回执。大证据与摘要分开，任何清理先预览。治理任务关闭只证明对应任务完成，不能扩展为版本已发布。

## 10. 稳定事实、提交与发布闭环

P0 / 开发过程：需求管理维护范围及可验 AC；本 IMP 记录决策、实现提交与未验项；协作包保留设计和问题来源。保持当前核心事实正确，本次不把候选规则复制进核心造成“双事实”。

P8 经真实验证并取得用户后续确认后，按各自权威位置回写：

| 落点 | 长期事实 |
|---|---|
| docs/02-项目核心 的统一事件与通知文档（具体名称入库时确定） | 全量事件 / 来源 / 任务 / 去重 / 解除 / 保留规则，后续新增事件均维护此处；其余核心页引用而不复制全表 |
| 数据与状态 / 领域模型 / 契约字段 | 活动根、平台默认 / 例外、迁移兼容、缓存与调度状态、备份 manifest / 范围 / 恢复保护，按事实归属各写一次 |
| 产品定义 / 能力地图 / UI规范 | 已具备更新能力、实际备份边界、共享更新与树表组件、按钮和光层规则；未来能力保持未实施 |
| docs/01-需求管理 | 真实 AC 验收与证据关联，不用协作包闭合替代验收 |
| docs/03-开发实施 的 IMP / REL / 必要 RUN | 实施与发布决定、制品身份、兼容路径、风险和观察，保留历史版本结论 |
| main 的 CHANGELOG / 必要共享 README | 面向用户的实际更新内容、兼容 / 迁移限制和已验证下载入口 |

提交按产物分组并复核暂存范围，文档提交 docs/governance-main，代码提交 feature / main；不顺带提交未归属的 apps 或用户其他工作。同步提交需可追溯源 SHA 与目标 SHA；核心、需求和索引由一个集成入口回写。候选制品制作与公开发布分开，CI 与候选打包职责明确，避免重复 publish 触发；失败的门禁不能因为另一个打包 job 成功而删除或忽略。

发布前提交可审阅检查单：版本和 main SHA、三类资产及摘要、签名 / 兼容验证、双平台 / 性能验收、未覆盖与阻塞、核心回写、旧目录 / 卸载 / 迁移说明、回滚路径。用户确认本版本后才执行公开发布与 stable 晋升；任一阻塞未闭合保持候选。

发布后核对固定版本和 stable/latest.json 公开内容、资产 URL、版本 / 平台 / 签名与当前 SHA，下载安装包和 OTA 在隔离设备验证；记录没有替换哪些日常安装。出问题先停止或撤回新通道投递，再确认回退方案，不能默认降级二进制能还原已变化数据；旧数据保护和兼容测试必须先准备。观察窗口在 P0 明确，并覆盖至少一个正常自动检查周期及恢复场景。

## 11. 2026-10-11 当前实现、证据与下一步

代码位于 feature/0.1.10-maintenance，基线 main@de6862d40389d527527b9db8ef52e186a58b625f，当前 HEAD 73458394，已推送，尚未合入 main，版本仍为 0.1.9。基础提交与逐项证据见执行基线；本轮补齐最终绑定提交、目标锁保留、重启锁内核验与迁移命令 / 确认界面，并接入更新偏好保护的完成核验及显式隔离容器边界。源码行为不替代版本验收。

1ef59403 集成：UI 全量 93 文件 / 431 项通过，类型检查与生产构建通过；Rust 全量 32 组 / 726 项通过、0 失败、4 忽略，Clippy all-targets -D warnings 与 fmt 通过。迁移命令定向 7 项、迁移确认挂载 4 项、目录 store 11 项通过。生产 Chromium 集成浏览器 428/428、顶部光层定向 99/99，无新增页面错误。共享原型 Node 测试 56/56、26 条资源、247 个静态 ID 无重复、两个内联脚本语法通过；原型浏览器被工具访问策略拒绝，未以 HTTP / CDP 或其他浏览器绕过，实屏与焦点保持待验。生产浏览器通过不抵原型或原生验收。历史失败及重跑均保留，路径与命令见执行基线。

reference-only 已有全写者冻结：要求 runtime 新鲜停止 / 未发现、无安装与待重启、写者计数为零，忙则拒绝；保存失败 / 无变化解除冻结，成功变更后当前进程拒绝新写事务，重启统一重绑定。没有等待清空 / 取消在途写者；只读状态与安全打开继续可用，OS 自有缓存、Unix IPC 退出清理及外部编辑属于明确例外。npm cache 位于活动根 cache/npm，真实 HOME / USERPROFILE 不改写。复制内核已有有界清单、双端摘要复核、独占目标与中断标记；ff000c55 仅在未发布目标内准备托管运行入口，保留外部 Node / 前缀与历史备份原文。3f7dce76 / 1ef59403 已接通 MigrateData：未发布目标准备与持久标记校验后原子提交绑定，立即冻结旧进程写者并保留目标锁，重启在锁内核对清单后才准入；结果不确定保持待核对与冻结。确认界面固定捕获目标、转义目录文案并保留旧活动路径直到重启。自动化覆盖不等于原生重启 / Windows 验收，未迁移真实用户数据。

macOS 隔离 debug 原生备份已保存截图、AX、运行身份、清单与摘要到协作包验证目录。源码 79821797 的首份备份后显示到月份、月份下级折叠；该证据不覆盖不同值恢复、安装器、实际更新、全主题 / 画质或配对性能。先前未继承隔离环境的启动尝试保留在记录中。未改日常安装。

生产注册表现有 8 triggers、28 jobs、56 events（54 active / 2 planned）、15 notification policies、62 emission sites、8 scheduling sites、7 UI feedback groups、12 paths、4 cleanup policies。信号校验、通用反馈、领域终态与持久通知分开核对；矩阵见事件规划。全产品领域动作 / 准入失败 / 恢复解除仍有缺口，不能称全量覆盖通过。生产与原型树表入口已盘点，共享组件接入已实现；原生对齐仍待验。

7ce357b2 将 PreferencesProtection 的临时保护与用户固定分开：失败 / 中断保留，面板真实版本核验后解除，管理器精确运行版本 / 候选 / 通道重启核验后解除关联备份；解除前持久回执，投递失败后即便备份已轮换仍可重试。同候选旧尝试不能覆盖新关联，未知 / 旧格式不推定解除。面板安装成功但保护标记失败继续显示安装成功与待核对提示；目前没有持久自动重试关联，不承诺自动解除。源码回归：全量 Rust 734 通过 / 4 忽略，随后新增同候选防覆盖的交接定向 7/7；UI 93 文件 / 432 项、类型检查 / 构建通过；全量 Clippy 在最后防覆盖补丁前通过。精确范围与失败原件见协作包验证记录，不将这些结果称为原生安装验收。

6da3cc28 允许显式 OPENCODEX_SANDBOX_BOUNDARY 隔离容器内的同级迁移；省略时保留 anchor 范围，不自动扩大父目录。启动前解析且捕获边界，初始化 / 引用 / 迁移 / 外部 HOME 写入均拒绝越界。Rust 全量 32 组 / 737 通过 / 4 忽略覆盖最终更新交接防覆盖与基础边界；随后最终迁移定向 8/8、Clippy all-targets -D warnings 与 fmt 通过。实际执行范围及原生启动合同见协作包验证/20261010-隔离容器边界，不推定原生已验收。

剩余重点：复制迁移原生重启与 Windows 验收、保护解除的原生更新 / 重启证据及面板待核对异常处理、Windows 安装路径与普通权限、查询进程树、升级 / 卸载保留、macOS 隔离原生安装及不同值恢复、同机 0.1.9 配对性能及真实 stable 24h / beta 6h 周期。开发浏览器性能只作定位，不算原生门禁。TASK-182 与 IMP-20 保持 in_progress，required_checks 未全通过，未颁发完成证书。稳定事件核心事实回写保留到验证后的确认阶段；没有真实用户迁移、日常安装替换、公开发布或 stable 晋升。

详见 [执行基线与契约](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/06-执行基线与契约.md)。

### 2026-10-10 隔离迁移 / 恢复重启补充

6da3cc28 显式隔离边界之后，8a318f8e 区分 canonical 与字面路径身份，1f18ff77 排除别名导致的误待重启，986299bf 区分新鲜运行状态未核实与确实任务占用；未知状态仍拒绝迁移。986 定向 Rust data_root 10/10、UI 2 文件17/17、Clippy all-targets、构建与 custom-protocol 原生构建通过，属于局部证据。

macOS 隔离原生复制迁移后重启核对通过；迁移目标的异值偏好恢复 system→dark、恢复前保护生成 / 精确关联 / committed、再重启持久回读通过，旧源保持 system。详见协作包“验证/20261010-macos迁移与异值恢复/README.md”。不覆盖真实用户迁移、Windows、安装器或版本整体验收。此前“macOS 原生迁移 / 不同值恢复待验”由本局部证据更新，失败矩阵仍待验。面板保护写入失败的持久重试关联、全领域事件、跨平台与性能门禁继续未完成；TASK/IMP 保持 in_progress。

### 5401dd5c 面板偏好保护持久核验

补齐精确安装尝试 / 版本 / 备份摘要的持久关联与激活证明，先持久化核验再解除临时保护；启动仅一次本地后台重试，不联网、不执行包。旧段落“没有持久自动重试关联”是历史状态，本补充取代其当前实现结论。未决 / 损坏关联阻止覆盖安装、卸载及目录切换 / 迁移；失败尝试不解除备份，用户 pinned 独立。损坏或无法判断的关联仍缺受控修复入口，保持未完成。

最终源码回归 Rust 33 组 / 753 通过 / 0 失败 / 4 忽略，fmt 与 Clippy all-targets -D warnings 通过；失败原件保留。UI 本补丁未改动且未重跑。见协作包“验证/20261010-面板保护持久核验/README.md”及 .adg/evidence/OCX-0110-20261010-PANEL-PROTECTION 的身份与摘要。原生更新 / 安装器、全领域事件、Windows 与配对性能 / 真实周期未通过；不闭合 TASK / IMP，不回写稳定核心或发布。

### 7d5b2e9d 卸载核验与事件补充

范围及确认在任何停止 / 写入前校验；工作线程持有准入至终态，实际步骤成功且无失败、非空残留全部 cleared 才写 succeeded 并显示 100%。exit 0、unknown / 缺证据 / 全跳过不推定成功；历史保留移除前真实来源版本。新增用户触发的卸载终态失败 / 成功事件，范围为 runtime / uninstall / execution / local；同根世代与候选成功才解除，通知只保存摘要，不存完整路径。

最终 Rust 33 组 / 756 通过 / 0 失败 / 4 忽略；fmt、Clippy all-targets -D warnings、UI 类型检查 / 生产构建通过；UI 全套 93 文件 / 438 项、卸载定向 8 项通过。见协作包“验证/20261010-卸载核验与事件/README.md”及 .adg/evidence/OCX-0110-20261010-UNINSTALL-VERIFICATION。准入前拒绝未向冻结根写持久通知，全域错误映射、保护回执受控修复、原生卸载 / Windows 与性能门禁仍未闭合；TASK / IMP 保持 in_progress。

### 56c8462c 保护状态、限定本地重试与退出码回收修复

设置已有只读保护状态与显式本地重试，损坏记录不暴露字段、不覆盖；pending / verified_pending 只有精确关联与激活证据通过才解除。重试持有运行变更准入，安装及管理器待重启冲突拒绝；失败与成功通过同一候选投递限定范围事件。损坏关联受控修复仍未具备，本地重试不能替代它。

0be9d8b3 修复 Unix Tokio 孤儿回收与裸 waitpid 竞争造成非零退出码丢失；8 路 / 512 次回归复现前失败、修复后通过。最终 Rust 33 组 / 765 通过 / 0 失败 / 4 忽略，fmt / Clippy 通过；UI 94 文件 / 447 项、类型检查和构建通过。当前计数 8 triggers / 29 jobs / 58 events（56 active / 2 planned）/ 16 policies / 64 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup，历史数量保留。详见协作包“验证/20261010-保护重试与进程回收”及 .adg/evidence/OCX-0110-20261010-PROTECTION-RETRY-REAPER。原生、跨平台、性能与事件全域门禁继续待验，不颁发完成证书。

### d3e4df32 来源绑定保护与隔离原生复验

56c8462c 隔离原生复验暴露来源切换未被保护阻断，失败 AX / 截图保留。d3e4df32 在来源变更锁内补齐后端门禁；前端未知 / 读取中 / 未决 / 不可读状态禁用两来源入口，文件选择返回再检查。定向 UI 12/12、Rust protection 16/16，类型检查 / 构建 / fmt / Clippy / custom-protocol debug 构建通过；未重跑本提交全量。

macOS 27.0.1 arm64 隔离原生中两个来源按钮、安装与离线导入均禁用；损坏记录本地重试拒绝，记录与四份备份文件摘要不变，字段未暴露。见协作包“验证/20261010-来源绑定保护门禁”及 .adg/evidence/OCX-0110-20261010-SOURCE-BINDING-GATE。来源绑定局部门禁通过，损坏关联受控修复、实际更新 / 重启、Windows、全域事件、性能与安装器仍待验，不闭合任务或发布。

### 27e425bf 共享树表刷新 / 重试焦点

macOS 隔离原生发现键盘刷新丢失焦点，修复共享组件刷新可聚焦性与加载 / 失败重试焦点恢复；树行删除不抢外部控件焦点。最终 UI 94 文件 / 454 项、类型检查 / 生产构建、custom-protocol debug 构建与 diff 检查通过；本提交未重跑 Rust 全量。

隔离原生默认到月份、键盘展开 / 折叠、刷新保留焦点与展开状态、失败重试聚焦 / 修正后回树行通过。只注入并移除临时链接，两份备份四文件摘要不变；未安装、恢复或迁移真实数据。旧失败 AX / 截图保留。见协作包“验证/20261010-树表刷新与重试焦点”与 .adg/evidence/OCX-0110-20261010-TREE-FOCUS。Windows、全矩阵像素 / 复选框、安装器、实际更新 / 重启、全域事件及性能门禁继续待验，TASK / IMP 保持 in_progress。

### d4f4dc29 设置终态事件补充

保存与恢复默认真实写入终态已登记；同动作 / 完整配置候选成功才解除失败，通道或动作变化不误解除；校验与准入拒绝只即时反馈，通知失败不影响设置已提交结果。注册表增至 31 jobs / 62 events（60 active / 2 planned）/ 18 policies / 68 emissions，其余维度不变；不代表全域覆盖完成。

最终 Rust 33 组 / 769 通过 / 0 失败 / 4 忽略、设置定向 7/7、注册表 31/31、fmt / Clippy / diff 检查通过。首次测试误断言的失败日志与修正证据保留。见协作包“验证/20261010-设置保存与恢复事件”及 .adg/evidence/OCX-0110-20261010-PREFERENCES-EVENTS。未改 UI / 未重跑原生；平台、全域事件、性能与发布门禁继续未完成，TASK / IMP 保持 in_progress。


### d24bd4d5 备份策略保存终态

策略实际保存完成后，在共享备份锁内投递失败 / 成功；完整策略摘要、动作与数据根世代隔离候选，精确成功只解除旧失败。校验 / 锁拒绝只即时反馈，通知失败不改变真实保存结果。保存自动轮换策略不立即清理：12 份已满足清理条件的备份完整清单保持不变。未增加计时器、网络查询或路径。

最终 Rust 33 组 / 773 通过 / 0 失败 / 4 忽略；定向 4/4、注册表 31/31、fmt / Clippy all-targets -D warnings / diff 检查通过。首轮工作目录错误实际 0 项定向，原日志保留且不作为通过结果。当前生产 8 triggers / 32 jobs / 64 events（62 active / 2 planned）/ 19 policies / 70 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；原型表独立。见协作包验证/20261010-备份策略保存事件及 .adg/evidence/OCX-0110-20261010-BACKUP-POLICY-EVENTS。UI / 原生未重跑；全域事件、双平台、安装器和性能门禁仍待验。TASK / IMP 保持 in_progress，不颁发完成证书，不回写稳定事件核心事实或发布。

### c4996219 备份固定状态精确核验与终态

共享备份锁内完成有界准入、保护核验、manifest 写入与成功后的完整回读，精确一致才发 succeeded；根世代、备份、摘要和 pinned 状态隔离候选。同候选成功只解除失败，不新增成功提醒。无效 ID / 非法锁路径 / 未决保护只即时拒绝，取消固定不绕过保护；通知失败不改变已核验落盘。合作锁占用会等待，未改变生产锁行为。

最终 Rust 33 组 / 778 通过 / 0 失败 / 4 忽略；固定定向 5/5、注册表 31/31、fmt / Clippy all-targets -D warnings / diff 通过。首轮零定向、字段编译失败、夹具失败及递归阻塞中止原件均保留，不作为通过证据。当前生产 8 triggers / 32 jobs / 66 events（64 active / 2 planned）/ 20 policies / 72 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。见协作包验证/20261010-备份固定状态事件及 .adg/evidence/OCX-0110-20261010-BACKUP-PIN-EVENTS。未重跑 UI / 原生；同步与全域事件、跨平台、安装器、实际更新 / 重启和性能门禁仍未闭合，TASK / IMP 保持 in_progress，未回写稳定核心或发布。

### a3b84474 同步连接测试所有权与终态

owned worker 持有存储准入至有界探测 / 回读 / 终态完成，取消 IPC 不释放执行；进程内非排队操作门覆盖连接测试、端点保存 / 删除及现有手动同步，忙拒绝不发网络失败。端点与凭据回读一致才按真实网络结果投递 sync-connection failed / succeeded；同根世代 / 动作 / 候选成功才解除，通知写失败不改变网络结果。user-only execution probe 不放宽 query / progress 恢复禁令，planned 同步能力仍未启用。手动同步直接 await 的取消所有权、导入 / 外部写入及跨进程串行化不在本轮保证范围。

最终 Rust 33 组 / 785 通过 / 0 失败 / 4 忽略；定向 7/7、注册表 31/31、fmt / Clippy all-targets -D warnings / diff 通过；旧源码零效轮次、编译失败和夹具失败原件保留。生产 8 triggers / 33 jobs / 68 events（66 active / 2 planned）/ 21 policies / 74 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。见协作包验证/20261011-同步连接测试事件及 .adg/evidence/OCX-0110-20261011-SYNC-PROBE-EVENTS。未重跑 UI / 原生；端点变更 / 手动同步终态及全域事件、平台、安装器、实际更新 / 重启和性能门禁仍待验，TASK / IMP 保持 in_progress。


### 153430bf GUI 手动同步任务所有权

GUI 手动同步在 owned blocking worker 内读取配置 / 凭据、运行既有 runner 并投影冲突通知与结果；观察者取消后任务与两种准入持续持有至返回。端点保存 / 删除与连接测试复用入口。忙 / 冻结零任务执行，worker 错误 / panic 释放准入；panic 测试不代表原生状态终态已验证。上传 failed DTO 与其他错误契约保持。未新增完整手动同步注册终态、定时器或自动检查频率。

最终 Rust 33 组 / 788 通过 / 0 失败 / 4 忽略，sync 定向 14/14（新增所有权 3/3、probe 7/7），全量内注册表 31/31；fmt / Clippy all-targets -D warnings / diff 通过。首次工作目录路径错误及旧源码测试原件保留，不作补丁证据。注册表计数不变，未重跑 UI / 原生。见协作包验证/20261011-手动同步任务所有权及 .adg/evidence/OCX-0110-20261011-SYNC-OWNERSHIP。

a3b84474 的直接 await 结论保留为历史，本轮仅更新 GUI 入口所有权。CLI/IPC、导入 / 外部写入、跨进程、端点 / 手动同步终态及全域事件、保护损坏受控修复、平台 / 安装器、真实更新 / 重启与性能门禁仍未闭合。TASK / IMP 保持 in_progress，不颁发完成证书，不回写稳定核心或发布。


### 73458394 端点变更核验与终态

保存精确回读配置与口令；删除只接受明确凭据缺失，删除核验失败保留配置引用供重试。输入与最终配置校验先于变更。凭据与配置没有跨存储原子回滚，保存 / 删除失败可能部分提交，不承诺完全未变更。

新增用户触发的 sync-endpoint-save / delete 失败与成功对；完整请求候选仅持久化摘要，同根世代 / 动作 / 候选的精确成功才解除对应失败。通知失败不改变真实变更结果。生产计数更新为 8 triggers / 35 jobs / 72 events（70 active / 2 planned）/ 23 policies / 78 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；旧数量保留为历史。无新增计时器或路径，planned 领域未启用。

最终 Rust 33 组 / 798 通过 / 0 失败 / 4 忽略，端点定向 9/9、缺失分类 1/1、注册表 31/31、fmt / Clippy all-targets -D warnings / diff 通过。见协作包验证/20261011-端点变更核验与事件及 .adg/evidence/OCX-0110-20261011-ENDPOINT-MUTATION。未改 UI / 未重跑原生；注入凭据不能替代平台原生核验。

手动同步终态、CLI/IPC 所有权、全域事件、保护损坏受控修复、Windows / 安装器、实际更新 / 重启与配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，不颁发完成证书，不回写稳定核心或发布。


### 82c3479b 手动同步终态与界面反馈

b2307736 核验最终 latest 指针 PUT 只接受 201 / 204，202 或协议异常不作成功。82c3479b 在原有任务所有权内补齐捕获内容后的手动执行终态；端点与凭据精确回读一致才投递成功。候选绑定端点、凭据和实际内容，仅保存 SHA256；同根世代 / 动作 / 候选成功只解除对应失败。冷同步、锁与捕获前拒绝不虚构执行终态；冲突保留旧提醒，不发同步成功、不声称解除冲突。

界面只把 synced / succeeded 作为成功；冲突、失败 DTO 及异常都刷新通知，保留真实状态，说明可能已有部分内容应用或上传。同步没有全事务原子回滚，也未证明远端持久落盘。缺失偏好恢复冷同步默认的回归通过；不把它当成空捕获分支已验证。

最终 Rust 33 组 / 805 通过 / 0 失败 / 4 忽略；同步定向 60 通过 / 1 忽略、WebDAV 10/10、注册表 31/31，UI 94 文件 / 459 项、类型检查 / 生产构建、fmt / Clippy all-targets -D warnings / diff 通过。编译、异步断言和冷同步预期失败原件保留。生产 8 triggers / 36 jobs / 74 events（72 active / 2 planned）/ 24 policies / 80 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；没有新定时器或路径。证据见协作包验证/20261011-手动同步终态与反馈及 .adg/evidence/OCX-0110-20261011-MANUAL-SYNC-TERMINALS。

本轮未重跑原生。CLI/IPC 所有权与冲突出口、全域事件、损坏保护受控修复、Windows / 安装器、实际更新 / 重启与配对性能 / 真实周期仍待验；TASK / IMP 保持 in_progress，无稳定核心事实回写或发布。


### d1ab651c CLI 同步所有权与终态

Unix CLI 服务绑定运行中 GUI 的共享状态 / 通知，复用 GUI 核验执行体与准入。冲突明确返回 TargetStateConflict，端点 / 凭据回读不一致返回 ExecutionFailed；配置读取错误与 GUI 上传失败 DTO 契约保持。取消观察者后，owned worker 持有存储与同步准入至真实终态、脱敏审计完成；忙 / 冻结 / 未确认零执行及执行审计，panic 失败审计后释放。审计失败只记日志，不覆盖已执行结果。

最终 Rust 33 组 / 809 通过 / 0 失败 / 4 忽略，IPC / GUI 各 7/7、fmt / Clippy all-targets -D warnings / diff 通过。首次错误枚举编译失败及补齐 setup 契约前轮次保留，不作为最终补丁证据。取消测试为注入执行体配合真实文件系统 / 准入 / 审计；不能替代真实 WebDAV、原生通知或跨进程证明。未改 UI / 未重跑原生；注册表 8 triggers / 36 jobs / 74 events（72 active / 2 planned）/ 24 policies / 80 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup 不变，无新增计时器或路径。

见协作包验证/20261011-CLI同步所有权与终态及 .adg/evidence/OCX-0110-20261011-CLI-SYNC-OWNERSHIP。本轮更新之前 CLI 同步所有权 / 冲突出口待验结论；其他 CLI 写入、全域事件、损坏保护受控修复、Windows / 安装器、实际更新 / 重启与配对性能 / 真实周期继续待验。TASK / IMP 保持 in_progress，不颁发完成证书、不回写稳定核心或发布。


### bc43e460 CLI 手动备份策略与所有权

CLI 复用 GUI 的偏好捕获 / 格式校验 / ManualPreferences 分类 / 保存策略与终态；后台执行者持有存储准入至真实结果与脱敏审计完成，取消观察者不取消执行。列表保留原字段并包含手动与旧事务，有界扫描 / 锁等待移出异步执行器，列表取消后仍持有准入。缺失 / 损坏源与坏策略统一返回 ExecutionFailed（旧缺失源为 TargetNotFound）；冻结返回 TargetStateConflict。成功备份后的清理失败不改写成功。

最终 Rust 33 组 / 815 通过 / 0 失败 / 4 忽略，IPC 定向 13/13（新增 6 项），fmt / Clippy all-targets -D warnings / diff 通过。首轮冻结错误码断言失败与旧成功轮次原件保留。锁等待测试为同进程实锁，不能替代跨进程或原生性能预算；隔离测试不覆盖原生事件传输。注册表计数不变，无新增计时器 / 路径。见协作包验证/20261011-CLI备份所有权与策略及 .adg/evidence/OCX-0110-20261011-CLI-BACKUP-OWNERSHIP。

本轮更新之前 CLI 备份策略 / 所有权待验结论。其他 CLI 写入、全域事件、跨进程 W2、损坏保护受控修复、双平台 / 安装器、真实更新 / 重启与配对性能 / 真实周期仍待验；TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### 904a1323 W2 合作进程锁与中断保护

仅新增集成测试，以独立子进程在临时根调用生产备份、固定、恢复、清理、策略保存与列表 API。持锁期间观测 150ms 未完成、manager-state / backups 字节未变化；释放后完成。恢复使用不同有效偏好并回到备份原值，清理确实删除 2 份到期普通备份。强制终止保护持锁者后，新清理进程可完成并删除 1 份到期普通备份；未决保护仍保留、不能通过取消固定解除。进程锁释放不等于事务核验完成。

当前提交定向 2 通过 / 0 失败 / 1 忽略；忽略项是被父测试显式调用的子进程入口，不是豁免回归。fmt / Clippy all-targets -D warnings / staged diff 检查通过。只新增测试，本轮未重跑全量；bc43e460 的 Rust 33 组 / 815 通过 / 4 忽略仍为历史基线，不能称为 904a1323 全量结果。未改生产行为、注册表、计时器或持久路径，未重跑 UI / 原生。证据见协作包验证/20261011-W2跨进程锁及 .adg/evidence/OCX-0110-20261011-W2-PROCESS-LOCK。

本轮仅证明 macOS 本机合作进程路径；started 标记先于 API 调用、150ms 是观测窗口，不是内部锁获取屏障。不能替代非合作写入、Windows 锁语义、崩溃恢复全矩阵或性能预算。其他 CLI 写入 / 全域事件、损坏保护受控修复、双平台 / 安装器、真实更新 / 重启、配对性能 / 真实周期仍待验；TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### 40f19871 CLI 导入导出所有权与显式 HOME

导入 / 导出移入 owned worker，持有存储准入至真实结果与脱敏审计完成；取消观察者不取消已准入任务。导入使用显式 dependencies.home，修正之前错误使用工作目录的来源。保留现有加密口令限制、exit 13、snake_case 与导入 / 导出 DTO；未新增迁移注册终态，不能称为全域事件已覆盖。

最终 Rust 34 组 / 820 通过 / 0 失败 / 5 忽略，IPC 定向 16/16、fmt / Clippy all-targets -D warnings / staged diff 检查通过。初轮定向日志保留，最终仅改为 Clippy 要求的夹具初始化表达。往返仅核验拓展配置保存，未执行真实 Skills / MCP 客户端资产投射；20ms executor 响应与取消测试为同进程实锁，不能替代跨进程或原生性能预算。注册表计数不变，无新增计时器或路径，未改 UI / 未重跑原生。

证据见协作包验证/20261011-CLI导入导出所有权及 .adg/evidence/OCX-0110-20261011-CLI-MIGRATION-OWNERSHIP。本轮更新 CLI 导入 / 导出所有权与 HOME 待验结论；CLI 启停 / 重启、迁移终态及全域事件、损坏保护受控修复、双平台 / 安装器、真实更新 / 重启和配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### 14b85c11 CLI 启停重启所有权与精确终态

Unix CLI 绑定 GUI 共享运行时变更准入，复用既有生命周期核验与注册通知；后台执行者持有运行时 / 存储准入至真实终态与脱敏审计完成，观察者取消不提前释放。仅 Start / Restart→Started、Stop→Stopped 成功；取消、失败、错误或不匹配终态失败，保留两字段 DTO、请求 ID 和显式执行环境。CLI 仍使用自己的 ControlledProcessRunner，未宣称与 GUI 共用 runner 实例。

同根世代 / 动作 / 候选的成功精确解除已有异常；通知偏好关闭不阻止已有异常解除。候选切换后恢复旧文件产生新世代，不能解除旧世代失败。初轮 19 通过 / 1 失败来自测试错误预期；仅修正测试，保留原件。最终 IPC 定向 20/20、process_action 8/8，Rust 34 组 / 824 通过 / 0 失败 / 5 忽略，fmt / Clippy all-targets -D warnings / staged diff 检查通过。

取消 / panic 与 20ms executor 响应测试使用注入 runner 和真实准入 / 审计，不替代真实原生生命周期或性能预算。注册表数量不变（8 triggers / 36 jobs / 74 events、72 active / 2 planned / 24 policies / 80 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup），无新增计时器或路径，未改 UI / 未重跑原生。证据见协作包验证/20261011-CLI启停重启所有权及 .adg/evidence/OCX-0110-20261011-CLI-LIFECYCLE-OWNERSHIP。

本轮更新 CLI 启停 / 重启所有权待验结论；CLI 查询 / 更新 / 数据根切换、迁移终态及全域事件、损坏保护受控修复、双平台 / 安装器、实际更新 / 重启与配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### f1a0e003 CLI 查询与切根所有权

获准查询持有存储准入至 worker 终态与脱敏审计；冻结后的纯元数据读取不写旧根审计，创建合作锁的备份列表拒绝。切根 worker 持有 GUI 共享运行时变更与独占存储准入，保存引用后错误 / panic / 观察者取消也保持旧根冻结，未保存则重新开放。真实偏好事务锁等待移出异步执行器；引用只写固定 app_data_root，不写 active root。

最终 IPC 定向 25/25，Rust 34 组 / 829 通过 / 0 失败 / 5 忽略；fmt / Clippy all-targets -D warnings / staged diff 检查通过。初轮 23 项成功但有 unused audit 警告的原件保留；最终删除字段并增加公共状态 / 切根实锁验证。20ms executor 响应、取消后落盘与终态矩阵为同进程实锁 / 隔离目录 / stopped shell 夹具，不能替代真实原生、Windows 路径或性能预算。

证据见协作包验证/20261011-CLI查询与切根所有权及 .adg/evidence/OCX-0110-20261011-CLI-QUERY-BINDING-OWNERSHIP。注册表计数不变，无新增计时器 / 路径，未改 UI / 未重跑原生。CLI UpdateCheck 仍为 not_checked / pending 占位，真实更新查询待接通；迁移终态与全域事件、损坏保护受控修复、双平台 / 安装器、实际更新 / 重启与配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### 7a3f290e CLI 真实共享更新查询

UpdateCheck 已接通 GUI 共享真实 updater 查询，共用状态、端点、通道、调度、缓存与终态事件。观察者取消不取消后台任务；持有存储准入至查询终态 / CLI 脱敏审计。过期世代零持久化 / 事件，冻结零 setup / 网络执行；网络失败清除旧详情，持久化失败不报告内存成功。保留 CLI 三字段 DTO，无 GUI AppHandle 明确失败，不返回虚假 pending。

最终管理器定向 14/14、IPC 28/28，Rust 34 组 / 837 通过 / 0 失败 / 5 忽略；fmt / Clippy all-targets -D warnings / staged diff 检查通过。初轮 / 中间轮原件保留，不绑定最终源码。缓存与调度分别写入，不能承诺双文件原子事务；缓存保存后调度失败可能部分落盘，本轮夹具未覆盖第二次写入失败回滚。取消 / 终态测试为注入执行体与真实文件系统，不替代原生 updater HTTP 或安装器。

证据见协作包验证/20261011-CLI真实共享更新查询及 .adg/evidence/OCX-0110-20261011-CLI-REAL-UPDATE。注册表不变，无新定时器 / 路径，未改 UI / 未重跑原生。本轮更新 f1a0e003 的 CLI 查询占位待办；迁移终态、损坏保护受控修复、双平台 / 安装器、真实更新 / 重启与配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，未回写稳定核心或发布。


### bd355e1c 配置导出回读核验与精确终态

目标锁在旧文件读取 / 备份前取得并持有到写入、精确回读和终态回调结束；只有实际文件与预期字节完全一致才成功。旧文件读取、备份、写入或回读失败报告失败；回读失败可能已有目标文件改变，不承诺自动回滚。源文档 / 容器构建与根 / 来源 / 锁拒绝发生在 begin 前，不虚构执行终态。

GUI / IPC 共用用户触发 ConfigExport、Action::Export 与 config-export-failed / succeeded；候选绑定完整文档摘要与无损目标路径，仅摘要持久化。同根世代 / 动作 / 候选的成功只解除捕获的失败；内容或目标切换不能解除之前失败。通知投递失败不覆盖实际导出结果；无 AppHandle 的库调用不虚构投递。generic Migration / Backup / Sync 仍保持 0.2.0 planned，不新增计时器 / 持久路径。

最终迁移定向 61/61、IPC 44/44、注册表 32/32；Rust 34 组 / 844 通过 / 0 失败 / 5 忽略，fmt / Clippy all-targets -D warnings / staged diff 检查通过。首轮注册表 31 通过 / 1 失败是新增测试误用 planned lookup 的返回契约；仅修正断言，原件保留。早期 migration 导出定向 9 项日志不绑定最终补丁。

生产注册表更新为 8 triggers / 37 jobs / 76 events（74 active / 2 planned）/ 25 policies / 82 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。真实临时文件腐败、备份拒绝、源 / 根 / 锁拒绝与持久异常重载重试验证通过，但不能代替原生 AppHandle 投递、跨存储一致快照或全事务原子保证。候选轮换后返回旧内容不复用旧 UUID。未改 UI / 未重跑原生。

证据见协作包验证/20261011-配置导出核验与终态及 .adg/evidence/OCX-0110-20261011-CONFIG-EXPORT；源码 bd355e1c93f1d7f856ca72bb124f916ddb634ab1。导入核验 / 终态、全域事件、损坏保护受控修复、双平台 / 安装器、实际更新 / 重启、配对性能 / 真实周期仍待验。TASK / IMP 保持 in_progress，不回写稳定核心、不关闭或发布。


### 8e54b7f5 配置导入核验与精确终态

合作锁持有至终态；读取 / 保护备份失败阻止写入，逐文件及最终全目标回读一致才成功。失败逆序恢复旧文件或清除新文件并核验，恢复失败明确报错；不保证跨文件原子性或崩溃回滚。GUI / IPC 共用用户触发 ConfigImport / Import / local，文档摘要与无损 HOME 绑定候选；精确重试解除捕获异常，错误通道 / 动作 / 候选轮换拒绝。只导入管理器拓展配置，不投射客户端资产。

最终 Rust 34 组 / 852 通过 / 0 失败 / 5 忽略，migration 68/68、extensions 22/22、IPC 44/44、注册表 33/33，fmt / Clippy / staged diff 通过。历史轮次与首次新增测试 E0599 编译失败原件保留。注册表为 8 triggers / 38 jobs / 78 events（76 active / 2 planned）/ 26 policies / 84 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup，无新计时器或路径。

证据见协作包 `验证/20261011-配置导入核验与终态` 及 `.adg/evidence/OCX-0110-20261011-CONFIG-IMPORT`。本结论更新之前“导入核验 / 终态待验”项；全域事件、损坏保护受控修复、Windows / 安装器、真实更新 / 重启、配对性能及真实周期仍待验。未改 UI / 未重跑原生，TASK / IMP 保持 in_progress，不回写稳定核心、不关闭或发布。


### 2064085e 扩展配置核验与精确终态

ToggleClient / SetSourceDir / SetSyncMethod 及独立 toggle 入口在合作锁内取得最终配置字节回读证据；路径 / 锁预拒绝无执行终态。ExtensionConfigSave / Save / local 的精确成功解除捕获失败，命令 / HOME / 通道 / 候选轮换保持隔离；仅认证管理器扩展配置，不代表 Skills / MCP 资产成功。

最终 Rust 34 组 / 857 通过 / 0 失败 / 5 忽略（扩展集成 39/39、注册表 33/33），fmt / Clippy / diff 通过。中间失败原件保留，final-* 绑定补丁。注册表 8 triggers / 39 jobs / 80 events（78 active / 2 planned）/ 27 policies / 86 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup，无新计时器 / 路径。未改 UI / 未重跑原生，未单独注入投射后回读破坏。

见 验证/20261011-扩展配置核验与终态 与 .adg/evidence/OCX-0110-20261011-EXTENSION-CONFIG。十个现有资产命令独立终态、损坏保护受控修复、Windows / 安装器、真实更新 / 重启、配对性能和真实周期继续待验；generic Migration / Backup / Sync 保持 0.2.0 planned。TASK / IMP 保持 in_progress，不回写稳定核心、不关闭或发布。


### a392afd9 MCP 配置写入安全终态

MCP 配置发现、读取和写入统一使用有界、拒绝符号链接与非普通文件的安全读取；16 MiB 以上配置、读取异常和损坏内容显式失败，只有明确不存在才按无配置处理。`WriteMcp` 的准备读取结果贯穿合并、备份和原子写入，避免事务内重复读取不同文件版本。

`WriteMcp`、`RemoveMcp`、`AddMcp`、`EditMcp` 对所有目标按稳定路径顺序加锁，锁覆盖源读取、合并、备份、原子写和写后核验；原子写完成后重新读取同一目标，只有字节完全一致才报告成功。目标 home、Skills / MCP / lock 路径、重复物理目标和越界路径均在执行前拒绝。

代码提交为 `a392afd9b6a81ea6d82eb5ba591600466765e0f4`，位于 `feature/0.1.10-maintenance`，已推送。提交未改变 0.2.0 planned 的 generic Migration / Backup / Sync，也没有宣称 Skills / MCP 客户端资产投射、Windows 原生、安装器、实际更新 / 重启或性能门禁已通过。

本地 `cargo test --workspace --features integration-test` 为 690 passed / 0 failed / 4 ignored；`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings` 与 `git diff --check` 通过。CI 的 frontend、backend、release-tools 通过；Windows backend regression 在测试进程启动前再次以 `STATUS_ENTRYPOINT_NOT_FOUND`（`0xc0000139`）失败，Windows release build 与 final EXE smoke 跳过。证据见 [MCP 配置终态证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-MCP配置终态/README.md)。

Windows CI 运行 `38077500332` 的 attempt 2 仍在测试执行前出现 `STATUS_ENTRYPOINT_NOT_FOUND`（`0xc0000139`），不能作为 Windows 回归通过证据；Windows 门禁保持未闭合。TASK-182 / IMP-20 继续 `in_progress`，不颁发完成证书，不回写稳定核心，不发布。

### 0b402c15 Windows 构建与候选安装门禁回写（2026-10-11）

代码分支 feature/0.1.10-maintenance 的提交 0b402c154be65760092b6817b3027a2d0a1dc6dc 修复 Windows 构建时 Tauri 默认 manifest 与项目 Common Controls v6 manifest 重复链接导致的资源构建失败，使用 WindowsAttributes::new_without_app_manifest() 保留项目 manifest。

普通 CI 38083572209 已完成并成功：frontend、release-tools、backend、backend-windows、Windows backend regression、Windows release build、Windows final EXE smoke 和 Windows 启动证据上传通过；build job 为条件跳过，不是失败。候选打包 CI 38083572237 已完成并成功，环境为 win25-vs2026 / 20260925.250.1，Rust 1.99.0、Cargo 1.99.0、Node v24.21.0、npm 11.19.0、Tauri CLI 2.11.4。

MSI 与 NSIS 均完成隔离安装、首屏和卸载：MSI OpenCodeX Desktop_0.1.9_x64_en-US.msi，SHA256 b7c20061e97b47156c9c5a4693786843f01bb9b949d0825eea2c23c6d3599116；NSIS OpenCodeX Desktop_0.1.9_x64-setup.exe，SHA256 9711d86321ac77d8723d1dfcae8d3e9c0a282debf6da20c738a953de89445438。两者卸载后 EXE 均已移除。候选 artifact 为 11682205024，候选证据为 11681514176，Windows 启动证据为 11682320162；ZIP SHA256 见 .adg/evidence/OCX-0110-20261011-WINDOWS-BUILD-INSTALL/artifact-sha256.json。

启动证据通过管理员权限路径：观察 30 秒，主窗口存在且响应，source commit 匹配，CDP HTTP 200，WebView2 153.0.4234.48，页面为 http://tauri.localhost/#overview。该证据不替代 Windows 普通权限人工复验、DPI / 外观验收。

本轮候选实际版本仍为 0.1.9，不能作为 0.1.10 发布资产；codeSigned 为 false，releaseAccepted 为 false，未产生发布授权。macOS 原生安装、真实更新 / 重启、双平台自动检查性能、stable 24h / beta 6h 周期、Windows 普通权限与安装路径 / 自定义路径 / DPI / 外观、真实用户迁移、签名发布仍未闭合。TASK / IMP 保持 in_progress，不回写稳定核心，不合并 main，不发布。


### 8f5a76f8 统一树表控件垂直对齐与本地回归

代码分支 `feature/0.1.10-maintenance` 的 `8f5a76f867272730ee22ba672934fdd82807d218` 已推送。共享生产树表组件统一复选框、箭头占位、图标与文字的 22px 对齐基线，移除原生复选框外边距补偿；未改变业务选择、展开、备份范围或树表数据语义。所有生产树表继续使用共享 `UiTreeTable.vue` / `treeTable.ts` 布局合同。

本地 Rust 全量验证为 690 passed / 0 failed / 4 ignored；UI 为 94 个测试文件、459 个测试通过；类型检查和生产构建通过（170 modules transformed）。Vite 既有 `INEFFECTIVE_DYNAMIC_IMPORT` 警告与测试中的 jsdom canvas notice 均未阻断结果。证据见 `.adg/evidence/OCX-0110-20261011-LOCAL-VALIDATION/` 及协作包验证记录。

该结果只证明源码级回归和共享组件对齐修复，不替代双平台原生、Windows 普通权限 / 外观 / DPI、macOS 安装、真实更新 / 重启、自动检查性能、真实周期、签名、发布资产、更新端点和全域事件覆盖审阅。IMP / TASK 保持 `in_progress`，不构成完成证书或发布授权。

### 24ed093a 更新调度触发与周期合同

`feature/0.1.10-maintenance` 的 `24ed093aea0758b5fdf542fa38fc98a0c1d6b1e9` 已推送。本轮将概览路由进入明确归类为 `foreground` 唤醒；持久化调度器到期仍使用 `deadline`，避免把路由触发和调度到期混成同一来源。旧偏好字段 `app_update_check_interval_seconds` 仅用于旧文档迁移与序列化往返，运行时不读取它，周期只有 native `Target::interval()` 一个事实源。

周期合同固定为：manager stable 24 小时、panel 24 小时、manager beta 6 小时；本轮新增 Rust 5 项调度定向测试全部通过。UI 定向回归覆盖 update scheduler、查询 / 安装所有权、更新中心、树表与备份文件等 6 个测试文件，共 34 项通过；`npm run typecheck`、`npm run build`、Rust fmt、Clippy `-D warnings` 与 `git diff --check` 均通过。

本轮只闭合调度来源与周期的源码级合同，不改变更新安装、重启、原生通知、双平台性能、真实 stable 24 小时 / beta 6 小时观察或发布门禁。macOS 原生、Windows 普通权限 / 外观 / 安装路径、真实更新 / 重启、签名制品、全域事件调用点和性能证据仍未闭合；IMP / TASK 保持 `in_progress`，不形成完成证书、合并授权或发布授权。

### 9497b4e5 0.1.10 版本身份固化与源码门禁收口（2026-10-11）

代码分支 `feature/0.1.10-maintenance` 的提交 `9497b4e56f01b92abf977c6b244ec8e087ffa6b5` 已推送。本提交把 Tauri 配置、Cargo 包 / 锁文件和 UI 包 / 锁文件统一为 `0.1.10`；测试夹具中的历史 0.1.9 输入保持不变。此前构建出的 Windows 候选仍是 0.1.9，不能复用为 0.1.10，必须基于新提交重打包并重新执行候选验收。

源码门禁均通过：`cargo metadata --locked --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace --features integration-test`、`git diff --check`。全量 Rust 命令退出码为 0，库目标 691 项通过 / 0 失败 / 4 忽略，二进制目标 2 项通过，集成测试目标均通过；UI 全量为 94 个测试文件、459 项通过，类型检查和生产构建通过（170 modules transformed）；Windows bundle identity smoke 5/5 通过。

该提交只闭合版本身份与源码级回归，不替代 macOS 原生 UI / 安装、Windows 普通权限与安装路径 / DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、真实更新 / 重启、自动检查性能、stable 24 小时 / beta 6 小时、签名制品、更新端点、全域事件调用点和真实用户迁移。`CHANGELOG.md` 仍只能在 `main` 集成阶段补写；所有 required checks 通过前保持 `in_progress`，不颁发完成证书、不合并 main、不发布或晋升 stable。证据见协作包验证/20261011-目标执行检查点及 `.adg/evidence/OCX-0110-20261011-TARGET-EXECUTION/`。

### c654287c：运行态观测异常隔离与精确解除

代码分支 feature/0.1.10-maintenance 的提交 c654287c06da3e621091c1051ebc19ce1c0b1077 将运行态观测异常与启停执行异常分开处理，并补齐注册表定义、注册校验和发出点。runtime-observation-recovered 仅能在可执行文件、工作目录、OPENCODEX_HOME、对象、动作、阶段、通道、候选身份均与捕获异常一致时解除同一观测异常；允许解除的异常为 runtime-starting-failed、run-unreachable、run-at-risk、external-takeover。它不能解除 start / stop / restart 的 execution 生命周期失败。

定向注册表命令：

    cargo test --locked --manifest-path apps/desktop/tauri/Cargo.toml --features integration-test --test event_registry -- --nocapture

结果为 33 passed、0 failed。最新生产注册表统计为 8 triggers / 39 jobs / 81 events / 27 notification policies / 87 emission sites / 8 scheduling sites / 8 UI feedback sites / 12 paths / 4 cleanup policies。该结果属于局部源码证据，不替代全域调用点审阅、macOS / Windows 原生 UI 与安装验收、性能与真实周期、签名制品、更新端点、真实迁移和发布门禁；IMP 继续保持 in_progress。


### 2026-10-11 树表与事件注册审阅检查点

当前代码基线为 feature/0.1.10-maintenance / c654287c06da3e621091c1051ebc19ce1c0b1077。静态盘点确认生产层级树表统一使用 UiTreeTable.vue / treeTable.ts；SettingsRoute.vue 的两处 data-root-table 仅为数据根路径平面清单。共享合同固定复选框 16px、箭头或占位 22px、图标 18px、文字行高 22px 和居中对齐，备份树初始展开深度为 2。

当前事件注册表统计为 8 triggers / 39 jobs / 81 events（79 active / 2 planned）/ 27 notification policies / 87 emission sites / 8 scheduling sites / 8 UI feedback sites / 12 paths / 4 cleanup policies。生产 Tauri 发出边界集中到 event_delivery.rs；四项 process signal 明确为 internal_only，不属于用户通知遗漏。event_registry 定向测试 33/33 通过；本地 Rust / UI 全量源码证据继续有效。

本检查点只完成静态范围审阅和定向测试，不生成完成证书。macOS / Windows 原生、安装、DPI / 外观、自动检查性能、真实更新 / 重启、签名、发布资产、真实迁移和发布门禁继续 open；TASK / IMP 保持 in_progress，不合并 main、不迁移真实用户数据、不替换日常安装、不公开发布、不晋升 stable。证据见 .adg/evidence/OCX-0110-20261011-TREE-EVENT-AUDIT 与协作包验证/20261011-树表与事件注册审阅/README.md。


### 2026-10-11 c654287c 最新源码全量复核

代码分支 `feature/0.1.10-maintenance` 的最新提交 `c654287c06da3e621091c1051ebc19ce1c0b1077` 已完成一次独立源码门禁复核。`cargo metadata --locked --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked --all-targets --all-features -- -D warnings` 均退出 0；`cargo test --locked` 为 34 个测试组共 `861 passed / 0 failed / 5 ignored`，其中 library `694 passed / 0 failed / 4 ignored`、binary `2 passed / 0 failed`。UI `npm test -- --run` 为 94 个测试文件、459 项通过，`npm run typecheck` 与生产构建通过（170 modules transformed）；Windows bundle identity smoke 为 5/5 通过。

本轮复核同时绑定运行态观测异常隔离：`runtime-observation-recovered` 只有在可执行文件、工作目录、`OPENCODEX_HOME`、对象、动作、阶段、通道和候选身份全部匹配时，才能解除对应观测异常；不能解除 start / stop / restart 的 execution 生命周期失败。当前注册表快照为 8 triggers / 39 jobs / 81 events / 27 notification policies / 87 emission sites / 8 scheduling sites / 8 UI feedback sites / 12 paths / 4 cleanup policies。

证据见 [2026-10-11 c654287c 源码复核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-c654源码复核/README.md) 与 `.adg/evidence/OCX-0110-20261011-C654-SOURCE-REVALIDATION/`。本轮只闭合源码级证据；macOS / Windows 原生与安装、真实更新 / 重启、自动检查性能、stable / beta 真实周期、签名制品、更新端点、真实用户迁移和发布门禁继续 open。`IMP` / TASK 保持 `in_progress`，不构成完成证书、合并 main、公开发布或 stable 晋升授权。

### 2026-10-11 main 集成 CI 门禁回写

代码分支 feature/0.1.10-maintenance 的实现已集成到 main，集成提交为 3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea，origin/main 已同步；CHANGELOG.md 已在 main 写入 0.1.10 未发布候选说明。GitHub Actions 运行 38096673516（https://github.com/gzers/opencodex-desktop/actions/runs/38096673516）已完成并成功，五个顶层 job 全部通过：release-tools、frontend、backend、backend-windows、build。其中 Windows backend regression、Windows release build、Windows final EXE smoke 与 artifact upload 通过，macOS arm64 bundle build 通过。

集成前后本地门禁保持通过：npm ci、UI 类型检查、生产构建、94 个 UI 测试文件 / 459 项测试、Cargo metadata、fmt、Clippy -D warnings、Rust 测试（library 694 passed / 0 failed / 4 ignored，binary 2 passed）和 git diff --check。完整回执见 .adg/evidence/OCX-0110-20261011-CI-38096673516/。

本次只闭合 main 集成源码、CI 构建和 smoke 门禁。CI 成功不能替代 macOS 原生 UI / 安装 / 更新重启、Windows 普通与管理员双权限 / 安装路径 / DPI / 云母 / 标题栏 / 动效 / 透明下拉框、配置与备份数据根、自动检查性能与 stable 24 小时 / beta 6 小时周期、真实端点更新、签名、公证 / Authenticode、用户迁移和发布后观察。状态继续为 in_progress，release_authorized: false；不创建 v0.1.10 标签，不公开发布，不替换日常 0.1.7。
