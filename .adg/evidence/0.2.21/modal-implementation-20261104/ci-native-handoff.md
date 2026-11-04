# 公共模态核心 CI 与原生宿主形态

核心定向命令见 ci-core-tests.txt；完整拥有消费者命令见 ci-owned-tests.txt（28 文件 / 256 项已通过，无未处理异常）。命令在代码根 apps/desktop/ui 运行。

| 核心文件 | 覆盖 |
| --- | --- |
| post09-modal-contract.test.ts | 公共X/空footer/同步异步守卫/actions.close、代表查看/编辑、WebKit指针来源、Tab与嵌套菜单 |
| post09-app-modal.test.ts | 动态查看/输入取消、四通道守卫、旧异步结果、嵌套AppModal焦点归还 |
| post09-owned-consumers.test.ts | SelectorPreview saving、解析观察、扩展业务图标、备份恢复busy、通知清理取消 |
| ui-dialog-keyboard.test.ts | 原生input Escape、晚挂菜单、嵌套原生层顺序 |
| settings-install-feedback.test.ts | 安装事实/完成、四退出限制、显式取消安装 |
| update-center.test.ts | 确认/进度观察、四通道关窗后台继续、候选固定/重挂载、业务重试/重启 |
| model-agent-batch.test.ts | 关观察/卸载不停止生成、显式停止、保存守卫、完成查看无footer |
| model-external-task.test.ts | 明确复制/激活grant单次执行、复制期间四通道关闭守卫 |

## 四个替换原自有遮罩的宿主

所有宿主现在都是原生 HTMLDialogElement，由同一个 UiDialog showModal/Teleport 到 document.body；没有旧的 .modal-mask/.modal 作为定位根。统一标题 header h2 + aria-labelledby、最右 header .modal-close；默认逻辑宽560，wide850，按现有ui-zoom/视口折算。

| 宿主 | 原生选择器 | 逻辑宽 | 操作形态/退出 |
| --- | --- | --- | --- |
| AppModal | dialog.app-modal-dialog[open] | 560 / wide850 | ModalContent的.modal-actions位于content内；查看型没有纯退出/空动作行。动态busy/closing及store beforeClose同守卫。 |
| 托管安装 | dialog.runtime-install-dialog[open]；data-testid=runtime-install-modal | 520 | 配置取消/开始；安装中仅取消安装，X等禁用；完成无footer；脚本授权失败保留实际重试。 |
| 更新中心 | dialog.update-center[open]；data-testid=update-center | 780 | 版本确认有取消/主操作；结果/进度无footer；正文保留重试/重启；更新中X始终可关闭观察。 |
| 扩展详情 | dialog.ext-detail[open] | 720 | 无footer；标题业务图标在X左；busy只禁业务写按钮，允许关观察。 |

原生检查须覆盖四通道退出（X/Esc/原生cancel/遮罩）、取消与确认副作用、Tab/Shift+Tab、菜单/子dialog先消费Esc、鼠标点击未聚焦触发点时仍归还。系统文件选择器按原归属排除。

SelectorPreviewDialog属同一贡献的既有UiDialog消费者：解析busy允许关，saving才限制所有退出，取消只退出预览不保存/丢草稿。可用于原生验证受限编辑状态。

详细机器描述见 native-host-contract.json。Chromium/WebKit已有24组流程/布局、30项几何检查通过；它们不替代macOS/Windows Tauri原生。同步消费者已由主线迁移，其定向回归由主线合并进入统一命令。
