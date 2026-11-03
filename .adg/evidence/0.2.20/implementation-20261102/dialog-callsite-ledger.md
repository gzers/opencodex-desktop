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
| App.vue | AppModal | 79 | AppModal: unit focus/trap/Escape/input/confirm/cancel plus rendered delete/notification/history consumers; RuntimeInstall: rendered path validation/cancel and Windows real install/failure/recovery smoke. Remaining: dynamic consumer-by-consumer reconciliation and final package native menu/owner checks. |
| App.vue | RuntimeInstallDialog | 80 | AppModal: unit focus/trap/Escape/input/confirm/cancel plus rendered delete/notification/history consumers; RuntimeInstall: rendered path validation/cancel and Windows real install/failure/recovery smoke. Remaining: dynamic consumer-by-consumer reconciliation and final package native menu/owner checks. |
| features/backup/ModelSaveHistory.vue | UiDialog | 56 | Rendered default-no-pointer/comparison/cancel; full model-history units. Separate legacy recovery-to-draft contract preserved. Final candidate native saved-history success/error remains to reconcile. |
| features/backup/TransferBackups.vue | UiDialog | 18 | Rendered compare error/busy Escape/empty scope/cancel/restore failure; native mixed-channel/Skill/MCP/preferences restore from one operation backup, concurrent preview refusal and outside-edit retention. Final candidate repeat pending. |
| features/models/AgentExecutionField.vue | UiDialog | 87 | Rendered unconfigured/disabled save/nested picker cancel; native unconfigured Chinese guidance and cancel. Model execution remains bounded to owned adapters; final candidate configured choice/error state reconciliation pending. |
| features/models/BatchAgentDialog.vue | UiDialog | 59 | Rendered/native unconfigured prerequisites, target scope and close; full batch units cover proposal/stale/save guards. Actual configured model request is not established by these UI probes. |
| features/models/ChannelBatchDialog.vue | UiDialog | 85 | Rendered initial/missing-template/family-not-adopted/error, exact adoption readonly compare, default keep overrides, cancel/Escape; batch contract units. Final candidate native applicable mutation/rollback states pending. |
| features/models/ChannelEditor.vue | UiDialog | 140 | Rendered manual-empty/duplicate-ID/close; native duplicate-ID refusal and config-byte preservation. Windows native real current-channel save/reopen/scope/key/backup checks passed in PR51; latest package repeat pending. |
| features/models/ChannelModelTable.vue | UiDialog | 123 | Rendered detail input/menu priority/busy/error/Escape and batch initial/cancel; native input/menu fix and Windows CDP real-key regression passed. Pointer sort/file save PR51 evidence. Latest low-effects positioning check/final package pending. |
| features/models/ChannelModelTable.vue | UiDialog | 126 | Rendered detail input/menu priority/busy/error/Escape and batch initial/cancel; native input/menu fix and Windows CDP real-key regression passed. Pointer sort/file save PR51 evidence. Latest low-effects positioning check/final package pending. |
| features/models/ChannelsView.vue | ChannelBatchDialog | 266 | Rendered NewChannel/Copy/Delete cancel/ChannelBatch/SavePreview consumers with unchanged snapshots; Windows real current/global file Save/cancel/exact backup and owner return. Final candidate plus remaining error/late consumers pending. |
| features/models/ChannelsView.vue | NewChannelDialog | 267 | Rendered NewChannel/Copy/Delete cancel/ChannelBatch/SavePreview consumers with unchanged snapshots; Windows real current/global file Save/cancel/exact backup and owner return. Final candidate plus remaining error/late consumers pending. |
| features/models/ChannelsView.vue | UiDialog | 268 | Rendered NewChannel/Copy/Delete cancel/ChannelBatch/SavePreview consumers with unchanged snapshots; Windows real current/global file Save/cancel/exact backup and owner return. Final candidate plus remaining error/late consumers pending. |
| features/models/ChannelsView.vue | SelectorPreviewDialog | 269 | Rendered NewChannel/Copy/Delete cancel/ChannelBatch/SavePreview consumers with unchanged snapshots; Windows real current/global file Save/cancel/exact backup and owner return. Final candidate plus remaining error/late consumers pending. |
| features/models/ExternalTaskDialog.vue | UiDialog | 35 | Native bounded task bundle preview/current target/default allowNew=false/close; Windows real buffer context bundle and no generation. Copy/action success and task receipt states remain explicit separate coverage. |
| features/models/ModelPicker.vue | UiDialog | 104 | Rendered unavailable/no-results/disabled confirm/search Escape/return to Agent; native Chinese failure and nested-owner keyboard; full picker units. Final candidate configured/busy/error variants require reconciliation. |
| features/models/NewChannelDialog.vue | UiDialog | 84 | Rendered two-step validation/discovery error/busy cancel/late result/selection/filter Escape and unchanged saved snapshot; native initial/required validation/cancel. Actual discovery uses owned fixture tests, not daily network. |
| features/models/SaveResultNotice.vue | UiDialog | 89 | Full saved-feedback/component tests plus Windows native saving and later action; restart confirm cancellation and final native process ownership remain separately checked, not authorized daily Codex restart. |
| features/models/SelectorPreviewDialog.vue | UiDialog | 130 | Rendered selection/cancel/parse error, readonly preview retains edited buffer; Windows native actual list confirm/cancel/file scope and exact prewrite backup. Final package repeat pending. |
| features/models/TemplateComparisonDialog.vue | UiDialog | 18 | Rendered modified buffer vs saved version, readonly/Escape/retained buffer; Windows native comparison before save. Final candidate long-content/native error variants pending. |
| features/models/TemplateUsageDialog.vue | UiDialog | 14 | Rendered search/filter/empty and Escape; native no-reference/zero-known-model state and close; usage/stale/late response units. Final candidate populated references/state reconciliation pending. |
| features/notifications/components/NotificationCenter.vue | UiDialog | 111 | Rendered pending-count clear confirmation/Escape/no deletion; full feedback/store tests; native center/detail observed. History cleanup pending/error falsely-success fixed by PR56, browser24 each; final package delivery/deletion checks pending. |
| features/sync/ConnectionDialog.vue | UiDialog | 19 | Rendered fields/test/cancel and imported child confirmation/error; native form/cancel from sync+overview; owned HTTPS transport tests. Final candidate real saved password/blank retention/failed-save and all host states pending. |
| features/sync/ImportedConnections.vue | UiDialog | 15 | Rendered missing-password disabled, explicit activation initial/cancel no call/error and return to Connection owner. Backend candidate import never activates current endpoint; final candidate native activation lifecycle pending. |
| features/sync/ModelTransfer.vue | UiDialog | 508 | Rendered file/remote scopes and shared import/decrypt/backup/export states; native encrypted preference import and mixed package21 objects/3 synthetic credentials, same-ID failed import retry, exact saved/backup/restore. Final package plus custom-directory/failure and remaining SY states pending. |
| features/sync/ObjectScopeTree.vue | UiDialog | 32 | Shared tree component and rendered scopes/selection tests; native node_repl unselected, hidden selection retained and group counts. Object detail/identity/credentials backed by real mixed package. Final per-object detail and final package geometry pending. |
| features/sync/SyncDialogHost.vue | ResultDialog | 14 | Overview/sync shared rendered host; native result/import close retains owner. Host references are callers, not unique dialogs. Every requested overview completion/scroll state still needs final candidate reconciliation. |
| features/sync/SyncDialogHost.vue | ConnectionDialog | 14 | Overview/sync shared rendered host; native result/import close retains owner. Host references are callers, not unique dialogs. Every requested overview completion/scroll state still needs final candidate reconciliation. |
| features/sync/SyncResultDialog.vue | UiDialog | 23 | Rendered empty/read-error/partial original-ID retry/stale compare/native credential-store reason; native export21 complete, import4 complete+17 unchanged and original-ID retry. Custom directory/notification cleanup/final package pending. |
| routes/ExtensionsRoute.vue | ExtensionDetailModal | 861 | Rendered low-read-error Skill/MCP detail and resource entry geometry, full-data document/body scroll tests; native200% reads real sandbox SKILL.md/config.toml, scroll reaches Hermes, Escape returns and files unchanged. Final candidate internal actions/dynamic consumer ledger pending. |
| routes/ModelsRoute.vue | TemplateUsageDialog | 208 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | UiDialog | 209 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | UiDialog | 214 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | UiDialog | 216 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | TemplateComparisonDialog | 217 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | ExternalTaskDialog | 218 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |
| routes/ModelsRoute.vue | BatchAgentDialog | 219 | Rendered version search/protected delete/usage/comparison/match and batch callers; native200% anchor+preview/Escape, Windows real default pointer confirms/back-to-version/immutable revisions. External task result/persisted delete and final state reconciliation pending. |

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

## PR #52/#53 扩展入口与完整详情受限布局

PR #52/f7d8ca11修复Skills/MCP固定六列将名称/详情入口压至0宽：保留名称最小宽度，列表在局部区域横向滚动，不引入未来多Agent扩展包页面。补充secondary-dialog-states三组配置1180浅100%、900暗100%、1180暗200%，两引擎各111项、各33截图、0页面异常/外部请求；覆盖范围备份错误/忙碌Escape/无范围/取消/恢复失败、历史恢复默认不含指针/比较/取消、导入连接缺密码禁用/取消不激活/失败保留、空回执/读取错误/部分原ID重试/过期重新比较、通知清理取消、安装路径校验取消、Skill/MCP详情读取失败保留列表基础信息。178文件1105项UI与typecheck/build通过。原驱动等待未释放的scope preview导致禁用超时已修正；MCP读取失败只保证原列表描述/transport，不错误要求仅成功详情才有的完整command。失败原件保留。

extension-native-f7d8ca11在200%/dark/low独立沙箱真实发现并读取SKILL.md、Codex config.toml定义，入口可打开、Escape回列表，文件摘要保持，未分发/执行。**该轮只证明入口与真实读取**：目检其skill-detail.png确认正文被元数据挤压且部分目标被裁切，不能据AX有文字宣称完整视觉通过。

PR #53/a2bbb35e修复上述受限高度详情：测量内容区、元数据与目标区实际高度，普通高度保留正文内部滚动；空间不足则正文/元数据/客户端共用内容区滚动，固定标题及关闭按钮。观察器销毁释放，异步详情到达后重测。全数据探针首次误挂在主壳外、没有继承200%缩放，该12项结果无效；已归档extension-full-red并明确不用。正确挂在.main内的原版extension-full-red2在200%实测doc.height=0失败。改后Chromium/WebKit各14项/8截图通过（900×600与1180×760、100/200%）：正文可读、关闭可见、滚动至末尾客户端。178文件1105项UI、typecheck/build通过。

extension-overflow-native-a2bbb35e原生200%实屏目检确认Skill正文和MCP完整配置可读，内容区滚动到底可见Hermes等全部客户端，标题及关闭按钮保持，Escape原位返回；两份真实夹具摘要均保持，已结束自有PID87059。原先PID86074也已停止。仅使用OPENCODEX_SANDBOX_ROOT内home的合成资源，没有分发、安装或启动MCP。所有原生构建均为独立bundle ID的custom-protocol调试副本，最终release候选仍待重建和验收。

PR #50～53仍需完整后端CI/Windows新增键盘smoke并按依赖合并；当前正式v0.2.20 draft/tag旧资产未更新，四安装、签名字节核对、相邻实际OTA和全入口剩余状态继续待验。原型基线仍cfbd90c4b+1da4cabca、首次全选/后续新增不选规则保持，整体S20/AC/RCP不关闭。

### 2026-11-03 混合资源原生包、全路由与发布顺序

原生隔离a2bbb35e混合资源导出已通过真实UI→IPC→文件：21项所选配置/3项合成凭据默认不加密、一次确认生成；MCP同名定义差异在预览选择Codex后，包中command及Key只来自该来源，另一个来源Key未带出；node_repl默认未选，Skill的所选.env文件带出，整份manager.extensions为null，可移植偏好仅4字段。真实manifest.secretObjects=3，持久回执21项均complete，输入配置/Skill/客户端文件SHA保持；文件与WebDAV切换后选择独立。只读结果无新操作，取消预览清理未尝试计划且exports为空。原始Claude夹具使用mcpServers未被本次冻结读器识别，已在实际导出前改为当前读器mcp_servers来测试来源；平台真实键路径适配仍归多Agent扩展包，不把这份夹具当平台适配已完成。混合数据全为合成值。

a2bbb35e全路由再次审计428/428、0页面异常，新报告独立保留；旧427/428与定向复跑均不删除。后续菜单定位修复aad48217的全路由复测仍在运行，不能用a2结果代替当前修改。

PR50已全CI通过合并c8c53ac1；PR51全CI通过（包含新增Windows原生Escape/菜单优先级）后进入合并。PR54/9a9a84a0只改验收工具：stable默认保持，允许使用产品已有beta端点先验收实际相邻签名OTA；禁止任意URL/未知通道。升级后必须核对EXE等于候选原始构建SHA。16项Node、7项发布工具、PowerShell7.5.4语法参数通过；没有因此声称实际OTA已执行。最终先干净重建私有草稿/核验签名与字节/独立四安装，再将同一份候选作为预发布供已有beta通道验收，最后才晋升stable。

混合路径实屏还复现低档dialog来源菜单脱离按钮。PR55/aad48217固定dialog菜单坐标空间并允许越过内容滚动区；改前WebKit菜单top308与按钮bottom491脱离。两宽×100/200%×三档，两引擎各48项对齐/可点击断言通过；模型各352、补充各111复跑通过，1105项UI、typecheck/build通过。独立原生低档复测确认菜单贴合按钮、实际选择Codex、菜单Escape保留外窗、第二次Escape原页取消，未尝试计划全部删除，无导出，夹具摘要保持；只停止自有PID。全部仍属调试代码复核，正式包/Windows最新新增定位检查/OTA继续待验。


### 2026-11-03 动态通知历史清理补正（PR56）

生产源码当前仍为37处组件弹窗引用，另发现25处AppModal/input动态调用（不是25个唯一窗口）；current-dialog-inventory.json记录每处文件/行号/标题表达式，不能用组件盘点代替动态消费者。显式测试运行条件另列explicit-test-admissions.json：19处Rust ignore属性包含平台专用、由父测试启动的子进程、拥有TLS夹具、原生凭据和显式外网/已安装官方入口；属性本身不是失败或自动豁免证明，默认cargo通过不等于这些已执行。已实际运行的HTTPS、原生凭据、浏览器文件和Windows原生矩阵分别引用其专用回执；日常真实源/账户或外部生成不在本次测试授权内，不以读取用户配置补齐。

对动态通知历史入口的测试复现旧实现错误：全部/已解决/已读三种清理在等待中或后端失败时立即显示成功，9项测试中6项改前失败。PR56/b810aec2使store返回本次请求布尔结果，公共反馈与历史入口等待结果，失败保留列表，不借其它操作的全局error标志判本次结果。9项新回归、36项通知定向、179文件1114项全UI通过。类型检查发现旧clearAll测试mock仍返回void，已改为真实boolean合同；修正后typecheck/build与最终19项定向通过。两引擎各24项/6截图从真实历史按钮触发store API无native桥失败路径，证明失败提示、原列表、无成功提示及原页保持；DEV浏览器未删除实际通知。

PR50合并c8c53ac1、PR51合并d1004371；pr51-windows目录保存工作流41629159210普通账户及另一原生smoke的键盘/文件保存/读回/备份/原操作导出回执。后续PR52～56尚在CI链路，不能将PR51结果扩展为最终包或后来低档定位检查通过。aad48217全路由新报告同为428/428、0页面异常；b810aec2只新增清理结果行为，已有修复的模型/补充/菜单专项结果保留，不覆盖旧失败轮次。

实际OTA尚未执行，v0.2.20仍非本轮完成候选；公开stable门继续保持未完成。

### 2026-11-03 恢复窗口归属与混合原生导入/恢复（PR57～59）

动态恢复引导四项反例全部在旧代码失败：准备期取消后迟到成功/失败会重开、摘要覆盖较新窗口、备份完成后复活已结束的引导。PR57/35c06338为模态store记录每次打开/resolve代次，异步恢复只在本次归属仍有效时投影；已确认的备份仍按原事务完成，不代执行restore。恢复8项、AppModal/通知定向与179文件1118项UI、typecheck/build通过；双引擎各14项/2截图显示取消/较新窗口保持。仅显式延迟适配器，无真实恢复。

mixed-import-native-35c06338是真实UI+原生文件选择器读取172155字节混合包（来源SHA70433826…）。首次临时进程HOME导致macOS Security.framework读不到原生安全库，原操作回执provider_auth_keychain、17项未变化/4项未完成；所有目标配置/Skill/客户端文件字节保持。该轮不能当成功，也不是产品错误归因定论。源码platform::home_dir已在OPENCODEX_SANDBOX_ROOT/home隔离业务源，安全库服务名固定Sandbox，故按已有native测试合同保留系统进程HOME后重启；期间一次旧进程锁未释放的第二启动exit-6不计通过，原件记录。

从持久结果“重试未完成项→核对原导入”继续相同b7a0a65b操作，真实完成4项、17项未变化；新增渠道及合成Key落已保存config.json，旧provider/Key/顶层sentinel保持；Skill采用所选.env与SKILL.md并移除该Skill旧文件，MCP只写管理器定义源及Sandbox原生秘密引用，未修改客户端config.toml。只有一个同ID backups/transfer-scope范围备份，原渠道/偏好/Skill三组原始字节均精确。初次检查曾误在manager-state找备份得到0，已按实际backups/transfer-scope更正为1并以原件验证，不作无备份结论。

从设置→数据与备份→恢复所选范围实际恢复所有组：第一次预览后新增独立字段导致旧确认被拒绝；重新核对后恢复原provider/Key、Skill及全部旧文件、dark主题，新增MCP管理器源被移除，同时保留afterImportManualSentinel；客户端MCP文件始终未改，原范围before快照不因重试或恢复重写。只结束本轮93617，按日志中的随机账户位删除并验证两项自有Sandbox安全库材料，未读/改日常凭据。

上述实屏发现同名渠道/Skill/MCP三个凭据子项显示完全同名、无法区别，以及安全库失败回执归通用文件权限提示。PR59/57239d5a增加对象类型标签（原ID保持），provider_auth_keychain单独说明本机安全存储；两反例改前失败，改后46项定向、179文件1120项UI及typecheck/build通过。类型检查先发现新测试缺digest，已补正确类型而非跳过校验。双引擎各10项验证同名标签与单独取消MCP仍提交原渠道/Skill身份；补充各117项含原生安全库原因。该新标签/原因仍需最终候选原生复核。

PR58/957d48528补发布说明：首次成功云端目录全选/后续新增不选、便携偏好与后续原生修复；当前为发布说明准备，不是已发布。PR50～53已按各自全CI通过顺序合并；54～59继续CI/合并，最终干净草稿、签名摘要、四安装及相邻实际OTA仍未完成。

### 2026-11-03 统一PR55与Windows驱动失败纠正

PR55旧aad48217的Windows workflow41635474209失败，步骤为final EXE smoke的菜单垂直贴合断言。原日志保留pr55-windows-menu-failed，报告不能当通过。该轮没有在断言前保存菜单矩形，不能从初始main-window截图推导菜单根因；不把本机复现当Windows最终结论。检查驱动发现其使用DOM.click触发未滚到视口内的Fast按钮；同样驱动在本机1028×749稳定复现trigger.top764/bottom798、menu.top660/bottom760，滚动后trigger.top534/bottom568、menu.top574/bottom674，原断言由失败变为满足。

328c2cd8修正Windows探针为先滚动、验证按钮中心在视口且elementFromPoint命中，再发真实WebView2鼠标press/release；不放宽原几何阈值，且断言前持久trigger/menu矩形。c8c7bfa4将OTA旧版准备限定为其已有生产保存文件API，两次真实写入、合成Key与精确前像备份；不在升级前执行新版本导入/弹窗断言。更新后读回同文件与提供方，原生签名/自动重启/候选EXE摘要门保留。17项Node工具与JS语法通过，实际OTA未跑。

PR55现为统一收口PR，head c8c7bfa453dbbfd01eee8696a21e75063ff5eb4d，所有原PR56～59提交均经git merge-base验证为其祖先，使用fast-forward汇入，未重写提交。原PR56因其目标分支已包含提交由GitHub标merged，但**不代表进入main**；57～59关闭为已被PR55覆盖。对应旧CI41637577990/41638473713/41640626634/41641241574与被新测试头替代的41642194514已请求取消，不记通过。唯一当前最终集成CI为41642730883，已确认in_progress；须其完整通过才能合并统一源码。

main当前已到PR54/9d0ca17994ce8ead193c153f12f6d056b827c80d，PR52/53分别合并3ace1df1、3e33ab6c。发布说明已在统一PR中准备。最终tag/draft仍指向旧efbe46e8一轮资产，尚未清理重建；需待固定最终源码，再干净构建、校验上载字节/三份更新签名和资源、独立MSI/NSIS×默认/自定义四安装、最终Mac原生包及实际相邻OTA。现有S20清单、37组件引用/25动态调用的剩余状态继续对账，不能以统一CI完成替代全部required_checks。整体目标保持active。


## c691ce7a最终候选补充

37组件引用/25动态调用口径保持。PR60补模型状态69项/引擎及已挂载Batch原生偏好读回，PR61补同步短高度16项/引擎、原生100/125%、当前同步648项/引擎及全路由428项。原件和失败轮次见acceptance-20261103/README.md末节；最终包/原生/OTA与逐业务状态对账未完成。产品c691与test-only PR62工具98e77975分别记身份。总体检查继续in_progress。


字体增量与原型依赖补正见../compact-typography-20261103/README.md。3e950b12的64项字号/引擎、Mac100/200%通过；完整Chromium原型启动失败、官方catalog合同及最终包/OTA未完成。旧漏上级共享CSS的原型截图不作为最终视觉一致性证据。


## 2026-11-04 动态备份/迁移/客户端消费者

实际生产入口通过两引擎各60项/12截图，覆盖BackupManagement三个动态调用（清理/恢复/打开目录及文件）、Settings迁移确认和ClientsPanel只读说明；取消不调用、pending/错误/成功返回及原页/输入保持都有显式断言。服务端用明确内存IPC，实际文件事务继续引用Rust与原生证据。扩展/日志动态48项/引擎、模型完成69项/引擎和现有模型/恢复/历史状态相互补充。源与原始失败/最终回执见integration-20261104，不把模拟结果当真实磁盘或OS操作。最终包/双平台OTA仍未完成，整体不关闭。


### 2026-11-04 最终包、完整CI与入口漏项纠正

冻结bda完整CI41671607927全部必需job已通过，含Mac后端、Windows后端/原生凭据/真实托管升级及离线重装/最终EXE、Node24/22.20、恢复路径及普通账户。构建41671938225仅重跑Mac后attempt2双平台成功；八原始资产摘要/长度、三份更新签名与sidecar、五平台manifest、Mac包deep/strict签名/嵌入bda源码与许可资源、DMG均独立验证通过。没有换公钥或重建已过的Windows包。

新Mac正式包测试副本仅改独立身份并ad-hoc重签，所有Mach-O文件载荷sections与原封包相同。真实原生目录选择器选自有custom-export-target：首次MCP来源暂不导出，持久结果准确21项中20complete/1skip并说明未选来源；第二次选Codex，21complete/0skip。两次同日不同UUID包均存在，manifest/内容摘要与默认明文/0未选凭据核对通过，第二包只含Codex command，Skill.env未带出，偏好仅4键。通知可分别回看原操作，系统Finder实际打开首包自定义目录，关闭后保持概览。原业务config SHA保持，已结束自有PID34824。初次AX只到窗口装饰随后正常挂载，不是此前不明白屏的重现；Toast自动收起导致旧AX索引失效，按持久铃铛重新定位，不视为产品故障。

新扫描逐文件核对内容等同bda：仍37处Dialog/Modal引用，动态调用应为26处。旧25口径漏了ChannelEditor直接useModalStore().open（调整模型公开范围），不是新增功能。已补两引擎各33项/6截图，覆盖渠道范围及行公开范围取消/确认/过期拒绝和保存态保持；再补ChannelsView/ModelsRoute两个调用组，两引擎各51项/6截图，覆盖删除/还原、取消新建/还原模版、默认版本取消回管理器、失败/过期/等待/成功及保留当前浏览版本。仅显式内存草稿和服务，不做真实删除或运行时写入。驱动初轮因隐藏旧窗错误同名定位、误把取消后重新打开版本管理器当未关闭而失败，修正为实际可见错误/正确原宿主，原件保留。

逐入口新证据映射见final-entry-reconciliation-20261104。历史35/37引用、25调用口径保留为当时盘点，最终用37+26，不以数量代替实际状态。多平台扩展mock的版本未定注释、未来对话/连接图和原型工具明确排除；既有生产消费者仍全部纳入。官方合同同实现诊断41672846917已2项全过；冻结bda本身仍有此前失败状态，已仅重跑原41671597006/attempt2取得直接回执，未改包、未重复其它已通过测试。该项与实际双平台OTA、最终审阅和公开stable仍未完成。


2026-11-04 冻结源码额外官方合同已更新：41671597006 attempt2 对 bda7c92d 两项全部通过（565.95s），与诊断分支不同，此次直接验证正式包冻结源。原始失败与诊断通过回执继续保留，不将此前暂时 provider_write 拒绝归因到未经证实的根因。完整CI、资产/签名和独立四安装已过；实际双平台OTA与公开stable继续待验。原件见 font-final-candidate-bda7c92d/official-contract-frozen-attempt2/。


### 2026-11-04 相邻实际 OTA 与连接消费者补证

PR64已按冻结head bda7c92d合并，merge f1b52e97557b0959d34898f23354b5afd0c255a1。v0.2.20当前已公开为prerelease（release432929365），八资产ID/字节及三份更新签名保持；匿名下载全部八件与冻结SHA一致。既有beta通道的latest.json逐字节相同，公开stable仍v0.2.19，尚未晋升。

Windows41683704589的MSI/NSIS两项实际0.2.19→0.2.20均通过，原生签名下载、普通退出保旧版、明确重启后系统观察新进程、重开文件/合成凭据与pending清理以及原始冻结EXE经唯一MSI/NSS标记核验，逐份原始报告已读取归档，不借四安装中跳过的OTA算通过。

Mac从公开0.2.19副本实际经beta检查、下载、签名安装、手动重启；安装后EXE SHA823a6286…与原上载包完全相同，原包deep/strict签名仍过，系统观察PID41698为0.2.20并继承sandbox变量。原启动使用/tmp软链使Tauri starting_binary拒绝解析，改同一文件的/private/tmp规范路径后正常；自动重启的新进程被本机已有AppKit crash-history恢复提示阻住，保留sample原件。停止自有提示进程，原封安装内容逐文件相同的readback副本仅加进程参数ApplePersistenceIgnoreState YES后正常挂载，日志为bda/0.2.20，实际打开原mixed-fixture渠道，43份基线文件及两次旧版保存前像保持，handoff消费且100%比例保留。明确区分自动新进程证据与随后同字节读回，不把OS提示抹成无条件自动挂载通过。所有自有进程已停止；日常安装/配置未变。

为纠正“导出实屏不能证明连接保存/候选启用”的错挂，另以同正式包测试副本进行ConnectionDialog与ImportedConnections真实原生验收。自有HTTPS服务只监听127.0.0.1，仅实现PROPFIND，不实现PUT；自有CA仅通过该进程SSL_CERT_FILE供正常证书校验，不改系统信任。认证失败、本地内容保持、测试成功但未保存、保存关闭回概览、重开密码空且留空保留、仅改目录再次成功、真实只读文件权限拒绝且旧文件字节保持均通过。显式seed候选目录后，未带密码项禁用，取消不切活动连接，实际启用后目录落盘。seed只证明原生候选消费者，不冒充实际包导入；转移链仍引用Rust真实文件证据。

连接密码沿用既有macOS本机0600文件/0700目录（Windows DPAPI）的旧后端；credential_ref.backend=keychain是历史元数据，不据它宣称系统钥匙串。内容保护口令独立使用transfer_protection Native keyring。原生探针先用security查唯一自有账户返回44，随后读源码和自有凭据文件证明实际后端；没有读取日常账户。目录保存的updated_at依法更新，改正了最初全对象相等的过严断言，稳定账户身份与真实密码另核对。自有凭据文件/进程/监听均已清理。权限失败保留系统详细原因，未声称其自定义中文文案。新证据在font-final-candidate-bda7c92d/native-connection-consumers。

独立复核发现最终入口映射还需明确非Dialog命名的UpdateCenter/既有AgentPanel，以及最终字体在其他窗口的布局凭证；同时纠正模型历史“恢复草稿”不可引用混合资源范围恢复。正在补证，不据OTA通过关闭全入口/原型审阅或整体目标。
