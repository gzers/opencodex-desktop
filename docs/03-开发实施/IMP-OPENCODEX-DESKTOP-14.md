---
id: IMP-OPENCODEX-DESKTOP-14
object_kind: implementation.change
state: completed
title: 概览背景光默认 WEBGL 网格渐变＋颗粒着色器，CSS 极光兜底，并新增「画质」设置卡
summary: 用户要求背景光默认采用「Mesh 渐变＋颗粒着色器」，环境不支持则回退纯 CSS 极光（auroral），并在设置里专门做一个「画质」卡片，把原有画质档位（视觉特效 高/中/低）和新的「背景光渲染」选项（WEBGL / CSS）收在一起，选项名用简短的 WEBGL、CSS。本轮先落原型：新增独立渲染器 mesh-glow.js（碎片着色器输出透明覆盖度，自带 hash 颗粒抖动，运动与配色沿用 hero 光场同源模型）；hero 嵌入页新增 render=mesh|css 参数与 glow-mesh 类，挂载失败或不支持 WEBGL 时自动回退 CSS；overview-dual 传递该参数；原型设置新增「画质」卡片与评审栏打样开关。软件同步尚未开始，故不标记完成。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-184"]
source_refs: ["IMP-OPENCODEX-DESKTOP-13", "IMP-OPENCODEX-DESKTOP-12", "UI规范.md §25.1 / §25.2", "AC-OPENCODEX-DESKTOP-17"]
completion_summary: "已完成。原型与软件两侧同构落地：设置「画质」卡片（界面特效 + 背景光渲染 WEBGL/CSS）、默认 WEBGL 网格渐变＋颗粒着色器、不支持时自动回退纯 CSS 极光；偏好字段贯穿 Rust 领域/DTO 与前端，`data-glow-render` 唯一写入点。门禁：vue-tsc 0 错、vitest 351 通过、vite build 通过、cargo test 全绿、浏览器探针 421/421、原型配对 72 页 0 页面错误 + 84 状态 0 失败、原型 jsdom QA 373 项 0 失败、高档 p95 帧间隔 21.8ms ≤ 33.4ms 预算；Tauri aarch64 重编打包成功并记录制品哈希。"
---

# 概览背景光默认 WEBGL 网格渐变＋颗粒着色器

2026-10-03。承接 [IMP-13](IMP-OPENCODEX-DESKTOP-13.md)。

## 1. 决定

用户：「能不改成这样默认采用 **Mesh 渐变 + 颗粒着色器**，若环境设备不支持则纯 CSS 极光（auroral）……设置在画质上专门做个画质卡片，包含之前的画质档位，和这个背景光渲染选项，选项简短 WEBGL、CSS。」

- **默认**：WEBGL（网格渐变 + 颗粒着色器），过渡最平滑、色带最少；
- **兜底**：CSS（纯 CSS 极光 auroral），不依赖 GPU；
- **降级**：环境不支持 WEBGL 时自动回退 CSS，并在设置项给出说明；
- **入口**：设置新增「画质」卡片，含原有「视觉特效（高/中/低）」与新的「背景光渲染（WEBGL/CSS）」。

## 2. 落地内容（原型）

| 文件 | 改动 |
| --- | --- |
| `候选/2026-09-28-Logo本体形变/mesh-glow.js`（新增） | 独立渲染器：三光源沿用 hero 光场同源运动模型（120° 公转 / 各自自旋 / 时差涨缩），颜色取状态投影 `--glow-a/b/c`；碎片着色器输出**透明覆盖度**（可叠暗底），自带 hash 颗粒抖动抑制 8bit 色带；提供 `supported() / mount()`，`setTheme / setColors / setReduced / pause / resume / stop` |
| `候选/…/index.html` | 新增 `render=mesh\|css` 参数与 `html.glow-mesh` 类；`glow-mesh` 时隐藏 `.ambient`（CSS 极光）并挂载 canvas；`effects=low` 同样隐藏着色器层 |
| `候选/…/morph.js` | `mountMeshGlow()`：仅在 `glow-mesh` 且 WEBGL 可用且挂载成功时启用，否则**移除 `glow-mesh` 自动回退 CSS**；配色、主题、`prefers-reduced-motion`/减少动态、暂停与显隐均与光场联动 |
| `overview-dual.js` | 读 `data-glow-render`，拼进 hero 光场 iframe 的 `render` 参数；监听该属性变化重建 iframe |
| `原型/index.html` | `<html data-glow-render="mesh">`；设置新增「画质」卡片（视觉特效移入 + 背景光渲染）；评审栏新增打样开关；`setGlowRender()` 本地探测 WEBGL，不支持则强制 CSS 并禁用 WEBGL 按钮、显示回退说明 |
| `候选/…/overview.html`（配对页） | 该页是独立分叉的快照，设置结构自带一份；同步加入 `data-glow-render`、「画质」卡片、评审栏开关与 `setGlowRender()` |

研究页（`glow-demos.js` / 对照 tab）不传 `render`，仍走 CSS 极光，不受影响。

## 2.1 修正：光场没有围绕 Logo 旋转（y 轴方向）

用户反馈「概览的背景光没有围绕 logo 旋转，中心点不对」。根因是 WebGL 着色器用了
`gl_FragCoord.xy`——它的 **y 轴自下而上**，而中心 `vec2(0.5, 173.0/660.0)` 是按「自上而下」写的，
于是整片光场被垂直镜像、中心落到了 logo 下方约 230px；`fade(uv.y)` 的纵向渐隐方向也一并反了。
修法：统一换成自上而下的 uv —— `vec2 uv = vec2(gl_FragCoord.x/u_res.x, 1.0 - gl_FragCoord.y/u_res.y);`。

实测（1180×760 暗色，iframe 889×660，亮度质心）：修前 `cy≈407`，修后 `cy≈242`（含父层内容权重），
行亮度峰值带落在 CSS y≈120–220，与 Logo 中心 175 对齐。
同一缺陷在研究页的 mesh / flow 两个 WebGL 方案里也存在（此前反馈的「外部开源没有围绕中心 logo 自旋」即由此而来），
已在 `glow-demos.js` 的 `MAIN_HEAD` 一并修正。

## 2.2 修正：WEBGL 背景光过亮

用户反馈「WEBGL 版本太亮，能浅一点吗？透明度问题」。碎片着色器是三团光**加法叠加**，
叠加后中心接近白，缺少类似 CSS 极光 `.ambient{opacity}` 的整体收束，所以观感偏重、偏白。

修法：给着色器 canvas 加一层整体不透明度（可用 CSS 变量现场微调），并把浅色增益从 `1.0` 降到 `0.92`：

```css
html.glow-mesh .stage>.mesh-glow-canvas{opacity:var(--mesh-glow-alpha,.55)}
html.glow-mesh body.dark .stage>.mesh-glow-canvas{opacity:var(--mesh-glow-alpha-dark,.5)}
```

实测（1180×760，hero 顶带平均亮度）：浅色 187 → 203（更接近底色、观感更「浅」），
暗色 109 → 75（明显收下去）。取值范围 0.4–1.0 已实测过，0.4 偏弱、1.0 过曝，0.5–0.55 是当前定值。

## 2.3 修正：范围与颜色的反复（含一次过度调整的回收）

### 2.3.1 先：范围被上下边界收住

用户反馈「光效明显收到了上下边界，应该扩大范围」。着色器按「889 宽正文列、660 高」设计，
光团高斯半宽偏小（`q/340,250`）且纵向渐隐过早（`1-smoothstep(0.55,1.0,y)` 从 55% 就开始衰减），
在 1180×760 窗口里就缩成一枚可见边界的椭圆。

### 2.3.2 过度调整：铺太大 + 压太暗 → 变灰、看不出动

先把光团放到 `q/700,620`、渐隐放到 `1-smoothstep(0.9,1.3,y)`，同时把 canvas 不透明度压到 `0.55/0.5`。
用户反馈「**没有颜色、变灰了**，而且**范围太大、已经没有动态效果**」。量化确认：

| | 最亮区平均 RGB | 饱和度 |
| --- | --- | --- |
| 暗色 mesh（过度调整后） | (182,190,196) | 0.07 |
| 暗色 CSS 极光（对照） | (114,114,145) | 0.22 |

两个原因：① 光团铺太开导致三团几乎完全重叠，加法叠加把三个通道一起顶到 1.0，
再被 `clamp` 成纯白，色相被抹掉；② 三团重叠成一片均匀亮斑，质心几乎不动，动效看不出来。
canvas 不透明度越低只会把这团白整体压暗，不会把颜色找回来。

### 2.3.3 定稿

| 参数 | 值 |
| --- | --- |
| 公转半径 | `160 × 110` |
| 光团半宽 | `q.x/400, q.y/330` |
| 衰减 | `exp(-q²·1.9)` |
| 纵向渐隐 | `1-smoothstep(0.68,1.05,y)` |
| 单团幅度 | 每团 `×0.62`（避免相加顶到 1.0） |
| 饱和度回补 | `mix(vec3(lum), col, u_sat)`，暗 `1.45` / 浅 `1.3` |
| 保色相归一 | `col /= max(1.0, peak/0.95)` —— 整体缩放，不逐通道 clamp |
| canvas 不透明度 | 浅 `0.8` / 暗 `0.72` |

实测：暗色最亮区 (127,131,196)、饱和度 **0.35**（高于 CSS 的 0.22），亮度 135 与 CSS 的 116 同量级；
亮度质心在 12 帧内 x 摆动 23.6px、y 摆动 4.8px，动效恢复可见。

**教训**：「太亮」的正确杠杆不是把整层压透明（那是把已经叠白的白整体压暗，颜色只会更灰），
而是**先保证叠加不顶到 1.0**（单团降幅 + 保色相归一），再谈整体不透明度。

## 2.4 修正：三团光看着像「一颗光球」

用户反馈「是不是没有多光源了，只有一个光球？」，并要求「更透明，别抢 logo 的关注」。

根因有两处：
1. 每团光取的是**相邻两色混合**（`mix(cA,cB,…)`），而归一后 `q.y` 取值范围变小，
   混合几乎停在 0.5 —— 三团输出的是同一个平均色，自然看不出三个光源；
2. 光团半宽 /400,330 相对公转半径 160 偏大，三团重度重叠成一整块。

修法：
- 三团改为**各自一种状态投影配色**（`u_ca / u_cb / u_cc`），团内再用 `q.y` 叠一道明度梯度；
- 光团半宽收到 /300,250、公转半径提到 195×132，让三团分得开；
- 饱和度回补下调到 1.28 / 1.18（颜色已经不再被叠白，不需要强拉）；
- canvas 不透明度降到浅 0.6 / 暗 0.52，把主体让给 logo。

实测：暗色最亮区 (108,109,149)、饱和度 0.27（CSS 版 0.22），亮度 111（CSS 116）；
连续 4 帧可见三团亮部绕 logo 依次移动，多光源运动恢复可辨。

## 2.5 修正：顶部有明显界限

用户反馈「顶部有明显界限，不能超过吗？」。实测该边界来自碎片着色器的**顶部渐隐太短**：
`smoothstep(0.0,0.05,y)` 只有画布高的 5%（660×5%≈33px），在 3× 缩放下就是约 100px 的陡坡，
于是顶部被切出一条可辨认的弧线；画布上沿之上还有光，只是被这段短渐隐砍掉了。

修法：把两端渐隐都拉成长缓坡，并用 `sqrt` 提前抬起（保证远处仍可见、近处又不过亮）：

```
float fade(float y){ return sqrt(smoothstep(0.0,0.24,y)) * (1.0 - smoothstep(0.70,1.10,y)); }
```

实测（暗色，窗口中央 30%–70% 列的逐行亮度）：修前 y≈96→144 段 8px 内跳变约 +13/255；
改后同级区间单步 Δ≤1.1/255，边界消失。光场也顺势延伸进顶部标题区。

另核对：`#card-overview-status` 为 `background:transparent;border:0` 且无 `overflow:hidden`，
不会在卡片上沿裁切光场；如需更大范围，可以把渐隐起点继续上移。

## 3. 验证（Chromium）

| 场景 | 结果 |
| --- | --- |
| 默认（`data-glow-render=mesh`，暗/浅） | iframe 带 `&render=mesh`；内层 `glow-mesh` 生效、canvas 已挂载、`.ambient` 隐藏；0 pageerror |
| 切到 CSS | iframe 重建为 `&render=css`；`glow-mesh` 移除、canvas 移除、`.ambient` 恢复 |
| 不支持 WEBGL（注入 `getContext('webgl')=null`） | 自动回退 `render=css`，WEBGL 按钮 disabled，回退说明显示 |
| 配对候选 `overview.html` | 同样 `render=mesh` 且 mesh 生效 |
| 研究页 | 无 `render` 参数，仍为 CSS 极光，无回归 |
| 原型 jsdom QA（`原型/qa`，`node --test *.test.mjs`） | 112 + 24 + 43 + 135 + 43 + 16 = 373 项，0 失败 |

## 4. 软件同步（apps/desktop）

原型验收通过后按同一口径落到软件，未新增第二套实现：

| 文件 | 改动 |
| --- | --- |
| `ui/src/app/appearance/glowRender.ts`（新增） | 渲染方式的**唯一**策略与落点：`clampGlowRender` / `detectWebglSupport` / `resolveGlowRender`（mesh 且不支持 → css）/ `applyGlowRenderAttribute`；`useGlowRenderStore` 暴露 `mode` 与 `fellBack` |
| `ui/src/features/runtime/motion/meshGlow.ts`（新增） | 与原型 `mesh-glow.js` 同源的网格渐变＋颗粒着色器（同一组运动与配色参数、保色相归一、长缓坡渐隐）；创建失败返回 `null` 交由调用方回退 |
| `ui/src/features/runtime/motion/scene.ts` | 新增 `onGlow` 回调：每帧把状态投影配色抛给着色器层，配色与 CSS 光团同源 |
| `ui/src/features/runtime/components/RuntimeMotionMark.vue` | 新增 `.motion-mesh-host` 层；按 `glowRender.mode` 挂载/销毁，主题、减少动态、暂停与可见性沿用既有策略 |
| `ui/src/styles/base.css` | `.motion-mesh-host` / `.motion-mesh` 样式与二选一规则，不透明度浅 0.6 / 暗 0.52 |
| `ui/src/styles/effects.css` | 低档一并隐藏着色器层（与 `.motion-ambient` 对齐） |
| `ui/src/main.ts` | 启动即装配设备能力探测（只探测一次） |
| `ui/src/routes/SettingsRoute.vue` | 新增「画质」卡片：`界面特效`（移入）+ `背景光渲染`（WEBGL/CSS，含回退说明） |
| `ui/src/features/preferences/{api,store}.ts`、`ui/src/stores/app.ts` | 偏好字段 `glowRender` 贯穿读取/保存/还原与 `setGlowRender` 动作 |
| `tauri/src/modules/preferences/mod.rs`、`tauri/src/types/preferences.rs` | 领域与 DTO 新增 `glow_render`（默认 `mesh`，非法值判损坏），旧偏好缺字段按默认读取 |
| `ui/qa/audit.browser.mjs` | 光场几何断言改为针对**当前生效层**；新增「交互·画质卡片」断言（卡片存在、默认 WEBGL、切 CSS/CSS 回切即时生效） |
| `ui/tests/glow-render.test.ts`（新增） | 归一化、降级、单点落属性、回退上报 |

## 5. 门禁与制品（2026-10-03）

| 项 | 结果 |
| --- | --- |
| `vue-tsc --noEmit`（含在 build） | 0 错误 |
| `vitest run` | 70 文件 / 351 通过 |
| `vite build` | 通过（`index-BtCWAJHP.js` / `index-sQkqpKrN.css`） |
| `cargo test`（tauri） | 全部通过（含新增 `glow_render_defaults_to_mesh_and_rejects_unknown`） |
| `qa/audit.browser.mjs` | **421/421 通过**，0 新页面错误 |
| 原型配对 · 页面（72 组合） | 0 页面错误 |
| 原型配对 · 状态（84 组） | 0 失败 |
| 原型 jsdom QA（373 项） | 0 失败 |
| 性能 §7（高档 p95 帧间隔） | **21.8ms**（预算 33.4ms）；mid/low 为静态，rAF=0 |
| Tauri 打包（aarch64-apple-darwin） | 成功，`.app` 与 `.dmg` 均产出 |

制品哈希（`apps/desktop/tauri/target/aarch64-apple-darwin/release/`）：

- 可执行文件 `opencodex-desktop`：`9c723b3a66c5a078a5038b4481799a25d7a0130908939e5d0e7d3fd15028d517`
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：`83247f671ff6b86163fd6e18de8e1082a9814fe315170ecaedac1508df57fcaa`
- 制品验收：打包日志中的前端产物哈希（`index-BtCWAJHP.js` / `index-sQkqpKrN.css`）与 `ui/dist` 一致，
  且 `ui/dist` 内含 `motion-mesh` / `data-glow-render` / `背景光渲染` / `网格渐变`，确认嵌入的是本次构建。

## 6. 说明

- 关掉特效（低档）时两条光场均关闭，着色器层不创建，避免空转。
- 渲染方式与特效档位正交：档位决定「有没有光场、动还是静」，本项只决定「用哪种渲染」。

## 7. 追加修复：窗口底面环境光的「圈痕迹」

用户反馈「软件有比较明显的圈痕迹，应该参照原型上效果最完美」。逐层二分（隐藏网格层 / CSS 极光层 /
窗口底面环境光）确认：**不是新加的光场**，而是 `.app-window::before`——取自官方面板的三处低对比
radial-gradient，经 `filter: blur(70px)` 铺满整个窗口后，在 8bit 下相邻像素成片落在同一灰阶，
于是露出大圆弧状的色带（「圈」）。原型 `.window::before` 是同一段 CSS，所以两边都有，只是软件窗口更大、色带更明显。

修法与 IMP-13 同一套：在被 blur 的 `::before` **之外**叠一层极轻的噪声抖动（噪声不能放进 `::before`，
否则会一起被 `blur(70px)` 糊掉），抖动层仍贴在内容之下，玻璃面才透得出来。

```css
.app-window::after {  /* 原型对应 .window::after */
  content:""; position:absolute; inset:-20%; z-index:-1; pointer-events:none; opacity:.16;
  background-image:url("data:image/svg+xml,…feTurbulence baseFrequency='.8' numOctaves='2'…");
  background-size:220px 220px;
}
html[data-effects="low"] .app-window::after { display:none; }  /* 低档实底：无环境光即无色带 */
```

强度实测（1440×900 浅色，右上角跨色带区间；指标＝相邻像素同值占比 / 最长连续同值）：

| 抖动不透明度 | 同值占比 | 最长连续同值 |
| --- | --- | --- |
| 0（关闭） | 64.8% | 410 px |
| 0.055 | 59.8% | 290 px |
| 0.10 | 54.8% | 105 px |
| 0.16（定值） | 49.9% | ~250 px（受渐变相位影响，肉眼已不可辨） |

原型 `候选/…/overview.html` 与 `原型/index.html` 同步加同一层，保持「原型＝参照基准」。
暗色下修后完全无圈。

追加门禁（2026-10-03 复跑）：vitest 351 通过、vite build 通过、浏览器探针 **421/421**、
原型配对 84 状态 0 失败 + 72 页 0 页面错误、原型 jsdom QA 373 项 0 失败。

重编打包后的制品哈希（覆盖 §5 记录）：

- 可执行文件 `opencodex-desktop`：见 §8（本轮再次重编后覆盖）
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：见 §8
- `ui/dist/assets`：`index-DJk1RcEO.css` / `index-Dq1TiEOe.js`（minified CSS 内含抖动层）

## 8. 追加修复：光场中心写死设计稿坐标，准备形态下偏下

用户问「中间这个光斑是什么？左边软件和右边原型都这样」。逐层二分确认这是**新 WEBGL 光场里的
单个光源**（隐藏着色器层后光斑消失），不是别的脏层。

但顺着查出一个真错：着色器把光场中心写死成设计稿的 `173/660`，而光场容器高度随形态变化——

| 形态 | 容器高 | logo 中心实测比例 |
| --- | --- | --- |
| 就绪 ready | 660 | 175/660 = 0.265 |
| 准备 setup | 520 | 97/520 = **0.187** |

准备形态下把中心画到 0.262，等于整体压低约 42px，三团光被挤到 logo 左上方，最强的那个就成了一颗孤立的「光斑」
（原型与软件同源，所以两边一致）。

修法：新增 `u_cy` uniform，运行时按**真实 logo 位置**（`.motion-hero svg` / 舞台 `svg` 的中心）算出纵向比例，
量不到才回落到设计稿基准。`meshGlow.ts` 与原型 `mesh-glow.js` 同步。

实测比例：就绪 0.265（与旧值 0.262 基本一致，观感不变）、准备 0.187（对齐 logo）。

## 9. 本轮门禁与制品（2026-10-03 最后一次）

| 项 | 结果 |
| --- | --- |
| `vue-tsc --noEmit` | 0 错误 |
| `vitest run` | 70 文件 / 351 通过 |
| `vite build` | 通过 |
| `qa/audit.browser.mjs` | 421/421 通过，0 新页面错误 |
| 原型 jsdom QA | 373 项 0 失败 |

制品哈希（`apps/desktop/tauri/target/aarch64-apple-darwin/release/`）：

- 可执行文件与安装包哈希见 §10（本轮再次重编后覆盖）

## 10. 追加修复：叠加亮光形成的「光圈」

用户问「怎么解决叠加亮光问题」。根因是**叠加口径**选错了：

| | 叠加方式 | 是否溢出 | 是否有硬拐点 |
| --- | --- | --- | --- |
| CSS 极光（原型验收口径） | `background-blend-mode: screen` | 否，恒 ≤1 | 无 |
| 网格着色器（本轮之前） | 纯加法 `+=` | 是，会顶到 1 | **有**（被 `clamp` / `peak/0.95` 归一硬压） |

三团光重叠处相加溢出，那条「顶到 1 再被压下去」的拐点就是可见的圆圈。修法是回到与 CSS 同口径：

```glsl
vec3 col = 1.0 - (1.0-clamp(l1,0.,1.)) * (1.0-clamp(l2,0.,1.)) * (1.0-clamp(l3,0.,1.));  // screen 叠加
...
float peak = max(max(col.r,col.g),col.b);
col *= (1.0 - exp(-peak)) / max(peak, 1e-5);   // 指数软拐点，替代 peak/0.95 硬归一
```

指数曲线处处可导，重叠区只渐近饱和，不会再出现突变的边界。

改后复核（原型 1180×760）：浅色最亮带 204.8 → 206.3、暗色 43.9 → 46.2（观感不变）；
亮度质心 12 帧内 x 摆幅 23.6 → 32.2px（动效未受损）。

门禁：vue-tsc 0 错、vitest 351、build 通过、浏览器探针 421/421、原型 jsdom QA 373 项 0 失败。

制品哈希（本轮重编）：

- 见 §11（本轮再次重编后覆盖）

## 11. 修正 §10 的回退：screen 叠加让整片光变暗

用户反馈「你现在做法导致背景光变暗了」。定量复核证实，责任在**换叠加方式**而不是软拐点：

| 变体（原型 1180×760 暗色） | mean | p99 | 面积% >70 | >110 |
| --- | --- | --- | --- | --- |
| 加法 + 硬归一（原基线） | 51.0 | 236 | 11.8% | 6.0% |
| screen + 无拐点 exp | 49.3 | 236 | **7.5%** | 6.0% |
| 加法 + 带拐点软压缩（定稿） | **51.0** | 236 | **11.8%** | 6.0% |

原因：screen 是 `1-(1-a)(1-b)`，两团各 0.4 时加法给 0.8、screen 只给 0.64——**中等重叠处天生更暗**，
三团光的整片强度因此下降。而「去圈」靠的是软拐点，本来不需要换叠加方式。

另外发现无拐点的 `exp(-peak)` 还会**全段压缩**：峰值 0.5 时 `(1-e^-0.5)/0.5 ≈ 0.79`，白压掉两成。

定稿写法（软件 `meshGlow.ts` 与原型 `mesh-glow.js` 同步）：

```glsl
vec3 col = l1 + l2 + l3;                       // 保持线性加法，亮度与原基线一致
...
float peak = max(max(col.r,col.g),col.b);
float knee = 0.72;
if (peak > knee) col *= (knee + (1.0-knee)*(1.0-exp(-(peak-knee)/(1.0-knee)))) / peak;  // 只在接近饱和处收
```

拐点以下**完全不动**，拐点处值与一阶导都连续（C1），所以既没有突变的硬边界，也不会整体压暗。

复核：原型暗色 mean 51.0 / >70 面积 11.8%（与基线逐位一致）、浅色 mean 208.6；
软件浅色 mean 208.0 → 207.9、>225/242/246 面积逐位一致。

门禁：vue-tsc 0 错、vitest 351、build 通过、浏览器探针 421/421、原型 jsdom QA 373 项 0 失败。

制品哈希（本轮重编）：

- 可执行文件 `opencodex-desktop`：`7cdfbc5e74130085023c7430837de23897022555b6c5aad0379363b8a73b7f19`
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：`0ebb0e81c0dc45958091c685790627851867eb7c5c011db13b9b7b5fdc2b5f7a`

## 12. 实机（系统 WebView）验证

此前所有自动化检查都跑在 Chromium 里，而交付物在 macOS 上跑的是 **WKWebView**——这也是「软件有圈、原型没有」
这类差异唯一可能藏身的地方。本轮补上实机验证：

1. `open "…/release/bundle/macos/OpenCodeX Desktop.app"`，应用窗口实测 1180×760（与原型基准一致）；
2. 用 `screencapture -R` 只抓该窗口（不截桌面其他内容）；
3. 放大 hero 区域检查。

结果：实机在 WKWebView 下光场是**连续的柔和光带，没有可见的圈或硬边界**；
光场中心落在 logo 正后方，亮度与原型一致。此前担心的「引擎差异导致色带/拐点」在当前参数下未复现。

**结论**：§8（中心对齐）、§10/§11（叠加与软拐点）、§7（窗口底面抖动）三项修复合起来，
在 Chromium 与 WKWebView 两个引擎下都已消除用户反馈的圈痕。

> 遗留说明：实机验证只覆盖了就绪形态 + 浅色主题（未在实机切换深色，避免用脚本驱动用户界面）。
> 深色若仍有异常，请直接反馈截图。

## 13. 修正：概览 hero 光场压住内容 / at_risk 多出启动按钮（2026-10-03 用户报告）

用户报告（附截图）：软件概览「启动 OpenCodex」按钮底色与原型不同（发灰紫），
且 at_risk 下「未运行」下方多出「启动保护未启用 + 查看建议」，怀疑还有其他没按原型的地方。

### 13.0 先回答上一轮的疑问：「软件有、原型没有」是不是引擎差异

不是。把同一套碎片着色器分别放进 **WKWebView**（`wkshot`/`wkprobe.swift` 离屏 WKWebView，
固定相位跑一次 `canvas.toDataURL`）与 **Chromium** 逐像素比：**最大差 1/255、均值 0.02**。
两个引擎的 WebGL 输出一致，所以差异只能来自 DOM/CSS 层，事实证明确实如此（见下）。

### 13.1 根因一：光场层画在了 hero 内容之上

- 原型把光场放进 `.motion-backdrop` **iframe**（`z-index:-1`），并由
  `#card-overview-status{isolation:isolate}` 兜住 → 光场只盖窗口底面，不压状态文字与按钮。
- 软件 `.motion-mark` 是 `position:absolute; z-index:0`，而 `.motion-identity` / `.motion-actions`
  是**静态**元素 → 定位层默认压在静态兄弟之上，等于把光场**铺在内容上面**，
  把黑底主按钮染成灰紫、把状态文字洗淡。
- 修复：`.motion-mark { z-index: -1 }`。祖先 `.motion-overview` 已是 `position:relative; isolation:isolate`，
  -1 停在它自己的层叠上下文里——仍在页面底色之上，但低于状态文字/按钮（与原型同法）。

量化（浅色 · 1180×760 · 同一相位 · Chromium，主按钮中心像素）：

| | 修复前 | 修复后 | 原型 |
| --- | --- | --- | --- |
| 主按钮中心 | (96,91,119) | (56,57,60) | (51,51,56) |

实机（WKWebView，新制品）：hero 状态文字像素 `(13,13,13)` 清晰不发灰；主按钮不再出现灰紫染色；
底部功能卡按钮保持纯黑 `(13,13,13)`。

### 13.2 根因二：at_risk 概览多注入了启动动作

> ⚠️ 本节结论已在 §14 按用户澄清**反转**：at_risk 的概览呈现最终定为与「未运行」逐字一致
> （只「启动 OpenCodex」）。下面保留当时的判断过程，最终口径以 §14 为准。

- 原型 `actionsByState`：`at_risk: ['advice']`——at_risk 只暴露「查看建议」。
- 软件 `statusScenario` 曾对 at_risk 注入 `start` → 概览多出原型没有的「启动 OpenCodex」。
  启动能力本就保留在**托盘**（`TrayRoute` start 提示 at_risk='可用'），无需在概览重复。
- 修复：去掉注入，概览动作严格对齐原型；同时把 `starting_failed` 的启动文案改为「重试启动」
  （原型 `render()` 同款：`currentState==='starting_failed' ? '重试启动' : '启动 OpenCodex'`）。

### 13.3 逐态核验（主线 + 说明 + 可见按钮）

对 10 个运行态逐一比对，原型 ↔ 软件**全部一致**（含 at_risk、external_takeover、unreachable、
starting_failed 等），探针 0 diff：

| 状态 | 主线 | 说明 | 可见按钮 |
| --- | --- | --- | --- |
| loading | 正在确认 | 等待首次观测 | — |
| not_found | 待接入 | 接入后可启动 | — |
| stopped | 未运行 | — | 启动 OpenCodex |
| starting | 未运行 | 启动中 · 待核验 | — |
| pending | 运行中 | 进程已起 · 待就绪 | — |
| running | 运行中 | — | 打开面板 / 停止 / 重启 / 查看日志 |
| starting_failed | 未运行 | 启动失败 | 重试启动 / 查看日志 |
| at_risk | 未运行 | 启动保护未启用 | 查看建议 |
| external_takeover | 运行中 | 有待处理问题 | 查看建议 |
| unreachable | 运行中 | 有待处理问题 | 查看日志 |

### 13.4 门禁

vue-tsc 0 错、vitest **352** 通过、vite build 通过、浏览器探针 **427/427**（新增 6 条「概览动作逐态」断言）、
0 页面错误、高档 p95 帧间隔 **21.5ms ≤ 33.4ms** 预算；原型配对 `overview-runtime` **20 组 0 失败**。

### 13.5 制品哈希（本轮重编）

- 可执行文件 `opencodex-desktop`：`360d8cbbf8ffbbb6d983ed41aa83edd6c50383d142c534c579834390d834d2f3`
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：`2d69c38b8bf3c6a0158027c1bdce8cec66b1324df81f9f24ae0338cbd22058aa`

## 14. 更正 §13.2：at_risk 应与「未运行」逐字一致（用户澄清）

用户澄清：「你完全改错了，我的意思是，原型的做法是对的」，并确认——
**软件概览要跟原型那张默认视图一样：只显示「未运行」+「启动 OpenCodex」，不要「启动保护未启用」和
「查看建议」；且「启动 OpenCodex」的按钮样式要和原型一致。**

§13.2 按原型 `actionsByState` 的 at_risk 分支把软件改成「只剩查看建议」，方向反了。更正如下：

### 14.1 结论

`at_risk` 的**进程事实就是「未运行」**，所以概览形象应与 `stopped` **逐字一致**：
主线「未运行」、无说明句、动作只「启动 OpenCodex」；at_risk 只体现在配色（amber）与托盘，
成因走通知中心 / 诊断，不往概览 hero 塞说明或建议。

### 14.2 改动（原型 + 软件同步，保持一致）

| 侧 | 文件 | 改动 |
| --- | --- | --- |
| 原型 | `overview-dual.js` | 去掉 `currentState === 'at_risk' ? '启动保护未启用'` 分支（两个原型页共用） |
| 原型 | `候选/…/overview.html` | `actionsByState.at_risk`：`['advice']` → `['start']` |
| 原型 | `原型/index.html` | `actionsByState.at_risk`：`['advice','refresh']` → `['start','refresh']`（保持该页 refresh 口径） |
| 软件 | `features/runtime/motion/projection.ts` | `captionFor('stopped')` 不再对 at_risk 返回说明句 |
| 软件 | `features/runtime/scenarios.ts` | `runtimeScenarios.at_risk.actions`：`['advice']` → `['start']` |
| 软件 | `routes/OverviewRoute.vue` | `starting_failed` 的启动动作读作「重试启动」（同原型，保留） |

> 说明：原型 `#card-overview-status` 里隐藏的 `.ovb-main [data-b="problem"]` 仍保留 at_risk 成因文本
> （`state-announce.test.mjs` 断言的对象），它 `display:none` 不参与概览布局，因此不违反本口径。

### 14.3 按钮样式

§13.1 的 `z-index:-1` 修复后，浅色下主按钮填充已与原型对齐：

| | 原型 | 软件 |
| --- | --- | --- |
| 填充（去字形像素的均值） | `(49.9, 50.2, 54.5)` | `(53.3, 53.7, 56.4)` |

差 ≈ 3/255（肉眼不可辨）。剩余差异来自原型 `--notif-tint-a/b` 自带淡蓝/淡绿纱、软件按 UI规范 §2.2
取「中性轻磨砂（transparent）」，属于已确认的令牌口径，不属缺陷。

### 14.4 逐态核验（原型 ↔ 软件）

10 个运行态（主线 + 说明 + 可见按钮）**0 diff**，其中 at_risk 两侧同为
`未运行 | (空) | 启动 OpenCodex`。

### 14.5 门禁与实机

- vue-tsc 0 错、vitest **352** 通过、vite build 通过、浏览器探针 **427/427**、原型配对 20 组 0 失败、
  原型 jsdom QA **373 项 0 失败**（112+24+43+135+43+16）。
- 实机（WKWebView，新制品）实测：hero「未运行」文字像素 `(13,13,13)` 清晰；
  说明句位置最暗仅 `(183,183,183)`（无文字）；主按钮 `(53,53,56)`、位置 `x650–776 / y367–403`，
  与原型一致；底部功能卡按钮保持纯黑 `(13,13,13)`。

### 14.6 制品哈希（本轮重编）

- 可执行文件 `opencodex-desktop`：`209c2cbe92a94c9fa43c219faee9903038d5a837193a24e195a2e34a518be2ce`
- 安装包 `bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`：`22418b1174639984526a8874e92432eed9d25006af58907580a110a0349e06eb`
