# 原型设计 QA

## 检查范围

- P0 主界面布局。
- 状态覆盖。
- 操作反馈。
- 脱敏与安全文案。
- 可读性与可扫读性。
- 原型约束是否成立。
- 与 OpenCodex 官方面板的视觉一致性。

## 已覆盖

- 主概览、安装状态、统一数据根、启停控制、日志、错误摘要、面板入口。
- loading、stopped、starting、pending、running、starting failed、at-risk、external takeover、unreachable 状态。
- 不可逆或高风险动作的确认模型。
- 敏感信息在 mock 日志中显示为掩码。
- P1/P2 使用“分析中”占位，避免误导为已实现。
- 新增“嵌入官方面板”视图与面板不可用状态。
- 新增 macOS 菜单结构示意：进程、视图、日志。
- 颜色体系已向 OpenCodex 官方面板对齐，采用中性 primary 与官方状态色。
- 官方面板布局已按评审定稿为方案 B：保留 64px 桌面壳图标栏和 App 标题栏，官方中文页面快照占满剩余内容区；不再使用全幅沉浸式。
- 2026-09-12 已复核：官方面板模式下左栏约 64px，中文官方面板主内容区约 1116px；概览页维持 232px 完整侧栏。
- 侧栏已替换为 OpenCodex 风格 2px 线性 SVG 图标，统一 18px；面板图标栏放大到 20px。品牌标记改为内联官方风格渐变云标，尺寸 28px，避免外部 Logo 被拉伸。
- 新增官方 GUI 源码级快照预览通道：从官方 GUI 渲染结果复制 DOM 与 CSS；light / dark 截图已验证无 console error。
- 新增 `design-tokens.md`：颜色、字体、间距、圆角、控件尺寸等以官方 `gui/dist` 构建产物为来源，原型 token 与官方基准对齐。
- 新增官方 light / dark 截图与原型 light / dark 截图对照；截图仅用于视觉 QA，不用作样式来源。
- 官方快照与原型 shell 已验证 `ocx-theme` 同步，dark 模式下双方均切换为官方 dark token。
- 原型已改为 `index.html` 根入口，hash 路由串连侧栏页面；界面按 1180 × 760 macOS 应用窗口呈现，弹层与提示约束在窗口内。
- 配置迁移、WebDAV 与升级已收敛为「设置」页的分区；概览保留导出 / 导入、立即同步 / 测试连接、检查更新 / 升级前备份等高频动作。
- 2026-09-12 Playwright 回归：8 条路由激活区块、hash 与标题正确；无 console error / page error / request failure；官方面板 iframe 逻辑尺寸为 1178 × 758，管理器侧栏和标题栏均隐藏。
- 已重做官方与原型的 light / dark QA 截图，截图仅作视觉证据。
- Playwright 已验证 light / dark 下各路由标题、激活区块、窗口尺寸和官方面板状态。

## 风格基线

- 主题：`light / dark / system`，与官方 GUI 保持一致。
- 背景：light `#f9f9f9 / #ffffff`，dark `#212121 / #262626`。
- 边框：light `#e6e6e6`，dark `#3d3d3d`。
- 文本：light `#0d0d0d / #6e6e6e`，dark `#ececec / #a6a6a6`。
- primary：中性近黑 / 近白，不使用品牌色做高风险进程控制主按钮。
- 状态色：沿用官方 green / amber / red / blue 语义。
- 品牌色：`#3941ff` 仅用于品牌识别、Logo、局部路径强调，不承担主按钮颜色。
- 圆角：小控件 8px、卡片 12px、弹层 16px。
- 控件高度：常规 34px，主要控制 40–44px。

## 待确认

| 编号 | 问题 | 建议 |
|---|---|---|
| QA-01 | 主概览是否需要同时展示安装路径和 `OPENCODEX_HOME`，还是拆成二级页 | 当前原型同屏展示；信息密度可接受 |
| QA-02 | 日志默认展示 50 行、200 行还是使用虚拟滚动 | 建议默认 100 行 + “打开完整日志目录” |
| QA-03 | at-risk 与外部 provider 接管是否合并为同一张风险卡 | 建议保持分开，风险来源和恢复方式不同 |
| QA-04 | restore 确认是否需要显示当前配置摘要 | 建议只显示分类摘要，敏感字段掩码 |
| QA-05 | 统一数据根切换后是否需要重启应用或只重启代理 | 属于 IMP 决策，原型先提示“需确认” |
| QA-07 | 官方面板主题跟随系统还是由桌面壳同步 | 生产实现建议跟随官方 GUI 自身主题；桌面壳提供跳转设置入口 |
| QA-08 | 生产版如何承载官方 GUI | 官方明确禁止 iframe 嵌入；原型真实页面是官方 GUI 源码级快照，生产应采用 Tauri 原生 WebView / 子窗口并另行确认承载方案 |
| QA-10 | P0 是否引入 `gui/dist` + mock API 做交互级演示 | 建议 P0 不引入；当前源码级快照足以验证视觉与布局。若需要验证真实嵌入行为，再单独确认 mock API 与版本锁定方案 |
| QA-11 | 官方快照版本升级策略 | 每次升级后重新提取 token、重做源码级快照与 light / dark QA；不把旧版本截图或 DOM 作为长期设计基准 |
| 已关闭 | 概览的基准窗口尺寸 | 已定为 **1180 × 760（默认）/ 900 × 600（最小）**，对齐同类 Tauri 桌面端；模拟窗口由 1440×1040 上限收回到 1180×760。 |
| 已关闭 | 概览的「单屏不滚动」是否是硬要求 | 不是硬约束。窗口 1180×760 下已实测单屏（溢出 0）；更小的窗口允许滚动。 |
| QA-14 | 概览是否保留 4 条完整环境路径 | 已按「一行摘要 + 折叠详情」定稿；完整路径收进运行详情，概览摘要只留数据目录缩写 |
| QA-15 | P1/P2 快捷卡是否继续显示「已连接 / 已启用」等真实态 | 建议按 design-qa 既有约束改回「规划中」占位，或用统一 `P1 规划` 标签，避免被读成已实现 |
| 已关闭 | 「统一数据根」是否统一改称「数据目录」 | 用户可见文案已改为「数据目录」；正式文档保留「统一数据根 / 数据根」作为存储契约术语，界面语境另注「数据目录」 |
| 已关闭 | 页面未串联、根入口错误 | 已改为 `index.html#overview` 根入口，7 条核心路由均可通过 hash 直达并互相连接。 |
| 已关闭 | 进程控制与概览功能重复 | 概览收敛为唯一进程状态与启停入口；旧 `#process` 自动回落概览。 |
| 已关闭 | 概览动作与迁移 / 同步 / 升级入口是否分散 | 概览保留高频动作，参数统一进入「设置」；旧 `#config`、`#webdav`、`#upgrade` 自动回落设置。 |

## 2026-09-12 晚 · 概览控制台复检
- 通过：`#overview` 在 1913×1145 桌面视口单屏承载状态指标、环境路径、运行控制与三类快捷动作。
- 通过：原型说明、按钮说明和状态测试器移到 App 窗口外，并按 `overview / panel / installation / logs / settings` 切换。
- 通过：面板路由继续使用静态中文快照，无需启动官方服务；快照两侧保留桌面壳图标栏与右侧注释栏。
- 通过：`git diff --check` 无空白错误。

## 2026-09-12 · 概览布局审计（运行控制 / 环境路径 / 窗口基准）

### 取证方式

- 用无头 Chromium 加载 `index.html#overview`，实测 viewport `1180×760 / 1280×800 / 1440×900 / 1512×982 / 1728×1117 / 1920×1080`。
- 记录 `.window` 实际尺寸、`.main` 滚动高度、各区块 bounding box、每个按钮的文本 / 类名 / 禁用态 / 坐标。
- 全程无 console error、page error。
- 证据截图：`截图/归档-20260913/概览方案/audit-overview-1180x760.png`、`audit-overview-1920x1080.png`、`audit-page-1180x760.png`。

### 结论摘要

1. 「运行控制」卡片的面积分配与它自己的命名不符：约 2/3 面积是只读展示，真正的控制按钮只有最后一行的 37px。
2. 按钮分组不成立：同类动作被拆到卡片首尾两处，异类动作并排在同一行，主操作不在主位。
3. 概览的 4 条环境路径占掉整页约 20% 高度，且其中 3 条可由统一数据根推导，属于冗余。
4. 「单屏控制台」只在窗口高约 1032–1040px 时成立；按标准 macOS 窗口（1180×760 / 1280×800）必须滚动 200 上下像素才看得完。

### A. 「运行控制」卡片

实测卡片高 441px，构成为：

| 区块 | 高度 | 性质 |
|---|---|---|
| 卡头（标题 + 刷新状态 / 面板 / 日志） | 45px | 混合：状态动作 + 两个导航 |
| 6 格只读指标（状态 / 健康 / 端口 / PID / 版本 / Runtime） | 144px | 只读 |
| 4 格环境路径 | 138px + 14px 间距 | 只读 |
| 卡片底部按钮行（启动 / 停止 / 重启 / Doctor 摘要 / 安装配置 / 设置） | 37px + 15px 间距 | 交互 |

问题：

- **A1 主操作不在主位。** 卡片叫「运行控制」，但启动 / 停止 / 重启被压到卡片最底部一行，且和 3 个非进程按钮并排；用户的视线要先穿过 282px 只读信息才能碰到控制。
- **A2 同类入口被拆成两处，且与侧栏重复。** 「面板」「日志」在卡头右上，「设置」「安装配置」在卡底；而左侧导航栏本来就有 概览 / 面板 / 日志 / 设置。同一类导航在概览页出现了两套，位置还不一致。
- **A3 一行 6 个按钮混合 4 类语义。** 进程控制（启动 / 停止 / 重启）、诊断（Doctor 摘要）、配置跳转（安装配置）、页面跳转（设置）排在同一条水平线上，没有任何分隔或分组，这是「按钮从逻辑上布局有点乱」的直接来源。
- **A4 禁用态没有解释。** 实测初始 `loading` 状态下，启动 / 停止 / 重启 三个按钮全部 `disabled`（`start` 只在 `stopped / not_found / starting_failed` 可用）。三个灰按钮并排却不说明原因，用户会以为是坏掉的控件，而不是「还在探测」。
- **A5 版本信息重复。** 只读指标里的「版本 2.50.0」与「版本升级」卡里的「当前版本 2.50.0」是同一份数据，出现两次。

### B. 环境路径详细程度

- 概览用了 4 个 metric 展示完整绝对路径：`OPENCODEX_HOME`、统一数据根、日志目录、备份目录，合计约 152px，占整页 20%。
- 其中 `日志目录 = 统一数据根/logs`、`备份目录 = 统一数据根/backups`、`OPENCODEX_HOME` 也在统一数据根之下，三条都可由第一条推导，属于冗余；用户在这张卡片上真正要确认的只有一件事——**数据根是不是他预期的那个**。
- 路径用 `white-space:nowrap` + `text-overflow:ellipsis`，窗口一窄就被截断（实测 1180 视口下 `OPENCODEX_HOME` 直接显示为 `—`），既占地方又不可读完整，是最差的组合。
- 建议：概览只保留一行「统一数据根 + 是否默认位置（是 / 已自定义）」，完整路径收进「设置 - 安装配置」；日志目录沿用它已有的「打开日志目录」入口，不必在概览复述。

### C. 窗口基准与「单屏」成立条件

| viewport | 应用窗口实际尺寸 | 概览内容需要高度 | 可显示高度 | 溢出 |
|---|---|---|---|---|
| 1180 × 760 | 1136 × 720 | 877 | 674 | **203px（需滚动）** |
| 1280 × 800 | 1236 × 752 | 877 | 706 | **171px（需滚动）** |
| 1440 × 900 | 1360 × 852 | 865 | 806 | **59px（需滚动）** |
| 1512 × 982 | 1126 × 934 | 888 | 888 | 0 |
| 1728 × 1117 | 1342 × 1040 | 994 | 994 | 0 |
| 1920 × 1080 | 1440 × 1032 | 986 | 986 | 0 |

- `main` 滚动高度随窗口宽度略变，所以「单屏」的成立阈值是窗口高约 1030–1040px，也就是 .window 的 `min(1040px, 100vh - 48px)` 上限。
- 这就是「概览单屏」这个结论此前的隐含前提：它是在 1913×1145 桌面视口下达成的，而这个窗口高度（1040px）明显大于标准 macOS 应用窗口。
- 根因不是间距不够紧，而是概览选择用「更高的窗口」而不是「更省的信息」来换取单屏：只读信息 + 交互信息全部堆在一屏，再靠拉高窗口容纳。
- 另注：`.desktop` 在 ≤1470px 时把 320px 原型注释栏折到窗口下方，窗口因此拿到全宽（1136px），这也是为什么窄视口下窗口没有横向崩掉、但纵向压力更大。

### 建议改法（只调布局，不改交互语义）

1. **按功能分三条带，而不是卡片首尾两处。**
   - 状态带：状态 / 健康 / 端口 / PID 合成一条状态条（1 行，含状态点与主文案）。
   - 控制带：启动 / 停止 / 重启 为主按钮组，靠左；Doctor 摘要 为次级；确认对话框语义不变。
   - 导航带：面板 / 日志 / 安装配置 / 设置 统一收到标题行或卡片右上角的次级位置，不再插入控制带。
2. **禁用按钮必须给理由。** 探测中 / 运行中 时，把三个进程按钮替换为一条状态说明（例如「正在探测本机安装…」/「运行中，可停止或重启」），比三个灰按钮更省空间也更清楚。
3. **路径压到 1 行。** 概览保留「统一数据根 + 是否默认位置」；完整路径进设置。顺带去掉版本重复。
4. **P1/P2 快捷卡去真实态。** 「已启用 / 严格 / 3 天前 / 已连接 / 今天 10:24」会被读成已经实现，与 design-qa 既定约束不一致；改成统一规划态占位。
5. **窗口基准回到标准 macOS 尺寸后重做题图。** 若采纳 QA-12 的 1180 × 760 基准，本页需要重新出 QA 截图，并在 `README.md`、`state-and-flows.md` 同步「单屏」表述。

### 本次未改动

- 未修改 `index.html`；以上仅为审计结论与改法建议，是否落原型待确认（QA-12 ~ QA-15）。
- 同类产品调研见 `同类产品调研-2026-09-12.md`：同类 Tauri 桌面端默认窗口为 1000×650 ~ 1280×800，进程控制普遍放在常驻状态条，按钮按状态渲染而非禁用，卡片有 compact / detailed 密度分级。

## 2026-09-13 · 概览布局方案 B / C / B+ 已落原型

按「先做 B 和 C 两个效果再过滤」的要求，两套方案已实现为可切换的概览变体；随后按评审结论追加合并版 B+。四套共用同一份 mock 状态与同一个状态测试器。

### 访问方式

| 入口 | 说明 |
|---|---|
| `index.html#overview` | 现行 baseline，未改动，保留作对照 |
| `index.html#overview-b` | 方案 B · 三段式重排 |
| `index.html#overview-c` | 方案 C · 摘要 + 详情抽屉 |
| `index.html#overview-b2` | 方案 B+ · 合并版（推荐） |
| 原型说明区「概览布局方案」 | 四态切换按钮，位于 App 窗口外，不属于软件界面 |

切换只改变概览排布；`#panel`、`#logs`、`#settings` 与 `#process / #config / #webdav / #upgrade` 旧回落行为均未受影响（已回归验证）。

### 两套方案都应用的公共优化项

- 进程按钮改为**状态驱动可见**：未运行只出现「启动」，运行中只出现「停止 / 重启 / Doctor」，不再出现无解释的禁用态。
- 状态区配一句**状态说明**（10 个状态各有文案），替代灰按钮传达原因。
- 快捷动作卡压缩为「状态文本行 + 横排按钮」，并把 P1 / P2 标为「规划」，不再显示「已连接 / 已启用」这类实时值。
- 版本只在「版本升级」卡出现一次，与运行区去重。
- 顶栏状态胶囊在两套方案中隐藏（状态改由页面内承载），baseline 保持原样以便对照。

### 方案差异

| | 方案 B | 方案 C |
|---|---|---|
| 状态与主操作 | 同一张卡左右分列 | 同一条摘要行 + 按钮行 |
| 运行详情 | 卡内 `<details>` 折叠，展开后显示完整路径 | 右侧抽屉，按「一行一项 + 右侧值」列出 |
| 数据根 | 折叠区显示完整路径 | 摘要行用 `~/OpenCodexData` 缩写，完整路径在抽屉 |
| 取舍 | 信息都在页内，多一层折叠 | 主屏最干净，路径与诊断多一次点击 |

### 方案 B+（合并版，推荐）

按「B 为骨架 + 取 C 的摘要行」的结论新增 `#overview-b2`：

- 状态区改用 C 的一行摘要：`● 运行中 · 端口 10100 · 版本 2.50.0 · 数据根 ~/OpenCodexData`，读起来是一句话而不是四个字段。
- 运行详情继续用 B 的页内折叠，不引入抽屉、遮罩与 Esc 退出路径。
- 折叠条自身带上关键事实（`v2.50.0 · Codex runtime 0.154.0-alpha.6.2 · ~/OpenCodexData`），收起状态高 51px，不再是接近空白的卡。

选择理由：概览是控制台不是报告页，运行状态与启停要零点击可见；抽屉属于「完成一个任务」的语义，用来「看一眼路径」偏重，且会与「设置 - 安装配置」「诊断中心」形成第三处路径落点。

### 实测（Playwright，无 console error / page error）

| 视口 | baseline 溢出 | 方案 B 溢出 | 方案 C 溢出 | 方案 B+ 溢出 |
|---|---|---|---|---|
| 1180 × 760 | **223px（需滚动）** | **0** | **0** | **0（收起态）** |
| 900 × 640 | **849px（需滚动）** | 201–254px | 180px | 228px |

- 1180 × 760 下 B / C / B+ 均实现单屏不滚动，baseline 仍溢出 223px。
- B+ 展开运行详情后 1180×760 溢出 187px：收起态单屏、展开态滚动，属于预期的渐进披露。
- 900 × 640（最小窗口基准）下 B / C 仍有滚动：因为 `.quick-grid` 在 `≤1100px` 视口会塌成单列。若后续采纳其中一套，建议把断点改为按窗口宽度判断或最小窗口保留 2 列。
- 方案 C 抽屉已验证：点击「运行详情 ›」打开、遮罩点击与 Esc 关闭、8 条路径行正常渲染。
- 方案 B+ 折叠已验证：默认收起（`open:false`，高 51px），点击后在「展开 / 收起」间切换；摘要行关键事实在窄窗口自动省略号。
- light / dark 均已出图。
- 面板路由往返后变体状态保持（`#overview-c → #panel → #overview-c` 回到 c）。

### 顺带修复

- `showPanel()` 引用了已被移除的 `#panelPreviewNote`，导致点击「面板」抛 `TypeError` 并中断渲染。该缺陷存在于工作区未提交改动中，非本次引入；已改为可空引用安全处理（in-window 注释条按既定结论应留在 App 窗口外）。

### 证据截图

方案对比期间的全部截图已归档到 `截图/归档-20260913/概览方案/`：`prototype-overview-baseline-1180x760.png`、`prototype-overview-b-light.png`、`prototype-overview-c-light.png`、`prototype-overview-c-sheet-light.png`、`prototype-overview-b-dark.png`、`prototype-overview-c-dark.png`、`prototype-overview-b-900x640.png`、`prototype-overview-c-900x640.png`、`prototype-overview-b2-light.png`、`prototype-overview-b2-running-light.png`、`prototype-overview-b2-expanded-light.png`、`prototype-overview-b2-dark.png`、`prototype-overview-b2-900x640.png`、`prototype-overview-b2-cards-light.png`、`prototype-overview-b2-cards-wide-light.png`。

定稿后的现行截图（`截图/现行/`）：`prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-expanded-light.png`、`prototype-overview-annotated-light.png`、`prototype-overview-annotated-dark.png`、`prototype-panel-light.png`、`prototype-panel-dark.png`、`prototype-logs-doctor-light.png`。

### 已关闭

- 「概览单屏只在大窗口成立」在方案 B / C 下不再成立（1180×760 溢出为 0）；baseline 保留该问题作为对照。

## 2026-09-13 · 方案 B / C / B+ 动作集合与 Doctor 定位修订

### 问题 1：`running` 状态缺少面板入口

- 现象：方案 B / C / B+ 在运行中只出现「停止 / 重启 / Doctor 摘要」，没有「打开面板」。
- 依据冲突：状态机 §1 明确 `running` 可用动作含「打开面板」；README / §2.2 也写「启动成功后显示端口、PID、健康结果和日志入口」。baseline 的卡头 toolbar 里有「面板」，变体把它丢了。
- 修订：动作集合改为按状态完整定义，`running` 出现「打开面板 / 停止 / 重启 / 查看日志」，且「打开面板」为该状态主按钮。

| 状态 | 出现动作 | 1180×760 溢出 |
|---|---|---|
| `loading` | 无 | 0 |
| `not_found` | 刷新状态 | 0 |
| `stopped` | 启动 OpenCodex、刷新状态 | 0 |
| `starting` / `pending` | 刷新状态 | 0 |
| `running` | 打开面板、停止、重启、查看日志 | 0 |
| `starting_failed` | 重试启动、查看日志 | 0 |
| `at_risk` / `external_takeover` | 查看建议、刷新状态 | 0 |
| `unreachable` | 刷新状态、查看日志 | 0 |

### 问题 2：Doctor 是否真有这个功能、该不该放在概览

核对官方已安装包 `@bitkyc08/opencodex@2.50.0`：

- `src/cli/doctor.ts` 文件头自述：**“`ocx doctor` - read-only environment diagnostics.”**，并写明 “Observe-only: it never sets proxy env, relocates state dirs, mutates quota, or changes networking.”
- `src/cli/registry.ts` 摘要：`Diagnose environment/network issues (paths, WSL /mnt, proxy env, ChatGPT reachability).`
- 输出的区块包括 `Paths`、`Response-state temp files`、`Codex app home targeting`、`Codex restart safety`、`Codex runtime selection` 等。
- 存在写入型参数：`--reclaim-response-temps`、`--recover-zero-byte-coordinator --yes`（后者要求先停止代理）。
- DMD §3.1 把 `ocx doctor` 列为**状态源**之一，与 `status / health / ready` 并列。

**结论：功能真实存在，但不属于运行控制。** 它是排障型只读诊断，输出是长报告，放在概览动作行会与「停止 / 重启」混成异类动作并排——正是审计 A3 指出的问题。

**修订**：概览动作行移除 Doctor；新增「诊断中心 - 运行环境诊断」区块承载它，含「运行 Doctor」按钮与只读摘要展示，并注明写入型修复参数需要停止代理 + 显式确认，原型不提供。运行异常时由 `at_risk / external_takeover` 的「查看建议」与 `starting_failed / unreachable` 的「查看日志」进入诊断中心。baseline 保留原 Doctor 按钮作为对照。

## 2026-09-13 · 概览评审意见落实（术语 / 按钮对齐 / 建议动作高亮）

### 1. 「数据根」改称「数据目录」

反馈：「数据根这个名字有点怪，改一下」。

- 用户可见文案中的「统一数据根」「数据根」全部改为「数据目录」，共 25 处：概览三套变体的摘要行、运行详情标签与按钮、方案 C 抽屉行、设置 - 安装配置分区、相关弹层说明、路由副标题与原型注释。
- `#overview` 基线的环境标签同步改名。基线保留的是**布局对照**，文案术语不属于对照变量。
- DMD 等正式文档仍写作「统一数据根」，是否一并统一列为 QA-16。

### 2. 快捷动作卡按钮对齐（三轮修订）

第一轮反馈：「三个按钮都应该居中，现在整体偏左」→ 先改为 `justify-content:center`。

第二轮反馈：「还是不对，我认为应该是左右边距与上面一致，然后再三个按钮间距一致。都这样改」→ 确认要的不是整体居中，而是**按钮行撑满内容宽度、与上方状态行左右对齐，且按钮之间等间距**。

- 实现：`.ov-compact .action-list{display:flex;gap:8px;flex-wrap:wrap;justify-content:space-between}`，紧凑卡按钮内边距收为 `7px 10px`。
- 三颗按钮一律保留（不缩减按钮数量），按钮保持自然宽度，不拉伸等宽，避免把「迁移设置」这类入口抬成和主操作同样的视觉权重。
- 状态文本行保持「标签靠左、值靠右」的满宽排布，不改。

第三轮（自查发现回归）：`space-between` 撑满后，在 **1280 与 1180（目标窗口）下三张卡的三颗按钮全部换行**。

- 根因：三列卡片在 1180 下只有 275px 宽，内容区 243px，而三颗按钮（4/5/4 字，最大 96px）加两个 8px 间距需要约 268px，装不下。此前 `flex-wrap:wrap` 一直在换行，只是页面还有余量，没有暴露成滚动溢出。
- 修法：为紧凑卡加**容器查询**，让卡片列数按内容宽度降级，而不是按视口宽度猜。

```css
.main{container-type:inline-size}
@container (max-width:979px){.quick-grid.ov-compact{grid-template-columns:repeat(2,minmax(0,1fr))}}
@container (max-width:659px){.quick-grid.ov-compact{grid-template-columns:1fr}}
```

- 之所以不用视口断点：卡片宽度同时受**侧栏是否堆叠**（`1470px` 断点）和**模拟窗口上限**（`1440px`）影响，视口断点会在 1470~1600 区间算错。
- 实测 11 个视口（900 / 1024 / 1101 / 1180 / 1280 / 1400 / 1470 / 1500 / 1600 / 1669 / 1920）：**三张卡的按钮行全部单行不换行**，左右边缘与状态行差值均为 0，卡内按钮间距相等。
- 副作用：1180 与 1500~1600 区间卡片变两列（卡片区高 176 → 368px）。1180 × 760 实测溢出仍为 0；1024 × 700 曾有 3px 溢出，已通过把紧凑卡栅格上边距由 16px 收到 12px 消除。

### 3. 建议动作高亮

反馈：「如果有新版本、检查有新版本按钮可以高亮」。

- 规则：**卡片存在明确的当前建议动作时，该按钮使用 primary 高亮。**
- 当前「版本升级」卡处于「可升级」，因此「检查更新」由默认按钮改为 primary 高亮；同卡的「升级前备份 / 升级设置」保持次级。
- 同一规则适用于后续状态：WebDAV 出现未同步变更时「立即同步」高亮、配置迁移出现未导出的本地变更时「导出配置」高亮。P1 / P2 当前为规划态，尚无可触发的建议动作，因此不预置高亮。

### 复检

- 三张卡的按钮组 `justify-content` 实测均为 `space-between`，左右边缘与状态行对齐、卡内间距相等；紧凑卡列数按容器宽度在 3 / 2 / 1 列间降级，按钮不换行；`检查更新` 类名为 `btn primary`。
- 摘要行实测：`未运行 · 端口 10100 · 版本 2.50.0 · 数据目录 ~/OpenCodexData`。
- 1180 × 760 / 900 × 640 溢出不变（B / C / B+ 在 1180 下均为 0）。
- 两种主题 × 三个视口 × 7 条路由回归，零 console error / page error。

## 2026-09-13 · 概览定稿（B+ 唯一版本 + 窗口回到标准尺寸）

### 决定

- 概览布局**定稿为 B+ 合并版**；其余三套（现行 baseline / B 三段式 / C 摘要 + 抽屉）**下线**。
- 模拟应用窗口由 `min(1440px,…) × min(1040px,…)` 收回到 **1180 × 760**（对齐同类 Tauri 桌面端默认窗口），最小 900 × 600。

### 落地

| 项 | 变化 |
|---|---|
| `#route-overview` | 直接就是 B+ 布局，不再有 `.ov-base / .ov-b / .ov-c / .ov-b2` 变体容器与 `data-ov` 属性 |
| 变体机制 | 删除变体路由（`#overview-b / #overview-c / #overview-b2`）与原型说明区的四态切换器 |
| 方案 C 抽屉 | 随变体一并删除（含 `#ovcSheetMask` 与 Esc / 遮罩 / 关闭处理、`sheet-row` 样式） |
| 旧路由回落 | `#overview-b`、`#overview-c`、`#overview-b2` 现在回落到概览，不报错 |
| 顶栏状态胶囊 | 概览页隐藏（状态已由运行摘要行承载），其余路由保留 |
| 运行详情 | 补上「PID / 进程」一格，补齐 DMD §3.2 对端口 / PID 的展示要求 |
| 窗口 | `.desktop` 列宽上限 1440 → 1180；`.window` 与 `.prototype-side` 高度上限 1040 → 760、最小 720 → 600 |

### 实测

| 视口 | 窗口实际尺寸 | 概览溢出 | 快捷卡列数 | 按钮换行 |
|---|---|---|---|---|
| 1224 × 836 | 1180 × 760 | 0 | 2 | 无 |
| 1440 × 900 | 1180 × 760 | 0 | 2 | 无 |
| 1669 × 1145 | 1180 × 760 | 0 | 2 | 无 |
| 1920 × 1080 | 1180 × 760 | 0 | 2 | 无 |

- light / dark 均无 console error / page error；`#overview-b*` 等旧 hash 回落正常。
- 视口 1180 × 760 时模拟窗口只得到 1136 × 712（低于目标窗口），概览仍无溢出；更小的窗口按需滚动。

### 清理

- 修掉随 baseline 卡片移除后产生的空引用报错：`#refreshBtn`、`#panelBtn` 的 `onclick` 绑定（`Cannot set properties of null`）。
- 方案对比期间的全部截图归档到 `截图/归档-20260913/概览方案/`。

## 2026-09-13 · 快捷动作卡改为「整行条目」

反馈：「缩小窗口后，三个板块就变成了两行，第二行空了一块，有什么更好的布局方式吗？」

### 问题

窗口收到 1180 后，快捷动作卡内容宽度约 902px，低于三列所需的 979px，于是落到两列；三张卡两列必然排成 2 + 1，第二行右侧空出一格。

### 考虑的方案

| 方案 | 做法 | 结论 |
|---|---|---|
| 甲 · 补齐第三格 | 让第三张卡 `grid-column:1/-1` 横跨两列 | 空洞没了，但卡片宽度不一、视觉重心失衡 |
| **乙 · 整行条目（采用）** | 放不下三列时直接改成单列「条目行」：标题 + 状态 + 操作同一行 | 采用 |
| 丙 · 保住三列 | 把「…设置」从按钮行移到卡片右上角，每卡只剩 2 颗按钮，三列即可容纳 | 保留为备选，见下 |

### 采用方案（乙）的实现

```css
@container (max-width:979px){
  .quick-grid.ov-compact{grid-template-columns:1fr;gap:10px}
  .quick-grid.ov-compact .card{display:grid;grid-template-columns:auto minmax(0,1fr) auto;align-items:center;gap:18px;padding:12px 16px}
  .quick-grid.ov-compact .card-head{margin-bottom:0}
  .quick-grid.ov-compact .status-lines{display:flex;flex-wrap:wrap;gap:8px 22px;margin-bottom:0}
  .quick-grid.ov-compact .status-lines span{justify-content:flex-start;gap:8px}
  .quick-grid.ov-compact .action-list{margin-top:0;flex-wrap:nowrap;justify-content:flex-end}
  .quick-grid.ov-compact .action-list .btn{white-space:nowrap}
}
```

- 三列与条目行是**同一套内容**的两种排布，只切布局不改语义：按钮撑满内容宽度、等间距、建议动作高亮等规则全部保留。
- 当前窗口上限就是 1180px（内容 ≈ 902px），因此**实际始终走条目行**；`≥979px` 的三列分支只在未来放宽窗口上限（约 >1255px）时才会生效，保留以免届时返工。

### 实测

| 窗口 | 概览溢出（改前 → 改后） | 快捷卡排布 |
|---|---|---|
| 1180 × 760（目标） | 0 → **0** | 三列 → **三行条目**（无空洞） |
| 936 × 652 | 292 → **0** | 两列带空洞 → 三行条目 |
| 856 × 600（最小） | 364 → **142** | 两列带空洞 → 三行条目 |

- 顺带把最小窗口的溢出从 364px 降到 142px：条目行比「两列 + 换行」矮得多。
- light / dark × 四个视口 × 7 条路由 + 10 个状态动作集合，零 console error / page error。

### 备选方案（丙）说明

若更希望保留标准窗口下的「三列卡片」观感，可把每张卡的「迁移设置 / 同步设置 / 升级设置」从按钮行移到卡片右上角做成次级链接，每卡只剩 2 颗按钮，三列即可容纳（三列需要每卡内容 ≈ 182px，两按钮约 164px）。代价是卡头要新增一个链接位，且按钮数由 3 变 2。

## 2026-09-13 · 多尺寸预览工具 + 模块区四种布局方案

### 预览工具（原型说明区，不属于软件界面）

为了让所有路由都能在各种窗口尺寸下看效果，原型外壳改成可由工具驱动尺寸：

| 能力 | 说明 |
|---|---|
| 尺寸预设 | 最小 900×600 / 默认 1180×760 / 大 1440×900 / 全屏 1920×1080 / 超宽 2560×1440 / 适配视口 |
| 拖拽缩放 | 模拟窗口的右边缘、下边缘、右下角三个手柄，可自由缩放（900–3400 × 600–2000） |
| 实时读数 | 说明区显示窗口实际尺寸，拖拽时同步更新 |

实现方式：窗口尺寸由 `--win-w / --win-h` 两个 CSS 变量驱动，`.window` 宽度取 `var(--win-w)`、高度取 `min(var(--win-h), calc(100vh - 48px))`（受宿主视口高度限制，读数会显示实际值）。`.main` 仍是容器查询的容器，所以页面内部的响应式与生产环境的视口断点一一对应。

一套工具对**所有路由**生效，`#overview / #panel / #logs / #settings` 都能直接换尺寸看。

### 模块区四种布局方案（同一份标记，只切位置与信息密度）

| 方案 | 位置 | 每格内容 | 概览高度 @1180×760 |
|---|---|---|---|
| 现行 | 运行详情之下 | 整行条目：标题 + 状态 + 3 按钮 | 373px |
| 甲 | **顶部一行三格** | 标题 + 状态 + 进入箭头（动作进二级） | 270px |
| 乙 | **顶部一行三格** | 标题 + 状态 + 2 个动作按钮 + 设置入口 | 319px |
| 丙 | **顶部一行三格（细条）** | 只有标题 + 进入箭头 | 212px |

切换入口在原型说明区的「布局方案 · 模块区」，切换只改 `html[data-mods]`，共用同一份 `.mod` 标记。

### 多尺寸实测（4 方案 × 5 尺寸）

| 尺寸 | 现行 | 甲 | 乙 | 丙 |
|---|---|---|---|---|
| 900 × 600 | 3 行 / 溢 0 | **1 行** / 溢 0 | **1 行** / 溢 0 | **1 行** / 溢 0 |
| 1180 × 760 | 3 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 |
| 1440 × 900 | 3 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 |
| 1920 × 1080 | 3 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 |
| 2560 × 1440 | 3 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 | 1 行 / 溢 0 |

- 甲 / 乙 / 丙 在**所有尺寸下都保持一行三格**，按钮从不换行；四种方案在所有尺寸下都无滚动溢出。
- 拖拽验证：右下角拖 +180 / +90 → 窗口由 1180×760 变为 1360×850，读数同步。
- 顺带修掉一个真实缺陷：`.card + .card{margin-top:16px}` 会让同一栅格行里的第 2、3 格整体下沉 16px，导致「一行三格」被误判为两行；已用 `.mods>.card{margin-top:0}` 修正。同时清理了失效的 `.quick-grid` 样式。

### 尚未决定（影响全屏观感）

- **W1 / W2**：全屏时内容是否限宽居中（当前未限宽，三格会拉伸到 ~840px/格）。
- **H1 / H2**：高窗口下「运行详情」是否默认展开、底部是否补最近事件区。〔H1/H2 均已定稿，见下文「高窗口最近事件区（H2 定稿）」〕

### 修复：概览与设置同时显示

- 现象：切到 `#settings` 时概览区块仍在页面上，两个路由叠在一起。
- 根因：概览改成 flex 列布局时用了 `#route-overview.ov{display:flex}`，**ID 选择器权重 (1,1,0) 压过了 `.route-section.active{display:block}` 的 (0,2,0)**，导致该区块无论是否 `.active` 都强制 `display:flex`，绕过了统一的显隐规则。
- 修法：改为 `.route-section.active.ov{display:flex}`（(0,3,0)，且必须带 `.active`），隐藏态回到 `.route-section{display:none}`。
- 复检：三个视口（1669×1145 / 1224×836 / 900×700）× 四条路由，**每条路由可见区块恒为 1**，概览在非概览路由下 `display:none`；零 console error / page error。
- 教训：任何给 `.route-section` 加排布样式的新规则都必须带上 `.active`，否则会破坏路由显隐。

## 2026-09-13 · 乙方案确认 + 窗口居中 + 动作按钮摆放五个选项

### 确认

- 模块区布局**确认为乙**：顶部一行三格，每格含状态与动作按钮，设置入口收在格内。已把 `data-mods` 默认值从 `current` 改为 `yi`。
- 原型模拟窗口改为**水平垂直居中**（`.desktop{align-items:safe center;justify-content:safe center}`）。实测 1669×1145 视口、窗口 1180×760 时上下留白各 193px。

### 修复：乙方案误隐藏「测试连接」

- 现象：WebDAV 格在乙方案下只剩「立即同步」，真实的「测试连接」不见了。
- 根因：乙用 `.btn.ghost{display:none}` 来收纳「进入设置」的按钮（迁移设置 / 同步设置 / 升级设置），但「测试连接」当初也是 `btn ghost`，被一并隐藏。
- 修法：改用 `data-settings-section` 属性区分——`.btn[data-settings-section]{display:none}`，只隐藏设置入口。三格现在都是 2 颗真实动作按钮：「导出配置 / 导入配置」「立即同步 / 测试连接」「检查更新 / 升级前备份」。

### 动作按钮摆放的五个选项（仅作用于乙）

切换入口在原型说明区「乙 · 动作按钮摆放」，只改 `html[data-mods-btns]`，共用同一份标记。

| 选项 | 做法 | 概览高度 | 单格高度 | 特点 |
|---|---|---|---|---|
| 现状 · 左对齐 | 按钮靠左，右侧留空 | 319px | 155px | 右侧近 1/3 空白，视觉偏左 |
| **等宽双列** | `grid-auto-flow:column`，两颗按钮等宽铺满 | 319px | 155px | 与状态行左右边缘对齐，整齐 |
| 主操作整行 | 首颗整行按钮 + 其余降为文字链 | 346px | 182px | 主次分明，但每格多占 27px |
| 沉底 + 页脚分隔 | `margin-top:auto` + 上边框，按钮沉到格底 | 330px | 166px | 三格页脚对齐，像卡片 footer |
| 两端对齐 | 一颗贴左一颗贴右 | 319px | 155px | 与状态行「左标签右值」节奏一致，中间留空 |

五个选项在 900×600 ~ 2560×1440 全程无溢出、按钮不换行。

### 截图

`截图/对比-20260913/`：`乙按钮-现状-左对齐.png`、`乙按钮-等宽双列.png`、`乙按钮-主操作整行.png`、`乙按钮-沉底页脚.png`、`乙按钮-两端对齐.png`

## 2026-09-13 · 模块区定稿（乙 + 等宽双列 + 页脚分隔线）

### 决策

用户从「乙 · 动作按钮摆放」的五个选项里选定**等宽双列**，并要求**补上参考卡片的页脚分隔线**。即：把原「沉底 + 页脚分隔」的 `margin-top:auto` + `border-top` 合并进「等宽双列」，作为模块区的**唯一配置**。

### 落地

- 移除变体机制：删掉 `html[data-mods]` / `html[data-mods-btns]` 两套属性与全部 `html[data-mods="..."]` 选择器，模块区 CSS 收敛为一份不带变体的规则。
- 删除原型说明区的两张切换卡（「布局方案 · 模块区」`#modsVariant`、「乙 · 动作按钮摆放」`#btnsVariant`）及对应 JS；保留「窗口尺寸预览」卡。
- JS 收敛：`#mods` 点击委托简化为「仅点右上角 `.mod-go` 箭头进入对应设置分区」（原来还要判断甲/丙方案）。
- 每格动作区：`display:grid;grid-auto-flow:column;grid-auto-columns:minmax(0,1fr)`（等宽双列）+ `margin-top:auto;padding-top:10px;border-top:1px solid var(--border-soft)`（页脚分隔线 + 沉底）。设置入口由右上角 `›` 承担，`.btn[data-settings-section]{display:none}` 不再占按钮位。

### 实测（无头 Chromium）

| 窗口 | 格宽 | 行数 | 每格按钮 | 按钮宽 | 页脚线 | 分隔线 | 三格页脚是否对齐 |
|---|---|---|---|---|---|---|---|
| 900 × 600 | 202 | **1 行** | 2 | 85 / 85 | 1px | 有 | 是（footerTop 同为 549） |
| 1180 × 760 | 293 | 1 行 | 2 | 125 / 125 | 1px | 有 | 是（439） |
| 1440 × 900 | 379 | 1 行 | 2 | 169 / 169 | 1px | 有 | 是（369） |
| 1920 × 1080 | 539 | 1 行 | 2 | 251 / 251 | 1px | 有 | 是 |
| 2560 × 1440 | 753 | 1 行 | 2 | 355 / 355 | 1px | 有 | 是（270） |

- 三格卡片等高（如 1180 下均为 166px），`.mod-actions` 用 `margin-top:auto` 沉底后三格页脚线严格对齐。
- 全尺寸无滚动溢出，按钮不换行；`<759px` 容器断点下仍保持一行三格。
- 路由回归：`#overview / #settings / #logs / #panel` 每条路由可见区块恒为 1（概览与设置不再叠显）。
- 交互回归：点任一格右上角 `›` → 路由切到 `#settings` 并落到对应分区。
- 零 console error / page error。

### 截图

- `截图/对比-20260913/定稿-模块区-乙-等宽双列-页脚分隔.png`（模块区特写）
- `截图/现行/prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-min-window-light.png`（已按定稿重出）

## 2026-09-13 · 大窗口限宽居中（W1）+ 高窗口默认展开运行详情（H1）

### 决策

上一轮遗留的两个多点尺寸观感项一并定稿：

- **W1**：内容区超过阈值时**限宽 1160px 并居中**，不再随窗口横向拉伸。官方面板 `#panelSection` 保持全幅。
- **H1**：**高窗口下「运行详情」默认展开**；用户一旦手动切换过，就不再自动干预。

### 落地

- W1：`.main` 已是容器（`container-type:inline-size`），新增 `@container (min-width:1280px){ .main>*{max-width:1160px;margin-inline:auto} .main>#panelSection{max-width:none} }`。
- H1：新增 `window.__syncDetailDefault()`——`details.open = (窗口高 ≥ 880)`；在 `setWinSize()` 与 `resize` 时调用；`summary` 的 `click` 会置 `userSet=true` 永久接管。

### 实测（无头 Chromium）

| 窗口 | `.main` 宽 | 概览宽 | 左右留白 | 运行详情 |
|---|---|---|---|---|
| 900 × 600 | 666 | 622 | 22 / 22 | 收起 |
| 1180 × 760 | 946 | 902 | 22 / 22 | 收起 |
| 1440 × 900 | 1206 | 1162 | 22 / 22 | **展开** |
| 1920 × 1080 | 1686 | **1160** | 263 / 263 | 展开 |
| 2560 × 1440 | 2326 | **1160** | 583 / 583 | 展开 |

- 限宽只在 `.main` > 1280 时生效，≤1440 窗口维持原有满宽表现，不产生额外留白。
- 官方面板在 2560 × 1440 下 `panelW = 2494 = mainW`，**未被限宽**。
- 手动切换后：在 1180 点开「运行详情」→ 放大到 1920 仍保持展开，自动逻辑不再覆盖用户选择。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-overview-wide-light.png`（1920 × 1080，限宽居中 + 展开详情）
- `截图/现行/prototype-overview-expanded-light.png`（1440 × 900，自动展开）

## 2026-09-13 · 图标修正（设置 / 日志 / 主题）

### 问题

1. **设置图标不对**：原图标是一个「圆心 + 八条放射短线」的造型，视觉上是太阳/亮度图标，不表达「设置」。
2. **日志图标不贴切**：原图标是「三条带圆点的短横」= 通用列表图标，读起来像菜单而非日志。
3. **主题切换是文字**（Light / Dark / System）：与图标化的侧边导航风格不一致。

### 修法

- 设置：换成标准齿轮（Lucide `settings`，外轮廓齿轮 + 中心圆）。
- 日志：换成「文档带横线」（`file-text` 造型，折角页 + 两行文字），表达「日志文件」而非列表。
- 主题：`Light / Dark / System` 文字改为「太阳 / 月亮 / 显示器」图标按钮，`title` 与 `aria-label` 保留中文（浅色 / 深色 / 跟随系统）；`.theme-seg` 按钮改为 28×26 图标位，选中态底色由 `--panel` 改为 `--raised`，在浅色主题下才看得出选中。

### 复核

- 浅色 / 深色两套主题下渲染无误，导航与主题按钮图标方向一致（stroke 1.5–2px，18px / 15px）。
- `applyTheme()` 依赖 `data-theme-option`，图标化后仍正常联动选中态。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-settings-light.png`、`prototype-settings-dark.png`（设置路由 + 新图标）

## 2026-09-13 · 面板主题跟随外壳（light / dark 联动）

### 问题

官方面板以 iframe 嵌入（`#panelReal` → `../原型/官方页面快照/rendered-dom.html`）。快照本身支持 `:root[data-theme=light|dark]`，但只在**加载时**读一次种子，外壳切主题后面板不跟随；且 iframe 是 file:// 跨源，父页无法直接改它的 DOM。

### 落地

- 快照 `rendered-dom.html`：头部脚本加 `window.addEventListener('message', …)`，收到 `{type:'ocx-theme',theme}` 时设置 `html[data-theme]`。
- 原型 `index.html`：新增 `syncPanelTheme()`，把解析后的 `light/dark` 用 `postMessage` 推给 iframe；在 `applyTheme()` 内调用，并监听 `#panelReal` 的 `load`（首次加载 / 重载后补推）。iframe 版本号 `?v=zh-CN-2`。
- 顺带修一个真实缺陷：快照头部脚本里 `localStorage` 被上一轮本地化误改成了 `local存储`（中文标识符，调用即 ReferenceError，被 try/catch 吞掉），导致 `ocx-shell-theme` 种子**从未生效**。已还原为 `localStorage`。

### 实测（无头 Chromium，取 `.panel-frame` 区域平均亮度，0–255）

| 状态 | 平均亮度 |
|---|---|
| 外壳 light → 打开面板 | 249.6 |
| 外壳 light → **不重载**切 dark | **37.4** |
| 再切回 light（不重载） | 249.6 |

- 面板内容随外壳主题**实时反转**，无需重载 iframe；面板与外壳同色系，无撕裂。
- 种子修复后：`localStorage` 设 `ocx-shell-theme=dark` 再加载快照，`data-theme` 正确为 `dark`（修复前为 `null`）。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-panel-light.png`、`prototype-panel-dark.png`（已按联动重出）

## 2026-09-13 · 亮暗色收敛为单一取值

### 决策

上一版是「外壳写 `ocx-theme` + `ocx-shell-theme` 两份，面板读两份」——两处状态就可能不一致。按反馈收敛为**只有一个取值** `ocx-theme`（`light` / `dark` / `system`），其它全部由它派生。

### 落地

| 角色 | 旧 | 新 |
|---|---|---|
| 外壳存储 | `ocx-theme` + `ocx-shell-theme` | 只写 `ocx-theme` |
| 面板首帧 | 读 localStorage 的 `ocx-theme` / `ocx-shell-theme` | 读外壳给的 `?theme=<resolved>` URL 参数 |
| 面板运行中 | 依赖 `postMessage` | `postMessage`（不变，仍由外壳单向下发） |
| 面板自身状态 | 有（读存储） | **无** |

- 新增 `resolvedTheme()` 统一解析 `system`；`applyTheme()` 只保存选择，`data-theme` 与面板主题都由它派生。
- 新增 `prefers-color-scheme` 变化监听：仅当取值为 `system` 时重算，其它取值不受系统影响。
- `ocx-shell-theme` 键彻底移除（外壳与快照均无残留）。

### 实测（无头 Chromium）

- `localStorage` 键：仅 `["ocx-route","ocx-theme"]`；`ocx-shell-theme` 为 `null`。
- 面板平均亮度：dark 37.4 → 不重载切 light 249.6 → 关闭再打开（首帧）249.6，全程一致。
- `system`：模拟 OS=dark → `data-theme=dark`；切 OS=light → `data-theme=light`（面板同步为 light 249.6）。
- 零 console error / page error。

## 2026-09-13 · 侧栏品牌 Logo 放大（分析 + 落地）

### 合理性分析（先测后改）

实测当前值：侧栏 232px（左右内边距各 14px，内容 203px），`.brand` 行高 `min-height:40px`，Logo `34×34`，右侧文字块 `164×35`（标题 16px / 副题 11px）。

1. **垂直方向几乎免费**：行高由 `max(40px, 内容)` 决定；文字块 35px、Logo 34px，实际由 40px 下限撑着。所以 Logo 放大到 **40px 也不会让行变高**，导航不会下移。
2. **水平方向原本就快满**：可用文字宽 = 203 − 34 − 10(gap) = 159px，而文字块本身 164px —— **已经溢出 5px**，只是 `.brand strong` 的省略号没生效（flex 子项默认 `min-width:auto`，宁可外溢也不收缩），所以标题是硬撑着排下去的。单纯放大 Logo 会把溢出推到 ~11px，更糟。
3. 结论：**合理，但必须同时给水平方向留头**。只有「放大 Logo + 侧栏加宽 + 让文字块可收缩」三者一起做才成立。

### 落地

- Logo `34 → 40px`（= `--control-lg`，正好填满行高；超过 40 会把导航顶下去，且 Logo 视觉质量压过两行文字，不建议）。
- 侧栏 `232 → 244px`，补回 Logo 变宽吃掉的 6px 并留 1px 余量。
- `.brand>div{min-width:0}`：修掉「溢出而不省略」的潜在缺陷，标题在极端窗口下才省略。
- 内嵌面板的 64px 图标栏**保持 34px**（它内部高 40px，40px Logo 会顶满无留白）。〔**已被下文「图标栏 Logo 尺寸统一」取代**〕

### 实测

| 项 | 改前 | 改后 |
|---|---|---|
| Logo | 34 × 34 | **40 × 40** |
| 侧栏宽 | 232 | 244 |
| 标题可用宽 / 文字块 | 159 / 164（**溢出 5**） | 165 / 164（余量 1） |
| `.brand` 行高 | 40 | 40（不变） |
| 导航顶坐标（1180×760） | 314 | 314（**不变**） |
| 面板图标栏 Logo | 34 | 34（不变） |

- 900×600 最小窗口下标题仍不省略。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-min-window-light.png`、`prototype-settings-light.png`、`prototype-panel-light.png` 等已按新侧栏重出。

## 2026-09-13 · 模块区右上角入口改为「更多」省略号

### 问题

三张模块卡右上角的入口图标是 `›`（右向箭头），读起来像「下一级 / 前进」，而不是「更多设置入口」。

### 落地

- 三张卡（配置迁移 / WebDAV 同步 / 版本升级）统一把 `›` 换成**水平省略号**（`⋯`，SVG 三圆点，`fill:currentColor`，13px），仍套在原来的 20px 圆形 chip 里。
- 点击行为不变：仍是 `.mod-go` 委托 → `openSettingsSection(...)`。

### 复核

- 三个 chip 几何一致：chip 20×20 / icon 13×13。
- 点任一 chip → 路由切到 `#settings` 并落到对应分区。
- 零 console error / page error。

### 截图

- `截图/对比-20260913/定稿-模块区-乙-等宽双列-页脚分隔.png`（已重出）

## 2026-09-13 · 日志 / 设置分区标签改为官方面板同款下划线 tab

### 问题

「诊断中心」与「设置」的分区切换控件是自造的 pill 分段器：设置页是 7 颗描边胶囊、由外向内换行；诊断页是右侧浮动的圆角分段器（`justify-content:space-between` 的右槽）。两者都**不是**官方面板的视觉语言，也不符合「从左起排」的阅读起点。

参考：官方面板 `.page-tabs / .page-tab / .page-tab--active` —— 整行 `border-bottom` 发丝线，文字 tab 靠左，选中项 2px 下划线 + 半粗字重。

### 落地

- 抽出与官方一致的下划线 tab 规则（`.diag-tabs` / `.settings-tabs` 共用）：
  - 容器：`display:flex;flex-wrap:wrap;gap:2px;border-bottom:1px solid var(--border)`，**从左侧起排**。
  - 文字：`--muted` → 选中 `--text`；`font-size:var(--text-control)`(13px)；`padding:8px 12px`；`margin-bottom:-1px`。
  - 选中：`border-bottom:2px solid var(--accent)`（近黑）+ `font-weight:600`。
- **诊断中心**：`.diag-tabs` 从 `.card-head` 的右槽移出，改为卡片内整行、标题下方左起（原来是右对齐浮动分段器）。
- 删除原来的 pill / 描边胶囊两套样式。

### 实测

| 项 | 值 |
|---|---|
| 设置 tab 左起点 | 341 = `.main` 内容左沿（完全左对齐） |
| 诊断 tab 左起点 | 盒子左 = 卡片内容左沿（= 卡片左 + 16 padding） |
| 容器发丝线 | `1px var(--border)` |
| 选中下划线 | `2px var(--accent)`，字重 600 |
| 设置 tab @1180×760 | 7 颗单行；@900×600 自动换 2 行（官方同为 `flex-wrap:wrap`） |

- 零 console error / page error。

### 截图

- `截图/现行/prototype-logs-light.png`、`prototype-logs-dark.png`、`prototype-settings-light.png`、`prototype-settings-dark.png`（已重出）

## 2026-09-13 · 修复导航选中态出现「双选中」

### 现象

在「日志」或「设置」页点「面板」进入面板模式后，导航里**同时有两项高亮**（「面板」+ 原来的「日志/设置」）。

### 根因

`showPanel()` 里只手动改了两项：

```js
document.getElementById('panelNavBtn').classList.toggle('active',next);
document.getElementById('overviewNavBtn').classList.toggle('active',!next);
```

它只覆盖「面板」和「概览」，**没有清掉**由 `setRoute()` 设上的 `.active`。所以从「日志 / 设置」切进面板时，旧项保持高亮 → 两项同时选中。

### 修法

改成整体重算，保证任意时刻**恰好一项**高亮：

```js
document.querySelectorAll('.nav button[data-route]').forEach(button=>{
  button.classList.toggle('active',next?button.dataset.route==='panel':button.dataset.route===currentRoute);
});
```

### 实测（无头 Chromium，逐步走查）

| 操作序列 | 高亮项 | 数量 |
|---|---|---|
| 起始 `#overview` | overview | 1 |
| → nav 日志 | logs | 1 |
| → nav 面板 | panel | 1 |
| → 顶栏「概览」 | overview | 1 |
| → nav 设置 | settings | 1 |
| → nav 面板 | panel | 1 |
| → 面板内点 nav 日志 | logs | 1 |
| → `setRoute('panel')` | panel | 1 |

- 8 步转移中高亮项恒为 1，零 console error / page error。

### 截图

- `截图/现行/prototype-panel-light.png`、`prototype-panel-dark.png`（已重出，图标栏只高亮「面板」）

## 2026-09-13 · 应用界面去掉阶段编号（P0 / P1 / P2）

### 决策

阶段编号（P0 / P1 / P2）是**项目管理语言**，不该出现在给用户看的软件界面里。应用界面只保留功能名与状态。

### 落地（应用窗口内）

| 位置 | 旧 | 新 |
|---|---|---|
| 设置分区标签 | `配置迁移 · P1` / `WebDAV 同步 · P2` | `配置迁移` / `WebDAV 同步` |
| 设置面板标题 | `配置迁移 · P1` / `WebDAV 同步 · P2` | `配置迁移` / `WebDAV 同步` |
| 模块卡标签 | `P1 · 规划` / `P2 · 规划` / `P0` | `规划中` / `规划中` / `可用` |
| 浏览器标签页标题 | `OpenCodeX-Desktop · P0 原型` | `OpenCodeX-Desktop · 原型` |

- 阶段信息只保留在**原型注译与测试器**侧栏和文档里——那是原型工具与资料，不属于软件界面。

### 复核

- 四条路由（overview / logs / settings / panel）+ 七个设置分区，逐一扫描应用窗口可见文本：`/\bP[0-2]\b/` **全部无命中**。
- 设置标签实际值：`通用 / 安装配置 / 数据与备份 / 配置迁移 / WebDAV 同步 / 日志与通知 / 版本升级`。
- 模块卡标签实际值：`规划中 / 规划中 / 可用`。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-min-window-light.png`、`prototype-settings-light.png`、`prototype-settings-dark.png`、`prototype-logs-light.png`、`prototype-logs-dark.png`（已重出）

## 2026-09-13 · 模块区卡片顺序调整

### 决策

按使用频率重排：**版本升级**（当前可用、需操作最高）放最左，**WebDAV 同步**（规划中）放最右，**配置迁移**居中。

### 落地

- 直接调整 `#mods` 内的 **DOM 顺序**（而不是只用 CSS `order`），保证视觉顺序 = 阅读顺序 = 键盘 Tab 顺序。
- 新顺序：`版本升级 → 配置迁移 → WebDAV 同步`；每张卡的三按钮与「更多」入口行为不变。

### 实测

| 卡片 | 左坐标 | 视觉位次 |
|---|---|---|
| 版本升级 | 341 | 1（最左） |
| 配置迁移 | 641 | 2 |
| WebDAV 同步 | 942 | 3（最右） |

- 点「版本升级」卡右上角 `⋯` → 路由切到 `#settings`（行为不变）。
- 零 console error / page error。

### 截图

- `截图/对比-20260913/定稿-模块区-乙-等宽双列-页脚分隔.png`（已重出）
- `截图/现行/prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-min-window-light.png`（已重出）

## 2026-09-13 · WebDAV 未配置态：内容态替换（方案 A）

### 问题

WebDAV 卡在没配置时仍显示「立即同步 / 测试连接」两颗动作按钮——点了必然失败，属于「先渲染按钮再让它失败」。

### 选项与决策

| 方案 | 做法 | 结论 |
|---|---|---|
| **A（选中）** | 状态行说明未配置，动作区换成一颗「配置 WebDAV」主按钮，`⋯` 保留 | **采纳** |
| B | 整卡置灰、只留 `⋯` 可点 | 否决：与「不渲染无解释禁用态」的既有原则冲突，且 20px 图标作唯一入口可发现性差 |
| C | 只换按钮、不灰状态行 | 与 A 差异过小，并入 A |
| D | 空态引导卡（隐藏状态行） | 会造成三格高度不一致，暂不采用 |

### 落地

- WebDAV 卡片改为**按状态渲染**：`unconfigured / disconnected / syncing / synced / error / conflict`。
- 状态 → 文案与动作：

| 状态 | 标签 | 连接状态 | 冲突策略 | 动作 |
|---|---|---|---|---|
| 未配置（默认） | 未配置 | 未配置 | 未设置 | **配置 WebDAV**（单颗整行主按钮） |
| 未连接 | 未连接 | 未连接 | 每次询问 | 立即同步 / 测试连接 |
| 同步中 | 同步中 | 同步中… | 每次询问 | 取消同步 / 测试连接 |
| 已同步 | 已同步 | 已同步 | 每次询问 | 立即同步 / 测试连接 |
| 连接失败 | 连接失败 | 连接失败 | 每次询问 | 重试连接 / 立即同步 |
| 冲突待处理 | 冲突待处理 | 已同步 | 待处理 | 处理冲突 / 立即同步 |

- 「配置 WebDAV」直达 `设置 → WebDAV 同步`；`⋯` 行为不变。动作区沿用等宽栅格，**单颗按钮自然整行铺满**，无需额外 CSS。
- 原型注译区新增「WebDAV 状态（mock 投影）」测试器，用于切换上述状态。
- 修掉一处实现缺陷：渲染调用一度写在 `const webdavStates` 之前，触发 TDZ（脚本中断）。已把首次渲染移到末尾启动序列。

### 实测（无头 Chromium）

- 默认态：标签「未配置」，动作区仅 1 颗「配置 WebDAV」，宽 255px（**整行铺满** 293px 卡宽 − padding）；三格卡高 166 / 166 / 166，**页脚线同高 439**。
- 六个状态逐一验证：标签 / 两行状态 / 动作按钮文案全部正确切换。
- 「配置 WebDAV」→ 路由 `#settings`，设置标签停在「WebDAV 同步」；「处理冲突」→ 触发对应提示；`⋯` → 仍进 `#settings`。
- 零 console error / page error。

### 截图

- `截图/对比-20260913/WebDAV-未配置-空态.png`、`WebDAV-未连接.png`、`WebDAV-已同步.png`、`WebDAV-冲突待处理.png`
- `截图/现行/prototype-overview-light.png`、`prototype-overview-dark.png`、`prototype-overview-min-window-light.png`（默认态已重出）

## 2026-09-13 · 套壳应用版本并入设置「版本升级」，外部只走通知

### 背景

界面此前只有一个「版本升级」，实际混了两个不同对象：OpenCodex 本体（官方 `ocx update`，桌面壳不接管）与套壳应用 OpenCodeX-Desktop（应用自更新，会**重启应用**）。`DMD` 的 AC-08 只覆盖前者，套壳自更新是**尚未立项的新需求**。

### 决策

- 套壳的版本升级**并入设置「版本升级」分区**，与该分区现有的 OpenCodex 版本**并列成两块**。
- 外部（概览/外壳）**不新增卡片**，套壳更新只通过**通知中心**提示；点通知进入设置对应分区。

### 落地

- 设置 → 版本升级：拆成两张卡
  - **OpenCodex 版本**：当前版本 `v2.50.0`、升级前备份、官方升级引导、失败后建议。
  - **桌面管理器版本**：当前版本 `v0.1.0 · macOS`、可用更新 `v0.1.1`（下载并重启）、更新通道与频率（stable · 每 24h，发现更新只发通知）。
- 新动作：`check-app-update`（提示）、`install-app-update`（确认弹窗，文案写明「重启不会停止 OpenCodex 代理」）、`app-update-channel`（说明通道独立）。
- 通知中心新增一条：**「桌面管理器有可用更新」**，点它 → `设置 → 版本升级`。未读角标随之 3 → 4。
- 通道区分写进文案：应用更新通道 ≠ OpenCodex 的 npm 通道。

### 实测（无头 Chromium）

- 未读角标 `4`；通知列表含「桌面管理器有可用更新」；点它 → `#settings` 且标签停在「版本升级」。
- 设置 → 版本升级两张卡：`OpenCodex 版本`（4 行）、`桌面管理器版本`（3 行：当前版本 / 可用更新 / 更新通道与频率），按钮文案与动作一一对应。
- `检查应用更新` → 提示 toast；`下载并重启` → 弹窗标题「更新 OpenCodeX-Desktop」。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-settings-upgrade-light.png`（设置 → 版本升级，两块并列）
- `截图/现行/prototype-settings-light.png`、`prototype-settings-dark.png`、`prototype-overview-light.png`、`prototype-overview-dark.png`（角标已更新）

### 遗留

- ~~`DMD` 尚无「套壳应用自更新」的验收条款~~ → **已回写**：`AC-OPENCODEX-DESKTOP-09` 已加入 `DMD-OPENCODEX-DESKTOP-MANAGER`，§6 升级边界同步补充（通道、签名校验、重启不停代理、失败可回滚），确认记录见协作包 `03-确认与回写/README.md`。

## 2026-09-13 · 通知：点开详情弹窗 + 单条删除

### 问题

通知只有「全部已读 / 清理通知 / 清理已读」这类**批量**动作；单条通知既看不到完整详情，也没法只删这一条。通知中心里点一条会直接跳转，等于把「查看」和「执行」压成了一个动作。

### 落地

- **点击任一通知 → 详情弹窗**（通知中心与「诊断中心 → 通知历史」共用）：
  - 标题、完整正文、`时间 · 级别` 元信息；
  - 有可执行动作时给「前往处理」（restore / 升级 / 应用更新 / 同步各自跳转），没有则只有「关闭」；
  - 弹窗内提供**删除**。
- **每条一个删除按钮**（`✕`）：通知中心与通知历史两个列表都有；删除后两个列表与未读角标**同步刷新**。
- 弹窗组件扩展：`openModal` 支持 `onDelete` / `cancelLabel` / `hideConfirm`，并新增 `closeModal()` 统一复位（避免上次的按钮状态残留）。

### 实测（无头 Chromium）

- 初始：未读角标 `4`；通知中心 4 条、通知历史 4 条，**页面上共 8 个单条删除按钮**。
- 点「OpenCodex 有可用更新」→ 弹窗标题正确，正文带 `10:18 · 警告`，按钮为「前往处理 / 关闭 / 删除」。
- 点「前往处理」→ 路由 `#settings` 且标签停在「版本升级」。
- 无动作的通知（应用更新）→ 只显示「关闭」+「删除」。
- 从弹窗删除 → 通知历史随之少一条，未读角标按未读状态正确递减。
- 从通知历史点删除（`sync-connected`）→ 历史与角标同步更新。
- 点通知历史里的条目 → 同样打开详情弹窗。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-notifications-light.png`、`prototype-notifications-dark.png`（通知中心 + 单条删除）
- `截图/现行/prototype-notification-detail-light.png`、`prototype-notification-detail-dark.png`（详情弹窗）
- `截图/现行/prototype-logs-light.png`、`prototype-logs-dark.png`（通知历史 + 单条删除）

## 2026-09-13 · 其他页面套用官方面板的环境光渐变

### 问题

官方面板有一层环境光背景（三处柔光叠加），原型自己的路由是纯色 `--bg`。两者并排时割裂：切到面板像换了个产品。

### 取源（官方构建产物）

官方面板 `assets/index-BoBRSehJ.css` 的 `body:before`：

```css
body:before{content:"";z-index:-1;pointer-events:none;filter:blur(70px);
  background:
    radial-gradient(42% 38% at 12% 6%,  #a4c4ff6b, #0000 70%),
    radial-gradient(46% 42% at 88% 18%, #a8e2c561, #0000 70%),
    radial-gradient(40% 36% at 70% 92%, #ffe0c24d, #0000 72%);
  position:fixed;inset:-20%}
```

暗色为 `#6082c829 / #58a08221 / #b48c6414`。

### 落地

- 新增 token `--glow-1/2/3`：light `#a4c4ff6b / #a8e2c561 / #ffe0c24d`，dark `#6082c829 / #58a08221 / #b48c6414`（与官方一致）。
- `.window::before` 承载同一组三层 radial-gradient，`filter:blur(70px)`、`pointer-events:none`、`z-index:-1`。
  - 挂在 `.window` 上：它已是 `position:relative;isolation:isolate;overflow:hidden`，负 z 伪元素正好贴在 `--bg` 之上、内容之下，且**不随 `.main` 滚动**。
  - `.main` 本身无背景色，因此光能透出；卡片仍是实色，光只在卡片间隙与页边读得到。
- 一处与官方的差异：官方是 `position:fixed;inset:-20%`（相对视口过扫），我们约束在模拟窗口内，改为 `inset:0`，光斑范围相对窗口略小，观感对齐。
- 侧栏 `--rail` 当时保持不透明；后续按反馈改成同款玻璃质感，见下一节。

### 实测（同位置取样像素）

| 位置 | 官方面板 light | 概览 light | 官方面板 dark | 概览 dark |
|---|---|---|---|---|
| 右上 | (237,249,243) | (229,244,236) | (37,42,40) | (38,44,41) |
| 右下 | (255,255,255)※ | (249,244,239) | (38,38,38) | (40,38,36) |

※ 官方面板该点落在卡片上，非背景。

- 强度与色相和面板同量级；概览/设置/日志三条路由都生效，面板路由仍是官方快照自带的环境光，不叠加。
- 零 console error / page error。

### 截图

- `截图/现行/` 全套已按新背景重出（概览 light/dark/最小窗口/大窗口/展开、设置、日志、面板、通知中心与详情）。
- `截图/对比-20260913/定稿-模块区-乙-等宽双列-页脚分隔.png` 与 4 张 WebDAV 状态图已重出。

## 2026-09-13 · 侧栏与标题栏套用官方面板玻璃质感

### 取源（官方构建产物）

官方对「栏类」元素统一用同一组 token：

```css
--glass-rail: #f9f9f9a8;   /* dark: #1717179e */
--glass-blur: saturate(1.6) blur(22px);
```

用法（官方侧栏与移动端顶栏）：

```css
.sidebar / .mobile-topbar{
  background:var(--glass-rail);
  -webkit-backdrop-filter:var(--glass-blur);
  border-right / border-bottom:1px solid var(--border);
}
```

### 落地

- 新增 token `--glass-rail`（亮 `#f9f9f9a8` / 暗 `#1717179e`）与 `--glass-blur`（`saturate(1.6) blur(22px)`），取值与官方一致。
- 套用位置：`.side`（侧栏）、`.titlebar`（标题栏）、`body.panel-mode .side`（64px 图标栏）。
- 两处都保留了原有的 1px 边框分隔。

### 实测

- 计算样式：亮 `rgba(249,249,249,0.66)` / 暗 `rgba(23,23,23,0.62)`，`backdrop-filter: saturate(1.6) blur(22px)` 均生效。
- 侧栏底色众数（取「栏内最常见颜色」，避开导航按钮）：

| | 我们 | 官方面板 |
|---|---|---|
| light | (249,249,249) | (249,249,249) |
| dark | (27,27,27) | (23,23,23) |

- 说明：官方的环境光是 `position:fixed;inset:-20%`，光斑中心被推到画布外，只有边缘渐隐进入视口，所以**玻璃栏本身看起来接近中性**——实测我们与官方一致（差 ≤4 级）。
- 顺带修正：环境光伪元素此前用 `inset:0`，导致左上角光斑过浓（侧栏顶部被染成 (231,233,237)）。改为与官方一致的 `inset:-20%` 后，中心移出画布、只留渐隐边，观感对齐。
- 零 console error / page error；导航选中态、WebDAV 状态机、通知详情与删除、面板 iframe 加载等既有回归全部通过。

### 截图

- `截图/现行/` 全套（19 张）与 `截图/对比-20260913/`（18 张）按玻璃栏 + 修正后的环境光重出。

## 2026-09-13 · 图标栏 Logo 尺寸统一

### 问题

展开侧栏与收起后的 64px 图标栏，品牌 Logo 尺寸不一致：展开态是 40px，收起态仍是上一轮保留的覆盖值 34px（`body.panel-mode .brand-mark`）。切换路由到面板时 Logo 会「变小一档」。

### 根因

上一轮把基础 `.brand-mark` 从 34px 放大到 40px 时，只改了基础规则，忘了同步面板模式的覆盖值，于是两处分叉。

### 修法

删掉 `body.panel-mode .brand-mark{width:34px;height:34px}` 这条覆盖，让图标栏直接继承基础的 40px。

### 实测

| 状态 | `.brand-mark` | `.brand` 行 | 侧栏宽 | 导航图标 |
|---|---|---|---|---|
| 展开侧栏 | 40 × 40 | 215 × 40 | 244 | 18 |
| 收起图标栏 | **40 × 40** | 40 × 40 | 64 | 18 |

- 两种状态 Logo 尺寸一致；图标栏 64px 宽，40px Logo 左右各留 12px，不顶边。
- 零 console error / page error。

### 截图

- `截图/现行/prototype-panel-light.png`、`prototype-panel-dark.png`（图标栏新尺寸）
- `截图/现行/` 其余截图一并重出。

## 2026-09-13 · 强调色三层映射收敛（方案 A，定稿）

### 问题

`design-tokens.md` 里的强调色 token 与官方逐字一致（`--accent` 亮 `#0d0d0d` / 暗 `#ececec`、`--accent-soft`、`--accent-ink`、`--accent-ring`），但「哪种语义用哪一层」的映射是乱的：同一个「选中」语义出现四种写法，且混入了第二套色相。

| 位置 | 改前写法 | 层 |
|---|---|---|
| 侧栏导航选中 | `--accent-soft` + 中性 `--border` 1px 内描边 | 浅，但描边把强调感中和掉 |
| 主题分段控件选中 | 中性 `--raised` | **与强调色脱钩** |
| 数据目录分段控件选中 | 中性 `--panel` + 阴影 | **与强调色脱钩** |
| 分区 tab 选中 | `--accent` 2px 下划线 + 字重 600 | 中 |
| 数据目录表 `code` | 品牌蓝 `--brand #3941ff` | **第二套色相** |

### 判据（定稿）

**选位置 → 浅**（`--accent-soft` + 字重 600）　**标位置 → 中**（`--accent` 2px 下划线）　**催动作 → 纯**（`--accent` 实底 + `--accent-ink`）

### 修法

| # | 选择器 | 改前 → 改后 |
|---|---|---|
| 1 | `.nav button.active` | `box-shadow:inset 0 0 0 1px var(--border)` → `font-weight:600`（去中性描边，靠浅底+字重） |
| 2 | `.theme-seg button.active` | `--raised` + `--shadow-sm` → `--accent` 实底 + `--accent-ink` |
| 3 | `.seg button.active` | `--panel` + `--shadow-sm` → `--accent` 实底 + `--accent-ink` |
| 4 | `.mock-rail span:first-child` | 去掉同一枚中性描边，与 #1 保持同层 |
| 5 | `.data-root-table code` | `--brand` → `--text`（蓝色收口） |
| 6 | `.prototype-pill` | 该选择器在 DOM 中已无引用，**整条删除**（连同 520px 媒体查询） |
| 7 | 品牌色字面量 | `.brand-mark` 阴影、resize 描边、`.proto-kicker` 三处 `#3941ff*` 魔法值改用 `var(--brand)`，避免 token 变成死代码 |

不动项：**分区 tab 的下划线保持原样**（改前已是官方 `page-tab--active` 写法）；`.scenarios button.active` 改前已是纯 `--accent`。

### 与官方的对应关系

- `.nav button.active`：官方 `nav-item.active{background:var(--accent-soft);color:var(--text)}` + `font-weight:semibold`，**不加边框**。与 #1 一致。
- `.seg`（数据目录「跟随数据目录 / 自定义路径」）：官方对应物是 `dash-ma-option` 组 —— 容器 `background:var(--surface-soft, var(--raised));padding:3px;border-radius:pill`，选中项是 `btn-primary`（纯）。我们轨道同为 `--raised`、内距同为 3px，故 #3 取纯 **与官方逐字一致**。
- `.theme-seg`：官方 CSS 里另有 `.usage-segmented`（轨道 `--surface`、选中 `--raised`）。我们的轨道是 `--panel`，若沿用 `--raised` 选中只有 Δ11，实测不可辨（这就是改前「脱钩」的观感来源），因此按 `dash-ma-option` 的「选中=强调纯色」处理。**此处是主动选择，不是照抄**。
- `--accent-soft` 三层映射：与官方 `.badge-accent`、`.page-tab--active`、`.btn-primary` 三处一一对应。

### 侧栏选中为什么维持 S0（不加深、不加指示条）

侧栏选中 = 一级导航「你在哪一页」，属**选位置**，按判据就该走浅层，与官方 `nav-item.active` 同值。实测三个候选：

| 候选 | 侧栏选中渲染底色 | 与侧栏底(249) 差 |
|---|---|---|
| **S0 浅底 + 字重（采用）** | 234 | 15 |
| S1 再加 3px `--accent` 左指示条 | 234 + 指示条 | 15 + 强调色实体 |
| S2 纯 `--accent` 实底 | 13 | 236 |

S2 会让一级导航常驻一块纯黑、与页面主按钮抢焦点；S1 是「觉得太弱」时的备选，本轮不采用。

### 实测

计算样式（headless Chromium，窗口 1180×760）：

| 元素 | light | dark |
|---|---|---|
| 侧栏导航选中 | `rgba(13,13,13,.06)` · `box-shadow:none` · `font-weight:600` | `rgba(255,255,255,.09)` · `none` · `600` |
| 主题分段选中 | `rgb(13,13,13)` + `rgb(255,255,255)` 图标 | `rgb(236,236,236)` + `rgb(13,13,13)` 图标 |
| 数据目录分段选中 | `rgb(13,13,13)` | `rgb(236,236,236)` |
| 分区 tab 选中下划线 | `rgb(13,13,13)` 2px | `rgb(236,236,236)` 2px |
| 数据目录表 code | `rgb(13,13,13)` | `rgb(236,236,236)` |

- 逐变体像素差分（对比图脚本）：`现状↔A` 在侧栏/Tab/分段处均有预期差异；`方案 A↔现状` 在分区 tab 处 **0 像素变化**，证明下划线未被误伤。
- 零 console error / page error；`node --check` 通过；导航、WebDAV 状态机、通知详情与删除、面板 iframe 加载等回归全部通过。

### 截图

- 决策用对比图：`截图/对比-20260913-强调色/强调色对比-light.png`、`强调色对比-dark.png`（5 组对比点 × 现状/方案 A/方案 B）、`侧栏选中强调色对比-light.png`、`侧栏选中强调色对比-dark.png`（S0/S1/S2）。
- 正式截图：`截图/现行/` 全套 19 张按方案 A 重出。

## 2026-09-13 · 高窗口最近事件区（H2 定稿）

### 决策

高窗口余量用于「最近事件」，而不是继续拉伸模块区或塞完整日志。

- 触发条件沿用 H1：运行详情展开时才显示，1180×760 基准窗口保持不显示。
- 位置放在运行详情之后、三张模块卡之前；语义顺序是「状态 → 详情 → 事件 → 参数型动作」。
- 只展示 **3 条状态相关摘要**，随当前状态切换 normal / risk / error 三组 mock 数据；完整排障仍进「诊断中心」。
- 用户手动收起运行详情后，最近事件跟随隐藏；不再重复展示。

### 实测

| 场景 | 运行详情 | 最近事件 | `.main` 高度溢出 | console / page error |
|---|---|---|---|---|
| 1180 × 760 | 收起 | `display:none` | 0 | 0 |
| 1180 × 900 | 自动展开 | 显示 3 条 | 264（可滚动） | 0 |
| 1180 × 900 + 外部接管 | 自动展开 | 3 条切换为 risk 摘要 | 264（可滚动） | 0 |
| 用户手动收起（900 高） | 收起 | 隐藏 | — | 0 |

- 单条事件采用固定 `time` 栏（66px）+ 自动文本列，避免等宽字体下时间轴参差。
- 4 条路由 × 2 主题 × 3 尺寸复测，激活区块数恒为 1；`node --check` 通过。

### 截图

- `截图/现行/prototype-overview-expanded-light.png`、`prototype-overview-wide-light.png` 等全套 19 张重出。

## 2026-09-13 · QA 清理

- **QA-14 已关闭**：不采用「保留 4 条完整环境路径」。定稿是一行运行摘要（含数据目录缩写）+ 页内折叠运行详情；完整路径、目录入口全部收进详情。
- **QA-16 已关闭**：用户可见文案统一「数据目录」；`DMD` 等正式文档保留「统一数据根 / 数据根」作为存储契约术语。原因：契约标题需要表达「产物统一归置」这一结构语义，而界面文案需要口语化。
- 上一版 H2「尚未决定」仅作历史索引，不作为当前开放项；最新结论以「高窗口最近事件区（H2 定稿）」为准。

## 2026-09-13 · CLI 控制面入口补齐（设置）

### 背景

`AC-10` 已确认「可选 CLI 控制面」，但原型此前没有对应界面表达；需要在设置里呈现它的默认关闭、运行中实例委托、能力边界与机读输出。

### 布局

新增「设置 → CLI 控制面」，放在「日志与通知」和「版本升级」之间：

| 区块 | 表达 |
|---|---|
| 启用开关 | 默认隐藏命令细节；「开启」先弹确认，说明本机 IPC 与仅自有域边界 |
| 命令入口 | 开启后显示 `opencodex-desktop <command> [options]` 与 `--json` |
| 运行中实例 | 「在线」状态提示；GUI 未运行时状态变更不做旁路写入 |
| 开放能力 | 发现与只读状态、启停、数据目录、备份、加密导出 / 导入、WebDAV 同步、自更新检查 |
| 明确不开放 | provider、路由、模型映射仍走官方 CLI；标记「官方 CLI」 |
| 示例命令 | `status/start/backup create/export --confirm --json`，并注明确认与备份语义同 GUI |

原型保持 mock：不写 PATH、不复制剪贴板、不连接真实 IPC。

### 实测

- 默认态：仅「启用 CLI 控制面」可见，命令入口 / 实例状态 / 能力 / 边界 / 示例均隐藏。
- 点击「开启」→ 弹窗确认 → 5 个隐藏区块全部显示。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归，active tab / active panel 恒为 1，零 console/page error。
- `pw-err.js` 5 路由零报错；全套 19 张截图重出。

## 2026-09-13 · CLI 能力边界移出选项行

### 反馈

「开放能力」「明确不开放」被渲染成两行 `setting-row`，看起来像两个可配置选项；但它们是 CLI 固有规则，不是开关。

### 修法

- 删除 `#cliScopeRow` / `#cliBoundaryRow` 两行选项。
- 新增 `#cliRules` **规则块**：仍保持「开放能力 / 明确不开放」两段，但不再有 `setting-row` 外框，标题弱化为 muted 小标题，正文弱化为 caption；位于选项列表之后、示例命令之前。
- 开启确认后仍一次性显示：命令入口 / 运行中实例 / 能力边界 / 示例。

### 实测

- 默认态 `#cliRules` 隐藏；确认开启后显示，`#cliScopeRow` 与 `#cliBoundaryRow` 已不存在。
- `node --check` 通过；`pw-err.js` 5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归无 active tab / panel 冲突，零 console/page error。
- 全套 19 张截图重出。

## 2026-09-13 · Agent 接入与提示片段

### 问题

CLI 开启后，用户仍不知道怎么让外部 Agent「知道可以操作本软件」。只给命令示例缺少两层信息：入口怎么复制、Agent 需要哪些边界提示。

### 修法

把「命令入口」升级为「Agent 控制面」入口，并拆成两块：

- **Agent 接入行**：提供「复制命令」与「复制 Agent 提示」两个动作。
- **终端验证**：保留 `status/start/backup create/export --confirm --json` 示例。
- **Agent 提示片段**：可复制文案，声明命令入口、先 `--help` 自发现能力、只操作管理器自有域、不改写 OpenCodex 配置、状态变更需确认、破坏性操作需确认标志、用 `--json` 校验输出。

原型保持 mock：不写 PATH、不复制剪贴板；真实实现应让 Agent 通过 `--help` / `--json` 自发现能力，提示片段只负责边界声明。

### 实测

- 默认态 5 个相关区块均隐藏；确认开启后「Agent 接入 / 运行中实例 / 规则块 / 终端验证 / Agent 提示片段」全部显示。
- 点击「复制 Agent 提示」触发原型 toast，不写剪贴板。
- `node --check` 通过；`pw-err.js` 5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归无 active tab / panel 冲突，零 console/page error。
- 全套 19 张截图重出。

## 2026-09-13 · CLI 帮助入口与官方 CLI 速查

### 反馈

1. 缺一个 `--help` 指令。
2. OpenCodex 自身配置仍走官方 CLI，但没有把官方入口列出来，Agent / 用户不知道去哪执行。

### 修法

- **先看帮助**：终端验证块改为以 `opencodex-desktop --help` 开头，说明 `--help` 是能力自发现入口；命令集待 IMP 冻结，不硬编码。
- **管理器自有域示例**：补充 `stop / restart / import / sync run`，与开放能力一致；保留确认标志与 mock 边界说明。
- **OpenCodex 配置 · 官方 CLI**：新增官方 CLI 速查块，列出 `ocx --help`、`ocx status --json`、`ocx restore`、`ocx update`；说明 provider / 路由 / 模型映射不属管理器 CLI，桌面壳只展示入口和检测到的接管 / at-risk 状态，不代理或包装写入动作。
- **规则块与 Agent 提示**：把「明确不开放」指向官方 CLI（`ocx`）速查；提示片段改为「如需变更，提示用户运行官方 CLI（ocx）」。

### 实测

- 默认态 7 个相关区块隐藏；确认开启后全部显示。
- `#cliAccess` 首行命令为 `opencodex-desktop --help`；`#cliOfficial` 显示 `ocx --help / status --json / restore / update`。
- `node --check` 通过；`pw-err.js` 5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归无 active tab / panel 冲突，零 console/page error。
- 全套 19 张截图重出。

## 2026-09-13 · CLI 命名定为 ocxd，Agent 提示收敛

### 决策

- 管理器 CLI 命令名定为 **`ocxd`**；官方 CLI 仍为 **`ocx`**。界面内所有示例从 `opencodex-desktop` 统一改为 `ocxd`。
- Agent 提示收敛为 3 行，并从底部移到「运行中实例」下方的选项行，行内给「复制 Agent 提示」：

```text
管理器 CLI：ocxd（先 ocxd --help；仅限管理器自有域）
官方 CLI：ocx（provider / 路由 / 模型映射）
规则：变更先确认；破坏性操作带确认标志；用 --json 校验输出。
```

- 原「Agent 接入」选项行删除；完整命令示例保留在「先看帮助 / 管理器自有域示例 / OpenCodex 配置 · 官方 CLI」。

### 待定方案 · 统一说明书（未实施）

CLI 详细用法不放设置常驻区，提供三个候选承载：

| 方案 | 形态 | 优点 | 代价 |
|---|---|---|---|
| A · 页内折叠 | 设置 CLI 分区加 `<details>`「完整命令说明」 | 单页可查、无新路由；与当前代码结构一致 | 设置页变长 |
| B · 帮助页 / 弹窗 | 「查看完整说明」打开专用帮助页或弹窗 | 设置保持短；说明书可含命令表、权限、错误码、退出码 | 需要新增路由/弹窗维护 |
| C · 真实 `ocxd --help` 引用 | 界面只留「运行 ocxd --help」 | 零文档漂移；最符合 CLI 单一事实源 | GUI 内无完整可浏览文档，依赖用户开终端 |

建议：**B 为目标形态，A 作为原型期过渡**；P0 不展开，命令集 IMP 冻结后再建说明书。

### 实测

- 页面文本中不再出现 `opencodex-desktop`；所有示例为 `ocxd`，官方示例保持 `ocx`。
- 默认态 7 个相关区块隐藏；确认开启后「运行中实例 / Agent 提示 / 规则块 / 帮助 / 自有域示例 / 官方 CLI / Agent 提示片段」显示。
- 点击「复制 Agent 提示」触发原型 toast，不写剪贴板。
- `node --check` 通过；`pw-err.js` 5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归无 active tab / panel 冲突，零 console/page error。
- 全套 19 张截图重出。

## 2026-09-13 · CLI 完整说明书定稿为方案 B，修复提示复制一致性

### 决策与修法

- 统一 CLI 说明书采用 **方案 B：帮助弹窗**。Agent 提示行提供「查看完整说明」，打开宽版滚动弹窗；不再做页内折叠。
- 说明弹窗承载：`ocxd --help` 自发现入口、管理器自有域命令示例、运行中实例与确认规则、官方 `ocx` 配置入口边界、`--json` 校验与退出码待 IMP 冻结说明。
- 修复「复制 Agent 提示」行为：点击后打开只读弹窗，展示与底部 `#cliAgentPrompt` 同源的提示正文；原型不写剪贴板，弹窗文案明确真实实现才写入剪贴板。
- 通用弹窗补充滚动与关闭时还原宽度，避免长说明破坏既有小弹窗布局。

### 实测

- 默认态 7 个相关区块隐藏；确认开启后全部显示。
- 弹窗内的 Agent 提示与 `#cliAgentPrompt` 逐字符一致，测试结果为 `copyMatch: true`。
- 完整说明弹窗启用 `modal-wide`，确认按钮隐藏、关闭文案正确；关闭后弹窗宽度还原。
- `node --check` 通过；5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
- 全套 19 张现行截图重出，并新增 CLI 分区、Agent 提示弹窗、完整说明弹窗三张特写。

## 2026-09-13 · CLI 帮助弹窗改表格式，复制提示去弹窗

### 反馈与修法

- 「复制 Agent 提示」恢复为 toast 原型模拟，不再打开弹窗；真实实现复制底部显示片段。
- 完整说明弹窗改为两个独占区块：`ocxd · 管理器自有域` 与 `ocx · OpenCodex 官方配置`。
- 弹窗内统一为「复制按钮 + 指令 + 说明」表格；每行一个指令，说明右侧纵向对齐。
- 帮助弹窗宽度从 760px 收敛到 640px；点击复制保持弹窗开启，toast 明确原型不写剪贴板。

### 实测

- 点击「复制 Agent 提示」不打开弹窗，仅显示 toast。
- 帮助弹窗只保留 `ocxd` 与 `ocx` 两个区块，共 13 条指令；点击第一条显示「复制 `ocxd --help`」toast，弹窗仍开启。
- `node --check` 通过；5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
- CLI 分区与帮助弹窗特写重出。

## 2026-09-13 · CLI 帮助弹窗按窗口比例与点击复制定稿

### 反馈与修法

- 帮助弹窗不再超出软件窗口：宽版尺寸改为 `min(86%, 720px)`，高度上限改为 `78%`；弹窗内部继续滚动。
- 弹窗遮罩从原型侧栏容器移回软件窗口内容区，使高度比例和遮罩都相对软件窗口计算。
- 去掉独立「复制」按钮：整条指令就是可点击复制面，右侧常态显示「复制」，点击后显示「已复制」并在 1.4 秒后还原；toast 保留原型边界提示。
- 指令列与说明列调整为 56% / 44%，长命令不再挤压说明。

### 实测

- 1180 × 760 软件窗口下，帮助弹窗约 720 × 560，宽高占比约 61% / 74%，未超出窗口。
- 点击 `ocxd --help` 后，指令面显示「已复制」，弹窗保持开启；1.4 秒后恢复「复制」。
- `node --check` 通过；5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
- CLI 分区与帮助弹窗特写重出。

## 2026-09-13 · CLI 帮助表格纵向居中对齐

### 修法

- 指令列与说明列的单元格由顶部对齐改为纵向居中。
- 指令复制面自身内部仍保持居中，长说明换行时整行视觉轴保持一致。

### 实测

- 13 行指令与说明的中心点误差均小于 1px，全部标记为 aligned。
- `node --check` 通过；5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
- CLI 分区与帮助弹窗特写重出。

## 2026-09-13 · 原型完整性复查

### 结论

CLI 控制面本轮反馈已全部收口：开启流程、运行实例、Agent 提示、完整说明弹窗、`ocxd / ocx` 命令表、点击复制态、窗口比例与纵向对齐均符合定稿。其余主流程无新增回归。

### 复查项

- 默认态 7 个 CLI 区块隐藏；开启后 7 个区块显示。
- Agent 提示复制不弹窗，只显示 toast；真实实现复制底部片段。
- 帮助弹窗 720 × 560，位于 1180 × 760 软件窗口内。
- 指令面点击后显示「已复制」，1.4 秒还原；弹窗保持开启。
- 13 行指令与说明纵向居中。
- 原型文本中无 `opencodex-desktop` 旧命名；仅使用 `ocxd` 与 `ocx`。
- HTML 无重复 ID；`node --check` 通过。
- Playwright 5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
- 22 张现行截图保持最新，包含 CLI 分区与帮助弹窗特写。

### 已知非缺陷

- `index.html` 中静态 `<div>` 开闭数量相差 1，来源是历史嵌套窗口与遮罩容器的既有布局标记；浏览器解析正常，当前功能、弹窗比例与遮罩定位均通过实测。不需要为本次 CLI 任务重排全文件结构。

## 2026-09-13 · 原型与需求一致性审计整改（确定性内容）

### 依据

采纳审计报告中可立即落地的确定性口径，不在原型中提前实现 IMP 才能冻结的状态机。

### 修法

- 官方 restore 前置确认改为显式链路：当前配置摘要与影响 → 生成备份 → 显式确认 → 官方 restore 引导 → 可取消；页面明确桌面壳不直接改写配置。
- 数据目录与 `OPENCODEX_HOME` 选择入口补充运行中禁止切换、已有数据摘要、「仅切换引用 / 迁移数据」、备份与失败回滚规则；`OPENCODEX_HOME` 补充不得互相嵌套。
- 日志离线文案区分「官方 observe 实时日志不可用」与「本地历史日志仍可查看」。
- CLI 设置示例与帮助弹窗统一标注「示例命令，非最终契约」；确认命令集与错误码仍待 IMP 冻结。
- 所有普通确认弹窗底部统一显示「当前为原型演示；真实实现尚未启用。」，与拦截 toast 形成一致边界。
- 删除原型注释中重复的 `settings` 注释对象，避免运行时覆盖。
- 文档清理：移除概览保留 Doctor 的旧表述；设置分区从五个改为当前八个；路由与首次发现术语统一为「管理器数据目录」，技术语境保留 `OPENCODEX_HOME`。

### 实测

- 数据目录切换弹窗包含运行中禁止切换、已有数据摘要、迁移方式、备份与回滚边界；`OPENCODEX_HOME` 弹窗包含不得互相嵌套。
- at-risk 的「查看建议」弹窗完整出现配置摘要、备份、显式确认、官方引导与取消边界。
- 诊断中心离线日志明确实时能力不可用、本地历史可查看。
- CLI 设置示例、官方 CLI 速查与帮助弹窗底部均标注非最终契约。
- 每个确认弹窗显示统一原型边界说明。
- `node --check` 通过；5 路由零报错；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 首次打开前置条件门禁定稿

### 方案决策

- 采用候选 A：概览环境门禁卡。
- 前置通过后隐藏门禁，恢复常规概览；完整说明收敛到设置 → 安装配置。
- 不采用强首屏引导：会打断产品主语境。
- 不采用顶部横幅：阻断原因与修复动作分散，首屏因果不够直接。

### 交互定稿

- 首次打开自动检查；不要求用户先进入设置。
- 检测按 `node` → `npm` → `ocx` 顺序短路；前项缺失时后续显示「未检查」。
- 缺 Node.js 时区分有 / 无 Homebrew：有 Homebrew 给 `brew install node`；无 Homebrew 给 Node.js LTS 官方渠道。
- 缺 npm 时建议重装 Node.js LTS 或检查 PATH。
- Node/npm 通过但缺 `ocx` 时展示官方 npm 包安装命令，并标注「示例命令，非最终契约」。
- 命令使用点击复制样式；原型声明不写剪贴板。
- 「查看完整指引」跳到设置 → 安装配置，不新增弹窗。
- 安装配置中的 OpenCodex 发现在前置缺失时显示被阻塞。
- 应用只做检测与引导，不执行 `brew`、`npm` 或 `ocx` 安装，也不修改 OpenCodex 配置。

### 实测

- 默认态为环境通过：测试器默认选中「环境通过」，门禁隐藏，安装发现恢复常规结果。
- 首次 800ms mock 检查后进入环境通过；点击测试器缺失态会展示对应门禁并阻塞安装发现。
- 缺 Node.js（无 brew）、缺 npm、缺 OpenCodex 三种缺失态分别展示对应最短修复命令。
- 缺 OpenCodex 态标注「示例命令，非最终契约」。
- 「查看完整指引」激活「设置 → 安装配置」；安装发现区域显示「已按前置条件暂停」。
- 命令点击后显示「已复制」，并提示「原型模拟；不写剪贴板」。
- 环境通过后门禁隐藏，安装发现恢复常规结果。
- 原型注释新增环境前置条件测试器，可切换 6 个状态。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 安装配置一列定稿

### 修法

- 安装配置从三列改为单列卡片流：运行环境与安装发现、数据目录与 `OPENCODEX_HOME`、数据目录分区。
- 安装发现改为统一表格行，新增 Node.js 与 npm 的路径和版本；路径与版本使用 `--text-caption`，与其他路径信息一致。
- 数据目录与 `OPENCODEX_HOME` 移除跟随 / 自定义 Tab，自动展示当前路径，并在同一行提供复制、打开和修改。
- 数据目录分区路径本身作为点击复制面，行尾提供 Finder / 文件资源管理器打开入口。
- 原型只模拟复制与打开边界，不写剪贴板、不打开真实目录。

### 实测

- 安装配置为单列；安装发现包含 6 行：Node.js、npm、可执行文件、安装形态、版本、Codex runtime。
- 路径与版本字号均为 11px，与其他路径信息保持一致。
- 数据目录 / `OPENCODEX_HOME` 无 Tab；每行具备复制、打开、修改。
- 分区路径点击显示「已复制」，并提示不写剪贴板；打开按钮只显示原型拦截。
- `node --check` 通过；浏览器零 console/page error。

## 2026-09-13 · 环境门禁层级与阻塞行为

### 修法

- 门禁标题从 16px 收敛到 14px；描述从 14px 收敛到 12px。
- 检查项名称、状态与底部说明分别对齐 `--text-label`、`--text-caption`；状态胶囊降低为行内尺寸。
- 命令行保持既有复制面层级；门禁按钮改为 30px 高、12px 字号，避免与主运行控制同级。
- 前置缺失时概览只保留门禁和「刷新状态」；启动、运行详情、最近事件与三张模块卡被阻塞隐藏。
- 环境通过后恢复运行状态机、详情、最近事件与模块卡。

### 实测

- 门禁字号实测：标题 14px、描述 12px、检查项 12px、检查值 11px、按钮 12px / 30px。
- 缺 Node.js 时「启动 OpenCodex」隐藏，「刷新状态」保留；运行详情、最近事件与模块卡均隐藏。
- 环境通过后模块卡与运行详情恢复显示。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 数据目录目标表结构统一

### 修法

- 「数据目录与 `OPENCODEX_HOME`」从独立路径行改为与数据目录分区一致的表格结构。
- 列结构统一为：目标 / 当前路径 / 操作。
- 删除单独「复制」按钮；路径本身复用点击复制面。
- 操作列保留「打开」与「修改」。

### 实测

- 该区块包含 1 张表、2 个路径复制面，无独立「复制」按钮。
- 点击路径显示「已复制」，并提示不写剪贴板。
- 行内操作为「打开 / 修改」。
- `node --check` 通过；浏览器零 console/page error。

## 2026-09-13 · 数据目录目标表列序统一

### 修法

- 「数据目录与 `OPENCODEX_HOME`」表头从「目标 / 当前路径 / 操作」改为「路径 / 用途 / 操作」。
- 路径在首列并保留点击复制；用途列改为「数据目录」或 `OPENCODEX_HOME`；操作列仍为「打开 / 修改」。

### 实测

- 表头实测为：路径、用途、操作。
- 首行内容为：路径复制面、数据目录、打开 / 修改。
- `node --check` 通过；浏览器零 console/page error。

## 2026-09-13 · 数据与备份控件可操作化

### 修法

- 三个自动备份开关改为真实开关控件，使用 `role="switch"` 与 `aria-checked` 表达开启 / 关闭。
- 「备份保留策略」和「备份完整性校验」改为可选中的配置控件样式，表达可修改但不在原型中真实保存。
- 点击开关切换 mock 开启态，点击配置项提示会打开选项；均声明不保存真实偏好。

### 实测

- 3 个开关默认开启；点击后状态在 true / false 之间切换。
- 「备份保留策略」「备份完整性校验」为 2 个可点击配置项。
- toast 显示 mock 边界；浏览器零 console/page error。

## 2026-09-13 · 数据目录分区路径列统一

### 修法

- 「数据目录分区」表头从「区域」改为「路径」。
- 分区单元格从短名改为完整路径，与上方数据目录表的路径表达一致。
- 两张表均为全宽，实际宽度一致。

### 实测

- 两张表实测宽度均为 856px。
- 分区首行为 `/Users/ezio/OpenCodexData/manager-state`；其余分区同样展示完整路径。
- `node --check` 通过；浏览器零 console/page error。

## 2026-09-13 · 设置页可编辑控件全覆盖

### 修法

- 通用、日志与通知、配置迁移、WebDAV 同步中的布尔项统一改为开关控件；枚举与策略项统一改为可点击配置项。
- WebDAV 的服务地址、远端目录、账号、应用密码、加密口令与冲突策略移除只读态，作为可编辑 mock 字段展示。
- 「启用 CLI 控制面」改为开关控件；开启和关闭都保留确认弹窗，开启后继续展示运行实例、Agent 提示与命令能力。
- 桌面管理器「更新通道与频率」改为可点击配置项，不再使用「改通道」动作按钮。
- 补充隐藏 setting row 的显示规则，避免 `hidden` 属性被 grid 布局覆盖。

### 实测

- 审计到 12 个开关与 10 个配置项；通用、数据与备份、迁移、同步、日志与通知、CLI、版本升级中的偏好项均具备可编辑控件。
- WebDAV 6 个字段均无 `readonly`，可编辑但原型不保存。
- CLI 开关默认关闭；开启 / 关闭确认后 `aria-checked` 正确切换，相关区块同步显示 / 隐藏。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 选项控件改为真实编辑器

### 修法

- 设置页 10 个策略型「配置项按钮」改为原生 `select`，当前值保留为默认选项，并补充可选枚举。
- 原生选项控件与输入框共用统一的边框、背景、圆角、字号和 hover / focus 反馈；选择控件使用紧凑高度，适合 setting row 右侧。
- 客户端加密保留「强制开启」单选项，表达不可关闭的强制语义。
- WebDAV 输入框继续允许编辑，但保持原型不保存边界。

### 实测

- 审计到 10 个可编辑下拉选项、6 个可编辑输入框、12 个开关控件。
- 下拉选项可切换；输入框无 `readonly` 且可编辑。
- 下拉控件实测高度 30px、字号 12px；输入框高度 34px、字号 12px，视觉字体一致。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · WebDAV 冲突策略改为下拉

### 修法

- WebDAV 设置中的「冲突策略」从可编辑输入框改为原生下拉。
- 默认值保留「每次询问」，新增「保留本地 / 保留远端 / 保留双份」可选策略。
- 下拉沿用 `.input` 尺寸与颜色体系，和同组输入框保持一致。

### 实测

- 冲突策略可从「每次询问」切换为「保留双份」。
- 控件实测高度 34px、字号 12px、边框 1px，与输入框一致。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 下拉控件应用内风格统一

### 修法

- 11 个原生 `select` 全部替换为应用内下拉，保留原有当前值和可选枚举。
- 触发器使用与按钮和输入框一致的边框、背景、圆角、34px 高度与 12px 字号。
- 菜单改为应用面板风格，包含统一边框、圆角、阴影、hover 态和当前选中态。
- 增加点击外部关闭、Escape 关闭、方向键移动选项、Enter 确认，以及底部空间不足时向上展开。

### 实测

- 11 个原生 `select` 均已移除；应用内下拉数量为 11。
- 可从「内嵌优先」切换为「浏览器兜底」，触发器同步显示新值。
- 打开其他下拉或点击页面空白处时，已打开菜单会关闭。
- 键盘可打开、移动、确认和 Escape 关闭，零 console/page error。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 关于页与官方归属优先

### 方案

- 采纳独立侧栏「关于」入口：版权与归属是长期事实，不放设置末尾，也不在官方面板底部混合两个版本对象。
- 页面顺序先官方、后本应用：顶部展示 OpenCodex 归属、MIT 许可与官网 / GitHub / Issues；中部声明 OpenCodeX-Desktop 只是独立桌面壳封装；随后展示版权与第三方声明；底部保留版本事实。
- 不重复分发 OpenCodex、不修改官方配置、不接管更新核心，均写进可见边界。

### 落地

- 侧栏新增「关于」，放在设置前。
- 新增 `#route-about`，官方项目区使用轻量品牌渐变提升优先级；本应用、版权与版本信息使用普通卡片层级。
- 官方入口与许可 / 第三方声明按钮仅拦截为原型 toast，不打开外部链接。
- 原型注译新增关于页说明。

### 实测

- 关于页标题为「关于」，官方项目区显示 OpenCodex、lidge-jun 创建、bitkyc08 npm 维护、MIT。
- 入口按钮实测：官网 / GitHub / Issues；版权区按钮为查看官方许可 / 查看第三方声明。
- 点击链接提示「原型拦截：不会打开……」，不打开真实外部站点。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。

## 2026-09-13 · 关于并入设置并统一下拉宽度

### 修法

- 关于不再作为独立侧栏路由，改为设置分区最后一个「关于」Tab。
- 保留 `#about` 作为兼容链接，进入后自动重定向到设置并选中「关于」。
- 下拉菜单默认宽度与触发器一致；仅当选项文本超过触发器宽度时，菜单可按内容扩展并保留视口上限。
- 官方归属、许可、第三方声明、桌面壳边界与版本信息集中到该 Tab。

### 实测

- 侧栏不再有「关于」按钮；设置 Tab 数量增至 9。
- `#about` 进入后标题为「设置」，当前 Tab 为「关于」，官方信息可见。
- 备份保留策略下拉实测触发器 150px、菜单 150px，宽度一致。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console/page error。
## 2026-09-13 · 关于页版本信息上移与官方入口收敛

### 修法

- 移除特殊背景色的「版本信息」独立卡，版本事实并入顶部官方项目区。
- 官方项目标题右侧增加 GitHub 图标入口，保持与官网 / Issues 的原型拦截行为一致。
- 官方项目描述收敛为「第三方 OpenCodex 的桌面壳封装。」，减少重复边界说明。
- 中窄屏版本事实改为两列，极窄屏保持单列。

### 实测

- 关于页不再存在 `about-compact` 卡；官方项目区展示 4 项版本事实。
- GitHub 图标入口数量为 1；点击提示「原型拦截：不会打开OpenCodex GitHub 仓库。」。
- 官方描述已更新，无 console / page error。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零错误。
## 2026-09-13 · 关于页布局分层与作者入口

### 修法

- 官方项目卡与本应用卡统一为普通面板背景，移除顶部品牌渐变特殊化。
- 官方项目卡拆分为「标题与说明 / 版本与归属 / 官方入口」三个纵向信息层，标题右侧不再插入图标导致基线错位。
- 归属信息改为创建者、npm 维护者、许可三项，官方入口用官网、GitHub 图标、Issues 统一排列。
- 本应用卡补齐作者 `gzers`，并提供与官方入口同风格的 GitHub 入口；作者、运行平台、官方入口按三列对齐。
- 版权卡说明与操作改为统一纵向分层。

### 实测

- 官方与本项目 GitHub 图标入口均可点击并原型拦截，不打开外部站点。
- 官方信息为 4 项版本事实 + 3 项归属；本应用为作者、运行平台、官方入口 3 项。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认浅色主题下卡片背景、三列对齐与入口基线一致。
## 2026-09-13 · 关于页同构布局与版权操作收纳

### 修法

- 官方项目卡与本应用卡使用同一结构：标题区、四列事实区、链接操作区。
- 官方项目事实合并为项目版本、发行包、代码作者、项目维护者；本应用保留应用版本、项目作者、运行平台、开源许可。
- 移除本应用卡的四条边界列表，避免与版权卡重复说明。
- 官方与本应用 GitHub 入口统一使用可辨识的实心图标加文字按钮。
- 版权卡改为标题、说明与右下角两个详情按钮，不再使用多列表格或左侧窄栏。

### 实测

- 本应用卡 `about-list` 数量为 0，关于页保持 3 个信息面板。
- 官方与本应用事实区各为 4 项，`about-facts` 数量为 2。
- GitHub 图标按钮为 2 个，点击均原型拦截，不打开外部站点。
- 版权卡说明数量为 1，操作按钮为 2 个且右下对齐。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认三块布局纵向节奏、四列事实与按钮位置一致。
## 2026-09-13 · 关于页标题字号与版权操作对齐

### 修法

- 官方项目标题从 26px 收敛为应用 subtitle 字号，与本应用标题保持一致。
- 版权详情按钮从右对齐改为左对齐。
- 按钮从普通 ghost 按钮改为链接 chip，形状与官方、本应用底部按钮一致。
- 版权操作区上方增加统一分割线，并保留与链接区一致的 14px 间距。

### 实测

- 官方标题与本应用标题计算字号均为 16px。
- 版权操作区 `justify-content` 为左对齐，上分割线 1px，上间距 14px。
- 版权区使用 2 个链接 chip，普通按钮数量为 0。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认三块卡片的标题、分割线和按钮形状一致。
## 2026-09-13 · 移除关于页外层卡框

### 修法

- 关于分区外层 `about-card` 取消边框、背景、内边距与阴影。
- 保留官方项目、本应用、版权与归属三个内层面板作为独立卡片。

### 实测

- 外层 `about-card` 计算样式为边框 0、透明背景、内边距 0、阴影 none。
- 内层官方面板保持边框 1px、普通面板背景与 20px 内边距。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认不再出现外层框。
## 2026-09-13 · 统一设置分区卡片间距

### 修法

- `.card + .card` 不再依赖外边距叠加，改为全部由设置分区的 grid gap 控制间距。
- 激活态 `.settings-panel` 统一使用 `display:grid;gap:16px`。
- 关于页 `about-card` 的内层间距从 14px 统一到 16px。
- 修正安装配置卡片的既有 16px 外边距，避免其与 grid gap 叠加成 32px。

### 实测

- 安装配置三张卡片的实际纵向间距均为 16px；修复前为 32px。
- 版本升级两张卡片实际间距为 15px（子元素盒模型导致的 0.5px 取整），不出现 32px。
- 关于页三块面板实际间距均为 16px。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认关于页不再出现外层框，卡片间距保持一致。
## 2026-09-13 · 本应用补充 Issues 入口

### 修法

- 「本应用 GitHub」右侧新增 Issues 链接 chip，与官方项目入口结构一致。
- `app-repo` 与 `app-issues` 分别映射到 OpenCodeX-Desktop 的仓库与 Issues 提示，不与官方项目混淆。

### 实测

- 本应用入口按钮为「本应用 GitHub」与「Issues」，共 2 个。
- 点击本应用 Issues 提示「原型拦截：不会打开OpenCodeX-Desktop Issues。」，不打开外部链接。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认本应用链接区与官方项目链接区结构一致。
## 2026-09-13 · 统一全站路由卡片间距

### 修法

- 通用激活路由从 block 改为 grid，并为所有路由统一设置 16px 间距。
- 概览专用 flex 路由规则收敛为同样的 grid + 16px，避免规则分叉。
- 概览三张模块卡的横纵间距从 12px 统一到 16px。

### 实测

- 诊断中心两张卡片实际纵向间距为 16px，修复前为 0。
- 概览运行摘要、运行详情、模块区之间的实际纵向间距为 16px。
- 概览三张模块卡之间的 grid gap 为 16px。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认诊断中心与概览的卡片节奏一致。
## 2026-09-13 · 诊断中心 Tab 提升为路由顶层

### 修法

- 将「日志历史 / 通知历史」从第一张卡内部移到路由顶层，紧接页面标题与副标题。
- 删除第一张卡重复的「诊断中心」卡头；当前 Tab 内容直接承载在卡片中。
- 保留「运行环境诊断」作为并列卡片，日志页结构与设置页的「顶层 Tab → 内容卡」一致。

### 实测

- 页面标题仍为「诊断中心」；路由内第一层子元素为诊断 Tab。
- 第一张卡不再包含 `card-head` 和 `h2`，只有当前 Tab 的日志 / 通知内容。
- 诊断 Tab 数量为 1；切换「通知历史」后通知视图 active、日志视图关闭。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认不再出现标题、Tab、内容三层嵌套框。
## 2026-09-13 · 统一顶层 Tab 与内容卡间距

### 修法

- 诊断 Tab 与设置 Tab 的自身上下外边距移除，间距全部交给路由层统一控制。
- 顶层 Tab 到当前内容卡片的间距与卡片之间间距保持 16px。

### 实测

- 概览、日志、设置从顶部内容区到首个内容的实际间距均为 18px，来自统一的页面顶栏外边距。
- 日志页 Tab 到内容卡、内容卡到环境诊断卡的实际间距均为 16px。
- 设置页 Tab 到当前分区内容卡的间距为 16px。
- `node --check` 通过；4 路由 × 8 设置分区 × 2 主题 × 2 尺寸回归零 console / page error。
- 截图确认日志与设置的 Tab、卡片间距节奏一致。

## 2026-09-13 · 面板与外壳字体一致性预览

### 修法

- 官方快照与桌面壳统一使用同一字体栈，默认 100% 保持 1:1 CSS 像素。
- 面板内嵌页按同一缩放因子渲染：80% / 90% 收入更多内容，110% / 120% 放大字面而不裁切内容。
- 设置 → 通用新增「界面缩放」，选项为 100% / 90% / 80%；面板内保留临时 +/− 测试控件。
- 原型注释明确：真实 Tauri 实现使用 setZoomFactor()，不通过 CSS 拉伸 iframe。

### 实测

- 80% 时面板 iframe 外框 1392.5×892.5，面板实际内容 1741.25×1801.5，返回 100% 后恢复 1114×714。
- 90% / 100% / 110% / 120% 实测 iframe 外框分别为 1237.77×793.33 / 1114×714 / 1012.72×649.08 / 928.33×595。
- 设置选择 90% 后 trigger 显示 90%，`--panel-zoom=0.9`，面板重新打开后同步生效。
- `node --check` 通过；面板与设置切换流程零 console / page error。

## 2026-09-13 · 通用设置共享官方启动项

### 修法

- 设置 → 通用新增「随 Codex 启动 OpenCodex」，并标注「官方共享」。
- 与官方面板保持同一配置语义，默认开启；说明中明确走官方 shim `ocx ensure`，不做旁路写入。

### 实测

- 通用偏好首行开关默认为开启。
- 开关可点击并显示原型模拟反馈。
- `node --check` 通过。

## 2026-09-13 · 审计共享配置并拆分通用卡片

### 审计结论

- 已定共享配置：`随 Codex 启动 OpenCodex`，配置源和写入通道都跟随官方；默认开启。
- 官方快照中可见的候选项 `不登录直接打开 Codex`、`使用客户端压缩`、`实时流式返回答案`、`Shadow Call 拦截` 属于官方运行行为，但未确认桌面壳共享收益与官方写入通道，暂不放入共享卡片。
- 语言、子代理、V2、模型、sidecar、替换模型、渠道属于官方模型 / 路由 / 升级域，不放进管理器通用设置；需要变更仍走官方面板或官方 CLI。
- 管理器自有项归入 `桌面壳偏好`：界面缩放、自动打开面板、面板打开方式、关闭窗口后保持代理运行、启停结果通知、同步冲突提醒。

### 落地

- 设置 → 通用拆成两张卡：第一张「桌面壳偏好」，第二张「官方共享配置」。
- 共享卡片说明与行内标签明确「与官方同源、不做旁路写入」；模型、路由与 provider 保持官方管辖。
- 原型注译同步改为「桌面壳偏好」与「官方共享配置」两组说明。

### 实测

- 通用设置两张卡标题分别为「桌面壳偏好」「官方共享配置」，共 7 行设置。
- 官方共享配置当前只有 1 行，且带「官方共享」标签；开关点击提示「原型模拟：随 Codex 启动 OpenCodex已关闭；不保存真实偏好。」。
- `node --check` 通过；通用设置切换流程零 console / page error。

## 2026-09-13 · 共享配置候选记录

### 结论

- 通用设置的「官方共享配置」当前只保留 `codexAutoStart`，对应「随 Codex 启动 OpenCodex」。
- `codexDesktopAuthless` 与 `codexClientCompaction` 虽然来自官方 `/api/settings`，可被共享，但本轮决定只记录，不加入原型。
- 原型不新增共享配置行，避免首批面暴露过多官方运行行为。

### 候选明细

| 官方字段 | 展示语义 | 结论 | 说明 |
|---|---|---|---|
| `codexAutoStart` | 随 Codex 启动 OpenCodex | 已共享 | 当前唯一放入通用设置的官方共享项，默认开启。 |
| `codexDesktopAuthless` | 不登录直接打开 Codex | 暂不共享 | 官方接口支持，但涉及登录形态，先记录。 |
| `codexClientCompaction` | 使用客户端压缩 | 暂不共享 | 官方接口支持，但涉及运行行为，先记录。 |
| `oauthOpenBrowser` | 登录时打开浏览器 | 不放入首批 | 偏登录流程细节。 |
| `streamMode` | 流式处理策略 | 不放入首批 | 偏运维/高级策略。 |
| `appOwnedMemoryBudgetMb` | 应用内存预算 | 不放入首批 | 数值型运维配置。 |
| `ultraFastTier` | 保留自定义 ultrafast 层级 | 不放入首批 | 属模型层级策略。 |
| `codexMainAccountHardLock` | 主账户高占用阻断 | 不放入首批 | 有阻断/安全含义，需要确认流程。 |
| `codexAccountPickerEnabled` | 账号选择器开关 | 不放入首批 | 需要账户映射配合。 |
| `codexQuotaAutoRefresh` | 按账户刷新额度 | 不放入首批 | 按账户结构，UI 复杂。 |
| `codexShimAutoRestore` | Codex shim 自动恢复 | 不共享 | 无 `/api/settings` 路由，只能走 `ocx config set`。 |

### 实测

- 审计基于官方源码 `v2.53.0 / commit aa05b3e`；原型本轮无代码变更。

## 2026-09-13 · 新增启动主界面偏好

### 修法

- 设置 → 通用 → 桌面壳偏好新增「启动时打开主界面」。
- 该项归属桌面壳自有配置，不使用「官方共享」标签；默认开启。
- 关闭时仅启动代理托管，不自动显示主窗口。

### 实测

- 启动时打开主界面开关默认开启，可点击并显示原型模拟反馈。

## 2026-09-13 · 设置默认分区统一为通用

### 修法

- 直接进入 `#settings` 时默认显示「通用」分区，不再跳到「关于」。
- 通过参数或其他入口指定分区时仍按指定分区打开。

### 实测

- 直接进入设置路由后，激活 Tab 为「通用」，激活面板为 `settings-general`。

## 2026-09-13 · 面板自动打开默认值调整

### 修法

- 「启动后自动打开面板」默认值从关闭调整为开启。
- 说明同步改为启动后直接载入官方面板，便于查看代理状态。

### 实测

- 该开关默认 `aria-checked=true`，视觉状态为开启。

## 2026-09-13 · 界面缩放改为滑块

### 修法

- 设置 → 通用 → 桌面壳偏好中「界面缩放」由固定枚举改为滑块。
- 范围为 50%–200%，步进 1%，默认 100%。
- 外壳与官方面板继续使用同一缩放因子；面板内临时 +/− 也会同步滑块与读数。

### 实测

- 滑块默认值 100%，读数 100%。
- 面板 +/− 后滑块同步变化；`node --check` 通过。

## 2026-09-13 · 界面缩放支持数值输入

### 修法

- 滑块右侧读数改为可编辑数值输入框，样式与设置输入框保持一致。
- 输入范围仍为 50%–200%；输入时实时同步缩放，失焦时规范化数值。
- 滑块、数值框与面板内 +/− 保持双向同步。

### 实测

- 数值框输入 `150` 后滑块与面板缩放同步为 150%。
- 数值框输入超范围值后，失焦被规范到最近有效值；`node --check` 通过。
