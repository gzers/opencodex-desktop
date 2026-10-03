---
id: IMP-OPENCODEX-DESKTOP-09
object_kind: implementation.change
state: completed
title: 面板悬浮球职责收敛（移除重复的「概览」入口）
summary: 用户在复核面板页时指出：面板页已有悬浮球「重载」，所以顶栏「刷新状态」在面板页是重复的——这正是 IMP-07 去掉面板页顶栏的理由之一。据此收敛悬浮球职责：它只承担「面板自身」的快捷操作（缩放 / 重载 / 浏览器），跨路由导航（概览）交给面板模式下常驻的左侧图标栏，移除同一页里重复的「概览」菜单项；原型与软件同改，并把该口径写进 UI规范。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-179"]
source_refs: ["IMP-OPENCODEX-DESKTOP-07", "UI规范.md §9", "AC-OPENCODEX-DESKTOP-17"]
completion_summary: "面板悬浮球职责收敛已完成：悬浮球只承担面板自身操作（缩放 / 重载 / 浏览器），移除同一页重复的「概览」（跨路由导航交给面板模式下常驻的 64px 图标栏）；原型两文件与软件注入脚本、Rust 放行集合、前端 handler 同步收敛，UI规范 §9 登记。门禁 typecheck/vitest(345)/build/audit(394) 全绿、悬浮球契约 13/13、原型 QA 573 项 0 失败、配对 72 页+84 状态通过、cargo fmt+lib(406) 通过；制品内核对 data-action 标记并重编打包完成。"
---

# 面板悬浮球职责收敛（移除重复的「概览」入口）

2026-10-02。用户在复核面板页（浏览器里对着原型）时提出：面板页的「刷新」已经由悬浮球的「重载」承担，所以顶栏「刷新状态」在面板页是多余的。核对后确认成立，并把悬浮球的职责一并收敛清楚。承接 [IMP-07](IMP-OPENCODEX-DESKTOP-07.md) 的「面板页不保留软件顶栏」决策。

## 1. 已确认事实（核对，2026-10-02）

| # | 事实 | 依据 |
| --- | --- | --- |
| 1 | 悬浮球「重载」只重载内嵌官方面板，不拉管理器全局状态快照 | `panel-hub.js` `data-action="reload"` → `commands/panel.rs` `on_navigation` → `PanelRoute.vue` `syncPanel('show', true)` → `view.reload()` |
| 2 | 面板页不显示管理器状态，所以「重载」足够，不需要顶栏「刷新状态」 | 面板路由只渲染 `.panel-shell`，无顶栏 |
| 3 | 面板模式下左侧栏收成 `64px` 图标栏、**图标仍在**（仅隐藏文字） | 软件 `base.css:147-152`；原型 `index.html:1164-1180` |
| 4 | 因此悬浮球里的「概览」与左栏「概览」图标是同一动作，属同页重复 | 上述两条 |

结论：悬浮球应只放**面板自身**的快捷操作（缩放 / 重载 / 浏览器）；跨路由导航由常驻左栏承担。移除「概览」菜单项。

## 2. 改动清单

| 文件 | 改动 |
| --- | --- |
| `apps/desktop/tauri/assets/panel-hub.js` | `ACTIONS` 去掉 `overview`；菜单移除「概览」项 |
| `apps/desktop/tauri/src/commands/panel.rs` | `ocxd-panel://` 放行集合去掉 `overview` |
| `apps/desktop/ui/src/routes/PanelRoute.vue` | 移除 `case 'overview'` 与随之不再使用的 `useRouteStore` |
| `apps/desktop/ui/qa/panel-hub.browser.mjs` | 菜单断言改为 `zoom-out,zoom-in,reload,browser` |
| `P/原型/index.html`、`P/候选/2026-09-28-Logo本体形变/overview.html` | 移除 `#panelOverviewBtn` 按钮与其 `onclick` |
| `docs/02-项目核心/UI规范.md` §9 | 内容/交互行收敛，并登记「悬浮球职责」与「面板页刷新由重载承担」的修订说明 |

`P = docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/`。

## 3. 门禁与验证

| 项 | 口径 |
| --- | --- |
| `typecheck` / `test -- --run` / `build` | 全绿 |
| `qa/panel-hub.browser.mjs` | 13/13，且断言菜单不再含 `overview` |
| `audit:browser` | 394/394 |
| 原型 QA | jsdom 6 组 + 无头 Chromium 573 项，0 失败 0 console 错误 |
| `prototype-parity-pages` / `-states` | 72 页 / 84 组 0 失败 |
| Cargo | `cargo fmt --check`、`cargo test --lib` 全绿 |
| 制品 | 重编打包；在制品可执行内核对注入脚本标记：`data-action="reload"` / `"browser"` 在、`data-action="overview"` 不在 |

## 4. 边界（硬约束）

- **禁止系统级文件的删除或修改**：不执行 `security` / 钥匙串命令，不修改 `~/Library/**`、`/Library/**`、系统目录与任何用户级配置；不在仓库外删除文件。测试数据一律放仓库内 `test/runtime/` 隔离目录，启动应用只用隔离 `HOME`。
- 不改官方页面样式、不注入凭据；面板页不获得管理器 IPC 权限。
- 公开发布、签名公证与真实用户数据的高风险迁移/恢复/全局同步不在本轮范围。

## 5. 执行记录（2026-10-02）

### 5.1 核对与决策

- 复核「悬浮球是否已有刷新能力」：`panel-hub.js` 的 `reload` → `commands/panel.rs` `on_navigation` → `PanelRoute.vue`
  `syncPanel('show', true)` → `view.reload()`，确实只重载内嵌官方面板（不拉管理器全局状态快照）。
- 面板页不显示管理器状态，故「重载」已够用；这条同时坐实了 IMP-07「面板页不需要顶栏刷新」的判断。
- 顺带发现同页重复：面板模式下左栏仍常驻 `64px` 图标栏（图标可见、仅隐藏文字），其中的「概览」与悬浮球的「概览」是同一动作。
- 决策（用户 2026-10-02 确认按建议改）：**悬浮球只放面板自身操作**（缩放 / 重载 / 浏览器），跨路由导航交给常驻左栏，移除「概览」。

### 5.2 门禁、契约与制品

| 项 | 结果 |
| --- | --- |
| `typecheck` / `test -- --run` / `build` | 通过 / **345 通过** / 通过 |
| `qa/panel-hub.browser.mjs` | **13/13**（菜单断言改为不含 `overview`） |
| `audit:browser` | **394/394**，新页面错误 0 |
| 原型 QA（jsdom 6 组 + 无头 Chromium） | 6 组 0 失败；浏览器 **573 项 0 失败 0 console 错误** |
| `prototype-parity-pages` / `prototype-parity-states` | 72/72（页面错误 0）/ **84 组 0 失败** |
| Cargo | `cargo fmt --check` 通过；`cargo test --lib` **406 通过 / 0 失败 / 2 忽略** |
| 制品核对 | 可执行内 `data-action="reload"`/`"browser"` 在、`data-action="overview"` 不在；CSS chunk 与 IMP-08 同哈希（居中修复未回退） |
| 重编打包 | `.app` 可执行 `bdff71d5…`、`.dmg` `4ad75fe8…` |

证据：`.adg/work/imp09-execution/evidence/`（`hub-scope.md`、`hashes.txt`）。
