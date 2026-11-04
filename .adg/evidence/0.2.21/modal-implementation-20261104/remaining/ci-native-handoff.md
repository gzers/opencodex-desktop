# 剩余消费者 CI / 原生定位交接

定向命令在ci-targeted.txt，在代码根apps/desktop/ui执行；15文件135项已通过。新增四个专项测试为post09-remaining-representative、post09-remaining-model-hosts、post09-remaining-editors、post09-remaining-overview（均.test.ts）。原有settings-uninstall与update-release-details-flow已接nativeDialog stub及document.body内的Teleport定位，业务断言保持。

所有宿主使用原生HTMLDialogElement，由UiDialog showModal/Teleport；只查dialog[open]。标题为header h2，公共X为header .modal-close。UiDialog新增可选role，默认dialog，异常提示传alertdialog。全体退出仍共享既有busy/beforeClose/actions.close守卫与焦点规则。

| 窗口 | 原生选择器 | 操作形态/守卫 |
| --- | --- | --- |
| 界面异常 | dialog.ui-fault-dialog[open] | alertdialog，X/Esc/遮罩只清提示；保留重新加载，无关闭提示按钮 |
| 运行详情 | dialog.overview-runtime-dialog[open] | 760逻辑px，保留打开目录动作，无底部关闭；观察退出不停止任务 |
| 环境详情 | dialog.overview-environment-dialog[open] | 580逻辑px，业务指引/重新检查保留；嵌套Escape只退此层 |
| 卸载 | dialog[data-testid="runtime-uninstall-modal"][open] | 默认560逻辑px；准备有取消/卸载；执行中所有出口受限；完成没有footer |
| 版本管理 | dialog.model-version-manager[open] | wide850，保留删除业务动作；不再切换成内嵌删除确认 |
| 删除版本确认 | dialog.model-version-delete[open] | 独立原生子层，取消保留父层/选择，busy限制退出；确认仍使用冻结ID及摘要 |
| 默认版本确认 | dialog.app-modal-dialog[open]，标题设为默认版本 | 现在父版本窗口保持；取消回实际设为默认按钮，不重开/重建父层 |
| 匹配预览 | dialog.model-match-preview[open] | 保留检查匹配动作，可关闭在途只读查询；晚回不污染新窗口 |
| 外部任务结果 | dialog.model-task-result[open] | 查看型无footer，不保存/删除/取消后台任务 |
| Agent配置 | dialog.agent-execution-dialog[open] | 临时选择只有明确保存才写入；busy同守卫，取消恢复已保存偏好 |
| 执行模型选择 | dialog.model-picker-dialog[open] | wide850，取消不采用；目录读取可关闭，Escape先退本子层 |
| 模型编辑明细 | dialog.model-detail-dialog[open] | wide850，取消保留既有行缓冲与外层草稿，validating同守卫 |
| 批量型号修改 | dialog.model-batch-dialog[open] | wide850，取消不批量修改，validating同守卫 |
| 官方恢复引导 | dialog.app-modal-dialog[open]，标题官方恢复引导 | mode:view，只有复制命令真实动作；不执行恢复命令 |

卸载data-testid原有计划、范围、步骤、输出、残留、确认请求均保留；uninstall-ack已移动到固定footer防止窄窗覆盖选项，按整个dialog查找。uninstall-finish已移除，结果退出改用公共X。原生卸载过程没有新增取消任务能力。

重要：不要用DOM顺序最后一个dialog推断顶层。AppModal可在DOM前面而成为最新打开的原生子层；用明确选择器或标题定位，再检验当前焦点和Esc归属。查看footer只保留真实业务动作，不再按关闭/完成文案选退出按钮。

浏览器新增证据：Chromium/WebKit各15组流程/布局、40项几何合计通过，无页面异常/控制台告警；组合为宽亮高100%与窄暗低125%，实际主题/画质也已核验。不可代替macOS/Windows Tauri原生。主线sync/WebDAV已128/128及每引擎56通过，这里没有重复执行。

行级建议复核：ChannelEditor.vue:156已采用#actions="{ close }"、取消@click="close"和disabled=busy，保持该接法即可，本贡献未改此文件。其余models worker文件未动。
