# 0.2.20 production dialog callsite verification ledger

Source: `prototype-audit-20261102/application-dialog-inventory.json`, based on v0.2.19 source `cb4c9c9b7`. It lists 37 component references across 21 files: 35 production references and two dev-gallery examples excluded from the production gate. A reference is not a unique runtime modal.

## Current route and native evidence

The first retained browser route audit report was 411 checks: 401 passed, 10 failed. It is preserved as an earlier failed run. Re-running `apps/desktop/ui/qa/audit.browser.mjs` against current source produced 428/428 passed, zero failed checks and zero page errors at 2026-11-02T13:18Z. Both reports remain in evidence; the new result is the current browser route result.

The isolated native macOS QA copy was re-signed under a test-only bundle ID and used `/tmp/ocx020-native-ui-sandbox-full/home` as HOME. Node and npm were symlinked into that sandbox; an inert `ocx` fixture only reported a local version and refused all commands. Observed states: overview; Settings general and data/backup; the backup cleanup preview empty state; overview export preview (default encryption off, opt-in password fields, canceled); import file chooser (closed without selecting); WebDAV connection entered from sync and overview (canceled without save/test/sync); notification center/detail opened and closed; model channel empty state and new-channel form closed without save; model template list and new-template editor; extension Skills/MCP/client tabs and Codex detail. No install, export generation, import, WebDAV test/sync, backup/restore, save, or notification clear/delete was performed. The native UI session did not capture a screenshot artifact to disk.

Every production reference below remains **not fully verified**; notes are bounded observations rather than acceptance.

## Production references

| File | Component | Line | Status | Bounded observation |
| --- | --- | ---: | --- | --- |
| `apps/desktop/ui/src/App.vue` | `AppModal` | 78 | Not fully verified | App shell present; app-specific modal states not individually opened. |
| `apps/desktop/ui/src/App.vue` | `RuntimeInstallDialog` | 79 | Not fully verified | Overview prerequisite card observed with isolated Node/npm symlinks and inert local CLI fixture; install action not invoked. |
| `apps/desktop/ui/src/features/models/AgentExecutionField.vue` | `UiDialog` | 87 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ModelPicker.vue` | `UiDialog` | 104 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/SelectorPreviewDialog.vue` | `UiDialog` | 130 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelBatchDialog.vue` | `UiDialog` | 85 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelModelTable.vue` | `UiDialog` | 123 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelModelTable.vue` | `UiDialog` | 126 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/NewChannelDialog.vue` | `UiDialog` | 84 | Not fully verified | New channel first-step form observed and closed without saving. |
| `apps/desktop/ui/src/features/models/ExternalTaskDialog.vue` | `UiDialog` | 35 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/TemplateComparisonDialog.vue` | `UiDialog` | 18 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelsView.vue` | `ChannelBatchDialog` | 266 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelsView.vue` | `NewChannelDialog` | 267 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelsView.vue` | `UiDialog` | 268 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelsView.vue` | `SelectorPreviewDialog` | 269 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/SaveResultNotice.vue` | `UiDialog` | 89 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/TemplateUsageDialog.vue` | `UiDialog` | 14 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/BatchAgentDialog.vue` | `UiDialog` | 59 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/models/ChannelEditor.vue` | `UiDialog` | 140 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/features/sync/ImportedConnections.vue` | `UiDialog` | 15 | Not fully verified | No imported connection fixture; not opened. |
| `apps/desktop/ui/src/features/sync/ModelTransfer.vue` | `UiDialog` | 435 | Not fully verified | Overview export preview opened; encryption default-off and password fields after opt-in observed; canceled without creating file. Import chooser opened and closed without selecting a file. Other WebDAV sync/import preview states not executed. |
| `apps/desktop/ui/src/features/sync/ModelTransfer.vue` | `UiDialog` | 436 | Not fully verified | Overview export preview opened; encryption default-off and password fields after opt-in observed; canceled without creating file. Import chooser opened and closed without selecting a file. Other WebDAV sync/import preview states not executed. |
| `apps/desktop/ui/src/features/sync/ModelTransfer.vue` | `UiDialog` | 479 | Not fully verified | Overview export preview opened; encryption default-off and password fields after opt-in observed; canceled without creating file. Import chooser opened and closed without selecting a file. Other WebDAV sync/import preview states not executed. |
| `apps/desktop/ui/src/features/sync/ModelTransfer.vue` | `UiDialog` | 483 | Not fully verified | Overview export preview opened; encryption default-off and password fields after opt-in observed; canceled without creating file. Import chooser opened and closed without selecting a file. Other WebDAV sync/import preview states not executed. |
| `apps/desktop/ui/src/features/backup/ModelSaveHistory.vue` | `UiDialog` | 56 | Not fully verified | No save-history records in isolated data; not opened. |
| `apps/desktop/ui/src/features/notifications/components/NotificationCenter.vue` | `UiDialog` | 111 | Not fully verified | Notification center and read-only notification detail opened; detail closed. Clear/delete actions not invoked. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `TemplateUsageDialog` | 208 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `UiDialog` | 209 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `UiDialog` | 214 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `UiDialog` | 216 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `TemplateComparisonDialog` | 217 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `ExternalTaskDialog` | 218 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/ModelsRoute.vue` | `BatchAgentDialog` | 219 | Not fully verified | Not individually exercised in the isolated native session; full state coverage remains pending. |
| `apps/desktop/ui/src/routes/SyncRoute.vue` | `UiDialog` | 21 | Not fully verified | WebDAV connection fields observed from both Sync and Overview entry; cancel closes while leaving endpoint unconfigured. No test connection/save/sync. |
| `apps/desktop/ui/src/routes/ExtensionsRoute.vue` | `ExtensionDetailModal` | 861 | Not fully verified | Skills/MCP/client views and Codex detail observed; extension detail modal not opened. |

## Excluded development references

| File | Component | Reason |
| --- | --- | --- |
| `apps/desktop/ui/src/dev/component-gallery/GalleryApp.vue` | `UiDialog` | Development component gallery; excluded from production count. |
| `apps/desktop/ui/src/dev/component-gallery/NodeExamples.vue` | `UiDialog` | Development component gallery; excluded from production count. |

## Closure conditions

- Map all 35 production references to a frozen-prototype interaction, or provide a source-based not-applicable reason.
- For each applicable dialog, inspect initial, busy, validation/error, confirm, cancel, close/Escape, and owner-return states as supported by design. Verify route and scroll preservation for in-place flows.
- Exercise failure and rollback with isolated fixtures without daily configuration, credentials, real WebDAV, or production accounts.
- Retain both browser reports. The earlier 411/401/10 failed report is not erased by the current 428/428 route pass.
- Do not close prototype parity or all-entry coverage based on route-shell screenshots, unit tests, or installer acceptance alone.

## Separate S20-17 filesystem lifecycle follow-up

Native Tauri IPC verification after the static 35-callsite ledger confirmed that preparing an export writes an internal frozen package before confirmation; Cancel now removes those cache files without creating anything in the user-facing exports directory. The source implementation is PR #37, head 0389bcc04cccc6000b1a90ef784e83c3a7785191, and is not merged into the current v0.2.20 package/tag. See export-preview-cancellation.md. This evidence covers a side effect of the export callsite; it does not turn the export dialog's other states into full parity acceptance.

## 2026-11-03 bounded browser follow-up

PR #46 passes 104 rendered assertions in Chromium/WebKit with 40 screenshots and zero page errors; see ../acceptance-20261103/README.md. Current ModelTransfer.vue has one shared UiDialog at line 508. Historical picker (435), plaintext-confirm (479) and password (483) callsites are consolidated or removed; those historic line numbers describe v0.2.19. Empty/protected/wrong-password/import-preview/default-backup/Escape/reopen now have explicit browser evidence. Export/WebDAV connection/preview/result and overview cancel checks pass after fixing the DEV discard fixture. No native saved-file/backup or whole-callsite acceptance is claimed. A fresh inventory of new/retired references and remaining native states is still required.

### Current source inventory (37 static references)

The source has 37 references under the same Dialog/Modal component boundary, excluding DEV. Compared with the historical 35: three ModelTransfer windows and SyncRoute direct UiDialog are retired; TransferBackups, ObjectScopeTree, ConnectionDialog, SyncResultDialog and two host references are new. A component implementation and its callers are separate references, not unique runtime windows. Source snapshot: ../acceptance-20261103/current-dialog-inventory.json. Every reference retains pending native/state acceptance.

| Current file | Component | Line | Evidence and remainder |
| --- | --- | ---: | --- |
| App.vue | AppModal | 79 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| App.vue | RuntimeInstallDialog | 80 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/backup/ModelSaveHistory.vue | UiDialog | 56 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/backup/TransferBackups.vue | UiDialog | 18 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/AgentExecutionField.vue | UiDialog | 87 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/BatchAgentDialog.vue | UiDialog | 59 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelBatchDialog.vue | UiDialog | 85 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelEditor.vue | UiDialog | 140 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelModelTable.vue | UiDialog | 123 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelModelTable.vue | UiDialog | 126 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelsView.vue | ChannelBatchDialog | 266 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelsView.vue | NewChannelDialog | 267 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelsView.vue | UiDialog | 268 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ChannelsView.vue | SelectorPreviewDialog | 269 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ExternalTaskDialog.vue | UiDialog | 35 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/ModelPicker.vue | UiDialog | 104 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/NewChannelDialog.vue | UiDialog | 84 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/SaveResultNotice.vue | UiDialog | 89 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/SelectorPreviewDialog.vue | UiDialog | 130 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/TemplateComparisonDialog.vue | UiDialog | 18 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/models/TemplateUsageDialog.vue | UiDialog | 14 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/notifications/components/NotificationCenter.vue | UiDialog | 111 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/sync/ConnectionDialog.vue | UiDialog | 19 | PR #46 browser fields/test/cancel; native persistence/errors pending |
| features/sync/ImportedConnections.vue | UiDialog | 15 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/sync/ModelTransfer.vue | UiDialog | 508 | PR #46 browser export/import/sync/cancel states; native/business edge states pending |
| features/sync/ObjectScopeTree.vue | UiDialog | 32 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| features/sync/SyncDialogHost.vue | ResultDialog | 14 | Host reached by export/sync/connection/result browser flows; all native callers pending |
| features/sync/SyncDialogHost.vue | ConnectionDialog | 14 | Host reached by export/sync/connection/result browser flows; all native callers pending |
| features/sync/SyncResultDialog.vue | UiDialog | 23 | PR #46 browser export receipt open/close; retry/partial/native states pending |
| routes/ExtensionsRoute.vue | ExtensionDetailModal | 861 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | TemplateUsageDialog | 208 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | UiDialog | 209 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | UiDialog | 214 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | UiDialog | 216 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | TemplateComparisonDialog | 217 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | ExternalTaskDialog | 218 | No new per-reference runtime acceptance in this follow-up; applicable states pending |
| routes/ModelsRoute.vue | BatchAgentDialog | 219 | No new per-reference runtime acceptance in this follow-up; applicable states pending |

## Native model dialogs follow-up (f1f26d8f)

See ../acceptance-20261103/native-integrated-f1f26d8f/native-model-dialogs.json and model-*.png. NewChannelDialog validation/Escape, AgentExecutionField unconfigured/save-disabled/cancel, ModelPicker catalog-error/empty/confirm-disabled/cancel, BatchAgentDialog missing-model and target count, ExternalTaskDialog local scope/default/close, ModelsRoute version builtin protection/search/close, and TemplateUsageDialog no-reference/no-model/filter/Escape states were observed natively. Raw English unconfigured feedback and version-search Escape dismissal need fixes and native rechecks. Opening an external-task dialog created a sandbox-local task plan, so no no-write claim is made. No external execution, copy/submission, channel save or version delete/default mutation was attempted. All callsites retain remaining states.

## PR #49 原生模型反馈修复复核

模型目录未配置时显示英文、版本搜索输入框内Escape未关闭，已由be695fc5修正。新增两项回归均在旧代码失败；修复后177文件/1097项UI测试、typecheck/build通过。隔离原生custom-protocol构建使用/tmp/ocx020-model-feedback-native-20261103和独立bundle ID；实屏确认模型选择器显示中文安装设置指引，未配置时确定仍禁用；版本搜索输入后按Escape关闭，源页GPT筛选和版本管理焦点保持。native-model-feedback-be695fc5保留截图、身份和完整AX。已停止自有PID73728，没有模型执行或版本变更。此为当前代码定向复核，最终release包与其余调用状态仍待验。

## PR #50：共用键盘关闭与菜单优先级

PR #48/#49 已全 CI 通过并合并为 0dbb781e4、3120d90a4。继续原生检查发现：be695fc5 的模型明细在别名输入内 Escape 不关闭；中间 377fd4bb 虽能关闭输入窗，但 AX 打开选择菜单后焦点仍在外窗，会关闭两层。最终 e1674de7 通过隔离原生复核：Escape 先只关 Fast 菜单，再从别名输入关闭明细并丢弃局部输入；合成 config.json 的 SHA 与前像完全一致。7 项键盘回归、178 文件/1104 项全 UI、typecheck/build/native 构建通过；PR #50 等待完整 CI，尚不是最终包验收。两次原生失败和最终复测均保留。

新增 model-dialog-states 驱动覆盖新增渠道双步与发现错误/等待/取消/迟到结果、复制/删除取消、渠道批量配置/精确采用预览、手动重复 ID、明细菜单/忙碌/错误/取消、型号批量、保存预览及解析失败、Agent 嵌套选择器、版本搜索、匹配使用、批量 Agent 前置条件、版本比较与匹配预览。四组分别为 1180浅高100%、900暗低100%、1180暗中200%、900浅中50%。WebKit344/344、32截图、0页面异常/外部请求；只证明显式 DEV 适配器下的渲染状态。

S20-19/20 新增待修：Chromium200% 模型详情导航链接区被操作按钮挤到不可用；自动滚动不稳定，显式导航入口也被本体拦截。两个原报告各有252条已通过检查，随后 runtime timeout，不能记为全量通过。改前 Chromium 新渠道搜索 Escape 超时前7条通过也不能算成功。相关 report/failure/截图在本目录 model-dialog-*，继续修复，不跳过尺寸。原型仍 cfbd90c4b+1da4cabca；全部 S20/AC/整体 RCP 保持未关闭，正式包重建/四安装/实际 OTA 继续待验。

## PR #51 高缩放导航修复与双引擎/原生复核

9ce6672e（PR #51，基于 #50）修复详情导航独立伸缩区被压至0，以及fitNavigation漏算主滚动区padding/clientTop造成固定后逐帧滚动。侧栏链接与辅助操作共用滚动区，主保存区独立保留。原型cfbd90c4b要求内部可滚导航、主操作可达和固定边界，未改采用范围。新增合同回归改前把scrollTop推至1800、修复后为0；滚动原件记录fitNavigation不断追加13px。浏览器导航尺寸检查改前links=0/oneLink=34失败，修复后共享滚动区保留完整行。

最终完整矩阵Chromium/WebKit各352/352、各36截图、0页面异常/外部请求，报告completed=true；178文件/1105项UI、13项详情/工作区、typecheck/build及7项Windows smoke工具回归通过。指针点击前显式核对目标在视口且elementFromPoint命中；没有使用force/DOM点击绕过被遮挡控件。前面中间版252条通过后timeout、菜单失败、几何失败和连续滚动追踪均保留。导航补丁单独修复链接区后仍有固定遮挡的中间轮次也保留，不将局部通过改写为全量成功。

navigation-native-9ce6672e为全新临时HOME/独立bundle ID的原生custom-protocol构建，持久200%/dark/mid偏好。实屏从模型模版进入GPT编辑，通过侧栏匹配规则打开匹配预览；空ID禁用检查，输入后Escape关闭并返回未修改编辑器。未调用检查/生成/发现，config.json SHA与合成停用渠道前像一致；已停止自有PID83944。原生200%证据不能代替Windows/最终包或其它数据状态。

Windows smoke新增真实WebView2 keyDown/keyUp：输入Escape取消明细、别名不进入草稿、AX等价外窗焦点时先关内菜单、第二次关外窗、文件/备份不变；仅候选0.2.20要求新合同，公开0.2.19基线不误判。PR #50/#51完整CI与此Windows原生新增检查仍在进行；发布资产及安装/OTA门继续未完成。
