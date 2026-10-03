# 通知体系回写与 UI 规范化 · 执行记录

> 编写日期：2026-09-19（本机 Asia/Shanghai）
> 依据：`02-开发实施/01-方案资料/通知体系回写与UI规范化执行计划.md`
> 性质：过程核对记录，不是验收证书，也不维护第二套实时任务状态。正式 TASK / RCP / 检查结果见 `.adg/`。

## 0. 开工基线核对

| 项 | 记录值 |
|---|---|
| 仓库 HEAD | `d1858a7`（`codex/implementation-base`） |
| 原型文件 | `01-需求分析/05-原型/原型/index.html` |
| 原型 SHA-256 | `15887731a4b3285c1667828a2aef321ba22514b9f92f343b33ab0a539a098c09` |
| 与计划记录一致性 | ✅ 与计划 §2 记录值一致，原型未发生漂移 |
| 工作树状态 | 存在前序对话未提交改动（`src-ui/*`、`src-tauri/*`、原型 `index.html`、原型 `README.md` 等）；本轮按修改范围定点阅读并保留 |

结论：原型版本可定位；计划 §2 记录的 SHA-256 与当前一致，无差异需要先核对。

## 1. 原型区块 → 依据 → 现行文档 → 生产实现 → 差异

对照口径：`原型/index.html`（冻结基线）→ 用户确认依据（原型 README / 审计回执 / 方案草案）→ 现状。

| # | 原型区块 / 选择器 | 用户确认依据 | 现行文档承载 | 生产实现 | 差异 |
|---|---|---|---|---|---|
| 1 | 全路由统一外壳底面 `.window{background:var(--glass-rail)}`，`.side`/`.titlebar` 透明，内容 `.main` 是圆角页 | README 7.13 | `UI规范.md`（本轮新增） | `base.css` `.titlebar`/`.side` 各自 `--glass-rail`+blur；`.main` 无圆角、无左/顶描边 | **差异**：生产仍是「两块玻璃」，未做「一整块底面 + 圆角内容页」 |
| 2 | 宽栏 244px / 面板窄栏 64px，品牌与菜单图标列对齐不跳动 | README 7.15 | `UI规范.md` | `.desktop-stage --sidebar-width`；`panel-mode` 用 `justify-content:center` 居中 | **差异**：生产窄栏居中，会跳动；`@media(max-width:1100px)` 还会自动收窄 |
| 3 | 通知中心默认展示全部存活通知，无「未读/未解决/全部历史」筛选行 | README（2026-09-19） | `数据与状态.md`（本轮）、`UI规范.md` | `AppTopbar` 通知面板按 `filterByCategory(controller.unreadList)` 只显示**未读** | **差异**：生产只列未读，未使用存活语义 |
| 4 | 分类 tab 透明底、自然宽度、底部指示线；计数徽标中性半透明 | README 7.6 / 7.7 | `UI规范.md` | `.notification-tabs`（未见同款令牌） | **差异**：生产 tab 无中性计数片与同款指示线规则 |
| 5 | 条目为半透明卡片，垃圾桶垂直居中；状态标签行内小标签 | README 7.4 / 7.5 | `UI规范.md` | `.notification-item` 中删除键用「×」图标、非垃圾桶；无 `resolved` 行内标签 | **差异** |
| 6 | `read`/`resolved`/`deleted` 三态；已读 ≠ 已解决 | 审计回执 §1 P1 | `数据与状态.md`（本轮新增通知状态）、`领域模型.md` §16 | `types/ui.ts NotificationItem` 只有 `read`；`filterByCategory` 无 `resolved/deleted` | **差异**：生产缺 `resolved`/`deleted` 维度 |
| 7 | Toast / 任务卡 / 通知中心右上角「收起到铃铛」；收起不改状态、不取消任务 | README（2026-09-19） | `UI规范.md`、`数据与状态.md` | `AppModal` 任务卡仅有「后台运行/关闭」，无收起到铃铛；Toast 无收起键 | **差异** |
| 8 | 通知内控件「半透明 + 小一档」，取消类半透明红 | README 7.1 | `UI规范.md` | `process-progress` 用 `.btn ghost` 主控制档 | **差异** |
| 9 | 面板快捷浮层用 `--glass-fill` + `--glass-blur-notif` | README 7.14 | `UI规范.md` | `.panel-hub-toggle` 用 `color-mix(--panel 94%)` + `blur(18px)` | **差异** |
| 10 | 覆盖式滚动条（不常驻） | README 7.10 | `UI规范.md` | `base.css` 未见 `.is-scrolling` 覆盖式滚动条 | **差异** |
| 11 | 六类渠道由同一套真实组件演出；宿主是全局壳层 | README（通知专题） | `UI规范.md` | 无独立 `ToastHost/TaskProgressHost/NotificationCenter` 组件，宿主逻辑散在 `App.vue`/`AppTopbar`/`AppModal` | **差异（结构）** |
| 12 | 面板路由窄栏内容为圆角页、`.panel-shell/.panel-frame` 底色 `--panel` | README 7.9 | `UI规范.md` | `base.css` `.panel-shell{background:var(--bg)}`、`.panel-frame{background:var(--bg)}` | **差异** |

> 说明：上表只登记**差异**与确认项，不把候选值写成冻结结论（见 §3）。

## 2. 前次审计项定点复核（对当前 HTML，不采信回执结论）

| 审计项 | 当前 HTML 证据 | 结论 |
|---|---|---|
| 初始未读数量一致 | `notifications` 每条显式 `read/resolved/deleted`；`unreadCount()=live().filter(!read&&!resolved)`；角标/中心/历史同源（`index.html:4047-4120`） | ✅ 已修且当前成立 |
| 任务卡接入正常操作 | `showTask({op:'op_start'…})` 由启动/停止/重启/同步触发（`index.html:4353-4369, 3959`） | ✅ 已修 |
| 任务/Toast 动作按钮可操作 | 动作绑定 mock 反馈（审计回执 §1 P0） | ✅ 已修 |
| 已读 ≠ 已解决 | 引入 `read/resolved/deleted` 三态；条目标签 `已解决/未解决`（`index.html:4080-4081`） | ✅ 已修 |
| 清理文案与软删除 | `openItems` 存在时改文案 + 二次确认；清理走软删除 `deleted=true`（`index.html:4020-4047`） | ✅ 已修 |
| Modal 焦点与 Escape | 审计回执 §1 P1 记录「Escape=取消、遮罩=取消、焦点移入/归还」 | ✅ 已修 |
| 「查看日志」进入正确 Tab | 审计回执 §1 P1 记录 `setRoute('logs')`+`activateDiagTab('logs')` | ✅ 已修 |

> 结论：前次审计项在当前原型 HTML 中均成立；原型侧无回归。生产侧对应缺口见 §1（尤其 #3/#6/#7）。

## 3. 明确候选（未定稿，不得写成冻结）

- Toast / 任务区「窗口内顶部」与「右上」的最终位置。
- 通知材质（纯透明 / 轻磨砂 / 实底）与颜色滤镜（冷色 / 无色）的打样结论。
- 系统通知的开关、权限与触达范围；哪些失败进入持久通知、终态停留时间、日志关联方式。
- 原型最小窗口（900×600）与 Tauri 配置最小窗口不一致时的双端验证尺寸。

## 4. 实现缺陷（原型已确认、生产缺失）

见 §1 差异列；按计划阶段 1–6 逐步回写契约并在生产实现，不用局部 CSS 或错误成功提示掩盖。

## 5. 本轮写入范围（结果落点）

- 权威长期事实：`docs/02-项目核心/领域模型.md` §16、`docs/02-项目核心/数据与状态.md`、`docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md`（FZ-43）。
- 新增规范：`docs/02-项目核心/UI规范.md`（登记到 `docs/02-项目核心/README.md`）。
- 派生矩阵：协作包 `01-需求分析/04-需求分析产物/场景与验收矩阵.md`（关联既有 AC，不自建权威）。
- 生产实现：`src-ui/**`（按阶段 3–4）。

> 复核结论以当前文件为准；候选不得冒认通过；不伪造 TASK/RCP/CERT。

---

## 6. 执行结果（2026-09-19，TASK-OPENCODEX-DESKTOP-61）

### 6.1 阶段 1 文档回写清单

| 文档 | 变更 | 状态 |
|---|---|---|
| `docs/02-项目核心/领域模型.md` §16 | 通知实体补齐 `category` / `source` / `resolved` / `resolved_at` / `deleted` / `expires_at` / `operation_id` / `dedupe_key`；新增 §16.1「与操作任务、技术日志、确认请求的边界」 | 已完成 |
| `docs/02-项目核心/数据与状态.md` §5 | 新增「反馈与通知状态」：六类渠道职责、操作进度与真实终态、通知三维度、过期与清理、去重排队触达、收起状态、持久化与恢复 | 已完成 |
| `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` FZ-43 | 就地修订并扩为 FZ-43.1~43.4（渠道/时机、触达/去重/排队、关闭/超时/清理、跨路由与承载）；新增变更记录行 | 已完成 |
| 协作包 `04-需求分析产物/场景与验收矩阵.md` §7 | 新增 S-11~S-16 六类渠道场景、组合边界场景与测试数据；关联既有 AC，未新建需求级 AC | 已完成 |
| `docs/02-项目核心/UI规范.md` | 新增；登记到 `docs/02-项目核心/README.md` | 已完成 |
| DMD | 未受影响（无范围/验收变更） | 无变更 |

一致性检查：§16、§5、FZ-43、验收矩阵口径一致；现有 AC/FZ 编号未复用成另一语义；候选（材质、窗口内位置、系统通知范围、最小窗口尺寸）未写成冻结。

### 6.2 阶段 2 UI 规范

`docs/02-项目核心/UI规范.md` 覆盖窗口与布局、设计令牌（含通知表面与控件令牌的亮/暗值）、基础控件、通知组件、操作规则、键盘与可访问性、边界，并给出「原型选择器 → token → 组件」映射与迁移状态。固定尺寸注明为 CSS 逻辑像素。

### 6.3 阶段 3 组件化与样式规范化（本轮范围）

| 层 | 落地 |
|---|---|
| 基础 UI | 复用既有 `.btn` / `.icon-btn` / tab 等；通知内控件新增 `--notif-ctl-h` 家族 |
| 反馈展示 | 新增 `NotificationCenter.vue`、`NotificationItem.vue`、`ToastHost.vue`、`TaskProgressHost.vue`；`AppModal.vue` 收敛为确认/详情 Dialog，长任务卡移出 |
| 业务状态 | 通知仍是单一事件来源（后端 `NotificationStore`）；前端 `notifications.ts` 只做投影，不复制数据 |
| 样式 | `tokens.css` 收敛公共设计值（新增通知令牌 + reduced-motion）；`base.css` 承载布局/控件/反馈展示，清除 `.notification-tabs` 旧规则 |

跨边界契约变更（Rust ↔ TS）：`Notification` 与 DTO 增加 `resolved` / `resolved_at` / `operation_id` / `dedupe_key`；聚合增加 `unresolved`；新增命令 `mark_notification_resolved`、`clear_resolved_notifications`；`delete` 改为软删除，`push` 支持 `dedupe_key` 去重，`all()` 保留底层、新增 `live()` 投影。

### 6.4 阶段 4 原型复刻进度

| 区块 | 状态 |
|---|---|
| 壳层：单一底面 + 圆角内容页 + 无分栏竖线 | 已复刻（`base.css` `.app-window/.side/.titlebar/.main`） |
| 宽窄栏图标列对齐（左对齐、不居中） | 已复刻（`.app-shell.panel-mode` 与 `@media` 分支） |
| 覆盖式滚动条（不常驻） | 已复刻（`base.css` + `main.ts` 捕获阶段 scroll 打点） |
| 通知中心（存活通知、分类 tab、中性计数、垃圾桶、行内状态标签） | 已复刻 |
| Toast / 任务卡 / 收起到铃铛 | 已复刻（共用通知玻璃底板） |
| 面板浮层玻璃、其余路由与设置分区 | **未迁移**，见 §6.7 缺口 |

### 6.5 阶段 5 Web 审计（本次证据）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过（`vue-tsc --noEmit` 无输出） |
| `npm --prefix src-ui test -- --run` | 通过：27 文件 / 101 测试 |
| `npm --prefix src-ui run build` | 通过（`dist/` 产出，仅有既有 `INEFFECTIVE_DYNAMIC_IMPORT` 提示） |

场景证据：`tests/notification-center.test.ts`（默认显示全部存活通知、已读/已解决行内标签、中性分类计数、收起到铃铛不改状态）；`tests/notifications.test.ts`（读/解决/删除/清理投影与命令契约）。

### 6.6 阶段 6 桌面审计（`cargo` 契约门）

| 命令 | 结果 |
|---|---|
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | 通过 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 通过 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 通过：lib 245 + 集成各套件全绿 |

环境事实：`tauri-cli 2.11.4`；`devUrl http://localhost:5173`；`beforeDevCommand npm --prefix ../src-ui run dev`；主窗口 `1180×760`，最小 `960×640`。

**未执行**：启动 Tauri 开发版进行原生红黄绿、子 WebView 合成、真实 IPC 异步链路与重启持久性的实机视觉验收。原因：需要有人在场的交互式会话与实机观察，本轮不可在无人值守下把启动动作冒充为通过；按计划保留为单独门禁项。

### 6.7 缺口与遗留项

1. 面板路由浮层（快捷球、弹出菜单、缩放读数）尚未改用 `--glass-fill` + `--glass-blur-notif`。
2. 概览、日志/诊断、设置各分区的原型复刻未在本轮完成。
3. 概览 spinner / 进度收起 / 失败重试的面板加载提示尚未全部改用真实状态。
4. 系统通知权限与触达范围、通知材质与窗口内位置仍未定稿（候选）。
5. 原型最小窗口 900×600 与 Tauri 配置 960×640 不一致，需双端验证尺寸后统一。
6. 桌面实机审计（§6.6）未执行。

### 6.8 未提交改动保留说明

本轮未执行提交、建分支、发布或全局同步。开始前工作树已有的未提交改动（含 `src-ui`、`src-tauri`、原型 `index.html` 与说明）按范围定点阅读后保留；本轮在同一工作树上继续修改，没有回退他人改动。

---

## 7. 续作：原型复刻深化与 Web 视觉证据（2026-09-19 第二段）

> 说明：TASK-61 已原子收口。本段只改 `src-ui` 行为与样式、新增开发期审计夹具并采集证据，**未改动权威长期事实文档**，因此未新建 TASK。

### 7.1 修正的实现缺陷

| # | 缺陷 | 处理 |
|---|---|---|
| 1 | 概览「最近事件」使用原型固定样例（固定时间、`10:23:52 启动失败：端口 10100 被占用` 等伪造结论） | 改为只登记**本机真实观测到的运行状态迁移**（`app.setStatusSnapshot` → `recordEvent`）；无观测时显示空态 |
| 2 | `runtimeScenarios` 默认 `port: '10100'` / `pid: '39421'`、失败态 note 断言未观测到的原因 | 占位改为 `—`，失败文案改为「以官方输出与日志为准」；真实值仍由 `statusScenario` 用快照覆盖 |
| 3 | 概览数据目录/`OPENCODEX_HOME` 回退到原型示例路径 `~/OpenCodexData*` | 回退改为占位 `—` |
| 4 | 顶栏按视口宽度（≤1100px）自动把侧栏收成 64px，冻结原型无此行为 | 删除该媒体查询；窄栏只由面板路由驱动；900×600 下侧栏保持 244px |
| 5 | 窗口缺左/顶 1px 描边，图标列落在 14/24px | 四边补 1px 描边；图标列回到原型的 15/25px（宽窄栏一致） |
| 6 | 诊断中心「清理通知」直接清空全部通知 | 改为默认只清已解决；存在未处理通知时改标题与文案并二次确认（`FZ-43.3`） |
| 7 | 诊断中心通知历史与通知中心视觉/结构不一致 | 历史复用 `NotificationItem`（新增 `show-time`），并显示「共 N 条 · 未读 X · 未解决 Y」 |
| 8 | 托盘预览写死 `127.0.0.1:10100` | 改为按真实快照显示端口，未运行时显示「未运行」 |
| 9 | 生产源码残留未使用的原型固定数据（`normalLogs` / `riskLogs` / `errorLogs` / `offlineLogs` / `doctorReport` / `initialNotifications`） | 从 `data/mock.ts` 删除，避免被误用为伪造日志或伪造通知 |
| 10 | 「标记已解决」只存在于数据层，详情弹窗无法操作 | `ModalState` 增加 `resolveLabel` / `onResolve`，详情弹窗可标记已解决 |
| 11 | 通知中心面板与 Toast / 任务卡锚定到同一右上坐标带，展开时互相压盖 | 三者收进全局 `.notif-layer`（列向 flex），通知中心改为同列的相对定位块；避让由正常流保证，不再依赖 JS 位移 |

### 7.2 新增：开发期审计夹具（`src-ui/src/dev/auditHarness.ts`）

- 只在 `import.meta.env.DEV` 且 URL 带 `?__audit=1` 时安装；**正式构建已确认不含入口**（`dist/assets/*.js` 中 `__audit` 计数为 0）。
- 提供可重复的场景参数：`theme` / `route` / `tab` / `panel` / `toast` / `task` / `report`；`report=1` 会把计算样式与结构指标写入 `#audit-report`，供无头浏览器 `--dump-dom` 取回。
- 不连接真实写入端点，也不把真实 IPC 失败当成 mock 成功。

### 7.3 Web 审计证据（无头 Chromium，`chrome-headless-shell`）

证据目录：`.adg/work/evidence/task61-web-audit/`（每张图配套一份同名 `.json` 计算样式报告）。

| 场景 | 视口 | 关键断言 |
|---|---|---|
| `01-overview-light` / `02-overview-dark` | 1180×760 | 外壳单底面 `rgba(249,249,249,.66)`；菜单/标题栏透明无模糊；内容页 `rgb(255,255,255)` + 顶/左 1px + 左上 12px 圆角；滚动条 `thin` + 透明拇指 |
| `03-notif-stack-dark` | 1180×760 | 通知中心 `display:block`、4 条（含 1 条已解决）、Toast 与任务卡同时存在（堆叠） |
| `04-notif-center-light` | 1180×760 | 分类 tab `background: transparent` + 底部 2px 指示条 + 顶边框 0；计数 `4/1/1/1/1`；垃圾桶图标中心偏移 `(0,0)`；标签 `未解决`/`已解决` |
| `05-panel-narrow-light` | 1180×760 | 面板路由侧栏 `64px`，图标列仍为 15/25px |
| `06-minwindow-900x600` | 900×600 | 侧栏保持 `244px`（不再按视口自动收窄），通知中心与 Toast 未溢出 |
| `07-wide-1600x1000` | 1600×1000 | 侧栏 244px、图标列 15/25px |
| `08-tray-light` / `09-settings-light` / `10-extensions-light` | 1180×900 | 各路由在当前壳层下可渲染（视觉细节待人工核对） |

通知层避让实测（`layerLayout`：`overlap` / `withinViewport`）：

| 场景 | 面板 | Toast | 任务卡 | 结论 |
|---|---|---|---|---|
| 1180×760 | `56→463` | `471→519` | `527→744` | `overlap=false`、`withinViewport=true` |
| 900×600 | `56→303` | `311→359` | `367→584` | `overlap=false`、`withinViewport=true` |

命令结果（本段复跑）：

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：27 文件 / 104 测试 |
| `npm --prefix src-ui run build` | 通过；审计夹具未进入产物 |

### 7.4 桌面实机审计（阶段 6）当前阻塞

- 已确认：`tauri-cli 2.11.4`；`devUrl http://localhost:5173`；`beforeDevCommand npm --prefix ../src-ui run dev`（以仓库根为工作目录时该相对路径不成立，需以 `src-tauri` 为工作目录或改用 `cargo run`）；主窗口 `1180×760`、最小 `960×640`。
- 已确认：以当前源码构建的 debug 二进制可以启动并进入前台进程列表，未崩溃。
- **阻塞**：`screencapture` 输出全黑，说明当前宿主未授予终端「屏幕录制」权限；`osascript` 读取窗口也因缺少辅助功能权限返回空。故原生红黄绿、子 WebView 合成、通知叠层避让与真实 IPC 异步链路的**实机视觉验收无法在本轮完成**。
- 需要用户授权（系统设置 → 隐私与安全性 → 屏幕录制/辅助功能）或由用户本人在场观察；在此之前不把启动动作冒充为视觉审计通过。

### 7.5 仍待完成

1. 概览、设置各分区、扩展页与原型逐块视觉核对（当前仅结构证据 + 截图，未做逐像素对照）。
2. 面板路由浮层的原生承载方案（`R-17`）：DOM 浮层会被原生子 WebView 遮挡，不能直接照搬原型。
3. 原型最小窗口 900×600 与 Tauri `960×640` 不一致，需双端确认后统一。
4. 系统通知权限与触达、通知材质与窗口内位置仍未定稿。

---

## 8. 续作：面板浮层原生化与原型结构对照（2026-09-19 第三段，`TASK-OPENCODEX-DESKTOP-62`）

### 8.1 官方面板快捷浮层改为原生承载

官方面板是原生子 WebView，管理器 DOM 里的浮层会被盖住（`R-17`）。本轮把浮层改为**随面板子 WebView 注入**，覆盖原型 `#panelHub` 的结构与材质：

| 项 | 旧实现 | 现实现 |
|---|---|---|
| 材质 | 写死深色 `rgba(30,30,35,.9)`，无主题 | 与通知玻璃同源：亮 `rgba(244,245,249,.52)` / 暗 `rgba(34,36,42,.56)` + `saturate(1.5) blur(8px)` |
| 主题 | 无 | 管理器下发 `PanelRequest.theme`（解析后 `light`/`dark`），未知值按 `light`；主题切换时重新下发 |
| 结构 | 悬浮球 + hover 菜单 | `38px` 快捷球 + 点击开合菜单（`172px`、圆角 `14`）+「界面 N%」+ 缩放组 `− / 读数 / +` + `概览` / `重载` / `浏览器` |
| 交互 | 仅 hover | 点击开合（`aria-expanded`）、点外部关闭、`Escape` 关闭、`:focus-visible` |
| 动作 | `overview/reload/browser/zoom-in/zoom-out` | 不变，仍走 `ocxd-panel://` 受限协议白名单 |

改动落点：`src-tauri/assets/panel-hub.js`、`src-tauri/src/commands/panel.rs`（`PanelRequest.theme` + `panel_state_detail`）、`src-ui/src/commands/panel.ts`、`src-ui/src/routes/PanelRoute.vue`。

### 8.2 原型 vs 生产：计算样式对照（无头 Chromium，`1180×760`）

为做客观对照，把冻结原型复制到 `/tmp` 临时副本并附加只读度量脚本（不改仓库内原型），同样用无头浏览器取计算样式。

| 指标 | 原型（窗口内） | 生产 | 结论 |
|---|---|---|---|
| 外壳底面 | `rgba(249,249,249,.66)` | `rgba(249,249,249,.66)` | 一致 |
| 菜单 / 标题栏 | 透明、`backdrop-filter:none` | 透明、`backdrop-filter:none` | 一致 |
| 内容页 | `#fff` + 顶/左 `1px` + 左上 `12px` 圆角 | 同 | 一致 |
| 滚动条 | `thin` + 透明拇指 | 同 | 一致 |
| 图标列（窗口内） | 品牌 `15px` / 菜单 `25px`（原型绝对坐标 37/47，含 22px 演示外壳偏移） | `15px` / `25px` | 一致 |
| 通知令牌 | `glass-fill .52` / `blur 8px` / `ctl 24px` / `mark #e0a53c` | 同 | 一致 |
| 设置分区 | `general…about` 共 10 个 | 同 10 个 | 一致 |
| 扩展 Tab | `skills` / `mcp` | 同 | 一致 |

> 另核对：原型托盘路由同样隐藏应用窗口、只显示托盘预览（`html[data-route="tray"] .desktop .stage{display:none}`），生产行为一致，**不是**差异。

### 8.3 其他修正

| # | 修正 |
|---|---|
| 1 | 侧栏宽度令牌由 `--sidebar-width` 统一为原型的 `--side-w`（`base.css`），`UI规范.md` 同步 |
| 2 | Rust 子 WebView 边界常量 `64 / 28` 与面板路由 CSS（`--side-w:64px` / 标题栏 `28px`）一致，未发现 CSS 与后端尺寸不一致 |
| 3 | 新增 `panel_state_detail` 单元测试：主题归一化（未知值→`light`）、缩放与提示透传 |

### 8.4 门禁结果（本段）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：27 文件 / 104 测试（新增面板主题随动用例） |
| `npm --prefix src-ui run build` | 通过 |
| `cargo fmt --check` / `cargo clippy -D warnings` | 通过 |
| `cargo test` | 通过：lib 246 + 集成套件全绿 |
| 注入浮层实测 | 亮/暗玻璃取值、`120%` 读数、开合与 `aria-expanded`、动作白名单均符合预期（`.adg/work/evidence/task61-web-audit/12-panel-hub.json`） |

### 8.5 仍待完成

1. 桌面实机审计（阶段 6）仍受屏幕录制 / 辅助功能权限阻塞。
2. 设置各分区与扩展页的逐项内容对照（当前只到结构、分区清单与壳层层级）。
3. 原型最小窗口 900×600 与 Tauri `960×640` 待统一；系统通知与材质/位置仍未定稿。

### 8.6 最小窗口双尺寸实测（不静默改动平台约束）

审计夹具新增 `overflow` 指标（文档与内容区横向/纵向溢出），在原型最小尺寸与应用最小尺寸各测一次：

| 尺寸 | 文档横向溢出 | 内容区横向溢出 | 内容区可纵向滚动 | 通知层重叠 | 是否越出视口 |
|---|---|---|---|---|---|
| `900×600`（原型最小） | `0` | `0` | 是 | 否 | 否 |
| `960×640`（Tauri 最小） | `0` | `0` | 是 | 否 | 否 |

结论：两档尺寸下均无横向溢出或浮层越界，差异目前只落在「最小可缩到多小」，**未改动平台窗口约束**；是否把 Tauri 最小尺寸对齐到原型 900×600 属待确认项。

---

## 9. 续作：设置分区与扩展页逐项内容对照（2026-09-19 第四段）

方法：把冻结原型复制到 `/tmp` 临时副本并附加只读脚本（不改仓库原型），分别取「原型」与「生产」在 `#settings?section=*`、`#extensions?tab=*` 下的**可见文案行集合**与标签集合，做集合差集对照。证据：`.adg/work/evidence/task61-web-audit/proto-text-*.json` 与 `prod-text-*.json`。

### 9.1 标签集合

| 项 | 原型 | 生产 | 结论 |
|---|---|---|---|
| 设置分区 | 通用 / 安装配置 / 数据与备份 / 扩展管理 / 配置迁移 / WebDAV 同步 / 日志与通知 / CLI 控制面 / 版本升级 / 关于 | 同 10 项、同顺序 | 一致 |
| 扩展分区 | Skills / MCP | Skills / MCP | 一致 |
| 诊断分区 | 日志历史 / 通知历史 | 日志历史 / 通知历史 | 一致 |

### 9.2 分区文案差集

| 分区 | 原型行数 | 生产行数 | 差异性质 |
|---|---|---|---|
| 通用 | 26 | 26 | 表述收尾差异（生产省略「保持 1:1 CSS 像素」等补语）；无字段增减 |
| 安装配置 | 26 | 20 | 原型写死 `~/OpenCodexData` / `~/OpenCodexData/opencodex-home`；生产按真实配置显示，未加载时为「正在读取数据目录配置…」——**原型固定数据未复制** |
| 数据与备份 | 15 | 15 | 无差异 |
| 扩展管理 | 60 | 35 | 原型写死 `~/.agents/skills` 与更长的说明句；生产按真实值显示（未加载为 `—`）并精简文案 |
| 配置迁移 | 14 | 18 | 生产文案更新为已冻结契约（`Argon2id 派生密钥，AES-256-GCM 认证加密`），替代原型「具体算法待安全评审后冻结」 |
| WebDAV 同步 | 19 | 19 | 仅动作文案不同（原型「查看同步历史」/ 生产「保存端点」） |
| 日志与通知 | 15 | 15 | 生产省略句尾补语（如「原始错误先保留摘要」） |
| CLI 控制面 | 4 | 4 | 无差异 |
| 版本升级 | 25 | 29 | 原型写死 `v2.50.0 · npm global`；生产显示真实探测结果（`检查中… · 未发现 npm 全局安装`）并补充签名校验说明 |
| 关于 | 35 | 35 | 原型写死 `v2.50.0` / `v0.1.0 · mock`；生产显示真实值与「未发现」——**原型固定数据未复制** |

### 9.3 扩展页视图差集

| 视图 | 差异 | 结论 |
|---|---|---|
| Skills | 原型含 6 个样例 Skill、写死 `~/.agents/skills`；生产在 Web 审计下显式提示「扩展发现失败 / 已保留上次结果」 | 差异全部来自「无 Tauri 后端」与「未复制原型固定数据」，不是结构缺失 |
| MCP | 原型含 1 个样例服务器（`node_repl`）；生产的操作集合（`导入` / `添加` / `设置` / 分页）与原型一致，数据为真实值（`已配置 0 个`） | 结构一致，差异仅在数据 |

### 9.4 结论与剩余清单

- 设置 10 个分区、扩展 2 个分区、诊断 2 个分区的**结构与标签已对齐**；文案差异均可归因于「不使用原型固定数据」或「按已冻结契约更新措辞」。
- 仍未逐项对照的部分：各分区**字段级控件与校验行为**（当前对照到标签与可见文案行）；原型扩展页 MCP 视图**不支持 URL 直达**，需交互切换，本轮已通过脚本点击补齐。
- 无新增 authoritative 事实变更，故本段不新建 TASK。

---

## 10. 续作：设置分区字段级控件对照（2026-09-19 第五段）

方法同上（临时原型副本 + 只读脚本）；本段把对照从「可见文案」下沉到**逐行控件**：标题、控件类型（`switch` / `select` / `range` / `input` / `buttons`）、`select` 选项集合、`range` 取值范围。证据：`proto-text-*.json` / `prod-text-settings-*.json` 的 `settingsControls`。

### 10.1 逐行控件对照结果

| 分区 | 原型行数 | 生产行数 | 行级（标题+类型）一致 | 说明 |
|---|---|---|---|---|
| 通用 | 8 | 8 | ✅ 完全一致 | 含 `range` 取值范围 `50–200`、`select` 选项 `['内嵌优先','浏览器兜底']` 逐项相同 |
| 数据与备份 | 5 | 5 | ✅ | — |
| 扩展管理 | 9 | 9 | ✅ | — |
| WebDAV 同步 | 3 | 3 | ✅ | — |
| 日志与通知 | 4 | 4 | ✅ | — |
| 安装配置 / 关于 | 0 | 0 | ✅ | 两处都用表格（`.data-root-table` / `.install-grid`）承载，没有 `setting-row` |
| 配置迁移 | 3 | 5 | ➖ | 生产多出 `容器口令` / `导入口令` 两个 `input`，对应 `AC-06` 的口令输入与标准输入约束（生产领先，非缺失） |
| CLI 控制面 | 3 | 1 | ➖ | 生产默认 `cliEnabled=false`，启用前只显示开关；`AC-10` 要求「默认关闭、启用后才注册或暴露入口」，**行为正确** |
| 版本升级 | 7 | 8 | ➖ | 原型「可用更新」行在生产拆为「更新结果」「安装完成后」，源于真实探测结果与签名校验说明 |

> 通用分区实测样例（原型 vs 生产逐行相同）：界面缩放 `range 50–200`；启动时打开主界面 / 启动后自动打开面板 / 关闭窗口后保持代理运行 / 启停结果通知 / 同步冲突提醒 / 随 Codex 启动 OpenCodex（官方共享）均为 `switch`；面板打开方式 `select ['内嵌优先','浏览器兜底']`。

### 10.2 本段结论

- 已确认「各路由与设置分区」在**结构、标签、逐行控件与选项**三个层级均已对齐；剩余差异全部可归因于 `AC-06`（口令输入）、`AC-10`（CLI 默认关闭）与「不使用原型固定数据」。
- 尚未覆盖：输入类控件的**校验行为**（非法值/越界时的提示）与键盘可达性细节；这些在单测层面已有交集（`preferences.test.ts` 等），但未做原型—生产行为级对照。

---

## 11. 续作：Web 审计场景适配与残余原型文案清理（2026-09-19 第六段）

### 11.1 显式场景适配（计划 §8.1.3 要求）

审计夹具新增 `&scenario=` 预设，覆盖计划要求的**成功 / 失败 / 慢响应 / 超时 / 取消 / 乱序 / 重复 / 空数据 / 长文本**；全部只改前端展示状态，不写真实资源。

| 场景 | 断言（实测） |
|---|---|
| `empty` | 面板空态「当前分类没有通知。」 |
| `longtext` | 超长标题 + 超长正文 + 超长 Toast；`docHorizontalOverflow=0`、`mainHorizontalOverflow=0`（换行收敛，不撑宽） |
| `duplicate` | 同 `dedupeKey` 两条由后端投影而来，展示层只渲染不崩溃（去重由 `NotificationStore.push` 单测覆盖） |
| `late` | 旧操作晚到终态只记录事实：任务卡「重启已完成」+ Toast 明示「旧操作晚到终态，仅记录事实」 |
| `task-busy` | 「正在启动 OpenCodex」+ 步骤 `done / active / active` |
| `task-success` | 「启动已完成」+「服务已进入可用状态」 |
| `task-failure` | 「启动未完成」+ 失败原因；步骤转 `pending` |
| `task-timeout` | 「启动未完成」+「等待时间较长，后台仍会继续观察状态」（等待超时**不写成失败**） |
| `task-cancelled` | 「停止未完成」+「停止请求失败；已保留当前状态。」 |

全部场景：`overlap=false`、`withinViewport=true`、横向溢出 `0`。证据：`sc-*.json` / `sc-*.png`。

### 11.2 残余原型文案与死代码清理

| 位置 | 旧文案 | 新文案 | 理由 |
|---|---|---|---|
| `components/EnvironmentGate.vue` | 「原型模拟：已复制 …；不执行命令。」/「原型拦截：不会执行 …。」 | 「已复制 …；不会代你执行，请手动运行。」/「复制失败；不会执行 …，请手动运行。」/「剪贴板不可用；不会执行 …」 | 复制是真实动作，不是模拟；不代执行是真实边界。新增剪贴板不可用分支，不再谎称已复制 |
| `composables/useAppController.ts` | 「环境检查暂不可用；原型状态保留。」 | 「环境检查暂不可用；已保留当前状态。」 | 正式界面不出现「原型」 |
| `routes/TrayRoute.vue` | 「托盘原型 · 状态菜单」 | 「托盘菜单预览」 | 同上 |
| `stores/routes.ts` | 「系统托盘状态菜单的跨平台原型」 | 「…的跨平台样式预览」 | 同上 |
| `composables/mockAction.ts` | 未使用的「原型模拟」helper | 已删除 | 避免被误用为伪造成功 |

新增回归：`tests/environment-gate.test.ts` 断言复制路径不出现「原型 / 模拟」字样，且剪贴板不可用时不谎称「已复制」。

---

## 12. 续作：确认弹窗键盘与可访问性（2026-09-19 第七段）

问题：`UI规范.md` §6 已规定「`Escape` = 取消；点击遮罩只允许取消，不得确认；打开时焦点移入，关闭后归还来源焦点；Tab 焦点约束在弹层内循环」，但生产 `AppModal.vue` 此前**没有任何键盘或焦点处理**（`grep` 全仓测试无 `Escape` / 焦点用例）。规范已存在，属实现缺陷，不新增权威事实。

### 12.1 改动落点

| 文件 | 变更 |
|---|---|
| `src-ui/src/components/AppModal.vue` | `role="dialog"` + `aria-modal="true"` + `aria-labelledby`（`useId` 关联标题）；`tabindex="-1"`；`Escape` → 取消（`window` 级监听，仅在弹窗打开时生效）；点击遮罩本体 → 取消，点击对话框内部不取消；打开时记录来源焦点并把焦点移入（优先主操作，其次首个可聚焦控件，最后对话框自身），关闭后归还来源焦点；`Tab` / `Shift+Tab` 在弹层内循环 |
| `src-ui/src/styles/base.css` | 新增 `.btn:focus-visible`（对应 `UI规范.md` §4 Button 的 `:focus-visible` 状态）；`.modal:focus { outline: none }`（容器获得焦点但不显示轮廓框） |
| `src-ui/tests/app-modal.test.ts`（新增） | 断言：语义属性与标题关联、`Escape` 取消且不确认、无弹窗时 `Escape` 无副作用、遮罩点击取消而弹窗内点击不取消、焦点移入与归还、`Tab` / `Shift+Tab` 焦点循环 |

### 12.2 关键行为

- `Escape` 与遮罩点击都只能触发「取消」，**不会**触发「确认」；关闭对话框不改变业务结果（与 §16/§5「关闭详情不修改业务结果」一致）。
- 焦点顺序覆盖「来源焦点 → 弹窗主操作 → 关闭后归还来源」。

### 12.3 门禁结果（本段）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：28 文件 / 113 测试（新增 `app-modal.test.ts` 6 例） |
| `npm --prefix src-ui run build` | 通过；正式产物仍未包含审计夹具（`__audit` 计数 0） |

### 12.4 仍待完成

1. 输入类控件的**校验行为**（非法值 / 越界提示）原型—生产级对照（承接 §10.2）。
2. 设置各分区与扩展页的逐像素视觉对照（当前到结构、标签、逐行控件与选项）。
3. 桌面实机审计（阶段 6）仍受屏幕录制 / 辅助功能权限阻塞。
4. 系统通知权限与触达、通知材质与窗口内位置、最小窗口尺寸差异仍未定稿（候选）。

---

## 13. 续作：输入控件校验行为对照（2026-09-19 第八段）

承接 §10.2 的遗留项：把对照从「逐行控件与选项」下沉到**输入类控件的校验/归一化行为**。本段只改 `src-ui` 与开发期审计夹具，未改动权威长期事实文档，故不新建 TASK。

### 13.1 冻结原型里唯一带校验语义的输入：界面缩放

prototype 只有 `#interfaceScaleReadout`（数字读数）与 `#interfaceScaleRange`（滑块）承载校验语义；其余输入（WebDAV 固定值、MCP / Skills 弹窗）是原型演示态，本身没有校验规则，计划 §7 明确禁止复制。

原型规则（`原型/index.html` `applyInterfaceScale`）：

```js
const value = Math.min(200, Math.max(50, Math.round(Number(scale) || 100)));
document.documentElement.style.setProperty('--panel-zoom', String(value / 100));
…
settingReadout.value = String(value);   // 归一化结果回写到读数
slider.value = String(value);           // 同步滑块
```

### 13.2 发现并修复的差异

| # | 差异 | 影响 | 处理 |
|---|---|---|---|
| 1 | 生产读数越界/清空后**不回写归一化结果**：`@change` 只调 `setScale`，Vue 不会在模型值未变化时改写 `input.value`，读数会停留在 `999` 或空串 | 读数与实际生效取值不一致 | 新增 `commitScale`：保存分支落地后以生效值回写读数 |
| 2 | `--panel-zoom` 只在取值变化时写入，等于当前值时提前 return，镜像停留在旧值/未设置 | 原型始终写入 `value/100`；DOM 取值与原型不一致 | 归一化基准值改由 `watch(preferences.interfaceScale)` 驱动（含加载、保存成功、还原默认值），始终等于生效偏好 |
| 3 | 面板缩放的夹取规则在 `SettingsRoute` 与 `PanelRoute` 各写一份 | 两处边界可能漂移 | 抽出 `src-ui/src/scale.ts` 的 `clampScale`，两处共用，并加单测 |
| 4 | 保存失败时读数可能显示一个并未生效的数字 | 「不用错误成功提示掩盖问题」 | 保存失败也回写到**当前生效值**，不显示幻值 |

### 13.3 对照结果（无头 Chromium + playwright-core，`1180×760`）

probe：把数字读数设为 `raw` 并派发 `change`，回读归一化后的读数、滑块值与 `--panel-zoom`。

| raw | 原型 | 生产（保存成功分支） | 结论 |
|---|---|---|---|
| `999` | `200` / 滑块 `200` / zoom `2` | `200` / `200` / `2` | 一致 |
| `""` | `100` / `100` / `1` | `100` / `100` / `1` | 一致 |
| `abc` | `100` / `100` / `1` | `100` / `100` / `1` | 一致 |
| `49` | `50` / `50` / `0.5` | `50` / `50` / `0.5` | 一致 |
| `120.6` | `121` / `121` / `1.21` | `121` / `121` / `1.21` | 一致 |
| `1e9` | `200` / `200` / `2` | `200` / `200` / `2` | 一致 |

Web 审计没有 Tauri 后端，保存必然失败；为可重复观察「保存成功」分支，审计夹具新增显式本地偏好适配 `&prefs=local`（只改内存状态，不连真实端点、不写盘）。保存失败分支实测：读数回写到当前生效值 `100`，滑块与 zoom 保持 `100` / `1`，不出现幻值。

证据：`.adg/work/evidence/task61-web-audit/14-scale-proto.json`、`14-scale-prod-save-ok.json`、`14-scale-prod-save-fail.json`。

### 13.4 保留的有意差异

- 数字读数在原型里 `input` 事件即生效（逐字生效，输入 `150` 会先跳到 `50`），生产只在 `change` 生效：生产避免逐字跳变，属有意保留；归一化与回写规则完全一致。

### 13.5 改动落点

| 文件 | 变更 |
|---|---|
| `src-ui/src/scale.ts`（新增） | `clampScale`：clamp 50–200、四舍五入、空值/不可解析回退 100 |
| `src-ui/src/routes/SettingsRoute.vue` | `setScale` 归一化并 await 保存；新增 `commitScale`；`--panel-zoom` 镜像改为随生效偏好写入；`persist` 返回保存结果 |
| `src-ui/src/routes/PanelRoute.vue` | 面板缩放改用共享 `clampScale` |
| `src-ui/src/dev/auditHarness.ts` | 新增 `&prefs=local` 本地偏好适配与 `scale` 探针（异步报告，含读数/滑块/zoom） |
| `src-ui/tests/scale.test.ts`（新增） | 归一化边界单测（含 `0` 回退 `100`、`1e9` 夹到 `200`） |

### 13.6 门禁结果（本段）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：29 文件 / 117 测试 |
| `npm --prefix src-ui run build` | 通过；正式产物不含审计夹具（`__audit` 计数 0） |
| `cargo fmt --check` | 通过 |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| `cargo test` | 通过：lib 246 + 集成套件全绿（本段未改 Rust） |

### 13.7 仍待完成

1. 桌面实机审计（阶段 6）仍受屏幕录制 / 辅助功能权限阻塞。
2. 设置各分区与扩展页的**逐像素**视觉对照（当前到结构、标签、逐行控件、选项与输入校验）。
3. 系统通知权限与触达、通知材质与窗口内位置、最小窗口尺寸差异仍未定稿（候选，需用户确认）。

---

> **本节已过期**：§14 写于阶段 5 / 6 早期，其后 §15–§24 改变了多项事实（屏幕权限判定、最小窗口候选、测试计数，以及 §20–§24 的 5 处契约 / 实现缺口与 §18 的构建钩子缺陷）。**现状以 §25 为准**，本节保留为历史。

## 14. 阶段 0–7 交付说明（2026-09-19）

> 本节是阶段 7 的收口说明：只登记实际结果、证据位置与缺口，不冒充验收证书。

### 14.1 文档回写清单与相互引用

| 文档 | 本轮回写内容 | 相互引用 |
|---|---|---|
| `docs/02-项目核心/领域模型.md` §16（+§16.1） | 通知实体、字段语义与「操作任务 / 技术日志 / 确认请求」的边界 | 状态迁移引用《数据与状态》，触达规则引用 FZ-43 |
| `docs/02-项目核心/数据与状态.md` §5 | 六类渠道职责、操作进度与真实终态、读/解决/删除/过期维度、去重排队、收起、持久化 | 不复制实体字段表与视觉尺寸 |
| `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` FZ-43.1~43.4 | 渠道与时机、触达/去重/排队、关闭/超时/清理、跨路由与承载；附变更记录行 | 就地修订，未另设冲突版本 |
| 协作包 `01-需求分析/04-需求分析产物/场景与验收矩阵.md` §7 | S-11~S-16 六类渠道与组合边界场景 | 关联既有 AC，未新建需求级 AC |
| `docs/02-项目核心/UI规范.md`（新增）+ `README.md` 登记 | 窗口与布局、设计令牌、基础控件、通知组件、操作规则、键盘与可访问性、边界，及「原型选择器 → token → 组件」映射 | 视觉参数唯一维护处 |
| `docs/02-项目核心/契约字段.md` | 未受影响（本轮 DTO 字段在领域模型 §16 与实现中同步说明） | — |
| DMD | 未受影响（无范围/验收变更） | — |

一致性检查：§16、§5、FZ-43、验收矩阵口径一致；既有编号未被复用成另一语义；候选未写成冻结。

### 14.2 UI 规范与原型—token—组件映射

`docs/02-项目核心/UI规范.md` 为唯一视觉参数维护处；映射表覆盖外壳、侧栏、标题栏、内容页、滚动条、通知令牌、通知中心/条目/Toast/任务卡/对话框、面板浮层。已实现部分在设计令牌、计算样式与原型逐项一致（§8.2、§9、§10）。

### 14.3 组件化与 CSS 整理范围

- 新增 `NotificationCenter.vue`、`NotificationItem.vue`、`ToastHost.vue`、`TaskProgressHost.vue`、`BrandMark.vue`；`AppModal.vue` 收敛为确认/详情对话框。
- `tokens.css` 收敛公共设计值（含通知令牌与 reduced-motion）；`base.css` 收敛重置、布局、控件与反馈展示；清除 `.notification-tabs` 旧覆盖规则与广域选择器误伤。
- 通知仍是单一事件来源（后端 `NotificationStore`），前端只做投影；无新增隐式旁路接口，无整页重写。
- 新增 `src-ui/src/scale.ts` 共享界面缩放归一化，消除设置页与面板路由的重复边界。

### 14.4 原型复刻完成 / 缺口清单

| 区块 | 状态 |
|---|---|
| 壳层（单一底面、圆角内容页、无分栏竖线、覆盖式滚动条） | 完成 |
| 宽窄栏图标列对齐（15/25px，不跳动） | 完成 |
| 通知中心 / 条目 / Toast / 任务卡 / 收起到铃铛 | 完成 |
| 面板快捷浮层（原生注入、玻璃材质、主题随动） | 完成 |
| 设置 10 分区、扩展 2 分区、诊断 2 分区：结构、标签、逐行控件、选项、输入校验 | 完成（差异均可归因于 AC-06 / AC-10 与「不使用原型固定数据」） |
| 设置各分区与扩展页的**逐像素**视觉对照 | 缺口（当前到结构、标签、控件、选项、校验行为） |
| 托盘预览、日志/诊断页视觉细节 | 部分完成（结构证据 + 截图，未逐像素对照） |

### 14.5 Web 与桌面结果（分别）

- **Web（阶段 5）**：`typecheck` / `test`（29 文件 / 117 测试）/ `build` 全绿；审计夹具未进入正式产物；关键场景有当次计算样式 JSON 与截图证据（`.adg/work/evidence/task61-web-audit/`，含 `01`–`14` 系列、`proto-text-*` / `prod-text-*`、`sc-*`）。Web 侧阻塞项已清零。
- **桌面（阶段 6）**：`cargo fmt` / `clippy -D warnings` / `test`（lib 246 + 集成套件）全绿，仅覆盖**契约与编译**。原生窗口、子 WebView 合成、真实 IPC 异步链路与重启持久性的**实机视觉验收未执行**：宿主未授予终端「屏幕录制 / 辅助功能」权限（`screencapture` 全黑、`osascript` 窗口查询为空）。编译通过不等于原生流程通过。

### 14.6 未提交改动保留说明

本轮未提交、未建分支、未发布、未做全局同步，也未执行高风险真实写入。开工前工作树已有的未提交改动按范围定点阅读后保留，未回退他人修改。

### 14.7 待用户确认的候选（未定稿）

1. Toast / 任务区「窗口内顶部」与「右上」的最终位置。
2. ~~通知材质（纯透明 / 轻磨砂 / 实底）与颜色滤镜打样结论。~~ **已确认**：`docs/02-项目核心/UI规范.md` §2.2 记录「中性轻磨砂 + 不叠颜色滤镜」为正式材质，材质与滤镜**不再属于候选**；§35 已把生产实现对齐并实测（本项从候选移出）。
3. 系统通知的开关、权限与触达范围；哪些失败进入持久通知、终态停留时间与日志关联方式。
4. 原型最小窗口 `900×600` 与 Tauri 配置 `960×640` 是否统一（两档实测均无溢出，未静默改动平台约束）。

### 14.8 需要的人工动作

- 授予终端「屏幕录制」与「辅助功能」权限，或由用户本人在场观察，方可完成阶段 6 实机审计。
- 对 §14.7 的四项候选给出结论，之后才能收敛到 `UI规范.md` 与 IMP（涉及权威文档的改动需单独登记 TASK）。

---

## 15. 续作：全路由逐项视觉对照与样式缺口修复（2026-09-19 第九段）

承接 §10.2 / §14.4 的遗留项「设置各分区与扩展页的逐像素视觉对照」。本段只改 `src-ui`，未改动权威长期事实文档，故不新建 TASK。

### 15.1 方法

1. 把冻结原型复制到 `/tmp` 只读副本（SHA-256 仍为 `15887731…c09`），不改仓库内原型。
2. 用无头 Chromium + playwright-core，对同一视口（`1180×900`）、同一主题，逐路由/逐设置分区取**计算样式**，控件的元素选取统一为「该选择器下第一个真实可见（`getClientRects()` 非空）的元素」，避免命中隐藏面板。
3. 另一路做**样式表级**检查：扫描 `src-ui/src/**/*.vue` 的 class 用法，找出「模板在用、但生产 CSS 里没有任何规则、且冻结原型有对应样式」的类，作为缺口的客观清单。

证据：`15-route-style-diff.json`、`15-precise-style-diff.json`、`16-class-coverage.json`、`16-missing-rules.json`、`17-dangling-classes.json`，以及 `15-route-{proto,prod}-*.png` 各路由截图。

### 15.2 检出并修复的真实缺口

| # | 缺口 | 冻结原型 | 修复前生产 | 修复 |
|---|---|---|---|---|
| 1 | **状态标签 `.tag` 没有基础规则** | `.tag{display:inline-flex;align-items:center;min-height:22px;padding:2px 8px;border:1px solid var(--border);border-radius:var(--radius-pill);background:var(--panel);color:var(--muted);font-size:var(--text-micro);font-weight:600;white-space:nowrap}` | 只有 `.skill-title .tag` 与 `.tag.danger`；页面上所有状态标签退化成 14px 纯文本（无底、无描边、无圆角） | `base.css` 补 `.tag` 基础规则 |
| 2 | **`.section-actions` 缺失** | `display:flex;justify-content:flex-end;gap:8px` | 无规则，`display:block`，按钮不右对齐、无 8px 间距 | `base.css` 补 `.section-actions` |
| 3 | **`.skills-source` 缺失** | `min-width:0;max-width:280px` | `max-width:none`，长路径不再收敛到 280px | `base.css` 补 `.skills-source` |
| 4 | **`.modal-body` 缺失** | `margin-bottom:14px` | 无规则（当前弹窗正文都是 `<p>`，外边距塌陷后暂无可观测差异，仍按原型补齐） | `base.css` 补 `.modal-body` |
| 5 | **概览模块卡多出一颗设置按钮** | 第三颗「…设置」保留在 DOM 但 `.mod-actions .btn[data-settings-section]{display:none}` 隐藏；「进入设置」由右上角 ⋯ 承担 | 三颗按钮全部可见 | 移除 `upgrade` / `migration` 卡的动作区设置按钮；把 `.mod-go` 由装饰 `span aria-hidden` 改为**真实 `<button>`**（带 `aria-label`、`:focus-visible`、hover），点击进入对应设置分区 |
| 6 | 死类名 `k-card` / `task-card` / `toast-collapsible` | 原型对应 `.toast.k-card` / `.task-card` / 无 | 模板里带了类名但生产没有任何规则，读起来像是已移植 | 删除；生产 `.toast` / `.process-progress` 本身就是对应变体 |

### 15.3 修复前后实测（计算样式）

| 指标 | 原型 | 修复前 | 修复后 |
|---|---|---|---|
| `.tag`（概览/通用/关于） | `display:flex` · `border-radius:999px` · `bg:#fff` · `border:1px` · `10px/600` · `h22` · `pad 2px 8px` | `display:block` · `radius:0` · `bg:transparent` · `border:0` · `14px/400` · `h21` · `pad 0` | **与原型逐项相同** |
| `.section-actions` | `flex` · `flex-end` · `gap:8px` | `block` · `normal` · `normal` | **`flex` · `flex-end` · `gap:8px`** |
| `.skills-source` | `max-width:280px` | `max-width:none` | **`max-width:280px`** |
| `.mod-actions .btn`（概览「检查更新」） | `width 122px` | `width 79px`（3 颗平分） | **`width 122px`**（2 颗平分，同原型可见按钮数） |
| 概览模块卡可见设置入口 | 右上角 ⋯（原型用容器 click 委托） | 无（未接） | ⋯ 为真实按钮，`aria-label="进入「版本升级」设置"`，点击后 `settingsSection==='upgrade'` |

### 15.4 判定为「非缺口」的剩余差异（附依据）

| 差异 | 依据 |
|---|---|
| 各路由首个 `button.active` / `button.btn*` / `span.tag` 的样式差 | **元素首匹配伪差**：原型把全部路由/分区渲染在同一 DOM 里，第一个可见 `button.active` 是 tab，生产是主题分段控件；按选择器精确对照（`.settings-tabs` / `.diag-tabs` / `.ext-tabs` / `.theme-seg` / `.mod-actions .btn`）后逐项一致。`.theme-seg` 五条规则与原型逐字相同。 |
| `.card-head h2` 的 `min-height(auto→0px)` 与宽度 | **无视觉影响**：原型用 `.card-head-title{display:flex}` 包一层（全原型仅 3 处使用），生产直接放 `<div>`。标题均为单行、左对齐、无底色/描边，渲染高度由行高决定。 |
| `.controls{align-items}`（原型 `normal`/拉伸，生产 `center`） | **原型无对应组合**：冻结原型没有任何 `setting-row` 把 `.tag` 放进 `.controls`，该差异在原型自身排版下不可观测；生产把状态标签放进 `.controls` 属生产侧组合，`center` 是有意选择。 |
| 扩展页 `.skills-list` / `.agent-path-list` 高度、`.tag` / `.select-trigger` 缺失 | **数据驱动**：Web 审计没有 Tauri 后端，未发现技能/客户端路径，列表为空；原型写死样例数据。 |
| `.panel-embedded-note` / `panel-state-actions` / `topbar-compact` 无规则 | 仅内容型包装类或冗余开关类；`compact` 只用于切换子节点显隐。已登记为低影响遗留，不作样式补齐。 |

### 15.5 新增回归

`src-ui/tests/overview-mods.test.ts`：断言 3 张模块卡都有真实 `button.mod-go`（有 `aria-label`、不 `aria-hidden`）、动作区不再出现「…设置」按钮；并断言点击「版本升级」卡的 ⋯ 后 `routes.current==='settings'` 且 `settingsSection==='upgrade'`。

### 15.6 门禁结果（本段）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：30 文件 / 119 测试 |
| `npm --prefix src-ui run build` | 通过；正式产物不含审计夹具（`__audit` 计数 0） |

### 15.7 仍待完成

1. 桌面实机审计（阶段 6）仍受屏幕录制权限阻塞（辅助功能已可用，见 §16）。
2. 系统通知权限与触达、通知材质与窗口内位置、最小窗口尺寸差异仍未定稿（候选，需用户确认）。

---

## 16. 续作：阶段 6 桌面审计的实机尝试（2026-09-19 第十段）

### 16.1 权限复核

| 能力 | 结果 |
|---|---|
| 屏幕录制 | **仍未授权**：`screencapture -x` 输出 6400×2700 但 60×40 采样只有 1 种颜色（纯黑） |
| 辅助功能 / 自动化 | **已可用**：`System Events` 能列出进程（Feishu、Code、Terminal…），`osascript` 不再返回空 |

结论：像素级验收仍不可用；非视觉的实机检查可以尝试。

### 16.2 检出并修复的启动缺陷

`src-tauri/tauri.conf.json` 的钩子命令用 `../src-ui`，但 Tauri CLI 执行钩子时的工作目录是**仓库根**，不是 `src-tauri`。

证据：`cargo tauri dev --config '{"build":{"beforeDevCommand":"pwd"}}' --no-dev-server --no-watch` 输出 `/Users/ezio/WorkSpace/Code/tools/opencodex-desktop`。因此原配置下 `npm --prefix ../src-ui run dev` 会去找 `…/tools/src-ui/package.json`：

```
npm error path /Users/ezio/WorkSpace/Code/tools/src-ui/package.json
Error The "beforeDevCommand" terminated with a non-zero status code.
```

即计划 §9 要求的「在 `src-tauri` 执行 `cargo tauri dev`」在当前配置下**开箱即失败**。

修复：`beforeDevCommand` → `npm --prefix src-ui run dev`；`beforeBuildCommand` → `npm --prefix src-ui run build`（`frontendDist` 仍按配置文件相对路径 `../src-ui/dist`，两者约定不同是这次错位的根因，已在记录中标注）。

复验：修复后 `cargo tauri dev --no-watch` 依次输出 `Running BeforeDevCommand` → Vite `ready in 76 ms` → `127.0.0.1:5173` → `Running target/debug/opencodex-desktop`，启动链路贯通。

### 16.3 隔离夹具下的启动实测

用 `HOME=/tmp/ocxd-audit-home`（同时保留 `RUSTUP_HOME`/`CARGO_HOME` 指向真实目录，否则 rustup 找不到工具链）隔离应用数据根：

- 应用数据根被正确初始化：`/tmp/ocxd-audit-home/Library/Application Support/com.gzers.opencodex.desktop/{opencodex-home,cache,manager-state,sync-state,exports,logs,backups}` —— 说明 FZ-02 的「首次启动初始化数据根」在真实二进制里执行成功。
- `lsappinfo` 显示进程为 `type="Foreground"`，并有 WebKit 子进程（`… Networking` / `… Graphics and Media`）→ env 进程与 webview 均已起来，未崩溃。

### 16.4 仍然阻塞：无法观测到原生窗口

| 检查 | 结果 |
|---|---|
| `System Events` `count of windows` | `0`（进程可见、`visible=true`） |
| `CGWindowListCopyWindowInfo`（JXA/ObjC 桥）`kCGWindowListOptionAll` | 770 个窗口，**属于该进程的 0 个** |
| 同上 `kCGWindowListOptionOnScreenOnly` | 66 个窗口，**属于该进程的 0 个** |

用此前会话留下的 release 包（`target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app`）作**诊断对照**（不作为本轮验收证据，因为不是当前源码构建）：同样进程存活、WebKit 子进程存在、窗口数为 0。

因此无法判定这是「裸二进制/终端启动下的激活策略限制」还是产品缺陷；需要一次由用户在真实桌面环境下从 `.app` 启动的观察来区分。本轮不把它写成缺陷，也不写成通过。

**未验证项（阶段 6）**：原生红黄绿与拖拽、安全区、宽窄导航、内容圆角、玻璃合成、窗口缩放与滚动、系统主题；真实面板首次打开/隐藏后再开/重载/失败兜底；内嵌面板上触发通知、任务、详情弹窗时原生子 WebView 不遮壳层不吞点击；真实异步链路（启动/停止/重启、通知接收、失败日志与任务进度）；重启后的通知持久性与计数；系统通知权限与触达。

> 计划 §6 的完成条件是「原生窗口、子 WebView、异步 UI 和实际通知流程均有本次实机证据」；当前只有**契约与编译**证据，不满足该条件。

### 16.5 门禁结果（本段）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过：30 文件 / 119 测试 |
| `npm --prefix src-ui run build` | 通过 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | 通过 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 通过 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 通过：307 个测试，0 失败（lib 246 + 集成各套件） |

### 16.6 需要的人工动作

1. 给宿主终端授予「屏幕录制」权限（或由用户本人在场观察），随后才能完成 §16.4 列出的实机验证。
2. 确认 §14.7 的四项候选（窗口内位置、通知材质、系统通知范围、最小窗口尺寸），之后才能收敛进 `UI规范.md` / IMP（涉及权威文档需单独登记 TASK）。

---

## 17. 回写：UI 规范补齐页面级状态标签（TASK-OPENCODEX-DESKTOP-63）

§15.2 的缺口 #1（页面级 `.tag` 没有基础规则）根因是**《UI规范》§3 的 Badge/Tag 行只写了通知侧的 `.n-tag` / `.n-count`，没有页面级 `.tag`**：规范缺条目 → 实现漏样式。本轮按治理流程单独开列有界 TASK 回写权威文档。

| 项 | 值 |
|---|---|
| TASK | `TASK-OPENCODEX-DESKTOP-63`（`completed` / `succeeded` / rev 3） |
| RCP | `RCP-OPENCODEX-DESKTOP-63` |
| CERT | `CERT-OPENCODEX-DESKTOP-63` |
| 回写目标 | `docs/02-项目核心/UI规范.md`（`before` `7720bccd…afc2` → `after` `9e8accc0…c13b`） |
| 验收绑定 | `AC-OPENCODEX-DESKTOP-04`、`AC-OPENCODEX-DESKTOP-12` |
| 写入范围 | `docs/02-项目核心/**` |

变更内容：

- §3 表内 Badge/Tag 行由「状态标签 `.n-tag`、计数徽标 `.n-count`」扩为「通知状态标签 `.n-tag`、计数徽标 `.n-count`；页面状态标签 `.tag`」，尺寸列补 `.tag` 的 `min-height:22px`、`padding:2px 8px`，状态列注明 `.tag` 是胶囊（`1px --border` + `--panel` 底 + `--radius-pill`）。
- §3 约定新增一条：页面级 `.tag` 与通知内 `.n-tag` 是同一控件族的两个变体，**都必须有基础规则**；否则 `.tag.danger`、`.skill-title .tag` 这类局部覆盖会失去依托而退化成纯文本。

治理校验：`python3 -m runtime project-refresh --root <项目根> --force-board` 后 `project-check` 结果为 `state: valid`（7 条 catalog warning 均为既有、指向 TASK-25/30/57 等既有对象，与本轮无关）。

> 说明：本段同时存在 §15 的 `src-ui` 代码修正（CSS/模板/测试），它们不改变权威长期事实，因此不单独开列 TASK；权威文档只改了《UI规范》，按治理要求走 TASK-63。

---

## 18. 补充验证：构建钩子的工作目录（2026-09-19）

§16.2 只验证了 `beforeDevCommand`；本轮用「让钩子快速失败」的方式确认 `beforeBuildCommand` 使用同一工作目录，避免跑完整 release 构建：

```
cargo tauri build --config '{"build":{"beforeBuildCommand":"pwd; false"}}'
→ Running beforeBuildCommand `pwd; false`
  /Users/ezio/WorkSpace/Code/tools/opencodex-desktop
  Error beforeBuildCommand `pwd; false` failed with exit code 1
```

结论：`beforeDevCommand` 与 `beforeBuildCommand` 都以**仓库根**为工作目录，因此 `npm --prefix src-ui run dev` / `npm --prefix src-ui run build` 是正确写法；配置里 `frontendDist` 仍按配置文件相对路径 `../src-ui/dist`。本次未执行完整 `cargo tauri build`（不在本轮授权范围内，且不等同签名或发布）。

---

## 19. 阶段 6 桌面实机审计（2026-09-19 第十一段，用户已授权查看/控制）

用户授权后，屏幕录制权限已生效（`screencapture` 采样由「1 色纯黑」变为正常多色）；辅助功能可用。本轮用**当前源码**构建的 Tauri 开发版完成原生实机审计。

### 19.1 环境与启动方式

| 项 | 值 |
|---|---|
| 二进制 | `src-tauri/target/debug/opencodex-desktop`（`cargo tauri dev --no-watch` 构建，前端来自 `http://localhost:5173`） |
| 平台 | macOS（ARM64） |
| 窗口 | 位置 `(1010,264)`，尺寸 `1180×760`，`subrole=AXStandardWindow`，标题为空（`hiddenTitle`） |
| 隔离夹具 | 先以 `HOME=/tmp/ocxd-audit-home` 运行（独立数据根），再以真实 HOME 运行真实链路 |

**启动方式的重要事实**：只有在**前台 tty 会话**中运行二进制时窗口才会出现；用 `nohup … &` 从非 tty 上下文启动时，进程会存活但**从不注册窗口**（`CGWindowListCopyWindowInfo` 对该 pid 返回 0，`lsappinfo` 也不登记为 Foreground）。这解释了 §16.4 把「无窗口」误判为环境阻塞的原因——那是启动方式问题，不是产品缺陷。用同一份数据根复现已确认。

### 19.2 已通过的原生验证（含像素/几何实测）

| # | 检查项 | 方法与证据 |
|---|---|---|
| 1 | 原生红黄绿独占顶部控制行 | 像素聚类：红 `x9–22`、黄 `x33–45`、绿 `x55–68`，均落在 `y9–22`（约 28px 控制行） |
| 2 | 品牌在控制行之下，导航在品牌之下 | 侧栏纵向扫描：`y=23–53` 为底面，品牌 Logo 出现在 `y≈54–81`，导航项在 `y≈105+` |
| 3 | 单一玻璃底面（菜单/标题栏不另叠玻璃） | `x=1..244` 为 `--glass-rail`（半透明，透出桌面；浅色实测 `(232,238,249)`、深色 `(21,21,21)`，随桌面内容变化） |
| 4 | 内容页左/顶 1px 描边 + 左上圆角 | `x=245` 为 1px 描边 `(230,230,230)`（深色 `(61,61,61)`）；左上角对角线 `d=1..5` 仍是底面、`d=6` 才转白 → 约 12px 圆角 |
| 5 | 宽窄导航且图标列不跳动 | 宽栏侧栏 244px / 窄栏 64px；图标起始列在两种模式下**完全相同**：品牌 `x=22`，导航 `x=27 / 12 / 29`（浅深主题均测） |
| 6 | 主题（浅色/深色）实时切换 | 通过 UI 按下「深色」：内容页由 `(255,255,255)` 变为 `(38,38,38)`，底面由 `(249,249,249)` 变为 `(21,21,21)` |
| 7 | 最小窗口约束 | AX 设尺寸：请求 `900×600` / `800×500` / `600×400` 均被夹到 **`960×640`**；`1400×900` 允许；恢复 `1180×760` 正常。（候选 (d) 由此获得实机结论：当前配置下**无法**缩到原型的 900×600） |
| 8 | 面板路由窄栏 | 侧栏 64px；导航文字收起后按钮名称为空（仅图标），图标列与宽栏一致 |
| 9 | 通知层在原生窗口内渲染 | 打开通知中心：右上区域出现区别于页面的半透明玻璃面（深色下 `(53,62,80)→(69,72,78)`，自 `y≈58` 起），未越出窗口 |
| 10 | 概览模块卡的设置入口（§15 修复） | AX：三张卡均为 `进入「版本升级」/「配置迁移」/「WebDAV 同步」设置` 的真实按钮，动作区不再有「…设置」按钮 |
| 11 | 不使用原型固定数据 | 隔离夹具下：端口/版本/数据目录均为 `—`、扩展页「暂无 Skills」；真实环境下显示真实值 `端口 10100`、`版本 2.50.0`、`npm 全局` |

> **更正（见 §21.4）**：本节第 11 项「不使用原型固定数据」的结论**不完整**。当时只核对了概览与扩展页的展示值，**漏检了通知中心**：后端在启动时仍以 `default_notification_store()` 预置 8 条原型演示通知（含固定版本号 `2.50.0/2.51.0`、`v0.1.0/v0.1.1`、模拟错误「OpenCodex 启动失败」「未检测到 Node.js」与示例未读数量）。该项在 §21 修复后重新成立，详见 §21。

### 19.3 真实异步链路（部分通过）

以真实 HOME 运行，从概览按下「启动 OpenCodex」：

| 步骤 | 实测 |
|---|---|
| 指令发出 | 任务卡出现：标题「启动未完成」，动作区步骤 `✓ 启动指令已发出` / `• 等待官方进程启动` / `• 等待 ready 健康检查`，文案「等待时间较长，后台仍会继续观察状态；可以先关闭此窗口」——**等待超时不写成失败**，符合 FZ-43.2 |
| 服务真的起来 | `ps` 出现 `node …/.local/bin/ocx start` 与官方 `bun` CLI；`lsof` 显示 `127.0.0.1:10100 LISTEN`；`ocx status` 为 `Proxy: running` / `healthz ok (live)` |
| 状态收敛 | 概览刷新后显示「运行中 · 端口 10100 · 版本 2.50.0 · 代理运行中，可停止或重启」；「官方 status 报告启动状态存在风险…（bundled）」的提示来自用户机器既有的 `Restart safety: AT RISK`，非本轮引入 |
| 停止 | 按下「停止」→ 任务卡「停止已完成 / 代理已停止，当前窗口仍可继续操作」，步骤 `✓ 停止指令已发出` / `✓ 确认代理已停止`；`ocx status` 回到 `Proxy: not running` |

**状态已还原**：任务开始前该服务处于停止状态，审计结束后已通过应用自身停止，服务回到未运行。审计只收发官方启停指令，未改写官方配置、未做 restore、未安装 service。

### 19.4 未通过 / 待定位

**官方面板在「运行中」状态下仍无法嵌入**：`/web` 实测 `200`（`/` 与 `/healthz` 同为 200），但面板路由持续显示回退提示「官方面板需要 OpenCodex 处于运行且健康可用状态。」，且该文案**不带**「请先启动服务」后缀（说明走的是 `panelError` 为空的回退分支），面板区域像素恒为应用底色 `(38,38,38)`，原生子 WebView 未出现；离开路由再进入、连按「重试」均复现。

可能相关的代码位置（尚未定位到根因，本轮不改动）：`src-ui/src/stores/app.ts#loadPanelUrl` 的 `panelLoading` 早退分支不清错、`PanelRoute.loadPanel` 仅在 `!app.statusSnapshot` 时刷新快照、`src-tauri/src/commands/panel.rs` 的 `Show` 分支要求采集器此刻为 `Running`。

因此「原生子 WebView 不遮壳层、不吞点击」这条**仍未验证**（面板没起来）。

### 19.5 本轮仍未验证

1. 内嵌面板上的通知/任务/弹窗与原生子 WebView 的合成关系（依赖 19.4）。
2. 「重启 OpenCodex」与「重启应用后的通知持久性与计数」（需要再做一轮重启；未执行以免再次改动用户运行态）。
3. 系统主题（跟随系统）随 macOS 外观切换（未改动用户系统外观设置）。
4. 拖拽窗口、滚动容器原生手势（未逐项复现）。

### 19.6 证据

`.adg/work/evidence/task63-desktop-audit/`：`01-overview-light`、`02-overview-dark`、`03-min-960x640`、`04-settings`、`05-logs`、`06-extensions`、`07-settings-about`、`08-settings-general`、`09-notification-center`、`10-panel-narrow-rail`、`11-overview-wide-dark`、`12-real-running-overview`、`13-start-task-timeout`、`14-panel-not-embedded`、`15-stop-completed`。

### 19.7 门禁（本轮未改代码，复跑确认）

`npm --prefix src-ui run typecheck` / `test`（30 文件 / 119 测试）/ `build` 通过；`cargo fmt --check`、`clippy -D warnings`、`cargo test`（307）通过。

## 20. 续作：官方面板「无法嵌入」缺陷定位与修复（2026-09-19 第十二段）

定位对象是 §19.4 未通过项「官方面板在运行中状态仍无法嵌入」。

### 20.1 根因

| 观测 | 推断 |
|---|---|
| 服务 `running`、`/web` 实测 `200`，但面板路由持续显示回退提示 | 与业务可达性无关，问题在壳层↔原生命令的边界 |
| 回退文案**不带**「请先启动服务」后缀 | 该文案来自 `PanelRoute.vue#syncPanel` 的 `catch` 分支兜底（`app.panelError` 此时已被 `loadPanelUrl()` 清空），即 `syncEmbeddedPanel` 抛出了异常，而不是走 `!result.visible` 分支 |
| 命令为何抛异常 | `src-tauri/src/commands/panel.rs#PanelBounds::validate` 要求 `x ≥ 64`（侧栏）且 `y ≥ 28`（控制行），否则返回 `Err(AppError::NotConfigured)`；命令返回 `Err` → 前端 Promise 拒绝 |
| 几何实测（无头） | 生产壳层此前把 `.app-shell` 铺满窗口高度，**第一行控制行被忽略**，`.panel-frame` 落在 `x=66, y=2`；`y=2 < 28` → 校验失败 → 未嵌入 |
| 对照冻结原型 | 原型 `.app` 用 `grid-template-rows:28px …`，`.main`/`.panel-frame` 在**每个**路由都位于 `y≈30`；生产此前 `.main` 在 `y=1` |

结论：这是**阶段 4 复刻壳层时遗留的坐标不一致**（控制行高度未参与壳层行分配），不是 URL 白名单、凭据或 IPC 权限问题；按计划 §7「改动壳层时同步核对 Rust 子 WebView 的坐标与尺寸校验」应在此修正。

### 20.2 修复

`src-ui/src/styles/base.css`（纯前端布局，未放宽任何 Rust 校验）：

```css
.app-shell { grid-template-rows: var(--titlebar-height) minmax(0, 1fr); } /* 第一行 = 控制行 */
.app-shell > .side { grid-row: 1 / -1; } /* 菜单/品牌跨两行，图标列不受内容页下移影响 */
.app-shell > .main { grid-row: 2; }      /* 内容页从第二行开始，保留顶部安全区与顶描边 */
```

该规则与《UI规范》§1「标题栏 | 原生红黄绿独占顶部控制行（高 28px）；品牌在其下、导航在品牌下；`.window{grid-template-rows:28px …}`」一致，属**代码向已确认规范收敛**，未改权威文档，故本轮不新开 TASK。

### 20.3 修复后几何（无头 Chromium，`1180×760`，macOS 平台 → `--window-controls-height:28px`）

| 路由 | 选择器 | x | y | 尺寸 |
|---|---|---|---|---|
| overview | `.app-shell>.side` | 1 | 1 | 244 × 758 |
| overview | `.main` | 245 | 29 | 934 × 730 |
| panel | `.app-shell>.side` | 1 | 1 | 64 × 758 |
| panel | `.main` | 65 | 29 | 1114 × 730 |
| panel | `.panel-frame` | **66** | **30** | 1113 × 729 |

对照 Rust 校验：`x=66 ≥ 64` ✓、`y=30 ≥ 28` ✓、右/底均在窗口内（`66+1113=1179 ≤ 1181`、`30+729=759 ≤ 761`）✓。与原型 `.main`/`.panel-frame` 的 `y=30` 一致。

### 20.4 门禁（本段改 CSS 后复跑）

| 命令 | 结果 |
|---|---|
| `npm --prefix src-ui run typecheck` | 通过 |
| `npm --prefix src-ui test -- --run` | 通过（30 文件 / 119 测试） |
| `npm --prefix src-ui run build` | 通过 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | 通过 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 通过 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 通过（**307 passed / 0 failed**，lib 246） |

### 20.5 桌面实机复验：本轮被系统锁屏阻塞（未通过，不计入完成）

- 已按 §19.1 结论以**前台 tty** 启动本轮源码对应的 `src-tauri/target/debug/opencodex-desktop`，并单起 `npm --prefix src-ui run dev`（`127.0.0.1:5173`）。`lsappinfo` 显示该进程为 `Foreground`，WebKit `WebContent`/`Networking`/`GPU` 子进程均已拉起（说明 webview 已加载），进程存活。
- 但本机当前 `CGSSessionScreenIsLocked: true`（锁屏时间 `2026-09-19 09:26:58`）：`screencapture -x` 全屏输出为**单一灰度 0（全黑）**；`System Events` 对**所有** App（含 Finder）都返回 `windows=0`。锁屏下既取不到窗口像素，也取不到 AX 树。
- 因此 §19.4 修复后的「内嵌面板像素非底色、注入浮层不遮壳层/不吞点击」**仍无实机证据**，不记为通过。
- 附带澄清：§19.1「窗口只在**前台 tty** 出现、后台启动不注册窗口」这一结论是在 `System Events` 得 `windows=0` 的基础上得出的，而锁屏同样会令 `windows=0`；该结论需在解锁后**重新确认**，本轮不据其下任何新判断。

### 20.6 需要的人工动作

解锁屏幕后可在**同一次运行内**继续：进入面板路由 → 确认面板区域像素不再是应用底色、注入浮层（球/缩放/概览/重载/浏览器、`aria-expanded`、Escape 与外部点击关闭）正常、通知/任务叠层不遮官方内容且点选不被吞；随后按 §19.3 还原服务到未运行。

### 20.7 本轮未提交改动

仅新增/修改 `src-ui/src/styles/base.css`（及本记录）；未提交、未建分支、未发布、未做全局同步。工作树保持既有未提交状态（HEAD 仍为 `d1858a7`）。

## 21. 续作：清除正式运行的通知演示种子（2026-09-19 第十三段）

> 附：清理种子时同时核实了通知的**持久化实现**，结论与 §19/§14 的「待验证」口径不同，须更正为**未实现**，见 §21.5。

### 21.1 发现

对照计划 §1「不把原型的模拟成功、固定数据或定时器复制到正式业务链路」与阶段 4「不复制原型固定版本号、示例端口/PID、未读数量、模拟错误、场景定时器和评审工具到正式页面」，定点检查通知数据来源，发现：

| 证据 | 内容 |
|---|---|
| 启动接线 | `src-tauri/src/lib.rs` 用 `default_notification_store()` 初始化 `SharedNotificationStore` |
| 种子内容 | `commands/notifications.rs` 硬编码 8 条：`external-takeover`、`upgrade-available`（「当前 2.50.0，最新 2.51.0」）、`app-update-available`（「当前 v0.1.0，最新 v0.1.1」）、`sync-connected`、`sync-conflict`（`op_5c11`）、`start-failed`（「OpenCodex 启动失败」`op_2a90`）、`env-node-missing`（「未检测到 Node.js」）、`auto-clean`（「清理了 12 条过期日志和 3 条已读通知」） |
| 有无真实生产者 | 全仓 `push(` 无一处面向 `NotificationStore`；通知中心的**全部内容**来自这份原型演示数据 |
| 违反的权威条款 | `领域模型.md` §16「正式运行不得由演示种子冒充真实通知」；`数据与状态.md` §5.7「正式运行不得用演示种子冒充真实通知；测试夹具与场景切换器只在开发 / 测试环境存在」 |
| 为何此前漏检 | 阶段 4 已清除**前端**同源固定数据（`src-ui` diff 删除 `{ id: 'upgrade-available' … '当前 2.50.0，最新 2.51.0' … }` 等），但**后端同源种子被漏掉**；§19.2 第 11 项只核对了概览与扩展页 |

### 21.2 修复

| 文件 | 改动 |
|---|---|
| `src-tauri/src/lib.rs` | 启动改为 `NotificationStore::new()`（空集），并注明依据 |
| `src-tauri/src/commands/notifications.rs` | 删除生产路径上的 `default_notification_store()`，收窄为 `use …::NotificationStore` |
| `src-tauri/src/commands/notifications.rs`（测试） | 新增 `#[cfg(test)] fixture_store()`；新增 `production_store_starts_without_demo_seed` |
| `src-tauri/tests/notifications.rs` | 夹具内联为测试本地函数；断言按 4 条夹具重算 |

- 演示数据只存在于**测试代码**，不随产品编译，也不需要新增依赖或 Cargo feature。
- 未放宽任何校验、未新增旁路接口；改动纯粹是把运行期数据源收回到「只由真实事件写入」。
- 前端已有空态（`NotificationCenter.vue`「当前分类没有通知。」、`LogsRoute.vue`「暂无通知历史。」），空集不会出现空白面板。

### 21.3 门禁（本段）

| 命令 | 结果 |
|---|---|
| `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` | 通过（无差异） |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 通过 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 通过（**307 passed / 0 failed**） |
| `npm --prefix src-ui run typecheck` / `test -- --run`（30 文件 / 119）/ `build` | 通过 |

已重建 `src-tauri/target/debug/opencodex-desktop`（09:45）并在前台 tty 重启，供解锁后续验。

### 21.4 与 §19 的关系

§19.2 第 11 项结论过宽，已在原处加更正说明；**修复后该项才成立**。仍待桌面实机确认（依赖解锁）：通知中心显示空态、计数为 0、铃铛无角标。

空集的呈现已在无头 Chromium 下用显式场景 `scenario=empty` 验证（证据 `.adg/work/evidence/task63-notification-empty/`）：

| 场景 | 实测 |
|---|---|
| 通知中心（`route=overview&scenario=empty&panel=1`） | 面板存在且尺寸正常（`x=760,y=56,400×169`）、条目 0、分类计数全为 `0`、铃铛无角标、空态文案「当前分类没有通知。」 |
| 通知历史（`route=logs&tab=notifications&scenario=empty`） | 空态文案「暂无通知历史。」、条目 0、无角标 |

即：清空生产种子后不会出现空白或破损面板。桌面实机（原生窗口内）确认仍待解锁。

- §19.6 证据中的 `09-notification-center.png` 记录的是**修复前**的状态（当时通知中心展示的即这段演示种子），保留为历史对照，不作为修复后证据。

### 21.5 本轮登记为缺口、未在本轮改动

> **已部分修复（见 §24）**：已接线「生命周期失败」这一真实事件源；本条保留原始发现记录。

- **通知尚无真实生产者**：FZ-43 的「操作性通知」目前没有任何运行期事件写入 store，因此正式运行的通知中心会长期为空。这是本次修复暴露出的**后续实施缺口**，需要单独授权把真实事件（启动/停止失败、同步冲突、更新可用、外部接管等）接入同一 store；本轮不做臆测式接线，也不回填任何演示数据。

> **已修复（见 §22）**：本条在下一段已实现，保留原始发现记录。

- **通知的"持久化与恢复"未实现（口径更正）**：《数据与状态》§5.7 规定通知实体落点为 `manager state`，且「重启后按 `read` / `resolved` / `deleted` 原样恢复；未解决通知不因重启丢失」。实测**没有**任何实现：

  | 证据 | 内容 |
  |---|---|
  | 存储形态 | `state.rs`：`SharedNotificationStore = Arc<Mutex<NotificationStore>>`，`NotificationStore` 只持有 `Vec<Notification>`，无文件 I/O |
  | 磁盘落点 | 全仓无 `notifications.json` / 通知存储文件；`manager-state/` 现有 `data-root.json`、`structure.lock`、`preferences.json`、`extension-config.json`、`skills-store`、`app.lock`，**不含通知** |
  | 启动加载 | `lib.rs` 直接构造 store（修复后为空集），未从磁盘恢复 |

  因此记录中「重启持久性与计数**待验证**」（§6.6/§14.5/§19.5）应更正为**未实现**：不是没测，而是没有可测的实现。该缺口依赖「真实生产者」先落地（空集持久化无意义），一并留待单独授权。

### 21.6 未提交改动

本轮新增修改 `src-tauri/src/lib.rs`、`src-tauri/src/commands/notifications.rs`、`src-tauri/tests/notifications.rs` 与本记录；未改权威文档，故不新开 TASK。仍未提交、未建分支、未发布。

## 22. 续作：实现通知的持久化与恢复（2026-09-19 第十四段）

修复 §21.5 登记的缺口：《数据与状态》§5.7 要求通知实体落点为 `manager state`、「重启后按 `read` / `resolved` / `deleted` 原样恢复；未解决通知不因重启丢失」，而此前**完全没有实现**（纯内存、无落盘）。

### 22.1 实现

| 层 | 改动 |
|---|---|
| 领域/基础设施 | 新增 `src-tauri/src/modules/notifications/persistence.rs`：落点 `manager-state/notifications.json`（`NOTIFICATIONS_RELATIVE_PATH`），格式 `{"version":1,"items":[…Notification…]}` |
| 读取 | 文件缺失 → **空集**（正式运行不预置演示种子）；损坏或版本不受支持 → **显式失败**，不静默丢弃；`read`/`resolved`/`deleted` 原样恢复（用 `all()` 而非对外投影 `live()`） |
| 写入 | 复用 `infrastructure::atomic_write` 原子替换，保持 `0600` |
| 命令层 | 改为「**先落盘、再提交内存**」：把变更应用到副本、`save_notifications` 成功后 `*guard = next`；任一步失败即返回显式错误且**内存不变**，避免「界面说成功、重启后丢失」的假成功 |
| 启动接线 | `lib.rs` 用 `startup_notification_store(&data_root)` 读取；`cleanup_local_notifications` 透传 `data_root` 以便清理后落盘 |
| 损坏处理 | 启动读取失败时**不静默丢弃**：以空集启动，并写入一条**真实**失败通知 `notifications-store-unreadable`（System/Warning/`Logs`）说明历史无法读取、原文件保留。这是「本次读取失败」这一真实事件的记录，不是演示种子 |

真实落点已确认：`~/Library/Application Support/com.gzers.opencodex.desktop/manager-state/`（与 `preferences.json`、`extension-config.json` 同分区）；首次启动且无该文件时确实不产生 `notifications.json`（只在变更时写入）。

### 22.2 测试与门禁（本段）

| 项 | 结果 |
|---|---|
| 新增模块测试 `persistence.rs` | 5 个：缺失文件→空集、往返保留 read/resolved/deleted + `0600`、损坏/未知版本显式失败、相对路径拒绝、错误映射不冒充成功 |
| 命令层测试 | 6 个（含「落盘失败不得改动内存」「变更后从磁盘重读仍一致」） |
| 集成测试 `tests/notifications.rs` | 5 个（含「首次启动不预置种子」「有历史文件时按原样恢复」） |
| `cargo test --manifest-path src-tauri/Cargo.toml` | **316 passed / 0 failed** |
| `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` | 通过 |
| `npm --prefix src-ui run typecheck` / `test -- --run`（30 文件 / 119）/ `build` | 通过（命令层新增的是 Tauri 注入的 `State` 参数，不进入前端 IPC 载荷，故前端调用与 DTO 不变） |

已重建 `src-tauri/target/debug/opencodex-desktop`（09:53）并在前台 tty 重启。

### 22.3 仍未验证 / 仍缺口

- **实机可见证据待解锁**：在原生窗口里观察「产生一条真实通知 → 重启后仍在」的完整链路。当前无真实生产者（见下），故这段只能在解锁后配合真实事件验证；Rust 层已用磁盘往返测试覆盖同一语义。
- **真实生产者仍是缺口**（§21.5 第一条，未在本段改动）：FZ-43 的「操作性通知」尚无运行期事件写入 store，通知中心在正式运行会长期为空。本段的 `notifications-store-unreadable` 是其中唯一已接线的真实事件。

### 22.4 未提交改动

本段新增 `src-tauri/src/modules/notifications/persistence.rs`，修改 `modules/notifications/mod.rs`、`commands/notifications.rs`、`commands/cleanup.rs`、`lib.rs`、`tests/notifications.rs` 与本记录。未改权威文档，故不新开 TASK；仍未提交、未建分支、未发布。

## 23. 续作：补齐《领域模型》§16 缺失的 `source` 与 `expires_at`（2026-09-19 第十五段）

### 23.1 发现

《领域模型》§16 的字段表把 `source`（触发来源 `user_action` / `runtime` / `sync` / `update` / `diagnostic`，**必填、冻结 ✅**）与 `expires_at`（timestamp \| null，**冻结 ✅**）列为实体字段，并明确「类型、严重程度、业务分类与**触发来源**是四个独立维度」；计划 §4.2 也要求「**触发来源分开**」。`expires_at` 的语义另在《数据与状态》§5.4（过期与清理）被引用。

定点核对实现后发现两者**完全没有实现**：

| 检查点 | 实测 |
|---|---|
| Rust 实体 `Notification` | 只有 `notification_id` / `level` / `category` / `title` / `body` / `created_at` / `read` / `resolved` / `resolved_at` / `deleted` / `operation_id` / `dedupe_key` / `action_ref`——**无 `source`、无 `expires_at`** |
| 跨边界 DTO `NotificationDto` | **无 `source` / `expiresAt`** |
| 前端 `NotificationItem` | **无对应字段** |

### 23.2 修复

| 层 | 改动 |
|---|---|
| 领域 | 新增 `NotificationSource` 枚举（`#[serde(rename_all = "snake_case")]`）；`Notification` 增加必填 `source` 与可选 `expires_at`；`Notification::new()` 增加 `source` 形参；新增 `with_expires_at()`（`None` = 不可自动过期） |
| 传输 | `NotificationDto` 增加 `source` / `expiresAt`（camelCase），`from_domain` 直通 |
| 前端契约 | `NotificationItem` 增加 `source?` / `expiresAt?`，注明后端始终下发、旧夹具可省略 |
| 调用点 | 16 处 `Notification::new` 与 2 处结构体字面量按语义补 `source`（运行期失败→`runtime`、同步→`sync`、更新→`update`、读取失败与外部接管→`diagnostic`） |
| 形参数量 | `new()` 共 8 个形参，逐一对应 §16 必填字段（身份、三个独立维度、标题、正文、时间、动作目标；时间与 id 由调用方给出，领域层不臆测时钟）。沿用仓库既有先例 `#[allow(clippy::too_many_arguments)]`（`modules/sync/engine.rs:408`）并写明理由，未整体放宽 lint |
| 存储格式 | `notifications.json` 的条目结构随之扩展；该特性与本段同属本次未提交改动（尚未发布、磁盘上也不存在旧文件），故 `NOTIFICATIONS_SCHEMA_VERSION` 保持 `1`，不引入无意义的迁移分支 |

### 23.3 测试与门禁（本段）

| 项 | 结果 |
|---|---|
| 新增/加强测试 | `source_and_expiry_are_independent_dimensions`（默认不可过期、过期资格不等于删除、枚举 snake_case 序列化）；DTO 断言 `source` / `expiresAt` 跨边界下发 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | **317 passed / 0 failed** |
| `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` | 通过 |
| `npm --prefix src-ui run typecheck` / `test -- --run`（30 文件 / 119）/ `build` | 通过 |

已重建 `src-tauri/target/debug/opencodex-desktop`（09:58）并在前台 tty 重启；进程与 3 个 WebKit 子进程正常。

### 23.4 仍未验证 / 仍缺口

- **实机视觉项仍待解锁**：官方面板嵌入（§20）、原生窗口内通知中心空态与计数。
- **真实生产者仍是缺口**（§21.5）：FZ-43 的操作性通知尚无运行期事件写入 store。`source` 维度已就位，为后续按来源分流提供契约基础。

### 23.5 未提交改动

本段修改 `src-tauri/src/modules/notifications/mod.rs`、`src-tauri/src/types/notifications.rs`、`src-tauri/src/commands/notifications.rs`、`src-tauri/src/modules/notifications/persistence.rs`、`src-tauri/tests/notifications.rs`、`src-ui/src/types/ui.ts` 与本记录。未改权威文档，故不新开 TASK；仍未提交、未建分支、未发布。

## 24. 续作：接线真实通知生产者（2026-09-19 第十六段）

修复 §21.5 第一条缺口：FZ-43 的「操作性通知」此前没有任何运行期事件写入 store，通知中心只能为空。

### 24.1 依据（不自行发明触达口径）

| 来源 | 口径 |
|---|---|
| 场景与验收矩阵 §7.1 S-12 | 长任务「失败保留错误摘要，**视情况生成通知中心记录**」；「收起 / 后台运行」只隐藏；等待超时收口为「仍在后台观察」，**不写成失败** |
| IMP `FZ-43.2` | 低风险成功走 Toast、**不占持久未读**；同一 `dedupe_key` 存活期内只保留一条；同一操作终态只触达一次 |
| `领域模型` §16.1 | 通知只承载**操作性通知**；同一事件允许一条日志 + 必要时一条操作性通知，**不得堆叠多条同类提醒** |
| IMP `FZ-43` | 失败为 `danger`；「查看日志」动作走 `Logs` |

### 24.2 实现

| 位置 | 改动 |
|---|---|
| `commands/notifications.rs` | 新增 `NotificationPublisher`（`store` + `data_root`）：只负责「写入同一份 store + 落盘」。**发布失败只记运行日志**，不改变主流程的返回值——进程动作的真实结果不会被通知层失败污染 |
| `commands/mod.rs` | `process_action` 注入 `SharedNotificationStore` 构造 publisher；`process_action_with_runner` 增加可选 publisher 形参（测试可传 `None`） |
| 判定规则 | 仅在 `LifecycleResult::Failed` 或 `Err`（spawn 失败）时发布；**成功与用户取消都不写入** |
| 通知内容 | `level=danger`、`category=run`、`source=runtime`、`action_ref=Logs`、`dedupe_key=run:<action>-failed`；正文只描述**已发生事实**（不含固定延迟、示例端口或版本号） |

一个由实现决定的重要细节：`Start` / `Restart` 按设计在拉起进程后**立即返回 `Started`**（守护式启动，不在冻结窗口内等待终态），因此「启动失败」在本层以 `Err`（spawn 失败）形态出现，而 `Failed` 终态主要来自 `Stop` 的非零退出。生产者对两条路径都覆盖，用例分别验证。

### 24.3 测试与门禁（本段）

| 项 | 结果 |
|---|---|
| 新增用例（`tests/process_action.rs`） | `run_failure_publishes_one_deduped_danger_notification`（Stop 非零退出：两次失败只留**一条**，`danger`/`run`/`runtime`/`Logs`，未读 1，且已落盘可重启恢复）、`spawn_failure_also_publishes_a_danger_notification`（`Err` 分支同样发布）、`run_success_publishes_no_notification`（成功不写通知） |
| `cargo test --manifest-path src-tauri/Cargo.toml` | **320 passed / 0 failed** |
| `cargo fmt --check` / `clippy --all-targets -- -D warnings` | 通过 |
| `npm --prefix src-ui run typecheck` / `test -- --run`（30 文件 / 119） | 通过 |

已重建 `src-tauri/target/debug/opencodex-desktop`（10:03）并在前台 tty 重启；进程与 3 个 WebKit 子进程正常。

### 24.4 本段仍未做（不得当成已完成）

- **`operation_id` 与日志的精确互跳仍缺**：进程层目前不产生可与其他通道共享的操作标识；凭空生成一个只被本条通知引用的 id 属于**伪关联**，比留空更差，故 `operation_id` 暂为 `None`。现有能力只是 `action_ref=Logs` 的入口级跳转（FZ-43.4 要求的「日志历史」Tab 激活由前端负责）。
- **其它事件源尚未接线**：同步冲突、更新可用、外部接管、环境门禁等。本段只接最小且契约明确的「生命周期失败」，不做臆测式铺开。
- **实机视觉项仍待解锁**：官方面板嵌入（§20）、原生窗口内通知中心空态与计数。

### 24.5 未提交改动

本段修改 `src-tauri/src/commands/mod.rs`、`src-tauri/src/commands/notifications.rs`、`src-tauri/tests/process_action.rs` 与本记录。未改权威文档，故不新开 TASK；仍未提交、未建分支、未发布。

## 25. 阶段 7 交付说明（更新版，取代 §14）（2026-09-19 第十七段）

> §14 写于阶段 5 / 6 早期，其后 §15–§24 改变了多项事实。本节是**当前**交付说明；§14 保留为历史，不再作为现状依据。本节只登记实际结果、证据位置与缺口，不冒充验收证书。

### 25.1 文档回写清单与相互引用

| 文档 | 本轮回写内容 | 相互引用 |
|---|---|---|
| `docs/02-项目核心/领域模型.md` §16（+§16.1） | `Notification` 实体、字段语义（含 `source` / `expires_at`）与「操作任务 / 技术日志 / 确认请求」边界 | 状态迁移引用《数据与状态》，触达规则引用 FZ-43 |
| `docs/02-项目核心/数据与状态.md` §5（含 §5.7） | 六类渠道职责、操作进度与真实终态、读/解决/删除/过期维度、去重排队、收起、持久化与恢复 | 不复制实体字段表与视觉尺寸 |
| `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` FZ-43.1~43.4 | 渠道与时机、触达/去重/排队、关闭/超时/清理、跨路由与承载；附变更记录行 | 就地修订，未另设冲突版本 |
| 协作包 `…/04-需求分析产物/场景与验收矩阵.md` §7 | S-11~S-16 六类渠道与组合边界场景 | 关联既有 AC，未新建需求级 AC |
| `docs/02-项目核心/UI规范.md`（新增）+ 该目录 `README.md` 登记 | 窗口与布局、设计令牌、基础控件、通知组件、操作规则、键盘与可访问性、边界，及「原型选择器 → token → 组件」映射；§15 补页面级 `.tag` | 视觉参数唯一维护处 |
| `docs/02-项目核心/契约字段.md` | 未受影响 | — |
| `docs/01-需求管理/需求/DMD-OPENCODEX-DESKTOP-MANAGER.md` | 未受影响（无范围 / 验收变更） | — |

§16–§24 期间的修复**一律是「代码向已确认契约收敛」**（seed 移除、持久化落地、`source`/`expires_at` 补齐、壳层坐标对齐、构建钩子工作目录），因此**没有**回写权威文档，也**没有**新开 TASK。唯一回写权威文档的一次是 §15 / TASK-63（页面级 `.tag` 补进《UI规范》§3）。

### 25.2 UI 规范与原型—token—组件映射

`docs/02-项目核心/UI规范.md` 为唯一视觉参数维护处；映射覆盖外壳、侧栏、标题栏、内容页、滚动条、通知令牌、通知中心/条目/Toast/任务卡/对话框、面板浮层、页面级状态标签。已实现部分与冻结原型在设计令牌和计算样式上逐项一致（§8.2、§9、§10、§15）。

### 25.3 组件化与 CSS 整理范围

- 新增 `NotificationCenter.vue`、`NotificationItem.vue`、`ToastHost.vue`、`TaskProgressHost.vue`、`BrandMark.vue`；`AppModal.vue` 收敛为确认 / 详情对话框。
- `tokens.css` 收敛公共设计值（含通知令牌与 reduced-motion）；`base.css` 收敛重置、布局、控件与反馈展示。
- §15：补齐页面级 `.tag` 基础规则、`.section-actions`、`.skills-source`、`.modal-body`；移除重复「…设置」入口与死类名；`overview-mods.test.ts` 回归。
- §20：`base.css` 的 `.app-shell` 恢复「控制行 + 内容行」两行分配（`.side` 跨两行、`.main` 落第二行），使生产壳层与冻结原型的 `y≈30` 对齐，并让 Rust 子 WebView 坐标校验得以通过。
- 通知保持单一事件来源（后端 `NotificationStore`），前端只做投影；无新增隐式旁路接口、无整页重写。

### 25.4 原型复刻完成 / 缺口清单

| 区块 | 状态 |
|---|---|
| 壳层（单一底面、圆角内容页、无分栏竖线、覆盖式滚动条） | 完成；§20 修正了控制行未参与行分配导致的 `y` 偏移 |
| 宽窄栏图标列对齐（15/25px，不跳动） | 完成 |
| 通知中心 / 条目 / Toast / 任务卡 / 收起到铃铛 / 空态 | 完成（Web 证据；桌面实机证据待解锁） |
| 面板快捷浮层（原生注入、玻璃材质、主题随动） | 实现完成；**「面板真正嵌入」的桌面实机证据待解锁**（§20 已修根因） |
| 设置 10 分区、扩展 2 分区、诊断 2 分区：结构、标签、逐行控件、选项、输入校验 | 完成（差异均可归因于 AC-06 / AC-10 与「不使用原型固定数据」） |
| 设置各分区与扩展页的**逐像素**视觉对照 | 缺口（当前到结构、标签、控件、选项、校验行为） |
| 托盘预览、日志 / 诊断页视觉细节 | 部分完成（结构证据 + 截图，未逐像素对照） |

### 25.5 Web 与桌面结果（分别，均为本次实跑）

- **Web（阶段 5）**：`npm --prefix src-ui run typecheck` / `test -- --run` / `build` 全绿（**30 文件 / 119 测试**）；审计夹具只在开发环境按显式 `?__audit=1` 安装，且被 `import.meta.env.DEV` 排除出正式构建。证据 `.adg/work/evidence/task61-web-audit/`（115 个文件）+ `.adg/work/evidence/task63-notification-empty/`（3 个文件，空集空态）。Web 侧阻塞项已清零。
- **桌面（阶段 6）**：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿（**320 passed / 0 failed**），但**编译与契约通过不等于原生流程通过**。
  - 已获实机证据：§19.2 第 1–10 项（原生红黄绿/控制行、品牌与导航层级、单一玻璃底面、内容圆角与描边、宽窄栏图标列、主题切换、最小窗口 `960×640`、面板窄栏、通知层在窗口内、概览模块卡入口）。
  - **未获证据（不计为完成）**：①官方面板嵌入（§20 修复后未复验）；②原生窗口内通知中心（空态与失败通知）。
  - 证据 `.adg/work/evidence/task63-desktop-audit/`（15 个截图；其中 `09-notification-center.png` 为**修复前**状态，仅作历史对照）。

### 25.6 §16–§24 期间修复的契约 / 实现缺口（阶段 7 的新增回写）

| 段 | 缺口 | 处理 |
|---|---|---|
| §18 | `cargo tauri dev` 开箱失败：`beforeDevCommand` / `beforeBuildCommand` 的相对路径与 Tauri CLI 的工作目录不一致 | 改为 `npm --prefix src-ui run …`，两个钩子实测生效 |
| §20 | 官方面板始终无法嵌入：`.panel-frame` 落在 `y=2`，未通过 Rust 的 `y ≥ 28` 校验 | 修 `base.css` 壳层行分配；无头实测 `.panel-frame` `x=66 / y=30` |
| §21 | 正式运行预置 8 条原型演示通知（违反 §16 / §5.7「不得由演示种子冒充真实通知」） | 删除生产种子；夹具只存在于测试代码；新增「生产启动为空集」用例并重建 |
| §22 | §5.7 的通知持久化未实现（纯内存、无落盘） | 新增 `modules/notifications/persistence.rs`（`manager-state/notifications.json`、原子写 `0600`）＋命令层「先落盘、再提交内存」＋启动加载；损坏显式失败并写一条**真实**失败通知 |
| §23 | §16 的 `source`（必填、冻结）与 `expires_at`（冻结）两层都没有实现 | 领域枚举 + 实体字段 + DTO（camelCase）+ 前端契约类型 + 全部调用点补齐 |
| §24 | 通知没有真实生产者 | 接线「生命周期失败」→ 去重的 `danger` 通知（`run:<action>-failed`）；成功与取消不写入 |

### 25.7 未提交改动保留说明

HEAD 仍为 `d1858a7`；工作树 72 项改动 / 未跟踪，涵盖 `src-ui/*`、`src-tauri/*`、`docs/02-项目核心/UI规范.md`（新增）与协作包材料。**未提交、未建分支、未发布、未做全局同步**，未执行高风险真实写入；开工前已有的未提交改动保留未回退。

### 25.8 待用户确认的候选（未定稿）

1. Toast / 任务区「窗口内顶部」与「右上」的最终位置。
2. 通知材质（纯透明 / 轻磨砂 / 实底）与颜色滤镜打样结论。
3. 系统通知的开关、权限与触达范围；终态停留时间与日志关联方式。（「哪些失败进入持久通知」现已部分落地：生命周期失败 → `danger` 通知。）
4. 原型最小窗口 `900×600` 与 Tauri 配置 `960×640` 是否统一。**已有实机结论**：当前配置会把 `900×600` / `800×500` / `600×400` 一律夹到 **`960×640`**（§19.2 #7）；未静默改动平台约束，等确认。

### 25.9 需要的人工动作

- **解锁屏幕**：解锁后即可在同一运行内完成两条待验项（面板嵌入；原生窗口内通知中心空态与失败通知），随后把服务还原到未运行。
- 对 §25.8 的四项候选给出结论；涉及权威文档的收敛需单独登记 TASK。

### 25.10 明确不计为完成

1. 官方面板嵌入的桌面实机证据（§20 已修根因，未复验）。
2. 原生窗口内通知中心（空态 / 失败通知）的实机证据。
3. `operation_id` 与日志的精确互跳（进程层无跨通道操作标识；不伪造关联）。
4. 其它通知事件源（同步冲突 / 更新可用 / 外部接管 / 环境门禁）未接线。
5. 设置分区、扩展页、托盘、诊断页的逐像素视觉对照。

## 26. 阶段 6 收口（一）：原生窗口内通知中心空态（2026-09-19 第十八段）

### 26.1 环境恢复

上一段因系统锁屏阻塞（§25.9）；本次续跑时锁屏已解除（`CGSSessionScreenIsLocked` 不再出现，`screencapture` 恢复非全黑），且发现锁屏期间的休眠 / 重启已终止上一轮的开发版进程与 Vite。已重新启动 `npm --prefix src-ui run dev` 与本次源码构建的 `src-tauri/target/debug/opencodex-desktop`（前台 tty）；窗口可见：`windows=1 pos=[1010,264] size=[1180,760]`。开工前服务处于停止状态（`ocx status` → `Proxy: not running`），本段**未启动官方服务**。

### 26.2 验证结果（原生窗口内）

在概览路由按下顶栏铃铛（AX 中为右上角 34×34 无名按钮，`pos=[2133,314]`）打开通知中心：

| 检查 | 实测 |
|---|---|
| 分类计数 | 全部 **0**、运行 **0**、同步 **0**、更新 **0**、系统 **0** |
| 空态 | `AXStaticText | 当前分类没有通知。` |
| 未读角标 | 无（铃铛无角标文本） |
| 深色主题 | 内容区像素 `(38,38,38)`，证据 `01-center-empty-dark.*` |
| 浅色主题 | 内容区像素 `(255,255,255)`，证据 `02-center-empty-light.*` |
| 附带观察 | 点击顶栏主题按钮会因「外部点击」收起通知中心（符合浮层行为）；切换主题后可再次打开 |

**结论**：§21（清除正式运行的演示种子）与 §22（空集启动 + 持久化）在**真实桌面应用**中获得实机证据——正式运行的通知中心为空，且空态渲染正常，不再是演示数据。

### 26.3 附带状态观察

停止态下概览显示「存在风险 / 端口 10100 / 版本 — / 数据目录 —」与「运行详情 尚未探测到安装」，与官方未运行一致；本段未改动任何服务或配置。

### 26.4 证据

`.adg/work/evidence/task63-desktop-verify/`：`01-center-empty-dark.png` + `01-center-empty-ax.txt`、`02-center-empty-light.png` + `02-center-empty-light-ax.txt`。

### 26.5 仍未完成

> **已完成（见 §27）**：本项已在下一段完成，并在过程中发现并修复了「面板复用被误报为加载超时」的真实缺陷。原始说明保留如下。

**官方面板嵌入的桌面复验未做**：该检查要求官方代理处于**运行**状态（Rust `sync_embedded_panel` 的 `Show` 分支要求 `RuntimeState::Running`，且面板 URL 取运行快照端口），因此必须先在真实环境启动官方服务。计划 §9「安全边界」规定「需要真实副作用时单独确认明确目标」，故本段停在此处，等确认后再做「启动 → 复验面板 → 停止还原」。

## 27. 阶段 6 收口（二）：官方面板嵌入复验与复用缺陷修复（2026-09-19 第十九段）

### 27.1 复验过程（真实链路，用户已授权查看/控制）

环境：`npm --prefix src-ui run dev` + 本次源码构建的 `src-tauri/target/debug/opencodex-desktop`（前台 tty），真实 HOME。开工前 `ocx status` = `Proxy: not running`。

从概览按下「启动 OpenCodex」→ 官方代理起来（`Proxy: running (PID 79255)`、`127.0.0.1:10100 LISTEN`）；任务卡显示「启动未完成 / 等待时间较长，后台仍会继续观察状态」——**等待超时没有写成失败**（FZ-43.2）；刷新状态后概览显示「运行中」；进入面板路由，官方面板**成功嵌入**。

### 27.2 本次实机通过项

| # | 检查 | 实测 |
|---|---|---|
| 1 | 面板真正嵌入 | 面板路由出现 `官方面板已嵌入主窗口`；AX 中出现**第二个 WebArea**，内容即官方页面（前往仪表板 / 仪表盘 / Codex 设置 / 提供方 / 模型 / 子代理 / 日志与调试 / 用量 / 存储 / 集成、`主题: 跟随系统`、`停止代理`、GitHub、状态 `在线`、版本 `2.50.0`、运行时间 `58秒`、内存可观测性 `RSS 283.6 MiB / 4.0 GiB`） |
| 2 | ~~原生子 WebView 不遮壳层~~ | ~~面板嵌入期间，壳层任务卡「启动未完成…」及其按钮**仍在 AX 中可见**~~ **——本项结论不成立，已由 §29 推翻：AX 只能证明 DOM 存在，不能证明视觉可见；受控像素 A/B 显示任务卡与面板矩形重叠的部分完全没有绘制。** |
| 3 | 点击不被吞 | 在官方页面按下「活跃提供方」标签，官方视图**真的切换**（出现 名称 / 适配器 / Base URL 表格） |
| 4 | 离开路由隐藏、再次进入恢复 | 离开到概览：官方标记 0；回到面板：官方标记恢复 |
| 5 | 成功不产生持久通知 | 启动成功后台通知中心仍为「全部 **0**」，空态正常（FZ-43.2 与 §24 的口径在实机成立） |
| 6 | 状态还原 | 审计结束按「停止」→ `ocx status` = `Proxy: not running`，与审计前一致 |

§19.4 的未通过项由此**转为通过**：§20 的坐标修复在真实桌面上被证实。

### 27.3 本次实机发现并修复的真实缺陷：面板复用被误报为「加载超时」

- **现象**：离开面板路由再进入后，官方面板**已经嵌入且可用**，但壳层 DOM 显示 `官方面板加载超时；主界面仍可继续操作，请重试或在浏览器打开。`（带「重试 / 在浏览器打开」）。该文案可被 AX 读到，属**假失败**（与「不用错误成功提示掩盖问题」相反方向的错误状态）。
- **根因**：原生子 WebView 在路由间被**复用**，`Show` 只做 `view.show()` 而不重新加载页面，因此 `on_page_load` 不再触发 → 前端 `panelReady` 永远为 `false` → 8 秒的 `scheduleLoadTimeout` 把它写成超时失败。
- **修复**（前端；`routes/PanelRoute.vue` + `stores/app.ts`）：
  1. 新增 `app.panelLoaded`：本轮运行中是否已观察到一次成功的官方页面加载；收到 `load: loading`（新的加载开始，含重载与端口变化后的重建）时重置为 `false`。
  2. `show` 分支改为三态：`!result.visible` → 停止加载并显示显式失败（此前会一直停在「正在嵌入」，导致「官方面板未能嵌入主窗口」这条文案**永远显示不出来**）；`visible && panelLoaded` → 判定为复用，直接就绪且**不启动**加载超时；其余 → 等待本次加载事件。
- **回归**：`tests/panel.test.ts` 新增「复用已加载面板不误报超时」与「未嵌入时显示显式失败」两条。
- **桌面复验**：修复前 re-enter → `timeout=1`；修复后 re-enter → `note=1`、**`timeout=0`**，官方内容在位。

### 27.4 证据

`.adg/work/evidence/task63-desktop-verify/`：`01`/`02` 通知中心空态（深/浅）+ AX；`03` 面板嵌入截图与 AX；`04` **修复前**复用假超时截图与 AX；`05`/`06` 修复后首次进入与复用进入的 AX；`07` 嵌入终态截图；`08` 点击穿透 AX。

### 27.5 门禁（本段）

`cargo fmt --check` / `clippy --all-targets -- -D warnings` / `cargo test` **320 passed / 0 failed**；`npm --prefix src-ui run typecheck` / `test -- --run`（**30 文件 / 121 测试**）/ `build` 全部通过。

### 27.6 仍未完成（不计为通过）

- **面板路由上通知层被原生子 WebView 遮挡，且面板路由没有通知入口**（§29，本轮新发现的真实缺陷，方案待定）。
- `operation_id` 与日志的精确互跳；其它通知事件源（同步冲突 / 更新可用 / 外部接管 / 环境门禁）。
- 设置分区、扩展页、托盘、诊断页的逐像素视觉对照。
- 系统通知权限与触达范围（候选）、最小窗口 900×600 与 960×640 是否统一（候选）。
- 本轮改动仍未提交、未建分支、未发布。

### 27.7 尝试：用隔离夹具验证「真实通知出现在桌面」（未完成，已还原）

为闭合「实际通知流程」这一阶段 6 要求，按计划 §9「使用隔离夹具」的口径，尝试在**隔离 HOME**（`/tmp/iso-home-notif-*`）下运行，并注入损坏的 `manager-state/notifications.json`，以触发 §22 的真实生产者 `notifications-store-unreadable`：

- 首次尝试直接预建 `manager-state` 导致启动失败：`create data root partition; File exists`——说明 `data_root::initialize` 不接受缺少元数据的既有分区目录（这本身是隔离夹具的正确用法约束，记录备查）。
- 改为让应用先自行创建数据根，再注入损坏文件；但此时**屏幕再次进入锁屏**（idle），窗口不可见、AX 不可读，无法继续观察。
- 已还原：结束隔离实例、删除临时夹具目录、确认真实数据根未被写入（`manager-state/` 仍无 `notifications.json`，`preferences.json` 时间戳未变），官方代理保持 `not running`。

**结论**：> **已完成（见 §28）**：本项已在下一段按同一夹具流程补做通过。

## 28. 阶段 6 收口（三）：真实通知流程与持久化恢复的桌面实机验证（2026-09-19 第二十段）

### 28.1 方法（隔离夹具）

按计划 §9「使用隔离夹具」：新建隔离 HOME（`/tmp/iso-notif-*`），先让应用自行创建数据根（避免 `data root partition; File exists`，见 §27.7），再向 `manager-state/notifications.json` 注入损坏内容 `{ broken json`，重启应用以触发 §22 的真实生产者 `notifications-store-unreadable`。全程只读写隔离数据根。

### 28.2 实测

| # | 检查 | 实测 |
|---|---|---|
| 1 | 真实事件产生真实通知 | 铃铛角标显示 **1**；通知中心计数：全部 **1** / 运行 0 / 同步 0 / 更新 0 / **系统 1** |
| 2 | 通知内容 | 「通知历史无法读取 · 通知历史文件无法解析，已按空历史启动；原文件保留在原处，可在诊断中心查看或清理。」，并带单条删除按钮 |
| 3 | 已读维度与角标 | 按「全部已读」→ 角标清空（未读只统计存活且未读且未解决，《数据与状态》§5.3） |
| 4 | 落盘 | `notifications.json` 由损坏内容被替换为合法 JSON：`version=1`、1 条，`level=warning`、`category=system`、`source=diagnostic`、`read=true`、`resolved=false`、`dedupe_key=store:notifications-unreadable`；文件模式 **0600**（原子写） |
| 5 | 重启恢复 | 重启应用后：全部 **1** / 系统 **1**，条目仍在并带「未解决」标签、角标为空（`read` 原样恢复）→《数据与状态》§5.7「重启后按 `read` / `resolved` / `deleted` 原样恢复」在真实桌面成立 |

### 28.3 结论

阶段 6 的「实际通知流程」由此取得实机证据，全链路为：**真实事件 → 真实通知 → 分类计数 → 已读 → 原子落盘 → 重启恢复**。同时验证了 §22 的损坏处理（不静默丢弃、以空历史启动并写一条**真实**失败通知）与 §23 的 `source` 维度在真实数据中落地为 `diagnostic`。

### 28.4 还原与边界

- 结束隔离实例并删除临时夹具目录；
- 确认真实数据根**未被写入**：`manager-state/` 仍无 `notifications.json`，`preferences.json` 时间戳仍为 `2026-09-18 21:44`；
- 官方代理保持 `Proxy: not running`（与审计前一致）。

### 28.5 证据

`.adg/work/evidence/task63-desktop-verify/`：`09-real-notification-fixture.png` + AX、`10-after-mark-all-read.png`、`11-after-restart-restored.png` + AX。

### 28.6 门禁

本段只做实机验证、未改代码；沿用上一段结果：`cargo fmt` / `clippy -D warnings` / `cargo test` **320 passed / 0 failed**，前端 typecheck / **121 测试** / build 通过。

## 29. 阶段 6 收口（四）：面板路由上的通知层被原生子 WebView 遮挡（发现真实缺陷，未修复）（2026-09-19 第二十一段）

### 29.1 起因：AX 可见 ≠ 视觉可见

§27.2 第 2 项用「任务卡仍在 AX 中可见」判定「原生子 WebView 不遮壳层」。但可访问性树只证明 DOM 元素存在，**不证明它被绘制在原生视图之上**；IMP `FZ-43.4` 恰恰警告「原生子 WebView 与 DOM 不是同一合成层，不得假定提高 `z-index` 就能盖住官方面板」。因此必须在面板已嵌入时做**受控像素 A/B**。

### 29.2 方法

面板嵌入、窗口固定 `[1010,264] 1180×760`、同一会话内：

1. **基线**：不做任何交互，间隔 4 秒两张截图，用于分离官方仪表盘自身的活动噪声（运行时间、内存读数）。
2. **A/B**：任务卡打开时截图 → 按任务卡「关闭」→ 关闭后截图。

统计算法：灰度绝对差 > 12 记为变化像素。

### 29.3 结果

| 区域 | 基线（无交互） | 任务卡打开 → 关闭 |
|---|---|---|
| 任务卡矩形（截图 px `540,12 → 2300,412`） | `mean_abs_diff = 0.00`、`max = 0`、变化 0/704000 | `mean_abs_diff = 0.68`、`max = 234`、**变化 30687/704000** |
| 「关闭」按钮小块（截图 px `2202,364 → 2288,412`，即屏幕 `[2111,446]` 43×24） | `mean_abs_diff = 0.00`、变化 0/4128 | `mean_abs_diff = 0.00`、`max = 0`、**变化 0/4128** |

结论：任务卡在面板路由上**只被部分绘制**——与原生子 WebView 矩形重叠的部分（含「关闭」按钮）**完全没有绘制**，其余部分正常。

### 29.4 判定

- **真实缺陷**：面板路由上，管理器的 DOM 通知层（任务卡；同一机制下的通知中心 / Modal / Toast）不与原生子 WebView 合成在同一层，**凡与面板矩形重叠处都被官方页面盖住**。`FZ-43.4` 要求的「通知宿主…正常路由与面板路由都生效」未满足。
- ~~**附带缺陷**：面板路由没有铃铛/通知入口~~ **——本条已于 §31 撤回：冻结原型本身就隐藏面板路由的顶栏**（`index.html` 第 1018 行 `body.panel-mode .topbar,…{display:none}`），生产不渲染顶栏与原型一致，**不是缺陷**；面板路由上真正需要成立的是「自动出现的 Toast / 任务卡可见」。
- **修正**：§27.2 第 2 项据此**改判为未通过**（原结论证据不足）。§27.2 第 1/3/4/5/6 项不受影响。

### 29.5 为什么没有当场修复

`FZ-43.4` 把该方案明确列为待定项（「确定通知 / Modal 的**原生承载或协调显示方案**」）。可选方向至少三种，且各有代价，属设计决策，不能在无确认时自行选定：

1. 把通知表面也做成**原生子 WebView**（与面板同一承载），彻底解决合成层级；
2. 通知表面打开时**临时隐藏 / 下移**面板子视图，关闭后恢复；
3. 将表面**重排到不与面板矩形重叠**的区域（面板路由下空间有限，可能牺牲可用性）。

选定后还需同步 `UI规范.md` 与 `FZ-43.4`（涉及权威文档修改，需单独登记 TASK）。

**倾向性证据（见 §30）**：本仓库对同一问题已有既定解法——`src-tauri/assets/panel-hub.js` 的注释明确写着「原生子 WebView 与主 WebView 不是同一合成层，管理器 DOM 的浮层会被官方面板盖住（R-17）」，因此**面板快捷浮层被注入到面板页内**而不是放在管理器 DOM 里。也就是说，方向 1（面板页内承载 / 注入）是本代码库已验证的既有模式；§29 的缺陷本质是**通知层没有享受同一处理**。

### 29.6 证据与还原

- 证据：`.adg/work/evidence/task63-desktop-verify/` 的 `12-panel-route-taskcard-open.png`、`13-panel-route-taskcard-closed.png`、`14-panel-route-baseline.png`（均为同会话同窗口）。
- 还原：审计后按「停止」→ `ocx status` = `Proxy: not running`、`10100` 无监听；真实数据根未被写入。

## 30. 阶段 6 收口（五）：面板快捷浮层实测，与 §29 的方向性证据（2026-09-19 第二十二段）

### 30.1 实测：面板快捷浮层确实渲染且可用

面板路由下对窗口做受控像素比对（用途同 §29：分离官方仪表盘自身活动噪声）：

| 观察 | 结果 |
|---|---|
| 浮层球体是否渲染 | 在面板视口右下角（`right:20px; bottom:18px`，38px）实测到玻璃圆球与其 `◉` 深色字形（暗像素 `(21,21,21)` / `(13,13,13)`），说明 `panel-hub.js` 已注入并绘制 |
| 浮层菜单能否打开 | 点击球体后，球体上方区域（截图 px `1980,800→2340,1400`）变化 **35813 / 216000** 像素（`mean=10.04`、`max=242`） |
| 同区基线 | 不做交互时同区变化 **0 / 216000**（`mean=0.00`） |

结论：`UI规范.md` / §25.4 中「面板快捷浮层（原生注入、玻璃材质、主题随动）」在真实桌面上成立。

**附带教训**：注入的元素**不出现在 AX 树中**（AX 里找不到「面板快捷操作」按钮），只能靠像素确认。这是对 §29 教训的补充——在面板路由上，**AX 既不能证明「可见」也不能证明「不存在」**，必须用像素。

### 30.2 与 §29 的关系：本仓库已有既定解法

`src-tauri/assets/panel-hub.js` 开头的注释写明：

> 为什么注入到面板页而不是放在管理器 DOM 里：原生子 WebView 与主 WebView 不是同一合成层，管理器 DOM 的浮层会被官方面板盖住（R-17）。因此浮层随面板一起渲染……

即：**同一问题在本仓库已有既定解法（注入面板页 / 原生承载）**，浮层正是这么做的；而**通知层（任务卡 / 通知中心 / Modal / Toast）没有享受同一处理**，于是出现 §29 的部分遮挡。这为 §29.5 的方向选择提供了强证据：方向 1 与本代码库既有模式一致，且已被证明可行（浮层就是活例），风险与未知量最低。

### 30.3 本段未做（登记）

- 面板**缩放**往返（需改 `界面缩放` 偏好并写盘，属真实配置写入，未在未确认下执行）。
- **加载失败**分支（需要人为构造失败，未做）。
- **外部浏览器兜底**（会打开用户系统浏览器，属可见副作用，未执行）。
- 浮层菜单内的「重载 / 概览 / 浏览器」逐项点击未逐个验证（仅验证球体渲染与菜单打开）。

### 30.4 还原

审计后按「停止」→ `ocx status` = `Proxy: not running`；真实数据根未被写入。

## 31. §29 缺陷的定位收敛与两种可行方案（2026-09-19 第二十三段）

### 31.1 撤回一条错误结论

§29.4 曾把「面板路由没有铃铛/通知入口」判为缺陷。核对**冻结原型**后撤回：原型 `index.html` 第 1018 行明确写着

```css
body.panel-mode .topbar,body.panel-mode .notice,body.panel-mode .menu-hint,
body.panel-mode .footer-note,body.panel-mode .state-pill{display:none}
```

顶栏（含铃铛、主题键）在原型的面板路由上**本来就是隐藏的**。生产的 `PanelRoute` 不含顶栏，与原型一致，**不是缺陷**。因此面板路由上真正必须成立的是：**自动出现的 Toast 与任务卡要能盖在官方面板之上**（原型里 `.notif-host` 是 `position:fixed;z-index:80` 的窗口级浮层，且 `pointer-events:none` + 子元素 `auto`，因此它在面板路由上会覆盖在面板之上，且不产生透明点击层）。

也就是说，§29 记录的部分遮挡**确定为原型复刻偏差**（不是产品口味问题）——修复方向由原型与 `FZ-43.4` 共同决定：**通知列必须位于官方面板之上**。

### 31.2 两种可行实现（均不需要改动原型语义）

| 方案 | 做法 | 代价 / 风险 |
|---|---|---|
| **A. 注入面板页** | 沿用本仓库既有模式（`panel-hub.js` 就是这么做、且已实测可用），把通知列（Toast + 任务卡）也注入面板子 WebView；管理器把通知状态经既有 `eval` 通道推入，动作经既有 `ocxd-panel://` 通道回传 | 需要**复制**通知列的标记与样式；与 IMP/计划「不每个页面各写一份同名视觉变体」的告诫相冲突，长期有两套实现需同步 |
| **B. 第二个原生子 WebView 承载** | 新增一个只挂载通知层的子 WebView，**尺寸精确等于当前可见通知面**（不存在面时隐藏，因此不留透明点击层），复用同一套 Vue 组件（无视觉复制）；管理器把 Toast / 任务进度（这些是纯前端状态、后端 IPC 取不到）经 Tauri 事件推给该 WebView | 需要新增 WebView 生命周期、状态推送与显隐/尺寸协调；改动面较大，但没有 UI 复制 |

**共同前提**：两种方案都需要同时核对「不遮挡点击」——方案 B 尤其要保证子 WebView 只覆盖真实存在的面，否则会变成 `FZ-43.4` 禁止的透明点击层。

### 31.3 本段未做

- 未实现任一方案：`FZ-43.4` 把该方案列为待定项，且两种方案分别在「避免视觉复制」与「改动面/状态通道」上各有取舍，属需要确认的设计决策；选定后还需要同步 `UI规范.md` 与 `FZ-43.4`（权威文档改动，按治理需单独登记 TASK）。
- 未执行 §30.3 登记的三项（面板缩放往返、加载失败分支、外部浏览器兜底）。

### 31.4 门禁

本段只改本记录，未改代码；沿用上一段：`cargo fmt` / `clippy -D warnings` / `cargo test` **320 passed / 0 failed**，前端 typecheck / **121 测试** / build 通过。

## 32. 阶段 6 收口（六）：FZ-43.4「协调显示」方案落地，面板路由通知层遮挡修复（2026-09-19 第二十四段）

### 32.1 方案选择：采用「协调显示」，不做视觉复制

§31 列出两条可行路线（A. 注入面板页；B. 第二个原生子 WebView 承载）。本段落地的是 **FZ-43.4 明文允许的第三种表述——「协调显示」**：

- 需求来源：`FZ-43.4` 原文把该问题列为「确定通知 / Modal 的**原生承载或协调显示方案**」。「协调显示」即在通知表面出现时**临时让出**面板子视图、表面消失后恢复。
- 语义来源：冻结原型 `.notif-host` 是 `position:fixed; z-index:80`、`pointer-events:none`、子元素 `auto` 的**窗口级浮层**，在面板路由上覆盖于面板之上（§31.1）。「协调显示」保持了「通知列盖在面板之上、且不产生透明点击层」这一原型语义。
- 取舍动机：方案 A 需要**复制**通知列标记与样式（与 IMP/计划「不每个页面各写一份同名视觉变体」相冲突）；方案 B 需要新增 WebView 生命周期与前端状态推送通道（改动面大）。「协调显示」**零 UI 复制、零新增承载**，且通知列本就位于窗口级 DOM，天然满足「不产生透明点击层」。

> 说明：本段同时否定了 §29.5 方向 3（把表面重排到不与面板重叠区）——面板路由下空间有限，重排会牺牲可用性。

### 32.2 实现

改动集中在 `src-ui/src/routes/PanelRoute.vue`，无新增组件、无标记复制：

1. 新增 `surfaceVisible` 计算属性：`app.notificationPanelOpen || app.toast !== '' || app.processProgressOpen || app.modal !== null`。
2. `watch(surfaceVisible, …)`：当表面出现（`true`）且尚未让出时，置 `panelHiddenForSurface = true` 并 `syncPanel('hide')`；当表面全部消失（`false`）且此前已让出时，复位并 `syncPanel('show')`。
3. `loadPanel()` 收尾：若加载时已有表面可见，同样立即 `syncPanel('hide')`，避免「进入面板路由瞬间仍被面板盖住」的窗口期。

隐藏/恢复走既有 `sync_embedded_panel`（`show` / `layout` / `hide`）命令，**未引入新的原生接口**；恢复时面板此前已加载完成，`app.panelLoaded` 为真，不会重新触发 §27.3 的「加载超时」计时。

### 32.3 实机 A/B 验证（基线噪声 0）

窗口固定 `[1010,264] 1180×760`、面板已嵌入、原生子 WebView 存在，同一会话内受控像素 A/B（灰度绝对差 > 12 记为变化像素）：

| 观察区域 | 修复前（§29.3） | 修复后 |
|---|---|---|
| 「关闭」按钮小块（截图 px `2202,364→2288,412`，4128 px） | **0 / 4128** | **559 / 4128** |
| 任务卡矩形（截图 px `540,12→2300,412`，704000 px） | 30687 / 704000 | **113842 / 704000** |
| 表面关闭后面板是否自动恢复 | —— | 是（官方面板标记 `markers=2`，同会话确认恢复） |
| 无交互基线 | 0 / 4128、0 / 704000 | 0（沿用同方法基线） |

结论：修复前被原生子 WebView 完全盖住、**变化 0 像素**的「关闭」按钮区域，修复后出现 559 像素变化（按钮可见并被绘制）——**§29 的部分遮挡已消除**；表面关闭后面板子视图自动恢复。

### 32.4 行为变化说明（需知悉）

- 在**面板路由**下，当通知中心 / Toast / 任务进度 / Modal 任一出现时，面板子视图会**先隐藏**、表面消失后立即恢复。这是 FZ-43.4 允许的「协调显示」语义，与原型「通知覆盖面板之上」一致（原型表现为浮层覆盖，本实现表现为让位后覆盖，用户可见结果都是「通知完整可见且可点击」）。
- **进入面板路由的瞬间若已有表面打开**（例如从别的路由带着任务进度切换过来），该表面会保持可见、面板在其后暂不显示，直到表面消失才显示面板。

### 32.5 证据

- `.adg/work/evidence/task63-desktop-verify/17-fixed-card-visible-panel-hidden.png`（表面可见、面板让出，任务卡完整绘制）
- `.adg/work/evidence/task63-desktop-verify/18-fixed-panel-restored.png`（表面关闭后面板恢复）
- 回归测试：`src-ui/tests/panel.test.ts` 新增「表面出现→隐藏面板 / 表面消失→恢复面板」用例（该文件 9 用例，前端套件 **122 passed**）。

### 32.6 治理与未做

- 本段改动**触及权威文档语义**（`FZ-43.4` 的待定项收敛、`UI规范.md` 需记录面板路由表面行为），按治理**单独登记 TASK** 后再回写（见下一段）。
- §30.3 登记的三项（面板缩放往返、加载失败分支、外部浏览器兜底）本段仍未做。

### 32.7 门禁

`cargo fmt --check` 通过；`cargo clippy --all-targets -D warnings` 通过；`cargo test` **320 passed / 0 failed**；前端 `typecheck` 通过、`test` **30 文件 / 122 用例** 通过、`build` 通过。

## 33. 阶段 1 / 7 回写（补）：面板路由承载方案收敛的权威回写与治理收口（2026-09-19 第二十五段）

### 33.1 为什么需要单独的 TASK

§32 的实现把 `FZ-43.4` 的「原生承载或协调显示方案」待定项**收敛为协调显示**，并需要在《UI规范》登记面板路由表面行为——两者都是**权威文档语义修改**，按治理须登记独立 TASK 后再回写，不能借实现任务顺带改权威事实。

### 33.2 登记与回写

- 登记 `TASK-OPENCODEX-DESKTOP-64`（`IMP-OPENCODEX-DESKTOP-01`，owner `Ezio`）：`write_scope = docs/02-项目核心/** + docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md`；`required_checks` 五项评审；`acceptance_bindings` 复用 `AC-04` / `AC-12`；`fact_writeback.mode = required`，目标 `ui-spec`（`docs/02-项目核心/UI规范.md`，before `9e8accc0…`）与 `imp-01`（`IMP-OPENCODEX-DESKTOP-01.md`，before `459a32ce…`）。
- 回写 `IMP-OPENCODEX-DESKTOP-01.md` FZ-43.4：就地修订承载方案为**协调显示**（表面可见时临时让出面板、消失后恢复；通知宿主仍为窗口级壳层 DOM，不重排为透明点击层、不复制面板页标记），更新候选说明并追加变更记录行。
- 回写 `docs/02-项目核心/UI规范.md`：新增 §4.7「面板路由的通知层（协调显示）」，并在 §4.6 指向 §4.7、在 §8 映射表登记该行。
- `project-refresh` 已把 `IMP-OPENCODEX-DESKTOP-01` 源对象重建（`reconciliation.updated`，`source_digest → 64d07b74…`）。

### 33.3 交付物与校验

| 对象 | 结果 |
|---|---|
| `TASK-OPENCODEX-DESKTOP-64` | `completed` / `succeeded`（revision 3） |
| `RCP-OPENCODEX-DESKTOP-64` | `after_digest`：`ui-spec → 4c8ff89b…`、`imp-01 → 64d07b74…` |
| `CERT-OPENCODEX-DESKTOP-64` | `ee7289cd…`；`RCP`/`CERT` 与 `TASK` 原子收口 |

校验：`project-refresh --root . --force-board` 后 `project-check` = `state: valid`、`source_binding.healthy = true`、`board.state = valid`、`warnings = 7`（均为既有 `TASK-25/30/57` 目录警告，未新增）。

### 33.4 门禁

本段只改权威文档与 `.adg` 治理对象，未改代码；沿用 §32：`cargo fmt --check` / `clippy -D warnings` 通过、`cargo test` **320 passed / 0 failed**，前端 `typecheck` / **30 文件 / 122 用例** / `build` 通过。

## 34. 阶段 7 交付说明（最终版，取代 §25）（2026-09-19 第二十六段）

> §25 写于 §26–§32 之前；其后完成了面板嵌入复验、通知持久化实机复验、面板浮层实测与 §29 遮挡修复及其权威回写。本节是**当前**交付说明，取代 §25；§14 / §25 保留为历史，不再作为现状依据。本节只登记实际结果、证据位置与缺口，不冒充验收证书。

### 34.1 文档回写清单与相互引用

| 文档 | 本轮回写内容 | 相互引用 |
|---|---|---|
| `docs/02-项目核心/领域模型.md` §16（+§16.1） | `Notification` 实体、字段语义（含 `source` / `expires_at`）与「操作任务 / 技术日志 / 确认请求」边界 | 状态迁移引用《数据与状态》，触达规则引用 FZ-43 |
| `docs/02-项目核心/数据与状态.md` §5（含 §5.7） | 六类渠道职责、操作进度与真实终态、读/解决/删除/过期维度、去重排队、收起、持久化与恢复 | 不复制实体字段表与视觉尺寸 |
| `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` FZ-43.1~43.4 | 渠道与时机、触达/去重/排队、关闭/超时/清理、跨路由与承载；§33 把 FZ-43.4 承载方案**收敛为「协调显示」**并附变更记录行 | 就地修订，未另设冲突版本 |
| 协作包 `…/04-需求分析产物/场景与验收矩阵.md` §7 | S-11~S-16 六类渠道与组合边界场景 | 关联既有 AC，未新建需求级 AC |
| `docs/02-项目核心/UI规范.md`（新增）+ 该目录 `README.md` 登记 | 窗口与布局、设计令牌、基础控件、通知组件、操作规则、键盘与可访问性、边界、映射；§15 补页面级 `.tag`；§33 补 §4.7「面板路由的通知层（协调显示）」与映射行 | 视觉参数唯一维护处 |
| `docs/02-项目核心/契约字段.md` | 未受影响 | — |
| `docs/01-需求管理/需求/DMD-OPENCODEX-DESKTOP-MANAGER.md` | 未受影响（无范围 / 验收变更） | — |

权威文档回写共两次，均经独立 TASK 收口：§15/`TASK-63`（页面级 `.tag` 补进《UI规范》§3）、§33/`TASK-64`（FZ-43.4 承载方案收敛 + 《UI规范》§4.7）。其余修复均为「代码向已确认契约收敛」，未改权威语义。

### 34.2 UI 规范与原型—token—组件映射

`docs/02-项目核心/UI规范.md` 为唯一视觉参数维护处；映射覆盖外壳、侧栏、标题栏、内容页、滚动条、通知令牌、通知中心/条目/Toast/任务卡/对话框、面板浮层、面板路由通知层、页面级状态标签。已实现部分与冻结原型在设计令牌与计算样式上逐项一致（§8.2、§9、§10、§15）。

### 34.3 组件化与 CSS 整理范围

- 新增 `NotificationCenter.vue`、`NotificationItem.vue`、`ToastHost.vue`、`TaskProgressHost.vue`、`BrandMark.vue`；`AppModal.vue` 收敛为确认 / 详情对话框。
- `tokens.css` 收敛公共设计值（含通知令牌与 reduced-motion）；`base.css` 收敛重置、布局、控件与反馈展示。
- §15 补页面级 `.tag` 基础规则等并移除死类名；§20 修 `.app-shell` 行分配使壳层与原型 `y≈30` 对齐并通过 Rust 坐标校验；§27 修 `app.panelLoaded` 消除面板重入的**假「加载超时」**；§32 在 `PanelRoute.vue` 加 `surfaceVisible` 协调显示（无 UI 复制）。
- 通知保持单一事件来源（后端 `NotificationStore` + `manager-state/notifications.json` 持久化），前端只做投影；无新增隐式旁路接口、无整页重写。

### 34.4 原型复刻完成 / 缺口清单

| 区块 | 状态 |
|---|---|
| 壳层（单一底面、圆角内容页、无分栏竖线、覆盖式滚动条） | 完成（§20 修正控制行行分配） |
| 宽窄栏图标列对齐（15/25px，不跳动） | 完成 |
| 通知中心 / 条目 / Toast / 任务卡 / 收起到铃铛 / 空态 | 完成（Web + 桌面实机证据，§26） |
| 面板快捷浮层（原生注入、玻璃、主题随动） | 完成（§30 桌面像素实测：渲染 + 菜单开合） |
| 官方面板真正嵌入 + 正常/面板路由通知层 | 完成（§27 嵌入复验；§32 协调显示消除遮挡，A/B 证实） |
| 设置 10 分区、扩展 2 分区、诊断 2 分区：结构、标签、逐行控件、选项、校验 | 完成（差异均可归因于 AC-06 / AC-10 与「不使用原型固定数据」） |
| 设置各分区与扩展页的**逐像素**视觉对照 | 缺口（当前到结构、标签、控件、选项、校验行为） |
| 托盘预览、日志 / 诊断页视觉细节 | 部分完成（结构证据 + 截图，未逐像素对照） |

### 34.5 Web 与桌面结果（分别，均为本次实跑）

- **Web（阶段 5）**：`npm --prefix src-ui run typecheck` / `test -- --run` / `build` 全绿（**30 文件 / 122 用例**）；审计夹具只在开发环境按显式 `?__audit=1` 安装，被 `import.meta.env.DEV` 排除出正式构建。证据 `.adg/work/evidence/task61-web-audit/`（115 文件）+ `task63-notification-empty/`（3 文件）。Web 阻塞项已清零。
- **桌面（阶段 6）**：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿（**320 passed / 0 failed**）；`npm --prefix src-ui run build` 通过。**编译与契约通过不等于原生流程通过**，故另列实机证据：
  - 已获实机证据：原生控制行 / 品牌 / 导航层级、单一玻璃底面、内容圆角与描边、宽窄栏图标列、主题切换、最小窗口 `960×640`、面板窄栏、概览模块卡入口（§19）；通知中心**空态**与「清除演示种子 / 空集启动」（§26）；官方**面板嵌入** + 面板 reentry 无假超时（§27）；**真实通知 → 计数 → 全部已读 → 原子落盘 → 重启恢复 `read`**（§28）；**面板浮层渲染与菜单开合**（§30）；**§29 遮挡修复**的 A/B 像素证据（§32）。
  - 证据：`.adg/work/evidence/task63-desktop-audit/`（15）、`task63-desktop-verify/`（18：`01…18`）、`task63-notification-empty/`（3）。

### 34.6 §26–§32 期间修复的契约 / 实现缺口

| 段 | 缺口 | 处理 |
|---|---|---|
| §27 | 面板重入误报「加载超时」 | 引入 `app.panelLoaded`，已加载完成的面板不重新计时 |
| §29 | **面板路由上通知层被原生子 WebView 部分遮挡**（关闭按钮区域像素变化 0/4128） | §32 按 FZ-43.4「协调显示」：表面可见时让出面板、消失后恢复；A/B 关闭按钮 0/4128 → 559/4128、任务卡矩形 30687 → 113842/704000 |
| §31 | §29.4 误把「面板路由无铃铛」当缺陷 | 撤回：冻结原型 `body.panel-mode` 本就隐藏顶栏，生产一致，非缺陷 |

（§18 / §20 / §21 / §22 / §23 / §24 的修复见 §25.6，仍有效。）

### 34.7 未提交改动保留说明

HEAD 仍为 `d1858a7`；工作树 **72 项**改动 / 未跟踪，涵盖 `src-ui/*`、`src-tauri/*`、`docs/02-项目核心/*`、`docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md`、协作包材料与 `.adg/` 治理对象。**未提交、未建分支、未发布、未做全局同步**；未执行高风险真实写入；开工前已有的未提交改动保留未回退。

### 34.8 待用户确认的候选（未定稿）

1. Toast / 任务区「窗口内顶部」与「右上」的最终位置。
2. ~~通知材质（纯透明 / 轻磨砂 / 实底）与颜色滤镜打样结论。~~ **已确认**：`docs/02-项目核心/UI规范.md` §2.2 记录「中性轻磨砂 + 不叠颜色滤镜」为正式材质，材质与滤镜**不再属于候选**；§35 已把生产实现对齐并实测（本项从候选移出）。
3. 系统通知的开关、权限与触达范围；终态停留时间与日志关联方式。
4. 原型最小窗口 `900×600` 与 Tauri 配置 `960×640` 是否统一。**已有实机结论**：当前配置会把 `900×600` / `800×500` / `600×400` 一律夹到 **`960×640`**（§19.2 #7）；未静默改动平台约束，等确认。
5. 面板路由通知层承载方案（§33 已收敛为「协调显示」并回写 FZ-43.4 /《UI规范》§4.7）——如后续希望改为注入面板页 / 第二原生子 WebView，需新登记 TASK 后推翻本文口径。

### 34.9 明确不计为完成

1. `operation_id` 与日志的精确互跳（进程层无跨通道操作标识；不伪造关联）。
2. 其它通知事件源（同步冲突 / 更新可用 / 外部接管 / 环境门禁）未接线。
3. 设置分区、扩展页、托盘、诊断页的**逐像素**视觉对照。
4. 面板**缩放**往返、**加载失败**分支、**外部浏览器兜底**（§30.3，均属可见副作用或真实配置写入，未在未确认下执行）。
5. 系统通知的权限与触达范围验证（候选，见 §34.8 #3）。

### 34.10 一致性与治理状态

- `project-check --root .` = `state: valid`（`source_binding.healthy = true`、`board.state = valid`、`warnings = 7`，均为既有 `TASK-25/30/57` 目录警告，未新增）。
- `TASK-OPENCODEX-DESKTOP-64` / `RCP-…-64` / `CERT-…-64` 原子收口，`verify_closed_task` 通过。
- 阶段 1–7 已按计划推进；遗留项集中在 §34.9（均为已登记、待确认或需真实副作用授权）。

## 35. 回写（补）：通知玻璃材质向已确认规范对齐（2026-09-19 第二十七段）

### 35.1 起因

用户指出 `docs/02-项目核心/UI规范.md` §2.2 已把通知材质**确认**为「中性轻磨砂 + 不叠颜色滤镜」。核对发现**生产 CSS 未对齐该已确认口径**：

| 漂移 | 位置 | 现象 |
|---|---|---|
| 颜色滤镜未关闭 | `tokens.css`（亮/暗） | `--notif-tint-a/b` 仍是冷蓝/淡青/深蓝/墨绿，不是 `transparent` |
| 表面仍叠渐变 | `base.css` `.notification-panel`、`.process-progress`（任务卡） | `background` 仍是「两条 `linear-gradient` + `--glass-fill`」 |
| Toast 走实底 | `base.css` `.toast` | `background: var(--panel)`（不透明、无模糊），未用统一轻磨砂 |
| 残留打样物 | `tokens.css` | 仍有 `--glass-fill-clear`/`--glass-blur-clear` 与「材质与滤镜为打样中候选」注释 |

**为何此前未发现**：§8.2 的原型—生产对照只比了**令牌取值**（`glass-fill .52` / `blur 8px`），没有比 `.toast`/`.process-progress`/`.notification-panel` 的**合成背景**，因而漏掉了实底与渐变层。

### 35.2 依据

- `UI规范.md` §2.2（材质与滤镜令牌，已确认）与 §4.1（给出通知三表面的确切 CSS：`background: var(--glass-fill)` + `--glass-blur-notif`）。
- 冻结原型 `index.html`：材质默认 `glass`、滤镜默认 `off`（`NOTIF_MATERIAL_DEFAULT='glass'` / `NOTIF_TINT_DEFAULT='off'`，第 4895、4904 行），与已确认口径一致；打样 chips 仅属原型审计工具，生产不暴露。
- `src-tauri/assets/panel-hub.js`：同源值 `rgba(244,245,249,.52)` / `rgba(34,36,42,.56)` + `saturate(1.5) blur(8px)`（§9 要求与通知同源）。

### 35.3 改动（代码向已确认契约收敛，未改权威语义）

- `src-ui/src/styles/tokens.css`：`--notif-tint-a/b` 亮/暗均置 `transparent`；删除未再使用的 `--glass-fill-clear` / `--glass-blur-clear`；注释由「打样中候选」改为「已确认，见 UI规范 §2.2」。
- `src-ui/src/styles/base.css`：`.toast` / `.process-progress` / `.notification-panel` 的 `background` 统一为 `var(--glass-fill)`（去除滤镜渐变层），并统一 `-webkit-backdrop-filter` / `backdrop-filter: var(--glass-blur-notif)`（`.toast` 原缺模糊，一并补上）。

### 35.4 验证（构建期 CSS 计算样式）

方法：`chrome-headless-shell` + `playwright-core`，注入构建产物 CSS，读取三个表面的 `getComputedStyle`：

| 表面 | 亮色 | 暗色 |
|---|---|---|
| `.toast` / `.process-progress` / `.notification-panel` | `background-color: rgba(244,245,249,0.52)`、`background-image: none`、`backdrop-filter: saturate(1.5) blur(8px)` | `rgba(34,36,42,0.56)`、`none`、`saturate(1.5) blur(8px)` |

- 三表面**完全一致**，且与 §9 `panel-hub.js` 同源值一致；`--notif-tint-a/b` = `transparent`；构建 CSS 中已无 `158deg` 渐变残留。
- 证据：`.adg/work/evidence/task64-glass-material/computed-styles.json`。

### 35.5 门禁与性质

- 前端 `typecheck` 通过、`test` **30 文件 / 122 用例**、`build` 通过；未改 Rust，沿用 `cargo test` **320 passed / 0 failed**。
- 性质：**代码向已确认契约收敛**（§25.1 口径），不改权威语义，故**无需新开 TASK**。

### 35.6 未做 / 登记

- **桌面实机验证**：本轮审计时 Vite dev 服务未在运行，且玻璃半透明不适合用像素法判定；已用构建期 CSS 的计算样式作为证据。如需桌面实测，需重启 Vite + 开发版后再做（可见副作用，未擅自启动）。
- §34.8 第 2 项（通知材质候选）据此**移出候选**。

## 36. 桌面实机：通知玻璃材质的实机复核（2026-09-19 第二十八段）

### 36.1 环境

重启 dev 链：`npm --prefix src-ui run dev`（`127.0.0.1:5173`，实测 dev 服务已返回更新后的令牌：`--notif-tint-a: transparent`、`base.css` 无 `158deg`）+ 当前源码构建的 `src-tauri/target/debug/opencodex-desktop`（dev 加载 `devUrl`）。窗口 `pos=[1010,264] size=[1180,760]`、亮色主题。开工前有一个 Release 包实例占用单实例锁，已 `quit` 后启动 dev 二进制。

### 36.2 方法：同坐标 A/B（关闭 vs 打开）

以桌面实机可见的**任务卡 `.process-progress`**为对象（与通知中心/Toast 同属 §2.2 / §4.1 三个表面）：先在任务卡可见时截图，再 `AXPress`「收起任务卡到铃铛」后同坐标截图，按像素比对。

| 观察 | 结果 |
|---|---|
| 任务卡屏幕矩形 | `x 1744–2189, y 304–574`（截图 2x） |
| 纯白底层处：关闭 → 打开 | `(255,255,255)` → `≈(248–249, 249–250, 251–252)` |
| §2.2/§4.1 预测（`α.52` × `fill(244,245,249)` over white） | `(249.3, 249.8, 251.9)` |
| 通道平衡 | `B−R ≈ 2` → **中性**（无冷蓝/淡青滤镜） |
| 透明性（底层结构是否透过） | `corr(closed_luma, open_luma)=0.351`、`slope=0.278`；不透明实底应为常数填充（`corr≈0`） |

结论：桌面实机上任务卡为**中性轻磨砂玻璃**——观测混合值与规范预测一致、通道接近等值、且底层内容透过表面（叠加 `blur(8px)` 平滑），与旧实现（`.toast` 实底 `#fff`、`.process-progress`/`.notification-panel` 叠冷色渐变）明显不同。

### 36.3 证据与还原

- 证据：`.adg/work/evidence/task64-glass-material/` 的 `20-desktop-taskcard-open.png`、`21-desktop-taskcard-closed.png`、`desktop-taskcard-blend.json`（另见 §35 的 `computed-styles.json`）。
- 还原：`ocx stop` → `ocx status` = `Proxy: not running`；dev 应用与 Vite 已退出；真实数据根 `preferences.json` mtime 仍为 `2026-09-18 21:44`、未生成 `notifications.json`（未写入真实配置）。

### 36.4 未做 / 限制（如实登记）

- **通知中心面板未能在本轮以 AX 打开**：本轮 `AXPress` 无法触发顶栏铃铛（同名 `@click` 的主题按钮可触发、铃铛无效），且本机无输入合成权限、`cliclick`/CUA 不可用，故未取到通知中心面板的桌面像素。通知中心与任务卡共用同一 `--glass-fill` / `--glass-blur-notif`（构建 CSS 已证三表面取值一致），任务卡实测即为该材质的桌面代表证据；如需通知中心自身像素，需在有输入权限或 CUA 的环境补测。
- 未逐像素比对 Toast 单独表面（其与任务卡同源同值）。

### 36.5 门禁

本段只做实机复核、未改代码；沿用：前端 `typecheck` / **30 文件 / 122 用例** / `build` 通过，`cargo test` **320 passed / 0 failed**，`project-check` = `state: valid`。

## 37. 缺陷修复：英文状态文案泄漏 + 首次启动不检测完成（2026-09-19 第二十九段）

用户报告两个问题：界面出现英文状态（`at_risk`、`(bundled)`、`ready`、「官方 status」）；点击启动后任务卡一直「等待」、检测不到完成，需关掉再点一次才成功，期望「点击启动 → 检测成功 → 自动退出通知」。

### 37.1 根因

| 问题 | 根因 |
|---|---|
| 英文文案泄漏 | 后端/官方英文枚举被直接渲染：`TaskProgressHost` 的运行时映射缺 `at_risk`/`external_takeover` 等 → 回退原始值；`mock.ts` 把 `runtime_label`（`bundled`）原样拼进备注；`OverviewRoute` 直接渲染 `runtime_label` 与 `facts.health`（`healthy`…）；`TrayRoute` 标签缺 `at_risk` 等；备注文案含 `status`/`ready`/`restore` |
| 首次启动不检测完成 | **前端从未收到状态事件**：`src-tauri/capabilities/` 不存在 → `gen/schemas/capabilities.json` 为 `{}` → 无 `core:event:default` 等权限 → `@tauri-apps/api/event.listen` 被 ACL 拒绝（异常被 `catch` 吞掉）。于是状态只在挂载/动作后各拉一次；启动后那一次拉取太早（仍 `at_risk`），此后不再刷新，直到用户第二次点启动触发新的拉取 |

> 复现证据（改前）：代理已 `running/health ok`（`ocx status` 与其 app 同源环境均返回 `running=True`），桌面应用却在 21s 内持续显示「存在风险 / 当前状态：at_risk」，随后转为「启动未完成」。确认是**前端未刷新**，而非后端映射错误（映射在 `proxy.running=true` 时返回 `Running`）。

### 37.2 修复

- **补 Tauri 能力（根因）**：新增 `src-tauri/capabilities/default.json`（`windows: ["main"]`，`permissions: ["core:default","process:allow-exit"]`），恢复核心事件订阅（`status-snapshot-changed`）与受控退出。
- **兜底轮询**：`stores/app.ts` 在进程动作进行中每 `1.5s` 拉一次状态快照，事件通道不可用时任务卡也能在真实迁移后收口。
- **成功后自动退出通知**：`finishProcessAction` 完成即安排 `2.6s` 自动关闭；失败/超时**保留**待处理（`FZ-43.3`）。
- **文案统一中文**：新增 `src-ui/src/labels.ts`（运行时/健康/连接/操作/运行时来源的中文映射，未知值回退中文占位），并接入 `TaskProgressHost`、`mock.ts`、`OverviewRoute`、`TrayRoute`、`useAppController`（恢复确认弹窗）、`stores/app.ts`；`TaskProgressHost` 的「等待 ready 健康检查」→「等待就绪健康检查」；备注 `status/ready/restore` 与 `provider/token/headers/env` 等改为中文。

### 37.3 实机验证（屏幕解锁后，dev 链 + 当前源码构建）

| 检查 | 结果 |
|---|---|
| 英文文案 | 改后概览显示「官方状态报告启动存在风险…（运行时来源：内置）」——原 `at_risk`、`(bundled)`、「官方 status」「ready」均已消失 |
| 首次启动检测 | 停止态点「启动 OpenCodex」→ t≈2s 概览已「运行中」，无需二次点击 |
| 任务卡出现 | 24 帧快速截图：任务卡帧暗像素 `491 → 1806`（出现），动作完成后回落，**无需用户点关闭** |
| 自动退出 | 动作完成后任务卡自动消失（AX 无「收起任务卡到铃铛」） |
| 任务卡文案 | 审计夹具 `?__audit=1&scenario=task-busy|task-success`：`正在启动 OpenCodex`、`等待就绪健康检查`、`启动已完成` 全中文，无 `ready` |
| 停止流 | 点「停止」→ 代理停止、卡片出现后自动退出、界面回到「启动 OpenCodex」 |

### 37.4 回归与门禁

- 新增 `src-ui/tests/process-progress-labels.test.ts`（文案映射 + 成功后自动退出 + 失败不自动退出）；更新 `process-action.test.ts` 的 health 期望为中文。
- 前端：`typecheck` 通过、`test` **31 文件 / 126 用例**、`build` 通过。Rust：`fmt --check`、`clippy -D warnings`、`cargo test` **320 passed / 0 failed**（新增能力后重新构建通过）。

### 37.5 还原

`ocx stop` → `Proxy: not running`；dev 应用与 Vite 已退出；真实数据根 `preferences.json` mtime 仍为 `2026-09-18 21:44`。

### 37.6 说明 / 边界

- 剩余英文仅限：官方命令名（`ocx`/`ocxd`）、产品与协议专有名词（OpenCodex、WebDAV、MCP、Skills、macOS、GitHub 等）、算法/字段标识（Argon2id、AES-256-GCM、socket、UID）、以及原型托盘预览刻意复刻的 macOS 菜单名（File/Edit/View…）。如需进一步本地化这些，请确认。
- 本轮为**缺陷修复**（代码向意图收敛），另按治理单独登记一个 TASK 把「界面文案只用中文」写进《UI规范》。

### 37.7 治理回写

`TASK-OPENCODEX-DESKTOP-65`（`write_scope: docs/02-项目核心/**`，`fact_writeback: ui-spec`）已把上述口径写入 `docs/02-项目核心/UI规范.md` **§10「文案语言」**（界面只用中文；官方英文枚举须经 `src-ui/src/labels.ts` 映射；未知回退中文占位），`RCP`/`CERT` 原子收口，`project-check = valid`。

## 38. 重开软件逐路由审计（2026-09-19 第三十段）

用户要求重开软件审计。dev 链 + 当前源码构建，逐路由检查（截图见 `.adg/work/evidence/task65-audit/`）。

### 38.1 通过项

- **全中文**：概览/面板/拓展/日志/设置五路由无英文状态泄漏；`存在风险`、`运行中`、`运行时来源：内置`、`官方状态报告…官方恢复（restore）` 等均为中文。
- **导航与主题**：五个导航项、浅色/深色/跟随系统、顶部状态与动作均正常。
- **官方面板嵌入**：运行态下嵌入主窗口成功，官方页面自身 UI 正常显示。
- **启动/停止**：点启动→`运行中`（首次即成功）；任务卡出现并在完成后自动退出（帧序列暗像素 `396 → 1661`）。

### 38.2 发现并修复

- **面板模式侧栏导航失去可访问名称**：`.app-shell.panel-mode` 隐藏 `.nav-label` 后，导航按钮只剩图标且**无 `aria-label`**（此前仅 `title`）→ 窄栏下 AXButton 名称为空，屏幕阅读器无法命名导航。修复：`AppSidebar.vue` 导航按钮加 `:aria-label="item.label"`；实机复验窄栏下五个按钮均恢复名称（`概览/面板/拓展/日志/设置`）。

### 38.3 观察（未改，登记）

1. 概览「数据目录 —」「版本 —」：官方 `status` 未返回 `dataRoot`（2.50 schema 无该顶层字段）时显示占位；可考虑回退到管理器已知的数据根/已安装版本。
2. 拓展页 Skills 说明含英文——来自各 Skill 自身 `SKILL.md` 元数据（用户内容），非界面文案。
3. 日志页「调用官方 ocx doctor」、设置页「官方 shim ocx ensure」含命令/技术词，按 §10 规则保留。
4. 受本机输入合成权限限制，铃铛/通知中心无法用 AX/自动化点击打开，未做该表面的实机点击复验（Web 审计夹具已验证其渲染）。

### 38.4 门禁

前端 `typecheck` 通过、`test` **31 文件 / 126 用例** 通过；还原：`ocx stop` → `Proxy: not running`、dev 应用与 Vite 退出。

## 39. 完整交互审计与修复（2026-09-19 第三十一段）

用户反馈「点导出卡 UI、打开导出无反应」，要求做一轮完整交互审计而非表面测试。

### 39.1 方法

- 构建**快速 AX 派发器**（Swift + AXUIElement，`/tmp/axpress`）：JXA 逐元素遍历太慢（~2s/次），无法在操作窗口内测「是否卡顿」；Swift 版 ~0.01s 完成派发。
- **主线程响应延迟**作为卡顿指标：操作期间用轻量 AX 查询（读取窗口几何）计时，基线 ~0.19s。

### 39.2 确认并修复的缺陷

**① 打开导出目录（及未预载分区的受管目录入口）无反应**
- 根因：`SettingsRoute.openManagedTarget` 直接读 `app.managedPathTargets`，但该投影只在「扩展管理 / 关于」分区加载；在「配置迁移」等分区为 `null` → 路径空 → 静默失败。失败兜底文案还会**泄漏英文 key**（`exports不可用；已保留当前状态。`）。
- 修复：新增 `app.openManagedPathByKey(key)` —— 按需 `loadManagedPathTargets()`、目标缺失时用中文名兜底（`MANAGED_PATH_LABELS`）、打开失败给中文提示；`SettingsRoute` 改用它。
- 验证：点「打开导出目录」后 Finder 前置窗口 = `…/com.gzers.opencodex.desktop/exports/`。

**② 同步命令阻塞主线程 →「点导出卡 UI」**
- 实测：点「运行 Doctor」后，紧接的 AX 查询延迟从 **0.19s 突增到 1.32s**（主线程被同步命令内联执行阻塞约 1.1s）。导出走 Argon2id（256 MiB × 3 迭代），debug 下单次约 **1–1.5s**（`cargo test --lib migration` 12 项 18.7s），即同一个卡顿成因。
- 修复：新增 `commands::run_blocking`（`spawn_blocking` 包装），把耗时命令移出 IPC/主线程并改为 `async`：`export_migration`、`import_migration`、`run_doctor`、`read_logs`、`create_upgrade_backup`、`create_restore_backup`、`extension_config`、`list_extensions`、`execute_extension_write`、`import_skill_archive`、`cleanup_local_logs`、`initialize_data_root`、`validate_data_root_structure`、`switch_data_root`、`set_opencodex_home_config`。
- 验证：改后点「运行 Doctor」，AX 延迟稳定 **~0.19s**（不再卡）。

### 39.3 审计覆盖与回归

- 覆盖面：五个路由（概览 / 面板 / 拓展 / 日志 / 设置）+ 设置全部 10 个分区 + 关键动作（启动/停止、刷新状态、运行 Doctor、打开导出目录、主题切换、通知铃铛）。
- 回归：10 个分区与三路由均正常渲染（AX 节点 27–129）；`cargo test` **320 passed**；前端 `typecheck` / **126 用例** / `build` 通过；`cargo fmt` / `clippy -D warnings` 通过。

### 39.4 其他观察（未改）

1. Doctor 报告为官方原文（英文行，如 `ok CODEX_HOME: …`）——属诊断事实，按《UI规范》§10 保留原文；UI 层已给中文标题与说明。
2. Toast 不出现在 AX 树中（视觉存在）；自动化难以对「提示是否弹出」做像素断言，需人工确认。
3. 受本机权限限制，无法合成真实鼠标/键盘输入，故「导出（带口令）」本身未能自动化复现，只验证了其阻塞成因已被消除。

### 39.5 门禁

前端 `typecheck` / **31 文件 / 126 用例** / `build`；`cargo fmt --check` / `clippy --all-targets -D warnings` / `cargo test` **320 passed**。

## 40. 穷尽式交互审计（2026-09-19 第三十二段）

用户要求「对每一个按钮点击、每一个流程都做测试」的完整审计，不接受表面测试。

### 40.1 方法

- **可用真实输入的自动化链**：Swift `AXUIElement` 快速枚举 `axenum`（~0.3s/次，替代 JXA 的 ~9s）；**真实 CG 鼠标点击/键盘** `click`/`type`（本机合成权限实际可用，先前误判为不可用）；**主线程延迟探针** `axwatch`（稳态 worst≈6.5ms，>50ms 记为超限）；**连拍差分**（Pillow `ImageChops`）判定短时浮层/Toast，规避 `screencapture` 抓帧时序问题。
- 覆盖面：五路由全部 + 设置 10 分区逐项控件 + 关键后端流程（迁移导出/导入、升级/应用备份、Doctor、日志/通知清理、数据根初始化/切换、OPENCODEX_HOME、WebDAV、扩展落点、启停/重启、通知中心、外链与本地文档）。

### 40.2 确认并修复的缺陷

**① 打开 OPENCODEX_HOME 无反应（概览「运行详情」与设置「安装配置」）**
- 根因：`commands/workspace.rs::open_managed_path_with_roots` 白名单仅含 6 个受管目录 + 数据根 + OS `$HOME`；真实 OPENCODEX_HOME（inside 模式 `<数据根>/opencodex-home`）未命中 → `Err(NotConfigured)`，前端仅弹 AX 不可见的 toast。
- 修复：白名单额外纳入由数据根运行配置解析出的真实 OPENCODEX_HOME；更新并加严单测。
- 验证：点击后 Finder 打开 `…/com.gzers.opencodex.desktop/opencodex-home/`。

**② 偏好保存整链失效（开关 / 选择器 / 缩放点后回滚）**
- 根因：命令 `save_preferences(preferences: Preferences)` 收领域结构（serde `snake_case` + `#[serde(default)]`），WebView 发的是 camelCase DTO；camelCase 字段被当作未知字段丢弃、默认值补齐 → 每次保存都写成默认值、界面随即回滚。
- 证据：Rust 单测证实 `serde_json::from_value::<Preferences>({"interfaceScale":150})` 得到 `100`；点控件后 `preferences.json` mtime 变化但内容不变。
- 修复：命令改收 `PreferencesDto`，显式 `From<PreferencesDto> for Preferences` 后落盘；新增 `types::preferences` 与 `commands::preferences` 两道回归测试。
- 验证：开关（`launchMain` true→false）、缩放（100→150）、下拉（`panelMode` embedded→browser）均落盘且 UI 不回滚；「还原通用默认」恢复 30 项默认。

**③ 拓展页「恢复 / MCP 添加 / MCP 编辑」静默无反应**
- 根因：依赖 `window.prompt`，Tauri 的 WKWebView 不实现 JS 对话框 → 返回 null → 直接 return。
- 修复：新增应用内输入型对话框（`app.openInputModal` + `AppModal` 字段渲染与样式），替换三处 `window.prompt`；取消 / Esc / 遮罩返回 null，不产生副作用。
- 验证：恢复弹窗（Skill 名称）、MCP 添加（名称+命令 → 本机执行边界确认）、MCP 编辑（预填现有命令）均正常。

### 40.3 其他核实（非缺陷）

- 迁移导出/导入（Argon2id + AES-256-GCM）在**改后**主线程无阻塞（导出期间 worst≈16ms），产物正确落盘。
- 日志/通知清理、打开日志/导出/受管目录、Doctor、备份生成、启停/重启、通知中心（全部已读/清理已解决/收起）、外链与本地文档打开均正常；空输入/无端点等异常路径给出中文 toast，不崩溃。
- 扩展目标开关在 skills-store 为空时为**优雅失败**（toast：「Skill 同步失败；目标冲突或文件不可写未覆盖。」），非静默。

### 40.4 观察项（未改，含用户已指示「先不动」项）

1. 生命周期任务卡（`process-progress`）为右上浮层，会拦截其覆盖区域的点击；启动/重启时常驻至 35s 超时才收。
2. 启动/重启「检测不到完成」：官方进程已就绪，任务卡仍停在「等待就绪健康检查」直到超时（用户已指示先不动）。
3. 英文文案（用户已指示先不动）；Doctor 报告保留官方英文原文。
4. 扩展目标开关失败文案可更精确（当前未区分「无本机 Skills 源」与「目标冲突」）。
5. 拓展页分页按钮无绑定处理（当前恒为「第 1 / 1 页」）。

> 审计方法学提醒：右上角浮层（任务卡 / 通知中心 / Toast）位于固定层，可能拦截 CG 点击；自动化需先关闭浮层再测其下控件，否则会把「被遮挡」误判为「无反应」。`screencapture` 存在抓帧时序，短时 Toast 需连拍差分判定。

### 40.5 门禁

`cargo fmt --check`、`clippy --all-targets -D warnings`、`cargo test` **319 passed / 0 failed**（lib 257 + 集成 62）；前端 `vue-tsc --noEmit`、`vitest` **31 文件 / 126 用例**、`vite build` 全通过。证据见 `.adg/work/evidence/task66-audit/`。

## 41. 拓展管理客户端图标对齐原型（2026-09-19 第三十三段）

用户反馈「拓展管理的各 Agent 工具图标没有像原型都显示出来，目前都是一个圆」。

### 41.1 根因

- `ExtensionsRoute.vue` 的两个目标按钮（Skills 行、MCP 行）都把图标硬编码为同一个占位圆 `<circle cx="12" cy="12" r="8"/>`，与原型不一致。
- 原型 `05-原型/原型/index.html` 在 `<defs>` 里定义了 6 个品牌图标 symbol：`ext-icon-claude / codex / gemini / grok / opencode / hermesagent`（单色 `<path>`，`viewBox 0 0 24 24`），并以 `<use href="#ext-icon-*">` 引用。
- 目标按钮同时缺少 `aria-label` / `title`（AX 名称为空，可访问性与可测性都受影响）。

### 41.2 修复

- 从原型 symbol **逐字提取**路径几何，生成 `src-ui/src/data/clientIcons.ts`（`clientIcons: Record<ExtensionClientId, ClientIcon>`，含 `viewBox` 与 `paths[{d, clipRule?, fillRule?}]`）。
- 新增 `src-ui/src/components/ClientIcon.vue`：按客户端 id 渲染内联 `<svg class="target-svg">`，沿用既有 `.target-icon .target-svg { fill: currentColor }` 样式（未选中 `--faint`、选中 `--text`+accent 底），与原型一致。
- Skills 行与 MCP 行改用 `<ClientIcon :client="target"/>`，并补 `aria-label`（「将 X 同步到 Claude」/「将 X 写入 Claude」）与 `title`。

### 41.3 验证

- 运行态实测（dev + 真实 AX）：Skills 与 MCP 两个标签页每行 6 个目标按钮均渲染为各自品牌图标（Claude 星芒 / Codex 回旋 / Gemini 四角星 / Grok 斜标 / OpenCode 方框 / Hermes），不再同形。
- 新增回归测试 `tests/client-icons.test.ts`：覆盖全部 6 个目标、断言几何非空且互不相同、且不再包含占位圆路径；`ClientIcon` 渲染测试断言内联 path 数据一致。
- 同时补 `tests/app-modal.test.ts` 两条输入型对话框用例（预填/回填/确认返回、取消返回 null）。

### 41.4 门禁

前端 `vue-tsc --noEmit`、`vitest` **32 文件 / 131 用例**、`vite build` 全通过。证据见 `.adg/work/evidence/task66-audit/100-客户端图标.png`、`101-图标放大.png`、`102-MCP图标.png`。

## 42. 设置·拓展管理「Skills 源目录」行对齐原型（2026-09-19 第三十四段）

用户反馈：设置 → 拓展管理的「Skills 源目录」在路径较长时挤压右侧按钮、显示错乱，要求按原型改。

### 42.1 原型依据

- 原型 `05-原型/原型/index.html`：`.setting-row` 是两列 grid（`minmax(0,1fr) auto`）；`Skills 源目录` 行的结构为「标题/说明块 + `.controls`（打开 / 选择）+ `.cli-copy-btn.skills-source`（独立路径条）」，并以 `.setting-row>.cli-copy-btn{grid-column:1/-1}` 让路径条**独占整行**；`.skills-source{min-width:0}`。

### 42.2 根因与修复

- 根因：实现对路径条做了自定义 `.skills-source{max-width:280px}`，并把它放进右侧 `.controls`，与按钮争抢同一列 → 长路径下按钮被挤压、视觉错乱。
- 修复（回到原型）：
  1. 模板：把 `.cli-copy-btn.skills-source` 移出 `.controls`，作为 `.setting-row` 的直接子元素；`.controls` 只保留「打开 / 选择」；补 `title="复制 Skills 源目录"`。
  2. 样式：新增 `.setting-row > .cli-copy-btn { grid-column: 1 / -1 }`；新增 `.cli-copy-btn code { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap }`（超长路径省略号而非溢出）；`.skills-source` 去掉 `max-width:280px`，与原型一致只保留 `min-width:0`。

### 42.3 验证

- 运行态（dev + 真实 AX）100% 缩放：标题/说明在左、「打开 / 选择」在右、路径条独占整行且完整显示，右侧按钮不再被挤压。
- 窄宽压力：窗口缩到最小 960px 仍保持该结构，无重叠溢出。
- 新增回归测试 `tests/settings-skills-source.test.ts`：挂载设置·扩展管理分区，断言 `.skills-source` 是 `.setting-row` 直接子元素、其 `code` 等于源路径、且**不再出现于 `.controls` 内**，「打开 / 选择」仍在右侧操作区。

### 42.4 备注

- 原型该行展示的是标准目录 `~/.agents/skills`；本应用「Skills 源目录」是管理器自有域 `数据根/manager-state/skills-store`（`ensure_preferred_skill` 依赖它）。本次只对齐**版式**，未改变源目录语义；若需把源目录改为 Agent Skills 标准目录，属领域变更，另行确认。
- 门禁：前端 `vue-tsc --noEmit`、`vitest` **33 文件 / 132 用例**、`vite build` 全通过。证据 `.adg/work/evidence/task66-audit/110..114-*.png`。

## 43. Skills 源目录改为平台标准目录（方案 A，2026-09-19 第三十五段）

用户确认执行**方案 A**：Skills 默认应为平台标准目录（`~/.agents/skills/`，Windows 为 `%USERPROFILE%\.agents\skills\`），且**不引入自定义 Skill 位置**。

### 43.1 契约变更（冻结项）

`FZ-23` 原为「首选 `<数据根>/manager-state/skills-store/`；`~/.agents/skills/` 只读兼容」，现改为：

| 项 | 新冻结 |
|---|---|
| 默认读源 | `~/.agents/skills/`（Windows `%USERPROFILE%\.agents\skills\`），管理器**只读** |
| 管理器写入区 | `<数据根>/manager-state/skills-store/`，仅承载 ZIP 导入 / 恢复 / 备份 / 导出产物 |
| 发现 | 合并两处，同名时默认读源胜出 |
| 链接源 | 按名解析：默认读源优先，其次写入区 |
| 卸载 | 只断链，不删除默认读源内容；写入区副本先备份再移除 |
| 自定义源目录 | 不提供 |

已同步回改 `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` §3.1 `FZ-23` 与 `docs/02-项目核心/领域模型.md` §10 `source_store`。

### 43.2 代码改动

- `discovery`：`SkillSourceKind` 由 `Preferred/Compat` 改为 **`Agents/Store`**；新增 `agents_skills_dir(home)` / `skills_store_dir(data_root)` / `skills_store_relative_path()`；`discover` 以主目录标准目录为默认读源、数据根 store 为补充。
- `projection`：`ensure_preferred_skill` → `resolve_skill_source(data_root, home, name)`（读源优先、逐名解析、拒绝符号链接）；`apply_skill_links` 改为逐名解析源；写入区仍为数据根 store（`skill_store` / `restore_skill_from_backup` 只改路径助手）。
- `commands/workspace.rs`：「Skills 源目录」投影改为 `<主目录>/.agents/skills`（`client` 标记 `agents`）。
- `migration`：容器 `source_path` 仍以写入区为基准（`skills_store_relative_path()`），保持「管理器自有资产」的导出语义。
- 前端：`ExtensionSkillSource` → `'agents' | 'store'`；标签 `本地 / 管理器`；设置页说明改为「默认使用 Agent Skills 标准目录……」，去掉「自定义目录」暗示。

### 43.3 一并修正的两处副作用（避免 A 在本机「看起来仍然坏」）

1. **一个冲突目标拖垮整批链接**：`apply_skill_links` 原来只要任一客户端已有**同名真实目录**就整体报错。现改为**跳过该目标**（不覆盖、不删除），其它 Skill / 客户端照常链接——符合「不静默覆盖」契约。
2. **卸载只应断链**：`uninstall_skill` 原来要求 Skill 必须存在于写入区；对只存在于标准目录的 Skill 会失败。现改为：写入区有副本则先备份再移除；没有副本则仅断链，**不删除**标准目录内容。UI 卸载弹窗文案同步改成「不会删除 Agent Skills 标准目录中的内容」。
3. 前端同步提示改为**以重新发现结果为准**：目标已有同名真实目录时提示「<客户端> 已有同名目录，未覆盖 X。」，不再假报成功。

### 43.4 验证

- 后端：`cargo test` **324 passed / 0 failed**；新增
  `links_skill_from_agents_source_without_manager_import`（只存在于标准目录的 Skill 可直接链接，链接指向标准目录，写入区不产生副本；卸载只断链且保留来源）、
  `skill_link_skips_existing_real_directory_without_overwriting`（同名真实目录被跳过且保持原样，其它客户端正常链接）、
  `merges_source_and_store_and_prefers_agents_source`（同名时读源胜出）。
- 运行态实测（dev + 真实点击）：设置 → 扩展管理 的「Skills 源目录」显示 `/Users/ezio/.agents/skills`；拓展页 6 条 Skill 标签由「兼容」变为「**本地**」；点 `hatch-pet × Claude` 后 toast 为「已同步 hatch-pet 到 Claude。」，并在 6 个客户端目录建立指向 `~/.agents/skills/hatch-pet` 的软链接。**验证后已精确回滚**（删除本次新建的 5 个链接、还原 `extension-config.json` 与本次产生的 3 份 extension-write 备份）。
- 门禁：`cargo fmt --check`、`clippy --all-targets -D warnings`、`cargo test` 324；前端 `vue-tsc`、`vitest` **33 文件 / 132 用例**、`vite build` 全通过。

### 43.5 观察项

- 本机 `~/.codex/skills/` 中 `design-studio`、`development-knowledge`、`project-governance`、`project-management` 仍是**用户自有的真实目录**（非管理器链接）。按「不静默覆盖」契约，这些目标会被跳过（提示「已有同名目录，未覆盖」）。若用户希望改为统一由标准目录投影，需要单独确认（涉及删除/迁移现有真实目录）。
- `FZ-23` 原「设置里手动切换源存储」不再适用（用户明确不要自定义位置），已在契约中移除；`安全评审输入.md` §4.4 的「可切换」表述属更早的分析输入，以 IMP 冻结为准。

> **后续更正（2026-09-19 第三十六段）**：上条结论**已作废**。用户给出最终口径：源目录「**默认为** `~/.agents/skills`，**不是恒定**」。
> 据此重新修订 `FZ-23`（新增「自定义源目录」「自定义源目录校验」两行）与 `docs/02-项目核心/领域模型.md` §10 `source_store`；`安全评审输入.md` §4.4 的「源存储可切换」恢复有效，并需补路径校验细节。
> 本段所述的**代码改动仍然有效**（默认读源 = 标准目录、发现合并两处、源目录胜出、`apply_skill_links` 逐名解析、卸载只断链），只是其中的**「不提供自定义源目录」一条作废**——实现侧因此落后于现行契约，记为 `R-22`；自定义路径校验记为 `R-23`。

## 44. 契约修订：Skills 源目录由「恒定」改为「默认为」标准目录（2026-09-19 第三十六段）

用户口径：「契约要求源目录恒为 `~/.agents/skills` 改成默认是这个不是恒定。」

### 44.1 契约变更（冻结项，就地修订）

`FZ-23` 由「源目录恒为标准目录；不提供自定义源目录」改为：

| 项 | 新冻结 |
|---|---|
| 默认源目录 | `~/.agents/skills/`（Windows `%USERPROFILE%\.agents\skills\`），管理器**只读**；**默认值，不是固定值** |
| 自定义源目录 | **允许**改为自定义目录，同样**只读**；切换只更新管理器引用，不移动 / 不复制 / 不删除任何已有 Skill |
| 自定义源目录校验 | 存在且可读；入口非符号链接；真实路径不得与数据根、管理器写入区、任一客户端 Skills 目录重叠；不满足则不切换、保留原值并给出原因。**参数待安全评审确认** |

- 随之调整：`发现` / `链接源` / `卸载` 三行的「默认读源」措辞统一为「源目录」。
- 同步回改 `docs/02-项目核心/领域模型.md` §10 `source_store`。
- 变更记录已写入 `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-01.md` §9。

### 44.2 安全语义变化（需评审）

原「不提供自定义源目录」的理由是**不引入任意路径输入**。放开后该立场变为「引入但受校验约束」——这是安全边界变更，已记为 `R-23`，校验参数在 IMP 内标注「待安全评审确认」。

`安全评审输入.md` §4.4 / §15.6 第 8 项原本就支持「源存储可切换、切换时平滑迁移」，本次修订与其一致；需补的是**路径校验与失败语义**细节。

### 44.3 本轮未改代码

本轮只改契约与文档，**未改动任何实现**。由此产生的实现差异：

| 现状 | 与现行 `FZ-23` 的差距 | 编号 |
|---|---|---|
| `commands/workspace.rs` 源目录投影写死 `<主目录>/.agents/skills` | 无「源目录」配置项、无自定义入口、投影不读配置 | `R-22` |
| 前端已按上一版去掉「自定义目录」暗示 | 需要恢复自定义目录入口与文案 | `R-22` |
| 无自定义源目录的路径校验 | 需按新校验行实现（非符号链接、不与受管域重叠等） | `R-23` |

代码侧对齐**待单独授权**；建议与 `R-18`（`skills_sync_mode` 接入写入路径）同批处理，二者同属「扩展配置的偏好未真正被写入路径消费」。

## 45. 落地：源目录可自定义 + 同步方式消费 + 核对/修正（2026-09-19 第三十七段）

用户授权：「整个事配套，一起做。」——把 §44 遗留的实现差异（`R-22`）、`R-18`（同步方式未被写入路径消费）、`R-19` 部分缓解、`R-20`（设置页不刷新发现）、`R-21`（`testSkillsSync()` 空壳）作为**一批跨 Rust + Vue 的改动**一起落地。

### 45.1 后端（`src-tauri`）

| 文件 | 改动 |
|---|---|
| `modules/extensions/mod.rs` | 新增 `SYNC_METHOD_SYMLINK` / `SYNC_METHOD_COPY` / `SYNC_METHODS`；`ExtensionConfig` 新增 `source_store: Option<String>` 与 `sync_method: String`（`#[serde(default)]`，默认 `symlink`）；新增 `sync_method_normalized()` / `uses_copy()` / `set_source_store()` / `set_sync_method()`；`ProjectionFingerprint::SCHEMA_VERSION` **1 → 2**，新增 `MIN_SUPPORTED_SCHEMA_VERSION = 1` 与 `needs_migration()` |
| `modules/extensions/source_dir.rs`（**新建**） | `SourceDirError` 七种原因码（`not_absolute` / `missing` / `symlink` / `not_directory` / `not_readable` / `unresolvable` / `overlaps_managed`）+ 中文文案；`default_source_dir` / `configured_source_dir` / `validate_or_default`（发现用，失败回落默认并带原因）/ `validate_source_dir`（`R-23` 严格校验）/ `list_subdirectories`（只返回子目录、不跟随符号链接、上限 500）/ `picker_start_dir` |
| `modules/extensions/projection.rs` | `ProjectionCommand` 新增 `SetSourceDir` / `SetSyncMethod` / `ResyncSkills`；`apply_skill_links` 读配置源目录 + 按 `sync_method` 分派（软链接失败回退复制，新增 `copy_skill_tree`，深度 40、不复制符号链接、保留文件模式）；新增 `load_config_lenient`；`read_stored` / `save_locked` / `apply_skill_links` 三处对 `needs_migration()` 放行（避免升级后旧配置被误判 `external_modified` 永久不可写）；`ResyncSkills` **不改配置、不涨修订号**（`FZ-25` 只在配置 / 启用矩阵变化时 +1）；`ProjectionError` 新增 `SourceDirInvalid` / `SyncMethodInvalid` |
| `modules/extensions/discovery.rs` | `discover(data_root, home, source_dir, targets)` 签名变更——不再自己推导源目录 |
| `commands/extensions.rs` | 新增 `list_extension_directories`（tauri command + `_with_paths` 变体）；`list_extensions_with_paths` 改用配置源目录并回填 `source_dir` / `source_dir_notice` / `source_dir_custom` / `sync_method`；`ExtensionWriteCommand` 新增三个变体；`projection_error` 补两个分支 |
| `commands/workspace.rs` | 设置页「Skills 源目录」投影改读配置（不再写死 `<主目录>/.agents/skills`） |
| `commands/preferences.rs` / `modules/preferences/mod.rs` / `types/preferences.rs` | **移除** `skills_sync_mode` 及其校验与测试 fixture |
| `types/extensions.rs` | `ExtensionsDto` 新增 4 字段；新增 `DirectoryListingDto` / `DirectoryEntryDto`；`ExtensionConfigDto` 新增 `source_dir` / `source_dir_custom` / `sync_method` |
| `lib.rs` | 注册 `list_extension_directories` |

### 45.2 前端（`src-ui`）

| 文件 | 改动 |
|---|---|
| `commands/extensions.ts` | `ExtensionSyncMethod`、`DirectoryListingDto` / `DirectoryEntryDto`、`listExtensionDirectories()`、`ExtensionsDto` / `ExtensionConfigDto` 新字段、`ExtensionWriteCommand` 三个新变体 |
| `commands/preferences.ts` | 移除 `skillsSyncMode` |
| `stores/app.ts` | 新增 `setExtensionSourceDir` / `setExtensionSyncMethod` / `resyncExtensionSkills` / `browseExtensionDirectories`；`ModalState` 新增 `picker?: "skills-source"` |
| `components/SourcePicker.vue`（**新建**） | 只读目录浏览器：上一级 + 可编辑路径 + 子目录列表 + 底部「可否选中」；原因码 → 中文映射在前端 |
| `components/AppModal.vue` | 支持 `modal.picker`，确认时回传 `path` |
| `styles/base.css` | 新增 `.source-picker*` 样式 |
| `routes/SettingsRoute.vue` | 删 `testSkillsSync()`，改为 `chooseSkillsSourceDir` / `restoreDefaultSkillsSourceDir` / `chooseSkillsSyncMethod` / `resyncSkills`；源目录行加「选择自定义目录」+「恢复默认」（自定义时显示）；同步方式行「测试同步」→「检查并修正」；同步方式读写 `app.extensionConfig.syncMethod`；`watch` 里进入 `extensions` 分区补 `loadExtensionConfig()` + `loadExtensions()`（修 `R-20`） |
| `dev/auditHarness.ts` + `tests/{preferences,panel,cli-ipc,extensions}.test.ts` | 移除 `skillsSyncMode`；补新字段 |

### 45.3 新增测试

- `source_dir.rs` 8 个单测（路径校验七类原因 + 目录列举）。
- `modules/extensions/mod.rs` 新增 `legacy_schema_version_is_readable_but_flagged_for_migration`、`source_store_and_sync_method_participate_in_fingerprint`；改 `fingerprint_shape_and_source_are_frozen`（schema 1 → 2）。
- `src-tauri/tests/extensions.rs` 新增 6 个：`copy_sync_method_writes_real_copies_and_replaces_leftover_symlinks`、`custom_source_dir_is_used_by_discovery_and_link`、`invalid_custom_source_dir_is_rejected_and_keeps_previous_value`、`directory_listing_returns_subdirectories_with_selectable_reasons`、`legacy_fingerprint_schema_is_migrated_without_conflict`、`resync_skills_realigns_targets_after_source_dir_change_without_bumping_revision`。
- `src-ui/tests/settings-skills-source.test.ts` 扩到 3 个；`src-ui/tests/source-picker.test.ts` 新建 3 个。

### 45.4 门禁（本机重跑，2026-09-19）

| 门禁 | 结果 |
|---|---|
| `cargo fmt --check` | OK |
| `cargo clippy --offline --all-targets -- -D warnings` | OK |
| `cargo test --offline` | **341 passed / 0 failed**（lib `268`、extensions 集成 `22`，其余为其它集成用例） |
| `vue-tsc --noEmit` | OK |
| `vitest run` | **34 文件 / 137 用例全通过** |
| `vite build` | OK（169ms；仅一条既有的 `update.ts` 动态导入提示，与本批无关） |

> 环境要点：`cargo` 带 `--offline` 可用（依赖已缓存）。本轮以**测试套件 + 静态检查 + 构建**作为验证证据，未在本地重跑真机点击流程（未发布）。

### 45.5 风险状态收口

| ID | 状态 |
|---|---|
| `R-18` | **已关闭**：`skills_sync_mode` 移除，`sync_method` 接入写入路径 |
| `R-19` | **未关闭（部分缓解）**：发现已读配置源目录；L1 结构核对 / `DriftReport` **未实现**，属有意保留的后续项（方案草案 §10 第 3 项） |
| `R-20` | **已关闭**：设置页进 `extensions` 分区刷新发现 |
| `R-21` | **已关闭**：`testSkillsSync()` → `resync_skills` |
| `R-22` | **已关闭**：`source_store` 配置 + `list_extension_directories` + `SourcePicker.vue` + 投影读配置 |
| `R-23` | **已实现，待安全评审**：七种原因码校验；**校验参数本身仍待安全评审确认** |

同步回写：`风险与开放问题.md` §3.5 / §3.5.1 与 §7、`01-需求分析/README.md` 阶段状态、`05-原型/README.md`、`02-方案草案/Skills 落点核对与漂移检测.md` §3 / §10。

### 45.6 有意未做

- `R-19` 的 L1 自动结构核对与 `DriftReport` 数据对象（原型侧有 mock，正式应用未做）。
- 原型侧：设置行结论徽标 +「上次核对 X 前」、体检弹窗区分 L1 / L2、拓展页 Skill 行漂移标记（方案草案 §10 第 7 项）。
- 本轮未 `git commit`（用户未要求）。

## 46. 选择器改用系统原生目录对话框，删除自造浏览器（2026-09-19 第三十八段）

用户口径：「选择路径不应该重复造轮子，应该用自带系统的（macOS / Windows 一致），原型和事实、开发都调整」，并补充「原来造的轮子代码不要了」。

### 46.1 结论

目录选择一律调用**操作系统原生目录选择器**（macOS 访达 / Windows 资源管理器）；管理器**不自造目录浏览器**。已落地的自造选择器（命令 + DTO + 组件 + 样式 + 测试）**全部删除**。选中的路径仍按**不可信输入**进服务端校验（`R-23` 结论不变）——系统对话框只负责选目录，不保证落在受管域之外。

### 46.2 依赖与权限

| 项 | 变更 |
|---|---|
| Rust | 新增 `tauri-plugin-dialog = "2.7.3"`；`lib.rs` 注册 `tauri_plugin_dialog::init()` |
| 前端 | 新增 `@tauri-apps/plugin-dialog ^2.7.3` |
| 权限 | `capabilities/default.json` 增 `dialog:allow-open`（最小权限，只开目录选择） |

### 46.3 删除的自造实现

| 位置 | 删除内容 |
|---|---|
| `src-tauri/src/commands/extensions.rs` | `list_extension_directories` 命令与 `list_extension_directories_with_paths` |
| `src-tauri/src/types/extensions.rs` | `DirectoryListingDto` / `DirectoryEntryDto` |
| `src-tauri/src/modules/extensions/source_dir.rs` | `list_subdirectories`、`picker_start_dir`、`LISTING_MAX_ENTRIES` |
| `src-tauri/src/lib.rs` | 命令注册 |
| `src-tauri/tests/extensions.rs` | `directory_listing_returns_subdirectories_with_selectable_reasons` |
| `src-ui/src/components/SourcePicker.vue` | 整个组件 |
| `src-ui/src/commands/extensions.ts` | `DirectoryListingDto` / `DirectoryEntryDto` / `listExtensionDirectories` |
| `src-ui/src/stores/app.ts` | `browseExtensionDirectories`、`ModalState.picker` |
| `src-ui/src/components/AppModal.vue` | picker 分支与 `SourcePicker` 引用 |
| `src-ui/src/styles/base.css` | `.source-picker*` 全部规则 |
| `src-ui/tests/source-picker.test.ts` | 整个文件 |

保留不动：`source_dir.rs` 的解析与校验（`validate_or_default` / `validate_source_dir` / 七种原因码）——**校验是服务端职责，与选择器无关**。

### 46.4 新实现

- `SettingsRoute.chooseSkillsSourceDir()`：`open({ directory: true, multiple: false, title, defaultPath })` → 取消返回 `null` 时保留原值并提示；抛错时提示「无法打开系统目录选择器」；成功则走既有 `setExtensionSourceDir(path)`（后端校验，失败保留原值）。
- 前端测试：新增「调用系统选择器并写入所选目录」与「取消时保留当前源目录」两条（mock `@tauri-apps/plugin-dialog`），断言 `directory: true` / `multiple: false` 与 `set_source_dir` 载荷。

### 46.5 原型

- `index.html`：删除自造迷你浏览器（`SKILLS_MOCK_FS` 导航、上一级 / 前往 / 子目录列表），改为**独立系统样式面板**示意系统对话框：交通灯窗头 + 「系统对话框 · 原型示意」标签 + 位置栏（个人文件夹 / 文稿 / 项目 / OpenCodexData）+ 文件夹列表 + 当前文件夹与「取消 / 选择」。不复用应用自己的弹窗壳，避免看起来仍像自造浏览器。
- 截图：新增 `20-系统目录选择器-light/dark.png`、`21-系统目录选择器-切换位置-light.png`；删除 `20-选择自定义目录-打开态-light.png`、`21-选择自定义目录-浏览态-light/dark.png`；`19-设置行-源目录与同步方式-light/dark.png` 重出（文案改为「可用系统目录选择器改为自定义目录」）。

### 46.6 契约与需求回写

| 文档 | 变更 |
|---|---|
| IMP `FZ-23` | 新增「**选择方式**」冻结行：一律操作系统原生目录选择器，不自造浏览器；路径仍按不可信输入校验 |
| IMP §9 | 追加本条变更记录 |
| `领域模型.md` §10 | `source_store` 说明补「用操作系统原生目录选择器改为自定义目录」 |
| `风险与开放问题.md` §3.5 / §3.5.1 | `R-22` 落地描述改为系统选择器；`R-23` 补「路径来源为系统对话框后仍按不可信输入校验」 |
| `安全评审输入.md` §4 | 新增「源目录的选取方式」待评审说明 |
| `方案草案/Skills 落点核对与漂移检测.md` §10 | 第 2 项补系统选择器 |
| `05-原型/README.md` | 条目由「源目录真选择器」改为「系统目录选择器」 |
| `通知专题-审计修订回执.md` | §7.18.4 证据图更新；新增 §8.1 选择器修订记录 |

### 46.7 门禁

| 门禁 | 结果 |
|---|---|
| `cargo fmt --check` / `clippy --all-targets -D warnings` | 通过 |
| `cargo test` | 通过（删除 1 条目录浏览集成用例，其余不变） |
| `vue-tsc --noEmit` | 通过 |
| `vitest run` | **33 文件 / 136 用例全通过**（删除 `source-picker.test.ts` 3 条，新增 2 条） |
| `vite build` | 通过 |

> 首次引入 `tauri-plugin-dialog` 需要一次联网取包；取包后 `--offline` 构建恢复可用。

## 47. 修复 WebDAV 无法连接（两个根因）与输入框无法 ⌘V（2026-09-20 第三十九段）

用户实测反馈：「webdav 无法连接」「所有输入框无法 command + v 复制粘贴」。定位到**三个独立缺陷**，全部为契约符合性修复，未改动任何冻结契约值。

### 47.1 根因一：`reqwest` 没有 TLS 后端（WebDAV 从未连通过）

| 项 | 事实 |
|---|---|
| 声明 | `Cargo.toml`：`reqwest = { version = "0.12", default-features = false, features = ["json", "macos-system-configuration"] }`——`default-features = false` 关掉了默认 TLS，之后未补任何 TLS feature |
| 实测 | 生产传输打真实 `https://` 站点：底层错误链为 `["client error (Connect)", "invalid URL, scheme is not http"]`，即**本客户端只认 http**；`WebDavError` 把它归类为 `Tls`，界面因此显示「TLS 证书校验失败…」，具有误导性 |
| 互斥 | `WebDavConfig::validate()` 强制 `base_url.starts_with("https://")`——「必须 https」+「不支持 https」= 100% 连不上 |
| 试探性排除 | 锁文件里的 `rustls` 属 `tauri-plugin-updater` 的 `reqwest 0.13.5`，与本 crate 的 `reqwest 0.12` 实例 feature 不共享 |

修复：补 `rustls-tls-native-roots`（系统根证书）。验证：同一探测由 `invalid URL, scheme is not http` 变为正常完成握手（`RAW=OK`）。

护栏 `src-tauri/tests/webdav_tls.rs`（不依赖外网、离线可跑）：
1. `tls_backend_is_compiled_in`：`reqwest::Client::builder().use_rustls_tls()` 仅在 rustls feature 启用时存在——**去掉 feature 时本测试直接编译失败**。
2. `https_scheme_reaches_the_transport_layer`：打 `https://127.0.0.1:1/`，断言错误链**不含** `scheme is not http`、**含** `Connection refused`。

两条护栏均已反向验证：临时去掉 feature 后，护栏 1 编译失败、护栏 2 报 `reqwest 没有可用的 TLS 后端，https 请求在 scheme 阶段就被拒绝：["client error (Connect)", "invalid URL, scheme is not http"]`。

### 47.2 根因二：WebDAV 口令与加密口令串位（后者覆盖前者）

`FZ-44` 冻结了两个 **purpose**：`webdav_credential` → `ocx.dav.<ref_id>`、`encryption_password` → `ocx.sync.<ref_id>`。实现违反了它：

| 位置 | 问题 |
|---|---|
| `modules/sync/config.rs:119-121` | `password` 与 `encryption_password` **都**调用 `store_webdav_password`，即都写 `ocx.dav.<ref_id>`；后写覆盖先写 |
| 后果 A | `load_webdav_password` 取到的是**加密口令** → Basic 认证必失败 |
| 后果 B | `load_encryption_password` 读的 `ocx.sync.<ref_id>`（`commands/sync.rs:292` 同步引擎使用）**从未被写入** → 同步链路也不可用 |
| 附带 | `delete_endpoint` 只删 `ocx.dav`，加密口令位即使写对了也会残留 |

修复：新增 `keychain::store_encryption_password` / `delete_encryption_password`（共用一个按 purpose 写入的私有函数），保存按 purpose 分位写入，删除端点时两个账户位一并清理。

护栏：
- 纯测试 `webdav_and_encryption_purposes_use_different_accounts`（进默认门禁）。
- 真机往返 `keychain_round_trip_keeps_both_secrets_separate`（`#[ignore]`，手动跑）：同一 `ref_id` 下两个秘密各存各的。
- **调用点**往返 `save_endpoint_keeps_webdav_and_encryption_secrets_separate`（`#[ignore]`）：走 `save_endpoint` 后核对两个账户位内容。

反向验证：把调用点改回 `store_webdav_password` 后，该测试报
`assertion left == right failed: left: "sync-pass", right: "password"` —— WebDAV 槽位里确实是加密口令，与推断一致。三条护栏修复后全部通过（真机钥匙串实测）。

### 47.3 根因三：应用菜单缺「编辑」子菜单

`infrastructure/tray_controller.rs::build_app_menu` 只构造 进程 / 视图 / 日志，并在 `lib.rs:155` 用 `set_menu` 整体替换默认菜单。macOS 的 ⌘C / ⌘V / ⌘X / ⌘A 由应用菜单 key equivalent 经 responder chain 转发给 WKWebView，缺失该子菜单即导致**所有输入框无法用键盘复制粘贴**（界面上的「复制」按钮走 `navigator.clipboard`，是另一条路径，不受影响）。

修复：加入「编辑」子菜单（undo / redo / separator / cut / copy / paste / select_all），全部使用 `PredefinedMenuItem`。这些项走系统预定义行为、**不产生菜单事件**，因此不进入 `TRAY_ACTIONS` 冻结动作域（该断言 `TRAY_ACTIONS.len() == 10` 未被触碰）。

运行态实测（dev 实例 + 辅助功能读取菜单栏）：
- 菜单栏：`Apple ｜ OpenCodeX Desktop ｜ 编辑 ｜ 视图 ｜ 日志`
- 「编辑」项：`Undo / Redo / Cut / Copy / Paste / Select All / AutoFill / Start Dictation… / Emoji & Symbols`
- 「启动 / 停止 / 重启 OpenCodex」仍在应用名菜单内（muda 把第一个子菜单用作应用菜单，属**改动前既有行为**，未被本次改动引入）。

> 注：⌘V 的端到端按键验证未能完成——操作时屏幕处于锁屏状态，无法聚焦输入框取证。菜单结构已确证正确，按键链路请用户在解锁后复验一次。

### 47.4 门禁

| 门禁 | 结果 |
|---|---|
| `cargo fmt --check` | 通过 |
| `cargo clippy --offline --all-targets -- -D warnings` | 通过 |
| `cargo test --offline` | **343 passed / 0 failed**（较上轮 +4：`webdav_tls` 2 条 + keychain 纯测试 1 条 + 调用点说明） |
| 手动 `--ignored` 真机钥匙串用例 | 3 条通过（keychain 往返、save_endpoint 往返） |

> 本轮未改前端，故未重跑 `vue-tsc` / `vitest` / `vite build`。

### 47.5 迁移提示与遗留

1. **必须重新保存一次 WebDAV 端点**：修复前保存过的端点，其钥匙串两个账户位内容已被写错（`ocx.dav` 里是加密口令、`ocx.sync` 为空），仅重装应用不会自愈。
2. **遗留（未改，待决策）**：`测试连接` 测的是**已保存**的端点（`config.active()`），若用户改了 URL / 口令但未点「保存端点」，测的仍是旧配置且失败信息不指向真实原因。上一轮已列为可选改进项，待确认「先保存再测 / 禁用并提示 / 支持未保存输入」三选一。
3. **未发现但需留意**：`FZ-41` 把「连接失败」统一映射为 `Tls` 的分类过粗——没有 TLS 后端、DNS 失败、连接被拒都会显示「TLS 证书校验失败」。本次只修了根因，未改分类策略（属契约表述，需单独授权）。
