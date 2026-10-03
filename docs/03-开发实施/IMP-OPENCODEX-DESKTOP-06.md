---
id: IMP-OPENCODEX-DESKTOP-06
object_kind: implementation.change
state: completed
title: 顶栏全局刷新与面板页原型对齐
summary: 按已确认决策补齐顶栏全局「刷新状态」入口：原型两文件删除「面板页隐藏整条顶栏」规则并给根入口补上刷新按钮；产品在顶栏主题与铃铛之间新增刷新按钮，刷新范围为全局状态快照加当前页主数据（面板页额外重载内嵌官方面板），在途排队补拉一次；补齐门禁断言、重跑原型配对对比并重编打包。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-175", "TASK-OPENCODEX-DESKTOP-176"]
completion_summary: "顶栏全局刷新与面板页原型对齐已完成：原型两文件面板页保留顶栏、根入口补全局刷新按钮；产品顶栏新增「刷新状态」按钮（全局快照+当前页主数据，面板页重载内嵌官方面板，在途排队补拉一次，转圈反馈、reduced-motion 不转、失败不假装成功）；门禁 typecheck/vitest(345)/build/audit(390) 全绿；原型配对对比 72 页+84 状态重跑通过；制品内 WKWebView 冒烟通过（三页刷新入口与 DOM 几何逐一相等、面板页保留顶栏）；重编打包完成。"
source_refs: ["IMP-OPENCODEX-DESKTOP-05", "UI规范.md §19 / §26", "系统架构.md §6.1", "契约字段.md §4"]
---

# 顶栏全局刷新与面板页原型对齐

2026-10-02。用户在使用软件并对照原型后确认：原型每个页面顶栏都有一个全局「刷新状态」按钮（`#globalRefreshBtn`），软件没有；并要求把面板页顶栏、全局刷新语义一次做对。本计划为该专项，承接 [IMP-05](IMP-OPENCODEX-DESKTOP-05.md) 的原型还原与配对装置，不重启已完成的后端或目录迁移工作。

## 1. 背景与已确认决策

原型简称 `P = docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/`。涉及两份原型：配对基准 `P/候选/2026-09-28-Logo本体形变/overview.html`（含 `#globalRefreshBtn`）与正式根入口 `P/原型/index.html`（缺该按钮）。

| # | 决策 | 来源 |
| --- | --- | --- |
| 1 | 刷新范围 = **全局状态快照 + 当前页主数据** | 用户 2026-10-02 |
| 2 | **面板页保留顶栏是正确的**；允许刷新内嵌的官方面板；**改原型去对齐** | 用户 2026-10-02 |
| 3 | 在途时**排队补拉一次**（点了就一定会刷到） | 用户 2026-10-02 |

产品现状（已核实）：`runAction('refresh')` → `loadStatusSnapshot()` → 后端命令 `get_status_snapshot`，其内部执行 `collector.refresh()`，即**真的重跑一次官方状态采集**（`apps/desktop/tauri/src/commands/mod.rs`）；失败保留上一次观测。后端另有 3s/15s 轮询并经 `status-snapshot-changed` 推送，手动刷新只是「立即补拉」。

## 2. 原型改动（阶段 A）

1. `P/候选/2026-09-28-Logo本体形变/overview.html`、`P/原型/index.html`：删除 `body.panel-mode .topbar …{display:none}` 中对 `.topbar` 的隐藏（面板页保留顶栏）；`.state-pill` 的隐藏保留（与 `html[data-route="overview"] #statePill{display:none}` 同一口径）。
2. `P/原型/index.html`：补上与候选一致的 `#globalRefreshBtn`（icon-btn、`title/aria-label="刷新状态"`、点击转圈且 `prefers-reduced-motion` 不转），使根入口与配对基准一致。
3. 因候选是**配对基准**，改动后旧配对截图作废，须重采（见 §5）。

## 3. 产品改动（阶段 B）

- `components/AppTopbar.vue`：在 `topbar-actions` 的主题分段与铃铛之间新增解析为图标按钮的「刷新状态」入口（`.icon-btn`、`aria-busy`、转圈、`prefers-reduced-motion` 不转、在途禁用）。
- 刷新编排（复用现有链路，不新增采集源/定时器）：

| 页面 | 刷新内容 |
| --- | --- |
| 概览 | 状态快照 + 最近日志/事件 |
| 面板 | 状态快照 + 重载内嵌官方面板 |
| 扩展 | 状态快照 + 重新发现 Skills/MCP |
| 诊断 | 状态快照 + 当前 Tab（环境诊断 / 日志历史 / 通知） |
| 设置 | 状态快照 + 运行来源 / 偏好 / 版本事实 |
| 托盘 | 状态快照 + 托盘状态投影 |

- 在途语义：`loadStatusSnapshot` 由「在途直接丢弃」改为「在途记录一次补拉请求，当前请求结束后立即再补拉一次」，保证点击必定产生一次真实采集。
- 失败：Toast 说明 + 保留旧观测 + 标陈旧，不假装成功（§26.1）。

## 4. 门禁与断言（阶段 C）

`apps/desktop/ui`：`npm run typecheck` / `npm run test -- --run` / `npm run build` / `npm run audit:browser`。
新增断言：刷新按钮存在且在各页可见（面板页保留顶栏后也在）、点击触发一次补拉、面板页点击会重载内嵌面板、在途禁用、`prefers-reduced-motion` 不转。
单测：路由→刷新集编排、在途排队补拉一次。

## 5. 配对与制品（阶段 D、E）

- `qa/prototype-parity-pages.browser.mjs`（72 组合）、`qa/prototype-parity-states.browser.mjs`（84 组）重跑并重采截图；面板页/顶栏相关结论逐项复核。
- 真机：用 `test/` 冒烟工具在制品内逐页点击刷新并截图留证。
- 重编打包：`npx tauri build --target aarch64-apple-darwin`，记录 `.app`/`.dmg` 哈希。

## 6. 阶段与交付

| 阶段 | 工作 | 出口 |
| --- | --- | --- |
| A | 原型两文件对齐 | `review:prototype-align` |
| B | 产品顶栏刷新 + 按页编排 + 在途排队 | `review:global-refresh` |
| C | 门禁与断言 | typecheck/vitest/build/audit 全绿 |
| D | 配对对比重跑 | `review:prototype-parity` |
| E | 真机冒烟 + 重编打包 + 回写收口 | `review:artifact-smoke` |

每阶段独立提交并回写本文件与 TASK/RCP；未通过或受阻的必需检查不得标记完成。

## 7. 边界（硬约束）

- **禁止系统级文件的删除或修改**：不执行 `security` / 钥匙串命令，不修改 `~/Library/**`、`/Library/**`、系统目录与任何用户级配置；不在仓库外删除文件。测试数据一律放仓库内 `test/runtime/` 隔离目录，启动应用只用隔离 `HOME`。
- 不引入第二套采集源或额外轮询；不改动既有 3s/15s 节奏与失败退避。
- 公开发布、签名公证与真实用户数据的高风险迁移/恢复/全局同步不在本轮范围。

## 8. 执行记录

### 阶段 A · 原型对齐（TASK-175，2026-10-02）

**状态：已完成，`review:prototype-align` 通过。**

- 两份原型（配对基准 `候选/2026-09-28-Logo本体形变/overview.html`、正式根入口 `原型/index.html`）：
  从 `body.panel-mode …{display:none}` 里移除 `.topbar`，**面板页保留顶栏**；`.main` 改为
  `display:grid;grid-template-rows:auto minmax(0,1fr)`，顶栏自成一行、面板占余下整块（新增
  `body.panel-mode .topbar{margin:0;padding:12px 16px 8px;align-items:center}`）。状态胶囊仍按原口径在面板页隐藏。
- 根入口补上与候选一致的 `#globalRefreshBtn`（HTML + `.is-busy` 转圈 + `tb-spin` + `prefers-reduced-motion` 不转 + 点击处理），
  使两份原型在面板页表现一致。
- **实测（1180×760，两文件一致）**：面板路由 `panelMode=true`、顶栏可见（高 82px）、`#globalRefreshBtn` 可见、状态胶囊隐藏、
  面板区 595px，均在 760 高窗口内不溢出。
- 原型自带浏览器 QA 全量复跑：`log-categories` 25、`nowrap` 38、`runtime-source` 115、`skill-detail` 100、`spacing` 11、
  `state-convergence` 130、`tables` 154 —— 共 **573 项 0 失败 0 控制台错误**；6 个 node 用例退出 0。
- 提交 `96cbc813`。
- **事实更正（对用户前提）**：实测**软件的面板页当前也没有顶栏**（`PanelRoute` 只渲染 `.panel-shell`，不渲染 `AppTopbar`），
  与原型改前的行为一致——并非"软件保留了顶栏"。按用户意图（每个页面都能刷新）本轮把**两边都改为保留顶栏**；
  产品侧的面板页顶栏在阶段 B 落地。

### 阶段 B · 产品顶栏全局刷新（TASK-176，2026-10-02）

**状态：已完成，`review:global-refresh` 通过（提交 `9e457788`）。**

- `AppTopbar`：主题分段与铃铛之间新增 `.topbar-refresh`（`.icon-btn`、`title/aria-label="刷新状态"`、转圈、
  `prefers-reduced-motion` 不转、在途禁用）。
- `useAppController.refreshCurrentView`：复用既有链路刷新「全局状态快照 + 当前页主数据」——概览=快照+日志；
  面板=快照+重载内嵌官方面板；扩展=重新发现 Skills/MCP；诊断=按当前 Tab（通知/环境诊断/日志）；
  设置=运行来源+偏好+版本事实（同步分区加读同步配置）。失败如实 Toast 并保留旧观测。
- `lifecycle/store.loadStatusSnapshot`：在途**不再直接丢弃**，改为 `statusQueued` 排队补拉一次（用户决策 3）。
- `panel/store` 新增 `reloadToken/requestReload`，`PanelRoute` 监听后重载内嵌官方面板（面板切片不反向依赖根 store）。
- `PanelRoute` 渲染 `AppTopbar`；`base.css` 面板模式 `.main` 改 `grid(auto + minmax(0,1fr))`，顶栏自成一行、面板占余下整块。
- 实测（1180×760）：面板页顶栏 82px、刷新按钮 32×32 可见、面板区 647px、无横向溢出。

### 阶段 C · 门禁与断言（TASK-176，2026-10-02）

| 门禁 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm run test -- --run` | **345 通过 / 0 失败**（新增 `tests/global-refresh.test.ts` 4 例：在途排队补拉一次、普通页与面板页均有刷新入口、面板重载计数器、逐路由刷新当前页主数据编排） |
| `npm run build` | 通过；主 JS `361.42 kB`（相对基线 360.03 kB **+0.39%**，≤5% 预算） |
| `npm run audit:browser` | **390/390 通过、失败 0**（新增「各页顶栏刷新入口存在且为『刷新状态』」与「面板页保留顶栏」断言） |

**阶段 C 补测（提交 `b559062b`）**：`tests/global-refresh.test.ts` 增加「按路由刷新当前页主数据（面板/扩展/设置/托盘）」
用例，逐路由断言 `refreshCurrentView` 的编排——面板=快照+`loadPanelUrl`+`requestReload`；扩展=快照+`loadExtensions`+`loadExtensionConfig`；
设置·通用=快照+`loadRuntimeSource`+`loadPreferences`+`refreshRuntimeVersionFact` 且**不**读同步配置；
设置·同步=额外读 `loadSyncConfig`；托盘=只补快照、不调页面级动作。typecheck/test(345)/build/audit(390) 全部复跑通过。

### 阶段 D · 原型配对对比重跑（TASK-176，2026-10-02）

- `qa/prototype-parity-pages.browser.mjs`：**72/72 采集成功、页面错误 0**。逐组合几何：**`.topbar` 超差 0 组**；
  `.main` 仅 `tray` 路由 12 组为既有结构差异（原型该路由走 `.tray-stage`），其余全部 ≤2px。
- 面板路由专项：修正原型 `setRoute('panel')` 未投影 `data-route`、以及面板帧 960px 最小宽度撑宽网格后，
  **12/12 组合几何 ≤2px**（此前 1180 组 topbar Δ44、900 组 Δ127）。
- `STATES_FORCE=1 qa/prototype-parity-states.browser.mjs`：**84 组、失败 0**。
- 结论：`review:prototype-parity` 维持满足（硬门几何与可见性一致；整窗像素仍按 §26.1 口径归因）。

### 原型修正（随阶段 A/D 一并落地，提交 `96cbc813`、`244d46b6`）

1. 面板页保留顶栏（两份原型一致），顶栏自成一行、面板占余下整块。
2. 根入口补齐 `#globalRefreshBtn`，与候选一致。
3. `setRoute('panel')` 投影 `data-route='panel'` + 导航高亮（此前概览专属规则会继续命中面板页）。
4. 面板模式网格列 `minmax(0,1fr)`、`#panelSection`/顶栏/标题 `min-width:0`，窄窗口下顶栏不再被面板帧最小宽度撑宽。

### 阶段 E · 真机冒烟与重编打包（TASK-176，2026-10-02）

**状态：已完成，`review:artifact-smoke` 通过（屏幕解锁后补做 ④ 层）。**

- **重新编译打包**：`npx tauri build --target aarch64-apple-darwin` 成功。
  `.app` 可执行 SHA-256 `26c670c9af79ee8bb5c4bb1bb91f11d6309a061e6084208aef5da5f6bd094e2b`；
  `.dmg` `fcc0ef65f6d562afc331719cf977c7976edc2a0d2b325b3ee1617a299064522c`；
  内嵌 `index-DNsgHy33.js` / `index-B_KbCz2X.css`（与 `ui/dist` 一致）。
- **③ 层（构建产物，与制品内嵌 bundle 同源）**：静态托管 `ui/dist` 后，`#overview` / `#panel` / `#settings`
  三页均渲染出 `.topbar-refresh`（可见、`title/aria-label="刷新状态"`、`svg fill:none`、未禁用、0 页面错误），
  且面板页 `data-route=panel` 并保留顶栏。证据 `artifact-smoke.md` §2。
- **④ 层（制品内 WKWebView 冒烟，屏幕解锁后补做，通过）**：隔离 `HOME=OPENCODEX_HOME=test/runtime/home-imp06`
  启动 `test/software/OpenCodeX Desktop.app`（可执行哈希与上表一致），逐页像素取证：
  概览 / 面板 / 设置三页顶栏右侧三组控件（主题分段 / 刷新按钮 / 铃铛）齐备，刷新按钮实测中心 css **1097 / 1103 / 1097**，
  与 ③ 层 DOM 探针 `.topbar-refresh` 的 `cx` **逐一相等**；面板页保留顶栏。按钮图标为圆形箭头，未禁用。
  点击刷新按钮后区域快速采样显示按钮出现持续视觉变化（点击被接收），采样窗内无失败 Toast（刷新成功分支）；
  本机刷新在约 75ms 采样间隔内完成，未截到转圈中间帧。证据 `artifact-smoke.md` §3 与 `evidence/imp06-04-*`。
- **工具加固（保留）**：`test/smoke/lib/ocxui.py` 的 `focus_and_capture` 与 `test/smoke/run-smoke.sh` 的锁屏守卫
  （锁屏时 `BLOCKED` 退出码 2）继续保留，避免将来产出假证据。

### 后续修订（2026-10-02，IMP-07 推翻本计划决策 2）

用户看了实际效果后确认：**面板页不应保留软件顶栏**，应尽可能给官方面板空间。本计划 §1 决策 2
（「面板页保留顶栏是正确的」）、阶段 A 的原型改动（两文件移除 `.topbar` 隐藏）与阶段 B 的
`PanelRoute` 渲染 `AppTopbar` 均被 **IMP-07 回退**：面板页恢复不渲染软件顶栏，面板自身操作改走
注入的品牌悬浮球。本文件其余结论（全局「刷新状态」按钮、按页刷新编排、在途排队补拉、门禁与打包）
继续有效；「刷新状态」不再属于面板页（面板页无顶栏）。
