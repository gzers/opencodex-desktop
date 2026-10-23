# UI样式问题记录 · 0.2.1

本文档记录0.2.1版本开发过程中发现的UI样式与原型不一致的问题，以及修复方案。

---

## UI-023：概览卡片文字大小与悬停背景边距

**优先级**：中  
**状态**：待修复  
**发现时间**：2026-10-22

### 问题描述
1. 版本升级卡片中"桌面管理器"和"OpenCodex 面板"两行文字大小与其他卡片不一致
2. 鼠标悬停时选中背景无内边距，与全局样式不符

### 用户反馈
> "概览卡片内，版本升级的两行和另外的卡片文字大小不一致，要么都增大。而且鼠标浮于上方的时候选中背景无边距，是不是也没有按照全局样式做。"

### 修复方案
1. 统一概览页各卡片内标题文字大小
2. 为悬停状态添加内边距，参考原型规范

### 相关文件
- `apps/desktop/ui/src/components/overview/VersionCard.vue`
- `apps/desktop/ui/src/styles/components/_cards.scss`

---

## UI-024：Tab区域背景到顶渐显效果

**优先级**：中  
**状态**：原型已修复，代码需实现  
**发现时间**：2026-10-22

### 问题描述
拓展、同步、设置、模型等页面的Tab区域在滚动到顶部固定时，背景色是完全不透明的纯色，缺少玻璃效果。原型已修复为滚动到顶部时才淡入背景。

### 用户反馈
> "还有就是原型也有的问题，tab存在带颜色背景，怎么办但是到顶部还得固定，玻璃背景不能丢。能否到顶了才出现背景，淡入效果。"

### 受影响页面
- 模型配置页 (`#models`)
- 拓展管理页 (`#extensions`)
- 配置同步页 (`#sync`)
- 设置页 (`#settings`)

### 修复方案
使用 Intersection Observer API 检测 Tab 区域是否到达顶部，当到达时为背景添加 200-300ms 的淡入过渡效果。

**实现步骤**：
1. 在 Tab 容器组件中添加滚动监听
2. 通过 CSS 类切换控制背景透明度
3. 添加 `transition: background-color 250ms ease-in-out`

### 相关文件
- `apps/desktop/ui/src/components/models/ModelsTabBar.vue`
- `apps/desktop/ui/src/components/extensions/ExtensionsTabBar.vue`
- `apps/desktop/ui/src/components/sync/SyncTabBar.vue`
- `apps/desktop/ui/src/components/settings/SettingsTabBar.vue`
- `apps/desktop/ui/src/styles/components/_tabs.scss`

---

## UI-025：更新检查时缺少加载动画

**优先级**：低  
**状态**：待修复  
**发现时间**：2026-10-22

### 问题描述
检查更新时，更新中心弹窗中管理器和面板卡片的更新按钮缺少加载动画，与底部"重新检查更新"按钮不一致。

### 用户反馈
> "我建议检查更新的时候，管理器和面板的更新按钮页面成动画。跟底部一样。"

### 修复方案
为两个卡片的更新按钮添加加载状态动画，复用底部按钮的加载样式。

### 范围
更新中心弹窗（`UpdateCenter.vue`）：
- 桌面管理器更新按钮
- OpenCodex 面板更新按钮

---

## WP-07：模型选择器过滤规则

**优先级**：高  
**状态**：已确认方案  
**发现时间**：2026-10-22

### 问题描述
模型选择器（Agent配置、渠道详情侧边栏等）显示了隐藏渠道的模型和`pickerAvailable: false`的模型。

### 用户反馈
> "模型选择器只能选择非隐藏的模型，这个清楚吧？"  
> "刚刚的模型选择器注意，渠道隐藏的也不要展示了。"

### 修复方案
实现三层过滤规则，按以下顺序筛选：

1. **模型层过滤**：`model.pickerAvailable !== false`
2. **渠道启用过滤**：`provider.disabled !== true`
3. **渠道隐藏过滤**：`provider.hidden !== true`

**预期过滤结果**：
- 原始模型总数：~243
- 第一步过滤后：~122
- 第二步过滤后：~89
- 第三步过滤后：~64（最终可选）

**代码示例**：
\`\`\`typescript
const availableModels = allModels.filter(model => {
  // 1. 模型层：排除明确标记为不可选的模型
  if (model.pickerAvailable === false) return false;
  
  // 2. 渠道启用状态：排除已停用渠道的模型
  const provider = providers.find(p => p.id === model.providerId);
  if (!provider || provider.disabled === true) return false;
  
  // 3. 渠道隐藏状态：排除隐藏渠道的模型
  if (provider.hidden === true) return false;
  
  return true;
});
\`\`\`

### 相关文件
- `apps/desktop/ui/src/components/settings/AgentExecutionField.vue`
- `apps/desktop/ui/src/features/models/ModelPicker.vue`
- `apps/desktop/ui/src/stores/models.ts`

### 补充说明
需注意与Agent合规检查的交互：部分国产模型虽然可选，但在特定Agent配置下可能被合规规则拦截。选择器应先完成基础过滤，合规校验留给后续保存阶段。

---

## WP-08：模型选择器加载性能

**优先级**：高  
**状态**：已确认方案A  
**发现时间**：2026-10-22

### 问题描述
每次打开模型选择器都需要长时间加载，用户体验差。

### 用户反馈
> "还有为什么模型选择器，每次都要加载很久？为什么？你的源头能否从模型渠道这一层就可以读取呢？"

### 根因分析
`AgentExecutionField.vue` 组件在每次打开选择器时重复调用 `modelApi.executionDirectory()`，该接口需要：
1. 读取所有渠道配置
2. 合并各渠道模型列表
3. 执行合规校验
4. 计算可用性状态

### 修复方案（方案A · 已确认）
**复用模型配置页的 Store 状态**

模型配置页（`#models`）已维护完整的渠道与模型数据，选择器直接从该 Store 读取，避免重复请求。

**实施步骤**：
1. 分析 `stores/models.ts` 的数据结构和更新机制
2. 重构 `AgentExecutionField.vue` 和 `ModelPicker.vue`，替换 API 调用为 Store 读取
3. 确保 Store 在后台同步更新（渠道启用/停用、模型检测等）
4. 保留首次进入应用时的初始化加载

**优势**：
- 选择器打开即显示，无需等待
- 数据源统一，减少不一致风险
- 模型配置页的操作实时反映到选择器

### 相关文件
- `apps/desktop/ui/src/features/models/AgentExecutionField.vue`（需重构）
- `apps/desktop/ui/src/features/models/ModelPicker.vue`（需重构）
- `apps/desktop/ui/src/stores/models.ts`（复用）
- `apps/desktop/ui/src/pages/ModelsPage.vue`（参考）

---

## 其他已记录问题

### UI-020：同步页文件同步卡片间距缺失
**状态**：待修复  
**发现**：2026-10-22  
同步页卡片之间缺少间距，多个卡片紧贴在一起。

### UI-021：渠道与模版表格列宽分配不合理
**状态**：待修复  
**发现**：2026-10-22  
- 模型渠道表：复选列过宽，其他列挤压
- 模型模版表：复选列未独立，模版列被挤压

### UI-022：侧边栏按钮图标与文字不对齐
**状态**：待修复  
**发现**：2026-10-22  
模版和渠道详情侧边栏的按钮中，图标与文字垂直对齐不正确。

---

## 补充说明

### 全局样式组件库
用户提出的对齐问题暴露了一个系统性问题：**代码层是否存在统一的全局样式定义**，类似原型中的组件样式规范。

**建议**：
1. 建立 Design Tokens 文件（字号、行高、间距、颜色等）
2. 封装常用组件（按钮、卡片、表格等）的基础样式
3. 文档化组件使用规范，确保新功能开发时直接复用

### 原型与实现的持续同步
0.2.1 的问题集中体现为"实现与原型偏离"。建议在开发流程中：
- 每个功能模块开发前，对照原型完成 UI Checklist
- PR Review 增加原型对比环节
- 定期进行全量 UI 走查

---

## UI-026：更新中心检查结果与单对象交互均偏离原型

**优先级**：高
**状态**：原型已修复，代码需实现
**发现时间**：2026-10-23
**对应登记**：0.2.7 问题记录 WP-63；契约见 UI 规范 §39

### 问题描述

1. 检查更新结果弹窗需要滚动才看得全（1180×760 溢出 271px）。
2. 软件把“检查结果”和“单个对象详情确认”合并成一个更新中心，结果页直接给出安装按钮；面板卡片只在文案里要求备份，没有可勾选项。
3. 两个对象各带 220px 内部滚动区，外层正文又滚动，出现滚动套滚动。

### 用户反馈

> “这个地方能不能调整一下，尽量一个弹窗不滚动展示完。但是布局还是要美观。”

> “这个地方更新界面实际上也是跟原型有差距，说明实际软件开发的界面走偏了，不论单个更新还是全检查，的交互，ui都偏离。记录分析”

### 修复方案

1. 结果页每个对象压成一行：名称 + 状态标签、版本、来源／一句说明摘要 + “更新内容”折叠／查看详情；说明展开后自身滚动，外层不滚。
2. 拆回两级：结果页只读，安装与备份回到单对象确认页。
3. 面板与管理器都提供“更新前备份”勾选，备份失败即阻止更新。
4. 宽度回到 §37 的 780px；结果页标题固定“检查更新结果”；footer 只保留“关闭”。

### 原型实测

改造前正文 757px（1180×760 可用 486px），改造后 290px；1180×760 / 1440×900 / 900×600 三档均无滚动。

### 相关文件

- `apps/desktop/ui/src/features/updates/UpdateCenter.vue`（L189 标题、L193 单一弹窗切换、L194/L219 双卡片、L207/L231 结果页安装按钮、L233 footer、L248 宽度、L257 外层滚动、L267 说明内滚）
- `apps/desktop/ui/src/features/updates/ReleaseNotes.vue`
- 原型：`原型/maintenance-ui.js` `showUpdates()` / `notesSummary()`、`原型/maintenance-ui.css` `.mnt-update-row`

---

**文档维护**：本文档随 0.2.1 开发持续更新，问题修复后状态改为"已修复"并注明修复日期。  
**最后更新**：2026-10-23

