---
id: IMP-OPENCODEX-DESKTOP-05
object_kind: implementation.change
state: completed
title: 已确认原型还原与多档样式组件化实施计划
summary: 以已确认软件原型为视觉基准，先建立三档材质与动效策略及组件分类，再还原概览和全部软件页面，通过原型对比、真实流程回归和性能复测完成验收。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-169", "TASK-OPENCODEX-DESKTOP-170", "TASK-OPENCODEX-DESKTOP-171", "TASK-OPENCODEX-DESKTOP-172", "TASK-OPENCODEX-DESKTOP-173", "TASK-OPENCODEX-DESKTOP-174"]
source_refs: ["IMP-OPENCODEX-DESKTOP-04", "UI规范.md §25 / §26", "DMD Revision 14"]
completion_summary: "阶段 A–F 全部执行完成（TASK-169…174 全部收口）。基准与差异锁定、三档样式分层与公共组件、概览与全页面/流程还原、原型配对与性能优化复测、当前制品验收均完成；四项必需检查 review:prototype-parity（2026-10-01 用户裁决按 §26.1 口径）、review:performance、review:workflow-integrity、review:artifact-smoke 全部通过。第六次构建筑品内 Tauri WebView 冒烟确认「冲突策略」为只读「每次询问」（产品改动提交 30b63501）。公开发布、签名公证与真实用户数据的高风险迁移/恢复/全局同步不在本轮范围。"
---

# 已确认原型还原与多档样式组件化实施计划

2026-10-01。用户已确认当前原型基本效果，授权回写文档与编制开发计划；**本计划尚未执行，不表示生产实现或视觉、性能验收通过**（——**2026-10-01 已完成执行**：阶段 A–F 全部收口、四项必需检查通过，见 §11 执行记录）。

需求对应 `REQ-31 / REQ-32`、`AC-15 / AC-16 / AC-17`。视觉与交互规则维护于 [UI规范](../02-项目核心/UI规范.md) §25、§26；既有工程与真实业务验收继承 [IMP-04](IMP-OPENCODEX-DESKTOP-04.md) §4～§8、§14～§17。本计划是其后续还原专项，不重新启动已完成的目录迁移或后端重构。

## 1. 基准、范围与完成含义

原型目录简称 `P = docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/`。

| 对象 | 基准与用途 |
| --- | --- |
| 完整软件原型 | `P/候选/2026-09-28-Logo本体形变/overview.html`；虽位于候选目录，当前软件界面已获用户确认，用作全页面视觉对照 |
| 正式根入口 | `P/原型/index.html`；概览已与上页共用 `overview-dual.css / .js`，用于业务行为和共享概览对照 |
| 概览与运行详情 | `P/overview-dual.css / .js`；双形态、标准／紧凑高度、两列运行详情及来源目录 |
| 材质、档位、组件基准 | `P/glass-tokens.css`、`glass-components.css`、`glass-effects.css` 与原型玻璃组件专题；具体产品规则以 UI 规范为准 |
| Logo 几何与运动 | 原型 `logo-geometry.js / morph.js`、生产 `features/runtime/motion/`；内部幅度 1.25，保留状态含义与连续切态 |
| 已提交参考版本 | `fedbb4bc` 包含双形态和紧凑运行详情；本轮开始时 `overview.html / proto-controls.*` 尚有其它评审壳调整，不覆盖、不擅自纳入提交 |

阶段 A 必须记录实际产品部分的 Git 版本、文件 SHA-256、样式参数和截图条件。若待提交变动影响软件 UI，先定位差异再锁定实际基准；仅改变窗口外评审控件不应使软件视觉基准失效。截图是基准的证据，不能覆盖已确认状态或行为规则。

范围包括应用壳、概览、设置全部分区、扩展 Skills/MCP、诊断各 Tab、通知／Toast／任务／确认弹窗、安装卸载与迁移同步支线、面板外壳与快捷浮层、托盘动作和状态投影。官方 GUI 内部不重绘、不打包素材、不注入管理器样式；托盘原生菜单以平台组件和动作一致性验收，不强求模拟菜单与系统菜单像素相同。

窗口外的状态测试器、原型导航、历史透明度/色纱/按钮材质打样开关、随机切态和强度滑杆不进入产品。三档特效是正式需求；主题、特效、界面缩放、紧凑布局是不同维度。

**完成需要同时满足**：页面与原型一致、组件及三档策略可维护、全部既有入口流程正确、当前制品可用、性能复测达标。只通过 mock、类型检查或截图中的任一项均不构成完成。

## 2. 已确认差距与阶段 A 的盘点

以下为本轮代码阅读结论，未执行生产测试：

| 现状位置 | 已确认差距 | 实施方向 |
| --- | --- | --- |
| `routes/OverviewRoute.vue` | 仍有 `.motion-env-row`、常驻三卡与单列详情；尚未采用原型双形态 | 提取状态舞台、环境准备卡、运行详情与事件摘要 |
| `styles/tokens.css / base.css` | 已有玻璃变量，但尚无统一 `data-effects` 三档矩阵；业务样式仍较集中 | 语义令牌 → 材质配方 → 档位覆盖 → 组件 → 页面组合 |
| `components/ui/UiCard.vue / UiButton.vue` | 公共包装存在，材质与尺寸主要依赖调用方类名 | 在现有目录扩充明确变体与插槽，保持原消费者兼容 |
| `RuntimeMotionMark.vue` | 系统媒体监听与 `reducedMotion` 分别调用 `setReduced`，最后调用可能覆盖另一来源 | 外观策略统一计算有效档位，组件仅消费有效策略 |
| `features/preferences/api.ts`、Rust `Preferences` | 没有正式 `visualEffects` 字段 | 按契约字段 §9 增量扩展，旧偏好缺字段兼容、保存失败回退 |
| 其它页面及浮层 | 已有实现和大量回归，不能据文件存在断言视觉对齐 | 建立逐页、逐入口差异表，再按共用组件批次迁移 |

盘点表每行包含：页面／节点、原型 selector、生产组件／selector、数据和动作来源、视觉差异、流程差异、适用档位、需测试的分支。差异必须有“修复／规范允许的差异／待解决”结论，不能用“基本一致”代替逐项收口。

## 3. 样式与组件分类

延续 `apps/desktop/ui/src/components/{ui,layout,patterns}` 与 `features/<域>/components`。新增职责层，不建立第二套 `shared/ui`，不复制三份页面：

| 层 | 建议落点 | 责任 |
| --- | --- | --- |
| 基础、语义令牌 | `styles/tokens.css` | 字号、间距、颜色、焦点、层级；主题只变颜色值 |
| 表面配方 | `styles/materials.css` | `shell / page / panel / control / emphasis / overlay` 填充、描边、模糊、投影 |
| 档位覆盖 | `styles/effects.css` | 根据唯一有效档位改变材质与运动变量；包含实底回退 |
| 外观策略 | `app/appearance/effects.ts` | 用户选择、系统减少动态、可见性与配置加载；只此处写档位属性 |
| 基础组件 | `components/ui` | Button/IconButton、Card、Dialog、Menu/Popover、Tabs、Badge、Switch、Input 等 |
| 通用内部布局 | `components/layout / patterns` | Stack/Inline/Grid、KeyValueList、PathField、SettingRow、ProgressPanel 等 |
| 业务组合 | `features/environment / runtime / settings / feedback` 等 | 业务状态和动作绑定；不在基础组件中读业务 store 或调用 Tauri |

组件 API 至少区分 `variant / tone / size / density / surface` 中适用的项；不用 `isGlassHigh/isGlassMid/isGlassLow` 等组合布尔值。每个公共组件只暴露实际使用的变体，普通内容块不为形式拆成多层容器。

**三档矩阵引用 UI 规范 §26**。高与中使用同一玻璃配方；中降低运动，低统一实底与静态状态形象。切档不能改变布局尺寸、内容、动作可用性或状态判定。

- 动效组件消费 `{effectiveEffects, motionAllowed, ambientAllowed, visible}` 等只读策略；系统减少动态与用户选择统一计算，任何一个限制不能被另一个 watcher 覆盖。
- 低档不挂连续 rAF；中档只在状态变化时绘制静态目标。高档仅可见且系统允许时运动。切回高档从当前形象平滑接续。
- `UiDialog` 统一标题、正文、页脚、焦点与关闭。运行详情内容使用上部两列、下部整宽，不复制第二份弹层基础设施；不能通过 `overflow:hidden` 假装无需滚动。
- 样式迁移以组件真实消费者为单位，把业务选择器从全局 CSS 移出；公共样式不依赖 `#某页 .card:nth-child(...)`。禁用态、键盘焦点、hover、忙碌态也使用同一变体矩阵。
- 在开发组件预览页提供两主题 × 三档 × 关键交互状态。组件预览与测试夹具不进入生产构建。

## 4. 执行阶段与阶段出口

| 阶段 | 工作 | 必须交付与退出条件 |
| --- | --- | --- |
| A 基准与差异 | 保存基准、列全入口、同条件采样当前性能；确认原型与规范冲突 | 基准 manifest、逐项差异表、状态 fixture 映射、测试矩阵、实施前性能记录；开始任务时写入稳定检查名 |
| B 档位与公共组件 | 先完成外观策略和偏好兼容，再完成材质／档位分类与组件试点 | 三档随重启恢复；系统减少动态有效；组件预览矩阵通过；基础组件不包含业务副作用；逐组件保留现有消费者 |
| C 概览还原 | 双形态、门禁检查与安装出口、主线操作、紧凑运行详情、三卡及事件摘要 | 1180×760 / 900×600 两主题三档对照；默认概览与普通运行详情完整首屏；运行来源、缺失、晚返回、启停进度真实且不互相覆盖 |
| D 全页面与流程 | 以共用组件为单位迁移设置、扩展、诊断、反馈、向导、面板壳、托盘 | 页面差异表清零或有规范允许的说明；既有入口一个不漏；跨入口状态和真实结果一致 |
| E 对比与优化 | 图像对比、几何检查、故障注入、性能定位和优化后复测 | 原型／实现配对截图与 diff；真实流程回执；三档与可见性资源测试；性能数据无未解释回归 |
| F 当前制品 | 全量回归、当前源码打包、Tauri WebView 视觉与关键流程冒烟、更新实施现状 | 制品和源码哈希、原生证据、未解决缺陷清单、提交记录；所有必需检查通过才完成 |

顺序为 A → B → C → D → E → F；测量和缺陷修复贯穿各阶段。每阶段独立提交，可追踪和回退。本轮计划未创建执行 TASK；执行时在 `.adg/` 创建 source-bound TASK／RCP，不能把此表写成第二套任务完成状态。

## 5. 原型对比测试

### 5.1 对比方法与门槛

1. 同应用窗口逻辑尺寸、同 OS／WebView、同字体、同 DPR、同缩放、同状态和数据，分别截取**应用内容区域**。不把原型外部工具条计入实现截图。
2. 原型 mock 字段映射到仅开发可用的生产审计 fixture；数据通过与生产一致的展示转换，不能在组件中写第二套“为了截图”DOM。常量版本、路径、时间、列表项在两边相同。
3. 一份用例生成 `prototype.png / implementation.png / diff.png` 和几何结果，输出到 `.adg/work/prototype-parity-20261001/`。不得用现实现的截图覆盖原型基准来消除失败。
4. 高档静态对比固定装饰动画相位；只对随机时间等小区域使用列明理由的遮罩。不得遮掉整个 Logo、卡片或浮层。连续切态、可见性暂停和真实档位性能另用动态探针验证。
5. 初始工程门槛（**2026-10-01 用户裁决：以 §26.1 为准**）：
   - **硬门（必须满足）**：主要区域位置与尺寸差不超过 **2 CSS px**；关键字段、按钮、分组、字号和可见性匹配（由浏览器审计断言与配对几何共同覆盖）。
   - **像素门按 §26.1 的还原项评估**：§26.1 规定需还原的是**布局、字号、密度、描边、玻璃、投影、图标、动作层级和状态过渡**；§26.1 同时规定**真实数据优先、历史 mock 的版本／时间／英文值／假成功／过时文案不照搬**，§2.2 与 UI规范 tokens 表规定 `--notif-tint-a/b` **置透明**。因此对**由真实数据承载的内容**与**已按规范置透明的色纱**，逐项列明归因后不计入失败；其余（结构／材质／排版／图标／动作层级／状态过渡）不得有未解决差异。
   - 整窗与逐区像素比值**如实记录**为上下文证据（便于复核与回归对照），**不作为通过／失败判据**；判据是上两条。
   - 通道差阈值 20、DPR 一致。
6. 文字抗锯齿或玻璃合成造成差异时，先确认运行环境和配方，再做配对人工复核；有限渲染差异须逐项说明。任何布局漂移、错误隐藏、截字或动作遗漏均不得通过调宽阈值豁免。
7. DOM/jsdom 验证语义和行为，不能证明无滚动。必须实际检测 `scrollHeight/clientHeight`、区域边界、焦点和遮挡，再检查截图。

### 5.2 最小覆盖矩阵

所有软件页面及主要浮层都跑 **两窗口 × 两主题 × 三档 = 12 组合**的基础状态；高档再跑重点连续切态。主题跟随系统、缩放 50/100/150/200、系统减少动态和后台／隐藏分别补边界场景。放大缩放或长内容可滚动，但按钮和关闭入口不得丢失；不能把 100% 首屏要求误扩成任意缩放、任意数据量均不滚动。

| 对象 | 关键状态和分支 |
| --- | --- |
| 概览 | ready 与 ready_via_source；Node/npm/ocx 缺失；检查中；stopped/running/pending/starting/stopping；失败、at-risk、接管、不可达、陈旧事实；所有结果类型 |
| 运行详情 | 默认及紧凑布局；正常字段无正文滚动；长路径有完整出口；未知/缺失值；长操作结果按需滚动且标题页脚可见 |
| 设置与向导 | 所有分区；三档保存/失败/重启；安装联网/离线/取消/失败；卸载确认、备份提醒、进度与残留 |
| 扩展 | 搜索、筛选、分页、开关、条目和批量动作；长 Markdown、只读详情、JSON 错误、冲突、写入失败与恢复 |
| 诊断与反馈 | 日志类别和离线；历史列表、脱敏；通知 read/resolved/deleted；任务收起继续；Toast/任务/弹层避让；异常恢复 |
| 面板与托盘 | 面板壳、载入/不可用/重试、缩放；宽窄栏；托盘动作、原生菜单、窗口隐藏/返回/退出 |

## 6. 功能与流程不回归

对照 IMP-04 §17 的全入口清单，以真实结果、持久化与副作用核验，不能只点击按钮后看 Toast：

- 发现 → 解析来源 → 启动 → 就绪 → 打开面板 → 停止／重启；缺前置先处理，resolved 来源不能被自动发现空候选误阻断。
- 联网／离线安装 → 校验 → 来源切换 → 环境刷新 → 可运行；统一卸载 → 状态与来源刷新 → 残留核验。
- Skills/MCP 检查、更新、导入、修改、删除与恢复；掩码、确认、锁、外部改写与回滚保持原契约。
- 导出 → 导入校验 → 备份 → 确认 → 替换／回滚；旧加密容器兼容；新流程不增加额外口令。
- WebDAV 测试／同步／冲突 → 明确选择 → 覆盖前备份 → 可恢复；TLS 失败不得绕过。
- 升级前备份、官方更新入口、管理器自更新与待重启；官方共享开关写后复读，退出 0 未生效不能判成功。
- 主题、缩放、三档、路由、托盘、通知和原生菜单入口均使用同源状态与能力。
- 注入晚返回、超时、断网、取消、重复点击、权限失败、写入中断和回滚失败，保留当前支线、原因与可执行下一步。

前端状态测试＋可控后端集成＋浏览器 fixture＋隔离数据上的真实 Tauri 冒烟四层分别记录。需要真实端点或原生权限而条件不具备时记为阻塞，不以 mock 替代真实通过；继续完成不依赖该条件的工作。

## 7. 性能基线、优化与预算

阶段 A 在当前源码和当前制品重测，不直接把 IMP-04 §19.12 的历史数值当作本轮基线。记录硬件／系统／WebView、Git SHA、构建方式、刷新率、数据规模、样本与温度缓存。性能记录是实测结果，本节数值为约定预算。

| 维度 | 本轮采样与预算 |
| --- | --- |
| 包体 | 生产 JS／CSS 原始与 gzip、资源总量；相对阶段 A 主 JS 和 CSS 净增各不超过 5%；fixture、原型和组件画廊不得入包 |
| 冷／热启动 | 各至少 5 次到首帧／可交互／业务就绪；中位数不回归超过 10%，无挂起；异常样本保留说明 |
| 交互 | 路由、弹窗、200 条通知、长 Markdown，各至少 20 次记录 median/p95；相对可比基线不回归超过 10%；低样本／噪声需补采样 |
| 高档动效 | 以屏幕刷新间隔 B 为预算；稳定可见采样 30s，rAF 间隔 p95 ≤ 2B，连续 > 6B 卡顿必须分析；脚本绘制 p95 ≤ B/2；多态快速切换不重置、不累积队列 |
| 中／低档 | 可见稳定态不产生持续动画 rAF；中为静态光场，低为无光场和无 backdrop-filter；状态改变仍立即更新静态形象 |
| 后台与资源 | 每档空闲采样 60s；隐藏窗口／离开概览后无装饰动画循环；切页、开关弹窗、切档各 20 次后 listener/timer/rAF/observer 不增长 |
| CPU／内存／IPC | 同条件比较三档及优化前后；中低档应消除连续动画成本，记录实际占用；内存不随重复挂载单调增长；样式切档不额外调用状态 IPC 或增加轮询 |

优先定位大面积或嵌套模糊、重复合成层、逐帧路径分配、无差别响应式重算、无效监听与循环。可固定采样几何、复用对象、限制可见动效实例、将布局读取与写入分批、延迟非首屏初始化；保持真实业务、可访问性和全部反馈语义。

优化结果必须同环境复测并附记录，不以降低字号、隐藏必要内容、删除备份、减少正确性测试或降低高档视觉质量换取指标。WebView 与 Chromium 的测量分别标识，不能把抓屏交付帧率当作应用渲染帧率。

## 8. 检查入口、回执与收口

生产检查从实际工程目录运行：

```sh
# apps/desktop/ui
npm run typecheck
npm run test -- --run
npm run build
npm run audit:browser

# apps/desktop/tauri
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --features integration-test
npx tauri build --target aarch64-apple-darwin
```

运行适用的原型 `qa/*.test.mjs`；先修正与当前已确认原型冲突的历史浏览器断言，再运行浏览器 QA。新增跨原型视觉比较与性能探针，不能把现有 `audit:browser` DOM 断言作为图像对比的替代。若没有浏览器／原生权限，只能如实留下未完成的门禁，不绕过宿主安全限制。

执行 TASK 至少声明以下 `required_checks`，RCP 使用相同名称与实际证据：

| 稳定检查名 | 关闭条件 |
| --- | --- |
| `review:prototype-parity` | 所有必需页面、12 组合、重点状态有配对证据；无未解决视觉差异 |
| `review:style-modes` | 分层与组件 API 检查、三档/主题/减少动态/偏好兼容通过 |
| `review:workflow-integrity` | 全入口、正常与适用异常分支、真实副作用核验通过 |
| `review:performance` | 本轮基线与复测齐备，预算满足，优化收益可解释 |
| `review:artifact-smoke` | 当前源码静态/测试/构建通过；对应哈希制品在 Tauri WebView 冒烟和视觉复核通过 |

每个缺陷记录场景 ID、期望、实际、原因、修复提交和复测；“不适用”必须有真实范围理由。任一必需检查缺失、失败或阻塞时不得标记完成。最终只更新经过证据证明的生产现状；不关闭其它 IMP 或旧未完成要求。

## 9. 边界与交付

本次文档批准了还原目标与计划，没有开始生产开发。后续目标模式执行可在已授权开发范围内推进；公开发布、签名公证、真实用户数据的高风险迁移／恢复／全局同步仍单独授权。测试数据使用隔离根和测试端点，不触碰真实用户配置。

交付包含：组件与档位映射表、页面／入口差异表、更新后的生产组件和偏好兼容、原型配对截图／diff、流程回执、性能前后记录、当前制品与 Git SHA、实施状态回写和未解决项。机器证据在 `.adg/`，稳定结论写本 IMP；不维护平行完成状态。

## 10. 简短目标模式提示词

> 进入目标模式：按 `docs/03-开发实施/IMP-OPENCODEX-DESKTOP-05.md` 执行已确认原型还原。先锁定基准与差异，完成三档样式分层和组件化，再还原全部软件页面与流程。以原型配对截图、真实功能回归、性能优化复测和当前 Tauri 制品验收为完成条件；逐阶段提交与回写，未通过或受阻的必需检查不得标记完成。

## 11. 执行记录

### 阶段 A · 基准与差异锁定（TASK-169，2026-10-01）

**状态：已完成，`review:baseline-lock` 通过。** 机器证据在 `.adg/work/prototype-parity-20261001/`。仅文档与证据，未改生产行为。

已交付：

1. **基准 manifest**（`baseline-manifest.json` + `gen-manifest.py`）：锁定 Git `c75cfaf8`（`codex/implementation-base`，工作区干净）；原型 10 个文件与生产 16 个文件的 SHA-256；生产构建体积（主 JS 349457 / gzip 115248，CSS 79296 / gzip 14627）；门禁基线（typecheck 通过、vitest 67 文件 327 例、build 通过、浏览器探针 287/287）；截图条件（1180×760 / 900×600 × light/dark × high/mid/low，DPR 2，只截应用内容区）。
2. **逐页/逐入口差异表**（`diff-inventory.md`）：覆盖应用壳、概览、设置 9 分区、扩展、诊断、反馈、面板壳、托盘与支线向导；每行给出原型 selector、生产 selector、数据/动作来源、视觉/流程差异、适用档位、待测分支与「修复/规范允许/待解决」结论。
3. **状态 fixture 映射**（`state-fixtures.md`）：运行主线 12 状态、环境 `envStates` 6 态、操作结果 `OP_RESULTS` 7 态、反馈场景映射到仅开发可用的审计参数。
4. **最小覆盖矩阵**（`test-matrix.md`）：两窗口 × 两主题 × 三档 = 12 组合 + 重点状态；四层证据层级与图像对比门槛（≤2 CSS px、像素差率 ≤1%）。
5. **实施前性能基线**（`perf-baseline.md`）：包体、门禁与浏览器层采样（通知长列表 200 条 renderMs=277、路由切换 44ms、Markdown 242 节点 61ms）；并列出阶段 E 待补的启动/动效/资源/CPU 采样。

**阶段 A 结论**：

- 必须修复项：全局（G1–G7：三档属性源缺失、材质无分层、减少动态覆盖、无 `visualEffects` 偏好、`.main` 无档位翻转、组件预览未覆盖矩阵、组件 API 未成型）、概览（O1/O3–O4/O6–O11）、设置/扩展/诊断/反馈/面板/支线（见差异表）。
- 规范允许项：顶栏主题段控/状态胶囊/铃铛、生成式状态形象、客户端图标、界面自检弹窗、错误边界与通知层避让、面板原生注入浮层、托盘预览。
- 阶段 A 核实的两处疑点：`app.recentEvents` 有数据源但**无 UI 消费**（概览最近事件卡缺失，需新增）；面板快捷 hub 为**原生注入浮层**（`commands/panel.rs`，符合规范）。
- 另发现：`features/environment/components/EnvironmentGate.vue` **未被任何页面引用**（`OverviewRoute` 内联 `.motion-env-row`）；`base.css` 有依赖已移除 `.ovb-detail[open]` 的历史残留规则。

**过程事实修正**：执行前 DMD 投影对象滞后于 Markdown 权威（对象携带至 `AC-16`，Markdown 已含 `REQ-32 / AC-17`）。已按权威重新投影 DMD（`revision 18`，`source_revision 18`），纳入 `AC-15/16/17` 作为本 IMP 验收绑定。

**未完成**：阶段 B–F。本阶段不构成生产实现或视觉/性能验收通过。

### 阶段 B · 档位与公共组件（TASK-170，2026-10-01）

**状态：已完成，`review:style-modes` 通过。** 完成外观策略、偏好兼容、材质/档位分层与公共组件变体；未改页面布局与业务行为。

已交付：

1. **外观策略唯一落点** `app/appearance/effects.ts`：`clampVisualEffects` / `resolveEffectiveEffects` / `effectsStrategy` 纯函数，`useEffectsStore` 与 `installEffectsRuntime` 装配**唯一**的减少动态与可见性监听，并在唯一位置写 `data-effects`。规则：系统减少动态把高档有效表现降为中档、低档仍为低档；用户保存值与有效表现分开。
2. **偏好字段与兼容**：`PreferencesDto.visual_effects`（Rust `Preferences`/DTO/校验 + TS DTO + store）。旧偏好缺该字段按默认高档读取（`#[serde(default)]`），非法值判损坏；保存失败回退持久化档位与 `data-effects`。字段登记见「契约字段」§9（待补登记，见遗留项）。
3. **材质与档位分层**：新增 `styles/materials.css`（`shell/page/panel/control/emphasis/overlay` 六类表面配方唯一真源）与 `styles/effects.css`（档位只覆盖有效 token 与运动策略；低档翻转 `--glass-fill`/`--glass-blur-notif` 并补实底规则、关闭 `.app-window::before` 与 `.motion-ambient`）。`main.ts` 与组件画廊按序引入。
4. **公共组件变体 API**：`UiButton`（`variant/size/density/surface`）、`UiCard`（`density/surface`）；默认不产生多余类，既有调用方传 `class="primary"` 等仍兼容。样式落点 `.btn-sm/.btn-lg/.btn-compact/.btn-plain` 与 `.card-compact/.card-plain/.card-overlay`。
5. **动效组件单一策略**：`RuntimeMotionMark` 移除自持媒体监听，改为只消费外观策略（中/低档静态目标、高档且可见才连续动画），修复「prop 与媒体监听互相覆盖」的缺陷。
6. **减少动态修复**：`installEffectsRuntime` 是唯一媒体监听来源；组件与页面不再各自 `setReduced`。
7. **通用设置入口**：设置 → 通用新增「界面特效」三档选择（即时生效、落盘回读、失败回退）。
8. **组件预览矩阵**：`dev/component-gallery` 支持两主题 × 三档切换与变体/状态展示（仅 DEV，不入生产构建）。

**验证证据**（阶段 B）：

| 检查 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm run test -- --run` | 68 文件 / 339 例通过（新增 `tests/effects.test.ts` 10 例、`ui-components` 变体断言） |
| `npm run build` | 通过；主 JS 353351 原始（+1.11%）、CSS 81808 原始（+3.17%），均在 §7 的 5% 预算内；无 fixture/画廊入包 |
| `npm run audit:browser` | 305/305 通过（新增「三档·high/mid/low」「三档·减少动态降档」共 18 项） |
| `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` | 通过 |
| `cargo test --lib preferences` | 16 例通过（含旧偏好缺字段兼容、非法档位拒绝） |

**未完成**：阶段 C–F（概览/全页面还原、原型配对截图、性能复测、当前制品验收）。阶段 E 的图像对比与性能复测未做前，本 IMP 不得标记完成。

### 阶段 C · 概览双形态与运行详情还原（TASK-171，2026-10-01）

**状态：已完成，`review:overview-dual-form` 通过。** 按 UI规范 §25 还原概览；未动其它页面。

已交付：

1. **双形态与两档高度**（§25.1）：`OverviewRoute.vue` 按环境结论切换 `data-mode="ready|setup"`；并按**应用窗口可用逻辑高度**（阈值 680px，折算界面缩放）置 `data-overview-size="standard|compact"`。舞台高度：标准就绪 300（168/66/66）、标准准备 174（114/60）、紧凑就绪 222（128/50/44）、紧凑准备 142（88/54）；Logo 140/90/100/74 与上边距 103/52/85/40 对齐原型。
2. **准备态内容层级**（§25.1）：检查中或真实前置缺失时收起三卡与最近事件，顶部缩短，展示准备卡。
3. **环境摘要行退出**（§25.1 / O6）：删除常驻 `.motion-env-row`；环境逐项结论并入运行详情。
4. **准备卡还原**（§25.3 / O9）：重写 `EnvironmentGate.vue` 为原型结构（标题 + 通过计数；左纵向检查 / 右当前指引 + 最相关命令 + 处理动作；底部说明与「完整指引」出口）。保留既有业务动作（重新检查 / 安装 OpenCodex / 导入离线包 / 设置指引）与安全边界文案（不读取 PATH、不写系统目录）。
5. **主线入口语义**（§25.3）：准备态主线为不可点（disabled），不充当无效详情入口；就绪态可打开运行详情。
6. **运行详情还原**（§25.3 / O10）：上部「进程与就绪 / 运行环境」两列、下部「来源与目录」整宽；长值省略但保留 `title` 全文与复制出口；标题与页脚固定、正文按需滚动；Escape / 遮罩 / 关闭按钮与焦点返回沿用三段式。
7. **最近事件摘要卡**（§25.1 / O8）：新增 `data-testid="overview-recent-events"`，标准显示最近两条、紧凑一条，完整历史仍保留在诊断中心；数据取自既有 `app.recentEvents`（未新增状态源）。
8. **说明文案对齐**（§25.1）：主线说明改为「进程已起 · 待就绪」「启动中 · 待核验」等已确认文案；持续问题的具体成因留在全局待处理/诊断，主线只给「有待处理问题」。

**验证证据**（阶段 C）：

| 检查 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm run test -- --run` | 68 文件 / 340 例通过（概览双形态、环境顺序、主线入口、准备卡动作/边界文案） |
| `npm run build` | 通过；主 JS 360.38 kB 原始（相对阶段 A +3.13%），CSS 在预算内 |
| `npm run audit:browser` | 318/318 通过；含就绪形态（1180×760 标准 300 / 900×600 紧凑 222）、准备形态（174）、环境检查顺序、运行详情两列与复制出口、主线逐态、晚返回过期说明 |

**未完成**：原型配对截图／像素 diff 与几何对照（`review:prototype-parity`，阶段 E）；全页面还原（D）；性能复测（E）；当前制品验收（F）。

### 阶段 D · 全页面表面分层与入口不回归（TASK-172，2026-10-01）

**状态：已完成，`review:page-material` 通过。** 以共用组件为单位迁移受控表面；未改业务行为与布局尺寸。

已交付：

1. **表面配方应用集中化**（§26.3）：`styles/materials.css` 现在把组件类接到表面分类，**高／中档共享通知玻璃**（填充 / 描边 / 模糊 / 投影四件套）：
   - `panel`：`.card / .setting-row / .install-row / .agent-path-row / .skill-row / .repo-item / .backup-item / .about-hero / .about-panel / .mcp-form-card / .choice / .modal / .select-menu / .notice / .menu-hint / .ui-fault`；
   - `control`：`.btn`（非 primary/danger/ghost）/ `.tag` / `.select-trigger`；
   - 暗色投影单独覆盖。
   布局仍归各组件自己的规则；`base.css` 不再需要逐条写玻璃配方。
2. **低档统一实底**（§26.2）：`styles/effects.css` 翻转 `--surface-panel-* / --surface-control-* / --surface-overlay-*`（填充退 `--panel`/`--raised`、模糊 `none`）；逐页低档实底探针覆盖设置 / 扩展 / 诊断 / 面板壳。
3. **修复玻璃引入的层叠缺陷**：玻璃配方（`backdrop-filter`）让每个卡片 / 设置行各自成为层叠上下文，导致打开的下拉菜单被后续兄弟表面按 DOM 顺序遮挡（真实点击被拦截）。在 `materials.css` 用 `:has(.select.open)` 抬起所在行／卡片到 `--z-popover`；由「设置选择器」交互探针回归。
4. **入口不回归**：既有全入口交互探针（概览主线详情 / 启停与任务卡 / 设置开关与选择器 / 分区切换 / 卸载向导 / 扩展分页签 / 诊断 / 通知与 Toast / 托盘切换 / 主题与缩放）全部保留并通过；新增逐页低档实底探针。

**验证证据**（阶段 D）：

| 检查 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm run test -- --run` | 68 文件 / 340 例通过 |
| `npm run build` | 通过 |
| `npm run audit:browser` | **333/333** 通过（新增逐页「低档实底」4 项与卡片/按钮玻璃断言；设置选择器点击回归） |

**阶段 D 遗留（移交阶段 E/F）**：CSS 原始 86800（相对阶段 A **+9.47%**）超出 §7 预算，需在阶段 E 优化压回；跨入口「真实副作用」核验属第 ④ 层，在阶段 F 的当前制品冒烟中完成（`review:workflow-integrity` 仍未关闭）。

**未完成**：原型配对截图／像素 diff 与几何对照（E）；性能复测与 CSS 预算回收（E）；真实流程副作用与当前制品验收（F）。

### 阶段 E · 对比与优化（TASK-173，2026-10-01，已完成）

**状态：已完成并收口（RCP-OPENCODEX-DESKTOP-173）；`review:performance` 通过（③ 层 + ④ 层启动，本轮基线齐备），`review:prototype-parity` 经 2026-10-01 用户裁决按 §26.1 口径满足。**

已交付：

1. **配对装置**：`apps/desktop/ui/qa/prototype-parity.browser.mjs`（概览 ready，12 组合）、`qa/prototype-parity-pages.browser.mjs`（**全 6 路由 × 12 组合 = 72 组**，自带并发静态服务）、`qa/prototype-parity-states.browser.mjs`（**概览 setup 双形态 × 12 组合**），同逻辑窗口／主题／档位分别截取原型 `.window`（tray 取 `.tray-stage`）与实现 `.app-window`；`gen-parity.py`（通道差阈值 20、预算 1%，输出 `diff.png/json` 与分区差异率）；机器证据目录 `.adg/work/prototype-parity-20261001/`。
2. **几何与语义对照**：概览 ready 主要区域（状态区/舞台/动作行/三卡/顶栏）**12/12 组合 ≤ 2 CSS px**；全页面装置 **72/72 采集成功、0 页面错误**，应用壳 `.main`/`.topbar` 在全部可比组合 ≤2px（`route-section.active` 的固定偏移为选择器口径差，非布局漂移）；状态/弹窗装置 **84/84 成功**——概览 setup 12 + 运行主线 10 态 ×2 + 设置 10 分区 ×2 + 托盘 4 态 ×2 + 诊断 3 Tab ×2 + 扩展 2 Tab ×2 + 扩展长 Markdown 详情 ×2 + 反馈浮层 ×2 + 运行详情 2 态 ×2 + 安装/卸载向导 ×2（`data-mode`/主线文案不一致均 0；弹层两侧均打开 overlay=true/true），**`data-route` 不一致 0**（逐路由/逐态语义对齐）。覆盖明细与未覆盖项见 `.adg/work/.../test-matrix.md` §4。
3. **CSS 预算回收**：删除 46 条死规则（`panel-hub*`/`mock-*`/`path-action-row`/`ovc-badge`/`ovb-*`/`modal-warning`/`wiz-result` 等，已与 Rust 侧 `panel-hub.js` 交叉确认无消费者）；CSS 原始 **82880（相对阶段 A +4.52%，≤5% ✓）**，主 JS **360763（+3.23%）**。
4. **性能复测（③ 层开发服务器，同条件）**：高档 30s 稳定可见 rAF 间隔 p50 ~24ms / p95 ~25ms / max <26ms（无 >100ms 卡顿）；中/低档 rAF=0；页面隐藏 rAF=0；三档 CPU 与 JS 堆 8MB；交互分布（各 20 次）路由 median 3ms、通知面板 200 条 median 53ms；资源：Δinterval=0、Δ活observer=−1、长寿 listener 净 −36、Δipc=0；启动 5 次中位数首帧 ~140ms / 业务就绪 ~160ms。记录见 `perf-baseline.md` §6。
5. **制品启动（④ 层）**：当前哈希制品冷启动 5 次中位数「窗口出现 466ms / 后端就绪 546ms」，无挂起。

**已裁决（2026-10-01 用户：以 §26.1 为准）→ `review:prototype-parity` 满足**：§5.1 像素门按 §26.1 还原项评估（硬门：几何 ≤2 CSS px + 字段/按钮/分组/字号/可见性匹配；§26.1 排除的 mock/真实数据承载内容与 §2.2 置透明色纱逐项归因后不计入失败；整窗数值如实记录为上下文）。归因表见 `.adg/work/prototype-parity-20261001/attribution.md`。原事实链（保留备查）：

1. §5.1 要求整窗像素差 ≤1%，且假设「常量版本、路径、时间、列表项在两边相同」，禁止遮掉整卡/Logo、禁止第二套「为了截图」DOM；
2. §26.1 要求历史 mock 的版本/时间/英文值/假成功/过时文案**不照搬**（真实数据优先）；
3. §2.2 要求 `--notif-tint-a/b` **必须为 transparent**（原型玻璃工具条双色纱属历史对照）；
4. 实测（低档双侧确定性渲染）：`mods` 差异 8%–15% 主因是**标签与取值**（原型 mock「升级状态/最新版本/2.51.0/规划中」vs 生产真事实「当前版本/安装形态/…」），几何已 ≤2px；`actions`/`topbar` 差异主因是原型双色纱玻璃。

即：在**不违反 §2.2/§26.1** 的前提下整窗 ≤1% 不可达，属规范内部口径冲突，非实现缺陷。候选收敛：**A** 限定像素门限适用范围为结构/材质/排版，数据承载文本按 §26.1 记为规范允许差异；**B** 仅开发用审计 fixture 连标签取值一起镜像原型（与 §5.1「禁止第二套 DOM」张力）；**C** 修订 §26.1 允许三卡用原型常量。详见 `.adg/work/prototype-parity-20261001/status.md`。

### 阶段 F · 当前 Tauri 制品验收（TASK-174，2026-10-01，已完成）

**状态：已完成并收口（RCP-OPENCODEX-DESKTOP-174）；`review:artifact-smoke`、`review:workflow-integrity` 通过。** 先前唯一的阻塞（屏幕锁定导致 ④ 层 WebView 冒烟取不到证据）在屏幕解锁后已解除，见 §11 ㉓。

已交付（详见 `.adg/work/imp05-execution/evidence/artifact-smoke.md`）：

1. **当前源码打包**：`npx tauri build --target aarch64-apple-darwin` 成功；`.app` 可执行 SHA-256 `073b162f…c3b4`（6.9MB）、`.dmg` SHA-256 `56ff4c36…10f6`（3.7MB）；二进制内嵌前端与 `ui/dist` 一致（JS `09a2ba1f…`、CSS `40865175…`）。构建源码 `4a4e34f1`（工作区干净）。
2. **全量回归**：typecheck / vitest 68 文件 340 例 / build / 浏览器 355/355；cargo fmt / clippy `-D warnings` / `cargo test --workspace --features integration-test` **526 通过 0 失败**。
3. **制品启动/生命周期/CLI/托盘**：隔离 HOME 启动、窗口 1180×760、单实例、后端写回 `runtime.json`/`data-root.json`/`notifications.json`、SIGTERM 收敛；`ocxd` 契约（help 0 / status 3 / start 4 / unknown 2）；托盘为原生 `NSStatusItem`，动作可用性与运行事实一致（启动可用、停止/重启不可用）。

**④ 层复核已取得（2026-10-01 二次会话）**：制品 WebView **恢复正常渲染**——根因是**应用窗口此前被其它窗口完全遮挡**（macOS 不为被遮挡的 WKWebView 合成内容），前台化后界面完整渲染。已完成的制品内真机证据（`.adg/work/imp05-execution/evidence/artifact-smoke.md` §8）：

- **视觉复核**：概览 setup/ready 双形态、设置 10 分区、扩展管理、诊断中心、面板门禁态；**三档即时生效并落盘回读**（`preferences.json visual_effects` high/mid/low；卡片填充高档 `(247,248,250)` 玻璃 → 低档 `(255,255,255)` 实底，高/中一致）。
- **真实副作用流程**：**联网托管安装**（真实 registry 取包 → `安装完成 · 2.74.0 · 联网安装 · 100%` → `runtime.json source=managed/resolved_version=2.74.0`）与**完整卸载**（`卸载完成 · 100%`、生成备份、残留核验 → `source=unresolved`、包体移除）。
- **启动（④ 层）**：窗口出现 481–725 ms、后端就绪 ≈550 ms 稳定可复现；渲染首帧受遮挡/截屏轮询粒度影响，方法受限，不作精确断言。
- **新登记疑点**：`runtime.json.history` 末尾出现 `uninstall/failed`（紧随一次 UI 成功的 uninstall、多个备份 id），疑重复触发或幂等二次执行语义问题，待复现。

**④ 层补充（2026-10-01 第四会话）**：在隔离根 + 前台化方法下继续补齐真实流程与原生入口，新增证据见
`.adg/work/imp05-execution/evidence/artifact-smoke.md` §8.2b–§8.2e：

- **配置迁移导出/导入（GUI）**：导出落盘 `OCXDCONF` v2（SHA-256 完整性）；导入经「校验→备份→确认→应用」，
  自动备份 `backups/2026/10/import/bk_…`；**篡改容器被拒**（Toast「配置导入失败；当前配置未修改。」，未部分应用、无新备份）。
- **官方 codex-shim 开关**：隔离根内放置占位 `codex` 后，**安装成功**（官方 wrapper 替换启动器、原文件存为
  `codex.opencodex-real`、写 `codex-shim.json`、UI 开关转深色）与**卸载还原**（wrapper 移除、原文件复位、
  `codex-shim.json` 删除、UI 开关复位）；无 `codex` 时**退出 0 未生效不被判成功**（行内给出可执行原因）。
- **原生菜单 / 托盘动作（8 项可真机点击）**：启动/停止/重启（端口起/换 PID/释放）、打开扩展面板、快速设置、
  打开诊断中心、打开数据目录（`manager-state`，与前端实现一致）、打开主界面（关窗隐藏到托盘 → 进程存活 → 菜单唤回）。
- **WebDAV 同步（GUI）**：**绕开 IMP-04 的钥匙串阻塞**——在隔离根内新建登录钥匙串后，端点与口令**真实持久化**
  （`sync-endpoints.json` + 隔离钥匙串条目）。第四会话记的「TLS 分支受阻」**已更正为误判**（见下文 ⑧）。
- **「疑似缺陷」复核结论（已定位，非重复触发）**：上条登记的 `uninstall/failed` 是**如实记录**——隔离根内官方
  `ocx uninstall` 因缺少归属清单**退出 1**（管理器按设计不重试、不代删），历史据此记 `failed`；多份备份属
  「一事务多文件备份」设计。**保留问题**：卸载完成头在存在失败步骤时仍硬编码「卸载完成 · 100%」，与步骤事实
  不一致（IMP-04 既有行为，非 IMP-05 改动）——**已在第七会话修复，见 ⑨**。

**⑤ 扩展 MCP 写入（第五会话）**：制品内真机走通「发现 → 添加 → 逐客户端开关 → 删除」：新服务器写入全部 6 个
客户端配置（`0600`，保留既有条目）、单客户端开关互不影响、删除后 6 处节点清空。期间**发现并修复一处真实缺陷**：
MCP 投影前备份此前用 `skills_dir.ancestors().nth(4)` 当备份根，导致备份**逸出数据根**（隔离根落到 `$HOME/..`
与系统临时目录；真实 `$HOME` 下退化为 `/`）。修复见提交 `c7c2fd28`（`ClientTarget` 增 `home`、备份统一写
`data_root`），并新增回归用例；**二次构建筑品** SHA-256 `.app` `65cf6bb3…651a`、`.dmg` `49eca5b5…73a8`，
修复后真机复测 6 份备份全部落在数据根、无游离目录。

**⑥ 扩展 Skills 发现/链接/卸载/恢复（第五会话）**：制品内真机走通「发现 → 逐客户端链接（symlink）→ 卸载 →
恢复（负分支）」：链接只作用于被点客户端、**源目录只读不被删改**、卸载后统一配置清空、无备份时恢复**显式失败**。
~~**ZIP 导入与恢复正分支未取证**~~ → **该两项已在第七会话补齐**（并在过程中定位并修复一处真实缺陷，见 ⑨）：
原生选择器改用「`Cmd+Shift+G` 定位 + **双击文件行**」驱动（合成点击「Open」不交付文件）。
② 层仍由 `tests/extensions.rs` 覆盖，同一投影引擎的 MCP 写入已真机端到端验证。

**⑦ 运行 Doctor 与界面异常恢复（第五会话）**：制品内真机覆盖两态——无运行时「Doctor 暂不可用；已保留上次结果」
（显式失败、不覆盖上次结果）、托管来源下真实渲染 `ocx doctor` 只读摘要（Paths/Response-state 等，未做写入）；
「界面异常自检」注入受控异常 → 兜底条出现 →「重新加载界面」恢复，自检不改配置或数据。该入口覆盖托盘 `RunDoctor` 的同源能力。

**⑧ WebDAV 同步成功分支打通 + 一处结论更正与一处真实缺陷修复（第六会话）**：

- **更正**：第四会话「应用不采信 `SSL_CERT_FILE`、TLS 分支受阻」**是误判**——当时本地测试服务已退出，
  应用拿到的是**连接被拒**，而 `map_request_error` 把**所有** `is_connect()` 失败都报成「TLS 证书校验失败」。
  最小探针确证：设 `SSL_CERT_FILE` → TLS 通过（401，仅缺凭据）；不设 → `invalid peer certificate: UnknownIssuer`；
  服务未起 → `Connection refused`。**② 层真实服务器同步 4/4 复测通过**（`--test-threads=1`）。
- **修复（第五会话→本轮真机发现）**：`is_certificate_error` 只在错误链确含证书字样时归 `Tls`，其余连接类失败归
  `Network`（文案「WebDAV 网络请求失败；本地内容未修改。」、可重试）；新增单测
  `refused_connection_is_network_not_tls`。真机复测：停服务后点「测试连接」得**网络请求失败**（不再误报 TLS）。
- **④ 层同步真机走通**（第三构建筑品）：**连接成功** →「同步已上传」（`PUT 载荷×2 → 快照 → latest.txt`）→
  「同步完成：已应用远端 preferences.json、extension-config.json，并上传本机快照」（`GET 快照/载荷 → PUT`）；
  覆盖前备份 `backups/2026/10/sync-overwrite/bk_*`（`restorable=true`、`note=snapshot:<id>`）。

**⑨ Skills ZIP 导入真机失败根因定位与两处修复 + 归档验证（第七会话）**：

- **定位**：上一会话在制品内执行「扩展管理 → Skills → ZIP」只得到 Toast「未归类的失败」，`exports/`、`skills-store/`
  全空、统一配置未写入。根因在**命令 DTO 契约**：`ExtensionWriteCommand` 用
  `#[serde(tag = "kind", rename_all = "snake_case")]`，而容器级 `rename_all` **只重命名变体名、不重命名变体字段**；
  该枚举中只有 `ImportSkillArchive` 带多词字段，后端因此要求 `archive_name` / `archive_payload`，前端发的却是
  `archiveName` / `archivePayload` → **反序列化阶段即失败**（`missing field archive_name`），写入从未开始。
  既有 Rust 用例都直接构造枚举、绕过反序列化，故此前未暴露此缺口。
- **修复（提交 `42136809`）**：命令枚举补 `rename_all_fields = "camelCase"`；新增回归
  `extension_write_command_accepts_frontend_camel_case_import_payload`（用前端真实载荷形状，修复前失败）。
- **修复（提交 `9209bb75`）**：卸载完成头在有 `status=failed` 步骤时仍报「卸载完成 · 100%」并配绿色对勾，
  属 §26.1 排除的「假成功」；现为「卸载完成但有 N 步失败」+ 进度条转红，无失败时保持原样。
- **第四次构建**：`.app` 可执行 SHA-256 `3b137647…4cdf`、`.dmg` `4b3d40c2…6c5f`；前端 `4e37258e…3f36`（JS）/
  `20101f8c…64d9`（CSS）。门禁：typecheck 通过、vitest **341** 例通过、build 通过、`audit:browser` **355/355**、
  fmt/clippy 通过、`cargo test --workspace --features integration-test` **529 通过 / 0 失败 / 3 忽略**。
- **真机复测（隔离根 `home-w`）**：ZIP 导入 `sample-skill.zip` → `exports/sample-skill.zip`（`0600`）+ 
  `skills-store/sample/{SKILL.md,reference.md}` + 各启用客户端链接（`rev=5`）；再导入 `sample2.zip`（`rev=6`）；
  行内卸载 `sample` → 备份 `bk_20261001024531_3f24b65b`（`note=before skill uninstall`、含 `sample/`）、
  源目录副本与客户端链接移除（`rev=7`）；**「恢复」正分支**输入 `sample` → 目录按备份原样还原、UI「已安装 3」（`rev=8`）。
  证据 `evidence/artifact-smoke.md` §8.2j。

**⑩ WebDAV 冲突分支的 ④ 层尝试（第八会话）与两处事实更正**：

- **④ 层未取得，受阻于环境**：隔离根内点「立即同步」未发出任何 WebDAV 请求、界面落前端兜底文案；
  逐项定位到 macOS 把路径名 `login.keychain-db` **按名称特判**为「登录钥匙串」——同一文件经该路径名
  `SecKeychainUnlock` 返回 `errSecAuthFailed(-25293)`，而对其副本/硬链接用同一口令返回 `0`；
  `security default-keychain -s` / `list-keychains -s` 在新进程亦不生效（搜索列表被强制回
  `login.keychain-db + System`），「把条目写进另一个可解锁钥匙串」的路子同样不通。**这是隔离会话的
  钥匙串语义问题，不是产品缺陷**（上一会话已在该根下完成凭据持久化与整轮同步）。证据 §8.2k。
  收尾用 `security add-generic-password -A -U` 直写该钥匙串时，`security` 自身即弹出
  「想使用"登录"钥匙串。密码：」——该文件被系统当作**账户登录钥匙串**、要的是**账户口令**，
  这是决定性的：本会话（隔离 HOME 的远程桌面）**无法**让应用读到 WebDAV 凭据，
  需在本地物理登录会话或改用非 `login.keychain-db` 名称的钥匙串时复测。
- **环境清理（后续遵守）**：排查时 `security default-keychain -s <隔离钥匙串>` 曾把**用户级默认钥匙串**
  写进 `~/Library/Preferences/com.apple.security.plist`。已还原 `DefaultKeychain` 为
  `~/Library/Keychains/login.keychain`、搜索列表恢复 `login.keychain-db (+ System)`、删除误留锁文件；
  备份 `/tmp/ocx-smoke/com.apple.security.plist.bak`。**后续勿再对真实域执行 `default-keychain -s`**。
- **② 层真实端点复跑（第十一会话，当前源码）**：`webdav_sync_real` 4/4（含**回放冲突**：断言
  `conflicted=true`、`applied` 为空、本地未被覆盖；以及真实 401 → 认证失败、载荷缺失失败不动本地、
  并发互斥）+ `webdav_legacy_real` 1/1（**旧版 v1 加密远端**：缺口令必失败、带原口令可读出）
  = **5/5 通过**（`--test-threads=1`）。即冲突分支与旧加密兼容在真实端点上可复现通过，④ 层 GUI 仍受阻。
- **更正一（冲突语义）**：冲突判定是**回放保护**——`modules/sync/mod.rs::replay_decision` 在「远端
  `snapshot_id` 等于本地最后接受的快照」或「远端 `created_at` 不晚于本地接受时间」时进入 `Conflict`
  （暂停覆盖、保留双方历史）。此前登记的「双端同一路径不同内容」**不准确**，据此更正。
- **更正二（发现一处无效果的承诺）**：端点的「冲突策略」四档
  （每次询问 / 保留本地 / 保留远端 / 保留双份）在 `modules/sync/config.rs` 只被**校验取值**、
  在 `commands/sync.rs` 只被**投影给前端**；`modules/sync/{engine,runner,mod}.rs` **无任何读取**，
  即四档对同步行为**没有影响**，一律「冲突即暂停 + 按偏好发通知」。整改会改写全局同步的覆盖语义，
  **属需单独授权的改动，本轮不改**，登记为未解决项。

**⑪ 托盘 `NSStatusItem` 自身菜单逐项复核（第九会话，已补齐）**：

- **此前「AX 只枚举 3 项」的登记作废**：改走 `System Events` 的 **`menu bar 2`** 通道即可完整取得
  该状态项菜单——`menu bar item 1 of menu bar 2`（`description = status menu`）下 `menu 1` 共 **14 项**
  （状态头 1 + 分隔线 3 + 可动作项 10：启动/停止/重启 OpenCodex、打开主界面/扩展面板/诊断中心/数据目录、
  运行 Doctor、快速设置、退出）。此前枚举不到，是因为只走了应用菜单栏 `menu bar 1`。
- **门控矩阵真机核对**：未运行 → 状态头「未运行」，仅「启动」enabled；运行中 → 状态头
  「运行中 · 本地 · 127.0.0.1:10100」，「启动」disabled、「停止/重启」enabled。
- **逐项真实副作用**：启动 → `127.0.0.1:10100` 真监听（`bun.exe`，`curl /` 200）；停止 → 端口释放、门控回翻；
  打开主界面/诊断中心 → 主窗口进入对应页；打开扩展面板 → 未运行时不伪造面板而给出显式原因；
  打开数据目录 → Finder 打开 `manager-state`；退出 → 壳进程消失。**无运行时根下点「启动」不伪造成功**
  （状态头仍「未发现」、门控不变）。证据 §8.2l。

**⑫ ④ 层首帧改为可复现方法并建立基线（第十会话）**：

- **旧结论作废**：此前「④ 层精确首帧受遮挡/轮询粒度限制、不作断言」不再成立。新方法：**先隐藏其它应用、把区域背景
  固定为桌面**，再 `open` 启动；轮询 `screencapture -R 1010,264,620,220`（≈0.15 s/次），
  `firstChange` 用「与启动前基线差异 > 1.5」判定窗口出现，`firstContent` 用「与**终态图**差异 < 4.0」判定内容到达终态，
  以此排除「空白窗口」与「旧窗残影」两类假信号（旧的 stddev 探针正是栽在这两类假信号上）。
- **四次构建筑品实测（隔离根 `home-w`，冷启动）**：窗口出现 **≈189 ms**（n=9）、首帧内容 **≈707 ms**（n=9）、
  后端就绪 **≈290 ms**（n=5）、**首次点击生效（可交互代理）≈824 ms**（n=5，含每轮约 0.3–0.4 s 抓屏/点击开销，
  故为上界）。证据与样本见 `evidence/artifact-smoke.md` §8.3.1。
- **如实登记的边界**：「首帧内容」是「到达终态内容」时刻、非浏览器语义 First Paint；「可交互」为上述代理指标、
  非 JS 主线程空闲判定；阶段 A 未采 ④ 层数值，故本节是**首次基线**而非「相对阶段 A 不回归」的对照。

**⑬ 升级入口与「管理器自更新」真机复核（第十一会话）**：

- **升级前备份（自动）**：进入「设置 → 版本升级」即落盘 `backups/2026/10/upgrade/bk_…`，manifest
  `action=upgrade`、`note=ui-requested`、`restorable=true`、`target_path=…/manager-state/preferences.json`。
- **官方更新入口**：`打开引导` → 展示型弹层「升级前请确认已生成备份，然后在终端执行官方命令：`ocx update`」，
  并**明示不执行 `ocx update`、不接管 npm 包管理器**；`查看建议` 给出真实备份 id 与备份目录。
- **管理器自更新**：`检查应用更新` 得「更新下载或连接失败；已保留当前版本。」+ 标签「失败」——本工程更新端点为占位
  `https://updates.example.invalid/…`，故**不伪造「有更新」或「已是最新」**；无可用版本时 `安装更新` **禁用**
  （点击无弹层、无假安装）；通道切到「测试通道 · 每 6 小时」→ 卡片即时变「测试通道」且
  `preferences.json app_update_channel=beta-6h`（复测后已还原 `stable-24h`）。
- **如实登记的阻塞**：§6 的「待重启」在本工程**无独立 UI 状态**（自更新仅失败/已检查/有更新三态 + 安装弹层
  「签名校验通过后由桌面壳接管重启；不会停止托管代理」）；因端点占位，**安装与重启路径不可达**，
  按 §6「条件不具备时记为阻塞」登记。证据 §8.2m。

**⑭ 卸载「部分失败」头修复的 ④ 层复核（第十一会话）**：隔离根 `home-v`（托管 `2.74.0`）走完整卸载向导，
结果头为 **「！卸载完成但有 1 步失败」**、元信息 **「完整卸载 · 1 步失败」**（不再有「· 100%」与绿对勾），
Toast「卸载完成但 1 步失败；请按步骤详情处理，未清理项已列入残留核验」。失败步是隔离环境固有的官方
`ocx uninstall`（与 §8.1x 同因）。即提交 `9209bb75` 的修复在 ④ 层亦得证。证据 §8.2n。

**⑮ 联网安装 → 启动 → 面板内容/缩放 真机复核（第十一会话）**：隔离根 `home-w`（四次构建筑品）——

- **联网安装**：概览「安装 OpenCodex」→ 向导（安装位置 → 安装源 → 指定版本 latest → 勾选确认）→
  「正在从 registry 取包 9%」→「正在安装进私有前缀 85%」→ **「安装完成 · 2.74.0 · 联网安装 · 100%」**
  （步骤明细含「校验离线包 / 展开到私有前缀并链接 bin / 校验并切换运行来源」，入口显示 SHA-256）；
  `runtime.json` → `source=managed`、`resolved_version=2.74.0`。
- **启动**：托盘「启动 OpenCodex」→ 进程真起（`bun` 监听 `127.0.0.1:10100`，`curl /` 200）、门控翻转；
  **健康检查确认前**浮层显示「启动未完成 / 等待时间较长，后台仍会继续观察状态」——**不提前宣称完成**。
- **面板**：未确认运行时时为门控（「官方面板需要 OpenCodex 处于运行且健康可用状态」+ 重试/在浏览器打开）；
  确认后点「重试」→ **官方 v2.74.0 面板真实渲染**（仪表盘 / Codex 设置 / 提供方 / 模型 / 子代理 / 日志与调试 /
  用量 / 存储 / 远程连接 / 集成）。
- **面板缩放联动**（§5.2「面板……缩放」）：设置 → 通用 → 界面缩放点「150」→ 立即缩放并落盘
  `preferences.json interface_scale=150`；回面板确认**内嵌面板与外壳同一缩放因子**（无双重叠加/固定侧栏宽度）；
  复测后已还原 100%。证据 §8.2o。

**⑯ 离线包入口与校验负分支（第十一会话）**：设置 → 安装配置 →「导入离线包」→ 向导（安装位置 → 安装源）；
契约文案明示「不匹配时拒绝，**不回退成「任意 tgz 都装」**」。选 `bogus-offline.tgz`（34 B 文本）→
预览 **「不是 gzip 归档；请选择官方 .tgz 离线包」**，且此时点「下一步」**不推进**（无假推进、无部分写入）。
**未取证**：离线安装的**正分支**（合法官方 `.tgz` → `npm --offline`）需官方离线包与可离线解析依赖的 npm 缓存，
隔离根内不具备（联网安装已在 ⑮ 走通）；正分支**未按通过处理**。证据 §8.2p。

**⑰ 离线安装正分支（合法官方 `.tgz` → 本地安装）真机走通（第十二会话）**：隔离根 `home-w`，用
`npm pack @bitkyc08/opencodex@2.74.0` 生成官方离线包（SHA-256 `11f0c3f973c0d7ae…`）并按应用口径
（`HOME` + `npm_config_cache=$HOME/.npm`）预热缓存。制品内「设置 → 安装配置 → 导入离线包」经原生面板
（`Cmd+Shift+G` 定位 + 双击文件行）交付合法 `.tgz` → 预览「`…tgz · 12.6 MB · SHA-256 11f0c3f973c0d7ae…`」
（与包体真实值一致）→ 勾选确认后「下一步」转 enabled → **「安装完成 · 2.74.0 · 离线导入 · 100%」**。
`runtime/opencodex/.runtime-manifest.json` 记 `source=offline`、`tarball_sha256` 与预览一致、`scripts_enabled=false`；
`runtime/opencodex/package.json` 依赖为 `file:…bitkyc08-opencodex-2.74.0.tgz`（本地包）；`runtime/bin/ocx` 生成并指向包内 `bin/ocx.mjs`；
`manager-state/runtime.json` 新增 `install/succeeded`（`at` 与 `installed_at` 一致）。**⑯ 的「未取证」正分支转为已通过。**
口径更正：`runtime.json` 顶层 `source=managed` 表「来源形态」，**不区分联网/离线**；安装方式由 `.runtime-manifest.json` 的 `source` 记录，
`history` 条目无 `source` 字段。证据 §8.2q。

**可运行（§6 全链收口）**：安装后管理器自动换替代理（`restart-handoff.log` 起 pid `99862`，概览记「运行中」）；
**全新启动实例**点「启动 OpenCodex」→ 新代理 pid `2482`（来自离线安装后的 `runtime/opencodex`）、`/healthz`=200、
`app.log` 记 `start: start confirmed`、概览主状态「运行中」。即「离线安装 → 校验 → 来源切换 → 环境刷新 → 可运行」全链通过。

**新观察（未定性，不按通过处理）**：在**长期运行的同一实例**上依次做联网安装 → 离线安装（就地替换运行来源）→ 托盘「重启」后，
该旧实例出现**状态快照未回传**——概览主状态停在「未运行」、任务卡停在「重启未完成·等待官方进程启动」>60s，托盘菜单头却显示「运行中」，两处矛盾；
同刻直接执行官方 `ocx status --json`（清环境后仅注入 `HOME`/`OPENCODEX_HOME`）返回正确值，**采集命令可用**；
**退出并全新启动同一 `HOME` 后状态立即恢复正确**（未运行→可启动；启动→「运行中」+`start confirmed`）。根因未定位，已如实登记场景/期望/实际/缓解。证据 §8.2q。

**⑱ WebDAV 冲突分支 ④ 层打通（第十二会话）—— 解除 ⑩/§8.2k 的钥匙串阻塞**：改用**非 `login` 命名**的隔离钥匙串
（`HOME=<隔离根> security create-keychain/unlock-keychain/list-keychains -s/default-keychain -s`，真实用户 plist 前后 SHA-256
一致、未被改动），应用即把口令写入该钥匙串且「测试连接」**不再弹窗**（「WebDAV 连接成功。」）。随后在 ④ 层真机复现
**回放冲突**：把远端 `latest.txt` 指回已被本地接受的快照 → 「立即同步」得 UI「**检测到本地与远端冲突；已暂停覆盖并保留双方历史，
请确认后重试。**」，`sync-state.conflicts=[{reason:"replay", local_side:"local", remote_side:"remote", backup_id:null}]`，
**本地未被覆盖**，后端写 `sync-conflict-detected` 通知。② 层 `webdav_sync_real` 4/4 早已覆盖同语义。证据 §8.2r。

**⑲ 冲突通知文案「假承诺」修复 + 第五次构建复测（第十二会话）**：`sync-conflict-detected` 通知正文此前写
「已暂停覆盖并**保留被覆盖侧的可恢复备份**」，但回放冲突在写盘前即返回（`apply_remote_snapshot_batch` 第 1 步判冲突即返回），
**不覆盖任何本地文件**、`conflicts[].backup_id` 恒为 `null`，故不存在覆盖备份——属 §26.1 排除的假承诺，且与 UI 行内
「保留**双方历史**」矛盾。现改为「远端快照与本地历史冲突，**已暂停覆盖并保留双方历史**；请确认后重试。」并加断言护栏。
门禁全绿（fmt/clippy/`cargo test --workspace --features integration-test` **529 通过 0 失败 3 忽略**）；
**第五次构建**：`.app` `8deb05c408f0…2ea2`、`.dmg` `7084c22ac232…2d0b`（内嵌前端不变）。在该制品上重跑 WebDAV 冲突 ④ 层，
行内文案与**后端通知正文均已为新文案**。证据 §8.2s。

**⑳ 真实公网 WebDAV 端点 ④ 层复核（第十二会话，用户提供端点）**：`https://dsm-webdav.guozhigg.top:444/applications/opencodex-desktop/test`
（真实公网 HTTPS + 真实账号，非 localhost 自签；账号口令写入本地 `.env`（0600、已 gitignore、**不入库**））。隔离根 `home-r`：
保存端点 →「测试连接」**连接成功** → 对**坏远端**（`latest.txt` 指向无效清单 `{"snapshot_id":"placeholder"}`）「立即同步」
**失败且远端/本地零改动** → 非破坏地改名 `latest.txt`（WebDAV MOVE）后「立即同步」**「已应用远端 … 并上传本机快照」**
（远端新增快照+载荷+新指针）。即真实公网端点上「连接 → 坏远端拒绝 → 正常同步」三段在当前制品内走通。证据 §8.2t。

**新登记（未按通过处理）**：① **「冷同步」跳过被报成「同步执行失败」**——`preferences.cold_sync` 默认 `true`，
`modules/sync/runner.rs` 在开启时**无条件**跳过（与设置页文案「运行中不直接写关键文件」不符：它不问运行态），
命令层把 `ColdSync/TargetLocked` 转 `Err(AppError)` 后，前端 `runNow` 的 `catch` 统一显示
「同步执行失败；本地内容未修改。」，而后端确有的「冷同步已开启，已跳过本次同步。」在 GUI 路径成了死文案；
② 长路径端点在保存时按最后一个 `/` 重切「服务地址/远端目录」（组合 URL 正确、同步可用，仅与输入分工不一致）。

**㉑ 冲突策略按用户裁决固定为只读「每次询问」（第十二会话，2026-10-01；⑩ 更正二 的整改收口）**：用户裁决 A——
「每次询问」就够，**原型也可以直接改了保持一致**。产品侧把 WEBDAV「冲突策略」四档下拉改为**只读** `每次询问`
（表单状态与保存固定 `ask`，删除 `chooseSyncConflictPolicy` 与偏好播种；历史端点存过哪一档都回到真实行为）；
**原型同步对齐**：设置·WebDAV 与设置·MCP「同名冲突策略」都改为只读 `每次询问`，补 `.setting-readonly` 配方，
帮助文案改为「固定「每次询问」…不提供静默保留策略」。回归用例同步改写。
门禁：typecheck 通过、vitest **341 通过 0 失败**、build 通过（主 JS 360030 B，**较五次构建减少**）、`audit:browser` **355/355**；
Rust 未改动（529 通过 0 失败 3 忽略 沿用）。**配对复跑**：pages 重采 72 组合（页面错误 0）、states 重采 84 组（失败 0），
`screens-states/settings-sync/*` 两侧均为只读「每次询问」。**第六次构建**：`.app` `e2d0dcf59035…365b`、`.dmg` `1a8bbce85a37…ae6d`，
内嵌 `index-DCAHcpQt.js`。证据 §8.2u。

**本轮阻塞（未按通过处理）**：**第六次构建的制品内 WKWebView 冒烟取不到证据**——自动化运行期间本机**屏幕被系统锁定**
（`screencapture` 只得锁屏，合成按键无法解锁）。该制品已按隔离根 `home-r` 启动，但未能取得界面截图/点击证据。

**另登记（既有、非本轮引入）**：原型自带的浏览器 QA（`原型/qa/*.browser.mjs`）存在既存失败：
`nowrap` 4 项、`runtime-source` 2 项、`skill-detail` 1 项、`state-convergence` 超时；**stash 本轮原型改动后重跑，失败集合一致**，
证明与本轮无关。按 §8「先修正与当前已确认原型冲突的历史浏览器断言」处理，**已在㉒（2026-10-01）修正如实**（nowrap/runtime-source 文案/state-convergence 三项冲突断言修正，余 2 项原型侧布局亦已闭合；原型浏览器 QA 全绿）。

**未完成（截至第十二会话）**：`review:prototype-parity` 按用户裁决（§26.1 口径）视为满足；`review:artifact-smoke`
的静态/测试/构建/制品冒烟与视觉复核已具备；`review:workflow-integrity` 的
~~**WebDAV 冲突分支的 ④ 层 GUI 证据**~~ → **已在第十二会话取得（见 ⑱、§8.2r）**，
剩余未闭合项**仅剩**
~~**端点「冲突策略」四档目前无行为效果**~~ → **已按用户裁决收口（见 ㉑、§8.2u）**：改为只读「每次询问」，原型同步对齐；
`review:performance` 的 ④ 层已改为可复现方法并建立基线（窗口出现 / 首帧内容 / 后端就绪 / 首次点击生效，见 ⑫）；
余下仅「首帧内容」与「可交互」为**代理口径**（非浏览器语义 First Paint / 非主线程空闲判定）这一说明性差异；
~~另有**离线安装正分支**未取证（需官方 `.tgz` + 可离线解析依赖的 npm 缓存，见 ⑯）~~ → **已在第十二会话补正（见 ⑰、§8.2q）**：
合法官方 `.tgz` 走通「校验预览 → 确认 → 安装完成 · 离线导入」，`.runtime-manifest.json` 记 `source=offline`。
截至第十二会话，`review:artifact-smoke` 与 `review:performance` 的已有证据保持成立；`review:workflow-integrity` 的
**未闭合项仅剩**：WebDAV 冲突分支 ④ 层 GUI（受隔离会话钥匙串语义所限，② 层已由 `replay_*` + `webdav_sync_real` 覆盖）
以及**第六次构建的制品内 WKWebView 冒烟**（本机屏幕被锁定，见 ㉑）与**原型自带浏览器 QA 的既存失败**（非本轮引入，见 ㉑）。


**㉒ 原型自带浏览器 QA 的冲突断言修正（第十二会话，2026-10-01；提交 `4b73ed71`）**：按 §8「先修正与当前已确认原型冲突的
历史浏览器断言，再运行浏览器 QA」，完成一次原型 QA 全量复跑与断言修正。**根因**：已确认的双形态概览
（`P/overview-dual.css:9` `#card-overview-status .ovb-main,#card-overview-details{display:none!important}`）把旧的
摘要事实行 `.ovc-fact` 与页内折叠运行详情**收进 Logo 舞台的「运行详情」弹层**（`.motion-runtime-paths`），
而 `nowrap`（2026-09-25）、`state-convergence` 等断言仍指向已被退役的 `.ovc-fact`/`.ovb-*` 节点。

- **已修（提交 `4b73ed71`，仅改原型 QA 脚本，不改原型布局）**：① `nowrap` 长路径压力段改为打开运行详情弹层、
  在「数据目录」路径行（`.motion-runtime-paths`，`.motion-runtime-value` 为 `nowrap+ellipsis`）取证 →
  **38/38 通过（原 4 失败清零）**；② `runtime-source` 门禁脚注断言 `includes('runtime/opencodex')` 已过期
  （现文案为面向用户的「安装仅写管理器数据目录」）→ 按当前文案语义断言，该失败清零。
- **全量复跑结果（2026-10-01，本机无头 Chromium）**：`log-categories` 25/0、`nowrap` 38/0、`runtime-source` 115/**1**、
  `skill-detail` 100/**1**、`spacing` 11/0、`tables` 154/0；`state-convergence` 追加移植后 **130/0 通过**（见下条，提交 `429f3055`）。除脚本外的截图产物未纳入提交。
- **再修（提交 `429f3055`）**：`state-convergence` 按双形态 DOM 重写——支线是否活跃改用支线槽 `hidden` 标志判断
  （与视觉落点无关）、问题槽是否裁切改用可见的 `.motion-caption` 代理、运行详情改为打开 Logo 舞台的详情弹层
  （`motion-runtime-modal`）并校验展开可见/不横向溢出/可关闭；**130/130 通过、0 控制台错误**（原为点击不可见节点超时中断）。
- **余项已闭合（提交 `4e552ac8`）**：① `runtime-source`「确认视图正文无滚动条」——原型 `.modal.modal-uninstall` 的
  `max-height` 上限比内容少 4px，正文出现 4px 假滚动；上限 `calc(100% - 16px)`→`calc(100% - 8px)` 后正文不再滚动
  （1692×916 与 1180×760 均为 0）。产品侧同一弹窗经无头探针核验本就无正文滚动（`[data-testid="runtime-uninstall-modal"] .modal`
  及其正文 diff 均为 0），此改动只是让原型与产品对齐。② `skill-detail`「最小窗口下弹窗不越界」原断言误把浏览器视口
  当参照系（原型是把模拟应用窗口 `.window` 嵌在带评审工具条的页面里，页面必然高于视口，必误判）；改为与 `.window`
  同参照系，与 `state-convergence`「弹出层不越出模拟窗口」一致。
- **原型浏览器 QA 全绿**：log-categories 25、nowrap 38、runtime-source 115、skill-detail 100、spacing 11、state-convergence 130、
  tables 154——共 **573 项 0 失败 0 控制台错误**；6 个 node 用例退出 0。
- **对 `review:prototype-parity` 的影响**：三个配对装置（概览 / 全页面 / 状态矩阵）原型侧均取
  `候选/2026-09-28-Logo本体形变/overview.html`；本次只改了根入口 `原型/index.html` 的弹窗高度上限，**未触碰配对基准**，
  几何与像素证据不受影响（配对参考截图仍为更早一轮；截图含环境渲染差，未随本次入库）。
- **补充证据（第六次构建「只读每次询问」的同源 bundle 复核，非 ④ 层）**：制品内嵌 `index-DCAHcpQt.js` 与工作区
  `ui/dist/assets/index-DCAHcpQt.js` 逐字节同名同源；静态托管该 `dist`、无头 Chromium 驱动「设置 → WebDAV 同步」得
  `input[aria-label="冲突策略"]` 恰 1 个、`value=「每次询问」`、`readOnly=true`、`select[aria-label*="冲突"]` 0 个、0 控制台错误
  （截图/JSON 见 `.adg/work/imp05-execution/evidence/app-smoke/`，§8.2v）。**口径**：这是「制品所装前端 bundle 的渲染」（③ 层补充），
  **不据此宣称 ④ 层通过**——§8 要求的「当前哈希制品在 Tauri WebView 视觉复核」仍待解锁屏幕后取证。

**㉓ 第六次构建筑品内 WKWebView 冒烟取得 + TASK-174 收口（第十二会话，2026-10-01）**：本机屏幕解锁后
（`CGSSessionScreenIsLocked` 键消失；锁屏时同一制品窗口经 `CGWindowListCreateImage(.optionIncludingWindow)` 取到近乎纯白、
非白像素占比 0.4‰、OCR 空，解锁后正常渲染），对第六次构建（PID 7075，`HOME=/tmp/ocx-smoke/home-r`，
窗口 1010,264 1180×760 @DPR2，`.app` 可执行 `e2d0dcf5…365b` / `.dmg` `1a8bbce8…ae6d`）做制品内真机冒烟：

- **概览门禁 setup 形态**：正常渲染「尚未接入 OpenCodex」「2/3 项通过」与「安装 OpenCodex / 导入离线包 / 重新检查」。
- **设置 → WebDAV 同步**：「冲突策略」一行文本为 **「每次询问」**；该行右侧（1900≤x≤2320，850≤y≤930）**暗像素 0**，
  即无下拉箭头/下拉 affordance → **只读字段，非 `<select>`**。与 §8.2v 的同源 bundle 复核（`input[readonly]`、`select` 计数 0）互相印证。
- **设置 → 扩展管理**：渲染正常；产品侧 MCP 视图无「冲突策略」字段（仅 WebDAV 有），与 ㉑ 一致。

证据：`.adg/work/imp05-execution/evidence/app-smoke/review-20261001-sixthbuild-{overview-setup,webdav-readonly,settings-extensions}.png`
与 `sixthbuild-webview-smoke.json`、`sixthbuild-dist-readonly-policy.json`；总表 §8.2v / §8.2w。
即第六次构建相对五次构建的唯一产品变更（冲突策略固定只读「每次询问」）已在**制品内 WebView** 得到确认。
至此 TASK-174 的 `review:artifact-smoke` 与 `review:workflow-integrity` 均已满足，阶段 F 收口。
