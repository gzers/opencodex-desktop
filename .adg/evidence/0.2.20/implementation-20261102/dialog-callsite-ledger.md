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
