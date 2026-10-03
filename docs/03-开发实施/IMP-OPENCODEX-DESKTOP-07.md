---
id: IMP-OPENCODEX-DESKTOP-07
object_kind: implementation.change
state: completed
title: 面板页让位官方页面与动作按钮材质对齐
summary: 用户看实际效果后推翻 IMP-06 的「面板页保留顶栏」决策：面板页不再渲染软件顶栏，把整块空间让给官方面板；面板自身操作只走注入的品牌悬浮球（按原型补齐品牌 logo、玻璃/阴影层级、点击展开、拖拽）。同时把概览动作按钮对齐原型的 glass 材质，并按原型 actionsByState 逐状态核对动作集合（刷新状态已是全局能力，不再挂在状态动作里）。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-177"]
completion_summary: "面板页让位官方页面与动作按钮材质对齐已完成：原型与软件面板页都不保留软件顶栏（推翻 IMP-06 决策），面板自身操作走注入的品牌悬浮球（品牌 logo、玻璃+分层阴影高阶层级、点击展开、悬浮不展开、可拖拽）；概览动作按钮按原型 glass 材质铺满整行并逐状态对齐动作集合（刷新改为全局顶栏能力）。门禁 typecheck/vitest(345)/build/audit(394) 全绿；配对 72 页+84 状态通过；制品内面板页无顶栏、悬浮球以二进制标记+③层契约取证；重编打包完成。"
source_refs: ["IMP-OPENCODEX-DESKTOP-06", "UI规范.md §9 / §19 / §26", "数据与状态.md §2.1"]
---

# 面板页让位官方页面与动作按钮材质对齐

2026-10-02。用户看实际效果后给出两条修正意见，本计划为对应专项，承接 [IMP-06](IMP-OPENCODEX-DESKTOP-06.md) 的顶栏刷新与配对装置。

## 1. 背景与已确认决策

| # | 决策 | 来源 |
| --- | --- | --- |
| 1 | **面板页不应保留软件顶栏**，应尽可能给官方面板空间（**推翻 IMP-06 决策 2**） | 用户 2026-10-02 |
| 2 | 面板页右下角悬浮球**按原型做**：品牌 logo、玻璃 + 阴影（高阶层级）、**点击**展开、保留悬浮特效、可拖拽 | 用户 2026-10-02 |
| 3 | 概览动作按钮**逐状态对齐原型**：玻璃材质铺满整行（非只有默认档有壳）；`刷新状态`是全局能力，不挂在状态动作里 | 用户 2026-10-02 |

事实核对（改动前）：
- 原型 `#panelHubToggle` 当时**悬浮自动展开**（`pointerenter`）、玻璃只有单层阴影、无 hover 上浮——与用户要求的「点击展开 + 高层级玻璃 + 悬浮特效」不符。
- 软件面板页**渲染了 `AppTopbar`**（IMP-06 落地）；注入悬浮球 `assets/panel-hub.js` 的图标是 `◉` 而非品牌 logo，玻璃较薄、无拖拽。
- 软件概览动作按钮：`materials.css` 只给**默认档** `.btn` 套了 `surface-control`（带实时模糊），`primary/danger/ghost` 回落实底——截图里只有「重启」有壳；而原型把 glass 铺满整行。
- 软件 `runtimeScenarios` 仍把 `refresh` 挂在 `not_found/stopped/starting/pending/at_risk/external_takeover/unreachable` 上，会多出按钮并折叠进「更多」；原型 `actionsByState` 已把刷新移出状态动作（改为全局顶栏图标）。

## 2. 原型改动（阶段 A）

1. `P/候选/2026-09-28-Logo本体形变/overview.html`、`P/原型/index.html`：
   - `body.panel-mode .topbar{display:none}`（恢复隐藏软件顶栏）；`body.panel-mode .main` 网格收成单行（面板占满）。
   - `.panel-hub-toggle`：玻璃铺上双色纱 + 分层阴影（近距 + 远距 + 顶部内高光）+ hover 上浮，表达高阶层级；低特效档由 `glass-effects.css` 退实底。
   - 交互：删除 `pointerenter/pointerleave` 悬浮自动展开；改为**点击**开合（键盘 Enter/Space 仍可），拖拽逻辑保留且拖动后不触发展开。
2. 因候选是**配对基准**，改动后旧配对截图作废，须重采（见 §5）。

## 3. 软件改动（阶段 B、C）

- `routes/PanelRoute.vue`：移除 `<AppTopbar />` 与其引用；`base.css` 的 `.app-shell.panel-mode .main` 收成单行，整块让给面板。
- `assets/panel-hub.js`（Rust 注入）：图标换成**品牌 logo**（自带走查渐变 + 镂空蒙版的内联 SVG）；材质补齐分层阴影 + 顶部内高光 + hover 上浮；新增按住拖拽（拖动后不展开）；仍为**点击**展开（悬浮不展开）。
- 概览动作按钮（`materials.css`）：`.motion-actions` 行内**所有**按钮统一玻璃观感（半透明分层填充 + 非透明边框 + 分层阴影含内高光；主按钮＝强调玻璃、扁平无棱线）。**刻意不做实时 `backdrop-filter`**：概览形象背后有持续动画层，模糊每帧重算会把高档帧率从 ~41fps 压到 ~29fps，撞 IMP-05 §7 的 `p95 ≤ 2B` 预算。
- 动作集合（`features/runtime/scenarios.ts`）：从各状态移除 `refresh`（全局顶栏图标承担），使动作集合与原型 `actionsByState` 一致；`at_risk` 保留「启动 OpenCodex」（`数据与状态.md` 2026-09-21 事实）。

## 4. 门禁与断言（阶段 D）

`apps/desktop/ui`：`typecheck` / `test -- --run` / `build` / `audit:browser`（更新断言：面板页**不**保留顶栏、面板页不参与顶栏刷新检查；新增概览动作按钮材质断言）。
新增 `qa/panel-hub.browser.mjs`：把注入脚本挂到空白页，断言品牌 logo、材质、点击展开、悬浮不展开、Esc 收起。
Cargo：`cargo fmt --check` / `cargo test --lib`。

## 5. 配对与制品（阶段 D）

- `qa/prototype-parity-pages.browser.mjs`（72 组合）、`qa/prototype-parity-states.browser.mjs`（84 组）重跑。
- 真机 `test/` 冒烟：面板页确认无软件顶栏、整块让位；注入悬浮球因本机无可运行的 OpenCodex 而无法在制品内呈现，改以「二进制含新脚本标记 + ③ 层脚本契约测试」取证。
- 重编打包并记录 `.app` / `.dmg` 哈希。

## 6. 阶段与交付

| 阶段 | 工作 | 出口 |
| --- | --- | --- |
| A | 原型：面板页去顶栏 + 悬浮球玻璃/点击展开 | `review:prototype-panel` |
| B | 软件：面板页去顶栏 + 注入悬浮球对齐原型 | `review:panel-space` |
| C | 软件：概览动作按钮玻璃材质 + 逐状态动作集合对齐 | `review:action-material` |
| D | 门禁/断言、配对重跑、制品冒烟、重编打包 | `review:prototype-parity`、`review:artifact-smoke` |

每阶段独立提交并回写本文件与 TASK/RCP；未通过或受阻的必需检查不得标记完成。

## 7. 边界（硬约束）

- **禁止系统级文件的删除或修改**：不执行 `security` / 钥匙串命令，不修改 `~/Library/**`、`/Library/**`、系统目录与任何用户级配置；不在仓库外删除文件。测试数据一律放仓库内 `test/runtime/` 隔离目录，启动应用只用隔离 `HOME`。
- 不修改官方页面样式、不注入凭据；面板页不获得管理器 IPC 权限。
- 公开发布、签名公证与真实用户数据的高风险迁移/恢复/全局同步不在本轮范围。

## 8. 执行记录（2026-10-02）

### 阶段 A · 原型改动

- 两份原型（配对基准 `候选/2026-09-28-Logo本体形变/overview.html`、正式根入口 `原型/index.html`）：
  `body.panel-mode .topbar{display:none}`（面板页不保留软件顶栏），`body.panel-mode .main` 收成单行。
- `.panel-hub-toggle` 玻璃铺双色纱 + 分层阴影（近距 + 远距 + 顶部内高光）+ hover 上浮；删除 `pointerenter/pointerleave`
  悬浮自动展开，改为**点击**开合（键盘 Enter/Space 仍可）；拖拽保留、拖动后不展开。
- 原型的 6 组 node 用例与浏览器 QA **573 项全部 0 失败 0 控制台错误**。

### 阶段 B · 软件面板页让位官方页面

- `routes/PanelRoute.vue` 移除 `<AppTopbar />`；`base.css` 面板模式 `.main` 收成单行，整块让给面板。
- `assets/panel-hub.js`：图标换成**品牌 logo**（自带走查渐变 + 镂空蒙版的内联 SVG，独立 id `ocxdHubGrad/ocxdHubKnock`）；
  材质补分层阴影 + 顶部内高光 + hover 上浮；新增按住拖拽；仍为点击展开。
- 制品内实测（隔离 HOME）：概览保留顶栏、面板页无品牌/标题（证据 `evidence/imp07-overview-topbar.png`、
  `evidence/imp07-panel-no-topbar.png`）。本机无 `ocx` 官方面板不加载，注入浮层改由二进制标记 + ③ 层脚本契约测试取证。

### 阶段 C · 概览动作按钮材质与动作集合

- `materials.css`：`.motion-actions` 行内**所有**按钮统一玻璃观感（半透明分层填充 + 非透明边框 + 分层阴影含内高光；
  主按钮＝强调玻璃、扁平无棱线）。**不做实时 `backdrop-filter`**：加上会把高档动画帧间隔从 ~25ms 拉到 ~34ms，撞 §7 预算。
- `features/runtime/scenarios.ts`：从各状态移除 `refresh`，动作集合对齐原型 `actionsByState`（刷新改为全局顶栏图标）；
  `at_risk` 保留「启动 OpenCodex」。
- `数据与状态.md` §2.1 动作表同步修订；`UI规范.md` §9 补齐悬浮球规则与「面板页不保留顶栏」修订。

### 阶段 D · 门禁、配对与制品

| 项 | 结果 |
| --- | --- |
| `typecheck` / `test -- --run` | 通过 / **345 通过** |
| `build` | 通过 |
| `audit:browser` | **394/394**（新增 6 条概览动作材质断言；面板页断言改为「不保留顶栏」） |
| `cargo fmt --check` / `cargo test --lib` | 通过 / panel 8/8 |
| `prototype-parity-pages` | 72/72、页面错误 0；`.topbar` 超差 0 组；**panel 12/12 ≤2px** |
| `prototype-parity-states` | **84 组 0 失败** |
| 重编打包 | `.app` `8d537c3a…`、`.dmg` `c9e287c8…` |
| 制品冒烟 | 面板页无软件顶栏；悬浮球以二进制标记 + ③ 层契约（13/13）取证 |

证据：`.adg/work/imp07-execution/evidence/`（`artifact-smoke.md`、`imp07-*.png`、`panel-hub-contract.log`、`binary-hub-markers.txt`）。

## 追加（2026-10-03）：悬浮球点击后留一圈黑色焦点环

用户报告：「为什么面板的悬浮按钮点击有一圈黑色边」（附截图，黑圈套在圆形悬浮球外）。

**根因**：`panel-hub.js` 的 `setOpen(false)` 无条件 `toggle.focus()`。鼠标点击/点空白/拖拽收起时
都会把焦点**程序化**送回悬浮球，而 WKWebView 会把程序化 `focus()` 判为 `:focus-visible`，
于是 `#… .hub-toggle:focus-visible{outline:2px solid var(--hub-text)}` 生效（浅色下 `--hub-text:#0d0d0d`＝黑），
点完就留下 2px 黑圈 + 2px 偏移。原型只在 **Esc** 关闭时才 `focus()`（`panel.html`：
`if(panelHubOpen&&event.key==='Escape'){setPanelHubOpen(false); …focus()}`），鼠标路径从不回焦。

**修复**：与原型对齐——`setOpen(false)` 不再回焦；只有键盘 Esc 关闭时才 `toggle.focus()`
（保留键盘连续性）。鼠标点击本身走 `pointerdown` + `preventDefault()`，本来就不会取焦，
去掉程序化回焦后点击不再出现焦点环。

**验证**：`qa/panel-hub.browser.mjs` 新增 5 条断言（鼠标收起/点空白收起后
`document.activeElement` 不是悬浮球；Esc 收起后焦点回到悬浮球并能继续键盘操作），
**面板悬浮球契约 18/18 通过**；`cargo test` 6/6 通过。

**制品哈希（本轮重编）**：

- 可执行文件 `opencodex-desktop`：`e5dcf367a880f81fa2400100b0e7f8d519e11d49ceb40f554abc80a67cc9438b`
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：`2d975ff9055a6d1ab55a0a3c06cb5aa5babc3114e016c624345f118cfcd0e372`
