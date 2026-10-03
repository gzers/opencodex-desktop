---
id: IMP-OPENCODEX-DESKTOP-04
object_kind: implementation.change
state: in_progress
title: 工程结构与组件体系调整草案
summary: 在 IMP-03 基础上规定模块与组件结构、样式、错误处理、性能修复、标准实施计划及全操作入口和端到端验收；方案方向已获认可，正按 §19 计划执行（A–F 阶段，F 的 ④ 层制品内交互待补）。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-80", "TASK-OPENCODEX-DESKTOP-82", "TASK-OPENCODEX-DESKTOP-83", "TASK-OPENCODEX-DESKTOP-84", "TASK-OPENCODEX-DESKTOP-85", "TASK-OPENCODEX-DESKTOP-86", "TASK-OPENCODEX-DESKTOP-87", "TASK-OPENCODEX-DESKTOP-88", "TASK-OPENCODEX-DESKTOP-89", "TASK-OPENCODEX-DESKTOP-90", "TASK-OPENCODEX-DESKTOP-91", "TASK-OPENCODEX-DESKTOP-92", "TASK-OPENCODEX-DESKTOP-93", "TASK-OPENCODEX-DESKTOP-94", "TASK-OPENCODEX-DESKTOP-95", "TASK-OPENCODEX-DESKTOP-96", "TASK-OPENCODEX-DESKTOP-97", "TASK-OPENCODEX-DESKTOP-98", "TASK-OPENCODEX-DESKTOP-99", "TASK-OPENCODEX-DESKTOP-100", "TASK-OPENCODEX-DESKTOP-101", "TASK-OPENCODEX-DESKTOP-102", "TASK-OPENCODEX-DESKTOP-103", "TASK-OPENCODEX-DESKTOP-104"]
completion_summary: null
source_refs: ["IMP-OPENCODEX-DESKTOP-03", "IMP-OPENCODEX-DESKTOP-01"]
---

# 工程结构与组件体系调整草案

> 2026-10-01 后续还原专项：用户已确认当前原型，双形态概览、紧凑运行详情及三档材质／动效组件化按 [IMP-05](IMP-OPENCODEX-DESKTOP-05.md) 计划实施。本文 §19.32 的旧环境摘要与固定舞台完成记录是历史基线，不表示 DMD Revision 14 的 `AC-16 / AC-17` 已达标；本文的工程、流程和性能纪律继续被新计划继承。

- 版本：v0.9，2026-09-28；新增 §19.19 ~ §19.29（安装版本管理与统一卸载：原型定稿、需求 Revision 11、后端统一卸载内核、前端弹窗重做、浏览器审计夹具、制品内原生复核，以及卸载后状态陈旧与官方卸载顺序的缺陷修复）。v0.8，2026-09-28；新增 §19.15 / §19.16 / §19.17 / §19.18（诊断中心交互调整、④ 层兜底条真机复核与新缺陷修复、§17.6 验收收口）。v0.6，2026-09-27；新增第 19 节可执行实施计划与 §19.14 F 阶段执行结果（入口清单、分层测试、原型复核、打包与冒烟、门禁）。
- 状态：方案方向已获用户认可；第 19 节冻结基线与阶段命令，随后按 B→F 顺序执行。
- 用户偏好：优雅、工程化、易维护；公共组件覆盖控件、卡片与内部小布局，页面通过组件组合表达。
- 本次范围：完善方案与实施验收要求，并提供后续执行提示词；不移动源码目录、不执行重构。已确认规则在实施中保持，待统一项按本文显式决定，不静默改变契约。
- 前置计划：[IMP-03 目录结构规范化](IMP-OPENCODEX-DESKTOP-03.md)。
- 现行依据：[系统架构](../02-项目核心/系统架构.md)、[UI 规范](../02-项目核心/UI规范.md)、[契约字段](../02-项目核心/契约字段.md)。
- 本文描述目标，不代表现状；实现完成后再更新核心文档中的现状事实。

## 1. 设计结论

采用一个桌面产品下的模块化组织：

1. 保留 `apps/desktop/{ui,tauri}` 作为前后端根目录。
2. 前端按功能聚合状态、接口、展示转换和业务组件；页面主要负责组合。
3. 建立完整的应用内组件体系，包括基础控件、视觉容器、布局、组合结构和业务内部组件。
4. 公共界面组件使用明确的 `components/` 名称；本版不设包含所有公共代码的顶层 `shared/`。
5. 样式跟随组件，Token 与基础样式集中维护；保留原生 CSS 及现有视觉规范。
6. 后端保留现有业务模块，逐步缩薄命令入口，收敛 GUI 与 CLI 的共同用例。
7. 用依赖检查、组件目录说明、组件预览和有意义的行为测试维持边界。

“组件化程度”不以文件数量评价，而看页面能否清楚表达结构、相同设计能否统一修改、业务状态是否有明确归属。

## 2. shared 是什么，命名如何选择

`shared` 是“共享的”。它描述代码的使用范围，并不专指公共组件；一种常见组织方式是 `shared/ui`、`shared/api`、`shared/lib`。FSD 的 Shared 层也包括 UI、库与外部连接能力，且允许布局和 UI 交互逻辑进入共享 UI，见参考 S3。

本项目需要区分：

| 名称 | 说明 | 本版取舍 |
|---|---|---|
| `shared` | 跨模块共享层，可包含 UI、工具、配置、适配 | 合法，但本版不作为总入口，避免与 `components` 形成两套归属 |
| `common` | 通用内容，仍需要解释具体职责 | 不增设顶层 `common`；不与 `shared`、`utils` 同时堆放杂项 |
| `components` | Vue 组件入口 | 用于应用内公共界面组件，职责直观 |
| `ui` | 界面控件及视觉基础部件 | 放在 `components/ui` |
| `layout` | 排列、间距、宽度、滚动、区域划分 | 放在 `components/layout` |
| `patterns` | 由多个基础部件构成、语义稳定的通用结构 | 放在 `components/patterns` |
| `features/*/components` | 有具体业务语义的组件，包括内部小组件 | 功能就近维护，可向多个页面提供同一个业务组件 |
| `packages/ui` | 独立包级 UI 复用 | 出现第二个实际消费者后再评估 |
| `public` | 当前 Vite 工程中的静态资源位置 | 不用于命名组件目录 |

这里的“公共”表示应用内部多个功能可以依赖，不意味着发布为开源组件包。

外部参考不是统一标准：

- Folo 同时有应用内 `components/common`、`components/ui`，以及 `packages/internal/components`，见 S1。其边界取决于产品与消费者范围。
- shadcn-vue 的示例使用 `@/components/ui/card`，卡片提供独立组成部件，见 S2。
- FSD 使用 `shared/ui`，它属于一套明确的分层方法，见 S3。

本项目采用“功能聚合 + 明确的公共目录”，不宣称完整采用 FSD。`features` 在本文表示应用能力模块，不能将其规则与完整 FSD 的 Features/Entities/Widgets 逐项等同。

## 3. 当前结构问题与保留项

以下为 2026-09-27 本地源码静态检查，未据此宣称测试或打包已通过。

| 位置（当前路径） | 观察 | 调整方向 |
|---|---|---|
| `src-ui/src/stores/app.ts`，1,479 行 | 运行、扩展、通知、同步、迁移和界面反馈集中 | 按功能拆状态，反馈独立 |
| `src-ui/src/composables/useAppController.ts`，416 行 | 启动、日志、诊断、托盘、恢复混合 | 启动与跨功能协调归 app；功能行为归各自功能 |
| `src-ui/src/routes/SettingsRoute.vue`，1,414 行 | 多个设置区及安装、卸载向导集中 | 页面组合功能区；功能区继续拆内部组件 |
| `src-ui/src/routes/ExtensionsRoute.vue`，731 行 | 列表、搜索、动作、详情交互集中 | Skills/MCP 列表、目标矩阵、详情与动作分别归位 |
| `src-ui/src/styles/base.css`，913 行 | 控件、壳层和业务专属样式混合 | 样式随组件迁入，基础样式仅保留基础职责 |
| `src-ui/src/components/AppModal.vue` | 绑定总 store，正文通过 HTML 字符串传入 | 对话框基础组件 + 具体业务对话框 + 模板与插槽 |
| `src-ui/src/data/mock.ts` | 包含生产使用的展示规则与常量 | 按功能拆归，真实 fixture 单独保留 |
| `src-tauri/src/commands/extensions.rs` | 命令适配与脱敏、配置编排共存 | 规则与用例移入模块或 application |
| `src-tauri/src/lib.rs` | 初始化、服务装配、后台运行与桌面生命周期集中 | 提取 bootstrap，保留清晰入口 |
| `src-tauri/tests/layer_boundaries.rs` | 检查公开符号存在，未限制依赖方向 | 补真正的边界检查 |

保留既有优势：Token、亮暗主题、缩放规则、前端严格类型检查、后端业务模块与来源抽象、业务测试以及真实打包门槛。

文件长度只是定位入口。例如 `runtime/install.rs` 的 2,694 行包含从第 1,709 行开始的测试，不能把总行数等同于生产复杂度。

## 4. 组件体系：公共组件与内部小组件都要建立

### 4.1 五种职责

| 类别 | 示例 | 归属与约束 |
|---|---|---|
| 基础控件与视觉部件 | Button、Switch、Card、Dialog、Badge | `components/ui`；统一交互、视觉、可访问性，不读业务 store |
| 通用布局 | PageLayout、Stack、Inline、ResponsiveGrid | `components/layout`；负责排列、尺寸、滚动与响应式 |
| 通用组合结构 | SettingRow、PathField、KeyValueList、ProgressPanel | `components/patterns`；组合控件与布局，数据由调用方传入 |
| 业务组件 | RuntimeSourceCard、SkillList、SyncSettings | `features/<name>/components`；承载业务语义，可使用本功能状态 |
| 应用壳与装配组件 | AppSidebar、TitleBar、FeedbackHost | `app`；承担应用导航、宿主与生命周期 |

“只用一次”不是禁止拆组件的条件。安装向导的步骤、运行来源卡片的头部信息、扩展详情的客户端区域，如果有独立语义或交互，就可以是功能内部组件。

“跨多个页面使用”也不是必须移进公共目录的条件。`RuntimeStatusCard` 即使在概览与设置中复用，仍属于 runtime 功能。

### 4.2 目标组件目录

以下是依据当前页面提出的候选清单。优先提取现有使用形态，逐项建立真实消费者；命名与 API 在试点时细化。

```text
components/
├── ui/
│   ├── button/        UiButton、UiIconButton、UiButtonGroup
│   ├── input/         UiInput、UiTextarea、UiNumberInput
│   ├── selection/     UiCheckbox、UiRadioGroup、UiSwitch
│   ├── select/        UiSelect
│   ├── slider/        UiSlider
│   ├── tabs/          UiTabs、UiTabList、UiTab、UiTabPanel
│   ├── menu/          UiDropdownMenu、UiMenuItem
│   ├── tooltip/       UiTooltip
│   ├── popover/       UiPopover
│   ├── dialog/        UiDialog、UiDialogHeader、UiDialogBody、UiDialogFooter
│   ├── card/          UiCard、UiCardHeader、UiCardTitle、
│   │                  UiCardDescription、UiCardBody、UiCardFooter
│   ├── badge/         UiBadge、UiStatusDot
│   ├── progress/      UiSpinner、UiProgressBar、UiSkeleton
│   ├── table/         UiTable（语义表格及插槽，不内置业务请求）
│   ├── pagination/    UiPagination
│   ├── separator/     UiSeparator
│   └── icon/          UiIcon（已有图标的统一尺寸与语义）
├── layout/
│   ├── PageLayout.vue
│   ├── PageHeader.vue
│   ├── PageSection.vue
│   ├── Stack.vue
│   ├── Inline.vue
│   ├── ResponsiveGrid.vue
│   ├── SplitLayout.vue
│   ├── ScrollArea.vue
│   └── ActionBar.vue
└── patterns/
    ├── form/          FormField、FormSection、FormActions
    ├── settings/      SettingGroup、SettingRow
    ├── details/       KeyValueList、KeyValueItem、PathField、CodeBlock、MarkdownContent
    ├── collection/    SearchField、FilterBar、ListToolbar
    ├── feedback/      InlineNotice、EmptyState、ErrorState、
    │                  LoadingState、ToastItem、ConfirmDialog
    └── progress/      StepIndicator、ProgressPanel
```

目录只为组件族而建；单个简单组件无需强制建立文件夹。`Ui` 前缀暂作为基础组件统一命名，避免 Button/BaseButton/AppButton 同时出现。布局与组合结构使用语义名称，`App` 留给应用壳。

### 4.3 卡片及内部布局应当怎样抽

卡片可分成三个尺度：

1. **外观基础**：`UiCard` 负责表面、边框、圆角；Header/Body/Footer 负责区域及默认间距。
2. **通用内容结构**：`SettingGroup`、`KeyValueList`、`ProgressPanel` 定义经常出现的内部组织。
3. **业务内容**：`RuntimeSourceCard`、`SyncStatusCard` 负责数据语义、动作与状态。

这样可以统一卡片外观，也可以单独调整业务布局，不必让每张卡片重新拼标题、间距、边框和操作区域。

不把所有卡片折叠成包含 `isRuntime/isSync/isDanger/showInstall` 等大量开关的万能组件。不同内容通过具名插槽和部件组合；有限的视觉变化用 `variant`、`density` 等枚举。

`Stack`、`Inline` 统一常用间距、对齐与换行；`ResponsiveGrid` 统一卡片列布局；`SettingRow` 统一“说明 + 控件 + 行内反馈”。独特的业务网格仍由局部 CSS 表达。

布局组件应保留正确 HTML 语义，可提供受限的 `as`，避免每个组件无条件新增一个 div。卡片仅在语义合适时用 article；标题级别可选且由页面层级决定；小布局不制造重复 landmark。

### 4.4 组件使用示意

以下为目标 API 示意，不是已实现接口：

```vue
<UiCard>
  <UiCardHeader>
    <UiCardTitle as="h2">运行来源</UiCardTitle>
    <UiCardDescription>选择当前使用的 OpenCodex。</UiCardDescription>
    <template #actions>
      <UiBadge :tone="statusTone">{{ statusLabel }}</UiBadge>
    </template>
  </UiCardHeader>

  <UiCardBody>
    <Stack gap="4">
      <KeyValueList>
        <KeyValueItem label="版本" :value="versionLabel" />
        <KeyValueItem label="来源" :value="sourceLabel" />
      </KeyValueList>
      <PathField :value="executablePath" readonly @open="openExecutableDir" />
      <InlineNotice v-if="errorMessage" tone="danger">
        {{ errorMessage }}
      </InlineNotice>
    </Stack>
  </UiCardBody>

  <UiCardFooter>
    <ActionBar>
      <UiButton variant="secondary" @click="chooseSource">选择来源</UiButton>
      <UiButton :loading="installing" @click="openInstall">安装</UiButton>
    </ActionBar>
  </UiCardFooter>
</UiCard>
```

`PathField` 只呈现路径并发出 open/copy 事件，不知道数据根，也不直接打开原生目录。业务组件决定该路径是否可打开、执行何种命令。

### 4.5 抽取原则

符合以下任一条件即可考虑组件化：

- 统一视觉或交互规范，例如输入框、卡片、状态标签。
- 有稳定结构，例如设置行、表单项、键值列表、操作栏。
- 有独立行为，例如详情展开、表单验证、焦点管理、向导步骤。
- 是可命名的业务区域，即使当前只出现一次。

不为提高数量包装没有语义、没有规则的每个 span/div。允许少量原生元素保留在组件内部；目标是让页面与功能区清晰，而非隐藏 HTML。

公共组件不接受完整业务 DTO，不使用业务名判断渲染，不订阅全局业务状态。向上通过事件报告动作，向下通过 props/slots 获取内容。

## 5. 完整工程目标结构

```text
opencodex-desktop/
├── apps/
│   └── desktop/
│       ├── README.md
│       ├── ui/
│       │   ├── package.json / package-lock.json
│       │   ├── index.html / vite.config.ts / tsconfig.json
│       │   ├── public/
│       │   ├── src/
│       │   │   ├── main.ts
│       │   │   ├── app/
│       │   │   │   ├── App.vue
│       │   │   │   ├── bootstrap.ts
│       │   │   │   ├── router/        # 导航实现、hash 适配及页面注册
│       │   │   │   ├── appearance/    # 主题、缩放与首帧应用
│       │   │   │   ├── shell/
│       │   │   │   ├── feedback/      # Toast、确认请求的队列与服务
│       │   │   │   ├── hosts/         # 在应用根装配通知、任务、弹窗
│       │   │   │   └── workflows/     # 启动、托盘动作、跨功能协调
│       │   │   ├── pages/
│       │   │   │   ├── overview/
│       │   │   │   ├── panel/
│       │   │   │   ├── extensions/
│       │   │   │   ├── logs/
│       │   │   │   ├── settings/
│       │   │   │   └── tray/
│       │   │   ├── features/
│       │   │   │   ├── runtime/
│       │   │   │   ├── environment/
│       │   │   │   ├── extensions/
│       │   │   │   ├── notifications/
│       │   │   │   ├── preferences/
│       │   │   │   ├── sync/
│       │   │   │   ├── data-root/
│       │   │   │   ├── backup/
│       │   │   │   ├── migration/
│       │   │   │   ├── diagnostics/
│       │   │   │   ├── updates/
│       │   │   │   ├── codex-shim/
│       │   │   │   ├── panel/
│       │   │   │   └── about/
│       │   │   ├── components/        # 见第 4 节
│       │   │   ├── composables/       # 通用 UI 行为、navigation/feedback 注入接口
│       │   │   ├── platform/          # Tauri 调用、事件、原生能力适配
│       │   │   ├── contracts/         # 跨 IPC 类型；不混入本地路由类型
│       │   │   ├── navigation/        # 路由名、参数、纯解析；不导入页面组件
│       │   │   ├── lib/               # 按日期、文本等明确主题组织纯函数
│       │   │   ├── styles/            # Token、reset、base、少量 utilities
│       │   │   ├── assets/
│       │   │   └── dev/
│       │   │       ├── fixtures/
│       │   │       ├── audit-harness.ts
│       │   │       └── component-gallery/
│       │   └── tests/
│       │       ├── integration/
│       │       ├── contracts/
│       │       ├── architecture/
│       │       └── setup.ts
│       └── tauri/
│           ├── Cargo.toml / Cargo.lock / build.rs
│           ├── tauri.conf.json / capabilities/ / icons/
│           ├── src/
│           │   ├── main.rs / lib.rs
│           │   ├── bin/ocxd.rs
│           │   ├── bootstrap/          # 装配、后台任务、桌面生命周期
│           │   ├── commands/           # Tauri 命令适配
│           │   ├── ipc/                # 本机协议与分派
│           │   ├── application/        # 跨模块用例
│           │   ├── modules/            # 保留现有业务模块
│           │   ├── infrastructure/     # 文件、进程、网络、凭据、桌面适配
│           │   ├── contracts/          # 明确跨边界的 DTO
│           │   ├── state.rs
│           │   └── errors/
│           └── tests/
├── scripts/
├── docs/                               # 保留既有中文分区
├── .adg/
├── .github/workflows/
├── package.json / package-lock.json     # 根命令入口与工具依赖
├── .editorconfig
├── eslint.config.mjs / prettier.config.mjs / stylelint.config.mjs
├── CONTRIBUTING.md
├── AGENTS.md
└── README.md
```

组件与纯函数的单元测试就近维护；跨功能集成、契约与架构检查放 `tests/`。不同时维护两套同职责测试。

当前不引入空 `packages/`、多 crate、构建编排框架或包管理器迁移。根 package 暂作为命令与工具入口；锁文件及安装步骤必须明确各自归属。

### 5.1 一个功能模块的内部结构

```text
features/runtime/
├── index.ts
├── api.ts
├── store.ts
├── model.ts
├── presentation.ts
├── composables/
│   ├── useRuntimeInstall.ts
│   └── useRuntimeUninstall.ts
├── components/
│   ├── RuntimeSettings.vue
│   ├── RuntimeStatusCard.vue
│   ├── RuntimeSourceCard.vue
│   ├── RuntimeInstallDialog.vue
│   ├── RuntimeUninstallDialog.vue
│   └── install/
│       ├── InstallSourceStep.vue
│       ├── InstallOptionsStep.vue
│       ├── InstallReviewStep.vue
│       └── InstallProgressStep.vue
├── presentation.test.ts
└── store.test.ts
```

此处步骤名是拆分示意，实施时按现有向导真实步骤映射，不新增用户流程。简单功能不必建立所有文件。

### 5.2 设置页面的组合结构

`SettingsPage` 处理导航和当前设置分区，组合 `GeneralSettings`、`RuntimeSettings`、`ExtensionSettings`、`SyncSettings`、`DataRootSettings`、`BackupSettings`、`UpdateSettings` 等功能组件。

功能组件内部再使用 `SettingGroup → SettingRow → UiSwitch/UiSelect`，并在需要时组合 `FormField`、`PathField`、`InlineNotice`。设置只是功能入口，不成为新的总业务模块。

## 6. 依赖与状态规则

依赖按下表限制，公共目录不意味着可以相互任意引用：

| 来源 | 允许依赖 | 禁止依赖 |
|---|---|---|
| `app` | pages、features、公共能力 | — |
| `pages` | features、components、navigation、通用 composables/lib | app 内部实现、其他页面内部文件 |
| `features/<name>` | 本功能内部、components、platform、contracts、navigation、通用 composables/lib | app/pages、其他 feature 内部 |
| `components/patterns` | ui、layout、通用 UI composables/lib | features、platform、业务 contracts/store |
| `components/layout` | 少量基础 UI、通用 UI composables/lib | patterns、features、platform |
| `components/ui` | 通用 UI composables/lib、Token | layout/patterns、业务与平台实现 |
| `platform` | contracts、必要工具与 Tauri SDK | Vue 页面、业务 store、components |
| `contracts/lib` | 纯类型及基础工具 | Vue UI、app、业务 store |
| `navigation` | 自有路由类型与纯工具 | app/pages/features、Vue 页面组件 |
| `composables` | Vue 基础能力、lib；服务注入接口可引用 navigation/基础契约类型 | app 实现、feature store、页面组件 |

`contracts` 与 `lib` 是两个独立目录，上表合并行仅表示它们都处于基础层。公开类型、注入接口不得反向引用 app 中的实现类型。

跨功能流程由 page 或 app/workflows 协调，通过显式参数、回调或注入接口连接。功能公开入口不等于允许所有 feature 互相调用。

反馈服务由 app 实现并提供；`useFeedback` 的接口与注入 key 可放通用 composables。功能仅依赖接口，不能反向导入 app 队列；需要业务决策的通知继续属于 notifications 功能。

状态分工：

- 搜索词、展开状态、未提交表单保留在组件或功能 composable。
- 多处消费的服务快照、列表和持久化设置归对应 feature store。
- 长任务进度在需要跨页面持续显示时归 feature store。
- 启动订阅、托盘轮询只由应用入口建立一次，并明确销毁。
- Toast、弹窗队列与后台持久通知分别管理，不混为同一个总 store。
- 后端返回结果是持久化事实依据，避免 UI 为方便展示制造另一套业务事实。

## 7. 样式与组件 API 规范

### 7.1 样式归属

- `styles/tokens.css`：颜色、字号、间距、圆角、阴影、层级、动效及主题。
- `reset.css/base.css`：根节点、重置、基础排版。
- 控件、卡片、布局、功能样式放对应 SFC；较长时可拆邻近 CSS。
- 组件默认 scoped；父组件通过 props、slots、文档化 CSS 变量调整，不依赖子组件内部选择器。
- 功能专属 `.install-*`、`.extension-*` 等不再进入全局 base。
- 品牌图形、动态进度、原生窗口尺寸有明确例外，不机械禁止所有数值和内联样式。

Token 优先表达语义；现有变量可逐步兼容迁移。`gap="4"` 等组件参数映射 Token，不接受任意 CSS 片段作为默认接口。

### 7.2 基础行为

- 统一 normal/hover/focus-visible/active/disabled/loading/error 等适用状态。
- Input 使用一致的 v-model、label、description、error 接口；不能仅依赖 placeholder 表达标签。
- IconButton 必须有可访问名称。
- Dialog 管理焦点、Escape、焦点恢复；危险确认由业务组件明确表达。
- 菜单、选择器、Tabs 的键盘行为需验证；采用现成无样式行为原语可以另行评估，本草稿不改变“不引入 UI 框架”的现有约束。
- Progress 区分可测百分比与不确定进度；不制造虚假进度。
- 组件可转发必要原生属性；普通按钮默认 type 为 button，提交场景显式指定。
- 复杂正文用模板/slot；不把 HTML 字符串作为普通弹窗内容接口。

### 7.3 组件预览与说明

建立开发态 component-gallery，集中展示真实公共组件的变体、长文本、错误、禁用、加载、键盘交互与亮暗主题。预览直接引用生产组件，不能另写一套演示 DOM。

初期可沿用现有 Vite 开发环境，不预设必须使用 Storybook/Histoire。开发入口与 fixtures 不进入正式构建；预览数据不触发真实 IPC。

每个组件族简要记录职责、props/slots/events、样式扩展点、可访问性约束及已有消费者。简单展示组件不要求单独制造低价值测试；复杂行为写交互测试，视觉与布局在组合预览中验证。

### 7.4 数值规范的状态与唯一来源

样式需要固定到“值、用途、组件消费方式、例外”，不能只规定使用 Token。

当前已有基础数值，但消费不完全统一。下列标记区分事实与提案：

- **沿用**：来自当前 `tokens.css` 或《UI 规范》的值。
- **提取**：当前 CSS 已使用的值，建议赋予组件 Token；不代表已经完成代码替换。
- **建议**：本草案新增的映射或接口，待组件试点验证后定稿。
- **待统一**：已有不同口径，记录差异，不能将重构顺带变成视觉调整。

数值以 100% 界面缩放下的 CSS 逻辑像素计；缩放继续使用现行统一因子，不为每个组件重复计算一遍字号和间距。固定的是设计规则，不是所有容器的绝对宽高；响应式布局与文字换行仍需正常工作。

维护职责：本文保存讨论方案；定稿后的视觉规则回写《UI 规范》，运行时值在 Token 文件中定义，组件通过引用消费。本文的现有值表是本次核对快照，不新增第二套长期维护的色板或配置 JSON。

### 7.5 间距：基础刻度与组件间距分别定义

基础间距沿用当前 Token：

| Token | 值 | 建议用途 |
|---|---|---|
| `--space-0-5` | 2px | 极小内部间隔 |
| `--space-1` | 4px | 标题与简短说明、紧凑内容 |
| `--space-1-5` | 6px | 图标与文字、标签与控件 |
| `--space-2` | 8px | 相邻操作、紧凑列表 |
| `--space-3` | 12px | 表单项、设置行内边距 |
| `--space-4` | 16px | 标准卡片内边距、同组卡片间隔 |
| `--space-5` | 20px | 对话框内边距 |
| `--space-6` | 24px | 大区域内边距 |
| `--space-8` | 32px | 独立大区域间隔 |
| `--space-10` / `--space-12` / `--space-16` | 40 / 48 / 64px | 空状态或大面积留白，按组件使用 |

用途列是默认建议；具体组件已有确认规则时，以组件配方为准。

当前 14px、15px、22px 等值不能一律视为错误。例如《UI 规范》§17 明确要求卡片内容块与操作行使用 14px 上间距。对此提取**组件语义 Token**，不将所有特殊值添加成全应用随意选用的基础刻度：

| 建议组件 Token | 值 | 状态与来源 |
|---|---|---|
| `--page-padding-block-start` | 20px | 提取 `.main` |
| `--page-padding-inline` | 22px | 提取 `.main` |
| `--page-padding-block-end` | 24px | 提取 `.main` |
| `--page-section-gap` | 16px | 提取 `.route-section.active` |
| `--card-padding` | `var(--space-4)` | 沿用卡片 16px |
| `--card-header-gap` | `var(--space-3)` | 沿用卡片头横向 12px |
| `--card-header-body-gap` | 15px | 提取 `.card-head`；尚未改成 16px |
| `--card-block-gap` | 14px | 沿用 UI 规范 §17 |
| `--setting-row-padding` | `var(--space-3)` | 沿用设置行 12px |
| `--setting-row-gap` | 14px | 提取 `.setting-row` |
| `--form-label-control-gap` | `var(--space-1-5)` | 提取 `.modal-field` 的 6px，作为通用表单候选 |
| `--action-bar-gap` | `var(--space-2)` | 建议通用操作栏 8px；原工具栏存在 7px，先登记再迁移 |
| `--dialog-padding` | `var(--space-5)` | 沿用对话框 20px |

相邻区域的距离只由一个布局拥有：采用父级 gap 后，移除对应子级 margin，避免间距叠加。`UiCard` 与 `UiCardBody` 不同时各加 16px；建议外层只管理表面，由内部区块分配内边距，最终外观保持现有尺寸。

`Stack/Inline` 默认仅接受基础刻度枚举；卡片的 14px 通过卡片内部规则处理，不开放随处传入 `gap="14px"`。

### 7.6 字体、字号、行高、字重

字体沿用两个入口，组件不得各自指定字体族：

```css
/* 现有字体栈快照；本次不调整字体优先顺序。 */
--font-ui: "OpenAI Sans", "Pretendard Variable", Pretendard,
  "Noto Sans KR", "Apple SD Gothic Neo", "Malgun Gothic",
  -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
  "PingFang SC", "Microsoft YaHei", system-ui, sans-serif;
--font-code: ui-monospace, "SFMono-Regular", "Cascadia Code",
  "JetBrains Mono", "Noto Sans Mono CJK KR", Menlo, Consolas, monospace;
```

这是字体选择顺序，不代表已经随应用打包这些字体。要保证不同机器使用完全相同的字体，需要另行明确随包字体及资源方案；本次保留现状，不宣称跨系统字形完全一致。路径、命令、日志使用 code 字体；普通标题、说明、控件使用 UI 字体。

字号沿用现有值；以下行高与字重组合为**目标语义配方建议**，并非当前组件全部已统一：

| 文字角色 | 字号 Token / 值 | 行高 | 字重 | 用途 |
|---|---|---|---|---|
| 页面标题 | `--text-display` / 24px | 1.2 | 600 | 页主标题 |
| 对话框标题 | `--text-title` / 20px | 1.35 | 600 | 对话框标题 |
| 卡片/区域标题 | `--text-subtitle` / 16px | 1.35 | 600 | CardTitle、Section 标题 |
| 正文 | `--text-body` / 14px | 1.5 | 400 | 主要阅读内容 |
| 设置项名称 | `--text-body` / 14px | 1.5 | 600 | SettingRow 标题 |
| 常规控件文字 | `--text-control` / 13px | 1.35 | 500 | 通用控件默认建议 |
| 字段标签/短说明 | `--text-label` / 12px | 1.5 | 400；标签可 500 | 表单说明、次要内容 |
| 辅助元信息 | `--text-caption` / 11px | 1.5 | 400 | 时间、计数说明 |
| 紧凑标记 | `--text-micro` / 10px | 按标记组件配方 | 600 | Badge、短标签 |

约束：

- 10/11px 不作为新增长正文的默认值；现有安装向导提示等保持现行口径，列入视觉复核，不静默改字号。
- Input 当前是 12px，而“常规控件 13px”是候选统一方案，不能未经验证直接覆盖全部输入框。
- 行高基础值沿用 `1.2 / 1.35 / 1.5 / 1.6`；代码块现有 1.7、通知标题 1.3 等先作为组件特例提取，是否统一另行判断。
- 建议新增 `--weight-regular:400`、`--weight-medium:500`、`--weight-semibold:600`、`--weight-bold:700`。现有 650 不机械改写；迁移时记录其所属组件与实际字体效果，再决定保留专用值或归档。
- 字距默认 0，辅助强调沿用 `.04em`；页面标题现有 `-.02em` 提取成标题专用 Token。
- 固定高度控件使用适当 line-height 与居中，不通过让文字行高等于控件高度处理所有场景。
- 字体栈、字重或字号变化后检查中英文混排、数字宽度、长路径截断；数字对齐场景可使用 tabular-nums。

### 7.7 颜色：固定亮暗映射及使用语义

以下沿用当前 Token 值；组件使用语义，不自行选近似色：

| 角色 | 当前 Token | 亮色 | 暗色 |
|---|---|---|---|
| 应用背景 | `--bg` | `#f9f9f9` | `#212121` |
| 卡片/对话框表面 | `--panel` | `#ffffff` | `#262626` |
| 次级底面 | `--rail` | `#f9f9f9` | `#171717` |
| 抬升表面 | `--raised` | `#f4f4f4` | `#303030` |
| 抬升表面悬停 | `--raised-hover` | `#ececec` | `#3a3a3a` |
| 普通描边 | `--border` | `#e6e6e6` | `#3d3d3d` |
| 弱描边 | `--border-soft` | `#f0f0f0` | `#333333` |
| 主要文字 | `--text` | `#0d0d0d` | `#ececec` |
| 次要文字 | `--muted` | `#6e6e6e` | `#a6a6a6` |
| 辅助文字 | `--faint` | `#707070` | `#9a9a9a` |
| 主操作底色 | `--accent` | `#0d0d0d` | `#ececec` |
| 主操作悬停 | `--accent-hover` | `#3d3d3d` | `#ffffff` |
| 主操作文字 | `--accent-ink` | `#ffffff` | `#0d0d0d` |
| 成功 | `--green` | `#0a7d5c` | `#4ecb9d` |
| 风险/失败 | `--red` | `#b91c1c` | `#f87171` |
| 警告 | `--amber` | `#9a4a08` | `#fbbf24` |
| 信息 | `--blue` | `#1d4ed8` | `#7aa2ff` |
| 品牌装饰 | `--brand` | `#3941ff` | 继承亮色值 |

建议为 green/red/amber/blue 增加 success/danger/warning/info 语义别名，保留旧变量兼容。状态浅底沿用各自 `*-soft`；不在业务页面重新计算透明度。

材质作为整体配方保留：

- 内容页：亮色 `rgba(255,255,255,.40)`，暗色 `rgba(38,38,38,.40)`；沿用《UI 规范》§21，不新增页面 backdrop-filter。
- 通知宿主：亮色 `rgba(244,245,249,.52)`，暗色 `rgba(34,36,42,.56)`，使用现有 8px 模糊及通知子层 Token；不将通知材质应用到每张普通卡片。
- 环境光、通知悬停与取消操作的完整值继续引用现行 Token 和《UI 规范》§2.2–2.3，不在这里建立另一份色表。
- 品牌色不自动替换主操作黑白强调色；成功状态也不意味着整张卡片染绿。
- 新增颜色需要明确角色及亮暗配对，并检查实际叠加背景上的可读性；已有色值不等于已经通过对比度验收。

### 7.8 尺寸、圆角、描边、阴影与层级

| 类别 | 基线/建议 | 状态 |
|---|---|---|
| 控件尺寸档 | sm 28px、md 34px、lg 40px、touch 44px | 沿用 Token；以最小高度为主，不能裁掉多行内容 |
| 常规图标按钮 | 32 × 32px | 沿用 UI 规范；提取专用尺寸 Token |
| 通知内控件 | 高 24px、文字按钮内距 2px 9px | 沿用通知 Token；图标按钮不叠文字内距 |
| Tab | 高 36px、指示条 2px | 沿用 UI 规范，提取专用 Token |
| 图标 | 14 / 16 / 20px；线宽 1.8 | 沿用 Token 与控件规范 |
| Switch | 38 × 22px，圆点 16px | 沿用 Token |
| 圆角刻度 | 4 / 6 / 8 / 12 / 16px，胶囊 999px，圆形 50% | 沿用 |
| 圆角映射 | Input 6px、Button/SettingRow 8px、Card 12px、Dialog 16px | 沿用主要实现；通知条目等独立变体另保留 |
| 描边 | 常规 1px；焦点环 2px、外偏移 2px | 提取现行常规规则；危险提示侧边标记保留专用宽度 |
| 阴影 | 常规卡片 `--shadow-card`，浮层 `--shadow-md` | 沿用；Modal 现有独立阴影需先提取，不能直接替换 |
| 过渡 | fast 120ms、normal 180ms | 沿用；减少动态效果时为 0 |
| 层级 | sticky 20、overlay 30、popover 40、modal 50 | 沿用；通知例外见下文 |

阴影亮暗配方由 Token 文件唯一给值；父组件不得通过任意 box-shadow 改变子组件层级表达。旋转加载等持续动画不套用 120/180ms 的过渡时长，按实际组件单独登记。

层级不能只整理数字：当前通知宿主有 80 的专用值，打开模态时还会降低层级。应提取宿主 Token 并保留模态避让规则；Dialog 内弹出菜单也要验证 Teleport 与 stacking context，不能简单认为 40 永远位于 50 以下或把所有浮层改成 9999。

### 7.9 组件样式配方与交互状态

每类组件固定一套默认配方，页面只选择有限的 variant/size/density，不自行覆盖颜色、字号、padding：

| 组件 | 配方要求 |
|---|---|
| UiButton | size 选控件高度；variant 选主/次/幽灵/危险；固定字体、圆角、图标间距；loading 不使按钮宽度跳变 |
| UiInput/UiSelect | 统一字段尺寸、描边、焦点环、错误态；只读与禁用分别表达；保留当前 12px 基线直至统一方案验收 |
| UiCard | 固定表面、描边、圆角；Header/Body/Footer 合作分配内距；普通卡片不因 hover 自动表现为可点击 |
| SettingRow | 固定 12px 内距与 14px 列间距；窄空间通过明确布局变化容纳控件；多行说明不被固定行高截断 |
| FormField | label/control/help/error 的位置与间距统一；错误信息不覆盖或替代标签 |
| UiBadge | 统一圆角、字体、内距；页面与通知是不同变体，不借父级 span 规则凑样式 |
| UiDialog | 固定标题、正文、操作区配方；业务通过插槽填内容；宽度用预设及视口上限，保留既有缩放折算 |

交互状态至少覆盖 default、hover、active、focus-visible、disabled、loading、invalid 中适用的部分。禁用透明度沿用现行 `.42`，作为待提取 Token；焦点不只依靠颜色变化；错误和危险动作必须同时有文字或图标语义。

主题、禁用和错误状态在公共组件内实现，业务卡片不用单独维护一份深色主题补丁。

### 7.10 规范怎样进入代码与检查

执行方式：

1. 先盘点数值与消费者，区分普通重复值、已确认特例和真实冲突。
2. 优先做等值 Token 提取，再做组件迁移；需要改变取值的调整另列视觉差异。
3. 新增/迁移代码必须使用批准的基础 Token 或组件配方；旧代码按功能迁移，不一次性格式化整个仓库。
4. 对样式采用可解析的检查，限制组件内新增颜色常量、任意字号/字重/层级及未登记间距；规则工具不足时补项目检查，不能只配置一个 Stylelint 就宣称已约束全部值。
5. 允许 0、auto、百分比、fr、内容尺寸、基于 Token 的 calc、动态进度等必要值；品牌图形和原生窗口尺寸以限定文件或限定属性登记例外，不做全局豁免。
6. 样式检查同时考虑 Vue SFC、独立 CSS 和模板内联 style；动态宽度允许，内联硬编码色板不允许绕过规则。
7. 预览覆盖真实组件的尺寸、主题、文本与状态组合；关键桌面布局另做真机验证。

已发现的待统一事项：页面 padding 使用 22px；卡片头部到正文 15px；UI 规范要求的卡片块间距 14px；局部 650 字重；Input 12px 与常规控件 13px 候选；通知宿主 80；窗口最小尺寸在 UI 文档/base CSS 为 900×600，而 Tauri 配置为 960×640。

这些差异先保留并标明来源。是否调整必须形成明确视觉决定；不能把“规范化”解释为把所有数字改成 4 的倍数，也不能在差异未处理前宣称整个项目样式已固定并落地。

### 7.11 响应式、内容与浮层样式补充

- 容器查询以实际内容区域为依据，不能只按操作系统窗口宽度判断可用空间。现有 `base.css` 的 759/819/900/1280px 查询属于不同区域，迁移时逐项登记查询容器与行为，不直接合并成一套手机/平板/桌面断点。
- 每个布局配方补充最小内容宽度、列数/换行变化、max-width 与滚动归属；Flex/Grid 子项明确何处需要 `min-width:0`，避免长路径撑破容器。
- 表格保留表头、键盘可达动作和稳定列语义；窄容器是横向滚动还是改布局按实际功能规定，不挤掉操作入口。
- 标题/按钮短标签、长路径、代码、Markdown 使用不同溢出策略。省略后的完整值应有可访问的查看/复制方式；tooltip 不能成为唯一信息来源。
- MarkdownContent 只负责受限正文呈现，扩展客户端信息仍在业务组件中。现有 `renderMarkdown` 的先转义、链接仅保留文字等策略保持；普通对话框继续使用模板。
- `v-html` 内容不自动带 scoped 标记，slot 内容也有自己的样式归属。Markdown 只允许在所属容器内使用受限 `:deep()` 或带命名空间的正文样式；不能为了解决正文失样式恢复全局 `p/span/button` 覆盖。依据见 S4。
- Teleport 的目标容器需继承主题与统一缩放。不能将原本位于 `.app-window` 缩放上下文中的弹层直接移到 body 后就假定尺寸不变；浮层坐标、遮罩范围、滚动锁与原生子 WebView 避让一起验证。
- Token 别名必须无循环、无未定义引用；主题覆盖只改变约定语义值。Token 类型区分长度、无单位行高、颜色、时长与层级，不能只检查名字存在。

## 8. 后端模块化调整

保留模块化单体，分离三个职责：

1. `commands/ipc`：各自协议适配、参数和错误映射。
2. `application`：备份后导入、安装后刷新等跨模块流程。
3. `modules`：业务规则与领域服务；基础设施承担外部访问。

简单查询可直接调用模块公开服务，不要求经过空 application 转发。GUI 与 CLI 复用业务服务，仍允许各自协议字段和呈现不同。

`ocxd` 保持运行中实例的客户端身份，不成为第二个独立写入者。

按职责拆安装校验、执行、提交、取消及测试；扩展的脱敏、投影、发现和配置策略明确归属。纯规则不依赖 Tauri；只在有替换或隔离测试价值的外部边界引入 trait。

DTO 和领域类型区分：跨 IPC 类型逐步归 contracts；领域模型留模块。先补序列化一致性测试，再评估 Rust → TypeScript 生成；不手改生成文件，不把编译期类型当外部数据验证。

## 9. 对 IMP-03 的执行前补充

IMP-03 负责目录与路径，本计划负责内部职责。两者分阶段实施。

上一轮评审对以下事项提出复核要求，本草稿不将它们视为已完成的构建验证：

- Tauri 后端配置查找与前端 package 查找分别验证，不仅凭目录深度推断。
- 明确仓库根命令、前端目录、Tauri 目录和构建 hook 的工作目录；新增根 package 后重新验证发现行为。
- `frontendDist` 与构建 hook 的相对路径分别判断，不能机械地使用同一基准。
- 当前 CI 打包 job 需要在自己的干净工作区安装前端依赖，不能依赖另一 job 的安装结果。
- 本地开发、类型检查、测试和真实打包使用同一组可复现入口。

这些事项应在执行 IMP-03 前回填其具体命令与配置，不在本次文档工作中直接改动构建行为。

## 10. 分阶段调整与验收

| 阶段 | 交付 | 核心检查 |
|---|---|---|
| A 草案细化 | 确认命名、组件分类和第一批 API | 本文可评审，候选与已定事项明确 |
| B 外层目录 | 按 IMP-03 迁移及补足路径入口 | 干净检出可安装、检查、真实打包 |
| C 公共组件试点 | Button、Card、SettingRow、FormField、Stack、Inline、Notice 及预览 | 从真实页面抽取，主题、缩放、焦点与布局不变 |
| D 功能纵向迁移 | 先扩展管理，再设置分区与 runtime | API、状态、组件、局部样式、测试一起归位 |
| E 应用与后端边界 | 拆总 store/controller、bootstrap，收敛共同用例 | 无重复订阅，GUI/CLI 业务行为一致 |
| F 门禁与回写 | 边界检查、格式、组件约定、现状文档 | 全量相关检查与真实桌面冒烟 |

每阶段还应覆盖第 13.8 节中相关场景；发现的行为修复单独标识，不夹在等值目录/CSS 迁移中。

本表为阶段概览；执行前必须按第 16 节补齐可执行计划。错误与性能修复按第 14–15 节纳入各阶段，最终按第 17 节完成全入口、流程、原型与打包复核，不能仅依据本表宣布完成。

公共组件试点至少应用于两个不同内容的现有区域，以验证真正的复用边界；业务内部组件不受“两处复用”条件限制。

验收关注：

- 页面可以从组件名称看出业务区域，细节位于对应功能。
- 同类卡片、设置行、表单错误和操作区只有一套公共规则。
- 公共组件不读取业务 store 或直接调用 Tauri。
- 大型 store/controller 不以另一个总文件的形式重新出现。
- 亮暗主题、50%/100%/200% 缩放、长路径与最小窗口均有检查。
- 原生标题栏拖动、子 WebView、通知与模态避让有真实桌面验证。
- 现有契约、安装/同步/迁移/取消/失败回滚行为保持。
- 格式化提交与语义改动尽量分开，降低评审噪声。
- 本轮只有文档编辑，以上均为后续验收目标。

## 11. 后续细调点

1. `components/ui + layout + patterns` 命名是否保留；若改用 shared，应整体替换公共层入口，不并存两套。
2. Ui 前缀、Card 部件和 slots 的最终 API。
3. `SettingGroup` 与 `FormSection` 的视觉和语义区别，避免重复抽象。
4. 布局组件的默认间距、换行、窄宽度与缩放策略。
5. 第一批公共组件的实际消费者和迁移顺序。
6. 开发预览的承载方式及与现有 audit harness 的关系。
7. 是否引入无样式交互原语，及对现有依赖约束的影响。
8. 第 7.5–7.10 节的组件专用值与待统一事项；数值调整须与等值迁移区分。
9. 第 13 节 P1 项的入口与状态契约，以及对应可执行验收；尚未完成实现修复。

## 12. 外部参考与核实范围

参考获取于 2026-09-27。GitHub 默认分支会变化，下列 commit 是本次 API 读取的树版本；不代表相关项目发布版本，也不代表它们全部架构都适合本项目。

### S1：Folo

- 仓库：`https://github.com/RSSNext/Folo`
- 分支：`dev`
- commit：`0f35b68deab3fa81216e17bcd3e33c58e8056239`
- 目录证据：`https://github.com/RSSNext/Folo/tree/0f35b68deab3fa81216e17bcd3e33c58e8056239/apps/desktop/layer/renderer/src/components`
- 包级证据：`https://github.com/RSSNext/Folo/tree/0f35b68deab3fa81216e17bcd3e33c58e8056239/packages/internal/components`
- 核实内容：应用内 common/ui 分类，以及独立 internal/components 包。借鉴归属与复用范围，不移植其 React/Electron 组织细节。

### S2：shadcn-vue

- 仓库：`https://github.com/unovue/shadcn-vue`
- 分支：`dev`
- commit：`67c9a3926dc0a854507b325c6337ff2210d16379`
- 源码证据：`https://github.com/unovue/shadcn-vue/tree/67c9a3926dc0a854507b325c6337ff2210d16379/apps/v4/registry/bases/reka/ui/card`
- 用法说明：`https://www.shadcn-vue.com/docs/components/card`
- 核实内容：Card/Header/Title/Description/Content/Footer/Action 的组成与 components/ui 导入方式。借鉴组合 API，不代表引入其样式体系或直接安装组件库。

### S3：Feature-Sliced Design

- 官方说明：`https://feature-sliced.design/docs/reference/layers`
- 核实内容：Shared 包含 UI、库、外部连接等基础能力；共享 UI 可以包含页面布局和 UI 交互逻辑。
- 本项目只参考公共层职责说明，采用自己的简化目录与依赖规则。

### S4：Vue 官方文档

- 样式：`https://vuejs.org/api/sfc-css-features.html`
- 生命周期：`https://vuejs.org/guide/reusability/composables.html`
- 核实内容：scoped、slot、v-html 的样式归属；有副作用的 composable 需管理清理。用于第 7.11、13.3 节。

### S5：Tauri 与 W3C 官方文档

- Tauri capabilities：`https://v2.tauri.app/security/capabilities/`
- 模态交互：`https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/`
- 文字对比度：`https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html`
- 核实内容：窗口/WebView 的权限边界，模态内焦点循环与关闭后焦点恢复，以及文字对比度要求。用于第 13.5–13.6 节；引用不代表本项目通过相应标准的完整验收。

## 13. 完整性审计与补充约束（v0.3）

### 13.1 审计结论与范围

本轮重新检查草案、核心 UI 规范、前端入口、路由/主题/store、设置/面板/详情组件、受限 Markdown、Tauri capabilities 和 CI；采用静态阅读及官方资料核对。未运行应用、回归测试、性能测试或完整安全审计。

结论：v0.2 已覆盖目录、组件分类和设计数值，但对模块间服务、异步状态、样式作用域与迁移验收仍有缺口。以下优先级用于安排方案落实：P1 在对应模块迁移前解决，P2 随迁移阶段落实；不是运行事故等级。所有条目均为“方案已补、实现待验证”，不表示缺陷已修复。

| ID | 优先级 | 遗漏/冲突及本地证据 | 文档处理 |
|---|---|---|---|
| A01 | P1 | 方案禁止 page/feature 导入 app，但现有页面直接使用路由 store；路由 store 又导入所有页面 | §13.2 明确导航类型、注入接口与 app 注册分离 |
| A02 | P1 | 未分配主题生命周期；`stores/theme.ts` 使用 localStorage、系统主题监听与立即 apply | §13.2 增加 appearance 与存储归属，保持首帧和面板同步 |
| A03 | P1 | 仅规定“初始化一次”，未覆盖晚返回、乱序、部分订阅成功与卸载；Settings 的匿名 `opencodex:guide-installation` 监听无配对移除 | §13.3 补生命周期与异步策略 |
| A04 | P1 | `startup.ts` 在 await refresh 后才检查 deadline，单次不返回无法由循环超时兜底；main 在挂载前 await preferences | §13.3 列入独立启动修复与挂起请求测试 |
| A05 | P1 | 新组件会重组弹层，但 PanelRoute 用多个总 store 字段判断何时隐藏原生子 WebView | §13.5 增加遮挡协调服务与多个表面并存规则 |
| A06 | P1 | `useAppController` 对 window 暴露 setState/setEnvironmentFixture，没有 DEV 条件；recheckEnvironment 又被页面用作生产入口 | §13.6 拆开发夹具与正式服务，检查正式包 |
| A07 | P1 | scoped 迁移方案缺富文本例外；ExtensionDetailModal 使用 v-html，现有样式在全局 CSS | §7.11、13.6 补正文样式与渲染边界 |
| A08 | P2 | 组件表缺表单草稿、异步错误、中文输入法、数字空值及密钥生命周期 | §13.4 补输入与提交协议 |
| A09 | P2 | 固定数值未绑定容器查询、长内容、层叠与缩放 | §7.11 补布局行为与 Token 检查 |
| A10 | P2 | 全局 types/ui 与 stores/routes 分别定义 DiagnosticsTab，前者没有 doctor；labels/日期格式归属不明确 | §13.2、13.7 补类型与展示规则的单一归属 |
| A11 | P2 | 已有焦点要求但未指定模态栈、播报、背景不可交互及真实检查 | §13.5–13.6 补交互验收 |
| A12 | P2 | 尚无逐模块迁移映射、旧入口退场条件、性能基线和测试责任表 | §13.7–13.8 补迁移与验收 |

### 13.2 应用服务、导航与持久化归属

导航拆成：

- `navigation/`：RouteName、各路由参数、hash 的纯解析/序列化和校验。诊断 tab、设置分区只能定义一次，未知参数回到规定默认值。
- `composables/useNavigation.ts`：只声明/获取类型化导航接口与只读当前路由；缺 provider 时报清楚的开发错误，不悄悄创建第二份状态。
- `app/router/`：实现导航服务、注册页面组件、安装/卸载 hash 监听。页面、feature 不直接导入 app 实现。
- `app` 统一选择页面组件，去掉 App 和路由 store 各自维护一份组件映射的结构。

保留现有 hash、显式入口优先级、托盘入口、设置分区和诊断 tab；迁移不要求引入 Vue Router。类型化接口可以使用现有实现，不为架构名义更换依赖。

主题与缩放归 `app/appearance` 装配，功能通过通用 composables 中注入的外观接口读取/提交请求。接口包含最小只读状态与动作，其类型不导入 app 实现。偏好文件仍归 preferences 功能；app workflow 将保存结果应用到外壳与面板。不能让 runtime、panel 直接修改 preferences store 内部。

| 状态 | 事实来源/归属 | 迁移约束 |
|---|---|---|
| 主题选择 | 当前 localStorage `ocx-theme`；系统模式由 matchMedia 解析 | 保留键名与失败兜底；首帧前应用，系统监听可释放 |
| 界面缩放与其他偏好 | 后端偏好契约；preferences 功能 | 保存回读为准，预览与已保存值区分 |
| 当前路由 | hash + 导航服务 | 不导入页面组件的类型定义可供下层使用 |
| 查询列表与运行状态 | 对应 feature store 的后端快照 | 不复制到 UI 公共组件或另一持久化层 |
| 表单草稿/临时密钥 | 所属功能的短生命周期状态 | 不自动持久化整个 Pinia 状态 |

相同主题选择如何同步到官方子面板沿用现有协议，不重绘其样式；localStorage 持久化与后端偏好不能不经说明合并。

### 13.3 异步、生命周期与错误处理协议

- 建立 idle/loading/ready/empty/error/stale 等**适用**状态，明确“从未加载”和“刷新失败但保留旧值”的区别；不强迫每个请求使用全套状态机。
- 查询响应用请求序号/当前参数校验过滤过期结果；快速切 tab、搜索、打开另一详情后，旧结果不得覆盖新界面。
- 保存按业务需要串行化或使用最后意图合并。参数去抖只处理界面输入，不作为后端写锁；避免旧保存回读覆盖用户刚输入的新草稿。
- 前端超时只表示停止等待，不意味着后端写操作取消。取消安装等动作必须取得后端明确结果；有副作用请求不能自动重试。
- 日志、列表等可重试查询需限制触发频率；事件通知触发刷新时可合并，不能让每个小组件重复轮询。
- 对事件、计时器、watch、ResizeObserver、requestAnimationFrame 明确拥有者与 disposer；卸载、测试重建和热更新后清理。异步 listen 晚于卸载返回时立即 unlisten，部分注册失败也释放已注册部分。Vue 生命周期依据见 S4。
- 本地静态检查发现的 `waitForSettledSnapshot` 不是严格的总等待上限。独立修复应同时覆盖偏好读取和快照请求的等待策略；延迟返回仍要过滤，不能仅添加 Promise.race 就宣称底层任务已取消。
- API 层保留既有后端 code/message 结构；feature 映射为中文反馈与可重试性。取消、未配置、校验失败、权限不足、操作失败分别处理；不要将所有结果压成无法排障的 false。
- 异常反馈明确唯一负责方，避免 store、composable 和页面各弹一次 Toast；失败细节在适当位置持久展示，日志只记脱敏后的技术上下文。

### 13.4 输入、表单与公共组件协议

- 输入组件明确 value 类型、空值、范围、失焦/回车行为；NumberInput 的编辑态允许空文本，提交时才按功能契约归一，不把清空立刻变成 0。
- 中文输入法组合期间不能误触发回车提交、危险动作或未完成的搜索；验证中文、粘贴、键盘与鼠标交互。
- props 只读；统一 v-model 更新事件，业务提交用独立 submit 事件。区分 loading、disabled、readonly，不把三者混用。
- 表单保留 draft 与 persisted 的区分，约定何时校验、何时展示错误、成功后何时清理。切页/关闭时根据已有行为保留或清理草稿，不默认新增所有表单的离开确认。
- 后端校验仍为写操作最终边界，前端校验不能替代它；提交中重复点击不产生第二次写入。
- label、description、error 的 ID 关联，必填语义、aria-invalid 与必要的错误摘要有统一实现；自动生成 ID 不冲突，attrs 透传到正确输入节点而非外层容器。
- Select/Tabs/Menu 明确键盘选择模型与禁用项处理；含交互的卡片不嵌套 button，也不把普通 div 仅靠 click 伪装成完整控件。
- 密码、同步凭据、安装代理 secret 在不再需要时释放引用，禁止进入 URL、日志、Toast、持久化 UI 状态或组件演示数据；不承诺 JavaScript 字符串可被安全擦除。
- ref 暴露仅限 focus 等必要能力；父组件不能通过 ref 修改组件内部业务状态。

### 13.5 浮层、焦点与原生桌面适配

增加一个由 app 提供的 surface 协调接口：记录可见表面及其关闭/焦点责任，向 panel workflow 提供“当前是否存在必须让原生面板避让的表面”。该接口只表达 UI 状态，不反向引用 notifications/runtime 的 store。

Dialog、通知中心、Toast、任务卡沿用现有避让范围；新组件逐项声明是否参与。多个表面重叠时，只有最后一个相关表面关闭才恢复子 WebView；show/hide 的异步结果必须保持最新意图。

模态需处理初始焦点、Tab 循环、Escape、背景不可交互、关闭后焦点恢复，以及触发按钮已卸载时的安全目标。嵌套下拉菜单的 Escape 先关闭最上层；危险操作不默认将焦点放在最易误触的确认按钮。依据见 S5。

滚动锁、返回页面后的滚动位置、拖拽区与控件命中分别验证；不让页面 Header、ButtonGroup 重构遮挡原生窗口拖拽区。原生文件对话框取消按正常结果处理，浏览器组件预览提供显式桩实现。

`platform` 包装 invoke/listen/原生对话框/剪贴板等适配，不把不同 SDK 堆成巨型 service。真实业务外部读写继续由后端承担。平台能力缺失返回明确不可用，不仅凭 user-agent 显示可用功能。首期真实验收仍为既定 macOS 平台，不扩大 Windows 支持承诺。

### 13.6 可访问性、渲染与开发入口

- 将只依赖颜色的状态补充文字/形状；Toast 适度播报，持续进度不要逐帧触发屏幕阅读器播报；关键失败不能只靠短暂 Toast。
- 文字对比度以实际叠加后的表面检查，普通文字目标至少 4.5:1、大文字至少 3:1；大文字按 W3C 定义判断，不把所有标题直接归为大文字。禁用控件等例外单独记录；这是验收目标，尚非通过声明。依据见 S5。
- 将焦点可见性、键盘路径、VoiceOver 冒烟、减少动态效果与缩放加入实际检查，不能只检测 aria 属性存在。
- Markdown 保持现有受限渲染契约；普通业务文本使用插值。外部 URL/路径交给受控平台入口处理，不能让通用富文本或卡片组件自行授予打开任意目标的能力。
- Vue 组件移动不需要扩大 Tauri capabilities。迁移保留窗口/WebView 标签与既有允许范围，特别核对官方子 WebView；若引入插件，单独评审能力变更。不能声称 Rust 自定义命令仅靠前端隐藏就获得授权保护。
- 调试隔离不能只检查 `dev/auditHarness`：`useAppController` 现有 setState/setEnvironmentFixture 属于需审查入口。`recheckEnvironment` 当前被正式设置页使用，先迁入正式环境服务，再删除生产全局入口；不能直接删掉导致功能断裂。
- 正式构建检查开发路由、fixture 模块和调试全局是否存在；浏览器预览的桩不会静默回退到真实安装、迁移或同步。

### 13.7 迁移映射、命名与工程约束

| 当前入口 | 目标归属 | 迁移完成条件 |
|---|---|---|
| stores/routes + App 页面 switch | navigation + app/router | 类型/解析唯一，页面注册唯一，原 hash 兼容 |
| stores/theme + scale | app/appearance + 通用纯归一函数 | 持久化键、首帧、系统切换、面板缩放不变 |
| stores/app | 各 feature store + app feedback/hosts | 无重复事实；跨功能调用经上层编排 |
| useAppController | app bootstrap/workflows + 各功能 composable | 无生产 fixture 全局，无重复订阅 |
| commands/*.ts | features/*/api + platform + contracts | 原命令名、参数与事件载荷兼容 |
| types/ui + labels + presentation | navigation / contracts / 功能 model、presentation | DTO/路由/视图类型明确；DiagnosticsTab 重复定义消除 |
| markdown + ExtensionDetailModal | 受限 MarkdownContent + 扩展详情组件 | 转义、截断、样式、焦点与动作保持 |
| 全局 base.css | 组件局部样式 + 基础样式 | 原选择器消费者迁完再删除；不以追加覆盖维持两套 |

中文界面及专有名词例外沿用《UI 规范》§10。功能枚举的中文映射放其 presentation，基础日期/数字格式能力放 lib，具体“何时显示什么”留功能；缺失值、时区和单位格式记录明确规则，不在重构中顺带换语言或引入国际化框架。

Vue 文件 PascalCase、composable 使用 use 前缀，功能目录 kebab-case、Rust 模块 snake_case；普通 TS 文件本版使用 kebab-case，`api/store/model/presentation/index` 等语义入口保持短名。命名搬迁与逻辑改动分开。

功能 index 只导出公开 API；本功能内部直接相对引用，避免从自身 index 绕回。禁止一个根 barrel 重新导出全部组件与功能并隐式加载所有页面。

新增就近测试后同步检查 tsconfig、Vitest、Lint 的包含/排除范围；生产源码类型检查与测试环境全局类型明确区分。工具版本、根/UI 安装步骤、锁文件归属、构建产物和生成类型的提交策略写入开发入口；依赖安全或版本升级不夹在结构迁移中。

每个功能切片记录“旧入口 → 新入口 → 对应回归 → 删除条件”。允许短期单向兼容导出，不允许复制一份 store/组件后长期并存；迁移后删除旧入口、失效样式与过时测试路径。阶段提交可回退；本次不迁移用户数据格式，因此不新增数据迁移程序。

### 13.8 验收矩阵与性能基线

| 层面 | 关键场景 | 检查方式 |
|---|---|---|
| 类型/依赖 | 禁止跨层引用、循环引用、未定义 Token、DEV 入口隔离 | 静态检查与正式构建产物检查 |
| 公共交互 | 键盘、输入法、v-model、禁用/只读、表单失败、模态焦点 | 组件交互测试；原生相关场景真机 |
| 生命周期 | 重复挂载/卸载、晚返回 listen、部分失败、重复窗口事件 | 可控事件桩和计时器，不依赖真实等待 |
| 请求竞态 | 慢旧请求晚返回、快速切详情、保存乱序、取消后仍有事件 | 可控 Promise 和事件序列 |
| 启动 | 偏好失败/挂起、状态请求挂起、已有 hash、未知参数 | 启动流程测试与真实冷启动 |
| 视觉布局 | 两主题、最小/默认窗口、50/100/200% 缩放、长中文/路径/空值 | 稳定 fixture 的组件预览与关键组合截图 |
| 桌面壳 | 原生拖动、子 WebView、通知/模态并存、主题与缩放、关闭隐藏/托盘恢复 | macOS 真机冒烟，jsdom 不能替代 |
| 业务契约 | 安装/卸载、扩展写入、同步、导入失败、回滚及 GUI/CLI 对齐 | 复用相关现有测试，新增改动边界的回归 |
| 工程入口 | 干净检出安装、类型、测试、构建、真实打包 | 同一组根入口与各 job 独立依赖安装 |

性能先记录可复现基线：冷启动到界面可交互、页面切换、长列表/Markdown 渲染、后台轮询次数、包体，以及重复开关页面/弹层后的资源是否释放。记录机器、数据量和采样方法，再确定允许波动，不凭空写统一毫秒门槛。

先复用现有分页和读取上限；仅在测量支持时加入虚拟列表、懒加载或缓存。组件增加不能导致每行重复注册全局监听或每个卡片独立查询同一快照。

验收不要求测试每个纯视觉 wrapper。保留有意义的契约/权限/样式约束测试，但原先直接读取 base.css 的测试需随样式归属迁移；关键布局通过真实渲染验证，不能删除失败断言后宣称通过。

文档补充完成不等于上述检查通过。进入实施前，相关阶段必须明确实际执行命令、环境和验收证据归属；仍按既有治理规则将执行记录放 `.adg/`。

## 14. 错误处理与日志规范

### 14.1 分层与契约兼容

错误处理覆盖“识别 → 状态收敛 → 展示 → 排障 → 恢复”，禁止仅统一 Toast 文案。

| 层面 | 必须承担的职责 |
|---|---|
| Rust 模块 | 保留真实原因、事务阶段与已发生的副作用；完成应有清理/回滚 |
| commands / IPC | 输出既有结构，脱敏，保留可判定的信息；不可把失败包装为成功 |
| 前端 API 适配 | 接收 unknown 并安全归一，识别协议与传输异常；不在每个组件重复解析 |
| 功能层 | 根据操作和状态决定中文摘要、可用动作、是否保留旧值和草稿 |
| 公共反馈组件 | 呈现字段错误、行内失败、通知或详情；不猜测业务失败原因 |
| 日志 | 记录脱敏技术信息和可关联的操作上下文；与用户摘要分离 |

当前 `src-tauri/src/errors/mod.rs` 使用 `{code: u16, message: string}`；码 1 对应不止一种原因，码 7 同时包含锁冲突与超时，托管运行时错误共用码 14。前端 `extensionErrors.ts` 还存在按消息正则归类的兼容逻辑。

因此：

- 不能仅看数字码臆断具体根因，也不能在普通组件里新增英文字符串匹配。
- 现有消息兼容识别集中在适配器，并为已知/未知载荷建立回归；未知原因明确表示未知。
- 可在前端内部定义分类、摘要、详情、操作建议等视图模型，不意味着将这些字段加入 IPC。
- 若需要细分稳定机器码或新增 `reason/operationId/retryable` 等跨边界字段，先更新《契约字段》与兼容方案，作为单独可评审改动；不擅自覆盖冻结码值或破坏 CLI 消费方。
- operationId 仅在已有或明确生成的操作上下文中关联，不伪造后端返回的标识。

### 14.2 错误分类与呈现规则

| 情况 | 用户表现 | 数据/动作要求 |
|---|---|---|
| 用户主动取消 | 关闭或“已取消”，通常不报红 | 必须区分取消选择、取消等待、后端确认取消 |
| 字段校验失败 | 错误定位到字段，保留输入 | 不发起无效写入；提交校验失败时焦点到错误区域 |
| 未配置/依赖未就绪 | 持久说明与前往配置动作 | 不把不可用显示为空列表或成功 |
| 查询失败 | 行内错误，允许适当重试 | 旧数据可保留但标明刷新失败/非最新，不能沿用旧成功态 |
| 写入失败 | “什么操作失败 + 已知原因 + 下一步” | 保留草稿；是否已修改、回滚与回读结果必须真实 |
| 超时/结果未知 | 明确“尚未确认结果” | 先回读/查询后端结果，不能直接重发安装、导入等写操作 |
| 部分成功/回滚失败 | 明确未完成部分与恢复入口 | 不宣称“当前配置未修改”；保留诊断证据 |
| 运行期非预期异常 | 可继续操作的局部兜底或启动失败页 | 清理加载状态，保留错误证据，避免白屏与无限转圈 |

例：只有确认原子写尚未提交或回滚成功时，才显示“保存失败，原配置保持不变”。超时且结果未知时应显示“尚未确认保存结果，请刷新后检查”，不能做同样保证。

字段错误、持久行内说明、Toast、通知和阻断对话框按影响选择；同一事件只由一个入口负责主反馈。Toast 用于短暂结果，不承载用户必须处理的唯一信息；持久通知沿用既有来源、去重、已读、解决状态契约。

### 14.3 排障、异常兜底与验证

- 默认显示中文可操作摘要，技术详情显式展开/复制；清理令牌、密码、认证头、带凭据 URL、代理 secret 和原始配置内容。
- 日志区分操作开始/结果、预期取消与异常；记录模块、动作、阶段、耗时和已有错误标识，完整本机路径按必要性最小记录。
- UI 与后端避免重复记录同一异常；必要关联通过上下文完成，不以泄露完整请求载荷换取可追踪性。
- Vue 错误边界、启动 catch、未处理 rejection 等兜底必须有责任位置与恢复方式；捕获不能只 console.log，也不能通过全局吞异常把测试变绿。
- 边界测试覆盖正常错误、未知码、畸形载荷、消息缺失、超时晚成功、取消、重复事件、部分写入失败与回滚失败；检查界面摘要、最终状态、日志脱敏及重试是否安全。
- UI、GUI 命令和 CLI 同一操作可采用不同展示形式，但事实与成功/失败结论一致。

## 15. 性能排查、修复与回归

### 15.1 必须有测量与修复闭环

性能工作不能以“组件拆小了”“改了 computed”作为优化结果。每项记录：场景和基线 → 复现与测量 → 原因 → 改动 → 同环境前后对比 → 业务回归。

| 场景 | 指标与测量边界 | 修复方向需由证据决定 |
|---|---|---|
| 冷启动/热启动 | 进程启动、首帧、可交互、业务就绪分别计时 | 挂起 IPC、有界等待、非必要阻塞初始化 |
| 路由切换/详情打开 | 动作到可见内容、主线程阻塞、请求次数 | 重复初始化、重渲染、同步 Markdown 工作 |
| 长列表/日志 | 数据规模、渲染耗时、滚动体验、DOM 数量 | 分页/上限、重复测量、无界积累；按需虚拟化 |
| 设置保存/缩放拖动 | 保存与布局调用次数、最终值、响应延迟 | 去抖、串行化、最后意图保护，不能丢最终保存 |
| 后台空闲 | 前后端 CPU、内存、轮询/网络/IPC 频率 | 无效轮询、重复订阅、日志刷屏 |
| 重复进入页面/开关弹窗 | 监听、定时器、内存及原生资源是否持续增长 | 生命周期释放、晚返回结果清理 |
| 安装/同步/导入 | 工作耗时、取消响应、锁持有、UI 可操作性 | 阻塞线程、重复读写、进度事件频率与背压 |
| 包体与构建 | JS/CSS/资源体积、开发资源是否混入 | 无效依赖、全量 barrel、重复资源、fixture 隔离 |

记录硬件、系统、构建模式、数据集规模、冷热缓存条件、样本次数与统计方式。交互测试记录可解释的多次样本；报告中位数与范围，样本足够时再报告尾延迟，不能用一次最优结果证明改善。

基线后在执行计划里写明指标目标与允许回归范围，再实施针对性优化；已存在的有界等待契约直接作为硬要求。基线本身存在严重缺陷时，不能以“不比原来差”作为合格。

### 15.2 本项目优先排查

1. 第 13 节发现的启动挂起、重复监听、晚到异步响应。
2. 多个卡片/组件是否重复查询同一快照，事件是否造成刷新风暴。
3. 快速缩放、ResizeObserver、子 WebView layout 调用是否叠加或丢最终状态。
4. 日志、扩展列表、详情正文是否保持读取/渲染上限。
5. Rust 长任务是否阻塞命令响应，锁是否跨不必要的外部等待持有。

可疑项经测量确认后修复；没有问题的项目保留测量结论，不制造无意义重构。优化不得降低状态真实性、遗漏事件、弱化权限/校验、取消备份或回滚保证。

### 15.3 修复验收

- 实质性性能缺陷建立可以复现旧问题的回归或基准探针；资源释放以重复生命周期观察为依据，不要求 GC 后内存字节完全相等。
- 首轮测量、修改后复测在可比条件下进行；额外偏差说明环境或数据变化。
- 对修复影响的操作、异常和原生流程补回归，再跑最终完整验收。
- 未达到目标继续定位；确实受外部 CLI/网络限制时分解本地与外部耗时，明确限制，不能伪造进度或用延时掩盖。

## 16. 标准实施计划与文档要求

### 16.1 文档职责与生命周期

遵循项目绑定的 `project-governance` Skill 及现行目录：

- IMP-03 是目录迁移事实入口，IMP-04 是本轮工程化与验收要求入口；不另建一份并行“总计划”导致范围和状态重复维护。
- `docs/` 保留长期技术决定、实施范围、最终行为和已知限制；《UI 规范》《系统架构》《契约字段》《术语与命名》在对应事实落实后回写。
- TASK、阶段检查点、RCP、测试清单实际执行状态、缺陷证据与证书位于 `.adg/`，按现行工具/格式创建；本文不发明新治理对象 schema。
- 正式审阅要求使用 TASK.required_checks 中稳定的 `review:<scope>` 名称；RCP.checks 记录相同名称、真实 passed/failed 和证据。阻塞项附明原因，不能伪造 passed。
- 没有必要发布时不建立虚假 REL；构建 .app/.dmg 不等于签名、公证或发布。

“先写计划”要产出能直接执行和评审的计划，不能只有“重构 → 测试 → 完成”。

### 16.2 执行前计划的必填项

| 项目 | 必须填写的内容 |
|---|---|
| 基线与依据 | HEAD、已有工作区改动、工具链/依赖锁、当前检查结果、原型版本/哈希、适用规范及契约 |
| 目标与非目标 | 本轮可验证结果；目录搬迁、行为修复、视觉变化分别列出；平台与授权边界 |
| 差异决定 | 本文待统一项的选定方案、来源与取舍；常规工程选择自行明确，有产品/契约影响的矛盾显式处理 |
| 工作分解 | 阶段、依赖顺序、职责、文件范围、输入与产物，关联 IMP/TASK |
| 组件与模块迁移 | 旧入口→新入口、公共 API、兼容期、删除条件；错误/性能修复分别标识 |
| 验收追踪 | 需求/AC/FZ→操作入口→流程→用例→证据；不能遗漏冻结能力 |
| 测试方案 | 各阶段命令与 cwd、依赖安装、数据集、故障注入、真实桌面与原型复核方法 |
| 性能方案 | 基线测量方式、目标、容许波动、受影响场景与回归 |
| 环境与风险 | 临时目录、测试账户/远端、原生权限、缺失工具与替代验证边界 |
| 回滚与恢复 | 每阶段代码回退方式；测试产生的数据如何清理；不覆盖已有用户改动 |
| 完成定义 | 必须通过的检查、零遗漏入口清单、未通过的处理方式、文档回写和交付物 |

所有命令在执行阶段按实际路径核实，不复制过期 `src-ui/src-tauri` 路径当作可执行计划。先记录已有失败，不能通过删除断言、跳过用例、降低规范或扩大 N/A 让本轮显得全绿。

### 16.3 执行顺序与记录

1. 基线与操作/流程清单 → 可执行计划、TASK 与检查要求。
2. IMP-03 路径与命令入口 → 公共组件/Token 试点 → 功能纵向迁移 → 应用与后端边界。
3. 各阶段完成对应错误处理与性能修复，做针对性检查并记录结果。
4. 集成后执行第 17 节全量验收；发现问题修复后重跑受影响用例及必要的整体门禁。
5. 完成原型/规范差异复核、当前源码打包与制品冒烟，再回写现状文档和交付结论。

阶段记录包含实际修改、检查结果、失败原因、下一步与证据位置。仅在新改动、失败或未解决问题需要时重复检查；阶段未过不得标成已完成。

## 17. 全操作入口、端到端流程与原型验收

### 17.1 先盘点再测试，完整性可核对

从需求与能力地图、原型页面、源码/路由和实际运行界面四方建立操作入口清单，覆盖所有页面及状态下可达的：

- 普通/图标按钮、卡片点击区、菜单项、Tabs、分页、搜索/筛选、开关、Select、范围滑块和数字输入。
- 弹窗确认/取消/关闭/遮罩/Escape、文件选择、拖放、复制、外链和打开目录。
- 通知与任务卡动作、托盘与原生菜单、快捷键、CLI 对应操作。
- 仅在错误、未安装、空数据、进行中、完成等状态出现的条件操作；不能只抓取初始 DOM。

每个入口至少具有稳定编号和以下字段：

```text
编号 | 页面/状态 | 名称与定位 | 需求/契约来源 | 前置数据
操作步骤 | 预期界面 | 预期后端/持久化副作用 | 失败/取消/禁用分支
关联流程/用例 | 验证层级与环境 | 实际结果 | 证据 | 缺陷与复测
```

重复列表行可按行为、权限与数据等价类覆盖，不要求每个相同记录都重复点一次；不同页面绑定、不同目标客户端适配和不同操作语义不能合并遗漏。

“全部按钮通过”必须来自已核对总数的清单。每项最终状态为通过、失败、阻塞、未执行或有理由的不适用；分类互斥，总数可对账。禁用按钮也要验证禁用条件、可理解原因及无法触发副作用，不算免测。

### 17.2 每个操作的通过标准

- 能正确到达入口，鼠标/适用键盘动作触发预期行为；不是只断言 click handler 调用过。
- 业务输入、命令参数和结果正确；保存类动作回读验证，必要时重启复核。
- 成功、失败、取消、进行中与重复点击按契约处理；对操作不适用的分支说明原因。
- 检查真实结果：文件/链接、配置、进程/端口、同步远端、通知持久化等；不能仅凭 Toast “成功”判断。
- 失败后不显示旧成功、不无限 loading、不吞错、不意外丢失输入或重复写入。
- 纯导航、帮助、复制等无后端写入的入口按对应结果验证，不制造无意义文件断言。

### 17.3 必须串联的完整流程

| 流程 | 至少覆盖的链路 |
|---|---|
| 启动与运行 | 首次启动/显式 hash → 环境检测 → 来源选择/安装 → 启动 → 就绪 → 面板 → 停止/重启 |
| 安装管理 | 在线/离线现有入口 → 校验 → 参数/确认 → 进度 → 成功回读；失败/取消；卸载范围与确认 |
| 偏好与目录 | 输入 → 校验 → 保存 → 回读 → 切页/重启保持；无权限/坏路径/取消；数据根引用与隔离迁移 |
| 扩展 | 搜索/筛选/分页 → 详情 → 各客户端启用/禁用 → 实际文件/链接验证；冲突/来源消失/只读 |
| 备份与迁移 | 生成备份/导出 → 内容校验 → 隔离导入 → 状态回读；旧口令、坏包、部分失败与回滚 |
| 同步 | 配置测试端点 → 认证 → 同步 → 双端结果 → 冲突处理/重复无变化；断网/超时/取消 |
| 日志与诊断 | 实际动作 → 日志分类/Doctor → 失败原因 → 通知动作 → 已读/解决/清理 → 重启回读 |
| 主题与缩放 | 亮/暗/系统 → 50/100/200% → 所有页面/弹层/面板 → 保存/重启；失败回退 |
| 桌面生命周期 | 关闭隐藏 → 托盘恢复 → 打开指定页面 → 原生菜单 → 退出；与 GUI/CLI 状态一致 |
| 更新与恢复引导 | 检查 → 有/无更新及失败 → 对应操作；恢复先备份再引导，不能越权代执行官方恢复 |
| 关于及辅助动作 | 本地文档、外链、目录打开、复制与帮助入口 → 正确目标和取消/失败反馈 |

本表是最低流程集合，清单以实际全部能力补齐；不得因为未列入表中而忽略现有功能。

### 17.4 原型 → 规范 → 实现复核

原型入口为：

`docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/原型/index.html`

执行时记录真实版本/哈希，并结合已确认的后续 UI 修订。原型用于视觉与布局判断，业务行为以需求、AC/FZ 和现行契约为准；原型演示数据/按钮不自动成为生产能力。

复核步骤：

1. 建立“原型区域 → 现行规范条款/后续确认 → 实现组件 → 差异 → 处理结果”的对照。
2. 用相同窗口、主题、缩放、测试数据和界面状态并排截图，检查所有页面及主要弹窗、条件状态。
3. 比较布局层级、字体/字重/行高、颜色/材质、间距、圆角、图标、滚动、长内容、焦点与操作区；同时测量关键几何尺寸。
4. 原型早于后续确认时记录合理差异，不将已确认改动错误回退；来源矛盾记录待决定，不默认“原型永远正确”。
5. 截图差异结合人工/视觉检查判断，字体抗锯齿等环境噪声可解释，不能自动接受所有变化或只要求整图像素完全相等。
6. CSS zoom、原生拖动、子 WebView 避让等回到实际桌面验证，浏览器截图不能证明原生行为。

验收截图必须有状态和构建身份，不使用历史截图冒充本轮结果。原型 QA 脚本如仍适用，修正路径后复跑；脚本局限必须记录。

### 17.5 分层执行与故障注入

| 层级 | 证明内容 | 不能替代 |
|---|---|---|
| 单元/组件 | 规则、格式、交互、状态与竞态 | 真实 IPC、文件和原生表现 |
| 服务集成 | 真实 Rust、临时文件、受控进程/本地服务的真实副作用 | 所有原生窗口与最终制品 |
| 浏览器功能/视觉 | 完整页面绑定、流程编排、原型/规范布局差异 | Tauri 能力授权和子 WebView |
| Tauri 桌面 | 真实命令桥、文件对话框、托盘、生命周期、主题与缩放 | 当前最终包是否包含最新变更 |
| 打包制品 | 当前源码实际 .app/.dmg 中的关键完整流程与启动/重启 | 未支持平台、未执行外部发布 |

故障注入包括权限不足、锁冲突、无依赖、未知错误、断网、超时、重复点击、旧响应晚返回、写入中断和回滚失败；根据实际操作选择相关分支。

使用隔离目录、测试服务和可恢复环境；安装/卸载、恢复、同步等不在真实用户配置上盲测。密钥、原生权限或外部测试环境缺失时明确阻塞，继续独立工作；不得改成纯 mock 后仍标“真实链路通过”。

可用工具支持时自动化；没有稳定原生自动化能力时采用有步骤、有结果、有证据的真机复核，并如实区分自动化与人工。不要承诺当前工具无法实际执行的点击或截图。

### 17.6 缺陷、复测和交付门槛

每个问题记录：来源入口/流程、复现环境与步骤、预期/实际、影响、证据、根因、修复位置、关联回归和复测结果。优先修复数据损坏、假成功、无法完成核心流程、启动挂死、泄漏与明显性能退化。

本轮范围内发现的功能、样式、错误处理与性能问题持续修复到通过。范围外问题明确登记；不能自行将必需验收降级成“已知问题”后宣布完成。

完成需要同时满足：

- 全部实际操作入口盘点完毕，无未登记入口；必需用例无失败、阻塞或未执行。
- 全流程正常与适用异常分支通过；契约、真实副作用与持久化结果一致。
- 原型/现行规范差异逐项解释或修复，样式数值和组件规则检查通过。
- 性能修复达到事先记录的目标，关键流程无未解释回归。
- 前后端静态检查、测试、构建和当前源码真实打包通过。
- 当前制品启动并完成关键流程冒烟；提交/工作区差异、构建环境、产物路径和哈希可追踪。
- 核心文档、实施结论和执行回执对齐；验收矩阵与缺陷清单有证据索引。

运行中必须验收的项目存在环境阻塞时，不宣称全部完成；报告已完成内容、缺失条件和继续入口。最终交付简洁说明改动、如何验证、性能前后结果、制品位置与实际支持平台。

## 18. 目标模式执行约定与提示词

本节供下一次明确执行时引用，本次只写文档，不创建执行目标。

启用目标模式后，以 IMP-03/04 和本次已认可方向为输入，先完善第 16 节规定的计划，再持续实施、修复、验证、复核与回写，直到第 17.6 节满足。创建目标时不擅自设定 token 预算。

授权范围内的常规命名、拆分和可逆修复自行推进，不因每个阶段反复索取确认。遇到真实产品决定冲突或缺少外部条件时明确处理；高风险用户数据迁移、发布、破坏性恢复、全局同步及 GitHub 推送等仍按既有授权边界，不从本提示词推导额外许可。

最简执行提示词：

> 开启目标模式：按 IMP-03/04 和项目规范先完善实施计划，再持续完成重构、错误与性能修复、全按钮/全流程测试、原型复核及打包验收，修复到通过并回写文档；必需验收未通过不算完成。

## 19. 可执行实施计划（目标模式）

- 建立日期：2026-09-27；目标模式下按第 16.2 节必填项在本节补齐；IMP-03 仍为目录迁移事实入口，本节不复制其范围。
- 状态：计划已细化、开始执行；执行记录、检查点、回执与证书在 `.adg/` 下的本轮工作目录维护。
- 适用：执行时必须按实际路径核实命令；本节命令在基线（19.1）上核对通过后才作为阶段入口。

### 19.1 基线与依据

| 项 | 值 |
|---|---|
| 提交 | `bdc4c2d20ee3a27dc4da7730951b2fb4a613f751`（分支 `codex/implementation-base`） |
| 工作区 | `docs/03-开发实施/README.md`（改动）、`docs/03-开发实施/IMP-OPENCODEX-DESKTOP-04.md`（未跟踪，本轮计划） |
| 系统 / 架构 | macOS 27.0 / arm64，目标 `aarch64-apple-darwin` |
| 工具链 | Node v26.8.1、npm 11.19.0、rustc 1.95.0、cargo 1.95.0、@tauri-apps/cli 2.11.4 |
| 依赖锁 | `src-tauri/Cargo.lock` = `e43b410a919c4f2d3d988efb…`；`src-ui/package-lock.json` = `24f9a5163c2377e087db8b3b…` |
| 原型入口 | `…/05-原型/原型/index.html`，sha256 前 24 位 `d5888597c0603918f14b9ea9` |
| 现行事实 | DMD（Revision 10）、IMP-01（原实施契约）、IMP-02（质量整改，A–E）、IMP-03（目录迁移）、`docs/02-项目核心/**` |
| 基线门禁 | 三方审计（`.adg/work/three-way-audit/summary.md`）在 `18310235` 上实测：前端 typecheck/vitest(259)/build、后端 fmt/clippy/test(517) 通过；原型 jsdom(315)/浏览器(415) 通过；`tauri build` 出 `.app`+`.dmg` |

### 19.2 目标与非目标

目标（可验证结果）：

1. 目录结构按 IMP-03 迁移到 `apps/desktop/{ui,tauri}`，除历史文本外无 `src-ui` / `src-tauri` 现行引用；干净检出可安装、检查、测试、真实打包。
2. 按 IMP-04 §4–§9 建立组件体系与工程结构，页面/功能区职责清晰；等价迁移与行为/视觉改动分开标识。
3. 按 §14 收敛错误处理，按 §15 建立性能基线与修复闭环。
4. 按 §17 完成全操作入口盘点、全流程测试、原型复核、当前源码打包与制品冒烟。
5. 回写《UI 规范》《系统架构》《契约字段》《术语与命名》等现行事实，收口 TASK/RCP/CERT。

非目标：

- 不改契约字段与冻结码值（§14.1）；不新增/修改 `AC-*` / `FZ-*`（需产品决定时按 19.3 显式处理）。
- 不新建 `packages/` / `crates/` / 多 crate、不迁移包管理器、不引入空目录。
- 不做发布、签名、公证、非 macOS-arm64 平台；真实用户数据上的迁移/恢复/全局同步不纳入本轮（授权边界）。
- 不迁就原型演示数据/按钮成为生产能力；不把“构件通过”当作“真机全链路通过”。

### 19.3 差异决定（待统一项）

以下为本轮需先选定、再实施的事项；常规工程选择自行明确，有产品/契约影响者已列。在选定前不做对应改动。

| 编号 | 事项 | 本轮选定 | 依据 |
|---|---|---|---|
| D1 | 组件公共目录命名 | 采用 `components/{ui,layout,patterns}`，本版不设顶层 `shared/` | §2、§4.1 |
| D2 | 基础组件前缀 | `Ui*`（Button/Card/…），布局/组合用语义名，`App` 留给应用壳 | §4.2 |
| D3 | 页面 padding（22px 等特例） | 先做**等值** Token 提取（§7.5），视觉取值不在等价迁移中改动 | §7.5、§7.10 |
| D4 | Input 12px vs 控件 13px | 保留 12px 基线，统一方案另列视觉决定后再改 | §7.6、§7.10 |
| D5 | 通知宿主 z=80 / 窗口最小尺寸 900×600 vs Tauri 960×640 | 先登记差异、提取宿主 Token 保留避让规则；改基数须单独视觉决定 | §7.8、§7.10 |
| D6 | 导航/主题归属 | 按 §13.2：`navigation/`（纯类型/解析）+ `composables/useNavigation` + `app/router`；主题缩放归 `app/appearance` | §13.2 A01/A02 |
| D7 | 开发夹具与正式入口 | `setState`/`setEnvironmentFixture` 加 DEV 隔离；`recheckEnvironment` 先迁入正式环境服务 | §13.6 A06 |
| D8 | 保留 hash 路由 | 迁移不引入 Vue Router，保留现有 hash 与显式入口优先级 | §13.2 |

### 19.4 工作分解（阶段、依赖、命令、产物）

顺序：B → C → D → E → F（A 已由本文完成）。每阶段以 19.11 门槛收口，未过不标记完成。

| 阶段 | 内容 | 主要命令 / cwd | 产物 |
|---|---|---|---|
| A 计划与基线 | 本节 + 基线冻结 + TASK/检查点 | —（`.adg/`） | 本文 §19、TASK 与检查点 |
| B 外层目录（IMP-03） | `git mv` 两目录 + 全部现行引用更新 | 见 19.7 | `apps/desktop/{ui,tauri}`；`ui/dist` |
| C 公共组件试点 | 从真实页面抽取 Button/Card/SettingRow/FormField/Stack/Inline/Notice + 预览 | 前端门禁 | 组件族 + component-gallery |
| D 功能纵向迁移 | 先扩展管理，再设置分区与 runtime；API/状态/组件/局部样式/测试同步 | 前端门禁 + 针对性用例 | feature 切片 |
| E 应用与后端边界 | 拆总 store/controller、bootstrap，收敛 GUI/CLI 共用用例 | 前端 + 后端门禁 | app/bootstrap/application |
| F 门禁与回写 | 边界检查、格式、全量验收（§17）、真实打包与冒烟、文档回写 | 全线 + `tauri build` | 通过门禁、制品、回执/证书 |

### 19.5 组件与模块迁移映射

沿用 §13.7 的“旧入口 → 目标归属 → 迁移完成条件”；每功能切片另记“旧入口 → 新入口 → 对应回归 → 删除条件”，允许短期单向兼容导出，迁移后删除旧入口、失效样式与过时测试路径。

### 19.6 验收追踪

从 DMD `AC-01`~`AC-14`、能力地图 `CAP-*` 与 §17.3 流程表建立：`需求/AC/FZ → 操作入口 → 流程 → 用例 → 证据`。入口清单四源（需求/能力地图、原型、源码路由、实际运行界面）合并、编号、去重；禁用项登记禁用条件与“不触发副作用”。冻结能力不得遗漏。

### 19.7 测试方案（命令与 cwd）

阶段 B 完成后路径为根入口；阶段 B 前用现行路径。**同一组根入口**贯穿本地开发、检查与打包。

| 层 | 命令（cwd） |
|---|---|
| 前端类型 | `npm --prefix apps/desktop/ui run typecheck`（根） |
| 前端单测 | `npm --prefix apps/desktop/ui run test -- --run`（根） |
| 前端构建 | `npm --prefix apps/desktop/ui run build`（根） |
| 后端格式/静态 | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`（`apps/desktop/tauri`） |
| 后端测试 | `cargo test --workspace --features integration-test`（`apps/desktop/tauri`） |
| 真实打包 | `npx tauri build --target aarch64-apple-darwin`（根） |
| 原型 jsdom | `node --test …/05-原型/原型/qa/*.test.mjs`（依赖解析路径随 IMP-03 修正） |
| 原型浏览器 | `node …/05-原型/原型/qa/*.browser.mjs`（真实浏览器/渲染层） |

依赖安装：阶段 B 后 CI 各 job 在自己的干净工作区安装（前端 job `npm ci`，打包 job 独立安装 Tauri CLI 与前端依赖），不依赖他 job 结果。

数据集/故障注入：隔离数据根、测试账户与测试远端；覆盖权限不足、锁冲突、无依赖、未知错误、断网、超时、重复点击、晚返回、写入中断与回滚失败（§17.5）。缺少原生权限或外部环境时明确阻塞，不改 mock 后标“真实链路通过”。

### 19.8 性能方案

先按 §15.1 在可比条件下记录基线（机器、系统、构建模式、数据量、冷热缓存、样本数）：冷/热启动到可交互、页面切换、长列表/Markdown 渲染、后台轮询/IPC 频率、包体、重复开关页面/弹层后的资源释放。再定指标目标与允许波动，只对测量确认项优化；优化不得降低状态真实性、遗漏事件、弱化权限/校验、取消备份或回滚。修复项建立可复现旧问题的回归/探针（§15.3）。

### 19.9 环境与风险

- 原生权限（Keychain、文件对话框、托盘/子 WebView）、真实 WebDAV 远端、签名/公证/发布：缺失即阻塞，继续独立工作，不伪造进度。
- Tauri 目录发现（3 层上限）：`apps/desktop/{ui,tauri}` 恰为深度 3；B 阶段先真实 `tauri build` 预检。
- 测试内相对路径漏改（`../../src-tauri/…`）：先 `grep` 兜底再跑门禁。
- 原型 QA 依赖解析失败：B 阶段同步改 `qa/*.mjs` 依赖路径。
- 视觉差异与“规范化”混淆：等值迁移与取值调整分开；不在差异未处理前宣称样式已固定。

### 19.10 回滚与恢复

每阶段独立提交、可 `git revert`；目录迁移在独立提交上完成，任一门槛不过回退基线，不影响数据与契约，不覆盖已有用户改动。测试产生的数据在隔离目录内，用后清理。IMP-03 回滚另见其“风险与回滚”。

### 19.11 完成定义（当前目标）

同时满足 §17.6 全部条目：入口清单无遗漏、必需用例无失败/阻塞/未执行；全流程正常与适用异常分支、契约与真实副作用/持久化一致；原型/规范差异逐项解释或修复且样式/组件规则检查通过；性能达到事先目标且无未解释回归；前后端静态检查/测试/构建/当前源码真实打包通过；当前制品启动并完成关键流程冒烟；提交与产物哈希可追踪；核心文档、实施结论与回执对齐。**运行中必须验收项存在环境阻塞时不宣称完成**，如实报告已完成内容、缺失条件与继续入口。

### 19.12 性能基线与目标（2026-09-27 实测）

§15 要求「先测量、再定目标、修复后同环境复测」。以下为当前源码（前端可测项）的实测基线与方法；启动/真机项需在当前制品上测量（F 阶段执行，此处给出方法与目标）。

环境：macOS 27.0 / arm64；Node v26.8.1、npm 11.19.0；前端 `vite build` 生产构建（未压缩产物与 gzip 并列记录）。测量点：`apps/desktop/ui/dist/`（对应当前源码）。

| 指标 | 基线（2026-09-27） | 目标 / 允许回归 |
|---|---|---|
| 主 JS 包（`index-*.js`） | 312,966 B 原始 / 102,757 B gzip | 不因本轮工程化净增 > 5%；新增依赖需说明理由 |
| 主 CSS（`index-*.css`） | 67,769 B 原始 / 12,483 B gzip | 同上 |
| 其余分块 | `core` 2,900 B、`event` 1,043 B、`sync` 842 B | 保持按需分块，不合并回主包 |
| `dist/` 总量 | 392 KB | 开发态资源（fixtures/gallery/auditHarness）不得进入正式构建 |
| 前端测试套件 | 62 文件 284 例，约 2.4s（本机） | 允许小幅增长；不得删断言换取通过 |

启动/交互（F 阶段在当前制品上测量，同机同数据量，取多次样本的中位数与范围）：冷启动到首帧 / 可交互、路由切换、长列表与 Markdown 渲染、后台空闲前后端 CPU 与 IPC 频率、重复开关页面/弹层后的资源释放。目标：不出现挂起（已有上限兜底，见 A04）、无随重复挂载单调增长的监听/计时器、后台空闲不出现无效轮询刷屏。

§15.2 优先排查项核查结论（本轮）：

1. **启动挂起 / 重复监听 / 晚到响应**：已修（A04 启动等待上限、A03 监听成对移除与日志请求有序、A06 调试全局 DEV 隔离）。回归见 `startup-deadline`/`logs-ordering`/`settings-guide-listener`。
2. **重复查询同一快照**：状态快照为事件驱动（`status-snapshot-changed`）加显式拉取；未见每个卡片各自轮询同一快照；托盘请求以 1s 间隔做轻量 IPC 排空（`drainTrayRequests`），非高频网络轮询。
3. **缩放 / ResizeObserver**：`PanelRoute` 的布局经 `requestAnimationFrame` 合并，单一 `ResizeObserver` 在卸载时释放；缩放保存已去抖并保护末次意图（历史 Q2-03 修复）。
4. **读取/渲染上限**：后端日志读取自带截断（`dropped_old_lines`/`truncated`，见 `modules/logs/mod.rs`）；扩展列表前端分页 `PAGE_SIZE=8`；详情正文由后端截断标记（`markdownTruncated`）。
5. **Rust 长任务阻塞命令响应**：属后端范畴，本轮未测量；作为 F 阶段的真机观察项登记（安装/同步/导入的 UI 可操作性与取消响应）。

说明：包体基线为生产构建产物，不代表冷启动耗时；启动与真机项在 F 阶段用当前源码 `.app` 补测后方可宣称达标。未测量项不视为通过。

**长列表、路由切换与 Markdown 渲染耗时（③ 探针，2026-09-27，TASK-133/136）：** 审计装置加 `scenario=manynotif`（200 条通知）与 `scenario=markdown`（长 Markdown 描述的 Skill）夹具；探针新增三个性能用例（数值写入报告 `performance`，不参与通过判定）。实测（本机 macOS 27 arm64、无头 Chromium、Vite 开发服务器）：**200 条通知列表渲染 228 ms**（另一次 245 ms）、**概览→诊断路由切换 26 ms**（另一次 19 ms）、**详情长文 Markdown 渲染 34 ms / 242 节点**（另一次 24 ms）；探针总数 **193/193 通过、0 未捕获错误**。与 ④ 制品内实测对照：制品内路由切换 < 150 ms、后台空闲 CPU avg 0.33%、重复切页 RSS 132.8→132.7 MB。证据 `evidence/perf-interaction.md`、`evidence/browser/audit-report.json`（`performance` 字段）。**后台空闲 IPC 频率逐条计数（① 层，2026-09-27，TASK-137）**：新增 `tests/idle-ipc.test.ts` 在无操作、无状态迁移的空闲窗口计数前端 IPC——空闲 60 s 恰好 **60 次**（1 次/秒的托盘请求排空），**无其它空闲 IPC**、前后各 30 s 均 30 次（**不随时间增长**，排除泄漏式轮询）；状态/通知/运行来源均为事件驱动（`listen`，非轮询）。据此 §19.12「路由切换、长列表、Markdown 渲染、后台空闲 IPC 频率」四项在前端可测层均已有实测数值（IPC 为确定性计数；制品内无 Tauri IPC 外部计数器，另以空闲 CPU avg 0.33% 作 ④ 等效证据）。

### 19.13 错误处理与日志分层核查（2026-09-27）

按 §14.1–§14.3 核查当前实现的分层职责与「单一反馈负责方」，结论如下（证据为文件与既有回归；未在真机复核的分支登记到 F）：

| 层面 | 结论与证据 |
|---|---|
| 识别（适配层） | 扩展失败集中在 `features/extensions/errors.ts`：`{code,message}` 归一为中文原因并保留原始技术描述；已知形态与**未知载荷**均有回归（`tests/extension-errors.test.ts`，含 `code:99` 未知报文 → 「未归类的失败；请把原始信息一并反馈。」）。不新增普通组件内的英文字符串匹配。 |
| 状态收敛 | store 只保留事实与原因：`executeExtensionWrite` 失败时置 `extensionConfigError` / `extensionWriteError`、成功时清空；返回 `null` 供调用方判定，不把失败包装为成功。 |
| 展示（单一负责方） | 扩展写失败由路由 `runExtensionWrite` **单点** `showToast`（原因取自 `extensionWriteError`，无原因时退回既有说法、不虚构）；store 不重复播报，避免 store/composable/页面各弹一次。 |
| 取消 / 未就绪 / 校验 | 取消按正常结果处理（不报红）；未就绪走 `setting-readonly` 等持久说明 + 前往配置动作；写入校验边界在后端（前端校验不替代）。 |
| 排障与脱敏 | 日志按 start/stop/restart 记录命令/结果/错误码；凭据（env/args/headers/口令/代理 secret）按既有规则掩码，不进入 URL/Toast/持久化 UI 状态。 |

需在 F 阶段真机复核的分支（当前未测量，不视为通过）：安装/同步/导入的**超时且结果未知**提示（须先回读后端结果，不直接重发写操作）、**部分成功/回滚失败**的未完成部分与恢复入口、运行期非预期异常的局部兜底与恢复方式。

### 19.14 F 阶段执行结果（2026-09-27）

按 §17 执行全量验收，产物与证据在 `.adg/work/imp03-04-execution/`（gitignore）。分支 `codex/implementation-base`。

**§17.1 入口清单**：从 `apps/desktop/ui/src` 静态提取 **18 个文件 268 个操作入口**（脚本 `.extract-entries.mjs`、原始 `entry-inventory.raw.json`、清单 `entry-inventory.md`，含等价类归纳）。覆盖 6 路由、全局壳层（侧栏/顶栏/通知中心/Toast/任务卡/弹窗/运行期异常兜底条）、环境门与扩展详情，记录条件禁用与动态绑定。合计与各文件计数一致，可复跑对账（`node .adg/work/imp03-04-execution/.extract-entries.mjs` → `.render-inventory.mjs`）。

**§17.3/§17.5 分层测试**：

| 层级 | 结果 | 证据 |
|---|---|---|
| ① 单元/组件 | 62 文件 **284** 例通过 | `apps/desktop/ui/tests` |
| ② 服务集成 | cargo **517** 例通过（真实临时文件/受控进程）；其中 WebDAV 真实链路原为「无环境变量即跳过」，本轮以本地受控 TLS 服务器**真实跑通 10 例** | `evidence/backend-tests.txt`；`evidence/backend-webdav-real.{md,txt}` |
| ③ 浏览器功能/视觉 | 无头 Chromium **190/190** 通过、0 错误（含交互回读与性能测量） | `apps/desktop/ui/qa/audit.browser.mjs`；`evidence/browser/` |
| ④ Tauri 桌面 | 制品真机：启动/环境发现/状态写回/关闭隐藏/再激活/退出、隔离实例真实 IPC/CLI、原生托盘菜单、WebView 交互（托盘路由回读/内嵌官方面板/原生目录选择器/CSS zoom）、GUI 流程与异常分支（安装向导逐步/同步失败/导入失败）、**安装执行/卸载执行** | `evidence/app-smoke/{smoke,isolated-ipc,tray,webview-interaction,gui-flows,install-uninstall}.md` |
| ⑤ 打包制品 | 当前源码 `.app` + `.dmg` 生成并冒烟 | 同上 |

③ 覆盖：6 路由 + 设置 10 分区结构、主题亮/暗×6 路由、缩放 50/100/200 成功、越界 999→200、空值→100、保存失败回退 50→100、通知中心/Toast/任务卡与空·长文本·超时·失败场景；断言结构存在、主题生效、`--ui-zoom` 生效与读数回写、无横向溢出。修复了审计装置报告早于应用挂载的竞态（避免把空 DOM 误判为通过）。

**§17.4 原型复核**：原型 `index.html` SHA-256 `d5888597…`（未被本轮改动）；复跑 jsdom **315**、浏览器 **415** 通过、0 控制台错误（浏览器探针重渲染的截图已回退）。原型区域→规范→实现的逐项对照见 `prototype-recheck.md`。

**§17.6 门禁**：前端 `vue-tsc --noEmit` 通过、`vitest --run` 284 通过、`vite build` 通过；后端 `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test`（517）通过；`npx @tauri-apps/cli@2.11.4 build --target aarch64-apple-darwin` 出 `.app`（可执行 SHA-256 `e3200278…`）与 `.dmg`（SHA-256 `08952550…`），提交 `2974a8ce`、工作区干净。

**④ 层进展与仍未达成（不宣称完成）**：制品内真机复核已覆盖——启动/环境发现/状态写回/关闭隐藏/再激活/退出（`smoke.md`）、隔离实例真实 IPC/CLI 的备份/导出/导入（含篡改包拒绝）/同步失败（`isolated-ipc.md`）、原生托盘菜单（`tray.md`）、**WebView 交互**：托盘五项菜单目标路由回读、内嵌官方面板子 WebView 运行态渲染与避让、原生目录选择器（打开+取消无副作用）、CSS zoom 200%（2.01×）/还原、重复切页无泄漏、路由切换 <150 ms（`webview-interaction.md`，TASK-123）。
**④ 层 GUI 流程与异常分支**（`gui-flows.md`，TASK-124）：安装向导逐步点击（步骤 1/2、原生目录/文件选择器、未选包时下一步被门禁挡住、取消关闭；**未执行最终安装**）、同步失败分支（端点未配置 →「同步执行失败；本地内容未修改。」）、配置导入失败分支（无容器 →「配置导入失败；当前配置未修改。」）已在制品内取得真机证据。
**④ 层安装与卸载执行**（`install-uninstall.md`，TASK-125）：制品内**联网安装** OpenCodex（真机逐步点击 → 真实 registry 拉包 → `安装完成 · 2.68.0` → 来源切为托管安装并经 `runtime.json` 核实）与**卸载**（仅移除托管包体 → 来源回退自动发现）已取得真机证据；安装/卸载缺口补齐。
**④ 层离线包安装执行**（`install-uninstall.md` §3，TASK-126）：合规离线包 `bitkyc08-opencodex-2.68.0.tgz` 经「导入离线包」路径真机安装完成（预览 SHA 与联网安装一致、子步骤含「校验离线包」），随后卸载回退。离线包安装缺口补齐。
**④ 层隔离实例安装/卸载重测**（`install-uninstall.md` §4，TASK-127）：TASK-125/126 的安装/卸载误在真实环境执行（`activate` 另起真实实例），本轮用 `open --env HOME=<sandbox>` 取得真正隔离实例并重测：无 npm 正确拒绝、空缓存离线导入正确失败、联网安装成功且全部落沙箱、卸载回退；**真实用户 `runtime.json` 全程未被读写**，满足 §17.5「不在真实用户配置上盲测」。
**§17.1 入口状态对账**（TASK-130/131/140）：268 入口 → **通过 265 · 失败 0 · 阻塞 0 · 未执行 0 · 不适用 3**（逐项状态/层级/证据见 `entry-inventory.md`「状态对账」；状态为等价类判定）。此前判为「未执行」的 18 项经复核均有 ①②③ 层级证据，改判通过；唯一无证据的 `SettingsRoute#24`「恢复自动发现」已补 `tests/runtime-source.test.ts` 用例。
**同步成功分支（②）**：`webdav_sync_real`/`webdav_legacy_real` 原为「无环境变量即跳过」；本轮以本地受控 TLS WebDAV 服务器（自签 CA 经 `SSL_CERT_FILE` 注入，不改系统信任库/不写钥匙串）**真实跑通 10 例**，§17.3 流程 6 的成功与异常分支取得真实服务器证据（TASK-132，见 §19.14）。
**仍未在制品内执行（全流程分支级，非入口级）**：④ **运行期异常兜底条原生外观与「重新加载界面」**（制品内无合法触发入口）。**已补齐**：后台空闲 IPC 频率与渲染耗时（TASK-136/137，前端可测层实测）、配置迁移**导出/导入成功分支 GUI**（TASK-139，隔离实例真机）、**制品内 GUI 驱动 WebDAV 同步成功分支**（连接成功 + 上传成功，TASK-143，隔离实例真机：凭据写沙箱钥匙串、TLS 经 `SSL_CERT_FILE` 注入自签 CA、`cold_sync=false`）。这些链路的结构/行为已由 ①/②/③ 层覆盖，但**除上表 ④ 已列项外，唯一剩余的 ④ 交互（异常兜底条原生外观）在制品内的真机通过尚未取得**，故 §17.6「全流程正常与适用异常分支通过」**仍不宣称达成**。继续入口：异常兜底条需制品内合法触发入口（或明确其为设计上不可达的恢复路径）；另见 §19.13 登记项。**（2026-09-28 更新：该入口已由 TASK-145 以「界面诊断」自检提供（受控异常走真实错误边界），并已由 TASK-146 在隔离实例内完成 ④ 层真机复核——外观与「重新加载界面」均通过，见 §19.16；复核同时发现兜底条文案泄漏英文文档链接的新缺陷，列 TASK-147 修复，见 §19.16 末段。）**

**核心文档回写**：`系统架构.md`（前端功能切片结构）、`UI规范.md` §23（公共组件体系与样式护栏）、`契约字段.md` §7（重构不改契约）、`术语与命名.md` §5（工程与验收术语）。

**真机启动性能（2026-09-27，当前制品）：** 冷启动 3 次（每次先 `quit` 等 2s，同机同数据量）实测：`open` → 主窗口出现 534 / 490 / 491 ms；`open` → 后端就绪（`manager-state/runtime.json` 首次写入）287 / 285 / 269 ms。方法与原始输出见 `.adg/work/imp03-04-execution/evidence/app-smoke/smoke.md`。结论：**启动无挂起**，首帧与后端就绪均在亚秒级（本机暖态）；交互态（路由切换、长列表/Markdown 渲染、后台空闲 CPU 与 IPC 频率、重复开关页面/弹层的资源释放）**仍未测量**，不视为达标。

**制品 CLI/IPC 冒烟：** 同一次 cargo 构建的 `ocxd`（`target/aarch64-apple-darwin/release/ocxd`）实测：`--help` 退出 0；`status --json` 在实例离线/CLI 关闭时输出 schema-v1 JSON `error.code=instance_offline`、退出 3（只读降级、不误报成功）；`start` 缺 `--confirm` → `RequireConfirm` 退出 4；未知命令 → `UnknownCommand` 退出 2。证据同上。

**④ 层隔离实例真实 IPC/CLI 端到端（2026-09-27）：** 以沙箱 `HOME` 运行当前制品并启用 CLI 端点，用 `ocxd` 经真实 Unix socket 驱动真实 Rust 后端：`status`（`runtime=not_found`、`connection=unconfigured`、`operation=idle`）、`data-root show`（沙箱路径、`home_mode=inside`）、`backup list`（空）；`backup create --confirm` → 磁盘生成 `backups/2026/09/upgrade/bk_…/{backup-manifest.json,preferences.json}` 且 `backup list` 回读 `restorable:true`；`export --confirm` → 磁盘生成 `OCXDCONF` 容器（`format_version:2`、`document_sha256:62fff694…`）；`import --confirm` → `applied_sections:[preferences]`、sha 与导出一致、导入前自动备份；坏路径与**篡改包**（翻转 1 bit）均 `execution_failed`、未部分应用；`sync run`（未配置端点）与 `start`（无运行时）为明确失败、不误报成功；退出码 0/9/12 符合契约。**全部在沙箱内，真实用户目录未被读写**，验证后删除沙箱。原始输出与磁盘回读见 `.adg/work/imp03-04-execution/evidence/app-smoke/isolated-ipc.md`。据此，§17.3 流程 5（备份与迁移）、6（同步失败分支）、1/2（启动失败分支）与 CLI/IPC 契约在**制品内**取得真实链路证据；纯 GUI 点击类（弹窗/原生目录选择器/托盘菜单项）仍未覆盖。

**③ 层真实点击交互（2026-09-27，TASK-107）：** 探针在结构断言之外增加**点击后回读状态迁移**（§17.2「不是只断言 click handler 调用过」）：设置开关翻转 + 保存反馈、选择器展开/选中回读、设置与诊断分区切换 + hash 同步、主题切换（亮↔暗 `data-theme`）、通知分类过滤数量与 `.n-count` 一致、无后端时通知写动作**不假成功**（未读不清零、已解决不移除）、托盘平台切换（macOS↔Windows）、概览详情折叠、扩展分页签切换。全套 **178/178 通过**、0 未捕获错误（结构 147 + 交互 31）。命令：`npm --prefix apps/desktop/ui run audit:browser`。成功路径类通知写动作仍由 ① 层 `tests/notification-actions.test.ts` / `notification-center.test.ts` 覆盖。

**④ 层托盘原生菜单真机复核（2026-09-27，TASK-108）：** 当前制品的托盘为原生 `NSStatusItem`，菜单经 macOS AX 可枚举（`status menu`）：未运行 · 启动/停止/重启 OpenCodex · 打开主界面/打开扩展面板/打开诊断中心/打开数据目录/运行 Doctor/快速设置 · 退出。实测：菜单状态首行「未运行」与 IPC `status.matrix.runtime=not_found` 一致；点「打开数据目录」→ Finder 窗口 4→5 且前窗口路径 `…/com.gzers.opencodex.desktop/manager-state/`（真实 shell 打开）；点「打开主界面」窗口保持显示；点「退出」→ 进程收敛、无新增崩溃报告。**未点击** 启动/停止/重启（真实 `ocx` 启停属真实环境副作用，按边界不盲测）；打开扩展面板/诊断中心/快速设置/运行 Doctor 无法用 AX 回读 WebView 路由（窗口标题设计隐藏），其路由逻辑由 ① 层 `tray-bridge`/`tray-exit`/`navigation` 覆盖。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/tray.md`。覆盖 §17.3 流程 9（桌面生命周期）与 11（打开目录类辅助动作）。

**④ 层 WebView 交互真机复核（2026-09-27，TASK-123）：** WebView 内容不进入原生 AX 树（窗口子元素仅 `AXGroup` + 3 `AXButton`，无 `AXDocument`/`AXURL`，读取报 -1728），窗口标题设计隐藏，故不以 AX 回读路由；改用**真机点击（`System Events` 点原生托盘菜单 / `CGEvent` 点窗口内控件，坐标取自同次截屏 OCR 包围盒）+ 定点 `screencapture` + `Vision` OCR 回读**的像素证据链。实测：① 托盘「打开主界面→概览」「打开扩展面板→面板（未运行门禁态）」「打开诊断中心→诊断中心」「快速设置→设置」「运行 Doctor→诊断中心/环境诊断」**目标路由逐一回读确认**；② 经概览「启动」置运行（`●运行中 端口 10100 版本 2.50.0`）后进面板，子 WebView **真实渲染官方面板**（回读「opencodex v2.50.0」「本地 opencodex 代理…」及官方面板自身侧栏），且不覆盖桌面壳标题栏（子 WebView 避让），随后「停止」回到 `●存在风险` 并释放端口 10100；③ 原生目录选择器为系统 `NSOpenPanel`（标题「选择 Skills 源目录」、Favorites/Applications 等系统侧栏、New Folder），`Esc` 取消无副作用并提示「未选择目录；已保留当前源目录。」；④ CSS zoom 原生生效：200% 使标题栏品牌文本 303×38→610×59 px（**2.01×**），还原 100%；⑤ 连续 10 次「设置↔概览」切页 RSS 132.8→132.7 MB（无增长）；⑥ 路由切换 <150 ms。原始文本行（含像素坐标）与截图见 `.adg/work/imp03-04-execution/evidence/app-smoke/webview-interaction.md`。**仍未覆盖**：运行期异常兜底条原生外观与「重新加载界面」（无制品内触发入口）、后台空闲 IPC 频率、渲染耗时精确分位。

**真机后台空闲 CPU（2026-09-27，当前制品）：** 就绪后对进程连续采样 12 次（每 2s，共 24s）`ps -o %cpu=` → min 0.0% / max 3.0% / avg 0.33%，**无持续占用、无无效轮询**。仍未测量：路由切换耗时、长列表/Markdown 渲染耗时、重复开关页面/弹层的资源释放、后台空闲 IPC 频率（制品内无桌面自动化回读 WebView）。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/smoke.md`。

**§17.4 关键几何尺寸并排测量（2026-09-27，TASK-110）：** 新增 `.adg/work/imp03-04-execution/geometry-compare.mjs`，无头 Chromium 在「宽栏/窄栏（面板）× 暗/亮」四组下并排测量原型与实现，逐项一致：侧栏 `--side-w` 244px/64px、`.main` 左上圆角 12px、左/顶描边 1px、`.main` 底色 `rgba(38,38,38,.4)`（暗）/`rgba(255,255,255,.4)`（亮）、`.titlebar` 高 28px、`--glass-rail` `#1717179e`/`#f9f9f9a8`、`--glass-blur` `saturate(1.6) blur(22px)`、`--accent` `#ececec`/`#0d0d0d`。命名差异已解释：原型 `--page-fill-{glass,mid,clear}` 三档按 UI规范 §21 收敛为单一 `--page-fill`，解析后的 `.main` 背景色完全相同。原始数据 `evidence/geometry-compare.json`，对照表 `prototype-recheck.md`。

**E 阶段切片一：通知域拆出总 store（2026-09-27，TASK-111）：** 新增 `apps/desktop/ui/src/features/notifications/store.ts`（面板开合、通知列表、加载/排队/错误、`notifications-changed` 事件流、读/全部读/删/清/清已读/标记解决/清已解决/按保留清理）；根 `stores/app.ts` 以**只读 getter + 动作委托**保留 `app.notifications*` 调用面，写入点（`App.vue`、`AppTopbar`、`dev/auditHarness.ts`、相关测试）改用通知 store。`stores/app.ts` 1492→1402 行。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。后续切片继续把 runtime、设置、扩展等域从总 store 迁出。

**E 阶段切片二：扩展域拆出总 store（2026-09-27，TASK-112）：** 新增 `apps/desktop/ui/src/features/extensions/store.ts`（扩展发现结果/加载/错误、扩展配置/加载/错误、写入错误与忙标志，动作：加载、写入、客户端开关、源目录、分发方式、对齐 Skills）；根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.extensions*` 调用面，写入点（`extension-detail`/`extensions-pagination` 测试）改用扩展 store。`stores/app.ts` 1402→1349 行（累计 1492→1349）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。

**E 阶段切片三：数据目录域拆出总 store（2026-09-27，TASK-113）：** 新增 `apps/desktop/ui/src/features/data-root/store.ts`（路径/配置/加载/错误、切换/保存/初始化状态、上次校验与初始化结果；动作：保存、加载配置、切换、保存 `OPENCODEX_HOME`）；根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.dataRoot*` 调用面，写入点（`open-data` 测试）改用数据目录 store。`stores/app.ts` 1349→1263 行（累计 1492→1263）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。

**E 阶段切片四/五：关于、诊断与同步域拆出总 store（2026-09-27，TASK-114）：** 新增 `features/about/store.ts`（关于元数据）、`features/diagnostics/store.ts`（Doctor 报告）、`features/sync/store.ts`（端点配置/连接/同步的 IPC 与自身状态）。同步域刻意**不让功能 store 反向依赖根 store**：`webdavState`、Toast、通知刷新仍由根壳动作编排，功能 store 只返回结果，避免循环依赖。根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.*` 调用面，写入点（`sync` 测试）改用同步 store。`stores/app.ts` 1263→1191 行（累计 1492→1191）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。

**E 阶段切片六：偏好域拆出总 store（2026-09-27，TASK-115）：** 新增 `features/preferences/store.ts`（偏好数据/加载/错误 + 界面缩放应用；动作：加载、保存、设置缩放、还原默认）。根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.preferences*` 调用面；审计装置的本地偏好适配改为覆盖偏好 store；写入点（`startup-preferences` 测试）改用偏好 store。`stores/app.ts` 1191→1152 行（累计 1492→1152）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿（含 `prefs=local` 缩放探针路径）。

**E 阶段切片七/八：配置迁移域与应用级模态拆出（2026-09-27，TASK-116）：** 新增 `features/migration/store.ts`（导出/导入状态与结果，导入成功后经偏好 store 重载偏好）与 `app/modal/store.ts`（模态框状态与 `open`/`openInput`/`resolve`）。根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.*` 调用面，写入点（`overlay-surfaces` 测试）改用模态 store。`stores/app.ts` 1152→1107 行（累计 1492→1107）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。

**E 阶段切片九：受管路径投影拆出（2026-09-27，TASK-117）：** 新增 `app/paths/store.ts`（数据目录分区与各客户端 Agent 目录的只读路径投影；打开目录/外链/本地文档转发平台命令）。根 `stores/app.ts` 以只读 getter + 动作委托保留 `app.managedPathTargets`/`agentPathTargets` 与 `open*` 调用面；`openManagedPathByKey` 与 `openOverview*` 仍由根壳编排（含 Toast）。`stores/app.ts` 1107→1076 行（累计 1492→1076）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。

**E 阶段切片十：更新/升级/恢复域拆出（2026-09-27，TASK-118）：** 新增 `features/updates/store.ts`（官方版本事实、升级前备份、restore 风险摘要与备份、应用自更新的 IPC 与自身状态）。根 `stores/app.ts` 保留 `loadOfficialProject`/`refreshRuntimeVersionFact` 的跨域编排（读版本后重读来源 `FZ-48`）与 Toast 文案（升级/restore 备份、更新安装），并保留只读 getter + 动作委托。`stores/app.ts` 1076→1014 行（累计 1492→1014）。门禁：typecheck / vitest 284 / build / 浏览器探针 178 全绿。（执行中一次错误边界编辑已用 `git checkout` 回退并重做，动作集合经 diff 校验与改动前一致。）

**§14.3 运行期异常兜底（2026-09-27，TASK-119）：** 新增 `apps/desktop/ui/src/app/errors.ts`：`sanitizeFaultText`（遮蔽带凭据 URL 与 ≥32 位令牌，截断 300 字）、`describeFault`（Error/字符串/对象/循环引用均不抛）、`installErrorHandlers`（Vue `config.errorHandler` + `window.unhandledrejection`，幂等：重复安装先移除旧监听）。**责任位置** `useAppStore().reportUiFault`（写 `uiFault` 并登记最近事件）；**恢复方式** 界面故障条（`role=alert`）「关闭提示」/「重新加载界面」（`App.vue` + `styles/base.css`）。`main.ts` 启动步骤加 `catch`（异常也挂载应用以显示恢复入口，不再只 log）。捕获**不吞异常**：故障事实进入最近事件与界面提示。新增 `tests/error-boundary.test.ts`（4 例）与浏览器探针用例（故障条出现、`role=alert`、含原因、可关闭 → 全套 185/185）。**包体**：主 JS 312.97→319.93 kB（gzip 102.76→106.20 kB，+2.2%，在 §19.12 允许的 ≤5% 内）。

**E 阶段：拆分 useAppController（2026-09-27，TASK-121）：** 把 `composables/useAppController.ts` 按功能拆出 `features/diagnostics/useDiagnosticsController.ts`（日志分类读取 + 晚到响应保护 + Doctor 只读摘要）与 `app/useLifecycleController.ts`（运行状态设置、启停动作、托盘请求分派、恢复前确认与官方恢复引导）；`useAppController` 变为组合根（408→189 行），**对外暴露字段与拆分前完全一致**（消费方零改动），行为等价。门禁：typecheck / vitest 63 文件 288 例 / build / 浏览器探针 185 全绿。

**E 阶段切片十一：环境发现域拆出总 store（2026-09-27，TASK-122）：** 新增 `apps/desktop/ui/src/features/environment/store.ts`（环境报告与其加载态；动作：`set`/`setLoading`/`refresh`，`refresh` 返回成功与否，Toast 文案仍由根壳负责）。根 `stores/app.ts` 以**只读 getter**（`environment`/`environmentLoading`）+ **动作委托**（`setEnvironment`/`refreshEnvironment`/`setEnvironmentLoading`）保留 `app.environment*` 调用面，写入点（`environment-gate` 测试）改用环境 store。`stores/app.ts` 1024→1019 行（累计 1492→1019）。门禁：typecheck / vitest 63 文件 288 例 / build / 浏览器探针 185 全绿。

**④ 层 GUI 流程与异常分支真机复核（2026-09-27，TASK-124）：** 方法同前（真机点击 + 定点截屏 + OCR 回读）。实测：① **安装向导逐步点击**——设置→安装配置→「安装 OpenCodex」打开向导，步骤 1 显示安装位置（默认落数据根内）与 选择…/恢复默认；点「选择…」弹**原生目录选择器**（标题「选择 OpenCodex 托管安装位置」、系统侧栏齐全），`Esc` 取消无副作用；「下一步」→ 步骤 2 显示「联网安装/导入离线包」单选与代理项；选「导入离线包」出现拖放区与「选择文件…」→ 弹**原生文件选择器**（标题「选择 OpenCodex 离线包」）；`Esc` 取消后未选包时「下一步」**被门禁挡住**（不进入步骤 3）；「取消」关闭向导，**未执行最终安装**。② **同步失败分支**（端点未配置）：「立即同步」→「同步执行失败；本地内容未修改。」。③ **配置导入失败分支**（数据根导出目录无容器）：「导入配置」→确认弹窗→「配置导入失败；当前配置未修改。」（GUI 导入读取数据根固定位置容器，不弹文件选择器；成功导入与篡改拒绝已由 `isolated-ipc.md` 的 CLI/IPC 覆盖）。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/gui-flows.md`。**仍未覆盖**：安装向导最终执行安装/卸载执行（真实联网/写盘副作用）、同步与导入的成功分支 GUI 复核、运行期异常兜底条原生复现。

**④ 层安装与卸载执行真机复核（2026-09-27，TASK-125）：** 制品内真机执行安装向导全流程（设置→安装配置→安装 OpenCodex→步骤 1/2/3→执行）：步骤 4 显示「正在从 registry 取包 · 9%」并出现真实 npm 日志（`npm http fetch GET 200 https://registry.npmjs.org/@bitkyc08%2fopencodex`），随后「**安装完成 · 2.68.0 · 联网安装 · 100%**」（入口 + `SHA-256 48ac7105…`）、Toast「OpenCodex 2.68.0 已就绪」，运行来源切为「**托管安装**」且真实数据根 `manager-state/runtime.json` 落盘为 `source=managed / 2.68.0`（`history` 追加 `install/succeeded`）——安装事实经后端核实、不凭命令返回即报完成。再执行「卸载…」（范围「仅移除托管包体」，保留 `~/.codex/config.toml`/`OPENCODEX_HOME`/Skills 源/备份）→「**已移除托管包体与入口，运行来源回退为「自动发现」**」、`runtime.json` 回到 `discovered / ocx 2.50.0`。**隔离说明**：本轮原意以 `HOME` 沙箱隔离数据根，实测 `HOME` 覆盖**未隔离数据根**（安装落点仍为真实 `~/Library/Application Support/com.gzers.opencodex.desktop/runtime`），故为**真实用户环境**执行，结束时已回退（残留：`runtime.json.history` 两条 append-only 审计记录、数据根空 `runtime/bin/`）。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/install-uninstall.md`。**仍未覆盖**：离线包安装执行（本轮无合规 `*.tgz`）、「同时执行官方卸载（破坏性）」分支。

**④ 层离线包安装执行真机复核（2026-09-27，TASK-126）：** 以 `npm pack @bitkyc08/opencodex@2.68.0` 取得合规离线包 `bitkyc08-opencodex-2.68.0.tgz`（12.4 MB，文件名匹配 `bitkyc08-opencodex-<semver>.tgz`），在制品内走「导入离线包」路径：步骤 2 原生文件选择器选包 → 预览成功（`SHA-256 48ac7105f446b768…`，与联网安装同包）→ 步骤 3 勾选确认 → 步骤 4 显示「校验离线包」等子步骤并最终「**安装完成 · 2.68.0 · 离线导入 · 100%**」；同步执行「卸载…（仅移除托管包体）」→ 来源回退自动发现、`runtime.json` 回 `discovered / 2.50.0`。**离线包安装路径在制品内真机跑通**；`HOME` 覆盖同样未隔离数据根，故仍在真实环境执行并已回退。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/install-uninstall.md` §3。**仍未覆盖**：「同时执行官方卸载（破坏性）」分支。

**④ 层隔离实例安装/卸载重测（2026-09-27，TASK-127）：** 纠正 TASK-125/126 的环境偏差并满足 §17.5「安装/卸载…不在真实用户配置上盲测」。**隔离方式**：`open --env HOME=/tmp/iso_test3 -a "<…>/OpenCodeX Desktop.app"`（此前「直接启动二进制 + `osascript activate`」会让 LaunchServices 另起**真实环境**实例，点击落到真实实例——即 TASK-125/126 命中真实数据根的原因）；自检：概览「数据目录」= 沙箱路径、仅 1 进程、`app.lock` 指向沙箱。沙箱内置 `$HOME/.local/bin/{node,npm}` 符号链接（`ocx` 不建，触发安装门禁）。**结果**：① 无 npm → 联网安装 `npm_missing` 正确拒绝；② 空缓存离线导入 → `npm_failed`（`--offline` 只用缓存、绝不联网，符合 `FZ-49`）；③ 联网安装（有 npm + 可用缓存）→ `安装完成 · 2.68.0 · 联网安装 · 100%`、入口 `SHA-256 48ac7105…`、Toast「已就绪」、来源切「托管安装」，**全部落在沙箱数据根**；④ 卸载 → 「已移除托管包体与入口，运行来源回退为「未解析」」。沙箱 `runtime.json`：`managed → unresolved`，`path` 全程在沙箱内；**真实 `runtime.json` 全程未被读写**（复核后 `discovered / 2.50.0`，history 条数不变）。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/install-uninstall.md` §4（4 张截图）。

**E 阶段切片十二：运行来源/安装/卸载域拆出总 store（2026-09-27，TASK-128）：** 新增 `apps/desktop/ui/src/features/runtime/store.ts`（运行来源事实 `source/sourceLoading/sourceError`、离线包预览、安装自身状态 `install/installLines/installOutcome/installError`、卸载状态 `uninstallResult/uninstallError/officialUninstallObservation`、`installModalRequest`、来源事件流与**在途去重**；动作：`loadSource`/`setSourcePath`/`restoreDiscovered`/`previewOffline`/`clearOfflinePreview`/`cancelInstall`/`loadOfficialUninstallObservation`/`requestInstallModal`/`consumeInstallModalRequest`/`startSourceEventStream` 及细粒度状态落位）与 `features/runtime/errors.ts`（`runtimeErrorText` 随切片迁出，根 store 兼容再导出）。**跨域编排留在根壳**：`installRuntime`/`uninstallRuntime`/`setRuntimeSourcePath`/`restoreDiscoveredRuntime` 仍由根 store 负责「读版本事实 / 刷新状态快照 / Toast」（避免功能 store 反向依赖根 store），后端调用与自身状态落位下沉到功能 store。根 `stores/app.ts` 以**只读 getter + 动作委托**保留 `app.runtimeSource*`/`app.runtimeInstall*`/`app.offlinePreview*`/`app.runtimeUninstall*`/`app.officialUninstallObservation`/`app.installModalRequest` 调用面；写入点（`SettingsRoute` 的卸载重置）改用 runtime store。**动作集合逐一对应（91→91，`comm` 校验无增删）**；`stores/app.ts` 1019→911 行（累计 1492→911）。门禁：typecheck / vitest 63 文件 288 例 / build / 浏览器探针 185 全绿。

**E 阶段切片十三：官方面板嵌入域拆出总 store（2026-09-27，TASK-129）：** 新增 `apps/desktop/ui/src/features/panel/store.ts`（面板 `url/loading/error/loaded` 与 `loadUrl(snapshot)` —— 由根壳传入运行状态快照推导面板地址，**不反向依赖根 store**；`setLoaded`）。根 `stores/app.ts` 以**只读 getter**（`panelUrl/panelLoading/panelError/panelLoaded`）+ 动作委托（`loadPanelUrl`/`openPanelInBrowser`）保留 `app.panel*` 调用面；写入点（`PanelRoute` 的 `panelLoaded` 置位）改用 panel store。动作集合 `comm` 校验无增删；`stores/app.ts` 911→889 行（累计 1492→889）。门禁：typecheck / vitest 63 文件 288 例 / build / 浏览器探针 185 全绿。

**§17.1 入口清单状态对账（2026-09-27，TASK-130）：** 扩展 `.render-inventory.mjs`，在既有 266 入口索引上输出**每入口「状态 / 验证层级 / 证据」**列与等价类状态表、状态对账汇总（可复跑：`node .adg/work/imp03-04-execution/.render-inventory.mjs`）。对账结果：**通过 245 · 失败 0 · 阻塞 0 · 未执行 18 · 不适用 3（合计 266）**。未执行 18 项均为 ④ 制品内原生或外部环境相关并逐项给出理由：`PanelRoute#1-2`（原生子 WebView 内「重试/在浏览器打开」未点击）、`ExtensionsRoute#14,#28`（客户端启用/禁用真实文件落点未在制品内验证）、`SettingsRoute#23,24,25,109,110,111,112,113,148,153,154,155,156,160`（更换运行来源/恢复自动发现/官方更新引导/自更新检查与安装/允许安装脚本/破坏性官方卸载）。不适用 3 项为 `GalleryApp#1-3`（DEV-only 预览，正式构建不含）。判定依据按等价类落表中「证据」列（①组件/单元、②服务集成、③浏览器探针、④制品真机，见 §17.5）。已通过但仍有未覆盖分支者（同步成功分支 GUI、异常兜底条原生复现、后台 IPC 频率/渲染分位）保留在 §17.6 未覆盖项。

**§17.1 入口清单对账修正 + 补用例（2026-09-27，TASK-131）：** 复核 TASK-130 判为「未执行」的 18 项后修正：`PanelRoute#1-2`（`tests/panel.test.ts` 覆盖「重试」「在浏览器打开」点击及不假成功分支）、`ExtensionsRoute#14,#28`（`tests/extensions.test.ts` 覆盖客户端启用/禁用成功·失败，写入真实副作用见 ②）均有层级证据，改判**通过**；`SettingsRoute#23`（`setRuntimeSourcePath` ①）、`#25`（路由跳转 ①③）、`#109/#110`（弹窗机制 ① + 升级引导自动备份回归）、`#111/#112`（`update.test.ts` 检查/安装更新，含失败保留版本）、`#113`（重装入口）、`#148`（② 安装脚本二次确认流程）、`#153-156/#160`（`runtime-source.test.ts`「官方卸载只记录不删除」+ ② 后端 `official_uninstall_*`）同样改判**通过**；唯一无任何层级证据的 `SettingsRoute#24`「恢复自动发现」**补 `tests/runtime-source.test.ts` 成功/失败两条用例**后转为通过。对账：**通过 263 · 失败 0 · 阻塞 0 · 未执行 0 · 不适用 3（合计 266）**（`entry-inventory.md`「状态对账」；状态为等价类判定，§17.1 允许同类绑定归并）。门禁：typecheck / vitest 63 文件 **290** 例 / build / 浏览器探针 185 全绿。

**② 真实 TLS WebDAV 同步双端链路（2026-09-27，TASK-132）：** 发现 `tests/webdav_sync_real.rs` / `webdav_legacy_real.rs` 在**未提供** `OCX_TEST_WEBDAV_*` 时只打印跳过并返回——即早先「后端 517 全绿」里这几条**实际未验证**。本轮以**本地受控** TLS WebDAV 服务器（`wsgidav`+`cheroot`，`https://127.0.0.1:8443/`，Basic `davuser/dav-pass-123`，自签 CA 经 **`SSL_CERT_FILE`** 注入 rustls-native-certs，**不改系统信任库、不写用户钥匙串**）转为真实跑通：`webdav_sync_real` 4/4（**双端同步成功**、并发端点锁、载荷缺失不改本地、错误口令拒绝）、`webdav_legacy_real` 1/1（旧版加密远端按原口令读取）、`sync_plaintext` 3/3（明文+SHA-256、篡改拒绝、旧传输包口令）、`webdav_tls` 2/2（TLS 后端编译期与运行期均在），共 **10 例全过**。据此 §17.3 流程 6（同步）的**成功分支**与认证失败/并发/载荷缺失/篡改等异常分支在 ② 取得**真实服务器 + 真实 TLS** 证据。原始输出 `.adg/work/imp03-04-execution/evidence/backend-webdav-real.txt`，说明 `evidence/backend-webdav-real.md`。**仍未覆盖**：④ 制品内 GUI 驱动同步（保存端点会写用户登录钥匙串 `ocx.dav.cred_*`，属真实凭据副作用，本轮未写）。

**§19.12 性能补测：长列表与路由切换（2026-09-27，TASK-133）：** 审计装置新增 `scenario=manynotif`（200 条通知）夹具；浏览器探针新增性能用例（数值写入报告 `performance` 字段，通过/失败判定仍用结构断言）。实测 **200 条通知渲染 228 ms / 245 ms**、**路由切换 26 ms / 19 ms**（③ 层开发服务器；制品内路由切换另测 < 150 ms）。探针 **190/190 通过、0 未捕获错误**。证据 `evidence/perf-interaction.md`。据此刻度，§19.12「长列表渲染」一项由未测量转为**已测**；仅剩「后台空闲 IPC 频率逐条计数」与「Markdown 渲染耗时」未测量（前者无制品内计数器，等效证据见 §19.12）。

**E 阶段切片十四：界面反馈域拆出总 store（2026-09-27，TASK-134）：** 新增 `apps/desktop/ui/src/app/feedback/store.ts`（`toast`/`toastRevision`/`uiFault`/`recentEvents` 与 `showToast`/`clearToast`/`recordEvent`/`reportUiFault`/`clearUiFault`）。方案 §19.5 要求「Toast、弹窗队列与后台持久通知分别管理，不混为同一个总 store」：弹窗队列在 `app/modal/store.ts`、后台通知在 `features/notifications/store.ts`，本轮把 **Toast 与运行期异常故障条**归入界面反馈域（`app feedback`，见 §19.4 目标结构）。本 store **不依赖其它 app store**，供根壳与各功能调用，避免循环依赖。根 `stores/app.ts` 以**只读 getter**（`toast`/`uiFault`/`recentEvents`）+ **动作委托**（`showToast`/`clearToast`/`recordEvent`/`reportUiFault`/`clearUiFault`）保留 `app.*` 调用面；写入点（`dev/auditHarness.ts`、`tests/panel.test.ts`、`tests/startup-preferences.test.ts`）改用界面反馈 store。`stores/app.ts` 889→886 行（累计 1492→886）。门禁：typecheck / vitest 63 文件 **290** 例 / build / 浏览器探针 **190/190** 全绿。

**E 阶段切片十五：运行生命周期与状态域拆出总 store（2026-09-27，TASK-135）：** 新增 `apps/desktop/ui/src/app/lifecycle/store.ts`（290 行）。收敛「桌面与后端边界」最核心的一对事实来源：① **后端状态快照**（`statusSnapshot`/`statusLoading`/`statusError`/`runtimeState`/`runtimeAnnouncePrimed`、事件流 `startStatusEventStream`、显式拉取 `loadStatusSnapshot`、`setStatusSnapshot`、`setRuntimeState`）；② **本机启停/重启进程动作**（`lifecycleState`、`requestProcessAction` 及进度卡状态与四类定时器、`syncProcessProgress`/`finishProcessAction`/`dismissProcessProgress`/`reopenProcessProgress` 等）。跨功能播报经**界面反馈域**（`app/feedback/store.ts`）完成，本 store **不反向依赖根 store**（依赖方向：lifecycle → feedback，无环）。方案把「长任务进度」定位为需要跨页面持续显示时的 feature store 归属（§19.4），进程进度与状态快照由此统一承载。**同步连接状态 `webdavState` 归位 `features/sync/store.ts`**（新增 `setWebdavState`），根壳的同步动作改经该 store 落位；**删除死状态 `proxyOffline`/`setProxyOffline`**（全仓无读/调用点）。根 `stores/app.ts` 以**只读 getter + 动作委托**保留 `app.statusSnapshot`/`app.runtimeState`/`app.processAction*`/`app.processProgress*`/`app.webdavState` 与 `setStatusSnapshot`/`loadStatusSnapshot`/`startStatusEventStream`/`requestProcessAction`/`dismissProcessProgress`/`syncProcessProgress`/`setRuntimeState`/`setWebdavState` 调用面；写入点（`tests/{panel,topbar,status-snapshot,process-action,process-progress-labels,overview-status-announce}.test.ts`、`dev/auditHarness.ts`）改用 lifecycle store。**公共动作集合无删除**（内部私有定时器/事件流句柄随域迁出；`proxyOffline` 为死代码清理）。`stores/app.ts` 886→654 行（累计 1492→654，约 −56%）。门禁：typecheck / vitest 63 文件 **290** 例 / build / 浏览器探针 **190/190** 全绿。

**§19.12 性能补测：Markdown 渲染耗时（2026-09-27，TASK-136）：** 审计装置新增 `scenario=markdown` 夹具（一条带约 48 小节长 Markdown 描述的 Skill，覆盖标题/段落/列表/加粗/行内代码）；浏览器探针新增「性能·Markdown 渲染」用例（点开详情弹窗计时到正文 `.ext-detail-md` 渲染出 > 50 个结构节点；数值写入报告 `performance`）。实测（③ 层开发服务器）**242 节点 / 约 34 ms**（另一次 24 ms）；探针总数 **193/193 通过、0 未捕获错误**。据此 §19.12「Markdown 渲染耗时」由未测量转为**已测**——与「长列表渲染」「路由切换」三项在前端可测层均已有实测数值。证据 `evidence/perf-interaction.md`。**仍无制品内计数器**：后台空闲 IPC 频率逐条计数（等效证据为事件驱动状态流 + 1s 轻量托盘排空 + 空闲 CPU 0.33%）。④ 层「渲染耗时精确分位」仍以 ③ 层数值 + ④ 制品内路由切换 < 150 ms 为对照，未在制品内逐分位计时。

**§19.12 性能补测：后台空闲 IPC 频率计数（① 层，2026-09-27，TASK-137）：** 新增 `tests/idle-ipc.test.ts`——在无用户操作、无状态迁移的空闲窗口，用假定时器推进时间并对前端 IPC 调用计数。实测：空闲 **60 s 恰好 60 次** IPC（全部为 `drain_tray_requests`，即 1 次/秒的托盘请求排空），**无其它空闲 IPC**；前后各 30 s 均 30 次（**速率恒定、不随时间增长**，排除泄漏式轮询）。状态快照、通知与运行来源均为**事件驱动**（`listen`，非前端轮询）。据此 §19.12「后台空闲 IPC 频率」由「无制品内计数器、无法测量」转为**已测（① 前端层确定性计数）**；④ 制品内因 Tauri v2 IPC 走 WebView 内部通道无外部计数器，仍以空闲 CPU avg 0.33%（§19.14 前述）+ 本节计数为等效证据。**至此 §19.12 前端可测项（路由切换、长列表、Markdown 渲染、后台空闲 IPC）均已有实测数值。**

**当前源码重打包与冒烟（E 切片后回归，2026-09-27，TASK-138）：** E 阶段全部切片（TASK-134/135）与 §19.12 性能补测（TASK-136/137）落地后，按 §19.11/§17.6「当前源码真实打包」要求重新构建制品：`cd apps/desktop/tauri && npx @tauri-apps/cli@2.11.4 build --target aarch64-apple-darwin`（构建前 `vue-tsc --noEmit && vite build` 通过）→ `.app`（可执行 SHA-256 `bd3b34b8…`，6.9 MB）与 `.dmg`（SHA-256 `f1ce5c1e…`）。**新制品冒烟**：主界面正常渲染（概览「可启动」+ 端口/版本/数据目录事实 + 最近事件空态 + 启停动作，证明状态快照 IPC 与事件流可用）、单进程、冷启动 3 次 **531/517/711 ms**（无挂起）、`quit` 后进程收敛。**包体**（对比 §19.12 基线）：主 JS 323,690 B（基线 312,966，**+3.4%**，在 ≤5% 内）、主 CSS 68,220 B（+0.7%）。证据 `evidence/app-smoke/smoke.md`（含 `smoke-current.png`）。**前端门禁（当前源码）**：typecheck / vitest 64 文件 291 例 / build / 浏览器探针 193/193；**后端门禁**：fmt / clippy `-D warnings` / `cargo test --workspace --features integration-test` **517** 全绿。据此，§19.11 中「前后端静态检查/测试/构建/当前源码真实打包/当前制品启动并完成关键流程冒烟」一项在**重构后源码**上重新取得证据；④ 层分支级交互项（GUI 同步成功、异常兜底条原生外观）仍受环境阻塞，见下条与前述证据。

**④ 配置迁移成功分支 GUI 真机复核（隔离实例，2026-09-27，TASK-139）：** 以 `open --env HOME=/tmp/ocx_gui1 -a "<当前制品>"` 取得隔离实例（自检概览「数据目录」= 沙箱、单进程），用 `cliclick` 坐标点击 + `screencapture` + `Vision` OCR 回读，补齐 §17.3 流程 5（备份与迁移）在 ④ 层缺失的**成功分支 GUI**：① **导出成功**——设置→配置迁移→「导出配置」，回读「**配置已导出（extension_config.enablement、extension_config.mcp、extension_config.skills、preferences）。**」，沙箱 `exports/opencodex-config.ocxdconf` 生成 **v2 未加密容器**（`container_magic: "OCXDCONF"`）；（首次因沙箱无 `preferences.json` 回读「配置导出失败；当前配置未修改。」——不伪造成功，符合预期）② **导入成功**——「导入配置」→ 确认弹窗（正文「将校验容器、备份当前配置并应用桌面壳设置…」）→ 确认，回读「**配置已导入（preferences）。**」，出现「导入 v2 · preferences」与「未应用的区段：extension_config（与当前一致）／asset_files（不在容器内）」，沙箱 `preferences.json` 被重写。**副作用边界**：全程只在沙箱内读写，真实用户 `runtime.json` **未被改动**（mtime 不变）、**无新增钥匙串条目**、沙箱已删除、进程已退出。证据 `evidence/app-smoke/gui-flows.md` §4 与 `migration-gui-{export-success,import-confirm,import-success}.png`。据此 ④「导入成功分支 GUI」缺口补齐；剩余 ④ 未达成项收敛为 **GUI 同步成功（凭据写真实登录钥匙串，端点校验要求非空口令，隔离实例钥匙串不可用）** 与 **运行期异常兜底条原生外观（无制品内合法触发入口）** 两项。

**§17.1 入口清单重新提取对账（2026-09-27，TASK-140）：** 复核发现既有 `entry-inventory.raw.json` 为**旧库存**——TASK-119 新增的 `App.vue` 运行期异常兜底条两个按钮（「关闭提示」「重新加载界面」）未被计入（此前做法是在旧 raw 上「渲染」，未重新「提取」）。本轮重新运行 `.extract-entries.mjs` 提取当前源码：**18 个文件 268 个入口**（较原记录 +1 文件 +2 入口）；在 `.render-inventory.mjs` 补 `App.vue` 状态映射（通过 / ①+③：`tests/error-boundary.test.ts` + 浏览器探针「交互·故障兜底条与恢复」；④ 原生外观仍受环境阻塞，见 §17.6）后重新渲染。**对账：通过 265 · 失败 0 · 阻塞 0 · 未执行 0 · 不适用 3（合计 268）**。据此更正上文入口总数与对账口径。清单：`entry-inventory.md`。

**§17.4 原型 QA 当前源码复跑（2026-09-27，TASK-141）：** 在 E 阶段重构（TASK-134/135）与性能补测（TASK-136/137）之后的当前源码上复跑原型 QA（脚本 `…/05-原型/原型/qa/*.{test,browser}.mjs`）：**jsdom 315 例（log-categories 91 / shim-progress 43 / skill-detail 135 / state-announce 30 / tray 16）× fail 0**；**无头 Chromium 415 例（log-categories 25 / nowrap 38 / runtime-source 87 / skill-detail 100 / spacing 11 / tables 154）× fail 0、console 错误 0**。原型 `index.html` SHA-256 `d5888597…`（未改动）。浏览器探针重渲染的 `…/05-原型/文档/截图/` 二进制改动已 `git checkout` 回退。据此，§17.4「原型 → 规范 → 实现复核」的脚本结论在**重构后当前源码**上复验通过（几何并排测量亦为当前源码，见 TASK-110）。

**修复：技术型文本输入被系统自动首字母大写（2026-09-27，TASK-142）：** 在 ④ 层制品内 GUI 同步复核中真机复现：在「账号」字段输入 `davuser`，制品实际发送 `Davuser`，服务端返回 `Authentication (basic) failed for user 'Davuser'` → 401（连接测试失败）。根因是技术型 `<input>` 未关闭 WKWebView 的按句自动首字母大写。**修复**：为 `SettingsRoute.vue` 的服务地址/远端目录/账号/应用密码/数据目录路径/外部 `OPENCODEX_HOME`/切换目标路径/安装位置/代理 host·用户名/指定版本/`uninstall` 确认，以及 `AppModal.vue` 的通用弹窗输入，统一补 `autocapitalize="off" autocorrect="off" spellcheck="false"`；**回归** `tests/technical-input-autocapitalize.test.ts`（同步/数据目录/弹窗三组）。修复后同一操作服务端记录 `davuser … -> 207`，认证通过。门禁：typecheck / vitest 65 文件 **294** 例 / build / 浏览器探针 **193/193** 全绿。证据 `evidence/app-smoke/webdav-gui-sync.md` §3。

**④ 制品内 GUI 驱动 WebDAV 同步成功分支（隔离实例，2026-09-27，TASK-143）：** 以 `open --env HOME=<沙箱> --env SSL_CERT_FILE=<自签 CA> -a "<当前制品>"` 取得隔离实例（先验沙箱「数据目录」）。**查明三条此前判为阻塞的先决条件**：① **凭据隔离可用**——`security_framework` 在 `HOME` 覆盖下按 `$HOME` 解析钥匙串，沙箱内 `security create-keychain/default-keychain/list-keychains/unlock-keychain` 建库并设为默认后，制品把 `ocx.dav.cred_*` 写入**沙箱钥匙串**（真实登录钥匙串计数不变）；② **TLS 信任可注入且不改系统信任库**——制品 `reqwest(rustls-tls-native-roots)` 经 `rustls-native-certs` 生效 `SSL_CERT_FILE`，但须用**CA 签发的叶证书**（自签「CA 兼叶」会被拒）；③ **冷同步默认开启会跳过同步**（`modules/sync/runner.rs` `cold_sync==true → ColdSync`，不发请求），复核前置 `cold_sync=false`。**GUI 步骤与回读**：设置→WebDAV 同步→填端点→「保存端点」回读「WebDAV 端点已保存。」；「测试连接」回读「**WebDAV 连接成功。**」（服务端 `davuser PROPFIND /desktop-sync/ → 207`）；「立即同步」回读「**WebDAV 同步已上传。**」（服务端 `GET latest.txt 404 → PUT 载荷 201 → PUT 清单 201 → PUT latest.txt 201`），远端 `/tmp/davroot/desktop-sync/` 生成 manifest/payload/latest.txt。**副作用边界**：全程仅沙箱（真实 `runtime.json` mtime 未变、无新增真实钥匙串条目、未改系统信任库），沙箱/证书/服务器/临时诊断文件均已清理。证据 `evidence/app-smoke/webdav-gui-sync.md`（含 `sync-gui-connection-success.png`、`sync-gui-upload-success.png`）。据此 ④「制品内 GUI 驱动同步成功分支」缺口补齐；§17.6 剩余 ④ 未达成项收敛为**唯一一项**：运行期异常兜底条原生外观与「重新加载界面」（无制品内合法触发入口）。

**输入修复后当前源码重打包（2026-09-27，TASK-144）：** TASK-142 修复（技术型文本输入关闭系统自动大写）落地后，按 §19.11「当前源码真实打包」重新构建：`cd apps/desktop/tauri && npx @tauri-apps/cli@2.11.4 build --target aarch64-apple-darwin` → `.app`（可执行 SHA-256 **`074437b0…`**）与 `.dmg`（SHA-256 **`4b866948…`**）。该制品即为 TASK-143 ④ GUI 驱动同步复核所用实例（连接成功 + 上传成功、冷启动正常、退出收敛），等价于一次完整制品冒烟。**门禁（当前源码）**：typecheck / vitest 65 文件 **294** 例 / build / 浏览器探针 **193/193**；后端 fmt / clippy `-D warnings` / `cargo test --workspace --features integration-test` **517**（此前 TASK-132 已发现的部分 WebDAV 用例需 `OCX_TEST_WEBDAV_URL/DIR` 才真跑，本轮补全 `DIR` 后 `webdav_sync_real` 4/4、真实 TLS 通过）。据此 §19.11 的打包/冒烟/门禁在**输入修复后的当前源码**上重新成立。


### 19.15 诊断中心交互调整与兜底条制品内触发（2026-09-28，TASK-145）

用户 2026-09-28 批注四项，原型与实现同步落地；其中第 ③ 项直接解除 §17.6 中「运行期异常兜底条原生外观」因**制品内无合法触发入口**而长期阻塞的状态。

1. **默认落地页改为「环境诊断」**（取代 2026-09-24 的「日志历史」口径）：`apps/desktop/ui/src/navigation/index.ts` 的 `normalizeDiagnosticsTab` 缺失 / 非法取值回落 `doctor`；原型 `index.html` 的 `.diag-tabs` / `#diag-doctor` 默认 `active`。`#logs?tab=logs` 仍可直达日志历史。
2. **Doctor 动作按钮改中文名「环境诊断」**（原「运行 Doctor」；运行中显示「诊断中…」）。
3. **新增「界面诊断」自检**（新增 `apps/desktop/ui/src/features/diagnostics/selfCheck.ts`）：在环境诊断面板内主动抛出一次**受控异常**，让它走**真实错误边界**——`installErrorHandlers` 安装的 Vue `errorHandler` 捕获后调用 `reportUiFault`，置位顶部兜底条 `.ui-fault`，再由用户点「关闭提示」或「重新加载界面」恢复。**验证的是整条链路是否真的接通，而不是绕过链路直接置位状态**；只在用户显式点击时执行，不写配置、不改数据，可重复执行。
4. **日志历史「面板请求日志」入口改用标准按钮规格**（`.btn ghost`，与同排「清理日志 / 刷新」一致）：原型侧此前是小号胶囊样式，现与实现对齐。

**门禁（全绿）**：前端 `vue-tsc --noEmit` 通过；`vite build` 通过；`vitest --run` **65 文件 295 例**（`navigation.test.ts` 默认值断言更新、`log-categories.test.ts` 新增「受控异常经 errorHandler 置位兜底条」用例）；浏览器探针 `npm run audit:browser` **204 通过 / 0 未捕获错误**（新增「交互·界面诊断自检（制品内触发兜底条）」：断言初始无兜底条 → 触发后出现且 `role=alert` → 文案含「界面异常自检」→ 两个恢复动作齐全 → 「关闭提示」恢复 → 可重复触发 → 「重新加载界面」重载后干净）。原型 QA（当前源码）：jsdom **332**（`log-categories` 108 / `shim-progress` 43 / `skill-detail` 135 / `state-announce` 30 / `tray` 16）× fail 0；浏览器 **415** × fail 0、console 错误 0。原型 `index.html` SHA-256 由 `d5888597…` 变更（默认 Tab / 按钮文案 / 面板入口规格 / 新增自检块）；浏览器 QA 重渲染的 `…/05-原型/文档/截图/` 二进制改动已 `git checkout` 回退。

**回写**：`UI规范.md` §11.1、`日志分类展示方案.md` §3 / §7、`IMP-OPENCODEX-DESKTOP-01.md` 变更记录行、本节。

**下一步**：以当前源码重打包制品，并在隔离实例内用真机点击 + OCR 回读验证兜底条原生外观与「重新加载界面」（TASK-146），据此闭合 §17.6 的 ④ 层最后一项。


### 19.16 ④ 层制品内兜底条真机复核与新缺陷（2026-09-28，TASK-146）

**当前源码重打包**：`cd apps/desktop/tauri && npx @tauri-apps/cli@2.11.4 build --target aarch64-apple-darwin`（构建前 `vue-tsc --noEmit && vite build` 通过）→ `.app` 可执行 SHA-256 `0cc2fe4f1c7a4b4b3a7f4b9bab4d494263e0e4afaff525f64b9e21a2759c3327`（6.66 MB）、`.dmg` SHA-256 `aa0c16a63b5efea8999438d04dad3711261b07f87048e80100e4b0951e7406a1`（3.6 MB）。

**④ 层真机复核（隔离实例 + 像素证据链）**：`open --env HOME=/tmp/ocx_fault_home -a "<制品>"`（自检「数据目录」= 沙箱，全部状态写 `/tmp/ocx_fault_home/Library/Application Support/com.gzers.opencodex.desktop/`），`cliclick` 真机点击 + `screencapture -R` 定域截屏 + `swiftc` 编译的 `Vision` OCR 回读。结果：

| 步骤 | 回读 |
|---|---|
| 进「诊断」 | Tab 行「环境诊断 / 日志历史 / 通知历史」，**默认落在「环境诊断」**；卡内「运行环境诊断」+ 只读说明 + 「界面异常自检」；按钮「环境诊断」「界面诊断」 |
| 点「界面诊断」 | 窗口**顶部原生出现告警条**：「界面异常：Error：界面异常自检：这是一次受控的示例异常，用于验证兜底条与恢复动作。（…）」，右侧「关闭提示」「重新加载界面」 |
| 点「关闭提示」 | 告警条消失，页面正常 |
| 再点「界面诊断」 | 告警条**可重复触发** |
| 点「重新加载界面」 | 整页重载后回到「诊断中心」（保留 `#logs?tab=doctor`），**告警条不存在** |

据此，§17.6 ④ 层最后一项「**运行期异常兜底条原生外观与「重新加载界面」**」**通过**（不再以「无制品内合法触发入口」为由阻塞）。副作用边界：真实数据根未被读写；真实登录钥匙串无新增条目（`ocx.dav` 计数仍为 1，即 2026-09-19 既有条目）；复核后应用已退出。证据 `evidence/app-smoke/faultbar-native.md`（含 5 张截图）。

**门禁**：前端 `vue-tsc --noEmit` 通过、`vite build` 通过；后端 `cargo fmt --all -- --check` / `clippy --workspace --all-targets -D warnings` / `cargo test --workspace --features integration-test` **517 通过 / 0 失败 / 3 忽略**。

**本轮发现的新缺陷（列 TASK-147）**：生产构建里 Vue 传给 `config.errorHandler` 的第三参数是**错误文档 URL**（`https://vuejs.org/error-reference/#runtime-N`，见 `@vue/runtime-core` 的 `handleError`：`process.env.NODE_ENV !== 'production' ? ErrorTypeStrings[type] : 'https://vuejs.org/error-reference/#runtime-' + type`），而 `installErrorHandlers` 把它**原样拼进用户可见文案**（`reportUiFault(message, info)` → `message（info）`），于是兜底条里出现一段英文链接，违反 `UI规范` §10「用户可见文案不得直接渲染英文技术串」。仅影响文案、不影响外观与恢复动作，故不阻塞本任务的 ④ 判定；修复与复验列入 TASK-147。


### 19.17 修复：兜底条文案泄漏英文错误文档链接（2026-09-28，TASK-147）

承接 §19.16 登记的新缺陷。**根因**：`@vue/runtime-core` 的 `handleError` 在生产构建下把第三参数设为错误文档链接——

```js
const errorInfo = process.env.NODE_ENV !== 'production'
  ? ErrorTypeStrings[type]
  : `https://vuejs.org/error-reference/#runtime-${type}`
```

而 `installErrorHandlers` 把该参数原样交给了 `reportUiFault(message, info)`（拼成 `message（info）`），于是用户可见的兜底条里出现一段英文链接（开发构建则是 `render function` 之类的英文标识），违反 `UI规范` §10。

**修复**：`apps/desktop/ui/src/app/errors.ts` 新增 `readableFaultInfo()`：链接一律丢弃；已知 Vue 标识映射为中文（`setup function`→初始化、`render function`→渲染、`v-on handler` / `native event handler`→事件处理…）；其余含拉丁字母的未知标识丢弃；非拉丁标识截断 60 字。`installErrorHandlers` 改用它。**用户可见文案现在只有中文原因**，`unhandledrejection` 分支（本就不传 info）不受影响。

**门禁**：前端 `vue-tsc --noEmit` 通过；`vitest --run` **65 文件 296 例**（`error-boundary.test.ts` 新增「出错位置标识归一」用例并覆盖生产入参：文案含 `prod broke`、**不含 `vuejs.org`**、不出现空括号）；`vite build` 通过；后端 `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace --features integration-test` **517 / 0 / 3**。

> 浏览器探针侧：自检用例的「兜底条文案不含英文链接」断言与其复跑（`audit:browser` 205/205）随验收收口任务（§19.18）落地；本节只含源码 / 单测 / 规范回写。

**新制品复验（隔离实例真机）**：`.app` 可执行 SHA-256 `70b58d42ae6edeb35def6c01833e520735799e22469426db940b0c49d49bbaca`、`.dmg` SHA-256 `2eed10313ea1b7149eeb180c8dc536e2e10e775a85dfd5d100fe06c4dd66d8c8`。同一像素证据链（`CGEvent` 点击 + 定域截屏 + Vision OCR）复跑：默认落「环境诊断」→ 点「界面诊断」→ 顶部告警条文案为「界面异常：Error：界面异常自检：这是一次受控的示例异常，用于验证兜底条与恢复动作。」**不含任何 `http(s)://`**（旧制品含 `https://vuejs.org/error-reference/#runtime-5`）→ 可重复触发 → 「关闭提示」恢复 → 「重新加载界面」重载后干净。副作用边界：真实数据根未读写、真实钥匙串无新增条目、应用已退出。证据 `evidence/app-smoke/faultbar-native.md`（附节，含 5 张截图）；规范回写 `UI规范.md` §24。

**工具备注**：`cliclick` 会把负坐标夹到 0，无法点击主显示器左侧的副显示器；副显示器点击需自行投递 `CGEvent`（已写入证据文件供复用）。


### 19.18 验收收口：§17.6 逐条对齐（2026-09-28，TASK-148）

**收口动作**：① 浏览器探针补「兜底条文案不含英文链接」断言并复跑（`audit:browser` **205/205、0 未捕获错误**）；② 入口清单按 ④ 层新证据改判并重渲染；③ 原型↔实现几何对照复跑；④ 逐条核对 §17.6。

**入口清单（§17.1）**：`.render-inventory.mjs` 复跑输出 **268 项**（18 文件）；`App.vue` 兜底条两入口由「①+③（④ 受阻塞）」改判为 **①+③+④**，证据列指向 `evidence/app-smoke/faultbar-native.md` 与本节前的 §19.16/§19.17；等价类汇总的「已通过但仍有未覆盖分支」一条改为**已全部补齐**。**对账：通过 265 · 失败 0 · 阻塞 0 · 未执行 0 · 不适用 3（合计 268）**。

**原型↔实现几何对照（§17.4）**：`geometry-compare.mjs` 复跑（宽/窄 × 亮/暗四组）**全部一致**——sideW 244/64、圆角 12、描边 1、main 底色、titlebar 28、glass-rail/glass-blur/accent 与 token 差异「无」。原型本轮四项批注（默认 Tab、按钮中文名、界面异常自检、面板请求日志标准按钮）与实现逐项一致。

**§17.6 逐条核对**：

| 门槛条目 | 结论 | 证据 |
|---|---|---|
| 入口盘点无遗漏；必需用例无失败/阻塞/未执行 | 通过 | 268 入口；通过 265 · 不适用 3（DEV-only）· 未执行 0（`entry-inventory.md`） |
| 全流程正常与适用异常分支通过；契约与真实副作用/持久化一致 | 通过 | ①/②/③/④ 四层；④ 层含启动/生命周期/托盘/原生选择器/官方面板/缩放/失败分支/联网安装/离线安装/卸载/隔离重测/备份/导出/导入/同步失败/**同步成功**/CLI 契约/**异常兜底条与重载**；② 真实 TLS 同步 10 例；契约未改（FZ-* 冻结） |
| 原型/规范差异逐项解释或修复；样式数值与组件规则检查通过 | 通过 | 本轮四项批注原型与实现同步；几何对照四组全一致；`vitest` 含 `component-style-rules` / `no-wrap-labels` 等规则用例 |
| 性能达目标、无未解释回归 | 通过 | 路由切换 17–29 ms、200 条通知渲染 229–246 ms、Markdown 渲染 20–66 ms、空闲 IPC 60 s/60 次；制品包体主 JS 325.00 kB / CSS 68.34 kB（对基线 +<5%） |
| 前后端静态检查、测试、构建、当前源码真实打包通过 | 通过 | 前端 `vue-tsc` / `vitest` 65 文件 296 例 / `vite build` / `audit:browser` 205；后端 `fmt` / `clippy -D warnings` / `cargo test --workspace --features integration-test` 517/0/3；`tauri build --target aarch64-apple-darwin` 出 `.app` + `.dmg` |
| 当前制品启动并完成关键流程冒烟；提交/环境/产物路径与哈希可追踪 | 通过 | 制品 `.app`（exe SHA-256 `70b58d42ae6edeb35def6c01833e520735799e22469426db940b0c49d49bbaca`，6.9 MB）与 `.dmg`（`2eed10313ea1b7149eeb180c8dc536e2e10e775a85dfd5d100fe06c4dd66d8c8`，3.6 MB），路径 `apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/`；隔离实例冒烟：启动→概览渲染→诊断（默认环境诊断）→界面诊断自检（兜底条原生出现、文案为中文无链接）→关闭提示→重复触发→重新加载界面（干净）→退出 |
| 核心文档、实施结论与回执对齐；验收矩阵与缺陷清单有证据索引 | 通过 | 回写 `UI规范` §11.1/§24、`日志分类展示方案` §3/§7、`IMP-01` 变更记录、本文件 §19.15–§19.18；缺陷（TASK-147）已修复并复验 |

**结论**：§17.6 各项均已取得当前源码/当前制品的可复核证据，**本轮目标的完成定义（§19.11）达成**。该结论仅覆盖本地构建、测试、原型复核与制品冒烟；**不包含**签名 / 公证 / 发布 / 推送等未授权动作（按 §18 授权边界，需另行确认）。


### 19.19 界面上澄清与修订：异常提示改弹窗（2026-09-28，TASK-149）

**用户澄清**：§19.15 引入的「界面诊断」自检所触发的**界面异常提示带恢复动作**，应改为**居中弹窗**；上一轮被误改的「通知中心」（铃铛下拉）**不在此列**，改回原样。据此：通知中心的改动（`NotificationCenter.vue` / `App.vue` / `AppTopbar.vue` / `base.css` / 测试 / 探针 / 原型）**全部 `git checkout` 回退，未提交**，通知中心保持既有浮层形态。

**改动**：`.ui-fault` 由「窗口内顶部整宽固定条」改为「居中弹窗 + 全窗遮罩」——`App.vue` 渲染 `.ui-fault-modal`（`role="alertdialog"` + `aria-modal="true"`，标题「界面出现异常」，正文可换行），`base.css` 定义容器 / 遮罩 / 卡片，`z-index = calc(var(--z-modal) + 1)` 保证故障提示压在应用弹窗之上；关闭路径为「关闭提示 / 点遮罩 / Esc」，三者都只清提示、不触碰其它状态。原型同步：`index.html` 的 `.selfcheck` 内联预览改为窗口内的弹窗（`#uiFaultModal` + `#uiFaultMask`），`qa/log-categories.test.mjs` 相应更新。

**门禁**：前端 `vue-tsc --noEmit` 通过；`vitest --run` **65 文件 296 例**；`vite build` 通过；浏览器探针 `audit:browser` **209/209 通过 / 0 未捕获错误**（故障提示用例断言 `role=alertdialog`、`aria-modal`、遮罩、居中、文案不含英文链接、可关闭、可重复触发、重载后干净；自检用例另断言点遮罩关闭）。原型 QA：jsdom **336**（`log-categories` 112）× fail 0；浏览器 **415** × fail 0。**生产构建产物校验**（`dist/` + 无头 Chromium，投递真实 `unhandledrejection`）：弹窗出现、`role=alertdialog`、`aria-modal=true`、有遮罩、居中（720,450 / 1440×900）、宽 480、两个恢复动作齐备、0 页面错误、bundle 内不含 DEV 审计装置。

**制品与真机复核**：以当前源码重打包 `.app`（可执行 SHA-256 `a3e6c504b537ba1f5e57f3d64e1b83d07eaa4dc7cbffb9c00aeeb697157ea554`）与 `.dmg`（`1acbb0a66da8a2c0d66e09afdab326a1d2c97efbdb6dccc35e131fc1d566e672`）。**制品的真机屏幕复核本轮未完成**：复核期间用户显示器布局变化（外接屏被移除）且随后**屏幕锁定**（`CGSSessionScreenIsLocked = True`），应用进程存活但窗口不可见、`screencapture` 返回空白，无法取得像素证据。该阻塞属环境状态，与本次改动无关；当时弹窗的结构/语义/居中/关闭路径已由 ③ 层探针与生产 bundle 校验覆盖，制品的屏上复核留待屏幕解锁后补做。**（2026-09-28 补记，TASK-150：已补做并通过）** 屏幕解锁后以同一链条完成：隔离实例（`open --env HOME=<沙箱>`，事后核对真实数据根当天**未被写入**、真实钥匙串 `ocx.dav` 计数仍为 1）内真机点击，「**界面出现异常**」弹窗在制品窗口内出现——回读标题「界面出现异常」、原因「界面异常：Error：界面异常自检：这是一次受控的示例异常…」、两个动作「关闭提示 / 重新加载界面」；四条恢复路径逐条通过：**关闭提示**→弹窗消失、**可重复触发**、**Esc**→消失、**点遮罩**→消失、**重新加载界面**→整页重载后回到「诊断中心」且弹窗不存在。截图 `evidence/app-smoke/fault-modal-0{1,2,3,4,5,6,7}-*.png`。**TASK-149 遗留项至此闭合。**


### 19.20 原型：安装版本管理（卸载覆盖任意允许来源、卸载干净、备份只提醒）(2026-09-28，TASK-151)

**用户决定（2026-09-28）**：现有卸载入口只在「托管安装」下出现，等于**没装托管版就永远看不到卸载**；用户明确要求改为——① **只要查出来是「允许来源」都允许卸载**（托管安装 / 用户指定 / 自动发现候选）；② **卸载要干净**；③ **备份只检测与提醒**，未检测到备份也**不阻断**卸载，只提醒。按既定「先原型后实现」，本轮**只改原型**（`index.html` + 其 QA），确认后再落实现与规范。

**原型改动**（`…/05-原型/原型/index.html`）：

- **卸载入口常显**：运行来源卡片动作行改为——来源 ≠「未解析」时始终渲染 `卸载…`（`data-prototype-action="runtime-uninstall"`）；`更新（官方 ocx update）`仍只在托管安装下出现。
- **卸载方案（按来源计算）**：弹窗不再只有「托管两级」，改为**统一方案**，先给事实（来源类型 / 路径 / 落点）与「将移除的对象」分层清单（**包体** / **入口** / 完整卸载时的**运行态** service·shim·config），范围单选 `完整卸载（推荐）` / `仅移除包体与入口`。
- **执行边界写清**：托管包体由应用直接删除；**外部包体**（npm 全局前缀 / 用户指定位置）**不接管**——只列精确终端命令（如 `npm rm -g @bitkyc08/opencodex`），由用户执行；运行态由官方 `ocx uninstall` 承担。
- **备份只提醒、不阻断**：`未检测到可恢复备份；卸载后不可恢复。不阻断卸载，但建议先备份`（琥珀）+ 默认勾选「卸载前自动生成备份」；检测到备份则绿字提示可复用。主行动的可用性**只**由「已阅读清单并确认继续」决定，与备份状态**无关**。
- **卸载后「残留核验」**：完成态重新扫描包体 / 入口 / service / shim / config，逐项给出结论；外部来源附「需在终端执行」的命令。
- 右侧测试器新增「卸载 · 备份检测（mock）」两态开关，便于复核两种提示。

**门禁（当前源码原型）**：原型 jsdom **336**（`log-categories` 112 / `shim-progress` 43 / `skill-detail` 135 / `state-announce` 30 / `tray` 16）× fail 0；原型浏览器 QA **430**（`log-categories` 25 / `nowrap` 38 / `runtime-source` 102 / `skill-detail` 100 / `spacing` 11 / `tables` 154）× fail 0、0 未捕获错误。其中 `runtime-source.browser.mjs` 卸载段重写为：任意来源可卸载、完整 / 仅包体两级、备份只提醒（无备份仍可确认）、外部包体给出精确命令、确认门禁、进度视图、残留核验。截图不再入仓，复核证据存 `.adg/work/imp03-04-execution/evidence/prototype-uninstall-20260928/`。

**待用户确认的设计决策（确认后再回写需求 / 规范 / 实现）**：

1. **官方 `ocx uninstall` 是「执行」还是「引导」**：`DMD` §10 写「本产品只做备份、确认与结果展示」，而 `UI规范` §18.4 写「**调用**官方 `ocx uninstall`」——两处口径不一致，现状实现是**只引导**。原型按「干净卸载」取**执行官方命令**（需先确认），与既有「经官方 CLI 转发」的 `ocx codex-shim` 先例一致。
2. **外部包体（全局 npm / Homebrew）是否由应用执行移除**：`AC-01` / `AC-08` 明确**不接管全局 npm 前缀**，故原型取**只引导命令**；若要求应用执行，需先改 `AC-01` / `AC-08` 与 `REQ-29`。
3. **`OPENCODEX_HOME`（含 provider 凭据）是否纳入卸载范围**：原型取**默认保留**，提供独立勾选「同时删除数据目录」，需确认。


### 19.21 原型：卸载进度动效 + 官方卸载真实行为核验（2026-09-28，TASK-152）

**用户追加**：① 卸载进度要动效；② 确认清楚 OpenCodex 到底装了哪些文件、怎样才算干净卸载。

#### 19.21.1 官方安装 / 卸载的真实文件面（读安装包源码核验，非文档推测）

核验对象：本机 npm 全局安装 `~/.local/lib/node_modules/@bitkyc08/opencodex`（v2.50.0，111 MB，含 bundled bun runtime）。依据 `src/cli/index.ts` 的 `handleUninstall()`、`src/cli/uninstall-plan.ts`、`src/lib/config-ownership.ts`、`src/service.ts`、`src/codex/shim.ts`。**macOS 口径**。

| 层 | 落点 | 由谁移除 |
|---|---|---|
| 包体（npm 全局） | `<npm prefix>/lib/node_modules/@bitkyc08/opencodex`（本机 `~/.local/lib/...`）+ `<prefix>/bin/ocx` 符号链接 | **官方不删**，末尾提示 `npm uninstall -g @bitkyc08/opencodex` |
| 包体（本产品托管） | `<数据根>/runtime/opencodex` + `<数据根>/runtime/bin/ocx` | 本产品（`FZ-51` 一级） |
| service | `~/Library/LaunchAgents/com.opencodex.proxy.plist`（label `com.opencodex.proxy`；Windows = Task Scheduler + WinSW；Linux = systemd user unit） | 官方 `ocx uninstall` |
| codex autostart shim | **原地替换**真实 `codex` 启动器并留备份；状态记在 `OPENCODEX_HOME/codex-shim.json` | 官方 `ocx uninstall`（还原备份） |
| 客户端共享配置 | `~/.codex/config.toml`（`restoreNativeCodexAsync`）、Grok 配置（`stripGrokConfig`） | 官方 `ocx uninstall` |
| macOS 环境 | 系统环境变量跟踪文件回退（`revertSystemEnv`）+ shell hook 移除（`uninstallShellHook`） | 官方 `ocx uninstall` |
| 数据（`OPENCODEX_HOME`，默认 `~/.opencodex`） | **只删 `.opencodex-uninstall.json` 清单内的「官方自有」路径** | 官方 `ocx uninstall`（`removeOwnedConfigState`） |

- **官方自有清单**（`INITIAL_OWNED_PATHS`，首次初始化时种子 44 项，之后按 `recordOwnedConfigPath` 追加）：`config.json`、`auth.json`、`admin-api-token`、`catalog-backup.json`、`codex-accounts.json`、`codex-shim.json`、`config-mutation.sqlite*`、`ocx.pid`、`runtime-port.json`、`service-state.json`、`service.log`、`service-api-token`、`tray-state.json`、`usage.jsonl`、`version.json`、`winsw`、`opencodex-service*.{cmd,xml,vbs}`、tray 图标/脚本、`crash.log`、`responses-state.json`、`thought-signature-replay.json`、`update-job.json` 等。
- **非官方自有 = 保留**（本机实际存在）：`integrations/`、`lab/`、`routing-history.sqlite{,-shm,-wal}`、`codex-quota-cache.json`、`catalog-backup-<hash>.json`、`config.json.invalid-*`、`config.json.pre-*.bak`、`config.json.managed-backup-*` —— 官方把它们当 residual 报告，不删。
- **拒绝条件**：`.opencodex-owner.json` / `.opencodex-uninstall.json` 缺失或非法 → `removeOwnedConfigState` 返回 `refused`，**一个都不删**（宁可留残留，不误删）。
- **共享 teardown 有前置证明**（`sharedTeardownAuthorized`）：service 已停 / 已移除 / 端点被三态探针证明 down，三者都满足才恢复原生 Codex 与 Grok 配置；否则跳过并保留 config/backups 供重试。
- **`~/.codex` 内另有 OpenCodex 自有文件**（如 `.opencodex-native-main.{claim,owner}.sqlite`），不在 `OPENCODEX_HOME` 清单内 —— 干净卸载的「残留核验」必须覆盖这一层。

**结论（干净卸载的定义）**：分四层移除并逐层核验 —— ① 包体与入口（npm 全局 / 用户指定 / 托管，各自的前缀与 bin）；② 运行态（service plist / shim 备份 / `~/.codex/config.toml` / shell hook）；③ 客户端与原生恢复（已由官方命令负责）；④ 数据（官方自有清单 + 非清单残留 + `~/.codex` 内自有文件）。本产品**可直接执行**的只有「托管包体 + 备份」；外部包体与 `npm uninstall -g` 属全局 npm 前缀，按 `AC-01`/`AC-08` **只引导**；官方 `ocx uninstall` 由谁执行仍待确认（§19.20 决策点 1）。

#### 19.21.2 原型改动（`…/05-原型/原型/index.html`）

- **进度动效**：标题行加转圈（完成后变绿色 ✓）；进度条进行中走**流光**（`.wiz-progress-fill.run`），完成后转绿；当前步骤圆点**脉冲**、已完成打勾；完成态**残留核验卡片弹入** + 绿色通过徽标 `pop`；`prefers-reduced-motion` 下全部关闭。
- **命令行明细**：完整卸载新增 `#uninstallConsole`，按**官方真实输出口径**逐行推进（`$ ocx uninstall` → service/proxy → backup → 官方各步 → `opencodex local state removed. Remove the package with: npm uninstall -g @bitkyc08/opencodex`），自动贴底；仅移除包体只给 `ocx stop` + 包体步骤。
- **口径修正**：运行态一行补上 shell hook 与 launchd label；数据目录复选框由「删除 `OPENCODEX_HOME`」改为「**清空非官方自有残留**」——官方卸载本就只删 `.opencodex-uninstall.json` 清单内的自有路径，`integrations/`、`lab/`、`routing-history.sqlite` 等默认保留、勾选后由应用另行清理。
- **残留核验**：改为逐项列表（包体 / 入口 / service plist / shim 与 `~/.codex/config.toml`），外部来源附「到终端执行 `npm rm -g …` 后回来重新发现」。

**门禁**：原型 jsdom **336/336**；原型浏览器 QA `runtime-source` **108/108**、0 未捕获错误。证据 `.adg/work/imp03-04-execution/evidence/prototype-uninstall-20260928/`（含进行中动效帧）。


### 19.22 修复：卸载弹窗「确认」项掉到折线以下，主行动灰着且无原因（2026-09-28，TASK-153）

**用户原型复核反馈**：「卸载按钮点不动」。

**根因（TASK-152 回归）**：卸载主行动由「我已阅读清单并确认继续」门禁（`syncUninstallConfirm`），而 §19.21.2 为压缩高度把该确认项排到了正文最末。窗口偏矮时它**落到 `#modalBody` 的折线以下**——用户只看到「卸载」灰着，既看不到那个勾选框、也看不到任何原因。这不是点击失效，是**门禁不可见**。

**修复**：

- 确认项改为**粘在弹窗正文底部的常驻条**（`.uninstall-ack-bar{position:sticky;bottom:-14px}`，带顶部分隔线与浅投影），无论正文滚到哪里都可见；正文高度也不再需要为此留白。
- 主行动禁用时该条内给一条**可读原因**：`勾选上面这一项，「卸载」才会亮起。`（`.ack-hint`，勾选后隐藏）。
- QA 补断言：确认项矩形必须落在弹窗可见区内、禁用时原因可见、勾选后原因消失。

**门禁**：原型浏览器 QA `runtime-source` **111/111**（+3）、0 未捕获错误。几何自检：1692×916 / 1180×760 / 900×600 三种窗口下确认项均**在可见区内**（此前 900×600 时在折线以下）。


### 19.23 卸载三向决策落地：应用执行 / 代执行 / OPENCODEX_HOME 纳入清理（2026-09-28，TASK-154）

**用户决策（2026-09-28）**：① 官方 `ocx uninstall` 由应用**执行**（不再是仅引导）；② 外部包体（npm 全局 / 用户指定）的移除由应用**代执行**；③ `OPENCODEX_HOME` **纳入清理范围**。

**需求 / 规范回写（DMD Revision 11）**：

- `AC-01`：卸载由「卸载管理器托管的安装」改为「对**任意已解析的允许来源**执行卸载（托管包体应用直接移除，用户指定与自动发现的安装由应用代执行官方卸包命令；完整卸载由应用调用官方 `ocx uninstall`），并做只读残留核验」；「不接管全局 npm 前缀」收窄为「**不安装**、不解析依赖、不改写全局前缀中的其它内容」。
- `AC-08`：保留「升级前备份 / 引导官方 `ocx update` / 不重写官方更新事务」，把「不接管 npm 包管理器」明确为「仅在用户显式确认的卸载流程中移除官方包自身」。
- §2.2 卸载分级重写为**两级统一流程**（仅包体与入口 / 完整卸载）+ 残留核验 + 备份只提醒；§8 P1 行与 §10 安全边界同步（官方命令由应用调用执行、删除边界仍由官方命令承担）。
- `UI规范` §18.1（动作行按来源常显 `卸载…`）与 §18.4 整节重写（范围两级、应用代执行、备份只提醒、确认项常驻、残留核验、进度动效）。

**原型改动**：

- 外部包体的文案由「需你在终端执行」改为「**确认后由应用代执行**」：`auto` 用 `npm uninstall -g @bitkyc08/opencodex`、`user` 用删除该前缀包体；命令行明细里这两行由「- 需在终端执行」改为 `$ <命令>` + `✅ removed <pkg>` / `✅ removed <entry>`。
- 残留核验与完成态：外部来源不再提示去终端，改为「外部包体已由应用执行 `<命令>` 移除」。
- `OPENCODEX_HOME` 清理项改为**完整卸载专属**（`仅移除包体与入口` 时禁用并注明），默认勾选；勾选后在命令行明细里出一行「✅ 非官方自有残留已清空」。
- 确认项文案补「全部由应用代执行，不会再要求你去终端」。

**门禁**：原型 jsdom **336/336**；浏览器 QA `runtime-source` **112/112**、0 未捕获错误。

**待办（本轮未做）**：实现侧（`apps/desktop/tauri` 的 `modules/runtime/uninstall.rs` 与 UI）与**本次需求修订对应的安全评审增量**（`安全评审输入.md` §12.4 的「应用代执行全局 npm 卸包 / 清空 `OPENCODEX_HOME` 残留」）尚未落地，按流程需在实现前补齐。


### 19.24 原型定稿留档 + Revision 11 派生文档回写（2026-09-28，TASK-155）

**原型定稿（用户确认「原型上没有问题了」）**，定稿口径：

- 卸载入口 `卸载`（**无省略号**），运行来源已解析即常显；
- 卸载弹窗：范围两级、`将移除的对象` 折叠清单、备份**只提醒不阻断**、数据清理为**紧凑单行**、确认项**常驻底部**（sticky + 禁用原因）；
- 进度视图：转圈 / 进度条流光 / 步骤脉冲 / 完成弹入（`prefers-reduced-motion` 全关）；**命令明细框固定 132px**（约 6 行，超出框内滚动 + 贴底）；
- **正文无滚动条**：默认窗口（`1692×916`）确认视图与完成视图 `#modalBody` 溢出均为 0；`≤1180×760` 时按需滚动；
- 完成态：**只读残留核验**卡片（包体 / 入口 / service plist / shim 与 `~/.codex/config.toml`），说明句与清单**同字号**（11px）。

**派生文档回写（Revision 11 追踪锚点）**：

| 文档 | 变更 |
|---|---|
| `需求分析产物/需求基线.md` | **补登记 `REQ-29`**（Revision 7 引入时漏记）并写入 Revision 11 口径；修订 `REQ-01` / `REQ-16`（「不接管 npm」收窄为「不安装 / 不解析依赖 / 不改写全局前缀其它内容，仅显式确认的卸载流程中移除官方包自身」） |
| `需求分析产物/场景与验收矩阵.md` | `AC-01` 段重写为「安装发现与卸载」，补托管 / 外部 / 数据清理 / 备份缺失 / 恢复五条路径；`AC-08` 补「涉及 npm 前缀」边界行；追踪表 `AC-01` → `REQ-01 / REQ-29` |
| `02-项目核心/能力地图.md` | `CAP-1.5` 由「卸载（两级）」改为「卸载（任意已解析来源）」并写明由应用执行与残留核验；`CAP-1.4` 说明与「禁止」项、`DOM-08` 升级域措辞同步 |
| `02-项目核心/产品定义.md` | `IN-08` 由「卸载管理器托管的安装」改为「卸载任意已解析来源的安装（应用执行）」 |

**结论**：需求侧（`DMD` Revision 11）+ 规范侧（`UI规范` §18.1 / §18.4）+ 派生侧（基线 / 矩阵 / 能力地图 / 产品定义）与**定稿原型**口径一致，可据此进入实现。


### 19.25 实现（后端）：安全评审增量 + 统一卸载内核（2026-09-28，TASK-156）

**安全评审增量（实现前置，`安全评审输入.md` §12.4.1）**：新增 T-I11 ~ T-I15 与增量控制 / 恢复路径 / 待冻结契约——① 卸载越权执行（固定两条命令、包名硬编码、不走 shell、留痕）；② `OPENCODEX_HOME` 清理越界（只用官方清单 + **显式类别** + 路径必须落在 home 内 + 不跟随符号链接）；③ 备份承诺的不可恢复（**勾选即承诺**：备份失败即中止）；④ 残留核验不可信（三项态 `cleared` / `present` / **`unknown`**，未知不得报「已清理干净」）；⑤ 掩盖官方跳过的共享 teardown（转发官方跳过/失败标记，非零退出判失败、不重试、不代删）。

**契约修订（`契约字段.md` §8，`FZ-51r`）**：范围轴由「托管 / 官方」改为 `body` / `full`；新增请求（`autoBackup` / `cleanData` / `confirmation`）、只读方案 DTO（`managed` / `removable` / `reason` / 清单 / 固定命令）与结果 DTO（`steps[]` / `residue[]` / `officialOutput[]`）；稳定错误码新增 `uninstall_backup_failed` / `uninstall_command_failed` / `bad_scope`。

**后端实现**：

- `modules/runtime/uninstall.rs`：新增 `BodyOwner`（`Managed` / `External`）与 `ExternalRemoval`（`NpmGlobal` / `PackageDir` / `Unsupported`）——按**已解析来源**判定归属，认不出来一律 `Unsupported` 并**拒绝代删**；`plan_uninstall`（只读方案 + 可复用备份检测 + 残留候选）；`execute_uninstall`（固定顺序，任一步失败如实记录）；`clean_opencodex_residue`（显式类别 + 路径校验 + 不跟随链接）；`residue_check`（三项态）；`SystemUninstallCommands`（受控环境跑 `ocx uninstall` 与 `npm uninstall -g @bitkyc08/opencodex`）；`UninstallCommands` 便于测试用桩。删除旧的「只引导」实现（`OfficialUninstallPlan` / `prepare_official_uninstall`）。
- `types/runtime.rs`：请求 / 方案 / 结果 DTO 重建（含 `confirmed()` 解析）。
- `commands/runtime.rs`：新增 `plan_runtime_uninstall`（只读）；`uninstall_runtime` 重写为「停代理 → 计划 → 执行 → 重解析来源 → 写历史 → 广播」；`lib.rs` 注册新命令。
- 删除 `official_uninstall_observation` 命令？**保留**（历史观察仍可用）。

**门禁（后端）**：`cargo fmt --all -- --check` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` **522 通过 / 0 失败 / 3 忽略**（新增 6 例：归属判定四形态、完整卸载跑固定命令并清残留、未确认拒绝、仅包体不碰运行态与 home、残留类别清理、残留三态）。

**尚未完成（下一轮）**：前端 `SettingsRoute.vue` 卸载弹窗按定稿原型重做（范围两级 / 清单 / 备份检测 / 确认常驻 / 进度动效 / 明细框固定高度 / 无滚动条 / 残留核验卡片），以及前端 typecheck / vitest / build / `audit:browser` 与原型门禁复跑。


### 19.26 实现（前端）：设置页卸载弹窗按定稿原型重做（2026-09-28，TASK-157）

**入口**：运行来源动作行改为——`更新（官方 ocx update）`仅托管安装时出现；**`卸载`（danger，无省略号）在来源 ≠「未解析」时常显**，与 `UI规范` §18.1 一致。

**弹窗（对齐 §18.4 / 定稿原型）**：

- **范围两级**：`完整卸载（推荐）` / `仅移除包体与入口`，各带一句后果说明。
- **将移除的对象**：折叠区列出包体 / 入口（完整卸载时追加运行态），并显示**将执行的固定命令**（外部包体 `npm uninstall -g @bitkyc08/opencodex`；完整卸载的 `ocx uninstall`）。
- **形态无法确认**：`removable=false` 时显示原因（`data-testid="uninstall-not-removable"`）并**禁用主行动**。
- **备份只提醒、不阻断**：未检测到可恢复备份时琥珀提示 + 默认勾选「卸载前自动生成备份」；检测到则提示复用并默认不再生成。
- **数据清理为完整卸载专属**：选「仅移除包体与入口」时禁用并注明。
- **确认项常驻**：`.wiz-ack-sticky`（`position: sticky`）贴在弹窗正文底部；未勾选时主行动禁用并给出可读原因（`data-testid="uninstall-ack-hint"`）。
- **进行中**：转圈 + 进度条流光（`prefers-reduced-motion` 关闭）+ 计划步骤；**完成后换成后端返回的真实步骤**（`ok` / `skipped` / `failed`）。
- **官方输出明细**：`officialOutput` 渲染进 **固定高度 132px** 的明细框（`.wiz-console-fixed`，超出框内滚动）。
- **残留核验卡片**：逐项 `已清除 / 残留 / 未知（未确认清除）`，并显示备份 id 与备份目录（可复制）。

**契约同步**：`features/runtime/api.ts`（`UninstallScope = body | full`、请求 `autoBackup` / `cleanData` / `confirmation`、方案与结果 DTO、`planRuntimeUninstall`）；`features/runtime/store.ts`（`uninstallPlan` / 加载态 / 错误）；`stores/app.ts`（`loadUninstallPlan`，卸载后同时读官方观察与状态快照）；`styles/base.css` 新增卸载弹窗样式（类名与安装向导的细进度条区分，互不覆盖）。

**门禁（前端）**：`vue-tsc --noEmit` 通过；`vitest --run` **66 文件 300 例**（新增 `tests/settings-uninstall.test.ts` 3 例：方案渲染 + 备份只提醒 + 未确认禁用与原因、勾选后按完整卸载请求执行并渲染真实步骤与残留、`removable=false` 禁止确认）；`vite build` 通过；`audit:browser` **209/209、0 未捕获错误**。**原型侧同步复跑**：jsdom **336/336**、浏览器 **443/443**（`runtime-source` 115），0 未捕获错误。

**未完成**：制品的真机屏幕复核（隔离实例点开卸载弹窗走一遍）本轮未做——按需在下一步补。


### 19.27 验证（前端）：卸载弹窗浏览器审计夹具与用例（2026-09-28，TASK-158）

**目的**：把 §19.26 的卸载弹窗纳入**浏览器层审计**（此前只有 jsdom 组件测试），让「入口常显 / 范围两级 / 将执行命令 / 备份只提醒 / 确认门禁 / 无滚动条 / 完成态 / 固定明细框 / 残留核验」在真实浏览器与真实布局下可回归。

**审计夹具（`src/dev/auditHarness.ts`，`?runtime=stub`）**：只改内存态、不连真实端点、不执行任何删除。场景固定为「自动发现到的 npm 全局安装」——来源 `discovered`、`removable=true`、未检测到可复用备份（故默认勾选自动备份）；`loadUninstallPlan` 返回真实形状的方案（将移除清单 / 运行态对象 / 残留候选 / 两条固定命令），`uninstallRuntime` 返回真实形状的结果（步骤 `ok` + `officialOutput` + 三项态残留）。仅在带 `runtime=stub` 参数时安装，生产构建不含该夹具。

**浏览器用例（`qa/audit.browser.mjs`「交互·卸载弹窗（Revision 11）」）**：非托管来源也常显入口且文案为 `卸载`（无省略号）→ 打开弹窗 → 范围两级 → 备份文案为「未检测到可恢复备份…不阻断卸载」→ 折叠区（`textContent` 取全集，与原型一致默认折叠）含 `npm uninstall -g @bitkyc08/opencodex` → 未勾选时主行动禁用且原因可读 → 弹窗 `scrollHeight ≤ clientHeight`（**无滚动条**）且确认条落在可见区内 → 勾选后可用 → 点确认走到完成态、步骤含「执行官方 ocx uninstall」、残留核验含「已清除」→ 明细框高度 **132px**。

**样式**：`.modal.wiz-uninstall` 放宽高度上限（`min(88vh, 100% - 24px)`），使卸载弹窗在默认窗口下无需滚动条即可容纳全部内容。

**门禁（前端）**：`vue-tsc --noEmit` 通过；`vitest --run` **66 文件 300 例**；`vite build` 通过;`audit:browser` **225/225、0 未捕获错误**（较 §19.26 的 209 增 16 条，来自新用例）。`dark·overview` 单次长跑偶发超时可复现通过（隔离复跑 3/3），属负载抖动，非缺陷。


### 19.28 验收（打包）：卸载弹窗制品内原生复核 + 后端格式修复（2026-09-28，TASK-159）

**后端格式修复**：`cargo fmt --all -- --check` 报 `modules/runtime/uninstall.rs` 测试模块两处换行/缩进偏差（§19.25 交付后遗留）。`cargo fmt --all` 就地修复（语义不变），复跑 `fmt --check` 通过；`clippy -D warnings` 通过；`cargo test --workspace --features integration-test` **522 通过 / 0 失败 / 3 忽略**。

**打包**：`npm --prefix apps/desktop/ui run build` + `tauri build --target aarch64-apple-darwin` 由**当前源码**产出两件制品——`.app` 可执行文件 `sha256 3cfcdabb…`、`.dmg` `sha256 ded7c418…`。

**制品内原生复核（隔离实例，不做破坏性动作）**：以 `open -n --env HOME=/tmp/ocx_uni_sbx -a "<.app>"` 起隔离实例（沙箱 `~/.local/bin/{node,npm,ocx}` 符号链接 ⇒ 来源 `discovered`，入口常显），用自写 `CGEvent` 点击 + 窗口定点 `screencapture` + `Vision` OCR 回读，走完**确认前**全链路：

- 运行来源动作行常显 **`卸载`（无省略号）**（`00-…png`）；
- 弹窗：标题 `卸载 OpenCodex`，`按当前运行来源 /tmp/ocx_uni_sbx/.local/bin/ocx 卸载`，**范围两级**（完整卸载 / 仅移除包体与入口）、折叠「将移除的对象：包体・入口·运行态」、**备份只提醒不阻断**（`未检测到可恢复备份…不阻断卸载，但建议先备份`）、`卸载前自动生成备份` 与 `清空 OPENCODEX_HOME 残留`（`01/02-…png`）；
- **确认门禁**：未勾选时给出原因、点主行动**无反应**（停留配置视图），勾选后原因消失、主行动底色由白（置灰 `(255,255,255)`）变浅红（danger `(244,231,230)`）（`03/04/05-…png`）；
- 取消关闭回到「安装配置」（`06-…png`）。

**安全与隔离**：仅验证到确认前，**未点击主行动执行**（沙箱 `ocx`/`npm` 为真实符号链接，完整卸载会触发真实 `ocx uninstall` 与真实全局 `npm uninstall -g`，按设计不执行）。复核前后真实 `manager-state/runtime.json` `sha256` 不变（`3c39d44d…`）、真实钥匙串 `ocx.dav` 计数不变（`1`）、沙箱 `runtime.json` `history` 为 `0`；沙箱实例随后 `kill`。证据目录 `.adg/work/imp03-04-execution/evidence/app-smoke/uninstall-native-158/`。


### 19.29 缺陷修复：卸载后状态不再沿用「可启动」+ 官方卸载顺序（2026-09-28，TASK-160）

**用户反馈**：「卸载后，其他按状态判断的地方不会更新，界面还直接允许启动。」

**根因一（状态陈旧）**：真实运行来源已 `unresolved`（`ocx` 入口消失）时，官方状态采集必然失败；`StatusCollector::refresh_at` 一贯「失败保留上一份状态」，于是把卸载前那份 `stopped`（FZ-06 视为可启动）沿用了下来——概览、托盘等消费同一份快照的地方都停在旧事实。

**修复一**：

- `StatusSource` 新增 `resolved()`（默认 `true`）；`OfficialStatusSource` 在**未解析出 `ocx` 或入口已不存在**时返回 `false`。
- `StatusCollector::refresh_at` 区分两类失败：来源未解析 ⇒ 置 `runtime=not_found` 并清空 `facts/port/pid`（确定性「未发现安装」，与 `FZ-07`「未发现安装 → `not_found`」一致）；**瞬时失败**（超时 / 解析失败）仍保留上一份状态。
- 前端 `installRuntime` / `uninstallRuntime` 终态补跑 `refreshEnvironment()`：环境发现（Node/npm/ocx）与运行来源是两套事实，卸载后环境卡 / 概览「安装形态」/ 环境门禁必须立刻不再显示「已发现」。

**根因二（官方卸载顺序）**：`execute_uninstall` 原顺序为「移除包体与入口 → 官方 `ocx uninstall`」。外部 npm 全局来源的入口（`~/.local/bin/ocx`）正是 `ocx` 本身，先跑 `npm uninstall -g` 会把它删除，官方命令随后必然失败——真机 `runtime.json` 的 `history` 出现过 `uninstall/failed`（`target=/Users/ezio/.local/bin/ocx`）。

**修复二**：`execute_uninstall` 完整卸载改为「停代理 → 备份 → **官方 `ocx uninstall`** → 移除包体与入口 → 清空 `OPENCODEX_HOME` 非自有残留 → 残留核验」；`UninstallScope::Full` 文档、UI 文案（范围说明与确认条）与**原型 `index.html`**（`uninstallSubsteps`、范围说明、确认条）同步。

**契约 / 规范回写**：`契约字段.md` §5 补「未发现安装」刷新规则；`IMP-01` §3.2 FZ-08 刷新表补同一条；`IMP-01` 变更记录一行。

**门禁**：后端 `fmt --check` ✔ / `clippy -D warnings` ✔ / `cargo test --workspace --features integration-test` **524 通过 / 0 失败 / 3 忽略**（新增 `unresolved_source_falls_back_to_not_found`、`transient_failure_preserves_last_state`，并在完整卸载用例加**命令顺序**断言）；前端 `vue-tsc` ✔ / `vitest` **66 文件 300 例**（卸载用例补 `discover_environment` 断言）/ `vite build` ✔ / `audit:browser` **225/225**；原型 jsdom **336/336**、浏览器 **443/443**。

**制品内原生复核（隔离实例，无破坏性动作）**：当前源码 `tauri build` 重建 `.app`/`.dmg`；沙箱实例（假 `ocx`）里入口存在时概览为「未运行 + 启动 OpenCodex」，移除入口并刷新后变为「**未发现** + 仅刷新状态（无启动按钮）」。证据 `.adg/work/imp03-04-execution/evidence/app-smoke/status-after-uninstall-160/`。

### 19.30 状态与流程收敛专题（2026-09-28，原 TASK-161 草案修订）

**需求方向已确认，原型/实现待完成**。用户由概览状态告警与占用问题进一步明确：“一个主线状态，然后每个节点有支线小流程操作”“所有状态做减法”“报错统一收敛”，并授权回写文档与事实。对应 DMD Revision 12（REQ-31 / AC-15）。

原草案的本机 at-risk 样本仅为历史观察，不能推出安装后必然如此；原“三层＋四类事件”方案不足以处理全部流程和一致性。原五项待决策清单不再作为方向确认阻断；具体字段迁移、视觉尺寸与每域更新时延仍待设计验证。

**专题与审计输入**：[状态呈现与全局实时状态机制专题](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/02-方案草案/状态呈现与全局实时状态机制专题.md) §3～§6；包含全功能盘点、S-01～S-15 静态发现与验收场景。

**实施约束**：先修正事实与操作结果，再删除重复副本；统一错误和刷新入口，验证后才移除点补。保留必要的取消、部分完成、待确认、待重启等差异及现有 CLI 兼容，不引入包揽所有领域的大 store。主线标签、支线步骤与详情布局以项目核心的唯一规则为准，不在此复制第二套表。

**事实回写**：DMD §3 / REQ-31 / AC-15；`数据与状态.md` §2.7、§5.8；`UI规范.md` §14、§19；`系统架构.md` §6.1；`契约字段.md` §5.1；领域模型、能力地图、核心目录索引及 IMP-01 §3.2 的对应引用。

**本轮边界（§19.30 首版）**：仅修订 Markdown；没有修改源码、原型或治理运行对象，没有关闭原 TASK、创建验收证书或宣告整改通过。静态审计不能替代代码/原型/制品验收；历史门禁数字不适用于本次新要求。

### 19.31 现行原型收敛与验证 + 生产实施交接（2026-09-28，TASK-162）

**范围**：依据 DMD Revision 12（REQ-31 / AC-15）与状态专题 §3～§6，把「主线状态＋节点操作支线＋统一报错」落到**文档规范与现行原型**。**仅改文档、原型与原型测试**；未改生产代码（`apps/desktop/**`）、未操作真实配置、未发布、未标记生产实现完成。

#### 19.31.1 原型改造（`01-需求分析/05-原型/原型/index.html`）

| 收敛目标 | 原型改动 |
|---|---|
| 一条运行主线 | 新增 `MAINLINE` / `STATE_MODEL`，把 mock 枚举投影为「待接入 / 未运行 / 运行中 / 正在确认」；主线标签不被操作或问题抢占 |
| 节点操作支线 | 摘要卡新增当前支线行（待确认 → 执行 → 核验 → 结果）；`starting`/`pending` 进行态、`starting_failed` 归结果并标失败色调 |
| 问题与主线分离 | 新增问题槽；仅持续问题（`external_takeover` / `unreachable`）进「待处理摘要」并在离开时解除；`at_risk` 单独不制造故障 |
| 报错统一收口 | 删除按枚举的固定告警/稳定态 Toast（`STATE_ALERT_NOTIFICATIONS` / `stateAnnounce` / `suppressNextStateAnnounce` 机械播报），改由问题生命周期与操作结果触达 |
| 概览简略＋详情 | 摘要卡与运行详情合并为同一张卡（同块内 `<details>`）；环境/路径/保护进详情；未运行不展示端口 |
| 动作同源＋做减法 | 主要操作直接露出，低频动作收进「更多」；`actionsByState` 仍为单一来源 |
| 统一操作结果 | 新增 `OP_RESULTS` 与操作结果测试器：成功 / 部分完成 / 失败 / 已取消 / 结果待确认 / 待重启，各有真实结论 |
| 状态测试器做减法 | `#scenarios` 按「主线节点 / 节点支线 / 持续问题」重新分组（保留 `data-state` 兼容既有测试） |
| 冲突不误报 | WebDAV 冲突态连接列由「已同步」改为「有未处理冲突」 |

**评审修正（2026-09-28，用户批注，TASK-163）**：初版摘要卡出现三处观感缺陷并已修正——① `display` 覆盖了 `[hidden]`，空的操作结果块渲染成长条黑框 → 补 `.ovb-*[hidden]{display:none}` 隐藏语义；② 空闲运行仍占一行「运行正常」，与主线标签重复 → 第二行只在「进行中 / 结果 / 待办」时出现；③ `at_risk` 改为行内中性说明（非告警），`not_found` 支线改为「选择接入方式」，去除与主线重复的措辞；并去掉结果标题与状态标签的重复。修正后重跑 `state-announce.test.mjs`（43 断言）、`state-convergence.browser.mjs`（130 检查）与既有原型回归测试，全部通过；修正后截图见 `05-原型/文档/截图/原型-状态收敛-20260928/`。

**测试器面板收敛（2026-09-28，用户批注，TASK-164）**：右侧「原型注释与测试器」的 mock 状态切换器统一按新规范分组与命名——托盘状态分「进程事实 / 应用内结论（托盘一律显示未运行）」；WebDAV 分「配置·连接事实 / 同步支线（冲突归具体问题，不写已同步）」；环境前置条件分「检测中 / 前置缺失（短路） / 通过」；连同已收敛的运行状态测试器与统一操作结果，面板全部切换器都按主线/支线/问题/事实归类。保留全部 `data-*` 钩子，既有测试不变。

**评审修正（2026-09-28，用户批注，TASK-165）**：① **WebDAV 状态切换器失效**：`renderWebdav` 的早退守卫仍检查历史 id `#syncCard`（现宿主为 `#syncActions`），导致整个切换器「点不动」——改为 `#syncActions` 守卫后，切换会同步更新概览 WebDAV 卡的连接/冲突列与动作。② **概览环境门禁卡片**：由「标题＋3 条全宽检查行＋命令＋动作＋脚注」的高大卡片改为「标题行（状态点＋标题＋标签）＋一行状态 chip（Node.js / npm / OpenCodex）＋命令＋动作＋脚注」的紧凑卡片；运行环境缺失时的「安装 OpenCodex / 导入离线包 / runtime/opencodex」文案与动作保持不变。修正后全量原型回归通过（含 `runtime-source.browser.mjs` 与 `state-convergence.browser.mjs`）。

**设计候选（2026-09-28，用户批注，TASK-166）**：用户希望用 **SVG 形象动画**表达状态、实时响应，替代占用大的颜色块。产出可运行候选与分析 `05-原型/候选/2026-09-28-状态形象动画/`（内联 SVG + CSS 动画 + `data-state` 单一驱动 + 实时驱动/stale/reduced-motion 开关；分析含映射表、技术选型、去抖、新鲜度、性能、可访问性、落地顺序与待决点）。**仅为候选与文档，未并入现行原型、未改生产代码、未冻结视觉**；定稿后并入本 §19.31。

#### 19.31.2 验证结果（原型层）

| 测试 | 结果 | 说明 |
|---|---|---|
| `qa/state-announce.test.mjs`（重写） | 43 断言 / 0 FAIL | 主线投影、问题生命周期、稳定态不播报、支线结果、统一结果语义、动作同源、启停结果不叠加说明句、空闲不重复占行 |
| `qa/state-convergence.browser.mjs`（新增） | 130 检查 / 0 FAIL / 0 console error | 宽 1180×760 与窄 900×600：主线标签集合、问题槽不裁切、摘要卡与页面不横向溢出、支线可见、详情可展开/关闭不溢出、「更多」弹出层不越出模拟窗口 |
| `qa/tray.test.mjs` / `log-categories.test.mjs` / `skill-detail.test.mjs` / `shim-progress.test.mjs` | 16 / 112 / 135 / 43 全通过 | 回归无破坏 |

证据：截图 `01-需求分析/05-原型/文档/截图/原型-状态收敛-20260928/`；命令输出 `.adg/work/evidence/task162-state-convergence/`。

#### 19.31.3 生产实施交接清单（未完成，禁止标记为已交付）

1. **事实与操作结果（优先）**：按专题 §4 的 S-01（采集失败仍投影 Live）、S-03（命令结果当观测）、S-04（进度卡无证据写「指令已返回」）、S-05（测试连接写 synced / 冲突写 succeeded）、S-06（卸载 failed 步骤被压成完成）、S-07（失败后页面拒绝重试）、S-08（`restart_required` 压成 boolean）、S-09（取消按失败记）逐项修正；先修假成功、无法重试与陈旧事实，再删重复状态。
2. **字段与协议冻结**：按 `契约字段.md` §5.1 冻结事实快照新鲜度/代次、操作结果（步骤证据、部分完成、生效条件）、问题/错误（稳定原因 + 脱敏详情 + 恢复动作）、动作能力、变更事件；保留 `AppErrorPayload{code,message}`、三维枚举与现有事件兼容，不做破坏性迁移，不把数字码去重写成已完成。
3. **错误收口（S-10）**：前端不再用正则解析英文 message 猜测原因，也不在无依据时声称「未修改／已保留原配置」；结构化原因落地，回滚失败/部分写入/待确认如实呈现。
4. **刷新与一致性（S-11）**：初始化先订阅再补快照；在途合并 + 变更排队补拉；来源/数据根切换使旧代次失效；采集失败保留旧观测并标新鲜度（S-01）；替代链路验证前不得删除既有补刷。
5. **各域接线**：概览/托盘/设置/扩展/同步/更新/数据根/诊断/通知按同源能力与 §5.8 收口（S-02/S-12/S-13/S-14/S-15）；通知由操作/问题生命周期派生，不以运行枚举为唯一触发。
6. **入口一致性**：概览、设置、托盘、原生菜单消费同一动作能力与限制原因，后端执行前复核；独立支线不被全局 busy 锁死。
7. **制品验收**：真机与制品内原生复核、故障注入、每域更新时延测量、键盘与原生面板恢复弹窗避让（专题 §6）——**全部未执行**。

> 完成定义：仅当上述 1～7 均在生产代码实施并通过制品验收、且验收证据齐备时，方可宣告 AC-15 生产验收通过。本节的文档与原型通过**不能**替代生产实现验收。

### 19.32 概览定稿 A：状态形象 + 无框居中布局（2026-09-29，TASK-168）

**范围与基准**：以**已确认的概览原型**为验收基准——候选 `05-原型/候选/2026-09-28-Logo本体形变/`（`index.html` 动效研究页 v08 + `overview.html` 融合 A 版 + `界面融合方案.md`）。对应 DMD Revision 13（`REQ-31` / `AC-15` / `AC-16`）与 `UI规范.md` §25。**先统一需求/UI/状态语义与验收口径，再分阶段开发**。

**边界（未经单独确认不做）**：不发布、不执行高风险迁移、不做全局同步；不改 `RuntimeState` 枚举与后端契约，不做破坏性字段迁移；不引入模拟数据或第二状态源。

#### 19.32.1 阶段计划

| 阶段 | 内容 | 交付 | 状态 |
|---|---|---|---|
| 0 统一口径 | DMD Rev 13（`AC-16`）、`UI规范.md` §25、`数据与状态.md` §2.7 补记、本节计划 | 文档 | 完成 |
| 1 状态形象 | `features/runtime/motion/`（`logo.ts` / `states.ts` / `projection.ts` / `scene.ts`）+ `RuntimeMotionMark.vue`：三结构 A/B/C、多光源背景、幅度 1.25、任意切态平滑过渡 | 组件 | 完成 |
| 2 布局与弹窗 | `routes/OverviewRoute.vue` 重写为无框居中固定 300px 状态区（+ 58px 环境摘要行）+ 运行详情弹窗 + 无边线环境摘要与环境弹窗；底部三卡保留 | 页面 | 完成 |
| 3 验证 | 前端单元测试（投影/几何/渲染）＋ 修订旧断言 ＋ 浏览器门禁（1180×760 / 900×600 × 深/浅）＋ 逐项对照原型 | 证据 | 完成 |
| 4 收口 | 结果/证据/遗留项回写（本节 + `docs/README.md`） | 文档 | 完成 |
| 5 待办 | `AC-15` 其余各域（§19.31.3 清单 1～7）、制品内真机原生复核 | 后续 | 未实施 |

#### 19.32.2 生产实现（`apps/desktop/ui/`）

| 文件 | 作用 |
|---|---|
| `src/features/runtime/motion/logo.ts` | 品牌 Logo 原始几何（云形 `d` + 原始镂空 + 变换）；中心 `(12,12)`、基半径 3.5、节点半径 1.18 |
| `src/features/runtime/motion/states.ts` | 九态演示模型（结构参数、三色配色、节奏）+ 纯几何 `computeGeometry`；`STRUCTURE_AMPLITUDE = 1.25` |
| `src/features/runtime/motion/projection.ts` | `projectMotion(runtimeState, fresh, problem)`：把既有运行主线/新鲜度/持续问题投影为形象状态；**不新增状态机** |
| `src/features/runtime/motion/scene.ts` | 渲染引擎（命令式、无依赖）：阻尼追踪 + 连续相位 + 独立光束角度；减少动态/暂停/隐藏页覆盖全部动画层 |
| `src/features/runtime/components/RuntimeMotionMark.vue` | 纯装饰层（`aria-hidden`）组件，装配背景与形象；枚举九态 |
| `src/routes/OverviewRoute.vue` | 无框居中状态区（固定 300px：168/62/54）＋无边线环境摘要行＋运行详情/环境弹窗；底部原三卡不变 |
| `src/styles/base.css` | `.motion-*` 样式（无框、固定高度、光场 412px、环境摘要、三段式弹窗） |
| `src/dev/auditHarness.ts` | 新增 `runtime=<state>` 夹具（只写内存快照），支撑九态可重复观测 |

**接线与真实性**：形象状态 = `projectMotion(app.runtimeState, fresh=true, problem=external_takeover|unreachable)`；主线文字、动作、端口/版本/路径、环境三项检查全部取自既有 store 与后端契约，**不新增状态来源、不使用 mock**。`fresh` 缺省为真，`stale` 只在明确判定过期时才可能出现（真实旧响应判定未实施，见遗留项）。

**原型对照（逐项）**：布局（无框居中/居中顺序/底部三卡）✅；固定高度（300 舞台 + 58 环境行 + 412 光场）✅；结构幅度 1.25 ✅；九态映射 ✅；任意切态连续过渡 ✅；多光源背景（三光团 + 一体光束 + 近地投影，低透明度）✅；主线详情弹窗 ✅；无边线环境摘要 + 环境弹窗（三项顺序/命令/安装入口）✅；暂停/减少动态/隐藏页 ✅。**差异**：候选以透明 iframe 复用研究页、并带「对照原始 Logo / 随机切态 / 强度滑杆」等评审控件；生产按 §25 抽取为原生组件、**不带评审控件**（幅度固定 1.25、无随机切态），符合定稿要求。

#### 19.32.3 验证结果

| 门禁 | 结果 | 说明 |
|---|---|---|
| `npm run typecheck`（`vue-tsc --noEmit`） | 通过 | 无类型错误 |
| `npx vitest run` | **319 通过 / 0 失败**（全量） | 含新增 `tests/overview-motion.test.ts`（19：投影 11 映射 + 问题/过期/新鲜度 + 常量 + 几何 + 渲染 + 概览集成） |
| `tests/overview-status-announce.test.ts`（修订） | 通过 | 断言无框状态区无常驻事实行、主线详情可达、无环境大卡 |
| `node qa/audit.browser.mjs`（全量浏览器门禁） | **272/272 通过 / 0 页面错误** | 含新增「概览形象」组：1180×760 与 900×600 × 深/浅各 10 项（固定总高 368、舞台 300、动画 168、光场 412、Logo 140、环境三项、底部三卡、无横向溢出、无错误）+ 弹窗可达/可关闭 + `交互·概览主线详情入口` |
| 截图证据 | `.adg/work/imp03-04-execution/evidence/browser/overview-motion-*.png`、`overview-motion-运行详情弹窗.png`、`overview-motion-环境弹窗.png` | 宽窄 × 深浅 4 组 + 2 弹窗 |

> 约定：审计环境缺失时 `audit.browser.mjs` 以退出码 2 报 BLOCKED，不把「没跑」当通过；本轮环境齐备（playwright-core + headless chromium）。

#### 19.32.4 遗留项（未完成）

1. **制品内真机原生复核**：本轮验证止于开发服务器 + 无头 Chromium；Tauri 制品内 WebView 的帧耗、`mask`/渐变兼容、原生面板避让、真机宽窄窗口与深浅主题尚未复核。
2. **事实新鲜度落地**：`projectMotion` 的 `fresh` 目前恒为真；真正判定旧响应/超阈值过期（§4 S-01）未实施，「事实过期」形象投影路径尚无真实数据驱动。
3. **`AC-15` 其余各域**：§19.31.3 清单 1～7（假成功、无法重试、陈旧事实、原因丢失、部分完成/待确认/待重启压平、双 busy、通知派生等）仍未实施。
4. **性能实测**：`scene.ts` 为 CPU 参与的逐帧路径更新（非纯 GPU 合成），运行态与九态高频切态的真机帧耗未实测；候选建议「构建时固化采样点、仅动画一个概览实例」尚未采用。
5. **旧组件清理**：`features/environment/components/EnvironmentGate.vue` 不再在概览渲染（改由无边线摘要 + 环境弹窗承担），仅保留其单元测试；后续可评估复用或移除。

> 完成定义：本轮仅完成「概览定稿 A」的**生产实现 + 前端/浏览器门禁**，不宣告 `AC-15`/`AC-16` 制品验收通过；未发布、未迁移、未全局同步。

#### 19.32.5 本轮增量：那些差异与验收（2026-09-29 续）

**修复（对照已确认原型与项目规范）**：

1. **真实状态新鲜度接线**：`fresh = !(statusError && statusSnapshot.source === 'live')`。最近一次采集失败且此前是真实观测（`source=live`）时保留旧事实并标「过期」（主线归「正在确认」）；fixture / 未配置来源不冒充过期；进行中的观测/操作（`loading`/`starting`/`pending`/`stopping`）不被旧事实顶替。生产不再恒 `fresh=true`，也不伪造过期。
2. **「待就绪」主线**：`pending`（进程已起、待健康与端口核验）主线归**运行中**、说明「待就绪」，与原型 `STATE_MODEL.pending.main='running'` 一致；原实现误归「未运行」。同步补齐 `at_risk`（启动保护未启用）、`external_takeover`（外部 provider 接管）、`unreachable`（进程或端口不可达）的中性主线说明。
3. **环境检查顺序**：新增 `environmentChecksOrdered`，检查进行中只有当前项「检查中」、其后为「待检查」，未执行到的项显示「待检查」而非含糊的「未检查」（顺序 Node.js → npm → OpenCodex）；摘要行与环境弹窗同源。

**针对性测试**：`tests/overview-motion.test.ts` 扩展——主线逐态标签（含 pending→运行中、at_risk 中性）、过期接线（旧观测 + 采集失败）、环境顺序语义；浏览器门禁新增「概览形象·主线／新鲜度／环境顺序」用例组。

**门禁（全绿）**：`vue-tsc` 通过；`vitest` 323/323；`npm run build` 通过；浏览器探针 287/287、0 页面错误；后端 `cargo fmt --check` / `clippy -D warnings` / `test`（403+，0 失败）全部通过。

**制品**：`tauri build`（app + dmg）。`.app` 主二进制 sha256 `05dfaffb…`；`.dmg` sha256 `2f4c3de5…`（完整路径与哈希见 `.adg/work/evidence/overview-final/README.md`）。

**动画性能（③ 层，非制品内）**：无头 Chromium 实测运行/启动/待接入态各 180 帧，median 16.7ms、p95 ≤16.8ms、max 16.8ms（稳定 ~60fps、无掉帧）。**不等于制品内 WebView 帧耗。**

**未执行（不得记为通过）**：制品内真机 GUI 复核与制品内帧耗——本轮以隔离 `HOME` 启动制品二进制（进程正常起、建自有数据根，未触真实数据根），但本会话无窗口渲染（`System Events` 窗口数 0、`screencapture` 画面不变，疑似远程会话无窗口服务器合成），故**制品内关键流程/动画复核未执行**；`AC-15` 其余各域与发布/迁移/全局同步同样未做。

#### 19.32.6 制品内真机复核（2026-09-29 续 · 本次会话具备窗口服务器后）

**前提变化**：本次会话从无窗口服务器变为具备可用 GUI 会话（`WindowServer` 在跑、`launchctl managername=Aqua`、外接显示器 `SwitchResX4 - H27P22S` 在线）。据此补做 §19.32.5 的遗留项「制品内真机 GUI 复核」。**本轮无源码改动**，制品哈希与 §19.32.5 相同（`.app` 主二进制 `05dfaffb…`、`.dmg` `2f4c3de5…`）。

**复核方法（可复现）**：

1. 以**隔离 `HOME`** 直接启动制品主二进制（假 `ocx` 返回 `running`），**未触真实数据根**（复核后真实 `runtime.json` 未被写入）。
2. **像素证据**：`screencapture -D<display>` 抓对应显示器，按 AX 窗口几何裁剪窗口位图（`Format=None`）。本机窗口会落在主屏或外接屏，故按 `AXWindow.position` 判显示器。
3. **文字回读**：自建 `Vision` OCR 工具（`swiftc` 编译，`VNRecognizeTextRequest`，`zh-Hans`+`en-US`）回读界面文本，不依赖截图肉眼辨认。
4. **交互**：以 AX `perform action "AXPress"`（`System Events` 遍历窗口 `entire contents`）触发按钮；`cliclick` 在该多屏（外接屏为负坐标）环境坐标不可用，改全 AX 驱动。

**逐项结果（制品内，全部通过）**：

| 项 | 证据 | 结论 |
|---|---|---|
| 制品启动并渲染概览窗口 | 进程存活、`AXWindow` 1180×760（Finder=4/Obsidian=2 作对照，排除探针假阴性） | ✅ 渲染成立（推翻 §19.32.5「无窗口」的结论——当时为无窗口服务器会话） |
| 宽窗口 × 浅色 | `in-artifact-wide-light.png`（2360×1520，均值≈239） | ✅ 侧栏＋无框状态区（形象/主线「运行中」/说明/动作）＋无边线环境摘要行＋底部三卡 |
| 宽窗口 × 深色 | `in-artifact-wide-dark.png`（均值≈37） | ✅ 同上，深色配色 |
| 窄窗口 × 浅色 | `in-artifact-narrow-light.png`（1920×1520=960pt） | ✅ 窄布局（环境项内上下排版、动作换行），内容不缺失 |
| 窄窗口 × 深色 | `in-artifact-narrow-dark.png` | ✅ |
| 运行详情**弹窗** | `in-artifact-modal-detail-dark.png`；OCR 回读「运行详情 / 进程与就绪 / 来源与目录 / 打开数据目录」 | ✅ 三段式（标题／正文／底部动作） |
| 环境**弹窗** | `in-artifact-modal-env-dark.png`；OCR 回读「运行环境 / Node.js·npm·OpenCodex / 下一步 / `node -v && npm -v`」 | ✅ |
| 无边线环境摘要行 | 宽/窄截图 + AX 单按钮承载三项结论 | ✅ 常驻旧环境大卡已移除 |
| Logo 状态形象渲染 | `in-artifact-motion-mark-light.png`（云形轮廓＋内部三节点结构 A） | ✅ 形象层渲染成立 |
| 结构幅度固定 1.25 | `motion/states.ts` `STRUCTURE_AMPLITUDE = 1.25` | ✅（源码常量，不随窗口/状态变化） |

**三结构（A/B/C）制品内取证**：以假 `ocx` 驱动运行态，**逐态冷启动制品并截图**（隔离 `HOME`），导出 `in-artifact-structure-{running,stopped,takeover}.png`：

- `running`（`{"proxy":{"running":true,…}}`）→ 主线「运行中」→ **结构 A · 完整内核**（实心聚合内核）；
- `stopped`（`{"proxy":{"running":false}}`）→ 主线「未运行」→ **结构 B · 分离三球**；
- `takeover`（`{"proxy":{"running":false},"startup":{"status":"external-takeover"}}`）→ 主线「运行中 · 外部 provider 接管」→ **结构 C · 柔性连接**。

三者像素显著不同（两两 `maxdiff`≈741–759），多光源背景在各态可见。**制品内三结构与主线语义均与 §25 一致。**（注意：制品状态来源经 `env_clear` 传入，假 `ocx` 不得依赖外部命令读取状态文件——须用 shell 内建读取，`cat` 在受控 PATH 下不存在。）

**模态对照澄清**：原型 HTML 的「概览定稿（B+ 合并版）」曾以**页内折叠**表达运行详情；**UI规范 §25.3（2026-09-28 确认）** 已改为**运行详情弹窗 + 环境弹窗 + 无边线环境摘要**。本实现对齐 §25 与 /goal 明确要求（「状态详情弹窗」「环境弹窗」），非偏差。

**仍受限（不得记为通过）**：**制品内动画逐帧/帧耗与「任意状态平滑过渡」的连续观测**未取得干净样本——本会话为远程（RustDesk）画面，应用窗口在无头合成下会被后置窗口遮挡，`WKWebView` 保持 `document.hidden`（`motionActive = 概览 && documentVisible` 为假），rAF 被节流/暂停；虽逐态冷启动可稳定取到三结构静态帧，但**无法在制品内得到可信的连续帧时序与帧耗分位，也无法观测切态过渡过程**。制品内动画性能仍以 §19.32.5 的 ③ 层受控采样（~60fps，median 16.7ms）为对照，**不宣称已测得制品内帧耗**。

**证据目录**：`.adg/work/evidence/overview-final/`（`in-artifact-*.png` + `README.md` 复核方法）。

> 本轮范围：仅补做「概览定稿 A」的**制品内真机 GUI 复核**并回写；未改源码、未重打包；`AC-15` 其余各域、发布、迁移、全局同步仍未做。

#### 19.32.7 修复：概览形象动画从未推进（dt 恒 0）+ 制品内动画复测（2026-09-29 续）

**根因（真实缺陷）**：`features/runtime/motion/scene.ts` 的 `tick()` 末尾调用 `wake()` 续帧，而 `wake()` **无条件执行 `lastTime = 0`**。于是下一帧 `dt = lastTime ? … : 0` 恒为 **0**：`clock` 不前进，所有 `track()` 以 `dt=0` 收敛即原地不动 ⇒ **rAF 循环持续运行（空转耗 CPU）但画面完全静止**。

- 「任意状态平滑过渡」因此**从未生效**（非 `immediate` 的 `setState` 只改目标，逐帧缓动因 `dt=0` 不推进）；
- 只有冷启动 `setState(state, immediate=true)` 直接落位，故 §19.32.5/§19.32.6 的**逐态静态截图**看似「正确」，掩盖了缺陷。

**修正 §19.32.6 的误判**：上一轮据「rAF 空转 + 画面静止」推断为「远程会话 rAF 被节流」——**该判断不成立**。实测 rAF 一直在运行；画面静止的真因是上述 `dt=0`。此处更正。

**修复**：拆分「排程」与「起跑」——

- 新增 `schedule()`（只排程、**不动** `lastTime`）；`tick()` 末尾改调 `schedule()`，使 `dt` 连续；
- `wake()` 保留 `lastTime = 0`，仅用于**重新起跑**（首帧 / 暂停恢复 / 切态），避免长时间暂停后的时间跳变。

**回归测试**（`tests/overview-motion.test.ts`，受控帧时钟，**已实证修复前失败**）：

1. `rotates the inner core across successive frames instead of freezing at 0`——逐帧角度须严格递增；旧实现角度恒 0（`expected 0 to be greater than 0`）失败；
2. `eases a state change over multiple frames instead of jumping`——非 `immediate` 切态后首节点 `cy` 出现 >3 个中间值，证明**多帧缓动而非一帧跳变**。

**验证**：

| 层 | 方法 | 结果 |
|---|---|---|
| ③ 开发服务器（无头 Chromium） | 采样 `.motion-hero svg g[transform]` | 修复前恒为 `rotate(0 12 12)`；修复后 250ms 序列 **21°→36°→51°→…→142°** 连续推进 |
| ④ **制品内**（隔离 HOME） | `screencapture -l<窗口 id>` 逐帧采样（`running`） | **13/13 帧均变化**（mark 区域每帧差 110–138），多光源背景与三结构动画可见 |

证据：`.adg/work/evidence/overview-final/in-artifact-anim-frame-{00,06,12}.png`、`in-artifact-animation-sheet.png`。
**制品内逐帧性能**：以**临时** rAF 计数探针（构建后即刻移除，最终制品不含该探针、已核验 `dist` 无残留）读得页面 rAF ≈ **100 fps**（连续 5 次读数 100/100/101/100/101，即显示器刷新率下满帧）；另以 ScreenCaptureKit 抓流测得**交付**帧率 ≈21.6 fps——该值受**抓取管线**限制，**不代表应用帧率**。制品内主机 CPU（`ps -o time` 增量）：WebContent ≈16%、GPU ≈24%。③ 层 rAF 间隔 ~16.7ms。探针证据 `in-artifact-fps-probe.png`。

**门禁（全绿）**：`vue-tsc` 通过；`vitest` **325/325**（+2 回归）；`vite build` 通过；浏览器探针 **287/287、0 页面错误**；后端 `cargo fmt --check` / `clippy -D warnings` / `test`（403+ / 0 失败）。

**新制品**（`tauri build`）：`.app` 主二进制 sha256 `d4f2ff2d457ab8074092b3edb6d8ca7c1ef11ba96819fa2ac0abb2356cc3bb42`（**可复现**：重编译后逐字节一致，是制品的稳定身份）；`.dmg` sha256 **随打包批次变化**（`bundle_dmg.sh` 非确定性），最近一次为 `754d976caee027ea163335cc177289a59fed0b9796dfa9313f2c4a547ad36c2a`。

> 本轮范围：修复概览形象动画推进缺陷 + 回归测试 + 制品内复测 + 重打包；未发布、未迁移、未全局同步。

#### 19.32.8 修正：at_risk 主线口径与悬浮幅度（2026-09-29 续 · 用户缺陷报告）

**用户报告**：①「动画效果并没有读取状态，无论启动停止都是运行中」；②「Logo 浮动的动画幅度太小，不仔细观察都看不出来在动」；③ 编译路径须遵守既有规则。

**① 根因（真机定位，非推测）**：本机真实 `ocx status --json` = `proxy.running:false` + `startup.status:"at-risk"` ⇒ 后端 `fold_proxy_runtime` 折叠为 **`AtRisk`**（该状态的定义就是「代理**未在运行** + 官方 startup at-risk」）；而前端 `motion/projection.ts` 把 `at_risk` 的主线写作「运行中」、形象映射到 **结构 A**。于是：**未启动**（`at_risk`）显示「运行中」，**启动后**（`running`）也显示「运行中」——启停无从区分，形象也不变。§19.32.6 曾照原型 `界面融合方案.md`「at_risk 单独仍映射运行」实现，属当时的设计选择，但真机上是**错的**：`at_risk` 的进程事实是「未运行」，不是运行。

**修法（与既有决策对齐）**：沿用「托盘只讲进程事实」（`runtime_label(AtRisk) = 未运行`，2026-09-24 决策）：

- `MAINLINE_BY_RUNTIME.at_risk = '未运行'`；`baseState('at_risk') = 'stopped'`（结构 **B 低位待命**，与「未运行」同结构）；
- 成因保留为说明句：`captionFor` 的 `stopped` 分支返回「启动保护未启用」；
- 审计夹具 `auditHarness` 的 `at_risk` 快照改为**未运行**口径（`running` 分支只对 `running`）。

**② 悬浮幅度**：`motion/scene.ts` 的 `floatTarget` 是 24 单位视图坐标下的**整体位移**；原 `running` 纵向振幅 `0.32` 单位 ≈ 1.9px，肉眼几乎看不出。按「肉眼可辨」重标定：`running` `1.0` 单位（≈11.7px 峰峰值）、`stopped` `0.5`、`confirming` `0.6`、`not_ready` `1.1`、`problem` `0.6`…；**结构幅度仍固定 1.25**（改的是整体位移，不是结构比例）。

**③ 编译路径**：按 README / 既有规则用 `--target aarch64-apple-darwin`（在 `apps/desktop/tauri` 执行 `npx @tauri-apps/cli@2.11.4 build --target aarch64-apple-darwin`），产物落 `apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app`。**更正 §19.32.6 / §19.32.7 的制品哈希记账**：那两节记的 `.app` 主二进制 `d4f2ff2d…` 实际对应 `target/release/…`（未带 `--target` 的那次构建），不是本路径产物；「重编译逐字节一致」只在同一 target 目录内成立。

**门禁（全绿）**：`vue-tsc --noEmit` 通过；`vitest --run` **327/327**（新增 2 条回归：`at_risk` 与 `running` 的主线/形象必须可区分；`running` 悬浮峰峰值 > 6%）；`vite build` 通过；浏览器探针 `qa/audit.browser.mjs` **287/287、0 页面错误**（「概览形象·主线逐态」中 `at_risk` 期望改为「未运行」）；后端 `cargo fmt --check` / `clippy --all-targets -D warnings` / `cargo test --workspace --features integration-test` 全绿。

**制品内复验**（隔离 `HOME` + 桩 `ocx`，`screencapture -l<窗口 id>` + Vision OCR）：

| 桩状态 | 主线 | 说明 | 主操作 |
|---|---|---|---|
| `stopped` | **未运行** | — | 启动 OpenCodex |
| `at_risk`（`proxy.running:false` + `startup.status:at-risk`） | **未运行** | 启动保护未启用 | 启动 OpenCodex |
| `running` | **运行中** | — | 打开面板 / 停止 / 重启 |

悬浮实测：`running` 制品内连续 30 帧采样，Logo 纵向包络 `156↔179`（2×）⇒ **峰峰值 ≈ 11.5pt**（修复前约 3.7pt）。证据 `.adg/work/evidence/overview-final/in-artifact-state-lineup.png`（未运行 / at_risk / 运行中 三态并排）与 `in-artifact-t_*.png`。

**新制品**（`--target aarch64-apple-darwin`）：`.app` 主二进制 sha256 `876c4a2b4bc9b313fae4faef6dbc693a62bc6056abf88ad1b59671ab5249b09f`；`.dmg` sha256 `5cd568caa8fda21334682ce2796b0526ff3bfb9d7136207e7fac9d7a82b808ff`。

**遗留项**：`external_takeover` 同样由「代理未在运行」折叠而来，主线目前仍作「运行中 · 外部 provider 接管」（§19.32.6 定稿口径，托盘那一层用「外部接管」标签）；是否与 `at_risk` 一并对齐为「未运行」待确认。

> 本轮范围：按用户缺陷报告修正概览主线口径与悬浮幅度 + 回归测试 + 制品内复验 + 重打包；未发布、未迁移、未全局同步。
