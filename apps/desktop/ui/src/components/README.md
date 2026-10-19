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

## 0.2.0模型切片新增接入

| 专题稳定ID | 正式实现 | 当前消费者与边界 |
| --- | --- | --- |
| choice.select | ui/UiSelectField + 既有UiSelectMenu | 模版版本/规则字段及gallery；modelValue/options/label/disabled与sm/md，复用菜单定位与键盘，不自动保存 |
| choice.slider | ui/UiDiscreteSlider | gallery及模型选择器连续设置；只在调用方真实离散集合吸附，支持默认/未知、键盘/滚轮、恢复默认，不读取型号或推断能力 |
| model.selection | features/models/ModelPicker + ModelChoiceOptions | 纯数据/事件组件；渠道/模型分别查询、表格原序/升降序、跨范围临时选择、失效引用、逐项推理/Fast及确认；业务adapter持有目录/保存边界 |
| node.selection | features/models/ChannelEditor + useNodeMeasurements | 共享渠道认证及手动当前节点；单/批模型清单检测和取消，输入变化作废结果，不自动切换，不调用生成；计时只描述清单请求 |
| overlay.dialog | ui/UiDialog | 模版版本管理/匹配预览及gallery；原生dialog焦点约束、Escape/遮罩、busy防止误关，表单数据由feature拥有 |
| navigation.tabs / navigation.anchor | layout/StickyTabs | 模型/设置/扩展/诊断主Tab及gallery；容器sticky、ResizeObserver量高，详情/锚点消费--page-tabs-height；不接概览/面板 |

尺寸、材质、语义层级沿styles与既有专题；新增组件不读store/IPC，ModelRuleFields保留业务feature职责。实现/单元检查不等于完整实屏指标通过，后续模型选择器及消费者迁移继续登记。

后续（阶段 C 续）：`UiCardBody/UiCardFooter`（现有实现无 `.card-body`/`.card-footer`，需先形成视觉/结构决定，见 IMP-04 §7.5）、`patterns/InlineNotice`（当前无 `.notice` 消费者，待出现真实使用再提取）、组件规则检查（限制组件内新增颜色/字号/字重/层级）。


## 0.2.0公共入口及源映射

registry.ts按专题43个稳定ID记录规格来源、当前实现/配方、实际消费者和DEV样例锚点；由gallery消费，正式入口不导入。原型注释仅适用于文档，原生平台另行检查；所有条目的完整矩阵仍待验收，不用“源文件存在”代表已适配完成。

设置14处开关共用ui/UiSwitch.vue，传受控值与必填label，上报update:modelValue，不在基础层读store/IPC；原受控保存者继续决定何时更新。视觉轨道38×22、点击高度28，令牌承载尺寸/拇指/阴影。普通按钮沿UiButton的sm28/md34/lg40，页面选择size，不覆盖最终高度/内距/字号；进行中保持原slot宽度，使用覆盖式装饰spinner与aria-busy，不改变可访问名称。

本轮117处原btn消费已迁移至UiButton；路径行、分页和最近事件次操作明确sm。原生caption/托盘、图标按钮、文本复制按钮及轻量HTML表单保持各自用途和登记，尚需逐项规格/尺寸与原生证据，不能因此视为普通btn的样式例外已验。


## 0.2.0反馈纯视图与基础样例

UiIconButton统一普通32与通知24两档，名称必填、原生禁用/忙碌；顶栏刷新、通知删除和反馈收起沿同一部件。TaskProgressCard/FeedbackToast为纯视图，应用TaskProgressHost/ToastHost仍是原状态与事件的唯一承接者，gallery只传固定样例，不创建第二个生产通知入口。NotificationItem仍由通知feature负责条目语义。

FoundationExamples直接引用品牌/平台资产、受限Markdown、原生表单配方、真实UiTreeTable/UiSelectMenu及反馈视图，基础元素不为组件数量机械封装。字段sm/md/lg沿28/34/40令牌，平台caption/托盘的原生行为仍单独验收。动作菜单键盘导航识别menuitem并跳过禁用，异步开闭在卸载后不写失效DOM。
