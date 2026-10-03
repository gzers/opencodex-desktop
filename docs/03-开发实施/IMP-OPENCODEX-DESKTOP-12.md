---
id: IMP-OPENCODEX-DESKTOP-12
object_kind: implementation.change
state: completed
title: 原型新增「背景光方案」对照 tab（左列表 / 右详情，多开对比）
summary: 用户要求把几种柔化与动态渐变方案都做出来对比、放进动效研究页并多加一个 tab，同时列出效果来源（外部项目或自制）。动效研究页新增「背景光方案」tab，采用「左列表 + 右详情」布局、默认全开、可任意多开：本项目自制 3 项（现状无柔化、方案 A 容器一层 blur、方案 B 横向×纵向渐隐无 blur，均为真实 hero iframe，888×660 与 overview 正文列同源，`?glow=base|blur|gradient` 切换）＋外部开源 4 项（LunarLogic/auroral 纯 CSS 极光、paper-design/shaders 风格 WebGL mesh+颗粒、ruucm/shadergradient 风格 WebGL 流光、DavidHDev/react-bits 风格 Canvas2D Aurora/Glow），外部四项均**按其公开思路自绘复现、不内联其源码**并在卡片上标注来源与星数。实测发现：在 889px 正文列这一几何下，现状/A/B 三者静态帧肉眼接近、边界光强都已接近背景，未复现此前的 13~15/255 硬边；`mask-composite:intersect` 已在 Chromium 生效但未观测到 A/B 可测差异，故方案 B 的边界/性能数字留白待复测，不写猜测值。原型 jsdom QA 0 失败、研究页 0 console 错误、概览嵌入态无回归。**本阶段只做原型对照，未选定方案、未改软件，故不标记完成。**
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-182"]
source_refs: ["IMP-OPENCODEX-DESKTOP-11", "IMP-OPENCODEX-DESKTOP-10", "UI规范.md §25.1 / §25.2", "AC-OPENCODEX-DESKTOP-17"]
completion_summary: "对照阶段完成。动效研究页「背景光方案」tab 已交付：左列表（缩略动效+名称+简介+来源）+ 右详情（920×704 模拟概览：顶栏／大 Logo／未运行／启动按钮／版本升级／配置迁移／WebDAV 同步／最近事件），支持多方案对照、深色预览与帧率读数；自制与外部方案的运动统一沿用概览光场模型（三光源 120° 公转 + 各自自旋 + 时差涨缩 + 能量束自转），外部方案背景透明可叠暗色窗口。用户查看后确认「基本没有什么差距」，选定 LunarLogic/auroral（纯 CSS 极光）方案，后续落地见 IMP-13。jsdom QA 0 失败、研究页 0 console 错误、概览嵌入态无回归。"
---

# 原型新增「背景光方案」对照 tab（现状 / A 容器模糊 / B 四向渐隐）

2026-10-02。承接 [IMP-11](IMP-OPENCODEX-DESKTOP-11.md)。用户在看过「容器一层 blur」的原型后提出
「现在效果明显边界，能不能还是模糊边界渐变」，随后确认「两个都做来对比，可以做到动效研究页面。多加一个 tab」。

## 1. 背景

- 光场（能量束 + 柔光团）比概览正文列宽，被容器 `overflow:hidden` 与蒙版裁切，理论上会在边界留下直线刀口。
- IMP-11 已把纵向渐隐改成长渐变（`6%/55%`）并去掉 blur，当时实测成像差 ≤0.5/255。
- 为了在「柔化边界」与「性能」之间做取舍，需要把两种做法放到同一页里直接对照。

## 2. 本阶段做了什么（仅原型）

- 动效研究页新增 tab：`形象动效`（原有）/ `背景光方案`（新增）。tab 通过 `data-lite-tab` / `data-lite-panel` 切换，
  嵌入态（`?embed=1`）下 tab 与对照区整体隐藏，概览背景光 iframe 不受影响。
- 新增变体开关：`?embed=1&layout=hero&glow=base|blur|gradient`。
  - `base`（默认）＝现状：只保留纵向长渐隐，不加柔化。
  - `blur`＝方案 A：`.hero-embed.glow-blur .ambient{filter:blur(20px)}`，整层只模糊一次。
  - `gradient`＝方案 B：`.hero-embed.glow-gradient .ambient` 用「横向 + 纵向」两层 `mask-image`
    以 `mask-composite:intersect` 取交集，`filter:none`，让光在到达边缘前淡到 0。
- 对照区用**真实的 hero iframe**（同 `overview.html` 的 `iframe.motion-backdrop` 管线），
  宽度取正文列 888px、高 660px，保证 `vw` 口径与概览一致；三张卡片同框展示现状 / A / B，
  支持「深色预览」按钮并联动两个页面主题按钮。
- 页面内附实测表，方案 B 的边界跳变与 p95 明确标注「待复测」。

## 3. 对照页形态（左列表缩略 + 右详情「模拟概览」，单屏）

- 布局：`glow-demos.css` + `glow-demos.js`。
  - 左列＝可滚动列表（它有独立滚动条，整页不滚），每项＝**缩略动效**（真实效果小窗）+ 名称 + 一句话介绍 + 来源标签/仓库链接/星数；
    分组「本项目自制」/「外部开源 · 自绘复现」。
  - 右列＝点击后的**放大详情**：把方案放进一个 **889×660 的「概览模拟」**里（顶栏 + 状态区 + 三张玻璃卡，
    与 `#card-overview-status` 同结构），光场在图层的真实位置渲染，玻璃卡用 `backdrop-filter` 透光；
    整个模拟按面板尺寸等比缩放（`transform:scale`，保留 `vw` 口径），下方是「做法 / 实现思路 / 代价·指标」。
  - 面板高度由脚本按视口算出（`innerHeight - 面板顶部 - 16`），并给 `html` 加 `glow-lock`
    （`overflow:hidden`）：对照 tab 打开时**整页不滚、高度不超过屏幕**，唯一的滚动条是左列表
    （`overscroll-behavior:contain`）；详情说明区不设内滚动，靠 stage 自适应让文字完整放下。
    切回「形象动效」tab 时解除锁，页面恢复常规滚动。
  - 详情说明分三栏：**做法 + 实现思路** / **代价与指标** / **适用建议**，均为完整句子。
- 本项目自制 3 项在模拟概览里用**真实 hero iframe**（与 `overview.html` 的 `iframe.motion-backdrop` 同管线、同 `top:-75px`）。
- 外部 4 项的**运动一律沿用概览光场模型**（`morph.js` `ambientTarget`）：三团光源相隔 120° 绕中心 Logo 公转
  （运行态约 12s/圈），每团按 `SPIN=[42, -33, 54]` 各自自旋，缩放按 120° 时差先后涨缩，能量束绕中心自转；
  各方案只替换材质/画质。演示把公转半径放大到 `210×150`（概览实值 `102×70`）以便看清，详情里已注明。
  `auroral`/`react-bits` 用 DOM 载体 `.orbit-cloud` 每帧写 `transform`；`paper-design mesh`/`shadergradient`
  在片元着色器内用同一组参数摆放三个光源（flow 另加绕中心自转的能量束）。
- 外部 4 项为自绘复现（非其源码）：
  | 方案 | 来源项目 | 星 | 复现做法 |
  | --- | --- | --- | --- |
  | 纯 CSS 极光渐变 | `LunarLogic/auroral` | 287 | 五层 `radial-gradient` + `background-blend-mode:screen` + 关键帧位移 + `blur(34px)` |
  | Mesh 渐变 + 颗粒 | `paper-design/shaders` · MeshGradient | 3.5k | 自绘 WebGL 片元着色器：5 个移动色锚按距离倒数归一化加权 + `fbm` 域扭曲 + 哈希颗粒（dither） |
  | 流光网格渐变 | `ruucm/shadergradient` | 2.7k | 自绘 WebGL 着色器：`fbm` 揉皱坐标 + 三色正弦场混合 + 丝绸条纹 + 暗角；原版需 Three.js，未内联 |
  | Aurora / Glow 组件 | `DavidHDev/react-bits` · Aurora | 48k | **复写其公开的两层 `repeating-linear-gradient`（100°）+ `background-size` 300%/200% + 60s 线性平移 + `blur(10px)`，深色用 `invert()`** |

## 4. 实测与结论（详见 `.adg/work/imp12-execution/evidence/measurements.md`）

| 变体（概览页 889 宽，dark） | 峰值 | 左/右边界 | 最大相邻列跳变 |
| --- | --- | --- | --- |
| base | 76.2 | −20.5 / −18.0（≈背景） | 6.58（x=491，渐变肩部） |
| blur | 75.8 | −20.5 / −17.9 | 6.88（x=491） |
| gradient | 77.9 | −18.5 / −15.9 | 6.91（x=491） |

结论（如实登记）：
1. 在 889px 正文列几何下，三者**当前静态帧肉眼接近**，边界处光强都已接近背景，**未复现**此前记录的
   13~15/255 硬边；是相位相关还是已被 IMP-11 消掉，需以复测为准。
2. `mask-composite:intersect` 在 Chromium 已生效（computed `intersect, intersect`），方案 B 横向渐隐已应用，
   但未观测到 A/B 的可测成像差异。
3. 帧内 rAF 间隔三者均为 16.7~16.8ms（垂直同步封顶），不能区分性能；真实差异需用
   `apps/desktop/ui/qa/audit.browser.mjs` 的 p95 口径复测。

沿用已确立的性能数字（本轮未复测）：现状 p95 ≈ 25.8ms；方案 A p95 ≈ 31.2ms；方案 B 待复测。

## 5. 门禁

| 项 | 结果 |
| --- | --- |
| 原型 jsdom QA（`原型/qa`，`node --test *.test.mjs`） | 0 失败 0 JS 错误 |
| 动效研究页加载（无头 Chromium） | 0 pageerror / 0 console error；DOM 自检：7 卡 / 3 iframe / 3 canvas |
| 概览嵌入态回归 | 无 tab、无对照区；背景光正常渲染 |
| 软件门禁 / 重编打包 | **本阶段未涉及**（未改软件） |

## 6. 未完成（不得标记完成）

- [ ] 用户在原型上选定方案 A 或 B。
- [ ] 选定后同步软件 `apps/desktop/ui/src/styles/base.css`，更新 `qa/audit.browser.mjs` 断言。
- [ ] 复测边界跳变与 p95，回填对照页与本文的「待复测」。
- [ ] 走完软件门禁（typecheck / vitest / build / audit / 配对 72 页+84 状态 / 原型 QA 573 项）→
      重编打包（`cd apps/desktop/tauri && npx tauri build --target aarch64-apple-darwin`）→ 记制品哈希。
