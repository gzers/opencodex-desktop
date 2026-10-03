---
id: IMP-OPENCODEX-DESKTOP-13
object_kind: implementation.change
state: in_progress
title: 概览背景光定稿为 LunarLogic/auroral 方案并落到正式原型
summary: 用户在「背景光方案」对照页比较后确认各方案「基本没有什么差距」，选定 LunarLogic/auroral（纯 CSS 极光）方案并要求落到正式原型。做法：光团改为每个叠两层 radial-gradient、以 background-blend-mode:screen 相加成极光带；柔化只在光场容器上做一次 blur(20px)（不做逐层模糊，避免超帧预算）；颜色仍取状态投影的 --glow-a/b/c，九态配色不变。正式原型 原型/index.html 与配对的 候选/…/overview.html 共用同一段 hero 光场，两处同时生效。旧口径与对照用的方案 A/B 收进 glow-plain / glow-blur / glow-gradient 变体，只服务动效研究页。本阶段只落原型；软件同步（base.css + 门禁 + 重编打包）尚未开始，故不标记完成。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-183"]
source_refs: ["IMP-OPENCODEX-DESKTOP-12", "IMP-OPENCODEX-DESKTOP-11", "UI规范.md §25.1 / §25.2", "AC-OPENCODEX-DESKTOP-17"]
completion_summary: "未完成。原型侧已落地并通过检查，但软件侧（apps/desktop/ui 的 base.css、门禁断言、重编打包与制品验收）尚未开始，故不标记完成。"
---

# 概览背景光定稿为 LunarLogic/auroral 方案并落到正式原型

2026-10-02。承接 [IMP-12](IMP-OPENCODEX-DESKTOP-12.md)。

## 1. 决定

用户在「背景光方案」对照页看过全部方案后：**「基本没有什么差距，采用 LunarLogic/auroral 的方案」**。
选它的理由：四者在 920×704 模拟概览下观感接近，而 auroral 是**纯 CSS**——没有 canvas / WebGL 依赖，
低端机与禁用硬件加速时都能降级。

## 2. 落地内容（正式原型）

`候选/2026-09-28-Logo本体形变/index.html` 的 `.hero-embed` 光场：

- **材质**：每个光团叠**两层** `radial-gradient`，用 `background-blend-mode:screen` 相加（越叠越亮、过渡更连续），
  颜色仍取状态投影的 `--glow-a/b/c`，九种状态配色不变；
- **柔化**：只在**光场容器**上做一次 `filter: blur(20px)`（延续 IMP-11 的结论：逐层模糊会超帧预算）；
- **运动**：不变，仍由 `morph.js` 驱动——三团相隔 120° 绕中心 Logo 公转、各自自旋、按 120° 时差涨缩，能量束绕中心自转；
- **生效范围**：正式原型 `原型/index.html` 与配对的 `候选/…/overview.html` 共用同一段 hero 光场，两处同时生效；
- **对照变体**：旧口径与方案 A/B 收进 `glow-plain` / `glow-blur` / `glow-gradient`，只服务动效研究页，不参与正式口径。

## 3. 验证

| 项 | 结果 |
| --- | --- |
| 正式原型（1180×760，明/暗） | 光场 `filter: blur(20px)`、`background-blend-mode: screen, screen`、每团 2 层渐变；0 pageerror |
| 配对候选 overview（1180×760，明/暗） | 同上，两处一致 |
| 动效研究页 | auroral 条目改为真实 hero（正式口径）并默认选中；0 console 错误 |
| 原型 jsdom QA（`原型/qa`） | 0 失败 |
| 布局回归 | 对照页仍单屏、单滚动条、模拟概览不越界 |

## 4. 顺带治理：暗色色带（banding）

改用 auroral 后暗色出现明显色带。根因是 **8bit 量化**：大范围、低对比的平滑渐变会成片落在同一灰阶。
度量（原型 1180×760 dark，光场区取 12 条横线）：

| | 相邻像素完全相同占比 | 平均连续同值 | 最长连续同值 |
| --- | --- | --- | --- |
| 改前 | 58.2% | 3.45 px | 53 px |
| 改后 | 17.8% | 2.23 px | 6 px |

做法（两层）：
1. **渐变改感知均匀插值**：光团渐变加 `in oklab`（注意语法——`in oklab` 必须写在形状/位置**之后**：
   `radial-gradient(ellipse … at … in oklab, …)`；写在最前面是无效语法，整条声明会被丢掉）。
   同时保留一份 sRGB 声明在前兜底，不支持的浏览器自然回退。
2. **叠极轻的噪声抖动（dither）**：在**被 blur 的 `.ambient` 之外**（`.hero-embed .stage::after`）
   用 `feTurbulence` 灰色噪声 220×220 平铺（`baseFrequency .8` / 2 octaves），
   浅色 `opacity .03` / 深色 `.05`，并套用与光场相同的纵向渐隐蒙版。
   关键点：噪声不能放进 `.ambient`，否则会被容器上的 `blur(20px)` 一起糊掉。

噪声尺度也实测定标过：128px 平铺 / `baseFrequency 1`（约 1px 尺度）会被合成器平均掉、等于没加；
260/300px 又偏粗。220px / `.8` / 2 octaves 是实测最优点。

其它杠杆（`opacity .74→.68`、退回单层渐变）在本指标上收益不明显（17.5% / 19.6% vs 17.6%），
残余的 ~18% 同值像素集中在光已衰减到 0 的边缘（噪声被同款蒙版挡住），不是感知色带。

## 5. 未完成（不得标记完成）

- [ ] 同步软件 `apps/desktop/ui/src/styles/base.css`（`.motion-ambient` / `.cloud-*` 换成同一材质），
      并更新 `qa/audit.browser.mjs` 断言。
- [ ] 软件门禁：`typecheck` / `vitest` / `build` / `audit:browser` / 配对 72 页 + 84 状态 / 原型 QA 573 项。
- [ ] 复测 §7 性能预算（p95 ≤ 33.4ms）与边界跳变。
- [ ] 重编打包（`cd apps/desktop/tauri && npx tauri build --target aarch64-apple-darwin`）→ 记制品哈希 → 制品验收。

## 6. 修复：动效研究页暗色下模拟概览「还是白的」

2026-10-03。用户在动效研究页切到深色后，模拟概览 hero 仍是一大片淡色，看不出暗色效果。

**根因（已定位）**：研究页 `.ov-glow iframe` 的规则漏了 `color-scheme: normal`。
页面切深色时 `color-scheme: dark` 会传给 iframe，内嵌文档的画布被当成**白底**；
光场材质又是 `background-blend-mode: screen` 相加，叠在白色画布上直接冲成近白。
正式原型的 `overview-dual.css` 里 `.motion-backdrop` 本来就有
`background: transparent; color-scheme: normal`，所以原型一直正常，只有研究页复写了一处漏声明。

实测（hero 区域平均亮度，1440×900）：

| 状态 | 修复前 | 修复后 |
| --- | --- | --- |
| auroral 暗色 | 233 | 62 |
| auroral 浅色 | 221 | 221（不变） |

**修复**：`glow-demos.css` 的 `.ov-glow iframe/.glow-orbit` 规则补上 `background: transparent; color-scheme: normal`。

**顺带**：`bits`（DavidHDev/react-bits）条目暗色下用 `mix-blend-mode: screen`，材质含白色条纹同样会发白；
改为暗色下 `mix-blend-mode: normal`、`opacity .35`、能量束 `.22`，保留斜向条纹极光观感同时读得出暗底。
八个条目此刻暗色 hero 平均亮度 44–158（`bits` 因保留高光条纹偏高），浅色 200–229，0 pageerror。
