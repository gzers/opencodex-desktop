# POST-09 软件公共模态与自有消费者实施证据

正式任务 TASK-OPENCODEX-UI021-MODALS-20261104 / AC-021-07。代码在 ocx-0219-fixes / codex/0.2.21-fixes；本目录是贡献者证据，不关闭 TASK、不更新共享 DMD/IMP/清单事实，也不代表整版 0.2.21 验收。

## 采用与代表流程

已读 DMD-UI-021、IMP-29、0.2.21 清单、UI §53/54、预审 software-modal-inventory.json；按适用项 G0 已放行。冻结来源是 cf5b50b4fb86e12b64cad209440813159ee06241 的 modal-contract.js / POST-09，复用其 predev 9/9 合同证据，不采用当前总原型所有未来能力。

先实施匹配与使用查看、新增渠道编辑、托管安装受限任务，代表流程通过后再扩其余消费者。编辑/检测指定代表消费者属于排除写集，主线继续接 WebDAV 与检测实例；本贡献只替换为授权范围内同类实例验证。

- before/ 与 baseline.json 保留旧公共层及三代表消费者来源摘要。
- before-contract-tests.log：旧实现 11 项合同检查，8 失败、3 通过，保留反例。
- representative-tests.log：扩迁移前代表与公共键盘检查通过。
- owned-consumer-regression-final.log：28 文件 / 256 测试通过，无未处理异常。
- browser-results.json：Chromium/WebKit 各 12 组流程与布局检查，共 24 组及 30 项几何检查；无 pageerror 或控制台 warning/error。宽/窄、亮/暗、高/中/低与 100%/125% 组合，非笛卡尔全矩阵。
- screenshots/：实际软件组件截图。已目检标题右侧 X、编辑取消/主操作、扩展标题动作顺序、无查看关闭底栏和窄窗适配。
- browser-results-initial.json / browser-run-initial.log：保留 WebKit 鼠标未聚焦触发点的旧实现失败；现公共层同时追踪实际点击/焦点来源，复测通过。

## 公共合同

UiDialog 唯一渲染标题栏和最右 X；header-actions 总在其左。actions slot.close 是同守卫退出 API，busy / beforeClose / 兼容 hideClose 同时约束 X、Esc、原生 cancel、遮罩及走 slot.close 的底部取消。beforeClose 支持同步/异步拒绝，旧窗口异步结果不能关闭新窗口。

hideClose 只保留兼容限制并显示禁用 X；本贡献已迁移消费者均不使用它。它不等于 POST-09 完成。条件插槽为空时没有 footer，查看保留重新读取、复制、重试等真实动作。AppModal/ModalContent 复用同一个原生 UiDialog；mode 可显式为 view/edit/confirm，输入字段推导为 edit，历史关闭类 cancelLabel 推导为 view。动态取消守卫及旧 generation 检查由 app/modal/store 持有。

Esc 仅由当前原生模态层处理；菜单的 data-ui-escape-owner 优先。Tab/Shift+Tab 约束当前层，返回实际触发点，兼容 WebKit 点击不聚焦按钮。关闭的 dialog 不保留 aria-modal=true，不阻挡其它非模态 Esc 消费者。所有字号沿现有 token；未改全局字体、App.vue 或全局 CSS。

## 已拥有消费者

完整路径、状态、退出守卫和对应测试见 owned-consumers.json；18 个实现文件和 21 个改动/新增测试文件摘要见 owned-files.json。备份管理三条动态确认直接消费公共合同，BackupFiles 无自有模态。

任务差异：RuntimeInstallDialog 安装期间限制观察关闭，显式取消安装单独请求终止；UpdateCenter 更新期间始终可关闭观察；BatchAgentDialog 关闭/卸载不再暗中 stopGeneration，仅停止检查取消，已保存结果没有 footer。SelectorPreviewDialog 的 busy 是解析观察、允许退出，saving 才锁所有退出；退出不触发确认保存。

## 集成限制

全 UI 捕获运行 ui-full-regression-second.log 为 177 文件通过/8 失败、1168 测试通过/22 失败，3 个未处理异常。检测、同步状态、保存反馈的主线迁移正在并行发生；不可把这些失败删断言后视为通过。构建曾通过，最后记录 build-final.log 被排除写集 ObjectScopeTree.vue:5 的未使用 UiButton 导入阻断；本贡献没有该文件写权。

仍需主线统一最终源码后重跑完整 UI/typecheck/build，以及原生 macOS/Windows 与必要后端/发布门。浏览器是隔离内存 Tauri 传输夹具，不安装/卸载真实运行时，不改日常配置、不重启日常 Codex，不替代原生或真实事务证据。全部排除写集接入点及 App.vue 具体需求见 main-handoff.md。

未执行 git add/commit/push；没有执行发布或修改版本号，未发 TASK/整版完成证书。

主线最新进展：全部 sync 模态已由主线迁移、回归运行中。本贡献的核心 CI 命令及四个原生宿主形态见 [CI/原生交接](ci-native-handoff.md)。

2026-11-04追加独占写集已实施：异常、概览、卸载、版本管理/查询/结果、行明细、执行模型与恢复引导的最新证据见 [remaining/README.md](remaining/README.md)。该子批次替代之前对应消费者的待迁移说明；最终全UI/原生仍归主线。
