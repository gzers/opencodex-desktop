# 应用内公共组件

本目录是应用内部“多个功能可依赖”的公共界面组件入口（不是发布给外部的组件包）。目录与职责见 IMP-04 §4：

- `ui/`：基础控件与视觉部件（Button / Card / …）。统一交互、视觉与可访问性；**不读取业务 store、不直接调用 Tauri**。
- `layout/`：排列、间距、宽度、滚动、区域划分（Stack / Inline / …）。
- `patterns/`：由多个基础部件构成、语义稳定的通用结构（SettingRow / InlineNotice / …）。

业务组件（承载具体业务语义、可使用本功能状态）放在 `features/<name>/components`，不放进本目录；应用壳与装配组件放在 `app/`。

## 约定

- 组件只通过 props / slots 取内容，通过事件向上报告动作；额外类名与属性透传到根元素，保证既有 CSS 选择器仍然有效。
- 样式跟随组件；通用数值来自 `styles/tokens.css`，组件内不新造色板或字号。
- “只用一次”不是禁止拆分的条件；“在多个页面复用”也不是必须移入本目录的条件（业务组件就近维护）。

## 当前状态（IMP-04 阶段 C 进行中）

已建立（均映射既有类，属等值迁移）：

- `ui/UiButton.vue`：承载 `.btn` 配方，支持 `loading`。
- `ui/UiCard.vue`：承载 `.card` 表面配方，属性/类名透传。
- `ui/UiCardHeader.vue`：承载 `.card-head` 配方，默认插槽放标题/说明、`actions` 插槽放右侧操作。
- `patterns/SettingRow.vue`：承载 `.setting-row` 配方，`title`/`description` + 默认插槽 + `actions` 插槽。
- `patterns/MarkdownContent.vue`：唯一的 `v-html` 边界，渲染受限 Markdown（先转义、链接不留 `href`）；正文外观由所属容器的命名空间样式提供（如 `.ext-detail-md`）。
- `layout/Stack.vue`、`layout/Inline.vue`：纵向/横向排列原语（`gap` 取设计刻度，间距只由容器拥有）。

试点采用已落地：

- 概览页（`routes/OverviewRoute.vue`）：状态卡、模块卡用 `UiCard`，动作按钮用 `UiButton`。
- 扩展页（`routes/ExtensionsRoute.vue`）：Skills / MCP 两张主卡用 `UiCard` + `UiCardHeader`。
- 设置页（`routes/SettingsRoute.vue`）：桌面壳偏好卡用 `UiCard` + `UiCardHeader`，界面缩放行用 `SettingRow`。

开发态预览：`gallery.html`（Vite dev 下 `/gallery.html`），直接引用生产组件；正式构建只打包 `index.html`，不含该页。

后续（阶段 C 续）：`UiCardBody/UiCardFooter`（现有实现无 `.card-body`/`.card-footer`，需先形成视觉/结构决定，见 IMP-04 §7.5）、`patterns/InlineNotice`（当前无 `.notice` 消费者，待出现真实使用再提取）、组件规则检查（限制组件内新增颜色/字号/字重/层级）。
