---
id: IMP-OPENCODEX-DESKTOP-08
object_kind: implementation.change
state: completed
title: 概览状态形象在 WebKit 下的居中修复
summary: 用户对照原型看实际制品，发现概览「环境准备」态的品牌图形与状态文字没有相对正文列居中。根因不是文案长短，而是状态舞台用 `grid + justify-items:center` 时，WebKit（Safari / 应用内 WKWebView）给出的 fit-content 宽度让形象块整体右偏（Blink 不重现，所以既有浏览器探针查不出来）。原型与软件同款写法，故两边一起改成「显式占满整行再各自居中」，并补 WKWebView 几何回归与制品实测。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-178"]
source_refs: ["IMP-OPENCODEX-DESKTOP-05", "UI规范.md §25.1", "AC-OPENCODEX-DESKTOP-17"]
completion_summary: "概览状态形象在 WebKit 下的居中修复已完成：软件 base.css 与原型 overview-dual.css 同款改为「形象块/动作行占满整行再各自居中、副标题 margin auto」；WKWebView 探针几何回归（修复前 754.6 / 818.9 → 修复后 712.5 / 734.5，均回到正文列中心）、真实制品「环境准备」态实测居中、门禁 typecheck/vitest(345)/build/audit(394) 全绿、原型 QA 573 项 0 失败、配对 72 页+84 状态通过；新增入库探针 test/smoke/wkprobe/；重编打包完成。"
---

# 概览状态形象在 WebKit 下的居中修复

2026-10-02。用户在真实制品里看到概览「环境准备」态（`待接入` / `启动条件未满足 · 请按下方指引处理`）的品牌图形与状态文字没有相对正文列居中，要求检查并修复。本计划承接 [IMP-05](IMP-OPENCODEX-DESKTOP-05.md) 的原型还原与配对装置，是 IMP-06／IMP-07 之后的新一轮用户复核专项。

## 1. 现象与根因

现象：概览副标题变长时，形象块（Logo 主线文字 + 副标题）相对正文列整体右偏；环境卡、动作按钮与装饰光场仍居中。

根因（自建最小 WKWebView 探针确证）：

- 状态舞台 `#card-overview-status .ovb-summary`（软件 `.motion-fixed`）是 `display:grid; grid-template-columns:minmax(0,1fr); justify-items:center`，形象块 `.motion-identity` 未占满整行，宽度取 fit-content。
- 在 WebKit 下，该网格项虽按 fit-content 计算宽度，却按另一套（更宽的）宽度定位，导致整体右偏约 42px（窗口 1180 逻辑宽、副标题约 190px 时实测 `ident.cx=754.6`，正文列中心应为 `712.5`）。Blink（无头 Chromium）不重现，所以 `audit:browser` 与原型配对对比此前查不出。
- 原型 `overview-dual.css` 与软件 `base.css` 是同一套写法，原型在 WebKit 下同样偏（实测 `ident.cx=818.9` vs 正文列中心 `734.5`），因此两边都要改。

## 2. 修复口径（用户 2026-10-02 已确认「原型直接改了保持一致」）

形象块不再依赖网格项尺寸：`.motion-identity`（软件 / 原型 `#card-overview-status .motion-identity`）**显式占满整行**（`grid-column:1; width:100%; min-width:0; max-width:100%`）再各自居中——标题 `text-align:center`、副标题 `margin:5px auto 0`、动作行走 `justify-content:center`。`.motion-stage`（软件）补 `grid-column:1` 消除自动放置歧义。

该写法在 Blink 下视觉中心不变（仍等于正文列中心），只是不再依赖 fit-content 定位，故不改变既有原型配对的视觉结论。

## 3. 改动清单

| 文件 | 改动 |
| --- | --- |
| `apps/desktop/ui/src/styles/base.css` | `.motion-identity` / `.motion-actions` 占满整行再居中；`.motion-stage` 补 `grid-column:1` |
| `P/overview-dual.css` | 原型同名 `.motion-identity` / `.ovb-actions` 同款修复；`.motion-caption` 改 `margin:5px auto 0` |
| `docs/02-项目核心/UI规范.md` §25.1 | 登记「形象区居中的实现口径（2026-10-02 修订）」 |

`P = docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/`；原型基准 `P/候选/2026-09-28-Logo本体形变/overview.html`、正式根入口 `P/原型/index.html` 共用 `P/overview-dual.css`。

## 4. 验证与门禁

| 项 | 口径 |
| --- | --- |
| WKWebView 几何（新增回归） | 入库探针 `test/smoke/wkprobe/`（`wkprobe` / `wkprobe2`）读 `.motion-identity` / `.motion-caption` / 主线中心，要求等于正文列中心（软件 1180 宽＝712.5，原型＝734.5）；修复前 754.6 / 818.9 作为对照 |
| 制品实测 | 隔离 `HOME` 启动构建后的 `.app`，在「环境准备」态截屏并按行扫描量出主线与副标题中心，要求等于正文列中心 |
| `typecheck` / `test -- --run` / `build` | 全绿 |
| `audit:browser` | 394/394 |
| 原型 QA | jsdom 6 组 + 无头 Chromium 573 项，0 失败 0 console 错误 |
| `prototype-parity-pages` / `-states` | 72 页 / 84 组 0 失败 |
| 重编打包 | `.app` / `.dmg` 重编并记录哈希 |

## 5. 边界（硬约束）

- **禁止系统级文件的删除或修改**：不执行 `security` / 钥匙串命令，不修改 `~/Library/**`、`/Library/**`、系统目录与任何用户级配置；不在仓库外删除文件。测试数据一律放仓库内 `test/runtime/` 隔离目录，启动应用只用隔离 `HOME`。
- 不改官方页面样式、不注入凭据；不重启已完成的后端或目录迁移工作。
- 公开发布、签名公证与真实用户数据的高风险迁移/恢复/全局同步不在本轮范围。

## 6. 执行记录（2026-10-02）

### 6.1 根因取证（WKWebView 探针）

- 新增入库的最小 WKWebView 探针 `test/smoke/wkprobe/`（构建脚本 `build.sh`，产物 `test/smoke/bin/wkprobe`／`wkprobe2`），
  `wkprobe` 指向软件 `ui/dist`、`wkprobe2` 可指定原型 html；
  以 `app://` 本地 scheme 加载、`evaluateJavaScript` 读几何；把 `.motion-caption` 改写为长副标题以复现「环境准备」态。
- 软件（正文列中心 712.5）：修复前 `ident/mainline/caption` 中心均为 **754.6**（右偏 +42.1），`mark`（图形）仍为 712.5；
  修复后全部 `712.5`。
- 原型 `P/候选/2026-09-28-Logo本体形变/overview.html`（正文列中心 734.5）：修复前形象块中心 **818.9**（右偏 +84.4）；
  修复后 `734.5`；根入口 `P/原型/index.html` 同值。
- 结论：非文案问题，而是 `grid + justify-items:center` 在 WebKit 下对网格项 fit-content 宽度的定位差异；Blink 不重现，
  故既有 `audit:browser` 与原型配对对比查不出。

### 6.2 修复

- `apps/desktop/ui/src/styles/base.css`：`.motion-identity`／`.motion-actions` 改为 `grid-column:1; width:100%` 占满整行再各自居中；
  `.motion-stage` 补 `grid-column:1`。
- `P/overview-dual.css`：`#card-overview-status .motion-identity`／`.ovb-actions` 同款；`.motion-caption` 改 `margin:5px auto 0`
  （形象块变整行宽后副标题仍需相对正文列居中）。
- `docs/02-项目核心/UI规范.md` §25.1：登记「形象区居中的实现口径（2026-10-02 修订）」，原型与软件同一套写法。

### 6.3 门禁、配对与制品

| 项 | 结果 |
| --- | --- |
| `typecheck` / `test -- --run` / `build` | 通过 / **345 通过** / 通过 |
| `audit:browser` | **394/394**，新页面错误 0 |
| 原型 QA（jsdom 6 组 + 无头 Chromium） | 6 组 0 失败 0 JS 错误；浏览器 **573 项 0 失败 0 console 错误** |
| `prototype-parity-pages` / `prototype-parity-states` | 72/72（页面错误 0）/ **84 组 0 失败** |
| `panel-hub.browser.mjs` | 13/13 |
| 制品实测（隔离 `HOME`） | 「环境准备」态形象主线中心 712.25、副标题中心 712.50（正文列中心 712.5） |
| 重编打包 | `.app` 可执行 `62899a58…`、`.dmg` `a2e7e38a…`、CSS chunk `index-JF4_ARqs.css` |

证据：`.adg/work/imp08-execution/evidence/`（`wkwebview-geometry.md`、`artifact-align.md`、`imp08-artifact-setup.png`、`hashes.txt`）。
