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

> **2026-10-18 当前执行结论：本轮 Windows 本机执行收口，停止新采样和采集工具扩展。** 已通过或已接受的结果复用，未测项保持未验；TASK-182 和版本验收未关闭。后文“下一前台”“继续补采”等历史计划不再自动执行。当前决策见文末“Windows 本轮执行收尾”。

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
| 真实原生 UI | macOS Apple Silicon / Windows x64；概览与设置共用更新、树表对齐 / 键盘、弹窗滚动、主题 / 三档、光层全边距、CSS / WebGL / 静态回退 | Windows 普通 / 管理员分别测，保留当前屏幕的正常 UI 验证；按 2026-10-16 用户裁决，取消本版本专项 DPI 矩阵，不要求额外物理 1440p / 200% 屏幕或 RDP 高 DPI 会话 |
| 生命周期 | 最小化 / 隐藏 / 挂起 / 网络恢复 / 退出 / 重启，托管面板与外部 HOME，通知跨启动去重、晚到回调、进度不丢失 | 0.1.9 原生标题栏、菜单迁移、关闭 / 托盘 / Snap 等相关回归保留；既有未覆盖项按适用性登记，不自动豁免 |
| 数据与失败 | 旧格式 / 自定义根 / 不可写 / 空间不足 / 冲突 / 损坏 / 非法路径 / 缓存清除 / 时钟改变、备份失败阻断、恢复中断与回滚、关联记录清理 | 用隔离可复现数据，验证文件内容和绑定，不只截图 toast；真实用户迁移单独确认 |
| 安装与升级 | macOS 当前候选安装 / 卸载；Windows MSI / NSIS 新装、0.1.9 升级、自定义位置与受保护路径、卸载保留 / 明确清理、运行版本首屏 | 精确制品 SHA 与版本；按 2026-10-16 用户授权直接使用本机 Windows，可卸载 OpenCodex / 桌面管理器并清理其配置，保留与明确清理分别用专用夹具核验；其它设备及已承诺旧版 OTA 兼容路径（含专用设备 0.1.7）保持原边界 |
| 性能 / 长时 | 第 5 节配对测量、缓存命中零请求、离线 / 慢网子进程收尾、真实周期到期与休眠恢复 | 单测、原型、真实周期与硬件测量四类证据分开；未测标待验 |
| 发布 | main 集成 CI、双平台 Release 构建、三种更新资产旧信任公钥密码学验签、公开端点与 Release SHA、真实下载 / OTA | updater 签名不等于 Apple 公证 / Authenticode，平台分发状态分别明确 |

阶段任务使用稳定检查名，例如 `review:scope-0110`、`review:storage-0110`、`review:scheduler-performance-0110`、`review:updates-0110`、`review:backup-tree-0110`、`review:event-coverage-0110`、`review:native-0110`、`review:installer-0110`、`review:core-writeback-0110`、`review:release-0110`。执行时通过治理 provider 建 TASK，required_checks 对应 RCP.checks 与真实证据；本次不伪造未开始阶段的通过回执或关闭证书。

每份回执带源码 SHA、制品 SHA / 版本、设备 / OS / 权限 / 实际缩放、隔离数据标识、命令 / 操作、结果、截图或日志及未覆盖原因；失败原件保留，修复后新增回执。大证据与摘要分开，任何清理先预览。治理任务关闭只证明对应任务完成，不能扩展为版本已发布。

2026-10-16 用户明确裁决：本版本专项 DPI 验收取消，历史“DPI 待验”记录保留为当时事实，此后不再作为未完成阻塞，也不改标为测试通过。本机 Windows 尚未实际使用 OpenCodex，允许真实安装 / 升级 / 卸载及其配置清理，范围限定为核实过的 OpenCodex 与桌面管理器目录；新增账户、虚拟机或 RDP 不作为这些测试的前置条件。安装位置、数据根绑定、卸载保留与明确清理功能仍须验证。其余原门禁、性能预算、周期与发布授权保持。

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


### 2026-10-11 main@3bcd9318 源码门禁回写

代码实现已集成到 main@3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea，且 origin/main 已同步。当前提交将版本身份固定为 0.1.10；Windows bundle identity smoke 5/5 通过。

本轮源码门禁全部通过：Cargo metadata、fmt、Clippy -D warnings、Rust workspace integration tests（library 694 passed / 0 failed / 4 ignored，binary 2 passed / 0 failed）、UI 94 个测试文件 / 459 项通过、类型检查、生产构建（170 modules transformed）和 git diff --check。完整命令及结果见 .adg/evidence/OCX-0110-20261011-MAIN-SOURCE-GATES/。

本证据只闭合 main 集成、源码回归、构建和版本身份检查。旧 feature 分支记录继续作为历史证据；此前 Windows 候选包真实身份仍为 0.1.9，不能作为 0.1.10 发布资产，必须基于当前 main 重新生成明确身份为 0.1.10 的 macOS / Windows 制品。

macOS / Windows 原生 UI、安装与路径、双权限数据根、DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、自动检查性能与稳定 / beta 周期、真实更新 / 重启 / 恢复、签名制品、全域事件调用点、真实用户迁移及发布门禁仍未闭合。当前 release_authorized: false、状态继续 in_progress；不创建标签、不公开发布、不晋升 stable、不替换日常 0.1.7。


### 2026-10-11 Windows 本机 IPC 控制面执行回写

代码提交 861fdeea39d1c45fa2beddb2cba6659b67735d9f 已推送到 main。本提交把 Windows 本机控制面接到 Tokio named pipe；管道身份由 OpenCodeXDesktop 与当前账户稳定摘要组成，并启用拒绝远程客户端与创建者默认 DACL。Unix 保持私有 Unix socket、UID 校验和 socket 权限。两端共用 8 字节大端长度前缀与 JSON frame，ocxd CLI、IPC 服务注册和 Tokio net feature 已跨平台接通。

本地源码门禁：cargo fmt --check、cargo test --lib（694 passed / 0 failed / 4 ignored）、cargo test --bin ocxd（2 passed / 0 failed）、cargo clippy --all-targets --all-features -- -D warnings、git diff --check 均通过。cargo check --target x86_64-pc-windows-msvc --all-targets 因当前 macOS 宿主缺少 Windows MSVC SDK/C 头文件，在 ring 找不到 assert.h 处失败；该结果记为环境阻塞，不能当作 Windows 编译通过。

本轮只闭合 IPC 源码实现和本地回归。Windows 普通权限 / 管理员权限真机 smoke、安装目录和数据根、DPI、云母、原生标题栏、动效、透明下拉框、macOS 原生安装、自动检查性能、真实更新 / 重启、签名制品、全域事件执行证据仍待验。0.1.10 双平台制品必须基于当前 main 重新生成，旧 0.1.9 候选不能复用。证据见协作包验证/20261011-Windows IPC本机控制面/README.md 与 .adg/evidence/OCX-0110-20261011-WINDOWS-IPC-TARGET/。IMP / TASK 继续 in_progress，release_authorized: false。

### 2026-10-11 macOS DMG 隔离安装生命周期观察

基于 main@861fdeea39d1c45fa2beddb2cba6659b67735d9f 的 0.1.10 arm64 候选已完成临时 DMG 隔离安装、启动和首屏读取。候选使用独立 bundle identifier 与 /tmp/ocx-0110-dmg-install-tuk3m7/data，未替换 /Applications 日常安装；窗口关闭后进程继续保活，随后通过应用退出路径退出并重新启动，当前只保留一个隔离测试实例。

当前复核显示测试 PID 86306、data/logs/app.log、data/manager-state/app.lock 和更新调度文件均存在，锁与保留进程匹配；日常安装 PID 75764 保持运行且未操作。历史强制终止后残留旧锁的失败证据继续保留，不被本轮局部成功覆盖。详见运行维护协作包中的 macOS DMG 安装生命周期验证记录与 .adg/evidence/OCX-0110-20261011-MACOS-LIFECYCLE/。

本轮只证明 macOS 临时隔离安装与局部生命周期观察，不证明正式签名 / 公证、原生外观、升级 / 卸载、真实更新 / 重启、Windows 0.1.10 真机、自动检查性能、稳定周期或发布授权。release_authorized: false、release_accepted: false；协作包与 IMP 继续 in_progress。

### 2026-10-11 main@861fdeea 调度与缓存定向复核

基于代码主分支 main@861fdeea39d1c45fa2beddb2cba6659b67735d9f 完成自动检查调度 / 缓存定向复核。Rust 命令 cargo test --locked --manifest-path apps/desktop/tauri/Cargo.toml --features integration-test schedule -- --nocapture 通过 5 项、失败 0 项、忽略 0 项；UI 命令覆盖 update-scheduler、update-query-ownership、update-install-ownership、update-version 四个文件，18 项全部通过，两个命令 exit 0。

该回执只确认固定周期、持久 deadline、缓存边界、通道隔离、single-flight 相关源码合同，以及查询 / 安装所有权定向回归；不把源码单测当作真实 CPU、RSS、网络 / 子进程、启动 p95、帧耗时或 stable 24 小时 / beta 6 小时性能证据。Windows 双权限 / DPI / 原生外观、macOS 原生安装、真实更新 / 重启、签名制品和发布门禁继续未闭合。详细记录见协作包验证/20261011-调度缓存复核及 .adg/evidence/OCX-0110-20261011-SCHEDULE-CACHE-REVALIDATION/。

IMP-OPENCODEX-DESKTOP-20 继续 in_progress，release_authorized: false；不创建标签、不公开发布、不晋升 stable、不替换日常安装。

### 2026-10-11 feature/0.1.10-maintenance@c654287c macOS arm64 候选重打包

在代码分支 `feature/0.1.10-maintenance` 的 `c654287c06da3e621091c1051ebc19ce1c0b1077` 上执行 `cargo tauri build --bundles app,dmg -- --locked`，退出码为 0；生成的应用和 DMG 身份均为 0.1.10，应用可执行文件为 arm64 Mach-O，DMG `hdiutil verify` 通过。当前 `.app` 为 ad hoc、linker-signed，未设置 TeamIdentifier，严格签名校验失败，因此该结果只证明候选构建和容器完整性，不构成正式签名、公证或发布资产。

本轮候选摘要与命令原件见 [.adg/evidence/OCX-0110-20261011-C654-MACOS-CANDIDATE](../../.adg/evidence/OCX-0110-20261011-C654-MACOS-CANDIDATE) 及协作包 [20261011-c654-macOS候选重打包](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-c654-macOS候选重打包/README.md)。该记录与此前 `main@861fdeea` 的 macOS 构建证据并列保存，不能混用制品哈希。

本轮只闭合 feature 分支的候选打包身份和 DMG 容器校验。macOS 正式签名 / 公证、真实安装升级 / 重启回读、Windows 普通与管理员权限及安装路径 / 数据根 / DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、自动检查性能与 stable 24h / beta 6h 周期、真实更新端点、全域事件执行证据和发布授权仍未闭合。`state: in_progress`、`release_authorized: false` 保持不变；不创建标签、不公开发布、不晋升 stable、不替换日常 0.1.7。

### 2026-10-11 c654 隔离原生 UI 局部观察与执行阻塞

当前仅保留 c654 隔离测试 PID `17410`，日常安装 PID `75764` 未操作；历史 PID `86306` 的 SIGTERM 退出不计为正常退出 / 锁清理通过。测试副本修改身份并 ad hoc 重签，不覆盖原候选严格签名失败，也不替代当前 `main@861fdeea` 候选验收。

本轮真实偏好备份大小及 SHA-256 与清单一致；树表初始根 / 年展开、月份收起，展开到文件后刷新能保留层级。焦点及复选框对齐仍待验。设置页可进入共用更新中心及两类详情；管理器当前 0.1.10 无可用版本，面板本地未发现且查询失败，安装按钮禁用；发行说明回退及面板官方网页入口可见，未下载、安装、重启或打开外部链接。概览准备态当前视口顶部与左右未见明显光层留缝，完整外观矩阵仍待验。截图仅在本会话观察、未落盘。详细边界见协作包 [c654 macOS 候选补充记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-c654-macOS候选重打包/README.md) 与 `.adg/evidence/OCX-0110-20261011-C654-NATIVE-UI-PARTIAL/`。

Windows 当前 main 双权限 / 安装路径 / 数据根 / 原生外观 / IPC、macOS 正式签名公证及真实更新重启、自动检查性能及 stable24h / beta6h、全域事件执行和发布授权等门禁已连续多轮未闭合。完成本轮回写后，目标执行设为 `blocked`，等待对应设备、签名与真实更新条件；IMP / TASK 保持未完成的 `in_progress`，不生成完成证书、不提前回写稳定核心事实。恢复时沿用源码与证据，只补受影响的缺口，不重复全套测试或另开多余实例。

### 2026-10-11 Windows IPC CI 失败与修复提交

`main@861fdeea39d1c45fa2beddb2cba6659b67735d9f` 的 [CI 38101892145](https://github.com/gzers/opencodex-desktop/actions/runs/38101892145) 已失败：`backend-windows` 的 Clippy 编译发现 `with_app_handle` 被 `cfg(unix)` 限制而 Windows GUI 注册仍调用该方法，以及 named pipe 接收路径多余的 `mut`。Windows 后端回归、release build、最终 EXE smoke 与 bundle build 均未执行。此前 `3bcd9318` 的成功 CI 不能作为后续 IPC 提交的 Windows 通过证据。

修复提交 `adccca8e92d28c95d4fa83c11d994cb6bf3445f6` 已推送到 `main`：让既有 GUI 共享状态绑定方法跨平台可用，删除不需要的可变绑定，保持 `-D warnings` 门禁。本机格式检查、IPC 定向回归（44 passed / 0 failed / 0 ignored）、workspace all-targets/all-features Clippy 和暂存 diff 检查均通过。新 [CI 38117383632](https://github.com/gzers/opencodex-desktop/actions/runs/38117383632) 与修复提交 SHA 匹配，观察时仍为 `in_progress`，暂不认定 Windows 门禁通过。证据见 `.adg/evidence/OCX-0110-20261011-WINDOWS-IPC-CI-FIX/`。

下一步先确认该提交 Windows CI 和候选构建，再用同一源码身份完成 Windows 双权限 / 安装路径 / 数据根 / DPI / 原生外观 / IPC 真机验收；同时补 macOS 正式签名公证、真实更新进度与重启、自动检查性能及全域事件执行证据。本轮没有启动额外测试软件；沿用现有唯一隔离实例。IMP / TASK 保持 `in_progress`，未迁移真实数据、未替换日常安装、未创建标签或公开发布，`release_authorized: false`。

### 2026-10-11 Windows IPC 释放顺序竞态与同源码候选交接

前述 `main@adccca8e` 的 [CI 38117383632](https://github.com/gzers/opencodex-desktop/actions/runs/38117383632) 已完成并失败：Windows Clippy 通过，后端库为 485 passed / 1 failed / 4 ignored，build 跳过。失败项 `cli_binding_owns_runtime_and_exclusive_storage_until_terminal_audit` 揭示运行锁先释放、未保存绑定的写门稍后解除的竞态；原断言继续保留。该 SHA 的 macOS DMG 构建及容器校验通过，但严格签名失败、宿主没有有效签名身份，不构成正式签名或公证证据。

释放顺序修复 `3d856fb561aa8dd21d0dd1347d8736c7b704f3d7` 已提交推送到 main。绑定守卫移入 worker 局部变量，未保存绑定先解除写门再释放运行操作锁；已保存的待重启绑定继续冻结。本地 fmt、IPC 定向 44 项、Rust workspace integration-test 34 组（861 passed / 0 failed / 6 ignored，其中库 694 passed / 4 ignored）以及 all-targets / all-features Clippy `-D warnings` 全部退出 0；忽略项原样列于回执，不能当作原生、真实网络或钥匙串验收通过。新 [CI 38118226895](https://github.com/gzers/opencodex-desktop/actions/runs/38118226895) 源码身份一致，观察时仍在运行，Windows 尚未认定通过。

本地命令、原失败日志、修复差异和哈希见 `.adg/evidence/OCX-0110-20261011-IPC-BINDING-RELEASE-ORDER/`；旧 macOS 候选见 `.adg/evidence/OCX-0110-20261011-ADCC-MACOS-CANDIDATE/`。后续按同一修复 SHA 收齐 Windows 与 macOS 候选，不混用旧制品。设备验收、物理 DPI 与双权限、安装目录数据根、自动检查性能预算、真实更新和发布边界统一见 [跨平台候选与真机交接](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-跨平台候选与真机交接/README.md)。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`；本轮未启动额外软件。

### 2026-10-11 main@3d856fb5 macOS 候选与下一步门禁

按修复后的 `main@3d856fb561aa8dd21d0dd1347d8736c7b704f3d7` 完成 macOS arm64 候选构建，命令退出 0，耗时 116.398 秒。应用版本 / 构建号均为 0.1.10，EXE 内嵌完整源码 SHA；DMG 为 4,383,683 bytes，SHA256 `bbe9ef00f9ea5a46e69c2ae09b10b611b9757f18a0b8b4e080680696708eede4`，容器校验通过。原 EXE 为 7,897,760 bytes，SHA256 `af154f85fd7116f681b951c331bd1b99ad49f589ed8da1107e68d2179cd550c7`；完整原 `.app` 和 DMG 单独留存，不被后续构建或测试重签覆盖。

严格签名 exit 1（资源未封装），宿主 0 个有效签名身份，公证未验证。该候选未安装 / 启动；旧 c654 测试实例的退出快捷键尝试未结束进程，本轮没有另开测试实例，因此正常退出 / 锁清理及最新 SHA 原生验收仍待验。摘要与原件见 `.adg/evidence/OCX-0110-20261011-3D856-MACOS-CANDIDATE/`。

执行顺序为：新 CI 全部通过后生成同 SHA Windows 候选；双平台隔离安装和设备验收；0.1.9 配对性能、缓存命中零请求与真实周期；真实更新 / 重启及事件生命周期；正式签名和发布资产审阅。各步完成条件已写入跨平台交接记录。最新运行快照中 frontend / release-tools / backend 已通过，Windows Clippy 和后端回归已通过且正在 release 构建；完整 CI 仍为 `in_progress`，快照见 `.adg/evidence/OCX-0110-20261011-CI-38118226895/`。IMP / TASK 保持 `in_progress`，`release_authorized: false`、`release_accepted: false`；正式发布和真实用户迁移仍需单独确认。

### 2026-10-11 main@3d856fb5 Windows CI 与最终 EXE 启动通过

06:53 UTC 新快照中 [CI 38118226895](https://github.com/gzers/opencodex-desktop/actions/runs/38118226895) 的 frontend / release-tools / backend / backend-windows 四个 job 已通过，macOS build 仍运行。Windows 严格 Clippy、后端回归、release EXE 构建和最终 EXE smoke 均成功。启动 EXE SHA256 为 `ab023fa342b360d39c0298c85b14a844d716877d44ea8f0109369bcfda73902b`，完整源码 SHA 与 `3d856fb561aa8dd21d0dd1347d8736c7b704f3d7` 匹配；30 秒观察窗口存在且响应，WebView 主 UI 挂载、关于信息 0.1.10 / Windows、隔离数据根初始化均通过。

artifact ID `11694523976` 的关键原件、窗口截图、环境、原始字节归档、摘要和新运行快照见 `.adg/evidence/OCX-0110-20261011-3D856-WINDOWS-CI-SMOKE/`。历史 CI 快照与失败证据保留。仍需完整 CI 成功后仅触发一次同 SHA Windows 候选，随后验 MSI / NSIS 安装身份、真机双权限 / 安装路径数据根 / 物理 DPI / 原生外观 / IPC，以及 macOS 安装 / 正常退出 / 树表焦点对齐和既定性能、真实更新、事件及签名门禁。CI 隔离启动不替代这些验收。

本轮未改代码、未重跑已通过回归、未启动额外测试实例。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`；未生成完成证书或提前晋升稳定事实。

### 2026-10-11 main@3d856fb5 完整 CI 通过与 Windows 候选触发

06:56 UTC 核查 [CI 38118226895](https://github.com/gzers/opencodex-desktop/actions/runs/38118226895) 已完成，frontend / release-tools / backend / backend-windows / build 五个 job 均成功，headSha 与 main 完整 SHA 一致。核实该 SHA 没有已触发的 Windows 候选后，仅 dispatch 一次 main 的 `windows-candidate.yml`；[候选 run 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 的 headSha 再次核对一致，观察时为 `in_progress`。

最终 CI 快照、候选快照、触发命令及哈希清单见 `.adg/evidence/OCX-0110-20261011-3D856-CI-AND-CANDIDATE-DISPATCH/`。下一步先核候选 MSI / NSIS 和安装后 EXE 身份，再补同 SHA 双平台真机、自动检查性能及真实周期、真实更新和事件生命周期、签名门禁。候选尚未认定通过，IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`；没有触发公开发布、标签或 stable 晋升，也未增加前台测试实例。

### 2026-10-11 main@03e0ad94 macOS 应用菜单与退出重启观察

已提交推送 `03e0ad942c16fe2ef75473eb6693012b3ae684ac`：恢复 macOS 首个应用菜单中的原生隐藏、隐藏其他、显示全部和退出。fmt、tray 单元 3 项、tray integration 4 项、window_close integration 6 项及 all-targets / all-features Clippy `-D warnings` 全部通过，定向共 13 项，Windows 菜单处理保持原有逻辑。

同 SHA macOS arm64 候选构建退出 0、耗时 121.254 秒，版本 / 构建号均为 0.1.10，EXE 内嵌完整源码 SHA。DMG 为 4,384,633 bytes，SHA256 `1bc69e1b42676dfff93c4c402ffe8bb5affd234ade924ed5f4b33600a6fa3ab4`，容器校验通过；原 EXE 为 7,897,776 bytes，SHA256 `196596e388f4f57d63c66c1067d70ac1be1879ec2a9096f657efbe2dc4590c0f`。原件严格签名仍 exit 1（资源未封装），正式签名与公证未通过；原 `.app` 和 DMG 单独留存，只有隔离测试副本修改 identifier 并 ad hoc 重签，不覆盖原件失败。

从只读挂载的 DMG 核对 EXE 后复制测试副本，首次启动 PID41949 使用新沙箱根 `test-data`。真实应用菜单含原生退出；点击后原 PID 消失、该根锁可非阻塞取得。随后 UI 状态读取期间出现 PID42076，沿用了旧测试根；保留初读与后续精确 PID / 锁观察，重开来源未独立定位，不记为同新根重启或完整生命周期通过。对 PID42076 按 ⌘Q 后，无信号 monitor 确认原 PID 退出、两根锁均释放，随后 CLI 明确使用新根重启为 PID42681，未删除锁文件，环境与锁 PID 一致。只保留 PID42681 一个测试实例，日常 PID75764 未操作。旧 PID17410 的 SIGTERM 清理不算正常退出。

原生菜单 AX、同根重启截图、构建与挂载原件、PID / 锁观察和哈希见 `.adg/evidence/OCX-0110-20261011-03E0-MACOS-APPLICATION-MENU/`。菜单截图不可用，⌘Q 后 AX 的 procNotFound / -600 仅有上下文记录，完整原始异常未留存；退出由独立进程 / 锁证据佐证。2026-10-11 07:13 UTC，[03e0 CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 的 frontend / release-tools / backend 通过，Windows 后端回归仍运行；旧 3d856 [候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 仍在 NSIS / MSI 构建，不作为 03e0 候选通过证据。

下一步按最新 SHA 收齐完整 CI 和安装候选，再完成真机、性能、真实更新 / 事件及正式签名验收。隔离复制安装与局部退出观察不替代真实安装升级、完整原生或发布门禁。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`；本轮未迁移真实数据、替换日常安装、创建标签或公开发布。

### 2026-10-11 main@03e0ad94 备份树表、更新详情与候选状态收口

继续使用唯一测试 PID42681 和既有隔离根，未另开实例。真实生成管理器偏好备份 `bk_20261011072259_f9e7a1ac`，目录仅含 preferences.json 与 backup-manifest.json；载荷 1371 bytes，独立重算 SHA256 与清单一致。根 / 年默认展开、月份收起；实际箭头逐层展开后可见两个真实文件，键盘焦点移至清单文件，刷新保留展开层级。当前 macOS 无复选框场景的箭头 / 文字 / 叶占位视觉对齐；不代替复选框、半选、禁用或 DPI 矩阵。点击刷新后的焦点改变，未独立验证行被移除时的焦点恢复，因此该项不记通过。

设置可进入共享更新中心及两类详情。管理器当前 0.1.10、可用版本为 —，显示已是最新、缺说明且安装禁用；面板当前未知、可用 2.83.0，版本查询失败，展示缺说明回退与官方网页入口，安装禁用。沙箱未发现 OpenCodex，不能代替概览就绪态入口；未点击网页或安装。长篇说明滚动、真实说明来源 / 安装进度、备份失败阻断和重启回读仍待验，未根据禁用状态认定真实更新门禁通过。

07:32 UTC 核查最新 [CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821)：headSha 与 03e0 一致，frontend / release-tools / backend / backend-windows 四项成功，macOS build 仍运行，整体尚未通过。旧 3d856 的 [Windows 候选 38119648935](https://github.com/gzers/opencodex-desktop/actions/runs/38119648935) 已 completed / success，构建、隔离 runner 安装验证与上传成功；尚未下载独立核验安装器，不能作为 03e0 候选通过证据。本轮未重复 dispatch。

15 组原生 AX / JPEG / metadata、偏好备份只读完整性报告、CI 原始 JSON 与哈希见 `.adg/evidence/OCX-0110-20261011-03E0-NATIVE-BACKUP-DETAILS/`。其中 04 文件名含 expanded 但画面未展开，08 首次焦点尝试未证明成功，原样保留并说明。后续按交接顺序完成同 SHA Windows 候选、双平台设备验收、性能 / 真实周期、真实更新 / 事件与正式签名审阅。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`，本轮未迁移真实数据、替换日常安装或公开发布，0.2.0 原型及资料变更未夹带。

### 2026-10-11 最新完整 CI 通过与同 SHA Windows 候选启动

[CI 38120149821](https://github.com/gzers/opencodex-desktop/actions/runs/38120149821) 已 completed / success，五个 job（frontend、release-tools、backend、backend-windows、macOS build）全部成功；headSha 为 `03e0ad942c16fe2ef75473eb6693012b3ae684ac`。此前 07:32 UTC 的运行中快照保留，以上为后续状态。核对远端 main 及同 SHA 候选查重后，2026-10-11 15:38（UTC+8）仅触发一次 `windows-candidate.yml`，新 [Windows 候选 38122059291](https://github.com/gzers/opencodex-desktop/actions/runs/38122059291) 已 in_progress，headSha 与最新 CI 一致；不能在完成、下载、核验前写作候选通过。

原始 CI JSON、触发前后列表、远端引用、dispatch stdout / stderr、命令时间与哈希见 `.adg/evidence/OCX-0110-20261011-03E0-CI-CANDIDATE-DISPATCH/`。下一步收齐该 run 的安装器及 runner 证据，独立核验资产身份后交 Windows 隔离设备完成双权限 / 物理 DPI / 默认数据根 / 原生验收；macOS、性能、真实周期、真实更新与正式签名门禁继续保留。未创建版本标签、Release 或 stable 晋升；唯一测试 PID42681、日常 PID75764 保持原状。

### 2026-10-11 同 SHA Windows 安装候选核验与 macOS 同根生命周期

最新 `main@03e0ad942c16fe2ef75473eb6693012b3ae684ac` 的 [Windows 候选 38122059291](https://github.com/gzers/opencodex-desktop/actions/runs/38122059291) 已 completed / success，package job 于 15:49:19（UTC+8）完成。两个 artifact ZIP 的本地摘要与 GitHub digest 一致；MSI / NSIS 本地重算大小与 SHA256，和 ZIP 条目、runner 回执相符。两份 artifact 的 installer-verification.json 字节一致。临时 runner 报告安装 / 最终 EXE 首屏 / 卸载均通过；最终 EXE 的预期 bundle marker 与安装后哈希相符，但安装后 EXE 只由 runner 回执证明，本机没有独立提取重算。安装器、ZIP、最终 EXE 摘要及环境原件见 `.adg/evidence/OCX-0110-20261011-03E0-WINDOWS-INSTALLER-CANDIDATE/`。

本轮发现并限定两项证据：main-window.png 来自 CDP WebView 捕获，图中左侧透明区域黑底、文字对比度低；需要真机整窗 / CDP 对照，根因未确认，不判云母 / 原生外观通过。startup-diagnostics 权限字段读取 PowerShell harness 当前 token，不是目标应用进程 token；不能代替普通 / 管理员应用双权限验收。runner 采用沙箱根、结束时强杀测试进程树，因此不替代新装默认 `<安装目录>\data`、正常退出 / 锁清理或物理 DPI。候选仍 codeSigned / releaseAccepted false。

macOS 继续使用原隔离副本，15:44:41（UTC+8）对 PID42681 按原生 ⌘Q；随后独立进程观察确认退出，非阻塞锁可取得，锁文件未删除或截断。15:45:03 显式以同一 test-data 根重启为 PID75489，环境、内嵌源码身份与锁 metadata 一致，只有一个测试实例。退出前 / 退出后 / 重启后 / 最终四次快照中，原备份 `bk_20261011072259_f9e7a1ac` 的 preferences.json（1371 bytes）与 backup-manifest.json 摘要完全一致；原生设置回读原根，月份默认收起，原备份记录可见。日常 PID75764 与启动时间不变。原始 AX / 截图、完整 procNotFound / -600 异常、只读锁探针方法及回执见 `.adg/evidence/OCX-0110-20261011-03E0-MACOS-SAME-ROOT-LIFECYCLE/`；no-change diff、误点和裁切画面均保留说明。该项只闭合隔离同根生命周期，不代替原件正式签名、默认数据根或真实安装升级。

当前同 SHA CI / Windows 候选资产已收齐，后续进入 Windows 实际目标进程双权限、物理 DPI、默认根及原生外观验收；并继续树表、配对性能 / 真实周期、真实更新 / 事件与正式签名门禁。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`。本轮只回写文档与证据，没有代码变更；未迁移真实数据、替换日常安装、创建标签、公开发布或晋升 stable。

### 2026-10-11 main@369ab12d 树表对齐、焦点修复及同源码候选

源码 `main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27` 已推送。统一 UiTreeTable 采用 16×22px 选择槽、16px 选择框，箭头 / 图标 / 叶占位与文字按 22px 首行对齐；多行名称不再使控件居中于整段。异步卸载前保存焦点及可见祖先，回退至可用选择框、同行主控件、可见祖先、首行；外部焦点不抢夺，错误移至重试入口，过期回调受保护。复选框方向键接入树导航，Space 保持原生选择。

扩展用例对父提交生产组件的基线为 8 passed / 6 failed，失败原件保留；修复后树表 15 项、全 UI 94 文件 / 466 项通过，typecheck / 构建 / diff check 通过。生产浏览器夹具浅深色、300 / 420px 名称列的首行中心偏差 11→0 CSS px；混合态选择框刷新保焦与 ArrowDown 实测通过，console 无错误。临时 QA 标签和 Vite PID4947 已清理；证据见 .adg/evidence/OCX-0110-20261011-369A-TREE-ALIGNMENT-FOCUS/，不代替完整原生 / 物理 DPI。

同源码 [CI 38124355301](https://github.com/gzers/opencodex-desktop/actions/runs/38124355301) completed / success，release-tools、frontend、backend、backend-windows、macOS build 五项成功。核对远端 main、最近 50 个候选查重后，于 17:04（UTC+8）只触发一次 [Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287)，headSha 一致。后续快照锁定依赖回归成功，NSIS / MSI 构建中，安装 smoke / 上传 / 本地核验待完成；原件见 .adg/evidence/OCX-0110-20261011-369A-CI-CANDIDATE-DISPATCH/。03e0 已通过项保留历史，不自动继承为最新 SHA 通过。

macOS 同源码 app / dmg 构建 exit 0，用时 121.016s；arm64、版本 / 构建号 0.1.10，完整 SHA 内嵌核验通过。DMG 4384841 bytes，SHA256 `7ccb4927be704711997674ad58dba8342c80a70fcf0b62de080440cea351f743`，hdiutil verify exit 0；原 EXE 7897776 bytes，SHA256 `3e670b4e2e7517e4d10897919b56769030bde3eac944d4642fe5c0e5ded9f52e`。原件保存于 /private/tmp/ocx-0110-369a-candidate-tgpmm_zq/original/；严格 codesign exit 1，资源未封装、只有 linker ad hoc，宿主 0 个有效签名身份，公证未验证。测试副本独立 identifier sandbox0110369a，ad hoc 严格校验通过，EXE SHA256 `8bb3920180d03c54e78392df71d8e0e57fefda0ddb023d60f2879625e71f4018`；不能代表原件正式签名。

旧测试 PID75489 原生 ⌘Q 退出、锁非阻塞可取得且字节不变；只读挂载 DMG 复制安装后，最新 PID9422 为唯一测试实例，test-data 根、环境与锁 metadata 匹配。日常 PID75764 与启动时间未变。真实偏好备份 `bk_20261011082942_7318022d` 仅含 preferences.json / backup-manifest.json，载荷 1371 bytes，独立摘要与清单一致；不含完整面板配置，未恢复。

原生根 / 年默认展开、月份收起；建立焦点后方向键到清单、Left 返回父箭头、刷新保留展开层级和浅深色局部可读已有证据。无复选框场景；刷新焦点位于 HTML 内容，不能记原生异步行焦点恢复通过。00—10 的 JPEG / AX / metadata 保留误点、无焦点按键和裁切尝试；08 只证明展开保留，09 树不在截图范围、10 滚动后才可见。见 .adg/evidence/OCX-0110-20261011-369A-MACOS-TREE-CANDIDATE/；原日志 gzip 按字节保存并记录摘要，各目录 checksums.sha256 独立校验。

继续完成最新 Windows 候选资产核验、实际目标 token 双权限 / 物理 DPI / 默认数据根 / 原生外观，以及 macOS 完整树表密度 / 画质矩阵、最新 SHA 生命周期 / 真实安装升级、同机 0.1.9 配对性能 / 真实周期、真实更新 / 事件和正式签名。性能门禁不调整；核心事实推广仍待后续确认。IMP / TASK 保持 `in_progress`、`release_authorized: false`、`release_accepted: false`。本轮文档提交只包括 0.1.10 证据与协作包，未夹带 0.2.0 资料、迁移真实数据、替换日常安装或公开发布。

### 2026-10-11 main@369ab12d Windows 安装候选最终核验

最新 [Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287) 于 17:17:48（UTC+8）completed / success，完整源码身份为 369ab12da55448fbfdc4d0c9bf03970b56fd6c27；该 SHA 只触发一次。锁定依赖回归、MSI / NSIS 构建、隔离 runner 安装 / 最终 EXE 首屏 / 卸载及上传成功。17:28（UTC+8）本地独立重算两个 ZIP 的大小、SHA256，与 GitHub API digest 一致，ZIP CRC 通过；候选 ZIP 只有两个安装器和回执。两份 artifact 的 installer-verification.json 字节一致（1970 bytes，SHA256 3144d002a110caca807595c2dd9ec7cb6c50e68de4d0d84f1e820a61b1ff4857）。

| 资产 | bytes | 独立 SHA256 |
|---|---:|---|
| 候选 ZIP，artifact 11696829726 | 7561927 | 77d4ca63098b18a93589761edbe639c6808fd1dc253e61d805c4b5e6513cc002 |
| 证据 ZIP，artifact 11696909705 | 1685851 | bae9aa0b6b1264d2c9df9d5f8fb1cec032ffc9a52f3b56559418a75337f68a5b |
| MSI | 4538368 | f149f01f3096fc7aafa8bc574358ab11fd6c77479db28d822ee414a57fed2fc3 |
| NSIS | 3231540 | c5a3b9dc9ed87e486933a64ff4402fb47c5d1eb9ecca46fcad5e090de6b3dd0f |

安装器摘要与 runner 回执一致。MSI / NSIS 安装后 EXE 哈希、预期 bundle marker、启动源码 / 关于信息 / WebView 回执匹配；artifact 没有安装后 EXE，该部分仍是 runner 证据，本机未独立提取重算。30 秒首屏观察均已挂载 UI、0.1.10 / Windows、准备态、CLI 禁用、沙箱根初始化，安装 / 卸载 exit 0 且卸载后 EXE 移除。两个 artifact 预计 2026-10-25 17:17（UTC+8）到期，可从 run 的 Artifacts 下载最新候选；本地 ZIP / 安装器留存于 /private/tmp/ocx-0110-369a-windows-3sb0v4xp，未执行安装器。

最新两张 CDP WebView 截图仍可见左侧透明区域黑底、部分文字对比度低，根因未确认；Windows 真机须保留同状态整窗 / CDP 对照，不能据图认定云母或完整原生外观通过。startup-diagnostics 权限字段属于 PowerShell harness 当前 token，不证明目标应用进程权限；沙箱根、DPR=1 与强制收尾也不替代默认安装目录 data、物理 DPI、正常退出或锁释放。

原件见 .adg/evidence/OCX-0110-20261011-369A-WINDOWS-INSTALLER-CANDIDATE/：精选 PNG 与 JSON 原始 gzip 保留 artifact 字节，JSON、日志及 API / 下载报告均附原始摘要和可读视图，checksums.json 覆盖其余文件；没有把 315 项证据中的缓存、偏好载荷或大安装器复制进仓库。此前“构建中”快照继续保留，以上为最终核验结果。后续推进双平台真机 / 最新 SHA 生命周期、配对性能 / 真实周期、真实更新 / 事件和正式签名；IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

### 2026-10-11 main@369ab12d macOS 同根正常退出与重启

最新隔离 DMG 测试副本在 17:37（UTC+8）原生 ⌘Q 后测试进程消失、非阻塞独占锁可取得，锁文件未删除 / 截断。随后 UI 工具自动重开 PID96522 时沿用旧沙箱根；该失败尝试及 after-quit.ax.txt 保留，不计退出证明或同根重启通过。再次正常退出后，于 17:38:37 显式指定同一 test-data 根重启，open exit 0，PID97688 的环境、根、内嵌完整源码 SHA、锁 metadata 一致。

原生设置回读正确根，根 / 年默认展开、月份收起，原备份 bk_20261011082942_7318022d 可见；退出前 / 后、显式重启后及 17:49 最终快照中 preferences.json 与 backup-manifest.json 大小 / SHA256 全部不变。日常 PID75764 及启动时间不变，最终只有一个测试实例。原始快照、只读锁探针方法、AX、失败重开与截图映射见 .adg/evidence/OCX-0110-20261011-369A-MACOS-SAME-ROOT-LIFECYCLE/；截图实为 JPEG，只改扩展名、未改字节。

仅闭合当前 SHA 的隔离同根生命周期局部验证；ad hoc 副本不替代原件正式签名 / 公证、默认 Application Support 根、真实安装升级或更新重启。Windows 实际目标双权限 / 物理 DPI / 默认根 / 原生外观、完整画质矩阵、配对性能 / 真实周期、真实更新 / 事件及正式签名继续待验。IMP / TASK 保持 in_progress，release_authorized: false、release_accepted: false。


### 2026-10-11 main@369ab12d macOS 十分钟 CPU / RSS 观测

18:05:43–18:15:43（UTC+8），唯一隔离测试 PID97688 在浅色 / 高画质 / mesh / 100% 的概览准备态完成 61 次、600.064179 秒采样；Node/npm 已发现、OpenCodex 缺失。应用及同 resource coalition 的 WebKit 进程累计 CPU 增加 350.13 秒，平均 58.34876% 单核（应用约 7.89%、GPU 约 34.17%、WebContent 约 16.28%）。RSS 进程 resident 之和 249.83 → 242.55 MiB，峰 265.97 MiB，不能解释成去重物理内存或整机 CPU。进程身份稳定，采集器本身 CPU 0.2364 秒。

缓存、偏好与调度文件的 SHA256 / mtime 未变；这不证明零网络，也不足以将开销归因于自动检查。无连续前台 / 可见性、主线程或网络仪器，10 秒轮询可能遗漏短命进程；缺受控配对 0.1.9 基线，性能门禁未评定。18:17:44 独立快照确认正确根 / 锁持有、两个备份文件摘要和日常 PID75764 / 启动时间不变。

原件、方法、首末 AX / JPEG、候选 coalition 筛选记录、CPU 分解及摘要见 .adg/evidence/OCX-0110-20261011-369A-MACOS-IDLE-OBSERVATION/。before.png 原件为 JPEG，归档只更正扩展名；未归档其他应用元数据或原始环境。下一步以隔离官方 0.1.9 进行同场景对照，保持一个测试实例。性能门禁及双平台真机、真实周期 / 更新、签名门禁保持不变；IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

### 2026-10-11 官方 0.1.9 与 main@369ab12d 探索性对照

官方 v0.1.9 的源码身份 de6862d40389d527527b9db8ef52e186a58b625f、下载资产及原 EXE 独立摘要已核验；隔离 ad hoc 副本在 18:29:37–18:39:37（UTC+8）完成 61 次 / 600.022329 秒采样。同宿主、浅色 / 高画质 / mesh / 100%、概览准备态，偏好字节摘要相同；累计 CPU 328.09 秒，平均 54.67963% 单核，RSS resident 之和 284.75 → 287.625 MiB、峰 288.171875 MiB。与前述候选 58.34876% 相差 +3.66913 百分点。

| 进程角色 | 0.1.9 平均单核 CPU | 0.1.10 平均单核 CPU |
|---|---:|---:|
| 应用 | 7.62972% | 7.89416% |
| WebKit GPU | 30.59719% | 34.17301% |
| WebContent | 16.45272% | 16.27993% |
| Networking | 0% | 0.00167% |

差值主要出现在 GPU 角色，尚不能证明渲染回归或自动检查开销。顺序观测没有受控预热 / 顺序重复或连续前台 / 可见性记录，独立根缓存 / schema 不同，无主线程 / 网络仪器。数字超过原 CPU 增量预算 0.5 百分点，保留待调查信号；既不认定因果回归，也不记性能通过。RSS 较低同样不构成通过。冷 / 热各至少 20 次、原 CPU / RSS / 主线程 / 零请求和真实周期预算不变。

基线 ⌘Q 退出未生效，normal-quit / quit-settled 快照仍见 PID19382 持锁；候选在基线退出未经独立确认时启动，曾短暂有两个测试实例。18:45 确认精确基线路径后，仅 SIGTERM 清理 PID19382；该项不计正常退出通过。候选早期启动快照未锁就绪，后续 settled 快照及 18:48:50 复核证明唯一候选 PID21610、正确环境 / 根、锁 metadata 一致，原两个备份文件摘要与日常 PID75764 / 启动时间不变；18:49 原生 AX / JPEG 证明准备态恢复。

证据见 .adg/evidence/OCX-0110-20261011-019-369A-MACOS-IDLE-COMPARISON/：33 份文件摘要、28 份复制来源逐一校验，AX 原字节 gzip、其余按字节保存；候选原采样引用已有证据，不重复复制。旧 0.1.9 的 boundary 沿用旧临时目录，仅实际 root 隔离；原件严格签名失败、测试副本 ad hoc 校验不能替代签名验收。后续先核实背景光移层后的几何定位，再开展受控配对；完整性能、双平台真机 / 真实周期 / 更新 / 签名门禁仍开放。IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

### 2026-10-11 main@c1e3c09d 背景光定位修复

光层 Teleport 后，旧相邻查找找不到留在原处的 Logo，回落固定比例。main@c1e3c09d8ed33238543ce5943e439991b8099088 显式传入 Logo 锚点，并复用一个 ResizeObserver 观察光层 / Logo 的布局变化；连续高档动画帧不读几何，后台和销毁仍受原生命周期控制。未改 shader、画质、DPR 或绘图缓冲策略，此修复不解释此前 CPU 差值。

有效修复前回归 5 fail / 1 pass，修复后相关 39 项、完整 UI 95 文件 / 472 项通过；最终测试类型修正后 6 项复核与类型 / 构建通过。首次漏传 active 的无效基线、首次构建类型失败和浏览器陈旧报告均保留。真实 WebGL 中档静态检查覆盖标准 / 紧凑 × 就绪 / 准备 × 浅 / 深 8 布局：中心误差 0 CSSpx，顶部 / 左侧覆盖，无水平溢出。高档 draws 前台持续增加，夹具后台约 15.55 秒不变，恢复后增加；这不是原生焦点或性能验收。

证据 .adg/evidence/OCX-0110-20261011-C1E3-MESH-CENTER/ 的 37 份摘要与 27 份复制来源全部校验。临时 QA 源已归档移除，Vite 停止、后台 tab 关闭；仍保留一个旧 369a 原生测试实例与原日常实例。代码已推送，新源码 CI 38134982244 和本地 macOS 候选构建进行中，Windows 候选待全 CI 通过后按新 SHA 触发一次；旧包不覆盖本修复。配对性能 / 真实周期、双平台真机 / 更新 / 事件与签名门禁不变，IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

### 2026-10-11 main@c1e3c09d 候选证据收口

完整 CI 38134982244 的五个 job 全部成功。19:45:41（UTC+8）查重后只触发一次 Windows 候选 38136743628，20:00:57 completed / success；MSI / NSIS 新装、首屏、卸载在 runner 通过，本机独立重算 ZIP / 安装器大小与 SHA256 均一致。安装后 EXE 不在 artifact 中，摘要仍仅为 runner 回执。沙箱覆写、harness token 和最终强杀不能证明安装位置默认根、目标双权限、普通退出或物理 DPI。CDP 图片仍见黑色透明背景和低对比，须补整窗对照，云母外观不计通过。

macOS 原件已构建：DMG 4384913 bytes / SHA256 821724953dedc7f8ef782d23834edc7f9c035a4bf8b65be0ac88a63989a0b541；原 EXE 7897776 bytes / SHA256 288f36c20354d377d55b29ce430c36f0383e2290e38c713a2167ce684e2dcca6。arm64 / 0.1.10 / 完整源码身份和 hdiutil 校验通过；原件 strict codesign exit 1、无有效签名 identity，正式签名 / 公证仍待闭合。只读 DMG 复制并仅对隔离副本改 identifier / ad hoc 签名后，原生准备态局部高 / 中 / 低与浅 / 深观察完成；不替代完整矩阵、动画 / 焦点 / 性能验收。

19:45:58 退出快照确认测试进程消失、锁可取得且字节未变；19:46:12 同根重启，后续确认唯一测试 PID28269、正确三项沙箱环境 / 源码、锁重新持有。管理器偏好备份 bk_20261011114045_e3fa4410 的清单 562 bytes 与载荷 1369 bytes 摘要在六次生命周期快照及末次复核均未变，重启原生回读根 / 年展开、月份 10 收起、记录可见。日常 PID75764 / Oct 9 22:00:24 启动时间不变。18-keyboard-to-payload 实际焦点在清单，22 用了错误半尺寸坐标；失败与误点保留，不计通过。

证据归档在协作包 `验证/20261011-c1e3-候选最终核验/`（本轮收口，非版本最终验收）：155 项归档摘要、153 份复制来源与无损 roundtrip 全部通过，10 份大文件仅保留摘要。按 TASK write_scope 写入协作包，未新增范围外 .adg/evidence；不复制二进制 / 载荷 / WebView profile。该目录列明候选身份、失败签名与后续真机 / 性能 / 更新 / 事件交接。

下一步为受控 0.1.9 配对性能 / 缓存零请求与真实 stable 24h / beta 6h 周期。旧十分钟顺序对照的 +3.66913 百分点仅为待调查信号，c1e3 几何修复不解释该差值；预算不变。八项 required_checks、双平台真机 / 真实更新 / 正式签名门禁仍开放；IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false，不创建关闭证书。

### 2026-10-11 缓存探针、用户协调前台复测与有效性回写

main@3ed00487958185a520e61ed2ba2d706f95e9a384 已提交推送，只新增 macOS 更新查询探针和自检程序；应用实现 / 同 SHA 候选仍为 c1e3。自检 exit 0（0.855883 秒），本地 loopback 覆盖网络 / spawn / exec 边界、非沙箱与符号链接保护及脱敏记录。真实手动查询阳性对照记录 DNS 4、connect 2、npm view 的 spawnp 1 / spawn 1，证明真实调用路径可观测；这些是边界调用尝试，不是成功请求数，子进程注入被清除，不能声称全系统网络捕获。注入轮次不用于 CPU / 启动比较。

用户明确“失去前台是我操作的原因，不是软件问题”，作为用户报告的原因记录；原始无效轮次仍保留，不推导为软件焦点缺陷。初始窗口未显示的样本原因另为未确认。用户随后让出前台并允许再试一次；本轮只执行一次协调复测，21:49:19 同隔离根启动注入 PID10029，原生就绪并 Raise 后采样 46.432740 秒，47 / 47 前台且可见、边界调用尝试 0，真实缓存 / 调度摘要不变且截止仍在未来。结合旧 PID2647，有效重启累计 2 / 10。已有 phase verifier 对 opportunity01 保留 null 前台字段，另用同 PID 原始采样独立合并核验，没有改写原件。采样间约 1 秒，不证明间隔中全部状态或启动至采样开始的连续前台。

旧页面组的设置 / 概览各 100 次及 200 个 AX URL 匹配，但动作时间窗 202 个采样全部非前台，因此不计有效缓存 100 次概览进入验收。该组与其余无效轮次保留，不能根据零边界记录或文件名 valid 认定门禁通过。

结果已告知用户并释放操作。注入 PID10029 通过 ⌘Q 正常退出，独立快照确认进程消失、锁可取得且字节不变；21:51:37 同根恢复非注入 PID10336。21:54:45 只读快照确认唯一隔离实例、沙箱根与边界、锁 metadata / 持有状态正确，备份清单与载荷摘要未变。日常 PID75764 / Oct 9 22:00:24 启动身份不变，保留一个普通测试实例，不再自动占用前台。

详细方法、阳性对照、真实缓存截止、失败 / 无效原件、用户原因说明和本轮复测见[缓存探针与前台门禁](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-c1e3-缓存探针与前台门禁/README.md)。82 份原始来源逐字节 / gzip roundtrip、86 项证据摘要及五项二进制仅摘要核验通过；历史计算指标保留，新增记录不覆盖旧轮次。不复制应用或偏好 / 缓存 / 备份载荷。

后续仍需 8 次有效重启、持续前台 100 次概览进入、受控 0.1.9 配对性能与真实周期 / 睡眠 / 离线慢网超时并发通道隔离，以及双平台原生 / 安装 / 真实更新 / 事件 / 正式签名验收。冷 / 热各 ≥20 次、交互 p95 ≤100ms 且增量 ≤5%、十分钟 CPU 增量 ≤0.5 百分点 / RSS ≤10MiB、无主线程任务 >50ms 等预算不变。八项 TASK.required_checks 仍开放；IMP / TASK 为 in_progress，release_authorized / release_accepted 为 false，整体 RCP failed 表达验收未完成。未迁移真实数据、替换日常安装、晋升核心事实、创建关闭证书或公开发布。

### 2026-10-12 回写 main@deb740c0 时钟修复与缓存前台复验

以下证据采集于 2026-10-11，回写于 2026-10-12。最新源码为 main@deb740c09857b5f0bef7c9b4090fe5e8587bb4b3，已提交推送。UI 启动 45 秒宽限使用单调时钟；后端只重整当前管理器通道和面板异常超远的未来截止，保留历史 / 失败计数及非活动通道，持久化一次且不立即查询。关闭自动检查、忙事务或数据根冻结时不写截止、不规划查询；非法历史拒绝读取，不覆盖原件。缓存 schema、存储路径和周期不变。

有效修复前 UI 7 pass / 2 fail、Rust 3 fail；修复后相关 UI 24 项及 Rust 命令边界 4 项通过。完整 UI 95 文件 / 474 pass，Rust workspace 34 suites / 866 pass / 0 fail / 6 ignored，fmt / strict clippy / 类型 / 构建 / diff exit 0。ignored 的 live、钥匙串、npm、WebDAV、helper 与 tray 等不计通过，早期日志未计时不补造持续时间。修复与原件见[调度时钟边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-调度时钟边界/README.md)。

用户让出前台后，以 deb740 同 SHA 隔离副本独立完成 10 个 PID 的有效缓存重启：每轮 47 个前台 / 可见窗口采样，单调时间至少 46.335297 秒，probe_loaded 各 1、主进程 DNS / connect / exec / spawn 边界尝试 0。设置 / 概览交替各 100 次、200 条 AX URL 序号匹配，动作窗位于 271.711039 秒、2493 个前台 / 可见采样内，同 PID 边界尝试 0。管理器 stable、面板与调度共三个文件大小 / SHA256 不变，截止仍在未来。第 2 轮误用 PID20962、超时 exit 2 的原件与更正 PID20777 的监控一起保留；错误样本不计通过，不认定应用焦点故障。历史 c1e3 的 2 / 10 和无效页面组保留，本新轮次独立满足 10 / 10 与 100 次的局部缓存检查。

仅证明有效缓存、未来截止、准备态及仪器采样时间窗；不证明采样间每一瞬间、全系统零网络、就绪态更新或性能。注入轮次不计启动 / CPU / 交互性能，阳性方法仍引用此前 c1e3 自检和真实手动查询对照，本轮未新增 deb740 阳性对照。第 10 个注入实例正常 ⌘Q 退出后锁可取得，未删除 / 截断锁；同根恢复非注入普通实例。UTC 2026-10-11T15:22:44.636615+00:00 末次快照为唯一测试 PID23060、正确根 / 锁、无 DYLD / probe 环境；测试 EXE 与缓存 / 调度摘要稳定，日常 PID75764 及启动身份未变。实例状态是快照，前台复验已结束，保留一个普通隔离测试实例。原件及限制见[deb740 缓存前台复验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261011-deb740-缓存前台复验/README.md)。

[CI 38147288262](https://github.com/gzers/opencodex-desktop/actions/runs/38147288262) 五个 job 成功；查重后只触发一次 [Windows candidate 38149645814](https://github.com/gzers/opencodex-desktop/actions/runs/38149645814)，UTC 2026-10-11 15:16:58 success。同 SHA ZIP / MSI / NSIS 大小、SHA256、CRC 与回执一致性独立核验通过；runner 新装 / 首屏 / 卸载通过，安装后 EXE 不在 artifact 内，该摘要仍仅为 runner 证据。本机未运行 Windows 安装器。harness token、沙箱根、DPR=1 与强制收尾不能替代实际应用 token、默认安装目录 data、物理 DPI 或正常退出。CDP 透明侧栏黑底与低对比根因未确认，需真机整窗对照，云母外观不计通过。

同 SHA macOS DMG 4391555 bytes / SHA256 b12b55e4c78e526d693d7deb9698f7922b9b2548f03befca63e9b9a1b88db877；arm64、版本 / build 0.1.10、完整源码身份与 hdiutil 通过。原件 strict codesign exit 1、无有效签名 identity / TeamID，正式签名 / 公证未通过；隔离 ad hoc 副本不替代原件。两轮分别归档 40 / 139 项摘要、38 / 138 份来源无损 roundtrip、2 / 8 项编译产物仅摘要；普通运行日志按字节快照留存，无复制数据根、用户载荷或 WebView profile。138 个无关脏路径保持原字节，TASK 未改。

性能措辞澄清：前节“交互 p95 ≤100ms 且增量 ≤5%”表述不准确，实施前[执行基线与契约](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/06-执行基线与契约.md)始终要求同机官方 0.1.9 配对、冷 / 热各至少 20 次，**启动可交互 p95 增量 ≤100ms 且 ≤5%**；闲置 10min CPU 增量 ≤0.5 百分点 / RSS 增量 ≤10MiB，检查无 >50ms 主线程任务。阈值未改，旧探索性 CPU +3.66913 百分点仍为未解决信号，不认定因果回归或通过。

后续继续受控配对性能、真实 stable 24h / beta 6h、睡眠 / 跨启动、最新 SHA 离线 / 慢网 / 超时 / 并发 / 通道隔离，以及双平台原生 / 安装 / 真实更新 / 事件与正式签名。交接基线已同步 deb740；八项 required_checks 仍开放，整体 RCP failed 表示版本验收未完成，acceptance_results 为空。IMP / TASK 为 in_progress，release_authorized / release_accepted 为 false；没有核心事实推广、真实用户迁移、日常安装替换、关闭证书、tag 或公开发布。

### 2026-10-12 官方 0.1.9 与 main@deb740c0 十分钟性能对照未通过

同一 macOS 27.0.1 arm64 宿主顺序运行官方 0.1.9（de6862d4）与最新 0.1.10（deb740c0）的隔离 ad hoc 副本；浅色 / 高档 / mesh / 100%、概览准备态，各自预热约 184.54 秒后完成 61 次 / 600 秒采样。平均单核 CPU 为 48.340381% / 50.158010%，增量 +1.817629 百分点，超过 ≤0.5 百分点预算；平均 RSS 为 284.617572 / 401.626537 MiB，增量 +117.008965 MiB，结束 RSS 增量 +220.671875 MiB，超过 ≤10 MiB 预算。RSS 是进程 resident 之和，候选轮内增长 +211.390625 MiB，主要出现在 WebContent；不认定去重物理内存、泄漏或自动检查根因。

样本进程身份、四个偏好 / 缓存 / 调度文件摘要稳定，采集器自身 CPU 分别 0.317583 / 0.324500 秒。该项不证明零网络；单组顺序对照、准备态、无连续前台 / 启动交互 / 主线程 / 网络仪器仍限制结论。性能预算比较未通过，不能用缓存局部通过覆盖。配对后的 WebContent sample 指向 SVG mask / stroke bounding box 重绘链，vmmap 的 WebKit Malloc 增长仅是后续调查线索；没有基线 vmmap 或可控消融，不认定因果。

设置页另做 90 秒短时诊断，CPU +0.07 秒，RSS 611.125 → 580.109375 MiB；锁屏起点未知、缺同时结束 UI / 连续前台，不归因于页面卸载、不计设置验收。用户让出前台后 CUA 仍报告 Mac 锁屏，已请求人工解锁；复用唯一测试实例，不重复尝试解锁或开新窗口。基线 ⌘Q / 菜单退出未生效，精确 PID29226 的 SIGTERM 仅作清理，不计正常退出。UTC 17:18:16.449950 快照为唯一隔离测试 PID31501，日常 PID75764 / 启动时间未变，属于快照结论。

方法、首末 AX / JPEG、原始采样、失败退出、锁屏限制及独立重算见[十分钟性能对照](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-deb740-十分钟性能对照/README.md)。46 项 manifest、42 份来源无损复制、3 份显式筛选归属衍生、3 份 gzip roundtrip、4 张 JPEG 扩展名修正和离线重算全部通过；未复制二进制 / 用户载荷 / WebView profile / 其他应用元数据。

下一步解锁后做中档静态、高档 CSS 与必要的 mesh 返回诊断，记录首末设置和连续前台以隔离原因，保留已确认的光层边距覆盖。建立复现并修复后重跑十分钟配对；原 CPU / RSS、冷 / 热各 ≥20 次、启动可交互 p95 增量 ≤100ms 且 ≤5%、检查无 >50ms 主线程任务、真实周期及双平台原生 / 安装 / 更新 / 事件 / 签名门禁不变。review:scheduler-performance-0110 记录 failed，八项门禁仍未整体闭合；IMP / TASK in_progress、acceptance_results 为空、发布未授权 / 未验收。

### 2026-10-12 main@fb34f5 Windows 应用进程权限与候选安装包复核

`main@fb34f5b91c3a6feb5b41f11f9527171c4bf742f5` 相对 deb740 仅修改 7 个 Windows smoke / token helper / workflow 文件，无应用实现变化。[CI 38162783294](https://github.com/gzers/opencodex-desktop/actions/runs/38162783294)与[候选安装 38163233949](https://github.com/gzers/opencodex-desktop/actions/runs/38163233949)同 SHA 均 completed / success。完整日志独立统计：Windows Rust 与候选 Rust 各 34 组 / 585 passed / 0 failed / 6 ignored，macOS Rust 34 组 / 866 passed / 0 failed / 6 ignored，前端 95 文件 / 474 passed，token helper 9 项通过；零测试组保留，ignored 不计通过。最新下载证据未含 macOS DMG，不声明其独立核验完成。

4 个 artifact 的 API 大小 / SHA256、源码身份、CRC 和安全条目路径通过；下载 MSI 4538368 bytes / SHA256 834dba519f3c159762ceb2656f312137806dbdf22f926e6c270c06cc1422a771、NSIS 3232774 bytes / SHA256 49f8d0e51fdcbce234ab0c70225d73e33486a2ef6fad517b3874597b504482b2 与回执一致。runner 新装 / 首屏 / 卸载通过，安装后 EXE 不在 artifact 内，该摘要仍为 runner-only；本机未运行 Windows 安装器。CI PID8308、MSI PID2592、NSIS PID2792 的实际应用 token / 全路径 / 源码匹配，均 elevated=true、elevationType=1、applicationTokenVerified=true；mode:any / applicationModeMatched=false，不计普通 / 管理员双权限验收。沙箱 override、DPR=1 和自动收尾不计默认安装目录 data、物理 DPI、正常退出、真实升级或用户数据保留。正式签名 / 发布 false，云母 / 标题栏 / 菜单及 CDP 黑底根因仍需真机确认。

证据与限制见[Windows 进程权限核验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-fb34f5-Windows进程权限核验/README.md)：34 份来源无损复制、6 份 gzip roundtrip、39 项摘要清单；原 helper-only 核验保留原义，不复制安装器 / 用户载荷 / WebView profile。前台锁屏等待人工解锁，复用唯一隔离实例，无新的前台诊断结果。既有十分钟 CPU / RSS 性能失败继续保留，原阈值不变；八项门禁未整体闭合，TASK / IMP in_progress、acceptance_results 为空，无核心事实推广、真实迁移、日常替换或发布。

### 2026-10-12 deb740 中档静态前台中断诊断，不计验收

复用唯一隔离测试 PID31501 / 同 deb740 EXE，概览准备态、浅色 / 中档静态 / mesh / 100%，预热约 91 秒后完成 19 个资源样本 / 180.009340 秒。前台监控 2427 个样本中 2060 个有效、367 个无效，首次无效 UTC 00:34:47.150252 为非前台但窗口可见；结束 CUA 报告锁屏，缺结束 AX / JPEG，整轮无效。锁屏起点未知，不能认定焦点缺陷；用户此前人工操作原因按用户报告保留，不作软件问题。三次准备阶段和另一次 preflight 在 10 秒内未前台就绪，均未资源采样，原件保留。

首次无效前仅 14 个资源样本 / 129.921825 秒（不是 140 秒）：累计 CPU +0.01 秒、单核 0.007697%，RSS 408.140625 → 407.953125 MiB。该子区间为调查线索，不是完整有效轮次、官方配对或因果证明。整轮原资源 CPU +0.10 秒 / 0.055553%、RSS 408.140625 → 269.531250 MiB 受末约 40 秒非前台限制，不归因于中档、页面卸载或锁屏回收。进程身份和四个文件摘要稳定，不证明零网络或无泄漏。

场景清单 context.candidate.appearance 仍写 high，来自旧静态 manifest；真实首末 settings 和同一 preferences SHA 均为 mid，回写明确两者区别，原件不修正。首 AX 是树未变化 / overview 焦点的增量，不能当完整树；settings-end.json 本身有效，没有修复。44 份来源无损归档、1 份 admitted-only 衍生、1 份 gzip roundtrip、47 项摘要与外部 / 归档输入独立重算通过；排除其他应用元数据、应用二进制、数据载荷和 WebView profile。详见[中档静态中断诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-deb740-中档静态中断诊断/README.md)。

用户已回复解锁，但末次原生控制仍报告锁屏，等待“桌面可操作”后再用同一个实例做中档 / 高档 CSS / 必要的高档 mesh 对照，补首末 UI 与持续前台；代理 Finder test-install 标签页待操作恢复后关闭。暂无根因或源码修复，不凭部分低 CPU 取消既有十分钟失败。原性能预算、光层边距覆盖、真实周期和双平台原生 / 安装 / 更新 / 事件 / 签名要求均不变；review:scheduler-performance-0110 failed、八项门禁开放，TASK / IMP in_progress、acceptance_results 为空、发布未授权 / 未验收。

### 2026-10-12 前台采样保护工具准备，不计应用验收

外部诊断工具补齐失焦 / 窗口隐藏即停、超过 1 秒的采样间隔与管道断流看门狗、原始非阻塞管道读取及只终止自有子进程。采集读真实持久化画质，保留旧清单 high 与实际 mid 的差别；预热前后核验唯一测试实例 / EXE / 进程身份，首末偏好须未变。工具不启动、激活或退出应用，不改系统设置。

Swift / Python 编译、当前 25 项合成自检和 4 项自有子进程 / 管道测试通过，首次自检失败及测试修正保留。完整 run_guard 实际应用集成、首末完整 AX / 截图和资源采样均未执行，不证明当前原生就绪或应用性能通过。此前用户回复解锁后原生工具仍返回锁屏，本轮没有追加 CUA 重试；只读快照确认唯一隔离 PID31501 / deb740 EXE / 浅色中档 mesh 100%，日常 PID75764 与 Oct 9 22:00:24 启动时间未变。

[采样工具准备证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-前台采样保护工具准备/README.md)按 TASK scope 归档 10 份无损来源与 12 项清单，未复制二进制、缓存、真实配置载荷或其他应用元数据。后续实际可操作后复用单实例补场景诊断与完整 UI 证据，确定原因再修源码，保持顶部 / 侧边光层覆盖。既有十分钟 CPU / RSS 失败和原预算不变，review:scheduler-performance-0110 failed；八项门禁未整体闭合，TASK / IMP in_progress、acceptance_results 为空、发布未授权 / 未验收，无源码修改、真实迁移或日常安装替换。

### 2026-10-12 内屏前台保护实际中断，不计性能验收

用户放好唯一隔离测试窗口后，只读位置查询确认 PID31501 主窗口完整位于 MacBook 内屏（x=-1453、y=211、1180×760），NSWorkspace 前台为目标；AX 查询 -25204 不可用，不作相反焦点结论。原 JSON 未保存，归档位置说明明确来自工具对话记录。继续复用 deb740 副本，浅色 / 中档静态 / mesh / 100%；旧清单 high 与真实 mid 的区别保留。

实际 run_guard 内屏轮先就绪，172 条事件中最后有效样本 elapsed 18.644822 秒，首次无效 18.750805 秒为非前台但仍可见；monitor exit 3、resource_collector_exit=null。90 秒预热未完成，资源采集未启动、样本零，无 summary.json；不计 CPU / RSS 或启动验收。此前 finder / prearmed 两轮均 10 秒未就绪、exit 2、资源零。三轮首末偏好摘要一致。窗口失去前台的原因未确定，不认定软件缺陷；用户此前人工切换按用户说明保留，不作软件问题。

内屏首末 JPEG 已目视核对同一概览准备态；开始 AX 仅增量，结束为完整树，截图不证明期间持续前台。Finder 准备轮无完整末 UI，不补造证据。已关闭代理打开的 test-install 标签，保留原“下载 / 存储 / work”三个标签；这一轮不再占用前台重试。[内屏中断证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-内屏前台保护中断/README.md)保留目标日志、原设置摘录、UI 与明确的衍生来源，按 manifest 核验；无产品源码变化。

既有性能比较未通过及其受控 / 连续前台限制保持原义，不能用本轮零样本解除。后续先取得可持续前台的场景诊断、确定原因再修源码，保留光层顶部 / 侧边覆盖。原性能预算、冷 / 热配对与主线程、真实周期及双平台原生 / 安装 / 更新 / 事件 / 签名门禁不变；review:scheduler-performance-0110 failed，八项 required checks 未整体闭合，TASK / IMP in_progress，acceptance_results 为空；无核心事实晋升、真实迁移、日常替换或发布。

### 2026-10-12 调度状态读取失败退避修复，局部验证

main@01a1d85504781f4cb189a68ed85d1d803a1409ff 已提交推送。自动调度读取状态失败后，原先只设置 30min 计时器；finally 中排队的唤醒或后续前台 / 网络恢复会把它改为 100ms。修复在错误分支以 performance.now() 保存重试下限，统一 arm 遵守该下限；仍只有一个计时器。启动 45s、原生三对象持久截止 / 网络退避、手动更新、路径 / schema 均未改变。

新增正负 7 天系统时钟变化、100 次在途唤醒、隐藏 / 恢复和 100 次后续唤醒回归：修复前 9 passed / 2 failed，失败为计划读取提前发生；修复后调度 11 / 11 通过，退避届满才查询，到期触发及 stop 清理通过。更新相关 8 文件 / 38 项（含调度 11 项）通过，类型检查与生产构建通过；构建保留 4 条动态导入不能拆分 chunk 的警告。日志、提交身份与摘要见 [退避修复证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-调度状态失败退避/README.md)。协作包执行契约同步说明此本进程保护，未将代码复制到文档分支。

本轮没有前台资源采样、原生安装或真实远程更新；单元测试的虚拟时间不代表实际 24h / 6h 周期，修复也尚不能解释此前 CPU / RSS 增量。原性能失败及预算保留，review:scheduler-performance-0110 failed、八项门禁仍未整体闭合。TASK / IMP 保持 in_progress，验收结果为空；实际运行的旧隔离副本未替换，不操作日常安装或真实数据，不晋升核心事实、不关闭或发布。

### 2026-10-12 Windows 接续核验，原任务与候选制品待同步

Windows 执行机已 fetch 并核对 main@01a1d855 与文档起点 a83250ba；隔离源码 worktree 精确检出该 main SHA。实际宿主是 Windows Server 2025 Datacenter / build 26100 / x64，未标为 Windows 11。日常 0.1.9 安装与 PID2232 保留，本轮未启动任何候选实例。工作区既有长路径证据删除状态原样保留。

同 SHA CI 38196769142 的五项前置 job 已通过，Windows 回归、release EXE build 与最终 smoke 步骤均 success；初次核验时 macOS bundle build 仍在运行，11:11:50（UTC+8）刷新后整条 CI completed / success，六项 job 均 success，首末快照分别保留。同 SHA 未发现 Windows candidate installer run，现有两份制品是启动 / 令牌证据，未下载，不计为安装器或本地摘要核验。精确源码上的本机令牌工具 9/9、CDP / bundle identity 工具 12/12 通过；实际普通令牌对象为 cmd 子进程，不替代候选应用双权限验收。

原 TASK-182 在当前执行机缺失，`.adg/work/` 被忽略，全部现有 refs 中也无该 TASK 的提交历史；不得从旧 RCP digest 重造契约或正式回执。本机无可用 GitHub 写入凭据，禁止交互的推送预检 exit128，尚不能触发安装器或推送；未检测到 PATH 中 cargo / rustc 或默认位置 vswhere。本轮归档[Windows 接续核验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows接续核验/README.md)，待原执行机同步 TASK 与可核验同 SHA 安装器，并恢复本机 GitHub 认证后继续。

候选原生、安装、更新、备份、树表、事件、30min 退避、同机配对性能及真实周期仍待验。原性能预算、review:scheduler-performance-0110 failed 与八项门禁未整体闭合的结论不变；IMP 继续 in_progress，不更改本机缺失的 TASK 状态，不颁发正式 RCP / 完成证书，不晋升核心事实、不替换日常版、不迁移真实数据、不发布。

### 2026-10-12 认证恢复、精确Windows候选及沙箱隔离修复

用户完成本机 GitHub 认证，已核实 gzers 写入权限并推送文档 b783d1b。精确 main@01a1d855 的候选构建 38199793144 success，版本 0.1.10；两份制品 ZIP 已下载且 GitHub digest 一致，MSI 内主 EXE 只读提取后与 CI 安装结果摘要一致。CI 中 MSI / NSIS 真实应用令牌均 elevated=true，不能代替本机双权限验收。官方 0.1.9 发布 MSI 与提取基线已备妥，未启动。

只读 MSI 注册核对发现日常版与候选共享 UpgradeCode，不能以不同 INSTALLDIR 保证隔离，当前执行机未运行安装器。启动前源码检查还发现 Windows Known Folder HOME 与宿主工具发现未随显式沙箱隔离，CLI named pipe 只按账户区分。修复 25d115b 已提交推送 feature/0.1.10-windows-isolation：沙箱 HOME / 工具候选限制在根内，CLI 按 canonical root 隔离，坏根拒绝回退，日常身份保留；增加真实 Windows 子进程与命名管道验证，环境测试不再修改并行进程全局环境。后续 6b9abc3 继续保留原数据根解析与拒绝回退断言，并在 Windows 真实子进程检验公共入口。格式检查通过；25d115b 的 CI 38201539240 / 候选 38201580630 与最终 6b9abc3 的 CI 38201999141 / 候选 38202038747 按 SHA 分开观察，仍待编译 / 回归结果，尚未并入 main。证据见[Windows 候选与隔离修复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows候选与隔离修复/README.md)。

TASK-182 原始对象仍待同步；本轮无正式 RCP、无候选真机运行 / 性能 / 真实周期证据，不改变既有性能失败、原预算或未完成门禁。日常版及真实数据保留，IMP in_progress，不提前关闭、不晋升核心事实、不发布。

### 2026-10-12 Windows普通权限单实例、真实备份事件与树表对齐修复

后续0a83761补齐沙箱子进程AppData隔离和真实Node验证；精确feature CI38202731337、候选38203025770及并入main后的集成CI38204104373均success。ZIP与只读提取EXE摘要匹配，版本0.1.10。本机首次启动因测试AppData目录缺失失败，原件保留；补齐测试目录后复用同根，唯一PID41528实际普通令牌false/type3、源码0a83761匹配。原生观测为Windows Server2025 build26100、DPI192/DPR2，前台云母属性2，标题栏系统按钮可见；不抵管理员、其它物理DPI或标题栏操作。

真实UI生成偏好备份，默认根/年展开、月份收起；theme不同值恢复与原SHA一致、用户pin保留、恢复临时保护解除。真实坏策略失败事件未被管理器检查解除，同内容备份重试才精确恢复。真实面板查询缓存与截止在manager beta→stable切换后保持；该检查不覆盖在途事务，未来24h截止不计已完成真实周期。原生输入恢复重试报GetCursorPos拒绝访问，后续证据明确为真实Tauri命令，UI确认待验。

原生树表暴露操作按钮低6px；cdfd706修复共享首行节奏，命中高度保留。真实Vue浏览器红绿覆盖两主题/四CSS缩放，旧组件失败、修复8组通过；全UI95文件/476项、类型和构建通过。4f7f334恢复误占的原QA入口并分离几何夹具，独立入口8组通过。当前main4f7f334已推送，CI38205914227/候选38206176747仍在运行；本机旧0a实例未替换，不宣称修复后原生验收。

6b9旧CI三项2秒IPC等待失败、本机同EXE定向通过及完整库496通过/1路径依赖失败/4忽略分别保留；未延长超时、删断言或用新SHA抹去旧失败。见[单实例局部门禁证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows单实例局部门禁/README.md)。TASK原件、双权限/完整DPI、安装卸载、更新与树表复验、轮换/全事件、30min错误退避、同机配对性能及真实周期仍待验；原性能失败/预算及八项门禁保持，IMP in_progress，不颁发RCP/证书、不操作日常版/真实数据、不关闭或发布。

### 2026-10-12 轮换局部通过、4f生产树表复验与npm路径故障

4f7f334的main CI38205914227及候选38206176747均success，下载ZIP及提取EXE摘要匹配。专用supervisor确认旧0a实例退出后，同根启动唯一4f PID20100，普通实际令牌false/type3。实际DPR2/DPI192的生产WebView中三行名称/箭头/用途/操作中心偏差0px、按钮34px，正文截图已核对；foreground=false且RDP断开，不抵完整交互/主题/密度/DPI门禁。

真实14份近期备份预览不删；两组显式历史日期夹具分别覆盖手动旧预览拒绝/固定保留与成功新备份后自动轮换，最终普通10份+固定项，存活manifest恢复原字节、策略回manual。坏载荷恢复被拒绝、偏好不变、原载荷恢复，未决保护保留。日期夹具不计真实30天/更新周期。真实custom-root初始化后引用切换blocked/runtime_unverified，不绕过准入。

为接续真实更新和状态回读，唯一4f GUI生产安装官方2.82.0，下载校验后npm安装栈溢出失败，来源仍unresolved；2006条原始事件及npm日志保留，旧阶段/日志percent不算可信下载进度。日志与同目录Arborist复现指向npm对canonical的\\\\?\\路径处理：带前缀失败、普通路径成功。ee65f11已推送feature/0.1.10-windows-npm-path，在npm参数边界使用锁中既有dunce1.0.5安全简化，内部canonical/保护不变；真实Node回归和本地fmt准备，CI38208548177/候选38208582603仍在运行，未并入main或宣称实际安装修复通过。

见[轮换与npm路径故障证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows轮换与npm路径故障/README.md)。Junction夹具命令被自动审批以blocked by policy拒绝，未执行或绕过；原TASK、双权限/其余物理DPI、安装卸载/custom-root成功、真实更新、全事件/在途通道、30min退避、同机配对及真实周期继续待验。性能failed/原预算/八项门禁保持，IMP in_progress，保留日常版和真实数据，不关闭或发布。

### 2026-10-12 npm路径实际复验、scripts_required与工作目录补丁

ee65f11精确feature CI38208548177/候选38208582603success，已并入推送main，main CI38210214739仍在运行。制品ZIP/MSI/EXE身份均核验；旧4f退出后原根唯一PID21592实际普通令牌false/type3。真实2.82.0无脚本重跑已完成npm install（102依赖/exit0），旧路径栈溢出消失，但版本探针返回scripts_required，来源仍unresolved、失败保留；不计完整安装或官方升级通过。

FZ-50明确用户二次确认后才可allowScripts=true重跑一次；未开启脚本。只读包/依赖锁审阅核实精确2.82.0摘要与bun1.4.2 postinstall，具体目标/权限/代码执行范围已形成待确认计划。npm日志还揭示cwd为manager EXE目录，e1a1aaa已推送feature/0.1.10-windows-npm-cwd补齐pack/install工作目录及真实Node断言，CI38210513009和候选待验，未并入main。见[npm复验与脚本门禁](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windowsnpm复验与脚本门禁/README.md)。

原TASK/桌面、双权限物理DPI、安装卸载/切根、真实升级/事件/30min退避/性能周期仍未完整闭合；原性能failed/预算/八项门禁保留，IMP in_progress，不颁发证书、不替换日常版、不迁移真实数据、不关闭或发布。

### 2026-10-12 主题回读实证与一次性历史导入修复

脚本二次确认尚未收到，allowScripts保持false。继续只读核查唯一ee实例发现：磁盘schema2/appearance.theme=system，实际get_preferences却返回themeNeedsImport=true，读前后摘要一致。后端只认顶层theme导致新格式误判；前端apply先写缓存又造成原历史值被覆盖、空缓存额外save以及导入失败error被清除，新增测试在旧store上9通过/3失败，原件保留。

cf71ea4已提交推送feature/0.1.10-preferences-theme：识别嵌套及扁平显式主题，先捕获历史缓存、避免创建自导入、save失败保留缓存/error；新增真实磁盘及四项前端合同回归。定向12/12、全UI95文件/480项、typecheck/构建/fmt通过，Rust编译和实际修复制品DTO仍待CI38211754315/候选38211818636；尚未并入main。详见[主题回读证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows主题回读与历史导入/README.md)。

原TASK/桌面/脚本授权缺口与所有未完成原生/安装/更新/30min退避/同机性能/真实周期门禁保留，既有性能failed和预算不变；唯一ee测试实例、日常安装文件和真实数据未替换，不颁发RCP/证书、不关闭或发布。

### 2026-10-12 npm工作目录精确制品实际回读

e1a1aaa精确feature CI38210513009与候选38210653937success，已快进推送main，集成CI38212329011仍运行。候选ZIP/MSI/提取EXE摘要匹配，旧ee退出后原根唯一PID32564真实普通令牌false/type3。无脚本官方2.82.0生产重跑pack/install均exit0；原npm日志核验install cwd=prefix、pack cwd=download destination且均归验证临时区，安全简化路径无原栈溢出。真实Windows回归日志无损归档，未调整原断言/超时。

重跑仍以scripts_required终止、来源unresolved；脚本二次确认未收到，不开启allowScripts、不计完整安装或官方更新通过。主题cf71ea4精确CI/候选仍待结果，不沿用e1原生证据。详见[工作目录实际复验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windowsnpm工作目录实际复验/README.md)。原TASK/桌面/双权限/完整DPI/安装卸载/更新/30min退避/同机性能/真实周期继续开放，性能failed/原预算保持，IMP in_progress，不关闭或发布。

### 2026-10-12 主题修复通过支线回归并与cwd集成

cf71ea4精确CI38211754315/候选38211818636均success，主题支线ZIP下载摘要一致，未在本机运行支线应用。主题支线与已通过cwd的main e1以双亲合并为afff490，已推送main；首次因提交身份未设而停止，后续命令级既有Codex身份完成无冲突，不改全局身份。代码归属未变。

合并SHA的CI38213445710与Windows候选38213528684已启动；不能用两个父提交绿色替代合并精确制品与原生DTO复验。见[主题与cwd集成证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows主题与cwd集成/README.md)。当前仍原根唯一e1 PID32564、脚本未开启；原TASK/桌面/脚本二次确认及完整原生安装更新性能周期门禁保持，IMP in_progress，既有性能failed/预算不变，不关闭或发布。

### 2026-10-12 合并精确制品与主题原生回读局部验证

afff490合并候选38213528684 completed/success，ZIP/MSI/提取EXE身份与CI安装EXE核验一致。main CI38213445710前置Windows回归/最终EXE/smoke通过后进入macOS打包，14:57观察仍in_progress，不记整条通过。八个相关合并文件blob与已测父提交完全一致，只作源码核验。

原e1测试进程确认退出后复用同容器启动唯一afff PID29936，实际普通令牌false/type3、startup源码匹配。production DTO/store reload记录schema2 appearance.theme=system、themeNeedsImport=false、cache=system；启动与reload前后偏好SHA/mtime完全不变，原用户pin跨启动保留。由真实合并制品纠正此前DTO误判，不抵全部历史导入/主题/物理DPI/前台交互矩阵；记录foreground=false/RDP断开。

见[合并制品主题原生回验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows合并制品主题原生回验/README.md)。脚本二次确认、原TASK、桌面与完整双权限/安装更新/30min退避/同机性能/真实周期门禁继续开放；原性能failed/预算不变，IMP in_progress，不颁发证书、不迁移真实数据、不替换日常安装、不关闭或发布。

### 2026-10-12 合并main最终CI通过，完整目标仍未达成

afff490的main CI38213445710于15:05:46（UTC+8）completed/success，六项job含macOS arm64实际bundle构建均success，最终原回执已归档；候选38213528684与精确本机制品身份另有记录。当前原根唯一普通PID29936身份/创建时间/supervisor匹配，日常EXE摘要一致，未替换真实安装。

按原目标逐项审计保留所有明确范围，详见[最终CI与未闭合门禁审计](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终CI与未闭合门禁审计/README.md)。CI/局部原生/备份/树表/事件证据不抵管理员/完整物理DPI/实际安装升级卸载/切根成功/完整更新与安全矩阵、真实30min退避、同机性能及真实周期。原性能failed/预算及八项门禁未闭合保持；目标overallResult=not_achieved，无正式RCP/证书。

原TASK缺失、FZ50脚本二次确认未收到的接续阻塞仍存在。写入本轮审计时query session原始输出已显示15119/session2 Active，重查确认；提交前纠正沿用Disconnected的状态，当前可继续独立原生工作，goal保持active。本IMP仍in_progress，不关闭TASK/AC、不降低门禁、不晋升核心事实、不迁移真实数据、不替换日常版、不发布。

连接恢复后同一afff普通/200%实例完成系统按钮最大化、Alt+Space真实菜单还原、系统最小化及恢复；原主HWND/PID核验与实际effect暂停/恢复云母证据保留，只覆盖这些具体操作。AX还原输入未生效不计通过；最小化后的MainWindowHandle托盘误选已以已选主HWND独立核验修正，不推断为产品缺陷。

真实30min退避观测prepare未启动成功：内部invoke为只读属性，透明记录器实际未安装，零计数不证明计划未执行。监测器terminal，schema99测试调度文件恢复原字节、auto-check恢复false；未加速时钟、未重启仍活产品或解除真实内部floor，也未计30min通过。失败原件/诊断归 `.adg/evidence/OCX-0110-20261012-WIN-REAL-BACKOFF/`，后续用可验证边界接续。

### 2026-10-12 真实树表键盘与更新缓存详情局部验证

同一afff普通/200%实例用真实Right/Enter/Space/Down/Left/Home输入及只读DOM核验，目录展开/文件焦点/父级返回/根导航通过；真实外部刷新aria-busy前后五级展开保留、焦点不被抢走。首次键盘序列受同时路由唤醒切走中断，工具失败原件保留；后续stable序列重建焦点，不混记。只覆盖现有无复选框备份树及记录场景。

真实设置入口查看共用管理器/面板详情与全部更新列表：无管理器候选与未知面板本地版本均禁用安装，说明缺失如实展示；未发起安装/下载，完整详情滚动和在途进度继续待验。见[键盘与更新详情证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows键盘树表与更新详情/README.md)。

新的计时器观测已独立验证安装，保持Tauri/实际时钟和延迟，保留旧floor。16:00:52生产错误分支再次安排完整30min后恢复原调度，200源唤醒及真实最小化/恢复未提前执行、最多一个有效timer；当前仅中间样本，待真实fire与cleanup终态，不提前通过30min或性能周期门禁。原TASK/脚本许可缺口、预算/性能failed和八项门禁保持，IMP in_progress，不关闭或发布。

### 2026-10-12 真实30分钟错误退避记录场景通过

上述中间观测现已完成：同一afff490普通令牌PID29936、同一隔离根，16:00:52.196错误分支锚点后首个真实回调为16:30:52.308（UTC+8），performance.now重算耗时1,800,001.5ms，无更早记录回调。79次周期观察、200个生成route/online源事件及真实最小化/恢复保持原下限，关联arm154次、fire1次、最多一个有效timer。生成online事件不抵物理重连；源归属由实际生产资源/调用栈确认，不以timer计数推断原始网络或native命令次数。

16:31:10.772清理完成，监测exec session31752 exit0；原调度字节SHA一致、auto-check恢复false、原计时器函数及Tauri桥身份不变，独立事后读取复核。未加速时钟、注入业务响应或重启产品。失败准备证据仍独立保留。详见[真实30分钟退避证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows真实30分钟退避/README.md)及原始目录 `.adg/evidence/OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF/`。

只通过本次真实timer错误下限场景；系统时钟变化矩阵、完整事件/通道、双权限/物理DPI、安装卸载/切根/更新、同机配对性能与真实周期仍未完整闭合。原TASK及脚本二次确认仍缺，既有性能failed/预算不变，IMP保持in_progress，无正式RCP/证书，不替换日常版或真实数据，不关闭或发布。

### 2026-10-12 原生标题栏拖动与顶边缩放局部验证

同一afff普通/DPI192主HWND852848，真实标题栏拖动使边界移动100×60物理像素、尺寸不变；真实顶边拖动使Top312→372、Bottom1903不变。独立只读原生助手核验所有者及DWM attr2/返回0。右下角未生效尝试、系统大小菜单未确认结果、窗口外恢复被工具拒绝均保留，不扩大通过范围。

一次截图仪器错帧已拒绝并移出公开证据，重选/激活候选后核验最终JPEG；见[标题栏与缩放证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows标题栏拖动与缩放/README.md)。仅本场景通过，其余原生/安装/更新/事件/配对性能/真实周期、原TASK和脚本二次确认继续开放，预算/既有性能failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 真实面板在途查询与管理器通道隔离局部验证

同一afff实例实际npm面板查询历时1754.8ms，stable→beta原生切换前后查询均仍在途，旧面板缓存保留，随后正常完成；管理器stable截止不变，面板成功更新自身24h截止。最终恢复stable/auto-check false，偏好SHA一致。只覆盖命令驱动的元数据查询，不抵在途安装/确认/重启矩阵。

临时隔离HOME的npm registry连接失败后，前端error=true、旧缓存不丢、failures=1退避写盘；真实管理器查询up_to_date，移除临时配置后真实面板重试成功并清前端错误、重置失败计数。生产panel-check事件policy=null/resolves=[]，没有持久失败行；不将本轮记为持久通知解除。见[在途通道与查询恢复证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows在途通道与查询失败恢复/README.md)。其余原门禁及TASK/脚本许可缺口保持，预算/既有性能failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 原生恢复事件同候选解除与轮换隔离

同一afff实例新建两个与当前canonical偏好字节一致的真实备份A/B，临时一字节破坏A后恢复被拒绝并生成持久异常；成功恢复B不解除它，返回A成功仍不解除旧世代，符合候选轮换不复用旧UUID的原契约。初始将返回A视为同候选的预期已纠正，失败/返回记录原件保留，不修改规则。

补做无其他候选插入的A失败→A成功：当前候选异常于17:05:15.520（UTC+8）真实解除，旧世代及原有异常resolved维度保持；失败/成功后event-scope相同。两轮活动偏好及修复后的载荷SHA一致，三个成功恢复的新事务保护均释放，原pin保留、manual策略/stable/auto-check false不变。见[恢复事件候选隔离证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows恢复事件候选隔离/README.md)。只覆盖该动作/阶段/通道及候选世代，完整事件及其余原门禁仍开放；TASK/脚本许可、原预算/既有性能failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 当前RDP物理缩放与独立环境权限限制

只读系统屏幕页明确“无法从远程会话更改显示器设置”，缩放控件禁用；未改设置或绕过限制，页面随后关闭。公开证据只保留去除账户栏的显示内容裁剪。Hyper-V功能已启用、服务Running，但当前普通令牌Get-VM访问被拒绝；不推断本机无VM，也未修改权限或操作既有VM。

见[原生环境限制](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows原生环境限制/README.md)。实际其余DPI、管理员、独立安装/升级卸载、配对性能及真实周期仍需原门禁证据，不豁免、不以zoom/VM存在代替；TASK原件/脚本二次确认仍缺，预算/既有性能failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 原生关闭隐藏、工具恢复限制与顺序受控重启

真实原生关闭按钮使原afff PID29936窗口visible=false/minimized=false且进程存活，app_foreground=false、Mica转inactive/attr1；WebView document.visibilityState实际仍visible，如实区分。没有运行代理，不计代理保持通过。Computer Use隐藏窗口绑定/激活分别返回not found/not usable，没有实际托盘恢复，不将工具限制记为产品故障或通过。

有意结束此测试会话，经原监督器受控清理并独立确认旧应用/监督器退出、进程数0后，才启动同根同EXE新普通PID41328/监督器44628/主HWND787306（exec91566）。偏好SHA及原pin、事件解除/旧异常状态跨启动保持，themeNeedsImport=false、auto-check false，日常EXE摘要未变。见[关闭隐藏与顺序重启证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows关闭隐藏与顺序重启/README.md)。不冒充同HWND恢复或正常退出验收；其余门禁/TASK/脚本确认缺口、原预算/既有性能failed保持，IMP in_progress，不关闭或发布。

### 2026-10-12 同机普通与管理员真实令牌及管理员备份回环

原普通PID41328实际false/type3，确认退出及候选数0后经标准RunAs启动同根afff管理员PID42372/监督器44052，应用真实elevated=true/type2。独立管理员只读助手打开同一进程句柄再次核验EXE/创建时间/令牌一致；只有一个应用。普通metrics身份门禁拒绝管理员路径null，未放松；管理员读主HWND381618848实际DPI192/visible/foreground=false，抓帧超时及AX只有标题栏不抵完整管理员UI/Mica通过。

管理员真实新建/固定备份、save light→restore system，DTO与偏好SHA恢复一致，无兜底保存替代；pin跨恢复保留、新事务保护释放，最后撤回临时pin，原pin/自动关闭保持。管理员只读Hyper-V查询成功返回0台VM，补充此前普通访问拒绝，未修改VM。管理员受控终态后才返回同根普通PID38924/监督器44780（exec58413），false/type3、源码匹配，无重叠。

见[双真实令牌与管理员备份](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows双真实令牌与管理员备份/README.md)。只通过令牌与该原生命令场景，管理员完整交互/其余DPI/安装更新/全事件/配对性能/真实周期继续开放；TASK/脚本确认、原预算/既有性能failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 Beta真实周期准备失败，stable/面板24h开始实际观察

17:57真实beta查询failed，未取得成功6h基线；恢复原stable/auto-check false/偏好原字节，原失败及退避保留。本机已认证GitHub API beta tag查询HTTP404，外部网页无可核验返回不作依据；没有发布/改端点/注入成功/缩短周期，beta6h仍未启动。

同一普通PID38924随后真实stable/面板初始化均成功，隔离根开启自动检查；源next_due分别为1791885667/1791885669，即2026-10-13 18:01:07/09（UTC+8），均为真实last_success+86400。当前RDP再次Disconnected/foreground=false，停止桌面输入，前台恢复与真实自动终态仍待验。

监测exec93170/PID18944已启动，并独立核验原应用/监督器及监测实际句柄存活；后续只读观察，不发手动检查/改截止/加速时钟/重启产品。连续记录在忽略运行目录，启动证据单独冻结，最终成功仍须审计；最长26h后尝试恢复原更新偏好。见[真实周期启动与接续](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows真实周期启动/README.md)。不提前记24h/性能通过，其余原门禁、TASK/脚本许可及原预算/既有性能failed保持，IMP in_progress，不关闭或发布。

### 2026-10-12 官方0.1.9基线准入与原预算复核

官方baseline019-v2 EXE SHA仍626d63ad…，来源de6862d4；未改基线或应用身份。对应旧platform::home_dir依然dirs::home_dir，启动取其HOME用于发现/共享来源；仅环境变量覆盖未证明完整隔离，因此未启动旧应用或将新main修复推定到旧EXE。

见[官方019基线准入复核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows官方019基线准入复核/README.md)。原冷/热各≥20、p95≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、>50ms主线程禁限及缓存100切页/10重启零自动查询预算保持；未测量、不清既有failed。当前真实周期实例/截止不变，原TASK/脚本确认及完整门禁保持，IMP in_progress，不关闭或发布。

### 2026-10-12 Windows查询进程树修复与同源红绿实际复验

main已集成推送4852820。原Windows查询仅终止直接子进程，实际生产Rust查询夹具证实分离后代可在超限/非零/成功后存活、stdout后代使超时收尾等待。修复以匿名不继承Job、暂停根启动→归属/线程核验→恢复执行，错误/结束清理自身Job；失败拒绝未归属运行，不误杀独立控制进程。生产30s/16KiB预算保持、准备耗时扣入总deadline，Unix进程组策略保留。

最终红2218b8与修复4852820测试blob完全相同。本机精确CI测试EXE最终旧0/4（超时需8.112s看门狗，失败）、修复4/4（超时约2.003s，未用看门狗/控制存活）；ZIP与EXE摘要独立匹配。feature CI38238993959全success，Windows34组598/0/7忽略；红CI38238999022 failure，主库498/4/4忽略。早期夹具仪器失败和自然退出影响原件保留，不以扩大超时、删断言或看门狗修复生产。

见[查询进程树证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows查询进程树清理/README.md)。main精确CI38242543274/候选38242585130仍待最终核验；本机唯一旧afff/PID38924的24h观测未替换，不据其旧生产/周期结果认证新SHA。完整门禁/同机性能/真实周期/TASK/脚本确认保持，原预算/既有failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 周期原窗口实际前台恢复，名义截止保持

19:27 RDP session2已重连Active，Computer Use激活原afff PID38924/HWND461032，production foreground/effects均true、DWM attr2。完整调度文件与初始化基线一致，lastCheckedAt/面板缓存未变，next_due仍为2026-10-13 18:01:07/09（UTC+8）；没有新应用、手动检查、改偏好/截止或时钟/timer。

见[周期窗口前台恢复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows周期窗口前台恢复/README.md)。只记录原窗口/截止实态，不据文件推断原始网络次数、系统睡眠恢复或24h通过；仍旧运行SHA，不抵新main原生验收。全部剩余门禁/TASK/脚本确认、原预算/既有failed保持，IMP in_progress，不关闭或发布。

### 2026-10-12 查询修复main同SHA CI与候选制品最终核验

main4852820 CI38242543274于19:48:41（UTC+8）completed/success，六job含实际macOS bundle均success；候选38242585130于19:46:52 completed/success，branch/main与SHA精确匹配。两个下载ZIP11744123025/11744028167摘要核对已认证API，MSI/NSIS与CI回执一致；只读MSI CAB提取EXE SHA7526561c…与CI安装marker EXE一致，产品0.1.10/Cargo.lock原字节一致。

候选仅CI fresh install/first-screen/uninstall smoke，实际token true/type1、mode any，不能抵本机双模式/默认data/升级卸载/OTA；未代码签名、不发布。包仍共享日常UpgradeCode，未本机执行安装器或新候选应用。唯一旧afff PID38924/监测18944实际句柄仍活、真实24h截止保持，日常摘要未变，main源码与运行旧SHA不混记。

见[集成CI与制品核验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows查询修复集成CI与制品/README.md)。只闭合同SHA CI/制品身份；新main本机原生、其余安装/更新/事件/配对性能/真实周期、TASK/脚本确认继续开放。预算/既有failed不变，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-12 查询Job生产Rust函数实际npm兼容性

用4852820精确CI测试EXE显式执行唯一ignored live官方元数据查询；独立HOME/AppData/cache/temp、清空后白名单环境，真实npm10.9.0/原字节Node执行，无响应注入或包生命周期脚本。20:07实际1/1、exit0、测试2.48s，生产函数版本/tag断言通过；不声称取得未输出的版本原文。

工具准备误用只读HOME变量、已存在证据目录处理失败均在测试启动前停止，原件保留、修正后才测试。独立核查测试根Node为0，原桌面PID38924/周期18944实际句柄与完整截止仍保持，不启动第二桌面应用。见[真实npm兼容性证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows查询Job真实npm兼容性/README.md)。只认证这个生产函数样本，不抵新候选WebView/完整矩阵/安装性能周期门禁，TASK/脚本确认与原预算/既有failed保持，IMP in_progress，不关闭或发布。

### 2026-10-12 查询函数同机普通/管理员实际测试进程令牌

同一4852820精确CI Rust EXE及测试blob，管理员PID2776实际true/type2、普通PID40324实际false/type3，各4/4/exit0；2秒超时分别2.0048149/2.0045155s返回，独立控制存活、无外部看门狗代替生产清理。实际测试进程和助手令牌分别读回，不以终端模式推断；前普通结果未含令牌字段保留，本轮补足，不追写历史。

管理员助手标准RunAs且只跑四项生产函数测试，未自动化认证/改安全设置。原唯一桌面38924仍普通、监测18944真实句柄仍活、周期文件原字节一致，不启动第二桌面或新候选UI。见[查询函数双真实令牌](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows查询函数双真实令牌/README.md)。只闭合该查询函数双模式场景，完整原生/安装/更新/事件/性能周期与TASK/脚本确认仍开放，预算/既有failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 官方019基线Profile实际边界核验

同机普通账户两个只读辅助进程调用SHGetKnownFolderPath(flags0/tokenNULL)：环境覆盖后Profile仍为原路径且在测试根外；Roaming/Local AppData则随环境进入测试根。准确记录区别，不将全部Known Folder视为不受环境影响；未改用户Profile/注册表或运行官方基线。

官方019 EXE SHA626d63ad…只读PE导入表确有shell32/SHGetKnownFolderPath；旧de6862d4源码HOME路径另有blob证据。dirs6/dirs-sys主源原始crate与当前锁摘要一致，仅作相关符号链核查，不反推旧无锁构建的依赖版本。见[019基线Known Folder边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows019基线KnownFolder边界/README.md)。原实例/周期/日常/基线字节保持，仍需实际隔离Profile的原身份基线环境后才做完整配对；不是性能结果，预算/既有failed及其余门禁/TASK/脚本确认保持，IMP in_progress，不关闭或发布。

### 2026-10-12 Windows版本探针进程树缺口修复与双令牌复验

main已集成推送eae8532。原版本探针kill_on_drop false/timeout丢弃future未收尾自有后代，成功/非零可留分离辅助进程、stdout后代使2秒超时等待看门狗。修复复用暂停启动/归属核验的Windows Job，返回前清理其自有树；启用直接子进程drop清理，准备耗时扣入总deadline，原固化2秒/输出字节/错误分类保持，元数据生产源码未改。

初次3d449ac版本本机0/3与PID空文件观察竞态分开保留；共享夹具改.pending→rename原子发布，两条最终线使用同一版本测试/元数据测试/夹具。红277f762本机版本0/3（超时8.1377929s看门狗，失败）/元数据4/4；修复普通false/type3版本3/3（2.0044058s）/元数据4/4，管理员实际true/type2版本3/3（2.0035329s）/元数据4/4，均无外部看门狗代替生产成功。精确CI ZIP/测试EXE摘要一致；修复feature CI38252093444全success，Windows34组601/0/7忽略；红CI38252159673主库502/3/4忽略。

main精确CI38254821701/候选38254885601已启动仍待最终核验。本机唯一旧afff PID38924/监测18944实际句柄保持、原周期文件字节及日常摘要未变，未启动新候选或冒充新SHA原生通过。见[版本探针进程树证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows版本探针进程树/README.md)。仅所记录函数场景通过，完整原生/安装更新/事件/配对性能/真实周期/TASK/脚本确认与原预算/既有failed保持，IMP in_progress，不关闭或发布。

### 2026-10-12 版本探针修复main精确CI与候选制品最终核验

main eae8532 CI38254821701于21:44:37（UTC+8）completed/success，六job含实际macOS bundle全部success；候选38254885601于21:32:49 completed/success，branch/main及SHA精确匹配。先前macOS在途观察保留，未提前通过或用feature skipped build抵main构建。

两个ZIP11750970498/11749763357原始摘要与已认证API一致，MSI/NSIS与CI回执匹配；只读MSI CAB提取EXE SHA f5e4a2ee…与CI实际安装marker EXE一致，产品0.1.10，Cargo.lock原字节与提交锁相同。CI fresh安装/首屏/卸载仅runner smoke，token true/type1/mode any；未代码签名，不抵本机双模式、data/custom根/升级卸载/OTA门禁。

新候选仅提取未启动/本机安装。独立实际句柄核验唯一旧afff PID38924/监测18944仍活，21:47已有45次周期样本/零观察错误，原周期文件字节及日常摘要不变；源码与旧运行SHA不混记。见[版本探针集成CI与制品证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows版本探针集成CI与制品/README.md)。只闭合同SHA CI/制品身份，其余完整原生/安装更新/事件/配对性能/真实周期及TASK/脚本/Profile环境缺口保持，原预算/既有failed不变，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-12 eae原目标与来源边界未闭合审计

按原用户目标14项逐项审计，overallResult=not_achieved；不将盘点项重建为缺失TASK的required_checks或关闭回执。最新eae精确CI/制品身份已证明，元数据/版本探针局部函数双令牌通过；旧afff原生/备份/树表/退避与当前函数证据不足以认证新eae桌面完整门禁。原指定main01a1d855/文档a83250ba均为当前祖先，共享五文件一致。

只读现态面板unresolved，原周期自动stable/true、autoPanel/launchWithCodex false、manual策略；runtimeActive根绑定不代表代理在运行。旧实例/新main四个调度blob相同仅作源码未变证明，不改标整份EXE/周期来源。49份完整观测前缀来源/PID一致、单调时间不降，截止仍为2026-10-13 18:01:07/09（UTC+8），尚未流逝24h，不据无文件变化推断零原始请求。

见[原目标eae未闭合审计](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows原目标eae未闭合审计/README.md)。完整原生/安装更新/全事件/同机配对性能/真实周期及TASK/脚本/Profile环境缺口保持；原全部预算和既有failed不变。IMP in_progress，不颁发证书、不晋升核心、不降低门禁、不提前关闭或发布。

### 2026-10-12 最终eae制品顺序切换与实际双令牌

22:43核验旧afff监测/应用仍活后明确纠正来源错位：旧EXE缺少后集成的查询/版本探针Job修复，继续等待不能认证最终制品。通过原stop.request有意结束旧观测，exec93170 exit0/cleanup恢复stable与auto-check false；56份旧原件保留、未满24h、未验收，不拼接新样本。此动作不由观察超时/丢句柄触发，旧过程历史不改写。

旧应用/监督器独立确认终态及应用数0后，复用原隔离根启动已核验eae EXE。普通45244/14136实际false/type3、主HWND854126前台DPI192/attr2；先退出再标准RunAs管理员40132/36156实际true/type2、独立提升只读助手复核主HWND985452前台DPI192/attr2。管理员AX仅标题栏、不算完整UI。再确认退出/数0返回普通37276/24300（exec80657），全程仅一个应用；未本机执行安装器，不计正常退出门禁。

最终制品主题/偏好生产重载未写原字节，原pin与候选世代通知状态跨启动保持、日常摘要不变。见[最终来源切换与双令牌](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终来源切换与双令牌/README.md)。仅已记录令牌/首屏/200%前台云母/持久回读通过，完整DPI/UI/安装更新/事件/性能周期/TASK/脚本及环境缺口保持，原预算/既有failed不变，IMP in_progress，不关闭或发布。

### 2026-10-12 最终eae源真实stable/面板24h重新建立基线

当前唯一最终eae普通PID37276/监督器24300（exec80657），真实初始化管理器stable up_to_date、面板npm远端2.83.0成功；启用自动检查后原生成功epoch1791816939/40，next_due1791903339/40，完整86400秒。当前准确名义截止为**2026-10-13 22:55:39/40（UTC+8）**；旧18:01属于已终止afff观测，旧未完成样本抵扣0，不拼接时间。

新监测3920/exec46484持有实际生命周期句柄，独立核验创建时间/存活，连续记录另在忽略目录cycles/eae-stable-panel-20261012。方法只更新源码/ID/目录，原26h观测与终态恢复原偏好规则保留；无deadline/timer/时钟/响应注入，监测不发手动查询。初始化前台实际false，不宣称持续前台，到期仍须真实可见唤醒/终态；不以未来截止或本次远端成功冒充面板安装/周期通过。

见[最终eae真实周期启动](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终eae真实周期启动/README.md)。14份冻结初始化证据只证明新源起点，stable/面板24h、beta6h、休眠跨启动/配对性能及其余原门禁仍未闭合；TASK/脚本/环境缺口、原预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-12 最终eae不同主题值备份命令恢复，真实周期保持

最终普通PID37276/RDP Disconnected下，只做真实WebView原生命令回环，不计鼠标/键盘或树表UI通过。当前system/stable/auto-check true/manual策略，真实新建并固定备份→保存light→restore system，DTO/原始活动偏好与snapshot SHA674ad619…一致，无兜底保存替代；保护payload实际light/SHA05ab9eb4…不同，transaction committed/protected false、refreshRequired/reconciliationPending false。pin跨恢复保留，最后撤回临时pin、原用户pin与库存旧异常状态保持。

三个阶段均stable/auto-check true，完整eae周期文件原字节不变，截止仍2026-10-13 22:55:39/40（UTC+8）；应用/监测实际句柄及唯一实例核验。见[最终eae备份命令回环](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终eae备份命令回环/README.md)。只认证本场景，不抵完整UI/安全轮换、全库存验签/事件或性能周期终态；全部原门禁/TASK/脚本/环境及预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-12 最终eae恢复异常同候选解除与其他动作隔离

同一eae普通PID37276在RDP断开时做真实原生命令场景：新测试snapshot一字节临时损坏→restore拒绝/新异常false；另一Backup真实成功仍不解除；finally修复原载荷字节后连续重试同一Restore候选成功，新异常于23:16:30.380（UTC+8）真实解除。活动偏好前/拒绝后/恢复后SHA相同、其他旧通知resolved及时间保持，原pin保留、新事务保护释放；无事件注入/手工解除。

event-scope整文件因Backup分区候选变化而不同，如实保留。Restore/local/execution分区UUID/指纹与object_id连续，指纹由实际snapshot ID重算匹配，不能用整文件等同要求误判动作隔离。最终源周期原字节/stable/auto-check true保持，名义截止仍2026-10-13 22:55:39/40，应用/监测句柄仍活。见[最终eae恢复事件解除](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终eae恢复事件解除/README.md)。只通过此动作/阶段/通道场景，不抵全事件/UI/安全矩阵或性能周期；原TASK/脚本/环境、全部预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-12 最终eae切根前提只读诊断

当前eae普通PID37276实际来源unresolved，状态刷新DTO为NotFound/Live、pid/port空/health unknown。精确源码确认：未解析真实来源返回Unreachable，collector投影NotFound但refresh仍Err；快照getter忽略该Err后组装Live，而切根入口单独要求本次refresh.is_ok()再验证idle。DTO/命令成功不能代替原门禁verified成功，也不放宽fresh状态。

本轮没有调用切根/迁移，准入结论按当前来源与精确函数路径推断；旧0a837实际runtime_unverified拒绝保留但不重新标eae结果。根配置/绑定文件、偏好与eae周期完整字节前后不变，应用/监测真实句柄仍活、日常摘要保持。见[最终eae切根准入诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终eae切根准入诊断/README.md)。只认证诊断，不计自定义根成功；真实来源/采集前提、TASK/脚本/安全环境及其余原门禁与预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-12 最终eae树表原生键盘、刷新与24行对齐

原普通PID37276/主HWND1051698在RDP Active时实际进入BackupFiles，鼠标展开月份后，以Right/Enter/Right/Space展开原用户固定备份，Right/Down进入文件，Left/Home返回父项/根。独立生产activeElement和5→22→24行回读对应真实输入；实际树外刷新busy true→false，五展开层级保持、焦点留在刷新。DOM仅观察，没有程序化导航/滚动/聚焦或刷新。

24个生产行用途/操作首行最大偏差0 CSSpx、按钮最小命中高度34 CSSpx；当前没有checkbox。Win32独立核验DPI192/foreground true/backdrop attr2、实际普通false/type3。首帧/AX延迟均再观察，最终新capture失败0x80070057后确认session2 Disconnected并停止UI输入；保留此前成功原帧，明确可能滞后且不是断开后当前截图。共享更新详情UI尚未执行。

见[最终eae树表原生证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261012-Windows最终eae树表原生键盘与刷新/README.md)。27份原件冻结/重验通过，偏好/根绑定/周期原字节保持、唯一应用/监测真实句柄仍活，真实截止仍2026-10-13 22:55:39/40（UTC+8）。只闭合该普通200%数据场景，不抵checkbox/密度/主题/完整DPI与管理员矩阵、更新详情/进度、配对性能或周期终态；TASK/脚本/环境、全部原门禁/预算及既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae备份保留、固定与成功后轮换

10月12日23:59:53起在原eae普通PID37276实际执行生产备份命令，手动新建3份后manual组20份均在D30内、预览空；仅把本轮3份manifest.created_at设置9月1日夹具，不改原有28条、载荷或系统时钟。实际固定1份后新预览仅另2份，陈旧预览拒绝/库存不变；撤回临时pin并取新预览，实际只删3份、原DTO库存一致。

另3份新历史夹具下，实际开启automatic未立即删；真实新备份成功才轮换这3份，保留新成功记录，finally恢复manual。原所有备份文件、偏好/根绑定/策略/调度SHA逐字节保持，原用户pin/保护DTO保持，最后只多一份真实成功备份。仅合该D30多于N10、历史pin排除、陈旧预览及成功后轮换场景，不抵全旧组N10固定计数、安全失败/关联矩阵、保留UI或真实流逝D30。

见[最终eae备份保留与轮换](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae备份保留与成功后轮换/README.md)。39份原件冻结/重验通过；生命周期包装误用外部退出码原件保留、按PowerShell成功状态复核，应用/监督器/周期监测实际句柄仍活，仅一个实例、日常摘要保持，截止仍2026-10-13 22:55:39/40（UTC+8）。其余原门禁、TASK/脚本/Profile环境缺口、全部预算及既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae全候选清理校验、失败关联保护与世代隔离

原eae普通PID37276在RDP断开时真实新建两份当前偏好备份，仅对新ID设置9月1日日期夹具、较旧载荷临时翻转1字节。原生预览正常在先/损坏在后；实际cleanup因第二候选SHA拒绝、第一候选也未删。实际Restore拒绝后新保护active/pinned false，关联源变为referenced-by-protection、预览排除二者；unpin及保护前旧预览实际拒绝，库存/活动偏好保持。

修复载荷后，实际成功恢复另一候选，再返回原记录成功，各新保护committed/无待核对；首次失败保护始终active。Restore/local/execution UUID首次f1a9a33e→另一候选7e89ecf4→返回原指纹a03c6895，指纹独立重算、object_id保持；旧异常始终未解除，原有通知解除维度保持、远端manager/panel分区未旋转。返回旧摘要不能复用旧UUID，不能将新世代成功当连续同候选解除；此前独立连续同候选解除证据保持。

见[最终eae关联保护与候选世代](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae关联保护与候选世代/README.md)。53份原件冻结/重验通过；校验器首次误期望store.error null、实际源码/原件为false，首错保留并改严格布尔校验，无原生重跑。两份夹具日期/载荷finally恢复、原所有备份与偏好/根绑定/策略/调度原字节保持，唯一应用/监督器/周期监测真实句柄与日常摘要保持，截止仍2026-10-13 22:55:39/40（UTC+8）。只闭合本局部原生命令/世代场景，完整备份安全/UI/事件通道及安装在途、性能/真实周期与TASK/脚本/Profile缺口、全部原预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae全旧记录固定项不占最近N10

原eae普通PID37276实际暂固定16条原普通测试记录，使原同组记录不占N10；原用户pin/关联保护不改。真实新建14份、D30内预览空；仅新ID日期置9月1日夹具后最旧4份为候选。实际固定夹具最新一份后候选降为3，旧预览拒绝，取新预览只删新3份，保留10普通+1最新pin。该全旧场景补足此前局部记录未覆盖的固定不占N10计数，不使用近期日期兜底。

finally恢复存活新夹具日期、原生命令撤回最新pin和原16个临时pin；原34条DTO/所有原文件/原manifest字节及存活新11份原件完全保持，未删除原库存。原偏好/根绑定/策略/调度字节、唯一应用/监督器/监测实际句柄和日常摘要保持，截止仍2026-10-13 22:55:39/40（UTC+8）。

见[最终eae固定项不占N10](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae固定项不占N10/README.md)。31份原件冻结/重验通过；00:33曾RDP Active但激活窗口前又断开、GetCursorPos 0x80070005，新capture/更新入口均未到，停止UI输入，详情仍待验。只闭合此N10原生命令场景，真实流逝D30/全备份安全/UI、其余原门禁/TASK/脚本/Profile与性能周期、全部预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae后台设置页真实十分钟资源样本

原eae普通PID37276/200%DPI，system/high/mesh/100%界面缩放、当前backup设置页始终后台。00:58:30.992–01:08:31.259（UTC+8）只读采样器36408/exec25560完成601样本/600.2127483秒、实际exit0；7个已观察应用/WebView2句柄身份保持，累计CPU0.515625s，平均单核0.085907%，20核归一化0.004295%仅补充不替换原预算口径。进程族工作集算术和平均625.630960/峰626.878906MiB，根平均42.668333MiB；共享页可能重复，不是去重物理内存，不据逐秒快照推定全部短命进程或零查询/网络。

官方019原EXE SHA626d63ad…保持，旧入口仍使用系统home发现/工作目录，Profile隔离未证，未启动基线；此后台设置页绝对样本不抵同机配对、前台概览/档位矩阵，不与历史macOS直接相减。a832精确TASK182路径及all-refs历史无blob，work树仅11个跟踪对象，忽略规则确认、未从RCP/digest重建TASK；原性能RCP仍failed及全部原阈值保持。

见[最终eae后台资源原件](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae后台闲置资源样本/README.md)。23份原件冻结/独立重算通过；采样句柄首查日期格式精度错误保留，原FILETIME/路径/参数与live handle确认同一进程，无重启。原偏好/根绑定/策略/调度与日常摘要保持，唯一应用/监督器/24h监测实际句柄仍活、截止仍2026-10-13 22:55:39/40（UTC+8）。只认证本观测，配对性能/其余完整原门禁、TASK/脚本/安全环境和真实周期继续未闭合，预算/既有failed不变，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae策略准入失败不轮换与同候选解除

原eae普通PID37276实际新建3份、仅新ID设9月1日日期夹具，真实开启automatic不立即删除。在同一协作锁下仅将隔离backup-cleanup-policy schema1暂设99；实际create和policy读取均拒绝unsupported schema，库存/候选完全保持、无新记录/轮换，不静默回落manual。finally原字节修复automatic后真实同载荷备份成功才轮换新3份，保留新成功记录、原45条DTO/原文件字节一致，最后原生命令恢复manual及原策略字节。

新Backup/local/execution失败resolved false→于01:15:50.373383（UTC+8）真实成功解除；失败/成功UUID9aa90412…及指纹相同，按manual-preferences+NUL+原始载荷独立重算匹配，原有通知解除维度及远端manager/panel分区保持，无手工解除/事件注入。只认证策略准入失败和本连续同候选场景，不覆盖磁盘满/中断/清理自身失败、全失败/UI或通道/在途矩阵。

见[最终eae失败备份不轮换](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae失败备份不轮换/README.md)。57份原件冻结/重验通过，原偏好/根绑定/策略/调度字节、唯一应用/监督器/24h监测实际句柄和日常摘要保持，截止仍2026-10-13 22:55:39/40（UTC+8）。其余完整原门禁、官方019配对/真实周期、TASK/脚本/Profile环境缺口、全部预算及既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae六个原生命令跨进程协作备份锁

原普通PID37276实际执行create/list/pin/policy/restore/cleanup，由各自独立普通真实false/type3助手持有同一Windows备份锁。每命令两次实际pending间隔1161.6–1229.0ms，开始时间一致/未完成，持锁期间备份载荷/manifest及偏好/根绑定/策略/调度字节不变；明确release后自然解锁、真实命令成功，六助手/调用进程原句柄终态exit0、控制器45304/exec74199实际exit0，不靠终止助手或产品制造结果。

Restore实际light→原system，保护载荷light/不同SHA、committed且关联本轮源，活动偏好原字节保持，无fallback保存。Cleanup只对本轮3份新ID历史日期夹具，真实预览/删除恰3份；最后原生撤回源pin、store system/error false。原46条DTO/所有原备份文件及偏好/根绑定/策略/调度原字节保持，最终只新增本轮源/create/restore-protection共3条。

见[最终eae原生跨进程锁](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae原生命令跨进程备份锁/README.md)。131份原件冻结/重验通过；事件元数据不在“持锁文件不变”声明范围，RDP断开不算UI。管理员/非协作写者/异常终止及完整备份门禁未验，原唯一应用/监督器/24h监测实际句柄与日常摘要保持、截止仍2026-10-13 22:55:39/40（UTC+8）。全部其余原门禁、配对性能/真实周期、TASK/脚本/Profile环境缺口和预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 最终eae固定beta来源重新准入仍404

01:43:56–01:44:00（UTC+8）核对远端main仍eae8532、精确runtime.defaults Git blob/固定beta端点；已认证同仓库release tag beta API404，独立无认证固定公开下载端点也404，原始body/时间/状态/摘要保留，无凭据落盘。与10月12日17:59:10来源观察相隔27885.983543秒；只是外部来源检查间隔，不能当beta真实6h，成功初始化/真实周期仍未启动/未验。

见[最终eae测试通道源重新准入](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows最终eae测试通道源重新准入/README.md)。11份原件冻结/重验通过，仅外部源可用性结论，未调用产品check/切通道、创建发布/清单或注入响应；原唯一应用/监督器/24h监测真实句柄、周期/偏好字节保持，截止仍2026-10-13 22:55:39/40（UTC+8）。原全部beta/周期与完整门禁、官方019配对、TASK/脚本/安全环境缺口及预算/既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 剩余门禁的Profile与安装隔离方案待准入

原eae唯一37276/监督器24300/24h监测3920真实句柄/创建时间仍活，原截止保持，冲突的配对/来源切换/错误夹具在周期终态后顺序执行。只读重算最终MSI17f70d35…及日常注册/EXE43e06bc3…保持；原MSI ALLUSERS1且共享UpgradeCode，因此独立Profile可解决019真实HOME隔离，但不能隔离本机机器级安装升级。实际安装/升级卸载仍需无日常注册的单独准入Windows环境，不能以改目录/账户或CI smoke替代。

候选普通测试账户ocx0110_accept与目录D:\OpenCodex-Acceptance\0.1.10-20261013当前不存在，仅提出供用户决定/提供等效环境；未创建账户/Profile/VM、改ACL/权限或收集密码。磁盘有空闲、vmms Running只证明本次只读状态，不证明有可用VM/授权，物理DPI及同物理机配对门禁不变。两份原EXE身份及原冷/热≥20、p95双条件、闲置/帧/缓存/请求进程等所有预算逐项与原件核对，历史failed不清。

见[剩余门禁执行前提](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows剩余门禁执行前提/README.md)。9份原件/计划一致性结果consistent，仅可审阅方案，不是执行回执或替代TASK。原TASK182、FZ50显式许可、稳定桌面、隔离环境、真实beta源及全部未完门禁/性能周期继续保留，截止仍2026-10-13 22:55:39/40（UTC+8）；预算/既有failed不变，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 早晨原窗口恢复与用户触发的前台窗口

原eae普通37276/主HWND1051698恢复前台，实际DPI192/Mica attr2，目标帧保留；版本入口AX45在视口外，实际click越界拒绝，未进入升级页/详情，不当产品缺陷或通过。再观察的非目标前台帧未归档为产品证据，随后恢复原BackupFiles；用户要求不影响工作后停止激活/输入，仅后台只读、缓存/偏好/根绑定/策略/调度字节保持。

见[早晨恢复与前台协作安排](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows早晨窗口恢复与前台协作安排/README.md)。冻结115份真实周期完整前缀/9.515984h，源/PID/EXE、单调时间/偏好/调度保持；原应用/监督器/监测真实句柄仍活，前缀不抵24h终态或零请求计数。12:30–12:50及22:50–23:10等均为建议窗口，须用户发开始信号才前台，时钟/自动续行不授权抢焦点；原截止22:55:39/40及26h上限保持。完整目标及全部原门禁/TASK/脚本/环境/性能失败仍开放，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 原周期后台终态审计工具，当前严格待验

只读审计工具现场核验原37276/24300/3920句柄及精确FILETIME，读取既有初始化/完整日志前缀，不invoke/抢前台或改timer。验证源/EXE/偏好/单调时间/无关beta分区；新成功须不早于原due、成功观察处经过完整86400000ms且新截止差86400秒、真实缓存结果可读。即便两个新成功也只返回需终态/范围审阅，不推断最早请求起点、自动触发或零网络次数，不替代owner真实终态/cleanup或完整周期矩阵。

见[真实周期终态审计工具](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows真实周期终态审计工具/README.md)。08:38:31（UTC+8）实际117份原记录输出pending_original_deadlines、三个原句柄仍活；三项错误源/倒退/提前成功内存工具夹具被拒绝，仅校验器测试，不注入应用或计产品通过。原期限、26h上限、前台用户触发及全部门禁/预算/既有failed保持，TASK/脚本/环境/真实beta源仍缺，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 中午缓存详情、原生标题栏与当前39行树表

原eae普通37276/主HWND1051698在12:42起实际前台验证，最后输入12:49:55.111，12:50后不再输入。真实设置缓存管理器/面板详情及全部对象视图：0.1.10/可用—、备份默认勾选，本地面板未知/远端缓存2.83.0/已有错误，安装均禁用；Shift+Tab/Tab循环及Escape归还管理器入口通过，没有点击check/install/restart/外链。短缺省说明不抵长说明滚动或真实进度。返回全部视图时发现BODY焦点外逸，原件保留、不计全焦点门禁通过。

Win32确认真实最大化/DPI192/attr2；后续意图还原却实际最小化、捕获失败，原因未隔离、不计还原通过，重新选择激活原HWND恢复。13×13最大化相关小帧保留但不当完整窗口图。实际重新挂载BackupFiles并展开月份/manual组，39行用途/操作首行偏差0、按钮至少34CSSpx、无checkbox，不抵完整矩阵。见[中午真机记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows中午缓存详情与树表验证/README.md)。55份原件冻结/重验通过，原偏好/绑定/策略/调度字节及唯一应用/监督器/监测实际句柄、日常摘要保持，22:55:39/40原周期不变。全部未完门禁/TASK/脚本/环境/预算/failed保持，IMP in_progress，不关闭或发布。

### 2026-10-13 更新详情切换焦点缺口特性修复，原周期不替换

代码已提交推送feature/0.1.10-update-center-focus@d975b23：切换渲染后仅在仍开弹窗且焦点外逸时恢复同名控件或dialog，不移动仍在内的有效焦点，关闭后不抢焦点。最终同一测试blob红3失败/8通过、绿11通过；全前端95文件/483项、类型检查及构建通过，首轮红与既有构建警告原件保持。只改UpdateCenter及对应回归，调度未改。

见[更新详情焦点修复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows更新详情焦点修复/README.md)。24份原件冻结重验；CI38349316806在13:02观察in_progress，未计完整CI/修复制品原生验收，尚未集成main。当前main/运行实例仍eae，原周期不热替换/重启，不能以修复源码证明旧EXE已修好。后续完整CI/按归属集成及同SHA原生复验保持，完整目标与原门禁/预算/failed仍开放，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 焦点特性精确CI失败，本机对照未隔离原因

CI38349316806/d975b23在13:12:56（北京时间）核验completed/failure，原在途观察保持。前端/Linux/发布工具/令牌success，Windows前端构建/fmt/Clippy success，Windows主库502/3failed/4ignored；三个version success/nonzero/timeout实际2.17/2.17/2.15s返回Timeout，observer未取得后代启动/保留句柄，release EXE与总build跳过。元数据四项通过，当前不足以定位故障，未集成main或重跑求绿。

精确失败run ZIP11790820992 SHA8da4ce67…匹配API，测试EXE e5aaed4c…匹配CI摘要；本机普通实际false/type3、白名单隔离环境中，Node24.19串行/7线程各9/9；官方摘要核验的精确Node24.21同样两组9/9。每组7个进程树+2路径、500filtered，不抵CI全库并发或通过。三后端Git blob与eae一致仅作无修改证明，未据此称无回归；生产2秒/夹具/并发/断言均不改，原因仍未隔离。

见[焦点分支CI失败与诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows焦点分支CI失败与隔离诊断/README.md)。31份原件冻结重验，仅归档与诊断结果，CI/fix原生未通过；原eae唯一实例/监督器/监测真实句柄及偏好/根绑定/策略/调度字节、日常摘要保持，截止仍22:55:39/40。本机Node隔离不替换日常，不抢前台/重启周期。完整目标/门禁/配对/真实周期/TASK/脚本/环境/全部预算与既有failed保持，IMP in_progress，不降低门禁、不关闭或发布。

### 2026-10-13 完整主库隔离对照与测试期阶段诊断

原失败run的精确测试EXE/Node24.21在本机白名单沙箱完成默认线程/显式4线程两组完整509项：均502passed/3failed/4ignored，原CI三个version场景均通过；本机失败另为内嵌CI文档路径不存在、两个日常凭据服务名夹具与沙箱验证不符。原件/exit101保持，不改路径、关闭沙箱或删断言求绿，不将相同计数混为相同失败，仍未隔离原CI原因。

feature焦点分支已推送be7395a，增加四个Windows测试期版本阶段耗时、夹具失败时PID/ready文件状态及CI宿主设置读回；原2秒/4秒/5秒预算、生产收尾、fixture脚本、cargo命令和CI并发不变。源码cfg去除等价检查与PS语法通过，不抵本机Rust编译；新精确CI38352525092在13:41:02观察in_progress，尚未集成main或计修复制品原生通过。

见[全库隔离对照与阶段诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows全库隔离对照与探针阶段诊断/README.md)。21份原件冻结重验；原eae唯一实例/三保留句柄及四项数据摘要、日常EXE保持，真实178份观察/0错误不抵终态，截止仍22:55:39/40。12:50后后台，无窗口输入；下一前台须新用户信号。原TASK/脚本/环境、全部未完门禁/配对/周期/预算及历史failed保持，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 诊断特性精确CI成功，制品与原故障范围保持

be7395a的push CI38352525092在14:00:39（北京时间）认证复核success，五必需job与Windowsfmt/Clippy/全回归/release/最终EXE smoke均success，feature上的main-only build保留skipped。主库505/0/4ignored，34条cargo目标合计601/0/7ignored；三个旧失败version场景通过。新宿主4逻辑CPU/Node24.21/测试线程变量未设，成功捕获日志隐藏阶段stderr，原失败原因仍未隔离，不覆盖d975原CI及本机全库failed。

ZIP11792440963 SHA3fae1e04…匹配API/CRC，165项与11份原件核验；测试EXE bcb9d346…匹配摘要，最终桌面8651ba83…只为同源smoke回执，包内无最终桌面EXE/安装器。CI实际elevated/type1但mode any，HOME移除/沙箱/overview通过；version端点超时诊断保持，不抵本机双令牌/物理UI/焦点修复。未候选打包、集成main或替换原实例。

见[诊断CI终态与制品范围](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows诊断分支CI终态与制品范围/README.md)。精确测试EXE本机普通7线程9/9，12条四阶段日志真实可读，原2秒预算及无外层cleanup保持；41份归档重验。原eae三保留句柄/唯一应用/四项字节/日常摘要、182份周期前缀及22:55:39/40保持。前台须新人的信号；TASK/脚本/安全环境/原门禁配对周期预算历史failed仍开放，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 完整CI后同SHA候选与本机只读提取

复核be7395a精确CI及五必需job success、远端feature前后SHA不变且无既有同源候选后，14:16:40只dispatch既有候选流程一次；实际run38355803684同SHA于14:30:02success。包锁全回归主库505/0/4ignored、34目标合计601/0/7ignored，NSIS/MSI构建及隔离runner逐个安装/首屏/卸载success，原失败cause仍未隔离，不清旧failed或集成main。

ZIP11792974377/11792959449摘要747fb141…/2e8639d5…匹配API/CRC，20份原件/两个一致回执保持。MSI/NSIS摘要6a500186…/09b2526c…，本机模式0只读MSI与cab导出EXE3a26098e…严格匹配runner实际启动文件，完整be7395a号存在；仅内存归一化正确补丁位置后整文件hash与本轮built8e2e9b3e…匹配，未改EXE字节。候选未本机安装/启动；ALLUSERS1/共享UpgradeCode及NotSigned保持，runner真实type1/mode any与overview不抵本机双令牌/物理UI/数据保留或焦点门禁。

见[同SHA候选与只读提取](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows焦点修复同SHA候选与只读提取/README.md)。60份全集合字节冻结重验，纠正检查器误判标记唯一及原仅PATH工具搜索范围；既有隔离Rustfmt本次两改动文件只读检查exit0，不抵本机完整Cargo。原eae三保留句柄/唯一桌面/四项字节/日常摘要、188份周期快照及22:55:39/40保持，不前台/替换或拼周期。候选用于原周期终态/cleanup后的人触发复验，TASK/脚本/安全环境、全部原门禁/配对/周期预算/历史failed仍开放，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 原周期到候选的只读顺序审计

新增只读审计工具复用摘要绑定的原周期校验，要求监测真实成功终态/finished、native-cleanup与原偏好pair/磁盘一致、监督器自身收尾及三原句柄退出、全机独立desktop=0。访问拒绝不能当退出；条件齐也仅返回需operator审阅，不启动/停止或授权焦点，不抵拥有者工具终态/完整周期门禁。14项内存工具夹具覆盖阻断，仅工具证据，不注入产品。

14:53:57/14:58:18实际192/193份前缀检查返回pending，原37276/24300/3920精确句柄仍活、独立全机唯一实例、auto=true/监测在途/无cleanup是当前预期，原22:55:39/40与26h上限不改。后续用新外部目录防覆盖冻结证据，原eae周期不接新be7395a源码拼为最终源证明。

见[原周期到候选的只读审计](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows原周期到候选切换的只读审计/README.md)。27份原件冻结重验；Get-VM普通权限拒绝/CIM只见宿主，不推断VM不存在或隔离就绪，测试Profile未建立，TASK182/显式脚本许可仍缺。没有前台、stop/start或权限/环境改变。全部原门禁/最终源真实周期/配对预算及历史failed保持，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 焦点草稿PR与中断后描述更正

PR #1为main/eae8532←feature/be7395a的5文件代码提案，open/draft、未合并且无auto_merge，已附加本任务。连接器403未创建后，本机认证API复核无重复再创建；凭据内存使用不存盘。16:05:19同head/base读回，PR CI38361436746与push38352525092均success，不混同PR合并检出二进制与单独候选字节。

16:34:21只更新描述为旧三进程已缺失、201份旧记录/原截止保持、候选未启动的实际范围，旧创建描述保留；真实跨启动验证仍待执行，未请求ready/合并/发布。见[焦点草稿PR与中断后说明](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows焦点草稿PR与中断后说明/README.md)。14份归档重验通过仅为PR事实，原门禁/预算/历史failed/TASK脚本环境仍开放，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 旧拥有者缺失、原截止保留与跨启动接续更正

独立全机desktop=0、旧37276/24300/3920不存在，陈旧running状态最后句柄15:39:58/原生15:37:48不作退出时刻或原因；201份旧完整前缀冻结，无cleanup/stop.request/拥有者终态日志。限定Application产品崩溃事件查询未取得匹配项，不足解释原因。原四项数据字节/日常摘要不变，stable/panel真实初始成功→next_due仍86400秒，原截止22:55:39/40保持。

原第5节第5项为真实24h/6h到期窗口与跨启动，并未要求同一进程连续24h；更正此前退出必然完整重计的过度判断，不修改条款/预算或把定时记录当通过。旧连续观察中断保持未验；可在新进程分别取证原持久截止的跨启动真实自动行为，三份调度源码blob eae→be7395a一致只是无修改证明。旧成功交接工具不改为伪造cleanup准入，中断恢复另留范围。

见[实例中断与原截止跨启动接续](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows实例中断与原截止跨启动接续/README.md)。15份冻结重验、只读观测JS语法通过，候选未启动；人触发窗口前不抢前台，后续保持原根/截止，不initialize或手动查询/改timer，新PID/源/时间与旧证据分开。自动终态/原始计数及所有剩余门禁/真实beta/30min/配对原预算/历史failed/TASK脚本环境仍待验，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 晚间原截止跨启动success读回，断连焦点待验

用户到期授权前台后核对RDP Active/全机desktop0/旧拥有者缺失与原根截止字节、同源候选身份，22:57:31.5757733启动唯一be7395a EXE3a26098e…普通false/type3 app10904/监督器32492/exec42581/HWND6685560。无手动check/install/通道根切换或时间修改，22:58:30cached读回manager成功22:58:23.4866704/panel epoch22:58:26最新2.84.0，两原截止后新last_success与next_due差86400、beta分区不变。该真实跨启动持久结果接受，不抵连续旧周期/最早请求起点/直接自动trigger或原始零重复计数；只读foreground也会同步活动可能唤醒调度，范围声明保持。

实际打开设置版本/管理器详情、备份勾选/安装禁用，焦点在dialog；返回全部关键动作尚未执行。Tab激活失败、重选捕获0x80070057，23:01:56 RDP Disc后停止输入；不计原生焦点fix通过。实际DPI192/DWMattr1/正常窗口且观测foreground false不抵100/125/150或Mica2矩阵。23:04:21新观察空闲收尾恢复原stable/auto=false、成功schedule原字节保持，应用仍保留，不伪造旧cleanup或启动下一自动周期。

见[原截止跨启动结果与断连焦点待验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows原截止跨启动结果与断连焦点待验/README.md)。45份全集合/字节冻结重验，新两保留句柄/唯一桌面/日常摘要保持；23:10后不前台，恢复需新的人的信号。原beta/30min/完整周期原始计数、配对预算、双令牌/物理UI/安装数据/更新备份树表事件、TASK脚本环境/旧failed仍开放，PR草稿main未集成，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

### 2026-10-13 重连接后的焦点真机复验子集通过

用户恢复连接后query Active，复用唯一be7395a/普通false-type3 app10904/监督器32492/HWND6685560/EXE3a26098e…；窗口到23:30、最后输入23:27:46.307。真实Tab/ShiftTab聚焦管理器/面板“全部更新”，Enter返回后dialog焦点保持，ShiftTab末项/Tab循环关闭与Escape归还原入口通过。只读DOM保存实际activeElement，不拿AX宿主pane代替；无check/install/重启/外链、无CDP焦点路由注入。长说明/真实进度仍未验。

普通DPI192/200%重连接后实际DWMattr2/foreground true、最大化true通过；caption还原click窗口仍最大化/边界不变，原因未隔离不计通过。AltSpace实际系统菜单选还原再Return，正常边界恢复/非最小化/attr2保持，仅计菜单还原，不清中午旧eae异常。Snap/托盘/退出/管理员/100125150物理矩阵继续待验。

见[焦点真机复验与标题栏局部结果](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261013-Windows焦点真机复验与标题栏局部结果/README.md)。48份字节冻结重验；截图JPEG字节误存png名及13×13辅助帧明确标识，原件不改/不重跑。首末四项数据字节/成功调度/auto=false及日常摘要、同两真实句柄保持。焦点子集接受不抵完整门禁；原CI/性能failed、TASK脚本环境、全更新备份树表事件/安装/配对预算/真实周期仍开放，PR草稿main未集成，IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。

23:37:31 PR #1标题移除已完成焦点子集的“待Windows原生复验”括注，正文引用5dee74b实际证据与caption等保留范围；只写title/body，head/base及open/draft/merged=false/无auto_merge保持，原API历史不改。`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-PR-NATIVE-WRITEBACK/`保留前后读回和原文，不请求合并、关闭或发布，原完整门禁与预算仍开放。

### 2026-10-14 Mac 后台接续与原 TASK 同步

远端 main@eae85322、feature/0.1.10-update-center-focus@be7395a、docs/governance-main@6bd9547c 已核验，main / 文档工作区快进同步，既有未提交文件摘要和删除状态保留。PR #1 仍 open / draft、未合并；焦点修复尚未进入 main。

Mac 找到本地原 TASK-182：3878 字节 / SHA256 bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93，2026-10-10 创建 / 修改，内容校验通过且与既有回执的 content / summary / baseline 绑定一致。因 .adg/work 被忽略，此前未 Git 跟踪；本轮按文档归属同步原件并加单文件 -text，未依据回执重建 / 修改 TASK。revision2、in_progress / pending、八项 required_checks 及空验收保持。[来源与分工记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Mac接续与原TASK同步/README.md)。

用户重新授权目标执行：Mac 源码审阅 / 定位 / 修复与后台回归，Windows 固定 SHA 真机验收。Windows be7395a / PID10904 为交接快照；Mac 暂不启动桌面应用或原生性能测试，切换原生平台须先协调关闭 Windows 实例并确认 0。先查 caption 还原与既有 macOS 同机配对失败，保留原始失败和原预算；FZ50 脚本未获许可不执行。不得以旧制品证明新源码，不自动合并 PR、签名、发布、晋升稳定通道或生成完成证书。

### 2026-10-14 Mac 焦点源码审阅、只读 caption 工具与旧性能重算

PR #1 仍 be7395a→eae85322、open/draft未合并。五文件有界审阅未发现阻止继续真机复验的补丁问题，Windows测试诊断不改变2s/4s/5s预算；不计全更新审阅通过，旧CI根因待查。精确be源码的更新焦点 / 动画 / 光层六文件52项回归通过；旧Windows原生焦点证据48份摘要重验通过。

caption还原原件缺实际点击坐标与命中，尚不能判产品缺陷。独立feature/0.1.10-caption-diagnostics@04c5f54d新增显式PID/HWND/EXE身份绑定的只读按钮几何与WM_NCHITTEST工具，12项便携契约通过并接入CI；没有生成输入、启动/结束应用或改产品运行代码，Windows API实际运行待验。CI38429545855最后in_progress；不触发新候选、自动合并或签名。Windows可在现有be实例采集，结果仅归be实际制品；生产修复后须另交新SHA真机复验。

旧macOS十分钟归档独立重算与原comparison逐字节一致，CPU +1.817629百分点、平均RSS +117.008965MiB失败保持。候选WebContent RSS约104→309MiB、主进程约119MiB稳定；Logo/SVG scene.ts与官方0.1.9源码一致，新增光层布局与合成范围为受控实验线索，非因果结论。原顺序单组/准备态/无连续前台限制保留；Mac本轮无桌面启动或新原生样本。需协调Windows实例0后才可切Mac原生，按单变量定位再完整配对，不能撤销顶部/左右覆盖、降画质或预算求绿。[审阅与诊断证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Mac源码审阅与标题栏诊断/README.md)。

原TASK revision2保持，八项门禁未闭合、performance failed、acceptance_results为空；原启动p95/冷热各20/10min CPU与RSS/50ms/缓存零请求/真实周期预算不变。FZ50未获许可不执行；不迁移真实数据、替换日常安装、晋升核心事实或通道、创建关闭证书或发布。

### 2026-10-14 Windows 分工、只读 caption 与原 TASK 独立复核

Windows 接续固定 be7395a 制品验收，Mac 协调源码修复；全局最多一个隔离原生实例，当前仅后台授权。PID10904/HWND6685560 只读身份与真实普通令牌复核，正常窗口 DPI192，按钮命中2/9/20。原测试最小化允许2/8，局部诊断仅期待8的差异另记纠正；不覆盖原件、不计产品失败或点击通过。caption还原仍待下一前台联合坐标复验，未定位根因。

Windows 已取得 ae6cd08 同步的原TASK182并用治理provider独立校验，3878字节/SHA bc7965b4…保持，content/summary/baseline与既有回执一致；原缺失阻碍已解除。revision2、in_progress/pending、九项验收绑定、八项required_checks及历史performance failed不变。

两批冻结证据12/13份清单校验通过。用户要求修改本线程目标；标准工具仍读paused，官方thread/goal/set被当前active writer拒绝。已启动有界后台等待器，等待本轮writer释放后正式写入并读回；当前冻结快照只证明waiting/live，不能宣称目标已恢复。没有绕锁、改数据库、清除/重建目标或标complete。[分工、原件与写入边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows分工与原TASK复核/README.md)。其余原门禁、预算、日常安装/真实数据保护保持，未关闭或发布。

### 2026-10-14 caption 坐标归属补充与旧工具完整 CI 终态

Windows文档b3eb4274已快进同步，正常窗口只读2/9/20与既有合同一致，不计最大化还原点击通过。Mac补充诊断feature/0.1.10-caption-diagnostics@5a36cda0：主窗口HTMAXBUTTON与WindowFromPoint/GA_ROOT并列，避免把主窗口查询当实际鼠标目标；无关窗口身份不落盘，不发送输入/改应用生命周期。同新增测试文件，旧04工具11通过/1失败/4错误，新16/16通过；均为fake工具合同，非产品原生故障复现。Tao0.37.1/Wry2.12.1审阅源与锁定crate归档摘要验证通过，不证明真实派发或还原根因。

旧04 CI38429545855完整success，五必需job成功、main-only build skipped；MacRust872passed/6ignored、Windows601passed/7ignored，均34组/0failed，ignored不计通过。新5a CI38431616359最后in_progress，不能沿用旧CI证明新工具。主main/eae与焦点be草稿未改变，未制作候选或自动合并/签名。两提交只改test/CI；现有be可供诊断，工具与实际制品身份分别记录。[坐标归属及CI证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-标题栏坐标归属补充/README.md)。

Mac继续后台工作，无新原生性能样本，切原生平台前仍须协调Windows实例0。原性能failed、全部预算/门禁及原TASK revision2/in_progress/pending/空验收保持；FZ50脚本未获许可不执行，无真实迁移、日常替换、核心事实晋升、关闭证书或发布。

### 2026-10-14 最新坐标归属工具完整 CI 通过

feature/0.1.10-caption-diagnostics@5a36cda0 的 CI38431616359 已completed/success，五必需job success、main-only build skipped；CI实际运行16项工具合同，Ran16/OK。本次只改test/CI，旧进行中及红绿原件保留，终态另归档。未在be真机执行新API或点击，不制作/安装候选，不用CI替代还原或原macOS配对性能；原门禁/预算、performance failed及TASK空验收保持。[同SHA终态证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-标题栏坐标归属补充/CI-终态.md)。

### 2026-10-14 Windows 后台完成精确5a CI与坐标归属观察

固定5a36cda工具的CI38431616359现completed/success，五必需job成功、main-only build skipped；两证据ZIP825/9218509字节的API摘要与实际SHA256、ZIP CRC核验一致，七份选定JSON原字节归档。第二包初次TLS EOF及已有包保护中断如实记录，只补缺失包，未关闭验证或执行包内文件。CI startup为any/elevated=true/type1，EXE不同于本机；不抵本机双令牌或安装候选。5a基于main/eae且不含be焦点修复，产品身份分别记录。

00:50:34 Windows实际PID10904/HWND6685560及完整creation/EXE摘要保持，普通false/type3、正常窗口DPI192/foreground=false/DWM1。本机16项便携合同通过，九物理坐标父窗口命中按按钮为2/2/8、8/9/9、20/20/20；WindowFromPoint/GA_ROOT九点均不归目标窗口/PID，当前遮挡状态不证明真实点击送达，也不定位旧caption还原根因。没有输入、前台激活、实例切换或安装，日常与隔离保护摘要前后一致、本机仅一个隔离实例，auto=false。

见[Windows坐标与精确CI](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows标题栏坐标与精确CI/README.md)。下一前台逐动作采完整帧、最大化后坐标归属/还原真实点击联合取证；其余双令牌/物理DPI、安装隔离与KnownFolder、脚本许可、新SHA制品、最终源退避/周期原始计数和官方019完整配对继续保留。原八项门禁/性能failed/预算不提升，IMP/TASK不关闭或发布。

### 2026-10-14 Windows 目标模式最终读回与当前执行边界

旧等待器00:33:41结束且未改目标，原声明600秒/实际630.172秒已纠正。新授权下00:48:34仅发一次官方status=active/origin=user恢复，不替换objective或budget，仍被active writer拒绝；随后及本轮最终宿主工具读回均为原objective/paused，createdAt及累计tokens/time/budget不变。官方文档说明新objective会重置累计用量，故不清除/重建/换目标绕过。Windows工作指令已备妥但尚未应用，不能宣称目标模式自动续行。

本轮具备条件的Windows后台诊断、精确CI/制品及证据回写已在c57c860完成推送；剩余前台、安装/Profile、脚本及Mac新候选依赖不伪造通过。`.adg/evidence/OCX-0110-20261014-WIN-GOAL-READBACK/`记录完整失败/读回和准备稿，[原分工记录补充](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows分工与原TASK复核/README.md)区分已授权工作与宿主控件恢复限制。未开新等待器或前台，原完整门禁/预算及TASK/IMP状态不变，不关闭或发布。

### 2026-10-14 精确版本更新说明与独立重试补齐

feature/0.1.10-release-details@6305747f 已从be7395a开发并推送，包含父焦点补丁但未合并PR#1/main。发现原面板固定无说明、管理器仅拼v-tag网页，与分析03/IMP四态来源合同不符；同新增黑盒测试在完整be夹具实际3项断言失败，新实现通过，另一次0测试导入失败不计产品红。只读npm2.84.0仓库与非草稿对应Release核验有7528字节正文，source审计不等于生产原生更新。

两入口共用ReleaseNotes，明确加载/可读/缺失/独立失败重试，正文纯文本内部滚动，仅核验精确Release才提供网页，updater正文优先，不触发版本检查/备份/安装。后端说明独立事务按候选/通道/世代与维护源校验；总30s、响应256KiB/正文64KiB，无重定向/自动重试。代理沿原策略，cache/updates/release-notes三槽按活动根/对象通道/版本归属，正文24h、缺失1h，失败不覆缓存，缓存命中零说明请求；原调度/退避/签名备份门禁不改。UI迟到结果与根冻结拒绝、重试焦点保留，真实反馈点登记，说明失败不解除版本/安装异常。[实现与证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-精确版本更新说明与独立重试/README.md)。

本地最终前端97文件/501通过，Rust34组/884通过/6忽略(不计通过)，说明针对29UI/12Rust项包含完整suite；fmt/strictClippy/类型/构建/diff通过，4条动态导入警告保留。业务红3项、首轮编译/Clippy/测试夹具失败原件保留。630同SHA CI38490716335最后in_progress，不称Windows构建或制品通过；Mac无桌面启动/新原生性能，Windows旧be不可证明新源码，后续须协调单实例切换新SHA候选。原macOS性能failed、全部预算/门禁、TASKrevision2/in_progress/pending/空验收及FZ50许可缺口保持，无核心晋升、真实迁移、日常替换、自动合并、签名或发布。

### 2026-10-14 630 精确双 CI 与 Windows 安装候选终态

6305747f的push CI38490716335、PR CI38491351619及单次隔离候选38492240481均completed/success。两轮CI五必需job成功/main-only build skipped，MacRust各34组/884passed/6ignored、Windows及候选各34组/612passed/7ignored，ignored不计通过。PR#2基于be焦点分支、17文件open/draft；PR#1/main未合并，不混同PR检出二进制与独立head候选。

两个ZIP11863515199/11863360486大小/sha256/API/CRC/安全条目通过，3/316项，两份安装回执字节一致；MSI4591616字节/SHA6725f7f7982a40229c7465a95bbb3ba822e28afab6c62fb008b3e18cbb4499d7、NSIS3275286字节/SHA3d85d834e689342e1e9dc27e14c533b7dd29a2d00a4185c016411825386eae30匹配回执。runner逐个新装/首屏/卸载成功，实际应用均elevated/type1、mode any，不计双权限；安装后EXE字节不在artifact内，本机仅核回执对应，Mac未执行Windows安装器或开应用。source630、包锁环境及bundle预期摘要一致；codeSigned/releaseAcceptedfalse，沙箱根/DPR/自动收尾不替代默认安装data/物理DPI/正常退出/真实保留/升级门禁。[终态与Windows接续证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-630-精确CI与Windows候选/README.md)。

新说明功能真机须固定630制品，先协调现有be实例结束并确认0，使用准入的安装/只读提取隔离方案，保护日常数据/注册；旧be不能证明新代码。四态/正文网页/长滚动/独立重试焦点/缓存真实计数与其他原矩阵仍待。原macOS性能failed、全部预算、TASKrevision2/in_progress/pending/空验收及FZ50许可缺口不变；没有核心事实晋升、真实迁移、日常替换、自动合并、签名、发布、通道晋升或完成证书。

### 2026-10-14 说明缓存语义校验与客户端按需初始化

feature/0.1.10-release-details@d3b83b38从630接续并推送，仅改说明领域/测试：旧缓存ready/正文/来源/发布时间不一致仍命中，且代理初始化可阻断有效离线缓存。相同测试字节旧12通过/4失败，新16/16；修复后坏语义缓存重查，ready/缺失/来源/时间一致才接受，客户端仅实际cache miss时在阻塞线程池惰性创建，事务内复用。原cache路径/schema/TTL/源/30s与响应限额、调度/安装门禁未改。

完整Rust34组/888通过/6忽略、fmt/strictClippy/diff通过；UI未改，不把630的501旧轮次称本轮重跑。原红与完整绿日志留存。[缓存修复证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-说明缓存一致性与延迟客户端/README.md)。新pushCI38495887680/PRCI38495891289观察in_progress；PR#2head为d3仍draft/basebe，main和PR#1未合并，须新SHA候选设备验收，630旧包只归旧证据。

Mac无桌面启动/原生性能或FZ50脚本，Windows旧实例不操作。原macOS配对failed、全部预算/门禁和TASKrevision2/in_progress/pending/空验收保持；无真实迁移、日常替换、核心晋升、自动合并、签名、发布或证书。

### 2026-10-14 用户确认 Windows 透明图标并完成专属资源接入

用户先要求预览/确认后提交，再确认“我看了效果可以，可以继续”。采用应用透明主体90%与独立亮暗单色托盘；16px增加一像素透明安全边距，32px以上保持确认外观。原品牌规范早已要求Windows无底板，0.1.8复用含底板AppIcon与托盘的实现冲突本轮纠正；macOS默认PNG/ICNS/模板及默认配置不变。旧Windows交付归档，当前素材清单/生成入口在品牌目录，不用旧26.6%小主体素材替换。

feature/0.1.10-windows-transparent-icons@bb3122d基于Mac d3说明修复推送，草稿PR#3/base说明分支，仅Windows资源/配置/托盘与测试；系统任务栏主题独立选色并在既有事件路径刷新，不用应用主题强制选色。旧七ICO帧被新底板合同拒绝、新七ICO/十四托盘通过，官方fmt通过；pushCI38499759600、PRCI38499860360、同SHA候选38499860139全success，五必需CIjob成功/main-only build skipped，runner新装/首屏/卸载成功。两ZIP摘要/CRC、安全条目与MSI/NSIS包回执一致；MSI只读提取EXE SHA3b5c5176…完整bundle/源码身份匹配，PE七档图标像素与源码一致，亮暗托盘PNG均嵌入。未在日常宿主执行安装器。

临时旧be预览10:11:06已还原/结束helper；10:30:51旧app10904/supervisor32492退出且本机原生0，10:31:15新唯一bb3122d PID39680/supervisor44136/exec91561/HWND2625640在原根启动，普通false/type3、日志匹配新源。原生标题栏无底板实际观察与原始JPEG归档，日常EXE及隔离preferences/root/policy/schedule摘要保持，auto=false。新源任务栏/托盘全像素、物理主题/高对比/完整DPI仍不计全通过。Mac原生仍须此新实例归零后交接。

58份实现原件冻结校验通过，PR#3最终说明读回仍open/draft/unmerged/无auto_merge。[透明图标接入与证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows透明图标接入/README.md)。本次只闭合此实现与子集，原完整门禁/性能failed/预算、FZ50许可缺口及TASKrevision2/in_progress/pending/空验收保留，无真实迁移、日常替换、自动合并、签名、发布或完成证书。

### 2026-10-14 d3 缓存修复精确双 CI 与候选终态

d3b83b38 pushCI38495887680、PRCI38495891289和单次Windows候选38497980368均completed/success。两CI MacRust各34组/888passed/6ignored，Windows及候选各34组/616passed/7ignored、0failed；ignored和0测试组不计通过。新CI前端完整测试/类型构建通过，本地旧501轮次不称新重跑。PR#2head更新d3仍open/draft/basebe，PR#1/main未合并。

两个ZIP11864817991/11865046498 API大小/sha256、CRC、安全条目通过，3/315项，两份安装回执原字节相同；MSI4595712字节/SHA5032162d28a2c0ead5387cca058aeb42af80bddd69206feb7abe690a1915171c、NSIS3277473字节/SHAd51f5b47cda0c9010de948a687090737e430c35a702ee53b90111eef5b38b55f与回执一致。runner新装/首屏/卸载通过，source/version/bundle预期摘要对应；安装后EXE字节不在下载artifact内，本机只核回执，不执行Windows二进制。实际MSI/NSIS应用elevated/type1、modeany，不抵真机双权限/物理DPI/默认安装根data/正常退出/升级数据保留，codeSigned/releaseAcceptedfalse。[新SHA候选与接续证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-d3-缓存修复双CI与Windows候选/README.md)。

新d3候选设备复验须协调旧实例结束并确认0，保留日常版/注册/真实数据；有效离线缓存、坏语义缓存重查、四态/长正文/网页/独立重试焦点/真实请求计数等按新SHA取证，不用630旧包通过代替。Mac无原生启动或新性能样本，原macOS性能failed及全部预算/门禁、TASKrevision2/in_progress/pending/空验收、FZ50未许可边界保持。无真实迁移、日常替换、核心晋升、自动合并、签名、发布或关闭证书。

### 2026-10-14 光层层叠后台实验，无性能因果或生产修复

用户明确Windows仍在验收，Mac继续后台；未启动/操作Mac原生应用或结束Windows实例。6路径Gitblob核对：Logo/scene从官方0.1.9到失败deb及当前d3一致，deb相关视觉源与当前相同；差异仍为0.1.9后光层Teleport/层叠覆盖与锚点观察线索，非CPU/RSS因果结论。

无头Chromium新fixture页面setup/ready×mesh/css、高/浅/DPR2中，临时恢复状态区isolation后八节点几何、SVG固定帧和canvas尺寸一致；首轮截图60–98像素差保留，控制CSS动画/通知后30/30/57/60像素差，仍不称全图相同。LayerTree部分初快照缺失/paint事件全0、120合成rAF同一JS任务，不能推断真实帧耗时或重绘/性能改善。临时脚本首解析失败与两轮原件均保留，只关闭自有browser/Vite，生产样式未改。[实验与边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-光层隔离后台诊断/README.md)。

继续等待Windows验收/平台交接，再有条件做真实受控配对；本轮不以静态浏览器证据解除原CPU+1.817629/RSS+117.008965失败、不降预算或撤回顶部/侧边光层需求。原八项门禁/性能failed、TASKrevision2/in_progress/pending/空验收和全部授权边界保持，无生产修复/真实迁移/日常替换/签名或发布。

### 2026-10-14 面板查询所有权修复与集成 Windows CI 失败

fd31d716 将获准的面板元数据查询交独立任务持有准入至持久缓存/截止/事件终态；观察者取消不丢提交。加强版相同测试字节 d3 红1通过/1失败、新2通过，真实临时根调度成功86400/失败300秒且管理器通道不受影响；事件为合成标记，非原生投递证据。本地Rust34组/890通过/6忽略、相关UI16通过、fmt/strictClippy通过。

fb4a6fe32以bb透明图标为父接入fd四路径，四路径/30图标路径原字节与来源相等；本地Rust35组/892通过/6忽略。fd pushCI38503064187 Windows三版本夹具失败，fd PRCI38503069354成功；fb pushCI38503486962同三项及metadata非零退出失败。后者2.0029228秒返回Timeout而非Failure，不是4秒清理断言。原失败完整保留，不归因为环境波动、不触发新候选。

8be5b4cc仅补充测试阶段标记和CI结束后输出，保持并发、生产两秒截止及所有权/清理全部断言；本地fmt/Node语法/diff通过，新pushCI38507499433最后in_progress。main仍eae、PR#1/2/3未自动合并。详见[修复、集成与失败原件](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-面板查询后台提交与Windows失败诊断/README.md)。

Windows仍验收、Mac仅后台，无新原生或性能样本。原macOS CPU/RSS failed、全部预算/矩阵、TASKrevision2/in_progress/pending/空验收及FZ50许可缺口保持；无真实迁移、日常替换、稳定核心晋升、签名、发布或完成证书。

### 2026-10-14 查询夹具诊断 CI 终态及平台接续

8be5b4cc pushCI38507499433已failure：Windows库516通过/7失败/4忽略、后续组未运行；PRCI38508035604 success，Windows35组/620通过/7忽略，两轮Mac各35组/892通过/6忽略。PR实际checkout032088fd为合成合并，不抵独立head失败或候选制品。新push七项查询结束前全部control/parent阶段标记缺失；独立control不属生产查询Job，故下一步同时查Node启动/宿主调度/标记I/O，尚无根因结论。成功PR版本约0.5秒有标记，仍不解除原失败。

两份完整日志ZIP/原job字节、REST终态及CRC/摘要核验独立封存；新测试二进制artifact下载超时，不称EXE/候选身份已核验。PR4已更新终态、保持draft；未触发新候选、不调并发/截止/预算。Windows固定8be源在避开性能采样时后台诊断，现有bb原生制品不抵fd修复；Mac继续后台，切原生仍须Windows实例0。见[终态与接续](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-查询夹具阶段CI终态/README.md)。原性能failed、TASK不变、全部门禁/许可/发布边界保持，目标active未关闭。

### 2026-10-14 面板持久提交顺序与失败合并

9cf21317先等待持久提交后发布新面板value，修正先发布再落盘的成功误报；a7feba5f补齐提交失败也记录终态序号与具体错误，使同次等待者复用失败且不再次派生metadata，后续显式重试仍可成功。字段仅内存，不增路径/schema/周期/自动重试，原2秒截止/退避与通知尽力投递合同不改。

真实cache目录冲突回归在9cf同函数红0通过/1失败，新实现通过，函数3654字节/SHA ca26e1f8…一致；初编译失败不计产品红。原取消测试改用同一生产提交函数，延迟commit保持mutex/未发布序号，metadata失败真实写300秒退避。最终Rust35组/895通过/6忽略、fmt/strictClippy/diff通过；UI未改、原生未运行。图标30路径与bb、五共享文件与docs/main原字节一致。

9cf pushCI38517002166已success（Mac893/Windows621），PR38517007573最后in_progress；a7 push38518361054/PR38518366179最后in_progress，旧SHA不抵新实现。PR4更新draft/open/unmerged，新候选未触发；原fd/fb/8be夹具失败和control启动线索不撤销。见[提交顺序、失败合并及证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-面板查询持久提交与失败合并/README.md)。

Mac继续后台、Windows仍验收，平台切Mac原生须确认Windows实例0。原CPU/RSS failed、全部预算/矩阵、TASK原字节/revision2/in_progress/pending/空验收及FZ50未许可保持，无核心事实晋升、真实迁移、日常替换、签名/发布或完成证书。

### 2026-10-14 更新存储静态链接隔离与 a7 双 CI 终态

aae2a3fe修复调度/版本缓存跟随manager-state/cache/updates链接读取和覆写其他根的实际缺陷，统一更新领域storage检查并接入说明缓存。最终相同调度测试模块在a7红0通过/2失败、aae绿；根内目录/叶类型、Windowsreparse/linkcount、读取上限及缺失读零创建覆盖，macOS最终Rust35组/900通过/6忽略、fmt/strictClippy/diff通过。路径/schema/TTL/2秒查询/周期预算不改，WindowsJunction新SHA待CI；未证明并发父目录替换race或全分区安全。

a7push38518361054/PR38518366179已completed/success，Mac各895/Windows各623；PR实际f5f774dd合成检出。9cfPR38517007573也已success，Mac893/Windows621；三份完整日志/REST/checkout/CRC独立封存。aae新push38521648491/PR38521655917最后in_progress，旧成功不抵新实现；PR4title/说明更新draft/open/unmerged，main仍eae，未生成新候选。见[静态链接修复及精确CI](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-更新存储链接隔离与a7双CI终态/README.md)。

Mac无原生或新性能，Windows仍验收，原query夹具失败/control未及时启动原因未解；原CPU/RSS failed、全部预算/双平台/安装/真实周期门禁及TASK原字节/revision2/in_progress/pending/空验收保持。无FZ50脚本、真实迁移、日常替换、核心晋升、自动合并、签名、发布或证书。

### 2026-10-14 aae 双 CI 终态与固定 SHA Windows 候选

aae2a3fe pushCI38521648491/PRCI38521655917均completed/success，Mac各35组/900passed/6ignored、Windows各35组/627passed/7ignored；新增Junction/硬链接/有界读取/持久失败合并实际执行通过。PR检出7365f7b0合成不当head制品；两份CIartifact大小/SHA/CRC与测试EXE/首屏原回执核验，PR下载reset截断/同artifact补取完整原件保留。

双CI终态及零已有候选核验后只发一次workflow_dispatch38524086177，headaae/package全success，Windows627通过/7忽略、MSI/NSIS隔离新装/首屏/卸载通过。artifact11875775415 ZIP7625913/SHAea980905…，证据11876045020 ZIP1690231/SHAe75f5949…；MSI4567040/SHA0dbcb8cf…、NSIS3270031/SHA3937bdcd…，两份安装回执byte-identical，startup source/hash/token绑定。实际应用均elevated/type1/modeany，未取得installed EXE本体独立核验，不抵普通权限/DPI/默认根data/升级卸载数据保留或完整性能。Cargo.lock仅CRLF/LF差异、原始摘要各存，TOML依赖一致；不称byte-identical。

见[精确候选与设备接续](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-aae-存储修复双CI与Windows候选/README.md)。新SHA验收先Windows旧实例结束确认0后启唯一新实例；Mac仍后台。PR4draft/unmerged/main仍eae，未签名/发布/stable晋升；原query启动失败、macOSCPU/RSSfailed、全部预算/矩阵、FZ50许可缺口及原TASK字节/revision2/in_progress/pending/空验收保持，无关闭证书或稳定核心事实晋升。

### 2026-10-14 原契约与八门禁复核，候选保持未验收

按原TASK182读取原字节并provider校验，9个AC绑定与当前DMDdigest全部匹配；23份原task_digest封存RCP有效、acceptance_results共0，performancefailed保持、其余七项整check尚无匹配整体验收回执，不撤销既有局部通过。八门禁的当前证明/缺口/退出条件及P8–P10顺序整理到[固定候选验收与合并发布门禁](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/07-固定候选验收与合并发布门禁.md)，不改预算、TASK或AC状态。

事件当前8triggers/39jobs/81events/27policies/87emissions/8scheduling/9UIfeedback/12paths/4cleanup，79active均登记source，generic sync/backup/migration仍planned；文件定位/词法入口不代替全域投递。损坏面板保护关联受控修复未证明，现有安全拒绝/保留不当完整恢复。旧节点名称/统计仅历史，本页不提前晋升核心事实。

四草稿PR身份核验maineae/focusbe/detailsfd/iconsbb/candidateaae；main×aae merge-tree干净且tree=d9d894fb等于候选，但details×aae的about.rs/content、about_query_ownership_tests.rs/add-add冲突。外部临时index保留aae两blob后整个tree等于已测候选，实际HEAD/index/ref/PR均未变，未合并。详见[审计证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-原契约与合并门禁复核/README.md)。新mergeSHA仍需mainCI/候选/门禁，旧artifact不晋升。

本轮无生产代码/原生/性能/安装动作，Windows仍验收、Mac仅后台，Mac原生先协调Windows实例0。aae候选冻结、原CPU/RSSfailed与全部矩阵/FZ50许可/日常数据保护保持；原TASKrevision2/in_progress/pending及空验收、goalactive不变，无稳定核心晋升、自动合并、签名、发布或关闭证书。

### 2026-10-14 安装向导取消阶段伪百分比

f545cb33修复SettingsRoute仍用安装percent阶段常量、失败100%及提前done的反馈；等待实际outcome/来源核对/事务finish才显示完成，不定条按既有软件画质/可见性策略动画或暂停，命令/离线/重启/保护待核验事实保留。后端协议/注册/脚本许可不改。相同最终10项挂载测试在父aae全失败、新全UI98文件/511通过、类型/构建/diff通过，原4动态导入和canvas警告保留；Rust/原生/性能未重跑。

新push38533772616/PR38533779767最后in_progress，PR4draft/open/headf545未合并；已验证aae候选不含新UI，不抵新SHA。main×f545只读tree模拟干净，无实际merge/branch改变。原保护损坏恢复仍缺精确关联事实，manifest ActivationProof不含backup ref，不能从版本/目录猜造；Windows向导POSIX路径判断另为下一挂载复现线索，本提交未改或验。见[向导修复与边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-安装向导真实阶段反馈/README.md)。

Mac继续后台，Windows仍唯一原生验收，切Mac须实例0；原性能failed、预算/八门禁、FZ50许可/日常真实数据边界及TASK原字节/revision2/in_progress/pending/空验收保持，不关闭/签名/发布或核心事实晋升。

### 2026-10-14 Windows 向导路径红绿与父提交 Windows CI 失败

51a35eff将SettingsRoute仅POSIX的路径校验改为平台事实识别盘符/UNC，按组件/分隔符/Windows大小写判断归属，同前缀相邻目录不误标；拒绝相对/根/设备/点段，mac保护检查保持，原请求仍由后端系统保护/链接/写权限验证，不固定盘符或放宽ACL。最终同22项测试父f545为14通过/8失败，新UI99文件/533通过、类型/构建/diff通过；原Tauri全部字节与aae相同，未本地Rust/原生/性能重跑。

f545 PRCI38533779767已failure：Mac900通过/6忽略，Windows库527通过/三版本夹具失败/4忽略。约2秒Timeout时独立control/parent标记缺失，完整日志/ZIP/CRC保留，原因未解不认定环境波动；父push最新job回归success但构建仍进行，不能覆盖PR失败。新51a push38535387208/PR38535398225进行中，frontend均success，不抵完整Windows或新SHA制品。PR4title/body经owner回执核验draft/open/unmerged，旧aae包不含51a/f545，暂不新打包。见[路径修复与失败原件](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows向导路径与父提交CI失败/README.md)。

Mac仅后台，Windows原唯一实例不操作；原性能failed、全部预算/门禁、保护损坏恢复缺口/FZ50许可、日常与真实数据和TASK原字节/revision2/in_progress/pending/空验收保持，无合并/核心晋升/签名/发布/关闭证书。

### 2026-10-14 控制 Node 自有句柄后台诊断

bd2f6eb1仅改Windows测试fixture，在query前/返回/原observer结束记录自有control句柄Wait0、GetProcessTimes/exitcode与本地标记，即使observerpanic也保留最终状态；JS/产品源码与51a相同，不预热/等待/新增进程，不改2s/4s/5s/8s、并发或清理断言。本地fmt/diff通过，WindowsAPI实跑待push38537161445/PR38537166560；父51a两CI38535387208/38535398225最后仍进行中，原f545失败保持。观察开销非性能样本、快照非根因结论；见[有界诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-控制Node句柄后台诊断/README.md)。

WindowsCLI传输层虽存在，但偏好命令仍明确禁用开启，界面与当前命令行为一致；本轮未以UI改动声称完整IPC启动。oldaae候选不含51a/f545，未新打包。Mac只后台、Windows唯一实例不操作，原预算/性能failed/八门禁/FZ50许可/保护恢复/日常与真实数据边界及原TASKrevision2/in_progress/pending/空验收保持，未合并/核心晋升/签名/发布或证书。

### 2026-10-14 51a 双 CI 终态，旧查询失败不撤销

51a35eff push38535387208/PR38535398225均completed/success，UI各99文件/533通过，Mac/Windows各35组/900/627通过、忽略6/7；PR实际fe4fb178合成检出。f545push38533772616也success、UI511/后端900/627，f545PR三版本夹具失败仍保留，不能用新绿撤销或推断根因。三份完整日志ZIP/REST/checkout与测试数独立核验，详见[51a终态](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-51a双CI终态/README.md)。

bd2仅诊断head两CI当前Windows回归仍in_progress，未计完整WindowsAPI通过；oldaae候选不含51a/f545UI，未新打包/原生动作。原预算/性能failed/八门禁/FZ50许可/日常真实数据和TASK原字节/revision2/in_progress/pending/空验收保持，goalactive，无自动合并、核心晋升、签名/发布/证书。

### 2026-10-14 bd2 控制进程快照证实低执行进展，PR失败保留

bd2PR38537166560完整failure：Mac35组/900通过/6忽略、Windows库523通过/七query夹具失败/4忽略。七独立control在原observer5.05–5.66秒结束仍同handle/creation、Wait0=258、exit259/exitFILETIME0，user CPU0–31.25ms/kernel0–31.25ms，control标记三个时点均缺失。WindowsAPI诊断已实际编译/执行但业务失败不解除；线索收窄到JS之前/初期低执行进展，尚无loader/线程调度/I/O根因，需Windows自有新PID的限定时间窗原证据。完整日志/REST/ZIP摘要/CRC和逐组analysis封存，见[失败与后台下一证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据/README.md)。

push38537161445最后进行中，旧51a/f545成功轮次不抵本PR失败；PR4draft/未合并，无新包/原生/性能。原2s/4s/5s/8s/并发/预算与macOSfailed、八门禁/FZ50/日常数据/原TASKrevision2/in_progress/pending/空验收保持。Mac继续后台，Windows旧runnerPID不当本机目标，后续不靠预热或放宽截止求绿，不合并/签名/发布/核心晋升或证书。

### 2026-10-14 bd2 push 完整终态，PR失败继续开放

push38537161445 completed/success，bd2独立head，Mac900/Windows627通过（35组各、忽略6/7）；完整日志/REST/checkout/CRC封存。七control在query返回前已写标记；但成功versioncontrol的userCPU也可0，不能以单CPU指标推断执行阶段。PR38537166560七夹具失败及5秒active/标记缺失联合线索保持，根因待Windows原线程/调度/I/O证据，不放宽预算/重跑求绿。见[终态对照](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据/push终态对照.md)。

两个bd2CI均终态，下一后台诊断依赖Windows原证据；旧aae候选不抵51aUI，无新包/原生操作。原CPU/RSSfailed/八门禁/预算/FZ50许可/日常数据和TASK原字节/revision2/in_progress/pending/空验收保持，目标active，未合并/签名/发布/核心晋升或证书。

### 2026-10-14 Windows 固定bd2后台单次线程/WCT/I/O成功对照

用户授权接续后，Windows固定bd2f6eb10d58115235e35d575507bb69f44666da，独立push38537161445实际checkout、artifact归属/ZIP摘要CRC与runner测试EXE摘要匹配；测试EXE SHA2709775e…不含commit字面量，初准入停止后改以实际checkout/原制品身份绑定，不混同PR合成或桌面候选。CI Node24.21.0官方ZIP/SHASUMS与Authenticode OpenJS/Valid核验，EXE SHAba4e6d11…，不安装npm/生命周期/改全局PATH/预热。

17:33:37.903–17:33:48.585仅执行一次原完整库，默认并发/capture、无filter和截止/清理断言变更。结果529passed/1failed/4ignored、exit101，七查询夹具均passed；失败为编译时runner路径下LICENSE的license.opened，本机不造路径或改断言求绿。不复现旧PR五秒active/标记缺失，原失败及根因问题保持。

836条原JSONL：554进程采样、六自有Node后代+测试根、80轮、274自有临时夹具文件通知；78次WCT成功/2次1444，保留句柄/EXE路径/新creation核验及线程状态/CPU/cycle/进程I/O累计量。原capture隐藏成功场景详细日志，case↔PID与短进程覆盖不全；观察器初始化晚约312ms、最长读取1459ms，快照/WCT不替代调度ETW，文件通知不代替逐文件读写时序/延迟，不作性能样本或Defender/loader/调度原因结论。外层保护未触发，无agent终止进程，原夹具正常清理后官方Node/desktop均0。WPR仅status未start，不能宣称ETW被OS拒绝。

35份方法/原件/分析清单冻结重验通过，[Windows线程与I/O成功对照](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2线程与IO对照/README.md)。本提交是本机成功对照证据，**不是bd2失败场景原始调度/文件I/O跟踪**；下一证据需实际失败时间窗及限定自有PID的适当录制。Mac继续后台，未正式交接原生；旧bb session最后14:58:12已不存活，不能当实时证据。日常EXE及隔离preferences/root/policy/schedule摘要保持，未启动新自动周期。

本线程目标工具仍paused，本轮单次正式status-only active/origin=user恢复被active writer拒绝，原目标/累计用量读回未变；用户授权已接收且诊断实际完成，准备的优先指令未应用，不能宣称目标自动续行。无绕锁、清除重建目标、新CI/包/desktop/防护或全局改变；原完整门禁/CPU-RSSfailed/预算、TASKrevision2/in_progress/pending/空验收及FZ50缺口保持，不合并、签名、发布或关闭。

### 2026-10-14 Mac 接收 Windows 成功对照并复核真实覆盖

后台同步docs019dd6c2，代码bd2/main eae不变；原TASK3878字节/SHA bc7965b4…仍一致。35份原证据逐路径字节数/摘要完全一致，原verify-evidence在Mac失败是Windows/Posix大小写名称排序造成的列表顺序差异，原清单/方法不改，独立只读复核通过且保存原失败stderr。

六Node各79条、共474条进程记录均Wait0=0、已退出；首次采样晚于退出约12.8–84.1ms，Node活跃/线程/WCT均0。2664线程和80WCT全部归测试主进程，Node累计I/O无区间变化，不能解释旧CI早期启动等待。下一项需实际失败时间窗内自有活Node的case/creation/句柄/时刻与调度/I/O证据；串行WCT最长读取1459ms的扰动保持显式，不以50ms等待推定固定采样。详见[Mac独立复核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows成功对照Mac复核/README.md)。未正式交接原生，Mac继续后台，原七CI失败/CPU-RSSfailed/预算/八门禁及FZ50许可保持，不推定根因或版本验收，不启动新CI/候选/desktop，不发布或关闭。

### 2026-10-14 Windows 原失败测试制品准入与实时采集准备

用户在宿主设置了完整新目标，本线程get_goal现读active、createdAt/预算保留；不再尝试目标恢复或重置累计用量。Windows按path-keyed摘要及原JSONL独立复核Mac b14a0f6结论：旧019dd6c六Node均为退出后采样，活Node线程/WCT为0，全部2664线程/80WCT归测试根。旧原件/清单不改，列表排序差异不冒充字节损坏。

取得原失败PR38537166560测试EXE并核验API归属/ZIP摘要CRC/runner回执，EXE SHA497814ca…与成功push2709775e…不同；PR实际checkout31e7748d整tree753be161…与bd2相同，未由树同或二进制差异推定根因。失败EXE本轮未运行，Node仍固定官方24.21.0/SHAba4e6d11…，未预热。

feature/0.1.10-windows-query-etw@5fa36a60仅新增三个test/smoke诊断文件，基于bd2，不改apps/原夹具/CI工作流。草稿PR#5/base integration-candidate/bd2未合并；本机Add-Type编译、38项生命周期/PID-TID重用合同、默认准备及摘要/输出边界/普通权限拒绝通过。固定Microsoft TraceEvent3.1.21官方metadata SHA512/ZIP CRC核验，本线程runtime解码DLL，无全局安装或包脚本。初始获取/压缩/编译错误如实记录；CS0618首轮stderr只在工具输出、重定向为空，不伪造日志。自动CI状态读取失败，不计新CI通过。

管理员单次录制可审阅入口已准备：默认只显示方案，明确批准且手动管理员PowerShell7才可能运行；源HEAD/无局部改动、制品/Node/DLL摘要、desktop0、新输出根均为执行前提。先启唯一实时session，再启动一次原完整库，默认并发/capture和2s/4s/5s/8s不改；120s只作原库外层保护，另短时排空。系统内核事件短暂内存解码，持久化按自有家族/原QPC生命周期筛选，不宣传kernel PID白名单、不写整机ETL或保存其他调度参与者ID/命令行/SID/无关路径。队列/IRP/输出均有界，无WCT串行轮询、全局防护修改或自提升。真实事件覆盖/失败复现/排空未执行，loss计数设计为stop前快照，不称最终零丢失。

35份准备原件冻结校验通过，[实时采集准备与权限范围](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2实时采集准备/README.md)。本轮原测试/Node/ETW新执行数0、本机desktop0，管理员录制批准待用户；不以工具准备代替失败原始证据。无手动CI重派/新候选/原生交接，原CI失败/性能failed/预算/八门禁/TASKrevision2/in_progress/pending及FZ50缺口保持，目标active、不关闭/签名/合并/发布。

### 2026-10-14 采集器分支双 CI 终态，实际录制仍待批准

只读核验5fa36a60的push38555890377/PR38556292712现completed/success，五必需job成功/main-only build skipped；每轮Mac900/Windows627passed、忽略6/7，各35组。七query夹具在两通用回归均passed，不撤销原bd2 PR38537166560失败。push实际5fa、PR实际a813c1dc，整tree均fa1d4d54…；apps/原CI工作流相对bd2未变。

完整两ZIP/CRC/安全条目与四后端checkout/test原件核验，日志解析初次误选Post checkout触发不覆盖保护，精确选择后只复用已下载/已保存字节；GBK读取失败改显式UTF8，不重造原件。exec67983/PID33408有界等待166.016秒后正常终态，期间读取错误不当远端停止，不重派/重启。PR#5最新仍open/draft/unmerged/无auto_merge。[双CI证据与边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2采集器双CI终态/README.md)。

CI未执行实时ETW采集器或38scope本机检查，不证明record/事件覆盖/排空/隐私过滤实跑或原失败复现。自动目标接续不是管理员批准；本机desktop/官方Node0、录制输出根不存在，未再次跑本机原库/Node/ETW或自提升，正式录制仍待人的批准，目标active。原CI/性能failed、全部预算/八门禁、TASK原字节/revision2/in_progress/pending及FZ50许可/日常数据边界保持，无新包/原生交接、合并、关闭、签名、发布或晋升。

### 2026-10-14 采集器启动/停止实际设计缺陷修正

管理员录制批准仍缺，本轮自动目标只推进代码审阅及纯准备检查。按固定TraceEvent3.1.21源核实5fa采集器先读Source会自动启动普通session，再EnableKernelProvider将被库拒绝；旧38scope与旧双CI不覆盖真实启动顺序。另核实库通用Stop会TraceSetInformation重设机器CPU采样间隔并做heap清理，不符合原“不改全局环境”范围。

abc3a914接续5fa，仅改原三个test/smoke工具，先EnableKernel再访问Source，StopOnDispose=false且不调用通用Stop，改用ControlTraceW按唯一自有session名STOP，并返回该session最终loss计数；布局及名字检查纯执行，实际ControlTrace仍待批准。writer错误捕获/排空仅编译审阅未注入线程故障，cleanup逐阶段异常继续有纯检查；完整consumer/writer/stop未真实跑，不预称通过。实际失败case/活Node/调度/IO仍缺，不关闭原失败。

本机Add-Type和53项纯检查（38scope/7生命周期/8native-stop布局与名称）通过，摘要/输出边界/普通权限拒绝保持，ETW/Node/原库/desktop新执行0。草稿PR#5更新标题正文到abc、仍open/draft/unmerged/无auto_merge；新push38561752521/PR38561759502末读仍in_progress，四通用job成功/Windows最终EXE构建，旧5fa绿不抵新head。只读等待600秒终态后按同run直接复核，没有把观测错误或截止当远端停止，不重派。

25份修正/权限原件冻结校验通过，[生命周期修正与当前入口](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2采集器生命周期修正/README.md)。待批准record-request.ps1新入口固定abc/无局部改动，旧2fd/e581准备证据不改，旧入口拒绝新HEAD。系统内存筛选范围、一次原库/120s外层保护+收尾、原2/4/5/8s/并发/隐私界限保持。目标active、desktop0，原失败/性能failed/预算/八门禁/TASK原字节revision2/in_progress/pending/FZ50许可/日常数据边界不变，无录制批准推定、原生交接、新包、合并、签名、发布或关闭。

### 2026-10-14 abc 采集器修正双 CI 终态与执行依赖

abc3a914 push38561752521/PR38561759502现completed/success，五必需job成功/main-only build skipped，Windows最终EXE首屏通过；各35组，Mac900/Windows627passed、忽略6/7。push实际abc，PR实际16c781ea，整tree均0aa5440b…；apps/原CI/锁/共享文件相对bd2不变，head/PR构建身份仍分别记录。完整两ZIP/SHA/CRC、安全条目与四后端checkout/test原件封存，不以树相同推断二进制相同。PR#5正文补新CI终态后读回仍draft/open/unmerged/无auto_merge，head/base/title不变，未重派CI或新候选。

新CI不执行C#实时collector或53项本机纯准备检查，不能证明真实kernel enable/过滤/排空/native stop、失败case/活Node覆盖或根因。管理员批准持续缺失，自动目标续行不作批准；本机desktop/官方Node0、录制根不存在，日常EXE及隔离偏好/根/策略/调度摘要匹配，未再次跑本机原库/Node/ETW。

27份终态原件冻结校验通过，[修正双CI及当前执行依赖](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2生命周期修正双CI/README.md)。已完成独立后台的工具修正、制品来源、准备合同和同源CI；下一实际失败取证需批准/提权，完整原生需新前台窗口和最终候选，安装/Profile需安全隔离，FZ50脚本许可仍缺，性能/周期需原控制条件，Mac需正式平台交接。没有当前获准可执行的替代工作能取代这些证据；不继续纯文档/测试求绿循环或缩小目标。

原完整版本目标/八门禁/CI失败/mac性能failed/预算/TASK原字节revision2/in_progress/pending和空验收保持；待用户权限或环境变化再接续实际取证，整体未完成，不关闭证书、合并、签名、发布、晋升或替换日常安装。

### 2026-10-14 Windows 管理员录制中止，根映像与异常输出修正

用户明确批准管理员后台ETW并完成提权；abc采集器一次执行原失败PR38537166560的EXE/SHA497814ca…，官方Node24.21.0不变。20:48:34创建提升控制器PID26048、真实管理员角色核验成功；20:48:37.665创建原库PID55376后Root image mismatch，事件消费者未启动。原库实际启动1/完成0、testExitCode null、强制外层收尾，原测试stdout/stderr未保存、事件0条；不能判断七query结果或认定旧失败复现。旧MainModule实际返回值缺失，具体误判原因仍未知。

唯一自有session native STOP=0、最终loss/buffer loss=0、writer排空成功，独立只读检查无session/自有进程/desktop残留；这些只证明收尾，不证明完整覆盖。只有控制器令牌已核验，测试根真实令牌未采得。日常EXE与隔离偏好/根/策略/调度摘要一致，无新原生/周期。原件全部七份封存，旧证据不改。

629b9c6f仅修改原三份test/smoke工具：以已持有Process.Handle的QueryFullProcessImageNameW查询根映像、校验前留记录，不等待初始化或预热；启动立即异步读输出，异常也有界排空，单列setup/run error及完成/收尾状态。本机编译、原53项纯检查+5项自身句柄映像检查及摘要/目录/普通权限拒绝通过，没有再次执行原库/Node/ETW，修正后真实录制待验证。代码已推送，PR5描述已独立读回draft/open/unmerged/无auto_merge；新push38567345341/PR38567352891初读仍in_progress，旧abc双CI不抵新head。

见[管理员录制中止与映像修正](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2管理员录制中止与映像修正/README.md)，证据`.adg/evidence/OCX-0110-20261014-WIN-BD2-ETW-RUN/`。本次批准的一次原库已启动，不自动第二次；下一步先核验修正SHA终态，再明确批准新的有界录制。Mac继续后台、未原生交接；旧查询失败/根因、性能failed、原预算/八门禁/TASK原字节revision2/in_progress/pending及FZ50许可保持，整体目标active且未完成，不关闭、合并、签名、晋升或发布。

### 2026-10-14 根映像修正双 CI 终态与实际检出准入

629b9c6f push38567345341/PR38567352891现均completed/success，五必需job成功/main-only build skipped；每轮Mac900/Windows627passed、忽略6/7，各35组，runner最终Windows EXE首屏成功。实际push checkout629b9c6f、PR合成513cea4a，原checkout日志与GitHub commit元数据分别核验，整tree均30e3be09…；完整两ZIP的SHA/CRC、安全条目及八份backend原日志字节通过，不把tree同推定为二进制同。

只读观察同两run，API间歇错误保留；后续读回确认继续构建，21:14:55两轮终态成功，观察器正常退出，无重派或新候选。PR5仅补正文并独立读回draft/open/unmerged/无auto_merge，head/base/title未变。见[双CI与录制准入](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2根映像修正双CI与录制准入/README.md)，证据`.adg/evidence/OCX-0110-20261014-WIN-BD2-IMAGEFIX-CI/`。

ci-admission仅供新review-only入口核验源码CI/实际检出/原日志，不是第二次管理员录制批准。通用CI未运行C#collector或53+5本机检查，不证明句柄查询后的真实启动/事件过滤/排空或失败取证。本机原库/Node/ETW/desktop新执行0，第二录制根不存在，旧中止原件重验、日常及隔离数据摘要保持、原TASK字节不变；下一实际执行仍需人的明确批准。Mac保持后台、无原生交接；旧七query失败/根因、性能failed/预算/八门禁/TASKrevision2/in_progress/pending及FZ50许可保持，目标active，整体未完成，不关闭、合并、签名、晋升或发布。

### 2026-10-14 持续管理员后台授权、完整原库与采集容量缺陷

用户明确“批准修正版后续都可以管理员后台 ETW 录制”，后续本目标内修正版必要有界后台ETW不再逐轮请求权限；仍固定源码/CI/制品、独立目录、一次原库/120s外层、原2/4/5/8s/默认并发capture/断言，无预热或无限重跑求绿，UAC仍人工、前台另有新开始信号。

629同SHA准入后21:23:35–46完成原失败PR38537166560制品的一次原库，529passed/1failed/4ignored，七query均通过；唯一license.opened仍编译时runner路径缺失，不改断言造路径。根55592的持有句柄映像匹配、真实elevated/type2，原输出保存，consumer/writer排空与自有session stop成功、最终kernel loss0、未外层强制。没有复现旧query失败，也不以测试根管理员令牌代替产品权限门禁。

实际JSONL183278条/67291856bytes，测试根178269条占97.27%，前约0.64秒耗尽序列化额度，1339173丢弃、无终止记录；kernel0不抵序列化丢失，覆盖不完整。CRLF实际每行多1byte，文件超64MiB182992bytes，旧预算只按LF计量是实现缺陷。原JSONL无损gzip2889091bytes/解压摘要一致，成功查询不清除旧失败或推定根因。

3a91c175仅修原三份test/smoke采集器：根身份/进程线程映像生命周期保留，高频调度/文件IO集中已准入Node/cmd后代、省略根热点显式计数；UTF8+LF真实写入及预算一致，queue失败不占额，64MiB/32768/120s上限保持。本机编译及78检查通过（原58+10后代范围+10实际内存字节），未新执行原库/Node/ETW；新push38572890687/PR38572902717初读in_progress，PR5draft/未合并。新review-only入口固定3a，先新SHA准入再依持续授权单次录制，不重复求绿。

详见[持续授权录制与容量修正](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2持续授权录制与容量修正/README.md)，原件`.adg/evidence/OCX-0110-20261014-WIN-BD2-ETW-IMAGEFIX-RUN/`。自有session/进程/desktop0，日常及隔离数据摘要保持；Mac后台未原生交接，旧失败/性能failed/预算/八门禁/TASK原字节revision2/in_progress/pending及FZ50边界不变，目标active，整体未完成，不关闭、合并、签名、晋升或发布。

### 2026-10-14 e6 basename 命令映像准入与完整自有家族录制

e6a4b32f push CI38583679031成功；PR CI38583689702保留原三个Windows version fixture失败（Windows527通过/3失败/4忽略），两轮CI均未执行collector。失败保持诊断范围，不能作为版本通过。e6本机准备91项通过。

持续授权下完成一次真实管理员后台录制：原库PID73328，真实测试根令牌elevated/type2，exit101；28,001事件/9,591,262bytes，零序列化丢弃、零kernel loss、零decoder error，session/writer/consumer正常收尾。ETW Process/Start仅给cmd basename时，e6按已归属父进程+映像/argv basename准入并由持有句柄复核；本轮记录测试根1、Node22、command-basename8，共31 start/31 stop，已知映像未准入0，补齐0407命令覆盖缺口。

原库529passed/1failed/4ignored，七query均通过，唯一license.opened仍为本机缺失CI runner路径；没有复现bd2失败时间窗。根高频调度915533、根I/O initiation311295仍按范围省略并计数，故证明的是自有Node/cmd家族及其线程/调度/IO/生命周期覆盖，不能宣称测试根全部热点完整。见[basename覆盖与完整家族录制](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261014-Windows-bd2basename覆盖与完整家族录制/README.md)，证据`.adg/evidence/OCX-0110-20261014-WIN-BD2-BASENAME-RUN/`。历史失败、性能failed、完整双平台门禁、真实周期/安装/更新/备份/树表/事件要求及TASK revision2/in_progress/pending保持；Mac后台未交接，不关闭、合并、签名、晋升或发布。

同 SHA 代码 worktree 的 Windows harness token 自检及纯 bundle identity 自检也已完成：普通 harness/child 真实令牌检查与拒绝错误身份、错误模式、退出/释放句柄等 9 项通过；NSIS/MSI marker 与 payload 纯测试 5 项通过。首轮因调用 CWD 为 docs 分支而错误记录 docs HEAD，原件保留；改在代码 worktree CWD 重跑后记录 `e6a4b32f`，证据`.adg/evidence/OCX-0110-20261014-WIN-TOKEN-HARNESS/`。该证据不替代应用普通/管理员双权限、原生 UI、安装器或性能门禁。

### 2026-10-15 e6 精确桌面候选与普通权限前台子集

用户开始30分钟Windows前台窗口，但开始时缺精确桌面候选；此次未提前准备的耗时如实保留，不当UI验收。仅dispatch一次e6 Windows candidate run38591604133，最终success；artifact11909280359/11909105485 API摘要、ZIP CRC/安全条目、包内source/回执核验，MSI只读提取EXE SHA2c9cad96…匹配精确bundle回执/版本。未本机安装或卸载，原e6 PR三个query失败及bd2失败不因新candidate绿撤销。

用户本轮“不用管dpi的问题”，物理缩放跳过、门禁保持待验；实际远程会话metric1/nativeDpi192不当物理200%。00:00创建唯一普通实例73920/elevated false/type3，新隔离根自动面板/自动检查/启动清理关闭；初次UI工具遭用户物理Escape即停止，只有用户00:16再“继续前台验证”后才操作同实例，至00:31短窗口收尾。

真实Sky输入及原生只读状态证明：同e6首屏/单系统标题/无应用菜单；最大化3200×1121；索引还原点击未生效原件保留，当前截图坐标点击回原1180×760及精确外框；最小化/恢复同PID，DWM backdrop2→1→2且日志mica→inactive solid→mica；真实设置关闭keep-on-close后系统关闭按钮正常退出，native0、未强制终止。时间字符串被JSON解析为DateTime造成只读方法误报PIDreuse，原记录保留并改UTCticks/原生creation核验；未修产品。

详见[普通权限前台子集](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows-e6普通权限前台子集/README.md)，57份证据方法/回执/截图/状态清单`.adg/evidence/OCX-0110-20261014-WIN-FG-REQUEST/`。日常EXE/旧隔离状态摘要保持，前台已结束；管理员、托盘重开/Snap、全部物理/安装/更新/备份树表/事件/退避周期及配对性能仍开放。完整原目标/八门禁/TASK原字节revision2/in_progress/pending/旧性能failed/Mac依赖不改，不关闭、自动合并、签名、晋升或发布。

### 2026-10-15 Windows e6 托盘隐藏阶段与官方基线补核

00:45:43创建唯一隔离普通实例56448，真实产品令牌false/type3、同e6候选/同EXE摘要；keep-on-close=true。首次关闭点击仍可见原件保留，随后Sky Alt+F4使同PID主HWND2229478隐藏、DWM2→1，未退出。MainWindowHandle误选13×13托盘helper的原件保留，只读方法改为绑定原主HWND/PID/creation。Snap提示出现但捕获no monitor found，未通过；Sky无系统托盘/隐藏窗口入口，恢复报window id 2229478 was not found，未使用其他输入或第二实例绕过。

用户提供托盘图标截图后，07:24:51仍为原PID/creation、主窗口隐藏；裁切图标不替代产品身份。07:27:49保护检查native1、普通未正常退出、管理员未启动。本次恢复仅后台读取，原15分钟前台时段已过，不自动延长；下一步待托盘打开主界面并实证重开/正常退出，归零后顺序管理员对照。自动检查关闭，经过的小时数不当真实周期证据。

官方0.1.9 release407422226/asset623593980的MSI摘要b33dd0c8…与本地4030464bytes匹配；标签API读取失败保持，独立Git远端peeled tag核验de6862d4，原只读提取EXE626d63ad…及ocxd14cfc0a7…逐项匹配。未启动基线/安装器，冷热/600s配对新样本0，不作性能通过。30份历史快照与5份补核原件分别校验，详见[托盘隐藏与基线补核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows-e6托盘隐藏与基线补核/README.md)，证据`.adg/evidence/OCX-0110-20261015-WIN-FG-TRAY-ADMIN/`。

日常EXE/旧隔离四摘要及原TASK字节不变，用户原删除不纳入提交；八门禁、历史query失败/失败时间窗ETW缺口、Mac CPU/RSSfailed、原预算/真实周期和Mac依赖继续开放，TASK/IMP保持in_progress，不关闭、自动合并、签名、晋升或发布。

### 2026-10-15 Windows 用户协助托盘重开、普通退出与管理员真实令牌

用户再次授权前台后，08:06:59核验原普通PID56448/creation/HWND2229478不变，主窗口已从托盘恢复，DWM1→2，产品日志tray/menu show/focus成功；菜单动作由用户完成、工具独立核验。真实设置关闭关窗保留后Sky Alt+F4，持有句柄观察器08:08:32记录exit0，08:08:48本机native0，无强制终止；未把此系统关闭当托盘“退出”菜单通过。

顺序RunAs启动管理员产品PID73048/creation2026-10-15T00:08:50.4343117Z，控制器和实际产品均真实elevated/type2；普通false/type3、EXE摘要相同、显式sandbox配置根相同，启动后偏好回读相同。此为真实双权限产品令牌及本次配置根证据，不替代默认安装目录普通可写/自定义根全矩阵。

普通读取器访问管理员令牌拒绝原件保留，换只读管理员观察器成功；Sky管理员AX仅窗口/标题栏，系统菜单及一次设置定位输入无确认变化。未以Alt+F10无变化判产品失败，也未绕其他UI输入。原生foreground=true而产品日志inactive/solid、DWM1，管理员活动云母待实际激活证据，未归因产品缺陷。已请求用户手动窗口状态/正常退出，绑定HWND只读采样最多240s、持有句柄退出观察最多600s，截止不当产品退出或门禁通过。

08:11:47日常/旧隔离四摘要、干净代码e6及原TASK字节保持；本机WindowsSandbox.exe不存在，仅只读盘点未启用功能/创建VM/执行安装器。详见[托盘重开与双权限令牌](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows-e6托盘重开与双权限令牌/README.md)，原`.adg/evidence/OCX-0110-20261015-WIN-FG-TRAY-ADMIN/`追加证据，退出未来结果另验。管理员生命周期/材质、托盘退出/Snap/物理DPI、其他完整门禁/旧query与性能failed/原预算/Mac依赖不提升，TASK/IMP仍in_progress，不关闭、自动合并、签名、晋升或发布。

随后用户回复“已退出”，08:20:31进程/窗口已无产品，08:21:18保护复核native0、原管理员PID73048及两个只读观察器均不存在，日常/旧隔离四摘要及TASK摘要一致。退出观察器08:19:44先到600s截止，原件timeout=true/exitObserved=false/exitCode=null；只记录用户协助退出后的真实归零，不补造退出码/实际退出时刻，也不覆盖前一历史快照。四份追加原件/方法清单及`verify-administrator-exit.py`核验；15分钟前台窗口到时不再启动实例，Mac原生尚未正式交接，完整门禁与原失败/预算保持。

### 2026-10-15 Windows CI 固定失败制品的单次 ETW

前台结束后后台推进，代码ca26e771只改两workflow与新增准入/离线测试，apps/原夹具/collector三份及预算未变。已有Windows工作流新增query-etw模式，固定bd2原失败run38537166560/artifact11881042603/EXE497814ca…、官方Node24.21.0/TraceEvent3.1.21，核验精确head成功push CI、API digest/ZIP CRC/安全条目/载荷及Node签名。离线10项/真实包3份/PS块10项通过，Windows ZIP反斜杠规范化红例/修正保留，本机原库/Node/ETW/desktop新执行0。

push38660621496及PR38660625647均success，各Mac900/Windows627passed、忽略6/7，各35组，完整日志ZIP及实际checkout分别核验；原API/TLS/运行中日志404失败保留，905.391s观察到时未重派，第二观察169.813s读取原handle真实终态。PR5说明更新最终读回一致，仍draft/open/unmerged，无auto_merge。

准入后只dispatch一次run38662786466/attempt1，query-etw成功/package skipped、无新安装候选。08:56:06–22原失败EXE完整库exit0、530passed/0failed/4ignored、七query通过，**目标失败仍未复现**。根PID9668/creation真实/句柄映像一致、runner elevated/type1；17085事件/5811506bytes，31process start/stop（test1/Node22/cmd-basename8）、940thread start/stop，serialization/kernel/decoder丢失错误0，consumer/writer/自有session stop完整。根热点158572调度/287481IO按原范围省略，不称根全覆盖；不重复求绿或清除历史失败。

artifact11939861138/340221bytes/API摘要6f53dd39…、13份原件及job实际checkout原日志核验，详见[CI固定失败制品ETW](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows-CI固定失败制品ETW/README.md)，`.adg/evidence/OCX-0110-20261015-WIN-CI-ETW/`。失败时间窗ETW/根因仍缺，官方019的profile隔离与安装隔离未解决、Mac原生未交接；本机native0/日常及旧隔离/TASK摘要不变，原性能failed/完整八门禁/预算/周期保持，不关闭、合并、签名、晋升或发布。

### 2026-10-15 ca26 候选提前准备及回放适用边界

只读对照原38537166560日志与38662786466成功回放：测试EXE字节相同，但EXE/Node位置、Temp/HOME/USERPROFILE及原cargo --show-output上下文不同；回放stdout不含成功夹具control阶段原文，31个已保存进程生命周期齐全不等于七场景逐项PID/creation/phase关联已证。环境/负载相同未证明，不归因路径或清除失败；下一失败证据需实际失败当次并审计上下文，不再同类回放求绿。

为下一前台提前只dispatch一次精确ca26 Windows候选run38663871103/attempt1，旧38662786466诊断模式被明确排除。09:09:58实际package进行中/query-etw skipped、conclusion null，尚非候选通过或宿主安装；600秒只读观察到时继续同run，不重派。12份准入前快照及差异方法见[候选准备与回放差异](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-ca26候选后台准备与回放差异/README.md)，`.adg/evidence/OCX-0110-20261015-WIN-CA26-CANDIDATE/`。

本机native0/日常及旧隔离四摘要/TASK原字节保持，无前台/原库/ETW新执行、真实迁移或安装；候选准入后仍等用户新的前台信号。Mac原生、失败时间窗、profile与安装隔离/原性能failed/全部门禁及原预算不提升，TASK/IMP保持in_progress，不关闭、合并、签名、晋升或发布。

### 2026-10-15 ca26 Windows 候选终态与只读载荷核验

原run38663871103于09:22:32读得completed/success，package成功/query-etw skipped。第一608.296s观察到时后仍读同run，第二271.078s取得真实终态，无重派。原始日志ZIP/CRC及20份阶段日志核验实际checkout ca26，包锁回归35组/627passed/0failed/7ignored。

artifact11940418610安装ZIP7627587bytes/SHA2e945d4b…与11940483370证据ZIP1662626bytes/SHAcff231eb…分别匹配API；安全条目/CRC/source/版本/回执通过。MSI6abf1095…与NSIS5162ef6f…在临时runner新装/首屏/卸载成功。本机MSI只读数据库/CAB提取EXE8777216bytes、0.1.10、SHAf43e81a9…，匹配精确MSI载荷回执并含完整ca26源码字面量；未在宿主执行安装器/应用。TLS/curl下载失败保留，核对现存字节后只读补缺，正常证书校验且跨host不带Authorization，无覆盖/重派。

94份最终原件方法清单和独立`verify-candidate.py`通过，见[候选准备与回放差异](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-ca26候选后台准备与回放差异/README.md)的终态段，原`.adg/evidence/OCX-0110-20261015-WIN-CA26-CANDIDATE/`追加。12份历史准备快照不改；下一前台候选就绪，但runner新装不抵本机双权限/安装根与升级卸载数据保留。native0/日常/旧隔离/TASK原摘要保持，仍等新的前台信号，原失败/预算/双平台依赖和八门禁不提升，无关闭/合并/签名/晋升/发布。

### 2026-10-15 ca26 备份前台只读预检

缺少新前台信号时仅后台准备启动/身份/保护方法：09:38:48精确ca26载荷、代码/defaults/token-reader只读准入，普通harness false/type3，native0/拟用runtime不存在/日常及旧隔离四摘要保持。默认review-only不建目录/写偏好或启动；显式启动需要当前source/备份树表scope及未过期的人类新信号，JSON字段不替代宿主的人类授权判定。

内存11输入检查（1正/10负）通过，首轮OrderedDictionary.Clone方法错误及空输出/说明保留、修正后通过，无产品/原生门禁推定。四方法语法通过；主HWND/creation/UTCticks修正沿用，phase文件名有界，未来实际启动先留identity-pending再核验真实产品token，拒绝第二实例或覆盖未知runtime。本轮无应用/备份/恢复/固定/清理/计时器/原库/ETW新执行，也未写合成授权文件。

详见[ca26备份前台预检](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-ca26备份前台预检/README.md)，`.adg/evidence/OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/`。下一段需新前台信号，profile/安装隔离、失败时间窗、Mac及全部剩余门禁/原预算仍开放；TASK/IMP in_progress，不关闭、合并、签名、晋升或发布。

### 2026-10-15 ca26 Windows 备份恢复、轮换与本地事件前台子集

用户09:43:50真实授权30分钟前台，先核验已准入ca26候选/EXE f43e81a9…，新隔离根启动普通PID18460/creation09:45:25.4876884、产品token false/type3。真实UI生成并固定备份，主题dark→light→恢复dark，active偏好与原payload逐字节相同；light恢复前保护v2/精确关联/committed，用户pin独立保持。全程Sky输入，CDP仅配置隔离profile，未改变物理DPI。

真实UI额外生成12份后共14份，新鲜备份超过10仍无可清理项。明确仅调整隔离manifest created_at的年龄夹具后，手动预览/确认恰删两份60/61天普通项，固定90天项保留；保存automatic策略本身不删，新备份成功才自动轮换一份掉出最近10的60天项，最后UI保存回manual。原件/前后摘要保留，不把合成年龄当真实30天、stable24h/beta6h或错误退避证据。

同大小/同JSON语义但摘要损坏的隔离备份被真实恢复拒绝，active preferences未改；失败前保护保持active。无关备份成功不解除恢复失败，修复原payload并UI重试同目标后，同notification_id/Local restore精确identity于10:00:30解除，仍未读未删，无关runtime警告未解除。新成功保护committed，旧失败guard仍active，不宣称新尝试清理旧事务；管理器stable/beta与全领域事件门禁不提升。

生产树表空态/默认层级、文件叶节点、空格收起/Enter展开、刷新后展开状态保留有原生截图/AX；未量化像素、完整焦点/全主题密度缩放或原型复选框，浅色切换截图不替代浅色完整树表验收。首实例10:02:05持有句柄观察exit0，native0后10:02:33顺序同根重启PID66488/普通token false/type3；15份备份所有文件字节/摘要、dark偏好与已解除通知状态持久。真实取消固定再固定不立即删除，最终仍15份。

第二实例10:04:59正常exit0，无强杀；10:05:29本机native0，早于10:13:50窗口截止结束前台。日常EXE/旧隔离四摘要/TASK原字节保持。原授权UTC类型误转换、损坏夹具末尾LF假设及离线verifier初始AX结构假设的方法错误均保留并修正；补10项纯离线时间类型回归，不延长授权或重启应用。582项独立清单、18快照/193次有效payload摘要及两次预期损坏、新旧verifier通过，见[备份恢复轮换与事件实测](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-ca26备份恢复轮换与事件/README.md)及原证据目录新增foreground清单；旧17项预检清单不改。

只完成本段备份与本地事件子集。管理员完整材质/窗口、托盘退出/Snap/物理DPI、默认安装data与自定义根及升级卸载保留、真实更新详情/进度/重启、备份全失败矩阵/完整树表、事件通道隔离、最终源码真实30分钟退避/周期/配对性能仍开放；019 profile与安装隔离、bd2失败时间窗ETW/原查询失败、Mac CPU/RSSfailed及原预算/Mac交接依赖保持。八项required_checks与TASKrevision2/in_progress/pending不改，不关闭、合并、签名、晋升或发布；后续原生需新的用户前台窗口。

### 2026-10-15 Windows 独立测试 Profile 只读准入

前台结束后继续后台，493da806在原Windows工具分支新增Profile只读准入/31项纯合同，并接入既有Windows smoke job；相对ca26仅两份test/smoke与ci.yml，apps/原查询夹具/collector/预算未改。本机31项与原有9项令牌/语法检查通过，提交前测试父HEAD ca26如实保留，以最终方法摘要与493三文件blob绑定；10:24:50提交后真实日常账户负向探针被拒绝，未启动官方019或新候选。

工具按真实令牌/SID/本地账户/线程状态、Profile及两种AppData、现有目录与全祖先重解析边界、本机native0准入；日常/错误账户、提升或受限管理员、环境路径假象、相邻同前缀/点段/设备路径及未知状态拒绝。回执只存路径/SID摘要，不创建账户或改变绑定/环境/ACL。合成正例不当实际新账户验收，Profile通过不代表前台授权或机器级安装隔离；此前false/type3产品普通模式结论保持独立。

493 push38670771363 / PR38670774817于10:29:53两轮均有Windows smoke/frontend/release-tools/Mac backend成功，Windows backend仍in_progress，旧ca26绿不代新head。草稿PR5元数据按当前工具范围更新并独立读回，仍head493/basebd2/open/draft/unmerged/无auto_merge。连接器权限/传输与本机TLS只读失败保留，未关闭证书校验/重派CI或重复原库。

见[独立Profile准入](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows独立Profile准入/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-PROFILE-ADMISSION/`。当前ocx0110_accept不存在，实际正向Profile/官方019配对需用户准备独立普通账户并首次登录、后续新前台窗口；ETW批准不含账户创建，安装还需独立Windows环境，不能仅换用户或目录。10:31:32本机native0，日常/ca26/旧隔离/TASK七摘要匹配；原TASK字节/八门禁/失败/全部预算/周期与Mac依赖保持，工具准备不作完整门禁通过，不关闭、合并、签名、晋升或发布。

随后读取原push38670771363/PR38670774817于10:42取得completed/success，各五必需job成功/main-only build skipped，未重派。两份完整日志ZIP/CRC及精确checkout/test/Profile合同原文核验：push实际493da806、PR实际23e55c8d，整tree均cd5f3dcf；每轮Mac900/Windows627passed、忽略6/7，各35组；Profile纯合同各31项，方法摘要匹配源码checkout字节。新`.adg/evidence/OCX-0110-20261015-WIN-PROFILE-CI/`独立冻结31项，verify-ci与旧25项verifier均通过；PR5终态说明更新并读回仍draft/open/unmerged/无auto_merge，head/base不变。Git TLS断连后仅命令级兼容参数完成原493/证据1b5推送与远端核对，无证书/全局配置改变。本机native0，真实新Profile/019配对/安装环境、失败时间窗ETW、全部原生/周期/预算及完整版本门禁仍待验，不关闭或发布。

### 2026-10-15 493 精确 Windows 候选与 beta 前置来源

旧ca26候选不抵新head，后台先核验493精确成功pushCI/干净worktree/远端一致/无已有候选，单次dispatch run38673288645/attempt1，仅candidate模式/query-etw skipped。第一633.657s观察结束仍在构建，继续同run第二387.735s取得11:10:02 completed/success，未重派。完整日志ZIP/CRC及20份阶段日志确认实际checkout493、包锁回归35组/627passed/0failed/7ignored。

artifact11945931457安装ZIP7626883bytes/SHA921c52cf…与11944729636证据ZIP1688746bytes/SHA77ee9a35…分别匹配API；安全条目/source/版本/回执通过。MSI3db2d45a…与NSIS09ace38e…在临时runner新装/首屏/卸载exit0；宿主仅只读MSI/CAB提取，主EXE8777216bytes/0.1.10/SHA9ca21e30…匹配精确MSI载荷并含完整493来源，未启动。裸构建/MSI/NSIS EXE摘要不同，分别核验，不混用。

10:59:54–56按493冻结defaults读取已认证beta Release API与公开beta/latest.json均404；只证明当前来源缺失，不作产品检查或beta6h/退避证据，不发布或改来源。日志curl与.NET读取失败和缺失原stderr边界保留，独立有界urllib取回同run原ZIP；无跨host凭据转发、证书放宽、覆盖失败或重跑。94项独立清单/verify-candidate通过，见[493精确Windows候选](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-493精确Windows候选/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-493-CANDIDATE/`。

11:10:40本机native0/日常EXE/旧隔离四摘要/TASK原字节保持，代码493干净；候选已就绪但后续原生仍需新人类前台窗口。独立普通账户/安装环境/beta有效来源、失败时间窗ETW/原查询失败、Mac性能failed/全部预算及剩余八门禁仍开放；不将ca26/e6历史原生重标为493，不关闭、合并、签名、晋升或发布。

### 2026-10-15 493 Windows 更新通道与树表前台子集

用户新“开始前台验证”11:30:22.806起30分钟，截止12:00:22.806。精确493/MSI载荷SHA9ca21e30…新隔离根启动普通PID4092/creation11:32:12.6267866、实际token false/type3；全程Sky输入、CDP仅隔离profile。管理器详情→全部更新键盘往返及切换后Tab仍在弹窗有截图，AX只报告容器，不冒充完整焦点矩阵。stable真实手动检查11:36:29成功；11:36:36才关闭，故只证明完成终态跨路由复用，不能称关闭时仍在途或后台继续完成。当前0.1.10/无候选/安装禁用，不虚构说明、签名或下载进度。

真实UI切beta后panel调度/stable缓存/通知字节保持；未据此推定绝对零请求。beta实际手动检查失败，manager-beta failures1与panel第二次手动失败落盘；回stable不再检查，完整schedule不变，失败/退避未被清除。持久通知只有runtime-not-found且未解除，不能称已测beta持久事件精确解除、panel在途事务或全部通道隔离。自动检查始终关闭，nextDue状态不当真实24h/6h或状态错误30分钟退避。

真实UI生成备份bk_20261015034010_ebc5cff3，空态/默认root-year展开month收起、文件叶节点、Space/Enter/Home/End/Down、刷新展开保留及刷新按钮后Tab入根有证据；浅深两套全展开树截图和普通最大化client1728×981/还原原外框核验。未量化像素、后台刷新同叶焦点、复选框原型/全档位密度缩放，物理DPI按用户指示跳过，remoteSession1/nativeDpi192不计物理200%。

首实例11:44:47持有句柄正常exit0，输入返回后捕获无PID不当产品崩溃；native0后11:45:11同根顺序重启PID77760/普通token false/type3，偏好/调度/stable缓存/备份五文件逐字节保持、浅色/stable/自动false持久。重启Sky两次FrameArrived timeout，按恢复规则重选唯一HWND2950032仍失败后停输入；只读token/HWND/日志不替代重启UI截图。已请用户正常关闭；11:51:32退出观察器300s截止exitObserved=false/exitCode=null，11:51:53独立确认同PID仍native1，没有强杀或新实例/Mac交接，不宣称已退出。

初次快照误读持有lock产生PermissionError，空目录/原方法保留，改为JSON自有状态另phase；非产品失败。209项独立清单/45Sky状态/41输入/9快照84次摘要复核与verify-foreground通过，明确断言checkpoint native1及全部局限，见[493前台更新通道与树表](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-493前台更新通道与树表/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-FG-493-UPDATES-TREE/`。日常EXE/旧隔离四摘要/原TASK字节保持，代码493干净；后续退出另追加原件，不覆盖旧checkpoint。原失败ETW/查询失败/MacCPU-RSSfailed、完整八门禁/预算/真实周期/安装与Profile环境/FZ50依赖均不提升，TASKrevision2/in_progress/pending不改，不关闭、合并、签名、晋升或发布。

提交后索引全字节核对发现basename排除输出名使九份嵌套config-migrations/manifest.json未直接列入旧209项；它们均已提交并由snapshot摘要复核，原件无损。旧清单/方法/结果保持，另增完整222项清单，仅排除自身两个根输出，含旧清单与九份嵌套文件；verify-complete-archive全字节及隔离旧文件集回放通过，未执行产品/新UI。此为归档覆盖方法补强，不改变原checkpoint native1/重启未观察退出或完整门禁结果。

随后用户“已退出”，11:59:20初次工具枚举native0（另存明确转录），12:00:50–51独立只读保护核对前后再次native0，原PID77760不再存在。旧观察器已11:51:32超时，实际退出时刻/exitCode仍null，不补造exit0或覆盖11:51:53 native1历史。没有新启动/强杀/前台输入或授权延长。日常/候选/旧隔离四摘要/TASK原3878字节SHA bc7965b4…保持、代码493干净；本轮偏好/调度/stable缓存/备份manifest与payload五文件与重启后逐字节一致。独立`.adg/evidence/OCX-0110-20261015-WIN-FG-493-EXIT/`11项与verify-exit通过，旧222项重验通过，原归档不改。本轮进程收尾已完成，Mac未正式原生交接；完整门禁/失败/预算/安装Profile来源等依赖保持，不关闭、合并、签名、晋升或发布。

### 2026-10-15 493 新前台继续、目录打开与管理员边界

用户直接“我批准继续”按新人类前台信号处理，无需精确口令；窗口12:38:46–13:08:46，未新增账户/安装或发布授权。普通PID49492/creation12:40:07.2524141、false/type3，同493/EXE9ca21e30…复用已退出隔离根，启动前五持久文件与保护摘要重核。真实目录/文件打开警告及取消、右键进入子层展开/左键回父层收起通过；再次确认根目录实际打开系统backups窗口，自有层级/2026目录核对后关闭，仅保留自有路径见证、不归档无关Explorer侧栏。文件实际编辑器打开/越界逃逸全矩阵未验。

普通最小化/恢复同PID-HWND，IsIconic与前台状态/DWM2→1→2、外框恢复一致；12:46:39持有句柄正常exit0，退出后捕获无PID不当崩溃。native0后12:47:46顺序管理员PID78512/creation12:47:46.0681912，控制器74792及实际产品true/type2，同精确EXE/显式data根。首屏截图可见、AX仅标题栏；旧普通helper前置拒绝未输入，改显式新管理员观察的一次设置坐标点击返回但无确认导航，不判产品设置失败或通过，也不用CDP/其他注入绕过。

请用户手动窗口操作/正常退出后，提升只读采样器78676限定原PID/creation/HWND最长240s。12:54:15终态240.298s/240行，均foregroundtrue/backdrop1、无max/min，产品仅inactive/solid日志；原生前台与产品材质不一致保留待验，无mica/人工动作通过推断。活采样文件独占PermissionError保留，待原writer关闭才解析，无重启/改共享；普通CIM提升映像null如实保留，以提升原件提供身份。

12:55:45检查点仍native1/PID78512，退出控制器74792活着/暂无终态/exitCode未知，未强杀/新实例或Mac交接。117项独立完成清单/analyze-session/verify-checkpoint通过，明确排除控制器稍后两个确切终态/错误输出，未来需另验；三普通快照11文件一致、日常/旧隔离/TASK原摘要及代码493干净保持，见[493继续前台与管理员边界](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-493继续前台与管理员边界/README.md)和`.adg/evidence/OCX-0110-20261015-WIN-FG-493-CONTINUE/`。本段不替代管理员完整窗口/活动材质、DPI/Snap/托盘、安装data/升级卸载、真实更新进度/事件、原失败ETW/查询失败、Mac性能failed/原预算与真实周期/Profile安装环境/FZ50依赖；八门禁/TASKrevision2/in_progress/pending不改，不关闭、合并、签名、晋升或发布。

随后用户确认“最大化最小化都没有问题”并附原图，按人工操作通过记录，不要求重复；原图3437×1885/SHA7b296a6e…只证明可见窗口状态，不单独证明令牌/PID、最小化瞬间或精确时间。旧采样已12:54结束，不否定后续人工反馈。13:06:26提升只读补读仍同管理员PID78512/creation/HWND8786154/true-type2，当前已还原原客户区、foregroundfalse/backdrop1；新同根日志另补12:58:31–13:00:18五次active/mica/dwmSome0及其间失焦回退，不冒充同步DWM2采样或完整管理员UI通过，旧不一致时间窗仍保留。

900s原退出观察器13:02:46超时，补读持有句柄120s观察13:08:26也超时，均exitObservedfalse/exitCodenull；13:08:43独立检查点native1，未强杀、无第二产品/正式Mac交接，前台13:08:46结束不延长。两次PowerShell5默认ANSI在访问应用前解析中文JSON失败原件保留，复用原PowerShell7成功；离线日期等价时区比较方法修正不改原回执。新增`.adg/evidence/OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW/`38项独立清单/verify-manual-window与旧117项重验通过；11份JSON中9份字节保持，两份仅管理员启动运行时间戳变化；日常/旧隔离/TASK摘要、代码493干净保持。后续正常关闭/归零需另补证，完整八门禁/历史失败/预算/周期与依赖均不提升，不关闭或发布。

### 2026-10-15 Windows 原失败窗口离线对齐与解读边界

前台结束后只后台读取11份显式输入，原摘要核对后重算5轮库/35条夹具时间线：原bd2PR七组5.051–5.657s observer结束仍缺control标记；38572890687及e6PR各三个version失败在query返回后才出现标记，距实际创建分别4.182–4.252s、3.213–3.334s，早于约5s observer。两成功push的七组标记均在query返回前已可读。保留各自时间窗和失败，不把后续control进展解释成查询通过；按内嵌时间/创建FILETIME/elapsed对齐，不用缓冲输出的GitHub日志前缀作执行时刻。

原成功CI ETW3175条scheduled-out均CSwitchversion5，其中402条waitMode没有固定3.1.21枚举名称；原数值保留，不把96等重映射为0/1。只读固定Microsoft源码getter/枚举及公开Thread_V2文档后明确version5等待语义未验证，不据枚举标签归因。7个control的标记文件写事件只证成功对照进展。下一失败采集需覆盖自有创建至原5s observer，保留进程代际/未知字段，不自动重复成功回放、预热或放宽预算。

新增`.adg/evidence/OCX-0110-20261015-WIN-FAILURE-WINDOW-REPLAY/`8项清单/11项输入摘要及两离线verifier通过，019dd6c2旧35项重验保持成功对照边界；未改产品/采集器/工作流或启动Node/原库/ETW。13:22:54仍同管理员PID78512/native1/exit未知，日常/候选/旧隔离/TASK摘要及493干净保持；未正式Mac原生交接。详见[失败时间线离线复核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows失败时间线离线复核/README.md)。实际失败ETW/根因、全部八门禁/预算/真实周期及依赖保持开放，不关闭或发布。

### 2026-10-15 Windows 共享树表后台几何执行与前台依赖

原生前台结束后复用既有geometry入口，在Windows内置Node24.19/playwright-core1.62.1/已安装Edge154.0.4258.62无头运行，13:27:55exit0/8组通过/stderr空/finally关闭browser-server；未改官方查询Node24.21、安装包或源码。浅深×四CSSzoom共40行/120控件，名称首行与复选框/箭头/图标/操作中心最大差0px、用途首行差0px、操作高度缩放前≥34px；24多行场景，半选/已选/禁用/无框均覆盖。仅生产组件浏览器几何，不替代原生WebView/物理DPI/全档位密度/原型及焦点矩阵。17项独立清单/verify-browser-evidence通过，9份方法组件样式锁文件摘要绑定493干净代码，见[后台几何复核](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows树表后台几何复核/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-TREE-BROWSER/`。

13:31:11仍原管理员PID78512/native1/exit未知，连续此前13:08:43、13:22:54同状态；前台13:08:46已到期，自动续办不延长授权。当前需用户正常关闭后独立归零，不能重启原实例/强杀/开第二实例或正式Mac原生交接。独立普通账户及WindowsSandbox.exe仍不存在，019配对需真实Profile/前台，宿主升级卸载需独立环境。日常/旧隔离/TASK摘要保持；原失败ETW、完整门禁/预算/周期继续开放，不关闭或发布。

### 2026-10-15 用户退出后管理员实例归零

用户“我已退出”后，14:48初次只读枚举无产品进程，14:50:09–11独立保护核对前后均native0；原管理员PID78512、控制器74792及观察器79860均不存在。实例占用解除，实际退出时间/exitCode仍null，因两观察器先前已超时，不补造正常exit0或覆盖13:31仍live的历史。用户退出不延长已结束前台授权，无新启动/强杀/安装/账户或正式Mac原生交接。

11份隔离JSON与管理员13:07:35快照逐字节一致，日常/候选/旧隔离/TASK原摘要及代码493干净保持。新`.adg/evidence/OCX-0110-20261015-WIN-FG-493-ADMIN-EXIT/`20项清单/verify-exit通过，旧117/38/17项三归档独立重验通过；TASKrevision2/in_progress/pending/八检查不改。首次fetchTLS失败后命令级schannel读取与fetch成功，远端无新差异。见[管理员退出追加证据](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-493继续前台与管理员边界/README.md)。

下一步仍优先实际失败窗口诊断，不能重复成功对照清除原失败；原生剩余矩阵需新前台窗口，官方019配对与安装保留需要真实独立Profile/独立Windows环境。完整门禁、历史失败、Mac性能failed、原预算及真实周期保持，未关闭、合并、签名、晋升或发布。

### 2026-10-15 新目标首轮：原构建后与系统临时盘单次取证

8e266aba只改6份test/工作流，apps/desktop整树仍bf4365586d与原bd2相同；新增仅CI-dispatch/attempt1的post-build-runner-temp模式，在常规构建且无测试执行后，固定原失败EXE497814ca/官方Node24.21.0-ba4e6d11单次完整原库，使用系统TEMP盘独占子目录、保留临时runner用户配置。保持2/4/5/8秒、120秒外层、默认并发/capture、64MiB及队列/IRP预算；作业45分钟仅供构建准备。无额外预热/人工压力/宿主环境改动。新增等待字段原解码数值，不宣称version5语义已确认。

本机11项准入/101项采集器纯合同、10脚本/11工作流块解析通过；同SHA push38699864356/PR38699869579全部成功，各Mac900/Windows627passed、忽略6/7，各35组。完整原日志/实际checkout核验：push8e266aba、PR02de6ed6整tree73c4af0f相同，不推断包字节相同。原失败制品API未过期/摘要、固定EXE/Node/解码DLL及签名独立重核。

单次dispatch38702455024/attempt1成功，package跳过；15:59:51–16:04:45常规构建，16:04:51–16:05:53原库530passed/0failed/4ignored/exit0、七query通过。根PID6208/creation08:04:51.4337703Z、runner真token type1，31进程/957线程开始结束，16865条ETW/6089687bytes-SHAb9a04471…，丢弃/内核丢失/解码错误/已知覆盖缺口0、排空/自有STOP0、未强制清理；根高频省略按原scope计数。artifact11957286829/351969bytes-SHA055eeed1…匹配API，14包内原件与原job日志核验。

该次仍成功对照，目标失败/实际失败窗口ETW及根因未取得，原失败不撤销，也不据成功排除任一环境原因；不自动再录相同条件，下一取证方向为普通CI首次回归的自然失败留证。134项独立清单/verify-evidence通过，PR5终态说明读回仍草稿/未合并；16:09本机native0、日常/旧隔离/TASK摘要及8e266aba干净保持，未新候选/前台/安装/账户或正式Mac交接。见[原构建后单次诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows原构建后单次ETW诊断/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-POSTBUILD-ETW/`。完整八门禁、原预算/周期/Macfailed与环境依赖保持，不关闭或发布。

### 2026-10-15 普通 CI 首次库回归同步留证

43142bdb只改7份test/ci.yml，apps/desktop与原bd2整tree保持；仅诊断分支普通push/PR-attempt1经固定失败参照/官方Node/解码器准入后，用当次输出目录的Cargo runner在正常编译库首次执行时同步录制，库每作业一次、原退出码传回Cargo，其他套件正常执行。--show-output、capture、默认并发、2/4/5/8秒、120秒外层和64MiB/队列/IRP预算不改。执行为当次编译EXE而非历史失败EXE，分别保存身份；临时runner用户配置保留/TEMP独占子目录与工具开销显式记录，不声称完全重建旧环境。此普通CI本身是新路径验证，不伪造先前同SHA成功准入。

本机12项准入/108项采集器纯检查/脚本与TOML数组回读通过，无本机Cargo/原库/固定Node/ETW。push38705201081/PR38705207794完整success，各Mac900/Windows627passed、忽略6/7，各35组；实际checkout43142bdb/485372d7整treeccb9f9ab相同。两轮各首次库530passed/0failed/4ignored/exit0、七query通过，EXE SHA874cd89c…/4e86d7b2…分别核验。自有ETW18732条-6787959bytes与16128条-5825028bytes，每轮31进程开始结束/948对线程，丢弃/内核丢失/解码错误/覆盖缺口0，排空/自有STOP0/未强制清理；根热点省略边界保持。

已验证首次回归同步留证能力，但当前失败及历史固定制品失败窗口ETW均未取得，根因未知；原失败不撤销、不自动追加重跑。118项清单/verify-evidence、制品API摘要/原日志/原字节核验通过；ETW压缩保留并验证解压原字节。16:45本机native0、日常/旧隔离/TASK摘要及43142bdb干净保持，PR5说明读回仍草稿/未合并。无新桌面候选，旧493 UI不重标；未前台/安装/账户或正式Mac交接。见[首次CI同步留证](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows首次CI回归同步留证/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-FIRST-CI-ETW/`。原完整八门禁、预算、周期、Macfailed及环境依赖保持，不关闭或发布。

### 2026-10-15 431 阶段候选、旧计时复用与专项环境方案

普通CI同SHA成功/实际checkout/制品链核对后单次candidate run38709701621，17:14:45 completed/success、query-etw跳过；实际检出43142bdb，包锁35组/627passed/0failed/7ignored。MSI/NSIS临时runner新装/精确首屏/卸载exit0；安装ZIPartifact11960342769/7626479bytes-SHA5c91c10a…与runner证据ZIP均API摘要/CRC/安全条目通过。宿主只读MSI/CAB提取8777216bytes EXE，版本0.1.10、完整431来源、SHA3be7ea60…匹配MSI载荷，未启动产品/安装器，不抵宿主升级保留或旧493原生证据。

独立复核8份旧manifest原件，保留stable/panel跨启动真实success间隔86564/86566秒及next_due24h、旧timer floor1800001.5ms事实；前端scheduler blob相同但后端两个blob已变，旧执行不冒充新版完整通过，只补受影响路径/直接触发/原始计数/睡眠恢复。17:02公开beta元数据404，API首读TLS错误保留，17:05认证API才实际404，beta成功6h基线仍缺；不发布或改源制造条件。

只读环境确认Hyper-V可见/服务运行，但Get-VM报VirtualizationException、VM可用性未知，拟用普通账户/WindowsSandbox.exe不存在、空闲内存约5GiB。已写可审阅的独立普通Profile及专用Windows安装环境方案，账户/VM/ACL/网络均未创建改变，需明确确认后实施；安装VM不替代宿主性能。17:16:02本机native0/日常/旧隔离/TASK摘要/431干净保持，无Mac正式原生交接。52项清单/verify-readiness通过，详见[候选与专项环境准备](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows候选与专项环境准备/README.md)及其中执行方案，证据`.adg/evidence/OCX-0110-20261015-WIN-431-READINESS/`。原失败ETW/根因、八门禁/全部预算/Macfailed/周期缺口保持，未关闭或发布。

### 2026-10-15 固定431候选的独立Windows安装器文件保留

账户/VM等待期间独立补安装器专项：工具3c51367f只改test/CI、产品tree不变，被测仍固定431候选run38709701621，不重贴源码身份。本机6项离线准入/宿主拒绝/语法通过，同SHA push38713455586/PR38713465220完整success，各Mac900/Windows627passed、忽略6/7、35组；实际push3c/PR6f92618e树相同、日志原字节核验。

单次安装专项run38716371739/attempt1成功、package/query-etw跳过；MSI/NSIS各独立全新runner实际安装官方019后，升级431、默认卸载、重装再卸载，每种5安装步骤exit0、6夹具文件×5读回阶段全部字节保持、两次卸载主EXE实际移除。MSI原日志确认旧ProductCode被FindRelatedProducts识别并RemoveExistingProducts成功。固定官方包/API摘要、431包及安装EXE/MSI3be7ea60…/NSIS014fae29…分别绑定核验。

未启动桌面应用；安装目录data、自有兄弟custom-root和哨兵为seeded夹具，结果只证明安装器文件保留，不代替应用根绑定/普通权限/不可写/旧AppData迁移/显式清理或完整installer门禁。两artifact原件/原job实际checkout/安装日志、139项清单/verify-evidence通过。17:59:15本机native0、日常/旧隔离/TASK摘要与3c工具干净保持，未账户/VM/ACL改变或正式Mac交接，PR5终态说明读回仍草稿/未合并。见[升级默认卸载保留实测](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows固定候选升级卸载保留/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-INSTALLER-RETENTION/`。原失败ETW/根因、Macfailed、全部预算/周期/来源/环境依赖保持，不关闭或发布。

### 2026-10-15 启动根恢复的源码接线发现

后台只读审阅3c/固定431相同的7份产品Git文件，确认`WIN-0110-STARTUP-ROOT-RECOVERY`仍开放：setup中根锁/初始化错误通过`?`离开，发生在SharedDataRootAnchor注册前；设置切根依赖该状态，当前调用点没有失败启动的显式根选择恢复。锁定Tauri2.12.1官方源码确认setup在Ready执行、错误映射后panic；初稿将其归于build.expect的推断已更正保留。此发现对应原第4节不可写提示/选择根合同，尚无普通账户真机故障、实际报错UI或修复通过结论。

下一步优先实现初始化服务前的显式根恢复及合法选择持久化，取消/失败保留旧绑定，损坏/过新/锁冲突/未决迁移不得静默回退；随后同SHA普通拒绝写/选根重启/双权限同根复验。10项归档清单/verify-review只证明审阅来源，不是产品测试；18:16:08native0/3c干净/日常旧隔离TASK摘要保持，无新执行或系统配置改动。见[启动根恢复源码缺口](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows启动根恢复源码缺口/README.md)及`.adg/evidence/OCX-0110-20261015-WIN-STARTUP-RECOVERY-REVIEW/`。此前安装器保留子集继续有效，storage/installer与原完整八门禁、历史失败、预算/周期/Mac依赖未关闭，不发布。

### 2026-10-15 Windows 新鲜默认根的显式恢复与首次回归

代码 `feature/0.1.10-windows-query-etw@a2e4ea17` 已推送并读回。在业务服务/WebView 初始化前，只有未配置的新鲜默认根发生根锁准入原生写权限拒绝才提示系统选根；锁内验证后持久保存当前用户 HKCU 启动指针并重启，取消退出。非法/损坏/未来根、迁移和实例冲突不转为回退，已选根丢失/损坏不重建，沙箱绕过个人选择。其他初始化错误及既有不可写根保持安全停止，完整不可写矩阵、真实选根/重启和同 SID 普通/管理员同根仍待验，不因本接线修复宣布 storage 通过。

首轮枚举漏配、Windows verbatim 跳转夹具失败、c9 PR 原120秒外层清理和审计夹具 Clippy 失败全部保留。审计历史改为一次准备同样5004行，再真实持久追加末行；生产审计代码逐字节保持、12个原断言保留且新增清理前行数断言，不替代持续写性能。a2 push38726727330/PR38726732452完整成功，每轮Mac918/Windows643passed、忽略6/7、35组，fmt/Clippy/前端/最终EXE沙箱冒烟通过；实际checkout a2/11f68e17完整tree相同，未假定EXE字节相同。两首次库546passed/4ignored/exit0、七query通过，ETW16613/17830条无丢失/解码错误/已知覆盖缺口或强制清理，原并发、2/4/5/8秒、120秒及采集预算不改；这仍不是bd2实际查询失败窗口，根因未知。

304项清单/独立verifier通过，新证据目录以-text保持原字节并核对Git对象；19:50:19本机native0、日常/旧隔离/TASK摘要与a2干净保持，PR5仍草稿未合并；没有账户/VM/ACL改动或正式Mac原生交接。仅请求一次同提交候选38729632145，已读回构建中，后续沿用句柄而非重发；旧431不证明新代码。详见[启动根显式恢复修复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows启动根显式恢复修复/README.md)、`.adg/evidence/OCX-0110-20261015-WIN-STARTUP-ROOT-FIX/`及候选专档。八门禁、历史失败、Mac CPU/RSS、全部性能预算/真实周期/来源与环境依赖保持开放，TASK revision2/in_progress/pending不改；未关闭、合并、签名、晋升或发布。

### 2026-10-15 a2 匹配候选就绪，只读载荷已核

单次候选38729632145现已成功，实际checkout a2，包锁35组/643passed/7ignored；新runner MSI/NSIS安装、隔离首屏及卸载子集通过，两artifact API摘要/ZIP CRC和分别记录的实装EXE摘要匹配。本机只下载和读取MSI数据库/提取CAB，未执行安装器或启动应用；MSI载荷0.1.10/8807936bytes/SHA4295367e…，含a2完整构建来源字串，路径固定在该worktree的`test/software/candidate-a2e4ea-38729632145/msi-readonly/`，不复用旧431或混称NSIS载荷。

`.adg/evidence/OCX-0110-20261015-WIN-A2-CANDIDATE/`42项清单/独立verify-candidate通过；20:14:02本机native0、日常/旧隔离/TASK摘要及a2干净保持，没有账户/VM/ACL/FZ50或Mac原生交接。该子集不替代本机普通/管理员/不可写恢复/根绑定、升级保留、019原预算配对、周期或历史失败窗口；继续保留原依赖及单独授权，八检查/TASK/发布状态不提升。详见同一[修复与候选记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows启动根显式恢复修复/README.md)。

### 2026-10-15 换安装位置的原 anchor 持久登记

审阅发现a2正常可写安装data首次启动未保存指针，后续换安装位置仍可能选新空根，与第4节不隐式改根冲突。02a0d4a8已修复：真实根锁准入后、业务服务前登记canonical anchor，后续沿用其自定义活动根引用及外置HOME；既有指针不重写，登记失败停止/释放锁，写入结果不确定时下次回读、不自动复位。新增5项测试使用临时指针和真实解析/锁/引用实现，不冒充HKCU或原生迁移验收。

push38734133095/PR38734138996完整success，每轮Mac923/Windows648passed、忽略6/7、35组，fmt/Clippy/前端及最终EXE沙箱冒烟通过；实际checkout02/ea03b2eb整tree相同。整轮ZIP传输失败后逐job取得四份原日志并独立核验；两artifact摘要/CRC/载荷原件核对，未合成归档。两首次库551passed/4ignored/exit0、七query通过，ETW16564/15849条无丢失/解码错误/已知覆盖缺口或强制清理，原预算不改；历史查询失败窗口和根因仍缺。

95项清单/verifier通过；20:55:29native0、日常/旧隔离/TASK摘要与02干净、PR5草稿未合并保持。只读TokenUser伴随方法4项自身/受控PowerShell子进程检查通过，初版时区比较失败及UTC ticks修正分析保留；没有观察产品SID或证明双权限。专项方案明确非管理员性能Profile与同SID过滤/提升令牌分别取证，不创建账户/VM或扩大授权。仅派发一次02候选38737628201并读回构建中，旧a2不重贴；详见同一[修复记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows启动根显式恢复修复/README.md)、`.adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/`及02候选专档。完整八门禁、原性能/周期、Macfailed、普通/管理员真实根及独立环境依赖保持，TASKrevision2/in_progress/pending不变，不关闭或发布。

02候选38737628201现已单次成功，包锁35组/648passed/7ignored、新runner MSI/NSIS安装首屏卸载子集通过；两artifact API摘要/CRC/实装EXE核验，本机只读提取MSI载荷0.1.10/8808448bytes/SHA7383c275…并静态核对02来源字串，路径固定`test/software/candidate-02a0d4-38737628201/msi-readonly/`。`.adg/evidence/OCX-0110-20261015-WIN-02-CANDIDATE/`42项清单/verifier与package job原日志核对通过；读取传输TLS失败保留为传输问题，未关证书校验、全局改网或重发构建。21:05:40native0、日常/旧隔离/TASK摘要及02干净保持，未宿主安装/启动。该候选子集不证明真实HKCU/双令牌/换安装位置、019配对、周期或历史失败根因；原八门禁与账户/专项环境/前台依赖、Macfailed保持，未关闭或发布。

### 2026-10-15 普通测试身份的离线准备包

代码/候选仍02a0d4a8/38737628201，未新增产品修改。自有`test/software/acceptance-kit-02a0d4-20261015/`准备663份文件/273438730bytes：官方019及精确0110主程序/ocxd、只读身份方法与独立PowerShell7.6.5副本；排除个人配置/profile，未移交测试Profile、安装运行时、创建账户、改PATH/ACL或启动产品。工具包主清单96a75694…独立绑定，逐文件摘要与签名核对；冷/热定义及采集方法仍须按原口径绑定，包准备不等于测量已完成。

复制运行时首次脚本策略拒绝原件保留，后续只对子进程指定RemoteSigned，父scope前后相同、用户/系统策略不改；日常账户/错误清单/篡改载荷3项拒绝检查通过，没有产品运行或普通Profile准入。`.adg/evidence/OCX-0110-20261015-WIN-ACCEPTANCE-KIT/`19项清单/verify-kit通过；21:20:35native0、账户仍不存在、日常/旧隔离/TASK摘要及02干净保持。具体移交步骤见[专项环境方案](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows候选与专项环境准备/执行方案.md)。账户批准/用户首次登录、同SID双权限环境及新前台窗口依赖保持；原失败ETW/Macfailed/全部预算和八门禁未提升，未关闭/合并/发布。

### 2026-10-15 取消为配置隔离新增账户的依赖

用户确认 Windows 版尚未投入使用，允许清理 OpenCodex Desktop 自身配置。执行方案改用当前真实 Windows 账户，不再等待新增账户、口令和首次登录；原独立账户工具包及拒绝结果保留原身份。重置范围限管理器自身配置/测试根，不涉及 Codex 会话、凭据、项目和仓库；现有配置暂留作迁移夹具，按实际验证步骤处理。

21:51:39只读核对当前控制器未提升/type3、native0、HKCU启动指针不存在；原工具包663文件/273438730bytes与96a75694…清单逐项相符，代码02干净、原TASK3878字节/bc7965b4…保持。证据`.adg/evidence/OCX-0110-20261015-WIN-CURRENT-ACCOUNT/`不包含产品启动、配置删除、安装器或性能样本。普通/管理员仍须观察实际产品令牌并验证同SID同根，019配对原预算、真实周期及所有未决门禁不变；实际前台操作继续等用户新的开始信号，原失败ETW/Macfailed不提升，未关闭或发布。

### 2026-10-15 当前账户启动根、真实恢复及Windows同机RSS失败

用户新“现在开始前台验证持续完成后续任务”后，以02匹配候选/MSI载荷7383c275…在自有程序位置顺序执行非sandbox根准入。旧管理器Roaming/Local旁存，不删Codex会话或改日常EXE。实际产品普通false/type3与管理员true/type2的TokenUser摘要相同；换位置仍显示原anchor/HOME，不建新data，管理员真实设置页及正常exit0已补。普通不可写夹具的取消exit0且不写指针/不回退AppData；选既有合法根登记后真实重启成功。损坏/过新指针均明确拒绝exit1、原值保持、不回退，原ACL与测试指针已恢复。

恢复交接的进程时间线曾native2：旧选择器尚未初始化业务、新EXE已启动；严格单EXE上限未通过，原件保留。外置HOME/设置切根被真实未核验面板状态阻止，不称成功。首轮偏好夹具medium误值及原生输入方法错误保留，修正mid后才计外观。固定浅色备份→深色→真实确认恢复回浅色，载荷逐字节一致，保护备份精确关联/committed、重启保留；新备份预览无删除，不替代轮换删除。152项公开清单和8份本地私有选择器原件逐字节验证，见[当前账户启动根与恢复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows当前账户启动根与恢复/README.md)。

用户“我给你前台”后，保留前台中断轮和599.47s计数不足/基线激活失败轮；修正采集器、冻结首个完整读数后≥600真实秒。正式019→02同机/同真实普通账户/浅色高档mesh100%/准备态，各598样本、600.538454/600.596923s、零前台中断/实例超限/子进程读取失败、各7进程族、正常exit0。单逻辑核CPU40.494909→29.517509%，差−10.977400pp；平均RSS544.724740→583.052832MiB，**+38.328092MiB超原10MiB预算，保持failed**。末端+40.2578125MiB，峰值+5.8515625MiB不替代平均失败；共享页未去重，短命子进程未全覆盖。基线复用此前存活进程/稳定60s，候选新启稳定60s，稳态子集不冒充冷启动或因果根因。

同候选真实偏好保存只读失败→恢复属性→同配置重试成功，原失败同对象/动作/阶段/local/candidate被解除，其他未解除项保留，正常退出；只计本地保存重试子集。四档共104份文件清单/独立verify-archives通过，见[同机性能与事件重试](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261015-Windows同机性能与事件重试/README.md)。代码02未改，未合并/发布；019冷/热各20/p95、主线程、完整请求/周期、更新来源、原失败ETW与Macfailed及全部门禁仍开放。真实状态错误30min另轮计时中，未预判终态。

### 2026-10-16 真实状态错误30分钟下限与光场辅助缓冲优化

02精确EXE的独立sandbox真实损坏schedule于10月15日23:38:39.954由生产planner返回错误；约10秒后仅修复自有状态。10月16日00:08:40.070首次真实重试成功，单调差1800007.100000024ms≥1800000ms，合成online唤醒未提前触发，选定查询IPC调用0、调试暂停0。桥接不可写观察器拒绝和Network看不到Windows postMessage的90秒失败方法均保留；最终仅用不暂停的条件日志点观察真实入参与回调，不替换函数/结果或计时器。正常exit0，00:08:50native0；未来deadline是夹具，不充当真实周期基线。

57项清单/verify-floor通过。scheduler.ts、useUpdateScheduler.ts、update_schedule.rs、schedule.rs在02与8be的Git blob相同，可按此范围复用，不重标EXE。详见[真实状态错误退避](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows真实状态错误30分钟退避/README.md)。原24h/6h、休眠/时钟回拨、完整请求/子进程、50ms及其它调度矩阵不提升。

针对Windows平均RSS+38.328092MiB失败，真实旧画布观察到depth=true/24位但深度测试关闭；同一WebView的24组原生产着色器对比证明取消深度缓冲后RGBA逐字节相等。代码8be87e2d已推送，仅为二维光场请求depth:false/stencil:false，不改画面、分辨率、光层尺寸或画质；本地UI99文件/533项、类型与构建通过，精确push/PR CI沿原run进行。PR5说明已读回仍draft/未合并；新候选只在完整同SHA CI通过后单次派发，尚无新EXE内存通过结论。详见[光场辅助缓冲优化](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows光场辅助缓冲优化/README.md)。冷/热采集方法提前冻结，尚未执行，不拿准备抵样本。

00:13:55两轮8be CI完整success：各Mac923/6ignored、Windows648/7ignored、35组，四份原job日志/摘要及实际checkout核对通过；PR实际7bf52c8e与head8be的完整tree均16749535…。fmt/Clippy/前端与最终EXE沙箱冒烟通过，历史失败不清。00:14:05仅派发一次8be候选38767255195，00:18:08沿同run构建中，不重复请求；50项优化与CI清单/独立verify通过。00:12:26再读可信来源，beta Release仍404、stable为正式v0.1.9；只作来源观察，不是产品周期检查，不为补条件发布beta或更改来源。

### 2026-10-16 已有自定义根的同SID双令牌启动

02精确EXE以当前账户普通false/type3、管理员true/type2分别原生启动，TokenUser摘要相同，两次设置页均读到自有anchor预置的custom-root与外置HOME；真实手动备份写入custom-root，原anchor日志未改，各正常退出0。只对自有夹具预置已有绑定，之后恢复anchor元数据原字节，HKCU指针未变；五个相关存储blob在02/8be相同，实际EXE身份仍记02。49项清单与独立verify通过，首次页面未就绪的观察失败保留。

详见[自定义根启动绑定](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows自定义根启动绑定/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-CUSTOM-ROOT/`。本子集不证明UI路径保存、真实OpenCodex子进程HOME继承、迁移或宿主安装/升级/卸载保留；此前被未核验运行态阻止的UI保存仍待验。八门禁与TASK revision2/in_progress/pending、Mac性能failed、全部原预算及发布状态不提升。

### 2026-10-16 8be精确候选就绪与空闲RSS补测失败

单次候选38767255195成功，精确8be checkout、包回归及runner MSI/NSIS安装/首屏/卸载子集通过；两artifact API digest/大小/CRC/回执与MSI实装EXE摘要核对。宿主只读提取0.1.10/8808448bytes/SHA26d5458d…，未执行宿主安装器。新EXE独立原生sandbox/CDP观察确认画布1496×1342、depth/stencil均0、GLerror0、正常退出0；20/16项清单各verify通过，不计内存或完整原生通过。

完整保留首轮AX过早检查的0样本方法失败，以及019完成/候选382.495044秒前台中断轮；后者01:02:42采集器确认窗口归属后正常关闭0，不是产品崩溃。用户新“可以继续吗？”后只补一次候选，不拼接旧样本、不重跑完整019。019598样本/600.562950秒与新候选596样本/600.409511秒均零前台/实例/子进程读取异常、普通同SID、新进程、准备态、light/high/mesh100%、客户区2360×1520。CPU35.778930→12.572148%，差−23.206782pp在预算内；平均RSS536.752149→570.001429MiB，**+33.249280MiB仍超原10MiB，继续failed**。末端+19.761719、峰值+20.800781另列；不跨轮基线差异推导优化收益。

本轮就绪观察器由UIA换为sky，基线明确复用且有中断间隔；不称无间断相邻配对、同观察器或因果归因。实际计数期无UI输入/观察、构建或ETW；进程族CPU保留退出成员，RSS共享页未去重，短命子进程未保证全覆盖。01:25:17绑定窗口Alt+F4正常退出0；01:26:00native0、日常/TASK摘要及代码8be干净、五共享文件一致。19/40/30项清单及独立原样本/跨目录复算验证通过，见[新候选空闲性能](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows新候选空闲性能/README.md)与同目录[缓冲优化记录](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows光场辅助缓冲优化/README.md)。

Windows平均RSS、Mac性能failed、bd2原失败窗口、严格单EXE恢复交接、冷/热p95与主线程/查询/周期、安装保留及其余八门禁继续开放；TASK revision2/in_progress/pending不改，DPI待验，未关闭/合并/签名/晋升/发布。前台已释放，后续先定位差异，不以重复采样挑通过轮次。

### 2026-10-16 光层定位完成后再分配缓冲

后台Edge组件诊断复现8be在Teleport更新前创建画布：正确Windows壳层/1180×760 CSS视口/DPR2下，宽高写入依次1426×150、1426×960、1496×960、1496×1342，前两次发生于旧motion-mark容器。d94c215d已提交推送，仅延后mesh至DOM移动完成，Logo保持同步，提前卸载不再分配。最终工作区高/中、浅/深8组均只在最终路由写1496×150、1496×1342，最终尺寸不改，移除后画布0；浅/深两份2360×1520静态RGBA逐字节一致。此为原组件/着色器/CSS的简化浏览器装置，不是原生EXE/RSS或完整动画门禁。

新增两项资源时序回归在8be失败；整scene延期引起两项既有即时Logo测试失败后，仅修正实现、保留原断言。最终本地UI99文件/535项、类型/生产构建通过。原始路径转义失败、简化壳尺寸差异及修正方法均保留；实际工作区含混合换行，逐文件以换行位置重建核对Git对象和诊断摘要，不重写原件。07:40:44native0、日常/TASK摘要、代码d94干净及五共享文件一致。

详见[光层定位后分配](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows光层定位后分配/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-MESH-ALLOCATION/`。PR5范围更新读回，仍draft/open/未合并。当前push38824163566/PR38824151573各首次CI于07:43:14仍进行，前端/发布工具/令牌检查已成功；后续终态另记`.adg/evidence/OCX-0110-20261016-WIN-D94-CI/`，未提前请求新候选。仅证明消除中间分配，未取得原生GPU分配归因或新EXE内存通过；8be平均RSS+33.249280MiB、Macfailed、bd2失败窗口、其余全部原门禁与TASK revision2/in_progress/pending保持，DPI待验，不关闭或发布。

### 2026-10-16 查询主线程热点的有界录制准备

既有固定bd2成功回放仍不能证明目标失败窗口，现录制器还省略整个Rust根进程的调度/I/O热点。独立诊断worktree/分支`feature/0.1.10-windows-etw-query-threads@7704a179`仅改三份test/smoke录制方法：通过ThreadStart/SetName精确识别七个查询测试线程，按原始QPC、根与线程代际及重命名时间限定热点；未知名称不保存，TID复用不继承，名称历史上限512后关闭根热点并显式缺口，未见齐七名也不声称覆盖完整。原64MiB/32768队列/IRP、2/4/5/8秒及120秒外层不改，整个apps tree与d94相同；已固定失败库497814ca…和官方Node ba4e6d11…逐字节复制准入，未执行二者或ETW。

本机Add-Type与新增39项范围、原有范围/字节预算/停止/身份守卫、13项CI准入测试及PowerShell解析通过；不是实际线程命名事件或失败窗口取证。10项清单/独立verifier通过，08:07:20两code worktree干净、native0、日常/TASK摘要及五共享文件一致。诊断push38826901050沿首次运行，尚未完整准入。详见[查询线程ETW准备](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows查询线程ETW准备/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-QUERY-THREAD-PREP/`。后续一次原库录制仍须明确实际命名/缺口与目标失败是否出现；不用成功回放清除历史失败。

d94产品PR整轮CI38824151573已success：五个必需job成功，main专用build在特性PR正常skip；原始frontend99/535、Mac35组923/6ignored、Windows35组648/7ignored及checkout f2c5f1bd完整tree f8e4654c…与d94相等。push38824163566的Windows/前端原件已核，Mac工具链仍未完成，不拼部分jobs宣称该轮成功。按一轮完整同源CI准入候选，单次派发38827408691，08:07:26package进行中、query-etw/retention跳过，尚无新EXE。准入脚本首版把main-only skip误拒绝的失败日志保留，修正为核对五项必需与预期skip，非放宽产品门禁；终态/制品分别续记D94-CI/D94-CANDIDATE专档。Windows平均RSS+33.249280MiB、Macfailed、bd2实际失败窗口及其余八门禁仍开放，不关闭/合并/签名/晋升/发布。

08:18:10 d94伴随push38824163566终态failure，唯一失败为Mac backend；GitHub原始注释托管runner失联，最后工具链安装尚进行，cargo fmt/Clippy/test未执行。原日志HTTP404缺失，保存API终态/check-run/jobs/两注释和8份清单，独立verifier通过；不据注释推断失联根因，不改标通过或重跑。完整PR成功证据另存，候选38827408691仍沿单次构建，08:20:17进行中；诊断7704 CI38826901050仍在最终Windows release构建，未准入或开始录制。新增失败不解除原bd2或Windows/Mac性能failed及任何门禁，TASK/IMP保持未完成。

08:22:38 d94单次候选38827408691已成功，原package日志checkout d94、35组648passed/7ignored、MSI/NSIS runner新装首屏卸载子集通过。两artifact API摘要/大小/CRC/载荷与回执核对，宿主只读MSI/CAB提取0.1.10/8806400bytes/EXE SHA b2284ba66a20c7ef11b61fea328f7aa960cd6530011d6e7a09fc78e782c5b224。29项清单与独立verify通过，绑定完整PR准入，未把伴随push失败改标；native0、日常/TASK摘要和d94代码干净保持，未安装/启动宿主产品。原Windows RSS+33.249280MiB、Macfailed及所有未完门禁继续开放，d94新EXE待新前台窗口实测；原存储/调度证据按实际来源和等价范围复用，未关闭或发布。

### 2026-10-16 命名查询线程的单次真实ETW结果

独立录制器7704的push38826901050完整success，五必需job成功；三原日志checkout7704、frontend99/535、Mac35组923/6ignored、Windows35组648/7ignored核验。首次日志DNS读取失败后只重取相同job，没有重跑。沿持续管理员后台授权单次派发query-etw/isolated-replay38829382543，原bd2失败库497814ca…/官方Node ba4e6d11…、原并发/捕获/2/4/5/8秒/120秒/64MiB/32768均保持，原库无参数且不预热。隔离runner提升true/type1，不算本机产品双令牌；未使用本机UAC/前台/产品，未声称原环境完全相同。

08:25:49–08:26:04一次原库exit0、530passed/4ignored、七查询ok，目标失败未复现。19058原事件/7412205bytes，七个真实ThreadSetName，根命名查询线程1128进出调度/113ready/320create/138read-write/458完成；31进程及953线程开始/结束匹配，丢失/解码/已知后代与命名缺口0，自有会话正常停止排空、未强清理。其它根热点仍省略154100调度/287011文件发起，命名前与其它根工作线程不补，v5等待语义未核验，不声称完整根因窗口。

后处理裸TID对应多次stop的失败保留，改按start/name/首次stop代际审计，不改原事件或再执行；57项清单/verifier独立重算通过。08:30:48native0、双代码干净、五共享文件一致、日常/TASK摘要保持。详见[查询线程ETW结果](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows查询线程ETW准备/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-QUERY-THREAD-RUN/`。实际bd2查询失败窗口和根因仍未取得，历史失败不解除，不继续重放碰运气；d94新EXE内存/原预算及其余八门禁、Macfailed保持，TASK/IMP未完成，无合并/签名/晋升/发布。

### 2026-10-16 d94前台配对未准入，补充设置与树表观察

用户新开始前台后，同期019→d94各一次，新进程、普通同SID、light/high/mesh/100%、同sky就绪观察与2360×1520客户区；各稳定≥60秒。019598样本/600.9124425秒但子进程读取失败1，缺失代际/计数未知，不准入；d94于08:54:37失去前台，仅148样本/150.4015179秒，不准入，原因未建立。01908:50:13退出0后才启动d94；没有重跑或用旧样本替换，不把未完成比较改判通过。进度文件曾滞后，中止已现场更正；完整对照分析明确拒绝生成comparison。

同一存活候选的设置真实截图为高档/WebGL，备份树默认月份折叠→实际展开用途目录→刷新完成后保留；记录浅色三列目视对齐、固定/保护项仍可读，不替代全像素/键盘/备份恢复轮换。没有修改偏好、代理、根或备份。候选09:00:45正常exit0，随后native/窗口0；09:02:25日常/TASK摘要、d94代码干净、五共享文件一致及测试偏好原字节保持。63项清单/verify-attempt独立重算通过，仅证明失败轮与UI子集完整，详见[前台配对与只读树表](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows-d94前台配对与只读树表/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-IDLE-D94-019/`。

先补采集器子进程异常PID/代际/错误及尽早中止，不自动重复长测或放宽原预算。新d94内存门禁未确定，原8be RSS失败、Macfailed、bd2实际失败窗口缺失与全部其它原门禁保持，DPI待验，TASK revision2/in_progress/pending不变，无关闭/合并/签名/晋升/发布。

### 2026-10-16 空闲采集器完整性修复

诊断分支f2271f36已推送，仅三份test/smoke空闲采集方法和CI检查；apps tree与产品d94相同、7704的ETW方法逐字节不变，不重新打包候选。只读附着入口保存读取/代际失败的PID、创建时间、阶段、异常类型/HRESULT并立即失效；退出成员CPU保留、父族按存活区间限定，前台/可见性/最小化/单实例拒绝不改，异常terminal与progress同步。600秒首末完整计数、零读取失败、CPU/RSS原口径和预算保持，不激活/强杀/自动重试。

本地55项检查及PowerShell解析通过，真实受控PowerShell子进程与CIM全快照覆盖句柄/退出CPU，合成缺失PID/代际/窗口与599.9/600仅测试拒绝/终态；没有产品或长测。测试首版恰好两成员的错误假设改为代际唯一并保留所有控制台后代，原方法/日志保留。Python缺YAML包不安装依赖，GitHub实际接受并执行新工作流；push38834762492的Windows采集器job成功，原日志checkout f227/55项、artifact12018632501摘要/CRC/原件核对，09:28:13整轮仍进行，仅Windows后端未完，不冒充整轮通过。

47项清单/独立verify通过，09:25:51native0、双code干净、共享一致及日常/TASK摘要保持，见[空闲采集完整性修复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows空闲采集完整性修复/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-IDLE-COLLECTOR-FIX/`。原失败样本不回写为有效；新入口还未完成产品600秒实测，d94内存与原其它门禁/Windows-Macfailed/bd2失败窗口缺失保持，DPI待验，TASK/IMP不关闭，无合并/签名/晋升/发布。

09:41–09:42同一首次CI38834762492完整success，五必需作业成功、main-only build正常skip；三原始日志均checkout f227，前端99/535、Mac35组923/6ignored、Windows35组648/7ignored。复用已冻结55项采集器及制品原件，不重跑或重打包。新档`.adg/evidence/OCX-0110-20261016-WIN-IDLE-D94-F227/`37项清单/独立verify通过，固定f227采集器、d94/b228产品与原019，准备下次人工窗口的夹具/启动/sky就绪/附着/正常退出绑定；七份PowerShell语法及宿主SID只读检查通过，不算产品令牌或端到端测量。

窗口回执模板仍为null，本轮未克隆夹具、启动产品、激活前台、录制ETW或长测，09:45:18native0、日常/TASK/夹具摘要保持。扩展路径误拒绝和main-only skip误判的初版方法/失败均保留修正；README原字节CRLF差异已核Git对象及内容一致，不改共享文件。详情续记[空闲采集完整性修复](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows空闲采集完整性修复/README.md)。旧失败不解除，新d94/019性能仍待真实完整样本，其余原预算/八门禁、DPI待验、TASK revision2/in_progress/pending及禁止发布状态保持。

### 2026-10-16 恢复交接等待旧进程退出

确认旧02两EXE恢复现场与d94相同启动源码直接调用Tauri重启。823a8a06已提交推送，仅改Windows恢复交接、嵌入后台等待方法和检查：保留旧代际真实句柄，准备/提交后等待退出再自动重开；失败保留已登记根并提示重新打开，不改普通成功启动。原生严格单EXE仍待验，未将旧失败改标。五对真实受控进程本机/CI各40项通过，无产品/前台/强制夹具清理；本机Rust格式及生产/测试元数据、PowerShell语法通过，完整链接因缺SDK交由CI。

push38838964154的Windows Clippy依赖下载遇phf_generator0.13.1连接中断而失败，后续测试/构建跳过，原日志/API制品保持，不重跑。PR38838969102原日志已核535前端/923Mac及实际7633c3ba全tree与823相同；10:21:46 Windows Clippy/回归步骤成功、release构建中，尚不宣称整轮通过或派发新候选。PR5说明更新读回仍草稿，详见[恢复交接等待退出](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-RECOVERY-HANDOFF/`。

此轮无宿主产品、ETW重放或长测，日常/TASK/夹具摘要保持，原8be与Mac性能失败、bd2实际失败窗口及其余全部预算/八门禁不变；新制品就绪后重新绑定前台方法，不以d94旧EXE顶替。DPI待验，TASK revision2/in_progress/pending、IMP未完成及禁止合并/关闭/签名/晋升/发布保持。

10:30:13同一PR CI38838969102完整success，五必需job成功、main-only build按配置skip；Windows原日志核650passed/7ignored/35组，Clippy、release及最终EXE启动冒烟成功。实际7633c3ba全tree与823相等，按整轮准入，不将伴随push依赖传输failure清除。PR完整验证说明再次读回仍草稿；原生恢复、配对性能及其余门禁仍待新候选实测。

79项修复/完整CI清单独立核验通过，10:33:54native0及日常/TASK/夹具摘要保持。10:34:23仅派发一次823候选38841117656，七份准入文件核验通过；首次只读传输失败后沿同一handle恢复，10:38:03package进行中、query-etw/retention正常skip。请求与进度另记`.adg/evidence/OCX-0110-20261016-WIN-823-CANDIDATE/`，尚无制品/EXE准入，不重派、不提前执行宿主原生或提升门禁。

### 2026-10-16 精确823候选与前台准备就绪

10:49:27同一候选38841117656成功，原package日志checkout823、35组650passed/7ignored，runner MSI/NSIS新装首屏卸载子集通过；两API制品摘要/大小/CRC、包回执均核验。宿主只读提取0.1.10/8830976bytes/EXE SHA2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53，与MSI实装载荷一致。39项清单/独立verify通过；MSI/NSS三字节固定包标记的精确摘要差异已复算，附加错误假设保留，未改原EXE或降低原校验，未执行宿主安装或产品。

另有29项前台准备清单/独立核验通过：固定823/261、019/626及f227采集方法，保留全部原预算；11份PowerShell语法、4个无人工信号拒绝、两份受控进程的内核时刻及4组100ns精度往返、缺输入不生成比较均通过，实际原生恢复和600秒配对尚未执行。只复制EXE到自有夹具，11:29:38native0及日常/TASK/配置摘要保持，无Profile移动、ACL/HKCU变更或新账户；待新的约30分钟前台信号。详见[恢复交接等待退出](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-823-FRONT-PREP/`。

原8be/Windows与Mac资源失败、bd2实际失败窗口、完整安装/更新/备份/事件/树表/周期及冷/热/主线程等原门禁继续开放，DPI待验；不把候选验真或准备检查当成版本通过。TASK revision2/in_progress/pending、IMP未完成及禁止合并/关闭/签名/晋升/发布保持。

11:38沿823固化真实来源做四次只读复核：stable Release API/生产清单HTTP200且为v0.1.9/0.1.9，beta tag API/生产清单均HTTP404；未观察到高于当前0.1.10的真实管理器版本。13项来源条件清单独立核验通过，见`.adg/evidence/OCX-0110-20261016-WIN-UPDATE-SOURCE-PRECONDITIONS/`及[恢复交接等待退出](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md)。仅为来源可用性，非产品周期/请求计数/更新进度；无产品、npm脚本、FZ50、调度写入或计时重置。11:41:37native0及日常/TASK/夹具摘要保持，真实条件缺口和全部原门禁不提升，不发布或伪造来源补测。

11:55仅审计既有7704成功回放与固定TraceEvent3.1.21 DLL：三个等待getter以GetByteAt读偏移12/13/14，自身没有v5分支/掩码。564条根查询切出与3095条后代切出均v5；根等待原因545条、模式42条超出当前返回类型枚举，不能据此推断内核语义或原因。旧原件无完整CSwitch payload/ETL，离线不能补足v5解释；保留全部semantics=false，实际bd2失败窗口及根因仍未取得。8项清单独立复算通过，见[查询线程ETW准备](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows查询线程ETW准备/README.md)及`.adg/evidence/OCX-0110-20261016-WIN-CSWITCH-V5-AUDIT/`。未重录/重放/预热或改解码器、产品、预算和失败判定；native0及日常/TASK摘要保持，823前台仍待新人工窗口，八门禁/原性能/Mac失败与发布限制不变。

12:12完成431/823 MSI只读表审阅及原表独立复算：核对两包实际摘要，所读删除/目录/动作/执行序列/升级/注册表等10表一致，仅ProductCode、两个EXE尺寸及ocxd组件GUID不同；基础/Windows打包配置、验证脚本及package工作流段Git字节相同。7项冻结原件与补充crosscheck见`.adg/evidence/OCX-0110-20261016-WIN-823-INSTALLER-REUSE-REVIEW/`、`OCX-0110-20261016-WIN-823-INSTALLER-REUSE-CHECK/`及[恢复交接等待退出](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md)。431原MSI/NSIS保留专项各5步骤、6夹具×5读回事实继续按旧来源引用；未比自定义动作二进制/NSIS结构，不改标为823安装或根绑定通过。无宿主安装、卸载、产品或前台，TASK/原失败/预算/八门禁保持，下一原生窗口仍需人工开始信号。

### 2026-10-16 823正式同机空闲配对，平均RSS仍超预算

用户12:57:34新“开始前台验证”后，在约定30分钟内依次新启019和823精确载荷；真实普通产品令牌false/type3、同SID，浅色/high/mesh/100%及物理2360×1520准备态，各真实稳定≥60秒。f227原方法记录557/559样本、600.892218/600.763383秒，零读取/前台/可见性/单实例失败，两次正常exit0且内核时刻证明旧019退出早于823创建。13:25:17native0并释放前台，没有第二轮性能重跑或旧样本替代。

单逻辑核CPU45.099602→39.787919%，差−5.311683pp满足≤+0.5pp；平均进程族RSS548.664841→577.886202MiB，**+29.221361MiB超原+10MiB，保持failed**。独立逐成员复算显示主进程约+1.415794MiB、WebView后代合计约+27.805567MiB，尚无角色/分配栈或因果归因。采集器CPU138.906250/139.765625秒另记；共享页未去重、短命子进程未全覆盖，不冒充完整性能门禁。

计数前46.482秒就绪和DPI不感知逻辑尺寸读取两次保护拒绝均保留，未生成计数目录；随后使用真实满60秒AX及仅观察子进程的HIGHDPIAWARE环境读取物理尺寸，原EXE/采集源码/系统缩放和窗口未改，非DPI验收。27份运行原件逐字节归档及独立复算见`.adg/evidence/OCX-0110-20261016-WIN-823-IDLE-PAIR/`与[恢复交接等待退出](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261016-Windows恢复交接等待退出/README.md)。配对关闭后余时不足180秒，恢复控制器未执行、ACL/HKCU未改，严格原生恢复继续待验。日常/TASK/候选偏好摘要及两代码分支干净保持，真实旁存仍在；其余全部原预算/周期/八检查、历史Windows/Mac失败及bd2失败窗口缺失保持，不关闭或发布。

### 2026-10-16 后台渲染假设检验，两方案均撤回

在隔离Edge154/Intel ANGLE D3D11中，裁切高度、同源控制、整数盒裁切与预乘透明各做4种几何×2主题×3时刻，未启动桌面产品或重做正式配对。两裁切原型保留区RGBA相同，但合成分别最大5/255，整数盒在分数DPR仍不等价；A/A和重复截图全部零差。预乘原型按RGBA8量化仍有1–2级字节差、最大2/255合成差，未证实实际内存节省。两方向均不采用，生产823已恢复并核到干净。

44份PNG引用按摘要去重为13个原件，独立复算及最小原型差异见`.adg/evidence/OCX-0110-20261016-WIN-823-RENDER-EXPERIMENTS/`。整数盒方法未另存原字节的范围已显式记录；这批证据用于否定未证实的改动，不证明根因或原生性能通过。+29.221361MiB平均RSS失败、bd2失败窗口、其余原预算/周期/八门禁及DPI待验保持，TASK/IMP不关闭，不发布。

### 2026-10-16 调度字节与GPU资源取证准备，网络阻塞前未录制

诊断54927d2c已推送，新增已准入CSwitch偏移8–15八字节记录，排除TID前缀/未知尾部，保留原值且v5语义仍false。本机159项纯检查通过（新增12项）；同SHA首次CI38858859494完整success，五必需作业通过、main-only build正常skip。连接器解码前端日志核到checkout549和99/535；Mac/Windows仅状态API核验，完整原日志读回未完成，不称原日志准入或新录制通过。原bd2库497814ca…、官方Node24.21.0/ba4e6d11…及全部原并发、截止、外层/采集预算保持，未运行或预热输入。

GPU只读伴随方法本机47144dbe已提交、21项纯检查通过，含真实Windows argv解析、角色允许表、PID过滤、缺失值与UInt64精度；真实产品快照/GL缓冲归因未做，代码未推送/CI，先保留549待录制源。网络原件读回遇Winsock10055及非任务进程大量UDP端点，本机配置/外部进程未改；细节留私有运行目录。证据和继续入口见`.adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-BYTES/`，网络恢复后须在新RUN档案补原日志与准入，最多派发一次，不修改冻结目录或重发已受理调用。未派发新ETW，无需重请既有管理员录制许可。

14:43只读保护核到native0、产品823干净及日常/TASK摘要保持。bd2实际失败窗口、+29.221361MiB RSS失败及原Mac/Windows资源失败、原全部预算/周期/八检查仍开放，DPI待验；TASK/IMP未关闭，不合并、签名、晋升或发布。

### 2026-10-16 GPU读取器完成受控执行与首次CI

独立工具分支768e2318已推送读回，保持ETW待录制分支549不变。本机37项检查含真实PowerShell根/子代保留句柄与正常退出，控制代际均已消失、native0；GPU实例为空不视为零显存，不当作桌面产品快照。首次push38864269467于15:17:42全CI成功，五必需作业通过、main-only build正常skip，原run/jobs API核对；完整job原日志仍未取得。初版UTC字符串尾零比较失败与修订校验器保留，原件未改。

证据`.adg/evidence/OCX-0110-20261016-WIN-GPU-BOUND-PROBE/`独立校验通过，只证明受控方法及CI状态。后续真实人工窗口的823恢复/GPU执行另档，不回改准备时事实。Windows RSS+29.221361MiB、原Mac/Windows失败、bd2实际失败窗口缺失及全部原预算/周期/八门禁保持；TASK revision2/in_progress/pending、DPI待验与禁止发布状态不变。

### 2026-10-16 823恢复前置遗漏，取得一次真实GPU快照

用户15:18:26新前台窗口内只启一次823精确EXE，实际普通false/type3、原同SID，未用sandbox/CDP或重复配对。因已有Codex应用容器LocalCache/Roaming管理器测试根，生产按default_anchor规则进入原根；准备只清除HKCU/拒绝install-d创建，遗漏实际平台根前置。240秒485次原计数最大native1、无替换代次，恢复子集未通过，不建立产品恢复缺陷，不自动重试或强杀。

同一实际产品代际由768e2318工具保留句柄取得7成员/1GPU/2计数器实例，共享GPU使用191668224字节、GPU进程RSS246169600字节；只是一时快照，不相加、不倒推历史性能PID或归因GL缓冲。工具首次全CI在窗口前成功，原job日志未完整核验。15:25:35正常Alt+F4/exit0/native0；冻结restore先恢复ACL再因自动登记的平台根指针拒绝，随后核当前原字节/实际UI/native0后精确恢复原指针，ACL及指针读回相同，原拒绝保留。

17份运行原件与独立验证见`.adg/evidence/OCX-0110-20261016-WIN-823-RECOVERY-GPU/`，私有SDDL/原SID不发布。15:32:09日常/TASK摘要和真实旁存存在性保持，前台已释放，无重启或数据移动/删除。下一恢复准备须核实际平台根并隔离自有夹具后另取人工窗口；本次失败不清除。+29.221361MiB RSS、Mac/Windows历史失败、bd2失败窗口与全部原预算/周期/八检查保持；DPI待验，TASK revision2/in_progress/pending、IMP未完成及禁止发布不变。

### 2026-10-16 审计方向复核与本机验收范围裁决

已读取“审计同步卡点并明确收尾”对话（`01a143b3-d13d-7523-b7bb-ce3390bfbee9`）。采纳已有效证据复用、先修恢复夹具及分别处理产品问题与外部依赖的方向；该对话提出的直接接受RSS增长、将ETW或安装/更新门禁延期不视为用户授权。RSS仍按原+10MiB预算，PSS/私有工作集可补充定位但不替换原指标；实际bd2失败窗口与根因未取得的结论保持。

审计中恢复观察的“140秒85次”与原件不符，应为240秒485次；原823的`default_anchor`以平台根是否存在决定优先级，清空内容仍不足以触发安装目录data恢复。后续先核实际平台根（含已观察的应用容器路径）及HKCU，按本机配置清理授权准备目标条件，再补一次严格单实例交接；不新增两小时人工恢复流程，不重复未修改823的有效配对。

本次用户明确取消专项DPI，并允许本机OpenCodex软件卸载及配置清理，已更新第9节当前验收口径；安装保留/明确清理仍以可删除的专用样本分别验证。TASK的八个required_checks及revision2/in_progress/pending保持，未改标任何失败为通过，未合并、关闭或发布。

### 2026-10-16 按原生目录ID修正恢复夹具，ETW原日志仍受阻

按用户本机OpenCodex配置清理授权，实际句柄核到当前普通Codex宿主的四条逻辑AppData/LocalCache路径只对应两份原生目录；首次逐路径移动在第三条不存在时停止，已完成两次移动先按原生ID还原，再按真实ID分组各保存一次。四条逻辑路径现均不存在，两原目录对象及创建时刻保持，内容留忽略运行目录，未删除或发布配置。HKCU指针/ACL未改；不把该命名空间推广为管理员或另一启动上下文。

12份原件、独立校验、空平台根拒绝与两个缺人工窗口拒绝见`.adg/evidence/OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP/`；原恢复控制器不改，新v2入口补实际根缺失与原生ID守卫。16:23读回native0、日常/TASK摘要保持，823/768两代码工作区无tracked改动，未启动产品或安装器。新完整profile还原脚本仅语法检查，先前部分还原已实际核ID；严格产品恢复仍待新的人工窗口，不能把本准备算通过。

同549首次CI38858859494的原日志继续档`.adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/`保留一次curl6/DNS失败与一次独立只读HTTPS路径curl7连接失败；网络状态未改变时停止相同读回。未准入、派发或录制，原bd2库与官方Node未执行，未动外部进程或系统网络。后续沿同CI及同RUN，最多一次修订录制，原预算/断言不改。

源码检查否定“所有路由同时挂载”的假设：App.vue只挂当前动态组件；光层几何有变化，但没有原生分配归因，不做臆测优化或重跑未改823配对。+29.221361MiB RSS、bd2实际失败窗口、Mac失败和其余原门禁保持，DPI专项按用户裁决取消；TASK revision2/in_progress/pending、禁止合并/关闭/签名/晋升/发布不变。

### 2026-10-16 单次549原始调度字节录制完成，仍未复现bd2失败

通过既有IPv6 SOCKS命令级代理及分域TLS参数取回同549首次CI38858859494三份原日志，实际checkout/前端535/Mac923/Windows648及五必需job核验完整。原DNS与TLS失败保留，证书校验未关闭、系统网络未改。沿持续管理员后台授权仅一次派发38876971001/attempt1，固定原bd2库497814ca/官方Node24.21.0-ba4e6d11，原并发/截止/外层/容量预算均不变、无预热；原库530passed/0failed/4ignored、七queryok，仍为成功对照，历史失败不解除。

制品12035074475摘要/CRC/13原件与job日志核验；事件17296条/8153824字节、31对进程/939对线程、七真实命名，丢失/解码/已知后代和命名缺口0，自有会话停止排空无强清理。偏移8–15字节原件6131行对应源计数5319次回调；812组进出双行及一组简化QPC/字节碰撞，离线初版计数假设错误原件保留，复算不改原数据、不声称重建唯一内核事件。v5等待语义仍false，根其它调度156797和文件发起287105按范围省略。

原件gzip往返及完整核验见`.adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN/`。实际bd2失败窗口与根因仍未取得，本轮不再追加回放。17:33本机native0、日常/TASK摘要及保存配置原生ID/原指针保持；产品823与工具768无tracked改动。RSS+29.221361MiB、Mac失败、严格恢复与其余原门禁保持，DPI专项已取消；TASK revision2/in_progress/pending，不合并、关闭或发布。

### 2026-10-16 隔离组件资源尝试未准入，未采用原型

一次headless组件资源对照在首组823参考场景绘制后，因快照期间自有子进程退出被原保留句柄守卫拒绝；后两组未执行，未运行已否决的预乘原型、未生成资源比较、不重跑或放宽守卫。方法与六份原件见`.adg/evidence/OCX-0110-20261016-WIN-GPU-COMPONENT-ATTEMPT/`。独立浏览器finally关闭请求后根代际已不存在/native0；exitCode、内核退出时刻未知，后代清理未完整证明，不补写正常退出。

本次未启动产品或固定官方查询Node，生产823无改动，不能据此解释或解除+29.221361MiB RSS失败。bd2单次549已录制但目标失败未复现，原Mac/Windows失败与其余门禁保持；DPI专项已取消，不关闭或发布。

### 2026-10-16 本机MSI正式基线链与执行守卫就绪

只读MSI库存核对本机已装0.1.9产品码7069与正式基线019的7C06不同，不能混称正式基线；正式包b33dd0c8/823包e6449171均固定，同UpgradeCode且ALLUSERS=1。按已获本机卸载/配置清理授权，准备一次“卸旧本机包→正式019→823升级→卸载→重装→最终卸载”，命令禁自动启动，校验实际载荷并保留种子data/custom-root/兄弟样本。custom-root只作种子、不算应用绑定，明确清理/NSIS/首屏和其它安装场景不提升。

`.adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/`保存盘点、一次性计划与实际方法。真实普通令牌调用Execute在安装/写标记前被拒绝，旧EXE/注册及计划摘要不变、native0；未调用安装器。执行仍需同用户管理员实际令牌及新的最长30分钟人工窗口；每步核窗口、native0与120秒上限，状态漂移/失败不强杀或重试。初次目录边界拒绝保留，未改全局设置。

恢复夹具已另行准备，下一人工窗口先补严格交接并正常退出，再做本机MSI专项。RSS原失败、bd2未复现、Mac依赖及全部其余门禁保持；DPI专项取消，TASK revision2/in_progress/pending不变，不关闭或发布。

### 2026-10-16 复用有效样本收窄RSS取证目标

离线复算原019/823有效557/559样本，无新产品或配对。各版自身最大WebView成员全程代际稳定，均值205.181329→231.411428MiB，差26.230100MiB占总差89.7634%；其它成员差2.991261MiB。仅按大小分解，未匹配跨版本角色或把历史PID称为GPU/renderer，亦不当作可回收量或根因。

第一相对分钟已差26.577281MiB、第十相对分钟差25.535795MiB；两版全跨度斜率1.661734/2.057205MiB每分钟。十个整分钟各排除相对600秒后的1个末样本，但完整原均值和斜率仍含全部记录；时间加权差29.197968只作描述，不换验收指标。不能据600秒确认/排除泄漏。方法、摘要与完整分解见`.adg/evidence/OCX-0110-20261016-WIN-RSS-TIMELINE/`，下一步在真实新代际先取角色及分配证据，不重跑未改823的正式配对。

原+29.221361MiB相对+10MiB预算继续failed，bd2实际失败窗口仍缺，Mac依赖与其它原门禁不变；DPI专项已取消，TASK revision2/in_progress/pending，不关闭或发布。

### 2026-10-16 恢复选择器受阻并完成本机MSI升级保留子集

用户18:34:34新“允许前台”窗口内，v2真实平台根前置通过，18:37:31唯一普通false/type3同SID823实例实际出现不可写恢复提示。点击选择目录后，实际目录选择器捕获0x80070057/激活失败，停止工具输入并交给用户；240秒无替换代次、原始普查最大native1，严格交接未通过，不建立产品缺陷、不自动重试。正常退出观察器18:43:51超时，用户随后退出，18:45:50native0；退出码/内核退出时刻未知，不补写exit0。18:46精确恢复ACL/HKCU及两份原生目录，四逻辑别名原ID/创建时刻和原指针摘要再次核同。

同窗口18:46:51–56按冻结方法，以同用户实际管理员true/type2令牌完成卸旧本机019→正式019→823升级→默认卸载→重装→最终默认卸载，六步退出0及原MSI引擎返回0均核验。019实际载荷626/823两次261与固定包一致；含空格安装路径、六份安装data/兄弟custom-root/外部哨兵在五阶段哈希保持。最终三产品码卸载项/主EXE均不存在、native0，旧本机安装依用户后续卸载授权移除，原配置已恢复。18:48:39释放前台，无正式配对重复或新GPU取样。

证据与独立校验见`.adg/evidence/OCX-0110-20261016-WIN-RECOVERY-MSI-RUN/`。原MSI日志含SID，仅私有忽略目录保留原字节，公开明确标记脱敏派生件及原摘要，不冒称原件。执行前fail标记与最终pass分别保留；最终状态source初版null不作为源码绑定，沿冻结计划/候选身份核验。只通过宿主MSI升级/种子默认保留子集，不将兄弟custom-root种子称应用绑定，管理员执行器令牌不称产品令牌；完整安装/明确清理/NSIS/双令牌首屏等继续待验。

恢复交接、RSS+29.221361MiB、bd2实际失败窗口、Mac依赖及全部其余原门禁保持；DPI专项已取消。TASK revision2/in_progress/pending、IMP未完成，不合并、关闭或发布。

### 2026-10-16 光层先定画布尺寸，完整CI与像素对照通过

独立代码b4d487e4已推送codex/0.1.10-windows-mesh-initial-size，草稿PR6依赖原823恢复分支。光层先按原公式设置画布后再创建WebGL，避免默认300×150上下文后的两次初始化尺寸重设，着色器/属性/最终分辨率/透明度/后续resize和动画策略不改。原823新增回归实际三组均得到300×150而失败，修正后23相关项、99文件538完整项、类型检查及构建通过。首次从仓库根执行造成两个已有相对资源路径文件失败保留，按CI前端目录再执行通过，未修改这些测试。

首次PR CI38891188021整轮success：五必需job成功，main-only build按配置skip；原日志核538前端/923 Mac/650 Windows，后端各35组与6/7ignored。五份原日志实际checkout均e7272ddd，完整tree6d984c20与b4d相同；补日志校验最初误要求分支SHA，后续“没有checkout行”的解释也错误，已保留原错误并以all-job-checkout-correction逐份复算更正，原日志不改、CI不重跑。ANSI解析错误及一次只读TLS失败同样保留。

自有后台Edge154/Intel D3D11组件装置四布局×深浅×三时刻共24组，RGBA摘要和合成PNG逐字节相同；48PNG引用去重24原件，代表图实际查看。直接注入着色器时刻只作图像比较，不测主线程/帧或RSS，浏览器根精确Node子进程exit0/无signal，后代清理不完全证明。详见`.adg/evidence/OCX-0110-20261016-WIN-MESH-INITIAL-SIZE/`。无宿主产品、查询库/官方查询Node或正式性能配对，原RSS+29.221361MiB仍failed，未建立内存收益或因果归因。

同源完整CI准入后仅一次派发候选38893610415，归档时构建中，尚无制品/EXE准入；后续沿同运行取证。DPI专项已取消，参数化用例不新增DPI验收。严格恢复、实际bd2失败窗口、Mac历史失败及其余全部原门禁保持，TASK revision2/in_progress/pending不变，未合并、关闭或发布。

### 2026-10-16 精确b4候选验真与下一原生窗口准备完成

唯一候选38893610415/attempt1完整success，package原日志checkout b4d487e4、35组650passed/0failed/7ignored，runner MSI/NSIS实际EXE首屏及卸载子集通过，query-etw/retention按purpose skip。两ZIP的API摘要/大小、成员摘要/CRC及650原回归核验，沿完整同tree CI38891188021准入，不重派。缓存收尾时预取ZIP没有提前准入，完整success后才正式只读提取。宿主未调用安装器或产品，精确0.1.10/8830976bytes EXE SHA f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db与runner实装MSI相同；真实偏移8148661的三字节包标记在内存中转换后精确匹配NSS及未修补构建摘要，不写原EXE。见`.adg/evidence/OCX-0110-20261016-WIN-B4-CANDIDATE/`，原ZIP仍在忽略专用software目录。

新前台入口绑定b4/f3及正式019/626、原f227三份采集方法字节；实际诊断HEAD768和角色方法另按摘要记录，不混称来源。9份PowerShell语法、四入口缺人工信号真实拒绝、缺输入不生成comparison与独立复核通过，配置/目录ID/指针/TASK摘要不变、native0；未生成新人工执行回执、复制/移动配置或原生配对。仅因产品已改才允许下一次019/b4配对，原823有效配对不重做。每版真实稳定60秒、完整600秒、零读取/前台/可见性/单实例失败、CPU≤+0.5pp/RSS≤+10MiB和原口径均保持，至少1500秒余量才准备整对。

角色快照安排在正式计数前后，绑定当代实际PID/创建时刻/角色，不给历史PID补标签或相加GPU/RSS。退出观察器从启动后保留句柄，覆盖人工窗口及120秒只读清理观察，实际关闭动作另记，不延长前台或验收期限；还原入口按原生ID保留测量克隆并恢复原配置，不删除数据。新退出/还原入口只完成准备和语法核验，不冒称产品通过；严格恢复另补，不挤占30分钟配对窗口。见`.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP/`；草稿PR6说明已读回为候选验真完成，仍draft/open/unmerged。

新b4原生RSS收益尚未测量，原823的+29.221361MiB失败、Mac历史失败、bd2实际失败窗口缺失和全部其余原预算/周期/八门禁保持。DPI专项已取消；TASK revision2/in_progress/pending、IMP未完成，不自动合并、关闭、签名、晋升或发布。下一原生启动仍需新的直接人工信号及约30分钟窗口。

### 2026-10-16 b4完整计数与实际GPU归账，RSS失败且退出收尾待处理

用户20:43:38新“开始前台验证”的30分钟窗口内，依次启动正式019/626与b4/f3，真实普通false/type3同SID，浅色/high/mesh/100%及2360×1520客户区，实际界面稳定观察117.891/90.132秒。每版597样本、完整跨度600.2231132/600.0758786秒，均在21:13:38截止前，读取/前台/可见性/单实例失败0；基线正常exit0且内核退出早于b4创建。本轮没有重复或拼接旧样本；只读观察子进程沿既有物理坐标环境，产品与系统显示未改，DPI专项仍取消。

按原单逻辑核CPU口径25.303091→21.770684%，差−3.532407pp满足≤+0.5pp；平均进程族RSS568.388413→588.428569MiB，**+20.040155MiB仍超原+10MiB，failed**。本轮实际角色快照逐个PID+100ns创建时刻匹配全部597样本，GPU进程平均RSS209.915162→234.001538MiB，差+24.086376，其余角色合计约−4.046221。不是按大小猜GPU、不给旧PID补标签；测前共享GPU152444928/174489600字节只作一时计数，不与RSS相加，不证明具体GL分配或相对旧823轮的修正收益。

候选计数21:13:10.989结束，关闭前只读观察21:13:27.993；随后关闭动作未赶上前台时限，被守卫拒绝，未发出输入，确切失败尝试时刻未另存。21:15:38原退出观察到期仍native1，之后单独5分钟只读清理观察于21:21:37同样到期，旧结果未覆盖、前台未延长、未强杀。21:27:20读回候选PID76372仍存活，正常退出码未知、配置尚未还原；原Roaming目录在本轮忽略run的baseline-previous按原生ID/创建时刻完整保留，指针/TASK摘要保持。当前只准入完成计数的算术与角色归账，不生成正式comparison，不声称完整配对生命周期或完整性能门禁通过；候选可选postspan角色观察未执行。

证据与独立复核见`.adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES/`，草稿PR6已回写RSS failed及退出/还原未完成，仍draft/open/unmerged。需要用户正常关闭测试实例后，在native0下执行已准备的原目录还原；之后基于真实GPU角色继续定位具体分配，本轮计数不自动重做。原b4/823与Mac失败、bd2实际失败窗口缺失、全部其它原预算/周期/八门禁保持，未追加查询库/官方Node回放、未执行FZ50。TASK revision2/in_progress/pending与IMP未完成保持，不合并、关闭、签名、晋升或发布。

### 2026-10-16 同代际GPU工作集页分类补充，未归因原RSS差值

独立工具3a2ff08a已按实现分支推送读回，本机最终34项及首次push CI38910385568的Windows方法作业117004428361通过。实际checkout、受控制品API摘要/大小/CRC、三份提取原件及作业原日志核验；CI方法CRLF与Git LF为同源码换行转换。核验时整轮仍在运行，仅准入该受控方法，未称产品候选或整轮CI通过；原推送TLS EOF及读回远端不存在后同源重推记录保留，无重派CI。

21:51:20在正式计数之外，仅一次读取仍存活的b4 GPU PID17236/原创建时刻，同一root76372、实际GPU角色/祖先/普通令牌及native1守卫通过；无产品启动、UI输入或目标内存内容读取。61,230页/250,798,080字节复核有效，private/image/mapped区域分别173.675781/60.468750/5.035156MiB。进程工作集与枚举差65,536字节，非原子及Shareable“可共享”语义显式保留，不称唯一物理量、回收量或GL分配栈。没有019对应页分类、当前场景未重观测，不能回填原600秒或解释+20.040155MiB差值。

33份原件、源码与先前冻结证据绑定及独立复算见`.adg/evidence/OCX-0110-20261016-WIN-WORKING-SET-POINT/`。21:54:09候选仍为原代际，原目录ID/创建时刻、当前测量克隆及HKCU指针/TASK摘要保持，还原未执行；原两次退出超时/正常退出码未知不改。需正常退出后另档恢复结果，不重跑本场景取样或原配对。原+10MiB预算继续failed，b4/823及Mac失败、bd2实际失败窗口缺失、严格恢复与其余全部预算/周期/八门禁保持；DPI专项取消，TASK revision2/in_progress/pending、禁止合并/关闭/签名/晋升/发布不变。

### 2026-10-16 工具完整CI终态与正式019原始构建版本补证

同3a2ff08a首次push CI38910385568/attempt1整轮success，五必需job原日志actual checkout均同源；前端99/535、Mac35组923/6ignored、Windows35组648/7ignored及最终EXE冒烟通过，main-only build正常skip。该终态晚于21:51快照，上一档方法准入时wholeCiAdmitted:false保留，不倒改时序，也不混称b4候选或宿主门禁通过。

只读取得正式019/de6862d4原Release37871899116 Windows作业原日志，与既有asset623593980/b33dd0c8及原b4候选package日志绑定。七核心包Compiling行实际相同：tauri/runtime/runtime-wry2.12.1、tao0.37.1、wry0.57.0、webview2-com/sys0.39.1；未从下载行或缺失旧Cargo.lock推断。原生updater2.13.2→2.13.1及其它依赖差异保留，不能称所有依赖/特征/二进制相同；019实际WebView子映像当时未采集，不补历史PID身份。证据收窄“核心框架版本升级”假设，不解释具体GL分配或解除RSS失败。

21份副本、原ANSI及文件名拼写解析错误/修订、全CI与依赖独立复核见`.adg/evidence/OCX-0110-20261016-WIN-TOOL-CI-BUILD-VERSIONS/`。22:10:21原候选仍存活，原目录/克隆身份、指针及TASK摘要保持，还原未执行；原退出超时和正常退出码未知不改。原+20.040155MiB相对+10MiB继续failed，原b4/823及Mac失败、bd2实际失败窗口、严格恢复和其余全部预算/周期/八检查保持。无新产品启动、前台输入、查询回放、签名或发布；DPI专项取消，TASK revision2/in_progress/pending、不合并/关闭状态不变。

### 2026-10-16 019/b4角色RSS时间线补充，未解除超预算

基于冻结的正式019与b4各597条、至少600秒原始样本，按同轮实际PID+创建时刻离线归账。GPU角色均值差`+24.0863758899MiB`，前60样本均值差`+24.8953776042MiB`，后60样本均值差`+21.2761718750MiB`；b4 GPU自身首末差`−2.265625MiB`，019同期`+11.80078125MiB`。差值主要在计数开始时已经存在，不能从本轮样本声称候选持续单调增长或已证明泄漏。

该结果只把排查方向收窄到启动后的GPU资源构成/初始状态差异，不能识别具体WebGL、驱动或页面分配，工作集页分类仍缺019基线，也不能替换原RSS公式。`.adg/evidence/OCX-0110-20261016-WIN-RSS-ROLE-TIMELINE/`保存计算脚本、输入文件SHA、先前冻结证据绑定和独立复核；原`+20.040155MiB`相对`+10MiB`预算继续failed，退出超时、配置未还原、bd2实际失败窗口及其余全部门禁保持。DPI取消，TASK revision2/in_progress/pending，不合并、关闭、签名、晋升或发布。

### 2026-10-16 用户退出后恢复原配置，历史退出和性能失败保持

收到用户“已退出”后22:41:22确认native0，沿冻结恢复脚本执行一次，22:42:01恢复原Roaming目录并保留测量克隆为baseline-measured。22:43:55只读复核原目录ID/创建时刻/实际LocalCache路径、原偏好和HKCU指针摘要与测试前相同，克隆ID/创建时刻保持；两版已记录14个进程代际均结束、前后native0，无产品启动/UI输入/强杀或数据删除。

`.adg/evidence/OCX-0110-20261016-WIN-B4-RESTORATION/`保存恢复标记原字节、独立读回、冻结来源绑定与新回执。仅配置恢复和当前代际清理子集通过，不改写旧档当时的未恢复事实；候选退出码及确切内核退出时刻仍未知，原两次退出超时、完整配对未准入和RSS+20.040155MiB超+10MiB继续保持。严格产品恢复交接、bd2实际失败窗口、Mac历史失败及其余全部预算/周期/八检查未完成；TASK revision2/in_progress/pending、DPI取消及禁止合并/关闭/签名/晋升/发布不变。

### 2026-10-16 独立组件转储方法在录制前停止，PR已同步恢复状态

22:51仅一次独立后台Edge方法检查，根保留句柄先绑定；在Tracing.getCategories返回值的类别前置处拒绝，未调用Tracing.start、取得转储、创建合成纹理或运行产品/已撤回原型，未自动重试。类别数组及Browser.getVersion返回值未在断言前保存，不能补原件或推断真实产品WebView能力。根同代际内核退出/exit0及观察器exit0核验，完整后代集合未取，不冒称全后代清理；后续native0及已恢复偏好/指针/TASK摘要保持，具体分配根因仍缺。

WPR只读盘点取得VirtualAllocation/GPU/ResidentSet配置，默认组合缓冲配置1772MiB仅为配置量，未采用为轻量采集、未录制/停止/取消会话。草稿PR6一次正文更新并GET读回，已将配置未恢复的过时状态改为22:42恢复及14代际结束，保留RSS失败、退出超时/退出码未知，仍draft/open/unmerged。证据见`.adg/evidence/OCX-0110-20261016-WIN-MEMORY-METHOD-PREFLIGHT/`；未修改产品、重跑配对/查询或降低门禁，TASK revision2/in_progress/pending、Mac及全部其它原依赖/预算/周期保持，不关闭或发布。

### 2026-10-16 独立WPR分配配置准备与同源Windows方法通过

工具494fc830已在feature/0.1.10-windows-working-set-pages提交推送读回，只增test/smoke生成器/校验与CI步骤，未改产品。由本机系统导出构建独立File配置，两collector共32MiB缓冲配置，保留进程/映像/虚拟分配与分配栈、三图形provider原参数。配置量不是实际全进程内存或ETL上限，不替换原RSS/CPU等预算。

本机18项及首次push CI38920950285的Windows方法作业117040410248均通过；actual checkout494、制品摘要/大小/CRC、六份提取文件与原日志核验，源码LF/CRLF身份分别绑定。核验时整轮仍运行，仅只读准备通过；事件丢失、文件增长、实际分配/目标代际与会话独占收尾均未证明，录制执行未准入，不以本通过自动申请或延长前台。

`.adg/evidence/OCX-0110-20261016-WIN-WPR-ALLOCATION-PREP/`保存运行原件、实现引用和独立XML/CI核对。23:07:43native0、已恢复原目录/偏好/指针/TASK摘要保持，无ETW/产品/查询执行。原RSS+20.040155MiB超+10MiB及退出失败、bd2实际失败窗口、严格恢复、Mac和全部其它原预算/周期/八检查保持；DPI取消，TASK revision2/in_progress/pending，不合并、关闭、签名、晋升或发布。

### 2026-10-16 受控自进程分配栈链路通过，产品根因未解除

工具a5a1aba2已推送读回，只读准备494的首次完整CI38920950285已success，五必需job原日志actual checkout均494；该全CI不套用到a5，旧准备时的未完成快照保留。a5固定只采当前控制程序事件，使用自有实时内核会话及原生按名停止器；不写全系统ETL，不调用通用Stop或写全局采样配置，未测量全局采样前后状态，未执行原bd2库/官方查询Node或产品。

本机普通令牌实际拒绝录制，10外层/20编译守卫通过。首次push CI38925613291 Windows方法作业117056254920在实际管理员令牌下15外层/20编译守卫通过，actual checkout a5、制品摘要/大小/CRC、提取原件与原日志核验。取得1217条/723913字节自进程事件，8MiB分配及对应栈1对，释放成功，丢失/遗漏/解码错误0，会话停止、读取线程自然排空。CI未执行普通令牌分支，不把两侧令牌检查混称产品双令牌验收。

核验时a5整轮CI仍运行；真实GPU目标绑定入口与产品分配根因未准入。`.adg/evidence/OCX-0110-20261016-WIN-CONTROLLED-MEMORY-PROOF/`独立复算原事件、源码/CI及关闭证据；CRLF/LF提交检查拒绝与元数据表述修订的原日志/私有快照保留。23:44:09native0、已恢复原配置/指针/TASK摘要保持，无宿主录制或产品启动。原RSS超预算、退出失败、bd2实际失败窗口缺失、Mac及其余全部预算/周期/八检查保持；DPI取消，TASK revision2/in_progress/pending，不合并、关闭、签名、晋升或发布。

### 2026-10-16 a5首次完整工具CI通过，原产品失败保持

a5a1aba2首次push CI38925613291/attempt1整轮success，五必需job原日志actual checkout均为同一a5，main-only build正常skip；前端99/535、Mac35组923/6ignored、Windows35组648/7ignored及最终EXE冒烟通过。仅沿原运行取得API run/jobs与五份原日志，没有重派/重试CI或追加查询回放。

完整核验见`.adg/evidence/OCX-0110-20261016-WIN-MEMORY-TOOL-FULL-CI/`；旧受控方法档中“当时整轮仍运行”及currentToolWholeCiAdmitted:false保持，不倒改时序。只准入完整工具CI，真实GPU目标绑定与产品根因仍未准入，RSS+20.040155MiB超+10MiB、原退出失败、bd2实际失败窗口、Mac及其余全部预算/周期/八检查继续保持。TASK revision2/in_progress/pending、原配置恢复、DPI取消及禁止合并/关闭/签名/晋升/发布不变。

### 2026-10-17 fdb只读绑定方法通过，IPC两项超时及独立夹具修订待验

fdb7e3cb已推送读回，只读绑定方法本机和首次CI38930174628 Windows方法作业各16项通过，actual checkout/制品摘要/CRC及LF/CRLF身份核验；真实产品GPU分支未执行，录制未准入。同一CI的Windows库回归549passed/2failed/4ignored，两项取消同步/备份观察测试在门闩释放后等待终态的2秒处Elapsed。isolated query panic属另项夹具预期输出，不当作失败根因；原线程/调度/I/O根因未取得，不重跑旧CI。

失败制品12059460774/7d536e17…/8875175字节及原库0490fb14…已只读验CRC/摘要并私有保存，未执行；它不同于原bd2固定库497814ca…，不替代或解除原失败。独立feature/0.1.10-windows-ipc-idle-signal从b4提交测试专用许可释放通知，替换指定两项热轮询，原24处2秒期限、默认并发、审计/终态断言全部保留；ea8首次CI格式失败原件保持，ca89fbac仅按rustfmt要求修正一行，新SHA首轮CI38933650482已过Windows格式/Clippy/回归步骤，整轮终态仍待核。本机未装Rust，未声称本机编译通过。

证据`.adg/evidence/OCX-0110-20261017-WIN-BINDING-IPC-FAILURE/`分别封存方法通过、整轮失败、源码边界及修订阶段状态。此修订不被写成根因已证实或旧失败已清除；原RSS/退出失败、bd2实际失败窗口、严格恢复、Mac及全部其余预算/周期/八检查保持。TASK revision2/in_progress/pending、原配置恢复、DPI取消，不合并、关闭、签名、晋升或发布。

### 2026-10-17 IPC测试通知修订首次完整CI通过

ca89fbac首次push CI38933650482/attempt1整轮success，五必需job原日志actual checkout均同SHA，main-only build正常skip。Windows35组651passed/0failed/7ignored、Mac35组924passed/0failed/6ignored，前端99文件538passed；原两项取消观察测试及新增最终许可通知用例均有明确ok行。Windows库554passed/0failed/4ignored，最终EXE冒烟步骤通过；以上属于runner验证，不代替本机原生门禁。

制品12060483211/30672f57…/9583894字节只读核API摘要、成员摘要及CRC，修订库a3c4afd1…保留在私有忽略目录且未执行，不替换fdb失败库或原bd2库。`.adg/evidence/OCX-0110-20261017-WIN-IPC-IDLE-FULL-CI/`保存终态、五份原日志、制品身份与独立复核；上一档“当时整轮待核”原件保持。修订保留2秒期限、默认并发、终态及审计断言，未证明旧失败的调度/I/O因果。无重派CI或宿主产品启动；RSS超预算、退出失败、bd2实际失败窗口、严格恢复、Mac及其余原门禁保持，TASK revision2/in_progress/pending、DPI取消，不合并、关闭或发布。

### 2026-10-17 绑定GPU采集入口完整工具CI通过，真实产品诊断尚未执行

工具391e7097已提交推送，首次push CI38934816693/attempt1整轮success，五必需job原日志actual checkout均同SHA；前端99文件535passed、Mac35组923passed/6ignored、Windows35组648passed/7ignored及最终EXE冒烟通过。该工具分支未合入ca89测试通知修订，不能用本次成功清除fdb历史失败或证明其原因。方法制品12059879326/62f039d7…/76710字节核API摘要、CRC及11份提取文本原件；LF/CRLF源码身份分别核验。

本机普通令牌10外层/20编译守卫、18绑定检查及8入口拒绝检查通过；CI实际管理员令牌15外层/20编译守卫和18绑定检查通过，受控自进程取得1225条/746512字节事件、8MiB分配与对应栈1对，丢失/遗漏/解码错误0，探针释放、自有会话消失、读取线程自然排空。只证明自进程链路和准备守卫，真实产品GPU绑定/录制未执行；GPU模式即使质量通过也可能无分配事件，必须另查allocationEvidenceObserved，不能把空记录解释为零RSS或已获根因。

`.adg/evidence/OCX-0110-20261017-WIN-BOUND-GPU-PREP/`封存完整CI、方法原件、宿主只读前置和下一诊断计划。01:08:44核native0、原Roaming目录ID/创建时刻/偏好、原HKCU指针、隔离根及精确b4/f3 EXE与固定解码器；未启动产品、移动原配置或录制。下一次仅安排一个b4实例、约15分钟前台、单次30秒录制（上限45秒），当代实际普通产品令牌及GPU代际/角色/祖先绑定，正常设置→概览返回一次以触发既有mesh销毁重建，并提前正常退出/保留退出观察。实际控制器当前非管理员，仍需实际管理员采集器及新的直接人工前台信号，准备和CI通过均不授予执行窗口。

原019/b4计数不重做，RSS+20.040155MiB超+10MiB、退出超时/退出码未知、bd2实际失败窗口、严格恢复、Mac及其它全部原预算/周期/八检查保持。TASK revision2/in_progress/pending、原配置恢复和DPI取消不变，不合并、关闭、签名、晋升或发布。

### 2026-10-17 实际GPU准备入口作用域失败，正常退出并修复工具

新人工“开始前台验证”窗口02:08:29–02:38:29内，同用户管理员后台采集器实际true/type2就绪后，仅启动一次精确b4/f3普通false/type3产品实例。两者TokenUser摘要一致，产品首屏及设置页实际观察；当代GPU角色、PID/100ns创建时刻、映像路径/摘要及角色快照已保存。Prepare入口初次Open返回后进入Invoke复核，因Get-WindowsProcessToken在Open局部加载、返回后消失而失败；错误仅有明确标注的工具输出派生观察，未冒称已保存原始stderr或完整初次绑定见证。原自进程测试复核绕过真实GPU令牌/角色分支，18项通过不能证明该路径可用。

没有批准录制请求或创建采集输出；随后给管理员等待器发送recordApproved:false/seconds0的取消包，等待器拒绝并退出，无ETW会话启动。02:14:36对实际窗口发送正常Alt+F4，保留句柄取得产品exit0及内核退出时刻；148条原普查最大native1，所有活跃记录匹配绑定root，七个记录代际均已结束。02:16:31释放前台，原Roaming目录ID/创建时刻、偏好与HKCU指针、TASK摘要保持，无原目录移动、数据删除、强杀、重复实例或配对。

工具fde1dbb5已中文提交推送读回，只改test/smoke两文件：令牌/角色读取器在绑定方法共同调用作用域加载，新增Open返回后可调用性、真实自进程令牌再读和角色解析四项回归。新增回归先在旧实现明确失败，修复后本机22项通过；受控自进程10外层/20编译守卫及普通令牌拒绝仍通过。产品apps/desktop tree未改，录制核心、CLI、时限、身份/单实例断言及缓冲/文件上限均未改。

首次CI38946198178的Windows方法作业117125807346通过，actual checkout fde、制品12064933397/c2acc976…/77413字节API摘要/CRC及提取原件核验；CI22绑定检查、15外层/20编译守卫和自进程8MiB分配/栈1对通过，1257条/765616字节、丢失/遗漏/解码错误0、自有会话清理。核验时整轮仍在运行，修复后的真实GPU分支未重试。证据`.adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FAILURE/`保留现场、正常退出、首次失败、源码边界和方法阶段状态，后续沿同运行取得终态；新工具完整CI与新的人工窗口就绪后才另开产品诊断。

本轮仅修复采集工具，不认定产品RSS根因；原+20.040155MiB超+10MiB、历史配对退出超时/退出码未知、bd2实际失败窗口、严格恢复、Mac及全部其它原预算/周期/八检查保持。DPI专项取消，TASK revision2/in_progress/pending、不合并/关闭/签名/晋升/发布不变。

### 2026-10-17 复用既有真实30分钟退避子集，避免重复计时

只读重验`.adg/evidence/OCX-0110-20261015-WIN-REAL-STATE-FLOOR/`的57份冻结文件、两次实际planner错误/成功回调及原JSON单调间隔1800007.1000000238ms、零调试暂停与正常退出。实际执行始终标为02/7383 EXE；四份原调度文件在02、当前产品b4和工具fde的Git blob完全相同，新增`.adg/evidence/OCX-0110-20261017-WIN-STATE-FLOOR-REUSE/`记录精确源码和原件摘要绑定，不重标EXE或启动新计时。

按原已声明的文件/行为范围，复用“真实状态错误→30分钟下限→成功重试、合成online唤醒未提前触发”子集；后续不再把该已验证子集泛列为全缺失而重做30分钟等待。选定查询IPC观察0不外推为全部网络/短命进程0。真实24h/6h、休眠/回拨、全部原生唤醒/手动入口、原始请求与子进程、50ms及完整调度/性能门禁仍未证明；RSS与bd2/Mac历史失败保持。无应用启动、前台输入或周期重置，TASK revision2/in_progress/pending、DPI取消及禁止合并/关闭/发布不变。

### 2026-10-17 GPU工具作用域修复首次完整CI通过，下一窗口准备就绪

fde1dbb5首次push CI38946198178/attempt1整轮success，五必需job原日志actual checkout均同SHA；前端99文件535passed、Mac35组923passed/6ignored、Windows35组648passed/7ignored及最终EXE冒烟通过，main-only build正常skip。此工具分支未合入ca89测试通知修订，指定两项取消观察测试本轮ok不清除fdb/bd2原失败，不建立旧调度/I/O因果。旧档“当时整轮运行中”的快照保持。

下一诊断入口改绑fde及同运行完整CI，仍为精确b4/f3一个普通实例、30秒录制/45秒上限与原身份/输出预算。Prepare用独立PowerShell进程，调用者导入函数不能掩盖CLI作用域错误；成功或失败均保留解码后的stdout/stderr及真实退出结果，输出不冒称原始管道字节。缺新人工回执、仅方法CI、旧源完整CI三项实际拒绝和四脚本语法通过。准备目录无人工回执、产品/GPU启动包、录制请求或采集输出，不启动/提权/录制；下一次从新运行目录使用这些方法，不复用旧窗口或旧PID。

`.adg/evidence/OCX-0110-20261017-WIN-GPU-SCOPE-FULL-CI/`保存五原日志、终态与准备方法。02:41:28再次只读核native0、原目录ID/创建时刻、偏好/指针/TASK摘要保持；原前台已释放且02:38:29截止已过，完整CI不续授权。实际修复后产品GPU准备/录制仍未执行；后续需新的人工开始信号和实际管理员采集器，原019/b4计数不重做。

原RSS+20.040155MiB超+10MiB、历史退出超时/退出码未知、bd2实际失败窗口、严格恢复、Mac及其它全部原预算/周期/八检查保持。已有真实30分钟退避子集按上一档范围复用，DPI专项取消，TASK revision2/in_progress/pending、不合并/关闭/签名/晋升/发布不变。

### 2026-10-17 实际GPU单次分配栈录制通过，原RSS根因仍未建立

新人工“开始前台验证”后，首次核验时刻08:46:09作为本轮30分钟窗口回执起点（不冒称精确用户消息时间）；修复工具fde/首次完整CI38946198178与精确b4/f3候选、隔离根、原配置再次核同。真实同用户管理员采集器true/type2先就绪，08:47:26仅启动一个普通false/type3产品，非sandbox/CDP。真实设置页下独立Prepare子进程exit0，令牌/角色/祖先复核可用，成功JSON和解码stdout/stderr分别保留；上轮失败不覆盖或自动重试。

08:48:49–08:49:20仅执行一次声明30秒的实时GPU采集，实际过程含启停约31.27秒，32MiB配置缓冲、4096条/4MiB输出上限不变。08:49:07按当前可访问树从设置返回概览一次，08:49:15实际观察概览及光层。root/GPU在前后绑定中PID、100ns创建时刻、映像SHA、同TokenUser及真实GPU角色/三进程祖先链一致；581条/269538字节、231次分配与231栈、119次free，分配证据非空，丢失/遗漏/解码错误0，自有会话已停止消失、读取线程自然排空。无探针或目标内存内容读写、全系统ETL、通用Stop/全局采样写入，也未执行原bd2库/官方查询Node。

同GPU代际在录制后取得14份命中模块的地址范围/版本/磁盘SHA，744个不同栈地址中705匹配、39内核地址未解析，未作函数符号解析或事件时刻模块装载证明。复算返回动作至首次概览观察间有209次分配；包含igd10um64xe.DLL范围的57栈事件长度合计72507392字节，D3D11范围82栈，UIAutomation范围5栈。模块包含统计互相重叠，不能相加；按录后模块范围归类，只收窄图形资源重建方向，不能证明具体WebGL纹理/驱动缺陷或原+20MiB差值。未保存分配地址，alloc/free无法身份配对，事件长度总和76189696不等于净提交、存活量、物理量或RSS。设置/概览前后GPU工作集单点264118272→239390720字节也不替代原平均口径、基线或泄漏结论，工具观察开销未排除。

08:50:23正常Alt+F4后同保留句柄取得exit0/内核退出时刻，251条原普查最大native1且所有活跃记录匹配root，七个记录代际均结束；08:51:01释放前台，原Roaming目录ID/创建时刻/偏好、HKCU指针和TASK摘要保持。`.adg/evidence/OCX-0110-20261017-WIN-GPU-ALLOCATION-RUN/`封存本轮原件、动作时间、模块映射、独立事件/栈复算及退出/还原读回。仅真实采集链路和该场景分配证据通过，工具/产品代码未新增修改，未重跑019/b4正式计数。

下一取证需对正式019的对应资源重建取得同方法模块/分配构成或对这些模块偏移作可信符号解析，才能比较具体差异；不依据单份候选事件猜改分辨率、光层边界或画质，不重复未改候选的正式性能配对。原RSS+20.040155MiB超+10MiB、历史配对退出超时/退出码未知、bd2实际失败线程/调度/I/O窗口、严格恢复、Mac及其它全部原预算/周期/八检查保持。已有真实30分钟退避子集继续按范围复用，DPI取消，TASK revision2/in_progress/pending，不合并/关闭/签名/晋升/发布。

### 2026-10-17 收尾固定清单与WebGL资源生命周期修复

按用户更新后的目标，后续以产品修复和必要补测推进。以下八行是固定剩余清单；已通过子集按源码影响复用，不因总门禁未通过而整套重测，不新增符号解析、驱动归因或反复ETW录制为验收前置。原RSS预算仍为+10MiB；本地main@94ec0deb交接文档中“接受+30MiB”“CI确定为环境原因”均未获用户裁决，也没有代替实际修复和失败分析，不纳入通过依据。该本地提交保留，未推送/改写历史。

2026-10-18 收尾更新：用户明确同意停止继续扩展采样并收尾。下表保留原八检查及未验边界，用于版本整合时判断风险，不再作为本对话自动补采队列；表中“仅补”等表述属于此前执行计划。现行 CPU/RSS 结论以表内用户已接受的实测值为准，DPI 取消。未测不改为通过，原验收要求未删除，后续专项须有新的明确执行安排。

| 原检查 | 已有可复用证据 | 固定待解决项 |
|---|---|---|
| review:scope-0110 | W1–W6、DPI取消、八检查及分支归属已冻结 | 最终修复候选的范围审阅和逐项证据汇总；不扩大功能 |
| review:scheduler-performance-0110 | 真实30分钟错误退避子集；019/ce完整配对及窗口内正常退出；CPU+0.661761个百分点/RSS+20.193866MiB已获用户接受；10月13日沿原24h截止跨启动后stable/面板成功持久化及下次86400秒已有真实原件；10月18日官方019/8e同机原生启动冷/热各20、共80样本全部正常exit0，冷p95 1354.5511→1273.1190ms、热1111.7454→1103.4023ms，原100ms且5%双预算通过 | 不重复已接受CPU/RSS、已通过启动配对及原截止跨启动到期行为，不重新计24h。仅补帧/50ms、100路由/10重启零自动查询、原始请求/子进程、休眠恢复及确实未证的自动触发/周期部分，不把持久成功或启动子项冒充完整性能门禁 |
| review:storage-0110 | 既有自定义根同SID双令牌；本机ce实际安装data、正常重启同根/偏好回读、真实备份及卸载保留；99空目录初始化、人工恢复选择→持久指针→自动重启顺序和原生根回读；8e复用该实际根完成普通重启同根/HOME、四文件摘要回读及两次正常exit0；原配置已恢复 | 本轮普通重启/正常退出缺口已补，99旧代未知退出事实保留，不再重放初始化/选择求回填旧退出码；最终范围审阅按真实证据汇总，不新增门槛 |
| review:installer-0110 | 真机MSI旧包→019→823、NSIS019→ce及实际data/自定义根默认保留；8e完整CI/唯一候选、runner确认/外部HOME/默认保留、本机真实勾选清理和18文件副本；真实产品双令牌既有证据；本轮8e受保护Program Files实际NSIS安装、普通产品首屏/写权限拒绝及默认卸载后四文件保持 | 受保护NSIS普通场景已抵扣，不重复安装/卸载；管理员产品剩余原生组合未执行，与native项一起仅补实际缺项。旧未知退出码/本轮父卸载器立即检查失败与最终子退出码未知保留，不新增重测门槛，旧CI失败不重派 |
| review:updates-0110 | 共用详情、视图切换键盘焦点、跨路由终态复用、手动stable成功及beta失败返回stable保留状态 | 真实下载/验证/应用进度、在途隐藏/跨路由、可取消阶段/失败重试及重启版本回读；终态详情不替代在途进度。可信更高管理器版本和beta来源单列，不发布制造条件 |
| review:backup-tree-0110 | 不同值真实恢复、摘要损坏拒绝且不覆盖、保护保留、轮换/固定/重启；原生已观察键盘/刷新/打开警告子集；原型同步范围、备份树、导入比较、模型策略/模版比较已补浏览器UI，52组修复后通过，备份动作8→0px且命中高度保持36px | 仅补原生越界/关联组清理未覆盖分支、异步刷新叶焦点及原生浅深/三档/密度剩余对齐组合；原型其它未覆盖状态保留，不把浏览器结果代替原生，也不重做已通过备份/键盘/原型用例 |
| review:event-coverage-0110 | 注册/发出点盘点；同配置保存精确解除；恢复同候选成功解除，其他动作/候选/轮换不误解除；切通道保留面板调度/通知，返回stable不清beta失败 | 安装/更新尚缺的真实终态和对象/阶段/候选隔离，写入/在途通道隔离；静态注册和其他成功不替代对应终态 |
| review:native-0110 | 普通托盘隐藏/人工重开/正常退出、标题栏及最小化/恢复DWM转换；管理员真实产品与用户确认最大化最小化/活动云母日志；本轮8e普通活动DWM2、人工左半屏Snap及同PID/HWND布局回读、按托盘退出指引的人工反馈和同代内核exit0 | 本轮实际退出晚于30分钟截止，按执行缺口留档，不写作窗口内收尾。Snap/人工退出场景不再重演；仅补管理员活动云母同步原生状态及原画质剩余组合，不做专项DPI |

外部依赖单列：Mac原性能失败/原生验收保持；beta真实6h尚缺可用可信来源（最近冻结来源证据为404），不修改来源或发布beta制造条件。bd2实际失败窗口仍未取得，保留失败与既有单次成功对照，停止碰运气回放。上述依赖不写为Windows产品通过，也不要求本线程重复Mac工作。

本轮直接修复可复现的生产缺口：能力探测先创建1×1无深度/模板缓冲的上下文，得到结果后主动释放；用WeakSet标记主动销毁的canvas，使全局contextlost/restored监听不将正常销毁判作设备故障，活跃上下文真实故障仍回退CSS；shader/程序/顶点缓冲创建失败也释放上下文，destroy幂等。两项新增回归在旧实现分别复现300×150探针未释放与正常销毁后意外CSS降级，修复后相关25项、完整100文件540项、类型与构建通过。复用既有后台Edge像素对照，四布局×两主题×三个时刻共24组RGBA和合成PNG逐字节相同，着色器、最终分辨率、CSS光层范围及三档策略未改。

产品代码ce670ce7基于ca89，已推送feature/0.1.10-windows-webgl-lifetime并读回；首次CI38995008914正在运行，完整CI和新候选尚未准入。本轮无原生产品启动/配对或官方查询Node/bd2回放。资源释放缺口已修复不等于原+20MiB根因被证明或内存预算通过；完整CI后只构建一个新候选，在新人工窗口测实际收益，再处理本表其它缺口。证据见`.adg/evidence/OCX-0110-20261017-WIN-WEBGL-LIFETIME/`。TASK revision2/in_progress/pending、原失败和禁止合并/关闭/签名/晋升/发布保持。

ce首次CI38995008914现已整轮success，五份必需job原日志actual checkout均ce；前端100/540、Mac35组924/6ignored、Windows35组651/7ignored及最终EXE冒烟通过。仅请求一次候选38996189227/attempt1，package作业117289165536正在运行，query-etw/retention按purpose跳过，尚无候选EXE准入，不重发。新候选与正式019将沿原方法各稳定60秒、完整计数600秒，以原CPU+0.5pp/RSS+10MiB判断；下次约27分钟前台，计数结束立即正常关闭，预留配置恢复余量。本轮仍native0。

同时更正上一段故障表述的强度：真实后台Edge检查中，loseContext后立即移除canvas，target收到一次异步丢失事件，但document捕获为0；canvas仍连接时document捕获为1。因此单元测试“正常销毁误降级”只复现同步模拟事件或仍连接节点的策略风险，不能宣称已证实原生卸载时会意外降级，更不能解释原RSS差值。WeakSet属于防御性修正；探测上下文原来未主动释放是确定的源码缺口，其实际回收幅度仍待候选实测。规范推荐loseContext释放底层图形资源，但未给出本应用RSS保证。该限定、真实事件时序原件和同源完整CI保存于原证据目录的terminal子档；旧阶段快照不改写。

### 2026-10-17 ce唯一候选核验完成，原口径RSS对照可开始

唯一候选38996189227/attempt1完整success，package117289165536实际checkout ce670ce7，原日志35组651passed/0failed/7ignored；runner MSI/NSIS安装、实际载荷首屏及卸载通过，其它两个purpose作业跳过。两ZIP分别12087722474/3b4b502f…/7668256字节和12087837119/e69b9705…/1702839字节，API摘要、大小、全部成员摘要和CRC核验；完整同SHA CI38995008914沿前档复用。只读提取MSI的0.1.10/8830976字节主EXE为4fe09bce265b51860da17f2cd1466a50430513e5ed29050f85caf5d913652bb5，与runner实装载荷相同；沿实际8148685偏移仅在内存替换三字节包类型，精确匹配NSIS及原构建EXE摘要，原包和EXE未改。提取前方法摘要因Windows文本LF→CRLF转换拒绝一次，未运行提取器；归一后源码字节相同，原拒绝与修正说明保留，未重构建或关闭验证。

`.adg/evidence/OCX-0110-20261017-WIN-CE-CANDIDATE/`记录候选与CI来源，私有ZIP保留在原test/software目录；本机未安装或启动产品。`.adg/evidence/OCX-0110-20261017-WIN-CE-FRONT-PREP/`复用原019/b4采集方法，仅更换候选身份和运行绑定，开始candidate/计数前分别保留840/750秒余量以供真实稳定60秒、完整600秒及正常退出/还原；这些是操作时间安排，不改CPU+0.5pp/RSS+10MiB指标、计数跨度或样本口径。九份PowerShell语法与四个缺新人工信号实际拒绝通过，原配置ID/偏好/指针/TASK前后相同、native0；无新人工回执、配置移动、产品启动或采样。

下一次新人工窗口预计27分钟：正式019→新ce，各稳定60秒/完整计数600秒，计完立即正常关闭并核保留句柄exit0，最后恢复原配置、保留测量克隆；最迟第25分钟正常关闭目标与30分钟总时限保持。不得用本次新启动的退出证据倒填旧配对生命周期，也不重复未修改b4版本的正式配对。只有本轮产品代码改变才安排此受影响复测；RSS收益仍未测量，原+20.040155MiB失败、固定剩余清单、真实周期/Mac依赖与禁止合并/关闭/签名/晋升/发布保持。

### 2026-10-17 ce前台配对完成，CPU与RSS均未达原预算

新人工“开始前台验证”后首个本机核时为11:30:45.556+08:00，按此保守限定至12:00:45.556；未延长。沿已核验的唯一ce候选、正式019和原采集方法顺序执行，同SID普通令牌、单原生实例、同准备页及2360×1520物理客户区，两版均实际稳定至少60秒，无沙箱/CDP/ETW。结果如下，CPU仍按一个逻辑处理器百分比计算，RSS为原进程家族工作集求和均值，未扣共享页或改归一方式。

| 本轮版本 | 完整计数跨度 | 样本数 | CPU | 平均RSS |
|---|---:|---:|---:|---:|
| 正式0.1.9 / de6862d4 | 600.2428945秒 | 597 | 13.778276% | 544.709472 MiB |
| 0.1.10 / ce670ce7 | 600.3273912秒 | 597 | 14.440037% | 564.903338 MiB |
| 候选减基线 | — | — | **+0.661761个百分点，超+0.5预算** | **+20.193866 MiB，超+10预算** |

两段read/frame失败均为0，采集器exit0。019于11:44:56.930、ce于11:56:48.412由实际Alt+F4正常退出，保留同一代进程句柄所得exit0和内核时间；候选启动晚于019退出，整个过程最多一个原生实例。11:57:00恢复原配置目录，保留测量克隆；最终原目录ID/创建时间、两处偏好、HKCU启动指针和TASK文件摘要与准备时一致，native0，已归还前台。新生命周期只属于本轮，不修补历史超时或未知退出码。

十进制独立复算与原分析一致。按计数前实际PID/创建时间对应的角色，GPU进程均值差+23.226262 MiB，仅说明差值所在，不证明具体分配或根因；计数期间另有一个新WebView代际在发现时已退出、工作集为0，保留累计CPU并标记角色未知，不凭旧快照推断。首次离线归档因该代际不在快照而KeyError，修正为如实保留未知角色后复核通过；原计数、预算及正式尝试均未重做。证据、该归档修正和37份原件见`.adg/evidence/OCX-0110-20261017-WIN-CE-IDLE-PAIR/`。

结论：ce资源释放缺口修复没有使本机性能达到原预算，本轮CPU也失败；不能把与旧b4差值相近解释成收益、稳定性或环境原因已证实。`review:scheduler-performance-0110`保持failed，完整性能及TASK仍未完成；固定八行清单、既有失败、真实周期和Mac依赖保持。不重复未变ce候选的正式配对；下一步只针对真实产品差异形成可验证修复，再安排其受影响补测，不增加ETW/符号解析前置，不自动合并、关闭、签名、晋升或发布。

### 2026-10-17 用户接受本轮Windows空闲资源增量

用户在得知上述实测与原预算后明确表示：“我认为完全是可以接受水平，如果提升有限，就可以接受了。”据此接受0.1.10候选ce670ce76bb1c29f70989ee54838d5b31e97059c（EXE SHA256 `4fe09bce265b51860da17f2cd1466a50430513e5ed29050f85caf5d913652bb5`）在本机、同准备页/light/high/mesh/100%场景相对正式019的CPU **+0.661761个百分点**、平均RSS **+20.193866 MiB**，作为本轮原预算偏差的人工验收例外。前述产品修复经本轮测量未达原预算，继续为这项偏差优化的收益尚未证明，停止该项优化、ETW取证与正式配对循环；该偏差不再阻塞后续验收。

本裁决从此处起取代上一节“下一步继续修复该性能偏差”的安排。原计数、超预算事实、失败回执及11f17b3b证据提交保留，未改写为自动达标，也不将本地main交接中的“+30MiB”追认为通用预算。裁决仅覆盖上述Windows空闲配对增量；完整调度/性能检查的其余必要缺测、其它七项检查及Mac依赖继续按固定清单处理，不因此重做已通过子集。TASK保持in_progress/pending；完整门禁收口与发布仍分别按原授权办理。

### 2026-10-17 本机NSIS升级与默认卸载保留补齐

按用户“继续”推进原固定清单，不启动前台产品或重测已接受性能。先核本机无已注册管理器安装、native0；正式019 NSIS来自Release407422226/asset623594038，2884185字节、SHA256 `e2538debc1f8acf5b5ed931f3e68d9b128263742c230fbc151513f2833a38006`，API摘要和下载原件相符。ce候选复用同源已准入38996189227，NSIS包`ee10b53f…`，实装主EXE两次均`059bc70d…`，版本0.1.10，与该runner原件匹配。

12:20:50–12:20:52在专用带空格目录静默完成五步：019安装→ce升级→默认卸载→ce重装→再次默认卸载。五个实际安装/卸载目标进程均普通令牌false/type3、exit0；六份安装data/兄弟自定义目录/外部哨兵种子在五个检查阶段字节摘要一致。最终主EXE、卸载注册项均不存在，独立读回无安装器及原生实例，原Roaming偏好、既有绑定夹具偏好、HKCU启动指针前后相同。安装器未带启动参数，未启动产品或注入UI，未清理真实配置。

证据见`.adg/evidence/OCX-0110-20261017-WIN-CE-NSIS-HOST/`。019载荷额外比对首次误用“MSI镜像只有一个包标记”的假设而断言停止，未写EXE或重跑安装器；019来源仍以精确官方NSIS包、实际版本及观测载荷摘要证明，未冒称MSI三字节变体已核验。仅本机NSIS种子保留子集完成；实际应用根绑定、显式勾选清理、受保护位置及产品令牌/首屏未由此替代。下一前台集中补不可写目录恢复的选择→持久绑定→正常重启与安装data/自定义根交接，不再重复MSI/NSIS种子测试。其余固定清单、用户性能接受和TASK in_progress/pending保持。

### 2026-10-17 实际安装data重启与卸载保留通过，空目录初始化缺口修复

新人工“开始前台验证”后的首个核时13:04:44+08:00，限定至13:34:44；13:18:17已提前归还。ce唯一NSIS包安装到本轮专用带空格目录，实际主EXE为059bc70d…；原四条平台路径按两份原生目录ID保存，启动指针和目录ACL原件仅留私有运行目录。

不可写用例只启动一次普通令牌76500，真实显示“安装目录下的数据目录无法写入”。Computer Use无截图读取成功，按钮点击返回`coordinate input geometry is unavailable`；Tab后可访问性仍只报告对话框焦点，Return后窗口随后退出，截图捕获另报`foreground window did not report a process id`。进程保留句柄证实exit0、最大native1，但无选择/新绑定/替换代际，因此恢复交接未通过；不认定产品缺陷，也未自动重试。固定240秒观察结束后才开始另一独立用例，避免把后续手动启动误当自动重启。

恢复专用安装目录原ACL、保持测试无旧绑定后，实际产品正常启动71960。设置页明确当前根及托管HOME分别为`<本轮实际安装目录>\data`及其`opencodex-home`；在UI保存浅色偏好、生成真实手动偏好备份bk_20261017051442_80c65217，并把本轮关闭行为设为正常退出。13:15:54普通实例exit0后再次正常启动64260，原生设置页回读同一data根，浅色偏好和文件摘要相同；13:17:19再次exit0，未重叠原生实例。NSIS静默卸载后，实际偏好、data-root元数据、备份manifest和备份payload四文件逐字节保留。首次卸载器父进程exit0时主EXE尚在是中间快照，另有有界等待和最终读回确认卸载完成、主EXE不存在，未重试卸载。

另两个现场结果按范围保留：对已存在空目录点击“校验并初始化”被报“结构损坏”；对尚不存在目标确认“迁移数据”被`runtime_unverified`阻止，旧绑定保留、未创建迁移根，没有伪造面板来源绕过门禁。后者是未接入面板时的安全阻止，不能算自定义根迁移通过。

已直接修复前者的源码缺口：后端将确认没有任何条目的目录标为`uninitialized`，非空缺元数据仍为corrupted；前端允许valid/uninitialized进入初始化，仅成功后显示结构有效。新增三项跨平台后端回归覆盖空目录只读识别/初始化、非空目录保留、空目录不能直接成为活动引用；前端新增空根和未来版本用例，旧逻辑稳定失败1项，修复后100文件542项、类型与构建通过。代码`feature/0.1.10-empty-root-initialization@edead4c5`已推送并读回；首次CI39006013483正在执行，尚不准入修复候选。本机无Rust工具链，后端结果以同SHA完整CI为准。

现场与本地修复验证见`.adg/evidence/OCX-0110-20261017-WIN-CE-DATA-ROOT/`。原目录ID/创建时间、原偏好及启动指针精确恢复，新产生的WebView目录和实际data均留测试目录，未删除原配置，最终native0。默认根/真实备份/普通重启/实际卸载保留子集完成，后续仅补修复后的初始化、需人工协助的原生选择器与自定义根/明确清理等剩余项；不因存储修复重测已接受CPU/RSS，不合并、关闭、签名、晋升或发布。

同轮CI终态补充：edead首次39006013483整轮failure，唯一失败为macOS旧用例把空目录断言为Corrupted（758passed/1failed/4ignored），新增三项回归已成功，Windows及其它job成功；原失败与日志保留。99a62e4a只将该旧用例的夹具明确为含manager-state、缺元数据的非空目录，Corrupted拒绝断言保留；真实空目录已有新的成功初始化及禁止直接活动引用回归，未删除保护断言。该新提交首次CI39006330384整轮success，五份必需job原日志actual checkout均99a62e4a；前端100文件542项、macOS35组927passed/6ignored、Windows35组654passed/7ignored，格式/Clippy与最终EXE首屏成功。

完整CI后只派发一次候选39007300430/attempt1，绑定admission39006330384及99a62e4a，归档时构建中、尚无制品准入或本机启动。终态原件、源码绑定、首次失败和唯一派发证据保存于原证据目录的`ci/`子档，不倒改上一档的阶段状态。本轮前台实际约14分钟并已还原；候选构建为后台工作，后续通过完整制品核验后才安排短窗口，只补空根初始化和需人工协助的原生目录选择/自定义根，已通过默认根及已接受性能均复用。

### 2026-10-17 空目录修复唯一候选制品核验完成

沿既有唯一运行39007300430/attempt1核得整轮success，package117322280951实际checkout为99a62e4a，Windows35组654passed/0failed/7ignored；runner的MSI/NSIS实装、最终EXE首屏和卸载均成功，其余purpose作业跳过。完整同SHA推送CI39006330384及五份成功job日志沿上一档复用，未重派构建或回放。

两份ZIP的API摘要、大小、成员字节摘要及CRC全部匹配：安装器artifact12090994649、7669517字节、`87294d844f9ca109eb54587f3ec4a8605c81092e6e398df25ed4aca3cbb112a9`；runner证据artifact12092230799、1675741字节、`b19f495e92270946fdd95abe0a0bcba2b0137001b3fc47afad0c254456270ce0`。复用已提交的只读MSI提取方法（98a35f58…），0.1.10主EXE为8831488字节、`43e6deb5b2ea4a1b886c51a0c75c0eb9ca4b19a32126d1d44e5dca55a40535c8`，与runner MSI实装载荷相同；仅在内存按实际8149245偏移替换三字节包后缀，NSIS变体`1ab0d12f…`和原构建变体`2ae2ad6f…`均精确匹配。原EXE、安装器和配置未修改。

证据见`.adg/evidence/OCX-0110-20261017-WIN-99-CANDIDATE/`，原ZIP保留在忽略的候选目录。本轮只有后台核验和惰性载荷提取，本机未安装/启动，native0；该准入只证明修复候选身份及runner冒烟，不代替本机空目录初始化、恢复选择器、实际自定义根保留/明确清理及其它原检查。下一前台从本候选直接补缺失项，若工具仍不能点选原生目录窗口立即交用户操作，不再重启循环；默认根通过子集、已接受CPU/RSS及DPI取消继续复用。TASK in_progress/pending，不合并、关闭、签名、晋升或发布。

### 2026-10-17 补齐显式卸载清理实现，保留默认数据策略

后台检查固定清单中的明确清理项：当前Windows配置未接入自定义卸载处理，固定CLI2.11.4模板的“删除应用数据”只递归清AppData，未解析本期安装data、持久启动锚点及活动自定义根。本结论是源码与构建模板缺口，不冒称已在本机勾选清理并观察失败。继续保留已准入99候选，另从99建立`feature/0.1.10-windows-uninstall-data`修复，最新`1dccf22ec3b93a0b08115eaf794fb45ec9814d2a`已推送。

NSIS仅在用户勾选清理且非UPDATE时调用实装ocxd的固定内部确认命令；普通CLI仍走既有IPC，清理入口不接收任意目标路径、不连接运行实例、不执行官方程序。清理沿已登记启动锚点及有界活动根链，复用现有迁移清单上限，验证结构版本、目录归属、嵌套/循环与链接；完整取得各根实例锁并再次核对绑定后才删除，外部HOME和安装目录内的其它文件保留。持有原用户启动选择锁至结束，成功后清除匹配的StartupDataRootV1；失败停止卸载并保留应用文件，不强杀进程。关闭模板额外的独立AppData递归删除，默认卸载和更新替换保留数据。MSI没有新增清理复选框；其载荷中的同一显式确认助手由后续runner用专用夹具验证，不把它写成MSI原生选择界面已完成。

新增8项跨平台/Unix临时目录回归，覆盖默认根与自定义活动根、外部/尚不存在HOME、只读规划、忙实例、未来/损坏/未知内容、绑定变化和链接目标保护；另补Windows内部命令精确确认参数回归。此前把该参数用例放入Unix限定模块导致Windows不执行，最新1dcc只修正此测试条件，并使原有两项CLI参数回归也能在Windows执行。一次性runner安装器冒烟另补真实实装助手的未确认拒绝、确认清理专用种子锚点/活动根、外置HOME保留、指针解除，以及随后默认卸载保留新写文件；该实装冒烟尚未运行，不能算通过。

实现阶段的首次格式检查失败及Windows E0364/E0603可见性编译失败原件保留，已分别修订，未重跑同SHA求通过。0e81500f的Mac原日志35组935passed/6ignored，其中清理核心8项通过；Windows后端回归步骤成功，归档时最终EXE构建/冒烟尚未整轮核验。最新1dcc首次CI39011042117正在执行，不能用前一提交局部通过替代其完整CI。证据见`.adg/evidence/OCX-0110-20261017-WIN-UNINSTALL-DATA/`；原始阶段验证绑定0e，`latest-code-binding.json`单列1dcc和CI身份，不改写阶段记录。

本轮始终无新人工前台信号，未安装、启动、卸载或清理本机应用/数据，原偏好摘要保持、native0；未派发清理修复候选。后续在最新完整同SHA CI及安装器实装验证通过后，再安排一个集中前台窗口补空目录初始化、人工协助原生选择器、自定义根绑定与保留/清理。默认data已有证据、用户已接受CPU/RSS及DPI取消继续复用；TASK in_progress/pending、Mac/其它外部依赖和其它原检查保持，不合并、关闭、签名、晋升或发布。

同源终态补充：最新1dccf22e首次CI39011042117整轮success，五份必需job原日志actual checkout全部相同；Mac35组935passed/6ignored、Windows35组664passed/7ignored，前端100文件542项、格式/Clippy与Windows最终EXE首屏均通过。Windows新确认参数用例真实执行成功（此前Unix条件遗漏不再沿用），清理核心Windows7项/Mac8项均通过。另在普通本机环境仅执行一次冒烟脚本入口守卫，因缺一次性runner上下文立即exit1拒绝，未调用清理助手、未修改根或指针；不是实际清理通过。

完整CI后只提交一次候选派发；首次POST204后运行发现暂未返回，沿原派发标记只读查询取得39012344655/attempt1，源码1dcc、admission39011042117，未重复POST。归档时package仍运行，query-etw/retention跳过，安装器钩子编译、实装助手清理冒烟及制品未准入；不能把Rust单测或最终EXE普通启动替代安装器执行。终态日志、本机拒绝原件和唯一派发绑定位于原证据目录的`ci/`子档；上一档0e阶段记录和latest-code-binding的当时in_progress保持历史原件。下一步核验这一个候选的同源制品/实装清理结果，再做缺失原生交接，继续不重复性能配对或默认保留子集。

### 2026-10-17 唯一清理候选在版本查询回归失败，停止同SHA重派

39012344655/attempt1现已终态failure，package117337352360实际checkout1dccf22e；“Windows regression with the package lock”中lib562passed/2failed/4ignored即停止，其余测试组及安装器构建、实装助手、上传安装器均未执行。失败为`version_success_reaps_owned_helper_and_preserves_version_output`与`version_nonzero_exit_reaps_owned_helper`，两次实际结果均Timeout（约2.00秒），原5秒夹具观察未见parent-pid/child-pid/ownership-ready；控制Node的同保留句柄在查询前、返回后、观察结束时都仍存活，控制就绪标记仅在观察结束时读到。只证明该次就绪条件没有在原预算内达成，不证明线程调度、I/O、驱动或环境根因，也不证明清理代码引起失败。版本源和查询夹具两份源码经Git blob核对与准入99候选相同；不从源码未改推导失败可忽略。

同源完整推送CI39011042117的935/664全部通过仍是事实，但不能抵销该次打包失败。失败诊断artifact12094175208、278234字节、SHA256 `a3142b734c9089dc2d0f7f5efa956f680440b9a1b37c3c5c1448863027767818`的API身份、五成员摘要和CRC已核；只作为诊断ZIP，不含可准入安装包，不执行其中载荷。原作业日志212190字节、摘要35763088…，终态、逐用例解析及原配置读回保存于`.adg/evidence/OCX-0110-20261017-WIN-UNINSTALL-DATA/candidate-failure/`。

现有夹具明确将冷启动计入2秒查询预算，不预热、不延长期限、不降低并发、不删除断言、不重跑同SHA或bd2碰运气，也不追加ETW工具链；缺少因果证据时不猜改查询生产逻辑。清理修复保持已推送待验，当前阻塞是候选打包前查询回归失败，安装器钩子编译及真实助手清理尚未完成。已准入99候选仍可在新人工前台窗口补空目录初始化和人工协助目录选择，不能用它验证尚未打包的清理修复。原bd2历史失败和其线程/调度/I/O证据缺口不由本轮日志替代。

本轮仅后台读取失败运行和诊断制品；本机未安装、启动、卸载或删除，原偏好、启动指针及TASK摘要保持，native0。固定剩余清单不扩充、不声称Windows完成或可发布；保留Mac/其它依赖、已接受CPU/RSS及DPI取消。下一步必须有可验证修正或明确人工裁决才能重启这条候选链；不靠再次成功覆盖失败，不合并、关闭、签名、晋升或发布。

### 2026-10-17 99原生空目录初始化与恢复绑定补齐，自定义根默认卸载保留通过

用户新人工前台信号按首个本机核时限定17:21:26.3638044–17:51:26.3638044（+08:00）。复核99唯一候选与同SHA完整CI原件后，专用带空格目录NSIS实装exit0、EXE为1ab0d12f…；原平台配置按两份原生目录ID保存，本轮最多一个原生实例。安装目录不可写时真实显示恢复提示，Sky点击仍报坐标几何不可用，立即交用户手工选择本轮selected-root，未猜键盘或重启重试。

原实例56124保留句柄证实exit0，退出FILETIME 134367028005560509；自动替换实例52948创建FILETIME 134367028005653251，两次均同SID普通真实令牌false/type3，观察最大native1。持久启动指针指向selected-root，原生设置页明确当前数据根及内部HOME，选择→持久绑定→自动重启顺序与根回读子集通过。另一专用空目录initialize-empty-root由用户点击“校验并初始化”后显示“结构有效、已创建”，七个冻结分区及版本1元数据生成，活动指针保持。此前路径误填外部HOME后被runtime_unverified拒绝，当前根和HOME未变；该次操作不算空目录初始化失败，也未绕过面板保护。

替换实例退出观察器在前台截止后60秒仍未观测退出，抛出Normal close not observed、宿主exit1。用户随后回复“已退出”，17:53:18实际读到native0；已释放句柄，退出时刻和退出码均未知，不倒填exit0，也不把完整恢复门禁写为通过。恢复原生根回读与已成功的初始化不重做；该替换代生命周期缺口及后续普通重启回读保留待补。前台过期后没有再启动产品。

17:54后台恢复本轮安装目录原ACL后执行NSIS默认静默卸载：父进程exit0，最终主EXE和卸载注册项不存在；selected-root与独立初始化根16份真实文件的大小/摘要及选定指针保持。随后保存新平台配置，恢复原两份物理配置目录及四逻辑路径、原HKCU指针，原偏好摘要07fcccd2…、指针摘要f62b7543…和目录ID/创建时间均一致，最终native0。两个测试根、新配置及私有原件均保留，未删除真实数据。本轮没有可准入1dcc安装包，明确清理项继续阻塞。

证据见`.adg/evidence/OCX-0110-20261017-WIN-99-ROOTS/`，含退出观察超时的明确限制；私有SDDL、原配置内容和数据载荷不提交。TASK revision2/in_progress/pending及八检查保持，已接受CPU/RSS、DPI取消与其它固定缺口、Mac/外部依赖均沿原记录，不合并、关闭、签名、晋升或发布。

### 2026-10-17 后台离线对照查询失败，保留夹具负载调整待裁决

按用户“继续执行”只做后台排查，未启动产品。复核1dcc源码与原日志：同SHA完整CI中的版本success/nonzero分别约0.663/0.695秒，唯一失败候选在约2.011/2.004秒返回Timeout；独立控制Node仍活但其脚本标记较晚出现。未取得生产缺陷或线程/调度/I/O因果证据，不据此猜改查询逻辑，也未重跑同SHA、bd2或增设ETW前置。

已形成未应用的有界方案：无关控制进程改为System32/cmd.exe在自有stdin等待，实际查询和子Node仍冷启动，原16个assert宏、期限及Rust测试并行度保留。本机独立机制检查确认cmd在250毫秒仍活、stdin关闭后exit1，无强杀或产品启动；不等于Rust回归或查询修复通过。但该方案减少同时冷启动的Node数量，改变既有负载，须用户裁决后才可实施，不能自行解释为满足“原并发不降低”。补丁仅在忽略运行目录，`git apply --check`通过、未Rust编译；实现分支仍1dcc且tracked clean，无新CI/候选派发。

离线对照与方案摘要见`.adg/evidence/OCX-0110-20261017-WIN-QUERY-CONTROL-REVIEW/`。原配置、指针、TASK及native0保持；打包前两项失败、明确清理验收阻塞、原固定缺口与禁止发布保持。下一步只在方案获裁决后按新SHA首轮完整CI和唯一候选验证，旧失败继续保留，不将一次新成功推定为历史根因已解决。

### 2026-10-17 用户批准无关控制进程调整，新SHA首轮完整CI执行中

用户对上述具体方案回复“允许”，批准无关控制Node改为显式System32/cmd.exe等待自有stdin，以及由此减少同时冷启动Node数量的负载变化；这是该处测试夹具的人工裁决，不放宽真实查询/子Node的冷启动两秒预算、Rust测试并行度、原观察期限或断言，也不解除历史失败。已将审阅补丁应用于feature/0.1.10-windows-uninstall-data，只有缩短trace签名/调用的格式整理；16处assert宏及完整内容、5秒/5毫秒/8秒/4秒期限均逐项相同，模块仍仅在Windows test编译。

实现提交ee0111e423867e1c3b304d4fe9fd7d4380384df0。普通Git传输多次TLS拒绝后，使用GitHub Git数据库API上传同一blob/tree/commit，逐层核验与本地SHA一致，再以force=false更新原分支并读回；没有改写提交、改TLS证书策略、改系统代理或重复触发。首轮完整push CI39029801652已发现，attempt1/实际源码绑定，归档阶段执行中，尚不准入候选。本机无Rust工具链，静态同一性检查不代替CI的格式、Clippy、双平台回归及最终EXE验证。

同轮终态补充：ee首次CI整轮failure；Mac35组935passed/6ignored，前端100文件542项与另外两必需作业成功，Windows库555passed/9failed/4ignored即停止，最终EXE构建/冒烟跳过。七项查询在2.014–2.135秒得到Timeout，五秒内父/子/持有标记均未出现；无关cmd的三次同句柄快照仍活。另两项IPC在放行后的两秒终态等待Elapsed。该调整没有令本轮查询通过；未证明具体调度/I/O或生产逻辑根因，不重跑ee、不派发候选，不降低原期限或断言。

只读取得该首次失败的诊断artifact12099538424，8913482字节、摘要243032e4…，12成员摘要/CRC通过，无安装包、无载荷执行或新ETW。另直接修订本轮两项IPC测试的热轮询：复用既有测试专用wait_until_idle_for_test，保留两秒截止并在通知后明确断言freeze可得，取消/终态/缓存/审计断言均保留；仅两处测试函数体变化，生产执行与查询夹具不变。实现644fcf35f3f18bbbb46da1e6fed1995c8a572b8c已以同一SHA上传读回，首轮完整CI39030636608执行中。该修正不宣称是七项查询失败的原因或解决；待同源完整结果再决定候选准入。

644首次完整CI39030636608终态success，五份必需job原日志actual checkout相同，Mac35组935passed/6ignored、Windows35组664passed/7ignored、前端100文件542项及最终EXE首屏通过；上述七项查询及两项IPC原断言均通过。该次新成功不证明ee或bd2历史失败根因，不覆盖原失败，也没有重跑同SHA。源码复核另发现明确清理会拒绝真实备份锁及审计根文件这一确定缺口，故644没有派发候选；该清理修复另按新源码办理。两轮原件、惰性诊断ZIP绑定与授权边界见`.adg/evidence/OCX-0110-20261017-WIN-QUERY-CONTROL-APPROVED/`。本机未启动/卸载/清理产品，原配置与TASK保持、native0；固定待测项及发布限制不变。

### 2026-10-17 修复明确清理拒绝真实自有根文件的缺口

生产备份固定在根下创建`.backup-w2.lock`，CLI审计固定写根下`audit.log`及最多五份轮转；原清理仅允许七分区，因此真实执行过备份/CLI的合法根会被当成未知内容拒绝。这是源码确定缺口，未在本机执行删除求复现。8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b已推送读回：复用备份锁名和审计文件名/轮转上限，只接受精确自有名称的普通文件；分区需为目录，未知、别名、链接、超限/非标准轮转继续拒绝。删除前在全部实例锁外再非阻塞取得所有备份锁，重验绑定后才删除，活动备份不被强停。

新增三项跨平台回归，覆盖实际保护备份后的合法根清理、真实备份锁占用时保留/释放后清理，以及保留名类型和未知轮转保护；runner实装夹具增加W2锁及审计种子，未确认拒绝核对全部七文件，外部HOME原哨兵位置保持。只有上述三处实现/测试文件变化，查询生产逻辑/批准夹具、IPC等待和工作流均与644相同；无加时、预热、减并发或原库回放。本机仅PowerShell解析与纯路径表达式检查，无Rust编译、产品启动或清理执行。

首轮同SHA完整CI39032153138执行中，Mac35组938passed/6ignored及新增三项已核原日志成功，Windows回归尚待整轮终态，候选未派发。原件与边界保存在`.adg/evidence/OCX-0110-20261017-WIN-UNINSTALL-OWNED-FILES/`；源码修复不等于安装器钩子或本机明确清理已通过。固定八检查、TASK pending、外部依赖、已接受性能及DPI取消保持，不合并、关闭、签名、晋升或发布。

同源终态补充：8e首次完整CI39032153138整轮success，五份必需job原日志actual checkout一致；Mac35组938passed/6ignored、Windows35组667passed/7ignored，三项新增清理回归在两平台均实际通过，前端100文件542项、格式/Clippy与Windows最终EXE首屏通过。没有重跑8e或改动原查询条件；历史失败继续保留、不证明根因。完整准入后按既有一次性派发方法请求唯一候选，具体安装器实装/制品状态另存候选证据，尚不将Rust通过写为安装器或本机清理通过。

### 2026-10-17 8e唯一候选核验完成，安装器与实装清理助手通过

完整CI后只派发一次39033532953/attempt1，package117398708696实际checkout8e，整轮success；原日志35组667passed/0failed/7ignored。NSIS新钩子与MSI构建、两种包实装首屏/来源及卸载均exit0；实装ocxd助手缺确认exit2并保留七份种子和指针，确认后exit0、锚点及活动根删除/指针解除、外部HOME保持，应用文件仍在；随后默认卸载保留新种子。此为一次性runner种子，未冒称本机UI根绑定或卸载复选框通过，MSI也未新增原生清理复选框。

安装器ZIP12101766414/7729981字节/26a206fb…，runner证据ZIP12101038569/1653736字节/67466dd7…，API来源、成员摘要及CRC一致。沿原只读MSI方法提取0.1.10主EXE8832000字节、SHA870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a，匹配runner MSI载荷；8149749偏移仅内存转换三字节后缀，精确匹配NSIS载荷b948fb7e…及构建载荷f4ee7588…，未写原EXE。本机未运行安装器、应用或清理助手，原偏好/指针/TASK保持、native0。

证据见`.adg/evidence/OCX-0110-20261017-WIN-8E-CANDIDATE/`。下次使用该精确NSIS包d7016d62…/3309326字节，补现有自定义根普通重启回读、窗口内正常退出及真实卸载勾选清理专用数据；99的空根初始化/人工恢复选择及默认保留沿原证据复用，DPI与已接受性能不重做。原17:51:26前台截止不延长，新原生操作仍待新人工信号；其它原清单、外部依赖、旧失败及TASK pending保持，不自动合并、关闭、签名、晋升或发布。

### 2026-10-17 8e本机普通重启与真实卸载明确清理完成

用户“同意继续”开启21:30:54.5994657–22:00:54.5994657新窗口。本轮复用已准入8e唯一候选，专用带空格目录NSIS实际安装exit0，主EXE为b948fb7e6601aa606551e72bb3afc15f0cf8cbc02b7c46584e69584f6565c1a8。原平台目录仍按两份真实目录ID保存；native0时临时回装99现场真实选定的测试指针，作为环境准备，不冒称新UI切根。8e原生设置回读同一selected-root及内部HOME，关闭保持后台选项，生成真实备份bk_20261017133409_c058f8a4。

首次22868于21:34:31实际Alt+F4正常exit0，保留句柄核验；普通重启22020创建时间晚于前代内核退出，原生设置与备份列表回读同根/HOME和备份ID，元数据、偏好、manifest及payload四文件字节摘要保持，启动指针未改。21:37前第二次Alt+F4也正常exit0，两次均同SID普通真实令牌false/type3、无沙箱/CDP，均在本轮窗口内。该证据补本轮普通重启与正常退出子集，不回填99旧替换代未知退出码。

21:37将实际测试根18文件复制至本轮私有before-clear-data-copy并核对所有大小/摘要，然后打开实际安装目录的NSIS卸载器。真实界面包含Delete the application data；工具点击未勾选，立即交用户手工勾选并Uninstall，用户回复“已卸载”。21:46:30读到该根、StartupDataRootV1、主EXE及卸载注册项全部不存在，产品/卸载器进程均0；无关初始化根元数据、根外哨兵摘要不变。没有由脚本直接调用清理助手或删除命令替代卸载。独立勾选后的截图/AX及卸载器退出码未取得，分别记未证/未知，不倒填exit0；实际清理行为有本机读回。

21:47:39已保存新平台配置并精确恢复原两份物理目录、四逻辑路径、偏好07fcccd2…、原启动指针f62b7543…及目录ID/创建时间，TASK摘要bc7965b4…保持，最终native0、卸载器0，前台提前归还。原真实数据未删，18文件测试副本保留；原测试根因本轮明确清理已不存在，旧99证据保留其当时事实，不要求可变测试目录在后续清理后仍原位存在。副本只证明8e关闭后的状态，不冒充99早期字节副本。

证据见`.adg/evidence/OCX-0110-20261017-WIN-8E-NATIVE-CLEAR/`。普通重启/正常退出及本机明确清理子项已补，不重复99初始化/恢复选择、默认保留、已接受CPU/RSS或DPI。其它原检查、受保护位置/产品双令牌剩余范围、Mac/周期外部依赖及TASK in_progress/pending保持；未合并、关闭、签名、晋升或发布。

### 2026-10-17 固定清单逐项核销，后续仅补实际缺项

以8e69ff6d候选为当前实现，复验6份旧方法（含582文件备份原生档、222文件493完整归档、托盘/管理员及人工窗口、45文件原截止跨启动、23文件双令牌档），另验6份清单所列220文件摘要，绑定18份原结果。相对原验证源，备份树表UI、备份安全/策略/浏览、更新与通知、窗口/托盘实现及前端调度路径对象一致；原生周期/退避常量与plan/reserve/complete等片段也相同。备份manager仅新增卸载非阻塞锁入口并复用原锁名，同源8e CI已覆盖；启动准入和画质等有改动范围不据此扩大通过。具体核销已直接更新上方固定八行，不另建验收标准或改TASK。

损坏恢复拒绝、不误解除、精确恢复解除、轮换/固定/重启、树表已观察键盘与刷新、打开警告/取消、普通/管理员真实产品令牌、普通托盘重开、用户已确认管理员最大化/最小化均不再泛列全缺失。轮换年龄为隔离created_at夹具，不算真实30天。10月13日晚已沿原24h截止取得跨启动stable/面板持久成功，不能要求重计该已覆盖到期行为；自动触发、请求/子进程原始计数和休眠等未证边界仍保留。

22:45只读Release API仍stable v0.1.9、beta404；本次两条生产manifest直读TLS退出35/HTTP0，不能写作已验证manifest或当前404，上次成功观察保持10月16日事实。真实更高管理器候选/beta6h来源、Mac和未授权FZ50单列，不改源、发布或盲重跑CI。原周期running文件不作当前活句柄证据。493旧209文件脚本直接执行因追加档案集合拒绝，改用目录既有完整归档方法复验222文件和原子集通过，原清单/脚本/失败不改写。

证据见`.adg/evidence/OCX-0110-20261017-WIN-FIXED-CHECK-RECONCILIATION/`。本轮未启动产品或使用前台，原偏好/指针/TASK摘要保持；存储完成、CPU/RSS接受、DPI取消及最多一个实例保持。下一步只补固定表中确切本机缺项，前台需新人工授权且每窗最多30分钟；Windows全门禁未通过，TASK revision2/in_progress/pending不变，不合并、关闭、签名、晋升或发布。

### 2026-10-17 原型共享树表后台补测与8px动作对齐修复

生产树表既有8组后台几何已核，组件未改，不重跑。直接在活动总原型`docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html`补原型组件缺测，使用已有Node24.19/playwright-core/Edge154无头浏览器与loopback静态服务，不启动Windows产品。最初24组同步文件/WebDAV范围验证选择与展开分离、半选、Space/Enter及重绘保焦；该轮只改浏览器视口，不算真实窄窗，后续另用实际1180×760/900×600原型窗口取证。

补备份树、导入比较和模型策略比较时，实际发现备份树7个动作按钮比名称首行中心低8px。首版方法还错误测量隐藏行/隐藏占位，导入和模型比较的假阳性原件保留；只测实际布局矩形后，原样式48组中只有12组备份树失败，每组7个动作均8px偏差。仅改原型共享tree-table.css两行，按按钮自身高度对齐首行，不缩点击区域或逐页微调。修复后48组/498行/1044可见控件最大偏差0px，84个备份动作按钮仍36px高；另补依赖模版禁用/不可聚焦与模版保存比较取消保留版本4组通过。17项现有注释/维护交互回归通过，已目检代表性浅深/宽窄截图。

原件、各阶段源文件摘要、实际窗口矩形、截图、方法修正及离线复验入口保存在`.adg/evidence/OCX-0110-20261017-WIN-PROTOTYPE-TREE/`。五次浏览器运行顺序完成，子进程exit0、loopback服务关闭、外部请求0，原型源在各次运行内保持；失败方法/旧样式失败不覆盖。仅抵扣原型共享组件UI子集，不验收0.2.0业务或替代Windows原生WebView/安装/性能。应用源码仍8e69ff6d，无CI重派、真实数据操作或前台；原生剩余清单、Mac/可信来源/FZ50边界及TASK revision2/in_progress/pending保持，不合并、关闭或发布。

### 2026-10-18 8e受保护安装与人工Snap/退出完成，前台收尾超时如实保留

用户10月17日“前台授权。”后首个本机核时23:40:15，窗口止于10月18日00:10:15，计划00:05:15收尾。精确8e唯一NSIS在真实管理员true/type2执行器中安装到专用Program Files目录，安装exit0及实装EXE b948fb7e…核同；普通false/type3产品PID552/创建23:43:33真实生产首屏与设置可用，独立数据副本/原指针私有保存，无sandbox/CDP。普通写安装目录探针实际拒绝；当代活动云母/DWM2和无应用菜单行核验，备份树End到preferences叶焦点仅作视觉子项，不替代异步刷新保焦。

工具拒绝跨窗口边界拖拽后交用户手动，用户“已贴靠”；00:07:52同PID/HWND读左边界-6、右1605、顶0、底1150，客户区1598×1114、未最大化，结合工作区和截图元数据，接受左半屏Snap。此时已接近截止，不启动管理员产品；00:10:15后无代理界面输入。用户按托盘退出指引回复“已经退出”，独立保留句柄读同代内核00:19:04.8800733退出/exit0，无强杀；退出方式按人工反馈接受，菜单点击未独立录制。计划第25分钟和30分钟内正常收尾均未达到，作为本轮执行缺口保留，不倒填窗口内完成或续授权。

后台仅执行已授权的默认卸载与原配置还原。00:19:07父卸载器exit0后立即检查仍见EXE，失败停止，原件保持；00:20:06后续只读确认同次卸载的整个测试安装目录、两注册表卸载项、产品/卸载器进程均无，未重跑卸载。00:20:35另存最终读回，实际子卸载器退出码未知；四份元数据/偏好/真实备份清单及payload大小SHA保持。00:20:37保存新测试WebView目录并恢复原两物理目录/四逻辑路径、偏好07fcccd2…、指针f62b7543…及TASK bc7965b4…；独立再验文件ID/创建时刻一致、native0/卸载器0/观察器已结束，测试根保留，原真实数据未删。

证据`.adg/evidence/OCX-0110-20261018-WIN-8E-PROTECTED-SNAP/`保留原AX/截图元数据、读回与失败，截图像素只在对话显示未保存，不把13×13辅助帧当完整窗口。管理员安装器SDDL仅发布标注派生的脱敏件及私有原摘要；私有指针未发布，方法仅作路径/SHA引用不复制生产源码。固定表已抵扣受保护NSIS普通首屏、Snap及人工退出子项，不重复这些场景；管理员原生、备份/事件/更新/调度周期实际剩余缺项保持，未改产品源码或重派CI。存储完成、CPU/RSS接受、DPI取消、Mac/可信来源/FZ50外部边界及TASK revision2/in_progress/pending保持，不合并、关闭、签名、晋升或发布。

### 2026-10-18 剩余启动及请求计数入口核对

后台复验旧启动准备5份清单文件，原冻结定义为新WebView磁盘配置的cache-cold、复用刚生成配置正常退出后的process-warm，两版本/两模式各20，共80样本，nearest-rank p95同时满足增量100ms和5%；没有要求重启Windows或清系统缓存，不重做已接受CPU/RSS。旧measure.ps1仍绑定8be/旧根，无新人工窗口/30分钟截止检查，偏好写入发生在测量try前，无配置还原finally，并将新结果写回封存目录；嵌入C# finally仅清线程连接。本轮未执行该脚本，下一入口须先完成当前8e/官方019绑定、私有新运行目录、配置保存/还原、窗口收尾与受支持的真实UI输入接线，不能只替换SHA即启动采集。原方法/阈值不改，未测不记通过。

既有闲置采样已声明按秒发现后代、可漏短命进程，未保存原始网络请求/字节，因此不能抵100路由/10重启的零查询门槛；需要实际计数入口先于该场景执行。具体引用与问题见`.adg/evidence/OCX-0110-20261018-WIN-MEASUREMENT-METHOD-AUDIT/record.json`。本轮无产品/前台/CI启动，原配置摘要保持，未新增门禁或长期等待，固定剩余范围和TASK in_progress/pending保持。

### 2026-10-18 启动对照授权与偏好还原辅助代码完成，真实计时驱动仍未接线

代码`feature/0.1.10-windows-startup-pair@26840e1b`已推送并API读回。仅新增启动对照状态辅助函数/测试及CI方法步骤，apps相对8e无变化，候选沿原8e唯一制品复用，不另构建。精确官方019与8e/MSI载荷SHA/版本/准入身份本机只读核验；固定原80样本交替排程、人工窗口最多30分钟与第25分钟收尾余量，未改原就绪端点或p95预算。两明确根偏好先完整保留再写夹具，正常/调用方失败可还原；第二夹具只读写失败回滚第一份，未变化只读原件不重复写；并发改动在还原写入前拒绝覆盖。临时根内8项方法检查通过，管道字节stdout/stderr及原配置摘要另存；合成窗口不授予真实前台，本轮无产品或真实偏好更改。

首次push CI39050729429/attempt1已受理，方法Windows作业、frontend、release-tools的job快照success，backend/backend-windows仍运行；未重跑或称整轮通过。证据`.adg/evidence/OCX-0110-20261018-WIN-STARTUP-STATE/`仅引用代码SHA/文件对象，不复制实现源码至文档分支。辅助函数不会启动产品/移动Profile/操纵注册表，不等于完整采集器；目录身份隔离与真实原生就绪连续计时、原始请求/短命子进程计数仍须接入，跨工具等待不能冒称产品启动耗时，也不能以旧直接UIA驱动旁路替代当前执行约束。后续先补这个实际方法缺口再请求性能前台，未测不通过，固定清单/TASK in_progress/pending和禁止发布边界保持。

### 2026-10-18 原生连续计时驱动形成可审阅草稿，执行路径待人工明确

代码私有test/runtime中已准备原生驱动、还原入口与计划模板；核心Process.Start→真实设置按钮调用→设置路由观察片段与原冻结脚本逐字相同，80样本/原p95预算不变。外围绑定当前019/8e、私有新目录、人工窗口/收尾余量及状态辅助代码，失败尝试正常关闭、不强杀，退出后还原原目录/指针。两脚本语法核对和最终计划两项实际无授权拒绝通过，未加载原生UIA输入或启动产品、未创建测量根，样本0。计划仍要求新窗口时实际保存/克隆目录身份与偏好绑定，草稿不作为可直接启用的完整采集器。

当前computer-use Skill操作约束只允许JS APIs界面自动化，旧连续驱动采用PowerShell UIA；分步JS模型等待会改变原端点计时，不能静默放宽口径。需要用户明确是否允许一次独立原生UIA采样路径，并重新授权最多30分钟前台；这是执行路径例外判断，不是Skill规定新增验收门禁。未决定前不执行，原00:10:15截止保持。可审阅绝对路径/SHA、原片段比较、拒绝原件与最新同run CI快照见`.adg/evidence/OCX-0110-20261018-WIN-STARTUP-DRIVER-PREP/`；实现草稿留代码worktree，不复制源码到docs。该CI后台Mac作业现success、Windows仍运行，不重派；原始请求/短命子进程计数缺口独立保留，TASK/原预算/外部依赖与禁止发布边界不变。

### 2026-10-18 原生UIA冷/热启动配对完成，窗口内提前还原

用户明确批准“原生UIA采样驱动＋新30分钟前台”，首个本机核时01:15:33.9752287，正常收尾目标01:40:33、绝对截止01:45:33（均+08:00）。使用单独PowerShell UIA连续驱动，未混入JS界面操作；只读复验精确官方019/de6862d4与已准入8e唯一候选MSI载荷身份，不另派CI或重构候选。原两份物理配置目录按原生文件ID保存，两份数据夹具来自先前任务数据副本；原指针私有保存，不发布原配置内容、指针原文或SID。

01:20:48–01:23:21完成20轮交替版本次序、每版先cache-cold后process-warm，共80份真实样本。端点仍为Process.Start→可用设置按钮→UIA Invoke→设置路由文本出现，原30秒上限不变；冷为新应用WebView磁盘配置，热为正常退出后复用刚生成配置，不清系统缓存、不称Windows重启冷启动。80次均保留进程句柄验证同SID普通false/type3、实际设置交互、对应WebView路径和正常exit0，轮间native0保护，无沙箱/CDP或强杀。

原分析器逐字节复用，nearest-rank p95：冷019 1354.5511ms、8e 1273.1190ms，变化-81.4321ms/-6.0117%；热019 1111.7454ms、8e 1103.4023ms，变化-8.3431ms/-0.7505%。两模式各20均满足原增量≤100ms且≤5%预算，启动子项通过。初轮019冷/热最大3756.6696/3696.8587ms保留在原样本，不删异常值或整套重跑；已接受CPU/RSS未重测。

采样完成后的偏好回滚因019重写JSON字节格式而触发并发变化保护，驱动宿主exit1、cleanup-result配置未还原原件均保留，不写为驱动无失败。候选偏好仍为精确seed字节，两版完整JSON语义逐项相等。native0后另行调用已绑定还原包装器，完整保存新夹具目录及其变化字节，再恢复原两物理目录、四逻辑路径和原HKCU指针；01:24:14.4933457完成，独立复核文件ID/创建时间、原偏好07fcccd2…、指针f62b7543…、TASK bc7965b4…及native0均一致。未覆盖变化的夹具偏好，原真实数据未删，前台提前归还。

证据`.adg/evidence/OCX-0110-20261018-WIN-STARTUP-PAIR/`含80份样本、原分析结果、清理失败与最终还原原件及方法路径/SHA引用；实现方法不复制到文档分支。26840e1b首次完整CI39050729429/attempt1已success，同SHA API/jobs终态一并保存，apps相对8e未变。固定表抵扣启动配对，帧/50ms、零自动查询与原始计数、休眠/周期及其它原清单实际缺项继续保留；不把启动通过写为完整性能/Windows门禁通过。TASK revision2/in_progress/pending、DPI取消、Mac/可信源/FZ50边界保持，不合并、关闭、签名、晋升或发布。

### 2026-10-18 后台补查询、短命进程和帧观察入口

实现`feature/0.1.10-windows-query-observer@0852b7ee6f3b58fb5439bf66cbea97c4b7982a3a`已推送并API核同。9个变更仅为test及既有CI方法接线，apps相对已准入8e无变化，继续复用精确8e/官方019制品。原生只读绑定入口记录保留产品句柄的真实普通令牌、EXE、单实例，以及回环调试监听器到该产品WebView的实际祖先链；CDP不自行冒充原生身份。

观察器采用原生postMessage/runCallback始终false条件，记录三个原调度/查询命令和异步完成，不替换只读桥或响应。RAF/longtask记录原间隔与主线程长任务，不替换时钟、RAF或计时器；增量排出页面记录，在外部汇总，保留跨批次在途关联。ETW入口只启用Process/NetworkTCPIP，按原QPC/世代保留全部测试后代及TCP/UDP IPv4/IPv6传输大小字段，避免每秒快照漏短命进程；双方回环且端口精确等于本轮调试端口的通信另标，其它回环代理不排除。传输事件不是逻辑HTTP请求数，RAF间隔不是回调耗时，不能用某一侧零记录替代完整计数或附着前区间。

本机13项最终协议测试全通过，覆盖实际V8调用、跨增量批次完成、溢出/过期授权拒绝，以及真实无头Chromium浏览器任务超过50ms的可观测性。原生代码按原固定TraceEvent3.1.21编译，24项新规则、38项既有世代和8项既有停止规则通过；普通令牌在创建会话/目录前明确拒绝。夹具初轮脚本尚未就绪、后续DevTools evaluate未产生真实浏览器longtask的失败及修正过程均保留；只改方法夹具，不在产品注入负载或重跑产品求通过。

首次同SHA CI39062488018/attempt1执行中，API原件及job观察已保存，不重派、不称整轮成功，也没有管理员内核真实事件正向校准或产品正式计数/帧样本。默认入口只准备，录制/产品仍受既有授权与新人工窗口约束；原第25分钟收尾、第30分钟截止不变。下步先补真实内核正向校准，再在新窗口按原场景执行，不重测已通过启动配对、已接受CPU/RSS或DPI。

证据`.adg/evidence/OCX-0110-20261018-WIN-QUERY-OBSERVER-PREP/`仅存原日志、绑定和方法路径/blob/SHA，不复制源码到docs。整个后台轮未启动产品/内核会话或占前台；原目录ID/创建时刻、偏好07fcccd2…、指针f62b7543…、TASK bc7965b4…及native0保持。固定八检查未核销未测项，TASK revision2/in_progress/pending、Mac/可信源/FZ50外部边界保持，不合并、关闭、签名、晋升或发布。

### 2026-10-18 复用真实缓存并准备启用自动检查的场景

0852首次完整CI39062488018/attempt1后续整轮success。另提交0d37866f20ddfe6ca27fa2bd95036182f6dcff42，仅新增只读缓存准备检查、纯内存偏好生成及其Windows方法接线；首次CI39063634995/attempt1也整轮success，均无同SHA重派，apps相对8e不变。

先前任务数据副本的stable/面板缓存及持久成功仍有效：原2026-10-17 17:27:31/33成功，原截止2026-10-18 17:27:31/33（+08:00）。新私有夹具只开启自动检查并设置正常关闭退出及既有显示条件，原三份缓存/调度文件逐字节保持，不改成功锚点、响应或截止；原任务副本和平台配置不动。11项合成方法检查验证这些准备与拒绝分支，明确不授予来源真实性、前台或产品通过。

启动配对的auto-check=false不能直接充当缓存自动检查场景；原10重启需实际覆盖原45秒评估成功，不能以45秒前快速关闭取得零计数。具体副本/原件和方法见`.adg/evidence/OCX-0110-20261018-WIN-CACHE-FIXTURE-PREP/`，后续真实窗口仍须重读当前有效期；过期则另取得真实成功，不回填时间或重新等待已覆盖24h。

### 2026-10-18 原生100/10控制与后台校准入口备妥，实际场景待新窗口

869692ce00298cede276ac05dc0a69a7ea7dac29已推送核同，包含原生单实例控制、只读代际观察、管理员录制校准入口及方法检查；首次CI39068216529/attempt1执行中。没有产品实现改动或新候选派发，精确8e/官方019沿原准入使用。完整入口保留在代码worktree的`test/runtime/cache-query-state-prep-20261018/execute-next-cache-window.ps1`，默认不运行，须新人工窗口及既有原生UIA路径许可。

每代原生UIA动作通过实际路由回读确认，原45秒启动评估真实返回后才正常退出；等待原生/WebView结束才下一代。默认十代各十个变化，支持根据原件审计仅补缺失序号，最终总体10/100与原始计数另审，不自动缩减或重放。实际根/内部HOME、普通保留句柄/令牌/SID、调试监听器到该WebView祖先链分别核对；只用普通SDK调试参数，保持应用sandbox-smoke变量未设置。配置保存/还原沿原两物理目录/四逻辑路径方法，源码和路径摘要均保留。

最终本机9项Node方法检查通过，含真实V8/Chromium、未完成启动评估/重复路由拒绝及原子标记并发不覆盖；原生代码编译与30项新/38项世代/8项停止规则通过。普通诊断负载实际20个短命子进程和4次回环请求/1024字节全部正常exit0，不访问外网；管理员默认/过期入口无录制。静态复核修正新控制器WTS信息类14为连接状态8，并只读核验，未用错误声明启动产品。未分类及本地版本探测保留分类，不自动豁免查询子进程；TCP/UDP大小字段不冒充HTTP请求数。

真实管理员内核正向校准、实际产品10/100与最终原始计数未执行，不能以方法通过抵扣；帧原件可随场景保存，原帧/019对照及其它固定缺项不改变。本轮无产品、内核会话、平台目录移动或前台；偏好/指针/原目录身份与TASK保持，证据`.adg/evidence/OCX-0110-20261018-WIN-CACHE-NATIVE-CONTROL/`含前置两CI终态、当前首次CI、方法输出及可审阅入口引用。窗口仍按本对话第25分钟正常收尾/30分钟截止，后台管理员ETW旧授权不重复索取。TASK revision2/in_progress/pending、DPI取消、已接受CPU/RSS、已通过启动配对与Mac/可信源/FZ50边界保持，不合并、关闭、签名、晋升或发布。

### 2026-10-18 当前SHA的hosted辅助制品核验

同869首次CI39068216529/attempt1的辅助作业已success，只读取得artifact12114620417/18436字节/SHA40571cb9…，API绑定、摘要、ZIP CRC及34成员核对一致。原JSON证实默认/过期授权保护有效、合成偏好auto-check开启；hosted上下文实际为管理员，因此普通机制明确跳过、退出字段null，内核校准仍false，不将其改称本机普通/实际录制通过。惰性ZIP及结构化报告保存在`.adg/evidence/OCX-0110-20261018-WIN-869-HOSTED-METHODS/`，未执行下载载荷或发布签名地址/凭据。

当前Windows后端仍在同一次普通回归作业中，整轮未定；没有重新派发、终止或重复运行该SHA。本机native0和原配置/TASK保持，原前台窗口已结束，实际100/10及其它固定缺项仍需新的直接人工前台信号。此补充只核同SHA辅助证据，不核销本机门禁或关闭/发布。

### 2026-10-18 Windows 本轮执行收尾

用户在重新评估反复采样的必要性后明确表示：“同意，现在我们得收尾了，不能无限这样下去。”本轮按此结束本机执行：停止新增采样、采集工具开发、校准和前台申请，不继续自动等待窗口或推进旧补采计划。最近三次实现提交 0852b7ee、0d37866f、869692ce 只增加测试及 CI 方法；代码 HEAD 与已准入 8e 的 `apps` Git tree 均为 `80a4d3d29d26899e14e1324b69c8e2866f1dc2d9`，没有需要因此重测产品的实现变化。此前把工具完善持续作为产品收尾前置，造成执行范围扩大；后续不得据此恢复同类循环。

本轮保留的结论：

- 019/8e 同机冷/热启动共 80 样本通过；CPU/RSS 实测已获用户接受，均不重测。专项 DPI 已取消，不改写成测试通过。
- 存储、初始化/选择/根绑定、普通重启、卸载默认保留和真实勾选清理等已核销子项，以及备份恢复/损坏拒绝/轮换固定、树表修复、真实令牌、已观察原生操作，按上方固定表复用，不重放求取更完整录像。
- 已有真实 30 分钟错误退避及 10 月 13 日原 24h 截止后的跨启动成功证据保留，不重新等待已覆盖周期。缓存 100 次切页/10 次重启零自动查询、帧/50ms、休眠和确实未证的自动触发仍未验，工具方法通过不能抵扣。
- 更新真实下载/验证/应用与重启版本回读、对应事件隔离，以及备份和管理员原生剩余组合，仍以上方固定八行为唯一详细清单，未增加验收项。可信更高版本/beta 来源、Mac 和历史 bd2 失败现场等依赖单列，不通过发布或反复回放制造条件。

08:10:45（+08:00）只读 GitHub Actions API 确认 `869692ce00298cede276ac05dc0a69a7ea7dac29` 首次 CI `39068216529/attempt1` 已 completed/success，五个必需作业全部 success，build 按配置 skipped；该轮于本机时间 05:08:47 结束。纠正上文当时“整轮未定”的观察，不重派 CI 或构建新候选。此前同 SHA 辅助制品已核摘要/CRC，仍不当作本机产品实测。

远端文档分支核至 `cac2282a766f4285a8ed9eb94e7e15987ba80991`，代码分支 `feature/0.1.10-windows-query-observer` 核至 `869692ce00298cede276ac05dc0a69a7ea7dac29`，均与本地对应 HEAD 相同；产品改动仍在已推送特性分支，本轮不自动并入 main。08:10:05 产品实例为 0，08:11:24 已知缓存/校准/计数执行器为 0；本轮没有启动产品、录制 ETW 或更改配置。TASK 文件 SHA256 仍为 `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93`。

收尾回执见 `.adg/evidence/OCX-0110-20261018-WIN-CLOSEOUT/record.json`。本结论只结束本轮 Windows 执行，不宣称 Windows 全门禁或 0.1.10 已可发布；TASK revision2/in_progress/pending、八检查及原失败保留，未关闭、合并、签名、晋升、发布或执行 FZ50。后续可回到 Mac 做已有成果的版本整合；本对话不再因未验清单自动续跑本机测试。

### 2026-10-18 Mac 接手收尾与产品基线整合

按用户最新指令同步docs b2e4ff59，Mac本地集成分支从bd2快进至已验产品8e69ff6d；未提交工作保留。8e与869的apps tree完全相同，后续采集工具不作为收尾前置；173项Windows封存文件bytes/SHA复核一致，已核销子项直接复用。原TASK字节/revision2/in_progress/pending与八检查保持。main仍eae，main×8e只读合并无冲突且tree等于完整8e，未实际合并/推送代码。

下一项为8e的Mac arm64候选构建及隔离安装准备，再集中补Mac正常退出/重启/数据保留和更新/备份树表/通知实际缺项，最后整理八检查、版本说明和集成发布审阅。CPU/RSS按用户接受处理，本轮不追加资源或重复性能采样；DPI取消。旧失败及未验边界保留，不由Windows局部成功外推Mac通过，也不自动扩建工具或恢复补采。详见[Mac收尾与版本整合](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/08-Mac收尾与版本整合.md)。本次未启动原生、重派CI/候选、迁移真实数据、替换日常安装、签名、发布、晋升或关闭。

### 2026-10-18 Mac 8e候选与原生收尾子集完成

更新后的原目标已读回active，终点扩展为发布及发布后验证，最终发布确认保留。Mac源8e未改；搬迁工作树的旧build-script/依赖权限绝对路径导致两次构建失败，原件保持，独立全新target下locked/offline/no-sign构建app/DMG成功。arm64/0.1.10/内嵌8e核同；EXE SHA d6df4df0…、DMG9cd02a39…，容器校验exit0，严格签名exit1按未正式签名候选边界保留。

11:33:33–11:45:45（+08:00）DMG隔离安装副本两代顺序原生运行，两次Command-Q均exit0，最终隔离0，日常57186及选定偏好/根配置摘要保持。真实备份/固定/深色和根回读、备份三文件跨重启字节一致；更新两类详情返回、Tab循环、真实键盘入口Escape归还、精确2.88.0说明及内部滚动、中低档选择/低档实底和通知入口有原生证据。关闭后CUA立即dock:reopen不计完整隐藏区间；默认准备态/密度不代替全矩阵。见[Mac8e候选与原生收尾](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261018-Mac8e候选与原生收尾/README.md)。

本轮无资源/DPI/周期采样、工具扩建、官方安装/FZ50、真实迁移或日常替换；未验真实更新/事件、备份边界/刷新焦点、原生剩余组合与正式签名等继续留档。接下来按原八检查汇总实际缺口和最终集成/发布准备，不重跑已验Windows或旧采样。TASK原字节/in_progress/pending及原失败保持，不合并/发布/晋升或关闭。

### 2026-10-18 汇总草稿PR与集成CI

Mac证据已提交推送docs9af58e6b。以既有远端产品分支8e直接建立[汇总草稿PR #7](https://github.com/gzers/opencodex-desktop/pull/7)，base main@eae、head feature/0.1.10-windows-uninstall-data@8e；已附加本任务，API读回open/draft/merged=false、mergeable=true，托管合成提交48cff7dd7079e6872e804709cd1da6c87ab1e126。没有推新代码、改源或重做Windows制品/现场验收；现有六个草稿保留，不自动依次合并。

PR事件自动启动首次集成CI39095241645，核验时五个必需作业均in_progress，不能以旧8e完整CI成功替代本次结果。后续只跟踪同一次运行并核对实际checkout/终态，不重派求绿；main仍eae，最后发布确认、原八检查未验边界与禁止提前关闭保持。

### 2026-10-18 发布说明、核心事实与执行提案

已在协作包准备[发布执行与核心事实回写提案](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/09-发布执行与核心事实回写提案.md)：集中列出版本/源码/制品/通道、更新签名与OS签名的差别、剩余未验项、损坏保护关联及WindowsCLI实现限制和最终执行顺序。原统一事件核心承诺已有完整审阅投影，绑定8e注册表SHA3dd0e6d0…，覆盖8触发/39任务/81事件/27策略/12路径/4清理；其发出/调度/UI接线数量分别87/8/9，登记不外推为全行为通过。数据/备份提案列真实manifest字段、目录树用途、保护/轮换及未来新增范围维护合同。核心权威尚未改写，最终只在核心维护单一事实，协作包保留过程。

0.1.10用户说明草案和main/CHANGELOG待应用补丁已准备，git apply --check通过但未应用；版本tauri/frontend/Cargo均0.1.10，HEAD/v0.1.9/v0.1.7内置更新公钥的解码SHA均30190687…，旧信任未换。只读Secret列表确认更新私钥条目存在，未读取私钥或执行签名。公开v0.1.9资产集合按原版本存在，v0.1.10按tag查询404；没有创建tag/Release、改变Latest或提前发布。真实资产仍须在最终批准源码构建后验证，不能用公钥一致/Secret存在代替密码学验签。

### 2026-10-18 PR7首次完整集成CI终态成功

39095241645/attempt1 completed/success，五份必需job原日志actual checkout均48cff7dd（head8e/baseeae）；Mac35组938passed/6ignored、Windows35组667passed/7ignored，前端100文件542passed、类型/构建/CDP、发布工具和Windows方法作业通过。Windows最终release EXE构建/首屏成功，main-only build按PR事件正常skip，不当作Release构建完成。未重派或新打包，原失败不抵销。原日志/REST/提案时点复核见[集成CI与发布准备](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/验证/20261018-集成CI与发布准备/README.md)。

当前产品/版本说明/核心事实审阅材料已具备，后续完成真实事实回写与共享说明纠错，并集中明确未验项/已知限制、OS签名策略、源码/候选及所选通道，才进入最后发布确认。main仍eae，原TASK字节及八检查未整体核销；没有合并、tag、正式签名、公开Release、晋升或关闭目标。

本轮已整理[最终发布确认单](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/10-0.1.10最终发布确认单.md)，固定8e产品/双平台候选、CI、说明/核心提案、拟stable投递、OS分发与updater签名差别，以及需人工明确裁决的未验/限制。本单待批准，不将“停止采样”解释为剩余要求自动通过；若接受本版例外，也须独立记录裁决而不改写历史测试。获准后继续到实际发布与发布后验证/任务收口，当前目标保持active。
